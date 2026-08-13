# PRE-C2 / Rocket V2 Production Cutover Gate

Status: `GATE COMPLETE`

## 1. Exact Candidate SHA

| Item | Value |
| --- | --- |
| Candidate HEAD | `b4bbc6f39f5d779cf087f16f90b61ceb9c6b5193` |
| Code integration commit | `4235045cd542cbc65dad092d9bc8c4da7768c95d` |
| Production base | `a1d9c3b6ad5cfbb45732f50efcca3231b21ecb4d` |
| Branch | `mimo/b4i4-rocket-v2-integration` |
| Remote HEAD SHA | `b4bbc6f39f5d779cf087f16f90b61ceb9c6b5193` |
| local HEAD == remote HEAD | CONFIRMED |

## 2. Exact Base SHA

Production correctness floor: `a1d9c3b6ad5cfbb45732f50efcca3231b21ecb4d`

This is the HEAD of `codex/c-b4i3c-canonical-v2-search`, the last accepted Canonical V2 production state before Rocket integration.

## 3. Changed-File Audit

| File | Status | Classification |
| --- | --- | --- |
| `docs/perf/B4I4_ROCKET_V2_INTEGRATION.md` | A | EXPECTED DOC |
| `tethers-0.1/engine-ocaml/bin/tethers_core_canonical_v2_ir.ml` | M | EXPECTED CODE |
| `tethers-0.1/engine-ocaml/bin/tethers_core_canonical_v2_ir_test.ml` | M | EXPECTED TEST |

No UNEXPECTED changes. No changes to: frozen V2 format encoder, frozen V2 spec, validator semantics, V1, replay, runtime, Trail, Tethers language semantics, Together semantics, capability/policy behaviour, C2 concurrency.

## 4. Code Blob Identity Against 92443ac

| File | Candidate blob | Rocket blob | Match |
| --- | --- | --- | --- |
| `tethers_core_canonical_v2_ir.ml` | `08149279a765b5d8aa55a0fc208ed0efe121cb52` | `08149279a765b5d8aa55a0fc208ed0efe121cb52` | BYTE-IDENTICAL |
| `tethers_core_canonical_v2_ir_test.ml` | `049979f72212c02d7e83da981f0c6822e2b29f80` | `049979f72212c02d7e83da981f0c6822e2b29f80` | BYTE-IDENTICAL |

Both production code files are byte-identical to the accepted Rocket candidate `92443ac0420e377154adcf3e12c259b729d394fe`.

## 5. Frozen Vectors

| Vector | Expected payload hash | Actual | Expected digest | Actual | Verdict |
| --- | --- | --- | --- | --- | --- |
| A empty | `03882b01ddaffd0944e1b38e3f55495e8e34d11bc25def374883cc262700c938` | MATCH | `tethers:v2:sha256:750a06eea394bb38eefc073cd77d6c36b291efa13f6ff5173eacce35ca7b4619` | MATCH | PASS |
| B simple | `9dd7aeb4e3bec49aed88ea4844461d0c1cb4846ebc781b7d3816458b8ce3ecdd` | MATCH | `tethers:v2:sha256:1bba9a344584c9b32d066a6de1e69ec196222682546ad7f40c51f04c061e3932` | MATCH | PASS |
| C persistent | `b0877dbca6b7c04634bb9e61fed850e4a832ec60fdfa7b25f51c1185a92a940b` | MATCH | `tethers:v2:sha256:6eae6604bb65580646be8cbc077284cf520c87eecbd81438ae8b4031606eb0f8` | MATCH | PASS |

A SINGLE BYTE CHANGE = STOP. No stop triggered.

## 6. Differential Results

### Dense 5000-case oracle/baseline/Rocket differential

```
Dense generated corpus: seed=308386 total=5000 valid=5000 mismatches=0 archetypes=16
```

oracle payload == baseline payload == Rocket payload for all 5000 cases.
oracle digest == baseline digest == Rocket digest for all 5000 cases.

### Anchor tie matrix

- `repaired minimal Anchor tie mismatch` — PASS
- `Anchor tie torture and residual pre-admission` — PASS
- Tied pair residual 2! — PASS
- Tied triple residual 3! — PASS
- Multiple tie classes — PASS
- 9/10/11/12 decimal boundaries — PASS
- Budget 2 accepts / budget 1 rejects — PASS

### Persistent Branch regression

- 576 → 6 (not 576 → 1) — PASS

### Decimal label boundaries

