# Tethers MCP Dependency Survey

Status: M1 complete  
Date: 2026-07-21  
Scope: read-only survey; no dependencies installed, pinned, vendored, or added

## Decision

Do not add `ocaml-mcp` or `snf_mcp` as a Tethers dependency for the first MCP
server milestone.

Use `ocaml-mcp` and the implementation vendored by `snf_mcp` as reference code
for MCP message shapes, lifecycle handling, stdio framing, and tests. If M4
needs a JSON-RPC helper, consider the smaller OCaml `jsonrpc` package directly
after M2 and M3 have fixed the evaluator boundary and transcript fixtures.

If `ocaml-mcp` later publishes a smaller, current, reproducibly pinned library
that supports the current MCP revision without bringing in an async or HTTP
stack, this decision can be revisited.

## Evidence Summary

### `tmattio/ocaml-mcp`

- Repository: <https://github.com/tmattio/ocaml-mcp>, inspected at shallow
  checkout commit `49ee348`.
- Identity: OCaml MCP protocol libraries plus a ready-to-use OCaml development
  MCP server.
- Maintainer/licence: Thibaut Mattio; ISC licence.
- Packaging: source tree contains `mcp.opam`, `mcp-eio.opam`, and
  `ocaml-mcp-server.opam`; no GitHub release is published in the repository
  page inspected on 2026-07-21.
- MCP revision: source declares latest support as `2025-06-18`, with older
  supported versions `2025-03-26`, `2024-11-05`, and `2024-10-07`.
- Current MCP spec: the public MCP specification page redirects to
  `2025-11-25`, so `ocaml-mcp` is behind the currently published revision.
- Lifecycle: core server code tracks initialization, rejects non-initialize
  requests before initialization, handles `notifications/initialized`, and
  provides `ping`.
- Tools support: code includes typed handlers for `tools/list` and
  `tools/call`.
- Cancellation: cancellation notifications are parsed and dispatched, but the
  README says request cancellation is not currently supported.
- Stdio framing: `mcp-eio` writes compact JSON followed by `\n`, reads one
  line at a time, strips trailing `\r`, skips empty lines, and enforces a
  1,000,000 byte read buffer limit.
- Dependency shape: `mcp` depends on `logs`, `jsonrpc`, `jsonschema`,
  `yojson`, `ppx_deriving_yojson`, and `re`; `mcp-eio` depends on Eio and also
  includes socket and HTTP-related dependencies.
- Test signal: the repository has unit and cram tests, including stdio, tools,
  metadata, cancellation, HTTP, filesystem, and OCaml-development tool cases.
  No `.github/workflows` directory was present in the shallow checkout, so
  CI/platform coverage was not established from that repository.
- Windows-native behaviour: not proven from `tmattio/ocaml-mcp` itself.

Assessment: useful and close, but too broad and not current enough for
Tethers M4. It is a reference, not a dependency decision.

### `mseri/snf-mcp`

- Repository: <https://github.com/mseri/snf-mcp>, inspected at shallow checkout
  commit `059ea34`.
- Identity: an OCaml MCP server for web search and content fetching.
- Maintainer/licence: Marcello Seri; ISC licence.
- Published package: opam lists `snf_mcp` `0.2.1`, published 2025-09-10.
- MCP implementation: repository vendors an `ocaml-mcp` implementation rather
  than depending on an opam-published MCP library.
- Transport support: README and source support stdio and HTTP modes.
- Tools support: tests and README cover `tools/list` and `tools/call`.
- Stdio framing: vendored server reads newline-delimited JSON from stdin and
  writes JSON-RPC responses followed by `\n`; the read buffer is bounded at
  1,000,000 bytes.
- Windows signal: CI matrix includes `ubuntu-latest`, `macos-latest`, and
  `windows-latest`; vendored stdio code explicitly treats a Windows broken-pipe
  read as a normal shutdown case.
- Dependency shape: opam dependencies include Eio, Cohttp, TLS, CA certs,
  Mirage crypto, Lambdasoup, Logs, Re, Uri, Yojson, and `jsonrpc`.

Assessment: strong proof that OCaml MCP over stdio can work on Windows, but it
is an application server with network/search dependencies. It should not be a
Tethers dependency or template. Use only as implementation reference.

### OCaml `jsonrpc`

- Package: <https://opam.ocaml.org/packages/jsonrpc/>.
- Latest inspected version: `1.27.0`, published 2026-06-23.
- Maintainer/licence: Rudi Grinberg; ISC licence.
- Dependencies: `dune >= 3.0`, `yojson >= 2.0`, and `ocaml >= 4.08`.
- Compatibility: compatible with the current Tethers local baseline of OCaml
  5.5.0, Dune 3.24.0, and Yojson 2.2.2.
- Scope: JSON-RPC helper only; it does not implement MCP lifecycle, tools,
  schemas, or stdio policy by itself.

Assessment: acceptable future candidate if transcript fixtures show it reduces
code without weakening Tethers' protocol control. Do not add it before M2/M3.

## Resulting Implementation Guidance

For M2, extract the canonical evaluator boundary without MCP dependencies.

For M3, write MCP transcript fixtures from the current MCP specification and
Tethers' own protocol needs, not from a library's defaults.

For M4, implement the narrow local stdio MCP server with one of two paths:

1. Use typed Tethers-owned OCaml modules plus Yojson directly.
2. Add `jsonrpc` only if it materially reduces request/response handling while
   preserving exact Tethers error semantics, initialization rules, stdout
   discipline, and transcript fixture expectations.

Do not add Eio, Cohttp, TLS, Streamable HTTP, OAuth, generic SDK machinery, or
OCaml-development tools to Tethers merely because the surveyed projects use
them.
