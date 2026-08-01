# J18G Worker Note

## Task

J18G - Security, Trust, Credentials and Sandbox Threat Model v1.

## Changes

Added the candidate security architecture and public security boundary, marked
J18F accepted, clarified J18D signature authority, added the decision entry,
aligned current-state documents, and created this evidence note. No
implementation is authorised.

## Decisions and assumptions

Host policy and provider containment are separate. Providers remain untrusted
after signing and conformance. Ed25519 signatures bind semantic package digests,
but publisher trust remains host-owned. Supervised and isolated profiles are
distinct; Job Objects are not a complete sandbox. Credentials and environments
are host-owned and minimised; durable evidence never contains credential values.

## Existing process supervision inspected

Inspected `child_process.rs`, `stdio_provider.rs`, and `host_execution.rs`.
Current truth is exact command/args, piped stdin/stdout, bounded protocol lines,
separately captured bounded stderr, retained child ownership, Windows Job Object
membership, kill-on-job-close process-tree handling, timeout-aware reads, and
bounded graceful shutdown. These are supervision boundaries, not hostile-code
containment.

## Existing trust and manifest boundaries inspected

Inspected `manifest.rs`, `trusted_store.rs`, `runtime_config.rs`,
`docs/CAPABILITY_BRIDGE.md`, and J18D. Strict duplicate/unknown-field parsing,
canonical digest verification, trusted manifest admission, identity/digest
conflict rejection, exact provider bindings, and pinned paths are present. No
package signature or publisher trust implementation exists.

## Existing credential behaviour inspected

Inspected the named host/configuration sources. No credential vault integration
exists in the current 0.2 host; the J18G credential store and session delivery
rules are architecture-only. Current environment construction and process
launch remain implementation facts to be constrained by a future task, not
claims of a completed sandbox.

## Primary security sources inspected

Accessed 2026-08-01 from official sources:

- Microsoft Learn, “Job Objects - Win32 apps”.
- Microsoft Learn, “AppContainer isolation - Win32 apps”.
- Microsoft Learn, “Launch an AppContainer - Win32 apps”.
- Microsoft Learn, “Restricted Tokens - Win32 apps”.
- Microsoft Learn, “Credentials Management - Win32 apps”.
- Microsoft Learn, “Kinds of Credentials - Win32 apps”.
- Microsoft Learn, “CredWriteW function (wincred.h) - Win32 apps”.
- Microsoft Learn, “CryptProtectData function (dpapi.h) - Win32 apps”.
- RFC Editor, “RFC 8032: Edwards-Curve Digital Signature Algorithm (EdDSA)”.
- RFC Editor, “RFC 8410: Algorithm Identifiers for Ed25519, Ed448, X25519, and
  X448 for Use in the Internet X.509 Public Key Infrastructure”.

The initially guessed Microsoft Credential Manager and DPAPI URLs returned 404;
no evidence was fabricated from them. Lucy subsequently supplied the verified
replacement links above, and Luna inspected them before this correction.

## Tool bootstrap

- `rg` 15.2.0
- `fd` 10.4.2
- `jq` 1.8.2
- `gh` 2.97.0
- `yq` 4.53.3

Existing WinGet locations were used process-locally. Nothing was installed,
upgraded, replaced, or permanently configured.

## Evidence

Base: `96549b1a18bc63d3e6c89ee80cf63a88361b13e2`. Release tag peels to
`b5546411661dcbcb53e1cf2538eaec594c6f76f2`. No implementation, schema, key,
signature, credential, provider, package, test, fixture, or runtime file changed.

## Discoveries

The existing host has strong lifecycle supervision and manifest integrity but no
proof of malicious-provider containment, package signatures, publisher trust, or
credential vault delivery. J18G therefore keeps those protections explicitly
future architecture rather than overstating the current 0.2 implementation.

Lucy’s security review accepted the ten-path shape and broader architecture but
found a contradiction between supervised-mode limits and the credential-delivery
wording, plus incomplete Credential Manager/DPAPI source inspection. The
correction distinguishes credential storage, deliberate secret delivery and
OS-enforced isolation. It adds exact 64-octet Ed25519 signature decoding. No
implementation, schema, key, signature, credential or sandbox artifact changed.

## Remaining risks

J18H must paper-test representative integrations against containment and refusal
boundaries. J18I must sequence implementation only after security and paper
validation acceptance. Exact Windows AppContainer, token, credential and network
enforcement mechanics remain unimplemented and require separate review.

## Next action

Lucy reviews J18G. Do not begin J18H or implementation before acceptance.

## References

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
