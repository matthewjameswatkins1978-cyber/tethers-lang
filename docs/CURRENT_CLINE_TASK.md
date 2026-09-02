# Tethers 0.5 — Final Forgotten-Things Audit and Release Closure

Control contract: `1`

Status: `COMPLETE`

Task colour: `Red`

Owner: `Codex`

Route: `Fresh dedicated final-audit worktree from the completed Tethers 0.5 release branch. Audit every named release capability against repository evidence first; implement only genuine gaps; then perform the final release acceptance and publication closure.`

Base commit: `31d5e39a1e3505880e9a98cd8c650b3cf112b16d`

Implementation checkpoint: `426d5ac0ece15e51d560d4bfccd6f14ad4e72b63`

Worker note: `docs/worker-notes/2026-09-02-tethers-v0.5-final-audit.md`

Suggested branch:

`release/tethers-v0.5-final-audit`

Source branch:

`release/tethers-v0.5`

Updated: 2026-09-02

## Objective

Perform the final evidence-based audit of Tethers 0.5 before release.

Do not assume an item is missing merely because this packet names it.

For every item:

```text
inspect
  ↓
classify:
DONE / PARTIAL / MISSING / DEFERRED-WITH-REASON
  ↓
if DONE:
    preserve it
if PARTIAL or MISSING:
    complete the smallest coherent release-quality implementation
if it genuinely cannot fit 0.5 safely:
    record the exact blocker and explicit defer decision
```

The goal is to remove accidental omissions from 0.5 without reopening settled architecture or turning the release into another research programme.

The practical release standard remains:

> Make common use excellent. Keep uncommon use exact. Keep pathological cases diagnosable and reference-checkable.

Frozen V2 identity and the accepted authority/trust model remain immutable.

---

## Relevant background and existing behaviour

The existing `release/tethers-v0.5` line already contains substantial finished work:

- exact Rocket V3 portfolio seam;
- permanent exhaustive/reference Rocket engine;
- B2 success-path fast solver;
- R3-2 refinement;
- exact V2 IR pruning/memoisation;
- runtime-only fallback behaviour;
- Rocket differential/metamorphic evidence;
- Agent Essentials discovery;
- workspace/text/hash/patch provider;
- Git/process/named-verification provider;
- Windows/Linux release packaging workflow;
- AI-first documentation;
- cold-agent discovery evidence;
- benchmark evidence;
- Tether Set documentation;
- existing Plug lifecycle, trust, policy, scope, Trail, replay, Together and Result Anchor machinery.

Do not redo these simply because they appear in this packet.

The audit of the current release branch has also identified likely gaps that require explicit verification:

1. benchmark scripts and benchmark evidence exist, but no obvious named first-class `Benchmarker` tool/artifact exists;
2. the current cold-agent transcript proves discovery and conformance but stops before actual bounded work and resulting Trail/receipt inspection;
3. current project documentation explicitly says the side-effect-free plan/preview surface and richer Trail-query ergonomics remain follow-on work;
4. the repository tree contains documentation about Tether Sets but no obvious actual starter Tether Set artifacts;
5. the current Agent Essentials Plugs cover workspace/text/hash/patch and coding/Git/process/verification, but do not obviously provide structured-data, archive, bounded-HTTP, SQLite or read-only system-orientation capabilities;
6. current Agent Plug READMEs still describe Windows/x64 package generation, with Linux publication deferred;
7. physical clean-install evidence, final release URL/assets/signatures and tagged workflow results must be proven rather than inferred.

Treat these as audit leads, not pre-decided implementation conclusions.

---

## Required behaviour

1. Start from exact base `31d5e39a1e3505880e9a98cd8c650b3cf112b16d` on a fresh dedicated final-audit branch/worktree.

2. Before implementation, produce a release matrix classifying every item in this packet as `DONE`, `PARTIAL`, `MISSING`, or `DEFERRED-WITH-REASON`, with exact repository evidence.

