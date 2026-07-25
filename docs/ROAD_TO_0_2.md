# Tethers Road To 0.2

Status: current release programme  
Owner and final product authority: Matthew  
Programme architect and controller: Lucy  
Red implementation and machine escalation: Codex  
Default implementation owner: Cline  
Updated: 25 July 2026

## 1. Release Decision

Tethers 0.2 will be the first usable standalone local runtime slice.

It proves this complete route:

```text
configured local provider
-> verified manifest admission
-> live capability projection
-> external Anchor plus immutable Facts
-> deterministic Tethers Plan
-> allow / ask / deny / unavailable policy
-> intent-first serial dispatch with no automatic retry
-> succeeded / failed / uncertain outcome
-> complete causal Trail
-> standard result Anchor
-> bounded serial follow-up evaluation
```

This is deliberately narrower than the complete Tethers vision. It is large
enough to be real and small enough to finish and trust.

Tethers 0.2 is not:

- Lantern Keeper's memory store or retrieval system;
- HQ or a graphical editor;
- a package marketplace;
- automatic provider discovery;
- a remote or multi-user service;
- Streamable HTTP, OAuth, or network listeners;
- parallel execution;
- automatic retries or compensation;
- a general programming language;
- an AI agent framework.

## 2. Current Baseline

The accepted implementation baseline through J04a includes:

- Tethers 0.1 language and protocol sign-off;
- OCaml-owned MCP planner and validation tools;
- verified manifest parsing, canonicalisation, digesting, and trusted storage;
- configured local stdio provider discovery and fail-closed admission;
- deterministic live capability projection;
- exact manifest, capability-version, and provider pins carried through planning;
- dispatch requiring a policy-created proof token and durable intent preparation;
- output validation and known-outcome Result Anchors;
- effective policy outcomes `allow`, `ask`, `deny`, and `unavailable`;
- fail-closed stale-digest and unestablished-scope handling.

Latest accepted implementation checkpoint:

`d5ed278d4a2cae5e9ab8a3e1d8700fdcba7ae851`

J04a is accepted. No J05 implementation is authorised until Lucy freezes the
one-shot approval and resume design.

## 3. Release Acceptance

Tethers 0.2 is accepted only when the native Windows checkout proves all of the
following:

1. A configured deterministic stdio MCP provider is admitted only through its
   verified manifest and live host binding.
2. One external Anchor and immutable Fact snapshot produce the expected
   deterministic Plan and evaluation Trail.
3. A bridge-backed Action carries exact capability version, manifest digest, and
   provider identity pins from planning to dispatch.
4. `allow`, `ask`, `deny`, and `unavailable` have distinct, tested, fail-closed
   behaviour.
5. An approved Action is attempted serially and at most once.
6. Intent is durably recorded before an effectful call.
7. Success output is schema-validated before success is recorded.
8. Executor failure and output-validation failure produce `failed`.
9. A deadline after possible dispatch produces `uncertain`, never guessed
   failure and never automatic retry.
10. Attempted calls produce exactly one standard Result Anchor:
    `capability.succeeded`, `capability.failed`, or `capability.uncertain`.
11. Unattempted Actions produce no false Result Anchor.
12. Generated Result Anchors are queued serially, duplicate event IDs are
    rejected, and causal generation stops visibly at depth eight.
13. Replaying a completed execution identity does not repeat an external effect.
14. The Trail distinguishes evaluation, permission, intent, attempt, outcome,
    result event, and audit failure without claiming more than is known.
15. A local check/run/inspect route completes the slice without manually
    assembling internal test objects.
16. Focused tests, the complete Rust suite, engine fixtures, MCP transcripts,
    host integration scripts, demo, OCaml build, and Git checks pass.
17. The release is documented, reproducible from a clean checkout, and signed
    off independently.

## 4. Gorilla Coding Operating Rule 🦄

The release programme describes outcomes and dependencies. It does not pre-write
stale implementation packets.

Only one implementation job is active at a time:

```text
Lucy inspects live GitHub state
-> Lucy compiles one bounded packet
-> Matthew routes it to Cline or Codex
-> one owner implements and verifies
-> owner writes the worker note and concise report
-> Matthew pastes the report to Lucy
-> Lucy inspects evidence and accepts, corrects, or escalates
```

Current routes:

| Work | Route |
| --- | --- |
| Architecture, semantics, task compilation, GitHub-visible review | Lucy |
| Ordinary Green and Amber implementation | Cline |
| Red implementation or computer-enabled Red sign-off | Codex |
| Difficult local failure, Git/environment/recovery, unpushed-state diagnosis | Codex |
| Product direction and consequential approval | Matthew |

Copilot is not part of the active route.

A pasted Cline report is an accepted handoff. It does not replace durable packet,
worker-note, code, test, Trail, or Git evidence.

No job is complete without its named worker note. Cline does not compile the next
job. Lucy controls continuation.

## 5. Standard Evidence

Each packet names focused checks and the relevant subset of this native Windows
gate:

```powershell
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

A worker never claims an unrun command passed. The packet may omit irrelevant
expensive checks only when it states why and retains every check needed to prove
the task.

## 6. Dependency Stages

| Stage | Jobs | Gate |
| --- | --- | --- |
| A. Accepted baseline | J00-J04a | Complete |
| B. Exact approval | J05 | One-shot approval and resume proven |
| C. Honest execution | J06-J09 | Deadline, uncertainty, Result Anchor, and replay semantics proven |
| D. Event continuation | J10-J11 | Result events queue safely to depth eight |
| E. Operable slice | J12-J14 | Check/run/trail route and real scenario proven |
| F. Harden and release | J15-J17 | Clean-checkout proof and 0.2 sign-off |

No stage starts merely because code exists. Its gate requires an evidence-backed
verdict.

## 7. Remaining Job Register

### J05: Exact One-Shot Ask Resolution

- Risk: Red.
- Architect: Lucy.
- Implementation and local verification: Codex unless Lucy deliberately splits a
  later bounded Green subtask to Cline.
- Depends on: accepted J04a.
- Outcome: unresolved Ask stops safely; one explicit decision bound to the exact
  Action proof may resume or deny that Action exactly once.
- Must prove:
  - changed arguments, manifest, provider, Action ID, or proof digest invalidate
    approval;
  - approval cannot be reused or consumed twice;
  - expiry and cancellation fail closed;
  - denial never calls the executor;
  - pending Ask emits no success, failure, or uncertain Result Anchor;
  - Trail distinguishes requested, approved, denied, stale, expired, cancelled,
    and consumed.

### J06: Freeze Deadline And Outcome Classification

- Risk: Red design.
- Architect: Lucy.
- Codex consulted where live implementation or machine behaviour is required.
- Depends on: J05.
- Outcome: one execution state machine defining unattempted, succeeded, failed,
  uncertain, and known-outcome-with-audit-failure.
- Must settle deadline ownership, timeout before or after possible dispatch,
  provider crash, EOF, malformed response, cancellation, invalid success output,
  final Trail failure, and no-retry behaviour.

### J07: Implement Deadlines And `uncertain`

- Risk: Red implementation.
- Owner: Codex.
- Depends on: J06.
- Outcome: provider calls obey the frozen deadline contract and classify outcomes
  honestly.
- Evidence must prove pre-dispatch failure, ordinary failure, timeout after
  possible effect, malformed success, valid success, exact call counts, and no
  retry.

### J08: Emit `capability.uncertain`

- Risk: Green when it follows the accepted Result Anchor pattern.
- Owner: Cline.
- Reviewer: Lucy.
- Depends on: J07 accepted.
- Outcome: exactly one uncertain Result Anchor after an attempted call with
  uncertain outcome; no Result Anchor for unattempted Actions.

### J09: Durable Replay Protection

- Risk: Red persistence boundary.
- Architect: Lucy.
- Owner: Codex.
- Depends on: J07.
- Outcome: a completed execution identity cannot repeat its external effect after
  restart.
- Corrupted or missing persistence fails closed. Uncertain identities are
  surfaced for resolution and are never automatically retried.

### J10: Queue Generated Result Events Serially

- Risk: Amber.
- Owner: Cline.
- Reviewer: Lucy.
- Depends on: J08 and J09.
- Outcome: standard Result Anchors enter a host-owned FIFO queue and are evaluated
  one at a time in stable order without recursive stack execution or parallel
  events.

### J11: Event Deduplication And Causal Depth Eight

- Risk: Red safety boundary.
- Architect: Lucy.
- Owner: Codex.
- Depends on: J10.
- Outcome: duplicate event IDs are rejected and generation beyond depth eight
  stops visibly with no ninth evaluation.

### J12: Minimal Runnable Tether Set And Configuration

- Risk: Red format decision.
- Architect: Lucy.
- Codex consulted for implementation constraints.
- Depends on: J11.
- Outcome: the smallest local configuration selects ordered Tethers, exact
  capability requirements, provider binding, and local policy without becoming
  package management or secret storage.

### J13: Implement Local `check`, `run`, And `trail`

- Risk: Amber.
- Owner: Cline.
- Reviewer: Lucy.
- Depends on: J12.
- Outcome:
  - `check` validates source, requirements, provider admission, and availability
    without dispatch;
  - `run` submits one explicit Anchor and Facts input through the real slice;
  - `trail` locates or prints one causal record.
- Must remain native Windows and PowerShell-friendly with no daemon, GUI, or
  remote transport.

### J14: Prove One Complete Local Scenario

- Risk: Amber integration with a Red milestone gate.
- Implementation owner: Cline.
- Review: Lucy from pushed evidence; Codex handles machine-required verification
  or unresolved local ambiguity.
- Depends on: J13.
- Outcome: one provider capability is admitted, planned, allowed, attempted once,
  output-validated, recorded, and converted into a Result Anchor through the
  public route.
- Reproducible negative cases cover malformed manifest, unavailable provider,
  Ask states, Deny, stale pin, intent failure, executor failure, invalid output,
  uncertain timeout, duplicate replay, and loop depth.

### J15: Consolidate The 0.2 Failure Matrix

- Risk: Amber verification.
- Owner: Cline.
- Reviewer: Lucy.
- Depends on: J14 accepted.
- Outcome: one discoverable Windows verification entry point reports every
  release case separately and honestly.

### J16: Clean-Checkout And Restart Proof

- Risk: Red release engineering and machine work.
- Owner: Codex.
- Depends on: J15.
- Outcome: setup, build, restart, replay, and verification work from a clean
  native Windows checkout with exact versions and commands recorded.
- Do not move `tethers-0.1/engine-ocaml`; its local opam switch is path-bound.

### J17: Independent 0.2 Release Sign-Off

- Risk: Red milestone gate.
- Architect and product-side verdict: Lucy with Matthew.
- Computer-enabled release verification and Git/tag work: Codex.
- Depends on: J16.
- Verdict: `SIGNED OFF FOR 0.2.0` or `NOT SIGNED OFF` with every failed acceptance
  claim mapped to the smallest correction.
- Tag `v0.2.0` only after every required claim and the clean native suite pass.

## 8. Matthew's Normal Interaction

1. Give Cline the current `READY` task with `/tethers-task.md`.
2. When Cline stops, paste its concise report to Lucy.
3. Lucy inspects GitHub and replies with acceptance, one correction, or a Codex
   escalation.
4. Route the next task exactly as Lucy states.

Matthew is not expected to understand or rewrite the technical handover. His
copy-and-paste bridge is acceptable because it is short, deliberate, and cheaper
than keeping a computer-enabled model running as permanent controller.

## 9. Deliberate Decisions

### Future Packets Are Compiled Just In Time

Exact base commits, dirty-state snapshots, touched files, and focused commands
depend on live work. Lucy compiles each executable packet immediately before the
job. This prevents stale or fictional contracts.

### The `tethers-0.1` Folder Remains During 0.2

The local opam switch is path-bound. Git tags and product metadata identify the
release; cosmetic folder symmetry is not worth the risk.

### `ask` Is A Trust Boundary

Approval binds the exact Action, arguments, capability version, manifest digest,
provider identity, and proof. A generic stored yes is not permission.

### `uncertain` Is A First-Class Honest Result

When an external effect may have occurred and no trustworthy response exists,
the system records uncertainty and stops. Automatic retry would trade honesty
for possible duplicate effects.

### 0.2 Does Not Wait For Lantern Keeper

Tethers must be useful and testable as its own product. Lantern Keeper integrates
later through a small stable capability surface.

## 10. Change Control

Change this programme only when implementation evidence invalidates it. Record:

- the evidence;
- affected jobs or dependencies;
- whether the 0.2 acceptance statement changes;
- who approved the change;
- why a smaller correction was insufficient.

Do not quietly expand 0.2 because a future feature is attractive. Do not quietly
shrink it because a trust boundary is difficult.

The release remains:

> A small deterministic language connected to one real, local, permissioned,
> fully explained execution loop.
