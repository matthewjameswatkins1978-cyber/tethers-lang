(** Tethers Core semantic type vocabulary.

    CORE-1 defines nominal types only. The module is dormant: no existing code
    consumes [Tethers_core] values. Later packets will lower Human Tether AST
    into these types, validate Core graphs, and drive planning.

    The critical invariant is that each identity-bearing type is a distinct
    OCaml type.  [OriginId] and [RoleId] are not interchangeable string
    aliases. *)

(* ------------------------------------------------------------------ *)
(*  Nominal Core identity types                                        *)
(* ------------------------------------------------------------------ *)

type program_id = private Program_id of string
type origin_id = private Origin_id of string
type fact_id = private Fact_id of string
type role_id = private Role_id of string
type capability_id = private Capability_id of string
type branch_id = private Branch_id of string
type group_id = private Group_id of string
type batch_id = private Batch_id of string
type item_template_id = private Item_template_id of string

val program_id_of_string : string -> program_id
val string_of_program_id : program_id -> string

val origin_id_of_string : string -> origin_id
val string_of_origin_id : origin_id -> string

val fact_id_of_string : string -> fact_id
val string_of_fact_id : fact_id -> string

val role_id_of_string : string -> role_id
val string_of_role_id : role_id -> string

val capability_id_of_string : string -> capability_id
val string_of_capability_id : capability_id -> string

val branch_id_of_string : string -> branch_id
val string_of_branch_id : branch_id -> string

val group_id_of_string : string -> group_id
val string_of_group_id : group_id -> string

val batch_id_of_string : string -> batch_id
val string_of_batch_id : batch_id -> string

val item_template_id_of_string : string -> item_template_id
val string_of_item_template_id : item_template_id -> string

(* ------------------------------------------------------------------ *)
(*  Distinct semantic value types                                      *)
(* ------------------------------------------------------------------ *)

type capability_contract_digest = private Capability_contract_digest of string
type core_version = private Core_version of string

val capability_contract_digest_of_string :
  string -> capability_contract_digest
val string_of_capability_contract_digest :
  capability_contract_digest -> string

val core_version_of_string : string -> core_version
val string_of_core_version : core_version -> string

(* ------------------------------------------------------------------ *)
(*  Core primitive types                                               *)
(* ------------------------------------------------------------------ *)

type terminal_outcome =
  | Success
  | Failure
  | Uncertain
  | Cancelled

type traversal_decision =
  | Continue
  | Stop

type fact_availability =
  | Optional
  | Guaranteed

(* ------------------------------------------------------------------ *)
(*  Fact                                                               *)
(* ------------------------------------------------------------------ *)

type fact_provenance =
  | Origin_provenance of origin_id
  | Role_proxy of role_id

type fact = {
  fact_id : fact_id;
  schema_description : string;
  provenance : fact_provenance;
}

(* ------------------------------------------------------------------ *)
(*  Execution Constraints                                              *)
(* ------------------------------------------------------------------ *)

type execution_constraint =
  | Deadline of string  (** semantic duration/bound; no timer impl *)

(* ------------------------------------------------------------------ *)
(*  Input Binding                                                      *)
(* ------------------------------------------------------------------ *)

type input_binding =
  | Literal_value of string
  | Fact_from_origin of fact_id * origin_id
  | Fact_through_role of fact_id * role_id
  | Batch_item_context of item_template_id

(* ------------------------------------------------------------------ *)
(*  Origin Sites                                                       *)
(* ------------------------------------------------------------------ *)

type anchor_origin = {
  anchor_origin_id : origin_id;
  event_name : string;
  declared_facts : fact list;
}

type action_origin = {
  action_origin_id : origin_id;
  capability_id : capability_id;
  contract_digest : capability_contract_digest;
  input_bindings : input_binding list;
  declared_facts : fact list;
  execution_constraints : execution_constraint list;
}

type together_objective =
  | All_members_succeed

type together_origin = {
  together_origin_id : origin_id;
  group_id : group_id;
  member_origin_ids : origin_id list;
  objective : together_objective;
}

type batch_collection_provenance = private Batch_collection_provenance of string
type batch_traversal_policy = private Batch_traversal_policy of string
type batch_objective = private Batch_objective of string

val batch_collection_provenance_of_string :
  string -> batch_collection_provenance
val string_of_batch_collection_provenance :
  batch_collection_provenance -> string

val batch_traversal_policy_of_string : string -> batch_traversal_policy
val string_of_batch_traversal_policy : batch_traversal_policy -> string

val batch_objective_of_string : string -> batch_objective
val string_of_batch_objective : batch_objective -> string

type batch_site = {
  batch_id : batch_id;
  collection_provenance : batch_collection_provenance;
  item_template_id : item_template_id;
  traversal_policy : batch_traversal_policy;
  composite_objective : batch_objective;
  aggregate_facts : fact list;
}

type origin_site =
  | Anchor_origin of anchor_origin
  | Action_origin of action_origin
  | Together_origin of together_origin
  | Batch_site of batch_site

(* ------------------------------------------------------------------ *)
(*  Branch                                                             *)
(* ------------------------------------------------------------------ *)

type branch_target =
  | Continue_to of origin_id
  | Stop

type branch = {
  branch_id : branch_id;
  branch_subject : origin_id;
  outcome_branches : (terminal_outcome * branch_target) list;
}

(* ------------------------------------------------------------------ *)
(*  Role                                                               *)
(* ------------------------------------------------------------------ *)

type role_scope =
  | Program_scope
  | Item_template_scope of item_template_id

type role_fact_contract = Role_fact_contract of fact_id list

type role_fulfillment = private Role_fulfillment of string

val role_fulfillment_of_string : string -> role_fulfillment
val string_of_role_fulfillment : role_fulfillment -> string

type role = {
  role_id : role_id;
  scope : role_scope;
  fact_contract : role_fact_contract;
  eligible_fulfillment : role_fulfillment;
}

(* ------------------------------------------------------------------ *)
(*  Item Template                                                      *)
(* ------------------------------------------------------------------ *)

type item_objective =
  | Required_role of role_id

type item_template = {
  item_template_id : item_template_id;
  origin_sites : origin_site list;
  branches : branch list;
  roles : role list;
  objective : item_objective;
}

(* ------------------------------------------------------------------ *)
(*  Capability Contract                                                *)
(* ------------------------------------------------------------------ *)

type capability_contract = {
  capability_id : capability_id;
  contract_digest : capability_contract_digest;
  schema_description : string;
}

(* ------------------------------------------------------------------ *)
(*  Program                                                            *)
(* ------------------------------------------------------------------ *)

type program = {
  program_id : program_id;
  core_version : core_version;
  origin_sites : origin_site list;
  branches : branch list;
  roles : role list;
  item_templates : item_template list;
  capability_contracts : capability_contract list;
}
