use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use tethers_reference_host::candidate::CandidateRegistry;
use tethers_reference_host::conformance::ConformanceEvidenceStore;
use tethers_reference_host::installed::InstallationApprovalStore;
use tethers_reference_host::launch_profile::{
    LaunchProfileEvidence, LaunchProfileEvidenceStore, SUPERVISED_PROFILE_LABEL,
    SUPERVISED_PROFILE_LIMITATION,
};
use tethers_reference_host::trust::{DeveloperApprovalStore, PublisherTrustStore};
use uuid::Uuid;

fn sha256(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn temp_dir(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("tethers-j24h-{name}-{}", Uuid::new_v4()))
}

fn snapshot(root: &Path) -> BTreeMap<String, String> {
    fn visit(root: &Path, path: &Path, output: &mut BTreeMap<String, String>) {
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
            } else if metadata.file_type().is_symlink() {
                output.insert(relative, "<symlink>".into());
            }
        }
    }

    let mut output = BTreeMap::new();
    if root.is_dir() {
        visit(root, root, &mut output);
    }
    output
}

fn valid_launch_profile_evidence(candidate_id: &str) -> LaunchProfileEvidence {
    let mut evidence = LaunchProfileEvidence {
        profile_format_version: 1,
        profile_label: SUPERVISED_PROFILE_LABEL.into(),
        isolated: false,
        limitation: SUPERVISED_PROFILE_LIMITATION.into(),
        candidate_id: candidate_id.to_owned(),
        semantic_package_digest: format!("sha256:{}", "a".repeat(64)),
        executable_digest: format!("sha256:{}", "b".repeat(64)),
        executable_relative_path: "provider/test.exe".into(),
        arguments: vec!["--test".into()],
        working_directory_relative_path: "provider".into(),
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
    evidence
}

fn digest_suffix(evidence: &LaunchProfileEvidence) -> &str {
    &evidence.profile_evidence_digest[7..]
}

// --- CandidateRegistry::open_existing ---

#[test]
fn candidate_open_existing_accepts_two_existing_safe_directories() {
    let root = temp_dir("candidate-accept");
    let candidates = root.join("candidates");
    let quarantine = root.join("quarantine");
    fs::create_dir_all(&candidates).unwrap();
    fs::create_dir_all(&quarantine).unwrap();
    let before = snapshot(&root);

    let registry = match CandidateRegistry::open_existing(&candidates, &quarantine) {
        Ok(r) => r,
        Err(e) => panic!("open_existing failed: {}: {}", e.code, e.message),
    };
    let records = registry.load_all().unwrap();
    assert!(records.is_empty());
    assert_eq!(before, snapshot(&root));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn candidate_open_existing_rejects_missing_roots_without_creation() {
    let root = temp_dir("candidate-missing");
    fs::create_dir_all(&root).unwrap();
    let missing_candidates = root.join("missing-candidates");
    let missing_quarantine = root.join("missing-quarantine");
    fs::write(root.join("sentinel"), b"unchanged").unwrap();
    let before = snapshot(&root);

    assert!(CandidateRegistry::open_existing(&missing_candidates, &missing_quarantine).is_err());
    assert!(!missing_candidates.exists());
    assert!(!missing_quarantine.exists());
    assert_eq!(before, snapshot(&root));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn candidate_open_existing_rejects_non_directory_roots() {
    let root = temp_dir("candidate-nondir");
    fs::create_dir_all(&root).unwrap();
    let file_root = root.join("file-as-root");
    fs::write(&file_root, b"not a directory").unwrap();
    let quarantine = root.join("quarantine");
    fs::create_dir_all(&quarantine).unwrap();
    let before = snapshot(&root);

    match CandidateRegistry::open_existing(&file_root, &quarantine) {
        Ok(_) => panic!("non-directory root was accepted"),
        Err(ref e) => assert_eq!(e.code, "registry_invalid"),
    }
    assert_eq!(before, snapshot(&root));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn candidate_open_existing_rejects_identical_lexical_paths() {
    let root = temp_dir("candidate-identical");
    fs::create_dir_all(&root).unwrap();
    let before = snapshot(&root);

    assert!(CandidateRegistry::open_existing(&root, &root).is_err());
    assert_eq!(before, snapshot(&root));
    fs::remove_dir_all(root).unwrap();
}

#[cfg(windows)]
#[test]
fn candidate_junction_roots_fail_closed_without_child_creation() {
    let root = temp_dir("candidate-junction");
    let target = root.join("target");
    let junction = root.join("junction");
    let candidates = root.join("candidates");
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(&target).unwrap();
    fs::create_dir(&candidates).unwrap();
    let status = std::process::Command::new("cmd")
        .args([
            "/C",
            "mklink",
            "/J",
            junction.to_str().unwrap(),
            target.to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(status.success());
    let before = snapshot(&root);

    match CandidateRegistry::open_existing(&candidates, &junction) {
        Ok(_) => panic!("junction root was accepted"),
        Err(ref e) => assert_eq!(e.code, "unsafe_destination"),
    }
    assert!(!target.join("any-child").exists());
    assert_eq!(before, snapshot(&root));
    fs::remove_dir(&junction).unwrap();
    fs::remove_dir_all(root).unwrap();
}

// --- M3 store open_existing ---

#[test]
fn every_m3_store_open_existing_accepts_existing_empty_root() {
    let root = temp_dir("m3-existing");
    fs::create_dir_all(&root).unwrap();

    let trust_dir = root.join("trust");
    let approval_dir = root.join("approvals");
    let conformance_dir = root.join("conformance");
    let install_approval_dir = root.join("install-approvals");
    fs::create_dir(&trust_dir).unwrap();
    fs::create_dir(&approval_dir).unwrap();
    fs::create_dir(&conformance_dir).unwrap();
    fs::create_dir(&install_approval_dir).unwrap();

    let before = snapshot(&root);

    let _trust = PublisherTrustStore::open_existing(&trust_dir).unwrap();
    let _approval = DeveloperApprovalStore::open_existing(&approval_dir).unwrap();
    let _conformance = ConformanceEvidenceStore::open_existing(&conformance_dir).unwrap();
    let _install_approval =
        InstallationApprovalStore::open_existing(&install_approval_dir).unwrap();

    assert_eq!(before, snapshot(&root));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn every_m3_store_open_existing_rejects_missing_root() {
    let root = temp_dir("m3-missing");
    fs::create_dir_all(&root).unwrap();
    let missing = root.join("not-here");
    let before = snapshot(&root);

    assert!(PublisherTrustStore::open_existing(&missing).is_err());
    assert!(DeveloperApprovalStore::open_existing(&missing).is_err());
    assert!(ConformanceEvidenceStore::open_existing(&missing).is_err());
    assert!(InstallationApprovalStore::open_existing(&missing).is_err());
    assert!(!missing.exists());
    assert_eq!(before, snapshot(&root));
    fs::remove_dir_all(root).unwrap();
}

// --- LaunchProfileEvidenceStore ---

#[test]
fn launch_profile_round_trip_is_exact() {
    let root = temp_dir("lp-roundtrip");
    fs::create_dir_all(&root).unwrap();
    let store = LaunchProfileEvidenceStore::open(&root).unwrap();
    let evidence = valid_launch_profile_evidence("3d846d40-01fc-4e1e-b77d-83944dbed76f");
    store.create(&evidence).unwrap();
    let loaded = store.load_all().unwrap();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0], evidence);

    let filename = format!("{}.json", digest_suffix(&evidence));
    assert!(root.join(&filename).is_file());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn launch_profile_filename_is_digest_suffix_no_uuid_or_timestamp() {
    let root = temp_dir("lp-filename");
    fs::create_dir_all(&root).unwrap();
    let store = LaunchProfileEvidenceStore::open(&root).unwrap();
    let evidence = valid_launch_profile_evidence("3d846d40-01fc-4e1e-b77d-83944dbed76f");
    store.create(&evidence).unwrap();

    let expected = format!("{}.json", digest_suffix(&evidence));
    assert!(root.join(&expected).is_file());

    let name = root
        .join(&expected)
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let stem = name.strip_suffix(".json").unwrap();
    assert_eq!(stem.len(), 64);
    assert!(stem
        .chars()
        .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn launch_profile_open_existing_and_load_all_change_no_byte() {
    let root = temp_dir("lp-nomutation");
    fs::create_dir_all(&root).unwrap();
    let store = LaunchProfileEvidenceStore::open(&root).unwrap();
    let evidence = valid_launch_profile_evidence("3d846d40-01fc-4e1e-b77d-83944dbed76f");
    store.create(&evidence).unwrap();
    let before = snapshot(&root);

    let existing = LaunchProfileEvidenceStore::open_existing(&root).unwrap();
    let loaded = existing.load_all().unwrap();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0], evidence);
    assert_eq!(before, snapshot(&root));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn launch_profile_duplicate_create_returns_record_conflict_and_no_mutation() {
    let root = temp_dir("lp-duplicate");
    fs::create_dir_all(&root).unwrap();
    let store = LaunchProfileEvidenceStore::open(&root).unwrap();
    let evidence = valid_launch_profile_evidence("3d846d40-01fc-4e1e-b77d-83944dbed76f");
    store.create(&evidence).unwrap();
    let before = snapshot(&root);

    let err = store.create(&evidence).unwrap_err();
    assert_eq!(err.code, "record_conflict");
    assert_eq!(before, snapshot(&root));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn launch_profile_torn_tmp_rejected() {
    let root = temp_dir("lp-torn");
    fs::create_dir_all(&root).unwrap();
    let store = LaunchProfileEvidenceStore::open(&root).unwrap();
    fs::write(root.join(".something.tmp"), b"partial").unwrap();
    let before = snapshot(&root);

    let err = store.load_all().unwrap_err();
    assert_eq!(err.code, "launch_profile_store_invalid");
    assert_eq!(err.message, "torn launch-profile evidence");
    assert_eq!(before, snapshot(&root));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn launch_profile_non_json_entry_rejected() {
    let root = temp_dir("lp-nonjson");
    fs::create_dir_all(&root).unwrap();
    let store = LaunchProfileEvidenceStore::open(&root).unwrap();
    fs::write(root.join("something.txt"), b"not json").unwrap();
    let before = snapshot(&root);

    let err = store.load_all().unwrap_err();
    assert_eq!(err.code, "launch_profile_store_invalid");
    assert_eq!(err.message, "unexpected launch-profile store entry");
    assert_eq!(before, snapshot(&root));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn launch_profile_filename_mismatch_rejected() {
    let root = temp_dir("lp-mismatch");
    fs::create_dir_all(&root).unwrap();
    let store = LaunchProfileEvidenceStore::open(&root).unwrap();
    let evidence = valid_launch_profile_evidence("3d846d40-01fc-4e1e-b77d-83944dbed76f");
    let good_suffix = digest_suffix(&evidence);
    store.create(&evidence).unwrap();

    let wrong = root.join(format!("{}.json", "0".repeat(64)));
    fs::rename(root.join(format!("{good_suffix}.json")), &wrong).unwrap();
    let before = snapshot(&root);

    let err = store.load_all().unwrap_err();
    assert_eq!(err.code, "launch_profile_store_invalid");
    assert_eq!(err.message, "launch-profile filename mismatch");
    assert_eq!(before, snapshot(&root));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn launch_profile_filename_mismatch_copied_evidence_rejected() {
    let root = temp_dir("lp-copied");
    fs::create_dir_all(&root).unwrap();
    let store = LaunchProfileEvidenceStore::open(&root).unwrap();
    let evidence = valid_launch_profile_evidence("3d846d40-01fc-4e1e-b77d-83944dbed76f");
    let suffix = digest_suffix(&evidence);
    store.create(&evidence).unwrap();

    let bytes = fs::read(root.join(format!("{suffix}.json"))).unwrap();
    fs::write(root.join(format!("{}.json", "f".repeat(64))), &bytes).unwrap();
    let before = snapshot(&root);

    let err = store.load_all().unwrap_err();
    assert_eq!(err.code, "launch_profile_store_invalid");
    assert_eq!(err.message, "launch-profile filename mismatch");
    assert_eq!(before, snapshot(&root));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn launch_profile_malformed_evidence_rejected() {
    let root = temp_dir("lp-malformed");
    fs::create_dir_all(&root).unwrap();
    let store = LaunchProfileEvidenceStore::open(&root).unwrap();
    fs::write(
        root.join(format!("{}.json", "0".repeat(64))),
        b"not valid evidence",
    )
    .unwrap();
    let before = snapshot(&root);

    assert!(store.load_all().is_err());
    assert_eq!(before, snapshot(&root));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn launch_profile_missing_root_stays_missing_after_open_existing_failure() {
    let root = temp_dir("lp-missing-root");
    fs::create_dir_all(&root).unwrap();
    let missing = root.join("not-here");
    let before = snapshot(&root);

    assert!(LaunchProfileEvidenceStore::open_existing(&missing).is_err());
    assert!(!missing.exists());
    assert_eq!(before, snapshot(&root));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn launch_profile_multiple_records_sorted_by_digest() {
    let root = temp_dir("lp-multi");
    fs::create_dir_all(&root).unwrap();
    let store = LaunchProfileEvidenceStore::open(&root).unwrap();

    let evidence_a = valid_launch_profile_evidence("aaaaaaaa-1111-4aaa-aaaa-aaaaaaaaaaaa");
    let evidence_b = valid_launch_profile_evidence("bbbbbbbb-2222-4bbb-bbbb-bbbbbbbbbbbb");

    store.create(&evidence_a).unwrap();
    store.create(&evidence_b).unwrap();

    let loaded = store.load_all().unwrap();
    assert_eq!(loaded.len(), 2);
    assert!(loaded[0].profile_evidence_digest <= loaded[1].profile_evidence_digest);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn candidate_open_existing_and_load_all_preserves_recursive_snapshot() {
    let root = temp_dir("candidate-snapshot");
    let candidates = root.join("candidates");
    let quarantine = root.join("quarantine");
    fs::create_dir_all(&candidates).unwrap();
    fs::create_dir_all(&quarantine).unwrap();
    fs::write(root.join("sentinel"), b"preserved").unwrap();
    let before = snapshot(&root);

    let registry = match CandidateRegistry::open_existing(&candidates, &quarantine) {
        Ok(r) => r,
        Err(e) => panic!("open_existing failed: {}: {}", e.code, e.message),
    };
    let records = registry.load_all().unwrap();
    assert!(records.is_empty());
    assert_eq!(before, snapshot(&root));
    fs::remove_dir_all(root).unwrap();
}
