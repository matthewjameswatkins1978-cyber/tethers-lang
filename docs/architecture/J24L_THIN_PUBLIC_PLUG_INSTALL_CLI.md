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

The production entry point delegates to a crate-private closure-based helper
`drive_with<F>(next_step: F)` for testability. Public API outside the crate is
forbidden.

## 12. J24L2 public plug install CLI

J24L2 is the second and final package of J24L.

### 12.1. Frozen CLI syntax

```text
plug install
    --host-data-root <ABSOLUTE_PATH>
    --request <ABSOLUTE_JSON_PATH>
```

No package, candidate, retry, iteration, authority, timeout, enable, recovery,
or confirmation arguments. The candidate must already have been created by
`plug stage`.

### 12.2. Canonical host-data layout

```text
<host-data-root>/
    candidates/          (existing stage-owned)
    quarantine/          (existing stage-owned)
    installation-trust/  (create if absent)
    launch-profiles/     (create if absent)
    conformance/         (create if absent)
    installation-approvals/ (create if absent)
    install/             (create if absent)
    installed-records/   (create if absent)
    enablements/         (create if absent, for plug list compatibility)
    conformance-scratch/ (passed to J24K)
    installation-intent/ (created by J24K)
    installation.lock    (passed to J24K)
```

### 12.3. Validation and creation order

1. Validate `--host-data-root` is absolute.
2. Validate `--request` is absolute.
3. Require `host_data_root` to be an existing directory.
4. Verify host-data-root chain through the accepted path-safety helper.
5. Load and validate the request through `load_installation_request`.
6. Open existing candidate and quarantine roots read-only (`CandidateRegistry::open_existing`).
7. Only then open or create remaining installation evidence roots.
8. Construct `InstallationExecutionContext`.
9. Construct frozen options.
10. Call `drive_installation` exactly once.
11. Map result or error into one `PlugCommandResult`.

Invalid CLI paths, unreadable requests, malformed requests, and missing stage
roots must not create later trust, conformance, approval, install, enablement,
or intent state.

### 12.4. Frozen options

```rust
const INSTALL_APPROVING_AUTHORITY: &str = "tethers-reference-host-cli";
const INSTALL_CONFORMANCE_WALL_TIME: Duration = Duration::from_secs(30);
let host_build_identity = concat!("tethers-reference-host/", env!("CARGO_PKG_VERSION"));
```

### 12.5. Public action names

| J24K action | Public string |
|---|---|
| `CreateExactCandidateTrust` | `create_exact_candidate_trust` |
| `RunSupervisedConformance` | `run_supervised_conformance` |
| `CreateInstallationApproval` | `create_installation_approval` |
| `PublishDisabledInstallation` | `publish_disabled_installation` |
| `Complete` | `complete` |

### 12.6. Public step shape

```json
{
  "before_action": "create_exact_candidate_trust",
  "after_action": "run_supervised_conformance",
  "outcome": "advanced",
  "executed_action": "create_exact_candidate_trust"
}
```

Conformance disposition strings: `passed`, `failed`, `interrupted`.
`Invalidated` is a stored-evidence state but not a legitimate live
`ConformanceRecordedWithoutAdvance` result from J24K. At the J24L boundary it
is treated as contradictory (same as `Passed` in a non-advancing stop) and
fails closed with `installation_execution_postcondition_failed`.

### 12.7. Completed output

```json
{
  "result": "complete",
  "candidate_id": "<request candidate UUID>",
  "step_count": 4,
  "steps": [],
  "installed_id": "<installed UUID>",
  "installed_record_digest": "sha256:..."
}
```

### 12.8. Non-advancing conformance mapping

| Disposition | Exit | Code |
|---|---|---|
| `Failed` | 6 | `installation_conformance_failed` |
| `Interrupted` | 10 | `installation_conformance_interrupted` |
| `Passed` or `Invalidated` | 6 | `installation_execution_postcondition_failed` |

### 12.9. Error status mapping

| Error codes | Status |
|---|---|
| `installation_request_io`, `candidate_io`, `store_io`, `installation_busy`, `installation_lock_io`, `installation_recovery_io` | `unavailable` |
| `installation_iteration_limit`, `installation_execution_stagnant`, `installation_execution_regressed`, `installation_execution_invalid_transition`, `installation_execution_postcondition_failed`, `installation_scratch_cleanup_failed` | `failed` |
| All other codes | `invalid_data` |

### 12.10. Integration evidence

- Clap tests: exact valid parse, reordered options, all missing/duplicate/unknown rejections.
- Pre-mutation validation: relative/missing paths, malformed requests, missing stage roots.
- Pure mapping: all completion, conformance, error, and contradiction branches.
- Windows end-to-end: fresh install in four steps, disabled record, no intent, scratch clean, enablements/ empty, second invocation already complete.

### 12.11. J24L completion and merge boundary

J24L is complete. No further J24L packages exist.

Merging the J24L branch into main requires the accepted J24L1 and J24L2
packages together. The J24L2 branch descends directly from J24L1's verified
tip.
