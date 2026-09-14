# Tethers verification and release control

Verification is evidence about the checkout being accepted. Previous task
packets and old binaries are historical evidence only; they are never inherited
as a current PASS.

## Everyday checks

From the repository root:

```powershell
just tools
just fmt
just check
```

`just verify` is the authoritative aggregate route. It checks the packet,
formatting, the declared toolchain, a current OCaml engine build, OCaml tests,
Rust and cross-language tests, protocol fixtures, MCP transcripts, and the
seed compatibility corpus, and the warning ratchet.

## Prerequisites and current engine

The supported Windows development environment uses the repository-local
path-bound switch at `tethers-0.1/engine-ocaml`, or an explicit absolute path
in `TETHERS_OCAML_SWITCH` / `-OcamlSwitchPath`. The switch must contain OCaml
5.5.0, Dune 3.24.0, Yojson 2.2.2, and Digestif 1.3.1. The locked opam metadata
is the dependency authority. If a package is missing, verification stops with
a prerequisite error before dependent host tests; it does not report product
failures caused by an old engine.

`scripts/prepare-current-engine.ps1` runs the repository-owned toolchain check,
builds `@all` from the current source, and writes the ignored report
`verification/current-engine-provenance.json`. That report binds the engine to
the current commit and tree and records its SHA-256. Rust cross-language tests
accept only the path and provenance supplied by that preparation step; they do
not search PATH, sibling worktrees, or an arbitrary old `_build`.

## Test classes and outcomes

Required product and first-party suites must be `PASS` or verification fails.
Optional provider integrations may be `SKIPPED WITH REASON` when the external
provider is absent. Benchmark evidence is reported separately from correctness.
Task/checkpoint machinery is `EXTERNAL PROJECT TOOLING`; platform-only suites
are `NOT APPLICABLE` when their supported target is not being claimed. The only
aggregate outcome words are `PASS`, `FAIL`, `SKIPPED WITH REASON`, and `NOT
APPLICABLE`.

## Compatibility corpus

`compat/0.7/` is the durable seed corpus. It contains exact representative
configuration/CLI input, Human Tether source, a capability manifest, Plan,
Trail, and MCP transcript fixtures from the tagged 0.7 release line. Run:

```powershell
pwsh -NoProfile -File scripts/check-compatibility-corpus.ps1
```

This proves the seed structure and required version identifiers. R6 expands it
into the complete migration audit.

## CI and release eligibility

The bounded PR workflow checks the repository packet, formatting, Rust static
checks, fixture structure, compatibility seeds, and whitespace. It is not a
claim of Linux runtime support and does not replace the Windows current-engine
route where the required OCaml switch is available.

Release-control evidence follows this order:

```text
clean source commit
  -> coherent product/language/protocol versions
  -> declared toolchains
  -> current engine and provenance
  -> required verification
  -> compatibility corpus
  -> package eligibility
  -> checksum and manifest
  -> SBOM and build attestation
  -> publication
```

Release mode rejects a dirty tree or an engine whose source commit/tree or
binary hash does not match the checkout. R5 will implement package creation,
SBOM, attestation, clean-machine installation and final publication. No
attestation is made over a file that can later be modified.

## Warning policy

The accepted Rust warning baseline currently contains the duplicate-target
Cargo warning for `src/main.rs`. Existing accepted debt is visible and may
remain; changed production code must not silently increase it. A trivial
metadata repair is deferred if it would broaden this packet.
