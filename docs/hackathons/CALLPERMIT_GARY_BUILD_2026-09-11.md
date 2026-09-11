# GARY JOB PACKET — CALLPERMIT / CALL-E HACKATHON BUILD

**Date:** 11 September 2026
**Priority:** URGENT — CALL-E Devpost submission closes 14 September 2026 at 15:45 UTC.
**Target repository:** `matthewjameswatkins1978-cyber/tethers-lang`
**Base:** current `main`
**Worker route:** `mimo-v2.5` (ordinary MiMo 2.5, not Pro)
**Working product name:** **CallPermit**

## 0. Mission

Build the smallest strong, real, judge-friendly CALL-E hackathon entry that demonstrates **delegated phone authority without invented permission**.

The product thesis is:

> CALL-E handles the conversation. Tethers defines the permission slip.

A user delegates a bounded routine phone task, for example:

> Book one standard appointment on a weekday after 1pm, costing no more than £60. Do not agree to extras or any materially different commitment.

Tethers deterministically evaluates and freezes the authority envelope **before** the call. CALL-E may converse naturally inside that envelope. If the recipient presents a materially new choice outside the authorised envelope, CALL-E must not invent permission. It defers, returns structured evidence, and the user can later authorise a follow-up atomic call.

This is **not** live mid-call Tethers interception. Do not claim that. The product is deliberately built around atomic calls because current CALL-E control is pre-call plus post-call observation/result handling.

The useful distinction is not “AI makes a phone call.” CALL-E already does that. The contribution is:

**natural conversation inside a deterministic externally-owned authority boundary.**

Finish as much as possible without Matthew. Stop only at a genuine human-only boundary.

---

# 1. Historical evidence — reuse, do not redo blindly

On 30 August 2026 an earlier GARY Gate 0 run created CALL-E spike work on the Tethers branch:

`gary/master-calle-tethers-ga-68a3ef`

Head observed previously:

`9df32ab307b15b9e824a63172e44f64980ca4ce0`

The original worker branch also existed as:

`gary/calle-tethers-ga-w1-j1-c52b9842`

Relevant old files:

- `spikes/calle-gate0/**`
- `docs/gate0-calle.md`

That spike used:

- Node 20.20.2
- TypeScript 5.x
- `@call-e/calle` 0.7.0 at that time
- `CalleClient`
- structured `resultSchema`
- deterministic local tests
- safe checks for `CALLE_API_KEY` and `CALLE_TEST_PHONE`

Important correction: the old `docs/gate0-calle.md` labelled itself PASS even though no live call occurred because both credentials were absent. Treat it as **PARTIAL historical evidence**, not proof of a live CALL-E boundary.

Do not merge that old branch wholesale. It also contains generated material that should not be copied blindly. Inspect it, salvage useful source/tests/ideas, then build cleanly against current `main` and current official CALL-E interfaces.

The second planned Gate 0 workstream, proving Tethers ALLOW/ASK/DENY for this use case, did not land. That is now part of this job.

---

# 2. Current external source of truth

Before implementation, inspect the current official CALL-E material. Do not rely on remembered August API shapes if the current SDK differs.

Official repositories:

- `https://github.com/CALLE-AI/call-e-integrations`
- `https://github.com/CALLE-AI/awesome-phone-call-agents`

Current official CALL-E integration patterns observed on 11 September include:

- TypeScript package `@call-e/calle`
- Python package `calle-ai`
- `CalleClient`
- `client.calls.createAndWait(...)`
- direct API `POST /v1/calls`
- read result `GET /v1/calls/{call_id}`
- events `GET /v1/calls/{call_id}/events`
- structured results, evidence, transcripts and completion fields
- MCP flow `plan_call -> run_call -> get_call_run`
- persisted call/run identity and polling rather than blindly placing a second call after local timeout

The official community contribution repository currently accepts reusable:

- Agent Skills under `skills/`
- Apps under `apps/`
- Workflow plugins under `plugins/`

A hackathon submission requires a pull request to `CALLE-AI/awesome-phone-call-agents`, plus a Devpost entry and roughly three-minute public demo.

Inspect the repository’s current README/contribution conventions before freezing the deliverable layout.

---

# 3. Tethers source of truth

Use the **current Tethers 0.7 implementation on `main`**, not historical Gate 0 assumptions.

