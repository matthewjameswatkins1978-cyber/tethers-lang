open Tethers_core
open Tethers_core_canonical

(* ================================================================== *)
(*  Test helpers                                                        *)
(* ================================================================== *)

let assert_ok = function Ok x -> x | Error (Invalid_core _) -> failwith "expected Ok" | Error Refinement_exceeded -> failwith "refinement exceeded"

let digest_of p =
  match canonicalize p with
  | Ok c -> string_of_program_digest (program_digest c)
  | Error (Invalid_core _) -> failwith "expected Ok"
  | Error Refinement_exceeded -> failwith "refinement exceeded"

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
(*  CORE-4A REGRESSION TESTS                                            *)
(* ================================================================== *)

(* 1. Rename GroupIds across two Together groups with reversed lexical order *)
let test_group_id_independence () =
  let mk g1 g2 =
    mk_program
      ~input_facts:[ mk_eval_fact "fx" "hk" String_type ]
      ~entry_origin:(Some (origin_id_of_string "ent"))
      ~origin_sites:[
        mk_anchor_origin "ent" "ev" [];
        mk_action_origin "A1" "cap.x" "sha256:d1" [ mk_lit_input "x" (String_value "a1") ] [];
        mk_action_origin "A2" "cap.x" "sha256:d1" [ mk_lit_input "x" (String_value "a2") ] [];
        mk_action_origin "B1" "cap.x" "sha256:d1" [ mk_lit_input "x" (String_value "b1") ] [];
        mk_action_origin "B2" "cap.x" "sha256:d1" [ mk_lit_input "x" (String_value "b2") ] [];
        Together_origin { together_origin_id = origin_id_of_string "TG1";
          group_id = group_id_of_string g1;
          member_origin_ids = [origin_id_of_string "A1"; origin_id_of_string "A2"];
          objective = All_members_succeed };
        Together_origin { together_origin_id = origin_id_of_string "TG2";
          group_id = group_id_of_string g2;
          member_origin_ids = [origin_id_of_string "B1"; origin_id_of_string "B2"];
          objective = All_members_succeed }
      ]
      ~capability_contracts:[ mk_cap_contract "cap.x" "sha256:d1" ]
      ()
  in
  assert (digest_of (mk "zzzz" "aaaa") = digest_of (mk "aaaa" "zzzz"))

(* 2. Rename BatchId *)
let test_batch_id_independence () =
  let mk bid_str =
    mk_program
      ~input_facts:[ mk_eval_fact "fx" "h" String_type ]
      ~entry_origin:(Some (origin_id_of_string "ent"))
      ~origin_sites:[
        mk_anchor_origin "ent" "ev" [];
        Batch_site { batch_id = batch_id_of_string bid_str;
          collection_provenance = batch_collection_provenance_of_string "prov1";
          item_template_id = item_template_id_of_string "IT1";
          traversal_policy = batch_traversal_policy_of_string "pol1";
          composite_objective = batch_objective_of_string "obj1";
          aggregate_facts = [] }
      ]
      ~item_templates:[{
        item_template_id = item_template_id_of_string "IT1";
        origin_sites = []; branches = [];
        roles = [{ role_id = role_id_of_string "R1";
                   scope = Item_template_scope (item_template_id_of_string "IT1");
                   fact_contract = Role_fact_contract [];
                   eligible_fulfillment = role_fulfillment_of_string "f" }];
        objective = Required_role (role_id_of_string "R1") }]
      ~capability_contracts:[]
      ()
  in
  assert (digest_of (mk "zzz_batch_99") = digest_of (mk "aaa_batch_01"))

(* 3. Rename Batch aggregate FactId consistently *)
let test_batch_fact_id_independence () =
  let mk fid_str =
    mk_program
      ~input_facts:[ mk_eval_fact "fx" "h" String_type ]
      ~entry_origin:(Some (origin_id_of_string "ent"))
      ~origin_sites:[
        mk_anchor_origin "ent" "ev" [];
        Batch_site { batch_id = batch_id_of_string "B1";
          collection_provenance = batch_collection_provenance_of_string "prov1";
          item_template_id = item_template_id_of_string "IT1";
          traversal_policy = batch_traversal_policy_of_string "pol1";
          composite_objective = batch_objective_of_string "obj1";
          aggregate_facts = [{ fact_id = fact_id_of_string fid_str;
            schema_description = "x";
            provenance = Evaluation_input (host_snapshot_key_of_string "k", String_type) }] }
      ]
      ~item_templates:[{
        item_template_id = item_template_id_of_string "IT1";
        origin_sites = []; branches = [];
        roles = [{ role_id = role_id_of_string "R1";
                   scope = Item_template_scope (item_template_id_of_string "IT1");
                   fact_contract = Role_fact_contract [];
                   eligible_fulfillment = role_fulfillment_of_string "f" }];
        objective = Required_role (role_id_of_string "R1") }]
      ~capability_contracts:[]
      ()
  in
  assert (digest_of (mk "zzzzz") = digest_of (mk "aaaaa"))

