# Tethers Security, Trust, Credentials and Sandbox v1

Status: J18G candidate, pending Lucy security review
Threat model generation: 1
Implementation: Not authorised

## 1. Security promise

Tethers limits what the trusted host authorises and records. Operating-system
containment limits what an untrusted or compromised provider can physically
reach. These are separate protections: host policy does not sandbox a provider,
a sandbox does not grant policy permission, a valid signature does not prove
safety, and passing conformance does not prove trust.

## 2. Security domains and actors

Core owns deterministic parsing, planning and evaluation only. The host owns
inspection, trust, installation, bindings, policy, approvals, credentials,
launch, containment, outcomes, replay, event admission, conformance and Trail.
The provider remains untrusted after signing, authorship, conformance, install,
or health. Packages and outside data are untrusted input and observation.

Threats include malformed or malicious packages, compromised or revoked keys,
payload mutation, malicious providers/dependencies, filesystem or network escape,
credential theft, secret logging, environment inheritance, shell/argument
injection, DLL side-loading, child escape, resource exhaustion, protocol floods,
oversized stderr, impersonation, stale bindings, conflicting event identities,
over-broad approval, operator error, and hostile parser input.

J18G does not protect against a compromised Windows kernel, administrator/SYSTEM,
compromised host account, malicious firmware/hardware, physical-memory attacks,
all side channels, certified high assurance, hard-real-time certification, or
host binary replacement outside accepted host integrity controls. The first Plug
Kit is not a secure container platform.

## 3. Trust ladder

The ordered gates are: bytes received; archive valid; semantic digest valid;
signature valid; key trusted; publisher mapped by host; contents reviewed;
conformance under accepted containment; installation approved; scopes and policy
configured; credentials attached; exact bindings enabled; launch integrity
verified; bounded invocation authorised. No earlier gate implies a later gate.

## 4. Package signature v1

Only Ed25519 is permitted, using the algorithm described by RFC 8032. There is
no algorithm negotiation, RSA, ECDSA, Ed448, or package-selected cryptography.
Trust-store keys use RFC 8410 DER SubjectPublicKeyInfo. The key identity is
`key_id = sha256:<lowercase hex SHA-256 of exact DER SubjectPublicKeyInfo bytes>`.

The exact UTF-8 signing input is:

```text
tethers.tetherplug.signature.v1
<semantic-package-digest>
```

The final newline is mandatory and the digest is the exact J18D lowercase
`sha256:<hex>` semantic package digest, not raw ZIP bytes. The strict conceptual
envelope contains only `signature_format_version` (`"1"`), `algorithm`
(`"ed25519"`), `key_id`, `semantic_package_digest`, and unpadded base64url
`signature`. Duplicate, unknown, malformed, mismatched, or unknown fields fail
closed. Unpadded base64url decoding must produce exactly 64 octets; any other
length fails before cryptographic verification. Verification uses an accepted
host cryptographic library. Signature
files are beneath `signatures/` with deterministic lowercase names such as
`ed25519-<64-lowercase-hex-key-id>.json`; multiple keys may sign, but duplicates
from one key add no authority. No JSON Schema or fixture is created here.

## 5. Signature meaning and trust store

A valid signature proves possession of a private key and binding to one exact
semantic digest. It does not prove publisher identity, safety, conformance,
permission, installation approval, or current trust. Publisher identity comes
only from a host-owned trust store; package `publisher` is presentation data.

The trust store is separate from packages, payloads, Tether Sets, provider
configuration, credentials and Trail. A conceptual key record has key ID, exact
public key, host-assigned publisher, trust state, optional namespace, times,
approving authority, expiry, and revocation reason. States may be trusted,
disabled or revoked. Changes require explicit authority and safe audit evidence;
trust-on-first-use is forbidden. Packages cannot add keys.

Rotation is explicit. Revocation preserves historical evidence but a revoked key
is not currently trusted. Without a trusted timestamp system, the host does not
invent proof of pre-compromise signing. Install, enable and launch re-evaluate
trust; running-session termination is an explicit recorded host decision.

## 6. Unsigned developer mode

Unsigned packages require explicit developer mode, exact digest approval, visible
unsigned status, no automatic enablement, no publisher-trust or marketplace
claim, no inheritance to later versions, and no silent production conversion.
Developer mode is not default and does not describe arbitrary third-party code as
safe.

## 7. Installation integrity and profiles

Approved payload is immutable host-owned material. Package, payload and provider
executable digests are rechecked immediately before every launch; changed,
missing or additional files fail closed. Execution is from the installation
location, never archive, Downloads or temporary extraction. Provider state is
separate and provider-writable.

The **supervised provider** profile provides exact launch, no shell, process-tree
ownership, bounded stdio/stderr, time/resource limits, shutdown and integrity
checks. It is not hostile-code filesystem, network, credential or process
isolation, and is only for explicitly trusted reference/development providers.

