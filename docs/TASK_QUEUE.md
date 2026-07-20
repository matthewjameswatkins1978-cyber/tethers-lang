# Task Queue

## Now

- [x] Inspect the initial workspace.
- [x] Inspect the archive contents before extraction.
- [x] Extract `Tethers-0.1-Prototype.tar.gz` without overwriting existing files.
- [x] Add project overview, current goal, decisions, and task queue documents.
- [x] Review the project guidance before verification.
- [x] Inspect complete workspace structure and Git status.
- [x] Confirm there is no unnecessary nested `tethers-0.1/` duplicate.
- [x] Check required tool availability.
- [x] Attempt existing fixture validation.
- [x] Build and test the Rust reference host.
- [x] Record verified state in `docs/CURRENT_GOAL.md`.
- [x] Add native PowerShell fixture validation script.
- [x] Add native PowerShell golden engine-response test script.
- [x] Add native PowerShell full demo script.
- [x] Add opam package metadata for the OCaml engine.
- [x] Check Git availability.
- [x] Install native Windows opam with winget.
- [x] Restart VS Code so the winget-installed `opam` command is visible on PATH.
- [x] Confirm `opam --version` reports `2.5.2`.
- [x] Run `opam init -y`.
- [x] Create a project-local opam switch in `tethers-0.1/engine-ocaml` with
      `ocaml-base-compiler.5.5.0`.
- [x] Install only declared local package dependencies with opam.
- [x] Confirm active OCaml, opam, Dune, and yojson versions.
- [x] Run `scripts/check-fixtures.ps1`.
- [x] Run Rust host unit tests with `cargo test`.
- [x] Build the OCaml engine through `opam exec -- dune build`.
- [x] Run `scripts/test-engine.ps1`.
- [x] Run `scripts/demo.ps1`.
- [x] Capture the first verified full round-trip output.
- [x] Decide that `tethers-0.1/` is the active development tree for the 0.1
      cycle.
- [x] Record the active-tree decision in `docs/DECISIONS.md`.
- [x] Ignore generated build output, the local opam switch, temporary files,
      editor-local files, and the imported archive.
- [x] Re-run the verified baseline checks before committing.
- [x] Run Git whitespace/error checks.
- [x] Stage only the intended project baseline.
- [x] Commit the verified native Windows baseline locally.
- [x] Update `docs/CURRENT_GOAL.md`.
- [x] Update `docs/TASK_QUEUE.md`.
- [x] Add and verify a `greater_than_or_equal` inclusive boundary fixture.
- [x] Configure workspace and Cline guidance to use PowerShell 7 (`pwsh.exe`).
- [x] Review and verify the OCaml parser extraction from `main.ml`.
- [x] Extract JSON/Capability protocol helpers from `main.ml` into
      `engine-ocaml/bin/tethers_protocol.ml`.
- [x] Correlate `missing_fact` errors raised during Condition evaluation while
      preserving known identifiers and accumulated Trail entries.
- [x] Add `docs/CONSTITUTION.md` as the enduring Tethers constitution and wire
      concise references from project guidance.

## Next

- [ ] Decide the first post-baseline implementation milestone.
- [ ] Migrate the remaining contextual evaluation-error paths deliberately,
      including parser, type, unknown Capability, missing argument, unknown
      argument, and missing reference errors where reliable evaluation context
      is available.

## Later

- [ ] Decide the first implementation milestone after the verified 0.1 baseline.
- [ ] Add contribution and development setup notes.
- [ ] Add a release or changelog document once changes begin.
- [ ] Consider a higher-level architecture note for adapters, HQ, AI
      capabilities, and Trail inspection.

## Deferred By Current Scope

- Installing WSL, Docker, Bash, jq, or unrelated OCaml editor tooling.
- Changing parser, evaluator, host, fixtures, scripts, or examples beyond
  definite build defects.
- Adding adapters, package management, scheduling, HQ, or AI integration.
