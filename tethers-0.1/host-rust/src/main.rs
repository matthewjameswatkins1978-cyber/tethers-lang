pub mod dispatch;
mod manifest;
pub mod policy;
pub mod provider;
pub mod resolver;
mod result_anchor;
pub mod stdio_provider;
pub mod trusted_store;
mod validation;

use dispatch::DispatchReadyAction;
use policy::PermissionDecision;
use resolver::ResolvedCapability;
use result_anchor::{ResultAnchor, ResultAnchorKind};
use serde_json::{json, Value};
use std::collections::HashMap;
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

    let mut request: Value = serde_json::from_str(&fs::read_to_string(request_path)?)?;

    // --- Build the approved capability view before planner evaluation ---

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
            "properties": {
                "status": { "type": "string" },
                "project": { "type": "string" },
                "task": { "type": "string" }
            },
            "required": ["status"],
            "additionalProperties": false
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

    // 4. Project before evaluation and inject bridge pins into request capabilities.
    inject_bridge_projection_into_request(&mut request, &store, &availability)?;

    let mut response = call_engine(&engine_path, &request)?;

    if response.get("status") == Some(&Value::String("matched".into())) {
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

        // Extract the original input event ID for Result Anchor correlation.
        let original_event_id = request["event"]["id"]
            .as_str()
            .ok_or("request event had no id")?;

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
                    original_event_id,
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
                    original_event_id,
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

/// Planner capability versions are explicitly represented as `<major>.0.0`.
///
/// Bridge-backed projection pinning converts trusted manifest major versions
/// to planner strings using this exact rule and rejects other formats.
fn planner_version_from_manifest_major(major: u32) -> String {
    format!("{major}.0.0")
}

fn manifest_major_from_planner_version(
    planner_version: &str,
) -> Result<u32, Box<dyn std::error::Error>> {
    let parts: Vec<&str> = planner_version.split('.').collect();
    if parts.len() != 3 {
        return Err(format!(
            "unsupported planner capability version '{}': expected '<major>.0.0'",
            planner_version
        )
        .into());
    }

    if parts[1] != "0" || parts[2] != "0" {
        return Err(format!(
            "unsupported planner capability version '{}': only '<major>.0.0' is bridge-mappable",
            planner_version
        )
        .into());
    }

    let major = parts[0].parse::<u32>().map_err(|_| {
        format!(
            "unsupported planner capability version '{}': major is not a positive integer",
            planner_version
        )
    })?;
    if major == 0 {
        return Err(format!(
            "unsupported planner capability version '{}': major is not a positive integer",
            planner_version
        )
        .into());
    }
    Ok(major)
}

/// Inject approved bridge projection pins into request capabilities before
/// planner evaluation.
///
/// For each planner capability whose version is representable by the explicit
/// `<major>.0.0` rule, projection supplies:
/// - exact major version (`bridge_capability_version`),
/// - opaque manifest digest (`manifest_digest`),
/// - provider identity (`bridge_provider_identity`).
///
/// Non-bridge capability entries remain unchanged, preserving existing fixture
/// compatibility.
fn inject_bridge_projection_into_request(
    request: &mut Value,
    store: &trusted_store::TrustedManifestStore,
    availability: &resolver::ProviderAvailability,
) -> Result<(), Box<dyn std::error::Error>> {
    let capabilities = request
        .get_mut("capabilities")
        .and_then(Value::as_array_mut)
        .ok_or("request capabilities must be an array")?;

    let mut requirements = Vec::<(String, u32)>::new();
    for capability in capabilities.iter() {
        let Some(obj) = capability.as_object() else {
            continue;
        };
        let Some(name) = obj.get("name").and_then(Value::as_str) else {
            continue;
        };
        let Some(version) = obj.get("version").and_then(Value::as_str) else {
            continue;
        };

        if let Ok(major) = manifest_major_from_planner_version(version) {
            requirements.push((name.to_owned(), major));
        }
    }

    let projection = resolver::project_capabilities(&requirements, store, availability);
    let projected_by_identity: HashMap<(String, u32), resolver::ProjectedCapability> = projection
        .into_iter()
        .map(|p| ((p.capability_name.clone(), p.capability_version), p))
        .collect();

    for capability in capabilities.iter_mut() {
        let Some(obj) = capability.as_object_mut() else {
            continue;
        };
        let Some(name) = obj.get("name").and_then(Value::as_str) else {
            continue;
        };
        let Some(version) = obj.get("version").and_then(Value::as_str) else {
            continue;
        };

        let major = match manifest_major_from_planner_version(version) {
            Ok(v) => v,
            Err(_) => continue,
        };

        if let Some(projected) = projected_by_identity.get(&(name.to_owned(), major)) {
            obj.insert(
                "version".to_owned(),
                Value::String(planner_version_from_manifest_major(
                    projected.capability_version,
                )),
            );
            obj.insert(
                "bridge_capability_version".to_owned(),
                Value::Number(projected.capability_version.into()),
            );
            obj.insert(
                "manifest_digest".to_owned(),
                Value::String(projected.manifest_digest.clone()),
            );
            obj.insert(
                "bridge_provider_identity".to_owned(),
                Value::String(projected.provider_identity.clone()),
            );
        }
    }

    Ok(())
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
    original_event_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    authorise_and_execute_inner(
        response,
        decision,
        resolved,
        trail,
        executor,
        original_event_id,
        true,
    )
}

#[cfg(test)]
fn authorise_and_execute_without_bridge_pins(
    response: &mut Value,
    decision: PermissionDecision,
    resolved: &ResolvedCapability,
    trail: &mut dyn dispatch::Trail,
    executor: &mut dyn CapabilityExecutor,
    original_event_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    authorise_and_execute_inner(
        response,
        decision,
        resolved,
        trail,
        executor,
        original_event_id,
        false,
    )
}

fn authorise_and_execute_inner(
    response: &mut Value,
    decision: PermissionDecision,
    resolved: &ResolvedCapability,
    trail: &mut dyn dispatch::Trail,
    executor: &mut dyn CapabilityExecutor,
    original_event_id: &str,
    bridge_pins_required: bool,
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

    // Bridge-backed actions may pin digest/version/provider from planning.
    // The host must fail closed when the currently verified binding differs.
    verify_action_bridge_pins(action, resolved, bridge_pins_required)?;

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
        .ok_or("response had no evaluation_id")?
        .to_owned();

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

    let timestamp_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default();

    let result_anchor = match executor.execute(&ready) {
        Ok(result) => {
            // Validate result against the capability's output_schema.
            let output_schema = &ready.verified_manifest().manifest().output_schema;
            match validation::validate_output(output_schema, &result) {
                Ok(()) => {
                    // Durably record succeeded outcome.
                    let outcome = dispatch::OutcomeEntry {
                        execution_id: ready.execution_id().0.clone(),
                        action_id: ready.action_id().0.clone(),
                        status: "succeeded".into(),
                        result: Some(result.clone()),
                        error_message: None,
                        timestamp_unix_ms: timestamp_ms,
                    };
                    match trail.append_outcome(&outcome) {
                        Ok(()) => {}
                        Err(e) => {
                            json_trail.push(trail_entry(
                                sequence,
                                "execution",
                                "audit_failure",
                                "failed",
                                format!("outcome write failed after succeeded Action: {e:?}"),
                                Some(&ready.action_id().0),
                            ));
                            sequence += 1;
                        }
                    }

                    let mut entry = trail_entry(
                        sequence,
                        "execution",
                        "action_completed",
                        "succeeded",
                        format!("Completed {}", ready.capability_name()),
                        Some(&ready.action_id().0),
                    );
                    // Clone before moving into the trail entry so the anchor
                    // receives the same validated result.
                    let result_for_anchor = result.clone();
                    entry["result"] = result;
                    json_trail.push(entry);
                    response["execution_status"] = Value::String("completed".into());

                    Some(ResultAnchor::new(
                        ResultAnchorKind::Succeeded(result_for_anchor),
                        &evaluation_id,
                        &action_id.0,
                        ready.capability_name(),
                        ready.capability_version(),
                        ready.manifest_digest(),
                        ready.provider_identity(),
                        timestamp_ms,
                        original_event_id,
                    ))
                }
                Err(validation_err) => {
                    let message = format!("output validation failed: {}", validation_err.message);
                    // Durably record failed outcome.
                    let outcome = dispatch::OutcomeEntry {
                        execution_id: ready.execution_id().0.clone(),
                        action_id: ready.action_id().0.clone(),
                        status: "failed".into(),
                        result: None,
                        error_message: Some(message.clone()),
                        timestamp_unix_ms: timestamp_ms,
                    };
                    match trail.append_outcome(&outcome) {
                        Ok(()) => {}
                        Err(e) => {
                            json_trail.push(trail_entry(
                                sequence,
                                "execution",
                                "audit_failure",
                                "failed",
                                format!("outcome write failed after validation failure: {e:?}"),
                                Some(&ready.action_id().0),
                            ));
                            sequence += 1;
                        }
                    }

                    json_trail.push(trail_entry(
                        sequence,
                        "execution",
                        "action_failed",
                        "failed",
                        message.clone(),
                        Some(&ready.action_id().0),
                    ));
                    response["execution_status"] = Value::String("failed".into());

                    Some(ResultAnchor::new(
                        ResultAnchorKind::ResultValidationFailed(message),
                        &evaluation_id,
                        &action_id.0,
                        ready.capability_name(),
                        ready.capability_version(),
                        ready.manifest_digest(),
                        ready.provider_identity(),
                        timestamp_ms,
                        original_event_id,
                    ))
                }
            }
        }
        Err(message) => {
            // Durably record outcome before appending to response Trail.
            let outcome = dispatch::OutcomeEntry {
                execution_id: ready.execution_id().0.clone(),
                action_id: ready.action_id().0.clone(),
                status: "failed".into(),
                result: None,
                error_message: Some(message.clone()),
                timestamp_unix_ms: timestamp_ms,
            };
            match trail.append_outcome(&outcome) {
                Ok(()) => {}
                Err(e) => {
                    json_trail.push(trail_entry(
                        sequence,
                        "execution",
                        "audit_failure",
                        "failed",
                        format!("outcome write failed after failed Action: {e:?}"),
                        Some(&ready.action_id().0),
                    ));
                    sequence += 1;
                }
            }

            json_trail.push(trail_entry(
                sequence,
                "execution",
                "action_failed",
                "failed",
                message.clone(),
                Some(&ready.action_id().0),
            ));
            response["execution_status"] = Value::String("failed".into());

            Some(ResultAnchor::new(
                ResultAnchorKind::ProviderError(message),
                &evaluation_id,
                &action_id.0,
                ready.capability_name(),
                ready.capability_version(),
                ready.manifest_digest(),
                ready.provider_identity(),
                timestamp_ms,
                original_event_id,
            ))
        }
    };

    // Attach the Result Anchor to the response.  No Result Anchor is
    // created for the preparation-failure path (Ask, Deny, Unavailable,
    // identity mismatch, intent-write failure), which returns early above.
    if let Some(anchor) = result_anchor {
        response["result_anchor"] = serde_json::to_value(&anchor)?;
    }

    Ok(())
}

fn required_str<'a>(value: &'a Value, field: &str) -> Result<&'a str, Box<dyn std::error::Error>> {
    value
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("expected string field {field}").into())
}

