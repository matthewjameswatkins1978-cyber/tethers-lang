# Project Workflow

Roles:

- ChatGPT/Lucy: architecture, product judgement, task selection, and semantic decisions.
- Cline/DeepSeek: bounded implementation, fixtures, tests, mechanical refactoring, PowerShell, and documentation.
- Codex: Git, environment setup, independent verification, difficult diagnosis, and recovery.
- Matthew: final authority for direction, installation, publishing, and irreversible actions.

Rules:

- Cline and Codex must not edit the repository simultaneously.
- Cline normally does not commit. Codex reviews and commits meaningful checkpoints.
- Cline gets one focused correction when confused; it must not thrash.
- Environment, Git, cross-language, and difficult recovery work goes to Codex.
- Architecture or language changes return to Lucy before implementation.
- Tests and fixtures determine correctness—not model confidence.

Current platform:

- Windows
- Rust host
- Native OCaml through project-local opam switch
- PowerShell 7 (`pwsh.exe`) is the required automation shell
- Unix scripts remain for portability

Shell rules:

- Use `pwsh.exe` for Tethers automation and Cline terminal commands.
- `powershell.exe` is Windows PowerShell 5.1 and is not a project requirement.
- Do not spend implementation time making scripts compatible with Windows
  PowerShell 5.1.

Important paths:

- Specification: tethers-0.1/SPEC.md
- OCaml engine: tethers-0.1/engine-ocaml/
- Rust host: tethers-0.1/host-rust/
- Protocol fixtures: tethers-0.1/protocol/
- Scripts: tethers-0.1/scripts/
- Project state: docs/CURRENT_GOAL.md and docs/TASK_QUEUE.md
- Decisions: docs/DECISIONS.md

Do not move `tethers-0.1/engine-ocaml/`; its local opam switch is path-bound.
