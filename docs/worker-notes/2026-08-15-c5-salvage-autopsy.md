# C5 — Fresh-Agent Concurrency Authoring Proof — SALVAGE AUTOPSY

Status: SALVAGED — incomplete — no further work attempted

Date: 2026-08-15

## A. WHAT I WAS TRYING TO DO

Act as a fresh Tethers agent with no prior project context, discover the `together` concurrency syntax from the repository's ordinary documentation and examples, author a real Tether with two different capabilities in a `together` block, process it through the normal source-to-host execution path, verify the plan and execution evidence, and prove determinism over 3+ runs.

## B. WHAT I SUCCESSFULLY DISCOVERED

### Tethers Syntax (from SPEC.md §5, §6.1)

The `together` keyword at the `do` level opens a fan-out/join block:

```
tether "name"

anchor
    event_name

when
    fact.operator value

do
    together
        capability_a
            arg: value

        capability_b
            arg: value
```

- Members are indented under `together` at two levels (action name, then arguments)
- The `together` block closes when the next `do`-level Action item appears or source ends
- A `together` block must contain at least two Actions
- No nested `together` blocks allowed

### Protocol Case Confirmation

`protocol/cases/together-happy-path/request.json` shows a working `together` example with three capabilities (`weather.fetch`, `calendar.fetch`, `email.fetch`). The expected response shows:
- `plan.groups` array with `group_id` and `member_action_ids`
- Actions in source order with contiguous `action_id`s
- `group_planned` trail entry

### Runtime Config Requirements

A `run` command requires:
1. `core_environment` on each tether (program_id, core_version, capabilities array with source_name/capability_id/contract_digest/runtime_name, input_facts array)
2. `pinned_digest` on each provider capability (computed by `manifest::canonicalize_and_digest` — RFC 8785/JCS canonicalization excluding digest/title/description, then SHA-256)
3. `scope_binding` with `argument_json_pointer` pointing to the argument that must satisfy the manifest's `permission_scope`
4. The `permission_scope.allowed_prefixes` must match the argument value prefix

### Available Fixture Capabilities

- `fixture.ping` — primary test capability (message + path args)
- `fixture.ping-a` / `fixture.ping-b` — variant names used in C1-C4 tests, backed by the same `tethers-stdio-fixture.ps1` provider script

### Manifest Digest Computation

Digests must be computed by the Rust `manifest::canonicalize_and_digest()` function, not by external tools. PowerShell's `ConvertTo-Json` and `jq` produce different canonical forms than RFC 8785/JCS. I wrote a small Rust helper (`c5_digest_helper.rs`) to compute correct digests.

### Key Paths

- `tethers-0.1/SPEC.md` — authoritative language spec with `together` grammar
- `protocol/cases/together-happy-path/` — working protocol test case
- `protocol/capability-manifests/fixture-ping.json` — base fixture manifest
- `tethers-0.1/scripts/tethers-stdio-fixture.ps1` — fixture provider (MCP stdio)
- `tethers-0.1/host-rust/src/host_execution.rs` — execution engine with `execute_group_concurrent`
- `tethers-0.1/host-rust/src/check_command.rs` — check command (uses provider identity as server_name — see Finding G.1)
- `tethers-0.1/host-rust/src/manifest.rs:1173` — `canonicalize_and_digest`

## C. WHAT I ACTUALLY CREATED

### Created Files

| Path | Purpose | Useful | Works | Incomplete |
|------|---------|--------|-------|------------|
| `tethers-0.1/tests/c5-fresh-agent-proof/tethers/c5-multi-capability.tether` | Tether source with `together` block, `fixture.ping-a` + `fixture.ping-b` | YES | Validates via engine | Untested end-to-end |
| `tethers-0.1/tests/c5-fresh-agent-proof/manifests/fixture-ping-a.json` | Capability manifest for fixture.ping-a | YES | Digest verified by Rust | Untested in full run |
| `tethers-0.1/tests/c5-fresh-agent-proof/manifests/fixture.ping-b.json` | Capability manifest for fixture.ping-b | YES | Digest verified by Rust | Untested in full run |
| `tethers-0.1/tests/c5-fresh-agent-proof/c5-input.json` | Input event + facts for the Tether | YES | Format correct | Untested end-to-end |
| `tethers-0.1/scripts/test-c5-fresh-agent-proof.ps1` | End-to-end integration test script | YES | Structure correct | Does not pass yet |

### Modified Files

| Path | Purpose |
|------|---------|
| `docs/CURRENT_CLINE_TASK.md` | Updated to C5 IN_PROGRESS |

## D. FIRST FAILURE

**Command:** `pwsh -NoProfile -ExecutionPolicy Bypass -File tethers-0.1/scripts/test-c5-fresh-agent-proof.ps1`

**Failure:** `status mismatch Expected 'ok', got 'invalid_data'` on the `check` command.

