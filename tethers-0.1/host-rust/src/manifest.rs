// Columbo capability manifest - data types and structured error model.
//
// C1a1: types and error codes only.
// Deferred to later tasks: JSON parsing, duplicate-key detection, unknown-field
// rejection, RFC 8785/JCS canonicalization, SHA-256 digesting, semantic and
// cross-field validation, capability projection, manifest storage, provider
// connections, credential handling, Action execution, standing approvals,
// Trail integration.

use serde_json::Value;

// ---------------------------------------------------------------------------
// Manifest format version
// ---------------------------------------------------------------------------

/// Manifest format version. Only `"1.0"` is recognised in 0.1.
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Determinism {
    Deterministic,
    NonDeterministic,
}

// ---------------------------------------------------------------------------
// Idempotency
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Idempotency {
    /// Dedicated idempotency-key argument; provider deduplicates by key.
    ArgumentKey {
        argument_name: String,
        key_source: String,
        description: Option<String>,
    },
    /// Provider guarantees idempotency internally without a client key.
    /// Trusted host/provider/adapter evidence must describe the deduplication
    /// key, scope, and lifetime, pinned by the manifest binding.
    ServerDedup {
        dedup_key: String,
        dedup_scope: String,
        dedup_lifetime: String,
        evidence: String,
        description: Option<String>,
    },
    /// No idempotency guarantee.
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

/// Adapter binding for tools requiring a separately reviewed typed adapter.
///
/// The exact adapter manifest format is an unresolved design decision
/// (CAPABILITY_BRIDGE.md section 16, item 4). This struct captures only the fields
/// that are explicitly settled: identity, version, and digest.
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

/// A fully validated, immutable capability manifest.
///
/// `input_schema` and `output_schema` are opaque JSON Schema values.
/// Structural validation of their contents belongs to later tasks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrustedManifest {
    pub manifest_format_version: String,
    pub capability_name: String,
    pub capability_version: u32,
    pub title: String,
    pub description: String,
    pub input_schema: Value,
    pub output_schema: Value,
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
    /// Computed SHA-256 digest in `"sha256:hex..."` form, or `None` when not
    /// yet computed/verified.
    pub digest: Option<String>,
}

// ---------------------------------------------------------------------------
// Structured error model
// ---------------------------------------------------------------------------

/// Stable error codes for manifest validation failures.
///
/// These are host-side codes, distinct from the Tethers protocol error codes
/// defined in SPEC.md section 11.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManifestErrorCode {
    /// JSON parse failure, including duplicate keys in any object.
    InvalidJson,
    /// `manifest_format_version` is not a recognised value.
    UnknownFormatVersion,
    /// A required top-level field is absent.
    MissingField,
    /// An unknown top-level field is present.
    UnknownField,
    /// A field has an unexpected JSON type (e.g. string where integer expected).
    InvalidType,
    /// A field has an invalid enum value, out-of-range number, or malformed string.
    InvalidValue,
    /// `permission_scope` has an unrecognised `kind` or missing required sub-fields.
    InvalidScope,
    /// `effects` is empty, contains duplicates, or has malformed entries.
    InvalidEffects,
    /// `idempotency` mechanism is missing required fields for its variant.
    InvalidIdempotency,
    /// Contradictory confirmation policy.
    InvalidConfirmation,
    /// Contradictory or unsafe retry policy.
    InvalidRetry,
    /// Provider identity has missing or invalid fields.
    InvalidProvider,
    /// Binding has missing or invalid fields.
    InvalidBinding,
    /// A value resembling a credential was detected (defence-in-depth).
    ContainsCredentials,
    /// The stored `digest` does not match the computed SHA-256 digest.
    DigestMismatch,
}

