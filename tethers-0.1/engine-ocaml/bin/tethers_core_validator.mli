(** Static Core Program validator.

    CORE-3 validates that a [Tethers_core.program] is internally well-formed
    according to the current Core semantics.  It MUST validate.  It MUST NOT
    repair, infer, reorder, canonicalise, or execute. *)

type validation_error =
  (* Identity uniqueness *)
  | Duplicate_origin_id of Tethers_core.origin_id
  | Duplicate_fact_id of Tethers_core.fact_id
  | Duplicate_role_id of Tethers_core.role_id
  | Duplicate_capability_id of Tethers_core.capability_id
  | Duplicate_branch_id of Tethers_core.branch_id
  | Duplicate_group_id of Tethers_core.group_id
  | Duplicate_batch_id of Tethers_core.batch_id
  | Duplicate_item_template_id of Tethers_core.item_template_id
  (* Reference integrity *)
  | Missing_origin of Tethers_core.origin_id
  | Missing_fact of Tethers_core.fact_id
  | Missing_role of Tethers_core.role_id
  | Missing_capability_contract of Tethers_core.capability_id
  | Missing_branch_target of Tethers_core.origin_id
  | Missing_item_template of Tethers_core.item_template_id
  (* Entry integrity *)
  | Missing_entry_origin_for_actions
  | Unknown_entry_origin of Tethers_core.origin_id
  (* Success continuation *)
  | Duplicate_success_continuation of Tethers_core.origin_id
  | Success_cycle of Tethers_core.origin_id list
  (* Capability contract *)
  | Capability_contract_digest_mismatch of Tethers_core.capability_id
  | Duplicate_capability_contract of Tethers_core.capability_id
  (* Input Fact / Guard *)
  | Input_fact_not_declared of Tethers_core.fact_id
  | Input_fact_wrong_provenance of Tethers_core.fact_id
  (* Fact provenance *)
  | Fact_origin_provenance_missing_origin of Tethers_core.fact_id
  | Fact_role_provenance_missing_role of Tethers_core.fact_id
  | Fact_from_origin_provenance_mismatch of Tethers_core.fact_id * Tethers_core.origin_id
  | Fact_role_contract_not_exposed of Tethers_core.fact_id * Tethers_core.role_id
  | Fact_dependency_cycle of Tethers_core.fact_id list
  (* Anchor binding *)
  | Anchor_origin_not_anchor of Tethers_core.origin_id
  | Anchor_path_empty
  | Anchor_path_empty_component of Tethers_core.origin_id * string list
  (* Branch *)
  | Branch_duplicate_outcome of Tethers_core.branch_id
  | Branch_subject_missing of Tethers_core.branch_id
  (* Together *)
  | Together_single_member of Tethers_core.group_id
  | Together_self_member of Tethers_core.group_id
  | Together_duplicate_member of Tethers_core.group_id
  | Together_unknown_member of Tethers_core.group_id * Tethers_core.origin_id
  (* Role *)
  | Role_fact_contract_invalid_fact of Tethers_core.role_id * Tethers_core.fact_id
  | Role_scope_missing_item_template of Tethers_core.role_id
  (* Item template *)
  | Item_objective_missing_role of Tethers_core.item_template_id * Tethers_core.role_id
  | Item_template_duplicate_origin_id of Tethers_core.item_template_id * Tethers_core.origin_id
  (* Batch *)
  | Batch_missing_item_template of Tethers_core.batch_id
  (* V2 additions *)
  | Role_scope_storage_mismatch of Tethers_core.role_id
  | Role_scope_template_mismatch of Tethers_core.role_id * Tethers_core.item_template_id * Tethers_core.item_template_id
  | Role_fact_contract_duplicate_fact of Tethers_core.role_id * Tethers_core.fact_id
  | Role_proxy_scope_mismatch of Tethers_core.fact_id * Tethers_core.role_id
  (* Deadline *)
  | Deadline_empty of Tethers_core.origin_id

val validate :
  Tethers_core.program ->
  (unit, validation_error list) result
(** Validate that [program] is internally well-formed according to current
    Core semantics.  Returns [Ok ()] for valid programs, or [Error errors]
    with deterministic ordered error list.  Never repairs or executes. *)