(* 4. Reverse two Batch sites *)
let test_batch_order_independence () =
  let mk (bids : string list) =
    let make_bat bid =
      Batch_site { batch_id = batch_id_of_string bid;
        collection_provenance = batch_collection_provenance_of_string "p1";
        item_template_id = item_template_id_of_string "IT1";
        traversal_policy = batch_traversal_policy_of_string "tp1";
        composite_objective = batch_objective_of_string "o1";
        aggregate_facts = [] }
    in
    mk_program
      ~input_facts:[ mk_eval_fact "fx" "h" String_type ]
      ~entry_origin:(Some (origin_id_of_string "ent"))
      ~origin_sites:(
        mk_anchor_origin "ent" "ev" [] ::
        List.map make_bat bids
      )
      ~item_templates:[{
        item_template_id = item_template_id_of_string "IT1";
        origin_sites = []; branches = [];
        roles = [{ role_id = role_id_of_string "R1";
                   scope = Item_template_scope (item_template_id_of_string "IT1");
                   fact_contract = Role_fact_contract [];
                   eligible_fulfillment = role_fulfillment_of_string "f" }];
        objective = Required_role (role_id_of_string "R1") }]
      ~capability_contracts:[]
      ()
  in
  assert (digest_of (mk ["B_zzz"; "B_xxx"]) = digest_of (mk ["B_xxx"; "B_zzz"]))

(* 5. Rename Anchor_value referenced OriginId *)
let test_anchor_value_origin_independence () =
  let mk anchor_oid_str =
    mk_program
      ~input_facts:[ mk_eval_fact "fx" "h" String_type ]
      ~entry_origin:(Some (origin_id_of_string "ent"))
      ~origin_sites:[
        mk_anchor_origin "ent" "ev" [];
        mk_anchor_origin anchor_oid_str "ev2" [];
        mk_action_origin "act" "cap.x" "sha256:d1"
          [ { input_name = capability_input_name_of_string "x";
              binding = Anchor_value (origin_id_of_string anchor_oid_str, ["a"; "b"]) } ] []
      ]
      ~capability_contracts:[ mk_cap_contract "cap.x" "sha256:d1" ]
      ()
  in
  assert (digest_of (mk "ref_777") = digest_of (mk "ref_111"))

(* 6. Rename Fact_from_origin referenced FactId *)
let test_fo_fact_id_independence () =
  let mk fid_str =
    mk_program
      ~input_facts:[ mk_eval_fact "fx" "h" String_type ]
      ~entry_origin:(Some (origin_id_of_string "ent"))
      ~origin_sites:[
        mk_anchor_origin "ent" "ev" [];
        mk_action_origin "src" "cap.x" "sha256:d1"
          [ mk_lit_input "x" (String_value "v") ]
          [ { fact_id = fact_id_of_string fid_str;
              schema_description = "d";
              provenance = Origin_provenance (origin_id_of_string "src") } ];
        mk_action_origin "cons" "cap.x" "sha256:d1"
          [ { input_name = capability_input_name_of_string "y";
              binding = Fact_from_origin (fact_id_of_string fid_str,
                                          origin_id_of_string "src") } ] []
      ]
      ~capability_contracts:[ mk_cap_contract "cap.x" "sha256:d1" ]
      ()
  in
  assert (digest_of (mk "zz_fid") = digest_of (mk "aa_fid"))

(* 7. Rename Fact_through_role referenced FactId *)
let test_ft_fact_id_independence () =
  let mk fid_str =
    mk_program
      ~input_facts:[ mk_eval_fact fid_str "h" String_type; mk_eval_fact "fx" "h" String_type ]
      ~entry_origin:(Some (origin_id_of_string "ent"))
      ~origin_sites:[
        mk_anchor_origin "ent" "ev" [];
        mk_action_origin "act" "cap.x" "sha256:d1"
          [ { input_name = capability_input_name_of_string "y";
              binding = Fact_through_role (fact_id_of_string fid_str,
                                           role_id_of_string "rl") } ] []
      ]
      ~roles:[{ role_id = role_id_of_string "rl";
                scope = Program_scope;
                fact_contract = Role_fact_contract [ fact_id_of_string fid_str ];
                eligible_fulfillment = role_fulfillment_of_string "f" }]
      ~capability_contracts:[ mk_cap_contract "cap.x" "sha256:d1" ]
      ()
  in
  assert (digest_of (mk "fid_z") = digest_of (mk "fid_a"))

(* 8. Rename Branch subject OriginId *)
let test_branch_subject_independence () =
  let mk subj_id_str =
    mk_program
      ~input_facts:[ mk_eval_fact "fx" "h" String_type ]
      ~entry_origin:(Some (origin_id_of_string subj_id_str))
      ~origin_sites:[
        mk_anchor_origin subj_id_str "ev" [];
        mk_action_origin "nx" "cap.x" "sha256:d1" [ mk_lit_input "x" (String_value "v") ] []
      ]
      ~branches:[{
        branch_id = branch_id_of_string "B1";
        branch_subject = origin_id_of_string subj_id_str;
        outcome_branches = [(Success, Continue_to (origin_id_of_string "nx"))] }]
      ~capability_contracts:[ mk_cap_contract "cap.x" "sha256:d1" ]
      ()
  in
  assert (digest_of (mk "subj_z") = digest_of (mk "subj_a"))

(* 9. Two Actions with swapped inputs, storage reversal *)
let test_input_storage_reversal () =
  let mk ordered =
    mk_program
      ~input_facts:[ mk_eval_fact "f" "h" String_type ]
      ~entry_origin:(Some (origin_id_of_string "ent"))
      ~origin_sites:(
        mk_anchor_origin "ent" "ev" [] :: ordered
      )
      ~capability_contracts:[ mk_cap_contract "cap.x" "sha256:d1" ]
      ()
  in
  let p1 = mk [
    mk_action_origin "A" "cap.x" "sha256:d1"
      [ mk_lit_input "x" (Integer_value 1); mk_lit_input "y" (Integer_value 2) ] [];
    mk_action_origin "B" "cap.x" "sha256:d1"
      [ mk_lit_input "x" (Integer_value 2); mk_lit_input "y" (Integer_value 1) ] []
  ] in
  let p2 = mk [
    mk_action_origin "B" "cap.x" "sha256:d1"
      [ mk_lit_input "x" (Integer_value 2); mk_lit_input "y" (Integer_value 1) ] [];
    mk_action_origin "A" "cap.x" "sha256:d1"
      [ mk_lit_input "x" (Integer_value 1); mk_lit_input "y" (Integer_value 2) ] []
  ] in
  assert (digest_of p1 = digest_of p2)

