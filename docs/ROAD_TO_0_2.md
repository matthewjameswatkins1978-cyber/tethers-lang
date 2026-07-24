# Tethers Road to 0.2

Status: release programme  
Owner and final product authority: Matthew  
Programme architect and Red-gate reviewer: Lucy/Codex  
Date: 24 July 2026

## 1. The decision

Tethers 0.2 will be the first usable **standalone local runtime slice**.

It will prove this complete route:

```text
configured local provider
→ verified manifest admission
→ live capability projection
→ external Anchor plus immutable Facts
→ deterministic Tethers Plan
→ allow / ask / deny / unavailable policy
→ intent-first serial dispatch with no automatic retry
→ succeeded / failed / uncertain outcome
→ complete causal Trail
→ standard result Anchor
→ bounded serial follow-up evaluation
```

This is deliberately narrower than the whole Tethers vision. It is large
enough to be real, but small enough to finish and trust.

Tethers 0.2 is **not**:

- Lantern Keeper's memory store or retrieval system;
- HQ or a graphical editor;
- a package marketplace;
- automatic provider discovery;
- a remote or multi-user service;
- Streamable HTTP, OAuth or network listeners;
- parallel execution;
- automatic retries or compensation;
- a general programming language;
- an AI agent framework;
- simultaneous integration with ChatGPT, Codex and Cline.

Lantern Keeper may use this runtime later. It does not define the 0.2 release
boundary.

## 2. Why this is the right boundary

Tethers 0.1 has already proved the language, deterministic evaluator, planner
protocol, reference host round trip, OCaml-owned MCP planner, validation tool,
manifest verification, trusted manifest store, provider admission, capability
projection, planner-to-dispatch manifest pinning, conservative dispatch proof
boundary, output validation and known-outcome result Anchors.

The remaining gap is not another language feature. The gap is turning those
parts into one honest local runtime:

- policy must have all four outcomes;
- `ask` must be tied to the exact proposed Action, not vague approval;
- deadlines must distinguish failure from uncertainty;
- uncertainty must produce the correct Trail and result Anchor;
- generated events need a bounded queue, deduplication and a causal-depth limit;
- restart and replay must not quietly duplicate effects;
- one command path must let a person check and run the slice;
- the complete failure matrix must be demonstrated on the real Windows setup.

Calling anything less 0.2 would give the project a version number without a
complete behavioural claim.

## 3. Release acceptance

0.2 is accepted only when all of the following are demonstrated from the native
Windows checkout:

1. A configured deterministic stdio MCP provider is admitted only through its
   verified manifest and live host binding.
2. One external Anchor and immutable Fact snapshot produce the expected
   deterministic Plan and evaluation Trail.
3. A bridge-backed Action carries exact capability version, manifest digest and
   provider identity pins from planning to dispatch.
4. `allow`, `ask`, `deny` and `unavailable` each have distinct, tested,
   fail-closed behaviour.
5. An approved Action is attempted serially and at most once unless a later
   release introduces separately proved retry semantics.
6. Intent is durably recorded before an effectful call.
7. Success output is schema-validated before success is recorded.
8. Executor failure and output-validation failure produce `failed`.
9. A deadline after dispatch where the effect may have occurred produces
   `uncertain`, never a guessed failure and never an automatic retry.
10. Attempted calls produce exactly one standard result Anchor:
    `capability.succeeded`, `capability.failed` or `capability.uncertain`.
11. Unattempted Actions—Ask pending, Deny, Unavailable, stale pin, invalid
    arguments or failed intent write—produce no false result Anchor.
12. Generated result Anchors are queued serially, duplicate event IDs are
    rejected, and causal generation stops visibly at the fixed depth of eight.
13. Replaying a completed execution identity does not repeat an external
    effect.
14. The Trail distinguishes evaluation, permission, intent, attempt, outcome,
    result event and audit failure without claiming more than is known.
15. A local check/run/inspect route completes the slice without manually
    assembling internal test objects.
16. Focused tests, the complete Rust suite, engine fixtures, MCP transcripts,
    host integration scripts, demo, OCaml build and Git checks all pass.
17. The release is documented, reproducible from a clean checkout and signed
    off independently.

## 4. Programme rules

