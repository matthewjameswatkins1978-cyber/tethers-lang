(** Reduced mathematical kernel for the accepted R3-3B2 path theorem. *)
From Stdlib Require Import List Arith Bool.
Import ListNotations.
Module PathModel.

Inductive target : Type := Origin : nat -> target | Complete : target.
Definition labels (n : nat) : list nat := seq 1 n.
Definition valid_label (n label : nat) : bool :=
  Nat.leb 1 label && Nat.leb label n.
Definition semantic_path (n : nat) : list nat := labels n.
Definition bijective_labelling (n : nat) (assignment : list nat) : Prop :=
  length assignment = n /\ NoDup assignment /\
  Forall (fun label => valid_label n label = true) assignment.
Definition entry_label (n : nat) : nat :=
  match labels n with [] => 0 | first :: _ => first end.
Fixpoint lookup_nat (key : nat) (values : list (nat * nat)) : option nat :=
  match values with
  | [] => None
  | (candidate, value) :: rest =>
      if Nat.eqb key candidate then Some value else lookup_nat key rest
  end.
Fixpoint update_nat (key value : nat) (values : list (nat * nat)) :
    list (nat * nat) :=
  match values with
  | [] => [(key, value)]
  | (candidate, old_value) :: rest =>
      if Nat.eqb key candidate then (key, value) :: rest
      else (candidate, old_value) :: update_nat key value rest
  end.
Fixpoint member_nat (key : nat) (values : list nat) : bool :=
  match values with
  | [] => false
  | head :: rest => Nat.eqb key head || member_nat key rest
  end.
Fixpoint digits_rev_fuel (fuel value : nat) : list nat :=
  match fuel with
  | 0 => []
  | S remaining => match value with
    | 0 => []
    | _ => (value mod 10) :: digits_rev_fuel remaining (value / 10)
    end
  end.
Definition digits_rev (value : nat) : list nat :=
  digits_rev_fuel (S value) value.
Definition decimal_digits (value : nat) : list nat := rev (digits_rev value).
Fixpoint compare_digits (left right : list nat) : comparison :=
  match left, right with
  | [], [] => Eq | [], _ :: _ => Gt | _ :: _, [] => Lt
  | lh :: lt, rh :: rt =>
      match Nat.compare lh rh with Eq => compare_digits lt rt | result => result end
  end.
Definition compare_encoded_int (left right : nat) : comparison :=
  compare_digits (decimal_digits left) (decimal_digits right).
Definition encoded_int_lt (left right : nat) : bool :=
  match compare_encoded_int left right with Lt => true | _ => false end.

Definition successor_table (n : nat) := list (nat * target).
Record solver_state : Type := {
  st_size : nat; st_entry : nat; st_edges : successor_table st_size;
  st_predecessors : list nat; st_parent : list (nat * nat);
  st_components : nat; st_terminal : option nat
}.
Definition make_state (size entry : nat) (edges : successor_table size)
    (predecessors : list nat) (parent : list (nat * nat))
    (components : nat) (terminal : option nat) : solver_state :=
  {| st_size := size; st_entry := entry; st_edges := edges;
     st_predecessors := predecessors; st_parent := parent;
     st_components := components; st_terminal := terminal |}.
Definition initial_state (size entry : nat) : solver_state :=
  make_state size entry [] []
    (map (fun label => (label, label)) (labels size)) size None.
Fixpoint dsu_find_fuel (fuel node : nat) (parent : list (nat * nat)) : nat :=
  match fuel with
  | 0 => node
  | S remaining =>
      match lookup_nat node parent with
      | None => node
      | Some root =>
          if Nat.eqb root node then node
          else dsu_find_fuel remaining root parent
      end
  end.
Definition dsu_find (size node : nat) (parent : list (nat * nat)) : nat :=
  dsu_find_fuel (S size) node parent.
Definition dsu_union (size left right : nat) (parent : list (nat * nat)) :
    option (list (nat * nat)) :=
  let left_root := dsu_find size left parent in
  let right_root := dsu_find size right parent in
  if Nat.eqb left_root right_root then None
  else Some (update_nat right_root left_root parent).
Definition state_with_edge (state : solver_state) (source : nat)
    (new_target : target) (new_predecessors : list nat)
    (new_parent : list (nat * nat)) (new_components : nat)
    (new_terminal : option nat) : solver_state :=
  make_state state.(st_size) state.(st_entry)
    ((source, new_target) :: state.(st_edges)) new_predecessors new_parent
    new_components new_terminal.
Definition try_origin (state : solver_state) (source target_label : nat) :
    option solver_state :=
  if valid_label state.(st_size) target_label then
    if Nat.eqb target_label state.(st_entry) then None
    else if member_nat target_label state.(st_predecessors) then None
    else if Nat.eqb source target_label then None
    else match dsu_union state.(st_size) source target_label state.(st_parent) with
    | None => None
    | Some parent' =>
        Some (state_with_edge state source (Origin target_label)
          (target_label :: state.(st_predecessors)) parent'
          (pred state.(st_components)) state.(st_terminal))
    end
  else None.
