# Project Overview

## Name

Tethers Lang

## Purpose

Tethers is a deterministic behaviour language and capability protocol. It lets
applications expose typed events, facts, and actions, then lets readable rules
connect those inputs to planned effects.

The joint Tethers/Lantern Keeper build foundation is
[`architecture/TETHERS_LANTERN_KEEPER_CANONICAL_ARCHITECTURE.md`](architecture/TETHERS_LANTERN_KEEPER_CANONICAL_ARCHITECTURE.md).
It keeps Tethers as the general coordination layer: Tethers plans with exact
capability versions, hosts authorise and execute, AI judgement is an explicit
capability Action, and application meanings such as Lantern Keeper memory
remain outside Tethers Core.

The imported `tethers-0.1` prototype is the active development tree for the
entire 0.1 cycle. It proves a narrow reference round trip:

1. A host supplies an event, facts, capability schemas, and Tether source.
2. The OCaml engine parses, validates, evaluates, and returns a plan.
3. The Rust reference host authorises the required effects.
4. The host executes a mock capability exactly once using an idempotency key.
5. Evaluation, authorisation, and execution are joined into one Trail.

## Current Contents

- `Tethers-0.1-Prototype.tar.gz`: preserved local source archive, not committed.
- `tethers-0.1/`: active 0.1 development tree.
- `tethers-0.1/SPEC.md`: 0.1 semantics and project constitution.
- `tethers-0.1/engine-ocaml/`: reference deterministic planner.
- `tethers-0.1/host-rust/`: reference host and mock executor.
- `tethers-0.1/protocol/`: request and expected response fixtures.
- `tethers-0.1/examples/`: first sample Tether.
- `tethers-0.1/scripts/`: demo and fixture scripts.

## Scope Of 0.1

The prototype is intentionally small. It supports one rule shape, immutable
facts, deterministic evaluation, typed capability validation, ordered plans,
host-side authorisation, host-side execution, idempotency keys, and a causal
Trail.

The prototype deliberately excludes loops, parallel actions, action-result
conditions, live fact queries, retries, compensation execution, adapters,
package management, scheduling, HQ, and AI integration.

## Integration Status

The native Windows baseline has been verified with opam, OCaml, Dune, yojson,
PowerShell fixture checks, Rust tests, the OCaml engine build, the golden
engine-response test, and the full Rust -> OCaml -> Rust demo.

Columbo manifest verification and the Trusted Manifest Store are complete
through checkpoint `25ab2bb`. The next integration target is one vertical
runtime slice around provider admission, live capability projection, effective
policy, conservative serial dispatch, honest uncertain outcomes, result
Anchors, and execution Trail writing.
