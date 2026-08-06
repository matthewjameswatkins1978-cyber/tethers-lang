# J24L Thin Public Plug Install CLI

Status: `FROZEN`
Owner: `Lucy`
Implementation owner: `DeepSeek Pro`
Planning base: `190e834b8afeca060adb3b07c7a18554497aaf31`

## 1. Purpose

J24L is a thin public driver over the accepted J24K locked single-step
installation executor. Its responsibility is bounded control-flow coordination:
repeatedly call J24K until completion, a legitimate stop, or a hard limit is
reached.

J24L contains no action-specific mutation, no lock acquisition, no recovery
implementation, no store construction, and no schema or protocol change.

## 2. Decomposition

J24L is split into two packages to keep the control-flow logic independently
testable and to defer all CLI and store-layout decisions:

- **J24L1** crate-private bounded driver (`installation_driver.rs`).
  Implemented by DeepSeek Pro.
- **J24L2** public `plug install` CLI, context/store assembly, request-file
  loading, canonical host-data layout, and JSON mapping. Deferred.

## 3. Four-call maximum

The accepted installation progression requires at most four mutations:

```text
CreateExactCandidateTrust
  -> RunSupervisedConformance
  -> CreateInstallationApproval
  -> PublishDisabledInstallation
  -> Complete
```

The driver calls J24K at most four times. A fifth call is forbidden.

A fresh installation legitimately completes on the fourth mutation when the
returned after-plan action is `Complete`. No fifth confirmation call to receive
`AlreadyComplete` is made.

## 4. Fresh lock and fresh plan

Every J24K call acquires its own installation lock and produces a fresh
authoritative plan. The driver never acquires or retains a lock itself. It never
calls the planner outside J24K.

## 5. Stop table

| J24K outcome | Driver action |
|---|---|
| `AlreadyComplete` | Return `Complete` immediately |
| `Advanced` with `after.action == Complete` | Return `Complete` immediately |
| `Advanced` with `after.action != Complete` | Continue if calls < 4 |
| `ConformanceRecordedWithoutAdvance` | Return `ConformanceRecordedWithoutAdvance` immediately |
| Error | Propagate exact error immediately |
| Four returned non-completing steps | Return `installation_iteration_limit` |

## 6. Completion through the fourth step

When an `Advanced` result has `step.after.action == Complete`, the driver
returns success immediately. It does not make a fifth call to confirm
`AlreadyComplete`.

A fresh four-action sequence completes on the fourth mutation whose after-plan
is `Complete`.

## 7. No fifth confirmation call

An installation already at `Complete` causes one normal J24K call. J24K returns
`AlreadyComplete`. The driver returns a completed result containing that exact
one step. No independent planner call is made.

## 8. No conformance retry

`ConformanceRecordedWithoutAdvance` is a legitimate stop result, not a driver
error. The driver returns it immediately and preserves the exact
`InstallationStepResult`. It does not retry conformance and does not distinguish
failed from interrupted conformance; J24K owns that classification.

## 9. Exact propagation of J24K errors

J24K errors are propagated with the exact `M3Error` code and message. No
wrapping, remapping, or new driver error is created.

## 10. Iteration limit

After exactly four returned non-completing `Advanced` results:

- Code: `installation_iteration_limit`
- Message: `installation did not complete within four executor calls`

No fifth call is made. No synthetic step is returned. No indefinite loop exists.

## 11. J24L1 crate-private Rust boundary

```rust
const MAX_INSTALLATION_EXECUTOR_CALLS: usize = 4;

pub(crate) enum InstallationDriveStop {
    Complete,
    ConformanceRecordedWithoutAdvance,
}

pub(crate) struct InstallationDriveResult {
    pub steps: Vec<InstallationStepResult>,
    pub stop: InstallationDriveStop,
}

pub(crate) fn drive_installation(
    request: &InstallationRequest,
    context: &InstallationExecutionContext<'_>,
    options: &InstallationExecutionOptions<'_>,
) -> Result<InstallationDriveResult>;
```

The production entry point delegates to a private closure-based helper
`drive_with<F>(next_step: F)` for testability. Public API outside the crate is
forbidden.

## 12. J24L2 deferred responsibilities

J24L2 (a later package) will add:

- request-file loading from host-data-root;
- canonical host-data subdirectory layout;
- store and context opening from local configuration;
- CLI argument parsing (`plug install`);
- `CliEnvelope` JSON mapping;
- public end-to-end integration tests.

None of these belong in J24L1. Do not invent J24L2 CLI arguments, output
schemas, path names, or error mappings in this package.

## 13. Non-goals

J24L does not:

- parse CLI arguments or print formatted output;
- load an installation request from disk;
- open or create stores;
- construct an `InstallationExecutionContext`;
- change `InstallationExecutionContext`, `InstallationExecutionOptions`, or any
  J24K type;
- acquire a lock;
- hold one lock across multiple executor calls;
- call the planner outside J24K;
- execute any action-specific mutation;
- retry conformance;
- make a fifth confirmation call;
- add a configurable call limit or general loop;
- add public API outside the crate;
- add serialization or a new schema;
- add dependencies or change `Cargo.toml`/`Cargo.lock`;
- change OCaml, language semantics, packaging, or protocols.
