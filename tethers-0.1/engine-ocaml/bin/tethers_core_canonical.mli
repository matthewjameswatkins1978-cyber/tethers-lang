(** Canonical semantic identity layer for Tethers Core.

    CORE-4 implements deterministic canonicalisation: structural fingerprinting,
    canonical ordering, internal ID assignment, reference rewriting, canonical
    byte encoding, SHA-256, and ProgramDigest.

    Same semantic meaning → same canonical bytes → same ProgramDigest.
    Different semantic meaning → different canonical bytes. *)

type program_digest
type canonicalized

type canonicalization_error =
  | Invalid_core of Tethers_core_validator.validation_error list
  | Refinement_exceeded

val canonicalize :
  Tethers_core.program ->
  (canonicalized, canonicalization_error) result
(** Validate and canonicalise.  Returns [Error (Invalid_core errors)] for
    invalid Core.  Never repairs, never executes. *)

val canonical_program :
  canonicalized ->
  Tethers_core.program
(** The canonicalised program with rewritten internal IDs and sorted
    unordered collections.  Retains the original [program_id] but it is
    not hashed. *)

val canonical_bytes :
  canonicalized ->
  string
(** The deterministic canonical byte encoding (including the version prefix). *)

val program_digest :
  canonicalized ->
  program_digest

val string_of_program_digest :
  program_digest ->
  string
(** ["sha256:<64 lowercase hexadecimal characters>"]. *)

(** {1 Internal pipeline (exposed for C-B1 benchmarking)} *)

module StringMap : Map.S with type key = string

type colour_map = {
  fact_colours : int StringMap.t;
  origin_colours : int StringMap.t;
  batch_colours : int StringMap.t;
  role_colours : int StringMap.t;
  branch_colours : int StringMap.t;
  item_template_colours : int StringMap.t;
}

type canonical_ids

val assign_canonical_ids : colour_map -> Tethers_core.program -> canonical_ids
val build_canonical_program : Tethers_core.program -> canonical_ids -> Tethers_core.program
val make_canonical_bytes : Tethers_core.program -> string
val compute_sha256 : string -> string
val make_program_digest : string -> program_digest
