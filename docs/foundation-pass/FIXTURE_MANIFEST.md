# F1 Fixture Manifest

All fixtures are hand-reviewed literal artifacts from accepted baseline sources. No fixture is generated or refreshed by the implementation under test. Fixture changes require an explicit compatibility decision.

Baseline: `24428139807cac0adeb0b62264547e61ca809d16` (`origin/main`)

## CLI Help

| Fixture | Path | Owner/Contract | Source Command | Exit Code | Capture Method | Review Purpose |
|---|---|---|---|---|---|---|
| Top-level help | `fixtures/cli-help/help.txt` | J13A | `tethers-reference-host.exe --help` | 2 | Binary capture at baseline (warm build) | Verify JSON envelope wraps all output |

## CLI Outputs

| Fixture | Path | Owner/Contract | Source Command | Exit Code | Capture Method | Normalisation |
|---|---|---|---|---|---|---|
| Version output | `fixtures/cli-output/version.txt` | J13A | `tethers-reference-host.exe --version` | 2 | Binary capture at baseline | Exact bytes |
| Check missing config | `fixtures/cli-output/check-missing-config.txt` | J13A | `tethers-reference-host.exe check` (no args) | 2 | Binary capture at baseline | Exact bytes |
| Run help | `fixtures/cli-output/run-help.txt` | J13A | `tethers-reference-host.exe run --help` | 2 | Binary capture at baseline | Exact bytes |

## Exit Cases

| Fixture | Path | Owner/Contract | Source Command | Exit Code | Notes |
|---|---|---|---|---|---|
| No command | `fixtures/exit-cases/no-command.txt` | J13A | `tethers-reference-host.exe` (no args) | 2 | Error envelope with full usage |
| Invalid CLI usage | `fixtures/exit-cases/exit-2.txt` | J13A | `tethers-reference-host.exe` (no args) | 2 | Clap returns exit code 2 per convention |
| Check no config | `fixtures/exit-cases/check-no-config.txt` | J13A | `tethers-reference-host.exe check` | 2 | Missing required args |

## JSON Envelopes

| Fixture | Path | Owner/Contract | Source | Exit Code | Capture Method | Review Purpose |
|---|---|---|---|---|---|---|
| Success: plug list (empty) | `fixtures/json-envelopes/success-plug-list.json` | J13A/J24B | `tethers-reference-host.exe plug list --host-data-root <tmp>` | 0 | Binary capture at baseline (warm build) | Concrete success envelope with `status: "ok"`, `exit_code: 0` |
| Error: missing config | `fixtures/json-envelopes/error-missing-config.json` | J13A | `tethers-reference-host.exe --help` | 2 | Binary capture at baseline | Error envelope with `status: "invalid_cli_usage"`, `exit_code: 2` |

## Trail Records

| Fixture | Path | Owner/Contract | Source | Review Purpose |
|---|---|---|---|---|
| Trail: matched evaluation | `fixtures/trail-records/trail-matched.json` | SPEC 0.1 / J09 | `tethers-0.1/protocol/expected-response.json` (committed at baseline) | Literal happy-path evaluation trail: 5 entries, `status: "matched"`, plan with `lantern.task.record` action |
| Trail: not-matched evaluation | `fixtures/trail-records/trail-not-matched.json` | SPEC 0.1 / J09 | `tethers-0.1/protocol/cases/false-condition/expected-response.json` (committed at baseline) | Literal false-condition trail: 4 entries, `status: "not_matched"`, `plan: null` |

Both Trail fixtures are exact copies of committed protocol fixtures at baseline `24428139807cac0adeb0b62264547e61ca809d16`. No values were normalised. All UUIDs, event IDs, evaluation IDs, and message strings are the original fixture values.

## Replay Digests

| Fixture | Path | Owner/Contract | Source | Review Purpose |
|---|---|---|---|---|
| Identity claim | `fixtures/replay-digests/replay-claim.json` | J09 / J16C | Production API: `Claim::new` + `canonical_bytes()` at baseline `24428139807cac0adeb0b62264547e61ca809d16` | Exact canonical bytes of an identity claim: `record_kind: "identity_claim"`, `execution_id` with `exec_` prefix, all digests are real SHA-256 over JCS |
| Generation 0 (intent) | `fixtures/replay-digests/replay-generation-0.json` | J09 / J16C | Production API: `Generation::intent` + `canonical_bytes()` at baseline | `state: "intent_recorded"`, `state_data: {}`, predecessor = claim_digest |
| Generation 1 (armed) | `fixtures/replay-digests/replay-generation-1.json` | J09 / J16C | Production API: `Generation::armed` + `canonical_bytes()` at baseline | `state: "invocation_armed"`, `state_data: {}`, predecessor = gen-0 record_digest |
| Generation 2 (succeeded) | `fixtures/replay-digests/replay-generation-2.json` | J09 / J16C | Production API: `Generation::terminal(Succeeded)` + `canonical_bytes()` at baseline | `state: "succeeded"`, `state_data: {"durable_outcome_digest": "..."}`, predecessor = gen-1 record_digest |

Capture procedure: A scratch Rust program (`f1-replay-gen`) used the accepted host crate as a path dependency and only its public replay APIs (`LogicalExecutionKey::derive`, `ExecutionId::parse`, `ExecutionBinding`, `Claim::new`, `Claim::canonical_bytes`, `Generation::intent`, `Generation::armed`, `durable_outcome_digest`, `Generation::terminal`, `Generation::canonical_bytes`, `Claim::from_canonical_bytes`, `Generation::from_canonical_bytes`, `validate_chain`). Fixed deterministic inputs: `exec_00000000-0000-4000-8000-000000000000`, anchor/eval/action IDs `anchor-event-1`/`eval-1`/`action-1`, binding values matching the test module conventions, and durable outcome `{"status":"completed","output":"ok"}`. The scratch generator is not committed.

