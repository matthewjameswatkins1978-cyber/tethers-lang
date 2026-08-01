# Current Implementation Task

Control contract: `1`
Task: `J18I-F1 - Restore Frozen Installation and Conformance Order`
Owner: `Codex`
Status: `COMPLETE`
Task colour: `Red`
Route: `Codex, frozen lifecycle sequencing correction and roadmap evidence review`
Base branch: `main`
Base commit: `e028b0b80f1a092f5f4198714c0b7a4477323cc8`
Frozen architecture base: `a5fd63593a9d9acd397030ecd2e27b4f318c87fd`
Branch: `luna/j18i-first-plug-kit-roadmap`
Worker note: `docs/worker-notes/2026-08-01-j18i-first-plug-kit-roadmap.md`

## Relevant background and existing behaviour

J18B through J18H are accepted contracts and the Universal Plug architecture is
frozen at the stated architecture base. Tethers 0.2.0 remains the released
baseline, with Tether 0.1 syntax and semantics unchanged. The existing Rust host
already provides typed execution, policy, approval, intent-first dispatch,
canonical outcomes, replay, Result Anchors, event admission, Trail and
supervised MCP/engine sessions, but these modules are largely binary-owned.

## Required behaviour

1. Produce a documentation-only, executable roadmap with exactly six vertical
   milestones.
2. Include a bounded future packet map, current-code inventory, compatibility
   strategy, evidence layers, durable-store sequencing, exclusions and risks.
3. Keep implementation unauthorised and do not create implementation artifacts.
4. Make the first future implementation packet the smallest Milestone 1
   extraction/parity task only.
5. Preserve released 0.2 behaviour and all frozen J18 trust/outcome boundaries.
6. Record evidence-based reuse, extraction, extension, supersession and deferral
   decisions.

## Relevant components

The authoritative roadmap must cover the Rust host modules, current CLI and
PowerShell checks, capability manifests, runtime configurations, MCP fixture
provider, local file provider, Rust/OCaml/integration test surfaces, and the
accepted J18B-J18H architecture documents named by this packet.

## Frozen decisions and invariants

Core remains deterministic and application-agnostic; the host owns trust,
policy, credentials, dispatch, outcomes, replay, admission, conformance and
Trail; providers translate vendor protocols. Action, Query and Anchor are first
classes, while Job, Stream and Human Task remain reserved. There are exactly
three attempted operation outcomes, no automatic retry, and inbound admission
is separate from operation outcomes. Supervision is not isolation, and packages,
signatures, conformance and installation grant no permission.

## Acceptance criteria

1. Exactly eight authorised Markdown paths change relative to the frozen base.
2. J18H is accepted and the architecture freeze is final without altering its
   validation analysis.
3. Current implementation is inventoried with evidence and reuse decisions.
4. Exactly six vertical milestones and a bounded worker-routed packet map exist.
5. P1 is the smallest Milestone 1 extraction/parity task and does not begin
   package, File Tools or security work.
6. Compatibility, exclusions, risks, evidence and durable-store sequencing are
   explicit; no implementation or schema is created.

## Required verification

Run whitespace, exact changed-path, staged-diff and task-packet checks; required
milestone, reuse, boundary, packet and forbidden-claim searches; no-
implementation-artifact verification; and remote main/tag reference checks.

## Forbidden changes

Do not modify or create Rust, OCaml, Cargo, Dune, opam, scripts, tests, fixtures,
manifests, runtime configuration, schemas, packages, providers, MCP transcripts,
credentials, trust stores, keys, signatures, sandboxes, durable store files,
Tether specification, Constitution, release material, tags or GitHub Releases.
Do not begin implementation.

## Stop conditions

Stop on branch, base, ownership, authorised-path or architecture mismatch; a
missing implementation fact that cannot be established from repository evidence;
an unresolved frozen-semantic contradiction; a failed required check; or any
request to implement rather than roadmap. After two materially similar failed
attempts, stop with exact evidence and one smallest unresolved question.

## Expected pre-existing changes

The control-plane commit `e028b0b80f1a092f5f4198714c0b7a4477323cc8` is expected as
the branch base. The worktree is expected to be clean before J18I documentation
changes; the frozen architecture base remains an earlier ancestor used for
changed-path verification.

## Control-plane starting rule

