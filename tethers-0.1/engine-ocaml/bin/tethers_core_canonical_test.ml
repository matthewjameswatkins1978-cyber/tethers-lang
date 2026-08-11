open Tethers_core
open Tethers_core_canonical

(* ================================================================== *)
(*  Test helpers                                                        *)
(* ================================================================== *)

let assert_ok = function Ok x -> x | Error _ -> failwith "expected Ok"

let digest_of p =
  match canonicalize p with
  | Ok c -> string_of_program_digest (program_digest c)
  | Error (Invalid_core _) -> failwith "expected Ok"

let bytes_of p =
  let c = assert_ok (canonicalize p) in
  canonical_bytes c

let canon_prog_of p =
  let c = assert_ok (canonicalize p) in
  canonical_program c

let mk_eval_fact fid key stype =
  { fact_id = fact_id_of_string fid;
    schema_description = "desc_" ^ fid;
    provenance = Evaluation_input (host_snapshot_key_of_string key, stype) }

let mk_origin_fact fid oid =
  { fact_id = fact_id_of_string fid;
    schema_description = "desc_" ^ fid;
    provenance = Origin_provenance (origin_id_of_string oid) }

let mk_anchor_origin oid event_name facts =
  Anchor_origin { anchor_origin_id = origin_id_of_string oid;
                  event_name;
                  declared_facts = facts }

let mk_action_origin oid cap_id contract_dig inputs facts =
  Action_origin { action_origin_id = origin_id_of_string oid;
                  capability_id = capability_id_of_string cap_id;
                  contract_digest = capability_contract_digest_of_string contract_dig;
                  inputs;
                  declared_facts = facts;
                  execution_constraints = [] }

let mk_lit_input name v =
  { input_name = capability_input_name_of_string name; binding = Literal_value v }

let mk_cap_contract cap_id digest =
  { capability_id = capability_id_of_string cap_id;
    contract_digest = capability_contract_digest_of_string digest;
    schema_description = "cap desc" }

let mk_success_cont from_oid target =
  { from_origin = origin_id_of_string from_oid; target }

let mk_program ?(id="test_prog") ?(core_version=core_version_of_string "0.1.0")
    ?(input_facts=[]) ?(entry_guards=[]) ?(entry_origin=None)
    ?(success_continuations=[]) ?(origin_sites=[]) ?(branches=[])
    ?(roles=[]) ?(item_templates=[]) ?(capability_contracts=[]) () =
  { program_id = program_id_of_string id;
    core_version;
    input_facts;
    entry_guards;
    entry_origin;
    success_continuations;
    origin_sites;
    branches;
    roles;
    item_templates;
    capability_contracts }

(* ================================================================== *)
(*  A. Baseline                                                         *)
(* ================================================================== *)

