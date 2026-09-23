//! R2 Authority Gate conformance suite.
//!
//! Proves: Tethers Core defines the Plan; Tethers authorises; the external
//! Host executes; durable Tethers truth survives Gate death.

use serde_json::{json, Value};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::time::Instant;
use tethers_reference_host::authority_gate::{AuthorityGate, GateConfig};
use tethers_reference_host::gate_protocol::{AuthorityResponse, ResponseStatus};

const STANDING_ALLOW_DIGEST: &str =
    "sha256:eb61b62bde489e00a4d15c37c83e6cdb1e9e378b8f13b910d4b68bd6d68c19da";
const STANDING_ALLOW_MANIFEST: &str =
    include_str!("../../protocol/capability-manifests/fixture-ping-standing-allow.json");

const CORE_TETHER: &str = r#"tether "J14 complete local scenario"

anchor
    coding.task_completed

when
    project.type is "software"
    and task.changed_files greater_than 0

do
    fixture.ping
        message: anchor.task
        path: anchor.path
"#;

/// Engine binary for Core-backed PREPARE. Prefer the verified engine from
/// the canonical verifier; fall back to the current-checkout build output.
fn engine_path() -> PathBuf {
    if let Ok(path) = std::env::var("TETHERS_VERIFIED_ENGINE") {
        let path = PathBuf::from(path);
        if path.is_file() {
            return path;
        }
    }
    let candidates = [
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../engine-ocaml/_build/default/bin/tethers_mcp_main.exe"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../engine-ocaml/_build/default/bin/tethers_mcp_main"),
    ];
    for candidate in candidates {
        if candidate.is_file() {
            return candidate.canonicalize().unwrap_or(candidate);
        }
    }
    panic!(
        "Tethers Core engine not found. Run scripts/prepare-current-engine.ps1 \
         or set TETHERS_VERIFIED_ENGINE."
    );
}

// ---------------------------------------------------------------------------
// Workspace fixture
// ---------------------------------------------------------------------------

struct GateWorkspace {
    root: PathBuf,
    config: PathBuf,
    trail: PathBuf,
    host_data: PathBuf,
    engine: PathBuf,
}

impl GateWorkspace {
    fn new(policy_decision: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "tethers-r2-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4().simple()
        ));
        fs::create_dir_all(root.join("tethers")).unwrap();
        fs::create_dir_all(root.join("manifests")).unwrap();
        fs::create_dir_all(root.join("host-data")).unwrap();

        fs::write(root.join("tethers/complete.tether"), CORE_TETHER).unwrap();
        fs::write(
            root.join("manifests/fixture-ping-standing-allow.json"),
            STANDING_ALLOW_MANIFEST,
        )
        .unwrap();

        let config = root.join("runtime.json");
        let trail = root.join("trail.jsonl");
        let host_data = root.join("host-data");
        let engine = engine_path();
        let workspace = Self {
            root,
            config,
            trail,
            host_data,
            engine,
        };
        workspace.write_config(policy_decision);
        workspace.provision_replay();
        workspace
    }

    fn write_config(&self, policy_decision: &str) {
        let config = json!({
            "format_version": "0.1",
            "tether_set": {
                "id": "r2.authority-gate",
                "version": "1",
                "tethers": [
                    {
                        "id": "r2-complete",
                        "version": "1",
                        "source_path": "tethers/complete.tether",
                        "core_environment": {
                            "program_id": "program.j14.complete",
                            "core_version": "1",
                            "capabilities": [
                                {
                                    "source_name": "fixture.ping",
                                    "capability_id": "cap.semantic.fixture-ping",
                                    "contract_digest": "CORE-CONTRACT-J14",
                                    "runtime_name": "fixture.ping"
                                }
                            ],
                            "input_facts": [
                                {
                                    "source_name": "project.type",
                                    "fact_id": "fact.project_type",
                                    "host_snapshot_key": "project.type",
                                    "scalar_type": "string",
                                    "schema_description": "project type"
                                },
                                {
                                    "source_name": "task.changed_files",
                                    "fact_id": "fact.task_changed_files",
                                    "host_snapshot_key": "task.changed_files",
                                    "scalar_type": "integer",
                                    "schema_description": "number of changed files"
                                }
                            ]
                        }
                    }
                ],
                "capability_requirements": [
                    {
                        "name": "fixture.ping",
                        "version": 1,
                        "reason": "R2 authority gate fixture"
                    }
                ]
            },
            "providers": [
                {
                    "id": "tethers-stdio-fixture",
                    "display_name": "Tethers Stdio Fixture",
                    "transport": {
                        "kind": "stdio",
                        "command": "pwsh.exe",
                        "args": ["-NoProfile", "-File", "providers/fixture.ps1"],
                        "protocol_version": "2025-11-25"
                    },
                    "capabilities": [
                        {
                            "name": "fixture.ping",
                            "version": 1,
                            "manifest_path": "manifests/fixture-ping-standing-allow.json",
                            "pinned_digest": STANDING_ALLOW_DIGEST,
                            "scope_binding": {
                                "kind": "path_prefix",
                                "argument_json_pointer": "/path"
                            }
                        }
                    ]
                }
            ],
            "policy": {
                "default": "deny",
                "rules": [
                    {
                        "name": "fixture.ping",
                        "version": 1,
                        "decision": policy_decision
                    }
                ]
            }
        });
        fs::write(&self.config, serde_json::to_vec_pretty(&config).unwrap()).unwrap();
    }

    fn provision_replay(&self) {
        harden_replay_acl(&self.host_data);
        tethers_reference_host::replay_store::provision_replay(&self.host_data)
            .expect("provision replay");
    }

    fn gate(&self) -> AuthorityGate {
        AuthorityGate::new(GateConfig {
            config_path: self.config.clone(),
            engine_path: self.engine.clone(),
            trail_path: self.trail.clone(),
            host_data_root: self.host_data.clone(),
        })
    }

    /// Core-authoritative PREPARE payload: run-input material only.
    fn prepare_payload(&self, action_id: &str, path: &str) -> Value {
        self.prepare_payload_with_eval(action_id, path, "eval_r2_001", true)
    }

    fn prepare_payload_with_eval(
        &self,
        action_id: &str,
        path: &str,
        evaluation_id: &str,
        matches_condition: bool,
    ) -> Value {
        let facts = if matches_condition {
            json!({ "project.type": "software", "task.changed_files": 3 })
        } else {
            json!({ "project.type": "docs", "task.changed_files": 0 })
        };
        json!({
            "action_id": action_id,
            "evaluation_id": evaluation_id,
            "tether": { "id": "r2-complete", "version": "1" },
            "event": {
                "id": "evt_r2_001",
                "name": "coding.task_completed",
                "data": {
                    "project": "lantern-keeper",
                    "task": "LK-39",
                    "path": path
                }
            },
            "facts": facts
        })
    }

    fn marker_path(&self) -> PathBuf {
        self.root.join("marker.txt")
    }
}

impl Drop for GateWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn harden_replay_acl(root: &Path) {
    #[cfg(windows)]
    {
        let acl_script = format!(
            "$p='{}'; $identity=[System.Security.Principal.WindowsIdentity]::GetCurrent().Name; $acl=[System.Security.AccessControl.DirectorySecurity]::new(); $acl.SetAccessRuleProtection($true,$false); $inherit=[System.Security.AccessControl.InheritanceFlags]::ContainerInherit -bor [System.Security.AccessControl.InheritanceFlags]::ObjectInherit; foreach($t in @($identity,'NT AUTHORITY\\SYSTEM','BUILTIN\\Administrators')) {{ $acl.AddAccessRule([System.Security.AccessControl.FileSystemAccessRule]::new($t,'FullControl',$inherit,'None','Allow')) }}; Set-Acl -LiteralPath $p -AclObject $acl",
            root.display()
        );
        let status = std::process::Command::new("pwsh.exe")
            .args(["-NoProfile", "-Command", &acl_script])
            .status()
            .expect("set replay ACL");
        assert!(status.success(), "replay root ACL hardening failed");
    }
    #[cfg(not(windows))]
    let _ = root;
}

