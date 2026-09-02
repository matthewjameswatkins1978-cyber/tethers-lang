type target =
  | Origin_label of int
  | Program_complete

type relation = {
  source : int;
  target : target;
}

type verdict =
  | Proven_feasible
  | Proven_infeasible
  | Unknown_global_packing of string

type stats = {
  candidate_states : int;
  candidate_pairs : int;
  matching_instances : int;
  matching_vertices : int;
  matching_edges : int;
  matching_failures : int;
}

val evaluate_connected_component :
  semantic_parent:int array ->
  entry_semantic:int ->
  entry_label:int ->
  processed_slots:int ->
  relation list ->
  (verdict * stats, string) result
