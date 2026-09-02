# Tethers 0.5 final-audit closeout evidence

## Frozen release boundary

- Base: `31d5e39a1e3505880e9a98cd8c650b3cf112b16d`
- Implementation checkpoint: `b69cca6ed9409c68295d7492253b353696dc4758`
- Branch: `release/tethers-v0.5-final-audit`
- Frozen Enc_V2/ProgramDigest V2 and the exhaustive Rocket reference were not
  changed.

## Rocket and Benchmarker

The first-class `tethers-bench` target was built with the packet-authorised
OCaml switch and run with warmups, samples, iterations, JSON output, and a
comparison input. The run reported:

- schema: `tethers.benchmarker/1`;
- five fixed corpus cases;
- portfolio/reference parity: PASS for all five cases;
- comparison rows: three;
- backend context: `native`;
- per-case backend labels and resource counters present;
- JSON predicate validation: `true`.

The benchmarker is observation-only. It does not invoke providers, request
authority, write Trail, or participate in canonical identity. A parity failure
fails the benchmark rather than producing a performance claim.

The OCaml release regression run also reported passing V2 oracle and production
tests, Rocket model/stage/portfolio/refinement suites, request/adapter/wire
suites, exact-hybrid differential/metamorphic corpus, origin-walk, success-path,
and exact-chain checks. The Rocket differential corpus reported zero mismatches.

## Cold-agent execution proof

The real J14A public journey passed with 7 cases and 121 assertions:

```text
discover -> inspect -> preview -> harmless execution -> result -> Trail receipt
```

The proof records that preview performs no provider invocation, authority grant,
or Trail write; harmless execution produces a Result Anchor; the public Trail
inspection and bounded receipt expose the causal story; and exact replay blocks
the duplicate provider effect while reusing the execution identity.

## Preview, receipt, and starter examples

- `preview` is a public read-only host command over the existing validation and
  planning boundary.
- `trail --receipt` is a bounded projection over validated existing Trail
  entries; it is not a second store and omits unapproved fields.
- `examples/tether-sets/` contains runnable ordinary `.tether` examples for
  typed work, Together workflow, and result/follow-on use.

## Toolbelt deferrals

The final manifest/provider re-check found no existing structured-data or
read-only system-orientation operation that could be exposed without a new
contract. Structured data would require a typed operation and validation
surface; system orientation would require an explicit secret-exclusion and
allow-list contract. Both remain `DEFERRED-WITH-REASON`, alongside archive,
HTTP, and SQLite. No new authority model, dependency, daemon, server, or
database was introduced.

## Platform and package evidence

Windows package and extraction smoke passed locally. The package contained the
native host, portable workbench, benchmarker manual, and three starter examples;
portable `version --json`, portable `doctor --json`, and native `describe --json`
all passed. The Windows package SHA-256 was:

```text
1A68268874575E62ADD708602C9C0891F9E5918CA6840F13D9C24A617C2FC9D1
```

The hosted Linux failures on the preceding release attempts were traced to the
Windows-only P2B/P2C Plug conformance journey being included in the Linux
matrix. The release workflow now keeps those journeys on Windows and retains
Linux host, discovery, pack, and package gates. The fresh tagged run
`33644808390` passed both platform package jobs and publication.

Published assets from [Tethers 0.5 release tag
`tethers-v0.5.8`](https://github.com/matthewjameswatkins1978-cyber/tethers-lang/releases/tag/tethers-v0.5.8):

```text
tethers-0.5.0-linux-x64-musl.zip
  sha256:f4dd11f2a8c652aa78ff908a002eae46c843b4950c37a1b2c630027e4691d168
tethers-0.5.0-windows-x64.zip
  sha256:13cf2aa4c16770fa1a8f8785774fd4e5cc092bb1ded639a4a496b81453b23d67
```

Both downloaded ZIPs matched their published `.sha256` sidecars byte-for-byte.
The tagged commit is `2a2fe3986805905a90aa48ad83e95d79f0357b04`.

## Closure commands

The following were run before the implementation checkpoint and passed:

```text
cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml -- --check
cargo check --manifest-path tethers-0.1/host-rust/Cargo.toml --locked --all-targets
cargo test ... --bin tethers-reference-host --bin m3_fixture_provider --bin file_tools_provider ...
pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\test-j14a-complete-scenario.ps1
pwsh -NoProfile -File .\scripts\package-tethers-release.ps1 -Target windows-x64
opam exec --switch='D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml' -- dune runtest --force
git diff --check
```

The final tagged workflow passed: Linux package/tests in 1m53s, Windows
package/tests in 4m06s, and publish in 12s. The release is public and
non-draft/non-prerelease. The branch and tag resolve to the tagged commit;
the local generated scratch directory was removed after the checks.
