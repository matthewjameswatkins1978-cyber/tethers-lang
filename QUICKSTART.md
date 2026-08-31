# Tethers quick-use manual

This is a short first lesson for Tethers Workbench 0.2.2. It uses one practical
question:

> May this worker inspect the repository, publish a commit, or force-push it?

Tethers answers that question deterministically. It does not run Git, call a
service, or decide what to do with an `ASK`; the surrounding tool owns those
effects.

## 1. Unpack and verify the workbench

Download the package for your platform from the
[Portable 0.2.2 release](https://github.com/matthewjameswatkins1978-cyber/tethers-lang/releases/tag/tethers-portable-v0.2.2).
Unpack it into a directory you control, then run the local self-check.

Windows PowerShell:

```powershell
.\tethers.exe version --json
.\tethers.exe doctor --json
```

Linux:

```bash
./tethers version --json
./tethers doctor --json
```

`doctor` checks the bundled version metadata, policy, evaluator decision, and
parity corpus. It is local and deterministic.

## 2. Make the mental model visible

Every check follows this same route:

```text
JSON request + JSON policy
          |
          v
      Tethers check
          |
          v
 ALLOW / ASK / DENY + evidence
```

The caller then applies the result:

- `ALLOW` — the caller may continue.
- `ASK` — pause and obtain explicit human approval.
- `DENY` — stop.

Tethers is an authority layer, not an action runner. Keeping those responsibilities
separate is the point.

## 3. Try the three decisions

The bundled coding-agent policy is the easiest first experiment.

```powershell
# Windows
.\tethers.exe check --action git.status --json
.\tethers.exe check --action git.push --explain
.\tethers.exe check --action git.force_push --json
```

```bash
# Linux
./tethers check --action git.status --json
./tethers check --action git.push --explain
./tethers check --action git.force_push --json
```

You should see:

| Request | Decision | Process status | Meaning |
| --- | --- | ---: | --- |
| `git.status` | `ALLOW` | `0` | Inspection is passive. |
| `git.push` | `ASK` | `10` | Publication needs a human decision. |
| `git.force_push` | `DENY` | `20` | The operation is prohibited. |

`--explain` is designed for learning and review. It shows the decision, matched
rule, deterministic reason, evaluated conditions, and available trace evidence.
It does not echo secret values.

## 4. Read a request as a Tethers learner

The long form of a request is JSON:

```json
{
  "schema_version": "1",
  "actor": "agent",
  "action": "git.push",
  "resource": "origin",
  "context": {
    "branch": "main"
  }
}
```

Read it as four questions:

1. **Who** is asking? — `actor`
2. **What** do they want to do? — `action`
3. **To what**? — `resource`
4. **Under which facts**? — `context`

The policy matches those facts to a rule. A rule can allow, ask, or deny, and
the policy has a default decision for anything unmatched. The bundled policy
defaults to deny.

## 5. Use files and standard input

The repository includes complete examples under
`tethers-0.1/portable-rust/examples/`.

```powershell
# Windows
.\tethers.exe check .\examples\gary-worker-request.json `
  --policy .\policies\coding-agent-default.json --json
Get-Content .\examples\gary-worker-request.json -Raw |
  .\tethers.exe check - --policy .\policies\coding-agent-default.json --json
```

```bash
# Linux
./tethers check ./examples/gary-worker-request.json \
  --policy ./policies/coding-agent-default.json --json
cat ./examples/gary-worker-request.json |
  ./tethers check - --policy ./policies/coding-agent-default.json --json
```

The file and stdin paths use the same ingestion and evaluator route. `--json`
is stable machine-readable output; `--quiet` emits no stdout and is useful when
the exit code is all the caller needs.

## 6. Create a small local configuration

`init` gives you a runnable starting point instead of a blank directory:

```powershell
.\tethers.exe init --profile coding-agent-default --output .tethers
.\tethers.exe check .tethers\request.json --policy .tethers\policy.json --json
```

Available profiles are `coding-agent-default`, `read-only-agent`, `ci-worker`,
and `gary-worker`. Inspect the generated files before using them in a real
workflow; a profile is a starting policy, not a universal security guarantee.

Validate a policy without evaluating a request:

```text
tethers validate policy.json
tethers lint policy.json
```

## 7. The first useful exercise

Work through a small orchestration loop:

1. Start with `git.status` and record its `ALLOW` result.
2. Change the request to `git.push` and inspect why it becomes `ASK`.
3. Change it to `git.force_push` and confirm the hard deny.
4. Add a harmless context field such as `branch` and rerun with `--json`.
5. Compare the outputs and keep the decision records with the experiment.

This is the important habit: make the work observable, preserve results, avoid
duplicate attempts, and route failure or approval explicitly. Do not bury the
decision inside an agent loop that can silently retry or reinterpret it.

For the complete event-to-action story, read [Bunny & Cookies](docs/BUNNY_AND_COOKIES.md).

## 8. Exit codes for scripts

| Code | Meaning |
| ---: | --- |
| `0` | `ALLOW` |
| `10` | `ASK` |
| `20` | `DENY` |
| `64` | Invalid CLI usage or conflicting options |
| `65` | Invalid request, policy, or schema |
| `66` | Required input or file unavailable |
| `70` | Internal Tethers failure |

An error status is never permission. A caller should treat missing binaries,
timeouts, malformed JSON, schema mismatches, and other operational uncertainty
as fail-closed.

## Next steps

- Read the [complete CLI contract](tethers-0.1/portable-rust/docs/CLI.md).
- Explore the [portable workbench guide](tethers-0.1/portable-rust/README.md).
- Read the [0.1 language specification](tethers-0.1/SPEC.md).
- Try the [record-completed-task example](tethers-0.1/examples/record-completed-task.tether).
