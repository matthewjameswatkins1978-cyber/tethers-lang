#![cfg(windows)]

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use ed25519_dalek::pkcs8::EncodePublicKey;
use ed25519_dalek::{Signer, SigningKey};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::time::Duration;
use tethers_reference_host::candidate::CandidateRecord;
use tethers_reference_host::conformance::ConformanceEvidence;
use tethers_reference_host::conformance::{
    run_host_conformance, ConformanceDisposition, ConformanceEvidenceStore,
};
use tethers_reference_host::installed::{InstallationApprovalRecord, InstalledPlugRecord};
use tethers_reference_host::installed::{InstallationApprovalStore, InstalledPlugRegistry};
use tethers_reference_host::launch_profile::LaunchProfileEvidence;
use tethers_reference_host::launch_profile::{
    PreparedSupervisedLaunch, SUPERVISED_PROFILE_LIMITATION,
};
use tethers_reference_host::package::{CapabilityEvidence, PayloadEvidence, PlatformEvidence};
use tethers_reference_host::trust::{
    key_id_from_spki, signing_input, verify_candidate_signatures, verify_signature_envelope,
    DeveloperApprovalRecord, DeveloperApprovalStore, PackageTrustEvidence, PublisherKeyRecord,
    PublisherTrustState, PublisherTrustStore, SignatureEnvelope,
};
use uuid::Uuid;
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::Security::SECURITY_ATTRIBUTES;
use windows_sys::Win32::System::Threading::CreateEventW;

fn make_writable(path: &Path) {
    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_readonly(false);
    fs::set_permissions(path, permissions).unwrap();
}

fn sha(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn root(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("tethers-m3-{name}-{}", Uuid::new_v4()));
    fs::create_dir_all(&path).unwrap();
    path
}

fn write_read_only(path: &Path, bytes: &[u8]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, bytes).unwrap();
    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_readonly(true);
    fs::set_permissions(path, permissions).unwrap();
}

fn assert_process_gone(pid: u32) {
    let state = std::process::Command::new("pwsh")
        .args([
            "-NoProfile",
            "-Command",
            &format!("if (Get-Process -Id {pid} -ErrorAction SilentlyContinue) {{ exit 1 }}"),
        ])
        .status()
        .unwrap();
    assert!(
        state.success(),
        "startup descendant {pid} survived shutdown"
    );
}

fn fixture_manifest() -> Vec<u8> {
    fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../protocol/capability-manifests/fixture-ping.json"),
    )
    .unwrap()
}

fn candidate_fixture(base: &Path, mode: &str) -> (CandidateRecord, PathBuf) {
    candidate_fixture_with_extra_arguments(base, mode, Vec::new())
}

fn candidate_fixture_with_extra_arguments(
    base: &Path,
    mode: &str,
    extra_arguments: Vec<String>,
) -> (CandidateRecord, PathBuf) {
    let quarantine_root = base.join("quarantine");
    let candidate_relative = format!("candidate-{}", Uuid::new_v4());
    let candidate_dir = quarantine_root.join(&candidate_relative);
    fs::create_dir_all(candidate_dir.join("provider")).unwrap();
    fs::create_dir_all(candidate_dir.join("manifests")).unwrap();

    let executable = fs::read(env!("CARGO_BIN_EXE_m3_fixture_provider")).unwrap();
    let manifest = fixture_manifest();
    let manifest_digest = "sha256:01fed7a4b877dd82abe91a1b6cfcd476b02e4c115489e70cbb285b8bf2d32d8b";
    let mut arguments = vec![
        "--mode".to_string(),
        mode.to_string(),
        "--ordered".to_string(),
        "first".to_string(),
        "second".to_string(),
    ];
    arguments.extend(extra_arguments);
    let plug_value = serde_json::json!({
        "package_format_version":"1",
        "package_id":"tethers.fixture",
        "package_version":"0.1.0",
        "display_name":"M3 Fixture",
        "description":"Native credential-free conformance fixture",
        "publisher":"Untrusted package presentation",
        "licence":"MIT",
        "socket_major":1,
        "protocol_bindings":[{"protocol":"MCP","version":"2025-11-25","transport":"stdio"}],
        "platforms":[{"os":"windows","architecture":"x86_64"}],
        "provider":{
            "provider_id":"tethers-stdio-fixture",
            "provider_version":"0.1.0",
            "launch":{"path":"provider/m3_fixture_provider.exe","arguments":arguments},
            "working_directory":"provider",
            "capability_operation_namespace":"fixture"
        },
        "capabilities":[{
            "capability_name":"fixture.ping",
            "capability_version":1,
            "manifest_path":"manifests/fixture-ping.json",
            "manifest_digest":manifest_digest,
            "provider_operation_name":"fixture_ping"
        }],
        "payload_index":[
            {"path":"manifests/fixture-ping.json","sha256":sha(&manifest),"size_bytes":manifest.len(),"role":"capability_manifest"},
            {"path":"provider/m3_fixture_provider.exe","sha256":sha(&executable),"size_bytes":executable.len(),"role":"provider_executable"}
        ]
    });
    let plug_json = serde_json_canonicalizer::to_vec(&plug_value).unwrap();
    let semantic_digest = sha(&plug_json);
    write_read_only(&candidate_dir.join("plug.json"), &plug_json);
    write_read_only(
        &candidate_dir.join("provider/m3_fixture_provider.exe"),
        &executable,
    );
    write_read_only(
        &candidate_dir.join("manifests/fixture-ping.json"),
        &manifest,
    );

    let mut record = CandidateRecord {
        schema_version: 1,
        candidate_id: Uuid::new_v4().to_string(),
        state: "quarantined_installation_candidate".into(),
        package_id: "tethers.fixture".into(),
        package_version: "0.1.0".into(),
        semantic_package_digest: semantic_digest,
        raw_archive_digest: sha(b"fixture archive identity"),
        source_size_bytes: 1234,
        quarantine_relative_path: candidate_relative,
        provider_id: "tethers-stdio-fixture".into(),
        provider_version: "0.1.0".into(),
        launch_path: "provider/m3_fixture_provider.exe".into(),
        launch_arguments: arguments,
        provider_working_directory: "provider".into(),
        capability_operation_namespace: "fixture".into(),
        selected_platform: PlatformEvidence {
            os: "windows".into(),
            architecture: "x86_64".into(),
        },
        plug_json: PayloadEvidence {
            path: "plug.json".into(),
            sha256: sha(&plug_json),
            size_bytes: plug_json.len() as u64,
            role: "package_descriptor".into(),
        },
        payloads: vec![
            PayloadEvidence {
                path: "manifests/fixture-ping.json".into(),
                sha256: sha(&manifest),
                size_bytes: manifest.len() as u64,
                role: "capability_manifest".into(),
            },
            PayloadEvidence {
                path: "provider/m3_fixture_provider.exe".into(),
                sha256: sha(&executable),
                size_bytes: executable.len() as u64,
                role: "provider_executable".into(),
            },
        ],
        signature_files: vec![],
        capabilities: vec![CapabilityEvidence {
            name: "fixture.ping".into(),
            version: 1,
            operation: "fixture_ping".into(),
            manifest_path: "manifests/fixture-ping.json".into(),
            manifest_digest: manifest_digest.into(),
        }],
        signatures_present: false,
        inspection_report_format_version: 1,
        inspection_evidence_digest: sha(b"inspection evidence"),
        created_unix_ms: 1,
        record_digest: String::new(),
    };
    refresh_candidate_digest(&mut record);
    (record, quarantine_root)
}

