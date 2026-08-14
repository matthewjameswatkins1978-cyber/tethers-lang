open Tethers_core

type capability_binding = {
  source_name : string;
  capability_id : capability_id;
  contract_digest : capability_contract_digest;
}

type input_fact_binding = {
  source_name : string;
  fact : fact;
}

type lowering_environment = {
  program_id : program_id;
  core_version : core_version;
  capabilities : capability_binding list;
  input_facts : input_fact_binding list;
}

type lowering_error =
  | Unsupported_construct of string
  | Unknown_capability of string
  | Duplicate_capability of string
  | Unknown_fact of string
  | Duplicate_fact of string
  | Conflicting_capability_contract of capability_id
  | Missing_anchor_reference of string

(* ------------------------------------------------------------------ *)
(*  Deterministic pre-canonical ID assignment                          *)
(* ------------------------------------------------------------------ *)

let anchor_origin_id = origin_id_of_string "O_anchor"

let action_origin_id i = origin_id_of_string ("O_action_" ^ string_of_int i)

(* ------------------------------------------------------------------ *)
(*  Value lowering                                                     *)
(* ------------------------------------------------------------------ *)

let lower_core_value = function
  | Tether_parser.String_value s -> String_value s
  | Tether_parser.Int_value i -> Integer_value i
  | Tether_parser.Bool_value b -> Boolean_value b
  | Tether_parser.Reference _ ->
      failwith "TETHERS_CORE_LOWERER BUG: Reference must be handled as anchor path, not core_value"

(* ------------------------------------------------------------------ *)
(*  Anchor reference lowering                                          *)
(* ------------------------------------------------------------------ *)

let lower_anchor_reference origin_id ref =
  let prefix = "anchor." in
  let prefix_len = String.length prefix in
  let path_text = String.sub ref prefix_len (String.length ref - prefix_len) in
  let parts = String.split_on_char '.' path_text in
  Anchor_value (origin_id, parts)

(* ------------------------------------------------------------------ *)
(*  Action argument lowering                                           *)
(* ------------------------------------------------------------------ *)

let lower_argument origin_id (name, value) =
  let input_name = capability_input_name_of_string name in
  match value with
  | Tether_parser.String_value s ->
      Ok { input_name; binding = Literal_value (String_value s) }
  | Tether_parser.Int_value i ->
      Ok { input_name; binding = Literal_value (Integer_value i) }
  | Tether_parser.Bool_value b ->
      Ok { input_name; binding = Literal_value (Boolean_value b) }
  | Tether_parser.Reference r ->
      let prefix = "anchor." in
      let prefix_len = String.length prefix in
      if String.length r > prefix_len && String.sub r 0 prefix_len = prefix then
        Ok { input_name; binding = lower_anchor_reference origin_id r }
      else
        Error (Missing_anchor_reference r)

(* ------------------------------------------------------------------ *)
(*  Capability resolution                                              *)
(* ------------------------------------------------------------------ *)

let resolve_capability caps name =
  let matches = List.filter (fun (b : capability_binding) -> b.source_name = name) caps in
  match matches with
  | [] -> Error (Unknown_capability name)
  | [ b ] -> Ok (b.capability_id, b.contract_digest)
  | _ :: _ :: _ -> Error (Duplicate_capability name)

(* ------------------------------------------------------------------ *)
(*  Input Fact resolution                                              *)
(* ------------------------------------------------------------------ *)

let resolve_fact all_facts name =
  let matches = List.filter (fun (b : input_fact_binding) -> b.source_name = name) all_facts in
  match matches with
  | [] -> Error (Unknown_fact name)
  | [ b ] -> Ok b.fact
  | _ :: _ :: _ -> Error (Duplicate_fact name)

(* ------------------------------------------------------------------ *)
(*  Operator lowering                                                  *)
(* ------------------------------------------------------------------ *)

let lower_operator = function
  | Tether_parser.Is -> Equals
  | Tether_parser.Contains -> Contains
  | Tether_parser.Greater_than -> Greater_than
  | Tether_parser.Greater_than_or_equal -> Greater_than_or_equal

(* ------------------------------------------------------------------ *)
(*  Fact Guard lowering                                                *)
(* ------------------------------------------------------------------ *)

let lower_guard all_facts (cond : Tether_parser.condition) =
  match resolve_fact all_facts cond.fact with
  | Error _ as e -> e
  | Ok resolved_fact ->
      let expected = lower_core_value cond.expected in
      Ok { fact_id = resolved_fact.fact_id; operator = lower_operator cond.operator; expected }

let lower_guards all_facts conditions =
  let rec loop acc = function
    | [] -> Ok (List.rev acc)
    | c :: cs ->
        (match lower_guard all_facts c with
         | Ok g -> loop (g :: acc) cs
         | Error _ as e -> e)
  in
  loop [] conditions

(* ------------------------------------------------------------------ *)
(*  Together detection                                                 *)
(* ------------------------------------------------------------------ *)

