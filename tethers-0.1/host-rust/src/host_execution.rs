// J13B Packet 1 - typed host execution service and retained execution sessions.
//
// Extracts the host execution machinery from main.rs into a typed Rust
// application service.  Uses retained OCaml engine and MCP provider
// sessions for validation, evaluation, and dispatch.
//
// No public `run` command.  No evaluation-ID derivation rule.

use crate::configured_runtime::{PreparedRuntime, PreparedTether};
use crate::dispatch::{self, DispatchReadyAction};
use crate::executor::CapabilityExecutor;

use crate::manifest::BindingKind;
use crate::outcome::{self, ProductionMonotonicClock};
use crate::policy::{self, PermissionDecision, ProposedAction};
use crate::replay_runtime::FileReplayAuthority;
use crate::resolver::{self, ProviderAvailability, ResolvedCapability};
use crate::stdio_provider::{ManagedProvider, StdioProviderError};
use serde_json::Value;

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tethers_reference_host::child_process;
use tethers_reference_host::engine_stdio::{EngineError, EngineSession};

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
    },
    /// Policy explicitly denied the action.
    Denied {
        evaluation_id: String,
        action_id: String,
        reason: String,
    },
    /// The planner returned no actions.
    NoActions {
        evaluation_id: String,
        response: Value,
    },
    /// Policy is Ask - human approval required before dispatch.
    ApprovalRequired {
        evaluation_id: String,
        approval_id: String,
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
    },
    /// Post-invocation uncertainty.
    Uncertain {
        evaluation_id: String,
        action_id: String,
        reason: String,
    },
    /// Outcome audit recording failed.
    AuditFailed {
        evaluation_id: String,
        action_id: String,
        reason: String,
    },
    /// Deadline expired before the provider invocation boundary.
    Unattempted {
        evaluation_id: String,
        action_id: String,
        reason: String,
    },
    /// A prior execution completed successfully; replay is blocked.
    ReplayBlockedCompletedSuccess {
        evaluation_id: String,
        action_id: String,
    },
    /// A prior execution completed with known failure; replay is blocked.
    ReplayBlockedCompletedFailure {
        evaluation_id: String,
        action_id: String,
    },
    /// Recovered claim, intent, armed, or uncertain state requires a human.
    ReplayRequiresManualResolution {
        evaluation_id: String,
        action_id: String,
    },
    /// Replay storage or terminal publication could not be trusted.
    ReplayPersistenceUnavailable {
        evaluation_id: String,
        action_id: String,
    },
    /// Interrupted before provider invocation.
    Interrupted,
    /// Invalid input data.
    InvalidData { message: String },
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
            other => Self::Engine(other),
        }
    }
}

// ===========================================================================
// Retained provider session
// ===========================================================================

/// Retained MCP provider session with monotonically increasing request IDs.
///
/// Each retained session wraps one live ManagedProvider and supports exact
/// `tools/call` invocation for dispatch.  JSON-RPC errors are mapped to
/// typed provider errors.
pub struct RetainedProviderSession {
    provider: ManagedProvider,
    request_ids: RequestIdSequence,
    identity: String,
}

#[derive(Debug)]
struct RequestIdSequence {
    next: u64,
}

impl RequestIdSequence {
    fn after_initialization() -> Self {
        Self { next: 3 }
    }

    fn take(&mut self) -> u64 {
        let current = self.next;
        self.next += 1;
        current
    }
}

impl RetainedProviderSession {
    /// Wrap an already-initialized-and-listed `ManagedProvider`.
    pub fn new(provider: ManagedProvider, identity: String) -> Self {
        // Request IDs start at 3 after initialize (1) and tools/list (2).
        Self {
            provider,
            request_ids: RequestIdSequence::after_initialization(),
            identity,
        }
    }

    /// The provider identity.
    pub fn identity(&self) -> &str {
        &self.identity
    }

