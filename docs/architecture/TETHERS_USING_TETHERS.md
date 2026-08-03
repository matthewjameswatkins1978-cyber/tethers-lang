# Tethers Using Tethers

Status: committed future direction, not yet scheduled.

## Decision

Tethers will eventually use Tether Sets to coordinate selected parts of its own development, testing, Plug admission, release preparation, documentation checks, and operational workflows.

This is not a plan to rewrite the trusted OCaml semantic core or Rust host in Tethers. It is a plan to move suitable visible behaviour out of hidden control flow and into inspectable Tether Sets.

The governing boundary is:

> Machinery stays below. Behaviour moves above.

Or more plainly:

> Tethers may coordinate its own workshop. It must not redefine its own physics.

## Why this matters

Using Tethers to operate parts of Tethers will:

- make the project its own demanding real-world customer;
- expose awkward language or capability design early;
- reduce workflow behaviour buried in Rust, scripts, or HQ;
- make internal processes visible in HQ and Tethers Shell;
- let the Trail explain Tethers in the same terms users see;
- make workflows portable, replaceable, and testable;
- sharpen the boundary between deterministic behaviour and trusted machinery;
- give AIs and humans a smaller, clearer model of why something happened.

The goal is not novelty or traditional compiler self-hosting. The goal is self-application: Tethers should be able to describe, coordinate, test, and explain much of the work around its own core.

## Trusted core that remains outside Tethers

The following stay implemented in ordinary trusted code:

- parsing and canonical syntax;
- semantic validation and deterministic evaluation;
- capability resolution;
- permission and policy enforcement;
- package, manifest, signature, and digest verification;
- process supervision and sandbox boundaries;
- replay identity and recovery machinery;
- durable Trail writing;
- cryptography;
- filesystem and operating-system safety primitives.

A Tether may request these operations only through declared capabilities. It may not redefine or bypass their rules.

## Good candidates for Tether Sets

Behaviour suitable for later self-application includes:

- selecting tests after a bounded code change;
- requesting review when evidence is complete;
- checking required documentation;
- coordinating Plug inspection, conformance, approval, installation, and disablement;
- preparing a release candidate after explicit gates pass;
- requiring Matthew's approval before consequential transitions;
- recording project and release evidence;
- compatibility and migration checks;
- operational recovery sequences built from safe host capabilities;
- HQ guidance flows that present only valid next actions.

These are questions of when, under what conditions, in what permitted order, and with which explicit approval. Those are Tethers questions.

## Example direction

```text
when plug.package.received
and package.inspection passed
and package.conformance passed
do human.approval.request
```

```text
when human.approval.granted
and candidate is still current
do plug.install_disabled
do trail.record
```

```text
when code.changed
and area is parser
do tests.run parser
do tests.run language
do review.request
```

The host still performs inspection, testing, installation, Git operations, and Trail durability. Tethers controls the visible sequence using only granted capabilities.

## Staged plan

### Stage 0: Preserve the boundary

Before implementation:

- keep this plan in the architecture record;
- identify hard-coded workflows that are actually behaviour;
- define the host capabilities those workflows would need;
- reject proposals that move trusted machinery into Tethers;
- keep all current milestones unchanged.

### Stage 1: Observer mode

Create one Tether Set that evaluates a real Tethers development event but performs no consequential action.

It should:

- receive a bounded Anchor such as `code.changed`;
- inspect supplied facts;
- produce a proposed test and review plan;
- record why each rule matched or did not match;
- remain advisory only.

Success means the Tether Set describes the same decision a trusted human or existing workflow would make, with no hidden authority.

### Stage 2: Safe development coordination

Allow a Tether Set to invoke low-risk capabilities such as:

- `tests.run`;
- `format.check`;
- `docs.check`;
- `review.request`;
- `trail.record`.

No merge, release, deletion, installation, or broad shell capability is permitted in this stage.

### Stage 3: Plug admission orchestration

Express the existing Plug lifecycle as visible behaviour:

```text
package received
→ inspect
→ establish trust
→ run conformance
→ request approval
→ install disabled
→ enable with exact scope
```

Each underlying operation remains host-owned. The Tether Set coordinates the gates and records why progress continued or stopped.

