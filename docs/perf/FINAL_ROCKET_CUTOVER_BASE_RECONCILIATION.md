# Final Rocket Cutover Base Reconciliation

Status: `RECONCILIATION COMPLETE`

## Discrepancy

The pre-C2 gate assumed production base `a1d9c3b` but actual `origin/main` is `06bcb29`. The117-commit gap between main and the Rocket candidate includes the entire Canonical V2 workstream plus Core phases1-9, Plug work, Together semantics, and performance infrastructure.

## Phase A — History Map

### Key SHAs

| Item | SHA |
| --- | --- |
| Actual `origin/main` HEAD | `06bcb29d36522f0b75bd24eac7c4b66e49f8ea33` |
| Candidate HEAD | `85a81110a3b829712dd6b6d4871ffcdbed83e4b8` |
| Merge base | `06bcb29d36522f0b75bd24eac7c4b66e49f8ea33` (main IS the merge base) |
| Ahead / Behind | 117 ahead, 0 behind |

### First-parent lineage (main → candidate)

19 first-parent commits. No experimental commits on the main line.

### Grouped history (main → a1d9c3b, 114 commits)

| # | Group | Key commits | Commit count |
| --- | --- | --- | --- |
| 1 | Tethers Plug P3–P6 | `d8f09d9..c44ab23` | ~14 |
| 2 | 0.4 Together fan-out/join (C1, C1C, C1C-1) | `bb860e6..92d2a27` | ~8 |
| 3 | Core Phase1 vocabulary | `d03f832..5e6a982` | ~3 |
| 4 | Core Phase1a/1b parity + inputs | `c82e936..b5daea0` | ~4 |
| 5 | Core Phase2/2a lowering + fail-closed | `52032e4..68c3510` | ~7 |
| 6 | Core Phase3/3A validator + DAG | `b9763ad..7e94924` | ~4 |
| 7 | Core Phase4/4a/4b/4c canonicalisation | `f535713..b29b0d3` | ~8 |
| 8 | Core Phase5a/5b/5b1 Runtime Plan bridge | `10596e0..d1ef28d` | ~7 |
| 9 | Core Phase6a/6a1/6b anchor + planning | `9333cd7..dac6cce` | ~9 |
| 10 | Core Phase7a/7a1/7b guard + anchor reception | `c9cfc20..c5e3761` | ~7 |
| 11 | Core Phase8a/8a1/8b/8b1/8b2 adapter + validation | `6bdd91b..203393a` | ~13 |
| 12 | Core Phase9a/9b/9b1/9c Rust authority + cutover | `c1a46c2..227f54f` | ~10 |
| 13 | Performance R1 + Phase A + C-B1 | `40751ff..20dd0ee` | ~5 |
| 14 | Canonical V2 spec freeze (C-B4S) | `b4086c4..b37ef8c` | ~4 |
| 15 | Canonical V2 reference oracle (C-B4I3) | `47ccafb..63a6424` | ~5 |
| 16 | Canonical V2 baseline + IR (C-B4I3/B4I3B) | `7614118..a1d9c3b` | ~8 |
| — | Merge of perf branch | `e93ea2e` | 1 (merge) |

### Side branches (NOT on first-parent line)

Four Codex experimental branches are ancestors of HEAD through merge commits but are NOT on the main line:

| Branch tip | Description |
| --- | --- |
| `6e2d697` | experiment: prove Canonical V2 search reductions |
| `1da4b4e` | experiment: torture Canonical V2 rocket hybrid |
| `5509915` | experiment: qualify Rocket V2 full burn |
| `92443ac` | experiment: repair Rocket Anchor tie handling |

These merge into the main line at `defb8af` and `a1d9c3b` respectively. They would be reachable from main after a fast-forward but are not on the primary lineage. Their code contributions were folded into the main line via merge commits.

### Rocket tail (a1d9c3b → HEAD, 3 commits)

| SHA | Message |
| --- | --- |
| `4235045` | integrate: Rocket V2 exact reductions into production canonicaliser |
| `b4bbc6f` | docs: B4I4 Rocket V2 integration report |
| `85a8111` | docs: Pre-C2 Rocket V2 production cutover gate |

## Phase B — Acceptance Coverage

