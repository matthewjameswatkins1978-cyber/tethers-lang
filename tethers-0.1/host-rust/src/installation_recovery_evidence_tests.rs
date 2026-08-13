use super::installation_recovery_evidence::{
    revalidate_installation_recovery_evidence, InstallationRecoveryEvidenceContext,
};
use crate::candidate::{extract_to_quarantine, CandidateRegistry};
use crate::conformance::{
    current_suite_digest, CaseDisposition, ConformanceCaseEvidence, ConformanceDisposition,
    ConformanceEvidence, ConformanceEvidenceStore,
};
use crate::current_trust::ExactCandidateTrustAuthority;
use crate::installation_publication_intent::InstallationPublicationIntent;
use crate::installation_request::{
    InstallationConformanceRequest, InstallationRequest, InstallationTargetRequest,
    InstallationTargetState, InstallationTrustRequest, InstallationTrustScope,
    INSTALLATION_REQUEST_SCHEMA,
};
use crate::installation_trust::ExactCandidateTrustStore;
use crate::installed::{DisabledBindingRecord, InstallationApprovalStore, InstalledPlugRecord};
use crate::launch_profile::{
    LaunchProfileEvidence, LaunchProfileEvidenceStore, PreparedSupervisedLaunch,
};
use crate::m3_store::{canonical, sha256};
use crate::package;
use crate::trust::{PackageTrustEvidence, TrustModeEvidence};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use uuid::Uuid;

