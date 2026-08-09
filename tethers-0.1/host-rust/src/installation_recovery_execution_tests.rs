use super::installation_recovery_execution::{
    execute_validated_installation_recovery, InstallationRecoveryExecutionOutcome,
};
use crate::candidate::{extract_to_quarantine, CandidateRegistry};
use crate::conformance::{
    current_suite_digest, CaseDisposition, ConformanceCaseEvidence, ConformanceDisposition,
    ConformanceEvidence, ConformanceEvidenceStore,
};
use crate::current_trust::ExactCandidateTrustAuthority;
use crate::installation_publication_intent::{
    InstallationPublicationIntent, InstallationPublicationIntentStore,
};
use crate::installation_recovery_evidence::InstallationRecoveryEvidenceContext;
use crate::installation_recovery_plan::{
    plan_installation_recovery, InstallationRecoveryPlanningContext,
};
use crate::installation_request::{
    InstallationConformanceRequest, InstallationRequest, InstallationTargetRequest,
    InstallationTargetState, InstallationTrustRequest, InstallationTrustScope,
    INSTALLATION_REQUEST_SCHEMA,
};
use crate::installation_trust::ExactCandidateTrustStore;
use crate::installed::{
    DisabledBindingRecord, InstallationApprovalStore, InstalledPlugRecord, InstalledPlugRegistry,
};
use crate::launch_profile::{
    LaunchProfileEvidence, LaunchProfileEvidenceStore, PreparedSupervisedLaunch,
};
use crate::m3_store::{canonical, sha256};
use crate::package::{self, PayloadEvidence};
use crate::trust::PackageTrustEvidence;
use std::collections::BTreeMap;
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

