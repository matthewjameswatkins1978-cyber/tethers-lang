# J18C Worker Note

## Task

J18C - Tethers Socket v1 Contract and MCP Stdio Binding.

## Changes

Added the Socket v1 semantic contract and MCP 2025-11-25 local stdio binding.
Aligned J18B acceptance, historical MCP and capability-bridge authority notes,
decision log, current state, task packet, and this worker note.

## Decisions and assumptions

Socket is semantic, not transport. The Tethers host is MCP client and the Plug
provider is MCP server. Standard MCP methods are used; JSON-RPC IDs remain
session-local correlation. Discovery and provider output remain untrusted until
host validation. No automatic retry or implementation is authorised.

## Existing protocol inspected

Inspected the existing Core-facing MCP plan, capability bridge, Universal Plug
architecture, OCaml MCP entry points, Rust manifest/provider structures, and MCP
transcript test locations. No implementation files were modified.

## Evidence

Confirmed base `2930fd4c672805b89eef566d4315a4773f6bd603` and peeled release
`b5546411661dcbcb53e1cf2538eaec594c6f76f2`. Confirmed WinGet installations:
ripgrep 15.2.0, fd 10.4.2, jq 1.8.2, yq 4.53.3, and gh 2.97.0. The four
required tool directories were exposed only in the verification process.

## Discoveries

The historical MCP plan describes Tethers as the server; the J18C binding
explicitly documents the opposite host-as-client provider direction without
rewriting that historical plan. Existing bridge retry examples are now marked
non-authoritative by the status note.

## Remaining risks

J18H paper validation and Lucy protocol review remain required. Exact runtime
limits and final canonical outcome mapping remain deferred to implementation
planning and J18F respectively.

## Next action

Lucy reviews J18C. Do not begin J18D or implementation before acceptance.

## References

- `docs/architecture/TETHERS_SOCKET_V1.md`
- `docs/architecture/TETHERS_SOCKET_V1_MCP_STDIO_BINDING.md`
- `docs/architecture/TETHERS_UNIVERSAL_PLUG_ARCHITECTURE.md`
- `docs/MCP_PLAN.md`
- `docs/CAPABILITY_BRIDGE.md`
