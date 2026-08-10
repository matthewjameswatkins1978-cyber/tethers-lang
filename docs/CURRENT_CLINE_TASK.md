# Current Implementation Task

Control contract: `1`
Task: `TETHERS-0.3-P6 — The Evil Bunny Test`
Owner: `OpenCode`
Status: `IN_PROGRESS`
Task colour: `Amber`
Route: `OpenCode implementation + evidence → Lucy independent GitHub review`
Worker note: `docs/worker-notes/2026-08-10-0.3-p6-evil-bunny-adversarial-provider-proof.md`
Base branch: `feature/0.3-p5-fresh-agent-authoring-proof`
Base commit: `ffbe25e1c36123301182383c97265a6174b5dd98`
Implementation branch: `feature/0.3-p6-evil-bunny-adversarial-provider-proof`

P5 is FINAL ACCEPTED. Do **not** start 0.4 concurrency or any later milestone.

## Objective

Prove that hostile providers cannot compromise host protocol correctness. Build a
deliberately badly behaved provider test fixture (human nickname: **The Evil
Bunny Test** — an adversarial *protocol* test, not a malware test, and never a
claim of OS isolation) and prove that Tethers detects and safely refuses
protocol violations rather than accepting false conformance. Evidence-first:
observe each adversarial case against the current host, preserve every correct
refusal, and make the smallest generic correction only where the host is
demonstrably fooled.

## Relevant background and existing behaviour

- P5 FINAL ACCEPTED at `ffbe25e1c36123301182383c97265a6174b5dd98`; the public
  author manual is `docs/PLUG_AUTHORING.md`, and the P5 experiment log is
  `docs/p5-fresh-agent-proof.md`.
- `plug conform` deliberately executes provider code under process supervision
  and reports `isolated: false`; it is **not** a security sandbox.
- Public `plug pack`, `plug inspect`, and `plug conform` are accepted P2/P3
  surfaces and must be used exactly as documented, including the mandatory
  approval gate: default conform refuses execution with status
  `approval_required`, exit code `5`, and error code
  `conformance_execution_approval_required`.
- The host conformance suite (`tethers-0.1/host-rust/src/conformance.rs`) drives
  an MCP stdio provider through `initialize`, `notifications/initialized`,
  `tools/list`, and (for `fixture*` operations only) `tools/call`. During
  approved conformance the host launches the provider with `TETHERS_CONFORMANCE=1`,
  a scratch `TEMP`/`TMP`, `SystemRoot`, and `WINDIR`.
- Discovery currently compares only `inputSchema` against the reviewed manifest;
  the retained-session dispatch path (`stdio_provider.rs`) additionally verifies
  `outputSchema` and JSON-RPC response id correlation. P5 discovered the manual
  gap (advertise both schemas) and fixed the manual; conformance itself still
  accepts an advertised-only-`inputSchema` or mismatched `outputSchema`.
- The conformance `request()` helper reads a line and parses it but does not
  validate the JSON-RPC response envelope or correlate the response `id` with
  the request.
- The `bounded_shutdown_process_cleanup` case is currently recorded as passed
  unconditionally; the provider-cleanup accounting from `SupervisedChild::shutdown`
  is discarded.
- The M3 fixture provider (`tethers-0.1/host-rust/src/bin/m3_fixture_provider.rs`)
  is a host-crate test-only binary used by the M3/P2 conformance tests; it
  advertises `tools/list` entries with only `inputSchema`.

## Required behaviour

1. Create `feature/0.3-p6-evil-bunny-adversarial-provider-proof` based on the
   exact P5 accepted HEAD `ffbe25e1c36123301182383c97265a6174b5dd98`.
2. Update `docs/CURRENT_CLINE_TASK.md` to the P6 packet with Status
   `IN_PROGRESS` and run the packet checker (`control-v1/IN_PROGRESS`).
3. Build a bounded adversarial provider fixture (`tethers.evil-bunny-proof` /
   `tethers-evil-bunny-provider`) under an appropriate test/reference location,
   with deterministic modes that can only lie, hang, crash, emit malformed
   protocol, or advertise false contracts at the Tethers/MCP boundary. It must
   not damage files, escape the filesystem, access credentials, attack the
   network, spawn uncontrolled processes, persist itself, act destructively, or
   claim process supervision is a sandbox.
4. EB-00 Good Bunny control: prove the harness can produce one fully conforming
   provider (approved conform → passed) so a broken harness cannot make every
   hostile case look correctly rejected.
5. Prove the required adversarial cases are each deterministically refused, with
   observable evidence identifying the violated contract: EB-01 identity liar,
   EB-02 protocol-version liar, EB-03 missing operation, EB-04 surprise
   operation, EB-05 wrong operation name, EB-06 input-schema liar, EB-07
   output-schema liar (mismatched and omitted), EB-08 malformed stdout, EB-09
   wrong response identity/envelope, EB-10 early death/crash, EB-11 silent
   Bunny/hang (bounded timeout and cleanup), EB-12 shutdown refusal (bounded
   cleanup, no orphan process, no clean success).
