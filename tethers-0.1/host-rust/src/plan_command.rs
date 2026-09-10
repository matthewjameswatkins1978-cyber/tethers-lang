//! Stable read-only `tethers plan` command coordinator.
//!
//! The command parses and validates the public input, prepares the configured
//! runtime, evaluates the selected Tether through the real Core engine, and
//! stops before policy, replay, Trail execution entries, provider launch, or
//! dispatch. Its `tethers.plan/1` data is intentionally explicit about those
//! absent effects so an agent cannot mistake a proposal for execution.

use crate::configured_runtime::prepare_runtime;
use crate::host_execution::{
    ExecutionServiceError, HostExecutionService, PlanResult, PreparedEvaluationInput,
};
use crate::run_input::{parse_run_input, RunInputError};
use crate::runtime_config::load_runtime_config;
use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf};
use tethers_reference_host::child_process::is_interrupted;
use tethers_reference_host::cli::{CliEnvelope, OutcomeStatus};

pub struct PlanCommandArgs {
    pub config: PathBuf,
    pub engine: PathBuf,
    pub input: PathBuf,
    pub host_data_root: PathBuf,
}

pub struct PlanResultEnvelope {
    pub envelope: CliEnvelope,
    pub exit_code: i32,
}

impl PlanResultEnvelope {
    fn from_envelope(envelope: CliEnvelope) -> Self {
        let exit_code = envelope.exit_code;
        Self {
            envelope,
            exit_code,
        }
    }
}

#[derive(Debug)]
struct ResolvedPlanPaths {
    config: PathBuf,
    engine: PathBuf,
    input: PathBuf,
    host_data_root: PathBuf,
}

fn canonical_regular_file(
    caller_cwd: &Path,
    supplied: &Path,
    option: &str,
    label: &str,
) -> Result<PathBuf, PlanResultEnvelope> {
    let resolved = if supplied.is_absolute() {
        supplied.to_path_buf()
    } else {
        caller_cwd.join(supplied)
    };
    let canonical = resolved.canonicalize().map_err(|_| {
        plan_failure(
            OutcomeStatus::InvalidData,
            format!("{label}_NOT_FOUND"),
            "required path was not found",
            Some(option.to_owned()),
        )
    })?;
    if !canonical.is_file() {
        return Err(plan_failure(
            OutcomeStatus::InvalidData,
            format!("{label}_NOT_FILE"),
            "required path must be a regular file",
            Some(option.to_owned()),
        ));
    }
    Ok(canonical)
}

fn resolve_plan_paths(
    caller_cwd: &Path,
    args: &PlanCommandArgs,
) -> Result<ResolvedPlanPaths, PlanResultEnvelope> {
    let config = canonical_regular_file(caller_cwd, &args.config, "--config", "CONFIG")?;
    let engine = canonical_regular_file(caller_cwd, &args.engine, "--engine", "ENGINE")?;
    let input = canonical_regular_file(caller_cwd, &args.input, "--input", "RUN_INPUT")?;
    if !args.host_data_root.is_absolute() {
        return Err(plan_failure(
            OutcomeStatus::InvalidData,
            "HOST_DATA_ROOT_NOT_ABSOLUTE",
            "host-data-root must be absolute",
            Some("--host-data-root".to_owned()),
        ));
    }
    Ok(ResolvedPlanPaths {
        config,
        engine,
        input,
        host_data_root: args.host_data_root.clone(),
    })
}

