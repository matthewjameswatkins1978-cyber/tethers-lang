# F1 Fixture Manifest

All fixtures are hand-reviewed literal artifacts, independently committed. No fixture is generated or refreshed by the implementation under test. Fixture changes require an explicit compatibility decision.

Source capture method: Manual capture from `tethers-reference-host.exe` built at baseline `24428139807cac0adeb0b62264547e61ca809d16`.

## CLI Help

| Fixture | Path | Source | Notes |
|---|---|---|---|
| Top-level help | `fixtures/cli-help/help.txt` | `tethers-reference-host.exe --help` | JSON-enveloped output; even `--help` produces a JSON envelope |

## CLI Outputs

| Fixture | Path | Source | Notes |
|---|---|---|---|
| Version output | `fixtures/cli-output/version.txt` | `tethers-reference-host.exe --version` | JSON envelope with version |
| Check command (missing config) | `fixtures/cli-output/check-missing-config.txt` | `tethers-reference-host.exe check` | Error envelope |
| Run command help | `fixtures/cli-output/run-help.txt` | `tethers-reference-host.exe run --help` | Sub-command help |

## Exit Cases

| Fixture | Path | Exit Code | Notes |
|---|---|---|---|
| Exit 0 (no command) | `fixtures/exit-cases/no-command.txt` | 0 | Envelope with error, but exit 0 per J13A contract |
| Exit 2 (invalid CLI usage) | `fixtures/exit-cases/exit-2.txt` | 2 | `--help` returns exit 2 per Clap convention |

## JSON Envelopes

| Fixture | Path | Schema |
|---|---|---|
| Error envelope (missing config) | `fixtures/json-envelopes/error-missing-config.json` | `tethers.cli/1` |
| Success check envelope (stub) | `fixtures/json-envelopes/success-check-template.json` | `tethers.cli/1` |

## Trail Records

| Fixture | Path | Notes |
|---|---|---|
| Trail record shape | `fixtures/trail-records/trail-shape.json` | Documented from `SPEC.md` and `dispatch.rs` |

## Replay Digests

| Fixture | Path | Notes |
|---|---|---|
| Replay digest shape | `fixtures/replay-digests/digest-shape.txt` | Documented from `replay.rs` conventions |

## Installation Outcomes

| Fixture | Path | Notes |
|---|---|---|
| Installation outcome status values | `fixtures/installation-outcomes/status-codes.txt` | From `outcome.rs` and `run_command.rs` |

## Recovery States

| Fixture | Path | Notes |
|---|---|---|
| Recovery state transitions | `fixtures/recovery-states/transitions.txt` | From `replay_windows.rs` ledger generations 0-3 |

## Fixture Independence Guarantee

All fixtures were manually captured from a single warm binary build at the baseline commit. They are committed as fixed `.txt` and `.json` files. No `update-fixtures` script, golden-file refresh command, or auto-regeneration mechanism exists. Any future fixture change requires:

1. A deliberate compatibility decision;
2. Manual review of the diff;
3. An explicit task packet authorizing the change.
