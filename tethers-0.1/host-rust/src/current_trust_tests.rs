use super::current_trust::{CurrentTrustAuthority, ExactCandidateTrustAuthority};
use crate::candidate::CandidateRecord;
use crate::conformance::{
    current_suite_digest, run_host_conformance_with_authority, CaseDisposition,
    ConformanceCaseEvidence, ConformanceDisposition, ConformanceEvidence,
};
use crate::installation_request::{
    InstallationConformanceRequest, InstallationRequest, InstallationTargetRequest,
    InstallationTargetState, InstallationTrustRequest, InstallationTrustScope,
    INSTALLATION_REQUEST_SCHEMA,
};
use crate::installation_trust::ExactCandidateTrustStore;
use crate::installed::{
    InstallationApprovalRecord, InstallationApprovalStore, InstalledPlugRegistry,
    ReviewedCapability,
};
use crate::launch_profile::{LaunchProfileEvidence, PreparedSupervisedLaunch};
use crate::m3_store::{canonical, sha256, M3Error, Result};
use crate::package::{CapabilityEvidence, PayloadEvidence, PlatformEvidence};
use crate::trust::{PackageTrustEvidence, TrustModeEvidence};
use std::cell::Cell;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use uuid::Uuid;

// ----------------------------------------------------------------
// crate-test-only authority types
// ----------------------------------------------------------------

struct RecordingAuthority {
    call_count: Cell<usize>,
}

impl RecordingAuthority {
    fn new() -> Self {
        Self {
            call_count: Cell::new(0),
        }
    }

    fn call_count(&self) -> usize {
        self.call_count.get()
    }

    fn sentinel() -> M3Error {
        M3Error::new("j24k1_authority_sentinel", "recording authority sentinel")
    }
}

impl CurrentTrustAuthority for RecordingAuthority {
    fn revalidate_current(
        &self,
        _candidate: &CandidateRecord,
        _evidence: &PackageTrustEvidence,
        _now_unix_ms: u64,
    ) -> Result<()> {
        self.call_count.set(self.call_count.get() + 1);
        Err(Self::sentinel())
    }
}

struct FailOnNthAuthority {
    call_count: Cell<usize>,
    fail_on: usize,
}

impl FailOnNthAuthority {
    fn new(fail_on: usize) -> Self {
        Self {
            call_count: Cell::new(0),
            fail_on,
        }
    }

    fn call_count(&self) -> usize {
        self.call_count.get()
    }

    fn sentinel() -> M3Error {
        M3Error::new("j24k1_authority_sentinel", "fail-on-nth authority sentinel")
    }
}

impl CurrentTrustAuthority for FailOnNthAuthority {
    fn revalidate_current(
        &self,
        _candidate: &CandidateRecord,
        _evidence: &PackageTrustEvidence,
        _now_unix_ms: u64,
    ) -> Result<()> {
        let n = self.call_count.get() + 1;
        self.call_count.set(n);
        if n >= self.fail_on {
            Err(Self::sentinel())
        } else {
            Ok(())
        }
    }
}

// ----------------------------------------------------------------
// fixture helpers
// ----------------------------------------------------------------

fn write_read_only(path: &Path, bytes: &[u8]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, bytes).unwrap();
    let mut perms = fs::metadata(path).unwrap().permissions();
    perms.set_readonly(true);
    fs::set_permissions(path, perms).unwrap();
}

fn manifest_bytes() -> &'static [u8] {
    include_bytes!("../../protocol/capability-manifests/fixture-ping.json")
}

