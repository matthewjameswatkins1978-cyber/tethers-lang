//! Public `run` command coordinator.
//!
//! This module owns only the public boundary: strict path/input validation,
//! durable initial-event admission, one selected Tether, and typed result to
//! CLI-envelope mapping.  Planning, policy, approval, replay, durable intent,
//! provider execution, and Result Anchors remain in their accepted seams.

use crate::configured_runtime::{prepare_runtime, PreparedRuntime};
use crate::dispatch::{FileTrail, Trail};
use crate::event_admission::EventAdmissionGate;
use crate::host_execution::{
    ExecutionServiceError, ExecutionServiceResult, HostExecutionService, PreparedEvaluationInput,
};
use crate::run_input::{parse_run_input, RunInput, RunInputError};
use crate::runtime_config::load_runtime_config;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use tethers_reference_host::child_process::is_interrupted;
use tethers_reference_host::cli::{CliEnvelope, OutcomeStatus};

pub struct RunCommandArgs {
    pub config: PathBuf,
    pub engine: PathBuf,
    pub input: PathBuf,
    pub trail: PathBuf,
    pub host_data_root: PathBuf,
}

#[derive(Debug)]
pub struct RunResult {
    pub envelope: CliEnvelope,
    pub exit_code: i32,
}

impl RunResult {
    fn from_envelope(envelope: CliEnvelope) -> Self {
        let exit_code = envelope.exit_code;
        Self {
            envelope,
            exit_code,
        }
    }
}

/// Execute exactly one public input.  All untrusted file and JSON processing
/// finishes before an engine or provider can launch.
pub fn run(args: RunCommandArgs) -> RunResult {
    run_with_boundaries(
        args,
        admit_external_event,
        |runtime, paths, prepared_input| {
            HostExecutionService::new(
                &runtime,
                &paths.engine,
                &paths.trail,
                Some(&paths.host_data_root),
            )
            .run_selected(std::slice::from_ref(&prepared_input))
        },
    )
}

/// Execute the one public coordinator while keeping its service boundary
/// injectable for focused ordering tests.
fn run_with_boundaries<A, S>(args: RunCommandArgs, admit: A, invoke_service: S) -> RunResult
where
    A: FnOnce(&Path, &RunInput) -> Result<(), RunResult>,
    S: FnOnce(
        PreparedRuntime,
        ResolvedRunPaths,
        PreparedEvaluationInput,
    ) -> Result<Vec<ExecutionServiceResult>, ExecutionServiceError>,
{
    let caller_cwd = match std::env::current_dir() {
        Ok(path) => path,
        Err(_) => {
            return failure(
                OutcomeStatus::Failed,
                "CURRENT_DIRECTORY_FAILED",
                "cannot determine caller directory",
                None,
            )
        }
    };

    let paths = match resolve_paths(&caller_cwd, &args) {
        Ok(paths) => paths,
        Err(result) => return result,
    };

    let input_text = match std::fs::read_to_string(&paths.input) {
        Ok(text) => text,
        Err(_) => {
            return failure(
                OutcomeStatus::InvalidData,
                "RUN_INPUT_READ_FAILED",
                "cannot read run input",
                Some("--input".to_owned()),
            )
        }
    };
    let input = match parse_run_input(&input_text) {
        Ok(input) => input,
        Err(error) => return input_failure(error),
    };

    if is_interrupted() {
        return failure(
            OutcomeStatus::Interrupted,
            "INTERRUPTED",
            "interrupted",
            None,
        );
    }

    let loaded = match load_runtime_config(&paths.config) {
        Ok(loaded) => loaded,
        Err(_) => {
            return failure(
                OutcomeStatus::InvalidData,
                "CONFIG_LOAD_FAILED",
                "cannot load runtime config",
                Some("--config".to_owned()),
            )
        }
    };
    let runtime = match prepare_runtime(&loaded) {
        Ok(runtime) => runtime,
        Err(_) => {
            return failure(
                OutcomeStatus::InvalidData,
                "RUNTIME_PREPARE_FAILED",
                "cannot prepare runtime",
                None,
            )
        }
    };

    let selected_count = runtime
        .tethers()
        .iter()
        .filter(|tether| tether.id == input.tether.id && tether.version == input.tether.version)
        .count();
    if selected_count != 1 {
        return failure(
            OutcomeStatus::InvalidData,
            "TETHER_NOT_FOUND",
            "selected Tether is not configured",
            Some("/tether".to_owned()),
        );
    }

    if let Err(result) = admit(&paths.trail, &input) {
        return result;
    }

    let prepared_input = PreparedEvaluationInput {
        tether_id: input.tether.id,
        tether_version: input.tether.version,
        evaluation_id: input.evaluation_id,
        anchor_event: json!({
            "id": input.event.id,
            "name": input.event.name,
            "data": input.event.data,
        }),
        facts: input.facts,
    };
    let results = match invoke_service(runtime, paths, prepared_input) {
        Ok(results) => results,
        Err(error) => return service_error_failure(error),
    };
    map_one_service_result(results)
}

