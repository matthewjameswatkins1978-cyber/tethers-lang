# PF1 Performance Forensics

Status: evidence collection only. No optimisation was performed.
Model: DeepSeek Flash. Thinking: HIGH.
Worktree: `D:\The Next Thing\Tethers Lang - Goose Integration`
Branch: `perf/b0-original-baseline`
Baseline HEAD: `1ce6b10f1de3cd10fef619483df444f83899c870`
Date: 2026-08-12

Boundaries honoured: NO C2A/C2B, NO production optimisation, NO semantic
changes, NO policy/replay/Trail weakening, NO canonical digest changes.
Benchmark-only instrumentation only.

---

## 1. WHAT WE PROVED

**P1. Retained production latency grows with retained state (MEASURED).**
One retained session, P10 (10 actions/eval), 12 individually-timed
evaluations with fresh evaluation IDs, exactly 10 provider `tools/call`
each (proven by marker-file delta; any mismatch stopped the run).

| eval | wall ms | replay_admit us/action | claims (cum) | provider calls (cum) |
| ---- | ------- | ---------------------- | ------------ | --------------------- |
| 1  | 765.6 | 25,994 | 10  | 10  |
| 2  | 949.0 | 43,252 | 20  | 20  |
| 3  | 1075.1 | 57,906 | 30  | 30  |
| 4  | 1242.4 | 74,578 | 40  | 40  |
| 5  | 1400.8 | 90,195 | 50  | 50  |
| 6  | 1553.1 | 106,597 | 60  | 60  |
| 7  | 1717.6 | 123,323 | 70  | 70  |
| 8  | 1860.7 | 137,629 | 80  | 80  |
| 9  | 1996.7 | 150,853 | 90  | 90  |
| 10 | 2126.2 | 164,606 | 100 | 100 |
| 11 | 2283.8 | 179,937 | 110 | 110 |
| 12 | 2447.1 | 195,712 | 120 | 120 |

Wall latency: first 765.6 ms, last 2447.1 ms, median 1635.4 ms, linear
regression slope ≈ +151 ms per evaluation. Growth observed: **YES**.

**P2. The one growing production stage is replay admission (MEASURED).**
Feature-gated timing hooks in the real production path (no behaviour,
ordering, persistence, or error-handling change) show `replay_admit` is the
only stage whose cost grows with retained claims:

| stage (per-eval total)      | eval 1 | eval 3  | eval 6  | eval 12 |
| --------------------------- | ------ | ------- | ------- | ------- |
| replay_admit                | 260 ms | 579 ms  | 1066 ms | 1957 ms |
| replay_publish_intent       | 151 ms | 149 ms  | 148 ms  | 149 ms  |
| replay_publish_armed        | 165 ms | 161 ms  | 160 ms  | 163 ms  |
| replay_publish_terminal     | 149 ms | 152 ms  | 148 ms  | 148 ms  |
| provider_call               | 20.8 ms | 15.1 ms | 11.7 ms | 11.8 ms |
| trail_intent                | 6.5 ms | 6.2 ms  | 6.7 ms  | 5.9 ms  |
| trail_outcome               | 6.7 ms | 6.5 ms  | 6.3 ms  | 5.9 ms  |
| core_mcp (1/eval)           | 0.7 ms | 0.8 ms  | 0.7 ms  | 0.7 ms  |
| scope_policy / capability / catalogue / envelope / result_anchor | < 1 ms total, flat | | | |

`replay_admit` grows ≈ linearly with retained claims per action (slope
≈ +15.4 ms per 10 new claims per action). Replay ledger size grows at a
constant rate per evaluation (+50 files, ~25.1 KB/eval: 10 claims + 10
locks + 30 chain-generation files). Trail grows at a constant rate
(+20 lines, ~5.0 KB/eval). Nothing else grows.

**P3. Core canonicalisation is the dominant and superlinear Core stage
(MEASURED).** Stage profiler (batched, same production functions, all sizes
verified `staged==wire==matched`):

| size | parse | lower | validate | canonicalize | plan | whole |
| ---- | ----- | ----- | -------- | ------------ | ---- | ----- |
| 5    | 5.0   | 0.0   | 0.0    | 55.0    | 10.0   | 80.0   |
| 10   | 10.0  | 0.0   | 10.0   | 170.0   | 20.0   | 210.0  |
| 25   | 16.7  | 0.0   | 66.6   | 866.7   | 83.3   | 1000.0 |
| 50   | 50.0  | 25.0  | 225.0  | 3525.1  | 325.0  | 3925.1 |
| 100  | 100.0 | 50.0  | 800.0  | 15550.3 | 1250.0 | 16950.3|
| 250  | 200.1 | 100.0 | 5000.0 | 136802.7| 7750.2 | 144703.0|
| 500  | 500.0 | 166.7 | 19333.8| 753186.9| 26503.2| 774940.2|

