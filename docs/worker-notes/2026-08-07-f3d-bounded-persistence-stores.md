# Worker Note

Task: `F3d - Remaining bounded persistence stores`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Codex`

Status: `COMPLETE`

Base commit: `40ec42eb2aac108901d428af3cbfe264d3edd6dc`

Implementation checkpoint: `c9fbe555f9c6dd8f72d857dedaf5ca4954c248e2`

## Requested outcome

Complete the F3d bounded-store evidence pass without changing production
persistence behaviour, correct the Publisher Trust scope, and make every F3d
PROVEN claim cite its own exact hard assertion.

## Changes made

- Corrected the packet: Publisher Trust is `PublisherTrustStore` in `trust.rs`.
  `ExactCandidateTrustStore` in `installation_trust.rs` is separate
  installation evidence and is not the Publisher Trust Store.
- Replaced the blanket `StoreRoot` inference with a nine-store evidence map in
  `docs/foundation-pass/PERSISTENCE_INVENTORY.md`. Each PROVEN entry names the
  test and the asserted outcome; dimensions without that proof are marked
  UNVERIFIED.
- Updated the current goal, dashboard, and F3d test-module comments to keep the
  same evidence boundary. `DEBT_LEDGER.md` was not changed because no direct
  production defect was demonstrated.
- Preserved F3b power-loss and directory-entry durability labels, and Local
  Anchor root reparse safety, as UNVERIFIED. Production code was unchanged.

## Decisions and assumptions

- Matthew explicitly reassigned the F3d correction to Codex after the
  independent review found incomplete evidence documentation.
- F3d remains an Amber characterization pass. It does not begin F3e or reopen
  F3c, Trail, Replay, StoreRoot, fixture, protocol, or production-persistence
  scope.
- A shared implementation is not a per-store proof. The inventory therefore
  reports only the properties with a cited hard assertion.

## Evidence

Focused F3d evidence tests:

```text
cargo test --lib f3d --all-features --locked  -> 6 passed, 0 failed
```

Serial verification:

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `cargo check --all-targets --all-features --locked` | PASS (existing warnings only) |
| `cargo test --all-targets --all-features --locked` | PASS (1325 passed, 0 failed) |
| `cargo clippy --all-targets --all-features --locked -- -W clippy::all` | PASS (existing warning inventory only) |
| `just verify` | PASS |
| `just verify-agent` | PASS (1581 passed, 2 skipped) |
| F3d assertion map | `docs/foundation-pass/PERSISTENCE_INVENTORY.md` names each cited test and exact asserted outcome |

## Discoveries

- The original F3d packet misidentified `installation_trust.rs` as Publisher
  Trust. The actual `PublisherTrustStore` and its transition/restart/torn-state
  test are in `trust.rs`.
- Existing tests establish useful bounded facts, but they do not justify a
  universal StoreRoot-derived claim for every consuming store.
- The only F3d source change is evidence-test commentary; the six
  characterization tests already pass and no production defect was found.

## Remaining risks

- Power-loss durability and directory-entry durability remain `UNVERIFIED
  (F3b)` for the bounded stores.
- Local Anchor root reparse safety remains `UNVERIFIED (F3b)`.
- Several individual store dimensions remain unverified where no direct
  negative assertion was available; the inventory identifies them rather than
  inferring proof.

## Smallest next action

Lucy independently reviews this completed F3d evidence correction. Do not begin
F3e without a new authorized packet.

## References

- `docs/CURRENT_CLINE_TASK.md`
- `docs/foundation-pass/PERSISTENCE_INVENTORY.md`
- `tethers-0.1/host-rust/src/f3d_bounded_persistence_stores_evidence.rs`
- `tethers-0.1/host-rust/src/trust.rs`
- `tethers-0.1/host-rust/tests/j24h_installation_evidence_access.rs`
- `tethers-0.1/host-rust/tests/j24j_installation_reconciliation.rs`
