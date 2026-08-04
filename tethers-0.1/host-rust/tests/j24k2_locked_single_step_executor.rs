#![cfg(windows)]

use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::io::{Cursor, Write};
use std::os::windows::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::time::Duration;
use tethers_reference_host::candidate::CandidateRecord;
use tethers_reference_host::candidate::{extract_to_quarantine, CandidateRegistry};
use tethers_reference_host::conformance::{
    run_host_conformance, ConformanceDisposition, ConformanceEvidenceStore,
};
use tethers_reference_host::installation_execution::{
    execute_next_installation_action, InstallationExecutionContext, InstallationExecutionOptions,
    InstallationStepOutcome,
};
use tethers_reference_host::installation_plan::InstallationPlanAction;
use tethers_reference_host::installation_request::{
    InstallationConformanceRequest, InstallationRequest, InstallationTargetRequest,
    InstallationTargetState, InstallationTrustRequest, InstallationTrustScope,
    INSTALLATION_REQUEST_SCHEMA,
};
use tethers_reference_host::installation_trust::ExactCandidateTrustStore;
use tethers_reference_host::installed::{InstallationApprovalStore, InstalledPlugRegistry};
use tethers_reference_host::launch_profile::LaunchProfileEvidenceStore;
use tethers_reference_host::package;
use tethers_reference_host::pdf_tools;
use tethers_reference_host::trust::{
    DeveloperApprovalStore, PackageTrustEvidence, PublisherTrustStore,
};
use uuid::Uuid;

fn sha256(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn temp_dir(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("tethers-j24k2-{name}-{}", Uuid::new_v4()))
}

fn snapshot(root: &Path) -> BTreeMap<String, String> {
    fn visit(root: &Path, path: &Path, output: &mut BTreeMap<String, String>) {
        if !path.is_dir() {
            return;
        }
        let mut entries = fs::read_dir(path)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect::<Vec<_>>();
        entries.sort();
        for entry in entries {
            let relative = entry
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            let metadata = fs::symlink_metadata(&entry).unwrap();
            if metadata.is_dir() {
                output.insert(relative, "<directory>".to_string());
                visit(root, &entry, output);
            } else {
                let content = fs::read(&entry).unwrap_or_default();
                output.insert(relative, sha256(&content));
            }
        }
    }
    let mut map = BTreeMap::new();
    if root.exists() {
        visit(root, root, &mut map);
    }
    map
}

fn setup_candidate(
    base: &Path,
) -> (
    CandidateRegistry,
    tethers_reference_host::candidate::CandidateRecord,
    PathBuf,
) {
    let quarantine_root = base.join("quarantine");
    let archive = base.join("pdf-tools.tetherplug");
    let provider_bytes =
        fs::read(env!("CARGO_BIN_EXE_pdf_tools_provider")).expect("compiled provider");
    fs::write(
        &archive,
        pdf_tools::build_reference_package(&provider_bytes).unwrap(),
    )
    .unwrap();
    let report = package::inspect(&archive).unwrap();
    let quarantined = extract_to_quarantine(&report, &quarantine_root).unwrap();
    let candidates = CandidateRegistry::open(&base.join("candidates"), &quarantine_root).unwrap();
    let candidate = candidates.create(&quarantined).unwrap();
    (candidates, candidate, quarantine_root)
}

