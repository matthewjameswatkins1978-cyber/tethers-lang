use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use uuid::Uuid;

use tethers_reference_host::candidate::{extract_to_quarantine, CandidateRegistry};
use tethers_reference_host::conformance::{run_host_conformance, ConformanceEvidenceStore};
use tethers_reference_host::enablement::EnablementStore;
use tethers_reference_host::installed::{InstallationApprovalStore, InstalledPlugRegistry};
use tethers_reference_host::launch_profile::PreparedSupervisedLaunch;
use tethers_reference_host::package;
use tethers_reference_host::trust::{
    DeveloperApprovalStore, PackageTrustEvidence, PublisherTrustStore,
};

const PACKAGE_ID: &str = "example.text-inspector";
const CAPABILITY_NAME: &str = "text.inspect";

fn synthetic_operational_scope_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "workspace": {
                "type": "string",
                "x-tethers-path": "canonical-directory"
            },
            "limit": {
                "type": "integer",
                "minimum": 1,
                "maximum": 1000
            }
        },
        "required": ["workspace", "limit"],
        "additionalProperties": false
    })
}

fn fixture_input_schema() -> Value {
    json!({
        "type": "object",
        "properties": { "message": { "type": "string" } },
        "required": ["message"],
        "additionalProperties": false
    })
}

fn fixture_output_schema() -> Value {
    json!({
        "type": "object",
        "properties": { "echo": { "type": "string" } },
        "required": ["echo"],
        "additionalProperties": false
    })
}

fn synthetic_manifest_without_digest() -> Value {
    json!({
        "manifest_format_version": "1.0",
        "capability_name": CAPABILITY_NAME,
        "capability_version": 1,
        "title": "Example Text Inspector",
        "description": "Synthetic text inspection for unrelated-Plug proof.",
        "input_schema": fixture_input_schema(),
        "output_schema": fixture_output_schema(),
        "effects": ["data.read", "metadata.read"],
        "permission_scope": { "kind": "path_prefix", "allowed_prefixes": ["workspace/"] },
        "reversibility": "reversible",
        "determinism": "deterministic",
        "idempotency": { "mechanism": "none" },
        "confirmation_policy": { "standing_permitted": true, "per_call_required": false },
        "timeout_ms": 5000,
        "retry_policy": { "max_retries": 0, "backoff_ms": 0, "allowed_on": [], "requires_idempotency_proof": false },
        "provider": {
            "identity": "tethers-stdio-fixture",
            "display_name": "Example Text Inspector Provider",
            "identity_source": "host_configuration",
            "description": "Synthetic text inspection provider for proof."
        },
        "binding": { "kind": "mcp", "server_name": "tethers-stdio-fixture", "tool_name": "fixture_ping", "adapter": null }
    })
}

fn manifest_with_digest(mut manifest: Value) -> Value {
    let mut covered = manifest.clone();
    let covered_object = covered.as_object_mut().unwrap();
    covered_object.remove("digest");
    covered_object.remove("title");
    covered_object.remove("description");
    let bytes = serde_json_canonicalizer::to_vec(&covered).unwrap();
    manifest.as_object_mut().unwrap().insert(
        "digest".into(),
        Value::String(format!("sha256:{:x}", Sha256::digest(bytes))),
    );
    manifest
}

