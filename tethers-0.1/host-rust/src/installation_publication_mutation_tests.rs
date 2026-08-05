use super::installation_publication_mutation::execute_prepared_disabled_installation_publication;
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
use crate::installation_publication_preparation::{
    prepare_disabled_installation_publication, PreparedInstallationPublication,
};
use crate::installation_recovery::InstallationRecoveryDisposition;
use crate::installation_recovery_evidence::InstallationRecoveryEvidenceContext;
use crate::installation_recovery_execution::execute_validated_installation_recovery;
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
    DisabledBindingRecord, InstallationApprovalRecord, InstallationApprovalStore,
    InstalledPlugRecord, InstalledPlugRegistry,
};
use crate::launch_profile::{
    LaunchProfileEvidence, LaunchProfileEvidenceStore, PreparedSupervisedLaunch,
};
use crate::m3_store::{canonical, sha256, Result};
use crate::package::{self, PayloadEvidence};
use crate::trust::PackageTrustEvidence;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
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
        host_build_identity: "j24k3e2-test".to_owned(),
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
        let base = std::env::temp_dir().join(format!("tethers-j24k3e2-{}", Uuid::new_v4()));
        fs::create_dir_all(&base).unwrap();
        let archive = base.join("test.tetherplug");
        fs::write(
            &archive,
            crate::pdf_tools::build_reference_package(b"j24k3e2-test").unwrap(),
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
            .create(&candidate, &request, "j24k3e2-authority")
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
                "j24k3e2-authority",
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

    fn prepared_intent(&self) -> InstallationPublicationIntent {
        let before = self.plan();
        let prepared =
            prepare_disabled_installation_publication(&self.request, &self.context(), &before)
                .unwrap();
        prepared.intent().clone()
    }

    fn prepared_record(&self) -> InstalledPlugRecord {
        let before = self.plan();
        let prepared =
            prepare_disabled_installation_publication(&self.request, &self.context(), &before)
                .unwrap();
        prepared.installed_record().clone()
    }

    fn prepare(&self) -> PreparedInstallationPublication {
        let before = self.plan();
        prepare_disabled_installation_publication(&self.request, &self.context(), &before).unwrap()
    }

    fn run(&self, prepared: PreparedInstallationPublication) -> Result<InstalledPlugRecord> {
        execute_prepared_disabled_installation_publication(&self.request, &self.context(), prepared)
    }

    fn staging_path(&self, intent: &InstallationPublicationIntent) -> PathBuf {
        self.install_root
            .join(format!(".staging-{}", intent.transaction_id))
    }

    fn destination_path(&self, intent: &InstallationPublicationIntent) -> PathBuf {
        self.install_root.join(&intent.destination_relative_path)
    }

    fn record_path(&self, id: &str) -> PathBuf {
        self.record_root.join(format!("{id}.json"))
    }

    fn published_record(&self, id: &str) -> InstalledPlugRecord {
        let bytes = fs::read(self.record_path(id)).unwrap();
        let record: InstalledPlugRecord = crate::m3_store::strict_json(&bytes).unwrap();
        record
    }

    /// Build the exact staging directory for an intent using the accepted seam.
    fn build_staging(&self, intent: &InstallationPublicationIntent) {
        self.installed
            .build_installation_recovery_staging(intent, &self.candidate, &self.quarantine_root)
            .unwrap();
    }

    /// Rename staging to the final destination using the accepted seam.
    fn rename_staging(&self, intent: &InstallationPublicationIntent) {
        self.installed
            .rename_installation_recovery_staging(intent)
            .unwrap();
    }

    fn write_record(&self, record: &InstalledPlugRecord) {
        fs::write(
            self.record_path(&record.installed_id),
            canonical(record).unwrap(),
        )
        .unwrap();
    }

    /// Publish a complete, store-consistent installed state: the read-only
    /// destination plus its owning record.
    fn publish_installed(&self, record: &InstalledPlugRecord) {
        let destination = self.install_root.join(&record.installation_relative_path);
        fs::create_dir(&destination).unwrap();
        let source = self
            .quarantine_root
            .join(&self.candidate.quarantine_relative_path);
        for evidence in std::iter::once(&record.plug_json)
            .chain(record.payloads.iter())
            .chain(record.signature_files.iter())
        {
            let bytes = fs::read(source.join(&evidence.path)).unwrap();
            let target = destination.join(&evidence.path);
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(&target, bytes).unwrap();
            let mut permissions = fs::metadata(&target).unwrap().permissions();
            permissions.set_readonly(true);
            fs::set_permissions(&target, permissions).unwrap();
        }
        self.write_record(record);
    }

    fn redigest(mut record: InstalledPlugRecord) -> InstalledPlugRecord {
        let mut covered = record.clone();
        covered.record_digest.clear();
        record.record_digest = sha256(&canonical(&covered).unwrap());
        record
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
// 1. Valid prepared publication completes exactly once
// ---------------------------------------------------------------------------

#[test]
fn j24k3e2_valid_prepared_publication_completes_exactly_once() {
    let fix = Fixture::new();
    let prepared = fix.prepare();
    let intent = prepared.intent().clone();

    let returned = fix.run(prepared).unwrap();

    // One destination, one record, no intent, no staging.
    assert!(fix.destination_path(&intent).is_dir());
    assert!(fix.record_path(&intent.transaction_id).is_file());
    assert!(fix.intent_store.load().unwrap().is_none());
    assert!(!fix.staging_path(&intent).exists());
    assert_eq!(returned.installed_id, intent.transaction_id);
}

#[test]
fn j24k3e2_returned_and_published_record_equal_prepared() {
    let fix = Fixture::new();
    let prepared = fix.prepare();
    let intent = prepared.intent().clone();
    let prepared_record = prepared.installed_record().clone();

    let returned = fix.run(prepared).unwrap();

    assert_eq!(returned, prepared_record);
    let from_disk = fix.published_record(&intent.transaction_id);
    assert_eq!(from_disk, prepared_record);
    // Internal identity never regenerated.
    assert_eq!(from_disk.installed_id, intent.transaction_id);
    assert_eq!(from_disk.record_digest, intent.installed_record_digest);
}

// ---------------------------------------------------------------------------
// 3. Persisted intent equals the prepared intent before later steps
// ---------------------------------------------------------------------------

#[test]
fn j24k3e2_persisted_intent_equals_prepared_before_later_steps() {
    let fix = Fixture::new();
    let intent = fix.prepared_intent();

    // Step 3 boundary: the exact intent must persist and reload equal.
    fix.intent_store.create(&intent).unwrap();
    assert_eq!(fix.intent_store.load().unwrap().as_ref(), Some(&intent));
}

// ---------------------------------------------------------------------------
// 4. Stale evidence before mutation creates no intent
// ---------------------------------------------------------------------------

#[test]
fn j24k3e2_stale_evidence_before_mutation_creates_no_intent() {
    let fix = Fixture::new();
    let prepared = fix.prepare();
    let intent = prepared.intent().clone();
    for entry in fs::read_dir(fix.base.join("trust")).unwrap() {
        let path = entry.unwrap().path();
        if path.is_file() {
            fs::remove_file(path).unwrap();
        }
    }

    // Fresh revalidation occurs inside the mutation before the first durable
    // write, so stale evidence must create no intent.
    let error =
        execute_prepared_disabled_installation_publication(&fix.request, &fix.context(), prepared)
            .unwrap_err();

    assert_eq!(error.code, "installation_intent_evidence_stale");
    assert!(fix.intent_store.load().unwrap().is_none());
    assert!(!fix.destination_path(&intent).exists());
}

// ---------------------------------------------------------------------------
// 5. Non-idle recovery creates no new state
// ---------------------------------------------------------------------------

#[test]
fn j24k3e2_non_idle_recovery_creates_no_new_state() {
    let fix = Fixture::new();
    let intent = fix.prepared_intent();
    // A pending intent already exists: recovery is not idle.
    fix.intent_store.create(&intent).unwrap();
    let snapshot = tree_snapshot(&fix.base);

    // Preparation (and therefore mutation) is refused while recovery is
    // non-idle; no new durable state is created.
    let before = fix.plan();
    let error = prepare_disabled_installation_publication(&fix.request, &fix.context(), &before)
        .unwrap_err();

    assert_eq!(error.code, "installation_recovery_conflict");
    // No staging, destination, record, or additional intent; existing retained.
    assert!(!fix.staging_path(&intent).exists());
    assert!(!fix.destination_path(&intent).exists());
    assert!(!fix.record_path(&intent.transaction_id).exists());
    assert_eq!(fix.intent_store.load().unwrap().as_ref(), Some(&intent));
    assert_eq!(snapshot, tree_snapshot(&fix.base));
}

// ---------------------------------------------------------------------------
// 6. Candidate or quarantine drift is refused
// ---------------------------------------------------------------------------

#[test]
fn j24k3e2_quarantine_byte_drift_is_refused_before_intent() {
    let fix = Fixture::new();
    let prepared = fix.prepare();
    let intent = prepared.intent().clone();
    let target = fix
        .quarantine_root
        .join(&fix.candidate.quarantine_relative_path)
        .join("plug.json");
    let mut permissions = fs::metadata(&target).unwrap().permissions();
    permissions.set_readonly(false);
    fs::set_permissions(&target, permissions).unwrap();
    fs::write(&target, b"{\"drifted\":true}").unwrap();

    let error =
        execute_prepared_disabled_installation_publication(&fix.request, &fix.context(), prepared)
            .unwrap_err();

    assert_eq!(error.code, "installation_intent_evidence_stale");
    assert!(fix.intent_store.load().unwrap().is_none());
    assert!(!fix.destination_path(&intent).exists());
}

// ---------------------------------------------------------------------------
// 7. Exact staging files, lengths, hashes and read-only permissions verified
// ---------------------------------------------------------------------------

#[test]
fn j24k3e2_exact_staging_files_lengths_hashes_and_read_only() {
    let fix = Fixture::new();
    let intent = fix.prepared_intent();
    fix.build_staging(&intent);

    let staging = fix.staging_path(&intent);
    let expected: BTreeMap<String, PayloadEvidence> = std::iter::once(&fix.candidate.plug_json)
        .chain(fix.candidate.payloads.iter())
        .chain(fix.candidate.signature_files.iter())
        .map(|e| (e.path.clone(), e.clone()))
        .collect();
    let mut actual = BTreeSet::new();
    collect_files(&staging, &staging, &mut actual);

    assert_eq!(actual, expected.keys().cloned().collect::<BTreeSet<_>>());
    for (relative, evidence) in &expected {
        let file = staging.join(relative);
        let metadata = fs::metadata(&file).unwrap();
        assert!(
            metadata.permissions().readonly(),
            "{relative} not read-only"
        );
        let bytes = fs::read(&file).unwrap();
        assert_eq!(bytes.len() as u64, evidence.size_bytes);
        assert_eq!(sha256(&bytes), evidence.sha256);
    }
    // The installed record is never written into the payload directory.
    assert!(!staging
        .join(format!("{}.json", intent.transaction_id))
        .exists());
}

fn collect_files(root: &Path, directory: &Path, out: &mut BTreeSet<String>) {
    for entry in fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            collect_files(root, &path, out);
        } else {
            out.insert(
                path.strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/"),
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 8/21. Unsafe or reparse staging paths fail closed preserving unsafe_store_path
// ---------------------------------------------------------------------------

#[cfg(windows)]
#[test]
fn j24k3e2_reparse_staging_path_fails_closed_with_unsafe_store_path() {
    let fix = Fixture::new();
    let prepared = fix.prepare();
    let intent = prepared.intent().clone();

    // Replace the staging location with a junction before mutation.
    let staging = fix.staging_path(&intent);
    let real = fix.base.join("elsewhere");
    fs::create_dir_all(&real).unwrap();
    let status = std::process::Command::new("cmd")
        .args([
            "/C",
            "mklink",
            "/J",
            staging.to_str().unwrap(),
            real.to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(status.success());

    let error = fix.run(prepared).unwrap_err();

    assert_eq!(error.code, "unsafe_store_path");
    // The intent was already created; recovery must still classify it idle-safe.
    assert!(fix.intent_store.load().unwrap().is_some());
    assert!(!fix.destination_path(&intent).exists());
}

// ---------------------------------------------------------------------------
// 9. Existing staging or destination is not overwritten or adopted
// ---------------------------------------------------------------------------

#[test]
fn j24k3e2_existing_staging_is_not_overwritten_or_adopted() {
    let fix = Fixture::new();
    let prepared = fix.prepare();
    let intent = prepared.intent().clone();
    fs::create_dir(fix.staging_path(&intent)).unwrap();
    fs::write(fix.staging_path(&intent).join("intruder.txt"), b"x").unwrap();

    // Staging construction refuses to create over the existing directory; the
    // intent was already persisted and must remain so recovery can resume.
    let error = fix.run(prepared).unwrap_err();

    assert_eq!(error.code, "installation_recovery_io");
    assert!(fix.staging_path(&intent).join("intruder.txt").exists());
    assert!(!fix.destination_path(&intent).exists());
    assert!(fix.intent_store.load().unwrap().is_some());
}

#[test]
fn j24k3e2_existing_destination_is_not_overwritten_or_adopted() {
    let fix = Fixture::new();
    let prepared = fix.prepare();
    let intent = prepared.intent().clone();
    fs::create_dir(fix.destination_path(&intent)).unwrap();
    fs::write(fix.destination_path(&intent).join("intruder.txt"), b"x").unwrap();

    // The global installed-root audit rejects the untracked final directory
    // before any durable write; it is never adopted or overwritten.
    let error = fix.run(prepared).unwrap_err();

    assert_eq!(error.code, "installation_destination_untracked");
    assert!(fix.destination_path(&intent).join("intruder.txt").exists());
    assert!(fix.intent_store.load().unwrap().is_none());
    assert!(!fix.record_path(&intent.transaction_id).exists());
}

// ---------------------------------------------------------------------------
// 10. Rename uses the exact prepared destination
// ---------------------------------------------------------------------------

#[test]
fn j24k3e2_rename_uses_exact_prepared_destination() {
    let fix = Fixture::new();
    let intent = fix.prepared_intent();
    fix.build_staging(&intent);
    fix.rename_staging(&intent);

    let destination = fix.destination_path(&intent);
    assert!(destination.is_dir());
    assert_eq!(
        destination.file_name().unwrap().to_string_lossy(),
        intent.destination_relative_path
    );
    assert_eq!(
        intent.destination_relative_path,
        format!("plug-{}", intent.transaction_id)
    );
    assert!(!fix.staging_path(&intent).exists());
}

// ---------------------------------------------------------------------------
// 11. Final destination is reverified before record publication
// ---------------------------------------------------------------------------

#[test]
fn j24k3e2_final_destination_reverified_before_publication() {
    let fix = Fixture::new();
    let intent = fix.prepared_intent();
    fix.build_staging(&intent);
    fix.rename_staging(&intent);

    // Corrupt one destination payload after rename; publication must refuse.
    let target = fix
        .destination_path(&intent)
        .join(&fix.candidate.plug_json.path);
    let mut permissions = fs::metadata(&target).unwrap().permissions();
    permissions.set_readonly(false);
    fs::set_permissions(&target, permissions).unwrap();
    fs::write(&target, b"{\"corrupted\":true}").unwrap();

    let error = fix
        .installed
        .publish_installation_recovery_record(&intent)
        .unwrap_err();
    assert_eq!(error.code, "installation_recovery_conflict");
    assert!(!fix.record_path(&intent.transaction_id).exists());
}

// ---------------------------------------------------------------------------
// 12. Exact record publication refuses mismatched identity, digest, duplicate
// ---------------------------------------------------------------------------

#[test]
fn j24k3e2_publication_refuses_mismatched_record_digest() {
    let fix = Fixture::new();
    let mut intent = fix.prepared_intent();
    // Tamper with the embedded record digest; the embedded record no longer
    // validates against its own digest, so publication must refuse it.
    intent.installed_record.record_digest = "sha256:".to_owned() + &"f".repeat(64);

    let error = fix
        .installed
        .publish_installation_recovery_record(&intent)
        .unwrap_err();
    assert_eq!(error.code, "installation_intent_invalid");
    assert!(!fix.record_path(&intent.transaction_id).exists());
}

#[test]
fn j24k3e2_publication_refuses_duplicate_installed_release() {
    let fix = Fixture::new();
    let prepared = fix.prepare();
    let intent = prepared.intent().clone();

    // After preparation, a contradictory same-release record appears, sourced
    // from a different candidate, so the ordinary plan still says publish.
    let mut duplicate = fix.prepared_record();
    duplicate.source_candidate_id = Uuid::new_v4().to_string();
    let duplicate = Fixture::redigest(duplicate);
    duplicate.validate().unwrap();
    fix.publish_installed(&duplicate);

    // The mutation freshly revalidates current installed state and refuses the
    // duplicate release before any durable write.
    let error = fix.run(prepared).unwrap_err();
    assert_eq!(error.code, "installed_conflict");
    assert!(!fix.destination_path(&intent).exists());
    assert!(fix.intent_store.load().unwrap().is_none());
    assert_eq!(fix.installed.load_all().unwrap().len(), 1);
}

// ---------------------------------------------------------------------------
// 13. Publication retains original UUID and timestamp
// ---------------------------------------------------------------------------

#[test]
fn j24k3e2_publication_retains_original_uuid_and_timestamp() {
    let fix = Fixture::new();
    let prepared = fix.prepare();
    let intent = prepared.intent().clone();
    let prepared_record = prepared.installed_record().clone();

    let returned = fix.run(prepared).unwrap();

    assert_eq!(returned.installed_id, prepared_record.installed_id);
    assert_eq!(returned.created_unix_ms, prepared_record.created_unix_ms);
    let from_disk = fix.published_record(&intent.transaction_id);
    assert_eq!(from_disk.installed_id, prepared_record.installed_id);
    assert_eq!(from_disk.created_unix_ms, prepared_record.created_unix_ms);
}

// ---------------------------------------------------------------------------
// 14. Completed publication is freshly recovery-planned before intent removal
// ---------------------------------------------------------------------------

#[test]
fn j24k3e2_completed_publication_classifies_completed_then_idle() {
    let fix = Fixture::new();
    let intent = fix.prepared_intent();
    fix.intent_store.create(&intent).unwrap();
    fix.build_staging(&intent);
    fix.rename_staging(&intent);
    fix.installed
        .publish_installation_recovery_record(&intent)
        .unwrap();

    // Fresh plan must classify the completed publication exactly.
    let plan = plan_installation_recovery(&fix.request, &fix.context()).unwrap();
    assert_eq!(
        plan.disposition(),
        Some(InstallationRecoveryDisposition::VerifyCompletedPublicationThenRemoveIntent)
    );

    // Removing the completed intent returns to idle.
    let outcome =
        execute_validated_installation_recovery(&fix.request, &fix.context(), plan).unwrap();
    assert_eq!(
        outcome,
        crate::installation_recovery_execution::InstallationRecoveryExecutionOutcome::Recovered {
            disposition:
                crate::installation_recovery::InstallationRecoveryDisposition::
                    VerifyCompletedPublicationThenRemoveIntent
        }
    );
    let final_plan = plan_installation_recovery(&fix.request, &fix.context()).unwrap();
    assert!(final_plan.is_idle());
}

// ---------------------------------------------------------------------------
// 15. Success leaves no intent, no staging, and idle recovery
// ---------------------------------------------------------------------------

#[test]
fn j24k3e2_success_leaves_idle_recovery_no_intent_no_staging() {
    let fix = Fixture::new();
    let prepared = fix.prepare();
    let intent = prepared.intent().clone();

    fix.run(prepared).unwrap();

    assert!(fix.intent_store.load().unwrap().is_none());
    assert!(!fix.staging_path(&intent).exists());
    let final_plan = plan_installation_recovery(&fix.request, &fix.context()).unwrap();
    assert!(final_plan.is_idle());
    assert!(final_plan.disposition().is_none());
}

// ---------------------------------------------------------------------------
// 16. Intent-only and intent-plus-staging prefixes are recoverable
// ---------------------------------------------------------------------------

#[test]
fn j24k3e2_intent_only_prefix_is_recoverable_to_idle() {
    let fix = Fixture::new();
    let intent = fix.prepared_intent();
    fix.intent_store.create(&intent).unwrap();

    let plan = plan_installation_recovery(&fix.request, &fix.context()).unwrap();
    let outcome =
        execute_validated_installation_recovery(&fix.request, &fix.context(), plan).unwrap();
    assert_eq!(
        outcome,
        crate::installation_recovery_execution::InstallationRecoveryExecutionOutcome::Recovered {
            disposition:
                crate::installation_recovery::InstallationRecoveryDisposition::RemoveIntentOnly
        }
    );
    assert!(fix.intent_store.load().unwrap().is_none());
}

#[test]
fn j24k3e2_intent_plus_staging_prefix_is_recoverable_to_idle() {
    let fix = Fixture::new();
    let intent = fix.prepared_intent();
    fix.intent_store.create(&intent).unwrap();
    fix.build_staging(&intent);

    let plan = plan_installation_recovery(&fix.request, &fix.context()).unwrap();
    let outcome =
        execute_validated_installation_recovery(&fix.request, &fix.context(), plan).unwrap();
    assert_eq!(
        outcome,
        crate::installation_recovery_execution::InstallationRecoveryExecutionOutcome::Recovered {
            disposition:
                crate::installation_recovery::InstallationRecoveryDisposition::RemoveStagingThenIntent
        }
    );
    assert!(fix.intent_store.load().unwrap().is_none());
    assert!(!fix.staging_path(&intent).exists());
}

// ---------------------------------------------------------------------------
// 17. Destination-without-record publishes the exact record once
// ---------------------------------------------------------------------------

#[test]
fn j24k3e2_destination_without_record_publishes_exact_record_once() {
    let fix = Fixture::new();
    let intent = fix.prepared_intent();
    fix.intent_store.create(&intent).unwrap();
    fix.build_staging(&intent);
    fix.rename_staging(&intent);

    let plan = plan_installation_recovery(&fix.request, &fix.context()).unwrap();
    assert_eq!(
        plan.disposition(),
        Some(
            crate::installation_recovery::InstallationRecoveryDisposition::
                RevalidateDestinationThenPublishRecord
        )
    );
    execute_validated_installation_recovery(&fix.request, &fix.context(), plan).unwrap();

    let published = fix.published_record(&intent.transaction_id);
    assert_eq!(published, intent.installed_record);
    let final_plan = plan_installation_recovery(&fix.request, &fix.context()).unwrap();
    assert!(final_plan.is_idle());
}

// ---------------------------------------------------------------------------
// 18. Destination-plus-matching-record removes only completed intent
// ---------------------------------------------------------------------------

#[test]
fn j24k3e2_destination_plus_matching_record_removes_only_intent() {
    let fix = Fixture::new();
    let intent = fix.prepared_intent();
    fix.intent_store.create(&intent).unwrap();
    fix.build_staging(&intent);
    fix.rename_staging(&intent);
    fix.write_record(&intent.installed_record);

    let plan = plan_installation_recovery(&fix.request, &fix.context()).unwrap();
    assert_eq!(
        plan.disposition(),
        Some(
            crate::installation_recovery::InstallationRecoveryDisposition::
                VerifyCompletedPublicationThenRemoveIntent
        )
    );
    execute_validated_installation_recovery(&fix.request, &fix.context(), plan).unwrap();

    assert!(fix.intent_store.load().unwrap().is_none());
    assert!(fix.destination_path(&intent).is_dir());
    assert!(fix.record_path(&intent.transaction_id).is_file());
}

// ---------------------------------------------------------------------------
// 19. Record-without-destination, mismatched destination, mismatched record fail
// ---------------------------------------------------------------------------

#[test]
fn j24k3e2_record_without_destination_fails_closed() {
    let fix = Fixture::new();
    let intent = fix.prepared_intent();
    fix.intent_store.create(&intent).unwrap();
    fix.write_record(&intent.installed_record);

    // A record without its destination is contradictory installed state; fresh
    // planning fails closed rather than cleaning or adopting it.
    let error = plan_installation_recovery(&fix.request, &fix.context()).unwrap_err();
    assert_eq!(error.code, "installation_recovery_conflict");
}

#[test]
fn j24k3e2_mismatched_destination_fails_closed() {
    let fix = Fixture::new();
    let intent = fix.prepared_intent();
    fix.intent_store.create(&intent).unwrap();
    // An untracked final directory that is not the intent's destination.
    let wrong = fix.install_root.join(format!("plug-{}", Uuid::new_v4()));
    fs::create_dir(&wrong).unwrap();

    let error = plan_installation_recovery(&fix.request, &fix.context()).unwrap_err();
    assert!(
        error.code == "installation_destination_untracked"
            || error.code == "installation_recovery_conflict"
    );
}

#[test]
fn j24k3e2_mismatched_record_fails_closed() {
    let fix = Fixture::new();
    let intent = fix.prepared_intent();
    fix.intent_store.create(&intent).unwrap();
    fix.build_staging(&intent);
    fix.rename_staging(&intent);
    // A record present but not equal to the intent's prepared record.
    let mut wrong = intent.installed_record.clone();
    wrong.created_unix_ms = wrong.created_unix_ms.wrapping_add(1);
    let wrong = Fixture::redigest(wrong);
    fix.write_record(&wrong);

    // Classification fails closed during fresh planning.
    let error = plan_installation_recovery(&fix.request, &fix.context()).unwrap_err();
    assert_eq!(error.code, "installation_recovery_conflict");
}

// ---------------------------------------------------------------------------
// 20. Staging-cleanup failure retains intent
// ---------------------------------------------------------------------------

#[cfg(windows)]
#[test]
fn j24k3e2_staging_cleanup_failure_retains_intent() {
    let fix = Fixture::new();
    let intent = fix.prepared_intent();
    fix.intent_store.create(&intent).unwrap();
    fix.build_staging(&intent);

    // Open a staging file with an exclusive (delete-denying) handle so the
    // accepted cleanup route cannot delete the staging directory.
    let locked = open_staging_file_exclusive(&fix.staging_path(&intent).join("plug.json"));

    let plan = plan_installation_recovery(&fix.request, &fix.context()).unwrap();
    assert_eq!(
        plan.disposition(),
        Some(
            crate::installation_recovery::InstallationRecoveryDisposition::RemoveStagingThenIntent
        )
    );
    let error =
        execute_validated_installation_recovery(&fix.request, &fix.context(), plan).unwrap_err();
    drop(locked);

    // The intent must be retained after the staging-cleanup failure.
    assert!(fix.staging_path(&intent).exists());
    assert_eq!(fix.intent_store.load().unwrap().as_ref(), Some(&intent));
    assert!(!fix.destination_path(&intent).exists());
    assert!(!fix.record_path(&intent.transaction_id).exists());
}

#[cfg(windows)]
fn open_staging_file_exclusive(path: &Path) -> std::fs::File {
    use std::os::windows::ffi::OsStrExt;
    use std::os::windows::io::FromRawHandle;
    use windows_sys::Win32::Foundation::{GENERIC_READ, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::Storage::FileSystem::{CreateFileW, OPEN_EXISTING};

    let path_w: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    // SAFETY: path_w is a live nul-terminated buffer through the call; the
    // handle is opened with read+write sharing but no delete sharing, so a
    // concurrent deletion attempt fails with a sharing violation.
    let raw = unsafe {
        CreateFileW(
            path_w.as_ptr(),
            GENERIC_READ,
            windows_sys::Win32::Storage::FileSystem::FILE_SHARE_READ
                | windows_sys::Win32::Storage::FileSystem::FILE_SHARE_WRITE,
            std::ptr::null(),
            OPEN_EXISTING,
            0,
            std::ptr::null_mut(),
        )
    };
    assert!(raw != INVALID_HANDLE_VALUE);
    // SAFETY: raw is a valid owned file handle; ownership is transferred to the File.
    unsafe { std::fs::File::from_raw_handle(raw as _) }
}

// ---------------------------------------------------------------------------
// 21. Path-safety failures preserve unsafe_store_path (covered by reparse test)
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// 22. The same prepared transaction cannot duplicate publication
// ---------------------------------------------------------------------------

#[test]
fn j24k3e2_same_transaction_cannot_duplicate_publication() {
    let fix = Fixture::new();
    let prepared = fix.prepare();
    let intent = prepared.intent().clone();

    // First full transaction completes and removes the intent.
    fix.run(prepared).unwrap();
    assert!(fix.intent_store.load().unwrap().is_none());

    // Replay the same transaction: re-introduce the exact same intent. Because
    // the destination and matching record already exist, the accepted recovery
    // authority classifies this as completed publication and removes only the
    // intent. It must never publish a second record for this transaction.
    fix.intent_store.create(&intent).unwrap();
    let plan = plan_installation_recovery(&fix.request, &fix.context()).unwrap();
    assert_eq!(
        plan.disposition(),
        Some(
            crate::installation_recovery::InstallationRecoveryDisposition::
                VerifyCompletedPublicationThenRemoveIntent
        )
    );
    execute_validated_installation_recovery(&fix.request, &fix.context(), plan).unwrap();

    // Exactly one record remains; no duplicate publication occurred.
    assert!(fix.intent_store.load().unwrap().is_none());
    let records = fix.installed.load_all().unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].installed_id, intent.transaction_id);
    assert_eq!(records[0], intent.installed_record);
}
