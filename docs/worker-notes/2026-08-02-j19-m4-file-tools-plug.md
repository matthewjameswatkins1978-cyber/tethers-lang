# J19 M4 File Tools Plug Worker Note

Task: `J19-M4 - Autonomous File Tools Plug Vertical Slice`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Status: `COMPLETE`
Owner: `Luna / OpenCode`
Branch: `opencode/j19-m4-file-tools-plug`
Control commit: `61cba8c76e7b013dd4b93b0a121ab31621067a96`
Base commit: `8cd8958d4880595dfff5e38ab5ec26de940944df`
Implementation checkpoint: `ab720b4ccab16438e5ec2ae8c38d7b90bdfb83be`
Accepted M3 baseline: `8cd8958d4880595dfff5e38ab5ec26de940944df`
Frozen architecture base: `a5fd63593a9d9acd397030ecd2e27b4f318c87fd`

Use this file as the durable M4 implementation ledger. Record:

- exact control commit and starting branch state;
- P11 contract decisions and frozen schemas;
- P12 provider/package implementation commits;
- P13 end-to-end implementation commits;
- focused tests and full regression evidence;
- security refusals, remaining risks and deferred work;
- final branch SHA.

Do not use this note to change frozen architecture or quietly expand M4 into Anchors, networking, credentials, arbitrary file access, PDF support or release work.

## Requested outcome

Complete P11, P12, and P13 as one bounded, credential-free File Tools vertical
slice while preserving the M3 trust boundary and released 0.2 behaviour.

## Changes made

Added strict File Tools contract projections, manifests, deterministic ZIP
package construction, bounded host filesystem checks, native MCP provider,
provider integration tests, and an immutable explicit enablement store.

## Decisions and assumptions

The Query is `file.metadata@1`; the Action is `file.move@1`. The provider is
MCP 2025-11-25 over stdio, Windows x86_64, local-only, credential-free, and
supervised but not isolated. Roots are host-owned and separate for query,
source, and destination operations.

## Evidence

See the Implementation Ledger and Verification sections below. All focused and
full commands listed there were run locally on native Windows.

## Discoveries

The supplied M4 control packet omitted the checker-required Base commit and
several canonical section headings. They were repaired to the accepted M3
baseline before final packet validation. The checkout's remote refspec also
required an explicit branch ref fetch.

## Remaining risks

Supervised execution is not hostile-code isolation. Windows same-user file and
administrator integrity limits remain those documented by M3. Public lifecycle
CLI and external Anchors remain deferred.

## Smallest next action

Independent Lucy review of the pushed M4 branch and evidence ledger. Do not
begin M5 from this worker session.

## References

P11/P12/P13 commits and all manifest/test paths are listed in the Implementation
Ledger. Governing contracts are the M4 packet's named J18C-J18G documents.

## Implementation Ledger

- Starting branch state: clean `opencode/j19-m4-file-tools-plug` at the control commit above; remote branch was fetched explicitly because the checkout tracks only `origin/main`.
- P11 commit: `734451e` (`feat: freeze M4 file tools contract`).
- P12 commit: `d5fc520` (`feat: add native M4 file tools provider`).
- P13 commit: `ee1a618` (`feat: add explicit M4 plug enablement`).
- Capability identities are `file.metadata@1` Query and `file.move@1` Action; operations are `file_metadata` and `file_move`.
- Manifest digests are `sha256:369f4034f702847bb82d1ef82e93f2c5661cad4ad2d7496c3685b406747db09a` and `sha256:2ac3793d4b61725fd130dac531d9690b93881341245f0c2f7c3aca2fd2dd2311`.
- The package builder fixes ZIP timestamps and ordering, indexes every payload, and produces equal bytes for equal provider bytes.
- File paths are host-approved relative slash paths. Query content is capped at 65536 bytes and must be UTF-8. Move is non-recursive, same-volume only through the Windows rename boundary, refuses overwrite, and rechecks reparse chains.
- The native provider is credential-free, local-only, stdout-protocol-only, and implements MCP 2025-11-25 initialize, tools/list, and tools/call for only the two reviewed operations.
- Enablement is a separate strict immutable store. It pins one installed ID, package/provider/conformance/approval evidence and exact capability bindings. Availability is false before enablement and after durable disablement; policy, intent, replay, outcomes, Result Anchor and Trail remain existing host seams.

