module Core = Tethers_core
module Validator = Tethers_core_validator
module Model = Tethers_core_rocket_v3_model
module Partition = Tethers_core_rocket_v3_partition
module Format = Tethers_core_canonical_v2_format

type leaf = {
  labels : Format.label_assignment;
  payload : string;
  preimage : bytes;
  digest : string;
}

type error =
  | Invalid_core of Validator.validation_error list
  | Model_mismatch
  | Partition_not_stable
  | Partition_not_discrete
  | Missing_vertex of string
  | No_legal_label_assignment of string

(*
   Refining deterministic ancestral ordination strategy.

   A discrete V3 partition identifies semantic vertices, but it does not
   choose their independent V2 numeric labels.  The ordering certificate
   therefore enumerates the complete legal residual domain: every non-role
   family ranges over every bijection onto 1..N, and every role scope ranges
   over every bijection onto its frozen contiguous block.  Each candidate is
   passed to the shared frozen Enc_V2 encoder and the unsigned-byte minimum is
   retained.  No candidate is discarded by a heuristic, identity handle, cell
   number or partial byte comparison.
*)

let add_origin map (id, label) = Format.OriginMap.add id label map
let add_batch map (id, label) = Format.BatchMap.add id label map

let enumerate_permutations count callback =
  let chosen = Array.make count 0 in
  let used = Array.make (count + 1) false in
  let rec visit position =
    if position = count then
      callback (Array.copy chosen)
    else
      for label = 1 to count do
        if not used.(label) then begin
          used.(label) <- true;
          chosen.(position) <- label;
          visit (position + 1);
          used.(label) <- false
        end
      done
  in
  visit 0

let map_of_array entries labels add empty =
  let indices = List.init (Array.length labels) Fun.id in
  List.fold_left2
    (fun map (id, _) index -> add map (id, labels.(index)))
    empty entries indices

let fact_map_of_array entries labels =
  let indices = List.init (Array.length labels) Fun.id in
  List.fold_left2
    (fun map (fact : Core.fact) index ->
       Format.FactMap.add fact.Core.fact_id labels.(index) map)
    Format.FactMap.empty entries indices

let branch_map_of_array entries labels =
  let indices = List.init (Array.length labels) Fun.id in
  List.fold_left2
    (fun map (branch, _) index ->
       Format.BranchMap.add branch.Core.branch_id labels.(index) map)
    Format.BranchMap.empty entries indices

let template_map_of_array entries labels =
  let indices = List.init (Array.length labels) Fun.id in
  List.fold_left2
    (fun map (template : Core.item_template) index ->
       Format.TemplateMap.add template.Core.item_template_id labels.(index) map)
    Format.TemplateMap.empty entries indices

let lookup_or_error family find entries =
  let rec loop = function
    | [] -> Ok ()
    | entry :: rest ->
        if Option.is_some (find entry) then loop rest
        else Error (Missing_vertex family)
  in
  loop entries

let scope_of_owner = function
  | `Program -> Core.Program_scope
  | `Template tid -> Core.Item_template_scope tid

let validate_lookups model origins facts batches branches templates roles =
  let origin_ids = List.map fst origins in
  let fact_ids = List.map (fun (fact : Core.fact) -> fact.Core.fact_id) facts in
  let batch_ids = List.map fst batches in
  let branch_ids = List.map (fun (branch, _) -> branch.Core.branch_id) branches in
  let template_ids = List.map (fun (template : Core.item_template) ->
    template.Core.item_template_id) templates in
  let role_keys = List.map (fun (role, owner) ->
    (scope_of_owner owner, role.Core.role_id)) roles
  in
  let ( let* ) = Result.bind in
  let* () = lookup_or_error "Origin" (Model.find_origin_vertex model) origin_ids in
  let* () = lookup_or_error "Fact" (Model.find_fact_vertex model) fact_ids in
  let* () = lookup_or_error "Batch" (Model.find_batch_vertex model) batch_ids in
  let* () = lookup_or_error "Branch"
      (Model.find_branch_vertex model) branch_ids in
  let* () = lookup_or_error "ItemTemplate"
      (Model.find_template_vertex model) template_ids in
  let rec check_roles = function
    | [] -> Ok ()
    | (scope, role_id) :: rest ->
        if Option.is_some (Model.find_scoped_role_vertex model scope role_id)
        then check_roles rest
        else Error (Missing_vertex "ScopedRole")
  in
  check_roles role_keys