fn valid_record() -> InstalledPlugRecord {
    let digest = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let mut trust = PackageTrustEvidence {
        evidence_format_version: 1,
        semantic_package_digest: digest.into(),
        mode: crate::trust::TrustModeEvidence::UnsignedDeveloper {
            approval_id: "approval".into(),
            approval_record_digest: digest.into(),
            visibly_unsigned: true,
        },
        evidence_digest: String::new(),
    };
    let mut trust_covered = trust.clone();
    trust_covered.evidence_digest.clear();
    trust.evidence_digest = sha256(&canonical(&trust_covered).unwrap());
    let id = Uuid::new_v4().to_string();
    let mut record = InstalledPlugRecord {
        schema_version: 1,
        installed_id: id.clone(),
        state: "present_disabled".into(),
        package_id: "tethers.file-tools".into(),
        package_version: "1.1.0".into(),
        semantic_package_digest: digest.into(),
        source_candidate_id: Uuid::new_v4().to_string(),
        installation_relative_path: format!("plug-{id}"),
        raw_archive_digest: digest.into(),
        plug_json: PayloadEvidence {
            path: "plug.json".into(),
            sha256: digest.into(),
            size_bytes: 1,
            role: "package_descriptor".into(),
        },
        payloads: vec![PayloadEvidence {
            path: "lib/engine.dll".into(),
            sha256: digest.into(),
            size_bytes: 1,
            role: "payload".into(),
        }],
        signature_files: Vec::new(),
        capability_manifests: Vec::new(),
        trust_evidence: trust,
        installation_approval_id: Uuid::new_v4().to_string(),
        installation_approval_digest: digest.into(),
        conformance_evidence_id: Uuid::new_v4().to_string(),
        conformance_evidence_digest: digest.into(),
        provider_id: "tethers-file-tools".into(),
        provider_version: "1.1.0".into(),
        launch_path: "tethers-host-engine.exe".into(),
        launch_arguments: vec!["--stdio".into()],
        provider_working_directory: "engine".into(),
        launch_profile_label: "supervised".into(),
        socket_major: 1,
        mcp_protocol_version: "2025-11-25".into(),
        platform: "windows".into(),
        architecture: "x86_64".into(),
        disabled_bindings: vec![DisabledBindingRecord {
            state: "disabled".into(),
            capability_name: "tethers.file-tools".into(),
            capability_version: 1,
            manifest_digest: digest.into(),
            provider_operation_name: "file_tools".into(),
        }],
        operational_scope_schema: None,
        operational_scope_schema_digest: None,
        created_unix_ms: 1,
        record_digest: String::new(),
    };
    let mut covered = record.clone();
    covered.record_digest.clear();
    record.record_digest = sha256(&canonical(&covered).unwrap());
    record
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
        host_build_identity: "j24k3d2-test".to_owned(),
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

fn build_installed_record(
    candidate: &crate::candidate::CandidateRecord,
    trust: &PackageTrustEvidence,
    launch: &LaunchProfileEvidence,
    conformance: &ConformanceEvidence,
    approval: &crate::installed::InstallationApprovalRecord,
) -> InstalledPlugRecord {
    let installed_id = Uuid::new_v4().to_string();
    let mut record = InstalledPlugRecord {
        schema_version: 1,
        installed_id: installed_id.clone(),
        state: "present_disabled".into(),
        package_id: candidate.package_id.clone(),
        package_version: candidate.package_version.clone(),
        semantic_package_digest: candidate.semantic_package_digest.clone(),
        source_candidate_id: candidate.candidate_id.clone(),
        installation_relative_path: format!("plug-{installed_id}"),
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
        disabled_bindings: candidate
            .capabilities
            .iter()
            .map(|capability| DisabledBindingRecord {
                state: "disabled".into(),
                capability_name: capability.name.clone(),
                capability_version: capability.version,
                manifest_digest: capability.manifest_digest.clone(),
                provider_operation_name: capability.operation.clone(),
            })
            .collect(),
        operational_scope_schema: None,
        operational_scope_schema_digest: None,
        created_unix_ms: 1,
        record_digest: String::new(),
    };
    let mut covered = record.clone();
    covered.record_digest.clear();
    record.record_digest = sha256(&canonical(&covered).unwrap());
    record.validate().unwrap();
    record
}

struct LightFixture {
    base: PathBuf,
    quarantine_root: PathBuf,
    install_root: PathBuf,
    record_root: PathBuf,
    intent_store: InstallationPublicationIntentStore,
    installed: InstalledPlugRegistry,
    candidates: CandidateRegistry,
    exact_trust: ExactCandidateTrustStore,
    launch_profiles: LaunchProfileEvidenceStore,
    conformance: ConformanceEvidenceStore,
    approvals: InstallationApprovalStore,
    request: InstallationRequest,
}

impl LightFixture {
    fn new() -> Self {
        let base = std::env::temp_dir().join(format!("tethers-j24k3d2-light-{}", Uuid::new_v4()));
        fs::create_dir_all(&base).unwrap();
        let install_root = base.join("install");
        let record_root = base.join("records");
        fs::create_dir_all(&install_root).unwrap();
        fs::create_dir_all(&record_root).unwrap();
        let quarantine = base.join("quarantine");
        let candidates = CandidateRegistry::open(&base.join("candidates"), &quarantine).unwrap();
        let exact_trust = ExactCandidateTrustStore::open(&base.join("trust")).unwrap();
        let launch_profiles = LaunchProfileEvidenceStore::open(&base.join("profiles")).unwrap();
        let conformance = ConformanceEvidenceStore::open(&base.join("conformance")).unwrap();
        let approvals = InstallationApprovalStore::open(&base.join("approvals")).unwrap();
        let request = complete_request(&Uuid::new_v4().to_string());
        Self {
            intent_store: InstallationPublicationIntentStore::open(&base.join("intents")).unwrap(),
            installed: InstalledPlugRegistry::open_existing(&install_root, &record_root).unwrap(),
            base,
            quarantine_root: quarantine,
            install_root,
            record_root,
            candidates,
            exact_trust,
            launch_profiles,
            conformance,
            approvals,
            request,
        }
    }

    fn context(&self) -> InstallationRecoveryPlanningContext<'_> {
        InstallationRecoveryPlanningContext {
            intents: &self.intent_store,
            installed: &self.installed,
            evidence: InstallationRecoveryEvidenceContext {
                quarantine_root: &self.quarantine_root,
                candidates: &self.candidates,
                exact_trust: &self.exact_trust,
                launch_profiles: &self.launch_profiles,
                conformance: &self.conformance,
                approvals: &self.approvals,
            },
        }
    }

    fn plan(
        &self,
    ) -> crate::m3_store::Result<crate::installation_recovery_plan::ValidatedInstallationRecoveryPlan>
    {
        plan_installation_recovery(&self.request, &self.context())
    }
}

