# Rocket Verified Kernel Experiment 1 — Proven Success Path (PARKED)

Status: PARKED
Owner: Codex
Type: isolated research side project
Base: 64d1557603366f2b8b934f987bfdef87e2b4ec0e
Branch: research/rocket-verified-kernel

## Goal

Build the smallest useful Rocq-to-OCaml verification experiment around an already-known-correct Rocket V3 theorem.

Do not prove the current B3A tree theorem first.

Formalise the accepted R3-3B2 simple success-path canonicalisation, prove its key properties in Rocq, extract executable OCaml, and compare the extracted result against both the existing B2 OCaml implementation and frozen Enc_V2 authorities.

This is a side project. It must not alter the active Rocket task packet, production call paths, frozen V2, Core, validator, R3-1/R3-2, or any accepted Rocket implementation.

## Why B2 first

B2 is an ideal calibration target because:

- its exact mathematical behaviour is already known;
- chains 1..11 match exhaustive authority;
- chain-11 crosses the decimal boundary;
- chain-1000 completes without complete permutation enumeration;
- any disagreement is therefore a problem in the formalisation/extraction experiment rather than an unknown Rocket theorem.

## Toolchain setup

1. Create a dedicated opam switch for the lab. Do not reuse the Tethers production switch.
2. Install/pin Rocq 9.2.0 plus its standard library.
3. Record:
   - opam version;
   - OCaml version;
   - Rocq version;
   - package lock/pin information.
4. Keep toolchain commands/documentation under this research directory.
5. Do not install optional plugins unless the proof genuinely requires them. Prefer Rocq core + stdlib for Experiment 1.

## Formal model

Define, at minimum:

- finite path size N;
- legal labels 1..N;
- distinguished entry vertex;
- semantic simple path;
- bijective Origin labelling;
- induced numeric successor table;
- ProgramComplete terminal;
- frozen integer byte ordering needed by the path theorem;
- lexicographic comparison of the relevant continuation representation.

Do not attempt to formalise all of Tethers Core or all of Enc_V2.

## Proof obligations

Machine-check at least:

1. The selected entry label is a legal label.
2. The generated label assignment is a bijection.
3. The generated successor table is one Hamiltonian path from the fixed entry to ProgramComplete.
4. Mapping the numeric successor path back to semantic path positions is total and unique.
5. Every committed target choice preserves existence of a legal completion.
6. The exact feasibility predicate is sound.
7. The exact feasibility predicate is complete for the supported path state.
8. The constructed continuation representation is lexicographically minimal among all legal path labellings.
9. The algorithm does not depend on raw Origin IDs or storage order.
10. Extraction yields a computational function with proof terms erased as expected.

If proving full Enc_V2 minimality directly would balloon the experiment, prove the path continuation theorem over a small formally specified serializer and then validate its correspondence to frozen Enc_V2 separately in OCaml. Document the trust boundary precisely.

## Extraction

Use Rocq program extraction to OCaml.

Keep generated source under:
research/rocket-verified-kernel/extracted/

Generated code must not be hand-edited.

Add a thin research-only OCaml adapter if required to translate between ordinary integers/lists and extracted types.

## Differential evidence

For the same fixtures, compare:

A. hand-written B2 OCaml;
B. Rocq-extracted OCaml;
C. frozen Enc_V2 exact/oracle result.

Required fixtures:

- chains 1..11;
- known chain-11 exact sequence;
- 9/10/11/12 decimal boundaries;
- 99/100;
- 999/1000;
- raw-ID renaming;
- storage reversal/permutation.

Require labels and relevant continuation representation to agree. Where integration is straightforward, require full frozen payload/digest equality too.

## Scale

After exact small-case agreement:

- 100
- 1000
- optionally 5000

Record deterministic work counts and extracted-code runtime as diagnostic information only.

Do not optimise prematurely.

## File layout

- rocq/theories/PathModel.v
- rocq/theories/PathCanon.v
- rocq/theories/PathProofs.v
- rocq/theories/Extract.v
- docs/PROOF_OBLIGATIONS.md
- extracted/ generated OCaml only
- tests/ research-only differential harnesses

The agent may add narrowly necessary files beneath research/rocket-verified-kernel/.

## Forbidden

- No docs/CURRENT_CLINE_TASK.md changes.
- No active B3A branch changes.
- No production Tethers wiring.
- No frozen V2 change.
- No accepted B2 code modification.
- No Core/validator changes.
- No redesign of Rocket.
- No tree/forest proof in Experiment 1.
- No generated extracted code outside this research directory.
- No claim that extracted code proves the surrounding OCaml adapters.
- No hiding axioms/admitted obligations.

## Axioms / admitted proofs

Target: zero project-specific axioms and zero Admitted.

If an unavoidable library axiom appears, list it explicitly using Rocq's assumption-printing facilities and explain it.

If any project theorem remains Admitted, the experiment is not COMPLETE.

## Completion criteria

COMPLETE only when:

- Rocq files compile from a clean dedicated switch;
- required proof obligations are machine checked;
- no project theorem is Admitted;
- extraction to OCaml succeeds;
- extracted code compiles in a research-only harness;
- three-way differential tests pass on the required tractable fixtures;
- chain-1000 extracted path completes correctly;
- production tree remains untouched;
- worker note records exact trust boundary, assumptions, versions and commands.

Otherwise finish as BLOCKED with the smallest precise obstruction.

## Deliverable

Write:
research/rocket-verified-kernel/docs/WORKER_NOTE.md

Include:
- environment;
- proof inventory;
- assumptions;
- extraction boundary;
- differential results;
- failures;
- whether Rocq feels suitable for a future verified Rocket kernel.


## Parked state — 2026-09-02

Experiment 1 / 1B is intentionally parked at the research frontier recorded in
`docs/WORKER_NOTE.md`.

Parked evidence head before this marker:
`91c6a64ede00f0a768d62aefe10c8abbcdc5fe05`

Outcome:

- Rocq proof/extraction pipeline established.
- Extracted B2 small cases and chain-11 exact result agree.
- Native-int extraction plus one redundant-pass removal makes chain 1000 complete in about 5.67 seconds.
- The executable representation remains superlinear because of repeated list/association-list scans.
- Required universal proof/refinement inventory remains incomplete.
- Complete three-way handwritten/extracted/frozen harness remains incomplete.
- Production Tethers and active Rocket work remain untouched.

Decision:

Do not continue Experiment 1C now. Preserve this branch as a research laboratory.
Future Rocq work should be theorem-focused and should not resume merely to optimise
the current extracted implementation. A future task may reopen the lab if a
specific Rocket theorem is worth formal verification.
