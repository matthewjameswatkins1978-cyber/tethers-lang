# Current Implementation Task

Control contract: `1`

Task: `TETHERS-0.4-C1 — Together: Deterministic Fan-Out / Join Foundation`

Owner: `OpenCode`

Status: `IN_PROGRESS`

Task colour: `Amber`

Route: `OpenCode implementation + evidence → Lucy independent GitHub review`

Worker note: `docs/worker-notes/2026-08-11-0.4-c1-together-fan-out-join.md`

Base branch: `feature/0.3-p6-evil-bunny-adversarial-provider-proof`

Base commit: `5ed7634d8abc4056e0faa1ff09924377dec6e645`

OCaml switch path: `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`

Rust toolchain: read exact channel from `rust-toolchain.toml`; use plain Cargo (resolved by root pin); `--locked` mandatory

Toolchain preflight: `pwsh -NoProfile -File scripts/check-dev-tools.ps1` (run; all tools present)

Rust change class: `NON_RUST`

P6 is FINAL ACCEPTED at `5ed7634d8abc4056e0faa1ff09924377dec6e645`. Do not alter P6 implementation or evidence except for necessary status references.

## Objective

Introduce the first real Tethers concurrency primitive — the `together` fan-out / join block — as deterministic language semantics in the OCaml engine, without turning the language into a scheduler and without requiring physical parallel execution. A Tether may declare that several independent Actions are members of one concurrency group; later Actions become executable only after every member has reached a terminal outcome and the group has joined. The C1 reference runtime may execute group members serially in deterministic source order as one valid schedule.

## Relevant background and existing behaviour

- P6 FINAL ACCEPTED at `5ed7634d8abc4056e0faa1ff09924377dec6e645`; the 0.1 language and protocol semantics are defined by `tethers-0.1/SPEC.md`.
- The engine (`tethers-0.1/engine-ocaml/bin/`) parses a frozen 0.1 grammar: `do` body contains Actions at 4-space indentation with arguments at 8-space; Actions are planned in source order with position-derived `action_id` (`action_1`, `action_2`, …), `idempotency_key = evaluation_id/action_id`, resolved arguments, and declared Effects; the deterministic planner Trail records `event_received`, `anchor_checked`, `condition_checked`, and `action_planned` entries in causal sequence.
- The plan response (`plan.actions`) is a flat, ordered array of Action objects; the Rust host consumes only `plan.actions` (and ignores unknown additive plan fields), and the demo boundary currently enforces exactly one Action per plan.
- `tethers.validate` (MCP adapter) reports `action_count` as the number of Actions in the parsed source.
- The parser rejects malformed structures with `parse_error` and exact, fixture-protected messages; protocol fixtures live under `tethers-0.1/protocol/cases/<case>/` (`request.json` + `expected-response.json`), MCP transcripts under `tethers-0.1/protocol/mcp-transcripts/<case>/`, and `test-engine.ps1` / `test-mcp-transcripts.ps1` auto-discover them.
- Determinism: identical input must produce byte-equivalent semantic output; array ordering and Trail sequence are semantic.
- `tethers-0.1/SPEC.md` currently defines no `together` construct; this packet is the explicit design gate that adds it.

## Required behaviour