Tethers 0.7 is the authority/execution substrate. Reuse the narrowest real supported machine interface that can faithfully provide the required deterministic decision.

Required semantic output for this product is equivalent to:

- `ALLOW` — the proposed commitment is inside the frozen delegated authority.
- `ASK` — it is outside current authority but is plausibly something the human may approve.
- `DENY` — it is explicitly prohibited, malformed, unsafe, or otherwise not eligible for escalation as an ordinary choice.

If current native Tethers terminology differs, use a thin documented mapping. Do **not** redesign Tethers semantics for the hackathon.

Malformed or untrusted input must never silently become ALLOW.

Do not build a new generic policy engine inside CallPermit. Tethers is the authority engine.

---

# 4. Product behaviour

## 4.1 Primary scenario

CallPermit supports **one bounded routine administrative appointment/service call**.

The demo policy should be synthetic and safe, for example:

- one standard appointment/service only;
- weekdays only;
- start time after 1:00 PM;
- maximum total price £60;
- no add-on service;
- no subscription/membership;
- no deposit or purchase unless explicitly authorised;
- no materially different commitment.

Avoid medical diagnosis, legal, financial, emergency, debt-collection or other high-stakes decision making in the demo. This is routine admin delegation.

## 4.2 Before the call

Input should be a small human-readable delegation plus structured facts.

Compile/normalise it into an inspectable **Authority Envelope**, containing at minimum:

- purpose;
- allowed commitment type;
- time/date bounds;
- price ceiling when applicable;
- allowed extras (normally none in demo);
- explicit prohibitions;
- what must cause DEFER/ASK;
- stable authority/envelope ID or digest;
- relevant Tethers version/evidence identity.

Tethers evaluates the intended call/action before any real dispatch.

If preflight is DENY, no CALL-E call may be created.

If preflight is ASK, no CALL-E call may be created until authority changes.

Only an allowed call plan proceeds.

## 4.3 During the call

There is **no claim of live Tethers callback/interception**.

Instead, the approved Authority Envelope is compiled into concise explicit call-task constraints for CALL-E.

CALL-E should be instructed to:

- converse naturally;
- pursue the authorised goal;
- commit only when all relevant offered terms are within the envelope;
- never infer broader authority from conversational convenience;
- if a new commitment or choice is outside/unknown, politely say it needs to check with the requester and do not commit;
- return the offered terms and reason for deferral in structured output.

The language should not force a robotic script. Authority is strict; conversation may remain flexible.

## 4.4 Structured result

Use the current supported CALL-E structured-result mechanism.

Target a schema approximately like:

```json
{
  "outcome": "COMMITTED | DEFERRED | NO_MATCH | FAILED",
  "offered_date": "string|null",
  "offered_time": "string|null",
  "offered_price_minor": "integer|null",
  "currency": "GBP|null",
  "service": "string|null",
  "extra_requested": "string|null",
  "commitment_made": "boolean",
  "defer_reason": "string|null",
  "recipient_words_supporting_result": ["string"]
}
```

Adapt to the current SDK’s actual JSON-schema support. Keep it small enough to be robust.

Do not claim a commitment solely because CALL-E reports `task_completed=true`. Reconcile the structured result and evidence/transcript.

## 4.5 After the call

Run deterministic reconciliation:

CALL-E result + frozen Authority Envelope -> Tethers / deterministic validation -> one of:

- `COMMITTED_WITHIN_AUTHORITY`
- `DEFERRED_NEEDS_MATTHEW`
- `NO_COMMITMENT`
- `INVALID_OR_UNCERTAIN_RESULT`

If CALL-E says a commitment was made but the returned terms exceed the frozen envelope, this is a **policy violation finding**, not success. Surface it loudly in tests/demo output.

When outcome is deferred, create a compact **Decision Card** containing:

- what the recipient offered/asked;
- why it exceeded current authority;
- evidence/transcript excerpt or evidence fields;
- the exact new authority that would be needed;
- no automatic follow-up call.

A later follow-up is a new atomic call with a new frozen authority envelope.

---

# 5. Required deterministic fixtures

Build mechanical tests around at least these cases:

1. **ALLOW / COMMIT**
   - Tuesday 3:30 PM
   - £45
   - standard service
   - no extras
   - expected authority: ALLOW

