# Integrating Tethers

## Supported integration boundary

Tethers 0.8.1 exposes `tethers.authority/1` through the
`tethers gate --stdio` process interface. A consumer starts the `tethers`
executable, negotiates the protocol with `hello`, sends versioned JSON frames,
and validates every response schema and request identity. See the frozen
[authority protocol](tethers.authority.1.md),
[external Host architecture](architecture/TETHERS_R2_EXTERNAL_AUTHORITY_GATE.md),
and the [small external consumer example](../examples/external-consumer/README.md).

For Rust source integration, the `tethers-reference-host` crate is packaged
with a narrow `tethers_reference_host::authority_v1` entry point. Pin it by
the `v0.8.1` Git tag until a registry release is deliberately made; see the
[crate README](../tethers-0.1/host-rust/README.md) and
[pinned Rust consumer](../examples/rust-crate-consumer/). The crate interface
and the process protocol are separate consumption routes. The versioned
process protocol remains the language-independent contract.

Product line 0.8.1 keeps the same `tethers.authority/1` contract and the same
`authority_v1` entry point; only the product/crate version advances. Crates.io
publication remains gated, so source consumers keep pinning the released
`v0.8.1` tag above.

The consumer provides Tethers Core's engine path, runtime configuration,
Trail path, and an already provisioned host-data root. PREPARE invokes Core to
produce and check the Plan. The Gate decides current host authority and emits
no provider calls. `allow_prepared` and human approval both still require a
successful COMMIT. COMMIT records durable intent and returns a
`tethers.dispatch/1` record; the external Host alone performs the physical
effect and must report its observed result with `outcome`. A Plan, approval,
COMMIT, or dispatch record is not evidence that the effect happened.

Reject unsupported schemas, malformed responses, duplicate identities, and
caller-supplied authority fields. Do not coerce `ask`, `deny`, or `unavailable`
into allow. Do not retry physical effects without reconciling Gate `status`,
Trail, and replay state.

## Installation and availability

The 0.8.1 full runtime is distributed as versioned release archives for
Windows x86-64 (`.zip`), Linux x86-64 (`.tar.gz`), and macOS (`.tar.gz` for
Apple Silicon ARM64 official and Intel x86-64 compatibility lane), each
containing the `tethers` host and matching OCaml engine. The reusable Rust
crate is versioned and package-ready as `tethers-reference-host 0.8.1`;
crates.io publication remains deliberately gated. A consumer can use the
crate from the versioned source tag or build and pin the exact crate archive.
The OCaml engine remains an executable package, not a public library.

Before first Gate use, initialize the host data store with the release-managed
administrative command `tethers provision-replay ABSOLUTE_ROOT`. It is hidden
from ordinary CLI help and must be treated as setup, never as an implicit
fallback during a consequential request. The host-data root must be private to
the consumer and stable across restarts.

## Consumer proof

The example under `examples/external-consumer/` is an unrelated standard
library Python Host. It uses only the `tethers` process boundary and a fixture
Capability/Tether. Its smoke test covers allow, ask before and after explicit
approval, deny, the durable Trail outcome receipt, malformed input, unsupported
protocol identity, and a forged authority field. The only physical effect is a
temporary marker written by the external Host after a permitted dispatch.

Run it against the exact built product and Core engine:

```powershell
python examples/external-consumer/smoke.py --tethers C:\path\to\tethers.exe --engine C:\path\to\tethers_mcp_main.exe
```

## Current limits

- The authority protocol is frozen at `tethers.authority/1`; product version
  and protocol version are independent.
- The Rust crate is package-ready under MIT OR Apache-2.0; crates.io
  publication remains deferred. The OCaml engine is still an executable
  package, not a public library.
- The repository support matrix treats Windows x86-64, Linux x86-64, and
  macOS ARM64 as official full-runtime release targets, with macOS Intel x86-64
  as a verified compatibility lane.
- Host policy, human approval UI, physical effects, secret handling, process
  supervision, and recovery decisions remain the consuming Host's duties.
