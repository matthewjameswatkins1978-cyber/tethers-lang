pub mod dispatch;
mod manifest;
pub mod policy;
pub mod provider;
pub mod resolver;
pub mod trusted_store;

use dispatch::DispatchReadyAction;
use policy::PermissionDecision;
use resolver::ResolvedCapability;
use serde_json::{json, Value};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let engine_path = args.next().ok_or(
        "usage: tethers-reference-host ENGINE REQUEST_JSON [POLICY] [TRAIL_PATH] [EXECUTOR_MODE]",
    )?;
    let request_path = args.next().ok_or(
        "usage: tethers-reference-host ENGINE REQUEST_JSON [POLICY] [TRAIL_PATH] [EXECUTOR_MODE]",
    )?;
    let policy_posture = args.next().unwrap_or_else(|| "allow".to_string());
    let trail_path = args.next();
    let executor_mode = args.next().unwrap_or_else(|| "success".to_string());

    let request: Value = serde_json::from_str(&fs::read_to_string(request_path)?)?;
    let mut response = call_engine(&engine_path, &request)?;

    if response.get("status") == Some(&Value::String("matched".into())) {
        // --- Wire through the full Columbo pipeline ---

        // 1. Build, verify, and admit a demo manifest for lantern.task.record.
        let mut manifest_json = json!({
            "manifest_format_version": "1.0",
            "capability_name": "lantern.task.record",
            "capability_version": 1,
            "title": "Record a task",
            "description": "Record a task in Lantern Keeper.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "project": { "type": "string" },
                    "task": { "type": "string" }
                },
                "required": ["project", "task"],
                "additionalProperties": false
            },
            "output_schema": {
                "type": "object",
                "properties": { "status": { "type": "string" } },
                "required": ["status"]
            },
            "effects": ["lantern.write"],
            "permission_scope": {
                "kind": "path_prefix",
                "allowed_prefixes": ["projects/"]
            },
            "reversibility": "compensatable",
            "determinism": "deterministic",
            "idempotency": {
                "mechanism": "argument_key",
                "argument_name": "idempotency_key",
                "key_source": "evaluation_id/action_id"
            },
            "confirmation_policy": {
                "standing_permitted": true,
                "per_call_required": false
            },
            "timeout_ms": 10000,
            "retry_policy": {
                "max_retries": 0,
                "backoff_ms": 500,
                "allowed_on": ["outcome_unknown"],
                "requires_idempotency_proof": false
            },
            "provider": {
                "identity": "lantern-local",
                "display_name": "Lantern Keeper (local mock)",
                "identity_source": "host_configuration",
                "description": "Demo mock executor for 0.1 round-trip."
            },
            "binding": {
                "kind": "mcp",
                "server_name": "lantern",
                "tool_name": "task_record",
                "adapter": null
            }
        });

        let manifest_str = serde_json::to_string(&manifest_json)?;
        let (_, digest) = manifest::canonicalize_and_digest(&manifest_str)
            .map_err(|e| format!("manifest canonicalization failed: {e:?}"))?;
        manifest_json["digest"] = json!(digest);
        let verified = manifest::verify_manifest(&serde_json::to_string(&manifest_json)?)
            .map_err(|e| format!("manifest verification failed: {e:?}"))?;

        // 2. Admit into the trusted store.
        let mut store = trusted_store::TrustedManifestStore::new();
        store
            .insert(verified)
            .map_err(|e| format!("store insertion failed: {e:?}"))?;

        // 3. Report provider availability.
        let availability = resolver::ProviderAvailability::from_identities(["lantern-local"]);

        // 4. Resolve the capability.
        let resolved = resolver::resolve_capability(
            &store,
            &availability,
            "lantern.task.record",
            1,
            Some("lantern-local"),
        )
        .map_err(|e| format!("capability resolution failed: {e:?}"))?;

        // 5. Evaluate policy — posture from CLI, defaults to Allow.
        let requirements = vec![policy::CapabilityRequirement::new("lantern.task.record", 1)];
        let rule = match policy_posture.as_str() {
            "allow" => policy::PolicyRule::Allow,
            "deny" => policy::PolicyRule::Deny,
            "ask" => policy::PolicyRule::Ask,
            other => return Err(format!("unknown policy posture: {other}").into()),
        };
        let host_policy = policy::HostLocalPolicy::new(rule);
        let decision = policy::evaluate_permission_resolved(&requirements, &resolved, &host_policy);

        // 6. Open file-backed durable Trail for intent recording.
        //    When no explicit path is supplied, use a temporary directory
        //    so repeated runs do not dirty the repository.
        let trail_path = trail_path.unwrap_or_else(|| {
            let tmp = std::env::temp_dir()
                .join("tethers-demo-trail")
                .join("trail.jsonl");
            tmp.to_string_lossy().into_owned()
        });
        if let Some(parent) = PathBuf::from(&trail_path).parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file_trail = dispatch::FileTrail::open(&trail_path)?;

        // 7. Select executor by mode, then dispatch through the proof boundary.
        match executor_mode.as_str() {
            "success" => {
                let mut executor = MockExecutor::new();
                authorise_and_execute(
                    &mut response,
                    decision,
                    &resolved,
                    &mut file_trail,
                    &mut executor,
                )?;
            }
            "fail" => {
                let mut executor = FailingExecutor;
                authorise_and_execute(
                    &mut response,
                    decision,
                    &resolved,
                    &mut file_trail,
                    &mut executor,
                )?;
            }
            other => return Err(format!("unknown executor mode: {other}").into()),
        }
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

