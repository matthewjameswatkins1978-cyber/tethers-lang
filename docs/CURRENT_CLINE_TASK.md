# Current Implementation Task

Control contract: `1`
Task: `J18G - Security, Trust, Credentials and Sandbox Threat Model v1`
Owner: `Luna`
Status: `IN_PROGRESS`
Task colour: `Amber`
Route: `Luna on OpenCode, security architecture and consistency audit`
Base branch: `main`
Base commit: `96549b1a18bc63d3e6c89ee80cf63a88361b13e2`
Branch: `luna/j18g-security-sandbox-threat-model-v1`
Worker note: `docs/worker-notes/2026-08-01-j18g-security-sandbox-threat-model-v1.md`

## Objective

Define the canonical J18 security contract for threat actors, trust boundaries,
signatures, publisher trust, revocation, installation approval, isolation,
Windows process containment, resources, credentials, diagnostics, integrity,
conformance, quarantine, and security-violation handling. Documentation only.

## Relevant background and existing behaviour

J18B through J18F are accepted architecture contracts. Released Tethers 0.2.0
is peeled by `v0.2.0` to `b5546411661dcbcb53e1cf2538eaec594c6f76f2`. The current
host has supervised process ownership, bounded stdio/stderr, strict manifests,
digest conflicts, approvals, replay, and redacted reasons, but does not provide
malicious-provider filesystem, network, credential, token, AppContainer, or DLL
containment.

## Required behaviour

1. Define threat actors, security domains, non-goals and ordered trust gates.
2. Define the narrow Ed25519 signature, key identity, trust-store, revocation,
   and unsigned developer-mode contract.
3. Define supervised versus isolated profiles and honest Windows containment.
4. Define environment, filesystem, network, credentials, diagnostics, resource,
   conformance, quarantine, and refusal boundaries.
5. Update J18F/J18D authority, current-state documents, decision log, SECURITY.md,
   and worker note.

## Relevant components

- `docs/architecture/TETHERS_SECURITY_TRUST_CREDENTIALS_SANDBOX_V1.md`
- `docs/architecture/TETHERS_LIFECYCLE_OUTCOMES_EVENTS_CONFORMANCE_V1.md`
- `docs/architecture/TETHERPLUG_PACKAGE_V1.md`
- `docs/CAPABILITY_BRIDGE.md`
- `tethers-0.1/host-rust/src/child_process.rs`
- `tethers-0.1/host-rust/src/stdio_provider.rs`
- `tethers-0.1/host-rust/src/manifest.rs`
- `tethers-0.1/host-rust/src/trusted_store.rs`
- `tethers-0.1/host-rust/src/runtime_config.rs`
- `tethers-0.1/host-rust/src/replay_windows.rs`

## Frozen decisions and invariants

- Host policy and OS containment remain separate.
- Providers remain untrusted after signing and conformance.
- Ed25519 is the only J18G signature algorithm.
- Publisher trust is host-owned; trust-on-first-use is forbidden.
- Job Objects are supervision, not complete sandboxing.
- Credentials remain host-owned and absent from durable evidence.
- Filesystem and network access begin at deny.
- No implementation, schema, key, signature, credential, or Tether change.

## Acceptance criteria

1. Exactly ten authorised documentation paths change.
2. J18F is marked accepted and J18D points to J18G for signature trust.
3. Threat actors, non-goals, trust, signatures, revocation and developer mode
   are precise.
4. Supervised and isolated profiles are distinct and Job Objects are not
   overstated.
5. Credentials and environment are host-owned and minimised.
6. Filesystem/network default deny and conformance strength are honest.
7. SECURITY.md matches the architecture.
8. No implementation or security artifact changes and all checks pass.

## Required verification

- `git diff --check`, exact changed paths and clean-worktree checks
- staged diff checks and task-packet checker
- required trust, sandbox, credential, honesty and forbidden-claim searches
- security-artifact search and published main/tag verification

## Forbidden changes

Do not modify Rust, OCaml, Cargo, Dune, opam, scripts, tests, fixtures,
manifests, runtime configuration, schemas, keys, signatures, packages, ZIPs,
providers, transcripts, credentials, trust stores, AppContainer profiles,
sandbox configuration, replay/event storage, Tether specification, Constitution,
release notes, tags, or GitHub Releases.

## Stop conditions

Stop on base, branch, ref, worktree, ownership, authorised-path or boundary
mismatch; false implementation claims; failed checks; or a need to redesign a
frozen semantic. After two materially similar failed attempts, stop with exact
evidence and one smallest unresolved question.

## Expected pre-existing changes

None on the new J18G branch before this task.

## Commit and publication boundary

Create exactly one commit: `docs: define plug security and sandbox model`.
Push only `luna/j18g-security-sandbox-threat-model-v1`. Do not push main, tags,
releases, or begin J18H.
