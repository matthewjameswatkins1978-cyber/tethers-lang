# J24G Installation Request Contract

## Purpose

J24G defines the only public machine-readable request accepted by the future
Plug installation pipeline. It is deliberately smaller than trust,
conformance, approval, installed-record, and enablement evidence.

The request expresses one human decision:

> Process this exact immutable candidate through supervised conformance and,
> only if every later gate succeeds, install it disabled.

J24G parses and validates that decision. It does not look up the candidate,
grant trust, launch a provider, run conformance, approve installation, copy a
payload, publish installed state, or enable a Plug.

## Exact public JSON

```json
{
  "schema": "tethers.plug-install/1",
  "candidate_id": "3d846d40-01fc-4e1e-b77d-83944dbed76f",
  "trust": {
    "scope": "exact_candidate"
  },
  "conformance": {
    "allow_non_isolated_supervised_execution": true
  },
  "installation": {
    "target_state": "disabled"
  }
}
```

No field is optional. No additional field is permitted at any depth.

## Public Rust seam

Add `src/installation_request.rs` and export it from `lib.rs`.

```rust
pub const INSTALLATION_REQUEST_SCHEMA: &str = "tethers.plug-install/1";
pub const INSTALLATION_REQUEST_MAX_BYTES: usize = 16 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallationRequest {
    pub schema: String,
    pub candidate_id: String,
    pub trust: InstallationTrustRequest,
    pub conformance: InstallationConformanceRequest,
    pub installation: InstallationTargetRequest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallationTrustRequest {
    pub scope: InstallationTrustScope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallationTrustScope {
    ExactCandidate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallationConformanceRequest {
    pub allow_non_isolated_supervised_execution: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallationTargetRequest {
    pub target_state: InstallationTargetState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallationTargetState {
    Disabled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallationRequestError {
    pub code: &'static str,
    pub message: &'static str,
    pub field: Option<String>,
}

pub fn load_installation_request(
    path: &Path,
) -> Result<InstallationRequest, InstallationRequestError>;

pub fn parse_installation_request_bytes(
    bytes: &[u8],
) -> Result<InstallationRequest, InstallationRequestError>;
```

Implement `Display` and `std::error::Error` for
`InstallationRequestError`. Keep its constructor private.

## Stable error codes

Only two public codes exist in J24G:

- `installation_request_io`
- `installation_request_invalid`

`installation_request_io` is used only when metadata, opening, or reading the
request file fails.

Every path or content validation failure uses
`installation_request_invalid`.

Errors must never contain an operating-system path, raw JSON text, candidate
record contents, or a platform-specific I/O message.

## File loading boundary

`load_installation_request` must:

1. require an absolute path;
2. inspect the final entry with `fs::symlink_metadata`;
3. require an ordinary regular file and reject a final symlink or directory;
4. open the file read-only;
5. read at most `INSTALLATION_REQUEST_MAX_BYTES + 1` bytes using a bounded
   reader;
6. return `installation_request_invalid` when the bounded result exceeds the
   limit;
7. pass the bytes to `parse_installation_request_bytes`.

Do not call `fs::read`, because it allocates according to the complete hostile
file. Do not canonicalise the request path. The request file is read once and
its contents, not its path, become the future authority.

Stable file-boundary messages:

| Condition | Code | Message | Field |
|---|---|---|---|
| relative path | `installation_request_invalid` | `installation request path must be absolute` | none |
| final entry is not an ordinary file | `installation_request_invalid` | `installation request path must name an ordinary file` | none |
| metadata/open/read failure | `installation_request_io` | `cannot read installation request` | none |
| larger than 16 KiB | `installation_request_invalid` | `installation request exceeds 16 KiB limit` | none |

## Byte and JSON boundary

`parse_installation_request_bytes` must validate in this order:

1. reject more than 16 KiB;
2. reject a UTF-8 BOM;
3. require valid UTF-8;
4. parse through `crate::manifest::parse_value_no_dupes`;
5. require exactly one complete JSON value;
6. validate the exact object shape and values below.

`parse_value_no_dupes` already rejects duplicate keys recursively and trailing
non-whitespace content. Reuse it. Do not add another JSON parser or custom
Serde visitor.

Stable byte/JSON messages:

| Condition | Message | Field |
|---|---|---|
| larger than 16 KiB | `installation request exceeds 16 KiB limit` | none |
| UTF-8 BOM | `installation request contains UTF-8 BOM` | none |
| invalid UTF-8 | `installation request is not valid UTF-8` | none |
| malformed JSON, duplicate key, or trailing JSON value | `installation request must be valid JSON with no duplicate keys or trailing content` | none |
| expected object | `value must be an object` | pointer to the value |
| missing field | `required field is missing` | pointer to the missing field |
| unknown field | `field is not permitted in installation request` | pointer to the unknown field |
| expected string | `value must be a string` | pointer to the value |
| expected boolean | `value must be a boolean` | pointer to the value |

Use RFC 6901 escaping when constructing field pointers.

## Exact semantic validation

The root object permits exactly:

```text
schema
candidate_id
trust
conformance
installation
```

`schema` must equal `tethers.plug-install/1`.

`candidate_id` must be a canonical lowercase hyphenated UUID. Parse it with
`Uuid::parse_str`, then require:

```rust
parsed.hyphenated().to_string() == supplied
```

The `trust` object permits exactly `scope`, which must equal
`exact_candidate`.

The `conformance` object permits exactly
`allow_non_isolated_supervised_execution`, which must be the JSON boolean
`true`. A string such as `"true"` is invalid. `false` is invalid.

The `installation` object permits exactly `target_state`, which must equal
`disabled`.

Stable semantic messages:

| Pointer | Message |
|---|---|
| `/schema` | `schema must be exactly "tethers.plug-install/1"` |
| `/candidate_id` | `candidate_id must be a canonical lowercase hyphenated UUID` |
| `/trust/scope` | `trust scope must be exactly "exact_candidate"` |
| `/conformance/allow_non_isolated_supervised_execution` | `non-isolated supervised execution must be explicitly approved` |
| `/installation/target_state` | `installation target_state must be exactly "disabled"` |

All of these use code `installation_request_invalid`.

## Suggested implementation shape

Use small private helpers similar to `run_input.rs`:

```rust
fn required_object(...)
fn required_value(...)
fn required_string(...)
fn required_bool(...)
fn reject_unknown(...)
fn child_pointer(...)
```

The core parser should read almost as a checklist:

```rust
pub fn parse_installation_request_bytes(bytes: &[u8]) -> Result<..., ...> {
    enforce_size(bytes)?;
    reject_bom(bytes)?;
    let text = require_utf8(bytes)?;
    let value = parse_value_no_dupes(text).map_err(|_| invalid_json())?;

    let root = required_object(&value, "")?;
    reject_unknown(root, "", &[...])?;

    let schema = required_string(root, "schema", "")?;
    validate_schema(&schema)?;

    let candidate_id = required_string(root, "candidate_id", "")?;
    validate_candidate_id(&candidate_id)?;

    // Parse and validate trust, conformance, and installation objects.

    Ok(InstallationRequest { ... })
}
```

Do not preserve the original `serde_json::Value`. J24H receives only the typed,
validated request.

## Required evidence

Unit and integration tests must prove:

- the exact valid document parses;
- the loader accepts an absolute ordinary file;
- exactly 16 KiB is accepted when the extra bytes are JSON whitespace;
- 16 KiB plus one byte is rejected;
- BOM, invalid UTF-8, malformed JSON, trailing JSON, root duplicates, and nested
  duplicates are rejected;
- every missing field is rejected with the exact pointer;
- unknown fields at the root and at every nested level are rejected;
- wrong root, object, string, and boolean types are rejected;
- unsupported schema is rejected;
- non-canonical, uppercase, simple, braced, and invalid UUIDs are rejected;
- any trust scope other than `exact_candidate` is rejected;
- absent, string, or false supervised-execution approval is rejected;
- any target state other than `disabled` is rejected;
- relative, missing, and non-regular request paths are rejected with the frozen
  codes and messages;
- parsing and loading create, remove, or modify no filesystem path.

## Non-goals

J24G does not:

- add `plug install` or any CLI placeholder;
- access candidate, quarantine, trust, conformance, approval, installed, or
  enablement stores;
- compute an installation plan or request digest;
- acquire a lock;
- launch provider code;
- copy or publish payloads;
- create any evidence;
- add publisher trust;
- allow installation to an enabled state;
- change package or candidate schemas;
- change dependencies or lockfiles.

The next packet, J24H, will consume this typed request in a read-only
installation reconciliation planner.