fn map_one_service_result(results: Vec<ExecutionServiceResult>) -> RunResult {
    let [result] = results.as_slice() else {
        return failure(
            OutcomeStatus::Failed,
            "SERVICE_RESULT_COUNT_INVALID",
            "execution service returned an invalid result count",
            None,
        );
    };
    RunResult::from_envelope(map_execution_result(result))
}

#[derive(Debug)]
struct ResolvedRunPaths {
    config: PathBuf,
    engine: PathBuf,
    input: PathBuf,
    trail: PathBuf,
    host_data_root: PathBuf,
}

fn resolve_paths(caller_cwd: &Path, args: &RunCommandArgs) -> Result<ResolvedRunPaths, RunResult> {
    let config = canonical_regular_file(caller_cwd, &args.config, "--config", "CONFIG")?;
    let engine = canonical_regular_file(caller_cwd, &args.engine, "--engine", "ENGINE")?;
    let input = canonical_regular_file(caller_cwd, &args.input, "--input", "RUN_INPUT")?;
    if !args.trail.is_absolute() {
        return Err(failure(
            OutcomeStatus::InvalidData,
            "TRAIL_NOT_ABSOLUTE",
            "trail path must be absolute",
            Some("--trail".to_owned()),
        ));
    }
    if args.trail.exists() && !args.trail.is_file() {
        return Err(failure(
            OutcomeStatus::InvalidData,
            "TRAIL_NOT_FILE",
            "trail path must be a regular file",
            Some("--trail".to_owned()),
        ));
    }
    if args.trail.parent().is_none_or(|parent| !parent.is_dir()) {
        return Err(failure(
            OutcomeStatus::InvalidData,
            "TRAIL_PARENT_NOT_FOUND",
            "trail parent directory must exist",
            Some("--trail".to_owned()),
        ));
    }
    if !args.host_data_root.is_absolute() {
        return Err(failure(
            OutcomeStatus::InvalidData,
            "HOST_DATA_ROOT_NOT_ABSOLUTE",
            "host-data-root must be absolute",
            Some("--host-data-root".to_owned()),
        ));
    }
    Ok(ResolvedRunPaths {
        config,
        engine,
        input,
        trail: args.trail.clone(),
        host_data_root: args.host_data_root.clone(),
    })
}

fn canonical_regular_file(
    caller_cwd: &Path,
    supplied: &Path,
    option: &str,
    label: &str,
) -> Result<PathBuf, RunResult> {
    let resolved = if supplied.is_absolute() {
        supplied.to_path_buf()
    } else {
        caller_cwd.join(supplied)
    };
    let canonical = resolved.canonicalize().map_err(|_| {
        failure(
            OutcomeStatus::InvalidData,
            format!("{label}_NOT_FOUND"),
            "required path was not found",
            Some(option.to_owned()),
        )
    })?;
    if !canonical.is_file() {
        return Err(failure(
            OutcomeStatus::InvalidData,
            format!("{label}_NOT_FILE"),
            "required path must be a regular file",
            Some(option.to_owned()),
        ));
    }
    Ok(canonical)
}

fn admit_external_event(trail_path: &Path, input: &RunInput) -> Result<(), RunResult> {
    let mut gate = EventAdmissionGate::new();
    let admission = gate.admit(&input.event.id, 0);
    let entry = crate::build_event_admission_entry(
        &input.event.id,
        &input.event.name,
        "external",
        &input.event.id,
        None,
        0,
        &admission,
        crate::now_unix_ms(),
    );
    let mut trail = FileTrail::open(trail_path).map_err(|_| {
        failure(
            OutcomeStatus::AuditFailed,
            "EVENT_ADMISSION_AUDIT_FAILED",
            "cannot open Trail for initial event admission",
            None,
        )
    })?;
    trail.append_event_admission(&entry).map_err(|_| {
        failure(
            OutcomeStatus::AuditFailed,
            "EVENT_ADMISSION_AUDIT_FAILED",
            "cannot durably record initial event admission",
            None,
        )
    })?;
    admission.map_err(|_| {
        failure(
            OutcomeStatus::InvalidData,
            "EVENT_ADMISSION_REJECTED",
            "initial external event was rejected",
            Some("/event/id".to_owned()),
        )
    })
}

