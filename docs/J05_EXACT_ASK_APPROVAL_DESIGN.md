# J05 Exact Ask Approval And Resume Design

Status: authoritative Red design  
Accepted by: Lucy, technical authority  
Date: 2026-07-25

## Purpose

Define the smallest host-owned, one-shot approval mechanism that can resume one
exact `ask` Action without becoming a standing permission, bypassing fresh
policy evaluation, or producing false execution claims.

This design supersedes any J05 implementation or draft preserved on
`safety/preserve-local-main-20260725`. That branch is reference evidence only.
No source code is accepted from it automatically.

## Governing Rule

Approval confirms only that one human approved one exact proposed Action proof.
It never replaces policy evaluation.

Every resume performs fresh ordinary resolution first. Approval is considered
only when all non-approval checks pass and the fresh ordinary outcome is still
`ask`.

## Approval Proof

A pending approval record contains these exact fields:

- `approval_format_version`: `"1"`;
- `evaluation_id`;
- `plan_id`;
- `action_id`;
- capability name;
- capability semantic version;
- `argument_digest`;
- `manifest_digest`;
- `provider_identity`;
- `approval_binding_digest`.

`argument_digest` is `sha256:` plus SHA-256 over RFC 8785/JCS canonical bytes of
the complete resolved non-secret Action arguments.

`approval_binding_digest` is `sha256:` plus SHA-256 over RFC 8785/JCS canonical
bytes of every preceding proof field, including `approval_format_version`.

The host compares both the complete constituent fields and the binding digest.
The digest is not a substitute for field-by-field comparison.

Credentials and dispatch-injected secrets are excluded from the Action proof,
approval record, and Trail.

## State Model

A record has exactly one state:

- `pending`;
- `approved`;
- `denied`;
- `cancelled`;
- `invalidated`;
- `consumed`.

Only `pending -> approved`, `pending -> denied`, and `pending -> cancelled` are
human decision transitions.

Only `approved -> consumed` authorises one resume.

Any proof mismatch or fresh non-approval gate failure moves a matching
`pending` or `approved` record to `invalidated` before the resume returns.
Terminal records are never reused or silently returned as new pending requests.
A later attempt creates a new record with a new approval identity.

Host-process restart invalidates all non-terminal in-memory records for 0.2.
Durable approval restoration is deferred until a separate persistence contract.

## Request Behaviour

When fresh policy resolution returns `ask`:

1. Build the exact proof.
2. Look up the exact approval identity and binding digest.
3. If no record exists, create one `pending` record and append one
   `approval_requested` Trail entry.
4. If the exact existing record is still `pending`, return the same pending
   record without duplicating the Trail entry.
5. If the existing record is terminal, do not reuse it. Create a new approval
   identity and new pending record, or return a precise terminal-state response
   requiring a new request. The implementation must choose one representation
   and test it consistently; it must never treat a terminal proof as pending.

No intent, executor call, execution outcome, or standard result Anchor occurs.

## Human Decision Behaviour

Only a host-recognised human decision boundary may approve, deny, or cancel a
pending record. AI, Tethers, providers, manifests, and callers cannot
self-approve.

The store transition occurs first. The corresponding Trail entry is appended
only after the transition succeeds:

- `approval_granted`;
- `approval_denied`;
- `approval_cancelled`.

If the transition fails, the Trail must not claim it occurred. The failure is
reported separately and does not change the record.

## Resume Behaviour

A resume API receives the proposed Action and approval identity, not a
caller-supplied final `PolicyEvaluation`.

The resume seam itself must:

1. Resolve the current declared Action identity and pins.
2. Resolve the current admitted manifest and provider binding.
3. Validate current input schema.
4. Obtain the current host-owned scope assessment.
5. Evaluate current Deny/Ask/Allow policy without applying approval.
6. Rebuild the complete approval proof from current values.
7. Compare every proof field and both digests with the stored record.
8. Continue only when the fresh ordinary result is exactly `ask` and the exact
   record is `approved`.
9. Atomically transition `approved -> consumed`.
10. Append `approval_consumed` only after successful consumption.
11. Issue the existing policy-owned exact Allow proof used by durable intent
    preparation.

Fresh `deny` or `unavailable`, schema failure, scope failure, stale binding,
changed arguments, changed identity, changed version, changed manifest,
changed provider, or changed digest invalidates a matching pending or approved
record and prevents dispatch.

A consumed approval is never restored, including when intent recording or the
provider call later fails. A new attempt must begin a new Ask.

## Error And Trail Truth

Store errors retain their real category. Missing, denied, cancelled,
invalidated, and already-consumed records must not all be reported as
`approval_invalidated`.

A Trail entry may describe only a state transition that actually completed.
Each entry carries known Action IDs, capability name/version, manifest digest,
provider identity, reason code, and `argument_digest`, with credentials absent.

## Dispatch And Result Anchors

Pending, denied, cancelled, invalidated, stale, unavailable, and failed-resume
Actions are unattempted:

- no durable intent;
- no executor/provider call;
- no execution outcome;
- no `capability.succeeded`, `capability.failed`, or
  `capability.uncertain` Anchor.

Only a successfully consumed exact approval may produce the existing Allow proof
that can cross the intent-first dispatch boundary.

## Required Verification Matrix

Focused tests must prove separately:

1. one exact pending request and no duplicate request Trail;
2. successful human approval transition and truthful Trail ordering;
3. successful one-shot resume and atomic consumption;
4. second resume cannot reuse a consumed record;
5. changed arguments invalidate;
6. changed evaluation ID invalidates;
7. changed plan ID invalidates;
8. changed Action ID invalidates;
9. changed capability name invalidates;
10. changed capability version invalidates;
11. changed manifest digest invalidates;
12. changed provider identity invalidates;
13. changed argument digest invalidates;
14. changed binding digest invalidates;
15. fresh Deny invalidates and prevents dispatch;
16. fresh Unavailable invalidates and prevents dispatch;
17. fresh schema failure prevents dispatch;
18. fresh scope violation prevents dispatch;
19. fresh scope-not-established prevents dispatch;
20. human denial prevents dispatch;
21. human cancellation prevents dispatch;
22. restart expiry prevents reuse;
23. intent-write failure does not restore consumed approval;
24. terminal records are never returned as new pending requests;
25. each store-transition failure produces no false Trail transition;
26. every unattempted branch produces zero standard result Anchors;
27. credentials never appear in records or Trail data;
28. deterministic proof construction repeats byte-for-byte for identical input.

The full relevant Rust, host integration, protocol, OCaml, packet, whitespace,
and Git checks must also pass.

## Implementation Boundary

J05 may add a focused approval module and the smallest host orchestration seam
needed to call it. It must not:

- change Tethers language or OCaml planner semantics;
- introduce standing approval;
- persist approvals across restart;
- add a GUI or remote decision service;
- implement J06 deadlines or J07 uncertain outcomes;
- transplant the preserved `main.rs` wholesale;
- import obsolete workflow-control files from the safety branch.
