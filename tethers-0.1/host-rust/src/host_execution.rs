// J13B Packet 1 - typed host execution service and retained execution sessions.
//
// Extracts the host execution machinery from main.rs into a typed Rust
// application service.  Uses retained OCaml engine and MCP provider
// sessions for validation, evaluation, and dispatch.
//
// Public CLI ownership remains outside this service.  Evaluation IDs remain
// explicit host input and are never derived here.

use crate::configured_runtime::{PreparedRuntime, PreparedTether};
use crate::dispatch::{self, DispatchReadyAction};
use crate::executor::CapabilityExecutor;

use crate::manifest::BindingKind;
use crate::outcome::{self, MonotonicClock, ProductionMonotonicClock};
use crate::policy::{self, PermissionDecision, ProposedAction};
use crate::replay_runtime::FileReplayAuthority;
use crate::resolver::{self, ProviderAvailability, ResolvedCapability};
use crate::socket::{RetainedProviderSession, Socket, SocketEstablishment};
use crate::stdio_provider::StdioProviderError;
use serde_json::Value;

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;
use tethers_reference_host::child_process;
use tethers_reference_host::engine_stdio::{EngineError, EngineSession, PlannerResponseWire};

// ===========================================================================
// Prepared evaluation input
// ===========================================================================

/// One fully formed evaluation input for the execution service.
///
/// Every field is explicitly supplied by the caller.  The service does not
/// derive, generate, hash, or otherwise invent an evaluation ID.
#[derive(Debug, Clone)]
pub struct PreparedEvaluationInput {
    /// Exact configured Tether identity (must match a PreparedTether.id in
    /// the PreparedRuntime).
    pub tether_id: String,
    /// Exact configured Tether version (must match a PreparedTether.version
    /// in the PreparedRuntime).
    pub tether_version: String,
    /// Existing explicit evaluation ID.  The service does not generate,
    /// derive or rewrite this value.
    pub evaluation_id: String,
    /// The Anchor event that wakes this Tether.
    pub anchor_event: Value,
    /// Immutable Facts available to Conditions.
    pub facts: Value,
}

// ===========================================================================
// Typed service result
// ===========================================================================

/// Typed result from the execution service.
///
/// Distinguishes completed, denied, unavailable, failed, uncertain, and
/// interrupted outcomes without constructing CLI envelopes.
#[derive(Debug)]
pub enum ExecutionServiceResult {
    /// Action dispatched and completed successfully.
    Completed {
        evaluation_id: String,
        action_id: String,
        response: Value,
        execution_id: Option<String>,
    },
    /// Policy explicitly denied the action.
    Denied {
        evaluation_id: String,
        action_id: String,
        reason: String,
        execution_id: Option<String>,
    },
    /// The planner returned no actions.
    NoActions {
        evaluation_id: String,
        response: Value,
    },
    /// Policy is Ask - human approval required before dispatch.
    ApprovalRequired {
        evaluation_id: String,
        action_id: String,
        reason: String,
    },
    /// The planner returned a valid error response rather than a Plan.
    PlannerError {
        evaluation_id: Option<String>,
        code: String,
        message: String,
    },
    /// Capability is not currently available.
    Unavailable {
        evaluation_id: String,
        reason: String,
    },
    /// Definite provider failure.
    Failed {
        evaluation_id: String,
        action_id: String,
        reason: String,
        execution_id: Option<String>,
    },
    /// Post-invocation uncertainty.
    Uncertain {
        evaluation_id: String,
        action_id: String,
        reason: String,
        execution_id: Option<String>,
    },
    /// Outcome audit recording failed.
    AuditFailed {
        evaluation_id: String,
        action_id: String,
        reason: String,
        execution_id: Option<String>,
    },
    /// Deadline expired before the provider invocation boundary.
    Unattempted {
        evaluation_id: String,
        action_id: String,
        reason: String,
        execution_id: Option<String>,
    },
    /// A prior execution completed successfully; replay is blocked.
    ReplayBlockedCompletedSuccess {
        evaluation_id: String,
        action_id: String,
        execution_id: Option<String>,
    },
    /// A prior execution completed with known failure; replay is blocked.
    ReplayBlockedCompletedFailure {
        evaluation_id: String,
        action_id: String,
        execution_id: Option<String>,
    },
    /// Recovered claim, intent, armed, or uncertain state requires a human.
    ReplayRequiresManualResolution {
        evaluation_id: String,
        action_id: String,
        execution_id: Option<String>,
    },
    /// Replay storage or terminal publication could not be trusted.
    ReplayPersistenceUnavailable {
        evaluation_id: String,
        action_id: String,
        execution_id: Option<String>,
    },
    /// Interrupted before provider invocation.
    Interrupted,
    /// Invalid input data.
    InvalidData { message: String },
}

#[derive(Debug)]
enum PlannerErrorOutcome {
    Contextual {
        evaluation_id: String,
        code: String,
        message: String,
    },
    Request {
        code: String,
        message: String,
    },
}

#[derive(Debug)]
enum PlannerOutcome {
    Matched(Value),
    NotMatched {
        evaluation_id: String,
        response: Value,
    },
    Error(PlannerErrorOutcome),
}

// ===========================================================================
// Service error
// ===========================================================================

/// Errors from the execution service that prevent any evaluation.
#[derive(Debug)]
pub enum ExecutionServiceError {
    Engine(EngineError),
    Provider(String),
    TetherValidation(String),
    InvalidInput(String),
    Interrupted,
}

fn selected_tether_indexes(
    tethers: &[PreparedTether],
    inputs: &[PreparedEvaluationInput],
) -> Result<Vec<usize>, ExecutionServiceError> {
    let mut indexes = Vec::new();
    for input in inputs {
        let Some((index, _)) = tethers.iter().enumerate().find(|(_, tether)| {
            tether.id == input.tether_id && tether.version == input.tether_version
        }) else {
            return Err(ExecutionServiceError::InvalidInput(format!(
                "selected tether is not configured: {} v{}",
                input.tether_id, input.tether_version
            )));
        };
        if !indexes.contains(&index) {
            indexes.push(index);
        }
    }
    Ok(indexes)
}

impl std::fmt::Display for ExecutionServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Engine(e) => write!(f, "engine error: {e}"),
            Self::Provider(m) => write!(f, "provider error: {m}"),
            Self::TetherValidation(m) => write!(f, "tether validation: {m}"),
            Self::InvalidInput(m) => write!(f, "invalid input: {m}"),
            Self::Interrupted => write!(f, "interrupted"),
        }
    }
}

impl std::error::Error for ExecutionServiceError {}

impl From<EngineError> for ExecutionServiceError {
    fn from(e: EngineError) -> Self {
        match e {
            EngineError::Interrupted => Self::Interrupted,
            EngineError::ValidationFailed { message, .. } => Self::TetherValidation(message),
            other => Self::Engine(other),
        }
    }
}

// ===========================================================================
// Provider session executor
// ===========================================================================

/// A `CapabilityExecutor` that dispatches through a retained provider session.
struct ProviderSessionExecutor<'a> {
    session: &'a mut RetainedProviderSession,
    tool_name: String,
}

fn classify_provider_error(error: &StdioProviderError) -> outcome::ProviderDiagnostic {
    match error {
        StdioProviderError::ExplicitProviderError(_) => {
            outcome::ProviderDiagnostic::ExplicitProviderError
        }
        StdioProviderError::Interrupted => outcome::ProviderDiagnostic::ProtocolInterrupted,
        StdioProviderError::EmptyResponse
        | StdioProviderError::CatalogueStale
        | StdioProviderError::CatalogueChangedDuringDiscovery
        | StdioProviderError::MalformedResponse(_)
        | StdioProviderError::ProtocolError(_)
        | StdioProviderError::ReadFailed(_)
        | StdioProviderError::WriteFailed(_)
        | StdioProviderError::LaunchFailed { .. }
        | StdioProviderError::StdinUnavailable
        | StdioProviderError::StdoutUnavailable
        | StdioProviderError::SerializeFailed(_)
        | StdioProviderError::TrustedManifestInvalid(_)
        | StdioProviderError::AdmissionFailed(_) => outcome::ProviderDiagnostic::NoFinalResponse,
    }
}

impl CapabilityExecutor for ProviderSessionExecutor<'_> {
    fn provider_identity(&self) -> &str {
        self.session.identity()
    }

    fn execute(&mut self, ready: &DispatchReadyAction) -> Result<Value, String> {
        let arguments = ready.arguments();
        self.session
            .tools_call(&self.tool_name, arguments, Duration::from_secs(10))
            .map_err(|e| format!("provider tools/call failed: {e}"))
    }

    fn execute_classified(
        &mut self,
        ready: &DispatchReadyAction,
        remaining: Duration,
    ) -> Result<Value, outcome::ProviderDiagnostic> {
        self.session
            .tools_call(&self.tool_name, ready.arguments(), remaining)
            .map_err(|error| classify_provider_error(&error))
    }
}

// ===========================================================================
// Host execution service
// ===========================================================================

/// Typed host execution service.
///
/// Validates configured Tethers, evaluates fully formed Tethers 0.1 requests,
/// applies capability resolution, scope and policy rules, and invokes
/// permitted Actions through retained provider sessions.
pub struct HostExecutionService<'a> {
    runtime: &'a PreparedRuntime,
    engine_path: &'a Path,
    trail_path: &'a Path,
    host_data_root: Option<&'a Path>,
}

/// Execute one already-planned Action through the existing host boundary using
/// an exact enabled installed binding. This is an adapter seam, not a second
/// dispatcher: policy, intent, replay, outcome, Result Anchor, and Trail all
/// remain in `execute_shared_boundary`.
pub fn execute_enabled_file_tools_action(
    response: &mut Value,
    requirements: &[policy::CapabilityRequirement],
    resolved: &ResolvedCapability,
    enabled: &crate::enablement::EnabledBindingSnapshot,
    executor: &mut dyn CapabilityExecutor,
    trail_path: &Path,
    replay_root: &Path,
    event_id: &str,
) -> Result<crate::SharedExecutionResult, Box<dyn std::error::Error>> {
    execute_enabled_installed_action(
        response,
        requirements,
        resolved,
        enabled,
        executor,
        trail_path,
        replay_root,
        event_id,
    )
}

/// Generic installed-provider execution entry point. The body is the same as the
/// historical file-tools-only adapter; it accepts any `CapabilityExecutor` and
/// `EnabledBindingSnapshot` without assuming a specific Plug.
pub fn execute_enabled_installed_action(
    response: &mut Value,
    requirements: &[policy::CapabilityRequirement],
    resolved: &ResolvedCapability,
    enabled: &crate::enablement::EnabledBindingSnapshot,
    executor: &mut dyn CapabilityExecutor,
    trail_path: &Path,
    replay_root: &Path,
    event_id: &str,
) -> Result<crate::SharedExecutionResult, Box<dyn std::error::Error>> {
    let proposed = crate::extract_proposed_action(response)?;
    let action = crate::extract_single_action(response)?.clone();
    if !enabled.contains(
        resolved.capability_name(),
        resolved.capability_version(),
        &resolved.manifest().manifest().binding.tool_name,
        resolved.manifest_digest(),
    ) || proposed.capability_name != resolved.capability_name()
    {
        return Err("enabled installed binding does not match planned capability".into());
    }
    let policy = policy::HostLocalPolicy::new(policy::PolicyRule::Allow);
    let decision = policy::evaluate_permission_resolved(requirements, resolved, &policy);
    let mut trail = dispatch::FileTrail::open(trail_path)?;
    let clock = ProductionMonotonicClock::new();
    let mut replay_authority = FileReplayAuthority::new(Some(replay_root));
    let mut anchor_writer = crate::ResponseResultAnchorWriter;
    let context = crate::InputEventContext::for_initial(event_id);
    crate::execute_shared_boundary(
        response,
        &action,
        decision,
        resolved,
        &mut trail,
        executor,
        &context,
        true,
        &clock,
        &mut replay_authority,
        None,
        &mut anchor_writer,
        None,
    )
}

impl<'a> HostExecutionService<'a> {
    /// Create a new service with the given immutable runtime, engine, trail,
    /// and host-data-root references.
    pub fn new(
        runtime: &'a PreparedRuntime,
        engine_path: &'a Path,
        trail_path: &'a Path,
        host_data_root: Option<&'a Path>,
    ) -> Self {
        Self {
            runtime,
            engine_path,
            trail_path,
            host_data_root,
        }
    }

    /// Expose the runtime reference for benchmark setup.
    ///
    /// This method exists solely for B0-C benchmark construction. It is not
    /// part of the production API surface.
    #[doc(hidden)]
    pub fn runtime(&self) -> &PreparedRuntime {
        self.runtime
    }

    /// Expose the engine path for benchmark setup.
    #[doc(hidden)]
    pub fn engine_path(&self) -> &Path {
        self.engine_path
    }

    /// Expose the trail path for benchmark setup.
    #[doc(hidden)]
    pub fn trail_path(&self) -> &Path {
        self.trail_path
    }

    /// Run a single evaluation through the warm engine and provider sessions.
    ///
    /// This method exists solely for B0-C benchmark measurement. The caller
    /// must have already launched and warmed the engine and provider sessions
    /// through the normal production setup path. This method is not part of
    /// the production API surface.
    #[doc(hidden)]
    pub fn bench_evaluate_one(
        &self,
        input: &PreparedEvaluationInput,
        engine: &mut EngineSession,
        provider_sessions: &mut HashMap<String, RetainedProviderSession>,
        provider_availability: &ProviderAvailability,
        approvals: &mut crate::approval::ApprovalStore,
        replay_authority: &mut dyn crate::replay_runtime::ReplayAuthority,
    ) -> ExecutionServiceResult {
        self.evaluate_one(
            input,
            engine,
            provider_sessions,
            provider_availability,
            approvals,
            replay_authority,
        )
    }

    /// Launch, initialize, and validate the warm state for benchmarking.
    ///
    /// Returns the warmed engine, provider sessions, and availability.
    /// This method exists solely for B0-C benchmark measurement.
    #[doc(hidden)]
    pub fn bench_warmup(
        &self,
    ) -> Result<
        (
            EngineSession,
            HashMap<String, RetainedProviderSession>,
            ProviderAvailability,
        ),
        ExecutionServiceError,
    > {
        let engine_working_dir = self
            .engine_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));
        let mut engine = EngineSession::launch(self.engine_path, &engine_working_dir)?;

        // Validate all tethers
        for (i, tether) in self.runtime.tethers().iter().enumerate() {
            engine.validate_tether(i, &tether.id, &tether.version, &tether.source)?;
        }

        // Launch providers
        let mut provider_sessions: HashMap<String, RetainedProviderSession> = HashMap::new();
        let mut provider_availability = ProviderAvailability::empty();

        for prepared_provider in self.runtime.providers() {
            if let Some(session) = self.launch_and_initialize_provider(prepared_provider)? {
                let identity = session.identity().to_owned();
                provider_sessions.insert(identity.clone(), session);
                let ids: Vec<String> = provider_sessions.keys().cloned().collect();
                provider_availability = ProviderAvailability::from_identities(ids);
            }
        }

        Ok((engine, provider_sessions, provider_availability))
    }

    /// Run the execution service end-to-end.
    ///
    /// Ordering:
    /// 1. Launch retained engine
    /// 2. Validate every configured Tether exactly once
    /// 3. Launch and initialize every provider exactly once
    /// 4. For each evaluation input: evaluate via engine, then dispatch
    ///    through all existing gates
    /// 5. Cleanup all retained children
    pub fn run(
        &self,
        inputs: &[PreparedEvaluationInput],
    ) -> Result<Vec<ExecutionServiceResult>, ExecutionServiceError> {
        let tether_indexes = (0..self.runtime.tethers().len()).collect::<Vec<_>>();
        self.run_with_tether_indexes(inputs, &tether_indexes)
    }

    /// Run a caller-selected subset of configured Tethers.
    ///
    /// This is the public-command seam: it resolves every requested Tether
    /// before any child process is launched, then validates only that exact
    /// subset.  The historical [`Self::run`] method deliberately retains its
    /// whole-runtime validation behaviour for existing callers.
    pub fn run_selected(
        &self,
        inputs: &[PreparedEvaluationInput],
    ) -> Result<Vec<ExecutionServiceResult>, ExecutionServiceError> {
        let tether_indexes = selected_tether_indexes(self.runtime.tethers(), inputs)?;
        self.run_with_tether_indexes(inputs, &tether_indexes)
    }

    fn run_with_tether_indexes(
        &self,
        inputs: &[PreparedEvaluationInput],
        tether_indexes: &[usize],
    ) -> Result<Vec<ExecutionServiceResult>, ExecutionServiceError> {
        // --- Interruption guard ---
        if child_process::is_interrupted() {
            return Err(ExecutionServiceError::Interrupted);
        }

        // --- 1. Launch retained engine ---
        let engine_working_dir = self
            .engine_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));
        let mut engine = EngineSession::launch(self.engine_path, &engine_working_dir)?;

        // --- 2. Validate the requested configured Tethers ---
        for &i in tether_indexes {
            if child_process::is_interrupted() {
                engine.shutdown();
                return Err(ExecutionServiceError::Interrupted);
            }
            let tether = &self.runtime.tethers()[i];
            engine.validate_tether(i, &tether.id, &tether.version, &tether.source)?;
        }

        // --- 3. Launch and initialize providers ---
        if child_process::is_interrupted() {
            engine.shutdown();
            return Err(ExecutionServiceError::Interrupted);
        }

        let mut provider_sessions: HashMap<String, RetainedProviderSession> = HashMap::new();
        let mut provider_availability = ProviderAvailability::empty();

        for prepared_provider in self.runtime.providers() {
            if child_process::is_interrupted() {
                engine.shutdown();
                for (_, mut session) in provider_sessions {
                    session.close();
                }
                return Err(ExecutionServiceError::Interrupted);
            }

            let Some(session) = self.launch_and_initialize_provider(prepared_provider)? else {
                // A live provider whose discovery evidence does not match the
                // trusted prepared capability is not available for policy or
                // bridge projection.
                continue;
            };
            let identity = session.identity().to_owned();
            provider_sessions.insert(identity.clone(), session);

            // Build availability set.
            let ids: Vec<String> = provider_sessions.keys().cloned().collect();
            provider_availability = ProviderAvailability::from_identities(ids);
        }

        // --- 4. Evaluate each input ---
        // The store is process-local to this service invocation.  An Ask
        // result deliberately never exposes its internal approval identity.
        let mut approvals = crate::approval::ApprovalStore::default();
        let mut replay_authority = FileReplayAuthority::new(self.host_data_root);
        let mut results = Vec::with_capacity(inputs.len());

        for input in inputs {
            if child_process::is_interrupted() {
                // Pre-boundary interruption
                results.push(ExecutionServiceResult::Interrupted);
                break;
            }

            let result = self.evaluate_one(
                input,
                &mut engine,
                &mut provider_sessions,
                &provider_availability,
                &mut approvals,
                &mut replay_authority,
            );
            results.push(result);
        }

        // --- 5. Cleanup ---
        engine.shutdown();
        for (_, mut session) in provider_sessions {
            session.close();
        }

        Ok(results)
    }

    /// Launch, initialize, and list tools for one prepared provider.
    fn launch_and_initialize_provider(
        &self,
        prepared: &crate::configured_runtime::PreparedProvider,
    ) -> Result<Option<RetainedProviderSession>, ExecutionServiceError> {
        let config = &prepared.stdio_config;

        let expected_server_name = prepared
            .capabilities
            .first()
            .map(|c| c.verified_manifest.manifest().binding.server_name.as_str())
            .unwrap_or("");
        let mut session = RetainedProviderSession::establish(SocketEstablishment {
            command: &config.command,
            args: &config.args,
            working_directory: &prepared.working_directory,
            protocol_version: &config.protocol_version,
            server_name: expected_server_name,
            identity: &prepared.identity,
        })
        .map_err(|e| provider_service_error("establish", &prepared.identity, e))?;

        if !refresh_prepared_catalogue(prepared, &mut session)
            .map_err(|e| provider_service_error("tools/list", &prepared.identity, e))?
        {
            session.close();
            return Ok(None);
        }

        Ok(Some(session))
    }

    /// Evaluate one prepared input through the engine and dispatch pipeline.
    fn evaluate_one(
        &self,
        input: &PreparedEvaluationInput,
        engine: &mut EngineSession,
        provider_sessions: &mut HashMap<String, RetainedProviderSession>,
        provider_availability: &ProviderAvailability,
        approvals: &mut crate::approval::ApprovalStore,
        replay_authority: &mut dyn crate::replay_runtime::ReplayAuthority,
    ) -> ExecutionServiceResult {
        // Find the matching Tether in the runtime.
        let tether = match self.find_tether(&input.tether_id, &input.tether_version) {
            Ok(t) => t,
            Err(result) => return result,
        };

        // Build the extended Core request envelope.
        let envelope = match crate::bench_timing::timed("envelope_build", || {
            self.build_core_request_envelope(input, tether, provider_availability)
        }) {
            Ok(envelope) => envelope,
            Err(result) => return result,
        };

        // Call tethers.evaluate (now Core).
        let wire_response = match crate::bench_timing::timed("core_mcp", || {
            engine.evaluate_tether(&input.evaluation_id, &envelope)
        }) {
            Ok(resp) => resp,
            Err(error) => return Self::classify_engine_evaluation_failure(input, error),
        };

        let outcome = Self::classify_planner_response(input, wire_response);
        Self::route_planner_outcome(outcome, |matched| {
            self.dispatch_matched_plan(
                input,
                matched,
                provider_sessions,
                provider_availability,
                approvals,
                replay_authority,
            )
        })
    }

    /// Find a configured Tether by identity.
    fn find_tether(
        &self,
        id: &str,
        version: &str,
    ) -> Result<&PreparedTether, ExecutionServiceResult> {
        self.runtime
            .tethers()
            .iter()
            .find(|t| t.id == id && t.version == version)
            .ok_or_else(|| ExecutionServiceResult::InvalidData {
                message: format!("tether not found: {id} v{version}"),
            })
    }

    /// Build the Tethers 0.1 request envelope for one evaluation.
    fn build_request_envelope(
        &self,
        input: &PreparedEvaluationInput,
        tether: &PreparedTether,
        provider_availability: &ProviderAvailability,
    ) -> Result<Value, ExecutionServiceResult> {
        let capabilities = self.runtime.planner_capabilities().map_err(|error| {
            ExecutionServiceResult::InvalidData {
                message: format!("planner capability projection failed: {error}"),
            }
        })?;
        let mut request = assemble_request_envelope(input, tether, capabilities);
        crate::inject_bridge_projection_into_request(
            &mut request,
            self.runtime.trusted_store(),
            provider_availability,
        )
        .map_err(|error| ExecutionServiceResult::InvalidData {
            message: format!("bridge capability projection failed: {error}"),
        })?;
        Ok(request)
    }

    /// Build the extended Core request envelope for one evaluation.
    ///
    /// Reuses the existing request assembly pipeline, then inserts the
    /// `core_environment` JSON from the configured PreparedTether.
    /// Fails explicitly when no core_environment is present.
    ///
    /// The proof matters because the Core request must contain the SAME
    /// runtime capability projection that real production request
    /// construction would use.
    pub(crate) fn build_core_request_envelope(
        &self,
        input: &PreparedEvaluationInput,
        tether: &PreparedTether,
        provider_availability: &ProviderAvailability,
    ) -> Result<Value, ExecutionServiceResult> {
        let mut request = self.build_request_envelope(input, tether, provider_availability)?;
        let core_env =
            tether
                .core_environment_json()
                .ok_or_else(|| ExecutionServiceResult::InvalidData {
                    message: format!(
                        "tether {} v{} has no core_environment",
                        tether.id, tether.version
                    ),
                })?;
        request
            .as_object_mut()
            .ok_or_else(|| ExecutionServiceResult::InvalidData {
                message: "request envelope is not a JSON object".to_owned(),
            })?
            .insert("core_environment".to_owned(), core_env);
        Ok(request)
    }

    fn route_planner_outcome<F>(
        outcome: Result<PlannerOutcome, ExecutionServiceResult>,
        dispatch: F,
    ) -> ExecutionServiceResult
    where
        F: FnOnce(Value) -> ExecutionServiceResult,
    {
        match outcome {
            Ok(PlannerOutcome::Matched(response)) => dispatch(response),
            Ok(PlannerOutcome::NotMatched {
                evaluation_id,
                response,
            }) => ExecutionServiceResult::NoActions {
                evaluation_id,
                response,
            },
            Ok(PlannerOutcome::Error(PlannerErrorOutcome::Contextual {
                evaluation_id,
                code,
                message,
            })) => ExecutionServiceResult::PlannerError {
                evaluation_id: Some(evaluation_id),
                code,
                message,
            },
            Ok(PlannerOutcome::Error(PlannerErrorOutcome::Request { code, message })) => {
                ExecutionServiceResult::PlannerError {
                    evaluation_id: None,
                    code,
                    message,
                }
            }
            Err(result) => result,
        }
    }

    /// Keep engine transport failures distinct from valid planner error data.
    fn classify_engine_evaluation_failure(
        input: &PreparedEvaluationInput,
        error: EngineError,
    ) -> ExecutionServiceResult {
        match error {
            EngineError::Interrupted => ExecutionServiceResult::Interrupted,
            _ => ExecutionServiceResult::Unavailable {
                evaluation_id: input.evaluation_id.clone(),
                reason: "engine evaluation unavailable".to_owned(),
            },
        }
    }

    fn classify_planner_response(
        input: &PreparedEvaluationInput,
        wire: PlannerResponseWire,
    ) -> Result<PlannerOutcome, ExecutionServiceResult> {
        match wire {
            PlannerResponseWire::Matched(response) => {
                Self::validate_planner_correlation(input, &response)
                    .map_err(|msg| ExecutionServiceResult::InvalidData { message: msg })?;
                Ok(PlannerOutcome::Matched(response))
            }
            PlannerResponseWire::NotMatched(response) => {
                Self::validate_planner_correlation(input, &response)
                    .map_err(|msg| ExecutionServiceResult::InvalidData { message: msg })?;
                Ok(PlannerOutcome::NotMatched {
                    evaluation_id: input.evaluation_id.clone(),
                    response,
                })
            }
            PlannerResponseWire::Error(response) => {
                Self::require_planner_field(&response, "protocol_version", "0.1")
                    .map_err(|msg| ExecutionServiceResult::InvalidData { message: msg })?;
                let correlation_fields =
                    ["evaluation_id", "event_id", "tether_id", "tether_version"];
                if correlation_fields
                    .iter()
                    .any(|field| response.get(*field).is_some())
                {
                    Self::validate_planner_correlation(input, &response)
                        .map_err(|msg| ExecutionServiceResult::InvalidData { message: msg })?;
                    let error = response
                        .get("error")
                        .and_then(Value::as_object)
                        .ok_or_else(|| ExecutionServiceResult::InvalidData {
                            message: "planner error response requires error object".to_owned(),
                        })?;
                    let code = error.get("code").and_then(Value::as_str).ok_or_else(|| {
                        ExecutionServiceResult::InvalidData {
                            message: "planner error response requires string error.code".to_owned(),
                        }
                    })?;
                    let message =
                        error
                            .get("message")
                            .and_then(Value::as_str)
                            .ok_or_else(|| ExecutionServiceResult::InvalidData {
                                message: "planner error response requires string error.message"
                                    .to_owned(),
                            })?;
                    Ok(PlannerOutcome::Error(PlannerErrorOutcome::Contextual {
                        evaluation_id: input.evaluation_id.clone(),
                        code: code.to_owned(),
                        message: message.to_owned(),
                    }))
                } else {
                    let error = response
                        .get("error")
                        .and_then(Value::as_object)
                        .ok_or_else(|| ExecutionServiceResult::InvalidData {
                            message: "planner error response requires error object".to_owned(),
                        })?;
                    let code = error.get("code").and_then(Value::as_str).ok_or_else(|| {
                        ExecutionServiceResult::InvalidData {
                            message: "planner error response requires string error.code".to_owned(),
                        }
                    })?;
                    let message =
                        error
                            .get("message")
                            .and_then(Value::as_str)
                            .ok_or_else(|| ExecutionServiceResult::InvalidData {
                                message: "planner error response requires string error.message"
                                    .to_owned(),
                            })?;
                    Ok(PlannerOutcome::Error(PlannerErrorOutcome::Request {
                        code: code.to_owned(),
                        message: message.to_owned(),
                    }))
                }
            }
            PlannerResponseWire::Unknown { status, .. } => {
                Err(ExecutionServiceResult::InvalidData {
                    message: format!("planner response has unknown status '{status}'"),
                })
            }
        }
    }

    fn validate_planner_correlation(
        input: &PreparedEvaluationInput,
        response: &Value,
    ) -> Result<(), String> {
        let event_id = input
            .anchor_event
            .get("id")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| "submitted Anchor event requires non-empty string id".to_owned())?;
        for (field, expected) in [
            ("protocol_version", "0.1"),
            ("evaluation_id", input.evaluation_id.as_str()),
            ("event_id", event_id),
            ("tether_id", input.tether_id.as_str()),
            ("tether_version", input.tether_version.as_str()),
        ] {
            Self::require_planner_field(response, field, expected)?;
        }
        Ok(())
    }

    fn require_planner_field(
        response: &Value,
        field: &'static str,
        expected: &str,
    ) -> Result<(), String> {
        match response.get(field).and_then(Value::as_str) {
            Some(actual) if actual == expected => Ok(()),
            Some(actual) => Err(format!(
                "planner response correlation mismatch for {field}: expected '{expected}', got '{actual}'"
            )),
            None => Err(format!(
                "planner response correlation requires string {field}"
            )),
        }
    }

    /// Dispatch a matched response through the one accepted J05-J11 boundary.
    ///
    /// The whole plan executes through one deterministic schedule: ordinary
    /// sequential Actions keep the existing stop-on-first-non-success
    /// behaviour, and every `together` group attempts all of its members once
    /// in source order before the join is decided; a non-success join blocks
    /// every later item.  This is the C1 reference schedule — group members
    /// run serially, which is one valid schedule for a future genuinely
    /// concurrent runtime.
    fn dispatch_matched_plan(
        &self,
        input: &PreparedEvaluationInput,
        mut response: Value,
        provider_sessions: &mut HashMap<String, RetainedProviderSession>,
        provider_availability: &ProviderAvailability,
        approvals: &mut crate::approval::ApprovalStore,
        replay_authority: &mut dyn crate::replay_runtime::ReplayAuthority,
    ) -> ExecutionServiceResult {
        // Strip any planner-supplied execution_id or _host_execution_id before
        // processing. Only the replay-admission identity may populate trusted evidence.
        if let Some(obj) = response.as_object_mut() {
            obj.remove("execution_id");
            obj.remove("_host_execution_id");
        }
        let plan = match response.get("plan") {
            Some(plan) => plan,
            None => {
                return ExecutionServiceResult::InvalidData {
                    message: "matched response had no plan".to_owned(),
                };
            }
        };
        let actions: Vec<Value> = match plan.get("actions").and_then(Value::as_array) {
            Some(actions) => actions.clone(),
            None => {
                return ExecutionServiceResult::InvalidData {
                    message: "plan had no actions".to_owned(),
                };
            }
        };
        let groups = match plan.get("groups") {
            // Absent optional field: an ordinary sequential plan, exactly as
            // pre-C1.  A present non-array value is malformed metadata and must
            // never be silently reinterpreted as sequential execution.
            None => None,
            Some(Value::Array(groups)) => Some(groups.as_slice()),
            Some(_) => {
                return ExecutionServiceResult::InvalidData {
                    message: "plan.groups was not an array".to_owned(),
                };
            }
        };
        let items = match crate::plan_execution::build_plan_schedule(&actions, groups) {
            Ok(items) => items,
            Err(message) => {
                return ExecutionServiceResult::InvalidData { message };
            }
        };
        let evaluation_id = match response.get("evaluation_id").and_then(Value::as_str) {
            Some(evaluation_id) => evaluation_id.to_owned(),
            None => {
                return ExecutionServiceResult::InvalidData {
                    message: "response had no evaluation_id".to_owned(),
                };
            }
        };
        if let Some(parent) = self.trail_path.parent() {
            if let Err(error) = fs::create_dir_all(parent) {
                return ExecutionServiceResult::AuditFailed {
                    evaluation_id,
                    action_id: String::new(),
                    reason: format!("trail directory create failed: {error}"),
                    execution_id: None,
                };
            }
        }
        let mut trail = match dispatch::FileTrail::open(self.trail_path) {
            Ok(trail) => trail,
            Err(error) => {
                return ExecutionServiceResult::AuditFailed {
                    evaluation_id,
                    action_id: String::new(),
                    reason: format!("trail open failed: {error}"),
                    execution_id: None,
                };
            }
        };

        // Execute the plan: Sequential items use the existing serial path,
        // Group items use the concurrent C2-A3a path.
        let mut response = response;
        let mut last_succeeded: Option<(String, Option<String>)> = None;

        for item in &items {
            match item {
                crate::plan_execution::PlanItem::Sequential { action_index } => {
                    let action_id = crate::plan_execution::action_id_of(&actions, *action_index);
                    let _position = dispatch::SemanticPosition {
                        action_ordinal: *action_index as u64,
                        group_id: None,
                        member_ordinal: None,
                        phase: dispatch::SemanticPhase::Action,
                    };
                    let proposed =
                        match crate::extract_proposed_action_at(&mut response, *action_index) {
                            Ok(proposed) => proposed,
                            Err(error) => {
                                return ExecutionServiceResult::InvalidData {
                                    message: format!("invalid planned Action: {error}"),
                                };
                            }
                        };
                    let step = match self.execute_one_action(
                        &mut response,
                        &actions[*action_index],
                        proposed,
                        input,
                        provider_sessions,
                        provider_availability,
                        approvals,
                        &mut trail,
                        replay_authority,
                    ) {
                        Ok(result) => crate::plan_execution::ActionStep::Boundary(result),
                        Err(result) => crate::plan_execution::ActionStep::Stopped(result),
                    };
                    if crate::plan_execution::step_succeeded(&step) {
                        last_succeeded = Some((
                            action_id,
                            crate::plan_execution::succeeded_execution_id(&step),
                        ));
                        continue;
                    }
                    return crate::plan_execution::aggregate_step(step, &evaluation_id, &action_id);
                }
                crate::plan_execution::PlanItem::Group {
                    group_id,
                    member_indexes,
                } => {
                    return execute_group_concurrent(
                        group_id,
                        member_indexes,
                        &actions,
                        &mut response,
                        &evaluation_id,
                        &mut trail,
                        self,
                        input,
                        provider_sessions,
                        provider_availability,
                        approvals,
                        replay_authority,
                    );
                }
            }
        }

        // All items succeeded — return Completed.
        let (action_id, execution_id) = match last_succeeded {
            Some(identity) => identity,
            None => {
                return ExecutionServiceResult::AuditFailed {
                    evaluation_id,
                    action_id: String::new(),
                    reason: "plan completed without any succeeded Action".to_owned(),
                    execution_id: None,
                };
            }
        };
        ExecutionServiceResult::Completed {
            evaluation_id,
            action_id,
            response,
            execution_id,
        }
    }

    /// Run one planned Action through the full production boundary: scope
    /// assessment, effective policy (Deny / Ask / Unavailable / Allow),
    /// exact capability resolution, retained MCP session and catalogue
    /// refresh, then the shared execution boundary.
    ///
    /// Returns the boundary result as `Ok`; any stop before the boundary
    /// (policy, availability, approval, malformed data) is `Err` with the
    /// exact service result.
    #[allow(clippy::too_many_arguments)]
    fn execute_one_action(
        &self,
        response: &mut Value,
        action: &Value,
        proposed: policy::ProposedAction,
        input: &PreparedEvaluationInput,
        provider_sessions: &mut HashMap<String, RetainedProviderSession>,
        provider_availability: &ProviderAvailability,
        approvals: &mut crate::approval::ApprovalStore,
        trail: &mut dyn dispatch::Trail,
        replay_authority: &mut dyn crate::replay_runtime::ReplayAuthority,
    ) -> Result<crate::SharedExecutionResult, ExecutionServiceResult> {
        let evaluation_id = proposed.evaluation_id.clone();
        let action_id = proposed.action_id.clone();
        let (scope_assessment, policy_evaluation) =
            crate::bench_timing::timed("scope_policy", || {
                let scope_assessment = self.runtime.assess_action_scope(&proposed);
                let policy_evaluation = policy::evaluate_effective_policy(
                    &proposed,
                    self.runtime.requirements(),
                    self.runtime.trusted_store(),
                    provider_availability,
                    self.runtime.policy(),
                    scope_assessment,
                );
                (scope_assessment, policy_evaluation)
            });

        match &policy_evaluation.decision {
            PermissionDecision::Deny => {
                return Err(ExecutionServiceResult::Denied {
                    evaluation_id,
                    action_id,
                    reason: format!("{:?}", policy_evaluation.reason),
                    execution_id: None,
                });
            }
            PermissionDecision::Ask => {
                match crate::request_exact_approval(
                    &proposed,
                    self.runtime.requirements(),
                    self.runtime.trusted_store(),
                    provider_availability,
                    self.runtime.policy(),
                    scope_assessment,
                    approvals,
                    trail,
                ) {
                    Ok(Some(_)) => {
                        return Err(approval_required_result(
                            evaluation_id,
                            action_id,
                            &policy_evaluation.reason,
                        ));
                    }
                    Ok(None) => {
                        return Err(ExecutionServiceResult::AuditFailed {
                            evaluation_id,
                            action_id,
                            reason: "approval request could not be established".to_owned(),
                            execution_id: None,
                        });
                    }
                    Err(_) => {
                        return Err(ExecutionServiceResult::AuditFailed {
                            evaluation_id,
                            action_id,
                            reason: "approval request Trail recording failed".to_owned(),
                            execution_id: None,
                        });
                    }
                }
            }
            PermissionDecision::Unavailable => {
                return Err(ExecutionServiceResult::Unavailable {
                    evaluation_id,
                    reason: format!("{:?}", policy_evaluation.reason),
                });
            }
            PermissionDecision::Allow(_) => {}
        }

        let resolved = crate::bench_timing::timed("capability_resolve", || {
            self.resolve_exact_capability(&proposed, provider_availability)
        })?;
        let binding = &resolved.manifest().manifest().binding;
        if binding.kind != BindingKind::Mcp {
            return Err(ExecutionServiceResult::Denied {
                evaluation_id,
                action_id,
                reason: "capability binding is not MCP".to_owned(),
                execution_id: None,
            });
        }
        let session = match provider_sessions.get_mut(resolved.provider_identity()) {
            Some(session) => session,
            None => {
                return Err(ExecutionServiceResult::Unavailable {
                    evaluation_id,
                    reason: format!(
                        "provider '{}' has no retained session",
                        resolved.provider_identity()
                    ),
                });
            }
        };
        let Some(prepared_provider) = self
            .runtime
            .providers()
            .iter()
            .find(|provider| provider.identity == resolved.provider_identity())
        else {
            return Err(ExecutionServiceResult::Unavailable {
                evaluation_id,
                reason: format!(
                    "provider '{}' has no prepared catalogue authority",
                    resolved.provider_identity()
                ),
            });
        };
        crate::bench_timing::timed("catalogue_refresh", || {
            match refresh_prepared_catalogue(prepared_provider, session) {
                Ok(true) => Ok(()),
                Ok(false) => {
                    return Err(ExecutionServiceResult::Unavailable {
                        evaluation_id: evaluation_id.clone(),
                        reason: format!(
                            "provider '{}' catalogue no longer matches trusted bindings",
                            resolved.provider_identity()
                        ),
                    });
                }
                Err(error) => {
                    return Err(ExecutionServiceResult::Unavailable {
                        evaluation_id: evaluation_id.clone(),
                        reason: format!(
                            "provider '{}' catalogue refresh unavailable: {error}",
                            resolved.provider_identity()
                        ),
                    });
                }
            }
        })?;
        let mut executor = ProviderSessionExecutor {
            session,
            tool_name: binding.tool_name.clone(),
        };
        let event_id = match input
            .anchor_event
            .get("id")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
        {
            Some(event_id) => event_id,
            None => {
                return Err(ExecutionServiceResult::InvalidData {
                    message: "Anchor event requires a non-empty string id".to_owned(),
                });
            }
        };
        let input_context = crate::InputEventContext::for_initial(event_id);
        let clock = ProductionMonotonicClock::new();
        let mut anchor_writer = crate::ResponseResultAnchorWriter;
        let shared_result = crate::bench_timing::timed("shared_boundary", || {
            crate::execute_shared_boundary(
                response,
                action,
                policy_evaluation.decision,
                &resolved,
                trail,
                &mut executor,
                &input_context,
                true,
                &clock,
                replay_authority,
                None,
                &mut anchor_writer,
                None,
            )
        });
        match shared_result {
            Ok(result) => Ok(result),
            Err(error) => Err(ExecutionServiceResult::AuditFailed {
                evaluation_id,
                action_id,
                reason: format!("shared execution boundary failed: {error}"),
                execution_id: None,
            }),
        }
    }

    /// Map one shared-boundary result to a service result, consuming the
    /// response only for the Completed arm (the plan executor preserves the
    /// accumulated response itself and passes a placeholder for aggregates,
    /// where Completed is unreachable by construction).
    pub(crate) fn map_shared_result(
        result: crate::SharedExecutionResult,
        evaluation_id: String,
        action_id: String,
        response: Value,
    ) -> ExecutionServiceResult {
        let execution_id = result.execution_id;
        match result.outcome {
            crate::SharedExecutionOutcome::Completed => ExecutionServiceResult::Completed {
                evaluation_id,
                action_id,
                response,
                execution_id,
            },
            crate::SharedExecutionOutcome::Failed => ExecutionServiceResult::Failed {
                evaluation_id,
                action_id,
                reason: "provider returned a known failure".to_owned(),
                execution_id,
            },
            crate::SharedExecutionOutcome::Uncertain => ExecutionServiceResult::Uncertain {
                evaluation_id,
                action_id,
                reason: "provider outcome is uncertain".to_owned(),
                execution_id,
            },
            crate::SharedExecutionOutcome::Unattempted => ExecutionServiceResult::Unattempted {
                evaluation_id,
                action_id,
                reason: "deadline expired before provider invocation".to_owned(),
                execution_id,
            },
            crate::SharedExecutionOutcome::Denied => ExecutionServiceResult::Denied {
                evaluation_id,
                action_id,
                reason: "shared execution boundary denied dispatch".to_owned(),
                execution_id,
            },
            crate::SharedExecutionOutcome::AuditFailed => ExecutionServiceResult::AuditFailed {
                evaluation_id,
                action_id,
                reason: "durable execution audit failed".to_owned(),
                execution_id,
            },
            crate::SharedExecutionOutcome::Replay(
                crate::replay_runtime::ReplayDispatchResult::PersistenceUnavailable,
            ) => ExecutionServiceResult::ReplayPersistenceUnavailable {
                evaluation_id,
                action_id,
                execution_id,
            },
            crate::SharedExecutionOutcome::Replay(
                crate::replay_runtime::ReplayDispatchResult::BlockedCompletedSuccess,
            ) => ExecutionServiceResult::ReplayBlockedCompletedSuccess {
                evaluation_id,
                action_id,
                execution_id,
            },
            crate::SharedExecutionOutcome::Replay(
                crate::replay_runtime::ReplayDispatchResult::BlockedCompletedFailure,
            ) => ExecutionServiceResult::ReplayBlockedCompletedFailure {
                evaluation_id,
                action_id,
                execution_id,
            },
            crate::SharedExecutionOutcome::Replay(
                crate::replay_runtime::ReplayDispatchResult::RequiresManualResolution,
            ) => ExecutionServiceResult::ReplayRequiresManualResolution {
                evaluation_id,
                action_id,
                execution_id,
            },
        }
    }

    /// Resolve the exact planner-pinned capability after policy returned Allow.
    fn resolve_exact_capability(
        &self,
        proposed: &ProposedAction,
        provider_availability: &ProviderAvailability,
    ) -> Result<ResolvedCapability, ExecutionServiceResult> {
        let version =
            proposed
                .bridge_capability_version
                .ok_or_else(|| ExecutionServiceResult::Denied {
                    evaluation_id: proposed.evaluation_id.clone(),
                    action_id: proposed.action_id.clone(),
                    reason: "missing bridge capability version".to_owned(),
                    execution_id: None,
                })?;
        let provider = proposed
            .bridge_provider_identity
            .as_deref()
            .ok_or_else(|| ExecutionServiceResult::Denied {
                evaluation_id: proposed.evaluation_id.clone(),
                action_id: proposed.action_id.clone(),
                reason: "missing bridge provider identity".to_owned(),
                execution_id: None,
            })?;
        resolver::resolve_capability(
            self.runtime.trusted_store(),
            provider_availability,
            &proposed.capability_name,
            version,
            Some(provider),
        )
        .map_err(|error| ExecutionServiceResult::Unavailable {
            evaluation_id: proposed.evaluation_id.clone(),
            reason: format!("exact capability resolution failed: {error:?}"),
        })
    }
}

fn provider_service_error(
    operation: &str,
    provider_identity: &str,
    error: StdioProviderError,
) -> ExecutionServiceError {
    if matches!(error, StdioProviderError::Interrupted) {
        ExecutionServiceError::Interrupted
    } else {
        ExecutionServiceError::Provider(format!(
            "{operation} failed for {provider_identity}: {error}"
        ))
    }
}

fn assemble_request_envelope(
    input: &PreparedEvaluationInput,
    tether: &PreparedTether,
    capabilities: Vec<Value>,
) -> Value {
    serde_json::json!({
        "protocol_version": "0.1",
        "language_version": "0.1",
        "evaluation_id": input.evaluation_id,
        "tether": {
            "id": input.tether_id,
            "version": input.tether_version,
            "source": tether.source
        },
        "event": input.anchor_event,
        "facts": input.facts,
        "capabilities": capabilities,
    })
}

fn redacted_policy_reason(reason: &policy::PolicyReason) -> &'static str {
    match reason {
        policy::PolicyReason::ManifestRequiresConfirmation => "manifest_requires_confirmation",
        policy::PolicyReason::HostPolicyAsk => "host_policy_ask",
        _ => "approval_required",
    }
}

fn approval_required_result(
    evaluation_id: String,
    action_id: String,
    reason: &policy::PolicyReason,
) -> ExecutionServiceResult {
    ExecutionServiceResult::ApprovalRequired {
        evaluation_id,
        action_id,
        reason: redacted_policy_reason(reason).to_owned(),
    }
}

fn validate_prepared_discovery(
    prepared: &crate::configured_runtime::PreparedProvider,
    tools: &[Value],
) -> Result<(), String> {
    for capability in &prepared.capabilities {
        crate::stdio_provider::compare_discovery_evidence(tools, &capability.verified_manifest)
            .map_err(|error| {
                format!(
                    "tools/list mismatch for {} capability {} v{}: {error}",
                    prepared.identity, capability.name, capability.version
                )
            })?;
    }
    Ok(())
}

const MAX_CATALOGUE_DISCOVERY_ATTEMPTS: usize = 2;

/// Refresh only when the semantic Socket reports stale state. Discovery is
/// bounded and the host, not Socket, owns exact trusted-schema revalidation.
fn refresh_prepared_catalogue(
    prepared: &crate::configured_runtime::PreparedProvider,
    session: &mut RetainedProviderSession,
) -> Result<bool, StdioProviderError> {
    if !session.observe_catalogue_change()? && session.catalogue().is_some() {
        return Ok(true);
    }

    for _ in 0..MAX_CATALOGUE_DISCOVERY_ATTEMPTS {
        match session.discover() {
            Ok(catalogue) => {
                if validate_prepared_discovery(prepared, catalogue.operations()).is_ok() {
                    return Ok(true);
                }
                session.invalidate_catalogue();
                return Ok(false);
            }
            Err(StdioProviderError::CatalogueChangedDuringDiscovery) => continue,
            Err(error) => return Err(error),
        }
    }
    session.invalidate_catalogue();
    Err(StdioProviderError::CatalogueChangedDuringDiscovery)
}

// ===========================================================================
// Concurrent group execution — C2-A3a
// ===========================================================================

/// Tracks each Together group member through its lifecycle.
///
/// Every member from `member_indexes` gets exactly one state slot from the
/// start.  The state progresses: PreparationTerminal | Prepared → Launched →
/// Terminal.  No member may disappear or be silently skipped.
pub(crate) enum GroupMemberState {
    /// Preparation failed before the provider invocation boundary.
    /// This member has a final terminal classification and will not be
    /// launched.  Trail and response have been updated.
    PreparationTerminal {
        action_index: usize,
        action_id: String,
        semantic_position: dispatch::SemanticPosition,
        step: crate::plan_execution::ActionStep,
    },

    /// Serial preparation succeeded.  Ready for Stage B launch.
    Prepared {
        action_index: usize,
        action_id: String,
        semantic_position: dispatch::SemanticPosition,
        ready: dispatch::DispatchReadyAction,
        prepared: crate::application::PreparedInvoke,
        admission: Option<Box<dyn crate::replay_runtime::ReplayAdmissionGuard>>,
        deadline: Duration,
    },

    /// Worker thread has been launched.  Coordinator retains the
    /// DispatchReadyAction for Stage C processing.  `ready` and `prepared`
    /// are taken out via `Option::take` for Stage C processing.
    Launched {
        action_index: usize,
        action_id: String,
        semantic_position: dispatch::SemanticPosition,
        ready: Option<dispatch::DispatchReadyAction>,
        prepared: Option<crate::application::PreparedInvoke>,
        admission: Option<Box<dyn crate::replay_runtime::ReplayAdmissionGuard>>,
    },

    /// Owns no domain object while a Prepared member is being moved into the
    /// next state.  This is a Rust ownership transition, not a semantic state.
    Transitioning {
        action_index: usize,
        action_id: String,
        semantic_position: dispatch::SemanticPosition,
    },

    /// Member has reached its final terminal classification.
    Terminal {
        action_index: usize,
        action_id: String,
        semantic_position: dispatch::SemanticPosition,
        step: crate::plan_execution::ActionStep,
    },
}

/// Material sent to a worker thread for provider invocation.
///
/// Contains only what the worker needs: arguments, provider, tool name,
/// deadline.  The `DispatchReadyAction` stays coordinator-owned for Stage C.
pub(crate) struct WorkerInput {
    pub action_index: usize,
    pub arguments: Value,
    pub provider: crate::configured_runtime::PreparedProvider,
    pub tool_name: String,
    pub remaining: Duration,
}

/// Result returned from a worker thread through an mpsc channel.
pub(crate) struct WorkerResult {
    pub action_index: usize,
    pub provider_result: Result<Value, outcome::ProviderDiagnostic>,
}

// Test-only cross-thread panic injection seam.
//
// Stores the `action_index` of the worker that should panic.  `usize::MAX`
// means disabled.  Set before calling `execute_group_concurrent`; the value
// is visible to the spawned worker threads.
#[cfg(test)]
static INJECT_WORKER_PANIC_ACTION_INDEX: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(usize::MAX);

/// Perform the full Socket establishment / discovery / invocation path in a
/// worker thread.  Uses the same trusted contract as the serial path:
/// `RetainedProviderSession::establish` → `refresh_prepared_catalogue` →
/// `session.tools_call` → `session.close`.
///
/// This function does NOT touch Trail, replay, response, or any
/// coordinator-owned state.  It is a pure provider invocation carrier.
///
/// Worker panics are caught via `catch_unwind` to prevent the coordinator
/// from hanging when a worker thread terminates without sending its result.
pub(crate) fn worker_invoke_provider(input: WorkerInput, tx: mpsc::Sender<WorkerResult>) {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        #[cfg(test)]
        {
            let target = INJECT_WORKER_PANIC_ACTION_INDEX.load(std::sync::atomic::Ordering::SeqCst);
            if target == input.action_index {
                panic!("injected C2-A3a worker panic");
            }
        }
        worker_invoke_inner(&input)
    }));
    let provider_result = match result {
        Ok(r) => r,
        Err(_) => Err(outcome::ProviderDiagnostic::NoFinalResponse),
    };
    let _ = tx.send(WorkerResult {
        action_index: input.action_index,
        provider_result,
    });
}

fn worker_invoke_inner(input: &WorkerInput) -> Result<Value, outcome::ProviderDiagnostic> {
    let config = &input.provider.stdio_config;
    let expected_server_name = input
        .provider
        .capabilities
        .first()
        .map(|c| c.verified_manifest.manifest().binding.server_name.as_str())
        .unwrap_or("");

    // 1. Establish ephemeral session (launch + initialize).
    let mut session = match RetainedProviderSession::establish(SocketEstablishment {
        command: &config.command,
        args: &config.args,
        working_directory: &input.provider.working_directory,
        protocol_version: &config.protocol_version,
        server_name: expected_server_name,
        identity: &input.provider.identity,
    }) {
        Ok(session) => session,
        Err(error) => {
            return Err(classify_worker_session_error(&error));
        }
    };

    // 2. Discover / refresh catalogue.
    match refresh_prepared_catalogue(&input.provider, &mut session) {
        Ok(true) => {}
        Ok(false) => {
            session.close();
            return Err(outcome::ProviderDiagnostic::NoFinalResponse);
        }
        Err(error) => {
            session.close();
            return Err(classify_worker_session_error(&error));
        }
    }

    // 3. Invoke via the normal Socket contract.
    let result = session.tools_call(&input.tool_name, &input.arguments, input.remaining);

    // 4. Close the ephemeral session.
    session.close();

    result.map_err(|error| classify_provider_error(&error))
}

fn classify_worker_session_error(error: &StdioProviderError) -> outcome::ProviderDiagnostic {
    match error {
        StdioProviderError::ExplicitProviderError(_) => {
            outcome::ProviderDiagnostic::ExplicitProviderError
        }
        StdioProviderError::Interrupted => outcome::ProviderDiagnostic::ProtocolInterrupted,
        StdioProviderError::LaunchFailed { .. } => outcome::ProviderDiagnostic::NoFinalResponse,
        _ => outcome::ProviderDiagnostic::NoFinalResponse,
    }
}

fn count_active_members(member_states: &[GroupMemberState]) -> usize {
    member_states
        .iter()
        .filter(|s| {
            matches!(
                s,
                GroupMemberState::Launched { .. } | GroupMemberState::Transitioning { .. }
            )
        })
        .count()
}

fn has_prepared_members(member_states: &[GroupMemberState]) -> bool {
    member_states
        .iter()
        .any(|s| matches!(s, GroupMemberState::Prepared { .. }))
}

/// Execute one `together` group with real provider invocation overlap.
///
/// Preserves existing callers by defaulting to group width (A3a-compatible full overlap).
pub(crate) fn execute_group_concurrent(
    group_id: &str,
    member_indexes: &[usize],
    actions: &[Value],
    response: &mut Value,
    evaluation_id: &str,
    trail: &mut dyn dispatch::Trail,
    service: &HostExecutionService<'_>,
    input: &PreparedEvaluationInput,
    provider_sessions: &mut HashMap<String, RetainedProviderSession>,
    provider_availability: &ProviderAvailability,
    approvals: &mut crate::approval::ApprovalStore,
    replay_authority: &mut dyn crate::replay_runtime::ReplayAuthority,
) -> ExecutionServiceResult {
    let limit = service.runtime.max_active_together_invocations();
    execute_group_concurrent_with_limit(
        group_id,
        member_indexes,
        actions,
        response,
        evaluation_id,
        trail,
        service,
        input,
        provider_sessions,
        provider_availability,
        approvals,
        replay_authority,
        limit,
    )
}

/// Execute one `together` group with bounded provider invocation overlap.
///
/// This is the C3-A1 parameterised execution path.  It bounds active
/// provider invocations to `max_active_together_invocations` (N >= 1)
/// while maintaining strict semantic ordering and Stage C release boundaries.
///
/// ## Execution phases
///
/// **STAGE A — Serial deterministic preparation** (coordinator-owned):
/// For every member, in Runtime Plan order: scope, policy, resolution,
/// replay admission, G0 intent, Trail intent.  No deadline, no G1, no
/// provider effect.  Prep failures are recorded immediately as terminal
/// states with exact classification.
///
/// **STAGE B — Physical provider invocation** (bounded launch window):
/// While active_count < N, the earliest semantic-order Prepared member is
/// launched: deadline start (clock.now()), remaining calculation, G1 armed,
/// and scoped worker thread spawn.
///
/// **STAGE C — Durable result collection** (coordinator-owned):
/// When results arrive, coordinator executes complete Stage C processing
/// (`execute_boundary_invoke_only`) and transitions state to `Terminal`.
/// Only after full terminalisation does active_count decrease to free capacity.
///
/// **STAGE D — Join** (coordinator-owned):
/// After all members terminal: GroupJoinEntry, all-success test.
pub(crate) fn execute_group_concurrent_with_limit(
    group_id: &str,
    member_indexes: &[usize],
    actions: &[Value],
    response: &mut Value,
    evaluation_id: &str,
    trail: &mut dyn dispatch::Trail,
    service: &HostExecutionService<'_>,
    input: &PreparedEvaluationInput,
    provider_sessions: &mut HashMap<String, RetainedProviderSession>,
    provider_availability: &ProviderAvailability,
    approvals: &mut crate::approval::ApprovalStore,
    replay_authority: &mut dyn crate::replay_runtime::ReplayAuthority,
    max_active_together_invocations: usize,
) -> ExecutionServiceResult {
    let max_active = max_active_together_invocations.max(1);

    // ── STAGE A: Serial deterministic preparation ──────────────────────
    //
    // Every member gets a state slot immediately.  Prep failures are
    // recorded as PreparationTerminal with exact classification.
    let mut member_states: Vec<GroupMemberState> = Vec::with_capacity(member_indexes.len());

    for (member_ordinal, action_index) in member_indexes.iter().enumerate() {
        let action_id = crate::plan_execution::action_id_of(actions, *action_index);
        let position = dispatch::SemanticPosition {
            action_ordinal: *action_index as u64,
            group_id: Some(group_id.to_owned()),
            member_ordinal: Some(member_ordinal as u64),
            phase: dispatch::SemanticPhase::Member,
        };

        let proposed = match crate::extract_proposed_action_at(response, *action_index) {
            Ok(proposed) => proposed,
            Err(error) => {
                member_states.push(GroupMemberState::PreparationTerminal {
                    action_index: *action_index,
                    action_id: action_id.clone(),
                    semantic_position: position,
                    step: crate::plan_execution::ActionStep::Stopped(
                        ExecutionServiceResult::InvalidData {
                            message: format!("invalid planned Action: {error}"),
                        },
                    ),
                });
                continue;
            }
        };

        // Scope + policy + resolution (mirrors execute_one_action steps 1-5).
        let (scope_assessment, policy_evaluation) =
            crate::bench_timing::timed("scope_policy", || {
                let scope_assessment = service.runtime.assess_action_scope(&proposed);
                let policy_evaluation = policy::evaluate_effective_policy(
                    &proposed,
                    service.runtime.requirements(),
                    service.runtime.trusted_store(),
                    provider_availability,
                    service.runtime.policy(),
                    scope_assessment,
                );
                (scope_assessment, policy_evaluation)
            });

        // Policy must allow.  Deny / Ask / Unavailable are terminal failures.
        match &policy_evaluation.decision {
            PermissionDecision::Deny => {
                member_states.push(GroupMemberState::PreparationTerminal {
                    action_index: *action_index,
                    action_id: action_id.clone(),
                    semantic_position: position,
                    step: crate::plan_execution::ActionStep::Stopped(
                        ExecutionServiceResult::Denied {
                            evaluation_id: proposed.evaluation_id.clone(),
                            action_id,
                            reason: format!("{:?}", policy_evaluation.reason),
                            execution_id: None,
                        },
                    ),
                });
                continue;
            }
            PermissionDecision::Ask => {
                let result = match crate::request_exact_approval(
                    &proposed,
                    service.runtime.requirements(),
                    service.runtime.trusted_store(),
                    provider_availability,
                    service.runtime.policy(),
                    scope_assessment,
                    approvals,
                    trail,
                ) {
                    Ok(Some(_)) => approval_required_result(
                        proposed.evaluation_id.clone(),
                        action_id.clone(),
                        &policy_evaluation.reason,
                    ),
                    Ok(None) => ExecutionServiceResult::AuditFailed {
                        evaluation_id: proposed.evaluation_id.clone(),
                        action_id: action_id.clone(),
                        reason: "approval request could not be established".to_owned(),
                        execution_id: None,
                    },
                    Err(_) => ExecutionServiceResult::AuditFailed {
                        evaluation_id: proposed.evaluation_id.clone(),
                        action_id: action_id.clone(),
                        reason: "approval request Trail recording failed".to_owned(),
                        execution_id: None,
                    },
                };
                member_states.push(GroupMemberState::PreparationTerminal {
                    action_index: *action_index,
                    action_id,
                    semantic_position: position,
                    step: crate::plan_execution::ActionStep::Stopped(result),
                });
                continue;
            }
            PermissionDecision::Unavailable => {
                member_states.push(GroupMemberState::PreparationTerminal {
                    action_index: *action_index,
                    action_id,
                    semantic_position: position,
                    step: crate::plan_execution::ActionStep::Stopped(
                        ExecutionServiceResult::Unavailable {
                            evaluation_id: proposed.evaluation_id.clone(),
                            reason: format!("{:?}", policy_evaluation.reason),
                        },
                    ),
                });
                continue;
            }
            PermissionDecision::Allow(_) => {}
        }

        let resolved = match crate::bench_timing::timed("capability_resolve", || {
            service.resolve_exact_capability(&proposed, provider_availability)
        }) {
            Ok(resolved) => resolved,
            Err(result) => {
                member_states.push(GroupMemberState::PreparationTerminal {
                    action_index: *action_index,
                    action_id,
                    semantic_position: position,
                    step: crate::plan_execution::ActionStep::Stopped(result),
                });
                continue;
            }
        };

        let binding = &resolved.manifest().manifest().binding;
        if binding.kind != BindingKind::Mcp {
            member_states.push(GroupMemberState::PreparationTerminal {
                action_index: *action_index,
                action_id: action_id.clone(),
                semantic_position: position,
                step: crate::plan_execution::ActionStep::Stopped(ExecutionServiceResult::Denied {
                    evaluation_id: proposed.evaluation_id.clone(),
                    action_id,
                    reason: "capability binding is not MCP".to_owned(),
                    execution_id: None,
                }),
            });
            continue;
        }

        // Ensure provider session exists.
        if provider_sessions
            .get(resolved.provider_identity())
            .is_none()
        {
            member_states.push(GroupMemberState::PreparationTerminal {
                action_index: *action_index,
                action_id,
                semantic_position: position,
                step: crate::plan_execution::ActionStep::Stopped(
                    ExecutionServiceResult::Unavailable {
                        evaluation_id: proposed.evaluation_id.clone(),
                        reason: format!(
                            "provider '{}' has no retained session",
                            resolved.provider_identity()
                        ),
                    },
                ),
            });
            continue;
        }

        // Ensure prepared provider exists.
        let Some(_prepared_provider) = service
            .runtime
            .providers()
            .iter()
            .find(|provider| provider.identity == resolved.provider_identity())
        else {
            member_states.push(GroupMemberState::PreparationTerminal {
                action_index: *action_index,
                action_id,
                semantic_position: position,
                step: crate::plan_execution::ActionStep::Stopped(
                    ExecutionServiceResult::Unavailable {
                        evaluation_id: proposed.evaluation_id.clone(),
                        reason: format!(
                            "provider '{}' has no prepared catalogue authority",
                            resolved.provider_identity()
                        ),
                    },
                ),
            });
            continue;
        };

        let event_id = match input
            .anchor_event
            .get("id")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
        {
            Some(event_id) => event_id,
            None => {
                member_states.push(GroupMemberState::PreparationTerminal {
                    action_index: *action_index,
                    action_id,
                    semantic_position: position,
                    step: crate::plan_execution::ActionStep::Stopped(
                        ExecutionServiceResult::InvalidData {
                            message: "Anchor event requires a non-empty string id".to_owned(),
                        },
                    ),
                });
                continue;
            }
        };
        let input_context = crate::InputEventContext::for_initial(event_id);

        // Execute the prepare phase (replay admission, G0, Trail intent).
        // This produces a real DispatchReadyAction — no fabricated objects.
        match crate::application::execute_boundary_prepare(
            response,
            &actions[*action_index],
            policy_evaluation.decision.clone(),
            &resolved,
            trail,
            &input_context,
            true,
            replay_authority,
            Some(&position),
        ) {
            Ok((ready, prepared, admission)) => {
                member_states.push(GroupMemberState::Prepared {
                    action_index: *action_index,
                    action_id,
                    semantic_position: position,
                    ready,
                    prepared,
                    admission: Some(admission),
                    deadline: Duration::from_millis(resolved.manifest().manifest().timeout_ms),
                });
            }
            Err(result) => {
                // Prepare phase failed (replay, Trail intent, etc.).
                // The prepare function already updated response and Trail.
                // Record the terminal classification.
                member_states.push(GroupMemberState::PreparationTerminal {
                    action_index: *action_index,
                    action_id,
                    semantic_position: position,
                    step: crate::plan_execution::ActionStep::Boundary(result),
                });
            }
        }
    }

    // ── STAGE B & C: Bounded launch window and durable collection ───────
    let clock = ProductionMonotonicClock::new();
    let mut anchor_writer = crate::ResponseResultAnchorWriter;
    let (tx, rx) = mpsc::channel::<WorkerResult>();

    std::thread::scope(|s| {
        let mut launches_halted = false;

        loop {
            // Stage B: Launch eligible Prepared members in semantic order while capacity allows.
            while !launches_halted && count_active_members(&member_states) < max_active {
                let next_prepared_idx = member_states
                    .iter()
                    .position(|st| matches!(st, GroupMemberState::Prepared { .. }));
                let idx = match next_prepared_idx {
                    Some(i) => i,
                    None => break,
                };

                let transition = match &member_states[idx] {
                    GroupMemberState::Prepared {
                        action_index,
                        action_id,
                        semantic_position,
                        ..
                    } => GroupMemberState::Transitioning {
                        action_index: *action_index,
                        action_id: action_id.clone(),
                        semantic_position: semantic_position.clone(),
                    },
                    _ => unreachable!("guaranteed by position check"),
                };
                let prior = std::mem::replace(&mut member_states[idx], transition);

                let (action_index, action_id, position, ready, prepared, mut admission, deadline) =
                    match prior {
                        GroupMemberState::Prepared {
                            action_index,
                            action_id,
                            semantic_position,
                            ready,
                            prepared,
                            admission: Some(admission),
                            deadline,
                        } => (
                            action_index,
                            action_id,
                            semantic_position,
                            ready,
                            prepared,
                            admission,
                            deadline,
                        ),
                        _ => unreachable!("only Prepared members enter the launch transition"),
                    };

                let deadline_start = clock.now();
                let remaining = match crate::outcome::remaining_until_deadline(
                    &clock,
                    deadline_start,
                    deadline,
                ) {
                    Some(remaining) => remaining,
                    None => {
                        member_states[idx] = GroupMemberState::Terminal {
                            action_index,
                            action_id: action_id.clone(),
                            semantic_position: position,
                            step: crate::plan_execution::ActionStep::Stopped(
                                ExecutionServiceResult::Unattempted {
                                    evaluation_id: evaluation_id.to_owned(),
                                    action_id,
                                    reason: "deadline expired before provider invocation"
                                        .to_owned(),
                                    execution_id: Some(ready.execution_id().0.clone()),
                                },
                            ),
                        };
                        continue;
                    }
                };

                if admission.publish_armed().is_err() {
                    launches_halted = true;
                    member_states[idx] = GroupMemberState::Terminal {
                        action_index,
                        action_id: action_id.clone(),
                        semantic_position: position,
                        step: crate::plan_execution::ActionStep::Stopped(
                            ExecutionServiceResult::ReplayPersistenceUnavailable {
                                evaluation_id: evaluation_id.to_owned(),
                                action_id,
                                execution_id: Some(ready.execution_id().0.clone()),
                            },
                        ),
                    };
                    continue;
                }

                let provider_identity = ready.provider_identity().to_owned();
                let prepared_provider = match service
                    .runtime
                    .providers()
                    .iter()
                    .find(|p| p.identity == provider_identity)
                {
                    Some(provider) => provider.clone(),
                    None => {
                        member_states[idx] = GroupMemberState::Terminal {
                            action_index,
                            action_id,
                            semantic_position: position,
                            step: crate::plan_execution::ActionStep::Stopped(
                                ExecutionServiceResult::Unavailable {
                                    evaluation_id: evaluation_id.to_owned(),
                                    reason: format!(
                                        "provider '{provider_identity}' has no prepared catalogue authority"
                                    ),
                                },
                            ),
                        };
                        continue;
                    }
                };

                let tool_name = ready
                    .verified_manifest()
                    .manifest()
                    .binding
                    .tool_name
                    .clone();
                let arguments = ready.arguments().clone();

                member_states[idx] = GroupMemberState::Launched {
                    action_index,
                    action_id: action_id.clone(),
                    semantic_position: position.clone(),
                    ready: Some(ready),
                    prepared: Some(prepared),
                    admission: Some(admission),
                };

                let worker_input = WorkerInput {
                    action_index,
                    arguments,
                    provider: prepared_provider,
                    tool_name,
                    remaining,
                };

                let tx_clone = tx.clone();
                s.spawn(move || worker_invoke_provider(worker_input, tx_clone));
            }

            let active_count = count_active_members(&member_states);
            if active_count == 0 && (launches_halted || !has_prepared_members(&member_states)) {
                break;
            }

            if active_count == 0 {
                break;
            }

            match rx.recv() {
                Ok(worker_result) => {
                    let mut found: Option<(
                        usize,
                        String,
                        dispatch::SemanticPosition,
                        dispatch::DispatchReadyAction,
                        crate::application::PreparedInvoke,
                        Box<dyn crate::replay_runtime::ReplayAdmissionGuard>,
                    )> = None;

                    for state in member_states.iter_mut() {
                        if let GroupMemberState::Launched {
                            action_index,
                            action_id,
                            semantic_position,
                            ready,
                            prepared,
                            admission,
                            ..
                        } = state
                        {
                            if *action_index == worker_result.action_index {
                                let taken_ready =
                                    ready.take().expect("Launched state must have ready");
                                let taken_prepared =
                                    prepared.take().expect("Launched state must have prepared");
                                let taken_admission = admission
                                    .take()
                                    .expect("Launched state must have replay admission");
                                found = Some((
                                    *action_index,
                                    action_id.clone(),
                                    semantic_position.clone(),
                                    taken_ready,
                                    taken_prepared,
                                    taken_admission,
                                ));
                                break;
                            }
                        }
                    }

                    let (action_index, action_id, position, ready, prepared, mut admission) =
                        match found {
                            Some(v) => v,
                            None => continue,
                        };

                    let response_trail_len_before = response
                        .get("trail")
                        .and_then(Value::as_array)
                        .map_or(0, Vec::len);

                    let step = match crate::application::execute_boundary_invoke_only(
                        response,
                        &ready,
                        &prepared,
                        trail,
                        admission.as_mut(),
                        &mut anchor_writer,
                        worker_result.provider_result,
                        false,
                    ) {
                        Ok(mut result) => {
                            let current_boundary_audit_failed = response
                                .get("trail")
                                .and_then(Value::as_array)
                                .is_some_and(|entries| {
                                    entries
                                        .iter()
                                        .skip(response_trail_len_before)
                                        .any(|entry| entry["kind"] == "audit_failure")
                                });
                            if current_boundary_audit_failed {
                                result.outcome = crate::SharedExecutionOutcome::AuditFailed;
                            }
                            if matches!(
                                result.outcome,
                                crate::SharedExecutionOutcome::AuditFailed
                                    | crate::SharedExecutionOutcome::Replay(
                                        crate::replay_runtime::ReplayDispatchResult::PersistenceUnavailable
                                    )
                            ) {
                                launches_halted = true;
                            }
                            crate::plan_execution::ActionStep::Boundary(result)
                        }
                        Err(error) => {
                            launches_halted = true;
                            crate::plan_execution::ActionStep::Stopped(
                                ExecutionServiceResult::AuditFailed {
                                    evaluation_id: evaluation_id.to_owned(),
                                    action_id: action_id.clone(),
                                    reason: format!("shared execution boundary failed: {error}"),
                                    execution_id: Some(ready.execution_id().0.clone()),
                                },
                            )
                        }
                    };

                    for s in member_states.iter_mut() {
                        if let GroupMemberState::Launched {
                            action_index: idx, ..
                        } = s
                        {
                            if *idx == action_index {
                                *s = GroupMemberState::Terminal {
                                    action_index,
                                    action_id,
                                    semantic_position: position,
                                    step,
                                };
                                break;
                            }
                        }
                    }
                }
                Err(_) => {
                    break;
                }
            }
        }

        drop(tx);
    });

    // ── STAGE D: Join ──────────────────────────────────────────────────
    // GroupJoin exists ONLY after every semantic member has reached its
    // legitimate terminal state. If any member remains nonterminal (e.g.
    // Prepared due to a fatal launch halt, or Launched due to channel drop),
    // do NOT publish GroupJoin and fail closed.
    let any_nonterminal = member_states.iter().any(|s| {
        !matches!(
            s,
            GroupMemberState::Terminal { .. } | GroupMemberState::PreparationTerminal { .. }
        )
    });

    if any_nonterminal {
        let audit_action_id = member_states
            .iter()
            .find_map(|s| match s {
                GroupMemberState::Terminal { action_id, .. }
                | GroupMemberState::PreparationTerminal { action_id, .. }
                | GroupMemberState::Prepared { action_id, .. }
                | GroupMemberState::Launched { action_id, .. }
                | GroupMemberState::Transitioning { action_id, .. } => Some(action_id.clone()),
            })
            .unwrap_or_default();

        if let Some((action_id, step)) = first_non_success_member_step(member_states) {
            return crate::plan_execution::aggregate_step(step, evaluation_id, &action_id);
        }

        return ExecutionServiceResult::AuditFailed {
            evaluation_id: evaluation_id.to_owned(),
            action_id: audit_action_id,
            reason: "group execution halted with nonterminal members".to_owned(),
            execution_id: None,
        };
    }

    // Determine which members succeeded.  Every member must be Terminal.
    let all_joined = member_states.iter().all(|s| match s {
        GroupMemberState::Terminal { step, .. }
        | GroupMemberState::PreparationTerminal { step, .. } => {
            crate::plan_execution::step_succeeded(step)
        }
        _ => false,
    });

    let member_action_ids: Vec<String> = member_states
        .iter()
        .map(|s| match s {
            GroupMemberState::Terminal { action_id, .. }
            | GroupMemberState::PreparationTerminal { action_id, .. }
            | GroupMemberState::Prepared { action_id, .. }
            | GroupMemberState::Launched { action_id, .. }
            | GroupMemberState::Transitioning { action_id, .. } => action_id.clone(),
        })
        .collect();

    let last_member_index = *member_indexes.last().unwrap();
    let join_position = dispatch::SemanticPosition {
        action_ordinal: last_member_index as u64,
        group_id: None,
        member_ordinal: None,
        phase: dispatch::SemanticPhase::Join,
    };

    let join_entry = dispatch::GroupJoinEntry {
        evaluation_id: evaluation_id.to_owned(),
        group_id: group_id.to_owned(),
        member_action_ids: member_action_ids.clone(),
        joined: all_joined,
        timestamp_unix_ms: crate::now_unix_ms(),
        semantic_position: Some(join_position),
    };

    // Append join to Trail.
    let sequence = response
        .get("trail")
        .and_then(Value::as_array)
        .map(|trail| trail.len() as u64 + 1)
        .unwrap_or(1);
    let presentation_entry = serde_json::json!({
        "sequence": sequence,
        "phase": "execution",
        "kind": "group_joined",
        "outcome": if all_joined { "success" } else { "non_success" },
        "message": format!(
            "{} group {} ({} members: {})",
            if all_joined { "Joined" } else { "Group did not join" },
            group_id,
            member_indexes.len(),
            member_action_ids.join(", ")
        ),
        "host_timestamp_unix_ms": join_entry.timestamp_unix_ms,
    });

    let audit_action_id = member_action_ids.first().cloned().unwrap_or_default();
    match response.get_mut("trail").and_then(Value::as_array_mut) {
        Some(trail_array) => trail_array.push(presentation_entry),
        None => {
            return ExecutionServiceResult::AuditFailed {
                evaluation_id: evaluation_id.to_owned(),
                action_id: audit_action_id,
                reason: "response had no Trail".to_owned(),
                execution_id: None,
            };
        }
    }

    if trail.append_group_join(&join_entry).is_err() {
        return ExecutionServiceResult::AuditFailed {
            evaluation_id: evaluation_id.to_owned(),
            action_id: audit_action_id,
            reason: "group join Trail recording failed".to_owned(),
            execution_id: None,
        };
    }

    if all_joined {
        return ExecutionServiceResult::Completed {
            evaluation_id: evaluation_id.to_owned(),
            action_id: member_action_ids.first().cloned().unwrap_or_default(),
            response: response.clone(),
            execution_id: None,
        };
    }

    // Select the first semantic non-success, not the first completion.
    if let Some((action_id, step)) = first_non_success_member_step(member_states) {
        return crate::plan_execution::aggregate_step(step, evaluation_id, &action_id);
    }

    ExecutionServiceResult::AuditFailed {
        evaluation_id: evaluation_id.to_owned(),
        action_id: audit_action_id,
        reason: "non-success join produced no non-success member".to_owned(),
        execution_id: None,
    }
}

/// Return the first terminal non-success in the original Runtime Plan member
/// order, retaining its complete C1 ActionStep for exact aggregation.
fn first_non_success_member_step(
    member_states: Vec<GroupMemberState>,
) -> Option<(String, crate::plan_execution::ActionStep)> {
    member_states.into_iter().find_map(|state| {
        let (action_id, step) = match state {
            GroupMemberState::Terminal {
                action_id, step, ..
            }
            | GroupMemberState::PreparationTerminal {
                action_id, step, ..
            } => (action_id, step),
            _ => return None,
        };
        (!crate::plan_execution::step_succeeded(&step)).then_some((action_id, step))
    })
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::configured_runtime::{prepare_runtime, PreparedCapability, PreparedProvider};
    use crate::dispatch::{self, ActionId, ExecutionId, RecordingTrail};
    use crate::policy::{CapabilityRequirement, HostLocalPolicy, ScopeAssessment};
    use crate::replay::{LogicalExecutionKey, ReplayState};
    use crate::run_command;
    use crate::stdio_provider::ManagedProvider;
    use crate::trusted_store::TrustedManifestStore;
    use serde_json::json;

    #[test]
    fn c2_a3a_semantic_first_non_success_preserves_exact_step() {
        let position = |ordinal| dispatch::SemanticPosition {
            action_ordinal: ordinal,
            group_id: Some("group".to_owned()),
            member_ordinal: Some(ordinal),
            phase: dispatch::SemanticPhase::Member,
        };
        let states = vec![
            GroupMemberState::Terminal {
                action_index: 0,
                action_id: "first".to_owned(),
                semantic_position: position(0),
                step: crate::plan_execution::ActionStep::Boundary(crate::SharedExecutionResult {
                    outcome: crate::SharedExecutionOutcome::Uncertain,
                    execution_id: Some("exec-first".to_owned()),
                }),
            },
            GroupMemberState::Terminal {
                action_index: 1,
                action_id: "second".to_owned(),
                semantic_position: position(1),
                step: crate::plan_execution::ActionStep::Boundary(crate::SharedExecutionResult {
                    outcome: crate::SharedExecutionOutcome::Failed,
                    execution_id: Some("exec-second".to_owned()),
                }),
            },
        ];

        let (action_id, step) = first_non_success_member_step(states).expect("non-success");
        let result = crate::plan_execution::aggregate_step(step, "eval", &action_id);
        assert!(matches!(
            result,
            ExecutionServiceResult::Uncertain {
                evaluation_id,
                action_id,
                execution_id: Some(execution_id),
                ..
            } if evaluation_id == "eval" && action_id == "first" && execution_id == "exec-first"
        ));
    }
    use std::process::Command;
    use tethers_reference_host::cli::OutcomeStatus;

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn test_manifest_json() -> Value {
        serde_json::json!({
            "manifest_format_version": "1.0",
            "capability_name": "lantern.task.record",
            "capability_version": 1,
            "title": "Record a task",
            "description": "Record a task.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "project": { "type": "string" },
                    "task": { "type": "string" }
                },
                "required": ["project", "task"],
                "additionalProperties": false
            },
            "output_schema": {
                "type": "object",
                "properties": {
                    "status": { "type": "string" },
                    "project": { "type": "string" },
                    "task": { "type": "string" }
                },
                "required": ["status"],
                "additionalProperties": false
            },
            "effects": ["lantern.write"],
            "permission_scope": {
                "kind": "path_prefix",
                "allowed_prefixes": ["projects/"]
            },
            "reversibility": "compensatable",
            "determinism": "deterministic",
            "idempotency": {
                "mechanism": "argument_key",
                "argument_name": "idempotency_key",
                "key_source": "evaluation_id/action_id"
            },
            "confirmation_policy": {
                "standing_permitted": true,
                "per_call_required": false
            },
            "timeout_ms": 10000,
            "retry_policy": {
                "max_retries": 0,
                "backoff_ms": 500,
                "allowed_on": ["outcome_unknown"],
                "requires_idempotency_proof": false
            },
            "provider": {
                "identity": "lantern-local",
                "display_name": "Lantern Keeper (local mock)",
                "identity_source": "host_configuration",
                "description": "Mock executor for tests."
            },
            "binding": {
                "kind": "mcp",
                "server_name": "lantern",
                "tool_name": "task_record",
                "adapter": null
            }
        })
    }

    fn catalogue_test_provider(mode: &str) -> PreparedProvider {
        let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("scripts")
            .join("tethers-stdio-fixture.ps1");
        let verified_manifest = crate::manifest::verify_manifest(include_str!(
            "../../protocol/capability-manifests/fixture-ping.json"
        ))
        .unwrap();
        PreparedProvider {
            identity: "tethers-stdio-fixture".to_owned(),
            display_name: "Tethers Stdio Fixture".to_owned(),
            working_directory: script.parent().unwrap().to_path_buf(),
            stdio_config: crate::stdio_provider::StdioProviderConfig {
                command: "pwsh.exe".to_owned(),
                args: vec![
                    "-NoProfile".to_owned(),
                    "-ExecutionPolicy".to_owned(),
                    "Bypass".to_owned(),
                    "-File".to_owned(),
                    script.to_string_lossy().into_owned(),
                    "-Mode".to_owned(),
                    mode.to_owned(),
                ],
                protocol_version: "2025-11-25".to_owned(),
                provider_config: crate::provider::ProviderConfig {
                    identity: "tethers-stdio-fixture".to_owned(),
                    display_name: "Tethers Stdio Fixture".to_owned(),
                    allowed_capabilities: Vec::new(),
                },
            },
            capabilities: vec![PreparedCapability {
                name: "fixture.ping".to_owned(),
                version: 1,
                manifest_path: PathBuf::from("fixture-ping.json"),
                verified_manifest,
                scope_binding: None,
            }],
        }
    }

    fn establish_catalogue_test_session(prepared: &PreparedProvider) -> RetainedProviderSession {
        let manifest = prepared.capabilities[0].verified_manifest.manifest();
        RetainedProviderSession::establish(SocketEstablishment {
            command: &prepared.stdio_config.command,
            args: &prepared.stdio_config.args,
            working_directory: &prepared.working_directory,
            protocol_version: &prepared.stdio_config.protocol_version,
            server_name: &manifest.binding.server_name,
            identity: &prepared.identity,
        })
        .unwrap()
    }

    fn barrier_test_provider(barrier_dir: &Path, identity: &str) -> PreparedProvider {
        let mut provider = catalogue_test_provider("c2-overlap-barrier");
        provider.identity = identity.to_owned();
        provider.stdio_config.provider_config.identity = identity.to_owned();
        provider.stdio_config.args.extend([
            "-BarrierDirectory".to_owned(),
            barrier_dir.to_string_lossy().into_owned(),
        ]);
        provider
    }

    fn prove_actual_worker_overlap(same_provider: bool) {
        let barrier_dir = std::env::temp_dir().join(format!(
            "tethers-c2-a3a-overlap-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&barrier_dir).unwrap();
        let left = barrier_test_provider(&barrier_dir, "tethers-stdio-fixture");
        let right = barrier_test_provider(
            &barrier_dir,
            if same_provider {
                "tethers-stdio-fixture"
            } else {
                "tethers-stdio-fixture-2"
            },
        );
        let (tx, rx) = mpsc::channel();
        std::thread::scope(|scope| {
            for (action_index, provider) in [(0, left), (1, right)] {
                let sender = tx.clone();
                scope.spawn(move || {
                    worker_invoke_provider(
                        WorkerInput {
                            action_index,
                            arguments: json!({"message": format!("member-{action_index}")}),
                            provider,
                            tool_name: "fixture_ping".to_owned(),
                            remaining: Duration::from_secs(15),
                        },
                        sender,
                    )
                });
            }
            drop(tx);
            let deadline = std::time::Instant::now() + Duration::from_secs(12);
            loop {
                let active = std::fs::read_dir(&barrier_dir)
                    .unwrap()
                    .filter_map(Result::ok)
                    .filter(|entry| entry.file_name().to_string_lossy().starts_with("active-"))
                    .count();
                if active >= 2 {
                    break;
                }
                assert!(
                    std::time::Instant::now() < deadline,
                    "both provider child processes did not reach tools/call"
                );
                std::thread::sleep(Duration::from_millis(10));
            }
            std::fs::write(barrier_dir.join("release"), "release").unwrap();
            for _ in 0..2 {
                assert!(rx
                    .recv_timeout(Duration::from_secs(10))
                    .unwrap()
                    .provider_result
                    .is_ok());
            }
        });
        std::fs::remove_dir_all(&barrier_dir).unwrap();
    }

    #[test]
    fn c2_a3a_same_provider_tools_call_overlap_is_real() {
        prove_actual_worker_overlap(true);
    }

    #[test]
    fn c2_a3a_different_provider_tools_call_overlap_is_real() {
        prove_actual_worker_overlap(false);
    }

    #[test]
    fn m1_host_bounded_rediscovery_retains_exact_unchanged_binding() {
        let prepared = catalogue_test_provider("catalogue-change-unchanged");
        let mut session = establish_catalogue_test_session(&prepared);
        assert!(refresh_prepared_catalogue(&prepared, &mut session).unwrap());
        assert!(!session.catalogue_is_stale());
        assert_eq!(session.catalogue().unwrap().operations().len(), 1);
        session.close();
    }

    #[test]
    fn m1_host_schema_drift_invalidates_binding_before_invocation() {
        let prepared = catalogue_test_provider("catalogue-change-drift");
        let mut session = establish_catalogue_test_session(&prepared);
        assert!(!refresh_prepared_catalogue(&prepared, &mut session).unwrap());
        assert!(session.catalogue_is_stale());
        assert!(session.catalogue().is_none());
        session.close();
    }

    // -----------------------------------------------------------------------
    // j13b_ tests
    // -----------------------------------------------------------------------

    fn resolved_test_capability() -> (TrustedManifestStore, ResolvedCapability) {
        let mut manifest = test_manifest_json();
        let (_, digest) = crate::manifest::canonicalize_and_digest(&manifest.to_string()).unwrap();
        manifest["digest"] = serde_json::json!(digest);
        let verified = crate::manifest::verify_manifest(&manifest.to_string()).unwrap();
        let mut store = TrustedManifestStore::new();
        store.insert(verified).unwrap();
        let availability = ProviderAvailability::from_identities(["lantern-local"]);
        let resolved = resolver::resolve_capability(
            &store,
            &availability,
            "lantern.task.record",
            1,
            Some("lantern-local"),
        )
        .unwrap();
        (store, resolved)
    }

    fn planner_input() -> PreparedEvaluationInput {
        PreparedEvaluationInput {
            tether_id: "fixture.selected".to_owned(),
            tether_version: "1.2.3".to_owned(),
            evaluation_id: "eval-correlation-1".to_owned(),
            anchor_event: serde_json::json!({
                "id": "event-correlation-1",
                "name": "fixture.start"
            }),
            facts: serde_json::json!({}),
        }
    }

    fn correlated_planner_response(status: &str) -> Value {
        serde_json::json!({
            "protocol_version": "0.1",
            "evaluation_id": "eval-correlation-1",
            "event_id": "event-correlation-1",
            "tether_id": "fixture.selected",
            "tether_version": "1.2.3",
            "status": status,
            "plan": null,
            "trail": []
        })
    }

    fn wire_from_response(status: &str, response: Value) -> PlannerResponseWire {
        match status {
            "matched" => PlannerResponseWire::Matched(response),
            "not_matched" => PlannerResponseWire::NotMatched(response),
            "error" => PlannerResponseWire::Error(response),
            other => PlannerResponseWire::Unknown {
                status: other.to_owned(),
                response,
            },
        }
    }

    #[test]
    fn j13b_matched_response_validates_every_correlation_before_dispatch() {
        let input = planner_input();
        let wire = PlannerResponseWire::Matched(correlated_planner_response("matched"));
        let outcome = HostExecutionService::classify_planner_response(&input, wire);
        let mut dispatch_calls = 0;
        let result = HostExecutionService::route_planner_outcome(outcome, |response| {
            dispatch_calls += 1;
            assert_eq!(response["status"], "matched");
            ExecutionServiceResult::InvalidData {
                message: "focused dispatch marker".to_owned(),
            }
        });
        assert_eq!(dispatch_calls, 1);
        assert!(matches!(result, ExecutionServiceResult::InvalidData { .. }));
    }

    #[test]
    fn j13b_not_matched_is_no_actions_without_dispatch() {
        let input = planner_input();
        let wire = PlannerResponseWire::NotMatched(correlated_planner_response("not_matched"));
        let outcome = HostExecutionService::classify_planner_response(&input, wire);
        let mut dispatch_calls = 0;
        let result = HostExecutionService::route_planner_outcome(outcome, |_| {
            dispatch_calls += 1;
            ExecutionServiceResult::Interrupted
        });
        assert_eq!(dispatch_calls, 0);
        assert!(matches!(
            result,
            ExecutionServiceResult::NoActions { evaluation_id, .. }
                if evaluation_id == "eval-correlation-1"
        ));
    }

    #[test]
    fn j13b_correlated_and_minimal_planner_errors_are_distinct() {
        let input = planner_input();
        let mut correlated = correlated_planner_response("error");
        correlated["error"] =
            serde_json::json!({"code": "type_error", "message": "invalid condition"});
        let minimal = serde_json::json!({
            "protocol_version": "0.1",
            "status": "error",
            "error": {"code": "parse_error", "message": "invalid Tether source"}
        });

        let correlated_result = HostExecutionService::route_planner_outcome(
            HostExecutionService::classify_planner_response(
                &input,
                PlannerResponseWire::Error(correlated),
            ),
            |_| ExecutionServiceResult::Interrupted,
        );
        assert!(matches!(
            correlated_result,
            ExecutionServiceResult::PlannerError {
                evaluation_id: Some(evaluation_id),
                code,
                ..
            } if evaluation_id == "eval-correlation-1" && code == "type_error"
        ));

        let minimal_result = HostExecutionService::route_planner_outcome(
            HostExecutionService::classify_planner_response(
                &input,
                PlannerResponseWire::Error(minimal),
            ),
            |_| ExecutionServiceResult::Interrupted,
        );
        assert!(matches!(
            minimal_result,
            ExecutionServiceResult::PlannerError {
                evaluation_id: None,
                code,
                ..
            } if code == "parse_error"
        ));
    }

    #[test]
    fn j13b_engine_validation_and_evaluation_failures_remain_typed() {
        let validation = ExecutionServiceError::from(EngineError::ValidationFailed {
            tether_index: 0,
            tether_id: "fixture.selected".to_owned(),
            tether_version: "1.2.3".to_owned(),
            error_code: "parse_error".to_owned(),
            message: "untrusted engine detail".to_owned(),
        });
        assert!(matches!(
            validation,
            ExecutionServiceError::TetherValidation(_)
        ));

        let input = planner_input();
        assert!(matches!(
            HostExecutionService::classify_engine_evaluation_failure(
                &input,
                EngineError::ProtocolError("malformed framing".to_owned()),
            ),
            ExecutionServiceResult::Unavailable { evaluation_id, .. }
                if evaluation_id == "eval-correlation-1"
        ));
        assert!(matches!(
            HostExecutionService::classify_engine_evaluation_failure(
                &input,
                EngineError::Interrupted
            ),
            ExecutionServiceResult::Interrupted
        ));
    }

    #[test]
    fn j13b_every_planner_correlation_mismatch_is_invalid_data() {
        let input = planner_input();
        for status in ["matched", "not_matched", "error"] {
            for field in [
                "protocol_version",
                "evaluation_id",
                "event_id",
                "tether_id",
                "tether_version",
            ] {
                let mut wrong = correlated_planner_response(status);
                wrong["error"] =
                    serde_json::json!({"code": "type_error", "message": "invalid condition"});
                wrong[field] = Value::String("wrong".to_owned());
                let wrong_result = HostExecutionService::route_planner_outcome(
                    HostExecutionService::classify_planner_response(
                        &input,
                        wire_from_response(status, wrong),
                    ),
                    |_| ExecutionServiceResult::Interrupted,
                );
                assert!(
                    matches!(wrong_result, ExecutionServiceResult::InvalidData { .. }),
                    "{status}: wrong {field}"
                );

                let mut missing = correlated_planner_response(status);
                missing["error"] =
                    serde_json::json!({"code": "type_error", "message": "invalid condition"});
                missing.as_object_mut().unwrap().remove(field);
                let missing_result = HostExecutionService::route_planner_outcome(
                    HostExecutionService::classify_planner_response(
                        &input,
                        wire_from_response(status, missing),
                    ),
                    |_| ExecutionServiceResult::Interrupted,
                );
                assert!(
                    matches!(missing_result, ExecutionServiceResult::InvalidData { .. }),
                    "{status}: missing {field}"
                );
            }
        }
    }

    #[test]
    fn j13b_unknown_planner_status_is_invalid_data() {
        let input = planner_input();
        let unknown = PlannerResponseWire::Unknown {
            status: "completed".to_owned(),
            response: correlated_planner_response("completed"),
        };
        let result = HostExecutionService::route_planner_outcome(
            HostExecutionService::classify_planner_response(&input, unknown),
            |_| ExecutionServiceResult::Interrupted,
        );
        assert!(matches!(result, ExecutionServiceResult::InvalidData { .. }));
    }

    #[test]
    fn j13b_ask_result_contains_no_invented_approval_id() {
        let result = approval_required_result(
            "eval-ask-1".to_owned(),
            "action-ask-1".to_owned(),
            &policy::PolicyReason::HostPolicyAsk,
        );
        assert!(matches!(
            result,
            ExecutionServiceResult::ApprovalRequired {
                evaluation_id,
                action_id,
                reason,
            } if evaluation_id == "eval-ask-1"
                && action_id == "action-ask-1"
                && reason == "host_policy_ask"
        ));
    }

    #[test]
    fn j13b_rejected_error_and_invalid_routes_make_zero_replay_or_provider_calls() {
        let input = planner_input();
        let mut correlated_error = correlated_planner_response("error");
        correlated_error["error"] =
            serde_json::json!({"code": "type_error", "message": "invalid condition"});
        let minimal_error = serde_json::json!({
            "protocol_version": "0.1",
            "status": "error",
            "error": {"code": "parse_error", "message": "invalid source"}
        });
        let mut mismatch = correlated_planner_response("matched");
        mismatch["event_id"] = Value::String("wrong-event".to_owned());
        let unknown = PlannerResponseWire::Unknown {
            status: "unknown".to_owned(),
            response: correlated_planner_response("unknown"),
        };

        for wire in [
            PlannerResponseWire::NotMatched(correlated_planner_response("not_matched")),
            PlannerResponseWire::Error(correlated_error),
            PlannerResponseWire::Error(minimal_error),
            PlannerResponseWire::Matched(mismatch),
            unknown,
        ] {
            let outcome = HostExecutionService::classify_planner_response(&input, wire);
            let mut dispatch_calls = 0;
            let _ = HostExecutionService::route_planner_outcome(outcome, |_| {
                dispatch_calls += 1;
                ExecutionServiceResult::Interrupted
            });
            assert_eq!(
                dispatch_calls, 0,
                "planner terminal route must stop before replay/provider dispatch"
            );
        }
    }

    #[test]
    fn j13b_extra_planner_response_fields_are_tolerated() {
        let input = planner_input();
        let mut response = correlated_planner_response("matched");
        response["extra_field"] = serde_json::json!("unrelated");
        let outcome = HostExecutionService::classify_planner_response(
            &input,
            PlannerResponseWire::Matched(response),
        );
        assert!(matches!(outcome, Ok(PlannerOutcome::Matched(_))));
    }

    /// Structured scope without binding-owned WithinScope evidence cannot
    /// produce a dispatch-ready token.
    #[test]
    fn j13b_structured_scope_requires_trusted_within_scope_before_dispatch() {
        let mut trail = RecordingTrail::new();
        let requirements = vec![CapabilityRequirement::new("lantern.task.record", 1)];
        let (store, resolved) = resolved_test_capability();
        let proposed = ProposedAction {
            evaluation_id: "eval-1".to_owned(),
            plan_id: "plan-1".to_owned(),
            action_id: "act-1".to_owned(),
            capability_name: "lantern.task.record".to_owned(),
            manifest_digest: Some(resolved.manifest_digest().to_owned()),
            bridge_capability_version: Some(1),
            bridge_provider_identity: Some("lantern-local".to_owned()),
            arguments: serde_json::json!({"project": "projects/a", "task": "t"}),
        };
        let availability = ProviderAvailability::from_identities(["lantern-local"]);
        let policy = HostLocalPolicy::new(policy::PolicyRule::Allow);
        let eval = policy::evaluate_effective_policy(
            &proposed,
            &requirements,
            &store,
            &availability,
            &policy,
            ScopeAssessment::ScopeNotEstablished,
        );
        assert!(matches!(eval.decision, PermissionDecision::Deny));
        let ready = dispatch::prepare_and_record(
            eval.decision,
            &resolved,
            ExecutionId("exec-1".to_owned()),
            ActionId("act-1".to_owned()),
            proposed.arguments,
            &mut trail,
            None,
        );
        assert_eq!(ready.unwrap_err(), dispatch::PrepareError::Deny);
        assert!(trail.entries.is_empty());
    }

    /// The request passed to the engine has the accepted direct 0.1 shape:
    /// one selected Tether including source and one direct capabilities array.
    #[test]
    fn j13b_request_envelope_is_exact_direct_and_contains_selected_source() {
        let input = PreparedEvaluationInput {
            tether_id: "fixture.selected".to_owned(),
            tether_version: "1.2.3".to_owned(),
            evaluation_id: "eval-explicit-7".to_owned(),
            anchor_event: serde_json::json!({"id": "event-7", "name": "fixture.start"}),
            facts: serde_json::json!({"immutable": true}),
        };
        let tether = PreparedTether {
            id: input.tether_id.clone(),
            version: input.tether_version.clone(),
            source_path: PathBuf::from("selected.tether"),
            source: "tether fixture.selected version 1.2.3\non fixture.start\ndo fixture.ping()"
                .to_owned(),
            core_environment: None,
        };
        let capabilities = vec![serde_json::json!({
            "name": "fixture.ping",
            "version": "1.0.0",
            "inputs": {"type": "object"},
            "effects": ["fixture.read"],
            "reversibility": "reversible"
        })];
        let request = assemble_request_envelope(&input, &tether, capabilities.clone());

        assert_eq!(
            request,
            serde_json::json!({
                "protocol_version": "0.1",
                "language_version": "0.1",
                "evaluation_id": "eval-explicit-7",
                "tether": {
                    "id": "fixture.selected",
                    "version": "1.2.3",
                    "source": tether.source
                },
                "event": input.anchor_event,
                "facts": input.facts,
                "capabilities": capabilities
            })
        );
        assert!(request["capabilities"].is_array());
        assert!(request.pointer("/capabilities/capabilities").is_none());
        assert_eq!(request["capabilities"].as_array().unwrap().len(), 1);
    }

    /// Prove Unavailable produces zero provider calls.
    #[test]
    fn j13b_unavailable_produces_zero_provider_calls() {
        let policy = HostLocalPolicy::new(policy::PolicyRule::Allow);
        let requirements = vec![CapabilityRequirement::new("test.cap", 1)];
        // Bridge pins are required by evaluate_effective_policy.
        let proposed = ProposedAction {
            evaluation_id: "eval-1".to_owned(),
            plan_id: "plan-1".to_owned(),
            action_id: "act-1".to_owned(),
            capability_name: "test.cap".to_owned(),
            manifest_digest: Some(
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                    .to_owned(),
            ),
            bridge_capability_version: Some(1),
            bridge_provider_identity: Some("test-provider".to_owned()),
            arguments: Value::Null,
        };
        let store = TrustedManifestStore::new();
        let availability = ProviderAvailability::empty();
        let eval = policy::evaluate_effective_policy(
            &proposed,
            &requirements,
            &store,
            &availability,
            &policy,
            ScopeAssessment::ScopeNotEstablished,
        );
        // Empty store means capability is not admitted -> Unavailable.
        assert!(matches!(eval.decision, PermissionDecision::Unavailable));
    }

    /// Shared replay classifications remain distinct at the service boundary.
    #[test]
    fn j13b_shared_replay_results_map_without_becoming_completed() {
        let cases = [
            (
                crate::replay_runtime::ReplayDispatchResult::PersistenceUnavailable,
                "ReplayPersistenceUnavailable",
            ),
            (
                crate::replay_runtime::ReplayDispatchResult::BlockedCompletedSuccess,
                "ReplayBlockedCompletedSuccess",
            ),
            (
                crate::replay_runtime::ReplayDispatchResult::BlockedCompletedFailure,
                "ReplayBlockedCompletedFailure",
            ),
            (
                crate::replay_runtime::ReplayDispatchResult::RequiresManualResolution,
                "ReplayRequiresManualResolution",
            ),
        ];
        for (replay, expected) in cases {
            let result = HostExecutionService::map_shared_result(
                crate::SharedExecutionResult {
                    outcome: crate::SharedExecutionOutcome::Replay(replay),
                    execution_id: None,
                },
                "eval-1".to_owned(),
                "action-1".to_owned(),
                serde_json::json!({}),
            );
            let actual = format!("{result:?}");
            assert!(actual.starts_with(expected), "{replay:?}: {actual}");
            assert!(!actual.starts_with("Completed"), "{replay:?}: {actual}");
        }
    }

    /// Prove PreparedEvaluationInput fields are explicit.
    #[test]
    fn j13b_evaluation_input_all_fields_explicit() {
        let input = PreparedEvaluationInput {
            tether_id: "my.tether".into(),
            tether_version: "1.0.0".into(),
            evaluation_id: "eval-explicit-001".into(),
            anchor_event: serde_json::json!({"id": "evt-1", "name": "test"}),
            facts: serde_json::json!({"key": "value"}),
        };

        assert_eq!(input.tether_id, "my.tether");
        assert_eq!(input.tether_version, "1.0.0");
        assert_eq!(input.evaluation_id, "eval-explicit-001");
        assert_eq!(input.anchor_event["id"], "evt-1");
        assert_eq!(input.facts["key"], "value");
    }

    /// Prove that missing bridge pins produce Deny (security boundary).
    #[test]
    fn j13b_missing_bridge_pins_deny() {
        let policy = HostLocalPolicy::new(policy::PolicyRule::Allow);
        let requirements = vec![CapabilityRequirement::new("test.cap", 1)];
        let proposed = ProposedAction {
            evaluation_id: "eval-1".into(),
            plan_id: "plan-1".into(),
            action_id: "act-1".into(),
            capability_name: "test.cap".into(),
            manifest_digest: None,
            bridge_capability_version: None,
            bridge_provider_identity: None,
            arguments: Value::Null,
        };
        let store = TrustedManifestStore::new();
        let availability = ProviderAvailability::empty();
        let eval = policy::evaluate_effective_policy(
            &proposed,
            &requirements,
            &store,
            &availability,
            &policy,
            ScopeAssessment::ScopeNotEstablished,
        );

        // Missing bridge pins should produce Deny even with Allow policy.
        // This is a security boundary: no dispatch without validated bridge pins.
        assert!(matches!(eval.decision, PermissionDecision::Deny));
    }

    /// Only a well-formed JSON-RPC error is trusted as provider-declared
    /// failure. Transport and framing failures remain uncertain.
    #[test]
    fn j13b_provider_error_classification_preserves_failure_vs_uncertainty() {
        assert_eq!(
            classify_provider_error(&StdioProviderError::ExplicitProviderError(
                "declared".to_owned()
            )),
            outcome::ProviderDiagnostic::ExplicitProviderError
        );
        for error in [
            StdioProviderError::EmptyResponse,
            StdioProviderError::MalformedResponse("bad JSON".to_owned()),
            StdioProviderError::ProtocolError("wrong response id".to_owned()),
            StdioProviderError::ReadFailed("EOF or timeout".to_owned()),
        ] {
            assert_eq!(
                classify_provider_error(&error),
                outcome::ProviderDiagnostic::NoFinalResponse,
                "{error:?}"
            );
        }
    }

    /// Replay identity includes the Anchor event ID as well as the explicit
    /// planner evaluation and Action IDs.
    #[test]
    fn j13b_anchor_event_id_participates_in_replay_logical_key() {
        let key1 = LogicalExecutionKey::derive("evt-1", "eval-1", "act-1").unwrap();
        let key2 = LogicalExecutionKey::derive("evt-1", "eval-1", "act-1").unwrap();
        assert_eq!(key1, key2);

        let key3 = LogicalExecutionKey::derive("evt-2", "eval-1", "act-1").unwrap();
        assert_ne!(key1, key3);
    }

    /// Pre-invocation deadline expiry has its own typed service result.
    #[test]
    fn j13b_unattempted_cannot_be_confused_with_failed() {
        let result = HostExecutionService::map_shared_result(
            crate::SharedExecutionResult {
                outcome: crate::SharedExecutionOutcome::Unattempted,
                execution_id: None,
            },
            "eval-1".to_owned(),
            "action-1".to_owned(),
            serde_json::json!({}),
        );
        assert!(matches!(result, ExecutionServiceResult::Unattempted { .. }));
    }

    #[test]
    fn j13b_one_retained_provider_serves_multiple_calls_with_monotonic_ids() {
        let marker = std::env::temp_dir().join(format!(
            "tethers-j13b-provider-{}.txt",
            uuid::Uuid::new_v4().simple()
        ));
        let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("scripts")
            .join("tethers-stdio-fixture.ps1");
        let working_dir = script.parent().unwrap().to_path_buf();
        let args = vec![
            "-NoProfile".to_owned(),
            "-ExecutionPolicy".to_owned(),
            "Bypass".to_owned(),
            "-File".to_owned(),
            script.to_string_lossy().into_owned(),
            "-Mode".to_owned(),
            "record-methods".to_owned(),
            "-MarkerFile".to_owned(),
            marker.to_string_lossy().into_owned(),
        ];
        let mut provider =
            ManagedProvider::launch("pwsh", &args, &working_dir, None, None).unwrap();
        provider
            .initialize("2025-11-25", "tethers-stdio-fixture")
            .unwrap();
        provider.list_tools().unwrap();
        let mut session =
            RetainedProviderSession::from_discovered(provider, "tethers-stdio-fixture".to_owned());
        assert!(matches!(
            session
                .tools_call(
                    "fixture_ping",
                    &serde_json::json!({"message": "first"}),
                    Duration::from_secs(1)
                )
                .unwrap_err(),
            StdioProviderError::ExplicitProviderError(_)
        ));
        assert!(matches!(
            session
                .tools_call(
                    "fixture_ping",
                    &serde_json::json!({"message": "second"}),
                    Duration::from_secs(1)
                )
                .unwrap_err(),
            StdioProviderError::ExplicitProviderError(_)
        ));
        assert_eq!(session.next_request_id(), 5);
        session.close();
        let methods = fs::read_to_string(&marker).unwrap();
        assert!(methods.contains("initialize"));
        assert!(methods.contains("tools/list"));
        assert_eq!(
            methods.lines().filter(|line| *line == "tools/call").count(),
            2
        );
        fs::remove_file(marker).unwrap();
    }

    /// The service exposes prepared-input execution only: CLI parsing remains
    /// outside it and evaluation IDs stay host-supplied.
    #[test]
    fn j13b_service_keeps_evaluation_ids_explicit() {
        let input = PreparedEvaluationInput {
            tether_id: "t".into(),
            tether_version: "1.0.0".into(),
            evaluation_id: "explicit-only-001".into(),
            anchor_event: serde_json::json!({}),
            facts: serde_json::json!({}),
        };
        assert_eq!(input.evaluation_id, "explicit-only-001");
    }

    #[test]
    fn j13b_run_selected_tether_indexes_are_exact_and_deduplicated() {
        let tethers = vec![
            PreparedTether {
                id: "selected".to_owned(),
                version: "1".to_owned(),
                source_path: PathBuf::from("selected.tether"),
                source: "selected".to_owned(),
                core_environment: None,
            },
            PreparedTether {
                id: "unselected".to_owned(),
                version: "1".to_owned(),
                source_path: PathBuf::from("unselected.tether"),
                source: "unselected".to_owned(),
                core_environment: None,
            },
        ];
        let selected = PreparedEvaluationInput {
            tether_id: "selected".to_owned(),
            tether_version: "1".to_owned(),
            evaluation_id: "eval".to_owned(),
            anchor_event: serde_json::json!({}),
            facts: serde_json::json!({}),
        };
        assert_eq!(
            selected_tether_indexes(&tethers, &[selected.clone(), selected]).unwrap(),
            vec![0]
        );
        let missing = PreparedEvaluationInput {
            tether_id: "missing".to_owned(),
            tether_version: "1".to_owned(),
            evaluation_id: "eval".to_owned(),
            anchor_event: serde_json::json!({}),
            facts: serde_json::json!({}),
        };
        assert!(matches!(
            selected_tether_indexes(&tethers, &[missing]),
            Err(ExecutionServiceError::InvalidInput(_))
        ));
    }

    // -----------------------------------------------------------------------
    // J14A: execution identity boundary tests
    // -----------------------------------------------------------------------

    /// Fresh trusted replay identity reaches the typed service result.
    #[test]
    fn j14a_fresh_replay_identity_reaches_typed_service_result() {
        let result = ExecutionServiceResult::Completed {
            evaluation_id: "eval".into(),
            action_id: "action".into(),
            response: json!({}),
            execution_id: Some("exec_00000000-0000-4000-8000-000000000000".into()),
        };
        let envelope = run_command::map_execution_result(&result);
        assert_eq!(envelope.status, OutcomeStatus::Completed);
        assert_eq!(
            envelope.data.get("execution_id").and_then(|v| v.as_str()),
            Some("exec_00000000-0000-4000-8000-000000000000")
        );
    }

    /// Completed result with no execution_id has no public execution_id.
    #[test]
    fn j14a_completed_without_identity_omits_execution_id() {
        let result = ExecutionServiceResult::Completed {
            evaluation_id: "eval".into(),
            action_id: "action".into(),
            response: json!({}),
            execution_id: None,
        };
        let envelope = run_command::map_execution_result(&result);
        assert!(envelope.data.get("execution_id").is_none());
    }

    /// Replay blocked completed success exposes the same identity.
    #[test]
    fn j14a_replay_blocked_success_exposes_identity() {
        let result = ExecutionServiceResult::ReplayBlockedCompletedSuccess {
            evaluation_id: "eval".into(),
            action_id: "action".into(),
            execution_id: Some("exec_00000000-0000-4000-8000-000000000000".into()),
        };
        let envelope = run_command::map_execution_result(&result);
        assert_eq!(envelope.status, OutcomeStatus::Completed);
        assert_eq!(
            envelope.data.get("execution_id").and_then(|v| v.as_str()),
            Some("exec_00000000-0000-4000-8000-000000000000")
        );
        assert_eq!(
            envelope.data["execution_status"],
            "replay_blocked_completed_success"
        );
    }

    /// Deny exposes no execution identity.
    #[test]
    fn j14a_deny_exposes_no_execution_id() {
        let result = ExecutionServiceResult::Denied {
            evaluation_id: "eval".into(),
            action_id: "action".into(),
            reason: "policy".into(),
            execution_id: None,
        };
        let envelope = run_command::map_execution_result(&result);
        assert!(envelope.data.get("execution_id").is_none());
        assert_eq!(envelope.data["execution_status"], "denied");
    }

    /// NoActions exposes no execution identity.
    #[test]
    fn j14a_no_actions_exposes_no_execution_id() {
        let result = ExecutionServiceResult::NoActions {
            evaluation_id: "eval".into(),
            response: json!({}),
        };
        let envelope = run_command::map_execution_result(&result);
        assert!(envelope.data.get("execution_id").is_none());
    }

    /// Ask exposes no execution identity.
    #[test]
    fn j14a_ask_exposes_no_execution_id() {
        let result = ExecutionServiceResult::ApprovalRequired {
            evaluation_id: "eval".into(),
            action_id: "action".into(),
            reason: "host_policy_ask".into(),
        };
        let envelope = run_command::map_execution_result(&result);
        assert!(envelope.data.get("execution_id").is_none());
    }

    /// Failed after admission exposes execution_id.
    #[test]
    fn j14a_failed_after_admission_exposes_execution_id() {
        let result = ExecutionServiceResult::Failed {
            evaluation_id: "eval".into(),
            action_id: "action".into(),
            reason: "provider error".into(),
            execution_id: Some("exec_00000000-0000-4000-8000-000000000000".into()),
        };
        let envelope = run_command::map_execution_result(&result);
        assert_eq!(
            envelope.data.get("execution_id").and_then(|v| v.as_str()),
            Some("exec_00000000-0000-4000-8000-000000000000")
        );
    }

    /// Uncertain after admission exposes execution_id.
    #[test]
    fn j14a_uncertain_after_admission_exposes_execution_id() {
        let result = ExecutionServiceResult::Uncertain {
            evaluation_id: "eval".into(),
            action_id: "action".into(),
            reason: "timeout".into(),
            execution_id: Some("exec_00000000-0000-4000-8000-000000000001".into()),
        };
        let envelope = run_command::map_execution_result(&result);
        assert_eq!(
            envelope.data.get("execution_id").and_then(|v| v.as_str()),
            Some("exec_00000000-0000-4000-8000-000000000001")
        );
    }

    /// Unattempted after admission exposes execution_id.
    #[test]
    fn j14a_unattempted_after_admission_exposes_execution_id() {
        let result = ExecutionServiceResult::Unattempted {
            evaluation_id: "eval".into(),
            action_id: "action".into(),
            reason: "deadline".into(),
            execution_id: Some("exec_00000000-0000-4000-8000-000000000002".into()),
        };
        let envelope = run_command::map_execution_result(&result);
        assert_eq!(
            envelope.data.get("execution_id").and_then(|v| v.as_str()),
            Some("exec_00000000-0000-4000-8000-000000000002")
        );
    }

    /// AuditFailed after admission may expose execution_id.
    #[test]
    fn j14a_audit_failed_after_admission_may_expose_execution_id() {
        let result = ExecutionServiceResult::AuditFailed {
            evaluation_id: "eval".into(),
            action_id: "action".into(),
            reason: "trail write failed".into(),
            execution_id: Some("exec_00000000-0000-4000-8000-000000000003".into()),
        };
        let envelope = run_command::map_execution_result(&result);
        assert_eq!(
            envelope.data.get("execution_id").and_then(|v| v.as_str()),
            Some("exec_00000000-0000-4000-8000-000000000003")
        );
    }

    /// Pre-admission unavailable exposes no execution_id.
    #[test]
    fn j14a_unavailable_before_admission_no_execution_id() {
        let result = ExecutionServiceResult::Unavailable {
            evaluation_id: "eval".into(),
            reason: "unavailable".into(),
        };
        let envelope = run_command::map_execution_result(&result);
        assert!(envelope.data.get("execution_id").is_none());
    }

    /// Replay persistence unavailable exposes no execution_id.
    #[test]
    fn j14a_replay_persistence_unavailable_no_execution_id() {
        let result = ExecutionServiceResult::ReplayPersistenceUnavailable {
            evaluation_id: "eval".into(),
            action_id: "action".into(),
            execution_id: None,
        };
        let envelope = run_command::map_execution_result(&result);
        assert!(envelope.data.get("execution_id").is_none());
    }

    /// InvalidData exposes no execution_id.
    #[test]
    fn j14a_invalid_data_no_execution_id() {
        let result = ExecutionServiceResult::InvalidData {
            message: "bad".into(),
        };
        let envelope = run_command::map_execution_result(&result);
        assert!(envelope.data.get("execution_id").is_none());
    }

    /// Interrupted exposes no execution_id.
    #[test]
    fn j14a_interrupted_no_execution_id() {
        let envelope = run_command::map_execution_result(&ExecutionServiceResult::Interrupted);
        assert!(envelope.data.get("execution_id").is_none());
    }

    /// PlannerError exposes no execution_id.
    #[test]
    fn j14a_planner_error_no_execution_id() {
        let result = ExecutionServiceResult::PlannerError {
            evaluation_id: Some("eval".into()),
            code: "parse".into(),
            message: "bad".into(),
        };
        let envelope = run_command::map_execution_result(&result);
        assert!(envelope.data.get("execution_id").is_none());
    }

    /// Post-admission Denied (from shared boundary) exposes execution_id.
    #[test]
    fn j14a_post_admission_denied_exposes_execution_id() {
        let result = HostExecutionService::map_shared_result(
            crate::SharedExecutionResult {
                outcome: crate::SharedExecutionOutcome::Denied,
                execution_id: Some("exec_00000000-0000-4000-8000-000000000001".into()),
            },
            "eval".into(),
            "action".into(),
            serde_json::json!({}),
        );
        assert!(matches!(result, ExecutionServiceResult::Denied {
            execution_id: Some(ref id), ..
        } if id == "exec_00000000-0000-4000-8000-000000000001"));
        let envelope = run_command::map_execution_result(&result);
        assert_eq!(
            envelope.data.get("execution_id").and_then(|v| v.as_str()),
            Some("exec_00000000-0000-4000-8000-000000000001")
        );
        assert_eq!(envelope.data["execution_status"], "denied");
    }

    /// ReplayPersistenceUnavailable carries execution_id from shared result.
    #[test]
    fn j14a_replay_persistence_unavailable_with_identity() {
        let result = HostExecutionService::map_shared_result(
            crate::SharedExecutionResult {
                outcome: crate::SharedExecutionOutcome::Replay(
                    crate::replay_runtime::ReplayDispatchResult::PersistenceUnavailable,
                ),
                execution_id: Some("exec_00000000-0000-4000-8000-000000000002".into()),
            },
            "eval".into(),
            "action".into(),
            serde_json::json!({}),
        );
        assert!(matches!(
            result,
            ExecutionServiceResult::ReplayPersistenceUnavailable {
                execution_id: Some(ref id), ..
            } if id == "exec_00000000-0000-4000-8000-000000000002"
        ));
        let envelope = run_command::map_execution_result(&result);
        assert_eq!(
            envelope.data.get("execution_id").and_then(|v| v.as_str()),
            Some("exec_00000000-0000-4000-8000-000000000002")
        );
    }

    /// The public ID passes replay::ExecutionId::parse.
    #[test]
    fn j14a_public_execution_id_passes_parse() {
        let id =
            crate::replay::ExecutionId::parse("exec_00000000-0000-4000-8000-000000000001".into())
                .unwrap();
        assert_eq!(id.as_str(), "exec_00000000-0000-4000-8000-000000000001");
    }

    // -----------------------------------------------------------------------
    // C1C-1 — present non-array top-level plan.groups fails closed
    // -----------------------------------------------------------------------

    /// Build a minimal verified manifest for the runtime fixture.
    fn c1c1_test_manifest() -> (String, String) {
        let mut m = json!({
            "manifest_format_version": "1.0",
            "capability_name": "lantern.task.record",
            "capability_version": 1,
            "title": "Test Capability",
            "description": "A test capability.",
            "input_schema": {
                "type": "object",
                "properties": { "path": { "type": "string" } },
                "required": ["path"]
            },
            "output_schema": {
                "type": "object",
                "properties": { "result": { "type": "string" } }
            },
            "effects": ["test.effect"],
            "permission_scope": null,
            "reversibility": "reversible",
            "determinism": "deterministic",
            "idempotency": { "mechanism": "none" },
            "confirmation_policy": {
                "standing_permitted": false,
                "per_call_required": true
            },
            "timeout_ms": 5000,
            "retry_policy": {
                "max_retries": 0,
                "backoff_ms": 500,
                "allowed_on": [],
                "requires_idempotency_proof": false
            },
            "provider": {
                "identity": "lantern-local",
                "display_name": "Test Provider",
                "identity_source": "host_configuration",
                "description": "Host-assigned."
            },
            "binding": {
                "kind": "mcp",
                "server_name": "test-server",
                "tool_name": "test_tool",
                "adapter": null
            }
        });
        let (_, digest) = crate::manifest::canonicalize_and_digest(&m.to_string()).unwrap();
        m["digest"] = json!(digest);
        (m.to_string(), digest)
    }

    /// Build a real `PreparedRuntime` (config + tether + manifest on disk),
    /// the established repository pattern for production-route host tests.
    fn prepared_runtime_for_c1c1() -> (PreparedRuntime, PathBuf) {
        let dir =
            std::env::temp_dir().join(format!("tethers-c1c1-runtime-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(dir.join("tethers")).unwrap();
        std::fs::create_dir_all(dir.join("manifests")).unwrap();
        std::fs::write(
            dir.join("tethers/record-completed-task.tether"),
            "when event.task.completed if task.status == \"done\" do lantern.task.record",
        )
        .unwrap();

        let (manifest_json, digest) = c1c1_test_manifest();
        std::fs::write(
            dir.join("manifests/lantern-task-record.json"),
            &manifest_json,
        )
        .unwrap();

        let mut config = json!({
            "format_version": "0.1",
            "tether_set": {
                "id": "example.local",
                "version": "1",
                "tethers": [
                    {
                        "id": "record-completed-task",
                        "version": "demo-v1",
                        "source_path": "tethers/record-completed-task.tether"
                    }
                ],
                "capability_requirements": [
                    {
                        "name": "lantern.task.record",
                        "version": 1,
                        "reason": "Record a completed task"
                    }
                ]
            },
            "providers": [
                {
                    "id": "lantern-local",
                    "display_name": "Lantern Local",
                    "transport": {
                        "kind": "stdio",
                        "command": "pwsh.exe",
                        "args": ["-NoProfile", "-File", "providers/lantern.ps1"],
                        "protocol_version": "2025-11-25"
                    },
                    "capabilities": [
                        {
                            "name": "lantern.task.record",
                            "version": 1,
                            "manifest_path": "manifests/lantern-task-record.json",
                            "pinned_digest": ""
                        }
                    ]
                }
            ],
            "policy": {
                "default": "deny",
                "rules": [
                    { "name": "lantern.task.record", "version": 1, "decision": "allow" }
                ]
            }
        });
        config["providers"][0]["capabilities"][0]["pinned_digest"] = json!(digest);
        let config_path = dir.join("tethers-config.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&config).unwrap()).unwrap();

        let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
        let prepared = prepare_runtime(&loaded).unwrap();
        (prepared, dir)
    }

    /// A matched planner response with the given `plan.actions` and optional
    /// `plan.groups` value, exactly as `dispatch_matched_plan` consumes it.
    fn c1c1_matched_response(actions: Vec<Value>, groups: Option<Value>) -> Value {
        let mut plan = json!({
            "id": "plan-c1c1",
            "actions": actions,
        });
        if let Some(groups) = groups {
            plan["groups"] = groups;
        }
        json!({
            "status": "matched",
            "evaluation_id": "eval-correlation-1",
            "plan": plan,
            "trail": [],
        })
    }

    fn c1c1_actions() -> Vec<Value> {
        vec![
            json!({
                "action_id": "a1",
                "idempotency_key": "eval-correlation-1/a1",
                "capability": "lantern.task.record",
                "capability_version": "1.0.0",
                "bridge_capability_version": 1,
                "bridge_provider_identity": "lantern-local",
                "arguments": {},
            }),
            json!({
                "action_id": "a2",
                "idempotency_key": "eval-correlation-1/a2",
                "capability": "lantern.task.record",
                "capability_version": "1.0.0",
                "bridge_capability_version": 1,
                "bridge_provider_identity": "lantern-local",
                "arguments": {},
            }),
        ]
    }

    /// A present non-array top-level `plan.groups` value is malformed metadata
    /// and must be rejected as `InvalidData` before any Action dispatch — it
    /// must never be silently reinterpreted as sequential execution.  Absent
    /// groups and a present JSON array must both proceed as valid plans.
    ///
    /// Dispatch is observed through the real production route
    /// (`dispatch_matched_plan` on a real `HostExecutionService` backed by a
    /// real `PreparedRuntime`).  `FileTrail::open` is the last gate before
    /// `execute_plan` / `execute_one_action`, so a trail file that was never
    /// created proves zero executor or provider activity.
    #[test]
    fn c1c1_present_non_array_plan_groups_fails_closed_before_dispatch() {
        let (runtime, runtime_dir) = prepared_runtime_for_c1c1();
        let engine_path = PathBuf::from("unused-engine");
        let input = planner_input();
        let availability = ProviderAvailability::from_identities(["lantern-local"]);
        let mut trail_paths = Vec::new();

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let malformed_values = [
                json!(null),
                json!({ "group_id": "g1" }),
                json!("carrot"),
                json!(1),
                json!(true),
            ];
            for (index, malformed) in malformed_values.iter().enumerate() {
                let trail_path = std::env::temp_dir().join(format!(
                    "tethers-c1c1-malformed-{}-{}.jsonl",
                    index,
                    uuid::Uuid::new_v4().simple()
                ));
                trail_paths.push(trail_path.clone());
                let service = HostExecutionService::new(&runtime, &engine_path, &trail_path, None);
                let mut sessions: HashMap<String, RetainedProviderSession> = HashMap::new();
                let mut approvals = crate::approval::ApprovalStore::default();
                let mut replay_authority = FileReplayAuthority::new(None);
                let result = service.dispatch_matched_plan(
                    &input,
                    c1c1_matched_response(c1c1_actions(), Some(malformed.clone())),
                    &mut sessions,
                    &availability,
                    &mut approvals,
                    &mut replay_authority,
                );
                assert!(
                    matches!(&result, ExecutionServiceResult::InvalidData { message } if message.contains("plan.groups")),
                    "present non-array plan.groups ({malformed:?}) must fail closed, got: {result:?}"
                );
                assert!(
                    !trail_path.exists(),
                    "malformed plan.groups must stop before Trail open / any executor or provider activity"
                );
            }

            for (label, groups) in [
                ("absent", None::<Value>),
                (
                    "valid_array",
                    Some(json!([
                        { "group_id": "g1", "member_action_ids": ["a1", "a2"] }
                    ])),
                ),
            ] {
                let trail_path = std::env::temp_dir().join(format!(
                    "tethers-c1c1-{label}-{}.jsonl",
                    uuid::Uuid::new_v4().simple()
                ));
                trail_paths.push(trail_path.clone());
                let service = HostExecutionService::new(&runtime, &engine_path, &trail_path, None);
                let mut sessions: HashMap<String, RetainedProviderSession> = HashMap::new();
                let mut approvals = crate::approval::ApprovalStore::default();
                let mut replay_authority = FileReplayAuthority::new(None);
                let result = service.dispatch_matched_plan(
                    &input,
                    c1c1_matched_response(c1c1_actions(), groups),
                    &mut sessions,
                    &availability,
                    &mut approvals,
                    &mut replay_authority,
                );
                assert!(
                    !matches!(&result, ExecutionServiceResult::InvalidData { message } if message.contains("plan.groups")),
                    "{label} plan.groups must not be rejected as malformed, got: {result:?}"
                );
                assert!(
                    trail_path.exists(),
                    "{label} plan.groups must proceed to Trail open (dispatch begins), got: {result:?}"
                );
            }
        }));

        let _ = std::fs::remove_dir_all(&runtime_dir);
        for path in &trail_paths {
            let _ = std::fs::remove_file(path);
        }
        if let Err(e) = result {
            std::panic::resume_unwind(e);
        }
    }

    // -----------------------------------------------------------------------
    // CORE-9B — real Rust builder E2E tests (T9–T14)
    //
    // These tests use build_core_request_envelope to prove the Rust host
    // request builder produces the correct extended request for the Core
    // pipeline.  T12-T14 send the builder output through a real MCP binary.
    // -----------------------------------------------------------------------

    fn core9b_engine_binary_path() -> Option<PathBuf> {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.pop();
        path.push("engine-ocaml");
        path.push("_build");
        path.push("default");
        path.push("bin");
        path.push("tethers_mcp_main.exe");
        if path.exists() {
            Some(path)
        } else {
            None
        }
    }

    fn core9b_require_engine() -> (PathBuf, PathBuf) {
        let ep = core9b_engine_binary_path()
            .expect("engine binary not found; build with opam exec -- dune build");
        let wd = ep.parent().unwrap().to_path_buf();
        (ep, wd)
    }

    const CORE_REHEARSAL_TETHER: &str =
        "tether \"core rehearsal\"\n\nanchor\n    fixture.start\n\nwhen\n\ndo\n    notify\n        message: anchor.message\n";

    /// Build the fixture-ping manifest from the repository source.
    fn core9b_fixture_ping_manifest() -> crate::manifest::VerifiedManifest {
        crate::manifest::verify_manifest(include_str!(
            "../../protocol/capability-manifests/fixture-ping.json"
        ))
        .unwrap()
    }

    /// Build a PreparedRuntime with CORE-9A semantic authority on the tether,
    /// using the real fixture-ping manifest and provider.
    fn core9b_prepared_runtime_with_core() -> (PreparedRuntime, PathBuf) {
        let dir =
            std::env::temp_dir().join(format!("tethers-core9b-runtime-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(dir.join("tethers")).unwrap();
        std::fs::create_dir_all(dir.join("manifests")).unwrap();
        std::fs::write(
            dir.join("tethers/core-rehearsal.tether"),
            CORE_REHEARSAL_TETHER,
        )
        .unwrap();

        let _verified_manifest = core9b_fixture_ping_manifest();
        let manifest_json = include_str!("../../protocol/capability-manifests/fixture-ping.json");
        let (_, manifest_digest) = crate::manifest::canonicalize_and_digest(manifest_json).unwrap();
        std::fs::write(dir.join("manifests/fixture-ping.json"), manifest_json).unwrap();

        let config = json!({
            "format_version": "0.1",
            "tether_set": {
                "id": "core9b.test",
                "version": "1",
                "tethers": [
                    {
                        "id": "core-rehearsal",
                        "version": "1",
                        "source_path": "tethers/core-rehearsal.tether",
                        "core_environment": {
                            "program_id": "program.core9b",
                            "core_version": "1",
                            "capabilities": [
                                {
                                    "source_name": "notify",
                                    "capability_id": "cap.semantic.notify",
                                    "contract_digest": "CORE-CONTRACT-9B",
                                    "runtime_name": "fixture.ping"
                                }
                            ],
                            "input_facts": []
                        }
                    }
                ],
                "capability_requirements": [
                    {
                        "name": "fixture.ping",
                        "version": 1,
                        "reason": "Core rehearsal"
                    }
                ]
            },
            "providers": [
                {
                    "id": "tethers-stdio-fixture",
                    "display_name": "Tethers Stdio Fixture",
                    "transport": {
                        "kind": "stdio",
                        "command": "pwsh.exe",
                        "args": ["-NoProfile", "-File", "providers/fixture.ps1"],
                        "protocol_version": "2025-11-25"
                    },
                    "capabilities": [
                        {
                            "name": "fixture.ping",
                            "version": 1,
                            "manifest_path": "manifests/fixture-ping.json",
                            "pinned_digest": manifest_digest
                        }
                    ]
                }
            ],
            "policy": {
                "default": "deny",
                "rules": [
                    {
                        "name": "fixture.ping",
                        "version": 1,
                        "decision": "allow"
                    }
                ]
            }
        });

        let config_path = dir.join("tethers-config.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&config).unwrap()).unwrap();

        let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
        let prepared = prepare_runtime(&loaded).unwrap();
        (prepared, dir)
    }

    /// Build a PreparedRuntime WITHOUT core_environment on the tether.
    fn core9b_prepared_runtime_no_core() -> (PreparedRuntime, PathBuf) {
        let dir =
            std::env::temp_dir().join(format!("tethers-core9b-no-core-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(dir.join("tethers")).unwrap();
        std::fs::create_dir_all(dir.join("manifests")).unwrap();
        std::fs::write(
            dir.join("tethers/core-rehearsal.tether"),
            CORE_REHEARSAL_TETHER,
        )
        .unwrap();

        let manifest_json = include_str!("../../protocol/capability-manifests/fixture-ping.json");
        let (_, manifest_digest) = crate::manifest::canonicalize_and_digest(manifest_json).unwrap();
        std::fs::write(dir.join("manifests/fixture-ping.json"), manifest_json).unwrap();

        let config = json!({
            "format_version": "0.1",
            "tether_set": {
                "id": "core9b.test",
                "version": "1",
                "tethers": [
                    {
                        "id": "core-rehearsal",
                        "version": "1",
                        "source_path": "tethers/core-rehearsal.tether"
                    }
                ],
                "capability_requirements": [
                    {
                        "name": "fixture.ping",
                        "version": 1,
                        "reason": "Core rehearsal"
                    }
                ]
            },
            "providers": [
                {
                    "id": "tethers-stdio-fixture",
                    "display_name": "Tethers Stdio Fixture",
                    "transport": {
                        "kind": "stdio",
                        "command": "pwsh.exe",
                        "args": ["-NoProfile", "-File", "providers/fixture.ps1"],
                        "protocol_version": "2025-11-25"
                    },
                    "capabilities": [
                        {
                            "name": "fixture.ping",
                            "version": 1,
                            "manifest_path": "manifests/fixture-ping.json",
                            "pinned_digest": manifest_digest
                        }
                    ]
                }
            ],
            "policy": {
                "default": "deny",
                "rules": [
                    {
                        "name": "fixture.ping",
                        "version": 1,
                        "decision": "allow"
                    }
                ]
            }
        });

        let config_path = dir.join("tethers-config.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&config).unwrap()).unwrap();

        let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
        let prepared = prepare_runtime(&loaded).unwrap();
        (prepared, dir)
    }

    /// Build a core request envelope through the real builder and return
    /// (request_json, service, runtime_dir).  Caller must keep runtime_dir
    /// alive until after assertions.
    fn core9b_build_request(
        runtime: &PreparedRuntime,
        engine_path: &Path,
        evaluation_id: &str,
        event_name: &str,
    ) -> Result<Value, ExecutionServiceResult> {
        let tether = &runtime.tethers()[0];
        let input = PreparedEvaluationInput {
            tether_id: tether.id.clone(),
            tether_version: tether.version.clone(),
            evaluation_id: evaluation_id.to_owned(),
            anchor_event: json!({
                "id": format!("evt_{evaluation_id}"),
                "name": event_name,
                "data": { "message": "Hello Core" }
            }),
            facts: json!({}),
        };
        let availability = ProviderAvailability::from_identities(["tethers-stdio-fixture"]);
        let trail_path = std::env::temp_dir().join(format!(
            "tethers-core9b-trail-{}.jsonl",
            uuid::Uuid::new_v4()
        ));
        let service = HostExecutionService::new(runtime, engine_path, &trail_path, None);
        service.build_core_request_envelope(&input, tether, &availability)
    }

    /// T9: build_core_request_envelope fails when core_environment is absent.
    #[test]
    fn core9b_t9_builder_fails_without_core_environment() {
        let (runtime, _dir) = core9b_prepared_runtime_no_core();
        let engine_path = PathBuf::from("unused-engine");
        let tether = &runtime.tethers()[0];
        let input = PreparedEvaluationInput {
            tether_id: tether.id.clone(),
            tether_version: tether.version.clone(),
            evaluation_id: "eval_t9_no_core".to_owned(),
            anchor_event: json!({
                "id": "evt_eval_t9_no_core",
                "name": "fixture.start",
                "data": { "message": "Hello Core" }
            }),
            facts: json!({}),
        };
        let availability = ProviderAvailability::from_identities(["tethers-stdio-fixture"]);
        let trail_path =
            std::env::temp_dir().join(format!("tethers-core9b-t9-{}.jsonl", uuid::Uuid::new_v4()));
        let service = HostExecutionService::new(&runtime, &engine_path, &trail_path, None);
        let result = service.build_core_request_envelope(&input, tether, &availability);
        match result {
            Err(ExecutionServiceResult::InvalidData { message }) => {
                assert!(
                    message.contains("no core_environment"),
                    "expected 'no core_environment' in error, got: {message}"
                );
            }
            other => {
                panic!("T9: expected InvalidData for missing core_environment, got: {other:?}")
            }
        }
        std::fs::remove_dir_all(&_dir).ok();
    }

    /// T10: Identity separation — inspect builder output for exact Core
    ///      identities and runtime identities.  No derivation.
    #[test]
    fn core9b_t10_builder_identity_separation() {
        let (runtime, _dir) = core9b_prepared_runtime_with_core();
        let engine_path = PathBuf::from("unused-engine");
        let request =
            core9b_build_request(&runtime, &engine_path, "eval_t10_identity", "fixture.start")
                .expect("builder should succeed");

        // Core identity in core_environment
        let core_caps = request
            .pointer("/core_environment/capabilities")
            .and_then(Value::as_array)
            .expect("core_environment.capabilities");
        assert_eq!(core_caps.len(), 1);
        let cc = &core_caps[0];
        assert_eq!(
            cc.get("source_name").and_then(Value::as_str),
            Some("notify"),
            "Core source_name must be notify"
        );
        assert_eq!(
            cc.get("capability_id").and_then(Value::as_str),
            Some("cap.semantic.notify"),
            "Core capability_id must be cap.semantic.notify"
        );
        assert_eq!(
            cc.get("contract_digest").and_then(Value::as_str),
            Some("CORE-CONTRACT-9B"),
            "Core contract_digest must be CORE-CONTRACT-9B"
        );
        assert_eq!(
            cc.get("runtime_name").and_then(Value::as_str),
            Some("fixture.ping"),
            "runtime_name must be fixture.ping"
        );

        // Top-level runtime capability name must be fixture.ping
        let caps = request
            .get("capabilities")
            .and_then(Value::as_array)
            .expect("capabilities");
        assert_eq!(caps.len(), 1);
        assert_eq!(
            caps[0].get("name").and_then(Value::as_str),
            Some("fixture.ping"),
            "runtime capability name must be fixture.ping"
        );

        std::fs::remove_dir_all(&_dir).ok();
    }

    /// T11: Bridge metadata separation — inspect builder output.
    ///      core_environment must NOT contain manifest_digest,
    ///      bridge_capability_version, or bridge_provider_identity.
    ///      Top-level runtime capability MUST contain them with real values.
    #[test]
    fn core9b_t11_builder_bridge_metadata_separation() {
        let (runtime, _dir) = core9b_prepared_runtime_with_core();
        let engine_path = PathBuf::from("unused-engine");
        let request =
            core9b_build_request(&runtime, &engine_path, "eval_t11_bridge", "fixture.start")
                .expect("builder should succeed");

        // core_environment must NOT contain bridge metadata
        let core_env = request.get("core_environment").expect("core_environment");
        assert!(
            core_env.get("manifest_digest").is_none(),
            "core_environment must not contain manifest_digest"
        );
        assert!(
            core_env.get("bridge_capability_version").is_none(),
            "core_environment must not contain bridge_capability_version"
        );
        assert!(
            core_env.get("bridge_provider_identity").is_none(),
            "core_environment must not contain bridge_provider_identity"
        );

        // Top-level runtime capability MUST contain bridge metadata
        let caps = request
            .get("capabilities")
            .and_then(Value::as_array)
            .expect("capabilities");
        assert_eq!(caps.len(), 1);
        let cap = &caps[0];
        assert_eq!(
            cap.get("manifest_digest").and_then(Value::as_str),
            Some("sha256:01fed7a4b877dd82abe91a1b6cfcd476b02e4c115489e70cbb285b8bf2d32d8b"),
            "runtime capability must have manifest_digest"
        );
        assert_eq!(
            cap.get("bridge_capability_version").and_then(Value::as_i64),
            Some(1),
            "runtime capability must have bridge_capability_version = 1"
        );
        assert_eq!(
            cap.get("bridge_provider_identity").and_then(Value::as_str),
            Some("tethers-stdio-fixture"),
            "runtime capability must have bridge_provider_identity"
        );

        // CORE-CONTRACT-9B must differ from manifest digest
        let contract_digest = core_env["capabilities"][0]["contract_digest"]
            .as_str()
            .unwrap();
        let manifest_digest = cap.get("manifest_digest").and_then(Value::as_str).unwrap();
        assert_ne!(
            contract_digest, manifest_digest,
            "Core contract_digest must differ from manifest digest"
        );

        std::fs::remove_dir_all(&_dir).ok();
    }

    /// T12: The mandatory E2E — runtime config → PreparedRuntime →
    ///      build_core_request_envelope → real MCP binary →
    ///      tethers.evaluate_core → Tethers_core_wire → CORE-8B →
    ///      canonical Core → Runtime Plan.
    #[test]
    fn core9b_t12_real_rust_built_cross_language_e2e() {
        let (runtime, _dir) = core9b_prepared_runtime_with_core();
        let (engine_path, working_dir) = core9b_require_engine();
        let request =
            core9b_build_request(&runtime, &engine_path, "eval_core9b_001", "fixture.start")
                .expect("builder should succeed");
        let mut session = EngineSession::launch(&engine_path, &working_dir).expect("engine launch");
        let wire = session
            .evaluate_tether("eval_core9b_001", &request)
            .expect("evaluate_tether E2E");
        let PlannerResponseWire::Matched(response) = wire else {
            panic!("T12: expected Matched, got {wire:?}");
        };

        // Correlation fields
        assert_eq!(response["protocol_version"], "0.1");
        assert_eq!(response["evaluation_id"], "eval_core9b_001");
        assert_eq!(response["event_id"], "evt_eval_core9b_001");
        assert_eq!(response["tether_id"], "core-rehearsal");
        assert_eq!(response["tether_version"], "1");

        // Plan structure
        let plan = response.get("plan").expect("plan missing");
        let plan_id = plan.get("id").and_then(Value::as_str).expect("plan.id");
        assert_eq!(
            plan_id, "eval_core9b_001/plan",
            "plan.id must be eval_id/plan"
        );

        // program_digest at top level (sibling of plan), NOT inside plan
        let pd = response
            .get("program_digest")
            .and_then(Value::as_str)
            .expect("program_digest missing from top level");
        assert!(
            pd.starts_with("sha256:"),
            "program_digest must start with sha256:"
        );
        assert_eq!(
            pd.len(),
            71,
            "program_digest must be sha256: + 64 hex chars"
        );
        assert!(
            plan.get("program_digest").is_none(),
            "program_digest must NOT be inside plan"
        );

        // Actions
        let actions = plan
            .get("actions")
            .and_then(Value::as_array)
            .expect("actions");
        assert_eq!(actions.len(), 1, "expected exactly one action");
        let action = &actions[0];
        assert_eq!(
            action.get("capability").and_then(Value::as_str),
            Some("fixture.ping"),
            "action capability must be fixture.ping"
        );
        let args = action.get("arguments").expect("action arguments");
        assert_eq!(
            args.get("message").and_then(Value::as_str),
            Some("Hello Core"),
            "action arguments.message must be Hello Core"
        );
        assert_eq!(
            action.get("idempotency_key").and_then(Value::as_str),
            Some("eval_core9b_001/action_1"),
            "idempotency_key must be eval_id/action_1"
        );

        // Effects
        let effects = action
            .get("effects")
            .and_then(Value::as_array)
            .expect("effects");
        assert!(
            effects.iter().any(|e| e.as_str() == Some("fixture.test")),
            "effects must contain fixture.test"
        );

        // Bridge metadata in the plan action
        assert_eq!(
            action.get("manifest_digest").and_then(Value::as_str),
            Some("sha256:01fed7a4b877dd82abe91a1b6cfcd476b02e4c115489e70cbb285b8bf2d32d8b"),
            "action manifest_digest must match fixture-ping"
        );
        assert_eq!(
            action
                .get("bridge_capability_version")
                .and_then(Value::as_i64),
            Some(1),
            "action bridge_capability_version must be 1"
        );
        assert_eq!(
            action
                .get("bridge_provider_identity")
                .and_then(Value::as_str),
            Some("tethers-stdio-fixture"),
            "action bridge_provider_identity must be tethers-stdio-fixture"
        );

        // Trail is empty (compatibility scaffolding)
        let trail = response.get("trail").expect("trail");
        assert!(
            trail.is_array() && trail.as_array().unwrap().is_empty(),
            "trail must be empty array"
        );

        session.shutdown();
    }

    /// T13: Wrong event via builder — produces NotMatched.
    #[test]
    fn core9b_t13_builder_wrong_event_not_matched() {
        let (runtime, _dir) = core9b_prepared_runtime_with_core();
        let (engine_path, working_dir) = core9b_require_engine();
        let request =
            core9b_build_request(&runtime, &engine_path, "eval_t13_wrong", "fixture.other")
                .expect("builder should succeed");
        let mut session = EngineSession::launch(&engine_path, &working_dir).expect("engine launch");
        let wire = session
            .evaluate_tether("eval_t13_wrong", &request)
            .expect("evaluate_tether wrong event");
        match wire {
            PlannerResponseWire::NotMatched(response) => {
                assert_eq!(response["status"], "not_matched");
                assert_eq!(response["evaluation_id"], "eval_t13_wrong");
                assert_eq!(response["event_id"], "evt_eval_t13_wrong");
            }
            other => panic!("T13: expected NotMatched, got {other:?}"),
        }
        session.shutdown();
    }

    /// T14: Occurrence identity via builder — same semantic program,
    ///      different evaluation_id produces same ProgramDigest but
    ///      different plan.id and idempotency keys.
    #[test]
    fn core9b_t14_builder_occurrence_identity() {
        let (runtime, _dir) = core9b_prepared_runtime_with_core();
        let (engine_path, working_dir) = core9b_require_engine();

        let req1 = core9b_build_request(&runtime, &engine_path, "eval_core9b_001", "fixture.start")
            .expect("first builder");
        let mut session = EngineSession::launch(&engine_path, &working_dir).expect("engine launch");
        let wire1 = session
            .evaluate_tether("eval_core9b_001", &req1)
            .expect("first evaluation");
        let PlannerResponseWire::Matched(resp1) = wire1 else {
            panic!("T14: expected Matched for first eval, got {wire1:?}")
        };

        let req2 = core9b_build_request(&runtime, &engine_path, "eval_core9b_002", "fixture.start")
            .expect("second builder");
        let wire2 = session
            .evaluate_tether("eval_core9b_002", &req2)
            .expect("second evaluation");
        let PlannerResponseWire::Matched(resp2) = wire2 else {
            panic!("T14: expected Matched for second eval, got {wire2:?}")
        };

        // Same program_digest
        let pd1 = resp1
            .get("program_digest")
            .and_then(Value::as_str)
            .expect("first program_digest");
        let pd2 = resp2
            .get("program_digest")
            .and_then(Value::as_str)
            .expect("second program_digest");
        assert_eq!(pd1, pd2, "same program must produce same program_digest");

        // Different plan.id
        let pid1 = resp1
            .pointer("/plan/id")
            .and_then(Value::as_str)
            .expect("first plan.id");
        let pid2 = resp2
            .pointer("/plan/id")
            .and_then(Value::as_str)
            .expect("second plan.id");
        assert_ne!(
            pid1, pid2,
            "different evaluations must produce different plan.id"
        );
        assert_eq!(pid1, "eval_core9b_001/plan");
        assert_eq!(pid2, "eval_core9b_002/plan");

        // Different idempotency keys
        let ik1 = resp1
            .pointer("/plan/actions/0/idempotency_key")
            .and_then(Value::as_str)
            .expect("first idempotency_key");
        let ik2 = resp2
            .pointer("/plan/actions/0/idempotency_key")
            .and_then(Value::as_str)
            .expect("second idempotency_key");
        assert_ne!(
            ik1, ik2,
            "different evaluations must produce different idempotency keys"
        );
        assert_eq!(ik1, "eval_core9b_001/action_1");
        assert_eq!(ik2, "eval_core9b_002/action_1");

        session.shutdown();
    }

    // -----------------------------------------------------------------------
    // CORE-9C — real production HostExecutionService proof
    //
    // These tests deliberately use the public service seam rather than the
    // request builder or EngineSession directly.  They prove the ordinary
    // production route owns Core request construction and that matched Core
    // plans continue through existing policy, replay, dispatch, and Trail
    // machinery without a legacy planning fallback.
    // -----------------------------------------------------------------------

    const CORE_PRODUCTION_TETHER: &str =
        "tether \"core production dispatch\"\n\nanchor\n    fixture.start\n\nwhen\n\ndo\n    notify\n        message: anchor.message\n        path: anchor.path\n";

    fn core9c_prepared_runtime_for_dispatch(
        with_core_environment: bool,
        marker_path: &Path,
    ) -> (PreparedRuntime, PathBuf) {
        let dir = std::env::temp_dir().join(format!(
            "tethers-core9c-production-{}",
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(dir.join("tethers")).unwrap();
        std::fs::create_dir_all(dir.join("manifests")).unwrap();
        std::fs::write(
            dir.join("tethers/core-production.tether"),
            CORE_PRODUCTION_TETHER,
        )
        .unwrap();

        let manifest =
            include_str!("../../protocol/capability-manifests/fixture-ping-standing-allow.json");
        let (_, manifest_digest) = crate::manifest::canonicalize_and_digest(manifest).unwrap();
        std::fs::write(dir.join("manifests/fixture-ping.json"), manifest).unwrap();

        let fixture_script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("scripts")
            .join("tethers-stdio-fixture.ps1");
        let mut tether = json!({
            "id": "core-production",
            "version": "1",
            "source_path": "tethers/core-production.tether"
        });
        if with_core_environment {
            tether.as_object_mut().unwrap().insert(
                "core_environment".to_owned(),
                json!({
                    "program_id": "program.core9c.production",
                    "core_version": "1",
                    "capabilities": [{
                        "source_name": "notify",
                        "capability_id": "cap.semantic.notify",
                        "contract_digest": "CORE-CONTRACT-9C",
                        "runtime_name": "fixture.ping"
                    }],
                    "input_facts": []
                }),
            );
        }

        let config = json!({
            "format_version": "0.1",
            "tether_set": {
                "id": "core9c.production.test",
                "version": "1",
                "tethers": [tether],
                "capability_requirements": [{
                    "name": "fixture.ping",
                    "version": 1,
                    "reason": "CORE-9C production dispatch proof"
                }]
            },
            "providers": [{
                "id": "tethers-stdio-fixture",
                "display_name": "Tethers Stdio Fixture",
                "transport": {
                    "kind": "stdio",
                    "command": "pwsh.exe",
                    "args": [
                        "-NoProfile", "-ExecutionPolicy", "Bypass", "-File",
                        fixture_script, "-Mode", "run-success", "-MarkerFile", marker_path
                    ],
                    "protocol_version": "2025-11-25"
                },
                "capabilities": [{
                    "name": "fixture.ping",
                    "version": 1,
                    "manifest_path": "manifests/fixture-ping.json",
                    "pinned_digest": manifest_digest,
                    "scope_binding": {
                        "kind": "path_prefix",
                        "argument_json_pointer": "/path"
                    }
                }]
            }],
            "policy": {
                "default": "deny",
                "rules": [{
                    "name": "fixture.ping",
                    "version": 1,
                    "decision": "allow"
                }]
            }
        });
        let config_path = dir.join("tethers-config.json");
        std::fs::write(&config_path, serde_json::to_vec_pretty(&config).unwrap()).unwrap();
        let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
        (prepare_runtime(&loaded).unwrap(), dir)
    }

    fn core9c_input(evaluation_id: &str, event_name: &str) -> PreparedEvaluationInput {
        PreparedEvaluationInput {
            tether_id: "core-production".to_owned(),
            tether_version: "1".to_owned(),
            evaluation_id: evaluation_id.to_owned(),
            anchor_event: json!({
                "id": format!("evt_{evaluation_id}"),
                "name": event_name,
                "data": {
                    "message": "Hello Core Production",
                    "path": "projects/core9c.txt"
                }
            }),
            facts: json!({}),
        }
    }

    fn marker_calls(marker_path: &Path) -> Vec<String> {
        std::fs::read_to_string(marker_path)
            .unwrap_or_default()
            .lines()
            .map(str::to_owned)
            .collect()
    }

    fn core9c_provision_replay_root(root: &Path) {
        std::fs::create_dir_all(root).expect("T15: create replay root");
        let acl_script = format!(
            "$p='{}'; $identity=[System.Security.Principal.WindowsIdentity]::GetCurrent().Name; $acl=[System.Security.AccessControl.DirectorySecurity]::new(); $acl.SetAccessRuleProtection($true,$false); $inherit=[System.Security.AccessControl.InheritanceFlags]::ContainerInherit -bor [System.Security.AccessControl.InheritanceFlags]::ObjectInherit; foreach($t in @($identity,'NT AUTHORITY\\SYSTEM','BUILTIN\\Administrators')) {{ $acl.AddAccessRule([System.Security.AccessControl.FileSystemAccessRule]::new($t,'FullControl',$inherit,'None','Allow')) }}; Set-Acl -LiteralPath $p -AclObject $acl",
            root.to_string_lossy()
        );
        assert!(
            Command::new("pwsh.exe")
                .args(["-NoProfile", "-Command", &acl_script])
                .status()
                .expect("T15: set replay-root ACL")
                .success(),
            "T15: replay root must receive the accepted protected ACL"
        );
        assert!(matches!(
            crate::replay_windows::provision_replay(root),
            Ok(crate::replay_windows::ProvisionReplayOutcome::Provisioned)
        ));
    }

    #[test]
    fn core9c_t15_production_service_dispatches_core_plan_and_preserves_identity() {
        let marker_path = std::env::temp_dir().join(format!(
            "tethers-core9c-marker-{}.txt",
            uuid::Uuid::new_v4()
        ));
        let (runtime, dir) = core9c_prepared_runtime_for_dispatch(true, &marker_path);
        let (engine_path, _) = core9b_require_engine();
        let trail_path = dir.join("trail.jsonl");
        let host_data_root = dir.join("host-data");
        core9c_provision_replay_root(&host_data_root);
        let service =
            HostExecutionService::new(&runtime, &engine_path, &trail_path, Some(&host_data_root));
        let results = service
            .run_selected(&[
                core9c_input("eval_core9c_001", "fixture.start"),
                core9c_input("eval_core9c_002", "fixture.start"),
            ])
            .expect("production service must complete");
        assert_eq!(results.len(), 2);

        let mut program_digests = Vec::new();
        for (index, result) in results.iter().enumerate() {
            let expected_evaluation_id = format!("eval_core9c_{:03}", index + 1);
            let ExecutionServiceResult::Completed {
                evaluation_id,
                response,
                ..
            } = result
            else {
                panic!("T15: expected completed Core production result, got {result:?}");
            };
            assert_eq!(evaluation_id, &expected_evaluation_id);
            assert_eq!(response["status"], "matched");
            assert_eq!(response["evaluation_id"], expected_evaluation_id);
            assert_eq!(
                response.pointer("/plan/id"),
                Some(&Value::String(format!("{expected_evaluation_id}/plan")))
            );
            let program_digest = response["program_digest"]
                .as_str()
                .expect("T15: program_digest must be top-level");
            assert!(program_digest.starts_with("sha256:"));
            assert!(response.pointer("/plan/program_digest").is_none());
            assert_eq!(
                response.pointer("/plan/actions/0/capability"),
                Some(&Value::String("fixture.ping".to_owned()))
            );
            assert_eq!(
                response.pointer("/plan/actions/0/arguments/path"),
                Some(&Value::String("projects/core9c.txt".to_owned()))
            );
            program_digests.push(program_digest.to_owned());
        }
        assert_eq!(
            program_digests[0], program_digests[1],
            "ProgramDigest identifies the stable Core program, not an occurrence"
        );

        let calls = marker_calls(&marker_path);
        assert_eq!(
            calls
                .iter()
                .filter(|call| call.as_str() == "tools/call")
                .count(),
            2,
            "each matched occurrence must dispatch exactly once"
        );
        let trail: Vec<Value> = std::fs::read_to_string(&trail_path)
            .expect("T15: Trail must exist after dispatch")
            .lines()
            .map(|line| serde_json::from_str(line).expect("T15: Trail JSONL"))
            .collect();
        assert_eq!(trail.len(), 4, "two actions must record intent and outcome");
        assert_eq!(trail[0]["capability_name"], "fixture.ping");
        assert_eq!(trail[1]["status"], "succeeded");
        assert_eq!(trail[3]["status"], "succeeded");

        let _ = std::fs::remove_dir_all(dir);
        let _ = std::fs::remove_file(marker_path);
    }

    #[test]
    fn core9c_t16_wrong_event_never_dispatches() {
        let marker_path = std::env::temp_dir().join(format!(
            "tethers-core9c-wrong-event-{}.txt",
            uuid::Uuid::new_v4()
        ));
        let (runtime, dir) = core9c_prepared_runtime_for_dispatch(true, &marker_path);
        let (engine_path, _) = core9b_require_engine();
        let trail_path = dir.join("trail.jsonl");
        let host_data_root = dir.join("host-data");
        let service =
            HostExecutionService::new(&runtime, &engine_path, &trail_path, Some(&host_data_root));
        let results = service
            .run_selected(&[core9c_input("eval_core9c_wrong", "fixture.other")])
            .expect("wrong event must be an ordinary planner outcome");
        assert!(matches!(
            &results[0],
            ExecutionServiceResult::NoActions { evaluation_id, response }
                if evaluation_id == "eval_core9c_wrong" && response["status"] == "not_matched"
        ));
        assert!(
            !marker_calls(&marker_path)
                .iter()
                .any(|call| call == "tools/call"),
            "wrong Core event must not cross the provider boundary"
        );
        assert!(
            !trail_path.exists(),
            "wrong Core event must not enter dispatch or create a host Trail"
        );

        let _ = std::fs::remove_dir_all(dir);
        let _ = std::fs::remove_file(marker_path);
    }

    #[test]
    fn core9c_t17_missing_core_environment_fails_closed_without_dispatch() {
        let marker_path = std::env::temp_dir().join(format!(
            "tethers-core9c-no-core-{}.txt",
            uuid::Uuid::new_v4()
        ));
        let (runtime, dir) = core9c_prepared_runtime_for_dispatch(false, &marker_path);
        let (engine_path, _) = core9b_require_engine();
        let trail_path = dir.join("trail.jsonl");
        let host_data_root = dir.join("host-data");
        let service =
            HostExecutionService::new(&runtime, &engine_path, &trail_path, Some(&host_data_root));
        let results = service
            .run_selected(&[core9c_input("eval_core9c_no_core", "fixture.start")])
            .expect("missing Core authority is a service result, not transport failure");
        assert!(matches!(
            &results[0],
            ExecutionServiceResult::InvalidData { message }
                if message.contains("no core_environment")
        ));
        assert!(
            !marker_calls(&marker_path)
                .iter()
                .any(|call| call == "tools/call"),
            "missing Core authority must not fall back to legacy evaluation or dispatch"
        );
        assert!(
            !trail_path.exists(),
            "missing Core authority must fail before dispatch or Trail creation"
        );

        let _ = std::fs::remove_dir_all(dir);
        let _ = std::fs::remove_file(marker_path);
    }

    // -----------------------------------------------------------------------
    // C2-A3a Concurrency Observability Tests
    //
    // Proves Stage C durability, physical Trail ordering, GroupJoin ordering,
    // and worker panic handling for the concurrent Together path.
    // -----------------------------------------------------------------------

    /// Build a PreparedRuntime with two barrier-fixture providers, each with
    /// a unique capability name.  Each provider points at `barrier_dir` for
    /// deterministic file-system synchronization.
    fn c2a3a_barrier_runtime(barrier_dir: &Path) -> (PreparedRuntime, PathBuf, String, String) {
        let dir = std::env::temp_dir().join(format!(
            "tethers-c2a3a-obs-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(dir.join("tethers")).unwrap();
        std::fs::create_dir_all(dir.join("manifests")).unwrap();

        std::fs::write(
            dir.join("tethers/together-test.tether"),
            "when event.test if true do fixture.ping-a do fixture.ping-b",
        )
        .unwrap();

        let manifest_json = include_str!("../../protocol/capability-manifests/fixture-ping.json");
        // Create two manifests with distinct capability names, provider identities, and valid digests.
        let mut manifest_a: serde_json::Value = serde_json::from_str(manifest_json).unwrap();
        manifest_a["capability_name"] = serde_json::json!("fixture.ping-a");
        manifest_a["provider"]["identity"] = serde_json::json!("provider-a");
        manifest_a["binding"]["server_name"] = serde_json::json!("tethers-stdio-fixture");
        manifest_a["permission_scope"] =
            serde_json::json!({"kind": "path_prefix", "allowed_prefixes": ["member/"]});
        manifest_a["confirmation_policy"] =
            serde_json::json!({"standing_permitted": true, "per_call_required": false});
        let manifest_a_str = serde_json::to_string(&manifest_a).unwrap();
        let (_, digest_a) = crate::manifest::canonicalize_and_digest(&manifest_a_str).unwrap();
        manifest_a["digest"] = serde_json::json!(digest_a);
        let manifest_a_final = serde_json::to_string_pretty(&manifest_a).unwrap();

        let mut manifest_b: serde_json::Value = serde_json::from_str(manifest_json).unwrap();
        manifest_b["capability_name"] = serde_json::json!("fixture.ping-b");
        manifest_b["provider"]["identity"] = serde_json::json!("provider-b");
        manifest_b["binding"]["server_name"] = serde_json::json!("tethers-stdio-fixture");
        manifest_b["permission_scope"] =
            serde_json::json!({"kind": "path_prefix", "allowed_prefixes": ["member/"]});
        manifest_b["confirmation_policy"] =
            serde_json::json!({"standing_permitted": true, "per_call_required": false});
        let manifest_b_str = serde_json::to_string(&manifest_b).unwrap();
        let (_, digest_b) = crate::manifest::canonicalize_and_digest(&manifest_b_str).unwrap();
        manifest_b["digest"] = serde_json::json!(digest_b);
        let manifest_b_final = serde_json::to_string_pretty(&manifest_b).unwrap();

        std::fs::write(dir.join("manifests/fixture-ping-a.json"), &manifest_a_final).unwrap();
        std::fs::write(dir.join("manifests/fixture-ping-b.json"), &manifest_b_final).unwrap();

        let barrier_script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("scripts")
            .join("tethers-stdio-fixture.ps1");
        let barrier_str = barrier_dir.to_str().unwrap().to_owned();

        let config = json!({
            "format_version": "0.1",
            "tether_set": {
                "id": "test.together",
                "version": "1",
                "tethers": [{
                    "id": "together-test",
                    "version": "1",
                    "source_path": "tethers/together-test.tether"
                }],
                "capability_requirements": [
                    {"name": "fixture.ping-a", "version": 1, "reason": "concurrency observability"},
                    {"name": "fixture.ping-b", "version": 1, "reason": "concurrency observability"}
                ]
            },
            "providers": [
                {
                    "id": "provider-a",
                    "display_name": "Provider A",
                    "transport": {
                        "kind": "stdio",
                        "command": "pwsh.exe",
                        "args": [
                            "-NoProfile", "-ExecutionPolicy", "Bypass", "-File",
                            barrier_script.to_str().unwrap(),
                            "-Mode", "c2-overlap-barrier",
                            "-BarrierDirectory", &barrier_str
                        ],
                        "protocol_version": "2025-11-25"
                    },
                    "capabilities": [{
                        "name": "fixture.ping-a",
                        "version": 1,
                        "manifest_path": "manifests/fixture-ping-a.json",
                        "pinned_digest": &digest_a,
                        "scope_binding": {"kind": "path_prefix", "argument_json_pointer": "/message"}
                    }]
                },
                {
                    "id": "provider-b",
                    "display_name": "Provider B",
                    "transport": {
                        "kind": "stdio",
                        "command": "pwsh.exe",
                        "args": [
                            "-NoProfile", "-ExecutionPolicy", "Bypass", "-File",
                            barrier_script.to_str().unwrap(),
                            "-Mode", "c2-overlap-barrier",
                            "-BarrierDirectory", &barrier_str
                        ],
                        "protocol_version": "2025-11-25"
                    },
                    "capabilities": [{
                        "name": "fixture.ping-b",
                        "version": 1,
                        "manifest_path": "manifests/fixture-ping-b.json",
                        "pinned_digest": &digest_b,
                        "scope_binding": {"kind": "path_prefix", "argument_json_pointer": "/message"}
                    }]
                }
            ],
            "policy": {
                "default": "deny",
                "rules": [
                    {"name": "fixture.ping-a", "version": 1, "decision": "allow"},
                    {"name": "fixture.ping-b", "version": 1, "decision": "allow"}
                ]
            }
        });

        let config_path = dir.join("tethers-config.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&config).unwrap()).unwrap();

        let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
        let prepared = prepare_runtime(&loaded).unwrap();
        (prepared, dir, digest_a, digest_b)
    }

    /// Establish retained sessions with the given providers.  The barrier
    /// fixture accepts `initialize` and `tools/list` without blocking; only
    /// `tools/call` is gated.
    fn c2a3a_establish_sessions(
        providers: &[PreparedProvider],
    ) -> HashMap<String, RetainedProviderSession> {
        let mut sessions = HashMap::new();
        for provider in providers {
            let manifest = provider.capabilities[0].verified_manifest.manifest();
            let session = RetainedProviderSession::establish(SocketEstablishment {
                command: &provider.stdio_config.command,
                args: &provider.stdio_config.args,
                working_directory: &provider.working_directory,
                protocol_version: &provider.stdio_config.protocol_version,
                server_name: &manifest.binding.server_name,
                identity: &provider.identity,
            })
            .expect("barrier provider session establishment");
            sessions.insert(provider.identity.clone(), session);
        }
        sessions
    }

    /// Build a two-member Together group actions and groups array.
    fn c2a3a_actions(digest_a: &str, digest_b: &str) -> (Vec<Value>, Vec<Value>) {
        let actions = vec![
            json!({
                "action_id": "member-a",
                "idempotency_key": "eval-obs/member-a",
                "capability": "fixture.ping-a",
                "capability_version": "1.0.0",
                "bridge_capability_version": 1,
                "bridge_provider_identity": "provider-a",
                "manifest_digest": digest_a,
                "arguments": {"message": "member/a"},
            }),
            json!({
                "action_id": "member-b",
                "idempotency_key": "eval-obs/member-b",
                "capability": "fixture.ping-b",
                "capability_version": "1.0.0",
                "bridge_capability_version": 1,
                "bridge_provider_identity": "provider-b",
                "manifest_digest": digest_b,
                "arguments": {"message": "member/b"},
            }),
        ];
        let groups = vec![json!({
            "group_id": "together-1",
            "member_action_ids": ["member-a", "member-b"],
        })];
        (actions, groups)
    }

    /// Build a matched planner response with groups.
    fn c2a3a_matched_response(
        evaluation_id: &str,
        actions: Vec<Value>,
        groups: Vec<Value>,
    ) -> Value {
        json!({
            "status": "matched",
            "evaluation_id": evaluation_id,
            "plan": {
                "id": "plan-c2a3a",
                "actions": actions,
                "groups": groups,
            },
            "trail": [],
        })
    }

    /// Parse a JSONL Trail file and return action_ids of OutcomeEntry
    /// records in physical append order.
    fn trail_outcome_action_ids(trail_path: &Path) -> Vec<String> {
        let content = match std::fs::read_to_string(trail_path) {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };
        content
            .lines()
            .filter_map(|line| {
                let v: Value = serde_json::from_str(line).ok()?;
                if v.get("execution_id").is_some() && v.get("status").is_some() {
                    Some(v["action_id"].as_str()?.to_owned())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Parse a JSONL Trail file and return entry kinds in physical append
    /// order: "outcome", "group_join", or "other".
    fn trail_entry_kinds(trail_path: &Path) -> Vec<String> {
        let content = match std::fs::read_to_string(trail_path) {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };
        content
            .lines()
            .filter_map(|line| {
                let v: Value = serde_json::from_str(line).ok()?;
                if v.get("execution_id").is_some() && v.get("status").is_some() {
                    Some("outcome".to_owned())
                } else if v.get("group_id").is_some() && v.get("joined").is_some() {
                    Some("group_join".to_owned())
                } else {
                    Some("other".to_owned())
                }
            })
            .collect()
    }

    /// Count barrier entered files: how many providers have reached tools/call.
    fn barrier_entered_count(barrier_dir: &Path) -> usize {
        std::fs::read_dir(barrier_dir)
            .unwrap_or_else(|_| std::fs::read_dir(".").unwrap())
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name().to_string_lossy().starts_with("entered-"))
            .count()
    }

    /// Check whether a specific member's OutcomeEntry is present in the durable Trail.
    fn trail_has_member_outcome(trail_path: &Path, member: &str) -> bool {
        trail_outcome_action_ids(trail_path)
            .iter()
            .any(|id| id == member)
    }

    /// Poll until a condition becomes true or deadline expires.
    fn poll_until(deadline: std::time::Instant, desc: &str, mut check: impl FnMut() -> bool) {
        while !check() {
            assert!(
                std::time::Instant::now() < deadline,
                "poll timed out: {desc}"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    // ===================================================================
    // C2-A3a Group Test Harness
    //
    // One reusable execute_group_concurrent harness for all group-level
    // observability tests: durability, ordering, GroupJoin, panic.
    // ===================================================================

    struct C2A3aGroupHarness {
        runtime: PreparedRuntime,
        _runtime_dir: PathBuf,
        trail_path: PathBuf,
        replay_dir: PathBuf,
        barrier_dir: PathBuf,
    }

    impl C2A3aGroupHarness {
        fn new(test_name: &str) -> Self {
            let barrier_dir = std::env::temp_dir().join(format!(
                "tethers-c2a3a-{test_name}-{}",
                uuid::Uuid::new_v4()
            ));
            std::fs::create_dir_all(&barrier_dir).unwrap();

            let runtime_dir = std::env::temp_dir().join(format!(
                "tethers-c2a3a-{test_name}-rt-{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            std::fs::create_dir_all(runtime_dir.join("tethers")).unwrap();
            std::fs::create_dir_all(runtime_dir.join("manifests")).unwrap();

            std::fs::write(
                runtime_dir.join("tethers/together-test.tether"),
                "when event.test if true do fixture.ping-a do fixture.ping-b",
            )
            .unwrap();

            let manifest_json =
                include_str!("../../protocol/capability-manifests/fixture-ping.json");
            let make_manifest = |cap_name: &str, provider_id: &str| -> String {
                let mut m: serde_json::Value = serde_json::from_str(manifest_json).unwrap();
                m["capability_name"] = serde_json::json!(cap_name);
                m["provider"]["identity"] = serde_json::json!(provider_id);
                m["binding"]["server_name"] = serde_json::json!("tethers-stdio-fixture");
                m["permission_scope"] =
                    serde_json::json!({"kind": "path_prefix", "allowed_prefixes": ["member/"]});
                m["confirmation_policy"] =
                    serde_json::json!({"standing_permitted": true, "per_call_required": false});
                let s = serde_json::to_string(&m).unwrap();
                let (_, digest) = crate::manifest::canonicalize_and_digest(&s).unwrap();
                m["digest"] = serde_json::json!(digest);
                serde_json::to_string_pretty(&m).unwrap()
            };
            let manifest_a = make_manifest("fixture.ping-a", "provider-a");
            let manifest_b = make_manifest("fixture.ping-b", "provider-b");
            std::fs::write(
                runtime_dir.join("manifests/fixture-ping-a.json"),
                &manifest_a,
            )
            .unwrap();
            std::fs::write(
                runtime_dir.join("manifests/fixture-ping-b.json"),
                &manifest_b,
            )
            .unwrap();

            let (_, digest_a) = crate::manifest::canonicalize_and_digest(&manifest_a).unwrap();
            let (_, digest_b) = crate::manifest::canonicalize_and_digest(&manifest_b).unwrap();

            let barrier_script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join("scripts")
                .join("tethers-stdio-fixture.ps1");
            let barrier_str = barrier_dir.to_str().unwrap().to_owned();

            let config = json!({
                "format_version": "0.1",
                "tether_set": {
                    "id": "test.together",
                    "version": "1",
                    "tethers": [{
                        "id": "together-test",
                        "version": "1",
                        "source_path": "tethers/together-test.tether"
                    }],
                    "capability_requirements": [
                        {"name": "fixture.ping-a", "version": 1, "reason": "concurrency observability"},
                        {"name": "fixture.ping-b", "version": 1, "reason": "concurrency observability"}
                    ]
                },
                "providers": [
                    {
                        "id": "provider-a",
                        "display_name": "Provider A",
                        "transport": {
                            "kind": "stdio",
                            "command": "pwsh.exe",
                            "args": [
                                "-NoProfile", "-ExecutionPolicy", "Bypass", "-File",
                                barrier_script.to_str().unwrap(),
                                "-Mode", "c2-overlap-barrier",
                                "-BarrierDirectory", &barrier_str
                            ],
                            "protocol_version": "2025-11-25"
                        },
                        "capabilities": [{
                            "name": "fixture.ping-a",
                            "version": 1,
                            "manifest_path": "manifests/fixture-ping-a.json",
                            "pinned_digest": &digest_a,
                            "scope_binding": {"kind": "path_prefix", "argument_json_pointer": "/message"}
                        }]
                    },
                    {
                        "id": "provider-b",
                        "display_name": "Provider B",
                        "transport": {
                            "kind": "stdio",
                            "command": "pwsh.exe",
                            "args": [
                                "-NoProfile", "-ExecutionPolicy", "Bypass", "-File",
                                barrier_script.to_str().unwrap(),
                                "-Mode", "c2-overlap-barrier",
                                "-BarrierDirectory", &barrier_str
                            ],
                            "protocol_version": "2025-11-25"
                        },
                        "capabilities": [{
                            "name": "fixture.ping-b",
                            "version": 1,
                            "manifest_path": "manifests/fixture-ping-b.json",
                            "pinned_digest": &digest_b,
                            "scope_binding": {"kind": "path_prefix", "argument_json_pointer": "/message"}
                        }]
                    }
                ],
                "policy": {
                    "default": "deny",
                    "rules": [
                        {"name": "fixture.ping-a", "version": 1, "decision": "allow"},
                        {"name": "fixture.ping-b", "version": 1, "decision": "allow"}
                    ]
                }
            });

            let config_path = runtime_dir.join("tethers-config.json");
            std::fs::write(&config_path, serde_json::to_string_pretty(&config).unwrap()).unwrap();
            let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
            let runtime = prepare_runtime(&loaded).unwrap();

            let trail_path = std::env::temp_dir().join(format!(
                "tethers-c2a3a-{test_name}-trail-{}.jsonl",
                uuid::Uuid::new_v4()
            ));
            let replay_dir = std::env::temp_dir().join(format!(
                "tethers-c2a3a-{test_name}-replay-{}",
                uuid::Uuid::new_v4()
            ));
            std::fs::create_dir_all(&replay_dir).unwrap();

            Self {
                runtime,
                _runtime_dir: runtime_dir,
                trail_path,
                replay_dir,
                barrier_dir,
            }
        }

        /// Run execute_group_concurrent in a spawned thread.
        /// Returns the ExecutionServiceResult.
        fn run_group(&self, eval_id: &str) -> ExecutionServiceResult {
            let providers = self.runtime.providers().to_vec();
            let mut sessions = HashMap::new();
            for provider in &providers {
                let manifest = provider.capabilities[0].verified_manifest.manifest();
                let session = RetainedProviderSession::establish(SocketEstablishment {
                    command: &provider.stdio_config.command,
                    args: &provider.stdio_config.args,
                    working_directory: &provider.working_directory,
                    protocol_version: &provider.stdio_config.protocol_version,
                    server_name: &manifest.binding.server_name,
                    identity: &provider.identity,
                })
                .expect("barrier provider session establishment");
                sessions.insert(provider.identity.clone(), session);
            }

            // Extract manifest digests from the prepared runtime providers.
            let providers = self.runtime.providers();
            let digest_a = providers
                .iter()
                .find(|p| p.identity == "provider-a")
                .unwrap()
                .capabilities[0]
                .verified_manifest
                .verified_digest()
                .to_owned();
            let digest_b = providers
                .iter()
                .find(|p| p.identity == "provider-b")
                .unwrap()
                .capabilities[0]
                .verified_manifest
                .verified_digest()
                .to_owned();

            let actions = vec![
                json!({
                    "action_id": "member-a",
                    "idempotency_key": format!("{eval_id}/member-a"),
                    "capability": "fixture.ping-a",
                    "capability_version": "1.0.0",
                    "bridge_capability_version": 1,
                    "bridge_provider_identity": "provider-a",
                    "manifest_digest": digest_a,
                    "arguments": {"message": "member/a"},
                }),
                json!({
                    "action_id": "member-b",
                    "idempotency_key": format!("{eval_id}/member-b"),
                    "capability": "fixture.ping-b",
                    "capability_version": "1.0.0",
                    "bridge_capability_version": 1,
                    "bridge_provider_identity": "provider-b",
                    "manifest_digest": digest_b,
                    "arguments": {"message": "member/b"},
                }),
            ];
            let groups = vec![json!({
                "group_id": "together-1",
                "member_action_ids": ["member-a", "member-b"],
            })];
            let mut response = json!({
                "status": "matched",
                "evaluation_id": eval_id,
                "plan": { "id": format!("plan-{eval_id}"), "actions": actions, "groups": groups },
                "trail": [],
            });
            let member_actions = response["plan"]["actions"].as_array().unwrap().clone();
            let availability = ProviderAvailability::from_identities(["provider-a", "provider-b"]);
            let mut trail = dispatch::FileTrail::open(&self.trail_path).unwrap();
            let mut approvals = crate::approval::ApprovalStore::default();
            let mut replay_authority =
                crate::replay_runtime::test_support::TestReplayAuthority::default();
            let engine_path = PathBuf::from("unused-engine");
            let service =
                HostExecutionService::new(&self.runtime, &engine_path, &self.trail_path, None);

            let result = execute_group_concurrent(
                "together-1",
                &[0, 1],
                &member_actions,
                &mut response,
                eval_id,
                &mut trail,
                &service,
                &PreparedEvaluationInput {
                    tether_id: "together-test".to_owned(),
                    tether_version: "1".to_owned(),
                    evaluation_id: eval_id.to_owned(),
                    anchor_event: json!({"id": format!("evt-{eval_id}"), "name": "test"}),
                    facts: json!({}),
                },
                &mut sessions,
                &availability,
                &mut approvals,
                &mut replay_authority,
            );
            eprintln!("execute_group_concurrent returned: {result:?}");
            result
        }

        /// Wait until the barrier directory has at least `count` entered-* files.
        fn wait_barrier_entries(&self, count: usize) {
            let deadline = std::time::Instant::now() + Duration::from_secs(15);
            while barrier_entered_count(&self.barrier_dir) < count {
                assert!(
                    std::time::Instant::now() < deadline,
                    "only {} of {count} providers entered tools/call",
                    barrier_entered_count(&self.barrier_dir)
                );
                std::thread::sleep(Duration::from_millis(10));
            }
        }

        /// Release a specific member via per-member release file.
        fn release_member(&self, member: &str) {
            std::fs::write(
                self.barrier_dir.join(format!("release-member-{member}")),
                "release",
            )
            .unwrap();
        }

        /// Read physical OutcomeEntry action_ids from the durable Trail.
        fn outcome_ids(&self) -> Vec<String> {
            trail_outcome_action_ids(&self.trail_path)
        }

        /// Read entry kinds from the durable Trail.
        fn entry_kinds(&self) -> Vec<String> {
            trail_entry_kinds(&self.trail_path)
        }

        /// Check if a member's OutcomeEntry exists in the Trail.
        fn has_member_outcome(&self, member: &str) -> bool {
            trail_has_member_outcome(&self.trail_path, member)
        }

        /// Read the raw Trail content.
        fn trail_content(&self) -> String {
            std::fs::read_to_string(&self.trail_path).unwrap_or_default()
        }
    }

    impl Drop for C2A3aGroupHarness {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.barrier_dir);
            let _ = std::fs::remove_dir_all(&self._runtime_dir);
            let _ = std::fs::remove_dir_all(&self.replay_dir);
            let _ = std::fs::remove_file(&self.trail_path);
        }
    }

    // ===================================================================
    // TEST 1 — Prompt Stage C durability while sibling blocked
    // ===================================================================

    #[test]
    fn c2a3a_stage_c_durability_while_sibling_blocked() {
        let h = C2A3aGroupHarness::new("durability");

        std::thread::scope(|s| {
            let handle = s.spawn(|| h.run_group("eval-durability-1"));

            // Both must enter real tools/call.
            h.wait_barrier_entries(2);

            // Release B only.  A remains blocked.
            h.release_member("b");

            // Poll until B's OutcomeEntry is durable in the Trail.
            let deadline = std::time::Instant::now() + Duration::from_secs(15);
            while !h.has_member_outcome("member-b") {
                assert!(
                    std::time::Instant::now() < deadline,
                    "B outcome never appeared in Trail"
                );
                std::thread::sleep(Duration::from_millis(10));
            }

            // INTERMEDIATE ASSERTIONS: B present, A absent, GroupJoin absent.
            assert!(
                h.has_member_outcome("member-b"),
                "B OutcomeEntry must be present before A release"
            );
            assert!(
                !h.has_member_outcome("member-a"),
                "A OutcomeEntry must NOT be present while A is still blocked"
            );
            assert!(
                !h.entry_kinds().iter().any(|k| k == "group_join"),
                "GroupJoinEntry must NOT be present while A is still blocked"
            );

            // Now release A.
            h.release_member("a");

            let result = handle.join().expect("concurrent group must not panic");
            assert!(
                matches!(result, ExecutionServiceResult::Completed { .. }),
                "expected Completed, got: {result:?}"
            );
        });

        // FINAL: both outcomes present, GroupJoin last.
        let ids = h.outcome_ids();
        assert_eq!(ids.len(), 2, "need exactly 2 outcomes: {ids:?}");
        assert!(ids.contains(&"member-a".to_owned()));
        assert!(ids.contains(&"member-b".to_owned()));
        assert_eq!(h.entry_kinds().last(), Some(&"group_join".to_owned()));
    }

    // ===================================================================
    // TEST 2 — Physical Trail order: B before A
    // ===================================================================

    #[test]
    fn c2a3a_trail_physical_order_b_before_a() {
        let h = C2A3aGroupHarness::new("order-ba");

        std::thread::scope(|s| {
            let handle = s.spawn(|| h.run_group("eval-order-ba"));

            h.wait_barrier_entries(2);

            // Release B first, keep A blocked.
            h.release_member("b");

            // Wait until B is durable.
            let deadline = std::time::Instant::now() + Duration::from_secs(15);
            while !h.has_member_outcome("member-b") {
                assert!(
                    std::time::Instant::now() < deadline,
                    "B outcome never appeared"
                );
                std::thread::sleep(Duration::from_millis(10));
            }

            // Release A.
            h.release_member("a");

            let result = handle.join().expect("group must not panic");
            assert!(matches!(result, ExecutionServiceResult::Completed { .. }));
        });

        // Physical append order must be exactly [B, A].
        assert_eq!(
            h.outcome_ids(),
            vec!["member-b".to_owned(), "member-a".to_owned()],
            "physical append order must be B then A"
        );

        // Semantic positions: A=0, B=1.
        let entries: Vec<Value> = h
            .trail_content()
            .lines()
            .filter_map(|l| serde_json::from_str(l).ok())
            .collect();
        for entry in &entries {
            if let Some(pos) = entry.get("semantic_position") {
                if pos.get("phase").and_then(Value::as_str) == Some("member") {
                    let ord = pos.get("member_ordinal").and_then(Value::as_u64);
                    let aord = pos.get("action_ordinal").and_then(Value::as_u64);
                    match entry.get("action_id").and_then(Value::as_str) {
                        Some("member-a") => {
                            assert_eq!(ord, Some(0));
                            assert_eq!(aord, Some(0));
                        }
                        Some("member-b") => {
                            assert_eq!(ord, Some(1));
                            assert_eq!(aord, Some(1));
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // ===================================================================
    // TEST 3 — Physical Trail order: A before B
    // ===================================================================

    #[test]
    fn c2a3a_trail_physical_order_a_before_b() {
        let h = C2A3aGroupHarness::new("order-ab");

        std::thread::scope(|s| {
            let handle = s.spawn(|| h.run_group("eval-order-ab"));

            h.wait_barrier_entries(2);

            // Release A first, keep B blocked.
            h.release_member("a");

            let deadline = std::time::Instant::now() + Duration::from_secs(15);
            while !h.has_member_outcome("member-a") {
                assert!(
                    std::time::Instant::now() < deadline,
                    "A outcome never appeared"
                );
                std::thread::sleep(Duration::from_millis(10));
            }

            // Release B.
            h.release_member("b");

            let result = handle.join().expect("group must not panic");
            assert!(matches!(result, ExecutionServiceResult::Completed { .. }));
        });

        // Physical append order must be exactly [A, B].
        assert_eq!(
            h.outcome_ids(),
            vec!["member-a".to_owned(), "member-b".to_owned()],
            "physical append order must be A then B"
        );

        // Semantic positions: A=0, B=1.
        let entries: Vec<Value> = h
            .trail_content()
            .lines()
            .filter_map(|l| serde_json::from_str(l).ok())
            .collect();
        for entry in &entries {
            if let Some(pos) = entry.get("semantic_position") {
                if pos.get("phase").and_then(Value::as_str) == Some("member") {
                    let ord = pos.get("member_ordinal").and_then(Value::as_u64);
                    let aord = pos.get("action_ordinal").and_then(Value::as_u64);
                    match entry.get("action_id").and_then(Value::as_str) {
                        Some("member-a") => {
                            assert_eq!(ord, Some(0));
                            assert_eq!(aord, Some(0));
                        }
                        Some("member-b") => {
                            assert_eq!(ord, Some(1));
                            assert_eq!(aord, Some(1));
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // ===================================================================
    // TEST 4 — GroupJoin after all terminals
    // ===================================================================

    #[test]
    fn c2a3a_group_join_after_all_terminals() {
        let h = C2A3aGroupHarness::new("join");

        std::thread::scope(|s| {
            let handle = s.spawn(|| h.run_group("eval-join-1"));

            h.wait_barrier_entries(2);

            // Release B only.
            h.release_member("b");

            // Wait for B outcome.
            let deadline = std::time::Instant::now() + Duration::from_secs(15);
            while !h.has_member_outcome("member-b") {
                assert!(std::time::Instant::now() < deadline);
                std::thread::sleep(Duration::from_millis(10));
            }

            // INTERMEDIATE: no GroupJoin while A is blocked.
            assert!(
                !h.entry_kinds().iter().any(|k| k == "group_join"),
                "GroupJoin must NOT exist while A is still blocked"
            );

            // Release A.
            h.release_member("a");

            let result = handle.join().expect("group must not panic");
            assert!(matches!(result, ExecutionServiceResult::Completed { .. }));
        });

        // FINAL: GroupJoin is last.
        let kinds = h.entry_kinds();
        assert_eq!(kinds.iter().filter(|k| k.as_str() == "outcome").count(), 2);
        assert_eq!(
            kinds.iter().filter(|k| k.as_str() == "group_join").count(),
            1
        );
        assert_eq!(kinds.last(), Some(&"group_join".to_owned()));
        let join_pos = kinds.iter().position(|k| k == "group_join").unwrap();
        assert!(
            join_pos >= 2,
            "GroupJoin at position {join_pos} must be after both outcomes"
        );
    }

    // ===================================================================
    // TEST 5 — Worker panic yields uncertain non-success join
    // ===================================================================

    struct PanicGuard;
    impl PanicGuard {
        fn target(action_index: usize) -> Self {
            INJECT_WORKER_PANIC_ACTION_INDEX
                .store(action_index, std::sync::atomic::Ordering::SeqCst);
            Self
        }
    }
    impl Drop for PanicGuard {
        fn drop(&mut self) {
            INJECT_WORKER_PANIC_ACTION_INDEX.store(usize::MAX, std::sync::atomic::Ordering::SeqCst);
        }
    }

    #[test]
    fn c2a3a_worker_panic_yields_uncertain_non_success_join() {
        let h = C2A3aGroupHarness::new("panic");

        // Target worker action_index=1 (member-b) for panic injection.
        // PanicGuard resets to usize::MAX on drop (even if test panics).
        let _guard = PanicGuard::target(1);

        std::thread::scope(|s| {
            let handle = s.spawn(|| h.run_group("eval-panic-1"));

            // Wait for A to enter the barrier (B panics before entering).
            h.wait_barrier_entries(1);

            // Release A.
            h.release_member("a");

            let result = handle.join().expect("coordinator must not hang");
            assert!(
                !matches!(result, ExecutionServiceResult::Completed { .. }),
                "panic must prevent Completed, got: {result:?}"
            );
        });

        // Verify Trail: at least one outcome, GroupJoin joined=false.
        let kinds = h.entry_kinds();
        let outcome_count = kinds.iter().filter(|k| k.as_str() == "outcome").count();
        let join_count = kinds.iter().filter(|k| k.as_str() == "group_join").count();
        assert!(
            outcome_count >= 1,
            "at least one outcome expected, got {outcome_count}"
        );
        assert_eq!(join_count, 1, "exactly one GroupJoin expected");

        for line in h.trail_content().lines() {
            if let Ok(v) = serde_json::from_str::<Value>(line) {
                if v.get("group_id").is_some() && v.get("joined").is_some() {
                    assert_eq!(
                        v["joined"].as_bool(),
                        Some(false),
                        "GroupJoin must be non-success when a worker panics"
                    );
                }
            }
        }
    }

    // ===================================================================
    // Low-level direct-worker overlap controls
    // (These test provider invocation, not coordinator behaviour.)
    // ===================================================================

    fn c2a3a_member_provider(barrier_dir: &Path, identity: &str) -> PreparedProvider {
        let mut provider = catalogue_test_provider("c2-overlap-barrier");
        provider.identity = identity.to_owned();
        provider.stdio_config.provider_config.identity = identity.to_owned();
        provider.stdio_config.args.extend([
            "-BarrierDirectory".to_owned(),
            barrier_dir.to_string_lossy().into_owned(),
        ]);
        provider
    }

    // ===================================================================
    // Observing Replay Authority (test-only)
    //
    // A coordinator-owned replay authority that:
    // - selectively recovers specific members as terminal (blocked), and
    // - records a Send+Sync trace of G0 (intent) / G1 (armed) / G2 (terminal)
    //   events observable from the test coordinator thread.
    //
    // Production ReplayAdmission ownership and !Send boundaries are unchanged;
    // this is a test seam only.
    // ===================================================================

    #[derive(Clone)]
    struct ReplayTrace {
        events: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
    }

    impl ReplayTrace {
        fn new() -> Self {
            Self {
                events: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            }
        }

        fn record(&self, event: String) {
            self.events.lock().unwrap().push(event);
        }

        fn snapshot(&self) -> Vec<String> {
            self.events.lock().unwrap().clone()
        }

        fn has(&self, needle: &str) -> bool {
            self.events.lock().unwrap().iter().any(|e| e == needle)
        }
    }

    struct ObservingReplayAuthority {
        blocked: std::collections::HashMap<String, ReplayState>,
        trace: ReplayTrace,
        fail_armed_actions: std::collections::HashSet<String>,
        fail_terminal_actions: std::collections::HashSet<String>,
    }

    impl ObservingReplayAuthority {
        fn new(blocked: &[(&str, ReplayState)]) -> Self {
            Self::with_trace(blocked, ReplayTrace::new())
        }

        fn with_trace(blocked: &[(&str, ReplayState)], trace: ReplayTrace) -> Self {
            Self {
                blocked: blocked
                    .iter()
                    .map(|(id, state)| (id.to_string(), *state))
                    .collect(),
                trace,
                fail_armed_actions: std::collections::HashSet::new(),
                fail_terminal_actions: std::collections::HashSet::new(),
            }
        }

        fn with_fail_points(
            trace: ReplayTrace,
            fail_armed_actions: &[&str],
            fail_terminal_actions: &[&str],
        ) -> Self {
            Self {
                blocked: std::collections::HashMap::new(),
                trace,
                fail_armed_actions: fail_armed_actions.iter().map(|s| s.to_string()).collect(),
                fail_terminal_actions: fail_terminal_actions
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
            }
        }
    }

    impl crate::replay_runtime::ReplayAuthority for ObservingReplayAuthority {
        fn admit(
            &self,
            _logical_key: &crate::replay::LogicalExecutionKey,
            binding: &crate::replay::ExecutionBinding,
        ) -> Result<Box<dyn crate::replay_runtime::ReplayAdmissionGuard>, crate::replay::ReplayError>
        {
            let action_id = binding.action_id.clone();
            let (fresh, state) = match self.blocked.get(&action_id) {
                Some(state) => {
                    self.trace
                        .record(format!("admit:{action_id}:blocked:{state:?}"));
                    (false, *state)
                }
                None => {
                    self.trace.record(format!("admit:{action_id}:fresh"));
                    (true, ReplayState::ClaimedNoState)
                }
            };
            let fail_armed = self.fail_armed_actions.contains(&action_id);
            let fail_terminal = self.fail_terminal_actions.contains(&action_id);
            Ok(Box::new(ObservingAdmission {
                action_id,
                fresh,
                state,
                trace: self.trace.clone(),
                fail_armed,
                fail_terminal,
            }))
        }
    }

    struct ObservingAdmission {
        action_id: String,
        fresh: bool,
        state: ReplayState,
        trace: ReplayTrace,
        fail_armed: bool,
        fail_terminal: bool,
    }

    impl crate::replay_runtime::ReplayAdmissionGuard for ObservingAdmission {
        fn execution_id(&self) -> &str {
            crate::replay_runtime::test_support::TEST_EXECUTION_ID
        }
        fn state(&self) -> ReplayState {
            self.state
        }
        fn is_fresh(&self) -> bool {
            self.fresh
        }
        fn publish_intent(&mut self) -> Result<(), crate::replay::ReplayError> {
            self.trace.record(format!("g0:{}", self.action_id));
            Ok(())
        }
        fn publish_armed(&mut self) -> Result<(), crate::replay::ReplayError> {
            if self.fail_armed {
                self.trace.record(format!("g1_fail:{}", self.action_id));
                return Err(crate::replay::ReplayError::PersistenceUnavailable);
            }
            self.trace.record(format!("g1:{}", self.action_id));
            Ok(())
        }
        fn publish_terminal(
            &mut self,
            _state: ReplayState,
            _durable_outcome_digest: String,
        ) -> Result<(), crate::replay::ReplayError> {
            if self.fail_terminal {
                self.trace.record(format!("g2_fail:{}", self.action_id));
                return Err(crate::replay::ReplayError::PersistenceUnavailable);
            }
            self.trace.record(format!("g2:{}", self.action_id));
            Ok(())
        }
    }

    // ===================================================================
    // C2-A3a Terminal Semantic Matrix Harness
    //
    // Flexible builder for tests that need custom policy, replay state,
    // or timeout configuration.  Supports:
    // - per-capability policy rules (Deny / Ask / Allow)
    // - custom replay authority state (fresh / recovered)
    // - per-capability timeout override
    // - provider removal (for Unavailable tests)
    // ===================================================================

    struct C2A3aTerminalHarness {
        runtime: PreparedRuntime,
        _runtime_dir: PathBuf,
        trail_path: PathBuf,
        replay_dir: PathBuf,
        barrier_dir: PathBuf,
        provider_a_unavailable: bool,
    }

    struct TerminalHarnessBuilder {
        test_name: String,
        policy_a: PolicyDecision,
        policy_b: PolicyDecision,
        timeout_a_ms: Option<u64>,
        timeout_b_ms: Option<u64>,
        provider_a_unavailable: bool,
        peer_count: usize,
        outcome_a: OutcomeMode,
        outcome_b: OutcomeMode,
    }

    #[derive(Clone, Copy)]
    enum PolicyDecision {
        Allow,
        Deny,
        Ask,
    }

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum OutcomeMode {
        Success,
        Failed,
        Uncertain,
    }

    impl TerminalHarnessBuilder {
        fn new(test_name: &str) -> Self {
            Self {
                test_name: test_name.to_owned(),
                policy_a: PolicyDecision::Allow,
                policy_b: PolicyDecision::Allow,
                timeout_a_ms: None,
                timeout_b_ms: None,
                provider_a_unavailable: false,
                peer_count: 2,
                outcome_a: OutcomeMode::Success,
                outcome_b: OutcomeMode::Success,
            }
        }

        fn policy_a(mut self, p: PolicyDecision) -> Self {
            self.policy_a = p;
            self
        }

        fn policy_b(mut self, p: PolicyDecision) -> Self {
            self.policy_b = p;
            self
        }

        fn timeout_a_ms(mut self, ms: u64) -> Self {
            self.timeout_a_ms = Some(ms);
            self
        }

        fn timeout_b_ms(mut self, ms: u64) -> Self {
            self.timeout_b_ms = Some(ms);
            self
        }

        /// Keep provider-a fully configured but exclude it from the host's
        /// availability snapshot so the semantic member becomes exactly
        /// `Unavailable` without being removed from the Runtime Plan.
        fn provider_a_unavailable(mut self) -> Self {
            self.provider_a_unavailable = true;
            self
        }

        /// Number of providers that must reach tools/call before any proceeds.
        fn peer_count(mut self, count: usize) -> Self {
            self.peer_count = count;
            self
        }

        fn outcome_a(mut self, mode: OutcomeMode) -> Self {
            self.outcome_a = mode;
            self
        }

        fn outcome_b(mut self, mode: OutcomeMode) -> Self {
            self.outcome_b = mode;
            self
        }

        fn build(self) -> C2A3aTerminalHarness {
            let barrier_dir = std::env::temp_dir().join(format!(
                "tethers-c2a3a-terminal-{}-{}",
                self.test_name,
                uuid::Uuid::new_v4()
            ));
            std::fs::create_dir_all(&barrier_dir).unwrap();

            let runtime_dir = std::env::temp_dir().join(format!(
                "tethers-c2a3a-terminal-{}-rt-{}",
                self.test_name,
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            std::fs::create_dir_all(runtime_dir.join("tethers")).unwrap();
            std::fs::create_dir_all(runtime_dir.join("manifests")).unwrap();

            std::fs::write(
                runtime_dir.join("tethers/together-test.tether"),
                "when event.test if true do fixture.ping-a do fixture.ping-b",
            )
            .unwrap();

            let manifest_json =
                include_str!("../../protocol/capability-manifests/fixture-ping.json");
            let make_manifest =
                |cap_name: &str, provider_id: &str, timeout_ms: Option<u64>| -> String {
                    let mut m: serde_json::Value = serde_json::from_str(manifest_json).unwrap();
                    m["capability_name"] = serde_json::json!(cap_name);
                    m["provider"]["identity"] = serde_json::json!(provider_id);
                    m["binding"]["server_name"] = serde_json::json!("tethers-stdio-fixture");
                    m["permission_scope"] =
                        serde_json::json!({"kind": "path_prefix", "allowed_prefixes": ["member/"]});
                    m["confirmation_policy"] =
                        serde_json::json!({"standing_permitted": true, "per_call_required": false});
                    if let Some(ms) = timeout_ms {
                        m["timeout_ms"] = serde_json::json!(ms);
                    }
                    let s = serde_json::to_string(&m).unwrap();
                    let (_, digest) = crate::manifest::canonicalize_and_digest(&s).unwrap();
                    m["digest"] = serde_json::json!(digest);
                    serde_json::to_string_pretty(&m).unwrap()
                };
            let manifest_a = make_manifest("fixture.ping-a", "provider-a", self.timeout_a_ms);
            let manifest_b = make_manifest("fixture.ping-b", "provider-b", self.timeout_b_ms);
            std::fs::write(
                runtime_dir.join("manifests/fixture-ping-a.json"),
                &manifest_a,
            )
            .unwrap();
            std::fs::write(
                runtime_dir.join("manifests/fixture-ping-b.json"),
                &manifest_b,
            )
            .unwrap();

            let (_, digest_a) = crate::manifest::canonicalize_and_digest(&manifest_a).unwrap();
            let (_, digest_b) = crate::manifest::canonicalize_and_digest(&manifest_b).unwrap();

            let barrier_script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join("scripts")
                .join("tethers-stdio-fixture.ps1");
            let barrier_str = barrier_dir.to_str().unwrap().to_owned();

            let policy_rules = |cap: &str, decision: PolicyDecision| -> serde_json::Value {
                let d = match decision {
                    PolicyDecision::Allow => "allow",
                    PolicyDecision::Deny => "deny",
                    PolicyDecision::Ask => "ask",
                };
                json!({"name": cap, "version": 1, "decision": d})
            };

            let mut providers = vec![
                json!({
                    "id": "provider-a",
                    "display_name": "Provider A",
                    "transport": {
                        "kind": "stdio",
                        "command": "pwsh.exe",
                        "args": [
                            "-NoProfile", "-ExecutionPolicy", "Bypass", "-File",
                            barrier_script.to_str().unwrap(),
                            "-Mode", "c2-overlap-barrier",
                            "-BarrierDirectory", &barrier_str
                        ],
                        "protocol_version": "2025-11-25"
                    },
                    "capabilities": [{
                        "name": "fixture.ping-a",
                        "version": 1,
                        "manifest_path": "manifests/fixture-ping-a.json",
                        "pinned_digest": &digest_a,
                        "scope_binding": {"kind": "path_prefix", "argument_json_pointer": "/message"}
                    }]
                }),
                json!({
                    "id": "provider-b",
                    "display_name": "Provider B",
                    "transport": {
                        "kind": "stdio",
                        "command": "pwsh.exe",
                        "args": [
                            "-NoProfile", "-ExecutionPolicy", "Bypass", "-File",
                            barrier_script.to_str().unwrap(),
                            "-Mode", "c2-overlap-barrier",
                            "-BarrierDirectory", &barrier_str
                        ],
                        "protocol_version": "2025-11-25"
                    },
                    "capabilities": [{
                        "name": "fixture.ping-b",
                        "version": 1,
                        "manifest_path": "manifests/fixture-ping-b.json",
                        "pinned_digest": &digest_b,
                        "scope_binding": {"kind": "path_prefix", "argument_json_pointer": "/message"}
                    }]
                }),
            ];

            // Both providers remain configured.  `provider_a_unavailable`
            // only affects the runtime availability snapshot, so member-a's
            // semantic Action always remains in the plan.
            let capability_requirements = vec![
                json!({"name": "fixture.ping-a", "version": 1, "reason": "concurrency observability"}),
                json!({"name": "fixture.ping-b", "version": 1, "reason": "concurrency observability"}),
            ];
            let policy_rules_list = vec![
                policy_rules("fixture.ping-a", self.policy_a),
                policy_rules("fixture.ping-b", self.policy_b),
            ];

            let config = json!({
                "format_version": "0.1",
                "tether_set": {
                    "id": "test.together",
                    "version": "1",
                    "tethers": [{
                        "id": "together-test",
                        "version": "1",
                        "source_path": "tethers/together-test.tether"
                    }],
                    "capability_requirements": capability_requirements
                },
                "providers": providers,
                "policy": {
                    "default": "deny",
                    "rules": policy_rules_list
                }
            });

            let config_path = runtime_dir.join("tethers-config.json");
            std::fs::write(&config_path, serde_json::to_string_pretty(&config).unwrap()).unwrap();
            let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
            let runtime = prepare_runtime(&loaded).unwrap();

            // Write barrier control files: peer count and per-member outcome.
            std::fs::write(barrier_dir.join("peer-count"), self.peer_count.to_string()).unwrap();
            let outcome_file = |tag: &str, mode: OutcomeMode| {
                let text = match mode {
                    OutcomeMode::Success => "success",
                    OutcomeMode::Failed => "failed",
                    OutcomeMode::Uncertain => "uncertain",
                };
                std::fs::write(barrier_dir.join(format!("outcome-member-{tag}")), text).unwrap();
            };
            outcome_file("a", self.outcome_a);
            outcome_file("b", self.outcome_b);

            let trail_path = std::env::temp_dir().join(format!(
                "tethers-c2a3a-terminal-{}-trail-{}.jsonl",
                self.test_name,
                uuid::Uuid::new_v4()
            ));
            let replay_dir = std::env::temp_dir().join(format!(
                "tethers-c2a3a-terminal-{}-replay-{}",
                self.test_name,
                uuid::Uuid::new_v4()
            ));
            std::fs::create_dir_all(&replay_dir).unwrap();

            C2A3aTerminalHarness {
                runtime,
                _runtime_dir: runtime_dir,
                trail_path,
                replay_dir,
                barrier_dir,
                provider_a_unavailable: self.provider_a_unavailable,
            }
        }
    }

    impl C2A3aTerminalHarness {
        fn run_group_with_boxed_replay(
            &self,
            eval_id: &str,
            replay_authority: Box<dyn crate::replay_runtime::ReplayAuthority>,
        ) -> ExecutionServiceResult {
            let providers = self.runtime.providers().to_vec();
            let mut sessions = HashMap::new();
            for provider in &providers {
                let manifest = provider.capabilities[0].verified_manifest.manifest();
                let session = RetainedProviderSession::establish(SocketEstablishment {
                    command: &provider.stdio_config.command,
                    args: &provider.stdio_config.args,
                    working_directory: &provider.working_directory,
                    protocol_version: &provider.stdio_config.protocol_version,
                    server_name: &manifest.binding.server_name,
                    identity: &provider.identity,
                })
                .expect("terminal provider session establishment");
                sessions.insert(provider.identity.clone(), session);
            }

            let providers = self.runtime.providers();
            let digest_a = providers
                .iter()
                .find(|p| p.identity == "provider-a")
                .map(|p| {
                    p.capabilities[0]
                        .verified_manifest
                        .verified_digest()
                        .to_owned()
                });
            let digest_b = providers
                .iter()
                .find(|p| p.identity == "provider-b")
                .map(|p| {
                    p.capabilities[0]
                        .verified_manifest
                        .verified_digest()
                        .to_owned()
                });

            let mut actions = Vec::new();
            if let Some(ref da) = digest_a {
                actions.push(json!({
                    "action_id": "member-a",
                    "idempotency_key": format!("{eval_id}/member-a"),
                    "capability": "fixture.ping-a",
                    "capability_version": "1.0.0",
                    "bridge_capability_version": 1,
                    "bridge_provider_identity": "provider-a",
                    "manifest_digest": da,
                    "arguments": {"message": "member/a"},
                }));
            }
            if let Some(ref db) = digest_b {
                actions.push(json!({
                    "action_id": "member-b",
                    "idempotency_key": format!("{eval_id}/member-b"),
                    "capability": "fixture.ping-b",
                    "capability_version": "1.0.0",
                    "bridge_capability_version": 1,
                    "bridge_provider_identity": "provider-b",
                    "manifest_digest": db,
                    "arguments": {"message": "member/b"},
                }));
            }

            let member_action_ids: Vec<String> = actions
                .iter()
                .filter_map(|a| a.get("action_id").and_then(Value::as_str).map(String::from))
                .collect();
            let member_indexes: Vec<usize> = (0..actions.len()).collect();

            let groups = vec![json!({
                "group_id": "together-1",
                "member_action_ids": member_action_ids,
            })];
            let mut response = json!({
                "status": "matched",
                "evaluation_id": eval_id,
                "plan": { "id": format!("plan-{eval_id}"), "actions": actions, "groups": groups },
                "trail": [],
            });
            let member_actions = response["plan"]["actions"].as_array().unwrap().clone();
            let availability = self.availability();
            let mut trail = dispatch::FileTrail::open(&self.trail_path).unwrap();
            let mut approvals = crate::approval::ApprovalStore::default();
            let mut replay_authority = replay_authority;
            let engine_path = PathBuf::from("unused-engine");
            let service =
                HostExecutionService::new(&self.runtime, &engine_path, &self.trail_path, None);

            execute_group_concurrent(
                "together-1",
                &member_indexes,
                &member_actions,
                &mut response,
                eval_id,
                &mut trail,
                &service,
                &PreparedEvaluationInput {
                    tether_id: "together-test".to_owned(),
                    tether_version: "1".to_owned(),
                    evaluation_id: eval_id.to_owned(),
                    anchor_event: json!({"id": format!("evt-{eval_id}"), "name": "test"}),
                    facts: json!({}),
                },
                &mut sessions,
                &availability,
                &mut approvals,
                replay_authority.as_mut(),
            )
        }

        /// Build the host availability snapshot for the group, excluding
        /// provider-a when the harness was built with
        /// `provider_a_unavailable`.
        fn availability(&self) -> ProviderAvailability {
            let identities = self
                .runtime
                .providers()
                .iter()
                .map(|p| p.identity.as_str())
                .filter(|id| !(self.provider_a_unavailable && *id == "provider-a"));
            ProviderAvailability::from_identities(identities)
        }

        fn run_group_with_replay(
            &self,
            eval_id: &str,
            replay_config: impl FnOnce() -> crate::replay_runtime::test_support::TestReplayAuthority,
        ) -> ExecutionServiceResult {
            let providers = self.runtime.providers().to_vec();
            let mut sessions = HashMap::new();
            for provider in &providers {
                let manifest = provider.capabilities[0].verified_manifest.manifest();
                let session = RetainedProviderSession::establish(SocketEstablishment {
                    command: &provider.stdio_config.command,
                    args: &provider.stdio_config.args,
                    working_directory: &provider.working_directory,
                    protocol_version: &provider.stdio_config.protocol_version,
                    server_name: &manifest.binding.server_name,
                    identity: &provider.identity,
                })
                .expect("terminal provider session establishment");
                sessions.insert(provider.identity.clone(), session);
            }

            let providers = self.runtime.providers();
            let digest_a = providers
                .iter()
                .find(|p| p.identity == "provider-a")
                .map(|p| {
                    p.capabilities[0]
                        .verified_manifest
                        .verified_digest()
                        .to_owned()
                });
            let digest_b = providers
                .iter()
                .find(|p| p.identity == "provider-b")
                .map(|p| {
                    p.capabilities[0]
                        .verified_manifest
                        .verified_digest()
                        .to_owned()
                });

            let mut actions = Vec::new();
            if let Some(ref da) = digest_a {
                actions.push(json!({
                    "action_id": "member-a",
                    "idempotency_key": format!("{eval_id}/member-a"),
                    "capability": "fixture.ping-a",
                    "capability_version": "1.0.0",
                    "bridge_capability_version": 1,
                    "bridge_provider_identity": "provider-a",
                    "manifest_digest": da,
                    "arguments": {"message": "member/a"},
                }));
            }
            if let Some(ref db) = digest_b {
                actions.push(json!({
                    "action_id": "member-b",
                    "idempotency_key": format!("{eval_id}/member-b"),
                    "capability": "fixture.ping-b",
                    "capability_version": "1.0.0",
                    "bridge_capability_version": 1,
                    "bridge_provider_identity": "provider-b",
                    "manifest_digest": db,
                    "arguments": {"message": "member/b"},
                }));
            }

            let member_action_ids: Vec<String> = actions
                .iter()
                .filter_map(|a| a.get("action_id").and_then(Value::as_str).map(String::from))
                .collect();
            let member_indexes: Vec<usize> = (0..actions.len()).collect();

            let groups = vec![json!({
                "group_id": "together-1",
                "member_action_ids": member_action_ids,
            })];
            let mut response = json!({
                "status": "matched",
                "evaluation_id": eval_id,
                "plan": { "id": format!("plan-{eval_id}"), "actions": actions, "groups": groups },
                "trail": [],
            });
            let member_actions = response["plan"]["actions"].as_array().unwrap().clone();
            let availability = self.availability();
            let mut trail = dispatch::FileTrail::open(&self.trail_path).unwrap();
            let mut approvals = crate::approval::ApprovalStore::default();
            let mut replay_authority = replay_config();
            let engine_path = PathBuf::from("unused-engine");
            let service =
                HostExecutionService::new(&self.runtime, &engine_path, &self.trail_path, None);

            execute_group_concurrent(
                "together-1",
                &member_indexes,
                &member_actions,
                &mut response,
                eval_id,
                &mut trail,
                &service,
                &PreparedEvaluationInput {
                    tether_id: "together-test".to_owned(),
                    tether_version: "1".to_owned(),
                    evaluation_id: eval_id.to_owned(),
                    anchor_event: json!({"id": format!("evt-{eval_id}"), "name": "test"}),
                    facts: json!({}),
                },
                &mut sessions,
                &availability,
                &mut approvals,
                &mut replay_authority,
            )
        }

        fn wait_barrier_entries(&self, count: usize) {
            let deadline = std::time::Instant::now() + Duration::from_secs(15);
            while barrier_entered_count(&self.barrier_dir) < count {
                assert!(
                    std::time::Instant::now() < deadline,
                    "only {} of {count} providers entered tools/call",
                    barrier_entered_count(&self.barrier_dir)
                );
                std::thread::sleep(Duration::from_millis(10));
            }
        }

        fn release_member(&self, member: &str) {
            std::fs::write(
                self.barrier_dir.join(format!("release-member-{member}")),
                "release",
            )
            .unwrap();
        }

        fn outcome_ids(&self) -> Vec<String> {
            trail_outcome_action_ids(&self.trail_path)
        }

        fn entry_kinds(&self) -> Vec<String> {
            trail_entry_kinds(&self.trail_path)
        }

        fn has_member_outcome(&self, member: &str) -> bool {
            trail_has_member_outcome(&self.trail_path, member)
        }

        fn trail_content(&self) -> String {
            std::fs::read_to_string(&self.trail_path).unwrap_or_default()
        }

        fn trail_entries(&self) -> Vec<Value> {
            self.trail_content()
                .lines()
                .filter_map(|l| serde_json::from_str(l).ok())
                .collect()
        }

        fn outcome_entry(&self, action_id: &str) -> Option<Value> {
            self.trail_entries().into_iter().find(|e| {
                e.get("execution_id").is_some()
                    && e.get("status").is_some()
                    && e.get("action_id").and_then(Value::as_str) == Some(action_id)
            })
        }

        fn group_join_entry(&self) -> Option<Value> {
            self.trail_entries()
                .into_iter()
                .find(|e| e.get("group_id").is_some() && e.get("joined").is_some())
        }
    }

    impl Drop for C2A3aTerminalHarness {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.barrier_dir);
            let _ = std::fs::remove_dir_all(&self._runtime_dir);
            let _ = std::fs::remove_dir_all(&self.replay_dir);
            let _ = std::fs::remove_file(&self.trail_path);
        }
    }

    // ===================================================================
    // TEST 1 — Denied + successful sibling
    // ===================================================================

    #[test]
    fn c2a3a_terminal_denied_plus_success() {
        let h = TerminalHarnessBuilder::new("denied")
            .policy_a(PolicyDecision::Deny)
            .policy_b(PolicyDecision::Allow)
            .build();

        let result = h.run_group_with_replay("eval-denied-1", || {
            crate::replay_runtime::test_support::TestReplayAuthority::default()
        });

        // A is denied during preparation (Stage A).  B is eligible and
        // invokes successfully through the provider barrier.
        std::thread::scope(|s| {
            s.spawn(|| {
                h.wait_barrier_entries(1);
                h.release_member("b");
            });
        });

        // Final result: first non-success is Denied for member-a.
        assert!(
            matches!(&result, ExecutionServiceResult::Denied { action_id, .. } if action_id == "member-a"),
            "expected Denied for member-a, got: {result:?}"
        );

        // GroupJoin joined=false.
        let join = h.group_join_entry().expect("GroupJoin must exist");
        assert_eq!(join["joined"].as_bool(), Some(false));

        // No fake OutcomeEntry invented for A if serial Denied path wouldn't create one.
        // Denied is a preparation terminal — no provider invocation, no OutcomeEntry.
        assert!(
            !h.has_member_outcome("member-a"),
            "Denied member must NOT have an OutcomeEntry"
        );
        assert!(
            h.has_member_outcome("member-b"),
            "successful sibling must have an OutcomeEntry"
        );
    }

    // ===================================================================
    // TEST 2 — ApprovalRequired + successful sibling
    // ===================================================================

    #[test]
    fn c2a3a_terminal_approval_required_plus_success() {
        let h = TerminalHarnessBuilder::new("approval")
            .policy_a(PolicyDecision::Ask)
            .policy_b(PolicyDecision::Allow)
            .build();

        let result = h.run_group_with_replay("eval-approval-1", || {
            crate::replay_runtime::test_support::TestReplayAuthority::default()
        });

        std::thread::scope(|s| {
            s.spawn(|| {
                h.wait_barrier_entries(1);
                h.release_member("b");
            });
        });

        // Final result: first non-success is ApprovalRequired for member-a.
        assert!(
            matches!(&result, ExecutionServiceResult::ApprovalRequired { action_id, .. } if action_id == "member-a"),
            "expected ApprovalRequired for member-a, got: {result:?}"
        );

        // GroupJoin joined=false.
        let join = h.group_join_entry().expect("GroupJoin must exist");
        assert_eq!(join["joined"].as_bool(), Some(false));

        // Approval creates an ApprovalEntry in Trail, not an OutcomeEntry.
        assert!(
            !h.has_member_outcome("member-a"),
            "ApprovalRequired member must NOT have an OutcomeEntry"
        );
        assert!(
            h.has_member_outcome("member-b"),
            "successful sibling must have an OutcomeEntry"
        );
    }

    // ===================================================================
    // TEST 3 — Unavailable + successful sibling
    // ===================================================================

    #[test]
    fn c2a3a_terminal_unavailable_plus_success() {
        // Keep both semantic Actions in the plan.  Exclude provider-a from the
        // host availability snapshot so member-a becomes exactly Unavailable
        // without being removed from the Runtime Plan.  B stays eligible.
        let h = TerminalHarnessBuilder::new("unavailable")
            .provider_a_unavailable()
            .peer_count(1)
            .policy_a(PolicyDecision::Allow)
            .policy_b(PolicyDecision::Allow)
            .build();

        let mut result: Option<ExecutionServiceResult> = None;
        std::thread::scope(|s| {
            let handle = s.spawn(|| {
                h.run_group_with_replay("eval-unavailable-1", || {
                    crate::replay_runtime::test_support::TestReplayAuthority::default()
                })
            });

            // B must enter real provider tools/call.
            h.wait_barrier_entries(1);
            h.release_member("b");

            result = Some(handle.join().expect("group must not panic"));
        });

        let result = result.expect("group must have produced a result");

        // Exact terminal classification, not a flattened non-success.
        assert!(
            matches!(result, ExecutionServiceResult::Unavailable { .. }),
            "expected exact Unavailable, got: {result:?}"
        );

        // GroupJoin joined=false.
        let join = h.group_join_entry().expect("GroupJoin must exist");
        assert_eq!(join["joined"].as_bool(), Some(false));

        // member-a was NOT silently removed: member_action_ids holds BOTH.
        let member_ids: Vec<String> = join["member_action_ids"]
            .as_array()
            .expect("GroupJoin must carry member_action_ids")
            .iter()
            .map(|v| v.as_str().expect("member id must be a string").to_owned())
            .collect();
        assert!(
            member_ids.contains(&"member-a".to_owned()),
            "member-a must remain a semantic member, got: {member_ids:?}"
        );
        assert!(
            member_ids.contains(&"member-b".to_owned()),
            "member-b must remain a semantic member, got: {member_ids:?}"
        );

        // A is a preparation terminal: no OutcomeEntry.  B completes.
        assert!(
            !h.has_member_outcome("member-a"),
            "Unavailable member must NOT have an OutcomeEntry"
        );
        let b_outcome = h
            .outcome_entry("member-b")
            .expect("B must have an OutcomeEntry");
        assert_eq!(
            b_outcome["status"].as_str(),
            Some("succeeded"),
            "B must complete successfully, got: {b_outcome:?}"
        );
    }

    // ===================================================================
    // TEST 4 — ReplayBlockedCompletedFailure
    // ===================================================================

    #[test]
    fn c2a3a_terminal_replay_blocked_completed_failure() {
        let h = TerminalHarnessBuilder::new("replay-fail")
            .peer_count(1)
            .policy_a(PolicyDecision::Allow)
            .policy_b(PolicyDecision::Allow)
            .build();

        let mut result: Option<ExecutionServiceResult> = None;
        std::thread::scope(|s| {
            let handle = s.spawn(|| {
                h.run_group_with_boxed_replay(
                    "eval-replay-fail-1",
                    Box::new(ObservingReplayAuthority::new(&[(
                        "member-a",
                        ReplayState::Failed,
                    )])),
                )
            });

            // Only B enters the barrier (A is replay-blocked during preparation).
            h.wait_barrier_entries(1);
            h.release_member("b");

            result = Some(handle.join().expect("group must not panic"));
        });

        let result = result.expect("group must have produced a result");

        // Final result: first non-success is ReplayBlockedCompletedFailure.
        assert!(
            matches!(&result, ExecutionServiceResult::ReplayBlockedCompletedFailure { action_id, .. } if action_id == "member-a"),
            "expected ReplayBlockedCompletedFailure for member-a, got: {result:?}"
        );

        let join = h.group_join_entry().expect("GroupJoin must exist");
        assert_eq!(join["joined"].as_bool(), Some(false));

        // ReplayBlocked is a preparation terminal — no OutcomeEntry.
        assert!(
            !h.has_member_outcome("member-a"),
            "replay-blocked member must NOT have an OutcomeEntry"
        );
        let b_outcome = h
            .outcome_entry("member-b")
            .expect("B must have an OutcomeEntry");
        assert_eq!(
            b_outcome["status"].as_str(),
            Some("succeeded"),
            "successful sibling must succeed, got: {b_outcome:?}"
        );
    }

    // ===================================================================
    // TEST 5 — ReplayBlockedCompletedSuccess
    // ===================================================================

    #[test]
    fn c2a3a_terminal_replay_blocked_completed_success() {
        // A = ReplayBlockedCompletedSuccess (recovered Succeeded state),
        // B = normal provider success.  No infrastructure failure permitted.
        let h = TerminalHarnessBuilder::new("replay-success")
            .peer_count(1)
            .policy_a(PolicyDecision::Allow)
            .policy_b(PolicyDecision::Allow)
            .build();

        let trace = ReplayTrace::new();
        let trace_for_thread = trace.clone();

        let mut result: Option<ExecutionServiceResult> = None;
        std::thread::scope(|s| {
            let handle = s.spawn(|| {
                h.run_group_with_boxed_replay(
                    "eval-replay-success-1",
                    Box::new(ObservingReplayAuthority::with_trace(
                        &[("member-a", ReplayState::Succeeded)],
                        trace_for_thread,
                    )),
                )
            });

            // Only B enters the barrier (A is replay-blocked during preparation).
            h.wait_barrier_entries(1);
            h.release_member("b");

            result = Some(handle.join().expect("group must not panic"));
        });

        let result = result.expect("group must have produced a result");

        // ReplayBlockedCompletedSuccess counts as success: the group must join.
        assert!(
            matches!(&result, ExecutionServiceResult::Completed { .. }),
            "expected Completed (replay-blocked-success counts as success), got: {result:?}"
        );

        let join = h.group_join_entry().expect("GroupJoin must exist");
        assert_eq!(join["joined"].as_bool(), Some(true));

        // B completed successfully.
        let b_outcome = h
            .outcome_entry("member-b")
            .expect("B must have an OutcomeEntry");
        assert_eq!(
            b_outcome["status"].as_str(),
            Some("succeeded"),
            "B must complete successfully, got: {b_outcome:?}"
        );

        // A was admitted as a recovered Succeeded state, which maps to
        // ReplayBlockedCompletedSuccess (the only success replay classification).
        assert!(
            trace.has("admit:member-a:blocked:Succeeded"),
            "A must be admitted as replay-blocked success, trace: {:?}",
            trace.snapshot()
        );
    }

    // ===================================================================
    // TEST 6 — Unattempted (deadline expiry before provider invocation)
    // ===================================================================

    #[test]
    fn c2a3a_terminal_unattempted() {
        // Set A's timeout to 0ms so the deadline expires during Stage B
        // before the provider can be invoked.  The monotonic clock check
        // uses >= so 0ms means the deadline is always expired.
        let h = TerminalHarnessBuilder::new("unattempted")
            .timeout_a_ms(0)
            .policy_a(PolicyDecision::Allow)
            .policy_b(PolicyDecision::Allow)
            .build();

        let mut result: Option<ExecutionServiceResult> = None;
        std::thread::scope(|s| {
            let handle = s.spawn(|| {
                h.run_group_with_replay("eval-unattempted-1", || {
                    crate::replay_runtime::test_support::TestReplayAuthority::default()
                })
            });

            // B may or may not enter depending on timing; release it eagerly.
            let deadline = std::time::Instant::now() + Duration::from_secs(5);
            while barrier_entered_count(&h.barrier_dir) < 1 {
                if std::time::Instant::now() >= deadline {
                    break;
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            h.release_member("b");

            result = Some(handle.join().expect("group must not panic"));
        });

        let result = result.expect("group must have produced a result");

        // Final result: first non-success is Unattempted for member-a.
        assert!(
            matches!(&result, ExecutionServiceResult::Unattempted { action_id, .. } if action_id == "member-a"),
            "expected Unattempted for member-a, got: {result:?}"
        );

        let join = h.group_join_entry().expect("GroupJoin must exist");
        assert_eq!(join["joined"].as_bool(), Some(false));
    }

    // ===================================================================
    // TEST 7 — Uncertain (provider error diagnostic)
    // ===================================================================

    #[test]
    fn c2a3a_terminal_uncertain() {
        // We cannot easily make the real provider return an error, but we
        // can inject a panic into worker action_index=0 (member-a).
        // The catch_unwind inside the real worker thread converts the panic
        // into Uncertain via NoFinalResponse diagnostic.
        let h = TerminalHarnessBuilder::new("uncertain")
            .policy_a(PolicyDecision::Allow)
            .policy_b(PolicyDecision::Allow)
            .build();

        let _guard = PanicGuard::target(0);

        let mut result: Option<ExecutionServiceResult> = None;
        std::thread::scope(|s| {
            let handle = s.spawn(|| {
                h.run_group_with_replay("eval-uncertain-1", || {
                    crate::replay_runtime::test_support::TestReplayAuthority::default()
                })
            });

            let deadline = std::time::Instant::now() + Duration::from_secs(5);
            while barrier_entered_count(&h.barrier_dir) < 1 {
                if std::time::Instant::now() >= deadline {
                    break;
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            h.release_member("b");

            result = Some(handle.join().expect("group must not panic"));
        });

        let result = result.expect("group must have produced a result");

        // Final result: first non-success is Uncertain for member-a.
        assert!(
            matches!(&result, ExecutionServiceResult::Uncertain { action_id, .. } if action_id == "member-a"),
            "expected Uncertain for member-a, got: {result:?}"
        );

        let join = h.group_join_entry().expect("GroupJoin must exist");
        assert_eq!(join["joined"].as_bool(), Some(false));
    }

    // ===================================================================
    // TEST 8 — Deterministic final result independent of physical order
    // ===================================================================

    #[test]
    fn c2a3a_terminal_deterministic_result_independent_of_order() {
        // Semantic order: A (Uncertain), B (Failed).  Both reach the real
        // provider boundary and produce distinct terminal results whose
        // physical delivery order can be inverted via independent release.
        //
        // The final aggregate must always be semantic member A's Uncertain,
        // regardless of which member physically completes first.
        for (run_label, first, second) in [("B-first", "b", "a"), ("A-first", "a", "b")] {
            let h = TerminalHarnessBuilder::new(&format!("order-{run_label}"))
                .policy_a(PolicyDecision::Allow)
                .policy_b(PolicyDecision::Allow)
                .outcome_a(OutcomeMode::Uncertain)
                .outcome_b(OutcomeMode::Failed)
                .build();

            let mut result: Option<ExecutionServiceResult> = None;
            std::thread::scope(|s| {
                let handle = s.spawn(|| {
                    h.run_group_with_replay(&format!("eval-order-{run_label}"), || {
                        crate::replay_runtime::test_support::TestReplayAuthority::default()
                    })
                });

                // Both members must reach real tools/call.
                h.wait_barrier_entries(2);

                // Release one member and require its durable Stage C outcome
                // before allowing the second member to complete.
                h.release_member(first);

                let first_action_id = format!("member-{first}");
                let deadline = std::time::Instant::now() + Duration::from_secs(15);
                while !h.has_member_outcome(&first_action_id) {
                    assert!(
                        std::time::Instant::now() < deadline,
                        "[{run_label}] {first_action_id} outcome never became durable"
                    );
                    std::thread::sleep(Duration::from_millis(10));
                }

                // The second provider is still physically blocked here.
                assert!(
                    !h.has_member_outcome(&format!("member-{second}")),
                    "[{run_label}] second member completed before release"
                );

                h.release_member(second);

                result = Some(handle.join().expect("group coordinator must not panic"));
            });

            let result = result.expect("group must produce a result");

            // Semantic order is always A then B.  A = Uncertain, B = Failed.
            // Therefore A must win aggregate selection regardless of physical order.
            assert!(
                matches!(
                    &result,
                    ExecutionServiceResult::Uncertain { action_id, .. } if action_id == "member-a"
                ),
                "[{run_label}] expected semantic member-a Uncertain, got {result:?}"
            );

            let expected_order = if first == "b" {
                vec!["member-b".to_owned(), "member-a".to_owned()]
            } else {
                vec!["member-a".to_owned(), "member-b".to_owned()]
            };
            assert_eq!(
                h.outcome_ids(),
                expected_order,
                "[{run_label}] durable physical OutcomeEntry order"
            );

            let join = h.group_join_entry().expect("GroupJoin must exist");
            assert_eq!(
                join["joined"].as_bool(),
                Some(false),
                "[{run_label}] two non-success members must not join successfully"
            );
        }
    }

    // ===================================================================
    // TEST 9 — Panic exact Uncertain classification (strengthened)
    // ===================================================================

    #[test]
    fn c2a3a_terminal_panic_exact_uncertain() {
        // member-b (semantic ordinal 1) panics inside the real spawned worker.
        // member-a reaches the provider and succeeds.  The final aggregate
        // must be EXACTLY Uncertain for member-b.
        let h = TerminalHarnessBuilder::new("panic-exact")
            .peer_count(1)
            .policy_a(PolicyDecision::Allow)
            .policy_b(PolicyDecision::Allow)
            .build();

        // Target action_index=1 (member-b) for panic injection.
        let _guard = PanicGuard::target(1);

        let mut result: Option<ExecutionServiceResult> = None;
        std::thread::scope(|s| {
            let handle = s.spawn(|| {
                h.run_group_with_replay("eval-panic-exact-1", || {
                    crate::replay_runtime::test_support::TestReplayAuthority::default()
                })
            });

            // member-a reaches the barrier (member-b panics before entering).
            h.wait_barrier_entries(1);
            h.release_member("a");

            result = Some(handle.join().expect("group must not panic"));
        });

        let result = result.expect("group must have produced a result");

        // Exact returned final result: Uncertain for semantic member-b.
        assert!(
            matches!(&result, ExecutionServiceResult::Uncertain { action_id, .. } if action_id == "member-b"),
            "expected exact Uncertain for member-b, got: {result:?}"
        );

        // member-a succeeded, member-b has no OutcomeEntry (panic is a
        // pre-provider classification carried through Stage C).
        let a_outcome = h
            .outcome_entry("member-a")
            .expect("A must have an OutcomeEntry");
        assert_eq!(
            a_outcome["status"].as_str(),
            Some("succeeded"),
            "member-a must succeed, got: {a_outcome:?}"
        );

        let join = h.group_join_entry().expect("GroupJoin must exist");
        assert_eq!(join["joined"].as_bool(), Some(false));
    }

    // ===================================================================
    // TEST 10 — Intent before provider effect
    // ===================================================================

    #[test]
    fn c2a3a_terminal_intent_before_effect() {
        // Both members use the barrier fixture.  Before either provider can
        // perform an effect, each member's durable Trail intent must exist.
        let h = TerminalHarnessBuilder::new("intent")
            .policy_a(PolicyDecision::Allow)
            .policy_b(PolicyDecision::Allow)
            .build();

        // Run in a thread so we can poll the Trail while both providers are
        // blocked at the effect gate.
        std::thread::scope(|s| {
            let handle = s.spawn(|| {
                h.run_group_with_replay("eval-intent-1", || {
                    crate::replay_runtime::test_support::TestReplayAuthority::default()
                })
            });

            // Wait for both to enter the barrier (both reached tools/call).
            h.wait_barrier_entries(2);

            // Before releasing provider effect, BOTH members must already have
            // durable Trail intent.
            let entries = h.trail_entries();
            let has_intent = |member: &str| {
                entries.iter().any(|e| {
                    e.get("execution_id").is_some()
                        && e.get("action_id").and_then(Value::as_str) == Some(member)
                        && e.get("status").is_none()
                        && e.get("capability_name").is_some()
                })
            };
            assert!(
                has_intent("member-a"),
                "member-a must have durable Trail intent before provider effect"
            );
            assert!(
                has_intent("member-b"),
                "member-b must have durable Trail intent before provider effect"
            );

            // Release both.
            h.release_member("a");
            h.release_member("b");

            let result = handle.join().expect("group must not panic");
            assert!(matches!(result, ExecutionServiceResult::Completed { .. }));
        });

        // After completion, each member's intent must precede its outcome.
        let entries = h.trail_entries();
        for member in ["member-a", "member-b"] {
            let intent_pos = entries
                .iter()
                .position(|e| {
                    e.get("execution_id").is_some()
                        && e.get("action_id").and_then(Value::as_str) == Some(member)
                        && e.get("status").is_none()
                        && e.get("capability_name").is_some()
                })
                .expect("intent entry must exist for {member}");
            let outcome_pos = entries
                .iter()
                .position(|e| {
                    e.get("execution_id").is_some()
                        && e.get("status").is_some()
                        && e.get("action_id").and_then(Value::as_str) == Some(member)
                })
                .expect("outcome entry must exist for {member}");
            assert!(
                intent_pos < outcome_pos,
                "intent (pos {intent_pos}) must precede outcome (pos {outcome_pos}) for {member}"
            );
        }
    }

    // ===================================================================
    // TEST 11 — G1 (armed) before provider effect
    // ===================================================================

    #[test]
    fn c2a3a_terminal_g1_before_effect() {
        // Directly observe replay G0 (intent) / G1 (armed) / G2 (terminal)
        // through an instrumented test-only replay authority.  G1 must be
        // observed before any provider effect is released, and G2 after
        // completion, with G0 -> G1 -> G2 ordering per member.
        let h = TerminalHarnessBuilder::new("g1-before")
            .policy_a(PolicyDecision::Allow)
            .policy_b(PolicyDecision::Allow)
            .build();

        let trace = ReplayTrace::new();
        let trace_for_thread = trace.clone();

        std::thread::scope(|s| {
            let handle = s.spawn(|| {
                h.run_group_with_boxed_replay(
                    "eval-g1-1",
                    Box::new(ObservingReplayAuthority::with_trace(&[], trace_for_thread)),
                )
            });

            // Wait for both to enter the barrier (both reached tools/call).
            h.wait_barrier_entries(2);

            // BEFORE releasing provider effect, G0 and G1 must be observed
            // for BOTH invoked members.
            for member in ["member-a", "member-b"] {
                assert!(
                    trace.has(&format!("g0:{member}")),
                    "G0 for {member} must be observed before provider effect"
                );
                assert!(
                    trace.has(&format!("g1:{member}")),
                    "G1 for {member} must be observed before provider effect, trace: {:?}",
                    trace.snapshot()
                );
            }

            // Release both.
            h.release_member("a");
            h.release_member("b");

            let result = handle.join().expect("group must not panic");
            assert!(matches!(result, ExecutionServiceResult::Completed { .. }));
        });

        // After completion, G2 must be observed, and ordering per member is
        // G0 -> G1 -> G2.
        let snapshot = trace.snapshot();
        for member in ["member-a", "member-b"] {
            let g0 = format!("g0:{member}");
            let g1 = format!("g1:{member}");
            let g2 = format!("g2:{member}");
            let pos = |e: &String| {
                snapshot
                    .iter()
                    .position(|x| x == e)
                    .unwrap_or_else(|| panic!("{e} must be observed"))
            };
            let g0_pos = pos(&g0);
            let g1_pos = pos(&g1);
            let g2_pos = pos(&g2);
            assert!(
                g0_pos < g1_pos,
                "G0 (pos {g0_pos}) before G1 (pos {g1_pos}) for {member}"
            );
            assert!(
                g1_pos < g2_pos,
                "G1 (pos {g1_pos}) before G2 (pos {g2_pos}) for {member}"
            );
        }
    }

    // ===================================================================
    // C3-A1 Group Test Harness and Proofs: Minimal Bounded Launch Window
    // ===================================================================

    struct C3A1GroupHarness {
        runtime: PreparedRuntime,
        _runtime_dir: PathBuf,
        trail_path: PathBuf,
        replay_dir: PathBuf,
        barrier_dir: PathBuf,
        members: Vec<String>,
    }

    impl C3A1GroupHarness {
        fn new(test_name: &str, member_tags: &[&str]) -> Self {
            Self::new_with_timeout_overrides(test_name, member_tags, &HashMap::new())
        }

        fn new_with_timeout_overrides(
            test_name: &str,
            member_tags: &[&str],
            timeout_overrides: &HashMap<String, u64>,
        ) -> Self {
            let barrier_dir = std::env::temp_dir()
                .join(format!("tethers-c3a1-{test_name}-{}", uuid::Uuid::new_v4()));
            std::fs::create_dir_all(&barrier_dir).unwrap();

            let runtime_dir = std::env::temp_dir().join(format!(
                "tethers-c3a1-{test_name}-rt-{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            std::fs::create_dir_all(runtime_dir.join("tethers")).unwrap();
            std::fs::create_dir_all(runtime_dir.join("manifests")).unwrap();

            let tether_actions = member_tags
                .iter()
                .map(|tag| format!("do fixture.ping-{tag}"))
                .collect::<Vec<_>>()
                .join(" ");
            std::fs::write(
                runtime_dir.join("tethers/together-test.tether"),
                format!("when event.test if true {tether_actions}"),
            )
            .unwrap();

            let manifest_json =
                include_str!("../../protocol/capability-manifests/fixture-ping.json");
            let make_manifest = |cap_name: &str, provider_id: &str| -> String {
                let mut m: serde_json::Value = serde_json::from_str(manifest_json).unwrap();
                m["capability_name"] = serde_json::json!(cap_name);
                m["provider"]["identity"] = serde_json::json!(provider_id);
                m["binding"]["server_name"] = serde_json::json!("tethers-stdio-fixture");
                m["permission_scope"] =
                    serde_json::json!({"kind": "path_prefix", "allowed_prefixes": ["member/"]});
                m["confirmation_policy"] =
                    serde_json::json!({"standing_permitted": true, "per_call_required": false});
                let s = serde_json::to_string(&m).unwrap();
                let (_, digest) = crate::manifest::canonicalize_and_digest(&s).unwrap();
                m["digest"] = serde_json::json!(digest);
                serde_json::to_string_pretty(&m).unwrap()
            };

            let mut digests = HashMap::new();
            for tag in member_tags {
                let cap_name = format!("fixture.ping-{tag}");
                let provider_id = format!("provider-{tag}");
                let manifest = make_manifest(&cap_name, &provider_id);
                let manifest_path = runtime_dir.join(format!("manifests/fixture-ping-{tag}.json"));
                let applied_manifest = if let Some(&timeout_ms) = timeout_overrides.get(*tag) {
                    let mut m: serde_json::Value = serde_json::from_str(&manifest).unwrap();
                    m["timeout_ms"] = serde_json::json!(timeout_ms);
                    let s = serde_json::to_string(&m).unwrap();
                    let (_, digest) = crate::manifest::canonicalize_and_digest(&s).unwrap();
                    m["digest"] = serde_json::json!(digest);
                    serde_json::to_string_pretty(&m).unwrap()
                } else {
                    manifest
                };
                std::fs::write(&manifest_path, &applied_manifest).unwrap();
                let (_, digest) =
                    crate::manifest::canonicalize_and_digest(&applied_manifest).unwrap();
                digests.insert((*tag).to_owned(), digest);
            }

            let barrier_script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join("scripts")
                .join("tethers-stdio-fixture.ps1");
            let barrier_str = barrier_dir.to_str().unwrap().to_owned();

            let reqs: Vec<serde_json::Value> = member_tags
                .iter()
                .map(|tag| {
                    json!({
                        "name": format!("fixture.ping-{tag}"),
                        "version": 1,
                        "reason": "c3 bounded concurrency"
                    })
                })
                .collect();

            let providers_json: Vec<serde_json::Value> = member_tags
                .iter()
                .map(|tag| {
                    let provider_id = format!("provider-{tag}");
                    let cap_name = format!("fixture.ping-{tag}");
                    let digest = digests.get(*tag).unwrap();
                    json!({
                        "id": provider_id,
                        "display_name": format!("Provider {tag}"),
                        "transport": {
                            "kind": "stdio",
                            "command": "pwsh.exe",
                            "args": [
                                "-NoProfile", "-ExecutionPolicy", "Bypass", "-File",
                                barrier_script.to_str().unwrap(),
                                "-Mode", "c2-overlap-barrier",
                                "-BarrierDirectory", &barrier_str
                            ],
                            "protocol_version": "2025-11-25"
                        },
                        "capabilities": [{
                            "name": cap_name,
                            "version": 1,
                            "manifest_path": format!("manifests/fixture-ping-{tag}.json"),
                            "pinned_digest": digest,
                            "scope_binding": {"kind": "path_prefix", "argument_json_pointer": "/message"}
                        }]
                    })
                })
                .collect();

            let rules_json: Vec<serde_json::Value> = member_tags
                .iter()
                .map(|tag| {
                    json!({
                        "name": format!("fixture.ping-{tag}"),
                        "version": 1,
                        "decision": "allow"
                    })
                })
                .collect();

            let config = json!({
                "format_version": "0.1",
                "tether_set": {
                    "id": "test.together",
                    "version": "1",
                    "tethers": [{
                        "id": "together-test",
                        "version": "1",
                        "source_path": "tethers/together-test.tether"
                    }],
                    "capability_requirements": reqs
                },
                "providers": providers_json,
                "policy": {
                    "default": "deny",
                    "rules": rules_json
                }
            });

            let config_path = runtime_dir.join("tethers-config.json");
            std::fs::write(&config_path, serde_json::to_string_pretty(&config).unwrap()).unwrap();
            let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
            let runtime = prepare_runtime(&loaded).unwrap();

            let trail_path = std::env::temp_dir().join(format!(
                "tethers-c3a1-{test_name}-trail-{}.jsonl",
                uuid::Uuid::new_v4()
            ));
            let replay_dir = std::env::temp_dir().join(format!(
                "tethers-c3a1-{test_name}-replay-{}",
                uuid::Uuid::new_v4()
            ));
            std::fs::create_dir_all(&replay_dir).unwrap();

            Self {
                runtime,
                _runtime_dir: runtime_dir,
                trail_path,
                replay_dir,
                barrier_dir,
                members: member_tags.iter().map(|s| (*s).to_owned()).collect(),
            }
        }

        fn set_peer_count(&self, count: usize) {
            std::fs::write(self.barrier_dir.join("peer-count"), count.to_string()).unwrap();
        }

        fn run_group_with_limit(&self, eval_id: &str, limit: usize) -> ExecutionServiceResult {
            let providers = self.runtime.providers().to_vec();
            let mut sessions = HashMap::new();
            for provider in &providers {
                let manifest = provider.capabilities[0].verified_manifest.manifest();
                let session = RetainedProviderSession::establish(SocketEstablishment {
                    command: &provider.stdio_config.command,
                    args: &provider.stdio_config.args,
                    working_directory: &provider.working_directory,
                    protocol_version: &provider.stdio_config.protocol_version,
                    server_name: &manifest.binding.server_name,
                    identity: &provider.identity,
                })
                .expect("barrier provider session establishment");
                sessions.insert(provider.identity.clone(), session);
            }

            let mut actions = Vec::new();
            let mut member_action_ids = Vec::new();
            let mut member_indexes = Vec::new();

            for (idx, tag) in self.members.iter().enumerate() {
                let action_id = format!("member-{tag}");
                let provider_id = format!("provider-{tag}");
                let cap_name = format!("fixture.ping-{tag}");
                let digest = self
                    .runtime
                    .providers()
                    .iter()
                    .find(|p| p.identity == provider_id)
                    .unwrap()
                    .capabilities[0]
                    .verified_manifest
                    .verified_digest()
                    .to_owned();

                actions.push(json!({
                    "action_id": action_id,
                    "idempotency_key": format!("{eval_id}/{action_id}"),
                    "capability": cap_name,
                    "capability_version": "1.0.0",
                    "bridge_capability_version": 1,
                    "bridge_provider_identity": provider_id,
                    "manifest_digest": digest,
                    "arguments": {"message": format!("member/{tag}")},
                }));
                member_action_ids.push(action_id);
                member_indexes.push(idx);
            }

            let groups = vec![json!({
                "group_id": "together-1",
                "member_action_ids": member_action_ids,
            })];
            let mut response = json!({
                "status": "matched",
                "evaluation_id": eval_id,
                "plan": { "id": format!("plan-{eval_id}"), "actions": actions, "groups": groups },
                "trail": [],
            });
            let member_actions = response["plan"]["actions"].as_array().unwrap().clone();
            let avail_identities: Vec<String> = self
                .members
                .iter()
                .map(|tag| format!("provider-{tag}"))
                .collect();
            let availability =
                ProviderAvailability::from_identities(avail_identities.iter().map(|s| s.as_str()));
            let mut trail = dispatch::FileTrail::open(&self.trail_path).unwrap();
            let mut approvals = crate::approval::ApprovalStore::default();
            let mut replay_authority =
                crate::replay_runtime::test_support::TestReplayAuthority::default();
            let engine_path = PathBuf::from("unused-engine");
            let service =
                HostExecutionService::new(&self.runtime, &engine_path, &self.trail_path, None);

            execute_group_concurrent_with_limit(
                "together-1",
                &member_indexes,
                &member_actions,
                &mut response,
                eval_id,
                &mut trail,
                &service,
                &PreparedEvaluationInput {
                    tether_id: "together-test".to_owned(),
                    tether_version: "1".to_owned(),
                    evaluation_id: eval_id.to_owned(),
                    anchor_event: json!({"id": format!("evt-{eval_id}"), "name": "test"}),
                    facts: json!({}),
                },
                &mut sessions,
                &availability,
                &mut approvals,
                &mut replay_authority,
                limit,
            )
        }

        fn has_active(&self, member: &str) -> bool {
            self.barrier_dir
                .join(format!("active-member-{member}"))
                .exists()
        }

        fn currently_active_count(&self) -> usize {
            self.members
                .iter()
                .filter(|tag| self.has_active(tag) && !self.has_member_outcome(tag))
                .count()
        }

        fn has_entered(&self, member: &str) -> bool {
            self.barrier_dir
                .join(format!("entered-member-{member}"))
                .exists()
        }

        fn wait_member_active(&self, member: &str) {
            let active_file = self.barrier_dir.join(format!("active-member-{member}"));
            let deadline = std::time::Instant::now() + Duration::from_secs(15);
            while !active_file.exists() {
                assert!(
                    std::time::Instant::now() < deadline,
                    "member-{member} did not reach active state in barrier within 15s"
                );
                std::thread::sleep(Duration::from_millis(10));
            }
        }

        fn wait_barrier_active_count(&self, count: usize) {
            let deadline = std::time::Instant::now() + Duration::from_secs(15);
            while self.currently_active_count() < count {
                assert!(
                    std::time::Instant::now() < deadline,
                    "only {} of {count} providers reached currently active state within 15s",
                    self.currently_active_count()
                );
                std::thread::sleep(Duration::from_millis(10));
            }
        }

        fn has_member_outcome(&self, member: &str) -> bool {
            let action_id = format!("member-{member}");
            trail_has_member_outcome(&self.trail_path, &action_id)
        }

        fn wait_member_outcome(&self, member: &str) {
            let deadline = std::time::Instant::now() + Duration::from_secs(15);
            while !self.has_member_outcome(member) {
                assert!(
                    std::time::Instant::now() < deadline,
                    "outcome for member-{member} did not appear in Trail within 15s"
                );
                std::thread::sleep(Duration::from_millis(10));
            }
        }

        fn release_member(&self, member: &str) {
            std::fs::write(
                self.barrier_dir.join(format!("release-member-{member}")),
                "release",
            )
            .unwrap();
        }

        fn release_all(&self) {
            std::fs::write(self.barrier_dir.join("release"), "release").unwrap();
        }

        fn trail_content(&self) -> String {
            std::fs::read_to_string(&self.trail_path).unwrap_or_default()
        }

        fn outcome_ids(&self) -> Vec<String> {
            trail_outcome_action_ids(&self.trail_path)
        }

        fn entry_kinds(&self) -> Vec<String> {
            trail_entry_kinds(&self.trail_path)
        }

        fn run_group_with_trace(
            &self,
            eval_id: &str,
            limit: usize,
        ) -> (ExecutionServiceResult, Vec<String>) {
            let providers = self.runtime.providers().to_vec();
            let mut sessions = HashMap::new();
            for provider in &providers {
                let manifest = provider.capabilities[0].verified_manifest.manifest();
                let session = RetainedProviderSession::establish(SocketEstablishment {
                    command: &provider.stdio_config.command,
                    args: &provider.stdio_config.args,
                    working_directory: &provider.working_directory,
                    protocol_version: &provider.stdio_config.protocol_version,
                    server_name: &manifest.binding.server_name,
                    identity: &provider.identity,
                })
                .expect("barrier provider session establishment");
                sessions.insert(provider.identity.clone(), session);
            }

            let mut actions = Vec::new();
            let mut member_action_ids = Vec::new();
            let mut member_indexes = Vec::new();

            for (idx, tag) in self.members.iter().enumerate() {
                let action_id = format!("member-{tag}");
                let provider_id = format!("provider-{tag}");
                let cap_name = format!("fixture.ping-{tag}");
                let digest = self
                    .runtime
                    .providers()
                    .iter()
                    .find(|p| p.identity == provider_id)
                    .unwrap()
                    .capabilities[0]
                    .verified_manifest
                    .verified_digest()
                    .to_owned();

                actions.push(json!({
                    "action_id": action_id,
                    "idempotency_key": format!("{eval_id}/{action_id}"),
                    "capability": cap_name,
                    "capability_version": "1.0.0",
                    "bridge_capability_version": 1,
                    "bridge_provider_identity": provider_id,
                    "manifest_digest": digest,
                    "arguments": {"message": format!("member/{tag}")},
                }));
                member_action_ids.push(action_id);
                member_indexes.push(idx);
            }

            let groups = vec![json!({
                "group_id": "together-1",
                "member_action_ids": member_action_ids,
            })];
            let mut response = json!({
                "status": "matched",
                "evaluation_id": eval_id,
                "plan": { "id": format!("plan-{eval_id}"), "actions": actions, "groups": groups },
                "trail": [],
            });
            let member_actions = response["plan"]["actions"].as_array().unwrap().clone();
            let avail_identities: Vec<String> = self
                .members
                .iter()
                .map(|tag| format!("provider-{tag}"))
                .collect();
            let availability =
                ProviderAvailability::from_identities(avail_identities.iter().map(|s| s.as_str()));
            let mut trail = dispatch::FileTrail::open(&self.trail_path).unwrap();
            let mut approvals = crate::approval::ApprovalStore::default();
            let trace = ReplayTrace::new();
            let mut replay_authority = ObservingReplayAuthority::with_trace(&[], trace.clone());
            let engine_path = PathBuf::from("unused-engine");
            let service =
                HostExecutionService::new(&self.runtime, &engine_path, &self.trail_path, None);

            let result = execute_group_concurrent_with_limit(
                "together-1",
                &member_indexes,
                &member_actions,
                &mut response,
                eval_id,
                &mut trail,
                &service,
                &PreparedEvaluationInput {
                    tether_id: "together-test".to_owned(),
                    tether_version: "1".to_owned(),
                    evaluation_id: eval_id.to_owned(),
                    anchor_event: json!({"id": format!("evt-{eval_id}"), "name": "test"}),
                    facts: json!({}),
                },
                &mut sessions,
                &availability,
                &mut approvals,
                &mut replay_authority,
                limit,
            );

            (result, trace.snapshot())
        }

        fn run_group_with_live_trace(
            &self,
            eval_id: &str,
            limit: usize,
            shared_trace: &ReplayTrace,
        ) -> (ExecutionServiceResult, Vec<String>) {
            let providers = self.runtime.providers().to_vec();
            let mut sessions = HashMap::new();
            for provider in &providers {
                let manifest = provider.capabilities[0].verified_manifest.manifest();
                let session = RetainedProviderSession::establish(SocketEstablishment {
                    command: &provider.stdio_config.command,
                    args: &provider.stdio_config.args,
                    working_directory: &provider.working_directory,
                    protocol_version: &provider.stdio_config.protocol_version,
                    server_name: &manifest.binding.server_name,
                    identity: &provider.identity,
                })
                .expect("barrier provider session establishment");
                sessions.insert(provider.identity.clone(), session);
            }

            let mut actions = Vec::new();
            let mut member_action_ids = Vec::new();
            let mut member_indexes = Vec::new();

            for (idx, tag) in self.members.iter().enumerate() {
                let action_id = format!("member-{tag}");
                let provider_id = format!("provider-{tag}");
                let cap_name = format!("fixture.ping-{tag}");
                let digest = self
                    .runtime
                    .providers()
                    .iter()
                    .find(|p| p.identity == provider_id)
                    .unwrap()
                    .capabilities[0]
                    .verified_manifest
                    .verified_digest()
                    .to_owned();

                actions.push(json!({
                    "action_id": action_id,
                    "idempotency_key": format!("{eval_id}/{action_id}"),
                    "capability": cap_name,
                    "capability_version": "1.0.0",
                    "bridge_capability_version": 1,
                    "bridge_provider_identity": provider_id,
                    "manifest_digest": digest,
                    "arguments": {"message": format!("member/{tag}")},
                }));
                member_action_ids.push(action_id);
                member_indexes.push(idx);
            }

            let groups = vec![json!({
                "group_id": "together-1",
                "member_action_ids": member_action_ids,
            })];
            let mut response = json!({
                "status": "matched",
                "evaluation_id": eval_id,
                "plan": { "id": format!("plan-{eval_id}"), "actions": actions, "groups": groups },
                "trail": [],
            });
            let member_actions = response["plan"]["actions"].as_array().unwrap().clone();
            let avail_identities: Vec<String> = self
                .members
                .iter()
                .map(|tag| format!("provider-{tag}"))
                .collect();
            let availability =
                ProviderAvailability::from_identities(avail_identities.iter().map(|s| s.as_str()));
            let mut trail = dispatch::FileTrail::open(&self.trail_path).unwrap();
            let mut approvals = crate::approval::ApprovalStore::default();
            let mut replay_authority =
                ObservingReplayAuthority::with_trace(&[], shared_trace.clone());
            let engine_path = PathBuf::from("unused-engine");
            let service =
                HostExecutionService::new(&self.runtime, &engine_path, &self.trail_path, None);

            let result = execute_group_concurrent_with_limit(
                "together-1",
                &member_indexes,
                &member_actions,
                &mut response,
                eval_id,
                &mut trail,
                &service,
                &PreparedEvaluationInput {
                    tether_id: "together-test".to_owned(),
                    tether_version: "1".to_owned(),
                    evaluation_id: eval_id.to_owned(),
                    anchor_event: json!({"id": format!("evt-{eval_id}"), "name": "test"}),
                    facts: json!({}),
                },
                &mut sessions,
                &availability,
                &mut approvals,
                &mut replay_authority,
                limit,
            );

            let final_snapshot = shared_trace.snapshot();
            (result, final_snapshot)
        }
        fn set_member_outcome(&self, member: &str, outcome: &str) {
            std::fs::write(
                self.barrier_dir.join(format!("outcome-member-{member}")),
                outcome,
            )
            .unwrap();
        }

        fn run_group_with_trail_and_authority(
            &self,
            eval_id: &str,
            limit: usize,
            trail: &mut dyn dispatch::Trail,
            replay_authority: &mut dyn crate::replay_runtime::ReplayAuthority,
        ) -> ExecutionServiceResult {
            let providers = self.runtime.providers().to_vec();
            let mut sessions = HashMap::new();
            for provider in &providers {
                let manifest = provider.capabilities[0].verified_manifest.manifest();
                let session = RetainedProviderSession::establish(SocketEstablishment {
                    command: &provider.stdio_config.command,
                    args: &provider.stdio_config.args,
                    working_directory: &provider.working_directory,
                    protocol_version: &provider.stdio_config.protocol_version,
                    server_name: &manifest.binding.server_name,
                    identity: &provider.identity,
                })
                .expect("barrier provider session establishment");
                sessions.insert(provider.identity.clone(), session);
            }

            let mut actions = Vec::new();
            let mut member_action_ids = Vec::new();
            let mut member_indexes = Vec::new();

            for (idx, tag) in self.members.iter().enumerate() {
                let action_id = format!("member-{tag}");
                let provider_id = format!("provider-{tag}");
                let cap_name = format!("fixture.ping-{tag}");
                let digest = self
                    .runtime
                    .providers()
                    .iter()
                    .find(|p| p.identity == provider_id)
                    .unwrap()
                    .capabilities[0]
                    .verified_manifest
                    .verified_digest()
                    .to_owned();

                actions.push(json!({
                    "action_id": action_id,
                    "idempotency_key": format!("{eval_id}/{action_id}"),
                    "capability": cap_name,
                    "capability_version": "1.0.0",
                    "bridge_capability_version": 1,
                    "bridge_provider_identity": provider_id,
                    "manifest_digest": digest,
                    "arguments": {"message": format!("member/{tag}")},
                }));
                member_action_ids.push(action_id);
                member_indexes.push(idx);
            }

            let groups = vec![json!({
                "group_id": "together-1",
                "member_action_ids": member_action_ids,
            })];
            let mut response = json!({
                "status": "matched",
                "evaluation_id": eval_id,
                "plan": { "id": format!("plan-{eval_id}"), "actions": actions, "groups": groups },
                "trail": [],
            });
            let member_actions = response["plan"]["actions"].as_array().unwrap().clone();
            let avail_identities: Vec<String> = self
                .members
                .iter()
                .map(|tag| format!("provider-{tag}"))
                .collect();
            let availability =
                ProviderAvailability::from_identities(avail_identities.iter().map(|s| s.as_str()));
            let mut approvals = crate::approval::ApprovalStore::default();
            let engine_path = PathBuf::from("unused-engine");
            let service =
                HostExecutionService::new(&self.runtime, &engine_path, &self.trail_path, None);

            execute_group_concurrent_with_limit(
                "together-1",
                &member_indexes,
                &member_actions,
                &mut response,
                eval_id,
                trail,
                &service,
                &PreparedEvaluationInput {
                    tether_id: "together-test".to_owned(),
                    tether_version: "1".to_owned(),
                    evaluation_id: eval_id.to_owned(),
                    anchor_event: json!({"id": format!("evt-{eval_id}"), "name": "test"}),
                    facts: json!({}),
                },
                &mut sessions,
                &availability,
                &mut approvals,
                replay_authority,
                limit,
            )
        }

        fn run_group_with_authority(
            &self,
            eval_id: &str,
            limit: usize,
            replay_authority: &mut dyn crate::replay_runtime::ReplayAuthority,
        ) -> ExecutionServiceResult {
            let mut trail = dispatch::FileTrail::open(&self.trail_path).unwrap();
            self.run_group_with_trail_and_authority(eval_id, limit, &mut trail, replay_authority)
        }
    }

    impl Drop for C3A1GroupHarness {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.barrier_dir);
            let _ = std::fs::remove_dir_all(&self._runtime_dir);
            let _ = std::fs::remove_dir_all(&self.replay_dir);
            let _ = std::fs::remove_file(&self.trail_path);
        }
    }

    // ===================================================================
    // C3-A1 Tests: Minimal Bounded Launch Window
    // ===================================================================

    #[test]
    fn c3_a1_n1_limits_active_invocations_to_at_most_one() {
        // Group of 3 members (a, b, c).
        // With N=1, only ONE provider is active at any time.
        let h = C3A1GroupHarness::new("n1-bound", &["a", "b", "c"]);
        h.set_peer_count(1);

        std::thread::scope(|s| {
            let handle = s.spawn(|| h.run_group_with_limit("eval-c3-n1", 1));

            // 1. Member A launches and enters active state.
            h.wait_member_active("a");

            // While A is active at the provider boundary:
            // A has not yet completed Stage C (outcome not in Trail).
            // Active count is exactly 1.
            // B and C have NOT been launched (entered files do not exist).
            assert_eq!(
                h.currently_active_count(),
                1,
                "N=1 must have exactly 1 active member"
            );
            assert!(
                !h.has_member_outcome("a"),
                "member-a outcome must not exist while in-flight"
            );
            assert!(!h.has_entered("b"), "member-b must wait for capacity");
            assert!(!h.has_entered("c"), "member-c must wait for capacity");

            // Release A and wait for its completion in Trail.
            h.release_member("a");
            h.wait_member_outcome("a");

            // 2. Member B launches after A completes.
            h.wait_member_active("b");
            assert_eq!(
                h.currently_active_count(),
                1,
                "N=1 must have exactly 1 active member"
            );
            assert!(
                !h.has_member_outcome("b"),
                "member-b outcome must not exist while in-flight"
            );
            assert!(!h.has_entered("c"), "member-c must wait for capacity");

            // Release B and wait for its completion in Trail.
            h.release_member("b");
            h.wait_member_outcome("b");

            // 3. Member C launches after B completes.
            h.wait_member_active("c");
            assert_eq!(
                h.currently_active_count(),
                1,
                "N=1 must have exactly 1 active member"
            );
            assert!(
                !h.has_member_outcome("c"),
                "member-c outcome must not exist while in-flight"
            );

            // Release C and wait for its completion in Trail.
            h.release_member("c");

            let result = handle.join().expect("group must complete without panic");
            assert!(
                matches!(result, ExecutionServiceResult::Completed { .. }),
                "expected Completed, got {result:?}"
            );
        });

        // Verify all 3 members produced outcomes in Trail in semantic order and GroupJoin occurred.
        let ids = h.outcome_ids();
        assert_eq!(ids.len(), 3);
        assert_eq!(ids, vec!["member-a", "member-b", "member-c"]);
        assert_eq!(h.entry_kinds().last(), Some(&"group_join".to_owned()));
    }

    #[test]
    fn c3_a1_n2_limits_active_invocations_to_at_most_two_and_reaches_two() {
        // Group of 3 members (a, b, c).
        // With N=2, at most 2 providers are active simultaneously, and 2 is reached.
        let h = C3A1GroupHarness::new("n2-bound", &["a", "b", "c"]);
        h.set_peer_count(2);

        std::thread::scope(|s| {
            let handle = s.spawn(|| h.run_group_with_limit("eval-c3-n2", 2));

            // Wait until both A and B become active simultaneously.
            h.wait_barrier_active_count(2);

            // Verify both A and B are currently active simultaneously (count is 2).
            assert_eq!(
                h.currently_active_count(),
                2,
                "N=2 must reach exactly 2 active members simultaneously"
            );
            assert!(!h.has_member_outcome("a"), "member-a must be in-flight");
            assert!(!h.has_member_outcome("b"), "member-b must be in-flight");
            assert!(!h.has_entered("c"), "member-c must wait for capacity");

            // Release B only and wait for B's outcome in Trail.
            h.release_member("b");
            h.wait_member_outcome("b");

            // With B complete, a slot opened, so C launches and enters active state.
            h.wait_member_active("c");

            // A is still in-flight, C is in-flight: exactly 2 currently active.
            assert_eq!(
                h.currently_active_count(),
                2,
                "N=2 must have exactly 2 active members simultaneously (A and C)"
            );
            assert!(
                !h.has_member_outcome("a"),
                "member-a must still be in-flight"
            );
            assert!(!h.has_member_outcome("c"), "member-c must be in-flight");

            // Release A and C.
            h.release_member("a");
            h.release_member("c");

            let result = handle.join().expect("group must complete without panic");
            assert!(
                matches!(result, ExecutionServiceResult::Completed { .. }),
                "expected Completed, got {result:?}"
            );
        });

        let ids = h.outcome_ids();
        assert_eq!(ids.len(), 3);
        assert!(ids.contains(&"member-a".to_owned()));
        assert!(ids.contains(&"member-b".to_owned()));
        assert!(ids.contains(&"member-c".to_owned()));
        assert_eq!(h.entry_kinds().last(), Some(&"group_join".to_owned()));
    }

    #[test]
    fn c3_a1_full_width_preserves_full_overlap() {
        // Group of 3 members (a, b, c) with N=3.
        // All 3 members must overlap simultaneously.
        let h = C3A1GroupHarness::new("full-width", &["a", "b", "c"]);
        h.set_peer_count(3);

        std::thread::scope(|s| {
            let handle = s.spawn(|| h.run_group_with_limit("eval-c3-full", 3));

            // Wait until all 3 members reach active simultaneously.
            h.wait_barrier_active_count(3);
            assert_eq!(
                h.currently_active_count(),
                3,
                "full width must reach 3 active members simultaneously"
            );
            assert!(!h.has_member_outcome("a"));
            assert!(!h.has_member_outcome("b"));
            assert!(!h.has_member_outcome("c"));

            // Release all at once.
            h.release_all();

            let result = handle.join().expect("group must complete without panic");
            assert!(
                matches!(result, ExecutionServiceResult::Completed { .. }),
                "expected Completed, got {result:?}"
            );
        });

        let ids = h.outcome_ids();
        assert_eq!(ids.len(), 3);
        assert_eq!(h.entry_kinds().last(), Some(&"group_join".to_owned()));
    }

    // ===================================================================
    // C3-V1 Proof Gap: Group-of-Five Matrix
    // ===================================================================

    #[test]
    fn c3_v1_n1_group_of_five_proves_bound_and_full_terminalisation() {
        // Frozen design §14.1: N=1, group=5.
        // - max active exactly 1 at all times
        // - every eligible member eventually invokes
        // - all five terminal
        // - join evaluates all five
        let h = C3A1GroupHarness::new("v1-n1-g5", &["a", "b", "c", "d", "e"]);
        h.set_peer_count(1);

        std::thread::scope(|s| {
            let handle = s.spawn(|| h.run_group_with_limit("eval-v1-n1-g5", 1));

            // 1. Member A launches first.
            h.wait_member_active("a");
            assert_eq!(
                h.currently_active_count(),
                1,
                "N=1 must have exactly 1 active member when A is active"
            );
            assert!(!h.has_member_outcome("a"), "A must be in-flight");
            assert!(!h.has_entered("b"), "B must wait for capacity");
            assert!(!h.has_entered("c"), "C must wait for capacity");
            assert!(!h.has_entered("d"), "D must wait for capacity");
            assert!(!h.has_entered("e"), "E must wait for capacity");

            // GroupJoin must NOT exist while A is active and siblings waiting.
            assert!(
                !h.entry_kinds().contains(&"group_join".to_owned()),
                "GroupJoin must not exist while A is active"
            );

            // Release A and wait for its durable outcome.
            h.release_member("a");
            h.wait_member_outcome("a");

            // 2. Member B launches after A completes.
            h.wait_member_active("b");
            assert_eq!(
                h.currently_active_count(),
                1,
                "N=1 must have exactly 1 active member when B is active"
            );
            assert!(!h.has_member_outcome("b"), "B must be in-flight");
            assert!(!h.has_entered("c"), "C must wait for capacity");
            assert!(!h.has_entered("d"), "D must wait for capacity");
            assert!(!h.has_entered("e"), "E must wait for capacity");

            // GroupJoin must NOT exist while B is active.
            assert!(
                !h.entry_kinds().contains(&"group_join".to_owned()),
                "GroupJoin must not exist while B is active"
            );

            // Release B and wait for its durable outcome.
            h.release_member("b");
            h.wait_member_outcome("b");

            // 3. Member C launches after B completes.
            h.wait_member_active("c");
            assert_eq!(
                h.currently_active_count(),
                1,
                "N=1 must have exactly 1 active member when C is active"
            );
            assert!(!h.has_member_outcome("c"), "C must be in-flight");
            assert!(!h.has_entered("d"), "D must wait for capacity");
            assert!(!h.has_entered("e"), "E must wait for capacity");

            // Release C and wait for its durable outcome.
            h.release_member("c");
            h.wait_member_outcome("c");

            // 4. Member D launches after C completes.
            h.wait_member_active("d");
            assert_eq!(
                h.currently_active_count(),
                1,
                "N=1 must have exactly 1 active member when D is active"
            );
            assert!(!h.has_member_outcome("d"), "D must be in-flight");
            assert!(!h.has_entered("e"), "E must wait for capacity");

            // Release D and wait for its durable outcome.
            h.release_member("d");
            h.wait_member_outcome("d");

            // 5. Member E launches after D completes.
            h.wait_member_active("e");
            assert_eq!(
                h.currently_active_count(),
                1,
                "N=1 must have exactly 1 active member when E is active"
            );
            assert!(!h.has_member_outcome("e"), "E must be in-flight");

            // Release E and allow completion.
            h.release_member("e");

            let result = handle.join().expect("group must complete without panic");
            assert!(
                matches!(result, ExecutionServiceResult::Completed { .. }),
                "expected Completed, got {result:?}"
            );
        });

        // All five members must have durable outcomes.
        let ids = h.outcome_ids();
        assert_eq!(ids.len(), 5, "exactly 5 durable member outcomes required");
        assert_eq!(
            ids,
            vec!["member-a", "member-b", "member-c", "member-d", "member-e"],
            "outcomes must be in semantic order"
        );

        // Successful GroupJoin must exist.
        let kinds = h.entry_kinds();
        assert_eq!(
            kinds.last(),
            Some(&"group_join".to_owned()),
            "GroupJoin must be the final entry"
        );
    }

    #[test]
    fn c3_v1_n2_group_of_five_proves_bound_reached_and_full_terminalisation() {
        // Frozen design §14.2: N=2, group=5.
        // - max active never exceeds 2
        // - observed max reaches 2
        // - every eligible member invokes
        // - all five terminal
        let h = C3A1GroupHarness::new("v1-n2-g5", &["a", "b", "c", "d", "e"]);
        h.set_peer_count(2);

        std::thread::scope(|s| {
            let handle = s.spawn(|| h.run_group_with_limit("eval-v1-n2-g5", 2));

            // 1. A and B become simultaneously active.
            h.wait_barrier_active_count(2);
            assert_eq!(
                h.currently_active_count(),
                2,
                "N=2 must reach exactly 2 active members (A and B)"
            );
            assert!(!h.has_member_outcome("a"), "A must be in-flight");
            assert!(!h.has_member_outcome("b"), "B must be in-flight");
            assert!(!h.has_entered("c"), "C must wait for capacity");
            assert!(!h.has_entered("d"), "D must wait for capacity");
            assert!(!h.has_entered("e"), "E must wait for capacity");

            // GroupJoin must NOT exist while A+B active and siblings waiting.
            assert!(
                !h.entry_kinds().contains(&"group_join".to_owned()),
                "GroupJoin must not exist while A+B active"
            );

            // Release B, wait for B's durable outcome.
            h.release_member("b");
            h.wait_member_outcome("b");

            // 2. C launches while A remains active.
            h.wait_member_active("c");
            assert_eq!(
                h.currently_active_count(),
                2,
                "N=2 must have exactly 2 active (A and C)"
            );
            assert!(!h.has_member_outcome("a"), "A must still be in-flight");
            assert!(!h.has_member_outcome("c"), "C must be in-flight");
            assert!(!h.has_entered("d"), "D must wait for capacity");
            assert!(!h.has_entered("e"), "E must wait for capacity");

            // GroupJoin must still NOT exist.
            assert!(
                !h.entry_kinds().contains(&"group_join".to_owned()),
                "GroupJoin must not exist while A+C active"
            );

            // Release C, wait for C's durable outcome.
            h.release_member("c");
            h.wait_member_outcome("c");

            // 3. D launches while A remains active.
            h.wait_member_active("d");
            assert_eq!(
                h.currently_active_count(),
                2,
                "N=2 must have exactly 2 active (A and D)"
            );
            assert!(!h.has_member_outcome("a"), "A must still be in-flight");
            assert!(!h.has_member_outcome("d"), "D must be in-flight");
            assert!(!h.has_entered("e"), "E must wait for capacity");

            // Release D, wait for D's durable outcome.
            h.release_member("d");
            h.wait_member_outcome("d");

            // 4. E launches while A remains active.
            h.wait_member_active("e");
            assert_eq!(
                h.currently_active_count(),
                2,
                "N=2 must have exactly 2 active (A and E)"
            );
            assert!(!h.has_member_outcome("a"), "A must still be in-flight");
            assert!(!h.has_member_outcome("e"), "E must be in-flight");

            // Release A and E.
            h.release_member("a");
            h.release_member("e");

            let result = handle.join().expect("group must complete without panic");
            assert!(
                matches!(result, ExecutionServiceResult::Completed { .. }),
                "expected Completed, got {result:?}"
            );
        });

        // All five members must have durable outcomes.
        let ids = h.outcome_ids();
        assert_eq!(ids.len(), 5, "exactly 5 durable member outcomes required");
        assert!(ids.contains(&"member-a".to_owned()), "A must have outcome");
        assert!(ids.contains(&"member-b".to_owned()), "B must have outcome");
        assert!(ids.contains(&"member-c".to_owned()), "C must have outcome");
        assert!(ids.contains(&"member-d".to_owned()), "D must have outcome");
        assert!(ids.contains(&"member-e".to_owned()), "E must have outcome");

        // Successful GroupJoin must exist.
        let kinds = h.entry_kinds();
        assert_eq!(
            kinds.last(),
            Some(&"group_join".to_owned()),
            "GroupJoin must be the final entry"
        );
    }

    // ===================================================================
    // C3-A2 Proof Tests: Resource / Deadline / G1 Crucible
    // ===================================================================

    #[test]
    fn c3_a2_waiting_member_has_g0_without_g1_or_provider_effect() {
        // N=1 with members A, B.
        // A occupies the only capacity slot.
        // B is prepared (G0) but waiting for capacity.
        // While A is active: B has G0, no G1, no provider effect.
        let h = C3A1GroupHarness::new("c3a2-g0-no-g1", &["a", "b"]);
        h.set_peer_count(1);

        let shared_trace = ReplayTrace::new();

        std::thread::scope(|s| {
            let handle = s.spawn(|| h.run_group_with_live_trace("eval-c3a2-g0", 1, &shared_trace));

            // Wait for A to physically enter the provider barrier.
            h.wait_member_active("a");

            // While A is active, snapshot the live replay trace.
            // B's G0 must already be recorded (published during Stage A).
            // B's G1 must NOT be recorded (B has not been launched).
            let live = shared_trace.snapshot();
            assert!(
                live.contains(&"g0:member-b".to_owned()),
                "B must have G0 while A is active (live snapshot: {live:?})"
            );
            assert!(
                !live.contains(&"g1:member-b".to_owned()),
                "B must NOT have G1 while A is active (live snapshot: {live:?})"
            );

            // While A is active:
            // - A has entered the provider (entered-member-a exists)
            assert!(
                h.has_entered("a"),
                "member-a must have entered provider invocation"
            );
            // - B has NOT entered the provider
            assert!(
                !h.has_entered("b"),
                "member-b must NOT have entered provider while waiting"
            );
            // - B has no outcome in Trail
            assert!(
                !h.has_member_outcome("b"),
                "member-b must have no outcome while waiting"
            );
            // - Active count is exactly 1
            assert_eq!(
                h.currently_active_count(),
                1,
                "N=1 must have exactly 1 active member"
            );

            // B's durable Trail intent must already be present while A is active.
            let trail_while_active = h.trail_content();
            assert!(
                trail_while_active.contains("member-b"),
                "B's durable Trail intent must exist while A is active"
            );

            // Release A and wait for its durable outcome.
            h.release_member("a");
            h.wait_member_outcome("a");

            // Now B should launch and enter the provider.
            h.wait_member_active("b");
            h.release_member("b");

            let (result, trace) = handle.join().expect("group must complete without panic");
            assert!(
                matches!(result, ExecutionServiceResult::Completed { .. }),
                "expected Completed, got {result:?}"
            );

            // Final replay trace assertions:
            // G0 for both members must exist.
            assert!(trace.contains(&"g0:member-a".to_owned()), "A must have G0");
            assert!(trace.contains(&"g0:member-b".to_owned()), "B must have G0");
            // G1 for A must exist (A was launched).
            assert!(trace.contains(&"g1:member-a".to_owned()), "A must have G1");
            // G1 for B must exist (B was eventually launched after A completed).
            assert!(trace.contains(&"g1:member-b".to_owned()), "B must have G1");
            // G2 for both must exist (both completed Stage C).
            assert!(trace.contains(&"g2:member-a".to_owned()), "A must have G2");
            assert!(trace.contains(&"g2:member-b".to_owned()), "B must have G2");

            // Prove ordering: G0(B) must precede G1(B) in the trace.
            let g0_b_pos = trace.iter().position(|e| e == "g0:member-b").unwrap();
            let g1_b_pos = trace.iter().position(|e| e == "g1:member-b").unwrap();
            assert!(
                g0_b_pos < g1_b_pos,
                "G0(B) must precede G1(B): g0 at {g0_b_pos}, g1 at {g1_b_pos}"
            );

            // Prove: G1(B) occurs AFTER G2(A) — B only launches after A completes Stage C.
            let g2_a_pos = trace.iter().position(|e| e == "g2:member-a").unwrap();
            assert!(
                g2_a_pos < g1_b_pos,
                "G2(A) must precede G1(B): g2_a at {g2_a_pos}, g1_b at {g1_b_pos}"
            );
        });

        // Trail: both members have outcomes, GroupJoin present.
        let ids = h.outcome_ids();
        assert_eq!(ids.len(), 2, "expected 2 outcomes in Trail");
        assert!(ids.contains(&"member-a".to_owned()));
        assert!(ids.contains(&"member-b".to_owned()));
        assert_eq!(h.entry_kinds().last(), Some(&"group_join".to_owned()));
    }

    #[test]
    fn c3_a2_queue_wait_does_not_consume_provider_timeout() {
        // N=1 with members A, B.
        // B has a deliberately short timeout (500ms) applied BEFORE prepare_runtime.
        // A occupies the slot for 1500ms > B's 500ms timeout.
        // B must NOT be classified Unattempted due to queue wait.
        let mut timeout_overrides = HashMap::new();
        timeout_overrides.insert("b".to_owned(), 500u64);
        let h = C3A1GroupHarness::new_with_timeout_overrides(
            "c3a2-queue-timeout",
            &["a", "b"],
            &timeout_overrides,
        );
        h.set_peer_count(1);

        // Verify the prepared runtime actually contains B's overridden timeout.
        let provider_b = h
            .runtime
            .providers()
            .iter()
            .find(|p| p.identity == "provider-b")
            .expect("provider-b must exist in prepared runtime");
        let effective_timeout_ms = provider_b.capabilities[0]
            .verified_manifest
            .manifest()
            .timeout_ms;
        assert_eq!(
            effective_timeout_ms, 500,
            "prepared runtime must contain B's overridden timeout of 500ms, got {effective_timeout_ms}ms"
        );

        std::thread::scope(|s| {
            let handle = s.spawn(|| h.run_group_with_trace("eval-c3a2-timeout", 1));

            // A launches and enters the barrier.
            h.wait_member_active("a");

            // Hold A active for longer than B's configured timeout (500ms).
            // Use a safe 1500ms wait to ensure the duration exceeds B's timeout.
            std::thread::sleep(Duration::from_millis(1500));

            // While A is still active (after waiting > B's timeout):
            assert!(!h.has_entered("b"), "B must still be waiting for capacity");
            assert!(
                !h.has_member_outcome("b"),
                "B must have no outcome while waiting"
            );

            // Release A and wait for its durable terminal outcome.
            h.release_member("a");
            h.wait_member_outcome("a");

            // B must now launch with a fresh deadline.
            h.wait_member_active("b");

            // B has entered the provider — it was NOT classified Unattempted.
            assert!(
                h.has_entered("b"),
                "B must have entered provider invocation (not Unattempted)"
            );

            // Release B and allow normal terminalisation.
            h.release_member("b");

            let (result, trace) = handle.join().expect("group must complete without panic");
            assert!(
                matches!(result, ExecutionServiceResult::Completed { .. }),
                "expected Completed, got {result:?}"
            );

            // B's G1 must exist — it was launched after queue wait.
            assert!(trace.contains(&"g1:member-b".to_owned()), "B must have G1");
            // B's G2 must exist — it completed Stage C.
            assert!(trace.contains(&"g2:member-b".to_owned()), "B must have G2");
        });

        // Trail: both members completed. B is NOT Unattempted.
        // Use structured outcome assertion, not just string search.
        let ids = h.outcome_ids();
        assert_eq!(ids.len(), 2, "expected 2 outcomes in Trail");
        assert!(
            ids.contains(&"member-b".to_owned()),
            "member-b must have a terminal outcome (not Unattempted)"
        );
        assert_eq!(h.entry_kinds().last(), Some(&"group_join".to_owned()));
    }

    #[test]
    fn c3_a2_next_slot_launches_earliest_semantic_waiter() {
        // A B C with N=1.
        // A launches first. B and C wait.
        // After A completes, B must launch next (earliest semantic waiter).
        // While B is active, C must still NOT have entered.
        // After B completes, C launches.
        let h = C3A1GroupHarness::new("c3a2-semantic-order", &["a", "b", "c"]);
        h.set_peer_count(1);

        std::thread::scope(|s| {
            let handle = s.spawn(|| h.run_group_with_trace("eval-c3a2-order", 1));

            // A launches and enters the barrier.
            h.wait_member_active("a");

            // While A is active: B and C must NOT have entered.
            assert!(!h.has_entered("b"), "B must wait while A is active");
            assert!(!h.has_entered("c"), "C must wait while A is active");
            assert_eq!(h.currently_active_count(), 1);

            // Release A and wait for its durable OutcomeEntry.
            h.release_member("a");
            h.wait_member_outcome("a");

            // B must launch next (earliest semantic waiter).
            h.wait_member_active("b");

            // While B is active: C must still NOT have entered.
            assert!(!h.has_entered("c"), "C must wait while B is active");
            assert_eq!(h.currently_active_count(), 1);

            // Release B and wait for its durable outcome.
            h.release_member("b");
            h.wait_member_outcome("b");

            // C must launch next.
            h.wait_member_active("c");
            assert_eq!(h.currently_active_count(), 1);

            // Release C and allow normal terminalisation.
            h.release_member("c");

            let (result, _trace) = handle.join().expect("group must complete without panic");
            assert!(
                matches!(result, ExecutionServiceResult::Completed { .. }),
                "expected Completed, got {result:?}"
            );
        });

        // All three members produced outcomes in semantic order.
        let ids = h.outcome_ids();
        assert_eq!(ids.len(), 3, "expected 3 outcomes in Trail");
        assert_eq!(
            ids,
            vec!["member-a", "member-b", "member-c"],
            "outcomes must be in semantic order"
        );
        assert_eq!(h.entry_kinds().last(), Some(&"group_join".to_owned()));
    }

    #[test]
    fn c3_a2_queued_member_replay_order_is_g0_wait_g1_g2() {
        // N=1 with members A, B.
        // Prove replay event ordering for the queued B path:
        // G0(B) → capacity wait → A durable terminal → G1(B) → provider B → G2(B)
        let h = C3A1GroupHarness::new("c3a2-replay-order", &["a", "b"]);
        h.set_peer_count(1);

        std::thread::scope(|s| {
            let handle = s.spawn(|| h.run_group_with_trace("eval-c3a2-replay", 1));

            // A launches first.
            h.wait_member_active("a");

            // B is waiting: has G0, no G1.
            assert!(!h.has_entered("b"), "B must be waiting for capacity");

            // Release A and wait for its durable terminal outcome.
            h.release_member("a");
            h.wait_member_outcome("a");

            // After A's Stage C completes, B must launch.
            h.wait_member_active("b");
            h.release_member("b");

            let (result, trace) = handle.join().expect("group must complete without panic");
            assert!(
                matches!(result, ExecutionServiceResult::Completed { .. }),
                "expected Completed, got {result:?}"
            );

            // Find positions in the replay trace.
            let pos = |needle: &str| -> usize {
                trace
                    .iter()
                    .position(|e| e == needle)
                    .unwrap_or_else(|| panic!("{needle} not found in replay trace"))
            };

            let g0_b = pos("g0:member-b");
            let g1_b = pos("g1:member-b");
            let g2_b = pos("g2:member-b");
            let g2_a = pos("g2:member-a");

            // Strict ordering: G0(B) < G2(A) < G1(B) < G2(B)
            // G0(B) is published during Stage A (before capacity wait).
            // G2(A) is published when A completes Stage C (capacity released).
            // G1(B) is published when B is launched (after capacity available).
            // G2(B) is published when B completes Stage C.
            assert!(
                g0_b < g2_a,
                "G0(B) ({g0_b}) must precede G2(A) ({g2_a}): B was prepared before A completed"
            );
            assert!(
                g2_a < g1_b,
                "G2(A) ({g2_a}) must precede G1(B) ({g1_b}): A completes Stage C before B launches"
            );
            assert!(
                g1_b < g2_b,
                "G1(B) ({g1_b}) must precede G2(B) ({g2_b}): B is armed before B completes"
            );

            // Verify G0(B) precedes G1(B) (the core G0 → capacity wait → G1 invariant).
            assert!(g0_b < g1_b, "G0(B) ({g0_b}) must precede G1(B) ({g1_b})");
        });

        // Trail: both members completed, GroupJoin present.
        let ids = h.outcome_ids();
        assert_eq!(ids.len(), 2);
        assert_eq!(h.entry_kinds().last(), Some(&"group_join".to_owned()));
    }

    // ===================================================================
    // C3-A3 Tests: Failure Boundaries and Fatal-Halt Guard
    // ===================================================================

    #[test]
    fn c3_a3_normal_provider_failure_releases_slot_and_joins() {
        // N=1 with members A, B.
        // A configured for provider outcome = Failed.
        // B configured for provider outcome = Success.
        // Prove:
        // - A launches and enters provider
        // - A returns provider Failed
        // - A completes Stage C and terminalises as Failed
        // - Capacity is released (active_count decreases)
        // - B launches and succeeds
        // - Both members terminalise
        // - GroupJoin is present and joined=false
        // - Final semantic non-success is A's Failed
        let h = C3A1GroupHarness::new("c3a3-normal-fail", &["a", "b"]);
        h.set_peer_count(1);
        h.set_member_outcome("a", "failed");

        std::thread::scope(|s| {
            let handle = s.spawn(|| h.run_group_with_trace("eval-c3a3-normfail", 1));

            // A launches and enters active state at provider.
            h.wait_member_active("a");

            // While A is in-flight, B has not entered.
            assert!(!h.has_entered("b"), "B must wait for capacity");

            // Release A and wait for its completion in Trail.
            h.release_member("a");
            h.wait_member_outcome("a");

            // After A terminalises, slot is released and B launches.
            h.wait_member_active("b");
            h.release_member("b");

            let (result, trace) = handle.join().expect("group must complete");

            // A's Failed result is selected as final group non-success.
            assert!(
                matches!(
                    result,
                    ExecutionServiceResult::Failed {
                        ref action_id,
                        ..
                    } if action_id == "member-a"
                ),
                "expected Failed for member-a, got {result:?}"
            );

            // Both entered provider.
            assert!(h.has_entered("a"));
            assert!(h.has_entered("b"));

            // Both have outcomes in Trail.
            let outcome_ids = h.outcome_ids();
            assert_eq!(outcome_ids, vec!["member-a", "member-b"]);

            // Replay trace shows G0, G1, G2 for both members in order.
            assert!(trace.contains(&"g0:member-a".to_string()));
            assert!(trace.contains(&"g0:member-b".to_string()));
            assert!(trace.contains(&"g1:member-a".to_string()));
            assert!(trace.contains(&"g2:member-a".to_string()));
            assert!(trace.contains(&"g1:member-b".to_string()));
            assert!(trace.contains(&"g2:member-b".to_string()));

            // GroupJoin entry IS present.
            assert_eq!(h.entry_kinds().last(), Some(&"group_join".to_owned()));
            let trail_content = h.trail_content();
            assert!(trail_content.contains("\"joined\":false"));
        });
    }

    #[test]
    fn c3_a3_worker_panic_terminalises_uncertain_and_releases_slot() {
        // N=1 with members A, B.
        // Inject worker panic for A (action_index=0).
        // Prove:
        // - A panic is caught via PanicGuard
        // - A terminalises as Uncertain
        // - Slot is released
        // - B launches and succeeds
        // - GroupJoin is present and joined=false
        // - Final semantic non-success is A's Uncertain
        let h = C3A1GroupHarness::new("c3a3-panic-slot", &["a", "b"]);
        h.set_peer_count(1);

        let _guard = PanicGuard::target(0);

        std::thread::scope(|s| {
            let handle = s.spawn(|| h.run_group_with_trace("eval-c3a3-panic", 1));

            // Wait for A's outcome in Trail (A panics in worker thread and terminalises as Uncertain).
            h.wait_member_outcome("a");

            // After A terminalises as Uncertain, capacity is released and B launches.
            h.wait_member_active("b");
            h.release_member("b");

            let (result, trace) = handle
                .join()
                .expect("group must complete without coordinator hang");

            // A's Uncertain result is selected as final group non-success.
            assert!(
                matches!(
                    result,
                    ExecutionServiceResult::Uncertain {
                        ref action_id,
                        ..
                    } if action_id == "member-a"
                ),
                "expected Uncertain for member-a, got {result:?}"
            );

            // B entered provider and succeeded.
            assert!(h.has_entered("b"));

            // Both have outcomes in Trail.
            let outcome_ids = h.outcome_ids();
            assert_eq!(outcome_ids, vec!["member-a", "member-b"]);

            // Replay trace shows G2 for both members.
            assert!(trace.contains(&"g2:member-a".to_string()));
            assert!(trace.contains(&"g2:member-b".to_string()));

            // GroupJoin entry IS present with joined=false.
            assert_eq!(h.entry_kinds().last(), Some(&"group_join".to_owned()));
            let trail_content = h.trail_content();
            assert!(trail_content.contains("\"joined\":false"));
        });
    }

    #[test]
    fn c3_a3_outcome_durability_failure_halts_queued_effects_without_join() {
        // N=1 with members A, B, C.
        // Inject failure of A's Stage C OutcomeEntry durability via RecordingTrail.
        // Prove:
        // - A launches and enters provider
        // - A returns to coordinator
        // - Stage C outcome durability fails
        // - No G2 for A (boundary stops before G2)
        // - B and C NEVER enter provider
        // - No new provider effect launches
        // - No GroupJoinEntry is appended (B and C remain Prepared)
        // - Result fails closed through AuditFailed
        let h = C3A1GroupHarness::new("c3a3-durability-fail", &["a", "b", "c"]);
        h.set_peer_count(1);

        let trace = ReplayTrace::new();

        std::thread::scope(|s| {
            let trace_clone = trace.clone();
            let h_ref = &h;
            let handle = s.spawn(move || {
                let mut trail = dispatch::RecordingTrail::new();
                trail.injected_outcome_error = Some(dispatch::TrailError::WriteFailed(
                    "injected OutcomeEntry durability failure".to_owned(),
                ));
                let mut replay_authority = ObservingReplayAuthority::with_trace(&[], trace_clone);
                let result = h_ref.run_group_with_trail_and_authority(
                    "eval-c3a3-durability",
                    1,
                    &mut trail,
                    &mut replay_authority,
                );
                (result, trail.outcome_entries, trail.group_join_entries)
            });

            // A launches and reaches provider.
            h.wait_member_active("a");

            // B and C have not entered.
            assert!(!h.has_entered("b"), "B must not have entered");
            assert!(!h.has_entered("c"), "C must not have entered");

            // Release A so provider returns to coordinator.
            h.release_member("a");

            let (result, outcome_entries, group_join_entries) =
                handle.join().expect("group must complete");

            // Result fails closed as AuditFailed for member-a.
            assert!(
                matches!(
                    result,
                    ExecutionServiceResult::AuditFailed {
                        ref action_id,
                        ..
                    } if action_id == "member-a"
                ),
                "expected AuditFailed for member-a, got {result:?}"
            );

            // A reached provider effect.
            assert!(h.has_entered("a"), "A must have reached provider");

            // B and C NEVER entered provider.
            assert!(
                !h.has_entered("b"),
                "B must never enter provider after fatal halt"
            );
            assert!(
                !h.has_entered("c"),
                "C must never enter provider after fatal halt"
            );

            // Replay trace: G0 for all 3, G1 for A, NO G2 for A, NO G1 for B/C.
            let snapshot = trace.snapshot();
            assert!(snapshot.contains(&"g0:member-a".to_string()));
            assert!(snapshot.contains(&"g0:member-b".to_string()));
            assert!(snapshot.contains(&"g0:member-c".to_string()));
            assert!(snapshot.contains(&"g1:member-a".to_string()));
            assert!(
                !snapshot.contains(&"g2:member-a".to_string()),
                "G2 must not occur when outcome durability fails"
            );
            assert!(
                !snapshot.contains(&"g1:member-b".to_string()),
                "B must never get G1"
            );
            assert!(
                !snapshot.contains(&"g1:member-c".to_string()),
                "C must never get G1"
            );

            // No GroupJoinEntry was appended to trail.
            assert!(
                group_join_entries.is_empty(),
                "GroupJoinEntry must not be appended when members remain nonterminal"
            );
            assert!(
                outcome_entries.is_empty(),
                "OutcomeEntry must not be durably recorded when append fails"
            );
        });
    }

    #[test]
    fn c3_a3_g2_failure_halts_queued_effects_without_join() {
        // N=1 with members A, B, C.
        // Inject G2 publish_terminal failure for member A.
        // Prove:
        // - G0(A), G0(B), G0(C)
        // - G1(A)
        // - A enters provider and returns
        // - Durable OutcomeEntry occurs before attempted G2
        // - Attempted G2 fails
        // - B NEVER gets G1, NEVER enters provider
        // - C NEVER gets G1, NEVER enters provider
        // - No GroupJoinEntry is appended
        // - Result fails closed through ReplayPersistenceUnavailable
        let h = C3A1GroupHarness::new("c3a3-g2-fail", &["a", "b", "c"]);
        h.set_peer_count(1);

        let trace = ReplayTrace::new();
        let mut replay_authority =
            ObservingReplayAuthority::with_fail_points(trace.clone(), &[], &["member-a"]);

        std::thread::scope(|s| {
            let handle = s
                .spawn(|| h.run_group_with_authority("eval-c3a3-g2fail", 1, &mut replay_authority));

            // A launches and enters provider.
            h.wait_member_active("a");

            // B and C have not entered.
            assert!(!h.has_entered("b"), "B must not have entered");
            assert!(!h.has_entered("c"), "C must not have entered");

            // Release A so provider returns.
            h.release_member("a");

            let result = handle.join().expect("group must complete");

            // Result fails closed as ReplayPersistenceUnavailable for member-a.
            assert!(
                matches!(
                    result,
                    ExecutionServiceResult::ReplayPersistenceUnavailable {
                        ref action_id,
                        ..
                    } if action_id == "member-a"
                ),
                "expected ReplayPersistenceUnavailable for member-a, got {result:?}"
            );

            // A entered provider.
            assert!(h.has_entered("a"));

            // A's OutcomeEntry WAS written durably to Trail before G2.
            assert!(
                h.has_member_outcome("a"),
                "A OutcomeEntry must precede G2 attempt"
            );

            // B and C NEVER entered provider.
            assert!(!h.has_entered("b"), "B must never enter provider");
            assert!(!h.has_entered("c"), "C must never enter provider");

            // Replay trace: G0 for all, G1 for A, G2 failed for A, NO G1 for B/C.
            let snapshot = trace.snapshot();
            assert!(snapshot.contains(&"g0:member-a".to_string()));
            assert!(snapshot.contains(&"g0:member-b".to_string()));
            assert!(snapshot.contains(&"g0:member-c".to_string()));
            assert!(snapshot.contains(&"g1:member-a".to_string()));
            assert!(
                snapshot.contains(&"g2_fail:member-a".to_string()),
                "G2 failure must be observed"
            );
            assert!(
                !snapshot.contains(&"g1:member-b".to_string()),
                "B must never get G1"
            );
            assert!(
                !snapshot.contains(&"g1:member-c".to_string()),
                "C must never get G1"
            );

            // No GroupJoinEntry is in Trail.
            assert!(
                !h.entry_kinds().contains(&"group_join".to_owned()),
                "GroupJoinEntry must not be appended when members remain nonterminal"
            );
        });
    }

    #[test]
    fn c3_a3_g1_failure_halts_before_any_later_effect_without_join() {
        // N=1 with members A, B.
        // Inject G1 publish_armed failure for member A.
        // Prove:
        // - A has G0, B has G0
        // - G1 for A fails
        // - A NEVER enters provider
        // - B NEVER gets G1, NEVER enters provider
        // - No GroupJoinEntry is appended
        // - Result is ReplayPersistenceUnavailable for member A
        let h = C3A1GroupHarness::new("c3a3-g1-fail", &["a", "b"]);
        h.set_peer_count(1);

        let trace = ReplayTrace::new();
        let mut replay_authority =
            ObservingReplayAuthority::with_fail_points(trace.clone(), &["member-a"], &[]);

        std::thread::scope(|s| {
            let handle = s
                .spawn(|| h.run_group_with_authority("eval-c3a3-g1fail", 1, &mut replay_authority));

            let result = handle.join().expect("group must complete");

            // Result fails closed as ReplayPersistenceUnavailable for member-a.
            assert!(
                matches!(
                    result,
                    ExecutionServiceResult::ReplayPersistenceUnavailable {
                        ref action_id,
                        ..
                    } if action_id == "member-a"
                ),
                "expected ReplayPersistenceUnavailable for member-a, got {result:?}"
            );

            // A NEVER entered provider (failed before launch).
            assert!(
                !h.has_entered("a"),
                "A must never enter provider on G1 failure"
            );

            // B NEVER entered provider.
            assert!(!h.has_entered("b"), "B must never enter provider");

            // Trace: G0 for both, G1 failed for A, NO G1 for B.
            let snapshot = trace.snapshot();
            assert!(snapshot.contains(&"g0:member-a".to_string()));
            assert!(snapshot.contains(&"g0:member-b".to_string()));
            assert!(snapshot.contains(&"g1_fail:member-a".to_string()));
            assert!(
                !snapshot.contains(&"g1:member-b".to_string()),
                "B must never get G1"
            );

            // No GroupJoinEntry in Trail.
            assert!(
                !h.entry_kinds().contains(&"group_join".to_owned()),
                "GroupJoinEntry must not be appended when members remain nonterminal"
            );
        });
    }

    #[test]
    fn c3_a3_all_terminal_preserves_group_join() {
        // Prove that when every member reaches a legitimate terminal state
        // (both all-success and mixed), GroupJoinEntry and presentation are preserved.
        let h = C3A1GroupHarness::new("c3a3-all-term", &["a", "b"]);
        h.set_peer_count(1);

        std::thread::scope(|s| {
            let handle = s.spawn(|| h.run_group_with_trace("eval-c3a3-allterm", 1));

            h.wait_member_active("a");
            h.release_member("a");
            h.wait_member_outcome("a");

            h.wait_member_active("b");
            h.release_member("b");

            let (result, trace) = handle.join().expect("group must complete");
            assert!(
                matches!(result, ExecutionServiceResult::Completed { .. }),
                "expected Completed, got {result:?}"
            );

            assert!(trace.contains(&"g2:member-a".to_string()));
            assert!(trace.contains(&"g2:member-b".to_string()));

            // GroupJoin IS present in Trail with joined=true.
            assert_eq!(h.entry_kinds().last(), Some(&"group_join".to_owned()));
            let trail_content = h.trail_content();
            assert!(trail_content.contains("\"joined\":true"));
        });
    }

    #[test]
    fn c3_a3_n2_active_sibling_survives_fatal_halt_truthfully() {
        // N=2 with members B, A, C (semantic order: B=0, A=1, C=2).
        // B and A both launch (N=2). C waits.
        // A's OutcomeEntry durability fails (first append_outcome takes injected error).
        // B's OutcomeEntry durability succeeds (error already consumed).
        //
        // Regression proof: with the old code, B's truthful success would be
        // overwritten to AuditFailed because the entire response Trail still
        // contains A's audit_failure entry.  With the fix, only entries
        // appended by the current boundary call are inspected.
        //
        // Proves:
        // - B and A both enter provider (N=2)
        // - A's outcome durability fails → audit_failure, launches_halted
        // - C NEVER launches, NEVER gets G1
        // - B completes normal successful Stage C
        // - B's outcome is truthfully Completed (NOT AuditFailed)
        // - Group result identifies member-a (NOT member-b)
        // - No GroupJoinEntry (C remains nonterminal)
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;

        let h = C3A1GroupHarness::new("c3a3-n2-bac", &["b", "a", "c"]);
        h.set_peer_count(2);

        let trace = ReplayTrace::new();
        // Deterministic signal: set to true when A's append_outcome fails.
        let outcome_error_signal = Arc::new(AtomicBool::new(false));

        std::thread::scope(|s| {
            let trace_clone = trace.clone();
            let h_ref = &h;
            let signal_clone = Arc::clone(&outcome_error_signal);
            let handle = s.spawn(move || {
                let mut trail = dispatch::RecordingTrail::new();
                trail.injected_outcome_error = Some(dispatch::TrailError::WriteFailed(
                    "injected A OutcomeEntry durability failure".to_owned(),
                ));
                // Signal the main thread when append_outcome consumes the error.
                trail.outcome_error_signal = Some(signal_clone);
                let mut replay_authority = ObservingReplayAuthority::with_trace(&[], trace_clone);
                let result = h_ref.run_group_with_trail_and_authority(
                    "eval-c3a3-n2-bac",
                    2,
                    &mut trail,
                    &mut replay_authority,
                );
                (result, trail.outcome_entries, trail.group_join_entries)
            });

            // B and A both launch (N=2 capacity).
            h.wait_barrier_active_count(2);
            assert!(h.has_active("b"), "B must be active");
            assert!(h.has_active("a"), "A must be active");
            assert!(!h.has_entered("c"), "C must wait for capacity");

            // Release A first — A's outcome durability will fail (takes injected error).
            h.release_member("a");

            // Wait deterministically for A's fatal Stage C durability failure
            // to be observed by the coordinator.  The signal is set inside
            // append_outcome when injected_outcome_error is consumed.
            let deadline = std::time::Instant::now() + Duration::from_secs(15);
            while !outcome_error_signal.load(Ordering::SeqCst) {
                assert!(
                    std::time::Instant::now() < deadline,
                    "A's outcome error signal was not set within 15s"
                );
                std::thread::sleep(Duration::from_millis(10));
            }

            // Release B — B's outcome durability succeeds (error already consumed).
            h.release_member("b");

            let (result, outcome_entries, group_join_entries) =
                handle.join().expect("group must complete");

            // The group result MUST identify member-a (the fatal member),
            // NOT member-b (the successful active sibling).
            assert!(
                matches!(
                    result,
                    ExecutionServiceResult::AuditFailed {
                        ref action_id,
                        ..
                    } if action_id == "member-a"
                ),
                "expected AuditFailed for member-a, got {result:?}"
            );

            // B entered provider and has a truthful successful OutcomeEntry.
            assert!(h.has_entered("b"), "B must have entered provider");
            assert_eq!(
                outcome_entries.len(),
                1,
                "only B's outcome should be durably recorded (A's write failed)"
            );
            assert_eq!(outcome_entries[0].action_id, "member-b");
            assert_eq!(outcome_entries[0].status, "succeeded");

            // A's outcome durability failed — no outcome entry for A.
            assert!(
                !outcome_entries.iter().any(|e| e.action_id == "member-a"),
                "A's outcome must NOT be durably recorded"
            );

            // C NEVER launched.
            assert!(!h.has_entered("c"), "C must never have entered provider");

            // Replay trace: G0 for all 3, G1 for B and A, G2 for B only.
            let snapshot = trace.snapshot();
            assert!(snapshot.contains(&"g0:member-b".to_string()));
            assert!(snapshot.contains(&"g0:member-a".to_string()));
            assert!(snapshot.contains(&"g0:member-c".to_string()));
            assert!(snapshot.contains(&"g1:member-b".to_string()));
            assert!(snapshot.contains(&"g1:member-a".to_string()));
            assert!(
                snapshot.contains(&"g2:member-b".to_string()),
                "B's G2 must succeed"
            );
            assert!(
                !snapshot.contains(&"g1:member-c".to_string()),
                "C must never get G1"
            );

            // No GroupJoinEntry because C remains nonterminal.
            assert!(
                group_join_entries.is_empty(),
                "GroupJoinEntry must not be appended when members remain nonterminal"
            );
        });
    }

    // ==================================================================
    // C3-A4 Tests: External Bounded-Concurrency Configuration
    //
    // These tests prove that the configuration value reaches the actual
    // production `execute_group_concurrent` wrapper, not merely the
    // `PreparedRuntime` accessor.
    // ==================================================================

    /// Build a C3A1-style harness with an explicit `max_active_together_invocations`
    /// in the config, then run `execute_group_concurrent` (the wrapper) to prove
    /// the configured value controls real production launch behavior.
    fn c3a4_harness_with_config(
        test_name: &str,
        member_tags: &[&str],
        max_active: Option<usize>,
    ) -> C3A1GroupHarness {
        use std::path::Path;

        let barrier_dir =
            std::env::temp_dir().join(format!("tethers-c3a4-{test_name}-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&barrier_dir).unwrap();

        let runtime_dir = std::env::temp_dir().join(format!(
            "tethers-c3a4-{test_name}-rt-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(runtime_dir.join("tethers")).unwrap();
        std::fs::create_dir_all(runtime_dir.join("manifests")).unwrap();

        std::fs::write(
            runtime_dir.join("tethers/together-test.tether"),
            "when event.test if true do fixture.ping-a do fixture.ping-b",
        )
        .unwrap();

        let manifest_template =
            include_str!("../../protocol/capability-manifests/fixture-ping.json");
        let make_manifest = |cap_name: &str, provider_id: &str| -> (String, String) {
            let mut m: serde_json::Value = serde_json::from_str(manifest_template).unwrap();
            m["capability_name"] = serde_json::json!(cap_name);
            m["provider"]["identity"] = serde_json::json!(provider_id);
            m["binding"]["server_name"] = serde_json::json!("tethers-stdio-fixture");
            m["permission_scope"] =
                serde_json::json!({"kind": "path_prefix", "allowed_prefixes": ["member/"]});
            m["confirmation_policy"] =
                serde_json::json!({"standing_permitted": true, "per_call_required": false});
            let s = serde_json::to_string(&m).unwrap();
            let (_, digest) = crate::manifest::canonicalize_and_digest(&s).unwrap();
            m["digest"] = serde_json::json!(digest);
            (serde_json::to_string_pretty(&m).unwrap(), digest)
        };

        let mut digests = std::collections::HashMap::new();
        for tag in member_tags {
            let (manifest_json, digest) =
                make_manifest(&format!("fixture.ping-{tag}"), &format!("provider-{tag}"));
            std::fs::write(
                runtime_dir.join(format!("manifests/fixture-ping-{tag}.json")),
                &manifest_json,
            )
            .unwrap();
            digests.insert(*tag, digest);
        }

        let barrier_script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("scripts")
            .join("tethers-stdio-fixture.ps1");
        let barrier_str = barrier_dir.to_str().unwrap().to_owned();

        let reqs: Vec<serde_json::Value> = member_tags
            .iter()
            .map(|tag| {
                json!({
                    "name": format!("fixture.ping-{tag}"),
                    "version": 1,
                    "reason": "c3-a4 config proof"
                })
            })
            .collect();

        let providers_json: Vec<serde_json::Value> = member_tags
            .iter()
            .map(|tag| {
                let provider_id = format!("provider-{tag}");
                let cap_name = format!("fixture.ping-{tag}");
                let digest = digests.get(*tag).unwrap();
                json!({
                    "id": provider_id,
                    "display_name": format!("Provider {tag}"),
                    "transport": {
                        "kind": "stdio",
                        "command": "pwsh.exe",
                        "args": [
                            "-NoProfile", "-ExecutionPolicy", "Bypass", "-File",
                            barrier_script.to_str().unwrap(),
                            "-Mode", "c2-overlap-barrier",
                            "-BarrierDirectory", &barrier_str
                        ],
                        "protocol_version": "2025-11-25"
                    },
                    "capabilities": [{
                        "name": cap_name,
                        "version": 1,
                        "manifest_path": format!("manifests/fixture-ping-{tag}.json"),
                        "pinned_digest": digest,
                        "scope_binding": {"kind": "path_prefix", "argument_json_pointer": "/message"}
                    }]
                })
            })
            .collect();

        let rules_json: Vec<serde_json::Value> = member_tags
            .iter()
            .map(|tag| {
                json!({
                    "name": format!("fixture.ping-{tag}"),
                    "version": 1,
                    "decision": "allow"
                })
            })
            .collect();

        let mut config = json!({
            "format_version": "0.1",
            "tether_set": {
                "id": "test.together",
                "version": "1",
                "tethers": [{
                    "id": "together-test",
                    "version": "1",
                    "source_path": "tethers/together-test.tether"
                }],
                "capability_requirements": reqs
            },
            "providers": providers_json,
            "policy": {
                "default": "deny",
                "rules": rules_json
            }
        });

        // Inject the configured max_active_together_invocations if specified.
        if let Some(n) = max_active {
            config["max_active_together_invocations"] = json!(n);
        }

        let config_path = runtime_dir.join("tethers-config.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&config).unwrap()).unwrap();
        let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
        let runtime = prepare_runtime(&loaded).unwrap();

        let trail_path = std::env::temp_dir().join(format!(
            "tethers-c3a4-{test_name}-trail-{}.jsonl",
            uuid::Uuid::new_v4()
        ));
        let replay_dir = std::env::temp_dir().join(format!(
            "tethers-c3a4-{test_name}-replay-{}",
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(&replay_dir).unwrap();

        C3A1GroupHarness {
            runtime,
            _runtime_dir: runtime_dir,
            trail_path,
            replay_dir,
            barrier_dir,
            members: member_tags.iter().map(|s| (*s).to_owned()).collect(),
        }
    }

    /// Run `execute_group_concurrent` (the wrapper) instead of
    /// `execute_group_concurrent_with_limit` to prove the configuration
    /// value reaches the production wrapper.
    fn c3a4_run_group_via_wrapper(
        harness: &C3A1GroupHarness,
        eval_id: &str,
    ) -> ExecutionServiceResult {
        let providers = harness.runtime.providers().to_vec();
        let mut sessions = HashMap::new();
        for provider in &providers {
            let manifest = provider.capabilities[0].verified_manifest.manifest();
            let session = RetainedProviderSession::establish(SocketEstablishment {
                command: &provider.stdio_config.command,
                args: &provider.stdio_config.args,
                working_directory: &provider.working_directory,
                protocol_version: &provider.stdio_config.protocol_version,
                server_name: &manifest.binding.server_name,
                identity: &provider.identity,
            })
            .expect("barrier provider session establishment");
            sessions.insert(provider.identity.clone(), session);
        }

        let mut actions = Vec::new();
        let mut member_action_ids = Vec::new();
        let mut member_indexes = Vec::new();

        for (idx, tag) in harness.members.iter().enumerate() {
            let action_id = format!("member-{tag}");
            let provider_id = format!("provider-{tag}");
            let cap_name = format!("fixture.ping-{tag}");
            let digest = harness
                .runtime
                .providers()
                .iter()
                .find(|p| p.identity == provider_id)
                .unwrap()
                .capabilities[0]
                .verified_manifest
                .verified_digest()
                .to_owned();

            actions.push(json!({
                "action_id": action_id,
                "idempotency_key": format!("{eval_id}/{action_id}"),
                "capability": cap_name,
                "capability_version": "1.0.0",
                "bridge_capability_version": 1,
                "bridge_provider_identity": provider_id,
                "manifest_digest": digest,
                "arguments": {"message": format!("member/{tag}")},
            }));
            member_action_ids.push(action_id);
            member_indexes.push(idx);
        }

        let groups = vec![json!({
            "group_id": "together-1",
            "member_action_ids": member_action_ids,
        })];
        let mut response = json!({
            "status": "matched",
            "evaluation_id": eval_id,
            "plan": { "id": format!("plan-{eval_id}"), "actions": actions, "groups": groups },
            "trail": [],
        });
        let member_actions = response["plan"]["actions"].as_array().unwrap().clone();
        let avail_identities: Vec<String> = harness
            .members
            .iter()
            .map(|tag| format!("provider-{tag}"))
            .collect();
        let availability =
            ProviderAvailability::from_identities(avail_identities.iter().map(|s| s.as_str()));
        let mut trail = dispatch::FileTrail::open(&harness.trail_path).unwrap();
        let mut approvals = crate::approval::ApprovalStore::default();
        let mut replay_authority =
            crate::replay_runtime::test_support::TestReplayAuthority::default();
        let engine_path = PathBuf::from("unused-engine");
        let service =
            HostExecutionService::new(&harness.runtime, &engine_path, &harness.trail_path, None);

        // Call execute_group_concurrent (the wrapper), NOT
        // execute_group_concurrent_with_limit.  This proves the
        // configuration value reaches the production wrapper.
        execute_group_concurrent(
            "together-1",
            &member_indexes,
            &member_actions,
            &mut response,
            eval_id,
            &mut trail,
            &service,
            &PreparedEvaluationInput {
                tether_id: "together-test".to_owned(),
                tether_version: "1".to_owned(),
                evaluation_id: eval_id.to_owned(),
                anchor_event: json!({"id": format!("evt-{eval_id}"), "name": "test"}),
                facts: json!({}),
            },
            &mut sessions,
            &availability,
            &mut approvals,
            &mut replay_authority,
        )
    }

    // A4.9: Physical default-N=2 proof using execute_group_concurrent wrapper.
    //
    // Group: A B C
    // Default N=2: A+B reach active simultaneously, C waits.
    // Release B → C launches.  Release A+C → all terminal → GroupJoin.
    #[test]
    fn c3_a4_default_two_controls_real_group_execution() {
        let h = c3a4_harness_with_config("default-two", &["a", "b", "c"], None);

        // Verify the runtime accessor matches the default.
        assert_eq!(h.runtime.max_active_together_invocations(), 2);

        std::thread::scope(|s| {
            let handle = s.spawn(|| c3a4_run_group_via_wrapper(&h, "eval-c3a4-d2"));

            // Wait until both A and B become active simultaneously.
            h.wait_barrier_active_count(2);
            assert_eq!(
                h.currently_active_count(),
                2,
                "default N=2 must have exactly 2 active members"
            );
            assert!(!h.has_entered("c"), "C must wait for capacity");

            // Release B → slot opens → C launches.
            h.release_member("b");
            h.wait_member_outcome("b");
            h.wait_member_active("c");

            // A is still in-flight, C is in-flight: exactly 2 active.
            assert_eq!(
                h.currently_active_count(),
                2,
                "after B completes, A+C must be exactly 2 active"
            );

            // Release A and C.
            h.release_member("a");
            h.release_member("c");

            let result = handle.join().expect("group must complete without panic");
            assert!(
                matches!(result, ExecutionServiceResult::Completed { .. }),
                "expected Completed, got {result:?}"
            );
        });

        // Verify all 3 outcomes present and GroupJoin succeeded.
        let ids = h.outcome_ids();
        assert_eq!(ids.len(), 3, "all 3 members must have outcomes");
        assert!(ids.contains(&"member-a".to_owned()));
        assert!(ids.contains(&"member-b".to_owned()));
        assert!(ids.contains(&"member-c".to_owned()));
        assert_eq!(h.entry_kinds().last(), Some(&"group_join".to_owned()));
    }

    // A4.10: Physical explicit-N=1 proof using execute_group_concurrent wrapper.
    //
    // Group: A B C
    // Explicit N=1: A launches, B/C wait.  After A completes, B launches.
    // After B completes, C launches.  Physical max active == 1.
    #[test]
    fn c3_a4_explicit_one_controls_real_group_execution() {
        let h = c3a4_harness_with_config("explicit-one", &["a", "b", "c"], Some(1));

        // Verify the runtime accessor matches the explicit value.
        assert_eq!(h.runtime.max_active_together_invocations(), 1);

        // Tell the barrier script to expect only 1 peer at a time.
        h.set_peer_count(1);

        std::thread::scope(|s| {
            let handle = s.spawn(|| c3a4_run_group_via_wrapper(&h, "eval-c3a4-e1"));

            // 1. Member A launches and enters active state.
            h.wait_member_active("a");
            assert_eq!(
                h.currently_active_count(),
                1,
                "N=1 must have exactly 1 active member"
            );
            assert!(!h.has_entered("b"), "B must wait for capacity");
            assert!(!h.has_entered("c"), "C must wait for capacity");

            // Release A and wait for its completion.
            h.release_member("a");
            h.wait_member_outcome("a");

            // 2. Member B launches after A completes.
            h.wait_member_active("b");
            assert_eq!(
                h.currently_active_count(),
                1,
                "N=1 must have exactly 1 active member"
            );
            assert!(!h.has_entered("c"), "C must wait for capacity");

            // Release B and wait for its completion.
            h.release_member("b");
            h.wait_member_outcome("b");

            // 3. Member C launches after B completes.
            h.wait_member_active("c");
            assert_eq!(
                h.currently_active_count(),
                1,
                "N=1 must have exactly 1 active member"
            );

            // Release C.
            h.release_member("c");

            let result = handle.join().expect("group must complete without panic");
            assert!(
                matches!(result, ExecutionServiceResult::Completed { .. }),
                "expected Completed, got {result:?}"
            );
        });

        // Verify all 3 outcomes present in semantic order and GroupJoin succeeded.
        let ids = h.outcome_ids();
        assert_eq!(ids.len(), 3, "all 3 members must have outcomes");
        assert_eq!(ids, vec!["member-a", "member-b", "member-c"]);
        assert_eq!(h.entry_kinds().last(), Some(&"group_join".to_owned()));
    }
}
