# Tethers CLI contract

This is the complete 0.2.2 command contract. stdout is reserved for the
requested result; diagnostics go to stderr. JSON output is one deterministic
JSON value with no banner or decoration.

## Decisions and exit status

| Code | Meaning |
| ---: | --- |
| 0 | ALLOW |
| 10 | ASK; human approval is required, not implied |
| 20 | DENY |
| 64 | Invalid CLI usage or conflicting options |
| 65 | Invalid request, policy, or schema; fail closed |
| 66 | Required input, policy, manifest, or file unavailable |
| 70 | Internal Tethers failure |

The check command returns 0, 10, or 20 for a valid decision. It returns an
error code for malformed input or operational failure. A non-zero error code
must never be interpreted as ALLOW.

## check

    tethers check request.json
    cat request.json | tethers check -
    tethers check --action git.push
    tethers check request.json --policy policy.json
    tethers check request.json --json
    tethers check request.json --quiet
    tethers check request.json --explain

The dash means stdin. A request file and stdin use the same ingestion and
evaluator path. The action option constructs the canonical request with actor
agent, resource workspace, and an empty context; it uses the bundled
coding-agent policy unless policy is supplied. It is a convenience facade, not
a second policy engine. Quiet emits no stdout and cannot be combined with JSON
or explain. Explain prints only mechanically available decision, matched-rule,
reason, condition, and optional trace evidence. JSON emits the normal response
object, including provenance fields when available.

## validate and version

    tethers validate policy.json
    tethers validate policy.json --json
    tethers validate - --json
    tethers version
    tethers version --json

Validate only parses and validates a policy. It never executes an action. Its
JSON result is { "valid": true } or { "valid": false, "errors": [...] }.
Line and column fields are omitted when the parser does not expose them;
Tethers never invents locations. Version JSON exposes stable name, version,
engine, policy schema, and target metadata without timestamps.

## doctor

    tethers doctor
    tethers doctor --json

Doctor checks version metadata, the bundled policy, a known evaluator decision,
and the bundled parity corpus. It is local and deterministic; it does not
contact a service or require the OCaml engine at runtime.

## Existing commands

Evaluate remains the backwards-compatible JSON facade and keeps its historic
process-code behavior. Explain, test, lint, init, and validate-manifest remain
available. Evaluate trace and audit retain the provenance features. New
automation should prefer check.

## Fail-closed cases

Empty or malformed stdin, unknown actions, unsupported schema, invalid policy,
missing required policy, inaccessible files, conflicting options, and evaluator
ambiguity all produce DENY or a non-zero error status. None can produce ALLOW.
