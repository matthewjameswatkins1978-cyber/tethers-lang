# Current Implementation Task

Control contract: `1`
Task: `TETHERS-0.3-P1-R1E — Synthetic Unrelated Plug Proof`
Owner: `OpenCode`
Status: `COMPLETE`
Task colour: `Amber`
Route: `OpenCode implements proof`
Worker note: `docs/worker-notes/2026-08-10-0.3-p1-r1e-synthetic-unrelated-plug.md`
Base branch: `feature/0.3-p1-r1d-reference-provider-scope`
Base commit: `530e429e80dc69a777af9708be8d6d1b917b9b22`
Implementation branch: `feature/0.3-p1-r1e-synthetic-unrelated-plug`
Implementation checkpoint: `4bfd587f05a5e60910faea8341bc65db86a3da6f`
OCaml switch path: `not applicable`
Rust toolchain: `1.97.1`
Rust change class: `GREEN_AMBER_SYNTHETIC_PLUG_PROOF`

## Objective

Prove that Tethers can carry a completely unrelated Plug and Operational Scope shape without adding subject knowledge to the generic host.

## Relevant background and existing behaviour

- Generic installed launch, package inspection, candidate/install/enablement machinery exists and is tested via the PDF reference Plug.
- `validate_and_canonicalize_operational_scope` in `validation.rs` provides generic scope validation with `x-tethers-path: "canonical-directory"` support.
- `OperationalScopeEvidence::create` in `operational_scope.rs` is plug-agnostic.
- The existing `build_reference_package` pattern in `pdf_tools.rs` shows how to construct .tetherplug packages.

## Required behaviour

Prove that a synthetic Plug (`example.text-inspector`) with capability `text.inspect@1`, operation `text_inspect`, and operational scope `{workspace (canonical-directory), limit (1-1000)}` passes through the entire generic pipeline without any production code changes.

## Frozen decisions and invariants

1. No production code changes expected.
2. Test/fixture/documentation changes only.
3. If a tiny generic bug is found, STOP and report it; do not quietly repair architecture.
4. Overall P1 remains `completion repair in progress`.
5. The synthetic names (`example.text-inspector`, `example-text-inspector-provider`, `text.inspect`, `text_inspect`, `workspace`, `limit`) must not appear in production Tethers code.

## Acceptance criteria

1. package inspection accepts `example.text-inspector`
2. inspection exposes the exact `operational_scope_schema`
3. inspection computes the exact deterministic schema digest
4. candidate evidence preserves the exact schema + digest
5. installed evidence preserves the exact schema + digest
6. enablement accepts valid scope with workspace + limit=37
7. workspace is canonicalised using the generic `canonical-directory` machinery
8. limit remains exactly 37
9. `OperationalScopeEvidence` `canonical_scope_json` contains only the canonical workspace and exact limit
10. `OperationalScopeEvidence` carries the exact installed schema digest
11. repeated creation from equivalent input produces deterministic evidence
12. negative: missing workspace rejected
13. negative: missing limit rejected
14. negative: relative workspace rejected
15. negative: nonexistent workspace rejected
16. negative: limit=0 rejected
17. negative: limit=1001 rejected
18. negative: limit wrong type rejected
19. negative: unknown scope field rejected
20. no production subject knowledge added (grep proof)
21. `cargo check --all-targets --all-features --locked` clean
22. `cargo fmt --all -- --check` clean
23. `git diff --check` clean
24. branch pushed; remote == local; worktree clean

## Relevant components

### Authorised paths

- `tethers-0.1/host-rust/tests/r1e_synthetic_unrelated_plug.rs` (new)
- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-08-10-0.3-p1-r1e-synthetic-unrelated-plug.md` (new)

## Required verification

1. New R1E synthetic Plug tests pass
2. Directly relevant generic package/candidate/install/enable regression tests pass
3. `cargo check --all-targets --all-features --locked`
4. `cargo fmt --all -- --check`
5. `git diff --check`
6. Bounded repository search proving synthetic names did not enter production code
7. `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1`

## Forbidden changes

- No production code changes
- No synthetic Plug names in production code
- No J23C2 changes
- No PDF conformance repair
- No File Tools changes
- No generic provider redesign
- No pack/inspect/conform public CLI work
- No P2, migration tooling, concurrency, or unrelated cleanup
- No full verify-agent, engine fixture suite, MCP transcript suite, or final P1 gate

## Stop conditions

- Production code needs one of the synthetic names
- A required test or check has two materially similar failed attempts
- An edit causes unrelated formatting or line-ending churn
- A tiny generic bug is found (report, don't fix)

## Expected pre-existing changes

None.