fn refresh_candidate_digest(record: &mut CandidateRecord) {
    let mut covered = record.clone();
    covered.record_digest.clear();
    record.record_digest = sha(&serde_json_canonicalizer::to_vec(&covered).unwrap());
}

#[test]
fn m3_candidate_to_installed_disabled_is_explicit_and_non_operational() {
    let base = root("lifecycle");
    let (candidate, quarantine_root) = candidate_fixture(&base, "valid");
    std::env::set_var("TETHERS_TEST_AMBIENT_SECRET", "must-not-reach-child");

    let trust_store = PublisherTrustStore::open(&base.join("trust")).unwrap();
    let developer_store = DeveloperApprovalStore::open(&base.join("developer-approvals")).unwrap();
    let developer = developer_store
        .approve_exact_digest(&candidate.semantic_package_digest, "Matthew")
        .unwrap();
    let trust = PackageTrustEvidence::unsigned(&developer).unwrap();
    let prepared = PreparedSupervisedLaunch::prepare(
        &candidate,
        &quarantine_root,
        &base.join("scratch"),
        Duration::from_secs(5),
    )
    .unwrap();
    assert_eq!(prepared.evidence.profile_label, "supervised");
    assert!(!prepared.evidence.isolated);
    assert_eq!(prepared.evidence.limitation, SUPERVISED_PROFILE_LIMITATION);
    assert!(!prepared
        .evidence
        .environment_names
        .iter()
        .any(|name| name == "PATH"));
    assert!(!prepared
        .evidence
        .environment_names
        .iter()
        .any(|name| name.contains("SECRET")));
    let conformance = run_host_conformance(
        &prepared,
        &candidate,
        &quarantine_root,
        &trust,
        &trust_store,
        &developer_store,
        "tethers-reference-host@0.2.0+m3",
    )
    .unwrap();
    assert_eq!(
        conformance.disposition,
        ConformanceDisposition::Passed,
        "cases: {:?}",
        conformance.cases
    );
    assert_eq!(conformance.cases.len(), 8);
    assert_eq!(conformance.retry_count, 0);
    assert!(!conformance.raw_stderr_persisted);

    let evidence_store = ConformanceEvidenceStore::open(&base.join("conformance")).unwrap();
    evidence_store.create(&conformance).unwrap();
    assert_eq!(
        evidence_store.load_all().unwrap(),
        vec![conformance.clone()]
    );
    assert_eq!(
        conformance
            .require_current(&candidate, &trust, &prepared.evidence, "sha256:stale-suite")
            .unwrap_err()
            .code,
        "conformance_stale"
    );
    let serialized_conformance = serde_json::to_string(&conformance).unwrap();
    assert!(!serialized_conformance.contains("M3_SECRET_CANARY"));
    assert!(!serialized_conformance.contains("must-not-reach-child"));
    fs::write(base.join("conformance/.torn.tmp"), b"{}").unwrap();
    assert_eq!(
        evidence_store.load_all().unwrap_err().code,
        "conformance_invalid"
    );
    fs::remove_file(base.join("conformance/.torn.tmp")).unwrap();
    assert!(
        !base.join("install").exists(),
        "conformance pass cannot install"
    );

    let approval_store =
        InstallationApprovalStore::open(&base.join("installation-approvals")).unwrap();
    let approval = approval_store
        .approve(
            &candidate,
            &quarantine_root,
            &trust,
            &trust_store,
            &developer_store,
            &prepared.evidence,
            &conformance,
            "Matthew",
        )
        .unwrap();
    let registry =
        InstalledPlugRegistry::open(&base.join("install"), &base.join("installed-records"))
            .unwrap();
    let installed = registry
        .install_disabled(
            &candidate,
            &quarantine_root,
            &trust,
            &trust_store,
            &developer_store,
            &prepared.evidence,
            &conformance,
            &approval,
        )
        .unwrap();
    assert_eq!(installed.state, "present_disabled");
    assert_eq!(installed.active_binding_count(), 0);
    assert!(installed
        .disabled_bindings
        .iter()
        .all(|binding| binding.state == "disabled"));
    assert_eq!(registry.load_all().unwrap(), vec![installed]);

    let loaded = registry.load_all().unwrap().remove(0);
    assert_eq!(
        registry
            .install_disabled(
                &candidate,
                &quarantine_root,
                &trust,
                &trust_store,
                &developer_store,
                &prepared.evidence,
                &conformance,
                &approval,
            )
            .unwrap_err()
            .code,
        "installed_conflict"
    );
    fs::write(base.join("installed-records/.torn.tmp"), b"{}").unwrap();
    assert_eq!(
        registry.load_all().unwrap_err().code,
        "installed_record_invalid"
    );
    fs::remove_file(base.join("installed-records/.torn.tmp")).unwrap();
    let record_path = base
        .join("installed-records")
        .join(format!("{}.json", loaded.installed_id));
    let mismatched_path = base.join("installed-records/00000000-0000-0000-0000-000000000000.json");
    fs::copy(&record_path, &mismatched_path).unwrap();
    assert_eq!(
        registry.load_all().unwrap_err().code,
        "installed_record_invalid"
    );
    fs::remove_file(mismatched_path).unwrap();
    let installed_manifest = base
        .join("install")
        .join(&loaded.installation_relative_path)
        .join("manifests/fixture-ping.json");
    let original_manifest = fs::read(&installed_manifest).unwrap();
    make_writable(&installed_manifest);
    fs::write(&installed_manifest, b"mutated").unwrap();
    assert_eq!(
        registry.load_all().unwrap_err().code,
        "installed_record_invalid"
    );
    write_read_only(&installed_manifest, &original_manifest);
    let unexpected = base
        .join("install")
        .join(&loaded.installation_relative_path)
        .join("provider/unexpected.exe");
    write_read_only(&unexpected, b"unexpected");
    assert_eq!(
        registry.load_all().unwrap_err().code,
        "installed_record_invalid"
    );
    make_writable(&unexpected);
    fs::remove_file(unexpected).unwrap();

    prepared.cleanup_scratch().unwrap();
    std::env::remove_var("TETHERS_TEST_AMBIENT_SECRET");
    fs::remove_dir_all(base).unwrap();
}

