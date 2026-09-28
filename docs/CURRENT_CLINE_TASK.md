# TETHERS 0.8.2 — ADVERSARIAL HARDENING AND THREE-PLATFORM TRUST

Control contract: `1`

Status: `IN_PROGRESS`

Task colour: `Red`

Owner: `Integration / Architecture Agent (OpenCode session)`

Route: `Four bounded lanes (A replay/persistence, B process supervision, C
release engineering, D independent red team) as worker branches off
fix/tethers-0.8.2-hardening; Linux x86-64 and macOS (ARM64 + Intel) proof via
GitHub Actions CI; Windows x86-64 proof native on the canonical machine`

Worker note: `docs/worker-notes/2026-09-28-tethers-0.8.2-hardening.md`

Base branch: `main`

Base commit: `2710e867768f33be3e6bd1e722004c1c50e3f53b`

The Base commit is the `v0.8.1` tag target. All 0.8.2 work builds forward from
it on the integration branch `fix/tethers-0.8.2-hardening`. The v0.8.1 tag is
immutable and is never moved.

Supersession note: this packet supersedes the 0.8.1 maintenance record
previously in this file. The 0.8.1 objective is complete on `main`
(`v0.8.1` tag); its packet text remains in Git history.

Rust toolchain: `1.97.1`; plain Cargo resolved by root pin; `--locked` mandatory

## Objective

Repair the security, durability, recovery and process-supervision weaknesses
identified by the independent Tethers 0.8.1 review, and prove the runtime
trustworthy across Windows x86-64, Linux x86-64, macOS ARM64 (official) and
macOS x86-64 (compatibility lane) when the machine crashes, filesystem
operations fail halfway, parents are terminated abruptly, children hang,
replay state is incomplete, permissions are unsafe, or another process
interferes. Target product version 0.8.2. No language or authority
architecture redesign.

## Findings under repair

- DEF-01: Unix replay publication strands deterministic temporary files and
  bricks ledger reopening after an interrupted write.
- DEF-02: macOS supervised children may survive host termination.
- DEF-03: an abandoned fresh replay admission enters `claimed_no_state` with
  no supported recovery route.
- DEF-04: Unix provision-replay failures emit `diagnostic: null`.
- DEF-05: git porcelain v1 `-z` parsing mishandles renames and hard-codes
  `conflict: false`.
- RSK-01: Unix replay ownership/permission validation weaker than Windows.
- RSK-02: macOS data-file durability (fsync vs F_FULLFSYNC).
- RSK-03: macOS archive reproducibility.
- RSK-04: POSIX process-tree and resource-limit enforcement truth.

## Frozen decisions and invariants

Core plans; the Gate evaluates authority; the external Host owns physical
effects. A Plan is not permission; approval is not execution; COMMIT records
durable intent, not observed success; outcomes stay truthful; forged or
malformed authority input fails closed. Uncertainty never becomes permission;
interrupted operations never become silently retryable; absent receipts never
prove absence of effect; existing replay claims are never deleted merely
because a later stage errored. Preserve `tethers.authority/1`, `tethers.cli/1`,
Human Tether language 0.1. The Windows J09 fail-closed debris contract
(ledger_29) is preserved; POSIX gains recognised-residue tolerance per packet.
No `tethers.authority/2`. No global release debug-assertions. No weakened
tests. Destructive/fault-injection tests only against isolated temporary
stores, never Matthew's installed Tethers data.

## Lane ownership

- Lane A (`fix/0.8.2-lane-a-replay`, PR #50): replay_linux.rs, replay.rs,
  replay_store.rs, replay_windows.rs parity, authority_gate.rs hint, CLI
  resolve command, audit_082_replay.
- Lane B (`fix/0.8.2-lane-b-supervision`, PR #51): child_process.rs,
  agent_core.rs (exec + git_status), agent_coding.rs (process paths only),
  CLI watchdog entry, audit_082_git_status, audit_082_process.
- Lane C (`fix/0.8.2-lane-c-packaging`, PR #52): package-macos-release.sh,
  package-linux-release.sh, workflows, support matrix, 0.8.2 docs.
- Lane D (`fix/0.8.2-lane-d-redteam`, evidence PR #53 against
  `audit/0.8.2-red-baseline`): RED reproduction evidence, hostile fixtures,
  independent final review. Agent D does not implement A/B/C fixes.

## Acceptance criteria (condensed)

1. audit_082 suites RED against v0.8.1 (preserved separately) and GREEN
   against the accepted candidate on Windows x86-64 (native), Linux x86-64,
   macOS ARM64 and macOS Intel (CI lanes).
2. R2 authority suite bit-for-bit behaviour preserved; no authority semantic
   change; forged/malformed input still fails closed.
3. Crash-safe Unix publication with all ten fault-injection boundaries
   distinguished (not committed / durably committed / reconciliation required
   / hostile state rejected).
4. Operator resolve route for `claimed_no_state` on both platforms with
   quarantine, audit log, and refusal of every other durable state.
5. No orphaned descendants in the supported abrupt-host-death, SIGTERM and
   SIGKILL scenarios on POSIX; truthful supervision reporting.
6. Exec timeouts terminate the whole POSIX process tree; resource-limit
   enforcement truthfully reported per platform.
7. Git status agrees with native Git for renames, copies, conflicts, spaces,
   Unicode, detached HEAD and clean repositories.
8. macOS and Linux archives byte-reproducible from identical staging input
   (double-build hash equality inside the packagers).
9. Full Windows suite genuine in debug and release profiles; no unexplained
   ignored tests; bounded timeouts.
10. VERSION/release notes/support matrix updated from evidence only at RC
    stage; v0.8.1 remains historical truth.

## Forbidden changes

No Core/language/protocol/authority/replay semantics redesign; no
`tethers.authority/2`; no PR #45 revival; no force-push, rebase of shared
history, tag movement, crates.io publication, GitHub release publication, or
main update from a worker; no signing/notarization claims without credentials
and proof; no deletion of historical worktrees or the legacy Goose checkout;
no secret material in logs or source control.

## Stop conditions

Conflicting authority derivation between check and run paths; a required
platform gate that cannot be satisfied (report, never fake); contradictory
frozen architecture; missing credentials at a gated publication step; after
two materially similar failed repair attempts on one underlying problem —
stop with exact evidence and the smallest unresolved question.

## Expected pre-existing changes

Untracked `tmp_macos_arm64_log/` in the canonical checkout (unrelated,
preserved). None in the lane worktrees.
