# Project Overview

## Name

Tethers Lang

## Purpose

Tethers is a deterministic behaviour language and capability protocol. It lets
applications expose typed events, Facts, and Capabilities, then lets readable
rules connect those inputs to planned Effects.

The joint Tethers/Lantern Keeper build foundation is
[`architecture/TETHERS_LANTERN_KEEPER_CANONICAL_ARCHITECTURE.md`](architecture/TETHERS_LANTERN_KEEPER_CANONICAL_ARCHITECTURE.md).
It keeps Tethers as the general coordination layer: Tethers plans with exact
capability versions, hosts authorise and execute, AI judgement is an explicit
Capability Action, and application meanings such as Lantern Keeper memory
remain outside Tethers Core.

The imported `tethers-0.1` prototype is the active development tree. It proves
the signed-off 0.1 round trip and now hosts the first 0.2 runtime slice:

1. A host supplies an event, immutable Facts, Capability schemas, and Tether
   source.
2. The OCaml engine parses, validates, evaluates, and returns a Plan.
3. The Rust host resolves trusted manifests and effective policy.
4. Approved Actions cross an intent-first dispatch boundary before execution.
5. Results are validated and joined to evaluation and execution through the
   Trail and known-outcome Result Anchors.

## Authority And Engineering Standards

- `docs/CONSTITUTION.md`: enduring Tethers language principles.
- `tethers-0.1/SPEC.md`: precise 0.1 language and protocol semantics.
- `docs/DECISIONS.md`: accepted architectural decisions.
- `docs/CAPABILITY_BRIDGE.md`: trusted manifest and host bridge contract.
- `docs/IMPLEMENTATION_LANGUAGE_STANDARD.md`: implementation technique for
  senior engineers and AI coding agents.
- `docs/CURRENT_GOAL.md`: present objective, boundaries, and next authorised work.

The Constitution's demand for a small, human-clear Tethers language does not
require elementary implementation code. OCaml, Rust, PowerShell, and future
implementation languages should be used idiomatically and to the depth justified
by the problem.

## Current Contents

- `tethers-0.1/`: active prototype and runtime development tree.
- `tethers-0.1/SPEC.md`: authoritative 0.1 semantics.
- `tethers-0.1/engine-ocaml/`: deterministic planner and MCP tools.
- `tethers-0.1/host-rust/`: trusted host, manifest store, policy, dispatch, and
  provider integration.
- `tethers-0.1/protocol/`: protocol, manifest, fixture, and transcript contracts.
- `tethers-0.1/scripts/`: build, verification, integration, and demo automation.
- `docs/worker-notes/`: durable implementation and review evidence.

## Scope Of The Tethers Language

The language remains intentionally small. It supports a canonical rule shape,
immutable Facts, deterministic evaluation, typed Capability validation, ordered
Plans, host-side authorisation, and causal Trails.

Language features such as loops, arbitrary mutation, hidden I/O, parallel
Actions, direct Action-result chaining, or application-specific modes remain
outside the signed-off 0.1 language unless an explicit design gate authorises a
future change.

This restriction applies to the Tethers language surface, not to the power of
the general-purpose languages used to implement it.

## Current Integration Status

Latest accepted implementation checkpoint:
`d5ed278d4a2cae5e9ab8a3e1d8700fdcba7ae851`.

The accepted runtime baseline includes verified manifest admission, deterministic
capability projection, planner-to-dispatch manifest and provider pins,
configured stdio MCP provider admission, intent-first serial dispatch, executor
output validation, known-outcome Result Anchors, and effective policy outcomes
of `allow`, `ask`, `deny`, and `unavailable`.

The stale-digest and unestablished-structured-scope paths fail closed. J04a is
accepted. No J05 implementation is authorised until a separate Red design gate
freezes approval and resume semantics.
