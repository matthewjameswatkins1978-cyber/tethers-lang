use crate::cli::{CliEnvelope, OutcomeStatus};
use crate::enablement::{EnablementState, EnablementStore};
use crate::installed::InstalledPlugRegistry;
use crate::m3_store::M3Error;
use crate::package;
use serde_json::json;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub struct PlugCommandResult {
    pub envelope: CliEnvelope,
    pub exit_code: i32,
}

fn candidate_is_newer(current: Option<u64>, candidate: u64) -> bool {
    current.is_none_or(|sequence| candidate > sequence)
}

pub fn run_inspect(package_path: &Path) -> PlugCommandResult {
    match package::inspect(package_path) {
        Ok(report) => {
            let envelope = CliEnvelope::ok("plug inspect", json!({ "inspection": report }));
            PlugCommandResult {
                exit_code: envelope.exit_code,
                envelope,
            }
        }
        Err(error) => {
            let status = if error.code == "archive_read" {
                OutcomeStatus::Unavailable
            } else {
                OutcomeStatus::InvalidData
            };
            let envelope =
                CliEnvelope::error("plug inspect", status, error.code, error.message, None);
            PlugCommandResult {
                exit_code: envelope.exit_code,
                envelope,
            }
        }
    }
}

fn list_error(error: M3Error, status: OutcomeStatus) -> PlugCommandResult {
    let envelope = CliEnvelope::error("plug list", status, error.code, error.message, None);
    PlugCommandResult {
        exit_code: envelope.exit_code,
        envelope,
    }
}

fn list_store_error(error: M3Error) -> PlugCommandResult {
    let status = if error.code == "store_io" {
        OutcomeStatus::Unavailable
    } else {
        OutcomeStatus::InvalidData
    };
    list_error(error, status)
}