// ---------------------------------------------------------------------------
// Capability executor — effect boundary
// ---------------------------------------------------------------------------

/// A host-installed executor that carries out one capability invocation.
///
/// The `execute` method requires `&DispatchReadyAction`, which is only
/// constructable via `dispatch::prepare_and_record()` after successful
/// durable intent recording.  There is no production path to `execute()`
/// without a genuine readiness token.
trait CapabilityExecutor {
    /// Honest provider identity.  Callers must verify this matches the
    /// resolved capability's `provider_identity()` before invoking
    /// `execute()`.
    fn provider_identity(&self) -> &str;

    /// Execute the capability Action described by `ready`.
    ///
    /// The executor receives the exact capability name, version,
    /// provider identity, manifest digest, arguments, and stable
    /// execution/action identifiers from the readiness token.  It must
    /// not use independently supplied identity fields.
    fn execute(&mut self, ready: &DispatchReadyAction) -> Result<Value, String>;
}

// ---------------------------------------------------------------------------
// Mock executor (demo — always succeeds for lantern.task.record)
// ---------------------------------------------------------------------------

struct MockExecutor {
    completed: HashSet<String>,
}

impl MockExecutor {
    fn new() -> Self {
        Self {
            completed: HashSet::new(),
        }
    }
}

impl CapabilityExecutor for MockExecutor {
    fn provider_identity(&self) -> &str {
        "lantern-local"
    }

    fn execute(&mut self, ready: &DispatchReadyAction) -> Result<Value, String> {
        let idempotency_key = format!("{}/{}", ready.execution_id().0, ready.action_id().0);

        if self.completed.contains(&idempotency_key) {
            return Ok(json!({"status": "already_completed"}));
        }

        let capability = ready.capability_name();
        let arguments = ready.arguments();

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
        self.completed.insert(idempotency_key);
        Ok(result)
    }
}

// ---------------------------------------------------------------------------
// Failing executor (test/demo — always fails after dispatch readiness)
// ---------------------------------------------------------------------------

/// A deterministic failing executor for testing the action_failed path.
///
/// Uses the same provider identity as MockExecutor.  Must receive a
/// genuine `&DispatchReadyAction` before returning `Err` — the compiler
/// enforces this.  No bypass around manifest verification, trusted-store
/// admission, policy evaluation, or `prepare_and_record()`.
struct FailingExecutor;

impl CapabilityExecutor for FailingExecutor {
    fn provider_identity(&self) -> &str {
        "lantern-local"
    }

    fn execute(&mut self, _ready: &DispatchReadyAction) -> Result<Value, String> {
        Err("executor failed as requested".to_string())
    }
}

// ---------------------------------------------------------------------------
// authorise_and_execute — the enforced proof boundary
// ---------------------------------------------------------------------------

