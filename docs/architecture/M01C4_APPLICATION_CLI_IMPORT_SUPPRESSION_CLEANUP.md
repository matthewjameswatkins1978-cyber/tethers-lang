# M01C4 Application CLI Import Suppression Cleanup

Status: frozen implementation blueprint

## Purpose

Remove the blanket `#[allow(unused_imports)]` attached to the CLI imports at the top of `tethers-0.1/host-rust/src/application.rs` and replace it with an honest import layout that matches the symbols' real production, debug-only, and test-only uses.

This is a source-hygiene task. It must not alter argument parsing, CLI envelopes, command routing, outcome serialization, debug probes, tests, or runtime behaviour.

## Accepted baseline

M01C3 is accepted on `main` at:

`40539e3084727e5357a448d9fd3cacd6fd08ce2d`

Accepted evidence:

```text
Rust             1.97.1
Cargo tests      926 passing minimum
Clippy messages  118 emitted warnings after M01C3
Cargo.lock       D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB
```

The exact current Clippy count must be captured before editing rather than assumed from the historical note.

OpenCode LSP is not required. It has not proved reliable in this workspace and must not be retried for this task. Exact `rg` reference evidence and the Rust compiler are sufficient.

## Problem

`application.rs` currently contains:

```rust
#[allow(unused_imports)]
use tethers_reference_host::cli::{Cli, CliEnvelope, Command as CliCommand, OutcomeStatus};
```

The attribute suppresses the entire import group across target configurations. It does not explain which names are production dependencies and which are used only inside `#[cfg(test)]` or `#[cfg(debug_assertions)]` code.

A blanket suppression makes future dead imports harder to detect. The import layout should encode the real configuration boundary instead.

## Required analysis

Before editing, run one bounded exact reference search for:

- `Cli`
- `CliEnvelope`
- `CliCommand`
- `OutcomeStatus`

Classify each use in `application.rs` as:

- always-compiled production;
- `#[cfg(debug_assertions)]` only;
- `#[cfg(test)]` only;
- both debug and test;
- not used.

Do not search unrelated repositories or redesign the CLI module.

## Required repair

1. Delete the blanket `#[allow(unused_imports)]` attribute.
2. Keep always-compiled symbols in an ordinary import.
3. Put test-only or debug-only symbols behind the narrowest truthful `#[cfg(...)]` import.
4. Remove a symbol entirely if it has no real use.
5. If all four symbols are genuinely needed in every compiled target and the attribute is redundant, remove only the attribute.
6. Keep aliases unchanged where they are used, including `Command as CliCommand`.
7. Do not qualify dozens of call sites merely to avoid imports unless a single isolated use makes that clearly simpler.

## Forbidden repair shapes

- No replacement `#[allow(...)]` or `#[expect(...)]`.
- No underscore imports, dummy references, `black_box`, unreachable uses, or test-only fake calls.
- No CLI type, enum, parser, command, serialization, exit-code, output, or error-message changes.
- No production function movement between configuration gates.
- No dependency, Cargo.lock, feature, Rust pin, tool configuration, Nextest policy, Just recipe, OCaml, protocol, Plug, Trail, replay, admission, concurrency, or release changes.
- No broad formatting or import reordering outside the target import block unless rustfmt requires it.

## Permitted files

Only:

- `tethers-0.1/host-rust/src/application.rs`;
- `docs/CURRENT_CLINE_TASK.md` for state and checkpoint;
- `docs/worker-notes/2026-08-04-m01c4-application-cli-import-suppression.md`.

Stop before changing another path.

## Tool policy

### Reference discovery

Use one bounded `rg` pass. OpenCode LSP is neither required nor useful here and must not be retried.

### Clippy

Capture machine-readable Clippy output before and after. The repair may leave the total warning count unchanged because the current attribute suppresses the import warning. Acceptance requires no new warning and no blanket import suppression at the target site.

### Focused tests

Run focused CLI/application tests only if the exact reference classification identifies a small meaningful filter. Do not invent a focused test filter merely for ceremony.

### Final authority

Run `just verify` once after the edit. It is the full Cargo authority for this task.

Do not run full Nextest, cargo-deny, cargo-machete, `just verify-agent`, OCaml tests, LSP diagnostics, or unrelated scripts. None can add relevant evidence to a one-import-block source cleanup.

## Warning accounting

Record before and after:

- total emitted warning messages;
- every warning whose primary span is the `application.rs` import block;
- any new or changed warning outside that block;
- all suppression attributes removed or added in the changed lines.

Acceptance requires:

- the target `#[allow(unused_imports)]` is absent;
- no replacement suppression is added;
- ordinary locked Clippy exits zero;
- no new warning is introduced;
- total emitted warnings are unchanged or lower;
- import configuration matches actual symbol uses.

## Behavioural invariants

- Normal CLI parsing and routing are byte-for-byte equivalent in observable behaviour.
- Debug-only probes remain available in debug builds exactly as before.
- Test-only code remains test-only.
- CLI envelope and outcome JSON remain unchanged.
- No test disappears.
- Cargo.lock remains byte-identical.

## Verification floor

Run only evidence-bearing checks:

1. task packet checker;
2. bounded exact `rg` reference classification;
3. Clippy JSON before edit;
4. rustfmt check after edit;
5. optional focused application/CLI tests only when a truthful narrow filter exists;
6. Clippy JSON after edit;
7. `just verify` once;
8. Cargo.lock hash;
9. `git diff --check` and final changed-file check.

## Completion evidence

The worker note must record:

- exact use classification for all four imported symbols;
- exact import layout chosen and why;
- before/after Clippy table;
- whether a focused test filter was useful or honestly skipped;
- final Cargo total and failures;
- unchanged Cargo.lock hash;
- exact changed files;
- confirmation that no CLI or runtime behaviour changed;
- smallest next action.
