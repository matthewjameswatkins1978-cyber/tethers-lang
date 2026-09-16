# Worker Note

- **Task Packet:** `TETHERS L1 IMPLEMENTATION PACKET — Native Linux Test-Portability Repair`
- **Owner:** `Codex`
- **Status:** `COMPLETE`
- **Base Commit:** `b0e10dfe1b2834e974a384cd14fc9c43c0460b5b`
- **Final Commit:** `dcec282abc6b93b9cc4aae8c334d867533bb51d1`
- **Branch / Worktree:** `feature/tethers-l1-linux-portability` / `/home/matmus/biscuit-linux/projects/tethers`

## Files Modified

- `tethers-0.1/host-rust/src/application.rs`
- `tethers-0.1/host-rust/src/p4b_lifecycle.rs`
- `tethers-0.1/host-rust/src/host_execution.rs`
- `tethers-0.1/host-rust/src/openshell_executor.rs`
- `docs/worker-notes/2026-09-16-tethers-l1-linux-portability.md`

## Behavioural Result

The nested P4b test module now resolves `p4b_lifecycle.rs` through `include!`,
relative to `application.rs`, instead of relying on the non-existent synthetic
`src/application/tests` directory. The P4b tests remain in their original
application test scope and their expectations are unchanged.

Existing tests that directly require the Windows replay backend and
`pwsh.exe` are explicitly Windows-only. They remain active on Windows and no
Linux test was weakened to hide the P4b portability defect.

## Invariants Preserved

- No Tethers semantics, Rust/OCaml API, protocol, or runtime behaviour changed.
- No provider, replay, policy, Trail, or execution boundary changed.
- The P4b test fixture still exercises the real guarded serial boundary.
- Windows-only replay/OpenShell tests remain compiled on Windows.
- Existing duplicate-target and dead-code warnings remain unchanged debt.

## Negative Tests Added or Updated

- No test expectations were changed.
- Native Linux workspace compilation now reaches and collects the existing P4b
  tests instead of failing at module path resolution.

## Commands Executed

- `cargo fmt --all -- --check` — PASS
- `cargo check --workspace` — PASS
- `cargo test --workspace --no-run` — PASS; all workspace test targets compiled
- `cargo test --lib p4b_lifecycle` — PASS; 6 passed, 0 failed
- `bl fast tethers` — PASS; 3.73s and 3.39s
- `bl verify tethers` — PASS; 0.80s; provenance reported final commit
- `bl doctor tethers` — PASS; mandatory readiness PASS
- `git diff --check` — PASS

## Unrun Checks and Reason

- `pwsh -NoProfile -File scripts/check-dev-tools.ps1` — NOT RUN: native WSL
  has no `pwsh`; Windows-side tooling was intentionally not invoked.
- `just test-rust` — NOT RUN: repository recipe invokes PowerShell, while the
  native WSL packet requires Linux-native commands.

## Discoveries

- Before the fix, `cargo test --lib` failed with:
  `couldn't read src/application/tests/../../p4b_lifecycle.rs: No such file or directory`
  at `src/application.rs:3648`.
- After the path fix, raw full test execution completed but reported
  `1258 passed; 245 failed; 4 ignored`; failures are existing Windows/PowerShell,
  engine-provenance, and Linux path/resource assumptions outside this bounded
  module-portability repair. Compilation itself passed.
- The repository root has no Cargo manifest; Rust commands run from
  `tethers-0.1/host-rust` as required by the project layout.

## Remaining Risks

- Full native Linux test execution remains a separate follow-up problem despite
  full test-target compilation succeeding. The exact raw command was
  `cargo test --workspace`; its terminal result was `FAILED. 1258 passed; 245 failed; 4 ignored`.

## Recommended Next Action

Route a separate, explicitly scoped Linux runtime-test portability packet for
the remaining platform and environment failures. Do not expand this module
path repair into that work.