fn quarantine_fixture() -> (CandidateRecord, PathBuf, PathBuf) {
    let base = std::env::temp_dir().join(format!("tethers-j24k1-behavior-{}", Uuid::new_v4()));
    fs::create_dir_all(&base).unwrap();
    let quarantine_root = base.join("quarantine");
    let candidate_relative = format!("candidate-{}", Uuid::new_v4());
    let candidate_dir = quarantine_root.join(&candidate_relative);
    fs::create_dir_all(candidate_dir.join("provider")).unwrap();
    fs::create_dir_all(candidate_dir.join("manifests")).unwrap();

    let exe = b"j24k1-dummy-provider-binary-content-64bytes-padding-xxxxxxxxxxxxxxx";
    let manifest = manifest_bytes();

    write_read_only(&candidate_dir.join("provider/tool.exe"), exe);
    write_read_only(&candidate_dir.join("manifests/read.json"), manifest);

    let plug_value = serde_json::json!({
        "package_format_version":"1",
        "package_id":"tethers.behavioral",
        "package_version":"0.1.0",
        "display_name":"J24K1 Behavioral Fixture",
        "description":"Minimal fixture for authority propagation tests",
        "publisher":"Test",
        "licence":"MIT",
        "socket_major":1,
        "protocol_bindings":[{"protocol":"MCP","version":"2025-11-25","transport":"stdio"}],
        "platforms":[{"os":"windows","architecture":"x86_64"}],
        "provider":{
            "provider_id":"tethers.behavioral",
            "provider_version":"0.1.0",
            "launch":{"path":"provider/tool.exe","arguments":["--serve"]},
            "working_directory":"provider",
            "capability_operation_namespace":"fixture"
        },
        "capabilities":[{
            "capability_name":"fixture.ping",
            "capability_version":1,
            "manifest_path":"manifests/read.json",
            "manifest_digest":"sha256:01fed7a4b877dd82abe91a1b6cfcd476b02e4c115489e70cbb285b8bf2d32d8b",
            "provider_operation_name":"fixture_ping"
        }],
        "payload_index":[
            {"path":"manifests/read.json","sha256":sha256(manifest),"size_bytes":manifest.len(),"role":"capability_manifest"},
            {"path":"provider/tool.exe","sha256":sha256(exe),"size_bytes":exe.len(),"role":"provider_executable"}
        ]
    });
    let plug_json = canonical(&plug_value).unwrap();
    write_read_only(&candidate_dir.join("plug.json"), &plug_json);

    let candidate_id = Uuid::new_v4().to_string();
    let semantic_digest = sha256(&plug_json);
    let manifest_digest = "sha256:01fed7a4b877dd82abe91a1b6cfcd476b02e4c115489e70cbb285b8bf2d32d8b";

    let mut record = CandidateRecord {
        schema_version: 1,
        candidate_id: candidate_id.clone(),
        state: "quarantined_installation_candidate".into(),
        package_id: "tethers.behavioral".into(),
        package_version: "0.1.0".into(),
        semantic_package_digest: semantic_digest.clone(),
        raw_archive_digest: format!("sha256:{}", "a".repeat(64)),
        source_size_bytes: 1234,
        quarantine_relative_path: candidate_relative,
        provider_id: "tethers.behavioral".into(),
        provider_version: "0.1.0".into(),
        launch_path: "provider/tool.exe".into(),
        launch_arguments: vec!["--serve".into()],
        provider_working_directory: "provider".into(),
        capability_operation_namespace: "fixture".into(),
        selected_platform: PlatformEvidence {
            os: "windows".into(),
            architecture: "x86_64".into(),
        },
        plug_json: PayloadEvidence {
            path: "plug.json".into(),
            sha256: semantic_digest.clone(),
            size_bytes: plug_json.len() as u64,
            role: "package_descriptor".into(),
        },
        payloads: vec![
            PayloadEvidence {
                path: "manifests/read.json".into(),
                sha256: sha256(manifest),
                size_bytes: manifest.len() as u64,
                role: "capability_manifest".into(),
            },
            PayloadEvidence {
                path: "provider/tool.exe".into(),
                sha256: sha256(exe),
                size_bytes: exe.len() as u64,
                role: "provider_executable".into(),
            },
        ],
        signature_files: vec![],
        capabilities: vec![CapabilityEvidence {
            name: "fixture.ping".into(),
            version: 1,
            operation: "fixture_ping".into(),
            manifest_path: "manifests/read.json".into(),
            manifest_digest: manifest_digest.into(),
        }],
        signatures_present: false,
        inspection_report_format_version: 1,
        inspection_evidence_digest: format!("sha256:{}", "b".repeat(64)),
        operational_scope_schema: None,
        operational_scope_schema_digest: None,
        created_unix_ms: 1,
        record_digest: String::new(),
    };
    record.record_digest = sha256(&canonical(&record).unwrap());
    record.validate().unwrap();

    (record, quarantine_root, base)
}

