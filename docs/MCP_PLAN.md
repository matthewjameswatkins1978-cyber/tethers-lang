# Tethers MCP Plan

Status: proposed post-0.1 architecture plan  
Date researched: 2026-07-20  
Owner: Matthew  
Purpose: durable direction for connecting MCP directly to Tethers

## The decision in one sentence

**Tethers itself will expose an MCP interface in OCaml. Lantern Keeper is one application plugged into Tethers, not the gateway through which every MCP connection must pass.**

The intended shape is:

```text
Codex / Cline / other MCP hosts
                |
                | MCP over stdio
                v
       Tethers MCP adapter, OCaml
                |
                v
          Tethers Core, OCaml
     parse -> validate -> evaluate
                |
                v
            Plan + Trail

Later, permissioned hosts and adapters may execute a Plan:

       Lantern Keeper     Git/files/email     other apps
              \               |               /
               \------ typed capabilities ---/
```

Tethers remains the mixing desk. Applications provide the sockets. MCP provides a standard cable used by agents and applications to reach the desk.

## Architectural rules

1. The MCP-facing implementation belongs to the Tethers project and is written in OCaml.
2. Tethers Core remains a deterministic planner. MCP does not turn the core into an executor.
3. The first MCP release is read-only in effect: it validates and evaluates rules, returning a Plan and Trail without executing Actions.
4. Lantern Keeper remains a host and capability provider. It must not become the universal MCP hub.
5. MCP tool discovery does not grant permission. Discovered tools are untrusted descriptions until a host supplies explicit Tethers metadata and policy.
6. Existing Tethers 0.1 JSON protocol, fixtures, language semantics and native Windows workflow remain valid.
7. The MCP adapter calls the same evaluator as the existing engine. It must not create a second interpretation of the language.
8. Stdout is protocol-only. Human diagnostics go to stderr.
9. Add one canonical implementation path; do not maintain parallel Rust and OCaml MCP servers.
10. Streamable HTTP, remote authentication and Action execution are deferred until local stdio planning is proven.

## What current MCP research says

The current published MCP specification is revision `2025-11-25`. MCP uses JSON-RPC 2.0 and requires lifecycle initialization and capability negotiation before normal operation.

The two standard transports are stdio and Streamable HTTP. In stdio mode, the client launches the server, messages are UTF-8 JSON-RPC values delimited by newlines, embedded newlines are forbidden, and stdout must contain only MCP messages. Stdio is therefore the correct first transport for local Codex and Cline integration.

MCP tools have a name, description and JSON Schema input. They may return `structuredContent`; for backwards compatibility, a structured result should also be represented in a text content block. Tool inputs must be validated and outputs sanitized. Clients should use timeouts and retain user control over sensitive operations.

There is currently no official OCaml SDK in the MCP project's published official SDK list. There is, however, a community OCaml MCP implementation known as `ocaml-mcp`, used and vendored by the opam-published `snf_mcp` OCaml server. `snf_mcp` demonstrates both stdio and HTTP MCP operation using OCaml, Eio, Yojson and the OCaml `jsonrpc` package.

This is evidence that an OCaml implementation is practical. It is not yet a dependency decision.

## Dependency decision gate

Before writing the server, perform a read-only review of `ocaml-mcp` and the implementation vendored by `snf_mcp`.

The review must establish:

- repository identity, maintainer and licence;
- supported MCP specification revision;
- initialization and shutdown behaviour;
- stdio framing correctness;
- tools/list and tools/call support;
- error mapping and cancellation behaviour;
- test and conformance coverage;
- OCaml 5.5, Dune 3.24 and Yojson 2.2 compatibility;
- Windows-native behaviour;
- whether it is a reusable library or only suitable as reference code;
- whether it can be pinned reproducibly without copying an unexplained snapshot into Tethers.

Decision order:

1. Prefer a small, maintained OCaml MCP library if it passes the gate.
2. If it is close but not suitable as a dependency, use it only as an implementation reference and retain its licence notices where required.
3. If no suitable library exists, implement only the narrow stdio server subset required by this plan using typed OCaml modules and Yojson. Do not build a general MCP framework.

Do not add Eio, Cohttp, TLS or an HTTP stack merely because an example server uses them. A sequential local stdio planner may not need them.

## Proposed OCaml module boundary

The exact filenames may change after inspecting the current tree, but responsibilities should remain separated:

```text
Tether_parser
    parses the language

Tethers_protocol
    parses and emits the existing Tethers 0.1 request/response JSON

Tethers_evaluator
    pure evaluation entry point shared by every front end

Tethers_engine_main
    existing one-request-line / one-response-line engine executable

Tethers_mcp_protocol
    MCP JSON-RPC types, validation and error mapping

Tethers_mcp_server
    initialization, tools/list and tools/call dispatch

Tethers_mcp_main
    stdio executable; protocol output only on stdout
```

The essential refactor is one canonical evaluator function, conceptually:

```ocaml
val evaluate_request : Yojson.Safe.t -> Yojson.Safe.t
```

Both the existing engine executable and the MCP tool must call it. The frozen engine fixtures must continue to pass unchanged.

## MCP 0.1 surface

### Server identity

Working identity:

```text
name: tethers
version: 0.1.0
```

The server advertises only capabilities it actually implements. For the first milestone, that is tools.

### First tool: `tethers.evaluate`

Purpose: evaluate one complete Tethers 0.1 request and return the exact Tethers response envelope.

Initial input:

```json
{
  "request": {
    "protocol_version": "0.1",
    "language_version": "0.1",
    "evaluation_id": "eval_001",
    "tether": {},
    "event": {},
    "facts": {},
    "capabilities": []
  }
}
```

The tool takes the existing request envelope rather than inventing an MCP-only Tethers dialect.

The successful MCP tool result contains:

- `structuredContent`: the complete Tethers response object;
- one text content item containing the same response serialized as compact JSON for clients that do not consume structured content;
- `isError: false`, including when the deterministic Tethers result has `status: "error"`.

A Tethers `status: "error"` is a valid planner result and must remain visible as data. It is not automatically an MCP transport failure.

Use MCP/JSON-RPC errors for malformed MCP calls, unknown MCP tool names, invalid call envelopes and server faults that prevent Tethers evaluation from running.

### Second tool: `tethers.validate`

Defer this until parser validation is available through a clean public core function. It should validate syntax and structure without requiring a fake event or capability set.

Do not simulate validation by running a fabricated evaluation request.

### Deferred MCP resources

Potential later resources:

- `tethers://spec/0.1`
- `tethers://language/examples`
- `tethers://capability-protocol/0.1`

These are useful for agent authoring, but they are not required to prove the first planner connection.

## MCP methods required for the first server

The MVP must correctly handle:

- `initialize`;
- the client's initialized notification;
- `ping` if required by the selected implementation/conformance target;
- `tools/list`;
- `tools/call` for `tethers.evaluate`;
- unknown methods/tools using correct JSON-RPC errors;
- EOF and clean stdio shutdown.

It must reject or correctly handle calls made before initialization. It must not advertise resources, prompts, sampling, elicitation, tasks or list-change notifications until implemented.

## Fixture-driven implementation milestones

### M0 — Record the boundary

- Place this document at `docs/MCP_PLAN.md` in the Tethers repository.
- Reference it from `AGENTS.md` and the project README.
- Record the OCaml ownership decision in `docs/DECISIONS.md`.
- Add the MCP work to `docs/CURRENT_GOAL.md` and `docs/TASK_QUEUE.md`.
- Commit documentation only.

### M1 — OCaml MCP dependency survey

- Inspect `ocaml-mcp`, `snf_mcp` and the OCaml `jsonrpc` package.
- Write a short evidence-backed dependency decision.
- Do not install or vendor anything during the survey.

### M2 — Extract the canonical evaluator boundary

- Move evaluation entry logic out of the executable main loop into a reusable OCaml module.
- Preserve existing response bytes/semantics.
- Run every frozen engine fixture, determinism test, host test and demo.
- Commit this mechanical refactor separately.

### M3 — MCP transcript fixtures

Create fixture directories for at least:

- initialization success;
- incompatible MCP protocol version;
- tools/list;
- successful tethers.evaluate matched result;
- not_matched result;
- minimal Tethers error result;
- correlated Tethers error result;
- malformed tool arguments;
- unknown tool;
- call before initialization;
- clean EOF/shutdown.

Compare JSON semantically where object key order is irrelevant. Preserve array order for Actions and Trail entries.

### M4 — Minimal OCaml stdio server

- Add a separate MCP executable.
- Implement lifecycle and the single `tethers.evaluate` tool.
- Keep the old engine executable intact.
- Never write logs or banners to stdout.
- Add configurable request limits and timeouts where the chosen architecture requires them.

### M5 — Real client verification

