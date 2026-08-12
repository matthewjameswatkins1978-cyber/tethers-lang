# C-B1 — Refinement Algorithm Proof

Date: 2026-08-12
Branch: `perf/c-core-cheap-structural-fixes`
Base: `42efc3431e96e1de229bf055b7170d8622d801f1`
Author: OpenCode
Task packet: user-provided C-B1 packet in chat

## 1. Root Cause: Why O(n) Rounds for High-Symmetry Chains

The current refinement algorithm (`tethers_core_canonical.ml`) processes all
entity types in a single batch per round. Each round computes signatures using
colours from the **previous round only**. Colour information propagates through
dependency edges at one hop per round.

### The sequential chain benchmark

For n identical actions chained through `success_continuations`:

```
O1 -> O2 -> O3 -> ... -> On
```

Round 0: All origins get colour 1 (identical scalar signature:
`"Ac:" ^ capability_id ^ ":" ^ contract_digest`).

Round 1: O1's signature changes (empty `:in=` — no predecessors). O2..On stay
the same. One origin distinguished.

Round 2: O2's signature changes (`:in=C:2` — O1's new colour). O3..On stay.
Two origins distinguished.

...Round n: On finally distinguished. **Converged.**

The bottleneck is the `success_in_map` dependency: each origin's signature
includes the colours of its predecessors. Colour information propagates one hop
per round because signatures use only the previous round's colours.

### Why this is fundamental

Any colour-refinement algorithm that:
1. uses only the previous round's colours as input to signatures, and
2. processes all entities in parallel (no within-round colour updates)

will require O(diameter) rounds for colour information to propagate through a
chain of that diameter. For the sequential chain benchmark, diameter = n/2.

The number of rounds is n/2+1 (entry origin + n/2 propagation steps).

## 2. Proposed Strategy: Topological Multi-Pass

Process entity types in dependency order within each round, and within origins,
process them in topological order (predecessors before successors). Update
colours as we go, so each origin sees the latest colours of its predecessors.

### Algorithm

```
refine_round_topological(prev, refs, program):
  1. Compute fact signatures using prev colours → compress → fact_colours
  2. Process origins in BFS order from entry:
     - Compute signature using running colour accumulator
     - Update accumulator immediately
  3. Compress accumulated origin signatures → origin_colours
  4. Compute batch/branch/role/item_template signatures using updated colours
  5. Return new colour_map
```

### Key modification

The running accumulator uses **provisional colours** (hash of signature string)
so that subsequent origins in topological order can distinguish predecessors
that have already been processed in this round.

## 3. Benchmark Results

### High-symmetry (identical actions, sequential chain)

| Size | Current Rounds | Topo Rounds | Current µs | Topo µs | Speedup |
|------|---------------|-------------|------------|---------|---------|
| 100  | 51            | 2           | 11,936     | 802     | 14.9×   |
| 250  | 126           | 2           | 82,307     | 2,600   | 31.7×   |
| 500  | 251           | 2           | 367,025    | 6,250   | 58.7×   |
| 1000 | 501           | 2           | 1,702,722  | 15,684  | 108.6×  |

### Low-symmetry (distinct literals)

| Size | Current Rounds | Topo Rounds | PartEq | OrdEq | CanonEq |
|------|---------------|-------------|--------|-------|---------|
| 250  | 2             | 2           | PASS   | FAIL  | FAIL    |

Low-symmetry programs converge in 2 rounds regardless of strategy (each
origin has a distinct scalar signature from round 0), but the topological
strategy still produces different colour values due to within-round
accumulator updates.

### Topological with exact colours (no provisional colours)

| Size | Rounds | µs     | vs Current |
|------|--------|--------|------------|
| 100  | 51     | 17,902 | 1.5× slower |
| 250  | 126    | 131,785| 1.6× slower |
| 500  | 251    | 605,513| 1.6× slower |
| 1000 | 501    | 2,849,544| 1.7× slower |

When the topological strategy uses the previous round's colours for ALL
signatures (preserving exact colour values), it provides **no convergence
benefit** and is actually **slower** due to topological sorting overhead.

## 4. Frozen Contract Comparison

The frozen acceptance contract is: **canonical bytes + ProgramDigest**.