2. **ASK / DEFER — price**
   - Tuesday 3:30 PM
   - £95
   - standard service
   - expected authority: ASK
   - must not be represented as authorised commitment

3. **ASK / DEFER — extra**
   - £45 standard service plus an unrequested add-on
   - expected authority: ASK

4. **DENY — explicitly forbidden commitment**
   - prohibited subscription/membership or another explicit forbidden action from the demo policy
   - expected authority: DENY

5. **DENY/ERROR — malformed input**
   - e.g. `{ "banana": "ferret" }`
   - must fail closed; never ALLOW

6. **Uncertain result**
   - CALL-E terminal/result fixture contradicts or lacks enough evidence for the reported commitment
   - expected: INVALID_OR_UNCERTAIN_RESULT

7. **Timeout/restart safety**
   - after a call/run ID exists, a local timeout/restart must resume observation of the same ID and must not silently dispatch another call.

8. **Authority immutability**
   - changing the authority envelope after dispatch must not retroactively change the authority identity attached to the existing call record.

Repeat deterministic Tethers fixtures and prove stable outputs.

---

# 6. Build location and separation

Create a self-contained hackathon/contribution area under the current Tethers repo, preferably:

`hackathons/callpermit/`

Do not make production Tethers depend on CALL-E.

Suggested clean structure, adapt only when the official contribution convention clearly argues for something better:

```text
hackathons/callpermit/
  README.md
  package.json
  package-lock.json
  tsconfig.json
  src/
    authority.ts
    calle.ts
    reconcile.ts
    cli.ts
    types.ts
  tests/
    authority.test.ts
    call-fixtures.test.ts
    reconcile.test.ts
    retry-safety.test.ts
  fixtures/
  demo/
    DEMO_SCRIPT.md
    demo-policy.json
  contribution/
    SKILL.md or app subtree matching awesome-phone-call-agents conventions
    README.md
  docs/
    ARCHITECTURE.md
    SECURITY_AND_AUTHORITY.md
    TESTING.md
    DEVPOST_DRAFT.md
    PR_DRAFT.md
```

Do not commit `node_modules`, built `dist`, secrets, phone numbers, transcripts containing private data, or generated clutter.

If a small reusable Agent Skill plus runnable TypeScript demo app is the strongest contribution, it is acceptable to produce both, but do not create two products. One concept, one architecture, one demo.

---

# 7. CALL-E live-call boundary

Environment variables may exist locally:

- `CALLE_API_KEY`
- `CALLE_TEST_PHONE`

Never print their values.

Do all non-live work first.

A live call is a real-world side effect. **Do not place one automatically merely because credentials exist.**

When the build is otherwise ready, if a controlled live call is needed, stop as `NEEDS_MATTHEW` and provide:

- exact proposed task text;
- which authorised test number will be used, masked except last few digits;
- expected maximum number of calls;
- confirmation that it creates no purchase, booking, promise or external commitment for the proof call;
- exact command/action GARY intends to resume with.

After Matthew explicitly approves, make normally **one** controlled proof call. A second call is permitted only when it demonstrates the DEFER path or fixes a concrete technical failure and Matthew’s approval covers it.

For a safe proof call, prefer an explicitly authorised controlled recipient/test scenario. Do not call an arbitrary business or third party without explicit authority.

Persist call/run identity immediately. Timeout means resume observation, not dispatch again.

---

# 8. Demo requirements

Prepare a demo that can be recorded in roughly three minutes.

The demo should make the differentiator visible in under 30 seconds.

Recommended narrative:

1. **Problem:** phone agents can converse, but conversational competence is not permission.
2. Show the user’s bounded delegation.
3. Show Tethers producing/fixing the frozen Authority Envelope.
4. Show one allowed fixture or controlled call.
5. Show an out-of-envelope offer such as £95 and the system returning DEFER / NEEDS HUMAN rather than pretending the agent had authority.
6. Show evidence/receipt and the fact that a follow-up requires a new authority envelope.
7. End on the line: **“The AI decides what to say. Tethers decides what it is allowed to agree to.”**

Prepare `DEMO_SCRIPT.md` with exact screen sequence and narration, but do not fabricate a live result.

If no real call has yet been authorised/completed, clearly mark where real-call footage must be inserted later.

---

