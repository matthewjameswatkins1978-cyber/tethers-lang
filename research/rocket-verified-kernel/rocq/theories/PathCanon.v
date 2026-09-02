(** Rocket Verified Kernel — Experiment 1
    Executable canonical path algorithm.

    Definitions placed in Type/Set and used computationally should be suitable
    for Rocq extraction to OCaml.  Keep proof-only material out of the
    computational interface where practical.
*)

From RocketVerifiedKernel Require Import PathModel.

Module PathCanon.

Import PathModel.
Definition canonical_assignment (size : nat) : list nat := canonical_labels size.

Definition induced_successors (size : nat) : successor_table size := canonical_successors size.

Definition assignment_is_bijective (size : nat) : Prop := bijective_labelling size (canonical_assignment size).
Definition canonical_entry_label (size : nat) : nat := minimum_encoded_label size.
Definition entry_is_legal (size : nat) : Prop := valid_label size (canonical_entry_label size) = true.
Definition continuation_order (left right : nat) : Prop := encoded_int_lt left right = true.

End PathCanon.
