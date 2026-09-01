open Tethers_core

type family =
  | Origin
  | Fact
  | Branch
  | Batch
  | ItemTemplate
  | ScopedRole

type vertex_kind =
  | Anonymous of family
  | ProgramRoot
  | ProgramScope
  | ProgramComplete
  | BranchStop

type relation_kind =
  | Rel_root_entry_origin
  | Rel_success_next
  | Rel_success_complete
  | Rel_origin_declared_fact
  | Rel_batch_aggregate_fact
  | Rel_fact_provenance_origin
  | Rel_fact_provenance_role
  | Rel_entry_guard_fact
  | Rel_action_input_fact
  | Rel_action_input_origin
  | Rel_action_input_role
  | Rel_action_input_anchor
  | Rel_action_input_template
  | Rel_together_member
  | Rel_branch_subject
  | Rel_branch_target
  | Rel_branch_stop
  | Rel_role_contract_fact
  | Rel_role_scope_template
  | Rel_role_scope_program
  | Rel_template_origin
  | Rel_template_batch
  | Rel_template_branch
  | Rel_template_role
  | Rel_template_objective_role
  | Rel_program_input_fact
  | Rel_program_origin
  | Rel_program_batch
  | Rel_program_branch
  | Rel_program_role
  | Rel_program_template
  | Rel_fact_scope_owner
  | Rel_batch_template_context

type binding_kind =
  | Binding_fact_from_origin
  | Binding_fact_through_role
  | Binding_anchor_value
  | Binding_batch_item_context

type relation_discriminator =
  | Discriminator_none
  | Origin_anchor
  | Origin_action
  | Origin_together
  | Batch_site_aggregate
  | Provenance_origin
  | Provenance_role_proxy
  | Entry_guard
  | Action_binding of binding_kind
  | Together_member
  | Branch_outcome of terminal_outcome
  | Branch_continue_to
  | Branch_stop_target
  | Success_continuation
  | Success_program_complete
  | Role_contract
  | Role_program_scope
  | Role_item_template_scope
  | Template_membership
  | Template_batch_membership
  | Template_objective
  | Program_input
  | Program_origin_membership
  | Program_batch_membership
  | Program_branch_membership
  | Program_role_membership
  | Program_template_membership
  | Fact_program_scope
  | Fact_template_scope
  | Fact_origin_scope
  | Fact_batch_scope
  | Batch_template_context

type edge = {
  target : int;
  relation : relation_kind;
  discriminator : relation_discriminator;
  payload : string;
}

type vertex = {
  kind : vertex_kind;
  scalar : string;
}

type t = {
  vertices : vertex array;
  forward : edge array array;
  reverse : edge array array;
}

type owner_scope =
  | Program_owner
  | Template_owner of item_template_id

type origin_entry = {
  owner : owner_scope;
  site : origin_site;
  id : origin_id;
}

type batch_entry = {
  owner : owner_scope;
  site : batch_site;
  id : batch_id;
}

type branch_entry = {
  owner : owner_scope;
  value : branch;
}

type role_entry = {
  owner : owner_scope;
  value : role;
}

type fact_scope =
  | Fact_program_scope
  | Fact_template_scope of item_template_id

let stable_string s = string_of_int (String.length s) ^ ":" ^ String.escaped s

let stable_list f xs =
  String.concat ";" (List.map (fun x -> stable_string (f x)) xs)

let core_value_descriptor = function
  | String_value s -> "string:" ^ stable_string s
  | Integer_value i -> "integer:" ^ string_of_int i
  | Boolean_value b -> "boolean:" ^ string_of_bool b

let scalar_type_descriptor = function
  | String_type -> "string"
  | Integer_type -> "integer"
  | Boolean_type -> "boolean"

let string_of_outcome = function
  | Success -> "success"
  | Failure -> "failure"
  | Uncertain -> "uncertain"
  | Cancelled -> "cancelled"

let string_of_family = function
  | Origin -> "Origin"
  | Fact -> "Fact"
  | Branch -> "Branch"
  | Batch -> "Batch"
  | ItemTemplate -> "ItemTemplate"
  | ScopedRole -> "ScopedRole"