### Stage 4: Release candidate preparation

Use Tethers to coordinate release evidence:

- required tests;
- compatibility checks;
- documentation status;
- package evidence;
- unresolved warnings;
- explicit Matthew approval.

The first implementation prepares a release candidate only. Tagging and publishing remain separately authorised capabilities and must never occur through an implicit rule.

### Stage 5: HQ shows Tethers governing Tethers

HQ should display these internal Tether Sets exactly as it displays ordinary user workflows:

- current rule;
- available capabilities;
- hard limits;
- pending approvals;
- failed conditions;
- Trail evidence;
- safe next actions.

The internal workflow must not receive a privileged secret interface. It should use the same stable contracts and explanation model as any other Tether Set.

### Stage 6: Expand only where it removes hidden behaviour

After the earlier stages prove useful, consider more internal workflows one at a time.

A behaviour moves into Tethers only when doing so makes it more visible, testable, portable, and understandable without weakening the trusted boundary.

## Entry conditions

Implementation should not begin merely because the idea is attractive. Begin when the relevant foundations are stable enough that this work tests Tethers rather than distorting unfinished foundations.

Recommended entry conditions:

- Tethers 1.0 language semantics are stable or near-stable;
- Socket and Plug v1 contracts are stable;
- capability effects, scopes, and permissions are machine-readable;
- HQ or Tethers Shell can display rules and Trail evidence faithfully;
- the host has narrow capabilities for tests, review, package admission, and release preparation;
- there is a trusted hard-coded fallback for the first self-applied workflow;
- golden tests can compare the Tether-governed decision with the previous trusted behaviour;
- the entire self-applied Tether Set can be disabled without disabling the Tethers core.

These are readiness conditions, not a fixed date.

## Safety rules

1. No self-modifying Tether Set.
2. No Tether may grant itself a capability.
3. No implicit shell or arbitrary-code capability.
4. No consequential action without the same policy and permission path used for ordinary Tethers.
5. No release, merge, installation, deletion, or external publication without explicit declared authority.
6. No hidden fallback that performs an action the visible Tether did not plan.
7. Every decision and action must appear in the canonical Trail.
8. The self-applied workflow must fail closed when its Tether Set, capability, evidence, or scope is unavailable.
9. The trusted core must be able to disable the self-applied workflow from outside that workflow.
10. Self-application must reduce hidden behaviour, not create a second internal language.

## Testing strategy

Each self-applied workflow requires:

- deterministic input fixtures;
- expected planned Actions;
- negative cases proving forbidden Actions are not planned;
- comparison against the previous trusted workflow where one exists;
- Trail ordering tests;
- permission and stale-evidence tests;
- replay tests where execution identity applies;
- disablement and fallback tests;
- HQ or Shell explanation snapshots.

The key test remains simple:

> Given this Anchor and these Facts, which Actions are planned, why, and which Actions are impossible?

## Success criteria

The first meaningful self-application is successful when:

- Matthew can read the active Tether Set and understand the workflow;
- HQ or Shell shows only actions supported by available capabilities;
- the same rules can be tested without invoking effects;
- real execution uses the ordinary policy, intent, replay, outcome, and Trail boundaries;
- disabling the Tether Set removes the behaviour cleanly;
- the host contains less hidden sequencing rather than more adapter code;
- a failure explains itself using the visible rule, failed condition, capability boundary, and Trail;
- no part of the trusted core depends on a Tether Set to remain safe.

## First recommended experiment

The safest first experiment is development-test selection in observer mode.

It is useful, frequent, easy to compare with existing practice, and does not need authority to alter code or releases. It will quickly reveal whether Tethers can express its own real workflows without adding language features merely to flatter the experiment.

## Long-term shape

```text
Small OCaml semantic core
    understands and evaluates Tethers

Small Rust host
    enforces authority and performs capabilities

Tether Sets
    coordinate much of Tethers' visible development and operational behaviour

HQ / Shell
    show the same rules, limits, choices, and Trail to Matthew
```

This is deliberately not traditional total self-hosting.

It is a more appropriate form for Tethers:

> Tethers does not need to build its own laws of physics. It should be able to run, test, and explain its own workshop.
