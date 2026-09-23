# tethers.authority/1

Status: frozen machine protocol (Tethers R2)
Owner: Tethers Authority Gate (`tethers gate --stdio`)

`tethers.authority/1` is the request/response protocol between an external
Host and the Tethers Authority Gate. It is an independent version axis from
the product version (`0.8.0`); a consumer must match `schema` exactly and
must never infer the protocol identity from the product version.

Architecture contract: `docs/architecture/TETHERS_R2_EXTERNAL_AUTHORITY_GATE.md`.

## Transport

- Newline-delimited JSON (NDJSON) over persistent local stdio. One frame per
  line; `\n` terminates a frame; trailing `\r`/`\n` are stripped before
  parsing.
- The Gate writes responses to stdout and flushes after each frame.
  Diagnostics go to stderr only, formatted `tethers-gate: <message>
  (<code>)`, bounded to 64 lines per session.
- stdin EOF ends the session cleanly. A partial frame at EOF is discarded
  with `frame.invalid_json` ("connection closed mid-frame").
- No prompts. No interactive reads. A partial frame that makes no progress
  for 30 s (`FRAME_WAIT`) is discarded with `frame.timeout` and the session
  continues.
- The process exits after `shutdown`, stdin EOF, or an IO failure. On clean
  exit it emits a `tethers.cli/1` envelope whose `data.schema` is
  `tethers.gate/1`.

CLI:

```text
tethers gate --stdio --config PATH --trail ABS --host-data-root ABS
```

`--stdio` is required. `--config`, `--trail`, and `--host-data-root` must be
absolute paths; the config must be an existing file. Violations fail closed
before any frame is accepted (see `GATE_*` envelope codes below).

## Frames

### Request frame

| Field | Type | Required | Constraints |
| --- | --- | --- | --- |
| `schema` | string | yes | must equal `tethers.authority/1`; otherwise `frame.unsupported_schema` |
| `request_id` | string | yes | 1–128 bytes, no control characters, unique per session |
| `operation` | string | yes | one of `hello`, `prepare`, `approval_decision`, `commit`, `outcome`, `status`, `shutdown`; otherwise `frame.unknown_operation` |
| `payload` | object | yes | JSON object; serialised size ≤ 512 KiB |

Rules enforced at the frame layer:

- The frame must be a JSON object (`frame.not_object`), valid JSON
  (`frame.invalid_json`), and non-empty (`frame.empty`).
- Duplicate top-level fields are refused (`frame.duplicate_field`); the Gate
  never silently takes the last duplicate.
- Unknown `operation` values are refused, never guessed.

### Response frame

| Field | Type | Required | Constraints |
| --- | --- | --- | --- |
| `schema` | string | yes | always `tethers.authority/1` |
| `request_id` | string | yes | echoes the request's `request_id`; `""` when the request could not be parsed far enough to recover one |
| `status` | string | yes | `ok` \| `error` |
| `result` | object | iff `status = ok` | operation-specific result |
| `error` | object | iff `status = error` | `{code, message, data?}` |

`error` fields:

| Field | Type | Constraints |
| --- | --- | --- |
| `code` | string | machine code from the tables below |
| `message` | string | ≤ 2048 bytes (truncated at a char boundary) |
| `data` | any | optional structured detail (present on `commit.replay_blocked`) |

Exactly one of `result` / `error` is present. A structured error never ends
the session; the Gate remains healthy and answers subsequent frames.

## Bounds

| Bound | Value |
| --- | --- |
| `MAX_FRAME_BYTES` — one raw frame | 1 048 576 bytes (1 MiB) |
| `MAX_PAYLOAD_BYTES` — serialised `payload` alone | 524 288 bytes (512 KiB) |
| `MAX_REQUEST_ID_BYTES` — `request_id` | 128 bytes |
| `MAX_ERROR_MESSAGE_BYTES` — `error.message` | 2 048 bytes |
| `MAX_REMEMBERED_REQUEST_IDS` — distinct `request_id` values per session | 8 192 |
| `MAX_STATUS_ENTRIES` — any collection returned by `status` | 128 |
| `FRAME_WAIT` — stalled partial frame | 30 s |
| stderr diagnostic lines per session | 64 |
| Required string payload fields | ≤ 4 096 bytes, non-empty |
| Optional string payload fields (`outcome`) | ≤ 16 384 bytes |
| `observations.available_provider_identities` | ≤ 64 entries, each 1–256 bytes |

