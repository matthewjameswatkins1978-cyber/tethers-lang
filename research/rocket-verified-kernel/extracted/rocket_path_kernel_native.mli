
val fst : ('a1 * 'a2) -> 'a1

val snd : ('a1 * 'a2) -> 'a2

val app : 'a1 list -> 'a1 list -> 'a1 list

type comparison =
| Eq
| Lt
| Gt

val pred : int -> int

module Nat :
 sig
  val sub : int -> int -> int

  val ltb : int -> int -> bool

  val compare : int -> int -> comparison

  val divmod : int -> int -> int -> int -> int * int

  val div : int -> int -> int

  val modulo : int -> int -> int
 end

val map : ('a1 -> 'a2) -> 'a1 list -> 'a2 list

val seq : int -> int -> int list

val rev : 'a1 list -> 'a1 list

module PathModel :
 sig
  type target =
  | Origin of int
  | Complete

  val labels : int -> int list

  val valid_label : int -> int -> bool

  val lookup_nat : int -> (int * int) list -> int option

  val update_nat : int -> int -> (int * int) list -> (int * int) list

  val member_nat : int -> int list -> bool

  val digits_rev_fuel : int -> int -> int list

  val digits_rev : int -> int list

  val decimal_digits : int -> int list

  val compare_digits : int list -> int list -> comparison

  val compare_encoded_int : int -> int -> comparison

  val encoded_int_lt : int -> int -> bool

  type successor_table = (int * target) list

  type solver_state = { st_size : int; st_entry : int;
                        st_edges : successor_table;
                        st_predecessors : int list;
                        st_parent : (int * int) list; st_components : 
                        int; st_terminal : int option }

  val st_size : solver_state -> int

  val st_entry : solver_state -> int

  val st_edges : solver_state -> successor_table

  val st_predecessors : solver_state -> int list

  val st_parent : solver_state -> (int * int) list

  val st_components : solver_state -> int

  val st_terminal : solver_state -> int option

  val make_state :
    int -> int -> successor_table -> int list -> (int * int) list -> int ->
    int option -> solver_state

  val initial_state : int -> int -> solver_state

  val dsu_find_fuel : int -> int -> (int * int) list -> int

  val dsu_find : int -> int -> (int * int) list -> int

  val dsu_union :
    int -> int -> int -> (int * int) list -> (int * int) list option

  val state_with_edge :
    solver_state -> int -> target -> int list -> (int * int) list -> int ->
    int option -> solver_state

  val try_origin : solver_state -> int -> int -> solver_state option

  val try_complete : solver_state -> int -> solver_state option

  val try_target : solver_state -> int -> target -> solver_state option

  val partial_state_feasible : solver_state -> int -> bool

  val insert_encoded : int -> int list -> int list

  val sort_encoded : int list -> int list

  val ordered_candidates : solver_state -> int list -> target list

  val choose_candidate :
    int -> solver_state -> int -> int -> target list -> solver_state option

  val process_sources :
    int -> int -> int list -> solver_state -> solver_state option

  val minimum_encoded_aux : int -> int list -> int

  val minimum_encoded_label : int -> int

  val canonical_state : int -> solver_state option

  val lookup_successor : int -> (int * target) list -> target option

  val follow_aux :
    int -> int -> int list -> (int * target) list -> int list -> int list
    option

  val canonical_labels : int -> int list

  val canonical_successors : int -> successor_table
 end

module PathCanon :
 sig
  val canonical_assignment : int -> int list

  val induced_successors : int -> PathModel.successor_table
 end
