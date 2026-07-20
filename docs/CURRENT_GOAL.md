# Current Goal

## Goal

Prepare and commit the verified native Windows Tethers 0.1 baseline.

## Immediate Definition Of Done

- Native Windows `opam` is visible after the VS Code restart.
- `opam init -y` has been run.
- `tethers-0.1/engine-ocaml` has a project-local opam switch using
  `ocaml-base-compiler.5.5.0`.
- Only the local package dependencies are installed in that switch.
- Active OCaml, opam, Dune, and yojson versions are recorded.
- Fixture validation, Rust tests, OCaml build, golden engine test, and full demo
  all pass.
- The demo proves the final Trail includes reception, evaluation,
  authorisation, and execution, with final execution status `completed`.
- `tethers-0.1/` is documented as the active 0.1 development tree.
- Generated build output, the local opam switch, temporary files, editor-local
  files, and the imported archive are ignored.
- The verified baseline is committed locally.

## Verified State On 2026-07-20

- Native opam is visible: `opam 2.5.2`.
- `opam init -y` was run. The first invocation exceeded the command timeout, but
  opam finished initialising enough to report root
  `C:\Users\Matmus\AppData\Local\opam` and usable switch operations.
- A project-local switch exists at `tethers-0.1/engine-ocaml` using
  `ocaml-base-compiler.5.5.0`.
- Installed local switch versions:
  - OCaml `5.5.0`
  - opam `2.5.2`
  - Dune `3.24.0`
  - yojson `2.2.2`
- Dependencies installed by the local opam package are Dune and yojson.
- The first switch creation attempt without `--deps-only` installed the compiler
  and dependencies but failed when opam tried to install the local package.
  Cause: Dune package metadata had no installable stanza.
- The switch was then recreated successfully with `--deps-only`, which installed
  only the declared dependency set.
- Compile-only defects fixed:
  - attached the engine executable to the Dune package with `public_name`;
  - removed an unused `Yojson.Safe` open;
  - removed an unused value renderer;
  - marked the parsed Tether title as deliberately read.
- Verification results:
  - `scripts/check-fixtures.ps1`: passed, `JSON fixtures are valid`.
  - `cargo test`: passed, `2 passed; 0 failed`.
  - `opam exec -- dune build`: passed.
  - `scripts/test-engine.ps1`: passed, engine response semantically matches
    `protocol/expected-response.json`.
  - `scripts/demo.ps1`: passed, full round trip completed.

## Round-Trip Evidence

The demo produced a matched Plan requiring `lantern.write`, the Rust host
authorised all required Effects, mock Action `lantern.task.record` completed,
and the final `execution_status` was `completed`.

The successful Trail contains all four stages:

- reception: `event_received`
- evaluation: `anchor_checked`, `condition_checked`, `action_planned`
- authorisation: `plan_authorised`
- execution: `action_started`, `action_completed`

## Near-Term Working Posture

Tethers 0.1 now has a verified native Windows baseline. Future work should keep
the core application-agnostic and make only small, explicit changes against the
documented 0.1 semantics. `tethers-0.1/` is the active development tree for the
0.1 cycle; do not move or rename it while the path-bound local opam switch is in
use.
