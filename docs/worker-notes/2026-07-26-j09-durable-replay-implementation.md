Task: `J09 durable replay protection`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Codex`

Status: `BLOCKED`

Base commit: `edab172d45cbec248a82002a949c2790696bb320`

Implementation checkpoint: `edab172d45cbec248a82002a949c2790696bb320`

## Requested outcome

Implement the frozen J09 durable replay ledger on the dedicated runtime branch,
using the reviewed native Windows NTFS substrate. Preserve the existing public
configuration and do not silently establish replay storage.

## Changes made

No runtime implementation was made. The existing authority-staging commit
`edab172d45cbec248a82002a949c2790696bb320` contains only the exact frozen J09
design and task-packet snapshots. This note and the blocked task state record
the demonstrated stop condition.

## Decisions and assumptions

No frozen architecture was reopened. The worker did not infer that optional
audit `TRAIL_PATH` authorises a replay root, because the design separates Trail
evidence from host-owned replay authority and requires explicit provisioning.

## Evidence

- Inspected `docs/J09_DURABLE_REPLAY_DESIGN.md`, which requires `FORMAT.json`
  during explicit host provisioning and prohibits startup or lookup creation.
- Inspected `tethers-0.1/host-rust/src/main.rs`: the only runtime storage input
  is optional `TRAIL_PATH`, and its parent is created with `fs::create_dir_all`
  before `dispatch::FileTrail::open`.
- Inspected `tethers-0.1/host-rust/src/main.rs`: the current dispatch boundary
  requires exactly one Action, so it cannot silently provide the design's
  per-Action host lifecycle either.
- Ran `git diff --check`; it passed before the blocked-record update.
- Ran the task-packet checker against the exact staged packet; it correctly
  rejected design commit `d67771f...` as a non-ancestor of runtime HEAD
  `edab172...`, motivating the packet checkpoint alignment in this record.

## Discoveries

The runtime branch descends from accepted `main` at
`e679338e2887510d907d3b1c77eaf7a922dfad37`. The reviewed design checkpoint is
deliberately on a separate branch, so the packet must cite a runtime-branch
checkpoint for the repository checker while retaining the reviewed design SHA
as authority provenance.

## Remaining risks

Implementing against a path inferred from Trail storage would create an
unapproved durable-data policy, could silently provision an empty ledger, and
would fail the J09 root-authority invariant. The requested 47-case evidence
cannot honestly be produced until a replay-root lifecycle is approved.

## Smallest next action

Freeze one product decision: either add an explicit provisioned replay-root
configuration/lifecycle to the host, or authorise a precise deterministic
mapping from an existing explicitly provisioned host data root. Then issue a
new bounded J09 runtime packet.

## References

- `docs/J09_DURABLE_REPLAY_DESIGN.md`, storage model and implementation boundary
- `tethers-0.1/host-rust/src/main.rs`, CLI Trail path and dispatch boundary
- `edab172d45cbec248a82002a949c2790696bb320`
- `d67771ff2d93e7fe0909835e13c0988fa10a0c18`
- `e679338e2887510d907d3b1c77eaf7a922dfad37`