| Check | High-symmetry (all sizes) | Low-symmetry (250) |
|-------|--------------------------|---------------------|
| Partition equivalence | PASS | PASS |
| Colour ordering | FAIL | FAIL |
| Canonical bytes | FAIL | FAIL |
| ProgramDigest | FAIL | FAIL |

**Detailed diagnostic (size 10):**

```
Origin        Cur     Topo    Match
------        ---     ----    -----
O_anchor      1       1       YES
O_action_1    2       2       YES
O_action_2    6       10      NO
O_action_3    7       5       NO
O_action_4    8       9       NO
O_action_5    9       3       NO
O_action_6    10      6       NO
O_action_7    3       7       NO
O_action_8    5       8       NO
O_action_9    4       4       YES
O_action_10   11      11      YES
```

Canonical bytes equal: NO
Digest cur:  `sha256:c91db527fa40b90ab07543690e97dbb029232ec320fa7299e0d50faf25899324`
Digest topo: `sha256:60107263434a0bcc2d0c9fcad72ac1b31f43e3fee4ea04d6a034a00087b26e7f`

## 5. Why Canonical Bytes Differ

The mechanism is:

1. Topological strategy produces **different colour values** (even for
   low-symmetry programs with identical round counts)
2. Different colour values → different sort order in `assign_canonical_ids`
   (which sorts by `(colour, id)`)
3. Different sort order → different canonical IDs (O1, O2, O3, ...)
4. Different canonical IDs → different canonical bytes
5. Different canonical bytes → different ProgramDigest

The colour values differ because the topological strategy's running accumulator
updates colours within a round. When origin O_k is processed, it sees
provisional colours for O_1..O_{k-1} that were computed in THIS round, not the
previous round. The current strategy uses only previous-round colours for ALL
origins. These different colour inputs produce different signature strings,
which produce different compressed colour values, even when the final partition
is identical.

## 6. Can Exact Final Canonical Colour Ordering Be Preserved?

**No, not with the topological multi-pass strategy.**

The colour values depend on the exact signatures, which depend on the colours
used during computation. The topological strategy's within-round colour
updates produce different signatures than the current strategy's
previous-round-only approach.

Two approaches preserve exact colour values:
1. Use previous round's colours for all signatures → same as current strategy,
   no convergence benefit
2. Run both strategies and map colours → requires running the current strategy
   anyway, defeating the purpose

## 7. Can Canonical-ID Assignment Remain Untouched?

If the colour values change, the canonical-ID assignment (which sorts by
colour) produces different canonical IDs. The canonical bytes and
ProgramDigest change.

If the colour values are preserved (approach 1 above), the canonical-ID
assignment is unchanged.

## 8. Projected Speedup (if identity contract were relaxed)

If the specification accepted "equivalent partition, deterministic numbering"
rather than "exact same colour values":

- High-symmetry: 14-109× faster (2 rounds vs n/2+1)
- Low-symmetry: no change (already 2 rounds)
- Overall: the superlinear canonicalization curve would become approximately
  linear for the sequential chain case

## 9. Incremental/Dirty Tracking Assessment

### Question

Can preserving the synchronous refinement rounds while doing incremental/dirty
work per round reduce complexity without changing the exact legacy colour
sequence?

### Analysis

The current round structure:
```
for each round:
  recompute ALL fact signatures → compress → fact_colours
  recompute ALL origin signatures → compress → origin_colours
  recompute ALL batch signatures → compress → batch_colours
  recompute ALL role signatures → compress → role_colours
  recompute ALL branch signatures → compress → branch_colours
  recompute ALL item_template signatures → compress → item_template_colours
```

Incremental approach:
```
for each round:
  for each entity type in dependency order:
    if any dependency colour changed since last round:
      recompute signatures for this entity type
    else:
      skip (colours unchanged)
```

### Dependency graph

- **Facts** depend on: origin colours (provenance), role colours (proxy),
  fact guards, fact consumers — all from previous round
- **Origins** depend on: fact colours (current round), origin colours
  (predecessors via `:in=`, branches, together), branch colours, batch
  colours — all from previous round
