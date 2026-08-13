(* ==================================================================
   CANONICAL FORMAT V2 — OPTIMISED IR SEARCH (interface)

   Separate optimised engine that returns EXACTLY the same
   CanonicalPayload_V2 and ProgramDigest_V2 as both the slow oracle
   and the exhaustive production baseline over every program where
   both succeed.

   Identity law:
     CanonicalPayload_V2(P) = min { Enc_V2(P, λ) | λ ∈ Λ(P) }
     CanonicalPreimage_V2(P) = DOMAIN_V2 || CanonicalPayload_V2(P)
     ProgramDigest_V2(P) = SHA-256(CanonicalPreimage_V2(P))

   This engine uses individualisation/refinement as SEARCH machinery
   only.  The only winning certificate is Enc_V2(P, λ) under
   compare_bytes_lex_unsigned.  Colours never enter Enc_V2.

   Three engines remain available:
     A. slow oracle (Tethers_core_canonical_v2_reference)
     B. production exhaustive baseline (Tethers_core_canonical_v2)
     C. this optimised IR search

   All three share ONLY Tethers_core_canonical_v2_format for frozen
   format/encoding.  No engine calls another for the answer.

   Fail-closed: on deterministic budget exhaustion returns
   Canonicalisation_too_complex with no payload, no digest, no best
   fallback.  No silent fallback to exhaustive baseline.

   No physical parallelism (Domains/threads) in this packet.
   ================================================================== *)

type canonicalized_v2_ir

type canonicalization_error_ir =
  | Invalid_core of Tethers_core_validator.validation_error list
  | Canonicalisation_too_complex

type search_budget_ir = {
  max_nodes : int;
  max_leaves : int;
  max_refinement_rounds : int;
}

val default_budget_ir : search_budget_ir
(** Default deterministic budget:
    max_nodes = 1_000_000
    max_leaves = 5_000_000
    max_refinement_rounds = 1000
    These numbers are NOT format identity. *)

type ir_stats = {
  nodes : int;
  leaves : int;
  refinement_rounds : int;
  pruned_prefix : int;
  pruned_memo : int;
}

val canonicalize_ir :
  ?budget:search_budget_ir ->
  Tethers_core.program ->
  (canonicalized_v2_ir * ir_stats, canonicalization_error_ir) result

val canonical_payload_ir : canonicalized_v2_ir -> string
val canonical_preimage_ir : canonicalized_v2_ir -> bytes
val program_digest_ir : canonicalized_v2_ir -> string

(* Testing / internal — candidate count helper delegates to same
   overflow-safe arithmetic as baseline but with IR budget limits. *)
val candidate_count_within_budget_ir : limit:int -> Tethers_core.program -> int option
