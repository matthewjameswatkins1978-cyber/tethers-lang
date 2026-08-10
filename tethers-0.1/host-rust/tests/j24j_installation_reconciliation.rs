#![cfg(windows)]

use serde_json_canonicalizer;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use tethers_reference_host::candidate::{
    extract_to_quarantine, CandidateRecord, CandidateRegistry,
};
use tethers_reference_host::conformance::{
    current_suite_digest, CaseDisposition, ConformanceCaseEvidence, ConformanceDisposition,
    ConformanceEvidence, ConformanceEvidenceStore,
};
use tethers_reference_host::installation_plan::{plan_installation, InstallationPlanAction};
use tethers_reference_host::installation_request::{
    InstallationConformanceRequest, InstallationRequest, InstallationTargetRequest,
    InstallationTargetState, InstallationTrustRequest, InstallationTrustScope,
    INSTALLATION_REQUEST_SCHEMA,
};
use tethers_reference_host::installation_trust::ExactCandidateTrustStore;
use tethers_reference_host::installed::{
    DisabledBindingRecord, InstallationApprovalRecord, InstallationApprovalStore,
    InstalledPlugRecord, InstalledPlugRegistry, ReviewedCapability,
};
use tethers_reference_host::launch_profile::{
    LaunchProfileEvidence, LaunchProfileEvidenceStore, SUPERVISED_PROFILE_LABEL,
    SUPERVISED_PROFILE_LIMITATION,
};
use tethers_reference_host::package;
use tethers_reference_host::test_fixture_package;
use tethers_reference_host::trust::PackageTrustEvidence;
use uuid::Uuid;

fn sha256(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn temp_dir(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("tethers-j24j-{name}-{}", Uuid::new_v4()))
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
                output.insert(format!("{relative}/"), "<directory>".into());
                visit(root, &entry, output);
            } else if metadata.is_file() {
                output.insert(relative, sha256(&fs::read(&entry).unwrap()));
            }
        }
    }

    let mut output = BTreeMap::new();
    visit(root, root, &mut output);
    output
}