Median microseconds. `canonicalize` scaling ≈ O(n^2.1–2.5) between sizes
50→500 (4.4× per 2× at 50→100; 8.8× per 2.5× at 100→250; 5.5× per 2× at
250→500). At size 500 canonicalization alone is ~753 ms — the
"large-program cliff". `validate` ≈ O(n^2), `plan` ≈ O(n^1.8–2.0). `parse`
and `lower` are linear. Canonicalization is ~97% of the whole pipeline at
size 500.

**P4. Canonical refinement cost depends strongly on symmetry (MEASURED).**
Shape probe at sizes 100 and 250 (canonicalize median):

| size | high-sym (identical actions) | low-sym (distinct literals) | ratio |
| ---- | ---------------------------- | --------------------------- | ----- |
| 100  | 17,513 us | 2,700 us | 6.5× |
| 250  | 153,557 us | 12,900 us | 11.9× |

High symmetry (the B0/P-cases) is the expensive case, not the cheap one:
identical Actions form a long sequential chain whose partition refinement
needs one round per origin; distinct signatures converge in ~1–2 rounds.

## 2. WHAT WE DID NOT PROVE

- We did NOT prove that retained growth is caused by any stage other than
  replay admission. All other measured stages were flat.
- We did NOT measure the actual instruction-level cause inside
  `ReplayLedger::open` (the scan could be dominated by directory
  enumeration, claim-file reads, chain-file reads, or digest verification).
  The code audit (section 6) identifies the full scan as the mechanism; the
  timing isolates it to `replay_admit`, but not to which sub-read.
- We did NOT measure a list-vs-index crossover by implementing an indexed
  variant (that would be an optimisation). The crossover region is
  INFERRED from the measured curve, not measured.
- We did NOT measure on a different machine, under load, or with realistic
  provider/Tether diversity.
- We did NOT re-run the full B0 baseline; B0 numbers are quoted as given.

## 3. RETAINED P10 CURVE

MEASURED values above. Summary statistics:

- first evaluation: **765.6 ms**
- last evaluation: **2447.1 ms**
- median wall: **1635.4 ms**
- slope (least-squares wall vs eval number): **≈ +151 ms/eval**
- Trail growth: **+20 lines / +5,012 B per evaluation** (constant rate)
- replay growth: **+50 files / +25,061 B per evaluation** (constant rate:
  10 claims, 10 locks, 30 chain files, 10 execution dirs)
- replay_admit growth: **≈ +15.4 ms per action per 10 new claims**
  (rate proportional to retained claims → superlinear total)

Interpretation: the curve is not flat and not merely noisy — it is
monotonically increasing with retained claim count. The per-evaluation cost
is dominated by per-action replay-ledger reopening which rescans all
retained claims and chains.

## 4. WHERE PRODUCTION TIME GOES

MEASURED (feature-gated timing hooks; observation-only). At eval 12, the
per-evaluation budget (~2.45 s) breaks down as:

- **replay_admit (replay ledger open + full validation scan): ~1.96 s (80%)** — GROWING
- replay_publish_intent + armed + terminal (3 immutable generations):
  ~0.46 s (19%) — flat
- provider tools/call: ~0.012 s — flat
- trail_intent + trail_outcome (append + fsync): ~0.012 s — flat
- core MCP evaluation: ~0.001 s — flat
- scope + policy + capability resolution + catalogue refresh + result
  anchor: < 0.001 s — flat

The single most important answer to "what grows?": **replay admission**
(`FileReplayAuthority` → `ReplayLedger::open` → `validate_whole_ledger`
→ `scan_claims` + `scan_chains`, re-read on every Action because a fresh
`FileReplayAuthority` is constructed per Action in `execute_one_action`).

Stages that could NOT be isolated safely without a benchmark hook were not
split further; the coarsest safe split that is informative is the one above.

## 5. CORE STAGE CURVE

MEASURED table in section 1 (P3). Key facts:

- linear: parse (~1 us/action), lower (~0.5 us/action)
- ~O(n^2): validate, plan
- ~O(n^2.1–2.5): canonicalize (dominant; ~97% of whole at size 500)
- whole pipeline: 80 us at size 5 → 775 ms at size 500 (~9,700× for 100×
  size)

## 6. CODE COMPLEXITY SUSPECTS

Audit of the accepted code (CORE-8B/CORE-9C path). No changes made.

