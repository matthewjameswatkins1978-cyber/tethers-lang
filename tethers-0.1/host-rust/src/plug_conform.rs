use crate::candidate_preparation::prepare_installation_candidate;
use crate::cli::{CliEnvelope, OutcomeStatus};
use crate::conformance::{run_host_conformance_with_authority, ConformanceDisposition};
use crate::current_trust::ExactCandidateTrustAuthority;
use crate::installation_request::{
    InstallationConformanceRequest, InstallationRequest, InstallationTargetRequest,
    InstallationTargetState, InstallationTrustRequest, InstallationTrustScope,
    INSTALLATION_REQUEST_SCHEMA,
};
use crate::installation_trust::ExactCandidateTrustStore;
use crate::launch_profile::PreparedSupervisedLaunch;
use crate::trust::PackageTrustEvidence;
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use uuid::Uuid;

pub struct PlugCommandResult {
    pub envelope: CliEnvelope,
    pub exit_code: i32,
}

const PUBLIC_CONFORM_WALL_TIME_SECS: u64 = 30;
const APPROVING_AUTHORITY: &str = "tethers-public-conform-cli";

fn host_build_identity() -> String {
    format!("tethers-reference-host/{}", env!("CARGO_PKG_VERSION"))
}

pub fn run_conform(package: &Path, allow_execution: bool) -> PlugCommandResult {
    if !package.is_absolute() {
        let envelope = CliEnvelope::error(
            "plug conform",
            OutcomeStatus::InvalidCliUsage,
            "invalid_cli_usage",
            "--package must be an absolute path",
            Some("/package".into()),
        );
        return PlugCommandResult {
            exit_code: envelope.exit_code,
            envelope,
        };
    }

    if !allow_execution {
        let envelope = CliEnvelope::error(
            "plug conform",
            OutcomeStatus::ApprovalRequired,
            "conformance_execution_approval_required",
            "conformance executes provider code under process supervision, not isolation; pass --allow-non-isolated-supervised-execution to proceed",
            None,
        );
        return PlugCommandResult {
            exit_code: envelope.exit_code,
            envelope,
        };
    }

    let workspace = match create_ephemeral_workspace() {
        Ok(ws) => ws,
        Err(error) => return error,
    };

    let result = run_conform_in_workspace(&workspace, package);

    cleanup_workspace(&workspace);

    result
}

struct EphemeralWorkspace {
    root: PathBuf,
}

impl EphemeralWorkspace {
    fn quarantine_root(&self) -> PathBuf {
        self.root.join("quarantine")
    }

    fn trust_root(&self) -> PathBuf {
        self.root.join("installation-trust")
    }

    fn scratch_root(&self) -> PathBuf {
        self.root.join("conformance-scratch")
    }
}

fn create_ephemeral_workspace() -> Result<EphemeralWorkspace, PlugCommandResult> {
    let root = std::env::temp_dir().join(format!("tethers-p2b-conform-{}", Uuid::new_v4()));
    fs::create_dir_all(&root).map_err(|e| {
        let envelope = CliEnvelope::error(
            "plug conform",
            OutcomeStatus::Unavailable,
            "conformance_workspace_unavailable",
            format!("cannot create ephemeral workspace: {e}"),
            None,
        );
        PlugCommandResult {
            exit_code: envelope.exit_code,
            envelope,
        }
    })?;
    Ok(EphemeralWorkspace { root })
}

fn cleanup_workspace(workspace: &EphemeralWorkspace) {
    if workspace.root.exists() {
        let _ = fs::remove_dir_all(&workspace.root);
    }
}

fn run_conform_in_workspace(workspace: &EphemeralWorkspace, package: &Path) -> PlugCommandResult {
    let prepared = match prepare_installation_candidate(&workspace.root, package) {
        Ok(p) => p,
        Err(error) => {
            let status = candidate_error_status(&error);
            let envelope =
                CliEnvelope::error("plug conform", status, error.code, error.message, None);
            return PlugCommandResult {
                exit_code: envelope.exit_code,
                envelope,
            };
        }
    };

    let candidate = &prepared.candidate;

    let request = InstallationRequest {
        schema: INSTALLATION_REQUEST_SCHEMA.to_owned(),
        candidate_id: candidate.candidate_id.clone(),
        trust: InstallationTrustRequest {
            scope: InstallationTrustScope::ExactCandidate,
        },
        conformance: InstallationConformanceRequest {
            allow_non_isolated_supervised_execution: true,
        },
        installation: InstallationTargetRequest {
            target_state: InstallationTargetState::Disabled,
        },
    };

    let trust_store = match ExactCandidateTrustStore::open(&workspace.trust_root()) {
        Ok(store) => store,
        Err(error) => {
            let envelope = CliEnvelope::error(
                "plug conform",
                OutcomeStatus::Failed,
                "conformance_trust_store",
                format!("{}: {}", error.code, error.message),
                None,
            );
            return PlugCommandResult {
                exit_code: envelope.exit_code,
                envelope,
            };
        }
    };

    let trust_record = match trust_store.create(candidate, &request, APPROVING_AUTHORITY) {
        Ok(record) => record,
        Err(error) => {
            let envelope = CliEnvelope::error(
                "plug conform",
                OutcomeStatus::InvalidData,
                "conformance_trust_invalid",
                format!("{}: {}", error.code, error.message),
                None,
            );
            return PlugCommandResult {
                exit_code: envelope.exit_code,
                envelope,
            };
        }
    };

    let trust_evidence = match PackageTrustEvidence::exact_candidate(&trust_record) {
        Ok(evidence) => evidence,
        Err(error) => {
            let envelope = CliEnvelope::error(
                "plug conform",
                OutcomeStatus::InvalidData,
                "conformance_trust_invalid",
                format!("{}: {}", error.code, error.message),
                None,
            );
            return PlugCommandResult {
                exit_code: envelope.exit_code,
                envelope,
            };
        }
    };

    let prepared_launch = match PreparedSupervisedLaunch::prepare(
        candidate,
        &workspace.quarantine_root(),
        &workspace.scratch_root(),
        Duration::from_secs(PUBLIC_CONFORM_WALL_TIME_SECS),
    ) {
        Ok(launch) => launch,
        Err(error) => {
            let envelope = CliEnvelope::error(
                "plug conform",
                OutcomeStatus::Failed,
                "conformance_launch_prepare",
                format!("{}: {}", error.code, error.message),
                None,
            );
            return PlugCommandResult {
                exit_code: envelope.exit_code,
                envelope,
            };
        }
    };

    let authority = ExactCandidateTrustAuthority::new(&trust_store);

    let evidence = run_host_conformance_with_authority(
        &prepared_launch,
        candidate,
        &workspace.quarantine_root(),
        &trust_evidence,
        &authority,
        &host_build_identity(),
    );

    prepared_launch.cleanup_scratch().ok();

    match evidence {
        Ok(evidence) => {
            let disposition = evidence.disposition;
            map_conformance_result(evidence, disposition)
        }
        Err(error) => {
            let status = if error.code == "conformance_launch" {
                OutcomeStatus::Failed
            } else if error.code == "candidate_invalid" || error.code == "trust_drift" {
                OutcomeStatus::InvalidData
            } else if error.code.contains("unavailable") || error.code.contains("launch_io") {
                OutcomeStatus::Unavailable
            } else {
                OutcomeStatus::Failed
            };
            let envelope =
                CliEnvelope::error("plug conform", status, error.code, error.message, None);
            PlugCommandResult {
                exit_code: envelope.exit_code,
                envelope,
            }
        }
    }
}

