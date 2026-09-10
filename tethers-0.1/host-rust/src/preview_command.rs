//! Read-only public preview coordinator.
//!
//! Preview validates the same public input and configured Tether used by
//! `run`, asks the retained Core engine for its proposal, and then shuts the
//! engine down. It never launches a provider, opens a Trail, requests
//! authority, or creates a replay/intent record.

use crate::configured_runtime::prepare_runtime;
use crate::host_execution::{HostExecutionService, PreparedEvaluationInput};
use crate::resolver::ProviderAvailability;
use crate::run_command::RunResult;
use crate::run_input::parse_run_input;
use crate::runtime_config::load_runtime_config;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use tethers_reference_host::cli::{CliEnvelope, OutcomeStatus};
use tethers_reference_host::engine_stdio::{EngineError, EngineSession, PlannerResponseWire};

pub struct PreviewCommandArgs {
    pub config: PathBuf,
    pub engine: PathBuf,
    pub input: PathBuf,
}

fn failure(status: OutcomeStatus, code: &str, message: impl Into<String>) -> RunResult {
    let envelope = CliEnvelope::error("preview", status, code, message, None);
    RunResult {
        exit_code: envelope.exit_code,
        envelope,
    }
}

fn canonical_file(path: &Path, label: &str) -> Result<PathBuf, RunResult> {
    let canonical = path.canonicalize().map_err(|_| {
        failure(
            OutcomeStatus::InvalidData,
            &format!("{label}_NOT_FOUND"),
            format!("{label} path was not found"),
        )
    })?;
    if !canonical.is_file() {
        return Err(failure(
            OutcomeStatus::InvalidData,
            &format!("{label}_NOT_FILE"),
            format!("{label} path must be a regular file"),
        ));
    }
    Ok(canonical)
}

fn engine_error(error: EngineError) -> RunResult {
    let (status, code) = match error {
        EngineError::Interrupted => (OutcomeStatus::Interrupted, "INTERRUPTED"),
        EngineError::ValidationFailed { .. } => (OutcomeStatus::InvalidData, "TETHER_INVALID"),
        EngineError::InitializeFailed(_) => {
            (OutcomeStatus::Unavailable, "ENGINE_INITIALIZE_FAILED")
        }
        _ => (OutcomeStatus::Unavailable, "ENGINE_UNAVAILABLE"),
    };
    failure(status, code, "preview engine operation failed")
}

pub fn run(args: PreviewCommandArgs) -> RunResult {
    let config = match canonical_file(&args.config, "CONFIG") {
        Ok(path) => path,
        Err(result) => return result,
    };
    let engine = match canonical_file(&args.engine, "ENGINE") {
        Ok(path) => path,
        Err(result) => return result,
    };
    let input_path = match canonical_file(&args.input, "RUN_INPUT") {
        Ok(path) => path,
        Err(result) => return result,
    };
    let input_text = match std::fs::read_to_string(&input_path) {
        Ok(text) => text,
        Err(_) => {
            return failure(
                OutcomeStatus::InvalidData,
                "RUN_INPUT_READ_FAILED",
                "cannot read preview input",
            )
        }
    };
    let input = match parse_run_input(&input_text) {
        Ok(input) => input,
        Err(error) => {
            return failure(
                OutcomeStatus::InvalidData,
                error.code.as_str(),
                error.to_string(),
            )
        }
    };
    let loaded = match load_runtime_config(&config) {
        Ok(loaded) => loaded,
        Err(error) => {
            return failure(
                OutcomeStatus::InvalidData,
                "CONFIG_LOAD_FAILED",
                error.to_string(),
            )
        }
    };
    let runtime = match prepare_runtime(&loaded) {
        Ok(runtime) => runtime,
        Err(error) => {
            return failure(
                OutcomeStatus::InvalidData,
                "RUNTIME_PREPARE_FAILED",
                error.to_string(),
            )
        }
    };
    let Some(tether) = runtime
        .tethers()
        .iter()
        .find(|tether| tether.id == input.tether.id && tether.version == input.tether.version)
    else {
        return failure(
            OutcomeStatus::InvalidData,
            "TETHER_NOT_FOUND",
            "selected Tether is not configured",
        );
    };
    let prepared_input = PreparedEvaluationInput {
        tether_id: input.tether.id.clone(),
        tether_version: input.tether.version.clone(),
        evaluation_id: input.evaluation_id.clone(),
        anchor_event: json!({
            "id": input.event.id,
            "name": input.event.name,
            "data": input.event.data,
        }),
        facts: input.facts,
    };
    let service = HostExecutionService::new(&runtime, &engine, Path::new("preview-no-trail"), None);
    let request = match service.build_core_request_envelope(
        &prepared_input,
        tether,
        &ProviderAvailability::empty(),
    ) {
        Ok(request) => request,
        Err(error) => {
            return failure(
                OutcomeStatus::InvalidData,
                "PREVIEW_REQUEST_FAILED",
                format!("{error:?}"),
            )
        }
    };
    let working_dir = engine.parent().unwrap_or_else(|| Path::new("."));
    let mut session = match EngineSession::launch(&engine, working_dir) {
        Ok(session) => session,
        Err(error) => return engine_error(error),
    };
    if let Err(error) = session.validate_tether(0, &tether.id, &tether.version, &tether.source) {
        session.shutdown();
        return engine_error(error);
    }
    let wire = match session.evaluate_tether(&input.evaluation_id, &request) {
        Ok(wire) => wire,
        Err(error) => {
            session.shutdown();
            return engine_error(error);
        }
    };
    session.shutdown();

    let (planner_status, proposal, planner_error) = match wire {
        PlannerResponseWire::Matched(response) => (
            "matched",
            response.get("plan").cloned().unwrap_or(Value::Null),
            Value::Null,
        ),
        PlannerResponseWire::NotMatched(response) => (
            "not_matched",
            Value::Null,
            response.get("error").cloned().unwrap_or(Value::Null),
        ),
        PlannerResponseWire::Error(response) => (
            "error",
            Value::Null,
            response.get("error").cloned().unwrap_or(Value::Null),
        ),
        PlannerResponseWire::Unknown { status, response } => (
            "unknown",
            Value::Null,
            json!({"status": status, "response": response}),
        ),
    };
    let data = json!({
        "phase": "preview",
        "input": {
            "parsed": true,
            "validated": true,
            "evaluation_id": input.evaluation_id,
            "tether": {"id": input.tether.id, "version": input.tether.version}
        },
        "planner_status": planner_status,
        "proposed_plan": proposal,
        "planner_error": planner_error,
        "authority": {"granted": false, "status": "not_requested"},
        "execution": {"performed": false, "provider_invocations": 0, "trail_written": false}
    });
    RunResult {
        envelope: CliEnvelope::ok("preview", data),
        exit_code: 0,
    }
}