fn unsigned_evidence(candidate: &CandidateRecord) -> PackageTrustEvidence {
    let mut evidence = PackageTrustEvidence {
        evidence_format_version: 1,
        semantic_package_digest: candidate.semantic_package_digest.clone(),
        mode: TrustModeEvidence::UnsignedDeveloper {
            approval_id: Uuid::new_v4().to_string(),
            approval_record_digest: format!("sha256:{}", "1".repeat(64)),
            visibly_unsigned: true,
        },
        evidence_digest: String::new(),
    };
    evidence.evidence_digest = sha256(&canonical(&evidence).unwrap());
    evidence
}

fn prepared_fixture(
    candidate: &CandidateRecord,
    quarantine_root: &Path,
) -> (PreparedSupervisedLaunch, PathBuf) {
    let scratch = std::env::temp_dir().join(format!("tethers-j24k1-scratch-{}", Uuid::new_v4()));
    let prepared = PreparedSupervisedLaunch::prepare(
        candidate,
        quarantine_root,
        &scratch,
        Duration::from_secs(3),
    )
    .unwrap();
    (prepared, scratch)
}

fn all_passed_cases() -> Vec<ConformanceCaseEvidence> {
    vec![
        "static_candidate_revalidation",
        "exact_launch_clean_environment",
        "mcp_initialize_protocol_pin",
        "provider_identity",
        "complete_discovery_exact_operations",
        "bounded_valid_fixture_call",
        "invalid_fixture_call_refused",
        "bounded_shutdown_process_cleanup",
    ]
    .into_iter()
    .map(|id| ConformanceCaseEvidence {
        case_id: id.into(),
        disposition: CaseDisposition::Passed,
        safe_diagnostic_code: None,
    })
    .collect()
}

fn build_conformance(
    candidate: &CandidateRecord,
    trust: &PackageTrustEvidence,
    launch: &LaunchProfileEvidence,
    suite_digest: &str,
) -> ConformanceEvidence {
    let mut evidence = ConformanceEvidence {
        schema_version: 1,
        evidence_id: Uuid::new_v4().to_string(),
        candidate_id: candidate.candidate_id.clone(),
        package_id: candidate.package_id.clone(),
        package_version: candidate.package_version.clone(),
        semantic_package_digest: candidate.semantic_package_digest.clone(),
        payloads: candidate.payloads.clone(),
        capabilities: candidate.capabilities.clone(),
        trust_evidence_digest: trust.evidence_digest.clone(),
        launch_profile_evidence_digest: launch.profile_evidence_digest.clone(),
        launch_profile_label: launch.profile_label.clone(),
        provider_id: candidate.provider_id.clone(),
        provider_version: candidate.provider_version.clone(),
        socket_major: 1,
        mcp_protocol_version: "2025-11-25".into(),
        binding_version: "mcp-stdio-2025-11-25".into(),
        host_build_identity: "test-build".into(),
        platform: "windows".into(),
        architecture: "x86_64".into(),
        suite_version: "m3-generic-1".into(),
        suite_digest: suite_digest.into(),
        test_configuration_digest: format!("sha256:{}", "d".repeat(64)),
        started_unix_ms: 1000,
        ended_unix_ms: 2000,
        cases: all_passed_cases(),
        disposition: ConformanceDisposition::Passed,
        retry_count: 0,
        raw_stderr_persisted: false,
        evidence_digest: String::new(),
    };
    let covered = {
        let mut copy = evidence.clone();
        copy.evidence_digest.clear();
        canonical(&copy).unwrap()
    };
    evidence.evidence_digest = sha256(&covered);
    evidence.validate().unwrap();
    evidence
}

