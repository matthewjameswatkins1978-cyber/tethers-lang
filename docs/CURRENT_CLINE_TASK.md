# Current Implementation Task

Control contract: `1`
Task: `J19-M3 - Autonomous Trust, Launch, Conformance and Install Programme`
Owner: `Codex Sol Medium`
Status: `IN_PROGRESS`
Task colour: `Red`
Route: `Codex Sol Medium, one continuous autonomous Milestone 3 implementation`
Base branch: `main`
Accepted M2 baseline: `17d2a17468a9d7395d31d4b66b5f6e828f82102c`
Frozen architecture base: `a5fd63593a9d9acd397030ecd2e27b4f318c87fd`
Branch: `codex/j19-m3-trust-launch-conformance`
Worker note: `docs/worker-notes/2026-08-01-j19-m3-trust-launch-conformance.md`

## Control-plane starting rule

Start a fresh Codex Sol Medium session.

Fetch `origin/main`, create or reset only the milestone branch named above from the commit containing this control packet, and record that exact control commit in the worker note.

The accepted implementation baseline is M2 at:

`17d2a17468a9d7395d31d4b66b5f6e828f82102c`

The control-only commit containing this packet authorises Milestone 3 but changes no runtime semantics by itself.

Use normal commits and normal pushes only. Do not amend or rebase published work, force-push, move `main`, move tags, create a release, or begin Milestone 4.

## Tooling preflight

Read `AGENTS.md` and run the repository-owned diagnostic before implementation:

```powershell
just tools
```

Use the installed workshop tools where they reduce friction:

- `rg` for repository and symbol search;
- `fd` for file discovery;
- `jq` and `yq` for structured fixture inspection;
- `gh` for GitHub inspection;
- `pwsh` for Windows verification;
- `just` for the existing repeatable project commands.

The root `justfile` and `scripts/check-dev-tools.ps1` are accepted project infrastructure. Extend the `justfile` only when a thin recipe genuinely reduces repeated command noise. Do not create a second build system, broad wrapper framework, generator layer, or parallel task authority.

## Mission

Complete all of Milestone 3 in one continuous implementation run:

1. `P7-TRUST-SIGNATURE`;
2. `P8-LAUNCH-PROFILE`;
3. `P9-CONFORMANCE-GATE`;
4. `P10-INSTALL-APPROVAL`.

Continue automatically from P7 through P10. Make bounded commits at coherent checkpoints, run focused evidence after each checkpoint, and finish with the complete regression matrix.

Return only at:

`M3 COMPLETE - INSTALLED DISABLED PLUG`

or at a genuine major stop condition defined in this packet.

Do not pause merely because a decision is difficult, a diff is larger than expected, a test needs a fixture, a store needs a schema, or ordinary compiler-guided untangling is required. Use engineering judgement within the frozen contracts.

## Required final state

Milestone 3 alone may transition one accepted M2 installation candidate through these separate gates:

1. exact package and payload evidence revalidated;
2. detached Ed25519 signature evidence verified;
3. signing key resolved through host-owned trust or explicit unsigned developer-mode approval;
4. exact supervised conformance launch prepared from accepted host-owned candidate material;
5. host-orchestrated conformance run under bounded test conditions;
6. provider stopped;
7. immutable conformance evidence reviewed;
8. explicit installation approval recorded;
9. accepted bytes published into a host-owned immutable installation location;
10. immutable installed-Plug record and exact provider/capability bindings created;
11. Plug remains present but disabled.

At M3 completion the Plug is:

- installed as host-owned immutable material;
- explicitly approved for installation;
- bound to exact package, payload, manifest, provider, protocol and conformance evidence;
- present but disabled;
- absent from active runtime availability;
- incapable of operational invocation;
- incapable of Anchor admission;
- incapable of receiving production credentials;
- incapable of silently enabling itself.

M3 does not implement operational enablement. The first enablement and real capability invocation belong to M4.

## Governing contracts

Implement against the accepted repository documents and accepted M1-M2 code:

- `docs/architecture/TETHERPLUG_PACKAGE_V1.md`;
- `docs/architecture/TETHERS_SOCKET_AND_MCP_BINDING_V1.md` or the accepted Socket/MCP contract present in the repository;
- `docs/architecture/TETHERS_CAPABILITIES_EFFECTS_SCOPES_V1.md`;
- `docs/architecture/TETHERS_LIFECYCLE_OUTCOMES_EVENTS_CONFORMANCE_V1.md`;
- `docs/architecture/TETHERS_SECURITY_TRUST_CREDENTIALS_SANDBOX_V1.md`;
- `docs/architecture/TETHERS_J18_IMPLEMENTATION_ROADMAP.md`;
- the accepted M1 Socket/application seam;
- the accepted M2 package inspector, quarantine boundary and candidate registry.

Where a frozen document names a conceptual record but does not freeze exact Rust field spelling or nesting, Sol may choose and freeze the smallest direct v1 machine representation consistent with every named distinction, refusal rule and lifecycle gate. Freeze each durable schema in Rust types, committed golden fixtures and the worker note before relying on it.

Do not silently reinterpret legacy 0.2 manifests, runtime configuration, trust stores or fixtures as Plug installation state. The released 0.2 path remains intact.

## Autonomy and decision authority

Sol owns ordinary engineering decisions needed to complete M3, including:

- module names and source boundaries;
- public versus crate-private visibility;
- value types and error enums;
- durable record field spelling and nesting;
- store layout beneath host-supplied roots;
- fixture and test layout;
- Windows API details needed by the accepted contract;
- exact bounded resource constants;
- clean-environment allow-list details for a self-contained native provider;
- launch-profile representation;
- conformance suite representation;
- atomic publication and crash-recovery mechanics;
- commit count and intermediate layout;
- narrow updates to scripts or `justfile` recipes whose assumptions legitimately move;
- mature Rust dependencies needed for accepted cryptography, strict base64url, DER SPKI parsing or Windows process controls.

These are not blockers. Sol should inspect existing patterns, choose a conservative solution, implement it, test it and record the decision.

Do not write custom cryptographic primitives. Use a mature, maintained Ed25519 implementation compatible with Rust 1.89. Pin dependencies through `Cargo.lock`, minimise feature sets, and document each new direct dependency and purpose in the worker note.

## Absolute security truths

These are frozen and must remain visible in code, types, tests and documentation:

- host policy and operating-system containment are separate;
- a provider remains untrusted after signing, conformance, installation or health checks;
- a valid signature proves possession of one key over one exact semantic digest, not safety or publisher identity;
- publisher identity is host-owned trust-store data, never package presentation text;
- no trust on first use;
- conformance is evidence, not trust, permission, installation approval or enablement;
- supervision is not hostile-code isolation;
- the M3 supervised profile is only for explicitly trusted reference/development providers;
- no production credential delivery is authorised;
- no shell, PATH lookup, command concatenation, profile script or launch download;
- no automatic retry;
- no active binding before explicit installation approval;
- no operational binding or invocation before M4 enablement;
- drift invalidates readiness and relevant evidence;
- raw provider stderr is never trusted or copied into durable canonical evidence.

## P7 required result: trust, signature and revocation

### Signature verification

Implement the accepted package-signature v1 contract exactly:

- algorithm is Ed25519 only, as defined by RFC 8032;
- trusted public keys are exact RFC 8410 DER SubjectPublicKeyInfo bytes;
- `key_id` is `sha256:<lowercase hex SHA-256 of the exact DER SPKI bytes>`;
- the exact UTF-8 signing input is:

```text
tethers.tetherplug.signature.v1
<semantic-package-digest>
```

The final newline is mandatory.

The strict signature envelope contains only:

- `signature_format_version` equal to `"1"`;
- `algorithm` equal to `"ed25519"`;
- `key_id`;
- `semantic_package_digest`;
- unpadded base64url `signature`.

Reject duplicate or unknown fields, padded or malformed base64url, signatures not exactly 64 bytes after decoding, wrong key IDs, wrong digest, wrong signing input, unsupported algorithm, duplicate authority from one key and malformed signature filenames.

Signature files remain detached evidence under `signatures/`. They are excluded from semantic package identity and never alter package or manifest authority.

### Host trust store

Create a separate host-owned publisher trust store. It must not share state with packages, candidates, installed records, capability manifests, policy, credentials, Trail or replay.