| # | FUNCTION | DATA STRUCTURE | OPERATION | EXPECTED COMPLEXITY | HOW OFTEN CALLED | PROFILING SUPPORTS? |
|---|----------|----------------|-----------|---------------------|------------------|---------------------|
| 1 | `compress_colours` (tethers_core_canonical.ml:172) | list of (id, sig); `sig_to_colour` list | `List.sort_uniq` then per-pair `List.assoc sig_str sig_to_colour` | O(n·u) per call (n entities, u unique sigs) | every refinement round, per entity type | YES: canonicalize superlinear |
| 2 | `refine_round` (tethers_core_canonical.ml:683) | lists of signatures | recomputes every signature + all 6 colour maps each round | O(n) sigs/round; rounds O(n) for sequential high-symmetry chain → total ~O(n³) worst case | every refinement round | YES: high-symmetry 6.5–11.9× slower |
| 3 | `int_map_partition_stable` (tethers_core_canonical.ml:183) | `StringMap` | `count_unique` via repeated `List.mem` over colour values | O(c²) per map (c colours); called 6× per `partitions_equal` per round | every round | YES (round count driver) |
| 4 | canonical ID rewrite `canonical_origin`/`canonical_fact`/… (tethers_core_canonical.ml:896-935) + `build_canonical_program` | order lists | `List.assoc_opt` per entity | O(n) per lookup, O(n²) total | once per canonicalize | YES (part of canonicalize) |
| 5 | `plan_core` walk `continuation_of`/`site_of` (tethers_core_plan.ml:467-521) | lists | rebuild continuation list + `List.assoc_opt` / `List.find_opt` per step | O(n²) for n-step walk | once per evaluation | YES: plan ≈ O(n²) |
| 6 | `unique_effects` (tethers_core_plan.ml:453) | list | `List.mem` + `acc @ [v]` | O(n²) | once per plan | partial (small absolute) |
| 7 | validator membership checks (tethers_core_validator.ml:208-211, 270, 606, …) | all-entity lists | `List.mem`/`List.find_opt`/`List.assoc_opt` against all-entity lists | O(n) per check, O(n²) across checks | once per canonicalize + once per plan | YES: validate ≈ O(n²) |
| 8 | `fact_sig_rn` consumer/guard sorting (tethers_core_canonical.ml:463-512) | lists | `List.sort` with string building per signature, per fact, per round | O(consumers·log consumers) per fact per round | every round | supports (in refine) |
| 9 | `sort_action_inputs` comparator (tethers_core_canonical.ml:1066) | list | builds binding string inside comparator | O(n·log n) compares with string concat | once | minor |
| 10 | `lowerer` `resolve_capability`/`resolve_fact` (tethers_core_lowerer.ml:85-101) | lists | `List.filter` per action/guard | O(a·c), O(g·f) | once | minor (linear overall) |

Suspects 1–4 + 7 explain the measured superlinear canonicalization and
validation curves. Suspects 5–6 explain the plan curve. The measured
symmetry effect (P4) is consistent with suspect 2's round-count dependence.

## 7. LIKELY OPTIMISATION TARGETS

Evidence-ranked (NOT implemented):

1. **Rust replay admission**: stop re-opening/re-scanning the whole replay
   ledger per Action. `ReplayLedger::open` → `validate_whole_ledger` is
   invoked once per Action because `FileReplayAuthority` is created per
   Action in `execute_one_action`. Keeping one ledger open per evaluation
   (or per service run) with O(1) per-key lookup would remove the entire
   measured growth — without weakening the durable audit, exclusion, or
   chain rules.
2. **OCaml canonical refinement**: the sequential-chain high-symmetry case
   needs one refinement round per origin. Reducing round count (better
   initial colouring) or making each round cheaper (indexed
   `compress_colours`) targets the n^2.1–2.5 canonicalize curve.
3. **OCaml `compress_colours` / partition stability**: replace
   `List.assoc` and `List.mem` scans with a Map/Hashtbl where measured
   sizes justify it.
4. **OCaml canonical ID rewrite + validator**: replace `List.assoc_opt`
   over order lists with an indexed map built once.
5. **OCaml `plan_core` walk**: index continuations and origin sites once.

## 8. QUESTIONS FOR GEMINI

1. Given the measured Core stage curves (canonicalize ~n^2.1–2.5 from
   size ~50, validate/plan ~n²), what data-structure / algorithm
   architecture preserves tiny-program speed (whole pipeline 80 us at size
   5) while avoiding the large-program cliff (775 ms at size 500)?
2. Can ONE semantic canonicalisation algorithm use adaptive internal data
   structures rather than maintaining multiple semantic algorithms?