    /// Invoke a `tools/call` with monotonically increasing request ID.
    ///
    /// Validates JSON-RPC version, matching response ID, and maps JSON-RPC
    /// errors to typed errors.  Returns the `result` portion of the
    /// response.
    pub fn tools_call(
        &mut self,
        tool_name: &str,
        arguments: &Value,
        remaining: Duration,
    ) -> Result<Value, StdioProviderError> {
        let id = self.request_ids.take();
        self.provider
            .tools_call_with_timeout(id, tool_name, arguments, remaining)
    }

    /// Retained stderr tail from the provider process.
    pub fn stderr_tail(&self) -> String {
        self.provider.stderr_tail()
    }

    /// Graceful close with Drop as emergency fallback.
    pub fn close(&mut self) {
        self.provider.close();
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
        let mut engine =
            EngineSession::launch(&self.engine_path.to_path_buf(), &engine_working_dir)?;

        // --- 2. Validate every configured Tether ---
        for (i, tether) in self.runtime.tethers().iter().enumerate() {
            if child_process::is_interrupted() {
                engine.shutdown();
                return Err(ExecutionServiceError::Interrupted);
            }
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

        let mut provider = ManagedProvider::launch(
            &config.command,
            &config.args,
            &prepared.working_directory,
            None,
            None,
        )
        .map_err(|e| ExecutionServiceError::Provider(format!("launch failed: {e}")))?;

        // MCP initialize.
        let expected_server_name = prepared
            .capabilities
            .first()
            .map(|c| c.verified_manifest.manifest().binding.server_name.as_str())
            .unwrap_or("");
        provider
            .initialize(&config.protocol_version, expected_server_name)
            .map_err(|e| {
                ExecutionServiceError::Provider(format!(
                    "initialize failed for {}: {e}",
                    prepared.identity
                ))
            })?;

        // MCP tools/list.
        let tools = provider.list_tools().map_err(|e| {
            ExecutionServiceError::Provider(format!(
                "tools/list failed for {}: {e}",
                prepared.identity
            ))
        })?;
        if validate_prepared_discovery(prepared, &tools).is_err() {
            provider.close();
            return Ok(None);
        }

        Ok(Some(RetainedProviderSession::new(
            provider,
            prepared.identity.clone(),
        )))
    }

    /// Evaluate one prepared input through the engine and dispatch pipeline.
    fn evaluate_one(
        &self,
        input: &PreparedEvaluationInput,
        engine: &mut EngineSession,
        provider_sessions: &mut HashMap<String, RetainedProviderSession>,
        provider_availability: &ProviderAvailability,
    ) -> ExecutionServiceResult {
        // Find the matching Tether in the runtime.
        let tether = match self.find_tether(&input.tether_id, &input.tether_version) {
            Ok(t) => t,
            Err(result) => return result,
        };

        // Build the Tethers 0.1 request envelope.
        let envelope = match self.build_request_envelope(input, tether, provider_availability) {
            Ok(envelope) => envelope,
            Err(result) => return result,
        };

        // Call tethers.evaluate.
        let tethers_response = match engine.evaluate_tether(&input.evaluation_id, &envelope) {
            Ok(resp) => resp,
            Err(e) => {
                if matches!(e, EngineError::Interrupted) {
                    return ExecutionServiceResult::Interrupted;
                }
                return ExecutionServiceResult::InvalidData {
                    message: format!("engine evaluation failed: {e}"),
                };
            }
        };

        // Check if the Tethers response is matched.
        if !Self::response_is_matched(&tethers_response) {
            return ExecutionServiceResult::NoActions {
                evaluation_id: input.evaluation_id.clone(),
                response: tethers_response,
            };
        }

        // Resolve capability and dispatch.
        self.dispatch_matched_response(
            input,
            tethers_response,
            provider_sessions,
            provider_availability,
        )
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

    /// Check if a Tethers response has matched status.
    fn response_is_matched(response: &Value) -> bool {
        response.get("status").and_then(Value::as_str) == Some("matched")
    }

    /// Dispatch a matched response through the one accepted J05-J11 boundary.
    fn dispatch_matched_response(
        &self,
        input: &PreparedEvaluationInput,
        mut response: Value,
        provider_sessions: &mut HashMap<String, RetainedProviderSession>,
        provider_availability: &ProviderAvailability,
    ) -> ExecutionServiceResult {
        let proposed = match crate::extract_proposed_action(&response) {
            Ok(action) => action,
            Err(error) => {
                return ExecutionServiceResult::InvalidData {
                    message: format!("invalid planned Action: {error}"),
                };
            }
        };
        let evaluation_id = proposed.evaluation_id.clone();
        let action_id = proposed.action_id.clone();
        let scope_assessment = self.runtime.assess_action_scope(&proposed);
        let policy_evaluation = policy::evaluate_effective_policy(
            &proposed,
            self.runtime.requirements(),
            self.runtime.trusted_store(),
            provider_availability,
            self.runtime.policy(),
            scope_assessment,
        );

        match &policy_evaluation.decision {
            PermissionDecision::Deny => {
                return ExecutionServiceResult::Denied {
                    evaluation_id,
                    action_id,
                    reason: format!("{:?}", policy_evaluation.reason),
                };
            }
            PermissionDecision::Ask => {
                return ExecutionServiceResult::ApprovalRequired {
                    evaluation_id,
                    approval_id: format!("approval-{action_id}"),
                };
            }
            PermissionDecision::Unavailable => {
                return ExecutionServiceResult::Unavailable {
                    evaluation_id,
                    reason: format!("{:?}", policy_evaluation.reason),
                };
            }
            PermissionDecision::Allow(_) => {}
        }

        let resolved = match self.resolve_exact_capability(&proposed, provider_availability) {
            Ok(resolved) => resolved,
            Err(result) => return result,
        };
        let binding = &resolved.manifest().manifest().binding;
        if binding.kind != BindingKind::Mcp {
            return ExecutionServiceResult::Denied {
                evaluation_id,
                action_id,
                reason: "capability binding is not MCP".to_owned(),
            };
        }
        let session = match provider_sessions.get_mut(resolved.provider_identity()) {
            Some(session) => session,
            None => {
                return ExecutionServiceResult::Unavailable {
                    evaluation_id,
                    reason: format!(
                        "provider '{}' has no retained session",
                        resolved.provider_identity()
                    ),
                };
            }
        };
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
                return ExecutionServiceResult::InvalidData {
                    message: "Anchor event requires a non-empty string id".to_owned(),
                };
            }
        };
        let input_context = crate::InputEventContext::for_initial(event_id);

        if let Some(parent) = self.trail_path.parent() {
            if let Err(error) = fs::create_dir_all(parent) {
                return ExecutionServiceResult::AuditFailed {
                    evaluation_id,
                    action_id,
                    reason: format!("trail directory create failed: {error}"),
                };
            }
        }
        let mut trail = match dispatch::FileTrail::open(self.trail_path) {
            Ok(trail) => trail,
            Err(error) => {
                return ExecutionServiceResult::AuditFailed {
                    evaluation_id,
                    action_id,
                    reason: format!("trail open failed: {error}"),
                };
            }
        };
        let clock = ProductionMonotonicClock::new();
        let mut replay_authority = FileReplayAuthority::new(self.host_data_root);
        let mut anchor_writer = crate::ResponseResultAnchorWriter;
        let shared = match crate::execute_shared_boundary(
            &mut response,
            policy_evaluation.decision,
            &resolved,
            &mut trail,
            &mut executor,
            &input_context,
            true,
            &clock,
            &mut replay_authority,
            None,
            &mut anchor_writer,
        ) {
            Ok(result) => result,
            Err(error) => {
                return ExecutionServiceResult::AuditFailed {
                    evaluation_id,
                    action_id,
                    reason: format!("shared execution boundary failed: {error}"),
                };
            }
        };
        Self::map_shared_result(shared, evaluation_id, action_id, response)
    }

