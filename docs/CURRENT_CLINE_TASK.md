# Check Command Provider Server-Name Bugfix

Control contract: `1`

Status: `ACCEPTED`

Task colour: `Red`

Owner: `Check Provider Server-Name Bugfix Agent`

Route: `Accepted by Lucy — awaiting integration to main`

Base commit: `7c9f846cf5c7681a919f321faf42657c386d99ca`

Implementation checkpoint: `ed786efbd156bbb4850a5c95077cae226eac5dcb`

Accepted branch tip: `14b2c65d1a830b4fc0a7a893ee3e72b684b09740`

Worker note: `docs/worker-notes/2026-08-15-check-server-name-bugfix.md`

Updated: 2026-08-15

## Objective

Fix the Tethers `check` command so MCP provider initialization validates the provider against the trusted capability manifest binding's `server_name`, not against the provider's configured identity. Add a regression test proving provider identity and MCP server name may legitimately differ.

## Accepted result

Lucy independently inspected the pushed branch and accepted the fix.

`check_providers` now derives the expected MCP server name from trusted prepared capability manifest data, mirroring the normal host run path, while leaving configured provider identity separate and unchanged.

The negative trust behaviour remains intact: a provider reporting a server name that does not match the trusted manifest binding still fails initialization.

## Acceptance evidence

- Focused positive regression: PASS.
- Focused wrong-server-name negative regression: PASS.
- Full Rust suite: 1550 passed, 0 failed, 2 ignored.
- `cargo fmt`: PASS.
- `cargo check`: PASS.
- `cargo check --locked`: PASS.
- `cargo check --all-targets --all-features`: PASS.
- `git diff --check`: PASS.
- No C5 salvage implementation was pulled into the fix branch.
- No unrelated production semantic change was found.

## Frozen decisions and invariants

- Provider identity and MCP server name remain distinct concepts.
- MCP server-name validation is derived from trusted manifest binding evidence.
- Do not weaken or remove server-name validation.
- Do not accept arbitrary reported server names.
- Do not invent a second server-name rule; the `check` and normal run paths should remain consistent.

## Current state

This task is finished and accepted.

No additional bugfix work is authorised under this packet.

The accepted integration chain has not yet been merged to `main`; that remains a separate Matthew-authorised action.