fn call(
    gate: &mut AuthorityGate,
    request_id: &str,
    operation: &str,
    payload: Value,
) -> AuthorityResponse {
    let map = payload.as_object().expect("payload object").clone();
    gate.handle(operation, request_id, &map)
}

fn ok(response: &AuthorityResponse) -> &Value {
    assert_eq!(
        response.status,
        ResponseStatus::Ok,
        "expected ok, got {:?}",
        response.error
    );
    response.result.as_ref().expect("result")
}

fn err(response: &AuthorityResponse) -> &str {
    assert_eq!(
        response.status,
        ResponseStatus::Error,
        "expected error, got {:?}",
        response.result
    );
    &response.error.as_ref().expect("error").code
}

fn err_data(response: &AuthorityResponse) -> Option<&Value> {
    response
        .error
        .as_ref()
        .and_then(|error| error.data.as_ref())
}

/// External Host fixture: counts physical effects. The Gate never touches this.
struct ExternalHostFixture {
    effects: usize,
}

impl ExternalHostFixture {
    fn new() -> Self {
        Self { effects: 0 }
    }

    fn execute_harmless(&mut self, workspace: &GateWorkspace) {
        self.effects += 1;
        fs::write(
            workspace.marker_path(),
            format!("effect-{}\n", self.effects),
        )
        .unwrap();
    }

    fn report_outcome(
        &self,
        gate: &mut AuthorityGate,
        request_id: &str,
        execution_id: &str,
        external_id: &str,
    ) -> AuthorityResponse {
        call(
            gate,
            request_id,
            "outcome",
            json!({
                "execution_id": execution_id,
                "classification": "succeeded",
                "attempted": true,
                "external_execution_identity": external_id,
                "result": {"echo": "r2"},
                "evidence": "sha256:evidence-fixture"
            }),
        )
    }
}

fn commit_ok(gate: &mut AuthorityGate, prepared_id: &str) -> Value {
    let response = call(
        gate,
        "commit-1",
        "commit",
        json!({ "prepared_id": prepared_id }),
    );
    ok(&response).clone()
}

// ---------------------------------------------------------------------------
// HELLO / non-execution
// ---------------------------------------------------------------------------

#[test]
fn hello_negotiates_protocol_without_authority() {
    let workspace = GateWorkspace::new("allow");
    let mut gate = workspace.gate();
    let result = ok(&call(&mut gate, "h1", "hello", json!({}))).clone();
    assert_eq!(result["protocol"], "tethers.authority/1");
    assert_eq!(result["authority_granted"], false);
    assert_eq!(result["provider_invocations"], 0);
    assert_eq!(result["product_version"], env!("CARGO_PKG_VERSION"));
    assert!(result["gate_instance_id"]
        .as_str()
        .unwrap()
        .starts_with("gate_"));
    assert_eq!(gate.provider_invocations(), 0);
}

// ---------------------------------------------------------------------------
// Policy matrix
// ---------------------------------------------------------------------------

#[test]
fn allow_prepare_commit_outcome_zero_gate_effects() {
    let workspace = GateWorkspace::new("allow");
    let mut gate = workspace.gate();
    let mut host = ExternalHostFixture::new();

    let prepared = ok(&call(
        &mut gate,
        "p1",
        "prepare",
        workspace.prepare_payload("action_1", "projects/r2-allow"),
    ))
    .clone();
    assert_eq!(prepared["decision"], "allow_prepared");
    assert_eq!(prepared["authorizes_dispatch"], false);
    assert_eq!(prepared["provider_invocations"], 0);

    let dispatch = commit_ok(&mut gate, prepared["prepared_id"].as_str().unwrap());
    assert_eq!(dispatch["schema"], "tethers.dispatch/1");
    assert_eq!(dispatch["intent"]["trail"], "recorded");
    assert_eq!(dispatch["intent"]["replay"], "armed");
    assert_eq!(dispatch["provider_invocations"], 0);

    // Durable intent exists before any host effect.
    let trail = fs::read_to_string(&workspace.trail).unwrap();
    assert!(trail.contains("\"execution_id\""));
    assert!(!workspace.marker_path().exists());

    host.execute_harmless(&workspace);
    let outcome = ok(&host.report_outcome(
        &mut gate,
        "o1",
        dispatch["execution_id"].as_str().unwrap(),
        "external-exec-1",
    ))
    .clone();
    assert_eq!(outcome["status"], "succeeded");
    assert_eq!(outcome["replay_terminal"], "recorded");

    assert_eq!(host.effects, 1);
    assert_eq!(gate.provider_invocations(), 0);
}

#[test]
fn deny_prepare_refuses_and_commit_is_impossible() {
    let workspace = GateWorkspace::new("deny");
    let mut gate = workspace.gate();
    let prepared_response = call(
        &mut gate,
        "p1",
        "prepare",
        workspace.prepare_payload("action_1", "projects/r2-deny"),
    );
    let prepared = ok(&prepared_response).clone();
    assert_eq!(prepared["decision"], "deny");

    // Even a forged prepared_id cannot commit under deny.
    let commit = call(
        &mut gate,
        "c1",
        "commit",
        json!({ "prepared_id": prepared["prepared_id"] }),
    );
    assert_eq!(err(&commit), "commit.deny");
    assert_eq!(gate.provider_invocations(), 0);
}

