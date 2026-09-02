(** Stage 0 executable smoke test for the independent Rocq environment. *)

From Stdlib Require Import Arith.

Module Stage0.

Definition successor (n : nat) : nat := S n.

Theorem successor_spec : forall n, successor n = S n.
Proof.
  intro n.
  reflexivity.
Qed.

Print Assumptions successor_spec.

End Stage0.