This file is the stable release map. It does not replace the active task packet.

Only one implementation job is active at a time. Immediately before a job
starts, the task compiler turns that job into
`docs/CURRENT_CLINE_TASK.md` using the live commit, dirty state, exact files and
current test evidence. Pre-writing all future packets would make their base
commits and code assumptions stale.

Every job follows:

```text
programme job
→ live bounded task packet
→ one named implementation owner
→ changes and objective evidence
→ worker note
→ independent verdict where required
→ dashboard and programme update
```

No job is complete without its named worker note under `docs/worker-notes/`.
The worker note is the technical return journey; Matthew is not the message bus.

### Roles

| Role | Current owner | Use |
| --- | --- | --- |
| Product authority | Matthew | Product intent, consequential trade-offs and final judgement |
| Programme architect | Lucy/Codex | Release boundary, Red designs, contradictions and milestone sign-off |
| Task compiler and routine verifier | Copilot | Inspect live state, compile one packet, verify evidence and route the next job |
| Green implementation owner | Cline/DeepSeek | Narrow existing-pattern changes with exact tests and no architectural judgement |
| Amber implementation owner | Copilot in an isolated worktree | Multi-file Rust/OCaml work with settled behaviour |
| Red implementation owner | Chosen after design | Implements only after Lucy/Codex freezes the contract; cannot sign off its own work |

Risk colour describes the work, not the vendor. Routing may change when measured
reliability, cost or availability changes.

### Standard evidence

Every implementation packet names focused checks, then the relevant portion of
this regression gate:

```powershell
Set-Location "D:\The Next Thing\Tethers Lang\tethers-0.1"
Set-Location host-rust
cargo fmt --check
cargo test
Set-Location ..
pwsh -NoProfile -File scripts/check-fixtures.ps1
pwsh -NoProfile -File scripts/test-engine.ps1
pwsh -NoProfile -File scripts/test-mcp-transcripts.ps1
pwsh -NoProfile -File scripts/test-host-denial.ps1
pwsh -NoProfile -File scripts/test-host-execution-failure.ps1
pwsh -NoProfile -File scripts/demo.ps1
Set-Location engine-ocaml
opam exec -- dune build
Set-Location ..
git diff --check
git status --short
```

A worker must not claim an unrun command passed. The packet compiler may omit
irrelevant expensive checks only when it names the reason and retains every
check needed to establish the job's truth.

## 5. Dependency timeline

This is a dependency schedule, not a promise of calendar dates. The elapsed
ranges assume one active implementation owner, prompt handoffs, no major design
contradiction and roughly one bounded job per working session.

| Stage | Jobs | Expected elapsed active work | Gate |
| --- | --- | ---: | --- |
| A. Establish control baseline | J00–J02 | 1–2 days | Project-control PR merged; current Red milestone signed off |
| B. Complete effective policy | J03–J05 | 3–6 days | All four policy outcomes and exact one-shot approval proven |
| C. Complete honest execution | J06–J09 | 4–8 days | Timeout, uncertainty, replay and Trail semantics proven |
| D. Complete event continuation | J10–J11 | 3–5 days | Standard result events queue safely to depth eight |
| E. Make the slice operable | J12–J14 | 4–7 days | Check/run/inspect path and real provider demonstration pass |
| F. Harden and release | J15–J17 | 3–6 days | Clean-machine proof, independent sign-off and 0.2.0 tag |

Likely total: **18–34 active working days**. The code may move faster. The
release should not be dated from optimistic model speed; trust-boundary review,
Windows verification and correction loops dominate the schedule.

No stage starts merely because the preceding code exists. Its gate must have a
recorded verdict.

## 6. Job register

### J00 — Prove the project-control checker

- Risk: Green
- Owner: Copilot
- Depends on: current PR #1 at `codex/project-control-loop`
- Outcome: exact PowerShell output and process exit code from the native Windows
  checkout.
- Scope: no code edits, no merge, no next task.
- Acceptance: checker output is present and exit code is `0`; branch and
  worktree remain clean.

Give Copilot:

```text
On codex/project-control-loop, run:
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1

Return the exact stdout/stderr and the numeric exit code. Also return:
git status --short --branch
git rev-parse HEAD

Do not edit files, merge PR #1 or begin another task.
```

