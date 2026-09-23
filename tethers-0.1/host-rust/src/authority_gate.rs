//! Tethers Authority Gate — non-executing authority façade for external Hosts.
//!
//! The Gate answers one consequential question: *may this exact effect be
//! dispatched now?* It never invokes a provider, never spawns the requested
//! Action, and never writes target files. Policy, approval, scope, replay, and
//! durable intent remain the existing Tethers authority machinery; this module
//! only sequences them across a persistent stdio session.
//!
//! Core law: Tethers authorises. The external Host executes.
//! A previous admission is never a reusable permission slip.

use crate::approval::{self, ApprovalState, ApprovalStore};
use crate::configured_runtime::{prepare_runtime, PreparedRuntime};
use crate::dispatch::{self, ActionId, ExecutionId, FileTrail, Trail};
use crate::gate_protocol::{
    AuthorityResponse, Observations, PreparePayload, AUTHORITY_PROTOCOL, MAX_STATUS_ENTRIES,
};
use crate::host_execution::{HostExecutionService, PlanResult, PreparedEvaluationInput};
use crate::policy::{self, PermissionDecision, PolicyReason, ProposedAction};
use crate::replay;
use crate::replay_runtime::{
    FileReplayAuthority, ReplayAdmissionGuard, ReplayAuthority, ReplayDispatchResult,
};
use crate::resolver;
use crate::runtime_config::load_runtime_config;
use crate::validation;
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Session configuration supplied by the CLI.
#[derive(Debug, Clone)]
pub struct GateConfig {
    pub config_path: PathBuf,
    pub engine_path: PathBuf,
    pub trail_path: PathBuf,
    pub host_data_root: PathBuf,
}

/// One exact Action prepared for a later last-responsible-moment commit.
#[derive(Debug)]
struct PreparedRecord {
    prepared_id: String,
    action: ProposedAction,
    event_id: String,
    tether_id: String,
    tether_version: String,
    /// Decision observed at prepare time. Informational only — commit re-checks.
    prepare_decision: &'static str,
    approval_id: Option<String>,
    observations: Observations,
    config_digest: String,
    committed: bool,
}

/// One admitted commit awaiting (or holding) an external outcome.
struct CommittedRecord {
    execution_id: String,
    action_id: String,
    evaluation_id: String,
    capability_name: String,
    capability_version: u32,
    manifest_digest: String,
    provider_identity: String,
    argument_digest: String,
    prepared_id: String,
    /// Trusted capability output schema captured at COMMIT for OUTCOME validation.
    output_schema: Value,
    outcome_status: Option<String>,
    /// Held across COMMIT → OUTCOME so terminal replay publication stays fresh.
    guard: Option<Box<dyn ReplayAdmissionGuard>>,
}

impl std::fmt::Debug for CommittedRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommittedRecord")
            .field("execution_id", &self.execution_id)
            .field("action_id", &self.action_id)
            .field("outcome_status", &self.outcome_status)
            .field("guard_held", &self.guard.is_some())
            .finish_non_exhaustive()
    }
}

/// Bounded in-process Authority Gate session.
pub struct AuthorityGate {
    config: GateConfig,
    gate_instance_id: String,
    product_version: &'static str,
    git_sha: Option<String>,
    prepared: HashMap<String, PreparedRecord>,
    committed: HashMap<String, CommittedRecord>,
    approvals: ApprovalStore,
    shutdown_requested: bool,
    /// Always zero. The Gate never invokes a provider or effectful capability.
    provider_invocations: u64,
}

impl AuthorityGate {
    pub fn new(config: GateConfig) -> Self {
        let gate_instance_id = format!("gate_{}", uuid::Uuid::new_v4().hyphenated());
        Self {
            config,
            gate_instance_id,
            product_version: env!("CARGO_PKG_VERSION"),
            git_sha: option_env!("TETHERS_GIT_SHA").map(str::to_owned),
            prepared: HashMap::new(),
            committed: HashMap::new(),
            approvals: ApprovalStore::default(),
            shutdown_requested: bool::default(),
            provider_invocations: 0,
        }
    }

    pub fn gate_instance_id(&self) -> &str {
        &self.gate_instance_id
    }

    pub fn shutdown_requested(&self) -> bool {
        self.shutdown_requested
    }

    /// Provider/effect invocations performed by this Gate. Always zero.
    pub fn provider_invocations(&self) -> u64 {
        self.provider_invocations
    }

    pub fn handle(
        &mut self,
        operation: &str,
        request_id: &str,
        payload: &Map<String, Value>,
    ) -> AuthorityResponse {
        let outcome = match operation {
            "hello" => self.op_hello(request_id, payload),
            "prepare" => self.op_prepare(payload),
            "approval_decision" => self.op_approval_decision(payload),
            "commit" => self.op_commit(payload),
            "outcome" => self.op_outcome(payload),
            "status" => self.op_status(),
            "shutdown" => self.op_shutdown(),
            other => Err(GateError::new(
                "frame.unknown_operation",
                format!("unknown operation: {other}"),
            )),
        };
        match outcome {
            Ok(result) => AuthorityResponse::ok(request_id, result),
            Err(error) => match error.data {
                Some(data) => {
                    AuthorityResponse::error_with_data(request_id, error.code, error.message, data)
                }
                None => AuthorityResponse::error(request_id, error.code, error.message),
            },
        }
    }

    fn op_hello(
        &self,
        _request_id: &str,
        payload: &Map<String, Value>,
    ) -> Result<Value, GateError> {
        if !payload.is_empty() {
            return Err(GateError::new(
                "frame.unexpected_payload",
                "hello does not accept payload fields",
            ));
        }
        Ok(json!({
            "protocol": AUTHORITY_PROTOCOL,
            "protocol_versions": [AUTHORITY_PROTOCOL],
            "product_version": self.product_version,
            "git_sha": self.git_sha,
            "features": [
                "prepare",
                "approval_decision",
                "commit",
                "outcome",
                "status",
                "shutdown"
            ],
            "gate_instance_id": self.gate_instance_id,
            "authority_granted": false,
            "provider_invocations": self.provider_invocations,
        }))
    }

    fn op_shutdown(&mut self) -> Result<Value, GateError> {
        self.shutdown_requested = true;
        Ok(json!({"shutdown": true, "provider_invocations": self.provider_invocations}))
    }

    // -----------------------------------------------------------------------
    // PREPARE — Core-authoritative read-only precheck. Does not authorise.
    // -----------------------------------------------------------------------

