# Worker Note

Task: `F8-D1 — First Production Dead-Code Warning Cleanup`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `66da0492d0bc681defefa66a92cdb40287dcb05c`

Implementation checkpoint: `9717b3265fdace8500a83a85a8e28359f8be9124`

## Requested outcome

Remove production dead-code warning D1: the unused `PROVISION_USAGE` constant
from `tethers-0.1/host-rust/src/application.rs`. Classification was confirmed as
DEAD through repository-wide reference search and live-cli-path analysis.

## Changes made

- `tethers-0.1/host-rust/src/application.rs`: Deleted `PROVISION_USAGE` constant
  (lines 24-25). Replaced two inline references inside `parse_provision_args`
  (D2) with the string literal. Replaced test assertion reference (line 6739)
  with the string literal. Formatting applied via `cargo fmt`.

## Decisions and assumptions

- Classification: **DEAD**. `PROVISION_USAGE` is only referenced inside
  `parse_provision_args` (D2), itself never called in production. The real
  provisioning route uses Clap-based `CliCommand::ProvisionReplay` which
  generates its own usage strings.
- Consequential adaptations to D2 internals and the test assertion are strictly
  required by the constant's removal and do not resolve D2's dead_code warning.
- D2-D15 remain entirely unchanged in warning identity and count (14 lib
  warnings remain).

## Evidence

- `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all -- --check` — PASS
- `cargo check --all-targets --all-features --locked` — 14 lib warnings (D2-D15), 9 lib test warnings (6 duplicates). D1 absent.
- `cargo clippy --all-targets --all-features --locked` — PASS (pre-existing warnings only)
- `cargo test --all-targets --all-features --locked` — 1589 passed, 0 failed, 2 skipped
- `cargo test j09_runtime_42_provisioning_wrong_shapes_are_rejected_without_mutation` — PASS
- `git diff --check` — PASS (LF/CRLF informational only)
- Packet checker: `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1` — PASS
- `just verify-agent` — PASS (1589 passed, 0 failed, 2 skipped; nextest 41.9s)
- `rg PROVISION_USAGE tethers-0.1/host-rust/src/` — Zero Rust source matches

## Publication evidence

Branch `foundation/f8-d1-provision-usage-cleanup` pushed to `origin`.

## Discoveries

None.

## Remaining risks

None known within packet scope. D2-D15 remain unresolved and retain their
existing production dead_code warnings.

## Smallest next action

Resolve D2 (`parse_provision_args`) or D3-D15 as prioritised by Lucy in a
follow-on packet.

## References

- `tethers-0.1/host-rust/src/application.rs` — constant removed at line 24
- `tethers-0.1/host-rust/src/cli.rs` — Clap-based provisioning route (lines 59-63)
- `docs/CURRENT_CLINE_TASK.md` — F8-D1 packet