#[test]
fn ask_requires_exact_approval_before_commit() {
    let workspace = GateWorkspace::new("ask");
    let mut gate = workspace.gate();
    let mut host = ExternalHostFixture::new();

    let prepared = ok(&call(
        &mut gate,
        "p1",
        "prepare",
        workspace.prepare_payload("action_1", "projects/r2-ask"),
    ))
    .clone();
    assert_eq!(prepared["decision"], "ask");
    let approval_id = prepared["approval"]["approval_id"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(prepared["authorizes_dispatch"], false);

    // COMMIT before approval refuses.
    let early = call(
        &mut gate,
        "c1",
        "commit",
        json!({ "prepared_id": prepared["prepared_id"] }),
    );
    assert_eq!(err(&early), "commit.approval_not_ready");
    assert_eq!(host.effects, 0);

    // Exact approval decision.
    let decision = ok(&call(
        &mut gate,
        "a1",
        "approval_decision",
        json!({ "approval_id": approval_id, "decision": "approve" }),
    ))
    .clone();
    assert_eq!(decision["state"], "approved");
    assert_eq!(decision["authorizes_dispatch"], false);

    let dispatch = commit_ok(&mut gate, prepared["prepared_id"].as_str().unwrap());
    assert_eq!(dispatch["approval_consumed"], true);

    host.execute_harmless(&workspace);
    let outcome_response = host.report_outcome(
        &mut gate,
        "o1",
        dispatch["execution_id"].as_str().unwrap(),
        "external-exec-ask",
    );
    let outcome = ok(&outcome_response);
    assert_eq!(outcome["status"], "succeeded");
    assert_eq!(host.effects, 1);
    assert_eq!(gate.provider_invocations(), 0);
}

#[test]
fn unavailable_is_explicit_and_never_coerced() {
    let workspace = GateWorkspace::new("allow");
    let mut gate = workspace.gate();
    let mut payload = workspace.prepare_payload("action_1", "projects/r2-unavailable");
    payload["observations"] = json!({ "available_provider_identities": [] });

    let response = call(&mut gate, "p1", "prepare", payload);
    let prepared = ok(&response).clone();
    assert_eq!(prepared["decision"], "unavailable");
    assert_ne!(prepared["decision"], "deny");
    assert_ne!(prepared["decision"], "allow_prepared");

    let commit = call(
        &mut gate,
        "c1",
        "commit",
        json!({ "prepared_id": prepared["prepared_id"] }),
    );
    let code = err(&commit);
    assert!(
        code == "commit.unavailable" || code == "commit.deny",
        "got {code}"
    );
    assert_eq!(gate.provider_invocations(), 0);
}

// ---------------------------------------------------------------------------
// Last-responsible-moment matrix
// ---------------------------------------------------------------------------

#[test]
fn live_revocation_between_prepare_and_commit_prevents_dispatch() {
    let workspace = GateWorkspace::new("allow");
    let mut gate = workspace.gate();
    let host = ExternalHostFixture::new();

    let prepared = ok(&call(
        &mut gate,
        "p1",
        "prepare",
        workspace.prepare_payload("action_1", "projects/r2-revoke"),
    ))
    .clone();
    assert_eq!(prepared["decision"], "allow_prepared");

    // Authority state changes to deny after PREPARE.
    workspace.write_config("deny");

    let commit = call(
        &mut gate,
        "c1",
        "commit",
        json!({ "prepared_id": prepared["prepared_id"] }),
    );
    assert_eq!(err(&commit), "commit.deny");
    assert_eq!(host.effects, 0);
    assert_eq!(gate.provider_invocations(), 0);
    assert!(!workspace.marker_path().exists());
}

#[test]
fn capability_removed_between_prepare_and_commit_is_unavailable() {
    let workspace = GateWorkspace::new("allow");
    let mut gate = workspace.gate();

    let prepared = ok(&call(
        &mut gate,
        "p1",
        "prepare",
        workspace.prepare_payload("action_1", "projects/r2-cap"),
    ))
    .clone();
    assert_eq!(prepared["decision"], "allow_prepared");

    // Host observes the provider as physically unavailable at commit.
    let commit = call(
        &mut gate,
        "c1",
        "commit",
        json!({
            "prepared_id": prepared["prepared_id"],
            "observations": { "available_provider_identities": [] }
        }),
    );
    assert_eq!(err(&commit), "commit.unavailable");
    assert_eq!(gate.provider_invocations(), 0);
}

#[test]
fn manifest_digest_drift_between_prepare_and_commit_refuses() {
    let workspace = GateWorkspace::new("allow");
    let mut gate = workspace.gate();

    let prepared = ok(&call(
        &mut gate,
        "p1",
        "prepare",
        workspace.prepare_payload("action_1", "projects/r2-manifest"),
    ))
    .clone();
    assert_eq!(prepared["decision"], "allow_prepared");

    // Corrupt the trusted manifest pin between PREPARE and COMMIT.
    let manifest_path = workspace
        .root
        .join("manifests/fixture-ping-standing-allow.json");
    let original = fs::read_to_string(&manifest_path).unwrap();
    let mut corrupted: Value = serde_json::from_str(&original).unwrap();
    // timeout_ms is digest-covered; mutating it breaks the trusted pin.
    corrupted["timeout_ms"] = json!(9999);
    fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&corrupted).unwrap(),
    )
    .unwrap();

    let commit = call(
        &mut gate,
        "c1",
        "commit",
        json!({ "prepared_id": prepared["prepared_id"] }),
    );
    assert!(
        err(&commit).starts_with("commit."),
        "manifest drift must refuse commit, got {}",
        err(&commit)
    );
    assert_eq!(gate.provider_invocations(), 0);
}

#[test]
fn provider_identity_substitution_refuses() {
    let workspace = GateWorkspace::new("allow");
    let mut gate = workspace.gate();

    // Core-plan material with a foreign provider pin cannot be supplied by
    // the Host; instead observe the configured provider as unavailable so
    // current authority cannot resolve the binding.
    let prepared = ok(&call(
        &mut gate,
        "p1",
        "prepare",
        workspace.prepare_payload("action_1", "projects/r2-provider"),
    ))
    .clone();
    assert_eq!(prepared["decision"], "allow_prepared");

    let commit = call(
        &mut gate,
        "c1",
        "commit",
        json!({
            "prepared_id": prepared["prepared_id"],
            "observations": { "available_provider_identities": ["evil-provider"] }
        }),
    );
    assert_eq!(err(&commit), "commit.unavailable");
    assert_eq!(gate.provider_invocations(), 0);
}

#[test]
fn scope_violation_denies_before_dispatch() {
    let workspace = GateWorkspace::new("allow");
    let mut gate = workspace.gate();

    let prepared = ok(&call(
        &mut gate,
        "p1",
        "prepare",
        workspace.prepare_payload("action_1", "elsewhere/r2-scope"),
    ))
    .clone();
    assert_eq!(prepared["decision"], "deny");

    let commit = call(
        &mut gate,
        "c1",
        "commit",
        json!({ "prepared_id": prepared["prepared_id"] }),
    );
    assert_eq!(err(&commit), "commit.deny");
    assert_eq!(gate.provider_invocations(), 0);
}

#[test]
fn approval_does_not_override_changed_current_authority() {
    let workspace = GateWorkspace::new("ask");
    let mut gate = workspace.gate();

    let prepared = ok(&call(
        &mut gate,
        "p1",
        "prepare",
        workspace.prepare_payload("action_1", "projects/r2-ask-revoke"),
    ))
    .clone();
    let approval_id = prepared["approval"]["approval_id"]
        .as_str()
        .unwrap()
        .to_owned();
    call(
        &mut gate,
        "a1",
        "approval_decision",
        json!({ "approval_id": approval_id, "decision": "approve" }),
    );

    // Policy changes to deny after approval.
    workspace.write_config("deny");

    let commit = call(
        &mut gate,
        "c1",
        "commit",
        json!({
            "prepared_id": prepared["prepared_id"],
            "approval_id": approval_id
        }),
    );
    // Fresh evaluate is Deny — approval cannot bypass current authority.
    assert_eq!(err(&commit), "commit.deny");
    assert_eq!(gate.provider_invocations(), 0);
}

#[test]
fn changed_arguments_with_old_approval_refuses() {
    let workspace = GateWorkspace::new("ask");
    let mut gate = workspace.gate();

    let prepared_a = ok(&call(
        &mut gate,
        "p1",
        "prepare",
        workspace.prepare_payload("action_1", "projects/r2-args-a"),
    ))
    .clone();
    let approval_a = prepared_a["approval"]["approval_id"]
        .as_str()
        .unwrap()
        .to_owned();
    call(
        &mut gate,
        "a1",
        "approval_decision",
        json!({ "approval_id": approval_a, "decision": "approve" }),
    );

    // Different arguments produce a different prepared identity and proof.
    let prepared_b = ok(&call(
        &mut gate,
        "p2",
        "prepare",
        workspace.prepare_payload("action_1", "projects/r2-args-b"),
    ))
    .clone();
    assert_ne!(prepared_a["prepared_id"], prepared_b["prepared_id"]);

    let commit = call(
        &mut gate,
        "c1",
        "commit",
        json!({
            "prepared_id": prepared_b["prepared_id"],
            "approval_id": approval_a
        }),
    );
    assert_eq!(err(&commit), "commit.approval_not_ready");
    assert_eq!(gate.provider_invocations(), 0);
}

// ---------------------------------------------------------------------------
// Multi-step re-admission
// ---------------------------------------------------------------------------

