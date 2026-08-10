# Current Implementation Task

Control contract: `1`
Task: `TETHERS-0.3-P4 — Plug Author Manual`
Owner: `OpenCode`
Status: `COMPLETE`
Task colour: `Green`
Route: `OpenCode implementation → Lucy GitHub review`
Worker note: `docs/worker-notes/2026-08-10-0.3-p4-plug-author-manual.md`
Base branch: `main`
Base commit: `e23030ad5e9820373133b25222680194af967c39`
Implementation branch: `feature/0.3-p4-plug-author-manual`
Implementation checkpoint: `8b90ce76b70b33276f6b633828cfc782064bb792`
OCaml switch path: `N/A`
Rust toolchain: `1.97.1` from root pin; no Rust source changes expected
Toolchain preflight: `pwsh -NoProfile -File scripts/check-dev-tools.ps1`
Rust change class: `DOCUMENTATION_ONLY` (no production Rust source changes)

## Objective

Write the first complete public Plug-authoring manual using only interfaces and
behaviour actually proven by P1–P3, then record small project-state cleanup and
the agreed delegation/blocking-rule workflow correction.

## Relevant background and existing behaviour

- P3 FINAL ACCEPTED at `e23030ad5e9820373133b25222680194af967c39`; final P3
  implementation correction checkpoint `fcf22bff911393869d8dd560efeee1442a50b119`.
- `reference-plugs/pdf-tools/` is the accepted reference Plug owning the
  provider, author material, and manifests.
- Public `plug pack`, `plug inspect`, and `plug conform` are accepted P2/P3
  surfaces and must be documented as-is.
- Generic Tethers owns trust, packaging, scope evidence, supervision, dispatch,
  and receipts; the Plug/provider owns application-specific meaning.

## Required behaviour

1. Create `docs/PLUG_AUTHORING.md` as the canonical public author manual,
   readable top to bottom, covering: what a Plug is; minimal authoring mental
   model; author source tree; `plug.json`; capability manifest; provider
   contract; Operational Scope Evidence; building the provider; assembling the
   temporary pack source; `plug pack`; `plug inspect`; `plug conform` (both
   paths); a complete PDF Tools walkthrough; common mistakes; and an author
   checklist.
2. Every command and field name must match the current CLI and accepted author
   format. Use the real PDF reference Plug as the concrete example.
3. Clearly separate author declarations from Tethers-generated evidence
   (author `payloads` vs generated `payload_index`; no manual hashes, sizes,
   manifest digest, or semantic package digest).
4. Describe conformance as supervised, non-isolated, non-installing and
   non-trust-creating. State the `--allow-non-isolated-supervised-execution`
   approval flag and the default approval-required refusal.
5. Explain Operational Scope Evidence ownership: generic host carries and
   validates scope evidence; the Plug/provider interprets its own scope meaning.
6. Update `docs/ROAD_TO_0_3.md`, `docs/CURRENT_GOAL.md`, and
   `docs/PROJECT_DASHBOARD.md` to record P3 FINAL ACCEPTED, P4 active, P5 next
   and not started.
7. Update `docs/AGENT_WORKFLOW.md` and `AGENTS.md` with the delegation/blocking
   rule principle.

## Relevant components

- `reference-plugs/pdf-tools/author/plug.json`
- `reference-plugs/pdf-tools/author/manifests/pdf-inspect-v1.json`
- `reference-plugs/pdf-tools/provider-rust/`
- `reference-plugs/pdf-tools/README.md`
- `tethers-0.1/host-rust/src/cli.rs` (public pack/inspect/conform arguments)
- `tethers-0.1/host-rust/tests/p3_pdf_reference_plug.rs`
- `tethers-0.1/host-rust/tests/p2a_plug_pack_cli.rs`
- `tethers-0.1/host-rust/tests/p2b_plug_conform_cli.rs`
- `justfile` (`test-pdf-reference` recipe)
- `docs/ROAD_TO_0_3.md`
- `docs/CURRENT_GOAL.md`
- `docs/PROJECT_DASHBOARD.md`
- `docs/AGENT_WORKFLOW.md`
- `AGENTS.md`

## Frozen decisions and invariants

- Manual documents only interfaces proven by P1–P3. No future APIs.
- Author declarations are separate from generated package evidence.
- Conformance is supervised but non-isolated, does not install, and does not
  create durable trust or enablement.
- No production Rust/OCaml, CLI, semantics, dependencies, or Cargo changes.

## Acceptance criteria

1. `docs/PLUG_AUTHORING.md` exists and covers the full current public author
   journey.
2. All documented commands and field names checked against current repository
   evidence.
3. The PDF reference Plug is the real example.
4. The manual clearly separates author declarations from Tethers-generated
   evidence.
5. Conformance correctly described as supervised, non-isolated, non-installing
   and non-trust-creating.
6. Operational Scope Evidence ownership explained correctly.
7. Author checklist and common-error section present.
8. Project-state docs and workflow docs updated truthfully.
9. Diff shows no production Rust/OCaml source changes.

## Required verification

1. Documented commands cross-checked against `cli.rs` and P2/P3 CLI tests.
2. Documented field names cross-checked against `plug.json`,
   `pdf-inspect-v1.json`, and pack/inspect output structures in tests.
3. `git diff --check` and full diff/status inspection.
4. Packet checker reports `control-v1/COMPLETE` on closeout.

## Formatting and checkpoint sequence

No Rust source changes are expected. If any formatting tool touches Rust source
it is out of scope and must be stopped. Documentation-only closeout.

## Completion and publication

Commit the implementation checkpoint, write the worker note at the named path,
set this packet to `COMPLETE`, require checker `control-v1/COMPLETE`, commit
docs-only closeout, then push the named branch normally and prove
`origin/feature/0.3-p4-plug-author-manual == HEAD` and a clean worktree.

## Forbidden changes

- No P5, no fresh-agent authoring proof, no Plug/package semantic changes.
- No CLI, Rust/OCaml production, Cargo dependency, or concurrency/Event
  Ingress/HQ work.
- No merge, amend, tag, force-push, PR, or direct `main` update.

## Stop conditions

- A real contradiction in the public Plug interface that cannot be resolved from
  repository evidence.
- A consequential architecture/product decision requiring external authority.
- Two materially similar implementation attempts fail on the same unresolved
  problem.

## Continuation authority

None required. This is documentation and project-control work.

## Expected pre-existing changes

None.