    fn op_prepare(&mut self, payload: &Map<String, Value>) -> Result<Value, GateError> {
        let prepare = crate::gate_protocol::parse_prepare_payload(payload).map_err(frame_err)?;
        let runtime = self.load_runtime()?;
        let config_digest = digest_config(&self.config.config_path)?;

        let tether = select_tether(
            &runtime,
            &prepare.run_input.tether.id,
            &prepare.run_input.tether.version,
        )?;
        let _ = tether;

        // Tethers Core produces the Plan. The Host never supplies one.
        let evaluation_input = PreparedEvaluationInput {
            tether_id: prepare.run_input.tether.id.clone(),
            tether_version: prepare.run_input.tether.version.clone(),
            evaluation_id: prepare.run_input.evaluation_id.clone(),
            anchor_event: json!({
                "id": prepare.run_input.event.id,
                "name": prepare.run_input.event.name,
                "data": prepare.run_input.event.data,
            }),
            facts: prepare.run_input.facts.clone(),
        };
        let plan_result = self.core_plan(&runtime, &evaluation_input)?;
        let (evaluation_id, event_id, plan) = match plan_result {
            PlanResult::PlanAvailable {
                evaluation_id,
                event_id,
                plan,
                ..
            } => (evaluation_id, event_id, plan),
            PlanResult::NoActions { .. } => {
                return Err(GateError::new(
                    "prepare.no_actions",
                    "Tethers Core produced no Actions for this evaluation",
                ));
            }
            PlanResult::PlannerError { code, message, .. } => {
                return Err(GateError::new(
                    "prepare.planner_error",
                    format!("Tethers Core planning failed: {code}: {message}"),
                ));
            }
            PlanResult::Interrupted => {
                return Err(GateError::new(
                    "prepare.interrupted",
                    "planning was interrupted",
                ));
            }
            PlanResult::InvalidData { message, .. } => {
                return Err(GateError::new(
                    "prepare.invalid_data",
                    format!("Tethers Core rejected the evaluation: {message}"),
                ));
            }
            PlanResult::Unavailable { reason, .. } => {
                return Err(GateError::new(
                    "prepare.unavailable",
                    format!("Tethers Core planning is unavailable: {reason}"),
                ));
            }
        };

        let action = proposed_action_from_core_plan(&evaluation_id, &plan, &prepare.action_id)?;
        // Core produces Action identity and arguments. Bridge pins are
        // host-side trusted projections from the admitted manifest store —
        // independent of live provider observations (those affect policy
        // availability, not pin identity).
        let pin_availability = resolver::ProviderAvailability::from_identities(
            runtime
                .providers()
                .iter()
                .map(|provider| provider.identity.as_str()),
        );
        let action = ensure_bridge_pins(action, &runtime, &pin_availability)?;
        let scope = runtime.assess_action_scope(&action);
        let availability = availability_for(&runtime, &prepare.observations);
        let evaluation = policy::evaluate_effective_policy(
            &action,
            runtime.requirements(),
            runtime.trusted_store(),
            &availability,
            runtime.policy(),
            scope,
        );

        let (decision, reason) = match evaluation.decision {
            PermissionDecision::Allow(_) => ("allow_prepared", "current_policy_allow"),
            PermissionDecision::Ask => ("ask", reason_code(&evaluation.reason)),
            PermissionDecision::Deny => ("deny", reason_code(&evaluation.reason)),
            PermissionDecision::Unavailable => ("unavailable", reason_code(&evaluation.reason)),
        };

        let mut approval_id = None;
        if decision == "ask" {
            let mut trail = self.open_trail()?;
            let record = crate::application::request_exact_approval(
                &action,
                runtime.requirements(),
                runtime.trusted_store(),
                &availability,
                runtime.policy(),
                scope,
                &mut self.approvals,
                &mut trail,
            )
            .map_err(|error| {
                GateError::new(
                    "prepare.approval_failed",
                    format!("approval request failed: {error}"),
                )
            })?;
            approval_id = record.map(|record| record.approval_id);
        }

        let prepared_id = prepared_identity(&prepare, &action, &event_id, &config_digest);

        // Replacing an identical prepared identity is refused: each prepare
        // creates a fresh opaque handle bound to this Core evaluation material.
        if self.prepared.contains_key(&prepared_id) {
            return Err(GateError::new(
                "prepare.duplicate_identity",
                "this exact evaluation material is already prepared",
            ));
        }

        self.prepared.insert(
            prepared_id.clone(),
            PreparedRecord {
                prepared_id: prepared_id.clone(),
                action: action.clone(),
                event_id: event_id.clone(),
                tether_id: prepare.run_input.tether.id.clone(),
                tether_version: prepare.run_input.tether.version.clone(),
                prepare_decision: decision,
                approval_id: approval_id.clone(),
                observations: prepare.observations.clone(),
                config_digest,
                committed: false,
            },
        );

        let mut result = json!({
            "prepared_id": prepared_id,
            "decision": decision,
            "reason": reason,
            // PREPARE never authorises physical execution.
            "authorizes_dispatch": false,
            "evaluation_id": evaluation_id,
            "action_id": prepare.action_id,
            "tether_id": prepare.run_input.tether.id,
            "tether_version": prepare.run_input.tether.version,
            "core_planned": true,
            "provider_invocations": self.provider_invocations,
            "execution": {
                "performed": false,
                "provider_invocations": 0,
                "replay_mutated": false
            }
        });
        if let Some(approval_id) = approval_id {
            result["approval"] = json!({
                "approval_id": approval_id,
                "action_id": prepare.action_id,
                "capability": {
                    "name": action.capability_name,
                    "version": action.bridge_capability_version,
                },
                "reason": reason,
                "argument_digest": approval::digest(&action.arguments),
                "effect_summary": effect_summary(&action),
                "state": "pending",
            });
        }
        Ok(result)
    }

    /// Run Tethers Core planning through the current engine. Deterministic
    /// semantic evaluation only — never a provider or effectful capability.
    fn core_plan(
        &self,
        runtime: &PreparedRuntime,
        input: &PreparedEvaluationInput,
    ) -> Result<PlanResult, GateError> {
        let service = HostExecutionService::new(
            runtime,
            &self.config.engine_path,
            &self.config.trail_path,
            Some(&self.config.host_data_root),
        );
        service.plan_only(input).map_err(|error| {
            GateError::new(
                "prepare.unavailable",
                format!("Tethers Core planning failed: {error}"),
            )
        })
    }

    // -----------------------------------------------------------------------
    // APPROVAL_DECISION — relay an exact human decision. Tethers owns the record.
    // -----------------------------------------------------------------------

    fn op_approval_decision(&mut self, payload: &Map<String, Value>) -> Result<Value, GateError> {
        let (approval_id, decision) =
            crate::gate_protocol::parse_approval_payload(payload).map_err(frame_err)?;
        let next = match decision.as_str() {
            "approve" => ApprovalState::Approved,
            "deny" => ApprovalState::Denied,
            "cancel" => ApprovalState::Cancelled,
            _ => {
                return Err(GateError::new(
                    "approval.invalid_decision",
                    "decision must be approve, deny, or cancel",
                ))
            }
        };
        let record = self
            .approvals
            .decide(&approval_id, next)
            .map_err(|error| GateError::new("approval.decision_refused", format!("{error}")))?;
        Ok(json!({
            "approval_id": record.approval_id,
            "state": state_name(record.state),
            "action_id": record.proof.action_id,
            "capability": {
                "name": record.proof.capability_name,
                "version": record.proof.capability_version,
            },
            // Approval is not standing permission; commit still re-checks.
            "authorizes_dispatch": false,
            "provider_invocations": self.provider_invocations,
        }))
    }

    // -----------------------------------------------------------------------
    // COMMIT — last-responsible-moment authority check + durable intent.
    // -----------------------------------------------------------------------

