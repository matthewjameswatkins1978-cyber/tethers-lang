# Tethers 1.0 Support Matrix

Status: living support promise
Updated: 2026-09-26

Support is a product promise, not a claim that code cannot happen to compile
elsewhere. A target becomes official only after every required layer below has
evidence.

## 0.8.1 target matrix

| Target | 0.8.1 status | Evidence/truth |
| --- | --- | --- |
| Windows x86-64 | OFFICIAL | Full native runtime release; verified archive, checksums, provenance, CLI smoke, R2 authority suite, clean-machine recovery |
| Linux x86-64 | OFFICIAL | Full native runtime release; verified Ubuntu CI, runtime packager, clean-install smoke, orphan proof, provenance manifest |
| macOS ARM64 | OFFICIAL | Apple Silicon target; full native Host and engine, POSIX child process supervision, clean-install smoke, authority and recovery proofs |
| macOS x86-64 | COMPATIBILITY-TESTED | Intel compatibility target; builds and packages in CI, tested on macos-15-intel runner; promote to OFFICIAL if full gate passes cleanly |

> **Trust note on macOS:** macOS ARM64 and Intel packages compile, package and pass Tethers authority/runtime proof. The release is not yet Apple Developer ID signed or notarized, so public Gatekeeper-trusted distribution remains unavailable until signing/notarization credentials exist.

## Other platforms

| Platform | Status for 0.8.1 |
| --- | --- |
| Windows ARM64 | FUTURE / UNSUPPORTED |
| Linux ARM64 | FUTURE / UNSUPPORTED |
| Mobile | FUTURE / UNSUPPORTED |
| Remote hosted Tethers | FUTURE / UNSUPPORTED |

These statuses do not claim that the software cannot work on those platforms.
They mean Tethers does not promise tested, packaged, supported parity there.

## Support levels

- **OFFICIAL** — covered by the current product promise and all required
  evidence gates.
- **EXPERIMENTAL** — intentionally exercised, but not covered by the stable
  support promise; breaking changes or missing layers are possible.
- **COMMUNITY / UNVERIFIED** — may work through user effort, without a Tethers
  release or acceptance promise.
- **UNSUPPORTED** — outside the declared target; no compatibility or support
  promise is made.

## Official-target gates

Each official target must prove all of these, with the appropriate product
semantics preserved:

1. code portability;
2. build portability;
3. package portability;
4. clean-install smoke;
5. runtime behaviour;
6. provider supervision semantics;
7. filesystem and confinement behaviour;
8. recovery and uncertainty behaviour;
9. CI verification;
10. release production and checksums/provenance;
11. documentation and support instructions.

Compilation alone is insufficient. Platform-specific process signalling,
path/config discovery, executable lookup, cleanup, archive naming, checksums,
manifest generation, and clean-machine behaviour must be tested on the target.

## Current platform truth

The published 0.8.1 release ships full-runtime packages for Windows x86-64,
Linux x86-64, and macOS ARM64 as OFFICIAL targets, with macOS x86-64
COMPATIBILITY-TESTED. Each archive carries `tethers` plus the matching OCaml
engine, a release manifest, and checksums. The Portable Workbench is a separate
Rust surface and does not prove full-host parity.
