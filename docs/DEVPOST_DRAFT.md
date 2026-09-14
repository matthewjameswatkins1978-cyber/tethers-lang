# CallPermit — Devpost submission draft

## Project name

CallPermit

## One-line summary

CALL-E handles the conversation; Tethers defines the permission slip.

## Short description

CallPermit gives a phone agent a bounded authority envelope before it calls:
weekday appointment, after 1pm, standard service, no more than £60, and no
extras or recurring commitments. Tethers makes the permission decision;
CALL-E conducts the natural conversation; returned structured evidence is
reconciled against the same immutable envelope.

## What we built

The TypeScript entry contains a frozen, digest-bearing authority envelope,
fail-closed request and response validation, deterministic `ALLOW` / `ASK` /
`DENY` fixtures, the official CALL-E SDK 0.7.0 adapter, persisted call
identity, duplicate-dispatch protection, and post-call reconciliation. The
offline demo covers a £45 allowed appointment, a £95 escalation, an
unrequested extra, a forbidden subscription, malformed input, contradictory
evidence, and restart safety.

## How it works

1. Compile the bounded delegation into an immutable authority envelope.
2. Evaluate the proposed appointment through Tethers before dispatch.
3. Dispatch one CALL-E task only for `ALLOW`, with the envelope digest in
   metadata and a deterministic idempotency key.
4. Treat the transcript, structured result, and `task_completed` as evidence,
   not permission.
5. Reconcile the evidence against the original envelope; escalate anything
   outside it as `ASK`, `DENY`, or a policy-violation Decision Card.

## Demo video

**Status:** not uploaded yet. Record using `demo/DEMO_SCRIPT.md` and replace
this line with the public URL only after the video is actually uploaded.

## Links

- Source repository: to be filled with the public repository URL.
- Community contribution PR: to be filled after the PR is opened and accepted
  for submission.

## Honest limitations

This entry does not claim live mid-call interception by Tethers. The default
demo is offline and makes no phone call. A live CALL-E proof, if used, must be a
separately approved call to a controlled test number and must not include a
purchase, booking, or other irreversible commitment.