(* 10. Two identical Actions in success chain, reversed storage *)
let test_chain_storage_reversal () =
  let mk ordered =
    mk_program
      ~input_facts:[ mk_eval_fact "f" "h" String_type ]
      ~entry_origin:(Some (origin_id_of_string "ent"))
      ~success_continuations:[
        mk_success_cont "A" (Origin_target (origin_id_of_string "B"))
      ]
      ~origin_sites:(
        mk_anchor_origin "ent" "ev" [] :: ordered
      )
      ~capability_contracts:[ mk_cap_contract "cap.x" "sha256:d1" ]
      ()
  in
  let p1 = mk [
    mk_action_origin "A" "cap.x" "sha256:d1" [ mk_lit_input "x" (String_value "v") ] [];
    mk_action_origin "B" "cap.x" "sha256:d1" [ mk_lit_input "x" (String_value "w") ] []
  ] in
  let p2 = mk [
    mk_action_origin "B" "cap.x" "sha256:d1" [ mk_lit_input "x" (String_value "w") ] [];
    mk_action_origin "A" "cap.x" "sha256:d1" [ mk_lit_input "x" (String_value "v") ] []
  ] in
  assert (digest_of p1 = digest_of p2)

(* 11. Entry Origin distinguishes, storage reversal *)
let test_entry_origin_distinguishes () =
  let mk ordered =
    mk_program
      ~input_facts:[ mk_eval_fact "f" "h" String_type ]
      ~entry_origin:(Some (origin_id_of_string "ent"))
      ~origin_sites: ordered
      ~capability_contracts:[ mk_cap_contract "cap.x" "sha256:d1" ]
      ()
  in
  let p1 = mk [
    mk_anchor_origin "ent" "ev" [];
    mk_action_origin "A" "cap.x" "sha256:d1" [ mk_lit_input "x" (String_value "a") ] [];
    mk_action_origin "B" "cap.x" "sha256:d1" [ mk_lit_input "x" (String_value "b") ] []
  ] in
  let p2 = mk [
    mk_anchor_origin "ent" "ev" [];
    mk_action_origin "B" "cap.x" "sha256:d1" [ mk_lit_input "x" (String_value "b") ] [];
    mk_action_origin "A" "cap.x" "sha256:d1" [ mk_lit_input "x" (String_value "a") ] []
  ] in
  assert (digest_of p1 = digest_of p2)

(* 12. Two Guards reversed storage *)
let test_guard_storage_reversal () =
  let p1 = mk_program
    ~input_facts:[ mk_eval_fact "f1" "h" String_type; mk_eval_fact "f2" "h" String_type ]
    ~entry_guards:[{ fact_id = fact_id_of_string "f1"; operator = Equals; expected = String_value "a" };
                   { fact_id = fact_id_of_string "f2"; operator = Contains; expected = String_value "b" }]
    ~entry_origin:(Some (origin_id_of_string "ent"))
    ~origin_sites:[
      mk_anchor_origin "ent" "ev" [];
      mk_action_origin "A" "cap.x" "sha256:d1" [ mk_lit_input "x" (String_value "v") ] []
    ]
    ~capability_contracts:[ mk_cap_contract "cap.x" "sha256:d1" ]
    ()
  in
  let p2 = mk_program
    ~input_facts:[ mk_eval_fact "f2" "h" String_type; mk_eval_fact "f1" "h" String_type ]
    ~entry_guards:[{ fact_id = fact_id_of_string "f2"; operator = Contains; expected = String_value "b" };
                   { fact_id = fact_id_of_string "f1"; operator = Equals; expected = String_value "a" }]
    ~entry_origin:(Some (origin_id_of_string "ent"))
    ~origin_sites:[
      mk_anchor_origin "ent" "ev" [];
      mk_action_origin "A" "cap.x" "sha256:d1" [ mk_lit_input "x" (String_value "v") ] []
    ]
    ~capability_contracts:[ mk_cap_contract "cap.x" "sha256:d1" ]
    ()
  in
  assert (digest_of p1 = digest_of p2)

(* 14. Two Item Templates each using local RoleId "R1" - template reordering preserves digest *)
let test_template_role_isolation () =
  let mk ordered_templates =
    mk_program
      ~input_facts:[ mk_eval_fact "fx" "h" String_type ]
      ~entry_origin:(Some (origin_id_of_string "ent"))
      ~origin_sites:[ mk_anchor_origin "ent" "ev" [] ]
      ~item_templates: ordered_templates
      ~capability_contracts:[]
      ()
  in
  let p1 = mk [
    { item_template_id = item_template_id_of_string "ita";
      origin_sites = []; branches = [];
      roles = [{ role_id = role_id_of_string "R1";
                 scope = Item_template_scope (item_template_id_of_string "ita");
                 fact_contract = Role_fact_contract [];
                 eligible_fulfillment = role_fulfillment_of_string "fa" }];
      objective = Required_role (role_id_of_string "R1") };
    { item_template_id = item_template_id_of_string "itb";
      origin_sites = []; branches = [];
      roles = [{ role_id = role_id_of_string "R1";
                 scope = Item_template_scope (item_template_id_of_string "itb");
                 fact_contract = Role_fact_contract [];
                 eligible_fulfillment = role_fulfillment_of_string "fb" }];
      objective = Required_role (role_id_of_string "R1") }
  ] in
  let p2 = mk [
    { item_template_id = item_template_id_of_string "itb";
      origin_sites = []; branches = [];
      roles = [{ role_id = role_id_of_string "R1";
                 scope = Item_template_scope (item_template_id_of_string "itb");
                 fact_contract = Role_fact_contract [];
                 eligible_fulfillment = role_fulfillment_of_string "fb" }];
      objective = Required_role (role_id_of_string "R1") };
    { item_template_id = item_template_id_of_string "ita";
      origin_sites = []; branches = [];
      roles = [{ role_id = role_id_of_string "R1";
                 scope = Item_template_scope (item_template_id_of_string "ita");
                 fact_contract = Role_fact_contract [];
                 eligible_fulfillment = role_fulfillment_of_string "fa" }];
      objective = Required_role (role_id_of_string "R1") }
  ] in
  assert (digest_of p1 = digest_of p2)

