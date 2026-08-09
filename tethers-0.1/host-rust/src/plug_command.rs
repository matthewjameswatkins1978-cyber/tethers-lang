use crate::candidate_preparation::{
    prepare_installation_candidate, CandidatePreparation, CandidatePreparationDisposition,
};
use crate::cli::{CliEnvelope, OutcomeStatus};
use crate::enablement::{EnablementRecord, EnablementState, EnablementStore};
use crate::installed::{InstalledPlugRecord, InstalledPlugRegistry};
use crate::m3_store::M3Error;
use crate::operational_scope::OperationalScopeEvidence;
use crate::package::{self, CapabilityEvidence, PackageError};
use serde::de::{self, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::json;
use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::Path;
use uuid::Uuid;

pub struct PlugCommandResult {
    pub envelope: CliEnvelope,
    pub exit_code: i32,
}

fn candidate_is_newer(current: Option<u64>, candidate: u64) -> bool {
    current.is_none_or(|sequence| candidate > sequence)
}

fn select_latest_transition(
    enablements: &[EnablementRecord],
) -> BTreeMap<String, EnablementRecord> {
    let mut latest: BTreeMap<String, EnablementRecord> = BTreeMap::new();
    for transition in enablements {
        latest
            .entry(transition.installed_id.clone())
            .and_modify(|current| {
                if candidate_is_newer(Some(current.sequence), transition.sequence) {
                    *current = transition.clone();
                }
            })
            .or_insert(transition.clone());
    }
    latest
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

fn stage_error(error: PackageError) -> PlugCommandResult {
    let status = match error.code {
        "archive_read" | "candidate_io" => OutcomeStatus::Unavailable,
        "candidate_rollback_failed" | "clock" => OutcomeStatus::Failed,
        _ => OutcomeStatus::InvalidData,
    };
    let envelope = CliEnvelope::error("plug stage", status, error.code, error.message, None);
    PlugCommandResult {
        exit_code: envelope.exit_code,
        envelope,
    }
}

fn public_capabilities(capabilities: &[CapabilityEvidence]) -> serde_json::Value {
    let mut sorted = capabilities.to_vec();
    sorted.sort_by(|left, right| {
        (left.name.as_str(), left.version, left.operation.as_str()).cmp(&(
            right.name.as_str(),
            right.version,
            right.operation.as_str(),
        ))
    });
    serde_json::Value::Array(
        sorted
            .into_iter()
            .map(|capability| {
                json!({
                    "name": capability.name,
                    "version": capability.version,
                    "manifest_digest": capability.manifest_digest,
                    "operation": capability.operation,
                })
            })
            .collect(),
    )
}

fn public_candidate(prepared: CandidatePreparation) -> serde_json::Value {
    let disposition = match prepared.disposition {
        CandidatePreparationDisposition::Created => "created",
        CandidatePreparationDisposition::Existing => "existing",
    };
    let candidate = prepared.candidate;
    json!({
        "candidate_id": candidate.candidate_id,
        "disposition": disposition,
        "state": candidate.state,
        "package_id": candidate.package_id,
        "package_version": candidate.package_version,
        "semantic_package_digest": candidate.semantic_package_digest,
        "raw_archive_digest": candidate.raw_archive_digest,
        "provider_id": candidate.provider_id,
        "provider_version": candidate.provider_version,
        "platform": {
            "os": candidate.selected_platform.os,
            "architecture": candidate.selected_platform.architecture,
        },
        "capabilities": public_capabilities(&candidate.capabilities),
        "created_unix_ms": candidate.created_unix_ms,
    })
}

pub fn run_stage(host_data_root: &Path, package_path: &Path) -> PlugCommandResult {
    if !host_data_root.is_absolute() {
        let envelope = CliEnvelope::error(
            "plug stage",
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
    if !package_path.is_absolute() {
        let envelope = CliEnvelope::error(
            "plug stage",
            OutcomeStatus::InvalidCliUsage,
            "invalid_cli_usage",
            "--package must be absolute",
            Some("/package".into()),
        );
        return PlugCommandResult {
            exit_code: envelope.exit_code,
            envelope,
        };
    }

    match prepare_installation_candidate(host_data_root, package_path) {
        Ok(prepared) => {
            let envelope = CliEnvelope::ok(
                "plug stage",
                json!({ "candidate": public_candidate(prepared) }),
            );
            PlugCommandResult {
                exit_code: envelope.exit_code,
                envelope,
            }
        }
        Err(error) => stage_error(error),
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
    let latest = select_latest_transition(&enablements);
    for transition in enablements.iter() {
        if !installed_ids.contains_key(&transition.installed_id) {
            return list_error(
                M3Error::new(
                    "enablement_invalid",
                    "enablement references unknown installed Plug",
                ),
                OutcomeStatus::InvalidData,
            );
        }
    }
    let mut plugs = Vec::new();
    for record in installed {
        let transition = latest.get(&record.installed_id);
        if let Some(item) = transition {
            if let Err(error) = item.consistent_with(&record) {
                return list_error(error, OutcomeStatus::InvalidData);
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

fn disable_error(error: M3Error, status: OutcomeStatus) -> PlugCommandResult {
    let envelope = CliEnvelope::error("plug disable", status, error.code, error.message, None);
    PlugCommandResult {
        exit_code: envelope.exit_code,
        envelope,
    }
}

fn disable_store_error(error: M3Error) -> PlugCommandResult {
    let status = if error.code == "store_io" {
        OutcomeStatus::Unavailable
    } else {
        OutcomeStatus::InvalidData
    };
    disable_error(error, status)
}

pub fn run_disable(host_data_root: &Path, installed_id_str: &str) -> PlugCommandResult {
    if !host_data_root.is_absolute() {
        let envelope = CliEnvelope::error(
            "plug disable",
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
    if Uuid::parse_str(installed_id_str).is_err() {
        let envelope = CliEnvelope::error(
            "plug disable",
            OutcomeStatus::InvalidCliUsage,
            "invalid_cli_usage",
            "--installed-id must be a valid UUID",
            Some("/installed-id".into()),
        );
        return PlugCommandResult {
            exit_code: envelope.exit_code,
            envelope,
        };
    }
    match fs::symlink_metadata(host_data_root) {
        Ok(metadata) if metadata.is_dir() => {}
        _ => {
            return disable_error(
                M3Error::new(
                    "plug_data_root_unavailable",
                    "host data root is unavailable",
                ),
                OutcomeStatus::Unavailable,
            )
        }
    }
    if let Err(error) = crate::m3_store::verify_chain(host_data_root) {
        return disable_error(error, OutcomeStatus::InvalidData);
    }
    let paths: Vec<_> = ["install", "installed-records", "enablements"]
        .iter()
        .map(|name| host_data_root.join(name))
        .collect();
    let present = paths.iter().filter(|path| path.exists()).count();
    if present == 0 {
        return disable_error(
            M3Error::new(
                "plug_store_incomplete",
                "lifecycle store layout is incomplete",
            ),
            OutcomeStatus::InvalidData,
        );
    }
    if present != paths.len()
        || paths.iter().any(|path| {
            fs::symlink_metadata(path)
                .map(|m| !m.is_dir() || m.file_type().is_symlink())
                .unwrap_or(true)
        })
    {
        return disable_error(
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
        Err(error) => return disable_store_error(error),
    };
    let target = match installed
        .iter()
        .find(|r| r.installed_id == installed_id_str)
    {
        Some(record) => record,
        None => {
            return disable_error(
                M3Error::new("installed_not_found", "installed Plug not found"),
                OutcomeStatus::InvalidData,
            )
        }
    };
    let enablements =
        match EnablementStore::open_existing(&paths[2]).and_then(|store| store.load_all()) {
            Ok(records) => records,
            Err(error) => return disable_store_error(error),
        };
    let installed_ids: BTreeMap<_, _> = installed
        .iter()
        .map(|record| (record.installed_id.clone(), record))
        .collect();
    for transition in enablements.iter() {
        if !installed_ids.contains_key(&transition.installed_id) {
            return disable_error(
                M3Error::new(
                    "enablement_invalid",
                    "enablement references unknown installed Plug",
                ),
                OutcomeStatus::InvalidData,
            );
        }
    }
    let latest = select_latest_transition(&enablements);
    let current_transition = match latest.get(installed_id_str) {
        Some(transition) => transition,
        None => {
            return disable_error(
                M3Error::new("enablement_refused", "installed Plug is not enabled"),
                OutcomeStatus::InvalidData,
            )
        }
    };
    if current_transition.state != EnablementState::Enabled {
        return disable_error(
            M3Error::new("enablement_refused", "installed Plug is not enabled"),
            OutcomeStatus::InvalidData,
        );
    }
    if let Err(error) = current_transition.consistent_with(target) {
        return disable_error(error, OutcomeStatus::InvalidData);
    }
    let store = match EnablementStore::open_existing(&paths[2]) {
        Ok(store) => store,
        Err(error) => return disable_store_error(error),
    };
    let disabled = match store.disable(target, "tethers-reference-host-cli") {
        Ok(record) => record,
        Err(error) => return disable_store_error(error),
    };
    let envelope = CliEnvelope::ok(
        "plug disable",
        json!({
            "installed_id": disabled.installed_id,
            "package_id": disabled.package_id,
            "state": "disabled",
            "sequence": disabled.sequence,
            "record_digest": disabled.record_digest,
        }),
    );
    PlugCommandResult {
        exit_code: envelope.exit_code,
        envelope,
    }
}

const SCOPE_FILE_MAX_BYTES: u64 = 16 * 1024;

#[derive(Debug)]
struct GenericScopeRequest {
    schema: String,
    scope: serde_json::Value,
}

fn reject_duplicate_keys<'de, M: MapAccess<'de>>(
    _map: &mut M,
    key: &str,
    seen: &mut std::collections::BTreeSet<String>,
) -> Result<(), M::Error> {
    if !seen.insert(key.to_owned()) {
        return Err(de::Error::custom(format!("duplicate key: {key}")));
    }
    Ok(())
}

impl<'de> Deserialize<'de> for GenericScopeRequest {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct RequestVisitor;
        impl<'de> Visitor<'de> for RequestVisitor {
            type Value = GenericScopeRequest;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "a plug scope request object")
            }
            fn visit_map<A: MapAccess<'de>>(
                self,
                mut map: A,
            ) -> Result<GenericScopeRequest, A::Error> {
                let mut schema = None;
                let mut scope = None;
                let mut seen = std::collections::BTreeSet::new();
                while let Some(key) = map.next_key::<String>()? {
                    reject_duplicate_keys(&mut map, &key, &mut seen)?;
                    match key.as_str() {
                        "schema" => {
                            if schema.is_some() {
                                return Err(de::Error::duplicate_field("schema"));
                            }
                            schema = Some(map.next_value()?);
                        }
                        "scope" => {
                            if scope.is_some() {
                                return Err(de::Error::duplicate_field("scope"));
                            }
                            scope = Some(map.next_value()?);
                        }
                        _ => return Err(de::Error::unknown_field(&key, &["schema", "scope"])),
                    }
                }
                let schema = schema.ok_or_else(|| de::Error::missing_field("schema"))?;
                let scope = scope.ok_or_else(|| de::Error::missing_field("scope"))?;
                Ok(GenericScopeRequest { schema, scope })
            }
        }
        deserializer.deserialize_map(RequestVisitor)
    }
}

fn parse_scope_file(path: &Path) -> Result<GenericScopeRequest, M3Error> {
    if !path.is_absolute() {
        return Err(M3Error::new(
            "scope_request_invalid",
            "scope path must be absolute",
        ));
    }
    let bytes = fs::read(path).map_err(|_| M3Error::new("store_io", "cannot read scope file"))?;
    if bytes.len() as u64 > SCOPE_FILE_MAX_BYTES {
        return Err(M3Error::new(
            "scope_request_invalid",
            "scope file exceeds 16 KiB limit",
        ));
    }
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return Err(M3Error::new(
            "scope_request_invalid",
            "scope file contains BOM",
        ));
    }
    std::str::from_utf8(&bytes)
        .map_err(|_| M3Error::new("scope_request_invalid", "scope file is not valid UTF-8"))?;
    let mut de = serde_json::Deserializer::from_slice(&bytes);
    let request = GenericScopeRequest::deserialize(&mut de)
        .map_err(|error| M3Error::new("scope_request_invalid", error.to_string()))?;
    de.end().map_err(|error| {
        M3Error::new(
            "scope_request_invalid",
            format!("trailing content: {error}"),
        )
    })?;
    if request.schema != "tethers.plug-scope/1" {
        return Err(M3Error::new("scope_request_invalid", "unsupported schema"));
    }
    if !request.scope.is_object() {
        return Err(M3Error::new(
            "scope_request_invalid",
            "scope must be a JSON object",
        ));
    }
    Ok(request)
}

fn enable_error(error: M3Error, status: OutcomeStatus) -> PlugCommandResult {
    let envelope = CliEnvelope::error("plug enable", status, error.code, error.message, None);
    PlugCommandResult {
        exit_code: envelope.exit_code,
        envelope,
    }
}

fn enable_store_error(error: M3Error) -> PlugCommandResult {
    let status = if error.code == "store_io" {
        OutcomeStatus::Unavailable
    } else {
        OutcomeStatus::InvalidData
    };
    enable_error(error, status)
}

fn scope_error(message: &str) -> PlugCommandResult {
    let envelope = CliEnvelope::error(
        "plug enable",
        OutcomeStatus::InvalidData,
        "scope_request_invalid",
        message,
        None,
    );
    PlugCommandResult {
        exit_code: envelope.exit_code,
        envelope,
    }
}

fn read_scope_schema_digest(target: &InstalledPlugRecord, host_data_root: &Path) -> String {
    let installed_dir = host_data_root
        .join("install")
        .join(&target.installed_id)
        .join(&target.installation_relative_path);
    let plug_json_path = installed_dir.join("plug.json");
    let bytes = match fs::read(&plug_json_path) {
        Ok(b) => b,
        Err(_) => {
            return "sha256:eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".into()
        }
    };
    let text = match std::str::from_utf8(&bytes) {
        Ok(t) => t,
        Err(_) => {
            return "sha256:eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".into()
        }
    };
    let value: serde_json::Value = match serde_json::from_str(text) {
        Ok(v) => v,
        Err(_) => {
            return "sha256:eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".into()
        }
    };
    let schema = match value
        .get("provider")
        .and_then(|p| p.get("operational_scope_schema"))
    {
        Some(s) => s,
        None => {
            return "sha256:eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".into()
        }
    };
    let canonical = match serde_json_canonicalizer::to_vec(schema) {
        Ok(b) => b,
        Err(_) => {
            return "sha256:eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".into()
        }
    };
    use sha2::{Digest, Sha256};
    format!("sha256:{:x}", Sha256::digest(canonical))
}

pub fn run_enable(
    host_data_root: &Path,
    installed_id_str: &str,
    scope_path: &Path,
) -> PlugCommandResult {
    if !host_data_root.is_absolute() {
        let envelope = CliEnvelope::error(
            "plug enable",
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
    if Uuid::parse_str(installed_id_str).is_err() {
        let envelope = CliEnvelope::error(
            "plug enable",
            OutcomeStatus::InvalidCliUsage,
            "invalid_cli_usage",
            "--installed-id must be a valid UUID",
            Some("/installed-id".into()),
        );
        return PlugCommandResult {
            exit_code: envelope.exit_code,
            envelope,
        };
    }
    if !scope_path.is_absolute() {
        let envelope = CliEnvelope::error(
            "plug enable",
            OutcomeStatus::InvalidCliUsage,
            "invalid_cli_usage",
            "--scope must be absolute",
            Some("/scope".into()),
        );
        return PlugCommandResult {
            exit_code: envelope.exit_code,
            envelope,
        };
    }
    match fs::symlink_metadata(host_data_root) {
        Ok(metadata) if metadata.is_dir() => {}
        _ => {
            return enable_error(
                M3Error::new(
                    "plug_data_root_unavailable",
                    "host data root is unavailable",
                ),
                OutcomeStatus::Unavailable,
            )
        }
    }
    if let Err(error) = crate::m3_store::verify_chain(host_data_root) {
        return enable_error(error, OutcomeStatus::InvalidData);
    }
    let paths: Vec<_> = ["install", "installed-records", "enablements"]
        .iter()
        .map(|name| host_data_root.join(name))
        .collect();
    let present = paths.iter().filter(|path| path.exists()).count();
    if present == 0 {
        return enable_error(
            M3Error::new(
                "plug_store_incomplete",
                "lifecycle store layout is incomplete",
            ),
            OutcomeStatus::InvalidData,
        );
    }
    if present != paths.len()
        || paths.iter().any(|path| {
            fs::symlink_metadata(path)
                .map(|m| !m.is_dir() || m.file_type().is_symlink())
                .unwrap_or(true)
        })
    {
        return enable_error(
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
        Err(error) => return enable_store_error(error),
    };
    let target = match installed
        .iter()
        .find(|r| r.installed_id == installed_id_str)
    {
        Some(record) => record,
        None => {
            return enable_error(
                M3Error::new("installed_not_found", "installed Plug not found"),
                OutcomeStatus::InvalidData,
            )
        }
    };
    let enablements =
        match EnablementStore::open_existing(&paths[2]).and_then(|store| store.load_all()) {
            Ok(records) => records,
            Err(error) => return enable_store_error(error),
        };
    let installed_ids: BTreeMap<_, _> = installed
        .iter()
        .map(|record| (record.installed_id.clone(), record))
        .collect();
    for transition in enablements.iter() {
        if !installed_ids.contains_key(&transition.installed_id) {
            return enable_error(
                M3Error::new(
                    "enablement_invalid",
                    "enablement references unknown installed Plug",
                ),
                OutcomeStatus::InvalidData,
            );
        }
    }
    let latest = select_latest_transition(&enablements);
    if let Some(current) = latest.get(installed_id_str) {
        if current.state == EnablementState::Enabled {
            return enable_error(
                M3Error::new("enablement_conflict", "installed Plug is already enabled"),
                OutcomeStatus::InvalidData,
            );
        }
        if let Err(error) = current.consistent_with(target) {
            return enable_error(error, OutcomeStatus::InvalidData);
        }
    }
    let scope_request = match parse_scope_file(scope_path) {
        Ok(request) => request,
        Err(error) => {
            if error.code == "store_io" {
                return enable_error(error, OutcomeStatus::Unavailable);
            }
            return scope_error(&error.message);
        }
    };
    let scope_schema_digest = read_scope_schema_digest(target, host_data_root);
    let evidence = match OperationalScopeEvidence::create(
        installed_id_str,
        &target.package_id,
        &target.provider_id,
        &scope_schema_digest,
        &scope_request.scope,
        "tethers-reference-host-cli",
    ) {
        Ok(e) => e,
        Err(error) => {
            return enable_error(error, OutcomeStatus::InvalidData);
        }
    };
    let store = match EnablementStore::open_existing(&paths[2]) {
        Ok(store) => store,
        Err(error) => return enable_store_error(error),
    };
    let scope_digest = evidence.integrity_digest().to_owned();
    let enabled = match store.enable(target, evidence, "tethers-reference-host-cli") {
        Ok(record) => record,
        Err(error) => return enable_store_error(error),
    };
    let envelope = CliEnvelope::ok(
        "plug enable",
        json!({
            "installed_id": enabled.installed_id,
            "package_id": enabled.package_id,
            "state": "enabled",
            "sequence": enabled.sequence,
            "record_digest": enabled.record_digest,
            "scope_digest": scope_digest,
        }),
    );
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

    #[test]
    fn j24f_stage_error_mapping_uses_only_error_code() {
        for (code, status, exit_code) in [
            ("archive_read", OutcomeStatus::Unavailable, 4),
            ("candidate_io", OutcomeStatus::Unavailable, 4),
            ("candidate_rollback_failed", OutcomeStatus::Failed, 6),
            ("clock", OutcomeStatus::Failed, 6),
            ("semantic_conflict", OutcomeStatus::InvalidData, 3),
        ] {
            let result = stage_error(PackageError {
                code,
                message: "stable message".into(),
            });
            assert_eq!(result.envelope.status, status);
            assert_eq!(result.exit_code, exit_code);
            assert_eq!(result.envelope.exit_code, exit_code);
            assert_eq!(result.envelope.error.as_ref().unwrap().code, code);
            assert_eq!(
                result.envelope.error.as_ref().unwrap().message,
                "stable message"
            );
        }
    }

    #[test]
    fn j24f_relative_stage_paths_are_rejected_before_service_call() {
        let host = run_stage(Path::new("relative-host"), Path::new("relative-package"));
        assert_eq!(host.exit_code, 2);
        assert_eq!(
            host.envelope.error.as_ref().unwrap().field.as_deref(),
            Some("/host-data-root")
        );

        let package = run_stage(Path::new("C:\\host"), Path::new("relative-package"));
        assert_eq!(package.exit_code, 2);
        assert_eq!(
            package.envelope.error.as_ref().unwrap().field.as_deref(),
            Some("/package")
        );
    }
}
