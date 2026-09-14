# CallPermit

CallPermit is a small, safety-first CALL-E app pattern: CALL-E handles the
conversation, while a Tethers-backed authority envelope defines the permission
slip before the call.

The app is intentionally bounded to one appointment offer: weekdays after
13:00, standard service, maximum £60, no extras, subscriptions, memberships,
deposits, purchases, or materially different commitments. `ALLOW` is the only
decision that reaches the CALL-E transport. `ASK` and `DENY` return a human
Decision Card instead.

## Run safely

The canonical implementation and deterministic test suite are in the CallPermit
hackathon package. From the source repository root:

```powershell
npm --prefix hackathons/callpermit run acceptance
npm --prefix hackathons/callpermit run dry-run
```

These commands use the offline fake transport and make no network request.
The official `@call-e/calle` 0.7.0 adapter is isolated from the default path;
it uses `CalleClient.calls.createAndWait`, a structured result schema, the
authority digest as metadata, and a deterministic idempotency key.

## Side effects and credentials

- No call is made merely because `CALLE_API_KEY` exists.
- A caller must construct the SDK adapter explicitly and supply credentials at
  the effect boundary.
- Only an explicitly approved controlled test number may be used for a live
  proof; no purchase or irreversible booking is permitted in the demo.
- A provider result is evidence, not authority. A result outside the envelope
  is never accepted as a successful commitment.
- A provider call ID survives timeout/restart; the app observes that identity
  rather than blindly dispatching a duplicate.

## Review scope

This is a pre-call permission and post-conversation reconciliation pattern. It
does not claim that Tethers interrupts or intercepts a live conversation.

## Genuine integration proof

The final proof made one real call through the official `@call-e/calle` 0.7.0
adapter to CALL-E's publicly announced hackathon integration-test hotline using
US/en-US routing. CALL-E returned a completed call identity, transcript,
provider evidence, and high completion confidence. Reloading the persisted
registry and repeating the same request returned the same identity with zero
second dispatches.

The hotline is a transport test, not an appointment provider, so CallPermit
correctly did not invent appointment terms from the response. The proof makes
no claim about current UK delivery. A redacted judge-facing view is available
at https://callpermit-live-proof-2026.matmusmeows.chatgpt.site.
