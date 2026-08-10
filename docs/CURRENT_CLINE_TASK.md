# Current Implementation Task

Control contract: `1`
Task: `TETHERS-0.3-P5 — Fresh-Agent Plug Authoring Proof`
Owner: `OpenCode`
Status: `IN_PROGRESS`
Task colour: `Green`
Route: `fresh-agent execution → OpenCode evidence capture → Lucy independent review`
Worker note: `docs/worker-notes/2026-08-10-0.3-p5-fresh-agent-authoring-proof.md`
Base branch: `main`
Base commit: `1e1f9b8738a48f727187316dd0078b7f9435f1c6`
Implementation branch: `feature/0.3-p5-fresh-agent-authoring-proof`
Implementation checkpoint: `WORKTREE`
OCaml switch path: `N/A`
Rust toolchain: `1.97.1` from root pin; provider built under `reference-plugs/`
Toolchain preflight: `pwsh -NoProfile -File scripts/check-dev-tools.ps1`
Rust change class: `REFERENCE_PROVIDER_RUST` (new provider crate under
`reference-plugs/`; no production host source changes)

## Objective

Prove that a fresh agent, using `docs/PLUG_AUTHORING.md` as its only authoring
guide, can build a new non-PDF Plug (Text Stats) from scratch and complete the
public pack → inspect → conform author journey without relying on hidden Tethers
knowledge. Capture honest evidence, record any manual gaps, and fix only narrow
manual deficiencies the proof reveals.

## Relevant background and existing behaviour

- P4 FINAL ACCEPTED at `1e1f9b8738a48f727187316dd0078b7f9435f1c6`; canonical
  public author manual is `docs/PLUG_AUTHORING.md`.
- `reference-plugs/pdf-tools/` is the accepted reference Plug owning the
  provider, author material, and manifests.
- Public `plug pack`, `plug inspect`, and `plug conform` are accepted P2/P3
  surfaces and must be used exactly as documented.
- The host conformance suite (`tethers-0.1/host-rust/src/conformance.rs`) drives
  an MCP stdio provider through `initialize`, `notifications/initialized`,
  `tools/list`, and (for `fixture*` operations only) `tools/call`; non-fixture
  operations are discovered and schema-checked without fixture calls.
- During approved conformance the host launches the provider with
  `TETHERS_CONFORMANCE=1`, a scratch `TEMP`/`TMP`, `SystemRoot`, and `WINDIR`.
  `TETHERS_OPERATIONAL_SCOPE_JSON` is present in installed execution only; the
  PDF provider falls back to `TEMP` as a safe root when it is absent during
  conformance.
- Generic Tethers owns trust, packaging, scope evidence, supervision, dispatch,
  and receipts; the Plug/provider owns application-specific meaning.

## Required behaviour

1. Create `feature/0.3-p5-fresh-agent-authoring-proof` based on the exact P4
   accepted HEAD `1e1f9b8738a48f727187316dd0078b7f9435f1c6`.
2. Update `docs/CURRENT_CLINE_TASK.md` to the P5 packet with Status
   `IN_PROGRESS` and run the packet checker.
3. Run a fresh-author experiment: a fresh agent/session whose only authoring
   guide is `docs/PLUG_AUTHORING.md` plus the short challenge prompt, builds a
   new Plug called Text Stats under `reference-plugs/text-stats-proof/`
   (provider-rust + author source + README), using only the manual, the public
   CLI, and files the manual explicitly references.
4. The fresh author must not be directed to P1/P2/P3 worker notes, P2/P3 test
   implementations, `docs/CURRENT_CLINE_TASK.md`, internal host Rust source,
   old PDF implementation code, or fixture builders. Any voluntary reach into
   undocumented internal material is recorded as a manual-quality finding.
5. The Text Stats provider must implement the required semantics: relative
   `path`; exact `size_bytes`; `sha256:<64 hex>`; logical `line_count`;
   whitespace-separated `word_count`; `character_count` after valid UTF-8
   decoding; scope keys `query_root` and `max_bytes`; path must stay inside
   `query_root`; regular file required; malformed UTF-8 fails cleanly;
   `max_bytes` honoured with an 8 MiB hard maximum; read-only; no network;
   no writes beyond ordinary diagnostics; no hidden/test-only behaviour.
6. The fresh author must successfully run the public journey in order: build
   provider → assemble pack source → `plug pack` → `plug inspect` → `plug
   conform` without approval (observe approval-required refusal) → `plug conform
   --allow-non-isolated-supervised-execution` (passed, non-isolated).
7. The provider's own semantics must be tested (valid UTF-8 stats; traversal /
   outside-root refusal; oversized refusal; malformed UTF-8 refusal; scope above
   the 8 MiB maximum refusal; unknown/missing arguments refusal; stdout stays
   MCP-protocol-only), without a test-only copy of production decision logic.
8. If the proof reveals a narrow manual deficiency, fix
   `docs/PLUG_AUTHORING.md` in this branch, record exactly what the fresh agent
   could not infer and the wording that fixed it, and rerun only the affected
   step. Do not hide the deficiency.
9. Create `docs/p5-fresh-agent-proof.md` recording the model
   (`DeepSeek V4 Flash`, Thinking ON, Effort High), the fresh prompt, author
   sources made available, prohibited sources, clarifications, manual gaps and
   corrections, pack/inspect/conform results, provider semantic-test results,
   and the final conclusion — without pasting the whole conversation.