# 9. Architecture diagram

Produce a judge-readable architecture diagram in Mermaid source and, if toolchain permits without fuss, an SVG/PNG export suitable for Devpost/video.

Canonical architecture:

```text
User delegation
      |
      v
Tethers 0.7 authority evaluation
      |
      v
Frozen Authority Envelope + digest
      |
      v
CallPermit task compiler
      |
      v
CALL-E atomic call
      |
      v
Structured result + transcript/evidence
      |
      v
Deterministic reconciliation against SAME envelope
      |
      +---- within authority ----> committed result/receipt
      |
      +---- outside/unknown -----> Decision Card -> Matthew
                                      |
                                      v
                           new authority if approved
                                      |
                                      v
                              new atomic call
```

The diagram must not imply live Tethers control inside the CALL-E conversation.

---

# 10. Hackathon contribution packaging

Inspect current `CALLE-AI/awesome-phone-call-agents` contribution conventions.

Prepare a PR-ready contribution under `hackathons/callpermit/contribution/` that can be copied with minimal/no transformation into the appropriate official folder, likely `skills/callpermit/` or `apps/typescript/callpermit/`.

Required qualities:

- focused and reusable;
- dry-run/fixture mode works without credentials;
- real-call side effects explicit;
- setup clear;
- credential handling clear;
- authority model clear;
- no unsupported claims;
- deterministic tests included;
- current CALL-E API genuinely invoked at runtime in the real path;
- Tethers genuinely invoked at runtime or through the narrowest faithful supported machine interface, not merely mentioned in prose.

Prepare `docs/PR_DRAFT.md` containing:

- proposed title;
- concise summary;
- why it is useful to the CALL-E community;
- safety/authority behaviour;
- tests run;
- live-call evidence status;
- exact target contribution path.

Do **not** open the public PR without Matthew’s explicit final approval. Fork/PR/public external mutation is a human boundary unless it has already been separately authorised.

---

# 11. Devpost package

Create `docs/DEVPOST_DRAFT.md` ready for Lucy/Matthew to paste or adapt.

It must cover:

- project title;
- one-line real-world task;
- who it is for;
- problem;
- what it does;
- why ordinary phone agents are insufficient;
- how CALL-E is used at runtime;
- how Tethers authority works without claiming mid-call interception;
- technical architecture;
- safety model;
- what was built during the hackathon;
- testing instructions;
- known limitations;
- public repo/PR placeholders;
- demo-video placeholder;

The hackathon judges score:

- real-world impact;
- quality/originality of idea;
- technical implementation with CALL-E genuinely used;
- product experience and demo.

Optimise for those, not feature count.

---

# 12. README standard

The CallPermit README should sell the project but remain exact.

Lead with something like:

> **CallPermit lets an AI make the phone call without letting it invent your permission.**

Then demonstrate the £60 vs £95 example immediately.

Keep installation/run/test instructions copy-pasteable.

Clearly distinguish:

- dry-run / fixtures;
- real CALL-E call;
- Tethers deterministic authority;
- CALL-E conversational behaviour;
- post-call reconciliation;
- human follow-up boundary.

Do not overclaim “secure,” “guaranteed,” “formally verified,” or “prevents all unauthorised actions.” State exactly what the implementation proves.

---

# 13. Version and dependency discipline

This is a deadline build, not an archaeological dig.

- Inspect current supported Node/TypeScript and current `@call-e/calle` package first.
- Prefer current non-beta stable versions unless the official CALL-E examples require otherwise.
- Do not downgrade the project merely to preserve August spike versions.
- Do not casually upgrade unrelated Tethers dependencies.
- Lock the CallPermit dependency tree.
- Run an ordinary dependency/security audit and record findings; fix high-confidence relevant issues, do not derail into unrelated dependency gardening.

---

# 14. Windows/worker discipline

Matthew’s main machine is Windows.

Use the project’s existing agent/workshop guidance where present.

In particular:

- establish repo, branch and dirty-state truth first;
- preserve user changes;
- use an isolated GARY worktree;
- prefer PowerShell 7 as Windows control shell;
- avoid heredoc/PowerShell quote soup and giant generated shell strings;
- use purpose-built tools for JSON/text manipulation;
- no `Invoke-Expression`;
- no destructive Git cleanup/reset to manufacture a clean state;
- tests/evidence outrank worker prose.