Fetch `origin/main`, switch to `main`, and fast-forward to the commit containing
this J18I control contract. Create `luna/j18i-first-plug-kit-roadmap` from that
updated `main`.

On the task branch, add a `Base commit` line containing the exact control-plane
commit from which the branch was created.

The architecture being roadmapped is frozen exactly at
`a5fd63593a9d9acd397030ecd2e27b4f318c87fd`. The control-plane commit changes
only task authority and is not a new architecture decision. Final changed-path
verification is measured against the frozen architecture base above.

## Objective

Produce the executable implementation roadmap for the first Tethers Plug Kit.

The roadmap must turn the accepted and paper-validated J18B through J18H
architecture into small, reviewable implementation packets that reuse the proven
Tethers 0.2 host rather than replacing it.

This task is documentation and sequencing only.

Do not implement code, schemas, packages, providers, signatures, credentials,
sandboxes, stores, CLI commands, fixtures or tests.

## Accepted and frozen baseline

Released product:

- Tethers 0.2.0.
- `v0.2.0^{}` -> `b5546411661dcbcb53e1cf2538eaec594c6f76f2`.
- Language syntax and semantics remain `0.1`.

Accepted J18 contracts:

- J18B Universal Plug Architecture.
- J18C Socket v1 and MCP 2025-11-25 stdio binding.
- J18D `.tetherplug` package v1.
- J18E capability classes, effects and scopes.
- J18F lifecycle, outcomes, events and conformance.
- J18G security, trust, credentials and sandbox.
- J18H paper validation, verdict `VALIDATED`.

J18B through J18H are accepted. The Universal Plug architecture is finally
frozen at `a5fd63593a9d9acd397030ecd2e27b4f318c87fd`.

Implementation remains unauthorised until Lucy accepts J18I and issues the first
bounded implementation packet.

## Frozen laws the roadmap must preserve

1. Tethers Core remains deterministic and application-agnostic.
2. Tethers coordinates; the host owns trust, policy, credentials, dispatch,
   canonical outcomes, replay, event admission, conformance and Trail.
3. Providers own vendor-specific translation.
4. Socket semantics, protocol binding and byte transport remain distinct.
5. Capability class, effects, scope, policy and outcome remain distinct.
6. Action, Query and Anchor are first-programme classes.
7. Job, Stream and Human Task remain reserved and unimplemented.
8. Attempted operation outcomes remain exactly `succeeded`, `failed`, and
   `uncertain`.
9. Unattempted is not an execution outcome.
10. No automatic retry or restart retry exists.
11. Replay authority remains separate from Trail.
12. Inbound event admission remains separate from operation outcomes.
13. Packages, signatures, conformance and installation grant no operational
    permission.
14. Supervised execution is not hostile-code isolation.
15. Credential-bearing production providers require proven isolation or a
    reviewed host-owned broker.
16. File Tools and bounded PDF Tools are the first reference Plugs.
17. First reference providers are credential-free and have no network access.
18. Arbitrary third-party Plug enablement remains unavailable without proven
    isolated execution.
19. Tether `0.1` syntax does not change.
20. Existing released `0.2.0` behaviour and refs remain intact.

## Canonical output

Create:

`docs/architecture/TETHERS_J18_IMPLEMENTATION_ROADMAP.md`

Begin exactly:

Status: J18I candidate, pending Lucy roadmap review
Architecture freeze: a5fd63593a9d9acd397030ecd2e27b4f318c87fd
Implementation: Not authorised

## Roadmap philosophy

The roadmap must be vertical, not a collection of horizontal rewrites.

Every implementation milestone must:

- deliver one testable end-to-end capability or host boundary;
- reuse proven 0.2 machinery where it is sound;
- leave `main` releasable and old 0.2 behaviour working;
- define exact entry and exit evidence;
- identify one rollback boundary;
- stop when frozen architecture would need revision;
- avoid speculative framework work.

No big-bang host rewrite is permitted.

No milestone may require all future security, marketplace, remote-provider or
reserved-class work before the first useful Plug runs.

## Required current-code inventory

Inspect and record the exact current role, reuse decision and known gap for at
least:

- `tethers-0.1/host-rust/src/main.rs`;
- `tethers-0.1/host-rust/src/lib.rs`;
- `runtime_config.rs`;
- `configured_runtime.rs`;
- `manifest.rs`;
- `trusted_store.rs`;
- `provider.rs`;
- `resolver.rs`;
- `policy.rs`;
- `approval.rs`;
- `dispatch.rs`;
- `stdio_provider.rs`;
- `host_execution.rs`;
- `outcome.rs`;
- `replay_runtime.rs` and `replay_windows.rs`;
- `result_anchor.rs`;
- `event_admission.rs` and `event_queue.rs`;
- `child_process.rs`;
- `engine_stdio.rs`;
- current CLI and check/run commands;
- current capability manifests, runtime configurations and MCP fixture provider;
- current Rust, OCaml and integration test surfaces.

For every component classify:

- reuse unchanged;
- extract behind a stable seam;
- extend under frozen semantics;
- supersede only for the new Plug path;
- defer.

Do not merely list files.

## Known implementation gaps to test

The roadmap must verify and place, not silently assume, at least these gaps:

- no explicit Tethers Socket application seam yet;
- host modules are still largely binary-owned while `lib.rs` exposes only a
  small foundation;
- current runtime configuration directly carries command/args and reviewed
  manifests rather than installed Plug identities;
- current manifest format is not the J18D package and J18E contract;
- current MCP discovery does not yet prove the full J18C pagination and catalogue
  lifecycle contract;
- no `.tetherplug` archive inspector or quarantine installer;
- no semantic package digest/signature/trust-store implementation;
- no immutable installed-Plug registry;
- no sanitized provider environment;
- no conformance evidence store and invalidation engine;
- no explicit supervised-versus-isolated launch profile implementation;
- no Windows Credential Manager profile store;
- no credential delivery or broker, which remains outside the first
  credential-free reference slice;
- no durable external-event admission authority across process restarts;
- no first-class Action/Query/Anchor machine contract for installed Plugs;
- no File Tools or PDF Tools packaged reference provider;
- no Plug inspection, install, enable, disable, conformance and removal user
  workflow;
- no arbitrary third-party hostile-code containment.

If repository inspection disproves or refines a gap, record the evidence.

## Compatibility and migration rule

Freeze this roadmap rule:

- the released 0.2 runtime path remains working;
- Plug Kit implementation is introduced through a new host-owned path and stable
  seam;
- existing runtime configuration, manifests and fixtures are reused as evidence
  and adapters where sound, not silently reinterpreted as `.tetherplug` v1;
- no in-place mutation of user 0.2 configuration is required;
- no legacy path is removed during the first Plug Kit;
- any migration or dual-read format requires a separately reviewed packet;
- the v0.2.0 tag never moves.

## Required six vertical milestones

Define exactly six implementation milestones. Each milestone must include:

- user-visible or machine-visible result;
- exact architectural contracts exercised;
- existing modules reused;
- new modules or stores anticipated, without freezing filenames unnecessarily;
- packet breakdown;
- owner and task colour recommendation;
- dependencies;
- tests and evidence;
- stop conditions;
- rollback boundary;
- explicit exclusions.

### Milestone 1: Socket seam and 0.2 parity

Required outcome:

- expose the existing host execution path behind a reusable application/library
  seam without changing released behaviour;
- introduce the semantic Socket boundary around retained MCP stdio sessions;
- preserve exact lifecycle, request identity, no-batch and no-retry rules;
- implement full J18C discovery pagination, duplicate detection, schema drift and
  catalogue-change invalidation;
- keep current engine, policy, outcome, replay, Result Anchor and Trail behaviour
  passing;
- no package installation yet.

The roadmap must identify the smallest extraction from binary-owned modules and
must forbid a broad `main.rs` rewrite.

### Milestone 2: Package inspection, quarantine and installation-candidate identity

Required outcome:

- inspect `.tetherplug` without execution;
- validate strict archive paths, `plug.json`, payload index and semantic digest;
- select the exact Windows x86_64 payload;
- extract only into quarantine;
- create immutable host-owned installation-candidate material in quarantine;
- keep package identity, candidate identity, provider identity and capability
  identity distinct;
- record candidate/quarantine evidence only, never an installed record, active
  binding, developer trust, installation approval or operational launch;
- no provider invocation from Downloads, source ZIP or quarantine.