fn setup_m3_candidate(
    base: &Path,
    mode: &str,
    extra_arguments: &[String],
) -> (CandidateRegistry, CandidateRecord, PathBuf) {
    let quarantine_root = base.join("quarantine");
    let archive_path = base.join("fixture.tetherplug");
    let provider = fs::read(env!("CARGO_BIN_EXE_m3_fixture_provider")).unwrap();
    let manifest = fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../protocol/capability-manifests/fixture-ping.json"),
    )
    .unwrap();
    let manifest_digest =
        "sha256:01fed7a4b877dd82abe91a1b6cfcd476b02e4c115489e70cbb285b8bf2d32d8b".to_owned();
    let mut arguments = vec![
        "--mode".to_owned(),
        mode.to_owned(),
        "--ordered".to_owned(),
        "first".to_owned(),
        "second".to_owned(),
    ];
    arguments.extend(extra_arguments.iter().cloned());
    let plug = serde_json::json!({
        "package_format_version":"1", "package_id":"tethers.fixture",
        "package_version":"0.1.0", "display_name":"M3 Fixture",
        "description":"J24K2 fixture", "publisher":"fixture", "licence":"MIT",
        "socket_major":1,
        "protocol_bindings":[{"protocol":"MCP","version":"2025-11-25","transport":"stdio"}],
        "platforms":[{"os":"windows","architecture":"x86_64"}],
        "provider":{"provider_id":"tethers-stdio-fixture","provider_version":"0.1.0",
          "launch":{"path":"provider/m3_fixture_provider.exe","arguments":arguments},
          "working_directory":"provider","capability_operation_namespace":"fixture"},
        "capabilities":[{"capability_name":"fixture.ping","capability_version":1,
          "manifest_path":"manifests/fixture-ping.json","manifest_digest":manifest_digest,
          "provider_operation_name":"fixture_ping"}],
        "payload_index":[
          {"path":"manifests/fixture-ping.json","sha256":sha256(&manifest),
            "size_bytes":manifest.len(),"role":"capability_manifest"},
          {"path":"provider/m3_fixture_provider.exe","sha256":sha256(&provider),
            "size_bytes":provider.len(),"role":"provider_executable"}]
    });
    let plug = serde_json_canonicalizer::to_vec(&plug).unwrap();
    let mut archive = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let options =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    for (path, bytes) in [
        ("plug.json", plug.as_slice()),
        ("manifests/fixture-ping.json", manifest.as_slice()),
        ("provider/m3_fixture_provider.exe", provider.as_slice()),
    ] {
        archive.start_file(path, options).unwrap();
        archive.write_all(bytes).unwrap();
    }
    fs::create_dir_all(base).unwrap();
    fs::write(&archive_path, archive.finish().unwrap().into_inner()).unwrap();
    let report = package::inspect(&archive_path).unwrap();
    let quarantined = extract_to_quarantine(&report, &quarantine_root).unwrap();
    let candidates = CandidateRegistry::open(&base.join("candidates"), &quarantine_root).unwrap();
    let candidate = candidates.create(&quarantined).unwrap();
    (candidates, candidate, quarantine_root)
}

struct InterruptionReset(bool);

static TEST_SERIAL: OnceLock<Mutex<()>> = OnceLock::new();

fn test_serial() -> MutexGuard<'static, ()> {
    TEST_SERIAL.get_or_init(|| Mutex::new(())).lock().unwrap()
}

impl InterruptionReset {
    fn set() -> Self {
        let previous =
            tethers_reference_host::child_process::INTERRUPTED.swap(true, Ordering::AcqRel);
        Self(previous)
    }
}

impl Drop for InterruptionReset {
    fn drop(&mut self) {
        tethers_reference_host::child_process::INTERRUPTED.store(self.0, Ordering::Release);
    }
}

fn valid_request(candidate_id: &str) -> InstallationRequest {
    InstallationRequest {
        schema: INSTALLATION_REQUEST_SCHEMA.to_string(),
        candidate_id: candidate_id.to_string(),
        trust: InstallationTrustRequest {
            scope: InstallationTrustScope::ExactCandidate,
        },
        conformance: InstallationConformanceRequest {
            allow_non_isolated_supervised_execution: true,
        },
        installation: InstallationTargetRequest {
            target_state: InstallationTargetState::Disabled,
        },
    }
}

fn make_context<'a>(
    lock_path: &'a Path,
    quarantine_root: &'a Path,
    scratch: &'a Path,
    candidates: &'a CandidateRegistry,
    exact_trust: &'a ExactCandidateTrustStore,
    profiles: &'a LaunchProfileEvidenceStore,
    conformance_store: &'a ConformanceEvidenceStore,
    approvals: &'a InstallationApprovalStore,
    installed: &'a InstalledPlugRegistry,
) -> InstallationExecutionContext<'a> {
    InstallationExecutionContext {
        lock_path,
        quarantine_root,
        conformance_scratch_root: scratch,
        candidates,
        exact_trust,
        launch_profiles: profiles,
        conformance: conformance_store,
        approvals,
        installed,
    }
}

fn valid_options() -> InstallationExecutionOptions<'static> {
    InstallationExecutionOptions {
        approving_authority: "test-integrator",
        host_build_identity: "tethers-j24k2-tests",
        conformance_wall_time: Duration::from_secs(30),
    }
}

