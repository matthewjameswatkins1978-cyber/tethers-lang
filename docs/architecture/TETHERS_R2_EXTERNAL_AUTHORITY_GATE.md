# Tethers R2: External Authority Gate

Status: `R2 FROZEN`

Product version: `0.8.0`. The R2 public machine surface (`tethers gate` and
`tethers.authority/1`) required a pre-1.0 minor product bump from `0.7.1`.
The protocol identity `tethers.authority/1` is independent of the product
version and must not be inferred from it.

Implementation:

- `tethers-0.1/host-rust/src/gate_protocol.rs` — frame contract, bounds,
  payload parsing, forbidden authority keys
- `tethers-0.1/host-rust/src/authority_gate.rs` — authority sequencing across
  a session
- `tethers-0.1/host-rust/src/gate_command.rs` — `tethers gate --stdio` loop
- `tethers-0.1/host-rust/tests/r2_authority_gate.rs` — conformance suite

Machine schema: `docs/tethers.authority.1.md`.
Host shape: `docs/architecture/TETHERS_HOST_ARCHITECTURE.md` § Shape D.
Preceding boundary: `docs/architecture/TETHERS_R1_EXTERNAL_HOST_BOUNDARY.md`.

## Core laws

1. Tethers authorises. The external Host executes.
2. A Plan is not permission. A proposed Plan, a prepared decision, and an
   approved Ask each remain short of physical execution.
3. Tethers evaluates current authority. No previous verdict is reusable
   state.
4. Ask is Tethers-owned and exact: one approval binds one exact Action with
   one exact argument digest and is consumed once.
5. COMMIT re-checks at the last responsible moment. PREPARE's decision is
   informational only.
6. Revocation between PREPARE and COMMIT prevents dispatch.
7. Durable intent precedes dispatch readiness. Nothing is armed before the
   Trail records intent.
8. A previous admission is never a reusable permission slip. Replay admits a
   logical execution once.
9. The Gate never executes the effect. `provider_invocations` is always `0`.
10. Gate failure fails closed. Unavailable authority is `unavailable`, never
    silently coerced to allow.
11. No external ontology. The Gate speaks Tethers vocabulary only —
    authority, policy, approval, replay, intent, outcome — and imports no
    foreign system model.

## Ownership

Core law: **Tethers authorises. The external Host executes.**

| Tethers (via the Gate) owns | The external Host owns |
| --- | --- |
| Current policy and scope evaluation | Physical execution and provider invocation |
| Exact Ask approval records and consumption | External execution identity |
| Replay admission and the admission guard | Outcome observation and reporting |
| Durable intent (`prepare_and_record`) and armed/terminal replay states | Application lifecycle, jobs, coordination, recovery |
| Trail intent and outcome entries | Its own durable application ledger |

The Gate is a non-executing authority façade. It answers one consequential
question — *may this exact effect be dispatched now?* — and never spawns the
requested Action, invokes a provider, or writes target files.

## Protocol

`tethers.authority/1` is newline-delimited JSON over persistent local stdio.
Every frame is `{schema, request_id, operation, payload}` →
`{schema, request_id, status, result | error}`. The operation set is closed:
`hello`, `prepare`, `approval_decision`, `commit`, `outcome`, `status`,
`shutdown`. Unknown operations, unsupported schemas, duplicate top-level
fields, duplicate `request_id` values, oversized frames, and forbidden
authority keys are refused with structured `frame.*` errors. Exact field
tables, bounds, error codes, and example frames are frozen in
`docs/tethers.authority.1.md`.

CLI surface:

```text
tethers gate --stdio --config PATH --trail ABS --host-data-root ABS
```

`--stdio` is mandatory; all three paths must be absolute; the config must
exist. Violations fail closed before any frame is accepted
(`GATE_STDIO_REQUIRED`, `GATE_PATH_NOT_ABSOLUTE`, `GATE_CONFIG_NOT_FOUND`).

Callers must never supply authority booleans — `permission`, `within_scope`,
`trusted`, `approved`, `authority_granted`, `granted` — on `prepare` (top
level or nested in `plan`). The frame layer rejects them with
`frame.forbidden_authority_key`. The Gate recomputes every decision itself.

## PREPARE versus COMMIT