1. Create branch `feature/0.4-c1-together-fan-out-join` from the exact P6 accepted HEAD `5ed7634d8abc4056e0faa1ff09924377dec6e645`, update `docs/CURRENT_CLINE_TASK.md` to this packet with Status `IN_PROGRESS`, and pass the packet checker (`control-v1/IN_PROGRESS`) before production edits.
2. P6 closeout bookkeeping: update `docs/ROAD_TO_0_3.md`, `docs/CURRENT_GOAL.md`, and `docs/PROJECT_DASHBOARD.md` wording so P6 is shown FINAL ACCEPTED at `5ed7634d8abc4056e0faa1ff09924377dec6e645`, C1 is the active increment, and nothing beyond C1 (P7 / physical-parallel 0.4 work) has started. Do not alter P6 implementation or evidence except for these status references.
3. Extend the parser so a `do` body contains Action items: an ordinary Action or a `together` block. A `together` block contains ordinary Actions one indentation level beneath it (members at 8 spaces, member arguments at 12 spaces) and closes when the next item appears at the `do` level or the source ends. Reject, using the existing `parse_error` convention: an empty `together` block, a block with fewer than two members, a nested `together` (a member line exactly `together`), wrong indentation for members or member arguments, and any other C1-restriction violation. `together` alone at the `do` level is the keyword and may not be used as an Action name.
4. Extend the evaluator so group members are planned as Actions in deterministic source order with contiguous position-derived `action_id`s, declared as one concurrency group with a join point: any Action whose source position follows the group is planned only after the whole group, and the deterministic planner Trail records one group-planning entry after the group's last member. No implicit concurrency: two adjacent ordinary Actions remain sequential, and a Tether without `together` must produce exactly the same semantic plan, Trail, and behaviour it produced before C1.
5. Represent the concurrency group in the matched plan additively: keep `plan.actions` as the flat, ordered list of every planned Action in source order (this list is the deterministic serial schedule, a valid C1 schedule), and add a `plan.groups` array (present only when at least one `together` block exists) whose entries declare `group_id` (position-derived, `group_1`, `group_2`, …) and the `member_action_ids` of the group in source order.
6. Keep the MCP `tethers.validate` surface coherent: `action_count` counts every planned Action, including `together` members.
7. Add targeted automated regressions: new engine fixture cases under `protocol/cases/` covering a valid fan-out/join, ordering across a group, a pure fan-out Tether, two sibling groups, and every rejected malformed shape (empty, single member, nested, wrong member indentation, member planning failure); a `tethers.validate` MCP transcript for a `together` source; and a deterministic repeat check for a `together` case in `test-engine.ps1`. Preserve all existing fixtures unchanged.
8. Do not change Rust host production code. Prove host compatibility: the additive plan field is ignored by the existing consumers, the engine binary still satisfies the host suite, and `cargo fmt --all -- --check` plus the full locked host test suite pass against the new engine output.
9. Update `tethers-0.1/SPEC.md` so the precise language and protocol semantics document the `together` grammar, the C1 restrictions, the join semantics, and the additive `groups` plan field, without disturbing any other 0.1 contract.
10. Close out per project control: commit the implementation checkpoint, write the worker note at the named path, set the packet to `COMPLETE`, require checker `control-v1/COMPLETE`, commit the docs-only closeout, push the branch normally to `origin`, resolve the full remote HEAD SHA, confirm local `HEAD == remote HEAD`, and confirm a clean worktree.

## Relevant components

- `tethers-0.1/engine-ocaml/bin/tether_parser.ml` and `tether_parser.mli` (Action-item grammar)
- `tethers-0.1/engine-ocaml/bin/tethers_evaluator.ml` (group planning, deterministic Trail)
- `tethers-0.1/engine-ocaml/bin/tethers_outcome.ml` and `tethers_outcome.mli` (plan type, `groups` encoding)
- `tethers-0.1/engine-ocaml/bin/tethers_mcp_server.ml` (`tethers.validate` `action_count`)
- `tethers-0.1/SPEC.md` (grammar and semantics update)
- `tethers-0.1/protocol/cases/` and `tethers-0.1/protocol/mcp-transcripts/` (new fixtures)
- `tethers-0.1/scripts/test-engine.ps1` (case discovery + deterministic repeat)
- `docs/ROAD_TO_0_3.md`, `docs/CURRENT_GOAL.md`, `docs/PROJECT_DASHBOARD.md` (P6 closeout wording)
- `docs/worker-notes/2026-08-11-0.4-c1-together-fan-out-join.md` (new worker note)
- Read-only compatibility reference: `tethers-0.1/host-rust/` (consumes `plan.actions`; unchanged)

## Frozen decisions and invariants

- `together` is the C1 keyword; it is reserved as an Action name. Only an explicit `together` block creates concurrent semantics; adjacent ordinary Actions stay sequential.
- A `together` block: must contain at least two Actions; cannot be empty; cannot contain another `together` block; cannot contain Conditions, branching, loops, Action-result references, retries, compensation, or dynamic membership.
- C1 establishes concurrency semantics, not physical parallelism: the flat ordered `plan.actions` list is the deterministic serial schedule and is a valid C1 execution schedule.
- Actions remain ordered; Action IDs stay position-derived across the whole plan; `idempotency_key` remains `evaluation_id/action_id`.
- The `groups` plan field is additive and omitted entirely when no `together` block exists, so a Tether without `together` produces byte-identical output to pre-C1.
- New rejection messages use the existing `parse_error` convention with stable, fixture-protected wording.
- Determinism, array ordering, and Trail sequence remain exact; Core stays timestamp-free and effect-free; the Core/host boundary is unchanged.
- No Rust host production change; no dependency, toolchain, Dune, or OCaml-version change; no change to sequential Tether semantics, identities, or existing fixtures.
- P6 implementation and evidence are not altered except for necessary status references.

## Acceptance criteria