The **isolated provider** profile requires OS-enforced default-deny filesystem,
registry, network, process, credential, device, child-process and handle access.
The first strong Windows profile must use AppContainer or another separately
reviewed equivalent. A restricted token may contribute but alone proves no
complete isolation. Third-party production providers require proven isolation.

The host computes the required profile from trust, class, effects, scope,
credentials, data classification, network and physical/security consequences.
Shared sessions receive the explicit union of requirements; incompatible
privilege sets require separate sessions or refusal.

## 8. Windows process and environment boundary

Every launch requires an exact absolute executable path, no PATH lookup, no
shell/cmd/PowerShell command, ordered arguments, host working directory, digest
recheck, no unexpected handles, explicit stdio, process-tree containment, child
count, memory, CPU/time, protocol, stderr and shutdown limits, and no surviving
unnoticed process. The existing Windows Job Object remains lifecycle ownership;
Job Object membership is not a complete sandbox. Microsoft documents Job Objects
as process-group management, limits, accounting and termination, including
kill-on-job-close, not universal security isolation.

The environment is constructed from scratch, not inherited wholesale. It contains
only minimal approved Windows variables, non-secret configuration, exact bound
credential variables, and explicitly required locale/temp settings. It excludes
ambient API/cloud/Git/SSH/editor/proxy/repository secrets and unrelated PATH
entries. The provider is resolved before environment construction and PATH never
selects an executable or interpreter.

## 9. Dependencies and containment

Prefer a self-contained packaged executable: exact executable pin, deterministic
dependency closure, no launch downloads, no writable search path, no side-loading,
shell startup or profile scripts. Interpreter-backed providers remain deferred.

Installation, mutable state, per-run scratch, conformance scratch, credentials,
Trail/replay and user resources are separate. Payload is read-only; scratch is
bounded/disposable; links, junctions and reparse escapes fail closed; provider
has no broad user-home access. Exact user-file access requires a reviewed broker,
OS grant, or explicitly trusted supervised provider.

Network is default deny. Access requires declared network effects, host policy,
enforceable isolation, exact credentials and matching conformance. File/PDF
reference providers require no network and the first Plug Kit has no listeners.
Unenforceable destinations make a capability unavailable.

## 10. Credentials

Windows Credential Manager is host-owned credential storage, not by itself a
provider-isolation boundary. Microsoft documents long-term generic-credential
storage and that generic credentials can be read and written by user processes.
Profiles are host-generated opaque identities bound to provider, account/service,
capability, rotation and state. Values never enter packages, manifests, runtime
JSON, Tether source, arguments, paths, Trail, replay, event admission,
conformance or Result Anchors. DPAPI-wrapped files require a later explicit
decision.

For local stdio, the host deliberately delivers only credentials required by
exact enabled bindings through a fresh per-process environment. Secret names are
host-owned; packages cannot choose arbitrary names. Values enter immediately
before process creation, never MCP messages or command line. A sanitized
environment prevents deliberate inheritance of unrelated secret variables, but
does not prevent a supervised provider in the same ordinary user security
context from attempting same-user Credential Manager API access, filesystem
access, process inspection or other ambient access not blocked by OS containment.
Exact session delivery means only the required secret is intentionally supplied;
it does not prove that unrelated credentials cannot be discovered. The authorised
provider necessarily receives its own secret.

A credential-bearing production provider requires an isolated profile proven to
prevent access to unrelated credentials, or a separately reviewed host-owned
credential broker. AppContainer is the intended first Windows isolation
candidate, but its exact Credential Manager behaviour is not implemented or
proven by J18G. A restricted token alone is insufficient proof. General
credential-bearing supervised production providers are unavailable.

Supervised development may receive a dedicated test credential only through
explicit risk acceptance, and the host must label that session not credential-
isolated. File Tools and PDF Tools reference providers require no credentials.

Different accounts, privilege, secret sets or scopes require separate sessions or
refusal. One low-risk capability must not make a high-privilege secret ambient.

## 11. Diagnostics, protocol and minimisation

Stdout is protocol-only. Stderr is untrusted, bounded and potentially secret;
it is not outcome evidence or automatically durable. Keep only a bounded local
tail, redact before explicit presentation, and never copy raw stderr to Trail,
replay, conformance, Result Anchors or reports. Preventing persistence is the
primary boundary; value replacement cannot guarantee redaction of encoded,
fragmented or transformed secrets.

Send only exact operation, validated arguments and approved input. Do not send
policy internals, unrelated manifests, trust-store data, credentials or the
complete environment. Provider text is untrusted and cannot alter trust, policy,
scope, credentials or bindings.

## 12. Resources and violations

Each session has finite process, memory, CPU/time, message, queue, stderr,
scratch and enforceable handle limits. Exceeding a security limit terminates or
quarantines the session, records bounded safe evidence, and never authorises
automatic retry.

Payload mutation, signature mismatch, revoked key, undeclared access, process
escape, flooding, secret leakage, schema drift, conflicting event identity or
conformance escape may stop the session, disable binding, quarantine installation
or require review. Preserve bounded evidence and historical Trail; never execute
repair code, retry, or silently restore enablement after restart.

