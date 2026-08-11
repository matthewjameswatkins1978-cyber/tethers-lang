(** Core → Runtime Plan bridge.

    CORE-5A is the first executable vertical slice from validated Tethers
    Core meaning toward the existing Runtime Plan representation
    ([Tethers_outcome.plan]).  It consumes Core meaning: it never executes
    Actions, never authorises, never repairs invalid Core, never infers
    missing semantics, and never reinterprets the program.

    Sequential execution order is derived from the semantic control-flow
    graph ([entry_origin] then [success_continuation] edges, stopping at
    [Program_complete]), never from [origin_sites] storage order.

    The bridge reuses [Tethers_outcome.plan] as its Runtime Plan model.  It
    does not create a second competing model.  A valid plan contains only
    real Core content: every field is either genuine Core meaning or absent.
    No placeholder strings, no "TODO" values, and no fabricated evaluation
    IDs may enter a valid plan. *)

type planning_error =
  | Invalid_core of Tethers_core_validator.validation_error list
  (** Core failed [Tethers_core_validator.validate]; no plan is produced. *)
  | Missing_entry_origin
  (** Valid Core declares no [entry_origin]; there is no control path to
      start the sequential plan. *)
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
  | Flow_cycle of Tethers_core.origin_id list
  (** Defensive: unreachable for validated Core, which rejects success
      cycles.  Guards the walk against a hang. *)
  | Unresolved_origin of Tethers_core.origin_id
  (** Defensive: unreachable for validated Core, which rejects unknown
      entry origins and missing continuation targets. *)

val plan :
  Tethers_core.program ->
  (Tethers_outcome.plan, planning_error) result
(** Validate the Core program, reject unsupported constructs with the
    precise errors above, then derive the sequential Runtime Plan by
    walking [entry_origin] → [success_continuation] → … → [Program_complete].

    Every planned Action carries its [CapabilityId] and
    [CapabilityContractDigest] exactly as semantic atoms.  Literal Action
    inputs become concrete argument values.  [plan.id] is the program's
    logical identity ([program_id]); no evaluation ID is fabricated.
    [required_effects] and [groups] are empty because Core declares no
    effects in this bridge and Together execution is unsupported. *)
