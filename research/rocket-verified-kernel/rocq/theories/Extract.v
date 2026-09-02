(** Rocket Verified Kernel — Experiment 1
    Extraction boundary.

    Generated OCaml belongs under ../../extracted/ and must not be hand edited.
*)

From Corelib Require Extraction.
From RocketVerifiedKernel Require Import PathCanon PathProofs.

Extraction Language OCaml.

Extraction "../../extracted/rocket_path_kernel.ml" PathCanon.canonical_assignment PathCanon.induced_successors.
Print Assumptions PathProofs.canonical_result_unique.

