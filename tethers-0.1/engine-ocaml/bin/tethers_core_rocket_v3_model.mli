(** Immutable typed semantic relation model for Rocket V3 R3-1.

    This module is a structural input for later refinement/search phases.  It
    does not assign canonical labels, emit Enc_V2, or produce a digest. *)

type family =
  | Origin
  | Fact
  | Branch
  | Batch
  | ItemTemplate
  | ScopedRole

type vertex_kind =
  | Anonymous of family
  | ProgramRoot
  | ProgramScope
  | ProgramComplete
  | BranchStop

type relation_kind =
  | Rel_root_entry_origin
  | Rel_success_next
  | Rel_success_complete
  | Rel_origin_declared_fact
  | Rel_batch_aggregate_fact
  | Rel_fact_provenance_origin
  | Rel_fact_provenance_role
  | Rel_entry_guard_fact
  | Rel_action_input_fact
  | Rel_action_input_origin
  | Rel_action_input_role
  | Rel_action_input_anchor
  | Rel_action_input_template
  | Rel_together_member
  | Rel_branch_subject
  | Rel_branch_target
  | Rel_branch_stop
  | Rel_role_contract_fact
  | Rel_role_scope_template
  | Rel_role_scope_program
  | Rel_template_origin
  | Rel_template_batch
  | Rel_template_branch
  | Rel_template_role
  | Rel_template_objective_role
  | Rel_program_input_fact
  | Rel_program_origin
  | Rel_program_batch
  | Rel_program_branch
  | Rel_program_role
  | Rel_program_template
  | Rel_fact_scope_owner
  | Rel_batch_template_context

type binding_kind =
  | Binding_fact_from_origin
  | Binding_fact_through_role
  | Binding_anchor_value
  | Binding_batch_item_context

type relation_discriminator =
  | Discriminator_none
  | Origin_anchor
  | Origin_action
  | Origin_together
  | Batch_site_aggregate
  | Provenance_origin
  | Provenance_role_proxy
  | Entry_guard
  | Action_binding of binding_kind
  | Together_member
  | Branch_outcome of Tethers_core.terminal_outcome
  | Branch_continue_to
  | Branch_stop_target
  | Success_continuation
  | Success_program_complete
  | Role_contract
  | Role_program_scope
  | Role_item_template_scope
  | Template_membership
  | Template_batch_membership
  | Template_objective
  | Program_input
  | Program_origin_membership
  | Program_batch_membership
  | Program_branch_membership
  | Program_role_membership
  | Program_template_membership
  | Fact_program_scope
  | Fact_template_scope
  | Fact_origin_scope
  | Fact_batch_scope
  | Batch_template_context

type edge = {
  target : int;
  relation : relation_kind;
  discriminator : relation_discriminator;
  payload : string;
}

type t

val build :
  Tethers_core.program ->
  (t, Tethers_core_validator.validation_error list) result
(** Validate first, then build the complete immutable model.  An invalid Core
    program returns the validator's errors and no partial model. *)

val vertex_count : t -> int
val vertex_kind : t -> int -> vertex_kind
val vertex_scalar : t -> int -> string
val vertex_family_count : t -> family -> int

(** Construction/leaf-encoding identity bridges.  These lookups expose the
    existing Core IDs only at the model boundary; IDs are not part of
    structural evidence or refinement decisions. *)
val find_origin_vertex : t -> Tethers_core.origin_id -> int option
val find_fact_vertex : t -> Tethers_core.fact_id -> int option
val find_branch_vertex : t -> Tethers_core.branch_id -> int option
val find_batch_vertex : t -> Tethers_core.batch_id -> int option
val find_template_vertex : t -> Tethers_core.item_template_id -> int option
val find_scoped_role_vertex :
  t -> Tethers_core.role_scope -> Tethers_core.role_id -> int option

val forward_edges : t -> int -> edge list
val reverse_edges : t -> int -> edge list
val all_forward_edges : t -> edge list
val relation_kinds_present : t -> relation_kind list
val relation_name : relation_kind -> string

val structural_evidence : t -> string
(** Deterministically sorted structural evidence.  It intentionally contains
    no raw Core IDs or internal vertex numbers. *)

val required_relation_kinds : relation_kind list
(** The complete R3-1 relation taxonomy, covering R01-R29. *)

val enc_v2_lookup_coverage : (string * relation_kind list) list
(** Machine-checkable mapping from each anonymous Enc_V2 label family to the
    V3 relation kinds that expose its occurrences. *)
