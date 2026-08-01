# Current Implementation Task

Control contract: `1`
Task: `J19-M2 - Autonomous Package Candidate Programme`
Owner: `Codex Terra High`
Status: `IN_PROGRESS`
Task colour: `Red`
Route: `Codex, autonomous package inspection, quarantine and candidate-registry implementation`
Base branch: `main`
Base commit: `337ab11c9cd4059402ef48d5949365c9517867a7`
Accepted implementation baseline: `43179db362efbfed4a0079249ef7a940cde7054e`
Frozen architecture base: `a5fd63593a9d9acd397030ecd2e27b4f318c87fd`
Branch: `codex/j19-m2-package-candidate`
Worker note: `docs/worker-notes/2026-08-01-j19-m2-package-candidate.md`

## Control-plane starting rule

Start a fresh Codex Terra High session. Fetch `origin/main`, create or reset only
the new clean milestone branch from the commit containing this control packet,
and record that exact control commit in the worker note.

The accepted M1 implementation baseline is
`43179db362efbfed4a0079249ef7a940cde7054e`. The control-only commit containing
this packet authorises M2 but changes no runtime semantics.

Use normal commits and normal pushes only. Do not amend, rebase published work,
force-push, move `main`, move tags, or create a release.

## Tooling preflight

Before writing a helper or searching manually, inspect the tools already installed:

- `rg` for repository and symbol search;
- `fd` for file discovery;
- `jq` and `yq` for structured test data inspection;
- `gh` for GitHub inspection;
- `pwsh` for the existing Windows verification scripts;
- `just` only when an existing or clearly useful repeatable project command
  already justifies it.

Use the existing tool before inventing a wrapper. Do not add a `justfile`, source
generator, second build layer, or broad helper framework merely for convenience.
Small test-fixture builders are allowed when they are the safest way to create
adversarial archives and remain test-owned.

## Objective

Complete Milestone 2 from the accepted J18I roadmap:

1. `P4-PACKAGE-INSPECT`;
2. `P5-QUARANTINE-PATHS`;
3. `P6-INSTALL-CANDIDATE-REGISTRY`.

The result is a host-owned, non-executing path that can inspect one
`.tetherplug`, strictly validate its package and manifest evidence, select the
accepted Windows x86_64 payload, compute exact raw and semantic identities,
extract accepted bytes only into a host-owned quarantine root, and create one
immutable installation-candidate record.

M2 ends with a candidate that is:

- uninstalled;
- disabled;
- unapproved;
- untrusted;
- non-operational;
- absent from active provider and capability bindings;
- incapable of provider launch, Socket establishment, invocation or Anchor
  admission.

## Relevant background and existing behaviour

The accepted M1 Socket/application seam and released 0.2 configuration path are
already present. `manifest.rs` supplies strict duplicate-key parsing, RFC 8785
canonicalisation and verified capability-manifest evidence; `replay_windows.rs`
contains the existing Windows reparse-safe persistence patterns. Neither legacy
manifests nor runtime configuration are Plug packages, candidates, or installed
records.

## Required behaviour

1. Inspect one untrusted `.tetherplug` archive without extraction, execution,
   launch, trust, installation, binding, or runtime mutation.
2. Extract only an accepted inspection result into a new host-owned quarantine
   directory using a staged, no-overwrite publication boundary.
3. Persist and reload immutable installation-candidate records that remain
   uninstalled, disabled, unapproved, untrusted, and non-operational.

Return only at:

`M2 COMPLETE - PACKAGE CANDIDATE`

or on a genuine stop condition defined below.

## Governing contracts

Implement against the accepted documents already in the repository:

- `docs/architecture/TETHERPLUG_PACKAGE_V1.md`;
- `docs/architecture/TETHERS_CAPABILITIES_EFFECTS_SCOPES_V1.md`;
- `docs/architecture/TETHERS_SECURITY_TRUST_CREDENTIALS_SANDBOX_V1.md`;
- `docs/architecture/TETHERS_LIFECYCLE_OUTCOMES_EVENTS_CONFORMANCE_V1.md`;
- `docs/architecture/TETHERS_J18_IMPLEMENTATION_ROADMAP.md`;
- the accepted M1 Socket/application seam at the implementation baseline.

Where J18D names a conceptual field but does not freeze its exact Rust type or
nested JSON spelling, this packet authorises Terra to choose and freeze the
smallest direct v1 machine representation consistent with every named field and
refusal rule. Record those choices in fixtures and the worker note. Do not add
fields that grant authority, policy, trust, credentials, installation,
enablement or runtime configuration.

