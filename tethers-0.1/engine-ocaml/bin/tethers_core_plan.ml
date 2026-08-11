open Tethers_core
open Tethers_core_validator
open Tethers_outcome

type planning_error =
  | Invalid_core of validation_error list
  | Missing_entry_origin
  | Incomplete_success_path of origin_id
  | Unsupported_together
  | Unsupported_batch
  | Unsupported_branch
  | Unsupported_role_binding
  | Unsupported_role_proxy
  | Unsupported_fact_binding
  | Unsupported_anchor_value
  | Unsupported_execution_constraint
  | Unsupported_item_template
  | Missing_capability_projection of capability_id
  | Capability_projection_identity_mismatch of capability_id
  | Capability_projection_digest_mismatch of capability_id
  | Capability_projection_incomplete of capability_id
  | Ambiguous_capability_projection of capability_id
  | Flow_cycle of origin_id list
  | Unresolved_origin of origin_id

type runtime_capability_projection = {
  capability_id : capability_id;
  contract_digest : capability_contract_digest;
  runtime : Tethers_protocol.capability;
}

type planning_context = {
  evaluation_id : string;
  capabilities : runtime_capability_projection list;
}

(* ------------------------------------------------------------------ *)
(*  Core value encoding                                                *)
(* ------------------------------------------------------------------ *)

