(* ==================================================================
   CANONICAL FORMAT V2 — PRODUCTION CANONICALISER (interface)

   This module implements the first non-oracle production
   implementation of Canonical Format V2.  It searches the complete
   Λ(P) using streaming permutation traversal (not materialising
   factorial candidate lists) and enforces a deterministic work
   budget.

   The reference oracle (Tethers_core_canonical_v2_reference) and
   this module share ONLY the frozen format layer.  Search
   implementations remain independent.
   ================================================================== *)

(** Opaque canonicalised result *)
type canonicalized_v2

type canonicalization_error =
  | Invalid_core of Tethers_core_validator.validation_error list
  | Canonicalisation_too_complex

type search_budget = {
  max_candidates : int;
}

(** Canonicalise a validated program.
    Returns the canonical payload, preimage, and digest.
    Default budget: 5_000_000 candidates. *)
val canonicalize :
  ?budget:search_budget ->
  Tethers_core.program ->
  (canonicalized_v2, canonicalization_error) result

(** Extract the canonical payload (§16) *)
val canonical_payload : canonicalized_v2 -> string

(** Extract the canonical preimage: DOMAIN_V2 || payload (§17) *)
val canonical_preimage : canonicalized_v2 -> bytes

(** Extract the external digest string: "tethers:v2:sha256:<hex>" *)
val program_digest : canonicalized_v2 -> string
