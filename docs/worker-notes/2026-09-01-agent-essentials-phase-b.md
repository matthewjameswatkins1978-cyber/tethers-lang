# Tethers Agent Essentials — Phase B Worker Note

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Lucy`

Base commit: `393fd5f06bf61b36bf270357a7f82e69f212ec26`

Branch: `feature/agent-essentials`

Worktree: `C:\Users\Matmus\Documents\Codex\2026-08-31\tethers-agent-essentials`

Status: `IN_PROGRESS`

Implementation checkpoint: `WORKTREE`

## Implemented

Added a separate `agent_workspace_provider` so the reviewed M4 File Tools
provider remains unchanged. The new provider uses the existing MCP stdio seam
and host-delivered operational scope, with an allow-listed operation set:

* `filesystem_read`, `filesystem_list`, `filesystem_stat`
* `text_search` (explicit literal or regex mode)
* `text_read_range`, `text_replace_exact`, `text_compare`
* `patch_apply` (one-file exact unified hunk with optional base digest)
* `hash_sha256`, `hash_verify`, `hash_directory_manifest`

Path resolution rejects traversal and symbolic-link ambiguity. Reads, search,
patches, and manifests are bounded; directory listings and manifests are
deterministically ordered; exact replacement refuses unexpected match counts;
patches refuse stale context and unrelated paths; hash verification reports a
boolean rather than treating mismatch as success. Installed operational scope
must be strict JSON and unknown scope fields are refused.

## Verification

* `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml -- --check`: PASS.
* `cargo check --all-targets --all-features --locked`: PASS.
* Phase B workspace unit tests: 8 passed, 0 failed.
* Existing M4 provider tests: 4 passed, 0 failed.
* Native MCP provider smoke: initialize, deterministic directory listing, and
  empty-string SHA-256 passed.
* Author source packed through the real `tethers plug pack` CLI with 11
  capabilities; `plug inspect` succeeded.
* Supervised `plug conform` passed all 6 generic conformance cases, including
  exact live tool discovery/schema matching.
* Two independent final package builds produced the same raw archive SHA-256
  `7e0fe65d225791ce51d79ab8c9a905761845c854e6f3aa56232af18e38715766` and
  semantic package digest
  `sha256:434cc4fcfb1b68c062c491ecfacd35701d2d9fcdd9658207505c1937518537c0`.
* `git diff --check`: PASS.
* Task-packet checker: PASS.

The Phase B author package material is now present and reproducibly packable.
The generated package is a local ignored artifact; trust, installation,
conformance, explicit scope, and enablement remain separate lifecycle steps.
The current packer emits Windows/x86_64 packages; Linux packaging and the
remaining Agent Essentials phases are not complete.
