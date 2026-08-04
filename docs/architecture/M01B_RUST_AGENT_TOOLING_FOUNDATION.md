# M01B Rust Agent Tooling Foundation

Status: frozen implementation blueprint

## Purpose

Equip the active Tethers implementation workflow with a small, reproducible Rust
agent toolset without combining tool installation with dependency changes,
warning cleanup, dead-file deletion, production refactoring, or Plug installation
work.

M01B is diagnostic and operational infrastructure. It makes better information
available to OpenCode and reviewers. It does not grant any tool authority to
rewrite code, remove dependencies, retry failing tests, or change accepted
behaviour automatically.

M01C will separately use the evidence produced by these tools to address warning
debt, inactive agent configuration, duplicated guidance, obsolete scripts, and
safe deletions.

## Accepted toolset

The exact accepted tools are:

```text
rust-analyzer   Rust 1.97.1 rustup component
cargo-nextest   0.9.140
cargo-deny      0.19.7
cargo-machete   0.9.2
```

These versions are frozen for M01B. Do not substitute a newer release discovered
during implementation.

`cargo-audit` is not adopted because cargo-deny already supplies the accepted
advisory gate. `cargo-semver-checks` is deferred until Tethers explicitly
supports a public Rust library API with a semantic-version compatibility promise.

## Tool roles

### rust-analyzer

`rust-analyzer` provides language-aware definitions, references, symbols, hover
information, and diagnostics. It is navigation and feedback assistance, not
build authority.

The compiler, Clippy, tests, and frozen contracts remain authoritative when LSP
feedback differs or becomes stale.

The component is owned by the exact Rust 1.97.1 toolchain. Add it to the root
`rust-toolchain.toml` component list rather than installing an unrelated weekly
binary.

### cargo-nextest

Nextest provides an alternative agent test loop with per-test process isolation,
filtering, and clearer test-level reporting.

It does not replace ordinary `cargo test` because its execution model differs and
ordinary Cargo remains the compatibility and completion authority.

Nextest retries are forbidden. A flaky or failing test must remain visible.
M01B must benchmark and record nextest on native Windows but must not claim it is
faster before measurement. The repository recipe is named `test-agent`, not
`test-fast`.

### cargo-deny

Cargo-deny is the repository dependency-policy gate for:

- security and soundness advisories;
- licence allowlisting;
- duplicate and banned dependency review;
- dependency-source restrictions.

It is the accepted advisory engine; do not add cargo-audit in parallel.

A generated default policy is not sufficient. Commit one small reviewed
`deny.toml` whose settings are understandable from the file itself.

Advisory ignores are forbidden in M01B. If the current lockfile triggers an
advisory, stop with the exact advisory and affected dependency rather than
silently adding an exception.

Licence policy may allow only these pre-approved permissive identifiers:

```text
MIT
Apache-2.0
Apache-2.0 WITH LLVM-exception
BSD-2-Clause
BSD-3-Clause
ISC
Unicode-3.0
Zlib
CC0-1.0
BSL-1.0
```

If the current graph requires another licence expression, stop and report the
exact crate and expression. Do not broaden the list autonomously.

Unknown registries and unknown Git sources must be denied. Multiple dependency
versions begin as warnings, not automatic failures, because transitive duplicate
removal is dependency work outside this task.

### cargo-machete

Cargo-machete is an advisory unused-dependency detector. Run it with metadata to
reduce false positives.

Never use `--fix`. M01B removes no dependency. Every finding is recorded for
M01C and requires independent source/reference inspection before any later
change.

## Repository-owned version authority

Add:

```text
tools/rust-agent-tools.json
```

with exactly:

```json
{
  "schema": 1,
  "cargo_nextest": "0.9.140",
  "cargo_deny": "0.19.7",
  "cargo_machete": "0.9.2",
  "rust_analyzer": "toolchain-component"
}
```

Installation and checking scripts must read these versions. Do not repeat the
cargo-tool versions in several executable scripts.

The Rust channel continues to come only from `rust-toolchain.toml`.

## Installation script

Add:

```text
scripts/install-rust-agent-tools.ps1
```

