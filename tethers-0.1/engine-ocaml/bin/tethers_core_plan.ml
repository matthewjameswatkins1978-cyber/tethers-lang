open Tethers_core
open Tethers_core_validator
open Tethers_outcome

type anchor_snapshot = {
  origin_id : Tethers_core.origin_id;
  data : Yojson.Safe.t;
}

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
  | Unsupported_execution_constraint
  | Unsupported_item_template
  | Missing_capability_projection of capability_id
  | Capability_projection_identity_mismatch of capability_id
  | Capability_projection_digest_mismatch of capability_id
  | Capability_projection_incomplete of capability_id
  | Ambiguous_capability_projection of capability_id
  | Flow_cycle of origin_id list
  | Unresolved_origin of origin_id
  | Unresolved_together_member of origin_id
  | Missing_anchor_snapshot of origin_id
  | Ambiguous_anchor_snapshot of origin_id
  | Anchor_path_missing of origin_id * string list
  | Anchor_path_not_object of origin_id * string list
  | Unsupported_anchor_value_type of origin_id * string list
  | Unresolved_entry_guards
  | Missing_fact_snapshot of host_snapshot_key
  | Ambiguous_fact_snapshot of host_snapshot_key
  | Fact_snapshot_type_mismatch of host_snapshot_key
  | Invalid_guard_comparison of fact_id
  | Missing_reception_anchor
  | Ambiguous_reception_anchor

type runtime_capability_projection = {
  capability_id : capability_id;
  contract_digest : capability_contract_digest;
  runtime : Tethers_protocol.capability;
}

type fact_snapshot = {
  key : host_snapshot_key;
  value : Yojson.Safe.t;
}

type planning_context = {
  evaluation_id : string;
  capabilities : runtime_capability_projection list;
  anchors : anchor_snapshot list;
  facts : fact_snapshot list;
}

type canonical_plan = {
  program_digest : Tethers_core_canonical.program_digest;
  runtime_plan : Tethers_outcome.plan;
}

type runtime_event = {
  name : string;
  data : Yojson.Safe.t;
}

type evaluation_context = {
  evaluation_id : string;
  event : runtime_event;
  capabilities : runtime_capability_projection list;
  facts : fact_snapshot list;
}

type canonical_evaluation =
  | Matched of canonical_plan
  | Not_matched

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
  | Anchor_value _ -> None
  | Batch_item_context _ -> Some Unsupported_batch

let action_inputs_of_site = function
  | Action_origin a -> a.inputs
  | _ -> []

let unsupported program =
  let sites = program.origin_sites in
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

let projection_of (context : planning_context) capability_id contract_digest =
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
(*  Anchor snapshot resolution                                         *)
(*                                                                     *)
(*  For an [Anchor_value (origin_id, path)] binding, the bridge finds   *)
(*  the snapshot for exactly the requested [origin_id], traverses the   *)
(*  ordered path through the JSON tree, and produces a concrete value    *)
(*  suitable for the Runtime Plan argument vocabulary.  Lookup is        *)
(*  identity-based, not first-match, not order-dependent.               *)
(* ------------------------------------------------------------------ *)

let find_snapshot (context : planning_context) origin_id =
  let matching =
    List.filter
      (fun (s : anchor_snapshot) -> s.origin_id = origin_id)
      context.anchors
  in
  match matching with
  | [] -> Error (Missing_anchor_snapshot origin_id)
  | [ snapshot ] -> Ok snapshot
  | _ -> Error (Ambiguous_anchor_snapshot origin_id)

