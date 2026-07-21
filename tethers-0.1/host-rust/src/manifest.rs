// Columbo capability manifest - data types, parsing, and structured errors.
//
// C1a1: types and error codes.
// C1a2: strict JSON parsing, recursive duplicate-key rejection, unknown-field rejection.

use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use std::collections::HashSet;
use std::fmt;

// ---------------------------------------------------------------------------
// Duplicate-key-detecting JSON value parser
// ---------------------------------------------------------------------------

/// Parse a JSON text into a `serde_json::Value`, rejecting duplicate keys in
/// any object at any depth.
///
/// Uses `serde_json`'s own tokenizer and number parser - this is not a
/// homemade JSON parser.  Only the map-key collection is intercepted via a
/// `DeserializeSeed` that detects duplicates recursively.
fn parse_value_no_dupes(json: &str) -> Result<serde_json::Value, serde_json::Error> {
    let mut de = serde_json::Deserializer::from_str(json);
    let value = DedupValueSeed.deserialize(&mut de)?;
    de.end()?;
    Ok(value)
}

/// A `DeserializeSeed` that produces `serde_json::Value` with duplicate-key
/// detection in every object at every depth.
#[derive(Clone, Copy)]
struct DedupValueSeed;

// Marker visitor for the seed - identical to the inner visitor except it
// produces `serde_json::Value` directly.
struct DedupValueVisitor;

impl<'de> Visitor<'de> for DedupValueVisitor {
    type Value = serde_json::Value;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("any JSON value")
    }

    fn visit_bool<E: de::Error>(self, v: bool) -> Result<serde_json::Value, E> {
        Ok(serde_json::Value::Bool(v))
    }

    fn visit_i64<E: de::Error>(self, v: i64) -> Result<serde_json::Value, E> {
        Ok(serde_json::Value::Number(v.into()))
    }

    fn visit_u64<E: de::Error>(self, v: u64) -> Result<serde_json::Value, E> {
        Ok(serde_json::Value::Number(v.into()))
    }

    fn visit_f64<E: de::Error>(self, v: f64) -> Result<serde_json::Value, E> {
        let num: serde_json::Number =
            serde_json::from_str(&v.to_string()).map_err(de::Error::custom)?;
        Ok(serde_json::Value::Number(num))
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<serde_json::Value, E> {
        Ok(serde_json::Value::String(v.to_owned()))
    }

    fn visit_string<E: de::Error>(self, v: String) -> Result<serde_json::Value, E> {
        Ok(serde_json::Value::String(v))
    }

    fn visit_none<E: de::Error>(self) -> Result<serde_json::Value, E> {
        Ok(serde_json::Value::Null)
    }

    fn visit_unit<E: de::Error>(self) -> Result<serde_json::Value, E> {
        Ok(serde_json::Value::Null)
    }

    fn visit_map<A>(self, mut map: A) -> Result<serde_json::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut fields = Vec::new();
        let mut seen = HashSet::new();
        while let Some(key) = map.next_key::<String>()? {
            if !seen.insert(key.clone()) {
                return Err(de::Error::custom(format!(
                    "duplicate key in object: {}",
                    key
                )));
            }
            let value = map.next_value_seed(DedupValueSeed)?;
            fields.push((key, value));
        }
        Ok(serde_json::Value::Object(fields.into_iter().collect()))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<serde_json::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(v) = seq.next_element_seed(DedupValueSeed)? {
            values.push(v);
        }
        Ok(serde_json::Value::Array(values))
    }
}

impl<'de> DeserializeSeed<'de> for DedupValueSeed {
    type Value = serde_json::Value;

    fn deserialize<D>(self, deserializer: D) -> Result<serde_json::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(DedupValueVisitor)
    }
}

// ---------------------------------------------------------------------------
// Manifest format version
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestFormatVersion(pub String);

impl ManifestFormatVersion {
    pub const V1_0: &'static str = "1.0";
}

// ---------------------------------------------------------------------------
// Behaviour enums (settled values)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reversibility {
    Reversible,
    Compensatable,
    Irreversible,
}