    fn op_commit(&mut self, payload: &Map<String, Value>) -> Result<Value, GateError> {
        let (prepared_id, approval_id_override, commit_observations) =
            crate::gate_protocol::parse_commit_payload(payload).map_err(frame_err)?;

        let record = self
            .prepared
            .get(&prepared_id)
            .ok_or_else(|| GateError::new("commit.unknown_prepared", "prepared_id is not known"))?;
        if record.committed {
            return Err(GateError::new(
                "commit.already_committed",
                "this prepared identity was already committed",
            ));
        }
        let event_id = record.event_id.clone();
        let action = record.action.clone();
        let tether_id = record.tether_id.clone();
        let tether_version = record.tether_version.clone();
        let approval_id = approval_id_override
            .clone()
            .or_else(|| record.approval_id.clone());
        let observations = if commit_observations.available_provider_identities.is_some() {
            commit_observations
        } else {
            record.observations.clone()
        };

        // Fresh current truth. Never trust PREPARE's old verdict.
        let runtime = self
            .load_runtime()
            .map_err(|_| GateError::new("commit.unavailable", "authority state is unavailable"))?;
        let _ = select_tether(&runtime, &tether_id, &tether_version)?;
        let scope = runtime.assess_action_scope(&action);
        let availability = availability_for(&runtime, &observations);
        let evaluation = policy::evaluate_effective_policy(
            &action,
            runtime.requirements(),
            runtime.trusted_store(),
            &availability,
            runtime.policy(),
            scope,
        );

        let mut trail = self.open_trail()?;
        let mut approval_consume: Option<(String, approval::ApprovalProof)> = None;

        let decision = match evaluation.decision {
            PermissionDecision::Allow(allowed) => PermissionDecision::Allow(allowed),
            PermissionDecision::Ask => {
                let approval_id = approval_id.ok_or_else(|| {
                    GateError::new("commit.approval_required", "commit requires an approval_id")
                })?;
                match crate::application::precheck_exact_approval(
                    &action,
                    &approval_id,
                    runtime.requirements(),
                    runtime.trusted_store(),
                    &availability,
                    runtime.policy(),
                    scope,
                    &mut self.approvals,
                    &mut trail,
                ) {
                    Ok(crate::application::ExactApprovalPrecheck::Ready(proof)) => {
                        let resolved = resolver::resolve_capability(
                            runtime.trusted_store(),
                            &availability,
                            &action.capability_name,
                            action.bridge_capability_version.ok_or_else(|| {
                                GateError::new(
                                    "commit.missing_pin",
                                    "bridge capability version is required",
                                )
                            })?,
                            action.bridge_provider_identity.as_deref(),
                        )
                        .map_err(|error| {
                            GateError::new(
                                "commit.unavailable",
                                format!("capability resolution failed: {error:?}"),
                            )
                        })?;
                        approval_consume = Some((approval_id.clone(), proof));
                        policy::allow_after_exact_approval(&resolved)
                    }
                    Ok(crate::application::ExactApprovalPrecheck::NotDispatchable(decision)) => {
                        return Err(match decision {
                            PermissionDecision::Ask => GateError::new(
                                "commit.approval_not_ready",
                                "approval is missing, pending, or not approved",
                            ),
                            PermissionDecision::Deny => GateError::new(
                                "commit.deny",
                                "current authority denies this effect",
                            ),
                            PermissionDecision::Unavailable => GateError::new(
                                "commit.unavailable",
                                "current authority cannot admit this effect",
                            ),
                            PermissionDecision::Allow(_) => GateError::new(
                                "commit.deny",
                                "approval precheck did not yield an allow",
                            ),
                        });
                    }
                    Err(error) => {
                        return Err(GateError::new(
                            "commit.approval_failed",
                            format!("approval precheck failed: {error}"),
                        ))
                    }
                }
            }
            PermissionDecision::Deny => {
                return Err(GateError::new(
                    "commit.deny",
                    format!(
                        "current authority denies: {}",
                        reason_code(&evaluation.reason)
                    ),
                ))
            }
            PermissionDecision::Unavailable => {
                return Err(GateError::new(
                    "commit.unavailable",
                    format!(
                        "current authority unavailable: {}",
                        reason_code(&evaluation.reason)
                    ),
                ))
            }
        };

        // Resolve under the same current store/availability the decision used.
        let resolved = resolver::resolve_capability(
            runtime.trusted_store(),
            &availability,
            &action.capability_name,
            action.bridge_capability_version.ok_or_else(|| {
                GateError::new("commit.missing_pin", "missing capability version")
            })?,
            action.bridge_provider_identity.as_deref(),
        )
        .map_err(|error| {
            GateError::new(
                "commit.unavailable",
                format!("capability resolution failed: {error:?}"),
            )
        })?;

        let PermissionDecision::Allow(allowed) = &decision else {
            return Err(GateError::new(
                "commit.deny",
                "commit did not receive an allow decision",
            ));
        };
        if allowed.capability_name() != resolved.capability_name()
            || allowed.capability_version() != resolved.capability_version()
        {
            return Err(GateError::new(
                "commit.identity_mismatch",
                "allow identity does not match resolved capability",
            ));
        }

        self.dispatch_admitted(
            &decision,
            &resolved,
            &action,
            &event_id,
            &prepared_id,
            approval_consume,
            &mut trail,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn dispatch_admitted(
        &mut self,
        decision: &PermissionDecision,
        resolved: &resolver::ResolvedCapability,
        action: &ProposedAction,
        event_id: &str,
        prepared_id: &str,
        approval_consume: Option<(String, approval::ApprovalProof)>,
        trail: &mut dyn Trail,
    ) -> Result<Value, GateError> {
        let action_id = ActionId(action.action_id.clone());
        let arguments = action.arguments.clone();
        let logical_key = replay::LogicalExecutionKey::derive(
            event_id,
            &action.evaluation_id,
            action_id.as_str(),
        )
        .map_err(|_| GateError::new("commit.unavailable", "replay key unavailable"))?;
        let binding = replay::ExecutionBinding {
            evaluation_id: action.evaluation_id.clone(),
            action_id: action.action_id.clone(),
            capability_name: resolved.capability_name().to_owned(),
            capability_version: resolved.capability_version(),
            manifest_digest: resolved.manifest_digest().to_owned(),
            provider_identity: resolved.provider_identity().to_owned(),
            argument_digest: approval::digest(&arguments),
        };

        let replay_authority = FileReplayAuthority::new(Some(&self.config.host_data_root));
        let mut admission = replay_authority
            .admit(&logical_key, &binding)
            .map_err(|_| {
                GateError::new(
                    "commit.replay_unavailable",
                    "durable replay admission is unavailable",
                )
            })?;

        if !admission.is_fresh() {
            let state = admission.state();
            let execution_id = admission.execution_id().to_owned();
            drop(admission);
            let replay_result = ReplayDispatchResult::from_recovered_state(state);
            return Err(GateError::with_data(
                "commit.replay_blocked",
                format!(
                    "replay refused a second admission: {}",
                    replay_result.as_str()
                ),
                json!({
                    "replay": replay_result.as_str(),
                    "execution_id": execution_id,
                }),
            ));
        }

        // Consume an approved Ask only after the fresh replay claim exists.
        // The authorisation entry must carry the Tethers execution identity,
        // never the evaluation identity.
        let execution_id_str = admission.execution_id().to_owned();
        if let Some((approval_id, proof)) = approval_consume.as_ref() {
            let consumed = self.approvals.consume(approval_id, proof);
            let Ok(consumed) = consumed else {
                drop(admission);
                return Err(GateError::new(
                    "commit.approval_consume_failed",
                    "approved approval could not be consumed; execution requires manual resolution",
                ));
            };
            let entry = dispatch::AuthorisationEntry {
                execution_id: execution_id_str.clone(),
                action_id: consumed.proof.action_id.clone(),
                capability_name: consumed.proof.capability_name.clone(),
                capability_version: consumed.proof.capability_version,
                provider_identity: consumed.proof.provider_identity.clone(),
                manifest_digest: consumed.proof.manifest_digest.clone(),
                kind: "approval_consumed".to_owned(),
                reason_code: "exact_approved_ask".to_owned(),
                argument_digest: consumed.proof.argument_digest.clone(),
            };
            if trail.append_authorisation(&entry).is_err() {
                drop(admission);
                return Err(GateError::new(
                    "commit.approval_consume_failed",
                    "approval consumption could not be recorded durably",
                ));
            }
        }

        if admission.publish_intent().is_err() {
            drop(admission);
            return Err(GateError::new(
                "commit.intent_failed",
                "durable replay intent could not be recorded",
            ));
        }

        let execution_id = ExecutionId::from_replay(&execution_id_str);
        let ready = dispatch::prepare_and_record(
            decision.clone(),
            resolved,
            execution_id,
            action_id.clone(),
            arguments,
            trail,
            None,
        )
        .map_err(|error| {
            GateError::new(
                "commit.intent_failed",
                format!("durable trail intent failed: {error:?}"),
            )
        })?;

        // Dispatch readiness for an external Host means the effect is armed for
        // immediate physical dispatch by that Host. Publishing armed here keeps
        // the same generation sequence the reference executor uses, and OUTCOME
        // can then publish terminal while this session still holds the guard.
        if admission.publish_armed().is_err() {
            drop(ready);
            drop(admission);
            return Err(GateError::new(
                "commit.armed_failed",
                "replay could not mark the execution armed for dispatch",
            ));
        }

        let prepared_id_owned = prepared_id.to_owned();
        if let Some(record) = self.prepared.get_mut(&prepared_id_owned) {
            record.committed = true;
        }

        let dispatch_record = json!({
            "schema": "tethers.dispatch/1",
            "execution_id": execution_id_str,
            "evaluation_id": action.evaluation_id,
            "action_id": action.action_id,
            "event_id": event_id,
            "capability": {
                "name": ready.capability_name(),
                "version": ready.capability_version(),
            },
            "argument_digest": approval::digest(ready.arguments()),
            "manifest_digest": ready.manifest_digest(),
            "provider_identity": ready.provider_identity(),
            "intent": {
                "trail": "recorded",
                "replay": "armed"
            },
            "authority_protocol": AUTHORITY_PROTOCOL,
            "prepared_id": prepared_id_owned,
            "approval_consumed": approval_consume.is_some(),
            "authorizes_physical_execution_by_tethers": false,
            "host_must_report_outcome": true,
            "provider_invocations": self.provider_invocations,
        });

        let output_schema = resolved.manifest().manifest().output_schema.clone();

        self.committed.insert(
            execution_id_str.clone(),
            CommittedRecord {
                execution_id: execution_id_str.clone(),
                action_id: action.action_id.clone(),
                evaluation_id: action.evaluation_id.clone(),
                capability_name: ready.capability_name().to_owned(),
                capability_version: ready.capability_version(),
                manifest_digest: ready.manifest_digest().to_owned(),
                provider_identity: ready.provider_identity().to_owned(),
                argument_digest: approval::digest(ready.arguments()),
                prepared_id: prepared_id_owned,
                output_schema,
                outcome_status: None,
                guard: Some(admission),
            },
        );

        Ok(dispatch_record)
    }

    // -----------------------------------------------------------------------
    // OUTCOME — Host physical observation bound to one committed execution.
    // -----------------------------------------------------------------------

    fn op_outcome(&mut self, payload: &Map<String, Value>) -> Result<Value, GateError> {
        let outcome = crate::gate_protocol::parse_outcome_payload(payload).map_err(frame_err)?;

        // Same-session path: in-memory committed record with a held guard.
        if self.committed.contains_key(&outcome.execution_id) {
            return self.op_outcome_session(outcome);
        }

        // Durable restart path: reconstruct from Trail + replay ledger.
        self.op_outcome_durable(outcome)
    }

    fn op_outcome_session(
        &mut self,
        outcome: crate::gate_protocol::OutcomePayload,
    ) -> Result<Value, GateError> {
        let record = self.committed.get(&outcome.execution_id).ok_or_else(|| {
            GateError::new(
                "outcome.unknown_execution",
                "execution_id is not a committed Gate execution",
            )
        })?;

        if let Some(existing) = &record.outcome_status {
            if existing == &outcome.classification {
                return Ok(json!({
                    "execution_id": outcome.execution_id,
                    "status": existing,
                    "idempotent": true,
                    "provider_invocations": self.provider_invocations,
                }));
            }
            return Err(GateError::new(
                "outcome.conflict",
                "a different outcome was already recorded for this execution",
            ));
        }

        if !outcome.attempted {
            return Err(GateError::new(
                "outcome.not_attempted",
                "a committed dispatch must report an attempted physical execution",
            ));
        }

        if let Some(identity) = &outcome.external_execution_identity {
            if identity == &record.action_id {
                return Err(GateError::new(
                    "outcome.identity_collapse",
                    "external execution identity must not equal the Tethers ActionId",
                ));
            }
        }

        // Successful results must pass the trusted capability output schema.
        let effective = classify_outcome(&record.output_schema, outcome)?;

        let action_id = record.action_id.clone();
        let evaluation_id = record.evaluation_id.clone();
        let capability_name = record.capability_name.clone();
        let capability_version = record.capability_version;
        let manifest_digest = record.manifest_digest.clone();
        let provider_identity = record.provider_identity.clone();
        let argument_digest = record.argument_digest.clone();
        let prepared_id = record.prepared_id.clone();

        let mut trail = self.open_trail()?;
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_millis() as u64)
            .unwrap_or_default();

        let entry = dispatch::OutcomeEntry {
            execution_id: effective.execution_id.clone(),
            action_id: action_id.clone(),
            status: effective.classification.clone(),
            result: effective.result.clone(),
            error_message: effective.error.clone(),
            reason_code: effective.reason_code.clone(),
            timestamp_unix_ms: timestamp_ms,
            semantic_position: None,
        };
        trail.append_outcome(&entry).map_err(|error| {
            GateError::new(
                "outcome.trail_failed",
                format!("outcome could not be recorded durably: {error}"),
            )
        })?;

        let outcome_digest = replay::durable_outcome_digest(&entry).map_err(|_| {
            GateError::new(
                "outcome.digest_failed",
                "outcome digest could not be computed",
            )
        })?;

        let terminal_state = match effective.classification.as_str() {
            "succeeded" => replay::ReplayState::Succeeded,
            "failed" => replay::ReplayState::Failed,
            _ => replay::ReplayState::Uncertain,
        };

        let record = self
            .committed
            .get_mut(&effective.execution_id)
            .expect("committed record checked above");

        let mut replay_terminal = "recorded";
        if let Some(mut guard) = record.guard.take() {
            if guard
                .publish_terminal(terminal_state, outcome_digest)
                .is_err()
            {
                replay_terminal = "recovery_required";
            }
            drop(guard);
        } else {
            replay_terminal = "recovery_required";
        }
        record.outcome_status = Some(effective.classification.clone());

        Ok(json!({
            "execution_id": effective.execution_id,
            "status": effective.classification,
            "attempted": effective.attempted,
            "external_execution_identity": effective.external_execution_identity,
            "evaluation_id": evaluation_id,
            "action_id": action_id,
            "capability": {
                "name": capability_name,
                "version": capability_version,
            },
            "manifest_digest": manifest_digest,
            "provider_identity": provider_identity,
            "argument_digest": argument_digest,
            "prepared_id": prepared_id,
            "replay_terminal": replay_terminal,
            "trail_outcome_recorded": true,
            "idempotent": false,
            "provider_invocations": self.provider_invocations,
        }))
    }

    /// Late OUTCOME after Gate restart: reconstruct durable committed truth
    /// from Trail intent + replay claim without the previous process map.
    fn op_outcome_durable(
        &mut self,
        outcome: crate::gate_protocol::OutcomePayload,
    ) -> Result<Value, GateError> {
        let view = reconcile_durable(&self.config.trail_path, &self.config.host_data_root);

        // Refuse recovery whenever the durable view cannot be fully trusted.
        if !view.trustworthy_for_recovery() {
            let reason = if view.trail_unavailable {
                "trail_unavailable"
            } else if view.trail_malformed {
                "trail_malformed"
            } else if view.truncated {
                "scan_truncated"
            } else if view.replay_unavailable {
                "replay_unavailable"
            } else {
                "durable_view_incomplete"
            };
            return Err(GateError::with_data(
                "outcome.unavailable",
                "durable Trail/replay view is not trustworthy for recovery",
                json!({ "reason": reason }),
            ));
        }

        let Some(intent) = view.intents.get(&outcome.execution_id) else {
            return Err(GateError::new(
                "outcome.unknown_execution",
                "execution_id has no durable committed intent",
            ));
        };

        if let Some(existing) = view.outcomes.get(&outcome.execution_id) {
            // A. Different classification is always a conflict, never
            // converted into success by reconciliation state.
            if existing.status != outcome.classification {
                return Err(GateError::new(
                    "outcome.conflict",
                    "a different outcome was already recorded for this execution",
                ));
            }

            // Target-specific recovery must be evaluated before any
            // idempotent acknowledgement. A known durable disagreement is
            // never blessed into successful idempotency.
            let target_states: Vec<String> = view
                .recovery_required
                .iter()
                .filter_map(|entry| {
                    if entry.get("execution_id").and_then(Value::as_str)
                        == Some(outcome.execution_id.as_str())
                    {
                        entry
                            .get("state")
                            .and_then(Value::as_str)
                            .map(str::to_owned)
                    } else {
                        None
                    }
                })
                .collect();

            if target_states.is_empty() {
                // B. Fully reconciled terminal: Trail and replay already agree.
                return Ok(json!({
                    "execution_id": outcome.execution_id,
                    "status": existing.status,
                    "idempotent": true,
                    "recovered": false,
                    "replay_terminal": "recorded",
                    "provider_invocations": self.provider_invocations,
                }));
            }

            // C. Sole supported crash window: terminal_replay_incomplete with
            // claim present, binding match, and replay still armed.
            let crash_window_only = target_states
                .iter()
                .all(|state| state == "terminal_replay_incomplete");
            let armed = view.replay_states.get(&outcome.execution_id)
                == Some(&replay::ReplayState::InvocationArmed);
            let claim_present = view.logical_keys.contains_key(&outcome.execution_id);
            let binding_ok = view.binding_matches_intent(&outcome.execution_id);
            if crash_window_only && armed && claim_present && binding_ok {
                let replay_terminal =
                    self.complete_armed_replay_terminal(&view, &outcome.execution_id, existing)?;
                return Ok(json!({
                    "execution_id": outcome.execution_id,
                    "status": existing.status,
                    "idempotent": true,
                    "recovered": true,
                    "replay_terminal": replay_terminal,
                    "provider_invocations": self.provider_invocations,
                }));
            }

            // D. Any other target durable disagreement refuses with an
            // explicit bounded reason. Never report replay_terminal=recorded.
            let reason = target_states
                .first()
                .cloned()
                .unwrap_or_else(|| "recovery_required".to_owned());
            return Err(GateError::with_data(
                "outcome.unavailable",
                "durable authorities disagree for this execution",
                json!({ "reason": reason }),
            ));
        }

        // New late OUTCOME: any recovery entry for this execution blocks
        // consequential recovery (ambiguity must refuse).
        if view.recovery_required.iter().any(|entry| {
            entry.get("execution_id").and_then(Value::as_str) == Some(outcome.execution_id.as_str())
        }) {
            let state = view
                .recovery_required
                .iter()
                .find_map(|entry| {
                    if entry.get("execution_id").and_then(Value::as_str)
                        == Some(outcome.execution_id.as_str())
                    {
                        entry
                            .get("state")
                            .and_then(Value::as_str)
                            .map(str::to_owned)
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "recovery_required".to_owned());
            return Err(GateError::with_data(
                "outcome.unavailable",
                "durable authorities disagree for this execution",
                json!({ "reason": state }),
            ));
        }

        // Late recovery requires the exact healthy crash-recovery shape:
        // intent + claim + matching binding + InvocationArmed + no outcome.
        if !view.logical_keys.contains_key(&outcome.execution_id) {
            return Err(GateError::with_data(
                "outcome.unavailable",
                "durable replay claim is missing for this execution",
                json!({ "reason": "missing_replay_claim" }),
            ));
        }
        if !view.binding_matches_intent(&outcome.execution_id) {
            return Err(GateError::with_data(
                "outcome.unavailable",
                "durable Trail/replay binding disagrees for this execution",
                json!({ "reason": "trail_replay_binding_disagreement" }),
            ));
        }
        match view.replay_states.get(&outcome.execution_id) {
            Some(replay::ReplayState::InvocationArmed) => {}
            Some(state) if is_terminal_replay_state(*state) => {
                return Err(GateError::with_data(
                    "outcome.unavailable",
                    "replay is already terminal without a matching recovery path",
                    json!({ "reason": "replay_terminal_without_trail_outcome" }),
                ));
            }
            Some(state) => {
                return Err(GateError::with_data(
                    "outcome.unavailable",
                    "replay claim is not armed for terminal publication",
                    json!({ "reason": "replay_not_armed", "replay_state": format!("{state:?}") }),
                ));
            }
            None => {
                return Err(GateError::with_data(
                    "outcome.unavailable",
                    "durable replay state is unavailable for this execution",
                    json!({ "reason": "replay_unavailable" }),
                ));
            }
        }

        if !outcome.attempted {
            return Err(GateError::new(
                "outcome.not_attempted",
                "a committed dispatch must report an attempted physical execution",
            ));
        }

        // Resolve trusted output schema from the durable intent's manifest pin.
        let runtime = self.load_runtime()?;
        let verified = runtime
            .trusted_store()
            .get_by_digest(&intent.manifest_digest)
            .ok_or_else(|| {
                GateError::new(
                    "outcome.unavailable",
                    "trusted manifest for durable intent is unavailable",
                )
            })?;
        let output_schema = verified.manifest().output_schema.clone();

        let effective = classify_outcome(&output_schema, outcome)?;

        // Re-admit the durable claim so terminal replay publication can complete.
        let logical_key = view
            .logical_keys
            .get(&effective.execution_id)
            .cloned()
            .ok_or_else(|| {
                GateError::new(
                    "outcome.unknown_execution",
                    "durable replay claim is missing for this execution",
                )
            })?;
        let binding = view
            .bindings
            .get(&effective.execution_id)
            .cloned()
            .ok_or_else(|| {
                GateError::new(
                    "outcome.unknown_execution",
                    "durable replay binding is missing for this execution",
                )
            })?;

        let replay_authority = FileReplayAuthority::new(Some(&self.config.host_data_root));
        let mut admission = replay_authority
            .admit(&logical_key, &binding)
            .map_err(|_| {
                GateError::new(
                    "outcome.unavailable",
                    "durable replay admission is unavailable",
                )
            })?;
        if admission.execution_id() != effective.execution_id {
            drop(admission);
            return Err(GateError::new(
                "outcome.unknown_execution",
                "replay claim does not match the reported execution identity",
            ));
        }

        let mut trail = self.open_trail()?;
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_millis() as u64)
            .unwrap_or_default();
        let entry = dispatch::OutcomeEntry {
            execution_id: effective.execution_id.clone(),
            action_id: intent.action_id.clone(),
            status: effective.classification.clone(),
            result: effective.result.clone(),
            error_message: effective.error.clone(),
            reason_code: effective.reason_code.clone(),
            timestamp_unix_ms: timestamp_ms,
            semantic_position: None,
        };
        trail.append_outcome(&entry).map_err(|error| {
            GateError::new(
                "outcome.trail_failed",
                format!("outcome could not be recorded durably: {error}"),
            )
        })?;

        let outcome_digest = replay::durable_outcome_digest(&entry).map_err(|_| {
            GateError::new(
                "outcome.digest_failed",
                "outcome digest could not be computed",
            )
        })?;
        let terminal_state = match effective.classification.as_str() {
            "succeeded" => replay::ReplayState::Succeeded,
            "failed" => replay::ReplayState::Failed,
            _ => replay::ReplayState::Uncertain,
        };

        // Recovered admissions are not fresh; terminal publication is allowed
        // only when durable state is already armed (COMMIT completed).
        let replay_terminal = if admission.state() == replay::ReplayState::InvocationArmed
            && admission
                .publish_terminal(terminal_state, outcome_digest)
                .is_ok()
        {
            "recorded"
        } else {
            "recovery_required"
        };
        drop(admission);

        Ok(json!({
            "execution_id": effective.execution_id,
            "status": effective.classification,
            "attempted": effective.attempted,
            "external_execution_identity": effective.external_execution_identity,
            "action_id": intent.action_id,
            "capability": {
                "name": intent.capability_name,
                "version": intent.capability_version,
            },
            "manifest_digest": intent.manifest_digest,
            "provider_identity": intent.provider_identity,
            "argument_digest": intent.argument_digest,
            "recovered": true,
            "replay_terminal": replay_terminal,
            "trail_outcome_recorded": true,
            "idempotent": false,
            "provider_invocations": self.provider_invocations,
        }))
    }

    // -----------------------------------------------------------------------
    // STATUS — bounded reconciliation surface for a reconnecting Host.
    // -----------------------------------------------------------------------

    fn op_status(&self) -> Result<Value, GateError> {
        let mut pending_approvals = Vec::new();
        let mut unresolved = Vec::new();
        let mut terminal = Vec::new();
        let mut truncated = false;

        // One canonical durable reconciliation shared with late OUTCOME.
        let view = reconcile_durable(&self.config.trail_path, &self.config.host_data_root);
        for (execution_id, intent) in &view.intents {
            let evaluation_id = view
                .bindings
                .get(execution_id)
                .map(|binding| binding.evaluation_id.as_str())
                .unwrap_or_default();
            let has_outcome = view.outcomes.contains_key(execution_id);
            // Terminal observation is reported even when replay publication
            // is incomplete; recovery_required carries the incompleteness.
            let state = if has_outcome {
                "TERMINAL_KNOWN"
            } else {
                "COMMITTED_OUTCOME_INCOMPLETE"
            };
            let summary = json!({
                "execution_id": execution_id,
                "action_id": intent.action_id,
                "evaluation_id": evaluation_id,
                "capability": {
                    "name": intent.capability_name,
                    "version": intent.capability_version,
                },
                "argument_digest": intent.argument_digest,
                "state": state,
            });
            // Ordinary unresolved only for healthy armed commits; incomplete
            // durable shapes stay out of unresolved_commits.
            if has_outcome {
                if terminal.len() < MAX_STATUS_ENTRIES {
                    terminal.push(summary);
                } else {
                    truncated = true;
                }
            } else if view.is_unresolved_armed_commit(execution_id) {
                if unresolved.len() < MAX_STATUS_ENTRIES {
                    unresolved.push(summary);
                } else {
                    truncated = true;
                }
            }
        }

        // In-memory session records that are not yet durable-merged.
        for record in self.committed.values() {
            if view.intents.contains_key(&record.execution_id) {
                continue;
            }
            let summary = json!({
                "execution_id": record.execution_id,
                "action_id": record.action_id,
                "evaluation_id": record.evaluation_id,
                "capability": {
                    "name": record.capability_name,
                    "version": record.capability_version,
                },
                "argument_digest": record.argument_digest,
                "prepared_id": record.prepared_id,
                "outcome": record.outcome_status,
                "state": if record.outcome_status.is_some() {
                    "TERMINAL_KNOWN"
                } else {
                    "COMMITTED_OUTCOME_INCOMPLETE"
                },
            });
            if record.outcome_status.is_some() {
                if terminal.len() < MAX_STATUS_ENTRIES {
                    terminal.push(summary);
                } else {
                    truncated = true;
                }
            } else if unresolved.len() < MAX_STATUS_ENTRIES {
                unresolved.push(summary);
            } else {
                truncated = true;
            }
        }

        let mut recovery_required = view.recovery_required.clone();
        if recovery_required.len() > MAX_STATUS_ENTRIES {
            recovery_required.truncate(MAX_STATUS_ENTRIES);
            truncated = true;
        }
        let integrity = view.integrity();
        truncated = truncated || view.truncated;

        // Process-local approvals: expose only live identities, never proofs.
        for record_id in self.approval_ids() {
            if let Ok(record) = self.approvals.record(&record_id) {
                if matches!(
                    record.state,
                    ApprovalState::Pending | ApprovalState::Approved
                ) && pending_approvals.len() < MAX_STATUS_ENTRIES
                {
                    pending_approvals.push(json!({
                        "approval_id": record.approval_id,
                        "state": state_name(record.state),
                        "action_id": record.proof.action_id,
                    }));
                }
            }
        }

        let mut prepared_views = Vec::new();
        for record in self.prepared.values() {
            if prepared_views.len() >= MAX_STATUS_ENTRIES {
                truncated = true;
                break;
            }
            prepared_views.push(json!({
                "prepared_id": record.prepared_id,
                "decision": record.prepare_decision,
                "committed": record.committed,
                "action_id": record.action.action_id,
                "tether_id": record.tether_id,
                "tether_version": record.tether_version,
                "config_digest": record.config_digest,
            }));
        }

        Ok(json!({
            "protocol": AUTHORITY_PROTOCOL,
            "product_version": self.product_version,
            "gate_instance_id": self.gate_instance_id,
            "healthy": true,
            "durable_reconciliation": {
                "state": integrity.as_str(),
                "reconciliation_complete": view.reconciliation_complete(),
                "truncated": view.truncated,
                "trail_unavailable": view.trail_unavailable,
                "replay_unavailable": view.replay_unavailable,
                "trail_malformed": view.trail_malformed,
            },
            "shutdown_requested": self.shutdown_requested,
            "provider_invocations": self.provider_invocations,
            "pending_approvals": pending_approvals,
            "unresolved_commits": unresolved,
            "terminal_outcomes": terminal,
            "recovery_required": recovery_required,
            "truncated": truncated,
            "prepared": prepared_views,
            "prepared_count": self.prepared.len(),
            "committed_count": self.committed.len(),
        }))
    }

    fn approval_ids(&self) -> Vec<String> {
        // ApprovalStore has no public iterator; probe sequential identities
        // created this session. Restart clears the store, matching expiry.
        (1..=4096)
            .map(|index| format!("approval-{index}"))
            .filter(|id| self.approvals.record(id).is_ok())
            .collect()
    }

    fn load_runtime(&self) -> Result<PreparedRuntime, GateError> {
        let loaded = load_runtime_config(&self.config.config_path).map_err(|_| {
            GateError::new(
                "gate.config_unavailable",
                "runtime configuration cannot be loaded",
            )
        })?;
        prepare_runtime(&loaded).map_err(|_| {
            GateError::new(
                "gate.config_unavailable",
                "runtime configuration cannot be prepared",
            )
        })
    }

    fn open_trail(&self) -> Result<FileTrail, GateError> {
        if let Some(parent) = Path::new(&self.config.trail_path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        FileTrail::open(self.config.trail_path.clone()).map_err(|error| {
            GateError::new(
                "gate.trail_unavailable",
                format!("trail cannot be opened: {error}"),
            )
        })
    }

    /// Complete replay terminal publication for a durable Trail outcome whose
    /// replay claim remains InvocationArmed (crash between Trail append and
    /// publish_terminal). Does not re-execute the physical effect.
    fn complete_armed_replay_terminal(
        &self,
        view: &DurableReconciliation,
        execution_id: &str,
        existing: &DurableOutcome,
    ) -> Result<&'static str, GateError> {
        let logical_key = view
            .logical_keys
            .get(execution_id)
            .cloned()
            .ok_or_else(|| {
                GateError::with_data(
                    "outcome.unavailable",
                    "durable replay claim is missing for terminal completion",
                    json!({ "reason": "missing_replay_claim" }),
                )
            })?;
        let binding = view.bindings.get(execution_id).cloned().ok_or_else(|| {
            GateError::with_data(
                "outcome.unavailable",
                "durable replay binding is missing for terminal completion",
                json!({ "reason": "trail_replay_binding_disagreement" }),
            )
        })?;
        if !view.binding_matches_intent(execution_id) {
            return Err(GateError::with_data(
                "outcome.unavailable",
                "durable Trail/replay binding disagrees for terminal completion",
                json!({ "reason": "trail_replay_binding_disagreement" }),
            ));
        }

        let outcome_digest = replay::durable_outcome_digest(&existing.entry).map_err(|_| {
            GateError::new(
                "outcome.digest_failed",
                "outcome digest could not be computed",
            )
        })?;
        let terminal_state = match existing.status.as_str() {
            "succeeded" => replay::ReplayState::Succeeded,
            "failed" => replay::ReplayState::Failed,
            _ => replay::ReplayState::Uncertain,
        };

        let replay_authority = FileReplayAuthority::new(Some(&self.config.host_data_root));
        let mut admission = replay_authority
            .admit(&logical_key, &binding)
            .map_err(|_| {
                GateError::new(
                    "outcome.unavailable",
                    "durable replay admission is unavailable",
                )
            })?;
        if admission.execution_id() != execution_id {
            drop(admission);
            return Err(GateError::new(
                "outcome.unknown_execution",
                "replay claim does not match the reported execution identity",
            ));
        }
        let result = if admission.state() == replay::ReplayState::InvocationArmed
            && admission
                .publish_terminal(terminal_state, outcome_digest)
                .is_ok()
        {
            "recorded"
        } else {
            "recovery_required"
        };
        drop(admission);
        Ok(result)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct GateError {
    pub code: &'static str,
    pub message: String,
    pub data: Option<Value>,
}

impl GateError {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }

    fn with_data(code: &'static str, message: impl Into<String>, data: Value) -> Self {
        Self {
            code,
            message: message.into(),
            data: Some(data),
        }
    }
}

fn frame_err(error: crate::gate_protocol::FrameError) -> GateError {
    GateError::new(error.code(), error.to_string())
}

fn state_name(state: ApprovalState) -> &'static str {
    match state {
        ApprovalState::Pending => "pending",
        ApprovalState::Approved => "approved",
        ApprovalState::Denied => "denied",
        ApprovalState::Cancelled => "cancelled",
        ApprovalState::Invalidated => "invalidated",
        ApprovalState::Consumed => "consumed",
    }
}

fn reason_code(reason: &PolicyReason) -> &'static str {
    match reason {
        PolicyReason::EmptyIdentifier(_) => "empty_identifier",
        PolicyReason::MissingBridgePin(_) => "missing_bridge_pin",
        PolicyReason::UndeclaredCapability => "undeclared_capability",
        PolicyReason::InputSchemaViolation(_) => "input_schema_violation",
        PolicyReason::ScopeViolation => "scope_violation",
        PolicyReason::ScopeNotEstablished => "scope_not_established",
        PolicyReason::NoAdmittedManifest => "no_admitted_manifest",
        PolicyReason::ProviderUnavailable => "provider_unavailable",
        PolicyReason::ProviderIdentityMismatch => "provider_identity_mismatch",
        PolicyReason::ManifestDigestMismatch => "manifest_digest_mismatch",
        PolicyReason::HostPolicyDeny => "host_policy_deny",
        PolicyReason::ManifestRequiresConfirmation => "manifest_requires_confirmation",
        PolicyReason::HostPolicyAsk => "host_policy_ask",
        PolicyReason::HostPolicyAllow => "host_policy_allow",
        PolicyReason::UnsupportedPolicyConfiguration => "unsupported_policy_configuration",
    }
}

fn select_tether<'a>(
    runtime: &'a PreparedRuntime,
    tether_id: &str,
    tether_version: &str,
) -> Result<&'a crate::configured_runtime::PreparedTether, GateError> {
    let matches: Vec<_> = runtime
        .tethers()
        .iter()
        .filter(|tether| tether.id == tether_id && tether.version == tether_version)
        .collect();
    match matches.as_slice() {
        [tether] => Ok(tether),
        _ => Err(GateError::new(
            "prepare.tether_not_found",
            "selected Tether is not configured exactly once",
        )),
    }
}

