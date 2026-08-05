use super::installation_recovery_evidence::InstallationRecoveryEvidenceContext;
use super::installation_recovery_plan::{
    plan_installation_recovery, InstallationRecoveryPlanningContext,
    ValidatedInstallationRecoveryPlan,
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
use crate::installation_recovery::InstallationRecoveryDisposition;
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
use std::time::{Duration, SystemTime};
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
        host_build_identity: "j24k3d1-test".to_owned(),
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
        created_unix_ms: 1,
        record_digest: String::new(),
    };
    let mut covered = record.clone();
    covered.record_digest.clear();
    record.record_digest = sha256(&canonical(&covered).unwrap());
    record.validate().unwrap();
    record
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
            size_bytes: 32,
            role: "package_descriptor".into(),
        },
        payloads: vec![PayloadEvidence {
            path: "lib/engine.dll".into(),
            sha256: digest.into(),
            size_bytes: 64,
            role: "payload".into(),
        }],
        signature_files: vec![],
        capability_manifests: vec![],
        trust_evidence: trust,
        installation_approval_id: Uuid::new_v4().to_string(),
        installation_approval_digest: digest.into(),
        conformance_evidence_id: Uuid::new_v4().to_string(),
        conformance_evidence_digest: digest.into(),
        provider_id: "tethers.file-tools".into(),
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
        created_unix_ms: 1,
        record_digest: String::new(),
    };
    let mut covered = record.clone();
    covered.record_digest.clear();
    record.record_digest = sha256(&canonical(&covered).unwrap());
    record.validate().unwrap();
    record
}

fn write_record(record_root: &Path, record: &InstalledPlugRecord) {
    let path = record_root.join(format!("{}.json", record.installed_id));
    let bytes = canonical(record).unwrap();
    fs::write(path, bytes).unwrap();
}

fn make_writable(path: &Path) {
    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_readonly(false);
    fs::set_permissions(path, permissions).unwrap();
}

struct LightFixture {
    base: PathBuf,
    intent_store: InstallationPublicationIntentStore,
    installed: InstalledPlugRegistry,
    install_root: PathBuf,
    record_root: PathBuf,
}

impl LightFixture {
    fn new() -> Self {
        let base = std::env::temp_dir().join(format!("tethers-j24k3d1-light-{}", Uuid::new_v4()));
        fs::create_dir_all(&base).unwrap();
        let install_root = base.join("install");
        let record_root = base.join("records");
        fs::create_dir_all(&install_root).unwrap();
        fs::create_dir_all(&record_root).unwrap();
        let intent_store = InstallationPublicationIntentStore::open(&base).unwrap();
        let installed = InstalledPlugRegistry::open_existing(&install_root, &record_root).unwrap();
        Self {
            base,
            intent_store,
            installed,
            install_root,
            record_root,
        }
    }
}

// Full fixture with evidence stores for destination-related dispositions.
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
    launch: LaunchProfileEvidence,
    conformance_evidence: ConformanceEvidence,
    approval: crate::installed::InstallationApprovalRecord,
    installed_record: InstalledPlugRecord,
    intent_store: InstallationPublicationIntentStore,
    installed: InstalledPlugRegistry,
    install_root: PathBuf,
    record_root: PathBuf,
}

impl FullFixture {
    fn new() -> Self {
        let base = std::env::temp_dir().join(format!("tethers-j24k3d1-full-{}", Uuid::new_v4()));
        fs::create_dir_all(&base).unwrap();

        let archive = base.join("test.tetherplug");
        fs::write(
            &archive,
            crate::pdf_tools::build_reference_package(b"j24k3d1-test").unwrap(),
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
            .create(&candidate, &request, "j24k3d1-authority")
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
                "j24k3d1-authority",
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

        let intent_store = InstallationPublicationIntentStore::open(&base.join("intents")).unwrap();
        let installed = InstalledPlugRegistry::open_existing(&install_root, &record_root).unwrap();

        Self {
            base,
            quarantined_root: quarantined.directory.clone(),
            quarantine_root,
            candidates,
            exact_trust,
            launch_profiles,
            conformance,
            approvals,
            request,
            intent,
            launch,
            conformance_evidence,
            approval,
            installed_record,
            intent_store,
            installed,
            install_root,
            record_root,
        }
    }

    fn evidence_context(&self) -> InstallationRecoveryEvidenceContext<'_> {
        InstallationRecoveryEvidenceContext {
            quarantine_root: &self.quarantine_root,
            candidates: &self.candidates,
            exact_trust: &self.exact_trust,
            launch_profiles: &self.launch_profiles,
            conformance: &self.conformance,
            approvals: &self.approvals,
        }
    }

