# Current Goal

Updated: 2026-08-04

## Goal

Complete the repository spring clean in bounded maintenance increments before
resuming the Plug-installation sequence at J24J.

```text
M01A  accepted Rust 1.97.1 toolchain refresh
M01B  Rust agent tooling foundation
M01C  warning cleanup and evidence-backed repository pruning
J24J  read-only installation reconciliation planner
```

## Accepted maintenance baseline

M01A is accepted at
`d561b8400a1398c3d5bdde2cf670eebe661a5cc4`.

The active repository baseline is now:

```text
Rust             1.97.1 exact root pin
Rust edition     2021
rust-version     1.97
OCaml            5.5.0
Dune             3.24.0
Yojson           2.2.2
Cargo.lock       committed and unchanged by M01A
```

Plain Cargo commands inherit the root pin. Just recipes fail on their first
failed command. The toolchain checker derives Rust and Cargo truth from
repository files instead of carrying copied version constants.

## Active maintenance increment

M01B adds and proves the small Rust toolset chosen to improve OpenCode
implementation and review:

```text
rust-analyzer   Rust 1.97.1 component
cargo-nextest   0.9.137
cargo-deny      0.19.7
cargo-machete   0.9.2
```

Their roles remain separate:

- rust-analyzer assists navigation and diagnostics;
- nextest provides an alternative no-retry agent test loop;
- cargo-deny owns dependency licence, source, duplicate, and advisory policy;
- cargo-machete suggests possible unused dependencies but has no deletion
  authority.

Cargo-audit is not added because cargo-deny supplies the accepted advisory gate.
Cargo-semver-checks remains deferred until Tethers promises compatibility for a
public Rust library API.

## M01B boundaries

- Exact tool versions are repository-owned and checked.
- Rust-analyzer belongs to the exact Rust 1.97.1 toolchain.
- OpenCode LSP is explicitly enabled and its direct query tool is opt-in and
  process-local.
- OpenCode may not download a second language server when using the repository
  launcher.
- Nextest retries remain zero and ordinary `cargo test` remains final authority.
- Native Windows nextest performance is measured rather than assumed.
- Cargo-deny receives no hidden advisory ignore or autonomous licence expansion.
- Cargo-machete never runs with `--fix`.
- Installation is exact, bounded, idempotent, and separate from verification.
- Cargo.toml, Cargo.lock, dependencies, production source/tests, OCaml, edition,
  Rust channel, and Tethers behaviour remain unchanged.
- M01B records cleanup evidence but deletes nothing.

## Next maintenance increment

M01C will use accepted M01B evidence to review:

- existing Rust warnings;
- cargo-machete findings;
- inactive `.clinerules` and `.clineignore` configuration;
- duplicated environment/tool checks;
- stale active guidance;
- obsolete one-off scripts and roadmaps;
- files that can be removed with direct reference evidence.

Historical worker notes, completed packets, releases, and architectural evidence
remain historical records and are not rewritten to appear current.

## Accepted product baseline

Tethers 0.2.0 remains the accepted and published baseline. The annotated
`v0.2.0` tag remains at
`b5546411661dcbcb53e1cf2538eaec594c6f76f2`; Tethers language semantics remain
0.1.

The accepted public Plug surface remains:

- `plug inspect`;
- `plug list`;
- `plug disable`;
- permission-file `plug enable`;
- `plug stage`.

J24G provides the strict installation request. J24H provides durable launch
profile evidence and non-creating store openings. J24I provides exact-candidate
trust and is accepted at
`88d8ab2e5c65052401b3860d8a7d68f3ccb06265`.

After maintenance, the reviewed installation sequence resumes as:

```text
J24J  read-only installation reconciliation planner
J24K  host installation lock and gate executor
J24L  thin public plug install CLI
```

## Frozen installation shape

Maintenance does not alter the installation design:

```text
validated installation request
→ exact candidate validation
→ exact-candidate trust
→ durable supervised launch profile
→ supervised conformance
→ installation approval
→ atomic installed publication
→ present_disabled
```

## Active development posture

Current operating mode: **Gorilla Coding**.

- Lucy: architecture, packet compilation, independent review, and routine safe
  merges.
- OpenCode: implementation programme.
- Luna: bounded Green and ordinary Amber implementation.
- HY3: mechanical, repetitive, low-risk implementation.
- DeepSeek Pro V4: thicker cross-file work under frozen contracts.
- Matthew: product authority, ideas, priorities, and human judgement.
- Cline and Goose are not used.
- Active prototype tree: `tethers-0.1/`.
- Current accepted Rust toolchain: exact 1.97.1.
- Required automation shell where applicable: PowerShell 7.

DeepSeek editing rule: after an exact `oldString` replacement failure, reread
the current file and create a fresh smaller patch. Never repeat the identical
failed edit, and stop after two materially different failed attempts rather than
rewriting a file wholesale.

## Authoritative references

- Current maintenance task: `docs/CURRENT_CLINE_TASK.md`
- M01B blueprint:
  `docs/architecture/M01B_RUST_AGENT_TOOLING_FOUNDATION.md`
- Toolchain policy: `docs/TOOLCHAIN_POLICY.md`
- Current root Rust pin: `rust-toolchain.toml`
- OpenCode project configuration: `opencode.json`
- Rust engineering guidance: `docs/RUST_ENGINEERING_GUIDE_FOR_AGENTS.md`
- Enduring principles: `docs/CONSTITUTION.md`
- Language semantics: `tethers-0.1/SPEC.md`
- Universal Plug architecture:
  `docs/architecture/TETHERS_UNIVERSAL_PLUG_ARCHITECTURE.md`
- J24I exact-candidate trust:
  `docs/architecture/J24I_EXACT_CANDIDATE_INSTALLATION_TRUST.md`
- Accepted decisions: `docs/DECISIONS.md`
- Short project status: `docs/PROJECT_DASHBOARD.md`
- Detailed queue: `docs/TASK_QUEUE.md`
- Historical evidence: `docs/worker-notes/`
