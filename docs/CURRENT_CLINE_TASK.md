# Tethers Agent Essentials — Phase A Discovery Surface

Control contract: `1`

Status: `IN_PROGRESS`

Task colour: `Red`

Owner: `Lucy`

Route: `Codex direct implementation in dedicated review worktree; do not merge to main`

Base commit: `5cce71f8f93be26a0dfd1a0e50935f9419a5c284`

Implementation checkpoint: `WORKTREE`

Worker note: `docs/worker-notes/2026-09-01-agent-essentials-phase-a.md`

Updated: 2026-09-01

## Objective

Expose the existing trusted Plug and capability state through deterministic,
read-only machine-readable discovery commands. This is Phase A of the Agent
Essentials milestone and must not redesign the Core, provider trust model,
policy engine, lifecycle stores, or execution semantics.

## Relevant background and existing behaviour

The reference host already owns installed Plug records, enablement transitions,
trusted capability manifests, provider bindings, operational scopes, and
conformance evidence. Existing `plug list` and `plug inspect --package`
commands are compact or package-source oriented. This phase projects the
installed state without starting providers or requiring the original package.

The existing CLI envelope, status vocabulary, exit codes, manifest verification,
installed-record registry, enablement store, and Plug lifecycle evidence are
frozen public behaviour.

## Required behaviour

Implement these additive commands:

1. `tethers describe --json`
2. `tethers capability list --host-data-root <absolute-path> [--all] [--effect <effect>] [--provider <provider>] [--plug <installed-id>] --json`
3. `tethers capability inspect <name> --host-data-root <absolute-path> [--version <version>] --json`
4. `tethers plug show --host-data-root <absolute-path> --installed-id <id> --json`

Discovery must be deterministic, sorted by canonical capability identity,
return trusted manifest data rather than live provider advertising, distinguish
disabled from available state, expose no secret values, and fail closed on
missing, incomplete, corrupted, mismatched, duplicate, or ambiguous state.

`describe` may report configured-state health only; it must not imply that a
provider was contacted. The `tethers` binary is an additive alias of the
existing reference-host entry point; existing binary and envelope behaviour
remain intact.

## Relevant components

* `tethers-0.1/host-rust/src/cli.rs`
* `tethers-0.1/host-rust/src/application.rs`
* `tethers-0.1/host-rust/src/discovery.rs`
* `tethers-0.1/host-rust/src/installed.rs`
* `tethers-0.1/host-rust/src/enablement.rs`
* `tethers-0.1/host-rust/src/manifest.rs`
* `tethers-0.1/host-rust/src/package.rs`
* `tethers-0.1/host-rust/src/plug_command.rs`

## Frozen decisions and invariants

* Do not redesign existing language, Core, planning, policy, trust, provider,
  Plug, Trail, replay, scope, or Together semantics.
* Do not create a second registry, policy engine, scheduler, daemon, or hidden
  authority path.
* Do not start providers, execute capabilities, mutate lifecycle state, or
  claim live health during discovery.
* Preserve the `tethers.cli/1` envelope, status vocabulary, and exit codes.
* Verify installed manifest identity and digest against installed evidence.
* Never silently select a capability version when several versions match.
* Do not expose secrets or mutable private provider state.

## Acceptance criteria

1. All four commands parse strictly and emit stable JSON.
2. Capability list is compact, filtered, and deterministic.
3. Capability inspect exposes the trusted contract, binding, provider, Plug,
  scope, availability, digest, and conformance evidence.
4. Plug show works without the source package and remains compact. Corruption,
   incomplete stores, digest mismatch, duplicate versions, and
  ambiguous version selection fail closed with precise error codes.
5. No provider process or external operation is started by any discovery command.
6. Existing commands and contracts remain compatible.

## Required verification

* `cargo fmt --all -- --check`
* `cargo check --all-targets --all-features --locked`
* focused CLI parsing and discovery tests
* `git diff --check`
* manual `tethers describe --json` smoke test

## Forbidden changes

* No implementation of workspace, Git, process, network, archive, SQLite,
  structured-data, or other Agent Essentials providers in this Phase A packet.
* No automatic Plug trust, enablement, broad scopes, shell escape hatch, or
  secret access.
* No merge to `main`.

## Stop conditions

Stop and report `BLOCKED` if the required projection cannot be achieved without
changing a frozen authority or evidence boundary, if authentication or review
is required, or if unrelated changes become necessary.

## Expected pre-existing changes

The baseline host suite contains known fresh-worktree engine-fixture and
concurrency stress failures; do not alter frozen fixtures or claim those
failures as caused by this Phase A slice without new evidence.
