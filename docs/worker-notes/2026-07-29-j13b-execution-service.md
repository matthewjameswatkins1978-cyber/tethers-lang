# J13B Worker Note

Task: `J13B Packet 1 — typed host execution service and retained execution sessions`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Codex`

Status: `COMPLETE`

Base commit: `982039fd3673bb2a65fe8ed63180c3082af658b8`

Implementation checkpoint: `eb9e5e56ee6306e9dadc3f5bd3c4385380bc8559`

## Independent Red review correction

Independent review rejected the original completion claim because the service
had a second handwritten execution boundary, malformed retained-engine framing,
an incorrect request envelope, hard-coded untrusted scope, incorrect replay
identity and recovered-state mapping, fixed provider wait time, string-based
provider-error classification, ignored terminal replay publication failure,
discarded live discovery evidence, and documentation-only tests.

## Requested outcome

Extract host execution machinery from main.rs into a typed Rust application
service. Extend retained OCaml engine and MCP provider sessions. No public run
command. No evaluation-ID derivation rule.

## Changes made

### New files
- `tethers-0.1/host-rust/src/executor.rs` — CapabilityExecutor trait
- `tethers-0.1/host-rust/src/host_execution.rs` — host execution service with
  explicit prepared input, typed terminal/replay results, retained provider
  sessions, trusted prepared-runtime scope assessment, exact request assembly,
  and focused behavioural tests

### Modified files
- `tethers-0.1/host-rust/src/main.rs` — exposes one typed shared execution seam
  around the accepted J05-J11 ordering; both existing and service paths call it
- `tethers-0.1/host-rust/src/engine_stdio.rs` — retained evaluate calls use
  `arguments.request`; real-engine regression covers two evaluations
- `tethers-0.1/host-rust/src/stdio_provider.rs` — deadline-aware tools/call and
  typed JSON-RPC provider errors
- `docs/CURRENT_CLINE_TASK.md` — updated for J13B
- `docs/DECISIONS.md` — added J13B architecture decision

## Decisions and assumptions

1. `execute_shared_boundary` is the sole typed seam over the accepted J05-J11
   implementation; `HostExecutionService` does not maintain a second copy.
2. The engine request contains one selected Tether with exact source, unchanged
   event/Facts, and one direct accepted planner-capability array with the
   existing bridge projection.
3. Trusted scope comes only from `PreparedRuntime::assess_action_scope`.
4. Replay uses Anchor event ID, planner evaluation ID, Action ID, and the
   accepted canonical argument digest.
5. Retained provider calls receive the host's exact remaining monotonic
   duration; JSON-RPC error is known failure and transport/framing loss is
   uncertainty.
6. Live tools/list evidence is compared against every prepared capability
   before provider availability is admitted.
7. No public run command or evaluation-ID derivation rule was added.

## Final planner-response correction

The execution service now classifies planner responses before entering the
shared dispatch boundary:

- `matched` requires exact protocol, evaluation, event, Tether ID, and Tether
  version correlation with the submitted input;
- `not_matched` returns `NoActions`;
- correlated and minimal `error` responses return the distinct
  `PlannerError` result;
- missing or unknown status and every missing or mismatched correlation field
  return `InvalidData`.

Only a validated `matched` response can reach replay admission or a provider.
`ApprovalRequired` now carries only the existing evaluation ID, Action ID, and
a redacted policy reason; it does not fabricate an approval identifier.

## Evidence

- cargo fmt --check: PASS
- cargo check: PASS
- cargo check --tests: PASS
- cargo test j12_ -- --nocapture: 99 passed, 0 failed
- cargo test j13a_ -- --nocapture: 74 passed, 0 failed
- cargo test j13b_ -- --nocapture: 34 host passed plus 1 library passed, 0 failed
- cargo test: 715 passed, 0 failed (32 library + 654 binary + 29 CLI)
- cargo clippy --all-targets --all-features: PASS (no errors)
- cargo build: PASS
- cargo build --release: PASS
- git diff --check: PASS
- check-tethers-task-packet.ps1: PASS at IN_PROGRESS checkpoint
- check-fixtures.ps1: PASS (46 JSON, 30 JSONL)
- test-engine.ps1: PASS (28 fixture/determinism/line-ending checks)
- test-mcp-transcripts.ps1: PASS (15 cases)
- test-j13a-check.ps1: PASS (25/25)
- read-only switch:
  `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`
- OCaml 5.5.0; Dune 3.24.0; explicit-switch `dune build`: PASS
- The first direct `test-engine.ps1` attempt correctly found no Goose-local
  switch. It passed on rerun with process-local `OPAMSWITCH` pointing to the
  authorised read-only original-worktree switch; no switch was selected or
  modified.

## Focused correction regressions

- retained engine `arguments.request` and two evaluations
- exact selected-Tether/source/direct-capability request shape
- structured scope requires trusted WithinScope evidence
- Anchor event ID in replay identity and canonical argument binding
- all recovered replay states: exact typed result and zero provider calls
- expired-before-call Unattempted and remaining duration propagation
- provider-declared failure versus malformed/EOF/timeout uncertainty
- terminal replay publication failure cannot report Completed or emit an Anchor
- live tools/list mismatch is not admitted as available
- retained provider calls use IDs 3 then 4
- CLI rejects `run`; evaluation ID remains caller-supplied
- matched response validates all five correlation fields before dispatch
- not-matched response returns NoActions without dispatch
- correlated and minimal planner errors return PlannerError without dispatch
- every missing or mismatched correlation field returns InvalidData
- missing and unknown planner status return InvalidData
- Ask returns no invented approval ID
- rejected, error, invalid, and Ask paths make zero replay-admission and
  provider calls

## Discoveries

The Goose worktree intentionally has no local opam switch. The documented
engine script can use the authorised original-worktree toolchain without
selecting it by setting `OPAMSWITCH` only for that child process.

## Remaining risks

Packet 1 intentionally does not expose a public execution command or define an
evaluation-ID generation rule. Packet 2 was not begun.

## Smallest next action

Independent Red review of this corrected Packet 1 commit.

## References

- Branch: goose/j13b-execution-service
- Reviewed candidate: eb9e5e56ee6306e9dadc3f5bd3c4385380bc8559
