# Tethers Security Boundary

Status: current security summary  
Updated: 2026-09-10
Tethers 0.7 provider execution uses process supervision but is not a hostile-code sandbox.
The current guarantees include explicit launch, bounded protocol I/O, retained
child ownership, Windows Job Object lifecycle management, replay protection,
strict manifest checks and redacted outcomes. They do not prove filesystem,
network, credential, token or DLL isolation.

Tethers has a serious trust and execution model, but it is important to describe exactly what that model does and does not guarantee.

## The short version
No automatic retry exists. Job Objects and process-tree termination provide
lifecycle supervision and cleanup; they do not prevent a provider running as
the host user from accessing that user's OS-visible filesystem, network,
account, or other resources. They must not be described as a generic security
sandbox.

> **Tethers can constrain and evidence provider execution. It does not turn arbitrary provider code into harmless sandboxed code.**

The full reference host verifies contracts, authority, scope, replay state, provider identity, and outcomes around an execution.

That is different from operating-system isolation.

For AI use, this distinction is the point rather than a footnote. Tethers is designed to make the boundary between “the agent wants this” and “this operation is trusted, scoped, authorised, attempted, and evidenced” explicit. It is not designed to make malicious native code safe merely by calling it a Plug.

## Two different execution surfaces

### Portable workbench

`tethers-0.1/portable-rust/` is a local authority façade.

It returns:

```text
ALLOW
ASK
DENY
```

It does **not** execute the requested action.

Its risk is therefore primarily incorrect authority classification, malformed configuration, or caller misuse. The workbench fails closed on malformed or unknown input.

### Full reference host

`tethers-0.1/host-rust/` can execute approved Capability Actions through installed Plug providers.

That path has a much larger security boundary and uses multiple independent controls.

## Trust layers in the full host

A normal provider call is not authorised merely because a tool exists.

Relevant layers include:

1. **Package evidence**  
   `.tetherplug` structure, payload evidence, package identity, and semantic package digest are validated.

2. **Conformance**  
   The provider can be exercised against the declared Socket/protocol contract. Conformance is evidence only. It is not installation, enablement, or authority.

3. **Installation state**  
   Staging and installation are host-owned lifecycle operations.

4. **Enablement and operational scope**  
   Installed Plugs are enabled with explicit scope evidence. A provider cannot widen its own host-approved scope by advertising a larger one.

5. **Trusted Capability manifest**  
   The host uses reviewed versioned manifests containing input/output schemas, Effects, provider binding, scope declarations, timeout/retry information, and other contract data.

6. **Live binding checks**  
   Provider identity, MCP server/tool binding, and relevant live discovery data are rechecked. Drift does not silently become a new trusted operation.

7. **Policy and approval**  
   Capability schemas do not grant authority. Effective host policy may allow, deny, require one-shot approval, or make a Capability unavailable.

8. **Durable intent**  
   Effectful execution crosses an intent-first boundary. If required intent evidence cannot be durably established, the effectful call does not proceed.

9. **Replay protection**  
   Replay state prevents completed or ambiguous prior execution from being treated as a clean fresh call.

10. **Deadline and outcome classification**  
    Provider calls are bounded and outcomes distinguish definite failure from post-invocation uncertainty.

11. **Output validation**  
    A provider success value must satisfy the trusted output schema before it can be treated as a validated success result.

12. **Trail and Result Anchor evidence**  
    Execution identity, capability/provider evidence, results, failures, uncertainty, joins, and causal events are recorded without turning proposals into fake executions.

## Conformance is not trust

The public Plug lifecycle intentionally keeps these separate:

```text
pack
inspect
conform
stage
install
enable
execute
```

A package that passes `plug conform` has demonstrated protocol behaviour under the conformance conditions.

It has **not** thereby gained:

- permission to install itself;
- permission to enable itself;
- unrestricted filesystem scope;
- network access;
- credentials;
- approval for every call;
- authority to redefine its own Effects;
- authority to change canonical outcomes.

## Provider code is not sandboxed

Current provider execution is supervised, but it is **not a hostile-code sandbox**.

Depending on platform and execution path, the host can include process ownership and supervision, bounded protocol I/O, deadlines, strict stdio discipline, shutdown checks, and platform-specific lifecycle controls.

Those controls do not prove isolation from:

- arbitrary filesystem access by malicious provider code;
- arbitrary network access;
- environment or credential theft;
- DLL or dynamic-library loading attacks;
- kernel or OS vulnerabilities;
- side channels;
- malicious native code outside the declared protocol;
- denial of service beyond the host's bounded supervision guarantees.

Do not install or execute untrusted native provider code merely because its package is well-formed or conforming.

## Scope

Scope is an execution boundary, not prose.

The generic host carries and validates operational scope evidence. The provider interprets domain-specific scope where required and must enforce the contract it declares.

Examples can include:

- canonical path roots;
- byte limits;
- repository bounds;
- named resources;
- executable allow-lists;
- bounded environment keys;
- other structured provider-specific limits.

Scope must not be inferred from a friendly description string.

