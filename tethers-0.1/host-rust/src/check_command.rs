// J13A check command: validate Tether sources, engine, and provider availability.
//
// The check command:
// 1. Resolves and validates paths (config, engine)
// 2. Loads runtime config
// 3. Prepares J12 PreparedRuntime
// 4. Launches retained engine
// 5. Validates all Tethers in configured order
// 6. Launches each configured provider once
// 7. Initializes and lists tools once per provider
// 8. Compares every configured capability against prepared trusted manifests
// 9. Emits one deterministic result
// 10. Closes all children

use crate::configured_runtime::{prepare_runtime, PreparedRuntime};
use crate::runtime_config::{load_runtime_config, LoadedRuntimeConfig};
use crate::stdio_provider::{compare_discovery_evidence, ManagedProvider, StdioProviderError};
use tethers_reference_host::child_process::is_interrupted;
use tethers_reference_host::child_process::set_interrupted;
use tethers_reference_host::cli::{CliEnvelope, OutcomeStatus};
use tethers_reference_host::engine_stdio::EngineSession;

use serde_json::{json, Value};
use std::path::PathBuf;

/// Result of the check command.
pub struct CheckResult {
    pub envelope: CliEnvelope,
    pub exit_code: i32,
}

/// Run the check command.
pub fn run_check(config_path: &std::path::Path, engine_path: &std::path::Path) -> CheckResult {
    // Capture caller CWD exactly once.
    let caller_cwd = match std::env::current_dir() {
        Ok(d) => d,
        Err(e) => {
            return CheckResult {
                envelope: CliEnvelope::error(
                    "check",
                    OutcomeStatus::Failed,
                    "INTERNAL_ERROR",
                    format!("cannot determine current directory: {e}"),
                    None,
                ),
                exit_code: OutcomeStatus::Failed.exit_code(),
            };
        }
    };

    // 1. Resolve and validate paths.
    let (canonical_config, canonical_engine) =
        match resolve_check_paths(&caller_cwd, config_path, engine_path) {
            Ok(p) => p,
            Err(result) => return result,
        };

    // 2. Load runtime config.
    let loaded = match load_runtime_config(&canonical_config) {
        Ok(l) => l,
        Err(e) => {
            return CheckResult {
                envelope: CliEnvelope::error(
                    "check",
                    OutcomeStatus::InvalidData,
                    "CONFIG_LOAD_FAILED",
                    format!("cannot load runtime config: {e}"),
                    Some("/config".to_owned()),
                ),
                exit_code: OutcomeStatus::InvalidData.exit_code(),
            };
        }
    };

    let config_dir = loaded.config_dir.clone();
    let tether_set_id = loaded.config.tether_set.id.clone();
    let tether_set_version = loaded.config.tether_set.version.clone();
    let tether_count = loaded.config.tether_set.tethers.len();
    let provider_count = loaded.config.providers.len();

    // 3. Prepare J12 PreparedRuntime.
    let prepared = match prepare_runtime(&loaded) {
        Ok(p) => p,
        Err(e) => {
            return CheckResult {
                envelope: CliEnvelope::error(
                    "check",
                    OutcomeStatus::InvalidData,
                    "RUNTIME_PREPARE_FAILED",
                    format!("cannot prepare runtime: {e}"),
                    None,
                ),
                exit_code: OutcomeStatus::InvalidData.exit_code(),
            };
        }
    };

    // Check for interruption before engine launch.
    if is_interrupted() {
        return CheckResult {
            envelope: CliEnvelope::error(
                "check",
                OutcomeStatus::Interrupted,
                "INTERRUPTED",
                "interrupted before engine launch",
                None,
            ),
            exit_code: OutcomeStatus::Interrupted.exit_code(),
        };
    }

    // 4. Launch retained engine.
    let mut engine_session = match EngineSession::launch(&canonical_engine, &config_dir) {
        Ok(s) => s,
        Err(e) => {
            let code = match &e {
                tethers_reference_host::engine_stdio::EngineError::Child(_) => {
                    "ENGINE_LAUNCH_FAILED"
                }
                tethers_reference_host::engine_stdio::EngineError::InitializeFailed(_) => {
                    "ENGINE_INITIALIZE_FAILED"
                }
                _ => "ENGINE_ERROR",
            };
            return CheckResult {
                envelope: CliEnvelope::error(
                    "check",
                    OutcomeStatus::Unavailable,
                    code,
                    format!("{e}"),
                    Some("/engine".to_owned()),
                ),
                exit_code: OutcomeStatus::Unavailable.exit_code(),
            };
        }
    };

    // 5. Validate all Tethers in configured order.
    let mut tether_results: Vec<Value> = Vec::new();

    for (index, tether) in prepared.tethers().iter().enumerate() {
        if is_interrupted() {
            engine_session.shutdown();
            return CheckResult {
                envelope: build_intermediate_result(
                    "check",
                    OutcomeStatus::Interrupted,
                    tether_set_id.clone(),
                    tether_set_version.clone(),
                    tether_count,
                    provider_count,
                    tether_results,
                    Vec::new(),
                ),
                exit_code: OutcomeStatus::Interrupted.exit_code(),
            };
        }

        match engine_session.validate_tether(index, &tether.id, &tether.version, &tether.source) {
            Ok(()) => {
                tether_results.push(json!({
                    "index": index,
                    "id": tether.id,
                    "version": tether.version,
                    "status": "valid"
                }));
            }
            Err(e) => {
                tether_results.push(json!({
                    "index": index,
                    "id": tether.id,
                    "version": tether.version,
                    "status": "invalid",
                    "error": e.to_string()
                }));
                // Stop before launching providers.
                engine_session.shutdown();
                return CheckResult {
                    envelope: CliEnvelope::error(
                        "check",
                        OutcomeStatus::InvalidData,
                        "TETHER_INVALID",
                        format!("validation failed at tether {index}: {e}"),
                        Some(format!("/tethers/{}", index)),
                    ),
                    exit_code: OutcomeStatus::InvalidData.exit_code(),
                };
            }
        }
    }

    // 6-9. Launch providers and verify availability.
    let provider_results = match check_providers(&prepared, &config_dir) {
        Ok(results) => results,
        Err(result) => {
            engine_session.shutdown();
            return result;
        }
    };

    // 10. Shut down engine.
    engine_session.shutdown();

    // Success!
    let data = json!({
        "config": {
            "tether_set_id": tether_set_id,
            "tether_set_version": tether_set_version,
            "tether_count": tether_count,
            "provider_count": provider_count
        },
        "tethers": tether_results,
        "providers": provider_results
    });

    CheckResult {
        envelope: CliEnvelope::ok("check", data),
        exit_code: 0,
    }
}