#[test]
fn m3_trust_launch_and_conformance_evidence_cannot_cross_candidates() {
    let base = root("cross-candidate-evidence");
    let (candidate_a, quarantine_a) = candidate_fixture(&base.join("a"), "valid");
    let (candidate_b, quarantine_b) = candidate_fixture(&base.join("b"), "valid-b");
    assert_eq!(candidate_a.package_id, candidate_b.package_id);
    assert_ne!(
        candidate_a.semantic_package_digest,
        candidate_b.semantic_package_digest
    );

    let developers = DeveloperApprovalStore::open(&base.join("developer-approvals")).unwrap();
    let trust_a = PackageTrustEvidence::unsigned(
        &developers
            .approve_exact_digest(&candidate_a.semantic_package_digest, "Matthew")
            .unwrap(),
    )
    .unwrap();
    let trust_b = PackageTrustEvidence::unsigned(
        &developers
            .approve_exact_digest(&candidate_b.semantic_package_digest, "Matthew")
            .unwrap(),
    )
    .unwrap();
    let publishers = PublisherTrustStore::open(&base.join("publisher-trust")).unwrap();
    let launch_a = PreparedSupervisedLaunch::prepare(
        &candidate_a,
        &quarantine_a,
        &base.join("scratch-a"),
        Duration::from_secs(2),
    )
    .unwrap();
    let launch_b = PreparedSupervisedLaunch::prepare(
        &candidate_b,
        &quarantine_b,
        &base.join("scratch-b"),
        Duration::from_secs(2),
    )
    .unwrap();

    assert_eq!(
        trust_a
            .require_for_candidate(&candidate_b)
            .unwrap_err()
            .code,
        "trust_candidate_mismatch"
    );
    assert_eq!(
        launch_a
            .evidence
            .require_for_candidate(&candidate_b)
            .unwrap_err()
            .code,
        "launch_candidate_mismatch"
    );
    assert_eq!(
        run_host_conformance(
            &launch_b,
            &candidate_b,
            &quarantine_b,
            &trust_a,
            &publishers,
            &developers,
            "host-build",
        )
        .unwrap_err()
        .code,
        "trust_candidate_mismatch"
    );
    assert_eq!(
        run_host_conformance(
            &launch_a,
            &candidate_b,
            &quarantine_b,
            &trust_b,
            &publishers,
            &developers,
            "host-build",
        )
        .unwrap_err()
        .code,
        "launch_candidate_mismatch"
    );

    let conformance_a = run_host_conformance(
        &launch_a,
        &candidate_a,
        &quarantine_a,
        &trust_a,
        &publishers,
        &developers,
        "host-build",
    )
    .unwrap();
    let conformance_b = run_host_conformance(
        &launch_b,
        &candidate_b,
        &quarantine_b,
        &trust_b,
        &publishers,
        &developers,
        "host-build",
    )
    .unwrap();
    let approvals = InstallationApprovalStore::open(&base.join("approvals")).unwrap();
    assert_eq!(
        approvals
            .approve(
                &candidate_b,
                &quarantine_b,
                &trust_b,
                &publishers,
                &developers,
                &launch_a.evidence,
                &conformance_a,
                "Matthew",
            )
            .unwrap_err()
            .code,
        "launch_candidate_mismatch"
    );
    assert_eq!(
        approvals
            .approve(
                &candidate_b,
                &quarantine_b,
                &trust_b,
                &publishers,
                &developers,
                &launch_b.evidence,
                &conformance_a,
                "Matthew",
            )
            .unwrap_err()
            .code,
        "conformance_stale"
    );
    let approval_b = approvals
        .approve(
            &candidate_b,
            &quarantine_b,
            &trust_b,
            &publishers,
            &developers,
            &launch_b.evidence,
            &conformance_b,
            "Matthew",
        )
        .unwrap();
    let installed =
        InstalledPlugRegistry::open(&base.join("install"), &base.join("records")).unwrap();
    assert_eq!(
        installed
            .install_disabled(
                &candidate_b,
                &quarantine_b,
                &trust_a,
                &publishers,
                &developers,
                &launch_b.evidence,
                &conformance_b,
                &approval_b,
            )
            .unwrap_err()
            .code,
        "trust_candidate_mismatch"
    );
    assert_eq!(
        installed
            .install_disabled(
                &candidate_b,
                &quarantine_b,
                &trust_b,
                &publishers,
                &developers,
                &launch_a.evidence,
                &conformance_b,
                &approval_b,
            )
            .unwrap_err()
            .code,
        "launch_candidate_mismatch"
    );
    launch_a.cleanup_scratch().unwrap();
    launch_b.cleanup_scratch().unwrap();
    fs::remove_dir_all(base).unwrap();
}