The current strict capability-manifest parser may be reused behind a clearly
named verifier seam. Existing loose 0.2 manifests and runtime configuration must
not be relabelled as packages, installed Plugs or candidate records. Reuse
parsing, duplicate-key, JCS, digest and semantic-validation code where sound;
do not silently change legacy manifest semantics.

## Autonomy rule

Codex owns ordinary engineering decisions needed to complete the milestone,
including:

- module names and source-file boundaries;
- public versus crate-private visibility;
- exact value types and error enums;
- archive-reader integration;
- test-fixture layout;
- conservative package resource limits;
- quarantine directory layout;
- immutable candidate-record representation;
- atomic-write and crash-recovery mechanics;
- focused test versus integration-test placement;
- commit count and intermediate local layout;
- compiler-guided untangling;
- narrow updates to existing scripts whose assumptions move without changing
  the proof they enforce.

These are not blockers. An unfinished packet, dirty worktree during work, a
larger-than-estimated but coherent diff, or later M2 packets remaining is not a
BLOCKED condition.

A minimal mature Rust ZIP/archive dependency and its necessary transitive
compression dependencies are authorised if required. It must support Rust
1.89, be used only for archive parsing/decompression, be pinned by `Cargo.lock`,
and be documented in the worker note with version, purpose and why a custom ZIP
parser was rejected. Existing JCS and SHA-256 dependencies must be reused. No
new cryptography, signature, network, shell or executable-helper dependency is
authorised. A justified archive dependency and resulting `Cargo.lock` change do
not require another approval.

## P4 required result: package inspection

P4 creates a pure inspection boundary. Inspection reads untrusted bytes and
returns a typed inspection report or typed refusal. It does not extract, launch,
install, approve, trust, bind, enable, mutate runtime configuration or invoke a
provider.

### Archive profile

Accept only the J18D v1 ZIP-compatible profile:

- `.tetherplug` source treated as untrusted bytes;
- ordinary files using stored or deflated compression only;
- exactly one root `plug.json`;
- required `provider/` payload and at least one `manifests/` entry;
- optional `tests/`, `docs/`, `assets/`, `licenses/` and `signatures/` areas;
- no files outside the canonical roots;
- no unnecessary empty directory entries;
- no encryption, multi-disk archive, self-extracting form, Zip64, unsupported
  compression, nested `.tetherplug`, symbolic link, hard link, device, FIFO,
  junction/reparse-point, alternate-data-stream or executable metadata feature;
- bounded archive size, entry count, per-entry size, total uncompressed size,
  compression ratio, path length, JSON size and manifest count;
- deterministic fail-closed refusal when a limit is exceeded.

Terra chooses conservative explicit limit constants and records them. Limits
must comfortably permit the planned credential-free File Tools and PDF Tools
reference packages without becoming an archive-bomb invitation.

### Package paths

Every archive path must:

- be relative and use `/`;
- contain lowercase ASCII segments matching
  `[a-z0-9][a-z0-9._-]*`;
- reject spaces, empty segments, `.`, `..`, leading slash, backslash, drive
  letters, colon, NUL/control characters and non-ASCII path bytes;
- reject trailing dots or spaces and Windows reserved device names, including
  reserved names with extensions;
- reject duplicate raw paths, case-insensitive collisions, normalized
  collisions, file/directory-prefix collisions and any path that can escape the
  eventual quarantine root;
- reject archive entries whose metadata claims link, reparse, device or other
  non-ordinary-file semantics.

Do not trust the archive library's convenience extraction path as the security
boundary. Validate the original archive name and the host destination
independently.

### Strict `plug.json`

Implement strict UTF-8 JSON without BOM, duplicate keys, unknown fields,
invalid I-JSON values or trailing data. Use RFC 8785 JCS for canonical bytes and
semantic digest input.

The implemented v1 machine model must directly represent the J18D fields:

- package format version `"1"`;
- lowercase dotted `package_id`;
- strict `MAJOR.MINOR.PATCH` `package_version`;
- display name, description, publisher presentation string and licence;
- Socket major 1;
- protocol binding MCP `2025-11-25` over local stdio;
- platform Windows x86_64;
- exactly one provider declaration with provider identity/version, launch,
  package-relative working directory and capability operation namespace;