struct FullFixture {
    base: PathBuf,
    quarantined_root: PathBuf,
    quarantine_root: PathBuf,
    candidates: CandidateRegistry,
    exact_trust: ExactCandidateTrustStore,
    launch_profiles: LaunchProfileEvidenceStore,
    conformance: ConformanceEvidenceStore,
    approvals: InstallationApprovalStore,
    request: InstallationRequest,
    intent: InstallationPublicationIntent,
    installed_record: InstalledPlugRecord,
    intent_store: InstallationPublicationIntentStore,
    installed: InstalledPlugRegistry,
    install_root: PathBuf,
    record_root: PathBuf,
}

impl FullFixture {
    fn new() -> Self {
        let base = std::env::temp_dir().join(format!("tethers-j24k3d2-full-{}", Uuid::new_v4()));
        fs::create_dir_all(&base).unwrap();
        let archive = base.join("test.tetherplug");
        fs::write(
            &archive,
            crate::pdf_tools::build_reference_package(b"j24k3d2-test").unwrap(),
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
            .create(&candidate, &request, "j24k3d2-authority")
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
                "j24k3d2-authority",
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
            InstallationPublicationIntent::from_precomputed_record(installed_record.clone())
                .unwrap();
        let install_root = base.join("install");
        let record_root = base.join("records");
        fs::create_dir_all(&install_root).unwrap();
        fs::create_dir_all(&record_root).unwrap();
        Self {
            intent_store: InstallationPublicationIntentStore::open(&base.join("intents")).unwrap(),
            installed: InstalledPlugRegistry::open_existing(&install_root, &record_root).unwrap(),
            base,
            quarantined_root: quarantined.directory,
            quarantine_root,
            candidates,
            exact_trust,
            launch_profiles,
            conformance,
            approvals,
            request,
            intent,
            installed_record,
            install_root,
            record_root,
        }
    }

    fn context(&self) -> InstallationRecoveryPlanningContext<'_> {
        InstallationRecoveryPlanningContext {
            intents: &self.intent_store,
            installed: &self.installed,
            evidence: InstallationRecoveryEvidenceContext {
                quarantine_root: &self.quarantine_root,
                candidates: &self.candidates,
                exact_trust: &self.exact_trust,
                launch_profiles: &self.launch_profiles,
                conformance: &self.conformance,
                approvals: &self.approvals,
            },
        }
    }

    fn plan(
        &self,
    ) -> crate::m3_store::Result<crate::installation_recovery_plan::ValidatedInstallationRecoveryPlan>
    {
        plan_installation_recovery(&self.request, &self.context())
    }

    fn build_destination(&self) {
        let destination = self
            .install_root
            .join(&self.intent.destination_relative_path);
        fs::create_dir(&destination).unwrap();
        for evidence in std::iter::once(&self.intent.installed_record.plug_json)
            .chain(self.intent.installed_record.payloads.iter())
            .chain(self.intent.installed_record.signature_files.iter())
        {
            let source = self.quarantined_root.join(&evidence.path);
            let bytes = fs::read(source).unwrap();
            let target = destination.join(&evidence.path);
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(&target, bytes).unwrap();
            let mut permissions = fs::metadata(&target).unwrap().permissions();
            permissions.set_readonly(true);
            fs::set_permissions(&target, permissions).unwrap();
        }
    }
}

fn write_record(root: &Path, record: &InstalledPlugRecord) {
    fs::write(
        root.join(format!("{}.json", record.installed_id)),
        canonical(record).unwrap(),
    )
    .unwrap();
}

fn replace_intent(fix: &LightFixture) -> InstallationPublicationIntent {
    let current = fix.intent_store.load().unwrap().unwrap();
    fix.intent_store.remove_if_matches(&current).unwrap();
    let mut record = current.installed_record;
    record.package_version.push_str("-changed");
    let mut covered = record.clone();
    covered.record_digest.clear();
    record.record_digest = sha256(&canonical(&covered).unwrap());
    let replacement = InstallationPublicationIntent::from_precomputed_record(record).unwrap();
    fix.intent_store.create(&replacement).unwrap();
    replacement
}

