(** Incremental stable typed equitable refinement. *)

type direction =
  | Forward
  | Reverse

type stats = {
  relation_visits : int;
  splitter_pops : int;
  cell_splits : int;
  max_worklist : int;
  final_cell_count : int;
}

type result = {
  partition : Tethers_core_rocket_v3_partition.t;
  stats : stats;
}

val refine : Tethers_core_rocket_v3_partition.t -> result
val run : Tethers_core_rocket_v3_model.t -> result
