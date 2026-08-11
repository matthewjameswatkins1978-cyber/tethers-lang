type program_id = Program_id of string
type origin_id = Origin_id of string
type fact_id = Fact_id of string
type role_id = Role_id of string
type capability_id = Capability_id of string
type branch_id = Branch_id of string
type group_id = Group_id of string
type batch_id = Batch_id of string
type item_template_id = Item_template_id of string

let program_id_of_string s = Program_id s
let string_of_program_id (Program_id s) = s

let origin_id_of_string s = Origin_id s
let string_of_origin_id (Origin_id s) = s

let fact_id_of_string s = Fact_id s
let string_of_fact_id (Fact_id s) = s

let role_id_of_string s = Role_id s
let string_of_role_id (Role_id s) = s

let capability_id_of_string s = Capability_id s
let string_of_capability_id (Capability_id s) = s

let branch_id_of_string s = Branch_id s
let string_of_branch_id (Branch_id s) = s

let group_id_of_string s = Group_id s
let string_of_group_id (Group_id s) = s

let batch_id_of_string s = Batch_id s
let string_of_batch_id (Batch_id s) = s

let item_template_id_of_string s = Item_template_id s
let string_of_item_template_id (Item_template_id s) = s

type capability_contract_digest = Capability_contract_digest of string
type core_version = Core_version of string

let capability_contract_digest_of_string s = Capability_contract_digest s
let string_of_capability_contract_digest (Capability_contract_digest s) = s

let core_version_of_string s = Core_version s
let string_of_core_version (Core_version s) = s

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

type fact_provenance =
  | Origin_provenance of origin_id
  | Role_proxy of role_id

type fact = {
  fact_id : fact_id;
  schema_description : string;
  provenance : fact_provenance;
}

type execution_constraint =
  | Deadline of string

type input_binding =
  | Literal_value of string
  | Fact_from_origin of fact_id * origin_id
  | Fact_through_role of fact_id * role_id
  | Batch_item_context of item_template_id

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

type batch_collection_provenance = Batch_collection_provenance of string
type batch_traversal_policy = Batch_traversal_policy of string
type batch_objective = Batch_objective of string

let batch_collection_provenance_of_string s = Batch_collection_provenance s
let string_of_batch_collection_provenance (Batch_collection_provenance s) = s

let batch_traversal_policy_of_string s = Batch_traversal_policy s
let string_of_batch_traversal_policy (Batch_traversal_policy s) = s

let batch_objective_of_string s = Batch_objective s
let string_of_batch_objective (Batch_objective s) = s

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

type branch_target =
  | Continue_to of origin_id
  | Stop

type branch = {
  branch_id : branch_id;
  branch_subject : origin_id;
  outcome_branches : (terminal_outcome * branch_target) list;
}

type role_scope =
  | Program_scope
  | Item_template_scope of item_template_id

type role_fact_contract = Role_fact_contract of fact_id list

type role_fulfillment = Role_fulfillment of string

let role_fulfillment_of_string s = Role_fulfillment s
let string_of_role_fulfillment (Role_fulfillment s) = s

type role = {
  role_id : role_id;
  scope : role_scope;
  fact_contract : role_fact_contract;
  eligible_fulfillment : role_fulfillment;
}

type item_objective =
  | Required_role of role_id

type item_template = {
  item_template_id : item_template_id;
  origin_sites : origin_site list;
  branches : branch list;
  roles : role list;
  objective : item_objective;
}

type capability_contract = {
  capability_id : capability_id;
  contract_digest : capability_contract_digest;
  schema_description : string;
}

type program = {
  program_id : program_id;
  core_version : core_version;
  origin_sites : origin_site list;
  branches : branch list;
  roles : role list;
  item_templates : item_template list;
  capability_contracts : capability_contract list;
}