pub(crate) fn map_execution_result(result: &ExecutionServiceResult) -> CliEnvelope {
    match result {
        ExecutionServiceResult::Completed {
            evaluation_id,
            action_id,
            response,
            execution_id,
        } => status_envelope(
            OutcomeStatus::Completed,
            completed_data(evaluation_id, action_id, response, execution_id.as_deref()),
        ),
        ExecutionServiceResult::Denied {
            evaluation_id,
            action_id,
            ..
        } => status_envelope(
            OutcomeStatus::Denied,
            execution_data(evaluation_id, Some(action_id), Some("denied")),
        ),
        ExecutionServiceResult::NoActions { evaluation_id, .. } => status_envelope(
            OutcomeStatus::NoActions,
            execution_data(evaluation_id, None, Some("no_actions")),
        ),
        ExecutionServiceResult::ApprovalRequired {
            evaluation_id,
            action_id,
            reason,
        } => status_envelope(
            OutcomeStatus::ApprovalRequired,
            json!({"evaluation_id": evaluation_id, "action_id": action_id, "reason": reason}),
        ),
        ExecutionServiceResult::PlannerError { evaluation_id, .. } => error_with_data(
            OutcomeStatus::InvalidData,
            "PLANNER_ERROR",
            "planner returned an error",
            evaluation_id
                .as_deref()
                .map(|id| execution_data(id, None, None))
                .unwrap_or_else(empty_data),
        ),
        ExecutionServiceResult::Unavailable { evaluation_id, .. } => error_with_data(
            OutcomeStatus::Unavailable,
            "EXECUTION_UNAVAILABLE",
            "execution is unavailable",
            execution_data(evaluation_id, None, None),
        ),
        ExecutionServiceResult::Failed {
            evaluation_id,
            action_id,
            execution_id,
            ..
        } => error_with_data(
            OutcomeStatus::Failed,
            "ACTION_FAILED",
            "Action failed",
            execution_data_with_id(
                evaluation_id,
                Some(action_id),
                Some("failed"),
                execution_id.as_deref(),
            ),
        ),
        ExecutionServiceResult::Uncertain {
            evaluation_id,
            action_id,
            execution_id,
            ..
        } => error_with_data(
            OutcomeStatus::Uncertain,
            "ACTION_UNCERTAIN",
            "Action outcome is uncertain",
            execution_data_with_id(
                evaluation_id,
                Some(action_id),
                Some("uncertain"),
                execution_id.as_deref(),
            ),
        ),
        ExecutionServiceResult::AuditFailed {
            evaluation_id,
            action_id,
            execution_id,
            ..
        } => error_with_data(
            OutcomeStatus::AuditFailed,
            "EXECUTION_AUDIT_FAILED",
            "execution audit could not be confirmed",
            execution_data_with_id(
                evaluation_id,
                Some(action_id),
                Some("audit_failed"),
                execution_id.as_deref(),
            ),
        ),
        ExecutionServiceResult::Unattempted {
            evaluation_id,
            action_id,
            execution_id,
            ..
        } => error_with_data(
            OutcomeStatus::Failed,
            "ACTION_UNATTEMPTED",
            "Action was not attempted",
            execution_data_with_id(
                evaluation_id,
                Some(action_id),
                Some("unattempted"),
                execution_id.as_deref(),
            ),
        ),
        ExecutionServiceResult::ReplayBlockedCompletedSuccess {
            evaluation_id,
            action_id,
            execution_id,
        } => status_envelope(
            OutcomeStatus::Completed,
            execution_data_with_id(
                evaluation_id,
                Some(action_id),
                Some("replay_blocked_completed_success"),
                execution_id.as_deref(),
            ),
        ),
        ExecutionServiceResult::ReplayBlockedCompletedFailure {
            evaluation_id,
            action_id,
            execution_id,
        } => error_with_data(
            OutcomeStatus::Failed,
            "REPLAY_BLOCKED_COMPLETED_FAILURE",
            "replay is blocked by a prior completed failure",
            execution_data_with_id(
                evaluation_id,
                Some(action_id),
                Some("replay_blocked_completed_failure"),
                execution_id.as_deref(),
            ),
        ),
        ExecutionServiceResult::ReplayRequiresManualResolution {
            evaluation_id,
            action_id,
            execution_id,
        } => error_with_data(
            OutcomeStatus::Uncertain,
            "REPLAY_REQUIRES_MANUAL_RESOLUTION",
            "replay requires manual resolution",
            execution_data_with_id(
                evaluation_id,
                Some(action_id),
                Some("replay_requires_manual_resolution"),
                execution_id.as_deref(),
            ),
        ),
        ExecutionServiceResult::ReplayPersistenceUnavailable {
            evaluation_id,
            action_id,
        } => error_with_data(
            OutcomeStatus::Unavailable,
            "REPLAY_PERSISTENCE_UNAVAILABLE",
            "replay persistence is unavailable",
            execution_data(
                evaluation_id,
                Some(action_id),
                Some("replay_persistence_unavailable"),
            ),
        ),
        ExecutionServiceResult::Interrupted => error_with_data(
            OutcomeStatus::Interrupted,
            "INTERRUPTED",
            "interrupted",
            empty_data(),
        ),
        ExecutionServiceResult::InvalidData { .. } => error_with_data(
            OutcomeStatus::InvalidData,
            "INVALID_DATA",
            "execution data is invalid",
            empty_data(),
        ),
    }
}