3. If a size crossover exists, what deterministic switching rule is safest
   for canonical identity (canonical identity must not depend on the
   switching rule)?
4. If symmetry/refinement dominates (high-sym 6.5–11.9× slower than
   low-sym at 100/250), at what point would stronger partition refinement
   (e.g. Paige–Tarjan-style) become justified?
5. For the Rust production runtime, if retained replay/Trail/provider state
   causes growing latency (proven here: replay-admit scan), what
   architecture gives O(1)/O(log n) hot-path lookups while keeping the
   durable Trail authoritative and the replay chain immutable?
6. How do we avoid "optimising" by weakening replay, audit, policy,
   provider verification, or determinism?
7. What benchmark would falsify each proposed optimisation?

## 9. CONCISE EVIDENCE TABLE (for another architecture model)

| Claim | Evidence | Status |
| ----- | -------- | ------ |
| Retained P10 latency grows | 766→2447 ms over 12 evals, slope +151 ms/eval | MEASURED |
| Growth driver = replay admission | replay_admit 260→1957 ms/eval; all other stages flat | MEASURED |
| Replay ledger open rescans all claims/chains per Action | FileReplayAuthority per Action → ReplayLedger::open → validate_whole_ledger (replay_windows.rs:1252, host_execution.rs:1210) | CODE |
| Canonicalize is Core bottleneck | ~97% of pipeline at size 500, 55 us@5 → 753 ms@500 | MEASURED |
| Canonicalize scales superlinear | O(n^2.1–2.5) from doubling ratios | MEASURED (exp. INFERRED) |
| Symmetry strongly affects canonicalize | high-sym 6.5×@100, 11.9×@250 slower | MEASURED |
| Validate and plan ~O(n²) | 0→22 ms and 10 us→30 ms across sizes | MEASURED |
| Trail and replay sizes grow linearly | +20 lines/+5.0 KB and +50 files/+25.1 KB per eval | MEASURED |
| Provider call, catalogue, policy, capability stages flat | <1 ms/eval flat across evals | MEASURED |

## 10. PRODUCTION STATE AUDIT (Part H)

Per-operation classification, where n is the retained-state size (claims):

| Operation | Location | Classification |
| --------- | -------- | -------------- |
| ReplayLedger::open → validate_whole_ledger (locks scan + claim scan + chain scan, digest verification) | replay_windows.rs:1252-1274 | **O(n)** per open; **O(n) per Action** (fresh FileReplayAuthority per Action) → O(n²) across retained evals — **the growth driver** |
| admit_or_recover: existing_claim (direct file by key) | replay_windows.rs:1424-1480 | O(1) per key + Win32 claim publish (temp+rename+reopen+verify) |
| publish_generation (intent/armed/terminal): reconstruct reads only this execution's chain (≤3 files) | replay_windows.rs:1540-1595 | O(1) per Action (3 heavy Win32 writes, flat) |
| FileTrail::open (append) / append+sync_data | dispatch.rs:353,366-448 | O(1) per write; fsync per line; file grows in bytes only |
| refresh_prepared_catalogue (warm, no change) | host_execution.rs:1427-1450 | O(1) |
| resolve_capability (trusted store) | resolver.rs:261-306 | O(1) HashMap |
| assess_action_scope / locate_capability | configured_runtime.rs:556-620 | O(providers×caps), constant for fixed config |
| planner_capabilities + bridge projection | configured_runtime.rs:680, application.rs:1071 | O(providers×caps), constant per evaluation |
| provider tools/call | socket.rs invoke | O(1) round-trip, flat |
| Result Anchor write | application.rs:2494-2532 | in-memory (response JSON), O(1) |
| directory enumeration in scan | replay_windows.rs:987,1074,1295-1346 | part of ledger open, O(n) |

## 11. SMALL CROSSOVER DATA (Part I)

No adaptive switching was implemented. From the measured curve alone:

- Whole pipeline at size 5 is 80 us — list/direct structures clearly fine at
  tiny sizes.
- Canonicalization starts bending sharply between size 50 (3.6 ms) and size
  100 (17.5 ms); the growth becomes severe from size 100 upward.
- INFERRED crossover region for refinement-cost dominance: roughly
  50–100 Actions for the current sequential-chain, high-symmetry workload.
  This is an inference from the measured curve, NOT a measured crossover
  (no indexed implementation was built). Treat it as a candidate threshold
  to test later, not as authority to switch.

---

## FINAL REPORT

**PF1 PERFORMANCE FORENSICS COMPLETE**

- MODEL: DeepSeek Flash
- THINKING: HIGH

