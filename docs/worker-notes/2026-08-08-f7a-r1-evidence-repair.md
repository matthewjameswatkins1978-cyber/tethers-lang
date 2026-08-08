# Worker Note

Task: `F7a-R1 — Evidence Repair`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `532126810ad51dfbf6d75472854c9cb49d8d0811`

Implementation checkpoint: `fd0149ffbf83f677534ae0bbf58fdf767381584c`

## Requested outcome

Repair three F7a evidence defects identified by Lucy's independent review: incorrect fixture verification path, unsupported all-features failure causation claim, and premature F7b authorisation. No production, test, fixture, build, protocol, script, or dependency changes.

## Changes made

- `docs/CURRENT_CLINE_TASK.md` — replaced F7a audit packet with F7a-R1 evidence-repair packet
- `docs/foundation-pass/TEST_CONTRACT_RECONCILIATION_F7A.md` — repaired evidence:
  - Recorded F7a final HEAD `532126810ad51dfbf6d75472854c9cb49d8d0811`
  - Corrected fixture verification path to `tethers-0.1/scripts/check-fixtures.ps1` (two occurrences)
  - Replaced "resolved by intervening Foundation work" with "CURRENTLY NOT REPRODUCED — PRIOR CAUSE UNVERIFIED"
  - Documented that `ea7426d..2a2417f5` contains only documentation changes, establishing no causal explanation for changed results
  - Updated section 5 (Failure Classifications) to remove "green" language implying resolution
  - Updated section 11 table: all F7b authorised entries changed to NO
  - Updated section 13 F7 Authorisation Table: M7 changed from YES to NO/DEFER with Dune topology rationale
  - Updated section 14: removed F7b recommendation; replaced with final verdict (NO TEST CONSOLIDATION AUTHORISED, NO F7b, F7 completes as NO-OP)

No Rust, OCaml, build, test, fixture, protocol, script, dependency, or production files changed.

## Decisions and assumptions

1. **M7 DEFER, not F7b.** The OCaml Dune topology has two executables sharing engine modules with no library seam. Introducing clean native tests would require structural Dune/library work whose benefit is unjustified by the trivial direct properties proposed in F7a. External engine/MCP compatibility evidence remains strong.

2. **Failure causation is UNVERIFIED, not resolved.** `ea7426d..2a2417f5` contains three documentation-only commits. No production/test change can causally explain the changed all-features result. Speculation about environment, timing, feature state, or other causes is withheld.

3. **F7 completes as NO-OP.** No test consolidation is authorised. M8 and M9 are correctly resolved or NO-ACTION. M7 is deferred to a future OCaml test/build architecture decision.

## Evidence

### Git state at R1 checkpoint

- `git status --short`: clean
- Branch: `foundation/f7a-r1-evidence-repair`
- R1 checkpoint HEAD: `fd0149ffbf83f677534ae0bbf58fdf767381584c`
- R1 base: `532126810ad51dfbf6d75472854c9cb49d8d0811`
- `git diff --name-only 5321268..fd0149f`: only `docs/CURRENT_CLINE_TASK.md`, `docs/foundation-pass/TEST_CONTRACT_RECONCILIATION_F7A.md`
- `git diff --check`: PASS
- `git diff --name-only -- tethers-0.1/host-rust/`: (empty)
- `git diff --name-only -- tethers-0.1/engine-ocaml/`: (empty)
- `git diff --name-only -- tethers-0.1/protocol/`: (empty)

### Tests at R1 checkpoint

- `cargo test --locked`: PASS — 1331 passed, 0 failed, 2 ignored (22.89s)
- `cargo test --all-targets --all-features --locked`: PASS — 1331 passed, 0 failed, 2 ignored (22.91s)
- `pwsh -NoProfile -File tethers-0.1/scripts/check-fixtures.ps1`: PASS — 46 JSON + 30 JSONL valid

### Packet checker

- `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1` at R1 checkpoint: PASS (control-v1/IN_PROGRESS, base 5321268, HEAD fd0149f)
- `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1` at COMPLETE: (to be run at closeout)

### Repair verification

- Fixture verification path corrected: `grep tethers-0.1/scripts/check-fixtures.ps1` confirms two occurrences in evidence document
- All-features analysis: documented as CURRENTLY NOT REPRODUCED with explicit `ea7426d..2a2417f5` commit range evidence
- F7 authorisation: M7 DEFER, NO F7b AUTHORISED, F7 COMPLETES AS NO-OP
- F7a final HEAD recorded: `532126810ad51dfbf6d75472854c9cb49d8d0811`

## Discoveries

None. All three defects were exactly as Lucy identified: path error, unsupported causation claim, and premature F7b recommendation. No unexpected defects found.

## Remaining risks

- The original F1-R1 six all-features failures remain with unverified cause. The current test surface passes, but the root cause of the earlier failures was not investigated and may re-emerge.
- M7 remains deferred; the OCaml Dune topology limitation (two executables sharing modules, no library seam) blocks clean native tests. This requires a separate design decision.

## Smallest next action

If Lucy accepts this evidence repair, F7 closes as NO-OP. No F7b. Next Foundation pass proceeds as directed by Lucy.

## References

- `docs/CURRENT_CLINE_TASK.md` — F7a-R1 evidence repair packet
- `docs/foundation-pass/TEST_CONTRACT_RECONCILIATION_F7A.md` — repaired evidence document
- R1 base: `532126810ad51dfbf6d75472854c9cb49d8d0811`
- R1 implementation checkpoint: `fd0149ffbf83f677534ae0bbf58fdf767381584c`
- Branch: `foundation/f7a-r1-evidence-repair`
- F1-R1 observation base: `ea7426dbeb1934cf336673d03ae2abf76146ea7d`
- F7a audit base: `2a2417f5d943a5c1ca27d6c646746cfaf7b93a86`
