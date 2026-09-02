(** Stage 0 extraction smoke test.  The generated file is disposable evidence. *)

From Corelib Require Import Extraction.
From RocketVerifiedKernel Require Import Stage0.

Extraction Language OCaml.
Extraction "../../extracted/stage0_kernel.ml" Stage0.successor.
