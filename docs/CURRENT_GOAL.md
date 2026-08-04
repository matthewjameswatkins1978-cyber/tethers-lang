# Current Goal

Updated: 2026-08-04

## Goal

Complete the repository spring clean in bounded maintenance increments before resuming the Plug-installation sequence at J24J.

```text
M01A   accepted Rust 1.97.1 toolchain refresh
M01B   accepted Rust agent tooling foundation
M01C1  engine-session warning cleanup pilot
M01C2  remaining warning clusters
M01C3  evidence-backed documentation and file pruning
J24J   read-only installation reconciliation planner
```

## Accepted maintenance baseline

M01A is accepted at `d561b8400a1398c3d5bdde2cf670eebe661a5cc4`.

M01B is accepted at `f7e84a467bf77a02f1f1b60cd319c55644dd9bbd`.

The active baseline is:

```text
Rust             1.97.1 exact root pin
Rust edition     2021
rust-version     1.97
rust-analyzer    Rust 1.97.1 component
cargo-nextest    0.9.140, retries 0
cargo-deny       0.19.7
cargo-machete    0.9.2
Cargo tests      926 passing
Nextest tests    1133 passing
OCaml            5.5.0
Dune             3.24.0
Yojson           2.2.2
Cargo.lock       committed
```

OpenCode’s real console CLI has proved the repository LSP configuration. Agent tooling fails closed when the executable or effective LSP permission is missing.

## Active maintenance increment

M01C1 is a deliberately small trial of the new tools against warnings in:

`tethers-0.1/host-rust/src/engine_stdio.rs`

The trial must demonstrate:

- rust-analyzer/OpenCode LSP reference discovery before editing;
- machine-readable warning accounting before and after;
- focused Nextest feedback with zero retries;
- ordinary Cargo as final test authority;
- Cargo-deny policy gates;
- cargo-machete advisory evidence;
- no warning suppression, dependency drift, protocol change or behavioural redesign.

The retained engine read timeout must become real authority while remaining exactly ten seconds. If accepted Clippy reports path-reference linting around `EngineSession::launch`, only LSP-proven direct call sites may change.

## M01C1 boundaries

- Target only warnings whose primary span is `src/engine_stdio.rs`.
- Do not repair other warning clusters in the same job.
- Do not change Cargo dependencies, lockfile, tool versions or configuration.
- Do not change OCaml, MCP protocol, CLI behaviour, Plug installation, concurrency or retry policy.
- Preserve 926 Cargo tests and 1133 Nextest tests as the minimum complete-suite floor.
- Record the usefulness of each new tool rather than assuming it helped.

## Later spring-clean work

After the pilot is accepted:

- M01C2 handles remaining warning clusters in separate coherent slices.
- M01C3 reviews inactive `.clinerules` and `.clineignore`, duplicated checks, stale live guidance, one-off scripts, obsolete roadmaps and safe deletions.
- Historical worker notes, completed packets, release records and architecture evidence remain historical and are not rewritten to appear current.

The existing `event_queue.rs` comment-only Send test is explicitly not part of M01C1 because correcting its warning would require an architectural decision about the queue’s actual thread-transfer contract.

## Accepted product baseline

Tethers 0.2.0 remains the accepted and published baseline. The annotated `v0.2.0` tag remains at `b5546411661dcbcb53e1cf2538eaec594c6f76f2`; Tethers language semantics remain 0.1.

The accepted public Plug surface remains:

- `plug inspect`;
- `plug list`;
- `plug disable`;
- permission-file `plug enable`;
- `plug stage`.

J24G provides the strict installation request. J24H provides durable launch-profile evidence and non-creating store openings. J24I provides exact-candidate trust and is accepted at `88d8ab2e5c65052401b3860d8a7d68f3ccb06265`.

After maintenance:

```text
J24J  read-only installation reconciliation planner
J24K  host installation lock and gate executor
J24L  thin public plug install CLI
```

## Active development posture

Current operating mode: **Gorilla Coding**.

- Lucy: architecture, packet compilation, independent review and routine safe merges.
- OpenCode: implementation programme.
- DeepSeek Pro V4: M01C1 implementation under the frozen warning contract.
- Matthew: product authority, ideas, priorities and human judgement.
- Active prototype tree: `tethers-0.1/`.
- Required automation shell where applicable: PowerShell 7.

DeepSeek editing rule: after an exact replacement failure, reread the current file and make a fresh smaller patch. Never repeat the identical failed edit; stop after two materially different failed attempts rather than rewriting a file wholesale.

## Authoritative references

- Current task: `docs/CURRENT_CLINE_TASK.md`
- M01C1 blueprint: `docs/architecture/M01C1_ENGINE_SESSION_WARNING_PILOT.md`
- M01B tooling blueprint: `docs/architecture/M01B_RUST_AGENT_TOOLING_FOUNDATION.md`
- Toolchain policy: `docs/TOOLCHAIN_POLICY.md`
- Rust engineering guidance: `docs/RUST_ENGINEERING_GUIDE_FOR_AGENTS.md`
- Enduring principles: `docs/CONSTITUTION.md`
- Language semantics: `tethers-0.1/SPEC.md`
- Universal Plug architecture: `docs/architecture/TETHERS_UNIVERSAL_PLUG_ARCHITECTURE.md`
- J24I exact-candidate trust: `docs/architecture/J24I_EXACT_CANDIDATE_INSTALLATION_TRUST.md`
- Historical evidence: `docs/worker-notes/`