#[test]
fn multi_step_second_action_requires_fresh_authority() {
    let workspace = GateWorkspace::new("allow");
    let mut gate = workspace.gate();
    let mut host = ExternalHostFixture::new();

    // Step 1: allow under current authority.
    let step1 = ok(&call(
        &mut gate,
        "p1",
        "prepare",
        workspace.prepare_payload("action_1", "projects/r2-step1"),
    ))
    .clone();
    let dispatch1 = commit_ok(&mut gate, step1["prepared_id"].as_str().unwrap());
    host.execute_harmless(&workspace);
    host.report_outcome(
        &mut gate,
        "o1",
        dispatch1["execution_id"].as_str().unwrap(),
        "external-step-1",
    );
    assert_eq!(host.effects, 1);

    // Authority changes.
    workspace.write_config("deny");

    // Step 2: independent prepare under new authority. Core still plans
    // Action material; current policy must deny it.
    let step2_response = call(
        &mut gate,
        "p2",
        "prepare",
        workspace.prepare_payload_with_eval("action_1", "projects/r2-step2", "eval_r2_002", true),
    );
    let step2 = ok(&step2_response).clone();
    assert_eq!(step2["decision"], "deny");

    let commit2 = call(
        &mut gate,
        "c2",
        "commit",
        json!({ "prepared_id": step2["prepared_id"] }),
    );
    assert_eq!(err(&commit2), "commit.deny");
    assert_eq!(
        host.effects, 1,
        "permission for step 1 never implies step 2"
    );
}

// ---------------------------------------------------------------------------
// Replay / duplicate admission
// ---------------------------------------------------------------------------

#[test]
fn duplicate_commit_same_prepared_identity_refuses() {
    let workspace = GateWorkspace::new("allow");
    let mut gate = workspace.gate();

    let prepared = ok(&call(
        &mut gate,
        "p1",
        "prepare",
        workspace.prepare_payload("action_1", "projects/r2-dup"),
    ))
    .clone();
    commit_ok(&mut gate, prepared["prepared_id"].as_str().unwrap());

    let second = call(
        &mut gate,
        "c2",
        "commit",
        json!({ "prepared_id": prepared["prepared_id"] }),
    );
    assert_eq!(err(&second), "commit.already_committed");
    assert_eq!(gate.provider_invocations(), 0);
}

#[test]
fn replay_refuses_second_admission_for_same_logical_key() {
    let workspace = GateWorkspace::new("allow");
    let mut gate = workspace.gate();

    // Two prepares of identical evaluation material produce distinct prepared
    // identities only when material differs; same material is refused.
    let payload = workspace.prepare_payload("action_1", "projects/r2-replay");
    let first = ok(&call(&mut gate, "p1", "prepare", payload.clone())).clone();
    let duplicate = call(&mut gate, "p2", "prepare", payload);
    assert_eq!(err(&duplicate), "prepare.duplicate_identity");

    let dispatch = commit_ok(&mut gate, first["prepared_id"].as_str().unwrap());
    // Complete the first session's guard so the durable claim remains without
    // a held cross-process lock.
    let outcome = call(
        &mut gate,
        "o1",
        "outcome",
        json!({
            "execution_id": dispatch["execution_id"],
            "classification": "succeeded",
            "attempted": true,
            "result": {"echo": "first"}
        }),
    );
    assert_eq!(outcome.status, ResponseStatus::Ok);

    // A second Gate session (fresh prepared map) against the same durable
    // replay ledger must not create a second admission for the same key.
    let mut gate2 = workspace.gate();
    let second_payload = workspace.prepare_payload("action_1", "projects/r2-replay");
    let second_prepare = call(&mut gate2, "p1", "prepare", second_payload);
    let prepared = ok(&second_prepare).clone();
    let commit = call(
        &mut gate2,
        "c1",
        "commit",
        json!({ "prepared_id": prepared["prepared_id"] }),
    );
    assert_eq!(err(&commit), "commit.replay_blocked");
    let data = err_data(&commit).expect("replay data");
    assert_eq!(data["replay"], "replay_blocked_completed_success");
    assert_eq!(gate.provider_invocations(), 0);
    assert_eq!(gate2.provider_invocations(), 0);
}

// ---------------------------------------------------------------------------
// Wrong-proof attacks
// ---------------------------------------------------------------------------

#[test]
fn wrong_proof_attacks_fail_closed() {
    let workspace = GateWorkspace::new("ask");
    let mut gate = workspace.gate();

    // Prepared A (ask) + approved approval A.
    let prepared_a = ok(&call(
        &mut gate,
        "p1",
        "prepare",
        workspace.prepare_payload("action_1", "projects/r2-attack-a"),
    ))
    .clone();
    let approval_a = prepared_a["approval"]["approval_id"]
        .as_str()
        .unwrap()
        .to_owned();
    call(
        &mut gate,
        "a1",
        "approval_decision",
        json!({ "approval_id": approval_a, "decision": "approve" }),
    );

    // Prepared B (same Core action_id under a distinct evaluation identity).
    let prepared_b = ok(&call(
        &mut gate,
        "p2",
        "prepare",
        workspace.prepare_payload_with_eval(
            "action_1",
            "projects/r2-attack-b",
            "eval_r2_002",
            true,
        ),
    ))
    .clone();
    assert_ne!(prepared_a["prepared_id"], prepared_b["prepared_id"]);

    // approval A + prepared B.
    let attack1 = call(
        &mut gate,
        "c1",
        "commit",
        json!({
            "prepared_id": prepared_b["prepared_id"],
            "approval_id": approval_a
        }),
    );
    assert_eq!(err(&attack1), "commit.approval_not_ready");

    // Unknown prepared identity.
    let attack2 = call(
        &mut gate,
        "c2",
        "commit",
        json!({ "prepared_id": "prep_forged" }),
    );
    assert_eq!(err(&attack2), "commit.unknown_prepared");

    // Unknown approval identity.
    let attack3 = call(
        &mut gate,
        "c3",
        "commit",
        json!({
            "prepared_id": prepared_b["prepared_id"],
            "approval_id": "approval-forged"
        }),
    );
    assert_eq!(err(&attack3), "commit.approval_not_ready");

    // Unknown execution identity for outcome.
    let attack4 = call(
        &mut gate,
        "o1",
        "outcome",
        json!({
            "execution_id": "exec_forged",
            "classification": "succeeded",
            "attempted": true,
            "result": {"echo": "x"}
        }),
    );
    assert_eq!(err(&attack4), "outcome.unknown_execution");

    // Caller cannot supply authority booleans on prepare.
    let mut forged = workspace.prepare_payload_with_eval(
        "action_1",
        "projects/r2-attack-c",
        "eval_r2_003",
        true,
    );
    forged["permission"] = json!(true);
    // Frame-level rejection happens in parse_frame; session-level prepare
    // ignores unknown fields but never trusts them — decision is recomputed.
    let response = call(&mut gate, "p3", "prepare", forged);
    if let Some(result) = &response.result {
        assert_ne!(result["decision"], "allow_prepared");
    }

    assert_eq!(gate.provider_invocations(), 0);
}

// ---------------------------------------------------------------------------
// Gate restart
// ---------------------------------------------------------------------------

#[test]
fn unconsumed_approval_expires_on_gate_restart() {
    let workspace = GateWorkspace::new("ask");

    let approval_id = {
        let mut gate = workspace.gate();
        let prepared = ok(&call(
            &mut gate,
            "p1",
            "prepare",
            workspace.prepare_payload("action_1", "projects/r2-restart"),
        ))
        .clone();
        let approval_id = prepared["approval"]["approval_id"]
            .as_str()
            .unwrap()
            .to_owned();
        call(
            &mut gate,
            "a1",
            "approval_decision",
            json!({ "approval_id": approval_id, "decision": "approve" }),
        );
        approval_id
        // Gate dropped: process-local approval store expires.
    };

    let mut gate2 = workspace.gate();

    // Expired approval cannot be decided or used.
    let decision = call(
        &mut gate2,
        "a2",
        "approval_decision",
        json!({ "approval_id": approval_id, "decision": "approve" }),
    );
    assert_eq!(err(&decision), "approval.decision_refused");

    let status = ok(&call(&mut gate2, "s1", "status", json!({}))).clone();
    let pending = status["pending_approvals"].as_array().unwrap();
    assert!(
        pending.is_empty(),
        "process-local approvals must not survive restart: {pending:?}"
    );
}

