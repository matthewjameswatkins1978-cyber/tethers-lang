# Tethers Security Boundary

Tethers 0.7 provider execution uses process supervision but is not a hostile-code sandbox.
The current guarantees include explicit launch, bounded protocol I/O, retained
child ownership, Windows Job Object lifecycle management, replay protection,
strict manifest checks and redacted outcomes. They do not prove filesystem,
network, credential, token or DLL isolation.

Universal Plug security remains architecture-only. Arbitrary third-party
`.tetherplug` enablement is not yet supported. Packages, signatures and
conformance grant no permission by themselves; host trust, approval, policy,
scope, credentials and containment remain separate.

No automatic retry exists. Job Objects and process-tree termination provide
lifecycle supervision and cleanup; they do not prevent a provider running as
the host user from accessing that user's OS-visible filesystem, network,
account, or other resources. They must not be described as a generic security
sandbox.

Implementation begins only after J18H and J18I acceptance. J18G introduces no
cryptography, sandbox, credential, package-verification, provider, schema, test,
or CLI implementation.