/// A structured manifest validation error.
///
/// `field` is a JSON pointer path (RFC 6901) to the offending value, or `None`
/// when the error is not specific to a single field.
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

    /// Construct a minimal valid read-only manifest for type-construction testing.
    fn minimal_read_manifest() -> TrustedManifest {
        TrustedManifest {
            manifest_format_version: ManifestFormatVersion::V1_0.to_string(),
            capability_name: "obsidian.note.read".to_string(),
            capability_version: 1,
            title: "Read an Obsidian note".to_string(),
            description: "Read the Markdown content of a note.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" }
                },
                "required": ["path"],
                "additionalProperties": false
            }),
            output_schema: json!({
                "type": "object",
                "properties": {
                    "content": { "type": "string" },
                    "frontmatter": { "type": "object" }
                },
                "required": ["content"]
            }),
            effects: vec!["filesystem.read".to_string()],
            permission_scope: PermissionScope::PathPrefix {
                allowed_prefixes: vec!["projects/".to_string(), "daily/".to_string()],
            },
            reversibility: Reversibility::Reversible,
            determinism: Determinism::Deterministic,
            idempotency: Idempotency::NoMechanism,
            confirmation_policy: ConfirmationPolicy {
                standing_permitted: true,
                per_call_required: false,
                description: None,
            },
            timeout_ms: 5000,
            retry_policy: RetryPolicy {
                max_retries: 3,
                backoff_ms: 500,
                allowed_on: vec![RetryCondition::OutcomeUnknown],
                requires_idempotency_proof: false,
            },
            provider: ProviderIdentity {
                identity: "obsidian-local".to_string(),
                display_name: "Obsidian (local vault)".to_string(),
                identity_source: IdentitySource::HostConfiguration,
                description: Some(
                    "Host-assigned identity for the local Obsidian MCP server.".to_string(),
                ),
            },
            binding: Binding {
                kind: BindingKind::Mcp,
                server_name: "obsidian".to_string(),
                tool_name: "obsidian_read_note".to_string(),
                adapter: None,
            },
            digest: None,
        }
    }

    /// Construct a valid scoped write manifest with argument_key idempotency.
    fn write_manifest() -> TrustedManifest {
        TrustedManifest {
            manifest_format_version: ManifestFormatVersion::V1_0.to_string(),
            capability_name: "notes.note.create".to_string(),
            capability_version: 1,
            title: "Create a project note".to_string(),
            description: "Create a new Markdown note in the project vault.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "title": { "type": "string" },
                    "content": { "type": "string" },
                    "tags": { "type": "array", "items": { "type": "string" } },
                    "idempotency_key": { "type": "string" }
                },
                "required": ["title", "content"],
                "additionalProperties": false
            }),
            output_schema: json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "modified": { "type": "boolean" }
                },
                "required": ["path", "modified"]
            }),
            effects: vec!["filesystem.write".to_string()],
            permission_scope: PermissionScope::PathPrefix {
                allowed_prefixes: vec!["projects/".to_string(), "daily/".to_string()],
            },
            reversibility: Reversibility::Compensatable,
            determinism: Determinism::Deterministic,
            idempotency: Idempotency::ArgumentKey {
                argument_name: "idempotency_key".to_string(),
                key_source: "evaluation_id/action_id".to_string(),
                description: Some("Server deduplicates by idempotency_key argument.".to_string()),
            },
            confirmation_policy: ConfirmationPolicy {
                standing_permitted: false,
                per_call_required: true,
                description: Some("Creating notes requires per-call confirmation.".to_string()),
            },
            timeout_ms: 10000,
            retry_policy: RetryPolicy {
                max_retries: 3,
                backoff_ms: 1000,
                allowed_on: vec![RetryCondition::OutcomeUnknown],
                requires_idempotency_proof: true,
            },
            provider: ProviderIdentity {
                identity: "obsidian-local".to_string(),
                display_name: "Obsidian (local vault)".to_string(),
                identity_source: IdentitySource::HostConfiguration,
                description: Some(
                    "Host-assigned identity for the local Obsidian MCP server.".to_string(),
                ),
            },
            binding: Binding {
                kind: BindingKind::Mcp,
                server_name: "obsidian".to_string(),
                tool_name: "obsidian_create_note".to_string(),
                adapter: None,
            },
            digest: None,
        }
    }

    // -- Type construction tests --

    #[test]
    fn construct_minimal_read_manifest() {
        let m = minimal_read_manifest();
        assert_eq!(m.manifest_format_version, "1.0");
        assert_eq!(m.capability_name, "obsidian.note.read");
        assert_eq!(m.capability_version, 1);
        assert!(matches!(m.reversibility, Reversibility::Reversible));
        assert!(matches!(m.determinism, Determinism::Deterministic));
        assert!(matches!(m.idempotency, Idempotency::NoMechanism));
        assert!(!m.confirmation_policy.per_call_required);
        assert_eq!(m.effects, vec!["filesystem.read"]);
        assert_eq!(m.timeout_ms, 5000);
    }

    #[test]
    fn construct_write_manifest_with_argument_key() {
        let m = write_manifest();
        assert_eq!(m.capability_name, "notes.note.create");
        assert!(matches!(m.reversibility, Reversibility::Compensatable));
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
        assert_eq!(m.effects, vec!["filesystem.write"]);
        assert!(m.retry_policy.requires_idempotency_proof);
    }

    #[test]
    fn provider_identity_is_host_configuration_only() {
        let m = minimal_read_manifest();
        assert_eq!(m.provider.identity, "obsidian-local");
        assert!(matches!(
            m.provider.identity_source,
            IdentitySource::HostConfiguration
        ));
    }

    #[test]
    fn binding_supports_direct_mcp_and_adapter() {
        let direct = minimal_read_manifest();
        assert!(matches!(direct.binding.kind, BindingKind::Mcp));
        assert!(direct.binding.adapter.is_none());

        let with_adapter = Binding {
            kind: BindingKind::Mcp,
            server_name: "obsidian".to_string(),
            tool_name: "obsidian_legacy".to_string(),
            adapter: Some(AdapterBinding {
                name: "legacy-parser".to_string(),
                version: "1.0.0".to_string(),
                digest: "sha256:abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
                    .to_string(),
            }),
        };
        let adapter = with_adapter.adapter.unwrap();
        assert_eq!(adapter.name, "legacy-parser");
        assert_eq!(adapter.version, "1.0.0");
        assert!(adapter.digest.starts_with("sha256:"));
    }

    // -- Error model tests --

    #[test]
    fn error_codes_are_stable() {
        // Verify every error code variant exists and can be constructed.
        let codes = [
            ManifestErrorCode::InvalidJson,
            ManifestErrorCode::UnknownFormatVersion,
            ManifestErrorCode::MissingField,
            ManifestErrorCode::UnknownField,
            ManifestErrorCode::InvalidType,
            ManifestErrorCode::InvalidValue,
            ManifestErrorCode::InvalidScope,
            ManifestErrorCode::InvalidEffects,
            ManifestErrorCode::InvalidIdempotency,
            ManifestErrorCode::InvalidConfirmation,
            ManifestErrorCode::InvalidRetry,
            ManifestErrorCode::InvalidProvider,
            ManifestErrorCode::InvalidBinding,
            ManifestErrorCode::ContainsCredentials,
            ManifestErrorCode::DigestMismatch,
        ];
        // Each code must have a distinct debug representation.
        let names: Vec<String> = codes.iter().map(|c| format!("{:?}", c)).collect();
        let unique: std::collections::HashSet<_> = names.iter().collect();
        assert_eq!(unique.len(), codes.len(), "error code names must be unique");
    }

    #[test]
    fn error_with_field_pointer() {
        let err = ManifestError::with_field(
            ManifestErrorCode::MissingField,
            "required field is absent",
            "/capability_name",
        );
        assert_eq!(err.code, ManifestErrorCode::MissingField);
        assert_eq!(err.field.as_deref(), Some("/capability_name"));
    }

    #[test]
    fn error_without_field_pointer() {
        let err = ManifestError::new(ManifestErrorCode::InvalidJson, "duplicate key in object");
        assert_eq!(err.code, ManifestErrorCode::InvalidJson);
        assert!(err.field.is_none());
    }

    #[test]
    fn permission_scope_variants_are_representable() {
        let path = PermissionScope::PathPrefix {
            allowed_prefixes: vec!["projects/".to_string()],
        };
        let repo = PermissionScope::Repository {
            allowed_repositories: vec!["org/repo".to_string()],
        };
        let cal = PermissionScope::Calendar {
            allowed_calendars: vec!["work".to_string()],
        };
        let unrestricted = PermissionScope::Unrestricted;

        match &path {
            PermissionScope::PathPrefix { allowed_prefixes } => {
                assert_eq!(allowed_prefixes, &vec!["projects/"]);
            }
            _ => panic!("expected PathPrefix"),
        }
        assert!(matches!(repo, PermissionScope::Repository { .. }));
        assert!(matches!(cal, PermissionScope::Calendar { .. }));
        assert!(matches!(unrestricted, PermissionScope::Unrestricted));
    }

    #[test]
    fn idempotency_variants_are_representable() {
        let arg_key = Idempotency::ArgumentKey {
            argument_name: "idem".to_string(),
            key_source: "eval/act".to_string(),
            description: Some("desc".to_string()),
        };
        let server = Idempotency::ServerDedup {
            dedup_key: "provider request id".to_string(),
            dedup_scope: "provider account and target collection".to_string(),
            dedup_lifetime: "at least 24 hours".to_string(),
            evidence: "adapter contract review obsidian-local@2026-07-21".to_string(),
            description: Some("dedup by content hash".to_string()),
        };
        let none = Idempotency::NoMechanism;

        assert!(matches!(arg_key, Idempotency::ArgumentKey { .. }));
        match server {
            Idempotency::ServerDedup {
                dedup_key,
                dedup_scope,
                dedup_lifetime,
                evidence,
                ..
            } => {
                assert_eq!(dedup_key, "provider request id");
                assert_eq!(dedup_scope, "provider account and target collection");
                assert_eq!(dedup_lifetime, "at least 24 hours");
                assert!(evidence.contains("adapter contract review"));
            }
            _ => panic!("expected ServerDedup"),
        }
        assert!(matches!(none, Idempotency::NoMechanism));
    }

    #[test]
    fn input_schema_preserves_nested_description_keys() {
        // Schemas are digested completely; nested 'description' keys must be
        // representable in the Value.
        let m = minimal_read_manifest();
        let path_prop = &m.input_schema["properties"]["path"];
        assert_eq!(path_prop["type"], "string");
        // No description in the minimal fixture, but the Value can hold one.
        let with_desc = json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "Note path" }
            }
        });
        assert_eq!(with_desc["properties"]["path"]["description"], "Note path");
    }
}
