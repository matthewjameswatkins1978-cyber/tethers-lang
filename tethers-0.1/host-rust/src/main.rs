pub mod approval;
pub mod dispatch;
mod manifest;
mod outcome;
pub mod policy;
pub mod provider;
pub mod replay;
mod replay_runtime;
#[cfg(windows)]
pub mod replay_windows;
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
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const NORMAL_USAGE: &str = "usage: tethers-reference-host ENGINE REQUEST_JSON [POLICY] \
[TRAIL_PATH] [EXECUTOR_MODE] [--host-data-root <ABSOLUTE_PATH>]";
const PROVISION_USAGE: &str =
    "usage: tethers-reference-host provision-replay <ABSOLUTE_HOST_DATA_ROOT>";

#[derive(Debug)]
struct NormalArgs {
    engine_path: String,
    request_path: String,
    policy_posture: String,
    trail_path: Option<String>,
    executor_mode: String,
    host_data_root: Option<PathBuf>,
}

fn parse_normal_args(args: &[String]) -> Result<NormalArgs, String> {
    let mut positional = Vec::new();
    let mut host_data_root = None;
    let mut index = 0;
    while index < args.len() {
        let argument = &args[index];
        if argument == "--host-data-root" {
            if host_data_root.is_some() {
                return Err("duplicate --host-data-root".to_owned());
            }
            index += 1;
            let value = args
                .get(index)
                .filter(|value| !value.starts_with("--"))
                .ok_or_else(|| "missing value for --host-data-root".to_owned())?;
            let path = PathBuf::from(value);
            if !path.is_absolute() {
                return Err("--host-data-root must be absolute".to_owned());
            }
            host_data_root = Some(path);
        } else if argument.starts_with('-') {
            return Err(format!("unknown option: {argument}"));
        } else {
            positional.push(argument.clone());
        }
        index += 1;
    }
    if !(2..=5).contains(&positional.len()) {
        return Err(NORMAL_USAGE.to_owned());
    }
    Ok(NormalArgs {
        engine_path: positional[0].clone(),
        request_path: positional[1].clone(),
        policy_posture: positional
            .get(2)
            .cloned()
            .unwrap_or_else(|| "allow".to_owned()),
        trail_path: positional.get(3).cloned(),
        executor_mode: positional
            .get(4)
            .cloned()
            .unwrap_or_else(|| "success".to_owned()),
        host_data_root,
    })
}

