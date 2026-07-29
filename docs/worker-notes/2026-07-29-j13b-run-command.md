# J13B Packet 2 Worker Note

Task: `J13B Packet 2 — strict public run command`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Codex`

Status: `COMPLETE`

Base commit: `f04c17b325d54327a8da3f851d70ef38f4dd4334`

Implementation checkpoint: `WORKTREE`

## Requested outcome

Add the frozen, strict public `run` command for one caller-selected configured
Tether while preserving the accepted host admission, approval, replay, Trail,
and execution boundaries.

## Changes made

- Added strict `run` input parsing in `run_input.rs`, including duplicate-key
  rejection at every depth, exact public shape, stable safe error codes, and
  caller-preserved evaluation/event identifiers.
- Added the thin `run_command.rs` coordinator: caller-CWD path handling, exact
  configured-Tether selection, durable external admission, one service call,
  one-result enforcement, and frozen CLI-envelope mapping.
- Added the public clap command and thin dispatch only. The execution service
  gained a selected-Tether entry point so a public one-Tether run does not
  validate unrelated configured Tethers before provider launch.
- Reused the existing approval-request seam for Ask with a process-local store
  and the same Trail; no approval ID is returned publicly.
- Added a reviewed standing-Allow fixture manifest and `run-success` fixture
  mode for the completed/replay proof. Existing `fixture-ping.json` semantics
  remain unchanged.
- Added the nine-case real-engine/public-provider acceptance script, including
  a real isolated-console Ctrl+C controller.
- Appended the Packet 2 public-boundary decision.

## Decisions and assumptions

The external OCaml switch was used only as a process-local read-only toolchain.
The public command owns no policy, causal metadata, replay identity, approval
identity, or execution seam. The replay backend remains the only authority
that accepts and provisions a host-data root.

## Evidence

Preflight confirmed clean `origin/main` and branch base
`f04c17b325d54327a8da3f851d70ef38f4dd4334`, all packet scripts, and the
external switch directory.

- `cargo fmt --check`, `cargo check`, `cargo check --tests`, `cargo clippy`,
  `cargo build`, and `cargo build --release`: PASS.
- Focused Rust: J12 `99`; J13A `74`; J13B `48`; J13B run `13`; full suite
  `728` passed, `0` failed.
- Fixture check: `46` JSON and `30` JSONL files valid.
- Engine script: all named cases PASS. MCP transcript script: `15` cases PASS.
- J13A public acceptance: `25` passed, `0` failed. J13B public run acceptance:
  `9` passed, `0` failed, covering completed, replay, no-actions, Deny, Ask,
  unavailable replay, invalid input, CLI shape, and actual Ctrl+C.
- Explicit external-switch build: OCaml `5.5.0`, Dune `3.24.0`, PASS.

## Discoveries

The project’s Rust implementation guidance is contained in
`docs/IMPLEMENTATION_LANGUAGE_STANDARD.md`; there is no separate Rust guide
under `docs`.

The first standing-Allow fixture attempt used an unrestricted scope with a
standing confirmation and was correctly rejected by manifest validation. The
final fixture uses the existing structured `/path` scope-binding model and its
verified canonical digest.

The first public acceptance attempt used an ordinary temporary replay root and
was correctly rejected as `PersistenceUnavailable` by the existing ACL gate.
The acceptance script now creates an isolated owner-restricted empty test root
and invokes the existing hidden `provision-replay` command; it does not create
replay state itself.

## Remaining risks

No unresolved implementation or verification risk remains in Packet 2.

## Smallest next action

Inspect the complete base-to-head diff, run whitespace and task-packet checks,
then commit and push only `codex/j13b-run-command`.

## References

- Branch: `codex/j13b-run-command`
- Base: `f04c17b325d54327a8da3f851d70ef38f4dd4334`
- External read-only switch: `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`