fn availability_for(
    runtime: &PreparedRuntime,
    observations: &Observations,
) -> resolver::ProviderAvailability {
    match &observations.available_provider_identities {
        Some(identities) => {
            resolver::ProviderAvailability::from_identities(identities.iter().map(String::as_str))
        }
        None => resolver::ProviderAvailability::from_identities(
            runtime
                .providers()
                .iter()
                .map(|provider| provider.identity.as_str()),
        ),
    }
}

/// Select one Action from a Tethers Core-produced Plan. The Host never
/// supplies this Plan; caller-shaped JSON is not a Tethers Plan.
fn proposed_action_from_core_plan(
    evaluation_id: &str,
    plan: &Value,
    requested_action_id: &str,
) -> Result<ProposedAction, GateError> {
    let plan_id = plan
        .get("id")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| GateError::new("prepare.invalid_plan", "plan.id is required"))?
        .to_owned();
    let actions = plan
        .get("actions")
        .and_then(Value::as_array)
        .ok_or_else(|| GateError::new("prepare.invalid_plan", "plan.actions is required"))?;
    let action = actions
        .iter()
        .find(|action| action.get("action_id").and_then(Value::as_str) == Some(requested_action_id))
        .ok_or_else(|| {
            GateError::new(
                "prepare.action_not_found",
                "requested action_id is not present in the Core-produced Plan",
            )
        })?;

    let action_id = action
        .get("action_id")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| GateError::new("prepare.invalid_plan", "action.action_id is required"))?
        .to_owned();
    let capability_name = action
        .get("capability")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| GateError::new("prepare.invalid_plan", "action.capability is required"))?
        .to_owned();
    let manifest_digest = action
        .get("manifest_digest")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .filter(|value| !value.is_empty());
    let bridge_capability_version = action
        .get("bridge_capability_version")
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .filter(|value| *value > 0);
    let bridge_provider_identity = action
        .get("bridge_provider_identity")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .filter(|value| !value.is_empty());
    let arguments = action.get("arguments").cloned().unwrap_or(Value::Null);

    Ok(ProposedAction {
        evaluation_id: evaluation_id.to_owned(),
        plan_id,
        action_id,
        capability_name,
        manifest_digest,
        bridge_capability_version,
        bridge_provider_identity,
        arguments,
    })
}