3. Preserve the permanent exhaustive Rocket reference engine as a named first-class internal/reference facility. Do not delete, hide or replace it with the fast portfolio.

4. Verify that the Rocket portfolio can deliberately force/reference-check bounded cases and that all fast/exact backends continue to emit byte-for-byte identical frozen Enc_V2 results.

5. Audit the Benchmarker requirement. If no first-class Benchmarker exists, expose the existing benchmark machinery through the smallest coherent, scriptable agent-facing surface rather than creating a second benchmark framework.

6. The Benchmarker must support deterministic machine-readable output, human-readable output, before/after comparison, version/environment metadata, selected Rocket/backend route where relevant, timing/resource counters and stable result fields.

7. Benchmark measurements, clocks and environment metadata must never enter semantic identity, ProgramDigest, Plan ordering or any other deterministic Tethers meaning.

8. Complete the cold-agent proof so that a genuinely unfamiliar client performs at least one harmless bounded real operation through public Tethers surfaces, then inspects the resulting execution evidence/Trail/receipt.

9. The cold-agent proof must require no undocumented repository knowledge or privileged development shortcut. Discovery must lead the agent to the necessary capability contract and public execution path.

10. Audit the public side-effect-free plan/preview surface. If absent, implement the smallest public preview command/API that exposes what deterministic work Tethers would propose without executing providers or granting authority.

11. Preview output must clearly distinguish:
    - parsed/validated input;
    - proposed Plan;
    - unavailable capability/configuration problems;
    - authority not yet granted;
    - actual execution, which must remain absent.

12. Audit Trail ergonomics. If current Trail commands do not provide practical querying and execution receipts, add bounded stable machine-readable query/receipt surfaces over existing Trail data rather than inventing a second evidence store.

13. A useful receipt should make it easy for an AI to connect:
    - event/request;
    - proposed Action;
    - authority decision;
    - provider invocation;
    - result/uncertainty;
    - Result Anchor where present;
    - relevant causal identifiers.

14. Audit actual starter Tether Sets. If none exist, add a small set of canonical useful examples using existing Tether/host configuration semantics only. Do not invent a second language or speculative package format merely to obtain the name “Tether Set.”

15. Starter material must demonstrate that Tethers is not merely ALLOW/ASK/DENY. Include examples covering ordinary typed Capability work, at least one `together` workflow, and at least one visible result/follow-on flow where supported by existing public semantics.

16. Audit the Agent Essentials toolbelt against the current product target:
    - workspace/filesystem/text/patch;
    - Git;
    - process and named verification;
    - structured data;
    - hashes/integrity;
    - archives;
    - bounded HTTP/network;
    - SQLite;
    - read-only system/environment orientation.

17. Preserve already-good workspace/coding providers. Implement missing toolbelt slices only where they can use the existing Capability/Plug/trust/scope model cleanly.

18. Structured-data capability work must remain bounded and deterministic. Prefer explicit JSON/structured inspection/manipulation operations over embedding a general scripting language.

19. Archive operations must be scope-bounded and path-safe. Extraction must defend against traversal/absolute-path escape and report exact files affected.

20. HTTP/network capability, if implemented for 0.5, must be explicitly bounded: allow-listed scheme/host policy, finite timeout, finite response size, no ambient credential harvesting, no hidden redirects across disallowed authority boundaries and no automatic effectful retry.

21. SQLite capability, if implemented for 0.5, must have an explicit database scope and safe bounded defaults. Read-only inspection/query is sufficient for the first release unless an existing trust contract already supports mutation cleanly.

22. System/environment orientation must be read-only and deliberately exclude secret environment values. It may expose safe orientation such as OS, architecture, current approved roots and explicitly allow-listed tool availability.

23. A missing toolbelt slice may be `DEFERRED-WITH-REASON` only if completing it would require a new authority model, significant new dependency/platform subsystem, unsafe scope expansion or release-sized architectural work. “Ran out of time” is not by itself a technical reason.

