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
