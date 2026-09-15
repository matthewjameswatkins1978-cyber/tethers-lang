# Tethers x Resolve01 P3 worker note

Task: `TETHERS x RESOLVE01 / P3 - Live Guard Transport & Outcome Delivery`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Codex`

Status: `IN_PROGRESS`

Base commit: `9fb8517acd1192e9a1dbee8fa70c0f520d6c0fad`

Implementation checkpoint: `WORKTREE`

## Requested outcome

Implement the bounded authenticated Resolve v1 HTTPS adapter below the
accepted P2 seam, plus explicit durable delivery of already-classified Tethers
outcomes, without changing Tethers product semantics or adding Resolve state
access.

## Changes made

The isolated branch is `codex/tethers-resolve-p3-live-transport`. The current
worktree adds `resolve_transport.rs`, exposes it from the host library, adds a
small validated preparation-digest ingress for the outcome journal, and
extends the P4 journal/adapter request with preparation identity so delivery
matches the accepted Resolve S3 contract. The application helper now requires
trusted action and preparation identities rather than deriving them from an
execution-result string. Cargo uses `ureq` with rustls only.

## Decisions and assumptions

The accepted Resolve contract at
`b450305e72815d33357359c6db641d315d6b2975` is authoritative. Tethers' existing
JCS/SHA-256 `approval::digest` is compatible with the Resolve semantic digest
projection. HTTPS is mandatory outside explicit loopback test construction;
timeouts, bodies, identifiers and ScopeKeys are bounded; redirects are off.
Outcome redelivery is explicit and uses P4 idempotency, while admission and
provider retry remain absent.

## Evidence

Passed so far: locked all-target Cargo check; focused `resolve_transport` unit
tests (4 passed); focused `resolve_guard` unit tests (8 passed); Rust format was
applied only to authorized files; the pre-existing P4 conformance API was
updated and compiles as part of the all-target check. A strict Clippy run was
not a clean gate because the accepted repository currently reports roughly 90
pre-existing warnings under `-D warnings`.

## Discoveries

Resolve S3 exposes the required routes in `control_plane/app.py` and validates
the same exact JSON shapes, digest prefixes, bounds and outcome result set. The
local machine has gcloud and the Resolve Python environment, but no Firestore
emulator or Resolve service is currently listening on the documented local
ports. The existing Rust full test run consequently reaches known
cross-language failures when the current OCaml engine is unavailable; this is
an environment prerequisite result, not evidence to weaken product tests.

## Remaining risks

The full repository verification, accepted Resolve emulator-backed smoke, and
independent review are not complete at this checkpoint. The transport tests
currently use an explicit loopback fake server. Before completion, verify
response/body edge cases, run the accepted repository gates, inspect the full
diff, and record exact external limitations if the emulator cannot be started.

## Smallest next action

Run focused outcome/transport tests after formatting, then run the repository
warning-ratchet, packet checker and complete verification route. Attempt the
accepted Resolve local smoke only if its existing emulator/service
prerequisites are available.

## References

- `docs/architecture/TETHERS_RESOLVE01_BRIDGE_P3_LIVE_TRANSPORT.md`
- `docs/architecture/TETHERS_RESOLVE01_BRIDGE_P2_GUARD_ADMISSION.md`
- `C:\dev\resolve-ai\docs\TETHERS_GUARD_PROTOCOL.md`
- `C:\dev\resolve-ai\control_plane\tethers_guard.py`
