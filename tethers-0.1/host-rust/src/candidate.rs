//! Host-owned quarantine and immutable M2 installation candidates.
//!
//! These records are deliberately not installed-Plug records: they carry no
//! trust, approval, binding, credential, session, policy, or launch authority.

use crate::manifest;
use crate::package::{self, InspectionReport, PackageError, PayloadEvidence, PlatformEvidence};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;
#[cfg(windows)]
use windows_sys::Win32::Storage::FileSystem::{
    GetFileAttributesW, SetFileAttributesW, FILE_ATTRIBUTE_READONLY, FILE_ATTRIBUTE_REPARSE_POINT,
    INVALID_FILE_ATTRIBUTES,
};

fn err(code: &'static str, text: impl Into<String>) -> PackageError {
    PackageError {
        code,
        message: text.into(),
    }
}
fn sha(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}
fn io<T>(result: std::io::Result<T>) -> Result<T, PackageError> {
    result.map_err(|e| err("candidate_io", e.to_string()))
}

#[cfg(windows)]
fn wide(path: &Path) -> Vec<u16> {
    path.as_os_str().encode_wide().chain(Some(0)).collect()
}
fn reject_reparse_or_link(path: &Path) -> Result<(), PackageError> {
    let metadata = io(fs::symlink_metadata(path))?;
    if metadata.file_type().is_symlink() {
        Err(err(
            "unsafe_destination",
            "symbolic-link destinations are refused",
        ))
    } else {
        #[cfg(windows)]
        {
            let path_w = wide(path);
            // SAFETY: the nul-terminated UTF-16 path remains live for this
            // attribute query; no handle or pointer escapes the call.
            let attributes = unsafe { GetFileAttributesW(path_w.as_ptr()) };
            if attributes == INVALID_FILE_ATTRIBUTES
                || attributes & FILE_ATTRIBUTE_REPARSE_POINT != 0
            {
                return Err(err(
                    "unsafe_destination",
                    "Windows reparse destinations are refused",
                ));
            }
        }
        Ok(())
    }
}
/// Check every existing component before and after directory creation. On
/// Windows this examines FILE_ATTRIBUTE_REPARSE_POINT, covering junctions,
/// mount points, and other reparse forms that Path::is_symlink cannot see.
fn verify_existing_chain(path: &Path) -> Result<(), PackageError> {
    if !path.is_absolute() {
        return Err(err("unsafe_destination", "host roots must be absolute"));
    }
    for component in path.ancestors() {
        match fs::symlink_metadata(component) {
            Ok(_) => reject_reparse_or_link(component)?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(err("candidate_io", error.to_string())),
        }
    }
    Ok(())
}
fn create_safe_dir_all(path: &Path) -> Result<(), PackageError> {
    verify_existing_chain(path)?;
    io(fs::create_dir_all(path))?;
    verify_existing_chain(path)
}
fn confined(root: &Path, child: &Path) -> Result<PathBuf, PackageError> {
    verify_existing_chain(root)?;
    verify_existing_chain(child)?;
    let root = io(fs::canonicalize(root))?;
    let child = io(fs::canonicalize(child))?;
    if child.starts_with(&root) {
        Ok(child)
    } else {
        Err(err(
            "unsafe_destination",
            "candidate escaped configured root",
        ))
    }
}
fn create_dir(path: &Path) -> Result<(), PackageError> {
    match fs::create_dir(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            Err(err("already_exists", "candidate target exists"))
        }
        Err(e) => Err(err("candidate_io", e.to_string())),
    }
}
fn write_new(path: &Path, bytes: &[u8]) -> Result<(), PackageError> {
    let mut out = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| err("candidate_io", e.to_string()))?;
    io(out.write_all(bytes))?;
    io(out.sync_all())
}
fn mark_read_only(path: &Path) -> Result<(), PackageError> {
    let mut permissions = io(fs::metadata(path))?.permissions();
    permissions.set_readonly(true);
    io(fs::set_permissions(path, permissions))?;
    #[cfg(windows)]
    {
        let path_w = wide(path);
        // SAFETY: the nul-terminated UTF-16 path remains live for the call.
        let attributes = unsafe { GetFileAttributesW(path_w.as_ptr()) };
        if attributes == INVALID_FILE_ATTRIBUTES {
            return Err(err("candidate_io", "could not read payload attributes"));
        }
        // SAFETY: the path remains live and only the restrictive READONLY bit
        // is added to the existing host-visible attributes.
        if unsafe { SetFileAttributesW(path_w.as_ptr(), attributes | FILE_ATTRIBUTE_READONLY) } == 0
        {
            return Err(err("candidate_io", "could not make payload read-only"));
        }
    }
    if !io(fs::metadata(path))?.permissions().readonly() {
        return Err(err("candidate_io", "payload remained writable"));
    }
    Ok(())
}
fn expected_files(
    plug_json: &PayloadEvidence,
    payloads: &[PayloadEvidence],
    signature_files: &[PayloadEvidence],
) -> Result<BTreeMap<String, PayloadEvidence>, PackageError> {
    let mut expected = BTreeMap::new();
    for evidence in std::iter::once(plug_json)
        .chain(payloads.iter())
        .chain(signature_files.iter())
    {
        if expected
            .insert(evidence.path.clone(), evidence.clone())
            .is_some()
        {
            return Err(err("record_invalid", "duplicate quarantine file evidence"));
        }
    }
    Ok(expected)
}
fn collect_quarantine_files(
    root: &Path,
    directory: &Path,
) -> Result<BTreeSet<String>, PackageError> {
    let mut files = BTreeSet::new();
    for entry in io(fs::read_dir(directory))? {
        let entry = io(entry)?;
        let path = entry.path();
        reject_reparse_or_link(&path)?;
        let file_type = io(entry.file_type())?;
        if file_type.is_dir() {
            files.extend(collect_quarantine_files(root, &path)?);
        } else if file_type.is_file() {
            let relative = path
                .strip_prefix(root)
                .map_err(|_| err("record_invalid", "quarantine path escaped root"))?
                .to_string_lossy()
                .replace('\\', "/");
            files.insert(relative);
        } else {
            return Err(err("record_invalid", "non-file quarantine entry"));
        }
    }
    Ok(files)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantinedPackage {
    pub directory: PathBuf,
    pub report: InspectionReport,
}

/// Explicit P5 continuation. Re-inspection and byte verification are repeated
/// before publication; no archive-library extraction helper is ever used.
pub fn extract_to_quarantine(
    report: &InspectionReport,
    quarantine_root: &Path,
) -> Result<QuarantinedPackage, PackageError> {
    create_safe_dir_all(quarantine_root)?;
    let root = io(fs::canonicalize(quarantine_root))?;
    verify_existing_chain(&root)?;
    let renewed = package::inspect(report.archive_path())?;
    if renewed.package != report.package || renewed.raw_archive_digest != report.raw_archive_digest
    {
        return Err(err("inspection_stale", "archive changed after inspection"));
    }
    let id = Uuid::new_v4().to_string();
    let staging = root.join(format!(".staging-{id}"));
    let final_dir = root.join(format!("candidate-{id}"));
    create_dir(&staging)?;
    confined(&root, &staging)?;
    let result = (|| -> Result<(), PackageError> {
        let source =
            File::open(report.archive_path()).map_err(|e| err("archive_read", e.to_string()))?;
        let mut archive =
            zip::ZipArchive::new(source).map_err(|e| err("archive_read", e.to_string()))?;
        let expected = expected_files(
            &renewed.plug_json,
            &renewed.payloads,
            &renewed.signature_files,
        )?;
        let mut written = BTreeSet::new();
        for index in 0..archive.len() {
            let entry = archive
                .by_index(index)
                .map_err(|e| err("archive_read", e.to_string()))?;
            let name = std::str::from_utf8(entry.name_raw())
                .map_err(|_| err("archive_read", "non UTF-8 entry after inspection"))?
                .to_owned();
            let evidence = expected
                .get(&name)
                .ok_or_else(|| err("unsafe_destination", "archive changed after inspection"))?;
            if !written.insert(name.clone()) {
                return Err(err(
                    "unsafe_destination",
                    "duplicate archive entry after inspection",
                ));
            }
            let relative = Path::new(&name);
            if relative.components().count() == 0
                || relative.is_absolute()
                || name.contains('\\')
                || name.contains(':')
                || name
                    .split('/')
                    .any(|part| part.is_empty() || part == "." || part == "..")
            {
                return Err(err("unsafe_destination", "invalid destination path"));
            }
            let target = staging.join(relative);
            if !target.starts_with(&staging) {
                return Err(err("unsafe_destination", "destination escape"));
            }
            if let Some(parent) = target.parent() {
                let mut cursor = staging.clone();
                for part in parent
                    .strip_prefix(&staging)
                    .map_err(|_| err("unsafe_destination", "parent escape"))?
                    .components()
                {
                    cursor.push(part);
                    if !cursor.exists() {
                        create_dir(&cursor)?;
                    }
                    verify_existing_chain(&cursor)?;
                }
            }
            let mut bytes = Vec::new();
            entry
                .take(evidence.size_bytes + 1)
                .read_to_end(&mut bytes)
                .map_err(|e| err("archive_read", e.to_string()))?;
            if bytes.len() as u64 != evidence.size_bytes || sha(&bytes) != evidence.sha256 {
                return Err(err(
                    "payload_mismatch",
                    "second payload verification failed",
                ));
            }
            write_new(&target, &bytes)?;
        }
        for (key, _) in &expected {
            let file = staging.join(key);
            if !file.is_file() {
                return Err(err(
                    "payload_mismatch",
                    "accepted archive entry was not extracted",
                ));
            }
            mark_read_only(&file)?;
        }
        Ok(())
    })();
    if let Err(failure) = result {
        let _ = fs::remove_dir_all(&staging);
        return Err(failure);
    }
    verify_existing_chain(&root)?;
    verify_existing_chain(
        staging
            .parent()
            .ok_or_else(|| err("unsafe_destination", "staging parent missing"))?,
    )?;
    if final_dir.exists() {
        return Err(err("already_exists", "candidate target exists"));
    }
    io(fs::rename(&staging, &final_dir))?;
    let directory = confined(&root, &final_dir)?;
    Ok(QuarantinedPackage {
        directory,
        report: renewed,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CandidateRecord {
    pub schema_version: u32,
    pub candidate_id: String,
    pub state: String,
    pub package_id: String,
    pub package_version: String,
    pub semantic_package_digest: String,
    pub raw_archive_digest: String,
    pub source_size_bytes: u64,
    pub quarantine_relative_path: String,
    pub provider_id: String,
    pub provider_version: String,
    pub launch_path: String,
    pub launch_arguments: Vec<String>,
    pub provider_working_directory: String,
    pub capability_operation_namespace: String,
    pub selected_platform: PlatformEvidence,
    pub plug_json: PayloadEvidence,
    pub payloads: Vec<PayloadEvidence>,
    pub signature_files: Vec<PayloadEvidence>,
    pub capabilities: Vec<crate::package::CapabilityEvidence>,
    pub signatures_present: bool,
    pub inspection_report_format_version: u32,
    pub inspection_evidence_digest: String,
    pub created_unix_ms: u64,
    pub record_digest: String,
}
impl CandidateRecord {
    fn covered_bytes(&self) -> Result<Vec<u8>, PackageError> {
        let mut copy = self.clone();
        copy.record_digest.clear();
        serde_json_canonicalizer::to_vec(&copy).map_err(|e| err("record_invalid", e.to_string()))
    }
    fn validate(&self) -> Result<(), PackageError> {
        if self.schema_version != 1
            || self.state != "quarantined_installation_candidate"
            || self.candidate_id.is_empty()
            || Uuid::parse_str(&self.candidate_id).is_err()
            || self.selected_platform.os != "windows"
            || self.selected_platform.architecture != "x86_64"
            || self.inspection_report_format_version != 1
            || self.inspection_evidence_digest.len() != 71
            || self.signatures_present != !self.signature_files.is_empty()
            || self.plug_json.path != "plug.json"
            || self.plug_json.role != "package_descriptor"
            || self.record_digest != sha(&self.covered_bytes()?)
        {
            Err(err("record_invalid", "invalid immutable candidate record"))
        } else {
            Ok(())
        }
    }
}

pub struct CandidateRegistry {
    root: PathBuf,
    quarantine_root: PathBuf,
}
impl CandidateRegistry {
    pub fn open(root: &Path, quarantine_root: &Path) -> Result<Self, PackageError> {
        if root == quarantine_root {
            return Err(err(
                "registry_invalid",
                "registry and quarantine roots must differ",
            ));
        }
        create_safe_dir_all(root)?;
        create_safe_dir_all(quarantine_root)?;
        let root = io(fs::canonicalize(root))?;
        let quarantine_root = io(fs::canonicalize(quarantine_root))?;
        verify_existing_chain(&root)?;
        verify_existing_chain(&quarantine_root)?;
        if root == quarantine_root {
            return Err(err(
                "registry_invalid",
                "registry and quarantine roots resolve to the same location",
            ));
        }
        Ok(Self {
            root,
            quarantine_root,
        })
    }
    fn verify_roots(&self) -> Result<(), PackageError> {
        verify_existing_chain(&self.root)?;
        verify_existing_chain(&self.quarantine_root)
    }
    pub fn create(
        &self,
        quarantined: &QuarantinedPackage,
    ) -> Result<CandidateRecord, PackageError> {
        self.verify_roots()?;
        for existing in self.load_all()? {
            if existing.package_id == quarantined.report.package.package_id
                && existing.package_version == quarantined.report.package.package_version
                && existing.semantic_package_digest != quarantined.report.package.semantic_digest
            {
                return Err(err(
                    "semantic_conflict",
                    "same package release has different semantic evidence",
                ));
            }
        }
        let directory = confined(&self.quarantine_root, &quarantined.directory)?;
        let relative = directory
            .strip_prefix(&self.quarantine_root)
            .map_err(|_| err("registry_invalid", "quarantine location"))?
            .to_string_lossy()
            .replace('\\', "/");
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| err("clock", "clock before epoch"))?
            .as_millis() as u64;
        let mut record = CandidateRecord {
            schema_version: 1,
            candidate_id: Uuid::new_v4().to_string(),
            state: "quarantined_installation_candidate".into(),
            package_id: quarantined.report.package.package_id.clone(),
            package_version: quarantined.report.package.package_version.clone(),
            semantic_package_digest: quarantined.report.package.semantic_digest.clone(),
            raw_archive_digest: quarantined.report.raw_archive_digest.clone(),
            source_size_bytes: quarantined.report.raw_archive_size,
            quarantine_relative_path: relative,
            provider_id: quarantined.report.provider_id.clone(),
            provider_version: quarantined.report.provider_version.clone(),
            launch_path: quarantined.report.provider_launch_path.clone(),
            launch_arguments: quarantined.report.provider_launch_arguments.clone(),
            provider_working_directory: quarantined.report.provider_working_directory.clone(),
            capability_operation_namespace: quarantined.report.provider_operation_namespace.clone(),
            selected_platform: quarantined.report.selected_platform.clone(),
            plug_json: quarantined.report.plug_json.clone(),
            payloads: quarantined.report.payloads.clone(),
            signature_files: quarantined.report.signature_files.clone(),
            capabilities: quarantined.report.capabilities.clone(),
            signatures_present: quarantined.report.signatures_present,
            inspection_report_format_version: quarantined.report.inspection_format_version,
            inspection_evidence_digest: quarantined.report.inspection_evidence_digest.clone(),
            created_unix_ms: now,
            record_digest: String::new(),
        };
        record.record_digest = sha(&record.covered_bytes()?);
        let destination = self.root.join(format!("{}.json", record.candidate_id));
        if destination.exists() {
            return Err(err("already_exists", "candidate record exists"));
        }
        let temporary = self.root.join(format!(".{}.tmp", record.candidate_id));
        write_new(
            &temporary,
            &serde_json_canonicalizer::to_vec(&record)
                .map_err(|e| err("record_invalid", e.to_string()))?,
        )?;
        self.verify_roots()?;
        io(fs::rename(&temporary, &destination))?;
        Ok(record)
    }
    pub fn load_all(&self) -> Result<Vec<CandidateRecord>, PackageError> {
        self.verify_roots()?;
        let mut records = Vec::new();
        let mut identities = HashSet::new();
        let mut releases = BTreeMap::new();
        for entry in io(fs::read_dir(&self.root))? {
            let entry = io(entry)?;
            let path = entry.path();
            reject_reparse_or_link(&path)?;
            if path.extension().and_then(|x| x.to_str()) == Some("tmp") {
                return Err(err("record_invalid", "torn temporary record present"));
            }
            if path.extension().and_then(|x| x.to_str()) != Some("json") {
                return Err(err("record_invalid", "unexpected registry entry"));
            }
            let text = io(fs::read_to_string(&path))?;
            let value = manifest::parse_value_no_dupes(&text)
                .map_err(|e| err("record_invalid", e.to_string()))?;
            let record: CandidateRecord =
                serde_json::from_value(value).map_err(|e| err("record_invalid", e.to_string()))?;
            record.validate()?;
            if path.file_stem().and_then(|stem| stem.to_str()) != Some(record.candidate_id.as_str())
            {
                return Err(err(
                    "record_invalid",
                    "record filename and candidate ID differ",
                ));
            }
            if !identities.insert(record.candidate_id.clone()) {
                return Err(err("record_invalid", "duplicate candidate identity"));
            }
            let release = (record.package_id.clone(), record.package_version.clone());
            if let Some(previous) = releases.insert(release, record.semantic_package_digest.clone())
            {
                if previous != record.semantic_package_digest {
                    return Err(err(
                        "semantic_conflict",
                        "conflicting semantic release evidence",
                    ));
                }
            }
            let payload = confined(
                &self.quarantine_root,
                &self.quarantine_root.join(&record.quarantine_relative_path),
            )?;
            if !payload.is_dir() {
                return Err(err("record_invalid", "quarantine payload missing"));
            }
            verify_existing_chain(&payload)?;
            let expected =
                expected_files(&record.plug_json, &record.payloads, &record.signature_files)?;
            let actual = collect_quarantine_files(&payload, &payload)?;
            if actual != expected.keys().cloned().collect::<BTreeSet<_>>() {
                return Err(err(
                    "record_invalid",
                    "unexpected or missing quarantine file",
                ));
            }
            for (relative, evidence) in expected {
                let file = payload.join(&relative);
                reject_reparse_or_link(&file)?;
                let metadata = io(fs::metadata(&file))?;
                if !metadata.is_file() || !metadata.permissions().readonly() {
                    return Err(err("record_invalid", "quarantine payload is not immutable"));
                }
                let bytes = io(fs::read(&file))?;
                if bytes.len() as u64 != evidence.size_bytes || sha(&bytes) != evidence.sha256 {
                    return Err(err("record_invalid", "quarantine payload was mutated"));
                }
            }
            let descriptor = io(fs::read(payload.join("plug.json")))?;
            if package::semantic_digest_for_plug_json(&descriptor)?
                != record.semantic_package_digest
            {
                return Err(err("record_invalid", "plug.json semantic evidence changed"));
            }
            records.push(record);
        }
        records.sort_by(|a, b| a.candidate_id.cmp(&b.candidate_id));
        Ok(records)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidate_record_golden_fixture_is_strict_and_covered() {
        let value =
            manifest::parse_value_no_dupes(include_str!("../fixtures/m2/candidate-record-v1.json"))
                .unwrap();
        let record: CandidateRecord = serde_json::from_value(value).unwrap();
        record.validate().unwrap();
    }

    #[test]
    fn registry_roots_must_remain_separate() {
        let root = std::env::temp_dir().join(format!("tethers-candidate-{}", Uuid::new_v4()));
        assert!(CandidateRegistry::open(&root, &root).is_err());
    }

    #[test]
    fn torn_temporary_record_fails_closed() {
        let root = std::env::temp_dir().join(format!("tethers-registry-{}", Uuid::new_v4()));
        let quarantine =
            std::env::temp_dir().join(format!("tethers-quarantine-{}", Uuid::new_v4()));
        let registry = CandidateRegistry::open(&root, &quarantine).unwrap();
        fs::write(root.join(".candidate.tmp"), b"partial").unwrap();
        assert_eq!(registry.load_all().unwrap_err().code, "record_invalid");
        fs::remove_dir_all(root).unwrap();
        fs::remove_dir_all(quarantine).unwrap();
    }

    #[test]
    fn filename_disagreement_and_duplicate_identity_evidence_fail_closed() {
        let root = std::env::temp_dir().join(format!("tethers-registry-{}", Uuid::new_v4()));
        let quarantine =
            std::env::temp_dir().join(format!("tethers-quarantine-{}", Uuid::new_v4()));
        let registry = CandidateRegistry::open(&root, &quarantine).unwrap();
        let fixture = include_str!("../fixtures/m2/candidate-record-v1.json");
        fs::write(root.join("wrong-name.json"), fixture).unwrap();
        assert_eq!(registry.load_all().unwrap_err().code, "record_invalid");
        fs::remove_file(root.join("wrong-name.json")).unwrap();

        let value = manifest::parse_value_no_dupes(fixture).unwrap();
        let record: CandidateRecord = serde_json::from_value(value).unwrap();
        fs::write(root.join(format!("{}.json", record.candidate_id)), fixture).unwrap();
        fs::write(root.join("duplicate-evidence.json"), fixture).unwrap();
        assert!(registry.load_all().is_err());
        fs::remove_dir_all(root).unwrap();
        fs::remove_dir_all(quarantine).unwrap();
    }

    #[test]
    fn preexisting_staging_target_and_escape_destination_fail_closed_without_write() {
        let root = std::env::temp_dir().join(format!("tethers-quarantine-{}", Uuid::new_v4()));
        let outside = std::env::temp_dir().join(format!("tethers-outside-{}", Uuid::new_v4()));
        fs::create_dir(&root).unwrap();
        fs::create_dir(&outside).unwrap();
        let staging = root.join(".staging-fixed-test");
        fs::create_dir(&staging).unwrap();
        assert_eq!(create_dir(&staging).unwrap_err().code, "already_exists");
        assert_eq!(
            confined(&root, &outside).unwrap_err().code,
            "unsafe_destination"
        );
        assert!(!outside.join("provider-marker.exe").exists());
        fs::remove_dir_all(root).unwrap();
        fs::remove_dir_all(outside).unwrap();
    }

    #[cfg(windows)]
    #[test]
    fn real_windows_junction_is_refused_as_a_quarantine_root() {
        let target = std::env::temp_dir().join(format!("tethers-target-{}", Uuid::new_v4()));
        let junction = std::env::temp_dir().join(format!("tethers-junction-{}", Uuid::new_v4()));
        let registry = std::env::temp_dir().join(format!("tethers-registry-{}", Uuid::new_v4()));
        fs::create_dir(&target).unwrap();
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
        assert!(
            status.success(),
            "could not create Windows junction fixture"
        );
        let configured_child = junction.join("configured-quarantine");
        let refusal = match CandidateRegistry::open(&registry, &configured_child) {
            Err(error) => error,
            Ok(_) => panic!("junction quarantine root was accepted"),
        };
        assert_eq!(refusal.code, "unsafe_destination");
        assert!(!target.join("configured-quarantine").exists());
        fs::remove_dir(junction).unwrap();
        fs::remove_dir_all(target).unwrap();
        fs::remove_dir_all(registry).unwrap();
    }
}