#[test]
fn j24k2_create_exact_candidate_trust_advances_once() {
    let _serial = test_serial();
    let base = temp_dir("create-trust");
    fs::create_dir_all(&base).unwrap();

    let (candidates, candidate, quarantine_root) = setup_candidate(&base);

    let lock_dir = base.join("lock");
    fs::create_dir_all(&lock_dir).unwrap();
    let lock_path = lock_dir.join("anchor.lock");

    let exact_trust = ExactCandidateTrustStore::open(&base.join("trust")).unwrap();
    let profiles = LaunchProfileEvidenceStore::open(&base.join("profiles")).unwrap();
    let conformance_store = ConformanceEvidenceStore::open(&base.join("conformance")).unwrap();
    let approvals = InstallationApprovalStore::open(&base.join("approvals")).unwrap();
    let installed =
        InstalledPlugRegistry::open(&base.join("install"), &base.join("records")).unwrap();

    let scratch = base.join("scratch");
    fs::create_dir_all(&scratch).unwrap();

    let context = make_context(
        &lock_path,
        &quarantine_root,
        &scratch,
        &candidates,
        &exact_trust,
        &profiles,
        &conformance_store,
        &approvals,
        &installed,
    );
    let options = valid_options();
    let request = valid_request(&candidate.candidate_id);

    let result = execute_next_installation_action(&request, &context, &options).unwrap();
    assert_eq!(
        result.before.action,
        InstallationPlanAction::CreateExactCandidateTrust
    );
    match &result.outcome {
        InstallationStepOutcome::Advanced { executed } => {
            assert_eq!(*executed, InstallationPlanAction::CreateExactCandidateTrust);
        }
        other => panic!("expected Advanced, got {:?}", other),
    }
    assert_eq!(
        result.after.action,
        InstallationPlanAction::RunSupervisedConformance
    );
    assert!(result.after.exact_candidate_trust_record_digest.is_some());
    assert!(result.after.trust_evidence_digest.is_some());

    fs::remove_dir_all(&base).unwrap();
}

#[test]
fn j24k2_lock_busy_before_planning() {
    let _serial = test_serial();
    let base = temp_dir("lock-busy");
    fs::create_dir_all(&base).unwrap();

    let (candidates, candidate, quarantine_root) = setup_candidate(&base);

    let lock_dir = base.join("lock");
    fs::create_dir_all(&lock_dir).unwrap();
    let lock_path = lock_dir.join("anchor.lock");

    let scratch = base.join("scratch");
    fs::create_dir_all(&scratch).unwrap();

    let exact_trust = ExactCandidateTrustStore::open(&base.join("trust")).unwrap();
    let profiles = LaunchProfileEvidenceStore::open(&base.join("profiles")).unwrap();
    let conformance_store = ConformanceEvidenceStore::open(&base.join("conformance")).unwrap();
    let approvals = InstallationApprovalStore::open(&base.join("approvals")).unwrap();
    let installed =
        InstalledPlugRegistry::open(&base.join("install"), &base.join("records")).unwrap();

    let request = valid_request(&candidate.candidate_id);
    let options = valid_options();

    let _lock_file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .share_mode(0)
        .open(&lock_path)
        .unwrap();

    let context = make_context(
        &lock_path,
        &quarantine_root,
        &scratch,
        &candidates,
        &exact_trust,
        &profiles,
        &conformance_store,
        &approvals,
        &installed,
    );

    let result = execute_next_installation_action(&request, &context, &options);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code, "installation_busy");

    drop(_lock_file);
    fs::remove_dir_all(&base).unwrap();
}