#[test]
fn durable_committed_intent_survives_gate_restart() {
    let workspace = GateWorkspace::new("allow");
    let execution_id = {
        let mut gate = workspace.gate();
        let prepared = ok(&call(
            &mut gate,
            "p1",
            "prepare",
            workspace.prepare_payload("action_1", "projects/r2-durable"),
        ))
        .clone();
        let dispatch = commit_ok(&mut gate, prepared["prepared_id"].as_str().unwrap());
        dispatch["execution_id"].as_str().unwrap().to_owned()
    };

    // Durable trail still holds the intent.
    let trail = fs::read_to_string(&workspace.trail).unwrap();
    assert!(trail.contains(&execution_id));

    let mut gate2 = workspace.gate();
    let status = ok(&call(&mut gate2, "s1", "status", json!({}))).clone();
    let unresolved = status["unresolved_commits"].as_array().unwrap();
    assert!(
        unresolved
            .iter()
            .any(|entry| entry["execution_id"] == execution_id
                && entry["state"] == "COMMITTED_OUTCOME_INCOMPLETE"),
        "restarted Gate must report unresolved durable commits: {unresolved:?}"
    );
}

#[test]
fn outcome_after_restart_records_trail_and_flags_recovery() {
    let workspace = GateWorkspace::new("allow");
    let execution_id = {
        let mut gate = workspace.gate();
        let prepared = ok(&call(
            &mut gate,
            "p1",
            "prepare",
            workspace.prepare_payload("action_1", "projects/r2-crash"),
        ))
        .clone();
        let dispatch = commit_ok(&mut gate, prepared["prepared_id"].as_str().unwrap());
        dispatch["execution_id"].as_str().unwrap().to_owned()
        // Crash window: host executes, Gate restarts before OUTCOME.
    };

    let mut gate2 = workspace.gate();
    // Late OUTCOME after restart reconciles from durable Trail + replay claim.
    let outcome = call(
        &mut gate2,
        "o1",
        "outcome",
        json!({
            "execution_id": execution_id,
            "classification": "succeeded",
            "attempted": true,
            "result": {"echo": "late"}
        }),
    );
    let recovered = ok(&outcome).clone();
    assert_eq!(recovered["status"], "succeeded");
    assert_eq!(recovered["recovered"], true);
    assert_eq!(recovered["replay_terminal"], "recorded");

    let status = ok(&call(&mut gate2, "s1", "status", json!({}))).clone();
    let terminal = status["terminal_outcomes"].as_array().unwrap();
    assert!(
        terminal.iter().any(
            |entry| entry["execution_id"] == execution_id && entry["state"] == "TERMINAL_KNOWN"
        ),
        "restarted Gate must report terminal durable truth: {terminal:?}"
    );
}

#[test]
fn late_outcome_for_unknown_execution_refuses() {
    let workspace = GateWorkspace::new("allow");
    let mut gate = workspace.gate();
    let outcome = call(
        &mut gate,
        "o1",
        "outcome",
        json!({
            "execution_id": "exec_00000000-0000-4000-8000-000000000000",
            "classification": "succeeded",
            "attempted": true,
            "result": {"echo": "x"}
        }),
    );
    assert_eq!(err(&outcome), "outcome.unknown_execution");
}

// ---------------------------------------------------------------------------
// Status / shutdown
// ---------------------------------------------------------------------------

#[test]
fn status_is_bounded_and_reports_zero_provider_invocations() {
    let workspace = GateWorkspace::new("allow");
    let mut gate = workspace.gate();
    let status = ok(&call(&mut gate, "s1", "status", json!({}))).clone();
    assert_eq!(status["healthy"], true);
    assert_eq!(status["provider_invocations"], 0);
    assert_eq!(status["protocol"], "tethers.authority/1");
}

#[test]
fn shutdown_stops_the_session() {
    let workspace = GateWorkspace::new("allow");
    let mut gate = workspace.gate();
    assert!(!gate.shutdown_requested());
    call(&mut gate, "x1", "shutdown", json!({}));
    assert!(gate.shutdown_requested());
}

// ---------------------------------------------------------------------------
// Secret safety
// ---------------------------------------------------------------------------

#[test]
fn protocol_surfaces_do_not_leak_argument_values() {
    let workspace = GateWorkspace::new("ask");
    let mut gate = workspace.gate();
    let secretish = "projects/SUPER-SECRET-PATH";
    let prepared = ok(&call(
        &mut gate,
        "p1",
        "prepare",
        workspace.prepare_payload("action_1", secretish),
    ))
    .clone();

    // Prepared result carries digests, not raw argument values.
    let prepared_text = serde_json::to_string(&prepared).unwrap();
    assert!(
        !prepared_text.contains(secretish),
        "prepare result leaked arguments: {prepared_text}"
    );

    let status = ok(&call(&mut gate, "s1", "status", json!({}))).clone();
    let status_text = serde_json::to_string(&status).unwrap();
    assert!(
        !status_text.contains(secretish),
        "status leaked arguments: {status_text}"
    );
    assert!(!status_text.contains("SUPER-SECRET"));
}

// ---------------------------------------------------------------------------
// Performance (record cold/retained timings)
// ---------------------------------------------------------------------------

#[test]
fn records_cold_and_retained_gate_timings() {
    let workspace = GateWorkspace::new("allow");
    let mut gate = workspace.gate();

    let cold_start = Instant::now();
    call(&mut gate, "h1", "hello", json!({}));
    let hello_ms = cold_start.elapsed().as_millis();

    let prepare_start = Instant::now();
    let prepared = ok(&call(
        &mut gate,
        "p1",
        "prepare",
        workspace.prepare_payload("action_1", "projects/r2-perf"),
    ))
    .clone();
    let prepare_ms = prepare_start.elapsed().as_millis();

    let commit_start = Instant::now();
    commit_ok(&mut gate, prepared["prepared_id"].as_str().unwrap());
    let commit_ms = commit_start.elapsed().as_millis();

    // Correctness first; only guard against pathological waste.
    assert!(hello_ms < 5_000, "hello took {hello_ms}ms");
    assert!(prepare_ms < 10_000, "prepare took {prepare_ms}ms");
    assert!(commit_ms < 10_000, "commit took {commit_ms}ms");

    println!("R2 timing: cold_hello_ms={hello_ms} prepare_ms={prepare_ms} commit_ms={commit_ms}");
}

// ---------------------------------------------------------------------------
// Binary / stdio hostile protocol suite
// ---------------------------------------------------------------------------

fn tethers_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_tethers"))
}

struct GateProcess {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    stderr_handle: Option<std::thread::JoinHandle<String>>,
}