It must:

1. Resolve and verify the intended Git repository root.
2. Read and validate `tools/rust-agent-tools.json`.
3. Read the exact Rust channel from `rust-toolchain.toml`.
4. Require that exact toolchain to be installed already.
5. Add the `rust-analyzer` component to that exact toolchain when missing.
6. Inspect each Cargo tool's current version before installation.
7. Install only a missing or mismatched accepted version through:

   ```powershell
   cargo install --locked --version <exact> <crate>
   ```

   Use `--force` only for an installed version mismatch.
8. Make no global Rust default, PATH, Cargo configuration, or unrelated package
   change.
9. Be idempotent: a second invocation must perform no installation and succeed.
10. Return non-zero on every missing, malformed, failed, or version-mismatched
    final state.

Matthew explicitly authorises these exact installations for M01B.

## Non-mutating checker

Add:

```text
scripts/check-rust-agent-tools.ps1
```

It must be read-only and prove:

- accepted JSON schema and exact configured versions;
- rust-analyzer is declared in `rust-toolchain.toml`;
- rust-analyzer is installed for the exact root toolchain;
- `rust-analyzer --version` succeeds from the repository;
- cargo-nextest is exactly 0.9.140;
- cargo-deny is exactly 0.19.7;
- cargo-machete is exactly 0.9.2;
- OpenCode is available and reports its version;
- repository `opencode.json` enables LSP and permits the LSP tool;
- `.config/nextest.toml` and `deny.toml` exist;
- running the checker changes no repository byte or Git status.

The checker must expose an in-process function so focused tests can inspect its
exit code and captured output without launching nested shells.

Add:

```text
scripts/test-check-rust-agent-tools.ps1
```

with direct evidence for:

- missing config rejection;
- malformed JSON rejection;
- wrong schema rejection;
- impossible configured version rejection;
- real accepted configuration success;
- repository non-mutation.

## OpenCode LSP integration

Update `opencode.json` to preserve its existing instruction list and add:

```json
"lsp": true,
"permission": {
  "lsp": "allow"
}
```

Do not alter unrelated permissions or model/provider configuration.

Add:

```text
scripts/start-opencode-lsp.ps1
```

This is an opt-in launcher for the next OpenCode process. It must set only these
process-local values while OpenCode runs:

```text
OPENCODE_EXPERIMENTAL_LSP_TOOL=true
OPENCODE_DISABLE_LSP_DOWNLOAD=true
```

The second value ensures OpenCode uses the repository-provided rust-analyzer
rather than downloading another language server.

The launcher must preserve and restore pre-existing environment values on normal
exit and failure. It must forward arbitrary OpenCode arguments and return the
OpenCode exit code.

M01B must verify `opencode debug config` sees the repository LSP configuration.
If the installed OpenCode build does not support current LSP runtime behaviour,
do not pretend otherwise. Record the exact version and limitation, leave the
repository configuration ready, and stop before claiming complete agent LSP
integration.

A currently running OpenCode process is not required to hot-reload the newly
added experimental tool. The next process launched through the wrapper is the
acceptance target.

## Nextest configuration

Add:

```text
.config/nextest.toml
```

with a required minimum of exact M01B version 0.9.140 and an explicit no-retry
policy:

```toml
nextest-version = { required = "0.9.140" }

[profile.default]
retries = 0
fail-fast = true
```

Do not add retry overrides, flaky-test allowances, archive/replay, CI partition,
or test-specific exceptions.

## Cargo-deny configuration

Add root `deny.toml`.

It must:

- use the accepted licence list only;
- deny unknown registries and Git sources;
- deny wildcard dependency declarations;
- deny advisories without ignores;
- report multiple versions as warnings for later review;
- avoid crate-specific exceptions unless the frozen licence list genuinely
  requires an expression clarification backed by the current crate source.

No generated template commentary dump is accepted. Keep the policy short and
reviewable.

## Just recipes

Extend the root `justfile` with fail-fast recipes:

```text
agent-tools       non-mutating exact tool check
test-agent        cargo nextest run for the complete Rust test graph
deps-policy       cargo-deny licences, bans, and sources
deps-advisories   cargo-deny advisory check
deps-unused       cargo-machete advisory scan with metadata
verify-agent      normal just verify, agent-tools, dependency policy, and
                  nextest; ordinary cargo test remains inside normal verify
```

`deps-unused` is intentionally not part of `verify-agent` because findings are
advisory until M01C reviews them.

Do not add automatic installation to any verification recipe.

## Agent guidance

Update live guidance only:

- `AGENTS.md`;
- `docs/RUST_ENGINEERING_GUIDE_FOR_AGENTS.md`;
- `docs/TOOLCHAIN_POLICY.md`.

State clearly:

- use rust-analyzer for navigation when available;
- reread or compile when LSP state may be stale;
- use nextest for the agent loop, ordinary Cargo for final completion;
- never enable nextest retries;
- run cargo-deny for dependency changes;
- treat cargo-machete as a question, never deletion authority;
- do not add cargo-audit or cargo-semver-checks without a later decision.

Historical worker notes, completed packets, and release notes remain unchanged.

## Benchmark evidence

After a warm compile, measure three complete runs each of:

```text
cargo test --all-targets --all-features --locked
cargo nextest run --all-targets --all-features
```

Use the host manifest explicitly. Record individual durations and medians.

Do not alter antivirus, Defender, PATH, drive placement, CPU affinity, or process
priority to improve the result.

The benchmark is descriptive. Nextest remains an available alternative even when
native Windows process creation makes it slower, but documentation must state the
measured result honestly.

## Permitted files

M01B may add or modify only:

- `rust-toolchain.toml`;
- `tools/rust-agent-tools.json`;
- `scripts/install-rust-agent-tools.ps1`;
- `scripts/check-rust-agent-tools.ps1`;
- `scripts/test-check-rust-agent-tools.ps1`;
- `scripts/start-opencode-lsp.ps1`;
- `opencode.json`;
- `.config/nextest.toml`;
- `deny.toml`;
- `justfile`;
- `AGENTS.md`;
- `docs/RUST_ENGINEERING_GUIDE_FOR_AGENTS.md`;
- `docs/TOOLCHAIN_POLICY.md`;
- `docs/CURRENT_CLINE_TASK.md` for control state;
- `docs/worker-notes/2026-08-04-m01b-rust-agent-tooling.md`.

## Forbidden work

Do not modify:

- Cargo.toml dependencies, features, package identity, edition, or rust-version;
- Cargo.lock;
- production Rust or OCaml source;
- production or integration tests;
- OCaml compiler, switch, opam files, Dune files, or lock;
- CI workflows;
- Plug installation work;
- warning-producing source;
- `.clinerules`, `.clineignore`, old scripts, roadmaps, or documentation selected
  for M01C review.

Do not remove dependencies or files. Do not run cargo-machete `--fix`. Do not add
cargo-audit, cargo-semver-checks, cargo-binstall, automatic retries, or automatic
background updates.

## Acceptance evidence

M01B is complete only when:

1. Every accepted tool is installed at its exact accepted version.
2. A second installer run is a no-op and succeeds.
3. The non-mutating checker and all focused checker tests pass.
4. rust-analyzer is owned by the exact Rust 1.97.1 toolchain.
5. OpenCode config enables LSP and the opt-in launcher preserves environment
   values.
6. Nextest runs the supported complete test graph with no retries and no failure.
7. Ordinary Cargo still passes the full authoritative test graph.
8. Cargo-deny licence, bans, sources, and advisories checks pass without ignores.
9. Cargo-machete findings are recorded without applying a fix.
10. Cargo.lock remains byte-identical.
11. No dependency, source, test, edition, compiler channel, OCaml, or Tethers
    behaviour changes.
12. Packet checker, Rustfmt, Just verification, and `git diff --check` pass.

## Editing recovery discipline

After an exact `oldString` replacement failure:

1. do not repeat the identical edit;
2. reread the current file;
3. use a fresh smaller patch against the latest contents;
4. stop after two materially different failed attempts rather than rewriting a
   file wholesale.