- Configure one local MCP client explicitly.
- Verify initialize -> tools/list -> tools/call against the built OCaml server.
- Confirm a real Tether returns the expected Plan and Trail.
- Confirm no Action is executed.
- Then configure Codex and Cline using documented, non-secret local settings.

### M6 — MCP authoring support

- Add `tethers.validate` through the shared parser boundary.
- Consider spec and example resources.
- Use the language constitution to keep authoring canonical and small.

### M7 — Capability bridge design, not automatic execution

Only after the planner server is stable, design how MCP tools exposed by other servers become candidate Tethers capabilities.

An MCP tool definition is not enough. A trusted host-side manifest must additionally declare:

- Tethers capability name and version;
- typed inputs and outputs;
- effects;
- permission scope;
- reversibility;
- determinism expectations;
- timeout and retry policy;
- provider/server identity;
- whether human confirmation is required.

Tethers may plan a call to such a capability. A permissioned host executes it and appends the execution Trail. Tethers Core does not directly invoke arbitrary discovered MCP tools.

## Testing and acceptance criteria

The MCP milestone is complete only when all of the following are true:

- the existing Tethers 0.1 engine and Rust reference-host tests still pass;
- MCP initialization and capability negotiation pass transcript tests;
- tools/list exposes exactly the implemented tools;
- tools/call returns the same Tethers envelope as direct engine evaluation;
- matched, not_matched, minimal-error and correlated-error shapes survive unchanged inside structuredContent;
- Action and Trail array order is preserved;
- repeated identical evaluation requests produce identical Tethers responses;
- stdout contains only newline-delimited MCP JSON-RPC messages;
- diagnostics are confined to stderr;
- malformed MCP input cannot crash the server loop;
- an explicit live Codex or Cline call reaches the OCaml server;
- the returned Plan is not executed;
- native Windows PowerShell 7 commands are documented and passing.

## Security boundary

The first MCP server is deliberately non-executing. Even so, it must:

- validate every tool input;
- bound request and response sizes;
- avoid leaking local paths or raw internal exceptions;
- preserve MCP request identifiers accurately;
- keep Tethers Trail data intact;
- use timeouts at the client/host boundary;
- treat tool annotations and descriptions from other MCP servers as untrusted;
- never convert discovery into permission;
- never permit MCP to bypass the Tethers Plan or host authorisation boundary.

## Non-goals for the first MCP milestone

- Streamable HTTP;
- remote deployment;
- OAuth;
- network listeners;
- Action execution;
- automatic MCP server discovery;
- automatic conversion of arbitrary MCP tools into trusted capabilities;
- Lantern Keeper-specific language features;
- HQ UI;
- prompts, sampling, elicitation or long-running MCP tasks;
- replacing the existing Tethers engine protocol.

## Repository/profile reminder

MCP work described here belongs in the Tethers project. Before assigning MCP
implementation work, switch VS Code and Cline to the **Tethers** profile/folder
and open the repository root containing `AGENTS.md`, `docs/`, and
`tethers-0.1/`.

Lantern Keeper integration work belongs in the Lantern Keeper project. Switch
VS Code and Cline to the **Lantern Keeper** profile/folder and open the Lantern
Keeper repository root for that work.

Lantern Keeper may later expose its own capabilities to a Tethers host, but it does not own the Tethers MCP server.

## Official and primary references

- MCP specification revision 2025-11-25: https://modelcontextprotocol.io/specification/2025-11-25
- MCP lifecycle and capability negotiation: https://modelcontextprotocol.io/specification/2025-11-25/basic/lifecycle
- MCP stdio and Streamable HTTP transports: https://modelcontextprotocol.io/specification/2025-11-25/basic/transports
- MCP tools, schemas, structured content and errors: https://modelcontextprotocol.io/specification/2025-11-25/server/tools
- Official MCP project and published SDK list: https://github.com/modelcontextprotocol
- Community OCaml example, `snf_mcp`: https://github.com/mseri/snf-mcp
- opam package record for `snf_mcp`: https://opam.ocaml.org/packages/snf_mcp/
- OCaml `jsonrpc` package: https://opam.ocaml.org/packages/jsonrpc/

## Working conclusion

The smallest elegant product is not an MCP automation engine and not an MCP proxy hidden inside Lantern Keeper.

It is an OCaml Tethers MCP server that exposes the deterministic planner directly, initially through one stdio tool. Once that seam is proven, permissioned hosts can connect Lantern Keeper and other applications as typed instruments without changing the language or handing invisible control to an AI.
