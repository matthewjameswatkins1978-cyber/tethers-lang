# Current Implementation Task

Control contract: `1`
Task: `F3-GATE — Combined Persistence Contract Reconciliation`
Owner: `DeepSeek Pro HIGH`
Model: `DeepSeek Pro HIGH`
Status: `COMPLETE`
Task colour: `Amber`
Route: `OpenCode implements F3 combined persistence gate reconciliation; do not begin F4`
Worker note: `docs/worker-notes/2026-08-08-f3-persistence-gate.md`
Base branch: `main`
Base commit: `ab58c83ba44680f3003db333f1e1ffd091aa5b3f`
Implementation branch: `foundation/f3-persistence-gate`
Implementation checkpoint: `WORKTREE`
Parent branch: `foundation/f3e2b-replay-generations-evidence`
Parent tip: `ab58c83ba44680f3003db333f1e1ffd091aa5b3f`
OCaml switch path: `N/A`
Rust toolchain: read exact channel from `rust-toolchain.toml`; use plain Cargo (resolved by root pin); `--locked` mandatory
Toolchain preflight: `pwsh -NoProfile -File scripts/check-dev-tools.ps1`

## Objective

Perform the final combined review required to close Foundation Pass F3. Answer one question: "Taken together, does the accepted F3 evidence describe one internally consistent persistence contract, with no contradictory durability, atomicity, recovery, corruption, or store-classification claims?"

This is a reconciliation/gate task. Do not re-run F3a–F3e2b as new implementation work. Do not add tests. Do not change production code.

## Scope and accepted base

Review the accepted F3 persistence evidence as one whole:
- F3a — inventory and vocabulary (`PERSISTENCE_INVENTORY.md` initial pass + r2 correction)
- F3b — Windows primitive evidence (sync, rename, directory, reparse)
- F3c — installation intent and publication contract
- F3d — remaining bounded immutable/current-state stores
- F3e1 — Trail evidence harvest
- F3e2a — Replay Claim evidence harvest
- F3e2b — Replay Generations & Recovery evidence harvest

Authoritative documents:
- `docs/architecture/TETHERS_FOUNDATION_PASS.md`
- `docs/foundation-pass/PERSISTENCE_INVENTORY.md`
- Accepted F3 worker notes where needed for provenance

## Relevant components

- `docs/architecture/TETHERS_FOUNDATION_PASS.md` — Foundation Pass programme, F3 gate requirement
- `docs/foundation-pass/PERSISTENCE_INVENTORY.md` — complete F3a–F3e2b evidence corpus
- Accepted worker notes: F3a, F3b, F3c, F3d, F3e1, F3e2a, F3e2b

## Relevant background and existing behaviour

F3a established the four-class persistence vocabulary (immutable atomic record, replaceable current-state record, append-only causal log, multi-step intent/recovery journal) and classified 14 stores, 1 coordination artifact, and 6 in-memory entries. F3b provided Windows primitive evidence across 5 clusters (StoreRoot sync+rename, parent-directory flush feasibility, Replay publish primitive, Trail JSONL interruption, Local Anchor reparse safety). F3c–F3e2b applied this vocabulary to bounded store families. The combined corpus spans 14 distinct persistence stores with consistent classification.

## Required behaviour

1. F3-GATE-1 — Audit store vocabulary for reclassification or count drift across all F3 sections.
2. F3-GATE-2 — Verify atomic visibility is never claimed PROVEN for any rename primitive.
3. F3-GATE-3 — Verify file-data durability across power loss is never claimed PROVEN.
4. F3-GATE-4 — Verify directory-entry durability across power loss is never claimed PROVEN.
5. F3-GATE-5 — Verify parent-directory flush DISPROVEN is consistently recorded.
6. F3-GATE-6 — Verify ordinary close/reopen evidence is not conflated with power-loss or atomicity.
7. F3-GATE-7 — Verify Trail remains append-only causal log (not converted to atomic record).
8. F3-GATE-8 — Verify Replay Claim/Generations retain distinct semantics (not genericised).
9. F3-GATE-9 — Verify installation semantics remain installation-specific (not universalised).
10. F3-GATE-10 — Verify corruption/fail-closed claims are bounded to tested stores.
11. F3-GATE-11 — Verify path-safety claims are attributed to the correct protection layer.
12. F3-GATE-12 — Produce final UNVERIFIED/DISPROVEN durability matrix.

## Frozen decisions and invariants

- Accepted F3e2b-R1 base: `ab58c83ba44680f3003db333f1e1ffd091aa5b3f`
- F3a–F3e2b evidence already accepted — do not re-audit
- No production code changes
- No new tests (expected: 0)
- Do not begin F4
- Do not state ACCEPTED — only READY FOR INDEPENDENT GATE REVIEW

## Contradiction ledger

Audited 12 areas. Zero material contradictions found.