    fn context(&self) -> InstallationRecoveryPlanningContext<'_> {
        InstallationRecoveryPlanningContext {
            intents: &self.intent_store,
            installed: &self.installed,
            evidence: self.evidence_context(),
        }
    }

    fn plan(&self) -> crate::m3_store::Result<ValidatedInstallationRecoveryPlan> {
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
            let bytes = fs::read(&source).unwrap();
            let target = destination.join(&evidence.path);
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(&target, &bytes).unwrap();
            let mut permissions = fs::metadata(&target).unwrap().permissions();
            permissions.set_readonly(true);
            fs::set_permissions(&target, permissions).unwrap();
        }
    }
}

fn snapshot_root(root: &Path) -> BTreeMap<String, (String, u128, bool)> {
    fn visit(root: &Path, path: &Path, out: &mut BTreeMap<String, (String, u128, bool)>) {
        if !path.is_dir() {
            return;
        }
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let entry_path = entry.path();
            let relative = entry_path
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            if entry_path.is_dir() {
                out.insert(relative, (String::new(), 0, false));
                visit(root, &entry_path, out);
            } else {
                let metadata = entry_path.metadata().unwrap();
                out.insert(
                    relative,
                    (
                        sha256(&fs::read(&entry_path).unwrap()),
                        metadata
                            .modified()
                            .unwrap()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap()
                            .as_millis(),
                        metadata.permissions().readonly(),
                    ),
                );
            }
        }
    }
    let mut out = BTreeMap::new();
    if root.exists() {
        visit(root, root, &mut out);
    }
    out
}

fn full_snapshot(
    intent_root: &Path,
    install_root: &Path,
    record_root: &Path,
    quarantine_root: &Path,
    candidate_root: &Path,
    trust_root: &Path,
    launch_root: &Path,
    conformance_root: &Path,
    approval_root: &Path,
) -> BTreeMap<String, BTreeMap<String, (String, u128, bool)>> {
    let mut all = BTreeMap::new();
    all.insert("intent".into(), snapshot_root(intent_root));
    all.insert("install".into(), snapshot_root(install_root));
    all.insert("record".into(), snapshot_root(record_root));
    all.insert("quarantine".into(), snapshot_root(quarantine_root));
    all.insert("candidates".into(), snapshot_root(candidate_root));
    all.insert("trust".into(), snapshot_root(trust_root));
    all.insert("launch".into(), snapshot_root(launch_root));
    all.insert("conformance".into(), snapshot_root(conformance_root));
    all.insert("approval".into(), snapshot_root(approval_root));
    all
}

#[cfg(windows)]
fn create_directory_link(link: &Path, target: &Path) {
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
    assert!(
        status.success(),
        "could not create Windows junction fixture"
    );
}

#[cfg(unix)]
fn create_directory_link(link: &Path, target: &Path) {
    std::os::unix::fs::symlink(target, link).unwrap();
}

fn assert_unrelated_stores_unchanged(
    before: &BTreeMap<String, BTreeMap<String, (String, u128, bool)>>,
    after: &BTreeMap<String, BTreeMap<String, (String, u128, bool)>>,
) {
    for store in [
        "intent",
        "quarantine",
        "candidates",
        "trust",
        "launch",
        "conformance",
        "approval",
    ] {
        assert_eq!(
            before.get(store),
            after.get(store),
            "store changed: {store}"
        );
    }
}