10. Update `docs/ROAD_TO_0_3.md`, `docs/CURRENT_GOAL.md`, and
    `docs/PROJECT_DASHBOARD.md` to show P4 FINAL ACCEPTED at
    `1e1f9b8738a48f727187316dd0078b7f9435f1c6`, P5 complete awaiting Lucy
    review, P6 next and NOT started.
11. Close out per project control: implementation checkpoint commit, worker
    note, packet COMPLETE, checker `control-v1/COMPLETE`, docs closeout commit,
    normal push, remote == local HEAD proof, clean worktree.

## Relevant components

- `docs/PLUG_AUTHORING.md`
- `reference-plugs/pdf-tools/` (reference Plug; NOT an authoring guide for the
  fresh author)
- `tethers-0.1/host-rust/src/cli.rs` (public pack/inspect/conform arguments)
- `tethers-0.1/host-rust/src/conformance.rs` (host conformance suite contract)
- `tethers-0.1/host-rust/src/launch_profile.rs` (approved conformance env)
- `tethers-0.1/host-rust/target/debug/tethers-reference-host.exe` (public CLI)
- `justfile` (`test-pdf-reference` recipe as reference-only)
- `docs/p5-fresh-agent-proof.md` (new experiment log)
- `docs/ROAD_TO_0_3.md`, `docs/CURRENT_GOAL.md`, `docs/PROJECT_DASHBOARD.md`
- `docs/worker-notes/2026-08-10-0.3-p5-fresh-agent-authoring-proof.md` (new)

## Frozen decisions and invariants

- The manual is the sole authoring guide for the fresh author.
- Text Stats package `tethers.text-stats` / provider `tethers-text-stats-provider`
  / capability `text.stats@1` / operation `text_stats`; versions `1.0.0`.
- `max_bytes` hard maximum is 8 MiB.
- Conformance is supervised, non-isolated, non-installing, non-trust-creating.
- Author declarations (`payloads`) remain separate from generated evidence
  (`payload_index`, digests, sizes).
- No production host Rust/OCaml, CLI, semantics, dependencies, or Cargo changes.
- No P6 work and no redesign of Plug/manifest/MCP/scope/trust semantics.

## Acceptance criteria

1. A fresh DeepSeek V4 Flash / High-thinking author was used with
   `docs/PLUG_AUTHORING.md` as its primary authoring guide.
2. It created a new non-PDF Plug `tethers.text-stats` under
   `reference-plugs/text-stats-proof/`.
3. The provider implements the required Text Stats semantics and its semantic
   tests pass.
4. Public `plug pack` and `plug inspect` pass with correct identities,
   capability, and generated evidence.
5. Default `plug conform` correctly refuses execution
   (`approval_required` / `conformance_execution_approval_required`).
6. Approved non-isolated `plug conform` passes (`passed`, `isolated: false`,
   non-isolation limitation present).
7. Digest continuity and source/package immutability are proven.
8. Any required manual clarification is documented honestly in
   `docs/p5-fresh-agent-proof.md`; genuine gaps were fixed in
   `docs/PLUG_AUTHORING.md` with narrow wording.
9. No hidden reliance on P2/P3 tests, worker notes, internal host source, or PDF
   implementation was used as the authoring guide.
10. `docs/p5-fresh-agent-proof.md` records the experiment.
11. Project docs show P5 complete awaiting Lucy review and P6 next/not started.
12. Packet checker reports `control-v1/COMPLETE` on closeout.
13. Branch pushed normally; remote HEAD == local HEAD; worktree clean.

## Required verification

1. Packet checker at start (`IN_PROGRESS`) and on closeout
   (`control-v1/COMPLETE`).
2. Fresh-author provider formatter/check/build and semantic tests.
3. Real `plug pack` proof with captured envelope.
4. Real `plug inspect` proof with captured envelope and digest match.
5. Real `plug conform` (no approval) refusal proof (exit 5, exact codes).
6. Real approved `plug conform` proof (exit 0, disposition `passed`,
   `isolated: false`, limitation present, digest continuity).
7. Source/package immutability byte checks around pack and inspect.
8. `git diff --check` and complete diff/status inspection.
9. Host warnings-denied check ONLY if host production Rust changes unexpectedly
   (none are expected).

## Formatting and checkpoint sequence

The only Rust source introduced lives under
`reference-plugs/text-stats-proof/provider-rust/`. Format that crate with
`cargo fmt --manifest-path <provider Cargo.toml> -- --check` (and, if the fresh
author chooses, `cargo fmt` on that crate only). No formatting tool may touch
production host Rust source. The implementation checkpoint precedes all worker
note, packet, and dashboard closeout edits.

## Completion and publication

Commit the implementation/proof checkpoint, write the worker note at the named
path, set this packet to `COMPLETE`, require checker `control-v1/COMPLETE`,
commit docs-only closeout, then push the named branch normally and prove
`origin/feature/0.3-p5-fresh-agent-authoring-proof == HEAD` and a clean
worktree. Do not start P6.

## Forbidden changes

- No P6, no HQ, no concurrency, no Event Ingress, no adversarial-provider work.
- No redesign of Plug format, manifests, MCP, Operational Scope Evidence, or
  trust/install/enable semantics.
- No CLI, host Rust/OCaml production, Cargo dependency, or conformance semantic
  changes.
- No merge, amend, tag, force-push, PR, or direct `main` update.

## Stop conditions

- A real contradiction between the P4 manual and the public Plug interface that
  cannot be resolved from repository evidence.
- A consequential architecture/product decision requiring external authority.
- Two materially similar implementation attempts fail on the same unresolved
  problem.

## Expected pre-existing changes

None. Base commit is the accepted P4 HEAD; the P5 branch starts clean at it.