let string_of_kind = function
  | Anonymous f -> "anonymous:" ^ string_of_family f
  | ProgramRoot -> "sentinel:ProgramRoot"
  | ProgramScope -> "sentinel:ProgramScope"
  | ProgramComplete -> "terminal:ProgramComplete"
  | BranchStop -> "terminal:BranchStop"

let relation_rank = function
  | Rel_root_entry_origin -> 1
  | Rel_success_next -> 2
  | Rel_success_complete -> 3
  | Rel_origin_declared_fact -> 4
  | Rel_batch_aggregate_fact -> 5
  | Rel_fact_provenance_origin -> 6
  | Rel_fact_provenance_role -> 7
  | Rel_entry_guard_fact -> 8
  | Rel_action_input_fact -> 9
  | Rel_action_input_origin -> 10
  | Rel_action_input_role -> 11
  | Rel_action_input_anchor -> 12
  | Rel_action_input_template -> 13
  | Rel_together_member -> 14
  | Rel_branch_subject -> 15
  | Rel_branch_target -> 16
  | Rel_branch_stop -> 17
  | Rel_role_contract_fact -> 18
  | Rel_role_scope_template -> 19
  | Rel_role_scope_program -> 20
  | Rel_template_origin -> 21
  | Rel_template_batch -> 22
  | Rel_template_branch -> 23
  | Rel_template_role -> 24
  | Rel_template_objective_role -> 25
  | Rel_program_input_fact -> 26
  | Rel_program_origin -> 27
  | Rel_program_batch -> 28
  | Rel_program_branch -> 29
  | Rel_program_role -> 30
  | Rel_program_template -> 31
  | Rel_fact_scope_owner -> 32
  | Rel_batch_template_context -> 33

let string_of_relation r =
  let names = [
    "Rel_root_entry_origin"; "Rel_success_next"; "Rel_success_complete";
    "Rel_origin_declared_fact"; "Rel_batch_aggregate_fact";
    "Rel_fact_provenance_origin"; "Rel_fact_provenance_role";
    "Rel_entry_guard_fact"; "Rel_action_input_fact";
    "Rel_action_input_origin"; "Rel_action_input_role";
    "Rel_action_input_anchor"; "Rel_action_input_template";
    "Rel_together_member"; "Rel_branch_subject"; "Rel_branch_target";
    "Rel_branch_stop"; "Rel_role_contract_fact";
    "Rel_role_scope_template"; "Rel_role_scope_program";
    "Rel_template_origin"; "Rel_template_batch"; "Rel_template_branch";
    "Rel_template_role"; "Rel_template_objective_role";
    "Rel_program_input_fact"; "Rel_program_origin"; "Rel_program_batch";
    "Rel_program_branch"; "Rel_program_role"; "Rel_program_template";
    "Rel_fact_scope_owner"; "Rel_batch_template_context"
  ] in
  List.nth names (relation_rank r - 1)

let string_of_binding = function
  | Binding_fact_from_origin -> "Fact_from_origin"
  | Binding_fact_through_role -> "Fact_through_role"
  | Binding_anchor_value -> "Anchor_value"
  | Binding_batch_item_context -> "Batch_item_context"

let discriminator_rank = function
  | Discriminator_none -> 0
  | Origin_anchor -> 1
  | Origin_action -> 2
  | Origin_together -> 3
  | Batch_site_aggregate -> 4
  | Provenance_origin -> 5
  | Provenance_role_proxy -> 6
  | Entry_guard -> 7
  | Action_binding Binding_fact_from_origin -> 8
  | Action_binding Binding_fact_through_role -> 9
  | Action_binding Binding_anchor_value -> 10
  | Action_binding Binding_batch_item_context -> 11
  | Together_member -> 12
  | Branch_outcome Success -> 13
  | Branch_outcome Failure -> 14
  | Branch_outcome Uncertain -> 15
  | Branch_outcome Cancelled -> 16
  | Branch_continue_to -> 17
  | Branch_stop_target -> 18
  | Success_continuation -> 19
  | Success_program_complete -> 20
  | Role_contract -> 21
  | Role_program_scope -> 22
  | Role_item_template_scope -> 23
  | Template_membership -> 24
  | Template_batch_membership -> 25
  | Template_objective -> 26
  | Program_input -> 27
  | Program_origin_membership -> 28
  | Program_batch_membership -> 29
  | Program_branch_membership -> 30
  | Program_role_membership -> 31
  | Program_template_membership -> 32
  | Fact_program_scope -> 33
  | Fact_template_scope -> 34
  | Fact_origin_scope -> 35
  | Fact_batch_scope -> 36
  | Batch_template_context -> 37

