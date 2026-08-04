Task: `M01C4 - Application CLI import suppression cleanup`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `OpenCode`
Status: `COMPLETE`
Base commit: `966ff269ee06f6182bd6029ffe1919b0a43acda8`
Implementation checkpoint: `976d2519c1629c751f219a246cba0328ac90efb3`
Branch: `opencode/m01c4-application-cli-import-suppression` (created from `origin/main` at `966ff26`; implementation commit `976d251`)

## Requested outcome

Remove the blanket `#[allow(unused_imports)]` attached to the CLI import group at
the top of `tethers-0.1/host-rust/src/application.rs` and replace it with an
honest import layout that reflects each symbol's real production, debug-only, or
test-only use. End with no new warning and no replacement suppression.

## Changes made

- `tethers-0.1/host-rust/src/application.rs` (lines 21-22): deleted the
  `#[allow(unused_imports)]` attribute immediately above the CLI import. The
  import itself is unchanged:

  ```rust
  use tethers_reference_host::cli::{Cli, CliEnvelope, Command as CliCommand, OutcomeStatus};
  ```

- No other change in `application.rs`. The `CliCommand` alias is preserved
  because it is used. All four symbols remain imported and used exactly as
  before; only the redundant suppression attribute was removed.

## Decisions and assumptions

- Classified every one of the four imported symbols through the bounded `rg`
  reference search required by the packet (see classification below).
- Concluded all four symbols are always-compiled production use, so the honest
  layout is an ordinary import and the attribute is simply redundant (blueprint
  repair rule 5: "If all four symbols are genuinely needed in every compiled
  target and the attribute is redundant, remove only the attribute").
- Did not add any `#[cfg(...)]` import gate, because none of the symbols is
  debug-only, test-only, or unused — gating would have been a fake boundary.
- Did not add any replacement `#[allow]`/`#[expect]`, dummy use, underscore
  import, or other concealment (all forbidden by the blueprint).
- Did not retry OpenCode LSP (proven ineffective in M01C1; carried forward by
  the packet and blueprint).

## Evidence

Bounded reference search (the exact search the packet recommends):

```text
rg -n --glob 'application.rs' '\b(Cli|CliEnvelope|CliCommand|OutcomeStatus)\b' tethers-0.1/host-rust/src
```

Result: the import is at line 22; all four symbols are used only inside
`pub fn run()` (line 532, ungated, part of `pub mod application;` in `lib.rs`).
No use appears inside a `#[cfg(test)]` or `#[cfg(debug_assertions)]`-only
region, and no use is missing. Confirmed `application.rs` is compiled in all
targets because `lib.rs` declares `pub mod application;` unconditionally and
`run()` is `pub` and ungated.

Clippy JSON capture (machine-readable, `--all-targets --all-features --locked`):

- before edit: total **118** emitted warnings, **0** in `application.rs`, **0**
  at the target import block (the blanket allow was suppressing any
  unused-import diagnostic there).
- after edit: total **118** emitted warnings, **0** in `application.rs`, **0**
  at the target import block. The attribute is gone and no `unused_imports`
  warning appeared because every imported symbol is genuinely used in the
  always-compiled `run()` body.

`cargo fmt --all --check` exit 0. `just verify` exit 0 (full Cargo graph:
926 lib unit tests passed plus every integration suite green, including
`j13a_cli` which exercises the CLI including `OutcomeStatus` values).

Note on a transient environmental failure: the first `just verify` run showed one
integration test, `m3_trust_launch_and_conformance_evidence_cannot_cross_candidates`
(in `tests/m3_lifecycle.rs`), panicking with `Os { code: 5, kind:
PermissionDenied, message: "Access is denied." }` at an ACL operation. This is a
Windows permission/antivirus flake in an unrelated trust/conformance lifecycle
test and is not reachable from the one-line import change (the test compiled and
ran to a `Result::unwrap()` on an OS error). A second `just verify` run passed
cleanly with that same test `ok`. The Cargo floor (926 passing) is met and no
test disappeared because of this task.

## Exact symbol-use classification

| Symbol | Classification | Evidence |
| --- | --- | --- |
| `Cli` | always-compiled production | `Cli::try_parse_from` at line 545; `match cli { Ok(Cli { .. }) }` destructure at 547 — both in `run()`, ungated |
| `CliEnvelope` | always-compiled production | `CliEnvelope::error(...)` at 638, 660, 673, 689, 705, 719, 732, 753, 763 — inside `run()`; arms at 638/753/763 are ungated |
| `CliCommand` | always-compiled production | `CliCommand::Check`/`Plug`/`Run`/`Legacy`/`ProvisionReplay`/`Trail` at 549, 556, 614, 632, 650, 743 and others — inside `run()`; arms at 549/556/614/632/650/743 are ungated |
| `OutcomeStatus` | always-compiled production | `OutcomeStatus::Failed` (645), `OutcomeStatus::Unavailable` (680), `OutcomeStatus::InvalidCliUsage` (760, 770) each appear in ungated arms; debug-gated arms only add more uses |

None of the four is debug-only, test-only, both, or unused.

## Before/after warning table

| | total emitted | `application.rs` warnings | target import-block warnings |
| --- | --- | --- | --- |
| before | 118 | 0 | 0 (suppressed by allow) |
| after | 118 | 0 | 0 (genuinely unused — none) |

Total unchanged; the target blanket suppression is removed; no replacement
suppression added; no new warning introduced.

## Focused-test decision

No separate focused test filter was invented. The reference classification
showed the four symbols are used only in the always-compiled `pub fn run()`, which
is exercised by the existing integration suites already run by `just verify`
(`j13a_cli` etc., including `j13a_outcome_status_values_correct`). Import
configuration is fully validated by compilation plus the full Cargo test graph,
so a ceremonial isolated filter would add no relevant evidence (per the blueprint
"Focused tests" guidance).

## Final Cargo evidence

- `just verify` exit 0 (final authority).
- 926 lib unit tests passed; 0 failed.
- All integration suites passed (j13a_cli 29, j23* 40, j24* 99, m3_lifecycle 13,
  m4_file_tools 4, m5_local_anchor 1, and the provider suites) — 0 failed.
- `cargo fmt --all --check` exit 0.
- `cargo clippy --all-targets --all-features --locked` exit 0; 118 warnings, none
  new, none in `application.rs`.

## Cargo.lock hash

`D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB` (byte-for-byte
unchanged from the M01C3 baseline).

## Remaining risks

None for this task. The edit removes only a redundant attribute; the imported
symbols and all CLI parsing, command routing, debug probes, serialization,
output, exit codes, and tests are unchanged in observable behaviour.

## Smallest next action

None. Task complete; ready for Lucy's independent review.

## References

- Packet: `docs/CURRENT_CLINE_TASK.md`
- Blueprint: `docs/architecture/M01C4_APPLICATION_CLI_IMPORT_SUPPRESSION_CLEANUP.md`
- Relevant source: `tethers-0.1/host-rust/src/application.rs` (import at line 22;
  CLI dispatch in `pub fn run()`), `tethers-0.1/host-rust/src/lib.rs`
  (`pub mod application;`)
- `justfile` — final Cargo verification route
- Implementation commit: `976d2519c1629c751f219a246cba0328ac90efb3`
- Branch: `opencode/m01c4-application-cli-import-suppression`