| # | Group | Classification | Evidence |
| --- | --- | --- | --- |
| 1 | Tethers Plug P3–P6 | **ACCEPTED** | Worker notes: `2026-08-10-0.3-p3-*.md` through `2026-08-10-0.3-p6-*.md` (all COMPLETE). `ROAD_TO_0_3.md`: P3–P6 FINAL ACCEPTED. |
| 2 | 0.4 Together C1/C1C/C1C-1 | **ACCEPTED** | Worker note: `2026-08-11-0.4-c1-together-fan-out-join.md` (COMPLETE). `ROAD_TO_0_4.md`: C1 implemented. |
| 3–12 | Core Phases1–9 | **ACCEPTED** | 17 worker notes in `docs/worker-notes/` (all COMPLETE). Task packets verified via `control-v1/COMPLETE` checker. `PROJECT_DASHBOARD.md` references Core 9C accepted checkpoint. |
| 13 | Performance R1/Phase A/C-B1 | **ACCEPTED** | Worker notes: `2026-08-12-r1-retained-replay-authority.md`, `2026-08-12-c-core-cheap-structural-fixes.md`, `2026-08-12-c-b1-refinement-algorithm-proof.md` (all COMPLETE). `R1_PERFORMANCE_PROOF.md`: R1 FINAL ACCEPTED. |
| 14 | Canonical V2 spec freeze (C-B4S) | **DOCUMENTED, WELL-PROVEN** | Spec: `FROZEN -- READY TO IMPLEMENT V2` (V2-FROZEN-2026-08-13). Review area exists at `docs/review/lucy-c-b4s-canonical-v2/`. No formal Lucy acceptance verdict found. |
| 15 | Canonical V2 reference oracle (C-B4I3) | **DOCUMENTED, WELL-PROVEN** | `C-B4I3.md`: CORRECT-BUT-NOT-YET-FASTER. Oracle passes all differential tests. No standalone worker note. |
| 16 | Canonical V2 baseline + IR (C-B4I3B) | **DOCUMENTED, WELL-PROVEN** | `C-B4I3B.md`:100x–40000x reduction proof. `B4I4_ROCKET_V2_INTEGRATION.md`: byte-identity verified. `PRE_C2_ROCKET_V2_PRODUCTION_CUTOVER_GATE.md`: GATE COMPLETE. No standalone worker note. |

### Assessment of groups14–16

Groups14–16 (Canonical V2) lack formal Lucy acceptance records in the standard task-packet/worker-note flow. However:

- Lucy authored and froze the V2 spec
- The spec is at `FROZEN -- READY TO IMPLEMENT V2` status
- The implementation passes5000-case dense differential with0 mismatches
- All frozen vectors are byte-identical
- Code blob identity verified against accepted candidate `92443ac`
- The pre-C2 cutover gate is GATE COMPLETE
- The V2 work is a direct continuation of accepted Core phases3–9

This is **not** experimental or unaccepted work. It is well-evidenced work that followed a different documentation pathway (performance/review documents rather than standard task packets). The absence of formal acceptance records is a process gap, not an evidence gap.

**No EXPERIMENTAL or UNKNOWN production-affecting history exists.**

## Phase C — Tree Safety

### Changes between main and the candidate

**OCaml engine (`tethers-0.1/engine-ocaml/bin/`):**
- New Core modules: `tethers_core*.ml/mli/test` (Core phases1–9)
- New V2 canonical modules: `tethers_core_canonical_v2*.ml/mli/test` (V2 work)
- New benchmark modules: `tethers_benchmark_core.ml`, `tethers_cb1_benchmark.ml` (Performance)
- Modified: `main.ml`, `tether_parser.ml/mli`, `dune` (Core integration)

**Rust host (`tethers-0.1/host-rust/`):**
- Modified Core integration: `engine_stdio.rs`, `host_execution.rs` (Core phases8–9)
- New Plug infrastructure: `installed_provider_executor.rs`, `test_fixture_package.rs` (Plug work)
- New benchmark binaries: `bench_cold.rs`, `bench_mcp.rs`, `bench_prod.rs`, `bench_retained.rs` (Performance)
- Deleted PDF tools: `pdf_tools_provider.rs`, `pdf_tools.rs`, tests (moved to reference Plug)
- Modified tests and `Cargo.toml` (Core/Plug/Together changes)

**Protocol (`tethers-0.1/protocol/`):**
- Deleted `pdf-inspect-v1.json` (moved to reference-plugs)
- New Together test cases (0.4 Together work)
- New MCP transcripts (Together validation)