#[test]
fn m3_immediate_startup_descendant_is_contained_by_suspended_job_assignment() {
    let base = root("startup-descendant");
    let (candidate, quarantine) = candidate_fixture(&base, "spawn-child");
    let developers = DeveloperApprovalStore::open(&base.join("developer-approvals")).unwrap();
    let publishers = PublisherTrustStore::open(&base.join("publisher-trust")).unwrap();
    let trust = PackageTrustEvidence::unsigned(
        &developers
            .approve_exact_digest(&candidate.semantic_package_digest, "Matthew")
            .unwrap(),
    )
    .unwrap();
    let prepared = PreparedSupervisedLaunch::prepare(
        &candidate,
        &quarantine,
        &base.join("scratch"),
        Duration::from_secs(3),
    )
    .unwrap();
    let conformance = run_host_conformance(
        &prepared,
        &candidate,
        &quarantine,
        &trust,
        &publishers,
        &developers,
        "host-build",
    )
    .unwrap();
    assert_eq!(conformance.disposition, ConformanceDisposition::Passed);
    let pid = fs::read_to_string(prepared.scratch_directory().join("m3-startup-child.pid"))
        .unwrap()
        .trim()
        .parse::<u32>()
        .unwrap();
    assert_process_gone(pid);
    prepared.cleanup_scratch().unwrap();

    let (failing_candidate, failing_quarantine) =
        candidate_fixture(&base.join("failure"), "spawn-child-malformed");
    let failing_trust = PackageTrustEvidence::unsigned(
        &developers
            .approve_exact_digest(&failing_candidate.semantic_package_digest, "Matthew")
            .unwrap(),
    )
    .unwrap();
    let failing = PreparedSupervisedLaunch::prepare(
        &failing_candidate,
        &failing_quarantine,
        &base.join("scratch-failure"),
        Duration::from_secs(3),
    )
    .unwrap();
    let evidence = run_host_conformance(
        &failing,
        &failing_candidate,
        &failing_quarantine,
        &failing_trust,
        &publishers,
        &developers,
        "host-build",
    )
    .unwrap();
    assert_eq!(evidence.disposition, ConformanceDisposition::Failed);
    let failed_pid = fs::read_to_string(failing.scratch_directory().join("m3-startup-child.pid"))
        .unwrap()
        .trim()
        .parse::<u32>()
        .unwrap();
    assert_process_gone(failed_pid);
    failing.cleanup_scratch().unwrap();
    fs::remove_dir_all(base).unwrap();
}