**B0 RAW HASHES** (unchanged; B0 not regenerated):
- raw.json: `BDF91A4F2D11432A0B990B05B39153EDD481CF9915385DC6A389810359859B1D`
- raw.csv:  `6C9E864F0686C1D8A76DE9A678871510ED3A2C0CD04B29FDF220D73193E9DF6C`

**RETAINED P10:**
- eval 1: 765.6 ms
- eval 3: 1075.1 ms
- eval 6: 1553.1 ms
- eval 12: 2447.1 ms
- growth observed: **YES**

**FASTEST GROWING PRODUCTION STAGE:** replay admission (`replay_admit`:
replay ledger open + full claims/chains scan per Action)
- evidence: replay_admit 260 ms → 1957 ms per evaluation (evals 1→12);
  every other stage flat; code path replay_windows.rs:1252 opened per
  Action from host_execution.rs:1210

**CORE BOTTLENECK STAGE:** canonicalize (colour refinement + digest)
- evidence: 55.0 us @ 5 → 753,187 us @ 500; ~97% of whole pipeline at 500

**CORE SCALING:** superlinear — canonicalize ≈ O(n^2.1–2.5), validate ≈
O(n²), plan ≈ O(n^1.8–2.0), parse/lower linear

**SYMMETRY EFFECT:** measured — high-symmetry canonicalize 6.5× (size 100)
and 11.9× (size 250) slower than low-symmetry

**TOP 5 CODE SUSPECTS:**
1. `compress_colours` — `List.assoc` per pair over `sig_to_colour`
   (tethers_core_canonical.ml:172)
2. `refine_round` — full signature/colour recomputation every round, O(n)
   rounds for high-symmetry sequential chains (tethers_core_canonical.ml:683)
3. `int_map_partition_stable` — `count_unique` via repeated `List.mem`
   (tethers_core_canonical.ml:183)
4. canonical ID rewrite + `build_canonical_program` — `List.assoc_opt`
   over order lists (tethers_core_canonical.ml:896,1168)
5. `ReplayLedger::open` → `validate_whole_ledger` full scan per Action
   (replay_windows.rs:1252)

**PROVEN:**
- Retained P10 latency grows with retained claims (MEASURED)
- Replay admission is the single growing production stage (MEASURED)
- Canonicalize is the dominant, superlinear Core stage (MEASURED)
- Symmetry strongly affects canonical refinement cost (MEASURED)

**INFERRED:**
- ReplayLedger::open full-scan is the mechanism behind replay_admit growth
  (timing isolates the stage; code identifies the scan)
- Canonicalize scaling exponents (O(n^2.1–2.5) fitted from doubling ratios)
- Size crossover region for refinement-cost dominance (~50–100 actions)
- validate/plan O(n²) attributed to repeated List.mem/assoc over entity lists

**UNKNOWN:**
- Which sub-read inside the ledger scan dominates (claims vs chains vs
  locks vs directory enumeration)
- Whether real multi-provider/multi-tether workloads show different scaling
- Whether the OCaml cliff reproduces on other compilers/OS

**FILES CREATED:**
- `docs/performance/pf1/retained-p10.json`
- `docs/performance/pf1/retained-p10.csv`
- `docs/performance/pf1/core-stages.json`
- `docs/performance/pf1/core-stages.csv`
- `docs/performance/PF1_FORENSICS.md` (this file)

**BENCHMARK-ONLY INSTRUMENTATION ADDED (not production optimisation):**
- `tethers-0.1/engine-ocaml/bin/tethers_benchmark_core.ml` — stage profiler
  + shape probe behind `--profile-stages` (default B0-A behaviour unchanged)
- `tethers-0.1/host-rust/src/bench_timing.rs` — zero-cost feature-gated
  timing sink (`bench-timing` cargo feature, off by default)
- `tethers-0.1/host-rust/src/host_execution.rs` + `application.rs` —
  observation-only `bench_timing::timed` wrappers (no-op without the
  feature)
- `tethers-0.1/host-rust/src/bin/bench_retained.rs` + Cargo.toml feature/bin
- `scripts/pf1-retained.ps1`, `scripts/pf1-core-stages.ps1`

**WALL-CLOCK JOB TIME:** ~10 minutes total (retained run ~40 s; core stage
profile ~90 s; builds/checks ~6 min)
**PROVIDER/API COST:** none (local benchmarks; no external API calls)
**INPUT TOKENS / OUTPUT TOKENS / REASONING TOKENS / CACHED TOKENS:**
UNAVAILABLE (not exposed)

No optimisation is claimed. Evidence collection is complete; the next step
belongs to the architecture model.