fn complete_request(candidate_id: &str) -> InstallationRequest {
    InstallationRequest {
        schema: INSTALLATION_REQUEST_SCHEMA.to_owned(),
        candidate_id: candidate_id.to_owned(),
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

fn build_conformance(
    candidate: &crate::candidate::CandidateRecord,
    trust: &PackageTrustEvidence,
    launch: &LaunchProfileEvidence,
) -> ConformanceEvidence {
    let cases = [
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
    .map(|case_id| ConformanceCaseEvidence {
        case_id: case_id.to_owned(),
        disposition: CaseDisposition::Passed,
        safe_diagnostic_code: None,
    })
    .collect();
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
        mcp_protocol_version: "2025-11-25".to_owned(),
        binding_version: "mcp-stdio-2025-11-25".to_owned(),
        host_build_identity: "j24k3c3-test".to_owned(),
        platform: candidate.selected_platform.os.clone(),
        architecture: candidate.selected_platform.architecture.clone(),
        suite_version: "m3-generic-1".to_owned(),
        suite_digest: current_suite_digest().unwrap(),
        test_configuration_digest: launch.environment_digest.clone(),
        started_unix_ms: 1,
        ended_unix_ms: 2,
        cases,
        disposition: ConformanceDisposition::Passed,
        retry_count: 0,
        raw_stderr_persisted: false,
        evidence_digest: String::new(),
    };
    let mut covered = evidence.clone();
    covered.evidence_digest.clear();
    evidence.evidence_digest = sha256(&canonical(&covered).unwrap());
    evidence.validate().unwrap();
    evidence
}

fn recompute_intent(intent: &mut InstallationPublicationIntent) {
    let mut covered_record = intent.installed_record.clone();
    covered_record.record_digest.clear();
    intent.installed_record.record_digest = sha256(&canonical(&covered_record).unwrap());
    intent.installed_record_digest = intent.installed_record.record_digest.clone();
    let mut covered_intent = intent.clone();
    covered_intent.intent_digest.clear();
    intent.intent_digest = sha256(&canonical(&covered_intent).unwrap());
    intent.validate().unwrap();
}

fn make_writable(path: &Path) {
    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_readonly(false);
    fs::set_permissions(path, permissions).unwrap();
}

fn build_installed_record(
    candidate: &crate::candidate::CandidateRecord,
    trust: &PackageTrustEvidence,
    launch: &LaunchProfileEvidence,
    conformance: &ConformanceEvidence,
    approval: &crate::installed::InstallationApprovalRecord,
) -> InstalledPlugRecord {
    let installed_id = Uuid::new_v4().to_string();
    let installation_relative_path = format!("plug-{installed_id}");
    let disabled_bindings: Vec<DisabledBindingRecord> = candidate
        .capabilities
        .iter()
        .map(|capability| DisabledBindingRecord {
            state: "disabled".into(),
            capability_name: capability.name.clone(),
            capability_version: capability.version,
            manifest_digest: capability.manifest_digest.clone(),
            provider_operation_name: capability.operation.clone(),
        })
        .collect();
    let mut record = InstalledPlugRecord {
        schema_version: 1,
        installed_id,
        state: "present_disabled".into(),
        package_id: candidate.package_id.clone(),
        package_version: candidate.package_version.clone(),
        semantic_package_digest: candidate.semantic_package_digest.clone(),
        source_candidate_id: candidate.candidate_id.clone(),
        installation_relative_path,
        raw_archive_digest: candidate.raw_archive_digest.clone(),
        plug_json: candidate.plug_json.clone(),
        payloads: candidate.payloads.clone(),
        signature_files: candidate.signature_files.clone(),
        capability_manifests: candidate.capabilities.clone(),
        trust_evidence: trust.clone(),
        installation_approval_id: approval.approval_id.clone(),
        installation_approval_digest: approval.record_digest.clone(),
        conformance_evidence_id: conformance.evidence_id.clone(),
        conformance_evidence_digest: conformance.evidence_digest.clone(),
        provider_id: candidate.provider_id.clone(),
        provider_version: candidate.provider_version.clone(),
        launch_path: candidate.launch_path.clone(),
        launch_arguments: candidate.launch_arguments.clone(),
        provider_working_directory: candidate.provider_working_directory.clone(),
        launch_profile_label: launch.profile_label.clone(),
        socket_major: 1,
        mcp_protocol_version: "2025-11-25".into(),
        platform: candidate.selected_platform.os.clone(),
        architecture: candidate.selected_platform.architecture.clone(),
        disabled_bindings,
        operational_scope_schema: candidate.operational_scope_schema.clone(),
        operational_scope_schema_digest: candidate.operational_scope_schema_digest.clone(),
        created_unix_ms: 1,
        record_digest: String::new(),
    };
    let mut covered = record.clone();
    covered.record_digest.clear();
    record.record_digest = sha256(&canonical(&covered).unwrap());
    record.validate().unwrap();
    record
}

struct RecoveryFixture {
    base: PathBuf,
    quarantine_root: PathBuf,
    candidates: CandidateRegistry,
    exact_trust: ExactCandidateTrustStore,
    launch_profiles: LaunchProfileEvidenceStore,
    conformance: ConformanceEvidenceStore,
    approvals: InstallationApprovalStore,
    request: InstallationRequest,
    intent: InstallationPublicationIntent,
    candidate: crate::candidate::CandidateRecord,
    trust: PackageTrustEvidence,
    launch: LaunchProfileEvidence,
    conformance_evidence: ConformanceEvidence,
    approval: crate::installed::InstallationApprovalRecord,
}

impl RecoveryFixture {
    fn new() -> Self {
        let base = std::env::temp_dir().join(format!("tethers-j24k3c3-{}", Uuid::new_v4()));
        fs::create_dir_all(&base).unwrap();
        let archive = base.join("test.tetherplug");
        fs::write(
            &archive,
            crate::test_fixture_package::build_fixture_package(b"j24k3c3-test").unwrap(),
        )
        .unwrap();
        let quarantine_root = base.join("quarantine");
        let report = package::inspect(&archive).unwrap();
        let quarantined = extract_to_quarantine(&report, &quarantine_root).unwrap();
        let candidates =
            CandidateRegistry::open(&base.join("candidates"), &quarantine_root).unwrap();
        let candidate = candidates.create(&quarantined).unwrap();

        let request = complete_request(&candidate.candidate_id);

        let exact_trust = ExactCandidateTrustStore::open(&base.join("trust")).unwrap();
        let trust_record = exact_trust
            .create(&candidate, &request, "j24k3c3-authority")
            .unwrap();
        let trust = PackageTrustEvidence::exact_candidate(&trust_record).unwrap();

        let prepared = PreparedSupervisedLaunch::prepare(
            &candidate,
            &quarantine_root,
            &base.join("scratch"),
            Duration::from_secs(3),
        )
        .unwrap();
        let launch = prepared.evidence.clone();
        let launch_profiles = LaunchProfileEvidenceStore::open(&base.join("profiles")).unwrap();
        launch_profiles.create(&launch).unwrap();
        prepared.cleanup_scratch().unwrap();

        let conformance_evidence = build_conformance(&candidate, &trust, &launch);
        let conformance = ConformanceEvidenceStore::open(&base.join("conformance")).unwrap();
        conformance.create(&conformance_evidence).unwrap();

        let authority = ExactCandidateTrustAuthority::new(&exact_trust);
        let approvals = InstallationApprovalStore::open(&base.join("approvals")).unwrap();
        let approval = approvals
            .approve_with_authority(
                &candidate,
                &quarantine_root,
                &trust,
                &authority,
                &launch,
                &conformance_evidence,
                "j24k3c3-authority",
            )
            .unwrap();

        let installed_record = build_installed_record(
            &candidate,
            &trust,
            &launch,
            &conformance_evidence,
            &approval,
        );
        let intent =
            InstallationPublicationIntent::from_precomputed_record(installed_record).unwrap();

        Self {
            base,
            quarantine_root,
            candidates,
            exact_trust,
            launch_profiles,
            conformance,
            approvals,
            request,
            intent,
            candidate,
            trust,
            launch,
            conformance_evidence,
            approval,
        }
    }

    fn context(&self) -> InstallationRecoveryEvidenceContext<'_> {
        InstallationRecoveryEvidenceContext {
            quarantine_root: &self.quarantine_root,
            candidates: &self.candidates,
            exact_trust: &self.exact_trust,
            launch_profiles: &self.launch_profiles,
            conformance: &self.conformance,
            approvals: &self.approvals,
        }
    }

    fn revalidate(&self) -> crate::m3_store::Result<()> {
        revalidate_installation_recovery_evidence(&self.request, &self.intent, &self.context())
    }
}

impl Drop for RecoveryFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.base);
    }
}