(* 15. Deep sequential structure >20 similar Origins *)
let test_deep_structure () =
  let mk ordered =
    mk_program
      ~input_facts:[ mk_eval_fact "f" "h" String_type ]
      ~entry_origin:(Some (origin_id_of_string "ent"))
      ~origin_sites:(
        mk_anchor_origin "ent" "ev" [] :: ordered
      )
      ~success_continuations:(
        List.init 29 (fun i ->
          mk_success_cont ("A" ^ string_of_int i)
            (Origin_target (origin_id_of_string ("A" ^ string_of_int (i + 1)))))
      )
      ~capability_contracts:[ mk_cap_contract "cap.x" "sha256:d1" ]
      ()
  in
  let actions ordered =
    List.map (fun oid ->
      mk_action_origin oid "cap.x" "sha256:d1"
        [ mk_lit_input "x" (String_value oid) ] []) ordered
  in
  let p1 = mk (actions (List.init 30 (fun i -> "A" ^ string_of_int i))) in
  let p2 = mk (actions (List.init 30 (fun i -> "A" ^ string_of_int (29 - i)))) in
  assert (digest_of p1 = digest_of p2)

(* ================================================================== *)
(*  REAL FROZEN BYTE FIXTURE                                            *)
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
  let expected = "TETHERS_CORE_CANON_V1\x005:0.1.01:2:F10:14:host.key.alpha0:0:1:2:O10:2:0:2:O110:event.ping0:1:2:O28:cap.ping9:sha256:p11:7:payload0:0:5:hello0:0:0:0:0:1:8:cap.ping9:sha256:p1" in
  assert (bytes = expected)

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
(*  CORE-4B REGRESSION TESTS                                            *)
(* ================================================================== *)

(* 1. Hash collision: "Aa" vs "BB" collide under base-31 hash,
   but are semantically different anchor event names *)
let test_hash_collision () =
  let mk ev =
    mk_program
      ~input_facts:[ mk_eval_fact "f" "h" String_type ]
      ~entry_origin:(Some (origin_id_of_string "oa"))
      ~origin_sites:[
        mk_anchor_origin "oa" ev [];
        mk_action_origin "ob" "cap.x" "sha256:d1"
          [ mk_lit_input "x" (String_value "v") ] []
      ]
      ~capability_contracts:[ mk_cap_contract "cap.x" "sha256:d1" ]
      ()
  in
  assert (digest_of (mk "Aa") <> digest_of (mk "BB"))

(* 2. Two semantically distinct Batch sites, reverse only storage order *)
let test_distinct_batch_reversal () =
  let mk bids =
    let make_bat bid prov =
      Batch_site { batch_id = batch_id_of_string bid;
        collection_provenance = batch_collection_provenance_of_string prov;
        item_template_id = item_template_id_of_string "IT1";
        traversal_policy = batch_traversal_policy_of_string "tp1";
        composite_objective = batch_objective_of_string "o1";
        aggregate_facts = [] }
    in
    mk_program
      ~input_facts:[ mk_eval_fact "f" "h" String_type ]
      ~entry_origin:(Some (origin_id_of_string "ent"))
      ~origin_sites:(
        mk_anchor_origin "ent" "ev" [] :: List.map (fun (bid, prov) -> make_bat bid prov) bids
      )
      ~item_templates:[{
        item_template_id = item_template_id_of_string "IT1";
        origin_sites = []; branches = [];
        roles = [{ role_id = role_id_of_string "R1";
                   scope = Item_template_scope (item_template_id_of_string "IT1");
                   fact_contract = Role_fact_contract [];
                   eligible_fulfillment = role_fulfillment_of_string "f" }];
        objective = Required_role (role_id_of_string "R1") }]
      ~capability_contracts:[]
      ()
  in
  assert (digest_of (mk [("B_z", "pA"); ("B_x", "pB")]) = digest_of (mk [("B_x", "pB"); ("B_z", "pA")]))

(* 3. Two Guards over same Fact with different operators, reverse order *)
let test_same_fact_guard_reversal () =
  let mk guards =
    mk_program
      ~input_facts:[ mk_eval_fact "f1" "h" String_type ]
      ~entry_guards:guards
      ~entry_origin:(Some (origin_id_of_string "ent"))
      ~origin_sites:[
        mk_anchor_origin "ent" "ev" [];
        mk_action_origin "A" "cap.x" "sha256:d1"
          [ mk_lit_input "x" (String_value "v") ] []
      ]
      ~capability_contracts:[ mk_cap_contract "cap.x" "sha256:d1" ]
      ()
  in
  let p1 = mk [{ fact_id = fact_id_of_string "f1"; operator = Equals; expected = String_value "a" };
                { fact_id = fact_id_of_string "f1"; operator = Contains; expected = String_value "b" }] in
  let p2 = mk [{ fact_id = fact_id_of_string "f1"; operator = Contains; expected = String_value "b" };
                { fact_id = fact_id_of_string "f1"; operator = Equals; expected = String_value "a" }] in
  assert (digest_of p1 = digest_of p2)