#[test]
fn j24k2_lock_releases_after_error() {
    let _serial = test_serial();
    let base = temp_dir("lock-error");
    fs::create_dir_all(&base).unwrap();

    let (candidates, candidate, quarantine_root) = setup_candidate(&base);

    let lock_dir = base.join("lock");
    fs::create_dir_all(&lock_dir).unwrap();
    let lock_path = lock_dir.join("anchor.lock");

    let scratch = base.join("scratch");
    fs::create_dir_all(&scratch).unwrap();

    let exact_trust = ExactCandidateTrustStore::open(&base.join("trust")).unwrap();
    let profiles = LaunchProfileEvidenceStore::open(&base.join("profiles")).unwrap();
    let conformance_store = ConformanceEvidenceStore::open(&base.join("conformance")).unwrap();
    let approvals = InstallationApprovalStore::open(&base.join("approvals")).unwrap();
    let installed =
        InstalledPlugRegistry::open(&base.join("install"), &base.join("records")).unwrap();

    let request = valid_request(&candidate.candidate_id);

    let context = make_context(
        &lock_path,
        &quarantine_root,
        &scratch,
        &candidates,
        &exact_trust,
        &profiles,
        &conformance_store,
        &approvals,
        &installed,
    );

    let bad_options = InstallationExecutionOptions {
        approving_authority: "",
        host_build_identity: "host",
        conformance_wall_time: Duration::from_secs(30),
    };

    let result = execute_next_installation_action(&request, &context, &bad_options);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().code,
        "installation_execution_options_invalid"
    );

    let result2 = execute_next_installation_action(&request, &context, &valid_options());
    assert!(result2.is_ok());

    fs::remove_dir_all(&base).unwrap();
}

#[test]
fn j24k2_options_invalid_rejected_before_mutation() {
    let _serial = test_serial();
    let base = temp_dir("opts-invalid");
    fs::create_dir_all(&base).unwrap();

    let (candidates, candidate, quarantine_root) = setup_candidate(&base);

    let lock_dir = base.join("lock");
    fs::create_dir_all(&lock_dir).unwrap();
    let lock_path = lock_dir.join("anchor.lock");

    let scratch = base.join("scratch");
    fs::create_dir_all(&scratch).unwrap();

    let exact_trust = ExactCandidateTrustStore::open(&base.join("trust")).unwrap();
    let profiles = LaunchProfileEvidenceStore::open(&base.join("profiles")).unwrap();
    let conformance_store = ConformanceEvidenceStore::open(&base.join("conformance")).unwrap();
    let approvals = InstallationApprovalStore::open(&base.join("approvals")).unwrap();
    let installed =
        InstalledPlugRegistry::open(&base.join("install"), &base.join("records")).unwrap();

    let request = valid_request(&candidate.candidate_id);

    let context = make_context(
        &lock_path,
        &quarantine_root,
        &scratch,
        &candidates,
        &exact_trust,
        &profiles,
        &conformance_store,
        &approvals,
        &installed,
    );

    let bad_options = InstallationExecutionOptions {
        approving_authority: "",
        host_build_identity: "",
        conformance_wall_time: Duration::ZERO,
    };

    let result = execute_next_installation_action(&request, &context, &bad_options);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().code,
        "installation_execution_options_invalid"
    );

    fs::remove_dir_all(&base).unwrap();
}

// --- Passed conformance and approval chain ---

