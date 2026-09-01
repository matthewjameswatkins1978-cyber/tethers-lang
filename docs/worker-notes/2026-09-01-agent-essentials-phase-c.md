# Tethers Agent Essentials — Phase C Worker Note

Task: `Phase C coding essentials`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Lucy`

Status: `IN_PROGRESS`

Base commit: `8d869c09a8c43b1d674c16b5274026f4b61d07df`

Implementation checkpoint: `WORKTREE`

## Requested outcome

Add structured Git, argv-only process execution, and named verification
capabilities through the existing trusted provider and Plug lifecycle.

## Changes made

Added `agent_coding` as an additive provider module and registered a native
`agent_coding_provider` MCP binary. The provider implements structured Git
status/diff/log/show/branch/add/create/checkout/commit operations, bounded
argv-only `process_execute`, and scope-configured `verification_run`.

Added the trusted author package material under
`reference-plugs/tethers-agent-coding`, its build script, twelve exact MCP
manifests, and the Phase C quickstart documentation.

## Decisions and assumptions

The current native MCP provider seam is extended additively. The packer remains
Windows/x86_64-only; this phase will not redesign package compatibility to add
Linux.

## Evidence

Pre-work state: Phase B commit `8d869c09a8c43b1d674c16b5274026f4b61d07df` is
present on `origin/feature/agent-essentials`, and the worktree was clean before
this packet was opened.

Current evidence:

* `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml` passed.
* `cargo check --manifest-path tethers-0.1/host-rust/Cargo.toml --all-targets --all-features --locked` passed.
* `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml --locked agent_coding::tests`: 5 passed.
* Coding Plug pack passed with 12 capabilities; latest semantic package digest was `sha256:80e1a006c2ef3ca2ddf5c927f5d9b741ab5493118097b4e89249714f4a0b2078`.
* Coding Plug supervised conformance passed all 6 generic cases on the prior packaged provider; a final post-guard package/conformance run remains before commit.
* Disposable-provider smoke passed initialize, structured status/diff, argv `git --version`, named verification, Git add, and local commit; the smoke repository was disposable and no remote operation was performed.
* `git diff --check` and the `IN_PROGRESS` task-packet checker passed.

## Discoveries

The existing pack/conformance path requires exact provider tool schemas and
supports the same author-source staging pattern established in Phase B. Git
status on an empty repository reports `master` after normalising Git's human
status phrase; repository calls also reject a configured directory nested
inside a different parent worktree.

## Remaining risks

The current package compatibility is platform-specific and must not be
broadened as an incidental Phase C change. Full host compatibility and the
frozen Phase B test evidence still need to be rerun after the final package
checkpoint. The generic conformance suite does not exercise every mutating
coding operation, so the direct smoke and unit evidence remain important.

## Smallest next action

Run the final package/inspect/conformance and compatibility checks, then commit
and push the completed Phase C branch checkpoint.

## References

* `docs/AGENT_QUICKSTART.md`
* `reference-plugs/tethers-agent-workspace/README.md`
* `tethers-0.1/host-rust/src/plug_pack.rs`
