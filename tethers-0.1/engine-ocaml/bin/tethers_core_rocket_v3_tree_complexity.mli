type target =
  | Origin_label of int
  | Program_complete

type prefix

type prefix_analysis = {
  acyclic : bool;
  terminal_edges : int;
  component_count : int;
  specified_sources : int;
}

type relabelled_forest = {
  slot_of_vertex : int array;
  processed_sources : int;
  targets : target list;
}

val make_prefix : size:int -> entry_label:int -> target list ->
  (prefix, string) result

val analyse_prefix : prefix -> prefix_analysis

val relabel_non_roots_first : parent:int array ->
  (relabelled_forest, string) result

(** An independent bounded witness predicate for the rooted spanning-forest
    interpretation.  It enumerates only in this research test module and is
    not a production completion algorithm. *)
val spanning_forest_witness :
  semantic_parent:int array ->
  semantic_entry:int ->
  entry_label:int ->
  prefix:prefix -> bool