## 13. Approval and conformance security

Security approval is separate from operational Ask and binds package digest,
signature/key/publisher, payload and manifest digests, isolation, effects, scopes,
credentials and conformance. Drift invalidates approval; one version does not
approve another.

Conformance uses the intended isolation profile or stricter, no production
credentials or Tether Sets, disposable data, and deny tests. It covers mutation,
file/network access, child containment, environment minimisation, absent secrets,
protocol/stderr/resource limits, cleanup, revocation, developer mode, trust drift,
rotation and durable redaction. Weaker conformance cannot approve a stronger
operational claim.

## 14. Removal, first envelope and refusal

Disable/removal stops sessions, prevents new invocation/event admission, removes
active bindings, preserves Trail/replay/event/conformance/trust evidence, and
handles shared credentials by explicit separate choice. Credential revocation
disables dependent readiness.

The first security envelope is Windows x86_64, local MCP 2025-11-25 stdio,
self-contained executable, exact hashing, immutable installation, supervised
ownership, sanitised environment, bounded resources, no reference-provider
network, Windows Credential Manager profile storage, Ed25519 verification and
explicit unsigned developer mode. File Tools and PDF Tools remain credential-
free. Supervised reference-provider mode is not authorised for credential-bearing
production integrations. Operational credential delivery waits for proven
isolated execution or an accepted host-owned credential broker. Storage
implementation must not be confused with delivery isolation. Tethers-authored
references may use supervised mode, but arbitrary third-party packages are not
thereby safe or currently supported. General third-party enablement waits for
proven isolated mode. J18G authorises no implementation.

Deferred: registries, certificates, transparency/timestamps, automatic
revocation, marketplace trust, remote/OAuth/listener/network providers,
interpreters, devices, drivers, shells, administrator providers, cross-user
services, VMs and certified assurance.

Refuse when identity, signature/trust, publisher, payload/dependency integrity,
isolation, filesystem/network boundary, environment minimisation, credential
scope, privilege union, conformance strength, evidence freshness or safe
diagnostic handling cannot be proved.

## 15. J18H obligation and acceptance

J18H must test each representative integration against trust, signature,
isolation, filesystem/network, credentials, environment, process tree, resource
limits, conformance, escape paths and refusal. It must classify supervised,
isolated, brokered or unsuitable. Final freeze remains gated on J18H.

Acceptance requires separate policy/containment, untrusted providers, ordered
trust, precise Ed25519 contract, host publisher trust, honest revocation and
developer mode, distinct profiles, scratch environment, host-owned credentials,
default-deny resources, immutable pre-launch integrity, equal-or-stronger
conformance, no retry on violations, honest reference-provider limits, unchanged
0.2 behaviour, and no implementation/schema/cryptography/sandbox/credential code.

## Primary sources inspected

Accessed 2026-08-01: Microsoft Learn, “Job Objects - Win32 apps”
(https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects);
“AppContainer isolation - Win32 apps”
(https://learn.microsoft.com/en-us/windows/win32/secauthz/appcontainer-isolation);
“Launch an AppContainer - Win32 apps”
(https://learn.microsoft.com/en-us/windows/win32/secauthz/implementing-an-appcontainer);
“Restricted Tokens - Win32 apps”
(https://learn.microsoft.com/en-us/windows/win32/secauthz/restricted-tokens);
“Credentials Management - Win32 apps”
(https://learn.microsoft.com/en-us/windows/win32/secauthn/credentials-management);
“Kinds of Credentials - Win32 apps”
(https://learn.microsoft.com/en-us/windows/win32/secauthn/kinds-of-credentials);
“CredWriteW function (wincred.h) - Win32 apps”
(https://learn.microsoft.com/en-us/windows/win32/api/wincred/nf-wincred-credwritew);
“CryptProtectData function (dpapi.h) - Win32 apps”
(https://learn.microsoft.com/en-us/windows/win32/api/dpapi/nf-dpapi-cryptprotectdata);
“RFC 8032: Edwards-Curve Digital Signature Algorithm (EdDSA)”
(https://www.rfc-editor.org/rfc/rfc8032.html); and “RFC 8410: Algorithm
Identifiers for Ed25519, Ed448, X25519, and X448 for Use in the Internet X.509
Public Key Infrastructure” (https://www.rfc-editor.org/rfc/rfc8410.html).

Two initially guessed Microsoft URLs returned 404; no evidence was fabricated
from them. Lucy supplied the verified Microsoft Learn replacements above, and
Luna inspected the corrected source set. Microsoft documents long-term generic-
credential storage and also states that generic credentials are readable and
writable by user processes. CredWrite associates credentials with the current
token's logon session. AppContainer supplies a stronger credential-isolation
boundary. CryptProtectData is user/machine-bound protection and remains a
deferred alternative rather than the accepted first vault.
