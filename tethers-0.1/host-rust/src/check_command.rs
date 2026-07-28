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
use tethers_reference_host::engine_stdio::{EngineError, EngineSession};

use serde_json::{json, Value};
use std::path::PathBuf;

pub struct CheckResult {
    pub envelope: CliEnvelope,
    pub exit_code: i32,
}

#[derive(Debug)]
struct CheckFailure {
    status: OutcomeStatus,
    code: &'static str,
    message: String,
    field: Option<String>,
}

impl CheckFailure {
    fn new(
        status: OutcomeStatus,
        code: &'static str,
        message: impl Into<String>,
        field: Option<String>,
    ) -> Self {
        Self {
            status,
            code,
            message: message.into(),
            field,
        }
    }

    fn provider(index: usize, identity: &str, code: &'static str, message: &str) -> Self {
        Self::new(
            OutcomeStatus::Unavailable,
            code,
            format!("provider {index} ({identity}): {message}"),
            Some(format!("/providers/{index}")),
        )
    }

    fn interrupted() -> Self {
        Self::new(
            OutcomeStatus::Interrupted,
            "INTERRUPTED",
            "interrupted",
            None,
        )
    }
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
        let data = build_check_data(
            &tether_set_id,
            &tether_set_version,
            tether_count,
            provider_count,
            &[],
            &[],
        );
        return fail_with_data("check", CheckFailure::interrupted(), data);
    }

    // 4. Launch engine.
    let mut engine_session = match EngineSession::launch(&canonical_engine, &config_dir) {
        Ok(s) => s,
        Err(e) => {
            let failure = match &e {
                EngineError::Interrupted => CheckFailure::interrupted(),
                EngineError::Child(_) => CheckFailure::new(
                    OutcomeStatus::Unavailable,
                    "ENGINE_LAUNCH_FAILED",
                    e.to_string(),
                    Some("/engine".to_owned()),
                ),
                EngineError::InitializeFailed(_) => CheckFailure::new(
                    OutcomeStatus::Unavailable,
                    "ENGINE_INITIALIZE_FAILED",
                    e.to_string(),
                    Some("/engine".to_owned()),
                ),
                _ => CheckFailure::new(
                    OutcomeStatus::Unavailable,
                    "ENGINE_ERROR",
                    e.to_string(),
                    Some("/engine".to_owned()),
                ),
            };
            let data = build_check_data(
                &tether_set_id,
                &tether_set_version,
                tether_count,
                provider_count,
                &[],
                &[],
            );
            return fail_with_data("check", failure, data);
        }
    };

    // 5. Validate all Tethers.
    let mut tether_results: Vec<Value> = Vec::new();
    for (index, tether) in prepared.tethers().iter().enumerate() {
        if is_interrupted() {
            engine_session.shutdown();
            let data = build_check_data(
                &tether_set_id,
                &tether_set_version,
                tether_count,
                provider_count,
                &tether_results,
                &[],
            );
            return fail_with_data("check", CheckFailure::interrupted(), data);
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
            Err(EngineError::Interrupted) => {
                engine_session.shutdown();
                let data = build_check_data(
                    &tether_set_id,
                    &tether_set_version,
                    tether_count,
                    provider_count,
                    &tether_results,
                    &[],
                );
                return fail_with_data("check", CheckFailure::interrupted(), data);
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
                let data = build_check_data(
                    &tether_set_id,
                    &tether_set_version,
                    tether_count,
                    provider_count,
                    &tether_results,
                    &[],
                );
                return fail_with_data(
                    "check",
                    CheckFailure::new(
                        OutcomeStatus::InvalidData,
                        "TETHER_INVALID",
                        format!("validation failed at tether {index}: {e}"),
                        Some(format!("/tethers/{index}")),
                    ),
                    data,
                );
            }
        }
    }

    // 6-9. Check providers.
    let (provider_results, failure) = check_providers(&prepared);
    engine_session.shutdown();

    let data = build_check_data(
        &tether_set_id,
        &tether_set_version,
        tether_count,
        provider_count,
        &tether_results,
        &provider_results,
    );
    match failure {
        Some(failure) => fail_with_data("check", failure, data),
        None => CheckResult {
            envelope: CliEnvelope::ok("check", data),
            exit_code: 0,
        },
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

fn check_providers(prepared: &PreparedRuntime) -> (Vec<Value>, Option<CheckFailure>) {
    let mut results: Vec<Value> = Vec::new();

    for (pi, provider) in prepared.providers().iter().enumerate() {
        if is_interrupted() {
            return (results, Some(CheckFailure::interrupted()));
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
                if is_interrupted() {
                    return (results, Some(CheckFailure::interrupted()));
                }
                results.push(json!({
                    "index": pi, "identity": identity,
                    "status": "launch_failed", "error": e.to_string(),
                    "capabilities": []
                }));
                return (
                    results,
                    Some(CheckFailure::provider(
                        pi,
                        &identity,
                        "PROVIDER_LAUNCH_FAILED",
                        &e.to_string(),
                    )),
                );
            }
        };

        if let Err(e) = mcp.initialize(&stdio.protocol_version, &stdio.provider_config.identity) {
            if is_interrupted() {
                mcp.close();
                return (results, Some(CheckFailure::interrupted()));
            }
            results.push(json!({
                "index": pi, "identity": identity,
                "status": "initialize_failed", "error": e.to_string(),
                "capabilities": []
            }));
            mcp.close();
            return (
                results,
                Some(CheckFailure::provider(
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
                if is_interrupted() {
                    mcp.close();
                    return (results, Some(CheckFailure::interrupted()));
                }
                results.push(json!({
                    "index": pi, "identity": identity,
                    "status": "tools_list_failed", "error": e.to_string(),
                    "capabilities": []
                }));
                mcp.close();
                return (
                    results,
                    Some(CheckFailure::provider(
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
                Some(CheckFailure::provider(
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

fn build_check_data(
    tether_set_id: &str,
    tether_set_version: &str,
    tether_count: usize,
    provider_count: usize,
    tethers: &[Value],
    providers: &[Value],
) -> Value {
    json!({
        "config": {
            "tether_set_id": tether_set_id,
            "tether_set_version": tether_set_version,
            "tether_count": tether_count,
            "provider_count": provider_count
        },
        "tethers": tethers,
        "providers": providers
    })
}

fn fail_with_data(cmd: &str, failure: CheckFailure, data: Value) -> CheckResult {
    let exit_code = failure.status.exit_code();
    CheckResult {
        envelope: CliEnvelope::error_with_data(
            cmd,
            failure.status,
            failure.code,
            failure.message,
            failure.field,
            data,
        ),
        exit_code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn envelope_value(result: &CheckResult) -> Value {
        serde_json::to_value(&result.envelope).unwrap()
    }

    fn test_data(tethers: &[Value], providers: &[Value]) -> Value {
        build_check_data("test.set", "1", 2, 2, tethers, providers)
    }

    fn test_failure(
        status: OutcomeStatus,
        code: &'static str,
        field: Option<&str>,
        data: Value,
    ) -> CheckResult {
        fail_with_data(
            "check",
            CheckFailure::new(status, code, "test failure", field.map(str::to_owned)),
            data,
        )
    }

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
    fn j13a_canonical_success_data_contains_config_tethers_and_providers() {
        let tethers = vec![json!({"index": 0, "status": "valid"})];
        let providers = vec![json!({"index": 0, "status": "available"})];
        let data = build_check_data("test.set", "7", 1, 1, &tethers, &providers);
        let result = CheckResult {
            envelope: CliEnvelope::ok("check", data),
            exit_code: 0,
        };
        let value = envelope_value(&result);

        assert_eq!(value["data"]["config"]["tether_set_id"], "test.set");
        assert_eq!(value["data"]["config"]["tether_set_version"], "7");
        assert_eq!(value["data"]["config"]["tether_count"], 1);
        assert_eq!(value["data"]["config"]["provider_count"], 1);
        assert_eq!(value["data"]["tethers"], json!(tethers));
        assert_eq!(value["data"]["providers"], json!(providers));
    }

    #[test]
    fn j13a_invalid_first_tether_preserves_failed_tether_and_no_providers() {
        let tethers = vec![json!({
            "index": 0, "id": "bad", "version": "1",
            "status": "invalid", "error": "parse failed"
        })];
        let result = test_failure(
            OutcomeStatus::InvalidData,
            "TETHER_INVALID",
            Some("/tethers/0"),
            test_data(&tethers, &[]),
        );
        let value = envelope_value(&result);

        assert_eq!(value["status"], "invalid_data");
        assert_eq!(value["data"]["tethers"], json!(tethers));
        assert_eq!(value["data"]["providers"], json!([]));
        assert_eq!(value["error"]["field"], "/tethers/0");
    }

    #[test]
    fn j13a_invalid_later_tether_preserves_earlier_valid_tethers() {
        let tethers = vec![
            json!({"index": 0, "id": "good", "version": "1", "status": "valid"}),
            json!({
                "index": 1, "id": "bad", "version": "1",
                "status": "invalid", "error": "parse failed"
            }),
        ];
        let result = test_failure(
            OutcomeStatus::InvalidData,
            "TETHER_INVALID",
            Some("/tethers/1"),
            test_data(&tethers, &[]),
        );
        let value = envelope_value(&result);

        assert_eq!(value["data"]["tethers"][0]["status"], "valid");
        assert_eq!(value["data"]["tethers"][1]["status"], "invalid");
        assert_eq!(value["data"]["providers"], json!([]));
    }

    #[test]
    fn j13a_provider_launch_failure_preserves_all_tether_evidence() {
        let tethers = vec![
            json!({"index": 0, "status": "valid"}),
            json!({"index": 1, "status": "valid"}),
        ];
        let providers = vec![json!({
            "index": 0, "identity": "failed", "status": "launch_failed",
            "error": "launch failed", "capabilities": []
        })];
        let result = test_failure(
            OutcomeStatus::Unavailable,
            "PROVIDER_LAUNCH_FAILED",
            Some("/providers/0"),
            test_data(&tethers, &providers),
        );
        let value = envelope_value(&result);

        assert_eq!(value["data"]["tethers"], json!(tethers));
        assert_eq!(value["data"]["providers"], json!(providers));
    }

    #[test]
    fn j13a_provider_initialize_failure_preserves_failed_provider_evidence() {
        let providers = vec![json!({
            "index": 0, "identity": "failed", "status": "initialize_failed",
            "error": "initialize failed", "capabilities": []
        })];
        let result = test_failure(
            OutcomeStatus::Unavailable,
            "PROVIDER_INITIALIZE_FAILED",
            Some("/providers/0"),
            test_data(&[json!({"index": 0, "status": "valid"})], &providers),
        );
        let value = envelope_value(&result);

        assert_eq!(value["data"]["providers"][0]["status"], "initialize_failed");
        assert_eq!(value["error"]["code"], "PROVIDER_INITIALIZE_FAILED");
    }

    #[test]
    fn j13a_tools_list_failure_preserves_failed_provider_evidence() {
        let providers = vec![json!({
            "index": 0, "identity": "failed", "status": "tools_list_failed",
            "error": "tools/list failed", "capabilities": []
        })];
        let result = test_failure(
            OutcomeStatus::Unavailable,
            "PROVIDER_TOOLS_LIST_FAILED",
            Some("/providers/0"),
            test_data(&[json!({"index": 0, "status": "valid"})], &providers),
        );
        let value = envelope_value(&result);

        assert_eq!(value["data"]["providers"][0]["status"], "tools_list_failed");
        assert_eq!(value["error"]["code"], "PROVIDER_TOOLS_LIST_FAILED");
    }

    #[test]
    fn j13a_capability_mismatch_preserves_available_and_unavailable_entries() {
        let providers = vec![json!({
            "index": 0,
            "identity": "mixed",
            "status": "unavailable",
            "capabilities": [
                {"name": "fixture.first", "version": 1, "status": "available"},
                {
                    "name": "fixture.missing", "version": 1,
                    "status": "unavailable", "error": "missing"
                }
            ]
        })];
        let result = test_failure(
            OutcomeStatus::Unavailable,
            "PROVIDER_CAPABILITY_UNAVAILABLE",
            Some("/providers/0"),
            test_data(&[json!({"index": 0, "status": "valid"})], &providers),
        );
        let value = envelope_value(&result);
        let capabilities = value["data"]["providers"][0]["capabilities"]
            .as_array()
            .unwrap();

        assert_eq!(capabilities[0]["status"], "available");
        assert_eq!(capabilities[1]["status"], "unavailable");
    }

    #[test]
    fn j13a_later_provider_failure_preserves_earlier_successful_provider() {
        let providers = vec![
            json!({
                "index": 0, "identity": "good", "status": "available",
                "capabilities": [{"name": "fixture.first", "version": 1, "status": "available"}]
            }),
            json!({
                "index": 1, "identity": "failed", "status": "initialize_failed",
                "error": "initialize failed", "capabilities": []
            }),
        ];
        let result = test_failure(
            OutcomeStatus::Unavailable,
            "PROVIDER_INITIALIZE_FAILED",
            Some("/providers/1"),
            test_data(&[json!({"index": 0, "status": "valid"})], &providers),
        );
        let value = envelope_value(&result);

        assert_eq!(value["data"]["providers"][0]["status"], "available");
        assert_eq!(value["data"]["providers"][1]["status"], "initialize_failed");
        assert_eq!(value["error"]["field"], "/providers/1");
    }

    #[test]
    fn j13a_interruption_preserves_completed_evidence_and_uses_interrupted_10() {
        let tethers = vec![json!({"index": 0, "status": "valid"})];
        let providers = vec![json!({"index": 0, "status": "available"})];
        let result = fail_with_data(
            "check",
            CheckFailure::interrupted(),
            test_data(&tethers, &providers),
        );
        let value = envelope_value(&result);

        assert_eq!(result.exit_code, 10);
        assert_eq!(value["status"], "interrupted");
        assert_eq!(value["exit_code"], 10);
        assert_eq!(value["data"]["tethers"], json!(tethers));
        assert_eq!(value["data"]["providers"], json!(providers));
    }

    #[test]
    fn j13a_every_partial_failure_envelope_exit_matches_check_result() {
        let classes = [
            (OutcomeStatus::InvalidData, "TETHER_INVALID"),
            (OutcomeStatus::Unavailable, "PROVIDER_LAUNCH_FAILED"),
            (OutcomeStatus::Unavailable, "PROVIDER_INITIALIZE_FAILED"),
            (OutcomeStatus::Unavailable, "PROVIDER_TOOLS_LIST_FAILED"),
            (
                OutcomeStatus::Unavailable,
                "PROVIDER_CAPABILITY_UNAVAILABLE",
            ),
            (OutcomeStatus::Interrupted, "INTERRUPTED"),
        ];

        for (status, code) in classes {
            let result = test_failure(status, code, None, test_data(&[], &[]));
            let value = envelope_value(&result);
            assert_eq!(result.exit_code, status.exit_code(), "{code}");
            assert_eq!(value["exit_code"], status.exit_code(), "{code}");
            assert_eq!(value["error"]["code"], code);
        }
    }
}