#[derive(Debug, PartialEq, Eq)]
enum SnapshotEntry {
    Directory,
    File {
        hash: String,
        modified_unix_ms: u128,
        readonly: bool,
    },
}

fn tree_snapshot(root: &Path) -> std::collections::BTreeMap<String, SnapshotEntry> {
    fn visit(
        root: &Path,
        path: &Path,
        output: &mut std::collections::BTreeMap<String, SnapshotEntry>,
    ) {
        if !path.is_dir() {
            return;
        }
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let relative = path
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            if path.is_dir() {
                output.insert(relative, SnapshotEntry::Directory);
                visit(root, &path, output);
            } else {
                let metadata = path.metadata().unwrap();
                output.insert(
                    relative,
                    SnapshotEntry::File {
                        hash: sha256(&fs::read(&path).unwrap()),
                        modified_unix_ms: metadata
                            .modified()
                            .unwrap()
                            .duration_since(std::time::SystemTime::UNIX_EPOCH)
                            .unwrap()
                            .as_millis(),
                        readonly: metadata.permissions().readonly(),
                    },
                );
            }
        }
    }
    let mut output = std::collections::BTreeMap::new();
    if root.exists() {
        visit(root, root, &mut output);
    }
    output
}

#[test]
fn j24k3c3_valid_chain_passes() {
    let fixture = RecoveryFixture::new();
    fixture.revalidate().unwrap();
}

#[test]
fn j24k3c3_invalid_intent_rejected_before_request_or_store() {
    let mut fixture = RecoveryFixture::new();
    fixture.intent.schema_version = 0;
    let err = fixture.revalidate().unwrap_err();
    assert_eq!(err.code, "installation_intent_invalid");
}

#[test]
fn j24k3c3_invalid_schema_fails_stale() {
    let mut fixture = RecoveryFixture::new();
    fixture.request.schema = "other".into();
    let err = fixture.revalidate().unwrap_err();
    assert_eq!(err.code, "installation_intent_evidence_stale");
}

#[test]
fn j24k3c3_candidate_mismatch_fails_stale() {
    let mut fixture = RecoveryFixture::new();
    fixture.request.candidate_id = Uuid::new_v4().to_string();
    let err = fixture.revalidate().unwrap_err();
    assert_eq!(err.code, "installation_intent_evidence_stale");
}

#[test]
fn j24k3c3_exact_candidate_trust_scope_passes_validation() {
    let fixture = RecoveryFixture::new();
    assert!(matches!(
        fixture.request.trust.scope,
        InstallationTrustScope::ExactCandidate
    ));
    fixture.revalidate().unwrap();
}

#[test]
fn j24k3c3_missing_non_isolated_consent_fails_stale() {
    let mut fixture = RecoveryFixture::new();
    fixture
        .request
        .conformance
        .allow_non_isolated_supervised_execution = false;
    let err = fixture.revalidate().unwrap_err();
    assert_eq!(err.code, "installation_intent_evidence_stale");
}

#[test]
fn j24k3c3_disabled_target_state_passes_validation() {
    let fixture = RecoveryFixture::new();
    assert!(matches!(
        fixture.request.installation.target_state,
        InstallationTargetState::Disabled
    ));
    fixture.revalidate().unwrap();
}

#[test]
fn j24k3c3_missing_candidate_fails_stale() {
    let fixture = RecoveryFixture::new();
    let path = fixture
        .base
        .join("candidates")
        .join(format!("{}.json", fixture.candidate.candidate_id));
    fs::remove_file(path).unwrap();
    let err = fixture.revalidate().unwrap_err();
    assert_eq!(err.code, "installation_intent_evidence_stale");
}