type lowered_group = {
  group_id : group_id;
  member_origin_ids : origin_id list;
}

let extract_actions_and_groups items =
  let rec loop acc_actions acc_groups = function
    | [] -> Ok (List.rev acc_actions, List.rev acc_groups)
    | Tether_parser.Action a :: rest ->
        loop (a :: acc_actions) acc_groups rest
    | Tether_parser.Together members :: rest ->
        let member_count = List.length acc_actions in
        let group_id = group_id_of_string ("G_group_" ^ string_of_int (List.length acc_groups + 1)) in
        let member_ids =
          List.init (List.length members) (fun i ->
            action_origin_id (member_count + i + 1))
        in
        let member_actions = List.map (fun m -> (m, true)) members in
        loop
          (List.map fst member_actions @ acc_actions)
          ({ group_id; member_origin_ids = member_ids } :: acc_groups)
          rest
  in
  loop [] [] items

(* ------------------------------------------------------------------ *)
(*  Result monad binding operator for lowering_error                    *)
(* ------------------------------------------------------------------ *)

let ( let* ) = Result.bind

(* ------------------------------------------------------------------ *)
(*  Main lowering function                                             *)
(* ------------------------------------------------------------------ *)

let lower env (tether : Tether_parser.tether) =
  let* actions, groups = extract_actions_and_groups tether.actions in
  let* guards = lower_guards env.input_facts tether.conditions in
  let* action_origins_rev, used_cap_contracts_rev =
    let rec loop i acc_origins acc_contracts actions =
      match actions with
      | [] -> Ok (List.rev acc_origins, List.rev acc_contracts)
      | (a : Tether_parser.action) :: rest ->
          let* (cap_id, contract_dig) = resolve_capability env.capabilities a.capability in
          let origin_id = action_origin_id i in
          let* inputs =
            let rec lower_args acc = function
              | [] -> Ok (List.rev acc)
              | arg :: more ->
                  (match lower_argument anchor_origin_id arg with
                   | Ok input -> lower_args (input :: acc) more
                   | Error _ as e -> e)
            in
            lower_args [] a.arguments
          in
          let contract = {
            capability_id = cap_id;
            contract_digest = contract_dig;
            schema_description = "";
          } in
          let origin = {
            action_origin_id = origin_id;
            capability_id = cap_id;
            contract_digest = contract_dig;
            inputs;
            declared_facts = [];
            execution_constraints = [];
          } in
          loop (i + 1) (origin :: acc_origins) (contract :: acc_contracts) rest
    in
    loop 1 [] [] actions
  in
  let action_origin_ids = List.map (fun (o : action_origin) -> o.action_origin_id) action_origins_rev in
  let success_continuations =
    let rec build_chain = function
      | [] -> []
      | [ last_id ] -> [ { from_origin = last_id; target = Program_complete } ]
      | a :: (b :: _ as rest) ->
          { from_origin = a; target = Origin_target b } :: build_chain rest
    in
    build_chain action_origin_ids
  in
  let together_origins =
    List.map (fun (g : lowered_group) ->
      Together_origin {
        together_origin_id = origin_id_of_string
          ("O_together_" ^ string_of_group_id g.group_id);
        group_id = g.group_id;
        member_origin_ids = g.member_origin_ids;
        objective = All_members_succeed;
      }
    ) groups
  in
  let entry_origin =
    match action_origins_rev with
    | first :: _ -> Some first.action_origin_id
    | [] -> None
  in
  let used_fact_ids = List.map (fun (g : fact_guard) -> g.fact_id) guards in
  let program_input_facts =
    List.filter (fun (b : input_fact_binding) ->
      List.mem b.fact.fact_id used_fact_ids
    ) env.input_facts
    |> List.map (fun b -> b.fact)
  in
  let* capability_contracts =
    let rec dedup (acc : capability_contract list) (input : capability_contract list) =
      match input with
      | [] -> Ok (List.rev acc)
      | (c : capability_contract) :: rest ->
          (match
             List.find_opt
               (fun (existing : capability_contract) -> existing.capability_id = c.capability_id)
               acc
           with
           | None -> dedup (c :: acc) rest
           | Some existing ->
               if existing.contract_digest = c.contract_digest then dedup acc rest
               else Error (Conflicting_capability_contract c.capability_id))
    in
    dedup [] used_cap_contracts_rev
  in
  let anchor_origin_record = {
    anchor_origin_id;
    event_name = tether.anchor;
    declared_facts = [];
  } in
  let action_origin_sites = List.map (fun o -> Action_origin o) action_origins_rev in
  Ok {
    program_id = env.program_id;
    core_version = env.core_version;
    input_facts = program_input_facts;
    entry_guards = guards;
    entry_origin;
    success_continuations;
    origin_sites = Anchor_origin anchor_origin_record :: action_origin_sites @ together_origins;
    branches = [];
    roles = [];
    item_templates = [];
    capability_contracts;
  }