- a non-empty canonical capability list containing capability identity,
  manifest path/digest and provider operation name;
- a complete canonical payload index containing path, lowercase
  `sha256:<hex>`, exact size and accepted role.

Capability entries sort by name then version. Payload entries sort by path.
Duplicate capability identities and duplicate provider operation names fail.
Launch is an ordered, package-relative declaration with no shell command string,
interpolation, `cmd /c`, PowerShell `-Command`, PATH lookup, install command or
unbounded user fragment. Interpreter-backed operational launch remains deferred;
M2 may inspect and report such a declaration only if the package contract marks
it unsupported for the first envelope. Nothing is launched.

### Payload and manifest validation

The payload index is complete for every non-signature payload. `plug.json` is
not self-indexed. Signature entries remain evidence-only and excluded from
semantic package identity.

For every indexed payload:

- the archive entry exists exactly once;
- no unindexed non-signature payload exists;
- path and role are compatible with the canonical root;
- declared size equals actual decompressed size;
- declared SHA-256 equals the exact payload bytes;
- the provider launch path resolves to one indexed provider payload;
- every capability manifest path resolves beneath `manifests/` with role
  `capability_manifest`;
- package capability identity, operation name and manifest digest agree with
  the verified authoritative manifest evidence;
- manifest schemas, effects, scopes and behaviour cannot be overridden by
  `plug.json` presentation fields.

Descriptions, annotations, publisher strings, archive metadata and signature
files are untrusted observations. They never alter trusted manifest semantics.

### Identity and inspection report

Keep these identities distinct:

- raw archive identity and SHA-256;
- package lineage `package_id`;
- human release `package_id + package_version`;
- exact package identity adding semantic package digest;
- provider identity;
- capability `name + version`;
- capability-manifest digest;
- later host-generated candidate identity.

Compute semantic package digest from exact RFC 8785 JCS bytes of the fully
validated `plug.json` only after array order, archive/index agreement, sizes,
payload bytes and payload digests have passed. The digest is not stored inside
`plug.json`.

The report must preserve enough bounded evidence to explain acceptance or
refusal without copying arbitrary raw provider stderr, executing content or
creating authority. Same package ID and version with different semantic digests
must conflict and fail closed or remain separately quarantined for explicit
later review, never silently merge.

P4 ends with inspection only. Commit P4 separately after its focused and
regression evidence passes. Continue directly to P5 without returning.

## P5 required result: safe quarantine extraction

P5 extracts only a P4-accepted inspection result and only after an explicit
library/API continuation call. It does not expose a public install CLI.

### Quarantine rules

- Destination is a new unique child of a host-supplied, host-owned quarantine
  root, never the source archive directory, Downloads, repository checkout,
  current working directory or final installation root.
- Create the candidate in a same-volume staging directory, then publish it by
  one atomic rename only after every entry and record passes.
- Revalidate every destination path independently from the archive-reader path.
- Reject pre-existing target files/directories, links, junctions, mount points,
  reparse points, case collisions and prefix conflicts.
- Use create-new/no-overwrite file creation and do not follow links.
- Verify bytes, sizes and SHA-256 again while or immediately after writing.
- Recheck quarantine-root and parent integrity before publication.
- Apply restrictive host-owned permissions where the current host can prove
  them; mark accepted payload immutable/read-only for M2 purposes.
- Keep package payload, signature evidence, inspection report and future mutable
  provider state conceptually and physically separate.
- Clean incomplete staging safely on ordinary failure. Preserve bounded refusal
  evidence without leaving executable-looking partial candidates.
- Never execute from the archive, source path, staging path or quarantine.
- Never call Socket, initialize MCP, run conformance, import credentials or
  create active bindings.

Use Windows APIs where required to identify and refuse reparse points and path
tricks. Existing `replay_windows.rs` patterns may be reused, but replay state and
candidate state remain separate.

Real Windows adversarial tests must prove no write outside the quarantine root,
including traversal, absolute/drive paths, alternate streams, case collisions,
pre-existing targets, link/junction/reparse destinations and file/directory
prefix collisions. Include a harmless provider-marker fixture and prove
inspection/extraction never creates the marker process effect.

P5 ends with a safely published quarantine directory and typed evidence, not an
installed Plug. Commit P5 separately after its focused and regression evidence
passes. Continue directly to P6 without returning.

## P6 required result: installation-candidate registry

