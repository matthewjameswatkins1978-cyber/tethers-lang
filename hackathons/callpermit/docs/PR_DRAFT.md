# PR draft — CallPermit for awesome-phone-call-agents

**Target:** `CALLE-AI/awesome-phone-call-agents`

**Suggested title:** Add CallPermit: Tethers-backed permission envelopes for bounded calls

## Summary

CallPermit is a small TypeScript app pattern for bounded appointment calls.
CALL-E handles the natural conversation; a frozen Tethers-backed authority
envelope decides whether the proposed terms are `ALLOW`, `ASK`, or `DENY`
before a call is dispatched. Provider structured results and transcripts remain
untrusted evidence and are reconciled against the same envelope afterward.

## Safety and setup

- Dry-run and deterministic tests are the default; they make no network call.
- The official `@call-e/calle` 0.7.0 adapter is isolated in one effect boundary.
- A real call requires caller-supplied credentials and a separately approved
  test number; the package does not auto-call because an environment variable
  exists.
- Credentials, private transcripts, `node_modules`, and build output are not
  part of the contribution.
- Timeout after provider call-ID allocation is observation-required, never an
  automatic retry.

## Verification

```powershell
npm --prefix hackathons/callpermit run acceptance
git diff --check
```

The local contribution snapshot is in
`hackathons/callpermit/contribution/apps/typescript/callpermit/`.

## Review note

The demo intentionally makes no claim of live mid-call Tethers interception.
The contribution should be reviewed as a pre-call permission and
post-conversation reconciliation pattern.
