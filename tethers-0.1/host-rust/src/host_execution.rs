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

/// Perform the full Socket establishment / discovery / invocation path in a
/// worker thread.  Uses the same trusted contract as the serial path:
/// `RetainedProviderSession::establish` → `refresh_prepared_catalogue` →
/// `session.tools_call` → `session.close`.
///
/// This function does NOT touch Trail, replay, response, or any
/// coordinator-owned state.  It is a pure provider invocation carrier.
pub(crate) fn worker_invoke_provider(input: WorkerInput, tx: mpsc::Sender<WorkerResult>) {
    let result = worker_invoke_inner(&input);
    // Channel send failure means the coordinator dropped — not an error
    // the worker can recover from.
    let _ = tx.send(WorkerResult {
        action_index: input.action_index,
        provider_result: result,
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

/// Execute one `together` group with real provider invocation overlap.
///
/// This is the C2-A3a concurrent execution path.  It replaces the serial
/// `execute_plan` call for Group items only.  Sequential items continue
/// using the existing serial path.
///
/// ## Execution phases
///
/// **STAGE A — Serial deterministic preparation** (coordinator-owned):
/// For every member, in Runtime Plan order: scope, policy, resolution,
/// replay admission, G0 intent, Trail intent.  No deadline, no G1, no
/// provider effect.  Prep failures are recorded immediately as terminal
/// states with exact classification.
///
/// **STAGE B — Physical provider invocation** (concurrent workers):
/// For each eligible member, immediately before launch: deadline start,
/// G1 armed, scoped thread spawn.  Workers return raw provider results
/// through an mpsc channel.
///
/// **STAGE C — Durable result collection** (coordinator-owned):
/// As results arrive on the channel: classify outcome, Trail outcome, G2,
/// result anchor.  Stage C runs per-result immediately, not in member
/// order.
///
/// **STAGE D — Join** (coordinator-owned):
/// After all members terminal: GroupJoinEntry, all-success test.
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

    // ── STAGE B: Launch workers through mpsc channel ───────────────────
    let clock = ProductionMonotonicClock::new();
    let mut anchor_writer = crate::ResponseResultAnchorWriter;
    let (tx, rx) = mpsc::channel::<WorkerResult>();
    let mut launched_count: usize = 0;

    struct PendingLaunch {
        worker_input: WorkerInput,
    }
    let mut pending_launches: Vec<PendingLaunch> = Vec::new();

    // Whole-enum movement prevents fabricated domain values when the real
    // coordinator-owned state moves from Prepared to Launched.
    for state in member_states.iter_mut() {
        let transition = match state {
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
            _ => continue,
        };
        let prior = std::mem::replace(state, transition);
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
        let remaining =
            match crate::outcome::remaining_until_deadline(&clock, deadline_start, deadline) {
                Some(remaining) => remaining,
                None => {
                    *state = GroupMemberState::Terminal {
                        action_index,
                        action_id: action_id.clone(),
                        semantic_position: position,
                        step: crate::plan_execution::ActionStep::Stopped(
                            ExecutionServiceResult::Unattempted {
                                evaluation_id: evaluation_id.to_owned(),
                                action_id,
                                reason: "deadline expired before provider invocation".to_owned(),
                                execution_id: Some(ready.execution_id().0.clone()),
                            },
                        ),
                    };
                    continue;
                }
            };

        if admission.publish_armed().is_err() {
            *state = GroupMemberState::Terminal {
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
                *state = GroupMemberState::Terminal {
                    action_index,
                    action_id,
                    semantic_position: position,
                    step: crate::plan_execution::ActionStep::Stopped(
                        ExecutionServiceResult::Unavailable {
                            evaluation_id: evaluation_id.to_owned(),
                            reason: format!("provider '{provider_identity}' has no prepared catalogue authority"),
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
        *state = GroupMemberState::Launched {
            action_index,
            action_id: action_id.clone(),
            semantic_position: position.clone(),
            ready: Some(ready),
            prepared: Some(prepared),
            admission: Some(admission),
        };
        pending_launches.push(PendingLaunch {
            worker_input: WorkerInput {
                action_index,
                arguments,
                provider: prepared_provider,
                tool_name,
                remaining,
            },
        });
    }

    std::thread::scope(|s| {
        // Spawn all workers.
        for pending in pending_launches {
            let tx_clone = tx.clone();
            s.spawn(move || worker_invoke_provider(pending.worker_input, tx_clone));
            launched_count += 1;
        }

        drop(tx);

        // Receive results as workers complete and process Stage C immediately.
        let mut received = 0usize;
        while received < launched_count {
            match rx.recv() {
                Ok(worker_result) => {
                    received += 1;

                    // Find the Launched member for this action_index.
                    // Extract ready and prepared for Stage C processing.
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
                                // Move real objects out for Stage C.
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

                    // Stage C: classify outcome, Trail outcome, G2, result anchor.
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
                        Ok(result) => crate::plan_execution::ActionStep::Boundary(result),
                        Err(error) => crate::plan_execution::ActionStep::Stopped(
                            ExecutionServiceResult::AuditFailed {
                                evaluation_id: evaluation_id.to_owned(),
                                action_id: action_id.clone(),
                                reason: format!("shared execution boundary failed: {error}"),
                                execution_id: Some(ready.execution_id().0.clone()),
                            },
                        ),
                    };

                    // Transition to Terminal.
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
                    // Channel closed — all workers finished but we haven't
                    // received all results.  This shouldn't happen since
                    // launched_count tracks exactly how many workers were
                    // spawned.
                    break;
                }
            }
        }
    });

    // ── STAGE D: Join ──────────────────────────────────────────────────
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
    use crate::replay::LogicalExecutionKey;
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
        let prepared = crate::configured_runtime::prepare_runtime(&loaded).unwrap();
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
}