Oversized frames: once the inbound buffer exceeds 1 MiB without a newline,
the Gate immediately emits `frame.oversized` (with `request_id: ""`) and
discards bytes until the next newline.

## Request identity

- `request_id` must be non-empty, ≤ 128 bytes, and contain no control
  characters; otherwise `frame.request_id_invalid`.
- Each distinct `request_id` may be used once per session; a repeat is
  refused with `frame.request_id_duplicate` before the operation runs.
- After 8 192 distinct identities, new ones are refused with
  `frame.request_id_space_exhausted` (already-seen identities remain
  reject-on-sight duplicates).

## Forbidden authority keys

On `prepare` (and nested anywhere inside `plan`), the keys
`permission`, `within_scope`, `trusted`, `approved`, `authority_granted`,
`granted` are refused with `frame.forbidden_authority_key`. The Gate
recomputes every decision; caller-supplied authority booleans are never
read. `approval_decision` legitimately carries its own `decision` field and
is exempt from this rule.

## Operations

### `hello`

Negotiates protocol identity. Payload must be empty; a non-empty payload is
refused with `frame.unexpected_payload`.

Request:

```json
{"schema":"tethers.authority/1","request_id":"req-hello-1","operation":"hello","payload":{}}
```

Result:

| Field | Type | Notes |
| --- | --- | --- |
| `protocol` | string | `tethers.authority/1` |
| `protocol_versions` | string[] | `[`tethers.authority/1`]` |
| `product_version` | string | e.g. `0.7.1`; independent of the protocol identity |
| `git_sha` | string \| null | build provenance when available |
| `features` | string[] | `[`prepare`, `approval_decision`, `commit`, `outcome`, `status`, `shutdown`]` |
| `gate_instance_id` | string | `gate_<uuid>` per process |
| `authority_granted` | boolean | always `false` |
| `provider_invocations` | integer | always `0` |

```json
{"schema":"tethers.authority/1","request_id":"req-hello-1","status":"ok","result":{"protocol":"tethers.authority/1","protocol_versions":["tethers.authority/1"],"product_version":"0.7.1","git_sha":null,"features":["prepare","approval_decision","commit","outcome","status","shutdown"],"gate_instance_id":"gate_2f6c1a40-0000-4000-8000-000000000001","authority_granted":false,"provider_invocations":0}}
```

### `prepare`

Read-only current-policy evaluation of one exact Action from supplied
`tethers.plan/1` material. Never authorises dispatch.

Payload fields:

| Field | Type | Required | Constraints |
| --- | --- | --- | --- |
| `tether_id` | string | yes | non-empty, ≤ 4096 bytes; Tether must be configured exactly once |
| `tether_version` | string | yes | non-empty, ≤ 4096 bytes |
| `evaluation_id` | string | yes | non-empty, ≤ 4096 bytes |
| `event_id` | string | yes | non-empty, ≤ 4096 bytes |
| `action_id` | string | yes | non-empty, ≤ 4096 bytes; must select an entry of `plan.actions` |
| `plan` | object | yes | must contain `id` (non-empty string) and `actions` (array) |
| `observations` | object | no | see below |

Fields read from the selected plan action:

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| `action_id` | string | yes | must equal the payload's `action_id` |
| `capability` | string | yes | capability name |
| `manifest_digest` | string | no | pinned manifest digest |
| `bridge_capability_version` | integer | no | `u32` > 0; required for COMMIT (`commit.missing_pin`) |
| `bridge_provider_identity` | string | no | pinned provider identity |
| `arguments` | any | no | defaults to `null`; digested, never echoed raw |

`observations` (optional):