fn build_synthetic_package(provider_bytes: &[u8]) -> Vec<u8> {
    let manifest_value = manifest_with_digest(synthetic_manifest_without_digest());
    let manifest_bytes = serde_json::to_vec(&manifest_value).unwrap();
    let manifest_digest = manifest_value["digest"].as_str().unwrap().to_owned();
    let digest = |bytes: &[u8]| format!("sha256:{:x}", Sha256::digest(bytes));

    let plug = json!({
        "package_format_version": "1",
        "package_id": PACKAGE_ID,
        "package_version": "1.0.0",
        "display_name": "Example Text Inspector",
        "description": "Synthetic Plug for unrelated Plug proof.",
        "publisher": "Tethers reference material",
        "licence": "MIT",
        "socket_major": 1,
        "protocol_bindings": [{ "protocol": "MCP", "version": "2025-11-25", "transport": "stdio" }],
        "platforms": [{ "os": "windows", "architecture": "x86_64" }],
        "provider": {
            "provider_id": "tethers-stdio-fixture",
            "provider_version": "0.1.0",
            "launch": { "path": "provider/example-text-inspector-provider.exe", "arguments": [] },
            "working_directory": "provider",
            "capability_operation_namespace": "text",
            "operational_scope_schema": synthetic_operational_scope_schema()
        },
        "capabilities": [{
            "capability_name": CAPABILITY_NAME,
            "capability_version": 1,
            "manifest_path": "manifests/text-inspect-v1.json",
            "manifest_digest": manifest_digest,
            "provider_operation_name": "fixture_ping"
        }],
        "payload_index": [
            { "path": "manifests/text-inspect-v1.json", "sha256": digest(&manifest_bytes), "size_bytes": manifest_bytes.len(), "role": "capability_manifest" },
            { "path": "provider/example-text-inspector-provider.exe", "sha256": digest(provider_bytes), "size_bytes": provider_bytes.len(), "role": "provider_executable" }
        ]
    });

    let plug_bytes = serde_json_canonicalizer::to_vec(&plug).unwrap();
    let mut archive = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let options = zip::write::FileOptions::<()>::default()
        .last_modified_time(zip::DateTime::from_date_and_time(1980, 1, 1, 0, 0, 0).unwrap());

    for (path, bytes) in [
        ("plug.json", plug_bytes.as_slice()),
        ("manifests/text-inspect-v1.json", manifest_bytes.as_slice()),
        (
            "provider/example-text-inspector-provider.exe",
            provider_bytes,
        ),
    ] {
        archive.start_file(path, options).unwrap();
        archive.write_all(bytes).unwrap();
    }
    archive.finish().map(|cursor| cursor.into_inner()).unwrap()
}

fn schema_digest(schema: &Value) -> String {
    let bytes = serde_json_canonicalizer::to_vec(schema).unwrap();
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn workspace_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("tethers-r1e-workspace-{}", Uuid::new_v4()));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn host_binary() -> PathBuf {
    std::env::var_os("CARGO_BIN_EXE_tethers-reference-host")
        .or_else(|| std::env::var_os("CARGO_BIN_EXE_tethers_reference_host"))
        .map(PathBuf::from)
        .or_else(|| {
            std::env::current_exe()
                .ok()?
                .parent()
                .map(|path| path.join("tethers-reference-host.exe"))
        })
        .expect("compiled reference host binary")
}

fn fixture_provider_bytes() -> Vec<u8> {
    fs::read(env!("CARGO_BIN_EXE_m3_fixture_provider")).unwrap()
}