let json_of_core_value = function
  | String_value s -> `String s
  | Integer_value i -> `Int i
  | Boolean_value b -> `Bool b

(* ------------------------------------------------------------------ *)
(*  Origin site helpers                                                *)
(* ------------------------------------------------------------------ *)

let origin_id_of_site = function
  | Anchor_origin a -> Some a.anchor_origin_id
  | Action_origin a -> Some a.action_origin_id
  | Together_origin t -> Some t.together_origin_id
  | Batch_site _ -> None

let declared_facts_of_site = function
  | Anchor_origin a -> a.declared_facts
  | Action_origin a -> a.declared_facts
  | Together_origin _ -> []
  | Batch_site b -> b.aggregate_facts

(* ------------------------------------------------------------------ *)
(*  Unsupported-construct pre-scan                                     *)
(*                                                                     *)
(*  Fail closed on the presence of any construct the sequential         *)
(*  runtime-plan vocabulary cannot represent.  Deterministic precedence: *)
(*  Together, then Batch (site or item-context input), then Branch,     *)
(*  then item templates, then Role_proxy facts, then input bindings,    *)
(*  then execution constraints.  Never partially plan.                  *)
(* ------------------------------------------------------------------ *)

let binding_error = function
  | Literal_value _ -> None
  | Fact_through_role _ -> Some Unsupported_role_binding
  | Fact_from_origin _ -> Some Unsupported_fact_binding
  | Anchor_value _ -> Some Unsupported_anchor_value
  | Batch_item_context _ -> Some Unsupported_batch

let action_inputs_of_site = function
  | Action_origin a -> a.inputs
  | _ -> []

let unsupported program =
  let sites = program.origin_sites in
  let has_together =
    List.exists (function Together_origin _ -> true | _ -> false) sites
  in
  if has_together then Some Unsupported_together
  else
    let has_batch_site =
      List.exists (function Batch_site _ -> true | _ -> false) sites
    in
    let has_batch_item =
      List.exists
        (fun (ai : action_input) ->
          match ai.binding with
          | Batch_item_context _ -> true
          | _ -> false)
        (List.concat_map action_inputs_of_site sites)
    in
    if has_batch_site || has_batch_item then Some Unsupported_batch
    else if program.branches <> [] then Some Unsupported_branch
    else if program.item_templates <> [] then Some Unsupported_item_template
    else
      let has_role_proxy =
        List.exists
          (fun (f : fact) ->
            match f.provenance with Role_proxy _ -> true | _ -> false)
          (List.concat_map declared_facts_of_site sites)
      in
      if has_role_proxy then Some Unsupported_role_proxy
      else
        let binding_err =
          List.find_map
            (fun (ai : action_input) -> binding_error ai.binding)
            (List.concat_map action_inputs_of_site sites)
        in
        match binding_err with
        | Some _ as err -> err
        | None ->
            let has_constraint =
              List.exists
                (fun (a : action_origin) -> a.execution_constraints <> [])
                (List.filter_map
                   (function Action_origin a -> Some a | _ -> None)
                   sites)
            in
            if has_constraint then Some Unsupported_execution_constraint
            else None

(* ------------------------------------------------------------------ *)
(*  Runtime capability projection resolution                           *)
(*                                                                     *)
(*  The bridge never trusts the full manifest.  It resolves the         *)
(*  approved projection keyed by the Core capability identity and       *)
(*  contract digest, and fails closed on every mismatch.  It never      *)
(*  substitutes another capability version or contract.                 *)
(* ------------------------------------------------------------------ *)

let projection_metadata_complete (p : runtime_capability_projection) =
  p.runtime.name <> "" && p.runtime.version <> ""
  &&
  match
    ( p.runtime.manifest_digest,
      p.runtime.bridge_capability_version,
      p.runtime.bridge_provider_identity )
  with
  | None, None, None | Some _, Some _, Some _ -> true
  | _ -> false

let projection_of context capability_id contract_digest =
  let by_id =
    List.filter
      (fun (p : runtime_capability_projection) -> p.capability_id = capability_id)
      context.capabilities
  in
  match by_id with
  | [] ->
      let by_digest =
        List.filter
          (fun (p : runtime_capability_projection) ->
            p.contract_digest = contract_digest)
          context.capabilities
      in
      if by_digest = [] then Error (Missing_capability_projection capability_id)
      else Error (Capability_projection_identity_mismatch capability_id)
  | _ -> (
      let exact =
        List.filter
          (fun (p : runtime_capability_projection) ->
            p.contract_digest = contract_digest)
          by_id
      in
      match exact with
      | [] -> Error (Capability_projection_digest_mismatch capability_id)
      | [ projection ] ->
          if projection_metadata_complete projection then Ok projection
          else Error (Capability_projection_incomplete capability_id)
      | _ -> Error (Ambiguous_capability_projection capability_id))

(* ------------------------------------------------------------------ *)
(*  Action planning                                                    *)
(*                                                                     *)
(*  Only literal inputs reach this point: the pre-scan rejects every    *)
(*  binding the sequential plan cannot carry.  Each planned Action      *)
(*  carries the existing Runtime Plan Action contract: [action_id],     *)
(*  [idempotency_key] derived from the occurrence context,              *)
(*  [capability] and [capability_version] from the approved projection, *)
(*  [arguments] as concrete values, [effects], and the projection's     *)
(*  bridge metadata fields when present.  The result keeps the          *)
(*  function total and honest even though the non-literal branches are  *)
(*  unreachable for a program that passed the pre-scan.                 *)
(* ------------------------------------------------------------------ *)

let plan_action context index (a : action_origin) =
  let rec build_arguments acc = function
    | [] -> Ok (List.rev acc)
    | (ai : action_input) :: rest -> (
        match ai.binding with
        | Literal_value value ->
            let name = string_of_capability_input_name ai.input_name in
            build_arguments ((name, json_of_core_value value) :: acc) rest
        | Fact_through_role _ -> Error Unsupported_role_binding
        | Fact_from_origin _ -> Error Unsupported_fact_binding
        | Anchor_value _ -> Error Unsupported_anchor_value
        | Batch_item_context _ -> Error Unsupported_batch)
  in
  match build_arguments [] a.inputs with
  | Error _ as err -> err
  | Ok arguments -> (
      match projection_of context a.capability_id a.contract_digest with
      | Error _ as err -> err
      | Ok projection ->
          let action_id = "action_" ^ string_of_int index in
          let idempotency_key = context.evaluation_id ^ "/" ^ action_id in
          let base_fields =
            [
              ("action_id", `String action_id);
              ("idempotency_key", `String idempotency_key);
              ("capability", `String projection.runtime.name);
              ("capability_version", `String projection.runtime.version);
              ("arguments", `Assoc arguments);
              ( "effects",
                `List
                  (List.map (fun item -> `String item) projection.runtime.effects) );
            ]
          in
          let bridge_fields =
            (match projection.runtime.manifest_digest with
            | Some digest -> [ ("manifest_digest", `String digest) ]
            | None -> [])
            @ (match projection.runtime.bridge_capability_version with
              | Some version -> [ ("bridge_capability_version", `Int version) ]
              | None -> [])
            @ (match projection.runtime.bridge_provider_identity with
              | Some provider -> [ ("bridge_provider_identity", `String provider) ]
              | None -> [])
          in
          Ok (`Assoc (base_fields @ bridge_fields), projection.runtime.effects))

(* ------------------------------------------------------------------ *)
(*  Deterministic required-effects uniqueness                          *)
(*                                                                     *)
(*  Matches the existing evaluator contract: unique values preserving  *)
(*  first-occurrence order across the planned Actions.                 *)
(* ------------------------------------------------------------------ *)

let unique_effects values =
  List.fold_left
    (fun acc value -> if List.mem value acc then acc else acc @ [ value ])
    [] values

(* ------------------------------------------------------------------ *)
(*  Control-flow walk                                                  *)
(*                                                                     *)
(*  Sequential execution follows [entry_origin] then success            *)
(*  continuations.  Every reachable path must reach [Program_complete]  *)
(*  explicitly; an origin with no continuation fails with               *)
(*  [Incomplete_success_path].  Anchor origins contribute no action.    *)
(* ------------------------------------------------------------------ *)

let plan program context =
  match validate program with
  | Error errors -> Error (Invalid_core errors)
  | Ok () -> (
      match unsupported program with
      | Some error -> Error error
      | None -> (
          match program.entry_origin with
          | None -> Error Missing_entry_origin
          | Some entry_oid ->
              let sites = program.origin_sites in
              let continuation_of oid =
                List.assoc_opt oid
                  (List.map
                     (fun (sc : success_continuation) ->
                       (sc.from_origin, sc.target))
                     program.success_continuations)
              in
              let site_of oid =
                List.find_opt (fun s -> origin_id_of_site s = Some oid) sites
              in
              let rec walk visited index planned effects oid =
                if List.mem oid visited then
                  Error (Flow_cycle (List.rev (oid :: visited)))
                else
                  match site_of oid with
                  | None -> Error (Unresolved_origin oid)
                  | Some site -> (
                      match site with
                      | Anchor_origin _ ->
                          advance (oid :: visited) index planned effects oid
                      | Action_origin action -> (
                          match plan_action context index action with
                          | Error _ as err -> err
                          | Ok (planned_action, action_effects) ->
                              advance (oid :: visited) (index + 1)
                                (planned_action :: planned)
                                (List.rev_append action_effects effects) oid)
                      | Together_origin _ -> Error Unsupported_together
                      | Batch_site _ -> Error Unsupported_batch)
              and advance visited index planned effects oid =
                match continuation_of oid with
                | None -> Error (Incomplete_success_path oid)
                | Some Program_complete ->
                    Ok (List.rev planned, unique_effects (List.rev effects))
                | Some (Origin_target next_oid) ->
                    walk visited index planned effects next_oid
              in
              match walk [] 1 [] [] entry_oid with
              | Error _ as err -> err
              | Ok (actions, required_effects) ->
                  Ok
                    {
                      id = context.evaluation_id ^ "/plan";
                      required_effects;
                      actions;
                      groups = [];
                    }))