| Field | Type | Constraints |
| --- | --- | --- |
| `available_provider_identities` | string[] \| null | the only permitted key; ≤ 64 entries, each 1–256 bytes; omit or `null` to use configured provider identities as the availability set |

Unknown keys inside `observations` are refused
(`frame.payload_invalid`, message prefix `prepare.unknown_observation`).
Observations are bounded physical facts supplied by the Host — never
authority decisions.

Result:

| Field | Type | Notes |
| --- | --- | --- |
| `prepared_id` | string | `prep_<hex>`; session handle bound to canonical evaluation material (includes config digest) |
| `decision` | string | `allow_prepared` \| `ask` \| `deny` \| `unavailable` |
| `reason` | string | machine reason code (table below) |
| `authorizes_dispatch` | boolean | always `false` |
| `evaluation_id` | string | echo |
| `action_id` | string | echo |
| `tether_id` | string | echo |
| `tether_version` | string | echo |
| `provider_invocations` | integer | always `0` |
| `execution.performed` | boolean | always `false` |
| `execution.provider_invocations` | integer | always `0` |
| `execution.replay_mutated` | boolean | always `false` |
| `approval` | object | present iff `decision = ask` |

`approval` object:

| Field | Type | Notes |
| --- | --- | --- |
| `approval_id` | string | `approval-N`, process-local |
| `action_id` | string | echo |
| `capability.name` | string | |
| `capability.version` | integer | bridge capability version |
| `reason` | string | same as top-level `reason` |
| `argument_digest` | string | `sha256:<hex>` of the Action arguments |
| `effect_summary` | string | one-line human summary |
| `state` | string | `pending` |

Request:

```json
{"schema":"tethers.authority/1","request_id":"req-prepare-1","operation":"prepare","payload":{"tether_id":"r2-complete","tether_version":"1","evaluation_id":"eval_r2_001","event_id":"evt_r2_001","action_id":"action_1","plan":{"id":"eval_r2_001/plan","actions":[{"action_id":"action_1","idempotency_key":"eval_r2_001/action_1","capability":"fixture.ping","capability_version":"1.0.0","arguments":{"message":"r2","path":"projects/r2-stdio"},"effects":["fixture.test"],"manifest_digest":"sha256:eb61b62bde489e00a4d15c37c83e6cdb1e9e378b8f13b910d4b68bd6d68c19da","bridge_capability_version":1,"bridge_provider_identity":"tethers-stdio-fixture"}]}}}
```

Result (`allow`):

```json
{"schema":"tethers.authority/1","request_id":"req-prepare-1","status":"ok","result":{"prepared_id":"prep_9c1f0a2b3c4d5e6f708192a3b4c5d6e7f8091a2b3c4d5e6f708192a3b4c5d6e7","decision":"allow_prepared","reason":"current_policy_allow","authorizes_dispatch":false,"evaluation_id":"eval_r2_001","action_id":"action_1","tether_id":"r2-complete","tether_version":"1","provider_invocations":0,"execution":{"performed":false,"provider_invocations":0,"replay_mutated":false}}}
```

Result (`ask` — adds `approval`):

```json
{"schema":"tethers.authority/1","request_id":"req-prepare-2","status":"ok","result":{"prepared_id":"prep_0f1e2d3c4b5a69788796a5b4c3d2e1f009182736455463728190aabbccddeeff","decision":"ask","reason":"host_policy_ask","authorizes_dispatch":false,"evaluation_id":"eval_r2_001","action_id":"action_1","tether_id":"r2-complete","tether_version":"1","provider_invocations":0,"execution":{"performed":false,"provider_invocations":0,"replay_mutated":false},"approval":{"approval_id":"approval-1","action_id":"action_1","capability":{"name":"fixture.ping","version":1},"reason":"host_policy_ask","argument_digest":"sha256:2f0c9d8e7b6a594837261504f3e2d1c0b9a88776655443322110ffeeddccbbaa","effect_summary":"Dispatch fixture.ping@Some(1) with argument digest sha256:2f0c9d8e7b6a594837261504f3e2d1c0b9a88776655443322110ffeeddccbbaa","state":"pending"}}}}
```