#[test]
#[cfg_attr(unix, ignore = "Windows junction regression")]
#[cfg_attr(windows, allow(clippy::needless_borrow))]
fn j24k3d1_destination_junction_is_rejected_at_planner_entry() {
    let fix = FullFixture::new();
    fix.intent_store.create(&fix.intent).unwrap();
    fix.build_destination();
    let destination = fix.install_root.join(&fix.intent.destination_relative_path);
    let target = fix.base.join("destination-target");
    fs::rename(&destination, &target).unwrap();
    create_directory_link(&destination, &target);

    let before = full_snapshot(
        fix.intent_store.root_path(),
        &fix.install_root,
        &fix.record_root,
        &fix.quarantine_root,
        &fix.base.join("candidates"),
        &fix.base.join("trust"),
        &fix.base.join("profiles"),
        &fix.base.join("conformance"),
        &fix.base.join("approvals"),
    );
    let error = fix.plan().unwrap_err();
    assert_eq!(error.code, "unsafe_store_path");
    let after = full_snapshot(
        fix.intent_store.root_path(),
        &fix.install_root,
        &fix.record_root,
        &fix.quarantine_root,
        &fix.base.join("candidates"),
        &fix.base.join("trust"),
        &fix.base.join("profiles"),
        &fix.base.join("conformance"),
        &fix.base.join("approvals"),
    );
    assert_unrelated_stores_unchanged(&before, &after);
}

#[test]
#[cfg_attr(windows, ignore = "Unix symbolic-link regression")]
#[cfg_attr(unix, allow(clippy::needless_borrow))]
fn j24k3d1_destination_symlink_is_rejected_at_planner_entry() {
    let fix = FullFixture::new();
    fix.intent_store.create(&fix.intent).unwrap();
    fix.build_destination();
    let destination = fix.install_root.join(&fix.intent.destination_relative_path);
    let target = fix.base.join("destination-target");
    fs::rename(&destination, &target).unwrap();
    create_directory_link(&destination, &target);

    let error = fix.plan().unwrap_err();
    assert_eq!(error.code, "unsafe_store_path");
}

#[test]
fn j24k3d1_removed_open_install_root_returns_recovery_io() {
    let fix = FullFixture::new();
    fs::remove_dir_all(&fix.install_root).unwrap();
    let error = fix.plan().unwrap_err();
    assert_eq!(error.code, "installation_recovery_io");
}

#[test]
#[cfg_attr(unix, ignore = "Windows junction regression")]
fn j24k3d1_record_root_junction_is_rejected_at_planner_entry() {
    let fix = FullFixture::new();
    fix.intent_store.create(&fix.intent).unwrap();
    let target = fix.base.join("record-target");
    fs::create_dir(&target).unwrap();
    fs::remove_dir(&fix.record_root).unwrap();
    create_directory_link(&fix.record_root, &target);

    let error = fix.plan().unwrap_err();
    assert_eq!(error.code, "unsafe_store_path");
}

#[test]
#[cfg_attr(windows, ignore = "Unix symbolic-link regression")]
fn j24k3d1_record_root_symlink_is_rejected_at_planner_entry() {
    let fix = FullFixture::new();
    fix.intent_store.create(&fix.intent).unwrap();
    let target = fix.base.join("record-target");
    fs::create_dir(&target).unwrap();
    fs::remove_dir(&fix.record_root).unwrap();
    create_directory_link(&fix.record_root, &target);

    let error = fix.plan().unwrap_err();
    assert_eq!(error.code, "unsafe_store_path");
}

#[test]
fn j24k3d1_empty_roots_return_idle_plan() {
    let fix = LightFixture::new();
    let context = InstallationRecoveryPlanningContext {
        intents: &fix.intent_store,
        installed: &fix.installed,
        evidence: InstallationRecoveryEvidenceContext {
            quarantine_root: fix.base.as_path(),
            candidates: &CandidateRegistry::open(
                &fix.base.join("no-candidates"),
                fix.base.as_path(),
            )
            .unwrap(),
            exact_trust: &ExactCandidateTrustStore::open(&fix.base.join("no-trust")).unwrap(),
            launch_profiles: &LaunchProfileEvidenceStore::open(&fix.base.join("no-launch"))
                .unwrap(),
            conformance: &ConformanceEvidenceStore::open(&fix.base.join("no-conformance")).unwrap(),
            approvals: &InstallationApprovalStore::open(&fix.base.join("no-approvals")).unwrap(),
        },
    };
    let request = complete_request(&Uuid::new_v4().to_string());
    let plan = plan_installation_recovery(&request, &context).unwrap();
    assert!(plan.is_idle());
    assert!(plan.intent().is_none());
    assert!(plan.disposition().is_none());
}