/// Fill missing bridge pins on a Core-produced Action from the trusted
/// manifest store. Core owns Action identity and arguments; pins are the
/// host's admitted-manifest projection, never caller-supplied authority.
fn ensure_bridge_pins(
    mut action: ProposedAction,
    runtime: &PreparedRuntime,
    availability: &resolver::ProviderAvailability,
) -> Result<ProposedAction, GateError> {
    if action.manifest_digest.is_some()
        && action.bridge_capability_version.is_some()
        && action.bridge_provider_identity.is_some()
    {
        return Ok(action);
    }
    let version = action.bridge_capability_version.unwrap_or(1);
    let resolved = resolver::resolve_capability(
        runtime.trusted_store(),
        availability,
        &action.capability_name,
        version,
        None,
    )
    .map_err(|error| {
        GateError::new(
            "prepare.unavailable",
            format!("capability resolution failed: {error:?}"),
        )
    })?;
    if action.manifest_digest.is_none() {
        action.manifest_digest = Some(resolved.manifest_digest().to_owned());
    }
    if action.bridge_capability_version.is_none() {
        action.bridge_capability_version = Some(resolved.capability_version());
    }
    if action.bridge_provider_identity.is_none() {
        action.bridge_provider_identity = Some(resolved.provider_identity().to_owned());
    }
    Ok(action)
}

