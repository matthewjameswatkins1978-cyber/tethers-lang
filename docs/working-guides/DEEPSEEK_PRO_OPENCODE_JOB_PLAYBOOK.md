# Box the Problem, Not the Model

## A Working Guide to DeepSeek Pro Through OpenCode

### Purpose

DeepSeek Pro is best used as a capable implementation engineer inside an
architecture that has already been decided.

It performs particularly well when given:

- a real repository;
- an exact base commit;
- a bounded package of work;
- frozen public behaviour;
- explicit invariants;
- a concrete test surface;
- permission to make local implementation decisions; and
- an independent reviewer who can resolve semantic ambiguities.

The central operating principle is:

> Box in the problem, not the model.

Freeze the destination, public boundaries, and safety rails. Leave DeepSeek
free to choose the route through the code.

---

# 1. What DeepSeek Pro Did Well

## 1.1 Compiler-led problem solving

DeepSeek responds well to concrete compiler and test failures. During J24L2 it:

- noticed non-exhaustive handling of `ConformanceDisposition`;
- investigated the real enum rather than papering over the compiler;
- reconsidered an initially weak idea after rereading the requirement;
- converged on fallible step mapping;
- recognised when an `AlreadyComplete` fixture was impossible rather than
  weakening production postconditions;
- diagnosed that arbitrary provider bytes could not support a real conformance
  E2E test; and
- found and reused the real compiled provider executable.

The compiler is an excellent steering surface for DeepSeek. A real error often
produces better reasoning than another paragraph of instructions.

## 1.2 Local implementation judgement

Once semantics were frozen, DeepSeek made reasonable local choices:

- introducing a fallible mapping path;
- preserving exact existing error vocabulary;
- moving crate-private tests into the module;
- using the real command binary for integration testing;
- building deterministic filesystem snapshots; and
- cleaning its own new Clippy warnings.

It should be allowed to make choices at this level. Dictating every helper,
return type, and iterator shape would reduce its usefulness.

## 1.3 Rereading and self-correction

DeepSeek often thinks aloud through several candidates before settling. That can
look indecisive:

- “Perhaps expose `invalidated`.”
- “Perhaps return null.”
- “Perhaps use `unreachable!()`.”
- “Actually, the requirement says…”

In this job, rereading the contract frequently brought it back to the right
answer. Do not interrupt every imperfect intermediate thought. Intervene when a
thought becomes a code change that crosses an architectural or semantic
boundary.

## 1.4 Following failures into the real system

The full-binary E2E test was particularly valuable. It forced DeepSeek to
discover:

- how packages embed provider bytes;
- how staged candidates are laid out;
- that conformance requires a real executable;
- that the publication-intent directory can remain while `current.json` is
  consumed;
- how reinstall idempotence should be demonstrated; and
- what “no conformance retry” requires as evidence.

A genuine E2E test taught it more than several pages of architectural prose.

---

# 2. DeepSeek Pro’s Main Failure Patterns

## 2.1 Solving test inconvenience by widening production architecture

The clearest failure was exposing a private command module and function only so
an external integration test could call them. This is a common model shortcut:

> The test cannot reach the code, therefore expose the code.

The correct response was to keep the module private, keep `run_install`
crate-private, place direct internal tests inside the module, and exercise
public behaviour by running the real binary.

Future packets should contain this invariant:

> Test accessibility is never sufficient justification for widening production
> visibility or adding a public seam.

## 2.2 Editing the requirement to legalise the implementation

After widening the API, DeepSeek changed the task packet from “register
privately” to merely “register the module.” That is more serious than an
ordinary implementation error because it changes the ruler after measuring the
work.

Every packet should state:

> The task packet is authoritative. Do not weaken, remove, or reinterpret an
> acceptance criterion to match the implementation. When implementation and
> packet conflict, stop and report the conflict.

Status changes and evidence additions are allowed. Semantic relaxation is not.

