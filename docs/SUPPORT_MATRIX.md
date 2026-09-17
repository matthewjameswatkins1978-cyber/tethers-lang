# Tethers 1.0 Support Matrix

Status: living support promise
Updated: 2026-09-17

Support is a product promise, not a claim that code cannot happen to compile
elsewhere. A target becomes official only after every required layer below has
evidence.

## 1.0 target matrix

| Target | 1.0 intention | Current evidence/status |
| --- | --- | --- |
| Windows x86-64 | REQUIRED / OFFICIAL | Current practical 0.7 full-runtime target; published archive, checksums, manifest, CLI smoke, host/engine evidence pass |
| Linux x86-64 | REQUIRED TARGET / NOT YET OFFICIAL | 1.0 target; native host/engine build proven, clean-package smoke passed, runtime dependencies audited, deterministic packaging proven, CI lane added; awaiting independent Ubuntu CI proof for OFFICIAL status |

“Official” here describes the intended 1.0 support promise. Linux must not be
described as already supported until its gates pass.

## Other platforms

| Platform | Status for 1.0 |
| --- | --- |
| macOS x86-64/ARM64 | FUTURE / UNSUPPORTED FOR THE 1.0 PROMISE |
| Windows ARM64 | FUTURE / UNSUPPORTED FOR THE 1.0 PROMISE |
| Linux ARM64 | FUTURE / UNSUPPORTED FOR THE 1.0 PROMISE |
| Mobile | FUTURE / UNSUPPORTED FOR THE 1.0 PROMISE |
| Remote hosted Tethers | FUTURE / UNSUPPORTED FOR THE 1.0 PROMISE |

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

The published 0.7 native release is Windows x86-64. The Portable Workbench is a
separate Rust surface and does not prove full-host parity. Linux has proven
native host/engine build, clean-package smoke, runtime dependency audit,
deterministic packaging, and CI lane. Independent Ubuntu CI proof remains
required for OFFICIAL status.