fn service_error_failure(error: ExecutionServiceError) -> RunResult {
    match error {
        ExecutionServiceError::Engine(_) | ExecutionServiceError::Provider(_) => failure(
            OutcomeStatus::Unavailable,
            "EXECUTION_UNAVAILABLE",
            "execution service is unavailable",
            None,
        ),
        ExecutionServiceError::TetherValidation(_) | ExecutionServiceError::InvalidInput(_) => {
            failure(
                OutcomeStatus::InvalidData,
                "TETHER_INVALID",
                "Tether validation failed",
                None,
            )
        }
        ExecutionServiceError::Interrupted => failure(
            OutcomeStatus::Interrupted,
            "INTERRUPTED",
            "interrupted",
            None,
        ),
    }
}

fn input_failure(error: RunInputError) -> RunResult {
    failure(
        OutcomeStatus::InvalidData,
        error.code.as_str(),
        error.message,
        error.field,
    )
}

fn status_envelope(status: OutcomeStatus, data: Value) -> CliEnvelope {
    CliEnvelope {
        schema: "tethers.cli/1",
        command: "run".to_owned(),
        status,
        exit_code: status.exit_code(),
        data,
        error: None,
    }
}

fn error_with_data(
    status: OutcomeStatus,
    code: impl Into<String>,
    message: impl Into<String>,
    data: Value,
) -> CliEnvelope {
    CliEnvelope::error_with_data("run", status, code, message, None, data)
}

fn failure(
    status: OutcomeStatus,
    code: impl Into<String>,
    message: impl Into<String>,
    field: Option<String>,
) -> RunResult {
    RunResult::from_envelope(CliEnvelope::error("run", status, code, message, field))
}

fn execution_data(
    evaluation_id: &str,
    action_id: Option<&str>,
    execution_status: Option<&str>,
) -> Value {
    let mut data = serde_json::Map::new();
    data.insert(
        "evaluation_id".to_owned(),
        Value::String(evaluation_id.to_owned()),
    );
    if let Some(action_id) = action_id {
        data.insert("action_id".to_owned(), Value::String(action_id.to_owned()));
    }
    if let Some(execution_status) = execution_status {
        data.insert(
            "execution_status".to_owned(),
            Value::String(execution_status.to_owned()),
        );
    }
    Value::Object(data)
}

fn execution_data_with_id(
    evaluation_id: &str,
    action_id: Option<&str>,
    execution_status: Option<&str>,
    execution_id: Option<&str>,
) -> Value {
    let mut data = execution_data(evaluation_id, action_id, execution_status);
    if let (Some(obj), Some(exec_id)) = (data.as_object_mut(), execution_id) {
        obj.insert("execution_id".to_owned(), Value::String(exec_id.to_owned()));
    }
    data
}