#[test]
fn j24k3c3_tampered_candidate_record_fails_stale() {
    let fixture = RecoveryFixture::new();
    let path = fixture
        .base
        .join("candidates")
        .join(format!("{}.json", fixture.candidate.candidate_id));
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    value["package_version"] = "9.9.9".into();
    fs::write(&path, serde_json_canonicalizer::to_vec(&value).unwrap()).unwrap();
    let err = fixture.revalidate().unwrap_err();
    assert_eq!(err.code, "installation_intent_evidence_stale");
}

#[test]
fn j24k3c3_quarantined_byte_mutation_fails_stale() {
    let fixture = RecoveryFixture::new();
    let plug_path = fixture
        .quarantine_root
        .join(&fixture.candidate.quarantine_relative_path)
        .join("plug.json");
    let original = fs::read(&plug_path).unwrap();
    let mut mutated = original.clone();
    mutated.push(b'\n');
    make_writable(&plug_path);
    fs::write(&plug_path, mutated).unwrap();
    let err = fixture.revalidate().unwrap_err();
    assert_eq!(err.code, "installation_intent_evidence_stale");
}

#[test]
fn j24k3c3_missing_exact_trust_fails_stale() {
    let fixture = RecoveryFixture::new();
    let path = fixture
        .base
        .join("trust")
        .join(format!("{}.json", fixture.candidate.candidate_id));
    fs::remove_file(path).unwrap();
    let err = fixture.revalidate().unwrap_err();
    assert_eq!(err.code, "installation_intent_evidence_stale");
}

#[test]
fn j24k3c3_changed_exact_trust_record_fails_stale() {
    let fixture = RecoveryFixture::new();
    let path = fixture
        .base
        .join("trust")
        .join(format!("{}.json", fixture.candidate.candidate_id));
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    value["approving_authority"] = "other-authority".into();
    fs::write(&path, serde_json_canonicalizer::to_vec(&value).unwrap()).unwrap();
    let err = fixture.revalidate().unwrap_err();
    assert_eq!(err.code, "installation_intent_evidence_stale");
}

#[test]
fn j24k3c3_differently_authorised_trust_fails_stale() {
    let fixture = RecoveryFixture::new();
    let path = fixture
        .base
        .join("trust")
        .join(format!("{}.json", fixture.candidate.candidate_id));
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    value["approving_authority"] = "another-authority".into();
    fs::write(&path, serde_json_canonicalizer::to_vec(&value).unwrap()).unwrap();
    let err = fixture.revalidate().unwrap_err();
    assert_eq!(err.code, "installation_intent_evidence_stale");
}

#[test]
fn j24k3c3_reconstructed_trust_must_equal_intent_trust_evidence() {
    let mut fixture = RecoveryFixture::new();
    let original_trust = fixture.intent.installed_record.trust_evidence.clone();
    let mut tampered_trust = original_trust.clone();
    if let TrustModeEvidence::ExactCandidate {
        ref mut approving_authority,
        ..
    } = tampered_trust.mode
    {
        *approving_authority = "tampered".into();
    }
    let mut covered = tampered_trust.clone();
    covered.evidence_digest.clear();
    tampered_trust.evidence_digest = sha256(&canonical(&covered).unwrap());
    fixture.intent.installed_record.trust_evidence = tampered_trust;
    recompute_intent(&mut fixture.intent);
    let err = fixture.revalidate().unwrap_err();
    assert_eq!(err.code, "installation_intent_evidence_stale");
}

#[test]
fn j24k3c3_missing_launch_profile_fails_stale() {
    let fixture = RecoveryFixture::new();
    let suffix = fixture
        .launch
        .profile_evidence_digest
        .strip_prefix("sha256:")
        .unwrap();
    let path = fixture
        .base
        .join("profiles")
        .join(format!("{}.json", suffix));
    fs::remove_file(path).unwrap();
    let err = fixture.revalidate().unwrap_err();
    assert_eq!(err.code, "installation_intent_evidence_stale");
}

#[test]
fn j24k3c3_mismatched_launch_profile_digest_fails_stale() {
    let fixture = RecoveryFixture::new();
    let path = fixture
        .base
        .join("conformance")
        .join(format!("{}.json", fixture.conformance_evidence.evidence_id));
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    value["launch_profile_evidence_digest"] =
        "sha256:0000000000000000000000000000000000000000000000000000000000000000".into();
    fs::write(&path, serde_json_canonicalizer::to_vec(&value).unwrap()).unwrap();
    let err = fixture.revalidate().unwrap_err();
    assert_eq!(err.code, "installation_intent_evidence_stale");
}