24. Audit Agent Plug portability. If workspace/coding/toolbelt providers are actually portable but only their pack scripts are Windows-specific, add the smallest reproducible Linux packaging path and prove it in CI. If provider/platform constraints genuinely prevent this, document the exact boundary.

25. Audit version coherence. The release must present one understandable version story for:
    - product release 0.5;
    - Human Tether language/protocol 0.1 where still frozen;
    - host/package versions that intentionally remain 0.2.2;
    - Plug package versions.
    Do not silently rewrite protocol or semantic version fields solely to make the numbers visually match.

26. `version --json`, release metadata and front-door documentation must not make an agent guess which version describes which layer.

27. Audit install, update and removal experience. A user or AI with the release asset must have a boring documented route to verify checksum, unpack/install, run `version`/`doctor`, locate docs, and remove/replace the installation without relying on hidden developer state.

28. Perform at least one clean Windows release-bundle smoke test outside the development worktree. Use hosted Linux CI for the equivalent Linux artifact if no genuinely native Linux release environment is available.

29. Audit final release publication. Do not claim Tethers 0.5 is published until the tag, hosted release, both intended platform assets and hashes actually exist and the tagged workflow has passed.

30. Finish by updating only living current-truth documentation and release evidence. Historical worker notes/roadmaps remain historical evidence and must not be rewritten to pretend later work existed earlier.

---

## Relevant components

Likely audit surfaces include, but are not limited to:

- `release/tethers-v0.5`
- `docs/CURRENT_GOAL.md`
- `docs/PROJECT_DASHBOARD.md`
- `docs/TETHERS_0_5_RELEASE.md`
- `docs/AGENT_QUICKSTART.md`
- `README.md`
- `QUICKSTART.md`
- `docs/PROJECT_OVERVIEW.md`
- `docs/SECURITY.md`
- `docs/evidence/tethers-0.5-cold-agent-transcript.md`
- `docs/evidence/tethers-0.5-rocket-benchmark.md`
- `scripts/benchmark-tethers.ps1`
- `scripts/benchmark/`
- `scripts/package-tethers-release.ps1`
- `.github/workflows/tethers-v0.5-release.yml`
- Rocket V3 portfolio/reference modules and tests
- `tethers-0.1/host-rust/src/discovery.rs`
- `tethers-0.1/host-rust/src/trail_command.rs`
- existing Plan/Core public boundaries
- `tethers-0.1/host-rust/src/agent_workspace.rs`
- `tethers-0.1/host-rust/src/agent_coding.rs`
- `reference-plugs/tethers-agent-workspace/`
- `reference-plugs/tethers-agent-coding/`
- any new narrowly-scoped Agent Essentials Plug introduced by this audit
- `tethers-0.1/examples/`
- release asset/checksum records.

Do not mutate every listed component automatically. They are inspection surfaces.

---

## Frozen decisions and invariants

- Enc_V2 and ProgramDigest V2 are immutable.
- Frozen V2 byte minimum remains semantic authority.
- The exhaustive Rocket reference engine remains available.
- Fast-path routing changes runtime only, never identity.
- Raw IDs, storage order, benchmark timing and heuristic choices are non-semantic.
- A Plan is a request, not permission.
- Capabilities describe.
- Policies authorise.
- Hosts enforce.
- Trails record.
- Plug conformance is not trust.
- Plug installation is not enablement.
- Enablement is not unlimited scope.
- Supervised provider execution is not a hostile-code sandbox.
- No automatic effectful retry without an existing idempotency proof.
- AI is a client/capability user, not an invisible authority inside Core.
- Core remains application-agnostic.
- Human Tether syntax remains small and canonical.
- Together physical scheduling must not change semantic meaning.
- Existing portable 0.2.2 artifacts/hashes remain immutable historical release artifacts.
- No HQ work is required for 0.5.
- No new canonicalisation research is required for 0.5 unless an actual release-blocking identity defect is discovered.