fn prepared_identity(
    prepare: &PreparePayload,
    action: &ProposedAction,
    event_id: &str,
    config_digest: &str,
) -> String {
    let material = json!({
        "protocol": AUTHORITY_PROTOCOL,
        "tether_id": prepare.run_input.tether.id,
        "tether_version": prepare.run_input.tether.version,
        "evaluation_id": action.evaluation_id,
        "plan_id": action.plan_id,
        "action_id": action.action_id,
        "event_id": event_id,
        "capability_name": action.capability_name,
        "capability_version": action.bridge_capability_version,
        "argument_digest": approval::digest(&action.arguments),
        "manifest_digest": action.manifest_digest,
        "provider_identity": action.bridge_provider_identity,
        "config_digest": config_digest,
        "core_planned": true,
    });
    let bytes = serde_json_canonicalizer::to_vec(&material).expect("canonical prepare material");
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("prep_{:x}", hasher.finalize())
}

fn digest_config(path: &Path) -> Result<String, GateError> {
    let bytes = std::fs::read(path)
        .map_err(|_| GateError::new("gate.config_unavailable", "cannot read runtime config"))?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(format!("sha256:{:x}", hasher.finalize()))
}

fn effect_summary(action: &ProposedAction) -> String {
    format!(
        "Dispatch {}@{:?} with argument digest {}",
        action.capability_name,
        action.bridge_capability_version,
        approval::digest(&action.arguments)
    )
}

/// One durable Trail intent reconstructed for restart recovery.
#[derive(Debug, Clone)]
struct DurableIntent {
    action_id: String,
    capability_name: String,
    capability_version: u32,
    manifest_digest: String,
    provider_identity: String,
    argument_digest: String,
}