1. Branch `feature/0.4-c1-together-fan-out-join` is based on `5ed7634d8abc4056e0faa1ff09924377dec6e645`, the packet is `IN_PROGRESS`, and the packet checker reports `control-v1/IN_PROGRESS` before production edits.
2. `docs/ROAD_TO_0_3.md`, `docs/CURRENT_GOAL.md`, and `docs/PROJECT_DASHBOARD.md` show P6 FINAL ACCEPTED at `5ed7634d8abc4056e0faa1ff09924377dec6e645`, C1 active, and P7 / physical-parallel 0.4 NOT started.
3. Engine fixture evidence proves every rejected shape (empty block, single member, nested `together`, wrong member indentation) is refused with `parse_error`, and a valid `together` block parses and plans.
4. Engine fixture evidence proves fan-out/join planning: members appear in the flat `actions` list in source order with contiguous `action_id`s, a `group_planned` Trail entry follows the group's last member, and later Actions are planned after the group.
5. Engine fixture evidence proves the additive protocol contract: `plan.groups` exists with the correct `group_id`/`member_action_ids` only when `together` is used; every pre-existing fixture (no `together`) passes unchanged; adjacent ordinary Actions remain sequential.
6. MCP transcript evidence proves `tethers.validate` reports `action_count` covering all planned Actions for a `together` source, with no change to existing validate transcripts.
7. All new fixture cases, the transcript, and the `test-engine.ps1` deterministic repeat for the `together` case are committed and pass; each negative branch has its own direct evidence.
8. `cargo fmt --all -- --check` passes and the full locked host test suite passes with zero failures against the new engine, with no host source change.
9. `tethers-0.1/SPEC.md` documents the `together` grammar, C1 restrictions, join semantics, and the additive `groups` plan field; the worker note records the exact section changes.
10. Closeout evidence: worker note exists at the named path with the implementation checkpoint SHA, checker reports `control-v1/COMPLETE`, branch pushed normally to `origin`, full remote HEAD SHA resolved, local `HEAD == remote HEAD`, and `git status --short --branch` clean.

## Required verification

1. Packet checker at start (`control-v1/IN_PROGRESS`) and on closeout (`control-v1/COMPLETE`):
   `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1`
2. OCaml build through the explicit switch from the engine source directory:
   `opam exec --switch=<OcamlSwitchPath> -- dune build`
3. Engine fixture and transcript suites:
   `pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\check-fixtures.ps1`
   `pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\test-engine.ps1`
   `pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\test-mcp-transcripts.ps1`
4. Host compatibility (no host source change; NON_RUST):
   `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all -- --check`
   `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml --all-targets --all-features --locked`
5. `git diff --check`, complete diff inspection, and final `git status --short --branch` inspection.

## Formatting and checkpoint sequence

NON_RUST task: run `cargo fmt --all -- --check` only; never run a mutating formatter and never modify Rust source. The engine has no project formatter; preserve local OCaml style. The implementation checkpoint commit precedes all worker-note, packet, and dashboard closeout edits. `docs/ROAD_TO_0_3.md` and `docs/CURRENT_GOAL.md` wording updates are implementation scope and precede the checkpoint commit; the packet, worker note, and `docs/PROJECT_DASHBOARD.md` are closeout scope.

## Completion and publication

Commit the implementation/proof checkpoint, write the worker note at the named path, set this packet to `COMPLETE`, require checker `control-v1/COMPLETE`, commit the docs-only closeout, then push the named branch normally and prove `origin/feature/0.4-c1-together-fan-out-join == HEAD` with a clean worktree. Do not start P7 or any physical-parallel 0.4 increment.

## Forbidden changes

- No P7, no physical-parallel execution increment, no nested concurrency, no scheduler, no worker/thread/async runtime in Core or host.
- No Rust host production code, dependency, toolchain, Dune, or OCaml-version changes.
- No change to 0.1 sequential Tether semantics, Action identities, idempotency material, existing error contracts, or existing fixtures/transcripts.
- No change to the Core/host boundary, permission, trust, replay, or Trail-ownership semantics.
- No merge, amend, tag, force-push, PR, or direct `main` update.

## Stop conditions

- A real contradiction between the frozen C1 semantics and repository evidence that cannot be resolved from this packet.
- A consequential architecture/product/security/trust decision beyond the frozen decisions requiring external authority.
- Two materially similar implementation attempts fail on the same unresolved underlying problem.
- An unrelated environmental failure prevents trustworthy verification of a required check.

## Expected pre-existing changes

None. Base commit is the accepted P6 HEAD; the C1 branch starts clean at it.