## Verification

- `just tools`: PASS.
- `just fmt`: PASS.
- `just check`: PASS; existing M3/J13 warnings only.
- `just test-m4`: PASS; 5 contract/provider unit tests and 2 Windows integration tests.
- `just test-m3`: PASS; 6 trust tests and 13 lifecycle tests.
- `just test-rust`: PASS; 805 unit tests, 29 J13A CLI tests, 13 M3 lifecycle tests, and 2 M4 integration tests.
- `just verify`: PASS.
- Toolchain gate, OCaml `dune build`, and OCaml `dune runtest`: PASS.
- Locked debug and release builds: PASS.
- Complete `verify-0.2.ps1`: PASS; J13A 25, J13B 10, J13C 19, J14A 5/95, J14B 11/243, J14C 9/196.
- `git diff --check`: PASS. Windows provider process cleanup: PASS.

## Security and Scope

No credentials, network listener, shell, PATH selection, arbitrary listing, delete, copy, glob, overwrite, recursive operation, external Anchor, PDF feature, marketplace, updater, or M5 behavior was added. Supervision remains distinct from isolation. The native executor adapter is only dispatchable after the existing resolver, policy, durable intent, replay, outcome, Result Anchor, and Trail boundaries.

Implementation branch SHA at completion checkpoint: `ee1a618`.

## Integration Correction

- Correction commit: `ab720b4ccab16438e5ec8c38d7b90bdfb83be`.
- Operational launch no longer accepts arbitrary command, arguments, or working
  directory. `FileToolsExecutor::launch_from_installed` derives the executable,
  payload set, working directory, provider identity, trust, approval,
  conformance, enablement, and exact enabled operation from installed state.
- Installed launch rehashes the exact file set, rejects reparse/drift, uses the
  M3 suspended-before-execution Job Object path, clean allow-listed environment,
  explicit stdio handles, bounded limits, and host-owned scope placeholders.
- `OperationalScopeBinding` pins separate canonical query/source/destination
  roots, a read bound, exact installed/capability identity, authority, and an
  integrity digest. Package arguments contain reviewed placeholders only; the
  host materialises them immediately before launch.
- Enablement now uses sequence/predecessor chains with genesis validation,
  fork/duplicate/conflict refusal, exact scope binding, and deterministic
  current-state selection. Disablement appends a transition and removes the
  resolver availability snapshot.
- `execute_enabled_file_tools_action` routes the installed binding through the
  existing policy, durable intent, replay, canonical outcome, Result Anchor,
  and Trail boundary. Provider JSON-RPC refusals classify as definite failed;
  transport timeout/process loss remains uncertain.
- The real Windows M4 integration now builds the native provider, creates and
  inspects the deterministic package, runs M3 candidate/trust/conformance/
  approval/install gates, proves unavailable-before-enable, launches from the
  installed payload, resolves exact enabled capability, performs Query and
  move, proves shared-path replay blocking and Result Anchor/Trail evidence,
  then disables and proves unavailable-after-disable.

Correction verification: `just test-m4` passed 4 integration tests, `just
test-m3` passed 6 trust and 13 lifecycle tests, serialized full Rust passed 805
unit + 29 J13A + 13 M3 + 4 M4 tests, `just verify` passed, toolchain/OCaml
gates passed sequentially, and `verify-0.2.ps1` passed all 6 suites. A native
M3 handle test showed one parallel-run race; isolated and serialized reruns
passed. No M5 behavior was added.