pub fn run_list(host_data_root: &Path) -> PlugCommandResult {
    if !host_data_root.is_absolute() {
        let envelope = CliEnvelope::error(
            "plug list",
            OutcomeStatus::InvalidCliUsage,
            "invalid_cli_usage",
            "--host-data-root must be absolute",
            Some("/host-data-root".into()),
        );
        return PlugCommandResult {
            exit_code: envelope.exit_code,
            envelope,
        };
    }
    match fs::symlink_metadata(host_data_root) {
        Ok(metadata) if metadata.is_dir() => {}
        _ => {
            return list_error(
                M3Error::new(
                    "plug_data_root_unavailable",
                    "host data root is unavailable",
                ),
                OutcomeStatus::Unavailable,
            )
        }
    }
    if let Err(error) = crate::m3_store::verify_chain(host_data_root) {
        return list_error(error, OutcomeStatus::InvalidData);
    }
    let paths: Vec<_> = ["install", "installed-records", "enablements"]
        .iter()
        .map(|name| host_data_root.join(name))
        .collect();
    let present = paths.iter().filter(|path| path.exists()).count();
    if present == 0 {
        let envelope = CliEnvelope::ok("plug list", json!({"count": 0, "plugs": []}));
        return PlugCommandResult {
            exit_code: envelope.exit_code,
            envelope,
        };
    }
    if present != paths.len()
        || paths.iter().any(|path| {
            fs::symlink_metadata(path)
                .map(|m| !m.is_dir() || m.file_type().is_symlink())
                .unwrap_or(true)
        })
    {
        return list_error(
            M3Error::new(
                "plug_store_incomplete",
                "lifecycle store layout is incomplete",
            ),
            OutcomeStatus::InvalidData,
        );
    }
    let installed = match InstalledPlugRegistry::open_existing(&paths[0], &paths[1])
        .and_then(|store| store.load_all())
    {
        Ok(records) => records,
        Err(error) => return list_store_error(error),
    };
    let enablements =
        match EnablementStore::open_existing(&paths[2]).and_then(|store| store.load_all()) {
            Ok(records) => records,
            Err(error) => return list_store_error(error),
        };
    let installed_ids: BTreeMap<_, _> = installed
        .iter()
        .map(|record| (record.installed_id.clone(), record))
        .collect();
    let mut latest: BTreeMap<String, crate::enablement::EnablementRecord> = BTreeMap::new();
    for transition in enablements {
        if !installed_ids.contains_key(&transition.installed_id) {
            return list_error(
                M3Error::new(
                    "enablement_invalid",
                    "enablement references unknown installed Plug",
                ),
                OutcomeStatus::InvalidData,
            );
        }
        latest
            .entry(transition.installed_id.clone())
            .and_modify(|current| {
                if candidate_is_newer(Some(current.sequence), transition.sequence) {
                    *current = transition.clone();
                }
            })
            .or_insert(transition);
    }
    let mut plugs = Vec::new();
    for record in installed {
        let transition = latest.get(&record.installed_id);
        if let Some(item) = transition {
            let expected: BTreeMap<_, _> = record
                .disabled_bindings
                .iter()
                .map(|binding| {
                    (
                        (binding.capability_name.clone(), binding.capability_version),
                        (
                            binding.manifest_digest.clone(),
                            binding.provider_operation_name.clone(),
                        ),
                    )
                })
                .collect();
            let actual: BTreeMap<_, _> = item
                .capabilities
                .iter()
                .map(|binding| {
                    (
                        (binding.name.clone(), binding.version),
                        (
                            binding.manifest_digest.clone(),
                            binding.provider_operation_name.clone(),
                        ),
                    )
                })
                .collect();
            if item.package_id != record.package_id
                || item.semantic_package_digest != record.semantic_package_digest
                || item.provider_id != record.provider_id
                || item.provider_version != record.provider_version
                || item.conformance_evidence_digest != record.conformance_evidence_digest
                || item.installation_approval_id != record.installation_approval_id
                || expected != actual
            {
                return list_error(
                    M3Error::new(
                        "enablement_invalid",
                        "enablement evidence does not match installed Plug",
                    ),
                    OutcomeStatus::InvalidData,
                );
            }
        }
        let capabilities = transition
            .filter(|item| item.state == EnablementState::Enabled)
            .map(|item| item.capabilities.clone())
            .unwrap_or_else(|| {
                record
                    .disabled_bindings
                    .iter()
                    .map(|binding| crate::enablement::EnabledCapability {
                        name: binding.capability_name.clone(),
                        version: binding.capability_version,
                        manifest_digest: binding.manifest_digest.clone(),
                        provider_operation_name: binding.provider_operation_name.clone(),
                    })
                    .collect()
            });
        let state = transition.map_or("disabled", |item| {
            if item.state == EnablementState::Enabled {
                "enabled"
            } else {
                "disabled"
            }
        });
        let mut capabilities = capabilities;
        capabilities.sort_by(|left, right| {
            (left.name.as_str(), left.version).cmp(&(right.name.as_str(), right.version))
        });
        plugs.push(json!({"installed_id": record.installed_id, "package_id": record.package_id, "package_version": record.package_version, "semantic_package_digest": record.semantic_package_digest, "provider_id": record.provider_id, "provider_version": record.provider_version, "state": state, "capabilities": capabilities, "created_unix_ms": record.created_unix_ms}));
    }
    plugs.sort_by(|left, right| {
        (
            left["package_id"].as_str(),
            left["package_version"].as_str(),
            left["installed_id"].as_str(),
        )
            .cmp(&(
                right["package_id"].as_str(),
                right["package_version"].as_str(),
                right["installed_id"].as_str(),
            ))
    });
    let envelope = CliEnvelope::ok("plug list", json!({"count": plugs.len(), "plugs": plugs}));
    PlugCommandResult {
        exit_code: envelope.exit_code,
        envelope,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn j24a_invalid_extension_maps_to_invalid_data() {
        let result = run_inspect(Path::new("not-a-package.zip"));
        assert_eq!(result.exit_code, 3);
        assert_eq!(result.envelope.status, OutcomeStatus::InvalidData);
        assert_eq!(
            result.envelope.error.as_ref().unwrap().code,
            "invalid_archive"
        );
    }

    #[test]
    fn j24a_missing_package_maps_to_unavailable() {
        let result = run_inspect(Path::new("missing.tetherplug"));
        assert_eq!(result.exit_code, 4);
        assert_eq!(result.envelope.status, OutcomeStatus::Unavailable);
        assert_eq!(result.envelope.error.as_ref().unwrap().code, "archive_read");
    }

    #[test]
    fn j24b_latest_transition_selection_uses_sequence_not_filename_order() {
        let mut selected = None;
        for (filename_order, sequence) in [("z.json", 2), ("a.json", 1)] {
            let _ = filename_order;
            if candidate_is_newer(selected, sequence) {
                selected = Some(sequence);
            }
        }
        assert_eq!(selected, Some(2));
        selected = None;
        for (filename_order, sequence) in [("a.json", 1), ("z.json", 2)] {
            let _ = filename_order;
            if candidate_is_newer(selected, sequence) {
                selected = Some(sequence);
            }
        }
        assert_eq!(selected, Some(2));
    }
}
