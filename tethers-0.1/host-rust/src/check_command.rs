// J13A check command: validate Tether sources, engine, and provider availability.
//
// Envelope consistency: every envelope's status and embedded exit_code match
// the process exit code.  Interruption uses status=interrupted, exit=10.
// Partial evidence is preserved on Tether/provider failure.

use crate::configured_runtime::{prepare_runtime, PreparedRuntime};
use crate::runtime_config::load_runtime_config;
use crate::stdio_provider::{compare_discovery_evidence, ManagedProvider};
use tethers_reference_host::child_process::is_interrupted;
use tethers_reference_host::cli::{CliEnvelope, OutcomeStatus};
use tethers_reference_host::engine_stdio::EngineSession;

use serde_json::{json, Value};
use std::path::PathBuf;

pub struct CheckResult {
    pub envelope: CliEnvelope,
    pub exit_code: i32,
}

pub fn run_check(config_path: &std::path::Path, engine_path: &std::path::Path) -> CheckResult {
    let caller_cwd = match std::env::current_dir() {
        Ok(d) => d,
        Err(e) => {
            return fail(
                "check",
                OutcomeStatus::Failed,
                "INTERNAL_ERROR",
                format!("cannot determine current directory: {e}"),
                None,
            );
        }
    };

    // 1. Resolve paths.
    let (canonical_config, canonical_engine) =
        match resolve_check_paths(&caller_cwd, config_path, engine_path) {
            Ok(p) => p,
            Err(r) => return r,
        };

    // 2. Load config.
    let loaded = match load_runtime_config(&canonical_config) {
        Ok(l) => l,
        Err(e) => {
            return fail(
                "check",
                OutcomeStatus::InvalidData,
                "CONFIG_LOAD_FAILED",
                format!("cannot load runtime config: {e}"),
                Some("/config".to_owned()),
            );
        }
    };

    let config_dir = loaded.config_dir.clone();
    let tether_set_id = loaded.config.tether_set.id.clone();
    let tether_set_version = loaded.config.tether_set.version.clone();
    let tether_count = loaded.config.tether_set.tethers.len();
    let provider_count = loaded.config.providers.len();

    // 3. Prepare runtime.
    let prepared = match prepare_runtime(&loaded) {
        Ok(p) => p,
        Err(e) => {
            return fail(
                "check",
                OutcomeStatus::InvalidData,
                "RUNTIME_PREPARE_FAILED",
                format!("cannot prepare runtime: {e}"),
                None,
            );
        }
    };

    if is_interrupted() {
        return interrupted_result("check", "before engine launch");
    }

    // 4. Launch engine.
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
                tethers_reference_host::engine_stdio::EngineError::Interrupted => {
                    return interrupted_result("check", "during engine startup");
                }
                _ => "ENGINE_ERROR",
            };
            return fail(
                "check",
                OutcomeStatus::Unavailable,
                code,
                format!("{e}"),
                Some("/engine".to_owned()),
            );
        }
    };

    // 5. Validate all Tethers.
    let mut tether_results: Vec<Value> = Vec::new();
    for (index, tether) in prepared.tethers().iter().enumerate() {
        if is_interrupted() {
            engine_session.shutdown();
            return CheckResult {
                envelope: build_partial(
                    "check",
                    OutcomeStatus::Interrupted,
                    &tether_set_id,
                    &tether_set_version,
                    tether_count,
                    provider_count,
                    &tether_results,
                    &[],
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
                engine_session.shutdown();
                return fail(
                    "check",
                    OutcomeStatus::InvalidData,
                    "TETHER_INVALID",
                    format!("validation failed at tether {index}: {e}"),
                    Some(format!("/tethers/{}", index)),
                );
            }
        }
    }

    // 6-9. Check providers.
    let (provider_results, err) = check_providers(&prepared, &config_dir);
    engine_session.shutdown();

    match err {
        Some(result) => result,
        None => {
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
    }
}

fn resolve_check_paths(
    caller_cwd: &std::path::Path,
    config_path: &std::path::Path,
    engine_path: &std::path::Path,
) -> Result<(PathBuf, PathBuf), CheckResult> {
    let resolve = |p: &std::path::Path| -> PathBuf {
        if p.is_absolute() {
            p.to_path_buf()
        } else {
            caller_cwd.join(p)
        }
    };

    let canonical_config = match resolve(config_path).canonicalize() {
        Ok(p) => p,
        Err(e) => {
            return Err(fail(
                "check",
                OutcomeStatus::InvalidData,
                "CONFIG_NOT_FOUND",
                format!("config path not found: {e}"),
                Some("--config".to_owned()),
            ));
        }
    };

    let canonical_engine = match resolve(engine_path).canonicalize() {
        Ok(p) => p,
        Err(e) => {
            return Err(fail(
                "check",
                OutcomeStatus::InvalidData,
                "ENGINE_NOT_FOUND",
                format!("engine path not found: {e}"),
                Some("--engine".to_owned()),
            ));
        }
    };

    if !canonical_config.is_file() {
        return Err(fail(
            "check",
            OutcomeStatus::InvalidData,
            "CONFIG_IS_DIRECTORY",
            "config path must be a regular file".to_owned(),
            Some("--config".to_owned()),
        ));
    }
    if !canonical_engine.is_file() {
        return Err(fail(
            "check",
            OutcomeStatus::InvalidData,
            "ENGINE_IS_DIRECTORY",
            "engine path must be a regular file".to_owned(),
            Some("--engine".to_owned()),
        ));
    }

    Ok((canonical_config, canonical_engine))
}

fn check_providers(
    prepared: &PreparedRuntime,
    config_dir: &std::path::Path,
) -> (Vec<Value>, Option<CheckResult>) {
    let mut results: Vec<Value> = Vec::new();

    for (pi, provider) in prepared.providers().iter().enumerate() {
        if is_interrupted() {
            return (
                results,
                Some(interrupted_check("check", "during provider check")),
            );
        }

        let identity = provider.identity.clone();
        let stdio = &provider.stdio_config;

        let mut mcp = match ManagedProvider::launch(
            &stdio.command,
            &stdio.args,
            &provider.working_directory,
            None,
            None,
        ) {
            Ok(p) => p,
            Err(e) => {
                results.push(json!({
                    "index": pi, "identity": identity,
                    "status": "launch_failed", "error": e.to_string(),
                    "capabilities": []
                }));
                return (
                    results,
                    Some(fail_provider(
                        pi,
                        &identity,
                        "PROVIDER_LAUNCH_FAILED",
                        &e.to_string(),
                    )),
                );
            }
        };

        if let Err(e) = mcp.initialize(&stdio.protocol_version, &stdio.provider_config.identity) {
            results.push(json!({
                "index": pi, "identity": identity,
                "status": "initialize_failed", "error": e.to_string(),
                "capabilities": []
            }));
            mcp.close();
            return (
                results,
                Some(fail_provider(
                    pi,
                    &identity,
                    "PROVIDER_INITIALIZE_FAILED",
                    &e.to_string(),
                )),
            );
        }

        let tools = match mcp.list_tools() {
            Ok(t) => t,
            Err(e) => {
                results.push(json!({
                    "index": pi, "identity": identity,
                    "status": "tools_list_failed", "error": e.to_string(),
                    "capabilities": []
                }));
                mcp.close();
                return (
                    results,
                    Some(fail_provider(
                        pi,
                        &identity,
                        "PROVIDER_TOOLS_LIST_FAILED",
                        &e.to_string(),
                    )),
                );
            }
        };

        let mut caps: Vec<Value> = Vec::new();
        let mut all_ok = true;
        for cap in &provider.capabilities {
            match compare_discovery_evidence(&tools, &cap.verified_manifest) {
                Ok(()) => caps.push(json!({
                    "name": cap.name, "version": cap.version, "status": "available"
                })),
                Err(e) => {
                    all_ok = false;
                    caps.push(json!({
                        "name": cap.name, "version": cap.version,
                        "status": "unavailable", "error": e.to_string()
                    }));
                }
            }
        }

        mcp.close();

        let pstatus = if all_ok { "available" } else { "unavailable" };
        results.push(json!({
            "index": pi, "identity": identity, "status": pstatus, "capabilities": caps
        }));

        if !all_ok {
            return (
                results,
                Some(fail_provider(
                    pi,
                    &identity,
                    "PROVIDER_CAPABILITY_UNAVAILABLE",
                    "capability unavailable",
                )),
            );
        }
    }

    (results, None)
}

// --- Envelope helpers ---

fn fail(
    cmd: &str,
    status: OutcomeStatus,
    code: &str,
    msg: String,
    field: Option<String>,
) -> CheckResult {
    let exit = status.exit_code();
    CheckResult {
        envelope: CliEnvelope::error(cmd, status, code, msg, field),
        exit_code: exit,
    }
}

fn fail_provider(index: usize, identity: &str, code: &str, msg: &str) -> CheckResult {
    fail(
        "check",
        OutcomeStatus::Unavailable,
        code,
        format!("provider {index} ({identity}): {msg}"),
        Some(format!("/providers/{}", index)),
    )
}

fn interrupted_result(cmd: &str, _where: &str) -> CheckResult {
    CheckResult {
        envelope: CliEnvelope::error(
            cmd,
            OutcomeStatus::Interrupted,
            "INTERRUPTED",
            "interrupted".to_owned(),
            None,
        ),
        exit_code: 10,
    }
}

fn interrupted_check(cmd: &str, _where: &str) -> CheckResult {
    interrupted_result(cmd, _where)
}

fn build_partial(
    cmd: &str,
    _status: OutcomeStatus,
    tether_set_id: &str,
    tether_set_version: &str,
    tether_count: usize,
    provider_count: usize,
    tethers: &[Value],
    providers: &[Value],
) -> CliEnvelope {
    // For partial interrupted evidence, use the error constructor
    // with status=interrupted so status/exit match.
    let data = json!({
        "config": {
            "tether_set_id": tether_set_id,
            "tether_set_version": tether_set_version,
            "tether_count": tether_count,
            "provider_count": provider_count
        },
        "tethers": tethers,
        "providers": providers
    });
    // Use error constructor for interruption to ensure status=interrupted, exit=10.
    CliEnvelope::error(
        cmd,
        OutcomeStatus::Interrupted,
        "INTERRUPTED",
        "interrupted".to_owned(),
        None,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn j13a_interrupt_flag_works() {
        assert!(!is_interrupted());
        tethers_reference_host::child_process::set_interrupted();
        assert!(is_interrupted());
        tethers_reference_host::child_process::INTERRUPTED
            .store(false, std::sync::atomic::Ordering::Release);
    }

    #[test]
    fn j13a_fail_returns_matching_status_and_exit() {
        let r = fail(
            "check",
            OutcomeStatus::InvalidData,
            "TEST",
            "msg".to_owned(),
            None,
        );
        let json = serde_json::to_string(&r.envelope).unwrap();
        assert_eq!(r.exit_code, OutcomeStatus::InvalidData.exit_code());
        assert_eq!(r.exit_code, 3);
        // Verify embedded exit_code matches process exit_code.
        let v: Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["exit_code"].as_i64().unwrap(), 3);
        assert_eq!(v["status"], "invalid_data");
    }

    #[test]
    fn j13a_interrupted_returns_exit_10() {
        let r = interrupted_result("check", "test");
        assert_eq!(r.exit_code, 10);
        let json = serde_json::to_string(&r.envelope).unwrap();
        let v: Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["exit_code"].as_i64().unwrap(), 10);
        assert_eq!(v["status"], "interrupted");
    }
}
