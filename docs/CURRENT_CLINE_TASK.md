# Current Implementation Task

Control contract: `1`
Task: `J19-M4 - Autonomous File Tools Plug Vertical Slice`
Owner: `Luna / OpenCode`
Status: `IN_PROGRESS`
Task colour: `Amber`
Route: `Luna in one continuous autonomous Milestone 4 implementation`
Base branch: `main`
Accepted M3 baseline: `8cd8958d4880595dfff5e38ab5ec26de940944df`
Frozen architecture base: `a5fd63593a9d9acd397030ecd2e27b4f318c87fd`
Branch: `opencode/j19-m4-file-tools-plug`
Worker note: `docs/worker-notes/2026-08-02-j19-m4-file-tools-plug.md`

## Start rule

Use Luna in OpenCode. Fetch the remote branch above, switch to it, require a clean worktree, and record the exact branch HEAD containing this control packet in the worker note before implementation.

Read `AGENTS.md`, this task, the M3 worker note, and the governing architecture documents. Run:

```powershell
just tools
```

Use normal commits and normal pushes only. Do not amend or rebase published work, force-push, move `main`, move tags, create a release, or begin M5.

## Mission

Complete all of Milestone 4 in one coherent autonomous run:

1. `P11-FILE-CONTRACT`
2. `P12-FILE-PROVIDER`
3. `P13-FILE-END-TO-END`

Continue automatically through ordinary engineering decisions. Make small coherent commits at useful checkpoints, run focused tests after each packet, and run the complete regression suite at the end.

Return only with:

`M4 COMPLETE - FIRST LIVE FILE TOOLS PLUG`

or a genuine major blocked report after two materially different evidence-based attempts.

Do not stop because a module boundary, schema spelling, fixture shape, compiler error, test harness, Windows API detail, package builder, or integration seam requires thought. Luna owns those ordinary decisions within the frozen rules.

## Objective

Turn one accepted M3 installed-disabled Plug into the first explicitly enabled operational Plug, then prove one bounded File Tools Query and one exact File Tools Action through the complete host path:

inspect -> quarantine -> candidate -> trust -> supervised conformance -> approval -> installed-disabled -> explicit enablement -> active exact binding -> discovery -> policy -> durable intent -> invocation -> canonical outcome -> replay terminal -> Result Anchor -> Trail.

The first operational Plug is credential-free, local-only, Windows x86_64, MCP 2025-11-25 over stdio, supervised but not isolated, and has no network access.

## Required final state

At M4 completion:

- a real `.tetherplug` File Tools package exists as committed deterministic reference material;
- it can pass the accepted M2/M3 package, trust, conformance and installation gates;
- explicit host authority can enable exactly one installed Plug identity;
- enablement creates exact active capability bindings only after current trust, installation, conformance, package, payload, manifest, provider, protocol and launch evidence are revalidated;
- disablement removes operational availability without deleting historical evidence;
- a bounded file metadata/read Query works only inside an exact host-approved scope;
- an exact file move Action works only inside approved source/destination scopes and refuses overwrite;
- host policy, approval, durable intent, replay, outcome, Result Anchor and Trail ordering remain intact;
- unattempted, succeeded, failed and uncertain paths are demonstrated without inventing a fourth attempted outcome;
- no Anchor admission, credential delivery, network listener, PDF feature, arbitrary provider marketplace, auto-update or M5 behaviour exists.

## Governing contracts

Implement against:

- `docs/architecture/TETHERS_SOCKET_AND_MCP_BINDING_V1.md` or the accepted equivalent;
- `docs/architecture/TETHERPLUG_PACKAGE_V1.md`;
- `docs/architecture/TETHERS_CAPABILITIES_EFFECTS_SCOPES_V1.md`;
- `docs/architecture/TETHERS_LIFECYCLE_OUTCOMES_EVENTS_CONFORMANCE_V1.md`;
- `docs/architecture/TETHERS_SECURITY_TRUST_CREDENTIALS_SANDBOX_V1.md`;
- `docs/architecture/TETHERS_J18_IMPLEMENTATION_ROADMAP.md`;
- accepted M1 Socket/application seam;
- accepted M2 package/candidate boundary;
- accepted M3 trust, supervised launch, conformance, approval and installed-disabled stores.