(* 4. Duplicate Action input names with distinct bindings — reverse order *)
let test_duplicate_input_names () =
  let p1 = mk_program
    ~input_facts:[ mk_eval_fact "f" "h" String_type ]
    ~entry_origin:(Some (origin_id_of_string "oa"))
    ~origin_sites:[
      mk_anchor_origin "oa" "ev" [];
      mk_action_origin "ob" "cap.x" "sha256:d1"
        [ { input_name = capability_input_name_of_string "x";
            binding = Literal_value (String_value "v1") };
          { input_name = capability_input_name_of_string "x";
            binding = Literal_value (String_value "v2") } ] []
    ]
    ~capability_contracts:[ mk_cap_contract "cap.x" "sha256:d1" ]
    ()
  in
  let p2 = mk_program
    ~input_facts:[ mk_eval_fact "f" "h" String_type ]
    ~entry_origin:(Some (origin_id_of_string "oa"))
    ~origin_sites:[
      mk_anchor_origin "oa" "ev" [];
      mk_action_origin "ob" "cap.x" "sha256:d1"
        [ { input_name = capability_input_name_of_string "x";
            binding = Literal_value (String_value "v2") };
          { input_name = capability_input_name_of_string "x";
            binding = Literal_value (String_value "v1") } ] []
    ]
    ~capability_contracts:[ mk_cap_contract "cap.x" "sha256:d1" ]
    ()
  in
  assert (digest_of p1 = digest_of p2)

(* 5. Scoped Roles: two Item Templates each with local RoleId "R1",
   rename template IDs and reorder *)
let test_scoped_role_preservation () =
  let mk ordered =
    mk_program
      ~input_facts:[ mk_eval_fact "fx" "h" String_type ]
      ~entry_origin:(Some (origin_id_of_string "ent"))
      ~origin_sites:[ mk_anchor_origin "ent" "ev" [] ]
      ~item_templates:ordered
      ~capability_contracts:[]
      ()
  in
  let t1 = { item_template_id = item_template_id_of_string "tpl_A";
    origin_sites = []; branches = [];
    roles = [{ role_id = role_id_of_string "R1";
               scope = Item_template_scope (item_template_id_of_string "tpl_A");
               fact_contract = Role_fact_contract [];
               eligible_fulfillment = role_fulfillment_of_string "fa" }];
    objective = Required_role (role_id_of_string "R1") }
  in
  let t2 = { item_template_id = item_template_id_of_string "tpl_B";
    origin_sites = []; branches = [];
    roles = [{ role_id = role_id_of_string "R1";
               scope = Item_template_scope (item_template_id_of_string "tpl_B");
               fact_contract = Role_fact_contract [];
               eligible_fulfillment = role_fulfillment_of_string "fb" }];
    objective = Required_role (role_id_of_string "R1") }
  in
  let d1 = digest_of (mk [t1; t2]) in
  let d2 = digest_of (mk [t2; t1]) in
  assert (d1 = d2);
  let cp = canon_prog_of (mk [t1; t2]) in
  assert (List.length cp.item_templates = 2);
  let rids = List.concat_map (fun (t : item_template) ->
    List.map (fun (r : role) -> string_of_role_id r.role_id) t.roles
  ) cp.item_templates in
  assert (List.length rids = 2);
  assert (List.mem "R1" rids);
  assert (List.mem "R2" rids)

(* 6. Truly identical success chain A→B→C→D:
   All actions have identical capability/contract/inputs/facts/constraints *)
let test_identical_success_chain () =
  let mk entry_oid ordered =
    let n = List.length ordered in
    let scs = List.init (n - 1) (fun i ->
      mk_success_cont (List.nth ordered i)
        (Origin_target (origin_id_of_string (List.nth ordered (i + 1))))
    ) in
    mk_program
      ~input_facts:[ mk_eval_fact "f" "h" String_type ]
      ~entry_origin:(Some (origin_id_of_string entry_oid))
      ~success_continuations:scs
      ~origin_sites:
        (List.map (fun oid ->
           mk_action_origin oid "cap.x" "sha256:d1"
             [ mk_lit_input "x" (String_value "same") ] []
         ) ordered)
      ~capability_contracts:[ mk_cap_contract "cap.x" "sha256:d1" ]
      ()
  in
  let d1 = digest_of (mk "A" ["A";"B";"C";"D"]) in
  let d2 = digest_of (mk "W" ["W";"X";"Y";"Z"]) in
  assert (d1 = d2)

(* 7. Deep identical chain >200-round safety horizon *)
let test_deep_identical_chain () =
  let n = 50 in
  let mk ordered =
    let scs = List.init (n - 1) (fun i ->
      mk_success_cont (List.nth ordered i)
        (Origin_target (origin_id_of_string (List.nth ordered (i + 1))))
    ) in
    mk_program
      ~input_facts:[ mk_eval_fact "f" "h" String_type ]
      ~entry_origin:(Some (origin_id_of_string (List.hd ordered)))
      ~success_continuations:scs
      ~origin_sites:
        (List.map (fun oid ->
           mk_action_origin oid "cap.x" "sha256:d1"
             [ mk_lit_input "x" (String_value "same") ] []
         ) ordered)
      ~capability_contracts:[ mk_cap_contract "cap.x" "sha256:d1" ]
      ()
  in
  let p_fwd = mk (List.init n (fun i -> "S" ^ string_of_int i)) in
  assert (match canonicalize p_fwd with Ok _ -> true | Error Refinement_exceeded -> false | _ -> true)

