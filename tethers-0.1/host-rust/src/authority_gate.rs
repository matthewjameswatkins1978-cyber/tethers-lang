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
use crate::policy::{self, PermissionDecision, PolicyReason, ProposedAction};
use crate::replay;
use crate::replay_runtime::{
    FileReplayAuthority, ReplayAdmissionGuard, ReplayAuthority, ReplayDispatchResult,
};
use crate::resolver;
use crate::runtime_config::load_runtime_config;
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Session configuration supplied by the CLI.
#[derive(Debug, Clone)]
pub struct GateConfig {
    pub config_path: PathBuf,
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
    // PREPARE — current read-only authority precheck. Does not authorise.
    // -----------------------------------------------------------------------

    fn op_prepare(&mut self, payload: &Map<String, Value>) -> Result<Value, GateError> {
        let prepare = crate::gate_protocol::parse_prepare_payload(payload).map_err(frame_err)?;
        let runtime = self.load_runtime()?;
        let config_digest = digest_config(&self.config.config_path)?;

        let tether = select_tether(&runtime, &prepare.tether_id, &prepare.tether_version)?;
        let _ = tether;

        let action = proposed_action_from_plan(&prepare)?;
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

        let prepared_id = prepared_identity(&prepare, &action, &config_digest);

        // Replacing an identical prepared identity is refused: each prepare
        // creates a fresh opaque handle bound to this evaluation material.
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
                event_id: prepare.event_id.clone(),
                tether_id: prepare.tether_id.clone(),
                tether_version: prepare.tether_version.clone(),
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
            "evaluation_id": prepare.evaluation_id,
            "action_id": prepare.action_id,
            "tether_id": prepare.tether_id,
            "tether_version": prepare.tether_version,
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
                execution_id: consumed.proof.evaluation_id.clone(),
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

        let execution_id_str = admission.execution_id().to_owned();
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
            execution_id: outcome.execution_id.clone(),
            action_id: action_id.clone(),
            status: outcome.classification.clone(),
            result: outcome.result.clone(),
            error_message: outcome.error.clone(),
            reason_code: outcome.reason_code.clone(),
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

        let terminal_state = match outcome.classification.as_str() {
            "succeeded" => replay::ReplayState::Succeeded,
            "failed" => replay::ReplayState::Failed,
            _ => replay::ReplayState::Uncertain,
        };

        let record = self
            .committed
            .get_mut(&outcome.execution_id)
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
            // Guard lost (e.g. after restart). Physical outcome remains true;
            // durable replay terminal requires recovery. Never claim success
            // or failure beyond the recorded observation.
            replay_terminal = "recovery_required";
        }
        record.outcome_status = Some(outcome.classification.clone());

