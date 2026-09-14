# Testing

Run:

```powershell
npm --prefix hackathons/callpermit run acceptance
```

The command runs `npm ci`, `tsc`, the Node test runner, and the offline CLI
smoke. The test suite covers:

- deterministic envelope digests and immutability;
- current Tethers request/response-shaped ALLOW, ASK, DENY, and fail-closed
  malformed handling;
- £45 ALLOW, £95 ASK, unrequested-extra ASK, and forbidden-subscription DENY;
- incomplete or contradictory provider evidence;
- policy-violation detection when a provider claims an out-of-envelope
  commitment;
- persisted authority digest and new identity for a new frozen envelope;
- duplicate dispatch prevention across registry persistence/restart;
- no dispatch when authority is `ASK` or `DENY`.

The acceptance path does not invoke a real phone provider. The official SDK
adapter is covered with an injected provider-shaped response, so mapping and
request construction are tested without a network. A live proof remains a
separate human-approved boundary.

`npm run tethers-check` invokes the installed Tethers host's read-only `check`
interface when `TETHERS_CONFIG` and `TETHERS_ENGINE` are supplied. It is kept
separate from acceptance because Tethers installation paths and host state are
machine-specific.