The roadmap must place archive-bomb, traversal, symlink/junction/reparse,
case-collision, duplicate-entry and TOCTOU tests.

### Milestone 3: Trust, launch and conformance gate

Required outcome:

- Ed25519 package verification and host-owned publisher trust;
- explicit trust, developer-mode and revocation state;
- pre-launch payload digest revalidation;
- exact executable and argument launch with no shell or PATH lookup;
- environment constructed from scratch;
- retained Job Object supervision and bounded resources;
- visible supervised-profile labelling;
- conformance run only from quarantine using the accepted test profile, followed
  by provider shutdown and immutable evidence review;
- conformance evidence pinned to package, payload, manifest, provider, Socket,
  protocol, host, platform and suite versions;
- passing conformance leaves the candidate uninstalled and non-operational;
- only after P7/P8/P9 may P10 review/refuse trust and conformance, explicitly
  approve installation, atomically create immutable installed identity and exact
  disabled bindings, and record the Plug present but disabled;
- no invocation or event admission is allowed before explicit enablement;
- Credential Manager work is deferred/optional; first providers are
  credential-free and receive no credential delivery.

Do not claim AppContainer or hostile-code isolation in this milestone.

### Milestone 4: File Tools Action/Query vertical slice

Required outcome:

- one self-contained Windows reference provider packaged as `.tetherplug`;
- no credentials and no network;
- bounded file read and metadata Query operations;
- bounded file move Action with exact source/destination, overwrite refusal and
  path-scope enforcement;
- complete path from package inspection through quarantine validation,
  conformance, evidence review, installation approval, installed-disabled exact
  binding, explicit enablement, discovery, policy, durable intent, one
  invocation, canonical outcome, replay terminal, Result Anchor and Trail;
- clear unattempted, failed and uncertain demonstrations;
- deterministic disposable filesystem conformance fixtures;
- first runnable Plug Kit demonstration.

Capability names and machine schemas must be frozen only in dedicated
implementation packets, not invented by J18I prose.

### J18I-F1 frozen installation and conformance order

The frozen lifecycle is: package received; archive inspected without execution;
package validated; payload extracted into quarantine; manifests and compatibility
validated; test configuration created; provider launched in the accepted test
profile; conformance from quarantine; provider stopped; conformance evidence
reviewed; installation approved; exact bindings created; Plug present but
disabled; explicit enablement; operational use. Conformance success does not
approve, install, bind or enable, and no active binding exists before approval.

M2 owns only candidate/quarantine identity and registry evidence. It is not an
installed Plug registry. M3 owns the candidate-to-installed-disabled transition:
P7 trust/signature, P8 quarantine launch profile, P9 host-orchestrated
conformance, then P10 installation approval. P10 depends on P6/P7/P8/P9 and
creates the immutable installed identity with exact disabled provider/capability
bindings. M4 owns first explicit File Tools enablement. P11-P13 depend on P10
where installed state is required. Candidate/quarantine registry, installed
registry, publisher trust, conformance evidence, credential metadata, replay,
event admission and Trail remain separate authorities; the transition is atomic
and audited. The roadmap retains exactly 20 future packets and routing totals of
5 Luna/OpenCode, 5 DeepSeek and 10 Codex.

Lifecycle ordering: conformance from quarantine; conformance evidence review; installation approval; Plug present but disabled; explicit enablement.

### Milestone 5: Durable local Anchor and lifecycle completion

Required outcome:

- host-owned durable external-event admission authority separate from operation
  replay, Trail and the existing per-invocation J11 gate;
- one bounded local reference source with provider-persisted stable event identity;
- admitted, duplicate, identity-conflict, rejected and admission-uncertain paths;
- acknowledgement only after durable admission;
- root Anchor generation 0 and existing generation 0 through 8 causal limits;
- install, enable, disable, restart and removal behaviour for the source;
- no network listener;
- no raw unbounded stream;
- no operation outcome created merely by event admission.

The roadmap must require a separate design packet for the exact durable store and
reference-source event identity before code.

### Milestone 6: PDF Tools and first Plug Kit release gate

Required outcome:

- bounded PDF extraction Query provider packaged as `.tetherplug`;
- PDF treated as hostile parser input;
- exact input materialization, page/byte/time/memory/output limits and disposable
  scratch;