#[test]
fn j24k2_full_passed_conformance_and_approval_chain() {
    let _serial = test_serial();
    let base = temp_dir("full-chain");
    fs::create_dir_all(&base).unwrap();

    let (candidates, candidate, quarantine_root) = setup_candidate(&base);

    let lock_dir = base.join("lock");
    fs::create_dir_all(&lock_dir).unwrap();
    let lock_path = lock_dir.join("anchor.lock");

    let scratch = base.join("scratch");
    fs::create_dir_all(&scratch).unwrap();

    let exact_trust = ExactCandidateTrustStore::open(&base.join("trust")).unwrap();
    let profiles = LaunchProfileEvidenceStore::open(&base.join("profiles")).unwrap();
    let conformance_store = ConformanceEvidenceStore::open(&base.join("conformance")).unwrap();
    let approvals = InstallationApprovalStore::open(&base.join("approvals")).unwrap();
    let installed =
        InstalledPlugRegistry::open(&base.join("install"), &base.join("records")).unwrap();

    let context = make_context(
        &lock_path,
        &quarantine_root,
        &scratch,
        &candidates,
        &exact_trust,
        &profiles,
        &conformance_store,
        &approvals,
        &installed,
    );
    let options = valid_options();
    let request = valid_request(&candidate.candidate_id);

    // Call 1: Create trust
    let r1 = execute_next_installation_action(&request, &context, &options).unwrap();
    assert_eq!(
        r1.before.action,
        InstallationPlanAction::CreateExactCandidateTrust
    );
    assert!(matches!(
        r1.outcome,
        InstallationStepOutcome::Advanced { .. }
    ));
    assert_eq!(
        r1.after.action,
        InstallationPlanAction::RunSupervisedConformance
    );
    let trust_digest = r1
        .after
        .exact_candidate_trust_record_digest
        .clone()
        .unwrap();
    let trust_evidence_digest = r1.after.trust_evidence_digest.clone().unwrap();
    assert_eq!(approvals.load_all().unwrap().len(), 0);
    assert_eq!(installed.load_all().unwrap().len(), 0);

    // Call 2: Run conformance (passed)
    let _before_snap = snapshot(&base);
    let r2 = execute_next_installation_action(&request, &context, &options).unwrap();
    assert_eq!(
        r2.before.action,
        InstallationPlanAction::RunSupervisedConformance
    );

    match &r2.outcome {
        InstallationStepOutcome::Advanced { executed } => {
            assert_eq!(*executed, InstallationPlanAction::RunSupervisedConformance);
        }
        other => panic!("expected Advanced, got {:?}", other),
    }

    assert_eq!(
        r2.after.action,
        InstallationPlanAction::CreateInstallationApproval
    );
    // Pins retained from before
    assert_eq!(
        r2.after.exact_candidate_trust_record_digest.as_deref(),
        Some(trust_digest.as_str())
    );
    assert_eq!(
        r2.after.trust_evidence_digest.as_deref(),
        Some(trust_evidence_digest.as_str())
    );
    // New pins
    let launch_digest = r2.after.launch_profile_evidence_digest.clone().unwrap();
    let conformance_id = r2.after.conformance_evidence_id.clone().unwrap();
    let conformance_digest = r2.after.conformance_evidence_digest.clone().unwrap();

    // Exactly one launch profile persisted
    let launch_records = profiles.load_all().unwrap();
    assert_eq!(launch_records.len(), 1);
    assert_eq!(launch_records[0].profile_evidence_digest, launch_digest);

    // Exactly one passed conformance persisted
    let conf_records = conformance_store.load_all().unwrap();
    assert_eq!(conf_records.len(), 1);
    assert_eq!(conf_records[0].disposition, ConformanceDisposition::Passed);
    assert_eq!(conf_records[0].evidence_id, conformance_id);
    assert_eq!(conf_records[0].evidence_digest, conformance_digest);

    assert_eq!(approvals.load_all().unwrap().len(), 0);
    assert_eq!(installed.load_all().unwrap().len(), 0);

    // Call 3: Create approval
    let r3 = execute_next_installation_action(&request, &context, &options).unwrap();
    assert_eq!(
        r3.before.action,
        InstallationPlanAction::CreateInstallationApproval
    );
    match &r3.outcome {
        InstallationStepOutcome::Advanced { executed } => {
            assert_eq!(
                *executed,
                InstallationPlanAction::CreateInstallationApproval
            );
        }
        other => panic!("expected Advanced, got {:?}", other),
    }
    assert_eq!(
        r3.after.action,
        InstallationPlanAction::PublishDisabledInstallation
    );
    // All previous pins retained
    assert_eq!(
        r3.after.exact_candidate_trust_record_digest.as_deref(),
        Some(trust_digest.as_str())
    );
    assert_eq!(
        r3.after.trust_evidence_digest.as_deref(),
        Some(trust_evidence_digest.as_str())
    );
    assert_eq!(
        r3.after.launch_profile_evidence_digest.as_deref(),
        Some(launch_digest.as_str())
    );
    assert_eq!(
        r3.after.conformance_evidence_id.as_deref(),
        Some(conformance_id.as_str())
    );
    assert_eq!(
        r3.after.conformance_evidence_digest.as_deref(),
        Some(conformance_digest.as_str())
    );
    // New pins
    let approval_id = r3.after.installation_approval_id.clone().unwrap();
    let approval_digest = r3.after.installation_approval_digest.clone().unwrap();

    let approval_records = approvals.load_all().unwrap();
    assert_eq!(approval_records.len(), 1);
    assert_eq!(approval_records[0].approval_id, approval_id);
    assert_eq!(approval_records[0].record_digest, approval_digest);

    assert_eq!(installed.load_all().unwrap().len(), 0);

    // Call 4: Deferred publication
    let deferred_roots = [
        base.join("profiles"),
        base.join("conformance"),
        base.join("approvals"),
        base.join("records"),
        base.join("install"),
        base.join("executor-state"),
        base.join("installation-intent"),
    ];
    let before_deferred = deferred_roots
        .iter()
        .map(|root| snapshot(root))
        .collect::<Vec<_>>();
    let r4 = execute_next_installation_action(&request, &context, &options);
    assert!(r4.is_err());
    assert_eq!(r4.unwrap_err().code, "installation_publication_deferred");

    // No evidence, installed record, staging, destination, intent, or executor
    // state changed. The persistent lock anchor is intentionally excluded.
    let after_deferred = deferred_roots
        .iter()
        .map(|root| snapshot(root))
        .collect::<Vec<_>>();
    assert_eq!(before_deferred, after_deferred);
    assert!(snapshot(&base.join("install"))
        .keys()
        .all(|path| !path.contains(".staging-") && !path.contains("plug-")));

    fs::remove_dir_all(&base).unwrap();
}

