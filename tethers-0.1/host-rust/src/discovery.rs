//! Read-only machine discovery for the trusted host state.
//!
//! Discovery deliberately projects existing installed records, enablement
//! transitions, and verified capability manifests. It does not create a
//! second registry, query live providers, grant permission, or start a
//! process. Enabled here means that the host has an enabled, integrity-
//! checked binding; live provider health remains a separate execution fact.

use crate::cli::{CliEnvelope, OutcomeStatus};
use crate::enablement::EnablementState;
use crate::installed::{InstalledPlugRecord, InstalledPlugRegistry};
use crate::m3_store::{verify_chain, M3Error};
use crate::manifest::{self, TrustedManifest};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

pub struct DiscoveryResult {
    pub envelope: CliEnvelope,
    pub exit_code: i32,
}

struct DiscoveryError {
    status: OutcomeStatus,
    code: String,
    message: String,
    field: Option<String>,
}

impl DiscoveryError {
    fn invalid_data(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            status: OutcomeStatus::InvalidData,
            code: code.into(),
            message: message.into(),
            field: None,
        }
    }

    fn unavailable(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            status: OutcomeStatus::Unavailable,
            code: code.into(),
            message: message.into(),
            field: None,
        }
    }

    fn usage(message: impl Into<String>, field: &str) -> Self {
        Self {
            status: OutcomeStatus::InvalidCliUsage,
            code: "invalid_cli_usage".to_owned(),
            message: message.into(),
            field: Some(field.to_owned()),
        }
    }
}

struct CapabilitySnapshot {
    installed: InstalledPlugRecord,
    enabled: bool,
    available: bool,
    availability_reason: &'static str,
    effective_scope: Option<Value>,
    manifest_evidence: crate::package::CapabilityEvidence,
    manifest: TrustedManifest,
    raw_manifest: Value,
}

struct StoreSnapshot {
    installed: Vec<InstalledPlugRecord>,
    capabilities: Vec<CapabilitySnapshot>,
}

fn result_from_error(command: &str, error: DiscoveryError) -> DiscoveryResult {
    let envelope = CliEnvelope::error(
        command,
        error.status,
        error.code,
        error.message,
        error.field,
    );
    DiscoveryResult {
        exit_code: envelope.exit_code,
        envelope,
    }
}

fn map_store_error(error: M3Error) -> DiscoveryError {
    let status = match error.code {
        "store_io" | "plug_data_root_unavailable" => OutcomeStatus::Unavailable,
        _ => OutcomeStatus::InvalidData,
    };
    DiscoveryError {
        status,
        code: error.code.to_owned(),
        message: error.message,
        field: None,
    }
}

fn require_absolute(root: &Path, field: &str) -> Result<(), DiscoveryError> {
    if root.is_absolute() {
        Ok(())
    } else {
        Err(DiscoveryError::usage(
            format!("{field} must be absolute"),
            field,
        ))
    }
}

fn store_paths(root: &Path) -> [PathBuf; 3] {
    [
        root.join("install"),
        root.join("installed-records"),
        root.join("enablements"),
    ]
}