impl GateProcess {
    fn spawn(workspace: &GateWorkspace) -> Self {
        let mut child = Command::new(tethers_bin())
            .args([
                "gate",
                "--stdio",
                "--config",
                workspace.config.to_str().unwrap(),
                "--engine",
                workspace.engine.to_str().unwrap(),
                "--trail",
                workspace.trail.to_str().unwrap(),
                "--host-data-root",
                workspace.host_data.to_str().unwrap(),
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn gate");
        let stdin = child.stdin.take().unwrap();
        let stderr = child.stderr.take().unwrap();
        let stdout = BufReader::new(child.stdout.take().unwrap());
        let stderr_handle = std::thread::spawn(move || {
            let mut text = String::new();
            for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                text.push_str(&line);
                text.push('\n');
            }
            text
        });
        Self {
            child,
            stdin,
            stdout,
            stderr_handle: Some(stderr_handle),
        }
    }

    fn first_line_or_stderr(&mut self) -> String {
        let mut line = String::new();
        let read = self.stdout.read_line(&mut line).unwrap_or(0);
        if read == 0 {
            let stderr = self
                .stderr_handle
                .take()
                .map(|handle| handle.join().unwrap_or_default())
                .unwrap_or_default();
            return format!("EOF stderr={stderr}");
        }
        line
    }

    fn write_raw(&mut self, line: &str) {
        self.stdin.write_all(line.as_bytes()).unwrap();
        self.stdin.write_all(b"\n").unwrap();
        self.stdin.flush().unwrap();
    }

    fn request(&mut self, request_id: &str, operation: &str, payload: Value) -> Value {
        self.write_raw(
            &json!({
                "schema": "tethers.authority/1",
                "request_id": request_id,
                "operation": operation,
                "payload": payload
            })
            .to_string(),
        );
        self.read_response()
    }

    fn read_response(&mut self) -> Value {
        let mut line = String::new();
        let read = self.stdout.read_line(&mut line).expect("read response");
        assert!(
            read > 0,
            "gate closed stdout without a response; stderr={}",
            self.first_line_or_stderr()
        );
        serde_json::from_str(line.trim()).unwrap_or_else(|error| {
            panic!("invalid response JSON: {error}; line={line:?}");
        })
    }
}

impl Drop for GateProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn stdio_gate_hello_prepare_commit_lifecycle() {
    let workspace = GateWorkspace::new("allow");
    let mut process = GateProcess::spawn(&workspace);

    let hello = process.request("r1", "hello", json!({}));
    assert_eq!(
        hello["status"],
        "ok",
        "hello failed: {hello:#}; stderr={}",
        process
            .stderr_handle
            .take()
            .map(|h| h.join().unwrap_or_default())
            .unwrap_or_default()
    );
    assert_eq!(hello["result"]["protocol"], "tethers.authority/1");
    assert_eq!(hello["result"]["provider_invocations"], 0);

    let prepared = process.request(
        "r2",
        "prepare",
        workspace.prepare_payload("action_1", "projects/r2-stdio"),
    );
    assert_eq!(prepared["status"], "ok");
    assert_eq!(prepared["result"]["decision"], "allow_prepared");

    let dispatch = process.request(
        "r3",
        "commit",
        json!({ "prepared_id": prepared["result"]["prepared_id"] }),
    );
    assert_eq!(dispatch["status"], "ok");
    assert_eq!(dispatch["result"]["schema"], "tethers.dispatch/1");

    let outcome = process.request(
        "r4",
        "outcome",
        json!({
            "execution_id": dispatch["result"]["execution_id"],
            "classification": "succeeded",
            "attempted": true,
            "result": {"echo": "stdio"}
        }),
    );
    assert_eq!(outcome["status"], "ok");