fn assert_failed_or_interrupted_executor_step(
    mode: &str,
    expected: ConformanceDisposition,
    interrupt: bool,
) {
    let base = temp_dir(mode);
    fs::create_dir_all(&base).unwrap();
    let (candidates, candidate, quarantine_root) = setup_m3_candidate(&base, mode, &[]);
    let lock_dir = base.join("lock");
    fs::create_dir_all(&lock_dir).unwrap();
    let lock_path = lock_dir.join("anchor.lock");
    let scratch = base.join("scratch");
    fs::create_dir_all(&scratch).unwrap();
    let exact_trust = ExactCandidateTrustStore::open(&base.join("trust")).unwrap();
    let profiles = LaunchProfileEvidenceStore::open(&base.join("profiles")).unwrap();
    let conformance_store = ConformanceEvidenceStore::open(&base.join("conformance")).unwrap();
    let approvals = InstallationApprovalStore::open(&base.join("approvals")).unwrap();
    let installed =
        InstalledPlugRegistry::open(&base.join("install"), &base.join("records")).unwrap();
    let context = make_context(
        &lock_path,
        &quarantine_root,
        &scratch,
        &candidates,
        &exact_trust,
        &profiles,
        &conformance_store,
        &approvals,
        &installed,
    );
    let request = valid_request(&candidate.candidate_id);
    let options = valid_options();

    let first = execute_next_installation_action(&request, &context, &options).unwrap();
    assert_eq!(
        first.before.action,
        InstallationPlanAction::CreateExactCandidateTrust
    );
    assert_eq!(
        first.after.action,
        InstallationPlanAction::RunSupervisedConformance
    );
    assert_eq!(exact_trust.load_all().unwrap().len(), 1);

    let scratch_before = snapshot(&scratch);
    let _reset = interrupt.then(InterruptionReset::set);
    let second = execute_next_installation_action(&request, &context, &options).unwrap();
    assert_eq!(
        second.before.action,
        InstallationPlanAction::RunSupervisedConformance
    );
    assert_eq!(
        second.after.action,
        InstallationPlanAction::RunSupervisedConformance
    );
    match second.outcome {
        InstallationStepOutcome::ConformanceRecordedWithoutAdvance {
            evidence_id,
            disposition,
        } => {
            assert_eq!(disposition, expected);
            let evidence = conformance_store
                .load_all()
                .unwrap()
                .into_iter()
                .find(|item| item.evidence_id == evidence_id)
                .expect("returned evidence must be durable");
            assert_eq!(evidence.disposition, expected);
            assert_eq!(evidence.retry_count, 0);
        }
        other => panic!("expected recorded conformance, got {other:?}"),
    }
    assert_eq!(profiles.load_all().unwrap().len(), 1);
    assert_eq!(conformance_store.load_all().unwrap().len(), 1);
    assert!(approvals.load_all().unwrap().is_empty());
    assert!(installed.load_all().unwrap().is_empty());
    assert!(snapshot(&base.join("install"))
        .keys()
        .all(|path| { !path.contains(".staging-") && !path.contains("plug-") }));
    assert_eq!(snapshot(&base.join("records")), BTreeMap::new());
    assert_eq!(snapshot(&scratch), scratch_before);
    fs::remove_dir_all(&base).unwrap();
}

#[test]
fn j24k2_failed_conformance_records_once_without_advance_or_retry() {
    let _serial = test_serial();
    assert_failed_or_interrupted_executor_step("malformed", ConformanceDisposition::Failed, false);
}

#[test]
fn j24k2_interrupted_conformance_records_once_without_advance_or_retry() {
    let _serial = test_serial();
    assert_failed_or_interrupted_executor_step("valid", ConformanceDisposition::Interrupted, true);
}