#[test]
fn m3_current_signed_trust_is_revalidated_before_conformance_process_creation() {
    let base = root("revoked-before-conformance");
    let marker = base.join("provider-created.marker");
    let (candidate, quarantine_root) = candidate_fixture_with_extra_arguments(
        &base,
        "valid",
        vec![
            "--provider-marker".into(),
            marker.to_string_lossy().into_owned(),
        ],
    );
    let signing_key = SigningKey::from_bytes(&[29u8; 32]);
    let der = signing_key
        .verifying_key()
        .to_public_key_der()
        .unwrap()
        .as_bytes()
        .to_vec();
    let envelope = SignatureEnvelope {
        signature_format_version: "1".into(),
        algorithm: "ed25519".into(),
        key_id: key_id_from_spki(&der).unwrap(),
        semantic_package_digest: candidate.semantic_package_digest.clone(),
        signature: URL_SAFE_NO_PAD.encode(
            signing_key
                .sign(&signing_input(&candidate.semantic_package_digest))
                .to_bytes(),
        ),
    };
    let verified = verify_signature_envelope(
        &serde_json::to_vec(&envelope).unwrap(),
        &candidate.semantic_package_digest,
        &der,
    )
    .unwrap();
    let publishers = PublisherTrustStore::open(&base.join("publisher-trust")).unwrap();
    let trusted = publishers
        .append(
            &der,
            "publisher:host-owned",
            Some("tethers.".into()),
            PublisherTrustState::Trusted,
            "Matthew",
            None,
            None,
        )
        .unwrap();
    let trust = PackageTrustEvidence::signed(&verified, &trusted).unwrap();
    let developers = DeveloperApprovalStore::open(&base.join("developer-approvals")).unwrap();
    let prepared = PreparedSupervisedLaunch::prepare(
        &candidate,
        &quarantine_root,
        &base.join("scratch"),
        Duration::from_secs(2),
    )
    .unwrap();

    publishers
        .append(
            &der,
            "publisher:host-owned",
            Some("tethers.".into()),
            PublisherTrustState::Revoked,
            "Matthew",
            None,
            Some("revoked before conformance".into()),
        )
        .unwrap();
    assert_eq!(
        run_host_conformance(
            &prepared,
            &candidate,
            &quarantine_root,
            &trust,
            &publishers,
            &developers,
            "host-build",
        )
        .unwrap_err()
        .code,
        "trust_not_current"
    );
    assert!(
        !marker.exists(),
        "revoked trust must refuse before provider process or protocol traffic"
    );
    prepared.cleanup_scratch().unwrap();
    fs::remove_dir_all(base).unwrap();
}

#[test]
fn m3_removed_or_corrupt_developer_approval_refuses_before_conformance_process_creation() {
    for corruption in ["removed", "corrupt"] {
        let base = root(&format!("developer-approval-{corruption}"));
        let marker = base.join("provider-created.marker");
        let (candidate, quarantine_root) = candidate_fixture_with_extra_arguments(
            &base,
            "valid",
            vec![
                "--provider-marker".into(),
                marker.to_string_lossy().into_owned(),
            ],
        );
        let publishers = PublisherTrustStore::open(&base.join("publisher-trust")).unwrap();
        let developers = DeveloperApprovalStore::open(&base.join("developer-approvals")).unwrap();
        let approval = developers
            .approve_exact_digest(&candidate.semantic_package_digest, "Matthew")
            .unwrap();
        let trust = PackageTrustEvidence::unsigned(&approval).unwrap();
        let prepared = PreparedSupervisedLaunch::prepare(
            &candidate,
            &quarantine_root,
            &base.join("scratch"),
            Duration::from_secs(2),
        )
        .unwrap();
        let approval_path = fs::read_dir(base.join("developer-approvals"))
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        if corruption == "removed" {
            fs::remove_file(&approval_path).unwrap();
        } else {
            make_writable(&approval_path);
            fs::write(&approval_path, b"corrupt developer approval").unwrap();
        }
        assert!(
            run_host_conformance(
                &prepared,
                &candidate,
                &quarantine_root,
                &trust,
                &publishers,
                &developers,
                "host-build",
            )
            .is_err(),
            "{corruption} approval must refuse"
        );
        assert!(
            !marker.exists(),
            "{corruption} approval must refuse before provider process or protocol traffic"
        );
        prepared.cleanup_scratch().unwrap();
        fs::remove_dir_all(base).unwrap();
    }
}

