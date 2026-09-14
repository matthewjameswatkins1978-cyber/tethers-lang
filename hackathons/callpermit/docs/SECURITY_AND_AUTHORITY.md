# Security and authority boundary

CallPermit separates three kinds of data:

1. Matthew's frozen authority envelope is trusted host-owned policy.
2. The Tethers response is the deterministic pre-call authority assessment.
3. CALL-E-shaped results, transcript text, and `task_completed` are untrusted
   provider evidence.

The envelope digest is computed from canonical JSON and checked before policy
evaluation. The appointment policy denies malformed input, explicitly denied
commitments, and deposits. Price or extra-service changes become `ASK`, never
implicit permission.

After dispatch, reconciliation uses the original envelope digest and the
returned terms. A provider claim that a £95 commitment was made under a £60
envelope is reported as `POLICY VIOLATION / INVALID RESULT`, not success.

The call registry prevents duplicate dispatch. A completed identity is reused;
an in-progress identity is not dispatched again; and any record with a provider
call ID is observed/reconciled rather than retried blindly after a timeout.
Only a failure with no provider ID can be retried safely.

Acceptance and dry-run use `src/fake-calle.ts`, which has no network path and
no secret handling. `src/calle-sdk.ts` is the separately gated official
transport adapter; it receives credentials only from its caller and does not
log them. No live CALL-E call, public PR, purchase, booking, or external
commitment is performed by acceptance or dry-run.
