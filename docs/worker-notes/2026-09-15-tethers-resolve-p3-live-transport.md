# Tethers x Resolve01 P3 worker note

Task: `TETHERS x RESOLVE01 / P3 - Live Guard Transport & Outcome Delivery`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Codex`

Status: `IN_PROGRESS`

Base commit: `9037862c27688b3715f01ba685e2ea1c2fd1c81d`

Implementation checkpoint: `c791feebd38496b2d7946b3dbfc96547d233ad88`

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
tests (6 passed, 2 ignored); focused `resolve_guard` unit tests (8 passed);
focused `resolve_outcome` unit tests (11 passed); the P4a identity/outcome
regression (1 passed); P2 guard regressions (3 passed); P4 conformance (2
passed); format, builds and the warning ratchet. A strict Clippy run was not a
clean gate because the accepted repository currently reports roughly 90
pre-existing warnings under `-D warnings`.

## Discoveries

Resolve S3 exposes the required routes in `control_plane/app.py` and validates
the same exact JSON shapes, digest prefixes, bounds and outcome result set. The
local machine has gcloud and the Resolve Python environment. The accepted
Resolve service plus Firestore emulator were started for the live smoke and
then stopped cleanly. A prior no-switch verification attempt failed before
dependent suites with the explicit OCaml prerequisite message; the final
verification used the repository's external OCaml 5.5 switch and current
engine provenance.

## Remaining risks

The transport tests use an explicit loopback fake server and the ignored live
test was run against the accepted Resolve service with its Firestore emulator:
admission, revoked rejection, outcome recording, exact redelivery and conflict
all passed. The branch was rebuilt from accepted main `9037862` after P4a
merged, retaining both P4a identity continuity and P3 preparation/outcome
evidence. Independent review found no publication blocker. Its only substantive
concern was whether the outcome response should echo action and preparation
fields; the accepted Resolve schema instead returns a digest over the complete
semantic request, which the client recomputes and compares. The review also
identified the need to keep the preparation digest in the P4a admission
request; that seam repair is included in checkpoint
`c791feebd38496b2d7946b3dbfc96547d233ad88`.

## Independent review

Reviewer: Gemini, model `gemini-3.8-flash`, thinking level `high`.

Transport status: completed, response complete, model matched. Recommendation:
safe to publish within the frozen P3 scope. The review confirmed the bounded
HTTPS/default, loopback-only HTTP test mode, strict JSON/digest validation,
opaque errors, no provider/replay/Core/Resolve-database changes, and outcome
idempotency/conflict handling. The local fake-server tests and real Resolve
emulator smoke are the deciding evidence; the review's proxy and response
binding objections remain covered by explicit client configuration and the
request digest contract.

## Smallest next action

Run the final evidence route and independent review, then publish the clean
branch for PR acceptance. Keep the canonical main checkout untouched until
the remote merge is verified.

## References

- `docs/architecture/TETHERS_RESOLVE01_BRIDGE_P3_LIVE_TRANSPORT.md`
- `docs/architecture/TETHERS_RESOLVE01_BRIDGE_P2_GUARD_ADMISSION.md`
- `C:\dev\resolve-ai\docs\TETHERS_GUARD_PROTOCOL.md`
- `C:\dev\resolve-ai\control_plane\tethers_guard.py`