`reason` values:

| `reason` | Meaning |
| --- | --- |
| `current_policy_allow` | effective policy allows (paired with `allow_prepared`; the only `reason` emitted for an allow) |
| `host_policy_ask` | effective policy asks |
| `host_policy_deny` | effective policy denies |
| `manifest_requires_confirmation` | manifest requires confirmation → ask |
| `scope_violation` | proposed Action outside established scope |
| `scope_not_established` | no scope established for this Action |
| `provider_unavailable` | no available provider for the binding |
| `provider_identity_mismatch` | observed/configured identity mismatch |
| `manifest_digest_mismatch` | pinned digest does not match admitted manifest |
| `no_admitted_manifest` | no admitted manifest for the capability |
| `missing_bridge_pin` | bridge capability version/provider pin missing |
| `undeclared_capability` | capability not declared in requirements |
| `input_schema_violation` | arguments violate the input schema |
| `empty_identifier` | empty required identifier |
| `unsupported_policy_configuration` | policy config cannot be evaluated → `unavailable` |

### `approval_decision`

Relays one exact human decision. Tethers owns the record.

Payload:

| Field | Type | Required | Constraints |
| --- | --- | --- | --- |
| `approval_id` | string | yes | non-empty, ≤ 4096 bytes |
| `decision` | string | yes | `approve` \| `deny` \| `cancel`; otherwise refusal (see payload-parser sub-codes) |

Result:

| Field | Type | Notes |
| --- | --- | --- |
| `approval_id` | string | echo |
| `state` | string | `approved` \| `denied` \| `cancelled` (stored state vocabulary also includes `pending`, `invalidated`, `consumed`) |
| `action_id` | string | from the bound proof |
| `capability.name` | string | from the bound proof |
| `capability.version` | integer | from the bound proof |
| `authorizes_dispatch` | boolean | always `false` — approval is not standing permission |
| `provider_invocations` | integer | always `0` |

Request:

```json
{"schema":"tethers.authority/1","request_id":"req-approval-1","operation":"approval_decision","payload":{"approval_id":"approval-1","decision":"approve"}}
```

Result:

```json
{"schema":"tethers.authority/1","request_id":"req-approval-1","status":"ok","result":{"approval_id":"approval-1","state":"approved","action_id":"action_1","capability":{"name":"fixture.ping","version":1},"authorizes_dispatch":false,"provider_invocations":0}}
```

Approval records are process-local and expire on Gate restart; a fresh
process refuses decisions for prior identities with
`approval.decision_refused`.

### `commit`

Last-responsible-moment authority check plus durable intent. Returns the
dispatch record or refuses.

Payload:

| Field | Type | Required | Constraints |
| --- | --- | --- | --- |
| `prepared_id` | string | yes | non-empty, ≤ 4096 bytes; must be a known, uncommitted preparation |
| `approval_id` | string \| null | no | non-empty if present; overrides the approval recorded at PREPARE |
| `observations` | object | no | same shape as `prepare.observations`; replaces the prepared observations when supplied |

Result is a `tethers.dispatch/1` record (next section).

Request:

```json
{"schema":"tethers.authority/1","request_id":"req-commit-1","operation":"commit","payload":{"prepared_id":"prep_9c1f0a2b3c4d5e6f708192a3b4c5d6e7f8091a2b3c4d5e6f708192a3b4c5d6e7"}}
```

Result:

```json
{"schema":"tethers.authority/1","request_id":"req-commit-1","status":"ok","result":{"schema":"tethers.dispatch/1","execution_id":"exec_01JQ0R2ALLOW00000000000000","evaluation_id":"eval_r2_001","action_id":"action_1","event_id":"evt_r2_001","capability":{"name":"fixture.ping","version":1},"argument_digest":"sha256:aa11bb22cc33dd44ee55ff667788990011223344556677889900aabbccddeeff","manifest_digest":"sha256:eb61b62bde489e00a4d15c37c83e6cdb1e9e378b8f13b910d4b68bd6d68c19da","provider_identity":"tethers-stdio-fixture","intent":{"trail":"recorded","replay":"armed"},"authority_protocol":"tethers.authority/1","prepared_id":"prep_9c1f0a2b3c4d5e6f708192a3b4c5d6e7f8091a2b3c4d5e6f708192a3b4c5d6e7","approval_consumed":false,"authorizes_physical_execution_by_tethers":false,"host_must_report_outcome":true,"provider_invocations":0}}
```

