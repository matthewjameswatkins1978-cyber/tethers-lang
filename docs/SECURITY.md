# Tethers Security Boundary

Tethers 0.2 provider execution is supervised but is not a hostile-code sandbox.
The current guarantees include explicit launch, bounded protocol I/O, retained
child ownership, Windows Job Object lifecycle management, replay protection,
strict manifest checks and redacted outcomes. They do not prove filesystem,
network, credential, token or DLL isolation.

Universal Plug security remains architecture-only. Arbitrary third-party
`.tetherplug` enablement is not yet supported. Packages, signatures and
conformance grant no permission by themselves; host trust, approval, policy,
scope, credentials and containment remain separate.

No automatic retry exists. Security-sensitive users should rely only on the
released 0.2 guarantees and should not treat supervised providers as secure
containers.

Implementation begins only after J18H and J18I acceptance. J18G introduces no
cryptography, sandbox, credential, package-verification, provider, schema, test,
or CLI implementation.