pub fn run_plan(args: PlanCommandArgs) -> PlanResultEnvelope {
    let caller_cwd = match std::env::current_dir() {
        Ok(path) => path,
        Err(_) => {
            return plan_failure(
                OutcomeStatus::Failed,
                "CURRENT_DIRECTORY_FAILED",
                "cannot determine caller directory",
                None,
            )
        }
    };
    let paths = match resolve_plan_paths(&caller_cwd, &args) {
        Ok(paths) => paths,
        Err(result) => return result,
    };
    let input_text = match std::fs::read_to_string(&paths.input) {
        Ok(text) => text,
        Err(_) => {
            return plan_failure(
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
        return plan_failure(
            OutcomeStatus::Interrupted,
            "INTERRUPTED",
            "interrupted",
            None,
        );
    }
    let loaded = match load_runtime_config(&paths.config) {
        Ok(loaded) => loaded,
        Err(_) => {
            return plan_failure(
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
            return plan_failure(
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
        return plan_failure(
            OutcomeStatus::InvalidData,
            "TETHER_NOT_FOUND",
            "selected Tether is not configured",
            Some("/tether".to_owned()),
        );
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
    let service = HostExecutionService::new(
        &runtime,
        &paths.engine,
        Path::new("plan-no-trail"),
        Some(&paths.host_data_root),
    );
    match service.plan_only(&prepared_input) {
        Ok(result) => map_plan_result(result),
        Err(error) => match error {
            ExecutionServiceError::Engine(_) | ExecutionServiceError::Provider(_) => plan_failure(
                OutcomeStatus::Unavailable,
                "PLAN_ENGINE_UNAVAILABLE",
                "plan engine is unavailable",
                None,
            ),
            ExecutionServiceError::TetherValidation(_) | ExecutionServiceError::InvalidInput(_) => {
                plan_failure(
                    OutcomeStatus::InvalidData,
                    "TETHER_INVALID",
                    "Tether validation failed",
                    None,
                )
            }
            ExecutionServiceError::Interrupted => plan_failure(
                OutcomeStatus::Interrupted,
                "INTERRUPTED",
                "interrupted",
                None,
            ),
        },
    }
}

fn plan_control_fields() -> Map<String, Value> {
    let mut fields = Map::new();
    fields.insert(
        "authority".to_owned(),
        json!({"granted": false, "status": "not_requested"}),
    );
    fields.insert(
        "execution".to_owned(),
        json!({
            "performed": false,
            "provider_invocations": 0,
            "policy_evaluated": false,
            "replay_mutated": false,
            "trail_execution_entries": 0
        }),
    );
    fields
}

fn plan_base_data(
    evaluation_id: Option<&str>,
    event_id: &str,
    tether_id: &str,
    tether_version: &str,
) -> Value {
    let mut data = Map::new();
    if let Some(evaluation_id) = evaluation_id {
        data.insert(
            "evaluation_id".to_owned(),
            Value::String(evaluation_id.to_owned()),
        );
    }
    data.insert("event_id".to_owned(), Value::String(event_id.to_owned()));
    data.insert("tether_id".to_owned(), Value::String(tether_id.to_owned()));
    data.insert(
        "tether_version".to_owned(),
        Value::String(tether_version.to_owned()),
    );
    data.extend(plan_control_fields());
    Value::Object(data)
}

fn plan_available_data(
    evaluation_id: Option<&str>,
    event_id: &str,
    tether_id: &str,
    tether_version: &str,
    plan: &Value,
) -> Value {
    let mut data = plan_base_data(evaluation_id, event_id, tether_id, tether_version);
    data["plan"] = plan.clone();
    data
}

fn plan_error_data(
    evaluation_id: Option<&str>,
    event_id: &str,
    tether_id: &str,
    tether_version: &str,
    code: &str,
    message: &str,
) -> Value {
    let mut data = plan_base_data(evaluation_id, event_id, tether_id, tether_version);
    data["error"] = json!({"code": code, "message": message});
    data
}

fn no_action_data(
    evaluation_id: &str,
    event_id: &str,
    tether_id: &str,
    tether_version: &str,
    response: &Value,
) -> Value {
    let mut data = plan_available_data(
        Some(evaluation_id),
        event_id,
        tether_id,
        tether_version,
        &Value::Null,
    );
    data["planner_response"] = response.clone();
    data
}

fn with_contract_marker(mut data: Value, result: &str) -> Value {
    if let Some(object) = data.as_object_mut() {
        object.insert(
            "schema".to_owned(),
            Value::String("tethers.plan/1".to_owned()),
        );
        object.insert("result".to_owned(), Value::String(result.to_owned()));
    }
    data
}

fn map_plan_result(result: PlanResult) -> PlanResultEnvelope {
    match result {
        PlanResult::PlanAvailable {
            evaluation_id,
            event_id,
            tether_id,
            tether_version,
            plan,
        } => success_envelope(
            "completed",
            plan_available_data(
                Some(&evaluation_id),
                &event_id,
                &tether_id,
                &tether_version,
                &plan,
            ),
        ),
        PlanResult::NoActions {
            evaluation_id,
            event_id,
            tether_id,
            tether_version,
            response,
        } => success_envelope(
            "no_actions",
            no_action_data(
                &evaluation_id,
                &event_id,
                &tether_id,
                &tether_version,
                &response,
            ),
        ),
        PlanResult::PlannerError {
            evaluation_id,
            event_id,
            tether_id,
            tether_version,
            code,
            message,
        } => error_envelope(
            OutcomeStatus::InvalidData,
            "PLANNER_ERROR",
            "plan planner_error",
            "planner_error",
            plan_error_data(
                evaluation_id.as_deref(),
                &event_id,
                &tether_id,
                &tether_version,
                &code,
                &message,
            ),
        ),
        PlanResult::Interrupted => error_envelope(
            OutcomeStatus::Interrupted,
            "PLAN_INTERRUPTED",
            "plan interrupted",
            "interrupted",
            plan_failure_data("interrupted"),
        ),
        PlanResult::Unavailable {
            evaluation_id,
            event_id,
            tether_id,
            tether_version,
            reason,
        } => error_envelope(
            OutcomeStatus::Unavailable,
            "PLAN_UNAVAILABLE",
            "plan unavailable",
            "unavailable",
            plan_with_reason(
                Some(&evaluation_id),
                &event_id,
                &tether_id,
                &tether_version,
                "unavailable",
                &reason,
            ),
        ),
        PlanResult::InvalidData {
            evaluation_id,
            event_id,
            tether_id,
            tether_version,
            message,
        } => error_envelope(
            OutcomeStatus::InvalidData,
            "PLAN_INVALID_DATA",
            "plan invalid_data",
            "invalid_data",
            plan_with_reason(
                Some(&evaluation_id),
                &event_id,
                &tether_id,
                &tether_version,
                "invalid_data",
                &message,
            ),
        ),
    }
}

fn success_envelope(result: &str, data: Value) -> PlanResultEnvelope {
    PlanResultEnvelope::from_envelope(CliEnvelope {
        schema: "tethers.cli/1",
        command: "plan".to_owned(),
        status: if result == "no_actions" {
            OutcomeStatus::NoActions
        } else {
            OutcomeStatus::Completed
        },
        exit_code: 0,
        data: with_contract_marker(data, result),
        error: None,
    })
}

fn error_envelope(
    status: OutcomeStatus,
    code: &str,
    message: &str,
    result: &str,
    data: Value,
) -> PlanResultEnvelope {
    PlanResultEnvelope::from_envelope(CliEnvelope::error_with_data(
        "plan",
        status,
        code,
        message,
        None,
        with_contract_marker(data, result),
    ))
}

fn plan_with_reason(
    evaluation_id: Option<&str>,
    event_id: &str,
    tether_id: &str,
    tether_version: &str,
    result: &str,
    reason: &str,
) -> Value {
    let mut data = plan_base_data(evaluation_id, event_id, tether_id, tether_version);
    data["reason"] = Value::String(reason.to_owned());
    data["plan"] = Value::Null;
    data["result"] = Value::String(result.to_owned());
    data
}

fn plan_failure_data(result: &str) -> Value {
    let mut data = Map::new();
    data.extend(plan_control_fields());
    data.insert("plan".to_owned(), Value::Null);
    data.insert("result".to_owned(), Value::String(result.to_owned()));
    Value::Object(data)
}

fn plan_failure(
    status: OutcomeStatus,
    code: impl Into<String>,
    message: impl Into<String>,
    field: Option<String>,
) -> PlanResultEnvelope {
    let result = match status {
        OutcomeStatus::Interrupted => "interrupted",
        OutcomeStatus::Unavailable => "unavailable",
        OutcomeStatus::Failed => "failed",
        _ => "invalid_data",
    };
    PlanResultEnvelope::from_envelope(CliEnvelope::error_with_data(
        "plan",
        status,
        code,
        message,
        field,
        with_contract_marker(plan_failure_data(result), result),
    ))
}

fn input_failure(error: RunInputError) -> PlanResultEnvelope {
    plan_failure(
        OutcomeStatus::InvalidData,
        error.code.as_str(),
        error.message,
        error.field,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn action_plan(count: usize) -> Value {
        json!({
            "actions": (0..count)
                .map(|index| json!({"action_id": format!("action_{index}")}))
                .collect::<Vec<_>>()
        })
    }

    #[test]
    fn valid_plan_preserves_zero_one_and_multiple_actions() {
        for count in [0, 1, 2] {
            let envelope = success_envelope(
                "completed",
                plan_available_data(Some("eval"), "event", "tether", "1", &action_plan(count)),
            );
            assert_eq!(
                envelope.envelope.data["schema"], "tethers.plan/1",
                "schema marker must be stable"
            );
            assert_eq!(
                envelope.envelope.data["plan"]["actions"]
                    .as_array()
                    .map(Vec::len),
                Some(count)
            );
        }
    }

    #[test]
    fn no_actions_is_successful_and_has_no_proposed_plan() {
        let result = PlanResult::NoActions {
            evaluation_id: "eval".to_owned(),
            event_id: "event".to_owned(),
            tether_id: "tether".to_owned(),
            tether_version: "1".to_owned(),
            response: json!({"status": "not_matched"}),
        };
        let envelope = map_plan_result(result).envelope;
        assert_eq!(envelope.status, OutcomeStatus::NoActions);
        assert_eq!(envelope.data["result"], "no_actions");
        assert!(envelope.data["plan"].is_null());
        assert_eq!(envelope.data["execution"]["provider_invocations"], 0);
    }

    #[test]
    fn invalid_input_keeps_machine_schema_and_no_execution_claim() {
        let envelope = plan_failure(
            OutcomeStatus::InvalidData,
            "RUN_INPUT_INVALID_JSON",
            "input must be valid JSON",
            Some("/".to_owned()),
        )
        .envelope;
        assert_eq!(envelope.status, OutcomeStatus::InvalidData);
        assert_eq!(envelope.data["schema"], "tethers.plan/1");
        assert_eq!(envelope.data["authority"]["granted"], false);
        assert_eq!(envelope.data["execution"]["performed"], false);
        assert_eq!(envelope.data["execution"]["trail_execution_entries"], 0);
    }

    #[test]
    fn planner_error_preserves_correlated_error_and_zero_effects() {
        let result = PlanResult::PlannerError {
            evaluation_id: Some("eval".to_owned()),
            event_id: "event".to_owned(),
            tether_id: "tether".to_owned(),
            tether_version: "1".to_owned(),
            code: "type_error".to_owned(),
            message: "bad value".to_owned(),
        };
        let envelope = map_plan_result(result).envelope;
        assert_eq!(envelope.data["schema"], "tethers.plan/1");
        assert_eq!(envelope.data["error"]["code"], "type_error");
        assert_eq!(envelope.data["execution"]["replay_mutated"], false);
        assert_eq!(envelope.data["authority"]["status"], "not_requested");
    }
}
