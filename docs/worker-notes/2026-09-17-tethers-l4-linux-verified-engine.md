# Tethers L4 — Linux Verified Engine Test Runner

## Authority

- Starting SHA: `8f8947e8363eb1731736e9e5dea0d01baa663002`
- Starting branch: `feature/tethers-l3-platform-test-boundary`
- Task branch: `feature/tethers-l4-linux-verified-engine`
- Worktree: native WSL checkout at `/home/matmus/biscuit-linux/projects/tethers`
- Windows checkout: not opened or modified

## Implementation

Added native Linux equivalents of the existing Windows verified-engine route:

- `scripts/prepare-current-engine.sh` builds the current OCaml engine through
  the prepared opam switch and writes the ignored
  `verification/current-engine-provenance.json` manifest.
- `scripts/run-rust-tests.sh` prepares and validates the engine before
  exporting `TETHERS_VERIFIED_ENGINE` and `TETHERS_ENGINE_PROVENANCE`.
- The normal Linux runner enumerates the host library and integration test
  targets explicitly. This avoids compiling the existing Windows-only
  benchmark binaries while preserving the product Rust test coverage.
- `--verify-only` validates existing provenance without rebuilding or running
  tests. It exists to make fail-closed behaviour directly testable; it is not
  the normal test route.

No Windows shell, Windows executable, PowerShell, or Windows path was used by
the Linux scripts. The Linux binary keeps the actual Dune-produced filename
`tethers_mcp_main.exe`; `file` identifies it as an ELF Linux executable, so
the name is not being used as a platform workaround.

## Linux engine evidence

The prepared switch is `bl-tethers-5.5.0`:

```text
engine path:
  tethers-0.1/engine-ocaml/_build/default/bin/tethers_mcp_main.exe
source commit:
  8f8947e8363eb1731736e9e5dea0d01baa663002
source tree:
  dfa3c9b88dbe27c5c59031ca0da0212e3d6bf3ce
binary SHA-256:
  4c89ff26024733218311086f0fff66c0a7fe2e652363a636a7efd358bd60e9af
OCaml:
  5.5.0
Dune:
  3.24.0
provenance schema/status:
  tethers.engine/1 / pass
```

The script builds `@all`, records the current Git commit/tree and binary hash,
and the runner checks all of them before Rust tests. It also resolves the
binary path and rejects a symlink that escapes the repository.

## Trust-check evidence

With a valid generated manifest restored after each case, the provenance-only
route produced these results:

| Case | Result |
| --- | --- |
| missing manifest | rejected before Rust tests: `Current engine provenance manifest was not produced` |
| stale source commit | rejected: `Current engine provenance does not match this source checkout` |
| tampered binary hash | rejected: `Current engine binary hash does not match provenance` |
| missing engine | rejected: `Current engine binary is missing or not executable` |
| valid current manifest and engine | accepted; Rust tests were not run in provenance-only mode |

## Test comparison

The raw Linux workspace run was reproduced before relying on the verified
engine:

```text
RAW LINUX:
1225 passed; 81 failed; 2 ignored
```

The explicit verified runner ran 26 product targets (host library plus Rust
integration targets). Both threading modes produced the same aggregate:

```text
VERIFIED ENGINE / DEFAULT THREADING:
1387 passed; 134 failed; 2 ignored

VERIFIED ENGINE / INTENDED SINGLE THREAD:
1387 passed; 134 failed; 2 ignored
```

The single-thread route is the authoritative Linux equivalent of the existing
Windows runner. The default-thread run was diagnostic only. The identical
failure count means the remaining concurrency cascades were not removed by
the intended runner; no production concurrency code was changed.

The verified runner's library target reported `1244 passed; 62 failed; 2
ignored`. The 21 missing-engine failures are absent as prerequisite failures.
Two tests that L2 grouped with the engine cluster now reach the provider path
and fail with `provider returned no response`, so they are classified with the
provider fixture failures rather than counted as engine failures.

## Taxonomy delta

| Category | L2/L3 baseline | L4 result | Interpretation |
| --- | ---: | ---: | --- |
| Missing verified OCaml engine | 21 | 0 | current engine is built, attributed, and verified before tests |
| Concurrency cascades | 41 | 41 | unchanged under single-thread protocol; deferred as out of scope |
| Provider fixtures | 16 | 18 in the comparable library run | two former engine-cluster cases now reach the provider and fail there |
| Unresolved Linux behaviour | 3 | 3 | unchanged and explicitly deferred |

The broader explicit integration-target run also exposed 72 additional
platform/provider fixture failures that the raw L3 workspace command did not
enumerate: Windows `SystemRoot` launch assumptions, Windows host-binary
lookup, and provider-conformance fixtures. They are measurement, not repairs,
and are not misreported as missing-engine failures.

## L3 enumeration audit

L3 changed the raw Linux enumeration from `1507` tests to `1308` tests:

```text
199 fewer enumerated tests = 164 known Windows-boundary failures
                         + 35 additional tests in those same families
```

The additional 35 are tests enclosed by the semantically Windows-only
installation/current-trust/recovery/publication, PowerShell/process-boundary,
and Windows lock families. They include tests that had passed on Windows; the
count therefore cannot be inferred from the 164 Linux failures. No
cross-platform semantic test was gated solely because it failed on Linux. The
explicit L4 runner then enumerated the host library and integration targets
separately, which is why its 1523-case aggregate is not an apples-to-apples
replacement for the L3 raw workspace count.

## Warnings and boundaries

No production semantics changed. No OCaml/Core, Tether syntax, provider
execution, replay, scheduling, outcome, Windows protocol, WSL configuration,
global Git configuration, or toolchain version changed.

Existing warnings remain non-blocking, including duplicate-target metadata,
unused/dead-code warnings, and Windows-specific support code warnings. No
global lint suppression was added.

`bl doctor tethers` continues to report the prepared workshop and toolchains
healthy but exits non-zero because the stable Rust toolchain lacks
`rust-analyzer`. The project override selects Rust `1.97.1`; the workshop was
not modified to change the doctor result.

## Verification commands

Completed or recorded for this slice:

- native engine preparation: pass;
- provenance-only valid and fail-closed cases: pass;
- raw Linux matrix: `1225 / 81 / 2`;
- verified default/single-thread matrices: `1387 / 134 / 2` each;
- `cargo fmt --all -- --check`: pass;
- `cargo check --workspace`: pass;
- `cargo test --workspace --no-run`: pass;
- `bl verify tethers`: pass (`tethers 0.7.1`);
- `git diff --check`: pass;
- `bl doctor tethers`: workshop healthy; non-zero only for missing stable
  `rust-analyzer`.

The final clean-tree check and publication state are recorded in the closeout
after the final commit.
