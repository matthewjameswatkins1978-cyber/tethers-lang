# Tethers Project Control Loop

Status: current operating procedure

## Purpose

Keep Matthew in control of product direction without making him reconstruct the technical project from memory. Preserve Tethers' architecture, trust boundaries, determinism, implementation quality and evidence while exploiting the speed and flexibility of a one-human, multi-AI shop.

The current operating model is **Gorilla Bunny Coding Shop 🦍🐇**.

It is intentionally not a miniature conventional software company. We borrow normal engineering practices when they buy real correctness, recoverability or speed, and discard them when they mainly solve coordination problems created by large groups of humans.

Full doctrine: `docs/GORILLA_BUNNY_MANIFESTO.md`.

The Constitution, specifications, accepted decisions, code, tests, fixtures, Trails and Git remain authoritative for product behaviour and engineering evidence.

## Control Roles

- **Matthew, product owner:** direction, taste, priorities, consequential trade-offs, publication and final human judgement. Matthew is also happy to act as the short copy/paste relay between Lucy and agents when that keeps him visibly in the loop without creating unnecessary admin.
- **Lucy, architecture department and controller:** holds the larger technical shape, resolves ambiguity, freezes decisions when necessary, decomposes work, chooses the level of proof, routes jobs to suitable agents, reviews pushed evidence, accepts/rejects results and continuously improves the shop itself.
- **Gem, peer technical sparring partner:** used when difficult architecture, semantics, concurrency, trust or other high-judgement work benefits from a second senior technical model challenging Lucy's assumptions. Gem is not a mandatory review stage.
- **Agents, replaceable specialist labour:** implement bounded changes, investigate, test, review, benchmark or attack designs adversarially. Agent/model choice follows task fit and economics rather than permanent organisational roles.
- **Repository:** holds durable packets, decisions, current state, worker notes, code, tests and evidence references.

Historical tools and harnesses may retain their names in branches, files and worker notes. No particular coding agent or model has a permanent institutional role.

## Operating Metric

Optimise the whole system:

**compute + retries + Matthew effort + elapsed time -> accepted correct work**

The cheapest token price is not automatically the cheapest job. A stronger model that solves a difficult task once may be cheaper than repeated retries. A powerful model doing mechanical work may be wasteful.

Matthew's attention is especially scarce. Do not save trivial compute by creating avoidable human faff.

## Process Must Earn Its Keep

Every control step must remove a concrete uncertainty, protect an important boundary, preserve useful memory or materially improve speed/recoverability.

Do not add review, onboarding, documentation or ceremony merely because a conventional software company would normally have it.

Before adding another step, ask:

**What uncertainty will this remove?**

If the answer is weak, stop.

## Risk Is Separate From Routing

A colour describes risk, not a permanent model assignment.

| Class | Meaning | Normal treatment |
| --- | --- | --- |
| Green | Existing pattern, narrow, reversible, objectively testable | cheapest capable route; Lucy may review lightly or directly |
| Amber | Multi-file/module interaction, settled behaviour, moderate judgement | suitable implementation agent plus one bounded Lucy review |
| Red | Semantics, permissions, trust, persistence, compatibility, concurrency, determinism, security or architecture | Lucy freezes the important decisions; use stronger reasoning, Gem or independent verification only where they materially reduce risk |

Do not lower a risk class to fit available compute. Do not escalate ordinary work merely because an advanced language or model is involved.

Foundational Red work often deserves adversarial or independent proof. That is a risk response, not an automatic corporate ritual.

## One Active-Task State Machine

`docs/CURRENT_CLINE_TASK.md` is the historical filename for the single current implementation contract. It does not imply Cline ownership.

1. `PROPOSED` — candidate task; implementation not yet authorised.
2. `READY` — approved with one owner, route, base and worker-note path.
3. `IN_PROGRESS` — owner is working.
4. `BLOCKED` — stopped cleanly with evidence and the smallest unresolved issue.
5. `COMPLETE` — owner claims work and required evidence exist.
6. `ACCEPTED` — Lucy or the chosen verifier has accepted the evidence.
7. `REJECTED` — evidence proves the implementation does not meet the contract.

Lucy normally compiles and routes tasks. Matthew need not approve every routine packet, but explicit Matthew approval is required for consequential human decisions such as merging `main` when that has been reserved to him, publication, meaningful expenditure or choices that materially affect product direction.

A task is not complete merely because code was written. Where the packet requires them, completion includes the implementation checkpoint, verification, worker note, normal push and clean remote state.

## Compiled Context Packet

