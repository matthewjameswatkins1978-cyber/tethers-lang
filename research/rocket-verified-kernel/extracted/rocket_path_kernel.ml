
type bool =
| True
| False

(** val negb : bool -> bool **)

let negb = function
| True -> False
| False -> True

type nat =
| O
| S of nat

type 'a option =
| Some of 'a
| None

type ('a, 'b) prod =
| Pair of 'a * 'b

(** val fst : ('a1, 'a2) prod -> 'a1 **)

let fst = function
| Pair (x, _) -> x

(** val snd : ('a1, 'a2) prod -> 'a2 **)

let snd = function
| Pair (_, y) -> y

type 'a list =
| Nil
| Cons of 'a * 'a list

(** val app : 'a1 list -> 'a1 list -> 'a1 list **)

let rec app l m =
  match l with
  | Nil -> m
  | Cons (a, l1) -> Cons (a, (app l1 m))

type comparison =
| Eq
| Lt
| Gt

(** val pred : nat -> nat **)

let pred n = match n with
| O -> n
| S u -> u

module Nat =
 struct
  (** val sub : nat -> nat -> nat **)

  let rec sub n m =
    match n with
    | O -> n
    | S k -> (match m with
              | O -> n
              | S l -> sub k l)

  (** val eqb : nat -> nat -> bool **)

  let rec eqb n m =
    match n with
    | O -> (match m with
            | O -> True
            | S _ -> False)
    | S n' -> (match m with
               | O -> False
               | S m' -> eqb n' m')

  (** val leb : nat -> nat -> bool **)

  let rec leb n m =
    match n with
    | O -> True
    | S n' -> (match m with
               | O -> False
               | S m' -> leb n' m')

  (** val ltb : nat -> nat -> bool **)

  let ltb n m =
    leb (S n) m

  (** val compare : nat -> nat -> comparison **)

  let rec compare n m =
    match n with
    | O -> (match m with
            | O -> Eq
            | S _ -> Lt)
    | S n' -> (match m with
               | O -> Gt
               | S m' -> compare n' m')

  (** val divmod : nat -> nat -> nat -> nat -> (nat, nat) prod **)

  let rec divmod x y q u =
    match x with
    | O -> Pair (q, u)
    | S x' ->
      (match u with
       | O -> divmod x' y (S q) y
       | S u' -> divmod x' y q u')

  (** val div : nat -> nat -> nat **)

  let div x y = match y with
  | O -> y
  | S y' -> fst (divmod x y' O y')

  (** val modulo : nat -> nat -> nat **)

  let modulo x = function
  | O -> x
  | S y' -> sub y' (snd (divmod x y' O y'))
 end

(** val map : ('a1 -> 'a2) -> 'a1 list -> 'a2 list **)

let rec map f = function
| Nil -> Nil
| Cons (a, l0) -> Cons ((f a), (map f l0))

(** val seq : nat -> nat -> nat list **)

let rec seq start = function
| O -> Nil
| S len0 -> Cons (start, (seq (S start) len0))

(** val rev : 'a1 list -> 'a1 list **)

let rec rev = function
| Nil -> Nil
| Cons (x, l') -> app (rev l') (Cons (x, Nil))

(** val filter : ('a1 -> bool) -> 'a1 list -> 'a1 list **)

let rec filter f = function
| Nil -> Nil
| Cons (x, l0) ->
  (match f x with
   | True -> Cons (x, (filter f l0))
   | False -> filter f l0)

module PathModel =
 struct
  type target =
  | Origin of nat
  | Complete

  (** val labels : nat -> nat list **)

  let labels n =
    seq (S O) n

  (** val valid_label : nat -> nat -> bool **)

  let valid_label n label =
    match Nat.leb (S O) label with
    | True -> Nat.leb label n
    | False -> False

  (** val lookup_nat : nat -> (nat, nat) prod list -> nat option **)

  let rec lookup_nat key = function
  | Nil -> None
  | Cons (p, rest) ->
    let Pair (candidate, value) = p in
    (match Nat.eqb key candidate with
     | True -> Some value
     | False -> lookup_nat key rest)

  (** val update_nat :
      nat -> nat -> (nat, nat) prod list -> (nat, nat) prod list **)

  let rec update_nat key value = function
  | Nil -> Cons ((Pair (key, value)), Nil)
  | Cons (p, rest) ->
    let Pair (candidate, old_value) = p in
    (match Nat.eqb key candidate with
     | True -> Cons ((Pair (key, value)), rest)
     | False ->
       Cons ((Pair (candidate, old_value)), (update_nat key value rest)))

  (** val member_nat : nat -> nat list -> bool **)

  let rec member_nat key = function
  | Nil -> False
  | Cons (head, rest) ->
    (match Nat.eqb key head with
     | True -> True
     | False -> member_nat key rest)

  (** val digits_rev_fuel : nat -> nat -> nat list **)

  let rec digits_rev_fuel fuel value =
    match fuel with
    | O -> Nil
    | S remaining ->
      (match value with
       | O -> Nil
       | S _ ->
         Cons ((Nat.modulo value (S (S (S (S (S (S (S (S (S (S O))))))))))),
           (digits_rev_fuel remaining
             (Nat.div value (S (S (S (S (S (S (S (S (S (S O))))))))))))))

  (** val digits_rev : nat -> nat list **)

  let digits_rev value =
    digits_rev_fuel (S value) value

  (** val decimal_digits : nat -> nat list **)

  let decimal_digits value =
    rev (digits_rev value)

  (** val compare_digits : nat list -> nat list -> comparison **)

  let rec compare_digits left right =
    match left with
    | Nil -> (match right with
              | Nil -> Eq
              | Cons (_, _) -> Gt)
    | Cons (lh, lt) ->
      (match right with
       | Nil -> Lt
       | Cons (rh, rt) ->
         (match Nat.compare lh rh with
          | Eq -> compare_digits lt rt
          | x -> x))

  (** val compare_encoded_int : nat -> nat -> comparison **)

  let compare_encoded_int left right =
    compare_digits (decimal_digits left) (decimal_digits right)

  (** val encoded_int_lt : nat -> nat -> bool **)

  let encoded_int_lt left right =
    match compare_encoded_int left right with
    | Lt -> True
    | _ -> False

  type successor_table = (nat, target) prod list

  type solver_state = { st_size : nat; st_entry : nat;
                        st_edges : successor_table;
                        st_predecessors : nat list;
                        st_parent : (nat, nat) prod list;
                        st_components : nat; st_terminal : nat option }

  (** val st_size : solver_state -> nat **)

  let st_size s =
    s.st_size

  (** val st_entry : solver_state -> nat **)

  let st_entry s =
    s.st_entry

  (** val st_edges : solver_state -> successor_table **)

  let st_edges s =
    s.st_edges

  (** val st_predecessors : solver_state -> nat list **)

  let st_predecessors s =
    s.st_predecessors

  (** val st_parent : solver_state -> (nat, nat) prod list **)

  let st_parent s =
    s.st_parent

  (** val st_components : solver_state -> nat **)

  let st_components s =
    s.st_components

  (** val st_terminal : solver_state -> nat option **)

  let st_terminal s =
    s.st_terminal

  (** val make_state :
      nat -> nat -> successor_table -> nat list -> (nat, nat) prod list ->
      nat -> nat option -> solver_state **)

  let make_state size entry edges predecessors parent components terminal =
    { st_size = size; st_entry = entry; st_edges = edges; st_predecessors =
      predecessors; st_parent = parent; st_components = components;
      st_terminal = terminal }

  (** val initial_state : nat -> nat -> solver_state **)

  let initial_state size entry =
    make_state size entry Nil Nil
      (map (fun label -> Pair (label, label)) (labels size)) size None

  (** val dsu_find_fuel : nat -> nat -> (nat, nat) prod list -> nat **)

  let rec dsu_find_fuel fuel node parent =
    match fuel with
    | O -> node
    | S remaining ->
      (match lookup_nat node parent with
       | Some root ->
         (match Nat.eqb root node with
          | True -> node
          | False -> dsu_find_fuel remaining root parent)
       | None -> node)

  (** val dsu_find : nat -> nat -> (nat, nat) prod list -> nat **)

  let dsu_find size node parent =
    dsu_find_fuel (S size) node parent

  (** val dsu_union :
      nat -> nat -> nat -> (nat, nat) prod list -> (nat, nat) prod list option **)

  let dsu_union size left right parent =
    let left_root = dsu_find size left parent in
    let right_root = dsu_find size right parent in
    (match Nat.eqb left_root right_root with
     | True -> None
     | False -> Some (update_nat right_root left_root parent))

  (** val state_with_edge :
      solver_state -> nat -> target -> nat list -> (nat, nat) prod list ->
      nat -> nat option -> solver_state **)

  let state_with_edge state source new_target new_predecessors new_parent new_components new_terminal =
    make_state state.st_size state.st_entry (Cons ((Pair (source,
      new_target)), state.st_edges)) new_predecessors new_parent
      new_components new_terminal

  (** val try_origin : solver_state -> nat -> nat -> solver_state option **)

  let try_origin state source target_label =
    match valid_label state.st_size target_label with
    | True ->
      (match Nat.eqb target_label state.st_entry with
       | True -> None
       | False ->
         (match member_nat target_label state.st_predecessors with
          | True -> None
          | False ->
            (match Nat.eqb source target_label with
             | True -> None
             | False ->
               (match dsu_union state.st_size source target_label
                        state.st_parent with
                | Some parent' ->
                  Some
                    (state_with_edge state source (Origin target_label) (Cons
                      (target_label, state.st_predecessors)) parent'
                      (pred state.st_components) state.st_terminal)
                | None -> None))))
    | False -> None

  (** val try_complete : solver_state -> nat -> solver_state option **)

  let try_complete state source =
    match state.st_terminal with
    | Some _ -> None
    | None ->
      Some
        (state_with_edge state source Complete state.st_predecessors
          state.st_parent state.st_components (Some source))

  (** val try_target :
      solver_state -> nat -> target -> solver_state option **)

  let try_target state source = function
  | Origin target_label -> try_origin state source target_label
  | Complete -> try_complete state source

  (** val partial_state_feasible : solver_state -> nat -> bool **)

  let partial_state_feasible state processed =
    match Nat.leb processed state.st_size with
    | True ->
      (match valid_label state.st_size state.st_entry with
       | True ->
         (match state.st_terminal with
          | Some terminal_source ->
            let terminal_root =
              dsu_find state.st_size terminal_source state.st_parent
            in
            let entry_root =
              dsu_find state.st_size state.st_entry state.st_parent
            in
            (match Nat.eqb terminal_root entry_root with
             | True ->
               (match Nat.ltb (S O) state.st_components with
                | True -> False
                | False ->
                  (match Nat.eqb processed state.st_size with
                   | True ->
                     (match Nat.eqb state.st_components (S O) with
                      | True ->
                        Nat.eqb entry_root
                          (dsu_find state.st_size (S O) state.st_parent)
                      | False -> False)
                   | False -> True))
             | False ->
               (match Nat.eqb processed state.st_size with
                | True -> False
                | False -> True))
          | None ->
            (match Nat.eqb processed state.st_size with
             | True -> False
             | False -> True))
       | False -> False)
    | False -> False

  (** val insert_encoded : nat -> nat list -> nat list **)

  let rec insert_encoded label sorted = match sorted with
  | Nil -> Cons (label, Nil)
  | Cons (head, rest) ->
    (match encoded_int_lt label head with
     | True -> Cons (label, sorted)
     | False -> Cons (head, (insert_encoded label rest)))

  (** val sort_encoded : nat list -> nat list **)

  let rec sort_encoded = function
  | Nil -> Nil
  | Cons (head, rest) -> insert_encoded head (sort_encoded rest)

  (** val candidate_labels : solver_state -> nat list -> nat list **)

  let candidate_labels state ordered =
    filter (fun label ->
      negb
        (match Nat.eqb label state.st_entry with
         | True -> True
         | False -> member_nat label state.st_predecessors))
      ordered

  (** val ordered_candidates : solver_state -> nat list -> target list **)

  let ordered_candidates state ordered =
    app (map (fun x -> Origin x) (candidate_labels state ordered))
      (match state.st_terminal with
       | Some _ -> Nil
       | None -> Cons (Complete, Nil))

  (** val choose_candidate :
      nat -> solver_state -> nat -> nat -> target list -> solver_state option **)

  let rec choose_candidate fuel state source processed candidates =
    match fuel with
    | O -> None
    | S remaining ->
      (match candidates with
       | Nil -> None
       | Cons (candidate, rest) ->
         (match try_target state source candidate with
          | Some next_state ->
            (match partial_state_feasible next_state processed with
             | True -> Some next_state
             | False -> choose_candidate remaining state source processed rest)
          | None -> choose_candidate remaining state source processed rest))

  (** val process_sources :
      nat -> nat -> nat list -> solver_state -> solver_state option **)

  let rec process_sources fuel source ordered state =
    match fuel with
    | O -> Some state
    | S remaining ->
      (match Nat.leb source state.st_size with
       | True ->
         (match choose_candidate (S state.st_size) state source source
                  (ordered_candidates state ordered) with
          | Some next_state ->
            process_sources remaining (S source) ordered next_state
          | None -> None)
       | False -> Some state)

  (** val minimum_encoded_aux : nat -> nat list -> nat **)

  let rec minimum_encoded_aux best = function
  | Nil -> best
  | Cons (head, rest) ->
    (match encoded_int_lt head best with
     | True -> minimum_encoded_aux head rest
     | False -> minimum_encoded_aux best rest)

  (** val minimum_encoded_label : nat -> nat **)

  let minimum_encoded_label size =
    match labels size with
    | Nil -> O
    | Cons (first, rest) -> minimum_encoded_aux first rest

  (** val canonical_state : nat -> solver_state option **)

  let canonical_state size =
    match Nat.eqb size O with
    | True -> None
    | False ->
      process_sources size (S O) (sort_encoded (labels size))
        (initial_state size (minimum_encoded_label size))

  (** val lookup_successor :
      nat -> (nat, target) prod list -> target option **)

  let rec lookup_successor key = function
  | Nil -> None
  | Cons (p, rest) ->
    let Pair (candidate, value) = p in
    (match Nat.eqb key candidate with
     | True -> Some value
     | False -> lookup_successor key rest)

  (** val follow_aux :
      nat -> nat -> nat list -> (nat, target) prod list -> nat list -> nat
      list option **)

  let rec follow_aux fuel current visited edges acc =
    match fuel with
    | O -> Some (rev acc)
    | S remaining ->
      (match Nat.eqb current O with
       | True -> None
       | False ->
         (match member_nat current visited with
          | True -> None
          | False ->
            (match lookup_successor current edges with
             | Some t ->
               (match t with
                | Origin next ->
                  follow_aux remaining next (Cons (current, visited)) edges
                    (Cons (current, acc))
                | Complete ->
                  (match Nat.eqb remaining O with
                   | True -> Some (rev (Cons (current, acc)))
                   | False -> None))
             | None -> None)))

  (** val canonical_labels : nat -> nat list **)

  let canonical_labels size =
    match canonical_state size with
    | Some state ->
      (match follow_aux size state.st_entry Nil state.st_edges Nil with
       | Some path -> path
       | None -> Nil)
    | None -> Nil

  (** val canonical_successors : nat -> successor_table **)

  let canonical_successors size =
    match canonical_state size with
    | Some state -> state.st_edges
    | None -> Nil
 end

module PathCanon =
 struct
  (** val canonical_assignment : nat -> nat list **)

  let canonical_assignment =
    PathModel.canonical_labels

  (** val induced_successors : nat -> PathModel.successor_table **)

  let induced_successors =
    PathModel.canonical_successors
 end