6. Preserve the mandatory approval gate for every packable Evil Bunny package:
   `plug conform` without approval must refuse with status `approval_required`,
   exit code `5`, error code `conformance_execution_approval_required`, and the
   provider must not execute. Only then run approved non-isolated conform.
7. Keep static package correctness separate from runtime hostility: where useful,
   `plug pack` and `plug inspect` succeed so the evidence proves the package is
   well formed while the provider lies or misbehaves when executed. Each case
   should violate one primary contract wherever practical.
8. If repository evidence confirms approved `plug conform` accepts a missing or
   mismatched `outputSchema`, or otherwise fails to refuse a hostile case, treat
   it as a real generic conformance gap and make the smallest generic production
   correction necessary, with regression evidence. Record the before/after
   honestly; do not hide an original false acceptance.
9. Generic host code must gain no Evil Bunny/provider-family knowledge.
10. Create `docs/p6-evil-bunny-proof.md` (The Evil Bunny Chronicles) recording
    every case compactly: evil behaviour, contract attacked, expected result,
    actual result, exit/status/error evidence, process cleanup result, whether a
    production correction was required, and final disposition, plus an overall
    matrix. Do not paste enormous raw transcripts.
11. Add targeted automated regressions for generic host behaviour where
    appropriate; production conformance and test conformance must use the same
    helpers/behaviour, and any `outputSchema` regression must prove the real
    generic discovery/conformance seam rejects the mismatch.
12. Update `docs/ROAD_TO_0_3.md`, `docs/CURRENT_GOAL.md`, and
    `docs/PROJECT_DASHBOARD.md` to show P5 FINAL ACCEPTED at
    `ffbe25e1c36123301182383c97265a6174b5dd98`, P6 complete awaiting Lucy
    review, P7 / 0.4 next and NOT started.
13. Close out per project control: implementation checkpoint commit, worker
    note, packet COMPLETE, checker `control-v1/COMPLETE`, docs closeout commit,
    normal push, remote == local HEAD proof, clean worktree.

## Relevant components

- `tethers-0.1/host-rust/src/conformance.rs` (host conformance suite contract)
- `tethers-0.1/host-rust/src/plug_conform.rs` (public conform CLI)
- `tethers-0.1/host-rust/src/launch_profile.rs` (approved conformance env and launch)
- `tethers-0.1/host-rust/src/child_process.rs` (`SupervisedChild` bounded cleanup)
- `tethers-0.1/host-rust/src/stdio_provider.rs` (retained-session discovery/dispatch
  contract: `outputSchema` + response-id correlation reference)
- `tethers-0.1/host-rust/src/bin/m3_fixture_provider.rs` (existing test-only fixture)
- `tethers-0.1/host-rust/tests/` (p2b, p2c, p3, m3_lifecycle patterns)
- `reference-plugs/evil-bunny-proof/` (new fixture: provider-rust, author cases, scripts)
- `justfile` (`test-pdf-reference` recipe as reference-only pattern)
- `docs/PLUG_AUTHORING.md`, `docs/p5-fresh-agent-proof.md`,
  `docs/ROAD_TO_0_3.md`, `docs/CURRENT_GOAL.md`, `docs/PROJECT_DASHBOARD.md`
- `docs/worker-notes/2026-08-10-0.3-p6-evil-bunny-adversarial-provider-proof.md` (new)

## Frozen decisions and invariants

- The Evil Bunny fixture is a safe, deterministic protocol test fixture; it is
  never a malware test and never claims operating-system isolation.
- `isolated: false` and the non-isolation limitation remain honest throughout.
- Default `plug conform` still requires explicit supervised-execution approval
  before any provider execution; do not bypass it.
- Generic host code contains no Evil Bunny/provider-family knowledge; every
  production correction is generic and backed by regression evidence.
- Bad provider output must never become trusted evidence; false provider
  identity must never override reviewed identity; undeclared operations must
  never be admitted; malformed protocol fails closed; unexpected exit is
  failure/uncertainty, never success; cleanup failure must never become clean
  conformance success; no Evil Bunny process may remain running after
  verification.
- Do not change 0.1 syntax, Plug format, manifests, MCP, Operational Scope
  Evidence, or trust/install/enable semantics.
- No P6 production correction may be added merely because a stronger check is
  imaginable; each must be driven by observed false acceptance or host-correctness
  breakage.

## Acceptance criteria

1. A bounded, safe, deterministic Evil Bunny fixture exists under
   `reference-plugs/evil-bunny-proof/` with identity `tethers.evil-bunny-proof` /
   `tethers-evil-bunny-provider`, mode-selected adversarial behaviours, and no
   destructive, persistent, credential, network, or uncontrolled-process behaviour.
