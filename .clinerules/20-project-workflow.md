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

## Cline and DeepSeek task construction

ChatGPT/Lucy acts as the task compiler:

Matthew's intention -> architectural contract -> bounded implementation task -> observable proof.

Each implementation task should contain:

1. Goal: one clear outcome.
2. Invariants: the behaviours and architectural boundaries that must remain true.
3. Evidence: observable acceptance criteria and the checks needed to prove them.
4. Boundaries: explicit exclusions, permissions, Git handling, and stopping conditions.

Rules:

- Give Cline the right context, not the maximum context.
- Keep persistent project constraints in repository rules instead of repeating them in every task.
- Point to relevant entry files, then allow Cline to follow references.
- Do not combine implementation, independent audit, Git administration, and a large retrospective report into one confused task.
- Use focused checks during development and proportionate final verification.
- A documentation-only correction does not normally require rebuilding the entire project.
- Treat a ten-minute limit as a runaway brake, not a target that rewards rushed or unverified work.
- If time expires, stop at a coherent, recoverable point and report exact remaining work.
- Reports should contain evidence: changed files, design decision, checks actually run, unresolved discrepancies, and Git status.
- Do not restate the entire task in the report.

Normal workflow:

implementation -> one independent read-only audit -> targeted correction of concrete findings, if required -> focused recheck -> push

Do not restart the original implementation task after a narrow audit finding.
Do not create repeated audit loops without new evidence.

## Frictionless task handoff

Use the project workflow `/tethers-task.md` for ordinary handoffs. Its
instructions live at `.clinerules/workflows/tethers-task.md`, the matching
on-demand skill lives at `.cline/skills/tethers-task/SKILL.md`, and the approved
task contract lives at `docs/CURRENT_CLINE_TASK.md`.

- A `READY` packet authorises only the bounded implementation and verification
  it contains.
- A `PROPOSED` packet is read-only until Matthew explicitly approves it.
- A `COMPLETE` or `BLOCKED` packet must not be silently replaced with invented
  work.
- The skill must verify the packet against the live Git state before editing.
- The packet never grants permission to commit or push unless it says so
  explicitly.

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
