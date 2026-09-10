module Core = Tethers_core

type target =
  | Origin_target of int
  | Program_complete

type partial_successor = {
  source : int;
  target : target;
}

type choice_order =
  | Encoded_ascending
  | Numeric_ascending
  | Numeric_descending

type stats = {
  path_size : int;
  successor_slots_processed : int;
  candidate_targets_considered : int;
  feasibility_checks : int;
  rejected_infeasible_choices : int;
  committed_choices : int;
  complete_permutations_enumerated : int;
  max_partial_components : int;
}

type result = {
  payload : string;
  labels : (Core.origin_id * int) list;
  stats : stats;
}

type error =
  | Invalid_core of Tethers_core_validator.validation_error list
  | Unsupported_success_path of string
  | No_legal_success_path

val feasible_partial :
  path_size:int ->
  entry_label:int ->
  processed_slots:int ->
  partial_successor list -> bool

val canonicalise :
  ?choice_order:choice_order ->
  Core.program -> (result, error) Stdlib.result