Definition try_complete (state : solver_state) (source : nat) :
    option solver_state :=
  match state.(st_terminal) with
  | Some _ => None
  | None => Some (state_with_edge state source Complete
      state.(st_predecessors) state.(st_parent) state.(st_components)
      (Some source))
  end.
Definition try_target (state : solver_state) (source : nat) (candidate : target) :
    option solver_state :=
  match candidate with
  | Origin target_label => try_origin state source target_label
  | Complete => try_complete state source
  end.
Definition partial_state_feasible (state : solver_state) (processed : nat) :
    bool :=
  if Nat.leb processed state.(st_size) then
    if valid_label state.(st_size) state.(st_entry) then
      match state.(st_terminal) with
      | Some terminal_source =>
          let terminal_root := dsu_find state.(st_size) terminal_source state.(st_parent) in
          let entry_root := dsu_find state.(st_size) state.(st_entry) state.(st_parent) in
          if Nat.eqb terminal_root entry_root then
            if Nat.ltb 1 state.(st_components) then false else
              if Nat.eqb processed state.(st_size) then
                Nat.eqb state.(st_components) 1 &&
                Nat.eqb entry_root (dsu_find state.(st_size) 1 state.(st_parent))
              else true
            else if Nat.eqb processed state.(st_size) then false else true
      | None => if Nat.eqb processed state.(st_size) then false else true
      end
    else false
  else false.
Fixpoint insert_encoded (label : nat) (sorted : list nat) : list nat :=
  match sorted with
  | [] => [label]
  | head :: rest =>
      if encoded_int_lt label head then label :: sorted
      else head :: insert_encoded label rest
  end.
Fixpoint sort_encoded (values : list nat) : list nat :=
  match values with [] => [] | head :: rest =>
    insert_encoded head (sort_encoded rest) end.
Definition ordered_candidates (state : solver_state) (ordered : list nat) : list target :=
  map Origin ordered ++
  match state.(st_terminal) with None => [Complete] | Some _ => [] end.
Fixpoint choose_candidate (fuel : nat) (state : solver_state) (source : nat)
    (processed : nat) (candidates : list target) : option solver_state :=
  match fuel with
  | 0 => None
  | S remaining => match candidates with
    | [] => None
    | candidate :: rest => match try_target state source candidate with
      | None => choose_candidate remaining state source processed rest
      | Some next_state =>
          if partial_state_feasible next_state processed then Some next_state
          else choose_candidate remaining state source processed rest
      end
    end
  end.
Fixpoint process_sources (fuel source : nat) (ordered : list nat)
    (state : solver_state) : option solver_state :=
  match fuel with
  | 0 => Some state
  | S remaining =>
      if Nat.leb source state.(st_size) then
        match choose_candidate (S state.(st_size)) state source source
            (ordered_candidates state ordered) with
        | None => None
        | Some next_state => process_sources remaining (S source) ordered next_state
        end
      else Some state
  end.
Fixpoint minimum_encoded_aux (best : nat) (values : list nat) : nat :=
  match values with
  | [] => best
  | head :: rest =>
      if encoded_int_lt head best then minimum_encoded_aux head rest
      else minimum_encoded_aux best rest
  end.
Definition minimum_encoded_label (size : nat) : nat :=
  match labels size with [] => 0 | first :: rest =>
    minimum_encoded_aux first rest end.
Definition canonical_state (size : nat) : option solver_state :=
  if Nat.eqb size 0 then None
  else process_sources size 1 (sort_encoded (labels size))
    (initial_state size (minimum_encoded_label size)).
Fixpoint lookup_successor (key : nat) (values : list (nat * target)) :
    option target :=
  match values with
  | [] => None
  | (candidate, value) :: rest =>
      if Nat.eqb key candidate then Some value
      else lookup_successor key rest
  end.
Fixpoint follow_aux (fuel current : nat) (visited : list nat)
    (edges : list (nat * target)) (acc : list nat) : option (list nat) :=
  match fuel with
  | 0 => Some (rev acc)
  | S remaining =>
      if Nat.eqb current 0 then None
      else if member_nat current visited then None
      else match lookup_successor current edges with
      | None => None
      | Some (Origin next) =>
          follow_aux remaining next (current :: visited) edges (current :: acc)
      | Some Complete =>
          if Nat.eqb remaining 0 then Some (rev (current :: acc)) else None
      end
  end.
Definition canonical_labels (size : nat) : list nat :=
  match canonical_state size with
  | None => []
  | Some state =>
      match follow_aux size state.(st_entry) [] state.(st_edges) [] with
      | None => [] | Some path => path end
  end.
Definition canonical_successors (size : nat) : successor_table size :=
  match canonical_state size with None => [] | Some state => state.(st_edges) end.
End PathModel.