Agent packets should contain only context capable of changing the job:

- exact outcome, route, risk and owner type;
- exact base SHA and expected branch;
- the few relevant files/interfaces/docs;
- frozen decisions and invariants;
- permitted and forbidden scope;
- acceptance criteria and proof;
- stop conditions;
- verification and normal-push requirements.

Durable architecture belongs in repository docs, not repeated giant archaeology packets.

Matthew should receive each agent job as **one complete copy/paste block**, never fragments that need manual reconstruction.

## Autonomy And Interruption

Autonomy is the default inside a bounded task.

Agents should continue when the next step is obvious, reversible and authorised. Do not repeatedly ask Matthew for routine permission.

Escalate when the decision genuinely needs human judgement, changes architecture or meaning, risks irreversible damage, commits meaningful money, publishes externally or presents alternatives where Matthew's intent matters.

The goal is to bring Matthew decisions, not administration.

## Work And Failure Rules

- One bounded implementation owner at a time unless a task explicitly benefits from another arrangement.
- Agents must not silently redesign frozen surrounding architecture.
- After two materially similar failed attempts, stop and preserve evidence rather than entering a loop.
- A failed job should be salvaged when it has produced useful findings, partial evidence or a reproducible defect.
- If an external effect may have occurred without a trustworthy result, report `uncertain`; never invent success.
- When acceptance evidence is sufficient, stop. Do not spend requests on ceremonial repetition.
- A report is a claim. Code, tests, fixtures, compiler output, Trails and Git are evidence.

## Gem Rule

Gem is a thinking surface, not a committee.

Bring Gem in when disagreement or a second senior technical model is likely to improve a consequential design. Typical cases include subtle semantics, concurrency, replay/persistence, trust boundaries or architecture with several plausible solutions.

Do not call Gem merely to obtain agreement on routine work. Lucy remains responsible for integrating the debate and making the architecture decision.

## Security And Reversibility

Security should fit the actual environment.

Protect real risks: credentials, destructive operations, irreplaceable data, repositories, external exposure and irreversible effects. Use Git, backups, bounded changes and inspectable evidence aggressively.

Do not cripple local development with enterprise security ceremony designed for an organisation we do not have.

Reversibility buys speed. Cheap contained experiments may move quickly. Irreversible actions deserve more care.

## Return Journey

Every substantial agent task names a worker note under `docs/worker-notes/` when durable evidence is useful.

Agents return a concise report. Matthew may paste that report into ordinary chat with Lucy. This manual relay is an accepted human-visible control surface, not a failure of automation.

The pasted report does not replace evidence. Lucy verifies pushed GitHub state for important work and decides accept, correct, salvage, reject or stop.

## Verification And Review

Proof follows risk.

Green work may be accepted from narrow objective evidence.

Amber work normally receives one bounded Lucy review.

Red work receives whatever additional design freeze, Gem debate, adversarial test or independent verification is justified by the actual failure cost. Avoid both extremes: trusting a high-risk result blindly and creating a ceremonial review chain that proves nothing new.

A verifier checks what matters:

1. branch, base and diff;
2. requirements against evidence;
3. relevant architectural/semantic boundaries;
4. unexpected changes and unsupported assumptions;
5. worker-note/report accuracy;
6. whether another step buys meaningful confidence.

If correction is required, prefer the smallest correction packet rather than an open-ended repair loop.

## Living Documentation

Historical acceptance reports and worker notes are records of their time and are not rewritten merely to sound current.

Living documents must track accepted truth:

- `docs/PROJECT_DASHBOARD.md`
- `docs/CURRENT_GOAL.md`
- `docs/CURRENT_CLINE_TASK.md`
- active roadmap documents such as `docs/ROAD_TO_0_4.md`
- this control procedure.

Document decisions future work needs. Do not create paperwork about paperwork.

## Improvement Rule

The shop itself is a product.

Matthew and Lucy should continually notice demonstrated friction: bad routing, repeated failure, missing context, wasted review, expensive retries, unnecessary handoff or a new tool/model that can materially improve the system.

Lucy is explicitly free to propose better workflows, routing, tools and agent arrangements. Change the smallest useful thing, test it on real work and keep what improves the whole system.

Do not become prisoners of processes we invented ourselves.

## Control Check

When an implementation packet uses the control contract, run:

```powershell
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1
```

Use the checker because it catches real packet/evidence drift, not because ceremony is intrinsically valuable.

## Final Principle

Build fast. Think where it matters. Prove what matters. Keep the truth visible. Spend intelligence carefully. Improve the machine. Then move.
