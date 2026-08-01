# J18I Worker Note

## Task

J18I - First Plug Kit Implementation Roadmap. Owner: Luna. Amber,
documentation and implementation sequencing only. Control-plane base:
`e028b0b80f1a092f5f4198714c0b7a4477323cc8`. Frozen architecture:
`a5fd63593a9d9acd397030ecd2e27b4f318c87fd`.

## Changes

Created `docs/architecture/TETHERS_J18_IMPLEMENTATION_ROADMAP.md` with the
required six vertical milestones, current-code inventory, bounded packet map,
routing, evidence plan, durable-store sequencing, exclusions and risk register.
Updated J18H status/freeze wording, decision log, current goal, dashboard, task
queue and current packet. No implementation artifact changed.

## Frozen architecture inspected

Inspected accepted J18B Universal Plug Architecture; J18C Socket v1 and MCP
stdio binding; J18D `.tetherplug` package v1; J18E capabilities/effects/scopes;
J18F lifecycle/outcomes/events/conformance; J18G security/trust/credentials/
sandbox; and J18H paper validation. J18H is recorded as accepted with verdict
`VALIDATED`; the architecture freeze is final. Tether `0.1` and released `0.2.0`
behaviour remain unchanged.

## Current implementation inventory

Inspected exact roles in `tethers-0.1/host-rust/src/main.rs`, `lib.rs`,
`runtime_config.rs`, `configured_runtime.rs`, `manifest.rs`, `trusted_store.rs`,
`provider.rs`, `resolver.rs`, `policy.rs`, `approval.rs`, `dispatch.rs`,
`stdio_provider.rs`, `host_execution.rs`, `outcome.rs`, `replay_runtime.rs`,
`replay_windows.rs`, `result_anchor.rs`, `event_admission.rs`, `event_queue.rs`,
`child_process.rs`, `engine_stdio.rs`, `cli.rs`, and `Cargo.toml`. Inspected
`tethers-0.1/README.md`, Rust tests, OCaml engine files, PowerShell verification
scripts, MCP transcripts, capability manifests, runtime configurations, the
local file provider and fixture provider.

## Reuse and extraction findings

The proven 0.2 policy, approval, dispatch, outcome, replay, Result Anchor, FIFO
queue, child supervision and engine session machinery is reused. The smallest
first extraction is a host application seam around `host_execution.rs` and the
retained `stdio_provider.rs` session. `main.rs` must not receive a broad rewrite.
The Socket boundary and discovery catalogue are new seams. Legacy runtime
configuration and manifests remain adapters/evidence, not `.tetherplug` v1.
Package inspection, installed identity, trust, conformance, launch profiles,
durable admission and reference providers are new bounded paths or stores.

## Compatibility strategy

Keep `main` releasable and the 0.2 runtime path working. Do not mutate user
configuration, remove the legacy path, move `v0.2.0`, or silently dual-read a
new format. New Plug paths are host-owned and introduced behind stable seams.
Migration or dual-read requires a separate reviewed packet.

## Six milestones

Exactly six milestones are recorded: Socket seam and 0.2 parity; package
inspection/quarantine/installed identity; trust/launch/conformance gate; File
Tools Action/Query vertical slice; durable local Anchor/lifecycle completion;
and PDF Tools/first Plug Kit release gate.

## Packet map

Twenty proposed packets are mapped as P1-P20. P1-SOCKET-PARITY is explicitly the
first implementation packet after J18I acceptance. It is the smallest Milestone
1 extraction/parity task and excludes package, File Tools and security work.
Packets are proposals, not authorisations.

## Worker routing

Luna/OpenCode handles bounded Green and ordinary Amber work. DeepSeek Pro V4
handles thicker middle integration. Codex Terra High handles Red Windows,
archive/path, cryptography/trust, durable storage and final release gates. Lucy
owns architecture, packet design, review and verdict. Matthew retains product
authority.

## Test and evidence plan