let test_baseline () =
  let prog = mk_program
    ~input_facts:[ mk_eval_fact "F1" "hk1" String_type ]
    ~entry_origin:(Some (origin_id_of_string "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "event.arrived" [];
      mk_action_origin "O_action" "cap.send" "sha256:abc"
        [ mk_lit_input "msg" (String_value "hello") ] []
    ]
    ~capability_contracts:[ mk_cap_contract "cap.send" "sha256:abc" ]
    ()
  in
  match canonicalize prog with
  | Ok c ->
      let canon = canonical_program c in
      assert (List.length canon.origin_sites = 2);
      let d = string_of_program_digest (program_digest c) in
      assert (String.length d > 7);
      assert (String.sub d 0 7 = "sha256:")
  | Error _ -> failwith "baseline: expected Ok"

(* ================================================================== *)
(*  B. Determinism                                                      *)
(* ================================================================== *)

let test_determinism () =
  let prog = mk_program
    ~input_facts:[ mk_eval_fact "F1" "hk1" String_type ]
    ~entry_origin:(Some (origin_id_of_string "anchor"))
    ~origin_sites:[
      mk_anchor_origin "anchor" "ev" [];
      mk_action_origin "act" "cap.x" "sha256:d1"
        [ mk_lit_input "x" (Integer_value 1) ] []
    ]
    ~capability_contracts:[ mk_cap_contract "cap.x" "sha256:d1" ]
    ()
  in
  let d1 = digest_of prog in
  let d2 = digest_of prog in
  let b1 = bytes_of prog in
  let b2 = bytes_of prog in
  assert (d1 = d2);
  assert (b1 = b2)

(* ================================================================== *)
(*  C. Temporary Origin ID independence                                *)
(* ================================================================== *)

let test_origin_id_independence () =
  let mk oid1 oid2 =
    mk_program
      ~input_facts:[ mk_eval_fact "fx" "hk1" String_type ]
      ~entry_origin:(Some (origin_id_of_string oid1))
      ~origin_sites:[
        mk_anchor_origin oid1 "event.x" [];
        mk_action_origin oid2 "cap.x" "sha256:d1"
          [ mk_lit_input "x" (String_value "a") ]
          [ mk_origin_fact "fy" oid2 ]
      ]
      ~capability_contracts:[ mk_cap_contract "cap.x" "sha256:d1" ]
      ()
  in
  let d1 = digest_of (mk "banana_thing_947" "fruit_42") in
  let d2 = digest_of (mk "O_action_1" "O_output_1") in
  assert (d1 = d2)

(* ================================================================== *)
(*  D. All internal ID independence                                     *)
(* ================================================================== *)

let test_all_id_independence () =
  let p1 = mk_program
    ~input_facts:[ mk_eval_fact "input_xyz" "hk1" String_type ]
    ~entry_guards:[{ fact_id = fact_id_of_string "input_xyz";
                     operator = Equals;
                     expected = String_value "t" }]
    ~entry_origin:(Some (origin_id_of_string "origin_abc"))
    ~success_continuations:[
      mk_success_cont "origin_abc" (Origin_target (origin_id_of_string "origin_def"))
    ]
    ~origin_sites:[
      mk_anchor_origin "origin_abc" "ev" [];
      mk_action_origin "origin_def" "cap.x" "sha256:d1"
        [ mk_lit_input "x" (String_value "v") ]
        [ mk_origin_fact "fact_gh" "origin_def" ]
    ]
    ~roles:[{ role_id = role_id_of_string "role_ijk";
              scope = Program_scope;
              fact_contract = Role_fact_contract [ fact_id_of_string "fact_gh" ];
              eligible_fulfillment = role_fulfillment_of_string "fulfill_x" }]
    ~capability_contracts:[ mk_cap_contract "cap.x" "sha256:d1" ]
    ()
  in
  let p2 = mk_program
    ~input_facts:[ mk_eval_fact "F1" "hk1" String_type ]
    ~entry_guards:[{ fact_id = fact_id_of_string "F1";
                     operator = Equals;
                     expected = String_value "t" }]
    ~entry_origin:(Some (origin_id_of_string "O1"))
    ~success_continuations:[
      mk_success_cont "O1" (Origin_target (origin_id_of_string "O2"))
    ]
    ~origin_sites:[
      mk_anchor_origin "O1" "ev" [];
      mk_action_origin "O2" "cap.x" "sha256:d1"
        [ mk_lit_input "x" (String_value "v") ]
        [ mk_origin_fact "F2" "O2" ]
    ]
    ~roles:[{ role_id = role_id_of_string "R1";
              scope = Program_scope;
              fact_contract = Role_fact_contract [ fact_id_of_string "F2" ];
              eligible_fulfillment = role_fulfillment_of_string "fulfill_x" }]
    ~capability_contracts:[ mk_cap_contract "cap.x" "sha256:d1" ]
    ()
  in
  let d1 = digest_of p1 in
  let d2 = digest_of p2 in
  assert (d1 = d2)

(* ================================================================== *)
(*  E. Storage-order independence                                       *)
(* ================================================================== *)

let test_storage_order_independence () =
  let facts ids =
    List.map (fun (fid, key) -> mk_eval_fact fid key String_type) ids
  in
  let origins oids =
    List.map (fun (oid, cap) ->
      mk_action_origin oid cap "sha256:d1"
        [ mk_lit_input "x" (String_value "v") ] []) oids
  in
  let p1 = mk_program
    ~input_facts:(facts [("F1","k1"); ("F2","k2"); ("F3","k3")])
    ~entry_origin:(Some (origin_id_of_string "O_anchor"))
    ~origin_sites:(
      mk_anchor_origin "O_anchor" "ev" [] ::
      origins [("O_A","cap.a"); ("O_M","cap.m"); ("O_Z","cap.z")]
    )
    ~capability_contracts:(List.map (fun c -> mk_cap_contract c "sha256:d1") ["cap.a"; "cap.m"; "cap.z"])
    ()
  in
  let p2 = mk_program
    ~input_facts:(facts [("F3","k3"); ("F2","k2"); ("F1","k1")])
    ~entry_origin:(Some (origin_id_of_string "O_anchor"))
    ~origin_sites:(
      mk_anchor_origin "O_anchor" "ev" [] ::
      origins [("O_Z","cap.z"); ("O_A","cap.a"); ("O_M","cap.m")]
    )
    ~capability_contracts:(List.map (fun c -> mk_cap_contract c "sha256:d1") ["cap.z"; "cap.a"; "cap.m"])
    ()
  in
  assert (digest_of p1 = digest_of p2)

(* ================================================================== *)
(*  F. Named Action input reordering                                    *)
(* ================================================================== *)

let test_input_reordering () =
  let mk inputs =
    mk_program
      ~input_facts:[ mk_eval_fact "fx" "h" String_type ]
      ~entry_origin:(Some (origin_id_of_string "oa"))
      ~origin_sites:[
        mk_anchor_origin "oa" "ev" [];
        mk_action_origin "ob" "cap.x" "sha256:d1" inputs [ mk_origin_fact "fy" "ob" ]
      ]
      ~capability_contracts:[ mk_cap_contract "cap.x" "sha256:d1" ]
      ()
  in
  let p1 = mk [
    mk_lit_input "name" (String_value "Alice");
    mk_lit_input "age" (Integer_value 30);
    mk_lit_input "city" (String_value "NYC");
  ] in
  let p2 = mk [
    mk_lit_input "city" (String_value "NYC");
    mk_lit_input "age" (Integer_value 30);
    mk_lit_input "name" (String_value "Alice");
  ] in
  assert (digest_of p1 = digest_of p2)

(* ================================================================== *)
(*  G. Together ordering                                                *)
(* ================================================================== *)

let test_together_ordering () =
  let mk members =
    mk_program
      ~input_facts:[ mk_eval_fact "fx" "hk" String_type ]
      ~entry_origin:(Some (origin_id_of_string "ent"))
      ~origin_sites:[
        mk_anchor_origin "ent" "ev" [];
        mk_action_origin "A" "cap.a" "sha256:a"
          [ mk_lit_input "x" (String_value "a1") ] [];
        mk_action_origin "B" "cap.a" "sha256:a"
          [ mk_lit_input "x" (String_value "a2") ] [];
        Together_origin {
          together_origin_id = origin_id_of_string "TG";
          group_id = group_id_of_string "G1";
          member_origin_ids = List.map (fun s -> origin_id_of_string s) members;
          objective = All_members_succeed }
      ]
      ~capability_contracts:[ mk_cap_contract "cap.a" "sha256:a" ]
      ()
  in
  assert (digest_of (mk ["A"; "B"]) = digest_of (mk ["B"; "A"]))

(* ================================================================== *)
(*  H. True symmetry                                                    *)
(* ================================================================== *)

let test_true_symmetry () =
  let mk a_id b_id =
    mk_program
      ~input_facts:[ mk_eval_fact "f" "h" String_type ]
      ~entry_origin:(Some (origin_id_of_string "e1"))
      ~origin_sites:[
        mk_anchor_origin "e1" "ev" [];
        mk_action_origin a_id "cap.sym" "sha256:s"
          [ mk_lit_input "m" (String_value "payload") ] [];
        mk_action_origin b_id "cap.sym" "sha256:s"
          [ mk_lit_input "m" (String_value "payload") ] [];
        Together_origin {
          together_origin_id = origin_id_of_string "TG";
          group_id = group_id_of_string "G1";
          member_origin_ids = [origin_id_of_string a_id; origin_id_of_string b_id];
          objective = All_members_succeed }
      ]
      ~capability_contracts:[ mk_cap_contract "cap.sym" "sha256:s" ]
      ()
  in
  let d1 = digest_of (mk "X" "Y") in
  let d2 = digest_of (mk "Y" "X") in
  assert (d1 = d2);
  let cp = canon_prog_of (mk "X" "Y") in
  let togs = List.filter (function Together_origin _ -> true | _ -> false) cp.origin_sites in
  assert (List.length togs = 1);
  (match List.hd togs with
   | Together_origin t -> assert (List.length t.member_origin_ids = 2)
   | _ -> failwith "expected Together_origin")

(* ================================================================== *)
(*  I. Branch ordering                                                  *)
(* ================================================================== *)

let test_branch_ordering () =
  let mk ordered_outcomes =
    mk_program
      ~input_facts:[ mk_eval_fact "f" "h" String_type ]
      ~entry_origin:(Some (origin_id_of_string "oa"))
      ~origin_sites:[
        mk_anchor_origin "oa" "ev" [];
        mk_action_origin "ob" "cap.x" "sha256:d1"
          [ mk_lit_input "x" (String_value "v") ] []
      ]
      ~branches:[{
        branch_id = branch_id_of_string "B1";
        branch_subject = origin_id_of_string "oa";
        outcome_branches = ordered_outcomes }]
      ~capability_contracts:[ mk_cap_contract "cap.x" "sha256:d1" ]
      ()
  in
  let p1 = mk [(Success, Continue_to (origin_id_of_string "ob"));
               (Failure, Stop); (Uncertain, Stop)] in
  let p2 = mk [(Uncertain, Stop);
               (Success, Continue_to (origin_id_of_string "ob"));
               (Failure, Stop)] in
  assert (digest_of p1 = digest_of p2)

(* ================================================================== *)
(*  J. Neutral descriptions                                             *)
(* ================================================================== *)

let test_neutral_descriptions () =
  let mk fact_desc cap_desc =
    let f = { fact_id = fact_id_of_string "F1";
              schema_description = fact_desc;
              provenance = Evaluation_input (host_snapshot_key_of_string "k", String_type) } in
    let cap = { capability_id = capability_id_of_string "cap.x";
                contract_digest = capability_contract_digest_of_string "sha256:d1";
                schema_description = cap_desc } in
    mk_program
      ~input_facts:[ f ]
      ~entry_origin:(Some (origin_id_of_string "oa"))
      ~origin_sites:[
        mk_anchor_origin "oa" "ev" [];
        mk_action_origin "ob" "cap.x" "sha256:d1"
          [ mk_lit_input "x" (String_value "v") ] []
      ]
      ~capability_contracts:[ cap ]
      ()
  in
  assert (digest_of (mk "desc A" "cap A") = digest_of (mk "desc B" "cap B"))

(* ================================================================== *)
(*  K. ProgramId neutral                                                *)
(* ================================================================== *)

let test_program_id_neutral () =
  let mk pid =
    mk_program ~id:pid
      ~input_facts:[ mk_eval_fact "F1" "k" String_type ]
      ~entry_origin:(Some (origin_id_of_string "oa"))
      ~origin_sites:[
        mk_anchor_origin "oa" "ev" [];
        mk_action_origin "ob" "cap.x" "sha256:d1"
          [ mk_lit_input "x" (String_value "v") ] []
      ]
      ~capability_contracts:[ mk_cap_contract "cap.x" "sha256:d1" ]
      ()
  in
  assert (digest_of (mk "prog-A") = digest_of (mk "prog-B"))

(* ================================================================== *)
(*  L. Literal meaning changes digest                                   *)
(* ================================================================== *)

let test_literal_changes_digest () =
  let mk v =
    mk_program
      ~input_facts:[ mk_eval_fact "F1" "k" String_type ]
      ~entry_origin:(Some (origin_id_of_string "oa"))
      ~origin_sites:[
        mk_anchor_origin "oa" "ev" [];
        mk_action_origin "ob" "cap.x" "sha256:d1"
          [ mk_lit_input "x" v ] []
      ]
      ~capability_contracts:[ mk_cap_contract "cap.x" "sha256:d1" ]
      ()
  in
  assert (digest_of (mk (String_value "hello")) <> digest_of (mk (String_value "world")))

(* ================================================================== *)
(*  M. CapabilityId changes digest                                      *)
(* ================================================================== *)

let test_capability_id_changes_digest () =
  let mk cid =
    mk_program
      ~input_facts:[ mk_eval_fact "F1" "k" String_type ]
      ~entry_origin:(Some (origin_id_of_string "oa"))
      ~origin_sites:[
        mk_anchor_origin "oa" "ev" [];
        mk_action_origin "ob" cid "sha256:d1"
          [ mk_lit_input "x" (String_value "v") ] []
      ]
      ~capability_contracts:[ mk_cap_contract cid "sha256:d1" ]
      ()
  in
  assert (digest_of (mk "cap.A") <> digest_of (mk "cap.B"))

(* ================================================================== *)
(*  N. Contract digest changes digest                                   *)
(* ================================================================== *)

let test_contract_digest_changes_digest () =
  let mk dig =
    mk_program
      ~input_facts:[ mk_eval_fact "F1" "k" String_type ]
      ~entry_origin:(Some (origin_id_of_string "oa"))
      ~origin_sites:[
        mk_anchor_origin "oa" "ev" [];
        mk_action_origin "ob" "cap.x" dig
          [ mk_lit_input "x" (String_value "v") ] []
      ]
      ~capability_contracts:[ mk_cap_contract "cap.x" dig ]
      ()
  in
  assert (digest_of (mk "sha256:aaa") <> digest_of (mk "sha256:bbb"))

(* ================================================================== *)
(*  O. Anchor meaning                                                   *)
(* ================================================================== *)

let test_anchor_meaning () =
  let mk ev =
    mk_program
      ~input_facts:[ mk_eval_fact "F1" "k" String_type ]
      ~entry_origin:(Some (origin_id_of_string "oa"))
      ~origin_sites:[
        mk_anchor_origin "oa" ev [];
        mk_action_origin "ob" "cap.x" "sha256:d1"
          [ mk_lit_input "x" (String_value "v") ] []
      ]
      ~capability_contracts:[ mk_cap_contract "cap.x" "sha256:d1" ]
      ()
  in
  assert (digest_of (mk "event.a") <> digest_of (mk "event.b"))

(* ================================================================== *)
(*  P. Guard meaning changes digest                                     *)
(* ================================================================== *)

let test_guard_meaning () =
  let mk op expected =
    mk_program
      ~input_facts:[ mk_eval_fact "F1" "k" String_type ]
      ~entry_guards:[{ fact_id = fact_id_of_string "F1";
                       operator = op; expected }]
      ~entry_origin:(Some (origin_id_of_string "oa"))
      ~origin_sites:[
        mk_anchor_origin "oa" "ev" [];
        mk_action_origin "ob" "cap.x" "sha256:d1"
          [ mk_lit_input "x" (String_value "v") ] []
      ]
      ~capability_contracts:[ mk_cap_contract "cap.x" "sha256:d1" ]
      ()
  in
  let p_eq = mk Equals (String_value "a") in
  let p_ne = mk Contains (String_value "b") in
  let p_val = mk Equals (String_value "c") in
  assert (digest_of p_eq <> digest_of p_ne);
  assert (digest_of p_eq <> digest_of p_val)

(* ================================================================== *)
(*  Q. Control flow changes digest                                      *)
(* ================================================================== *)

let test_control_flow_changes_digest () =
  let mk target =
    mk_program
      ~input_facts:[ mk_eval_fact "F1" "k" String_type ]
      ~entry_origin:(Some (origin_id_of_string "oa"))
      ~success_continuations:[ mk_success_cont "oa" target ]
      ~origin_sites:[
        mk_anchor_origin "oa" "ev" [];
        mk_action_origin "ob" "cap.x" "sha256:d1"
          [ mk_lit_input "x" (String_value "v") ] [];
        mk_action_origin "oc" "cap.x" "sha256:d1"
          [ mk_lit_input "x" (String_value "v2") ] []
      ]
      ~capability_contracts:[ mk_cap_contract "cap.x" "sha256:d1" ]
      ()
  in
  let d1 = digest_of (mk (Origin_target (origin_id_of_string "ob"))) in
  let d2 = digest_of (mk (Origin_target (origin_id_of_string "oc"))) in
  assert (d1 <> d2)

(* ================================================================== *)
(*  R. Branch meaning changes digest                                    *)
(* ================================================================== *)

let test_branch_meaning_changes () =
  let mk success_target =
    mk_program
      ~input_facts:[ mk_eval_fact "F1" "k" String_type ]
      ~entry_origin:(Some (origin_id_of_string "oa"))
      ~origin_sites:[
        mk_anchor_origin "oa" "ev" [];
        mk_action_origin "ob" "cap.x" "sha256:d1"
          [ mk_lit_input "x" (String_value "v") ] [];
        mk_action_origin "oc" "cap.x" "sha256:d1"
          [ mk_lit_input "x" (String_value "v2") ] []
      ]
      ~branches:[{
        branch_id = branch_id_of_string "B1";
        branch_subject = origin_id_of_string "oa";
        outcome_branches = [(Success, success_target); (Failure, Stop)] }]
      ~capability_contracts:[ mk_cap_contract "cap.x" "sha256:d1" ]
      ()
  in
  assert (digest_of (mk (Continue_to (origin_id_of_string "ob")))
          <> digest_of (mk (Continue_to (origin_id_of_string "oc"))))

(* ================================================================== *)
(*  S. Role meaning                                                     *)
(* ================================================================== *)

let test_role_meaning () =
  let mk role_id_str fulfillment_str =
    mk_program
      ~input_facts:[ mk_eval_fact "fx" "h" String_type ]
      ~entry_origin:(Some (origin_id_of_string "oa"))
      ~origin_sites:[
        mk_anchor_origin "oa" "ev" [];
        mk_action_origin "ob" "cap.x" "sha256:d1"
          [ mk_lit_input "x" (String_value "v") ]
          [ mk_origin_fact "fy" "ob" ]
      ]
      ~roles:[{
        role_id = role_id_of_string role_id_str;
        scope = Program_scope;
        fact_contract = Role_fact_contract [ fact_id_of_string "fy" ];
        eligible_fulfillment = role_fulfillment_of_string fulfillment_str }]
      ~capability_contracts:[ mk_cap_contract "cap.x" "sha256:d1" ]
      ()
  in
  assert (digest_of (mk "role-A" "f1") = digest_of (mk "role-B" "f1"));
  assert (digest_of (mk "role-A" "f1") <> digest_of (mk "role-C" "f2"))

(* ================================================================== *)
(*  T. Item Template meaning                                            *)
(* ================================================================== *)

let test_item_template_meaning () =
  let mk tid body_role_id_str =
    mk_program
      ~input_facts:[ mk_eval_fact "fx" "h" String_type ]
      ~entry_origin:(Some (origin_id_of_string "ent"))
      ~origin_sites:[
        mk_anchor_origin "ent" "ev" [];
        mk_action_origin "act" "cap.x" "sha256:d1"
          [ mk_lit_input "x" (String_value "v") ] []
      ]
      ~item_templates:[{
        item_template_id = item_template_id_of_string tid;
        origin_sites = [ mk_anchor_origin "it_o" "item_ev" [] ];
        branches = [];
        roles = [{ role_id = role_id_of_string body_role_id_str;
                   scope = Item_template_scope (item_template_id_of_string tid);
                   fact_contract = Role_fact_contract [];
                   eligible_fulfillment = role_fulfillment_of_string "f" }];
        objective = Required_role (role_id_of_string body_role_id_str) }]
      ~capability_contracts:[ mk_cap_contract "cap.x" "sha256:d1" ]
      ()
  in
  assert (digest_of (mk "it-A" "R1") = digest_of (mk "it-B" "R1"));
  let p_same = mk "it-X" "RX" in
  (* Different semantic template structure: change role fulfillment *)
  let p_diff = mk_program
    ~input_facts:[ mk_eval_fact "fx" "h" String_type ]
    ~entry_origin:(Some (origin_id_of_string "ent"))
    ~origin_sites:[
      mk_anchor_origin "ent" "ev" [];
      mk_action_origin "act" "cap.x" "sha256:d1"
        [ mk_lit_input "x" (String_value "v") ] []
    ]
    ~item_templates:[{
      item_template_id = item_template_id_of_string "it-D";
      origin_sites = [ mk_anchor_origin "it_o" "item_ev" [] ];
      branches = [];
      roles = [{ role_id = role_id_of_string "RD";
                 scope = Item_template_scope (item_template_id_of_string "it-D");
                 fact_contract = Role_fact_contract [];
                 eligible_fulfillment = role_fulfillment_of_string "different_fulfillment" }];
      objective = Required_role (role_id_of_string "RD") }]
    ~capability_contracts:[ mk_cap_contract "cap.x" "sha256:d1" ]
    ()
  in
  assert (digest_of p_same <> digest_of p_diff)

(* ================================================================== *)
(*  U. Batch meaning                                                    *)
(* ================================================================== *)

let test_batch_meaning () =
  let mk prov policy obj =
    let batch = Batch_site {
      batch_id = batch_id_of_string "BAT1";
      collection_provenance = batch_collection_provenance_of_string prov;
      item_template_id = item_template_id_of_string "IT1";
      traversal_policy = batch_traversal_policy_of_string policy;
      composite_objective = batch_objective_of_string obj;
      aggregate_facts = [] }
    in
    mk_program
      ~input_facts:[ mk_eval_fact "fx" "h" String_type ]
      ~entry_origin:(Some (origin_id_of_string "ent"))
      ~origin_sites:[ mk_anchor_origin "ent" "ev" []; batch ]
      ~item_templates:[{
        item_template_id = item_template_id_of_string "IT1";
        origin_sites = []; branches = [];
        roles = [{
          role_id = role_id_of_string "R1";
          scope = Item_template_scope (item_template_id_of_string "IT1");
          fact_contract = Role_fact_contract [];
          eligible_fulfillment = role_fulfillment_of_string "f" }];
        objective = Required_role (role_id_of_string "R1") }]
      ~capability_contracts:[]
      ()
  in
  assert (digest_of (mk "prov1" "pol1" "obj1") <> digest_of (mk "prov2" "pol1" "obj1"))

(* ================================================================== *)
(*  V. CoreVersion changes digest                                       *)
(* ================================================================== *)

let test_core_version_changes_digest () =
  let mk ver =
    mk_program ~core_version:(core_version_of_string ver)
      ~input_facts:[ mk_eval_fact "F1" "k" String_type ]
      ~entry_origin:(Some (origin_id_of_string "oa"))
      ~origin_sites:[
        mk_anchor_origin "oa" "ev" [];
        mk_action_origin "ob" "cap.x" "sha256:d1"
          [ mk_lit_input "x" (String_value "v") ] []
      ]
      ~capability_contracts:[ mk_cap_contract "cap.x" "sha256:d1" ]
      ()
  in
  assert (digest_of (mk "0.1.0") <> digest_of (mk "0.2.0"))

(* ================================================================== *)
(*  W. Multiplicity                                                     *)
(* ================================================================== *)

let test_multiplicity () =
  let p_one = mk_program
    ~input_facts:[ mk_eval_fact "F1" "k" String_type ]
    ~entry_origin:(Some (origin_id_of_string "ent"))
    ~origin_sites:[
      mk_anchor_origin "ent" "ev" [];
      mk_action_origin "only" "cap.x" "sha256:d1"
        [ mk_lit_input "x" (String_value "one") ] []
    ]
    ~capability_contracts:[ mk_cap_contract "cap.x" "sha256:d1" ]
    ()
  in
  let p_two = mk_program
    ~input_facts:[ mk_eval_fact "F1" "k" String_type ]
    ~entry_origin:(Some (origin_id_of_string "ent"))
    ~origin_sites:[
      mk_anchor_origin "ent" "ev" [];
      mk_action_origin "a1" "cap.x" "sha256:d1"
        [ mk_lit_input "x" (String_value "a") ] [];
      mk_action_origin "a2" "cap.x" "sha256:d1"
        [ mk_lit_input "x" (String_value "b") ] []
    ]
    ~capability_contracts:[ mk_cap_contract "cap.x" "sha256:d1" ]
    ()
  in
  assert (digest_of p_one <> digest_of p_two);
  let cp = canon_prog_of p_two in
  assert (List.length cp.origin_sites = 3)

(* ================================================================== *)
(*  X. Invalid Core returns Invalid_core                                *)
(* ================================================================== *)

let test_invalid_core () =
  let prog = mk_program
    ~origin_sites:[
      mk_action_origin "oa" "cap.x" "sha256:d1"
        [ mk_lit_input "x" (String_value "v") ] []
    ]
    ~capability_contracts:[ mk_cap_contract "cap.x" "sha256:d1" ]
    ()
  in
  match canonicalize prog with
  | Error (Invalid_core _) -> ()
  | _ -> failwith "expected Invalid_core"

(* ================================================================== *)
(*  Y. Raw-ID inversion trap                                            *)
(* ================================================================== *)

let test_raw_id_inversion_trap () =
  let mk origin_ids =
    let ids = List.map (fun s -> origin_id_of_string s) origin_ids in
    let anchor_a = mk_anchor_origin (List.hd origin_ids) "ev" [] in
    let actions = List.mapi (fun i id_s ->
      mk_action_origin id_s "cap.x" "sha256:d1"
        [ mk_lit_input "x" (String_value (string_of_int i)) ] [])
      (List.tl origin_ids)
    in
    mk_program
      ~input_facts:[ mk_eval_fact "F1" "k" String_type ]
      ~entry_origin:(Some (List.hd ids))
      ~origin_sites: (anchor_a :: actions)
      ~capability_contracts:[ mk_cap_contract "cap.x" "sha256:d1" ]
      ()
  in
  let d_asc = digest_of (mk (["anchor"] @ List.init 5 (fun i -> "act_" ^ string_of_int i))) in
  let d_desc = digest_of (mk (["anchor"] @ List.init 5 (fun i -> "z_act_" ^ string_of_int (4 - i)))) in
  assert (d_asc = d_desc)

(* ================================================================== *)
(*  Frozen canonical-byte fixture                                       *)
(* ================================================================== *)

let test_canonical_byte_fixture () =
  let prog = mk_program
    ~input_facts:[ mk_eval_fact "F1" "host.key.alpha" String_type ]
    ~entry_origin:(Some (origin_id_of_string "O_anchor"))
    ~origin_sites:[
      mk_anchor_origin "O_anchor" "event.ping" [];
      mk_action_origin "O_action" "cap.ping" "sha256:p1"
        [ mk_lit_input "payload" (String_value "hello") ] []
    ]
    ~capability_contracts:[ mk_cap_contract "cap.ping" "sha256:p1" ]
    ()
  in
  let bytes = bytes_of prog in
  let prefix_len = String.length "TETHERS_CORE_CANON_V1" in
  assert (String.sub bytes 0 prefix_len = "TETHERS_CORE_CANON_V1");
  assert (bytes.[prefix_len] = '\x00');
  assert (String.length bytes > prefix_len + 1)

(* ================================================================== *)
(*  Frozen SHA-256 ProgramDigest fixture                                *)
(* ================================================================== *)

let test_program_digest_fixture () =
  let prog = mk_program
    ~input_facts:[ mk_eval_fact "Fdata" "key.data" Integer_type ]
    ~entry_origin:(Some (origin_id_of_string "Oa"))
    ~origin_sites:[
      mk_anchor_origin "Oa" "system.start" [];
      mk_action_origin "Ob" "cap.log" "sha256:log1"
        [ mk_lit_input "message" (String_value "ready") ] []
    ]
    ~capability_contracts:[ mk_cap_contract "cap.log" "sha256:log1" ]
    ()
  in
  let d = digest_of prog in
  let expected = "sha256:8f622fd5e9379727216b277d81808f2f1037c6510de46968408b171f733796f5" in
  assert (d = expected)

(* ================================================================== *)
(*  Canonical prefix in bytes test                                      *)
(* ================================================================== *)

let test_canonical_prefix_in_bytes () =
  let prog = mk_program
    ~input_facts:[ mk_eval_fact "f" "h" String_type ]
    ~entry_origin:(Some (origin_id_of_string "o"))
    ~origin_sites:[
      mk_anchor_origin "o" "e" [];
      mk_action_origin "a" "c" "d" [ mk_lit_input "x" (String_value "y") ] []
    ]
    ~capability_contracts:[ mk_cap_contract "c" "d" ]
    ()
  in
  let bytes = bytes_of prog in
  let prefix = "TETHERS_CORE_CANON_V1\x00" in
  let prefix_len = String.length prefix in
  assert (String.sub bytes 0 prefix_len = prefix);
  assert (String.length bytes > prefix_len)

(* ================================================================== *)
(*  Run all tests                                                       *)
(* ================================================================== *)

let test name f =
  try f () with e ->
    Printf.eprintf "FAIL %s: %s\n%!" name (Printexc.to_string e);
    raise e

let () =
  test "A" test_baseline;
  test "B" test_determinism;
  test "C" test_origin_id_independence;
  test "D" test_all_id_independence;
  test "E" test_storage_order_independence;
  test "F" test_input_reordering;
  test "G" test_together_ordering;
  test "H" test_true_symmetry;
  test "I" test_branch_ordering;
  test "J" test_neutral_descriptions;
  test "K" test_program_id_neutral;
  test "L" test_literal_changes_digest;
  test "M" test_capability_id_changes_digest;
  test "N" test_contract_digest_changes_digest;
  test "O" test_anchor_meaning;
  test "P" test_guard_meaning;
  test "Q" test_control_flow_changes_digest;
  test "R" test_branch_meaning_changes;
  test "S" test_role_meaning;
  test "T" test_item_template_meaning;
  test "U" test_batch_meaning;
  test "V" test_core_version_changes_digest;
  test "W" test_multiplicity;
  test "X" test_invalid_core;
  test "Y" test_raw_id_inversion_trap;
  test "canonical_byte_fixture" test_canonical_byte_fixture;
  test "prefix" test_canonical_prefix_in_bytes;
  test "digest_fixture" test_program_digest_fixture