Validation (all performed by the scratch program):
- Every emitted record parsed through its production parser (`Claim::from_canonical_bytes`, `Generation::from_canonical_bytes`) — PASS
- Complete chain validation (`validate_chain`) — PASS
- Terminal state is `Succeeded` — PASS
- Re-serialising each parsed record via `canonical_bytes()` returns byte-for-byte identical output — PASS

Bytes are exact canonical output from production code. No normalisation was applied. All digests are the real SHA-256 values computed by `canonical_digest`/`durable_outcome_digest` over JCS-canonicalized JSON.

## Installation Outcomes (J24L Envelopes)

| Fixture | Path | Owner/Contract | Source Command | Exit Code | Capture Method | Review Purpose |
|---|---|---|---|---|---|---|
| Install success | `fixtures/installation-outcomes/j24l-install-success.json` | J24L2 | `tethers-reference-host.exe plug install --host-data-root <tmp> --request <tmp>\install-request.json` | 0 | Binary capture at baseline (warm build) | Fresh install: 4 steps with `before_action`/`after_action`/`executed_action`/`outcome` fields, concrete UUIDs and SHA-256 digest |
| Already complete | `fixtures/installation-outcomes/j24l-install-already-complete.json` | J24L2 | `tethers-reference-host.exe plug install --host-data-root <tmp> --request <tmp>\install-request.json` (second invocation) | 0 | Binary capture at baseline (warm build) | Idempotent re-install: 1 step, `outcome: "already_complete"`, no `executed_action` field |
| Refusal: candidate missing | `fixtures/installation-outcomes/j24l-install-refusal.json` | J24L2 | `tethers-reference-host.exe plug install --host-data-root <tmp> --request <tmp>\install-refusal.json` (non-existent candidate) | 3 | Binary capture at baseline (warm build) | Real refusal path: `status: "invalid_data"`, `exit_code: 3`, error code `installation_plan_candidate_missing` |

Capture procedure: A `.tetherplug` package was built from `pdf_tools_provider.exe` at baseline using `build_reference_package`. The package was staged via `plug stage`, then `plug install` was executed twice (fresh + reinstall). The refusal was captured by submitting a request with `candidate_id: "00000000-0000-0000-0000-000000000000"` which triggers `installation_plan_candidate_missing`.

Normalisation: Exact emitted JSON bytes preserved. UUIDs (`candidate_id`, `installed_id`) and digest (`installed_record_digest`) are volatile per-execution — they are the concrete values from one real execution instance, not invented placeholders. No other values were normalised.

## Installation/Recovery State Files

| Fixture | Path | Owner/Contract | Source | Review Purpose |
|---|---|---|---|---|
| M3 golden schema v1 | `fixtures/recovery-states/m3-installation-state.json` | M3 / J24D-J24L | `tethers-0.1/host-rust/fixtures/m3/m3-schema-golden-v1.json` (committed at baseline) | Complete M3 trust lifecycle: publisher key, developer approval, launch profile, conformance evidence, installation approval, installed plug record. All digests are concrete. |
| M2 candidate record v1 | `fixtures/recovery-states/m2-candidate-record.json` | M2 / J24E | `tethers-0.1/host-rust/fixtures/m2/candidate-record-v1.json` (committed at baseline) | Quarantined installation candidate record with real SHA-256 digests, payload listing, and capability manifests. |

Both are exact copies of committed test fixtures at baseline `24428139807cac0adeb0b62264547e61ca809d16`. No values were normalised.

## Fixture Independence Guarantee

All fixtures were captured or copied from:

1. **Binary capture**: `tethers-reference-host.exe` built from baseline `24428139807cac0adeb0b62264547e61ca809d16` (warm build). Commands: `--help`, `--version`, `check`, `run --help`, `plug list --host-data-root <tmp>`. Exit codes recorded from `$LASTEXITCODE`.

2. **Committed protocol fixtures**: `tethers-0.1/protocol/expected-response.json` and `tethers-0.1/protocol/cases/false-condition/expected-response.json` at baseline `24428139807cac0adeb0b62264547e61ca809d16`. These are independent test fixtures, not generated by the implementation under test.

3. **Committed Rust test fixtures**: `tethers-0.1/host-rust/fixtures/m3/m3-schema-golden-v1.json` and `tethers-0.1/host-rust/fixtures/m2/candidate-record-v1.json` at baseline `24428139807cac0adeb0b62264547e61ca809d16`.

4. **Binary capture (J24L)**: `tethers-reference-host.exe` built from baseline `24428139807cac0adeb0b62264547e61ca809d16` (warm build). Fresh install, idempotent reinstall, and missing-candidate refusal captured via `plug install` against staged `pdf-tools.tetherplug` package.

5. **Production API (replay)**: Scratch Rust program (`f1-replay-gen`, not committed) using the accepted host crate as a path dependency. Called `Claim::new`, `Generation::intent`, `Generation::armed`, `Generation::terminal`, and `canonical_bytes()` with fixed deterministic inputs. All records passed production parse, chain validation, terminal=Succeeded check, and byte-for-byte re-serialisation before output.

No `update-fixtures` script, golden-file refresh command, or auto-regeneration mechanism exists or was added.
