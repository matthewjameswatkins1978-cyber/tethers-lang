# Title

CallPermit

## One-line Summary

A deterministic authority and idempotency layer that lets CALL-E make a real phone call without letting an AI invent permission or duplicate the side effect.

## Problem

An AI phone call is a real-world side effect. A retry can become a second customer call, appointment request, escalation, or interruption. Prompt wording alone is not a reliable boundary for what the agent may do, which target it may call, or whether a repeat is safe.

## Solution

CallPermit freezes a bounded authority envelope before a call. Tethers-backed evaluation produces `ALLOW`, `ASK`, or `DENY`; only `ALLOW` reaches the official `@call-e/calle` 0.7.0 adapter. The returned CALL-E evidence is reconciled against the same envelope afterward. A deterministic identity and persisted registry prevent an identical authorized action from being dispatched twice.

The genuine live integration proof uses CALL-E's official hackathon integration-test hotline with `US` / `en-US` routing and a harmless test prompt. It proves the real CALL-E dispatch path, not UK delivery and not an appointment booking.

## Why This Matters

Conversational intelligence can negotiate a conversation, but it should not negotiate its authority. CallPermit makes the control point explicit, machine-checkable, and inspectable before and after an external phone effect.

## How We Used AI

CALL-E handled the live phone conversation and returned the transcript, provider evidence, completion status, and confidence metadata. The live task explicitly requested an integration-test response and prohibited appointments, purchases, subscriptions, payments, deposits, and other commitments. Provider output remained evidence; it was not allowed to redefine the authority envelope.

## How We Used Codex

Codex inspected the existing CallPermit implementation and public contribution PR, added the official-hotline proof profile and provider-evidence mapping, repaired the live runner's unsafe retry edge case, ran the acceptance suite, performed the one authorized live CALL-E run, redacted the evidence, built the public demo page, and prepared this draft. No API key is committed or published.

## Key Features

- Deterministic Tethers-style `ALLOW` / `ASK` / `DENY` authority decisions.
- Proven no-dispatch behavior for `ASK` and `DENY`.
- Official `@call-e/calle` 0.7.0 SDK adapter.
- Explicit CALL-E official-hotline profile using `US` / `en-US`.
- Real completed CALL-E call with returned call identity, transcript, evidence, and 0.95/high confidence.
- Post-call reconciliation that refuses to invent appointment terms from a hotline response.
- Persisted idempotency: the repeat returned the same call identity with zero second dispatches.
- Redacted public evidence and judge-facing public demo page.

## Architecture

1. A bounded appointment authority envelope is frozen and hashed.
2. Tethers-shaped preflight evidence is checked for identity and scope.
3. `ASK` and `DENY` stop before the provider; `ALLOW` compiles one request.
4. The official CALL-E SDK creates and waits for the call using the deterministic identity.
5. Provider result, transcript, evidence, and confidence are persisted separately from authority.
6. Reconciliation treats provider claims as evidence and returns an uncertainty state when the hotline does not return appointment terms.

## Testing Instructions

From the CallPermit repository:

```powershell
npm run acceptance
```

This runs the build, 75 deterministic tests, and the offline fake-CALL-E smoke path. The optional read-only Tethers host check requires an existing `TETHERS_CONFIG` and `TETHERS_ENGINE`; none was invented for this run.

The genuine live evidence is in the ignored local directory `work/live-hotline-20260914/`. The raw evidence contains sensitive call details and must remain local. The public redacted evidence is `work/live-hotline-20260914/public-evidence.json`.

## Public Demo Link

https://callpermit-live-proof-2026.matmusmeows.chatgpt.site

## Public Repository Link

https://github.com/CALLE-AI/awesome-phone-call-agents/pull/584

## Demo Video

Local evidence video prepared at:

`C:\Users\Matmus\Documents\Codex\2026-09-14\referenced-chatgpt-conversation-this-is-an-2\outputs\callpermit-demo-video\callpermit-live-proof-demo.mp4`

TODO: upload this video to YouTube or Vimeo and paste the public URL into the official form. The video is a redacted evidence sequence; it does not simulate CALL-E audio.

## Screenshot Shot List

- Public demo page showing the completed live transport proof.
- Permission matrix showing `DENY` / `ASK` at zero dispatches and `ALLOW` as the sole live path.
- CALL-E result panel showing completed status, returned identity, transcript availability, evidence, and confidence.
- Reconciliation panel showing why the hotline response is not treated as an appointment commitment.
- Idempotency panel showing the same returned identity and zero repeat dispatches.

## Submission Readiness Notes

- The public contribution PR is open and remains the project submission link.
- The public demo URL is live.
- A genuine CALL-E integration proof exists and is preserved in redacted/local evidence.
- The local three-minute evidence video is prepared but still needs a public YouTube/Vimeo upload.
- Nothing has been submitted to Devpost by Codex in this pass.

## Known Limitations

- The official hotline proves CALL-E integration and real dispatch, not UK delivery.
- The hotline returns integration-test evidence rather than appointment terms, so CallPermit correctly marks appointment reconciliation as uncertain.
- The optional Tethers CLI check could not run because no project-specific config/engine pair was available; the deterministic Tethers response/authority path and its tests pass.
- The public page redacts the call ID, destination, and transcript text; the unredacted evidence remains local only.

## TODO Official Form Fields

- Submitter Type: Matthew to choose.
- Country of residence/incorporation: Matthew to choose.
- App status: `Pre-existing and updated`.
- Pre-existing update explanation: use the Solution/How We Used Codex sections above.
- Testing instructions: use the Testing Instructions section above.
- Functional demo URL: use the Public Demo Link above.
- Project submission pull request URL: use PR #584 above.
- CALL-E account email: use the authenticated CALL-E account email on the form.
- Primary use case: `Appointment scheduling & confirmation`.
- One-sentence real-world task: `Bound an AI phone appointment workflow so only an explicitly authorized call can dispatch and a repeat cannot create a duplicate side effect.`
- Eligible Age: Matthew must confirm personally.
- Country eligibility: Matthew must confirm personally.
- Conflict of interest: Matthew must confirm personally.
- Video URL: TODO after YouTube/Vimeo upload.