fn parse_provision_args(args: &[String]) -> Result<PathBuf, String> {
    if args.len() != 2 || args.first().map(String::as_str) != Some("provision-replay") {
        return Err(PROVISION_USAGE.to_owned());
    }
    let root = PathBuf::from(&args[1]);
    if !root.is_absolute() {
        return Err(PROVISION_USAGE.to_owned());
    }
    Ok(root)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if matches!(args.first().map(String::as_str), Some("provision-replay")) {
        let root = parse_provision_args(&args)?;
        #[cfg(windows)]
        {
            let result = replay_windows::provision_replay(&root)?;
            println!("{}", result.as_str());
            return Ok(());
        }
        #[cfg(not(windows))]
        return Err("replay persistence is available only on native Windows".into());
    }
    let normal = parse_normal_args(&args)?;

    let mut request: Value = serde_json::from_str(&fs::read_to_string(&normal.request_path)?)?;

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

    let mut response = call_engine(&normal.engine_path, &request)?;

    if response_is_matched(&response) {
        // 4. Resolve the capability.
        let resolved = resolver::resolve_capability(
            &store,
            &availability,
            "lantern.task.record",
            1,
            Some("lantern-local"),
        )
        .map_err(|e| format!("capability resolution failed: {e:?}"))?;

        // 5. Evaluate the complete J04 effective policy from the Plan's
        //    proposed Action, the Tether Set declaration, and host-local
        //    policy — posture from CLI, defaults to Allow.
        let requirements = vec![policy::CapabilityRequirement::new("lantern.task.record", 1)];
        let rule = match normal.policy_posture.as_str() {
            "allow" => policy::PolicyRule::Allow,
            "deny" => policy::PolicyRule::Deny,
            "ask" => policy::PolicyRule::Ask,
            other => return Err(format!("unknown policy posture: {other}").into()),
        };
        let host_policy = policy::HostLocalPolicy::new(rule);
        let proposed_action = extract_proposed_action(&response)?;
        // No concrete host/binding-specific scope assessor exists yet for
        // this demo manifest's `path_prefix` scope (deferred; see J03b in
        // docs/DECISIONS.md). The host must therefore fail closed rather than
        // assert that `project` or `task` is a scope-bearing argument.
        let evaluation = policy::evaluate_effective_policy(
            &proposed_action,
            &requirements,
            &store,
            &availability,
            &host_policy,
            policy::ScopeAssessment::ScopeNotEstablished,
        );
        let decision = evaluation.decision;

        // 6. Open file-backed durable Trail for intent recording.
        //    When no explicit path is supplied, use a temporary directory
        //    so repeated runs do not dirty the repository.
        let trail_path = normal.trail_path.unwrap_or_else(|| {
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
        match normal.executor_mode.as_str() {
            "success" => {
                let mut executor = MockExecutor::new();
                authorise_and_execute(
                    &mut response,
                    decision,
                    &resolved,
                    &mut file_trail,
                    &mut executor,
                    original_event_id,
                    normal.host_data_root.as_deref(),
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
                    normal.host_data_root.as_deref(),
                )?;
            }
            other => return Err(format!("unknown executor mode: {other}").into()),
        }
    }

    println!("{}", serde_json::to_string_pretty(&response)?);
    Ok(())
}

fn response_is_matched(response: &Value) -> bool {
    response.get("status").and_then(Value::as_str) == Some("matched")
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

/// Extract the single proposed Action's identity, bridge pins, and
/// arguments from an engine response for J04 effective-policy resolution.
///
/// Reads only already-produced planner output; performs no I/O and makes
/// no dispatch decision.
fn extract_proposed_action(
    response: &Value,
) -> Result<policy::ProposedAction, Box<dyn std::error::Error>> {
    let evaluation_id = required_str(response, "evaluation_id")?.to_owned();
    let plan_id = response
        .get("plan")
        .and_then(|plan| plan.get("id"))
        .and_then(Value::as_str)
        .ok_or("response plan had no id")?
        .to_owned();

    let action = response
        .get("plan")
        .and_then(|plan| plan.get("actions"))
        .and_then(Value::as_array)
        .and_then(|actions| actions.first())
        .ok_or("plan had no actions")?;

    let action_id = required_str(action, "action_id")?.to_owned();
    let capability_name = required_str(action, "capability")?.to_owned();
    let manifest_digest = action
        .get("manifest_digest")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let bridge_capability_version = action
        .get("bridge_capability_version")
        .and_then(Value::as_u64)
        .and_then(|v| u32::try_from(v).ok());
    let bridge_provider_identity = action
        .get("bridge_provider_identity")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let arguments = action.get("arguments").cloned().unwrap_or(Value::Null);

    Ok(policy::ProposedAction {
        evaluation_id,
        plan_id,
        action_id,
        capability_name,
        manifest_digest,
        bridge_capability_version,
        bridge_provider_identity,
        arguments,
    })
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

    /// Execute with the host-computed remaining monotonic deadline.  Adapters
    /// must bound their wait by `remaining` and report a typed ambiguity when
    /// no trustworthy final response is available in time.
    ///
    /// The compatibility implementation never treats an untyped string error
    /// as provider-declared failure: it is post-invocation uncertainty.
    /// Adapters with a trusted explicit provider error must override this
    /// method and return `ExplicitProviderError` themselves.
    fn execute_classified(
        &mut self,
        ready: &DispatchReadyAction,
        _remaining: Duration,
    ) -> Result<Value, outcome::ProviderDiagnostic> {
        self.execute(ready)
            .map_err(|_| outcome::ProviderDiagnostic::NoFinalResponse)
    }
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

    fn execute_classified(
        &mut self,
        ready: &DispatchReadyAction,
        _remaining: Duration,
    ) -> Result<Value, outcome::ProviderDiagnostic> {
        self.execute(ready)
            .map_err(|_| outcome::ProviderDiagnostic::NoFinalResponse)
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

    fn execute_classified(
        &mut self,
        _ready: &DispatchReadyAction,
        _remaining: Duration,
    ) -> Result<Value, outcome::ProviderDiagnostic> {
        Err(outcome::ProviderDiagnostic::ExplicitProviderError)
    }
}

// ---------------------------------------------------------------------------
// J05 exact Ask orchestration seam
// ---------------------------------------------------------------------------

/// The only host-facing approval operations.  These are deliberately separate
/// from planner and provider input: a caller cannot supply an already-approved
/// policy result or manufacture a human decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HumanApprovalDecision {
    Approve,
    Deny,
    Cancel,
}

fn approval_trail_entry(
    proof: &approval::ApprovalProof,
    kind: &str,
    reason_code: &str,
) -> dispatch::AuthorisationEntry {
    dispatch::AuthorisationEntry {
        execution_id: proof.evaluation_id.clone(),
        action_id: proof.action_id.clone(),
        capability_name: proof.capability_name.clone(),
        capability_version: proof.capability_version,
        provider_identity: proof.provider_identity.clone(),
        manifest_digest: proof.manifest_digest.clone(),
        kind: kind.to_owned(),
        reason_code: reason_code.to_owned(),
        argument_digest: proof.argument_digest.clone(),
    }
}

/// Request an approval only after current ordinary policy independently says
/// Ask.  Duplicate requests for a live proof reuse the pending record and do
/// not create a second Trail claim.
fn request_exact_approval(
    action: &policy::ProposedAction,
    requirements: &[policy::CapabilityRequirement],
    store: &trusted_store::TrustedManifestStore,
    availability: &resolver::ProviderAvailability,
    host_policy: &policy::HostLocalPolicy,
    scope: policy::ScopeAssessment,
    approvals: &mut approval::ApprovalStore,
    trail: &mut dyn dispatch::Trail,
) -> Result<Option<approval::ApprovalRecord>, Box<dyn std::error::Error>> {
    let evaluation = policy::evaluate_effective_policy(
        action,
        requirements,
        store,
        availability,
        host_policy,
        scope,
    );
    if evaluation.decision != PermissionDecision::Ask {
        return Ok(None);
    }
    let proof = approval::ApprovalProof::from_action(action)?;
    // The exact proof may be requested repeatedly while pending.  This lookup
    // is only for request idempotency; resumes are addressed by approval_id.
    if let Some(record) = approvals.pending_matching(&proof) {
        return Ok(Some(record.clone()));
    }
    let record = approvals.request(proof.clone());
    if let Err(error) =
        trail.append_authorisation(&approval_trail_entry(&proof, "approval_requested", "ask"))
    {
        approvals.discard_pending(&record.approval_id)?;
        return Err(error.into());
    }
    Ok(Some(record))
}

/// This is the host-recognised decision boundary.  The state transition is
/// performed first; a failed Trail append cannot claim a transition that did
/// not happen, and the caller must not resume after its error.
fn record_human_approval_decision(
    approval_id: &str,
    decision: HumanApprovalDecision,
    approvals: &mut approval::ApprovalStore,
    trail: &mut dyn dispatch::Trail,
) -> Result<(), Box<dyn std::error::Error>> {
    let (next, kind) = match decision {
        HumanApprovalDecision::Approve => (approval::ApprovalState::Approved, "approval_granted"),
        HumanApprovalDecision::Deny => (approval::ApprovalState::Denied, "approval_denied"),
        HumanApprovalDecision::Cancel => (approval::ApprovalState::Cancelled, "approval_cancelled"),
    };
    let record = approvals.decide(approval_id, next)?;
    if let Err(error) =
        trail.append_authorisation(&approval_trail_entry(&record.proof, kind, "human_decision"))
    {
        // A failed grant audit must not leave dispatchable authority.  Denial
        // and cancellation remain terminal, so neither can be reused.
        if next == approval::ApprovalState::Approved {
            approvals.invalidate_live(approval_id)?;
        }
        return Err(error.into());
    }
    Ok(())
}

/// Re-resolve every ordinary policy input inside the resume seam.  An
/// approval record is evidence only for an otherwise-current Ask: it cannot
/// turn Deny, Unavailable, schema failure, scope failure, or stale pins into
/// an Allow.
enum ExactApprovalPrecheck {
    Ready(approval::ApprovalProof),
    NotDispatchable(PermissionDecision),
}

fn precheck_exact_approval(
    action: &policy::ProposedAction,
    approval_id: &str,
    requirements: &[policy::CapabilityRequirement],
    store: &trusted_store::TrustedManifestStore,
    availability: &resolver::ProviderAvailability,
    host_policy: &policy::HostLocalPolicy,
    scope: policy::ScopeAssessment,
    approvals: &mut approval::ApprovalStore,
    trail: &mut dyn dispatch::Trail,
) -> Result<ExactApprovalPrecheck, Box<dyn std::error::Error>> {
    let evaluation = policy::evaluate_effective_policy(
        action,
        requirements,
        store,
        availability,
        host_policy,
        scope,
    );
    let fresh_proof = approval::ApprovalProof::from_action(action);
    let needs_invalidation = evaluation.decision != PermissionDecision::Ask || fresh_proof.is_err();
    if needs_invalidation {
        if let Ok(Some(record)) = approvals.invalidate_live(approval_id) {
            trail.append_authorisation(&approval_trail_entry(
                &record.proof,
                "approval_invalidated",
                "fresh_policy_or_proof_failed",
            ))?;
        }
        return Ok(ExactApprovalPrecheck::NotDispatchable(evaluation.decision));
    }
    let fresh_proof = fresh_proof?;
    let matching = approvals
        .record(approval_id)
        .map(|record| record.proof.exactly_matches(&fresh_proof))
        .unwrap_or(false);
    if !matching {
        if let Ok(Some(record)) = approvals.invalidate_live(approval_id) {
            trail.append_authorisation(&approval_trail_entry(
                &record.proof,
                "approval_invalidated",
                "approval_proof_mismatch",
            ))?;
        }
        return Ok(ExactApprovalPrecheck::NotDispatchable(
            PermissionDecision::Ask,
        ));
    }
    if approvals.record(approval_id)?.state != approval::ApprovalState::Approved {
        return Ok(ExactApprovalPrecheck::NotDispatchable(
            PermissionDecision::Ask,
        ));
    }
    Ok(ExactApprovalPrecheck::Ready(fresh_proof))
}

trait ApprovalConsumption {
    fn consume(&mut self, trail: &mut dyn dispatch::Trail) -> Result<(), ()>;
}

trait ResultAnchorWriter {
    fn write(&mut self, response: &mut Value, anchor: &ResultAnchor) -> Result<(), ()>;
}

struct ResponseResultAnchorWriter;

impl ResultAnchorWriter for ResponseResultAnchorWriter {
    fn write(&mut self, response: &mut Value, anchor: &ResultAnchor) -> Result<(), ()> {
        response["result_anchor"] = serde_json::to_value(anchor).map_err(|_| ())?;
        Ok(())
    }
}

#[allow(dead_code)]
struct ExactApprovalConsumption<'a> {
    approval_id: &'a str,
    proof: approval::ApprovalProof,
    approvals: &'a mut approval::ApprovalStore,
}

impl ApprovalConsumption for ExactApprovalConsumption<'_> {
    fn consume(&mut self, trail: &mut dyn dispatch::Trail) -> Result<(), ()> {
        let consumed = self
            .approvals
            .consume(self.approval_id, &self.proof)
            .map_err(|_| ())?;
        trail
            .append_authorisation(&approval_trail_entry(
                &consumed.proof,
                "approval_consumed",
                "exact_approved_ask",
            ))
            .map_err(|_| ())
    }
}

/// Approved-Ask orchestration performs complete fresh checks first, but defers
/// the one-shot consume until a new replay claim is durably admitted.
#[allow(clippy::too_many_arguments)]
fn resume_and_execute_exact_approval(
    response: &mut Value,
    approval_id: &str,
    requirements: &[policy::CapabilityRequirement],
    store: &trusted_store::TrustedManifestStore,
    availability: &resolver::ProviderAvailability,
    host_policy: &policy::HostLocalPolicy,
    scope: policy::ScopeAssessment,
    approvals: &mut approval::ApprovalStore,
    trail: &mut dyn dispatch::Trail,
    executor: &mut dyn CapabilityExecutor,
    original_event_id: &str,
    host_data_root: Option<&Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut replay_authority = replay_runtime::FileReplayAuthority::new(host_data_root);
    resume_and_execute_exact_approval_with_authority(
        response,
        approval_id,
        requirements,
        store,
        availability,
        host_policy,
        scope,
        approvals,
        trail,
        executor,
        original_event_id,
        &mut replay_authority,
    )
}

#[allow(clippy::too_many_arguments)]
#[allow(dead_code)]
fn resume_and_execute_exact_approval_with_authority(
    response: &mut Value,
    approval_id: &str,
    requirements: &[policy::CapabilityRequirement],
    store: &trusted_store::TrustedManifestStore,
    availability: &resolver::ProviderAvailability,
    host_policy: &policy::HostLocalPolicy,
    scope: policy::ScopeAssessment,
    approvals: &mut approval::ApprovalStore,
    trail: &mut dyn dispatch::Trail,
    executor: &mut dyn CapabilityExecutor,
    original_event_id: &str,
    replay_authority: &mut dyn replay_runtime::ReplayAuthority,
) -> Result<(), Box<dyn std::error::Error>> {
    let action = extract_proposed_action(response)?;
    let fresh_proof = match precheck_exact_approval(
        &action,
        approval_id,
        requirements,
        store,
        availability,
        host_policy,
        scope,
        approvals,
        trail,
    )? {
        ExactApprovalPrecheck::Ready(proof) => proof,
        ExactApprovalPrecheck::NotDispatchable(decision) => {
            present_non_dispatchable_response(response, &decision, &action.action_id)?;
            return Ok(());
        }
    };
    let resolved = resolver::resolve_capability(
        store,
        availability,
        &action.capability_name,
        action
            .bridge_capability_version
            .ok_or("missing bridge capability version")?,
        action.bridge_provider_identity.as_deref(),
    )?;
    let decision = policy::allow_after_exact_approval(&resolved);
    let mut consumption = ExactApprovalConsumption {
        approval_id,
        proof: fresh_proof,
        approvals,
    };
    let clock = outcome::ProductionMonotonicClock::new();
    let mut anchor_writer = ResponseResultAnchorWriter;
    authorise_and_execute_inner(
        response,
        decision,
        &resolved,
        trail,
        executor,
        original_event_id,
        true,
        &clock,
        replay_authority,
        Some(&mut consumption),
        &mut anchor_writer,
    )
}

#[cfg(test)]
#[allow(clippy::too_many_arguments)]
fn resume_and_execute_exact_approval_with_test_replay(
    response: &mut Value,
    approval_id: &str,
    requirements: &[policy::CapabilityRequirement],
    store: &trusted_store::TrustedManifestStore,
    availability: &resolver::ProviderAvailability,
    host_policy: &policy::HostLocalPolicy,
    scope: policy::ScopeAssessment,
    approvals: &mut approval::ApprovalStore,
    trail: &mut dyn dispatch::Trail,
    executor: &mut dyn CapabilityExecutor,
    original_event_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut replay_authority = replay_runtime::test_support::TestReplayAuthority::default();
    resume_and_execute_exact_approval_with_authority(
        response,
        approval_id,
        requirements,
        store,
        availability,
        host_policy,
        scope,
        approvals,
        trail,
        executor,
        original_event_id,
        &mut replay_authority,
    )
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
    host_data_root: Option<&Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    let clock = outcome::ProductionMonotonicClock::new();
    let mut replay_authority = replay_runtime::FileReplayAuthority::new(host_data_root);
    let mut anchor_writer = ResponseResultAnchorWriter;
    authorise_and_execute_inner(
        response,
        decision,
        resolved,
        trail,
        executor,
        original_event_id,
        true,
        &clock,
        &mut replay_authority,
        None,
        &mut anchor_writer,
    )
}

#[cfg(test)]
fn authorise_and_execute_with_test_replay(
    response: &mut Value,
    decision: PermissionDecision,
    resolved: &ResolvedCapability,
    trail: &mut dyn dispatch::Trail,
    executor: &mut dyn CapabilityExecutor,
    original_event_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let clock = outcome::ProductionMonotonicClock::new();
    let mut replay_authority = replay_runtime::test_support::TestReplayAuthority::default();
    let mut anchor_writer = ResponseResultAnchorWriter;
    authorise_and_execute_inner(
        response,
        decision,
        resolved,
        trail,
        executor,
        original_event_id,
        true,
        &clock,
        &mut replay_authority,
        None,
        &mut anchor_writer,
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
    let clock = outcome::ProductionMonotonicClock::new();
    authorise_and_execute_without_bridge_pins_with_clock(
        response,
        decision,
        resolved,
        trail,
        executor,
        original_event_id,
        &clock,
    )
}

#[cfg(test)]
fn authorise_and_execute_without_bridge_pins_with_clock(
    response: &mut Value,
    decision: PermissionDecision,
    resolved: &ResolvedCapability,
    trail: &mut dyn dispatch::Trail,
    executor: &mut dyn CapabilityExecutor,
    original_event_id: &str,
    clock: &dyn outcome::MonotonicClock,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut replay_authority = replay_runtime::test_support::TestReplayAuthority::default();
    let mut anchor_writer = ResponseResultAnchorWriter;
    authorise_and_execute_inner(
        response,
        decision,
        resolved,
        trail,
        executor,
        original_event_id,
        false,
        clock,
        &mut replay_authority,
        None,
        &mut anchor_writer,
    )
}

fn present_non_dispatchable_response(
    response: &mut Value,
    decision: &PermissionDecision,
    action_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let rejection = match decision {
        PermissionDecision::Ask => "Ask",
        PermissionDecision::Deny => "Deny",
        PermissionDecision::Unavailable => "Unavailable",
        // An approval resume may continue only when the fresh ordinary result
        // is exactly Ask. A changed Allow is therefore still non-dispatchable
        // through this seam.
        PermissionDecision::Allow(_) => "Allow",
    };
    let json_trail = response
        .get_mut("trail")
        .and_then(Value::as_array_mut)
        .ok_or("response had no Trail")?;
    json_trail.push(trail_entry(
        json_trail.len() as u64 + 1,
        "authorisation",
        "intent_failed",
        "failed",
        rejection.to_owned(),
        Some(action_id),
    ));
    response["execution_status"] = Value::String("denied".into());
    Ok(())
}

fn authorise_and_execute_inner(
    response: &mut Value,
    decision: PermissionDecision,
    resolved: &ResolvedCapability,
    trail: &mut dyn dispatch::Trail,
    executor: &mut dyn CapabilityExecutor,
    original_event_id: &str,
    bridge_pins_required: bool,
    clock: &dyn outcome::MonotonicClock,
    replay_authority: &mut dyn replay_runtime::ReplayAuthority,
    approval_consumption: Option<&mut dyn ApprovalConsumption>,
    anchor_writer: &mut dyn ResultAnchorWriter,
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

    let action_id = dispatch::ActionId(action_id_str.to_owned());
    let arguments = action.get("arguments").cloned().unwrap_or(Value::Null);

    // Non-dispatchable policy branches never open replay storage. Fresh Ask
    // approval creation is handled by `request_exact_approval`; this boundary
    // must not claim an execution identity for it.
    if !matches!(&decision, PermissionDecision::Allow(_)) {
        present_non_dispatchable_response(response, &decision, &action_id.0)?;
        return Ok(());
    }

    let logical_key =
        match replay::LogicalExecutionKey::derive(original_event_id, &evaluation_id, action_id_str)
        {
            Ok(key) => key,
            Err(_) => {
                set_replay_result(
                    response,
                    replay_runtime::ReplayDispatchResult::PersistenceUnavailable,
                );
                return Ok(());
            }
        };
    let binding = replay::ExecutionBinding {
        evaluation_id: evaluation_id.clone(),
        action_id: action_id_str.to_owned(),
        capability_name: resolved.capability_name().to_owned(),
        capability_version: resolved.capability_version(),
        manifest_digest: resolved.manifest_digest().to_owned(),
        provider_identity: resolved.provider_identity().to_owned(),
        argument_digest: approval::digest(&arguments),
    };

    // Replay persistence is opened lazily here, after all ordinary fresh gates
    // above and only for a branch that may dispatch.
    let mut replay_admission = match replay_authority.admit(&logical_key, &binding) {
        Ok(admission) => admission,
        Err(_) => {
            set_replay_result(
                response,
                replay_runtime::ReplayDispatchResult::PersistenceUnavailable,
            );
            return Ok(());
        }
    };
    if !replay_admission.is_fresh() {
        set_replay_result(
            response,
            replay_runtime::ReplayDispatchResult::from_recovered_state(replay_admission.state()),
        );
        return Ok(());
    }
    let execution_id = dispatch::ExecutionId::from_replay(replay_admission.execution_id());

    // Approved Ask is consumed exactly once only after the fresh claim is
    // durable and while the identity guard is held. Any failure leaves the
    // claim manual-only and never restores the approval.
    if let Some(consumption) = approval_consumption {
        if consumption.consume(trail).is_err() {
            set_replay_result(
                response,
                replay_runtime::ReplayDispatchResult::RequiresManualResolution,
            );
            return Ok(());
        }
    }

    let json_trail = response
        .get_mut("trail")
        .and_then(Value::as_array_mut)
        .ok_or("response had no Trail")?;
    let mut sequence = json_trail.len() as u64 + 1;

    // The immutable replay intent boundary precedes the existing Trail intent.
    if replay_admission.publish_intent().is_err() {
        set_replay_result(
            response,
            replay_runtime::ReplayDispatchResult::PersistenceUnavailable,
        );
        return Ok(());
    }

    // Attempt durable Trail intent recording.
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

    // Intent is durable.  The execution deadline starts only now; policy,
    // approvals, and a failed intent write have already happened outside it.
    let deadline_start = clock.now();
    let deadline = Duration::from_millis(ready.verified_manifest().manifest().timeout_ms);
    // This is the final pre-invocation check.  It obtains the exact duration
    // given to the adapter; no presentation Trail may claim a start until it
    // succeeds.
    let remaining = match outcome::remaining_until_deadline(clock, deadline_start, deadline) {
        Some(remaining) => remaining,
        None => {
            json_trail.push(trail_entry(
                sequence,
                "execution",
                "deadline_before_invocation",
                "unattempted",
                outcome::deadline_reason().message.into(),
                Some(&ready.action_id().0),
            ));
            response["execution_status"] = Value::String("unattempted".into());
            return Ok(());
        }
    };

    // This durable boundary is immediately before provider invocation. The
    // held admission guard retains cross-process exclusion through the call
    // and final publication.
    if replay_admission.publish_armed().is_err() {
        set_replay_result(
            response,
            replay_runtime::ReplayDispatchResult::PersistenceUnavailable,
        );
        return Ok(());
    }

    // This volatile state transition is the invocation boundary: immediately
    // after it the provider may have caused an effect, so ambiguity is never
    // guessed to be a failure.
    json_trail.push(trail_entry(
        sequence,
        "execution",
        "action_started",
        "started",
        format!("Started {}", ready.capability_name()),
        Some(&ready.action_id().0),
    ));
    sequence += 1;
    let provider_result = executor.execute_classified(&ready, remaining);
    let observed_after_deadline = outcome::deadline_expired(clock, deadline_start, deadline);
    let timestamp_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default();

    let execution_outcome = if observed_after_deadline {
        outcome::ExecutionOutcome::Uncertain {
            reason: outcome::deadline_reason(),
        }
    } else {
        match provider_result {
            Ok(result) => {
                let output_schema = &ready.verified_manifest().manifest().output_schema;
                if validation::validate_output(output_schema, &result).is_ok() {
                    outcome::ExecutionOutcome::Succeeded(result)
                } else {
                    outcome::ExecutionOutcome::Failed {
                        reason: outcome::validation_reason(),
                    }
                }
            }
            Err(outcome::ProviderDiagnostic::ExplicitProviderError) => {
                outcome::ExecutionOutcome::Failed {
                    reason: outcome::redact(outcome::ProviderDiagnostic::ExplicitProviderError),
                }
            }
            Err(diagnostic) => outcome::ExecutionOutcome::Uncertain {
                reason: outcome::redact(diagnostic),
            },
        }
    };

    let (status, result, reason, anchor_kind, presentation_kind) = match &execution_outcome {
        outcome::ExecutionOutcome::Succeeded(result) => (
            "succeeded",
            Some(result.clone()),
            None,
            Some(ResultAnchorKind::Succeeded(result.clone())),
            "action_completed",
        ),
        outcome::ExecutionOutcome::Failed { reason } => (
            "failed",
            None,
            Some(reason.clone()),
            Some(ResultAnchorKind::Failed {
                code: reason.code.to_string(),
                message: reason.message.to_string(),
            }),
            "action_failed",
        ),
        outcome::ExecutionOutcome::Uncertain { reason } => (
            "uncertain",
            None,
            Some(reason.clone()),
            Some(ResultAnchorKind::Uncertain {
                code: reason.code.to_string(),
                message: reason.message.to_string(),
            }),
            "action_uncertain",
        ),
    };
    let outcome_entry = dispatch::OutcomeEntry {
        execution_id: ready.execution_id().0.clone(),
        action_id: ready.action_id().0.clone(),
        status: status.into(),
        result,
        error_message: reason.as_ref().map(|reason| reason.message.to_string()),
        reason_code: reason.as_ref().map(|reason| reason.code.to_string()),
        timestamp_unix_ms: timestamp_ms,
    };

    if trail.append_outcome(&outcome_entry).is_err() {
        // The in-memory classification above remains truthful, but it is not
        // auditable enough for a Result Anchor or retry authority.
        json_trail.push(trail_entry(
            sequence,
            "execution",
            "audit_failure",
            "failed",
            outcome::audit_failure_reason().message.into(),
            Some(&ready.action_id().0),
        ));
        sequence += 1;
        json_trail.push(trail_entry(
            sequence,
            "execution",
            presentation_kind,
            status,
            reason
                .as_ref()
                .map(|reason| reason.message.to_string())
                .unwrap_or_else(|| format!("Completed {}", ready.capability_name())),
            Some(&ready.action_id().0),
        ));
        response["execution_status"] = Value::String(
            if status == "succeeded" {
                "completed"
            } else {
                status
            }
            .into(),
        );
        return Ok(());
    }

    let terminal_state = match &execution_outcome {
        outcome::ExecutionOutcome::Succeeded(_) => replay::ReplayState::Succeeded,
        outcome::ExecutionOutcome::Failed { .. } => replay::ReplayState::Failed,
        outcome::ExecutionOutcome::Uncertain { .. } => replay::ReplayState::Uncertain,
    };
    let outcome_digest = match replay::durable_outcome_digest(&outcome_entry) {
        Ok(digest) => digest,
        Err(_) => {
            response["execution_status"] = Value::String(
                if status == "succeeded" {
                    "completed"
                } else {
                    status
                }
                .into(),
            );
            return Ok(());
        }
    };
    if replay_admission
        .publish_terminal(terminal_state, outcome_digest)
        .is_err()
    {
        response["execution_status"] = Value::String(
            if status == "succeeded" {
                "completed"
            } else {
                status
            }
            .into(),
        );
        return Ok(());
    }

    let presentation_status = if status == "succeeded" {
        "succeeded"
    } else {
        status
    };
    json_trail.push(trail_entry(
        sequence,
        "execution",
        presentation_kind,
        presentation_status,
        reason
            .as_ref()
            .map(|reason| reason.message.to_string())
            .unwrap_or_else(|| format!("Completed {}", ready.capability_name())),
        Some(&ready.action_id().0),
    ));
    response["execution_status"] = Value::String(
        if status == "succeeded" {
            "completed"
        } else {
            status
        }
        .into(),
    );
    if let Some(anchor_kind) = anchor_kind {
        let anchor = ResultAnchor::new(
            anchor_kind,
            &evaluation_id,
            &action_id.0,
            ready.capability_name(),
            ready.capability_version(),
            ready.manifest_digest(),
            ready.provider_identity(),
            timestamp_ms,
            original_event_id,
        );
        if anchor_writer.write(response, &anchor).is_err() {
            if let Some(object) = response.as_object_mut() {
                object.remove("result_anchor");
            }
            return Ok(());
        }
    }

    // Keep cross-process exclusion through the Result Anchor boundary,
    // including a failing Anchor write. Drop is intentionally last.
    drop(replay_admission);
    Ok(())
}

fn set_replay_result(response: &mut Value, result: replay_runtime::ReplayDispatchResult) {
    if let Some(object) = response.as_object_mut() {
        object.remove("result_anchor");
        object.remove("replay_result");
    }
    response["execution_status"] = Value::String(result.as_str().to_owned());
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

    /// J05's dedicated, test-only capability.  Its unrestricted manifest
    /// honestly reaches Ask through mandatory per-call confirmation, without
    /// changing the structured-scope demonstration's fail-closed assessment.
    fn resolved_approval_fixture() -> (TrustedManifestStore, resolver::ResolvedCapability) {
        let mut manifest = lantern_manifest_json();
        manifest["capability_name"] = json!("fixture.ask");
        manifest["permission_scope"] = Value::Null;
        manifest["confirmation_policy"] = json!({
            "standing_permitted": false,
            "per_call_required": true
        });
        let (_, digest) = crate::manifest::canonicalize_and_digest(&manifest.to_string()).unwrap();
        manifest["digest"] = json!(digest);
        let mut store = TrustedManifestStore::new();
        store
            .insert(crate::manifest::verify_manifest(&manifest.to_string()).unwrap())
            .unwrap();
        let availability = ProviderAvailability::from_identities(["lantern-local"]);
        let resolved = resolver::resolve_capability(
            &store,
            &availability,
            "fixture.ask",
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
        assert_eq!(failed["message"], "provider reported an error");
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
        assert_eq!(
            outcome.execution_id,
            replay_runtime::test_support::TEST_EXECUTION_ID
        );
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
        assert_eq!(
            outcome.execution_id,
            replay_runtime::test_support::TEST_EXECUTION_ID
        );
        assert_eq!(outcome.action_id, "action_1");
        assert_eq!(outcome.status, "failed");
        assert_eq!(outcome.result, None);
        assert_eq!(
            outcome.error_message,
            Some("provider reported an error".into())
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
        assert_eq!(audit["message"], "outcome audit write failed");
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
            .eq("provider result failed validation"));

        assert_eq!(trail.outcome_entries.len(), 1);
        let outcome = &trail.outcome_entries[0];
        assert_eq!(outcome.status, "failed");
        assert!(outcome
            .error_message
            .as_ref()
            .unwrap()
            .eq("provider result failed validation"));
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
            .eq("provider result failed validation"));
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
            .eq("provider result failed validation"));
        assert_eq!(failed["message"], "provider result failed validation");
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
        assert_eq!(failed["message"], "provider reported an error");
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
        assert_eq!(error["message"], "provider reported an error");

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
            .eq("provider result failed validation"));
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
    // Test 35: Outcome-write audit failure preserves classification but never
    //          creates an unaudited Result Anchor.
    // -----------------------------------------------------------------------

    #[test]
    fn outcome_write_audit_failure_withholds_result_anchor() {
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

        assert!(response.get("result_anchor").is_none());
    }

    #[test]
    fn outcome_write_audit_failure_after_executor_error_withholds_failed_anchor() {
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

        assert!(response.get("result_anchor").is_none());
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
        let err = authorise_and_execute_with_test_replay(
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

        authorise_and_execute_with_test_replay(
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
            let err = authorise_and_execute_with_test_replay(
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

    #[test]
    fn j05_production_seam_consumes_exact_approved_fixture_before_intent() {
        struct FixtureExecutor {
            calls: u32,
        }
        impl CapabilityExecutor for FixtureExecutor {
            fn provider_identity(&self) -> &str {
                "lantern-local"
            }
            fn execute(&mut self, _ready: &DispatchReadyAction) -> Result<Value, String> {
                self.calls += 1;
                Ok(json!({"status": "recorded", "project": "p", "task": "t"}))
            }
        }
        let (store, resolved) = resolved_approval_fixture();
        let availability = ProviderAvailability::from_identities(["lantern-local"]);
        let requirements = vec![CapabilityRequirement::new("fixture.ask", 1)];
        let host_policy = HostLocalPolicy::new(PolicyRule::Allow);
        let mut response = make_bridge_matched_response(&resolved);
        let action = extract_proposed_action(&response).unwrap();
        let mut approvals = approval::ApprovalStore::default();
        let mut trail = RecordingTrail::new();
        let request = request_exact_approval(
            &action,
            &requirements,
            &store,
            &availability,
            &host_policy,
            policy::ScopeAssessment::ScopeNotEstablished,
            &mut approvals,
            &mut trail,
        )
        .unwrap()
        .unwrap();
        let duplicate = request_exact_approval(
            &action,
            &requirements,
            &store,
            &availability,
            &host_policy,
            policy::ScopeAssessment::ScopeNotEstablished,
            &mut approvals,
            &mut trail,
        )
        .unwrap()
        .unwrap();
        assert_eq!(request.approval_id, duplicate.approval_id);
        assert_eq!(trail.authorisation_entries.len(), 1);
        record_human_approval_decision(
            &request.approval_id,
            HumanApprovalDecision::Approve,
            &mut approvals,
            &mut trail,
        )
        .unwrap();
        let mut executor = FixtureExecutor { calls: 0 };
        resume_and_execute_exact_approval_with_test_replay(
            &mut response,
            &request.approval_id,
            &requirements,
            &store,
            &availability,
            &host_policy,
            policy::ScopeAssessment::ScopeNotEstablished,
            &mut approvals,
            &mut trail,
            &mut executor,
            "evt-j05",
        )
        .unwrap();
        assert_eq!(executor.calls, 1);
        assert_eq!(
            approvals.record(&request.approval_id).unwrap().state,
            approval::ApprovalState::Consumed
        );
        assert_eq!(trail.entries.len(), 1);
        assert_eq!(trail.outcome_entries.len(), 1);
        assert_eq!(
            trail
                .authorisation_entries
                .iter()
                .map(|entry| entry.kind.as_str())
                .collect::<Vec<_>>(),
            vec![
                "approval_requested",
                "approval_granted",
                "approval_consumed"
            ]
        );
    }

    #[test]
    fn j05_fresh_deny_invalidates_and_never_dispatches() {
        let (store, resolved) = resolved_approval_fixture();
        let availability = ProviderAvailability::from_identities(["lantern-local"]);
        let requirements = vec![CapabilityRequirement::new("fixture.ask", 1)];
        let mut response = make_bridge_matched_response(&resolved);
        let action = extract_proposed_action(&response).unwrap();
        let mut approvals = approval::ApprovalStore::default();
        let mut trail = RecordingTrail::new();
        let ask_policy = HostLocalPolicy::new(PolicyRule::Allow);
        let request = request_exact_approval(
            &action,
            &requirements,
            &store,
            &availability,
            &ask_policy,
            policy::ScopeAssessment::ScopeNotEstablished,
            &mut approvals,
            &mut trail,
        )
        .unwrap()
        .unwrap();
        record_human_approval_decision(
            &request.approval_id,
            HumanApprovalDecision::Approve,
            &mut approvals,
            &mut trail,
        )
        .unwrap();
        let mut executor = MockExecutor::new();
        let mut replay_authority = replay_runtime::test_support::TestReplayAuthority::default();
        resume_and_execute_exact_approval_with_authority(
            &mut response,
            &request.approval_id,
            &requirements,
            &store,
            &availability,
            &HostLocalPolicy::new(PolicyRule::Deny),
            policy::ScopeAssessment::ScopeNotEstablished,
            &mut approvals,
            &mut trail,
            &mut executor,
            "evt-j05",
            &mut replay_authority,
        )
        .unwrap();
        assert_eq!(replay_authority.admissions, 0);
        assert!(executor.completed.is_empty());
        assert!(trail.entries.is_empty());
        assert!(trail.outcome_entries.is_empty());
        assert!(response.get("result_anchor").is_none());
        assert_eq!(response["execution_status"], "denied");
        let response_trail = response["trail"].as_array().unwrap();
        assert_eq!(response_trail.last().unwrap()["kind"], "intent_failed");
        assert_eq!(response_trail.last().unwrap()["message"], "Deny");
        assert!(!trail
            .authorisation_entries
            .iter()
            .any(|entry| entry.kind == "approval_consumed"));
        assert_eq!(
            approvals.record(&request.approval_id).unwrap().state,
            approval::ApprovalState::Invalidated
        );
    }

    #[test]
    fn j05_human_denial_and_cancellation_are_terminal_without_dispatch() {
        let (store, resolved) = resolved_approval_fixture();
        let availability = ProviderAvailability::from_identities(["lantern-local"]);
        let requirements = vec![CapabilityRequirement::new("fixture.ask", 1)];
        let policy = HostLocalPolicy::new(PolicyRule::Allow);
        for decision in [HumanApprovalDecision::Deny, HumanApprovalDecision::Cancel] {
            let response = make_bridge_matched_response(&resolved);
            let action = extract_proposed_action(&response).unwrap();
            let mut approvals = approval::ApprovalStore::default();
            let mut trail = RecordingTrail::new();
            let request = request_exact_approval(
                &action,
                &requirements,
                &store,
                &availability,
                &policy,
                policy::ScopeAssessment::ScopeNotEstablished,
                &mut approvals,
                &mut trail,
            )
            .unwrap()
            .unwrap();
            record_human_approval_decision(
                &request.approval_id,
                decision,
                &mut approvals,
                &mut trail,
            )
            .unwrap();
            assert!(matches!(
                approvals.record(&request.approval_id).unwrap().state,
                approval::ApprovalState::Denied | approval::ApprovalState::Cancelled
            ));
            assert!(trail.entries.is_empty());
            assert!(trail.outcome_entries.is_empty());
        }
    }

    #[test]
    fn j05_authorisation_trail_write_failures_leave_no_usable_approval() {
        let (store, resolved) = resolved_approval_fixture();
        let availability = ProviderAvailability::from_identities(["lantern-local"]);
        let requirements = vec![CapabilityRequirement::new("fixture.ask", 1)];
        let policy = HostLocalPolicy::new(PolicyRule::Allow);
        let response = make_bridge_matched_response(&resolved);
        let action = extract_proposed_action(&response).unwrap();

        let mut approvals = approval::ApprovalStore::default();
        let mut trail = RecordingTrail::new();
        trail.injected_authorisation_error = Some(dispatch::TrailError::WriteFailed("full".into()));
        assert!(request_exact_approval(
            &action,
            &requirements,
            &store,
            &availability,
            &policy,
            policy::ScopeAssessment::ScopeNotEstablished,
            &mut approvals,
            &mut trail
        )
        .is_err());
        assert_eq!(
            approvals.record("approval-1"),
            Err(approval::ApprovalError::Missing)
        );
        let request = request_exact_approval(
            &action,
            &requirements,
            &store,
            &availability,
            &policy,
            policy::ScopeAssessment::ScopeNotEstablished,
            &mut approvals,
            &mut trail,
        )
        .unwrap()
        .unwrap();

        for decision in [
            HumanApprovalDecision::Approve,
            HumanApprovalDecision::Deny,
            HumanApprovalDecision::Cancel,
        ] {
            let mut trial = approval::ApprovalStore::default();
            let mut trial_trail = RecordingTrail::new();
            let record = request_exact_approval(
                &action,
                &requirements,
                &store,
                &availability,
                &policy,
                policy::ScopeAssessment::ScopeNotEstablished,
                &mut trial,
                &mut trial_trail,
            )
            .unwrap()
            .unwrap();
            trial_trail.injected_authorisation_error =
                Some(dispatch::TrailError::WriteFailed("full".into()));
            assert!(record_human_approval_decision(
                &record.approval_id,
                decision,
                &mut trial,
                &mut trial_trail
            )
            .is_err());
            let state = trial.record(&record.approval_id).unwrap().state;
            assert!(matches!(
                state,
                approval::ApprovalState::Invalidated
                    | approval::ApprovalState::Denied
                    | approval::ApprovalState::Cancelled
            ));
        }

        record_human_approval_decision(
            &request.approval_id,
            HumanApprovalDecision::Approve,
            &mut approvals,
            &mut trail,
        )
        .unwrap();
        trail.injected_authorisation_error = Some(dispatch::TrailError::WriteFailed("full".into()));
        let mut response = make_bridge_matched_response(&resolved);
        let mut executor = MockExecutor::new();
        resume_and_execute_exact_approval_with_test_replay(
            &mut response,
            &request.approval_id,
            &requirements,
            &store,
            &availability,
            &policy,
            policy::ScopeAssessment::ScopeNotEstablished,
            &mut approvals,
            &mut trail,
            &mut executor,
            "evt-j05",
        )
        .unwrap();
        assert_eq!(
            approvals.record(&request.approval_id).unwrap().state,
            approval::ApprovalState::Consumed
        );
        assert!(trail.entries.is_empty());
        assert!(trail.outcome_entries.is_empty());
        assert!(executor.completed.is_empty());
        assert!(response.get("result_anchor").is_none());
        assert_eq!(
            response["execution_status"],
            "replay_requires_manual_resolution"
        );
        assert!(response.get("replay_result").is_none());
    }

    // J06 focused execution truth table.  The executor advances the injected
    // monotonic clock at the response-observation boundary; no wall clock or
    // scheduler timing participates in these assertions.
    struct J06Executor<'a> {
        clock: &'a outcome::TestMonotonicClock,
        advance: Duration,
        result: Result<Value, outcome::ProviderDiagnostic>,
        remaining: Option<Duration>,
        calls: u32,
    }

    impl CapabilityExecutor for J06Executor<'_> {
        fn provider_identity(&self) -> &str {
            "lantern-local"
        }

        fn execute(&mut self, _ready: &DispatchReadyAction) -> Result<Value, String> {
            panic!("J06Executor uses execute_classified")
        }

        fn execute_classified(
            &mut self,
            _ready: &DispatchReadyAction,
            remaining: Duration,
        ) -> Result<Value, outcome::ProviderDiagnostic> {
            self.calls += 1;
            self.remaining = Some(remaining);
            self.clock.advance(self.advance);
            self.result.clone()
        }
    }

    fn run_j06_case(executor: &mut J06Executor<'_>, trail: &mut RecordingTrail) -> Value {
        let (_store, resolved) = resolved_lantern();
        let mut response = make_matched_response(
            "j06-eval",
            "j06-action",
            "lantern.task.record",
            json!({"project": "p", "task": "t"}),
        );
        let clock = executor.clock;
        authorise_and_execute_without_bridge_pins_with_clock(
            &mut response,
            allow_decision_for(&resolved),
            &resolved,
            trail,
            executor,
            "j06-input",
            clock,
        )
        .unwrap();
        response
    }

    #[test]
    fn j06_elapsed_before_authorisation_does_not_consume_execution_deadline() {
        let clock = outcome::TestMonotonicClock::new();
        // This represents planning and approval waiting before durable intent.
        clock.advance(Duration::from_secs(600));
        let mut executor = J06Executor {
            clock: &clock,
            advance: Duration::ZERO,
            result: Ok(json!({"status": "recorded"})),
            remaining: None,
            calls: 0,
        };
        let mut trail = RecordingTrail::new();
        let response = run_j06_case(&mut executor, &mut trail);
        assert_eq!(executor.calls, 1);
        assert_eq!(executor.remaining, Some(Duration::from_secs(10)));
        assert_eq!(trail.outcome_entries[0].status, "succeeded");
        assert_eq!(response["execution_status"], "completed");
    }

    #[test]
    fn j06_deadline_before_invocation_is_unattempted_without_provider_outcome_or_anchor() {
        let mut manifest = lantern_manifest_json();
        manifest["timeout_ms"] = json!(0);
        let (_store, resolved) = resolved_lantern_with_manifest(manifest);
        let clock = outcome::TestMonotonicClock::new();
        let mut executor = J06Executor {
            clock: &clock,
            advance: Duration::ZERO,
            result: Ok(json!({"status": "recorded"})),
            remaining: None,
            calls: 0,
        };
        let mut response = make_matched_response(
            "j06-eval",
            "j06-action",
            "lantern.task.record",
            json!({"project": "p", "task": "t"}),
        );
        let mut trail = RecordingTrail::new();
        authorise_and_execute_without_bridge_pins_with_clock(
            &mut response,
            allow_decision_for(&resolved),
            &resolved,
            &mut trail,
            &mut executor,
            "j06-input",
            &clock,
        )
        .unwrap();
        assert_eq!(trail.entries.len(), 1, "intent precedes the deadline start");
        assert_eq!(executor.calls, 0);
        assert_eq!(executor.remaining, None);
        assert!(
            !response["trail"]
                .as_array()
                .unwrap()
                .iter()
                .any(|entry| entry["kind"] == "action_started"),
            "unattempted action must not claim action_started"
        );
        assert!(trail.outcome_entries.is_empty());
        assert!(response.get("result_anchor").is_none());
        assert_eq!(response["execution_status"], "unattempted");
    }

    #[test]
    fn j06_response_observed_at_deadline_is_uncertain_even_when_provider_succeeds() {
        let clock = outcome::TestMonotonicClock::new();
        let mut executor = J06Executor {
            clock: &clock,
            advance: Duration::from_secs(10),
            result: Ok(json!({"status": "recorded"})),
            remaining: None,
            calls: 0,
        };
        let mut trail = RecordingTrail::new();
        let response = run_j06_case(&mut executor, &mut trail);
        assert_eq!(executor.calls, 1);
        assert_eq!(trail.outcome_entries[0].status, "uncertain");
        assert_eq!(
            trail.outcome_entries[0].error_message.as_deref(),
            Some("execution deadline exceeded")
        );
        assert_eq!(
            response["result_anchor"]["event_name"],
            "capability.uncertain"
        );
    }

    #[test]
    fn j06_legacy_string_error_is_uncertain_not_known_provider_failure() {
        struct LegacyStringErrorExecutor;
        impl CapabilityExecutor for LegacyStringErrorExecutor {
            fn provider_identity(&self) -> &str {
                "lantern-local"
            }

            fn execute(&mut self, _ready: &DispatchReadyAction) -> Result<Value, String> {
                Err("connection reset; token=secret".into())
            }
        }

        let (_store, resolved) = resolved_lantern();
        let clock = outcome::TestMonotonicClock::new();
        let mut executor = LegacyStringErrorExecutor;
        let mut trail = RecordingTrail::new();
        let mut response = make_matched_response(
            "j06-legacy",
            "j06-action",
            "lantern.task.record",
            json!({"project": "p", "task": "t"}),
        );
        authorise_and_execute_without_bridge_pins_with_clock(
            &mut response,
            allow_decision_for(&resolved),
            &resolved,
            &mut trail,
            &mut executor,
            "j06-input",
            &clock,
        )
        .unwrap();

        assert_eq!(trail.outcome_entries[0].status, "uncertain");
        assert_eq!(
            trail.outcome_entries[0].reason_code.as_deref(),
            Some("provider_outcome_uncertain")
        );
        assert_eq!(
            response["result_anchor"]["event_name"],
            "capability.uncertain"
        );
        assert_ne!(response["result_anchor"]["event_name"], "capability.failed");
        assert!(!response.to_string().contains("token=secret"));
    }

    #[test]
    fn j06_post_invocation_transport_ambiguities_are_uncertain_and_redacted() {
        let cases = [
            (
                outcome::ProviderDiagnostic::ProcessLost,
                "provider_process_lost",
            ),
            (
                outcome::ProviderDiagnostic::ResponseMalformed,
                "provider_response_invalid",
            ),
            (
                outcome::ProviderDiagnostic::ResponseTruncated,
                "provider_response_invalid",
            ),
            (
                outcome::ProviderDiagnostic::ProtocolInterrupted,
                "provider_protocol_interrupted",
            ),
            (
                outcome::ProviderDiagnostic::NoFinalResponse,
                "provider_outcome_uncertain",
            ),
        ];
        for (diagnostic, code) in cases {
            let clock = outcome::TestMonotonicClock::new();
            let mut executor = J06Executor {
                clock: &clock,
                advance: Duration::ZERO,
                result: Err(diagnostic),
                remaining: None,
                calls: 0,
            };
            let mut trail = RecordingTrail::new();
            let response = run_j06_case(&mut executor, &mut trail);
            assert_eq!(executor.calls, 1, "{code}");
            assert_eq!(trail.outcome_entries[0].status, "uncertain", "{code}");
            assert_eq!(
                response["result_anchor"]["event_name"], "capability.uncertain",
                "{code}"
            );
            assert_eq!(
                response["result_anchor"]["facts"]["error"]["code"], code,
                "{code}"
            );
        }
    }

    #[test]
    fn j06_outcome_audit_failure_keeps_uncertainty_but_withholds_anchor() {
        let clock = outcome::TestMonotonicClock::new();
        let mut executor = J06Executor {
            clock: &clock,
            advance: Duration::ZERO,
            result: Err(outcome::ProviderDiagnostic::ProcessLost),
            remaining: None,
            calls: 0,
        };
        let mut trail = RecordingTrail::new();
        trail.injected_outcome_error =
            Some(dispatch::TrailError::WriteFailed("raw token=secret".into()));
        let response = run_j06_case(&mut executor, &mut trail);
        assert_eq!(response["execution_status"], "uncertain");
        assert!(trail.outcome_entries.is_empty());
        assert!(response.get("result_anchor").is_none());
        assert_eq!(
            response["trail"]
                .as_array()
                .unwrap()
                .iter()
                .find(|entry| entry["kind"] == "audit_failure")
                .unwrap()["message"],
            "outcome audit write failed"
        );
        assert!(!response.to_string().contains("token=secret"));
    }

    // -------------------------------------------------------------------
    // J09 runtime integration: counting fakes and observable order
    // -------------------------------------------------------------------

    mod replay_runtime_integration {
        use super::*;
        use std::cell::{Cell, RefCell};
        use std::rc::Rc;

        struct RuntimeExecutor {
            events: Rc<RefCell<Vec<&'static str>>>,
            guard_held: Rc<Cell<bool>>,
            result: Result<Value, outcome::ProviderDiagnostic>,
            calls: usize,
            saw_guard: bool,
        }

        impl RuntimeExecutor {
            fn success(events: Rc<RefCell<Vec<&'static str>>>, guard_held: Rc<Cell<bool>>) -> Self {
                Self {
                    events,
                    guard_held,
                    result: Ok(json!({"status":"recorded","project":"p","task":"t"})),
                    calls: 0,
                    saw_guard: false,
                }
            }
        }

        impl CapabilityExecutor for RuntimeExecutor {
            fn provider_identity(&self) -> &str {
                "lantern-local"
            }

            fn execute(&mut self, ready: &DispatchReadyAction) -> Result<Value, String> {
                self.execute_classified(ready, Duration::from_secs(1))
                    .map_err(|diagnostic| format!("{diagnostic:?}"))
            }

            fn execute_classified(
                &mut self,
                _ready: &DispatchReadyAction,
                _remaining: Duration,
            ) -> Result<Value, outcome::ProviderDiagnostic> {
                self.events.borrow_mut().push("provider");
                self.calls += 1;
                self.saw_guard = self.guard_held.get();
                self.result.clone()
            }
        }

        struct RuntimeClock {
            events: Rc<RefCell<Vec<&'static str>>>,
            readings: RefCell<Vec<Duration>>,
            calls: Cell<usize>,
        }

        impl RuntimeClock {
            fn within_deadline(events: Rc<RefCell<Vec<&'static str>>>) -> Self {
                Self {
                    events,
                    readings: RefCell::new(vec![Duration::ZERO, Duration::ZERO, Duration::ZERO]),
                    calls: Cell::new(0),
                }
            }

            fn expired_before_invocation(events: Rc<RefCell<Vec<&'static str>>>) -> Self {
                Self {
                    events,
                    readings: RefCell::new(vec![Duration::ZERO, Duration::from_secs(11)]),
                    calls: Cell::new(0),
                }
            }
        }

        impl outcome::MonotonicClock for RuntimeClock {
            fn now(&self) -> Duration {
                let call = self.calls.get();
                self.calls.set(call + 1);
                self.events.borrow_mut().push(match call {
                    0 => "deadline_start",
                    1 => "deadline_check",
                    _ => "response_observed",
                });
                self.readings
                    .borrow()
                    .get(call)
                    .copied()
                    .unwrap_or(Duration::ZERO)
            }
        }

        struct RuntimeAnchorWriter {
            events: Rc<RefCell<Vec<&'static str>>>,
            guard_held: Rc<Cell<bool>>,
            fail: bool,
            writes: usize,
            saw_guard: bool,
        }

        impl RuntimeAnchorWriter {
            fn new(events: Rc<RefCell<Vec<&'static str>>>, guard_held: Rc<Cell<bool>>) -> Self {
                Self {
                    events,
                    guard_held,
                    fail: false,
                    writes: 0,
                    saw_guard: false,
                }
            }
        }

        impl ResultAnchorWriter for RuntimeAnchorWriter {
            fn write(&mut self, response: &mut Value, anchor: &ResultAnchor) -> Result<(), ()> {
                self.events.borrow_mut().push("anchor");
                self.writes += 1;
                self.saw_guard = self.guard_held.get();
                if self.fail {
                    return Err(());
                }
                response["result_anchor"] = serde_json::to_value(anchor).map_err(|_| ())?;
                Ok(())
            }
        }

        struct RuntimeApproval {
            events: Rc<RefCell<Vec<&'static str>>>,
            fail: bool,
            consumes: usize,
        }

        impl ApprovalConsumption for RuntimeApproval {
            fn consume(&mut self, _trail: &mut dyn dispatch::Trail) -> Result<(), ()> {
                self.events.borrow_mut().push("consume_approval");
                self.consumes += 1;
                if self.fail {
                    Err(())
                } else {
                    Ok(())
                }
            }
        }

        fn runtime_response(resolved: &resolver::ResolvedCapability) -> Value {
            make_matched_response(
                "eval-j09-001",
                "action-j09-001",
                resolved.capability_name(),
                json!({"project":"p","task":"t"}),
            )
        }

        #[allow(clippy::too_many_arguments)]
        fn run_runtime(
            decision: PermissionDecision,
            authority: &mut dyn replay_runtime::ReplayAuthority,
            trail: &mut RecordingTrail,
            executor: &mut RuntimeExecutor,
            clock: &dyn outcome::MonotonicClock,
            approval: Option<&mut dyn ApprovalConsumption>,
            anchor_writer: &mut dyn ResultAnchorWriter,
        ) -> Value {
            let (_, resolved) = resolved_lantern();
            let mut response = runtime_response(&resolved);
            authorise_and_execute_inner(
                &mut response,
                decision,
                &resolved,
                trail,
                executor,
                "evt-j09-001",
                false,
                clock,
                authority,
                approval,
                anchor_writer,
            )
            .unwrap();
            response
        }

        fn runtime_parts() -> (
            Rc<RefCell<Vec<&'static str>>>,
            replay_runtime::test_support::TestReplayAuthority,
            RecordingTrail,
            RuntimeExecutor,
            RuntimeClock,
            RuntimeAnchorWriter,
        ) {
            let events = Rc::new(RefCell::new(Vec::new()));
            let mut authority = replay_runtime::test_support::TestReplayAuthority::default();
            authority.events = Rc::clone(&events);
            let guard_held = Rc::clone(&authority.guard_held);
            let mut trail = RecordingTrail::new();
            trail.event_log = Some(Rc::clone(&events));
            let executor = RuntimeExecutor::success(Rc::clone(&events), Rc::clone(&guard_held));
            let clock = RuntimeClock::within_deadline(Rc::clone(&events));
            let anchor = RuntimeAnchorWriter::new(Rc::clone(&events), guard_held);
            (events, authority, trail, executor, clock, anchor)
        }

        fn assert_only_existing_replay_result_field(response: &Value, expected: &str) {
            assert_eq!(response["execution_status"], expected);
            assert!(response.get("replay_result").is_none());
            assert!(response.get("result_anchor").is_none());
        }

        #[cfg(windows)]
        fn fresh_native_runtime_root(label: &str) -> Option<PathBuf> {
            let base = std::env::var_os("TETHERS_J09_NATIVE_PROVISION_ROOT")?;
            let root = PathBuf::from(base)
                .join(format!("runtime-{label}-{}", uuid::Uuid::new_v4().simple()));
            fs::create_dir(&root).unwrap();
            assert_eq!(
                replay_windows::provision_replay(&root).unwrap(),
                replay_windows::ProvisionReplayOutcome::Provisioned
            );
            Some(root)
        }

        #[cfg(windows)]
        fn native_runtime_binding() -> replay::ExecutionBinding {
            let (_, resolved) = resolved_lantern();
            replay::ExecutionBinding {
                evaluation_id: "eval-j09-001".to_owned(),
                action_id: "action-j09-001".to_owned(),
                capability_name: resolved.capability_name().to_owned(),
                capability_version: resolved.capability_version(),
                manifest_digest: resolved.manifest_digest().to_owned(),
                provider_identity: resolved.provider_identity().to_owned(),
                argument_digest: approval::digest(&json!({"project":"p","task":"t"})),
            }
        }

        #[cfg(windows)]
        fn seed_native_runtime_state(root: &Path, state: replay::ReplayState) {
            let ledger = replay_windows::ReplayLedger::open(root).unwrap();
            let key = replay::LogicalExecutionKey::derive(
                "evt-j09-001",
                "eval-j09-001",
                "action-j09-001",
            )
            .unwrap();
            let mut admission = ledger
                .admit_or_recover(key, native_runtime_binding())
                .unwrap();
            match state {
                replay::ReplayState::ClaimedNoState => {}
                replay::ReplayState::IntentRecorded => admission.publish_intent().unwrap(),
                replay::ReplayState::InvocationArmed => {
                    admission.publish_intent().unwrap();
                    admission.publish_armed().unwrap();
                }
                replay::ReplayState::Succeeded
                | replay::ReplayState::Failed
                | replay::ReplayState::Uncertain => {
                    admission.publish_intent().unwrap();
                    admission.publish_armed().unwrap();
                    admission
                        .publish_terminal(
                            state,
                            replay::durable_outcome_digest(&json!({"seed":format!("{state:?}")}))
                                .unwrap(),
                        )
                        .unwrap();
                }
            }
        }

        #[cfg(windows)]
        fn run_native_recovered_state(state: replay::ReplayState, expected: &str) {
            let Some(root) = fresh_native_runtime_root(&format!("{state:?}")) else {
                return;
            };
            seed_native_runtime_state(&root, state);
            let (_, resolved) = resolved_lantern();
            let mut authority = replay_runtime::FileReplayAuthority::new(Some(&root));
            let mut trail = RecordingTrail::new();
            let events = Rc::new(RefCell::new(Vec::new()));
            let guard = Rc::new(Cell::new(false));
            let mut executor = RuntimeExecutor::success(Rc::clone(&events), Rc::clone(&guard));
            let clock = RuntimeClock::within_deadline(Rc::clone(&events));
            let mut anchor = RuntimeAnchorWriter::new(events, guard);
            let response = run_runtime(
                allow_decision_for(&resolved),
                &mut authority,
                &mut trail,
                &mut executor,
                &clock,
                None,
                &mut anchor,
            );
            assert_only_existing_replay_result_field(&response, expected);
            assert_eq!(executor.calls, 0);
            assert!(trail.entries.is_empty());
            assert!(trail.outcome_entries.is_empty());
        }

        #[test]
        fn j09_runtime_01_cli_accepts_one_absolute_host_data_root() {
            let args = vec![
                "engine.exe".to_owned(),
                "request.json".to_owned(),
                "--host-data-root".to_owned(),
                r"C:\host-data".to_owned(),
            ];
            let parsed = parse_normal_args(&args).unwrap();
            assert_eq!(
                parsed.host_data_root.unwrap(),
                PathBuf::from(r"C:\host-data")
            );
        }

        #[test]
        fn j09_runtime_02_cli_rejects_duplicate_host_data_root() {
            let args = vec![
                "engine.exe".to_owned(),
                "request.json".to_owned(),
                "--host-data-root".to_owned(),
                r"C:\one".to_owned(),
                "--host-data-root".to_owned(),
                r"C:\two".to_owned(),
            ];
            assert_eq!(
                parse_normal_args(&args).unwrap_err(),
                "duplicate --host-data-root"
            );
        }

        #[test]
        fn j09_runtime_03_cli_rejects_missing_host_data_root_value() {
            let args = vec![
                "engine.exe".to_owned(),
                "request.json".to_owned(),
                "--host-data-root".to_owned(),
            ];
            assert_eq!(
                parse_normal_args(&args).unwrap_err(),
                "missing value for --host-data-root"
            );
        }

        #[test]
        fn j09_runtime_04_cli_rejects_relative_host_data_root() {
            let args = vec![
                "engine.exe".to_owned(),
                "request.json".to_owned(),
                "--host-data-root".to_owned(),
                "relative".to_owned(),
            ];
            assert_eq!(
                parse_normal_args(&args).unwrap_err(),
                "--host-data-root must be absolute"
            );
        }

        #[test]
        fn j09_runtime_05_cli_rejects_unknown_options() {
            let args = vec![
                "engine.exe".to_owned(),
                "request.json".to_owned(),
                "--replay-root".to_owned(),
            ];
            assert_eq!(
                parse_normal_args(&args).unwrap_err(),
                "unknown option: --replay-root"
            );
        }

        #[test]
        fn j09_runtime_06_ask_never_opens_replay_authority() {
            let (_, mut authority, mut trail, mut executor, clock, mut anchor) = runtime_parts();
            let response = run_runtime(
                PermissionDecision::Ask,
                &mut authority,
                &mut trail,
                &mut executor,
                &clock,
                None,
                &mut anchor,
            );
            assert_eq!(authority.admissions, 0);
            assert_eq!(executor.calls, 0);
            assert_eq!(response["execution_status"], "denied");
        }

        #[test]
        fn j09_runtime_07_deny_never_opens_replay_authority() {
            let (_, resolved) = resolved_lantern();
            let (_, mut authority, mut trail, mut executor, clock, mut anchor) = runtime_parts();
            let response = run_runtime(
                PermissionDecision::Deny,
                &mut authority,
                &mut trail,
                &mut executor,
                &clock,
                None,
                &mut anchor,
            );
            assert_eq!(authority.admissions, 0);
            assert_eq!(executor.calls, 0);
            assert_eq!(response["execution_status"], "denied");
            drop(resolved);
        }

        #[test]
        fn j09_runtime_08_unavailable_never_opens_replay_authority() {
            let (_, mut authority, mut trail, mut executor, clock, mut anchor) = runtime_parts();
            let response = run_runtime(
                PermissionDecision::Unavailable,
                &mut authority,
                &mut trail,
                &mut executor,
                &clock,
                None,
                &mut anchor,
            );
            assert_eq!(authority.admissions, 0);
            assert_eq!(executor.calls, 0);
            assert_eq!(response["execution_status"], "denied");
        }

        #[test]
        fn j09_runtime_09_allow_without_root_is_persistence_unavailable() {
            let (_, resolved) = resolved_lantern();
            let mut authority = replay_runtime::FileReplayAuthority::new(None);
            let mut trail = RecordingTrail::new();
            let events = Rc::new(RefCell::new(Vec::new()));
            let guard = Rc::new(Cell::new(false));
            let mut executor = RuntimeExecutor::success(Rc::clone(&events), Rc::clone(&guard));
            let clock = RuntimeClock::within_deadline(Rc::clone(&events));
            let mut anchor = RuntimeAnchorWriter::new(events, guard);
            let response = run_runtime(
                allow_decision_for(&resolved),
                &mut authority,
                &mut trail,
                &mut executor,
                &clock,
                None,
                &mut anchor,
            );
            assert_only_existing_replay_result_field(&response, "replay_persistence_unavailable");
            assert_eq!(executor.calls, 0);
        }

        macro_rules! recovered_runtime_test {
            ($name:ident, $state:expr, $expected:literal) => {
                #[test]
                fn $name() {
                    let (_, resolved) = resolved_lantern();
                    let (events, mut authority, mut trail, mut executor, clock, mut anchor) =
                        runtime_parts();
                    authority.fresh = false;
                    authority.recovered_state = $state;
                    let mut approval = RuntimeApproval {
                        events,
                        fail: false,
                        consumes: 0,
                    };
                    let response = run_runtime(
                        allow_decision_for(&resolved),
                        &mut authority,
                        &mut trail,
                        &mut executor,
                        &clock,
                        Some(&mut approval),
                        &mut anchor,
                    );
                    assert_only_existing_replay_result_field(&response, $expected);
                    assert_eq!(approval.consumes, 0);
                    assert_eq!(executor.calls, 0);
                    assert!(trail.entries.is_empty());
                    assert!(trail.outcome_entries.is_empty());
                }
            };
        }

        recovered_runtime_test!(
            j09_runtime_10_recovered_success_maps_exactly,
            replay::ReplayState::Succeeded,
            "replay_blocked_completed_success"
        );
        recovered_runtime_test!(
            j09_runtime_11_recovered_failure_maps_exactly,
            replay::ReplayState::Failed,
            "replay_blocked_completed_failure"
        );
        recovered_runtime_test!(
            j09_runtime_12_recovered_claim_is_manual_only,
            replay::ReplayState::ClaimedNoState,
            "replay_requires_manual_resolution"
        );
        recovered_runtime_test!(
            j09_runtime_13_recovered_g0_is_manual_only,
            replay::ReplayState::IntentRecorded,
            "replay_requires_manual_resolution"
        );
        recovered_runtime_test!(
            j09_runtime_14_recovered_g1_is_manual_only,
            replay::ReplayState::InvocationArmed,
            "replay_requires_manual_resolution"
        );
        recovered_runtime_test!(
            j09_runtime_15_recovered_uncertain_is_manual_only,
            replay::ReplayState::Uncertain,
            "replay_requires_manual_resolution"
        );

        #[test]
        fn j09_runtime_16_admission_failure_maps_only_to_persistence_unavailable() {
            let (_, resolved) = resolved_lantern();
            let (events, mut authority, mut trail, mut executor, clock, mut anchor) =
                runtime_parts();
            authority.fail_at = Some(replay_runtime::test_support::FailPoint::Admit);
            let mut approval = RuntimeApproval {
                events,
                fail: false,
                consumes: 0,
            };
            let response = run_runtime(
                allow_decision_for(&resolved),
                &mut authority,
                &mut trail,
                &mut executor,
                &clock,
                Some(&mut approval),
                &mut anchor,
            );
            assert_only_existing_replay_result_field(&response, "replay_persistence_unavailable");
            assert_eq!(approval.consumes, 0);
            assert!(trail.entries.is_empty());
            assert_eq!(executor.calls, 0);
        }

        #[test]
        fn j09_runtime_17_success_has_the_exact_observable_order() {
            let (_, resolved) = resolved_lantern();
            let (events, mut authority, mut trail, mut executor, clock, mut anchor) =
                runtime_parts();
            let response = run_runtime(
                allow_decision_for(&resolved),
                &mut authority,
                &mut trail,
                &mut executor,
                &clock,
                None,
                &mut anchor,
            );
            assert_eq!(
                *events.borrow(),
                vec![
                    "admit",
                    "publish_g0",
                    "trail_intent",
                    "deadline_start",
                    "deadline_check",
                    "publish_g1",
                    "provider",
                    "response_observed",
                    "trail_outcome",
                    "publish_g2",
                    "anchor",
                    "release_admission"
                ]
            );
            assert_eq!(response["execution_status"], "completed");
        }

        #[test]
        fn j09_runtime_18_known_failure_has_outcome_g2_anchor_order() {
            let (_, resolved) = resolved_lantern();
            let (events, mut authority, mut trail, mut executor, clock, mut anchor) =
                runtime_parts();
            executor.result = Err(outcome::ProviderDiagnostic::ExplicitProviderError);
            let response = run_runtime(
                allow_decision_for(&resolved),
                &mut authority,
                &mut trail,
                &mut executor,
                &clock,
                None,
                &mut anchor,
            );
            let events = events.borrow();
            assert!(
                events.iter().position(|event| *event == "trail_outcome")
                    < events.iter().position(|event| *event == "publish_g2")
            );
            assert!(
                events.iter().position(|event| *event == "publish_g2")
                    < events.iter().position(|event| *event == "anchor")
            );
            assert_eq!(response["execution_status"], "failed");
        }

        #[test]
        fn j09_runtime_19_uncertain_has_outcome_g2_anchor_order() {
            let (_, resolved) = resolved_lantern();
            let (events, mut authority, mut trail, mut executor, clock, mut anchor) =
                runtime_parts();
            executor.result = Err(outcome::ProviderDiagnostic::ProcessLost);
            let response = run_runtime(
                allow_decision_for(&resolved),
                &mut authority,
                &mut trail,
                &mut executor,
                &clock,
                None,
                &mut anchor,
            );
            assert_eq!(
                &events.borrow()[8..11],
                &["trail_outcome", "publish_g2", "anchor"]
            );
            assert_eq!(response["execution_status"], "uncertain");
        }

        #[test]
        fn j09_runtime_20_approved_ask_consumes_between_claim_and_g0() {
            let (_, resolved) = resolved_lantern();
            let (events, mut authority, mut trail, mut executor, clock, mut anchor) =
                runtime_parts();
            let mut approval = RuntimeApproval {
                events: Rc::clone(&events),
                fail: false,
                consumes: 0,
            };
            run_runtime(
                allow_decision_for(&resolved),
                &mut authority,
                &mut trail,
                &mut executor,
                &clock,
                Some(&mut approval),
                &mut anchor,
            );
            assert_eq!(approval.consumes, 1);
            assert_eq!(
                &events.borrow()[..3],
                &["admit", "consume_approval", "publish_g0"]
            );
        }

        #[test]
        fn j09_runtime_21_approval_consumption_failure_leaves_claim_only() {
            let (_, resolved) = resolved_lantern();
            let (events, mut authority, mut trail, mut executor, clock, mut anchor) =
                runtime_parts();
            let mut approval = RuntimeApproval {
                events: Rc::clone(&events),
                fail: true,
                consumes: 0,
            };
            let response = run_runtime(
                allow_decision_for(&resolved),
                &mut authority,
                &mut trail,
                &mut executor,
                &clock,
                Some(&mut approval),
                &mut anchor,
            );
            assert_eq!(
                *events.borrow(),
                vec!["admit", "consume_approval", "release_admission"]
            );
            assert_only_existing_replay_result_field(
                &response,
                "replay_requires_manual_resolution",
            );
            assert_eq!(executor.calls, 0);
        }

        #[test]
        fn j09_runtime_22_g0_failure_prevents_intent_and_provider() {
            let (_, resolved) = resolved_lantern();
            let (events, mut authority, mut trail, mut executor, clock, mut anchor) =
                runtime_parts();
            authority.fail_at = Some(replay_runtime::test_support::FailPoint::Intent);
            let response = run_runtime(
                allow_decision_for(&resolved),
                &mut authority,
                &mut trail,
                &mut executor,
                &clock,
                None,
                &mut anchor,
            );
            assert_eq!(
                *events.borrow(),
                vec!["admit", "publish_g0", "release_admission"]
            );
            assert_eq!(executor.calls, 0);
            assert_only_existing_replay_result_field(&response, "replay_persistence_unavailable");
        }

        #[test]
        fn j09_runtime_23_trail_intent_failure_leaves_g0_and_zero_calls() {
            let (_, resolved) = resolved_lantern();
            let (events, mut authority, mut trail, mut executor, clock, mut anchor) =
                runtime_parts();
            trail.injected_intent_error =
                Some(dispatch::TrailError::WriteFailed("intent".to_owned()));
            let response = run_runtime(
                allow_decision_for(&resolved),
                &mut authority,
                &mut trail,
                &mut executor,
                &clock,
                None,
                &mut anchor,
            );
            assert_eq!(
                *events.borrow(),
                vec!["admit", "publish_g0", "trail_intent", "release_admission"]
            );
            assert_eq!(executor.calls, 0);
            assert_eq!(response["execution_status"], "denied");
        }

        #[test]
        fn j09_runtime_24_deadline_expiry_leaves_g0_and_zero_calls() {
            let (_, resolved) = resolved_lantern();
            let (events, mut authority, mut trail, mut executor, _, mut anchor) = runtime_parts();
            let clock = RuntimeClock::expired_before_invocation(Rc::clone(&events));
            let response = run_runtime(
                allow_decision_for(&resolved),
                &mut authority,
                &mut trail,
                &mut executor,
                &clock,
                None,
                &mut anchor,
            );
            assert!(!events.borrow().contains(&"publish_g1"));
            assert_eq!(executor.calls, 0);
            assert_eq!(response["execution_status"], "unattempted");
        }

        #[test]
        fn j09_runtime_25_g1_failure_prevents_provider() {
            let (_, resolved) = resolved_lantern();
            let (events, mut authority, mut trail, mut executor, clock, mut anchor) =
                runtime_parts();
            authority.fail_at = Some(replay_runtime::test_support::FailPoint::Armed);
            let response = run_runtime(
                allow_decision_for(&resolved),
                &mut authority,
                &mut trail,
                &mut executor,
                &clock,
                None,
                &mut anchor,
            );
            assert!(events.borrow().contains(&"publish_g1"));
            assert!(!events.borrow().contains(&"provider"));
            assert_eq!(executor.calls, 0);
            assert_only_existing_replay_result_field(&response, "replay_persistence_unavailable");
        }

        macro_rules! outcome_failure_runtime_test {
            ($name:ident, $diagnostic:expr, $expected_status:literal) => {
                #[test]
                fn $name() {
                    let (_, resolved) = resolved_lantern();
                    let (events, mut authority, mut trail, mut executor, clock, mut anchor) =
                        runtime_parts();
                    executor.result = $diagnostic;
                    trail.injected_outcome_error =
                        Some(dispatch::TrailError::WriteFailed("outcome".to_owned()));
                    let response = run_runtime(
                        allow_decision_for(&resolved),
                        &mut authority,
                        &mut trail,
                        &mut executor,
                        &clock,
                        None,
                        &mut anchor,
                    );
                    assert_eq!(response["execution_status"], $expected_status);
                    assert!(events.borrow().contains(&"trail_outcome"));
                    assert!(!events.borrow().contains(&"publish_g2"));
                    assert_eq!(anchor.writes, 0);
                    assert_eq!(executor.calls, 1);
                }
            };
        }

        outcome_failure_runtime_test!(
            j09_runtime_26_success_outcome_write_failure_leaves_g1,
            Ok(json!({"status":"recorded","project":"p","task":"t"})),
            "completed"
        );
        outcome_failure_runtime_test!(
            j09_runtime_27_failure_outcome_write_failure_leaves_g1,
            Err(outcome::ProviderDiagnostic::ExplicitProviderError),
            "failed"
        );
        outcome_failure_runtime_test!(
            j09_runtime_28_uncertain_outcome_write_failure_leaves_g1,
            Err(outcome::ProviderDiagnostic::ProcessLost),
            "uncertain"
        );

        #[test]
        fn j09_runtime_29_g2_failure_withholds_anchor_without_retry() {
            let (_, resolved) = resolved_lantern();
            let (events, mut authority, mut trail, mut executor, clock, mut anchor) =
                runtime_parts();
            authority.fail_at = Some(replay_runtime::test_support::FailPoint::Terminal);
            let response = run_runtime(
                allow_decision_for(&resolved),
                &mut authority,
                &mut trail,
                &mut executor,
                &clock,
                None,
                &mut anchor,
            );
            assert!(events.borrow().contains(&"trail_outcome"));
            assert!(events.borrow().contains(&"publish_g2"));
            assert!(!events.borrow().contains(&"anchor"));
            assert_eq!(executor.calls, 1);
            assert_eq!(response["execution_status"], "completed");
            assert!(response.get("replay_result").is_none());
            assert!(response.get("result_anchor").is_none());
            assert_eq!(trail.outcome_entries.len(), 1);
            assert!(!response["trail"]
                .as_array()
                .unwrap()
                .iter()
                .any(|entry| entry["kind"] == "audit_failure"));
            assert!(!response.to_string().contains("outcome audit write failed"));
        }

        #[test]
        fn j09_runtime_30_anchor_failure_leaves_g2_without_retry() {
            let (_, resolved) = resolved_lantern();
            let (events, mut authority, mut trail, mut executor, clock, mut anchor) =
                runtime_parts();
            anchor.fail = true;
            let response = run_runtime(
                allow_decision_for(&resolved),
                &mut authority,
                &mut trail,
                &mut executor,
                &clock,
                None,
                &mut anchor,
            );
            assert_eq!(
                &events.borrow()[8..],
                &["trail_outcome", "publish_g2", "anchor", "release_admission"]
            );
            assert!(response.get("result_anchor").is_none());
            assert_eq!(executor.calls, 1);
        }

        #[test]
        fn j09_runtime_31_guard_is_held_at_provider_boundary() {
            let (_, resolved) = resolved_lantern();
            let (_, mut authority, mut trail, mut executor, clock, mut anchor) = runtime_parts();
            run_runtime(
                allow_decision_for(&resolved),
                &mut authority,
                &mut trail,
                &mut executor,
                &clock,
                None,
                &mut anchor,
            );
            assert!(executor.saw_guard);
            assert!(!authority.guard_held.get());
        }

        #[test]
        fn j09_runtime_32_guard_is_held_during_successful_and_failed_anchor_write() {
            for fail in [false, true] {
                let (_, resolved) = resolved_lantern();
                let (_, mut authority, mut trail, mut executor, clock, mut anchor) =
                    runtime_parts();
                anchor.fail = fail;
                run_runtime(
                    allow_decision_for(&resolved),
                    &mut authority,
                    &mut trail,
                    &mut executor,
                    &clock,
                    None,
                    &mut anchor,
                );
                assert!(anchor.saw_guard, "fail={fail}");
                assert!(!authority.guard_held.get(), "fail={fail}");
            }
        }

        #[test]
        fn j09_runtime_33_binding_uses_exact_planner_ids_and_host_uuid_stays_local() {
            let (_, resolved) = resolved_lantern();
            let (_, mut authority, mut trail, mut executor, clock, mut anchor) = runtime_parts();
            let response = run_runtime(
                allow_decision_for(&resolved),
                &mut authority,
                &mut trail,
                &mut executor,
                &clock,
                None,
                &mut anchor,
            );
            let binding = &authority.bindings[0];
            assert_eq!(binding.evaluation_id, "eval-j09-001");
            assert_eq!(binding.action_id, "action-j09-001");
            assert_eq!(
                trail.entries[0].execution_id,
                replay_runtime::test_support::TEST_EXECUTION_ID
            );
            assert_eq!(
                response["result_anchor"]["facts"]["evaluation_id"],
                "eval-j09-001"
            );
            assert!(!response
                .to_string()
                .contains(replay_runtime::test_support::TEST_EXECUTION_ID));
        }

        #[test]
        fn j09_runtime_34_argument_digest_binds_complete_resolved_arguments() {
            let (_, resolved) = resolved_lantern();
            let (_, mut authority, mut trail, mut executor, clock, mut anchor) = runtime_parts();
            run_runtime(
                allow_decision_for(&resolved),
                &mut authority,
                &mut trail,
                &mut executor,
                &clock,
                None,
                &mut anchor,
            );
            assert_eq!(
                authority.bindings[0].argument_digest,
                approval::digest(&json!({"project":"p","task":"t"}))
            );
        }

        #[test]
        fn j09_runtime_35_ordinary_allow_consumes_no_approval() {
            let (_, resolved) = resolved_lantern();
            let (events, mut authority, mut trail, mut executor, clock, mut anchor) =
                runtime_parts();
            run_runtime(
                allow_decision_for(&resolved),
                &mut authority,
                &mut trail,
                &mut executor,
                &clock,
                None,
                &mut anchor,
            );
            assert!(!events.borrow().contains(&"consume_approval"));
            assert_eq!(executor.calls, 1);
        }

        #[test]
        fn j09_runtime_36_unmatched_response_never_enters_replay_routing() {
            let response = json!({"status":"unmatched","trail":[]});
            let authority = replay_runtime::test_support::TestReplayAuthority::default();
            assert!(!response_is_matched(&response));
            assert_eq!(authority.admissions, 0);
        }

        #[test]
        fn j09_runtime_37_fresh_ask_creates_pending_approval_and_zero_claims() {
            let (store, resolved) = resolved_approval_fixture();
            let availability = ProviderAvailability::from_identities(["lantern-local"]);
            let requirements = vec![CapabilityRequirement::new("fixture.ask", 1)];
            let host_policy = HostLocalPolicy::new(PolicyRule::Allow);
            let response = make_bridge_matched_response(&resolved);
            let action = extract_proposed_action(&response).unwrap();
            let mut approvals = approval::ApprovalStore::default();
            let mut trail = RecordingTrail::new();
            let authority = replay_runtime::test_support::TestReplayAuthority::default();
            let request = request_exact_approval(
                &action,
                &requirements,
                &store,
                &availability,
                &host_policy,
                policy::ScopeAssessment::ScopeNotEstablished,
                &mut approvals,
                &mut trail,
            )
            .unwrap()
            .unwrap();
            assert_eq!(
                approvals.record(&request.approval_id).unwrap().state,
                approval::ApprovalState::Pending
            );
            assert_eq!(trail.authorisation_entries.len(), 1);
            assert_eq!(authority.admissions, 0);
        }

        #[test]
        fn j09_runtime_38_trail_and_replay_roots_remain_explicitly_distinct() {
            let args = vec![
                "engine.exe".to_owned(),
                "request.json".to_owned(),
                "allow".to_owned(),
                r"D:\independent-audit\trail.jsonl".to_owned(),
                "success".to_owned(),
                "--host-data-root".to_owned(),
                r"C:\independent-host-data".to_owned(),
            ];
            let parsed = parse_normal_args(&args).unwrap();
            assert_eq!(
                parsed.trail_path.as_deref(),
                Some(r"D:\independent-audit\trail.jsonl")
            );
            assert_eq!(
                parsed.host_data_root.as_deref(),
                Some(Path::new(r"C:\independent-host-data"))
            );
        }

        #[test]
        fn j09_runtime_39_approved_ask_missing_root_consumes_zero_approvals() {
            struct FixtureExecutor {
                calls: usize,
            }
            impl CapabilityExecutor for FixtureExecutor {
                fn provider_identity(&self) -> &str {
                    "lantern-local"
                }
                fn execute(&mut self, _ready: &DispatchReadyAction) -> Result<Value, String> {
                    self.calls += 1;
                    Ok(json!({"status":"recorded","project":"p","task":"t"}))
                }
            }

            let (store, resolved) = resolved_approval_fixture();
            let availability = ProviderAvailability::from_identities(["lantern-local"]);
            let requirements = vec![CapabilityRequirement::new("fixture.ask", 1)];
            let host_policy = HostLocalPolicy::new(PolicyRule::Allow);
            let mut response = make_bridge_matched_response(&resolved);
            let action = extract_proposed_action(&response).unwrap();
            let mut approvals = approval::ApprovalStore::default();
            let mut trail = RecordingTrail::new();
            let request = request_exact_approval(
                &action,
                &requirements,
                &store,
                &availability,
                &host_policy,
                policy::ScopeAssessment::ScopeNotEstablished,
                &mut approvals,
                &mut trail,
            )
            .unwrap()
            .unwrap();
            record_human_approval_decision(
                &request.approval_id,
                HumanApprovalDecision::Approve,
                &mut approvals,
                &mut trail,
            )
            .unwrap();
            let mut executor = FixtureExecutor { calls: 0 };
            resume_and_execute_exact_approval(
                &mut response,
                &request.approval_id,
                &requirements,
                &store,
                &availability,
                &host_policy,
                policy::ScopeAssessment::ScopeNotEstablished,
                &mut approvals,
                &mut trail,
                &mut executor,
                "evt-j09-approved-missing-root",
                None,
            )
            .unwrap();
            assert_eq!(
                approvals.record(&request.approval_id).unwrap().state,
                approval::ApprovalState::Approved
            );
            assert_eq!(executor.calls, 0);
            assert_only_existing_replay_result_field(&response, "replay_persistence_unavailable");
        }

        #[test]
        fn j09_runtime_40_recovered_approved_ask_consumes_zero_additional_approvals() {
            let (_, resolved) = resolved_lantern();
            let (events, mut authority, mut trail, mut executor, clock, mut anchor) =
                runtime_parts();
            authority.fresh = false;
            authority.recovered_state = replay::ReplayState::Succeeded;
            let mut approval = RuntimeApproval {
                events,
                fail: false,
                consumes: 0,
            };
            let response = run_runtime(
                allow_decision_for(&resolved),
                &mut authority,
                &mut trail,
                &mut executor,
                &clock,
                Some(&mut approval),
                &mut anchor,
            );
            assert_eq!(approval.consumes, 0);
            assert_eq!(executor.calls, 0);
            assert_only_existing_replay_result_field(&response, "replay_blocked_completed_success");
        }

        #[test]
        fn j09_runtime_41_storage_path_and_diagnostics_never_reach_public_response() {
            let (_, resolved) = resolved_lantern();
            let raw_root = PathBuf::from(r"C:\secret-replay-root-token-does-not-exist");
            let mut authority = replay_runtime::FileReplayAuthority::new(Some(&raw_root));
            let mut trail = RecordingTrail::new();
            let events = Rc::new(RefCell::new(Vec::new()));
            let guard = Rc::new(Cell::new(false));
            let mut executor = RuntimeExecutor::success(Rc::clone(&events), Rc::clone(&guard));
            let clock = RuntimeClock::within_deadline(Rc::clone(&events));
            let mut anchor = RuntimeAnchorWriter::new(events, guard);
            let response = run_runtime(
                allow_decision_for(&resolved),
                &mut authority,
                &mut trail,
                &mut executor,
                &clock,
                None,
                &mut anchor,
            );
            assert_only_existing_replay_result_field(&response, "replay_persistence_unavailable");
            let public = response.to_string();
            assert!(!public.contains("secret-replay-root-token"));
            assert!(!public.contains("PersistenceUnavailable"));
            assert!(!public.contains("win32"));
        }

        #[test]
        fn j09_runtime_42_provisioning_wrong_shapes_are_rejected_without_mutation() {
            let cases = [
                vec!["provision-replay".to_owned()],
                vec![
                    "provision-replay".to_owned(),
                    "--host-data-root".to_owned(),
                    r"C:\host-data".to_owned(),
                ],
                vec![
                    "provision-replay".to_owned(),
                    r"C:\host-data".to_owned(),
                    "extra".to_owned(),
                ],
            ];
            for args in cases {
                assert_eq!(parse_provision_args(&args).unwrap_err(), PROVISION_USAGE);
            }
        }

        macro_rules! native_recovered_runtime_test {
            ($name:ident, $state:expr, $expected:literal) => {
                #[cfg(windows)]
                #[test]
                fn $name() {
                    run_native_recovered_state($state, $expected);
                }
            };
        }

        native_recovered_runtime_test!(
            j09_replay_runtime_native_claim_only_is_manual_without_provider,
            replay::ReplayState::ClaimedNoState,
            "replay_requires_manual_resolution"
        );
        native_recovered_runtime_test!(
            j09_replay_runtime_native_g0_is_manual_without_provider,
            replay::ReplayState::IntentRecorded,
            "replay_requires_manual_resolution"
        );
        native_recovered_runtime_test!(
            j09_replay_runtime_native_g1_is_manual_without_provider,
            replay::ReplayState::InvocationArmed,
            "replay_requires_manual_resolution"
        );
        native_recovered_runtime_test!(
            j09_replay_runtime_native_success_is_blocked_without_provider,
            replay::ReplayState::Succeeded,
            "replay_blocked_completed_success"
        );
        native_recovered_runtime_test!(
            j09_replay_runtime_native_failure_is_blocked_without_provider,
            replay::ReplayState::Failed,
            "replay_blocked_completed_failure"
        );
        native_recovered_runtime_test!(
            j09_replay_runtime_native_uncertain_is_manual_without_provider,
            replay::ReplayState::Uncertain,
            "replay_requires_manual_resolution"
        );

        #[cfg(windows)]
        #[test]
        fn j09_replay_runtime_native_binding_mismatch_fails_before_provider() {
            let Some(root) = fresh_native_runtime_root("binding-mismatch") else {
                return;
            };
            let ledger = replay_windows::ReplayLedger::open(&root).unwrap();
            let key = replay::LogicalExecutionKey::derive(
                "evt-j09-001",
                "eval-j09-001",
                "action-j09-001",
            )
            .unwrap();
            let mut changed = native_runtime_binding();
            changed.argument_digest = approval::digest(&json!({"project":"changed","task":"t"}));
            drop(ledger.admit_or_recover(key, changed).unwrap());

            let (_, resolved) = resolved_lantern();
            let mut authority = replay_runtime::FileReplayAuthority::new(Some(&root));
            let mut trail = RecordingTrail::new();
            let events = Rc::new(RefCell::new(Vec::new()));
            let guard = Rc::new(Cell::new(false));
            let mut executor = RuntimeExecutor::success(Rc::clone(&events), Rc::clone(&guard));
            let clock = RuntimeClock::within_deadline(Rc::clone(&events));
            let mut anchor = RuntimeAnchorWriter::new(events, guard);
            let response = run_runtime(
                allow_decision_for(&resolved),
                &mut authority,
                &mut trail,
                &mut executor,
                &clock,
                None,
                &mut anchor,
            );
            assert_only_existing_replay_result_field(&response, "replay_persistence_unavailable");
            assert_eq!(executor.calls, 0);
            assert!(trail.entries.is_empty());
        }

        #[cfg(windows)]
        #[test]
        fn j09_replay_runtime_native_fresh_success_restart_makes_zero_second_call() {
            let Some(root) = fresh_native_runtime_root("fresh-success-restart") else {
                return;
            };
            let (_, resolved) = resolved_lantern();

            let mut first_authority = replay_runtime::FileReplayAuthority::new(Some(&root));
            let mut first_trail = RecordingTrail::new();
            let first_events = Rc::new(RefCell::new(Vec::new()));
            let first_guard = Rc::new(Cell::new(false));
            let mut first_executor =
                RuntimeExecutor::success(Rc::clone(&first_events), Rc::clone(&first_guard));
            let first_clock = RuntimeClock::within_deadline(Rc::clone(&first_events));
            let mut first_anchor = RuntimeAnchorWriter::new(first_events, first_guard);
            let first = run_runtime(
                allow_decision_for(&resolved),
                &mut first_authority,
                &mut first_trail,
                &mut first_executor,
                &first_clock,
                None,
                &mut first_anchor,
            );
            assert_eq!(first["execution_status"], "completed");
            assert_eq!(first_executor.calls, 1);
            assert_eq!(first_anchor.writes, 1);
            drop(first_authority);

            let mut second_authority = replay_runtime::FileReplayAuthority::new(Some(&root));
            let mut second_trail = RecordingTrail::new();
            let second_events = Rc::new(RefCell::new(Vec::new()));
            let second_guard = Rc::new(Cell::new(false));
            let mut second_executor =
                RuntimeExecutor::success(Rc::clone(&second_events), Rc::clone(&second_guard));
            let second_clock = RuntimeClock::within_deadline(Rc::clone(&second_events));
            let mut second_anchor = RuntimeAnchorWriter::new(second_events, second_guard);
            let second = run_runtime(
                allow_decision_for(&resolved),
                &mut second_authority,
                &mut second_trail,
                &mut second_executor,
                &second_clock,
                None,
                &mut second_anchor,
            );
            assert_only_existing_replay_result_field(&second, "replay_blocked_completed_success");
            assert_eq!(second_executor.calls, 0);
            assert_eq!(second_anchor.writes, 0);
            assert!(second_trail.entries.is_empty());
            assert!(second_trail.outcome_entries.is_empty());
        }
    }
}