fn verify_action_bridge_pins(
    action: &Value,
    resolved: &ResolvedCapability,
    bridge_pins_required: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let has_digest = action.get("manifest_digest").is_some();
    let has_version = action.get("bridge_capability_version").is_some();
    let has_provider = action.get("bridge_provider_identity").is_some();
    let has_any_bridge_pin = has_digest || has_version || has_provider;
    let has_complete_bridge_pins = has_digest && has_version && has_provider;

    if (bridge_pins_required || has_any_bridge_pin) && !has_complete_bridge_pins {
        return Err(
            "bridge Action requires manifest_digest, bridge_capability_version, and bridge_provider_identity together"
                .into(),
        );
    }

    if !has_complete_bridge_pins {
        return Ok(());
    }

    if let Some(pinned_digest) = action.get("manifest_digest") {
        let pinned_digest = pinned_digest
            .as_str()
            .ok_or("manifest_digest must be a string when present")?;
        if pinned_digest != resolved.manifest_digest() {
            return Err(format!(
                "stale plan digest for '{}' v{}: planned '{}', current '{}'",
                resolved.capability_name(),
                resolved.capability_version(),
                pinned_digest,
                resolved.manifest_digest()
            )
            .into());
        }
    }

    if let Some(pinned_version) = action.get("bridge_capability_version") {
        let pinned_version_u64 = pinned_version
            .as_u64()
            .ok_or("bridge_capability_version must be an integer when present")?;
        let pinned_version = u32::try_from(pinned_version_u64)
            .map_err(|_| "bridge_capability_version exceeds u32 range")?;
        if pinned_version == 0 {
            return Err("bridge_capability_version must be a positive integer".into());
        }
        if pinned_version != resolved.capability_version() {
            return Err(format!(
                "stale plan capability version for '{}': planned {}, current {}",
                resolved.capability_name(),
                pinned_version,
                resolved.capability_version()
            )
            .into());
        }

        let planner_version = required_str(action, "capability_version")?;
        let mapped_version = manifest_major_from_planner_version(planner_version)?;
        if mapped_version != pinned_version {
            return Err(format!(
                "Action capability_version '{}' does not match bridge_capability_version {}",
                planner_version, pinned_version
            )
            .into());
        }
    }

    if let Some(pinned_provider) = action.get("bridge_provider_identity") {
        let pinned_provider = pinned_provider
            .as_str()
            .ok_or("bridge_provider_identity must be a string when present")?;
        if pinned_provider != resolved.provider_identity() {
            return Err(format!(
                "stale plan provider for '{}' v{}: planned '{}', current '{}'",
                resolved.capability_name(),
                resolved.capability_version(),
                pinned_provider,
                resolved.provider_identity()
            )
            .into());
        }
    }

    Ok(())
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
                "properties": {
                    "status": { "type": "string" },
                    "project": { "type": "string" },
                    "task": { "type": "string" }
                },
                "required": ["status"],
                "additionalProperties": false
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

    fn resolved_lantern_with_manifest(
        manifest_json: Value,
    ) -> (TrustedManifestStore, resolver::ResolvedCapability) {
        let mut manifest_json = manifest_json;
        let (_, digest) =
            crate::manifest::canonicalize_and_digest(&manifest_json.to_string()).unwrap();
        manifest_json["digest"] = json!(digest);
        let verified = crate::manifest::verify_manifest(&manifest_json.to_string()).unwrap();

        let mut store = TrustedManifestStore::new();
        store.insert(verified).unwrap();
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

    fn make_bridge_matched_response(resolved: &resolver::ResolvedCapability) -> Value {
        let mut response = make_matched_response(
            "eval-bridge-001",
            "action_1",
            resolved.capability_name(),
            json!({"project": "p", "task": "t"}),
        );
        let action = response["plan"]["actions"][0].as_object_mut().unwrap();
        action.insert(
            "capability_version".to_owned(),
            Value::String(planner_version_from_manifest_major(
                resolved.capability_version(),
            )),
        );
        action.insert(
            "bridge_capability_version".to_owned(),
            Value::from(resolved.capability_version()),
        );
        action.insert(
            "manifest_digest".to_owned(),
            Value::String(resolved.manifest_digest().to_owned()),
        );
        action.insert(
            "bridge_provider_identity".to_owned(),
            Value::String(resolved.provider_identity().to_owned()),
        );
        response
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
        trail.injected_intent_error = Some(dispatch::TrailError::WriteFailed("disk full".into()));

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
        trail.injected_intent_error =
            Some(dispatch::TrailError::FlushFailed("sync_data failed".into()));

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
        authorise_and_execute_without_bridge_pins(
            &mut response,
            decision,
            &resolved,
            &mut trail,
            &mut executor,
            "evt_input_001",
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
    fn capability_mismatch_rejected_by_authorise_and_execute_without_bridge_pins() {
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
        let result = authorise_and_execute_without_bridge_pins(
            &mut response,
            decision,
            &resolved,
            &mut trail,
            &mut executor,
            "evt_input_001",
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
        let result = authorise_and_execute_without_bridge_pins(
            &mut response,
            decision,
            &resolved,
            &mut trail,
            &mut executor,
            "evt_input_001",
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
        let result = authorise_and_execute_without_bridge_pins(
            &mut response,
            decision,
            &resolved,
            &mut trail,
            &mut executor,
            "evt_input_001",
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
        let result = authorise_and_execute_without_bridge_pins(
            &mut response,
            decision,
            &resolved,
            &mut trail,
            &mut executor,
            "evt_input_001",
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
        authorise_and_execute_without_bridge_pins(
            &mut response,
            decision,
            &resolved,
            &mut trail,
            &mut executor,
            "evt_input_001",
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

    // -----------------------------------------------------------------------
    // Test 20: authorise_and_execute writes a succeeded outcome to the
    //          durable Trail before appending action_completed.
    // -----------------------------------------------------------------------

    #[test]
    fn authorise_and_execute_writes_succeeded_outcome() {
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
        authorise_and_execute_without_bridge_pins(
            &mut response,
            decision,
            &resolved,
            &mut trail,
            &mut executor,
            "evt_input_001",
        )
        .unwrap();

        assert_eq!(trail.entries.len(), 1);
        assert_eq!(trail.outcome_entries.len(), 1);

        let outcome = &trail.outcome_entries[0];
        assert_eq!(outcome.execution_id, "eval-001");
        assert_eq!(outcome.action_id, "action_1");
        assert_eq!(outcome.status, "succeeded");
        assert_eq!(
            outcome.result,
            Some(json!({"status": "recorded", "project": "lantern-keeper", "task": "LK-39"}))
        );
        assert_eq!(outcome.error_message, None);
        assert_eq!(response["execution_status"], "completed");

        let json_trail = response["trail"].as_array().unwrap();
        let audit_count = json_trail
            .iter()
            .filter(|e| e["kind"] == "audit_failure")
            .count();
        assert_eq!(audit_count, 0);
    }

    // -----------------------------------------------------------------------
    // Test 21: authorise_and_execute writes a failed outcome to the
    //          durable Trail before appending action_failed.
    // -----------------------------------------------------------------------

    #[test]
    fn authorise_and_execute_writes_failed_outcome() {
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
        authorise_and_execute_without_bridge_pins(
            &mut response,
            decision,
            &resolved,
            &mut trail,
            &mut executor,
            "evt_input_001",
        )
        .unwrap();

        assert_eq!(trail.entries.len(), 1);
        assert_eq!(trail.outcome_entries.len(), 1);

        let outcome = &trail.outcome_entries[0];
        assert_eq!(outcome.execution_id, "eval-001");
        assert_eq!(outcome.action_id, "action_1");
        assert_eq!(outcome.status, "failed");
        assert_eq!(outcome.result, None);
        assert_eq!(
            outcome.error_message,
            Some("executor failed as requested".into())
        );
        assert_eq!(response["execution_status"], "failed");
    }

    // -----------------------------------------------------------------------
    // Test 22: Outcome write failure after executor success preserves
    //          execution_status "completed" and appends audit_failure.
    // -----------------------------------------------------------------------

    #[test]
    fn outcome_write_failure_after_success_preserves_status_and_audits() {
        let (_store, resolved) = resolved_lantern();
        let decision = allow_decision_for(&resolved);
        let mut response = make_matched_response(
            "eval-001",
            "action_1",
            "lantern.task.record",
            json!({"project": "lantern-keeper", "task": "LK-39"}),
        );

        let mut trail = RecordingTrail::new();
        trail.injected_outcome_error = Some(dispatch::TrailError::FlushFailed(
            "outcome sync_data failed".into(),
        ));
        let mut executor = MockExecutor::new();
        authorise_and_execute_without_bridge_pins(
            &mut response,
            decision,
            &resolved,
            &mut trail,
            &mut executor,
            "evt_input_001",
        )
        .unwrap();

        assert_eq!(trail.entries.len(), 1);
        assert!(trail.outcome_entries.is_empty());

        assert_eq!(response["execution_status"], "completed");

        let json_trail = response["trail"].as_array().unwrap();
        let audit_count = json_trail
            .iter()
            .filter(|e| e["kind"] == "audit_failure")
            .count();
        assert_eq!(audit_count, 1);
        let audit = json_trail
            .iter()
            .find(|e| e["kind"] == "audit_failure")
            .unwrap();
        assert_eq!(audit["action_id"], "action_1");
        assert!(audit["message"]
            .as_str()
            .unwrap()
            .contains("outcome write failed after succeeded Action"));
    }

    // -----------------------------------------------------------------------
    // Test 23: Outcome write failure after executor failure preserves
    //          execution_status "failed" and appends audit_failure.
    // -----------------------------------------------------------------------

    #[test]
    fn outcome_write_failure_after_failure_preserves_status_and_audits() {
        let (_store, resolved) = resolved_lantern();
        let decision = allow_decision_for(&resolved);
        let mut response = make_matched_response(
            "eval-001",
            "action_1",
            "lantern.task.record",
            json!({"project": "lantern-keeper", "task": "LK-39"}),
        );

        let mut trail = RecordingTrail::new();
        trail.injected_outcome_error = Some(dispatch::TrailError::WriteFailed(
            "outcome disk full".into(),
        ));
        let mut executor = FailingExecutor;
        authorise_and_execute_without_bridge_pins(
            &mut response,
            decision,
            &resolved,
            &mut trail,
            &mut executor,
            "evt_input_001",
        )
        .unwrap();

        assert_eq!(trail.entries.len(), 1);
        assert!(trail.outcome_entries.is_empty());

        assert_eq!(response["execution_status"], "failed");

        let json_trail = response["trail"].as_array().unwrap();
        let audit_count = json_trail
            .iter()
            .filter(|e| e["kind"] == "audit_failure")
            .count();
        assert_eq!(audit_count, 1);
    }

    // -----------------------------------------------------------------------
    // Test 24: Conforming output passes validation and succeeds.
    // -----------------------------------------------------------------------

    #[test]
    fn conforming_output_passes_validation_and_succeeds() {
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
        authorise_and_execute_without_bridge_pins(
            &mut response,
            decision,
            &resolved,
            &mut trail,
            &mut executor,
            "evt_input_001",
        )
        .unwrap();

        assert_eq!(response["execution_status"], "completed");
        let json_trail = response["trail"].as_array().unwrap();
        assert!(json_trail.iter().any(|e| e["kind"] == "action_completed"));
        assert!(!json_trail.iter().any(|e| e["kind"] == "action_failed"));
        assert_eq!(trail.outcome_entries.len(), 1);
        assert_eq!(trail.outcome_entries[0].status, "succeeded");
    }

    // -----------------------------------------------------------------------
    // Test 25: Missing required field in output produces validation failure.
    // -----------------------------------------------------------------------

    #[test]
    fn missing_required_output_field_fails_validation() {
        let (_store, resolved) = resolved_lantern();
        let decision = allow_decision_for(&resolved);
        let mut response = make_matched_response(
            "eval-001",
            "action_1",
            "lantern.task.record",
            json!({"project": "lantern-keeper", "task": "LK-39"}),
        );

        struct MissingStatusExecutor;
        impl CapabilityExecutor for MissingStatusExecutor {
            fn provider_identity(&self) -> &str {
                "lantern-local"
            }
            fn execute(&mut self, _ready: &DispatchReadyAction) -> Result<Value, String> {
                Ok(json!({"wrong_field": 1}))
            }
        }

        let mut trail = RecordingTrail::new();
        let mut executor = MissingStatusExecutor;
        authorise_and_execute_without_bridge_pins(
            &mut response,
            decision,
            &resolved,
            &mut trail,
            &mut executor,
            "evt_input_001",
        )
        .unwrap();

        assert_eq!(response["execution_status"], "failed");
        let json_trail = response["trail"].as_array().unwrap();
        assert!(json_trail.iter().any(|e| e["kind"] == "action_failed"));
        assert!(!json_trail.iter().any(|e| e["kind"] == "action_completed"));

        let failed = json_trail
            .iter()
            .find(|e| e["kind"] == "action_failed")
            .unwrap();
        assert!(failed["message"]
            .as_str()
            .unwrap()
            .starts_with("output validation failed:"));
        assert!(failed["message"]
            .as_str()
            .unwrap()
            .contains("missing required property"));

        assert_eq!(trail.outcome_entries.len(), 1);
        let outcome = &trail.outcome_entries[0];
        assert_eq!(outcome.status, "failed");
        assert!(outcome
            .error_message
            .as_ref()
            .unwrap()
            .starts_with("output validation failed:"));
        assert_eq!(outcome.result, None);
    }

    // -----------------------------------------------------------------------
    // Test 26: Wrong property type produces validation failure.
    // -----------------------------------------------------------------------

    #[test]
    fn wrong_property_type_fails_validation() {
        let (_store, resolved) = resolved_lantern();
        let decision = allow_decision_for(&resolved);
        let mut response = make_matched_response(
            "eval-001",
            "action_1",
            "lantern.task.record",
            json!({"project": "lantern-keeper", "task": "LK-39"}),
        );

        struct WrongTypeExecutor;
        impl CapabilityExecutor for WrongTypeExecutor {
            fn provider_identity(&self) -> &str {
                "lantern-local"
            }
            fn execute(&mut self, _ready: &DispatchReadyAction) -> Result<Value, String> {
                Ok(json!({"status": 123}))
            }
        }

        let mut trail = RecordingTrail::new();
        let mut executor = WrongTypeExecutor;
        authorise_and_execute_without_bridge_pins(
            &mut response,
            decision,
            &resolved,
            &mut trail,
            &mut executor,
            "evt_input_001",
        )
        .unwrap();

        assert_eq!(response["execution_status"], "failed");
        let json_trail = response["trail"].as_array().unwrap();
        let failed = json_trail
            .iter()
            .find(|e| e["kind"] == "action_failed")
            .unwrap();
        assert!(failed["message"]
            .as_str()
            .unwrap()
            .starts_with("output validation failed:"));
        assert!(failed["message"]
            .as_str()
            .unwrap()
            .contains("type mismatch"));
    }

    // -----------------------------------------------------------------------
    // Test 27: Additional property rejected when additionalProperties false.
    // -----------------------------------------------------------------------

    #[test]
    fn additional_property_fails_validation() {
        let (_store, resolved) = resolved_lantern();
        let decision = allow_decision_for(&resolved);
        let mut response = make_matched_response(
            "eval-001",
            "action_1",
            "lantern.task.record",
            json!({"project": "lantern-keeper", "task": "LK-39"}),
        );

        struct ExtraPropExecutor;
        impl CapabilityExecutor for ExtraPropExecutor {
            fn provider_identity(&self) -> &str {
                "lantern-local"
            }
            fn execute(&mut self, _ready: &DispatchReadyAction) -> Result<Value, String> {
                Ok(json!({"status": "ok", "extra": true}))
            }
        }

        let mut trail = RecordingTrail::new();
        let mut executor = ExtraPropExecutor;
        authorise_and_execute_without_bridge_pins(
            &mut response,
            decision,
            &resolved,
            &mut trail,
            &mut executor,
            "evt_input_001",
        )
        .unwrap();

        assert_eq!(response["execution_status"], "failed");
        let json_trail = response["trail"].as_array().unwrap();
        let failed = json_trail
            .iter()
            .find(|e| e["kind"] == "action_failed")
            .unwrap();
        assert!(failed["message"]
            .as_str()
            .unwrap()
            .starts_with("output validation failed:"));
        assert!(failed["message"]
            .as_str()
            .unwrap()
            .contains("additional property"));
    }

    #[test]
    fn additional_property_allowed_when_schema_permits_it() {
        let mut manifest = lantern_manifest_json();
        manifest["output_schema"]["additionalProperties"] = json!(true);
        let (_store, resolved) = resolved_lantern_with_manifest(manifest);
        let decision = allow_decision_for(&resolved);
        let mut response = make_matched_response(
            "eval-001",
            "action_1",
            "lantern.task.record",
            json!({"project": "lantern-keeper", "task": "LK-39"}),
        );

        struct ExtraPropExecutor;
        impl CapabilityExecutor for ExtraPropExecutor {
            fn provider_identity(&self) -> &str {
                "lantern-local"
            }
            fn execute(&mut self, _ready: &DispatchReadyAction) -> Result<Value, String> {
                Ok(json!({"status": "ok", "extra": true}))
            }
        }

        let mut trail = RecordingTrail::new();
        let mut executor = ExtraPropExecutor;
        authorise_and_execute_without_bridge_pins(
            &mut response,
            decision,
            &resolved,
            &mut trail,
            &mut executor,
            "evt_input_001",
        )
        .unwrap();

        assert_eq!(response["execution_status"], "completed");
        let json_trail = response["trail"].as_array().unwrap();
        assert!(json_trail.iter().any(|e| e["kind"] == "action_completed"));
        assert!(!json_trail.iter().any(|e| e["kind"] == "action_failed"));
        assert_eq!(trail.outcome_entries.len(), 1);
        assert_eq!(trail.outcome_entries[0].status, "succeeded");
        assert_eq!(
            trail.outcome_entries[0].result.as_ref().unwrap()["extra"],
            true
        );
    }

    // -----------------------------------------------------------------------
    // Test 28: Executor Err message is preserved, not replaced by validation.
    // -----------------------------------------------------------------------

    #[test]
    fn executor_error_is_not_replaced_by_validation() {
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
        authorise_and_execute_without_bridge_pins(
            &mut response,
            decision,
            &resolved,
            &mut trail,
            &mut executor,
            "evt_input_001",
        )
        .unwrap();

        assert_eq!(response["execution_status"], "failed");
        let json_trail = response["trail"].as_array().unwrap();
        let failed = json_trail
            .iter()
            .find(|e| e["kind"] == "action_failed")
            .unwrap();
        assert_eq!(failed["message"], "executor failed as requested");
        assert!(!failed["message"]
            .as_str()
            .unwrap()
            .contains("output validation"));
    }

    // -----------------------------------------------------------------------
    // Test 29: Output validation failure + outcome-write failure preserves
    //          execution_status "failed" and audits.
    // -----------------------------------------------------------------------

    #[test]
    fn validation_failure_with_outcome_write_failure_audits() {
        let (_store, resolved) = resolved_lantern();
        let decision = allow_decision_for(&resolved);
        let mut response = make_matched_response(
            "eval-001",
            "action_1",
            "lantern.task.record",
            json!({"project": "lantern-keeper", "task": "LK-39"}),
        );

        struct MissingStatusExecutor;
        impl CapabilityExecutor for MissingStatusExecutor {
            fn provider_identity(&self) -> &str {
                "lantern-local"
            }
            fn execute(&mut self, _ready: &DispatchReadyAction) -> Result<Value, String> {
                Ok(json!({"wrong_field": 1}))
            }
        }

        let mut trail = RecordingTrail::new();
        trail.injected_outcome_error = Some(dispatch::TrailError::WriteFailed(
            "outcome disk full".into(),
        ));
        let mut executor = MissingStatusExecutor;
        authorise_and_execute_without_bridge_pins(
            &mut response,
            decision,
            &resolved,
            &mut trail,
            &mut executor,
            "evt_input_001",
        )
        .unwrap();

        assert_eq!(response["execution_status"], "failed");
        let json_trail = response["trail"].as_array().unwrap();
        let audit_count = json_trail
            .iter()
            .filter(|e| e["kind"] == "audit_failure")
            .count();
        assert_eq!(audit_count, 1);
        assert!(json_trail.iter().any(|e| e["kind"] == "action_failed"));
    }

    // -----------------------------------------------------------------------
    // Test 30: Schema comes from verified manifest, not hard-coded.
    // -----------------------------------------------------------------------

    #[test]
    fn schema_read_from_verified_manifest_not_hard_coded() {
        let (_store, resolved) = resolved_lantern();
        let manifest_schema = resolved.manifest().manifest().output_schema.clone();
        assert_eq!(manifest_schema["type"], "object");
        assert_eq!(manifest_schema["required"][0], "status");
        assert_eq!(manifest_schema["properties"]["status"]["type"], "string");
        assert_eq!(manifest_schema["properties"]["project"]["type"], "string");
        assert_eq!(manifest_schema["properties"]["task"]["type"], "string");
        assert_eq!(manifest_schema["additionalProperties"], false);

        // Now use authorise_and_execute with conforming output.
        let decision = allow_decision_for(&resolved);
        let mut response = make_matched_response(
            "eval-001",
            "action_1",
            "lantern.task.record",
            json!({"project": "lantern-keeper", "task": "LK-39"}),
        );

        let mut trail = RecordingTrail::new();
        let mut executor = MockExecutor::new();
        authorise_and_execute_without_bridge_pins(
            &mut response,
            decision,
            &resolved,
            &mut trail,
            &mut executor,
            "evt_input_001",
        )
        .unwrap();

        assert_eq!(response["execution_status"], "completed");
    }

    // -----------------------------------------------------------------------
    // Test 31: Result Anchor — success path assertions.
    // -----------------------------------------------------------------------

    #[test]
    fn result_anchor_success_integration() {
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
        authorise_and_execute_without_bridge_pins(
            &mut response,
            decision,
            &resolved,
            &mut trail,
            &mut executor,
            "evt_input_001",
        )
        .unwrap();

        assert_eq!(response["execution_status"], "completed");

        let anchor = &response["result_anchor"];
        assert_eq!(anchor["event_id"], "eval-001/action_1/result");
        assert_eq!(anchor["event_name"], "capability.succeeded");
        assert_eq!(anchor["producer"], "tethers-reference-host");
        assert_eq!(anchor["correlation_id"], "evt_input_001");
        assert_eq!(anchor["causation_id"], "evt_input_001");
        assert_eq!(anchor["generation"], 1);

        let facts = &anchor["facts"];
        assert_eq!(facts["evaluation_id"], "eval-001");
        assert_eq!(facts["action_id"], "action_1");
        assert_eq!(facts["capability"]["name"], "lantern.task.record");
        assert_eq!(facts["capability"]["version"], 1);
        assert_eq!(facts["manifest_digest"], resolved.manifest_digest());
        assert_eq!(facts["provider_identity"], "lantern-local");

        assert_eq!(
            facts["result"],
            json!({"status": "recorded", "project": "lantern-keeper", "task": "LK-39"})
        );
        assert!(facts.get("error").is_none());

        let outcome = &trail.outcome_entries[0];
        assert_eq!(anchor["occurred_at"], outcome.timestamp_unix_ms);
    }

    // -----------------------------------------------------------------------
    // Test 32: Result Anchor — executor error path.
    // -----------------------------------------------------------------------

    #[test]
    fn result_anchor_executor_error_integration() {
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
        authorise_and_execute_without_bridge_pins(
            &mut response,
            decision,
            &resolved,
            &mut trail,
            &mut executor,
            "evt_input_001",
        )
        .unwrap();

        assert_eq!(response["execution_status"], "failed");

        let anchor = &response["result_anchor"];
        assert_eq!(anchor["event_name"], "capability.failed");
        assert_eq!(anchor["correlation_id"], "evt_input_001");
        assert_eq!(anchor["causation_id"], "evt_input_001");

        let facts = &anchor["facts"];
        assert_eq!(facts["capability"]["name"], "lantern.task.record");
        assert_eq!(facts["capability"]["version"], 1);
        assert_eq!(facts["manifest_digest"], resolved.manifest_digest());
        assert_eq!(facts["provider_identity"], "lantern-local");

        assert!(facts.get("result").is_none());
        let error = &facts["error"];
        assert_eq!(error["code"], "provider_error");
        assert_eq!(error["message"], "executor failed as requested");

        let outcome = &trail.outcome_entries[0];
        assert_eq!(anchor["occurred_at"], outcome.timestamp_unix_ms);
    }

    // -----------------------------------------------------------------------
    // Test 33: Result Anchor — output-validation failure path.
    // -----------------------------------------------------------------------

    #[test]
    fn result_anchor_validation_failure_integration() {
        let (_store, resolved) = resolved_lantern();
        let decision = allow_decision_for(&resolved);
        let mut response = make_matched_response(
            "eval-001",
            "action_1",
            "lantern.task.record",
            json!({"project": "lantern-keeper", "task": "LK-39"}),
        );

        struct MissingStatusExecutor;
        impl CapabilityExecutor for MissingStatusExecutor {
            fn provider_identity(&self) -> &str {
                "lantern-local"
            }
            fn execute(&mut self, _ready: &DispatchReadyAction) -> Result<Value, String> {
                Ok(json!({"wrong_field": 1}))
            }
        }

        let mut trail = RecordingTrail::new();
        let mut executor = MissingStatusExecutor;
        authorise_and_execute_without_bridge_pins(
            &mut response,
            decision,
            &resolved,
            &mut trail,
            &mut executor,
            "evt_input_001",
        )
        .unwrap();

        assert_eq!(response["execution_status"], "failed");

        let anchor = &response["result_anchor"];
        assert_eq!(anchor["event_name"], "capability.failed");

        let facts = &anchor["facts"];
        assert!(facts.get("result").is_none());
        let error = &facts["error"];
        assert_eq!(error["code"], "result_validation_failed");
        assert!(error["message"]
            .as_str()
            .unwrap()
            .contains("output validation failed"));
    }

    // -----------------------------------------------------------------------
    // Test 34: No Result Anchor for preparation failures.
    // -----------------------------------------------------------------------

    fn assert_no_result_anchor(response: &Value, description: &str) {
        assert!(
            response.get("result_anchor").is_none(),
            "result_anchor must be absent for {description}"
        );
    }

    #[test]
    fn no_result_anchor_on_deny() {
        let (_store, resolved) = resolved_lantern();
        let mut response = make_matched_response(
            "eval-001",
            "action_1",
            "lantern.task.record",
            json!({"project": "p", "task": "t"}),
        );
        let mut trail = RecordingTrail::new();
        let mut executor = MockExecutor::new();
        authorise_and_execute_without_bridge_pins(
            &mut response,
            PermissionDecision::Deny,
            &resolved,
            &mut trail,
            &mut executor,
            "evt_input_001",
        )
        .unwrap();
        assert_no_result_anchor(&response, "Deny");
        assert_eq!(response["execution_status"], "denied");
    }

    #[test]
    fn no_result_anchor_on_ask() {
        let (_store, resolved) = resolved_lantern();
        let mut response = make_matched_response(
            "eval-001",
            "action_1",
            "lantern.task.record",
            json!({"project": "p", "task": "t"}),
        );
        let mut trail = RecordingTrail::new();
        let mut executor = MockExecutor::new();
        authorise_and_execute_without_bridge_pins(
            &mut response,
            PermissionDecision::Ask,
            &resolved,
            &mut trail,
            &mut executor,
            "evt_input_001",
        )
        .unwrap();
        assert_no_result_anchor(&response, "Ask");
        assert_eq!(response["execution_status"], "denied");
    }

    #[test]
    fn no_result_anchor_on_unavailable() {
        let (_store, resolved) = resolved_lantern();
        let mut response = make_matched_response(
            "eval-001",
            "action_1",
            "lantern.task.record",
            json!({"project": "p", "task": "t"}),
        );
        let mut trail = RecordingTrail::new();
        let mut executor = MockExecutor::new();
        authorise_and_execute_without_bridge_pins(
            &mut response,
            PermissionDecision::Unavailable,
            &resolved,
            &mut trail,
            &mut executor,
            "evt_input_001",
        )
        .unwrap();
        assert_no_result_anchor(&response, "Unavailable");
        assert_eq!(response["execution_status"], "denied");
    }

    #[test]
    fn no_result_anchor_on_capability_mismatch() {
        let (_store, resolved) = resolved_lantern();
        let decision = allow_decision_for(&resolved);
        let mut response = make_matched_response(
            "eval-001",
            "action_1",
            "wrong.capability",
            json!({"project": "x", "task": "y"}),
        );

        let mut trail = RecordingTrail::new();
        let mut executor = MockExecutor::new();
        let result = authorise_and_execute_without_bridge_pins(
            &mut response,
            decision,
            &resolved,
            &mut trail,
            &mut executor,
            "evt_input_001",
        );
        assert!(result.is_err());
        assert_no_result_anchor(&response, "capability mismatch");
    }

    #[test]
    fn no_result_anchor_on_provider_identity_mismatch() {
        let (_store, resolved) = resolved_lantern();
        let decision = allow_decision_for(&resolved);

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
        let result = authorise_and_execute_without_bridge_pins(
            &mut response,
            decision,
            &resolved,
            &mut trail,
            &mut executor,
            "evt_input_001",
        );
        assert!(result.is_err());
        assert_no_result_anchor(&response, "provider identity mismatch");
    }

    #[test]
    fn no_result_anchor_on_intent_write_failure() {
        let (_store, resolved) = resolved_lantern();
        let decision = allow_decision_for(&resolved);
        let mut response = make_matched_response(
            "eval-001",
            "action_1",
            "lantern.task.record",
            json!({"project": "p", "task": "t"}),
        );
        let mut trail = RecordingTrail::new();
        trail.injected_intent_error = Some(dispatch::TrailError::WriteFailed("disk full".into()));
        let mut executor = MockExecutor::new();
        authorise_and_execute_without_bridge_pins(
            &mut response,
            decision,
            &resolved,
            &mut trail,
            &mut executor,
            "evt_input_001",
        )
        .unwrap();
        assert_no_result_anchor(&response, "intent write failure");
        assert_eq!(response["execution_status"], "denied");
    }

    #[test]
    fn no_result_anchor_on_intent_flush_failure() {
        let (_store, resolved) = resolved_lantern();
        let decision = allow_decision_for(&resolved);
        let mut response = make_matched_response(
            "eval-001",
            "action_1",
            "lantern.task.record",
            json!({"project": "p", "task": "t"}),
        );
        let mut trail = RecordingTrail::new();
        trail.injected_intent_error =
            Some(dispatch::TrailError::FlushFailed("sync_data failed".into()));
        let mut executor = MockExecutor::new();
        authorise_and_execute_without_bridge_pins(
            &mut response,
            decision,
            &resolved,
            &mut trail,
            &mut executor,
            "evt_input_001",
        )
        .unwrap();
        assert_no_result_anchor(&response, "intent flush failure");
        assert_eq!(response["execution_status"], "denied");
    }

    // -----------------------------------------------------------------------
    // Test 35: Outcome-write audit failure preserves Result Anchor outcome.
    // -----------------------------------------------------------------------

    #[test]
    fn outcome_write_audit_failure_preserves_result_anchor() {
        let (_store, resolved) = resolved_lantern();
        let decision = allow_decision_for(&resolved);
        let mut response = make_matched_response(
            "eval-001",
            "action_1",
            "lantern.task.record",
            json!({"project": "lantern-keeper", "task": "LK-39"}),
        );

        let mut trail = RecordingTrail::new();
        trail.injected_outcome_error = Some(dispatch::TrailError::FlushFailed(
            "outcome sync_data failed".into(),
        ));
        let mut executor = MockExecutor::new();
        authorise_and_execute_without_bridge_pins(
            &mut response,
            decision,
            &resolved,
            &mut trail,
            &mut executor,
            "evt_input_001",
        )
        .unwrap();

        assert_eq!(response["execution_status"], "completed");

        let anchor = &response["result_anchor"];
        assert_eq!(anchor["event_name"], "capability.succeeded");
        let facts = &anchor["facts"];
        assert_eq!(
            facts["result"],
            json!({"status": "recorded", "project": "lantern-keeper", "task": "LK-39"})
        );
        assert!(facts.get("error").is_none());
    }

    #[test]
    fn outcome_write_audit_failure_after_executor_error_preserves_failed_anchor() {
        let (_store, resolved) = resolved_lantern();
        let decision = allow_decision_for(&resolved);
        let mut response = make_matched_response(
            "eval-001",
            "action_1",
            "lantern.task.record",
            json!({"project": "lantern-keeper", "task": "LK-39"}),
        );

        let mut trail = RecordingTrail::new();
        trail.injected_outcome_error = Some(dispatch::TrailError::WriteFailed(
            "outcome disk full".into(),
        ));
        let mut executor = FailingExecutor;
        authorise_and_execute_without_bridge_pins(
            &mut response,
            decision,
            &resolved,
            &mut trail,
            &mut executor,
            "evt_input_001",
        )
        .unwrap();

        assert_eq!(response["execution_status"], "failed");

        let anchor = &response["result_anchor"];
        assert_eq!(anchor["event_name"], "capability.failed");
        let error = &anchor["facts"]["error"];
        assert_eq!(error["code"], "provider_error");
        assert_eq!(error["message"], "executor failed as requested");
    }

    // -----------------------------------------------------------------------
    // Test 36: Explicit planner version mapping for bridge pinning.
    // -----------------------------------------------------------------------

    #[test]
    fn planner_version_mapping_is_explicit_and_strict() {
        assert_eq!(planner_version_from_manifest_major(1), "1.0.0");
        assert_eq!(manifest_major_from_planner_version("1.0.0").unwrap(), 1);

        let not_bridge = manifest_major_from_planner_version("1.2.3");
        assert!(not_bridge.is_err());
        let malformed = manifest_major_from_planner_version("v1");
        assert!(malformed.is_err());
        let zero = manifest_major_from_planner_version("0.0.0");
        assert!(zero.is_err());
    }

    // -----------------------------------------------------------------------
    // Test 37: Pre-evaluation projection injects bridge pins into request.
    // -----------------------------------------------------------------------

    #[test]
    fn projection_injected_into_request_capabilities() {
        let (store, _resolved) = resolved_lantern();
        let availability = ProviderAvailability::from_identities(["lantern-local"]);
        let mut request = json!({
            "capabilities": [
                {
                    "name": "lantern.task.record",
                    "version": "1.0.0",
                    "inputs": {"project": "string", "task": "string"},
                    "effects": ["lantern.write"]
                },
                {
                    "name": "legacy.non.bridge",
                    "version": "beta",
                    "inputs": {},
                    "effects": []
                }
            ]
        });

        inject_bridge_projection_into_request(&mut request, &store, &availability).unwrap();

        let caps = request["capabilities"].as_array().unwrap();
        let projected = caps[0].as_object().unwrap();
        assert_eq!(projected["version"], "1.0.0");
        assert_eq!(projected["bridge_capability_version"], 1);
        assert!(projected
            .get("manifest_digest")
            .and_then(Value::as_str)
            .unwrap()
            .starts_with("sha256:"));
        assert_eq!(projected["bridge_provider_identity"], "lantern-local");

        let legacy = caps[1].as_object().unwrap();
        assert!(legacy.get("manifest_digest").is_none());
        assert!(legacy.get("bridge_capability_version").is_none());
    }

    // -----------------------------------------------------------------------
    // Test 38: Stale plan digest D1 fails closed against current D2 binding.
    // -----------------------------------------------------------------------

    #[test]
    fn stale_plan_digest_fails_closed_before_dispatch() {
        // Snapshot T1: plan built from manifest digest D1.
        let (store_t1, resolved_t1) = resolved_lantern();
        let availability = ProviderAvailability::from_identities(["lantern-local"]);
        let mut request_t1 = json!({
            "capabilities": [
                {
                    "name": "lantern.task.record",
                    "version": "1.0.0",
                    "inputs": {"project": "string", "task": "string"},
                    "effects": ["lantern.write"]
                }
            ]
        });
        inject_bridge_projection_into_request(&mut request_t1, &store_t1, &availability).unwrap();

        let stale_digest = request_t1["capabilities"][0]["manifest_digest"]
            .as_str()
            .unwrap()
            .to_owned();

        // Snapshot T2: current verified binding now resolves to digest D2.
        let mut manifest_t2 = lantern_manifest_json();
        manifest_t2["effects"] = json!(["lantern.write", "network.access"]);
        let (store_t2, resolved_t2) = resolved_lantern_with_manifest(manifest_t2);
        assert_ne!(stale_digest, resolved_t2.manifest_digest());

        struct CallCountingExecutor {
            calls: u32,
        }
        impl CapabilityExecutor for CallCountingExecutor {
            fn provider_identity(&self) -> &str {
                "lantern-local"
            }
            fn execute(&mut self, _ready: &DispatchReadyAction) -> Result<Value, String> {
                self.calls += 1;
                Err("must not be called".into())
            }
        }

        let decision = allow_decision_for(&resolved_t2);
        let mut response = make_matched_response(
            "eval-001",
            "action_1",
            "lantern.task.record",
            json!({"project": "p", "task": "t"}),
        );
        response["plan"]["actions"][0]["manifest_digest"] = Value::String(stale_digest);
        response["plan"]["actions"][0]["bridge_capability_version"] = Value::from(1u64);
        response["plan"]["actions"][0]["bridge_provider_identity"] =
            Value::String("lantern-local".to_owned());
        response["plan"]["actions"][0]["capability_version"] = Value::String("1.0.0".to_owned());

        let mut trail = RecordingTrail::new();
        let mut executor = CallCountingExecutor { calls: 0 };
        let err = authorise_and_execute(
            &mut response,
            decision,
            &resolved_t2,
            &mut trail,
            &mut executor,
            "evt_input_001",
        )
        .expect_err("stale digest must fail closed");

        assert!(err.to_string().contains("stale plan digest"));
        assert_eq!(executor.calls, 0);
        assert_eq!(store_t2.len(), 1);

        // Sanity: dispatch remains executable when digest is current.
        let mut success_response = make_matched_response(
            "eval-001",
            "action_1",
            "lantern.task.record",
            json!({"project": "p", "task": "t"}),
        );
        success_response["plan"]["actions"][0]["manifest_digest"] =
            Value::String(resolved_t2.manifest_digest().to_owned());
        success_response["plan"]["actions"][0]["bridge_capability_version"] = Value::from(1u64);
        success_response["plan"]["actions"][0]["bridge_provider_identity"] =
            Value::String(resolved_t2.provider_identity().to_owned());
        success_response["plan"]["actions"][0]["capability_version"] =
            Value::String("1.0.0".to_owned());

        authorise_and_execute(
            &mut success_response,
            allow_decision_for(&resolved_t2),
            &resolved_t2,
            &mut RecordingTrail::new(),
            &mut executor,
            "evt_input_001",
        )
        .unwrap();
        assert_eq!(executor.calls, 1);
        assert_eq!(
            resolved_t1.provider_identity(),
            resolved_t2.provider_identity()
        );
    }

    #[test]
    fn incomplete_or_invalid_bridge_pins_fail_closed_before_dispatch() {
        struct CallCountingExecutor {
            calls: u32,
        }
        impl CapabilityExecutor for CallCountingExecutor {
            fn provider_identity(&self) -> &str {
                "lantern-local"
            }

            fn execute(&mut self, _ready: &DispatchReadyAction) -> Result<Value, String> {
                self.calls += 1;
                Err("must not be called".into())
            }
        }

        let (_store, resolved) = resolved_lantern();
        let mut cases = Vec::<(&str, Value)>::new();

        let mut missing_all = make_bridge_matched_response(&resolved);
        let missing_all_action = missing_all["plan"]["actions"][0].as_object_mut().unwrap();
        missing_all_action.remove("manifest_digest");
        missing_all_action.remove("bridge_capability_version");
        missing_all_action.remove("bridge_provider_identity");
        cases.push(("missing all bridge pins", missing_all));

        let mut missing_digest = make_bridge_matched_response(&resolved);
        missing_digest["plan"]["actions"][0]
            .as_object_mut()
            .unwrap()
            .remove("manifest_digest");
        cases.push(("missing digest pin", missing_digest));

        let mut missing_version = make_bridge_matched_response(&resolved);
        missing_version["plan"]["actions"][0]
            .as_object_mut()
            .unwrap()
            .remove("bridge_capability_version");
        cases.push(("missing version pin", missing_version));

        let mut missing_provider = make_bridge_matched_response(&resolved);
        missing_provider["plan"]["actions"][0]
            .as_object_mut()
            .unwrap()
            .remove("bridge_provider_identity");
        cases.push(("missing provider pin", missing_provider));

        let mut stale_version = make_bridge_matched_response(&resolved);
        stale_version["plan"]["actions"][0]["bridge_capability_version"] = Value::from(2u64);
        cases.push(("stale version pin", stale_version));

        let mut stale_provider = make_bridge_matched_response(&resolved);
        stale_provider["plan"]["actions"][0]["bridge_provider_identity"] =
            Value::String("other-provider".to_owned());
        cases.push(("stale provider pin", stale_provider));

        let mut oversized_version = make_bridge_matched_response(&resolved);
        oversized_version["plan"]["actions"][0]["bridge_capability_version"] =
            Value::from(u64::from(u32::MAX) + 1);
        cases.push(("oversized version pin", oversized_version));

        let mut inconsistent_version = make_bridge_matched_response(&resolved);
        inconsistent_version["plan"]["actions"][0]["capability_version"] =
            Value::String("2.0.0".to_owned());
        cases.push(("inconsistent planner version", inconsistent_version));

        for (label, mut response) in cases {
            let mut executor = CallCountingExecutor { calls: 0 };
            let err = authorise_and_execute(
                &mut response,
                allow_decision_for(&resolved),
                &resolved,
                &mut RecordingTrail::new(),
                &mut executor,
                "evt_input_001",
            )
            .expect_err(label);

            assert!(!err.to_string().is_empty(), "{label}");
            assert_eq!(executor.calls, 0, "{label}");
        }
    }
}