2. EB-00 Good Bunny control passes approved non-isolated conform
   (`passed`, `isolated: false`, non-isolation limitation present).
3. Every required Evil Bunny execution case (EB-01 through EB-12) has observable
   public-journey evidence: pack/inspect where applicable, default conform
   refusal (exit 5, `approval_required`,
   `conformance_execution_approval_required`), approved conform, and the expected
   safe non-success disposition.
4. No required hostile case produces false conformance success; identity
   mismatch, protocol-version mismatch, missing expected operation, undeclared
   extra operation, wrong operation name, `inputSchema` mismatch, missing or
   mismatched `outputSchema`, malformed stdout, wrong response
   identity/envelope, early provider death, silent/hanging provider, and
   shutdown/cleanup failure are all refused or bounded non-success.
5. The response-envelope/correlation gap (EB-09) is either already refused with
   evidence or corrected generically with regression proof.
6. The `outputSchema` conformance gap (EB-07) is corrected generically so
   approved conform verifies both reviewed schemas consistently, with regression
   evidence proving the real discovery/conformance seam rejects the mismatch;
   any original false acceptance is recorded honestly as before/after evidence.
7. The shutdown-refusal gap (EB-12) is corrected generically so cleanup failure
   cannot produce clean conformance success, cleanup stays bounded, and no
   orphan Evil Bunny process remains.
8. Approval is still required before provider execution; `isolated:false` and
   the non-isolation limitation remain honest.
9. Generic host code contains no Evil Bunny/provider-family knowledge; all
   production corrections are generic and backed by regression evidence.
10. `docs/p6-evil-bunny-proof.md` records the experiment with per-case evidence
    and an overall matrix, and preserves any before/after false acceptance
    honestly.
11. Project docs show P5 FINAL ACCEPTED at
    `ffbe25e1c36123301182383c97265a6174b5dd98`, P6 complete awaiting Lucy
    review, and P7 / 0.4 next and NOT started.
12. Packet checker reports `control-v1/IN_PROGRESS` at start and
    `control-v1/COMPLETE` on closeout.
13. Branch pushed normally; remote HEAD == local HEAD; worktree clean; 0.4 has
    not started.

## Required verification

1. Packet checker at start (`control-v1/IN_PROGRESS`) and on closeout
   (`control-v1/COMPLETE`).
2. Evil Bunny fixture provider formatter/check/build and any provider tests.
3. Real `plug pack` proof for every packable Evil Bunny package.
4. Real `plug inspect` proof for every packable Evil Bunny package.
5. Real default `plug conform` refusal proof (exit 5,
   `approval_required`, `conformance_execution_approval_required`; provider not
   executed).
6. Real approved `plug conform` proof for every case with the expected safe
   non-success disposition and bounded cleanup; no provider process remains.
7. For the fixed generic gaps, targeted host tests/regressions that prove the
   real generic conformance seam (same production helpers) rejects the violation.
8. If production host Rust changes, the repository's full warnings-denied /
   agent verification gate for production Rust changes (e.g. `just verify` plus
   the normal host test suite), with any unrelated environmental failure
   preserved honestly.
9. `git diff --check` and complete diff/status inspection.

## Formatting and checkpoint sequence

Rust source introduced under `reference-plugs/evil-bunny-proof/provider-rust/`
is formatted with `cargo fmt --manifest-path <provider Cargo.toml> -- --check`
(and, if chosen, `cargo fmt` on that crate only). Production host Rust changes
must satisfy the normal host gate (`cargo fmt --all -- --check` first). The
implementation checkpoint precedes all worker note, packet, and dashboard
closeout edits.

## Completion and publication

Commit the implementation/proof checkpoint, write the worker note at the named
path, set this packet to `COMPLETE`, require checker `control-v1/COMPLETE`,
commit docs-only closeout, then push the named branch normally and prove
`origin/feature/0.3-p6-evil-bunny-adversarial-provider-proof == HEAD` and a
clean worktree. Do not start P7 or 0.4.

## Forbidden changes

- No P7, no 0.4 concurrency, no HQ, no Event Ingress.
- No redesign of Plug format, manifests, MCP, Operational Scope Evidence, or
  trust/install/enable semantics.
- No CLI, host OCaml production, dependency, or conformance-semantic changes
  beyond the smallest generic corrections required by observed false acceptance.
- No Evil Bunny/provider-family knowledge in generic host code.
- No claims that process supervision is a security sandbox.
- No merge, amend, tag, force-push, PR, or direct `main` update.

## Stop conditions

- A real contradiction between the frozen architecture and repository evidence
  that cannot be resolved from the packet.
- A consequential architecture/product/security/trust decision requiring
  external authority.
- Required completion would weaken the explicit approval boundary or claim
  isolation that does not exist.
- Two materially similar implementation attempts fail on the same unresolved
  underlying problem.

## Expected pre-existing changes

None. Base commit is the accepted P5 HEAD; the P6 branch starts clean at it.
