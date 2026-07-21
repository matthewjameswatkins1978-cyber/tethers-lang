mod manifest;
pub mod provider;
pub mod trusted_store;

use serde_json::{json, Value};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let engine_path = args
        .next()
        .ok_or("usage: tethers-reference-host ENGINE REQUEST_JSON")?;
    let request_path = args
        .next()
        .ok_or("usage: tethers-reference-host ENGINE REQUEST_JSON")?;

    let request: Value = serde_json::from_str(&fs::read_to_string(request_path)?)?;
    let mut response = call_engine(&engine_path, &request)?;

    if response.get("status") == Some(&Value::String("matched".into())) {
        let policy = HostPolicy::new(["lantern.write"]);
        let mut executor = MockExecutor::default();
        authorise_and_execute(&mut response, &policy, &mut executor)?;
    }

    println!("{}", serde_json::to_string_pretty(&response)?);
    Ok(())
}

fn call_engine(engine_path: &str, request: &Value) -> Result<Value, Box<dyn std::error::Error>> {
    let mut child = Command::new(engine_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    {
        let mut stdin = child.stdin.take().ok_or("engine stdin was unavailable")?;
        writeln!(stdin, "{}", serde_json::to_string(request)?)?;
    }

    let stdout = child.stdout.take().ok_or("engine stdout was unavailable")?;
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();
    reader.read_line(&mut line)?;
    let _ = child.wait();

    if line.trim().is_empty() {
        return Err("engine returned no response".into());
    }
    Ok(serde_json::from_str(&line)?)
}

struct HostPolicy {
    allowed_effects: HashSet<String>,
}

impl HostPolicy {
    fn new<const N: usize>(effects: [&str; N]) -> Self {
        Self {
            allowed_effects: effects.into_iter().map(str::to_owned).collect(),
        }
    }

    fn permits(&self, effect: &str) -> bool {
        self.allowed_effects.contains(effect)
    }
}

trait CapabilityExecutor {
    fn execute(
        &mut self,
        capability: &str,
        arguments: &Value,
        idempotency_key: &str,
    ) -> Result<Value, String>;
}

#[derive(Default)]
struct MockExecutor {
    completed: HashSet<String>,
}

impl CapabilityExecutor for MockExecutor {
    fn execute(
        &mut self,
        capability: &str,
        arguments: &Value,
        idempotency_key: &str,
    ) -> Result<Value, String> {
        if self.completed.contains(idempotency_key) {
            return Ok(json!({"status": "already_completed"}));
        }

        let result = match capability {
            "lantern.task.record" => {
                let project = arguments
                    .get("project")
                    .and_then(Value::as_str)
                    .ok_or("lantern.task.record requires string argument project")?;
                let task = arguments
                    .get("task")
                    .and_then(Value::as_str)
                    .ok_or("lantern.task.record requires string argument task")?;
                json!({
                    "status": "recorded",
                    "project": project,
                    "task": task
                })
            }
            other => return Err(format!("no host executor is installed for {other}")),
        };

        // A production host persists this key atomically with the external effect.
        self.completed.insert(idempotency_key.to_owned());
        Ok(result)
    }
}

fn authorise_and_execute(
    response: &mut Value,
    policy: &HostPolicy,
    executor: &mut dyn CapabilityExecutor,
) -> Result<(), Box<dyn std::error::Error>> {
    let plan = response.get("plan").ok_or("matched response had no plan")?;
    let effects = plan
        .get("required_effects")
        .and_then(Value::as_array)
        .ok_or("plan had no required_effects")?;
    let denied: Vec<String> = effects
        .iter()
        .filter_map(Value::as_str)
        .filter(|effect| !policy.permits(effect))
        .map(str::to_owned)
        .collect();

    let actions = plan
        .get("actions")
        .and_then(Value::as_array)
        .ok_or("plan had no actions")?
        .clone();

    let trail = response
        .get_mut("trail")
        .and_then(Value::as_array_mut)
        .ok_or("response had no Trail")?;
    let mut sequence = trail.len() as u64 + 1;

    if !denied.is_empty() {
        trail.push(trail_entry(
            sequence,
            "authorisation",
            "plan_denied",
            "denied",
            format!("Host denied effects: {}", denied.join(", ")),
            None,
        ));
        response["execution_status"] = Value::String("denied".into());
        return Ok(());
    }

    trail.push(trail_entry(
        sequence,
        "authorisation",
        "plan_authorised",
        "accepted",
        "Host authorised all required effects".into(),
        None,
    ));
    sequence += 1;

    for action in actions {
        let action_id = required_str(&action, "action_id")?;
        let idempotency_key = required_str(&action, "idempotency_key")?;
        let capability = required_str(&action, "capability")?;
        let arguments = action.get("arguments").ok_or("action had no arguments")?;

        trail.push(trail_entry(
            sequence,
            "execution",
            "action_started",
            "started",
            format!("Started {capability}"),
            Some(action_id),
        ));
        sequence += 1;

        match executor.execute(capability, arguments, idempotency_key) {
            Ok(result) => {
                let mut entry = trail_entry(
                    sequence,
                    "execution",
                    "action_completed",
                    "succeeded",
                    format!("Completed {capability}"),
                    Some(action_id),
                );
                entry["result"] = result;
                trail.push(entry);
                sequence += 1;
            }
            Err(message) => {
                trail.push(trail_entry(
                    sequence,
                    "execution",
                    "action_failed",
                    "failed",
                    message,
                    Some(action_id),
                ));
                response["execution_status"] = Value::String("failed".into());
                return Ok(());
            }
        }
    }

    response["execution_status"] = Value::String("completed".into());
    Ok(())
}

fn required_str<'a>(value: &'a Value, field: &str) -> Result<&'a str, Box<dyn std::error::Error>> {
    value
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("expected string field {field}").into())
}

fn trail_entry(
    sequence: u64,
    phase: &str,
    kind: &str,
    outcome: &str,
    message: String,
    action_id: Option<&str>,
) -> Value {
    let timestamp_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    let mut entry = json!({
        "sequence": sequence,
        "phase": phase,
        "kind": kind,
        "outcome": outcome,
        "message": message,
        "host_timestamp_unix_ms": timestamp_ms
    });
    if let Some(value) = action_id {
        entry["action_id"] = Value::String(value.to_owned());
    }
    entry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_requires_every_effect() {
        let policy = HostPolicy::new(["lantern.write"]);
        assert!(policy.permits("lantern.write"));
        assert!(!policy.permits("network.write"));
    }

    #[test]
    fn mock_execution_is_idempotent() {
        let mut executor = MockExecutor::default();
        let args = json!({"project": "lantern-keeper", "task": "LK-39"});
        let first = executor
            .execute("lantern.task.record", &args, "eval/action")
            .unwrap();
        let second = executor
            .execute("lantern.task.record", &args, "eval/action")
            .unwrap();
        assert_eq!(first["status"], "recorded");
        assert_eq!(second["status"], "already_completed");
    }
}
