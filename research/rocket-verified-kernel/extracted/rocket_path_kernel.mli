
type bool =
| True
| False

val negb : bool -> bool

type nat =
| O
| S of nat

type 'a option =
| Some of 'a
| None

type ('a, 'b) prod =
| Pair of 'a * 'b

val fst : ('a1, 'a2) prod -> 'a1

val snd : ('a1, 'a2) prod -> 'a2

type 'a list =
| Nil
| Cons of 'a * 'a list

val app : 'a1 list -> 'a1 list -> 'a1 list

type comparison =
| Eq
| Lt
| Gt

val pred : nat -> nat

module Nat :
 sig
  val sub : nat -> nat -> nat

  val eqb : nat -> nat -> bool

  val leb : nat -> nat -> bool

  val ltb : nat -> nat -> bool

  val compare : nat -> nat -> comparison

  val divmod : nat -> nat -> nat -> nat -> (nat, nat) prod

  val div : nat -> nat -> nat

  val modulo : nat -> nat -> nat
 end

val map : ('a1 -> 'a2) -> 'a1 list -> 'a2 list

val seq : nat -> nat -> nat list

val rev : 'a1 list -> 'a1 list

val filter : ('a1 -> bool) -> 'a1 list -> 'a1 list

module PathModel :
 sig
  type target =
  | Origin of nat
  | Complete

  val labels : nat -> nat list

  val valid_label : nat -> nat -> bool

  val lookup_nat : nat -> (nat, nat) prod list -> nat option

  val update_nat : nat -> nat -> (nat, nat) prod list -> (nat, nat) prod list

  val member_nat : nat -> nat list -> bool

  val digits_rev_fuel : nat -> nat -> nat list

  val digits_rev : nat -> nat list

  val decimal_digits : nat -> nat list

  val compare_digits : nat list -> nat list -> comparison

  val compare_encoded_int : nat -> nat -> comparison

  val encoded_int_lt : nat -> nat -> bool

  type successor_table = (nat, target) prod list

  type solver_state = { st_size : nat; st_entry : nat;
                        st_edges : successor_table;
                        st_predecessors : nat list;
                        st_parent : (nat, nat) prod list;
                        st_components : nat; st_terminal : nat option }

  val st_size : solver_state -> nat

  val st_entry : solver_state -> nat

  val st_edges : solver_state -> successor_table

  val st_predecessors : solver_state -> nat list

  val st_parent : solver_state -> (nat, nat) prod list

  val st_components : solver_state -> nat

  val st_terminal : solver_state -> nat option

  val make_state :
    nat -> nat -> successor_table -> nat list -> (nat, nat) prod list -> nat
    -> nat option -> solver_state

  val initial_state : nat -> nat -> solver_state

  val dsu_find_fuel : nat -> nat -> (nat, nat) prod list -> nat

  val dsu_find : nat -> nat -> (nat, nat) prod list -> nat

  val dsu_union :
    nat -> nat -> nat -> (nat, nat) prod list -> (nat, nat) prod list option

  val state_with_edge :
    solver_state -> nat -> target -> nat list -> (nat, nat) prod list -> nat
    -> nat option -> solver_state

  val try_origin : solver_state -> nat -> nat -> solver_state option

  val try_complete : solver_state -> nat -> solver_state option

  val try_target : solver_state -> nat -> target -> solver_state option

  val partial_state_feasible : solver_state -> nat -> bool

  val insert_encoded : nat -> nat list -> nat list

  val sort_encoded : nat list -> nat list

  val candidate_labels : solver_state -> nat list -> nat list

  val ordered_candidates : solver_state -> nat list -> target list

  val choose_candidate :
    nat -> solver_state -> nat -> nat -> target list -> solver_state option

  val process_sources :
    nat -> nat -> nat list -> solver_state -> solver_state option

  val minimum_encoded_aux : nat -> nat list -> nat

  val minimum_encoded_label : nat -> nat

  val canonical_state : nat -> solver_state option

  val lookup_successor : nat -> (nat, target) prod list -> target option

  val follow_aux :
    nat -> nat -> nat list -> (nat, target) prod list -> nat list -> nat list
    option

  val canonical_labels : nat -> nat list

  val canonical_successors : nat -> successor_table
 end

module PathCanon :
 sig
  val canonical_assignment : nat -> nat list

  val induced_successors : nat -> PathModel.successor_table
 end