---

## Acceptance criteria

1. Final-audit work begins from exact base and a clean dedicated worktree.

2. A complete evidence matrix classifies every packet item before implementation and is updated at closeout.

3. Exhaustive Rocket reference mode remains present, callable for bounded verification and unchanged as correctness authority.

4. Portfolio/reference differential tests report zero frozen payload/digest mismatches on the accepted bounded/random/metamorphic corpus.

5. A first-class Benchmarker exists or existing evidence proves an equivalent shipped surface already satisfies the requirement.

6. Benchmarker JSON/human output and before/after comparison are deterministic in structure and include sufficient version/environment/backend context without affecting semantics.

7. Semantic output remains independent of benchmark timing/resource measurements.

8. Cold-agent evidence includes at least one actual harmless bounded capability execution, not merely discovery or conformance.

9. The cold agent can then locate and interpret resulting Trail/receipt evidence using public surfaces only.

10. Side-effect-free public plan/preview capability exists or exact evidence proves an equivalent public surface already exists.

11. Preview cannot execute a provider or accidentally imply authority/execution.

12. Trail querying/receipt ergonomics allow an AI to recover one complete causal execution story without parsing arbitrary internal files.

13. Trail/receipt output remains grounded in the existing Trail store and causal model.

14. At least one real starter Tether Set/example collection exists using only established semantics, including typed work, `together`, and result/follow-on examples where supported.

15. Starter material cannot reasonably leave a fresh agent believing Tethers is fundamentally limited to ALLOW/ASK/DENY.

16. Every Agent Essentials toolbelt category is marked DONE or explicitly DEFERRED-WITH-REASON.

17. Existing workspace/coding provider security, scope and conformance tests remain green.

18. Any new structured/archive/network/SQLite/system capability uses explicit trusted manifests and the normal Plug boundary.

19. Archive/network/SQLite/system-orientation negative safety tests prove their stated bounds.

20. No missing toolbelt feature introduces a general shell escape, hidden credential access or second policy system.

21. Linux/Windows portability claims for Agent Essentials Plugs match actual build/package evidence exactly.

22. Product/language/host/Plug versions are clearly distinguishable in JSON and front-door documentation.

23. Clean-install/update/removal instructions are complete enough to follow from a release asset without repository-local assumptions.

24. Clean Windows bundle smoke evidence is recorded; Linux equivalent is recorded from actual hosted/native evidence only.

25. Final release docs contain exact artifact hashes and tagged workflow evidence rather than placeholders.

26. Final tag and release assets exist remotely before the release is described as published.

27. README, QUICKSTART, release notes, project goal/dashboard and agent quickstart agree on actual shipped behaviour.

28. Historical documentation is not rewritten as though later 0.5 features existed at earlier checkpoints.

29. Full required regression/release gates pass with zero unexplained identity, trust or evidence regressions.

30. Worktree is clean, final local HEAD equals remote HEAD, tag/release identities are recorded, and no unfinished audit item remains silently unclassified.

---

## Required verification

Startup:

- fetch `origin/release/tethers-v0.5`;
- prove expected base `31d5e39a1e3505880e9a98cd8c650b3cf112b16d`;
- use a fresh dedicated worktree;
- require clean Git state;
- run `scripts/check-dev-tools.ps1`;
- run the packet checker and require `control-v1/READY`;
- verify the exact authorised OCaml switch before OCaml work.

Audit first:

- inspect all current 0.5 release evidence;
- inspect current CLI help and JSON commands;
- inspect existing benchmark scripts/output;
- inspect Trail CLI;
- inspect existing planning public surface;
- inspect actual Tether/Tether Set examples;
- enumerate Agent Essentials Plug manifests;
- inspect Windows/Linux packaging;
- inspect current tags/releases/workflow state;
- record DONE/PARTIAL/MISSING/DEFERRED matrix before implementation.