---

# 15. Scope exclusions

Do NOT:

- build a general personal assistant;
- build inbound calling;
- build live mid-call policy callback fiction;
- modify CALL-E itself;
- redesign Tethers semantics;
- build a database unless the demo genuinely needs one (it probably does not);
- add accounts/multi-user SaaS;
- build billing;
- build a mobile app;
- build recurrence/scheduling infrastructure;
- auto-call third parties;
- auto-submit to Devpost;
- auto-open the external public PR;
- spend money;
- add unrelated Tethers features;
- turn this into a giant framework.

The job is a small excellent hackathon product.

---

# 16. Definition of done

GARY may call the software candidate technically DONE only when all non-human gates below are satisfied:

1. `hackathons/callpermit/` exists and is self-contained.
2. Current CALL-E SDK/API is genuinely used by the live execution path.
3. Current Tethers 0.7 is genuinely used as the deterministic authority boundary through a faithful supported machine interface.
4. ALLOW/ASK/DENY fixture set passes.
5. malformed input fails closed.
6. timeout/restart cannot silently duplicate a real call.
7. post-call reconciliation detects out-of-envelope claimed commitments.
8. no secrets/phone numbers are committed.
9. dry-run/fixture demo works with no credentials.
10. README is complete and copy-paste runnable.
11. architecture documentation and diagram source exist.
12. contribution subtree is PR-ready for `CALLE-AI/awesome-phone-call-agents`.
13. PR draft exists.
14. Devpost draft exists.
15. ~3-minute demo script exists.
16. package typecheck/build/tests pass.
17. relevant Tethers tests used by the integration pass.
18. `git diff --check` passes.
19. exact branch/commit and test evidence are reported.

If live proof call has not been explicitly authorised, that is a valid final **NEEDS_MATTHEW** boundary after all non-live gates pass. Do not mark fabricated live-call evidence.

Likewise, external PR creation and Devpost submission remain human-authorised boundaries.

---

# 17. Independent acceptance surface

Create an `npm run acceptance` command inside `hackathons/callpermit` that runs the complete deterministic CallPermit verification suite without requiring live credentials or making network calls.

It should include at least:

- format/lint as configured;
- typecheck;
- unit tests;
- authority fixtures;
- reconciliation fixtures;
- duplicate-dispatch safety test;
- dry-run demo/smoke test.

The real-call proof must be a separate explicit command that cannot run accidentally as part of acceptance/CI.

GARY final acceptance should include:

```text
git diff --check
npm --prefix hackathons/callpermit run acceptance
```

plus any narrow Tethers regression command needed to prove the machine authority integration.

---

# 18. Authority granted to GARY for this job

GARY/worker MAY:

- inspect current local/remote repository state;
- inspect the old CALL-E spike branch;
- read current official CALL-E public repositories/docs;
- install ordinary local development dependencies required for this build;
- create/edit/delete files inside the isolated job worktree within the allowed scope;
- run local tests/builds/linters/audits;
- use the explicitly selected `mimo-v2.5` worker route;
- create a local feature branch and commit accepted candidate work;
- prepare contribution/PR/Devpost/demo materials;
- research errors and change implementation tactics without asking Matthew about ordinary engineering choices.

GARY/worker MUST STOP as NEEDS_MATTHEW before:

- any live real phone call;
- any public fork or external pull request if not already separately authorised;
- any Devpost submission;
- any purchase/spend;
- any use of unavailable credentials;
- any irreversible external action;
- any product decision that materially changes the core thesis.

Do not bother Matthew for naming, formatting, internal implementation or other reversible engineering details.

---

# 19. Delivery report

Return a concise final report containing:

- status: DONE / NEEDS_MATTHEW / FAILED;
- exact branch and commit;
- what was built;
- old spike material reused or rejected;
- current CALL-E SDK/version/methods actually used;
- Tethers interface actually used;
- deterministic fixture results;
- test/build/audit results;
- whether a live call occurred (normally no until approved);
- exact human boundary if one remains;
- contribution target path;
- PR draft path;
- Devpost draft path;
- demo script path;
- next single action Matthew should take.

## Final product rule

**Do not confuse conversational intelligence with authority.**

A model may understand the offer. That does not mean it has permission to accept it.