fn candidate_error_status(error: &crate::package::PackageError) -> OutcomeStatus {
    match error.code {
        "archive_read" | "candidate_io" => OutcomeStatus::Unavailable,
        _ => OutcomeStatus::InvalidData,
    }
}

fn conformance_cases_json(evidence: &crate::conformance::ConformanceEvidence) -> serde_json::Value {
    serde_json::Value::Array(
        evidence
            .cases
            .iter()
            .map(|case| {
                json!({
                    "case_id": case.case_id,
                    "disposition": serde_json::to_value(&case.disposition).unwrap_or(serde_json::Value::Null),
                    "safe_diagnostic_code": case.safe_diagnostic_code,
                })
            })
            .collect(),
    )
}

fn conformance_disposition_json(
    disposition: &crate::conformance::ConformanceDisposition,
) -> &'static str {
    match disposition {
        ConformanceDisposition::Passed => "passed",
        ConformanceDisposition::Failed => "failed",
        ConformanceDisposition::Interrupted => "interrupted",
        ConformanceDisposition::Invalidated => "invalidated",
    }
}

fn map_conformance_result(
    evidence: crate::conformance::ConformanceEvidence,
    disposition: crate::conformance::ConformanceDisposition,
) -> PlugCommandResult {
    let public_data = json!({
        "package_id": evidence.package_id,
        "package_version": evidence.package_version,
        "semantic_package_digest": evidence.semantic_package_digest,
        "provider_id": evidence.provider_id,
        "provider_version": evidence.provider_version,
        "conformance": {
            "disposition": conformance_disposition_json(&disposition),
            "suite_version": evidence.suite_version,
            "suite_digest": evidence.suite_digest,
            "case_count": evidence.cases.len(),
            "cases": conformance_cases_json(&evidence),
        },
        "launch_profile": {
            "label": evidence.launch_profile_label,
            "isolated": false,
            "limitation": crate::launch_profile::SUPERVISED_PROFILE_LIMITATION,
            "wall_time_limit_ms": PUBLIC_CONFORM_WALL_TIME_SECS * 1000,
            "max_processes": 8_u64,
            "process_memory_limit_bytes": 256_u64 * 1024 * 1024,
        },
        "retry_count": evidence.retry_count,
        "raw_stderr_persisted": evidence.raw_stderr_persisted,
        "conformance_evidence_id": evidence.evidence_id,
        "conformance_evidence_digest": evidence.evidence_digest,
    });

    match disposition {
        ConformanceDisposition::Passed => {
            let envelope = CliEnvelope::ok("plug conform", public_data);
            PlugCommandResult {
                exit_code: envelope.exit_code,
                envelope,
            }
        }
        ConformanceDisposition::Failed => {
            let envelope = CliEnvelope::error_with_data(
                "plug conform",
                OutcomeStatus::Failed,
                "plug_conformance_failed",
                "conformance suite failed",
                None,
                public_data,
            );
            PlugCommandResult {
                exit_code: envelope.exit_code,
                envelope,
            }
        }
        ConformanceDisposition::Interrupted => {
            let envelope = CliEnvelope::error_with_data(
                "plug conform",
                OutcomeStatus::Interrupted,
                "plug_conformance_interrupted",
                "conformance suite was interrupted",
                None,
                public_data,
            );
            PlugCommandResult {
                exit_code: envelope.exit_code,
                envelope,
            }
        }
        ConformanceDisposition::Invalidated => {
            let envelope = CliEnvelope::error_with_data(
                "plug conform",
                OutcomeStatus::Failed,
                "plug_conformance_invalidated",
                "conformance evidence was invalidated",
                None,
                public_data,
            );
            PlugCommandResult {
                exit_code: envelope.exit_code,
                envelope,
            }
        }
    }
}