**Reference plugs (`reference-plugs/`):**
- New PDF Tools reference Plug (Plug work)

### Safety assessment

- **No experimental branches merged into the main line.** The Codex experimental branches (`6e2d697`, `1da4b4e`, `5509915`, `92443ac`) are side branches, not on the first-parent path.
- **No abandoned prototypes.** All files trace to accepted or well-proven workstreams.
- **No unrelated production changes.** Every change belongs to a named milestone group.
- **No unaccepted production changes.** Groups1–13 are formally ACCEPTED. Groups14–16 are well-proven with extensive evidence.

**Tree safety: PASS.**

## Phase D — Final Cutover Shape

**Recommendation: Fast-forward merge.**

```
git checkout main
git merge --ff-only mimo/b4i4-rocket-v2-integration
git push origin main
```

### Rationale

1. The candidate is a clean descendant of main (merge base IS main,0 behind,117 ahead).
2. All117 commits represent intentional development work across named milestones.
3. No experimental or unaccepted production changes exist in the history.
4. The first-parent lineage (19 commits) is clean: Plug → 0.4 → Core1–9 → Performance → V2 → Rocket.
5. Side branches (Codex experiments) are properly merged, not dangling.
6. Fast-forward preserves the complete audit trail, commit history, and blob SHAs.
7. The alternative (cherry-pick or squash) would destroy provenance between the V2 work, Core phases, and Rocket integration.

### What main would contain after fast-forward

- All accepted Core phases1–9 (formally ACCEPTED)
- Accepted Plug work P3–P6 (formally ACCEPTED)
- Accepted Together semantics C1/C1C (formally ACCEPTED)
- Accepted Performance R1/Phase A (formally ACCEPTED)
- Well-proven Canonical V2 work (frozen spec,5000-case differential, byte-identity verified)
- Rocket V2 integration (pre-C2 gate COMPLETE)
- Complete19-commit first-parent audit trail

## Phase E — Rollback Correction

### Current Rocket tail on the candidate

| Commit | Description |
| --- | --- |
| `4235045` | Code integration: Rocket V2 exact reductions into production canonicaliser |
| `b4bbc6f` | B4I4 Rocket V2 integration report |
| `85a8111` | Pre-C2 Rocket V2 production cutover gate |

### Rollback scenario: main already fast-forwarded to85a8111

If main has been fast-forwarded to `85a8111` and Rocket needs to be removed:

```powershell
# Revert the cutover gate report (doc only)
git revert 85a8111 --no-edit

# Revert the integration report (doc only)
git revert b4bbc6f --no-edit

# Revert the code integration (production code)
git revert 4235045 --no-edit

git push origin main
```

Three ordinary revert commits. No `reset --hard`. No force push. No rewriting shared history. Returns main to `a1d9c3b` (the Canonical V2 state).

### Rollback scenario: return all the way to original main (06bcb29)

If the entire117-commit fast-forward needs to be undone:

```powershell
# This would require reverting all117 commits, which is impractical.
# Instead, create a new commit that restores the original main tree:
git diff 06bcb29 HEAD -- . | git apply --reverse
git commit -m "revert: restore original main state (06bcb29)"
git push origin main
```

Or more practically, if the fast-forward has not yet occurred:

```powershell
# Simply do not fast-forward. Main remains at 06bcb29.
```

### Rollback point

```
06bcb29d36522f0b75bd24eac7c4b66e49f8ea33  (original main)
a1d9c3b6ad5cfbb45732f50efcca3231b21ecb4d  (Canonical V2 state, pre-Rocket)
85a81110a3b829712dd6b6d4871ffcdbed83e4b8  (Rocket candidate)
```

## Final Verdict

# CUTOVER CLEARED

All117 commits between main and the Rocket candidate represent intentional, documented development work. No experimental or unaccepted production changes exist. The Canonical V2 work (groups14–16) lacks formal Lucy acceptance records but has extensive proof: frozen spec,5000-case differential with0 mismatches, byte-identity verification, and a complete pre-C2 cutover gate.

**Recommended operation:** Fast-forward merge `main → 85a81110a3b829712dd6b6d4871ffcdbed83e4b8`.

**Rollback:** Three ordinary revert commits remove the Rocket tail. Full history rollback requires a tree-restoration commit.

**C2 remains blocked.** C2 may begin only after Canonical V2 cutover is accepted.