If the checker fails, Copilot returns the exact failure. Lucy/Codex compiles one
small correction; Copilot must not improvise a repair.

### J01 — Accept and merge the control loop

- Risk: Amber process change
- Owner: Lucy/Codex
- Verifier: Copilot's J00 evidence
- Depends on: J00
- Outcome: PR #1 is accepted and merged without runtime-semantic changes.
- Acceptance: the merged commit is on `main`; the Windows checker passes on
  merged `main`; dashboard and instructions agree on the one-owner/worker-note
  loop.

Give Lucy/Codex:

```text
Use PR #1 and the J00 worker evidence. Confirm the project-control change is
internally consistent, merge it if and only if the checker passed, then verify
the merged main branch. Do not begin runtime implementation.
```

### J02 — Sign off the completed planner-to-dispatch pin milestone

- Risk: Red
- Owner and verifier: Lucy/Codex
- Depends on: J01
- Outcome: independent verdict on the legacy completed Red task at
  `docs/CURRENT_CLINE_TASK.md`.
- Required context: the packet; base `9ed81b8...`; implementation diff through
  the current implementation checkpoint; `CAPABILITY_BRIDGE.md`; focused pin,
  stale-D1/D2 and version-representation tests; full regression evidence.
- Acceptance: one verdict—`SIGNED OFF` or `NOT SIGNED OFF`—with exact defects
  if rejected. Dashboard and cadence baseline are updated. No fake retrospective
  worker note is created for the legacy task.

Give Lucy/Codex:

```text
Review the completed Red planner-to-dispatch manifest-pin milestone described
by docs/CURRENT_CLINE_TASK.md. Inspect the live implementation and evidence,
especially pre-evaluation projection, opaque digest copying, D1/D2 stale-plan
denial, explicit version mapping and non-bridge compatibility. Record SIGNED
OFF or NOT SIGNED OFF, update the dashboard/cadence facts, and stop.
```

### J03 — Freeze the four-outcome policy contract

- Risk: Red design
- Owner: Lucy/Codex
- Depends on: J02 signed off
- Outcome: a focused decision record defining `allow`, `ask`, `deny` and
  `unavailable` as host-owned outcomes.
- Must settle:
  - inputs to effective policy;
  - precedence and fail-closed behaviour;
  - exact Action/argument/manifest/provider digest bound to Ask;
  - one-shot approval lifetime and consumption;
  - no AI self-approval;
  - no dispatch and no result Anchor while Ask remains unresolved;
  - Trail entries for every outcome.
- Excludes: GUI prompts, remote accounts, standing-policy editor.
- Acceptance: decision and test matrix are precise enough for an implementation
  worker to make no policy choice.

Give Lucy/Codex:

```text
Design J03 from docs/ROAD_TO_0_2.md. Read only the current policy/dispatch code,
the Constitution, CAPABILITY_BRIDGE.md and the canonical architecture sections
named by the job. Freeze the smallest fail-closed four-outcome policy contract
and its test matrix. Do not implement it.
```

### J04 — Implement effective policy resolution

- Risk: Amber
- Owner: Copilot in an isolated worktree
- Depends on: J03
- Outcome: deterministic effective policy resolver returning all four outcomes
  without executing anything.
- Acceptance: focused tests cover precedence, exact scope, provider absence,
  stale binding, malformed input and deterministic repeat. Existing denial and
  dispatch tests remain green.

Matthew normally gives Copilot only:

```text
/next-tethers-task
```

Copilot must compile J04 from the programme and J03 decision, then show Matthew
the one-paragraph packet summary. After approval it owns implementation and the
worker note. It does not redesign J03.

### J05 — Implement exact one-shot Ask resolution

- Risk: Red boundary, Amber implementation after design
- Architect: Lucy/Codex
- Implementation owner: Copilot in an isolated worktree
- Depends on: J04
- Outcome: unresolved Ask stops safely; an explicit one-shot decision bound to
  the exact Action proof can resume or deny that Action once.
- Acceptance:
  - changed arguments, manifest, provider, Action ID or proof digest invalidate
    approval;
  - approval cannot be reused;
  - denial never calls the executor;
  - pending Ask emits no success/failure/uncertain result Anchor;
  - Trail distinguishes requested, approved, denied, stale and consumed.