fn load_store(root: &Path) -> Result<StoreSnapshot, DiscoveryError> {
    require_absolute(root, "--host-data-root")?;
    if fs::symlink_metadata(root)
        .map(|metadata| !metadata.is_dir())
        .unwrap_or(true)
    {
        return Err(DiscoveryError::unavailable(
            "plug_data_root_unavailable",
            "host data root is unavailable",
        ));
    }
    verify_chain(root).map_err(map_store_error)?;
    let paths = store_paths(root);
    let present = paths.iter().filter(|path| path.exists()).count();
    if present == 0 {
        return Ok(StoreSnapshot {
            installed: Vec::new(),
            capabilities: Vec::new(),
        });
    }
    if present != paths.len()
        || paths.iter().any(|path| {
            fs::symlink_metadata(path)
                .map(|metadata| !metadata.is_dir() || metadata.file_type().is_symlink())
                .unwrap_or(true)
        })
    {
        return Err(DiscoveryError::invalid_data(
            "plug_store_incomplete",
            "lifecycle store layout is incomplete",
        ));
    }

    let registry =
        InstalledPlugRegistry::open_existing(&paths[0], &paths[1]).map_err(map_store_error)?;
    let installed = registry.load_all().map_err(map_store_error)?;
    let enablements = crate::enablement::EnablementStore::open_existing(&paths[2])
        .and_then(|store| store.load_all())
        .map_err(map_store_error)?;
    let latest = crate::plug_command::select_latest_transition(&enablements);

    let installed_by_id: BTreeMap<_, _> = installed
        .iter()
        .map(|record| (record.installed_id.as_str(), record))
        .collect();
    for transition in &enablements {
        let Some(record) = installed_by_id.get(transition.installed_id.as_str()) else {
            return Err(DiscoveryError::invalid_data(
                "enablement_invalid",
                "enablement references unknown installed Plug",
            ));
        };
        transition
            .consistent_with(record)
            .map_err(map_store_error)?;
    }

    let mut capabilities = Vec::new();
    for record in &installed {
        let transition = latest.get(&record.installed_id);
        let enabled = transition.is_some_and(|item| item.state == EnablementState::Enabled);
        let effective_scope = transition
            .filter(|item| item.state == EnablementState::Enabled)
            .map(|item| item.operational_scope.canonical_scope())
            .transpose()
            .map_err(map_store_error)?;
        let directory = registry
            .installation_directory(record)
            .map_err(map_store_error)?;
        for evidence in &record.capability_manifests {
            let manifest_path = directory.join(&evidence.manifest_path);
            let bytes = fs::read(&manifest_path).map_err(|error| {
                DiscoveryError::invalid_data(
                    "manifest_unavailable",
                    format!("installed capability manifest could not be read: {error}"),
                )
            })?;
            let text = std::str::from_utf8(&bytes).map_err(|_| {
                DiscoveryError::invalid_data(
                    "manifest_invalid",
                    "installed capability manifest is not UTF-8",
                )
            })?;
            let verified = manifest::verify_manifest(text)
                .map_err(|error| DiscoveryError::invalid_data("manifest_invalid", error.message))?;
            if verified.capability_name() != evidence.name
                || verified.capability_version() != evidence.version
                || verified.verified_digest() != evidence.manifest_digest
            {
                return Err(DiscoveryError::invalid_data(
                    "manifest_mismatch",
                    "installed capability manifest identity or digest mismatch",
                ));
            }
            let raw_manifest = serde_json::from_str(text).map_err(|error| {
                DiscoveryError::invalid_data(
                    "manifest_invalid",
                    format!("installed capability manifest is not an object: {error}"),
                )
            })?;
            let availability_reason = if enabled {
                "enabled_binding_not_live_checked"
            } else {
                "plug_disabled"
            };
            capabilities.push(CapabilitySnapshot {
                installed: record.clone(),
                enabled,
                available: enabled,
                availability_reason,
                effective_scope: effective_scope.clone(),
                manifest_evidence: evidence.clone(),
                manifest: verified.manifest().clone(),
                raw_manifest,
            });
        }
    }

    let mut identities = BTreeSet::new();
    for capability in &capabilities {
        let identity = (
            capability.manifest.capability_name.clone(),
            capability.manifest.capability_version,
        );
        if !identities.insert(identity) {
            return Err(DiscoveryError::invalid_data(
                "duplicate_capability_version",
                "multiple installed Plugs provide the same capability name and version",
            ));
        }
    }
    capabilities.sort_by(|left, right| {
        (
            left.manifest.capability_name.as_str(),
            left.manifest.capability_version,
            left.installed.package_id.as_str(),
            left.installed.installed_id.as_str(),
        )
            .cmp(&(
                right.manifest.capability_name.as_str(),
                right.manifest.capability_version,
                right.installed.package_id.as_str(),
                right.installed.installed_id.as_str(),
            ))
    });
    Ok(StoreSnapshot {
        installed,
        capabilities,
    })
}

fn capability_summary(capability: &CapabilitySnapshot) -> Value {
    json!({
        "name": capability.manifest.capability_name,
        "version": capability.manifest.capability_version,
        "title": capability.manifest.title,
        "description": capability.manifest.description,
        "effects": capability.manifest.effects,
        "provider": capability.installed.provider_id,
        "provider_version": capability.installed.provider_version,
        "provider_operation": capability.manifest.binding.tool_name,
        "manifest_digest": capability.manifest_evidence.manifest_digest,
        "installed_id": capability.installed.installed_id,
        "plug": {
            "package_id": capability.installed.package_id,
            "package_version": capability.installed.package_version,
        },
        "state": if capability.enabled { "enabled" } else { "disabled" },
        "available": capability.available,
        "availability_reason": capability.availability_reason,
    })
}

fn matches_filters(
    capability: &CapabilitySnapshot,
    effect: Option<&str>,
    provider: Option<&str>,
    plug: Option<&str>,
) -> bool {
    effect.is_none_or(|value| capability.manifest.effects.iter().any(|item| item == value))
        && provider.is_none_or(|value| capability.installed.provider_id == value)
        && plug.is_none_or(|value| capability.installed.installed_id == value)
}

