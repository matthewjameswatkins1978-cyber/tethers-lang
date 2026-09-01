(** Human Request → Rocket V2 Evaluation Adapter.

    CORE-8A makes the assembly of Core plumbing one explicit typed boundary.
    A caller supplies Human-world source text, an evaluation input, and an
    adapter environment carrying explicit semantic lowering identities. The
    adapter performs the full pipeline:

    {v
    Human source → parse → lower → Rocket V2 → evaluate
    v}

    The adapter is a pure assembly seam. It does not execute Actions, grant
    permission, or invent capability identity from runtime names. *)

type capability_binding = {
  source_name : string;
  capability_id : Tethers_core.capability_id;
  contract_digest : Tethers_core.capability_contract_digest;
  runtime : Tethers_protocol.capability;
}
(** An explicit binding from a Human-authored capability source name to the
    Core semantic identity, contract digest, and runtime capability projection.

    The same binding produces both the lowerer capability binding (for name
    resolution) and the plan runtime_capability_projection (for planning).
    These share capability_id and contract_digest by construction. *)

type input_fact_binding = {
  source_name : string;
  fact : Tethers_core.fact;
}
(** An explicit binding from a Human-authored fact source name to the Core
    fact declaration.  The fact's provenance carries the HostSnapshotKey
    and scalar type needed for runtime Fact snapshot mapping. *)

type environment = {
  program_id : Tethers_core.program_id;
  core_version : Tethers_core.core_version;
  capabilities : capability_binding list;
  input_facts : input_fact_binding list;
}
(** The adapter environment.  Caller supplies all semantic lowering identities.
    The adapter does not invent capability_id, contract_digest, or fact
    declarations. *)

type evaluation_input = {
  evaluation_id : string;
  source : string;
  event_name : string;
  event_data : Yojson.Safe.t;
  facts : (string * Yojson.Safe.t) list;
}
(** Human-world occurrence input.  The source is a Tether text.
    No canonical OriginId, FactId, or ProgramDigest is supplied. *)

type adapter_error =
  | Parse_error of string * string
  | Lowering_error of Tethers_core_lowerer.lowering_error
  | Canonicalization_error of Tethers_core_canonical_v2_ir.canonicalization_error_ir
  | Planning_error of Tethers_core_plan.planning_error
  | Unknown_runtime_fact_name of string
  | Ambiguous_runtime_fact_name of string
  | Duplicate_runtime_fact_name of string
(** Typed adapter error preserving layer ownership. *)

val evaluate :
  environment ->
  evaluation_input ->
  (Tethers_core_plan.canonical_evaluation, adapter_error) result
(** One-call adapter: parse → lower → Rocket V2 → map facts → build
    context → evaluate the Rocket V2 token.

    The caller supplies only the environment and evaluation input.  The
    adapter performs all internal steps and returns the deterministic evaluation
    result.

    ProgramDigest is produced only by Rocket V2 canonicalisation.  evaluation_id passes
    through unchanged into the plan (plan.id = evaluation_id ^ "/plan"). *)