Give Lucy/Codex at the gate:

```text
Review J04 and compile the frozen J05 approval-token/resume contract. Make every
trust input and invalidation rule explicit. Do not implement.
```

After that contract is recorded, use `/next-tethers-task` in Copilot.

### J06 — Freeze deadline and outcome classification

- Risk: Red design
- Owner: Lucy/Codex
- Depends on: J05
- Outcome: one execution state machine defining when an Action is unattempted,
  succeeded, failed, uncertain or known-outcome-with-audit-failure.
- Must settle:
  - deadline ownership and start point;
  - timeout before dispatch versus after possible dispatch;
  - provider crash, EOF, malformed response and cancellation;
  - success followed by invalid output;
  - final Trail write failure after known outcome;
  - absolutely no automatic retry.
- Acceptance: every transition has one Trail/result-event expectation and no
  state disguises uncertainty as failure.

Give Lucy/Codex:

```text
Design J06 from the current dispatch, provider, output-validation and Trail
code. Produce a complete transition table and focused acceptance matrix.
Preserve no-retry semantics. Do not implement.
```

### J07 — Implement deadlines and `uncertain`

- Risk: Amber
- Owner: Copilot in an isolated worktree
- Depends on: J06
- Outcome: provider calls obey the frozen local deadline contract and return
  honest outcome classification.
- Acceptance: deterministic test providers prove pre-dispatch failure,
  ordinary executor failure, timeout with possible effect, malformed success,
  valid success and no retry. Exact call counts are asserted.

Use `/next-tethers-task` in Copilot. Lucy/Codex reviews the final transition
diff because J07 implements a Red contract.

### J08 — Emit `capability.uncertain`

- Risk: Green if it follows the existing result-anchor pattern
- Owner: Cline/DeepSeek
- Depends on: J07 signed off
- Outcome: exactly one standard uncertain Result Anchor after an attempted call
  with uncertain outcome.
- Acceptance: correlation, causation, capability pins and structured error are
  correct; Ask/Deny/Unavailable/preparation failure emit none; existing
  succeeded/failed cases remain unchanged.

Give Cline:

```text
/tethers-task.md
```

The packet must name only the result-anchor module, directly related tests and
the J06/J07 worker notes. Cline writes its worker note and stops.

### J09 — Add durable replay protection

- Risk: Red persistence boundary
- Architect: Lucy/Codex
- Implementation owner: Copilot
- Depends on: J07
- Outcome: a completed execution/action identity cannot repeat its external
  effect after process restart.
- Design requirement: reuse the smallest trustworthy host-owned persistence
  mechanism already supported by the intent/outcome Trail. Do not invent a
  general database or enable retries.
- Acceptance: restart/replay tests prove completed identities do not dispatch
  twice; uncertain identities are surfaced for human resolution rather than
  retried; corrupted or missing persistence fails closed.

Give Lucy/Codex:

```text
Design J09 as the smallest durable replay guard needed by 0.2. Inspect the live
Trail and dispatch persistence code. Separate duplicate suppression from retry,
and specify restart/corruption behaviour. Do not implement.
```

Then use `/next-tethers-task` in Copilot after the design is accepted.

### J10 — Queue generated result events serially

- Risk: Amber
- Owner: Copilot
- Depends on: J08 and J09
- Outcome: standard Result Anchors enter a host-owned FIFO queue and are
  evaluated one at a time in declared order.
- Acceptance: stable ordering, preserved correlation/causation, no recursive
  call-stack execution, and no parallel Actions or events.

Use `/next-tethers-task` in Copilot.

### J11 — Add event deduplication and causal depth eight

- Risk: Red safety boundary, Amber implementation after contract
- Architect: Lucy/Codex
- Implementation owner: Copilot
- Depends on: J10
- Outcome: duplicate event IDs are rejected and generation greater than eight
  stops visibly with a loop-limit Trail failure.
- Acceptance: focused cycles prove no ninth-generation evaluation, duplicates
  produce no Actions, independent event chains remain unaffected, and restart
  behaviour matches the J09 persistence decision.

Give Lucy/Codex:

```text
Freeze J11 event identity, duplicate and generation-limit semantics using the
canonical architecture's fixed maximum depth of eight. Specify exact Trail
behaviour and persistence interaction. Do not implement.
```

Then route the frozen packet to Copilot.

### J12 — Freeze the minimal runnable Tether Set/config boundary

- Risk: Red format decision
- Owner: Lucy/Codex
- Depends on: J11
- Outcome: the smallest local configuration needed to select ordered Tethers,
  exact capability requirements, provider binding and local policy for one
  runnable set.
- Constraint: reuse existing representations where possible. Do not build
  package management, installation, signing, marketplace metadata or secret
  storage.
- Acceptance: one canonical fixture can be checked without starting effects;
  missing/wrong capability versions fail before execution; secrets cannot be
  embedded.

Give Lucy/Codex:

```text
Design J12 as a deliberately tiny local runnable-set/config contract. Reuse the
existing requirement, provider and policy types. It must support the 0.2 demo
without becoming a package format. Do not implement.
```

### J13 — Implement local `check`, `run` and `trail` routes

- Risk: Amber
- Owner: Copilot
- Depends on: J12
- Outcome:
  - `check` validates source, set requirements, provider admission and effective
    availability without dispatch;
  - `run` submits one explicit Anchor/Facts input through the real slice;
  - `trail` prints or locates the causal record for one execution.
- Constraint: PowerShell-friendly native Windows path; no daemon manager, GUI
  or remote transport.
- Acceptance: stable exit codes, useful errors, protocol-only stdout where
  required, and no hidden I/O inside Tethers Core.

Use `/next-tethers-task` in Copilot.

### J14 — Prove one complete real local scenario

- Risk: Amber integration
- Owner: Copilot
- Independent verifier: Lucy/Codex
- Depends on: J13
- Outcome: one deterministic local stdio provider capability is admitted,
  planned, allowed, attempted once, output-validated, recorded and converted
  into a result Anchor through the public command path.
- Required negative runs: malformed manifest, unavailable provider, Ask
  pending/approved/denied, Deny, stale pin, intent-write failure, executor
  failure, invalid output, timeout/uncertain, duplicate replay and loop depth.
- Acceptance: evidence is captured as reproducible scripts/fixtures, not a chat
  description or hand-edited Trail.

Give Copilot:

```text
Compile and run J14 from docs/ROAD_TO_0_2.md. This is an integration-proof job,
not permission to add features or repair unrelated code. Produce reproducible
fixtures/scripts, exact results and a worker note, then stop for Codex review.
```

### J15 — Consolidate the 0.2 failure matrix

- Risk: Amber verification
- Owner: Copilot
- Depends on: J14 signed off
- Outcome: one discoverable Windows verification entry point covers the release
  acceptance matrix and reports every case separately.
- Acceptance: failures identify the exact case; deterministic cases repeat;
  Actions and Trail arrays retain order; no test claims an unavailable external
  environment passed.

Use `/next-tethers-task` in Copilot.

### J16 — Clean-checkout and restart proof

- Risk: Amber release engineering
- Owner: Copilot
- Verifier: Matthew observes only any genuinely manual installation step
- Depends on: J15
- Outcome: documented setup, build, restart and replay checks work from a clean
  checkout on the native Windows machine.
- Constraint: do not move `tethers-0.1/engine-ocaml` during this cycle because
  its opam switch is path-bound. The folder name does not determine the release
  tag.
- Acceptance: exact tool versions and commands are recorded; no secret or
  machine-specific absolute path is committed; restart preserves replay safety
  and Trail truth.

Give Copilot:

```text
Run J16 from docs/ROAD_TO_0_2.md on the native Windows checkout. Record exact
commands, versions and results. Fix only reproducibility defects inside the
approved packet. Do not rename tethers-0.1 or redesign installation.
```

### J17 — Independent 0.2 release sign-off

- Risk: Red milestone gate
- Owner: Lucy/Codex
- Depends on: J16
- Outcome: independent review against all seventeen release-acceptance claims.
- Required verdict: `SIGNED OFF FOR 0.2.0` or `NOT SIGNED OFF`, with each failed
  claim mapped to the smallest correction job.
