<div align="center">
  <img src="assets/tethers-icon.png" alt="Tethers icon" width="160" />
  <h1>Tethers</h1>
  <p><strong>A small, deterministic language for connecting events to actions across tools, services, and AI.</strong></p>
  <p>
    <a href="https://github.com/matthewjameswatkins1978-cyber/tethers-lang/releases/tag/tethers-portable-v0.2.2">Portable 0.2.2</a>
    ·
    <a href="QUICKSTART.md">Quick start</a>
    ·
    <a href="tethers-0.1/SPEC.md">Language specification</a>
  </p>
</div>

Tethers makes automation inspectable before it becomes consequential. A Tether
describes a chain of intent:

```text
event -> conditions -> actions -> result
```

The runtime keeps planning separate from permission and execution. Effects are
explicit, uncertainty stays visible, and every decision can carry its reason
and provenance.

> **Make things happen. Keep the receipts.**

## See it in 60 seconds

The portable workbench is a local executable. It decides whether a requested
action is `ALLOW`, `ASK`, or `DENY`; it never performs the action itself.

```text
request -> policy match -> decision -> caller acts, asks, or stops
```

```powershell
# After downloading and unpacking the Windows bundle:
.\tethers.exe doctor --json
.\tethers.exe check --action git.status --json
.\tethers.exe check --action git.push --explain
.\tethers.exe check --action git.force_push --json
```

The same commands work on Linux with `./tethers`. The expected exit statuses
are deliberately scriptable: `0` for `ALLOW`, `10` for `ASK`, and `20` for
`DENY`. Configuration and input failures use separate non-zero statuses and
never mean `ALLOW`.

## Download the portable workbench

Download both self-contained x64 packages from the
[Tethers Portable 0.2.2 release](https://github.com/matthewjameswatkins1978-cyber/tethers-lang/releases/tag/tethers-portable-v0.2.2):

| Platform | Package |
| --- | --- |
| Windows x64 | `tethers-portable-0.2.2-windows-x64.zip` |
| Linux x64 (static musl) | `tethers-portable-0.2.2-linux-x64-musl.zip` |

Each bundle includes the executable, policies, schemas, examples, wrappers,
documentation, version metadata, and SHA-256 checksums. The release record and
the checked-in checksum file are in
[`tethers-0.1/portable-rust/RELEASE.md`](tethers-0.1/portable-rust/RELEASE.md)
and [`tethers-0.1/portable-rust/SHA256SUMS-0.2.2`](tethers-0.1/portable-rust/SHA256SUMS-0.2.2).

## Learn Tethers by doing

Start with the [Quick-use manual](QUICKSTART.md). It teaches one concept at a
time using a visible, useful scenario: an agent may inspect a repository, must
ask before pushing, and is denied from force-pushing. You will see the request,
the matching rule, the decision, the exit code, and the evidence before moving
to a custom policy.

For the full storybook version of the architecture, follow
[Bunny & Cookies](docs/BUNNY_AND_COOKIES.md): a button press becomes an event
proposal, the host admits it, a Tether plans an action, a Plug reaches a
provider, and a sensor can later report what really happened. It is the clearest
way to understand why an action result and an observation are not the same
evidence.

Then continue with:

- [Portable CLI contract](tethers-0.1/portable-rust/docs/CLI.md)
- [Portable workbench guide](tethers-0.1/portable-rust/README.md)
- [Language specification](tethers-0.1/SPEC.md)
- [Example Tether](tethers-0.1/examples/record-completed-task.tether)
- [Architecture](docs/architecture/TETHERS_LANTERN_KEEPER_CANONICAL_ARCHITECTURE.md)

The teaching path follows the way Tethers is meant to be used in real
orchestration: make work observable, preserve results, avoid duplicate work,
and route failures or retries explicitly instead of hiding them in an agent
loop.

## What is in this repository?

- `tethers-0.1/` — the active language, protocol, OCaml core, host integration,
  portable Rust façade, examples, and tests.
- `tethers-0.1/portable-rust/` — the small local authority layer for scripts,
  agents, and workbench integrations.
- `docs/` — architecture, operating rules, evidence, project state, and release
  records.
- `assets/tethers-icon.png` — the canonical project icon used on this page and
  in the distribution materials.

The portable façade is intentionally not a replacement for the OCaml Core
evaluator. It is a compatibility boundary: local, deterministic, fail-closed,
and free of server, daemon, database, scheduler, telemetry, or LLM runtime
dependencies.

## Core principles

- **Small language:** the syntax stays readable and deliberately constrained.
- **Explicit authority:** capabilities and policies decide what may happen.
- **Fail closed:** malformed input, ambiguity, unavailable dependencies, and
  unsafe operations cannot silently become permission.
- **Observable execution:** decisions include stable reasons and provenance;
  audit output records metadata without echoing request secrets.
- **Replaceable integrations:** providers and agents translate into Tethers
  requests; they do not smuggle a second policy engine into the system.

## Build and test from source

For the portable Rust workbench:

```powershell
cd tethers-0.1/portable-rust
cargo test --locked
cargo build --release --locked
.\target\release\tethers.exe doctor --json
```

On Windows, use `tethers.exe` in place of `tethers`. The reproducible package
commands and Linux musl build are documented in the
[portable workbench guide](tethers-0.1/portable-rust/README.md).

## Project status

The current public release is Tethers 0.2.2. It hardens the portable 0.2
workbench with the script-friendly `check` command, structured validation and
version commands, deterministic doctor checks, explanations, frozen decision
exit codes, and release parity evidence. The language semantics remain 0.1.

For the live development map, see
[`docs/PROJECT_DASHBOARD.md`](docs/PROJECT_DASHBOARD.md) and
[`docs/CURRENT_GOAL.md`](docs/CURRENT_GOAL.md).

## License and contribution

This repository is under active development. Read the specification and the
project control documents before changing language semantics, policy meaning,
or release artifacts. Small, evidenced changes are easiest to review.