#[test]
fn j24k3d1_no_intent_untracked_destination_fails() {
    let fix = LightFixture::new();
    let orphan = fix
        .install_root
        .join("plug-00000000-0000-0000-0000-000000000000");
    fs::create_dir(&orphan).unwrap();
    let context = InstallationRecoveryPlanningContext {
        intents: &fix.intent_store,
        installed: &fix.installed,
        evidence: InstallationRecoveryEvidenceContext {
            quarantine_root: fix.base.as_path(),
            candidates: &CandidateRegistry::open(&fix.base.join("candidates"), fix.base.as_path())
                .unwrap(),
            exact_trust: &ExactCandidateTrustStore::open(&fix.base.join("trust")).unwrap(),
            launch_profiles: &LaunchProfileEvidenceStore::open(&fix.base.join("launch")).unwrap(),
            conformance: &ConformanceEvidenceStore::open(&fix.base.join("conformance")).unwrap(),
            approvals: &InstallationApprovalStore::open(&fix.base.join("approvals")).unwrap(),
        },
    };
    let request = complete_request(&Uuid::new_v4().to_string());
    let err = plan_installation_recovery(&request, &context).unwrap_err();
    assert_eq!(err.code, "installation_destination_untracked");
}

#[test]
fn j24k3d1_malformed_intent_fails_before_audit() {
    let fix = LightFixture::new();
    let intent_store = InstallationPublicationIntentStore::open(&fix.base).unwrap();
    let intent_root = fix.base.join("installation-intent");
    fs::create_dir_all(&intent_root).unwrap();
    fs::write(intent_root.join("current.json"), b"{").unwrap();
    let context = InstallationRecoveryPlanningContext {
        intents: &intent_store,
        installed: &fix.installed,
        evidence: InstallationRecoveryEvidenceContext {
            quarantine_root: fix.base.as_path(),
            candidates: &CandidateRegistry::open(&fix.base.join("candidates"), fix.base.as_path())
                .unwrap(),
            exact_trust: &ExactCandidateTrustStore::open(&fix.base.join("trust")).unwrap(),
            launch_profiles: &LaunchProfileEvidenceStore::open(&fix.base.join("launch")).unwrap(),
            conformance: &ConformanceEvidenceStore::open(&fix.base.join("conformance")).unwrap(),
            approvals: &InstallationApprovalStore::open(&fix.base.join("approvals")).unwrap(),
        },
    };
    let request = complete_request(&Uuid::new_v4().to_string());
    let err = plan_installation_recovery(&request, &context).unwrap_err();
    assert_eq!(err.code, "installation_intent_invalid");
}

#[test]
fn j24k3d1_intent_only_returns_remove_intent() {
    let fix = LightFixture::new();
    let record = valid_record();
    let intent = InstallationPublicationIntent::from_precomputed_record(record).unwrap();
    fix.intent_store.create(&intent).unwrap();
    let context = InstallationRecoveryPlanningContext {
        intents: &fix.intent_store,
        installed: &fix.installed,
        evidence: InstallationRecoveryEvidenceContext {
            quarantine_root: fix.base.as_path(),
            candidates: &CandidateRegistry::open(&fix.base.join("candidates"), fix.base.as_path())
                .unwrap(),
            exact_trust: &ExactCandidateTrustStore::open(&fix.base.join("trust")).unwrap(),
            launch_profiles: &LaunchProfileEvidenceStore::open(&fix.base.join("launch")).unwrap(),
            conformance: &ConformanceEvidenceStore::open(&fix.base.join("conformance")).unwrap(),
            approvals: &InstallationApprovalStore::open(&fix.base.join("approvals")).unwrap(),
        },
    };
    let request = complete_request(&Uuid::new_v4().to_string());
    let plan = plan_installation_recovery(&request, &context).unwrap();
    assert!(!plan.is_idle());
    assert_eq!(
        plan.disposition(),
        Some(InstallationRecoveryDisposition::RemoveIntentOnly)
    );
}

