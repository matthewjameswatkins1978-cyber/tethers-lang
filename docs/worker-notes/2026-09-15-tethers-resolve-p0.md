# Tethers x Resolve01 P0 worker note

Task: `TETHERS x RESOLVE01 / P0`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Codex`

Status: `IN_PROGRESS`

Base commit: `ddc5d0fccfc0a38ff0af3013c5e9ff6ba357e0b2`

Implementation checkpoint: `WORKTREE`

## Requested outcome

Freeze the Resolve Guard Adapter contract and exact Tethers host insertion
points in a documentation-only P0 change, preserving Tethers Core and all
existing authority, replay, Trail, provider, outcome, and Plug machinery.

## Changes made

Created the P0 control packet and the architecture/seam audit. The note names
the two future components, maps current Tethers ownership and code seams,
freezes preparation/resume and ScopeKey rules, records the shared serial and
Together-group insertion boundary, and defers the live Resolve wire and public
Resolve01 capability contract.

## Decisions and assumptions

The exact base is the fetched `origin/main` commit. The P0 branch is isolated
from the canonical `main` checkout. P0 is documentation-only: no Rust, OCaml,
configuration, package, fixture, or test semantics are changed. The current
reviewed scope forms remain bounded; P0 does not broaden scope machinery.

## Evidence

The canonical checkout was fetched and verified clean at the exact base before
the isolated worktree was created. `scripts/check-dev-tools.ps1` passed. The
P0 worktree resolves to the exact base and had no branch-owned implementation
changes before this note. The repository control documents and Tethers
specialist guidance were read before mutation.

## Discoveries

The existing shared boundary already orders replay admission, durable intent,
invocation arming, provider call, outcome classification, durable outcome,
and Result Anchor. Together groups split preparation from invocation but retain
the same coordinator-owned replay and provider machinery. The correct future
guard seam is post-durable-intent and pre-provider, including before existing
G1 arming; a Resolve Plug must not carry the target capability call.

## Remaining risks

P1 must select the exact typed proof representation and safe scope projection
without introducing a second canonicaliser. P2 depends on a narrow fake seam
until Resolve S3 freezes the live protocol. Outcome-delivery persistence and
redelivery remain unimplemented and must not be inferred from this note.
Resolve01 public provider operations are intentionally unknown until its
contract is frozen.

## Smallest next action

Run the P0 documentation verification and inspect the complete branch diff.
Then record the committed checkpoint and final packet state only if every P0
acceptance criterion is evidenced; stop for review before selecting P1.

## References

- `docs/PROJECT_CONTROL.md`
- `docs/AGENT_WORKFLOW.md`
- `docs/IMPLEMENTATION_LANGUAGE_STANDARD.md`
- `docs/GIT_WORKTREES_AND_LINE_ENDINGS_FOR_AGENTS.md`
- `docs/CONSTITUTION.md`
- `tethers-0.1/SPEC.md`
- `docs/DECISIONS.md`
- `docs/CAPABILITY_BRIDGE.md`
- `docs/VERSIONING.md`
- `docs/COMPATIBILITY.md`
- `tethers-0.1/host-rust/src/configured_runtime.rs`
- `tethers-0.1/host-rust/src/application.rs`
- `tethers-0.1/host-rust/src/host_execution.rs`
- `tethers-0.1/host-rust/src/dispatch.rs`
- `tethers-0.1/host-rust/src/replay_runtime.rs`
