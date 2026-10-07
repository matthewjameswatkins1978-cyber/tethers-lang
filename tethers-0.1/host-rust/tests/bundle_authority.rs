//! Phase A atomic host-execution bundle authority acceptance (A14 proofs 1–22).
//!
//! Proves, over a Host-bound fixture capability (`fixture.bundle`):
//! ordinary Actions + sequential PREPARE + atomic bundle COMMIT
//! (`commit_bundle` on `tethers.authority/2`) with Omen-owned opaque
//! composition. Every test asserts zero provider invocation: the Gate
//! authorises, it never executes.
//!
//! Proof map:
//!  1 sequential PREPARE, zero provider invocation
//!  2 deny-C commit_bundle, zero admission
//!  3 unavailable-C commit_bundle, zero admission
//!  4 replay-conflict-B commit_bundle, zero admission
//!  5 invalid approval consumes nothing (retry with valid approvals succeeds)
//!  6 success returns all member execution identities
//!  7 exactly one bundle identity, one committed marker
//!  8 no post-bundle individual commit (session + restart)
//!  9 no second bundle commit (session + restart)
//! 10 no replay after terminal outcomes (restart re-admission refused)
//! 11 reorder fails, identical retry resumes
//! 12 remove (member_count) fails
//! 13 add fails
//! 14 changed composition fails
//! 15 changed args (cross-approval) fails
//! 16 changed executor fails (last-responsible-moment re-resolution)
//! 17 pre-visibility crash exposes zero dispatchable members
//! 18 post-commit crash recovers all members (durable outcomes)
//! 19 no half-visible bundle at every injected failure point
//! 20 authority/1 frozen (commit_bundle + not_attempted refused; /1 flow green)
//! 21 execute@1 behaviour-identical output truth (parity with @2)
//! 22 together untouched by bundles (no protocol interaction)
//!
//! Proof 23 is this file green on Windows; proof 24 is this file green on
//! Linux (WSL). They are recorded in the mission receipt, not in-file.

use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use tethers_reference_host::authority_gate::{AuthorityGate, GateConfig};
use tethers_reference_host::bundle::BundleFailPoint;
use tethers_reference_host::gate_protocol::{AuthorityResponse, ResponseStatus};

const BUNDLE_ALLOW_DIGEST: &str =
    "sha256:257476fa6b84e91699260428ea0fd85087d357b8127764c93f669ff75c27b06d";
const BUNDLE_MANIFEST: &str =
    include_str!("../fixtures/capability-manifests/fixture-bundle-standing-allow.json");

const BUNDLE_TETHER: &str = r#"tether "Phase A bundle scenario"

anchor
    coding.task_completed

when
    project.type is "software"
    and task.changed_files greater_than 0

do
    fixture.bundle
        message: anchor.task
        path: anchor.path
        composition_digest: anchor.composition
"#;

/// Engine binary for Core-backed PREPARE. Same resolution as the R2 suite:
/// verified engine from the environment, else the current-checkout build.
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
        "Tethers Core engine not found. Set TETHERS_VERIFIED_ENGINE \
         or run scripts/prepare-current-engine.ps1."
    );
}

struct BundleWorkspace {
    root: PathBuf,
    config: PathBuf,
    trail: PathBuf,
    host_data: PathBuf,
    engine: PathBuf,
}