#[test]
fn j24k3d1_staging_only_returns_remove_staging_then_intent() {
    let fix = LightFixture::new();
    let record = valid_record();
    let intent = InstallationPublicationIntent::from_precomputed_record(record).unwrap();
    fix.intent_store.create(&intent).unwrap();
    let staging_path = fix
        .install_root
        .join(format!(".staging-{}", intent.transaction_id));
    fs::create_dir(&staging_path).unwrap();
    let context = InstallationRecoveryPlanningContext {
        intents: &fix.intent_store,
        installed: &fix.installed,
        evidence: InstallationRecoveryEvidenceContext {
            quarantine_root: fix.base.as_path(),
            candidates: &CandidateRegistry::open(&fix.base.join("candidates"), fix.base.as_path())
                .unwrap(),
            exact_trust: &ExactCandidateTrustStore::open(&fix.base.join("trust")).unwrap(),
            launch_profiles: &LaunchProfileEvidenceStore::open(&fix.base.join("launch")).unwrap(),
            conformance: &ConformanceEvidenceStore::open(&fix.base.join("conformance")).unwrap(),
            approvals: &InstallationApprovalStore::open(&fix.base.join("approvals")).unwrap(),
        },
    };
    let request = complete_request(&Uuid::new_v4().to_string());
    let plan = plan_installation_recovery(&request, &context).unwrap();
    assert_eq!(
        plan.disposition(),
        Some(InstallationRecoveryDisposition::RemoveStagingThenIntent)
    );
}

#[test]
fn j24k3d1_destination_only_with_evidence_returns_revalidate_then_publish() {
    let fix = FullFixture::new();
    fix.intent_store.create(&fix.intent).unwrap();
    fix.build_destination();
    let plan = fix.plan().unwrap();
    assert_eq!(
        plan.disposition(),
        Some(InstallationRecoveryDisposition::RevalidateDestinationThenPublishRecord)
    );
}

#[test]
fn j24k3d1_completed_publication_returns_verify_then_remove_intent() {
    let fix = FullFixture::new();
    fix.intent_store.create(&fix.intent).unwrap();
    fix.build_destination();
    write_record(&fix.record_root, &fix.installed_record);
    let plan = fix.plan().unwrap();
    assert_eq!(
        plan.disposition(),
        Some(InstallationRecoveryDisposition::VerifyCompletedPublicationThenRemoveIntent)
    );
}

#[test]
fn j24k3d1_staging_and_destination_fails_conflict() {
    let fix = LightFixture::new();
    let record = valid_record();
    let intent = InstallationPublicationIntent::from_precomputed_record(record).unwrap();
    fix.intent_store.create(&intent).unwrap();
    let staging_path = fix
        .install_root
        .join(format!(".staging-{}", intent.transaction_id));
    fs::create_dir(&staging_path).unwrap();
    let dest_path = fix.install_root.join(&intent.destination_relative_path);
    fs::create_dir(&dest_path).unwrap();
    let context = InstallationRecoveryPlanningContext {
        intents: &fix.intent_store,
        installed: &fix.installed,
        evidence: InstallationRecoveryEvidenceContext {
            quarantine_root: fix.base.as_path(),
            candidates: &CandidateRegistry::open(&fix.base.join("candidates"), fix.base.as_path())
                .unwrap(),
            exact_trust: &ExactCandidateTrustStore::open(&fix.base.join("trust")).unwrap(),
            launch_profiles: &LaunchProfileEvidenceStore::open(&fix.base.join("launch")).unwrap(),
            conformance: &ConformanceEvidenceStore::open(&fix.base.join("conformance")).unwrap(),
            approvals: &InstallationApprovalStore::open(&fix.base.join("approvals")).unwrap(),
        },
    };
    let request = complete_request(&Uuid::new_v4().to_string());
    let err = plan_installation_recovery(&request, &context).unwrap_err();
    assert_eq!(err.code, "installation_recovery_conflict");
}

#[test]
fn j24k3d1_record_without_destination_fails_closed() {
    let fix = LightFixture::new();
    let record = valid_record();
    let intent = InstallationPublicationIntent::from_precomputed_record(record.clone()).unwrap();
    fix.intent_store.create(&intent).unwrap();
    write_record(&fix.record_root, &record);
    let context = InstallationRecoveryPlanningContext {
        intents: &fix.intent_store,
        installed: &fix.installed,
        evidence: InstallationRecoveryEvidenceContext {
            quarantine_root: fix.base.as_path(),
            candidates: &CandidateRegistry::open(&fix.base.join("candidates"), fix.base.as_path())
                .unwrap(),
            exact_trust: &ExactCandidateTrustStore::open(&fix.base.join("trust")).unwrap(),
            launch_profiles: &LaunchProfileEvidenceStore::open(&fix.base.join("launch")).unwrap(),
            conformance: &ConformanceEvidenceStore::open(&fix.base.join("conformance")).unwrap(),
            approvals: &InstallationApprovalStore::open(&fix.base.join("approvals")).unwrap(),
        },
    };
    let request = complete_request(&Uuid::new_v4().to_string());
    let err = plan_installation_recovery(&request, &context).unwrap_err();
    assert_eq!(err.code, "installation_recovery_conflict");
}