pub fn run_describe(host_data_root: Option<&Path>) -> DiscoveryResult {
    let snapshot = match host_data_root {
        Some(root) => match load_store(root) {
            Ok(snapshot) => Some(snapshot),
            Err(error) => return result_from_error("describe", error),
        },
        None => None,
    };
    let installed = snapshot.as_ref().map_or(0, |value| value.installed.len());
    let capabilities = snapshot.as_ref().map_or(0, |value| {
        value
            .capabilities
            .iter()
            .filter(|item| item.available)
            .count()
    });
    let enabled = snapshot.as_ref().map_or(0, |value| {
        value
            .installed
            .iter()
            .filter(|record| {
                value
                    .capabilities
                    .iter()
                    .any(|item| item.installed.installed_id == record.installed_id && item.enabled)
            })
            .count()
    });
    let data = json!({
        "schema": "tethers.describe/1",
        "version": env!("CARGO_PKG_VERSION"),
        "cli_schema": "tethers.cli/1",
        "supported_protocol_versions": ["0.1"],
        "supported_language_versions": ["0.1"],
        "features": {
            "capabilities": true,
            "plugs": true,
            "planning": true,
            "together": true,
            "trails": true
        },
        "concurrency": {
            "together_supported": true,
            "limit_source": "runtime_configuration"
        },
        "installed_plugs": installed,
        "enabled_plugs": enabled,
        "available_capabilities": capabilities,
        "supported_discovery_commands": ["describe", "capability list", "capability inspect", "plug show"],
        "host_health": {
            "status": if host_data_root.is_some() { "configured_state_only" } else { "not_configured" },
            "provider_health_checked": false,
            "host_data_configured": host_data_root.is_some()
        }
    });
    let envelope = CliEnvelope::ok("describe", data);
    DiscoveryResult {
        exit_code: envelope.exit_code,
        envelope,
    }
}

pub fn run_capability_list(
    host_data_root: &Path,
    all: bool,
    effect: Option<&str>,
    provider: Option<&str>,
    plug: Option<&str>,
) -> DiscoveryResult {
    let snapshot = match load_store(host_data_root) {
        Ok(snapshot) => snapshot,
        Err(error) => return result_from_error("capability list", error),
    };
    let capabilities: Vec<_> = snapshot
        .capabilities
        .iter()
        .filter(|capability| all || capability.available)
        .filter(|capability| matches_filters(capability, effect, provider, plug))
        .map(capability_summary)
        .collect();
    let envelope = CliEnvelope::ok(
        "capability list",
        json!({"count": capabilities.len(), "capabilities": capabilities}),
    );
    DiscoveryResult {
        exit_code: envelope.exit_code,
        envelope,
    }
}

fn valid_versions(snapshot: &StoreSnapshot, name: &str) -> Vec<u32> {
    let mut versions: Vec<_> = snapshot
        .capabilities
        .iter()
        .filter(|item| item.manifest.capability_name == name)
        .map(|item| item.manifest.capability_version)
        .collect();
    versions.sort_unstable();
    versions.dedup();
    versions
}

pub fn run_capability_inspect(
    host_data_root: &Path,
    name: &str,
    version: Option<u32>,
) -> DiscoveryResult {
    let snapshot = match load_store(host_data_root) {
        Ok(snapshot) => snapshot,
        Err(error) => return result_from_error("capability inspect", error),
    };
    let versions = valid_versions(&snapshot, name);
    let selected = match version {
        Some(version) => snapshot.capabilities.iter().find(|item| {
            item.manifest.capability_name == name && item.manifest.capability_version == version
        }),
        None if versions.len() == 1 => snapshot.capabilities.iter().find(|item| {
            item.manifest.capability_name == name && item.manifest.capability_version == versions[0]
        }),
        None if versions.is_empty() => None,
        None => {
            return result_from_error("capability inspect", DiscoveryError::invalid_data("ambiguous_capability_version", format!("capability has multiple versions; supply --version; valid versions: {versions:?}")));
        }
    };
    let Some(capability) = selected else {
        let message = if versions.is_empty() {
            "capability was not found".to_owned()
        } else {
            format!("requested capability version is unavailable; valid versions: {versions:?}")
        };
        return result_from_error(
            "capability inspect",
            DiscoveryError {
                status: OutcomeStatus::NotFound,
                code: "capability_not_found".to_owned(),
                message,
                field: Some("name".to_owned()),
            },
        );
    };
    let mut contract = capability.raw_manifest.clone();
    if let Some(object) = contract.as_object_mut() {
        object.insert(
            "provider_operation_binding".to_owned(),
            json!({
                "server_name": capability.manifest.binding.server_name,
                "tool_name": capability.manifest.binding.tool_name,
                "binding_kind": "mcp"
            }),
        );
    }
    let data = json!({
        "schema": "tethers.capability.inspect/1",
        "contract": contract,
        "provider": {
            "identity": capability.installed.provider_id,
            "version": capability.installed.provider_version,
            "operation": capability.manifest.binding.tool_name
        },
        "plug": {
            "installed_id": capability.installed.installed_id,
            "package_id": capability.installed.package_id,
            "package_version": capability.installed.package_version,
            "semantic_package_digest": capability.installed.semantic_package_digest
        },
        "manifest_digest": capability.manifest_evidence.manifest_digest,
        "availability": {
            "enabled": capability.enabled,
            "available": capability.available,
            "reason": capability.availability_reason,
            "effective_scope": capability.effective_scope
        },
        "conformance": {
            "status": "evidence_present",
            "evidence_id": capability.installed.conformance_evidence_id,
            "evidence_digest": capability.installed.conformance_evidence_digest
        }
    });
    let envelope = CliEnvelope::ok("capability inspect", data);
    DiscoveryResult {
        exit_code: envelope.exit_code,
        envelope,
    }
}