The roadmap requires pure unit, parser/duplicate-key, archive/path adversarial,
Windows process/Job Object, MCP pagination, package/install/trust lifecycle,
policy/scope/approval, durable replay/admission restart, Result Anchor/Trail,
provider conformance, full Rust/OCaml, end-to-end File/PDF and clean-machine or
isolated-host evidence as applicable. Performance defaults require a separate
measurement packet.

## Durable stores and schemas

Installed Plug registry, publisher trust, conformance evidence, credential
metadata, operation replay, external-event admission and Trail remain separate
authorities. Every new store requires schema/version, atomicity/recovery,
permissions/confidentiality, migration/rollback, corruption and no-retry/false-
admission evidence before implementation.

## Risks

The roadmap covers 0.2 regression, archive attacks, identity/digest conflation,
false trust, supervised/isolation confusion, Windows reparse escape, environment
or credential leakage, stale discovery, process survival, outcome errors, replay
or Result Anchor publication, event identity conflicts, durable corruption,
scope growth and deadline pressure. Each has prevention, detection, containment
and an owner in the canonical roadmap.

## Tool bootstrap

Resolved repository tools from the native Windows checkout: `git`, `rg`, `fd`,
`jq`, `gh`, `yq`, and `pwsh.exe`. No software was installed or configured.
The repository guidance requires PowerShell 7 for automation. Rust package
metadata records Rust 1.89 and the existing dependencies; no build or test was
needed because this task forbids implementation changes.

## Evidence

Preflight fast-forwarded `main` to `e028b0b80f1a092f5f4198714c0b7a4477323cc8`,
confirmed a clean worktree, confirmed the frozen architecture base and created
`luna/j18i-first-plug-kit-roadmap`. The current code inventory is based on the
exact files listed above. The roadmap contains exactly six milestone headings
and twenty proposed packets. No Rust, OCaml, Cargo, Dune, opam, script, test,
fixture, manifest, provider, package, schema, store, credential, key or
signature changed.

## Discoveries

The 0.2 host already has strong reusable proof boundaries, especially typed host
execution, intent-first dispatch, exact outcomes, replay admission, Result
Anchors, event-depth checks and Windows Job Object supervision. The main missing
boundary is application ownership: most modules remain binary-owned and the
current stdio provider is not yet a semantic Socket with complete catalogue
invalidation. Durable event admission is intentionally not the existing replay
authority.

## Remaining questions

The later packets must freeze exact capability names and machine schemas for
File/PDF Tools, the installed-registry and admission-store schemas, the local
source event identity, measured resource limits, and the public Plug lifecycle
CLI. J18I deliberately does not decide those implementation details.

## Next action

Lucy reviews the roadmap. If accepted, issue only P1-SOCKET-PARITY as the first
implementation packet. Do not begin package parsing, File Tools or security work
from this roadmap alone.

## References

- `docs/CURRENT_CLINE_TASK.md`
- `docs/architecture/TETHERS_J18_IMPLEMENTATION_ROADMAP.md`
- `docs/architecture/TETHERS_J18_PAPER_VALIDATION.md`
- `docs/architecture/TETHERS_UNIVERSAL_PLUG_ARCHITECTURE.md`
- `docs/architecture/TETHERS_SOCKET_V1.md`
- `docs/architecture/TETHERS_SOCKET_V1_MCP_STDIO_BINDING.md`
- `docs/architecture/TETHERPLUG_PACKAGE_V1.md`
- `docs/architecture/TETHERS_CAPABILITIES_EFFECTS_SCOPES_V1.md`
- `docs/architecture/TETHERS_LIFECYCLE_OUTCOMES_EVENTS_CONFORMANCE_V1.md`
- `docs/architecture/TETHERS_SECURITY_TRUST_CREDENTIALS_SANDBOX_V1.md`
- `tethers-0.1/README.md`
- `tethers-0.1/host-rust/src/host_execution.rs`
- `tethers-0.1/host-rust/src/stdio_provider.rs`
- `tethers-0.1/host-rust/src/replay_runtime.rs`
- `tethers-0.1/host-rust/src/replay_windows.rs`
