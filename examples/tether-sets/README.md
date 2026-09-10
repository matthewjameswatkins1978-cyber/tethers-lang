# Starter Tether Set examples

This directory is a small, real starter collection for the 0.5 release. The
files are ordinary Tethers 0.1 source files; a host selects them by placing
their relative paths in the existing runtime configuration
`tether_set.tethers` array. There is no second Set language or package format.

The examples deliberately show more than an ALLOW / ASK / DENY decision:

| File | Demonstrates | Existing public evidence |
| --- | --- | --- |
| `typed-work.tether` | typed Facts, Conditions, and a Capability Action | `tethers-0.1/examples/record-completed-task.tether` |
| `together-workflow.tether` | a `together` group followed by a composing Action | `tethers-0.1/protocol/cases/together-happy-path/` |
| `result-follow-on.tether` | an ordinary Action whose successful Result Anchor can wake a later evaluation | `tethers-0.1/host-rust/src/application.rs` Result Anchor/follow-up tests |

The runtime still decides trust, availability, policy, scope, authority,
provider execution, replay, and Trail recording. These source files do not
grant permission merely by being present.