P6 introduces a dedicated candidate/quarantine registry only. It is not the M3
installed-Plug registry.

### Schema-first rule

Before durable registry code, freeze one explicit candidate-record schema in
Rust types, golden fixtures and the worker note. This schema decision is part of
P6 and does not require a separate Lucy round trip.

The immutable record must include at least:

- host-generated candidate ID;
- fixed state identifying a quarantined installation candidate;
- package ID and version;
- semantic package digest;
- raw archive digest and source size;
- exact quarantine location beneath the configured root;
- selected Windows x86_64 payload identity and payload digests;
- provider claim identity/version and launch evidence;
- capability identities, operation names and verified manifest digests;
- inspection-report identity/version;
- unverified signature-presence evidence without trust claims;
- creation time and schema version;
- enough information to detect same-release semantic conflicts and corrupted
  candidate records on reload.

The record must not contain:

- installed identity;
- active binding identity;
- publisher trust;
- signature validity or key trust;
- installation approval;
- conformance approval;
- enablement;
- policy decisions or installation grants;
- credentials or credential references;
- generated runtime configuration;
- provider session state;
- Trail, replay or Anchor authority.

### Durable behaviour

- Candidate IDs are host-generated and never package-selected.
- Candidate records are immutable and create-only.
- Candidate registry and quarantine payload roots are separate.
- Use crash-aware create, flush and atomic publication on the same volume.
- Never overwrite or merge an existing candidate.
- Reload strictly validates schema, duplicate identities, exact root
  confinement and record integrity.
- Torn temporary files, malformed records and missing/mutated payload evidence
  fail closed and cannot become available candidates.
- Same package ID/version with a different semantic digest is an explicit
  conflict.
- A repeated exact package may be reported as already represented or receive a
  distinct candidate ID according to one recorded deterministic rule, but it
  must never silently replace existing evidence.
- Candidate creation produces no active provider/capability availability and no
  mutation of legacy 0.2 configuration or stores.

Tests must prove process restart/load, exact immutable replay of records,
conflict refusal, torn-write handling, missing/corrupted payload refusal,
registry/quarantine separation and total absence of provider launch, Socket
traffic, runtime binding, policy availability and event admission.

Commit P6 separately after all M2 evidence passes.

## Required source boundaries

Create the smallest coherent M2 implementation. Reasonable concepts include:

- package/archive inspection;
- strict package value types and report/error types;
- safe package path validation;
- quarantine extraction;
- candidate record and registry.

These names and file counts are not requirements. Keep archive and quarantine
code out of Core and out of the MCP Socket module. M1 Socket semantics should
not need modification except for a genuine compile/import correction. The
legacy 0.2 path must remain callable and unchanged.

No public Plug lifecycle CLI is authorised in M2. Library APIs and test-owned
fixtures are sufficient. Do not add broad public module visibility merely to
make tests convenient.

## Relevant components

- `tethers-0.1/host-rust/src/manifest.rs` for the existing strict capability
  manifest verifier and JCS/SHA-256 implementation.
- `tethers-0.1/host-rust/src/replay_windows.rs` for target-specific safe
  publication and reparse-point patterns; replay state itself remains separate.
- `tethers-0.1/host-rust/src/lib.rs` for the smallest M2 library surface.
- `tethers-0.1/host-rust/Cargo.toml` and `Cargo.lock` only for the authorised
  archive parser and its transitive compression dependencies.

## Acceptance criteria

1. P4 accepts valid stored and deflated packages, produces separate raw and
   semantic identities, and refuses every specified malformed archive, path,
   JSON, payload, manifest, compatibility, and resource-limit branch.
2. P5 extracts only accepted bytes under a fresh quarantine child, verifies
   bytes a second time, refuses destination tricks, and proves no launch or
   write outside the configured root.
3. P6 creates and reloads strict immutable candidate records, refuses conflict,
   corruption and mutation, and proves that candidates create no runtime
   binding, availability, policy, event, or provider effect.

## Required evidence

Add deterministic positive and negative evidence for at least:

### P4 inspection

- valid stored and deflated package variants;
- raw archive digest changes while semantic digest remains stable across ZIP
  ordering, timestamps and compression representation;
- duplicate `plug.json`, duplicate archive entries and unknown roots;
- missing provider/manifests, unindexed payload and missing indexed payload;
- size and SHA mismatch;
- strict JSON duplicate keys, unknown fields, BOM, trailing data and invalid
  values;
