(** Human Tether AST → Tethers Core program lowering.

    CORE-2 translates the supported sequential Tethers 0.1 subset into the
    dormant Core representation.  It is a deterministic sidecar path that does
    not replace the existing evaluator.

    ID assignment is deterministic and source-position-derived.  These are
    pre-canonical static identities; CORE-4 will later canonicalise semantic
    identity.  Do not treat the current ID shape as stable for hashing. *)

type capability_binding = {
  source_name : string;
  capability_id : Tethers_core.capability_id;
  contract_digest : Tethers_core.capability_contract_digest;
}

type input_fact_binding = {
  source_name : string;
  fact : Tethers_core.fact;
}

type lowering_environment = {
  program_id : Tethers_core.program_id;
  core_version : Tethers_core.core_version;
  capabilities : capability_binding list;
  input_facts : input_fact_binding list;
}

type lowering_error =
  | Unsupported_construct of string
  | Unknown_capability of string
  | Duplicate_capability of string
  | Unknown_fact of string
  | Missing_anchor_reference of string

val lower :
  lowering_environment ->
  Tether_parser.tether ->
  (Tethers_core.program, lowering_error) result
(** Translate a parsed sequential Tether into a Core program using the
    supplied explicit environment.  Never invents capability identities,
    contract digests, or input Fact declarations. *)
