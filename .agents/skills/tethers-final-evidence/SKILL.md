---
name: tethers-final-evidence
description: Verify the truth and completeness of a Tethers task before final commit, push, acceptance, or PASS reporting. Use for final evidence reviews, packet closure, acceptance checks, and comparisons between code, tests, worker notes, commit SHAs, warnings, and Git state.
---

# Tethers Final Evidence

Apply this skill as the final evidence review for a Tethers task.

Its purpose is accurate proof, not ceremony. It does not expand the task's scope, grant additional permission, or redesign accepted behaviour.

Evidence review is read-only by default. Modify files, delete generated files, commit, or push only when the current task explicitly authorises those actions. After any authorised correction, repeat the relevant checks.

## 1. Freeze the contract

Read the current task packet, worker note, project guardrails, and changed-file list.

Record:

- required worktree;
- required branch;
- accepted base commit;
- expected local and remote HEAD;
- authorised files;
- required checks;
- warning baselines;
- required final marker.

Do not silently resolve contradictory or missing requirements. Stop with `NEEDS REVIEW` when the contract cannot be determined from repository evidence.

## 2. Verify actual Git state

Check the actual repository rather than relying on a previous report.

At minimum inspect:

```powershell
Get-Location
git branch --show-current
git rev-parse HEAD
git status --porcelain=v1 --untracked-files=all
git diff --name-status
git diff --check
````

When a remote branch is part of the task, also inspect its SHA and ahead/behind state.

Never describe a worktree as clean when porcelain contains modified, staged, deleted, ignored-as-relevant, or untracked task-generated files.

Validate every recorded commit SHA with:

```powershell
git cat-file -e "<SHA>^{commit}"
```

A similar-looking or abbreviated SHA is not proof that the recorded SHA is correct.

## 3. Build a claim-to-evidence ledger

For every material claim in the final report or worker note, identify its actual support.

Classify each claim as one of:

* directly verified;
* verified through a named seam;
* reasonable inference;
* not verified.

Do not promote seam evidence or inference into direct production proof.

Examples:

* A callback not being entered proves code inside that callback was unreachable. It does not prove the internal state of every downstream component was directly inspected.
* A provider call count remaining unchanged after direct gate rejection is a dispatch-seam check. It is not automatically proof that the production queue drain reached that dispatch seam.
* A test-only copy of a production loop does not prove the real production loop.
* Applying an outcome to a response is not proven merely by inspecting the outcome object. Assert the mutated response itself.

Test names, comments, worker notes, and reports must describe only what the test actually observes.

## 4. Verify the production boundary

Determine which tests exercise:

* the real production path;
* a shared production helper;
* a lower-level unit boundary;
* a dispatch or storage seam;
* a test-only substitute.

Do not claim all focused tests use the production boundary unless every focused test actually does.

For rejection, queue, response, or safety behaviour, verify the relevant observable outcomes, such as:

* rejected work never enters evaluation;
* later siblings stop when required;
* already completed work remains visible;
* no retry or reinsertion occurs;
* clean routes omit rejection fields;
* exact rejection JSON is correct;
* generation and identity values are correct;
* existing response fields survive mutation;
* queue order remains FIFO when children are appended.

Use only the outcomes required by the current task.

## 5. Run the required verification

Run every check required by the task packet.

Report:

* exact focused-test count;
* exact full-suite count;
* exact integration-script results;
* exact warning totals;
* baseline warnings versus new warnings;
* checks not run and the reason.

Never write "all checks passed" when some required checks were skipped, unavailable, inferred from an earlier run, or replaced by a narrower check.

Do not introduce unrelated checks merely to make the report look larger.

## 6. Reconcile documentation

Read the complete worker note and relevant task packet after the code is final.

Check for:

* stale descriptions of removed helpers or old architecture;
* old test names;
* incorrect test counts;
* mistyped commit SHAs;
* claims stronger than the tests;
* contradictory warning reports;
* incorrect changed-file lists;
* future actions accidentally bundled into the completed packet;
* statements that all tests exercise one boundary when some use other seams.

Preserve useful review history, but make the current factual record internally consistent.

## 7. Inspect text and generated residue

Run `git diff --check`.

Inspect changed text files for unexpected control characters, excluding ordinary TAB, CR, and LF where appropriate.

List untracked files using:

```powershell
git status --porcelain=v1 --untracked-files=all
```

Do not delete unfamiliar untracked content. Establish provenance first. Delete it only when the current task authorises cleanup and repository evidence shows it was generated by the task.

## 8. Issue the verdict

Return `PASS` only when:

* the task contract is satisfied;
* required checks passed;
* warning deltas meet the contract;
* code, tests and documentation agree;
* recorded SHAs resolve;
* changed paths remain within scope;
* final Git state meets the task;
* the report makes no claim stronger than its evidence.

Return `FAIL` for a demonstrated acceptance failure.

Return `NEEDS REVIEW` when evidence is missing, contradictory, unavailable, or outside the authorised scope to correct.

The final report must include:

* implementation and documentation SHAs where applicable;
* exact changed files;
* test and warning counts;
* production and seam boundaries actually exercised;
* checks not run;
* task-packet checker result;
* local and remote SHA;
* ahead/behind state;
* complete porcelain result;
* whether force-push occurred;
* whether `main` was touched;
* the exact final marker required by the task.

