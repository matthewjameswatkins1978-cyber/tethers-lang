# The Evil Bunny Chronicles

## P6 Adversarial Provider Proof

Status: experiment complete, awaiting Lucy independent GitHub review.

> Can a hostile provider fool Tethers into believing it conforms? This is an
> adversarial **protocol** test, not a malware test, and it never claims
> operating-system isolation. `plug conform` deliberately executes provider
> code under process supervision and reports `isolated: false`.

## Experiment

- **Fixture:** `reference-plugs/evil-bunny-proof/` — package
  `tethers.evil-bunny-proof` 1.0.0, provider `tethers-evil-bunny-provider`
  1.0.0, capability `evil.probe@1` / operation `evil_probe` (non-fixture, so
  conformance exercises the pure discovery/shutdown boundary).
- **Mechanism:** one provider binary, deterministic `--mode` launch argument,
  fourteen cases (EB-00 through EB-12 with EB-07 split into mismatch/omitted).
- **Journey per case:** real public `plug pack` → `plug inspect` → `plug
  conform` (default refusal) → `plug conform`
  (`--allow-non-isolated-supervised-execution`). Raw evidence for every case is
  committed under `reference-plugs/evil-bunny-proof/evidence/<case>/`.
- **Baseline:** current host at the P5 accepted HEAD
  `ffbe25e1c36123301182383c97265a6174b5dd98` (before the P6 generic
  corrections), then the corrected host.

The Good Bunny control (EB-00) semantic package digest is
`sha256:bebe73f221ccbd71a9992e3aaec6fb962f8b626162bb637f50ce6535dbf4b618`;
each case carries its own deterministic digest because the launch argument
(mode) differs.

## Per-case evidence

Legend for `session code`: the `safe_diagnostic_code` of the
`conformance_session` case; EB-12 instead fails the
`bounded_shutdown_process_cleanup` case.

| Case | Evil behaviour | Contract attacked | Expected | Actual | Exit / status / error evidence | Cleanup | Fix? | Disposition |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| EB-00 Good Bunny | fully conforming provider | none (control) | approved conform → passed | passed | exit 0, status `ok`, disposition `passed`, `isolated:false`, "not isolated" limitation present | graceful, no process | no | PASS |
| EB-01 Identity liar | `serverInfo.name` = `tethers-impostor-provider` | provider identity vs reviewed binding | refusal | failed | exit 6, `plug_conformance_failed`, session `provider_identity` | graceful, no process | no | REFUSED |
| EB-02 Protocol-version liar | `protocolVersion` = `2024-11-05` | MCP protocol pin | refusal | failed | exit 6, session `protocol_pin` | graceful, no process | no | REFUSED |
| EB-03 Missing operation | `tools/list` returns no tools | declared operation absent | refusal | failed | exit 6, session `catalogue_drift` | graceful, no process | no | REFUSED |
| EB-04 Surprise operation | advertises `evil_probe` + `surprise_tool` | undeclared operation admission | refusal | failed | exit 6, session `catalogue_drift` | graceful, no process | no | REFUSED |
| EB-05 Wrong operation name | advertises `evil_probe_v2` | operation name vs binding | refusal | failed | exit 6, session `catalogue_drift` | graceful, no process | no | REFUSED |
| EB-06 Input-schema liar | `inputSchema.required` = `["different"]` | advertised input schema vs manifest | refusal | failed | exit 6, session `catalogue_drift` | graceful, no process | no | REFUSED |
| EB-07a Output-schema liar (mismatch) | `outputSchema.required` = `["different"]` | advertised output schema vs manifest | refusal | failed | exit 6, session `catalogue_drift` | graceful, no process | **yes** | REFUSED |
| EB-07b Output-schema liar (omitted) | `outputSchema` absent from `tools/list` | advertised output schema vs manifest | refusal | failed | exit 6, session `catalogue_drift` | graceful, no process | **yes** | REFUSED |
| EB-08 Malformed stdout | writes `not json` as the initialize response | protocol framing / parse | refusal | failed | exit 6, session `conformance_protocol` | graceful, no process | no | REFUSED |
| EB-09 Wrong response identity | initialize response bound to id `1001`, not `1` | JSON-RPC request/response correlation | refusal | failed | exit 6, session `protocol_correlation` | graceful, no process | **yes** | REFUSED |
| EB-10 Early death / crash | exits with code 7 before any exchange | unexpected provider exit | refusal | failed | exit 6, session `conformance_protocol` | reaped (already exited), no process | no | REFUSED |
| EB-11 Silent Bunny / hang | starts but never responds | bounded timeout + cleanup | refusal | failed | exit 6, session `conformance_protocol` (read timeout) | terminated by Job Object, no process | no | REFUSED |
| EB-12 Shutdown refusal | correct protocol, then ignores stdin close and sleeps forever | graceful-shutdown cooperation | refusal | failed | exit 6, `bounded_shutdown_process_cleanup` case `failed`, code `provider_did_not_exit_gracefully` | terminated by Job Object, bounded, no process | **yes** | REFUSED |

Every packable case proved the mandatory approval gate first: `plug conform`
without approval returned exit `5`, status `approval_required`, error code
`conformance_execution_approval_required`, and the provider was not executed.
`plug pack` and `plug inspect` succeeded for every case (exit 0), proving each
package is structurally well formed while its provider lies or misbehaves when
executed.

