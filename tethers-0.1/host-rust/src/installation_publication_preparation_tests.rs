use super::installation_publication_preparation::prepare_disabled_installation_publication;
use crate::candidate::{extract_to_quarantine, CandidateRecord, CandidateRegistry};
use crate::conformance::{
    current_suite_digest, CaseDisposition, ConformanceCaseEvidence, ConformanceDisposition,
    ConformanceEvidence, ConformanceEvidenceStore,
};
use crate::current_trust::ExactCandidateTrustAuthority;
use crate::installation_plan::{plan_installation, InstallationPlan, InstallationPlanAction};
use crate::installation_publication_intent::{
    InstallationPublicationIntent, InstallationPublicationIntentStore,
};
use crate::installation_recovery_evidence::InstallationRecoveryEvidenceContext;
use crate::installation_recovery_plan::InstallationRecoveryPlanningContext;
use crate::installation_request::{
    InstallationConformanceRequest, InstallationRequest, InstallationTargetRequest,
    InstallationTargetState, InstallationTrustRequest, InstallationTrustScope,
    INSTALLATION_REQUEST_SCHEMA,
};
use crate::installation_trust::ExactCandidateTrustStore;
use crate::installed::{
    DisabledBindingRecord, InstallationApprovalRecord, InstallationApprovalStore,
    InstalledPlugRecord, InstalledPlugRegistry,
};
use crate::launch_profile::{
    LaunchProfileEvidence, LaunchProfileEvidenceStore, PreparedSupervisedLaunch,
};
use crate::m3_store::{canonical, sha256};
use crate::package;
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

