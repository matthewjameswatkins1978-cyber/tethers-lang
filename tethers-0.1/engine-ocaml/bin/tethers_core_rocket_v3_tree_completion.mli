type target =
  | Origin_label of int
  | Program_complete

type partial_parent_vector
type rooted_tree

type completion = {
  targets : target array;
  slot_nodes : int array;
}

type oracle_stats = {
  completions_considered : int;
}

val make_tree : parent:int array -> entry:int -> (rooted_tree, string) result
val make_prefix :
  tree_size:int -> processed_slots:int -> target list ->
  (partial_parent_vector, string) result

val brute_force_completable :
  rooted_tree -> entry_label:int -> partial_parent_vector -> bool

val brute_force_completable_with_stats :
  rooted_tree -> entry_label:int -> partial_parent_vector -> bool * oracle_stats

val brute_force_minimum :
  rooted_tree -> entry_label:int -> completion option

val local_capacity_candidate :
  rooted_tree -> entry_label:int -> partial_parent_vector -> bool

val target_to_string : target -> string
