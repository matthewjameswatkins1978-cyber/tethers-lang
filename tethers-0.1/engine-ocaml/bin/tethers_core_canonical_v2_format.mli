(* ==================================================================
   CANONICAL FORMAT V2 — SHARED FROZEN FORMAT LAYER (interface)

   This module contains the frozen Enc_V2 byte encoder, domain
   separation, primitive encoders, typed label maps, entity
   collection helpers, and the unsigned-byte lexicographic comparator.

   Both the reference oracle and the production canonicaliser import
   this module.  Search implementations remain independent.
   ================================================================== *)

open Tethers_core

(** Domain separation bytes: "TETHERS_CORE_CANON_V2" || 0x00 *)
val domain_v2 : bytes

(** SHA-256 hex digest *)
val sha256_hex : bytes -> string

(** Construct the external digest string: "tethers:v2:sha256:<hex>" *)
val digest_string_v2 : string -> string

(* ================================================================== *)
(*  Unsigned byte lexicographic comparator                              *)
(*  compare each byte as 0..255; first differing byte decides;          *)
(*  exact prefix: shorter string wins.                                  *)
(* ================================================================== *)

val compare_bytes_lex_unsigned : string -> string -> int

(* ================================================================== *)
(*  Family-safe typed label maps                                        *)
(* ================================================================== *)

module OriginMap : Map.S with type key = origin_id
module FactMap : Map.S with type key = fact_id
module BranchMap : Map.S with type key = branch_id
module BatchMap : Map.S with type key = batch_id
module TemplateMap : Map.S with type key = item_template_id
module RoleMap : Map.S with type key = role_id

type scoped_role_key =
  | Program_role of role_id
  | Template_role of item_template_id * role_id

module ScopedRoleMap : Map.S with type key = scoped_role_key

(* ================================================================== *)
(*  Label assignment                                                   *)
(* ================================================================== *)

type label_assignment = {
  origin_labels   : int OriginMap.t;
  fact_labels     : int FactMap.t;
  branch_labels   : int BranchMap.t;
  batch_labels    : int BatchMap.t;
  template_labels : int TemplateMap.t;
  role_labels     : int ScopedRoleMap.t;
}

(* ================================================================== *)
(*  Label lookups — failwith on missing label (internal error)          *)
(* ================================================================== *)

val lookup_origin  : label_assignment -> origin_id -> int
val lookup_fact    : label_assignment -> fact_id -> int
val lookup_branch  : label_assignment -> branch_id -> int
val lookup_batch   : label_assignment -> batch_id -> int
val lookup_template: label_assignment -> item_template_id -> int
val lookup_scoped_role : label_assignment -> scoped_role_key -> int
val lookup_role_in_scope : label_assignment -> role_scope -> role_id -> int

(* ================================================================== *)
(*  Primitive encoders (frozen §6.2)                                    *)
(* ================================================================== *)

val encode_string : string -> string
val encode_int    : int -> string
val encode_tag    : int -> string
val encode_list   : ('a -> string) -> 'a list -> string
val encode_option : ('a -> string) -> 'a option -> string

(* ================================================================== *)
(*  Tag / rank helpers (frozen §6.3)                                    *)
(* ================================================================== *)

val operator_rank : comparison_operator -> int
val outcome_rank  : terminal_outcome -> int

(* ================================================================== *)
(*  Entity collection from program                                      *)
(* ================================================================== *)

val origin_id_of_site : origin_site -> origin_id option
val collect_origins   : program -> (origin_id * origin_site) list
val collect_facts     : program -> fact list
val collect_batches   : program -> (batch_id * batch_site) list
val collect_branches  : program -> (branch * [`Program | `Template of item_template_id]) list
val collect_roles     : program -> (role * [`Program | `Template of item_template_id]) list

(* ================================================================== *)
(*  Mixed-origin/Batch sort key (§9.3.1)                                *)
(* ================================================================== *)

type sort_key = int * int

val origin_sort_key : label_assignment -> origin_site -> sort_key
val sort_origin_sites : label_assignment -> origin_site list -> origin_site list

(* ================================================================== *)
(*  Frozen Enc_V2 encoder (§6.4)                                        *)
(* ================================================================== *)

val encode_scalar_type    : core_scalar_type -> string
val encode_value          : core_value -> string
val encode_provenance     : label_assignment -> role_scope -> fact_provenance -> string
val encode_fact           : label_assignment -> fact_scope:role_scope -> fact -> string
val encode_fact_guard     : label_assignment -> fact_guard -> string
val encode_binding        : label_assignment -> origin_scope:role_scope -> input_binding -> string
val encode_action_input   : label_assignment -> origin_scope:role_scope -> action_input -> string
val encode_constraint     : execution_constraint -> string
val encode_origin_site    : label_assignment -> origin_scope:role_scope -> origin_site -> string
val encode_branch         : label_assignment -> branch -> string
val encode_role_scope     : label_assignment -> role_scope -> string
val encode_role           : label_assignment -> role_scope:role_scope -> role -> string
val encode_item_objective : label_assignment -> template_scope:item_template_id -> item_objective -> string
val encode_item_template  : label_assignment -> item_template -> string
val encode_capability_contract : label_assignment -> capability_contract -> string

(** Top-level encoder: frozen field order (§6.4) *)
val encode_program : label_assignment -> program -> string