pub fn run_plug_show(host_data_root: &Path, installed_id: &str) -> DiscoveryResult {
    let snapshot = match load_store(host_data_root) {
        Ok(snapshot) => snapshot,
        Err(error) => return result_from_error("plug show", error),
    };
    let Some(record) = snapshot
        .installed
        .iter()
        .find(|record| record.installed_id == installed_id)
    else {
        return result_from_error(
            "plug show",
            DiscoveryError {
                status: OutcomeStatus::NotFound,
                code: "installed_plug_not_found".to_owned(),
                message: "installed Plug was not found".to_owned(),
                field: Some("installed-id".to_owned()),
            },
        );
    };
    let capabilities: Vec<_> = snapshot
        .capabilities
        .iter()
        .filter(|capability| capability.installed.installed_id == installed_id)
        .map(capability_summary)
        .collect();
    let scope = snapshot
        .capabilities
        .iter()
        .find(|capability| capability.installed.installed_id == installed_id)
        .and_then(|capability| capability.effective_scope.clone());
    let enabled = capabilities.iter().any(|item| item["available"] == true);
    let data = json!({
        "schema": "tethers.plug.show/1",
        "installed_id": record.installed_id,
        "state": if enabled { "enabled" } else { "disabled" },
        "package": {
            "id": record.package_id,
            "version": record.package_version,
            "semantic_digest": record.semantic_package_digest
        },
        "provider": {
            "identity": record.provider_id,
            "version": record.provider_version
        },
        "operational_scope": scope,
        "conformance": {
            "status": "evidence_present",
            "evidence_id": record.conformance_evidence_id,
            "evidence_digest": record.conformance_evidence_digest
        },
        "availability": {
            "configured_bindings": capabilities.iter().filter(|item| item["available"] == true).count(),
            "provider_health_checked": false
        },
        "capabilities": capabilities
    });
    let envelope = CliEnvelope::ok("plug show", data);
    DiscoveryResult {
        exit_code: envelope.exit_code,
        envelope,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn describe_without_configuration_is_deterministic_and_secret_free() {
        let first = run_describe(None);
        let second = run_describe(None);
        assert_eq!(first.exit_code, 0);
        assert_eq!(second.exit_code, 0);
        let first_json = serde_json::to_string(&first.envelope).unwrap();
        let second_json = serde_json::to_string(&second.envelope).unwrap();
        assert_eq!(first_json, second_json);
        assert_eq!(first.envelope.data["schema"], "tethers.describe/1");
        assert_eq!(
            first.envelope.data["host_health"]["host_data_configured"],
            false
        );
        assert_eq!(first.envelope.data["installed_plugs"], 0);
        assert_eq!(first.envelope.data["available_capabilities"], 0);
        assert!(!first_json.contains("OPENAI_API_KEY"));
        assert!(!first_json.contains("private_key"));
    }

    #[test]
    fn discovery_rejects_relative_host_data_root() {
        let result = run_capability_list(Path::new("relative-root"), false, None, None, None);
        assert_eq!(result.envelope.status, OutcomeStatus::InvalidCliUsage);
        assert_eq!(
            result.envelope.error.as_ref().unwrap().field.as_deref(),
            Some("--host-data-root")
        );
    }
}
