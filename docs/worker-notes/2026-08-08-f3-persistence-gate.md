# Worker Note

Task: `F3-GATE — Combined Persistence Contract Reconciliation`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `DeepSeek Pro HIGH`

Status: `COMPLETE`

Base commit: `ab58c83ba44680f3003db333f1e1ffd091aa5b3f`

Implementation checkpoint: `3215bdcadfcde5ec1ef49c5076e758da6744a374`

## Requested outcome

Perform the final combined review required to close Foundation Pass F3. Answer: "Taken together, does the accepted F3 evidence describe one internally consistent persistence contract, with no contradictory durability, atomicity, recovery, corruption, or store-classification claims?"

## Changes made

- Updated `docs/CURRENT_CLINE_TASK.md` to describe F3-GATE with full contradiction ledger and final durability matrix.
- Added F3-GATE combined contract reconciliation section to `docs/foundation-pass/PERSISTENCE_INVENTORY.md`.
- Created this worker note.
- No production code changed. Zero tests added (expected: 0).

## Decisions and assumptions

- F3a–F3e2b evidence was already accepted independently. This gate task reconciles them as one whole without re-auditing individual packages.
- Two minor non-contradictions noted (stale F3e3 forward-reference in F3e2a, F3a architecture text predating F3d evidence). Neither is material enough to warrant edit.
- Store counts and classifications verified consistent: 14 stores (9+1+1+3), 1 coordination artifact, 6 in-memory entries.
- All F3b UNVERIFIED/DISPROVEN boundaries preserved across all store families.

## Evidence — contradiction ledger

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

## Discoveries

- The F3 evidence corpus is internally consistent. Zero contradictions were found across 12 audit areas.
- All F3 sections preserve the same UNVERIFIED/DISPROVEN boundaries for platform durability properties.
- Store counts and classifications are consistent: 14 stores (9+1+1+3), 1 coordination artifact, 6 in-memory entries.
- Two minor non-contradictions observed: (1) F3e2a line 422 references "F3e3" as a provisional name — F3e2b completed Generations coverage, making this stale but harmless; (2) line 54 "Key Differences From F1 Inventory" describes StoreRoot architecture predating F3d bounded evidence — no contradiction, just architectural text followed by evidence.
- No production defects surfaced from the combined review.

## Final UNVERIFIED/DISPROVEN durability matrix

| Property | Final F3 status | Scope |
|---|---|---|
| Atomic visibility during rename | UNVERIFIED (F3b) | All stores using rename |
| File data survives sudden power loss | UNVERIFIED (F3b) | All stores |
| Directory entry survives power loss | UNVERIFIED (F3b) | All stores |
| Parent-directory flush in production | DISPROVEN (F3b) | All stores |
| Exact bytes survive ordinary close/reopen | PROVEN where directly tested; UNVERIFIED otherwise | Varies by store |
| Root reparse-point defence | PROVEN (StoreRoot/Replay); DISPROVEN (Local Anchor); UNVERIFIED (Candidate Registry) | Varies by store |

## Remaining risks

- Platform durability properties (power-loss, directory-entry) remain UNVERIFIED — these depend on system-level hardware/OS behaviour not testable in the current environment.
- Parent-directory flush is DISPROVEN — every store family shares the same gap.
- F3b UNVERIFIED/DISPROVEN boundaries remain the authoritative durability contract.

## Confirmation

- No production code changed.
- No tests added (expected: 0).
- F4 was not started.
- No earlier worker notes edited.
- Earlier task packets remain as historical records.

## Smallest next action

Independent gate review by Lucy. The combined F3 persistence contract is READY FOR INDEPENDENT GATE REVIEW.

## References

- `docs/architecture/TETHERS_FOUNDATION_PASS.md` — Foundation Pass programme, F3 gate
- `docs/foundation-pass/PERSISTENCE_INVENTORY.md` — complete F3 evidence corpus
- Accepted worker notes: F3a, F3b, F3c, F3d, F3e1, F3e2a, F3e2b