#[test]
fn j24k3d1_untracked_sibling_destination_fails() {
    let fix = FullFixture::new();
    fix.intent_store.create(&fix.intent).unwrap();
    fix.build_destination();
    let orphan = fix
        .install_root
        .join("plug-00000000-0000-0000-0000-000000000000");
    fs::create_dir(&orphan).unwrap();
    let err = fix.plan().unwrap_err();
    assert_eq!(err.code, "installation_destination_untracked");
}

#[test]
fn j24k3d1_stale_request_pin_fails_evidence_stale() {
    let mut fix = FullFixture::new();
    fix.intent_store.create(&fix.intent).unwrap();
    fix.build_destination();
    fix.request.schema = "other".into();
    let err = fix.plan().unwrap_err();
    assert_eq!(err.code, "installation_intent_evidence_stale");
}

#[test]
fn j24k3d1_stale_candidate_pin_fails_evidence_stale() {
    let mut fix = FullFixture::new();
    fix.intent_store.create(&fix.intent).unwrap();
    fix.build_destination();
    fix.request.candidate_id = Uuid::new_v4().to_string();
    let err = fix.plan().unwrap_err();
    assert_eq!(err.code, "installation_intent_evidence_stale");
}

#[test]
fn j24k3d1_stale_trust_pin_fails_evidence_stale() {
    let fix = FullFixture::new();
    fix.intent_store.create(&fix.intent).unwrap();
    fix.build_destination();
    let other_candidate_id = Uuid::new_v4().to_string();
    let other_request = complete_request(&other_candidate_id);
    let wrong_context = InstallationRecoveryPlanningContext {
        intents: &fix.intent_store,
        installed: &fix.installed,
        evidence: fix.evidence_context(),
    };
    let err = plan_installation_recovery(&other_request, &wrong_context).unwrap_err();
    assert_eq!(err.code, "installation_intent_evidence_stale");
}

#[test]
fn j24k3d1_stale_launch_pin_fails_evidence_stale() {
    let fix = FullFixture::new();
    fix.intent_store.create(&fix.intent).unwrap();
    fix.build_destination();
    let empty_launch = LaunchProfileEvidenceStore::open(&fix.base.join("empty-launch")).unwrap();
    let context = InstallationRecoveryPlanningContext {
        intents: &fix.intent_store,
        installed: &fix.installed,
        evidence: InstallationRecoveryEvidenceContext {
            quarantine_root: &fix.quarantine_root,
            candidates: &fix.candidates,
            exact_trust: &fix.exact_trust,
            launch_profiles: &empty_launch,
            conformance: &fix.conformance,
            approvals: &fix.approvals,
        },
    };
    let err = plan_installation_recovery(&fix.request, &context).unwrap_err();
    assert_eq!(err.code, "installation_intent_evidence_stale");
}

#[test]
fn j24k3d1_stale_conformance_pin_fails_evidence_stale() {
    let fix = FullFixture::new();
    fix.intent_store.create(&fix.intent).unwrap();
    fix.build_destination();
    let empty_conformance =
        ConformanceEvidenceStore::open(&fix.base.join("empty-conformance")).unwrap();
    let context = InstallationRecoveryPlanningContext {
        intents: &fix.intent_store,
        installed: &fix.installed,
        evidence: InstallationRecoveryEvidenceContext {
            quarantine_root: &fix.quarantine_root,
            candidates: &fix.candidates,
            exact_trust: &fix.exact_trust,
            launch_profiles: &fix.launch_profiles,
            conformance: &empty_conformance,
            approvals: &fix.approvals,
        },
    };
    let err = plan_installation_recovery(&fix.request, &context).unwrap_err();
    assert_eq!(err.code, "installation_intent_evidence_stale");
}

