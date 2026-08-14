// plan_execution.rs — deterministic C1 plan-level execution
//
// The reference host executes a matched plan as one deterministic schedule
// built from the flat source-order `plan.actions` array and the optional
// additive `plan.groups` array.  Ordinary sequential Actions keep the
// existing stop-on-first-non-success behaviour.  A `together` group
// (fan-out / join) attempts every member once in source order, regardless
// of whether an earlier sibling fails; only after every member reaches a
// terminal outcome does the group join.  The group joins successfully only
// when every member succeeded; any other outcome blocks every later item.
//
// Serial execution is the valid C1 reference schedule.  No physical
// parallelism is introduced; the serial behaviour matches what a future
// genuinely concurrent runtime would observe: failure stops at the join,
// not inside the fan-out.

use crate::dispatch::{GroupJoinEntry, SemanticPhase, SemanticPosition, Trail};
use crate::host_execution::ExecutionServiceResult;
use crate::SharedExecutionResult;
use serde_json::Value;
use std::collections::{HashMap, HashSet};

/// One validated `together` group decoded from `plan.groups`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanGroup {
    pub group_id: String,
    pub member_action_ids: Vec<String>,
}

/// One item of the deterministic execution schedule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanItem {
    /// One ordinary sequential Action.  A non-success outcome stops the plan.
    Sequential { action_index: usize },
    /// A `together` fan-out group.  Every member is attempted before the
    /// join is decided; a non-success join blocks all later items.
    Group {
        group_id: String,
        member_indexes: Vec<usize>,
    },
}

/// Build and validate the deterministic C1 execution schedule from the flat
/// `plan.actions` array and the optional additive `plan.groups` array.
///
/// `groups` absent means an ordinary sequential plan, exactly as pre-C1.
/// Malformed group metadata fails closed before any execution: unknown
/// member Action IDs, duplicate member IDs within one group, an Action
/// belonging to more than one group, duplicate group IDs, empty or
/// one-member groups, and members whose ordering contradicts the C1 plan
/// contract (a group's members must be contiguous and ascending in source
/// order) are all rejected.  Invalid group metadata is never silently
/// reinterpreted as sequential execution.
pub fn build_plan_schedule(
    actions: &[Value],
    groups: Option<&[Value]>,
) -> Result<Vec<PlanItem>, String> {
    if actions.is_empty() {
        return Err("plan had no actions".to_owned());
    }

    let mut action_indexes = HashMap::new();
    for (index, action) in actions.iter().enumerate() {
        let action_id = action
            .get("action_id")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| format!("plan action {index} had no action_id"))?;
        if action_indexes.insert(action_id, index).is_some() {
            return Err(format!("duplicate Action ID in plan: {action_id}"));
        }
    }

    let groups = match groups {
        None => {
            return Ok((0..actions.len())
                .map(|action_index| PlanItem::Sequential { action_index })
                .collect());
        }
        Some(groups) => groups,
    };

    let mut parsed_groups = Vec::with_capacity(groups.len());
    let mut seen_group_ids = HashSet::new();
    let mut member_owners = HashMap::new();
    for (group_position, group) in groups.iter().enumerate() {
        let group_object = group
            .as_object()
            .ok_or_else(|| format!("plan group {group_position} was not an object"))?;
        let group_id = group_object
            .get("group_id")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| format!("plan group {group_position} had no group_id"))?
            .to_owned();
        if !seen_group_ids.insert(group_id.clone()) {
            return Err(format!("duplicate group ID: {group_id}"));
        }
        let member_values = group_object
            .get("member_action_ids")
            .and_then(Value::as_array)
            .ok_or_else(|| format!("plan group {group_id} had no member_action_ids"))?;
        if member_values.len() < 2 {
            return Err(format!(
                "plan group {group_id} must contain at least two members"
            ));
        }
        let mut member_action_ids = Vec::with_capacity(member_values.len());
        let mut member_indexes = Vec::with_capacity(member_values.len());
        let mut seen_members = HashSet::new();
        for member in member_values {
            let member_id = member
                .as_str()
                .filter(|value| !value.is_empty())
                .ok_or_else(|| format!("plan group {group_id} member was not a string"))?;
            let member_index = action_indexes.get(member_id).copied().ok_or_else(|| {
                format!("plan group {group_id} member Action ID not found in plan: {member_id}")
            })?;
            if !seen_members.insert(member_index) {
                return Err(format!(
                    "plan group {group_id} contains duplicate member Action ID: {member_id}"
                ));
            }
            if let Some(owner) = member_owners.insert(member_index, group_id.clone()) {
                return Err(format!(
                    "Action ID {member_id} belongs to more than one group: {owner} and {group_id}"
                ));
            }
            member_action_ids.push(member_id.to_owned());
            member_indexes.push(member_index);
        }
        // The C1 plan contract requires members contiguous and ascending in
        // source order; anything else is a contradictory plan, not a plan we
        // may silently reorder.
        if !member_indexes
            .windows(2)
            .all(|window| window[1] == window[0] + 1)
        {
            return Err(format!(
                "plan group {group_id} members are not contiguous in source order"
            ));
        }
        parsed_groups.push(PlanGroup {
            group_id,
            member_action_ids,
        });
    }

    let group_by_first_index: HashMap<usize, &PlanGroup> = parsed_groups
        .iter()
        .map(|group| {
            let first_index = action_indexes[group.member_action_ids[0].as_str()];
            (first_index, group)
        })
        .collect();

    let mut items = Vec::new();
    let mut index = 0;
    while index < actions.len() {
        if let Some(group) = group_by_first_index.get(&index) {
            let member_indexes: Vec<usize> = group
                .member_action_ids
                .iter()
                .map(|member_id| action_indexes[member_id.as_str()])
                .collect();
            let count = member_indexes.len();
            items.push(PlanItem::Group {
                group_id: group.group_id.clone(),
                member_indexes,
            });
            index += count;
        } else {
            items.push(PlanItem::Sequential {
                action_index: index,
            });
            index += 1;
        }
    }
    Ok(items)
}

/// Whether one Action step counts as a success for a `together` join.
///
/// Completed and replay-blocked completed-success count as success: the
/// member reached a terminal success outcome, and a known-completed Action
/// must not block its group's join forever under existing idempotency rules.
/// Everything else — Failed, Uncertain, Denied, Unattempted, AuditFailed,
/// other replay states, or a service-level stop — is non-success.
pub fn step_succeeded(step: &ActionStep) -> bool {
    match step {
        ActionStep::Boundary(result) => matches!(
            result.outcome,
            crate::SharedExecutionOutcome::Completed
                | crate::SharedExecutionOutcome::Replay(
                    crate::replay_runtime::ReplayDispatchResult::BlockedCompletedSuccess
                )
        ),
        ActionStep::Stopped(_) => false,
    }
}

/// One Action's dispatch step: either it reached the shared execution
/// boundary, or it stopped before the boundary (policy, availability,
/// approval, or malformed-data result).
#[derive(Debug)]
pub enum ActionStep {
    /// The Action crossed the shared execution boundary.
    Boundary(SharedExecutionResult),
    /// The Action stopped before the boundary with a service result.
    Stopped(ExecutionServiceResult),
}

