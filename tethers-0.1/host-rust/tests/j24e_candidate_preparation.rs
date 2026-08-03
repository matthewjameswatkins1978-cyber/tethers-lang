use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use tethers_reference_host::candidate_preparation::{
    prepare_installation_candidate, CandidatePreparationDisposition,
};
use tethers_reference_host::pdf_tools;
use uuid::Uuid;

fn temp_dir(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("tethers-j24e-{}-{}", Uuid::new_v4(), name))
}

fn sha256(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn snapshot(root: &Path) -> BTreeMap<String, String> {
    let root_canonical = fs::canonicalize(root).unwrap();
    let mut digest_snapshot = BTreeMap::new();
    if !root_canonical.is_dir() {
        return digest_snapshot;
    }
    collect_snapshot(&root_canonical, &root_canonical, &mut digest_snapshot);
    digest_snapshot
}

fn collect_snapshot(base: &Path, dir: &Path, snapshot: &mut BTreeMap<String, String>) {
    let mut entries: Vec<_> = fs::read_dir(dir).unwrap().filter_map(|e| e.ok()).collect();
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let path = entry.path();
        let relative = path
            .strip_prefix(base)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        if path.is_file() {
            let bytes = fs::read(&path).unwrap();
            snapshot.insert(relative, sha256(&bytes));
        } else if path.is_dir() {
            snapshot.insert(format!("{relative}/"), String::new());
            collect_snapshot(base, &path, snapshot);
        }
    }
}

fn assert_no_lifecycle_paths(root: &Path) {
    for name in &[
        "install",
        "installed-records",
        "enablements",
        "trust",
        "conformance",
        "approvals",
    ] {
        let path = root.join(name);
        assert!(
            !path.exists(),
            "lifecycle path {name} must not exist after candidate preparation"
        );
    }
}

fn write_package(root: &Path, name: &str, provider_bytes: &[u8]) -> PathBuf {
    let archive = pdf_tools::build_reference_package(provider_bytes).unwrap();
    let package_path = root.join(name);
    fs::write(&package_path, archive).unwrap();
    package_path
}

fn write_package_default(root: &Path, provider_bytes: &[u8]) -> PathBuf {
    write_package(root, "pdf-tools.tetherplug", provider_bytes)
}

