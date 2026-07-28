// J12 Packet 1 - strict local runtime configuration foundation
//
// Parses, validates, and materialises the frozen J12 JSON configuration
// that selects one Tether Set, its ordered source files, exact capability
// requirements, explicit stdio provider bindings, reviewed manifest files
// with pinned digests, scope bindings, and exact local policy rules.
//
// This module does not launch providers, read manifests, invoke the engine,
// assess live Actions, dispatch, or write a Trail.  Packet 2 owns runtime
// wiring and live scope assessment.

use serde::Deserialize;
use std::collections::HashSet;
use std::fmt;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Re-export the shared duplicate-key-rejecting JSON parser as a crate-
// visible helper.  runtime_config uses it before serde deserialization so
// that duplicate keys are rejected at every depth before any type-level
// validation begins.
// ---------------------------------------------------------------------------
pub(crate) use crate::manifest::parse_value_no_dupes;

// ===========================================================================
// Data types – exact frozen J12 shape
// ===========================================================================

/// The complete parsed and semantically validated runtime configuration.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeConfig {
    pub format_version: String,
    pub tether_set: TetherSetConfig,
    pub providers: Vec<ProviderBindingConfig>,
    pub policy: PolicyConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TetherSetConfig {
    pub id: String,
    pub version: String,
    pub tethers: Vec<TetherRef>,
    pub capability_requirements: Vec<CapabilityRequirementConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TetherRef {
    pub id: String,
    pub version: String,
    pub source_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityRequirementConfig {
    pub name: String,
    pub version: u32,
    #[serde(default)]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderBindingConfig {
    pub id: String,
    pub display_name: String,
    pub transport: TransportConfig,
    pub capabilities: Vec<ProviderCapabilityConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransportConfig {
    pub kind: TransportKind,
    pub command: String,
    pub args: Vec<String>,
    pub protocol_version: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransportKind {
    Stdio,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderCapabilityConfig {
    pub name: String,
    pub version: u32,
    pub manifest_path: String,
    pub pinned_digest: String,
    #[serde(default)]
    pub scope_binding: Option<ScopeBindingConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScopeBindingConfig {
    pub kind: ScopeBindingKind,
    pub argument_json_pointer: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScopeBindingKind {
    PathPrefix,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyConfig {
    pub default: PolicyDecision,
    pub rules: Vec<PolicyRuleConfig>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyDecision {
    Allow,
    Ask,
    Deny,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyRuleConfig {
    pub name: String,
    pub version: u32,
    pub decision: PolicyDecision,
}

// ===========================================================================
// Loaded configuration – resolved paths
// ===========================================================================

/// A parsed configuration together with its filesystem provenance.
///
/// Source and manifest paths are resolved relative to the configuration
/// file's parent directory.  The configuration directory is stored as an
/// absolute path so that later stages (Packet 2) can resolve relative
/// paths for Tether source files and manifest files.
#[derive(Debug, Clone)]
pub struct LoadedRuntimeConfig {
    pub config: RuntimeConfig,
    /// Absolute path to the configuration file itself.
    pub config_path: PathBuf,
    /// Absolute parent directory of the configuration file.
    pub config_dir: PathBuf,
}

impl LoadedRuntimeConfig {
    /// Resolve a relative source or manifest path against `config_dir`.
    fn resolve(&self, relative: &str) -> PathBuf {
        self.config_dir.join(relative)
    }

    /// Resolved absolute path for a Tether source file.
    pub fn tether_source_path(&self, source_path: &str) -> PathBuf {
        self.resolve(source_path)
    }

    /// Resolved absolute path for a manifest file.
    pub fn manifest_path(&self, manifest_path: &str) -> PathBuf {
        self.resolve(manifest_path)
    }
}

// ===========================================================================
// Structured error model
// ===========================================================================

/// Machine-readable error codes for runtime configuration errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeConfigErrorCode {
    /// The JSON text could not be parsed (malformed syntax or duplicate key).
    InvalidJson,
    /// The format_version field is missing or has an unsupported value.
    UnknownFormatVersion,
    /// A required field is missing or a value has the wrong Serde type.
    InvalidType,
    /// A field whose value is invalid according to semantic rules.
    InvalidValue,
    /// A duplicate entry was found where uniqueness is required.
    DuplicateEntry,
    /// An entry references something that does not exist.
    UnmatchedReference,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeConfigError {
    pub code: RuntimeConfigErrorCode,
    pub message: String,
    /// JSON Pointer to the offending field, when applicable.
    pub field: Option<String>,
}

impl RuntimeConfigError {
    fn new(code: RuntimeConfigErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            field: None,
        }
    }

    fn with_field(
        code: RuntimeConfigErrorCode,
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

impl fmt::Display for RuntimeConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref field) = self.field {
            write!(f, "{:?}: {} (at {})", self.code, self.message, field)
        } else {
            write!(f, "{:?}: {}", self.code, self.message)
        }
    }
}

// ===========================================================================
// Serde deserialization helpers
// ===========================================================================

/// Deserialize a `serde_json::Value` into a target type that uses
/// `#[serde(deny_unknown_fields)]`, converting unknown-field errors
/// into our structured error model.
fn deserialize_config<T: serde::de::DeserializeOwned>(
    value: &serde_json::Value,
    pointer: &str,
) -> Result<T, RuntimeConfigError> {
    T::deserialize(value).map_err(|e| {
        // Try to extract a JSON Pointer path from the serde error.
        let field = extract_serde_field(&e, pointer);
        RuntimeConfigError::with_field(RuntimeConfigErrorCode::InvalidType, e.to_string(), field)
    })
}

/// Best-effort extraction of a JSON-Pointer-like path from a serde error.
fn extract_serde_field(e: &serde_json::Error, parent: &str) -> String {
    let msg = e.to_string();
    // serde unknown field errors contain the field name in the message.
    // Try to extract it and build a JSON Pointer.
    for prefix in &["unknown field `", "missing field `"] {
        if let Some(start) = msg.find(prefix) {
            let remainder = &msg[start + prefix.len()..];
            if let Some(end) = remainder.find('`') {
                let field_name = &remainder[..end];
                return json_pointer_child(parent, field_name);
            }
        }
    }
    parent.to_owned()
}

/// Append a segment to a JSON Pointer string, escaping `~` and `/`.
fn json_pointer_child(parent: &str, segment: &str) -> String {
    let escaped = segment.replace('~', "~0").replace('/', "~1");
    if parent.is_empty() {
        format!("/{escaped}")
    } else {
        format!("{parent}/{escaped}")
    }
}

// ===========================================================================
// Semantic validation
// ===========================================================================

/// Validate the configuration after structural deserialization.
fn validate(config: &RuntimeConfig) -> Result<(), RuntimeConfigError> {
    validate_format_version(config)?;
    validate_tether_set(&config.tether_set)?;
    validate_providers(&config.providers)?;
    validate_policy(&config.policy)?;
    validate_cross_references(config)?;
    Ok(())
}

fn validate_format_version(config: &RuntimeConfig) -> Result<(), RuntimeConfigError> {
    if config.format_version != "0.1" {
        return Err(RuntimeConfigError::with_field(
            RuntimeConfigErrorCode::UnknownFormatVersion,
            format!(
                "unsupported format_version: \"{}\", expected \"0.1\"",
                config.format_version
            ),
            "/format_version",
        ));
    }
    Ok(())
}

fn validate_tether_set(set: &TetherSetConfig) -> Result<(), RuntimeConfigError> {
    // Non-empty id
    if set.id.trim().is_empty() {
        return Err(RuntimeConfigError::with_field(
            RuntimeConfigErrorCode::InvalidValue,
            "tether_set.id must not be empty or whitespace-only",
            "/tether_set/id",
        ));
    }

    // Non-empty version
    if set.version.trim().is_empty() {
        return Err(RuntimeConfigError::with_field(
            RuntimeConfigErrorCode::InvalidValue,
            "tether_set.version must not be empty or whitespace-only",
            "/tether_set/version",
        ));
    }

    // Non-empty tethers list
    if set.tethers.is_empty() {
        return Err(RuntimeConfigError::with_field(
            RuntimeConfigErrorCode::InvalidValue,
            "tether_set.tethers must not be empty",
            "/tether_set/tethers",
        ));
    }

    // Non-empty requirements list
    if set.capability_requirements.is_empty() {
        return Err(RuntimeConfigError::with_field(
            RuntimeConfigErrorCode::InvalidValue,
            "tether_set.capability_requirements must not be empty",
            "/tether_set/capability_requirements",
        ));
    }

    // Validate each Tether
    let mut seen_tether_ids: HashSet<(String, String)> = HashSet::new();
    for (i, tether) in set.tethers.iter().enumerate() {
        let ptr = format!("/tether_set/tethers/{}", i);

        if tether.id.trim().is_empty() {
            return Err(RuntimeConfigError::with_field(
                RuntimeConfigErrorCode::InvalidValue,
                "tether id must not be empty or whitespace-only",
                format!("{}/id", ptr),
            ));
        }
        if tether.version.trim().is_empty() {
            return Err(RuntimeConfigError::with_field(
                RuntimeConfigErrorCode::InvalidValue,
                "tether version must not be empty or whitespace-only",
                format!("{}/version", ptr),
            ));
        }
        // source_path must not be empty
        if tether.source_path.trim().is_empty() {
            return Err(RuntimeConfigError::with_field(
                RuntimeConfigErrorCode::InvalidValue,
                "tether source_path must not be empty",
                format!("{}/source_path", ptr),
            ));
        }
        // source_path must not be absolute
        if Path::new(&tether.source_path).is_absolute() {
            return Err(RuntimeConfigError::with_field(
                RuntimeConfigErrorCode::InvalidValue,
                format!(
                    "tether source_path must be relative, got \"{}\"",
                    tether.source_path
                ),
                format!("{}/source_path", ptr),
            ));
        }

        // Duplicate Tether id/version pair
        let key = (tether.id.clone(), tether.version.clone());
        if !seen_tether_ids.insert(key) {
            return Err(RuntimeConfigError::with_field(
                RuntimeConfigErrorCode::DuplicateEntry,
                format!(
                    "duplicate tether identity: id=\"{}\" version=\"{}\"",
                    tether.id, tether.version
                ),
                format!("{}/id", ptr),
            ));
        }
    }

    // Validate each capability requirement
    let mut seen_req_ids: HashSet<(String, u32)> = HashSet::new();
    for (i, req) in set.capability_requirements.iter().enumerate() {
        let ptr = format!("/tether_set/capability_requirements/{}", i);

        if req.name.trim().is_empty() {
            return Err(RuntimeConfigError::with_field(
                RuntimeConfigErrorCode::InvalidValue,
                "capability requirement name must not be empty or whitespace-only",
                format!("{}/name", ptr),
            ));
        }
        if req.version == 0 {
            return Err(RuntimeConfigError::with_field(
                RuntimeConfigErrorCode::InvalidValue,
                "capability requirement version must be greater than zero",
                format!("{}/version", ptr),
            ));
        }

        // Duplicate requirement name/version pair
        let key = (req.name.clone(), req.version);
        if !seen_req_ids.insert(key) {
            return Err(RuntimeConfigError::with_field(
                RuntimeConfigErrorCode::DuplicateEntry,
                format!(
                    "duplicate capability requirement: name=\"{}\" version={}",
                    req.name, req.version
                ),
                format!("{}/name", ptr),
            ));
        }
    }

    Ok(())
}

fn validate_providers(providers: &[ProviderBindingConfig]) -> Result<(), RuntimeConfigError> {
    // Non-empty providers list
    if providers.is_empty() {
        return Err(RuntimeConfigError::with_field(
            RuntimeConfigErrorCode::InvalidValue,
            "providers must not be empty",
            "/providers",
        ));
    }

    // First pass: count exact capability identities across ALL providers.
    // Used below to reject scope-bound capabilities whose identity appears
    // more than once anywhere in the configuration.
    let mut identity_counts: std::collections::HashMap<(String, u32), usize> =
        std::collections::HashMap::new();
    for provider in providers {
        for cap in &provider.capabilities {
            *identity_counts
                .entry((cap.name.clone(), cap.version))
                .or_insert(0) += 1;
        }
    }

    let mut seen_provider_ids: HashSet<String> = HashSet::new();

    for (pi, provider) in providers.iter().enumerate() {
        let pptr = format!("/providers/{}", pi);

        // Non-empty provider id
        if provider.id.trim().is_empty() {
            return Err(RuntimeConfigError::with_field(
                RuntimeConfigErrorCode::InvalidValue,
                "provider id must not be empty or whitespace-only",
                format!("{}/id", pptr),
            ));
        }
        // Duplicate provider ID
        if !seen_provider_ids.insert(provider.id.clone()) {
            return Err(RuntimeConfigError::with_field(
                RuntimeConfigErrorCode::DuplicateEntry,
                format!("duplicate provider id: \"{}\"", provider.id),
                format!("{}/id", pptr),
            ));
        }

        // Transport validation
        match provider.transport.kind {
            TransportKind::Stdio => {
                if provider.transport.command.trim().is_empty() {
                    return Err(RuntimeConfigError::with_field(
                        RuntimeConfigErrorCode::InvalidValue,
                        "stdio transport command must not be empty or whitespace-only",
                        format!("{}/transport/command", pptr),
                    ));
                }
                if provider.transport.protocol_version.trim().is_empty() {
                    return Err(RuntimeConfigError::with_field(
                        RuntimeConfigErrorCode::InvalidValue,
                        "stdio transport protocol_version must not be empty or whitespace-only",
                        format!("{}/transport/protocol_version", pptr),
                    ));
                }
            }
        }

        // Non-empty capabilities list
        if provider.capabilities.is_empty() {
            return Err(RuntimeConfigError::with_field(
                RuntimeConfigErrorCode::InvalidValue,
                "provider capabilities must not be empty",
                format!("{}/capabilities", pptr),
            ));
        }

        // Validate each provider capability
        let mut seen_cap_ids: HashSet<(String, u32)> = HashSet::new();

        for (ci, cap) in provider.capabilities.iter().enumerate() {
            let cptr = format!("{}/capabilities/{}", pptr, ci);

            if cap.name.trim().is_empty() {
                return Err(RuntimeConfigError::with_field(
                    RuntimeConfigErrorCode::InvalidValue,
                    "provider capability name must not be empty or whitespace-only",
                    format!("{}/name", cptr),
                ));
            }
            if cap.version == 0 {
                return Err(RuntimeConfigError::with_field(
                    RuntimeConfigErrorCode::InvalidValue,
                    "provider capability version must be greater than zero",
                    format!("{}/version", cptr),
                ));
            }

            // Duplicate provider capability name/version pair
            let key = (cap.name.clone(), cap.version);
            if !seen_cap_ids.insert(key) {
                return Err(RuntimeConfigError::with_field(
                    RuntimeConfigErrorCode::DuplicateEntry,
                    format!(
                        "duplicate provider capability: name=\"{}\" version={}",
                        cap.name, cap.version
                    ),
                    format!("{}/name", cptr),
                ));
            }

            // manifest_path must not be empty
            if cap.manifest_path.trim().is_empty() {
                return Err(RuntimeConfigError::with_field(
                    RuntimeConfigErrorCode::InvalidValue,
                    "manifest_path must not be empty",
                    format!("{}/manifest_path", cptr),
                ));
            }
            // manifest_path must not be absolute
            if Path::new(&cap.manifest_path).is_absolute() {
                return Err(RuntimeConfigError::with_field(
                    RuntimeConfigErrorCode::InvalidValue,
                    format!(
                        "manifest_path must be relative, got \"{}\"",
                        cap.manifest_path
                    ),
                    format!("{}/manifest_path", cptr),
                ));
            }

            // Validate pinned digest: exactly "sha256:" followed by 64 lowercase hex chars
            validate_digest(&cap.pinned_digest, &format!("{}/pinned_digest", cptr))?;

            // Global exact identity uniqueness check (J12 Packet 2):
            // Every exact capability identity (name, version) must appear under
            // exactly one configured provider.  Duplicate exact identities are
            // rejected whether or not scope_binding is present.
            let count = identity_counts
                .get(&(cap.name.clone(), cap.version))
                .copied()
                .unwrap_or(1);
            if count > 1 {
                return Err(RuntimeConfigError::with_field(
                    RuntimeConfigErrorCode::DuplicateEntry,
                    format!(
                        "capability \"{}\" version {} appears {} times across \
                         providers; every exact (name, version) identity must \
                         be globally unique",
                        cap.name, cap.version, count
                    ),
                    format!("{}/name", cptr),
                ));
            }

            // Validate scope binding
            if let Some(ref scope) = cap.scope_binding {
                match scope.kind {
                    ScopeBindingKind::PathPrefix => {
                        validate_json_pointer(
                            &scope.argument_json_pointer,
                            &format!("{}/scope_binding/argument_json_pointer", cptr),
                        )?;
                    }
                }
            }
        }
    }

    Ok(())
}

fn validate_digest(raw: &str, pointer: &str) -> Result<(), RuntimeConfigError> {
    if raw.len() != 71 {
        return Err(RuntimeConfigError::with_field(
            RuntimeConfigErrorCode::InvalidValue,
            format!(
                "pinned digest must be \"sha256:\" followed by 64 lowercase hex characters, \
                 got {} bytes",
                raw.len()
            ),
            pointer,
        ));
    }

    if &raw[..7] != "sha256:" {
        return Err(RuntimeConfigError::with_field(
            RuntimeConfigErrorCode::InvalidValue,
            "pinned digest must begin with \"sha256:\"",
            pointer,
        ));
    }

    for (i, b) in raw.bytes().enumerate().skip(7) {
        if !b.is_ascii_hexdigit() || b.is_ascii_uppercase() {
            return Err(RuntimeConfigError::with_field(
                RuntimeConfigErrorCode::InvalidValue,
                format!(
                    "pinned digest hex character at position {} must be lowercase hexadecimal",
                    i
                ),
                pointer,
            ));
        }
    }

    Ok(())
}

fn validate_json_pointer(pointer: &str, field: &str) -> Result<(), RuntimeConfigError> {
    if pointer.is_empty() {
        return Err(RuntimeConfigError::with_field(
            RuntimeConfigErrorCode::InvalidValue,
            "JSON Pointer must not be empty",
            field,
        ));
    }
    if !pointer.starts_with('/') {
        return Err(RuntimeConfigError::with_field(
            RuntimeConfigErrorCode::InvalidValue,
            format!("JSON Pointer must begin with '/', got \"{}\"", pointer),
            field,
        ));
    }
    Ok(())
}

fn validate_policy(policy: &PolicyConfig) -> Result<(), RuntimeConfigError> {
    // Default must be deny
    match policy.default {
        PolicyDecision::Deny => {}
        other => {
            return Err(RuntimeConfigError::with_field(
                RuntimeConfigErrorCode::InvalidValue,
                format!("policy default must be \"deny\", got {:?}", other),
                "/policy/default",
            ));
        }
    }

    // Validate rules
    let mut seen_rule_ids: HashSet<(String, u32)> = HashSet::new();
    for (i, rule) in policy.rules.iter().enumerate() {
        let ptr = format!("/policy/rules/{}", i);

        if rule.name.trim().is_empty() {
            return Err(RuntimeConfigError::with_field(
                RuntimeConfigErrorCode::InvalidValue,
                "policy rule name must not be empty or whitespace-only",
                format!("{}/name", ptr),
            ));
        }
        if rule.version == 0 {
            return Err(RuntimeConfigError::with_field(
                RuntimeConfigErrorCode::InvalidValue,
                "policy rule version must be greater than zero",
                format!("{}/version", ptr),
            ));
        }

        // Duplicate policy rule
        let key = (rule.name.clone(), rule.version);
        if !seen_rule_ids.insert(key) {
            return Err(RuntimeConfigError::with_field(
                RuntimeConfigErrorCode::DuplicateEntry,
                format!(
                    "duplicate policy rule: name=\"{}\" version={}",
                    rule.name, rule.version
                ),
                format!("{}/name", ptr),
            ));
        }
    }

    Ok(())
}

/// Cross-reference validation between sections.
fn validate_cross_references(config: &RuntimeConfig) -> Result<(), RuntimeConfigError> {
    let requirements = &config.tether_set.capability_requirements;
    let providers = &config.providers;

    // Build a set of all provider capabilities: (name, version)
    let mut provider_caps: HashSet<(String, u32)> = HashSet::new();
    for provider in providers {
        for cap in &provider.capabilities {
            provider_caps.insert((cap.name.clone(), cap.version));
        }
    }

    // 1. Every requirement must have an exactly matching provider capability.
    for (i, req) in requirements.iter().enumerate() {
        let key = (req.name.clone(), req.version);
        if !provider_caps.contains(&key) {
            return Err(RuntimeConfigError::with_field(
                RuntimeConfigErrorCode::UnmatchedReference,
                format!(
                    "capability requirement name=\"{}\" version={} has no matching \
                     configured provider capability",
                    req.name, req.version
                ),
                format!("/tether_set/capability_requirements/{}/name", i),
            ));
        }
    }

    // Build set of requirement identities
    let req_set: HashSet<(String, u32)> = requirements
        .iter()
        .map(|r| (r.name.clone(), r.version))
        .collect();

    // 2. Every provider capability must be required by the Tether Set.
    for (pi, provider) in providers.iter().enumerate() {
        for (ci, cap) in provider.capabilities.iter().enumerate() {
            let key = (cap.name.clone(), cap.version);
            if !req_set.contains(&key) {
                return Err(RuntimeConfigError::with_field(
                    RuntimeConfigErrorCode::UnmatchedReference,
                    format!(
                        "provider capability name=\"{}\" version={} is not required \
                         by the selected Tether Set",
                        cap.name, cap.version
                    ),
                    format!("/providers/{}/capabilities/{}/name", pi, ci),
                ));
            }
        }
    }

    // 3. Every policy rule must target a declared requirement.
    for (i, rule) in config.policy.rules.iter().enumerate() {
        let key = (rule.name.clone(), rule.version);
        if !req_set.contains(&key) {
            return Err(RuntimeConfigError::with_field(
                RuntimeConfigErrorCode::UnmatchedReference,
                format!(
                    "policy rule for name=\"{}\" version={} does not match any \
                     declared capability requirement",
                    rule.name, rule.version
                ),
                format!("/policy/rules/{}/name", i),
            ));
        }
    }

    Ok(())
}

// ===========================================================================
// Public parse API
// ===========================================================================

/// Parse a runtime configuration JSON string.
///
/// Pipeline:
/// 1. Strict duplicate-key-rejecting JSON parse (shared with manifest).
/// 2. Serde deserialization with `deny_unknown_fields`.
/// 3. Semantic cross-field validation.
pub fn parse_runtime_config(json: &str) -> Result<RuntimeConfig, RuntimeConfigError> {
    // Step 1: duplicate-key-rejecting parse
    let value = parse_value_no_dupes(json)
        .map_err(|e| RuntimeConfigError::new(RuntimeConfigErrorCode::InvalidJson, e.to_string()))?;

    // Step 2: serde deserialization
    let config: RuntimeConfig = deserialize_config(&value, "")?;

    // Step 3: semantic validation
    validate(&config)?;

    Ok(config)
}

/// Load and parse a runtime configuration file from disk.
///
/// Reads the file at `path`, parses it with [`parse_runtime_config`], and
/// returns a [`LoadedRuntimeConfig`] that includes the absolute config-file
/// path and its parent directory for relative-path resolution.
///
/// Does not read Tether source or manifest files.
pub fn load_runtime_config(path: &Path) -> Result<LoadedRuntimeConfig, RuntimeConfigError> {
    let json = std::fs::read_to_string(path).map_err(|e| {
        RuntimeConfigError::new(
            RuntimeConfigErrorCode::InvalidJson,
            format!("could not read config file \"{}\": {}", path.display(), e),
        )
    })?;

    let config = parse_runtime_config(&json)?;

    let config_path = path.canonicalize().map_err(|e| {
        RuntimeConfigError::new(
            RuntimeConfigErrorCode::InvalidValue,
            format!(
                "could not resolve config file path \"{}\": {}",
                path.display(),
                e
            ),
        )
    })?;

    let config_dir = config_path
        .parent()
        .ok_or_else(|| {
            RuntimeConfigError::new(
                RuntimeConfigErrorCode::InvalidValue,
                format!(
                    "config file path \"{}\" has no parent directory",
                    config_path.display()
                ),
            )
        })?
        .to_path_buf();

    Ok(LoadedRuntimeConfig {
        config,
        config_path,
        config_dir,
    })
}

// ===========================================================================
// Materialisation helpers
// ===========================================================================

impl RuntimeConfig {
    /// Produce the ordered list of capability requirements declared by the
    /// Tether Set.  These are pure data values suitable for passing to the
    /// policy resolver.
    pub fn capability_requirements(&self) -> Vec<crate::policy::CapabilityRequirement> {
        self.tether_set
            .capability_requirements
            .iter()
            .map(|req| {
                let mut cr = crate::policy::CapabilityRequirement::new(&req.name, req.version);
                if let Some(ref reason) = req.reason {
                    cr = cr.with_reason(reason);
                }
                cr
            })
            .collect()
    }

    /// Produce host-local policy from the configuration.
    ///
    /// This uses the existing `HostLocalPolicy` constructors without modifying
    /// `policy.rs`.  The default is always `Deny`, with per-capability rules
    /// converted to the `PolicyRule` enum.
    pub fn host_local_policy(&self) -> crate::policy::HostLocalPolicy {
        let mut policy = crate::policy::HostLocalPolicy::new(crate::policy::PolicyRule::Deny);
        for rule in &self.policy.rules {
            let rule_enum = match rule.decision {
                PolicyDecision::Allow => crate::policy::PolicyRule::Allow,
                PolicyDecision::Ask => crate::policy::PolicyRule::Ask,
                PolicyDecision::Deny => crate::policy::PolicyRule::Deny,
            };
            policy.insert(&rule.name, rule.version, rule_enum);
        }
        policy
    }
}

/// Materialised provider binding with scope information preserved.
///
/// Packet 2 owns live scope assessment.  This intermediate type carries
/// the scope binding declarations so that Packet 2 can wire them into
/// the host-owned scope assessor without modifying `provider.rs` or
/// `stdio_provider.rs`.
#[derive(Debug, Clone)]
pub struct ProviderMaterialization {
    /// The host-assigned provider identity.
    pub identity: String,
    /// Human-readable display name.
    pub display_name: String,
    /// Transport configuration (command, args, protocol_version).
    pub transport: TransportConfig,
    /// Per-capability materialized data including scope bindings.
    pub capabilities: Vec<CapabilityMaterialization>,
}

/// One materialized provider capability, including its scope binding.
#[derive(Debug, Clone)]
pub struct CapabilityMaterialization {
    pub name: String,
    pub version: u32,
    pub manifest_path: String,
    pub pinned_digest: String,
    pub scope_binding: Option<ScopeBindingConfig>,
}

impl ProviderMaterialization {
    /// Convert to a `provider::ProviderConfig` using only the public
    /// constructors and fields of `provider.rs`.  Scope bindings are
    /// deliberately excluded because `provider::AllowedCapability` has
    /// no scope-binding field – Packet 2 will use the
    /// `CapabilityMaterialization` values directly for scope assessment.
    pub fn to_provider_config(&self) -> crate::provider::ProviderConfig {
        crate::provider::ProviderConfig {
            identity: self.identity.clone(),
            display_name: self.display_name.clone(),
            allowed_capabilities: self
                .capabilities
                .iter()
                .map(|c| crate::provider::AllowedCapability {
                    capability_name: c.name.clone(),
                    capability_version: c.version,
                    pinned_digest: Some(c.pinned_digest.clone()),
                })
                .collect(),
        }
    }

    /// Convert to a `stdio_provider::StdioProviderConfig` using only the
    /// public fields of that struct.  Scope bindings are deliberately
    /// excluded.
    pub fn to_stdio_config(&self) -> crate::stdio_provider::StdioProviderConfig {
        crate::stdio_provider::StdioProviderConfig {
            command: self.transport.command.clone(),
            args: self.transport.args.clone(),
            protocol_version: self.transport.protocol_version.clone(),
            provider_config: self.to_provider_config(),
        }
    }
}

impl RuntimeConfig {
    /// Produce materialized provider bindings from the configuration.
    ///
    /// Returns `ProviderMaterialization` values that preserve scope-binding
    /// declarations alongside the data needed to construct `ProviderConfig`
    /// and `StdioProviderConfig`.  Packet 2 will use the scope bindings for
    /// live scope assessment; Packet 1 validates the declaration shape only.
    pub fn provider_materializations(&self) -> Vec<ProviderMaterialization> {
        self.providers
            .iter()
            .map(|p| ProviderMaterialization {
                identity: p.id.clone(),
                display_name: p.display_name.clone(),
                transport: p.transport.clone(),
                capabilities: p
                    .capabilities
                    .iter()
                    .map(|c| CapabilityMaterialization {
                        name: c.name.clone(),
                        version: c.version,
                        manifest_path: c.manifest_path.clone(),
                        pinned_digest: c.pinned_digest.clone(),
                        scope_binding: c.scope_binding.clone(),
                    })
                    .collect(),
            })
            .collect()
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    // ------------------------------------------------------------------
    // Helpers
    // ------------------------------------------------------------------

    fn minimal_config_json() -> serde_json::Value {
        serde_json::json!({
            "format_version": "0.1",
            "tether_set": {
                "id": "example.local",
                "version": "1",
                "tethers": [
                    {
                        "id": "record-completed-task",
                        "version": "demo-v1",
                        "source_path": "tethers/record-completed-task.tether"
                    }
                ],
                "capability_requirements": [
                    {
                        "name": "lantern.task.record",
                        "version": 1,
                        "reason": "Record a completed task"
                    }
                ]
            },
            "providers": [
                {
                    "id": "lantern-local",
                    "display_name": "Lantern Local",
                    "transport": {
                        "kind": "stdio",
                        "command": "pwsh.exe",
                        "args": ["-NoProfile", "-File", "providers/lantern.ps1"],
                        "protocol_version": "2025-11-25"
                    },
                    "capabilities": [
                        {
                            "name": "lantern.task.record",
                            "version": 1,
                            "manifest_path": "manifests/lantern-task-record.json",
                            "pinned_digest": "sha256:0000000000000000000000000000000000000000000000000000000000000000"
                        }
                    ]
                }
            ],
            "policy": {
                "default": "deny",
                "rules": [
                    {
                        "name": "lantern.task.record",
                        "version": 1,
                        "decision": "allow"
                    }
                ]
            }
        })
    }

    fn parse_ok(json: &serde_json::Value) -> RuntimeConfig {
        parse_runtime_config(&json.to_string()).unwrap()
    }

    fn parse_err(json: &serde_json::Value) -> RuntimeConfigError {
        parse_runtime_config(&json.to_string()).unwrap_err()
    }

    fn with_temp_dir<F>(test_fn: F)
    where
        F: FnOnce(&Path),
    {
        let dir = std::env::temp_dir().join(format!("j12-cfg-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            test_fn(&dir);
        }));
        let _ = std::fs::remove_dir_all(&dir);
        if let Err(e) = result {
            std::panic::resume_unwind(e);
        }
    }

    fn write_config(dir: &Path, name: &str, json: &serde_json::Value) -> PathBuf {
        let path = dir.join(name);
        let mut f = std::fs::File::create(&path).unwrap();
        write!(f, "{}", serde_json::to_string_pretty(json).unwrap()).unwrap();
        path
    }

    // ------------------------------------------------------------------
    // J12 Packet 1 focused tests
    // ------------------------------------------------------------------

    // 1. valid minimal configuration parses
    #[test]
    fn j12_packet1_valid_minimal_configuration_parses() {
        let cfg = parse_ok(&minimal_config_json());
        assert_eq!(cfg.format_version, "0.1");
        assert_eq!(cfg.tether_set.id, "example.local");
        assert_eq!(cfg.tether_set.tethers.len(), 1);
        assert_eq!(cfg.tether_set.capability_requirements.len(), 1);
        assert_eq!(cfg.providers.len(), 1);
        assert_eq!(cfg.policy.rules.len(), 1);
    }

    // 2. listed Tether order is preserved
    #[test]
    fn j12_packet1_tether_order_preserved() {
        let mut json = minimal_config_json();
        json["tether_set"]["tethers"] = serde_json::json!([
            {"id": "third", "version": "v1", "source_path": "third.tether"},
            {"id": "first", "version": "v1", "source_path": "first.tether"},
            {"id": "second", "version": "v1", "source_path": "second.tether"}
        ]);
        let cfg = parse_ok(&json);
        let ids: Vec<&str> = cfg
            .tether_set
            .tethers
            .iter()
            .map(|t| t.id.as_str())
            .collect();
        assert_eq!(ids, vec!["third", "first", "second"]);
    }

    // 3. duplicate key is rejected
    #[test]
    fn j12_packet1_duplicate_key_rejected() {
        // Construct JSON text with a duplicate key using string manipulation
        let json = r#"{"format_version":"0.1","format_version":"0.2","tether_set":{"id":"a","version":"1","tethers":[{"id":"t","version":"v1","source_path":"t.tether"}],"capability_requirements":[{"name":"c","version":1}]},"providers":[{"id":"p","display_name":"P","transport":{"kind":"stdio","command":"cmd","args":[],"protocol_version":"2025"},"capabilities":[{"name":"c","version":1,"manifest_path":"m.json","pinned_digest":"sha256:0000000000000000000000000000000000000000000000000000000000000000"}]}],"policy":{"default":"deny","rules":[{"name":"c","version":1,"decision":"allow"}]}}"#;
        let err = parse_runtime_config(json).unwrap_err();
        assert_eq!(err.code, RuntimeConfigErrorCode::InvalidJson);
    }

    // 4. unknown field is rejected
    #[test]
    fn j12_packet1_unknown_field_rejected() {
        let mut json = minimal_config_json();
        json.as_object_mut()
            .unwrap()
            .insert("extra_field".into(), serde_json::json!("value"));
        let err = parse_err(&json);
        assert_eq!(err.code, RuntimeConfigErrorCode::InvalidType);
    }

    // 5. wrong format version is rejected
    #[test]
    fn j12_packet1_wrong_format_version_rejected() {
        let mut json = minimal_config_json();
        json["format_version"] = serde_json::json!("2.0");
        let err = parse_err(&json);
        assert_eq!(err.code, RuntimeConfigErrorCode::UnknownFormatVersion);
    }

    // 6. empty Tether list is rejected
    #[test]
    fn j12_packet1_empty_tether_list_rejected() {
        let mut json = minimal_config_json();
        json["tether_set"]["tethers"] = serde_json::json!([]);
        let err = parse_err(&json);
        assert_eq!(err.code, RuntimeConfigErrorCode::InvalidValue);
    }

    // 7. duplicate Tether identity is rejected
    #[test]
    fn j12_packet1_duplicate_tether_identity_rejected() {
        let mut json = minimal_config_json();
        json["tether_set"]["tethers"] = serde_json::json!([
            {"id": "t", "version": "v1", "source_path": "a.tether"},
            {"id": "t", "version": "v1", "source_path": "b.tether"}
        ]);
        let err = parse_err(&json);
        assert_eq!(err.code, RuntimeConfigErrorCode::DuplicateEntry);
    }

    // 8. duplicate requirement is rejected
    #[test]
    fn j12_packet1_duplicate_requirement_rejected() {
        let mut json = minimal_config_json();
        json["tether_set"]["capability_requirements"] = serde_json::json!([
            {"name": "c", "version": 1},
            {"name": "c", "version": 1}
        ]);
        // Need a matching provider capability too
        json["providers"][0]["capabilities"] = serde_json::json!([
            {"name": "c", "version": 1, "manifest_path": "m.json", "pinned_digest": "sha256:0000000000000000000000000000000000000000000000000000000000000000"}
        ]);
        json["policy"]["rules"] = serde_json::json!([
            {"name": "c", "version": 1, "decision": "allow"}
        ]);
        let err = parse_err(&json);
        assert_eq!(err.code, RuntimeConfigErrorCode::DuplicateEntry);
    }

    // 9. duplicate provider identity is rejected
    #[test]
    fn j12_packet1_duplicate_provider_id_rejected() {
        let mut json = minimal_config_json();
        // This requires two requirements and two provider caps for cross-ref to pass
        json["tether_set"]["capability_requirements"] = serde_json::json!([
            {"name": "c1", "version": 1},
            {"name": "c2", "version": 1}
        ]);
        json["providers"] = serde_json::json!([
            {
                "id": "same-id",
                "display_name": "P1",
                "transport": {"kind": "stdio", "command": "cmd", "args": [], "protocol_version": "2025"},
                "capabilities": [
                    {"name": "c1", "version": 1, "manifest_path": "m1.json", "pinned_digest": "sha256:0000000000000000000000000000000000000000000000000000000000000000"}
                ]
            },
            {
                "id": "same-id",
                "display_name": "P2",
                "transport": {"kind": "stdio", "command": "cmd", "args": [], "protocol_version": "2025"},
                "capabilities": [
                    {"name": "c2", "version": 1, "manifest_path": "m2.json", "pinned_digest": "sha256:0000000000000000000000000000000000000000000000000000000000000000"}
                ]
            }
        ]);
        json["policy"]["rules"] = serde_json::json!([
            {"name": "c1", "version": 1, "decision": "allow"},
            {"name": "c2", "version": 1, "decision": "allow"}
        ]);
        let err = parse_err(&json);
        assert_eq!(err.code, RuntimeConfigErrorCode::DuplicateEntry);
    }

    // 10. duplicate provider capability is rejected
    #[test]
    fn j12_packet1_duplicate_provider_capability_rejected() {
        let mut json = minimal_config_json();
        json["providers"][0]["capabilities"] = serde_json::json!([
            {"name": "lantern.task.record", "version": 1, "manifest_path": "m1.json", "pinned_digest": "sha256:0000000000000000000000000000000000000000000000000000000000000000"},
            {"name": "lantern.task.record", "version": 1, "manifest_path": "m2.json", "pinned_digest": "sha256:0000000000000000000000000000000000000000000000000000000000000000"}
        ]);
        let err = parse_err(&json);
        assert_eq!(err.code, RuntimeConfigErrorCode::DuplicateEntry);
    }

    // 11. duplicate policy rule is rejected
    #[test]
    fn j12_packet1_duplicate_policy_rule_rejected() {
        let mut json = minimal_config_json();
        json["policy"]["rules"] = serde_json::json!([
            {"name": "lantern.task.record", "version": 1, "decision": "allow"},
            {"name": "lantern.task.record", "version": 1, "decision": "deny"}
        ]);
        let err = parse_err(&json);
        assert_eq!(err.code, RuntimeConfigErrorCode::DuplicateEntry);
    }

    // 12. missing provider capability for a requirement is rejected
    #[test]
    fn j12_packet1_missing_provider_capability_rejected() {
        let mut json = minimal_config_json();
        json["tether_set"]["capability_requirements"] = serde_json::json!([
            {"name": "lantern.task.record", "version": 1},
            {"name": "missing.cap", "version": 2}
        ]);
        let err = parse_err(&json);
        assert_eq!(err.code, RuntimeConfigErrorCode::UnmatchedReference);
    }

    // 13. unused provider capability is rejected
    #[test]
    fn j12_packet1_unused_provider_capability_rejected() {
        let mut json = minimal_config_json();
        json["providers"][0]["capabilities"] = serde_json::json!([
            {"name": "lantern.task.record", "version": 1, "manifest_path": "m1.json", "pinned_digest": "sha256:0000000000000000000000000000000000000000000000000000000000000000"},
            {"name": "unused.cap", "version": 1, "manifest_path": "m2.json", "pinned_digest": "sha256:0000000000000000000000000000000000000000000000000000000000000000"}
        ]);
        let err = parse_err(&json);
        assert_eq!(err.code, RuntimeConfigErrorCode::UnmatchedReference);
    }

    // 14. policy rule for undeclared capability is rejected
    #[test]
    fn j12_packet1_policy_rule_undeclared_rejected() {
        let mut json = minimal_config_json();
        json["policy"]["rules"] = serde_json::json!([
            {"name": "undeclared.cap", "version": 1, "decision": "allow"}
        ]);
        let err = parse_err(&json);
        assert_eq!(err.code, RuntimeConfigErrorCode::UnmatchedReference);
    }

    // 15. invalid pinned digest is rejected
    #[test]
    fn j12_packet1_invalid_pinned_digest_rejected() {
        let mut json = minimal_config_json();
        json["providers"][0]["capabilities"][0]["pinned_digest"] = serde_json::json!("sha256:bad");
        let err = parse_err(&json);
        assert_eq!(err.code, RuntimeConfigErrorCode::InvalidValue);
    }

    // 16. absolute Tether source path is rejected
    #[test]
    fn j12_packet1_absolute_source_path_rejected() {
        let mut json = minimal_config_json();
        json["tether_set"]["tethers"][0]["source_path"] =
            serde_json::json!("C:\\absolute\\path.tether");
        let err = parse_err(&json);
        assert_eq!(err.code, RuntimeConfigErrorCode::InvalidValue);
    }

    // 17. absolute manifest path is rejected
    #[test]
    fn j12_packet1_absolute_manifest_path_rejected() {
        let mut json = minimal_config_json();
        // Use a Windows absolute path (with drive letter) which Path::is_absolute()
        // recognises on Windows.
        json["providers"][0]["capabilities"][0]["manifest_path"] =
            serde_json::json!("C:\\absolute\\path.json");
        let err = parse_err(&json);
        assert_eq!(err.code, RuntimeConfigErrorCode::InvalidValue);
    }

    // 18. unsupported transport is rejected
    #[test]
    fn j12_packet1_unsupported_transport_rejected() {
        let mut json = minimal_config_json();
        json["providers"][0]["transport"]["kind"] = serde_json::json!("http");
        let err = parse_err(&json);
        assert_eq!(err.code, RuntimeConfigErrorCode::InvalidType);
    }

    // 19. non-deny default is rejected
    #[test]
    fn j12_packet1_non_deny_default_rejected() {
        let mut json = minimal_config_json();
        json["policy"]["default"] = serde_json::json!("allow");
        let err = parse_err(&json);
        assert_eq!(err.code, RuntimeConfigErrorCode::InvalidValue);
    }

    // 20. malformed scope pointer is rejected
    #[test]
    fn j12_packet1_malformed_scope_pointer_rejected() {
        let mut json = minimal_config_json();
        json["providers"][0]["capabilities"][0]["scope_binding"] = serde_json::json!({
            "kind": "path_prefix",
            "argument_json_pointer": "no-leading-slash"
        });
        let err = parse_err(&json);
        assert_eq!(err.code, RuntimeConfigErrorCode::InvalidValue);
    }

    // 21. valid loaded configuration resolves relative paths against the config directory
    #[test]
    fn j12_packet1_loaded_config_resolves_relative_paths() {
        with_temp_dir(|dir| {
            let config_path = write_config(dir, "tethers-config.json", &minimal_config_json());
            let loaded = load_runtime_config(&config_path).unwrap();

            // config path and dir should be absolute
            assert!(loaded.config_path.is_absolute());
            assert!(loaded.config_dir.is_absolute());
            assert_eq!(loaded.config_path, config_path.canonicalize().unwrap());
            assert_eq!(loaded.config_dir, dir.canonicalize().unwrap());

            // Tether source path is resolved relative to config dir
            let resolved = loaded.tether_source_path("tethers/record-completed-task.tether");
            assert_eq!(
                resolved,
                dir.canonicalize()
                    .unwrap()
                    .join("tethers/record-completed-task.tether")
            );

            // Manifest path is resolved relative to config dir
            let manifest = loaded.manifest_path("manifests/lantern-task-record.json");
            assert_eq!(
                manifest,
                dir.canonicalize()
                    .unwrap()
                    .join("manifests/lantern-task-record.json")
            );
        });
    }

    // 22. manifest duplicate-key rejection behaviour remains unchanged
    #[test]
    fn j12_packet1_manifest_duplicate_key_unchanged() {
        // Verify that parsing a valid manifest still works
        use crate::manifest;
        let json = r#"{
            "manifest_format_version": "1.0",
            "capability_name": "test.cap",
            "capability_version": 1,
            "title": "Test Manifest",
            "description": "A test capability manifest.",
            "input_schema": {"type": "object", "properties": {"path": {"type": "string"}}},
            "output_schema": {"type": "object", "properties": {"status": {"type": "string"}}},
            "effects": ["test.create"],
            "permission_scope": null,
            "reversibility": "irreversible",
            "determinism": "deterministic",
            "idempotency": {"mechanism": "none"},
            "confirmation_policy": {"standing_permitted": false, "per_call_required": true},
            "timeout_ms": 5000,
            "retry_policy": {"max_retries": 0, "backoff_ms": 0, "allowed_on": [], "requires_idempotency_proof": false},
            "provider": {"identity": "test-provider", "display_name": "Test Provider", "identity_source": "host_configuration"},
            "binding": {"kind": "mcp", "server_name": "test-server", "tool_name": "test_tool"},
            "digest": "sha256:0000000000000000000000000000000000000000000000000000000000000000"
        }"#;
        // This should parse just fine (digest will mismatch, but the parsing
        // and structural validation should succeed)
        let parsed = manifest::TrustedManifest::parse(json);
        assert!(
            parsed.is_ok(),
            "manifest parse should succeed: {:?}",
            parsed.err()
        );

        // Now verify that duplicate keys are still rejected at the manifest level
        let dup_json = r#"{"manifest_format_version":"1.0","manifest_format_version":"2.0","capability_name":"test.cap","capability_version":1,"title":"Test","description":"Desc","input_schema":{},"output_schema":{"type":"object"},"effects":["test.read"],"permission_scope":null,"reversibility":"reversible","determinism":"deterministic","idempotency":{"mechanism":"none"},"confirmation_policy":{"standing_permitted":true,"per_call_required":false},"timeout_ms":5000,"retry_policy":{"max_retries":0,"backoff_ms":0,"allowed_on":[],"requires_idempotency_proof":false},"provider":{"identity":"p","display_name":"P","identity_source":"host_configuration"},"binding":{"kind":"mcp","server_name":"s","tool_name":"t"}}"#;
        let err = manifest::TrustedManifest::parse(dup_json).unwrap_err();
        assert_eq!(err.code, manifest::ManifestErrorCode::InvalidJson);
    }

    // ------------------------------------------------------------------
    // Additional validation tests
    // ------------------------------------------------------------------

    #[test]
    fn j12_packet1_missing_format_version_rejected() {
        let json = r#"{"tether_set":{"id":"a","version":"1","tethers":[{"id":"t","version":"v1","source_path":"t.tether"}],"capability_requirements":[{"name":"c","version":1}]},"providers":[{"id":"p","display_name":"P","transport":{"kind":"stdio","command":"cmd","args":[],"protocol_version":"2025"},"capabilities":[{"name":"c","version":1,"manifest_path":"m.json","pinned_digest":"sha256:0000000000000000000000000000000000000000000000000000000000000000"}]}],"policy":{"default":"deny","rules":[]}}"#;
        let err = parse_runtime_config(json).unwrap_err();
        assert_eq!(err.code, RuntimeConfigErrorCode::InvalidType);
    }

    #[test]
    fn j12_packet1_empty_provider_id_rejected() {
        let mut json = minimal_config_json();
        json["providers"][0]["id"] = serde_json::json!("   ");
        let err = parse_err(&json);
        assert_eq!(err.code, RuntimeConfigErrorCode::InvalidValue);
    }

    #[test]
    fn j12_packet1_empty_source_path_rejected() {
        let mut json = minimal_config_json();
        json["tether_set"]["tethers"][0]["source_path"] = serde_json::json!("");
        let err = parse_err(&json);
        assert_eq!(err.code, RuntimeConfigErrorCode::InvalidValue);
    }

    #[test]
    fn j12_packet1_empty_manifest_path_rejected() {
        let mut json = minimal_config_json();
        json["providers"][0]["capabilities"][0]["manifest_path"] = serde_json::json!("");
        let err = parse_err(&json);
        assert_eq!(err.code, RuntimeConfigErrorCode::InvalidValue);
    }

    #[test]
    fn j12_packet1_zero_capability_version_rejected() {
        let mut json = minimal_config_json();
        json["tether_set"]["capability_requirements"][0]["version"] = serde_json::json!(0);
        let err = parse_err(&json);
        assert_eq!(err.code, RuntimeConfigErrorCode::InvalidValue);
    }

    #[test]
    fn j12_packet1_empty_scope_pointer_rejected() {
        let mut json = minimal_config_json();
        json["providers"][0]["capabilities"][0]["scope_binding"] = serde_json::json!({
            "kind": "path_prefix",
            "argument_json_pointer": ""
        });
        let err = parse_err(&json);
        assert_eq!(err.code, RuntimeConfigErrorCode::InvalidValue);
    }

    #[test]
    fn j12_packet1_materialization_produces_correct_requirements() {
        let cfg = parse_ok(&minimal_config_json());
        let reqs = cfg.capability_requirements();
        assert_eq!(reqs.len(), 1);
        assert_eq!(reqs[0].capability_name, "lantern.task.record");
        assert_eq!(reqs[0].capability_version, 1);
        assert_eq!(reqs[0].reason.as_deref(), Some("Record a completed task"));
    }

    #[test]
    fn j12_packet1_materialization_produces_correct_policy() {
        let cfg = parse_ok(&minimal_config_json());
        let policy = cfg.host_local_policy();
        // Default is deny; one allow rule exists
        let rule = policy.rule_for("lantern.task.record", 1);
        assert_eq!(rule, crate::policy::PolicyRule::Allow);
        // Unlisted capability gets default deny
        let unknown = policy.rule_for("unknown", 1);
        assert_eq!(unknown, crate::policy::PolicyRule::Deny);
    }

    #[test]
    fn j12_packet1_provider_materialization_preserves_scope() {
        let mut json = minimal_config_json();
        json["providers"][0]["capabilities"][0]["scope_binding"] = serde_json::json!({
            "kind": "path_prefix",
            "argument_json_pointer": "/path"
        });
        let cfg = parse_ok(&json);
        let mats = cfg.provider_materializations();
        assert_eq!(mats.len(), 1);
        assert_eq!(mats[0].identity, "lantern-local");
        assert_eq!(mats[0].capabilities.len(), 1);
        let scope = mats[0].capabilities[0].scope_binding.as_ref().unwrap();
        assert_eq!(scope.kind, ScopeBindingKind::PathPrefix);
        assert_eq!(scope.argument_json_pointer, "/path");
    }

    #[test]
    fn j12_packet1_provider_config_from_materialization() {
        let cfg = parse_ok(&minimal_config_json());
        let mats = cfg.provider_materializations();
        let pc = mats[0].to_provider_config();
        assert_eq!(pc.identity, "lantern-local");
        assert_eq!(pc.display_name, "Lantern Local");
        assert_eq!(pc.allowed_capabilities.len(), 1);
        assert_eq!(
            pc.allowed_capabilities[0].capability_name,
            "lantern.task.record"
        );
        assert_eq!(pc.allowed_capabilities[0].capability_version, 1);
        assert_eq!(
            pc.allowed_capabilities[0].pinned_digest.as_deref(),
            Some("sha256:0000000000000000000000000000000000000000000000000000000000000000")
        );
    }

    // ------------------------------------------------------------------
    // Global scoped-identity validation
    // ------------------------------------------------------------------

    // 23. capability identity duplicated across providers is rejected (one scoped, one unscoped)
    #[test]
    fn j12_packet1_scoped_identity_duplicated_in_another_provider_rejected() {
        // Two providers each carry the same (name, version).  One has
        // scope_binding set; the other does not.  J12 Packet 2 rejects all
        // cross-provider duplicates regardless of scope_binding.
        let json = serde_json::json!({
            "format_version": "0.1",
            "tether_set": {
                "id": "example.local",
                "version": "1",
                "tethers": [
                    {"id": "t", "version": "v1", "source_path": "t.tether"}
                ],
                "capability_requirements": [
                    {"name": "lantern.task.record", "version": 1}
                ]
            },
            "providers": [
                {
                    "id": "lantern-a",
                    "display_name": "Lantern A",
                    "transport": {
                        "kind": "stdio",
                        "command": "pwsh.exe",
                        "args": [],
                        "protocol_version": "2025-11-25"
                    },
                    "capabilities": [
                        {
                            "name": "lantern.task.record",
                            "version": 1,
                            "manifest_path": "manifests/a.json",
                            "pinned_digest": "sha256:0000000000000000000000000000000000000000000000000000000000000000",
                            "scope_binding": {
                                "kind": "path_prefix",
                                "argument_json_pointer": "/path"
                            }
                        }
                    ]
                },
                {
                    "id": "lantern-b",
                    "display_name": "Lantern B",
                    "transport": {
                        "kind": "stdio",
                        "command": "pwsh.exe",
                        "args": [],
                        "protocol_version": "2025-11-25"
                    },
                    "capabilities": [
                        {
                            "name": "lantern.task.record",
                            "version": 1,
                            "manifest_path": "manifests/b.json",
                            "pinned_digest": "sha256:0000000000000000000000000000000000000000000000000000000000000000"
                        }
                    ]
                }
            ],
            "policy": {
                "default": "deny",
                "rules": [
                    {"name": "lantern.task.record", "version": 1, "decision": "allow"}
                ]
            }
        });
        let err = parse_err(&json);
        assert_eq!(err.code, RuntimeConfigErrorCode::DuplicateEntry);
        assert!(
            err.message.contains("globally unique"),
            "expected global-uniqueness error, got: {}",
            err.message
        );
    }

    // 24. duplicate across providers rejected regardless of which provider is scoped
    #[test]
    fn j12_packet1_scoped_identity_duplicate_order_independent() {
        // Swap the scope binding to the second provider.  Rejection must
        // still fire because the exact identity count is still > 1.
        let json = serde_json::json!({
            "format_version": "0.1",
            "tether_set": {
                "id": "example.local",
                "version": "1",
                "tethers": [
                    {"id": "t", "version": "v1", "source_path": "t.tether"}
                ],
                "capability_requirements": [
                    {"name": "lantern.task.record", "version": 1}
                ]
            },
            "providers": [
                {
                    "id": "lantern-a",
                    "display_name": "Lantern A",
                    "transport": {
                        "kind": "stdio",
                        "command": "pwsh.exe",
                        "args": [],
                        "protocol_version": "2025-11-25"
                    },
                    "capabilities": [
                        {
                            "name": "lantern.task.record",
                            "version": 1,
                            "manifest_path": "manifests/a.json",
                            "pinned_digest": "sha256:0000000000000000000000000000000000000000000000000000000000000000"
                        }
                    ]
                },
                {
                    "id": "lantern-b",
                    "display_name": "Lantern B",
                    "transport": {
                        "kind": "stdio",
                        "command": "pwsh.exe",
                        "args": [],
                        "protocol_version": "2025-11-25"
                    },
                    "capabilities": [
                        {
                            "name": "lantern.task.record",
                            "version": 1,
                            "manifest_path": "manifests/b.json",
                            "pinned_digest": "sha256:0000000000000000000000000000000000000000000000000000000000000000",
                            "scope_binding": {
                                "kind": "path_prefix",
                                "argument_json_pointer": "/path"
                            }
                        }
                    ]
                }
            ],
            "policy": {
                "default": "deny",
                "rules": [
                    {"name": "lantern.task.record", "version": 1, "decision": "allow"}
                ]
            }
        });
        let err = parse_err(&json);
        assert_eq!(err.code, RuntimeConfigErrorCode::DuplicateEntry);
        assert!(
            err.message.contains("globally unique"),
            "expected global-uniqueness error, got: {}",
            err.message
        );
    }

    // 25. same name at different versions is NOT rejected (distinct identities)
    #[test]
    fn j12_packet1_same_name_different_version_scoped_not_rejected() {
        // Both entries have scope_binding but differ in version, so their
        // identity is not duplicated.  The config must parse successfully.
        let json = serde_json::json!({
            "format_version": "0.1",
            "tether_set": {
                "id": "example.local",
                "version": "1",
                "tethers": [
                    {"id": "t", "version": "v1", "source_path": "t.tether"}
                ],
                "capability_requirements": [
                    {"name": "lantern.task.record", "version": 1},
                    {"name": "lantern.task.record", "version": 2}
                ]
            },
            "providers": [
                {
                    "id": "lantern-local",
                    "display_name": "Lantern Local",
                    "transport": {
                        "kind": "stdio",
                        "command": "pwsh.exe",
                        "args": [],
                        "protocol_version": "2025-11-25"
                    },
                    "capabilities": [
                        {
                            "name": "lantern.task.record",
                            "version": 1,
                            "manifest_path": "manifests/a.json",
                            "pinned_digest": "sha256:0000000000000000000000000000000000000000000000000000000000000000",
                            "scope_binding": {
                                "kind": "path_prefix",
                                "argument_json_pointer": "/path"
                            }
                        },
                        {
                            "name": "lantern.task.record",
                            "version": 2,
                            "manifest_path": "manifests/b.json",
                            "pinned_digest": "sha256:0000000000000000000000000000000000000000000000000000000000000000",
                            "scope_binding": {
                                "kind": "path_prefix",
                                "argument_json_pointer": "/other"
                            }
                        }
                    ]
                }
            ],
            "policy": {
                "default": "deny",
                "rules": [
                    {"name": "lantern.task.record", "version": 1, "decision": "allow"},
                    {"name": "lantern.task.record", "version": 2, "decision": "allow"}
                ]
            }
        });
        let cfg = parse_ok(&json);
        assert_eq!(cfg.providers[0].capabilities.len(), 2);
    }

    // ------------------------------------------------------------------
    // J12 Packet 2 global exact-identity uniqueness
    // ------------------------------------------------------------------

    // 26. duplicate unscoped exact identity across providers is rejected
    #[test]
    fn j12_packet2_duplicate_unscoped_across_providers_rejected() {
        // Two providers carry the same (name, version) with no scope_binding.
        // J12 Packet 2 rejects all cross-provider duplicates.
        let json = serde_json::json!({
            "format_version": "0.1",
            "tether_set": {
                "id": "example.local",
                "version": "1",
                "tethers": [
                    {"id": "t", "version": "v1", "source_path": "t.tether"}
                ],
                "capability_requirements": [
                    {"name": "lantern.task.record", "version": 1}
                ]
            },
            "providers": [
                {
                    "id": "lantern-a",
                    "display_name": "Lantern A",
                    "transport": {
                        "kind": "stdio",
                        "command": "pwsh.exe",
                        "args": [],
                        "protocol_version": "2025-11-25"
                    },
                    "capabilities": [
                        {
                            "name": "lantern.task.record",
                            "version": 1,
                            "manifest_path": "manifests/a.json",
                            "pinned_digest": "sha256:0000000000000000000000000000000000000000000000000000000000000000"
                        }
                    ]
                },
                {
                    "id": "lantern-b",
                    "display_name": "Lantern B",
                    "transport": {
                        "kind": "stdio",
                        "command": "pwsh.exe",
                        "args": [],
                        "protocol_version": "2025-11-25"
                    },
                    "capabilities": [
                        {
                            "name": "lantern.task.record",
                            "version": 1,
                            "manifest_path": "manifests/b.json",
                            "pinned_digest": "sha256:0000000000000000000000000000000000000000000000000000000000000000"
                        }
                    ]
                }
            ],
            "policy": {
                "default": "deny",
                "rules": [
                    {"name": "lantern.task.record", "version": 1, "decision": "allow"}
                ]
            }
        });
        let err = parse_err(&json);
        assert_eq!(err.code, RuntimeConfigErrorCode::DuplicateEntry);
        assert!(
            err.message.contains("globally unique"),
            "expected global-uniqueness error, got: {}",
            err.message
        );
    }
}