/// Resolve and validate --config and --engine paths relative to caller CWD.
fn resolve_check_paths(
    caller_cwd: &std::path::Path,
    config_path: &std::path::Path,
    engine_path: &std::path::Path,
) -> Result<(PathBuf, PathBuf), CheckResult> {
    // Resolve relative paths against caller CWD.
    let resolved_config = if config_path.is_absolute() {
        config_path.to_path_buf()
    } else {
        caller_cwd.join(config_path)
    };

    let resolved_engine = if engine_path.is_absolute() {
        engine_path.to_path_buf()
    } else {
        caller_cwd.join(engine_path)
    };

    // Canonicalise.
    let canonical_config = match resolved_config.canonicalize() {
        Ok(p) => p,
        Err(e) => {
            return Err(CheckResult {
                envelope: CliEnvelope::error(
                    "check",
                    OutcomeStatus::InvalidData,
                    "CONFIG_NOT_FOUND",
                    format!("config path not found: {e}"),
                    Some("--config".to_owned()),
                ),
                exit_code: OutcomeStatus::InvalidData.exit_code(),
            });
        }
    };

    let canonical_engine = match resolved_engine.canonicalize() {
        Ok(p) => p,
        Err(e) => {
            return Err(CheckResult {
                envelope: CliEnvelope::error(
                    "check",
                    OutcomeStatus::InvalidData,
                    "ENGINE_NOT_FOUND",
                    format!("engine path not found: {e}"),
                    Some("--engine".to_owned()),
                ),
                exit_code: OutcomeStatus::InvalidData.exit_code(),
            });
        }
    };

    // Require regular files.
    if !canonical_config.is_file() {
        return Err(CheckResult {
            envelope: CliEnvelope::error(
                "check",
                OutcomeStatus::InvalidData,
                "CONFIG_IS_DIRECTORY",
                "config path must be a regular file, not a directory".to_owned(),
                Some("--config".to_owned()),
            ),
            exit_code: OutcomeStatus::InvalidData.exit_code(),
        });
    }

    if !canonical_engine.is_file() {
        return Err(CheckResult {
            envelope: CliEnvelope::error(
                "check",
                OutcomeStatus::InvalidData,
                "ENGINE_IS_DIRECTORY",
                "engine path must be a regular file, not a directory".to_owned(),
                Some("--engine".to_owned()),
            ),
            exit_code: OutcomeStatus::InvalidData.exit_code(),
        });
    }

    Ok((canonical_config, canonical_engine))
}