Preserve released 0.2 behaviour. Do not reinterpret legacy runtime configuration or legacy manifests as installed Plug state.

## Frozen architectural truths

- Tethers plans; the host authorises and executes.
- The Socket is semantic; MCP is the binding; stdio is the transport.
- Provider code remains untrusted after signing, conformance, installation and enablement.
- Supervision is not isolation.
- Signature is not trust; trust is not conformance; conformance is not installation approval; installation is not enablement; enablement is not per-action permission.
- Current trust must be checked before every operational launch.
- Exact installed bytes and manifests must be revalidated before operational launch.
- No shell, PATH selection, command concatenation, ambient environment inheritance or production credential delivery.
- No automatic retry.
- Attempted outcomes remain exactly `succeeded`, `failed`, `uncertain`.
- Unattempted is a host execution state, not a canonical attempted outcome.
- Durable intent precedes provider invocation.
- Result Anchor publication occurs only after canonical outcome and replay-terminal publication.
- Trail records what happened and why; it does not grant authority.
- M4 owns explicit enablement and operational Action/Query only. External Anchors belong to M5.

## P11 - File capability contract

Freeze the smallest direct v1 File Tools contract before implementation.

Choose stable capability names under one clear namespace. At minimum provide:

1. one Query for bounded file metadata and optionally bounded UTF-8 content read;
2. one Action for exact file move.

The Query must be read-only. The Action must declare filesystem effects explicitly.

Freeze strict input/output JSON schemas, operation names, manifest identities, effect declarations and permission scopes in committed files and tests.

The contract must define:

- path values as host-resolved scoped relative paths, never unrestricted arbitrary absolute paths from a Tether;
- separate approved roots or equivalent host-owned scope identities;
- canonical path/reparse handling on Windows;
- bounded file size for content reads;
- exact behaviour for missing file, wrong type, invalid UTF-8 when content is requested, destination exists, source equals destination, cross-volume move if unsupported, path escape, junction/reparse encounter, permission refusal and provider loss;
- overwrite is always refused in M4;
- no recursive directory operations;
- no delete, copy, write, glob, arbitrary listing, shell or network capability;
- deterministic output fields and safe diagnostics.

Do not freeze a broader File API for hypothetical future needs. One good Query and one good move Action are enough.

Commit P11 separately after schema and contract tests pass. Record the final names and schema digests in the worker note.

## P12 - File Tools provider and package

Build one self-contained native Windows x86_64 reference provider with no credentials and no network use.

Package it as deterministic `.tetherplug` reference material using the accepted package format. Do not hand-edit inconsistent digest evidence. A small repository-owned deterministic packaging script or build step is acceptable when it produces reproducible exact package bytes and validates its own output.

The provider must:

- implement MCP initialize, complete tools/list and tools/call for only the P11 operations;
- advertise exact schemas matching committed capability manifests;
- reject unknown operations and malformed arguments;
- resolve only host-provided scoped roots/configuration;
- perform real Windows reparse-safe path checks;
- use bounded I/O;
- refuse overwrite;
- emit no canonical authority, policy decision, outcome, Result Anchor or Trail itself;
- write protocol only to stdout and treat stderr as untrusted diagnostics;
- support deterministic conformance fixtures without production user data;
- remain stopped after conformance.

Integrate the package through the accepted inspect, quarantine, candidate, trust, launch, conformance, approval and installed-disabled path. Do not bypass M3 stores by constructing an installed record directly in tests.

Commit P12 separately after focused provider, package and conformance evidence passes.

## P13 - Explicit enablement and end-to-end execution

Add the smallest host-owned explicit enablement model.

Enablement must be a separate durable host authority record or state transition, distinct from installation approval and policy. It must pin the exact installed Plug identity and current evidence needed for readiness.

Before enabling and before each operational launch, require at least:

- installed record is valid and `present_disabled` or the accepted enabled-state transition source;
- exact installed file set and all digests are current;
- current trust or exact developer approval is current;
- installation approval and conformance evidence remain current;
- launch profile remains the accepted credential-free supervised profile;
- exact provider and capability bindings match committed manifests;
- no conflicting active binding exists;
- host-approved scope configuration is valid and reparse-safe.

Enablement must not silently enable every version, provider or capability. Bind only exact reviewed capabilities for one exact installed identity.

Disablement must be explicit, durable and fail closed. Restart/reload must reconstruct the same enabled or disabled state without using package claims as authority.

Adapt enabled installed bindings into the existing resolver/policy/dispatch path rather than creating a parallel execution engine.

Demonstrate:

### Query success

A file inside an approved disposable root is queried through Tethers and returns bounded canonical metadata, and bounded content only when requested and valid.

### Action success

A source file inside an approved source root is moved to an approved destination path with overwrite refused. The exact action travels through evaluation, resolution, policy, intent, Socket invocation, outcome, replay terminal, Result Anchor and Trail.

### Required negative and boundary cases

- disabled Plug is unavailable and unattempted;
- enablement with stale/revoked trust refuses;
- payload or manifest drift refuses;
- scope escape and reparse/junction paths refuse before provider effect;
- destination exists refuses without overwrite;
- policy deny and approval-required remain host-owned and unattempted;
- provider-declared extra operation/schema drift invalidates readiness;
- malformed response is failed or uncertain according to the accepted lifecycle boundary;
- timeout/process loss after invocation is uncertain;
- replay blocks duplicate operation execution across restart;
- no Result Anchor for unattempted;
- exactly one Result Anchor attempt after durable attempted outcome and replay-terminal publication;
- Trail ordering and safe diagnostic redaction are preserved;
- disablement removes availability;
- no external Anchor admission occurs.

Use disposable Windows fixture roots. Never operate on arbitrary user files during tests or demonstrations.

Commit P13 separately after focused vertical-slice evidence and full regression pass.

## Autonomy and judgement

Luna may choose:

- module and type names not frozen above;
- exact durable enablement schema and store layout;
- provider binary layout;
- deterministic package builder implementation;
- capability names and schemas during P11, once frozen in committed evidence;
- bounded constants;
- test and fixture structure;
- thin CLI or harness additions needed to demonstrate M4;
- narrow `justfile` additions;
- conservative reuse/refactoring needed to route enabled Plug bindings through the existing host execution path.

Prefer reuse over a second system. Avoid broad rewrites. Keep commits reviewable.

Do not weaken M3 security to make M4 easier. Do not bypass stores, trust checks, intent, policy, replay, outcomes, Result Anchor or Trail in the happy-path demo.

## Major stop rule

Stop only when one of these remains after two materially different evidence-based attempts:

- frozen contracts contradict each other in a way that changes public semantics;
- safe path confinement or overwrite refusal cannot be proven;
- current trust or exact installed-byte validation cannot be preserved at operational launch;
- the existing dispatch/outcome/replay/Result Anchor ordering cannot support the installed Plug path without architectural change;
- a regression in released 0.2 behaviour cannot be isolated;
- a security boundary would have to be weakened;
- repository corruption, missing required baseline, or unavailable required toolchain makes progress impossible.

A large diff, new fixture, compiler problem, Windows API complexity, package-generation detail, or ordinary cross-module integration is not a major blocker.

## Required verification

Run at minimum:

```powershell
just tools
just fmt
just check
just test-m3
just test-rust
just verify
```

Add and run focused M4 recipes/tests for P11, P12 and P13.

Also run:

- complete OCaml build and tests;
- complete `verify-0.2.ps1`;
- locked debug and release Rust builds;
- deterministic package rebuild comparison;
- real Windows path, junction/reparse, overwrite and process-survivor tests;
- replay restart evidence;
- `git diff --check`;
- final clean-worktree check after the completion ledger commit.

Record exact counts and commands in the worker note. Local test claims are evidence, not CI; state that honestly.

## Completion response

Return only:

`M4 COMPLETE - FIRST LIVE FILE TOOLS PLUG`

plus the branch name and final SHA if OpenCode requires additional text.

Do not begin M5. Do not move `main`. Do not create a release.
