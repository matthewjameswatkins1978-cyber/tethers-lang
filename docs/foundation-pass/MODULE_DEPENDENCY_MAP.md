# F1 Module and Dependency Map

## Rust Host (`tethers-0.1/host-rust/`)

Crate: `tethers-reference-host` v0.2.0, Rust edition 2021, MSRV 1.97

### External Dependencies

| Dependency | Version | Purpose |
|---|---|---|
| `serde` | 1.0 (derive) | Serialization framework |
| `serde_json` | 1.0 | JSON parsing |
| `serde_json_canonicalizer` | 0.3 | JCS RFC 8785 canonicalization |
| `sha2` | 0.10 | SHA-256 hashing |
| `base64` | 0.22 | Base64 encoding |
| `ed25519-dalek` | 2.1.1 (pkcs8) | Ed25519 signatures |
| `clap` | 4 (derive) | CLI argument parsing |
| `zip` | 2.4.2 (deflate) | ZIP archive handling |
| `uuid` | 1 (v4) | UUID generation |
| `windows-sys` (cfg windows) | 0.61 | Win32 API bindings |

### Architecture Layers

```
CLI (main, cli)
  |
Command Coordinators (check_command, run_command, trail_command, plug_command, plug_install_command)
  |
Runtime (configured_runtime, runtime_config, application)
  |
Execution Services (host_execution, execution_environment, engine_stdio, dispatch, executor, event_admission, event_queue, outcome)
  |
Capability Resolution & Policy (resolver, policy, trust, trusted_store, provider, stdio_provider, socket)
  |
Manifest & Package (manifest, package, validation)
  |
M3 Trust & Installation (trust, current_trust, candidate, candidate_preparation, installed, installation_request, installation_trust, installation_plan, installation_execution, installation_driver, installation_publication_*, installation_recovery_*, enablement, launch_profile, conformance, approval)
  |
Operational Scope (operational_scope, file_tools, pdf_tools)
  |
Replay & Persistence (replay, replay_runtime, replay_windows, m3_store, local_anchor, result_anchor)
  |
Process Supervision (child_process)
```

### Module Line Counts (production source)

| Module | Lines | Layer |
|---|---|---|
| `application.rs` | 8,260 | Runtime/Command |
| `configured_runtime.rs` | 2,837 | Runtime |
| `replay_windows.rs` | 2,719 | Persistence |
| `manifest.rs` | 2,455 | Package |
| `execution_environment.rs` | 2,183 | Execution |
| `host_execution.rs` | 2,134 | Execution |
| `runtime_config.rs` | 1,733 | Runtime |
| `policy.rs` | 1,528 | Policy |
| `installed.rs` | 1,377 | Installation |
| `dispatch.rs` | 1,283 | Execution |
| `child_process.rs` | 1,244 | Supervision |
| `installation_execution.rs` | 1,215 | Installation |
| `package.rs` | 1,099 | Package |
| `file_tools.rs` | 1,094 | Operational Scope |
| `trust.rs` | 1,050 | Trust |
| `run_command.rs` | 1,050 | Command |
| `plug_command.rs` | 1,024 | Command |
| `plug_install_command.rs` | 897 | Command |
| `pdf_tools.rs` | 881 | Operational Scope |
| `resolver.rs` | 857 | Capability |
| `validation.rs` | 838 | Package |
| `cli.rs` | 825 | CLI |
| `local_anchor.rs` | 813 | Persistence |
| `stdio_provider.rs` | 786 | Provider |
| `check_command.rs` | 697 | Command |
| `trail_command.rs` | 687 | Command |
| `candidate.rs` | 674 | Installation |
| `trusted_store.rs` | 660 | Trust |
| `launch_profile.rs` | 662 | Installation |
| `replay.rs` | 610 | Persistence |
| `conformance.rs` | 603 | Installation |
| `provider.rs` | 554 | Provider |
| `enablement.rs` | 466 | Installation |
| `installation_plan.rs` | 439 | Installation |
| `engine_stdio.rs` | 415 | Execution |
| `approval.rs` | 389 | Installation |
| `candidate_preparation.rs` | 357 | Installation |
| `result_anchor.rs` | 347 | Persistence |
| `socket.rs` | 329 | Provider |
| `run_input.rs` | 318 | Command |
| `installation_request.rs` | 294 | Installation |
| `event_admission.rs` | 281 | Execution |
| `installation_trust.rs` | 227 | Trust |
| `event_queue.rs` | 238 | Execution |
| `replay_runtime.rs` | 235 | Persistence |
| `m3_store.rs` | 171 | Persistence |
| `outcome.rs` | 151 | Execution |
| `current_trust.rs` | 114 | Trust |
| `operational_scope.rs` | 60 | Scope |
| `lib.rs` | 92 | Root |
| `executor.rs` | 45 | Execution |
| `main.rs` | 3 | Entry |

**Total production source: ~54,082 lines across 53 modules.**

### Provider Binaries

| Binary | Lines | Module |
|---|---|---|
| `tethers-reference-host.exe` | (via lib.rs) | Main host CLI |
| `file_tools_provider.exe` | 124 | `src/bin/file_tools_provider.rs` |
| `m3_fixture_provider.exe` | 151 | `src/bin/m3_fixture_provider.rs` |
| `pdf_tools_provider.exe` | 201 | `src/bin/pdf_tools_provider.rs` |

## OCaml Engine (`tethers-0.1/engine-ocaml/`)

### Dependencies

| Dependency | Version | Purpose |
|---|---|---|
| `ocaml` | 5.5.0 | Compiler runtime |
| `dune` | 3.10+ (locked 3.24.0) | Build system |
| `yojson` | 2.0+ (locked 2.2.2) | JSON parsing/serialization |

### Built Executables

| Binary | Modules | Purpose |
|---|---|---|
| `tethers_engine` | main, tether_parser, tethers_protocol, tethers_evaluator | CLI engine (stdin/stdout JSON) |
| `tethers_mcp_server` | tethers_mcp_main, tethers_mcp_server, tether_parser, tethers_protocol, tethers_evaluator | MCP protocol server |

### Module Graph

```
tether_parser.ml (180 lines)
    |
tethers_protocol.ml (100 lines)
    |
tethers_evaluator.ml (294 lines)
 /                    \
main.ml (8 lines)    tethers_mcp_server.ml (313 lines)
                          |
                      tethers_mcp_main.ml (29 lines)
```

**Total engine source: 924 lines across 6 `.ml` files.**
No `.mli` interface files exist — all modules expose their entire contents.

### Existing Test Integration

No OCaml-native tests exist. Engine testing is done via:
- PowerShell integration test scripts (`tethers-0.1/scripts/test-*.ps1`, 15 scripts)
- Rust host integration tests that invoke the engine as a child process