## Before / after: the three genuine generic conformance gaps

P6 ran every case against the original host first. Four cases were **falsely
accepted** as `passed` (exit 0) by the original public conformance path:

```text
BEFORE EB-07a output-schema mismatch  -> approved conform exit 0, disposition passed   FALSE ACCEPTANCE
BEFORE EB-07b output-schema omitted   -> approved conform exit 0, disposition passed   FALSE ACCEPTANCE (the P5 manual gap, reproduced)
BEFORE EB-09 wrong response id        -> approved conform exit 0, disposition passed   FALSE ACCEPTANCE
BEFORE EB-12 shutdown refusal         -> approved conform exit 0, disposition passed   FALSE ACCEPTANCE
```

These are genuine generic conformance gaps, not Evil-Bunny-specific bugs. The
smallest generic production corrections were made in
`tethers-0.1/host-rust/src/conformance.rs`:

1. **EB-07 (outputSchema):** discovery now compares BOTH `inputSchema` and
   `outputSchema` against the reviewed manifest (it previously compared only
   `inputSchema`, matching the P5 manual gap). After: both mismatch and omitted
   output schemas are refused with `catalogue_drift`.
2. **EB-09 (response correlation/envelope):** the `request()` helper now
   validates the JSON-RPC response envelope (`jsonrpc 2.0`, object, matching
   response `id`, no top-level error) before trusting `result`. After: a wrong
   response id is refused with `protocol_correlation`; malformed stdout now
   reports `conformance_protocol` instead of the store-internal `record_invalid`.
3. **EB-12 (shutdown cooperation):** `bounded_shutdown_process_cleanup` now uses
   the actual `SupervisedChild` cleanup accounting and fails with
   `provider_did_not_exit_gracefully` when the provider does not exit
   gracefully after stdin closes, instead of always passing. Cleanup stays
   bounded and no process remains.

The M3 fixture provider (`src/bin/m3_fixture_provider.rs`) was corrected to
advertise `outputSchema` in `tools/list` (it advertised only `inputSchema`, the
same staleness the manual now forbids), so the existing conformance tests keep
passing under the tightened discovery.

## Overall matrix

```text
EB-00 Good Bunny              PASS
EB-01 Identity liar           REFUSED
EB-02 Protocol liar           REFUSED
EB-03 Missing operation       REFUSED
EB-04 Surprise operation      REFUSED
EB-05 Wrong operation name    REFUSED
EB-06 Input-schema liar       REFUSED
EB-07a Output-schema mismatch REFUSED
EB-07b Output-schema omitted  REFUSED
EB-08 Malformed stdout        REFUSED
EB-09 Wrong response identity REFUSED
EB-10 Early death             REFUSED
EB-11 Silent Bunny / hang     REFUSED
EB-12 Shutdown refusal        REFUSED
```

## Automated regressions

`tethers-0.1/host-rust/tests/p6_evil_bunny.rs` (three `#[ignore]` tests, run
via `just test-evil-bunny-proof`) drives the real production seam — the public
CLI and `run_host_conformance` — against the committed fixture:

- `p6_evil_bunny_good_control_public_journey` — Good Bunny full public journey
  passes; `isolated:false` and the non-isolation limitation are asserted.
- `p6_evil_bunny_hostile_cases_refused_public_journey` — every EB-01..EB-12
  approved conform fails with the exact violated-contract code; EB-12 fails the
  shutdown case; default conform always refuses (exit 5, approval_required).
- `p6_evil_bunny_fixed_gaps_rejected_at_real_conformance_seam` — the direct
  `run_host_conformance` seam (the exact function behind `plug conform`)
  rejects EB-07a, EB-07b, EB-09, and EB-12.

## Verification run against the corrected host

- Evil Bunny provider: `cargo build` + `cargo test --locked` (5/5 pass), `cargo
  fmt --check` clean.
- Public journey evidence: `reference-plugs/evil-bunny-proof/evidence/` for all
  14 cases (pack, inspect, denied conform, approved conform; exit codes and
  stderr captured).
- p6 integration tests: `3 passed; 0 failed` (via the exact
  `TETHERS_EVIL_BUNNY_PROVIDER_EXE` environment the recipe sets).
- Host full suite: `cargo test --all-targets --all-features --locked` — all
  test binaries pass, `0 failed`.
- PDF reference crucible (`just test-pdf-reference`): 2/2 pass, confirming the
  tightened discovery and shutdown accounting do not break the real PDF Plug.
- `cargo clippy --all-targets --all-features --locked` — no warnings in P6
  files; `cargo fmt --all -- --check` clean; `cargo build --release --locked`
  passes; `git diff --check` clean.
- No `tethers_evil_bunny_provider.exe` process remained after any run.

## Conclusion

P6 proves hostile providers cannot compromise host protocol correctness. Of the
thirteen adversarial cases, nine were already refused by the current host;
four (output-schema mismatch, output-schema omission, wrong response identity,
shutdown refusal) were falsely accepted and are now refused by the smallest
generic conformance corrections, each backed by regression evidence at the real
discovery/conformance seam. The mandatory approval gate, `isolated:false`, and
the non-isolation limitation remain honest throughout, and no Evil Bunny
process survives verification.