fn install_synthetic(root: &Path) -> tethers_reference_host::installed::InstalledPlugRecord {
    let archive = root.join("synthetic.tetherplug");
    fs::write(&archive, build_synthetic_package(&fixture_provider_bytes())).unwrap();
    let report = package::inspect(&archive).unwrap();
    let quarantine_root = root.join("quarantine");
    let quarantined = extract_to_quarantine(&report, &quarantine_root).unwrap();
    let candidate = CandidateRegistry::open(&root.join("candidates"), &quarantine_root)
        .unwrap()
        .create(&quarantined)
        .unwrap();
    let developers = DeveloperApprovalStore::open(&root.join("developer")).unwrap();
    let developer = developers
        .approve_exact_digest(&candidate.semantic_package_digest, "Matthew")
        .unwrap();
    let trust = PackageTrustEvidence::unsigned(&developer).unwrap();
    let publishers = PublisherTrustStore::open(&root.join("publishers")).unwrap();
    let prepared = PreparedSupervisedLaunch::prepare(
        &candidate,
        &quarantine_root,
        &root.join("scratch"),
        std::time::Duration::from_secs(10),
    )
    .unwrap();
    let conformance = run_host_conformance(
        &prepared,
        &candidate,
        &quarantine_root,
        &trust,
        &publishers,
        &developers,
        "tethers-reference-host@0.2.0+r1e",
    )
    .unwrap();
    ConformanceEvidenceStore::open(&root.join("conformance"))
        .unwrap()
        .create(&conformance)
        .unwrap();
    let approval = InstallationApprovalStore::open(&root.join("approvals"))
        .unwrap()
        .approve(
            &candidate,
            &quarantine_root,
            &trust,
            &publishers,
            &developers,
            &prepared.evidence,
            &conformance,
            "Matthew",
        )
        .unwrap();
    let registry =
        InstalledPlugRegistry::open(&root.join("install"), &root.join("installed-records"))
            .unwrap();
    registry
        .install_disabled(
            &candidate,
            &quarantine_root,
            &trust,
            &publishers,
            &developers,
            &prepared.evidence,
            &conformance,
            &approval,
        )
        .unwrap()
}

fn enable_positive_via_cli(root: &Path, installed_id: &str, scope_file: &Path) -> (i32, Value) {
    run_enable_cli(root, installed_id, scope_file)
}

fn write_valid_scope_file(path: &Path, workspace: &Path, limit: u64) {
    let content = json!({
        "schema": "tethers.plug-scope/1",
        "scope": { "workspace": workspace.to_string_lossy(), "limit": limit }
    });
    fs::write(path, serde_json::to_vec(&content).unwrap()).unwrap();
}

fn load_enablement(
    root: &Path,
    installed_id: &str,
) -> tethers_reference_host::enablement::EnablementRecord {
    let enablements = EnablementStore::open_existing(&root.join("enablements")).unwrap();
    let all = enablements.load_all().unwrap();
    latest_for(&all, installed_id).clone()
}

fn records_for<'a>(
    records: &'a [tethers_reference_host::enablement::EnablementRecord],
    installed_id: &str,
) -> Vec<&'a tethers_reference_host::enablement::EnablementRecord> {
    let mut found: Vec<_> = records
        .iter()
        .filter(|r| r.installed_id == installed_id)
        .collect();
    found.sort_by_key(|r| r.sequence);
    found
}

fn latest_for<'a>(
    records: &'a [tethers_reference_host::enablement::EnablementRecord],
    installed_id: &str,
) -> &'a tethers_reference_host::enablement::EnablementRecord {
    let chain = records_for(records, installed_id);
    assert!(
        !chain.is_empty(),
        "no enablement records for {installed_id}"
    );
    chain.last().unwrap()
}

fn run_enable_cli(root: &Path, installed_id: &str, scope_path: &Path) -> (i32, Value) {
    let output = Command::new(host_binary())
        .args(["plug", "enable", "--host-data-root"])
        .arg(root)
        .args(["--installed-id", installed_id])
        .args(["--scope"])
        .arg(scope_path)
        .output()
        .expect("reference host process");
    let code = output.status.code().expect("process exit code");
    let envelope: Value = serde_json::from_slice(&output.stdout).expect("one JSON envelope");
    assert_eq!(code, envelope["exit_code"].as_i64().unwrap() as i32);
    assert_eq!(String::from_utf8(output.stderr).unwrap(), "");
    (code, envelope)
}