fn valid_request(candidate_id: &str) -> InstallationRequest {
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

fn build_launch_profile_evidence(candidate: &CandidateRecord) -> LaunchProfileEvidence {
    let executable = candidate
        .payloads
        .iter()
        .find(|p| p.path == candidate.launch_path)
        .expect("executable payload");
    let mut evidence = LaunchProfileEvidence {
        profile_format_version: 1,
        profile_label: SUPERVISED_PROFILE_LABEL.into(),
        isolated: false,
        limitation: SUPERVISED_PROFILE_LIMITATION.into(),
        candidate_id: candidate.candidate_id.clone(),
        semantic_package_digest: candidate.semantic_package_digest.clone(),
        executable_digest: executable.sha256.clone(),
        executable_relative_path: candidate.launch_path.clone(),
        arguments: candidate.launch_arguments.clone(),
        working_directory_relative_path: candidate.provider_working_directory.clone(),
        environment_names: vec!["SystemRoot".into(), "TEMP".into()],
        environment_digest: format!("sha256:{}", "c".repeat(64)),
        max_processes: 8,
        process_memory_limit_bytes: 256 * 1024 * 1024,
        protocol_line_limit_bytes: 1024 * 1024,
        stderr_tail_limit_bytes: 16 * 1024,
        wall_time_limit_ms: 30000,
        profile_evidence_digest: String::new(),
    };
    let covered = {
        let mut copy = evidence.clone();
        copy.profile_evidence_digest.clear();
        serde_json_canonicalizer::to_vec(&copy).unwrap()
    };
    evidence.profile_evidence_digest = sha256(&covered);
    evidence.validate().unwrap();
    evidence
}

fn build_passing_conformance(
    candidate: &CandidateRecord,
    trust: &PackageTrustEvidence,
    launch: &LaunchProfileEvidence,
    suite_digest: &str,
    ended_unix_ms: u64,
    evidence_id: &str,
) -> ConformanceEvidence {
    let mut evidence = ConformanceEvidence {
        schema_version: 1,
        evidence_id: evidence_id.into(),
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
        ended_unix_ms,
        cases: all_passed_cases(),
        disposition: ConformanceDisposition::Passed,
        retry_count: 0,
        raw_stderr_persisted: false,
        evidence_digest: String::new(),
    };
    let covered = {
        let mut copy = evidence.clone();
        copy.evidence_digest.clear();
        serde_json_canonicalizer::to_vec(&copy).unwrap()
    };
    evidence.evidence_digest = sha256(&covered);
    evidence.validate().unwrap();
    evidence
}

fn build_approval_record(
    candidate: &CandidateRecord,
    trust: &PackageTrustEvidence,
    launch: &LaunchProfileEvidence,
    conformance: &ConformanceEvidence,
    approval_id: &str,
    approving_authority: &str,
    approved_unix_ms: u64,
) -> InstallationApprovalRecord {
    let reviewed_capabilities: Vec<ReviewedCapability> = candidate
        .capabilities
        .iter()
        .map(|cap| ReviewedCapability {
            capability_name: cap.name.clone(),
            capability_version: cap.version,
            manifest_digest: cap.manifest_digest.clone(),
            provider_operation_name: cap.operation.clone(),
            effects: vec![],
            permission_scope: serde_json::Value::Object(serde_json::Map::new()),
            permission_scope_digest: format!("sha256:{}", "e".repeat(64)),
        })
        .collect();
    let mut record = InstallationApprovalRecord {
        schema_version: 1,
        approval_id: approval_id.into(),
        candidate_id: candidate.candidate_id.clone(),
        package_id: candidate.package_id.clone(),
        package_version: candidate.package_version.clone(),
        semantic_package_digest: candidate.semantic_package_digest.clone(),
        raw_archive_digest: candidate.raw_archive_digest.clone(),
        source_size_bytes: candidate.source_size_bytes,
        payloads: candidate.payloads.clone(),
        reviewed_capabilities,
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
        approving_authority: approving_authority.into(),
        approved_unix_ms,
        record_digest: String::new(),
    };
    let covered = {
        let mut copy = record.clone();
        copy.record_digest.clear();
        serde_json_canonicalizer::to_vec(&copy).unwrap()
    };
    record.record_digest = sha256(&covered);
    record.validate().unwrap();
    record
}

fn write_approval_json(store_dir: &Path, record: &InstallationApprovalRecord) {
    let canonical_bytes = serde_json_canonicalizer::to_vec(record).unwrap();
    let path = store_dir.join(format!("{}.json", record.approval_id));
    fs::write(&path, &canonical_bytes).unwrap();
}

fn copy_files_from_quarantine(
    quarantine_root: &Path,
    candidate: &CandidateRecord,
    destination: &Path,
) {
    let quarantine = quarantine_root.join(&candidate.quarantine_relative_path);
    let files: Vec<&tethers_reference_host::package::PayloadEvidence> =
        std::iter::once(&candidate.plug_json)
            .chain(candidate.payloads.iter())
            .chain(candidate.signature_files.iter())
            .collect();

    for payload in files {
        let source = quarantine.join(&payload.path);
        let dest = destination.join(&payload.path);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::copy(&source, &dest).unwrap();
        let mut perms = fs::metadata(&dest).unwrap().permissions();
        perms.set_readonly(true);
        fs::set_permissions(&dest, perms).unwrap();
    }
}

fn build_installed_record(
    candidate: &CandidateRecord,
    trust: &PackageTrustEvidence,
    launch: &LaunchProfileEvidence,
    conformance: &ConformanceEvidence,
    approval: &InstallationApprovalRecord,
    installed_id: &str,
    created_unix_ms: u64,
    installation_relative_path: &str,
) -> InstalledPlugRecord {
    let disabled_bindings: Vec<DisabledBindingRecord> = candidate
        .capabilities
        .iter()
        .map(|cap| DisabledBindingRecord {
            state: "disabled".into(),
            capability_name: cap.name.clone(),
            capability_version: cap.version,
            manifest_digest: cap.manifest_digest.clone(),
            provider_operation_name: cap.operation.clone(),
        })
        .collect();
    let mut record = InstalledPlugRecord {
        schema_version: 1,
        installed_id: installed_id.into(),
        state: "present_disabled".into(),
        package_id: candidate.package_id.clone(),
        package_version: candidate.package_version.clone(),
        semantic_package_digest: candidate.semantic_package_digest.clone(),
        source_candidate_id: candidate.candidate_id.clone(),
        installation_relative_path: installation_relative_path.into(),
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
        platform: "windows".into(),
        architecture: "x86_64".into(),
        disabled_bindings,
        operational_scope_schema: None,
        operational_scope_schema_digest: None,
        created_unix_ms,
        record_digest: String::new(),
    };
    let covered = {
        let mut copy = record.clone();
        copy.record_digest.clear();
        serde_json_canonicalizer::to_vec(&copy).unwrap()
    };
    record.record_digest = sha256(&covered);
    record.validate().unwrap();
    record
}

fn write_installed_json(record_root: &Path, record: &InstalledPlugRecord) {
    let canonical_bytes = serde_json_canonicalizer::to_vec(record).unwrap();
    let path = record_root.join(format!("{}.json", record.installed_id));
    fs::write(&path, &canonical_bytes).unwrap();
}

fn setup_candidate(base: &Path) -> (CandidateRegistry, CandidateRecord, PathBuf) {
    let quarantine_root = base.join("quarantine");
    let archive = base.join("fixture.tetherplug");
    let provider_bytes =
        fs::read(env!("CARGO_BIN_EXE_m3_fixture_provider")).expect("compiled provider");
    fs::write(
        &archive,
        test_fixture_package::build_fixture_package(&provider_bytes).unwrap(),
    )
    .unwrap();
    let report = package::inspect(&archive).unwrap();
    let quarantined = extract_to_quarantine(&report, &quarantine_root).unwrap();
    let candidates = CandidateRegistry::open(&base.join("candidates"), &quarantine_root).unwrap();
    let candidate = candidates.create(&quarantined).unwrap();
    (candidates, candidate, quarantine_root)
}

// --- Stage 1: No trust ---

#[test]
fn no_trust_returns_create_exact_candidate_trust() {
    let base = temp_dir("no-trust");
    fs::create_dir_all(&base).unwrap();

    let (candidates, candidate, _quarantine) = setup_candidate(&base);

    let exact_trust = ExactCandidateTrustStore::open(&base.join("trust")).unwrap();
    let profiles = LaunchProfileEvidenceStore::open(&base.join("profiles")).unwrap();
    let conformance_store = ConformanceEvidenceStore::open(&base.join("conformance")).unwrap();
    let approvals = InstallationApprovalStore::open(&base.join("approvals")).unwrap();
    let installed =
        InstalledPlugRegistry::open(&base.join("install"), &base.join("records")).unwrap();

    let request = valid_request(&candidate.candidate_id);
    let before = snapshot(&base);
    let plan = plan_installation(
        &request,
        &candidates,
        &exact_trust,
        &profiles,
        &conformance_store,
        &approvals,
        &installed,
    )
    .unwrap();

    assert_eq!(
        plan.action,
        InstallationPlanAction::CreateExactCandidateTrust
    );
    assert_eq!(plan.candidate_id, candidate.candidate_id);
    assert_eq!(plan.package_id, candidate.package_id);
    assert!(plan.exact_candidate_trust_record_digest.is_none());
    assert!(plan.trust_evidence_digest.is_none());
    assert!(plan.launch_profile_evidence_digest.is_none());
    assert!(plan.conformance_evidence_id.is_none());
    assert!(plan.conformance_evidence_digest.is_none());
    assert!(plan.installation_approval_id.is_none());
    assert!(plan.installation_approval_digest.is_none());
    assert!(plan.installed_id.is_none());
    assert!(plan.installed_record_digest.is_none());
    assert_eq!(before, snapshot(&base));

    fs::remove_dir_all(base).unwrap();
}

// --- Stage 2: Trust without conformance ---

#[test]
fn exact_trust_without_conformance_returns_run_supervised_conformance() {
    let base = temp_dir("trust-no-conf");
    fs::create_dir_all(&base).unwrap();

    let (candidates, candidate, _quarantine) = setup_candidate(&base);

    let exact_trust = ExactCandidateTrustStore::open(&base.join("trust")).unwrap();
    let profiles = LaunchProfileEvidenceStore::open(&base.join("profiles")).unwrap();
    let conformance_store = ConformanceEvidenceStore::open(&base.join("conformance")).unwrap();
    let approvals = InstallationApprovalStore::open(&base.join("approvals")).unwrap();
    let installed =
        InstalledPlugRegistry::open(&base.join("install"), &base.join("records")).unwrap();

    let request = valid_request(&candidate.candidate_id);
    let trust_record = exact_trust
        .create(&candidate, &request, "test-authority")
        .unwrap();

    let before = snapshot(&base);
    let plan = plan_installation(
        &request,
        &candidates,
        &exact_trust,
        &profiles,
        &conformance_store,
        &approvals,
        &installed,
    )
    .unwrap();

    assert_eq!(
        plan.action,
        InstallationPlanAction::RunSupervisedConformance
    );
    assert_eq!(plan.candidate_id, candidate.candidate_id);
    assert_eq!(
        plan.exact_candidate_trust_record_digest.as_deref(),
        Some(trust_record.record_digest.as_str())
    );
    assert!(plan.trust_evidence_digest.is_some());
    assert!(plan.launch_profile_evidence_digest.is_none());
    assert!(plan.conformance_evidence_id.is_none());
    assert!(plan.conformance_evidence_digest.is_none());
    assert!(plan.installation_approval_id.is_none());
    assert!(plan.installation_approval_digest.is_none());
    assert!(plan.installed_id.is_none());
    assert!(plan.installed_record_digest.is_none());
    assert_eq!(before, snapshot(&base));

    fs::remove_dir_all(base).unwrap();
}

// --- Stage 3: Conformance -> CreateInstallationApproval ---

#[test]
fn current_passed_conformance_returns_create_installation_approval() {
    let base = temp_dir("conf-to-approval");
    fs::create_dir_all(&base).unwrap();

    let (candidates, candidate, _quarantine) = setup_candidate(&base);

    let exact_trust = ExactCandidateTrustStore::open(&base.join("trust")).unwrap();
    let profiles = LaunchProfileEvidenceStore::open(&base.join("profiles")).unwrap();
    let conformance_store = ConformanceEvidenceStore::open(&base.join("conformance")).unwrap();
    let approvals = InstallationApprovalStore::open(&base.join("approvals")).unwrap();
    let installed =
        InstalledPlugRegistry::open(&base.join("install"), &base.join("records")).unwrap();

    let request = valid_request(&candidate.candidate_id);
    let trust_record = exact_trust
        .create(&candidate, &request, "test-authority")
        .unwrap();
    let trust_evidence = PackageTrustEvidence::exact_candidate(&trust_record).unwrap();

    let launch = build_launch_profile_evidence(&candidate);
    profiles.create(&launch).unwrap();

    let suite_digest = current_suite_digest().unwrap();
    let conformance = build_passing_conformance(
        &candidate,
        &trust_evidence,
        &launch,
        &suite_digest,
        1000,
        "c8688504-93a9-4321-b5a4-7535e7e9af06",
    );
    conformance_store.create(&conformance).unwrap();

    let before = snapshot(&base);
    let plan = plan_installation(
        &request,
        &candidates,
        &exact_trust,
        &profiles,
        &conformance_store,
        &approvals,
        &installed,
    )
    .unwrap();

    assert_eq!(
        plan.action,
        InstallationPlanAction::CreateInstallationApproval
    );
    assert_eq!(plan.candidate_id, candidate.candidate_id);
    assert_eq!(
        plan.exact_candidate_trust_record_digest.as_deref(),
        Some(trust_record.record_digest.as_str())
    );
    assert_eq!(
        plan.trust_evidence_digest.as_deref(),
        Some(trust_evidence.evidence_digest.as_str())
    );
    assert_eq!(
        plan.launch_profile_evidence_digest.as_deref(),
        Some(launch.profile_evidence_digest.as_str())
    );
    assert_eq!(
        plan.conformance_evidence_id.as_deref(),
        Some(conformance.evidence_id.as_str())
    );
    assert_eq!(
        plan.conformance_evidence_digest.as_deref(),
        Some(conformance.evidence_digest.as_str())
    );
    assert!(plan.installation_approval_id.is_none());
    assert!(plan.installation_approval_digest.is_none());
    assert!(plan.installed_id.is_none());
    assert!(plan.installed_record_digest.is_none());
    assert_eq!(before, snapshot(&base));

    fs::remove_dir_all(base).unwrap();
}

#[test]
fn multiple_passed_conformances_select_greatest_ended_unix_ms_then_greatest_evidence_id() {
    let base = temp_dir("multi-conf");
    fs::create_dir_all(&base).unwrap();

    let (candidates, candidate, _quarantine) = setup_candidate(&base);

    let exact_trust = ExactCandidateTrustStore::open(&base.join("trust")).unwrap();
    let profiles = LaunchProfileEvidenceStore::open(&base.join("profiles")).unwrap();
    let conformance_store = ConformanceEvidenceStore::open(&base.join("conformance")).unwrap();
    let approvals = InstallationApprovalStore::open(&base.join("approvals")).unwrap();
    let installed =
        InstalledPlugRegistry::open(&base.join("install"), &base.join("records")).unwrap();

    let request = valid_request(&candidate.candidate_id);
    let trust_record = exact_trust
        .create(&candidate, &request, "test-authority")
        .unwrap();
    let trust_evidence = PackageTrustEvidence::exact_candidate(&trust_record).unwrap();

    let launch = build_launch_profile_evidence(&candidate);
    profiles.create(&launch).unwrap();

    let suite_digest = current_suite_digest().unwrap();

    // Older conformance (earlier ended_unix_ms)
    let conf_old = build_passing_conformance(
        &candidate,
        &trust_evidence,
        &launch,
        &suite_digest,
        1000,
        "a0000000-93a9-4321-b5a4-7535e7e9af06",
    );
    conformance_store.create(&conf_old).unwrap();

    // Newer conformance (later ended_unix_ms)
    let conf_new = build_passing_conformance(
        &candidate,
        &trust_evidence,
        &launch,
        &suite_digest,
        2000,
        "b0000000-93a9-4321-b5a4-7535e7e9af06",
    );
    conformance_store.create(&conf_new).unwrap();

    // Another with same ended_unix_ms but greater evidence_id
    let conf_same_time_greater_id = build_passing_conformance(
        &candidate,
        &trust_evidence,
        &launch,
        &suite_digest,
        2000,
        "c0000000-93a9-4321-b5a4-7535e7e9af06",
    );
    conformance_store
        .create(&conf_same_time_greater_id)
        .unwrap();

    let before = snapshot(&base);
    let plan = plan_installation(
        &request,
        &candidates,
        &exact_trust,
        &profiles,
        &conformance_store,
        &approvals,
        &installed,
    )
    .unwrap();

    // Should select the one with greatest ended_unix_ms=2000 and greatest evidence_id ("c0...")
    assert_eq!(
        plan.action,
        InstallationPlanAction::CreateInstallationApproval
    );
    assert_eq!(
        plan.conformance_evidence_id.as_deref(),
        Some(conf_same_time_greater_id.evidence_id.as_str())
    );
    assert_eq!(
        plan.conformance_evidence_digest.as_deref(),
        Some(conf_same_time_greater_id.evidence_digest.as_str())
    );
    assert_eq!(before, snapshot(&base));

    fs::remove_dir_all(base).unwrap();
}

#[test]
fn failed_conformance_ignored() {
    let base = temp_dir("failed-conf");
    fs::create_dir_all(&base).unwrap();

    let (candidates, candidate, _quarantine) = setup_candidate(&base);

    let exact_trust = ExactCandidateTrustStore::open(&base.join("trust")).unwrap();
    let profiles = LaunchProfileEvidenceStore::open(&base.join("profiles")).unwrap();
    let conformance_store = ConformanceEvidenceStore::open(&base.join("conformance")).unwrap();
    let approvals = InstallationApprovalStore::open(&base.join("approvals")).unwrap();
    let installed =
        InstalledPlugRegistry::open(&base.join("install"), &base.join("records")).unwrap();

    let request = valid_request(&candidate.candidate_id);
    let trust_record = exact_trust
        .create(&candidate, &request, "test-authority")
        .unwrap();
    let trust_evidence = PackageTrustEvidence::exact_candidate(&trust_record).unwrap();

    let launch = build_launch_profile_evidence(&candidate);
    profiles.create(&launch).unwrap();

    let suite_digest = current_suite_digest().unwrap();

    // A failed conformance (one case Failed)
    let failed_cases = {
        let mut cases = all_passed_cases();
        cases[3] = ConformanceCaseEvidence {
            case_id: "provider_identity".into(),
            disposition: CaseDisposition::Failed,
            safe_diagnostic_code: Some("test-failure".into()),
        };
        cases
    };
    let mut failed_conf = ConformanceEvidence {
        schema_version: 1,
        evidence_id: "f0000000-93a9-4321-b5a4-7535e7e9af06".into(),
        candidate_id: candidate.candidate_id.clone(),
        package_id: candidate.package_id.clone(),
        package_version: candidate.package_version.clone(),
        semantic_package_digest: candidate.semantic_package_digest.clone(),
        payloads: candidate.payloads.clone(),
        capabilities: candidate.capabilities.clone(),
        trust_evidence_digest: trust_evidence.evidence_digest.clone(),
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
        suite_digest: suite_digest.clone(),
        test_configuration_digest: format!("sha256:{}", "d".repeat(64)),
        started_unix_ms: 1000,
        ended_unix_ms: 5000,
        cases: failed_cases,
        disposition: ConformanceDisposition::Failed,
        retry_count: 0,
        raw_stderr_persisted: false,
        evidence_digest: String::new(),
    };
    {
        let mut copy = failed_conf.clone();
        copy.evidence_digest.clear();
        failed_conf.evidence_digest = sha256(&serde_json_canonicalizer::to_vec(&copy).unwrap());
    }
    failed_conf.validate().unwrap();
    conformance_store.create(&failed_conf).unwrap();

    let before = snapshot(&base);
    let plan = plan_installation(
        &request,
        &candidates,
        &exact_trust,
        &profiles,
        &conformance_store,
        &approvals,
        &installed,
    )
    .unwrap();

    // Failed conformance is ignored, plan should fall back to RunSupervisedConformance
    assert_eq!(
        plan.action,
        InstallationPlanAction::RunSupervisedConformance
    );
    assert_eq!(before, snapshot(&base));

    fs::remove_dir_all(base).unwrap();
}

#[test]
fn interrupted_conformance_ignored() {
    let base = temp_dir("interrupted-conf");
    fs::create_dir_all(&base).unwrap();

    let (candidates, candidate, _quarantine) = setup_candidate(&base);

    let exact_trust = ExactCandidateTrustStore::open(&base.join("trust")).unwrap();
    let profiles = LaunchProfileEvidenceStore::open(&base.join("profiles")).unwrap();
    let conformance_store = ConformanceEvidenceStore::open(&base.join("conformance")).unwrap();
    let approvals = InstallationApprovalStore::open(&base.join("approvals")).unwrap();
    let installed =
        InstalledPlugRegistry::open(&base.join("install"), &base.join("records")).unwrap();

    let request = valid_request(&candidate.candidate_id);
    let trust_record = exact_trust
        .create(&candidate, &request, "test-authority")
        .unwrap();
    let trust_evidence = PackageTrustEvidence::exact_candidate(&trust_record).unwrap();

    let launch = build_launch_profile_evidence(&candidate);
    profiles.create(&launch).unwrap();

    let suite_digest = current_suite_digest().unwrap();

    let interrupted_cases = {
        let mut cases = all_passed_cases();
        cases[4] = ConformanceCaseEvidence {
            case_id: "complete_discovery_exact_operations".into(),
            disposition: CaseDisposition::Interrupted,
            safe_diagnostic_code: Some("test-interrupt".into()),
        };
        cases
    };
    let mut interrupted_conf = ConformanceEvidence {
        schema_version: 1,
        evidence_id: "aaaaaaaa-93a9-4321-b5a4-7535e7e9af06".into(),
        candidate_id: candidate.candidate_id.clone(),
        package_id: candidate.package_id.clone(),
        package_version: candidate.package_version.clone(),
        semantic_package_digest: candidate.semantic_package_digest.clone(),
        payloads: candidate.payloads.clone(),
        capabilities: candidate.capabilities.clone(),
        trust_evidence_digest: trust_evidence.evidence_digest.clone(),
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
        suite_digest: suite_digest.clone(),
        test_configuration_digest: format!("sha256:{}", "d".repeat(64)),
        started_unix_ms: 1000,
        ended_unix_ms: 5000,
        cases: interrupted_cases,
        disposition: ConformanceDisposition::Interrupted,
        retry_count: 0,
        raw_stderr_persisted: false,
        evidence_digest: String::new(),
    };
    {
        let mut copy = interrupted_conf.clone();
        copy.evidence_digest.clear();
        interrupted_conf.evidence_digest =
            sha256(&serde_json_canonicalizer::to_vec(&copy).unwrap());
    }
    interrupted_conf.validate().unwrap();
    conformance_store.create(&interrupted_conf).unwrap();

    let before = snapshot(&base);
    let plan = plan_installation(
        &request,
        &candidates,
        &exact_trust,
        &profiles,
        &conformance_store,
        &approvals,
        &installed,
    )
    .unwrap();

    assert_eq!(
        plan.action,
        InstallationPlanAction::RunSupervisedConformance
    );
    assert_eq!(before, snapshot(&base));

    fs::remove_dir_all(base).unwrap();
}

#[test]
fn invalidated_conformance_ignored() {
    let base = temp_dir("invalidated-conf");
    fs::create_dir_all(&base).unwrap();

    let (candidates, candidate, _quarantine) = setup_candidate(&base);

    let exact_trust = ExactCandidateTrustStore::open(&base.join("trust")).unwrap();
    let profiles = LaunchProfileEvidenceStore::open(&base.join("profiles")).unwrap();
    let conformance_store = ConformanceEvidenceStore::open(&base.join("conformance")).unwrap();
    let approvals = InstallationApprovalStore::open(&base.join("approvals")).unwrap();
    let installed =
        InstalledPlugRegistry::open(&base.join("install"), &base.join("records")).unwrap();

    let request = valid_request(&candidate.candidate_id);
    let trust_record = exact_trust
        .create(&candidate, &request, "test-authority")
        .unwrap();
    let trust_evidence = PackageTrustEvidence::exact_candidate(&trust_record).unwrap();

    let launch = build_launch_profile_evidence(&candidate);
    profiles.create(&launch).unwrap();

    let suite_digest = current_suite_digest().unwrap();

    // Invalidated: all Passed cases but disposition is Invalidated
    let mut invalidated_conf = ConformanceEvidence {
        schema_version: 1,
        evidence_id: "bbbbbbbb-93a9-4321-b5a4-7535e7e9af06".into(),
        candidate_id: candidate.candidate_id.clone(),
        package_id: candidate.package_id.clone(),
        package_version: candidate.package_version.clone(),
        semantic_package_digest: candidate.semantic_package_digest.clone(),
        payloads: candidate.payloads.clone(),
        capabilities: candidate.capabilities.clone(),
        trust_evidence_digest: trust_evidence.evidence_digest.clone(),
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
        suite_digest: suite_digest.clone(),
        test_configuration_digest: format!("sha256:{}", "d".repeat(64)),
        started_unix_ms: 1000,
        ended_unix_ms: 5000,
        cases: all_passed_cases(),
        disposition: ConformanceDisposition::Invalidated,
        retry_count: 0,
        raw_stderr_persisted: false,
        evidence_digest: String::new(),
    };
    {
        let mut copy = invalidated_conf.clone();
        copy.evidence_digest.clear();
        invalidated_conf.evidence_digest =
            sha256(&serde_json_canonicalizer::to_vec(&copy).unwrap());
    }
    invalidated_conf.validate().unwrap();
    conformance_store.create(&invalidated_conf).unwrap();

    let before = snapshot(&base);
    let plan = plan_installation(
        &request,
        &candidates,
        &exact_trust,
        &profiles,
        &conformance_store,
        &approvals,
        &installed,
    )
    .unwrap();

    assert_eq!(
        plan.action,
        InstallationPlanAction::RunSupervisedConformance
    );
    assert_eq!(before, snapshot(&base));

    fs::remove_dir_all(base).unwrap();
}

#[test]
fn stale_passed_conformance_ignored() {
    let base = temp_dir("stale-conf");
    fs::create_dir_all(&base).unwrap();

    let (candidates, candidate, _quarantine) = setup_candidate(&base);

    let exact_trust = ExactCandidateTrustStore::open(&base.join("trust")).unwrap();
    let profiles = LaunchProfileEvidenceStore::open(&base.join("profiles")).unwrap();
    let conformance_store = ConformanceEvidenceStore::open(&base.join("conformance")).unwrap();
    let approvals = InstallationApprovalStore::open(&base.join("approvals")).unwrap();
    let installed =
        InstalledPlugRegistry::open(&base.join("install"), &base.join("records")).unwrap();

    let request = valid_request(&candidate.candidate_id);
    let trust_record = exact_trust
        .create(&candidate, &request, "test-authority")
        .unwrap();
    let trust_evidence = PackageTrustEvidence::exact_candidate(&trust_record).unwrap();

    let launch = build_launch_profile_evidence(&candidate);
    profiles.create(&launch).unwrap();

    let _suite_digest = current_suite_digest().unwrap();

    // Stale: wrong suite_digest
    let mut stale_conf = build_passing_conformance(
        &candidate,
        &trust_evidence,
        &launch,
        "sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
        5000,
        "e0e0e0e0-93a9-4321-b5a4-7535e7e9af06",
    );
    // Override the suite_digest after creation
    stale_conf.suite_digest = format!("sha256:{}", "f".repeat(64));
    {
        let mut copy = stale_conf.clone();
        copy.evidence_digest.clear();
        stale_conf.evidence_digest = sha256(&serde_json_canonicalizer::to_vec(&copy).unwrap());
    }
    stale_conf.validate().unwrap();
    conformance_store.create(&stale_conf).unwrap();

    let before = snapshot(&base);
    let plan = plan_installation(
        &request,
        &candidates,
        &exact_trust,
        &profiles,
        &conformance_store,
        &approvals,
        &installed,
    )
    .unwrap();

    assert_eq!(
        plan.action,
        InstallationPlanAction::RunSupervisedConformance
    );
    assert_eq!(before, snapshot(&base));

    fs::remove_dir_all(base).unwrap();
}

#[test]
fn launch_profile_not_exposed_without_conformance() {
    let base = temp_dir("lp-not-exposed");
    fs::create_dir_all(&base).unwrap();

    let (candidates, candidate, _quarantine) = setup_candidate(&base);

    let exact_trust = ExactCandidateTrustStore::open(&base.join("trust")).unwrap();
    let profiles = LaunchProfileEvidenceStore::open(&base.join("profiles")).unwrap();
    let conformance_store = ConformanceEvidenceStore::open(&base.join("conformance")).unwrap();
    let approvals = InstallationApprovalStore::open(&base.join("approvals")).unwrap();
    let installed =
        InstalledPlugRegistry::open(&base.join("install"), &base.join("records")).unwrap();

    let request = valid_request(&candidate.candidate_id);
    exact_trust
        .create(&candidate, &request, "test-authority")
        .unwrap();

    // Create a launch profile, but no conformance references it
    let launch = build_launch_profile_evidence(&candidate);
    profiles.create(&launch).unwrap();

    let before = snapshot(&base);
    let plan = plan_installation(
        &request,
        &candidates,
        &exact_trust,
        &profiles,
        &conformance_store,
        &approvals,
        &installed,
    )
    .unwrap();

    assert_eq!(
        plan.action,
        InstallationPlanAction::RunSupervisedConformance
    );
    // Launch profile evidence digest must not be exposed
    assert!(plan.launch_profile_evidence_digest.is_none());
    assert_eq!(before, snapshot(&base));

    fs::remove_dir_all(base).unwrap();
}

// --- Stage 4: Approval -> PublishDisabledInstallation ---

#[test]
fn current_installation_approval_returns_publish_disabled_installation() {
    let base = temp_dir("approval-to-install");
    fs::create_dir_all(&base).unwrap();

    let (candidates, candidate, _quarantine) = setup_candidate(&base);

    let exact_trust = ExactCandidateTrustStore::open(&base.join("trust")).unwrap();
    let profiles = LaunchProfileEvidenceStore::open(&base.join("profiles")).unwrap();
    let conformance_store = ConformanceEvidenceStore::open(&base.join("conformance")).unwrap();
    let approval_dir = base.join("approvals");
    let _approvals = InstallationApprovalStore::open(&approval_dir).unwrap();
    let installed =
        InstalledPlugRegistry::open(&base.join("install"), &base.join("records")).unwrap();

    let request = valid_request(&candidate.candidate_id);
    let trust_record = exact_trust
        .create(&candidate, &request, "test-authority")
        .unwrap();
    let trust_evidence = PackageTrustEvidence::exact_candidate(&trust_record).unwrap();

    let launch = build_launch_profile_evidence(&candidate);
    profiles.create(&launch).unwrap();

    let suite_digest = current_suite_digest().unwrap();
    let conformance = build_passing_conformance(
        &candidate,
        &trust_evidence,
        &launch,
        &suite_digest,
        1000,
        "f0f0f0f0-93a9-4321-b5a4-7535e7e9af06",
    );
    conformance_store.create(&conformance).unwrap();

    let approval = build_approval_record(
        &candidate,
        &trust_evidence,
        &launch,
        &conformance,
        "a0000000-93a9-4321-b5a4-7535e7e9af06",
        "test-authority",
        2000,
    );
    write_approval_json(&approval_dir, &approval);

    // Re-open so load_all picks up the manually written record
    let approvals_reopened = InstallationApprovalStore::open_existing(&approval_dir).unwrap();

    let before = snapshot(&base);
    let plan = plan_installation(
        &request,
        &candidates,
        &exact_trust,
        &profiles,
        &conformance_store,
        &approvals_reopened,
        &installed,
    )
    .unwrap();

    assert_eq!(
        plan.action,
        InstallationPlanAction::PublishDisabledInstallation
    );
    assert_eq!(plan.candidate_id, candidate.candidate_id);
    assert_eq!(
        plan.exact_candidate_trust_record_digest.as_deref(),
        Some(trust_record.record_digest.as_str())
    );
    assert_eq!(
        plan.trust_evidence_digest.as_deref(),
        Some(trust_evidence.evidence_digest.as_str())
    );
    assert_eq!(
        plan.launch_profile_evidence_digest.as_deref(),
        Some(launch.profile_evidence_digest.as_str())
    );
    assert_eq!(
        plan.conformance_evidence_id.as_deref(),
        Some(conformance.evidence_id.as_str())
    );
    assert_eq!(
        plan.conformance_evidence_digest.as_deref(),
        Some(conformance.evidence_digest.as_str())
    );
    assert_eq!(
        plan.installation_approval_id.as_deref(),
        Some(approval.approval_id.as_str())
    );
    assert_eq!(
        plan.installation_approval_digest.as_deref(),
        Some(approval.record_digest.as_str())
    );
    assert!(plan.installed_id.is_none());
    assert!(plan.installed_record_digest.is_none());
    assert_eq!(before, snapshot(&base));

    fs::remove_dir_all(base).unwrap();
}

// --- Stage 5: Installed -> Complete ---

#[test]
fn current_installed_returns_complete() {
    let base = temp_dir("installed-complete");
    fs::create_dir_all(&base).unwrap();

    let (candidates, candidate, quarantine_root) = setup_candidate(&base);

    let exact_trust = ExactCandidateTrustStore::open(&base.join("trust")).unwrap();
    let profiles = LaunchProfileEvidenceStore::open(&base.join("profiles")).unwrap();
    let conformance_store = ConformanceEvidenceStore::open(&base.join("conformance")).unwrap();
    let approval_dir = base.join("approvals");
    let _approvals = InstallationApprovalStore::open(&approval_dir).unwrap();
    let install_dir = base.join("install");
    let records_dir = base.join("records");
    let _installed = InstalledPlugRegistry::open(&install_dir, &records_dir).unwrap();

    let request = valid_request(&candidate.candidate_id);
    let trust_record = exact_trust
        .create(&candidate, &request, "test-authority")
        .unwrap();
    let trust_evidence = PackageTrustEvidence::exact_candidate(&trust_record).unwrap();

    let launch = build_launch_profile_evidence(&candidate);
    profiles.create(&launch).unwrap();

    let suite_digest = current_suite_digest().unwrap();
    let conformance = build_passing_conformance(
        &candidate,
        &trust_evidence,
        &launch,
        &suite_digest,
        1000,
        "1a1a1a1a-93a9-4321-b5a4-7535e7e9af06",
    );
    conformance_store.create(&conformance).unwrap();

    let approval = build_approval_record(
        &candidate,
        &trust_evidence,
        &launch,
        &conformance,
        "b0000000-93a9-4321-b5a4-7535e7e9af06",
        "test-authority",
        2000,
    );
    write_approval_json(&approval_dir, &approval);

    let installed_id = "c0000000-93a9-4321-b5a4-7535e7e9af06";
    let relative_path = format!("plug-{installed_id}");
    let install_target = install_dir.join(&relative_path);
    fs::create_dir_all(&install_target).unwrap();
    copy_files_from_quarantine(&quarantine_root, &candidate, &install_target);

    let installed_record = build_installed_record(
        &candidate,
        &trust_evidence,
        &launch,
        &conformance,
        &approval,
        installed_id,
        3000,
        &relative_path,
    );
    write_installed_json(&records_dir, &installed_record);

    let approvals_reopened = InstallationApprovalStore::open_existing(&approval_dir).unwrap();
    let installed_reopened =
        InstalledPlugRegistry::open_existing(&install_dir, &records_dir).unwrap();

    let before = snapshot(&base);
    let plan = plan_installation(
        &request,
        &candidates,
        &exact_trust,
        &profiles,
        &conformance_store,
        &approvals_reopened,
        &installed_reopened,
    )
    .unwrap();

    assert_eq!(plan.action, InstallationPlanAction::Complete);
    assert_eq!(plan.candidate_id, candidate.candidate_id);
    assert_eq!(
        plan.exact_candidate_trust_record_digest.as_deref(),
        Some(trust_record.record_digest.as_str())
    );
    assert_eq!(
        plan.trust_evidence_digest.as_deref(),
        Some(trust_evidence.evidence_digest.as_str())
    );
    assert_eq!(
        plan.launch_profile_evidence_digest.as_deref(),
        Some(launch.profile_evidence_digest.as_str())
    );
    assert_eq!(
        plan.conformance_evidence_id.as_deref(),
        Some(conformance.evidence_id.as_str())
    );
    assert_eq!(
        plan.conformance_evidence_digest.as_deref(),
        Some(conformance.evidence_digest.as_str())
    );
    assert_eq!(
        plan.installation_approval_id.as_deref(),
        Some(approval.approval_id.as_str())
    );
    assert_eq!(
        plan.installation_approval_digest.as_deref(),
        Some(approval.record_digest.as_str())
    );
    assert_eq!(
        plan.installed_id.as_deref(),
        Some(installed_record.installed_id.as_str())
    );
    assert_eq!(
        plan.installed_record_digest.as_deref(),
        Some(installed_record.record_digest.as_str())
    );
    assert_eq!(before, snapshot(&base));

    fs::remove_dir_all(base).unwrap();
}

// --- Error path tests ---

#[test]
fn request_validation_fails_before_evidence_reads() {
    let base = temp_dir("req-validation");
    fs::create_dir_all(&base).unwrap();
    let sentinel_dir = base.join("sentinel");
    fs::create_dir(&sentinel_dir).unwrap();

    let (candidates, candidate, _quarantine) = setup_candidate(&base);

    let trust = ExactCandidateTrustStore::open(&base.join("trust")).unwrap();
    let profiles = LaunchProfileEvidenceStore::open(&base.join("profiles")).unwrap();
    let conformance = ConformanceEvidenceStore::open(&base.join("conformance")).unwrap();
    let approvals = InstallationApprovalStore::open(&base.join("approvals")).unwrap();
    let installed =
        InstalledPlugRegistry::open(&base.join("install"), &base.join("records")).unwrap();

    let before = snapshot(&sentinel_dir);

    // Wrong schema
    let mut req = valid_request(&candidate.candidate_id);
    req.schema = "wrong".to_owned();
    let err = plan_installation(
        &req,
        &candidates,
        &trust,
        &profiles,
        &conformance,
        &approvals,
        &installed,
    )
    .unwrap_err();
    assert_eq!(err.code, "installation_plan_request_invalid");

    // Non-canonical UUID (no hyphens)
    req = valid_request(&candidate.candidate_id);
    req.candidate_id = "3d846d4001fc4e1eb77d83944dbed76f".to_owned();
    let err = plan_installation(
        &req,
        &candidates,
        &trust,
        &profiles,
        &conformance,
        &approvals,
        &installed,
    )
    .unwrap_err();
    assert_eq!(err.code, "installation_plan_request_invalid");

    // Uppercase UUID
    req = valid_request(&candidate.candidate_id);
    req.candidate_id = "3D846D40-01FC-4E1E-B77D-83944DBED76F".to_owned();
    let err = plan_installation(
        &req,
        &candidates,
        &trust,
        &profiles,
        &conformance,
        &approvals,
        &installed,
    )
    .unwrap_err();
    assert_eq!(err.code, "installation_plan_request_invalid");

    // False supervised execution approval
    req = valid_request(&candidate.candidate_id);
    req.conformance.allow_non_isolated_supervised_execution = false;
    let err = plan_installation(
        &req,
        &candidates,
        &trust,
        &profiles,
        &conformance,
        &approvals,
        &installed,
    )
    .unwrap_err();
    assert_eq!(err.code, "installation_plan_request_invalid");

    assert_eq!(before, snapshot(&sentinel_dir));
    fs::remove_dir_all(base).unwrap();
}

#[test]
fn missing_candidate_fails_with_frozen_error() {
    let base = temp_dir("missing-candidate");
    fs::create_dir_all(&base).unwrap();
    let candidates =
        CandidateRegistry::open(&base.join("candidates"), &base.join("quarantine")).unwrap();
    let trust = ExactCandidateTrustStore::open(&base.join("trust")).unwrap();
    let profiles = LaunchProfileEvidenceStore::open(&base.join("profiles")).unwrap();
    let conformance = ConformanceEvidenceStore::open(&base.join("conformance")).unwrap();
    let approvals = InstallationApprovalStore::open(&base.join("approvals")).unwrap();
    let installed =
        InstalledPlugRegistry::open(&base.join("install"), &base.join("records")).unwrap();

    let req = valid_request("3d846d40-01fc-4e1e-b77d-83944dbed76f");
    let before = snapshot(&base);
    let err = plan_installation(
        &req,
        &candidates,
        &trust,
        &profiles,
        &conformance,
        &approvals,
        &installed,
    )
    .unwrap_err();
    assert_eq!(err.code, "installation_plan_candidate_missing");
    assert!(err.message.contains("not present"));
    assert_eq!(before, snapshot(&base));

    fs::remove_dir_all(base).unwrap();
}

#[test]
fn mismatched_trust_fails_closed() {
    let base = temp_dir("mismatched-trust");
    fs::create_dir_all(&base).unwrap();

    let (candidates, candidate, _quarantine) = setup_candidate(&base);

    let trust = ExactCandidateTrustStore::open(&base.join("trust")).unwrap();
    let profiles = LaunchProfileEvidenceStore::open(&base.join("profiles")).unwrap();
    let conformance = ConformanceEvidenceStore::open(&base.join("conformance")).unwrap();
    let approvals = InstallationApprovalStore::open(&base.join("approvals")).unwrap();
    let installed =
        InstalledPlugRegistry::open(&base.join("install"), &base.join("records")).unwrap();

    let request = valid_request(&candidate.candidate_id);
    trust
        .create(&candidate, &request, "test-authority")
        .unwrap();

    // Mutate the candidate record's digest field to cause require_for_candidate failure
    let candidate_path = base
        .join("candidates")
        .join(format!("{}.json", candidate.candidate_id));
    let text = fs::read_to_string(&candidate_path).unwrap();
    let mut value: serde_json::Value = serde_json::from_str(&text).unwrap();
    value["inspection_evidence_digest"] =
        serde_json::Value::String(format!("sha256:{}", "f".repeat(64)));
    let mut record_copy = value.clone();
    record_copy["record_digest"] = serde_json::Value::String(String::new());
    let covered = serde_json_canonicalizer::to_vec(&record_copy).unwrap();
    let new_digest = sha256(&covered);
    record_copy["record_digest"] = serde_json::Value::String(new_digest);
    fs::write(&candidate_path, &serde_json::to_vec(&record_copy).unwrap()).unwrap();

    let before = snapshot(&base);
    let err = plan_installation(
        &request,
        &candidates,
        &trust,
        &profiles,
        &conformance,
        &approvals,
        &installed,
    )
    .unwrap_err();
    assert_eq!(err.code, "installation_trust_candidate_mismatch");
    assert_eq!(before, snapshot(&base));

    fs::remove_dir_all(base).unwrap();
}

#[test]
fn stale_approval_fails_closed() {
    let base = temp_dir("stale-approval");
    fs::create_dir_all(&base).unwrap();

    let (candidates, candidate, _quarantine) = setup_candidate(&base);

    let exact_trust = ExactCandidateTrustStore::open(&base.join("trust")).unwrap();
    let profiles = LaunchProfileEvidenceStore::open(&base.join("profiles")).unwrap();
    let conformance_store = ConformanceEvidenceStore::open(&base.join("conformance")).unwrap();
    let approval_dir = base.join("approvals");
    let _approvals = InstallationApprovalStore::open(&approval_dir).unwrap();
    let installed =
        InstalledPlugRegistry::open(&base.join("install"), &base.join("records")).unwrap();

    let request = valid_request(&candidate.candidate_id);
    let trust_record = exact_trust
        .create(&candidate, &request, "test-authority")
        .unwrap();
    let trust_evidence = PackageTrustEvidence::exact_candidate(&trust_record).unwrap();

    let launch = build_launch_profile_evidence(&candidate);
    profiles.create(&launch).unwrap();

    let _suite_digest = current_suite_digest().unwrap();
    let conformance = build_passing_conformance(
        &candidate,
        &trust_evidence,
        &launch,
        &_suite_digest,
        1000,
        "e0e0e0e0-93a9-4321-b5a4-7535e7e9af06",
    );
    conformance_store.create(&conformance).unwrap();

    // Build an approval with a stale trust_evidence_digest
    let mut stale_approval = build_approval_record(
        &candidate,
        &trust_evidence,
        &launch,
        &conformance,
        "e0e0e0e0-93a9-4321-b5a4-7535e7e9af06",
        "test-authority",
        2000,
    );
    // Override the trust_evidence with a stale semantic_package_digest
    stale_approval.trust_evidence = {
        let mut t = trust_evidence.clone();
        t.semantic_package_digest = format!("sha256:{}", "f".repeat(64));
        let mut copy = t.clone();
        copy.evidence_digest.clear();
        t.evidence_digest = sha256(&serde_json_canonicalizer::to_vec(&copy).unwrap());
        t
    };
    {
        let mut copy = stale_approval.clone();
        copy.record_digest.clear();
        stale_approval.record_digest = sha256(&serde_json_canonicalizer::to_vec(&copy).unwrap());
    }
    stale_approval.validate().unwrap();
    write_approval_json(&approval_dir, &stale_approval);

    let approvals_reopened = InstallationApprovalStore::open_existing(&approval_dir).unwrap();

    let before = snapshot(&base);
    let err = plan_installation(
        &request,
        &candidates,
        &exact_trust,
        &profiles,
        &conformance_store,
        &approvals_reopened,
        &installed,
    )
    .unwrap_err();
    assert!(err.code.contains("stale"));
    assert_eq!(before, snapshot(&base));

    fs::remove_dir_all(base).unwrap();
}

#[test]
fn stale_installed_record_fails_closed() {
    let base = temp_dir("stale-installed");
    fs::create_dir_all(&base).unwrap();

    let (candidates, candidate, quarantine_root) = setup_candidate(&base);

    let exact_trust = ExactCandidateTrustStore::open(&base.join("trust")).unwrap();
    let profiles = LaunchProfileEvidenceStore::open(&base.join("profiles")).unwrap();
    let conformance_store = ConformanceEvidenceStore::open(&base.join("conformance")).unwrap();
    let approval_dir = base.join("approvals");
    let _approvals = InstallationApprovalStore::open(&approval_dir).unwrap();
    let install_dir = base.join("install");
    let records_dir = base.join("records");
    let _installed = InstalledPlugRegistry::open(&install_dir, &records_dir).unwrap();

    let request = valid_request(&candidate.candidate_id);
    let trust_record = exact_trust
        .create(&candidate, &request, "test-authority")
        .unwrap();
    let trust_evidence = PackageTrustEvidence::exact_candidate(&trust_record).unwrap();

    let launch = build_launch_profile_evidence(&candidate);
    profiles.create(&launch).unwrap();

    let suite_digest = current_suite_digest().unwrap();
    let conformance = build_passing_conformance(
        &candidate,
        &trust_evidence,
        &launch,
        &suite_digest,
        1000,
        "cccccccc-93a9-4321-b5a4-7535e7e9af06",
    );
    conformance_store.create(&conformance).unwrap();

    let approval = build_approval_record(
        &candidate,
        &trust_evidence,
        &launch,
        &conformance,
        "cccccccc-93a9-4321-b5a4-7535e7e9af06",
        "test-authority",
        2000,
    );
    write_approval_json(&approval_dir, &approval);

    let installed_id = "cccccccc-93a9-4321-b5a4-7535e7e9af06";
    let relative_path = format!("plug-{installed_id}");
    let install_target = install_dir.join(&relative_path);
    fs::create_dir_all(&install_target).unwrap();
    copy_files_from_quarantine(&quarantine_root, &candidate, &install_target);

    // Build stale installed record with wrong approval_digest
    let mut stale_installed = build_installed_record(
        &candidate,
        &trust_evidence,
        &launch,
        &conformance,
        &approval,
        installed_id,
        3000,
        &relative_path,
    );
    stale_installed.installation_approval_digest = format!("sha256:{}", "f".repeat(64));
    {
        let mut copy = stale_installed.clone();
        copy.record_digest.clear();
        stale_installed.record_digest = sha256(&serde_json_canonicalizer::to_vec(&copy).unwrap());
    }
    stale_installed.validate().unwrap();
    write_installed_json(&records_dir, &stale_installed);

    let approvals_reopened = InstallationApprovalStore::open_existing(&approval_dir).unwrap();
    let installed_reopened =
        InstalledPlugRegistry::open_existing(&install_dir, &records_dir).unwrap();

    let before = snapshot(&base);
    let err = plan_installation(
        &request,
        &candidates,
        &exact_trust,
        &profiles,
        &conformance_store,
        &approvals_reopened,
        &installed_reopened,
    )
    .unwrap_err();
    assert!(err.code.contains("stale"));
    assert_eq!(before, snapshot(&base));

    fs::remove_dir_all(base).unwrap();
}

#[test]
fn corrupt_store_evidence_fails_closed_not_treated_as_absence() {
    let base = temp_dir("corrupt");
    fs::create_dir_all(&base).unwrap();

    let (candidates, candidate, _quarantine) = setup_candidate(&base);

    let request = valid_request(&candidate.candidate_id);
    let trust = ExactCandidateTrustStore::open(&base.join("trust")).unwrap();
    trust
        .create(&candidate, &request, "test-authority")
        .unwrap();

    let profiles = LaunchProfileEvidenceStore::open(&base.join("profiles")).unwrap();
    let conformance_store = ConformanceEvidenceStore::open(&base.join("conformance")).unwrap();
    let approvals = InstallationApprovalStore::open(&base.join("approvals")).unwrap();
    let installed =
        InstalledPlugRegistry::open(&base.join("install"), &base.join("records")).unwrap();

    // Place a torn .tmp file in the trust store
    let torn = base.join("trust").join(".torn.tmp");
    fs::write(&torn, b"partial").unwrap();

    let before1 = snapshot(&base);
    let err = plan_installation(
        &request,
        &candidates,
        &trust,
        &profiles,
        &conformance_store,
        &approvals,
        &installed,
    )
    .unwrap_err();
    assert_eq!(err.code, "installation_trust_invalid");
    assert!(err.message.contains("torn"));
    assert_eq!(before1, snapshot(&base));

    // Remove torn file, add a non-JSON entry
    fs::remove_file(&torn).unwrap();
    let bad_entry = base.join("trust").join("bad.unknown");
    fs::write(&bad_entry, b"not json").unwrap();

    let before2 = snapshot(&base);
    let err = plan_installation(
        &request,
        &candidates,
        &trust,
        &profiles,
        &conformance_store,
        &approvals,
        &installed,
    )
    .unwrap_err();
    assert_eq!(err.code, "installation_trust_invalid");
    assert!(err.message.contains("unexpected"));
    assert_eq!(before2, snapshot(&base));

    fs::remove_dir_all(base).unwrap();
}

#[test]
fn corrupt_launch_profile_evidence_fails_closed() {
    let base = temp_dir("corrupt-lp");
    fs::create_dir_all(&base).unwrap();

    let (candidates, candidate, _quarantine) = setup_candidate(&base);

    let exact_trust = ExactCandidateTrustStore::open(&base.join("trust")).unwrap();
    let profiles = LaunchProfileEvidenceStore::open(&base.join("profiles")).unwrap();
    let conformance_store = ConformanceEvidenceStore::open(&base.join("conformance")).unwrap();
    let approvals = InstallationApprovalStore::open(&base.join("approvals")).unwrap();
    let installed =
        InstalledPlugRegistry::open(&base.join("install"), &base.join("records")).unwrap();

    let request = valid_request(&candidate.candidate_id);
    exact_trust
        .create(&candidate, &request, "test-authority")
        .unwrap();

    // Write a torn .tmp file into the launch profile store
    let torn = base.join("profiles").join(".torn.tmp");
    fs::write(&torn, b"partial").unwrap();

    let before = snapshot(&base);
    let err = plan_installation(
        &request,
        &candidates,
        &exact_trust,
        &profiles,
        &conformance_store,
        &approvals,
        &installed,
    )
    .unwrap_err();
    assert!(err.code.contains("invalid"));
    assert_eq!(before, snapshot(&base));

    fs::remove_dir_all(base).unwrap();
}

#[test]
fn corrupt_conformance_evidence_fails_closed() {
    let base = temp_dir("corrupt-conf");
    fs::create_dir_all(&base).unwrap();

    let (candidates, candidate, _quarantine) = setup_candidate(&base);

    let exact_trust = ExactCandidateTrustStore::open(&base.join("trust")).unwrap();
    let profiles = LaunchProfileEvidenceStore::open(&base.join("profiles")).unwrap();
    let conformance_store = ConformanceEvidenceStore::open(&base.join("conformance")).unwrap();
    let approvals = InstallationApprovalStore::open(&base.join("approvals")).unwrap();
    let installed =
        InstalledPlugRegistry::open(&base.join("install"), &base.join("records")).unwrap();

    let request = valid_request(&candidate.candidate_id);
    let trust_record = exact_trust
        .create(&candidate, &request, "test-authority")
        .unwrap();
    let _trust_evidence = PackageTrustEvidence::exact_candidate(&trust_record).unwrap();

    let launch = build_launch_profile_evidence(&candidate);
    profiles.create(&launch).unwrap();

    // Write a torn .tmp file in the conformance store
    let torn = base.join("conformance").join(".torn.tmp");
    fs::write(&torn, b"partial").unwrap();

    let before = snapshot(&base);
    let err = plan_installation(
        &request,
        &candidates,
        &exact_trust,
        &profiles,
        &conformance_store,
        &approvals,
        &installed,
    )
    .unwrap_err();
    assert!(err.code.contains("invalid"));
    assert_eq!(before, snapshot(&base));

    // Ensure error is NOT treated as absence (not RunSupervisedConformance)
    // Also test with a bad JSON entry
    fs::remove_file(&torn).unwrap();
    let bad = base.join("conformance").join("bad.unknown");
    fs::write(&bad, b"not json").unwrap();

    let before2 = snapshot(&base);
    let err = plan_installation(
        &request,
        &candidates,
        &exact_trust,
        &profiles,
        &conformance_store,
        &approvals,
        &installed,
    )
    .unwrap_err();
    assert!(err.code.contains("invalid"));
    assert_eq!(before2, snapshot(&base));

    fs::remove_dir_all(base).unwrap();
}

#[test]
fn corrupt_approval_evidence_fails_closed() {
    let base = temp_dir("corrupt-approval");
    fs::create_dir_all(&base).unwrap();

    let (candidates, candidate, _quarantine) = setup_candidate(&base);

    let exact_trust = ExactCandidateTrustStore::open(&base.join("trust")).unwrap();
    let profiles = LaunchProfileEvidenceStore::open(&base.join("profiles")).unwrap();
    let conformance_store = ConformanceEvidenceStore::open(&base.join("conformance")).unwrap();
    let approval_dir = base.join("approvals");
    let approvals = InstallationApprovalStore::open(&approval_dir).unwrap();
    let installed =
        InstalledPlugRegistry::open(&base.join("install"), &base.join("records")).unwrap();

    let request = valid_request(&candidate.candidate_id);
    let trust_record = exact_trust
        .create(&candidate, &request, "test-authority")
        .unwrap();
    let trust_evidence = PackageTrustEvidence::exact_candidate(&trust_record).unwrap();

    let launch = build_launch_profile_evidence(&candidate);
    profiles.create(&launch).unwrap();

    let suite_digest = current_suite_digest().unwrap();
    let conformance = build_passing_conformance(
        &candidate,
        &trust_evidence,
        &launch,
        &suite_digest,
        1000,
        "dddddddd-93a9-4321-b5a4-7535e7e9af06",
    );
    conformance_store.create(&conformance).unwrap();

    // Write a torn .tmp file into the approval store
    let torn = approval_dir.join(".torn.tmp");
    fs::write(&torn, b"partial").unwrap();

    let before = snapshot(&base);
    let err = plan_installation(
        &request,
        &candidates,
        &exact_trust,
        &profiles,
        &conformance_store,
        &approvals,
        &installed,
    )
    .unwrap_err();
    assert!(err.code.contains("invalid"));
    assert_eq!(before, snapshot(&base));

    fs::remove_dir_all(base).unwrap();
}

#[test]
fn corrupt_installed_evidence_fails_closed() {
    let base = temp_dir("corrupt-installed");
    fs::create_dir_all(&base).unwrap();

    let (candidates, candidate, _quarantine_root) = setup_candidate(&base);

    let exact_trust = ExactCandidateTrustStore::open(&base.join("trust")).unwrap();
    let profiles = LaunchProfileEvidenceStore::open(&base.join("profiles")).unwrap();
    let conformance_store = ConformanceEvidenceStore::open(&base.join("conformance")).unwrap();
    let approval_dir = base.join("approvals");
    let _approvals = InstallationApprovalStore::open(&approval_dir).unwrap();
    let install_dir = base.join("install");
    let records_dir = base.join("records");
    let installed = InstalledPlugRegistry::open(&install_dir, &records_dir).unwrap();

    let request = valid_request(&candidate.candidate_id);
    let trust_record = exact_trust
        .create(&candidate, &request, "test-authority")
        .unwrap();
    let trust_evidence = PackageTrustEvidence::exact_candidate(&trust_record).unwrap();

    let launch = build_launch_profile_evidence(&candidate);
    profiles.create(&launch).unwrap();

    let suite_digest = current_suite_digest().unwrap();
    let conformance = build_passing_conformance(
        &candidate,
        &trust_evidence,
        &launch,
        &suite_digest,
        1000,
        "bbbbbbbb-93a9-4321-b5a4-7535e7e9af06",
    );
    conformance_store.create(&conformance).unwrap();

    let approval = build_approval_record(
        &candidate,
        &trust_evidence,
        &launch,
        &conformance,
        "bbbbbbbb-93a9-4321-b5a4-7535e7e9af06",
        "test-authority",
        2000,
    );
    write_approval_json(&approval_dir, &approval);

    // Write a torn .tmp file into the installed records store
    let torn = records_dir.join(".torn.tmp");
    fs::write(&torn, b"partial").unwrap();

    let approvals_reopened = InstallationApprovalStore::open_existing(&approval_dir).unwrap();

    let before = snapshot(&base);
    let err = plan_installation(
        &request,
        &candidates,
        &exact_trust,
        &profiles,
        &conformance_store,
        &approvals_reopened,
        &installed,
    )
    .unwrap_err();
    assert!(err.code.contains("invalid"));
    assert_eq!(before, snapshot(&base));

    fs::remove_dir_all(base).unwrap();
}

#[test]
fn planning_never_mutates_filesystem() {
    let base = temp_dir("no-mutation");
    fs::create_dir_all(&base).unwrap();

    let (candidates, candidate, _quarantine) = setup_candidate(&base);

    let request = valid_request(&candidate.candidate_id);
    let trust = ExactCandidateTrustStore::open(&base.join("trust")).unwrap();
    let profiles = LaunchProfileEvidenceStore::open(&base.join("profiles")).unwrap();
    let conformance_store = ConformanceEvidenceStore::open(&base.join("conformance")).unwrap();
    let approvals = InstallationApprovalStore::open(&base.join("approvals")).unwrap();
    let installed =
        InstalledPlugRegistry::open(&base.join("install"), &base.join("records")).unwrap();

    // No trust state
    let before1 = snapshot(&base);
    plan_installation(
        &request,
        &candidates,
        &trust,
        &profiles,
        &conformance_store,
        &approvals,
        &installed,
    )
    .unwrap();
    assert_eq!(before1, snapshot(&base));

    // Create trust, verify planning doesn't mutate
    trust
        .create(&candidate, &request, "test-authority")
        .unwrap();
    let before2 = snapshot(&base);
    plan_installation(
        &request,
        &candidates,
        &trust,
        &profiles,
        &conformance_store,
        &approvals,
        &installed,
    )
    .unwrap();
    assert_eq!(before2, snapshot(&base));

    fs::remove_dir_all(base).unwrap();
}

#[test]
fn no_evidence_created_by_planning() {
    let base = temp_dir("no-evidence");
    fs::create_dir_all(&base).unwrap();

    let (candidates, candidate, _quarantine) = setup_candidate(&base);

    let request = valid_request(&candidate.candidate_id);
    let trust = ExactCandidateTrustStore::open(&base.join("trust")).unwrap();
    let profiles = LaunchProfileEvidenceStore::open(&base.join("profiles")).unwrap();
    let conformance_store = ConformanceEvidenceStore::open(&base.join("conformance")).unwrap();
    let approvals = InstallationApprovalStore::open(&base.join("approvals")).unwrap();
    let installed =
        InstalledPlugRegistry::open(&base.join("install"), &base.join("records")).unwrap();

    // Verify stores are empty
    assert!(trust.load_all().unwrap().is_empty());
    assert!(conformance_store.load_all().unwrap().is_empty());
    assert!(approvals.load_all().unwrap().is_empty());

    let before = snapshot(&base);
    plan_installation(
        &request,
        &candidates,
        &trust,
        &profiles,
        &conformance_store,
        &approvals,
        &installed,
    )
    .unwrap();
    assert_eq!(before, snapshot(&base));

    // Trust store still empty (planner didn't create trust)
    assert!(trust.load_all().unwrap().is_empty());
    assert!(conformance_store.load_all().unwrap().is_empty());
    assert!(approvals.load_all().unwrap().is_empty());

    fs::remove_dir_all(base).unwrap();
}

#[test]
fn validates_canonical_lowercase_hyphenated_uuid() {
    let req = valid_request("3d846d40-01fc-4e1e-b77d-83944dbed76f");
    let mut bad = req.clone();
    bad.candidate_id = "3D846D40-01FC-4E1E-B77D-83944DBED76F".to_owned();

    let base = temp_dir("uuid-check");
    fs::create_dir_all(&base).unwrap();
    let candidates =
        CandidateRegistry::open(&base.join("candidates"), &base.join("quarantine")).unwrap();
    let trust = ExactCandidateTrustStore::open(&base.join("trust")).unwrap();
    let profiles = LaunchProfileEvidenceStore::open(&base.join("profiles")).unwrap();
    let conformance_store = ConformanceEvidenceStore::open(&base.join("conformance")).unwrap();
    let approvals = InstallationApprovalStore::open(&base.join("approvals")).unwrap();
    let installed =
        InstalledPlugRegistry::open(&base.join("install"), &base.join("records")).unwrap();

    let err = plan_installation(
        &bad,
        &candidates,
        &trust,
        &profiles,
        &conformance_store,
        &approvals,
        &installed,
    )
    .unwrap_err();
    assert_eq!(err.code, "installation_plan_request_invalid");

    let no_hyphens = "3d846d4001fc4e1eb77d83944dbed76f";
    let mut bad2 = req;
    bad2.candidate_id = no_hyphens.to_owned();
    let err = plan_installation(
        &bad2,
        &candidates,
        &trust,
        &profiles,
        &conformance_store,
        &approvals,
        &installed,
    )
    .unwrap_err();
    assert_eq!(err.code, "installation_plan_request_invalid");

    fs::remove_dir_all(base).unwrap();
}