#[test]
fn j24k3d1_stale_approval_pin_fails_evidence_stale() {
    let fix = FullFixture::new();
    fix.intent_store.create(&fix.intent).unwrap();
    fix.build_destination();
    let empty_approvals =
        InstallationApprovalStore::open(&fix.base.join("empty-approvals")).unwrap();
    let context = InstallationRecoveryPlanningContext {
        intents: &fix.intent_store,
        installed: &fix.installed,
        evidence: InstallationRecoveryEvidenceContext {
            quarantine_root: &fix.quarantine_root,
            candidates: &fix.candidates,
            exact_trust: &fix.exact_trust,
            launch_profiles: &fix.launch_profiles,
            conformance: &fix.conformance,
            approvals: &empty_approvals,
        },
    };
    let err = plan_installation_recovery(&fix.request, &context).unwrap_err();
    assert_eq!(err.code, "installation_intent_evidence_stale");
}

#[test]
fn j24k3d1_completed_with_stale_installed_record_fails_evidence_stale() {
    let fix = FullFixture::new();
    fix.intent_store.create(&fix.intent).unwrap();
    fix.build_destination();
    write_record(&fix.record_root, &fix.installed_record);
    let mut tampered_record = fix.installed_record.clone();
    tampered_record.package_id = "other.package".into();
    let mut covered_record = tampered_record.clone();
    covered_record.record_digest.clear();
    tampered_record.record_digest = sha256(&canonical(&covered_record).unwrap());
    tampered_record.validate().unwrap();
    let tampered_intent =
        InstallationPublicationIntent::from_precomputed_record(tampered_record.clone()).unwrap();
    fix.intent_store.remove_if_matches(&fix.intent).unwrap();
    fix.intent_store.create(&tampered_intent).unwrap();
    fs::remove_file(
        fix.record_root
            .join(format!("{}.json", fix.installed_record.installed_id)),
    )
    .unwrap();
    write_record(&fix.record_root, &tampered_record);
    let err = fix.plan().unwrap_err();
    assert_eq!(err.code, "installation_intent_evidence_stale");
}

#[test]
fn j24k3d1_destination_file_set_drift_fails_recovery_conflict() {
    let fix = FullFixture::new();
    fix.intent_store.create(&fix.intent).unwrap();
    let destination = fix.install_root.join(&fix.intent.destination_relative_path);
    fs::create_dir(&destination).unwrap();
    let extra = destination.join("unexpected.txt");
    fs::write(&extra, b"extra").unwrap();
    let err = fix.plan().unwrap_err();
    assert_eq!(err.code, "installation_recovery_conflict");
}

#[test]
fn j24k3d1_destination_digest_drift_fails_recovery_conflict() {
    let fix = FullFixture::new();
    fix.intent_store.create(&fix.intent).unwrap();
    fix.build_destination();
    let first_payload = fix
        .install_root
        .join(&fix.intent.destination_relative_path)
        .join(&fix.intent.installed_record.plug_json.path);
    make_writable(&first_payload);
    fs::write(&first_payload, b"corrupted content").unwrap();
    let err = fix.plan().unwrap_err();
    assert_eq!(err.code, "installation_recovery_conflict");
}

#[test]
fn j24k3d1_destination_size_drift_fails_recovery_conflict() {
    let fix = FullFixture::new();
    fix.intent_store.create(&fix.intent).unwrap();
    fix.build_destination();
    let first_payload = fix
        .install_root
        .join(&fix.intent.destination_relative_path)
        .join(&fix.intent.installed_record.plug_json.path);
    make_writable(&first_payload);
    fs::write(&first_payload, b"too short").unwrap();
    let err = fix.plan().unwrap_err();
    assert_eq!(err.code, "installation_recovery_conflict");
}

#[test]
fn j24k3d1_destination_permission_drift_fails_recovery_conflict() {
    let fix = FullFixture::new();
    fix.intent_store.create(&fix.intent).unwrap();
    fix.build_destination();
    let first_payload = fix
        .install_root
        .join(&fix.intent.destination_relative_path)
        .join(&fix.intent.installed_record.plug_json.path);
    let mut permissions = fs::metadata(&first_payload).unwrap().permissions();
    permissions.set_readonly(false);
    fs::set_permissions(&first_payload, permissions).unwrap();
    let err = fix.plan().unwrap_err();
    assert_eq!(err.code, "installation_recovery_conflict");
}

