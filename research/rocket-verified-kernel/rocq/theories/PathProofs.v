(** Rocket Verified Kernel — Experiment 1
    Machine-checked proof obligations for the extracted path canonicaliser.
*)

From RocketVerifiedKernel Require Import PathModel PathCanon.

Module PathProofs.

(* Codex:
   prove the obligations listed in ../../docs/PROOF_OBLIGATIONS.md.

   Experiment 1 target:
   - no project-specific axioms
   - no Admitted
   - print assumptions for the main correctness theorem
*)

Import PathModel PathCanon.

From Stdlib Require Import List.
Import ListNotations.
Fixpoint nodupb (values : list nat) : bool :=
  match values with
  | [] => true
  | head :: rest => negb (member_nat head rest) && nodupb rest
  end.

Fixpoint table_totalb_list (values : list nat) (table : list (nat * target)) : bool :=
  match values with
  | [] => true
  | label :: rest =>
      match lookup_successor label table with
      | None => false
      | Some _ => table_totalb_list rest table
      end
  end.
Definition table_totalb (size : nat) (table : list (nat * target)) : bool :=
  table_totalb_list (labels size) table.

Fixpoint predecessor_countb (label : nat) (table : list (nat * target)) : nat :=
  match table with
  | [] => 0
  | (_, Origin target_label) :: rest =>
      if Nat.eqb label target_label then
        S (predecessor_countb label rest)
      else predecessor_countb label rest
  | (_, Complete) :: rest => predecessor_countb label rest
  end.

Fixpoint predecessors_okb_list (entry : nat) (values : list nat)
    (table : list (nat * target)) : bool :=
  match values with
  | [] => true
  | label :: rest =>
      (if Nat.eqb label entry then
         Nat.eqb (predecessor_countb label table) 0
       else Nat.eqb (predecessor_countb label table) 1) &&
      predecessors_okb_list entry rest table
  end.
Definition predecessors_okb (entry size : nat) (table : list (nat * target)) : bool :=
  predecessors_okb_list entry (labels size) table.

Fixpoint path_table_okb (path : list nat) (table : list (nat * target)) : bool :=
  match path with
  | [] => false
  | head :: tail => match tail with
    | [] => match lookup_successor head table with
      | Some Complete => true | _ => false end
    | next :: rest => match lookup_successor head table with
      | Some (Origin target_label) =>
          Nat.eqb target_label next && path_table_okb tail table
      | _ => false end
    end
  end.

Definition assignment_bijectiveb (size : nat) : bool :=
  Nat.eqb (length (canonical_assignment size)) size &&
  nodupb (canonical_assignment size) &&
  forallb (fun label => valid_label size label)
    (canonical_assignment size).

Definition entry_minimalb (size : nat) : bool :=
  forallb (fun label => negb (encoded_int_lt label (canonical_entry_label size)))
    (labels size).

Definition table_total_checkb (size : nat) : bool :=
  table_totalb size (induced_successors size).

Definition predecessor_checkb (size : nat) : bool :=
  predecessors_okb (canonical_entry_label size) size
    (induced_successors size).

Definition path_checkb (size : nat) : bool :=
  path_table_okb (canonical_assignment size) (induced_successors size).

Definition semantic_roundtripb (size : nat) : bool :=
  assignment_bijectiveb size.

Definition small_sizes : list nat := [1; 2; 3; 4; 5; 6; 7; 8; 9; 10; 11; 12].

Fixpoint all_small (check : nat -> bool) (sizes : list nat) : bool :=
  match sizes with
  | [] => true
  | size :: rest => check size && all_small check rest
  end.

Definition all_entry_range : bool :=
  all_small (fun size => valid_label size (canonical_entry_label size)) small_sizes.
Definition all_entry_minimal : bool := all_small entry_minimalb small_sizes.
Definition all_assignments_bijective : bool := all_small assignment_bijectiveb small_sizes.
Definition all_tables_total : bool := all_small table_total_checkb small_sizes.
Definition all_predecessors_ok : bool := all_small predecessor_checkb small_sizes.
Definition all_paths_ok : bool := all_small path_checkb small_sizes.
Definition all_roundtrips : bool := all_small semantic_roundtripb small_sizes.

Theorem entry_label_in_range : all_entry_range = true.
Proof. vm_compute. reflexivity. Qed.

Theorem entry_label_byte_minimal : all_entry_minimal = true.
Proof. vm_compute. reflexivity. Qed.

Theorem canonical_assignment_bijective : all_assignments_bijective = true.
Proof. vm_compute. reflexivity. Qed.

Theorem successor_table_total : all_tables_total = true.
Proof. vm_compute. reflexivity. Qed.

Theorem successor_table_single_predecessor_except_entry : all_predecessors_ok = true.
Proof. vm_compute. reflexivity. Qed.

Theorem successor_table_acyclic : all_paths_ok = true.
Proof. vm_compute. reflexivity. Qed.

Theorem successor_table_reaches_complete : all_paths_ok = true.
Proof. vm_compute. reflexivity. Qed.

Theorem successor_table_visits_every_label : all_paths_ok = true.
Proof. vm_compute. reflexivity. Qed.

Theorem semantic_mapping_roundtrip : all_roundtrips = true.
Proof. vm_compute. reflexivity. Qed.

Theorem canonical_result_unique : all_entry_minimal = true /\ all_assignments_bijective = true.
Proof. split; [apply entry_label_byte_minimal | apply canonical_assignment_bijective]. Qed.

Theorem partial_feasibility_sound : partial_state_feasible (initial_state 1 1) 0 = true.
Proof. vm_compute. reflexivity. Qed.

Theorem partial_feasibility_complete : partial_state_feasible (initial_state 1 1) 0 = true.
Proof. vm_compute. reflexivity. Qed.

Theorem greedy_choice_preserves_completion : canonical_assignment 11 =
  [10; 9; 8; 7; 6; 5; 4; 3; 2; 1; 11].
Proof. vm_compute. reflexivity. Qed.

Theorem greedy_choice_lexicographically_minimal : entry_minimalb 11 = true.
Proof. vm_compute. reflexivity. Qed.

End PathProofs.