fn build_conformance(
    candidate: &CandidateRecord,
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
        host_build_identity: "j24k3e1-test".to_owned(),
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

/// A complete publication-ready fixture: real archive, real quarantine, real
/// stores, and a J24J plan that is exactly `PublishDisabledInstallation`.
struct Fixture {
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
    candidate: CandidateRecord,
    trust: PackageTrustEvidence,
    launch: LaunchProfileEvidence,
    conformance_evidence: ConformanceEvidence,
    approval: InstallationApprovalRecord,
}

impl Fixture {
    fn new() -> Self {
        let base = std::env::temp_dir().join(format!("tethers-j24k3e1-{}", Uuid::new_v4()));
        fs::create_dir_all(&base).unwrap();
        let archive = base.join("test.tetherplug");
        fs::write(
            &archive,
            crate::pdf_tools::build_reference_package(b"j24k3e1-test").unwrap(),
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
            .create(&candidate, &request, "j24k3e1-authority")
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
                "j24k3e1-authority",
            )
            .unwrap();
        let install_root = base.join("install");
        let record_root = base.join("records");
        fs::create_dir_all(&install_root).unwrap();
        fs::create_dir_all(&record_root).unwrap();
        Self {
            intent_store: InstallationPublicationIntentStore::open(&base.join("intents")).unwrap(),
            installed: InstalledPlugRegistry::open_existing(&install_root, &record_root).unwrap(),
            base,
            quarantine_root,
            install_root,
            record_root,
            candidates,
            exact_trust,
            launch_profiles,
            conformance,
            approvals,
            request,
            candidate,
            trust,
            launch,
            conformance_evidence,
            approval,
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

    fn plan(&self) -> InstallationPlan {
        plan_installation(
            &self.request,
            &self.candidates,
            &self.exact_trust,
            &self.launch_profiles,
            &self.conformance,
            &self.approvals,
            &self.installed,
        )
        .unwrap()
    }

    fn installed_record(&self) -> InstalledPlugRecord {
        let installed_id = Uuid::new_v4().to_string();
        let mut record = InstalledPlugRecord {
            schema_version: 1,
            installed_id: installed_id.clone(),
            state: "present_disabled".into(),
            package_id: self.candidate.package_id.clone(),
            package_version: self.candidate.package_version.clone(),
            semantic_package_digest: self.candidate.semantic_package_digest.clone(),
            source_candidate_id: self.candidate.candidate_id.clone(),
            installation_relative_path: format!("plug-{installed_id}"),
            raw_archive_digest: self.candidate.raw_archive_digest.clone(),
            plug_json: self.candidate.plug_json.clone(),
            payloads: self.candidate.payloads.clone(),
            signature_files: self.candidate.signature_files.clone(),
            capability_manifests: self.candidate.capabilities.clone(),
            trust_evidence: self.trust.clone(),
            installation_approval_id: self.approval.approval_id.clone(),
            installation_approval_digest: self.approval.record_digest.clone(),
            conformance_evidence_id: self.conformance_evidence.evidence_id.clone(),
            conformance_evidence_digest: self.conformance_evidence.evidence_digest.clone(),
            provider_id: self.candidate.provider_id.clone(),
            provider_version: self.candidate.provider_version.clone(),
            launch_path: self.candidate.launch_path.clone(),
            launch_arguments: self.candidate.launch_arguments.clone(),
            provider_working_directory: self.candidate.provider_working_directory.clone(),
            launch_profile_label: self.launch.profile_label.clone(),
            socket_major: 1,
            mcp_protocol_version: "2025-11-25".into(),
            platform: self.candidate.selected_platform.os.clone(),
            architecture: self.candidate.selected_platform.architecture.clone(),
            disabled_bindings: self
                .candidate
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
            operational_scope_schema: self.candidate.operational_scope_schema.clone(),
            operational_scope_schema_digest: self
                .candidate
                .operational_scope_schema_digest
                .clone(),
            created_unix_ms: 1,
            record_digest: String::new(),
        };
        let mut covered = record.clone();
        covered.record_digest.clear();
        record.record_digest = sha256(&canonical(&covered).unwrap());
        record.validate().unwrap();
        record
    }

    /// Create a complete durable installed state: the exact read-only
    /// destination plus its owning record. The installed registry refuses a
    /// record whose destination is absent, so both halves are required.
    fn publish_installed(&self, record: &InstalledPlugRecord) {
        let destination = self.install_root.join(&record.installation_relative_path);
        fs::create_dir(&destination).unwrap();
        let quarantined = self
            .quarantine_root
            .join(&self.candidate.quarantine_relative_path);
        for evidence in std::iter::once(&record.plug_json)
            .chain(record.payloads.iter())
            .chain(record.signature_files.iter())
        {
            let bytes = fs::read(quarantined.join(&evidence.path)).unwrap();
            let target = destination.join(&evidence.path);
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(&target, bytes).unwrap();
            let mut permissions = fs::metadata(&target).unwrap().permissions();
            permissions.set_readonly(true);
            fs::set_permissions(&target, permissions).unwrap();
        }
        fs::write(
            self.record_root
                .join(format!("{}.json", record.installed_id)),
            canonical(record).unwrap(),
        )
        .unwrap();
    }
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

// ---------------------------------------------------------------------------
// Success path
// ---------------------------------------------------------------------------

#[test]
fn j24k3e1_publication_ready_evidence_produces_sealed_prepared_value() {
    let fix = Fixture::new();
    let before = fix.plan();
    assert_eq!(
        before.action,
        InstallationPlanAction::PublishDisabledInstallation
    );

    let prepared =
        prepare_disabled_installation_publication(&fix.request, &fix.context(), &before).unwrap();

    prepared.intent().validate().unwrap();
    prepared.installed_record().validate().unwrap();
}

#[test]
fn j24k3e1_prepared_installed_id_is_canonical_lowercase_uuid() {
    let fix = Fixture::new();
    let before = fix.plan();
    let prepared =
        prepare_disabled_installation_publication(&fix.request, &fix.context(), &before).unwrap();

    let id = &prepared.installed_record().installed_id;
    let parsed = Uuid::parse_str(id).unwrap();
    assert_eq!(parsed.hyphenated().to_string(), *id);
    assert_eq!(id.to_lowercase(), *id);
}

#[test]
fn j24k3e1_destination_is_exactly_plug_installed_id() {
    let fix = Fixture::new();
    let before = fix.plan();
    let prepared =
        prepare_disabled_installation_publication(&fix.request, &fix.context(), &before).unwrap();

    let record = prepared.installed_record();
    let expected = format!("plug-{}", record.installed_id);
    assert_eq!(record.installation_relative_path, expected);
    assert_eq!(prepared.intent().destination_relative_path, expected);
}

#[test]
fn j24k3e1_intent_record_and_destination_identity_agree() {
    let fix = Fixture::new();
    let before = fix.plan();
    let prepared =
        prepare_disabled_installation_publication(&fix.request, &fix.context(), &before).unwrap();

    let intent = prepared.intent();
    let record = prepared.installed_record();
    assert_eq!(intent.transaction_id, record.installed_id);
    assert_eq!(intent.candidate_id, record.source_candidate_id);
    assert_eq!(intent.candidate_id, fix.candidate.candidate_id);
    assert_eq!(
        intent.destination_relative_path,
        record.installation_relative_path
    );
    assert_eq!(intent.installed_record_digest, record.record_digest);
    assert_eq!(*record, intent.installed_record);
}

#[test]
fn j24k3e1_record_fields_exactly_match_pinned_evidence() {
    let fix = Fixture::new();
    let before = fix.plan();
    let prepared =
        prepare_disabled_installation_publication(&fix.request, &fix.context(), &before).unwrap();
    let record = prepared.installed_record();

    assert_eq!(record.state, "present_disabled");
    assert_eq!(record.package_id, fix.candidate.package_id);
    assert_eq!(record.package_version, fix.candidate.package_version);
    assert_eq!(
        record.semantic_package_digest,
        fix.candidate.semantic_package_digest
    );
    assert_eq!(record.raw_archive_digest, fix.candidate.raw_archive_digest);
    assert_eq!(record.plug_json, fix.candidate.plug_json);
    assert_eq!(record.payloads, fix.candidate.payloads);
    assert_eq!(record.signature_files, fix.candidate.signature_files);
    assert_eq!(record.capability_manifests, fix.candidate.capabilities);
    assert_eq!(record.trust_evidence, fix.trust);
    assert_eq!(record.installation_approval_id, fix.approval.approval_id);
    assert_eq!(
        record.installation_approval_digest,
        fix.approval.record_digest
    );
    assert_eq!(
        record.conformance_evidence_id,
        fix.conformance_evidence.evidence_id
    );
    assert_eq!(
        record.conformance_evidence_digest,
        fix.conformance_evidence.evidence_digest
    );
    assert_eq!(record.provider_id, fix.candidate.provider_id);
    assert_eq!(record.provider_version, fix.candidate.provider_version);
    assert_eq!(record.launch_path, fix.candidate.launch_path);
    assert_eq!(record.launch_arguments, fix.candidate.launch_arguments);
    assert_eq!(
        record.provider_working_directory,
        fix.candidate.provider_working_directory
    );
    assert_eq!(record.launch_profile_label, fix.launch.profile_label);
    assert_eq!(record.platform, fix.candidate.selected_platform.os);
    assert_eq!(
        record.architecture,
        fix.candidate.selected_platform.architecture
    );
    assert_eq!(record.socket_major, 1);
    assert_eq!(record.mcp_protocol_version, "2025-11-25");

    // The pinned plan digests are exactly what the record carries.
    assert_eq!(
        before.trust_evidence_digest.as_ref(),
        Some(&record.trust_evidence.evidence_digest)
    );
    assert_eq!(
        before.installation_approval_id.as_ref(),
        Some(&record.installation_approval_id)
    );
    assert_eq!(
        before.conformance_evidence_id.as_ref(),
        Some(&record.conformance_evidence_id)
    );
    assert_eq!(
        before.launch_profile_evidence_digest.as_ref(),
        Some(&fix.launch.profile_evidence_digest)
    );
}

#[test]
fn j24k3e1_disabled_bindings_match_candidate_capabilities_in_accepted_order() {
    let fix = Fixture::new();
    let before = fix.plan();
    let prepared =
        prepare_disabled_installation_publication(&fix.request, &fix.context(), &before).unwrap();

    let expected: Vec<DisabledBindingRecord> = fix
        .candidate
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

    assert!(!expected.is_empty());
    assert_eq!(prepared.installed_record().disabled_bindings, expected);
}

#[test]
fn j24k3e1_created_time_is_nonzero_and_frozen_through_validation() {
    let fix = Fixture::new();
    let before = fix.plan();
    let prepared =
        prepare_disabled_installation_publication(&fix.request, &fix.context(), &before).unwrap();

    let created = prepared.installed_record().created_unix_ms;
    assert!(created > 0);

    prepared.intent().validate().unwrap();
    prepared.installed_record().validate().unwrap();
    assert_eq!(prepared.installed_record().created_unix_ms, created);
    assert_eq!(prepared.intent().installed_record.created_unix_ms, created);
}

#[test]
fn j24k3e1_digests_remain_stable_under_repeated_validation() {
    let fix = Fixture::new();
    let before = fix.plan();
    let prepared =
        prepare_disabled_installation_publication(&fix.request, &fix.context(), &before).unwrap();

    let record_digest = prepared.installed_record().record_digest.clone();
    let intent_digest = prepared.intent().intent_digest.clone();

    for _ in 0..3 {
        prepared.intent().validate().unwrap();
        prepared.installed_record().validate().unwrap();
    }

    assert_eq!(prepared.installed_record().record_digest, record_digest);
    assert_eq!(prepared.intent().intent_digest, intent_digest);
}

#[test]
fn j24k3e1_successful_preparation_leaves_all_durable_roots_unchanged() {
    let fix = Fixture::new();
    let before = fix.plan();
    let snapshot = tree_snapshot(&fix.base);

    let prepared =
        prepare_disabled_installation_publication(&fix.request, &fix.context(), &before).unwrap();

    assert_eq!(snapshot, tree_snapshot(&fix.base));
    assert!(fix.intent_store.load().unwrap().is_none());
    assert!(fix.installed.load_all().unwrap().is_empty());
    assert!(!fix
        .install_root
        .join(&prepared.intent().destination_relative_path)
        .exists());
    assert!(!fix
        .install_root
        .join(format!(".staging-{}", prepared.intent().transaction_id))
        .exists());
    assert!(!fix
        .record_root
        .join(format!("{}.json", prepared.installed_record().installed_id))
        .exists());
}

#[test]
fn j24k3e1_no_intent_staging_destination_or_record_is_created() {
    let fix = Fixture::new();
    let before = fix.plan();

    prepare_disabled_installation_publication(&fix.request, &fix.context(), &before).unwrap();

    // The intent root remains empty, and the install/record roots have no
    // entries whatsoever.
    assert!(fix.intent_store.load().unwrap().is_none());
    assert_eq!(fs::read_dir(&fix.install_root).unwrap().count(), 0);
    assert_eq!(fs::read_dir(&fix.record_root).unwrap().count(), 0);
}

#[test]
fn j24k3e1_independent_preparations_differ_in_identity_but_both_stay_read_only() {
    let fix = Fixture::new();
    let before = fix.plan();
    let snapshot = tree_snapshot(&fix.base);

    let first =
        prepare_disabled_installation_publication(&fix.request, &fix.context(), &before).unwrap();
    let second =
        prepare_disabled_installation_publication(&fix.request, &fix.context(), &before).unwrap();

    assert_ne!(
        first.intent().transaction_id,
        second.intent().transaction_id
    );
    assert_ne!(
        first.installed_record().record_digest,
        second.installed_record().record_digest
    );

    // Each remains internally exact.
    for prepared in [&first, &second] {
        prepared.intent().validate().unwrap();
        prepared.installed_record().validate().unwrap();
        assert_eq!(
            prepared.intent().transaction_id,
            prepared.installed_record().installed_id
        );
    }

    assert_eq!(snapshot, tree_snapshot(&fix.base));
}

// ---------------------------------------------------------------------------
// Plan refusal
// ---------------------------------------------------------------------------

#[test]
fn j24k3e1_stale_before_plan_identity_is_refused_without_mutation() {
    let fix = Fixture::new();
    let mut before = fix.plan();
    before.package_version.push_str("-changed");
    let snapshot = tree_snapshot(&fix.base);

    let error = prepare_disabled_installation_publication(&fix.request, &fix.context(), &before)
        .unwrap_err();

    assert_eq!(error.code, "installation_execution_plan_stale");
    assert_eq!(snapshot, tree_snapshot(&fix.base));
}

#[test]
fn j24k3e1_forged_before_plan_pins_are_refused_without_mutation() {
    let fix = Fixture::new();
    let mut before = fix.plan();
    before.trust_evidence_digest = Some("sha256:".to_owned() + &"b".repeat(64));
    let snapshot = tree_snapshot(&fix.base);

    let error = prepare_disabled_installation_publication(&fix.request, &fix.context(), &before)
        .unwrap_err();

    assert_eq!(error.code, "installation_execution_plan_stale");
    assert_eq!(snapshot, tree_snapshot(&fix.base));
}

#[test]
fn j24k3e1_forged_installed_pins_are_refused_without_mutation() {
    let fix = Fixture::new();
    let mut before = fix.plan();
    before.installed_id = Some(Uuid::new_v4().to_string());
    let snapshot = tree_snapshot(&fix.base);

    let error = prepare_disabled_installation_publication(&fix.request, &fix.context(), &before)
        .unwrap_err();

    assert_eq!(error.code, "installation_execution_plan_stale");
    assert_eq!(snapshot, tree_snapshot(&fix.base));
}

#[test]
fn j24k3e1_wrong_before_plan_action_is_refused_without_mutation() {
    let fix = Fixture::new();
    let mut before = fix.plan();
    before.action = InstallationPlanAction::Complete;
    let snapshot = tree_snapshot(&fix.base);

    let error = prepare_disabled_installation_publication(&fix.request, &fix.context(), &before)
        .unwrap_err();

    // A changed action no longer equals fresh authority, so the plan-stale
    // guard fires first. Either refusal is fail-closed and mutation-free.
    assert_eq!(error.code, "installation_execution_plan_stale");
    assert_eq!(snapshot, tree_snapshot(&fix.base));
}

#[test]
fn j24k3e1_non_publication_action_is_refused_without_mutation() {
    // A candidate with no exact trust plans CreateExactCandidateTrust.
    let fix = Fixture::new();
    let base = std::env::temp_dir().join(format!("tethers-j24k3e1-early-{}", Uuid::new_v4()));
    fs::create_dir_all(&base).unwrap();
    let archive = base.join("test.tetherplug");
    fs::write(
        &archive,
        crate::pdf_tools::build_reference_package(b"j24k3e1-early").unwrap(),
    )
    .unwrap();
    let quarantine_root = base.join("quarantine");
    let report = package::inspect(&archive).unwrap();
    let quarantined = extract_to_quarantine(&report, &quarantine_root).unwrap();
    let candidates = CandidateRegistry::open(&base.join("candidates"), &quarantine_root).unwrap();
    let candidate = candidates.create(&quarantined).unwrap();
    let request = complete_request(&candidate.candidate_id);
    let exact_trust = ExactCandidateTrustStore::open(&base.join("trust")).unwrap();
    let launch_profiles = LaunchProfileEvidenceStore::open(&base.join("profiles")).unwrap();
    let conformance = ConformanceEvidenceStore::open(&base.join("conformance")).unwrap();
    let approvals = InstallationApprovalStore::open(&base.join("approvals")).unwrap();
    let install_root = base.join("install");
    let record_root = base.join("records");
    fs::create_dir_all(&install_root).unwrap();
    fs::create_dir_all(&record_root).unwrap();
    let installed = InstalledPlugRegistry::open_existing(&install_root, &record_root).unwrap();
    let intent_store = InstallationPublicationIntentStore::open(&base.join("intents")).unwrap();

    let before = plan_installation(
        &request,
        &candidates,
        &exact_trust,
        &launch_profiles,
        &conformance,
        &approvals,
        &installed,
    )
    .unwrap();
    assert_eq!(
        before.action,
        InstallationPlanAction::CreateExactCandidateTrust
    );

    let context = InstallationRecoveryPlanningContext {
        intents: &intent_store,
        installed: &installed,
        evidence: InstallationRecoveryEvidenceContext {
            quarantine_root: &quarantine_root,
            candidates: &candidates,
            exact_trust: &exact_trust,
            launch_profiles: &launch_profiles,
            conformance: &conformance,
            approvals: &approvals,
        },
    };
    let snapshot = tree_snapshot(&base);

    let error = prepare_disabled_installation_publication(&request, &context, &before).unwrap_err();

    assert_eq!(error.code, "installation_execution_invalid_transition");
    assert_eq!(snapshot, tree_snapshot(&base));
    drop(fix);
}

#[test]
fn j24k3e1_authoritative_plan_changed_after_before_plan_is_refused() {
    let fix = Fixture::new();
    let before = fix.plan();

    // Publish an installed record so fresh authority now plans Complete.
    fix.publish_installed(&fix.installed_record());

    let error = prepare_disabled_installation_publication(&fix.request, &fix.context(), &before)
        .unwrap_err();

    assert_eq!(error.code, "installation_execution_plan_stale");
}

// ---------------------------------------------------------------------------
// Recovery refusal
// ---------------------------------------------------------------------------

#[test]
fn j24k3e1_pending_recovery_intent_blocks_preparation_and_is_retained() {
    let fix = Fixture::new();
    let before = fix.plan();
    let intent =
        InstallationPublicationIntent::from_precomputed_record(fix.installed_record()).unwrap();
    fix.intent_store.create(&intent).unwrap();
    let snapshot = tree_snapshot(&fix.base);

    let error = prepare_disabled_installation_publication(&fix.request, &fix.context(), &before)
        .unwrap_err();

    assert_eq!(error.code, "installation_recovery_conflict");
    // The intent was neither cleaned nor adopted.
    assert_eq!(fix.intent_store.load().unwrap(), Some(intent));
    assert_eq!(snapshot, tree_snapshot(&fix.base));
}

#[test]
fn j24k3e1_torn_intent_state_blocks_preparation_and_is_retained() {
    let fix = Fixture::new();
    let before = fix.plan();
    let torn = fix.intent_store.root_path().join("current.json.tmp");
    fs::write(&torn, b"{}").unwrap();
    let snapshot = tree_snapshot(&fix.base);

    let error = prepare_disabled_installation_publication(&fix.request, &fix.context(), &before)
        .unwrap_err();

    assert_eq!(error.code, "installation_intent_invalid");
    assert!(torn.exists());
    assert_eq!(snapshot, tree_snapshot(&fix.base));
}

#[test]
fn j24k3e1_malformed_intent_blocks_preparation_and_is_retained() {
    let fix = Fixture::new();
    let before = fix.plan();
    let path = fix.intent_store.root_path().join("current.json");
    fs::write(&path, b"{\"schema_version\":1}").unwrap();
    let snapshot = tree_snapshot(&fix.base);

    let error = prepare_disabled_installation_publication(&fix.request, &fix.context(), &before)
        .unwrap_err();

    assert_eq!(error.code, "installation_intent_invalid");
    assert!(path.exists());
    assert_eq!(snapshot, tree_snapshot(&fix.base));
}

#[test]
fn j24k3e1_staging_recovery_conflict_is_not_cleaned_or_adopted() {
    let fix = Fixture::new();
    let before = fix.plan();
    let intent =
        InstallationPublicationIntent::from_precomputed_record(fix.installed_record()).unwrap();
    fix.intent_store.create(&intent).unwrap();
    let staging = fix
        .install_root
        .join(format!(".staging-{}", intent.transaction_id));
    fs::create_dir(&staging).unwrap();
    let snapshot = tree_snapshot(&fix.base);

    let error = prepare_disabled_installation_publication(&fix.request, &fix.context(), &before)
        .unwrap_err();

    assert_eq!(error.code, "installation_recovery_conflict");
    assert!(staging.exists());
    assert!(fix.intent_store.load().unwrap().is_some());
    assert_eq!(snapshot, tree_snapshot(&fix.base));
}

#[test]
fn j24k3e1_global_untracked_final_destination_blocks_preparation() {
    let fix = Fixture::new();
    let before = fix.plan();
    // An installed-looking destination with no owning record is fail-closed.
    let untracked = fix.install_root.join(format!("plug-{}", Uuid::new_v4()));
    fs::create_dir(&untracked).unwrap();
    let snapshot = tree_snapshot(&fix.base);

    let error = prepare_disabled_installation_publication(&fix.request, &fix.context(), &before)
        .unwrap_err();

    assert_eq!(error.code, "installation_destination_untracked");
    assert!(untracked.exists());
    assert_eq!(snapshot, tree_snapshot(&fix.base));
}

// ---------------------------------------------------------------------------
// Evidence refusal
// ---------------------------------------------------------------------------

#[test]
fn j24k3e1_missing_exact_trust_is_refused() {
    let fix = Fixture::new();
    let before = fix.plan();
    for entry in fs::read_dir(fix.base.join("trust")).unwrap() {
        let path = entry.unwrap().path();
        if path.is_file() {
            fs::remove_file(path).unwrap();
        }
    }

    let error = prepare_disabled_installation_publication(&fix.request, &fix.context(), &before)
        .unwrap_err();

    // Fresh authority can no longer reach the publication action.
    assert_eq!(error.code, "installation_execution_plan_stale");
}

#[test]
fn j24k3e1_drifted_approval_evidence_is_refused() {
    let fix = Fixture::new();
    let before = fix.plan();
    for entry in fs::read_dir(fix.base.join("approvals")).unwrap() {
        let path = entry.unwrap().path();
        if path.is_file() {
            fs::remove_file(path).unwrap();
        }
    }

    let error = prepare_disabled_installation_publication(&fix.request, &fix.context(), &before)
        .unwrap_err();

    assert_eq!(error.code, "installation_execution_plan_stale");
}

#[test]
fn j24k3e1_drifted_conformance_evidence_is_refused() {
    let fix = Fixture::new();
    let before = fix.plan();
    for entry in fs::read_dir(fix.base.join("conformance")).unwrap() {
        let path = entry.unwrap().path();
        if path.is_file() {
            fs::remove_file(path).unwrap();
        }
    }

    let error = prepare_disabled_installation_publication(&fix.request, &fix.context(), &before)
        .unwrap_err();

    assert_eq!(error.code, "installation_execution_plan_stale");
}

#[test]
fn j24k3e1_drifted_launch_profile_is_refused() {
    let fix = Fixture::new();
    let before = fix.plan();
    for entry in fs::read_dir(fix.base.join("profiles")).unwrap() {
        let path = entry.unwrap().path();
        if path.is_file() {
            fs::remove_file(path).unwrap();
        }
    }

    let error = prepare_disabled_installation_publication(&fix.request, &fix.context(), &before)
        .unwrap_err();

    assert_eq!(error.code, "installation_execution_plan_stale");
}

#[test]
fn j24k3e1_quarantine_byte_drift_is_refused() {
    let fix = Fixture::new();
    let before = fix.plan();
    let target = fix
        .quarantine_root
        .join(&fix.candidate.quarantine_relative_path)
        .join("plug.json");
    let mut permissions = fs::metadata(&target).unwrap().permissions();
    permissions.set_readonly(false);
    fs::set_permissions(&target, permissions).unwrap();
    fs::write(&target, b"{\"drifted\":true}").unwrap();

    let error = prepare_disabled_installation_publication(&fix.request, &fix.context(), &before)
        .unwrap_err();

    // Quarantine byte drift is caught while regenerating fresh authoritative
    // J24J authority, so the plan layer's own classification is preserved
    // rather than remapped into generic evidence staleness.
    assert_eq!(error.code, "candidate_invalid");
    assert!(fix.installed.load_all().unwrap().is_empty());
}

#[test]
fn j24k3e1_missing_candidate_is_refused() {
    let fix = Fixture::new();
    let before = fix.plan();
    for entry in fs::read_dir(fix.base.join("candidates")).unwrap() {
        let path = entry.unwrap().path();
        if path.is_file() {
            fs::remove_file(path).unwrap();
        }
    }

    let error = prepare_disabled_installation_publication(&fix.request, &fix.context(), &before)
        .unwrap_err();

    assert_eq!(error.code, "installation_plan_candidate_missing");
}

#[test]
fn j24k3e1_duplicate_installed_release_is_refused() {
    let fix = Fixture::new();
    let before = fix.plan();

    // Another installed record for the same package release, sourced from a
    // different candidate, so the ordinary plan still says publish.
    let mut duplicate = fix.installed_record();
    duplicate.source_candidate_id = Uuid::new_v4().to_string();
    let mut covered = duplicate.clone();
    covered.record_digest.clear();
    duplicate.record_digest = sha256(&canonical(&covered).unwrap());
    duplicate.validate().unwrap();
    fix.publish_installed(&duplicate);

    // The ordinary plan still reaches publication, because J24J matches
    // installed state by source candidate.
    assert_eq!(
        fix.plan().action,
        InstallationPlanAction::PublishDisabledInstallation
    );

    let error = prepare_disabled_installation_publication(&fix.request, &fix.context(), &before)
        .unwrap_err();

    assert_eq!(error.code, "installed_conflict");
    // Nothing new was published.
    assert_eq!(fix.installed.load_all().unwrap(), vec![duplicate]);
}

#[test]
fn j24k3e1_unsafe_intent_root_preserves_unsafe_store_path() {
    let fix = Fixture::new();
    let before = fix.plan();

    // Replace the intent root with a junction/symlink to another directory.
    let real = fix.base.join("elsewhere");
    fs::create_dir_all(&real).unwrap();
    let root = fix.intent_store.root_path().to_path_buf();
    fs::remove_dir_all(&root).unwrap();
    #[cfg(windows)]
    {
        let status = std::process::Command::new("cmd")
            .args([
                "/C",
                "mklink",
                "/J",
                root.to_str().unwrap(),
                real.to_str().unwrap(),
            ])
            .status()
            .unwrap();
        assert!(status.success());
    }
    #[cfg(unix)]
    std::os::unix::fs::symlink(&real, &root).unwrap();

    let error = prepare_disabled_installation_publication(&fix.request, &fix.context(), &before)
        .unwrap_err();

    assert_eq!(error.code, "unsafe_store_path");
}