fn build_approval(
    candidate: &CandidateRecord,
    trust: &PackageTrustEvidence,
    launch: &LaunchProfileEvidence,
    conformance: &ConformanceEvidence,
) -> InstallationApprovalRecord {
    let reviewed = candidate
        .capabilities
        .iter()
        .map(|cap| ReviewedCapability {
            capability_name: cap.name.clone(),
            capability_version: cap.version,
            manifest_digest: cap.manifest_digest.clone(),
            provider_operation_name: cap.operation.clone(),
            effects: vec![],
            permission_scope: serde_json::Value::Object(Default::default()),
            permission_scope_digest: format!("sha256:{}", "e".repeat(64)),
        })
        .collect();
    let mut record = InstallationApprovalRecord {
        schema_version: 1,
        approval_id: Uuid::new_v4().to_string(),
        candidate_id: candidate.candidate_id.clone(),
        package_id: candidate.package_id.clone(),
        package_version: candidate.package_version.clone(),
        semantic_package_digest: candidate.semantic_package_digest.clone(),
        raw_archive_digest: candidate.raw_archive_digest.clone(),
        source_size_bytes: candidate.source_size_bytes,
        payloads: candidate.payloads.clone(),
        reviewed_capabilities: reviewed,
        trust_evidence: trust.clone(),
        provider_id: candidate.provider_id.clone(),
        provider_version: candidate.provider_version.clone(),
        launch_path: candidate.launch_path.clone(),
        launch_arguments: candidate.launch_arguments.clone(),
        provider_working_directory: candidate.provider_working_directory.clone(),
        launch_profile_label: launch.profile_label.clone(),
        launch_profile_limitation: launch.limitation.clone(),
        launch_profile_evidence_digest: launch.profile_evidence_digest.clone(),
        conformance_evidence_id: conformance.evidence_id.clone(),
        conformance_evidence_digest: conformance.evidence_digest.clone(),
        approving_authority: "test".into(),
        approved_unix_ms: 1000,
        record_digest: String::new(),
    };
    record.record_digest = sha256(&canonical(&record).unwrap());
    record.validate().unwrap();
    record
}

// ----------------------------------------------------------------
// existing exact-candidate authority tests (preserved)
// ----------------------------------------------------------------

fn candidate() -> CandidateRecord {
    serde_json::from_str(include_str!("../fixtures/m2/candidate-record-v1.json")).unwrap()
}