- invalid package IDs/versions and incompatible Socket/protocol/platform;
- non-canonical capability/payload ordering;
- duplicate capability identity and operation name;
- manifest identity/schema/effect/scope/digest mismatch;
- traversal, absolute, drive, backslash, colon/ADS, Unicode path, reserved
  device, trailing dot/space and prefix/case collisions;
- encrypted, Zip64, multi-disk, unsupported compression, links, devices,
  reparse metadata, nested package and resource-limit/archive-bomb refusal;
- signature presence remains unverified evidence and grants nothing;
- inspection produces no extraction, process or host-binding effect.

### P5 quarantine

- real Windows safe extraction of an accepted package;
- second byte/digest verification;
- no overwrite and no escape;
- malicious destination link/junction/reparse refusal;
- case and prefix collision refusal;
- incomplete staging cleanup;
- published payload read-only/immutable evidence;
- no launch from source, archive, staging or quarantine;
- no provider process remains beneath the test checkout.

### P6 candidate registry

- schema golden and strict unknown/duplicate-field refusal;
- create, flush, atomic publish, reload and lookup;
- immutable no-overwrite behaviour;
- same-release semantic conflict;
- torn temp and malformed/corrupted record refusal;
- missing or mutated quarantine payload refusal;
- candidate/provider/package/capability identity distinctions;
- candidate remains uninstalled, disabled, unapproved, untrusted and absent
  from every runtime availability/binding path.

## Required verification

Run the full regression matrix named below, including locked Rust checks and
builds, existing M1 Socket/catalogue and MCP tests, host PowerShell suites,
the J14C proof, OCaml build/tests through the established switch, fixture and
runner validation, the packet checker, whitespace check, lockfile hashes, and
process cleanup proof. Do not suppress a failing proof.

### Full regression

Before reporting M2, run at least:

- Rust 1.89 formatting check;
- `cargo check --all-targets --all-features --locked`;
- `cargo test --all-targets --all-features --locked`;
- locked debug build;
- locked release build;
- all accepted M1 Socket/catalogue tests;
- all existing MCP transcripts;
- all existing host PowerShell suites and consolidated matrix;
- J14C real file move and zero-replay-move proof;
- OCaml `dune build` and `dune runtest` through the established switch;
- engine fixtures, JSON/JSONL fixture validation, demo and runner contract;
- task-packet checker;
- `git diff --check`;
- before/after `Cargo.lock` hashes and dependency explanation;
- process cleanup proving no child remains beneath the checkout.

Do not weaken, skip, rename away or suppress a failing proof merely to finish
M2. Existing tracked non-fatal warnings may remain visible; new warnings caused
by M2 should be fixed unless there is a documented compatibility reason.

## Frozen decisions and invariants

- Tethers Core remains deterministic and application-agnostic.
- Tether language syntax and semantics remain `0.1`.
- The accepted M1 Socket/application seam remains intact.
- Socket, MCP protocol binding and byte transport remain distinct.
- The host owns inspection, identity, quarantine and candidate state.
- Providers and packages remain untrusted observations.
- Package possession, valid structure and semantic digest grant no authority.
- Signature presence grants no authority; signature verification and publisher
  trust are M3.
- Candidate is not installed, approved, enabled, bound or operational.
- No execution occurs during inspection, extraction or candidate creation.
- No launch occurs from archive, Downloads, source, staging or quarantine.
- No active provider/capability binding exists before M3 installation approval
  and later explicit enablement.
- No policy permission, credential, conformance result, provider health or
  runtime availability is created in M2.
- Attempted-operation outcomes, replay, Result Anchors, event admission and
  Trail ordering remain unchanged and unused by package inspection.
- No automatic retry exists.
- Released `v0.2.0`, tags, releases and legacy user configuration remain
  unchanged.

## Explicit exclusions

Do not implement:

- Ed25519 verification or signature-envelope trust decisions;
- publisher trust, key rotation or revocation;
- unsigned developer-mode approval;
- conformance launch or conformance evidence approval;
- installation approval or installed-Plug registry;
- present-disabled installed bindings;
- provider launch profiles, clean environment or AppContainer;
- credentials or Credential Manager;
- operational launch, Socket establishment or invocation from a candidate;
- enable/disable/remove lifecycle;
- File Tools or PDF Tools packaged provider payloads;
- public inspect/install/enable CLI;
- durable external Anchor admission;
- Jobs, Streams or Human Tasks;
- network providers, listeners, update channels, marketplace or registry
  downloads;
