# Current Goal

Updated: 2026-08-11

## Goal

Tethers 0.3 public Plug authoring. P1 (Generic Operational Scope Evidence) is
ACCEPTED. P2 (public pack / inspect / conform authoring surface) is FINAL
ACCEPTED. P3 (PDF Tools Reference Plug Crucible) is FINAL ACCEPTED. P4 (Plug
Author Manual) is FINAL ACCEPTED. P5 (fresh-agent authoring proof) is FINAL
ACCEPTED. P6 (The Evil Bunny Test) is FINAL ACCEPTED at
`5ed7634d8abc4056e0faa1ff09924377dec6e645`. The active increment is 0.4 C1
(Together: Deterministic Fan-Out / Join Foundation); P7 / physical-parallel 0.4
work has NOT started.

P6 acceptance is anchored to accepted head
`5ed7634d8abc4056e0faa1ff09924377dec6e645`.

## Last accepted increment

P6 — The Evil Bunny Test — FINAL ACCEPTED at
`5ed7634d8abc4056e0faa1ff09924377dec6e645`.

A safe, deterministic adversarial protocol fixture under
`reference-plugs/evil-bunny-proof/` proved that hostile providers cannot turn
bad evidence into conformance success; three genuine generic conformance gaps
were corrected in `tethers-0.1/host-rust/src/conformance.rs`.
Experiment log: `docs/p6-evil-bunny-proof.md`.

## Active increment

0.4 C1 — Together: Deterministic Fan-Out / Join Foundation — is in progress.
The OCaml engine introduces the `together` fan-out / join block as deterministic
language semantics: independent Actions in one concurrency group, a join before
later Actions, an additive `groups` plan field, and a deterministic
`group_planned` Trail entry, without any scheduler or physical-parallel
requirement.

P7 / physical-parallel 0.4 work comes after C1 and has NOT started.

## Foundation Pass boundaries

- Preserve external JSON, exit codes, Trail shape, replay digests, and recovery
  behaviour unless a later packet explicitly authorises a migration.
- Compatibility fixtures are literal committed evidence and are not generated
  by the implementation being tested.
- Every packet reports each required command as PASS, FAIL, or NOT RUN; a
  mandatory NOT RUN blocks COMPLETE.
- Final packet verification is serial after the last permitted code/test edit.
- P6 must not smuggle adversarial-provider-specific knowledge into generic host
  code; every production correction is generic and regression-backed.
- 0.4 C1 establishes concurrency semantics only; it must not introduce a
  scheduler, threads, or physical-parallel execution, and a Tether without
  `together` must keep producing the pre-C1 semantic output.

## Authoritative references

- Road to 0.3: `docs/ROAD_TO_0_3.md`
- Current task: `docs/CURRENT_CLINE_TASK.md`
- Foundation Pass plan: `docs/architecture/TETHERS_FOUNDATION_PASS.md`
- F3a persistence inventory: `docs/foundation-pass/PERSISTENCE_INVENTORY.md`
- F1 debt ledger: `docs/foundation-pass/DEBT_LEDGER.md`
- Enduring principles: `docs/CONSTITUTION.md`
- Language semantics: `tethers-0.1/SPEC.md`
