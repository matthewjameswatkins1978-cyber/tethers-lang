# Tethers x Resolve01 P3a worker note

Task: `TETHERS x RESOLVE01 / P3a - Protocol Provenance Closeout`

Owner: `Codex`

Status: `COMPLETE`

## Objective

Replace the unreachable historical Resolve protocol citation with one durable,
fetchable canonical source for `resolve.tethers-guard/1`, without changing P3
transport or outcome semantics.

## Provenance classification

The previously cited Resolve SHA
`b450305e72815d33357359c6db641d315d6b2975` is classified as orphan provenance.
It was absent from all fetched Resolve branches, tags, reflog, public commit
lookup, commit search, and Resolve pull-request history. It was not fabricated
or recovered from the old `resolve-ai` repository.

## Durable source

- Resolve R0 starting main: `8d42e5b061f86b2b2a2c1949c629654968a550ff`
- Canonical document: `Resolve01/docs/TETHERS_GUARD_PROTOCOL.md`
- Protocol-source content SHA:
  `70f86ff47cbfd38e702cdd8c1433ccd04ecb43bd`
- Resolve provenance document:
  `Resolve01/docs/TETHERS_GUARD_PROTOCOL_PROVENANCE.md`
- Resolve PR: #18
- Merged Resolve main: `b3b985133314f6f349e7520a25c3d737573970ec`
- Tethers P3 implementation head:
  `979057de8d1b6294dbcd87405f7a9fdcb2085b39`
- Tethers P3 merge: `63aa8e21bfa359ad444d8b32b3007b398fa9002e`

The content SHA is the stable pin because it directly contains the canonical
protocol document; the merged Resolve main SHA also contains it and records the
normal PR integration.

## Comparison result

`MATCH` for the accepted P3 implementation and evidence:

- protocol/version, POST admission route, and POST outcome route;
- exact request and response JSON members;
- preparation digest and canonical digest verification;
- closed admission decision/reason and outcome-result vocabularies;
- explicit bridge header and HTTPS/default transport configuration;
- timeout, body, UTF-8, duplicate-member, unknown-field, redirect, and proxy
  behaviour;
- failure mapping, idempotent exact duplicate delivery, and conflict handling.

The canonical document describes accepted behaviour only. No Tethers source,
transport, provider, replay, Core, syntax, policy, or outcome taxonomy changed
in P3a.

## Verification and publication

Resolve docs-only verification: `cargo fmt --all -- --check`, `cargo check
--workspace`, `cargo clippy --workspace --all-targets -- -D warnings`,
`cargo test --workspace`, and `git diff --check` — PASS.

Tethers full authoritative `just verify` passed 10/10 suites on the accepted
P4a branch before this P3a pin update; P3a will rerun the same gate. The direct
repository-wide strict Clippy command remains outside the accepted baseline
because it reports pre-existing diagnostics; the repository static-check and
warning-ratchet gates are authoritative.

## Scope exclusions

No new endpoint, JSON member, protocol version, authentication scheme, retry
policy, scheduler, Resolve Plug, MCP/A2A transport, provider retry,
UNCERTAIN reconciliation, Core/OCaml change, Tether syntax change, or P5 work
was added.