| # | Area | Claim A | Potential conflict | Verdict | Evidence |
|---|---|---|---|---|---|
| 1 | Store vocabulary | PERSISTENCE_INVENTORY.md r2: 14 stores (9+1+1+3), 1 artifact, 6 in-memory | F3e2a/F3e2b addition of Replay stores | NO — Replay Claim was already counted in the 9 immutable atomic records (line 26); Replay Generations in the 3 multi-step journals (line 50). F3e2a/F3e2b evidence sections are consistent with these classifications. | PERSISTENCE_INVENTORY.md:26, :50, :395–424 |
| 2 | Atomic visibility | Every F3 section: "atomic visibility guarantee UNVERIFIED (F3b)" | Any claim of PROVEN rename atomicity | NO — zero sections claim PROVEN atomic visibility. "Write-then-rename" is factual description, not proof. | F3b:121,187–204; F3c:302; F3e1:364; F3e2a:411; F3e2b:§Remaining UNVERIFIED |
| 3 | File data durability (power loss) | Every F3 section: "sudden power loss: UNVERIFIED (F3b)" | Any claim flush/sync proves survival | NO — every section records UNVERIFIED. FlushFileBuffers success is not equated to power-loss survival. | F3b:122,196; F3c:302; F3d:317; F3e1:365 |
| 4 | Directory-entry durability | Corrected evidence matrix: "Directory entry durable after power loss: UNVERIFIED" | Any implicit claim from rename testing | NO — consistent UNVERIFIED across all primitive classes and store families. | F3b:123,197 |
| 5 | Parent-directory flush | "Production performs parent-directory flush: DISPROVEN (F3b)" | Converting to "directory durability is disproven" | NO — documented as "Production performs parent-directory flush: DISPROVEN." Never converted to "therefore directory durability is disproven." | F3b:133,203 |
| 6 | Ordinary close/reopen | PROVEN for exact bytes where tested; properly separated from UNVERIFIED power-loss | Close/reopen used to imply crash survival | NO — every store family separates "exact bytes survive close/reopen (PROVEN)" from "sudden power loss (UNVERIFIED)." | F3b:115,121; F3e2b dim #14 |
| 7 | Trail semantics | Appendix: "Append-Only Causal Log" (line 42) | Trail reclassified as atomic record | NO — Trail remains append-only causal log throughout. F3e1 findings preserve JSONL semantics. | PERSISTENCE_INVENTORY.md:42; F3e1:345–375 |
| 8 | Replay semantics | Claim as immutable atomic record (line 26), Generations as multi-step journal (line 50) | Generic "atomic store" language erasing Replay-specific meaning | NO — F3e2a preserves Claim as immutable identity record with execution_id/claim_digest semantics. F3e2b preserves G0/G1/G2 chain, predecessor linkage, reconstruction, and recovered-admission immutability. | F3e2a:395–408; F3e2b table |
| 9 | Installation semantics | F3c: specialised J24K intent/recovery contract | Universalised to generic persistence abstraction | NO — F3c section is explicitly bounded to installation intent/publication/recovery. Not extended to other stores. | F3c:221–305 |
| 10 | Corruption/fail-closed language | Each section bounds claims to tested store | Universal "all corruption is detected" | NO — F3d explicitly states "Every dimension not named in the PROVEN column is UNVERIFIED." Corruption claims are store-specific. | F3d:316 |
| 11 | Path safety | Replay host-root, StoreRoot chain verification, Trail callers, Local Anchor (DISPROVEN root reparse) | Cross-store safety transfer | NO — Replay host-root protection is Replay-specific. StoreRoot verification covers its 7 stores. Trail has no internal path validation (stays with callers). Local Anchor root reparse safety is honestly DISPROVEN. | Replay: PERSISTENCE_INVENTORY.md:26; Trail: F3e1; Local Anchor: F3b:178 |
| 12 | Combined contract alignment | "F3 cannot pass until every subpackage has independent evidence and the combined contract has no contradictory durability claim" (TETHERS_FOUNDATION_PASS.md:143-144) | Evidence corpus under review | NO — All subpackages (F3a–F3e2b) have independent evidence recorded in PERSISTENCE_INVENTORY.md with consistent UNVERIFIED/DISPROVEN boundaries across all store families and primitive classes. | Entire F3 corpus |

### Observed non-contradictions (minor)

- F3e2a line 422: "Replay Generations are explicitly deferred to F3e2b/F3e3" — F3e3 was a provisional name; F3e2b completed Generations coverage. No contradiction, just stale forward-reference. Not material enough to warrant edit.
- Line 54 (end of file) "Key Differences From F1 Inventory" describes StoreRoot as common persistence layer for 7 stores. This predates the F3d evidence that added explicit testing for each of those 7 stores. F3d is appended after this section. No contradiction — the F3a text describes the architecture; F3d provides the bounded evidence.

## F3-GATE findings

**Combined contract verdict: READY FOR INDEPENDENT GATE REVIEW**

