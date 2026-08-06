# Tethers Foundation Pass

## F1-F10 technical implementation plan

Prepared: 2026-08-06  
Status: planned strengthening programme; F1 packet preparation is authorised.

## Executive decision

The Foundation Pass is a bounded strengthening campaign after the accepted J24K
and J24L Plug-install vertical slice. It adds no product capability. Its only
permitted outcomes are removal of a demonstrated defect, invalid state,
duplication, misleading contract, maintenance hazard, or measured cost.

The programme has ten separately reviewable packages, F1 through F10. Each
package uses a bounded branch and independent review. Only F10, verified from a
clean checkout on the primary Windows target, completes the programme.

## Provisional pre-pass baseline

`origin/main` was revalidated on 2026-08-06 as:

```text
24428139807cac0adeb0b62264547e61ca809d16
```

This is the provisional pre-pass baseline. F1 must revalidate it from Git before
claiming it as its source; if `main` has advanced, it must record the new SHA and
stop for a packet correction rather than silently substituting a base.

## Programme boundaries

In scope: the current Rust host and OCaml engine, their persistence, protocol,
execution, CLI, tests, CI/tooling, and authoritative documentation.

Out of scope: language syntax or semantics, new Plug capabilities or CLI
commands, a universal storage framework, speculative parallelism, replacement
of Rust or OCaml, cosmetic renaming, and dependency additions without a
documented unsolved defect.

## Non-negotiable operating rules

1. The repository and Git are authoritative; inspect current state before any
   claim.
2. Test accessibility never justifies widening production visibility or adding
   a public production seam. Internal tests belong at the appropriate private
   boundary; public behaviour uses public surfaces.
3. A packet is the ruler. It must never be weakened, removed, or reinterpreted
   to legalise an implementation. Stop on a real conflict.
4. Report every verification command exactly once as `PASS`, `FAIL`, or
   `NOT RUN`. A mandatory `NOT RUN` blocks `COMPLETE`.
5. Capture every reported SHA directly from Git. Never reconstruct or expand a
   SHA from memory or a prefix.
6. After the final code or test change, run the complete required verification
   matrix serially. Focused checks may be iterative; final evidence may not be
   parallel or interleaved.
7. Every test must prove its named property directly. Nearby evidence, a
   comment, or generated output is not proof of a negative property.
8. A documentation checkpoint changes documentation only. Production and test
   changes require an earlier implementation/verification checkpoint.
9. Preserve external JSON, CLI output, exit codes, Trail shape, replay digests,
   and recovery semantics unless a package explicitly authorises a migration.
10. One owner works one bounded package or subpackage at a time; an independent
    reviewer who did not implement it is required.

## Compatibility evidence rule

F1 establishes literal committed compatibility fixtures for the public boundary:
CLI help and representative output, exit-code cases, JSON envelopes, Trail
records, replay digests, installation outcomes, and recovery states. Fixtures
must be hand-reviewed literals, owned independently of the implementation that
is tested. They must not be regenerated or updated by the production generator
under test; fixture changes require an explicit compatibility decision.

## Sequence and gates

- F1 measures and records; it does not repair.
- F2 repairs only reproduced operational defects.
- F3 and F4 finish before structural extraction in F5.
- F5 finishes before F7 consolidates tests.
- F8 reaches zero warnings in a separate cleanup checkpoint before any warning
  denial or CI enforcement checkpoint.
- F8 gates F9 and F10 because documentation must describe actual enforced
  commands.
- F10 alone is the programme completion gate.

## F1 — Baseline and debt inventory

Objective: create reproducible evidence before broad edits.

Deliverables: accepted baseline SHA; clean-checkout transcript; module and
dependency map; contract-mapped test inventory; warning inventory; literal
compatibility fixture set; persistence inventory; and a debt ledger classified
as Confirmed defect, Contract ambiguity, Maintainability debt, Performance
hypothesis, or Documentation debt.

Required baseline commands are adapted only to live repository reality:

```powershell
rustup show
cargo --version
cargo fmt --all -- --check
cargo check --all-targets --all-features --locked
cargo test --all-targets --all-features --locked
cargo clippy --all-targets --all-features --locked -- -W clippy::all
just verify
just verify-agent
```

Capture cold and warm timings separately. F1 may not repair findings. The
described live-stderr-tail issue is a candidate only until a targeted baseline
characterisation proves it.

## F2 — Operational correctness defects

Repair only F1-confirmed runtime defects without protocol or persistence
redesign. The first candidate is truthful live stderr-tail capture in
`child_process.rs`: diagnostics must reflect bytes observed before a hung or
timed-out child exits, retain bounded byte storage, distinguish timeout, EOF,
I/O, child-exit, kill, and join failures, and prove Windows cleanup directly.

## F3 — Persistence contract alignment (one programme gate)

F3 remains one review and acceptance gate, but executes serially as bounded
subpackages:

- **F3a: inventory and vocabulary.** Classify each store as immutable atomic
  record, replaceable current-state record, append-only causal log, or
  multi-step intent/recovery journal; record write primitive, atomic visibility,
  file and directory durability, recovery reader, corruption classification,
  unsafe-path protection, and direct tests.
- **F3b: Windows primitive evidence.** Establish what temporary-file sync,
  rename, directory handling, reparse-point defence, and interruption tests
  actually promise on the primary target. Do not overclaim directory durability.
- **F3c: installation intent and publication.** Audit the specialised J24K
  intent/recovery contract, exact-match removal, canonical bytes, and recovery
  matrix without universalising it.
- **F3d: immutable and current-state stores.** Align bounded stores with the
  proven vocabulary and explicit recovery/corruption contract.
- **F3e: Trail and replay.** Preserve append-only causal-log and replay-specific
  semantics; do not force either into an atomic-record format.

F3 cannot pass until every subpackage has independent evidence and the combined
contract has no contradictory durability claim.

## F4 — Outcome and protocol boundary

Make invalid internal execution states unrepresentable while retaining exact
wire compatibility through a deliberate boundary such as
`OutcomeWire <-> ExecutionOutcome`. Literal fixtures protect Trail bytes, replay
digests, and public JSON from implementation-coupled drift.

## F5 — Structural extraction

Extract only modules whose contracts are stable after F2–F4. No forwarding
layers, speculative traits, or cosmetic moves; each extraction must reduce a
demonstrated ownership or maintenance hazard.

## F6 — Measured performance and operational cost

Address only F1-measured costs. Preserve deterministic ordering, trust, and
recovery semantics; a concern without a measurement is not an optimisation task.

## F7 — Test-suite contract consolidation

Consolidate duplication only after F5. Retain direct tests for named properties,
especially failures and compatibility fixtures; reduce count only where the
same contract remains at least as strongly evidenced.

## F8 — Warnings, tooling, and enforcement

First remove or justify the live warning inventory in bounded cleanup packages.
Verify zero intended warnings and record a documentation-only checkpoint. Only
then, in a separate bounded change, add `-D warnings` and CI/tooling enforcement.
Never combine gate activation with warning repairs.

## F9 — Documentation and operator truth

Align authoritative documentation, task templates, and operator guidance to the
implemented contracts and enforced commands. This is documentation-only and may
not disguise product, test, or tooling changes.

## F10 — Clean-checkout proof

From a genuinely clean Windows checkout, independently run the documented
serial matrix, verify literal fixtures, inspect the complete programme diff,
and record all commands as PASS/FAIL/NOT RUN. Any mandatory NOT RUN prevents
programme completion.
