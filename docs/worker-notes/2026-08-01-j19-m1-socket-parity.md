# Worker Note

Task: `J19-M1 - Autonomous Socket Parity Programme`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Codex Terra High`

Status: `COMPLETE`

Base commit: `41a8a67737c7987a0fa05e219aeb5f202a96be26`

Implementation checkpoint: `5bc483c4a03fe4e062614507f8b50b6ac98656b2`

Branch: `codex/j19-first-plug-kit`

Accepted implementation baseline: `cfdb372ab18c7935c6046faf5cf82da2fe742440`

## Requested outcome

Complete Milestone 1 P1, P2 and P3: extract the reusable host application
seam, place retained MCP stdio sessions behind semantic Socket operations, and
implement complete discovery catalogues with fail-closed drift handling while
preserving released 0.2 behavior and every frozen host boundary.

## Changes made

- Moved shared application ownership from `src/main.rs` to `src/application.rs`
  and the library module graph, leaving a thin binary dispatcher.
- Added `src/socket.rs` and routed retained provider establishment, discovery,
  invocation, observation, probe and close through it.
- Added paginated MCP discovery, notification observation and a non-blocking
  supervised-child notification read seam.
- Added focused pagination, cursor, duplicate, stale, drift and revalidation
  tests plus fixture modes.
- Updated the J14B focused-proof matcher for the new library test owner.
- Completed `docs/CURRENT_CLINE_TASK.md` and this worker note.

## Commit map and rollback points

- P1 `a7ea653f87a554405dbfbcceda2493c893cf181d` — `refactor: extract host
  application library seam`. Roll back here to the control commit if the
  library extraction itself must be removed.
- P2 `34be88b` — `refactor: add semantic socket boundary`. Reverting this commit
  removes Socket while retaining P1.
- P3 `3c525ae` — `feat: complete socket discovery catalogue`. Reverting this
  commit removes pagination and invalidation while retaining P1/P2.
- Compatibility correction `5bc483c` — `test: follow extracted j14b proof
  paths`. It changes only the J14B focused-test owner matcher.

## Chosen source layout

- `src/lib.rs` owns the shared host module graph and deliberately exports the
  application and Socket modules.
- `src/application.rs` owns the former binary-root application dispatcher,
  compatibility/debug routes, the shared execution boundary, and their
  preserved tests.
- `src/main.rs` is a three-line process entry that delegates to
  `tethers_reference_host::application::run()`; no host implementation is
  compiled twice.
- `src/socket.rs` owns `Socket`, `SocketEstablishment`, `SocketCatalogue` and
  `RetainedProviderSession`.
- `src/stdio_provider.rs` remains the MCP `2025-11-25` wire binding;
  `SupervisedChild` remains the sole byte-transport/process supervisor.

This layout makes the application service and retained provider lifecycle
library code without creating another dispatch, replay, policy, Trail or
outcome boundary.

## Decisions and assumptions

Socket owns only semantic session lifecycle and protocol observations. The
host continues to own trusted-schema comparison and all authority-bearing
decisions. A catalogue notification invalidates Socket state but is not an
Anchor. Rediscovery is capped at two attempts; unchanged exact bindings are
retained, while missing or changed bindings remain unavailable. Additional
live operations remain untrusted observations because planner availability is
still projected only from prepared trusted capabilities.

## P1 evidence

- Baseline library run before extraction: `44` passed, `0` failed.
- Extracted library run: `768` passed, `0` failed; full crate `768 + 29 = 797`
  passed.
- `cargo fmt --check`, full Rust tests and locked debug build passed before the
  P1 checkpoint.
- `HostExecutionService` compiles as library code. The released CLI parser,
  envelopes, exit codes, legacy routes, replay, Result Anchors and Trail order
  are the same production implementations moved under library ownership.
- The two J14B proofs moved unchanged from `tests::j14b_*` to
  `application::tests::j14b_*`; the final harness correction retained both
  exact names and the exact count of two.

## P2 Socket decisions and evidence

Socket v1 exposes `establish`, `discover`, `invoke`, `observe_result`,
`observe_catalogue_change`, `probe` and `close`. One mutable retained session
owns one provider process and one checked, monotonically increasing JSON-RPC
request-ID sequence. Mutable access preserves serial invocation and one active
request; there is no batching, parallel call, hidden restart queue, automatic
retry or changed launch behavior.

Socket returns protocol observations only. Trusted binding comparison, policy,
approval, replay, canonical outcome classification, Result Anchors and Trail
remain in the host. MCP method names remain standard wire mappings; diagnostic
stderr remains separate from protocol stdout; close still delegates to the
bounded supervised-child cleanup.

P2 verification passed `768` library tests, the complete Rust crate, and all
15 MCP transcript cases before commit `34be88b`.

## P3 discovery and invalidation evidence

- Complete `tools/list` pagination consumes every page and passes the cursor
  string through unchanged; the fixture rejects any alteration of
  `opaque::+/=`.
- Repeated/looping cursors and duplicate operation names across pages fail
  closed.
- `SocketCatalogue` retains complete operation JSON, including untrusted
  descriptions, annotations and additions. Those observations do not enter the
  trusted manifest store or planner availability.
- Existing exact input/output schema comparison validates every prepared
  capability. Missing or changed bindings remain unavailable; unapproved
  additions remain observations only.
- `notifications/tools/list_changed` is consumed only by the protocol binding,
  marks the catalogue stale, and has no event-admission or Anchor path.
- Invocation polls already-observed notifications and refuses stale state
  before request-ID allocation or transport. A notification observed during a
  response invalidates the next invocation.
- Host-owned rediscovery is bounded to two attempts. An unchanged binding is
  retained after exact revalidation; schema drift clears the snapshot and
  returns unavailable before the shared execution/provider-call boundary.

Focused production/seam evidence: `8` Socket tests and `2` host rediscovery
tests passed. The stale-invocation proof observes unchanged request-ID state at
the transport seam; notification-to-stale mapping uses the real PowerShell MCP
fixture. The absence of a notification Anchor is a structural boundary proof:
the Socket/stdio modules have no event admission callback and only mutate
catalogue state.

## Regression and resolution

The first consolidated run passed J13A, J13B, J13C, J14A and J14C but J14B
reported `focused Rust j14b_ test count Expected '2', got '0'`. P1 had moved
the two tests into the library, while the harness still matched the old binary
crate path. Runtime rows had not failed. The harness matcher was narrowed to
the new `application::tests::j14b_` owner while preserving the two exact names
and count. Direct J14B then passed `11/11` with `243` assertions, and the full
six-suite matrix passed on the next run.

## Discoveries

The application extraction changed Rust unit-test owner paths even though it
preserved test bodies and assertions. The frozen J14B harness intentionally
counts two exact internal proofs, so its path matcher needed to follow the
library owner. MCP server notifications can arrive between responses; draining
only already-buffered protocol lines before the next serial request preserves
the existing supervisor and avoids inventing a background or parallel Socket
runtime.

## Evidence

- Toolchain gate: PASS — Rust `1.89.0`, OCaml `5.5.0`, Dune `3.24.0`, Yojson
  `2.2.2`, locked project metadata.
- `cargo fmt --check`: PASS.
- `cargo check --all-targets --all-features --locked`: PASS. It emitted the
  tracked warning set: 13 library warnings, 5 integration-target warnings and
  7 library-test warnings (the target totals include duplicates).
- `cargo test --all-targets --all-features --locked`: PASS — `777` library and
  `29` integration tests, `806` total, zero failed or ignored.
- Locked debug build: PASS. Locked release build: PASS.
- OCaml `dune build` and `dune runtest` through the exact local switch: PASS.
- Fixture validation: PASS — `46` JSON and `30` JSONL files.
- Engine fixture suite: PASS — 28 reported cases including deterministic and
  line-ending equivalence proofs.
- MCP transcript suite: PASS — 15 cases.
- All tracked `test-host-*.ps1` scripts in lexical order: PASS.
- Demo: PASS. Runner contract: `6/6`, 49 assertions.
- Consolidated verification: J13A `25/25`; J13B `10/10`; J13C `19/19`;
  J14A `5/5`, 95 assertions; J14B `11/11`, 243 assertions; J14C `9/9`, 196
  assertions; total `6` suites passed, `0` failed, `RESULT: PASS`.
- J14C preserved one real move, the same replay execution ID and zero
  additional replay moves; cleanup removed its temporary root.
- No executable beneath the J16 Clean checkout remained after verification.
- `git diff --check`: PASS.
- `Cargo.lock`: unchanged, SHA-256
  `894F2CE6692837FA4C449C0FC593A37ED5597577EA5B4093DA0912E6EE2B14E3`.

Clippy with `-D warnings` was not run: it is not in the controlling M1 packet's
required gate, and the packet accepts the tracked non-fatal compiler warning
baseline reported above. No warning was suppressed or deleted.

## Frozen-boundary reconciliation

OCaml Core, Tether 0.1 syntax/semantics, policy, approval, provider launch,
serial dispatch, outcomes, durable replay, Result Anchor creation, event
admission and Trail ordering are unchanged. No dependency, manifest, lock,
public CLI, retry, network provider, package, trust, credential, File Tools or
Milestone 2 behavior was introduced. Released `v0.2.0` history was untouched.

## Remaining risks

The tracked Rust warning set remains non-fatal and visible. No functional or
acceptance risk remains inside M1; Lucy's independent review is the next gate.

## Smallest next action

Lucy reviews the pushed M1 commit stack and this evidence. Milestone 2 must not
begin without a new authoritative packet.

## References

- Control packet and completion ledger: `docs/CURRENT_CLINE_TASK.md`.
- Rust implementation: `tethers-0.1/host-rust/src/application.rs`, `main.rs`,
  `lib.rs`, `socket.rs`, `stdio_provider.rs`, `host_execution.rs`, and
  `child_process.rs`.
- Focused fixtures: `tethers-0.1/scripts/tethers-stdio-fixture.ps1` and
  `test-j14b-negative-matrix.ps1`.
- P1/P2/P3 commits: `a7ea653f87a554405dbfbcceda2493c893cf181d`,
  `34be88b0f9b8d5ac8d8f3e304ca9684473b5e2ee`, and
  `3c525ae93fca0750104310c3799480839701bbb7`.