Refusal example with structured data:

```json
{"schema":"tethers.authority/1","request_id":"req-commit-2","status":"error","error":{"code":"commit.replay_blocked","message":"replay refused a second admission: replay_blocked_completed_success","data":{"replay":"replay_blocked_completed_success","execution_id":"exec_01JQ0R2ALLOW00000000000000"}}}
```

`error.data.replay` vocabulary: `replay_blocked_completed_success`,
`replay_blocked_completed_failure`, `replay_requires_manual_resolution`,
`replay_persistence_unavailable`.

On success the Gate holds the replay admission guard until the matching
`outcome`.

### `outcome`

Binds one Host physical observation to one committed execution.

Payload:

| Field | Type | Required | Constraints |
| --- | --- | --- | --- |
| `execution_id` | string | yes | non-empty, ≤ 4096 bytes; must be a committed Gate execution |
| `classification` | string | yes | `succeeded` \| `failed` \| `uncertain` |
| `attempted` | boolean | no (default `true`) | must be `true` for a committed dispatch |
| `external_execution_identity` | string | no | ≤ 16384 bytes; must not equal the Tethers `action_id` |
| `result` | any non-null | conditional | required iff `succeeded`; mutually exclusive with `error` |
| `error` | string | conditional | required iff `failed`; ≤ 16384 bytes; mutually exclusive with `result` |
| `reason_code` | string | no | ≤ 16384 bytes |
| `evidence` | string | no | ≤ 16384 bytes |

Result (first report):

| Field | Type | Notes |
| --- | --- | --- |
| `execution_id` | string | echo |
| `status` | string | the recorded classification |
| `attempted` | boolean | echo |
| `external_execution_identity` | string \| null | echo |
| `evaluation_id` | string | from the committed record |
| `action_id` | string | from the committed record |
| `capability.name` / `capability.version` | string / integer | from the committed record |
| `manifest_digest` | string | from the committed record |
| `provider_identity` | string | from the committed record |
| `argument_digest` | string | from the committed record |
| `prepared_id` | string | originating preparation |
| `replay_terminal` | string | `recorded` \| `recovery_required` |
| `trail_outcome_recorded` | boolean | `true` |
| `idempotent` | boolean | `false` on first report |
| `provider_invocations` | integer | always `0` |

Result (idempotent repeat of the same classification):

```json
{"schema":"tethers.authority/1","request_id":"req-outcome-2","status":"ok","result":{"execution_id":"exec_01JQ0R2ALLOW00000000000000","status":"succeeded","idempotent":true,"provider_invocations":0}}
```

A different classification for an already-recorded execution is refused with
`outcome.conflict`. After a Gate restart the in-memory committed map is
empty, so a late outcome for a prior execution is refused with
`outcome.unknown_execution` (durable state is surfaced through
`status.recovery_required`, never guessed).

Request:

```json
{"schema":"tethers.authority/1","request_id":"req-outcome-1","operation":"outcome","payload":{"execution_id":"exec_01JQ0R2ALLOW00000000000000","classification":"succeeded","attempted":true,"external_execution_identity":"external-exec-1","result":{"echo":"r2"},"evidence":"sha256:evidence-fixture"}}
```

Result:

```json
{"schema":"tethers.authority/1","request_id":"req-outcome-1","status":"ok","result":{"execution_id":"exec_01JQ0R2ALLOW00000000000000","status":"succeeded","attempted":true,"external_execution_identity":"external-exec-1","evaluation_id":"eval_r2_001","action_id":"action_1","capability":{"name":"fixture.ping","version":1},"manifest_digest":"sha256:eb61b62bde489e00a4d15c37c83e6cdb1e9e378b8f13b910d4b68bd6d68c19da","provider_identity":"tethers-stdio-fixture","argument_digest":"sha256:aa11bb22cc33dd44ee55ff667788990011223344556677889900aabbccddeeff","prepared_id":"prep_9c1f0a2b3c4d5e6f708192a3b4c5d6e7f8091a2b3c4d5e6f708192a3b4c5d6e7","replay_terminal":"recorded","trail_outcome_recorded":true,"idempotent":false,"provider_invocations":0}}
```

### `status`

Bounded reconciliation surface for a reconnecting Host. Payload is ignored;
`{}` is the canonical form.

Result:

| Field | Type | Notes |
| --- | --- | --- |
| `protocol` | string | `tethers.authority/1` |
| `product_version` | string | e.g. `0.7.1` |
| `gate_instance_id` | string | `gate_<uuid>` |
| `healthy` | boolean | `true` when the Gate is answering |
| `shutdown_requested` | boolean | `true` after a `shutdown` operation |
| `provider_invocations` | integer | always `0` |
| `pending_approvals` | object[] | ≤ 128; `{approval_id, state, action_id}` for `pending`/`approved` records; proofs are never exposed |
| `unresolved_commits` | object[] | ≤ 128; committed executions with no recorded outcome |
| `prepared` | object[] | ≤ 128; `{prepared_id, decision, committed, action_id, tether_id, tether_version, config_digest}` |
| `terminal_outcomes` | object[] | ≤ 128; committed executions with a recorded outcome |
| `recovery_required` | object[] | ≤ 128; durable Trail intents without a matching outcome: `{execution_id, action_id, state: "committed_outcome_incomplete"}` |
| `prepared_count` | integer | total preparations this session |
| `committed_count` | integer | total commits this session |

Commit summary object (used by `unresolved_commits` and
`terminal_outcomes`):

| Field | Type |
| --- | --- |
| `execution_id` | string |
| `action_id` | string |
| `evaluation_id` | string |
| `capability.name` / `capability.version` | string / integer |
| `argument_digest` | string |
| `prepared_id` | string |
| `outcome` | string \| null |

Request:

```json
{"schema":"tethers.authority/1","request_id":"req-status-1","operation":"status","payload":{}}
```

Result:

```json
{"schema":"tethers.authority/1","request_id":"req-status-1","status":"ok","result":{"protocol":"tethers.authority/1","product_version":"0.7.1","gate_instance_id":"gate_2f6c1a40-0000-4000-8000-000000000001","healthy":true,"shutdown_requested":false,"provider_invocations":0,"pending_approvals":[],"unresolved_commits":[],"prepared":[{"prepared_id":"prep_9c1f0a2b3c4d5e6f708192a3b4c5d6e7f8091a2b3c4d5e6f708192a3b4c5d6e7","decision":"allow_prepared","committed":false,"action_id":"action_1","tether_id":"r2-complete","tether_version":"1","config_digest":"sha256:1122334455667788990011223344556677889900112233445566778899001122"}],"terminal_outcomes":[],"recovery_required":[],"prepared_count":1,"committed_count":0}}
```

No protocol surface echoes raw argument values; only digests travel.

### `shutdown`

Requests orderly session end. Payload is ignored; `{}` is the canonical
form. The primary response is written first, then the process exits.

Request:

```json
{"schema":"tethers.authority/1","request_id":"req-shutdown-1","operation":"shutdown","payload":{}}
```

Result:

```json
{"schema":"tethers.authority/1","request_id":"req-shutdown-1","status":"ok","result":{"shutdown":true,"provider_invocations":0}}
```

Final CLI envelope on clean exit (`tethers.cli/1`, abbreviated):

```json
{"schema":"tethers.cli/1","command":"gate","status":"ok","exit_code":0,"data":{"schema":"tethers.gate/1","transport":"stdio","shutdown":true,"gate_instance_id":"gate_2f6c1a40-0000-4000-8000-000000000001","provider_invocations":0}}
```

