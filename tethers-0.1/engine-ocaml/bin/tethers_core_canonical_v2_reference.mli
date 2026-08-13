(* ==================================================================
   CANONICAL FORMAT V2 — REFERENCE ENCODER AND SLOW ORACLE (interface)

   This module implements the frozen Enc_V2 byte encoder and a
   deliberately slow complete Λ(P) oracle for testing.
   ================================================================== *)

open Tethers_core

(** Domain separation bytes: "TETHERS_CORE_CANON_V2" || 0x00 *)
val domain_v2 : bytes

(** SHA-256 hex digest *)
val sha256_hex : bytes -> string

(** Construct the external digest string *)
val digest_string_v2 : string -> string

(** Label map from entity raw key to canonical label *)
module StringMap : Map.S with type key = string

type label_map = int StringMap.t

(** Encode a validated program with the given label map.
    This is the frozen Enc_V2 encoder (§6). *)
val encode_program : label_map -> program -> string

(** Primitive encoders (§6.2) *)
val encode_string : string -> string
val encode_int : int -> string
val encode_tag : int -> string

(** Oracle result *)
type oracle_result = {
  payload : string;
  preimage : bytes;
  raw_digest : string;
  digest_string : string;
  candidate_count : int;
}

(** Oracle error *)
type oracle_error =
  | Invalid_core of Tethers_core_validator.validation_error list
  | Oracle_too_large

(** Run the slow complete Λ(P) oracle.
    Returns the canonical payload, preimage, digest, and candidate count. *)
val slow_oracle : program -> (oracle_result, oracle_error) result

(** Convenience: compute just the digest string *)
val compute_digest : program -> (string * string, oracle_error) result