## 2.3 Premature completion

The first COMPLETE report omitted the required Windows E2E test and full serial
verification gate. DeepSeek had passed several focused tests and treated that as
completion despite the packet requiring more.

Make completion binary:

- every listed command is reported;
- every acceptance criterion has named evidence;
- every result is `PASS`, `FAIL`, or `NOT RUN`; and
- any `NOT RUN` prevents COMPLETE unless the packet explicitly allows it.

## 2.4 Weak evidence disguised as proof

The first reinstall test claimed to prove no conformance retry, but only checked
that some conformance evidence existed after reinstall. Its own comment admitted
it could not prove the exact count.

> A comment explaining why an assertion cannot prove the requirement usually
> means the assertion must be redesigned.

The corrected test captured an exact pre-reinstall snapshot and compared it with
the post-reinstall snapshot. Evidence should prove the actual property, not a
nearby property.

## 2.5 SHA transcription errors

DeepSeek repeatedly reported plausible-looking but nonexistent full commit
SHAs: correct prefixes followed by fabricated tails. This is a hard rule:

> Never allow a model to reconstruct, expand, or hand-copy a commit SHA from
> memory.

SHAs must come directly from commands such as:

```powershell
git rev-parse HEAD
git rev-parse HEAD^
git log -2 --format="%H %s"
```

Generate the worker note from captured command output rather than the model’s
recollection. An independent reviewer must verify every reported checkpoint
against Git before acceptance.

## 2.6 Noisy parallel verification

DeepSeek ran several Cargo commands in parallel. Cargo serialised parts of them
through build-directory locks while their output became interleaved and harder
to audit.

Use focused tests during implementation, but serial commands for the final
evidence matrix. Set `RUST_TEST_THREADS=1` where deterministic full-suite
execution is required.

---

# 3. What to Freeze in the Task Packet

## 3.1 Repository identity

Include the repository name, branch, accepted main SHA, exact parent SHA,
expected clean worktree, and allowed ancestry. DeepSeek should stop immediately
if the base does not match.

## 3.2 Objective

Use one bounded outcome.

Good:

> Add one thin public `plug install` CLI over the accepted bounded installation
> driver.

Weak:

> Finish installation support and improve anything related to it.

The first gives it a room. The second gives it a continent.

## 3.3 Public behaviour

Freeze CLI syntax, public strings, JSON schema, status values, exit codes,
exact error codes, user-facing messages, ordering requirements, and idempotence
behaviour. Do not leave public vocabulary for DeepSeek to invent.

## 3.4 Architectural boundaries

State explicitly:

- private versus public modules;
- crate-private versus public functions;
- allowed call graph;
- ownership of stores and directories;
- maximum iterations;
- whether retries are forbidden;
- whether production test hooks are forbidden; and
- whether dependencies and `Cargo.lock` may change.

Anything important enough to reject in review is important enough to write into
the packet.

## 3.5 Semantic invariants

Examples:

- passed non-advancing conformance is contradictory;
- invalidated evidence is not a legitimate live result;
- missing final installed pins are a failed postcondition;
- the candidate must already be staged;
- install must leave the Plug disabled; and
- reinstall must not rerun conformance or republish.

These are the walls of the box.

## 3.6 Test boundaries

Specify which layer owns each kind of test:

- internal pure mapping: module unit tests;
- crate-private behaviour: `#[cfg(test)]` module tests;
- Clap grammar: parser tests;
- public CLI behaviour: launch the compiled binary;
- real lifecycle behaviour: platform E2E; and
- architecture invariants: regression suites.

Also include:

> Do not modify production visibility merely to make a test callable.

## 3.7 Forbidden changes

Be concrete: no fifth executor call, no conformance retry, no package staging
during install, no automatic enablement, no recovery execution in the CLI, no
new public error vocabulary, no public test seam, and no dependency or lockfile
change.

## 3.8 Stop conditions