/// The host execution identity of a succeeded step, when it has one.
fn succeeded_execution_id(step: &ActionStep) -> Option<String> {
    match step {
        ActionStep::Boundary(result) => match &result.outcome {
            crate::SharedExecutionOutcome::Completed
            | crate::SharedExecutionOutcome::Replay(
                crate::replay_runtime::ReplayDispatchResult::BlockedCompletedSuccess,
            ) => result.execution_id.clone(),
            _ => None,
        },
        ActionStep::Stopped(result) => match result {
            ExecutionServiceResult::Completed { execution_id, .. }
            | ExecutionServiceResult::ReplayBlockedCompletedSuccess { execution_id, .. } => {
                execution_id.clone()
            }
            _ => None,
        },
    }
}

/// Execute a validated plan schedule serially through one production
/// per-Action dispatcher.
///
/// `execute_action` runs the Action at the given index through the host's
/// production dispatch boundary and reports whether it reached the boundary
/// and with which outcome.  The same `trail` handle is shared by every step
/// and by the join records so the whole plan is one sequential audit stream.
pub fn execute_plan(
    response: Value,
    items: &[PlanItem],
    actions: &[Value],
    evaluation_id: &str,
    trail: &mut dyn Trail,
    mut execute_action: impl FnMut(
        &mut Value,
        usize,
        &mut dyn Trail,
        &SemanticPosition,
    ) -> Result<SharedExecutionResult, ExecutionServiceResult>,
) -> ExecutionServiceResult {
    let mut response = response;
    let mut last_succeeded: Option<(String, Option<String>)> = None;
    let mut global_ordinal: u64 = 0;

    for item in items {
        match item {
            PlanItem::Sequential { action_index } => {
                let action_id = action_id_of(actions, *action_index);
                let position = SemanticPosition {
                    action_ordinal: global_ordinal,
                    group_id: None,
                    member_ordinal: None,
                    phase: SemanticPhase::Action,
                };
                global_ordinal += 1;
                let step = match execute_action(&mut response, *action_index, trail, &position) {
                    Ok(result) => ActionStep::Boundary(result),
                    Err(result) => ActionStep::Stopped(result),
                };
                if step_succeeded(&step) {
                    last_succeeded = Some((action_id, succeeded_execution_id(&step)));
                    continue;
                }
                return aggregate_step(step, evaluation_id, &action_id);
            }
            PlanItem::Group {
                group_id,
                member_indexes,
            } => {
                let mut steps: Vec<(String, ActionStep)> = Vec::with_capacity(member_indexes.len());
                for (member_ordinal, action_index) in member_indexes.iter().enumerate() {
                    let action_id = action_id_of(actions, *action_index);
                    let position = SemanticPosition {
                        action_ordinal: global_ordinal,
                        group_id: Some(group_id.clone()),
                        member_ordinal: Some(member_ordinal as u64),
                        phase: SemanticPhase::Member,
                    };
                    global_ordinal += 1;
                    let step = match execute_action(&mut response, *action_index, trail, &position)
                    {
                        Ok(result) => ActionStep::Boundary(result),
                        Err(result) => ActionStep::Stopped(result),
                    };
                    if step_succeeded(&step) {
                        last_succeeded = Some((action_id.clone(), succeeded_execution_id(&step)));
                    }
                    steps.push((action_id, step));
                }
                let joined = steps.iter().all(|(_, step)| step_succeeded(step));
                let member_action_ids: Vec<String> = member_indexes
                    .iter()
                    .map(|action_index| action_id_of(actions, *action_index))
                    .collect();
                // Join semantic position: action_ordinal is the ordinal after
                // the last group member, derived deterministically from the
                // flat Runtime Plan order.  This places the join at the group
                // boundary without depending on physical completion timing.
                let join_position = SemanticPosition {
                    action_ordinal: global_ordinal,
                    group_id: None,
                    member_ordinal: None,
                    phase: SemanticPhase::Join,
                };
                let join_entry = GroupJoinEntry {
                    evaluation_id: evaluation_id.to_owned(),
                    group_id: group_id.clone(),
                    member_action_ids: member_action_ids.clone(),
                    joined,
                    timestamp_unix_ms: crate::now_unix_ms(),
                    semantic_position: Some(join_position),
                };
                let sequence = response
                    .get("trail")
                    .and_then(Value::as_array)
                    .map(|trail| trail.len() as u64 + 1)
                    .unwrap_or(1);
                let presentation_entry = serde_json::json!({
                    "sequence": sequence,
                    "phase": "execution",
                    "kind": "group_joined",
                    "outcome": if joined { "success" } else { "non_success" },
                    "message": format!(
                        "{} group {} ({} members: {})",
                        if joined { "Joined" } else { "Group did not join" },
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
                if joined {
                    continue;
                }
                return steps
                    .into_iter()
                    .find(|(_, step)| !step_succeeded(step))
                    .map(|(action_id, step)| aggregate_step(step, evaluation_id, &action_id))
                    .unwrap_or_else(|| ExecutionServiceResult::AuditFailed {
                        evaluation_id: evaluation_id.to_owned(),
                        action_id: audit_action_id,
                        reason: "non-success join produced no non-success member".to_owned(),
                        execution_id: None,
                    });
            }
        }
    }

    let (action_id, execution_id) = match last_succeeded {
        Some(identity) => identity,
        None => {
            return ExecutionServiceResult::AuditFailed {
                evaluation_id: evaluation_id.to_owned(),
                action_id: String::new(),
                reason: "plan completed without any succeeded Action".to_owned(),
                execution_id: None,
            };
        }
    };
    ExecutionServiceResult::Completed {
        evaluation_id: evaluation_id.to_owned(),
        action_id,
        response,
        execution_id,
    }
}

/// The `action_id` of one planned Action.
fn action_id_of(actions: &[Value], action_index: usize) -> String {
    actions[action_index]
        .get("action_id")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned()
}

/// Aggregate one non-success step into the plan result, preserving the
/// member's exact outcome distinction (Failed, Uncertain, Denied, replay,
/// approval-required, …) rather than flattening it.
fn aggregate_step(
    step: ActionStep,
    evaluation_id: &str,
    action_id: &str,
) -> ExecutionServiceResult {
    match step {
        ActionStep::Stopped(result) => result,
        ActionStep::Boundary(result) => {
            crate::host_execution::HostExecutionService::map_shared_result(
                result,
                evaluation_id.to_owned(),
                action_id.to_owned(),
                // The Completed arm consumes `response`, and Completed is a
                // success outcome that never reaches this aggregation, so the
                // response placeholder is never materialised here.
                Value::Null,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // -----------------------------------------------------------------------
    // Schedule validation (malformed group metadata fails closed)
    // -----------------------------------------------------------------------

    fn action(value: &str) -> Value {
        json!({"action_id": value, "capability": "bunny"})
    }

    fn group(value: &str, members: &[&str]) -> Value {
        json!({"group_id": value, "member_action_ids": members})
    }

    fn sequential(actions: &[Value]) -> Vec<PlanItem> {
        (0..actions.len())
            .map(|action_index| PlanItem::Sequential { action_index })
            .collect()
    }

    #[test]
    fn groups_absent_produces_ordinary_sequential_plan() {
        let actions = vec![action("action_1"), action("action_2"), action("action_3")];
        let items = build_plan_schedule(&actions, None).unwrap();
        assert_eq!(items, sequential(&actions));
    }

    #[test]
    fn valid_group_builds_one_group_item() {
        let actions = vec![action("action_1"), action("action_2"), action("action_3")];
        let groups = vec![group("group_1", &["action_1", "action_2", "action_3"])];
        let items = build_plan_schedule(&actions, Some(&groups)).unwrap();
        assert_eq!(
            items,
            vec![PlanItem::Group {
                group_id: "group_1".to_owned(),
                member_indexes: vec![0, 1, 2],
            }]
        );
    }

    #[test]
    fn two_sibling_groups_with_sequential_items_build_correct_schedule() {
        let actions = vec![
            action("action_1"),
            action("action_2"),
            action("action_3"),
            action("action_4"),
            action("action_5"),
            action("action_6"),
            action("action_7"),
        ];
        let groups = vec![
            group("group_1", &["action_2", "action_3"]),
            group("group_2", &["action_5", "action_6"]),
        ];
        let items = build_plan_schedule(&actions, Some(&groups)).unwrap();
        assert_eq!(
            items,
            vec![
                PlanItem::Sequential { action_index: 0 },
                PlanItem::Group {
                    group_id: "group_1".to_owned(),
                    member_indexes: vec![1, 2],
                },
                PlanItem::Sequential { action_index: 3 },
                PlanItem::Group {
                    group_id: "group_2".to_owned(),
                    member_indexes: vec![4, 5],
                },
                PlanItem::Sequential { action_index: 6 },
            ]
        );
    }

    #[test]
    fn unknown_member_action_id_rejected() {
        let actions = vec![action("action_1"), action("action_2")];
        let groups = vec![group("group_1", &["action_1", "action_9"])];
        let error = build_plan_schedule(&actions, Some(&groups)).unwrap_err();
        assert!(
            error.contains("member Action ID not found in plan: action_9"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn duplicate_member_within_group_rejected() {
        let actions = vec![action("action_1"), action("action_2")];
        let groups = vec![group("group_1", &["action_1", "action_1"])];
        let error = build_plan_schedule(&actions, Some(&groups)).unwrap_err();
        assert!(
            error.contains("duplicate member Action ID: action_1"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn action_in_more_than_one_group_rejected() {
        let actions = vec![action("action_1"), action("action_2"), action("action_3")];
        let groups = vec![
            group("group_1", &["action_1", "action_2"]),
            group("group_2", &["action_2", "action_3"]),
        ];
        let error = build_plan_schedule(&actions, Some(&groups)).unwrap_err();
        assert!(
            error.contains("belongs to more than one group: group_1 and group_2"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn duplicate_group_id_rejected() {
        let actions = vec![
            action("action_1"),
            action("action_2"),
            action("action_3"),
            action("action_4"),
        ];
        let groups = vec![
            group("group_1", &["action_1", "action_2"]),
            group("group_1", &["action_3", "action_4"]),
        ];
        let error = build_plan_schedule(&actions, Some(&groups)).unwrap_err();
        assert!(
            error.contains("duplicate group ID: group_1"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn empty_group_rejected() {
        let actions = vec![action("action_1")];
        let groups = vec![group("group_1", &[])];
        let error = build_plan_schedule(&actions, Some(&groups)).unwrap_err();
        assert!(
            error.contains("must contain at least two members"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn one_member_group_rejected() {
        let actions = vec![action("action_1"), action("action_2")];
        let groups = vec![group("group_1", &["action_1"])];
        let error = build_plan_schedule(&actions, Some(&groups)).unwrap_err();
        assert!(
            error.contains("must contain at least two members"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn non_contiguous_members_rejected() {
        let actions = vec![action("action_1"), action("action_2"), action("action_3")];
        let groups = vec![group("group_1", &["action_1", "action_3"])];
        let error = build_plan_schedule(&actions, Some(&groups)).unwrap_err();
        assert!(
            error.contains("members are not contiguous in source order"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn reversed_member_order_rejected() {
        let actions = vec![action("action_1"), action("action_2")];
        let groups = vec![group("group_1", &["action_2", "action_1"])];
        let error = build_plan_schedule(&actions, Some(&groups)).unwrap_err();
        assert!(
            error.contains("members are not contiguous in source order"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn action_without_action_id_rejected() {
        let actions = vec![json!({"capability": "bunny"})];
        let error = build_plan_schedule(&actions, None).unwrap_err();
        assert!(
            error.contains("had no action_id"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn empty_actions_rejected() {
        let error = build_plan_schedule(&[], None).unwrap_err();
        assert_eq!(error, "plan had no actions");
    }

    #[test]
    fn malformed_group_entries_rejected() {
        let actions = vec![action("action_1"), action("action_2")];
        let not_object = vec![json!([1, 2])];
        assert!(build_plan_schedule(&actions, Some(&not_object)).is_err());
        let missing_group_id = vec![json!({"member_action_ids": ["action_1", "action_2"]})];
        assert!(build_plan_schedule(&actions, Some(&missing_group_id)).is_err());
        let missing_members = vec![json!({"group_id": "group_1"})];
        assert!(build_plan_schedule(&actions, Some(&missing_members)).is_err());
        let non_string_member = vec![json!({"group_id": "group_1", "member_action_ids": [1, 2]})];
        assert!(build_plan_schedule(&actions, Some(&non_string_member)).is_err());
    }

    // -----------------------------------------------------------------------
    // Three Bunny Breakfast 🐇🐇🐇 — production-path runtime crucible
    //
    // Windows-gated because the reference host's durable replay proof is
    // native Windows: every member Action crosses `execute_shared_boundary`
    // with a real FileTrail and a real provisioned FileReplayAuthority.
    // -----------------------------------------------------------------------

    #[cfg(windows)]
    mod tb_crucible {
        use super::*;
        use crate::dispatch::FileTrail;
        use crate::executor::CapabilityExecutor;
        use crate::outcome;
        use crate::policy::{self, CapabilityRequirement, HostLocalPolicy, PolicyRule};
        use crate::replay_runtime::FileReplayAuthority;
        use crate::resolver::{self, ProviderAvailability};
        use crate::trusted_store::TrustedManifestStore;
        use serde_json::json;
        use std::collections::HashMap;
        use std::path::Path;
        use std::time::Duration;

        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        enum BunnyOutcome {
            Succeed,
            Fail,
            Uncertain,
        }

        /// Deterministic scripted executor keyed by capability name.  `Succeed`
        /// returns a valid output; `Fail` reports an explicit provider error;
        /// `Uncertain` reports no final response, which the host classifies as
        /// uncertainty (an effect may have occurred).
        struct ThreeBunnyExecutor {
            outcomes: HashMap<String, BunnyOutcome>,
            attempts: Vec<String>,
        }

        impl ThreeBunnyExecutor {
            fn new(outcomes: HashMap<String, BunnyOutcome>) -> Self {
                Self {
                    outcomes,
                    attempts: Vec::new(),
                }
            }

            fn attempts(&self) -> &[String] {
                &self.attempts
            }
        }

        impl CapabilityExecutor for ThreeBunnyExecutor {
            fn provider_identity(&self) -> &str {
                "lantern-local"
            }

            fn execute(
                &mut self,
                ready: &crate::dispatch::DispatchReadyAction,
            ) -> Result<Value, String> {
                let capability = ready.capability_name().to_owned();
                self.attempts.push(capability.clone());
                match self.outcomes.get(&capability) {
                    Some(BunnyOutcome::Succeed) => Ok(json!({"status": "ok"})),
                    _ => Err(format!("{capability} did not hop")),
                }
            }

            fn execute_classified(
                &mut self,
                ready: &crate::dispatch::DispatchReadyAction,
                _remaining: Duration,
            ) -> Result<Value, outcome::ProviderDiagnostic> {
                let capability = ready.capability_name().to_owned();
                self.attempts.push(capability.clone());
                match self.outcomes.get(&capability) {
                    Some(BunnyOutcome::Succeed) => Ok(json!({"status": "ok"})),
                    Some(BunnyOutcome::Fail) => {
                        Err(outcome::ProviderDiagnostic::ExplicitProviderError)
                    }
                    Some(BunnyOutcome::Uncertain) => {
                        Err(outcome::ProviderDiagnostic::NoFinalResponse)
                    }
                    None => Err(outcome::ProviderDiagnostic::ExplicitProviderError),
                }
            }
        }

        fn bunny_manifest(capability_name: &str) -> Value {
            json!({
                "manifest_format_version": "1.0",
                "capability_name": capability_name,
                "capability_version": 1,
                "title": capability_name,
                "description": format!("Three Bunny Breakfast {capability_name}"),
                "input_schema": { "type": "object", "properties": {}, "required": [] },
                "output_schema": {
                    "type": "object",
                    "properties": { "status": { "type": "string" } },
                    "required": ["status"],
                    "additionalProperties": false
                },
                "effects": ["bunny.taste"],
                "permission_scope": { "kind": "path_prefix", "allowed_prefixes": ["bunny/"] },
                "reversibility": "compensatable",
                "determinism": "deterministic",
                "idempotency": {
                    "mechanism": "argument_key",
                    "argument_name": "idempotency_key",
                    "key_source": "evaluation_id/action_id"
                },
                "confirmation_policy": { "standing_permitted": true, "per_call_required": false },
                "timeout_ms": 10000,
                "retry_policy": {
                    "max_retries": 0,
                    "backoff_ms": 500,
                    "allowed_on": ["outcome_unknown"],
                    "requires_idempotency_proof": false
                },
                "provider": {
                    "identity": "lantern-local",
                    "display_name": "Three Bunny Breakfast",
                    "identity_source": "host_configuration",
                    "description": "Test."
                },
                "binding": { "kind": "mcp", "server_name": "bunny", "tool_name": "bunny_hop", "adapter": null },
            })
        }

        fn resolved_bunny(capability_name: &str) -> resolver::ResolvedCapability {
            let mut manifest = bunny_manifest(capability_name);
            let (_, digest) =
                crate::manifest::canonicalize_and_digest(&manifest.to_string()).unwrap();
            manifest["digest"] = json!(digest);
            let mut store = TrustedManifestStore::new();
            store
                .insert(crate::manifest::verify_manifest(&manifest.to_string()).unwrap())
                .unwrap();
            let availability = ProviderAvailability::from_identities(["lantern-local"]);
            resolver::resolve_capability(
                &store,
                &availability,
                capability_name,
                1,
                Some("lantern-local"),
            )
            .unwrap()
        }

        fn allow_decision_for(
            resolved: &resolver::ResolvedCapability,
        ) -> policy::PermissionDecision {
            let requirements = vec![CapabilityRequirement::new(
                resolved.capability_name().to_owned(),
                resolved.capability_version(),
            )];
            let policy = HostLocalPolicy::new(PolicyRule::Allow);
            policy::evaluate_permission_resolved(&requirements, resolved, &policy)
        }

        fn bunny_action(action_id: &str, capability: &str) -> Value {
            json!({
                "action_id": action_id,
                "idempotency_key": format!("eval_tb_001/{action_id}"),
                "capability": capability,
                "capability_version": "1.0.0",
                "arguments": {},
                "effects": ["bunny.taste"],
            })
        }

        fn bunny_response(actions: Vec<Value>, groups: Option<Vec<Value>>) -> Value {
            let mut plan = json!({
                "id": "eval_tb_001/plan",
                "required_effects": ["bunny.taste"],
                "actions": actions,
            });
            if let Some(groups) = groups {
                plan["groups"] = json!(groups);
            }
            json!({
                "protocol_version": "0.1",
                "evaluation_id": "eval_tb_001",
                "event_id": "evt_three_bunnies_001",
                "tether_id": "three-bunny-breakfast",
                "tether_version": "c1c-v1",
                "status": "matched",
                "plan": plan,
                "trail": [{
                    "sequence": 1,
                    "phase": "reception",
                    "kind": "event_received",
                    "outcome": "accepted",
                    "message": "Received morning.started"
                }],
            })
        }

        /// Harden a test directory's DACL to exactly the trusted writers the
        /// native Windows replay proof requires (current user, SYSTEM,
        /// Administrators), mirroring the production evidence fixtures.
        fn harden_directory_acl(path: &Path) {
            let script = format!(
            "$p='{}'; $identity=[System.Security.Principal.WindowsIdentity]::GetCurrent().Name; $acl=[System.Security.AccessControl.DirectorySecurity]::new(); $acl.SetAccessRuleProtection($true,$false); $inherit=[System.Security.AccessControl.InheritanceFlags]::ContainerInherit -bor [System.Security.AccessControl.InheritanceFlags]::ObjectInherit; foreach($t in @($identity,'NT AUTHORITY\\SYSTEM','BUILTIN\\Administrators')) {{ $acl.AddAccessRule([System.Security.AccessControl.FileSystemAccessRule]::new($t,'FullControl',$inherit,'None','Allow')) }}; Set-Acl -LiteralPath $p -AclObject $acl",
            path.to_string_lossy()
        );
            let status = std::process::Command::new("pwsh")
                .args(["-NoProfile", "-Command", &script])
                .status()
                .expect("pwsh must be available to harden the replay test root");
            assert!(status.success(), "replay test root ACL hardening failed");
        }

        /// Run a plan through the production execution loop: every Action goes
        /// through `execute_shared_boundary` with a real durable FileTrail and a
        /// real provisioned FileReplayAuthority, exactly like the production
        /// service route.
        struct BunnyRun {
            result: ExecutionServiceResult,
            response: Option<Value>,
            executor: ThreeBunnyExecutor,
            trail_path: std::path::PathBuf,
        }

        fn run_bunny_plan(
            actions: Vec<Value>,
            groups: Option<Vec<Value>>,
            outcomes: HashMap<String, BunnyOutcome>,
        ) -> BunnyRun {
            let root = std::env::temp_dir().join(format!("tethers-tb-{}", uuid::Uuid::new_v4()));
            std::fs::create_dir_all(&root).unwrap();
            harden_directory_acl(&root);
            assert_eq!(
                crate::replay_windows::provision_replay(&root),
                Ok(crate::replay_windows::ProvisionReplayOutcome::Provisioned),
                "replay root must be provisionable"
            );
            let trail_path =
                std::env::temp_dir().join(format!("tethers-tb-trail-{}", uuid::Uuid::new_v4()));

            let response = bunny_response(actions.clone(), groups.clone());
            let resolved_by_name: HashMap<String, resolver::ResolvedCapability> = actions
                .iter()
                .filter_map(|action| {
                    action
                        .get("capability")
                        .and_then(Value::as_str)
                        .map(|name| (name.to_owned(), resolved_bunny(name)))
                })
                .collect();

            let mut executor = ThreeBunnyExecutor::new(outcomes);
            let mut trail = FileTrail::open(&trail_path).unwrap();
            let result = execute_plan(
                response,
                &build_plan_schedule(&actions, groups.as_deref()).unwrap(),
                &actions,
                "eval_tb_001",
                &mut trail,
                |response: &mut Value,
                 action_index: usize,
                 trail: &mut dyn Trail,
                 _position: &SemanticPosition| {
                    let action = &actions[action_index];
                    let proposed = crate::extract_proposed_action_at(response, action_index)
                        .expect("valid bunny action");
                    let capability = proposed.capability_name.clone();
                    let resolved = &resolved_by_name[&capability];
                    let decision = allow_decision_for(resolved);
                    let mut replay_authority = FileReplayAuthority::new(Some(&root));
                    let clock = outcome::ProductionMonotonicClock::new();
                    let mut anchor_writer = crate::ResponseResultAnchorWriter;
                    let context = crate::InputEventContext::for_initial("evt_three_bunnies_001");
                    crate::execute_shared_boundary(
                        response,
                        action,
                        decision,
                        resolved,
                        trail,
                        &mut executor,
                        &context,
                        false,
                        &clock,
                        &mut replay_authority,
                        None,
                        &mut anchor_writer,
                        Some(_position),
                    )
                    .map_err(|error| ExecutionServiceResult::AuditFailed {
                        evaluation_id: "eval_tb_001".to_owned(),
                        action_id: proposed.action_id,
                        reason: format!("shared execution boundary failed: {error}"),
                        execution_id: None,
                    })
                },
            );

            let response = match &result {
                ExecutionServiceResult::Completed { response, .. } => Some(response.clone()),
                _ => None,
            };
            BunnyRun {
                result,
                response,
                executor,
                trail_path,
            }
        }

        /// Read the durable Trail as parsed JSON lines.
        fn read_trail(path: &Path) -> Vec<Value> {
            std::fs::read_to_string(path)
                .unwrap()
                .lines()
                .filter(|line| !line.trim().is_empty())
                .map(|line| serde_json::from_str(line).unwrap())
                .collect()
        }

        fn outcomes_in_trail(path: &Path) -> Vec<(String, String)> {
            read_trail(path)
                .into_iter()
                .filter_map(|entry| {
                    let action_id = entry.get("action_id")?.as_str()?.to_owned();
                    let status = entry.get("status")?.as_str()?.to_owned();
                    Some((action_id, status))
                })
                .collect()
        }

        fn group_joins_in_trail(path: &Path) -> Vec<(String, Vec<String>, bool)> {
            read_trail(path)
                .into_iter()
                .filter_map(|entry| {
                    let group_id = entry.get("group_id")?.as_str()?.to_owned();
                    let member_action_ids = entry
                        .get("member_action_ids")?
                        .as_array()?
                        .iter()
                        .filter_map(|member| member.as_str().map(str::to_owned))
                        .collect();
                    let joined = entry.get("joined")?.as_bool()?;
                    Some((group_id, member_action_ids, joined))
                })
                .collect()
        }

        const CARROT: &str = "carrot.fetch";
        const TOAST: &str = "toast.make";
        const COFFEE: &str = "coffee.brew";
        const REPORT: &str = "report.compose";

        fn bunny_group() -> Vec<Value> {
            vec![
                bunny_action("action_1", CARROT),
                bunny_action("action_2", TOAST),
                bunny_action("action_3", COFFEE),
            ]
        }

        // TB-00 — sequential control: A B C with B failing stops the plan.
        #[test]
        fn tb_00_sequential_stop_on_first_failure() {
            let actions = vec![
                bunny_action("action_1", CARROT),
                bunny_action("action_2", TOAST),
                bunny_action("action_3", COFFEE),
            ];
            let outcomes = HashMap::from([
                (CARROT.to_owned(), BunnyOutcome::Succeed),
                (TOAST.to_owned(), BunnyOutcome::Fail),
                (COFFEE.to_owned(), BunnyOutcome::Succeed),
            ]);
            let run = run_bunny_plan(actions, None, outcomes);

            assert!(
                matches!(
                    run.result,
                    ExecutionServiceResult::Failed { ref action_id, .. } if action_id == "action_2"
                ),
                "TB-00 result was: {:?}",
                run.result
            );
            assert_eq!(
                run.executor.attempts(),
                &["carrot.fetch", "toast.make"],
                "C must NOT be attempted after B fails in a sequential plan"
            );
            let outcomes = outcomes_in_trail(&run.trail_path);
            assert_eq!(outcomes.len(), 2, "exactly two durable outcomes");
            assert!(group_joins_in_trail(&run.trail_path).is_empty());
        }

        // TB-01 — three bunny success: every member succeeds, join succeeds,
        // the report Action after the group is attempted.
        #[test]
        fn tb_01_group_all_succeed_then_report_attempted() {
            let mut actions = bunny_group();
            actions.push(bunny_action("action_4", REPORT));
            let outcomes = HashMap::from([
                (CARROT.to_owned(), BunnyOutcome::Succeed),
                (TOAST.to_owned(), BunnyOutcome::Succeed),
                (COFFEE.to_owned(), BunnyOutcome::Succeed),
                (REPORT.to_owned(), BunnyOutcome::Succeed),
            ]);
            let groups = vec![group("group_1", &["action_1", "action_2", "action_3"])];
            let run = run_bunny_plan(actions, Some(groups), outcomes);

            assert!(
                matches!(run.result, ExecutionServiceResult::Completed { ref action_id, .. } if action_id == "action_4")
            );
            assert_eq!(
                run.executor.attempts(),
                &[
                    "carrot.fetch",
                    "toast.make",
                    "coffee.brew",
                    "report.compose"
                ],
                "all members and the report must be attempted in source order"
            );
            assert_eq!(
                group_joins_in_trail(&run.trail_path),
                vec![(
                    "group_1".to_owned(),
                    vec![
                        "action_1".to_owned(),
                        "action_2".to_owned(),
                        "action_3".to_owned()
                    ],
                    true,
                )],
                "durable Trail must record one successful join"
            );
            let response = run.response.expect("Completed result carries the response");
            let kinds: Vec<&str> = response["trail"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|entry| entry["kind"].as_str())
                .collect();
            assert!(
                kinds.iter().any(|kind| *kind == "group_joined"),
                "response Trail must contain a group_joined presentation entry"
            );
            assert!(
                kinds
                    .iter()
                    .filter(|kind| **kind == "action_completed")
                    .count()
                    == 4,
                "all four Actions must show action_completed"
            );
        }

        // TB-02 — the middle bunny falls over: every member is still attempted,
        // the join is non-success, and the report is NOT attempted.
        #[test]
        fn tb_02_middle_bunny_failure_stops_at_the_join() {
            let mut actions = bunny_group();
            actions.push(bunny_action("action_4", REPORT));
            let outcomes = HashMap::from([
                (CARROT.to_owned(), BunnyOutcome::Succeed),
                (TOAST.to_owned(), BunnyOutcome::Fail),
                (COFFEE.to_owned(), BunnyOutcome::Succeed),
                (REPORT.to_owned(), BunnyOutcome::Succeed),
            ]);
            let groups = vec![group("group_1", &["action_1", "action_2", "action_3"])];
            let run = run_bunny_plan(actions, Some(groups), outcomes);

            assert!(matches!(
                run.result,
                ExecutionServiceResult::Failed { ref action_id, .. } if action_id == "action_2"
            ));
            assert_eq!(
                run.executor.attempts(),
                &["carrot.fetch", "toast.make", "coffee.brew"],
                "the failing member must not stop the fan-out: coffee is still attempted"
            );
            assert!(
                !run.executor
                    .attempts()
                    .iter()
                    .any(|capability| capability == REPORT),
                "the report Action must NOT be attempted after a non-success join"
            );
            assert_eq!(
                group_joins_in_trail(&run.trail_path),
                vec![(
                    "group_1".to_owned(),
                    vec![
                        "action_1".to_owned(),
                        "action_2".to_owned(),
                        "action_3".to_owned()
                    ],
                    false,
                )],
                "durable Trail must record the non-success join"
            );
            let outcomes = outcomes_in_trail(&run.trail_path);
            assert_eq!(
                outcomes,
                vec![
                    ("action_1".to_owned(), "succeeded".to_owned()),
                    ("action_2".to_owned(), "failed".to_owned()),
                    ("action_3".to_owned(), "succeeded".to_owned()),
                ],
                "every member's durable outcome is preserved"
            );
        }

        // TB-03 — the first bunny falls over: same shape as TB-02.
        #[test]
        fn tb_03_first_bunny_failure_stops_at_the_join() {
            let mut actions = bunny_group();
            actions.push(bunny_action("action_4", REPORT));
            let outcomes = HashMap::from([
                (CARROT.to_owned(), BunnyOutcome::Fail),
                (TOAST.to_owned(), BunnyOutcome::Succeed),
                (COFFEE.to_owned(), BunnyOutcome::Succeed),
                (REPORT.to_owned(), BunnyOutcome::Succeed),
            ]);
            let groups = vec![group("group_1", &["action_1", "action_2", "action_3"])];
            let run = run_bunny_plan(actions, Some(groups), outcomes);

            assert!(matches!(
                run.result,
                ExecutionServiceResult::Failed { ref action_id, .. } if action_id == "action_1"
            ));
            assert_eq!(
                run.executor.attempts(),
                &["carrot.fetch", "toast.make", "coffee.brew"],
                "all members are attempted even when the first sibling fails"
            );
            assert!(
                !run.executor
                    .attempts()
                    .iter()
                    .any(|capability| capability == REPORT),
                "the report Action must NOT be attempted after a non-success join"
            );
            assert_eq!(
                group_joins_in_trail(&run.trail_path),
                vec![(
                    "group_1".to_owned(),
                    vec![
                        "action_1".to_owned(),
                        "action_2".to_owned(),
                        "action_3".to_owned()
                    ],
                    false,
                )],
            );
        }

        // TB-04 — one bunny is uncertain: all siblings still attempted, the join
        // is non-success, the later Action is blocked, and the uncertain outcome
        // is preserved (not flattened into failure).
        #[test]
        fn tb_04_uncertain_bunny_blocks_the_join_without_flattening() {
            let mut actions = bunny_group();
            actions.push(bunny_action("action_4", REPORT));
            let outcomes = HashMap::from([
                (CARROT.to_owned(), BunnyOutcome::Succeed),
                (TOAST.to_owned(), BunnyOutcome::Uncertain),
                (COFFEE.to_owned(), BunnyOutcome::Succeed),
                (REPORT.to_owned(), BunnyOutcome::Succeed),
            ]);
            let groups = vec![group("group_1", &["action_1", "action_2", "action_3"])];
            let run = run_bunny_plan(actions, Some(groups), outcomes);

            assert!(matches!(
                run.result,
                ExecutionServiceResult::Uncertain { ref action_id, .. } if action_id == "action_2"
            ));
            assert_eq!(
                run.executor.attempts(),
                &["carrot.fetch", "toast.make", "coffee.brew"],
                "all members are attempted when one sibling is uncertain"
            );
            assert!(
                !run.executor
                    .attempts()
                    .iter()
                    .any(|capability| capability == REPORT),
                "the report Action must NOT be attempted after a non-success join"
            );
            assert_eq!(
                group_joins_in_trail(&run.trail_path),
                vec![(
                    "group_1".to_owned(),
                    vec![
                        "action_1".to_owned(),
                        "action_2".to_owned(),
                        "action_3".to_owned()
                    ],
                    false,
                )],
            );
            let outcomes = outcomes_in_trail(&run.trail_path);
            assert!(
                outcomes
                    .iter()
                    .any(|(action_id, status)| action_id == "action_2" && status == "uncertain"),
                "the uncertain member outcome must be preserved in the durable Trail: {outcomes:?}"
            );
        }

        // No group-wide idempotency key appears in any produced record.
        #[test]
        fn tb_records_contain_no_group_wide_idempotency_key() {
            let actions = bunny_group();
            let outcomes = HashMap::from([
                (CARROT.to_owned(), BunnyOutcome::Succeed),
                (TOAST.to_owned(), BunnyOutcome::Succeed),
                (COFFEE.to_owned(), BunnyOutcome::Succeed),
            ]);
            let groups = vec![group("group_1", &["action_1", "action_2", "action_3"])];
            let run = run_bunny_plan(actions, Some(groups), outcomes);
            let raw = std::fs::read_to_string(&run.trail_path).unwrap();
            assert!(
                !raw.contains("group_1/action_") && !raw.contains("group-wide"),
                "no group-wide idempotency material may appear in durable records"
            );
            let response = run.response.expect("Completed result carries the response");
            assert!(
                response["plan"]["groups"][0]["group_id"] == json!("group_1"),
                "the planner's group declaration remains the only group-wide identity"
            );
        }

        // TB-05 — semantic position on sequential actions.
        #[test]
        fn tb_05_sequential_actions_carry_semantic_position() {
            let actions = vec![
                bunny_action("action_1", CARROT),
                bunny_action("action_2", TOAST),
            ];
            let outcomes = HashMap::from([
                (CARROT.to_owned(), BunnyOutcome::Succeed),
                (TOAST.to_owned(), BunnyOutcome::Succeed),
            ]);
            let run = run_bunny_plan(actions, None, outcomes);
            assert!(matches!(
                run.result,
                ExecutionServiceResult::Completed { .. }
            ));

            let entries = read_trail(&run.trail_path);
            let intents: Vec<&serde_json::Value> = entries
                .iter()
                .filter(|e| e.get("capability_name").is_some())
                .collect();
            assert_eq!(intents.len(), 2);
            assert_eq!(intents[0]["semantic_position"]["action_ordinal"], 0);
            assert_eq!(intents[0]["semantic_position"]["phase"], "action");
            assert!(intents[0]["semantic_position"].get("group_id").is_none());
            assert_eq!(intents[1]["semantic_position"]["action_ordinal"], 1);
            assert_eq!(intents[1]["semantic_position"]["phase"], "action");
        }

        // TB-06 — semantic position on Together group members.
        #[test]
        fn tb_06_together_members_carry_semantic_position_from_flat_plan_order() {
            let actions = bunny_group();
            let outcomes = HashMap::from([
                (CARROT.to_owned(), BunnyOutcome::Succeed),
                (TOAST.to_owned(), BunnyOutcome::Succeed),
                (COFFEE.to_owned(), BunnyOutcome::Succeed),
            ]);
            let groups = vec![group("group_1", &["action_1", "action_2", "action_3"])];
            let run = run_bunny_plan(actions, Some(groups), outcomes);
            assert!(matches!(
                run.result,
                ExecutionServiceResult::Completed { .. }
            ));

            let entries = read_trail(&run.trail_path);
            let intents: Vec<&serde_json::Value> = entries
                .iter()
                .filter(|e| e.get("capability_name").is_some())
                .collect();
            assert_eq!(intents.len(), 3);
            // Members have global action_ordinal from flat plan order.
            assert_eq!(intents[0]["semantic_position"]["action_ordinal"], 0);
            assert_eq!(intents[0]["semantic_position"]["group_id"], "group_1");
            assert_eq!(intents[0]["semantic_position"]["member_ordinal"], 0);
            assert_eq!(intents[0]["semantic_position"]["phase"], "member");

            assert_eq!(intents[1]["semantic_position"]["action_ordinal"], 1);
            assert_eq!(intents[1]["semantic_position"]["group_id"], "group_1");
            assert_eq!(intents[1]["semantic_position"]["member_ordinal"], 1);
            assert_eq!(intents[1]["semantic_position"]["phase"], "member");

            assert_eq!(intents[2]["semantic_position"]["action_ordinal"], 2);
            assert_eq!(intents[2]["semantic_position"]["group_id"], "group_1");
            assert_eq!(intents[2]["semantic_position"]["member_ordinal"], 2);
            assert_eq!(intents[2]["semantic_position"]["phase"], "member");
        }

        // TB-07 — intent and outcome carry matching semantic position.
        #[test]
        fn tb_07_intent_and_outcome_carry_matching_semantic_position() {
            let actions = vec![bunny_action("action_1", CARROT)];
            let outcomes = HashMap::from([(CARROT.to_owned(), BunnyOutcome::Succeed)]);
            let run = run_bunny_plan(actions, None, outcomes);
            assert!(matches!(
                run.result,
                ExecutionServiceResult::Completed { .. }
            ));

            let entries = read_trail(&run.trail_path);
            let intent = entries
                .iter()
                .find(|e| e.get("capability_name").is_some())
                .unwrap();
            let outcome = entries.iter().find(|e| e.get("status").is_some()).unwrap();
            assert_eq!(
                intent["semantic_position"]["action_ordinal"],
                outcome["semantic_position"]["action_ordinal"]
            );
            assert_eq!(
                intent["semantic_position"]["phase"],
                outcome["semantic_position"]["phase"]
            );
        }

        // TB-08 — physical append order is preserved (no semantic sorting).
        #[test]
        fn tb_08_physical_append_order_preserved() {
            let actions = bunny_group();
            let outcomes = HashMap::from([
                (CARROT.to_owned(), BunnyOutcome::Succeed),
                (TOAST.to_owned(), BunnyOutcome::Succeed),
                (COFFEE.to_owned(), BunnyOutcome::Succeed),
            ]);
            let groups = vec![group("group_1", &["action_1", "action_2", "action_3"])];
            let run = run_bunny_plan(actions, Some(groups), outcomes);
            assert!(matches!(
                run.result,
                ExecutionServiceResult::Completed { .. }
            ));

            let entries = read_trail(&run.trail_path);
            // Physical order: intent_1, outcome_1, intent_2, outcome_2, intent_3, outcome_3, join
            let kinds: Vec<String> = entries
                .iter()
                .map(|e| {
                    if e.get("capability_name").is_some() {
                        "intent".to_string()
                    } else if e.get("group_id").is_some() && e.get("joined").is_some() {
                        "join".to_string()
                    } else {
                        "outcome".to_string()
                    }
                })
                .collect();
            assert_eq!(
                kinds,
                vec!["intent", "outcome", "intent", "outcome", "intent", "outcome", "join"]
            );
        }

        // TB-09 — GroupJoinEntry remains after all member terminal outcomes.
        #[test]
        fn tb_09_group_join_appended_after_all_member_outcomes() {
            let actions = bunny_group();
            let outcomes = HashMap::from([
                (CARROT.to_owned(), BunnyOutcome::Succeed),
                (TOAST.to_owned(), BunnyOutcome::Fail),
                (COFFEE.to_owned(), BunnyOutcome::Succeed),
            ]);
            let groups = vec![group("group_1", &["action_1", "action_2", "action_3"])];
            let run = run_bunny_plan(actions, Some(groups), outcomes);

            let entries = read_trail(&run.trail_path);
            let join_idx = entries
                .iter()
                .position(|e| e.get("joined").is_some())
                .expect("join entry must exist");
            // Join must be after all 3 member outcomes.
            assert!(
                join_idx >= 6,
                "join at {join_idx} must be after 6 member entries"
            );
        }

        // TB-10 — Join semantic position is present and has phase "join".
        #[test]
        fn tb_10_join_semantic_position_present_and_phase_join() {
            let actions = bunny_group();
            let outcomes = HashMap::from([
                (CARROT.to_owned(), BunnyOutcome::Succeed),
                (TOAST.to_owned(), BunnyOutcome::Succeed),
                (COFFEE.to_owned(), BunnyOutcome::Succeed),
            ]);
            let groups = vec![group("group_1", &["action_1", "action_2", "action_3"])];
            let run = run_bunny_plan(actions, Some(groups), outcomes);

            let entries = read_trail(&run.trail_path);
            let join = entries
                .iter()
                .find(|e| e.get("joined").is_some())
                .expect("join entry must exist");
            let sp = join
                .get("semantic_position")
                .expect("join must have semantic_position");
            assert_eq!(sp["phase"], "join");
            assert_eq!(sp["action_ordinal"], 3);
            assert!(sp.get("group_id").is_none());
            assert!(sp.get("member_ordinal").is_none());
        }

        // TB-11 — Join semantic position ordinal equals last member ordinal + 1.
        #[test]
        fn tb_11_join_determinism_ordinal_equals_last_member_plus_one() {
            let actions = bunny_group();
            let outcomes = HashMap::from([
                (CARROT.to_owned(), BunnyOutcome::Succeed),
                (TOAST.to_owned(), BunnyOutcome::Succeed),
                (COFFEE.to_owned(), BunnyOutcome::Succeed),
            ]);
            let groups = vec![group("group_1", &["action_1", "action_2", "action_3"])];
            let run = run_bunny_plan(actions, Some(groups), outcomes);

            let entries = read_trail(&run.trail_path);
            let intents: Vec<&serde_json::Value> = entries
                .iter()
                .filter(|e| e.get("capability_name").is_some())
                .collect();
            let last_member_ordinal = intents.last().unwrap()["semantic_position"]
                ["action_ordinal"]
                .as_u64()
                .unwrap();

            let join = entries.iter().find(|e| e.get("joined").is_some()).unwrap();
            let join_ordinal = join["semantic_position"]["action_ordinal"]
                .as_u64()
                .unwrap();
            assert_eq!(
                join_ordinal,
                last_member_ordinal + 1,
                "join ordinal must equal last member ordinal + 1"
            );
        }

        // TB-12 — Mixed plan: sequential + group + sequential.
        // Flat action ordinals must be continuous across plan items.
        #[test]
        fn tb_12_flat_action_ordinals_continuous_across_plan_items() {
            let actions = vec![
                bunny_action("action_1", CARROT),
                bunny_action("action_2", TOAST),
                bunny_action("action_3", COFFEE),
                bunny_action("action_4", CARROT),
            ];
            let outcomes = HashMap::from([
                (CARROT.to_owned(), BunnyOutcome::Succeed),
                (TOAST.to_owned(), BunnyOutcome::Succeed),
                (COFFEE.to_owned(), BunnyOutcome::Succeed),
            ]);
            let groups = vec![group("group_1", &["action_2", "action_3"])];
            let run = run_bunny_plan(actions, Some(groups), outcomes);
            assert!(matches!(
                run.result,
                ExecutionServiceResult::Completed { .. }
            ));

            let entries = read_trail(&run.trail_path);
            let intents: Vec<&serde_json::Value> = entries
                .iter()
                .filter(|e| e.get("capability_name").is_some())
                .collect();
            // action_1 = ordinal 0, action_2 = ordinal 1, action_3 = ordinal 2,
            // action_4 = ordinal 3
            assert_eq!(intents[0]["semantic_position"]["action_ordinal"], 0);
            assert_eq!(intents[0]["semantic_position"]["phase"], "action");
            assert_eq!(intents[1]["semantic_position"]["action_ordinal"], 1);
            assert_eq!(intents[1]["semantic_position"]["phase"], "member");
            assert_eq!(intents[2]["semantic_position"]["action_ordinal"], 2);
            assert_eq!(intents[2]["semantic_position"]["phase"], "member");
            assert_eq!(intents[3]["semantic_position"]["action_ordinal"], 3);
            assert_eq!(intents[3]["semantic_position"]["phase"], "action");
        }

        // TB-13 — Flat action ordinal equals the actual flat Runtime Plan
        // action index for sequential actions (no group offset).
        #[test]
        fn tb_13_flat_action_ordinal_equals_plan_action_index() {
            let actions = vec![
                bunny_action("action_1", CARROT),
                bunny_action("action_2", TOAST),
            ];
            let outcomes = HashMap::from([
                (CARROT.to_owned(), BunnyOutcome::Succeed),
                (TOAST.to_owned(), BunnyOutcome::Succeed),
            ]);
            let run = run_bunny_plan(actions, None, outcomes);

            let entries = read_trail(&run.trail_path);
            let intents: Vec<&serde_json::Value> = entries
                .iter()
                .filter(|e| e.get("capability_name").is_some())
                .collect();
            // action_1 is at flat plan index 0, action_2 at index 1.
            assert_eq!(intents[0]["semantic_position"]["action_ordinal"], 0);
            assert_eq!(intents[1]["semantic_position"]["action_ordinal"], 1);
        }

        // TB-14 — Group member order follows member_action_ids, not canonical V2.
        #[test]
        fn tb_14_group_member_order_follows_runtime_plan_ids() {
            let actions = bunny_group();
            let outcomes = HashMap::from([
                (CARROT.to_owned(), BunnyOutcome::Succeed),
                (TOAST.to_owned(), BunnyOutcome::Succeed),
                (COFFEE.to_owned(), BunnyOutcome::Succeed),
            ]);
            let groups = vec![group("group_1", &["action_1", "action_2", "action_3"])];
            let run = run_bunny_plan(actions, Some(groups), outcomes);

            let entries = read_trail(&run.trail_path);
            let intents: Vec<&serde_json::Value> = entries
                .iter()
                .filter(|e| e.get("capability_name").is_some())
                .collect();
            // member_ordinal 0, 1, 2 must match Runtime Plan source order.
            assert_eq!(intents[0]["semantic_position"]["member_ordinal"], 0);
            assert_eq!(intents[1]["semantic_position"]["member_ordinal"], 1);
            assert_eq!(intents[2]["semantic_position"]["member_ordinal"], 2);
        }

        // TB-15 — Trail contains join semantic_position and preserves physical
        // order (no semantic sorting).
        #[test]
        fn tb_15_trail_contains_join_position_in_physical_order() {
            let actions = bunny_group();
            let outcomes = HashMap::from([
                (CARROT.to_owned(), BunnyOutcome::Succeed),
                (TOAST.to_owned(), BunnyOutcome::Succeed),
                (COFFEE.to_owned(), BunnyOutcome::Succeed),
            ]);
            let groups = vec![group("group_1", &["action_1", "action_2", "action_3"])];
            let run = run_bunny_plan(actions, Some(groups), outcomes);

            let entries = read_trail(&run.trail_path);
            assert_eq!(entries.len(), 7, "expected 7 trail entries");

            // Verify join entry has semantic_position with phase "join".
            let join = entries
                .iter()
                .find(|e| e.get("joined").is_some())
                .expect("join entry must exist");
            let sp = join
                .get("semantic_position")
                .expect("join must have semantic_position");
            assert_eq!(sp["phase"], "join");

            // Verify raw trail output preserves join position.
            let raw = std::fs::read_to_string(&run.trail_path).unwrap();
            assert!(
                raw.contains("\"phase\":\"join\""),
                "join position must appear in Trail: {raw}"
            );

            // Verify physical order: intents and outcomes appear interleaved
            // before the join (intent, outcome, intent, outcome, ... join).
            let kinds: Vec<String> = entries
                .iter()
                .map(|e| {
                    if e.get("capability_name").is_some() {
                        "intent".to_string()
                    } else if e.get("joined").is_some() {
                        "join".to_string()
                    } else {
                        "outcome".to_string()
                    }
                })
                .collect();
            assert_eq!(
                kinds,
                vec!["intent", "outcome", "intent", "outcome", "intent", "outcome", "join"],
                "physical append order must be preserved"
            );
        }
    }
}