/// One durable Trail outcome reconstructed for restart recovery.
#[derive(Debug, Clone)]
struct DurableOutcome {
    status: String,
    /// Full Trail OutcomeEntry JSON so the durable outcome digest can be
    /// recomputed and compared against replay terminal state.
    entry: Value,
}

/// Canonical integrity state of durable Trail + replay reconciliation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DurableIntegrity {
    /// Both authorities readable, scan complete, no disagreements.
    Healthy,
    /// Readable but disagreeing / malformed / truncated / incomplete.
    RecoveryRequired,
    /// An authority could not be read or opened; durable truth not established.
    Unavailable,
}

impl DurableIntegrity {
    fn as_str(self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::RecoveryRequired => "recovery_required",
            Self::Unavailable => "unavailable",
        }
    }
}

/// One canonical bounded reconciliation of Trail intent/outcome records against
/// the durable replay ledger. Shared by STATUS and late OUTCOME recovery so the
/// two paths cannot invent separate truth.
#[derive(Debug, Default)]
struct DurableReconciliation {
    intents: HashMap<String, DurableIntent>,
    outcomes: HashMap<String, DurableOutcome>,
    /// Replay claim material keyed by execution_id (only when replay opened).
    logical_keys: HashMap<String, replay::LogicalExecutionKey>,
    bindings: HashMap<String, replay::ExecutionBinding>,
    /// Reconstructed replay state keyed by execution_id.
    replay_states: HashMap<String, replay::ReplayState>,
    /// Replay terminal outcome digests keyed by execution_id.
    replay_outcome_digests: HashMap<String, String>,
    /// Explicit recovery entries (durable disagreement / integrity failure).
    recovery_required: Vec<Value>,
    /// True when the Trail scan hit the line bound before finishing the file.
    truncated: bool,
    /// True when Trail could not be read.
    trail_unavailable: bool,
    /// True when replay could not be opened or inspected.
    replay_unavailable: bool,
    /// True when any non-empty Trail line failed JSON parsing.
    trail_malformed: bool,
}

impl DurableReconciliation {
    fn integrity(&self) -> DurableIntegrity {
        if self.trail_unavailable || self.replay_unavailable {
            DurableIntegrity::Unavailable
        } else if self.trail_malformed || self.truncated || !self.recovery_required.is_empty() {
            DurableIntegrity::RecoveryRequired
        } else {
            DurableIntegrity::Healthy
        }
    }

    /// True when the durable scan finished and both authorities were readable.
    /// Does not imply every execution is fully reconciled.
    fn scan_complete(&self) -> bool {
        !self.trail_unavailable
            && !self.replay_unavailable
            && !self.trail_malformed
            && !self.truncated
    }

    /// Full durable reconciliation: scan complete and zero disagreements.
    fn reconciliation_complete(&self) -> bool {
        self.scan_complete() && self.recovery_required.is_empty()
    }

    /// Late OUTCOME may proceed only when the durable view is trustworthy and
    /// complete for the whole scan (any malformation/truncation poisons it).
    fn trustworthy_for_recovery(&self) -> bool {
        self.scan_complete()
    }

    fn push_recovery(&mut self, entry: Value) {
        if self.recovery_required.len() < MAX_STATUS_ENTRIES {
            self.recovery_required.push(entry);
        }
    }

    fn binding_matches_intent(&self, execution_id: &str) -> bool {
        let (Some(intent), Some(binding)) = (
            self.intents.get(execution_id),
            self.bindings.get(execution_id),
        ) else {
            return false;
        };
        intent.action_id == binding.action_id
            && intent.capability_name == binding.capability_name
            && intent.capability_version == binding.capability_version
            && intent.manifest_digest == binding.manifest_digest
            && intent.provider_identity == binding.provider_identity
            && intent.argument_digest == binding.argument_digest
    }

    /// Healthy crash-recovery unresolved commit: intent + matching claim +
    /// InvocationArmed + no Trail outcome.
    fn is_unresolved_armed_commit(&self, execution_id: &str) -> bool {
        self.intents.contains_key(execution_id)
            && !self.outcomes.contains_key(execution_id)
            && self.logical_keys.contains_key(execution_id)
            && self.binding_matches_intent(execution_id)
            && self.replay_states.get(execution_id) == Some(&replay::ReplayState::InvocationArmed)
    }
}

fn is_terminal_replay_state(state: replay::ReplayState) -> bool {
    matches!(
        state,
        replay::ReplayState::Succeeded
            | replay::ReplayState::Failed
            | replay::ReplayState::Uncertain
    )
}

const MAX_RECONCILE_TRAIL_LINES: usize = 10_000;

