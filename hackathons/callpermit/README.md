# CallPermit

> CallPermit lets an AI make the phone call without letting it invent your permission.

CallPermit compiles one bounded appointment delegation into a frozen authority
envelope, evaluates the proposed terms through the Tethers machine contract,
and only then allows a CALL-E transport to run. Returned provider evidence is
reconciled against the same envelope.

The repository contains both transport paths:

- `src/fake-calle.ts` is the default deterministic transport for acceptance and
  `npm run dry-run`; it never opens a socket or reads credentials.
- `src/calle-sdk.ts` is the real `@call-e/calle` 0.7.0 adapter. It uses the
  official `CalleClient.calls.createAndWait` API, passes a structured result
  schema, carries the authority digest as metadata, and sends a deterministic
  idempotency key. It is never instantiated by the offline commands.

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

The read-only Tethers boundary proof is explicit and separate:

```powershell
$env:TETHERS_CONFIG = 'C:/path/to/runtime.json'
$env:TETHERS_ENGINE = 'C:/path/to/tethers-engine.exe'
npm --prefix hackathons/callpermit run tethers-check
```

It invokes the installed `tethers check` interface and parses its JSON
envelope. It does not evaluate a call or invoke a provider.

## Authority boundary

The current Tethers 0.7 machine path supplies a validated Tethers request and
response shape. CallPermit checks the request against the frozen envelope,
maps the Tethers result to `ALLOW` / `ASK` / `DENY`, and then applies the
appointment constraints. A provider's `task_completed` flag is evidence only.

There is no claim of live Tethers interception inside an active conversation.
If a recipient offers £95, CallPermit returns a Decision Card for Matthew. A
later approval must create a new envelope and a new atomic call identity.

## Controlled live-call boundary

No live call is run by this repository's tests, smoke command, or proof
command. A caller that has separately obtained explicit human approval may
construct `CalleSdkClient` with `CALLE_API_KEY` and pass it to
`runAppointmentCall`; the API key must stay in the local process and must not
be printed or committed. Use only an approved test number in
`CALLE_TEST_PHONE`, never an inferred recipient or a purchase flow. A timeout
after CALL-E has assigned a call ID is observation-required, not permission to
retry.

The contribution package and honest demo script are under
`contribution/apps/typescript/callpermit/` and `demo/DEMO_SCRIPT.md`.

## Official CALL-E integration-test profile

The explicit live-proof runner supports only `CALLPERMIT_LIVE_PROFILE=official-hotline`.
That profile targets CALL-E's publicly announced hackathon integration-test
hotline with `US` / `en-US` routing. It is a CALL-E test destination, not
Matthew's recipient and not evidence that UK delivery is currently available.

The live proof uses a harmless integration-test prompt and keeps the API key
in `CALLE_API_KEY` only. It never falls back to an arbitrary `CALLE_TEST_PHONE`;
if that variable is present it must match the named profile. `ASK` and `DENY`
stop before the SDK boundary. A repeat is attempted only after the first
provider response contains a call ID; otherwise replay is skipped to avoid
turning uncertain provider state into a second call.

The earlier controlled UK attempt reached the genuine SDK boundary but was
rejected before call creation by current runtime routing restrictions. No UK
call ID, transcript, structured result, or UK delivery claim is represented
as successful. The official hotline profile proves the CALL-E integration and
real dispatch path only; it does not prove UK route availability.