Rocket:

- run portfolio focused tests;
- force reference mode;
- force fallback mode;
- run existing 5,000-case V2 corpus;
- run accepted metamorphic/raw-ID/storage-order corpus;
- require zero payload/digest differences.

Rust/host:

- `cargo fmt --all -- --check`;
- locked check/build/test gates appropriate to changed host/provider surfaces;
- existing discovery/workspace/coding focused suites;
- existing Plug pack/inspect/conform tests;
- new focused tests for every added agent-facing capability;
- serialise known environment-sensitive tests rather than falsely reporting an unsafe parallel gate as clean.

Cold-agent:

Run the actual public journey from a fresh data root:

```text
discover
→ inspect trusted Capability
→ inspect/install/configure bounded Plug as required
→ preview intended work
→ execute one harmless operation
→ inspect result
→ inspect Trail/receipt
```

Record exact commands and machine-readable responses.

Packaging:

- deterministic Windows release package;
- hosted/native Linux package;
- checksum verification;
- clean extraction/install smoke;
- `version --json`;
- `doctor --json`;
- capability discovery smoke;
- documentation presence.

Closeout:

- `dune build @all`;
- `dune runtest --force`;
- relevant full V2/Rocket regression suites;
- `git diff --check`;
- inspect complete base-to-HEAD diff;
- record implementation checkpoint;
- no implementation/test mutations after checkpoint;
- write worker note;
- transition packet to `COMPLETE` or `BLOCKED`;
- rerun task checker in terminal state;
- push normally;
- prove local HEAD == remote HEAD;
- prove clean worktree;
- only then tag/publish if all publication gates are satisfied.

---

## Forbidden changes

- No Enc_V2 change.
- No ProgramDigest V2 change.
- No semantic identity redesign.
- No deletion of exhaustive Rocket reference.
- No resurrection of disproved B3 subtree-rank/local-capacity theories.
- No new generic graph canonicalisation project.
- No Rocq restart for 0.5.
- No HQ implementation.
- No new AI agent framework.
- No hidden LLM judgement in Tethers Core.
- No generic unrestricted shell Capability.
- No ambient credential/environment-secret dumping.
- No unbounded HTTP.
- No unsafe archive extraction.
- No SQLite access outside explicit scope.
- No new policy/authority engine.
- No daemon/server/database merely to support this audit.
- No force push/reset/destructive branch cleanup.
- No mutation of portable 0.2.2 historical assets.
- No unsupported “Linux works” claim.
- No unsupported “release published” claim.
- No broad new feature added solely because it is interesting.

---

## Stop conditions

Stop and finish `BLOCKED` with exact evidence if:

- any fast Rocket route disagrees with the exhaustive reference;
- completing a missing feature requires changing frozen Core/V2 semantics;
- a proposed toolbelt Capability cannot be made meaningfully bounded under the existing trust/scope model;
- a network/archive/SQLite implementation would require unsafe authority expansion;
- a clean-agent real-work journey cannot be completed through public surfaces after two materially different diagnoses;
- required Linux/Windows release infrastructure is unavailable and cannot be honestly evidenced;
- publication credentials/workflow access prevent the actual release;
- an audit item would require a substantial new subsystem rather than release closure.

Do not mark the entire audit BLOCKED merely because one optional toolbelt slice is correctly classified `DEFERRED-WITH-REASON`, provided the worker proves why it is not an accidental omission and the 0.5 release remains coherent.

---

## Expected pre-existing changes

None.

---

## Final decision rule

Do not ask:

> “Can we squeeze one more clever feature into 0.5?”

Ask:

> “Would a fresh AI or technically competent human reasonably expect this in the thing we are claiming Tethers 0.5 is?”

If yes and it is safely achievable through the existing architecture, finish it.

If no, defer it explicitly.

When this packet closes, **stop feature accumulation and release Tethers 0.5**.