impl Reversibility {
    fn from_str(s: &str, field: &str) -> Result<Self, ManifestError> {
        match s {
            "reversible" => Ok(Reversibility::Reversible),
            "compensatable" => Ok(Reversibility::Compensatable),
            "irreversible" => Ok(Reversibility::Irreversible),
            _ => Err(ManifestError::with_field(
                ManifestErrorCode::InvalidValue,
                format!("unknown reversibility: {}", s),
                field,
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Determinism {
    Deterministic,
    NonDeterministic,
}

impl Determinism {
    fn from_str(s: &str, field: &str) -> Result<Self, ManifestError> {
        match s {
            "deterministic" => Ok(Determinism::Deterministic),
            "non_deterministic" => Ok(Determinism::NonDeterministic),
            _ => Err(ManifestError::with_field(
                ManifestErrorCode::InvalidValue,
                format!("unknown determinism: {}", s),
                field,
            )),
        }
    }
}

// ---------------------------------------------------------------------------
// Idempotency
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Idempotency {
    ArgumentKey {
        argument_name: String,
        key_source: String,
        description: Option<String>,
    },
    ServerDedup {
        dedup_key: String,
        dedup_scope: String,
        dedup_lifetime: String,
        evidence: String,
        description: Option<String>,
    },
    NoMechanism,
}

// ---------------------------------------------------------------------------
// Confirmation policy
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfirmationPolicy {
    pub standing_permitted: bool,
    pub per_call_required: bool,
    pub description: Option<String>,
}

// ---------------------------------------------------------------------------
// Retry policy
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryCondition {
    OutcomeUnknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub backoff_ms: u64,
    pub allowed_on: Vec<RetryCondition>,
    pub requires_idempotency_proof: bool,
}

// ---------------------------------------------------------------------------
// Permission scope
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionScope {
    PathPrefix { allowed_prefixes: Vec<String> },
    Repository { allowed_repositories: Vec<String> },
    Calendar { allowed_calendars: Vec<String> },
    Unrestricted,
}

// ---------------------------------------------------------------------------
// Provider identity (host-assigned only)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentitySource {
    HostConfiguration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderIdentity {
    pub identity: String,
    pub display_name: String,
    pub identity_source: IdentitySource,
    pub description: Option<String>,
}

// ---------------------------------------------------------------------------
// Binding
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingKind {
    Mcp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterBinding {
    pub name: String,
    pub version: String,
    pub digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binding {
    pub kind: BindingKind,
    pub server_name: String,
    pub tool_name: String,
    pub adapter: Option<AdapterBinding>,
}

// ---------------------------------------------------------------------------
// The complete trusted manifest
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrustedManifest {
    pub manifest_format_version: String,
    pub capability_name: String,
    pub capability_version: u32,
    pub title: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub output_schema: serde_json::Value,
    pub effects: Vec<String>,
    pub permission_scope: PermissionScope,
    pub reversibility: Reversibility,
    pub determinism: Determinism,
    pub idempotency: Idempotency,
    pub confirmation_policy: ConfirmationPolicy,
    pub timeout_ms: u64,
    pub retry_policy: RetryPolicy,
    pub provider: ProviderIdentity,
    pub binding: Binding,
    pub digest: Option<String>,
}

/// Known top-level authoritative keys.  Any top-level key not in this set is
/// rejected as `UnknownField`.  `input_schema` and `output_schema` are opaque
/// and accept arbitrary nested keys.
const TOP_LEVEL_KEYS: &[&str] = &[
    "manifest_format_version",
    "capability_name",
    "capability_version",
    "title",
    "description",
    "input_schema",
    "output_schema",
    "effects",
    "permission_scope",
    "reversibility",
    "determinism",
    "idempotency",
    "confirmation_policy",
    "timeout_ms",
    "retry_policy",
    "provider",
    "binding",
    "digest",
];

// ---------------------------------------------------------------------------
// Parsing helpers
// ---------------------------------------------------------------------------

fn json_pointer_child(parent: &str, segment: &str) -> String {
    let escaped = segment.replace('~', "~0").replace('/', "~1");
    if parent.is_empty() {
        format!("/{}", escaped)
    } else {
        format!("{}/{}", parent, escaped)
    }
}

/// Require `field` to be a JSON string, returning a reference.
fn require_str<'v>(
    obj: &'v serde_json::Map<String, serde_json::Value>,
    field: &str,
    pointer: &str,
) -> Result<&'v str, ManifestError> {
    match obj.get(field) {
        Some(serde_json::Value::String(s)) => Ok(s.as_str()),
        Some(_) => Err(ManifestError::with_field(
            ManifestErrorCode::InvalidType,
            format!("{} must be a string", field),
            json_pointer_child(pointer, field),
        )),
        None => Err(ManifestError::with_field(
            ManifestErrorCode::MissingField,
            format!("missing required field: {}", field),
            json_pointer_child(pointer, field),
        )),
    }
}

/// Require `field` to be a JSON boolean.
fn require_bool(
    obj: &serde_json::Map<String, serde_json::Value>,
    field: &str,
    pointer: &str,
) -> Result<bool, ManifestError> {
    match obj.get(field) {
        Some(serde_json::Value::Bool(b)) => Ok(*b),
        Some(_) => Err(ManifestError::with_field(
            ManifestErrorCode::InvalidType,
            format!("{} must be a boolean", field),
            json_pointer_child(pointer, field),
        )),
        None => Err(ManifestError::with_field(
            ManifestErrorCode::MissingField,
            format!("missing required field: {}", field),
            json_pointer_child(pointer, field),
        )),
    }
}

/// Require `field` to be a JSON number, returned as u64.
fn require_u64(
    obj: &serde_json::Map<String, serde_json::Value>,
    field: &str,
    pointer: &str,
) -> Result<u64, ManifestError> {
    match obj.get(field) {
        Some(serde_json::Value::Number(n)) => n.as_u64().ok_or_else(|| {
            ManifestError::with_field(
                ManifestErrorCode::InvalidValue,
                format!("{} must be a non-negative integer", field),
                json_pointer_child(pointer, field),
            )
        }),
        Some(_) => Err(ManifestError::with_field(
            ManifestErrorCode::InvalidType,
            format!("{} must be a number", field),
            json_pointer_child(pointer, field),
        )),
        None => Err(ManifestError::with_field(
            ManifestErrorCode::MissingField,
            format!("missing required field: {}", field),
            json_pointer_child(pointer, field),
        )),
    }
}

/// Require `field` to be a JSON number, returned as u32.
fn require_u32(
    obj: &serde_json::Map<String, serde_json::Value>,
    field: &str,
    pointer: &str,
) -> Result<u32, ManifestError> {
    let v = require_u64(obj, field, pointer)?;
    u32::try_from(v).map_err(|_| {
        ManifestError::with_field(
            ManifestErrorCode::InvalidValue,
            format!("{} out of range", field),
            json_pointer_child(pointer, field),
        )
    })
}

/// Require `field` to be a JSON array of strings.
fn require_string_array(
    obj: &serde_json::Map<String, serde_json::Value>,
    field: &str,
    pointer: &str,
) -> Result<Vec<String>, ManifestError> {
    match obj.get(field) {
        Some(serde_json::Value::Array(arr)) => {
            let mut out = Vec::with_capacity(arr.len());
            for (i, v) in arr.iter().enumerate() {
                match v {
                    serde_json::Value::String(s) => out.push(s.clone()),
                    _ => {
                        return Err(ManifestError::with_field(
                            ManifestErrorCode::InvalidType,
                            format!("{}[{}] must be a string", field, i),
                            format!("{}/{}", json_pointer_child(pointer, field), i),
                        ));
                    }
                }
            }
            Ok(out)
        }
        Some(_) => Err(ManifestError::with_field(
            ManifestErrorCode::InvalidType,
            format!("{} must be an array", field),
            json_pointer_child(pointer, field),
        )),
        None => Err(ManifestError::with_field(
            ManifestErrorCode::MissingField,
            format!("missing required field: {}", field),
            json_pointer_child(pointer, field),
        )),
    }
}

/// Require `field` to be a JSON object, returning the inner map.
fn require_object<'v>(
    obj: &'v serde_json::Map<String, serde_json::Value>,
    field: &str,
    pointer: &str,
) -> Result<&'v serde_json::Map<String, serde_json::Value>, ManifestError> {
    match obj.get(field) {
        Some(serde_json::Value::Object(m)) => Ok(m),
        Some(_) => Err(ManifestError::with_field(
            ManifestErrorCode::InvalidType,
            format!("{} must be an object", field),
            json_pointer_child(pointer, field),
        )),
        None => Err(ManifestError::with_field(
            ManifestErrorCode::MissingField,
            format!("missing required field: {}", field),
            json_pointer_child(pointer, field),
        )),
    }
}

/// Require `field` to be present and a JSON value (any type).
fn require_value<'v>(
    obj: &'v serde_json::Map<String, serde_json::Value>,
    field: &str,
    pointer: &str,
) -> Result<&'v serde_json::Value, ManifestError> {
    match obj.get(field) {
        Some(v) => Ok(v),
        None => Err(ManifestError::with_field(
            ManifestErrorCode::MissingField,
            format!("missing required field: {}", field),
            json_pointer_child(pointer, field),
        )),
    }
}

fn optional_string_at(
    obj: &serde_json::Map<String, serde_json::Value>,
    field: &str,
    pointer: &str,
) -> Result<Option<String>, ManifestError> {
    match obj.get(field) {
        Some(serde_json::Value::String(s)) => Ok(Some(s.clone())),
        Some(serde_json::Value::Null) | None => Ok(None),
        Some(_) => Err(ManifestError::with_field(
            ManifestErrorCode::InvalidType,
            format!("{} must be a string or null", field),
            json_pointer_child(pointer, field),
        )),
    }
}

/// Reject any keys in `obj` that are not listed in `allowed`, unless the
/// object is an opaque schema (checked by caller).
fn reject_unknown_keys(
    obj: &serde_json::Map<String, serde_json::Value>,
    allowed: &[&str],
    pointer: &str,
) -> Result<(), ManifestError> {
    for key in obj.keys() {
        if !allowed.contains(&key.as_str()) {
            return Err(ManifestError::with_field(
                ManifestErrorCode::UnknownField,
                format!("unknown field: {}", key),
                json_pointer_child(pointer, key),
            ));
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Sub-parsers
// ---------------------------------------------------------------------------

fn parse_confirmation_policy(
    obj: &serde_json::Map<String, serde_json::Value>,
    pointer: &str,
) -> Result<ConfirmationPolicy, ManifestError> {
    reject_unknown_keys(
        obj,
        &["standing_permitted", "per_call_required", "description"],
        pointer,
    )?;
    Ok(ConfirmationPolicy {
        standing_permitted: require_bool(obj, "standing_permitted", pointer)?,
        per_call_required: require_bool(obj, "per_call_required", pointer)?,
        description: optional_string_at(obj, "description", pointer)?,
    })
}

fn parse_retry_policy(
    obj: &serde_json::Map<String, serde_json::Value>,
    pointer: &str,
) -> Result<RetryPolicy, ManifestError> {
    reject_unknown_keys(
        obj,
        &[
            "max_retries",
            "backoff_ms",
            "allowed_on",
            "requires_idempotency_proof",
        ],
        pointer,
    )?;
    let allowed_on_raw = require_string_array(obj, "allowed_on", pointer)?;
    let mut allowed_on = Vec::new();
    for s in &allowed_on_raw {
        match s.as_str() {
            "outcome_unknown" => allowed_on.push(RetryCondition::OutcomeUnknown),
            other => {
                return Err(ManifestError::with_field(
                    ManifestErrorCode::InvalidValue,
                    format!("unknown retry condition: {}", other),
                    json_pointer_child(pointer, "allowed_on"),
                ));
            }
        }
    }
    Ok(RetryPolicy {
        max_retries: require_u32(obj, "max_retries", pointer)?,
        backoff_ms: require_u64(obj, "backoff_ms", pointer)?,
        allowed_on,
        requires_idempotency_proof: require_bool(obj, "requires_idempotency_proof", pointer)?,
    })
}

fn parse_permission_scope(
    value: &serde_json::Value,
    pointer: &str,
) -> Result<PermissionScope, ManifestError> {
    match value {
        serde_json::Value::Null => Ok(PermissionScope::Unrestricted),
        serde_json::Value::Object(obj) => {
            let kind = require_str(obj, "kind", pointer)?;
            match kind {
                "path_prefix" => {
                    reject_unknown_keys(obj, &["kind", "allowed_prefixes"], pointer)?;
                    Ok(PermissionScope::PathPrefix {
                        allowed_prefixes: require_string_array(obj, "allowed_prefixes", pointer)?,
                    })
                }
                "repository" => {
                    reject_unknown_keys(obj, &["kind", "allowed_repositories"], pointer)?;
                    Ok(PermissionScope::Repository {
                        allowed_repositories: require_string_array(
                            obj,
                            "allowed_repositories",
                            pointer,
                        )?,
                    })
                }
                "calendar" => {
                    reject_unknown_keys(obj, &["kind", "allowed_calendars"], pointer)?;
                    Ok(PermissionScope::Calendar {
                        allowed_calendars: require_string_array(obj, "allowed_calendars", pointer)?,
                    })
                }
                _ => Err(ManifestError::with_field(
                    ManifestErrorCode::InvalidScope,
                    format!("unknown scope kind: {}", kind),
                    json_pointer_child(pointer, "kind"),
                )),
            }
        }
        _ => Err(ManifestError::with_field(
            ManifestErrorCode::InvalidType,
            "permission_scope must be an object or null",
            pointer,
        )),
    }
}

fn parse_idempotency(
    obj: &serde_json::Map<String, serde_json::Value>,
    pointer: &str,
) -> Result<Idempotency, ManifestError> {
    let mechanism = require_str(obj, "mechanism", pointer)?;
    match mechanism {
        "argument_key" => {
            reject_unknown_keys(
                obj,
                &["mechanism", "argument_name", "key_source", "description"],
                pointer,
            )?;
            Ok(Idempotency::ArgumentKey {
                argument_name: require_str(obj, "argument_name", pointer)?.to_string(),
                key_source: require_str(obj, "key_source", pointer)?.to_string(),
                description: optional_string_at(obj, "description", pointer)?,
            })
        }
        "server_dedup" => {
            reject_unknown_keys(
                obj,
                &[
                    "mechanism",
                    "dedup_key",
                    "dedup_scope",
                    "dedup_lifetime",
                    "evidence",
                    "description",
                ],
                pointer,
            )?;
            Ok(Idempotency::ServerDedup {
                dedup_key: require_str(obj, "dedup_key", pointer)?.to_string(),
                dedup_scope: require_str(obj, "dedup_scope", pointer)?.to_string(),
                dedup_lifetime: require_str(obj, "dedup_lifetime", pointer)?.to_string(),
                evidence: require_str(obj, "evidence", pointer)?.to_string(),
                description: optional_string_at(obj, "description", pointer)?,
            })
        }
        "none" => {
            reject_unknown_keys(obj, &["mechanism"], pointer)?;
            Ok(Idempotency::NoMechanism)
        }
        _ => Err(ManifestError::with_field(
            ManifestErrorCode::InvalidValue,
            format!("unknown idempotency mechanism: {}", mechanism),
            json_pointer_child(pointer, "mechanism"),
        )),
    }
}

fn parse_provider_identity(
    obj: &serde_json::Map<String, serde_json::Value>,
    pointer: &str,
) -> Result<ProviderIdentity, ManifestError> {
    reject_unknown_keys(
        obj,
        &["identity", "display_name", "identity_source", "description"],
        pointer,
    )?;
    let identity_source_str = require_str(obj, "identity_source", pointer)?;
    let identity_source = match identity_source_str {
        "host_configuration" => IdentitySource::HostConfiguration,
        _ => {
            return Err(ManifestError::with_field(
                ManifestErrorCode::InvalidProvider,
                format!(
                    "identity_source must be \"host_configuration\", got: {}",
                    identity_source_str
                ),
                json_pointer_child(pointer, "identity_source"),
            ));
        }
    };
    Ok(ProviderIdentity {
        identity: require_str(obj, "identity", pointer)?.to_string(),
        display_name: require_str(obj, "display_name", pointer)?.to_string(),
        identity_source,
        description: optional_string_at(obj, "description", pointer)?,
    })
}

fn parse_adapter_binding(
    obj: &serde_json::Map<String, serde_json::Value>,
    pointer: &str,
) -> Result<AdapterBinding, ManifestError> {
    reject_unknown_keys(obj, &["name", "version", "digest"], pointer)?;
    Ok(AdapterBinding {
        name: require_str(obj, "name", pointer)?.to_string(),
        version: require_str(obj, "version", pointer)?.to_string(),
        digest: require_str(obj, "digest", pointer)?.to_string(),
    })
}

fn parse_binding(
    obj: &serde_json::Map<String, serde_json::Value>,
    pointer: &str,
) -> Result<Binding, ManifestError> {
    reject_unknown_keys(
        obj,
        &["kind", "server_name", "tool_name", "adapter"],
        pointer,
    )?;
    let kind_str = require_str(obj, "kind", pointer)?;
    let kind = match kind_str {
        "mcp" => BindingKind::Mcp,
        _ => {
            return Err(ManifestError::with_field(
                ManifestErrorCode::InvalidBinding,
                format!("unknown binding kind: {}", kind_str),
                json_pointer_child(pointer, "kind"),
            ));
        }
    };
    let adapter = match obj.get("adapter") {
        Some(serde_json::Value::Null) | None => None,
        Some(serde_json::Value::Object(a)) => Some(parse_adapter_binding(
            a,
            &json_pointer_child(pointer, "adapter"),
        )?),
        Some(_) => {
            return Err(ManifestError::with_field(
                ManifestErrorCode::InvalidType,
                "adapter must be an object or null",
                json_pointer_child(pointer, "adapter"),
            ));
        }
    };
    Ok(Binding {
        kind,
        server_name: require_str(obj, "server_name", pointer)?.to_string(),
        tool_name: require_str(obj, "tool_name", pointer)?.to_string(),
        adapter,
    })
}

// ---------------------------------------------------------------------------
// Public parse entry point
// ---------------------------------------------------------------------------

impl TrustedManifest {
    /// Parse a manifest from JSON text.
    ///
    /// Performs recursive duplicate-key rejection, required-field checks,
    /// type validation, enum validation, and unknown-field rejection on all
    /// authoritative structures.  `input_schema` and `output_schema` are
    /// accepted as opaque JSON Schema objects with arbitrary keys.
    pub fn parse(json: &str) -> Result<Self, ManifestError> {
        let root = parse_value_no_dupes(json)
            .map_err(|e| ManifestError::new(ManifestErrorCode::InvalidJson, e.to_string()))?;

        let obj = match &root {
            serde_json::Value::Object(m) => m,
            _ => {
                return Err(ManifestError::new(
                    ManifestErrorCode::InvalidJson,
                    "manifest root must be a JSON object",
                ));
            }
        };

        reject_unknown_keys(obj, TOP_LEVEL_KEYS, "")?;

        // -- identity --
        let manifest_format_version = require_str(obj, "manifest_format_version", "")?.to_string();
        if manifest_format_version != ManifestFormatVersion::V1_0 {
            return Err(ManifestError::with_field(
                ManifestErrorCode::UnknownFormatVersion,
                format!(
                    "unknown manifest_format_version: {}",
                    manifest_format_version
                ),
                "/manifest_format_version",
            ));
        }
        let capability_name = require_str(obj, "capability_name", "")?.to_string();
        let capability_version = require_u32(obj, "capability_version", "")?;
        let title = require_str(obj, "title", "")?.to_string();
        let description = require_str(obj, "description", "")?.to_string();

        // -- schemas (opaque - no unknown-key rejection) --
        let input_schema = require_value(obj, "input_schema", "")?.clone();
        let output_schema = require_value(obj, "output_schema", "")?.clone();

        // -- effects --
        let effects = require_string_array(obj, "effects", "")?;

        // -- permission scope --
        let permission_scope = parse_permission_scope(
            require_value(obj, "permission_scope", "")?,
            "/permission_scope",
        )?;

        // -- behaviour --
        let reversibility =
            Reversibility::from_str(require_str(obj, "reversibility", "")?, "/reversibility")?;
        let determinism =
            Determinism::from_str(require_str(obj, "determinism", "")?, "/determinism")?;

        let idempotency_obj = require_object(obj, "idempotency", "")?;
        let idempotency = parse_idempotency(idempotency_obj, "/idempotency")?;

        let confirmation_obj = require_object(obj, "confirmation_policy", "")?;
        let confirmation_policy =
            parse_confirmation_policy(confirmation_obj, "/confirmation_policy")?;

        // -- execution policy --
        let timeout_ms = require_u64(obj, "timeout_ms", "")?;
        let retry_obj = require_object(obj, "retry_policy", "")?;
        let retry_policy = parse_retry_policy(retry_obj, "/retry_policy")?;

        // -- provider and binding --
        let provider_obj = require_object(obj, "provider", "")?;
        let provider = parse_provider_identity(provider_obj, "/provider")?;

        let binding_obj = require_object(obj, "binding", "")?;
        let binding = parse_binding(binding_obj, "/binding")?;

        // -- digest --
        let digest = match obj.get("digest") {
            Some(serde_json::Value::String(s)) => Some(s.clone()),
            Some(_) => {
                return Err(ManifestError::with_field(
                    ManifestErrorCode::InvalidType,
                    "digest must be a string",
                    "/digest",
                ));
            }
            None => None,
        };

        Ok(TrustedManifest {
            manifest_format_version,
            capability_name,
            capability_version,
            title,
            description,
            input_schema,
            output_schema,
            effects,
            permission_scope,
            reversibility,
            determinism,
            idempotency,
            confirmation_policy,
            timeout_ms,
            retry_policy,
            provider,
            binding,
            digest,
        })
    }
}

// ---------------------------------------------------------------------------
// Structured error model
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManifestErrorCode {
    InvalidJson,
    UnknownFormatVersion,
    MissingField,
    UnknownField,
    InvalidType,
    InvalidValue,
    InvalidScope,
    InvalidEffects,
    InvalidIdempotency,
    InvalidConfirmation,
    InvalidRetry,
    InvalidProvider,
    InvalidBinding,
    ContainsCredentials,
    DigestMismatch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestError {
    pub code: ManifestErrorCode,
    pub message: String,
    pub field: Option<String>,
}

impl ManifestError {
    pub fn new(code: ManifestErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            field: None,
        }
    }

    pub fn with_field(
        code: ManifestErrorCode,
        message: impl Into<String>,
        field: impl Into<String>,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            field: Some(field.into()),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // -- helper: minimal valid JSON manifest --
    fn minimal_manifest_json() -> serde_json::Value {
        json!({
            "manifest_format_version": "1.0",
            "capability_name": "notes.note.read",
            "capability_version": 1,
            "title": "Read a note",
            "description": "Read a note.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string" }
                },
                "required": ["path"],
                "additionalProperties": false
            },
            "output_schema": {
                "type": "object",
                "properties": {
                    "content": { "type": "string" }
                },
                "required": ["content"]
            },
            "effects": ["filesystem.read"],
            "permission_scope": {
                "kind": "path_prefix",
                "allowed_prefixes": ["projects/"]
            },
            "reversibility": "reversible",
            "determinism": "deterministic",
            "idempotency": {
                "mechanism": "none"
            },
            "confirmation_policy": {
                "standing_permitted": true,
                "per_call_required": false
            },
            "timeout_ms": 5000,
            "retry_policy": {
                "max_retries": 3,
                "backoff_ms": 500,
                "allowed_on": ["outcome_unknown"],
                "requires_idempotency_proof": false
            },
            "provider": {
                "identity": "obsidian-local",
                "display_name": "Obsidian (local vault)",
                "identity_source": "host_configuration",
                "description": "Host-assigned identity."
            },
            "binding": {
                "kind": "mcp",
                "server_name": "obsidian",
                "tool_name": "obsidian_read_note",
                "adapter": null
            }
        })
    }

    fn valid_write_json() -> serde_json::Value {
        json!({
            "manifest_format_version": "1.0",
            "capability_name": "notes.note.create",
            "capability_version": 1,
            "title": "Create a project note",
            "description": "Create a note.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "title": { "type": "string" },
                    "content": { "type": "string" },
                    "idempotency_key": { "type": "string" }
                },
                "required": ["title", "content"],
                "additionalProperties": false
            },
            "output_schema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "modified": { "type": "boolean" }
                },
                "required": ["path", "modified"]
            },
            "effects": ["filesystem.write"],
            "permission_scope": {
                "kind": "path_prefix",
                "allowed_prefixes": ["projects/"]
            },
            "reversibility": "compensatable",
            "determinism": "deterministic",
            "idempotency": {
                "mechanism": "argument_key",
                "argument_name": "idempotency_key",
                "key_source": "evaluation_id/action_id",
                "description": "Deduplicates by key."
            },
            "confirmation_policy": {
                "standing_permitted": false,
                "per_call_required": true,
                "description": "Per-call confirmation required."
            },
            "timeout_ms": 10000,
            "retry_policy": {
                "max_retries": 3,
                "backoff_ms": 1000,
                "allowed_on": ["outcome_unknown"],
                "requires_idempotency_proof": true
            },
            "provider": {
                "identity": "obsidian-local",
                "display_name": "Obsidian (local vault)",
                "identity_source": "host_configuration",
                "description": "Host-assigned."
            },
            "binding": {
                "kind": "mcp",
                "server_name": "obsidian",
                "tool_name": "obsidian_create_note",
                "adapter": null
            }
        })
    }

    // === parsing success ===

    #[test]
    fn parse_minimal_read_manifest() {
        let json = minimal_manifest_json().to_string();
        let m = TrustedManifest::parse(&json).unwrap();
        assert_eq!(m.capability_name, "notes.note.read");
        assert_eq!(m.capability_version, 1);
        assert!(matches!(m.reversibility, Reversibility::Reversible));
        assert!(matches!(m.idempotency, Idempotency::NoMechanism));
    }

    #[test]
    fn parse_write_manifest_with_argument_key() {
        let json = valid_write_json().to_string();
        let m = TrustedManifest::parse(&json).unwrap();
        assert_eq!(m.capability_name, "notes.note.create");
        assert!(m.confirmation_policy.per_call_required);
        match &m.idempotency {
            Idempotency::ArgumentKey {
                argument_name,
                key_source,
                ..
            } => {
                assert_eq!(argument_name, "idempotency_key");
                assert_eq!(key_source, "evaluation_id/action_id");
            }
            _ => panic!("expected ArgumentKey"),
        }
    }

    #[test]
    fn parse_server_dedup_idempotency() {
        let mut m = minimal_manifest_json();
        m["idempotency"] = json!({
            "mechanism": "server_dedup",
            "dedup_key": "request_content_hash",
            "dedup_scope": "24h_window",
            "dedup_lifetime": "24h",
            "evidence": "Server stores hash for 24h per documentation v2.1."
        });
        let parsed = TrustedManifest::parse(&m.to_string()).unwrap();
        match &parsed.idempotency {
            Idempotency::ServerDedup { dedup_key, .. } => {
                assert_eq!(dedup_key, "request_content_hash");
            }
            _ => panic!("expected ServerDedup"),
        }
    }

    // === malformed JSON ===

    #[test]
    fn reject_malformed_json() {
        let err = TrustedManifest::parse("{not json}").unwrap_err();
        assert_eq!(err.code, ManifestErrorCode::InvalidJson);
    }

    #[test]
    fn reject_trailing_json_after_manifest() {
        let json = format!("{} {{}}", minimal_manifest_json());
        let err = TrustedManifest::parse(&json).unwrap_err();
        assert_eq!(err.code, ManifestErrorCode::InvalidJson);
    }

    // === missing required field ===

    #[test]
    fn reject_missing_capability_name() {
        let mut m = minimal_manifest_json();
        m.as_object_mut().unwrap().remove("capability_name");
        let err = TrustedManifest::parse(&m.to_string()).unwrap_err();
        assert_eq!(err.code, ManifestErrorCode::MissingField);
        assert_eq!(err.field.as_deref(), Some("/capability_name"));
    }

    #[test]
    fn reject_unknown_manifest_format_version() {
        let mut m = minimal_manifest_json();
        m["manifest_format_version"] = json!("2.0");
        let err = TrustedManifest::parse(&m.to_string()).unwrap_err();
        assert_eq!(err.code, ManifestErrorCode::UnknownFormatVersion);
        assert_eq!(err.field.as_deref(), Some("/manifest_format_version"));
    }

    // === unknown top-level field ===

    #[test]
    fn reject_unknown_top_level_field() {
        let mut m = minimal_manifest_json();
        m.as_object_mut()
            .unwrap()
            .insert("extra_prop".into(), json!("value"));
        let err = TrustedManifest::parse(&m.to_string()).unwrap_err();
        assert_eq!(err.code, ManifestErrorCode::UnknownField);
        assert_eq!(err.field.as_deref(), Some("/extra_prop"));
    }

    #[test]
    fn unknown_field_pointer_escapes_json_pointer_segments() {
        let mut m = minimal_manifest_json();
        m.as_object_mut()
            .unwrap()
            .insert("a/b~c".into(), json!("value"));
        let err = TrustedManifest::parse(&m.to_string()).unwrap_err();
        assert_eq!(err.code, ManifestErrorCode::UnknownField);
        assert_eq!(err.field.as_deref(), Some("/a~1b~0c"));
    }

    // === unknown nested field in authoritative object ===

    #[test]
    fn reject_unknown_nested_field_in_confirmation() {
        let mut m = minimal_manifest_json();
        m["confirmation_policy"]
            .as_object_mut()
            .unwrap()
            .insert("auto_approve".into(), json!(true));
        let err = TrustedManifest::parse(&m.to_string()).unwrap_err();
        assert_eq!(err.code, ManifestErrorCode::UnknownField);
        assert!(err
            .field
            .as_deref()
            .unwrap()
            .contains("confirmation_policy"));
    }

    #[test]
    fn reject_unknown_provider_field() {
        let mut m = minimal_manifest_json();
        m["provider"]
            .as_object_mut()
            .unwrap()
            .insert("fingerprint".into(), json!("abc"));
        let err = TrustedManifest::parse(&m.to_string()).unwrap_err();
        assert_eq!(err.code, ManifestErrorCode::UnknownField);
        assert!(err.field.as_deref().unwrap().contains("provider"));
    }

    // === duplicate keys ===

    #[test]
    fn reject_duplicate_top_level_key() {
        // Duplicate keys in valid JSON are technically undefined, but
        // serde_json sees the last value.  We test by crafting raw text.
        let json = r#"{"manifest_format_version": "1.0", "capability_name": "a.b", "manifest_format_version": "2.0"}"#;
        let err = TrustedManifest::parse(json).unwrap_err();
        assert_eq!(err.code, ManifestErrorCode::InvalidJson);
    }

    #[test]
    fn reject_duplicate_key_in_nested_object() {
        let json = r#"{"manifest_format_version":"1.0","capability_name":"a.b","capability_version":1,"title":"t","description":"d","input_schema":{"type":"object"},"output_schema":{"type":"object"},"effects":["a"],"permission_scope":null,"reversibility":"reversible","determinism":"deterministic","idempotency":{"mechanism":"none"},"confirmation_policy":{"standing_permitted":true,"per_call_required":false,"per_call_required":false},"timeout_ms":5000,"retry_policy":{"max_retries":0,"backoff_ms":0,"allowed_on":[],"requires_idempotency_proof":false},"provider":{"identity":"x","display_name":"x","identity_source":"host_configuration"},"binding":{"kind":"mcp","server_name":"x","tool_name":"x","adapter":null}}"#;
        let err = TrustedManifest::parse(json).unwrap_err();
        assert_eq!(err.code, ManifestErrorCode::InvalidJson);
    }

    #[test]
    fn reject_duplicate_key_deep_in_input_schema() {
        let raw = format!(
            r#"{{"manifest_format_version":"1.0","capability_name":"a.b","capability_version":1,"title":"t","description":"d","input_schema":{{"type":"object","type":"object"}},"output_schema":{{"type":"object"}},"effects":["a"],"permission_scope":null,"reversibility":"reversible","determinism":"deterministic","idempotency":{{"mechanism":"none"}},"confirmation_policy":{{"standing_permitted":true,"per_call_required":false}},"timeout_ms":5000,"retry_policy":{{"max_retries":0,"backoff_ms":0,"allowed_on":[],"requires_idempotency_proof":false}},"provider":{{"identity":"x","display_name":"x","identity_source":"host_configuration"}},"binding":{{"kind":"mcp","server_name":"x","tool_name":"x","adapter":null}}}}"#,
        );
        let err = TrustedManifest::parse(&raw).unwrap_err();
        assert_eq!(err.code, ManifestErrorCode::InvalidJson);
    }

    #[test]
    fn reject_duplicate_key_deep_in_output_schema() {
        let raw = r#"{"manifest_format_version":"1.0","capability_name":"a.b","capability_version":1,"title":"t","description":"d","input_schema":{"type":"object"},"output_schema":{"required":["a"],"required":["b"]},"effects":["a"],"permission_scope":null,"reversibility":"reversible","determinism":"deterministic","idempotency":{"mechanism":"none"},"confirmation_policy":{"standing_permitted":true,"per_call_required":false},"timeout_ms":5000,"retry_policy":{"max_retries":0,"backoff_ms":0,"allowed_on":[],"requires_idempotency_proof":false},"provider":{"identity":"x","display_name":"x","identity_source":"host_configuration"},"binding":{"kind":"mcp","server_name":"x","tool_name":"x","adapter":null}}"#;
        let err = TrustedManifest::parse(raw).unwrap_err();
        assert_eq!(err.code, ManifestErrorCode::InvalidJson);
    }

    #[test]
    fn reject_escaped_equivalent_duplicate_key_in_schema() {
        let raw = r#"{"manifest_format_version":"1.0","capability_name":"a.b","capability_version":1,"title":"t","description":"d","input_schema":{"type":"object","\u0074ype":"object"},"output_schema":{"type":"object"},"effects":["a"],"permission_scope":null,"reversibility":"reversible","determinism":"deterministic","idempotency":{"mechanism":"none"},"confirmation_policy":{"standing_permitted":true,"per_call_required":false},"timeout_ms":5000,"retry_policy":{"max_retries":0,"backoff_ms":0,"allowed_on":[],"requires_idempotency_proof":false},"provider":{"identity":"x","display_name":"x","identity_source":"host_configuration"},"binding":{"kind":"mcp","server_name":"x","tool_name":"x","adapter":null}}"#;
        let err = TrustedManifest::parse(raw).unwrap_err();
        assert_eq!(err.code, ManifestErrorCode::InvalidJson);
    }

    // === repeated keys in sibling objects (valid) ===

    #[test]
    fn allow_repeated_keys_in_different_sibling_objects() {
        // `description` appears in `provider` and `confirmation_policy` - valid.
        let mut m = minimal_manifest_json();
        m["provider"]
            .as_object_mut()
            .unwrap()
            .insert("description".into(), json!("A"));
        m["confirmation_policy"]
            .as_object_mut()
            .unwrap()
            .insert("description".into(), json!("B"));
        let parsed = TrustedManifest::parse(&m.to_string()).unwrap();
        assert_eq!(parsed.provider.description, Some("A".to_string()));
        assert_eq!(
            parsed.confirmation_policy.description,
            Some("B".to_string())
        );
    }

    // === arbitrary schema keywords accepted ===

    #[test]
    fn input_schema_accepts_arbitrary_keywords() {
        let mut m = minimal_manifest_json();
        m["input_schema"] = json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "minLength": 1, "pattern": "^[a-z]+$" }
            },
            "required": ["path"],
            "additionalProperties": false,
            "$comment": "arbitrary extension"
        });
        let parsed = TrustedManifest::parse(&m.to_string()).unwrap();
        assert_eq!(parsed.input_schema["$comment"], "arbitrary extension");
    }

    // === invalid enum / type ===

    #[test]
    fn reject_invalid_reversibility_enum() {
        let mut m = minimal_manifest_json();
        m["reversibility"] = json!("undoable");
        let err = TrustedManifest::parse(&m.to_string()).unwrap_err();
        assert_eq!(err.code, ManifestErrorCode::InvalidValue);
        assert_eq!(err.field.as_deref(), Some("/reversibility"));
    }

    #[test]
    fn reject_invalid_idempotency_mechanism() {
        let mut m = minimal_manifest_json();
        m["idempotency"] = json!({"mechanism": "conditional"});
        let err = TrustedManifest::parse(&m.to_string()).unwrap_err();
        assert_eq!(err.code, ManifestErrorCode::InvalidValue);
        assert!(err.field.as_deref().unwrap().contains("idempotency"));
    }

    #[test]
    fn reject_invalid_permission_scope_kind() {
        let mut m = minimal_manifest_json();
        m["permission_scope"] = json!({"kind": "workspace"});
        let err = TrustedManifest::parse(&m.to_string()).unwrap_err();
        assert_eq!(err.code, ManifestErrorCode::InvalidScope);
        assert_eq!(err.field.as_deref(), Some("/permission_scope/kind"));
    }

    #[test]
    fn reject_non_string_effects_element() {
        let mut m = minimal_manifest_json();
        m["effects"] = json!(["filesystem.read", 42]);
        let err = TrustedManifest::parse(&m.to_string()).unwrap_err();
        assert_eq!(err.code, ManifestErrorCode::InvalidType);
    }

    #[test]
    fn reject_wrong_type_for_capability_version() {
        let mut m = minimal_manifest_json();
        m["capability_version"] = json!("one");
        let err = TrustedManifest::parse(&m.to_string()).unwrap_err();
        assert_eq!(err.code, ManifestErrorCode::InvalidType);
    }

    #[test]
    fn reject_invalid_binding_kind() {
        let mut m = minimal_manifest_json();
        m["binding"]["kind"] = json!("http");
        let err = TrustedManifest::parse(&m.to_string()).unwrap_err();
        assert_eq!(err.code, ManifestErrorCode::InvalidBinding);
    }

    #[test]
    fn reject_invalid_identity_source() {
        let mut m = minimal_manifest_json();
        m["provider"]["identity_source"] = json!("mcp_server_info");
        let err = TrustedManifest::parse(&m.to_string()).unwrap_err();
        assert_eq!(err.code, ManifestErrorCode::InvalidProvider);
    }

    // === null permission_scope is Unrestricted ===

    #[test]
    fn null_permission_scope_is_unrestricted() {
        let mut m = minimal_manifest_json();
        m["permission_scope"] = serde_json::Value::Null;
        let parsed = TrustedManifest::parse(&m.to_string()).unwrap();
        assert!(matches!(
            parsed.permission_scope,
            PermissionScope::Unrestricted
        ));
    }
}
