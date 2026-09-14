# Linux x86-64 portability audit

Status: R3 implementation audit
Updated: 2026-09-14

This document records the platform-specific mechanisms required to preserve
Tethers semantics on native GNU/Linux x86-64. It is an engineering boundary
record, not a claim that Linux support is complete by itself. WSL runs are
additional development evidence; the GitHub-hosted Linux job is the independent
release evidence.

## Support target

The R3 target is `x86_64-unknown-linux-gnu` on a current Ubuntu LTS runner.
The package is dynamically linked and therefore supports distributions whose
glibc and loader meet the versions reported by the Linux packaging job. R3 does
not promise every Linux distribution, Linux ARM64, or musl compatibility.

The native Linux build must construct the current OCaml engine and Rust host
from the same checkout. The engine identity remains `tethers.engine/1`; the
machine report adds an additive platform object while retaining its existing
schema identity.

## Mechanism inventory

| Component | Current Windows mechanism | Required semantic property | Linux candidate mechanism | Security consequence | Tests required | Implementation decision |
| --- | --- | --- | --- | --- | --- | --- |
| Provider launch | Win32 process creation and Job Object | argv-based launch, bounded pipes, attributable exit, bounded shutdown | `std::process::Command`, new session/process group, `PR_SET_PDEATHSIG` | shell-free execution; direct child and descendants that remain in the session are terminated together | normal exit, crash, timeout, flood, descendant, parent-death smoke | Keep shared supervision logic; use a small Unix adapter and document that a child which deliberately creates a new session is outside the guarantee |
| Timeout and kill | Job Object termination plus Win32 wait | monotonic timeout and honest failure/uncertainty | `Instant`, `kill(-process_group, SIGKILL)`, `wait` | process-group kill prevents ordinary descendants surviving timeout or close | child and descendant hang, pending output | Linux shutdown kills the supervised session group before reaping the direct child |
| Parent termination | Windows host cleanup path | provider does not silently survive host death in supported cases | `PR_SET_PDEATHSIG(SIGKILL)` in the child before exec | direct provider death is inherited by ordinary descendants | parent-death fixture in Linux CI | Set the parent-death signal in the pre-exec adapter |
| Replay persistence | handle-bound NTFS/reparse-safe store and Win32 exclusive lock | immutable canonical records, fail-closed recovery, per-key exclusion | canonical files with `create_new`, `sync_all`, non-replacing rename, directory sync, exclusive lock file | symlink ancestors and non-regular records are rejected; no path-prefix-only authority | provision, restart, collision, corruption, symlink escape | Add a Linux replay store with the same record graph and public outcomes; keep Windows backend unchanged |
| Filesystem scope | Win32 reparse-point rejection | resolved target remains under the authorised root | `symlink_metadata`, `canonicalize`, ancestor validation | symlink escape is refused before the operation | in-scope path, traversal, symlink escape, missing components | Reuse existing shared `verify_chain`/canonicalisation rules and add Linux fixtures where the shared seam is not enough |
| Permissions | ACL and Win32 access checks | permission failures are explicit and not mislabeled | Unix metadata, executable-bit and native I/O errors | package/provider permissions are checked at the launch boundary | non-executable provider, unreadable config, read-only target | Preserve native error context and map only at the public Tethers error boundary |
| Temporary and atomic state | Win32 write-through temporary + rename | no replacement of an existing record; durable visibility before acknowledgement | unique UUID directory, `create_new`, `sync_all`, rename, parent-directory `sync_all` | collisions and torn files fail closed | parallel temp roots, restart, interrupted publication | Keep existing store format; use Linux durable publication helper |
| Paths and environment | drive-aware paths, `SystemRoot`, Windows temp | platform state does not alter Core meaning | `Path`, `HOME`/`TMPDIR` only at host boundary, explicit environment | no shell interpolation and no ambient Core dependency | LF/CRLF corpus, locale, stale PATH, package outside checkout | Normalise only for host diagnostics; do not alter canonical semantic identity |
| Packaging | Windows ZIP and `.exe` siblings | clean package contains current binaries and no checkout dependency | tar.gz with native `tethers` and `tethers-engine` | package smoke must run from an empty directory | file/ldd/readelf, clean unpack smoke | Add a Linux package recipe and run it from CI; retain Windows packaging |

## Explicit Linux supervision guarantee

The host launches providers without a shell. Each provider becomes a session
leader; the host records its PID and terminates its process group on timeout or
shutdown. The Linux child also receives `SIGKILL` when its original host parent
dies. This covers the provider and ordinary descendants which retain the
session/process-group relationship. A provider that deliberately calls
`setsid(2)` or otherwise escapes its process group is not a hostile-code
sandbox and is outside this R3 guarantee. Tethers must report the resulting
native termination honestly; it must not claim arbitrary-code containment.

## Evidence classes

- Shared semantic and protocol suites are run on Windows and Linux.
- Windows reparse/ACL fixtures and Linux symlink/permission fixtures are
  different traps protecting the same stated property.
- WSL is useful for development and manual smoke only.
- The GitHub-hosted Ubuntu job is required independent Linux evidence.

## Current audit findings

The pre-change Linux compile failed because replay provisioning and the replay
authority were gated to the Windows module. Unix child shutdown also only
addressed the direct child. These are implementation defects, not acceptable
platform-specific differences. The R3 change therefore keeps the Windows
backend and adds a Linux adapter behind the same replay-runtime seam, while
adding Unix process-group supervision without changing Core, Plan, Trail, or
authority semantics.
