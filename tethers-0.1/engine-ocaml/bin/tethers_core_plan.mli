(** Core → Runtime Plan bridge.

    CORE-6A is the runtime-plan boundary slice from validated Tethers Core to
    the existing Runtime Plan representation ([Tethers_outcome.plan]).  It
    combines what Core means, this runtime occurrence, approved host Capability
    projections, and runtime Anchor snapshot data into a concrete Runtime Plan
    request.

    Core defines the program.  Runtime instantiates an occurrence.  The bridge
    never executes Actions, never authorises, never repairs invalid Core, never
    infers missing semantics, and never reinterprets the program.  The Plan
    remains a request, not permission. *)

type anchor_snapshot = {
  origin_id : Tethers_core.origin_id;
  (** Core Anchor identity this snapshot is for. *)
  data : Yojson.Safe.t;
  (** Runtime-supplied JSON snapshot for this Anchor. *)
}
(** A runtime-supplied Anchor snapshot.  The bridge resolves [Anchor_value]
    bindings by finding the snapshot for the exact [origin_id] and traversing
    the requested path. *)

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
  anchors : anchor_snapshot list;
  (** Runtime-supplied Anchor snapshot data for resolving [Anchor_value]
      bindings.  Each snapshot is keyed by its Core [origin_id]. *)
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
  | Ambiguous_capability_projection of Tethers_core.capability_id
  (** More than one approved projection matches the exact Core capability
      identity and contract digest pair.  Planning must not silently select
      one; the host must deduplicate projections before supply. *)
  | Flow_cycle of Tethers_core.origin_id list
  (** Defensive: unreachable for validated Core, which rejects success
      cycles.  Guards the walk against a hang. *)
  | Unresolved_origin of Tethers_core.origin_id
  (** Defensive: unreachable for validated Core, which rejects unknown
      entry origins and missing continuation targets. *)
  | Missing_anchor_snapshot of Tethers_core.origin_id
  (** No snapshot exists for the Core Anchor [origin_id] referenced by an
      [Anchor_value] binding. *)
  | Ambiguous_anchor_snapshot of Tethers_core.origin_id
  (** More than one snapshot exists for the same Core Anchor [origin_id].
      The host must deduplicate snapshots before supply. *)
  | Anchor_path_missing of Tethers_core.origin_id * string list
  (** The requested path component does not exist in the Anchor snapshot. *)
  | Anchor_path_not_object of Tethers_core.origin_id * string list
  (** Traversal attempted to continue through a non-object value. *)
  | Unsupported_anchor_value_type of Tethers_core.origin_id * string list
  (** The terminal value at the resolved path is not a string, integer, or
      boolean and cannot be represented by the Runtime Plan argument
      vocabulary. *)

type canonical_plan = {
  program_digest : Tethers_core_canonical.program_digest;
  runtime_plan : Tethers_outcome.plan;
}
(** A Runtime Plan together with the semantic Core identity (ProgramDigest)
    that produced it.  This is not a second Runtime Plan model; [runtime_plan]
    remains the existing [Tethers_outcome.plan].  The wrapper carries the
    semantic Core identity alongside the runtime occurrence plan. *)

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
    values, plus [Anchor_value] inputs resolved through runtime Anchor
    snapshots), [effects], and the projection's bridge metadata fields when
    present.  [plan.id] is [context.evaluation_id ^ "/plan"]; [program_id]
    remains Core logical identity and is never used as an occurrence identity.
    [required_effects] aggregates the planned capabilities' effects with
    deterministic first-occurrence uniqueness. *)

val plan_canonicalized :
  Tethers_core_canonical.canonicalized ->
  planning_context ->
  (canonical_plan, planning_error) result
(** Plan from an already-canonicalised Core value.  The caller cannot
    accidentally pass non-canonical Core to this entry point: it requires a
    [canonicalized] value produced by [Tethers_core_canonical.canonicalize].

    Internally obtains the Core program through
    [Tethers_core_canonical.canonical_program] and delegates to [plan].
    Returns the existing Runtime Plan together with the canonical
    [ProgramDigest] so that semantic program identity is preserved. *)