- `Fact/Program Role/Template Role decimal label boundaries` — PASS

### Raw-ID/storage metamorphism

- `metamorphic raw-ID/storage` — PASS
- `raw-ID rename` — PASS
- `cross-family same raw` — PASS
- `mixed Branch torture hostile raw IDs` — PASS

### Fail-closed budgets

- `deterministic budget fail-closed` — PASS
- `reduced pre-admission 11-Branch shortcut` — PASS (39,916,800 raw → 1 leaf)
- `overflow / max_int safety (21!)` — PASS

### Compound reduction

- `compound factorial factor collapses` — PASS

### Full OCaml suite

- Lowerer: 49/49 PASS
- Validator: 51/51 PASS
- Plan bridge: 179/179 PASS
- Adapter: 43/43 PASS
- Request adapter: 89/89 PASS
- Core wire: 3/3 PASS
- V2 reference oracle: all PASS
- V2 production: all PASS
- V2 IR: all PASS

## 7. Whole-Repo Results

| Gate | Command | Result |
| --- | --- | --- |
| Whitespace | `git diff --check` | PASS |
| Rust formatting | `cargo fmt --check` | PASS |
| Rust compilation | `cargo check` | PASS |
| Rust tests | `cargo test` | 1451 passed, 0 failed, 5 ignored |
| OCaml build | `dune build @all` | PASS |
| OCaml tests | `dune runtest --force` | PASS (all green) |

Working directory: `D:\The Next Thing\Tethers Lang - Goose Integration`
Git HEAD: `b4bbc6f39f5d779cf087f16f90b61ceb9c6b5193`

## 8. Rollback Point

### Production correctness floor

```
a1d9c3b6ad5cfbb45732f50efcca3231b21ecb4d
```

This is the last accepted production state before Rocket. To return to this state after a hypothetical merge:

```powershell
git revert b4bbc6f39f5d779cf087f16f90b61ceb9c6b5193 --no-edit
git revert 4235045cd542cbc65dad092d9bc8c4da7768c95d --no-edit
git push origin main
```

Two ordinary revert commits reverse the integration report and the code change respectively. No destructive history rewriting. No `reset --hard`. No force push. No rewriting shared history.

### Accepted Rocket candidate

```
b4bbc6f39f5d779cf087f16f90b61ceb9c6b5193
```

## 9. Recommended Cutover Operation

**Fast-forward merge.**

The integration branch is a clean descendant of the production base:

```
a1d9c3b (production base)
  └─ 4235045 (code integration)
     └─ b4bbc6f (integration report)
```

No merge commit is needed. A fast-forward preserves the exact commit history, audit trail, and provenance:

```powershell
git checkout main
git merge --ff-only mimo/b4i4-rocket-v2-integration
git push origin main
```

This preserves:
- The code integration commit with its message naming the Rocket candidate
- The integration report commit
- Full blob SHAs for independent verification
- Clean revert path (two revert commits if ever needed)

Alternative: cherry-pick of `4235045` + `b4bbc6f` if main has advanced. But fast-forward is preferred because the branch is already a clean descendant.

Do NOT squash. Squashing would destroy the provenance chain between `4235045` and `92443ac`.

## 10. Explicit C2 Remains Blocked

- Physical concurrency C2 remains **OFF**
- Rocket search remains **single-threaded** (OCaml's `assign_next_ir` is a recursive sequential loop)
- ProgramDigest output is **independent of thread count/hardware** (deterministic `SHA-256` over canonical payload bytes)
- No scheduling information enters canonical bytes (Enc_V2 is a pure function of `(Program, LabelAssignment)`)
- Semantic concurrency (Together fan-out/join) and physical execution remain **separate concerns**

C2 may begin **ONLY AFTER** Canonical V2 cutover is accepted.

## 11. Final Verdict

# READY FOR CUTOVER

All eight phases passed. No stop conditions triggered.

## 12. Final Question

> "Can this exact commit become the canonical V2 production implementation without changing Tethers identity or weakening rollback/auditability?"

**Yes.**

- Tethers identity is preserved: all frozen vectors are byte-identical
- Rollback is preserved: two ordinary revert commits return to pre-Rocket state
- Auditability is preserved: fast-forward merge retains full commit history and blob SHAs
- The exact commit `b4bbc6f39f5d779cf087f16f90b61ceb9c6b5193` is the one verified by this gate

This is a boring production implementation of the already-approved Rocket mathematics.
