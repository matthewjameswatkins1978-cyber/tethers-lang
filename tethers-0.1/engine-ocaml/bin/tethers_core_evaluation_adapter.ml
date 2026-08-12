open Tethers_core

(* ================================================================== *)
(*  Types                                                              *)
(* ================================================================== *)

type capability_binding = {
  source_name : string;
  capability_id : capability_id;
  contract_digest : capability_contract_digest;
  runtime : Tethers_protocol.capability;
}

type input_fact_binding = {
  source_name : string;
  fact : fact;
}

type environment = {
  program_id : program_id;
  core_version : core_version;
  capabilities : capability_binding list;
  input_facts : input_fact_binding list;
}

type evaluation_input = {
  evaluation_id : string;
  source : string;
  event_name : string;
  event_data : Yojson.Safe.t;
  facts : (string * Yojson.Safe.t) list;
}

type adapter_error =
  | Parse_error of string * string
  | Lowering_error of Tethers_core_lowerer.lowering_error
  | Canonicalization_error of Tethers_core_canonical.canonicalization_error
  | Planning_error of Tethers_core_plan.planning_error
  | Unknown_runtime_fact_name of string
  | Ambiguous_runtime_fact_name of string
  | Duplicate_runtime_fact_name of string

(* ================================================================== *)
(*  Helpers                                                            *)
(* ================================================================== *)

let ( >>= ) = Result.bind

(* Detect duplicate names in runtime fact input. *)
let check_duplicate_runtime_facts facts =
  let rec check seen = function
    | [] -> Ok ()
    | (name, _) :: rest ->
        if List.mem name seen then Error (Duplicate_runtime_fact_name name)
        else check (name :: seen) rest
  in
  check [] facts

(* Extract HostSnapshotKey from a fact's Evaluation_input provenance. *)
let host_snapshot_key_of_fact f =
  match f.provenance with
  | Evaluation_input (hsk, _) -> Some hsk
  | Origin_provenance _ | Role_proxy _ -> None

(* Map one runtime fact through environment.input_facts by source_name. *)
let map_one_fact env_fact_bindings (name, value) =
  let matching =
    List.filter (fun b -> b.source_name = name) env_fact_bindings
  in
  match matching with
  | [] -> Error (Unknown_runtime_fact_name name)
  | [ b ] ->
      (match host_snapshot_key_of_fact b.fact with
       | Some key ->
           Ok { Tethers_core_plan.key; value }
       | None ->
           (* Fact provenance is not Evaluation_input; cannot map at runtime. *)
           Error (Unknown_runtime_fact_name name))
  | _ -> Error (Ambiguous_runtime_fact_name name)

(* Map all runtime facts through environment.input_facts. *)
let map_facts env_fact_bindings runtime_facts =
  check_duplicate_runtime_facts runtime_facts >>= fun () ->
  List.fold_left
    (fun acc item ->
       match acc with
       | Error _ -> acc
       | Ok mapped ->
           match map_one_fact env_fact_bindings item with
           | Ok snap -> Ok (snap :: mapped)
           | Error e -> Error e)
    (Ok []) runtime_facts
  |> Result.map List.rev

(* Build lowerer capability bindings from adapter environment. *)
let lowerer_capabilities (env_caps : capability_binding list) : Tethers_core_lowerer.capability_binding list =
  List.map
    (fun (b : capability_binding) ->
       { Tethers_core_lowerer.source_name = b.source_name;
         capability_id = b.capability_id;
         contract_digest = b.contract_digest })
    env_caps

(* Build lowerer fact bindings from adapter environment. *)
let lowerer_facts (env_facts : input_fact_binding list) : Tethers_core_lowerer.input_fact_binding list =
  List.map
    (fun (b : input_fact_binding) ->
       { Tethers_core_lowerer.source_name = b.source_name;
         fact = b.fact })
    env_facts

(* Build plan runtime_capability_projections from adapter environment. *)
let plan_projections (env_caps : capability_binding list) : Tethers_core_plan.runtime_capability_projection list =
  List.map
    (fun (b : capability_binding) ->
       { Tethers_core_plan.capability_id = b.capability_id;
         contract_digest = b.contract_digest;
         runtime = b.runtime })
    env_caps

(* Reject environment-wide capability contract conflicts before lowering.
   For each CapabilityId, the first digest establishes its contract.
   A later binding with the same CapabilityId but a different digest
   is a conflict regardless of whether the Human source uses it. *)
let check_conflicting_capability_contracts (caps : capability_binding list) =
  let rec check acc = function
    | [] -> Ok ()
    | b :: rest ->
        let key = Tethers_core.string_of_capability_id b.capability_id in
        match List.assoc_opt key acc with
        | None ->
            check ((key, b.contract_digest) :: acc) rest
        | Some digest ->
            if b.contract_digest = digest then
              check acc rest
            else
              Error (Lowering_error
                       (Tethers_core_lowerer.Conflicting_capability_contract
                          b.capability_id))
  in
  check [] caps

(* ================================================================== *)
(*  Main evaluate                                                     *)
(* ================================================================== *)

let evaluate env input =
  let open Result in
  (* 1. Parse Human source. *)
  let parsed =
    try Ok (Tether_parser.parse_tether input.source)
    with Tethers_error.Tethers_error (code, msg) -> Error (Parse_error (code, msg))
  in
  parsed >>= fun tether ->
  (* 1b. Validate environment-wide capability contract consistency. *)
  check_conflicting_capability_contracts env.capabilities >>= fun () ->
  (* 2. Build lowerer environment and lower. *)
  let lowerer_env =
    { Tethers_core_lowerer.program_id = env.program_id;
      core_version = env.core_version;
      capabilities = lowerer_capabilities env.capabilities;
      input_facts = lowerer_facts env.input_facts }
  in
  Tethers_core_lowerer.lower lowerer_env tether
  |> Result.map_error (fun e -> Lowering_error e)
  >>= fun core_program ->
  (* 3. Canonicalize. *)
  Tethers_core_canonical.canonicalize core_program
  |> Result.map_error (fun e -> Canonicalization_error e)
  >>= fun canonicalized ->
  (* 4. Map runtime facts. *)
  map_facts env.input_facts input.facts
  >>= fun fact_snapshots ->
  (* 5. Build evaluation context. *)
  let event =
    { Tethers_core_plan.name = input.event_name;
      data = input.event_data }
  in
  let eval_ctx =
    { Tethers_core_plan.evaluation_id = input.evaluation_id;
      event;
      capabilities = plan_projections env.capabilities;
      facts = fact_snapshots }
  in
  (* 6. Evaluate canonicalized. *)
  Tethers_core_plan.evaluate_canonicalized canonicalized eval_ctx
  |> Result.map_error (fun e -> Planning_error e)
