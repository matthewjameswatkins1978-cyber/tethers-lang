# Independent Review Note

Task: `J24K3e1 - Read-only disabled installation publication preparation`
Reviewer: `Lucy`
Status: `ACCEPTED`
Accepted main before merge: `fe4f0e84569e793be3c0e8818799ac36e895da1a`
Reviewed worker tip before this note: `bc9731217b8b5cbf83761de0281019b4e651b2db`
Implementation checkpoint: `6a82dd529a47f2561234e72a8b7154ede92cabb0`
Verification checkpoint: `e255e8b3875af3e270f861a523289af5106b600b`

## Review outcome

Accepted for routine fast-forward merge.

The crate-private J24K3e1 preparation boundary:

- regenerates the authoritative J24J plan and requires exact equality with the supplied before-plan;
- requires the fresh action to be `PublishDisabledInstallation` with absent installed-state pins;
- requires an idle J24K3d1 recovery plan before transaction identity is generated;
- reloads the exact plan-pinned candidate, exact-candidate trust, launch, conformance and approval chain from authoritative stores;
- revalidates quarantine state, current trust, current conformance and the complete approval chain;
- precomputes one immutable disabled installed record without filesystem mutation;
- constructs the publication intent only through the accepted precomputed-record boundary;
- reruns complete prepared-intent recovery evidence validation;
- requires recovery to remain idle after preparation;
- returns only a sealed crate-private prepared value.

No intent file, staging directory, final destination or installed record is created. No recovery mutation, installation lock, public executor wiring, ordinary installation action, public API, dependency or Cargo.lock change is introduced.

## Shared installed-record constructor

The extracted `build_disabled_installed_record` helper is pure. Installed identity, destination and creation time are supplied by each caller. It centralises only schema field derivation, disabled-binding order and record-digest coverage.

The legacy `install_disabled_with_authority` path retains its existing order:

1. current evidence validation;
2. duplicate-release refusal;
3. staging creation and copy;
4. final evidence revalidation;
5. staging-to-destination rename;
6. installed-record construction and validation;
7. immutable record publication.

The J24K3e1 seam reads and validates the existing installed registry, refuses duplicate package release or source candidate, generates one UUID and one timestamp, validates the resulting record, and returns it only in memory.

## Error-classification review

Quarantine byte drift is correctly reported as `candidate_invalid` when the fresh authoritative J24J regeneration detects the drift before J24K3e1's preparation-layer evidence remapping is reached. This preserves the actual authority order and remains fail-closed and mutation-free. Rewriting that earlier failure as `installation_intent_evidence_stale` would obscure where the invalidity was established.

## Packet-scaffold repairs

Two mechanical packet defects were encountered:

- the READY task packet lacked checker-visible numbered required behaviours even though the ten frozen behaviours existed as numbered level-three headings;
- the READY worker-note scaffold lacked the template-required `## Changes made` section, which becomes mandatory when status changes to COMPLETE.

The first correction was explicitly authorised before implementation. Adding the second template-defined heading and factual change summary at completion was appropriate and did not alter scope, acceptance criteria or production behaviour.

## Verification evidence

HY3 recorded:

- 30 direct J24K3e1 tests passed;
- focused Nextest: 30 passed, 0 failed, 0 retries;
- all named J24K3d2 through J24K3a, J24K2, J24J and M3 lifecycle regressions passed;
- full serial `just verify` passed with zero failures;
- task-packet checker passed at COMPLETE;
- `cargo fmt --check` passed;
- `git diff --check` passed;
- Cargo.lock SHA-256 unchanged: `D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB`;
- final working tree clean and synchronized with the remote branch.

The reviewer inspected the actual preparation implementation, shared record constructor, legacy call site, read-only preparation seam, direct test matrix, changed-file boundary and commit ancestry. The Rust suite was not independently rerun by the reviewer.

GitHub shows the three commits after the implementation checkpoint changed only the task packet and worker note. The final remote tip changed only verification documentation after the tested checkpoint.

## Remaining boundary

A later package must freshly revalidate the sealed preparation immediately before creating durable intent state. Durable intent creation, staging construction and verification, destination rename, exact record publication, intent removal, lock-lifetime composition and public executor wiring remain outside J24K3e1.

This independent note is the external fixed point for the review trail and deliberately does not name its own resulting commit.