let string_of_discriminator d =
  match d with
  | Discriminator_none -> "none"
  | Origin_anchor -> "origin:anchor"
  | Origin_action -> "origin:action"
  | Origin_together -> "origin:together"
  | Batch_site_aggregate -> "batch-site:aggregate"
  | Provenance_origin -> "provenance:origin"
  | Provenance_role_proxy -> "provenance:role-proxy"
  | Entry_guard -> "entry-guard"
  | Action_binding b -> "action-binding:" ^ string_of_binding b
  | Together_member -> "together-member"
  | Branch_outcome o -> "branch-outcome:" ^ string_of_outcome o
  | Branch_continue_to -> "branch:continue-to"
  | Branch_stop_target -> "branch:stop"
  | Success_continuation -> "success-continuation"
  | Success_program_complete -> "success:program-complete"
  | Role_contract -> "role:fact-contract"
  | Role_program_scope -> "role:program-scope"
  | Role_item_template_scope -> "role:item-template-scope"
  | Template_membership -> "template:origin-membership"
  | Template_batch_membership -> "template:batch-membership"
  | Template_objective -> "template:objective"
  | Program_input -> "program:input"
  | Program_origin_membership -> "program:origin-membership"
  | Program_batch_membership -> "program:batch-membership"
  | Program_branch_membership -> "program:branch-membership"
  | Program_role_membership -> "program:role-membership"
  | Program_template_membership -> "program:template-membership"
  | Fact_program_scope -> "fact:program-scope"
  | Fact_template_scope -> "fact:template-scope"
  | Fact_origin_scope -> "fact:origin-scope"
  | Fact_batch_scope -> "fact:batch-scope"
  | Batch_template_context -> "batch:template-context"

let scope_key = function
  | Program_scope -> "program"
  | Item_template_scope tid -> "template:" ^ string_of_item_template_id tid

let role_key scope rid = scope_key scope ^ "|role:" ^ string_of_role_id rid

let owner_scope_to_role_scope = function
  | Program_owner -> Program_scope
  | Template_owner tid -> Item_template_scope tid

let owner_scope_to_fact_scope = function
  | Program_owner -> Fact_program_scope
  | Template_owner tid -> Fact_template_scope tid

let origin_id_of_site = function
  | Anchor_origin a -> Some a.anchor_origin_id
  | Action_origin a -> Some a.action_origin_id
  | Together_origin t -> Some t.together_origin_id
  | Batch_site _ -> None

let origin_scalar = function
  | Anchor_origin a ->
      "origin:anchor:event=" ^ stable_string a.event_name
  | Action_origin a ->
      "origin:action:cap=" ^ stable_string (string_of_capability_id a.capability_id) ^
      ":digest=" ^ stable_string (string_of_capability_contract_digest a.contract_digest) ^
      ":constraints=" ^
      stable_list (function Deadline s -> "deadline:" ^ s) a.execution_constraints
  | Together_origin _ -> "origin:together:objective=all-members-succeed"
  | Batch_site _ -> "origin-site:batch-structural-variant"

let batch_scalar b =
  "batch:collection=" ^ stable_string (string_of_batch_collection_provenance b.collection_provenance) ^
  ":traversal=" ^ stable_string (string_of_batch_traversal_policy b.traversal_policy) ^
  ":objective=" ^ stable_string (string_of_batch_objective b.composite_objective)

let fact_scalar f =
  match f.provenance with
  | Evaluation_input (key, scalar_type) ->
      "fact:evaluation-input:key=" ^
      stable_string (string_of_host_snapshot_key key) ^
      ":type=" ^ scalar_type_descriptor scalar_type
  | Origin_provenance _ -> "fact:origin-provenance"
  | Role_proxy _ -> "fact:role-proxy"

