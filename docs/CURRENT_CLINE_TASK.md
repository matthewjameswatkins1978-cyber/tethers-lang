# Current Implementation Task

Control contract: `1`
Task: `TETHERS-0.3-P3 — PDF Tools Reference Plug Crucible`
Owner: `Codex`
Status: `COMPLETE`
Task colour: `Red`
Route: `Codex implementation → Lucy independent GitHub review/acceptance`
Worker note: `docs/worker-notes/2026-08-10-0.3-p3-pdf-reference-plug.md`
Base branch: `main`
Base commit: `06bcb29d36522f0b75bd24eac7c4b66e49f8ea33`
Implementation branch: `feature/0.3-p3-pdf-reference-plug`
Implementation checkpoint: `907eaa75b17a4441806df342af30fd5ffd9c8ea7`
OCaml switch path: `N/A`
Rust toolchain: `1.97.1` from root pin; plain Cargo; `--locked` mandatory
Toolchain preflight: `pwsh -NoProfile -File scripts/check-dev-tools.ps1`
Rust change class: `RUST_CHANGING`

## Objective

Move the real PDF Tools capability, provider, manifest authority, and PDF tests
out of the generic host into an independently built reference Plug, then prove
the public pack → inspect → conform journey and real installed execution through
generic Tethers machinery.

## Relevant background and existing behaviour

- P1 and P2 are FINAL ACCEPTED and must not be reopened without contradictory
  evidence.
- The generic host already owns packages, Operational Scope Evidence,
  supervision, MCP transport, discovery comparison, trust, installation,
  enablement, CapabilityExecutor, result, and Trail machinery.
- `pdf_tools.rs`, `pdf_tools_provider.rs`, `PdfOperationalScopeBinding`, and
  `InstalledPdfToolsExecutor` are the old host-owned PDF seam being replaced.
- Public `plug pack`, `plug inspect`, and `plug conform` are accepted P2
  surfaces and must be used as-is.

## Required behaviour

1. Create `reference-plugs/pdf-tools/provider-rust/` as a standalone Rust
   package that contains the observable PDF inspection semantics and its useful
   unit tests, has no `tethers-reference-host` dependency, and preserves
   `pdf.inspect@1`, `tethers-pdf-provider` `1.0.0`, `pdf_inspect`, the 64 MiB
   bound, bounded page scan, header/version, page count, SHA-256, file size,
   `is_pdf`, containment, scope enforcement, and observable errors.
2. Move the MCP stdio provider process to that package, interpreting only
   `TETHERS_OPERATIONAL_SCOPE_JSON`, `TETHERS_OPERATIONAL_SCOPE_DIGEST`,
   `TETHERS_CONFORMANCE`, and `TEMP`; do not add marker-only production
   behaviour or retain obsolete `PdfOperationalScopeBinding`.
3. Create the real reference author tree:
   `reference-plugs/pdf-tools/author/plug.json` and
   `reference-plugs/pdf-tools/author/manifests/pdf-inspect-v1.json`, using
   author `payloads` (not generated `payload_index`) and no manually authored
   payload hashes, sizes, manifest digest, or semantic package digest. Preserve
   package/provider/capability/operation/launch/working-directory/namespace and
   `query_root`, `max_bytes 1..67108864` semantics.
4. Remove the old generic-tree PDF manifest, host `pdf_tools.rs`, host provider
   binary, host module export, PDF-specific installed executor, and every
   remaining production PDF-family reference from `host-rust/src`; preserve the
   accepted manifest digest
   `sha256:26da081128608859c1259da7ddd784d343241504cb47339ca54a9b5979b6297c`
   unless an explicitly understood semantic correction requires a stop.
5. Add the smallest generic path: wrap an already supervised installed child in
   `ManagedProvider`, admit it to the normal retained MCP session, and add one
   generic installed-provider `CapabilityExecutor` that launches, initializes,
   discovers once, compares trusted `VerifiedManifest` evidence, dispatches the
   trusted operation using the remaining deadline, and preserves known failure
   versus uncertainty classification. It must contain no PDF branch/string and
   must not duplicate MCP JSON-RPC implementation.
