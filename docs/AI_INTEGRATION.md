# AI integration

Tethers is intended to be discoverable by a capable AI without a private
architecture prompt. Start with `tethers --help`, then initialise and inspect:

```powershell
tethers init
tethers doctor
tethers capability list
tethers capability inspect workspace.read
```

Use `workspace.read` and `workspace.list` to inspect files. Use
`workspace.create` for new files and `workspace.replace` only with the exact
SHA-256 returned by a prior read/stat. Use `git.status`, `git.diff`,
`git.stage`, and `git.commit` for local version-control work. Use `exec` only
with an explicit program and repeated `--arg` values; Tethers does not create a
hidden command shell.

Use `tethers trail --limit 20` to inspect recent receipts, then
`tethers trail <receipt_id>` for one receipt. The legacy filtered reader remains
available with `--trail <absolute-jsonl> --execution-id <id>`.

When Threadmoth is installed, pass a workspace-relative protocol request to
`tethers threadmoth preview --request <file>` before applying it with
`tethers threadmoth apply --request <file>`. Tethers validates every target
path first and records the Threadmoth provider and protocol in its Trail.

Every operation returns a structured CLI envelope and appends a host Trail
receipt under the host-owned workspace state. A refusal is a result to reason
about, not permission to silently fall back to an unrestricted equivalent.
