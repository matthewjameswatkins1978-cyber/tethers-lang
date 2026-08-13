# B0 Original Baseline Report

Status: historical baseline. Recorded as measured, no optimisation performed.
Worktree: `D:\The Next Thing\Tethers Lang - Goose Integration`
Branch: `perf/b0-original-baseline`
Baseline SHA: `1ce6b10f1de3cd10fef619483df444f83899c870`
Date: 2026-08-12

The raw evidence lives in `docs/performance/b0-original-baseline/`. See its
`README.md` for SHA-256 hashes and overwrite protection. Quick-mode numbers are
NOT baseline performance numbers.

## Measured Baseline Values

| Layer | Case | Median |
| ----- | ---- | ------ |
| Core (B0-A) | P1 | ~18.5 us |
| Core (B0-A) | P50 | ~3.96 ms |
| Warm MCP (B0-B) | P1 | ~106 us |
| Warm MCP (B0-B) | P50 | ~4.64 ms |
| Warm production (B0-C) | P1 | ~110 ms |
| Warm production (B0-C) | P10 | ~3.35 s |
| Warm production (B0-C) | P25 | ~15.23 s |
| Warm production (B0-C) | P50 | ~47.21 s |
| Cold production (B0-D) | P10 | ~1.33 s |

Warm production P50: **750 provider tools/call** across the completed 15
warm + measured evaluations.

## Recorded Observations

State these as observations, not causes:

- **Core shows superlinear growth** with program size.
- **Warm production shows severe superlinear / state-dependent growth.**
- **Cold P10 beating retained P10 is observed** (fresh-process P10 ~1.33 s
  versus retained production P10 ~3.35 s).
- **Retained state growth is a hypothesis awaiting PF1**, not a conclusion.

The PF1 evidence-collection run and code audit are recorded in
`docs/performance/PF1_FORENSICS.md`.

## Baseline Integrity

`raw.json` and `raw.csv` are immutable. SHA-256 hashes are recorded in
`docs/performance/b0-original-baseline/README.md`. Benchmark scripts require an
explicit overwrite flag before replacing the historical aggregate.