fn request(candidate_id: &str) -> InstallationRequest {
    InstallationRequest {
        schema: INSTALLATION_REQUEST_SCHEMA.into(),
        candidate_id: candidate_id.into(),
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

fn root(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("tethers-j24k1-{name}-{}", Uuid::new_v4()))
}

fn unsigned_evidence_exact(candidate: &CandidateRecord) -> PackageTrustEvidence {
    let mut evidence = PackageTrustEvidence {
        evidence_format_version: 1,
        semantic_package_digest: candidate.semantic_package_digest.clone(),
        mode: TrustModeEvidence::UnsignedDeveloper {
            approval_id: Uuid::new_v4().to_string(),
            approval_record_digest: format!("sha256:{}", "1".repeat(64)),
            visibly_unsigned: true,
        },
        evidence_digest: String::new(),
    };
    evidence.evidence_digest = sha256(&canonical(&evidence).unwrap());
    evidence
}

#[test]
fn j24k1_exact_authority_accepts_matching_current_record() {
    let path = root("matching");
    let store = ExactCandidateTrustStore::open(&path).unwrap();
    let candidate = candidate();
    let record = store
        .create(&candidate, &request(&candidate.candidate_id), "authority-a")
        .unwrap();
    let evidence = PackageTrustEvidence::exact_candidate(&record).unwrap();

    ExactCandidateTrustAuthority::new(&store)
        .revalidate_current(&candidate, &evidence, 0)
        .unwrap();
    fs::remove_dir_all(path).unwrap();
}

#[test]
fn j24k1_exact_authority_rejects_changed_current_record() {
    let path = root("changed");
    let store = ExactCandidateTrustStore::open(&path).unwrap();
    let candidate = candidate();
    let original = store
        .create(&candidate, &request(&candidate.candidate_id), "authority-a")
        .unwrap();
    let evidence = PackageTrustEvidence::exact_candidate(&original).unwrap();
    let mut current = original.clone();
    current.approving_authority = "authority-b".into();
    current.record_digest.clear();
    current.record_digest = sha256(&canonical(&current).unwrap());
    fs::write(
        path.join(format!("{}.json", candidate.candidate_id)),
        serde_json::to_vec(&current).unwrap(),
    )
    .unwrap();

    let error = ExactCandidateTrustAuthority::new(&store)
        .revalidate_current(&candidate, &evidence, 0)
        .unwrap_err();
    assert_eq!(error.code, "trust_drift");
    assert_eq!(error.message, "exact-candidate installation trust changed");
    fs::remove_dir_all(path).unwrap();
}

#[test]
fn j24k1_exact_authority_rejects_absent_wrong_mode_and_corrupt_store() {
    let candidate = candidate();
    let absent_path = root("absent");
    let absent = ExactCandidateTrustStore::open(&absent_path).unwrap();
    let record_path = root("wrong-mode");
    let wrong_mode_store = ExactCandidateTrustStore::open(&record_path).unwrap();
    let record = wrong_mode_store
        .create(&candidate, &request(&candidate.candidate_id), "authority-a")
        .unwrap();
    let error = ExactCandidateTrustAuthority::new(&absent)
        .revalidate_current(
            &candidate,
            &PackageTrustEvidence::exact_candidate(&record).unwrap(),
            0,
        )
        .unwrap_err();
    assert_eq!(error.code, "trust_drift");
    assert_eq!(
        error.message,
        "exact-candidate installation trust is absent"
    );

    let error = ExactCandidateTrustAuthority::new(&wrong_mode_store)
        .revalidate_current(&candidate, &unsigned_evidence_exact(&candidate), 0)
        .unwrap_err();
    assert_eq!(error.code, "trust_exact_candidate_authority_required");

    fs::write(record_path.join("broken.tmp"), b"partial").unwrap();
    let error = ExactCandidateTrustAuthority::new(&wrong_mode_store)
        .revalidate_current(
            &candidate,
            &PackageTrustEvidence::exact_candidate(&record).unwrap(),
            0,
        )
        .unwrap_err();
    assert_eq!(error.code, "installation_trust_invalid");
    fs::remove_dir_all(absent_path).unwrap();
    fs::remove_dir_all(record_path).unwrap();
}

// ----------------------------------------------------------------
// behavioral propagation tests
// ----------------------------------------------------------------

#[test]
fn j24k1_prepared_launch_revalidate_current_trust_with_uses_supplied_authority() {
    let (candidate, quarantine_root, base) = quarantine_fixture();
    let (prepared, scratch) = prepared_fixture(&candidate, &quarantine_root);
    let trust = unsigned_evidence(&candidate);
    let authority = RecordingAuthority::new();

    let error = prepared
        .revalidate_current_trust_with(&candidate, &trust, &authority)
        .unwrap_err();

    assert_eq!(authority.call_count(), 1);
    assert_eq!(error.code, "j24k1_authority_sentinel");
    assert_eq!(error.message, "recording authority sentinel");
    let _ = prepared.cleanup_scratch();
    fs::remove_dir_all(scratch).ok();
    fs::remove_dir_all(base).ok();
}

#[test]
fn j24k1_prepared_launch_launch_for_candidate_with_refuses_before_launch() {
    let (candidate, quarantine_root, base) = quarantine_fixture();
    let (prepared, scratch) = prepared_fixture(&candidate, &quarantine_root);
    let trust = unsigned_evidence(&candidate);
    let authority = RecordingAuthority::new();

    let error = prepared
        .launch_for_candidate_with(&candidate, &trust, &authority)
        .unwrap_err();

    assert_eq!(authority.call_count(), 1);
    let message = format!("{}", error);
    assert!(
        message.contains("j24k1_authority_sentinel"),
        "sentinel error was not propagated: {message}"
    );
    let _ = prepared.cleanup_scratch();
    fs::remove_dir_all(scratch).ok();
    fs::remove_dir_all(base).ok();
}

#[test]
fn j24k1_run_host_conformance_with_authority_uses_supplied_authority() {
    let (candidate, quarantine_root, base) = quarantine_fixture();
    let (prepared, scratch) = prepared_fixture(&candidate, &quarantine_root);
    let trust = unsigned_evidence(&candidate);
    let authority = RecordingAuthority::new();

    let error = run_host_conformance_with_authority(
        &prepared,
        &candidate,
        &quarantine_root,
        &trust,
        &authority,
        "test",
    )
    .unwrap_err();

    assert_eq!(authority.call_count(), 1);
    assert_eq!(error.code, "j24k1_authority_sentinel");
    let _ = prepared.cleanup_scratch();
    fs::remove_dir_all(scratch).ok();
    fs::remove_dir_all(base).ok();
}

#[test]
fn j24k1_approve_with_authority_uses_and_propagates_supplied_authority() {
    let (candidate, quarantine_root, base) = quarantine_fixture();
    let (prepared, scratch) = prepared_fixture(&candidate, &quarantine_root);
    let trust = unsigned_evidence(&candidate);
    let suite_digest = current_suite_digest().unwrap();
    let conformance = build_conformance(&candidate, &trust, &prepared.evidence, &suite_digest);
    let authority = RecordingAuthority::new();
    let approval_store = InstallationApprovalStore::open(&base.join("approvals")).unwrap();

    let error = approval_store
        .approve_with_authority(
            &candidate,
            &quarantine_root,
            &trust,
            &authority,
            &prepared.evidence,
            &conformance,
            "test",
        )
        .unwrap_err();

    assert_eq!(authority.call_count(), 1);
    assert_eq!(error.code, "j24k1_authority_sentinel");
    let _ = prepared.cleanup_scratch();
    fs::remove_dir_all(scratch).ok();
    fs::remove_dir_all(base).ok();
}

#[test]
fn j24k1_install_disabled_with_authority_uses_supplied_authority_at_entry() {
    let (candidate, quarantine_root, base) = quarantine_fixture();
    let (prepared, scratch) = prepared_fixture(&candidate, &quarantine_root);
    let trust = unsigned_evidence(&candidate);
    let suite_digest = current_suite_digest().unwrap();
    let conformance = build_conformance(&candidate, &trust, &prepared.evidence, &suite_digest);
    let approval = build_approval(&candidate, &trust, &prepared.evidence, &conformance);
    let authority = RecordingAuthority::new();
    let registry =
        InstalledPlugRegistry::open(&base.join("install"), &base.join("records")).unwrap();

    let error = registry
        .install_disabled_with_authority(
            &candidate,
            &quarantine_root,
            &trust,
            &authority,
            &prepared.evidence,
            &conformance,
            &approval,
        )
        .unwrap_err();

    assert_eq!(authority.call_count(), 1);
    assert_eq!(error.code, "j24k1_authority_sentinel");
    let _ = prepared.cleanup_scratch();
    fs::remove_dir_all(scratch).ok();
    fs::remove_dir_all(base).ok();
}

#[test]
fn j24k1_install_disabled_invokes_authority_again_after_staging_before_publication() {
    let (candidate, quarantine_root, base) = quarantine_fixture();
    let (prepared, scratch) = prepared_fixture(&candidate, &quarantine_root);
    let trust = unsigned_evidence(&candidate);
    let suite_digest = current_suite_digest().unwrap();
    let conformance = build_conformance(&candidate, &trust, &prepared.evidence, &suite_digest);
    let approval = build_approval(&candidate, &trust, &prepared.evidence, &conformance);
    let authority = FailOnNthAuthority::new(2);
    let registry =
        InstalledPlugRegistry::open(&base.join("install"), &base.join("records")).unwrap();

    let error = registry
        .install_disabled_with_authority(
            &candidate,
            &quarantine_root,
            &trust,
            &authority,
            &prepared.evidence,
            &conformance,
            &approval,
        )
        .unwrap_err();

    assert_eq!(authority.call_count(), 2);
    assert_eq!(error.code, "j24k1_authority_sentinel");

    let records = registry.load_all().unwrap();
    assert!(
        records.is_empty(),
        "no installed record should be published"
    );

    let install_dir = base.join("install");
    let has_plug_dest = fs::read_dir(&install_dir).map_or(false, |entries| {
        entries
            .filter_map(|e| e.ok())
            .any(|e| e.file_name().to_string_lossy().starts_with("plug-"))
    });
    assert!(!has_plug_dest, "no final plug-* destination should remain");

    let has_staging = fs::read_dir(&install_dir).map_or(false, |entries| {
        entries
            .filter_map(|e| e.ok())
            .any(|e| e.file_name().to_string_lossy().starts_with(".staging-"))
    });
    assert!(
        !has_staging,
        "staging directory should be cleaned up after final revalidation failure"
    );

    let _ = prepared.cleanup_scratch();
    fs::remove_dir_all(scratch).ok();
    fs::remove_dir_all(base).ok();
}