A v1 key record must preserve at least:

- exact key ID;
- exact DER SPKI bytes or an exact stable encoding of them;
- host-assigned publisher identity;
- trust state: trusted, disabled or revoked;
- optional namespace restriction where implemented;
- creation/update times;
- approving authority evidence;
- optional expiry;
- revocation reason and time where relevant;
- schema version and record-integrity evidence.

Trust changes require explicit host authority. Packages cannot add keys, map publishers or choose trust state. No TOFU exists.

Rotation is explicit. Revoked keys preserve historical evidence but are not currently trusted. Install and later launch re-evaluate current trust. Do not invent trusted timestamps or proof of pre-compromise signing.

Use create-only or explicit state-transition records with crash-aware publication, duplicate/conflict refusal, reparse-safe roots, strict reload and corruption refusal.

### Unsigned developer mode

Implement explicit unsigned developer-mode evidence as a separate host-owned approval path:

- off by default;
- exact semantic digest approval only;
- visibly unsigned;
- no publisher-trust claim;
- no inheritance to another digest or version;
- no automatic enablement;
- no silent conversion to production trust.

Developer mode does not make arbitrary third-party code safe and may only feed the visibly supervised, non-isolated M3 path.

### P7 evidence

Include deterministic evidence for:

- accepted RFC 8032 vectors;
- wrong message, digest, key and signature;
- strict 64-byte unpadded base64url handling;
- exact mandatory final newline;
- RFC 8410 SPKI parsing and key-ID derivation;
- duplicate/unknown envelope fields;
- unknown, disabled, expired and revoked keys;
- host publisher mapping distinct from package `publisher` text;
- trust-store restart, corruption, torn write, duplicate and conflict handling;
- developer mode exact-digest binding and non-inheritance;
- no package-driven trust mutation.

Commit P7 separately after focused and regression evidence passes, then continue directly to P8.

## P8 required result: supervised conformance launch profile

P8 extends the existing Windows child-process and Socket foundations behind an explicit launch-profile boundary. Preserve existing Job Object ownership and released behaviour.

### Honest profile model

Implement and visibly label a `supervised` profile. It provides bounded process ownership and launch hygiene only. It must never be named or reported as isolated, sandboxed, hostile-code-safe or production credential-safe.

AppContainer or another strong isolated profile remains deferred. Do not simulate isolation with a restricted token or Job Object label.

### Exact launch

For an accepted candidate test launch:

- revalidate candidate record, package semantic digest, every payload digest, manifest digest, launch path and exact file set immediately before launch;
- launch only an exact absolute executable resolved from host-owned accepted candidate material;
- never launch from the source archive, Downloads, repository checkout, arbitrary current directory or incomplete staging directory;
- use no shell, `cmd /c`, PowerShell command, PATH lookup, file association, interpreter discovery or command-string concatenation;
- preserve ordered package arguments exactly after host validation;
- use an exact host-approved working directory;
- use explicit stdio handles;
- retain process-tree ownership and kill-on-close semantics;
- bound child count, memory, CPU or wall time where the accepted Windows API can prove it;
- bound protocol line/message sizes, queues and stderr tail;
- require bounded graceful shutdown followed by forced tree termination when necessary;
- prove no unnoticed surviving process.

### Environment from scratch

Construct the provider environment from scratch. Do not inherit the ambient process environment wholesale.

Include only variables required for a self-contained native Windows provider, such as accepted Windows system location values and host-owned disposable temp/scratch paths. Exclude ambient API, cloud, Git, SSH, editor, proxy, repository and unrelated PATH secrets.

PATH must not select the executable or interpreter. No production credential variables are authorised. The conformance fixture should prove representative ambient secret variables are absent from the child.

Use a fresh host-owned bounded scratch directory separate from package payload, installation, candidate registry, trust, conformance evidence, Trail, replay and user resources. Clean it after the test session where safe; preserve only bounded host-owned evidence.

### P8 evidence

Include real Windows tests for:

- exact executable and argument delivery;
- no shell or PATH selection;
- clean environment and absent ambient test secrets;
- exact working directory;
- payload mutation, missing file, additional file and reparse refusal before launch;
- Job Object process-tree termination;
- child/process/resource limits where supported;
- timeout and forced shutdown;
- bounded stdout/protocol and stderr behaviour;
- no surviving child;
- no credential delivery;
- profile labelled supervised and explicitly not isolated.

Commit P8 separately after focused and regression evidence passes, then continue directly to P9.

## P9 required result: host-owned conformance gate

Conformance is host-orchestrated test evidence for one exact package/provider/capability combination. Package tests are untrusted data and cannot certify themselves.

### Conformance execution

Run only after inspection and candidate creation, before installation approval and before active bindings.

Use:

- the accepted supervised profile or a stricter profile if one genuinely exists;
- exact candidate and launch evidence;
- test-only configuration;
- disposable fixture data and scratch;
- no production Tether Sets or effective policy;
- no production credentials;
- no production filesystem or network effects;
- no active provider/capability bindings.

The host owns the suite, test orchestration, pass/fail criteria, deadlines, output validation and evidence publication. Package-provided conformance material may be bounded declarative input only. Do not execute a package-supplied test runner or script.

The provider must be stopped at the end of pass, fail or interruption. A passing run leaves the candidate uninstalled and non-operational until separate review and approval.

### Required conformance categories

Implement the smallest coherent M3 suite covering the generic first envelope:

- static package and candidate revalidation;
- exact launch and clean environment;
- MCP initialize and protocol pin;
- provider identity;
- complete discovery and exact operation/schema agreement;
- catalogue drift/staleness behaviour;
- bounded valid and invalid fixture calls where an accepted fixture provider permits them;
- trusted output/schema validation;
- no hidden retry;
- timeout, malformed response and process-loss observation;
- bounded stderr and redaction/non-persistence;
- shutdown and process cleanup;
- trust, revocation and payload-drift refusal.

Do not implement M4 File Tools capability contracts merely to satisfy P9. Reuse accepted fixture providers and test-owned deterministic operations.

### Conformance evidence store

Freeze a separate immutable v1 conformance-evidence schema. It must pin at least:

- candidate and exact package identity;
- semantic package digest;
- every relevant payload and capability-manifest digest;
- signature/key/publisher or unsigned-developer evidence used;
- launch profile and exact launch identity;
- provider identity and version;
- Socket major, MCP protocol and binding versions;
- host build identity;
- platform and architecture;
- suite version and digest;
- test-configuration digest;
- start/end times;
- each case identity and result;
- bounded safe diagnostics only;
- final disposition: passed, failed or interrupted;
- schema and evidence digest.

Evidence is immutable historical evidence. It grants no permission.

Invalidate or mark stale when package, payload, manifest, capability, launch, Socket, protocol/binding, tested platform, suite or material security boundary changes. Trust revocation must prevent later approval/readiness even when historical conformance remains preserved.

Store states such as not-run, running, passed, failed, interrupted and invalidated as separate conformance state, not package trust or installation state.

### P9 evidence

Prove:

- pass, fail and interrupted paths;
- exact pinning and deterministic evidence digest;
- provider always stopped;
- no installation, approval, binding or enablement after pass alone;
- stale/invalidation after payload, manifest, launch, suite or security-boundary drift;
- trust revocation prevents use of historical pass;
- malformed/corrupt/torn evidence fails closed;
- raw stderr and secrets are absent from durable evidence;
- conformance performs no automatic retry.

Commit P9 separately after focused and regression evidence passes, then continue directly to P10.

## P10 required result: review, installation approval and installed-disabled state

P10 is the only M3 packet allowed to create installed Plug state. It does not enable it.

### Explicit review and approval

Create a host-owned installation-review and approval boundary separate from operational Ask approval.

The review must bind the exact:

- candidate identity;
- package ID, version and semantic digest;
- raw archive and payload evidence;
- signature, key and host publisher evidence, or explicit unsigned developer-mode evidence;
- current trust/revocation state;
- provider identity/version and launch declaration;
- capability identities, manifest digests, effects and scopes;
- supervised profile label and its explicit limitations;
- complete current conformance evidence;
- approval authority, time, schema and evidence digest.

Approval is explicit and create-only. It cannot be inferred from a signature, trust state, successful conformance, package metadata, existing 0.2 configuration or previous version.