let role_scalar r =
  "role:fulfillment=" ^
  stable_string (string_of_role_fulfillment r.eligible_fulfillment)

let guard_payload g =
  let operator = match g.operator with
    | Equals -> "equals"
    | Contains -> "contains"
    | Greater_than -> "greater-than"
    | Greater_than_or_equal -> "greater-than-or-equal"
  in
  "operator=" ^ operator ^ ":expected=" ^ core_value_descriptor g.expected

let input_payload name suffix =
  "input=" ^ stable_string (string_of_capability_input_name name) ^ suffix

let path_payload path = stable_list (fun x -> x) path

let role_scope_for_fact scope = function
  | Fact_program_scope -> scope
  | Fact_template_scope tid -> Item_template_scope tid

let build program =
  match Tethers_core_validator.validate program with
  | Error errors -> Error errors
  | Ok () ->
      let origins = ref [] in
      let batches = ref [] in
      let branches = ref [] in
      let roles = ref [] in
      let templates = ref program.item_templates in
      let origins_of (owner : owner_scope) (sites : origin_site list) =
        List.iter (fun site ->
          match site with
          | Batch_site b -> batches := ({ owner; site = b; id = b.batch_id } : batch_entry) :: !batches
          | Anchor_origin a -> origins := ({ owner; site; id = a.anchor_origin_id } : origin_entry) :: !origins
          | Action_origin a -> origins := ({ owner; site; id = a.action_origin_id } : origin_entry) :: !origins
          | Together_origin t -> origins := ({ owner; site; id = t.together_origin_id } : origin_entry) :: !origins
        ) sites
      in
      origins_of Program_owner program.origin_sites;
      List.iter (fun (t : item_template) ->
        origins_of (Template_owner t.item_template_id) t.origin_sites;
        List.iter (fun b ->
          branches := ({ owner = Template_owner t.item_template_id; value = b } : branch_entry) :: !branches
        ) t.branches;
        List.iter (fun r ->
          roles := ({ owner = Template_owner t.item_template_id; value = r } : role_entry) :: !roles
        ) t.roles
      ) program.item_templates;
      List.iter (fun b ->
        branches := ({ owner = Program_owner; value = b } : branch_entry) :: !branches
      ) program.branches;
      List.iter (fun r ->
        roles := ({ owner = Program_owner; value = r } : role_entry) :: !roles
      ) program.roles;
      let origins = List.rev !origins in
      let batches = List.rev !batches in
      let branches = List.rev !branches in
      let roles = List.rev !roles in
      let facts = ref [] in
      let fact_by_id = Hashtbl.create 32 in
      let fact_scope_by_id = Hashtbl.create 32 in
      let add_fact scope (f : fact) =
        let key = string_of_fact_id f.fact_id in
        if not (Hashtbl.mem fact_by_id key) then begin
          Hashtbl.add fact_by_id key f;
          facts := f :: !facts
        end;
        if not (Hashtbl.mem fact_scope_by_id key) then
          Hashtbl.add fact_scope_by_id key scope
      in
      List.iter (add_fact Fact_program_scope) program.input_facts;
      List.iter (fun (entry : origin_entry) ->
        let scope = owner_scope_to_fact_scope entry.owner in
        match entry.site with
        | Anchor_origin a -> List.iter (add_fact scope) a.declared_facts
        | Action_origin a -> List.iter (add_fact scope) a.declared_facts
        | Together_origin _ -> ()
        | Batch_site _ -> assert false
      ) origins;
      List.iter (fun (entry : batch_entry) ->
        let scope = owner_scope_to_fact_scope entry.owner in
        List.iter (add_fact scope) entry.site.aggregate_facts
      ) batches;
      let facts = List.rev !facts in
      let vertices = ref [
        { kind = ProgramRoot; scalar = "fixed" };
        { kind = ProgramScope; scalar = "fixed" };
        { kind = ProgramComplete; scalar = "fixed" };
        { kind = BranchStop; scalar = "fixed" }
      ] in
      let add_vertex kind scalar =
        let index = List.length !vertices in
        vertices := !vertices @ [{ kind; scalar }];
        index
      in
      let origin_index = Hashtbl.create 32 in
      let batch_index = Hashtbl.create 32 in
      let branch_index = Hashtbl.create 32 in
      let fact_index = Hashtbl.create 32 in
      let template_index = Hashtbl.create 16 in
      let role_index = Hashtbl.create 32 in
      List.iter (fun (entry : origin_entry) ->
        Hashtbl.add origin_index (string_of_origin_id entry.id)
          (add_vertex (Anonymous Origin) (origin_scalar entry.site))) origins;
      List.iter (fun (entry : batch_entry) ->
        Hashtbl.add batch_index (string_of_batch_id entry.id)
          (add_vertex (Anonymous Batch) (batch_scalar entry.site))) batches;
      List.iter (fun (entry : branch_entry) ->
        Hashtbl.add branch_index (string_of_branch_id entry.value.branch_id)
          (add_vertex (Anonymous Branch) "branch")) branches;
      List.iter (fun (f : fact) ->
        Hashtbl.add fact_index (string_of_fact_id f.fact_id)
          (add_vertex (Anonymous Fact) (fact_scalar f))) facts;
      List.iter (fun t ->
        Hashtbl.add template_index (string_of_item_template_id t.item_template_id)
          (add_vertex (Anonymous ItemTemplate) "item-template")) !templates;
      List.iter (fun (entry : role_entry) ->
        let scope = owner_scope_to_role_scope entry.owner in
        Hashtbl.add role_index (role_key scope entry.value.role_id)
          (add_vertex (Anonymous ScopedRole) (role_scalar entry.value))) roles;
      let vertex_array = Array.of_list !vertices in
      let forward_builders = Array.init (Array.length vertex_array) (fun _ -> ref []) in
      let reverse_builders = Array.init (Array.length vertex_array) (fun _ -> ref []) in
      let add_edge src dst relation discriminator payload =
        let edge = { target = dst; relation; discriminator; payload } in
        let reverse = { target = src; relation; discriminator; payload } in
        forward_builders.(src) := edge :: !(forward_builders.(src));
        reverse_builders.(dst) := reverse :: !(reverse_builders.(dst))
      in
      let find table key = Hashtbl.find table key in
      let find_origin oid = find origin_index (string_of_origin_id oid) in
      let find_batch bid = find batch_index (string_of_batch_id bid) in
      let find_branch bid = find branch_index (string_of_branch_id bid) in
      let find_fact fid = find fact_index (string_of_fact_id fid) in
      let find_template tid = find template_index (string_of_item_template_id tid) in
      let find_role scope rid = find role_index (role_key scope rid) in
      let root = 0 in
      let program_scope = 1 in
      let program_complete = 2 in
      let branch_stop = 3 in
      let add_fact_scope_edge (f : fact) =
        let source = find_fact f.fact_id in
        match Hashtbl.find fact_scope_by_id (string_of_fact_id f.fact_id) with
        | Fact_program_scope ->
            add_edge source program_scope Rel_fact_scope_owner Fact_program_scope ""
        | Fact_template_scope tid ->
            add_edge source (find_template tid) Rel_fact_scope_owner Fact_template_scope ""
      in
      List.iter add_fact_scope_edge facts;
      let add_declared_facts source discriminator (fs : fact list) =
        List.iter (fun (f : fact) ->
          add_edge source (find_fact f.fact_id) Rel_origin_declared_fact discriminator ""
        ) fs
      in
      let add_binding source origin_scope name binding =
        match binding with
        | Literal_value _ -> ()
        | Fact_from_origin (fid, oid) ->
            let d = Action_binding Binding_fact_from_origin in
            let payload = input_payload name "" in
            add_edge source (find_fact fid) Rel_action_input_fact d payload;
            add_edge source (find_origin oid) Rel_action_input_origin d payload
        | Fact_through_role (fid, rid) ->
            let d = Action_binding Binding_fact_through_role in
            let payload = input_payload name "" in
            add_edge source (find_fact fid) Rel_action_input_fact d payload;
            add_edge source (find_role origin_scope rid) Rel_action_input_role d payload
        | Anchor_value (oid, path) ->
            let d = Action_binding Binding_anchor_value in
            let payload = input_payload name (":path=" ^ path_payload path) in
            add_edge source (find_origin oid) Rel_action_input_anchor d payload
        | Batch_item_context tid ->
            let d = Action_binding Binding_batch_item_context in
            let payload = input_payload name "" in
            add_edge source (find_template tid) Rel_action_input_template d payload
      in
      List.iter (fun (entry : origin_entry) ->
        match entry.site with
        | Anchor_origin a ->
            let source = find_origin a.anchor_origin_id in
            add_declared_facts source Origin_anchor a.declared_facts
        | Action_origin a ->
            let source = find_origin a.action_origin_id in
            let scope = owner_scope_to_role_scope entry.owner in
            add_declared_facts source Origin_action a.declared_facts;
            List.iter (fun input ->
              add_binding source scope input.input_name input.binding
            ) a.inputs
        | Together_origin t ->
            let source = find_origin t.together_origin_id in
            List.iter (fun oid ->
              add_edge source (find_origin oid) Rel_together_member Together_member ""
            ) t.member_origin_ids
        | Batch_site _ ->
            assert false
      ) origins;
      List.iter (fun (entry : batch_entry) ->
        let source = find_batch entry.id in
        List.iter (fun (f : fact) ->
          add_edge source (find_fact f.fact_id) Rel_batch_aggregate_fact
            Batch_site_aggregate ""
        ) entry.site.aggregate_facts;
        add_edge source (find_template entry.site.item_template_id)
          Rel_batch_template_context Batch_template_context ""
      ) batches;
      List.iter (fun (entry : branch_entry) ->
        let source = find_branch entry.value.branch_id in
        add_edge source (find_origin entry.value.branch_subject)
          Rel_branch_subject Discriminator_none "";
        List.iter (fun (outcome, target) ->
          match target with
          | Continue_to oid ->
              add_edge source (find_origin oid) Rel_branch_target
                (Branch_outcome outcome) "continue-to"
          | Stop ->
              add_edge source branch_stop Rel_branch_stop
                (Branch_outcome outcome) "stop"
        ) entry.value.outcome_branches
      ) branches;
      let add_role_entry (entry : role_entry) =
        let scope = owner_scope_to_role_scope entry.owner in
        let source = find_role scope entry.value.role_id in
        begin match scope with
        | Program_scope ->
            add_edge source program_scope Rel_role_scope_program Role_program_scope ""
        | Item_template_scope tid ->
            add_edge source (find_template tid) Rel_role_scope_template
              Role_item_template_scope ""
        end;
        let (Role_fact_contract contract) = entry.value.fact_contract in
        List.iter (fun fid ->
          add_edge source (find_fact fid) Rel_role_contract_fact Role_contract ""
        ) contract
      in
      List.iter add_role_entry roles;
      let add_template_entry (t : item_template) =
        let source = find_template t.item_template_id in
        List.iter (fun site ->
          match site with
          | Batch_site b ->
              add_edge source (find_batch b.batch_id) Rel_template_batch
                Template_batch_membership ""
          | Anchor_origin a ->
              add_edge source (find_origin a.anchor_origin_id) Rel_template_origin
                Template_membership ""
          | Action_origin a ->
              add_edge source (find_origin a.action_origin_id) Rel_template_origin
                Template_membership ""
          | Together_origin x ->
              add_edge source (find_origin x.together_origin_id) Rel_template_origin
                Template_membership ""
        ) t.origin_sites;
        List.iter (fun b ->
          add_edge source (find_branch b.branch_id) Rel_template_branch
            Template_membership ""
        ) t.branches;
        List.iter (fun r ->
          add_edge source (find_role (Item_template_scope t.item_template_id) r.role_id)
            Rel_template_role Template_membership ""
        ) t.roles;
        match t.objective with
        | Required_role rid ->
            add_edge source (find_role (Item_template_scope t.item_template_id) rid)
              Rel_template_objective_role Template_objective ""
      in
      List.iter add_template_entry !templates;
      List.iter (fun (entry : origin_entry) ->
        match entry.owner with
        | Program_owner ->
            let source = match origin_id_of_site entry.site with
              | Some oid -> find_origin oid
              | None -> assert false
            in
            add_edge program_scope source Rel_program_origin Program_origin_membership ""
        | Template_owner _ -> ()
      ) origins;
      List.iter (fun (entry : batch_entry) ->
        match entry.owner with
        | Program_owner ->
            add_edge program_scope (find_batch entry.id)
              Rel_program_batch Program_batch_membership ""
        | Template_owner _ -> ()
      ) batches;
      List.iter (fun (entry : branch_entry) ->
        match entry.owner with
        | Program_owner ->
            add_edge program_scope (find_branch entry.value.branch_id)
              Rel_program_branch Program_branch_membership ""
        | Template_owner _ -> ()
      ) branches;
      List.iter (fun (entry : role_entry) ->
        match entry.owner with
        | Program_owner ->
            add_edge program_scope
              (find_role Program_scope entry.value.role_id)
              Rel_program_role Program_role_membership ""
        | Template_owner _ -> ()
      ) roles;
      List.iter (fun (t : item_template) ->
        add_edge program_scope (find_template t.item_template_id)
          Rel_program_template Program_template_membership ""
      ) !templates;
      List.iter (fun (f : fact) ->
        add_edge program_scope (find_fact f.fact_id) Rel_program_input_fact Program_input ""
      ) program.input_facts;
      List.iter (fun guard ->
        add_edge program_scope (find_fact guard.fact_id) Rel_entry_guard_fact
          Entry_guard (guard_payload guard)
      ) program.entry_guards;
      begin match program.entry_origin with
      | None -> ()
      | Some oid -> add_edge root (find_origin oid) Rel_root_entry_origin
          Success_continuation "entry"
      end;
      List.iter (fun continuation ->
        let source = find_origin continuation.from_origin in
        match continuation.target with
        | Origin_target oid ->
            add_edge source (find_origin oid) Rel_success_next
              Success_continuation ""
        | Program_complete ->
            add_edge source program_complete Rel_success_complete
              Success_program_complete ""
      ) program.success_continuations;
      List.iter (fun f ->
        match f.provenance with
        | Evaluation_input _ -> ()
        | Origin_provenance oid ->
            add_edge (find_fact f.fact_id) (find_origin oid)
              Rel_fact_provenance_origin Provenance_origin ""
        | Role_proxy rid ->
            let scope = Hashtbl.find fact_scope_by_id (string_of_fact_id f.fact_id) in
            let role_scope = role_scope_for_fact (owner_scope_to_role_scope Program_owner) scope in
            add_edge (find_fact f.fact_id) (find_role role_scope rid)
              Rel_fact_provenance_role Provenance_role_proxy ""
      ) facts;
      let sort_edges edges =
        List.sort (fun a b ->
          let c = compare (relation_rank a.relation) (relation_rank b.relation) in
          if c <> 0 then c else
          let c = compare (discriminator_rank a.discriminator) (discriminator_rank b.discriminator) in
          if c <> 0 then c else
          let c = String.compare a.payload b.payload in
          if c <> 0 then c else compare a.target b.target
        ) edges
      in
      let forward = Array.map (fun r -> Array.of_list (sort_edges !r)) forward_builders in
      let reverse = Array.map (fun r -> Array.of_list (sort_edges !r)) reverse_builders in
      Ok { vertices = vertex_array; forward; reverse }