#[test]
fn m3_windows_handle_allow_list_excludes_unrelated_inheritable_handle() {
    let base = root("handle-allow-list");
    let security = SECURITY_ATTRIBUTES {
        nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: std::ptr::null_mut(),
        bInheritHandle: 1,
    };
    // SAFETY: the test owns and closes this unnamed event handle after the
    // fixture has attempted to inspect it inside the supervised child.
    let canary = unsafe { CreateEventW(&security, 1, 0, std::ptr::null()) };
    assert!(!canary.is_null(), "test canary event must be created");
    let (candidate, quarantine_root) = candidate_fixture_with_extra_arguments(
        &base,
        "valid",
        vec![
            "--unrelated-inheritable-handle".into(),
            (canary as isize).to_string(),
        ],
    );
    let publishers = PublisherTrustStore::open(&base.join("publisher-trust")).unwrap();
    let developers = DeveloperApprovalStore::open(&base.join("developer-approvals")).unwrap();
    let trust = PackageTrustEvidence::unsigned(
        &developers
            .approve_exact_digest(&candidate.semantic_package_digest, "Matthew")
            .unwrap(),
    )
    .unwrap();
    let prepared = PreparedSupervisedLaunch::prepare(
        &candidate,
        &quarantine_root,
        &base.join("scratch"),
        Duration::from_secs(2),
    )
    .unwrap();
    let conformance = run_host_conformance(
        &prepared,
        &candidate,
        &quarantine_root,
        &trust,
        &publishers,
        &developers,
        "host-build",
    )
    .unwrap();
    assert_eq!(conformance.disposition, ConformanceDisposition::Passed);
    // SAFETY: this process still owns the canary event and no child endpoint
    // aliases it after the CreateProcessW allow-list launch has returned.
    unsafe { CloseHandle(canary) };
    prepared.cleanup_scratch().unwrap();
    fs::remove_dir_all(base).unwrap();
}

#[test]
fn m3_malformed_and_interrupted_conformance_fail_without_retry_or_install() {
    let base = root("conformance-refusal");
    let developer_store = DeveloperApprovalStore::open(&base.join("developer-approvals")).unwrap();
    let publisher_store = PublisherTrustStore::open(&base.join("publisher-trust")).unwrap();

    let (malformed, malformed_root) = candidate_fixture(&base.join("malformed"), "malformed");
    let approval = developer_store
        .approve_exact_digest(&malformed.semantic_package_digest, "Matthew")
        .unwrap();
    let trust = PackageTrustEvidence::unsigned(&approval).unwrap();
    let prepared = PreparedSupervisedLaunch::prepare(
        &malformed,
        &malformed_root,
        &base.join("scratch-malformed"),
        Duration::from_secs(2),
    )
    .unwrap();
    let evidence = run_host_conformance(
        &prepared,
        &malformed,
        &malformed_root,
        &trust,
        &publisher_store,
        &developer_store,
        "host-build",
    )
    .unwrap();
    assert_eq!(evidence.disposition, ConformanceDisposition::Failed);
    assert_eq!(evidence.retry_count, 0);
    prepared.cleanup_scratch().unwrap();

    for (name, mode, limit, expected_code) in [
        (
            "wrong-schema",
            "wrong-schema",
            Duration::from_secs(2),
            "catalogue_drift",
        ),
        (
            "wrong-output",
            "wrong-output",
            Duration::from_secs(2),
            "fixture_valid_call",
        ),
        (
            "oversized",
            "oversized",
            Duration::from_secs(2),
            "conformance_protocol",
        ),
        (
            "timeout",
            "hang",
            Duration::from_millis(100),
            "conformance_protocol",
        ),
    ] {
        let (candidate, quarantine) = candidate_fixture(&base.join(name), mode);
        let approval = developer_store
            .approve_exact_digest(&candidate.semantic_package_digest, "Matthew")
            .unwrap();
        let trust = PackageTrustEvidence::unsigned(&approval).unwrap();
        let prepared = PreparedSupervisedLaunch::prepare(
            &candidate,
            &quarantine,
            &base.join(format!("scratch-{name}")),
            limit,
        )
        .unwrap();
        let evidence = run_host_conformance(
            &prepared,
            &candidate,
            &quarantine,
            &trust,
            &publisher_store,
            &developer_store,
            "host-build",
        )
        .unwrap();
        assert_eq!(evidence.disposition, ConformanceDisposition::Failed);
        assert_eq!(evidence.retry_count, 0);
        assert_eq!(
            evidence
                .cases
                .iter()
                .find(|case| case.case_id == "conformance_session")
                .unwrap()
                .safe_diagnostic_code
                .as_deref(),
            Some(expected_code),
            "mode {mode} returned unexpected typed evidence: {:?}",
            evidence.cases
        );
        prepared.cleanup_scratch().unwrap();
    }

    let (paginated, paginated_root) = candidate_fixture(&base.join("paginated"), "paginated");
    let paginated_approval = developer_store
        .approve_exact_digest(&paginated.semantic_package_digest, "Matthew")
        .unwrap();
    let paginated_trust = PackageTrustEvidence::unsigned(&paginated_approval).unwrap();
    let prepared = PreparedSupervisedLaunch::prepare(
        &paginated,
        &paginated_root,
        &base.join("scratch-paginated"),
        Duration::from_secs(2),
    )
    .unwrap();
    let evidence = run_host_conformance(
        &prepared,
        &paginated,
        &paginated_root,
        &paginated_trust,
        &publisher_store,
        &developer_store,
        "host-build",
    )
    .unwrap();
    assert_eq!(evidence.disposition, ConformanceDisposition::Passed);
    prepared.cleanup_scratch().unwrap();

    let (interrupted, interrupted_root) = candidate_fixture(&base.join("interrupted"), "valid");
    let interrupted_approval = developer_store
        .approve_exact_digest(&interrupted.semantic_package_digest, "Matthew")
        .unwrap();
    let interrupted_trust = PackageTrustEvidence::unsigned(&interrupted_approval).unwrap();
    let prepared = PreparedSupervisedLaunch::prepare(
        &interrupted,
        &interrupted_root,
        &base.join("scratch-interrupted"),
        Duration::from_secs(2),
    )
    .unwrap();
    tethers_reference_host::child_process::set_interrupted();
    let evidence = run_host_conformance(
        &prepared,
        &interrupted,
        &interrupted_root,
        &interrupted_trust,
        &publisher_store,
        &developer_store,
        "host-build",
    )
    .unwrap();
    tethers_reference_host::child_process::INTERRUPTED.store(false, Ordering::Release);
    assert_eq!(evidence.disposition, ConformanceDisposition::Interrupted);
    assert_eq!(evidence.retry_count, 0);
    prepared.cleanup_scratch().unwrap();
    assert!(!base.join("install").exists());
    fs::remove_dir_all(base).unwrap();
}