#[test]
fn valid_package_creates_candidate_with_disposition_created() {
    let root = temp_dir("valid");
    fs::create_dir_all(&root).unwrap();
    let package = write_package_default(&root, b"not-an-executable-provider");

    let result = prepare_installation_candidate(&root, &package).unwrap();
    assert_eq!(result.disposition, CandidatePreparationDisposition::Created);

    let candidate = &result.candidate;
    assert_eq!(candidate.state, "quarantined_installation_candidate");
    assert_eq!(candidate.package_id, "tethers.pdf-tools");
    assert_eq!(candidate.package_version, "1.0.0");
    assert_eq!(candidate.provider_id, "tethers-pdf-provider");
    assert_eq!(candidate.provider_version, "1.0.0");
    assert_eq!(candidate.capabilities.len(), 1);
    assert_eq!(candidate.capabilities[0].name, "pdf.inspect");
    assert_eq!(candidate.capabilities[0].version, 1);
    assert_eq!(candidate.selected_platform.os, "windows");
    assert_eq!(candidate.selected_platform.architecture, "x86_64");
    assert!(!candidate.inspection_evidence_digest.is_empty());
    assert!(!candidate.record_digest.is_empty());
    assert!(!candidate.candidate_id.is_empty());
    assert!(Uuid::parse_str(&candidate.candidate_id).is_ok());
    assert!(candidate.signature_files.is_empty());
    assert!(!candidate.signatures_present);

    let candidates_root = root.join("candidates");
    let quarantine_root = root.join("quarantine");
    assert!(candidates_root.is_dir());
    assert!(quarantine_root.is_dir());

    let record_path = candidates_root.join(format!("{}.json", candidate.candidate_id));
    assert!(record_path.is_file());

    assert!(quarantine_root
        .join(&candidate.quarantine_relative_path)
        .is_dir());

    assert_no_lifecycle_paths(&root);

    let snap = snapshot(&root);
    assert!(snap.contains_key(&format!("candidates/{}.json", candidate.candidate_id)));
    assert!(snap.contains_key(&format!(
        "quarantine/{}/plug.json",
        candidate.quarantine_relative_path
    )));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn exact_archive_replay_returns_existing_same_candidate_id_no_bytes_changed() {
    let root = temp_dir("replay");
    fs::create_dir_all(&root).unwrap();
    let package = write_package_default(&root, b"non-executable-replay-bytes");

    let first = prepare_installation_candidate(&root, &package).unwrap();
    assert_eq!(first.disposition, CandidatePreparationDisposition::Created);
    let first_id = first.candidate.candidate_id.clone();

    let snap_before = snapshot(&root);

    let second = prepare_installation_candidate(&root, &package).unwrap();
    assert_eq!(
        second.disposition,
        CandidatePreparationDisposition::Existing
    );
    assert_eq!(second.candidate.candidate_id, first_id);
    assert_eq!(second.candidate, first.candidate);

    let snap_after = snapshot(&root);
    assert_eq!(snap_before, snap_after, "exact replay must change no byte");

    assert_no_lifecycle_paths(&root);

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn different_provider_bytes_same_release_fails_semantic_conflict() {
    let root = temp_dir("sem-conflict");
    fs::create_dir_all(&root).unwrap();
    let package_a = write_package(&root, "pdf-tools-a.tetherplug", b"first-provider-bytes-XYZ");
    let package_b = write_package(
        &root,
        "pdf-tools-b.tetherplug",
        b"second-provider-bytes-ABC",
    );

    let first = prepare_installation_candidate(&root, &package_a).unwrap();
    assert_eq!(first.disposition, CandidatePreparationDisposition::Created);

    let snap_before = snapshot(&root);

    let err = prepare_installation_candidate(&root, &package_b).unwrap_err();
    assert_eq!(err.code, "semantic_conflict", "must fail before extraction");

    let snap_after = snapshot(&root);
    assert_eq!(
        snap_before, snap_after,
        "semantic conflict must change nothing"
    );

    assert_no_lifecycle_paths(&root);

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn malformed_package_fails_before_candidates_or_quarantine_exist() {
    let root = temp_dir("malformed");
    fs::create_dir_all(&root).unwrap();
    let bogus = root.join("not-a-zip.tetherplug");
    fs::write(&bogus, b"this is not a valid ZIP archive").unwrap();

    let err = prepare_installation_candidate(&root, &bogus).unwrap_err();
    assert!(
        matches!(err.code, "invalid_archive" | "archive_read"),
        "expected archive refusal, got {}",
        err.code
    );

    assert!(
        !root.join("candidates").exists(),
        "candidates/ must not be created for malformed package"
    );
    assert!(
        !root.join("quarantine").exists(),
        "quarantine/ must not be created for malformed package"
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn missing_host_root_fails_without_creating_it() {
    let root = temp_dir("missing-root");
    fs::create_dir_all(&root).unwrap();
    let subdir = root.join("subdir");
    let package = write_package_default(&root, b"test-bytes");

    let err = prepare_installation_candidate(&subdir, &package).unwrap_err();
    assert_eq!(err.code, "unsafe_destination");

    assert!(!subdir.exists(), "host root must never be created");

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn relative_host_root_fails_without_creating_it() {
    let root = temp_dir("relative-root");
    fs::create_dir_all(&root).unwrap();
    let package = write_package_default(&root, b"test-bytes");

    let relative = Path::new("relative-root");
    let err = prepare_installation_candidate(relative, &package).unwrap_err();
    assert_eq!(err.code, "unsafe_destination");

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn relative_package_path_fails() {
    let root = temp_dir("relative-pkg");
    fs::create_dir_all(&root).unwrap();
    let _package = write_package_default(&root, b"test-bytes");

    let relative = Path::new("pdf-tools.tetherplug");
    let err = prepare_installation_candidate(&root, relative).unwrap_err();
    assert_eq!(err.code, "invalid_archive");

    assert!(!root.join("candidates").exists());
    assert!(!root.join("quarantine").exists());

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn non_directory_host_root_fails() {
    let root = temp_dir("non-dir");
    fs::create_dir_all(&root).unwrap();
    let package = write_package_default(&root, b"test-bytes");
    let file_as_root = root.join("file-root.txt");
    fs::write(&file_as_root, b"not a directory").unwrap();

    let err = prepare_installation_candidate(&file_as_root, &package).unwrap_err();
    assert_eq!(err.code, "unsafe_destination");

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn missing_package_path_fails_before_creating_child_dirs() {
    let root = temp_dir("missing-pkg");
    fs::create_dir_all(&root).unwrap();
    let missing = root.join("nonexistent.tetherplug");

    let err = prepare_installation_candidate(&root, &missing).unwrap_err();
    assert_eq!(err.code, "archive_read");

    assert!(!root.join("candidates").exists());
    assert!(!root.join("quarantine").exists());

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn candidate_output_pins_pdf_evidence() {
    let root = temp_dir("evidence-pin");
    fs::create_dir_all(&root).unwrap();
    let package = write_package_default(&root, b"provider-payload-evidence-pin");

    let result = prepare_installation_candidate(&root, &package).unwrap();
    let candidate = &result.candidate;

    assert_eq!(
        candidate.capabilities[0].name, "pdf.inspect",
        "must pin pdf.inspect capability"
    );
    assert_eq!(candidate.capabilities[0].version, 1);
    assert_eq!(candidate.capabilities[0].operation, "pdf_inspect");
    assert_eq!(candidate.launch_path, "provider/pdf_tools_provider.exe");
    assert_eq!(
        candidate.launch_arguments,
        vec!["--query-root", "__TETHERS_PDF_QUERY_ROOT__"]
    );
    assert_eq!(candidate.provider_working_directory, "provider");
    assert_eq!(candidate.capability_operation_namespace, "pdf");
    assert_eq!(
        candidate.source_size_bytes,
        fs::read(&package).unwrap().len() as u64
    );
    assert_eq!(candidate.inspection_report_format_version, 1);

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn quarantine_files_are_immutable_read_only() {
    let root = temp_dir("immutable");
    fs::create_dir_all(&root).unwrap();
    let package = write_package_default(&root, b"make-it-immutable-test");

    let result = prepare_installation_candidate(&root, &package).unwrap();
    let candidate = &result.candidate;

    let quarantine_dir = root
        .join("quarantine")
        .join(&candidate.quarantine_relative_path);
    assert!(quarantine_dir.is_dir());

    for relative in &[
        "plug.json",
        "manifests/pdf-inspect-v1.json",
        "provider/pdf_tools_provider.exe",
    ] {
        let file = quarantine_dir.join(relative);
        assert!(
            fs::metadata(&file).unwrap().permissions().readonly(),
            "{relative} must be read-only in quarantine"
        );
    }

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn corrupt_existing_candidate_evidence_fails_closed() {
    let root = temp_dir("corrupt");
    fs::create_dir_all(&root).unwrap();
    let package = write_package_default(&root, b"before-tampering");

    let result = prepare_installation_candidate(&root, &package).unwrap();
    assert_eq!(result.disposition, CandidatePreparationDisposition::Created);

    let quarantine_dir = root
        .join("quarantine")
        .join(&result.candidate.quarantine_relative_path);
    let plug_json = quarantine_dir.join("plug.json");
    let mut perms = fs::metadata(&plug_json).unwrap().permissions();
    perms.set_readonly(false);
    fs::set_permissions(&plug_json, perms).unwrap();
    fs::write(&plug_json, b"corrupted content").unwrap();

    let snap_before = snapshot(&root);

    let err = prepare_installation_candidate(&root, &package).unwrap_err();
    assert_eq!(err.code, "record_invalid");

    let snap_after = snapshot(&root);
    assert_eq!(
        snap_before, snap_after,
        "corrupt evidence must change nothing"
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn no_install_or_enablement_paths_are_ever_created() {
    let root = temp_dir("no-lifecycle");
    fs::create_dir_all(&root).unwrap();
    let package = write_package_default(&root, b"no-lifecycle-bytes");

    let _result = prepare_installation_candidate(&root, &package).unwrap();

    assert_no_lifecycle_paths(&root);

    let snap = snapshot(&root);
    for (path, _) in &snap {
        assert!(
            !path.starts_with("install/")
                && !path.starts_with("installed-records/")
                && !path.starts_with("enablements/")
                && !path.starts_with("trust/")
                && !path.starts_with("conformance/")
                && !path.starts_with("approvals/"),
            "forbidden lifecycle path found: {path}"
        );
    }

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn second_valid_archive_with_different_package_can_coexist() {
    let root = temp_dir("coexist");
    fs::create_dir_all(&root).unwrap();
    let package_a = write_package(&root, "pdf-a.tetherplug", b"first-archive-bytes");

    let first = prepare_installation_candidate(&root, &package_a).unwrap();
    assert_eq!(first.disposition, CandidatePreparationDisposition::Created);

    let snap_before = snapshot(&root);

    let second = prepare_installation_candidate(&root, &package_a).unwrap();
    assert_eq!(
        second.disposition,
        CandidatePreparationDisposition::Existing
    );
    assert_eq!(second.candidate.candidate_id, first.candidate.candidate_id);

    let snap_after = snapshot(&root);
    assert_eq!(
        snap_before, snap_after,
        "exact replay from different-name file with same bytes"
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn host_root_with_file_where_candidates_should_go_fails_store_io() {
    let root = temp_dir("file-block");
    fs::create_dir_all(&root).unwrap();
    let file_block = root.join("candidates");
    fs::write(&file_block, b"blocking file").unwrap();
    let package = write_package_default(&root, b"blocked-by-file");

    let err = prepare_installation_candidate(&root, &package).unwrap_err();
    assert_eq!(err.code, "candidate_io");

    assert_no_lifecycle_paths(&root);

    fs::remove_dir_all(root).unwrap();
}