fn install_and_enable_setup() -> (
    PathBuf,
    tethers_reference_host::installed::InstalledPlugRecord,
    PathBuf,
) {
    let root = std::env::temp_dir().join(format!("tethers-r1e-{}-", Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(root.join("enablements")).unwrap();
    let installed = install_synthetic(&root);
    let workspace = workspace_dir();
    (root, installed, workspace)
}

// ── Inspection tests ──

#[test]
fn inspection_accepts_synthetic_package() {
    let archive_bytes = build_synthetic_package(&fixture_provider_bytes());
    let root = std::env::temp_dir().join(format!("tethers-r1e-inspect-{}", Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    let archive = root.join("synthetic.tetherplug");
    fs::write(&archive, &archive_bytes).unwrap();

    let report = package::inspect(&archive).unwrap();
    assert_eq!(report.package.package_id, PACKAGE_ID);
    assert_eq!(report.package.package_version, "1.0.0");
    assert_eq!(report.capabilities.len(), 1);
    assert_eq!(report.capabilities[0].name, CAPABILITY_NAME);
    assert_eq!(report.capabilities[0].version, 1);
    assert!(!report.inspection_evidence_digest.is_empty());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn inspection_exposes_exact_operational_scope_schema() {
    let archive_bytes = build_synthetic_package(&fixture_provider_bytes());
    let root = std::env::temp_dir().join(format!("tethers-r1e-schema-{}", Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    let archive = root.join("synthetic.tetherplug");
    fs::write(&archive, &archive_bytes).unwrap();

    let report = package::inspect(&archive).unwrap();
    assert!(report.operational_scope_schema.is_some());
    let schema = report.operational_scope_schema.unwrap();
    let expected = synthetic_operational_scope_schema();
    let canonical_schema = serde_json_canonicalizer::to_vec(&schema).unwrap();
    let canonical_expected = serde_json_canonicalizer::to_vec(&expected).unwrap();
    assert_eq!(canonical_schema, canonical_expected);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn schema_digest_is_deterministic() {
    let schema = synthetic_operational_scope_schema();
    let d1 = schema_digest(&schema);
    let d2 = schema_digest(&schema);
    assert_eq!(d1, d2);
    assert!(d1.starts_with("sha256:"));
    assert_eq!(d1.len(), 71);
}

#[test]
fn inspection_computes_exact_schema_digest() {
    let archive_bytes = build_synthetic_package(&fixture_provider_bytes());
    let root = std::env::temp_dir().join(format!("tethers-r1e-digest-{}", Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    let archive = root.join("synthetic.tetherplug");
    fs::write(&archive, &archive_bytes).unwrap();

    let report = package::inspect(&archive).unwrap();
    assert!(report.operational_scope_schema_digest.is_some());
    let expected = schema_digest(&synthetic_operational_scope_schema());
    assert_eq!(report.operational_scope_schema_digest.unwrap(), expected);
    let _ = fs::remove_dir_all(root);
}

// ── Candidate evidence chain ──

#[test]
fn candidate_evidence_preserves_schema_and_digest() {
    let root = std::env::temp_dir().join(format!("tethers-r1e-candidate-{}", Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    let archive = root.join("synthetic.tetherplug");
    fs::write(&archive, build_synthetic_package(&fixture_provider_bytes())).unwrap();

    let report = package::inspect(&archive).unwrap();
    let quarantine_root = root.join("quarantine");
    let quarantined = extract_to_quarantine(&report, &quarantine_root).unwrap();
    let candidate = CandidateRegistry::open(&root.join("candidates"), &quarantine_root)
        .unwrap()
        .create(&quarantined)
        .unwrap();

    assert_eq!(candidate.package_id, PACKAGE_ID);
    let expected_schema = synthetic_operational_scope_schema();
    let canonical_schema =
        serde_json_canonicalizer::to_vec(candidate.operational_scope_schema.as_ref().unwrap())
            .unwrap();
    let canonical_expected = serde_json_canonicalizer::to_vec(&expected_schema).unwrap();
    assert_eq!(canonical_schema, canonical_expected);
    assert_eq!(
        candidate
            .operational_scope_schema_digest
            .as_deref()
            .unwrap(),
        schema_digest(&expected_schema)
    );
    let _ = fs::remove_dir_all(root);
}

// ── Installed evidence chain ──

#[test]
fn installed_evidence_preserves_schema_and_digest() {
    let root = std::env::temp_dir().join(format!("tethers-r1e-installed-{}", Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(root.join("enablements")).unwrap();
    let installed = install_synthetic(&root);

    assert_eq!(installed.package_id, PACKAGE_ID);
    let expected_schema = synthetic_operational_scope_schema();
    let canonical_schema =
        serde_json_canonicalizer::to_vec(installed.operational_scope_schema.as_ref().unwrap())
            .unwrap();
    let canonical_expected = serde_json_canonicalizer::to_vec(&expected_schema).unwrap();
    assert_eq!(canonical_schema, canonical_expected);
    assert_eq!(
        installed
            .operational_scope_schema_digest
            .as_deref()
            .unwrap(),
        schema_digest(&expected_schema)
    );
    let _ = fs::remove_dir_all(root);
}

// ── Positive enablement through real CLI ──

#[test]
fn enablement_with_valid_scope_succeeds() {
    let (root, installed, workspace) = install_and_enable_setup();
    let scope_file = root.join("valid_scope.json");
    write_valid_scope_file(&scope_file, &workspace, 37);
    let (code, envelope) = enable_positive_via_cli(&root, &installed.installed_id, &scope_file);
    assert_eq!(code, 0, "CLI enable failed: {envelope}");
    assert_eq!(envelope["status"], "ok");

    let record = load_enablement(&root, &installed.installed_id);
    assert!(!record.operational_scope_digest.is_empty());

    let _ = fs::remove_dir_all(workspace);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn workspace_is_canonicalised_generically() {
    let (root, installed, workspace) = install_and_enable_setup();
    let scope_file = root.join("valid_scope.json");
    write_valid_scope_file(&scope_file, &workspace, 42);
    let (code, _envelope) = enable_positive_via_cli(&root, &installed.installed_id, &scope_file);
    assert_eq!(code, 0);

    let record = load_enablement(&root, &installed.installed_id);
    let scope = record.operational_scope.canonical_scope().unwrap();
    let canonical_path = scope.get("workspace").unwrap().as_str().unwrap();
    let from_evidence = fs::canonicalize(canonical_path).unwrap();
    let expected = fs::canonicalize(&workspace).unwrap();
    assert_eq!(from_evidence, expected);

    let _ = fs::remove_dir_all(workspace);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn limit_is_preserved_exactly() {
    let (root, installed, workspace) = install_and_enable_setup();
    let scope_file = root.join("valid_scope.json");
    write_valid_scope_file(&scope_file, &workspace, 37);
    let (code, _envelope) = enable_positive_via_cli(&root, &installed.installed_id, &scope_file);
    assert_eq!(code, 0);

    let record = load_enablement(&root, &installed.installed_id);
    let scope = record.operational_scope.canonical_scope().unwrap();
    assert_eq!(scope.get("limit").unwrap().as_i64().unwrap(), 37);

    let _ = fs::remove_dir_all(workspace);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn operational_scope_evidence_contains_canonical_workspace_and_limit() {
    let (root, installed, workspace) = install_and_enable_setup();
    let scope_file = root.join("valid_scope.json");
    write_valid_scope_file(&scope_file, &workspace, 37);
    let (code, _envelope) = enable_positive_via_cli(&root, &installed.installed_id, &scope_file);
    assert_eq!(code, 0);

    let record = load_enablement(&root, &installed.installed_id);
    let evidence = &record.operational_scope;

    let canonical: Value = serde_json::from_str(&evidence.canonical_scope_json).unwrap();
    let canonical_path = canonical.get("workspace").unwrap().as_str().unwrap();
    let from_evidence = fs::canonicalize(canonical_path).unwrap();
    let expected = fs::canonicalize(&workspace).unwrap();
    assert_eq!(from_evidence, expected);
    assert_eq!(canonical.get("limit").unwrap().as_i64().unwrap(), 37);
    assert_eq!(canonical.as_object().unwrap().len(), 2);

    let _ = fs::remove_dir_all(workspace);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn operational_scope_evidence_carries_exact_schema_digest() {
    let (root, installed, workspace) = install_and_enable_setup();
    let scope_file = root.join("valid_scope.json");
    write_valid_scope_file(&scope_file, &workspace, 50);
    let (code, _envelope) = enable_positive_via_cli(&root, &installed.installed_id, &scope_file);
    assert_eq!(code, 0);

    let record = load_enablement(&root, &installed.installed_id);
    let evidence = &record.operational_scope;

    assert_eq!(
        &evidence.scope_schema_digest,
        installed
            .operational_scope_schema_digest
            .as_deref()
            .unwrap()
    );

    let _ = fs::remove_dir_all(workspace);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn repeated_creation_produces_deterministic_evidence() {
    let workspace = workspace_dir();
    let dir1 = std::env::temp_dir().join(format!("tethers-r1e-det-1-{}", Uuid::new_v4()));
    let dir2 = std::env::temp_dir().join(format!("tethers-r1e-det-2-{}", Uuid::new_v4()));
    fs::create_dir_all(&dir1).unwrap();
    fs::create_dir_all(&dir2).unwrap();
    fs::create_dir_all(dir1.join("enablements")).unwrap();
    fs::create_dir_all(dir2.join("enablements")).unwrap();

    let installed1 = install_synthetic(&dir1);
    let scope1_file = dir1.join("scope.json");
    write_valid_scope_file(&scope1_file, &workspace, 37);
    let (code1, _) = enable_positive_via_cli(&dir1, &installed1.installed_id, &scope1_file);
    assert_eq!(code1, 0);

    let installed2 = install_synthetic(&dir2);
    let scope2_file = dir2.join("scope.json");
    write_valid_scope_file(&scope2_file, &workspace, 37);
    let (code2, _) = enable_positive_via_cli(&dir2, &installed2.installed_id, &scope2_file);
    assert_eq!(code2, 0);

    let record1 = load_enablement(&dir1, &installed1.installed_id);
    let record2 = load_enablement(&dir2, &installed2.installed_id);

    assert_eq!(
        record1.operational_scope.canonical_scope_json,
        record2.operational_scope.canonical_scope_json
    );
    assert_eq!(
        record1.operational_scope.scope_schema_digest,
        record2.operational_scope.scope_schema_digest
    );

    let _ = fs::remove_dir_all(workspace);
    let _ = fs::remove_dir_all(dir1);
    let _ = fs::remove_dir_all(dir2);
}

// ── Negative scope validation tests (via CLI plug enable) ──

#[test]
fn missing_workspace_fails_via_cli() {
    let (root, installed, _workspace) = install_and_enable_setup();
    let path = root.join("no_workspace.json");
    let content = json!({
        "schema": "tethers.plug-scope/1",
        "scope": { "limit": 37 }
    });
    fs::write(&path, serde_json::to_vec(&content).unwrap()).unwrap();
    let (code, envelope) = run_enable_cli(&root, &installed.installed_id, &path);
    assert_ne!(code, 0, "{envelope}");
    assert_eq!(envelope["status"], "invalid_data");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn missing_limit_fails_via_cli() {
    let (root, installed, workspace) = install_and_enable_setup();
    let path = root.join("no_limit.json");
    let content = json!({
        "schema": "tethers.plug-scope/1",
        "scope": { "workspace": workspace.to_string_lossy() }
    });
    fs::write(&path, serde_json::to_vec(&content).unwrap()).unwrap();
    let (code, envelope) = run_enable_cli(&root, &installed.installed_id, &path);
    assert_ne!(code, 0, "{envelope}");
    assert_eq!(envelope["status"], "invalid_data");
    let _ = fs::remove_dir_all(workspace);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn relative_workspace_fails_via_cli() {
    let (root, installed, _workspace) = install_and_enable_setup();
    let path = root.join("relative.json");
    let content = json!({
        "schema": "tethers.plug-scope/1",
        "scope": { "workspace": "relative/path", "limit": 37 }
    });
    fs::write(&path, serde_json::to_vec(&content).unwrap()).unwrap();
    let (code, envelope) = run_enable_cli(&root, &installed.installed_id, &path);
    assert_ne!(code, 0, "{envelope}");
    assert_eq!(envelope["status"], "invalid_data");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn nonexistent_workspace_fails_via_cli() {
    let (root, installed, _workspace) = install_and_enable_setup();
    let path = root.join("nonexistent.json");
    let content = json!({
        "schema": "tethers.plug-scope/1",
        "scope": { "workspace": "C:\\nonexistent-directory-r1ezzzz", "limit": 37 }
    });
    fs::write(&path, serde_json::to_vec(&content).unwrap()).unwrap();
    let (code, envelope) = run_enable_cli(&root, &installed.installed_id, &path);
    assert_ne!(code, 0, "{envelope}");
    assert_eq!(envelope["status"], "invalid_data");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn limit_zero_fails_via_cli() {
    let (root, installed, workspace) = install_and_enable_setup();
    let path = root.join("zero.json");
    let content = json!({
        "schema": "tethers.plug-scope/1",
        "scope": { "workspace": workspace.to_string_lossy(), "limit": 0 }
    });
    fs::write(&path, serde_json::to_vec(&content).unwrap()).unwrap();
    let (code, envelope) = run_enable_cli(&root, &installed.installed_id, &path);
    assert_ne!(code, 0, "{envelope}");
    assert_eq!(envelope["status"], "invalid_data");
    let _ = fs::remove_dir_all(workspace);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn limit_1001_fails_via_cli() {
    let (root, installed, workspace) = install_and_enable_setup();
    let path = root.join("over.json");
    let content = json!({
        "schema": "tethers.plug-scope/1",
        "scope": { "workspace": workspace.to_string_lossy(), "limit": 1001 }
    });
    fs::write(&path, serde_json::to_vec(&content).unwrap()).unwrap();
    let (code, envelope) = run_enable_cli(&root, &installed.installed_id, &path);
    assert_ne!(code, 0, "{envelope}");
    assert_eq!(envelope["status"], "invalid_data");
    let _ = fs::remove_dir_all(workspace);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn limit_wrong_type_fails_via_cli() {
    let (root, installed, workspace) = install_and_enable_setup();
    let path = root.join("bad_type.json");
    let content = json!({
        "schema": "tethers.plug-scope/1",
        "scope": { "workspace": workspace.to_string_lossy(), "limit": "not-a-number" }
    });
    fs::write(&path, serde_json::to_vec(&content).unwrap()).unwrap();
    let (code, envelope) = run_enable_cli(&root, &installed.installed_id, &path);
    assert_ne!(code, 0, "{envelope}");
    assert_eq!(envelope["status"], "invalid_data");
    let _ = fs::remove_dir_all(workspace);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn unknown_scope_field_fails_via_cli() {
    let (root, installed, workspace) = install_and_enable_setup();
    let path = root.join("extra.json");
    let content = json!({
        "schema": "tethers.plug-scope/1",
        "scope": { "workspace": workspace.to_string_lossy(), "limit": 37, "extra": "bad" }
    });
    fs::write(&path, serde_json::to_vec(&content).unwrap()).unwrap();
    let (code, envelope) = run_enable_cli(&root, &installed.installed_id, &path);
    assert_ne!(code, 0, "{envelope}");
    assert_eq!(envelope["status"], "invalid_data");
    let _ = fs::remove_dir_all(workspace);
    let _ = fs::remove_dir_all(root);
}
