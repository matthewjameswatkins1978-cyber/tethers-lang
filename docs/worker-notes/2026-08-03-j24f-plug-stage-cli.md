# Worker Note

Task: `J24F - Public Plug stage CLI`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `9ceb7b2711bc387365b9a5382b84af1bb285384b`

Implementation checkpoint: `191273ff5297c1d93f64c6c491c87fc5961e6ce1`

## Requested outcome

Expose the accepted J24E candidate-preparation service through one public
`plug stage` command. Keep the adapter thin: validate only the two absolute
CLI paths, call J24E once, map its stable errors, and emit only the frozen
public candidate shape.

## Changes made

- `tethers-0.1/host-rust/src/cli.rs` — added the exact `PlugCommand::Stage`
  variant and strict syntax tests for split/equal-sign options, missing,
  duplicate, unknown and positional arguments.
- `tethers-0.1/host-rust/src/application.rs` — added one route to
  `plug_command::run_stage`.
- `tethers-0.1/host-rust/src/plug_command.rs` — added `run_stage`, absolute
  path usage checks, one J24E service call, exact error-code/message mapping,
  disposition mapping, capability ordering, and the explicit public output
  allowlist.
- `tethers-0.1/host-rust/tests/j24f_plug_stage_cli.rs` — compiled-binary tests
  for success, exact replay, service failures, semantic conflict, corrupt
  evidence, strict CLI usage, process/envelope parity, and Windows junction
  rejection.

## Decisions and assumptions

- Public formatting is constructed field-by-field rather than serialising
  `CandidateRecord`; this prevents quarantine paths, payload evidence, launch
  details, record digests and lifecycle evidence from crossing the boundary.
- Capability ordering is `(name, version, operation)` as frozen by the J24F
  blueprint.
- The adapter performs no filesystem operation itself. All candidate state
  mutation remains inside `prepare_installation_candidate`.

## Evidence

- `cargo +1.89.0 fmt --all -- --check` — PASS
- `cargo +1.89.0 test cli --locked` — 36 passed
- `cargo +1.89.0 test plug_command --locked` — 5 passed
- `cargo +1.89.0 test --test j24f_plug_stage_cli --locked` — 6 passed
- `cargo +1.89.0 test --test j24a_plug_inspect_cli --locked` — 3 passed
- `cargo +1.89.0 test --test j24b_plug_list_cli --locked` — 4 passed
- `cargo +1.89.0 test --test j24c_plug_disable_cli --locked` — 9 passed
- `cargo +1.89.0 test --test j24d_plug_enable_scope_file --locked` — 16 passed
- `cargo +1.89.0 test candidate_preparation --locked` — 10 passed
- `cargo +1.89.0 test --test j24e_candidate_preparation --locked` — 17 passed
- `cargo +1.89.0 test --all-targets --all-features --locked` — 919 passed,
  5 documented `pwsh.exe not found` baseline failures
- `git diff --check` — PASS
- `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1` — BLOCKED:
  the authoritative J24F packet is missing the checker-required section
  `Relevant background and existing behaviour` (and also omits the checker’s
  `Relevant components` and `Frozen decisions and invariants` sections).

Compiled-binary evidence proves:

- first deterministic PDF package stage returns `created`, pins
  `tethers.pdf-tools`, `tethers-pdf-provider`, Windows `x86_64`, and
  `pdf.inspect@1`, and creates exactly candidate/quarantine state;
- exact replay returns `existing`, the same candidate ID, and an identical
  relative-path/SHA-256 host snapshot;
- malformed, missing, semantic-conflict, corrupt-record, relative-path and
  junction scenarios preserve the required error code, status, exit code and
  no-mutation boundary;
- every parsed invocation emits one JSON line and the process exit equals the
  envelope exit code.

## Discoveries

- The full-suite five failures remain the documented environment baseline: the
  execution-environment tests cannot find `pwsh.exe` in their test lookup.
- Existing compiler warnings are unrelated dead-code and unused-test-fixture
  warnings; no new warning was introduced by J24F.

## Remaining risks

None known within packet scope.

## Smallest next action

Lucy performs the bounded final review of the pushed J24F branch.

## References

- Branch: `opencode/j24f-plug-stage-cli`
- Base: `9ceb7b2711bc387365b9a5382b84af1bb285384b`
- Implementation checkpoint: `191273ff5297c1d93f64c6c491c87fc5961e6ce1`
- Blueprint: `docs/architecture/J24F_PLUG_STAGE_CLI_BLUEPRINT.md`
- Service boundary: `tethers-0.1/host-rust/src/candidate_preparation.rs`
- Public adapter: `tethers-0.1/host-rust/src/plug_command.rs`
- Compiled evidence: `tethers-0.1/host-rust/tests/j24f_plug_stage_cli.rs`