(* 8. Fact usage position: two structurally identical Facts
   distinguished only by which semantically distinct Action consumes them *)
let test_fact_usage_position () =
  let mk fid_a fid_b cons_a cons_b =
    mk_program
      ~input_facts:[ mk_eval_fact "f" "h" String_type ]
      ~entry_origin:(Some (origin_id_of_string "oa"))
      ~origin_sites:[
        mk_anchor_origin "oa" "ev" [];
        mk_action_origin "src" "cap.x" "sha256:d1"
          [ mk_lit_input "x" (String_value "v") ]
          [ { fact_id = fact_id_of_string fid_a;
              schema_description = "d";
              provenance = Origin_provenance (origin_id_of_string "src") };
            { fact_id = fact_id_of_string fid_b;
              schema_description = "d";
              provenance = Origin_provenance (origin_id_of_string "src") } ];
        mk_action_origin cons_a "cap.a" "sha256:a"
          [ { input_name = capability_input_name_of_string "y";
              binding = Fact_from_origin (fact_id_of_string fid_a, origin_id_of_string "src") } ] [];
        mk_action_origin cons_b "cap.b" "sha256:b"
          [ { input_name = capability_input_name_of_string "z";
              binding = Fact_from_origin (fact_id_of_string fid_b, origin_id_of_string "src") } ] []
      ]
      ~capability_contracts:[ mk_cap_contract "cap.x" "sha256:d1";
                              mk_cap_contract "cap.a" "sha256:a";
                              mk_cap_contract "cap.b" "sha256:b" ]
      ()
  in
  let p1 = mk "fa_alpha" "fb_beta" "cons_alpha" "cons_beta" in
  let p2 = mk "fb_beta" "fa_alpha" "cons_beta" "cons_alpha" in
  assert (digest_of p1 = digest_of p2)

(* ================================================================== *)
(*  CORE-4C REGRESSION TESTS                                            *)
(* ================================================================== *)

(* T1 — Role_proxy rename invariance *)
let test_role_proxy_rename_invariance () =
  let mk prog_role_id fact_id =
    let role = {
      role_id = role_id_of_string prog_role_id;
      scope = Program_scope;
      fact_contract = Role_fact_contract [];
      eligible_fulfillment = role_fulfillment_of_string "fulfill_alpha";
    } in
    let fact_rp = {
      fact_id = fact_id_of_string fact_id;
      schema_description = "desc";
      provenance = Role_proxy (role_id_of_string prog_role_id);
    } in
    mk_program
      ~input_facts:[ mk_eval_fact "F0" "hk0" String_type ]
      ~entry_origin:(Some (origin_id_of_string "O_anchor"))
      ~origin_sites:[
        mk_anchor_origin "O_anchor" "event.proxy" [];
        mk_action_origin "O_action" "cap.x" "sha256:d1"
          [ mk_lit_input "x" (String_value "v") ]
          [ fact_rp ]
      ]
      ~roles:[ role ]
      ~capability_contracts:[ mk_cap_contract "cap.x" "sha256:d1" ]
      ()
  in
  let p1 = mk "ROLE_ALPHA" "F_proxy" in
  let p2 = mk "ROLE_ZETA" "F_proxy2" in
  (* also rename fact id consistently: Fact id rename should not affect digest separately,
     but role rename is the core invariant. We rename fact id to ensure fact identity
     colour-compression covers the proxy fact itself. *)
  assert (bytes_of p1 = bytes_of p2);
  assert (digest_of p1 = digest_of p2)

(* T2 — Fact_through_role rename invariance *)
let test_fact_through_role_rename_invariance () =
  let mk role_id_str =
    let fid = "F_target" in
    let fact_eval = mk_eval_fact fid "hk1" String_type in
    let role = {
      role_id = role_id_of_string role_id_str;
      scope = Program_scope;
      fact_contract = Role_fact_contract [ fact_id_of_string fid ];
      eligible_fulfillment = role_fulfillment_of_string "fulfill_beta";
    } in
    mk_program
      ~input_facts:[ fact_eval ]
      ~entry_origin:(Some (origin_id_of_string "O_anchor"))
      ~origin_sites:[
        mk_anchor_origin "O_anchor" "event.ft" [];
        mk_action_origin "O_consumer" "cap.x" "sha256:d1"
          [ { input_name = capability_input_name_of_string "y";
              binding = Fact_through_role (fact_id_of_string fid, role_id_of_string role_id_str) } ]
          []
      ]
      ~roles:[ role ]
      ~capability_contracts:[ mk_cap_contract "cap.x" "sha256:d1" ]
      ()
  in
  let p_a = mk "ROLE_ALPHA" in
  let p_b = mk "ROLE_ZETA" in
  assert (bytes_of p_a = bytes_of p_b);
  assert (digest_of p_a = digest_of p_b)

