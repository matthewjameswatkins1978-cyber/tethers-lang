# Worker Note: C-B1 — Refinement Algorithm Proof

Task: `C-B1 — REFINEMENT ALGORITHM PROOF`

Task packet: user-provided C-B1 packet in chat

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `42efc3431e96e1de229bf055b7170d8622d801f1`

Implementation checkpoint: `42efc3431e96e1de229bf055b7170d8622d801f1` (no production code changes)

## Requested outcome

Design the smallest stronger refinement strategy that removes the
high-symmetry round-count cliff while guaranteeing the exact same final
canonical bytes and ProgramDigest as the accepted canonicaliser. No production
implementation in C-B1.

## Changes made

- `tethers-0.1/engine-ocaml/bin/tethers_cb1_benchmark.ml` — NEW benchmark prototype (benchmark-only, not production)
- `tethers-0.1/engine-ocaml/bin/tethers_core_canonical.mli` — exposed pipeline functions for benchmarking (assign_canonical_ids, build_canonical_program, make_canonical_bytes, compute_sha256, make_program_digest, colour_map type, StringMap module)
- `tethers-0.1/engine-ocaml/bin/dune` — added executable entry for benchmark

No other production files changed.

## Decisions and assumptions

1. **Topological multi-pass strategy selected** as the strongest candidate.
   Processes origins in BFS order with a running colour accumulator so each
   origin sees the latest colours of its predecessors within the same round.

2. **Three strategies benchmarked**: (A) current, (B) topological with
   provisional colours, (C) topological with exact colours (previous round's
   colours only).

3. **Frozen contract comparison**: canonical bytes + ProgramDigest, not just
   partition equivalence. The benchmark now runs the full canonicalization
   pipeline (colours → IDs → program → bytes → SHA-256) for both strategies.

## Evidence

### Root cause analysis

Sequential chain of n identical actions requires n/2+1 refinement rounds
because colour information propagates one hop per round through the
`success_in_map` dependency edges.

### Benchmark results

High-symmetry (identical actions):
- Size 100: 51→2 rounds, 11936→802µs (14.9×)
- Size 250: 126→2 rounds, 82307→2600µs (31.7×)
- Size 500: 251→2 rounds, 367025→6250µs (58.7×)
- Size 1000: 501→2 rounds, 1702722→15684µs (108.6×)

Low-symmetry (250 distinct actions): 2→2 rounds, same partition, but
canonical bytes and ProgramDigest STILL differ.

### Frozen contract comparison

| Check | High-symmetry | Low-symmetry |
|-------|--------------|--------------|
| Partition equivalence | PASS | PASS |
| Colour ordering | FAIL | FAIL |
| Canonical bytes | FAIL | FAIL |
| ProgramDigest | FAIL | FAIL |

### Why canonical bytes differ

The mechanism is:
1. Topological strategy produces different colour values (running accumulator
   updates within a round)
2. Different colour values → different sort order in assign_canonical_ids
3. Different sort order → different canonical IDs
4. Different canonical IDs → different canonical bytes
5. Different canonical bytes → different ProgramDigest

This occurs even for low-symmetry programs with identical round counts.

### Topological with exact colours

Same rounds as current (51/126/251/501), 1.5-1.7× SLOWER. No convergence
benefit.

### Incremental dirty tracking

Cannot reduce the O(n) round count. Can only skip unchanged entity types
(constant factor). For the sequential chain, all origin signatures must be
recomputed every round (per-round work remains O(n)). The round count is
bounded by graph diameter, which is structural.

### Commands run
- `dune build bin/tethers_cb1_benchmark.exe` — PASS
- `dune exec bin/tethers_cb1_benchmark.exe` — results recorded above
- `dune runtest --force` — PASS (all existing tests green)

## Discoveries

1. The topological strategy produces the same partition but different colour
   values at ALL tested sizes, including low-symmetry programs.

2. The colour value difference is fundamental: within-round accumulator
   updates produce different signature strings than previous-round-only
   approach, even when the final partition is identical.

3. `assign_canonical_ids` sorts by (colour, id), so different colour values
   produce different canonical IDs, which produce different canonical bytes.

4. Incremental dirty tracking cannot reduce the O(n) round count for the
   sequential chain. The per-round work remains O(n) because all origin
   signatures must be recomputed to detect changes.

5. The graph diameter is the fundamental lower bound on round count for
   any parallel colour-refinement algorithm.

## Remaining risks

- The benchmark prototype remains in the dune file. It should be removed
  before any production work.
- The .mli file exposes internal pipeline functions for benchmarking only.

## Smallest next action

Await Lucy review of the C-B1 design note and recommendation.

## References

- Design note: `docs/C-B1_DESIGN_NOTE.md`
- Benchmark prototype: `tethers-0.1/engine-ocaml/bin/tethers_cb1_benchmark.ml`
- Changed files: `tethers-0.1/engine-ocaml/bin/dune`, `tethers-0.1/engine-ocaml/bin/tethers_core_canonical.mli`
- PF1 forensics: `docs/performance/PF1_FORENSICS.md`
- C-A results: `docs/performance/core-phase-a/RESULT.md`