#[test]
fn m3_payload_and_file_set_drift_are_refused_before_launch() {
    let base = root("drift");
    let (candidate, quarantine_root) = candidate_fixture(&base, "valid");
    let directory = quarantine_root.join(&candidate.quarantine_relative_path);
    write_read_only(
        &directory.join("provider/unexpected.exe"),
        b"hostile addition",
    );
    let error = match PreparedSupervisedLaunch::prepare(
        &candidate,
        &quarantine_root,
        &base.join("scratch"),
        Duration::from_secs(1),
    ) {
        Ok(_) => panic!("unexpected file must be refused before launch"),
        Err(error) => error,
    };
    assert_eq!(error.code, "candidate_drift");
    fs::remove_dir_all(base).unwrap();
}

#[test]
fn m3_missing_payload_is_refused_before_launch() {
    let base = root("missing-payload");
    let (candidate, quarantine_root) = candidate_fixture(&base, "valid");
    let executable = quarantine_root
        .join(&candidate.quarantine_relative_path)
        .join(&candidate.launch_path);
    make_writable(&executable);
    fs::remove_file(executable).unwrap();
    let error = match PreparedSupervisedLaunch::prepare(
        &candidate,
        &quarantine_root,
        &base.join("scratch"),
        Duration::from_secs(1),
    ) {
        Ok(_) => panic!("missing launch payload must be refused"),
        Err(error) => error,
    };
    assert_eq!(error.code, "candidate_drift");
    fs::remove_dir_all(base).unwrap();
}