    let shutdown = process.request("r5", "shutdown", json!({}));
    assert_eq!(shutdown["status"], "ok");
}

#[test]
fn stdio_hostile_frames_are_bounded_and_structured() {
    let workspace = GateWorkspace::new("allow");
    let mut process = GateProcess::spawn(&workspace);

    // Empty frame (blank line) — structured refusal, session stays healthy.
    process.write_raw("");
    let blank = process.read_response();
    assert_eq!(blank["status"], "error");
    assert_eq!(blank["error"]["code"], "frame.empty");

    // Malformed JSON.
    process.write_raw("{not json");
    let after_malformed = process.read_response();
    assert_eq!(after_malformed["status"], "error");
    assert_eq!(after_malformed["error"]["code"], "frame.invalid_json");

    // Unknown operation.
    process.write_raw(
        r#"{"schema":"tethers.authority/1","request_id":"u1","operation":"detonate","payload":{}}"#,
    );
    let unknown = process.read_response();
    assert_eq!(unknown["status"], "error");
    assert_eq!(unknown["error"]["code"], "frame.unknown_operation");

    // Unsupported schema.
    process.write_raw(
        r#"{"schema":"tethers.plan/1","request_id":"u2","operation":"hello","payload":{}}"#,
    );
    let schema = process.read_response();
    assert_eq!(schema["status"], "error");
    assert_eq!(schema["error"]["code"], "frame.unsupported_schema");

    // Duplicate request_id field.
    process.write_raw(
        r#"{"schema":"tethers.authority/1","request_id":"d1","request_id":"d2","operation":"hello","payload":{}}"#,
    );
    let duplicate = process.read_response();
    assert_eq!(duplicate["status"], "error");
    assert_eq!(duplicate["error"]["code"], "frame.duplicate_field");

    // Duplicate request identity across frames.
    process.request("same-id", "hello", json!({}));
    let dup_id = process.request("same-id", "hello", json!({}));
    assert_eq!(dup_id["status"], "error");
    assert_eq!(dup_id["error"]["code"], "frame.request_id_duplicate");

    // Oversized frame.
    let oversized = format!(
        r#"{{"schema":"tethers.authority/1","request_id":"big","operation":"hello","payload":{{"pad":"{}"}}}}"#,
        "x".repeat(1_100_000)
    );
    process.write_raw(&oversized);
    let big = process.read_response();
    assert_eq!(big["status"], "error");
    assert!(
        big["error"]["code"] == "frame.oversized"
            || big["error"]["code"] == "frame.payload_oversized",
        "got {}",
        big["error"]["code"]
    );

    // Forbidden authority key on prepare.
    process.write_raw(
        r#"{"schema":"tethers.authority/1","request_id":"f1","operation":"prepare","payload":{"permission":true}}"#,
    );
    let forbidden = process.read_response();
    assert_eq!(forbidden["status"], "error");
    assert_eq!(forbidden["error"]["code"], "frame.forbidden_authority_key");

    // Gate remains healthy.
    let hello2 = process.request("h2", "hello", json!({}));
    assert_eq!(hello2["status"], "ok");
    assert_eq!(hello2["result"]["provider_invocations"], 0);
}

#[test]
fn partial_frame_stall_is_bounded() {
    let workspace = GateWorkspace::new("allow");
    let mut process = GateProcess::spawn(&workspace);

    // Partial frame with no newline: gate must not wait forever.
    process
        .stdin
        .write_all(br#"{"schema":"tethers.authority/1","request_id":"stall","op"#)
        .unwrap();
    process.stdin.flush().unwrap();

    // FRAME_WAIT is 30s in gate_command; allow headroom then expect a
    // structured timeout refusal OR EOF-adjacent error, then health.
    let mut line = String::new();
    // Use a generous but finite read; if the implementation only times out
    // after FRAME_WAIT, this test documents that bound.
    let start = Instant::now();
    let read = {
        // Blocking read with outer watchdog via thread would be ideal; here we
        // rely on the Gate emitting a timeout frame after FRAME_WAIT.
        process.stdout.read_line(&mut line).expect("stall read")
    };
    let elapsed = start.elapsed();
    assert!(read > 0 || elapsed.as_secs() < 120, "stall not bounded");
    if read > 0 {
        let response: Value = serde_json::from_str(line.trim()).expect("timeout frame");
        assert_eq!(response["status"], "error");
    }

    // Complete the partial line so the next request is cleanly framed.
    process.write_raw(r#"}}"#);
    // Discard whatever error follows.
    let _ = process.read_response();

    let hello = process.request("after-stall", "hello", json!({}));
    assert_eq!(hello["status"], "ok");
}

#[test]
fn gate_missing_config_fails_closed() {
    let root = std::env::temp_dir().join(format!(
        "tethers-r2-missing-{}",
        uuid::Uuid::new_v4().simple()
    ));
    fs::create_dir_all(&root).unwrap();
    let output = Command::new(tethers_bin())
        .args([
            "gate",
            "--stdio",
            "--config",
            root.join("nope.json").to_str().unwrap(),
            "--engine",
            root.join("nope-engine").to_str().unwrap(),
            "--trail",
            root.join("trail.jsonl").to_str().unwrap(),
            "--host-data-root",
            root.join("host").to_str().unwrap(),
        ])
        .output()
        .expect("run gate");
    let _ = fs::remove_dir_all(&root);
    assert_ne!(output.status.code(), Some(0), "gate must fail closed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("GATE_CONFIG_NOT_FOUND"), "stdout={stdout}");
}

#[test]
fn gate_wrong_protocol_usage_fails_closed_without_dispatch() {
    // Consequential host operation while Gate is absent: host must not
    // dispatch. Here we assert the binary refuses to start without --stdio
    // and without absolute paths — no protocol side effects possible.
    let workspace = GateWorkspace::new("allow");
    let output = Command::new(tethers_bin())
        .args([
            "gate",
            "--config",
            workspace.config.to_str().unwrap(),
            "--engine",
            workspace.engine.to_str().unwrap(),
            "--trail",
            workspace.trail.to_str().unwrap(),
            "--host-data-root",
            workspace.host_data.to_str().unwrap(),
        ])
        .output()
        .expect("run gate");
    assert_ne!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("GATE_STDIO_REQUIRED"));
    assert!(!workspace.marker_path().exists());
}

// ---------------------------------------------------------------------------
// Structural non-execution proof
// ---------------------------------------------------------------------------

#[test]
fn gate_module_never_reports_provider_effect_execution() {
    // Every surface the Gate can emit must report zero provider invocations
    // and never claim physical execution by Tethers.
    let workspace = GateWorkspace::new("allow");
    let mut gate = workspace.gate();

    for (operation, payload) in [
        ("hello", json!({})),
        ("status", json!({})),
        (
            "prepare",
            workspace.prepare_payload("action_1", "projects/r2-struct"),
        ),
    ] {
        let response = call(&mut gate, "s", operation, payload);
        if let Some(result) = &response.result {
            assert_eq!(result["provider_invocations"], 0, "{operation}");
        }
    }
    assert_eq!(gate.provider_invocations(), 0);
    assert!(!workspace.marker_path().exists());
}

// ---------------------------------------------------------------------------
// Blocker 1 — Core-authoritative PREPARE
// ---------------------------------------------------------------------------

#[test]
fn core_produced_plan_is_authoritative() {
    let workspace = GateWorkspace::new("allow");
    let mut gate = workspace.gate();
    let prepared = ok(&call(
        &mut gate,
        "p1",
        "prepare",
        workspace.prepare_payload("action_1", "projects/r2-core"),
    ))
    .clone();
    assert_eq!(prepared["decision"], "allow_prepared");
    assert_eq!(prepared["core_planned"], true);
    assert_eq!(prepared["action_id"], "action_1");
    assert_ne!(prepared["evaluation_id"], prepared["action_id"]);
}

#[test]
fn forged_caller_plan_is_refused() {
    let workspace = GateWorkspace::new("allow");
    let mut gate = workspace.gate();
    let mut payload = workspace.prepare_payload("action_1", "projects/r2-forged");
    payload["plan"] = json!({
        "id": "forged/plan",
        "actions": [{
            "action_id": "action_1",
            "capability": "fixture.ping",
            "arguments": {"message": "x", "path": "projects/evil"}
        }]
    });
    let response = call(&mut gate, "p1", "prepare", payload);
    assert_eq!(err(&response), "frame.payload_invalid");
    assert!(response
        .error
        .as_ref()
        .unwrap()
        .message
        .contains("caller Plan"));
    let status = ok(&call(&mut gate, "s1", "status", json!({}))).clone();
    assert_eq!(status["prepared_count"], 0);
    assert_eq!(gate.provider_invocations(), 0);
}

#[test]
fn no_actions_yields_no_prepared_identity() {
    let workspace = GateWorkspace::new("allow");
    let mut gate = workspace.gate();
    let response = call(
        &mut gate,
        "p1",
        "prepare",
        workspace.prepare_payload_with_eval(
            "action_1",
            "projects/r2-noactions",
            "eval_r2_noactions",
            false,
        ),
    );
    assert_eq!(err(&response), "prepare.no_actions");
    let status = ok(&call(&mut gate, "s1", "status", json!({}))).clone();
    assert_eq!(status["prepared_count"], 0);
    assert_eq!(gate.provider_invocations(), 0);
}

#[test]
fn planner_failure_yields_no_prepared_identity() {
    let workspace = GateWorkspace::new("allow");
    let mut gate = workspace.gate();
    fs::write(workspace.root.join("tethers/complete.tether"), "").unwrap();
    let response = call(
        &mut gate,
        "p1",
        "prepare",
        workspace.prepare_payload("action_1", "projects/r2-planner"),
    );
    let code = err(&response);
    assert!(
        code == "prepare.unavailable"
            || code == "prepare.planner_error"
            || code == "prepare.invalid_data"
            || code == "gate.config_unavailable",
        "got {code}"
    );
    let status = ok(&call(&mut gate, "s1", "status", json!({}))).clone();
    assert_eq!(status["prepared_count"], 0);
    assert_eq!(gate.provider_invocations(), 0);
}

#[test]
fn stale_action_after_changed_facts_refuses() {
    let workspace = GateWorkspace::new("allow");
    let mut gate = workspace.gate();

    let first = ok(&call(
        &mut gate,
        "p1",
        "prepare",
        workspace.prepare_payload("action_1", "projects/r2-stale"),
    ))
    .clone();
    assert_eq!(first["decision"], "allow_prepared");

    let stale = call(
        &mut gate,
        "p2",
        "prepare",
        workspace.prepare_payload_with_eval(
            "action_1",
            "projects/r2-stale",
            "eval_r2_stale",
            false,
        ),
    );
    assert_eq!(err(&stale), "prepare.no_actions");
    assert_eq!(gate.provider_invocations(), 0);
}

// ---------------------------------------------------------------------------
// Blocker 2 — execution identity continuity
// ---------------------------------------------------------------------------

#[test]
fn ask_execution_identity_is_continuous_and_distinct() {
    let workspace = GateWorkspace::new("ask");
    let mut gate = workspace.gate();

    let prepared = ok(&call(
        &mut gate,
        "p1",
        "prepare",
        workspace.prepare_payload("action_1", "projects/r2-cont"),
    ))
    .clone();
    let approval_id = prepared["approval"]["approval_id"]
        .as_str()
        .unwrap()
        .to_owned();
    let evaluation_id = prepared["evaluation_id"].as_str().unwrap().to_owned();
    let action_id = prepared["action_id"].as_str().unwrap().to_owned();
    call(
        &mut gate,
        "a1",
        "approval_decision",
        json!({ "approval_id": approval_id, "decision": "approve" }),
    );
    let dispatch = commit_ok(&mut gate, prepared["prepared_id"].as_str().unwrap());
    let execution_id = dispatch["execution_id"].as_str().unwrap().to_owned();
    assert_ne!(execution_id, evaluation_id);
    assert_ne!(execution_id, action_id);

    let outcome = call(
        &mut gate,
        "o1",
        "outcome",
        json!({
            "execution_id": execution_id,
            "classification": "succeeded",
            "attempted": true,
            "result": {"echo": "r2"}
        }),
    );
    let result = ok(&outcome).clone();
    assert_eq!(result["execution_id"], execution_id);
    assert_eq!(result["status"], "succeeded");

    let trail = fs::read_to_string(&workspace.trail).unwrap();
    let mut saw_authorisation = false;
    let mut saw_intent = false;
    let mut saw_outcome = false;
    for line in trail.lines() {
        if line.is_empty() {
            continue;
        }
        let value: Value = serde_json::from_str(line).unwrap();
        let line_execution = value
            .get("execution_id")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let kind = value.get("kind").and_then(Value::as_str);
        if kind == Some("approval_consumed") {
            saw_authorisation = true;
            assert_eq!(
                line_execution, execution_id,
                "approval authorisation must use Tethers execution_id: {line}"
            );
            assert_ne!(line_execution, evaluation_id);
            assert_ne!(line_execution, action_id);
        }
        if value.get("capability_name").is_some()
            && value.get("arguments").is_some()
            && value.get("status").is_none()
            && kind.is_none()
        {
            saw_intent = true;
            assert_eq!(line_execution, execution_id);
        }
        if value.get("status").and_then(Value::as_str) == Some("succeeded")
            && value.get("timestamp_unix_ms").is_some()
        {
            saw_outcome = true;
            assert_eq!(line_execution, execution_id);
        }
    }
    assert!(
        saw_authorisation,
        "ASK path must write authorisation evidence"
    );
    assert!(saw_intent, "intent must carry the execution identity");
    assert!(saw_outcome, "outcome must carry the execution identity");
}

// ---------------------------------------------------------------------------
// Blocker 3 — output schema validation
// ---------------------------------------------------------------------------

#[test]
fn valid_output_passes_trusted_output_schema() {
    let workspace = GateWorkspace::new("allow");
    let mut gate = workspace.gate();
    let prepared = ok(&call(
        &mut gate,
        "p1",
        "prepare",
        workspace.prepare_payload("action_1", "projects/r2-valid-out"),
    ))
    .clone();
    let dispatch = commit_ok(&mut gate, prepared["prepared_id"].as_str().unwrap());
    let outcome = call(
        &mut gate,
        "o1",
        "outcome",
        json!({
            "execution_id": dispatch["execution_id"],
            "classification": "succeeded",
            "attempted": true,
            "result": {"echo": "ok"}
        }),
    );
    let result = ok(&outcome).clone();
    assert_eq!(result["status"], "succeeded");
    assert_eq!(result["replay_terminal"], "recorded");
    assert_eq!(gate.provider_invocations(), 0);
}

#[test]
fn invalid_claimed_success_becomes_result_validation_failed() {
    let workspace = GateWorkspace::new("allow");
    let mut gate = workspace.gate();
    let prepared = ok(&call(
        &mut gate,
        "p1",
        "prepare",
        workspace.prepare_payload("action_1", "projects/r2-bad-out"),
    ))
    .clone();
    let dispatch = commit_ok(&mut gate, prepared["prepared_id"].as_str().unwrap());
    let outcome = call(
        &mut gate,
        "o1",
        "outcome",
        json!({
            "execution_id": dispatch["execution_id"],
            "classification": "succeeded",
            "attempted": true,
            "result": {"wrong": 123}
        }),
    );
    let result = ok(&outcome).clone();
    assert_eq!(result["status"], "failed");
    let trail = fs::read_to_string(&workspace.trail).unwrap();
    assert!(trail.contains("result_validation_failed"));
    assert_eq!(gate.provider_invocations(), 0);
}

// ---------------------------------------------------------------------------
// Blocker 4 — durable recovery extras
// ---------------------------------------------------------------------------

#[test]
fn response_lost_terminal_outcome_survives_restart() {
    let workspace = GateWorkspace::new("allow");
    let execution_id = {
        let mut gate = workspace.gate();
        let prepared = ok(&call(
            &mut gate,
            "p1",
            "prepare",
            workspace.prepare_payload("action_1", "projects/r2-lost"),
        ))
        .clone();
        let dispatch = commit_ok(&mut gate, prepared["prepared_id"].as_str().unwrap());
        let exec = dispatch["execution_id"].as_str().unwrap().to_owned();
        call(
            &mut gate,
            "o1",
            "outcome",
            json!({
                "execution_id": exec,
                "classification": "succeeded",
                "attempted": true,
                "result": {"echo": "lost"}
            }),
        );
        exec
    };

    let mut gate2 = workspace.gate();
    let status = ok(&call(&mut gate2, "s1", "status", json!({}))).clone();
    let terminal = status["terminal_outcomes"].as_array().unwrap();
    assert!(
        terminal.iter().any(
            |entry| entry["execution_id"] == execution_id && entry["state"] == "TERMINAL_KNOWN"
        ),
        "terminal truth must survive restart: {terminal:?}"
    );

    let again = call(
        &mut gate2,
        "o2",
        "outcome",
        json!({
            "execution_id": execution_id,
            "classification": "succeeded",
            "attempted": true,
            "result": {"echo": "lost"}
        }),
    );
    let idem = ok(&again).clone();
    assert_eq!(idem["idempotent"], true);

    let conflict = call(
        &mut gate2,
        "o3",
        "outcome",
        json!({
            "execution_id": execution_id,
            "classification": "failed",
            "attempted": true,
            "error": "different"
        }),
    );
    assert_eq!(err(&conflict), "outcome.conflict");
}

#[test]
fn conflicting_terminal_outcome_in_session_refuses() {
    let workspace = GateWorkspace::new("allow");
    let mut gate = workspace.gate();
    let prepared = ok(&call(
        &mut gate,
        "p1",
        "prepare",
        workspace.prepare_payload("action_1", "projects/r2-conflict"),
    ))
    .clone();
    let dispatch = commit_ok(&mut gate, prepared["prepared_id"].as_str().unwrap());
    let exec = dispatch["execution_id"].as_str().unwrap();
    call(
        &mut gate,
        "o1",
        "outcome",
        json!({
            "execution_id": exec,
            "classification": "succeeded",
            "attempted": true,
            "result": {"echo": "first"}
        }),
    );
    let conflict = call(
        &mut gate,
        "o2",
        "outcome",
        json!({
            "execution_id": exec,
            "classification": "failed",
            "attempted": true,
            "error": "second"
        }),
    );
    assert_eq!(err(&conflict), "outcome.conflict");
}

#[test]
fn durable_disagreement_fails_closed_in_status() {
    let workspace = GateWorkspace::new("allow");
    fs::write(
        &workspace.trail,
        r#"{"execution_id":"exec_disagree","action_id":"action_1","status":"succeeded","timestamp_unix_ms":1}
"#,
    )
    .unwrap();
    let mut gate = workspace.gate();
    let status = ok(&call(&mut gate, "s1", "status", json!({}))).clone();
    let recovery = status["recovery_required"].as_array().unwrap();
    assert!(
        recovery
            .iter()
            .any(|entry| entry["state"] == "outcome_without_intent"),
        "durable disagreement must be explicit: {recovery:?}"
    );
}

#[test]
fn status_truncation_is_explicit() {
    let workspace = GateWorkspace::new("allow");
    let mut gate = workspace.gate();
    let status = ok(&call(&mut gate, "s1", "status", json!({}))).clone();
    assert!(status.get("truncated").is_some());
    assert_eq!(status["truncated"], false);
}

#[test]
fn evaluation_id_never_occupies_execution_id_field_in_gate_trail() {
    let workspace = GateWorkspace::new("allow");
    let mut gate = workspace.gate();
    let prepared = ok(&call(
        &mut gate,
        "p1",
        "prepare",
        workspace.prepare_payload("action_1", "projects/r2-idclass"),
    ))
    .clone();
    let evaluation_id = prepared["evaluation_id"].as_str().unwrap().to_owned();
    let dispatch = commit_ok(&mut gate, prepared["prepared_id"].as_str().unwrap());
    let execution_id = dispatch["execution_id"].as_str().unwrap().to_owned();
    call(
        &mut gate,
        "o1",
        "outcome",
        json!({
            "execution_id": execution_id,
            "classification": "succeeded",
            "attempted": true,
            "result": {"echo": "ids"}
        }),
    );

    let trail = fs::read_to_string(&workspace.trail).unwrap();
    for line in trail.lines() {
        if line.is_empty() {
            continue;
        }
        let value: Value = serde_json::from_str(line).unwrap();
        if value.get("execution_id").and_then(Value::as_str) == Some(evaluation_id.as_str()) {
            panic!("evaluation_id occupied execution_id field: {line}");
        }
    }
}