fn create_directory_link(link: &Path, target: &Path) {
    #[cfg(windows)]
    {
        let status = std::process::Command::new("cmd")
            .args([
                "/C",
                "mklink",
                "/J",
                link.to_str().unwrap(),
                target.to_str().unwrap(),
            ])
            .status()
            .unwrap();
        assert!(status.success());
    }
    #[cfg(unix)]
    std::os::unix::fs::symlink(target, link).unwrap();
}

fn tree_snapshot(root: &Path) -> BTreeMap<String, Vec<u8>> {
    fn visit(root: &Path, current: &Path, out: &mut BTreeMap<String, Vec<u8>>) {
        for entry in fs::read_dir(current).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                visit(root, &path, out);
            } else {
                out.insert(
                    path.strip_prefix(root)
                        .unwrap()
                        .to_string_lossy()
                        .into_owned(),
                    fs::read(path).unwrap(),
                );
            }
        }
    }
    let mut result = BTreeMap::new();
    visit(root, root, &mut result);
    result
}

#[test]
fn j24k3d2_idle_plan_performs_no_mutation() {
    let fix = LightFixture::new();
    let before = tree_snapshot(&fix.base);
    let plan = fix.plan().unwrap();
    let outcome =
        execute_validated_installation_recovery(&fix.request, &fix.context(), plan).unwrap();
    assert_eq!(outcome, InstallationRecoveryExecutionOutcome::Idle);
    assert_eq!(before, tree_snapshot(&fix.base));
}

#[test]
fn j24k3d2_intent_only_removes_exact_intent_and_returns_idle() {
    let fix = LightFixture::new();
    let intent = InstallationPublicationIntent::from_precomputed_record(valid_record()).unwrap();
    fix.intent_store.create(&intent).unwrap();
    let plan = fix.plan().unwrap();
    let outcome =
        execute_validated_installation_recovery(&fix.request, &fix.context(), plan).unwrap();
    assert_eq!(
        outcome,
        InstallationRecoveryExecutionOutcome::Recovered {
            disposition:
                crate::installation_recovery::InstallationRecoveryDisposition::RemoveIntentOnly
        }
    );
    assert!(fix.intent_store.load().unwrap().is_none());
    assert!(fix.installed.load_all().unwrap().is_empty());
}

#[test]
fn j24k3d2_staging_recovery_removes_exact_staging_then_intent() {
    let fix = LightFixture::new();
    let intent = InstallationPublicationIntent::from_precomputed_record(valid_record()).unwrap();
    fix.intent_store.create(&intent).unwrap();
    let staging = fix
        .install_root
        .join(format!(".staging-{}", intent.transaction_id));
    let sibling = fix
        .install_root
        .join(".staging-00000000-0000-0000-0000-000000000000");
    fs::create_dir(&staging).unwrap();
    fs::create_dir(&sibling).unwrap();
    let plan = fix.plan().unwrap();
    let outcome =
        execute_validated_installation_recovery(&fix.request, &fix.context(), plan).unwrap();
    assert!(matches!(
        outcome,
        InstallationRecoveryExecutionOutcome::Recovered {
            disposition: crate::installation_recovery::InstallationRecoveryDisposition::RemoveStagingThenIntent
        }
    ));
    assert!(!staging.exists());
    assert!(sibling.exists());
    assert!(fix.intent_store.load().unwrap().is_none());
}

#[test]
fn j24k3d2_destination_only_publishes_exact_record_then_returns_idle() {
    let fix = FullFixture::new();
    fix.intent_store.create(&fix.intent).unwrap();
    fix.build_destination();
    let plan = fix.plan().unwrap();
    let expected = fix.installed_record.clone();
    let outcome =
        execute_validated_installation_recovery(&fix.request, &fix.context(), plan).unwrap();
    assert!(matches!(
        outcome,
        InstallationRecoveryExecutionOutcome::Recovered {
            disposition: crate::installation_recovery::InstallationRecoveryDisposition::RevalidateDestinationThenPublishRecord
        }
    ));
    let records = fix.installed.load_all().unwrap();
    assert_eq!(records, vec![expected]);
    assert!(fix.intent_store.load().unwrap().is_none());
}

