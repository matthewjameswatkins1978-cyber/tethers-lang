# Current Goal

Updated: 2026-08-07

## Goal

Continue the Tethers Foundation Pass through F2, repairing only demonstrated
operational-correctness defects from the F1 baseline evidence. The pass adds
no product capability; it raises the existing host and engine to a consistent,
evidence-backed engineering standard.

The accepted main is `f295daa288f4d3dc48181888d6655df798675033`.

## Active increment

F2 — Operational correctness defects — repairs two F1-confirmed defects:
truthful live stderr capture in `child_process.rs` and nondeterministic M3
handle allow-list test behaviour. The work is one review gate with two serial
subpackages (F2a and F2b) on branch `foundation/f2-operational-correctness`.

## Foundation Pass boundaries

- No language-semantic, Plug-capability, or new-CLI work.
- Preserve external JSON, exit codes, Trail shape, replay digests, and recovery
  behaviour unless a later package explicitly authorises a migration.
- Compatibility fixtures are literal committed evidence and are not generated
  by the implementation being tested.
- Every package reports each required command as PASS, FAIL, or NOT RUN; a
  mandatory NOT RUN blocks COMPLETE.
- Final package verification is serial after the last code or test change.

## Authoritative references

- Foundation Pass plan: `docs/architecture/TETHERS_FOUNDATION_PASS.md`
- Current task: `docs/CURRENT_CLINE_TASK.md`
- F1 worker note: `docs/worker-notes/2026-08-06-f1-baseline.md`
- J24L/OpenCode lessons: `docs/working-guides/DEEPSEEK_PRO_OPENCODE_JOB_PLAYBOOK.md`
- Enduring principles: `docs/CONSTITUTION.md`
- Language semantics: `tethers-0.1/SPEC.md`
