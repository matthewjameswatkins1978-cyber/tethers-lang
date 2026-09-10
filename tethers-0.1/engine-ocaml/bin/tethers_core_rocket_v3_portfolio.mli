(** Exact Rocket V3 solver portfolio over the frozen Enc_V2 format. *)

type backend =
  | B2_success_path
  | R3_2_refined_exact_leaf
  | Frozen_v2_exact_search
  | Exhaustive_reference

type stats = {
  backend : backend;
  refinement_cells : int option;
  refinement_discrete : bool option;
  path_attempted : bool;
  reference_candidates : int option;
}

type result = {
  payload : string;
  preimage : bytes;
  digest : string;
  stats : stats;
}

type error =
  | Invalid_core of Tethers_core_validator.validation_error list
  | Canonicalisation_too_complex

val backend_name : backend -> string

val canonicalise :
  ?max_candidates:int ->
  ?reference:bool ->
  Tethers_core.program -> (result, error) Stdlib.result
