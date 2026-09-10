(** Abstract mutable state for one complete typed partition refinement run. *)

type t

type split_result = {
  changed : bool;
  retained : int;
  parts : int list;
}

val create : Tethers_core_rocket_v3_model.t -> t
val model : t -> Tethers_core_rocket_v3_model.t
val vertex_count : t -> int
val cell_count : t -> int
val cell_ids : t -> int list
val cell_of_vertex : t -> int -> int
val cell_members : t -> int -> int list
val cell_size : t -> int -> int
val cell_key : t -> int -> string
val initial_key : t -> int -> string
val same_cell : t -> int -> int -> bool
val is_discrete : t -> bool
val is_stable : t -> bool
val evidence : t -> string

(** [split_cell partition cell groups] replaces [cell] by the supplied
    equivalence classes.  Group keys are semantic refinement keys and are used
    only to select the retained largest part and to schedule work
    deterministically; vertex handles never participate in those decisions. *)
val split_cell : t -> int -> (string * int list) list -> split_result

val mark_stable : t -> unit
