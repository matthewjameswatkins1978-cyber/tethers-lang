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

type fact_snapshot = {
  key : Tethers_core.host_snapshot_key;
  value : Yojson.Safe.t;
}
(** A runtime-supplied evaluation Fact value, keyed by the host snapshot key
    declared in the Core program's [Evaluation_input] provenance.  Runtime
    Facts are keyed by [HostSnapshotKey], NOT by canonical FactId. *)

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
  facts : fact_snapshot list;
  (** Runtime-supplied evaluation Fact snapshots for entry guard evaluation.
      Each snapshot is keyed by the [HostSnapshotKey] declared in the
      corresponding [Evaluation_input] provenance.  Guard lookup is
      deterministic by [HostSnapshotKey]; 0 matches is an error, 2+ is an
      error. *)
}
(** The full runtime planning and evaluation context.  Bridges Core meaning
    with this occurrence's runtime data. *)

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
  (** A Together_origin member [origin_id] does not resolve to any planned
      Action.  Fail-closed: the group is never silently shrunk. *)
  | Unresolved_together_member of Tethers_core.origin_id
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
  | Unresolved_entry_guards
  (** The Core program declares entry guards but the lower-level [plan] or
      [plan_canonicalized] API was called without evaluating them first.
      Use [evaluate_canonicalized] for guarded programs. *)
  | Missing_fact_snapshot of Tethers_core.host_snapshot_key
  (** No runtime Fact snapshot exists for the [HostSnapshotKey] declared in
      an [Evaluation_input] provenance referenced by an entry guard. *)
  | Ambiguous_fact_snapshot of Tethers_core.host_snapshot_key
  (** More than one runtime Fact snapshot exists for the same
      [HostSnapshotKey].  The host must deduplicate Fact snapshots before
      supply. *)
  | Fact_snapshot_type_mismatch of Tethers_core.host_snapshot_key
  (** The runtime Fact snapshot JSON type does not match the declared Core
      scalar type (e.g. JSON string supplied for an [Integer_type] fact). *)
  | Invalid_guard_comparison of Tethers_core.fact_id
  (** The guard's declared Fact type and expected Core value cannot form a
      valid comparison with the given operator (e.g. [Contains] with a
      non-string expected value, or [Greater_than] with a non-integer
      expected value). *)
  | Missing_reception_anchor
  (** The canonical program has zero top-level [Anchor_origin] sites.
      Reception requires exactly one. *)
  | Ambiguous_reception_anchor
  (** The canonical program has two or more top-level [Anchor_origin] sites.
      Reception requires exactly one; the evaluator does not silently pick
      one by storage order. *)

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
    [ProgramDigest] so that semantic program identity is preserved.

    FAILS CLOSED with [Unresolved_entry_guards] if the program declares
    entry guards.  Use [evaluate_canonicalized] for guarded programs. *)

type canonical_evaluation =
  | Matched of canonical_plan
  | Not_matched
(** The outcome of evaluating a canonical Core program with entry guards
    against runtime Fact snapshots.

    - [Matched plan]: the event matched the canonical Anchor, all entry
      guards evaluated to true; the plan is produced with the preserved
      [ProgramDigest].
    - [Not_matched]: the event name did not match the canonical Anchor, or
      at least one valid guard evaluated to false; no Runtime Plan is
      produced.

    Missing, ambiguous, wrongly typed, or malformed runtime Facts produce
    an [Error] through the [planning_error] result, NOT [Not_matched].
    Event mismatch is a normal [Not_matched], not an error. *)

type runtime_event = {
  name : string;
  (** Exact event name to match against the canonical Anchor_origin. *)
  data : Yojson.Safe.t;
  (** Immutable event-data JSON snapshot. *)
}
(** A typed runtime event value for canonical evaluation. *)

type evaluation_context = {
  evaluation_id : string;
  (** Runtime execution occurrence identity. *)
  event : runtime_event;
  (** The triggering runtime event. *)
  capabilities : runtime_capability_projection list;
  (** Approved runtime Capability projections supplied by the host. *)
  facts : fact_snapshot list;
  (** Runtime-supplied evaluation Fact snapshots for entry guard evaluation.
      Each snapshot is keyed by the [HostSnapshotKey] declared in the
      corresponding [Evaluation_input] provenance. *)
}
(** High-level canonical evaluation context.  The caller supplies
    Human-world occurrence data (event name + event data) without knowing
    the canonical Anchor OriginId.  The evaluator maps it to Core
    identities internally. *)

val evaluate_canonicalized :
  Tethers_core_canonical.canonicalized ->
  evaluation_context ->
  (canonical_evaluation, planning_error) result
(** High-level canonical evaluation: reception → guards → plan.

    Evaluation order (semantic and REQUIRED):
    1. Anchor reception: locate the single top-level [Anchor_origin] in the
       canonical program.  0 → [Missing_reception_anchor]; 2+ →
       [Ambiguous_reception_anchor].
    2. Exact event name match: compare [context.event.name] with the
       canonical [Anchor_origin.event_name] using exact string equality.
       Mismatch → [Ok Not_matched].
    3. Entry guard evaluation: bind [context.event.data] to the canonical
       Anchor OriginId internally, then evaluate guards against
       [context.facts].
    4. Planning: produce the Runtime Plan if all guards pass.

    Wrong event + missing or malformed Fact snapshots → [Ok Not_matched]
    (the Tether was never awakened, so its Conditions are not evaluated).

    The caller must NOT supply the canonical Anchor OriginId; the evaluator
    derives it internally from the canonical program.  Event name and data
    are occurrence inputs and MUST NOT alter [ProgramDigest].

    For programs with zero entry guards, equivalent to reception + plan
    wrapped in [Matched]. *)