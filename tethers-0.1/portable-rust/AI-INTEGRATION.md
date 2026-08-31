# AI integration contract

An agent or workbench owns execution; Tethers owns the authority decision.
The integration constructs a canonical request, invokes the executable as a
subprocess, validates the response schema, and only then interprets it. No
wrapper may implement a second policy engine.

The decision order is fixed: request/schema/action validation; manifest and
policy validation; manifest and scope narrowing; global hard denies; first
matching policy rule; policy default. Conceptually this is
`GLOBAL HARD` -> `WORKBENCH DEFAULT` -> `PROJECT` -> `JOB`, with later layers
only narrowing authority. `ASK` requires an explicit human approval step.

`decision_id` and `policy_sha256` provide deterministic provenance. `--trace`
adds rule and condition outcomes without echoing condition values. `--audit`
adds an optional JSONL record and fails closed if it cannot append.

The official plug translators under `plugs/` cover common host surfaces. They
only construct requests. Path resolution and symlink/junction containment are
performed before evaluation; a host that cannot resolve a path reliably must
deny the operation.