(* T3 — Same local RoleId in two ItemTemplates, distinct semantics, no collapse *)
let test_same_local_role_in_two_templates () =
  let mk mk_templates =
    let input_facts = [
      mk_eval_fact "F_tplA" "hkA" String_type;
      mk_eval_fact "F_tplB" "hkB" String_type
    ] in
    let t1_fixed = {
      item_template_id = item_template_id_of_string "tplA";
      origin_sites = [
        mk_action_origin "O_tplA" "cap.t" "sha256:t"
          [ { input_name = capability_input_name_of_string "inp";
              binding = Fact_through_role (fact_id_of_string "F_tplA", role_id_of_string "R1") } ]
          []
      ];
      branches = [];
      roles = [{
        role_id = role_id_of_string "R1";
        scope = Item_template_scope (item_template_id_of_string "tplA");
        fact_contract = Role_fact_contract [ fact_id_of_string "F_tplA" ];
        eligible_fulfillment = role_fulfillment_of_string "fa" }];
      objective = Required_role (role_id_of_string "R1")
    } in
    let t2_fixed = {
      item_template_id = item_template_id_of_string "tplB";
      origin_sites = [
        mk_action_origin "O_tplB" "cap.t" "sha256:t"
          [ { input_name = capability_input_name_of_string "inp";
              binding = Fact_through_role (fact_id_of_string "F_tplB", role_id_of_string "R1") } ]
          []
      ];
      branches = [];
      roles = [{
        role_id = role_id_of_string "R1";
        scope = Item_template_scope (item_template_id_of_string "tplB");
        fact_contract = Role_fact_contract [ fact_id_of_string "F_tplB" ];
        eligible_fulfillment = role_fulfillment_of_string "fb" }];
      objective = Required_role (role_id_of_string "R1")
    } in
    let t1 = t1_fixed in
    let t2 = t2_fixed in
    let t1_fixed = { t1 with
      origin_sites = [
        mk_action_origin "O_tplA" "cap.t" "sha256:t"
          [ { input_name = capability_input_name_of_string "inp";
              binding = Fact_through_role (fact_id_of_string "F_tplA", role_id_of_string "R1") } ]
          []
      ];
      roles = [{ role_id = role_id_of_string "R1";
                 scope = Item_template_scope (item_template_id_of_string "tplA");
                 fact_contract = Role_fact_contract [ fact_id_of_string "F_tplA" ];
                 eligible_fulfillment = role_fulfillment_of_string "fa" }]
    } in
    let t2_fixed = { t2 with
      origin_sites = [
        mk_action_origin "O_tplB" "cap.t" "sha256:t"
          [ { input_name = capability_input_name_of_string "inp";
              binding = Fact_through_role (fact_id_of_string "F_tplB", role_id_of_string "R1") } ]
          []
      ];
      roles = [{ role_id = role_id_of_string "R1";
                 scope = Item_template_scope (item_template_id_of_string "tplB");
                 fact_contract = Role_fact_contract [ fact_id_of_string "F_tplB" ];
                 eligible_fulfillment = role_fulfillment_of_string "fb" }]
    } in
    mk_program
      ~input_facts
      ~entry_origin:(Some (origin_id_of_string "ent"))
      ~origin_sites:[ mk_anchor_origin "ent" "ev" [] ]
      ~item_templates:(mk_templates t1_fixed t2_fixed)
      ~capability_contracts:[ mk_cap_contract "cap.t" "sha256:t" ]
      ()
  in
  let p1 = mk (fun a b -> [a; b]) in
  let p2 = mk (fun a b -> [b; a]) in
  assert (digest_of p1 = digest_of p2);
  assert (bytes_of p1 = bytes_of p2);
  (* Inspect rewritten canonical Core: roles should be distinct canonical IDs *)
  let cp = canon_prog_of p1 in
  assert (List.length cp.item_templates = 2);
  let all_roles = List.concat_map (fun (t : item_template) -> t.roles) cp.item_templates in
  assert (List.length all_roles = 2);
  let rids = List.map (fun (r : role) -> string_of_role_id r.role_id) all_roles in
  assert (List.length (List.sort_uniq String.compare rids) = 2);
  (* No collapse: each template's action should reference its own canonical role *)
  let check_references (prog : program) =
    List.iter (fun (t : item_template) ->
      match t.origin_sites with
      | [Action_origin a] -> (
        match a.inputs with
        | [{ input_name = _; binding = Fact_through_role (_, rid) }] ->
            let expected_scope = Item_template_scope t.item_template_id in
            let matching_role = List.find_opt (fun (r : role) -> r.role_id = rid && r.scope = expected_scope) t.roles in
            assert (matching_role <> None)
        | _ -> failwith "expected one FT input"
      )
      | _ -> failwith "expected one action per template"
    ) prog.item_templates
  in
  check_references cp;
  (* Second program with renamed template IDs and local RoleIds, reversed storage *)
  let mk_renamed () =
    let input_facts = [
      mk_eval_fact "F_tplA" "hkA" String_type;
      mk_eval_fact "F_tplB" "hkB" String_type
    ] in
    let tA = {
      item_template_id = item_template_id_of_string "ZETA_TPL";
      origin_sites = [
        mk_action_origin "O_ZETA_A" "cap.t" "sha256:t"
          [ { input_name = capability_input_name_of_string "inp";
              binding = Fact_through_role (fact_id_of_string "F_tplA", role_id_of_string "R9") } ]
          []
      ];
      branches = [];
      roles = [{
        role_id = role_id_of_string "R9";
        scope = Item_template_scope (item_template_id_of_string "ZETA_TPL");
        fact_contract = Role_fact_contract [ fact_id_of_string "F_tplA" ];
        eligible_fulfillment = role_fulfillment_of_string "fa" }];
      objective = Required_role (role_id_of_string "R9")
    } in
    let tB = {
      item_template_id = item_template_id_of_string "ALPHA_TPL";
      origin_sites = [
        mk_action_origin "O_ALPHA_B" "cap.t" "sha256:t"
          [ { input_name = capability_input_name_of_string "inp";
              binding = Fact_through_role (fact_id_of_string "F_tplB", role_id_of_string "R9") } ]
          []
      ];
      branches = [];
      roles = [{
        role_id = role_id_of_string "R9";
        scope = Item_template_scope (item_template_id_of_string "ALPHA_TPL");
        fact_contract = Role_fact_contract [ fact_id_of_string "F_tplB" ];
        eligible_fulfillment = role_fulfillment_of_string "fb" }];
      objective = Required_role (role_id_of_string "R9")
    } in
    mk_program
      ~input_facts
      ~entry_origin:(Some (origin_id_of_string "ent"))
      ~origin_sites:[ mk_anchor_origin "ent" "ev" [] ]
      ~item_templates:[ tB; tA ]
      ~capability_contracts:[ mk_cap_contract "cap.t" "sha256:t" ]
      ()
  in
  let p_renamed = mk_renamed () in
  assert (digest_of p1 = digest_of p_renamed);
  assert (bytes_of p1 = bytes_of p_renamed)

(* T4 — Join predecessor rename invariance *)
let test_join_predecessor_rename_invariance () =
  let mk a_id b_id =
    let c_id = "O_C" in
    mk_program
      ~input_facts:[ mk_eval_fact "F0" "hk0" String_type ]
      ~entry_origin:(Some (origin_id_of_string "ent"))
      ~success_continuations:[
        mk_success_cont a_id (Origin_target (origin_id_of_string c_id));
        mk_success_cont b_id (Origin_target (origin_id_of_string c_id));
      ]
      ~origin_sites:[
        mk_anchor_origin "ent" "event.join" [];
        mk_action_origin a_id "cap.alpha" "sha256:alpha"
          [ mk_lit_input "x" (String_value "alpha_val") ] [];
        mk_action_origin b_id "cap.beta" "sha256:beta"
          [ mk_lit_input "x" (String_value "beta_val") ] [];
        mk_action_origin c_id "cap.gamma" "sha256:gamma"
          [ mk_lit_input "x" (String_value "gamma_val") ] [];
      ]
      ~capability_contracts:[
        mk_cap_contract "cap.alpha" "sha256:alpha";
        mk_cap_contract "cap.beta" "sha256:beta";
        mk_cap_contract "cap.gamma" "sha256:gamma";
      ]
      ()
  in
  (* Program1: A=O_A, B=O_Z ; Program2: swap lexical ids while preserving A/B semantics *)
  let p1 = mk "O_A" "O_Z" in
  let p2 = mk "O_Z" "O_A" in
  assert (digest_of p1 = digest_of p2);
  assert (bytes_of p1 = bytes_of p2)

(* T5 — Multi-branch rename invariance *)
let test_multi_branch_rename_invariance () =
  let mk b1_id b2_id =
    mk_program
      ~input_facts:[ mk_eval_fact "F0" "hk0" String_type ]
      ~entry_origin:(Some (origin_id_of_string "O_anchor"))
      ~origin_sites:[
        mk_anchor_origin "O_anchor" "event.branch" [];
        mk_action_origin "O_X" "cap.x" "sha256:x"
          [ mk_lit_input "v" (String_value "x") ] [];
        mk_action_origin "O_Y" "cap.y" "sha256:y"
          [ mk_lit_input "v" (String_value "y") ] [];
      ]
      ~branches:[
        { branch_id = branch_id_of_string b1_id;
          branch_subject = origin_id_of_string "O_anchor";
          outcome_branches = [ (Failure, Continue_to (origin_id_of_string "O_X")) ] };
        { branch_id = branch_id_of_string b2_id;
          branch_subject = origin_id_of_string "O_anchor";
          outcome_branches = [ (Uncertain, Continue_to (origin_id_of_string "O_Y")) ] };
      ]
      ~capability_contracts:[
        mk_cap_contract "cap.x" "sha256:x";
        mk_cap_contract "cap.y" "sha256:y";
      ]
      ()
  in
  let p1 = mk "B_zzz" "B_aaa" in
  let p2 = mk "B_aaa" "B_zzz" in
  assert (digest_of p1 = digest_of p2);
  assert (bytes_of p1 = bytes_of p2)

(* ================================================================== *)
(*  RUN ALL TESTS                                                       *)
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
  test "4A-1-group-id" test_group_id_independence;
  test "4A-2-batch-id" test_batch_id_independence;
  test "4A-3-batch-fact-id" test_batch_fact_id_independence;
  test "4A-4-batch-order" test_batch_order_independence;
  test "4A-5-av-origin" test_anchor_value_origin_independence;
  test "4A-6-fo-fact" test_fo_fact_id_independence;
  test "4A-7-ft-fact" test_ft_fact_id_independence;
  test "4A-8-branch-subj" test_branch_subject_independence;
  test "4A-9-input-rev" test_input_storage_reversal;
  test "4A-10-chain-rev" test_chain_storage_reversal;
  test "4A-11-entry-dist" test_entry_origin_distinguishes;
  test "4A-12-guard-rev" test_guard_storage_reversal;
  test "4A-14-template-iso" test_template_role_isolation;
  test "4A-15-deep" test_deep_structure;
  test "4B-1-hash-coll" test_hash_collision;
  test "4B-2-dist-batch" test_distinct_batch_reversal;
  test "4B-3-guard-rev" test_same_fact_guard_reversal;
  test "4B-4-dup-inputs" test_duplicate_input_names;
  test "4B-5-scope-role" test_scoped_role_preservation;
  test "4B-6-id-chain" test_identical_success_chain;
  test "4B-7-deep-chain" test_deep_identical_chain;
  test "4B-8-fact-usage" test_fact_usage_position;
  test "byte_fixture" test_canonical_byte_fixture;
  test "prefix" test_canonical_prefix_in_bytes;
  test "digest_fixture" test_program_digest_fixture;
  test "4C-T1-role-proxy" test_role_proxy_rename_invariance;
  test "4C-T2-ft-role" test_fact_through_role_rename_invariance;
  test "4C-T3-same-local-role" test_same_local_role_in_two_templates;
  test "4C-T4-join-predecessors" test_join_predecessor_rename_invariance;
  test "4C-T5-multi-branch" test_multi_branch_rename_invariance