The accepted F3 evidence describes one internally consistent persistence contract:
- 14 classified persistence stores (9 immutable atomic + 1 replaceable current-state + 1 append-only causal log + 3 multi-step intent/recovery journals), 1 filesystem coordination artifact, 6 in-memory entries — consistent across all sections
- Atomic visibility during rename: UNVERIFIED across all stores
- File data survival across sudden power loss: UNVERIFIED across all stores
- Directory-entry durability across power loss: UNVERIFIED across all stores
- Parent-directory flush in production: DISPROVEN across all stores
- Ordinary close/reopen: PROVEN where directly tested, properly distinguished from power-loss/crash durability
- Correction/fail-closed claims bounded to tested stores
- Path safety attributed to correct protection layers
- No reclassification of Trail, Installation Execution Lock, Installation Recovery Plan, or Candidate Registry
- No universalisation of installation or Replay semantics into generic abstractions

### Final UNVERIFIED/DISPROVEN durability matrix

| Property | StoreRoot-backed (7) | Candidate Registry | Replay (Claim+Gen) | Trail | Local Anchor | Final F3 status |
|---|---|---|---|---|---|---|
| Exact bytes survive ordinary close/reopen | PROVEN (F3b) | UNVERIFIED (F3d) | PROVEN (F3e2a/#14) | PROVEN (F3e1) | UNVERIFIED | Varies by store |
| Atomic visibility during rename | UNVERIFIED (F3b) | UNVERIFIED (F3b) | UNVERIFIED (F3b) | N/A (no rename) | UNVERIFIED (F3b) | UNVERIFIED |
| File data survives sudden power loss | UNVERIFIED (F3b) | UNVERIFIED (F3b) | UNVERIFIED (F3b) | UNVERIFIED (F3b) | UNVERIFIED (F3b) | UNVERIFIED |
| Directory entry survives power loss | UNVERIFIED (F3b) | UNVERIFIED (F3b) | UNVERIFIED (F3b) | UNVERIFIED (F3b) | UNVERIFIED (F3b) | UNVERIFIED |
| Production performs parent-directory flush | DISPROVEN (F3b) | DISPROVEN (F3b) | DISPROVEN (F3b) | DISPROVEN (F3b) | DISPROVEN (F3b) | DISPROVEN |
| Root reparse-point defence | PROVEN (StoreRoot) | UNVERIFIED (F3d) | PROVEN (ValidatedHostRoot) | N/A (caller-managed) | DISPROVEN (F3b) | Varies by store |
| Malformed input fail-closed | PROVEN (F3d) | PROVEN (F3d) | PROVEN (F3e2a/#7, F3e2b/#12) | PROVEN (F3e1) | PROVEN (F3d) | Varies by store |
| Digest corruption rejected | PROVEN | PROVEN (F3d) | PROVEN (F3e2a/#8) | N/A (no digest) | PROVEN (F3d) | Varies by store |
| Orphan/partial state rejected | N/A | N/A | PROVEN (F3e2b/#5, #12) | PROVEN (F3e1) | PROVEN (F3d) | Varies by store |
| Recovery correct state reconstruction | N/A | N/A | PROVEN (F3e2b/#10, #11) | N/A | PROVEN (F3d) | Varies by store |

## Forbidden changes

- No production code changes
- No new tests
- No re-audit of accepted F3a–F3e2b evidence
- No starting F4
- No stating ACCEPTED (Lucy performs independent acceptance)
- No changing durability or atomicity status
- No rewriting existing worker notes

## Stop conditions

STOP if:
- A material contradiction is found that cannot be resolved by documentation edit
- A contradiction implies production code inconsistency
- Any code or test file changes unexpectedly
- Two materially similar attempts fail

## Expected pre-existing changes

None

## Acceptance criteria

1. Store vocabulary consistent: 14 stores (9+1+1+3), 1 artifact, 6 in-memory across all F3 sections (F3-GATE-1).
2. Atomic visibility never claimed PROVEN for any rename primitive (F3-GATE-2).
3. File-data durability across power loss never claimed PROVEN (F3-GATE-3).
4. Directory-entry durability across power loss never claimed PROVEN (F3-GATE-4).
5. Parent-directory flush DISPROVEN consistently recorded across all stores (F3-GATE-5).
6. Ordinary close/reopen evidence not conflated with power-loss or atomicity (F3-GATE-6).
7. Trail remains append-only causal log, not converted to atomic record (F3-GATE-7).
8. Replay Claim/Generations retain distinct semantics, not genericised (F3-GATE-8).
9. Installation semantics remain installation-specific, not universalised (F3-GATE-9).
10. Corruption/fail-closed claims bounded to tested stores (F3-GATE-10).
11. Path-safety claims attributed to correct protection layer (F3-GATE-11).
12. Final UNVERIFIED/DISPROVEN durability matrix produced, combined-gate section appended to PERSISTENCE_INVENTORY.md, worker note created, no production code changed, no new tests, status READY FOR INDEPENDENT GATE REVIEW (F3-GATE-12).

## Required verification

```powershell
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1
git diff --check
```