- Tether syntax or Core semantic changes.

## Forbidden changes

Do not add package trust, signatures, approval, installed state, bindings,
provider launch, Socket use, credentials, conformance, lifecycle CLI, network
behaviour, runtime configuration changes, legacy 0.2 reinterpretation, M3
work, or Tether/Core semantic changes.

## Genuine stop conditions

Continue and record the decision when the issue concerns ordinary module layout,
resource-limit values, test-fixture construction, archive-crate API usage,
visibility, error wording, atomic-file representation, candidate ID generation,
commit structure or compiler-guided refactoring.

Begin `BLOCKED` only when one of these remains after at least two materially
different evidence-based attempts:

- the frozen J18D/J18E/J18G contract must change;
- safe archive inspection cannot reject a required forbidden ZIP feature with
  the selected library and no mature Rust alternative can do so;
- Windows destination confinement or reparse refusal cannot be proved;
- semantic package identity cannot be computed without conflating raw archive,
  package, manifest or candidate identity;
- a persistent released-0.2 or M1 regression cannot be repaired without
  semantic change;
- a dependency beyond the authorised archive/decompression purpose is required;
- inspection/extraction would require execution, shelling out or entering M3;
- durable candidate state would need trust, installation approval, bindings,
  credentials, conformance or provider launch;
- public CLI or machine contracts outside the named M2 schemas must change;
- Git history would require force, published rebase or release-ref mutation.

## Stop conditions

Only the genuine stop conditions above qualify, and only after two materially
different evidence-based attempts. Ordinary implementation decisions, coherent
scope growth inside M2, or later packets remaining unfinished are not stops.

A BLOCKED report must include the exact failing command, two attempted
approaches, smallest relevant evidence, external effects, safe rollback and one
concrete decision that cannot reasonably be made by the implementation owner.

## Expected starting state

- `main` equals accepted M1 SHA
  `43179db362efbfed4a0079249ef7a940cde7054e` before this control-only commit;
- the new M2 branch starts from the control commit containing this packet;
- worktree is clean;
- no M2 implementation exists;
- Milestone 3 is not authorised.

## Expected pre-existing changes

None. The M2 branch starts clean at the control commit; the current task packet
is the only control-plane content added there.

## Worker note

Create and maintain:

`docs/worker-notes/2026-08-01-j19-m2-package-candidate.md`

Record:

- exact control commit, branch base and toolchain;
- installed-tools preflight and any helper retained;
- archive dependency and lockfile change, if any;
- machine `plug.json` representation choices;
- explicit resource limits;
- P4/P5/P6 commit map and rollback points;
- package/path/digest/manifest identity decisions;
- quarantine Windows path and reparse strategy;
- candidate schema and crash/recovery strategy;
- positive and adversarial fixtures;
- every regression and correction;
- exact final commands and totals;
- proof of no execution, no binding and no M3 work;
- remaining risks.

## Suggested commit map

Use bounded, reviewable commits. A reasonable stack is:

- `feat: inspect tetherplug packages`;
- `feat: extract packages into quarantine`;
- `feat: record immutable installation candidates`;
- optional narrowly scoped fixture or compatibility correction commits.

The wording is not mandatory. P4, P5 and P6 must remain identifiable and
individually reversible.

Push only:

`codex/j19-m2-package-candidate`

Do not push `main`, tags or releases.

## Completion report

Begin exactly:

`M2 COMPLETE - PACKAGE CANDIDATE`

Report:

1. branch, final SHA and control commit;
2. P4/P5/P6 commit map and rollback points;
3. changed paths by packet;
4. final module and store layout;
5. archive dependency and final `Cargo.lock` hash;
6. exact `plug.json` machine representation choices;
7. package/path/resource refusal rules;
8. raw, semantic, manifest and candidate identity evidence;
9. quarantine extraction and Windows reparse evidence;
10. candidate schema, atomicity and restart evidence;
11. no-execution, no-binding and no-M3 proofs;
12. full test commands and exact totals;
13. 0.2 and M1 compatibility confirmation;
14. remaining risks;
15. clean worktree and branch ahead/behind `main`;
16. confirmation that `main`, tags and releases are untouched.

On a genuine stop condition begin exactly:

`BLOCKED`

Stop after the report. Do not begin Milestone 3 until Lucy accepts M2 and installs
a new authoritative packet.
