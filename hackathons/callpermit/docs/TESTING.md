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

The acceptance path does not invoke a real phone provider. A real CALL-E SDK
adapter and live proof are intentionally deferred until the offline checkpoint
is accepted and Matthew explicitly approves the separate live-call boundary.