        Ok(json!({
            "execution_id": outcome.execution_id,
            "status": outcome.classification,
            "attempted": outcome.attempted,
            "external_execution_identity": outcome.external_execution_identity,
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

    // -----------------------------------------------------------------------
    // STATUS — bounded reconciliation surface for a reconnecting Host.
    // -----------------------------------------------------------------------

    fn op_status(&self) -> Result<Value, GateError> {
        let mut pending_approvals = Vec::new();
        let mut unresolved = Vec::new();
        let mut terminal = Vec::new();

        for record in self.committed.values() {
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
            });
            if record.outcome_status.is_some() {
                if terminal.len() < MAX_STATUS_ENTRIES {
                    terminal.push(summary);
                }
            } else if unresolved.len() < MAX_STATUS_ENTRIES {
                unresolved.push(summary);
            }
        }

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

        let trail_recovery = scan_trail_recovery(&self.config.trail_path);

        let mut prepared_views = Vec::new();
        for record in self.prepared.values() {
            if prepared_views.len() >= MAX_STATUS_ENTRIES {
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
            "shutdown_requested": self.shutdown_requested,
            "provider_invocations": self.provider_invocations,
            "pending_approvals": pending_approvals,
            "unresolved_commits": unresolved,
            "prepared": prepared_views,
            "terminal_outcomes": terminal,
            "recovery_required": trail_recovery,
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

fn proposed_action_from_plan(prepare: &PreparePayload) -> Result<ProposedAction, GateError> {
    let plan_id = prepare
        .plan
        .get("id")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| GateError::new("prepare.invalid_plan", "plan.id is required"))?
        .to_owned();
    let actions = prepare
        .plan
        .get("actions")
        .and_then(Value::as_array)
        .ok_or_else(|| GateError::new("prepare.invalid_plan", "plan.actions is required"))?;
    let action = actions
        .iter()
        .find(|action| {
            action.get("action_id").and_then(Value::as_str) == Some(prepare.action_id.as_str())
        })
        .ok_or_else(|| {
            GateError::new(
                "prepare.action_not_found",
                "action_id is not present in the supplied plan",
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
        evaluation_id: prepare.evaluation_id.clone(),
        plan_id,
        action_id,
        capability_name,
        manifest_digest,
        bridge_capability_version,
        bridge_provider_identity,
        arguments,
    })
}

fn prepared_identity(
    prepare: &PreparePayload,
    action: &ProposedAction,
    config_digest: &str,
) -> String {
    let material = json!({
        "protocol": AUTHORITY_PROTOCOL,
        "tether_id": prepare.tether_id,
        "tether_version": prepare.tether_version,
        "evaluation_id": action.evaluation_id,
        "plan_id": action.plan_id,
        "action_id": action.action_id,
        "event_id": prepare.event_id,
        "capability_name": action.capability_name,
        "capability_version": action.bridge_capability_version,
        "argument_digest": approval::digest(&action.arguments),
        "manifest_digest": action.manifest_digest,
        "provider_identity": action.bridge_provider_identity,
        "config_digest": config_digest,
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

/// Bounded recovery identities derived from durable Trail intent records that
/// have no matching outcome. Never repairs authority state by resetting it.
fn scan_trail_recovery(trail_path: &Path) -> Vec<Value> {
    let mut intents: HashMap<String, String> = HashMap::new();
    let mut outcomes: std::collections::HashSet<String> = std::collections::HashSet::new();

    let Ok(text) = std::fs::read_to_string(trail_path) else {
        return Vec::new();
    };
    for line in text.lines().take(10_000) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        let kind = value.get("kind").and_then(Value::as_str);
        let execution_id = value
            .get("execution_id")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if execution_id.is_empty() {
            continue;
        }
        match kind {
            Some("intent") | None => {
                // IntentEntry serialises without a kind field; presence of
                // capability_name distinguishes it from other records.
                if value.get("capability_name").is_some() && value.get("status").is_none() {
                    intents.insert(
                        execution_id.to_owned(),
                        value
                            .get("action_id")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_owned(),
                    );
                }
            }
            _ => {}
        }
        if value.get("status").and_then(Value::as_str).is_some()
            && value.get("action_id").is_some()
            && value.get("timestamp_unix_ms").is_some()
        {
            outcomes.insert(execution_id.to_owned());
        }
    }

    intents
        .into_iter()
        .filter(|(execution_id, _)| !outcomes.contains(execution_id))
        .take(MAX_STATUS_ENTRIES)
        .map(|(execution_id, action_id)| {
            json!({
                "execution_id": execution_id,
                "action_id": action_id,
                "state": "committed_outcome_incomplete",
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_invocations_always_zero() {
        let mut gate = AuthorityGate::new(GateConfig {
            config_path: PathBuf::from("unused.json"),
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
            trail_path: PathBuf::from("unused.trail.jsonl"),
            host_data_root: PathBuf::from("unused-root"),
        });
        let payload = json!({
            "tether_id": "t",
            "tether_version": "1",
            "evaluation_id": "e",
            "event_id": "evt",
            "action_id": "a",
            "plan": {"id": "p", "actions": []}
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