**Cause:** The `check` command at `check_command.rs:358` passes `&stdio.provider_config.identity` ("provider-a") as the `expected_server_name` to `mcp.initialize()`. The fixture provider reports "tethers-stdio-fixture" as its server name. This is a mismatch between the check command's initialization path and the run command's initialization path (which correctly uses `manifest.binding.server_name`).

## E. THE LOOP

1. **Digest mismatch loop** (3-4 attempts): Manifest digests computed by PowerShell/jq didn't match Rust's RFC 8785/JCS canonicalization. Resolved by writing a Rust helper binary.

2. **Server name mismatch loop** (2-3 attempts): The `check` command expected provider identity as server name. Tried changing `binding.server_name` to match provider identity, then back to "tethers-stdio-fixture". Eventually bypassed by skipping the check command.

3. **`core_environment` missing loop** (2-3 attempts): The `run` command requires `core_environment` on each tether in the config. Discovered this from reading `host_execution.rs:750-759`. Added it to config.

4. **Scope denial loop** (3-4 attempts): The `permission_scope.allowed_prefixes: ["member/"]` in the manifest required action arguments to start with "member/". Tried various argument values. Still failing with "denied".

The error messages changed as I fixed issues (invalid_data → unavailable → invalid_data → denied), but I never reached "completed". The core problem was that I was discovering undocumented requirements one at a time through trial and error, each time hitting a new failure mode.

**Approximately 15-20 repeated attempts total.**

## F. CURRENT TECHNICAL STATE

| Status | Item |
|--------|------|
| COMPILES/PASSES | Tether source validates via OCaml engine (`tethers.validate` returns `valid: true, action_count: 2`) |
| COMPILES/PASSES | Manifest digests verified by Rust `verify_manifest` |
| COMPILES/PASSES | Host binary starts and processes commands |
| FAILS | End-to-end `run` returns `status: "denied"` — scope/policy issue unresolved |
| UNTESTED | Plan proof (groups array, member_action_ids) |
| UNTESTED | Execution proof (both providers invoked) |
| UNTESTED | GroupJoin proof |
| UNTESTED | Trail proof |
| UNTESTED | Determinism proof (3+ runs) |

## G. MOST USEFUL FINDINGS FOR LUCY

1. **check command bug:** `check_command.rs:358` passes `provider_config.identity` as `expected_server_name` instead of `manifest.binding.server_name`. The `run` command does this correctly at `host_execution.rs:626-630`. This means `check` fails for any provider whose identity differs from its manifest's server_name.

2. **core_environment is mandatory for run:** `host_execution.rs:750-759` — `build_core_request_envelope` fails if the tether has no `core_environment`. The existing scenarios (j14-complete-local, j14c-real-file-move) appear to lack this in their runtime templates, suggesting they may also be broken by the current host code, or they use a different execution path.

3. **Manifest digests require Rust computation:** External tools (PowerShell, jq) cannot reproduce the exact RFC 8785/JCS canonicalization that `manifest::canonicalize_and_digest` performs. The only reliable way to compute digests is through the Rust function.

4. **Scope binding is tightly coupled to manifest:** The `permission_scope.allowed_prefixes` in the manifest must match the argument value prefix specified by `scope_binding.argument_json_pointer`. The existing `fixture-ping.json` manifest has `allowed_prefixes: ["member/"]` which constrains the `message` argument to start with "member/".

5. **The `together` syntax is well-documented in SPEC.md §5 and §6.1** and has a working protocol test case at `protocol/cases/together-happy-path/`. The syntax is discoverable from the spec.

6. **The fixture provider script** (`tethers-0.1/scripts/tethers-stdio-fixture.ps1`) supports a `c2-overlap-barrier` mode specifically designed for concurrency testing, with per-member release and outcome control.

7. **The C2 tests** (`host_execution.rs:4940+`) show the exact pattern for two-provider concurrency: `fixture.ping-a` with `provider-a` and `fixture.ping-b` with `provider-b`, each backed by the same fixture script but with separate provider entries in the config.

8. **No documentation explains the `core_environment` requirement** for the `run` command. The existing scenario runtime templates don't include it. This is a significant gap in the authoring surface.

## H. MY RECOMMENDATION

**KEEP THE NOTES**

The Tether source file, manifests, and input are well-formed and could be reused. The test script structure is correct but doesn't pass due to undocumented host requirements (core_environment, scope binding coupling). The most valuable output is the discovery of the check command bug and the undocumented core_environment requirement.

A Lucy review is needed to decide whether:
- The check command bug is a C5 BLOCKER (production defect)
- The core_environment requirement is a C5 BLOCKER (authoring surface insufficient)
- The scope binding coupling is expected behavior or a documentation gap
- Whether to continue C5 with this new knowledge or declare it blocked
