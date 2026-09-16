# TETHERS R0 - HOST ARCHITECTURE RECOVERY

Task: `TETHERS R0 / Host Architecture Recovery`

Control contract: `1`

Status: `COMPLETE`

Task colour: `Red`

Owner: `Codex`

Route: `Architecture recovery only: preserve existing history, classify the Tethers reference Host versus external application Hosts, create the required architecture and ADR documents, reconcile bounded living documentation, and verify without production-code changes.`

Base commit: `4493e563a81b7c193c0c815db4ac74b81d1c6e59`

Evidence checkpoint: `48f3a54b9362534156a00dfd9d3d8795cd89aae6`

Worker note: `docs/worker-notes/2026-09-16-tethers-r0-host-architecture-recovery.md`

Suggested branch:

`codex/tethers-r0-host-architecture-recovery`

## Objective

Recover and freeze the correct relationship between Tethers Core, capability
contracts, Plugs/providers, the Tethers reference Host, and external Hosts such
as Resolve01. Preserve P0-P4b as historical implementation frozen for recovery;
do not delete, revert, rewrite, squash, or hide it.

## Relevant background and existing behaviour

The current accepted main contains the Tethers reference Host and the merged
P0-P4b Resolve integration line. The reference Host legitimately owns policy,
approvals, replay, durable intent, provider execution, Trail and recovery for
its own use. The existing P0-P4b integration was implemented against a model in
which that reference Host was treated as the mandatory execution owner for
Resolve; R0 must classify that direction as superseded without calling the
implementation invalid or deleting its history. Lantern already exposes genuine
memory/claim service surfaces, but R0 records only the architectural consequence
and makes no Lantern change.

## Required behaviour

1. Establish the current fetched `origin/main` and exact R0 starting SHA.
2. Preserve every valuable currently unreachable commit under explicit archive
   refs, without rewriting history or deleting branches, tags, or worktrees.
3. Classify all currently visible Tethers worktrees and preserve dirty or
   unknown content untouched.
4. Create the durable Host architecture document, execution-ownership ADR,
   Resolve reclassification table, and preservation report.
5. Reconcile narrowly ambiguous living documentation so Core, reference Host,
   external Host, Capability, Plug, Provider, authority, policy, and runtime are
   not confused.
6. Test the recovered model against the three required Host shapes and preserve
   the standalone Tethers and standalone Resolve boundaries.
7. Verify that no production code, OCaml/Core semantics, Tether syntax, Resolve
   implementation, Lantern implementation, or P0-P4b history changed.
8. Leave the task packet and worker note as an evidence-backed R0 closeout.

## Relevant components

- `docs/architecture/TETHERS_HOST_ARCHITECTURE.md`
- `docs/decisions/ADR_HOST_EXECUTION_OWNERSHIP.md`
- `docs/recovery/TETHERS_RESOLVE_RECLASSIFICATION.md`
- `docs/recovery/TETHERS_R0_PRESERVATION.md`
- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-09-16-tethers-r0-host-architecture-recovery.md`
- narrowly relevant living documentation, especially `README.md`
- Git refs under `refs/archive/tethers-r0/`

## Frozen decisions and invariants

Tethers Core owns deterministic semantics and planning. Capability contracts own
stable operation meaning. A Host owns application execution lifecycle, policy,
approvals, recovery, and provider invocation. The Tethers reference Host remains
a first-class supplied Host implementation, but it is not mandatory for an
external application such as Resolve01. Plugs/providers translate capability
contracts into real operations for whichever compatible Host invokes them.

P0-P4b is classified as `HISTORICAL IMPLEMENTATION / FROZEN FOR ARCHITECTURE
RECOVERY`; its original execution-direction contract is `SUPERSEDED`. Existing
machinery may be reusable, but no R0 document may claim the historical work was
invalid. Lantern Plug work remains paused.

## Acceptance criteria

1. Fresh current `origin/main` is the R0 base and is recorded.
2. Every valuable currently unreachable commit is protected by a resolving
   archive ref, and no history or meaningful content is deleted.
3. Worktree and branch preservation evidence records current counts, dirty
   paths, unique unpublished work, open PRs, and the do-not-delete set.
4. The architecture document and ADR state the ownership model explicitly.
5. The reclassification table identifies relevant P0-P4b areas as reusable,
   reference-host, Resolve-host, redesign-required, or historical/document-only.
6. The terminology audit covers the three Host shapes without broad stylistic
   rewriting.
7. The Lantern consequence is recorded without Lantern code or Plug changes.
8. Production code, Core semantics, Tether syntax, Resolve implementation,
   Lantern implementation, and historical Git content remain unchanged.
9. Required verification passes, with any unavailable result recorded honestly.
10. The worker note and task packet contain exact final SHA and evidence.

## Required verification

Run `git status`, `git diff --check`, the task-packet checker, documentation/link
checks, and `just verify`. Verify every preservation ref resolves, every
currently required unreachable commit is protected, no worktree content changed
accidentally, no branch or tag was deleted, and `origin/main` is unchanged
apart from a later R0 documentation merge if accepted. Run `cargo fmt --all --
--check` only because R0 is evidence/documentation-only.

## Forbidden changes

No Rust or OCaml production changes; no Tether syntax or semantic changes; no
Resolve or Lantern implementation changes; no new Plug/provider; no P0-P4b
revert; no history rewrite; no branch, tag, or worktree deletion; no force push;
no direct `main` update; no broad stylistic rewrite; no cleanup of dirty or
unknown worktrees.

## Stop conditions

Stop if preservation would require discarding or overwriting unknown work, if
the Host ownership model remains ambiguous, if a required unreachable commit
cannot be identified or protected, or if the task starts requiring production
implementation or a protected semantic decision. After two materially similar
failed attempts against the same unresolved issue, stop with exact evidence and
the smallest unresolved question.

## Expected pre-existing changes

None in the fresh R0 worktree. Other occupied worktrees are preserved as found;
their dirty paths are not imported into R0.
