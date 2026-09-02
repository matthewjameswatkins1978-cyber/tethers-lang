From Corelib Require Extraction.
From Stdlib Require Import ExtrOcamlNatInt.
From RocketVerifiedKernel Require Import PathCanon PathProofs.

Extraction Language OCaml.

Extraction "../../extracted/rocket_path_kernel_native.ml"
  PathCanon.canonical_assignment PathCanon.induced_successors.
Print Assumptions PathProofs.canonical_result_unique.