Any material drift before publication invalidates the approval attempt.

### Host-owned installation publication

After explicit approval:

- revalidate current trust and all exact candidate/package/payload/manifest/conformance pins;
- copy only accepted files into a new host-owned same-volume installation staging directory;
- use create-new/no-overwrite semantics;
- independently revalidate destination paths and existing ancestors;
- refuse links, junctions, mount points and other reparse points;
- verify sizes and digests during or immediately after copy;
- publish by one atomic rename into a unique immutable installation location;
- mark package payload read-only and keep future mutable provider state/scratch separate;
- never execute during the installation transaction;
- preserve candidate, trust, conformance and approval history in their separate stores.

Execution integrity must be rechecked again before any later launch. M3 does not perform an operational launch from the installed location.

### Installed Plug registry and disabled bindings

Freeze a separate immutable installed-Plug v1 schema. It must preserve at least:

- host-generated installed identity;
- fixed present-but-disabled state;
- package lineage, version and exact semantic digest;
- source candidate identity;
- installation location beneath the configured install root;
- raw archive, payload and manifest identities/digests;
- signature/key/publisher or unsigned developer evidence;
- installation approval identity;
- conformance evidence identity;
- provider identity, version, exact launch path/arguments/working directory;
- supervised profile label;
- exact Socket, MCP and platform pins;
- exact capability identities and provider operations;
- exact disabled binding records;
- creation time, schema version and record digest.

The installed record must not contain:

- enabled state;
- operational policy grant;
- production credentials or credential references;
- active provider session;
- active resolver availability;
- runtime Ask approval;
- operation replay state;
- Anchor admission authority;
- fabricated isolation claim.

Create exact provider/capability binding records only in a disabled state. Do not insert them into active resolver/provider availability, do not create runtime configuration that makes them invocable, and do not create a public enable command.

### P10 evidence

Prove:

- conformance pass alone creates no install or binding;
- explicit approval is required and is bound to exact evidence;
- trust revocation, payload drift, manifest drift or conformance invalidation before publication refuses installation;
- atomic installation and restart/reload;
- torn writes, corruption, duplicate IDs, same-release conflicts, path escapes and reparse destinations fail closed;
- installed bytes and record are immutable and exact;
- installed Plug is present but disabled;
- disabled bindings are absent from active runtime availability;
- no provider launch, Socket invocation, policy availability, replay admission, Result Anchor, external Anchor admission or Trail operation effect occurs;
- no credentials are stored or delivered;
- M4 enablement remains impossible without a new authoritative packet.

Commit P10 separately after all M3 evidence passes.

## Durable-store rules

Publisher trust, unsigned developer approval, conformance evidence, installation approval and installed Plug state are separate stores and state families. They may share a small audited persistence utility but must not collapse into one generic status file or one authority-bearing record.

For every durable store:

- freeze schema first in Rust types and committed golden fixtures;
- use strict JSON without duplicate or unknown fields;
- use canonical covered bytes and a record digest where appropriate;
- use host-generated IDs;
- use create-only or explicit immutable state-transition records;
- use same-volume temporary publication and atomic rename;
- flush file data and directory metadata where the host can prove it;
- refuse torn temporary files, malformed records, unexpected files and duplicate/conflicting identities;
- revalidate root confinement and Windows reparse safety on open and before publication;
- reload strictly across process restart;
- preserve historical evidence without making it current authority.

## Required source boundaries

Keep these concerns distinct even when implementation utilities are shared:

- signature envelope and cryptographic verification;
- publisher trust and revocation;
- unsigned developer approval;
- launch profile/environment construction;
- conformance orchestration;
- conformance evidence and invalidation;
- installation review/approval;
- immutable installation publication;
- installed Plug registry;
- disabled binding representation.

Do not put trust, installation or conformance authority in Core, Socket, provider output, package metadata, capability manifests, Trail, replay or result Anchors.

Do not alter the three canonical attempted outcomes. M3 is pre-operational and should not need to create a normal provider operation outcome or Result Anchor.

## Forbidden changes

Do not implement:

- M4 File Tools capability contracts or provider package;
- operational enablement;
- active Plug invocation;
- external Anchor delivery or durable event admission;
- AppContainer or claims of hostile-code isolation unless a separately reviewed contract is first required and approved;
- production credential storage or delivery;
- OAuth, network providers, listeners or remote transport;
- marketplace, registry, updater or automatic download;
- interpreter-backed provider launch;
- shell execution;
- automatic retry;
- changes to Tether syntax or OCaml Core semantics;
- movement of `v0.2.0`, release tags or release objects;
- silent migration of legacy 0.2 configuration.

## Major stop rule

Sol must continue through ordinary engineering problems and use its own reasoning to resolve them conservatively.

A stop is permitted only for a major issue that cannot be resolved within the frozen contracts after at least two materially different, evidence-based attempts.

Major stop conditions are limited to:

- a direct contradiction between accepted J18 contracts that changes a security or lifecycle outcome and cannot be reconciled without architecture authority;
- inability to implement or honestly test a required cryptographic boundary using a mature Rust 1.89-compatible library;
- inability to enforce the exact Windows launch/no-shell/clean-environment boundary without weakening the accepted security promise;
- evidence that the accepted supervised profile would be falsely represented as isolation;
- a discovered secret-leak path that cannot be safely closed within M3;
- an unavoidable regression in released 0.2 or accepted M1-M2 semantics after two materially different repairs;
- a required new external service, administrator privilege, kernel component, network dependency or production credential;
- a need to enter M4, alter frozen Tether/Core semantics, move release refs, force-push or rewrite published history.

The following are explicitly not blockers:

- choosing module names or file counts;
- designing the named v1 schemas;
- adding golden fixtures;
- selecting mature authorised dependencies;
- ordinary compiler errors;
- Windows API friction;
- moving tests to sensible units or integration files;
- creating deterministic fixture executables;
- a coherent larger diff;
- test runtime;
- an unfinished later packet;
- a dirty worktree while actively working;
- uncertainty that can be resolved by inspecting accepted documents and code.

A genuine `BLOCKED` report must include:

- the exact frozen-contract conflict or failing security boundary;
- exact command and smallest evidence;
- two materially different attempts;
- observed external effects;
- safe rollback/current clean checkpoint;
- one concrete architecture decision that only Lucy or Matthew can make.

Do not report `BLOCKED` merely to ask permission for an ordinary engineering choice.

## Commit and continuation discipline

Use bounded normal commits. A suitable map is:

- P7 trust/signature;
- P8 supervised launch profile;
- P9 conformance gate;
- P10 installation approval and installed-disabled registry;
- narrow compatibility or evidence commits when genuinely required;
- final worker-note ledger.

Commit names and count may vary. Continue automatically after each successful checkpoint.

Do not amend or rebase commits once pushed. Push normal checkpoints to:

`codex/j19-m3-trust-launch-conformance`

## Required full verification

At final handoff run, at minimum:

```powershell
just tools
just fmt
just check
just test-rust
just verify
```

Run all focused P7-P10 tests, locked debug and release builds, all Rust targets/features, the established OCaml toolchain gate, `dune build`, `dune runtest`, the complete `verify-0.2.ps1` matrix, packet checker, whitespace check and process-cleanup proof.

Add or extend a thin `just test-m3` or `just verify-m3` recipe only when it directly exposes the real underlying commands and remains readable.

Do not suppress failing tests or warnings newly introduced by M3. Distinguish pre-existing warnings in the worker note.

## Worker note

Create and maintain:

`docs/worker-notes/2026-08-01-j19-m3-trust-launch-conformance.md`

Record:

- exact control commit and branch starting point;
- commit map for P7-P10;
- every schema and golden fixture;
- direct dependencies added and why;
- exact cryptographic and Windows API decisions;
- clean-environment allow-list and exclusions;
- supervised-profile limitations;
- conformance categories, suite identity and invalidation rules;
- installation and disabled-binding transaction;
- focused and full verification commands and counts;
- externally visible effects;
- remaining risks and deferred isolation/credentials;
- final branch SHA;
- confirmation that no M4 behaviour was added.

## Completion report

Return only:

`M3 COMPLETE - INSTALLED DISABLED PLUG`

Include no progress essay in the completion message. The worker note is the evidence ledger.

Do not begin M4.
