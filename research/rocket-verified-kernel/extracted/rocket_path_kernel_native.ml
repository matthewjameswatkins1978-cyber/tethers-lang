
(** val fst : ('a1 * 'a2) -> 'a1 **)

let fst = function
| (x, _) -> x

(** val snd : ('a1 * 'a2) -> 'a2 **)

let snd = function
| (_, y) -> y

(** val app : 'a1 list -> 'a1 list -> 'a1 list **)

let rec app l m =
  match l with
  | [] -> m
  | a :: l1 -> a :: (app l1 m)

type comparison =
| Eq
| Lt
| Gt

(** val pred : int -> int **)

let pred = fun n -> Stdlib.max 0 (n-1)

module Nat =
 struct
  (** val sub : int -> int -> int **)

  let rec sub n m =
    (fun fO fS n -> if n=0 then fO () else fS (n-1))
      (fun _ -> n)
      (fun k ->
      (fun fO fS n -> if n=0 then fO () else fS (n-1))
        (fun _ -> n)
        (fun l -> sub k l)
        m)
      n

  (** val ltb : int -> int -> bool **)

  let ltb n m =
    (<=) (Stdlib.Int.succ n) m

  (** val compare : int -> int -> comparison **)

  let rec compare = fun n m -> if n=m then Eq else if n<m then Lt else Gt

  (** val divmod : int -> int -> int -> int -> int * int **)

  let rec divmod x y q u =
    (fun fO fS n -> if n=0 then fO () else fS (n-1))
      (fun _ -> (q, u))
      (fun x' ->
      (fun fO fS n -> if n=0 then fO () else fS (n-1))
        (fun _ -> divmod x' y (Stdlib.Int.succ q) y)
        (fun u' -> divmod x' y q u')
        u)
      x

  (** val div : int -> int -> int **)

  let div x y =
    (fun fO fS n -> if n=0 then fO () else fS (n-1))
      (fun _ -> y)
      (fun y' -> fst (divmod x y' 0 y'))
      y

  (** val modulo : int -> int -> int **)

  let modulo x y =
    (fun fO fS n -> if n=0 then fO () else fS (n-1))
      (fun _ -> x)
      (fun y' -> sub y' (snd (divmod x y' 0 y')))
      y
 end

(** val map : ('a1 -> 'a2) -> 'a1 list -> 'a2 list **)

let rec map f = function
| [] -> []
| a :: l0 -> (f a) :: (map f l0)

(** val seq : int -> int -> int list **)

let rec seq start len =
  (fun fO fS n -> if n=0 then fO () else fS (n-1))
    (fun _ -> [])
    (fun len0 -> start :: (seq (Stdlib.Int.succ start) len0))
    len

(** val rev : 'a1 list -> 'a1 list **)

let rec rev = function
| [] -> []
| x :: l' -> app (rev l') (x :: [])

module PathModel =
 struct
  type target =
  | Origin of int
  | Complete

  (** val labels : int -> int list **)

  let labels n =
    seq (Stdlib.Int.succ 0) n

  (** val valid_label : int -> int -> bool **)

  let valid_label n label =
    (&&) ((<=) (Stdlib.Int.succ 0) label) ((<=) label n)

  (** val lookup_nat : int -> (int * int) list -> int option **)

  let rec lookup_nat key = function
  | [] -> None
  | p :: rest ->
    let (candidate, value) = p in
    if (=) key candidate then Some value else lookup_nat key rest

  (** val update_nat : int -> int -> (int * int) list -> (int * int) list **)

  let rec update_nat key value = function
  | [] -> (key, value) :: []
  | p :: rest ->
    let (candidate, old_value) = p in
    if (=) key candidate
    then (key, value) :: rest
    else (candidate, old_value) :: (update_nat key value rest)

  (** val member_nat : int -> int list -> bool **)

  let rec member_nat key = function
  | [] -> false
  | head :: rest -> (||) ((=) key head) (member_nat key rest)

  (** val digits_rev_fuel : int -> int -> int list **)

  let rec digits_rev_fuel fuel value =
    (fun fO fS n -> if n=0 then fO () else fS (n-1))
      (fun _ -> [])
      (fun remaining ->
      (fun fO fS n -> if n=0 then fO () else fS (n-1))
        (fun _ -> [])
        (fun _ ->
        (Nat.modulo value (Stdlib.Int.succ (Stdlib.Int.succ (Stdlib.Int.succ
          (Stdlib.Int.succ (Stdlib.Int.succ (Stdlib.Int.succ (Stdlib.Int.succ
          (Stdlib.Int.succ (Stdlib.Int.succ (Stdlib.Int.succ 0))))))))))) :: 
        (digits_rev_fuel remaining
          (Nat.div value (Stdlib.Int.succ (Stdlib.Int.succ (Stdlib.Int.succ
            (Stdlib.Int.succ (Stdlib.Int.succ (Stdlib.Int.succ
            (Stdlib.Int.succ (Stdlib.Int.succ (Stdlib.Int.succ
            (Stdlib.Int.succ 0)))))))))))))
        value)
      fuel

  (** val digits_rev : int -> int list **)

  let digits_rev value =
    digits_rev_fuel (Stdlib.Int.succ value) value

  (** val decimal_digits : int -> int list **)

  let decimal_digits value =
    rev (digits_rev value)

  (** val compare_digits : int list -> int list -> comparison **)

  let rec compare_digits left right =
    match left with
    | [] -> (match right with
             | [] -> Eq
             | _ :: _ -> Gt)
    | lh :: lt ->
      (match right with
       | [] -> Lt
       | rh :: rt ->
         (match Nat.compare lh rh with
          | Eq -> compare_digits lt rt
          | x -> x))

  (** val compare_encoded_int : int -> int -> comparison **)

  let compare_encoded_int left right =
    compare_digits (decimal_digits left) (decimal_digits right)

  (** val encoded_int_lt : int -> int -> bool **)

  let encoded_int_lt left right =
    match compare_encoded_int left right with
    | Lt -> true
    | _ -> false

  type successor_table = (int * target) list

  type solver_state = { st_size : int; st_entry : int;
                        st_edges : successor_table;
                        st_predecessors : int list;
                        st_parent : (int * int) list; st_components : 
                        int; st_terminal : int option }

  (** val st_size : solver_state -> int **)

  let st_size s =
    s.st_size

  (** val st_entry : solver_state -> int **)

  let st_entry s =
    s.st_entry

  (** val st_edges : solver_state -> successor_table **)

  let st_edges s =
    s.st_edges

  (** val st_predecessors : solver_state -> int list **)

  let st_predecessors s =
    s.st_predecessors

  (** val st_parent : solver_state -> (int * int) list **)

  let st_parent s =
    s.st_parent

  (** val st_components : solver_state -> int **)

  let st_components s =
    s.st_components

  (** val st_terminal : solver_state -> int option **)

  let st_terminal s =
    s.st_terminal

  (** val make_state :
      int -> int -> successor_table -> int list -> (int * int) list -> int ->
      int option -> solver_state **)

  let make_state size entry edges predecessors parent components terminal =
    { st_size = size; st_entry = entry; st_edges = edges; st_predecessors =
      predecessors; st_parent = parent; st_components = components;
      st_terminal = terminal }

  (** val initial_state : int -> int -> solver_state **)

  let initial_state size entry =
    make_state size entry [] []
      (map (fun label -> (label, label)) (labels size)) size None

  (** val dsu_find_fuel : int -> int -> (int * int) list -> int **)

  let rec dsu_find_fuel fuel node parent =
    (fun fO fS n -> if n=0 then fO () else fS (n-1))
      (fun _ -> node)
      (fun remaining ->
      match lookup_nat node parent with
      | Some root ->
        if (=) root node then node else dsu_find_fuel remaining root parent
      | None -> node)
      fuel

  (** val dsu_find : int -> int -> (int * int) list -> int **)

  let dsu_find size node parent =
    dsu_find_fuel (Stdlib.Int.succ size) node parent

  (** val dsu_union :
      int -> int -> int -> (int * int) list -> (int * int) list option **)

  let dsu_union size left right parent =
    let left_root = dsu_find size left parent in
    let right_root = dsu_find size right parent in
    if (=) left_root right_root
    then None
    else Some (update_nat right_root left_root parent)

  (** val state_with_edge :
      solver_state -> int -> target -> int list -> (int * int) list -> int ->
      int option -> solver_state **)

  let state_with_edge state source new_target new_predecessors new_parent new_components new_terminal =
    make_state state.st_size state.st_entry ((source,
      new_target) :: state.st_edges) new_predecessors new_parent
      new_components new_terminal

  (** val try_origin : solver_state -> int -> int -> solver_state option **)

  let try_origin state source target_label =
    if valid_label state.st_size target_label
    then if (=) target_label state.st_entry
         then None
         else if member_nat target_label state.st_predecessors
              then None
              else if (=) source target_label
                   then None
                   else (match dsu_union state.st_size source target_label
                                 state.st_parent with
                         | Some parent' ->
                           Some
                             (state_with_edge state source (Origin
                               target_label)
                               (target_label :: state.st_predecessors)
                               parent' (pred state.st_components)
                               state.st_terminal)
                         | None -> None)
    else None

  (** val try_complete : solver_state -> int -> solver_state option **)

  let try_complete state source =
    match state.st_terminal with
    | Some _ -> None
    | None ->
      Some
        (state_with_edge state source Complete state.st_predecessors
          state.st_parent state.st_components (Some source))

  (** val try_target :
      solver_state -> int -> target -> solver_state option **)

  let try_target state source = function
  | Origin target_label -> try_origin state source target_label
  | Complete -> try_complete state source

  (** val partial_state_feasible : solver_state -> int -> bool **)

  let partial_state_feasible state processed =
    if (<=) processed state.st_size
    then if valid_label state.st_size state.st_entry
         then (match state.st_terminal with
               | Some terminal_source ->
                 let terminal_root =
                   dsu_find state.st_size terminal_source state.st_parent
                 in
                 let entry_root =
                   dsu_find state.st_size state.st_entry state.st_parent
                 in
                 if (=) terminal_root entry_root
                 then if Nat.ltb (Stdlib.Int.succ 0) state.st_components
                      then false
                      else if (=) processed state.st_size
                           then (&&)
                                  ((=) state.st_components (Stdlib.Int.succ
                                    0))
                                  ((=) entry_root
                                    (dsu_find state.st_size (Stdlib.Int.succ
                                      0) state.st_parent))
                           else true
                 else if (=) processed state.st_size then false else true
               | None -> if (=) processed state.st_size then false else true)
         else false
    else false

  (** val insert_encoded : int -> int list -> int list **)

  let rec insert_encoded label sorted = match sorted with
  | [] -> label :: []
  | head :: rest ->
    if encoded_int_lt label head
    then label :: sorted
    else head :: (insert_encoded label rest)

  (** val sort_encoded : int list -> int list **)

  let rec sort_encoded = function
  | [] -> []
  | head :: rest -> insert_encoded head (sort_encoded rest)

  (** val ordered_candidates : solver_state -> int list -> target list **)

  let ordered_candidates state ordered =
    app (map (fun x -> Origin x) ordered)
      (match state.st_terminal with
       | Some _ -> []
       | None -> Complete :: [])

  (** val choose_candidate :
      int -> solver_state -> int -> int -> target list -> solver_state option **)

  let rec choose_candidate fuel state source processed candidates =
    (fun fO fS n -> if n=0 then fO () else fS (n-1))
      (fun _ -> None)
      (fun remaining ->
      match candidates with
      | [] -> None
      | candidate :: rest ->
        (match try_target state source candidate with
         | Some next_state ->
           if partial_state_feasible next_state processed
           then Some next_state
           else choose_candidate remaining state source processed rest
         | None -> choose_candidate remaining state source processed rest))
      fuel

  (** val process_sources :
      int -> int -> int list -> solver_state -> solver_state option **)

  let rec process_sources fuel source ordered state =
    (fun fO fS n -> if n=0 then fO () else fS (n-1))
      (fun _ -> Some state)
      (fun remaining ->
      if (<=) source state.st_size
      then (match choose_candidate (Stdlib.Int.succ state.st_size) state
                    source source (ordered_candidates state ordered) with
            | Some next_state ->
              process_sources remaining (Stdlib.Int.succ source) ordered
                next_state
            | None -> None)
      else Some state)
      fuel

  (** val minimum_encoded_aux : int -> int list -> int **)

  let rec minimum_encoded_aux best = function
  | [] -> best
  | head :: rest ->
    if encoded_int_lt head best
    then minimum_encoded_aux head rest
    else minimum_encoded_aux best rest

  (** val minimum_encoded_label : int -> int **)

  let minimum_encoded_label size =
    match labels size with
    | [] -> 0
    | first :: rest -> minimum_encoded_aux first rest

  (** val canonical_state : int -> solver_state option **)

  let canonical_state size =
    if (=) size 0
    then None
    else process_sources size (Stdlib.Int.succ 0)
           (sort_encoded (labels size))
           (initial_state size (minimum_encoded_label size))

  (** val lookup_successor : int -> (int * target) list -> target option **)

  let rec lookup_successor key = function
  | [] -> None
  | p :: rest ->
    let (candidate, value) = p in
    if (=) key candidate then Some value else lookup_successor key rest

  (** val follow_aux :
      int -> int -> int list -> (int * target) list -> int list -> int list
      option **)

  let rec follow_aux fuel current visited edges acc =
    (fun fO fS n -> if n=0 then fO () else fS (n-1))
      (fun _ -> Some (rev acc))
      (fun remaining ->
      if (=) current 0
      then None
      else if member_nat current visited
           then None
           else (match lookup_successor current edges with
                 | Some t ->
                   (match t with
                    | Origin next ->
                      follow_aux remaining next (current :: visited) edges
                        (current :: acc)
                    | Complete ->
                      if (=) remaining 0
                      then Some (rev (current :: acc))
                      else None)
                 | None -> None))
      fuel

  (** val canonical_labels : int -> int list **)

  let canonical_labels size =
    match canonical_state size with
    | Some state ->
      (match follow_aux size state.st_entry [] state.st_edges [] with
       | Some path -> path
       | None -> [])
    | None -> []

  (** val canonical_successors : int -> successor_table **)

  let canonical_successors size =
    match canonical_state size with
    | Some state -> state.st_edges
    | None -> []
 end

module PathCanon =
 struct
  (** val canonical_assignment : int -> int list **)

  let canonical_assignment =
    PathModel.canonical_labels

  (** val induced_successors : int -> PathModel.successor_table **)

  let induced_successors =
    PathModel.canonical_successors
 end