## tethers.dispatch/1

The `result` of a successful `commit`. Shape:

| Field | Type | Notes |
| --- | --- | --- |
| `schema` | string | always `tethers.dispatch/1` |
| `execution_id` | string | Gate-issued durable execution identity; correlate `outcome` by this value |
| `evaluation_id` | string | echo from the prepared material |
| `action_id` | string | echo |
| `event_id` | string | echo |
| `capability.name` | string | resolved capability name |
| `capability.version` | integer | bridge capability version |
| `argument_digest` | string | `sha256:<hex>` of the Action arguments |
| `manifest_digest` | string | resolved manifest digest |
| `provider_identity` | string | resolved provider identity |
| `intent.trail` | string | always `recorded` |
| `intent.replay` | string | always `armed` |
| `authority_protocol` | string | always `tethers.authority/1` |
| `prepared_id` | string | originating preparation |
| `approval_consumed` | boolean | `true` iff an exact approval was consumed for this commit |
| `authorizes_physical_execution_by_tethers` | boolean | always `false` — Tethers did not execute and does not execute |
| `host_must_report_outcome` | boolean | always `true` |
| `provider_invocations` | integer | always `0` |

Meaning: durable Trail intent is recorded, replay is armed, and the external
Host may now dispatch the effect physically and must report exactly one
`outcome` for this `execution_id`.

## Error codes

### `frame.*` — frame and transport layer

| Code | Condition |
| --- | --- |
| `frame.empty` | blank line / empty frame |
| `frame.oversized` | frame exceeds 1 MiB (emitted immediately; remainder discarded to the next newline) |
| `frame.invalid_json` | not valid JSON; also used when the connection closes mid-frame (partial discarded) |
| `frame.duplicate_field` | duplicate top-level field in the request |
| `frame.not_object` | frame is not a JSON object |
| `frame.missing_field` | required frame or payload field absent |
| `frame.wrong_type` | field present with the wrong JSON type |
| `frame.unsupported_schema` | `schema` ≠ `tethers.authority/1` |
| `frame.request_id_invalid` | empty, > 128 bytes, or contains control characters |
| `frame.request_id_duplicate` | `request_id` already used in this session |
| `frame.request_id_space_exhausted` | > 8192 distinct `request_id` values this session |
| `frame.unknown_operation` | `operation` outside the closed set |
| `frame.payload_oversized` | serialised `payload` > 512 KiB (also a required string field > 4096 bytes) |
| `frame.payload_invalid` | payload failed semantic validation; see payload-parser sub-codes |
| `frame.forbidden_authority_key` | forbidden authority key in `prepare` payload or nested `plan` |
| `frame.unexpected_payload` | `hello` received payload fields |
| `frame.timeout` | partial frame stalled 30 s without completing; partial discarded, session continues |
| `frame.response_serialize_failed` | internal fallback when a response could not be serialised |

### `prepare.*` — PREPARE operation

