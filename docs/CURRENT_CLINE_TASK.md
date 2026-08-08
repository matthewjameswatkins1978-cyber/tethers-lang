# Current Implementation Task

Control contract: `1`
Task: `F3e1 - Trail evidence harvest`
Owner: `DeepSeek Pro HIGH`
Model: `DeepSeek Pro HIGH`
Status: `COMPLETE`
Task colour: `Amber`
Route: `OpenCode completed F3e1 Trail evidence harvest; do not describe all F3e as complete`
Worker note: `docs/worker-notes/2026-08-07-f3e1-trail-evidence.md`
Base branch: `main`
Base commit: `c9332bab072ce273db3aecc367faf64be71a8586`
Implementation branch: `foundation/f3e1-trail-evidence`
Parent branch: `main`
Parent tip: `c9332bab072ce273db3aecc367faf64be71a8586`
Implementation checkpoint: `fb07c607a5c938d326489a03a7e1b474d6e88461`
OCaml switch path: `N/A`
Rust toolchain: read exact channel from `rust-toolchain.toml`; use plain Cargo (resolved by root pin); `--locked` mandatory
Toolchain preflight: `pwsh -NoProfile -File scripts/check-dev-tools.ps1`

## Objective

Audit Trail/FileTrail only as the append-only causal-log persistence store.
This was an evidence harvest, not a redesign and not a general persistence task.

First answer: what is already directly proved about Trail, and what remains genuinely unverified?

## F3e1 scope

Trail only — `FileTrail` (dispatch.rs:320-405) as writer and `run_trail()` (trail_command.rs:27) as production reader. Replay was not touched.

## Relevant components

- `tethers-0.1/host-rust/src/dispatch.rs` — FileTrail, Trail trait, inline tests
- `tethers-0.1/host-rust/src/trail_command.rs` — production reader, inline tests
- `docs/foundation-pass/PERSISTENCE_INVENTORY.md` — F3b Trail row

## Evidence dimensions checked

1. Append order
2. One JSONL record per completed write
3. Flush/sync behaviour already established by F3b
4. Ordinary close/reopen readback
5. Truncated final-line behaviour
6. Malformed complete-line behaviour
7. Production reader classification of malformed/truncated records
8. Execution_id filtering behaviour
9. Whether previous valid lines remain readable when the final line is damaged
10. Path safety actually provided by FileTrail::open()

## Relevant background and existing behaviour

F3a classified Trail as an append-only causal log. F3b characterized Windows flush/sync primitives and established that truncated final lines are present and non-parseable by raw serde_json, but the production Trail reader (trail_command.rs:run_trail()) was not exercised. F3d explicitly excluded Trail and Replay.

## Required behaviour

1. F3e1-1 — Harvest existing Trail tests and map every property to PROVEN/DISPROVEN/UNVERIFIED with exact test citations and hard assertions.
2. F3e1-2 — Exercise the production reader with a truncated final line to close the remaining F3b UNVERIFIED gap.
3. F3e1-3 — Characterize path safety actually provided by FileTrail::open().
4. F3e1-4 — Record exact remaining UNVERIFIED properties. Do not upgrade F3b claims.
5. F3e1-5 — No production code changed. Replay untouched.

## Frozen decisions and invariants

- Accepted F3d main: `c9332bab072ce273db3aecc367faf64be71a8586`
- F3b UNVERIFIED platform properties preserved
- Trail is append-only causal log — no conversion to atomic records
- No production code redesign
- Replay untouched

## F3e1 findings

Three characterization tests added (tests only, no production changes):

1. `f3e1_truncated_final_line_maps_to_audit_failed` (trail_command.rs) — production reader classifies truncated final line as TRAIL_INVALID (fail-closed). Was F3b UNVERIFIED; now PROVEN.
2. `f3e1_file_trail_open_has_no_path_validation` (dispatch.rs) — FileTrail::open() has no root/reparse/chain validation.
3. `f3e1_file_trail_open_accepts_relative_path` (dispatch.rs) — FileTrail::open() accepts relative paths without validation. Path validation inside FileTrail::open is DISPROVEN.

No defect found. Replay untouched. 58 Trail tests pass.

## Remaining UNVERIFIED

- Power-loss durability: UNVERIFIED (F3b) — never upgrade
- Directory-entry durability: UNVERIFIED (F3b) — never upgrade
- Parent-directory flush in production: DISPROVEN (F3b)

## Forbidden changes

- Touch Replay
- Redesign Trail
- Convert Trail to atomic records
- Add checksums/digests
- Add persistence abstraction
- Modify CLI/JSON/Trail public shape
- Change F1 fixtures
- Upgrade F3b UNVERIFIED claims
- Begin F4 or F3e2

## Stop conditions

STOP if:
- `origin/main` differs from `c9332bab072ce273db3aecc367faf64be71a8586`
- A required property cannot be characterized
- A repair would require redesign outside F3e1
- A required check fails
- Two materially similar attempts fail

## Expected pre-existing changes

None

## Acceptance criteria

1. Trail evidence map across 10 dimensions.
2. Exact remaining UNVERIFIED properties recorded.
3. PERSISTENCE_INVENTORY.md updated with F3e1 Trail evidence.
4. F3e1 worker note records exact evidence and findings.
5. No production code changed.
6. Replay untouched.

## Required verification

```powershell
cargo fmt --all -- --check
cargo test --lib -- trail
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1
git diff --check
```