- no network and no credentials;
- supervised mode labelled for bounded reference/competition use only;
- production use refused without proven isolation;
- user workflow for inspect, install, conformance, approve, enable, list, disable
  and remove;
- retained 0.2 path regression coverage;
- complete release evidence for the first Plug Kit slice;
- no claim of arbitrary third-party safety.

The roadmap must identify the final Red review and Git publication gate but must
not create a release or version number.

## Packet map

Within the six milestones, define a sequence of bounded future work packets.

Every packet must record:

- stable packet ID;
- objective;
- dependencies;
- proposed owner;
- Green, Amber or Red classification;
- expected code/document/test areas;
- exact acceptance evidence;
- stop conditions;
- whether it may modify schemas;
- whether it may change public CLI;
- rollback or revert boundary.

Prefer packets that fit one focused agent run and one review.

Do not authorize any packet merely by listing it.

The first implementation packet after J18I acceptance must be the smallest
Milestone 1 extraction/parity task. It must not begin package parsing, File Tools
or security work at the same time.

## Worker routing

Use this default routing unless inspection gives a stronger reason:

- Luna on OpenCode: bounded Green implementation, fixtures, documentation and
  ordinary Amber work under frozen interfaces;
- DeepSeek Pro V4: thicker middle implementation and cross-module integration
  requiring Lucy review;
- Codex Terra High: Red gates, Windows security/process boundaries, archive and
  path attacks, cryptography/trust, durable storage migrations, Git surgery and
  final release verification;
- Lucy: architecture guard, packet design, review and final verdict;
- Matthew: final product authority.

Do not route every task to the strongest worker.

## Test and evidence strategy

Define the required evidence layers:

1. pure unit tests;
2. parser and duplicate-key tests;
3. archive/path adversarial tests;
4. real Windows child-process and Job Object tests;
5. MCP transcript and pagination tests;
6. package/install/trust lifecycle tests;
7. policy/scope/approval tests;
8. durable replay and event-admission restart tests;
9. Result Anchor and Trail ordering tests;
10. reference-provider conformance tests;
11. full Rust and OCaml regression suites;
12. end-to-end File Tools and PDF Tools demonstrations;
13. clean-machine or isolated test-host evidence before release.

For each milestone state which layers are mandatory.

Do not freeze performance numbers without a measurement packet. Use bounded
host-owned limits and require measurement before choosing final defaults.

## Storage and schema sequencing

Identify separate implementation authorities for:

- installed Plug registry;
- publisher trust store;
- conformance evidence;
- credential profile metadata;
- operation replay;
- external-event admission;
- Trail.

Do not merge these because they are all durable.

For every new durable store require:

- schema/version design packet;
- atomicity and crash-recovery model;
- permissions and confidentiality review;
- migration/rollback plan;
- corruption behaviour;
- tests proving no automatic retry or false admission.

## First-slice exclusions

Keep outside the first Plug Kit implementation:

- public registry or marketplace;
- automatic download or update;
- remote HTTP providers;
- OAuth implementation;
- general network egress;
- network listeners;
- general credential-bearing production integrations;
- arbitrary third-party enablement;
- AppContainer completion unless separately authorised as a security track;
- Jobs;
- Streams;
- Human Tasks;
- long-running renderers;
- live sensors;
- printers and MIDI;
- smart locks;
- industrial actuation;
- unrestricted shell;
- interpreter-backed production providers;
- dependency installation;
- plug-to-plug communication;
- Tether language changes.

## Risk register

Include at least:

- accidental 0.2 regression during module extraction;
- package parser and archive extraction attack surface;
- identity/digest conflation;
- false trust from signing or conformance;
- supervised mode being mistaken for isolation;
- Windows path and reparse escape;
- environment or credential leakage;
- stale live discovery;
- provider process survival;
- outcome misclassification after invocation;
- replay-terminal or Result Anchor publication failure;
- external-event identity conflict;
- durable-store corruption;
- first-slice scope growth;
- competition deadline pressure causing security shortcuts.

For each risk give prevention, detection, containment and owner.

## Architecture and status alignment

Update:

`docs/architecture/TETHERS_J18_PAPER_VALIDATION.md`

Change only its status preamble and final-freeze wording so that it records:

Status: Accepted J18H paper validation
Accepted by Lucy: 2026-08-01
Verdict: VALIDATED
Architecture freeze: Final
Implementation: Not authorised

Do not alter the validated integration analysis.

## Decision log

Prepend to `docs/DECISIONS.md`:

`## 2026-08-01: J18 architecture frozen and implementation roadmap opened`

Record concisely:

1. J18H is accepted with verdict `VALIDATED`.
2. J18B through J18H form the frozen Universal Plug architecture.
3. Tether `0.1` semantics remain unchanged.
4. J18I is roadmap-only and authorises no implementation.
5. The first Plug Kit remains credential-free File Tools and bounded PDF Tools.
6. Action, Query and Anchor are first-programme classes.
7. Job, Stream and Human Task remain reserved.
8. The six-milestone vertical implementation route is being planned.
9. Existing 0.2 behaviour remains supported.
10. Only a later explicit packet may start implementation.

## Current-state updates

Update:

- `docs/CURRENT_GOAL.md`;
- `docs/PROJECT_DASHBOARD.md`;
- `docs/TASK_QUEUE.md`;
- `docs/CURRENT_CLINE_TASK.md`.

Required state:

- J18B through J18H accepted;
- architecture frozen at `a5fd63593a9d9acd397030ecd2e27b4f318c87fd`;
- J18I active and pending Lucy roadmap review;
- roadmap candidate status recorded;
- implementation remains unauthorised;
- the first implementation packet follows only after J18I acceptance;
- v0.2.0 refs remain unchanged.

## Worker note

Create:

`docs/worker-notes/2026-08-01-j18i-first-plug-kit-roadmap.md`

Use headings:

- Task
- Changes
- Frozen architecture inspected
- Current implementation inventory
- Reuse and extraction findings
- Compatibility strategy
- Six milestones
- Packet map
- Worker routing
- Test and evidence plan
- Durable stores and schemas
- Risks
- Tool bootstrap
- Evidence
- Discoveries
- Remaining questions
- Next action
- References

Record resolved tool paths and versions, exact repository files inspected,
current implementation gaps, milestone/packet counts, and confirmation that no
implementation or schema changed.

## Authorised changed paths

Exactly eight paths relative to frozen architecture base
`a5fd63593a9d9acd397030ecd2e27b4f318c87fd`:

1. `docs/architecture/TETHERS_J18_IMPLEMENTATION_ROADMAP.md`
2. `docs/architecture/TETHERS_J18_PAPER_VALIDATION.md`
3. `docs/DECISIONS.md`
4. `docs/CURRENT_GOAL.md`
5. `docs/PROJECT_DASHBOARD.md`
6. `docs/TASK_QUEUE.md`
7. `docs/CURRENT_CLINE_TASK.md`
8. `docs/worker-notes/2026-08-01-j18i-first-plug-kit-roadmap.md`

No other path may change.

## Forbidden changes

Do not modify or create:

- Rust;
- OCaml;
- Cargo;
- Dune;
- opam;
- scripts;
- tests;
- fixtures;
- manifests;
- runtime configuration;
- JSON Schema;
- `.tetherplug` or ZIP files;
- providers;
- MCP transcripts;
- credentials;
- trust stores;
- keys or signatures;
- AppContainer profiles;
- durable store files;
- Tether specification;
- Constitution;
- release notes;
- tags;
- GitHub Releases.

Do not begin implementation.

## Preflight

Run:

```text
git fetch origin --prune --tags
git switch main
git pull --ff-only origin main
git rev-parse HEAD
git rev-parse v0.2.0^{}
git status --short
```

Require the worktree to be clean and the peeled release tag to remain:

`b5546411661dcbcb53e1cf2538eaec594c6f76f2`

Confirm the fetched `docs/CURRENT_CLINE_TASK.md` contains this J18I contract and
the frozen architecture base.

Create:

```text
git switch -c luna/j18i-first-plug-kit-roadmap
```

## Required verification

Before staging:

```text
git diff --check
git diff --name-only
git status --short
```

Require exactly the eight authorised paths relative to the frozen architecture
base.

Stage all intended files, then run:

```text
git diff --cached --check
git diff --cached --name-only
```

Require no whitespace errors and exactly eight staged paths.

Run the task-packet checker.

Required milestone search:

```text
rg -n "^## Milestone [1-6]|Socket seam|Package inspection|Trust.*conformance|File Tools|Durable.*Anchor|PDF Tools" docs/architecture/TETHERS_J18_IMPLEMENTATION_ROADMAP.md
```

Require exactly six milestone headings and all six required subjects.

Required reuse search:

```text
rg -n "reuse unchanged|extract behind|extend under|supersede|defer|main.rs|runtime_config|configured_runtime|stdio_provider|host_execution|replay|result_anchor|event_admission|child_process" docs/architecture/TETHERS_J18_IMPLEMENTATION_ROADMAP.md
```

Required boundary search:

```text
rg -n "no automatic retry|operation outcomes|event admission|supervised.*not.*isolation|credential-free|third-party|Job|Stream|Human Task|0.1" docs/architecture/TETHERS_J18_IMPLEMENTATION_ROADMAP.md
```

Required packet search:

```text
rg -n "packet ID|dependencies|owner|Green|Amber|Red|acceptance evidence|stop conditions|rollback|schema|public CLI" docs/architecture/TETHERS_J18_IMPLEMENTATION_ROADMAP.md
```

Forbidden implementation claim search:

```text
rg -n "implemented in J18I|J18I adds code|Plug Kit is now implemented|third-party packages are supported|AppContainer is implemented|credentials are securely delivered|automatic retry" docs/architecture/TETHERS_J18_IMPLEMENTATION_ROADMAP.md docs/CURRENT_GOAL.md docs/PROJECT_DASHBOARD.md
```

Any positive implementation claim fails. Explicit negations are permitted.

Confirm no implementation artifact changed:

```text
git diff --name-only a5fd63593a9d9acd397030ecd2e27b4f318c87fd...HEAD | rg "\.(rs|ml|mli|toml|opam|json|ps1|sh|key|pem|pfx|p12|sig|tetherplug|zip)$"
```

Require no result.

Verify refs:

```text
git ls-remote origin refs/heads/main refs/tags/v0.2.0 refs/tags/v0.2.0^{}
```

## Acceptance criteria

1. Exactly eight authorised Markdown paths change relative to the frozen base.
2. J18H is marked accepted without altering its validation analysis.
3. The architecture freeze is recorded as final.
4. Current implementation is inventoried with evidence, not assumptions.
5. Reuse, extraction, extension, supersession and deferral decisions are clear.
6. Exactly six vertical milestones are defined.
7. Every milestone has outcome, dependencies, evidence, stop and rollback gates.
8. The packet map is bounded and worker-routed.
9. The first implementation packet is a Milestone 1 extraction/parity task only.
10. Existing 0.2 compatibility and tag immutability are preserved.
11. File Tools is the first runnable reference Plug.
12. PDF Tools remains bounded reference/competition work.
13. Durable Anchor admission is separately sequenced.
14. Trust, signing, conformance and supervised-mode honesty are sequenced.
15. Credential delivery and arbitrary third-party support remain outside the
    first credential-free slice.
16. Jobs, Streams and Human Tasks remain reserved.
17. No schema, CLI, code, fixture, package or provider is implemented.
18. All checks pass, worktree is clean and branch is pushed normally.

## Commit and publication boundary

Create one commit:

`docs: define first plug kit implementation roadmap`

Push only:

`luna/j18i-first-plug-kit-roadmap`

Do not push `main`, tags or releases.

## Completion report

Begin exactly:

`COMPLETE - READY_FOR_LUCY_ROADMAP_REVIEW`

Report:

1. branch and commit SHA;
2. exact control-plane base commit;
3. tools and versions;
4. exact eight changed paths;
5. final architecture-freeze status;
6. current implementation inventory and key reuse decisions;
7. six milestones;
8. packet count and routing distribution;
9. first implementation packet recommendation;
10. compatibility strategy;
11. test/evidence strategy;
12. durable-store/schema sequencing;
13. first-slice exclusions;
14. risk register highlights;
15. unresolved questions;
16. J18H status and decision-log updates;
17. required and forbidden search results;
18. diff, staged-diff and packet checks;
19. published main and tag verification;
20. clean worktree and ahead/behind;
21. confirmation no implementation or schema changed;
22. smallest next action.

On failure begin exactly:

`BLOCKED`

Stop after the report.