#[test]
fn j24k2_candidate_tampering_after_prepare_refuses_before_provider_marker() {
    let _serial = test_serial();
    let base = temp_dir("tamper-after-prepare");
    fs::create_dir_all(&base).unwrap();
    let marker = base.join("provider-created.marker");
    let (_candidates, candidate, quarantine_root) = setup_m3_candidate(
        &base,
        "valid",
        &[
            "--provider-marker".to_owned(),
            marker.to_string_lossy().into_owned(),
        ],
    );
    let publisher_store = PublisherTrustStore::open(&base.join("publisher-trust")).unwrap();
    let developer_store = DeveloperApprovalStore::open(&base.join("developer-approvals")).unwrap();
    let approval = developer_store
        .approve_exact_digest(&candidate.semantic_package_digest, "test")
        .unwrap();
    let trust = PackageTrustEvidence::unsigned(&approval).unwrap();
    let prepared = tethers_reference_host::launch_profile::PreparedSupervisedLaunch::prepare(
        &candidate,
        &quarantine_root,
        &base.join("scratch"),
        Duration::from_secs(3),
    )
    .unwrap();
    let payload = quarantine_root
        .join(&candidate.quarantine_relative_path)
        .join(&candidate.launch_path);
    let mut permissions = fs::metadata(&payload).unwrap().permissions();
    permissions.set_readonly(false);
    fs::set_permissions(&payload, permissions).unwrap();
    let mut bytes = fs::read(&payload).unwrap();
    bytes[0] ^= 1;
    fs::write(&payload, bytes).unwrap();
    let error = run_host_conformance(
        &prepared,
        &candidate,
        &quarantine_root,
        &trust,
        &publisher_store,
        &developer_store,
        "host-build",
    )
    .unwrap_err();
    assert_eq!(error.code, "candidate_drift");
    assert!(!marker.exists());
    prepared.cleanup_scratch().unwrap();
    fs::remove_dir_all(&base).unwrap();
}

// --- Post-plan failure and resumability ---

#[test]
fn j24k2_postplan_failure_resumable() {
    let _serial = test_serial();
    let base = temp_dir("postplan-fail");
    fs::create_dir_all(&base).unwrap();

    let (candidates, candidate, quarantine_root) = setup_candidate(&base);
    let lock_dir = base.join("lock");
    fs::create_dir_all(&lock_dir).unwrap();
    let lock_path = lock_dir.join("anchor.lock");
    let scratch = base.join("scratch");
    fs::create_dir_all(&scratch).unwrap();

    let exact_trust = ExactCandidateTrustStore::open(&base.join("trust")).unwrap();
    let profiles = LaunchProfileEvidenceStore::open(&base.join("profiles")).unwrap();
    let conformance_store = ConformanceEvidenceStore::open(&base.join("conformance")).unwrap();
    let approvals = InstallationApprovalStore::open(&base.join("approvals")).unwrap();
    let installed =
        InstalledPlugRegistry::open(&base.join("install"), &base.join("records")).unwrap();

    let request = valid_request(&candidate.candidate_id);
    let options = valid_options();
    let context = make_context(
        &lock_path,
        &quarantine_root,
        &scratch,
        &candidates,
        &exact_trust,
        &profiles,
        &conformance_store,
        &approvals,
        &installed,
    );

    // Inject a torn .tmp file into the launch_profiles store so that
    // replan fails after trust creation. Initial planning only reads
    // the trust store, not launch_profiles, so trust creation succeeds.
    let torn_path = base.join("profiles").join(".torn.tmp");
    fs::write(&torn_path, b"garbage").unwrap();

    let result = execute_next_installation_action(&request, &context, &options);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().code,
        "installation_execution_postcondition_failed"
    );

    // Trust must remain durably present
    assert_eq!(exact_trust.load_all().unwrap().len(), 1);

    // Lock was released (prove by re-calling)
    // First, remove the corruption
    fs::remove_file(&torn_path).unwrap();

    // Resumption must proceed from RunSupervisedConformance, not re-create trust
    let result2 = execute_next_installation_action(&request, &context, &options).unwrap();
    assert_eq!(
        result2.before.action,
        InstallationPlanAction::RunSupervisedConformance
    );

    fs::remove_dir_all(&base).unwrap();
}