/// Authorise and execute one Action from the engine response.
///
/// Requires exactly one Action in the Plan.  Every route to the executor
/// passes through `dispatch::prepare_and_record()`, which durably records
/// intent before returning a `DispatchReadyAction` proof token.
///
/// On any preparation failure (Ask, Deny, Unavailable, identity mismatch,
/// write failure, flush failure): zero executor calls occur.
fn authorise_and_execute(
    response: &mut Value,
    decision: PermissionDecision,
    resolved: &ResolvedCapability,
    trail: &mut dyn dispatch::Trail,
    executor: &mut dyn CapabilityExecutor,
) -> Result<(), Box<dyn std::error::Error>> {
    let plan = response.get("plan").ok_or("matched response had no plan")?;
    let actions = plan
        .get("actions")
        .and_then(Value::as_array)
        .ok_or("plan had no actions")?;

    // Exactly one Action is required for the 0.1 demo boundary.
    if actions.len() != 1 {
        return Err(format!("expected exactly one Action in Plan, got {}", actions.len()).into());
    }
    let action = &actions[0];

    let action_id_str = required_str(action, "action_id")?;
    let action_capability = required_str(action, "capability")?;

    // Verify the Action's capability matches the resolved capability.
    if action_capability != resolved.capability_name() {
        return Err(format!(
            "action capability '{}' does not match resolved capability '{}'",
            action_capability,
            resolved.capability_name()
        )
        .into());
    }

    // Verify executor identity matches the resolved provider.
    if executor.provider_identity() != resolved.provider_identity() {
        return Err(format!(
            "executor provider '{}' does not match resolved provider '{}'",
            executor.provider_identity(),
            resolved.provider_identity()
        )
        .into());
    }

    let evaluation_id = response
        .get("evaluation_id")
        .and_then(Value::as_str)
        .ok_or("response had no evaluation_id")?;

    let execution_id = dispatch::ExecutionId(evaluation_id.to_owned());
    let action_id = dispatch::ActionId(action_id_str.to_owned());
    let arguments = action.get("arguments").cloned().unwrap_or(Value::Null);

    let json_trail = response
        .get_mut("trail")
        .and_then(Value::as_array_mut)
        .ok_or("response had no Trail")?;
    let mut sequence = json_trail.len() as u64 + 1;

    // Attempt durable intent recording.
    let ready = match dispatch::prepare_and_record(
        decision,
        resolved,
        execution_id,
        action_id.clone(),
        arguments,
        trail,
    ) {
        Ok(ready) => ready,
        Err(err) => {
            // Zero executor calls.  The durable Trail may contain no bytes,
            // a partial record, or an unconfirmed complete record.
            json_trail.push(trail_entry(
                sequence,
                "authorisation",
                "intent_failed",
                "failed",
                format!("{err:?}"),
                Some(&action_id.0),
            ));
            response["execution_status"] = Value::String("denied".into());
            return Ok(());
        }
    };

    // Intent is durably recorded.  Execute exactly once.
    json_trail.push(trail_entry(
        sequence,
        "execution",
        "action_started",
        "started",
        format!("Started {}", ready.capability_name()),
        Some(&ready.action_id().0),
    ));
    sequence += 1;

    match executor.execute(&ready) {
        Ok(result) => {
            let mut entry = trail_entry(
                sequence,
                "execution",
                "action_completed",
                "succeeded",
                format!("Completed {}", ready.capability_name()),
                Some(&ready.action_id().0),
            );
            entry["result"] = result;
            json_trail.push(entry);
            response["execution_status"] = Value::String("completed".into());
        }
        Err(message) => {
            json_trail.push(trail_entry(
                sequence,
                "execution",
                "action_failed",
                "failed",
                message,
                Some(&ready.action_id().0),
            ));
            response["execution_status"] = Value::String("failed".into());
        }
    }

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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dispatch::{self, ActionId, ExecutionId, RecordingTrail};
    use crate::policy::{self, CapabilityRequirement, HostLocalPolicy, PolicyRule};
    use crate::resolver::{self, ProviderAvailability};
    use crate::trusted_store::TrustedManifestStore;

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn lantern_manifest_json() -> Value {
        json!({
            "manifest_format_version": "1.0",
            "capability_name": "lantern.task.record",
            "capability_version": 1,
            "title": "Record a task",
            "description": "Record a task.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "project": { "type": "string" },
                    "task": { "type": "string" }
                },
                "required": ["project", "task"],
                "additionalProperties": false
            },
            "output_schema": {
                "type": "object",
                "properties": { "status": { "type": "string" } },
                "required": ["status"]
            },
            "effects": ["lantern.write"],
            "permission_scope": {
                "kind": "path_prefix",
                "allowed_prefixes": ["projects/"]
            },
            "reversibility": "compensatable",
            "determinism": "deterministic",
            "idempotency": {
                "mechanism": "argument_key",
                "argument_name": "idempotency_key",
                "key_source": "evaluation_id/action_id"
            },
            "confirmation_policy": {
                "standing_permitted": true,
                "per_call_required": false
            },
            "timeout_ms": 10000,
            "retry_policy": {
                "max_retries": 0,
                "backoff_ms": 500,
                "allowed_on": ["outcome_unknown"],
                "requires_idempotency_proof": false
            },
            "provider": {
                "identity": "lantern-local",
                "display_name": "Lantern Keeper (local mock)",
                "identity_source": "host_configuration",
                "description": "Demo."
            },
            "binding": {
                "kind": "mcp",
                "server_name": "lantern",
                "tool_name": "task_record",
                "adapter": null
            }
        })
    }

    fn verified_lantern() -> crate::manifest::VerifiedManifest {
        let mut m = lantern_manifest_json();
        let (_, digest) = crate::manifest::canonicalize_and_digest(&m.to_string()).unwrap();
        m["digest"] = json!(digest);
        crate::manifest::verify_manifest(&m.to_string()).unwrap()
    }

    fn resolved_lantern() -> (TrustedManifestStore, resolver::ResolvedCapability) {
        let mut store = TrustedManifestStore::new();
        store.insert(verified_lantern()).unwrap();
        let availability = ProviderAvailability::from_identities(["lantern-local"]);
        let resolved = resolver::resolve_capability(
            &store,
            &availability,
            "lantern.task.record",
            1,
            Some("lantern-local"),
        )
        .unwrap();
        (store, resolved)
    }

    fn allow_decision_for(resolved: &resolver::ResolvedCapability) -> PermissionDecision {
        let requirements = vec![CapabilityRequirement::new(
            resolved.capability_name().to_owned(),
            resolved.capability_version(),
        )];
        let policy = HostLocalPolicy::new(PolicyRule::Allow);
        policy::evaluate_permission_resolved(&requirements, resolved, &policy)
    }

    fn make_matched_response(
        evaluation_id: &str,
        action_id: &str,
        capability: &str,
        arguments: Value,
    ) -> Value {
        json!({
            "evaluation_id": evaluation_id,
            "plan": {
                "id": "plan-001",
                "required_effects": ["lantern.write"],
                "actions": [
                    {
                        "action_id": action_id,
                        "idempotency_key": format!("{}/{}", evaluation_id, action_id),
                        "capability": capability,
                        "arguments": arguments
                    }
                ]
            },
            "trail": []
        })
    }

    fn assert_allow(decision: &PermissionDecision, name: &str, version: u32) {
        match decision {
            PermissionDecision::Allow(allowed) => {
                assert_eq!(allowed.capability_name(), name);
                assert_eq!(allowed.capability_version(), version);
            }
            other => panic!("expected Allow, got {other:?}"),
        }
    }

    // -----------------------------------------------------------------------
    // Test 1: Executor receives exact recorded arguments.
    // -----------------------------------------------------------------------

    #[test]
    fn executor_receives_exact_recorded_arguments() {
        let (_store, resolved) = resolved_lantern();
        let decision = allow_decision_for(&resolved);
        let args = json!({"project": "lantern-keeper", "task": "LK-39"});

        let mut trail = RecordingTrail::new();
        let ready = dispatch::prepare_and_record(
            decision,
            &resolved,
            ExecutionId("eval-001".into()),
            ActionId("action_1".into()),
            args.clone(),
            &mut trail,
        )
        .unwrap();

        let mut executor = MockExecutor::new();
        let result = executor.execute(&ready).unwrap();

        assert_eq!(result["status"], "recorded");
        assert_eq!(result["project"], "lantern-keeper");
        assert_eq!(result["task"], "LK-39");

        // Verify the recorded intent matches the arguments the executor received.
        assert_eq!(trail.entries.len(), 1);
        assert_eq!(trail.entries[0].arguments, args);
    }

    // -----------------------------------------------------------------------
    // Test 2: Capability name, version, and provider identity match resolved.
    // -----------------------------------------------------------------------

    #[test]
    fn ready_token_carries_resolved_identity() {
        let (_store, resolved) = resolved_lantern();
        let decision = allow_decision_for(&resolved);

        let mut trail = RecordingTrail::new();
        let ready = dispatch::prepare_and_record(
            decision,
            &resolved,
            ExecutionId("eval-001".into()),
            ActionId("action_1".into()),
            json!({"project": "p", "task": "t"}),
            &mut trail,
        )
        .unwrap();

        assert_eq!(ready.capability_name(), resolved.capability_name());
        assert_eq!(ready.capability_version(), resolved.capability_version());
        assert_eq!(ready.provider_identity(), resolved.provider_identity());
        assert_eq!(ready.manifest_digest(), resolved.manifest_digest());
    }

    // -----------------------------------------------------------------------
    // Test 3: Executor provider identity matches resolved identity.
    // -----------------------------------------------------------------------

    #[test]
    fn executor_provider_identity_matches_resolved() {
        let (_store, resolved) = resolved_lantern();
        assert_eq!(resolved.provider_identity(), "lantern-local");

        let executor = MockExecutor::new();
        assert_eq!(executor.provider_identity(), "lantern-local");
        assert_eq!(executor.provider_identity(), resolved.provider_identity());
    }

    // -----------------------------------------------------------------------
    // Test 4: Permission for capability A cannot authorise capability B.
    // -----------------------------------------------------------------------

    #[test]
    fn permission_for_capability_a_cannot_authorise_capability_b() {
        let (_store, resolved_a) = resolved_lantern(); // lantern.task.record
        let decision_a = allow_decision_for(&resolved_a);
        assert_allow(&decision_a, "lantern.task.record", 1);

        // Create a different resolved capability (same as dispatch test 6 pattern).
        // Build a notes.note.read manifest and try to use lantern.task.record
        // permission with it.
        let mut read_json = json!({
            "manifest_format_version": "1.0",
            "capability_name": "notes.note.read",
            "capability_version": 1,
            "title": "Read",
            "description": "Read.",
            "input_schema": { "type": "object", "properties": { "path": { "type": "string" } }, "required": ["path"], "additionalProperties": false },
            "output_schema": { "type": "object", "properties": { "content": { "type": "string" } }, "required": ["content"] },
            "effects": ["filesystem.read"],
            "permission_scope": { "kind": "path_prefix", "allowed_prefixes": ["projects/"] },
            "reversibility": "reversible",
            "determinism": "deterministic",
            "idempotency": { "mechanism": "none" },
            "confirmation_policy": { "standing_permitted": true, "per_call_required": false },
            "timeout_ms": 5000,
            "retry_policy": { "max_retries": 0, "backoff_ms": 500, "allowed_on": ["outcome_unknown"], "requires_idempotency_proof": false },
            "provider": { "identity": "obsidian-local", "display_name": "Obsidian", "identity_source": "host_configuration", "description": null },
            "binding": { "kind": "mcp", "server_name": "obsidian", "tool_name": "obsidian_read_note", "adapter": null }
        });
        let (_, digest) = crate::manifest::canonicalize_and_digest(&read_json.to_string()).unwrap();
        read_json["digest"] = json!(digest);
        let mut store_b = TrustedManifestStore::new();
        store_b
            .insert(crate::manifest::verify_manifest(&read_json.to_string()).unwrap())
            .unwrap();
        let availability = ProviderAvailability::from_identities(["obsidian-local"]);
        let resolved_b = resolver::resolve_capability(
            &store_b,
            &availability,
            "notes.note.read",
            1,
            Some("obsidian-local"),
        )
        .unwrap();

        // Try to prepare notes.note.read with lantern.task.record permission.
        let mut trail = RecordingTrail::new();
        let err = dispatch::prepare_and_record(
            decision_a,  // lantern.task.record Allow
            &resolved_b, // notes.note.read ResolvedCapability
            ExecutionId("eval-001".into()),
            ActionId("action_1".into()),
            json!({}),
            &mut trail,
        )
        .unwrap_err();

        assert_eq!(
            err,
            dispatch::PrepareError::CapabilityIdentityMismatch {
                allowed_name: "lantern.task.record".into(),
                allowed_version: 1,
                resolved_name: "notes.note.read".into(),
                resolved_version: 1,
            }
        );
        assert!(trail.entries.is_empty());
    }

    // -----------------------------------------------------------------------
    // Test 5: Ask produces zero intent records and zero executor calls.
    // -----------------------------------------------------------------------

    #[test]
    fn ask_policy_produces_zero_effect_calls() {
        let (_store, resolved) = resolved_lantern();
        let requirements = vec![CapabilityRequirement::new("lantern.task.record", 1)];
        let policy = HostLocalPolicy::new(PolicyRule::Ask);
        let decision = policy::evaluate_permission_resolved(&requirements, &resolved, &policy);
        assert_eq!(decision, PermissionDecision::Ask);

        let mut trail = RecordingTrail::new();
        let err = dispatch::prepare_and_record(
            decision,
            &resolved,
            ExecutionId("eval-001".into()),
            ActionId("action_1".into()),
            json!({}),
            &mut trail,
        )
        .unwrap_err();

        assert_eq!(err, dispatch::PrepareError::Ask);
        assert!(trail.entries.is_empty());
    }

    // -----------------------------------------------------------------------
    // Test 6: Deny produces zero intent records and zero executor calls.
    // -----------------------------------------------------------------------

    #[test]
    fn deny_policy_produces_zero_effect_calls() {
        let (_store, resolved) = resolved_lantern();
        let requirements = vec![CapabilityRequirement::new("lantern.task.record", 1)];
        let policy = HostLocalPolicy::new(PolicyRule::Deny);
        let decision = policy::evaluate_permission_resolved(&requirements, &resolved, &policy);
        assert_eq!(decision, PermissionDecision::Deny);

        let mut trail = RecordingTrail::new();
        let err = dispatch::prepare_and_record(
            decision,
            &resolved,
            ExecutionId("eval-001".into()),
            ActionId("action_1".into()),
            json!({}),
            &mut trail,
        )
        .unwrap_err();

        assert_eq!(err, dispatch::PrepareError::Deny);
        assert!(trail.entries.is_empty());
    }

    // -----------------------------------------------------------------------
    // Test 7: Unavailable produces zero intent records and zero executor calls.
    // -----------------------------------------------------------------------

    #[test]
    fn unavailable_produces_zero_effect_calls() {
        let (_store, resolved) = resolved_lantern();
        let mut trail = RecordingTrail::new();
        let err = dispatch::prepare_and_record(
            PermissionDecision::Unavailable,
            &resolved,
            ExecutionId("eval-001".into()),
            ActionId("action_1".into()),
            json!({}),
            &mut trail,
        )
        .unwrap_err();

        assert_eq!(err, dispatch::PrepareError::Unavailable);
        assert!(trail.entries.is_empty());
    }

    // -----------------------------------------------------------------------
    // Test 8: Intent write failure produces zero executor calls.
    // -----------------------------------------------------------------------

    #[test]
    fn intent_write_failure_produces_zero_executor_calls() {
        let (_store, resolved) = resolved_lantern();
        let decision = allow_decision_for(&resolved);

        let mut trail = RecordingTrail::new();
        trail.injected_error = Some(dispatch::TrailError::WriteFailed("disk full".into()));

        let err = dispatch::prepare_and_record(
            decision,
            &resolved,
            ExecutionId("eval-001".into()),
            ActionId("action_1".into()),
            json!({}),
            &mut trail,
        )
        .unwrap_err();

        assert_eq!(
            err,
            dispatch::PrepareError::IntentWriteFailed {
                message: "disk full".into()
            }
        );
    }

    // -----------------------------------------------------------------------
    // Test 9: Intent flush failure produces zero executor calls.
    // -----------------------------------------------------------------------

    #[test]
    fn intent_flush_failure_produces_zero_executor_calls() {
        let (_store, resolved) = resolved_lantern();
        let decision = allow_decision_for(&resolved);

        let mut trail = RecordingTrail::new();
        trail.injected_error = Some(dispatch::TrailError::FlushFailed("sync_data failed".into()));

        let err = dispatch::prepare_and_record(
            decision,
            &resolved,
            ExecutionId("eval-001".into()),
            ActionId("action_1".into()),
            json!({}),
            &mut trail,
        )
        .unwrap_err();

        assert_eq!(
            err,
            dispatch::PrepareError::IntentFlushFailed {
                message: "sync_data failed".into()
            }
        );
    }

    // -----------------------------------------------------------------------
    // Test 10: Execution does not append a second intent record.
    // -----------------------------------------------------------------------

    #[test]
    fn execution_does_not_append_second_intent_record() {
        let (_store, resolved) = resolved_lantern();
        let decision = allow_decision_for(&resolved);

        let mut trail = RecordingTrail::new();
        let ready = dispatch::prepare_and_record(
            decision,
            &resolved,
            ExecutionId("eval-001".into()),
            ActionId("action_1".into()),
            json!({"project": "p", "task": "t"}),
            &mut trail,
        )
        .unwrap();

        // Intent was recorded once.
        assert_eq!(trail.entries.len(), 1);

        let mut executor = MockExecutor::new();
        let _result = executor.execute(&ready).unwrap();

        // Execution does not record a second intent.
        assert_eq!(trail.entries.len(), 1);
        let intent = &trail.entries[0];
        assert_eq!(intent.execution_id, "eval-001");
        assert_eq!(intent.action_id, "action_1");
        assert_eq!(intent.capability_name, "lantern.task.record");
    }

    // -----------------------------------------------------------------------
    // Test 11: Repeated prepare-and-execute is deterministic.
    // -----------------------------------------------------------------------

    #[test]
    fn repeated_prepare_and_execute_is_deterministic() {
        let (_store, resolved) = resolved_lantern();

        for _ in 0..5 {
            let decision = allow_decision_for(&resolved);
            let mut trail = RecordingTrail::new();
            let ready = dispatch::prepare_and_record(
                decision,
                &resolved,
                ExecutionId("eval-001".into()),
                ActionId("action_1".into()),
                json!({"project": "lantern-keeper", "task": "LK-39"}),
                &mut trail,
            )
            .unwrap();

            assert_eq!(ready.capability_name(), "lantern.task.record");
            assert_eq!(ready.capability_version(), 1);
            assert_eq!(trail.entries.len(), 1);

            let mut executor = MockExecutor::new();
            let result = executor.execute(&ready).unwrap();
            assert_eq!(result["status"], "recorded");
            assert_eq!(result["project"], "lantern-keeper");
            assert_eq!(result["task"], "LK-39");
        }
    }

    // -----------------------------------------------------------------------
    // Test 12: Mock executor idempotency — second execution returns
    //          already_completed.
    // -----------------------------------------------------------------------

    #[test]
    fn mock_execution_is_idempotent() {
        let (_store, resolved) = resolved_lantern();
        let decision = allow_decision_for(&resolved);

        let mut trail = RecordingTrail::new();
        let ready = dispatch::prepare_and_record(
            decision,
            &resolved,
            ExecutionId("eval-001".into()),
            ActionId("action_1".into()),
            json!({"project": "lantern-keeper", "task": "LK-39"}),
            &mut trail,
        )
        .unwrap();

        let mut executor = MockExecutor::new();
        let first = executor.execute(&ready).unwrap();
        assert_eq!(first["status"], "recorded");

        let second = executor.execute(&ready).unwrap();
        assert_eq!(second["status"], "already_completed");
    }

    // -----------------------------------------------------------------------
    // Test 13: authorise_and_execute integration — success path.
    // -----------------------------------------------------------------------

    #[test]
    fn authorise_and_execute_success_path() {
        let (_store, resolved) = resolved_lantern();
        let decision = allow_decision_for(&resolved);
        let mut response = make_matched_response(
            "eval-001",
            "action_1",
            "lantern.task.record",
            json!({"project": "lantern-keeper", "task": "LK-39"}),
        );

        let mut trail = RecordingTrail::new();
        let mut executor = MockExecutor::new();
        authorise_and_execute(
            &mut response,
            decision,
            &resolved,
            &mut trail,
            &mut executor,
        )
        .unwrap();

        // Exactly one durable intent record.
        assert_eq!(trail.entries.len(), 1);
        assert_eq!(trail.entries[0].capability_name, "lantern.task.record");
        assert_eq!(
            trail.entries[0].arguments,
            json!({"project": "lantern-keeper", "task": "LK-39"})
        );

        // Response Trail was appended.
        let json_trail = response["trail"].as_array().unwrap();
        assert!(json_trail.len() >= 2); // at least action_started + action_completed
        assert_eq!(response["execution_status"], "completed");
    }

    // -----------------------------------------------------------------------
    // Test 14: Capability mismatch between action and resolved is rejected.
    // -----------------------------------------------------------------------

    #[test]
    fn capability_mismatch_rejected_by_authorise_and_execute() {
        let (_store, resolved) = resolved_lantern();
        let decision = allow_decision_for(&resolved);
        // Action says "wrong.capability" but resolved says "lantern.task.record".
        let mut response = make_matched_response(
            "eval-001",
            "action_1",
            "wrong.capability",
            json!({"project": "x", "task": "y"}),
        );

        let mut trail = RecordingTrail::new();
        let mut executor = MockExecutor::new();
        let result = authorise_and_execute(
            &mut response,
            decision,
            &resolved,
            &mut trail,
            &mut executor,
        );

        assert!(result.is_err());
        // Zero intent records — the call failed before prepare_and_record.
        assert!(trail.entries.is_empty());
    }

    // -----------------------------------------------------------------------
    // Test 15: Provider identity mismatch is rejected before any effect.
    // -----------------------------------------------------------------------

    #[test]
    fn provider_identity_mismatch_rejected() {
        let (_store, resolved) = resolved_lantern();
        let decision = allow_decision_for(&resolved);

        // Use an executor whose provider identity differs from resolved.
        struct OtherExecutor;
        impl CapabilityExecutor for OtherExecutor {
            fn provider_identity(&self) -> &str {
                "other-provider"
            }
            fn execute(&mut self, _ready: &DispatchReadyAction) -> Result<Value, String> {
                panic!("must not be called");
            }
        }

        let mut response = make_matched_response(
            "eval-001",
            "action_1",
            "lantern.task.record",
            json!({"project": "p", "task": "t"}),
        );

        let mut trail = RecordingTrail::new();
        let mut executor = OtherExecutor;
        let result = authorise_and_execute(
            &mut response,
            decision,
            &resolved,
            &mut trail,
            &mut executor,
        );

        assert!(result.is_err());
        assert!(trail.entries.is_empty());
    }

    // -----------------------------------------------------------------------
    // Test 16: Zero actions in plan is rejected.
    // -----------------------------------------------------------------------

    #[test]
    fn zero_actions_rejected() {
        let (_store, resolved) = resolved_lantern();
        let decision = allow_decision_for(&resolved);
        let mut response = json!({
            "evaluation_id": "eval-001",
            "plan": {
                "id": "plan-001",
                "required_effects": ["lantern.write"],
                "actions": []
            },
            "trail": []
        });

        let mut trail = RecordingTrail::new();
        let mut executor = MockExecutor::new();
        let result = authorise_and_execute(
            &mut response,
            decision,
            &resolved,
            &mut trail,
            &mut executor,
        );

        assert!(result.is_err());
        assert!(trail.entries.is_empty());
    }

    // -----------------------------------------------------------------------
    // Test 17: Multiple actions in plan is rejected (0.1 enforces exactly 1).
    // -----------------------------------------------------------------------

    #[test]
    fn multiple_actions_rejected() {
        let (_store, resolved) = resolved_lantern();
        let decision = allow_decision_for(&resolved);
        let mut response = json!({
            "evaluation_id": "eval-001",
            "plan": {
                "id": "plan-001",
                "required_effects": ["lantern.write"],
                "actions": [
                    {
                        "action_id": "action_1",
                        "idempotency_key": "eval-001/action_1",
                        "capability": "lantern.task.record",
                        "arguments": { "project": "a", "task": "b" }
                    },
                    {
                        "action_id": "action_2",
                        "idempotency_key": "eval-001/action_2",
                        "capability": "lantern.task.record",
                        "arguments": { "project": "c", "task": "d" }
                    }
                ]
            },
            "trail": []
        });

        let mut trail = RecordingTrail::new();
        let mut executor = MockExecutor::new();
        let result = authorise_and_execute(
            &mut response,
            decision,
            &resolved,
            &mut trail,
            &mut executor,
        );

        assert!(result.is_err());
        assert!(trail.entries.is_empty());
    }

    // -----------------------------------------------------------------------
    // Test 18: No outcome record or result Anchor appears in the intent.
    // -----------------------------------------------------------------------

    #[test]
    fn intent_contains_no_outcome_or_result_anchor() {
        let (_store, resolved) = resolved_lantern();
        let decision = allow_decision_for(&resolved);

        let mut trail = RecordingTrail::new();
        let _ready = dispatch::prepare_and_record(
            decision,
            &resolved,
            ExecutionId("eval-001".into()),
            ActionId("action_1".into()),
            json!({"project": "p", "task": "t"}),
            &mut trail,
        )
        .unwrap();

        assert_eq!(trail.entries.len(), 1);
        let entry = &trail.entries[0];
        let as_value = serde_json::to_value(entry).unwrap();
        assert!(as_value.get("outcome").is_none());
        assert!(as_value.get("result").is_none());
    }

    // -----------------------------------------------------------------------
    // Test 19: Failing executor path through authorise_and_execute.
    // -----------------------------------------------------------------------

    #[test]
    fn failing_executor_produces_action_failed() {
        let (_store, resolved) = resolved_lantern();
        let decision = allow_decision_for(&resolved);
        let mut response = make_matched_response(
            "eval-001",
            "action_1",
            "lantern.task.record",
            json!({"project": "lantern-keeper", "task": "LK-39"}),
        );

        let mut trail = RecordingTrail::new();
        let mut executor = FailingExecutor;
        authorise_and_execute(
            &mut response,
            decision,
            &resolved,
            &mut trail,
            &mut executor,
        )
        .unwrap();

        // Exactly one durable intent record before invocation.
        assert_eq!(trail.entries.len(), 1);
        assert_eq!(trail.entries[0].capability_name, "lantern.task.record");

        // Response Trail has action_started + action_failed.
        let json_trail = response["trail"].as_array().unwrap();
        let start_count = json_trail
            .iter()
            .filter(|e| e["kind"] == "action_started")
            .count();
        let fail_count = json_trail
            .iter()
            .filter(|e| e["kind"] == "action_failed")
            .count();
        let complete_count = json_trail
            .iter()
            .filter(|e| e["kind"] == "action_completed")
            .count();

        assert_eq!(start_count, 1);
        assert_eq!(fail_count, 1);
        assert_eq!(complete_count, 0);
        assert_eq!(response["execution_status"], "failed");

        // Verify the failure message.
        let failed = json_trail
            .iter()
            .find(|e| e["kind"] == "action_failed")
            .unwrap();
        assert_eq!(failed["message"], "executor failed as requested");
        assert_eq!(failed["action_id"], "action_1");
        assert_eq!(failed["phase"], "execution");
        assert_eq!(failed["outcome"], "failed");
    }
}
