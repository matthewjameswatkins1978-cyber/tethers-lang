# Policy precedence

The global hard-deny set is immutable by project policy. A supplied manifest
can further restrict the action set. A request scope can further restrict
actions and exact files. The selected policy is then evaluated in rule order,
followed by its default.

`GLOBAL HARD` -> `WORKBENCH DEFAULT` -> `PROJECT` -> `JOB` means narrowing
authority, not overriding a stronger layer. Ambiguous conditions and malformed
policy are `DENY`.
