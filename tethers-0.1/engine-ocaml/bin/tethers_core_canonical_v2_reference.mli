(* ==================================================================
   CANONICAL FORMAT V2 — REFERENCE ORACLE (interface)

   This module implements a deliberately slow complete Λ(P) oracle
   for testing.  The frozen Enc_V2 byte encoder lives in the shared
   format module (Tethers_core_canonical_v2_format).
   ================================================================== *)

open Tethers_core

type oracle_result = {
  payload : string;
  preimage : bytes;
  raw_digest : string;
  digest_string : string;
  candidate_count : int;
}

type oracle_error =
  | Invalid_core of Tethers_core_validator.validation_error list
  | Oracle_too_large

(** Run the slow complete Λ(P) oracle. *)
val slow_oracle : program -> (oracle_result, oracle_error) result

(** Convenience: compute just the digest string *)
val compute_digest : program -> (string * string, oracle_error) result

(** Generate all permutations of a list (test-only helper) *)
val perm : 'a list -> 'a list list
