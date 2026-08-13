# B0 Original Baseline — Immutable Evidence

This directory holds the completed B0 performance baseline. These files are
historical evidence and are treated as **immutable**:

- `raw.json` — aggregate raw JSON across B0-A / B0-B / B0-C / B0-D
- `raw.csv` — per-batch sample rows derived from `raw.json`
- `b0a.json` / `b0b.json` / `b0c.json` / `b0d.json` — per-layer captures

Semantic / runtime baseline SHA: `1ce6b10f1de3cd10fef619483df444f83899c870`

## SHA-256 Hashes

| File | SHA-256 |
| ---- | ------- |
| `raw.json` | `BDF91A4F2D11432A0B990B05B39153EDD481CF9915385DC6A389810359859B1D` |
| `raw.csv`  | `6C9E864F0686C1D8A76DE9A678871510ED3A2C0CD04B29FDF220D73193E9DF6C` |

Verify with:

```powershell
Get-FileHash raw.json -Algorithm SHA256
Get-FileHash raw.csv -Algorithm SHA256
```

## Overwrite Protection

Benchmark scripts write to timestamped `docs/performance/runs/run-*`
directories by default. Replacing the aggregate in this directory requires an
explicit overwrite flag:

```powershell
pwsh scripts/benchmark-tethers.ps1 -Full -OverwriteBaseline
pwsh scripts/benchmark/collect-baseline.ps1 -OverwriteBaseline
```

See `docs/performance/B0_ORIGINAL_BASELINE.md` for the recorded findings.
