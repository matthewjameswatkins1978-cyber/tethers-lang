(** Core → Runtime Plan bridge.

    CORE-5B is the runtime-plan boundary slice from validated Tethers Core to
    the existing Runtime Plan representation ([Tethers_outcome.plan]).  It
    combines what Core means, this runtime occurrence, and approved host
    Capability projections into a concrete Runtime Plan request.

    Core defines the program.  Runtime instantiates an occurrence.  The bridge
    never executes Actions, never authorises, never repairs invalid Core, never
    infers missing semantics, and never reinterprets the program.  The Plan
    remains a request, not permission. *)

type runtime_capability_projection = {
  capability_id : Tethers_core.capability_id;
  (** Core capability identity this projection is approved for. *)
  contract_digest : Tethers_core.capability_contract_digest;
  (** Core-pinned contract digest the projection must match exactly. *)
  runtime : Tethers_protocol.capability;
  (** Existing runtime capability schema carrying the plan-relevant fields:
      name, version, effects, and optional bridge metadata (manifest digest,
      bridge capability version, bridge provider identity). *)
}
(** Approved runtime Capability projection keyed and pinned by the Core
    capability identity and contract digest.  The bridge never trusts the full
    manifest; it copies planning-relevant fields from this projection. *)

type planning_context = {
  evaluation_id : string;
  (** Runtime execution occurrence identity.  Occurrence-derived Plan and
      idempotency identities must derive from this value, never from
      [program_id]. *)
  capabilities : runtime_capability_projection list;
  (** Approved runtime Capability projections supplied by the host. *)
}

type planning_error =
  | Invalid_core of Tethers_core_validator.validation_error list
  (** Core failed [Tethers_core_validator.validate]; no plan is produced. *)
  | Missing_entry_origin
  (** Valid Core declares no [entry_origin]; there is no control path to
      start the sequential plan. *)
  | Incomplete_success_path of Tethers_core.origin_id
  (** A reachable sequential path ran out of continuation at this origin:
      runtime execution order must reach [Program_complete] explicitly.
      Running out of continuation is incomplete meaning, not completion. *)
  | Unsupported_together
  (** [Together_origin] present: physical concurrency is outside the
      sequential runtime-plan vocabulary. *)
  | Unsupported_batch
  (** [Batch_site] (or a [Batch_item_context] input) present: batch/item
      execution is outside the sequential runtime-plan vocabulary. *)
  | Unsupported_branch
  (** Branch-driven control flow present: outcome routing is outside the
      sequential [success_continuation] vocabulary. *)
  | Unsupported_role_binding
  (** [Fact_through_role] input binding present: role resolution is outside
      the runtime-plan vocabulary. *)
  | Unsupported_role_proxy
  (** A Fact with [Role_proxy] provenance is present: role resolution is
      outside the runtime-plan vocabulary. *)
  | Unsupported_fact_binding
  (** [Fact_from_origin] input binding present: the existing runtime plan
      carries concrete resolved argument values and has no event-data or
      fact-carrying vocabulary. *)
  | Unsupported_anchor_value
  (** [Anchor_value] input binding present: the existing runtime plan
      carries concrete resolved argument values and has no event-data
      vocabulary for anchor paths. *)
  | Unsupported_execution_constraint
  (** An execution constraint (e.g. [Deadline]) is present: the existing
      runtime-plan vocabulary has no field for it. *)
  | Unsupported_item_template
  (** [item_template] present: item/batch execution is outside the
      sequential runtime-plan vocabulary. *)
  | Missing_capability_projection of Tethers_core.capability_id
  (** No approved projection exists for this Core capability identity. *)
  | Capability_projection_identity_mismatch of Tethers_core.capability_id
  (** The contract digest is approved but only under a different Core
      capability identity; the bridge must not silently substitute it. *)
  | Capability_projection_digest_mismatch of Tethers_core.capability_id
  (** A projection exists for the capability identity but its pinned
      contract digest differs from the Core-pinned digest. *)
  | Capability_projection_incomplete of Tethers_core.capability_id
  (** The approved projection lacks required runtime metadata (empty
      capability name or version, or partially present bridge fields). *)
  | Flow_cycle of Tethers_core.origin_id list
  (** Defensive: unreachable for validated Core, which rejects success
      cycles.  Guards the walk against a hang. *)
  | Unresolved_origin of Tethers_core.origin_id
  (** Defensive: unreachable for validated Core, which rejects unknown
      entry origins and missing continuation targets. *)

val plan :
  Tethers_core.program ->
  planning_context ->
  (Tethers_outcome.plan, planning_error) result
(** Validate the Core program, reject unsupported constructs with the precise
    errors above, verify every planned capability against an approved pinned
    projection, then derive the sequential Runtime Plan by walking
    [entry_origin] → [success_continuation] → … → [Program_complete].

    Every reachable sequential path must terminate explicitly at
    [Program_complete]; a path that runs out of continuation fails with
    [Incomplete_success_path].  Execution order derives only from semantic
    control flow, never from [origin_sites] storage order.

    Each planned Action carries the existing Runtime Plan contract fields:
    [action_id], [idempotency_key] (derived from
    [context.evaluation_id]), [capability] and [capability_version] (from the
    approved projection), [arguments] (literal Core inputs as concrete
    values), [effects], and the projection's bridge metadata fields when
    present.  [plan.id] is [context.evaluation_id ^ "/plan"]; [program_id]
    remains Core logical identity and is never used as an occurrence identity.
    [required_effects] aggregates the planned capabilities' effects with
    deterministic first-occurrence uniqueness. *)