- If signed off:
  - update `README.md`, `CURRENT_GOAL.md`, `TASK_QUEUE.md`,
    `PROJECT_DASHBOARD.md`, decisions and changelog;
  - set externally visible runtime/server version to `0.2.0` where the
    versioning contract requires it;
  - create the release commit and annotated `v0.2.0` tag only after the clean
    suite passes;
  - preserve 0.1 semantic fixtures and history.
- Excludes: beginning 0.3, Lantern Keeper, HQ or celebratory cleanup.

Give Lucy/Codex:

```text
Perform J17 release sign-off from docs/ROAD_TO_0_2.md. Review the complete diff
and worker-note chain from the last signed baseline, run or verify the complete
native Windows evidence, and decide every release-acceptance claim. Tag 0.2.0
only if every required claim is proved. Stop after the release verdict.
```

## 7. Matthew's normal interaction

Matthew should not have to paste architecture or technical histories into every
agent.

For ordinary continuation:

1. Open Copilot in the Tethers repository.
2. Run `/next-tethers-task`.
3. Read its short plain-English summary and route.
4. If it routes to Cline, run `/tethers-task.md`.
5. If it says a Red or milestone gate is due, give Lucy/Codex the exact block
   from that job above.

The repository supplies:

- this programme;
- current dashboard;
- current bounded task packet;
- relevant frozen decisions;
- exact files and tests;
- selected prior worker notes;
- live Git evidence.

Matthew supplies product judgement only when the dashboard names a real
decision. “Please carry this technical report to another AI” is a workflow
failure and must be fixed in the repository.

## 8. Planned review cadence

Lucy/Codex is required at:

- J01 control-loop acceptance;
- J02 current Red milestone verdict;
- J03 policy contract;
- J05 one-shot approval contract and final review;
- J06 outcome/uncertainty contract and J07 final review;
- J09 replay-persistence contract and final review;
- J11 causal-loop safety contract and final review;
- J12 runnable-set/config format;
- J14 vertical-slice sign-off;
- J17 release sign-off.

Lucy/Codex is not automatically required after every Green change. Copilot
verifies routine evidence and compiles the next packet. Any disagreement,
semantic change, permission/persistence/determinism concern, or two similar
failures escalates immediately.

## 9. Known issues and deliberate decisions

### The checker evidence is currently incomplete

The first Windows report says the packet checker ran but omits its output and
exit code. J00 is therefore still open. Invoking a command is not proof that it
passed.

### Future packets cannot all be fully instantiated today

Their exact base commits, dirty-state snapshots, touched files and focused
commands depend on work that does not yet exist. This programme freezes
outcomes, boundaries, dependencies, owners and handoff inputs. Copilot compiles
the executable packet just before each job.

That is not incomplete planning. It is the only way to avoid stale or fictional
task contracts.

### The 0.1 folder remains during 0.2

The project-local opam switch is bound to
`tethers-0.1/engine-ocaml`. Renaming the directory for cosmetic version symmetry
would create risk with no runtime value. Git tags and product metadata identify
the release. A later separately planned repository-layout migration may remove
the historical folder name.

### `ask` is a trust boundary, not a prompt string

The approval must bind the exact Action, arguments, capability version,
manifest digest and provider identity. A generic “yes” stored independently of
those facts is not permission.

### `uncertain` is a first-class honest result

An external effect and a separate local record cannot be made perfectly atomic.
When the call may have reached the provider and no trustworthy response exists,
the system records uncertainty and stops. Automatically retrying would exchange
honesty for duplicate effects.

### 0.2 does not wait for Lantern Keeper

Tethers must be useful and testable as its own product. Lantern Keeper becomes a
provider and host integration after it exposes a small stable capability
surface. Pulling its database and memory workflow into this release would blur
both architectures and delay the first complete Tethers runtime.

## 10. Change control

This programme may change when implementation reveals evidence the current
architecture did not know.

A change must record:

- what evidence invalidated the current plan;
- which jobs or dependencies change;
- whether the 0.2 acceptance statement changes;
- who approved the change;
- why the smaller correction was insufficient.

Do not quietly expand 0.2 because a future feature is attractive. Do not quietly
shrink it because a difficult trust boundary takes longer than expected.

The release remains:

> A small deterministic language connected to one real, local, permissioned and
> fully explained execution loop.