let traverse_path origin_id path json =
  let rec go remaining current =
    match remaining with
    | [] -> Ok current
    | component :: rest -> (
        match current with
        | `Assoc members -> (
            match List.assoc_opt component members with
            | None ->
                Error
                  (Anchor_path_missing (origin_id, path))
            | Some value -> go rest value)
        | _ ->
            Error
              (Anchor_path_not_object (origin_id, path)))
  in
  go path json

let json_value_of_terminal origin_id path = function
  | `String s -> Ok (`String s)
  | `Int i -> Ok (`Int i)
  | `Bool b -> Ok (`Bool b)
  | _ -> Error (Unsupported_anchor_value_type (origin_id, path))

let resolve_anchor_value context origin_id path =
  let open Result.Syntax in
  let* snapshot = find_snapshot context origin_id in
  let* terminal = traverse_path origin_id path snapshot.data in
  json_value_of_terminal origin_id path terminal

(* ------------------------------------------------------------------ *)
(*  Entry guard evaluation                                             *)
(*                                                                     *)
(*  For each entry guard the bridge resolves the canonical FactId to    *)
(*  its declaration in [input_facts], extracts the [HostSnapshotKey]   *)
(*  from [Evaluation_input] provenance, looks up the runtime snapshot   *)
(*  by exactly that key, decodes the JSON value according to the       *)
(*  declared scalar type, and compares against the expected Core value. *)
(*                                                                     *)
(*  Runtime Facts are keyed by [HostSnapshotKey], never by canonical   *)
(*  FactId, never by source-name text, never by list position, and     *)
(*  never by "only available fact".                                    *)
(* ------------------------------------------------------------------ *)

let find_fact_by_id program fact_id =
  List.find_opt (fun (f : Tethers_core.fact) -> f.fact_id = fact_id)
    program.input_facts

let find_fact_snapshot (context : planning_context) host_key =
  let matching =
    List.filter
      (fun (s : fact_snapshot) -> s.key = host_key)
      context.facts
  in
  match matching with
  | [] -> Error (Missing_fact_snapshot host_key)
  | [ snapshot ] -> Ok snapshot
  | _ -> Error (Ambiguous_fact_snapshot host_key)

let runtime_value_matches_type (json : Yojson.Safe.t) (scalar_type : Tethers_core.core_scalar_type) =
  match scalar_type, json with
  | String_type, `String _ -> true
  | Integer_type, `Int _ -> true
  | Boolean_type, `Bool _ -> true
  | _ -> false

let compare_guard operator (runtime_json : Yojson.Safe.t) (expected : Tethers_core.core_value) =
  match operator, runtime_json, expected with
  | Equals, `String r, String_value e -> r = e
  | Equals, `Int r, Integer_value e -> r = e
  | Equals, `Bool r, Boolean_value e -> r = e
  | Contains, `String r, String_value e ->
      let r_len = String.length r in
      let e_len = String.length e in
      if e_len = 0 then true
      else
        let rec search i =
          if i + e_len > r_len then false
          else if String.sub r i e_len = e then true
          else search (i + 1)
        in
        search 0
  | Greater_than, `Int r, Integer_value e -> r > e
  | Greater_than_or_equal, `Int r, Integer_value e -> r >= e
  | _ -> false

let validate_guard_expected operator (expected : Tethers_core.core_value) (scalar_type : Tethers_core.core_scalar_type) =
  match operator, expected, scalar_type with
  | Contains, String_value _, String_type -> true
  | Equals, String_value _, String_type -> true
  | Equals, Integer_value _, Integer_type -> true
  | Equals, Boolean_value _, Boolean_type -> true
  | Greater_than, Integer_value _, Integer_type -> true
  | Greater_than_or_equal, Integer_value _, Integer_type -> true
  | _ -> false

type guard_single_result = Guard_ok | Guard_false

let evaluate_single_guard (context : planning_context) program (guard : Tethers_core.fact_guard) =
  let open Result.Syntax in
  let* fact =
    match find_fact_by_id program guard.fact_id with
    | Some f -> Ok f
    | None -> Error (Invalid_guard_comparison guard.fact_id)
  in
  let* host_key, scalar_type =
    match fact.provenance with
    | Evaluation_input (hk, st) -> Ok (hk, st)
    | _ -> Error (Invalid_guard_comparison guard.fact_id)
  in
  let* snapshot = find_fact_snapshot context host_key in
  if not (runtime_value_matches_type snapshot.value scalar_type) then
    Error (Fact_snapshot_type_mismatch host_key)
  else if not (validate_guard_expected guard.operator guard.expected scalar_type) then
    Error (Invalid_guard_comparison guard.fact_id)
  else if compare_guard guard.operator snapshot.value guard.expected then Ok Guard_ok
  else Ok Guard_false

type guard_result =
  | All_guards_passed
  | Guard_not_matched

let evaluate_entry_guards (context : planning_context) program =
  let guards = program.entry_guards in
  let rec loop = function
    | [] -> Ok All_guards_passed
    | guard :: rest ->
        (match evaluate_single_guard context program guard with
         | Ok Guard_ok -> loop rest
         | Ok Guard_false -> Ok Guard_not_matched
         | Error _ as err -> err)
  in
  match guards with
  | [] -> Ok All_guards_passed
  | _ -> loop guards

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

let plan_action (context : planning_context) index (a : action_origin) =
  let rec build_arguments acc = function
    | [] -> Ok (List.rev acc)
    | (ai : action_input) :: rest -> (
        match ai.binding with
        | Literal_value value ->
            let name = string_of_capability_input_name ai.input_name in
            build_arguments ((name, json_of_core_value value) :: acc) rest
        | Fact_through_role _ -> Error Unsupported_role_binding
        | Fact_from_origin _ -> Error Unsupported_fact_binding
        | Anchor_value (origin_id, path) ->
            let name = string_of_capability_input_name ai.input_name in
            let open Result.Syntax in
            let* value = resolve_anchor_value context origin_id path in
            build_arguments ((name, value) :: acc) rest
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

let plan_core program (context : planning_context) =
  match unsupported program with
  | Some error -> Error error
  | None -> (
      match program.entry_origin with
      | None -> Error Missing_entry_origin
      | Some entry_oid ->
          let sites = program.origin_sites in
          let together_sites =
            List.filter_map
              (function Together_origin t -> Some t | _ -> None)
              sites
          in
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
                  | Together_origin _ ->
                      advance (oid :: visited) index planned effects oid
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
              let action_plan_index =
                let rec collect acc idx = function
                  | [] -> List.rev acc
                  | site :: rest ->
                      (match site with
                       | Action_origin a ->
                           collect ((a.action_origin_id, idx) :: acc) (idx + 1) rest
                       | _ -> collect acc idx rest)
                in
                collect [] 1 sites
              in
              let plan_idx_of_oid oid =
                match List.assoc_opt oid action_plan_index with
                | Some i -> i
                | None -> -1
              in
              let groups =
                let open Tethers_core in
                let rec resolve_groups acc = function
                  | [] -> Ok (List.rev acc)
                  | t :: rest ->
                      let rec resolve_members acc_ids = function
                        | [] -> Ok (List.rev acc_ids)
                        | member_oid :: mrest ->
                            (match plan_idx_of_oid member_oid with
                             | -1 -> Error (Unresolved_together_member member_oid)
                             | idx ->
                                 resolve_members
                                   ((idx, "action_" ^ string_of_int idx) :: acc_ids)
                                   mrest)
                      in
                      match resolve_members [] t.member_origin_ids with
                      | Error _ as err -> err
                      | Ok unsorted ->
                          let member_action_ids =
                            List.sort (fun (a, _) (b, _) -> compare a b) unsorted
                            |> List.map snd
                          in
                          resolve_groups
                            ({ group_id = string_of_group_id t.group_id;
                               member_action_ids } :: acc)
                            rest
                in
                resolve_groups [] together_sites
              in
              match groups with
              | Error _ as err -> err
              | Ok groups ->
              Ok
                {
                  id = context.evaluation_id ^ "/plan";
                  required_effects;
                  actions;
                  groups;
                })

let plan program (context : planning_context) =
  match validate program with
  | Error errors -> Error (Invalid_core errors)
  | Ok () -> (
      if program.entry_guards <> [] then Error Unresolved_entry_guards
      else plan_core program context)

let plan_canonicalized canonicalized (context : planning_context) =
  let c_program = Tethers_core_canonical.canonical_program canonicalized in
  let c_digest = Tethers_core_canonical.program_digest canonicalized in
  if c_program.entry_guards <> [] then Error Unresolved_entry_guards
  else match plan_core c_program context with
    | Error err -> Error err
    | Ok runtime_plan -> Ok { program_digest = c_digest; runtime_plan }

let plan_internal program (context : planning_context) =
  match validate program with
  | Error errors -> Error (Invalid_core errors)
  | Ok () -> plan_core program context

let evaluate_canonicalized canonicalized context =
  let c_program = Tethers_core_canonical.canonical_program canonicalized in
  let c_digest = Tethers_core_canonical.program_digest canonicalized in
  let open Result.Syntax in
  (* Anchor reception: exactly one top-level Anchor_origin required *)
  let anchor_origins =
    List.filter_map
      (function Anchor_origin a -> Some a | _ -> None)
      c_program.origin_sites
  in
  let* anchor =
    match anchor_origins with
    | [] -> Error Missing_reception_anchor
    | [ a ] -> Ok a
    | _ :: _ -> Error Ambiguous_reception_anchor
  in
  (* Exact event name match *)
  if anchor.event_name <> context.event.name then
    Ok Not_matched
  else
    (* Event matched: bind event data to canonical Anchor OriginId *)
    let derived_anchor =
      { origin_id = anchor.anchor_origin_id; data = context.event.data }
    in
    let planning_ctx =
      { evaluation_id = context.evaluation_id;
        capabilities = context.capabilities;
        anchors = [ derived_anchor ];
        facts = context.facts }
    in
    let* guard_result = evaluate_entry_guards planning_ctx c_program in
    match guard_result with
    | Guard_not_matched -> Ok Not_matched
    | All_guards_passed ->
        match plan_internal c_program planning_ctx with
        | Error err -> Error err
        | Ok runtime_plan -> Ok (Matched { program_digest = c_digest; runtime_plan })