#[test]
fn j24k3d2_completed_publication_removes_only_intent() {
    let fix = FullFixture::new();
    fix.intent_store.create(&fix.intent).unwrap();
    fix.build_destination();
    write_record(&fix.record_root, &fix.installed_record);
    let destination = tree_snapshot(&fix.install_root);
    let record = tree_snapshot(&fix.record_root);
    let plan = fix.plan().unwrap();
    let outcome =
        execute_validated_installation_recovery(&fix.request, &fix.context(), plan).unwrap();
    assert!(matches!(
        outcome,
        InstallationRecoveryExecutionOutcome::Recovered {
            disposition: crate::installation_recovery::InstallationRecoveryDisposition::VerifyCompletedPublicationThenRemoveIntent
        }
    ));
    assert_eq!(destination, tree_snapshot(&fix.install_root));
    assert_eq!(record, tree_snapshot(&fix.record_root));
    assert!(fix.intent_store.load().unwrap().is_none());
}

#[test]
fn j24k3d2_changed_authoritative_intent_conflicts_without_mutation() {
    let fix = LightFixture::new();
    let intent = InstallationPublicationIntent::from_precomputed_record(valid_record()).unwrap();
    fix.intent_store.create(&intent).unwrap();
    let plan = fix.plan().unwrap();
    let replacement = replace_intent(&fix);
    let error =
        execute_validated_installation_recovery(&fix.request, &fix.context(), plan).unwrap_err();
    assert_eq!(error.code, "installation_recovery_conflict");
    assert_eq!(fix.intent_store.load().unwrap(), Some(replacement));
}

#[test]
fn j24k3d2_changed_disposition_conflicts_without_intent_removal() {
    let fix = LightFixture::new();
    let intent = InstallationPublicationIntent::from_precomputed_record(valid_record()).unwrap();
    fix.intent_store.create(&intent).unwrap();
    let staging = fix
        .install_root
        .join(format!(".staging-{}", intent.transaction_id));
    fs::create_dir(&staging).unwrap();
    let plan = fix.plan().unwrap();
    fs::remove_dir(&staging).unwrap();
    let error =
        execute_validated_installation_recovery(&fix.request, &fix.context(), plan).unwrap_err();
    assert_eq!(error.code, "installation_recovery_conflict");
    assert_eq!(fix.intent_store.load().unwrap(), Some(intent));
}

#[test]
fn j24k3d2_stale_destination_prevents_publication_and_retains_intent() {
    let fix = FullFixture::new();
    fix.intent_store.create(&fix.intent).unwrap();
    fix.build_destination();
    let plan = fix.plan().unwrap();
    let file = fix
        .install_root
        .join(&fix.intent.destination_relative_path)
        .join(&fix.intent.installed_record.plug_json.path);
    let mut permissions = fs::metadata(&file).unwrap().permissions();
    permissions.set_readonly(false);
    fs::set_permissions(&file, permissions).unwrap();
    fs::write(&file, b"drift").unwrap();
    let error =
        execute_validated_installation_recovery(&fix.request, &fix.context(), plan).unwrap_err();
    assert_eq!(error.code, "installation_recovery_conflict");
    assert!(fix.intent_store.load().unwrap().is_some());
    assert!(fix.installed.load_all().unwrap().is_empty());
}

#[cfg(unix)]
#[test]
fn j24k3d2_staging_cleanup_failure_retains_intent_and_staging() {
    use std::os::unix::fs::PermissionsExt;
    let fix = LightFixture::new();
    let intent = InstallationPublicationIntent::from_precomputed_record(valid_record()).unwrap();
    fix.intent_store.create(&intent).unwrap();
    let staging = fix
        .install_root
        .join(format!(".staging-{}", intent.transaction_id));
    fs::create_dir(&staging).unwrap();
    let plan = fix.plan().unwrap();
    fs::set_permissions(&fix.install_root, fs::Permissions::from_mode(0o555)).unwrap();
    let error =
        execute_validated_installation_recovery(&fix.request, &fix.context(), plan).unwrap_err();
    fs::set_permissions(&fix.install_root, fs::Permissions::from_mode(0o755)).unwrap();
    assert_eq!(error.code, "installation_recovery_io");
    assert!(staging.exists());
    assert!(fix.intent_store.load().unwrap().is_some());
}

