# Tethers Agent Essentials — Phase B Worker Note

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Lucy`

Base commit: `393fd5f06bf61b36bf270357a7f82e69f212ec26`

Branch: `feature/agent-essentials`

Worktree: `C:\Users\Matmus\Documents\Codex\2026-08-31\tethers-agent-essentials`

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
* Phase B workspace unit tests: 6 passed, 0 failed.
* Existing M4 provider tests: 4 passed, 0 failed.
* Native MCP provider smoke: initialize, explicit text search, and exact patch
  call passed.
* `git diff --check`: PASS.
* Task-packet checker: PASS.

The provider is implemented and tested, but official signed/packaged Plug
artifacts and host-installed discovery manifests remain a later packaging
step; this note does not claim the full Agent Essentials release is complete.
