# CallPermit

> CallPermit lets an AI make the phone call without letting it invent your permission.

CallPermit compiles one bounded appointment delegation into a frozen authority
envelope, evaluates the proposed terms through the Tethers 0.7-shaped machine
contract, and only then allows one injected CALL-E-shaped transport to run.
The returned provider evidence is reconciled against the same envelope.

The current build is deliberately offline:

- `src/fake-calle.ts` is the only transport used by acceptance and `npm run dry-run`.
- It never opens a socket, reads `CALLE_API_KEY`, or places a phone call.
- The production CALL-E SDK adapter is not claimed as complete yet.
- No demo recording, public PR, or Devpost submission is included in this pass.

The important cases are deterministic:

| Offered terms | Decision |
| --- | --- |
| Tuesday 15:30, standard service, £45, no extra | `ALLOW` |
| Tuesday 15:30, standard service, £95 | `ASK` / no dispatch |
| £45 plus an unrequested add-on | `ASK` / no dispatch |
| Subscription, membership, deposit, or purchase | `DENY` |
| Malformed input such as `{ "banana": "ferret" }` | `DENY` |
| Provider claims a £95 commitment under a £60 envelope | `POLICY VIOLATION / INVALID RESULT` |

## Run it

From the repository root:

```powershell
npm --prefix hackathons/callpermit run acceptance
npm --prefix hackathons/callpermit run dry-run
```

`acceptance` performs a lockfile bootstrap, TypeScript build, deterministic
tests, and the offline smoke path. It cannot make a real call.

## Authority boundary

The current Tethers 0.7 machine path supplies a validated Tethers request and
response shape. CallPermit checks the request against the frozen envelope,
maps the Tethers result to `ALLOW` / `ASK` / `DENY`, and then applies the
appointment constraints. A provider's `task_completed` flag is evidence only.

There is no claim of live Tethers interception inside an active conversation.
If a recipient offers £95, CallPermit returns a Decision Card for Matthew. A
later approval must create a new envelope and a new atomic call identity.

## Current limitation

The fake transport is the honest stopping point for now. Wiring the current
official `@call-e/calle` SDK, proving one controlled live call, preparing the
community contribution, and recording the final demo are separate follow-up
work and are intentionally not represented as complete here.