#[cfg(windows)]
#[test]
fn j24k3d2_staging_cleanup_failure_retains_intent_and_staging() {
    use std::fs::OpenOptions;
    use std::os::windows::fs::OpenOptionsExt;
    let fix = LightFixture::new();
    let intent = InstallationPublicationIntent::from_precomputed_record(valid_record()).unwrap();
    fix.intent_store.create(&intent).unwrap();
    let staging = fix
        .install_root
        .join(format!(".staging-{}", intent.transaction_id));
    fs::create_dir(&staging).unwrap();
    let held = staging.join("held.bin");
    fs::write(&held, b"held").unwrap();
    let handle = OpenOptions::new()
        .read(true)
        .write(true)
        .share_mode(0)
        .open(&held)
        .unwrap();
    let plan = fix.plan().unwrap();
    let error =
        execute_validated_installation_recovery(&fix.request, &fix.context(), plan).unwrap_err();
    drop(handle);
    assert_eq!(error.code, "installation_recovery_io");
    assert!(staging.exists());
    assert!(fix.intent_store.load().unwrap().is_some());
}

#[test]
fn j24k3d2_staging_reparse_state_is_refused_without_target_deletion() {
    let fix = LightFixture::new();
    let intent = InstallationPublicationIntent::from_precomputed_record(valid_record()).unwrap();
    fix.intent_store.create(&intent).unwrap();
    let staging = fix
        .install_root
        .join(format!(".staging-{}", intent.transaction_id));
    let target = fix.install_root.join("staging-target");
    fs::create_dir(&staging).unwrap();
    fs::create_dir(&target).unwrap();
    let plan = fix.plan().unwrap();
    fs::remove_dir(&staging).unwrap();
    create_directory_link(&staging, &target);
    let error =
        execute_validated_installation_recovery(&fix.request, &fix.context(), plan).unwrap_err();
    assert_eq!(error.code, "unsafe_store_path");
    assert!(target.exists());
    assert!(fix.intent_store.load().unwrap().is_some());
}

#[test]
fn j24k3d2_missing_install_root_preserves_recovery_io() {
    let fix = LightFixture::new();
    let plan = fix.plan().unwrap();
    fs::remove_dir_all(&fix.install_root).unwrap();
    let error =
        execute_validated_installation_recovery(&fix.request, &fix.context(), plan).unwrap_err();
    assert_eq!(error.code, "installation_recovery_io");
}

#[test]
fn j24k3d2_unsafe_record_root_preserves_unsafe_store_path() {
    let fix = LightFixture::new();
    let plan = fix.plan().unwrap();
    fs::remove_dir(&fix.record_root).unwrap();
    let target = fix.base.join("record-target");
    fs::create_dir(&target).unwrap();
    create_directory_link(&fix.record_root, &target);
    let error =
        execute_validated_installation_recovery(&fix.request, &fix.context(), plan).unwrap_err();
    assert_eq!(error.code, "unsafe_store_path");
    assert!(target.exists());
}

#[test]
fn j24k3d2_record_conflict_retains_intent_and_destination() {
    let fix = FullFixture::new();
    fix.intent_store.create(&fix.intent).unwrap();
    fix.build_destination();
    let plan = fix.plan().unwrap();
    write_record(&fix.record_root, &fix.installed_record);
    let error =
        execute_validated_installation_recovery(&fix.request, &fix.context(), plan).unwrap_err();
    assert_eq!(error.code, "installation_recovery_conflict");
    assert!(fix.intent_store.load().unwrap().is_some());
    assert_eq!(
        fix.installed.load_all().unwrap(),
        vec![fix.installed_record.clone()]
    );
}

#[test]
fn j24k3d2_exact_record_publication_preserves_all_record_fields() {
    let fix = FullFixture::new();
    fix.intent_store.create(&fix.intent).unwrap();
    fix.build_destination();
    let expected = fix.installed_record.clone();
    let plan = fix.plan().unwrap();
    execute_validated_installation_recovery(&fix.request, &fix.context(), plan).unwrap();
    let actual = fix.installed.load_all().unwrap().pop().unwrap();
    assert_eq!(actual, expected);
    assert_eq!(actual.installed_id, expected.installed_id);
    assert_eq!(actual.created_unix_ms, expected.created_unix_ms);
    assert_eq!(actual.record_digest, expected.record_digest);
}

