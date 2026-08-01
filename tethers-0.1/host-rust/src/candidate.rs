//! Host-owned quarantine and immutable M2 installation candidates.
//!
//! These records are deliberately not installed-Plug records: they carry no
//! trust, approval, binding, credential, session, policy, or launch authority.

use crate::manifest;
use crate::package::{self, InspectionReport, PackageError};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

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

fn reject_link(path: &Path) -> Result<(), PackageError> {
    let metadata = io(fs::symlink_metadata(path))?;
    if metadata.file_type().is_symlink() {
        Err(err(
            "unsafe_destination",
            "links and reparse destinations are refused",
        ))
    } else {
        Ok(())
    }
}
fn confined(root: &Path, child: &Path) -> Result<PathBuf, PackageError> {
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
    if quarantine_root.exists() {
        reject_link(quarantine_root)?;
    } else {
        io(fs::create_dir_all(quarantine_root))?;
    }
    let root = io(fs::canonicalize(quarantine_root))?;
    reject_link(&root)?;
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
        let mut expected = std::collections::BTreeMap::new();
        expected.insert("plug.json".to_owned(), None);
        for p in &report.payloads {
            expected.insert(p.path.clone(), Some((p.sha256.clone(), p.size_bytes)));
        }
        for index in 0..archive.len() {
            let mut entry = archive
                .by_index(index)
                .map_err(|e| err("archive_read", e.to_string()))?;
            let name = std::str::from_utf8(entry.name_raw())
                .map_err(|_| err("archive_read", "non UTF-8 entry after inspection"))?
                .to_owned();
            let signature = name.starts_with("signatures/");
            if !expected.contains_key(&name) && !signature {
                continue;
            }
            let relative = Path::new(&name);
            if relative.components().count() == 0 || relative.is_absolute() {
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
                    reject_link(&cursor)?;
                }
            }
            let mut bytes = Vec::new();
            entry
                .read_to_end(&mut bytes)
                .map_err(|e| err("archive_read", e.to_string()))?;
            if let Some(Some((digest, size))) = expected.get(&name) {
                if bytes.len() as u64 != *size || sha(&bytes) != *digest {
                    return Err(err(
                        "payload_mismatch",
                        "second payload verification failed",
                    ));
                }
            }
            write_new(&target, &bytes)?;
        }
        for key in expected.keys() {
            if !staging.join(key).is_file() {
                return Err(err(
                    "payload_mismatch",
                    "accepted archive entry was not extracted",
                ));
            }
        }
        Ok(())
    })();
    if let Err(failure) = result {
        let _ = fs::remove_dir_all(&staging);
        return Err(failure);
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
    pub payload_digests: Vec<(String, String)>,
    pub capabilities: Vec<(String, u32, String, String)>,
    pub signatures_present: bool,
    pub inspection_format_version: u32,
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
        io(fs::create_dir_all(root))?;
        io(fs::create_dir_all(quarantine_root))?;
        reject_link(root)?;
        reject_link(quarantine_root)?;
        Ok(Self {
            root: io(fs::canonicalize(root))?,
            quarantine_root: io(fs::canonicalize(quarantine_root))?,
        })
    }
    pub fn create(
        &self,
        quarantined: &QuarantinedPackage,
    ) -> Result<CandidateRecord, PackageError> {
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
            payload_digests: quarantined
                .report
                .payloads
                .iter()
                .map(|p| (p.path.clone(), p.sha256.clone()))
                .collect(),
            capabilities: quarantined
                .report
                .capabilities
                .iter()
                .map(|c| {
                    (
                        c.name.clone(),
                        c.version,
                        c.operation.clone(),
                        c.manifest_digest.clone(),
                    )
                })
                .collect(),
            signatures_present: quarantined.report.signatures_present,
            inspection_format_version: 1,
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
        io(fs::rename(&temporary, &destination))?;
        Ok(record)
    }
    pub fn load_all(&self) -> Result<Vec<CandidateRecord>, PackageError> {
        let mut records = Vec::new();
        for entry in io(fs::read_dir(&self.root))? {
            let path = io(entry)?.path();
            if path.extension().and_then(|x| x.to_str()) == Some("tmp") {
                return Err(err("record_invalid", "torn temporary record present"));
            }
            if path.extension().and_then(|x| x.to_str()) != Some("json") {
                continue;
            }
            let text = io(fs::read_to_string(&path))?;
            let value = manifest::parse_value_no_dupes(&text)
                .map_err(|e| err("record_invalid", e.to_string()))?;
            let record: CandidateRecord =
                serde_json::from_value(value).map_err(|e| err("record_invalid", e.to_string()))?;
            record.validate()?;
            let payload = confined(
                &self.quarantine_root,
                &self.quarantine_root.join(&record.quarantine_relative_path),
            )?;
            if !payload.is_dir() {
                return Err(err("record_invalid", "quarantine payload missing"));
            }
            for (relative, expected_digest) in &record.payload_digests {
                let bytes = io(fs::read(payload.join(relative)))?;
                if sha(&bytes) != *expected_digest {
                    return Err(err("record_invalid", "quarantine payload was mutated"));
                }
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
}