#[test]
fn j24k3c3_candidate_stale_launch_profile_fails_stale() {
    let fixture = RecoveryFixture::new();
    let suffix = fixture
        .launch
        .profile_evidence_digest
        .strip_prefix("sha256:")
        .unwrap();
    let path = fixture
        .base
        .join("profiles")
        .join(format!("{}.json", suffix));
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    value["candidate_id"] = Uuid::new_v4().to_string().into();
    value["profile_evidence_digest"] = String::new().into();
    fs::write(&path, serde_json_canonicalizer::to_vec(&value).unwrap()).unwrap();
    let err = fixture.revalidate().unwrap_err();
    assert_eq!(err.code, "installation_intent_evidence_stale");
}

#[test]
fn j24k3c3_missing_conformance_fails_stale() {
    let fixture = RecoveryFixture::new();
    let path = fixture
        .base
        .join("conformance")
        .join(format!("{}.json", fixture.conformance_evidence.evidence_id));
    fs::remove_file(path).unwrap();
    let err = fixture.revalidate().unwrap_err();
    assert_eq!(err.code, "installation_intent_evidence_stale");
}

#[test]
fn j24k3c3_non_passed_conformance_fails_stale() {
    let fixture = RecoveryFixture::new();
    let path = fixture
        .base
        .join("conformance")
        .join(format!("{}.json", fixture.conformance_evidence.evidence_id));
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    value["disposition"] = "failed".into();
    value["evidence_digest"] = String::new().into();
    fs::write(&path, serde_json_canonicalizer::to_vec(&value).unwrap()).unwrap();
    let err = fixture.revalidate().unwrap_err();
    assert_eq!(err.code, "installation_intent_evidence_stale");
}

#[test]
fn j24k3c3_old_suite_conformance_fails_stale() {
    let fixture = RecoveryFixture::new();
    let path = fixture
        .base
        .join("conformance")
        .join(format!("{}.json", fixture.conformance_evidence.evidence_id));
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    value["suite_digest"] =
        "sha256:0000000000000000000000000000000000000000000000000000000000000000".into();
    value["evidence_digest"] = String::new().into();
    fs::write(&path, serde_json_canonicalizer::to_vec(&value).unwrap()).unwrap();
    let err = fixture.revalidate().unwrap_err();
    assert_eq!(err.code, "installation_intent_evidence_stale");
}

#[test]
fn j24k3c3_missing_approval_fails_stale() {
    let fixture = RecoveryFixture::new();
    let path = fixture
        .base
        .join("approvals")
        .join(format!("{}.json", fixture.approval.approval_id));
    fs::remove_file(path).unwrap();
    let err = fixture.revalidate().unwrap_err();
    assert_eq!(err.code, "installation_intent_evidence_stale");
}

#[test]
fn j24k3c3_digest_mismatched_approval_fails_stale() {
    let mut fixture = RecoveryFixture::new();
    fixture.intent.installed_record.installation_approval_digest =
        "sha256:0000000000000000000000000000000000000000000000000000000000000000".into();
    recompute_intent(&mut fixture.intent);
    let err = fixture.revalidate().unwrap_err();
    assert_eq!(err.code, "installation_intent_evidence_stale");
}

#[test]
fn j24k3c3_approval_trust_drift_fails_stale() {
    let fixture = RecoveryFixture::new();
    let path = fixture
        .base
        .join("approvals")
        .join(format!("{}.json", fixture.approval.approval_id));
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let mut trust: serde_json::Value = value["trust_evidence"].clone();
    trust["evidence_digest"] =
        "sha256:0000000000000000000000000000000000000000000000000000000000000000".into();
    value["trust_evidence"] = trust;
    value["record_digest"] = String::new().into();
    fs::write(&path, serde_json_canonicalizer::to_vec(&value).unwrap()).unwrap();
    let err = fixture.revalidate().unwrap_err();
    assert_eq!(err.code, "installation_intent_evidence_stale");
}

#[test]
fn j24k3c3_approval_launch_drift_fails_stale() {
    let fixture = RecoveryFixture::new();
    let path = fixture
        .base
        .join("approvals")
        .join(format!("{}.json", fixture.approval.approval_id));
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    value["launch_profile_evidence_digest"] =
        "sha256:0000000000000000000000000000000000000000000000000000000000000000".into();
    value["record_digest"] = String::new().into();
    fs::write(&path, serde_json_canonicalizer::to_vec(&value).unwrap()).unwrap();
    let err = fixture.revalidate().unwrap_err();
    assert_eq!(err.code, "installation_intent_evidence_stale");
}

