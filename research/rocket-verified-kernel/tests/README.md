# Differential Harness

Experiment 1 compares three authorities:

1. accepted hand-written R3-3B2 OCaml;
2. Rocq-extracted OCaml kernel;
3. frozen Enc_V2 exact/oracle result.

Required initial fixtures:

- chain 1..11;
- known chain-11 label sequence;
- decimal boundaries 9/10/11/12;
- 99/100;
- 999/1000;
- raw-ID renaming;
- storage reversal/permutation.

The test harness is research-only and must not alter production call paths.

Where possible compare:

- label assignment;
- numeric successor table;
- relevant frozen continuation bytes;
- complete payload/digest.

If a comparison cannot reach full payload/digest without widening the proof boundary, document that boundary instead of pretending it is verified.
