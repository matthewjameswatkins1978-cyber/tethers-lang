# CallPermit demo script (about 3 minutes)

This is an honest offline demo unless Matthew has separately approved one
controlled test call. Do not imply that Tethers intercepts or stops speech
mid-call: CALL-E handles the conversation; Tethers defines the permission slip
before the call.

## Recording sequence

1. **0:00–0:25 — thesis.** Show the README and say: “CallPermit lets an AI
   make a bounded appointment call without letting it invent permission.”
2. **0:25–1:05 — frozen authority.** Show the envelope digest and the policy:
   weekdays after 13:00, standard service, maximum £60, no extras,
   subscriptions, memberships, deposits, or purchases.
3. **1:05–1:45 — pre-call decision matrix.** Run:

   ```powershell
   npm --prefix hackathons/callpermit run acceptance
   ```

   Point out the deterministic Tuesday 15:30 / £45 `ALLOW`, £95 `ASK`,
   extra-service `ASK`, and subscription `DENY` cases. No phone call is made.
4. **1:45–2:15 — transport boundary.** Show `src/calle-sdk.ts` and explain
   that it calls the official `CalleClient.calls.createAndWait` API only after
   the frozen authority returns `ALLOW`; the default fake transport keeps this
   recording offline.
5. **2:15–2:45 — evidence reconciliation.** Show the policy-violation test:
   provider evidence claiming £95 is not accepted as success. A deferred or
   contradictory result becomes a Decision Card for Matthew.
6. **2:45–3:00 — recovery.** Show the persisted call identity and explain that
   a timeout after CALL-E assigns an ID is observed, never blindly retried. A
   later approval must create a new envelope and a new atomic call identity.

## Optional Tethers proof insertion

If the local machine has a compatible Tethers fixture, insert the output of:

```powershell
$env:TETHERS_CONFIG = 'C:/path/to/runtime.json'
$env:TETHERS_ENGINE = 'C:/path/to/tethers-engine.exe'
npm --prefix hackathons/callpermit run tethers-check
```

Label this as the read-only Tethers host check. The installed host on the
development machine may be an older schema revision than the checked-out
Tethers source; do not hide that mismatch or present a failed live call as
evidence.

## Claims checklist

- Say “offline deterministic proof” when using the fake transport.
- Say “official SDK adapter” only for request construction and injected mapping
  evidence unless a real approved call was actually completed.
- Do not show API keys, private transcripts, phone numbers, or fabricated video
  links.
- Do not claim mid-call interception, automatic booking, or a public PR until
  those things have actually happened.
