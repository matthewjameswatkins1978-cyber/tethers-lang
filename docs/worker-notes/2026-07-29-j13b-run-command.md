# J13B Packet 2 Worker Note

Task: `J13B Packet 2 — strict public run command`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Codex`

Status: `COMPLETE`

Base commit: `f04c17b325d54327a8da3f851d70ef38f4dd4334`

Original Packet 2 commit: `615ecf9b13117649de65ab8f0cac55393113450d`

Correction commit: `cf0d99a58d859b787b4fa6ff8f33fab58177fc97`

Implementation checkpoint: `cf0d99a58d859b787b4fa6ff8f33fab58177fc97`

## Requested outcome

Add the frozen, strict public `run` command for one caller-selected configured
Tether while preserving the accepted host admission, approval, replay, Trail,
and execution boundaries.

## Changes made

- `EngineError::ValidationFailed` now reaches the existing
  `ExecutionServiceError::TetherValidation` production path, which maps to the
  safe public `invalid_data` / exit `3` / `TETHER_INVALID` envelope.
- Interrupted validation and evaluation remain `interrupted` / exit `10`.
  Child, initialise, protocol, serialisation, EOF, framing, response, and
  other operational engine failures remain unavailable / exit `4` with the
  safe `EXECUTION_UNAVAILABLE` public code.
- A valid planner `status: "error"` remains a typed `PlannerError`, mapped to
  `invalid_data` / `PLANNER_ERROR`; transport failures are no longer reported
  as invalid user data.
- The public coordinator now has one small internal service-invocation seam.
  Its real `run` path still uses the sole `HostExecutionService` coordinator;
  focused tests prove the service boundary is reached only after canonical path
  checks, strict parsing, runtime preparation, exact configured-Tether
  selection, and durable initial event admission.
- The public acceptance script now verifies invalid configured Tethers return
  `TETHER_INVALID` with no provider method marker, and malformed or unknown
  Tether input produces no provider launch marker/methods.

## Decisions and assumptions

- Strict `run` input parsing rejects duplicate keys at every depth and preserves
  caller-provided evaluation and event identifiers.
- The command owns no policy, causal metadata, replay identity, approval
  identity, or execution machinery. Ask exposes only the evaluation ID, action
  ID, and redacted policy reason; it creates no public approval ID.
- The reviewed standing-Allow fixture and real-engine/public-provider script
  continue to cover completed, replay, no-actions, Deny, Ask, unavailable
  replay, frozen CLI behaviour, and real Ctrl+C handling.

## Evidence

- `j13b_engine_validation_and_evaluation_failures_remain_typed` proves
  validation maps to `TetherValidation`, a protocol failure maps to
  `Unavailable`, and interruption remains `Interrupted`.
- `j13b_run_service_errors_have_safe_frozen_statuses` proves public
  `TETHER_INVALID` and safe `EXECUTION_UNAVAILABLE` mappings without raw engine
  diagnostics.
- `j13b_correlated_and_minimal_planner_errors_are_distinct` retains the
  `PlannerError` / `PLANNER_ERROR` classification proof.
- `j13b_run_rejections_do_not_reach_the_service_boundary` covers malformed
  JSON, duplicate key, unknown field, invalid type and value, unknown Tether,
  invalid path, and simulated initial Trail open/write/sync failures with zero
  service calls.
- `j13b_run_admits_before_invoking_the_service_boundary` proves exactly one
  service invocation only after an `event_admitted` Trail entry exists.
- Existing correlation, not-matched, planner-error, missing-status, Ask, and
  zero replay/provider call tests remain in the J13B focused suite.

- OCaml `5.5.0`, Dune `3.24.0`, and current-worktree `dune build`: PASS.
- `cargo fmt --check`, `cargo check`, `cargo check --tests`, `cargo clippy`,
  `cargo build`, and `cargo build --release`: PASS.
- Focused Rust: J12 `99`; J13A `74`; J13B `51`; J13B run `15`; full suite
  `731` passed, `0` failed.
- Fixture check: `46` JSON and `30` JSONL files valid. Engine script: PASS.
  MCP transcript script: `15` cases PASS. J13A public acceptance: `25` passed,
  `0` failed. J13B public run acceptance: `10` passed, `0` failed.
- `git diff --check`: PASS before the correction commit; rerun after this
  documentation checkpoint.

## Discoveries

- The first standing-Allow fixture attempt used an unrestricted scope with a
  standing confirmation and was rejected by manifest validation. The final
  fixture uses the existing structured `/path` scope-binding model and its
  verified canonical digest.
- The first public acceptance attempt used an ordinary temporary replay root
  and was rejected as `PersistenceUnavailable` by the existing ACL gate. The
  script now creates an isolated owner-restricted empty test root and invokes
  the hidden `provision-replay` command; it does not create replay state itself.
- The first aggregate script pass completed its first nine public run cases but
  observed the existing Ctrl+C case as `unavailable`; a clean repeat passed all
  ten cases. No source change was made for that non-repeatable observation.
- `docs/RUST_ENGINEERING_GUIDE_FOR_AGENTS.md` was read before the correction.
  Its Rust `1.89.0` toolchain is not installed (`rustup run 1.89.0 rustc
  --version` reports a missing manifest), so no installation was attempted.
  The packet's specified bare Cargo commands were verified with installed Rust
  `1.97.1` / Cargo `1.97.1`.

## Remaining risks

No Packet 2 implementation or verification risk remains. The unavailable
Rust 1.89.0 guide baseline is recorded above without modifying local tooling.

## Smallest next action

Publish the two completed correction/checkpoint commits only on
`codex/j13b-run-command`; do not begin a subsequent packet.

## References

- Branch: `codex/j13b-run-command`
- Base: `f04c17b325d54327a8da3f851d70ef38f4dd4334`
- External read-only switch: `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`
