module Core = Tethers_core
module Validator = Tethers_core_validator
module Format = Tethers_core_canonical_v2_format
module V2 = Tethers_core_canonical_v2
module Reference = Tethers_core_canonical_v2_reference
module Model = Tethers_core_rocket_v3_model
module Refine = Tethers_core_rocket_v3_refine
module Partition = Tethers_core_rocket_v3_partition
module Encode = Tethers_core_rocket_v3_encode
module Path = Tethers_core_rocket_v3_success_path

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
  | Invalid_core of Validator.validation_error list
  | Canonicalisation_too_complex

let backend_name = function
  | B2_success_path -> "b2_success_path"
  | R3_2_refined_exact_leaf -> "r3_2_refined_exact_leaf"
  | Frozen_v2_exact_search -> "frozen_v2_exact_search"
  | Exhaustive_reference -> "exhaustive_reference"

let result_of_payload backend ?refinement_cells ?refinement_discrete
    ?(path_attempted = false) ?reference_candidates payload =
  let preimage = Bytes.cat Format.domain_v2 (Bytes.of_string payload) in
  {
    payload;
    preimage;
    digest = Format.digest_string_v2 (Format.sha256_hex preimage);
    stats = {
      backend;
      refinement_cells;
      refinement_discrete;
      path_attempted;
      reference_candidates;
    };
  }

let error_of_reference = function
  | Reference.Invalid_core errors -> Invalid_core errors
  | Reference.Oracle_too_large -> Canonicalisation_too_complex

let error_of_v2 = function
  | V2.Invalid_core errors -> Invalid_core errors
  | V2.Canonicalisation_too_complex -> Canonicalisation_too_complex

let exact_reference program ~path_attempted =
  match Reference.slow_oracle program with
  | Ok reference ->
      Ok (result_of_payload Exhaustive_reference ~path_attempted
            ~reference_candidates:reference.candidate_count reference.payload)
  | Error error -> Error (error_of_reference error)

let frozen_v2_search program ~max_candidates ~path_attempted
    ?refinement_cells ?refinement_discrete () =
  let budget = { V2.max_candidates } in
  match V2.canonicalize ~budget program with
  | Ok canonicalized ->
      Ok (result_of_payload Frozen_v2_exact_search ~path_attempted
            ?refinement_cells ?refinement_discrete
            (V2.canonical_payload canonicalized))
  | Error V2.Canonicalisation_too_complex ->
      (* The optimisation budget is a runtime valve, never an identity
         decision.  Give the permanent reference engine the last exact say
         whenever the bounded production search declines the case. *)
      exact_reference program ~path_attempted
  | Error error -> Error (error_of_v2 error)

let refined_exact_or_fallback program ~max_candidates ~path_attempted model
    refined =
  let refinement_cells = Some (Partition.cell_count refined.Refine.partition) in
  let refinement_discrete = Some (Partition.is_discrete refined.partition) in
  if max_candidates > 0 && Partition.is_discrete refined.partition &&
     Model.vertex_count model <= 12
  then
    match Encode.encode program model refined.partition with
    | Ok leaf ->
        Ok (result_of_payload R3_2_refined_exact_leaf ~path_attempted
              ?refinement_cells ?refinement_discrete leaf.payload)
    | Error _ ->
        frozen_v2_search program ~max_candidates ~path_attempted
          ?refinement_cells ?refinement_discrete ()
  else
    frozen_v2_search program ~max_candidates ~path_attempted
      ?refinement_cells ?refinement_discrete ()

let canonicalise ?(max_candidates = 5_000_000) ?(reference = false) program =
  match Validator.validate program with
  | Error errors -> Error (Invalid_core errors)
  | Ok () when reference -> exact_reference program ~path_attempted:false
  | Ok () ->
      begin match Path.canonicalise program with
      | Ok path ->
          Ok (result_of_payload B2_success_path ~path_attempted:true
                path.payload)
      | Error _ ->
          match Model.build program with
          | Error errors -> Error (Invalid_core errors)
          | Ok model ->
              let refined = Refine.run model in
              refined_exact_or_fallback program ~max_candidates
                ~path_attempted:true model refined
      end
