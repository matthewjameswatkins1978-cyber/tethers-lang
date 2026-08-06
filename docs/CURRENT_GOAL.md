# Current Goal

Updated: 2026-08-06

## Goal

Begin the Tethers Foundation Pass only as a bounded evidence programme now that
J24K and J24L are independently accepted. The pass adds no product capability;
it raises the existing host and engine to a consistent, evidence-backed
engineering standard.

The provisional pre-pass baseline is
`24428139807cac0adeb0b62264547e61ca809d16`, revalidated as `origin/main` on
2026-08-06. Its use remains provisional until F1 captures it directly from Git.

## Active increment

F1 — Baseline and debt inventory — is prepared as a documentation-and-evidence
task. It measures the accepted baseline, records compatibility and persistence
facts, and classifies debt. It does not repair production code, alter tests, or
change public behaviour.

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
- J24L/OpenCode lessons: `docs/working-guides/DEEPSEEK_PRO_OPENCODE_JOB_PLAYBOOK.md`
- Enduring principles: `docs/CONSTITUTION.md`
- Language semantics: `tethers-0.1/SPEC.md`