let role_groups template_labels templates roles =
  let groups = List.filter_map (fun (template : Core.item_template) ->
    let template_id = template.Core.item_template_id in
    let members = List.filter_map (fun (role, owner) ->
      match owner with
      | `Template tid when tid = template_id -> Some role
      | `Program
      | `Template _ -> None
    ) roles in
    if members = [] then None else Some (template_id, members)
  ) templates in
  List.sort (fun (left, _) (right, _) ->
    Int.compare
      (Format.TemplateMap.find left template_labels)
      (Format.TemplateMap.find right template_labels)
  ) groups

let enumerate_role_labels template_labels templates roles callback =
  let program_roles = List.filter_map (fun (role, owner) ->
    match owner with
    | `Program -> Some role
    | `Template _ -> None
  ) roles in
  let groups = role_groups template_labels templates roles in
  let add_program_roles map labels =
    let indices = List.init (Array.length labels) Fun.id in
    List.fold_left2 (fun map role index ->
      Format.ScopedRoleMap.add
        (Format.Program_role role.Core.role_id) labels.(index) map
    ) map program_roles indices
  in
  let add_template_roles map start template_id template_roles labels =
    let indices = List.init (Array.length labels) Fun.id in
    List.fold_left2 (fun map role index ->
      Format.ScopedRoleMap.add
        (Format.Template_role (template_id, role.Core.role_id))
        (start + labels.(index) - 1) map
    ) map template_roles indices
  in
  enumerate_permutations (List.length program_roles) (fun program_labels ->
    let initial = add_program_roles Format.ScopedRoleMap.empty program_labels in
    let rec visit next_label map = function
      | [] -> callback map
      | (template_id, template_roles) :: rest ->
          enumerate_permutations (List.length template_roles) (fun labels ->
            let map = add_template_roles map next_label template_id
                template_roles labels in
            visit (next_label + List.length template_roles) map rest)
    in
    visit (1 + List.length program_roles) initial groups)

let encode program model partition =
  match Validator.validate program with
  | Error errors -> Error (Invalid_core errors)
  | Ok () ->
      let partition_model = Partition.model partition in
      if Model.structural_evidence model <> Model.structural_evidence partition_model then
        Error Model_mismatch
      else if not (Partition.is_stable partition) then
        Error Partition_not_stable
      else if not (Partition.is_discrete partition) then
        Error Partition_not_discrete
      else
        let origins = Format.collect_origins program in
        let facts = Format.collect_facts program in
        let batches = Format.collect_batches program in
        let branches = Format.collect_branches program in
        let templates = program.Core.item_templates in
        let roles = Format.collect_roles program in
        begin match validate_lookups model origins facts batches branches templates roles with
        | Error error -> Error error
        | Ok () ->
            let best = ref None in
            let consider labels =
              let payload = Format.encode_program labels program in
              match !best with
              | None -> best := Some (labels, payload)
              | Some (_, current) ->
                  if Format.compare_bytes_lex_unsigned payload current < 0 then
                    best := Some (labels, payload)
            in
            enumerate_permutations (List.length facts) (fun fact_labels ->
              let fact_map = fact_map_of_array facts fact_labels in
              enumerate_permutations (List.length origins) (fun origin_labels ->
                let origin_map = map_of_array origins origin_labels
                    add_origin Format.OriginMap.empty in
                enumerate_permutations (List.length batches) (fun batch_labels ->
                  let batch_map = map_of_array batches batch_labels
                      add_batch Format.BatchMap.empty in
                  enumerate_permutations (List.length branches) (fun branch_labels ->
                    let branch_map = branch_map_of_array branches branch_labels in
                    enumerate_permutations (List.length templates)
                      (fun template_labels ->
                        let template_map = template_map_of_array templates
                            template_labels in
                        enumerate_role_labels template_map templates roles
                          (fun role_map ->
                            consider {
                              Format.origin_labels = origin_map;
                              fact_labels = fact_map;
                              branch_labels = branch_map;
                              batch_labels = batch_map;
                              template_labels = template_map;
                              role_labels = role_map;
                            })))))
            );
            match !best with
            | None -> Error (No_legal_label_assignment "empty label domain")
            | Some (labels, payload) ->
                let preimage = Bytes.concat Bytes.empty
                    [Format.domain_v2; Bytes.of_string payload] in
                let digest = Format.digest_string_v2
                    (Format.sha256_hex preimage) in
                Ok { labels; payload; preimage; digest }
        end