let vertex_count model = Array.length model.vertices

let vertex_kind model index = model.vertices.(index).kind

let vertex_scalar model index = model.vertices.(index).scalar

let vertex_family_count model family =
  Array.fold_left (fun count vertex ->
    match vertex.kind with
    | Anonymous f when f = family -> count + 1
    | _ -> count
  ) 0 model.vertices

let forward_edges model index = Array.to_list model.forward.(index)

let reverse_edges model index = Array.to_list model.reverse.(index)

let all_forward_edges model =
  Array.fold_left (fun acc edges -> List.rev_append (Array.to_list edges) acc) [] model.forward

let relation_kinds_present model =
  let add relation seen = if List.mem relation seen then seen else relation :: seen in
  List.sort (fun a b -> compare (relation_rank a) (relation_rank b))
    (List.fold_left (fun seen edge -> add edge.relation seen) [] (all_forward_edges model))

let relation_name = string_of_relation

let edge_evidence model edge =
  string_of_relation edge.relation ^ ":" ^ string_of_discriminator edge.discriminator ^
  ":payload=" ^ stable_string edge.payload ^
  ":target=" ^ string_of_kind model.vertices.(edge.target).kind ^
  ":" ^ stable_string model.vertices.(edge.target).scalar

let vertex_evidence model index =
  let vertex = model.vertices.(index) in
  let forward = List.sort String.compare
      (List.map (edge_evidence model) (Array.to_list model.forward.(index))) in
  let reverse = List.sort String.compare
      (List.map (edge_evidence model) (Array.to_list model.reverse.(index))) in
  string_of_kind vertex.kind ^ ":scalar=" ^ stable_string vertex.scalar ^
  ":forward=[" ^ String.concat "," forward ^ "]" ^
  ":reverse=[" ^ String.concat "," reverse ^ "]"

