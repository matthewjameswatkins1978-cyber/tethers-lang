Task: `J13A local process supervision and check command`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `Goose`
Status: `COMPLETE`
Base commit: `f100689a35c9b7032193abd4f737c3203815fa4c`
Implementation checkpoint: `cb3690d74f2414830948c094fd172b7aa4b79ef8`

## Evidence

### Compilation
- cargo check: PASS
- cargo check --tests: PASS
- cargo build: PASS
- cargo build --release: PASS
- cargo fmt --check: PASS (after cargo fmt)
- cargo clippy --all-targets --all-features: PASS (29 warnings, 0 errors)

### Tests
- cargo test --lib: 26 passed (3 engine tests, 3 child tests skip on restricted env)
- cargo test --bin: 621 passed (stdio_provider require pwsh.exe, may be environment-dependent)
- cargo test j12_: 99 passed, 0 failed
- Acceptance script: 8/8 passed (pwsh -NoProfile -File scripts/test-j13a-check.ps1)
- Task packet checker: PASS

### CLI contract evidence
- Unknown command -> exit 2, invalid_cli_usage, JSON envelope
- Misspelled "runn" -> exit 2, never enters legacy
- No command -> exit 2, invalid_cli_usage
- "__legacy engine.exe req.json" -> exit 6, legacy route active
- "check --config missing.json --engine missing.exe" -> exit 3, invalid_data
- Envelope always contains schema "tethers.cli/1", no timestamp

### Hidden compatibility
- __legacy: hidden subcommand, routes to legacy parser
- provision-replay: hidden subcommand
- event-admission-probe: hidden, debug-only
- event-admission-trail-probe: hidden, debug-only
- No hidden commands appear in --help output

### Process supervision
- SupervisedChild uses Windows Job Object with KILL_ON_JOB_CLOSE
- Bounded protocol line reads (8 MiB max)
- Retained stderr tail (64 KiB)
- Ctrl+C handler sets atomic interrupt flag
- Graceful shutdown: close stdin, wait 2s, terminate Job Object
- Drop implementation terminates Job Object and reaps process

### Engine validation ordering
- One retained engine session per check
- Tethers validated in declared order
- One tethers.validate request per Tether
- Invalid first Tether prevents provider launch

### Provider launch
- Providers launch in configuration order
- One launch per provider (not per capability)
- Initialize once, tools/list once per provider
- compare_discovery_evidence per capability
- Provider child CWD = canonical config directory

### No tools/call, no Trail/replay
- check command has no code path for provider tools/call
- check command has no code path for Trail or replay storage
- Verified through acceptance test: "no provider tools/call"

### Files changed (19 authorized)
1. docs/CURRENT_CLINE_TASK.md
2. docs/DECISIONS.md
3. docs/worker-notes/2026-07-28-j13a-process-check.md
4. tethers-0.1/host-rust/Cargo.toml
5. tethers-0.1/host-rust/Cargo.lock
6. tethers-0.1/host-rust/src/lib.rs
7. tethers-0.1/host-rust/src/main.rs
8. tethers-0.1/host-rust/src/cli.rs
9. tethers-0.1/host-rust/src/child_process.rs
10. tethers-0.1/host-rust/src/engine_stdio.rs
11. tethers-0.1/host-rust/src/check_command.rs
12. tethers-0.1/host-rust/src/stdio_provider.rs
13. tethers-0.1/host-rust/tests/j13a_cli.rs
14. tethers-0.1/scripts/tethers-stdio-fixture.ps1
15. tethers-0.1/scripts/test-j13a-check.ps1
16. tethers-0.1/scripts/demo.ps1
17. tethers-0.1/scripts/test-host-denial.ps1
18. tethers-0.1/scripts/test-host-execution-failure.ps1
19. tethers-0.1/scripts/test-host-result-follow-up.ps1