impl BundleWorkspace {
    fn new(policy_decision: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "tethers-bundle-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4().simple()
        ));
        fs::create_dir_all(root.join("tethers")).unwrap();
        fs::create_dir_all(root.join("manifests")).unwrap();
        fs::create_dir_all(root.join("host-data")).unwrap();

        fs::write(root.join("tethers/bundle.tether"), BUNDLE_TETHER).unwrap();
        fs::write(
            root.join("manifests/fixture-bundle-standing-allow.json"),
            BUNDLE_MANIFEST,
        )
        .unwrap();

        let workspace = Self {
            config: root.join("runtime.json"),
            trail: root.join("trail.jsonl"),
            host_data: root.join("host-data"),
            engine: engine_path(),
            root,
        };
        workspace.write_config(policy_decision);
        workspace.provision_replay();
        workspace
    }

    fn write_config(&self, policy_decision: &str) {
        let config = json!({
            "format_version": "0.1",
            "tether_set": {
                "id": "bundle.authority-gate",
                "version": "1",
                "tethers": [
                    {
                        "id": "bundle-complete",
                        "version": "1",
                        "source_path": "tethers/bundle.tether",
                        "core_environment": {
                            "program_id": "program.bundle.complete",
                            "core_version": "1",
                            "capabilities": [
                                {
                                    "source_name": "fixture.bundle",
                                    "capability_id": "cap.semantic.fixture-bundle",
                                    "contract_digest": "CORE-CONTRACT-BUNDLE",
                                    "runtime_name": "fixture.bundle"
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
                        "name": "fixture.bundle",
                        "version": 1,
                        "reason": "Phase A bundle acceptance fixture"
                    }
                ]
            },
            "providers": [
                {
                    "id": "tethers-host-fixture",
                    "display_name": "Tethers Host Fixture",
                    "transport": {
                        "kind": "stdio",
                        "command": "pwsh.exe",
                        "args": ["-NoProfile", "-File", "providers/fixture.ps1"],
                        "protocol_version": "2025-11-25"
                    },
                    "capabilities": [
                        {
                            "name": "fixture.bundle",
                            "version": 1,
                            "manifest_path": "manifests/fixture-bundle-standing-allow.json",
                            "pinned_digest": BUNDLE_ALLOW_DIGEST,
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
                        "name": "fixture.bundle",
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

    /// Core-authoritative PREPARE payload for one bundle-stage evaluation.
    fn prepare_payload(
        &self,
        evaluation_id: &str,
        event_id: &str,
        task: &str,
        path: &str,
        composition: &str,
    ) -> Value {
        json!({
            "action_id": "action_1",
            "evaluation_id": evaluation_id,
            "tether": { "id": "bundle-complete", "version": "1" },
            "event": {
                "id": event_id,
                "name": "coding.task_completed",
                "data": {
                    "project": "omen-shell",
                    "task": task,
                    "path": path,
                    "composition": composition
                }
            },
            "facts": { "project.type": "software", "task.changed_files": 3 }
        })
    }

    fn bundle_dir(&self) -> PathBuf {
        self.host_data.join("bundles")
    }

    fn committed_files(&self) -> Vec<PathBuf> {
        list_suffix(&self.bundle_dir(), ".committed.json")
    }

    fn committing_files(&self) -> Vec<PathBuf> {
        list_suffix(&self.bundle_dir(), ".committing.json")
    }
}

impl Drop for BundleWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn list_suffix(dir: &Path, suffix: &str) -> Vec<PathBuf> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return Vec::new(),
    };
    entries
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(suffix))
        })
        .collect()
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

fn call2(
    gate: &mut AuthorityGate,
    request_id: &str,
    operation: &str,
    payload: Value,
) -> AuthorityResponse {
    let map = payload.as_object().expect("payload object").clone();
    gate.handle_v2(operation, request_id, &map)
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

/// One bundle-stage evaluation to PREPARE. Keeps the helper arity small.
struct MemberSpec {
    request: &'static str,
    evaluation_id: String,
    event_id: String,
    task: &'static str,
    path: &'static str,
    composition: String,
}

impl MemberSpec {
    fn new(
        request: &'static str,
        evaluation_id: impl Into<String>,
        event_id: impl Into<String>,
        task: &'static str,
        path: &'static str,
        composition: impl Into<String>,
    ) -> Self {
        Self {
            request,
            evaluation_id: evaluation_id.into(),
            event_id: event_id.into(),
            task,
            path,
            composition: composition.into(),
        }
    }
}

fn prepare_member(
    gate: &mut AuthorityGate,
    workspace: &BundleWorkspace,
    spec: MemberSpec,
) -> Value {
    ok(&call(
        gate,
        spec.request,
        "prepare",
        workspace.prepare_payload(
            &spec.evaluation_id,
            &spec.event_id,
            spec.task,
            spec.path,
            &spec.composition,
        ),
    ))
    .clone()
}

fn commit_bundle(
    gate: &mut AuthorityGate,
    request: &str,
    prepared_ids: &[&str],
    approvals: Option<Value>,
) -> AuthorityResponse {
    let mut payload = json!({ "prepared_ids": prepared_ids });
    if let Some(approvals) = approvals {
        payload["approvals"] = approvals;
    }
    call2(gate, request, "commit_bundle", payload)
}

fn commit_bundle_ok(
    gate: &mut AuthorityGate,
    request: &str,
    prepared_ids: &[&str],
    approvals: Option<Value>,
) -> Value {
    ok(&commit_bundle(gate, request, prepared_ids, approvals)).clone()
}

fn report_succeeded(
    gate: &mut AuthorityGate,
    request: &str,
    execution_id: &str,
    external_id: &str,
) -> Value {
    ok(&call2(
        gate,
        request,
        "outcome",
        json!({
            "execution_id": execution_id,
            "classification": "succeeded",
            "attempted": true,
            "external_execution_identity": external_id,
            "result": {"echo": "bundle-ok"},
            "evidence": "sha256:evidence-fixture"
        }),
    ))
    .clone()
}

fn approvals_map(pairs: Vec<(String, Value)>) -> Value {
    let mut map = serde_json::Map::new();
    for (key, value) in pairs {
        map.insert(key, value);
    }
    Value::Object(map)
}

// ---------------------------------------------------------------------------
// Proof 1: sequential PREPARE admits zero provider invocation.
// ---------------------------------------------------------------------------

#[test]
fn proof01_sequential_prepare_zero_provider_invocation() {
    let workspace = BundleWorkspace::new("allow");
    let mut gate = workspace.gate();

    let first = prepare_member(
        &mut gate,
        &workspace,
        MemberSpec::new(
            "p1",
            "eval_b01_1",
            "evt_b01_1",
            "BK-1",
            "projects/bundle-one",
            "comp-proof-01",
        ),
    );
    assert_eq!(first["decision"], "allow_prepared");
    assert_eq!(first["authorizes_dispatch"], false);
    assert_eq!(first["provider_invocations"], 0);

    let second = prepare_member(
        &mut gate,
        &workspace,
        MemberSpec::new(
            "p2",
            "eval_b01_2",
            "evt_b01_2",
            "BK-2",
            "projects/bundle-two",
            "comp-proof-01",
        ),
    );
    assert_eq!(second["decision"], "allow_prepared");
    assert_eq!(second["authorizes_dispatch"], false);
    assert_eq!(second["provider_invocations"], 0);

    assert_ne!(first["prepared_id"], second["prepared_id"]);
    assert_eq!(gate.provider_invocations(), 0);
    assert!(workspace.committed_files().is_empty());
    assert!(workspace.committing_files().is_empty());
}

// ---------------------------------------------------------------------------
// Proof 2: deny-C commit_bundle admits zero durable authority.
// ---------------------------------------------------------------------------

#[test]
fn proof02_deny_commit_bundle_zero_admission() {
    let workspace = BundleWorkspace::new("deny");
    let mut gate = workspace.gate();

    let first = prepare_member(
        &mut gate,
        &workspace,
        MemberSpec::new(
            "p1",
            "eval_b02_1",
            "evt_b02_1",
            "BK-1",
            "projects/bundle-deny-a",
            "comp-proof-02",
        ),
    );
    assert_eq!(first["decision"], "deny");
    let second = prepare_member(
        &mut gate,
        &workspace,
        MemberSpec::new(
            "p2",
            "eval_b02_2",
            "evt_b02_2",
            "BK-2",
            "projects/bundle-deny-b",
            "comp-proof-02",
        ),
    );
    assert_eq!(second["decision"], "deny");

    let commit = commit_bundle(
        &mut gate,
        "c1",
        &[
            first["prepared_id"].as_str().unwrap(),
            second["prepared_id"].as_str().unwrap(),
        ],
        None,
    );
    assert_eq!(err(&commit), "commit_bundle.deny");
    assert_eq!(gate.provider_invocations(), 0);
    assert!(workspace.committed_files().is_empty());
    assert!(workspace.committing_files().is_empty());
}

// ---------------------------------------------------------------------------
// Proof 3: unavailable-C commit_bundle admits zero durable authority.
// ---------------------------------------------------------------------------

#[test]
fn proof03_unavailable_commit_bundle_zero_admission() {
    let workspace = BundleWorkspace::new("allow");
    let mut gate = workspace.gate();

    // Empty live provider observations make current authority unavailable.
    let mut payload_a = workspace.prepare_payload(
        "eval_b03_1",
        "evt_b03_1",
        "BK-1",
        "projects/bundle-un-a",
        "comp-proof-03",
    );
    payload_a["observations"] = json!({ "available_provider_identities": [] });
    let first = ok(&call(&mut gate, "p1", "prepare", payload_a)).clone();
    assert_eq!(first["decision"], "unavailable");

    let mut payload_b = workspace.prepare_payload(
        "eval_b03_2",
        "evt_b03_2",
        "BK-2",
        "projects/bundle-un-b",
        "comp-proof-03",
    );
    payload_b["observations"] = json!({ "available_provider_identities": [] });
    let second = ok(&call(&mut gate, "p2", "prepare", payload_b)).clone();
    assert_eq!(second["decision"], "unavailable");

    let commit = commit_bundle(
        &mut gate,
        "c1",
        &[
            first["prepared_id"].as_str().unwrap(),
            second["prepared_id"].as_str().unwrap(),
        ],
        None,
    );
    assert_eq!(err(&commit), "commit_bundle.unavailable");
    assert_eq!(gate.provider_invocations(), 0);
    assert!(workspace.committed_files().is_empty());
    assert!(workspace.committing_files().is_empty());
}

// ---------------------------------------------------------------------------
// Proof 4: replay-conflict-B commit_bundle admits zero durable authority.
// ---------------------------------------------------------------------------

#[test]
fn proof04_replay_conflict_zero_admission() {
    use tethers_reference_host::replay::{ExecutionBinding, LogicalExecutionKey};
    use tethers_reference_host::replay_runtime::{FileReplayAuthority, ReplayAuthority};

    let workspace = BundleWorkspace::new("allow");
    let mut gate = workspace.gate();

    let first = prepare_member(
        &mut gate,
        &workspace,
        MemberSpec::new(
            "p1",
            "eval_b04_1",
            "evt_b04_1",
            "BK-1",
            "projects/bundle-rp-a",
            "comp-proof-04",
        ),
    );
    let second = prepare_member(
        &mut gate,
        &workspace,
        MemberSpec::new(
            "p2",
            "eval_b04_2",
            "evt_b04_2",
            "BK-2",
            "projects/bundle-rp-b",
            "comp-proof-04",
        ),
    );

    // A foreign durable claim already occupies B's logical key with a
    // binding that disagrees with bundle material (no bundle identity,
    // different argument digest).
    let key =
        LogicalExecutionKey::derive("evt_b04_2", "eval_b04_2", "action_1").expect("logical key");
    let binding = ExecutionBinding {
        evaluation_id: "eval_b04_2".to_owned(),
        action_id: "action_1".to_owned(),
        capability_name: "fixture.bundle".to_owned(),
        capability_version: 1,
        manifest_digest: BUNDLE_ALLOW_DIGEST.to_owned(),
        provider_identity: "tethers-host-fixture".to_owned(),
        argument_digest: "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
            .to_owned(),
        bundle_id: None,
    };
    let authority = FileReplayAuthority::new(Some(&workspace.host_data));
    let guard = authority
        .admit(&key, &binding)
        .expect("admit foreign claim");
    assert!(guard.is_fresh());
    drop(guard);

    let commit = commit_bundle(
        &mut gate,
        "c1",
        &[
            first["prepared_id"].as_str().unwrap(),
            second["prepared_id"].as_str().unwrap(),
        ],
        None,
    );
    let code = err(&commit);
    assert!(
        code == "commit_bundle.replay_blocked",
        "expected replay_blocked, got {code}"
    );
    assert_eq!(gate.provider_invocations(), 0);
    assert!(workspace.committed_files().is_empty());
    assert!(workspace.committing_files().is_empty());
}

// ---------------------------------------------------------------------------
// Proof 5: an invalid approval consumes nothing (retry succeeds).
// ---------------------------------------------------------------------------

#[test]
fn proof05_invalid_approval_consumes_nothing() {
    let workspace = BundleWorkspace::new("ask");
    let mut gate = workspace.gate();

    let first = prepare_member(
        &mut gate,
        &workspace,
        MemberSpec::new(
            "p1",
            "eval_b05_1",
            "evt_b05_1",
            "BK-1",
            "projects/bundle-ask-a",
            "comp-proof-05",
        ),
    );
    assert_eq!(first["decision"], "ask");
    let second = prepare_member(
        &mut gate,
        &workspace,
        MemberSpec::new(
            "p2",
            "eval_b05_2",
            "evt_b05_2",
            "BK-2",
            "projects/bundle-ask-b",
            "comp-proof-05",
        ),
    );
    assert_eq!(second["decision"], "ask");
    let approval_a = first["approval"]["approval_id"]
        .as_str()
        .unwrap()
        .to_owned();
    let approval_b = second["approval"]["approval_id"]
        .as_str()
        .unwrap()
        .to_owned();
    let prep_a = first["prepared_id"].as_str().unwrap().to_owned();
    let prep_b = second["prepared_id"].as_str().unwrap().to_owned();

    for (approval_id, request) in [(&approval_a, "a1"), (&approval_b, "a2")] {
        let decision = ok(&call(
            &mut gate,
            request,
            "approval_decision",
            json!({ "approval_id": approval_id, "decision": "approve" }),
        ))
        .clone();
        assert_eq!(decision["state"], "approved");
    }

    // B's approval identity is forged: the whole bundle must fail closed
    // with zero durable admission and zero consumption of A's approval.
    // Approval overrides are keyed by prepared identity.
    let bad = commit_bundle(
        &mut gate,
        "c1",
        &[&prep_a, &prep_b],
        Some(approvals_map(vec![
            (prep_a.clone(), json!(approval_a)),
            (prep_b.clone(), json!("approval-0000-bogus")),
        ])),
    );
    let code = err(&bad);
    assert!(
        code.starts_with("commit_bundle."),
        "expected a commit_bundle refusal, got {code}"
    );
    assert!(workspace.committed_files().is_empty());
    assert!(workspace.committing_files().is_empty());

    // Retry with both exact approvals succeeds: nothing was consumed.
    let dispatch = commit_bundle_ok(&mut gate, "c2", &[&prep_a, &prep_b], None);
    assert_eq!(dispatch["members"].as_array().unwrap().len(), 2);
    assert_eq!(dispatch["members"][0]["approval_consumed"], true);
    assert_eq!(dispatch["members"][1]["approval_consumed"], true);
    assert_eq!(gate.provider_invocations(), 0);
}

// ---------------------------------------------------------------------------
// Proofs 6 + 7: success returns all member execution identities under one
// bundle identity with one committed marker.
// ---------------------------------------------------------------------------

#[test]
fn proof06_and_07_success_returns_identities_under_one_bundle() {
    let workspace = BundleWorkspace::new("allow");
    let mut gate = workspace.gate();

    let first = prepare_member(
        &mut gate,
        &workspace,
        MemberSpec::new(
            "p1",
            "eval_b06_1",
            "evt_b06_1",
            "BK-1",
            "projects/bundle-ok-a",
            "comp-proof-06",
        ),
    );
    let second = prepare_member(
        &mut gate,
        &workspace,
        MemberSpec::new(
            "p2",
            "eval_b06_2",
            "evt_b06_2",
            "BK-2",
            "projects/bundle-ok-b",
            "comp-proof-06",
        ),
    );

    let dispatch = commit_bundle_ok(
        &mut gate,
        "c1",
        &[
            first["prepared_id"].as_str().unwrap(),
            second["prepared_id"].as_str().unwrap(),
        ],
        None,
    );
    assert_eq!(dispatch["schema"], "tethers.bundle_dispatch/2");
    assert_eq!(dispatch["composition_digest"], "comp-proof-06");
    assert_eq!(dispatch["authority_protocol"], "tethers.authority/2");
    assert_eq!(dispatch["authorizes_physical_execution_by_tethers"], false);
    assert_eq!(dispatch["host_must_report_outcome"], true);
    assert_eq!(dispatch["provider_invocations"], 0);

    let bundle_id = dispatch["bundle_id"].as_str().unwrap().to_owned();
    assert!(bundle_id.starts_with("bundle_"), "got {bundle_id}");

    let members = dispatch["members"].as_array().unwrap();
    assert_eq!(members.len(), 2);
    let exec_a = members[0]["execution_id"].as_str().unwrap().to_owned();
    let exec_b = members[1]["execution_id"].as_str().unwrap().to_owned();
    assert!(!exec_a.is_empty() && !exec_b.is_empty() && exec_a != exec_b);
    assert_eq!(members[0]["evaluation_id"], "eval_b06_1");
    assert_eq!(members[1]["evaluation_id"], "eval_b06_2");
    assert_eq!(members[0]["prepared_id"], first["prepared_id"]);
    assert_eq!(members[1]["prepared_id"], second["prepared_id"]);

    // Exactly one committed marker. The committing file remains as history
    // (status summaries collapse it under the committed state).
    let committed = workspace.committed_files();
    assert_eq!(committed.len(), 1, "expected one committed marker");
    assert_eq!(
        workspace.committing_files().len(),
        1,
        "committing history retained"
    );
    let record: Value =
        serde_json::from_slice(&fs::read(&committed[0]).unwrap()).expect("bundle record");
    assert_eq!(record["bundle_id"], bundle_id);
    assert_eq!(record["composition_digest"], "comp-proof-06");
    assert_eq!(record["members"].as_array().unwrap().len(), 2);
    assert_eq!(gate.provider_invocations(), 0);
}

// ---------------------------------------------------------------------------
// Proof 8: no post-bundle individual commit (session + restart).
// ---------------------------------------------------------------------------

#[test]
fn proof08_no_post_bundle_individual_commit() {
    let workspace = BundleWorkspace::new("allow");
    let mut gate = workspace.gate();

    let first = prepare_member(
        &mut gate,
        &workspace,
        MemberSpec::new(
            "p1",
            "eval_b08_1",
            "evt_b08_1",
            "BK-1",
            "projects/bundle-ic-a",
            "comp-proof-08",
        ),
    );
    let second = prepare_member(
        &mut gate,
        &workspace,
        MemberSpec::new(
            "p2",
            "eval_b08_2",
            "evt_b08_2",
            "BK-2",
            "projects/bundle-ic-b",
            "comp-proof-08",
        ),
    );
    let prep_a = first["prepared_id"].as_str().unwrap().to_owned();
    commit_bundle_ok(
        &mut gate,
        "c1",
        &[&prep_a, second["prepared_id"].as_str().unwrap()],
        None,
    );

    // Same session: the prepared identity is spent.
    let single = call(&mut gate, "c2", "commit", json!({ "prepared_id": prep_a }));
    assert_eq!(err(&single), "commit.already_committed");

    // After restart the ledger still refuses the individual commit with
    // the explicit bundle-membership code.
    drop(gate);
    let mut gate = workspace.gate();
    let re = prepare_member(
        &mut gate,
        &workspace,
        MemberSpec::new(
            "p3",
            "eval_b08_1",
            "evt_b08_1",
            "BK-1",
            "projects/bundle-ic-a",
            "comp-proof-08",
        ),
    );
    assert_eq!(
        re["prepared_id"], prep_a,
        "prepared identity is deterministic"
    );
    let single = call(&mut gate, "c3", "commit", json!({ "prepared_id": prep_a }));
    assert_eq!(err(&single), "commit.bundle_member");
    assert_eq!(workspace.committed_files().len(), 1);
    assert_eq!(gate.provider_invocations(), 0);
}

// ---------------------------------------------------------------------------
// Proof 9: no second bundle commit (session + restart).
// ---------------------------------------------------------------------------

#[test]
fn proof09_no_second_bundle_commit() {
    let workspace = BundleWorkspace::new("allow");
    let mut gate = workspace.gate();

    let first = prepare_member(
        &mut gate,
        &workspace,
        MemberSpec::new(
            "p1",
            "eval_b09_1",
            "evt_b09_1",
            "BK-1",
            "projects/bundle-2x-a",
            "comp-proof-09",
        ),
    );
    let second = prepare_member(
        &mut gate,
        &workspace,
        MemberSpec::new(
            "p2",
            "eval_b09_2",
            "evt_b09_2",
            "BK-2",
            "projects/bundle-2x-b",
            "comp-proof-09",
        ),
    );
    let ids = [
        first["prepared_id"].as_str().unwrap().to_owned(),
        second["prepared_id"].as_str().unwrap().to_owned(),
    ];
    commit_bundle_ok(&mut gate, "c1", &[&ids[0], &ids[1]], None);

    let again = commit_bundle(&mut gate, "c2", &[&ids[0], &ids[1]], None);
    assert_eq!(err(&again), "commit_bundle.already_committed");
    assert_eq!(workspace.committed_files().len(), 1);

    // After restart the committed bundle still owns the members: the
    // committed-marker check fires before any member scan.
    drop(gate);
    let mut gate = workspace.gate();
    prepare_member(
        &mut gate,
        &workspace,
        MemberSpec::new(
            "p3",
            "eval_b09_1",
            "evt_b09_1",
            "BK-1",
            "projects/bundle-2x-a",
            "comp-proof-09",
        ),
    );
    prepare_member(
        &mut gate,
        &workspace,
        MemberSpec::new(
            "p4",
            "eval_b09_2",
            "evt_b09_2",
            "BK-2",
            "projects/bundle-2x-b",
            "comp-proof-09",
        ),
    );
    let replay = commit_bundle(&mut gate, "c3", &[&ids[0], &ids[1]], None);
    assert_eq!(err(&replay), "commit_bundle.replay_blocked");
    assert_eq!(workspace.committed_files().len(), 1);
    assert_eq!(gate.provider_invocations(), 0);
}

// ---------------------------------------------------------------------------
// Proof 10: no replay after terminal outcomes.
// ---------------------------------------------------------------------------

#[test]
fn proof10_no_replay_after_terminal_outcomes() {
    let workspace = BundleWorkspace::new("allow");
    let mut gate = workspace.gate();

    let first = prepare_member(
        &mut gate,
        &workspace,
        MemberSpec::new(
            "p1",
            "eval_b10_1",
            "evt_b10_1",
            "BK-1",
            "projects/bundle-rp-a",
            "comp-proof-10",
        ),
    );
    let second = prepare_member(
        &mut gate,
        &workspace,
        MemberSpec::new(
            "p2",
            "eval_b10_2",
            "evt_b10_2",
            "BK-2",
            "projects/bundle-rp-b",
            "comp-proof-10",
        ),
    );
    let ids = [
        first["prepared_id"].as_str().unwrap().to_owned(),
        second["prepared_id"].as_str().unwrap().to_owned(),
    ];
    let dispatch = commit_bundle_ok(&mut gate, "c1", &[&ids[0], &ids[1]], None);
    for (member, request, external) in [
        (&dispatch["members"][0], "o1", "external-b10-a"),
        (&dispatch["members"][1], "o2", "external-b10-b"),
    ] {
        let outcome = report_succeeded(
            &mut gate,
            request,
            member["execution_id"].as_str().unwrap(),
            external,
        );
        assert_eq!(outcome["status"], "succeeded");
    }

    // After restart, re-admission of either member is refused: the
    // committed bundle still owns both prepared identities.
    drop(gate);
    let mut gate = workspace.gate();
    prepare_member(
        &mut gate,
        &workspace,
        MemberSpec::new(
            "p3",
            "eval_b10_1",
            "evt_b10_1",
            "BK-1",
            "projects/bundle-rp-a",
            "comp-proof-10",
        ),
    );
    prepare_member(
        &mut gate,
        &workspace,
        MemberSpec::new(
            "p4",
            "eval_b10_2",
            "evt_b10_2",
            "BK-2",
            "projects/bundle-rp-b",
            "comp-proof-10",
        ),
    );
    let replay = commit_bundle(&mut gate, "c2", &[&ids[0], &ids[1]], None);
    assert_eq!(err(&replay), "commit_bundle.replay_blocked");
    assert_eq!(workspace.committed_files().len(), 1);
    assert_eq!(gate.provider_invocations(), 0);
}

// ---------------------------------------------------------------------------
// Proof 11: reorder fails, identical retry resumes.
// ---------------------------------------------------------------------------

#[test]
fn proof11_reorder_fails() {
    let workspace = BundleWorkspace::new("allow");
    let mut gate = workspace.gate();

    let first = prepare_member(
        &mut gate,
        &workspace,
        MemberSpec::new(
            "p1",
            "eval_b11_1",
            "evt_b11_1",
            "BK-1",
            "projects/bundle-ro-a",
            "comp-proof-11",
        ),
    );
    let second = prepare_member(
        &mut gate,
        &workspace,
        MemberSpec::new(
            "p2",
            "eval_b11_2",
            "evt_b11_2",
            "BK-2",
            "projects/bundle-ro-b",
            "comp-proof-11",
        ),
    );
    let id_a = first["prepared_id"].as_str().unwrap().to_owned();
    let id_b = second["prepared_id"].as_str().unwrap().to_owned();

    // Crash after the bundle intent: members are admitted under [A, B].
    gate.set_bundle_fail_point(Some(BundleFailPoint::AfterBundleIntent));
    let crashed = commit_bundle(&mut gate, "c1", &[&id_a, &id_b], None);
    assert_eq!(err(&crashed), "commit_bundle.crash_simulated");
    gate.set_bundle_fail_point(None);
    assert_eq!(workspace.committing_files().len(), 1);
    assert!(workspace.committed_files().is_empty());

    // Reordered retry collides with the unfinished bundle's member claim.
    let reordered = commit_bundle(&mut gate, "c2", &[&id_b, &id_a], None);
    assert_eq!(err(&reordered), "commit_bundle.bundle_member");
    assert!(workspace.committed_files().is_empty());

    // The identical retry instead resumes the same unfinished bundle.
    let resumed = commit_bundle_ok(&mut gate, "c3", &[&id_a, &id_b], None);
    assert_eq!(resumed["members"].as_array().unwrap().len(), 2);
    assert_eq!(workspace.committed_files().len(), 1);
    assert_eq!(gate.provider_invocations(), 0);
}

// ---------------------------------------------------------------------------
// Proof 12: remove (fewer than two members) fails at the frame boundary.
// ---------------------------------------------------------------------------

#[test]
fn proof12_remove_fails() {
    let workspace = BundleWorkspace::new("allow");
    let mut gate = workspace.gate();

    let first = prepare_member(
        &mut gate,
        &workspace,
        MemberSpec::new(
            "p1",
            "eval_b12_1",
            "evt_b12_1",
            "BK-1",
            "projects/bundle-rm-a",
            "comp-proof-12",
        ),
    );
    let id_a = first["prepared_id"].as_str().unwrap().to_owned();

    let removed = commit_bundle(&mut gate, "c1", &[&id_a], None);
    // Frame-boundary refusal: fewer than MIN_BUNDLE_MEMBERS members.
    // (Frame-layer codes surface as frame.payload_invalid; the member_count
    // reason is preserved in the message.)
    assert_eq!(err(&removed), "frame.payload_invalid");
    assert!(workspace.committed_files().is_empty());
    assert!(workspace.committing_files().is_empty());
    assert_eq!(gate.provider_invocations(), 0);
}

// ---------------------------------------------------------------------------
// Proof 13: add (a third member onto a committed bundle) fails.
// ---------------------------------------------------------------------------

#[test]
fn proof13_add_fails() {
    let workspace = BundleWorkspace::new("allow");
    let mut gate = workspace.gate();

    let first = prepare_member(
        &mut gate,
        &workspace,
        MemberSpec::new(
            "p1",
            "eval_b13_1",
            "evt_b13_1",
            "BK-1",
            "projects/bundle-ad-a",
            "comp-proof-13",
        ),
    );
    let second = prepare_member(
        &mut gate,
        &workspace,
        MemberSpec::new(
            "p2",
            "eval_b13_2",
            "evt_b13_2",
            "BK-2",
            "projects/bundle-ad-b",
            "comp-proof-13",
        ),
    );
    let third = prepare_member(
        &mut gate,
        &workspace,
        MemberSpec::new(
            "p3",
            "eval_b13_3",
            "evt_b13_3",
            "BK-3",
            "projects/bundle-ad-c",
            "comp-proof-13",
        ),
    );
    let (id_a, id_b, id_c) = (
        first["prepared_id"].as_str().unwrap().to_owned(),
        second["prepared_id"].as_str().unwrap().to_owned(),
        third["prepared_id"].as_str().unwrap().to_owned(),
    );
    commit_bundle_ok(&mut gate, "c1", &[&id_a, &id_b], None);

    let added = commit_bundle(&mut gate, "c2", &[&id_a, &id_b, &id_c], None);
    let code = err(&added);
    assert!(
        code == "commit_bundle.already_committed" || code == "commit_bundle.bundle_member",
        "expected a spent-member refusal, got {code}"
    );
    assert_eq!(workspace.committed_files().len(), 1);
    assert_eq!(gate.provider_invocations(), 0);
}

// ---------------------------------------------------------------------------
// Proof 14: changed composition fails (members must share one identity).
// ---------------------------------------------------------------------------

#[test]
fn proof14_changed_composition_fails() {
    let workspace = BundleWorkspace::new("allow");
    let mut gate = workspace.gate();

    let first = prepare_member(
        &mut gate,
        &workspace,
        MemberSpec::new(
            "p1",
            "eval_b14_1",
            "evt_b14_1",
            "BK-1",
            "projects/bundle-cc-a",
            "comp-proof-14-a",
        ),
    );
    let second = prepare_member(
        &mut gate,
        &workspace,
        MemberSpec::new(
            "p2",
            "eval_b14_2",
            "evt_b14_2",
            "BK-2",
            "projects/bundle-cc-b",
            "comp-proof-14-b",
        ),
    );

    let commit = commit_bundle(
        &mut gate,
        "c1",
        &[
            first["prepared_id"].as_str().unwrap(),
            second["prepared_id"].as_str().unwrap(),
        ],
        None,
    );
    assert_eq!(err(&commit), "commit_bundle.composition_mismatch");
    assert!(workspace.committed_files().is_empty());
    assert!(workspace.committing_files().is_empty());
    assert_eq!(gate.provider_invocations(), 0);
}

// ---------------------------------------------------------------------------
// Proof 15: changed args fail (an approval bound to other args is exact).
// ---------------------------------------------------------------------------

#[test]
fn proof15_changed_args_fail() {
    let workspace = BundleWorkspace::new("ask");
    let mut gate = workspace.gate();

    // A1 and A2 are the same logical stage with different arguments.
    let a1 = prepare_member(
        &mut gate,
        &workspace,
        MemberSpec::new(
            "p1",
            "eval_b15_1",
            "evt_b15_1",
            "BK-1",
            "projects/bundle-ca-a",
            "comp-proof-15",
        ),
    );
    let a2 = prepare_member(
        &mut gate,
        &workspace,
        MemberSpec::new(
            "p2",
            "eval_b15_2",
            "evt_b15_2",
            "BK-1-changed",
            "projects/bundle-ca-a",
            "comp-proof-15",
        ),
    );
    let b = prepare_member(
        &mut gate,
        &workspace,
        MemberSpec::new(
            "p3",
            "eval_b15_3",
            "evt_b15_3",
            "BK-2",
            "projects/bundle-ca-b",
            "comp-proof-15",
        ),
    );
    let (appr_a1, appr_a2, appr_b) = (
        a1["approval"]["approval_id"].as_str().unwrap().to_owned(),
        a2["approval"]["approval_id"].as_str().unwrap().to_owned(),
        b["approval"]["approval_id"].as_str().unwrap().to_owned(),
    );
    for (approval_id, request) in [(&appr_a1, "a1"), (&appr_a2, "a2"), (&appr_b, "a3")] {
        let decision = ok(&call(
            &mut gate,
            request,
            "approval_decision",
            json!({ "approval_id": approval_id, "decision": "approve" }),
        ))
        .clone();
        assert_eq!(decision["state"], "approved");
    }

    // A2 presented with A1's approval: argument digests disagree exactly.
    // Approval overrides are keyed by prepared identity.
    let prep_a2 = a2["prepared_id"].as_str().unwrap().to_owned();
    let prep_b = b["prepared_id"].as_str().unwrap().to_owned();
    let changed = commit_bundle(
        &mut gate,
        "c1",
        &[&prep_a2, &prep_b],
        Some(approvals_map(vec![
            (prep_a2.clone(), json!(appr_a1)),
            (prep_b.clone(), json!(appr_b)),
        ])),
    );
    let code = err(&changed);
    assert!(
        code.starts_with("commit_bundle."),
        "expected a commit_bundle refusal, got {code}"
    );
    assert!(workspace.committed_files().is_empty());
    assert!(workspace.committing_files().is_empty());

    // Control: exact approvals for the same members succeed.
    let dispatch = commit_bundle_ok(&mut gate, "c2", &[&prep_a2, &prep_b], None);
    assert_eq!(dispatch["members"].as_array().unwrap().len(), 2);
    assert_eq!(gate.provider_invocations(), 0);
}

// ---------------------------------------------------------------------------
// Proof 16: changed executor fails (last-responsible-moment re-resolution).
// ---------------------------------------------------------------------------

#[test]
fn proof16_changed_executor_fails() {
    let workspace = BundleWorkspace::new("allow");
    let mut gate = workspace.gate();

    let first = prepare_member(
        &mut gate,
        &workspace,
        MemberSpec::new(
            "p1",
            "eval_b16_1",
            "evt_b16_1",
            "BK-1",
            "projects/bundle-ce-a",
            "comp-proof-16",
        ),
    );
    let second = prepare_member(
        &mut gate,
        &workspace,
        MemberSpec::new(
            "p2",
            "eval_b16_2",
            "evt_b16_2",
            "BK-2",
            "projects/bundle-ce-b",
            "comp-proof-16",
        ),
    );

    // Rotate the trusted executor identity between PREPARE and COMMIT.
    rotate_executor_identity(&workspace, "tethers-host-fixture-2");

    let commit = commit_bundle(
        &mut gate,
        "c1",
        &[
            first["prepared_id"].as_str().unwrap(),
            second["prepared_id"].as_str().unwrap(),
        ],
        None,
    );
    let code = err(&commit);
    assert!(
        code.starts_with("commit_bundle."),
        "expected a commit_bundle refusal, got {code}"
    );
    assert!(workspace.committed_files().is_empty());
    assert!(workspace.committing_files().is_empty());
    assert_eq!(gate.provider_invocations(), 0);
}

/// Rewrite the fixture manifest + runtime config so the trusted executor
/// identity is `new_identity`, keeping every digest pin consistent.
fn rotate_executor_identity(workspace: &BundleWorkspace, new_identity: &str) {
    let manifest_path = workspace.root.join("manifests/fixture-bundle-rotated.json");
    let mut manifest: Value =
        serde_json::from_str(BUNDLE_MANIFEST).expect("fixture manifest parses");
    manifest["provider"]["identity"] = json!(new_identity);
    manifest["binding"]["executor_identity"] = json!(new_identity);
    manifest.as_object_mut().unwrap().remove("digest");
    let without_digest = serde_json::to_string(&manifest).expect("manifest serialises");
    let (_, digest) = tethers_reference_host::manifest::canonicalize_and_digest(&without_digest)
        .expect("rotated manifest digests");
    manifest["digest"] = json!(digest);
    fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();

    let mut config: Value =
        serde_json::from_slice(&fs::read(&workspace.config).unwrap()).expect("config parses");
    config["providers"][0]["id"] = json!(new_identity);
    config["providers"][0]["capabilities"][0]["manifest_path"] =
        json!("manifests/fixture-bundle-rotated.json");
    config["providers"][0]["capabilities"][0]["pinned_digest"] = json!(digest);
    fs::write(
        &workspace.config,
        serde_json::to_vec_pretty(&config).unwrap(),
    )
    .unwrap();
}

// ---------------------------------------------------------------------------
// Proof 17: a pre-visibility crash exposes zero dispatchable members.
// ---------------------------------------------------------------------------

#[test]
fn proof17_pre_visibility_crash_exposes_zero() {
    let workspace = BundleWorkspace::new("allow");
    let mut gate = workspace.gate();

    let first = prepare_member(
        &mut gate,
        &workspace,
        MemberSpec::new(
            "p1",
            "eval_b17_1",
            "evt_b17_1",
            "BK-1",
            "projects/bundle-pv-a",
            "comp-proof-17",
        ),
    );
    let second = prepare_member(
        &mut gate,
        &workspace,
        MemberSpec::new(
            "p2",
            "eval_b17_2",
            "evt_b17_2",
            "BK-2",
            "projects/bundle-pv-b",
            "comp-proof-17",
        ),
    );
    let (id_a, id_b) = (
        first["prepared_id"].as_str().unwrap().to_owned(),
        second["prepared_id"].as_str().unwrap().to_owned(),
    );

    // Crash at the deepest pre-visibility point: everything durable is
    // written except the committed marker.
    gate.set_bundle_fail_point(Some(BundleFailPoint::BeforeCommittedMarker));
    let crashed = commit_bundle(&mut gate, "c1", &[&id_a, &id_b], None);
    assert_eq!(err(&crashed), "commit_bundle.crash_simulated");
    gate.set_bundle_fail_point(None);

    // Crash state is evidence, not authority: committing without committed.
    assert_eq!(workspace.committing_files().len(), 1);
    assert!(workspace.committed_files().is_empty());

    // Zero dispatchable members: neither member is individually committable
    // on a fresh gate, and nothing is unresolved for dispatch.
    drop(gate);
    let mut gate = workspace.gate();
    prepare_member(
        &mut gate,
        &workspace,
        MemberSpec::new(
            "p3",
            "eval_b17_1",
            "evt_b17_1",
            "BK-1",
            "projects/bundle-pv-a",
            "comp-proof-17",
        ),
    );
    let single = call(&mut gate, "c2", "commit", json!({ "prepared_id": id_a }));
    assert_eq!(err(&single), "commit.bundle_member");
    let status = ok(&call2(&mut gate, "s1", "status", json!({}))).clone();
    assert_eq!(status["provider_invocations"], 0);
    let bundles = status["bundles"].as_array().unwrap();
    assert!(
        bundles.iter().all(|entry| entry["state"] != "committed"),
        "no committed bundle may be visible"
    );
    assert_eq!(status["unresolved_commits"].as_array().unwrap().len(), 0);
}

// ---------------------------------------------------------------------------
// Proof 18: a post-commit crash recovers all members (durable outcomes).
// ---------------------------------------------------------------------------

#[test]
fn proof18_post_commit_crash_recovers_all() {
    let workspace = BundleWorkspace::new("allow");
    let mut gate = workspace.gate();

    let first = prepare_member(
        &mut gate,
        &workspace,
        MemberSpec::new(
            "p1",
            "eval_b18_1",
            "evt_b18_1",
            "BK-1",
            "projects/bundle-pc-a",
            "comp-proof-18",
        ),
    );
    let second = prepare_member(
        &mut gate,
        &workspace,
        MemberSpec::new(
            "p2",
            "eval_b18_2",
            "evt_b18_2",
            "BK-2",
            "projects/bundle-pc-b",
            "comp-proof-18",
        ),
    );
    let dispatch = commit_bundle_ok(
        &mut gate,
        "c1",
        &[
            first["prepared_id"].as_str().unwrap(),
            second["prepared_id"].as_str().unwrap(),
        ],
        None,
    );
    let exec_a = dispatch["members"][0]["execution_id"]
        .as_str()
        .unwrap()
        .to_owned();
    let exec_b = dispatch["members"][1]["execution_id"]
        .as_str()
        .unwrap()
        .to_owned();

    // Crash: the whole Gate session is gone; only durable truth remains.
    drop(gate);
    let mut gate = workspace.gate();

    // Both members accept terminal outcomes on the durable path.
    for (execution_id, request, external) in [
        (&exec_a, "o1", "external-b18-a"),
        (&exec_b, "o2", "external-b18-b"),
    ] {
        let outcome = ok(&call2(
            &mut gate,
            request,
            "outcome",
            json!({
                "execution_id": execution_id,
                "classification": "succeeded",
                "attempted": true,
                "external_execution_identity": external,
                "result": {"echo": "bundle-ok"},
                "evidence": "sha256:evidence-fixture"
            }),
        ))
        .clone();
        assert_eq!(outcome["status"], "succeeded");
        assert_eq!(outcome["trail_outcome_recorded"], true);
    }

    // The committed bundle with both recovered members is visible.
    let status = ok(&call2(&mut gate, "s1", "status", json!({}))).clone();
    let bundles = status["bundles"].as_array().unwrap();
    assert_eq!(bundles.len(), 1);
    assert_eq!(bundles[0]["state"], "committed");
    assert_eq!(bundles[0]["members"].as_array().unwrap().len(), 2);
    assert_eq!(gate.provider_invocations(), 0);
}

// ---------------------------------------------------------------------------
// Proof 19: no half-visible bundle at every injected failure point.
// ---------------------------------------------------------------------------

#[test]
fn proof19_no_half_visible_bundle_at_every_fail_point() {
    let fail_points = [
        BundleFailPoint::AfterBundleIntent,
        BundleFailPoint::AfterMemberIntent(0),
        BundleFailPoint::AfterMemberIntent(1),
        BundleFailPoint::AfterMemberArmed(0),
        BundleFailPoint::AfterMemberArmed(1),
        BundleFailPoint::AfterApprovalConsume,
        BundleFailPoint::BeforeCommittedMarker,
    ];
    for (index, fail_point) in fail_points.into_iter().enumerate() {
        let workspace = BundleWorkspace::new("allow");
        let mut gate = workspace.gate();
        let first = prepare_member(
            &mut gate,
            &workspace,
            MemberSpec::new(
                "p1",
                format!("eval_b19_{index}_a"),
                format!("evt_b19_{index}_a"),
                "BK-1",
                "projects/bundle-hv-a",
                format!("comp-proof-19-{index}"),
            ),
        );
        let second = prepare_member(
            &mut gate,
            &workspace,
            MemberSpec::new(
                "p2",
                format!("eval_b19_{index}_b"),
                format!("evt_b19_{index}_b"),
                "BK-2",
                "projects/bundle-hv-b",
                format!("comp-proof-19-{index}"),
            ),
        );
        let (id_a, id_b) = (
            first["prepared_id"].as_str().unwrap().to_owned(),
            second["prepared_id"].as_str().unwrap().to_owned(),
        );

        gate.set_bundle_fail_point(Some(fail_point));
        let crashed = commit_bundle(&mut gate, "c1", &[&id_a, &id_b], None);
        gate.set_bundle_fail_point(None);
        assert_eq!(
            err(&crashed),
            "commit_bundle.crash_simulated",
            "fail point {fail_point:?}"
        );

        // Never half-visible: no committed marker at any failure point,
        // and neither member is individually committable in-session.
        assert!(
            workspace.committed_files().is_empty(),
            "fail point {fail_point:?} left a committed marker"
        );
        for (id, request) in [(&id_a, "c2"), (&id_b, "c3")] {
            let single = call(&mut gate, request, "commit", json!({ "prepared_id": id }));
            assert_eq!(
                err(&single),
                "commit.bundle_member",
                "fail point {fail_point:?}: crashed-bundle members stay individually uncommittable"
            );
        }
        assert_eq!(gate.provider_invocations(), 0);
    }
}

// ---------------------------------------------------------------------------
// Proof 20: authority/1 is frozen (bundle surface refused; /1 flow green).
// ---------------------------------------------------------------------------

#[test]
fn proof20_authority_v1_frozen() {
    use tethers_reference_host::gate_protocol::parse_frame;

    let workspace = BundleWorkspace::new("allow");
    let mut gate = workspace.gate();

    // /1 hello carries no bundle feature.
    let hello = ok(&call(&mut gate, "h1", "hello", json!({}))).clone();
    assert_eq!(hello["protocol"], "tethers.authority/1");
    assert!(!hello["features"]
        .as_array()
        .unwrap()
        .iter()
        .any(|feature| feature == "commit_bundle"));

    // /2 hello advertises the additive operation.
    let hello2 = ok(&call2(&mut gate, "h2", "hello", json!({}))).clone();
    assert_eq!(hello2["protocol"], "tethers.authority/2");
    assert!(hello2["features"]
        .as_array()
        .unwrap()
        .iter()
        .any(|feature| feature == "commit_bundle"));

    // commit_bundle on /1 is an unknown operation, not a silent downgrade.
    let refused = call(
        &mut gate,
        "c0",
        "commit_bundle",
        json!({ "prepared_ids": ["prep_a", "prep_b"] }),
    );
    assert_eq!(err(&refused), "frame.unknown_operation");

    // The frame layer refuses commit_bundle on /1 wire frames too.
    let frame = parse_frame(
        r#"{"schema":"tethers.authority/1","request_id":"r1","operation":"commit_bundle","payload":{}}"#,
    );
    assert!(frame.is_err(), "expected /1 wire refusal, got {frame:?}");

    // The full /1 single-commit flow still works, including for the new
    // Host-bound capability, and /1 refuses the bundle-only outcome.
    let prepared = prepare_member(
        &mut gate,
        &workspace,
        MemberSpec::new(
            "p1",
            "eval_b20_1",
            "evt_b20_1",
            "BK-1",
            "projects/bundle-v1-a",
            "comp-proof-20",
        ),
    );
    let dispatch = ok(&call(
        &mut gate,
        "c1",
        "commit",
        json!({ "prepared_id": prepared["prepared_id"] }),
    ))
    .clone();
    assert_eq!(dispatch["schema"], "tethers.dispatch/1");
    let outcome = ok(&call(
        &mut gate,
        "o1",
        "outcome",
        json!({
            "execution_id": dispatch["execution_id"],
            "classification": "succeeded",
            "attempted": true,
            "external_execution_identity": "external-b20",
            "result": {"echo": "bundle-ok"},
            "evidence": "sha256:evidence-fixture"
        }),
    ))
    .clone();
    assert_eq!(outcome["status"], "succeeded");

    // /1 refuses the bundle-only outcome classification at the frame layer.
    let not_attempted = call(
        &mut gate,
        "o2",
        "outcome",
        json!({
            "execution_id": dispatch["execution_id"],
            "classification": "not_attempted",
            "attempted": false
        }),
    );
    assert_eq!(err(&not_attempted), "frame.payload_invalid");

    // /1 status keeps its frozen shape: no bundles array.
    let status = ok(&call(&mut gate, "s1", "status", json!({}))).clone();
    assert!(
        status.get("bundles").is_none(),
        "/1 status must not carry bundles"
    );
    assert_eq!(gate.provider_invocations(), 0);
}

// ---------------------------------------------------------------------------
// Proof 21: process.execute@1 behaviour-identical output truth.
// ---------------------------------------------------------------------------

#[test]
fn proof21_execute_v1_output_truth_unchanged() {
    use std::collections::{BTreeMap, BTreeSet};
    use tethers_reference_host::agent_coding::{
        process_execute, process_execute_argv, CodingScope,
    };

    // @1 and @2 execute the same resolved program with stdin closed and
    // bounded text streams; the outputs must agree field-for-field on
    // every shared truth key (@2 only adds the argv + composition echoes).
    let root = std::env::temp_dir().join(format!(
        "tethers-bundle-parity-{}",
        uuid::Uuid::new_v4().simple()
    ));
    fs::create_dir_all(&root).unwrap();
    let root = fs::canonicalize(&root).unwrap();
    let scope = CodingScope {
        repository_root: root.clone(),
        process_cwd_root: root,
        allowed_programs: ["git".to_owned()].into_iter().collect::<BTreeSet<_>>(),
        max_runtime_ms: 30_000,
        max_output_bytes: 4096,
        allowed_environment_keys: BTreeSet::new(),
        verification_checks: BTreeMap::new(),
    };
    let v1 = process_execute(&scope, &json!({"program": "git", "args": ["--version"]}))
        .expect("@1 executes");
    let v2 = process_execute_argv(
        &scope,
        &json!({"argv": ["git", "--version"], "composition_digest": "sha256:parity"}),
    )
    .expect("@2 executes");
    for key in [
        "program",
        "cwd",
        "exit_code",
        "timed_out",
        "stdout",
        "stdout_utf8",
        "stdout_truncated",
        "stderr",
        "stderr_utf8",
        "stderr_truncated",
    ] {
        assert_eq!(v1[key], v2[key], "output truth agrees on {key}");
    }
    assert_eq!(v1["exit_code"], 0);
    assert!(v1["stdout"].as_str().unwrap().starts_with("git version "));
    // @2-only echoes never leak into @1, and shell parsing stays absent:
    // argv[0] is the allow-listed program, no shell involved.
    assert_eq!(v2["argv"][0], "git");
    assert_eq!(v2["composition_digest"], "sha256:parity");
    assert!(v1.get("argv").is_none());
    fs::remove_dir_all(&scope.repository_root).unwrap();
}

// ---------------------------------------------------------------------------
// Proof 22: together stays fan-out/join (never a pipe), untouched by bundles.
// ---------------------------------------------------------------------------

#[test]
fn proof22_together_untouched_by_bundles() {
    // The bundle protocol has no interaction with together: commit_bundle
    // admits an ordered set of ordinary Actions atomically, while together
    // remains the fan-out/join coordinator in plan_execution. This test
    // pins the non-interaction at the protocol surface: no together field
    // appears on any bundle record, request, or response shape. Behavioural
    // together suites (host_execution, plan_execution) run unchanged.
    let workspace = BundleWorkspace::new("allow");
    let mut gate = workspace.gate();
    let first = prepare_member(
        &mut gate,
        &workspace,
        MemberSpec::new(
            "p1",
            "eval_b22_1",
            "evt_b22_1",
            "BK-1",
            "projects/bundle-tg-a",
            "comp-proof-22",
        ),
    );
    let second = prepare_member(
        &mut gate,
        &workspace,
        MemberSpec::new(
            "p2",
            "eval_b22_2",
            "evt_b22_2",
            "BK-2",
            "projects/bundle-tg-b",
            "comp-proof-22",
        ),
    );
    let dispatch = commit_bundle_ok(
        &mut gate,
        "c1",
        &[
            first["prepared_id"].as_str().unwrap(),
            second["prepared_id"].as_str().unwrap(),
        ],
        None,
    );
    let serialised = serde_json::to_string(&dispatch).unwrap();
    assert!(
        !serialised.contains("together"),
        "bundle dispatch must not mention together"
    );
    let committed = workspace.committed_files();
    assert_eq!(committed.len(), 1);
    let record = fs::read_to_string(&committed[0]).unwrap();
    assert!(
        !record.contains("together"),
        "bundle record must not mention together"
    );
    assert_eq!(gate.provider_invocations(), 0);
}

// ---------------------------------------------------------------------------
// Bundle-start nonattempt outcome (A10): narrowly bounded terminal truth.
// ---------------------------------------------------------------------------

#[test]
fn bundle_start_not_attempted_outcome() {
    let workspace = BundleWorkspace::new("allow");
    let mut gate = workspace.gate();

    let first = prepare_member(
        &mut gate,
        &workspace,
        MemberSpec::new(
            "p1",
            "eval_bna_1",
            "evt_bna_1",
            "BK-1",
            "projects/bundle-na-a",
            "comp-proof-na",
        ),
    );
    let second = prepare_member(
        &mut gate,
        &workspace,
        MemberSpec::new(
            "p2",
            "eval_bna_2",
            "evt_bna_2",
            "BK-2",
            "projects/bundle-na-b",
            "comp-proof-na",
        ),
    );
    let dispatch = commit_bundle_ok(
        &mut gate,
        "c1",
        &[
            first["prepared_id"].as_str().unwrap(),
            second["prepared_id"].as_str().unwrap(),
        ],
        None,
    );
    let exec_a = dispatch["members"][0]["execution_id"].as_str().unwrap();
    let exec_b = dispatch["members"][1]["execution_id"].as_str().unwrap();

    // Bundle-start nonattempt: attempted=false, no result, no error.
    let nonattempt = ok(&call2(
        &mut gate,
        "o1",
        "outcome",
        json!({
            "execution_id": exec_a,
            "classification": "not_attempted",
            "attempted": false,
            "evidence": "sha256:evidence-fixture"
        }),
    ))
    .clone();
    assert_eq!(nonattempt["status"], "not_attempted");

    // The 4-way distinction holds: the sibling still succeeds distinctly.
    let outcome = report_succeeded(&mut gate, "o2", exec_b, "external-bna-b");
    assert_eq!(outcome["status"], "succeeded");
    assert_ne!(nonattempt["status"], outcome["status"]);

    // Nonattempt is terminal: a later success for the same member conflicts.
    let conflict = call2(
        &mut gate,
        "o3",
        "outcome",
        json!({
            "execution_id": exec_a,
            "classification": "succeeded",
            "attempted": true,
            "external_execution_identity": "external-bna-a",
            "result": {"echo": "bundle-ok"},
            "evidence": "sha256:evidence-fixture"
        }),
    );
    assert_eq!(err(&conflict), "outcome.conflict");
    assert_eq!(gate.provider_invocations(), 0);
}