#[test]
fn j24k3d1_completed_publication_still_requires_evidence_and_destination() {
    let mut fix = FullFixture::new();
    fix.intent_store.create(&fix.intent).unwrap();
    fix.build_destination();
    write_record(&fix.record_root, &fix.installed_record);
    fix.request.schema = "other".into();
    let err = fix.plan().unwrap_err();
    assert_eq!(err.code, "installation_intent_evidence_stale");
}

#[test]
fn j24k3d1_idle_plan_has_no_intent_or_disposition() {
    let fix = LightFixture::new();
    let context = InstallationRecoveryPlanningContext {
        intents: &fix.intent_store,
        installed: &fix.installed,
        evidence: InstallationRecoveryEvidenceContext {
            quarantine_root: fix.base.as_path(),
            candidates: &CandidateRegistry::open(&fix.base.join("candidates"), fix.base.as_path())
                .unwrap(),
            exact_trust: &ExactCandidateTrustStore::open(&fix.base.join("trust")).unwrap(),
            launch_profiles: &LaunchProfileEvidenceStore::open(&fix.base.join("launch")).unwrap(),
            conformance: &ConformanceEvidenceStore::open(&fix.base.join("conformance")).unwrap(),
            approvals: &InstallationApprovalStore::open(&fix.base.join("approvals")).unwrap(),
        },
    };
    let request = complete_request(&Uuid::new_v4().to_string());
    let plan = plan_installation_recovery(&request, &context).unwrap();
    assert!(plan.is_idle());
    assert!(plan.intent().is_none());
    assert!(plan.disposition().is_none());
}

#[test]
fn j24k3d1_staging_only_does_not_require_evidence_stores() {
    let fix = LightFixture::new();
    let record = valid_record();
    let intent = InstallationPublicationIntent::from_precomputed_record(record).unwrap();
    fix.intent_store.create(&intent).unwrap();
    let staging_path = fix
        .install_root
        .join(format!(".staging-{}", intent.transaction_id));
    fs::create_dir(&staging_path).unwrap();
    let context = InstallationRecoveryPlanningContext {
        intents: &fix.intent_store,
        installed: &fix.installed,
        evidence: InstallationRecoveryEvidenceContext {
            quarantine_root: fix.base.as_path(),
            candidates: &CandidateRegistry::open(&fix.base.join("candidates"), fix.base.as_path())
                .unwrap(),
            exact_trust: &ExactCandidateTrustStore::open(&fix.base.join("trust")).unwrap(),
            launch_profiles: &LaunchProfileEvidenceStore::open(&fix.base.join("launch")).unwrap(),
            conformance: &ConformanceEvidenceStore::open(&fix.base.join("conformance")).unwrap(),
            approvals: &InstallationApprovalStore::open(&fix.base.join("approvals")).unwrap(),
        },
    };
    let request = complete_request(&Uuid::new_v4().to_string());
    let plan = plan_installation_recovery(&request, &context).unwrap();
    assert_eq!(
        plan.disposition(),
        Some(InstallationRecoveryDisposition::RemoveStagingThenIntent)
    );
}

#[test]
fn j24k3d1_successful_planning_route_is_read_only() {
    let fix = FullFixture::new();
    fix.intent_store.create(&fix.intent).unwrap();
    fix.build_destination();
    write_record(&fix.record_root, &fix.installed_record);

    let before = full_snapshot(
        fix.intent_store.root_path(),
        &fix.install_root,
        &fix.record_root,
        &fix.quarantine_root,
        &fix.base.join("candidates"),
        &fix.base.join("trust"),
        &fix.base.join("profiles"),
        &fix.base.join("conformance"),
        &fix.base.join("approvals"),
    );

    let plan = fix.plan().unwrap();
    assert!(!plan.is_idle());

    let after = full_snapshot(
        fix.intent_store.root_path(),
        &fix.install_root,
        &fix.record_root,
        &fix.quarantine_root,
        &fix.base.join("candidates"),
        &fix.base.join("trust"),
        &fix.base.join("profiles"),
        &fix.base.join("conformance"),
        &fix.base.join("approvals"),
    );

    assert_eq!(before, after);
}