6. Replace only generic lifecycle tests that import `pdf_tools` or call
   `build_reference_package` with one existing neutral fixture or one new shared
   neutral fixture helper, preserving the existing lifecycle assertions.
7. Replace the useful old split PDF integration proofs with one ignored
   `tests/p3_pdf_reference_plug.rs` crucible. It accepts
   `TETHERS_PDF_REFERENCE_PROVIDER_EXE`, creates a temporary source from the
   committed author material, copies the independently built provider to
   `provider/pdf_tools_provider.exe`, and proves public pack/inspect/conform,
   required approval refusal, approved non-isolated conformance, output hygiene,
   source/provider/package immutability, manifest compatibility, and semantic
   digest continuity.
8. The same crucible must use the real package through generic
   inspect/stage/trust/conformance/install/enable/launch/executor machinery with
   generic Operational Scope Evidence, invoke `pdf.inspect` on a real small PDF
   inside its approved root, and prove the returned real PDF facts.
9. Add one durable `test-pdf-reference` Just recipe that fmt-checks, tests, and
   builds the standalone provider, sets its executable environment variable,
   and explicitly runs the ignored crucible; include it in `verify-agent`.
10. Do not change host `Cargo.toml` or `Cargo.lock`, do not create a repository
    workspace, and keep the provider dependencies to the small required set.
11. Complete the focused checks, commit the implementation checkpoint, run
    `just verify-agent` exactly once against it, then run all final search gates
    and complete the named worker note and project control documents.
12. Push normally without a PR, merge, force-push, tag, or direct `main`
    update; resolve and compare the remote branch head, then leave a clean tree.

## Relevant components

- `reference-plugs/pdf-tools/`
- `tethers-0.1/host-rust/src/pdf_tools.rs`
- `tethers-0.1/host-rust/src/bin/pdf_tools_provider.rs`
- `tethers-0.1/host-rust/src/stdio_provider.rs`
- `tethers-0.1/host-rust/src/child_process.rs`
- `tethers-0.1/host-rust/src/installed.rs`
- `tethers-0.1/host-rust/tests/j23a_pdf_provider.rs`
- `tethers-0.1/host-rust/tests/j23b_pdf_package.rs`
- `tethers-0.1/host-rust/tests/j23c2_pdf_conformance.rs`
- `tethers-0.1/host-rust/tests/j23c3_installed_pdf_execution.rs`
- `tethers-0.1/protocol/capability-manifests/pdf-inspect-v1.json`
- `justfile`
- `docs/ROAD_TO_0_3.md`
- `docs/CURRENT_GOAL.md`
- `docs/PROJECT_DASHBOARD.md`

## Frozen decisions and invariants

- PDF-specific meaning lives in the reference Plug; generic Tethers only hosts
  it. The final ownership boundary is `reference-plugs/pdf-tools/`.
- Retain the current generic operational-scope environment contract and existing
  P1/P2 public semantics.
- Reuse `SupervisedChild`, normal MCP session/discovery/comparison, and the one
  accepted execution seam. No second supervisor, MCP implementation, replay,
  permission, or Trail route.
- Preserve serial execution, deadlines, explicit-provider-error classification,
  uncertainty classification, trust, and output redaction boundaries.
- P4, Event Ingress, File Tools, and concurrency are outside this packet.

## Acceptance criteria

1. Standalone provider check/tests demonstrate retained PDF semantics and no
   production host dependency.
2. Provider MCP tests demonstrate the current environment contract and protocol
   without a test-only marker effect.
3. Author files contain only declarations and public packing creates generated
   package evidence with the frozen identities and scope schema.
4. Final source search is empty for PDF production references and the generated
   manifest remains semantically compatible at the frozen digest.
5. Focused generic-executor tests demonstrate supervised generic launch, normal
   retained session/discovery validation, trusted dynamic dispatch, deadlines,
   and known-failure/uncertain distinction without PDF code.