| Code | Condition |
| --- | --- |
| `prepare.tether_not_found` | `tether_id`/`tether_version` not configured exactly once (also raised during COMMIT's Tether re-selection) |
| `prepare.invalid_plan` | `plan.id` or `plan.actions` missing, or a required action field invalid |
| `prepare.action_not_found` | `action_id` not present in `plan.actions` |
| `prepare.duplicate_identity` | identical evaluation material already prepared this session |
| `prepare.approval_failed` | exact approval request could not be created |

### `commit.*` — COMMIT operation

| Code | Condition |
| --- | --- |
| `commit.unknown_prepared` | `prepared_id` not known to this session |
| `commit.already_committed` | preparation already committed |
| `commit.deny` | fresh current authority denies (including after an approval) |
| `commit.unavailable` | fresh authority cannot admit: config load/prepare failure, capability resolution failure, empty provider availability, or an `unavailable` evaluation |
| `commit.missing_pin` | `bridge_capability_version` missing on the action |
| `commit.identity_mismatch` | allow identity does not match the resolved capability |
| `commit.approval_required` | policy evaluates Ask but no `approval_id` is available |
| `commit.approval_not_ready` | approval missing, pending, denied, cancelled, consumed, unknown, or bound to different material |
| `commit.approval_failed` | approval precheck errored |
| `commit.approval_consume_failed` | approved record could not be consumed or its consumption not recorded durably; manual resolution required |
| `commit.replay_unavailable` | durable replay admission unavailable |
| `commit.replay_blocked` | fresh admission refused; `error.data` carries `{replay, execution_id}` |
| `commit.intent_failed` | durable replay intent or Trail intent could not be recorded |
| `commit.armed_failed` | replay could not mark the execution armed |

### `approval.*` — approval records

| Code | Condition |
| --- | --- |
| `approval.decision_refused` | unknown `approval_id`, or illegal state transition (e.g. deciding a non-`pending` record) |
| `approval.invalid_decision` | `decision` not in {`approve`, `deny`, `cancel`}; surfaces on the wire as `frame.payload_invalid` with this code prefixed in `error.message` (see below) |

### `outcome.*` — OUTCOME operation

| Code | Condition |
| --- | --- |
| `outcome.unknown_execution` | `execution_id` is not a committed Gate execution (typical after a Gate restart) |
| `outcome.conflict` | a different classification was already recorded |
| `outcome.not_attempted` | `attempted: false` for a committed dispatch |
| `outcome.identity_collapse` | `external_execution_identity` equals the Tethers `action_id` |
| `outcome.trail_failed` | outcome could not be recorded durably in the Trail |
| `outcome.digest_failed` | durable outcome digest could not be computed |

Payload-parser sub-codes emitted under `frame.payload_invalid` with the
specific code prefixed in `error.message` (`"<sub-code>: <detail>"`):

| Sub-code | Condition |
| --- | --- |
| `frame.empty_field` | a required string payload field is empty |
| `prepare.unknown_observation` | unknown key inside `observations` |
| `prepare.invalid_observation` | `available_provider_identities` malformed, entry > 256 bytes, or > 64 entries |
| `approval.invalid_decision` | `decision` not in {`approve`, `deny`, `cancel`} |
| `commit.invalid_approval` | `approval_id` present but empty |
| `outcome.invalid_classification` | `classification` not in {`succeeded`, `failed`, `uncertain`} |
| `outcome.missing_result` | `succeeded` without `result` |
| `outcome.missing_error` | `failed` without `error` |
| `outcome.conflicting_fields` | both `result` and `error` present |

### `gate.*` — session/runtime layer (in-band)

| Code | Condition |
| --- | --- |
| `gate.config_unavailable` | runtime configuration cannot be read, digested, loaded, or prepared (fails closed) |
| `gate.trail_unavailable` | Trail file cannot be opened |

### CLI envelope codes (`GATE_*`, out-of-band)

Emitted as the `tethers.cli/1` error envelope before or instead of any
protocol session; the process exits non-zero. Not part of the in-band frame
protocol.

| Code | Condition |
| --- | --- |
| `GATE_STDIO_REQUIRED` | `--stdio` not supplied |
| `GATE_PATH_NOT_ABSOLUTE` | `--config`, `--trail`, or `--host-data-root` is relative |
| `GATE_CONFIG_NOT_FOUND` | `--config` path is not an existing file |
| `GATE_TRAIL_UNAVAILABLE` | trail directory cannot be created |
| `GATE_HOST_DATA_UNAVAILABLE` | host data root cannot be created or hardened |
| `GATE_REPLAY_UNAVAILABLE` | durable replay state cannot be provisioned |
| `GATE_IO_FAILED` | stdin/stdout transport failure |

## Session lifecycle

1. CLI validates flags/paths and provisions the host-data root (fail closed
   on any violation).
2. Host sends `hello`; Gate answers protocol identity with
   `authority_granted: false`.
3. Per effect: `prepare` → (optional `approval_decision`) → `commit` →
   Host dispatches physically → `outcome`.
4. `status` at any time for bounded reconciliation; `shutdown` to end.
5. Every response carries `provider_invocations: 0`.