- **Batches** depend on: origin colours (via item_template), fact colours
- **Branches** depend on: origin colours
- **Roles** depend on: fact colours, item_template colours
- **Item templates** depend on: origin, branch, role, batch colours

### For the sequential chain benchmark

After round 1:
- **Facts**: unchanged (no origin provenance, no role proxy, no guards, no
  consumers). Can skip recomputation after round 1.
- **Origins**: change every round (colour propagation through `:in=`). Must
  recompute every round.
- **Batches, branches, roles, item templates**: none exist in this benchmark.

So for the sequential chain:
- Facts can be skipped after round 1 (saves O(n) signature computations per
  round)
- Origins must be recomputed every round (O(n) signatures per round)
- Total work: O(n) rounds × O(n) origin signatures = O(n²) (same as current)
- Speedup: constant factor only (skip fact recomputation)

### For the general case

The round count is determined by colour propagation through dependency edges.
Incremental dirty tracking can skip entity types whose dependencies haven't
changed, reducing per-round work. But:

1. If ANY entity type's colours change every round (like origins in the
   sequential chain), that type must be recomputed every round
2. The round count itself is unchanged (O(diameter))
3. Total work = O(diameter) × O(changed entities per round)

For the sequential chain, changed entities per round = O(1) (only one origin's
colour changes per round, but we must recompute ALL origin signatures to detect
which ones changed). So the per-round work remains O(n).

A smarter approach: track which specific origins' signatures changed, and only
recompute signatures that depend on those origins. But in the sequential chain,
each origin depends on its predecessor, so a change to O_k propagates to O_{k+1}
in the next round. This is exactly the one-hop-per-round constraint — we can't
speed it up without changing the colour computation.

### Conclusion

Incremental dirty tracking can provide a constant-factor speedup by skipping
unchanged entity types, but cannot change the O(n) round count for the
sequential chain. The round count is bounded by the graph diameter, which is
a structural property of the program, not an artifact of the algorithm.

The per-round work can be reduced from O(n²) to O(n × k) where k is the
number of entity types that change per round, but k remains O(n) for the
sequential chain (all origins change every round until convergence).

## 10. Implementation Risk

**Low** for the topological strategy itself (small code change, same
data structures). **High** for identity preservation (the strategy fundamentally
produces different colour values).

## 11. Recommendation

### A. TOPO SAFE — with caveat

The topological strategy is **semantically safe**: it produces the same
partition as the current strategy at all tested sizes. The canonical bytes and
ProgramDigest differ, but this is because the frozen contract requires exact
colour value identity, not just partition identity.

If the frozen contract were relaxed to accept "same partition + deterministic
numbering" (which would produce semantically identical canonical programs), the
topological strategy would be a valid production candidate with 14-109× speedup.

The topological strategy does NOT claim "stronger refinement is impossible" in
general. It demonstrates one specific acceleration (within-round colour
propagation) that achieves dramatic speedup but produces different colour values.

### Incremental: B. NOT PLAUSIBLE without changing the colour sequence

Incremental dirty tracking within the existing synchronous round structure
cannot reduce the O(n) round count. It can only provide a constant-factor
speedup by skipping unchanged entity types. For the sequential chain, the
per-round work remains O(n) because all origin signatures must be recomputed
to detect changes.

The fundamental constraint is the graph diameter: colour information must
propagate one hop per round through dependency edges. Incremental approaches
don't change this propagation speed.

---

## Evidence

### Files created/modified
- `tethers-0.1/engine-ocaml/bin/tethers_cb1_benchmark.ml` — benchmark prototype (modified)
- `tethers-0.1/engine-ocaml/bin/tethers_core_canonical.mli` — exposed pipeline functions for benchmarking
- `tethers-0.1/engine-ocaml/bin/dune` — added executable entry for benchmark

### Commands run
- `dune build bin/tethers_cb1_benchmark.exe` — PASS
- `dune exec bin/tethers_cb1_benchmark.exe` — results above
- `dune runtest --force` — PASS (all existing tests green)

### Benchmark file location
- `tethers-0.1/engine-ocaml/bin/tethers_cb1_benchmark.ml` (benchmark-only code)

### Worker note
- `docs/worker-notes/2026-08-12-c-b1-refinement-algorithm-proof.md`