6. Initially identified generic tests no longer import or build PDF fixtures;
   their original behavioural assertions pass with neutral fixtures.
7. The P3 crucible explicitly passes public pack/inspect/conform, approval
   refusal (`exit 5`, `approval_required`,
   `conformance_execution_approval_required`), approved conformance, digest
   continuity, immutability, and output hygiene.
8. The same crucible passes real installed generic execution and asserts a real
   `pdf.inspect` response with expected PDF facts.
9. `just test-pdf-reference` passes and `just verify-agent` invokes it.
10. Diff inspection proves host `Cargo.toml` and `Cargo.lock` are unchanged and
    no repository Cargo workspace was added.
11. Focused checks, one final `just verify-agent`, complete diff/status checks,
    and all three final search gates are recorded truthfully.
12. Packet checker reports `control-v1/COMPLETE`; normal push proves local and
    remote branch heads match with a clean worktree.

## Required verification

1. One initial `rg` inventory for the packet-named PDF/executor symbols and
   generic test imports/builders; record it in the worker note.
2. Standalone provider `cargo fmt -- --check`, `cargo check --locked`, focused
   tests, and build.
3. Host warnings-denied focused compile, generic executor focused tests, and
   only affected changed test binaries during development.
4. Before checkpoint, run `cargo fmt --all` for authorised host Rust paths and
   provider `cargo fmt`, inspect the immediate formatter diff, and stop on
   unrelated formatting.
5. Explicit ignored P3 crucible with the independently built provider.
6. `git diff --check`, complete diff inspection, and final status inspection.
7. Commit implementation; capture `git rev-parse HEAD`; run `just verify-agent`
   once against that exact checkpoint.
8. Final searches for host PDF purge, provider host dependency, and old generic
   test builder/import are empty; then update packet, worker note, roadmap,
   current goal, and dashboard, re-run the packet checker, and commit docs-only
   closeout without rerunning the expensive verification.

## Formatting and checkpoint sequence

Authorised Rust paths are the standalone provider and the host modules/tests
strictly required by this packet. Run `cargo fmt --all` only after the focused
implementation is ready, inspect its immediate diff, and stop if it changes
unrelated paths. Run provider `cargo fmt` separately from its package root.

## Completion and publication

Commit the implementation checkpoint before the sole `just verify-agent` run.
After its evidence is recorded, set this packet to `COMPLETE`, use the exact
checkpoint SHA in this packet and worker note, require checker
`control-v1/COMPLETE`, commit docs-only closeout, then push the named branch
normally and prove `origin/feature/0.3-p3-pdf-reference-plug == HEAD` and a
clean worktree.

## Forbidden changes

- No P4, Event Ingress, File Tools, or concurrency work.
- No host `Cargo.toml` or `Cargo.lock` change; no repository Cargo workspace.
- No host PDF family production logic, PDF-specific generic executor branch, or
  duplicate MCP/supervision/dispatch path.
- No redesign of `plug pack`, `plug inspect`, `plug conform`, P1, or P2.
- No merge, amend, tag, force-push, PR, or direct `main` update.

## Stop conditions

- A standalone provider production dependency on `tethers-reference-host`.
- A need for a generic-host PDF branch, semantic redesign, changed P1/P2
  behaviour, unexplained manifest-digest change, host dependency change, or
  duplicated MCP protocol.
- Scope expansion into File Tools, Event Ingress, concurrency, or P4.
- Two materially similar implementation attempts fail.

## Continuation authority

Lucy has authorised a bounded continuation in this existing dirty worktree to
finish the neutral-fixture migration. Repeated stale PDF identity assertions in
generic lifecycle tests are one mechanical migration pass, not separate failed
implementation attempts. The repeated-failure stop rule now applies only when
the same corrected assertion continues to fail after two materially similar
fixes, or a failure reveals a genuine architectural or semantic contradiction.

## Expected pre-existing changes

None.
