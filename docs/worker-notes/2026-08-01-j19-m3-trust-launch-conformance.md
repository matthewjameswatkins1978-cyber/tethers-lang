# Worker Note

Task: `J19-M3 - Autonomous Trust, Launch, Conformance and Install Programme`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Codex Sol Medium`

Status: `COMPLETE`

Base commit: `17d2a17468a9d7395d31d4b66b5f6e828f82102c`

Implementation checkpoint: `16c173f60a4ca8c6ad7a3500379ab22969032f79`

## Requested outcome

Complete P7 through P10 as one M3 programme: establish package signature and
host trust evidence, prepare an honestly labelled supervised Windows launch,
run a host-owned exact-pinned conformance gate, and require explicit
installation approval before publishing an immutable installed-disabled Plug.
M3 must add no active binding, operational launch, policy availability, runtime
Ask approval, credential handling, Trail effect, replay admission, Result
Anchor, external Anchor admission, or M4 behaviour.

## Changes made

- Added `m3_store.rs`, a small strict host-owned persistence utility with JCS
  record bytes, SHA-256 record integrity, create-new temporary files, flush,
  same-root rename, and Windows reparse-chain refusal.
- Added `trust.rs` with strict detached Ed25519 envelope verification, RFC 8410
  DER SubjectPublicKeyInfo key identities, exact semantic-digest signing input,
  host-owned publisher state transitions, revocation, namespace/expiry checks,
  and a visibly separate exact-digest unsigned developer approval store.
- Added `launch_profile.rs` and extended `child_process.rs` for exact absolute
  no-shell launch from revalidated quarantine material, clean environment,
  bounded scratch, Job Object ownership, eight-process and 256 MiB per-process
  limits, bounded protocol/stderr handling, and kill-on-close cleanup.
- Added `conformance.rs` and the native credential-free
  `m3_fixture_provider` test binary. The host suite validates MCP 2025-11-25,
  provider identity, every discovery page, exact trusted operation/input
  schemas, trusted output schemas, valid/invalid calls, bounded failure and
  interruption, zero retry, shutdown, and immutable exact-pinned evidence.
- Added `installed.rs` with separate installation-review/approval and installed
  record stores, final trust/evidence revalidation, same-volume staging,
  exact copy/digest/file-set verification, read-only payload publication,
  create-only records, disabled bindings, and strict restart validation.
- Added `fixtures/m3/m3-schema-golden-v1.json`, six trust tests, and eight
  end-to-end M3 lifecycle tests including real Windows junction evidence.
- Added the thin `just test-m3` recipe and refreshed three J14 lockfile
  integrity sentinels after the authorised cryptography dependency changed
  `Cargo.lock`.
- Corrected conformance discovery after review so manifest-backed schemas,
  pagination, repeated/empty cursor refusal, duplicate/unapproved operation
  refusal, and trusted output validation are enforced rather than inferred.

## Decisions and assumptions

- The signature contract is Ed25519 over
  `TETHERS-PACKAGE-SIGNATURE-V1\n<semantic-digest>\n`; signature validity and
  publisher trust remain separate evidence. `ed25519-dalek` 2.2.0, `base64`
  0.22.1, and their mature locked transitive dependencies implement the narrow
  cryptographic primitive; host code enforces the Tethers envelope and trust
  rules.
- Publisher identity is host-owned. Unknown, disabled, expired, namespace-
  mismatched, conflicting, and revoked keys fail closed. Revocation cannot be
  reversed into trust; historical evidence remains but carries no trusted-time
  or pre-compromise claim.
- The unsigned developer path is visibly unsigned, host-created, create-only,
  and bound to one exact lowercase `sha256:<64 hex>` semantic identity. It is
  not publisher trust and is revalidated before approval and publication.
- The launch profile is named `supervised`, sets `isolated=false`, and records
  `process supervision only; not isolated or hostile-code-safe`. It is process
  ownership and launch hygiene, not a sandbox or production credential boundary.
- The clean child environment contains only `SystemRoot`, `WINDIR`, `TEMP`,
  `TMP`, and `TETHERS_CONFORMANCE`. `PATH`, profile scripts, inherited secrets,
  package-selected environment, shells, downloads, and command concatenation
  are excluded.
- Conformance suite identity is `m3-generic-1` plus a canonical suite digest.
  Evidence separately represents passed, failed, interrupted, and invalidated
  outcomes and pins package, payload, manifest, capability, protocol, platform,
  launch, trust, suite, host build, cases, retry count, and evidence digest.
- Installation approval is distinct from runtime Ask approval. Installed state
  is exactly `present_disabled`; every recorded binding is `disabled`, and the
  registry exposes zero active bindings. No production resolver consumes these
  M3 records.
- Windows read-only attributes and host-owned roots protect against accidental
  ordinary mutation and make exact reload checks provable. They do not claim to
  resist an administrator, the same user deliberately changing permissions, or
  hostile code; later execution integrity must still be rechecked.

## Evidence

- Control commit `acefdde8dae8f4e19492129dec6384b4808de3bc` was checked out
  clean on `codex/j19-m3-trust-launch-conformance`; the packet checker passed
  after the control document was aligned with its required headings.
- `just tools` - PASS: rg 15.2.0, fd 10.4.2, jq 1.8.2, yq 4.53.3, gh 2.97.0,
  just 1.57.0, Git 2.54.0 and PowerShell 7.6.4 resolved by executable path.
- `just fmt` - PASS.
- `just check` - PASS for all targets/features with the accepted legacy warning
  baseline (13 library warnings, seven library-test warnings including four
  duplicates, and five J13A test warnings). No M3 module warning was emitted.
- `just test-m3` - PASS: six trust tests and eight M3 lifecycle tests. The
  lifecycle set covers golden schemas; exact signature names and duplicates;
  trust restart/revocation; prelaunch mutation, missing/additional file and
  reparse refusal; real Windows junction roots/no outside write; clean exact
  launch; paginated discovery; typed schema/output/malformed/oversize/timeout
  refusal; interruption; zero retry; evidence invalidation; approval; immutable
  read-only install; restart corruption/torn/conflict refusal; and zero active
  binding or operational M3 effect.
- `just test-rust` - PASS: 798 unit tests, 29 J13A CLI integration tests and
  eight M3 integration tests, 835 total, all targets/features and locked.
- `just verify` - PASS while the packet was `IN_PROGRESS`; packet consistency,
  format, check, and the complete locked Rust suite passed.
- `.github/scripts/check-tethers-toolchains.ps1` with
  `D:\The Next Thing\Tethers Lang - J16 Clean\tethers-0.1\engine-ocaml` - PASS:
  Rust 1.89.0, opam 2.5.2, OCaml/ocamlopt 5.5.0, Dune 3.24.0 and Yojson 2.2.2.
- `opam exec --switch=<exact worktree switch> -- dune build` - PASS.
- `opam exec --switch=<exact worktree switch> -- dune runtest` - PASS.
- `cargo +1.89.0 build --locked` and
  `cargo +1.89.0 build --release --locked` - PASS.
- First `verify-0.2.ps1` attempt - J13A 25/25, J13B 10/10 and J13C 19/19
  passed; J14A/B/C stopped before behaviour checks because their old fixed
  `Cargo.lock` digest did not recognise the authorised M3 dependency change.
- Second complete `verify-0.2.ps1` attempt after commit `16c173f` - PASS: six
  of six suites. Counts were J13A 25/25, J13B 10/10, J13C 19/19, J14A 5/5 with
  95 assertions, J14B 11/11 with 243 assertions, and J14C 9/9 with 196
  assertions.
- Windows process-cleanup proof after all tests - PASS: no surviving M3 fixture,
  reference-host, OCaml engine, or provider process beneath the repository.
- `git diff --check` - PASS before completion ledger creation.

## Discoveries

- The branch remote exists but this checkout's configured fetch refspec tracks
  only `origin/main`; the exact milestone ref was therefore fetched and pushed
  explicitly without changing repository configuration.
- The J14 harness intentionally hashes `Cargo.lock` to prove test non-mutation.
  P7's authorised Ed25519 dependency changed that file, so the three frozen
  expected values were updated to
  `4238151009218547ce20e9686c2a8cf12d321e31998b35e1d087b10d0ce674d7`.

## Remaining risks

- The supervised profile is deliberately not isolated and must never be used
  as hostile-code or production-credential protection.
- A crash after installed payload rename but before record publication can
  leave an inert host-owned orphan directory. With no valid installed record it
  is unavailable and non-operational; later maintenance may remove such orphans.
- Windows read-only attributes and same-user host roots are integrity evidence,
  not an administrator-resistant security boundary.
- Existing tracked non-fatal Rust warnings remain as permitted by the packet.
  None originate in the new M3 modules.
- M4 activation, runtime availability, operational launch, binding enablement,
  production credentials, health/restart policy and capability implementation
  remain deliberately unimplemented.

## Smallest next action

Lucy independently reviews the published M3 branch and evidence ledger before
authorising any M4 packet.

## References

- Branch: `codex/j19-m3-trust-launch-conformance`
- Control commit: `acefdde8dae8f4e19492129dec6384b4808de3bc`
- Packet-control repair: `8bb36c620b0a1d79c21b1cc321fec4a9ba2fa018`
- P7 trust/signature: `b08394eabf84f1162c34d69b515302dbec01da0f`
- P8 supervised launch: `df0323ee2bc59b1f264041fa89cb682850508b58`
- P9 conformance: `68027972a9fb0d98c03340b3b30afe26e504be89`
- P10 installed-disabled: `2d8e66e7b96b824707d880ea6fdc94fb84fe82e1`
- Final M3 implementation/test correction: `4e27e8c05eb1148127cda221df0573b8099aff1e`
- Regression-harness evidence commit: `16c173f60a4ca8c6ad7a3500379ab22969032f79`
- Golden schema fixture:
  `tethers-0.1/host-rust/fixtures/m3/m3-schema-golden-v1.json`
- Focused lifecycle evidence: `tethers-0.1/host-rust/tests/m3_lifecycle.rs`
- Final branch SHA is the completion-ledger commit which contains this note and
  the packet `COMPLETE` transition; it is verified against the remote after push.