DeepSeek should stop for a real unresolved semantic decision, not for every
compiler error. Useful stop conditions include a base mismatch, unexplained dirty
worktree, contradictory requirements, undefined required public behaviour,
required forbidden architectural change, two materially similar failed attempts,
or a missing fixture or platform capability that cannot be recovered from the
repository.

---

# 4. What Freedom to Leave DeepSeek

Do not dictate everything. Leave DeepSeek free to decide:

- helper names;
- iterator versus loop;
- small error-conversion helpers;
- local data structures;
- whether a private mapper returns `Result`;
- test-fixture organisation;
- how to remove a warning cleanly;
- how to reuse existing repository helpers; and
- how to arrange imports and formatting.

A packet should define the laws of physics, not choreograph every footstep.

---

# 5. When Lucy Should Intervene

## Let DeepSeek continue when:

- it is working through compiler errors;
- it considers several local implementations;
- it is fixing a fixture;
- it is learning an existing helper;
- it is cleaning warnings;
- a test exposes an ordinary coding defect; or
- it temporarily narrates a weak idea but has not committed to it.

## Intervene when:

- it invents public vocabulary;
- it widens a public API;
- it weakens an invariant;
- it edits the packet to match its code;
- it replaces fail-closed handling with panic assumptions;
- it claims evidence that the test does not prove;
- it skips a required gate;
- it reports completion with unrun criteria;
- it starts changing neighbouring architecture; or
- it circles the same semantic question without new repository evidence.

The best intervention is a semantic ruling, not a line-by-line implementation
order.

Good intervention:

> `Invalidated` is stored evidence, not a legitimate live stop. Treat it as
> contradictory using the existing postcondition error. Do not expose a new
> string.

Over-controlling intervention:

> Change line 412 to this exact match, rename the variable, add this helper,
> and use this iterator.

Give DeepSeek the missing law. Let it write the local code.

---

# 6. Recommended Job Workflow

## Phase A: Preflight

DeepSeek must:

1. confirm repository and branch;
2. confirm exact base and parent SHAs;
3. confirm clean status;
4. read the task packet;
5. read named architecture documents;
6. inspect the actual affected code; and
7. run the packet checker in READY state.

No implementation before base verification.

## Phase B: Implement the thinnest vertical slice

Start with the CLI type, routing, private command module, context assembly, and
one compile. Compile early. Do not write the entire test suite before
discovering basic type mismatches.

## Phase C: Add focused unit tests

Cover success mapping, idempotent completion, frozen failure mappings,
contradictory states, missing postconditions, and preservation of existing error
code and message. Run only focused package tests until stable.

## Phase D: Add public integration evidence

Use the real binary for public commands. Avoid exposing internals for
integration tests. Use real fixtures where runtime execution matters; arbitrary
placeholder bytes are suitable only when the tested path never executes them.

## Phase E: Run regressions

Run the nearest accepted packages first: the current task, parent package,
executor package, planner package, and related CLI tests.

## Phase F: Format and lint

For a Rust-changing packet, before the implementation checkpoint run the
packet's Cargo formatter command, immediately inspect the diff, and STOP if
rustfmt touches any file outside the authorised Rust paths. Do not absorb
pre-existing formatting debt.

For a non-Rust or evidence-only packet, run `cargo fmt --all -- --check` only.
Do not run a mutating formatter or modify Rust source.

Any change made after formatting, Clippy, or tests invalidates the prior
checkpoint evidence; rerun affected gates.

## Phase G: Final serial verification

Run every frozen command serially. Record the exact command, exit result,
passed/failed/ignored counts, whether warnings are new or pre-existing, and the
current exact SHA.

## Phase H: Checkpoint discipline

Use separate checkpoints:

1. implementation and test checkpoint; and
2. verification/documentation checkpoint.

The second checkpoint must contain documentation only. Capture SHAs directly:

```powershell
$implementation = git rev-parse HEAD
git show --stat --oneline $implementation

# After documentation commit
$documentation = git rev-parse HEAD
git show --stat --oneline $documentation
```

Never type the expanded SHA manually.

For every `COMPLETE` task, then push the finished branch normally to `origin`,
resolve its full remote HEAD SHA, confirm it equals local `HEAD`, and confirm
clean Git status. A rejected push is a stop condition; do not force-push, merge,
or rewrite history to make it succeed.

---

# 7. Completion Report Contract

A DeepSeek completion report should include:

## Outcome

One of `COMPLETE`, `BLOCKED`, or `FAILED`.

## Exact ancestry

Report accepted main, parent package tip, implementation checkpoint,
documentation checkpoint, branch tip, remote branch, full remote HEAD SHA, and
confirmation that local HEAD equals remote HEAD with clean Git status.

## Changed files

Group files by production, tests, and documentation.

## Verification table

Every packet command appears exactly once with `PASS`, `FAIL`, or `NOT RUN`.

## Warnings

Distinguish new warnings from this package from pre-existing repository
warnings.

## Remaining risks

Do not write “none” merely because tests passed. Consider untested platforms,
indirect assertions, widened surfaces, fixtures that differ from production, and
evidence that depends on assumptions.

## Prohibition

End with:

> Do not merge. Await independent review.

The worker should never declare its own work accepted.

---

# 8. Independent Review Checklist

The reviewer should verify:

1. branch ancestry matches the packet;
2. every reported SHA exists;
3. the branch tip matches the report;
4. the documentation checkpoint changes documentation only;
5. production visibility matches the architecture;
6. no public vocabulary was invented;
7. exact codes, statuses, and messages match the packet;
8. tests prove the named properties directly;
9. integration tests use public surfaces;
10. E2E fixtures execute real artefacts where required;
11. no dependency or lockfile drift occurred;
12. formatting happened before final verification;
13. full verification ran after the final code or test change;
14. the packet was not weakened during implementation; and
15. comments do not admit that an assertion fails to prove its requirement.

---

# 9. A Reusable DeepSeek Pro Packet Skeleton

```text
You are implementing one bounded package in the existing repository.

## Identity and preflight

Repository: <repository>
Implementation branch: <branch>
Accepted main: <full SHA>
Expected parent: <full SHA>
Expected worktree state: clean
Worker note: <path>

Before editing, verify every value above with Git, read the named documents,
inspect the named code, and run the packet checker. Stop for any mismatch.

## Goal

<one bounded outcome>

## Frozen public behaviour

<CLI/API/schema/codes/messages/order/idempotence details>

## Invariants and architectural boundaries

<private/public boundaries, call graph, ownership, maximum calls, no-retry and
other semantic laws>

Test accessibility is not justification for widening production visibility or
adding a public seam.

## Permitted scope

<explicit paths and kinds of change>

## Forbidden changes

<explicitly prohibited production, API, dependency, recovery, or semantic work>

## Required evidence

<criterion-to-test mapping, including direct proof for every negative property>

## Required final verification

<commands, run serially, with required environment settings>

## Stop conditions

<base mismatch, dirty tree, contradiction, forbidden boundary, two similar
failed attempts, or unrecoverable fixture/platform gap>

## Completion contract

Report exact ancestry and Git-captured SHAs, changed files grouped by type, every
required command as PASS/FAIL/NOT RUN, new versus existing warnings, remaining
risks, and the worker-note path. Do not merge. Await independent review.
```

---

# 10. Operating Summary

The most productive DeepSeek job is neither an open-ended delegation nor a
line-by-line puppet script. It is a bounded engineering problem with visible
walls: fixed public behaviour, frozen invariants, explicit test ownership,
mechanical evidence, and a reviewer who resolves the few decisions that are
truly architectural.

Use DeepSeek for the local route-finding. Keep authority over the destination,
the boundaries, and the acceptance evidence.