let structural_evidence model =
  Array.to_list (Array.mapi (fun index _ -> vertex_evidence model index) model.vertices)
  |> List.sort String.compare
  |> String.concat "\n"

let required_relation_kinds = [
  Rel_root_entry_origin;
  Rel_success_next;
  Rel_success_complete;
  Rel_origin_declared_fact;
  Rel_batch_aggregate_fact;
  Rel_fact_provenance_origin;
  Rel_fact_provenance_role;
  Rel_entry_guard_fact;
  Rel_action_input_fact;
  Rel_action_input_origin;
  Rel_action_input_role;
  Rel_action_input_anchor;
  Rel_action_input_template;
  Rel_together_member;
  Rel_branch_subject;
  Rel_branch_target;
  Rel_branch_stop;
  Rel_role_contract_fact;
  Rel_role_scope_template;
  Rel_role_scope_program;
  Rel_template_origin;
  Rel_template_batch;
  Rel_template_branch;
  Rel_template_role;
  Rel_template_objective_role;
  Rel_program_input_fact;
  Rel_program_origin;
  Rel_program_batch;
  Rel_program_branch;
  Rel_program_role;
  Rel_program_template;
  Rel_fact_scope_owner;
  Rel_batch_template_context
]

let enc_v2_lookup_coverage = [
  ("OriginMap", [
    Rel_root_entry_origin; Rel_success_next; Rel_success_complete;
    Rel_origin_declared_fact; Rel_fact_provenance_origin;
    Rel_action_input_origin; Rel_action_input_anchor; Rel_together_member;
    Rel_branch_subject; Rel_branch_target; Rel_template_origin; Rel_program_origin
  ]);
  ("FactMap", [
    Rel_origin_declared_fact; Rel_batch_aggregate_fact;
    Rel_fact_provenance_origin; Rel_fact_provenance_role;
    Rel_entry_guard_fact; Rel_action_input_fact; Rel_role_contract_fact;
    Rel_program_input_fact; Rel_fact_scope_owner
  ]);
  ("BranchMap", [Rel_branch_subject; Rel_branch_target; Rel_branch_stop; Rel_template_branch; Rel_program_branch]);
  ("BatchMap", [Rel_batch_aggregate_fact; Rel_template_batch; Rel_program_batch; Rel_batch_template_context]);
  ("TemplateMap", [
    Rel_action_input_template; Rel_template_batch; Rel_role_scope_template;
    Rel_template_origin; Rel_template_branch; Rel_template_role;
    Rel_template_objective_role; Rel_program_template; Rel_batch_template_context
  ]);
  ("ScopedRoleMap", [
    Rel_fact_provenance_role; Rel_action_input_role; Rel_role_contract_fact;
    Rel_role_scope_template; Rel_role_scope_program; Rel_template_role;
    Rel_template_objective_role; Rel_program_role
  ])
]