/// Check all providers: launch, initialize, list tools, compare capabilities.
fn check_providers(
    prepared: &PreparedRuntime,
    config_dir: &std::path::Path,
) -> Result<Vec<Value>, CheckResult> {
    let mut provider_results: Vec<Value> = Vec::new();

    for (provider_index, provider) in prepared.providers().iter().enumerate() {
        if is_interrupted() {
            return Err(CheckResult {
                envelope: build_intermediate_result(
                    "check",
                    OutcomeStatus::Interrupted,
                    prepared.tether_set_id().to_owned(),
                    prepared.tether_set_version().to_owned(),
                    prepared.tethers().len(),
                    prepared.providers().len(),
                    // Tether results already all valid.
                    prepared
                        .tethers()
                        .iter()
                        .enumerate()
                        .map(|(i, t)| {
                            json!({
                                "index": i,
                                "id": t.id,
                                "version": t.version,
                                "status": "valid"
                            })
                        })
                        .collect(),
                    provider_results,
                ),
                exit_code: OutcomeStatus::Interrupted.exit_code(),
            });
        }

        let identity = provider.identity.clone();
        let stdio_config = &provider.stdio_config;

        // Launch provider with explicit current dir.
        let mut mcp_provider = match ManagedProvider::launch(
            &stdio_config.command,
            &stdio_config.args,
            &provider.working_directory,
            None,
            None,
        ) {
            Ok(p) => p,
            Err(e) => {
                let result = build_provider_error(
                    provider_index,
                    &identity,
                    "launch_failed",
                    &e.to_string(),
                    prepared.providers().len(),
                );
                provider_results.push(result);
                // Stop on first failure.
                return Err(CheckResult {
                    envelope: CliEnvelope::error(
                        "check",
                        OutcomeStatus::Unavailable,
                        "PROVIDER_LAUNCH_FAILED",
                        format!("provider {provider_index} ({identity}) launch failed: {e}"),
                        Some(format!("/providers/{}", provider_index)),
                    ),
                    exit_code: 4,
                });
            }
        };

        // Initialize.
        let server_name = &stdio_config.provider_config.identity;
        if let Err(e) = mcp_provider.initialize(&stdio_config.protocol_version, server_name) {
            let result = build_provider_error(
                provider_index,
                &identity,
                "initialize_failed",
                &e.to_string(),
                prepared.providers().len(),
            );
            provider_results.push(result);
            mcp_provider.close();
            return Err(CheckResult {
                envelope: CliEnvelope::error(
                    "check",
                    OutcomeStatus::Unavailable,
                    "PROVIDER_INITIALIZE_FAILED",
                    format!("provider {provider_index} ({identity}) initialize failed: {e}"),
                    Some(format!("/providers/{}", provider_index)),
                ),
                exit_code: 4,
            });
        }

        // List tools.
        let tools = match mcp_provider.list_tools() {
            Ok(t) => t,
            Err(e) => {
                let result = build_provider_error(
                    provider_index,
                    &identity,
                    "tools_list_failed",
                    &e.to_string(),
                    prepared.providers().len(),
                );
                provider_results.push(result);
                mcp_provider.close();
                return Err(CheckResult {
                    envelope: CliEnvelope::error(
                        "check",
                        OutcomeStatus::Unavailable,
                        "PROVIDER_TOOLS_LIST_FAILED",
                        format!("provider {provider_index} ({identity}) tools/list failed: {e}"),
                        Some(format!("/providers/{}", provider_index)),
                    ),
                    exit_code: 4,
                });
            }
        };

        // Compare all configured capabilities.
        let mut cap_results: Vec<Value> = Vec::new();
        let mut all_available = true;

        for cap in &provider.capabilities {
            match compare_discovery_evidence(&tools, &cap.verified_manifest) {
                Ok(()) => {
                    cap_results.push(json!({
                        "name": cap.name,
                        "version": cap.version,
                        "status": "available"
                    }));
                }
                Err(e) => {
                    all_available = false;
                    cap_results.push(json!({
                        "name": cap.name,
                        "version": cap.version,
                        "status": "unavailable",
                        "error": e.to_string()
                    }));
                }
            }
        }

        // Close the provider session.
        mcp_provider.close();

        let provider_status = if all_available {
            "available"
        } else {
            "unavailable"
        };
        provider_results.push(json!({
            "index": provider_index,
            "identity": identity,
            "status": provider_status,
            "capabilities": cap_results
        }));

        if !all_available {
            return Err(CheckResult {
                envelope: CliEnvelope::error(
                    "check",
                    OutcomeStatus::Unavailable,
                    "PROVIDER_CAPABILITY_UNAVAILABLE",
                    format!("provider {provider_index} ({identity}) has unavailable capabilities"),
                    Some(format!("/providers/{}", provider_index)),
                ),
                exit_code: 4,
            });
        }
    }

    Ok(provider_results)
}

fn build_provider_error(
    index: usize,
    identity: &str,
    status: &str,
    error: &str,
    _total: usize,
) -> Value {
    json!({
        "index": index,
        "identity": identity,
        "status": status,
        "error": error,
        "capabilities": []
    })
}

fn build_intermediate_result(
    command: &str,
    _status: OutcomeStatus,
    tether_set_id: String,
    tether_set_version: String,
    tether_count: usize,
    provider_count: usize,
    tethers: Vec<Value>,
    providers: Vec<Value>,
) -> CliEnvelope {
    CliEnvelope::ok(
        command,
        json!({
            "config": {
                "tether_set_id": tether_set_id,
                "tether_set_version": tether_set_version,
                "tether_count": tether_count,
                "provider_count": provider_count
            },
            "tethers": tethers,
            "providers": providers,
            "interrupted": true
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn j13a_interrupt_flag_respects_ctrl_c_semantics() {
        // Verify that is_interrupted works correctly even when not set.
        assert!(!is_interrupted());
        set_interrupted();
        assert!(is_interrupted());
        // Reset for other tests.
        tethers_reference_host::child_process::INTERRUPTED
            .store(false, std::sync::atomic::Ordering::Release);
    }
}
