use std::fs;
use std::path::Path;

use crate::candidate::{self, verify_existing_chain, CandidateRecord, CandidateRegistry};
use crate::package::{self, InspectionReport, PackageError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CandidatePreparationDisposition {
    Created,
    Existing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidatePreparation {
    pub candidate: CandidateRecord,
    pub disposition: CandidatePreparationDisposition,
}

fn failure(code: &'static str, message: impl Into<String>) -> PackageError {
    PackageError {
        code,
        message: message.into(),
    }
}

fn is_empty_dir(path: &Path) -> bool {
    fs::read_dir(path)
        .map(|mut dir| dir.next().is_none())
        .unwrap_or(false)
}

fn cleanup_new_empty_roots(
    candidate_root: &Path,
    quarantine_root: &Path,
    candidate_root_existed: bool,
    quarantine_root_existed: bool,
) -> Result<(), PackageError> {
    if !candidate_root_existed && candidate_root.exists() && is_empty_dir(candidate_root) {
        fs::remove_dir(candidate_root)
            .map_err(|e| failure("candidate_rollback_failed", e.to_string()))?;
    }
    if !quarantine_root_existed && quarantine_root.exists() && is_empty_dir(quarantine_root) {
        fs::remove_dir(quarantine_root)
            .map_err(|e| failure("candidate_rollback_failed", e.to_string()))?;
    }
    Ok(())
}

fn rollback_new_quarantine(quarantine_root: &Path, directory: &Path) -> Result<(), PackageError> {
    let root = fs::canonicalize(quarantine_root)
        .map_err(|e| failure("candidate_rollback_failed", e.to_string()))?;
    let dir = fs::canonicalize(directory)
        .map_err(|e| failure("candidate_rollback_failed", e.to_string()))?;
    if !dir.starts_with(&root) || dir == root {
        return Err(failure(
            "candidate_rollback_failed",
            "quarantine directory escaped configured root",
        ));
    }
    fs::remove_dir_all(&dir).map_err(|e| failure("candidate_rollback_failed", e.to_string()))?;
    Ok(())
}

fn require_absolute_existing_file(path: &Path) -> Result<(), PackageError> {
    if !path.is_absolute() {
        return Err(failure("invalid_archive", "package path must be absolute"));
    }
    if !path.is_file() {
        return Err(failure("archive_read", "package file not found"));
    }
    Ok(())
}

fn require_absolute_existing_safe_directory(path: &Path) -> Result<(), PackageError> {
    if !path.is_absolute() {
        return Err(failure(
            "unsafe_destination",
            "host data root must be absolute",
        ));
    }
    if !path.is_dir() {
        return Err(failure(
            "unsafe_destination",
            "host data root must be an existing directory",
        ));
    }
    verify_existing_chain(path)
}

fn exact_replay(
    existing: &[CandidateRecord],
    report: &InspectionReport,
) -> Result<Option<CandidateRecord>, PackageError> {
    let matches: Vec<&CandidateRecord> = existing
        .iter()
        .filter(|c| c.raw_archive_digest == report.raw_archive_digest)
        .collect();
    match matches.len() {
        0 => Ok(None),
        1 => {
            let candidate = matches[0];
            if candidate.package_id == report.package.package_id
                && candidate.package_version == report.package.package_version
                && candidate.semantic_package_digest == report.package.semantic_digest
                && candidate.source_size_bytes == report.raw_archive_size
                && candidate.provider_id == report.provider_id
                && candidate.provider_version == report.provider_version
                && candidate.launch_path == report.provider_launch_path
                && candidate.launch_arguments == report.provider_launch_arguments
                && candidate.provider_working_directory == report.provider_working_directory
                && candidate.capability_operation_namespace == report.provider_operation_namespace
                && candidate.selected_platform == report.selected_platform
                && candidate.plug_json == report.plug_json
                && candidate.payloads == report.payloads
                && candidate.signature_files == report.signature_files
                && candidate.signatures_present == report.signatures_present
                && candidate.capabilities == report.capabilities
                && candidate.inspection_report_format_version == report.inspection_format_version
                && candidate.inspection_evidence_digest == report.inspection_evidence_digest
            {
                Ok(Some(candidate.clone()))
            } else {
                Err(failure(
                    "record_invalid",
                    "existing candidate evidence does not match this archive",
                ))
            }
        }
        _ => Err(failure(
            "candidate_conflict",
            "multiple candidates share the same archive bytes",
        )),
    }
}

fn refuse_semantic_conflict(
    existing: &[CandidateRecord],
    report: &InspectionReport,
) -> Result<(), PackageError> {
    for record in existing {
        if record.package_id == report.package.package_id
            && record.package_version == report.package.package_version
            && record.semantic_package_digest != report.package.semantic_digest
        {
            return Err(failure(
                "semantic_conflict",
                "same package release has different semantic evidence",
            ));
        }
    }
    Ok(())
}

pub fn prepare_installation_candidate(
    host_data_root: &Path,
    package_path: &Path,
) -> Result<CandidatePreparation, PackageError> {
    require_absolute_existing_file(package_path)?;
    require_absolute_existing_safe_directory(host_data_root)?;

    let report = package::inspect(package_path)?;

    let candidate_root = host_data_root.join("candidates");
    let quarantine_root = host_data_root.join("quarantine");
    let candidate_root_existed = candidate_root.exists();
    let quarantine_root_existed = quarantine_root.exists();

    let registry = match CandidateRegistry::open(&candidate_root, &quarantine_root) {
        Ok(registry) => registry,
        Err(error) => {
            cleanup_new_empty_roots(
                &candidate_root,
                &quarantine_root,
                candidate_root_existed,
                quarantine_root_existed,
            )?;
            return Err(error);
        }
    };

    let existing = match registry.load_all() {
        Ok(records) => records,
        Err(error) => {
            cleanup_new_empty_roots(
                &candidate_root,
                &quarantine_root,
                candidate_root_existed,
                quarantine_root_existed,
            )?;
            return Err(error);
        }
    };

    if let Some(candidate) = exact_replay(&existing, &report)? {
        return Ok(CandidatePreparation {
            candidate,
            disposition: CandidatePreparationDisposition::Existing,
        });
    }

    refuse_semantic_conflict(&existing, &report)?;

    let quarantined = match candidate::extract_to_quarantine(&report, &quarantine_root) {
        Ok(value) => value,
        Err(error) => {
            cleanup_new_empty_roots(
                &candidate_root,
                &quarantine_root,
                candidate_root_existed,
                quarantine_root_existed,
            )?;
            return Err(error);
        }
    };

    match registry.create(&quarantined) {
        Ok(candidate) => Ok(CandidatePreparation {
            candidate,
            disposition: CandidatePreparationDisposition::Created,
        }),
        Err(error) => {
            rollback_new_quarantine(&quarantine_root, &quarantined.directory)?;
            cleanup_new_empty_roots(
                &candidate_root,
                &quarantine_root,
                candidate_root_existed,
                quarantine_root_existed,
            )?;
            Err(error)
        }
    }
}