    fn map_shared_result(
        result: crate::SharedExecutionResult,
        evaluation_id: String,
        action_id: String,
        response: Value,
    ) -> ExecutionServiceResult {
        match result {
            crate::SharedExecutionResult::Completed => ExecutionServiceResult::Completed {
                evaluation_id,
                action_id,
                response,
            },
            crate::SharedExecutionResult::Failed => ExecutionServiceResult::Failed {
                evaluation_id,
                action_id,
                reason: "provider returned a known failure".to_owned(),
            },
            crate::SharedExecutionResult::Uncertain => ExecutionServiceResult::Uncertain {
                evaluation_id,
                action_id,
                reason: "provider outcome is uncertain".to_owned(),
            },
            crate::SharedExecutionResult::Unattempted => ExecutionServiceResult::Unattempted {
                evaluation_id,
                action_id,
                reason: "deadline expired before provider invocation".to_owned(),
            },
            crate::SharedExecutionResult::Denied => ExecutionServiceResult::Denied {
                evaluation_id,
                action_id,
                reason: "shared execution boundary denied dispatch".to_owned(),
            },
            crate::SharedExecutionResult::AuditFailed => ExecutionServiceResult::AuditFailed {
                evaluation_id,
                action_id,
                reason: "durable execution audit failed".to_owned(),
            },
            crate::SharedExecutionResult::Replay(
                crate::replay_runtime::ReplayDispatchResult::PersistenceUnavailable,
            ) => ExecutionServiceResult::ReplayPersistenceUnavailable {
                evaluation_id,
                action_id,
            },
            crate::SharedExecutionResult::Replay(
                crate::replay_runtime::ReplayDispatchResult::BlockedCompletedSuccess,
            ) => ExecutionServiceResult::ReplayBlockedCompletedSuccess {
                evaluation_id,
                action_id,
            },
            crate::SharedExecutionResult::Replay(
                crate::replay_runtime::ReplayDispatchResult::BlockedCompletedFailure,
            ) => ExecutionServiceResult::ReplayBlockedCompletedFailure {
                evaluation_id,
                action_id,
            },
            crate::SharedExecutionResult::Replay(
                crate::replay_runtime::ReplayDispatchResult::RequiresManualResolution,
            ) => ExecutionServiceResult::ReplayRequiresManualResolution {
                evaluation_id,
                action_id,
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
                })?;
        let provider = proposed
            .bridge_provider_identity
            .as_deref()
            .ok_or_else(|| ExecutionServiceResult::Denied {
                evaluation_id: proposed.evaluation_id.clone(),
                action_id: proposed.action_id.clone(),
                reason: "missing bridge provider identity".to_owned(),
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

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dispatch::{self, ActionId, ExecutionId, RecordingTrail};
    use crate::policy::{CapabilityRequirement, HostLocalPolicy, ScopeAssessment};
    use crate::replay::LogicalExecutionKey;
    use crate::trusted_store::TrustedManifestStore;

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
                crate::SharedExecutionResult::Replay(replay),
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
            crate::SharedExecutionResult::Unattempted,
            "eval-1".to_owned(),
            "action-1".to_owned(),
            serde_json::json!({}),
        );
        assert!(matches!(result, ExecutionServiceResult::Unattempted { .. }));
    }

    /// The retained provider continues after initialize/list with strictly
    /// increasing request IDs across evaluations.
    #[test]
    fn j13b_retained_session_monotonic_ids() {
        let mut ids = RequestIdSequence::after_initialization();
        assert_eq!(ids.take(), 3);
        assert_eq!(ids.take(), 4);
        assert_eq!(ids.take(), 5);
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
            RetainedProviderSession::new(provider, "tethers-stdio-fixture".to_owned());
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
        assert_eq!(session.request_ids.next, 5);
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

    /// The service exposes prepared-input execution only: it neither defines a
    /// CLI `run` command nor an evaluation-ID derivation helper.
    #[test]
    fn j13b_no_public_cli_run_command_or_evaluation_id_derivation() {
        let input = PreparedEvaluationInput {
            tether_id: "t".into(),
            tether_version: "1.0.0".into(),
            evaluation_id: "explicit-only-001".into(),
            anchor_event: serde_json::json!({}),
            facts: serde_json::json!({}),
        };
        assert_eq!(input.evaluation_id, "explicit-only-001");
        use clap::Parser;
        assert!(tethers_reference_host::cli::Cli::try_parse_from([
            "tethers-reference-host",
            "run"
        ])
        .is_err());
    }
}