Ambiguous, missing, malformed, or mismatched scope evidence fails closed.

## Effects and permission

Effects describe the kinds of consequences a Capability may have.

They are inputs to policy, not grants of permission.

```text
Capability manifest -> describes Effects
Host policy         -> authorises
Host/runtime        -> enforces
Trail               -> records
```

A reversible operation is not automatically safe. A deterministic operation is not automatically permitted. A provider advertising an operation is not evidence that the host trusts it.

## Secrets and credentials

Secrets must not be placed casually in:

- Tether source;
- Capability descriptions;
- package metadata;
- public Trails;
- discovery output;
- provider stdout protocol noise.

Provider stdout is protocol data. Diagnostics belong on stderr.

Credential handling belongs to host-owned configuration and brokering boundaries rather than provider self-declaration.

The repository's architecture contains broader credential and sandbox design work. Do not overstate an architecture document as an implemented isolation guarantee.

## Replay and uncertainty

Tethers deliberately refuses a dangerous simplification:

```text
no response == failure == safe retry
```

If a call may have reached a provider but no trustworthy final response is available, the outcome is uncertain.

The host does not automatically retry arbitrary effectful calls.

> **No automatic retry until idempotency is proved end to end.**

Recovered ambiguous replay states may require explicit manual resolution rather than a second provider call.

This conservative treatment is particularly important for autonomous callers, which otherwise tend to interpret “something went wrong” as an invitation to try the same operation again.

## Concurrency

Together concurrency does not weaken the trust model.

Before a Together member can invoke its provider, it still crosses the relevant preparation gates for Capability resolution, policy/scope, replay, intent, and Trail evidence.

Physical completion order is not allowed to rewrite semantic order.

If a fatal trusted-state failure prevents new members from launching, already-running members are still allowed to terminalise truthfully so the Trail does not lie about effects that may already have happened.

## Result Anchors are evidence, not omniscience

`capability.succeeded` means the trusted execution path accepted a provider success result for that Capability.

It does not necessarily prove an independently observed physical-world fact.

For example, a device provider may successfully report that it issued a command while a separate sensor later reports whether the physical outcome occurred.

Tethers keeps those claims distinct.

## Discovery and preview do not grant authority

The published Tethers 0.6 release line includes read-only discovery, Capability inspection, installed Plug inspection, side-effect-free planning, and bounded Trail receipt surfaces.

Those are observation surfaces. They do not:

- start a provider merely to inspect trusted configuration;
- install or enable a Plug;
- grant operational scope;
- approve a planned Action;
- turn a preview into durable execution evidence.

Agents should be able to learn what Tethers can do without that act of discovery itself becoming consequential.

## Platform note

The published Tethers 0.6 practical release provides Windows x64 and Linux x64 musl bundles. The smaller Portable Workbench is also packaged for both platforms.

That packaging fact should not be overstated into a claim that every platform-specific containment or durability mechanism is identical. Some host lifecycle and containment paths are platform-specific, and the security guarantee is the documented semantic/trust boundary, not “Windows and Linux implement every low-level mechanism in exactly the same way.”

When evaluating a particular platform, verify the tagged release source and tests for the feature you depend on.

## What Tethers currently protects well

Tethers is strongest at making these boundaries explicit and testable:

- intent versus permission;
- trusted manifest versus provider advertising;
- scoped Capability versus unrestricted tool access;
- proposed Plan versus actual execution;
- first attempt versus replay;
- success versus failure versus uncertainty;
- semantic order versus physical completion order;
- provider result versus later external observation;
- current trust state versus stale package/manifest evidence.

That is a substantial security contribution even though it is not process sandboxing.

## Release/source note

The latest published GitHub product release is Tethers 0.6, tagged `tethers-v0.6.0`.

Security claims about exact 0.6 discovery/provider surfaces should be verified against the tagged release source and its SHA-256 package evidence.

## Read the deeper contracts

For detailed architecture and evidence:

- [`CAPABILITY_BRIDGE.md`](CAPABILITY_BRIDGE.md)
- [`architecture/TETHERS_SECURITY_TRUST_CREDENTIALS_SANDBOX_V1.md`](architecture/TETHERS_SECURITY_TRUST_CREDENTIALS_SANDBOX_V1.md)
- [`architecture/TETHERS_UNIVERSAL_PLUG_ARCHITECTURE.md`](architecture/TETHERS_UNIVERSAL_PLUG_ARCHITECTURE.md)
- [`J09_DURABLE_REPLAY_DESIGN.md`](J09_DURABLE_REPLAY_DESIGN.md)
- [`concurrency/C2_A3_PHYSICAL_CONCURRENCY_DESIGN.md`](concurrency/C2_A3_PHYSICAL_CONCURRENCY_DESIGN.md)
- [`concurrency/C3_BOUNDED_CONCURRENCY_DESIGN.md`](concurrency/C3_BOUNDED_CONCURRENCY_DESIGN.md)

Those documents include historical design-stage wording. Use this file, the relevant tagged/current code, and tests for present-day summary claims.