/// One canonical durable reconciliation used by STATUS and late OUTCOME.
/// Never repairs authority state; only observes and classifies disagreement.
fn reconcile_durable(trail_path: &Path, host_data_root: &Path) -> DurableReconciliation {
    let mut view = DurableReconciliation::default();

    // Trail read failure is not an empty Trail.
    match std::fs::read_to_string(trail_path) {
        Ok(text) => {
            let mut line_index = 0usize;
            for line in text.lines() {
                if line_index >= MAX_RECONCILE_TRAIL_LINES {
                    view.truncated = true;
                    break;
                }
                line_index += 1;
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                let value = match serde_json::from_str::<Value>(line) {
                    Ok(value) => value,
                    Err(_) => {
                        // Malformed durable Trail line: integrity cannot be
                        // fully established. Do not delete or rewrite it.
                        view.trail_malformed = true;
                        view.push_recovery(json!({
                            "state": "trail_malformed",
                            "line": line_index,
                            "bounded_reason": "trail_malformed",
                        }));
                        continue;
                    }
                };
                let execution_id = value
                    .get("execution_id")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned();
                if execution_id.is_empty() {
                    continue;
                }

                // IntentEntry: arguments present, no status/kind/timestamp.
                if value.get("capability_name").is_some()
                    && value.get("arguments").is_some()
                    && value.get("status").is_none()
                    && value.get("kind").is_none()
                    && value.get("timestamp_unix_ms").is_none()
                {
                    let argument_digest = value
                        .get("arguments")
                        .cloned()
                        .map(|arguments| approval::digest(&arguments))
                        .unwrap_or_default();
                    view.intents.insert(
                        execution_id.clone(),
                        DurableIntent {
                            action_id: value
                                .get("action_id")
                                .and_then(Value::as_str)
                                .unwrap_or_default()
                                .to_owned(),
                            capability_name: value
                                .get("capability_name")
                                .and_then(Value::as_str)
                                .unwrap_or_default()
                                .to_owned(),
                            capability_version: value
                                .get("capability_version")
                                .and_then(Value::as_u64)
                                .unwrap_or_default()
                                as u32,
                            manifest_digest: value
                                .get("manifest_digest")
                                .and_then(Value::as_str)
                                .unwrap_or_default()
                                .to_owned(),
                            provider_identity: value
                                .get("provider_identity")
                                .and_then(Value::as_str)
                                .unwrap_or_default()
                                .to_owned(),
                            argument_digest,
                        },
                    );
                    continue;
                }

                // OutcomeEntry: status + timestamp present.
                if value.get("status").and_then(Value::as_str).is_some()
                    && value.get("timestamp_unix_ms").is_some()
                {
                    view.outcomes.insert(
                        execution_id.clone(),
                        DurableOutcome {
                            status: value
                                .get("status")
                                .and_then(Value::as_str)
                                .unwrap_or_default()
                                .to_owned(),
                            entry: value,
                        },
                    );
                }
            }
            // Detect truncation when the file has more lines than the bound
            // even if the last scanned line was empty.
            if !view.truncated {
                let total = text.lines().count();
                if total > MAX_RECONCILE_TRAIL_LINES {
                    view.truncated = true;
                }
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            // First start: no Trail file yet is an empty durable view, not a
            // read failure.
        }
        Err(_) => {
            view.trail_unavailable = true;
            view.push_recovery(json!({
                "state": "trail_unavailable",
                "bounded_reason": "trail_unavailable",
            }));
        }
    }

    if view.truncated {
        view.push_recovery(json!({
            "state": "scan_truncated",
            "bounded_reason": "scan_truncated",
            "line_bound": MAX_RECONCILE_TRAIL_LINES,
        }));
    }

    // Replay ledger: open failure is never "no claim found".
    match crate::replay_store::ReplayLedger::open(host_data_root) {
        Ok(ledger) => match ledger.inspect_durable() {
            Ok(claims) => {
                for claim in claims {
                    view.logical_keys
                        .insert(claim.execution_id.clone(), claim.logical_key);
                    view.bindings
                        .insert(claim.execution_id.clone(), claim.binding);
                    view.replay_states
                        .insert(claim.execution_id.clone(), claim.state);
                    if let Some(digest) = claim.durable_outcome_digest {
                        view.replay_outcome_digests
                            .insert(claim.execution_id.clone(), digest);
                    }
                }
            }
            Err(replay::ReplayError::InvalidChain) => {
                view.replay_unavailable = true;
                view.push_recovery(json!({
                    "state": "replay_integrity_failure",
                    "bounded_reason": "replay_integrity_failure",
                }));
            }
            Err(_) => {
                view.replay_unavailable = true;
                view.push_recovery(json!({
                    "state": "replay_unavailable",
                    "bounded_reason": "replay_unavailable",
                }));
            }
        },
        Err(replay::ReplayError::InvalidChain) => {
            view.replay_unavailable = true;
            view.push_recovery(json!({
                "state": "replay_integrity_failure",
                "bounded_reason": "replay_integrity_failure",
            }));
        }
        Err(_) => {
            view.replay_unavailable = true;
            view.push_recovery(json!({
                "state": "replay_unavailable",
                "bounded_reason": "replay_unavailable",
            }));
        }
    }

    if view.replay_unavailable {
        return view;
    }

    // Intent without matching replay claim → durable disagreement.
    let missing_claim: Vec<String> = view
        .intents
        .keys()
        .filter(|execution_id| !view.logical_keys.contains_key(*execution_id))
        .cloned()
        .collect();
    for execution_id in missing_claim {
        view.push_recovery(json!({
            "execution_id": execution_id,
            "state": "missing_replay_claim",
        }));
    }

    // Replay claim without Trail intent → durable disagreement.
    let claim_only: Vec<String> = view
        .bindings
        .keys()
        .filter(|execution_id| !view.intents.contains_key(*execution_id))
        .cloned()
        .collect();
    for execution_id in claim_only {
        view.push_recovery(json!({
            "execution_id": execution_id,
            "state": "replay_claim_without_trail_intent",
        }));
    }

    // Binding comparison where both sides exist.
    let binding_mismatch: Vec<String> = view
        .intents
        .keys()
        .filter(|execution_id| view.logical_keys.contains_key(*execution_id))
        .filter(|execution_id| !view.binding_matches_intent(execution_id))
        .cloned()
        .collect();
    for execution_id in binding_mismatch {
        view.push_recovery(json!({
            "execution_id": execution_id,
            "state": "trail_replay_binding_disagreement",
        }));
    }

    // Terminal reconciliation comparisons.
    let intent_ids: Vec<String> = view.intents.keys().cloned().collect();
    for execution_id in intent_ids {
        let has_outcome = view.outcomes.contains_key(&execution_id);
        let replay_state = view.replay_states.get(&execution_id).copied();
        let binding_ok =
            view.bindings.contains_key(&execution_id) && view.binding_matches_intent(&execution_id);

        match (has_outcome, replay_state) {
            // Trail intent + outcome + replay still armed: physical observation
            // recorded, replay terminal publication incomplete.
            (true, Some(replay::ReplayState::InvocationArmed)) => {
                view.push_recovery(json!({
                    "execution_id": execution_id,
                    "state": "terminal_replay_incomplete",
                }));
            }
            // Trail intent + outcome + replay non-terminal non-armed.
            (true, Some(state)) if !is_terminal_replay_state(state) => {
                view.push_recovery(json!({
                    "execution_id": execution_id,
                    "state": "terminal_replay_incomplete",
                }));
            }
            // Trail terminal + replay terminal: compare classification and digest.
            (true, Some(state)) if is_terminal_replay_state(state) => {
                if let Some(outcome) = view.outcomes.get(&execution_id) {
                    let expected = match outcome.status.as_str() {
                        "succeeded" => replay::ReplayState::Succeeded,
                        "failed" => replay::ReplayState::Failed,
                        _ => replay::ReplayState::Uncertain,
                    };
                    if state != expected {
                        view.push_recovery(json!({
                            "execution_id": execution_id,
                            "state": "terminal_classification_mismatch",
                            "trail_status": outcome.status,
                            "replay_state": format!("{state:?}"),
                        }));
                    } else if binding_ok {
                        match replay::durable_outcome_digest(&outcome.entry) {
                            Ok(trail_digest) => {
                                let replay_digest = view.replay_outcome_digests.get(&execution_id);
                                if replay_digest.map(String::as_str) != Some(trail_digest.as_str())
                                {
                                    view.push_recovery(json!({
                                        "execution_id": execution_id,
                                        "state": "terminal_outcome_digest_mismatch",
                                    }));
                                }
                            }
                            Err(_) => {
                                view.push_recovery(json!({
                                    "execution_id": execution_id,
                                    "state": "terminal_outcome_digest_mismatch",
                                }));
                            }
                        }
                    }
                }
            }
            // Replay terminal + no Trail outcome.
            (false, Some(state)) if is_terminal_replay_state(state) => {
                view.push_recovery(json!({
                    "execution_id": execution_id,
                    "state": "replay_terminal_without_trail_outcome",
                }));
            }
            // Intent without outcome and replay not armed (not a healthy
            // unresolved armed commit).
            (false, Some(state)) if state != replay::ReplayState::InvocationArmed && binding_ok => {
                view.push_recovery(json!({
                    "execution_id": execution_id,
                    "state": "replay_not_armed",
                    "replay_state": format!("{state:?}"),
                }));
            }
            // Missing claim already reported above.
            (false, None) | (true, None) => {}
            // Healthy unresolved armed commit: no recovery entry.
            (false, Some(replay::ReplayState::InvocationArmed)) => {}
            // Unmatched arms already handled.
            _ => {}
        }
    }

    // Outcome without intent.
    let outcome_only: Vec<(String, String)> = view
        .outcomes
        .iter()
        .filter(|(execution_id, _)| !view.intents.contains_key(*execution_id))
        .map(|(execution_id, outcome)| (execution_id.clone(), outcome.status.clone()))
        .collect();
    for (execution_id, status) in outcome_only {
        view.push_recovery(json!({
            "execution_id": execution_id,
            "state": "outcome_without_intent",
            "status": status,
        }));
    }

    view
}

/// Validate a reported success against the trusted capability output schema.
/// An invalid claimed success becomes canonical `result_validation_failed`.
fn classify_outcome(
    output_schema: &Value,
    outcome: crate::gate_protocol::OutcomePayload,
) -> Result<crate::gate_protocol::OutcomePayload, GateError> {
    if outcome.classification != "succeeded" {
        return Ok(outcome);
    }
    let Some(result) = outcome.result.as_ref() else {
        return Err(GateError::new(
            "outcome.missing_result",
            "succeeded outcome requires result",
        ));
    };
    if validation::validate_output(output_schema, result).is_ok() {
        return Ok(outcome);
    }
    let reason = crate::outcome::validation_reason();
    Ok(crate::gate_protocol::OutcomePayload {
        classification: "failed".to_owned(),
        result: None,
        error: Some(reason.message.to_owned()),
        reason_code: Some(reason.code.to_owned()),
        ..outcome
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_invocations_always_zero() {
        let mut gate = AuthorityGate::new(GateConfig {
            config_path: PathBuf::from("unused.json"),
            engine_path: PathBuf::from("unused-engine"),
            trail_path: PathBuf::from("unused.trail.jsonl"),
            host_data_root: PathBuf::from("unused-root"),
        });
        assert_eq!(gate.provider_invocations(), 0);
        let hello = gate.handle("hello", "req-1", &Map::new());
        assert_eq!(hello.status, crate::gate_protocol::ResponseStatus::Ok);
        let result = hello.result.expect("hello result");
        assert_eq!(result["provider_invocations"], 0);
        assert_eq!(result["authority_granted"], false);
        assert_eq!(result["protocol"], AUTHORITY_PROTOCOL);
    }

    #[test]
    fn hello_rejects_unexpected_payload() {
        let mut gate = AuthorityGate::new(GateConfig {
            config_path: PathBuf::from("unused.json"),
            engine_path: PathBuf::from("unused-engine"),
            trail_path: PathBuf::from("unused.trail.jsonl"),
            host_data_root: PathBuf::from("unused-root"),
        });
        let mut payload = Map::new();
        payload.insert("evil".into(), json!(true));
        let response = gate.handle("hello", "req-2", &payload);
        assert_eq!(response.status, crate::gate_protocol::ResponseStatus::Error);
        assert_eq!(
            response.error.expect("error").code,
            "frame.unexpected_payload"
        );
    }

    #[test]
    fn shutdown_sets_flag() {
        let mut gate = AuthorityGate::new(GateConfig {
            config_path: PathBuf::from("unused.json"),
            engine_path: PathBuf::from("unused-engine"),
            trail_path: PathBuf::from("unused.trail.jsonl"),
            host_data_root: PathBuf::from("unused-root"),
        });
        assert!(!gate.shutdown_requested());
        gate.handle("shutdown", "req-3", &Map::new());
        assert!(gate.shutdown_requested());
    }

    #[test]
    fn prepare_without_config_fails_closed() {
        let mut gate = AuthorityGate::new(GateConfig {
            config_path: PathBuf::from("definitely-missing-gate-config.json"),
            engine_path: PathBuf::from("unused-engine"),
            trail_path: PathBuf::from("unused.trail.jsonl"),
            host_data_root: PathBuf::from("unused-root"),
        });
        let payload = json!({
            "action_id": "action_1",
            "evaluation_id": "eval_1",
            "tether": {"id": "t", "version": "1"},
            "event": {"id": "evt", "name": "coding.task_completed", "data": {}},
            "facts": {}
        });
        let map = payload.as_object().unwrap().clone();
        let response = gate.handle("prepare", "req-4", &map);
        assert_eq!(response.status, crate::gate_protocol::ResponseStatus::Error);
        let error = response.error.expect("error");
        assert!(
            error.code == "gate.config_unavailable" || error.code == "prepare.tether_not_found",
            "got {}",
            error.code
        );
    }
}
