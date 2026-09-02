# Experiment 1 Proof Obligations

The first experiment formalises only the accepted R3-3B2 simple success-path theorem.

## Mathematical kernel

Let N > 0.

A legal labelling is a bijection between semantic path positions and integer labels 1..N.

The distinguished entry semantic vertex is assigned the frozen byte-minimal legal entry label.

A numeric successor table assigns every source label exactly one target:
- another Origin label, or
- ProgramComplete.

Legal completion requires one Hamiltonian path beginning at entry, visiting every label exactly once, and terminating at ProgramComplete.

## Required theorems

- entry_label_in_range
- entry_label_byte_minimal
- canonical_assignment_bijective
- successor_table_total
- successor_table_single_predecessor_except_entry
- successor_table_acyclic
- successor_table_reaches_complete
- successor_table_visits_every_label
- partial_feasibility_sound
- partial_feasibility_complete
- greedy_choice_preserves_completion
- greedy_choice_lexicographically_minimal
- canonical_result_unique
- semantic_mapping_roundtrip

## Trust boundary

Experiment 1 does not initially formalise SHA-256, the entire frozen Enc_V2 encoder, Tethers Core validation, OCaml runtime semantics, or the adapter from production Core values.

The formal theorem should cover the exact path-label/successor problem. An OCaml differential harness must then prove empirically that the research serializer representation corresponds to the relevant frozen Enc_V2 continuation bytes on the required corpus.

Every boundary must be explicit. No “verified” wording may silently include code outside the proved/extracted kernel.