fn completed_data(
    evaluation_id: &str,
    action_id: &str,
    response: &Value,
    execution_id: Option<&str>,
) -> Value {
    let mut data = execution_data_with_id(
        evaluation_id,
        Some(action_id),
        Some("completed"),
        execution_id,
    );
    if let Some(anchor) = response.get("result_anchor") {
        data["result_anchor"] = anchor.clone();
    }
    data
}

fn empty_data() -> Value {
    Value::Object(serde_json::Map::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_input() -> RunInput {
        RunInput {
            evaluation_id: "eval-public-001".to_owned(),
            tether: crate::run_input::RunTether {
                id: "selected".to_owned(),
                version: "1".to_owned(),
            },
            event: crate::run_input::RunEvent {
                id: "evt-public-001".to_owned(),
                name: "coding.task_completed".to_owned(),
                data: json!({"task": "LK-39"}),
            },
            facts: json!({"project.type": "software"}),
        }
    }

    fn completed_result() -> ExecutionServiceResult {
        ExecutionServiceResult::Completed {
            evaluation_id: "eval".to_owned(),
            action_id: "action".to_owned(),
            response: json!({}),
            execution_id: None,
        }
    }

    fn assert_mapping(result: ExecutionServiceResult, status: OutcomeStatus, code: Option<&str>) {
        let envelope = map_execution_result(&result);
        assert_eq!(envelope.command, "run");
        assert_eq!(envelope.status, status);
        assert_eq!(envelope.exit_code, status.exit_code());
        assert_eq!(
            envelope.error.as_ref().map(|error| error.code.as_str()),
            code
        );
    }

    #[test]
    fn j13b_run_every_execution_service_result_has_frozen_envelope_mapping() {
        let e = "eval".to_owned();
        let a = "action".to_owned();
        assert_mapping(
            ExecutionServiceResult::Completed {
                evaluation_id: e.clone(),
                action_id: a.clone(),
                response: json!({}),
                execution_id: None,
            },
            OutcomeStatus::Completed,
            None,
        );
        assert_mapping(
            ExecutionServiceResult::Denied {
                evaluation_id: e.clone(),
                action_id: a.clone(),
                reason: String::new(),
            },
            OutcomeStatus::Denied,
            None,
        );
        assert_mapping(
            ExecutionServiceResult::NoActions {
                evaluation_id: e.clone(),
                response: json!({}),
            },
            OutcomeStatus::NoActions,
            None,
        );
        assert_mapping(
            ExecutionServiceResult::ApprovalRequired {
                evaluation_id: e.clone(),
                action_id: a.clone(),
                reason: "host_policy_ask".to_owned(),
            },
            OutcomeStatus::ApprovalRequired,
            None,
        );
        assert_mapping(
            ExecutionServiceResult::PlannerError {
                evaluation_id: Some(e.clone()),
                code: "parse".to_owned(),
                message: String::new(),
            },
            OutcomeStatus::InvalidData,
            Some("PLANNER_ERROR"),
        );
        assert_mapping(
            ExecutionServiceResult::Unavailable {
                evaluation_id: e.clone(),
                reason: String::new(),
            },
            OutcomeStatus::Unavailable,
            Some("EXECUTION_UNAVAILABLE"),
        );
        assert_mapping(
            ExecutionServiceResult::Failed {
                evaluation_id: e.clone(),
                action_id: a.clone(),
                reason: String::new(),
                execution_id: None,
            },
            OutcomeStatus::Failed,
            Some("ACTION_FAILED"),
        );
        assert_mapping(
            ExecutionServiceResult::Uncertain {
                evaluation_id: e.clone(),
                action_id: a.clone(),
                reason: String::new(),
                execution_id: None,
            },
            OutcomeStatus::Uncertain,
            Some("ACTION_UNCERTAIN"),
        );
        assert_mapping(
            ExecutionServiceResult::AuditFailed {
                evaluation_id: e.clone(),
                action_id: a.clone(),
                reason: String::new(),
                execution_id: None,
            },
            OutcomeStatus::AuditFailed,
            Some("EXECUTION_AUDIT_FAILED"),
        );
        assert_mapping(
            ExecutionServiceResult::Unattempted {
                evaluation_id: e.clone(),
                action_id: a.clone(),
                reason: String::new(),
                execution_id: None,
            },
            OutcomeStatus::Failed,
            Some("ACTION_UNATTEMPTED"),
        );
        assert_mapping(
            ExecutionServiceResult::ReplayBlockedCompletedSuccess {
                evaluation_id: e.clone(),
                action_id: a.clone(),
                execution_id: None,
            },
            OutcomeStatus::Completed,
            None,
        );
        assert_mapping(
            ExecutionServiceResult::ReplayBlockedCompletedFailure {
                evaluation_id: e.clone(),
                action_id: a.clone(),
                execution_id: None,
            },
            OutcomeStatus::Failed,
            Some("REPLAY_BLOCKED_COMPLETED_FAILURE"),
        );
        assert_mapping(
            ExecutionServiceResult::ReplayRequiresManualResolution {
                evaluation_id: e.clone(),
                action_id: a.clone(),
                execution_id: None,
            },
            OutcomeStatus::Uncertain,
            Some("REPLAY_REQUIRES_MANUAL_RESOLUTION"),
        );
        assert_mapping(
            ExecutionServiceResult::ReplayPersistenceUnavailable {
                evaluation_id: e.clone(),
                action_id: a.clone(),
            },
            OutcomeStatus::Unavailable,
            Some("REPLAY_PERSISTENCE_UNAVAILABLE"),
        );
        assert_mapping(
            ExecutionServiceResult::Interrupted,
            OutcomeStatus::Interrupted,
            Some("INTERRUPTED"),
        );
        assert_mapping(
            ExecutionServiceResult::InvalidData {
                message: String::new(),
            },
            OutcomeStatus::InvalidData,
            Some("INVALID_DATA"),
        );
    }

    #[test]
    fn j13b_run_approval_envelope_has_no_approval_id() {
        let envelope = map_execution_result(&ExecutionServiceResult::ApprovalRequired {
            evaluation_id: "eval".to_owned(),
            action_id: "action".to_owned(),
            reason: "host_policy_ask".to_owned(),
        });
        let encoded = serde_json::to_string(&envelope).unwrap();
        assert!(!encoded.contains("approval_id"));
        assert_eq!(envelope.data["evaluation_id"], "eval");
        assert_eq!(envelope.data["action_id"], "action");
    }

    #[test]
    fn j13b_run_service_errors_have_safe_frozen_statuses() {
        assert_eq!(
            service_error_failure(ExecutionServiceError::Interrupted)
                .envelope
                .status,
            OutcomeStatus::Interrupted
        );
        assert_eq!(
            service_error_failure(ExecutionServiceError::InvalidInput("untrusted".to_owned()))
                .envelope
                .status,
            OutcomeStatus::InvalidData
        );
        let validation = service_error_failure(ExecutionServiceError::TetherValidation(
            "untrusted engine detail".to_owned(),
        ));
        assert_eq!(validation.envelope.status, OutcomeStatus::InvalidData);
        assert_eq!(validation.envelope.exit_code, 3);
        assert_eq!(validation.envelope.error.unwrap().code, "TETHER_INVALID");

        let operational = service_error_failure(ExecutionServiceError::Engine(
            tethers_reference_host::engine_stdio::EngineError::ProtocolError(
                "untrusted engine detail".to_owned(),
            ),
        ));
        assert_eq!(operational.envelope.status, OutcomeStatus::Unavailable);
        assert_eq!(operational.envelope.exit_code, 4);
        let encoded = serde_json::to_string(&operational.envelope).unwrap();
        assert!(encoded.contains("EXECUTION_UNAVAILABLE"));
        assert!(!encoded.contains("untrusted engine detail"));
    }

    #[test]
    fn j13b_run_standing_allow_fixture_has_a_reviewed_canonical_digest() {
        let source =
            include_str!("../../protocol/capability-manifests/fixture-ping-standing-allow.json");
        let (_, digest) = crate::manifest::canonicalize_and_digest(source).unwrap();
        assert_eq!(
            digest,
            "sha256:eb61b62bde489e00a4d15c37c83e6cdb1e9e378b8f13b910d4b68bd6d68c19da"
        );
        crate::manifest::verify_manifest(source).unwrap();
    }

    #[test]
    fn j13b_run_initial_admission_is_external_durable_and_host_owned() {
        let directory =
            std::env::temp_dir().join(format!("j13b-run-admission-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let trail_path = directory.join("trail.jsonl");
        let input = sample_input();
        admit_external_event(&trail_path, &input).unwrap();
        let entry: Value =
            serde_json::from_str(&std::fs::read_to_string(&trail_path).unwrap()).unwrap();
        assert_eq!(entry["kind"], "event_admitted");
        assert_eq!(entry["event_id"], "evt-public-001");
        assert_eq!(entry["source"], "external");
        assert_eq!(entry["correlation_id"], "evt-public-001");
        assert_eq!(entry["causation_id"], Value::Null);
        assert_eq!(entry["generation"], 0);
        std::fs::remove_dir_all(directory).unwrap();
    }

    fn write_runtime_fixture(directory: &Path) -> RunCommandArgs {
        let manifest_dir = directory.join("manifests");
        let tether_dir = directory.join("tethers");
        std::fs::create_dir_all(&manifest_dir).unwrap();
        std::fs::create_dir_all(&tether_dir).unwrap();
        let manifest =
            include_str!("../../protocol/capability-manifests/fixture-ping-standing-allow.json");
        let (_, digest) = crate::manifest::canonicalize_and_digest(manifest).unwrap();
        std::fs::write(manifest_dir.join("fixture-ping.json"), manifest).unwrap();
        std::fs::write(tether_dir.join("selected.tether"), "fixture source").unwrap();
        let config = json!({
            "format_version": "0.1",
            "tether_set": {
                "id": "j13b-run-test",
                "version": "1",
                "tethers": [{"id": "selected", "version": "1", "source_path": "tethers/selected.tether"}],
                "capability_requirements": [{"name": "fixture.ping", "version": 1, "reason": "test"}]
            },
            "providers": [{
                "id": "tethers-stdio-fixture",
                "display_name": "Tethers Stdio Fixture",
                "transport": {"kind": "stdio", "command": "unused.exe", "args": [], "protocol_version": "2025-11-25"},
                "capabilities": [{
                    "name": "fixture.ping", "version": 1,
                    "manifest_path": "manifests/fixture-ping.json", "pinned_digest": digest,
                    "scope_binding": {"kind": "path_prefix", "argument_json_pointer": "/path"}
                }]
            }],
            "policy": {"default": "deny", "rules": []}
        });
        let config_path = directory.join("runtime.json");
        std::fs::write(&config_path, serde_json::to_vec(&config).unwrap()).unwrap();
        let engine_path = directory.join("engine.exe");
        std::fs::write(&engine_path, "unused").unwrap();
        let input_path = directory.join("input.json");
        std::fs::write(
            &input_path,
            serde_json::to_vec(&json!({
                "format_version": "1",
                "evaluation_id": "eval-public-001",
                "tether": {"id": "selected", "version": "1"},
                "event": {"id": "evt-public-001", "name": "coding.task_completed", "data": {}},
                "facts": {}
            }))
            .unwrap(),
        )
        .unwrap();
        RunCommandArgs {
            config: config_path,
            engine: engine_path,
            input: input_path,
            trail: directory.join("trail.jsonl"),
            host_data_root: directory.join("host-data"),
        }
    }

    #[test]
    fn j13b_run_rejections_do_not_reach_the_service_boundary() {
        use std::cell::Cell;
        use std::rc::Rc;

        let directory =
            std::env::temp_dir().join(format!("j13b-run-boundary-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let calls = Rc::new(Cell::new(0));
        let invoke = |calls: Rc<Cell<usize>>| {
            move |_, _, _| {
                calls.set(calls.get() + 1);
                Ok(vec![completed_result()])
            }
        };

        for invalid in [
            "{bad json",
            r#"{"format_version":"1","format_version":"1"}"#,
            r#"{"format_version":"1","evaluation_id":"eval","extra":true,"tether":{"id":"selected","version":"1"},"event":{"id":"evt","name":"event","data":{}},"facts":{}}"#,
            r#"{"format_version":"1","evaluation_id":2,"tether":{"id":"selected","version":"1"},"event":{"id":"evt","name":"event","data":{}},"facts":{}}"#,
            r#"{"format_version":"1","evaluation_id":"","tether":{"id":"selected","version":"1"},"event":{"id":"evt","name":"event","data":{}},"facts":{}}"#,
        ] {
            let mut invalid_args = write_runtime_fixture(&directory);
            std::fs::write(&invalid_args.input, invalid).unwrap();
            invalid_args.trail = directory.join(format!("invalid-{}.jsonl", uuid::Uuid::new_v4()));
            let result =
                run_with_boundaries(invalid_args, admit_external_event, invoke(calls.clone()));
            assert_eq!(result.envelope.status, OutcomeStatus::InvalidData);
        }

        let mut unknown_args = write_runtime_fixture(&directory);
        std::fs::write(&unknown_args.input, serde_json::to_vec(&json!({
            "format_version": "1", "evaluation_id": "eval", "tether": {"id": "missing", "version": "1"},
            "event": {"id": "evt", "name": "event", "data": {}}, "facts": {}
        })).unwrap()).unwrap();
        unknown_args.trail = directory.join("unknown.jsonl");
        let unknown =
            run_with_boundaries(unknown_args, admit_external_event, invoke(calls.clone()));
        assert_eq!(unknown.envelope.error.unwrap().code, "TETHER_NOT_FOUND");

        let invalid_path = run_with_boundaries(
            RunCommandArgs {
                engine: directory.join("missing.exe"),
                trail: directory.join("path.jsonl"),
                ..write_runtime_fixture(&directory)
            },
            admit_external_event,
            invoke(calls.clone()),
        );
        assert_eq!(invalid_path.envelope.status, OutcomeStatus::InvalidData);

        for phase in ["open", "write", "sync"] {
            let result = run_with_boundaries(
                RunCommandArgs {
                    trail: directory.join(format!("{phase}.jsonl")),
                    ..write_runtime_fixture(&directory)
                },
                |_, _| {
                    Err(failure(
                        OutcomeStatus::AuditFailed,
                        "EVENT_ADMISSION_AUDIT_FAILED",
                        "admission unavailable",
                        None,
                    ))
                },
                invoke(calls.clone()),
            );
            assert_eq!(result.envelope.status, OutcomeStatus::AuditFailed);
        }
        assert_eq!(calls.get(), 0);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn j13b_run_admits_before_invoking_the_service_boundary() {
        use std::cell::Cell;
        use std::rc::Rc;

        let directory =
            std::env::temp_dir().join(format!("j13b-run-admitted-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let args = write_runtime_fixture(&directory);
        let trail = args.trail.clone();
        let calls = Rc::new(Cell::new(0));
        let result = run_with_boundaries(args, admit_external_event, {
            let calls = calls.clone();
            move |_, _, _| {
                calls.set(calls.get() + 1);
                let entry: Value =
                    serde_json::from_str(&std::fs::read_to_string(&trail).unwrap()).unwrap();
                assert_eq!(entry["kind"], "event_admitted");
                Ok(vec![completed_result()])
            }
        });
        assert_eq!(result.envelope.status, OutcomeStatus::Completed);
        assert_eq!(calls.get(), 1);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn j13b_run_paths_canonicalise_only_regular_config_engine_and_input_files() {
        let directory =
            std::env::temp_dir().join(format!("j13b-run-paths-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        for file in ["config.json", "engine.exe", "input.json"] {
            std::fs::write(directory.join(file), "fixture").unwrap();
        }
        let args = RunCommandArgs {
            config: PathBuf::from("config.json"),
            engine: PathBuf::from("engine.exe"),
            input: PathBuf::from("input.json"),
            trail: directory.join("trail.jsonl"),
            host_data_root: directory.join("host-data"),
        };
        let paths = resolve_paths(&directory, &args).unwrap();
        assert_eq!(
            paths.config,
            directory.join("config.json").canonicalize().unwrap()
        );
        assert_eq!(
            paths.engine,
            directory.join("engine.exe").canonicalize().unwrap()
        );
        assert_eq!(
            paths.input,
            directory.join("input.json").canonicalize().unwrap()
        );
        let relative_trail = RunCommandArgs {
            trail: PathBuf::from("trail.jsonl"),
            ..args
        };
        assert_eq!(
            resolve_paths(&directory, &relative_trail)
                .unwrap_err()
                .envelope
                .error
                .unwrap()
                .code,
            "TRAIL_NOT_ABSOLUTE"
        );
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn j13b_run_requires_one_typed_service_result_and_derives_exit_from_status() {
        let one = map_one_service_result(vec![completed_result()]);
        assert_eq!(one.envelope.status, OutcomeStatus::Completed);
        assert_eq!(one.exit_code, one.envelope.status.exit_code());
        for results in [Vec::new(), vec![completed_result(), completed_result()]] {
            let result = map_one_service_result(results);
            assert_eq!(result.envelope.status, OutcomeStatus::Failed);
            assert_eq!(
                result.envelope.error.unwrap().code,
                "SERVICE_RESULT_COUNT_INVALID"
            );
            assert_eq!(result.exit_code, result.envelope.status.exit_code());
        }
    }
}
