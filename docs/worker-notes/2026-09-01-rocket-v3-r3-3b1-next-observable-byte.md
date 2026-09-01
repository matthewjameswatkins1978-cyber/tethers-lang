# Rocket V3 R3-3B1 Worker Note

Task: `Rocket V3 R3-3B1 next-observable-byte correctness repair`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `Codex`
Status: `BLOCKED`
Base commit: `c3d136dc4217059d4434f8d39a273fa398c4e64d`
Implementation checkpoint: `04242c51cc3c1345efcb8eee1f4d804158db8a8a`

## Requested outcome

Repair the Origin-only R3-3B forcing rule so continuation targets are forced only at the next completion-invariant Enc_V2 observable point, and prove the repair against exact exhaustive chain-10 and chain-11 references.

## Changes made

The walker now scans continuation source slots from numeric slot 1 upward and stops at the first unresolved owner. It forces an unresolved continuation target only when all lower source slots and preceding continuation elements are resolved. The previous rule, which scanned every occupied source slot and forced later targets, was removed.

The tests retain an independent frozen-encoder exhaustive reference. They enumerate 9! residual assignments for chain-10 and 10! residual assignments for chain-11 after the entry label is fixed by the frozen first-field law. They record the historical chain-11 assignment and first differing byte, then require the repaired walker to match the exact minimum. Temporary progress output and scale-only test selection were removed before the checkpoint.

## Decisions and assumptions

Frozen Enc_V2 remains the sole byte authority. Numeric labels still determine continuation collection order, while encoded integer bytes determine lexicographic comparison. No raw ID, internal vertex, source order, refinement colour, heuristic chain rule, new budget, or general I/R search was added. The larger-scale test list was limited to completed 10 and 12 cases after the deterministic chain-100 performance stop; the 100 and 1000 cases were not presented as solved.

## Evidence

Startup checks passed on the authorised branch and base; the packet was READY before mutation. The authorised OCaml switch was verified as D:\\The Next Thing\\Tethers Lang\\tethers-0.1\\engine-ocaml with OCaml 5.5.0 and Dune 3.24.0. The repaired focused suite passed 105/105 checks.

The required pre-repair chain-11 evidence is preserved by the test: former labels `[10,11,1,2,3,4,5,6,7,8,9]`, exact labels `[10,9,8,7,6,5,4,3,2,1,11]`, 3,628,800 exhaustive candidates, and first full-payload difference at byte offset 23 (`0x32` versus `0x31`) in the first unresolved success-continuation target label. The repaired result has no differing byte.

Chain-10 matched the 362,880-candidate exact reference under all three branch policies, including digest parity. Chain-11 matched the 3,628,800-candidate exact reference. Chain-12 completed with emitted_bytes=152923, forced_assignments=1767, decision_points=295, branches_explored=1521, prefix_prunes=1202, completed_candidates=26, max_depth=9. Chain-10 completed with emitted_bytes=2506, forced_assignments=37, decision_points=1, branches_explored=9, prefix_prunes=8, completed_candidates=2, max_depth=1.

The repaired chain-100 probe did not complete. At deterministic interruption after 27,000 explored branches it reported emitted_bytes=195471123, forced_assignments=2186535, decision_points=333, prefix_prunes=24387, completed_candidates=2281, max_depth=8. This is the authorised performance stop; chain-1000 was not started after it.

`dune build @all` passed. R3-1 passed 214/214, R3-2 passed 4807/4807, R3-3A passed 39/39, the V2 reference and production suites passed, and the V2 IR suite passed its 5,000-case generated corpus with zero mismatches.

## Discoveries

The repaired condition is necessary: a known numeric source slot does not make its target next-observable when lower numeric source ownership remains unresolved. The chain-11 counterexample crosses the decimal-width boundary and demonstrates that preserving exactness changes the search from the previous zero-branch result to a materially larger exact walk. The exactness repair is successful through chain-12, but the larger Origin-only walk remains computationally explosive enough to trigger the packet stop condition.

## Remaining risks

The Origin-only walker has no completed exact chain-100 or chain-1000 result after the repair. Full `dune runtest --force` was not run because the default Origin scaling probe would re-enter the known non-terminating chain-100 path; this is recorded as a consequence of the authorised performance stop, not as a passing regression claim. R3-3C, cross-family generalisation, and production integration remain unstarted.

## Smallest next action

Design a separately authorised exact performance improvement for the Origin-only walker, beginning from checkpoint `04242c51cc3c1345efcb8eee1f4d804158db8a8a`, without weakening the next-observable-byte proof or introducing heuristic pruning.

## References

Implementation checkpoint: `04242c51cc3c1345efcb8eee1f4d804158db8a8a`

Historical R3-3B base: `c3d136dc4217059d4434f8d39a273fa398c4e64d`

Authorised task packet: `docs/CURRENT_CLINE_TASK.md`
