(** Rocket Verified Kernel — Experiment 1
    Extraction boundary.

    Generated OCaml belongs under ../../extracted/ and must not be hand edited.
*)

From Corelib Require Extraction.
From RocketVerifiedKernel Require Import PathCanon PathProofs.

Extraction Language OCaml.

(* Codex:
   once the executable canonical entry point is final, extract only the
   minimal computational definitions required by the research harness.

   Example shape:
   Extraction "../../extracted/rocket_path_kernel.ml" PathCanon.<entrypoint>.
*)

Print Assumptions PathProofs.

