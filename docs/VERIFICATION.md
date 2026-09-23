# Tethers verification and release control

`just verify` is the authoritative aggregate verification route. It is a
repository-owned Python coordinator, `scripts/verify-tethers.py`, with thin
PowerShell compatibility wrappers for Windows automation. The coordinator is
the same on Windows and Linux; only the process launcher for the current OCaml
engine and Rust host is platform-specific.

## What it proves

In this order, the verifier runs:

1. task packet consistency;
2. Rust formatting;
3. current OCaml engine build and provenance;
4. OCaml tests;
5. Rust static checks;
6. warning ratchet;
7. Rust and cross-language tests;
8. protocol fixture sanity;
9. MCP transcript suite;
10. compatibility corpus.

If the current engine cannot be built or its provenance cannot be proved,
dependent suites are reported as `SKIPPED WITH REASON`; they are not reported
as product failures. Any required suite that actually runs and fails makes the
aggregate verdict `FAIL`.

The only suite outcomes are `PASS`, `FAIL`, `SKIPPED WITH REASON`, and `NOT
APPLICABLE`. The ignored machine report is written to
`verification/tethers-verification.json` with schema `tethers.verify/1` and
contains the source commit/tree, dirty state, platform, toolchain status,
engine provenance, ordered suites, outcome counts, first meaningful failure,
verdict and `release_eligible`.

## Prerequisites and current engine

The verifier discovers the OCaml switch automatically. It first honours
`--ocaml-switch` or `TETHERS_OCAML_SWITCH`, then checks this checkout's local
`tethers-0.1/engine-ocaml/_opam`, then walks the switches registered with opam.
It selects only a switch that runs the repository-pinned OCaml 5.5.0 and Dune
3.24.0 and has Yojson 2.2.2 plus Digestif 1.3.1 installed. If none is
installed, verification reports a prerequisite failure and skips dependent
suites; it does not install or alter a toolchain. A supplied switch remains
authoritative and fails closed if it is invalid.

The current engine is always built from this checkout; the switch supplies
only the OCaml compiler and dependencies. The provenance record binds the
resulting binary to this checkout's commit, tree and SHA-256. A fresh agent
worktree therefore needs no remembered absolute switch path when the machine
already has a registered compatible opam switch. The native preparation path
is:

```sh
bash scripts/prepare-current-engine.sh
bash scripts/run-rust-tests.sh
```

The preparation step builds the current OCaml source and writes
`verification/current-engine-provenance.json` using schema `tethers.engine/1`.
The Rust runner verifies the source commit, source tree, binary existence and
SHA-256 before exporting `TETHERS_VERIFIED_ENGINE` and
`TETHERS_ENGINE_PROVENANCE`. It never searches PATH or accepts an arbitrary
older engine. Windows uses the existing PowerShell engine/test launchers at
the same narrow boundary; the verification coordinator is still Python.

## Everyday checks

```sh
just fmt
just check
```

For authoritative evidence:

```sh
just verify
```

On Linux this command is native. It does not call `pwsh.exe`,
`powershell.exe`, `cmd.exe`, or Windows-side tools. Run it from the native WSL
filesystem, not from `/mnt/c` or `/mnt/d`.

## Test classes

Required first-party product and protocol suites must pass. Task/checkpoint
machinery is external project tooling but its failure still invalidates the
aggregate evidence. Optional provider integrations may be skipped only with an
explicit reason. Platform-only suites are `NOT APPLICABLE` when the claimed
platform is not being verified. No old packet, binary or previous PASS is
inherited.

## Compatibility corpus

`compat/0.7/` is the durable historical seed corpus. Its native checker proves
the required files, JSON/JSONL syntax, historical version identifiers and
representative Human Tether, manifest and Plan shape. Future migration audits
may expand this corpus; they do not change the current verifier contract.

## Release eligibility

The report marks `release_eligible` only when the checkout is clean and every
required suite passes. This is a control-layer gate, not a complete package
release proof. Later release work must add clean-machine packaging, checksums,
SBOM, attestations and publication evidence in that order:

```text
verified source -> build -> package -> checksum/manifest -> SBOM
-> attestation -> publish
```

## Warning policy

The static Rust check is a locked, all-targets/all-features compilation check.
The separate warning ratchet owns the accepted baseline. The current baseline
includes Cargo's duplicate-target warning and the Linux-only unused
`RetainedProviderSession::from_discovered` helper warning carried by the
accepted L1 parity work. These are visible non-blocking debt; changed code must
not add another warning without an explicit baseline update.

## Compatibility wrappers

The existing `.ps1` helper names remain for Windows callers and CI, but they
delegate to the native Python checks. They do not define a second verification
semantics. Linux calls the Python modules directly and uses only the native
engine/test launchers.
