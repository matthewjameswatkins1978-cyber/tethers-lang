(** Experimental Origin-only canonical augmentation for the R3-3B crucible. *)

type branch_order =
  | Numeric_ascending
  | Numeric_descending
  | Semantic_first

type decision =
  | NeedLabel of Tethers_core.origin_id
  | NeedOwnerOfNumericSlot of int

type stats = {
  emitted_bytes : int;
  forced_assignments : int;
  decision_points : int;
  branches_explored : int;
  prefix_prunes : int;
  completed_candidates : int;
  max_depth : int;
}

type result = {
  payload : string;
  stats : stats;
}

type error =
  | Invalid_core of Tethers_core_validator.validation_error list
  | Empty_origin_domain
  | Unsupported_origin_projection of string
  | No_legal_origin_assignment

val initial_decision : Tethers_core.program -> (decision, error) Stdlib.result

val walk :
  ?branch_order:branch_order ->
  Tethers_core.program ->
  (result, error) Stdlib.result