#[test]
fn j24k3c3_approval_conformance_drift_fails_stale() {
    let fixture = RecoveryFixture::new();
    let path = fixture
        .base
        .join("approvals")
        .join(format!("{}.json", fixture.approval.approval_id));
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    value["conformance_evidence_digest"] =
        "sha256:0000000000000000000000000000000000000000000000000000000000000000".into();
    value["record_digest"] = String::new().into();
    fs::write(&path, serde_json_canonicalizer::to_vec(&value).unwrap()).unwrap();
    let err = fixture.revalidate().unwrap_err();
    assert_eq!(err.code, "installation_intent_evidence_stale");
}

#[test]
fn j24k3c3_approval_candidate_drift_fails_stale() {
    let fixture = RecoveryFixture::new();
    let path = fixture
        .base
        .join("approvals")
        .join(format!("{}.json", fixture.approval.approval_id));
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    value["candidate_id"] = Uuid::new_v4().to_string().into();
    value["record_digest"] = String::new().into();
    fs::write(&path, serde_json_canonicalizer::to_vec(&value).unwrap()).unwrap();
    let err = fixture.revalidate().unwrap_err();
    assert_eq!(err.code, "installation_intent_evidence_stale");
}

#[test]
fn j24k3c3_approval_provider_drift_fails_stale() {
    let fixture = RecoveryFixture::new();
    let path = fixture
        .base
        .join("approvals")
        .join(format!("{}.json", fixture.approval.approval_id));
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    value["provider_version"] = "9.9.9".into();
    value["record_digest"] = String::new().into();
    fs::write(&path, serde_json_canonicalizer::to_vec(&value).unwrap()).unwrap();
    let err = fixture.revalidate().unwrap_err();
    assert_eq!(err.code, "installation_intent_evidence_stale");
}

#[test]
fn j24k3c3_approval_payload_drift_fails_stale() {
    let fixture = RecoveryFixture::new();
    let path = fixture
        .base
        .join("approvals")
        .join(format!("{}.json", fixture.approval.approval_id));
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let mut payloads = value["payloads"].as_array().unwrap().clone();
    if !payloads.is_empty() {
        payloads[0]["size_bytes"] = 99999.into();
    }
    value["payloads"] = payloads.into();
    value["record_digest"] = String::new().into();
    fs::write(&path, serde_json_canonicalizer::to_vec(&value).unwrap()).unwrap();
    let err = fixture.revalidate().unwrap_err();
    assert_eq!(err.code, "installation_intent_evidence_stale");
}