#[test]
fn m3_real_windows_junction_roots_are_refused_without_outside_write() {
    let base = root("junction-root");
    let outside = base.join("outside");
    fs::create_dir_all(&outside).unwrap();
    let junction = base.join("hostile-root");
    let status = std::process::Command::new("cmd")
        .args([
            "/c",
            "mklink",
            "/J",
            junction.to_str().unwrap(),
            outside.to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(status.success());
    let trust_error = match PublisherTrustStore::open(&junction) {
        Ok(_) => panic!("junction trust root must be refused"),
        Err(error) => error,
    };
    assert_eq!(trust_error.code, "unsafe_store_path");
    let install_error = match InstalledPlugRegistry::open(&junction, &base.join("records")) {
        Ok(_) => panic!("junction installation root must be refused"),
        Err(error) => error,
    };
    assert_eq!(install_error.code, "unsafe_store_path");
    assert_eq!(fs::read_dir(&outside).unwrap().count(), 0);
    fs::remove_dir(&junction).unwrap();
    fs::remove_dir_all(base).unwrap();
}

#[test]
fn m3_trust_revocation_after_approval_refuses_publication() {
    let base = root("revocation");
    let (candidate, quarantine_root) = candidate_fixture(&base, "valid");
    let signing_key = SigningKey::from_bytes(&[11u8; 32]);
    let der = signing_key
        .verifying_key()
        .to_public_key_der()
        .unwrap()
        .as_bytes()
        .to_vec();
    let signature = signing_key.sign(&signing_input(&candidate.semantic_package_digest));
    let envelope = SignatureEnvelope {
        signature_format_version: "1".into(),
        algorithm: "ed25519".into(),
        key_id: key_id_from_spki(&der).unwrap(),
        semantic_package_digest: candidate.semantic_package_digest.clone(),
        signature: URL_SAFE_NO_PAD.encode(signature.to_bytes()),
    };
    let verified = verify_signature_envelope(
        &serde_json::to_vec(&envelope).unwrap(),
        &candidate.semantic_package_digest,
        &der,
    )
    .unwrap();
    let trust_store = PublisherTrustStore::open(&base.join("trust")).unwrap();
    let publisher = trust_store
        .append(
            &der,
            "publisher:host-owned",
            Some("tethers.".into()),
            PublisherTrustState::Trusted,
            "Matthew",
            None,
            None,
        )
        .unwrap();
    let developer_store = DeveloperApprovalStore::open(&base.join("developer-approvals")).unwrap();
    let trust = PackageTrustEvidence::signed(&verified, &publisher).unwrap();
    let prepared = PreparedSupervisedLaunch::prepare(
        &candidate,
        &quarantine_root,
        &base.join("scratch"),
        Duration::from_secs(5),
    )
    .unwrap();
    let conformance = run_host_conformance(
        &prepared,
        &candidate,
        &quarantine_root,
        &trust,
        &trust_store,
        &developer_store,
        "host-build",
    )
    .unwrap();
    assert_eq!(conformance.disposition, ConformanceDisposition::Passed);
    let approval_store =
        InstallationApprovalStore::open(&base.join("installation-approvals")).unwrap();
    let approval = approval_store
        .approve(
            &candidate,
            &quarantine_root,
            &trust,
            &trust_store,
            &developer_store,
            &prepared.evidence,
            &conformance,
            "Matthew",
        )
        .unwrap();
    trust_store
        .append(
            &der,
            "publisher:host-owned",
            Some("tethers.".into()),
            PublisherTrustState::Revoked,
            "Matthew",
            None,
            Some("revoked before publication".into()),
        )
        .unwrap();
    let registry =
        InstalledPlugRegistry::open(&base.join("install"), &base.join("installed-records"))
            .unwrap();
    let error = registry
        .install_disabled(
            &candidate,
            &quarantine_root,
            &trust,
            &trust_store,
            &developer_store,
            &prepared.evidence,
            &conformance,
            &approval,
        )
        .unwrap_err();
    assert_eq!(error.code, "trust_not_current");
    assert_eq!(fs::read_dir(base.join("install")).unwrap().count(), 0);
    prepared.cleanup_scratch().unwrap();
    fs::remove_dir_all(base).unwrap();
}

#[test]
fn m3_detached_signature_filename_duplicate_and_no_package_trust_mutation() {
    let base = root("detached-signature");
    let (mut candidate, quarantine_root) = candidate_fixture(&base, "valid");
    let signing_key = SigningKey::from_bytes(&[13u8; 32]);
    let der = signing_key
        .verifying_key()
        .to_public_key_der()
        .unwrap()
        .as_bytes()
        .to_vec();
    let key_id = key_id_from_spki(&der).unwrap();
    let envelope = SignatureEnvelope {
        signature_format_version: "1".into(),
        algorithm: "ed25519".into(),
        key_id: key_id.clone(),
        semantic_package_digest: candidate.semantic_package_digest.clone(),
        signature: URL_SAFE_NO_PAD.encode(
            signing_key
                .sign(&signing_input(&candidate.semantic_package_digest))
                .to_bytes(),
        ),
    };
    let bytes = serde_json::to_vec(&envelope).unwrap();
    let path = format!(
        "signatures/ed25519-{}.json",
        key_id.strip_prefix("sha256:").unwrap()
    );
    write_read_only(
        &quarantine_root
            .join(&candidate.quarantine_relative_path)
            .join(&path),
        &bytes,
    );
    let signature_evidence = PayloadEvidence {
        path,
        sha256: sha(&bytes),
        size_bytes: bytes.len() as u64,
        role: "signature_evidence".into(),
    };
    candidate.signature_files = vec![signature_evidence.clone()];
    candidate.signatures_present = true;
    refresh_candidate_digest(&mut candidate);
    let trust_store = PublisherTrustStore::open(&base.join("trust")).unwrap();
    trust_store
        .append(
            &der,
            "publisher:host-owned",
            Some("tethers.".into()),
            PublisherTrustState::Trusted,
            "Matthew",
            None,
            None,
        )
        .unwrap();
    let trust_files_before = fs::read_dir(base.join("trust")).unwrap().count();
    let verified =
        verify_candidate_signatures(&candidate, &quarantine_root, &trust_store, 0).unwrap();
    assert_eq!(verified.len(), 1);
    assert_eq!(
        fs::read_dir(base.join("trust")).unwrap().count(),
        trust_files_before
    );

    candidate.signature_files.push(signature_evidence);
    refresh_candidate_digest(&mut candidate);
    assert_eq!(
        verify_candidate_signatures(&candidate, &quarantine_root, &trust_store, 0)
            .unwrap_err()
            .code,
        "signature_duplicate"
    );
    candidate.signature_files[0].path = "signatures/ED25519-invalid.json".into();
    candidate.signature_files.truncate(1);
    refresh_candidate_digest(&mut candidate);
    assert_eq!(
        verify_candidate_signatures(&candidate, &quarantine_root, &trust_store, 0)
            .unwrap_err()
            .code,
        "signature_filename"
    );
    fs::remove_dir_all(base).unwrap();
}

#[test]
fn m3_golden_schemas_are_committed_and_strictly_typed() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../fixtures/m3/m3-schema-golden-v1.json")).unwrap();
    serde_json::from_value::<PublisherKeyRecord>(fixture["publisher_key_record"].clone()).unwrap();
    serde_json::from_value::<DeveloperApprovalRecord>(fixture["developer_approval_record"].clone())
        .unwrap();
    serde_json::from_value::<LaunchProfileEvidence>(fixture["launch_profile_evidence"].clone())
        .unwrap();
    serde_json::from_value::<ConformanceEvidence>(fixture["conformance_evidence"].clone()).unwrap();
    serde_json::from_value::<InstallationApprovalRecord>(
        fixture["installation_approval_record"].clone(),
    )
    .unwrap();
    serde_json::from_value::<InstalledPlugRecord>(fixture["installed_plug_record"].clone())
        .unwrap();

    let mut unknown = fixture["installed_plug_record"].clone();
    unknown["enabled"] = serde_json::Value::Bool(true);
    assert!(serde_json::from_value::<InstalledPlugRecord>(unknown).is_err());
}
