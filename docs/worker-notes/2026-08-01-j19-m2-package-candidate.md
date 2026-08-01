# Worker Note

Task: `J19-M2 - Autonomous Package Candidate Programme`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Codex Terra High`

Status: `COMPLETE`

Base commit: `337ab11c9cd4059402ef48d5949365c9517867a7`

Implementation checkpoint: `c2069974c6c9d89481a1e9b583077ebe7df45e7b`

## Requested outcome

Inspect `.tetherplug` archives without execution, extract only accepted bytes to
host-owned quarantine, and persist immutable uninstalled installation-candidate
records. M2 creates no provider launch, Socket session, binding, invocation,
event admission, trust, approval, installation, or enablement.

## Changes made

- Added `package.rs`: strict archive/path/payload/manifest inspection with raw
  and semantic package identities, explicit resource limits, and ZIP 2.4.2
  deflate-only parsing.
- Added `candidate.rs`: staged quarantine extraction, second payload digest
  verification, strict immutable candidate records, conflict detection, reload
  integrity checks, and Windows junction refusal.
- Added positive/adversarial stored, deflated, traversal, collision, mutation,
  conflict, torn-record, and real Windows junction fixtures.
- Added root `justfile`, `scripts/check-dev-tools.ps1`, and the single AGENTS
  startup diagnostic rule. Updated J14 lockfile assertions for the authorised
  archive dependency.

## Decisions and assumptions

- `plug.json` uses strict typed JSON fields named in the packet; capability and
  payload arrays are canonical-order evidence. Semantic digest is RFC 8785 JCS
  over validated `plug.json`; raw ZIP digest remains separate.
- Limits are 64 MiB archive/total uncompressed data, 16 MiB entry, 512 entries,
  100:1 ratio, 256 KiB JSON, and 64 manifests.
- Quarantine uses a unique same-root staging child then rename. Candidate state
  is fixed to `quarantined_installation_candidate`; repeat exact evidence may
  receive a new candidate ID, while a same release with different semantic
  evidence fails closed.
- `justfile` was chosen because it shortens repeated root-level tooling, format,
  check, Rust-test, M2-test, and packet-verification commands without adding a
  build system. No tools were installed: `fd`, `jq`, `yq`, and `just` were
  already installed and correctly present in User PATH; stale applications need
  restart to inherit it.

## Evidence

- `just tools` — PASS: rg 15.2.0, fd 10.4.2, jq 1.8.2, yq 4.53.3, gh 2.97.0,
  just 1.57.0, Git 2.54.0, and PowerShell 7.6.4 resolved from a new process.
- `cargo +1.89.0 test package::tests --locked` — PASS: five M2 archive,
  identity, extraction, candidate-conflict, and mutation tests.
- `cargo +1.89.0 test candidate::tests --locked` — PASS: three registry and
  real Windows junction tests.
- `cargo +1.89.0 test --all-targets --all-features --locked` — PASS: 785 Rust
  unit tests and 29 integration tests.
- `check-tethers-toolchains.ps1` with the current-worktree OCaml switch — PASS;
  OCaml 5.5.0 and Dune 3.24.0.
- `dune build` and `dune runtest` through that switch — PASS.
- `verify-0.2.ps1` — PASS: 6/6 suites, including J14C 9 rows and 196 assertions.
- `check-tethers-task-packet.ps1` and `git diff --check` — PASS before handoff.

## Discoveries

- The former J14 scripts intentionally pin Cargo.lock. Their expected hash was
  updated from the accepted baseline to
  `c72087d25475c82a13e3b57396f57e965dbeca1f76a33b738322523a54fc20a3` so the
  non-M2 legacy matrix continues to prove no unexpected lockfile mutation.

## Correction pass (after `c2069974c6c9d89481a1e9b583077ebe7df45e7b`)

- Correction implementation commit: `eef54c4dd4caa58193f4dd4457c19f969a3b2957`.

- Replaced the invalid four-byte EOCD probe with full fixed-EOCD validation for
  multi-disk and Zip64 refusal; retained `zip` as the archive reader while
  independently enforcing the frozen TetherPlug ZIP profile.
- Made the payload index an exact two-way match with archive payload files and
  made indexed documentation absence fail during `inspect()`.
- Added explicit Windows `FILE_ATTRIBUTE_REPARSE_POINT` checks for every
  existing root/ancestor and revalidation before publication; junctions,
  symlinks, mount points and reparse paths fail closed.
- Quarantine now records and rechecks `plug.json`, signatures and every indexed
  file, marks all accepted files read-only before publication, and records the
  limited M2 claim: this protects against ordinary writes, not an administrator
  or a malicious same-user process able to change attributes or ACLs.
- Candidate v1 now preserves launch arguments, working directory, operation
  namespace, selected Windows platform, full payload role/size/digest evidence,
  and a distinct inspection-evidence digest. The committed golden record is
  `tethers-0.1/host-rust/fixtures/m2/candidate-record-v1.json`.
- Reload requires filename/ID agreement, exact file-set equality, read-only
  files, payload size/digest agreement, strict `plug.json` semantic identity,
  no unexpected executable-looking file, and no same-release semantic conflict.

## Correction evidence

- `cargo +1.89.0 test package::tests --locked` — PASS (8 tests).
- `cargo +1.89.0 test candidate::tests --locked` — PASS (4 tests, including a
  real Windows junction refusal).
- `cargo +1.89.0 fmt --all -- --check`; `check`; `test`; debug and release
  locked builds — PASS (789 Rust unit tests and 29 integration tests; pre-existing
  warnings unchanged, no new M2 warning).
- Toolchain gate, OCaml `dune build` and `dune runtest`, and
  `tethers-0.1/scripts/verify-0.2.ps1` — PASS (6/6 suites; J14C 9 rows,
  196 assertions). Packet checker and whitespace-range check — PASS.

## Final evidence patch

- Final implementation/test commit: `cb4d5b0` (final branch SHA is recorded by
  the normal publication handoff).
- Added real inspector fixtures for a nested `.tetherplug`, entry/total/ratio
  limits, ZIP comments, and Unix symlink metadata; stable profile refusals use
  `resource_limit` or `unsupported_archive_feature` as applicable.
- Added candidate evidence for filename/identity disagreement, duplicate
  identity evidence, pre-existing staging-target refusal, ancestor junction
  refusal, and a no-outside-write marker assertion. Existing unexpected-file,
  `plug.json` mutation, payload mutation, read-only, semantic-conflict, torn
  record, and reparse-root tests remain in the focused matrix.
- Final focused counts: `package::tests` 9 passed; `candidate::tests` 6 passed.
  `just test-rust` passed 792 unit tests and 29 integration tests. `just fmt`,
  `just check`, `just test-m2`, and `just verify` passed; pre-existing warnings
  remained unchanged. OCaml build/tests and `verify-0.2.ps1` passed again.
- No M3 behaviour, provider launch, Socket establishment, binding, trust,
  installation, approval, credential, policy, session, enablement, or Anchor
  admission was added.

## Remaining risks

The candidate/quarantine path is intentionally non-operational. Signature
trust, installation approval, launch, Socket use, bindings, credentials,
conformance, and enablement remain M3 or later work.

## Smallest next action

Lucy reviews the pushed M2 branch and accepts or rejects this evidence; no M3
work begins without a new authoritative packet.

## References

- Control commit `337ab11c9cd4059402ef48d5949365c9517867a7`.
- M2 commits `9fa27ac`, `b82672c`, `ffb7d2b`, `acb30c6`, `ecd3b0b`, `6c34dfa`,
  and `76570b6`.
- `tethers-0.1/host-rust/src/package.rs` and
  `tethers-0.1/host-rust/src/candidate.rs`.