#[test]
fn j24k3c3_approval_reviewed_capability_drift_fails_stale() {
    let fixture = RecoveryFixture::new();
    let manifest_path = fixture
        .quarantine_root
        .join(&fixture.candidate.quarantine_relative_path)
        .join("manifests/fixture-ping.json");
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
    manifest["effects"] = serde_json::json!(["unexpected"]);
    make_writable(&manifest_path);
    fs::write(
        &manifest_path,
        serde_json_canonicalizer::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    let err = fixture.revalidate().unwrap_err();
    assert_eq!(err.code, "installation_intent_evidence_stale");
}

#[test]
fn j24k3c3_installed_record_package_drift_fails_stale() {
    let mut fixture = RecoveryFixture::new();
    fixture.intent.installed_record.package_version = "9.9.9".into();
    recompute_intent(&mut fixture.intent);
    let err = fixture.revalidate().unwrap_err();
    assert_eq!(err.code, "installation_intent_evidence_stale");
}

#[test]
fn j24k3c3_installed_record_physical_evidence_drift_fails_stale() {
    let mut fixture = RecoveryFixture::new();
    fixture.intent.installed_record.payloads[0].size_bytes += 1;
    recompute_intent(&mut fixture.intent);
    let err = fixture.revalidate().unwrap_err();
    assert_eq!(err.code, "installation_intent_evidence_stale");
}

#[test]
fn j24k3c3_installed_record_capability_drift_fails_stale() {
    let mut fixture = RecoveryFixture::new();
    if !fixture
        .intent
        .installed_record
        .capability_manifests
        .is_empty()
    {
        fixture.intent.installed_record.capability_manifests[0].version += 1;
    }
    recompute_intent(&mut fixture.intent);
    let err = fixture.revalidate().unwrap_err();
    assert_eq!(err.code, "installation_intent_evidence_stale");
}

#[test]
fn j24k3c3_installed_record_trust_drift_fails_stale() {
    let mut fixture = RecoveryFixture::new();
    let mut tampered = fixture.intent.installed_record.trust_evidence.clone();
    if let TrustModeEvidence::ExactCandidate {
        ref mut approving_authority,
        ..
    } = tampered.mode
    {
        *approving_authority = "tampered".into();
    }
    let mut covered = tampered.clone();
    covered.evidence_digest.clear();
    tampered.evidence_digest = sha256(&canonical(&covered).unwrap());
    fixture.intent.installed_record.trust_evidence = tampered;
    recompute_intent(&mut fixture.intent);
    let err = fixture.revalidate().unwrap_err();
    assert_eq!(err.code, "installation_intent_evidence_stale");
}

#[test]
fn j24k3c3_installed_record_approval_drift_fails_stale() {
    let mut fixture = RecoveryFixture::new();
    fixture.intent.installed_record.installation_approval_digest =
        "sha256:0000000000000000000000000000000000000000000000000000000000000000".into();
    recompute_intent(&mut fixture.intent);
    let err = fixture.revalidate().unwrap_err();
    assert_eq!(err.code, "installation_intent_evidence_stale");
}

#[test]
fn j24k3c3_installed_record_conformance_drift_fails_stale() {
    let mut fixture = RecoveryFixture::new();
    fixture.intent.installed_record.conformance_evidence_digest =
        "sha256:0000000000000000000000000000000000000000000000000000000000000000".into();
    recompute_intent(&mut fixture.intent);
    let err = fixture.revalidate().unwrap_err();
    assert_eq!(err.code, "installation_intent_evidence_stale");
}

#[test]
fn j24k3c3_installed_record_provider_drift_fails_stale() {
    let mut fixture = RecoveryFixture::new();
    fixture.intent.installed_record.provider_version = "9.9.9".into();
    recompute_intent(&mut fixture.intent);
    let err = fixture.revalidate().unwrap_err();
    assert_eq!(err.code, "installation_intent_evidence_stale");
}

#[test]
fn j24k3c3_installed_record_launch_drift_fails_stale() {
    let mut fixture = RecoveryFixture::new();
    fixture
        .intent
        .installed_record
        .launch_arguments
        .push("extra".into());
    recompute_intent(&mut fixture.intent);
    let err = fixture.revalidate().unwrap_err();
    assert_eq!(err.code, "installation_intent_evidence_stale");
}

#[test]
fn j24k3c3_installed_record_platform_drift_fails_stale() {
    let fixture = RecoveryFixture::new();
    let mut synthetic_candidate = fixture.candidate.clone();
    synthetic_candidate.selected_platform.os = "linux".into();
    let err = fixture
        .intent
        .installed_record
        .require_for_recovery(
            &fixture.intent,
            &synthetic_candidate,
            &fixture.trust,
            &fixture.launch,
            &fixture.conformance_evidence,
            &fixture.approval,
        )
        .unwrap_err();
    assert_eq!(err.code, "installed_record_invalid");
}

#[test]
fn j24k3c3_installed_record_architecture_drift_fails_stale() {
    let fixture = RecoveryFixture::new();
    let mut synthetic_candidate = fixture.candidate.clone();
    synthetic_candidate.selected_platform.architecture = "aarch64".into();
    let err = fixture
        .intent
        .installed_record
        .require_for_recovery(
            &fixture.intent,
            &synthetic_candidate,
            &fixture.trust,
            &fixture.launch,
            &fixture.conformance_evidence,
            &fixture.approval,
        )
        .unwrap_err();
    assert_eq!(err.code, "installed_record_invalid");
}

#[test]
fn j24k3c3_scope_schema_match_passes_recovery() {
    let fixture = RecoveryFixture::new();
    fixture
        .intent
        .installed_record
        .require_for_recovery(
            &fixture.intent,
            &fixture.candidate,
            &fixture.trust,
            &fixture.launch,
            &fixture.conformance_evidence,
            &fixture.approval,
        )
        .expect("matching scope schema/digest must pass recovery");
}

#[test]
fn j24k3c3_changed_scope_schema_with_valid_digest_fails_stale() {
    let fixture = RecoveryFixture::new();
    let mut synthetic = fixture.candidate.clone();
    let changed = serde_json::json!({"type":"object","properties":{"other":{"type":"string"}},"required":["other"],"additionalProperties":false});
    let canonical = crate::m3_store::canonical(&changed).unwrap();
    let digest = crate::m3_store::sha256(&canonical);
    synthetic.operational_scope_schema = Some(changed);
    synthetic.operational_scope_schema_digest = Some(digest);
    let err = fixture
        .intent
        .installed_record
        .require_for_recovery(
            &fixture.intent,
            &synthetic,
            &fixture.trust,
            &fixture.launch,
            &fixture.conformance_evidence,
            &fixture.approval,
        )
        .unwrap_err();
    assert_eq!(err.code, "installed_record_invalid");
}

#[test]
fn j24k3c3_changed_scope_digest_fails_stale() {
    let fixture = RecoveryFixture::new();
    let mut synthetic = fixture.candidate.clone();
    synthetic.operational_scope_schema_digest =
        Some("sha256:0000000000000000000000000000000000000000000000000000000000000000".into());
    let err = fixture
        .intent
        .installed_record
        .require_for_recovery(
            &fixture.intent,
            &synthetic,
            &fixture.trust,
            &fixture.launch,
            &fixture.conformance_evidence,
            &fixture.approval,
        )
        .unwrap_err();
    assert_eq!(err.code, "installed_record_invalid");
}

#[test]
fn j24k3c3_installed_record_disabled_binding_drift_fails_stale() {
    let mut fixture = RecoveryFixture::new();
    if !fixture.intent.installed_record.disabled_bindings.is_empty() {
        fixture.intent.installed_record.disabled_bindings[0].capability_version += 1;
    }
    recompute_intent(&mut fixture.intent);
    let err = fixture.revalidate().unwrap_err();
    assert_eq!(err.code, "installation_intent_evidence_stale");
}

#[test]
fn j24k3c3_unrelated_evidence_does_not_satisfy_pinned_records() {
    let fixture = RecoveryFixture::new();
    let other = RecoveryFixture::new();

    // Replace the intent to pin the other chain while keeping this candidate store.
    // The candidate is different, so revalidation must fail stale rather than
    // accidentally matching unrelated evidence.
    let mut fixture = fixture;
    fixture.intent = other.intent.clone();
    fixture.request = other.request.clone();

    let err = fixture.revalidate().unwrap_err();
    assert_eq!(err.code, "installation_intent_evidence_stale");
}

#[cfg(windows)]
#[test]
fn j24k3c3_unsafe_quarantine_path_remains_unsafe() {
    let fixture = RecoveryFixture::new();
    let target = fixture.base.join("quarantine-target");
    fs::rename(&fixture.quarantine_root, &target).unwrap();
    let status = std::process::Command::new("cmd")
        .args([
            "/C",
            "mklink",
            "/J",
            fixture.quarantine_root.to_str().unwrap(),
            target.to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(status.success(), "could not create junction fixture");
    let err = fixture.revalidate().unwrap_err();
    assert_eq!(err.code, "unsafe_store_path");
}

#[test]
fn j24k3c3_genuine_io_failure_maps_to_recovery_io() {
    let fixture = RecoveryFixture::new();
    fs::remove_dir_all(&fixture.quarantine_root).unwrap();
    let err = fixture.revalidate().unwrap_err();
    assert_eq!(err.code, "installation_recovery_io");
}

#[test]
fn j24k3c3_success_leaves_stores_quarantine_and_permissions_unchanged() {
    let fixture = RecoveryFixture::new();
    let candidate_root = fixture.base.join("candidates");
    let quarantine_root = fixture.quarantine_root.clone();
    let exact_trust_root = fixture.base.join("trust");
    let launch_profiles_root = fixture.base.join("profiles");
    let conformance_root = fixture.base.join("conformance");
    let approvals_root = fixture.base.join("approvals");

    let candidates_before = tree_snapshot(&candidate_root);
    let quarantine_before = tree_snapshot(&quarantine_root);
    let exact_trust_before = tree_snapshot(&exact_trust_root);
    let launch_profiles_before = tree_snapshot(&launch_profiles_root);
    let conformance_before = tree_snapshot(&conformance_root);
    let approvals_before = tree_snapshot(&approvals_root);

    fixture.revalidate().unwrap();

    let candidates_after = tree_snapshot(&candidate_root);
    let quarantine_after = tree_snapshot(&quarantine_root);
    let exact_trust_after = tree_snapshot(&exact_trust_root);
    let launch_profiles_after = tree_snapshot(&launch_profiles_root);
    let conformance_after = tree_snapshot(&conformance_root);
    let approvals_after = tree_snapshot(&approvals_root);

    assert_eq!(candidates_before, candidates_after);
    assert_eq!(quarantine_before, quarantine_after);
    assert_eq!(exact_trust_before, exact_trust_after);
    assert_eq!(launch_profiles_before, launch_profiles_after);
    assert_eq!(conformance_before, conformance_after);
    assert_eq!(approvals_before, approvals_after);
}
