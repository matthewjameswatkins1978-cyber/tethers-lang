# Current Implementation Task

Control contract: `1`
Task: `TETHERS-0.3-P2C — End-to-End Public Author Proof + Final P2 Gate`
Owner: `OpenCode`
Status: `COMPLETE`
Task colour: `Amber`
Route: `OpenCode implements proof + final gate`
Worker note: `docs/worker-notes/2026-08-10-0.3-p2c-public-author-proof.md`
Base branch: `feature/0.3-p2b-fix2-wire-cleanup-proof`
Base commit: `061a57d4bd48e59cae2d496b889834df7fe54418`
Implementation branch: `feature/0.3-p2c-public-author-proof`
Implementation checkpoint: `7430b6c40ff96a408ec0f5b5b514482dee599b8a`
OCaml switch path: `not applicable`
Rust toolchain: `1.97.1`
Rust change class: `TEST_AND_DOCS_ONLY`

## Objective

Prove the complete public Plug-author journey using only the public CLI. Run the
expensive final P2 verification gate. No production code changes are expected.

## Relevant background and existing behaviour

- P2A (public deterministic plug pack) is ACCEPTED.
- P2B (public supervised plug conform) is ACCEPTED at `061a57d4bd48e59cae2d496b889834df7fe54418`.
- The public CLI: `plug pack`, `plug inspect`, `plug conform` all exist and pass their suites.
- P2C is primarily PROOF, not new production behaviour.

## Required behaviour

### FIX 1 — End-to-end public author journey test

Create `tests/p2c_public_author_journey.rs` with ONE coherent test proving:

1. **Author source tree** — construct from scratch with synthetic identity
   (`example.public-author-proof`, `tethers-stdio-fixture`, `fixture.ping@1`).
   Uses `payloads` not `payload_index`. Omits `manifest_digest` and manifest `digest`.

2. **STEP 1 — Real public pack** — launch `tethers-reference-host plug pack`.
   Prove: exit 0, schema, command, status ok, expected identities,
   digest fields, exact one JSON envelope line.

3. **STEP 2 — Real public inspect** — launch `plug inspect` on the packed package.
   Prove: exit 0, identity unchanged, manifest digest now exists,
   digest continuity (`pack.semantic_package_digest == inspect.package.semantic_digest`),
   package bytes unchanged by inspect.

4. **STEP 3 — Execution safety gate** — launch `plug conform` WITHOUT
   `--allow-non-isolated-supervised-execution`. Prove: exit 5,
   `approval_required`, `conformance_execution_approval_required`,
   provider marker file does NOT exist.

5. **STEP 4 — Real approved public conform** — launch `plug conform` WITH
   `--allow-non-isolated-supervised-execution` using a dedicated TEMP/TMP parent.
   Prove: exit 0, disposition passed, isolated=false, limitation present,
   retry_count=0, raw_stderr_persisted=false, evidence fields present.

6. **Provider execution proof** — after approved conform, provider marker MUST exist.

7. **Cleanup proof** — no `tethers-p2b-conform-*` directories remain in the
   dedicated TEMP/TMP parent.

8. **Digest continuity** — `pack semantic_package_digest == inspect semantic_package_digest == conform semantic_package_digest`.

9. **Immutability proof** — source plug.json, manifest, and provider bytes
   unchanged; package bytes unchanged since pack.

10. **Public output hygiene** — no quarantine path, trust-store path, temp
    workspace path, raw provider stderr, or M3_SECRET_CANARY in public output.

The test must use only the public CLI (launching `tethers-reference-host` binary).
It must NOT call internal Tethers functions.

### FIX 2 — Final P2 verification gate

Run `just verify-agent` from repository root once. Record all component results.

### FIX 3 — Documentation closeout

Update `docs/ROAD_TO_0_3.md`, `docs/CURRENT_GOAL.md`, `docs/PROJECT_DASHBOARD.md`
to record P2B accepted, P2C proof complete, P3 next.

## Frozen decisions and invariants

1. No src/ production code changes
2. No Cargo.toml or Cargo.lock changes
3. No conformance case changes
4. No supervised launch semantics changes
5. No P3 work
6. No `plug stage`, `plug install`, `plug enable` in the journey

## Relevant components

### Authorised paths

- `tests/p2c_public_author_journey.rs` (new)
- `docs/CURRENT_CLINE_TASK.md` (update)
- `docs/ROAD_TO_0_3.md` (update)
- `docs/CURRENT_GOAL.md` (update)
- `docs/PROJECT_DASHBOARD.md` (update)
- `docs/worker-notes/2026-08-10-0.3-p2c-public-author-proof.md` (new)

## Acceptance criteria

1. Author source tree constructed with synthetic identity, `payloads` not `payload_index`, no manifest_digest
2. Public pack exits 0 with correct identities, digest fields, one JSON envelope line
3. Public inspect exits 0, identity unchanged, manifest digest exists, digest continuity with pack
4. Conform without approval exits 5; provider not executed
5. Approved conform exits 0, disposition passed, isolated=false, limitation present, evidence fields
6. Provider marker exists after approved conform
7. No `tethers-p2b-conform-*` directories remain in dedicated TEMP/TMP parent
8. `pack semantic_package_digest == inspect semantic_package_digest == conform semantic_package_digest`
9. Source and package bytes unchanged throughout journey
10. Public output contains no internal paths, raw stderr, or M3_SECRET_CANARY
11. `just verify-agent` — PASS from repository root
12. Documentation closeout: `ROAD_TO_0_3.md`, `CURRENT_GOAL.md`, `PROJECT_DASHBOARD.md` updated

## Required verification

1. Focused P2C test: `cargo test --locked --test p2c_public_author_journey`
2. `cargo fmt --all -- --check`
3. `git diff --check`
4. `cargo clippy --all-targets --all-features --locked`
5. `just verify-agent` (once, after all code changes)
6. Complete diff contains no src/ production changes
7. No dependency changes from P2B base (Cargo.toml + Cargo.lock unchanged)
8. Task packet checker `control-v1/COMPLETE`
9. Branch pushed, remote == local, genuinely clean worktree

## Required verification

1. Focused P2C test
2. Cargo fmt check
3. Git diff check
4. Clippy
5. `just verify-agent` (once, after all code changes)
6. Task packet checker
7. Git publish + remote/local equality

## Forbidden changes

- No src/ production files
- No Cargo.toml
- No Cargo.lock
- No conformance case changes
- No P3 work
- No `plug stage`, `plug install`, `plug enable` in the journey

## Stop conditions

- Any production src/ change appears necessary
- Pack/inspect/conform digest continuity breaks
- Provider executes without explicit conform approval
- Conform leaves ephemeral state behind
- Source files are mutated
- Inspect mutates package
- Conform mutates package
- Dependency changes appear
- `just verify-agent` fails
- Two materially similar attempts fail

## Expected pre-existing changes

None. HEAD equals `061a57d4bd48e59cae2d496b889834df7fe54418` (P2B FINAL ACCEPTED).
Branch `feature/0.3-p2c-public-author-proof` created from P2B HEAD. Working tree clean.