#[test]
fn j24k3d2_destination_publication_requires_completed_replan() {
    let fix = FullFixture::new();
    fix.intent_store.create(&fix.intent).unwrap();
    fix.build_destination();
    let plan = fix.plan().unwrap();
    let outcome =
        execute_validated_installation_recovery(&fix.request, &fix.context(), plan).unwrap();
    assert!(matches!(
        outcome,
        InstallationRecoveryExecutionOutcome::Recovered {
            disposition: crate::installation_recovery::InstallationRecoveryDisposition::RevalidateDestinationThenPublishRecord
        }
    ));
    assert!(fix.intent_store.load().unwrap().is_none());
}

#[test]
fn j24k3d2_staging_cleanup_requires_intent_only_replan() {
    let fix = LightFixture::new();
    let intent = InstallationPublicationIntent::from_precomputed_record(valid_record()).unwrap();
    fix.intent_store.create(&intent).unwrap();
    fs::create_dir(
        fix.install_root
            .join(format!(".staging-{}", intent.transaction_id)),
    )
    .unwrap();
    let plan = fix.plan().unwrap();
    let outcome =
        execute_validated_installation_recovery(&fix.request, &fix.context(), plan).unwrap();
    assert!(matches!(
        outcome,
        InstallationRecoveryExecutionOutcome::Recovered {
            disposition: crate::installation_recovery::InstallationRecoveryDisposition::RemoveStagingThenIntent
        }
    ));
    assert!(fix.intent_store.load().unwrap().is_none());
}

#[test]
fn j24k3d2_second_idle_call_performs_no_mutation() {
    let fix = LightFixture::new();
    let first = fix.plan().unwrap();
    execute_validated_installation_recovery(&fix.request, &fix.context(), first).unwrap();
    let before = tree_snapshot(&fix.base);
    let second = fix.plan().unwrap();
    assert_eq!(
        execute_validated_installation_recovery(&fix.request, &fix.context(), second).unwrap(),
        InstallationRecoveryExecutionOutcome::Idle
    );
    assert_eq!(before, tree_snapshot(&fix.base));
}

#[test]
fn j24k3d2_unrelated_stores_remain_unchanged() {
    let fix = FullFixture::new();
    fix.intent_store.create(&fix.intent).unwrap();
    fix.build_destination();
    let unrelated_before = [
        tree_snapshot(&fix.quarantine_root),
        tree_snapshot(&fix.base.join("candidates")),
        tree_snapshot(&fix.base.join("trust")),
        tree_snapshot(&fix.base.join("profiles")),
        tree_snapshot(&fix.base.join("conformance")),
        tree_snapshot(&fix.base.join("approvals")),
    ];
    let plan = fix.plan().unwrap();
    execute_validated_installation_recovery(&fix.request, &fix.context(), plan).unwrap();
    let unrelated_after = [
        tree_snapshot(&fix.quarantine_root),
        tree_snapshot(&fix.base.join("candidates")),
        tree_snapshot(&fix.base.join("trust")),
        tree_snapshot(&fix.base.join("profiles")),
        tree_snapshot(&fix.base.join("conformance")),
        tree_snapshot(&fix.base.join("approvals")),
    ];
    assert_eq!(unrelated_before, unrelated_after);
}

#[test]
fn j24k3d2_recovery_never_adopts_or_deletes_final_destination() {
    let fix = FullFixture::new();
    fix.intent_store.create(&fix.intent).unwrap();
    fix.build_destination();
    let destination_before = tree_snapshot(&fix.install_root);
    let plan = fix.plan().unwrap();
    execute_validated_installation_recovery(&fix.request, &fix.context(), plan).unwrap();
    assert_eq!(destination_before, tree_snapshot(&fix.install_root));
}

#[test]
fn j24k3d2_never_runs_ordinary_installation_execution() {
    let fix = LightFixture::new();
    let intent = InstallationPublicationIntent::from_precomputed_record(valid_record()).unwrap();
    fix.intent_store.create(&intent).unwrap();
    let plan = fix.plan().unwrap();
    execute_validated_installation_recovery(&fix.request, &fix.context(), plan).unwrap();
    assert!(fix.installed.load_all().unwrap().is_empty());
    assert!(!fix
        .install_root
        .join(&intent.destination_relative_path)
        .exists());
}