| | PREPARE | COMMIT |
| --- | --- | --- |
| Question | What would current authority say? | May this exact effect be dispatched now? |
| Authority state | Read-only evaluation via `evaluate_effective_policy` | Fresh `load_runtime_config` + `prepare_runtime`, re-evaluated |
| Result | `allow_prepared` \| `ask` \| `deny` \| `unavailable` | `tethers.dispatch/1` or a `commit.*` refusal |
| Authorises dispatch | Never (`authorizes_dispatch: false`) | Arms intent for Host-side dispatch only |
| Replay | Not touched (`replay_mutated: false`) | Fresh admission required; guard held |
| Provider calls | 0 | 0 |

PREPARE loads the current runtime, selects the Tether, reconstructs the
proposed Action from the supplied `tethers.plan/1` material, assesses scope,
and evaluates current policy. It stores a `prepared_id` bound to canonical
evaluation material (including the runtime config digest) and, when the
decision is `ask`, creates an exact ApprovalStore record. PREPARE mutates no
replay state and never authorises physical execution.

COMMIT re-does the consequential half against current truth. It refuses a
forged or unknown `prepared_id` (`commit.unknown_prepared`), refuses a second
commit of the same preparation (`commit.already_committed`), re-resolves the
capability under the same store and availability the decision used, and
verifies the allow identity matches the resolved capability
(`commit.identity_mismatch`).

## Approval

Approval is Tethers-owned and exact:

- Created only when current policy evaluates to Ask; the record binds the
  exact Action identity, capability pin, manifest digest, provider identity
  and argument digest.
- Relayed to a human by the Host via `approval_decision`
  (`approve` \| `deny` \| `cancel`); the Gate holds the record.
- Process-local (`ApprovalStore`) with restart expiry: a new Gate process has
  no approval records, and none may be resurrected. Unconsumed approvals do
  not survive a restart.
- One-shot consumption: `commit.approval_consume_failed` if an approved
  record cannot be consumed, and consumption is recorded durably in the
  Trail (`approval_consumed` / `exact_approved_ask`).
- Not standing permission: `authorizes_dispatch` stays `false` after
  approval, and an approval never overrides a fresh Deny (`commit.deny`). A
  wrong, missing, or not-yet-approved approval surfaces as
  `commit.approval_not_ready`. Changed arguments produce a different proof,
  so an old approval cannot authorise new material.

## Current re-admission at the last responsible moment

COMMIT never trusts PREPARE. Between the two phases the Host, the config, the
manifest, the provider set, or the policy may change; COMMIT re-loads
configuration, re-evaluates policy, re-resolves the capability, and performs a
fresh replay admission. The fresh claim is required: a non-fresh admission is
refused with `commit.replay_blocked` and the recovered replay classification
(`replay_blocked_completed_success`, `replay_blocked_completed_failure`,
`replay_requires_manual_resolution`, `replay_persistence_unavailable`).

Refusal matrix proven by the conformance suite — each phase refuses at the
first moment it observes the problem:

- policy rewritten to deny after PREPARE → COMMIT returns `commit.deny`
- provider observation emptied at COMMIT → `commit.unavailable`
- manifest digest drift → PREPARE already returns `unavailable`; COMMIT still
  refuses under any `commit.*` code
- provider identity substitution → same: `unavailable` at PREPARE, refused at
  COMMIT
- scope violation → `deny` before any dispatch
- approval recorded, then policy denied → COMMIT returns `commit.deny`
  (approval cannot bypass current authority)

Multi-step work re-admits per step: permission for step 1 never implies
permission for step 2.

## Durable intent before dispatch readiness

COMMIT ordering is fixed:

1. fresh policy evaluation and (when Ask) exact approval precheck;
2. capability resolution under the decision's own store and availability;
3. fresh replay admission (guard acquired);
4. approval consumption, recorded to the Trail, only after the fresh claim;
5. `publish_intent` on the replay guard;
6. `dispatch::prepare_and_record` — durable Trail intent;
7. `publish_armed` — dispatch readiness;
8. return `tethers.dispatch/1` while still holding the guard.

Failure at any step aborts before arming (`commit.intent_failed`,
`commit.armed_failed`, `commit.approval_consume_failed`) and control returns
to the caller with the session still usable. Durable intent therefore exists
before the Host is told the effect is armed, and the conformance suite
asserts the Trail contains `execution_id` before any host-side effect.

## Outcome correlation

The Host executes physically, then reports one `outcome` bound to the
`execution_id` from `tethers.dispatch/1`:

- `execution_id` must be a committed Gate execution
  (`outcome.unknown_execution` otherwise);
- classification is `succeeded` \| `failed` \| `uncertain`, with `succeeded`
  requiring `result` and `failed` requiring `error`;
- `result` and `error` are mutually exclusive;
- `external_execution_identity` must not equal the Tethers `ActionId`
  (`outcome.identity_collapse`) — the external execution identity remains
  distinct from Tethers Action identity;
- a committed dispatch must report `attempted: true`
  (`outcome.not_attempted` otherwise).

The Gate writes an `OutcomeEntry` to the Trail, computes the durable outcome
digest, and publishes terminal replay state on the guard held since COMMIT.
A repeat of the same classification is idempotent (`idempotent: true`); a
different classification for the same execution is refused
(`outcome.conflict`).

## Crash recovery

The admission guard is held in-process from COMMIT to OUTCOME. Recovery
semantics are explicit, never guessed:

- **Restart before OUTCOME.** The in-memory committed map is empty, so a late
  `outcome` for the prior execution is refused as `outcome.unknown_execution`
  rather than reconstructed. The durable Trail still holds the intent, and
  `status.recovery_required` lists it as
  `committed_outcome_incomplete`.
- **Outcome recorded but guard lost** (restart between commit and terminal
  publication): the Trail outcome is written, but `replay_terminal` reports
  `recovery_required`. The Gate never claims success or failure beyond the
  recorded observation.
- **Intent without outcome.** `status` scans the Trail for intent records
  lacking a matching outcome and reports them, bounded to
  `MAX_STATUS_ENTRIES`.
- **Approvals.** Process-local and restart-expiring; `status` on a fresh
  process shows no pending approvals.
- **No silent reset.** Recovery surfaces unresolved state; it never repairs
  authority by discarding it.

## Non-execution guarantee

The Gate never invokes a provider. `provider_invocations` is `0` on every
surface (`hello`, `prepare`, `approval_decision`, commit record, `outcome`,
`status`, `shutdown`, final CLI envelope), and the counter is structural, not
observable luck: the Gate module contains no provider dispatch path.
Related invariants on the same surfaces:

- `hello.authority_granted: false`
- `prepare.authorizes_dispatch: false`
- `approval_decision.authorizes_dispatch: false`
- `tethers.dispatch/1.authorizes_physical_execution_by_tethers: false`
- `prepare.execution`: `performed: false`, `provider_invocations: 0`,
  `replay_mutated: false`

Physical execution belongs to the Host, which must report the outcome
(`host_must_report_outcome: true`).

## Live revocation between prepare and commit

Revocation is tested, not assumed. The conformance suite rewrites the runtime
config to `deny` after a successful `allow_prepared` PREPARE and proves that
COMMIT returns `commit.deny`, the external Host fixture performs zero effects,
no marker file exists, and `provider_invocations` remains `0`. The same holds
for capability removal (→ `commit.unavailable`), manifest drift, provider
substitution, scope violations, and post-approval policy changes.

## Plan is not permission

The claim is enforced structurally rather than documented as intent:

- `tethers plan` (R1) reports `authority.granted: false`,
  `execution.performed: false`, `provider_invocations: 0`.
- Gate `hello` reports `authority_granted: false`.
- Gate `prepare` reports `authorizes_dispatch: false` for every decision,
  including `allow_prepared`.
- Callers cannot inject authority booleans into `prepare`
  (`frame.forbidden_authority_key`), and unknown injected fields never change
  the recomputed decision.
- Only COMMIT produces a `tethers.dispatch/1`, and even that record states
  `authorizes_physical_execution_by_tethers: false`.

## What R2 does not change

- The R0 ownership model and its frozen content; Shape D is an addition, not
  a revision. `docs/architecture/TETHERS_HOST_ARCHITECTURE.md` keeps its
  `R0 RECOVERED AND FROZEN` Status line.
- OCaml/Core semantics, Tether syntax, Plan meaning, or the R1
  `tethers plan --json` boundary, which remains the full-external-Host
  (policy-owning) path.
- The reference Host's own ability to execute. Shape A remains legitimate.
- Approval durability: R2 keeps process-local, restart-expiring approvals.
  A durable approval store is a separate design gate.
- No Resolve, Lantern, or Omen ontology or dependency is introduced.
