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

PowerShell 7 (`pwsh.exe`) is the required shell for Tethers automation and Cline
tasks. Windows PowerShell 5.1 (`powershell.exe`) is not a project requirement.

## Fixture Contract Follow-Up

The evaluation fixture contract now covers the canonical happy path, Anchor
mismatch, false Condition, and the current sparse missing-Fact error response.
The missing-Fact case intentionally documents current behaviour only; a
correlated evaluation-error envelope remains a queued design task before the
error contract expands.

The fixture contract also now covers the inclusive boundary for
`greater_than_or_equal`: `task.changed_files = 3` with
`greater_than_or_equal 3` evaluates to `matched` and plans
`lantern.task.record`.

The OCaml Tether parser has been mechanically extracted from `main.ml` into
`engine-ocaml/bin/tether_parser.ml` without changing the verified fixture or demo
behaviour.

The JSON/Capability protocol helpers have been mechanically extracted from
`main.ml` into `engine-ocaml/bin/tethers_protocol.ml` without changing behaviour.
Module dependency chain: `main.ml` → `Tethers_protocol` → `Tether_parser`.
All seven fixture cases, the demo round-trip, and fixture validation continue to
pass.

The missing-Fact fixture now uses the correlated evaluation-error envelope for
`missing_fact` raised during Condition evaluation. The canonical missing-Fact
request still includes `project.type` and omits `task.changed_files`, so the
error Trail preserves the matched first Condition before appending a single
`condition_failed` entry for `Missing Fact: task.changed_files`. Other
contextual evaluation errors remain a separate migration task.

`docs/CONSTITUTION.md` now records the enduring Tethers design principles.
Project guidance references it as the constitutional authority, while
`tethers-0.1/SPEC.md` remains the authority for current precise 0.1 language and
protocol semantics.

The Condition type-error fixture now uses the correlated evaluation-error
envelope for `type_error` raised during Condition evaluation. The fixture keeps
the canonical Tether source and changes `project.type` to integer `7`, so the
engine preserves reception and matched Anchor Trail entries before appending one
`condition_failed` entry at sequence 3.

`docs/OCAML_GUIDE_FOR_AGENTS.md` now records the verified OCaml 5.5.0 local
toolchain, current engine module structure, project OCaml subset, Yojson usage,
and official source links for AI coding agents. `.clinerules/30-ocaml.md`
points Cline to the guide for OCaml implementation tasks without duplicating it.

`docs/TETHERS_LUCY_NOTES.md` now preserves Lucy's compact project-orientation
notes. AGENTS.md references it as optional orientation, not as an authoritative
source for semantics.

The unknown-Capability fixture now uses the correlated error envelope for
`unknown_capability` raised during Action planning. The fixture copies the
canonical happy-path Tether and changes the Action capability to
`lantern.task.save` (not supplied in capabilities), so the engine preserves
the full evaluation Trail (reception, Anchor match, both matched Conditions)
before appending one `action_planning_failed` entry at sequence 5.

The missing-Action-argument fixture now uses the correlated error envelope for
`missing_argument` raised during Action planning. The fixture copies the
canonical happy-path Tether and removes the required `task` argument from the
Action, so the engine preserves the full evaluation Trail before appending one
`action_planning_failed` entry at sequence 5.

The unknown-Action-argument fixture now uses the correlated error envelope for
`unknown_argument` raised during Action planning. The fixture copies the
canonical happy-path Tether and adds an undeclared `extra` argument to the
Action, so the engine preserves the full evaluation Trail before appending one
`action_planning_failed` entry at sequence 5.

The Action-type-error fixture now uses the correlated error envelope for
`type_error` raised during Action planning. The fixture copies the canonical
happy-path Tether and changes the `task` argument from a string to integer
`42` (capability declares `task` as `string`), so the engine preserves the
full evaluation Trail before appending one `action_planning_failed` entry at
sequence 5.

The missing-Action-reference fixture now uses the correlated error envelope
for `missing_reference` raised during Action planning. The fixture copies the
canonical happy-path Tether and removes `task` from the event data while the
Action references `anchor.task`, so the engine preserves the full evaluation
Trail before appending one `action_planning_failed` entry at sequence 5.

The Tether parse-error fixture now documents the existing minimal
pre-evaluation error contract for `parse_error`. The fixture changes the Tether
opening keyword from `tether` to `bad`; the engine returns only
`protocol_version`, `status`, and `error` — no evaluation identifiers, plan,
or Trail. `docs/DECISIONS.md` records the design decision that parse errors
remain minimal because evaluation has not begun and the two-category error
model (minimal pre-evaluation, fully correlated evaluation/planning) is
preferred over partial correlation.

The duplicate-Action-argument fixture enforces that each argument name may
appear at most once per Action. The Tether source duplicates `task` with
a discernibly different value; the parser rejects it as a `parse_error` before
evaluation begins. Different Actions may independently use the same argument
name.

The reused-argument-across-actions fixture proves that the same argument
name (`task`) may be used once in each of two separate
`lantern.task.record` Actions within a single Tether. Both Actions resolve
`anchor.task` independently, both appear in the Plan in source order with
consecutive `action_id` values (`action_1`, `action_2`), and the Trail
contains two `action_planned` entries at sequential positions 5 and 6.
`required_effects` remains deduplicated (`["lantern.write"]`). Duplicate
argument names inside a single Action remain rejected; reuse across separate
Actions is valid.

The duplicate-capability fixture enforces that every Capability name must
be unique within a request. The fixture duplicates the `lantern.task.record`
schema identically; the engine rejects it as a minimal pre-evaluation
`invalid_capability` error with no evaluation identifiers, plan, or Trail.
Capability names are compared without regard to version because Actions
address Capabilities by name. The uniqueness check runs after Capability
declarations are parsed but before evaluation begins, preserving original
order for valid requests without changing Action lookup behaviour.
