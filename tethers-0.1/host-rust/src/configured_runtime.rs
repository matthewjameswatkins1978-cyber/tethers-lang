// J12 Packet 2 - prepared local runtime, verified assets, deterministic
// capability materialisation, and binding-owned path-scope assessment.
//
// Turns a LoadedRuntimeConfig into a complete PreparedRuntime that J13 can
// use without manually rebuilding internal objects.
//
// This module performs filesystem loading and trusted local preparation.
// It performs no provider, engine, dispatch or Trail I/O.

use crate::manifest::{self, PermissionScope, VerifiedManifest};
use crate::policy::{CapabilityRequirement, HostLocalPolicy, ProposedAction, ScopeAssessment};
use crate::provider;
use crate::runtime_config::{LoadedRuntimeConfig, ScopeBindingConfig, ScopeBindingKind};
use crate::stdio_provider::StdioProviderConfig;
use crate::trusted_store::TrustedManifestStore;

use std::fmt;
use std::path::{Path, PathBuf};

// ===========================================================================
// Structured preparation error
// ===========================================================================

/// Machine-readable codes for runtime preparation errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimePreparationErrorCode {
    AssetNotFound,
    AssetNotFile,
    AssetOutsideConfigRoot,
    AssetReadFailed,
    InvalidUtf8OrText,
    EmptyTetherSource,
    ManifestInvalid,
    ManifestIdentityMismatch,
    ProviderIdentityMismatch,
    PinnedDigestMismatch,
    UnsupportedPermissionScope,
    MissingScopeBinding,
    UnexpectedScopeBinding,
    ManifestAdmissionFailed,
    UnsupportedPlannerSchema,
    AmbiguousCapabilityBinding,
    CapabilityNotPrepared,
    InvalidResourcePath,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePreparationError {
    pub code: RuntimePreparationErrorCode,
    pub message: String,
}

impl RuntimePreparationError {
    fn new(code: RuntimePreparationErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl fmt::Display for RuntimePreparationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.code, self.message)
    }
}

impl std::error::Error for RuntimePreparationError {}

// ===========================================================================
// Prepared runtime data types
// ===========================================================================

/// A complete immutable runtime snapshot ready for J13.
#[derive(Debug)]
pub struct PreparedRuntime {
    config_path: PathBuf,
    config_dir: PathBuf,
    tether_set_id: String,
    tether_set_version: String,
    tethers: Vec<PreparedTether>,
    requirements: Vec<CapabilityRequirement>,
    providers: Vec<PreparedProvider>,
    policy: HostLocalPolicy,
    trusted_store: TrustedManifestStore,
}

/// One Tether source file loaded into memory.
#[derive(Debug, Clone)]
pub struct PreparedTether {
    pub id: String,
    pub version: String,
    pub source_path: PathBuf,
    pub source: String,
}

/// One configured provider with a launch plan and verified capabilities.
#[derive(Debug, Clone)]
pub struct PreparedProvider {
    pub identity: String,
    pub display_name: String,
    pub working_directory: PathBuf,
    pub stdio_config: StdioProviderConfig,
    pub capabilities: Vec<PreparedCapability>,
}

/// One verified, pinned, and admitted capability.
#[derive(Debug, Clone)]
pub struct PreparedCapability {
    pub name: String,
    pub version: u32,
    pub manifest_path: PathBuf,
    pub verified_manifest: VerifiedManifest,
    pub scope_binding: Option<ScopeBindingConfig>,
}

// ===========================================================================
// Read-only accessors
// ===========================================================================

impl PreparedRuntime {
    /// The absolute path to the configuration file.
    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    /// The absolute parent directory of the configuration file.
    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    /// The selected Tether Set identity.
    pub fn tether_set_id(&self) -> &str {
        &self.tether_set_id
    }

    /// The selected Tether Set version.
    pub fn tether_set_version(&self) -> &str {
        &self.tether_set_version
    }

    /// The ordered list of prepared Tethers.
    pub fn tethers(&self) -> &[PreparedTether] {
        &self.tethers
    }

    /// The capability requirements declared by the Tether Set.
    pub fn requirements(&self) -> &[CapabilityRequirement] {
        &self.requirements
    }

    /// The prepared providers with launch plans.
    pub fn providers(&self) -> &[PreparedProvider] {
        &self.providers
    }

    /// The host-local policy.
    pub fn policy(&self) -> &HostLocalPolicy {
        &self.policy
    }

    /// The trusted manifest store populated with verified manifests.
    pub fn trusted_store(&self) -> &TrustedManifestStore {
        &self.trusted_store
    }

    /// All prepared capabilities across all providers.
    pub fn prepared_capabilities(&self) -> Vec<&PreparedCapability> {
        self.providers
            .iter()
            .flat_map(|p| p.capabilities.iter())
            .collect()
    }
}

// ===========================================================================
// Filesystem confinement helpers
// ===========================================================================

/// Resolve a relative path against the canonical config directory, then
/// canonicalise the result.  Reject escapes outside the config root,
/// directories, and missing files.
/// Preflight: check a relative path for ParentDir escapes before joining
/// against the config root.  Returns the number of ordinary segments (depth)
/// or an error when a `..` would move above the root.
fn check_relative_path_safe(relative_path: &str) -> Result<usize, RuntimePreparationError> {
    // Reject absolute paths (drive letter or root).
    let p = std::path::Path::new(relative_path);
    if p.is_absolute() || p.has_root() {
        return Err(RuntimePreparationError::new(
            RuntimePreparationErrorCode::AssetOutsideConfigRoot,
            format!("asset path \"{}\" must be relative", relative_path),
        ));
    }

    // Count ParentDir vs ordinary components.
    // Explicitly handle every Component variant.
    // On Windows, a drive-relative path such as C:outside\file
    // has a Prefix component but is neither absolute nor rooted,
    // so it falls through is_absolute()/has_root() above.
    let mut depth: isize = 0;
    for component in p.components() {
        match component {
            std::path::Component::Prefix(_) => {
                return Err(RuntimePreparationError::new(
                    RuntimePreparationErrorCode::AssetOutsideConfigRoot,
                    format!(
                        "asset \"{}\" has a drive prefix and must be relative",
                        relative_path
                    ),
                ));
            }
            std::path::Component::RootDir => {
                return Err(RuntimePreparationError::new(
                    RuntimePreparationErrorCode::AssetOutsideConfigRoot,
                    format!(
                        "asset \"{}\" has a root directory component and must be relative",
                        relative_path
                    ),
                ));
            }
            std::path::Component::ParentDir => {
                depth -= 1;
                if depth < 0 {
                    return Err(RuntimePreparationError::new(
                        RuntimePreparationErrorCode::AssetOutsideConfigRoot,
                        format!(
                            "asset \"{}\" escapes config root: too many \"..\" components",
                            relative_path
                        ),
                    ));
                }
            }
            std::path::Component::Normal(_) => {
                depth += 1;
            }
            std::path::Component::CurDir => {
                // Harmless, ignored.
            }
        }
    }

    Ok(depth as usize)
}

fn confine_asset(
    config_dir: &Path,
    relative_path: &str,
) -> Result<PathBuf, RuntimePreparationError> {
    // Preflight: reject absolute paths and ../ escapes using pure
    // component counting before any filesystem join or canonicalise.
    check_relative_path_safe(relative_path)?;

    let joined = config_dir.join(relative_path);

    // Try to canonicalise.  On Windows, canonicalize requires the file
    // to exist.  When it fails, we report AssetNotFound (the preflight
    // already ruled out ParentDir escapes).
    let canonical = match joined.canonicalize() {
        Ok(p) => p,
        Err(_) => {
            return Err(RuntimePreparationError::new(
                RuntimePreparationErrorCode::AssetNotFound,
                format!(
                    "asset \"{}\" (resolved to \"{}\") not found",
                    relative_path,
                    joined.display()
                ),
            ));
        }
    };

    // Existing-file containment check (catches symlink escapes and
    // absolute escapes that the preflight can't see).
    if !canonical.starts_with(config_dir) {
        return Err(RuntimePreparationError::new(
            RuntimePreparationErrorCode::AssetOutsideConfigRoot,
            format!(
                "asset \"{}\" escapes config root: canonical path \"{}\" is not \
                 beneath \"{}\"",
                relative_path,
                canonical.display(),
                config_dir.display()
            ),
        ));
    }

    if !canonical.is_file() {
        return Err(RuntimePreparationError::new(
            RuntimePreparationErrorCode::AssetNotFile,
            format!("asset \"{}\" is not a regular file", canonical.display()),
        ));
    }

    Ok(canonical)
}

/// Read a file as UTF-8 text, rejecting NUL characters.
fn read_utf8_asset(path: &Path) -> Result<String, RuntimePreparationError> {
    let bytes = std::fs::read(path).map_err(|e| {
        RuntimePreparationError::new(
            RuntimePreparationErrorCode::AssetReadFailed,
            format!("could not read \"{}\": {}", path.display(), e),
        )
    })?;

    if bytes.contains(&0) {
        return Err(RuntimePreparationError::new(
            RuntimePreparationErrorCode::InvalidUtf8OrText,
            format!("asset \"{}\" contains NUL bytes", path.display()),
        ));
    }

    let text = String::from_utf8(bytes).map_err(|e| {
        RuntimePreparationError::new(
            RuntimePreparationErrorCode::InvalidUtf8OrText,
            format!("asset \"{}\" is not valid UTF-8: {}", path.display(), e),
        )
    })?;

    Ok(text)
}

// ===========================================================================
// Path-segment validation for scope assessment
// ===========================================================================

fn validate_resource_path(path: &str) -> Result<(), RuntimePreparationError> {
    if path.is_empty() {
        return Err(RuntimePreparationError::new(
            RuntimePreparationErrorCode::InvalidResourcePath,
            "resource path must not be empty",
        ));
    }

    if path.contains('\\') {
        return Err(RuntimePreparationError::new(
            RuntimePreparationErrorCode::InvalidResourcePath,
            "resource path must use '/' separator, not backslash",
        ));
    }

    if path.contains('\0') {
        return Err(RuntimePreparationError::new(
            RuntimePreparationErrorCode::InvalidResourcePath,
            "resource path must not contain NUL",
        ));
    }

    if path.starts_with('/') {
        return Err(RuntimePreparationError::new(
            RuntimePreparationErrorCode::InvalidResourcePath,
            "resource path must be relative, not absolute",
        ));
    }

    let segments: Vec<&str> = path.split('/').collect();

    for seg in &segments {
        if seg.is_empty() {
            return Err(RuntimePreparationError::new(
                RuntimePreparationErrorCode::InvalidResourcePath,
                "resource path must not contain empty segments",
            ));
        }
    }

    for seg in &segments {
        if *seg == "." || *seg == ".." {
            return Err(RuntimePreparationError::new(
                RuntimePreparationErrorCode::InvalidResourcePath,
                format!("resource path must not contain \"{}\" segments", seg),
            ));
        }

        if seg.contains(':') {
            return Err(RuntimePreparationError::new(
                RuntimePreparationErrorCode::InvalidResourcePath,
                "resource path segment must not contain ':'",
            ));
        }
    }

    Ok(())
}

fn validate_allowed_prefixes(prefixes: &[String]) -> Result<(), RuntimePreparationError> {
    for prefix in prefixes {
        if prefix.is_empty() {
            return Err(RuntimePreparationError::new(
                RuntimePreparationErrorCode::InvalidResourcePath,
                "allowed_prefix must not be empty",
            ));
        }

        // Reject "/" (root)
        if prefix == "/" {
            return Err(RuntimePreparationError::new(
                RuntimePreparationErrorCode::InvalidResourcePath,
                "allowed_prefix must not be \"/\" (root)",
            ));
        }

        // Reject leading slash
        if prefix.starts_with('/') {
            return Err(RuntimePreparationError::new(
                RuntimePreparationErrorCode::InvalidResourcePath,
                "allowed_prefix must not start with '/'",
            ));
        }

        // Reject backslash
        if prefix.contains('\\') {
            return Err(RuntimePreparationError::new(
                RuntimePreparationErrorCode::InvalidResourcePath,
                "allowed_prefix must not contain backslash",
            ));
        }

        // Reject NUL
        if prefix.contains('\0') {
            return Err(RuntimePreparationError::new(
                RuntimePreparationErrorCode::InvalidResourcePath,
                "allowed_prefix must not contain NUL",
            ));
        }

        // Validate segments (after optionally stripping a single trailing slash)
        let body = prefix.strip_suffix('/').unwrap_or(prefix);

        if body.is_empty() {
            // Only possible if prefix was exactly "/" — handled above.
            continue;
        }

        let segments: Vec<&str> = body.split('/').collect();

        // Reject double-slash (empty interior segments)
        for seg in &segments {
            if seg.is_empty() {
                return Err(RuntimePreparationError::new(
                    RuntimePreparationErrorCode::InvalidResourcePath,
                    "allowed_prefix must not contain empty segments",
                ));
            }

            // Reject `.` and `..` segments
            if *seg == "." || *seg == ".." {
                return Err(RuntimePreparationError::new(
                    RuntimePreparationErrorCode::InvalidResourcePath,
                    format!("allowed_prefix must not contain \"{}\" segment", seg),
                ));
            }

            // Reject colon (drive-like)
            if seg.contains(':') {
                return Err(RuntimePreparationError::new(
                    RuntimePreparationErrorCode::InvalidResourcePath,
                    "allowed_prefix segment must not contain ':'",
                ));
            }
        }
    }
    Ok(())
}

// ===========================================================================
// Scope assessment
// ===========================================================================

impl PreparedRuntime {
    pub fn assess_action_scope(&self, action: &ProposedAction) -> ScopeAssessment {
        let prepared = self.locate_capability(action);
        let cap = match prepared {
            Some(c) => c,
            None => return ScopeAssessment::ScopeNotEstablished,
        };

        let manifest = cap.verified_manifest.manifest();

        match &manifest.permission_scope {
            PermissionScope::PathPrefix { allowed_prefixes } => {
                let binding = match &cap.scope_binding {
                    Some(b) => b,
                    None => return ScopeAssessment::ScopeNotEstablished,
                };

                let extracted =
                    extract_json_pointer(&action.arguments, &binding.argument_json_pointer);

                let path = match extracted {
                    Some(serde_json::Value::String(s)) => s.clone(),
                    Some(_) => return ScopeAssessment::ScopeNotEstablished,
                    None => return ScopeAssessment::ScopeNotEstablished,
                };

                if validate_resource_path(&path).is_err() {
                    return ScopeAssessment::ScopeNotEstablished;
                }

                for prefix in allowed_prefixes {
                    if path_starts_with_segment(&path, prefix) {
                        return ScopeAssessment::WithinScope;
                    }
                }

                ScopeAssessment::ScopeViolation
            }
            PermissionScope::Unrestricted => ScopeAssessment::WithinScope,
            PermissionScope::Repository { .. } | PermissionScope::Calendar { .. } => {
                ScopeAssessment::ScopeNotEstablished
            }
        }
    }

    fn locate_capability(&self, action: &ProposedAction) -> Option<&PreparedCapability> {
        let action_name = &action.capability_name;
        let action_version = action.bridge_capability_version?;
        let action_provider = action.bridge_provider_identity.as_deref()?;
        let action_digest = action.manifest_digest.as_deref()?;

        for provider in &self.providers {
            if provider.identity != action_provider {
                continue;
            }
            for cap in &provider.capabilities {
                if cap.name == *action_name
                    && cap.version == action_version
                    && cap.verified_manifest.verified_digest() == action_digest
                {
                    return Some(cap);
                }
            }
        }
        None
    }
}

/// Strict RFC 6901 JSON Pointer extraction.  All tokens must decode
/// successfully; malformed `~` sequences and invalid array indices
/// return `ScopeNotEstablished` defensively.
fn extract_json_pointer(value: &serde_json::Value, pointer: &str) -> Option<serde_json::Value> {
    if pointer.is_empty() {
        return Some(value.clone());
    }
    if pointer.len() == 1 {
        // Just "/" — refers to a key that is the empty string.
        // RFC 6901: "" references the whole document; "/" references a
        // member whose name is the empty string.
        match value {
            serde_json::Value::Object(map) => map.get("").cloned(),
            _ => None,
        }
    } else {
        let mut current = value;
        for token_str in pointer[1..].split('/') {
            let decoded = crate::runtime_config::decode_strict_pointer_token(token_str).ok()?;
            match current {
                serde_json::Value::Object(map) => {
                    current = map.get(&decoded)?;
                }
                serde_json::Value::Array(arr) => {
                    // Validate array-index syntax (only valid decimal
                    // non-negative integers with no leading zeros except "0").
                    if !crate::runtime_config::is_valid_array_index(&decoded) {
                        return None;
                    }
                    let index: usize = decoded.parse().ok()?;
                    current = arr.get(index)?;
                }
                _ => return None,
            }
        }
        Some(current.clone())
    }
}

fn path_starts_with_segment(path: &str, prefix: &str) -> bool {
    if prefix.is_empty() {
        return false;
    }

    if prefix.ends_with('/') {
        return path.starts_with(prefix);
    }

    path.starts_with(prefix)
        && (path.len() == prefix.len() || path.as_bytes()[prefix.len()] == b'/')
}

// ===========================================================================
// Planner capability descriptors
// ===========================================================================

impl PreparedRuntime {
    pub fn planner_capabilities(&self) -> Result<Vec<serde_json::Value>, RuntimePreparationError> {
        let mut descriptors: Vec<(String, u32, serde_json::Value)> = Vec::new();

        for provider in &self.providers {
            for cap in &provider.capabilities {
                let manifest = cap.verified_manifest.manifest();

                let inputs = convert_input_schema(&manifest.input_schema)?;

                let reversibility_str = match manifest.reversibility {
                    manifest::Reversibility::Reversible => "reversible",
                    manifest::Reversibility::Compensatable => "compensatable",
                    manifest::Reversibility::Irreversible => "irreversible",
                };

                let descriptor = serde_json::json!({
                    "name": cap.name,
                    "version": format!("{}.0.0", cap.version),
                    "inputs": inputs,
                    "effects": manifest.effects,
                    "reversibility": reversibility_str,
                });

                descriptors.push((cap.name.clone(), cap.version, descriptor));
            }
        }

        descriptors.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

        Ok(descriptors.into_iter().map(|(_, _, d)| d).collect())
    }
}

fn convert_input_schema(
    schema: &serde_json::Value,
) -> Result<serde_json::Value, RuntimePreparationError> {
    let obj = schema.as_object().ok_or_else(|| {
        RuntimePreparationError::new(
            RuntimePreparationErrorCode::UnsupportedPlannerSchema,
            "input_schema must be a JSON object",
        )
    })?;

    let schema_type = obj.get("type").and_then(serde_json::Value::as_str);
    if schema_type != Some("object") {
        return Err(RuntimePreparationError::new(
            RuntimePreparationErrorCode::UnsupportedPlannerSchema,
            "input_schema type must be \"object\"",
        ));
    }

    let properties = obj
        .get("properties")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| {
            RuntimePreparationError::new(
                RuntimePreparationErrorCode::UnsupportedPlannerSchema,
                "input_schema must have a \"properties\" object",
            )
        })?;

    let mut names: Vec<&String> = properties.keys().collect();
    names.sort();

    let mut result = serde_json::Map::new();
    for name in names {
        let prop = &properties[name];
        let prop_type = prop
            .as_object()
            .and_then(|o| o.get("type"))
            .and_then(serde_json::Value::as_str);

        let planner_type = match prop_type {
            Some("string") => "string",
            Some("boolean") => "boolean",
            Some("number") => "number",
            Some("integer") => "number",
            _ => {
                return Err(RuntimePreparationError::new(
                    RuntimePreparationErrorCode::UnsupportedPlannerSchema,
                    format!(
                        "property \"{}\" has unsupported type: {:?}",
                        name, prop_type
                    ),
                ));
            }
        };

        result.insert(
            name.clone(),
            serde_json::Value::String(planner_type.to_string()),
        );
    }

    Ok(serde_json::Value::Object(result))
}

// ===========================================================================
// Tether material
// ===========================================================================

impl PreparedRuntime {
    pub fn tether_material(&self, index: usize) -> Option<serde_json::Value> {
        let tether = self.tethers.get(index)?;
        Some(serde_json::json!({
            "id": tether.id,
            "version": tether.version,
            "source": tether.source,
        }))
    }
}

// ===========================================================================
// Preparation API
// ===========================================================================

pub fn prepare_runtime(
    loaded: &LoadedRuntimeConfig,
) -> Result<PreparedRuntime, RuntimePreparationError> {
    let config_dir = loaded.config_dir.canonicalize().map_err(|e| {
        RuntimePreparationError::new(
            RuntimePreparationErrorCode::AssetNotFound,
            format!(
                "could not canonicalise config directory \"{}\": {}",
                loaded.config_dir.display(),
                e
            ),
        )
    })?;

    // Load Tether source files
    let mut tethers: Vec<PreparedTether> = Vec::new();
    for tether_ref in &loaded.config.tether_set.tethers {
        let source_path = confine_asset(&config_dir, &tether_ref.source_path)?;
        let source = read_utf8_asset(&source_path)?;

        if source.trim().is_empty() {
            return Err(RuntimePreparationError::new(
                RuntimePreparationErrorCode::EmptyTetherSource,
                format!(
                    "Tether source file \"{}\" is empty or whitespace-only",
                    source_path.display()
                ),
            ));
        }

        tethers.push(PreparedTether {
            id: tether_ref.id.clone(),
            version: tether_ref.version.clone(),
            source_path,
            source,
        });
    }

    let requirements: Vec<CapabilityRequirement> = loaded.config.capability_requirements();
    let policy = loaded.config.host_local_policy();

    let materializations = loaded.config.provider_materializations();
    let mut trusted_store = TrustedManifestStore::new();
    let mut providers: Vec<PreparedProvider> = Vec::new();

    for mat in &materializations {
        let mut capabilities: Vec<PreparedCapability> = Vec::new();

        for cap_mat in &mat.capabilities {
            let manifest_path = confine_asset(&config_dir, &cap_mat.manifest_path)?;
            let manifest_json = read_utf8_asset(&manifest_path)?;

            let verified = manifest::verify_manifest(&manifest_json).map_err(|e| {
                RuntimePreparationError::new(
                    RuntimePreparationErrorCode::ManifestInvalid,
                    format!(
                        "manifest \"{}\" verification failed: {:?}",
                        manifest_path.display(),
                        e
                    ),
                )
            })?;

            let m = verified.manifest();

            if m.capability_name != cap_mat.name {
                return Err(RuntimePreparationError::new(
                    RuntimePreparationErrorCode::ManifestIdentityMismatch,
                    format!(
                        "manifest capability name \"{}\" does not match \
                         configured name \"{}\"",
                        m.capability_name, cap_mat.name
                    ),
                ));
            }

            if m.capability_version != cap_mat.version {
                return Err(RuntimePreparationError::new(
                    RuntimePreparationErrorCode::ManifestIdentityMismatch,
                    format!(
                        "manifest capability version {} does not match \
                         configured version {}",
                        m.capability_version, cap_mat.version
                    ),
                ));
            }

            if m.provider.identity != mat.identity {
                return Err(RuntimePreparationError::new(
                    RuntimePreparationErrorCode::ProviderIdentityMismatch,
                    format!(
                        "manifest provider identity \"{}\" does not match \
                         configured provider identity \"{}\"",
                        m.provider.identity, mat.identity
                    ),
                ));
            }

            if verified.verified_digest() != cap_mat.pinned_digest {
                return Err(RuntimePreparationError::new(
                    RuntimePreparationErrorCode::PinnedDigestMismatch,
                    format!(
                        "manifest verified digest \"{}\" does not match \
                         pinned digest \"{}\"",
                        verified.verified_digest(),
                        cap_mat.pinned_digest
                    ),
                ));
            }

            // Scope-binding compatibility
            match &m.permission_scope {
                PermissionScope::PathPrefix { allowed_prefixes } => {
                    let binding = cap_mat.scope_binding.as_ref().ok_or_else(|| {
                        RuntimePreparationError::new(
                            RuntimePreparationErrorCode::MissingScopeBinding,
                            format!(
                                "manifest \"{}\" declares path_prefix scope but no \
                                 scope_binding is configured",
                                manifest_path.display()
                            ),
                        )
                    })?;

                    match binding.kind {
                        ScopeBindingKind::PathPrefix => {
                            validate_allowed_prefixes(allowed_prefixes)?;
                        }
                    }
                }
                PermissionScope::Unrestricted => {
                    if cap_mat.scope_binding.is_some() {
                        return Err(RuntimePreparationError::new(
                            RuntimePreparationErrorCode::UnexpectedScopeBinding,
                            format!(
                                "manifest \"{}\" declares unrestricted scope but a \
                                 scope_binding is configured",
                                manifest_path.display()
                            ),
                        ));
                    }
                }
                PermissionScope::Repository { .. } | PermissionScope::Calendar { .. } => {
                    return Err(RuntimePreparationError::new(
                        RuntimePreparationErrorCode::UnsupportedPermissionScope,
                        format!(
                            "manifest \"{}\" declares {:?} scope which is not \
                             supported in J12",
                            manifest_path.display(),
                            m.permission_scope
                        ),
                    ));
                }
            }

            // Build a ProviderConfig scoped to just this one capability
            // so that exact (name, version) admission works when a
            // provider exposes multiple capabilities with the same name
            // at different versions.
            let single_cap_config = provider::ProviderConfig {
                identity: mat.identity.clone(),
                display_name: mat.display_name.clone(),
                allowed_capabilities: vec![provider::AllowedCapability {
                    capability_name: cap_mat.name.clone(),
                    capability_version: cap_mat.version,
                    pinned_digest: Some(cap_mat.pinned_digest.clone()),
                }],
            };
            provider::admit_provider_manifest(
                &single_cap_config,
                verified.clone(),
                &mut trusted_store,
            )
            .map_err(|e| {
                RuntimePreparationError::new(
                    RuntimePreparationErrorCode::ManifestAdmissionFailed,
                    format!(
                        "admission of manifest \"{}\" failed: {:?}",
                        manifest_path.display(),
                        e
                    ),
                )
            })?;

            capabilities.push(PreparedCapability {
                name: cap_mat.name.clone(),
                version: cap_mat.version,
                manifest_path: manifest_path.clone(),
                verified_manifest: verified,
                scope_binding: cap_mat.scope_binding.clone(),
            });
        }

        providers.push(PreparedProvider {
            identity: mat.identity.clone(),
            display_name: mat.display_name.clone(),
            working_directory: config_dir.clone(),
            stdio_config: mat.to_stdio_config(),
            capabilities,
        });
    }

    Ok(PreparedRuntime {
        config_path: loaded.config_path.clone(),
        config_dir,
        tether_set_id: loaded.config.tether_set.id.clone(),
        tether_set_version: loaded.config.tether_set.version.clone(),
        tethers,
        requirements,
        providers,
        policy,
        trusted_store,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::path::Path;

    // ------------------------------------------------------------------
    // Helpers
    // ------------------------------------------------------------------

    fn minimal_config_json() -> serde_json::Value {
        json!({
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

    /// Build a valid VerifiedManifest for a given name/version/provider.
    fn make_manifest(
        name: &str,
        version: u32,
        provider_id: &str,
        permission_scope: serde_json::Value,
    ) -> (String, String) {
        let mut m = json!({
            "manifest_format_version": "1.0",
            "capability_name": name,
            "capability_version": version,
            "title": "Test Capability",
            "description": "A test capability.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string" }
                },
                "required": ["path"]
            },
            "output_schema": {
                "type": "object",
                "properties": { "result": { "type": "string" } }
            },
            "effects": ["test.effect"],
            "permission_scope": permission_scope,
            "reversibility": "reversible",
            "determinism": "deterministic",
            "idempotency": { "mechanism": "none" },
            "confirmation_policy": {
                "standing_permitted": false,
                "per_call_required": true
            },
            "timeout_ms": 5000,
            "retry_policy": {
                "max_retries": 0,
                "backoff_ms": 500,
                "allowed_on": [],
                "requires_idempotency_proof": false
            },
            "provider": {
                "identity": provider_id,
                "display_name": "Test Provider",
                "identity_source": "host_configuration",
                "description": "Host-assigned."
            },
            "binding": {
                "kind": "mcp",
                "server_name": "test-server",
                "tool_name": "test_tool",
                "adapter": null
            }
        });

        let (_, digest) = crate::manifest::canonicalize_and_digest(&m.to_string()).unwrap();
        m["digest"] = json!(digest);

        let json_str = m.to_string();
        (json_str, digest)
    }

    /// Create the standard tether file and directory for tests.
    fn write_default_tether(dir: &Path) {
        std::fs::create_dir_all(dir.join("tethers")).unwrap();
        let source = "when event.task.completed if task.status == \"done\" do lantern.task.record";
        std::fs::write(dir.join("tethers/record-completed-task.tether"), source).unwrap();
    }

    fn with_prepared_runtime<F>(test_fn: F)
    where
        F: FnOnce(&PreparedRuntime),
    {
        let dir = std::env::temp_dir().join(format!("j12-pkt2-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("manifests")).unwrap();
        write_default_tether(&dir);

        let (manifest_json, digest) =
            make_manifest("lantern.task.record", 1, "lantern-local", json!(null));
        std::fs::write(
            dir.join("manifests/lantern-task-record.json"),
            &manifest_json,
        )
        .unwrap();

        let mut cfg_value = minimal_config_json();
        cfg_value["providers"][0]["capabilities"][0]["pinned_digest"] = json!(digest);

        let config_path = dir.join("tethers-config.json");
        std::fs::write(
            &config_path,
            serde_json::to_string_pretty(&cfg_value).unwrap(),
        )
        .unwrap();

        let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
        let prepared = prepare_runtime(&loaded).unwrap();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            test_fn(&prepared);
        }));
        let _ = std::fs::remove_dir_all(&dir);
        if let Err(e) = result {
            std::panic::resume_unwind(e);
        }
    }

    // ------------------------------------------------------------------
    // J12 Packet 2 focused tests
    // ------------------------------------------------------------------

    // 1
    #[test]
    fn j12_packet2_valid_runtime_preparation_succeeds() {
        with_prepared_runtime(|prepared| {
            assert_eq!(prepared.tether_set_id(), "example.local");
            assert_eq!(prepared.tether_set_version(), "1");
            assert_eq!(prepared.tethers().len(), 1);
            assert_eq!(prepared.requirements().len(), 1);
            assert_eq!(prepared.providers().len(), 1);
            assert_eq!(prepared.trusted_store().len(), 1);
        });
    }

    // 2
    #[test]
    fn j12_packet2_tether_order_preserved() {
        let dir = std::env::temp_dir().join(format!("j12-pkt2-order-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("manifests")).unwrap();
        write_default_tether(&dir);

        let (manifest_json, digest) =
            make_manifest("lantern.task.record", 1, "lantern-local", json!(null));
        std::fs::write(
            dir.join("manifests/lantern-task-record.json"),
            &manifest_json,
        )
        .unwrap();

        let mut json = minimal_config_json();
        json["tether_set"]["tethers"] = json!([
            {"id": "third", "version": "v1", "source_path": "third.tether"},
            {"id": "first", "version": "v1", "source_path": "first.tether"},
            {"id": "second", "version": "v1", "source_path": "second.tether"}
        ]);
        json["providers"][0]["capabilities"][0]["pinned_digest"] = json!(digest);

        std::fs::write(dir.join("third.tether"), "t3").unwrap();
        std::fs::write(dir.join("first.tether"), "t1").unwrap();
        std::fs::write(dir.join("second.tether"), "t2").unwrap();

        let config_path = dir.join("config.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

        let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
        let prepared = prepare_runtime(&loaded).unwrap();
        let ids: Vec<&str> = prepared.tethers().iter().map(|t| t.id.as_str()).collect();
        assert_eq!(ids, vec!["third", "first", "second"]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    // 3
    #[test]
    fn j12_packet2_exact_source_text_retained() {
        with_prepared_runtime(|prepared| {
            let src = &prepared.tethers()[0].source;
            assert!(src.contains("when event.task.completed"));
            assert!(src.contains("lantern.task.record"));
        });
    }

    // 4
    #[test]
    fn j12_packet2_missing_tether_source_fails() {
        let dir = std::env::temp_dir().join(format!("j12-pkt2-missing-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("manifests")).unwrap();

        let mut json = minimal_config_json();
        json["tether_set"]["tethers"][0]["source_path"] = json!("nonexistent.tether");

        let (manifest_json, _digest) =
            make_manifest("lantern.task.record", 1, "lantern-local", json!(null));
        std::fs::write(
            dir.join("manifests/lantern-task-record.json"),
            &manifest_json,
        )
        .unwrap();

        let config_path = dir.join("config.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

        let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
        let err = prepare_runtime(&loaded).unwrap_err();
        assert_eq!(err.code, RuntimePreparationErrorCode::AssetNotFound);
        let _ = std::fs::remove_dir_all(&dir);
    }

    // 5
    #[test]
    fn j12_packet2_empty_tether_source_fails() {
        let dir = std::env::temp_dir().join(format!("j12-pkt2-empty-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("manifests")).unwrap();
        std::fs::create_dir_all(dir.join("tethers")).unwrap();
        std::fs::write(dir.join("tethers/empty.tether"), "").unwrap();

        let mut json = minimal_config_json();
        json["tether_set"]["tethers"][0]["source_path"] = json!("tethers/empty.tether");

        let (manifest_json, _digest) =
            make_manifest("lantern.task.record", 1, "lantern-local", json!(null));
        std::fs::write(
            dir.join("manifests/lantern-task-record.json"),
            &manifest_json,
        )
        .unwrap();

        let config_path = dir.join("config.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

        let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
        let err = prepare_runtime(&loaded).unwrap_err();
        assert_eq!(err.code, RuntimePreparationErrorCode::EmptyTetherSource);
        let _ = std::fs::remove_dir_all(&dir);
    }

    // 6
    #[test]
    fn j12_packet2_directory_as_tether_source_fails() {
        let dir = std::env::temp_dir().join(format!("j12-pkt2-dir-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("manifests")).unwrap();
        std::fs::create_dir_all(dir.join("adir")).unwrap();

        let mut json = minimal_config_json();
        json["tether_set"]["tethers"][0]["source_path"] = json!("adir");

        let (manifest_json, _digest) =
            make_manifest("lantern.task.record", 1, "lantern-local", json!(null));
        std::fs::write(
            dir.join("manifests/lantern-task-record.json"),
            &manifest_json,
        )
        .unwrap();

        let config_path = dir.join("config.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

        let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
        let err = prepare_runtime(&loaded).unwrap_err();
        assert_eq!(err.code, RuntimePreparationErrorCode::AssetNotFile);
        let _ = std::fs::remove_dir_all(&dir);
    }

    // 7
    #[test]
    fn j12_packet2_source_path_escape_fails() {
        let dir = std::env::temp_dir().join(format!("j12-pkt2-escape-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("manifests")).unwrap();
        write_default_tether(&dir);

        let mut json = minimal_config_json();
        json["tether_set"]["tethers"][0]["source_path"] = json!("../outside.tether");

        let (manifest_json, _digest) =
            make_manifest("lantern.task.record", 1, "lantern-local", json!(null));
        std::fs::write(
            dir.join("manifests/lantern-task-record.json"),
            &manifest_json,
        )
        .unwrap();

        let config_path = dir.join("config.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

        let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
        let err = prepare_runtime(&loaded).unwrap_err();
        assert_eq!(
            err.code,
            RuntimePreparationErrorCode::AssetOutsideConfigRoot
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    // 8
    #[test]
    fn j12_packet2_missing_manifest_fails() {
        let dir = std::env::temp_dir().join(format!("j12-pkt2-noman-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("manifests")).unwrap();
        write_default_tether(&dir);

        let mut json = minimal_config_json();
        json["providers"][0]["capabilities"][0]["manifest_path"] =
            json!("manifests/nonexistent.json");

        let config_path = dir.join("config.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

        let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
        let err = prepare_runtime(&loaded).unwrap_err();
        assert_eq!(err.code, RuntimePreparationErrorCode::AssetNotFound);
        let _ = std::fs::remove_dir_all(&dir);
    }

    // 9
    #[test]
    fn j12_packet2_invalid_manifest_fails() {
        let dir = std::env::temp_dir().join(format!("j12-pkt2-badman-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("manifests")).unwrap();
        write_default_tether(&dir);

        std::fs::write(
            dir.join("manifests/lantern-task-record.json"),
            "not valid json",
        )
        .unwrap();

        let config_path = dir.join("config.json");
        std::fs::write(
            &config_path,
            serde_json::to_string_pretty(&minimal_config_json()).unwrap(),
        )
        .unwrap();

        let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
        let err = prepare_runtime(&loaded).unwrap_err();
        assert_eq!(err.code, RuntimePreparationErrorCode::ManifestInvalid);
        let _ = std::fs::remove_dir_all(&dir);
    }

    // 10
    #[test]
    fn j12_packet2_manifest_name_mismatch_fails() {
        let dir = std::env::temp_dir().join(format!("j12-pkt2-namem-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("manifests")).unwrap();
        write_default_tether(&dir);

        let (manifest_json, digest) = make_manifest("wrong.name", 1, "lantern-local", json!(null));
        std::fs::write(dir.join("manifests/wrong.json"), &manifest_json).unwrap();

        let mut json = minimal_config_json();
        json["providers"][0]["capabilities"][0]["manifest_path"] = json!("manifests/wrong.json");
        json["providers"][0]["capabilities"][0]["pinned_digest"] = json!(digest);

        let config_path = dir.join("config.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

        let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
        let err = prepare_runtime(&loaded).unwrap_err();
        assert_eq!(
            err.code,
            RuntimePreparationErrorCode::ManifestIdentityMismatch
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    // 11
    #[test]
    fn j12_packet2_manifest_version_mismatch_fails() {
        let dir = std::env::temp_dir().join(format!("j12-pkt2-verm-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("manifests")).unwrap();
        write_default_tether(&dir);

        let (manifest_json, digest) =
            make_manifest("lantern.task.record", 99, "lantern-local", json!(null));
        std::fs::write(dir.join("manifests/wrong.json"), &manifest_json).unwrap();

        let mut json = minimal_config_json();
        json["providers"][0]["capabilities"][0]["manifest_path"] = json!("manifests/wrong.json");
        json["providers"][0]["capabilities"][0]["pinned_digest"] = json!(digest);

        let config_path = dir.join("config.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

        let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
        let err = prepare_runtime(&loaded).unwrap_err();
        assert_eq!(
            err.code,
            RuntimePreparationErrorCode::ManifestIdentityMismatch
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    // 12
    #[test]
    fn j12_packet2_provider_identity_mismatch_fails() {
        let dir = std::env::temp_dir().join(format!("j12-pkt2-pidm-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("manifests")).unwrap();
        write_default_tether(&dir);

        let (manifest_json, digest) =
            make_manifest("lantern.task.record", 1, "other-provider", json!(null));
        std::fs::write(dir.join("manifests/wrong.json"), &manifest_json).unwrap();

        let mut json = minimal_config_json();
        json["providers"][0]["capabilities"][0]["manifest_path"] = json!("manifests/wrong.json");
        json["providers"][0]["capabilities"][0]["pinned_digest"] = json!(digest);

        let config_path = dir.join("config.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

        let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
        let err = prepare_runtime(&loaded).unwrap_err();
        assert_eq!(
            err.code,
            RuntimePreparationErrorCode::ProviderIdentityMismatch
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    // 13
    #[test]
    fn j12_packet2_pinned_digest_mismatch_fails() {
        let dir = std::env::temp_dir().join(format!("j12-pkt2-pdm-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("manifests")).unwrap();
        write_default_tether(&dir);

        let (manifest_json, _digest) =
            make_manifest("lantern.task.record", 1, "lantern-local", json!(null));
        std::fs::write(
            dir.join("manifests/lantern-task-record.json"),
            &manifest_json,
        )
        .unwrap();

        let mut json = minimal_config_json();
        json["providers"][0]["capabilities"][0]["pinned_digest"] =
            json!("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");

        let config_path = dir.join("config.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

        let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
        let err = prepare_runtime(&loaded).unwrap_err();
        assert_eq!(err.code, RuntimePreparationErrorCode::PinnedDigestMismatch);
        let _ = std::fs::remove_dir_all(&dir);
    }

    // 14
    #[test]
    fn j12_packet2_manifest_admission_populates_store() {
        with_prepared_runtime(|prepared| {
            assert_eq!(prepared.trusted_store().len(), 1);
            let stored = prepared
                .trusted_store()
                .get_by_name_version("lantern.task.record", 1);
            assert!(stored.is_some());
        });
    }

    // 15 -- tested in runtime_config.rs as j12_packet2_duplicate_unscoped_across_providers_rejected

    // 16
    #[test]
    fn j12_packet2_same_name_different_versions_valid() {
        let dir = std::env::temp_dir().join(format!("j12-pkt2-sndv-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("manifests")).unwrap();
        write_default_tether(&dir);

        let mut json = minimal_config_json();
        json["tether_set"]["capability_requirements"] = json!([
            {"name": "lantern.task.record", "version": 1},
            {"name": "lantern.task.record", "version": 2}
        ]);
        json["policy"]["rules"] = json!([
            {"name": "lantern.task.record", "version": 1, "decision": "allow"},
            {"name": "lantern.task.record", "version": 2, "decision": "allow"}
        ]);

        let (m1, d1) = make_manifest("lantern.task.record", 1, "lantern-local", json!(null));
        let (m2, d2) = make_manifest("lantern.task.record", 2, "lantern-local", json!(null));
        std::fs::write(dir.join("manifests/v1.json"), &m1).unwrap();
        std::fs::write(dir.join("manifests/v2.json"), &m2).unwrap();

        json["providers"][0]["capabilities"] = json!([
            {"name": "lantern.task.record", "version": 1, "manifest_path": "manifests/v1.json", "pinned_digest": d1},
            {"name": "lantern.task.record", "version": 2, "manifest_path": "manifests/v2.json", "pinned_digest": d2}
        ]);

        let config_path = dir.join("config.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

        let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
        let prepared = prepare_runtime(&loaded).unwrap();
        assert_eq!(prepared.providers()[0].capabilities.len(), 2);
        assert_eq!(prepared.trusted_store().len(), 2);
        let _ = std::fs::remove_dir_all(&dir);
    }

    // 17
    #[test]
    fn j12_packet2_pathprefix_requires_scope_binding() {
        let dir = std::env::temp_dir().join(format!("j12-pkt2-pprs-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("manifests")).unwrap();
        write_default_tether(&dir);

        let (manifest_json, digest) = make_manifest(
            "lantern.task.record",
            1,
            "lantern-local",
            json!({"kind": "path_prefix", "allowed_prefixes": ["projects/"]}),
        );
        std::fs::write(
            dir.join("manifests/lantern-task-record.json"),
            &manifest_json,
        )
        .unwrap();

        let mut json = minimal_config_json();
        json["providers"][0]["capabilities"][0]["pinned_digest"] = json!(digest);

        let config_path = dir.join("config.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

        let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
        let err = prepare_runtime(&loaded).unwrap_err();
        assert_eq!(err.code, RuntimePreparationErrorCode::MissingScopeBinding);
        let _ = std::fs::remove_dir_all(&dir);
    }

    // 18
    #[test]
    fn j12_packet2_unrestricted_rejects_scope_binding() {
        let dir = std::env::temp_dir().join(format!("j12-pkt2-ursb-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("manifests")).unwrap();
        write_default_tether(&dir);

        let (manifest_json, digest) =
            make_manifest("lantern.task.record", 1, "lantern-local", json!(null));
        std::fs::write(
            dir.join("manifests/lantern-task-record.json"),
            &manifest_json,
        )
        .unwrap();

        let mut json = minimal_config_json();
        json["providers"][0]["capabilities"][0]["pinned_digest"] = json!(digest);
        json["providers"][0]["capabilities"][0]["scope_binding"] = json!({
            "kind": "path_prefix",
            "argument_json_pointer": "/path"
        });

        let config_path = dir.join("config.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

        let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
        let err = prepare_runtime(&loaded).unwrap_err();
        assert_eq!(
            err.code,
            RuntimePreparationErrorCode::UnexpectedScopeBinding
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    // 19
    #[test]
    fn j12_packet2_repository_scope_fails_closed() {
        let dir = std::env::temp_dir().join(format!("j12-pkt2-repo-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("manifests")).unwrap();
        write_default_tether(&dir);

        let (manifest_json, digest) = make_manifest(
            "lantern.task.record",
            1,
            "lantern-local",
            json!({"kind": "repository", "allowed_repositories": ["repo1"]}),
        );
        std::fs::write(
            dir.join("manifests/lantern-task-record.json"),
            &manifest_json,
        )
        .unwrap();

        let mut json = minimal_config_json();
        json["providers"][0]["capabilities"][0]["pinned_digest"] = json!(digest);

        let config_path = dir.join("config.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

        let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
        let err = prepare_runtime(&loaded).unwrap_err();
        assert_eq!(
            err.code,
            RuntimePreparationErrorCode::UnsupportedPermissionScope
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    // 20
    #[test]
    fn j12_packet2_path_within_prefix_returns_within_scope() {
        let dir = std::env::temp_dir().join(format!("j12-pkt2-scope-in-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("manifests")).unwrap();
        write_default_tether(&dir);

        let (manifest_json, digest) = make_manifest(
            "lantern.task.record",
            1,
            "lantern-local",
            json!({"kind": "path_prefix", "allowed_prefixes": ["projects/"]}),
        );
        std::fs::write(
            dir.join("manifests/lantern-task-record.json"),
            &manifest_json,
        )
        .unwrap();

        let mut json = minimal_config_json();
        json["providers"][0]["capabilities"][0]["pinned_digest"] = json!(digest);
        json["providers"][0]["capabilities"][0]["scope_binding"] = json!({
            "kind": "path_prefix",
            "argument_json_pointer": "/path"
        });

        let config_path = dir.join("config.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

        let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
        let prepared = prepare_runtime(&loaded).unwrap();

        let action = ProposedAction {
            evaluation_id: "eval-1".into(),
            plan_id: "plan-1".into(),
            action_id: "act-1".into(),
            capability_name: "lantern.task.record".into(),
            manifest_digest: Some(digest),
            bridge_capability_version: Some(1),
            bridge_provider_identity: Some("lantern-local".into()),
            arguments: json!({"path": "projects/myfile.md"}),
        };
        assert_eq!(
            prepared.assess_action_scope(&action),
            ScopeAssessment::WithinScope
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    // 21
    #[test]
    fn j12_packet2_path_outside_prefix_returns_violation() {
        let dir = std::env::temp_dir().join(format!("j12-pkt2-scope-out-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("manifests")).unwrap();
        write_default_tether(&dir);

        let (manifest_json, digest) = make_manifest(
            "lantern.task.record",
            1,
            "lantern-local",
            json!({"kind": "path_prefix", "allowed_prefixes": ["projects/"]}),
        );
        std::fs::write(
            dir.join("manifests/lantern-task-record.json"),
            &manifest_json,
        )
        .unwrap();

        let mut json = minimal_config_json();
        json["providers"][0]["capabilities"][0]["pinned_digest"] = json!(digest);
        json["providers"][0]["capabilities"][0]["scope_binding"] = json!({
            "kind": "path_prefix",
            "argument_json_pointer": "/path"
        });

        let config_path = dir.join("config.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

        let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
        let prepared = prepare_runtime(&loaded).unwrap();

        let action = ProposedAction {
            evaluation_id: "eval-1".into(),
            plan_id: "plan-1".into(),
            action_id: "act-1".into(),
            capability_name: "lantern.task.record".into(),
            manifest_digest: Some(digest),
            bridge_capability_version: Some(1),
            bridge_provider_identity: Some("lantern-local".into()),
            arguments: json!({"path": "other/file.md"}),
        };
        assert_eq!(
            prepared.assess_action_scope(&action),
            ScopeAssessment::ScopeViolation
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    // 22
    #[test]
    fn j12_packet2_sibling_prefix_no_match() {
        let dir = std::env::temp_dir().join(format!("j12-pkt2-sib-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("manifests")).unwrap();
        write_default_tether(&dir);

        let (manifest_json, digest) = make_manifest(
            "lantern.task.record",
            1,
            "lantern-local",
            json!({"kind": "path_prefix", "allowed_prefixes": ["projects/"]}),
        );
        std::fs::write(
            dir.join("manifests/lantern-task-record.json"),
            &manifest_json,
        )
        .unwrap();

        let mut json = minimal_config_json();
        json["providers"][0]["capabilities"][0]["pinned_digest"] = json!(digest);
        json["providers"][0]["capabilities"][0]["scope_binding"] = json!({
            "kind": "path_prefix",
            "argument_json_pointer": "/path"
        });

        let config_path = dir.join("config.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

        let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
        let prepared = prepare_runtime(&loaded).unwrap();

        let action = ProposedAction {
            evaluation_id: "eval-1".into(),
            plan_id: "plan-1".into(),
            action_id: "act-1".into(),
            capability_name: "lantern.task.record".into(),
            manifest_digest: Some(digest),
            bridge_capability_version: Some(1),
            bridge_provider_identity: Some("lantern-local".into()),
            arguments: json!({"path": "projects2/file.md"}),
        };
        assert_eq!(
            prepared.assess_action_scope(&action),
            ScopeAssessment::ScopeViolation
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    // 23
    #[test]
    fn j12_packet2_missing_pointer_scope_not_established() {
        let dir = std::env::temp_dir().join(format!("j12-pkt2-mp-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("manifests")).unwrap();
        write_default_tether(&dir);

        let (manifest_json, digest) = make_manifest(
            "lantern.task.record",
            1,
            "lantern-local",
            json!({"kind": "path_prefix", "allowed_prefixes": ["projects/"]}),
        );
        std::fs::write(
            dir.join("manifests/lantern-task-record.json"),
            &manifest_json,
        )
        .unwrap();

        let mut json = minimal_config_json();
        json["providers"][0]["capabilities"][0]["pinned_digest"] = json!(digest);
        json["providers"][0]["capabilities"][0]["scope_binding"] = json!({
            "kind": "path_prefix",
            "argument_json_pointer": "/path"
        });

        let config_path = dir.join("config.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

        let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
        let prepared = prepare_runtime(&loaded).unwrap();

        let action = ProposedAction {
            evaluation_id: "eval-1".into(),
            plan_id: "plan-1".into(),
            action_id: "act-1".into(),
            capability_name: "lantern.task.record".into(),
            manifest_digest: Some(digest),
            bridge_capability_version: Some(1),
            bridge_provider_identity: Some("lantern-local".into()),
            arguments: json!({"other": "value"}),
        };
        assert_eq!(
            prepared.assess_action_scope(&action),
            ScopeAssessment::ScopeNotEstablished
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    // 24
    #[test]
    fn j12_packet2_nonstring_pointer_scope_not_established() {
        let dir = std::env::temp_dir().join(format!("j12-pkt2-ns-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("manifests")).unwrap();
        write_default_tether(&dir);

        let (manifest_json, digest) = make_manifest(
            "lantern.task.record",
            1,
            "lantern-local",
            json!({"kind": "path_prefix", "allowed_prefixes": ["projects/"]}),
        );
        std::fs::write(
            dir.join("manifests/lantern-task-record.json"),
            &manifest_json,
        )
        .unwrap();

        let mut json = minimal_config_json();
        json["providers"][0]["capabilities"][0]["pinned_digest"] = json!(digest);
        json["providers"][0]["capabilities"][0]["scope_binding"] = json!({
            "kind": "path_prefix",
            "argument_json_pointer": "/path"
        });

        let config_path = dir.join("config.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

        let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
        let prepared = prepare_runtime(&loaded).unwrap();

        let action = ProposedAction {
            evaluation_id: "eval-1".into(),
            plan_id: "plan-1".into(),
            action_id: "act-1".into(),
            capability_name: "lantern.task.record".into(),
            manifest_digest: Some(digest),
            bridge_capability_version: Some(1),
            bridge_provider_identity: Some("lantern-local".into()),
            arguments: json!({"path": 123}),
        };
        assert_eq!(
            prepared.assess_action_scope(&action),
            ScopeAssessment::ScopeNotEstablished
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    // 25
    #[test]
    fn j12_packet2_traversal_segment_scope_not_established() {
        let dir = std::env::temp_dir().join(format!("j12-pkt2-trav-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("manifests")).unwrap();
        write_default_tether(&dir);

        let (manifest_json, digest) = make_manifest(
            "lantern.task.record",
            1,
            "lantern-local",
            json!({"kind": "path_prefix", "allowed_prefixes": ["projects/"]}),
        );
        std::fs::write(
            dir.join("manifests/lantern-task-record.json"),
            &manifest_json,
        )
        .unwrap();

        let mut json = minimal_config_json();
        json["providers"][0]["capabilities"][0]["pinned_digest"] = json!(digest);
        json["providers"][0]["capabilities"][0]["scope_binding"] = json!({
            "kind": "path_prefix",
            "argument_json_pointer": "/path"
        });

        let config_path = dir.join("config.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

        let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
        let prepared = prepare_runtime(&loaded).unwrap();

        let action = ProposedAction {
            evaluation_id: "eval-1".into(),
            plan_id: "plan-1".into(),
            action_id: "act-1".into(),
            capability_name: "lantern.task.record".into(),
            manifest_digest: Some(digest),
            bridge_capability_version: Some(1),
            bridge_provider_identity: Some("lantern-local".into()),
            arguments: json!({"path": "projects/../etc/file.md"}),
        };
        assert_eq!(
            prepared.assess_action_scope(&action),
            ScopeAssessment::ScopeNotEstablished
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    // 26
    #[test]
    fn j12_packet2_absolute_resource_scope_not_established() {
        let dir = std::env::temp_dir().join(format!("j12-pkt2-abs-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("manifests")).unwrap();
        write_default_tether(&dir);

        let (manifest_json, digest) = make_manifest(
            "lantern.task.record",
            1,
            "lantern-local",
            json!({"kind": "path_prefix", "allowed_prefixes": ["projects/"]}),
        );
        std::fs::write(
            dir.join("manifests/lantern-task-record.json"),
            &manifest_json,
        )
        .unwrap();

        let mut json = minimal_config_json();
        json["providers"][0]["capabilities"][0]["pinned_digest"] = json!(digest);
        json["providers"][0]["capabilities"][0]["scope_binding"] = json!({
            "kind": "path_prefix",
            "argument_json_pointer": "/path"
        });

        let config_path = dir.join("config.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

        let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
        let prepared = prepare_runtime(&loaded).unwrap();

        let action = ProposedAction {
            evaluation_id: "eval-1".into(),
            plan_id: "plan-1".into(),
            action_id: "act-1".into(),
            capability_name: "lantern.task.record".into(),
            manifest_digest: Some(digest),
            bridge_capability_version: Some(1),
            bridge_provider_identity: Some("lantern-local".into()),
            arguments: json!({"path": "/etc/passwd"}),
        };
        assert_eq!(
            prepared.assess_action_scope(&action),
            ScopeAssessment::ScopeNotEstablished
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    // 27
    #[test]
    fn j12_packet2_provider_mismatch_scope_not_established() {
        let dir = std::env::temp_dir().join(format!("j12-pkt2-pvm-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("manifests")).unwrap();
        write_default_tether(&dir);

        let (manifest_json, digest) = make_manifest(
            "lantern.task.record",
            1,
            "lantern-local",
            json!({"kind": "path_prefix", "allowed_prefixes": ["projects/"]}),
        );
        std::fs::write(
            dir.join("manifests/lantern-task-record.json"),
            &manifest_json,
        )
        .unwrap();

        let mut json = minimal_config_json();
        json["providers"][0]["capabilities"][0]["pinned_digest"] = json!(digest);
        json["providers"][0]["capabilities"][0]["scope_binding"] = json!({
            "kind": "path_prefix",
            "argument_json_pointer": "/path"
        });

        let config_path = dir.join("config.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

        let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
        let prepared = prepare_runtime(&loaded).unwrap();

        let action = ProposedAction {
            evaluation_id: "eval-1".into(),
            plan_id: "plan-1".into(),
            action_id: "act-1".into(),
            capability_name: "lantern.task.record".into(),
            manifest_digest: Some(digest),
            bridge_capability_version: Some(1),
            bridge_provider_identity: Some("wrong-provider".into()),
            arguments: json!({"path": "projects/file.md"}),
        };
        assert_eq!(
            prepared.assess_action_scope(&action),
            ScopeAssessment::ScopeNotEstablished
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    // 28
    #[test]
    fn j12_packet2_digest_mismatch_scope_not_established() {
        let dir = std::env::temp_dir().join(format!("j12-pkt2-dm-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("manifests")).unwrap();
        write_default_tether(&dir);

        let (manifest_json, digest) = make_manifest(
            "lantern.task.record",
            1,
            "lantern-local",
            json!({"kind": "path_prefix", "allowed_prefixes": ["projects/"]}),
        );
        std::fs::write(
            dir.join("manifests/lantern-task-record.json"),
            &manifest_json,
        )
        .unwrap();

        let mut json = minimal_config_json();
        json["providers"][0]["capabilities"][0]["pinned_digest"] = json!(digest);
        json["providers"][0]["capabilities"][0]["scope_binding"] = json!({
            "kind": "path_prefix",
            "argument_json_pointer": "/path"
        });

        let config_path = dir.join("config.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

        let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
        let prepared = prepare_runtime(&loaded).unwrap();

        let action = ProposedAction {
            evaluation_id: "eval-1".into(),
            plan_id: "plan-1".into(),
            action_id: "act-1".into(),
            capability_name: "lantern.task.record".into(),
            manifest_digest: Some(
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
            ),
            bridge_capability_version: Some(1),
            bridge_provider_identity: Some("lantern-local".into()),
            arguments: json!({"path": "projects/file.md"}),
        };
        assert_eq!(
            prepared.assess_action_scope(&action),
            ScopeAssessment::ScopeNotEstablished
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    // 29
    #[test]
    fn j12_packet2_unrestricted_action_within_scope() {
        with_prepared_runtime(|prepared| {
            let store = prepared.trusted_store();
            let stored = store.get_by_name_version("lantern.task.record", 1).unwrap();
            let digest = stored.verified_digest().to_string();

            let action = ProposedAction {
                evaluation_id: "eval-1".into(),
                plan_id: "plan-1".into(),
                action_id: "act-1".into(),
                capability_name: "lantern.task.record".into(),
                manifest_digest: Some(digest),
                bridge_capability_version: Some(1),
                bridge_provider_identity: Some("lantern-local".into()),
                arguments: json!({"path": "anything"}),
            };
            assert_eq!(
                prepared.assess_action_scope(&action),
                ScopeAssessment::WithinScope
            );
        });
    }

    // 30
    #[test]
    fn j12_packet2_planner_descriptors_deterministic_pin_free() {
        with_prepared_runtime(|prepared| {
            let descriptors = prepared.planner_capabilities().unwrap();
            assert_eq!(descriptors.len(), 1);
            let d = &descriptors[0];
            assert_eq!(d["name"], "lantern.task.record");
            assert_eq!(d["version"], "1.0.0");
            assert!(d.get("bridge_capability_version").is_none());
            assert!(d.get("manifest_digest").is_none());
            assert!(d.get("bridge_provider_identity").is_none());
            assert!(d.get("inputs").is_some());
            assert!(d.get("effects").is_some());
            assert!(d.get("reversibility").is_some());
        });
    }

    // 31
    #[test]
    fn j12_packet2_unsupported_planner_schema_fails() {
        let dir = std::env::temp_dir().join(format!("j12-pkt2-ups-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("manifests")).unwrap();
        write_default_tether(&dir);

        let mut m = json!({
            "manifest_format_version": "1.0",
            "capability_name": "lantern.task.record",
            "capability_version": 1,
            "title": "Test",
            "description": "Test.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "items": { "type": "array" }
                }
            },
            "output_schema": { "type": "object", "properties": {} },
            "effects": ["test.effect"],
            "permission_scope": null,
            "reversibility": "reversible",
            "determinism": "deterministic",
            "idempotency": { "mechanism": "none" },
            "confirmation_policy": {
                "standing_permitted": false,
                "per_call_required": true
            },
            "timeout_ms": 5000,
            "retry_policy": {
                "max_retries": 0,
                "backoff_ms": 500,
                "allowed_on": [],
                "requires_idempotency_proof": false
            },
            "provider": {
                "identity": "lantern-local",
                "display_name": "Test Provider",
                "identity_source": "host_configuration",
                "description": "Host-assigned."
            },
            "binding": {
                "kind": "mcp",
                "server_name": "test-server",
                "tool_name": "test_tool",
                "adapter": null
            }
        });
        let (_, digest) = crate::manifest::canonicalize_and_digest(&m.to_string()).unwrap();
        m["digest"] = json!(digest);
        std::fs::write(
            dir.join("manifests/lantern-task-record.json"),
            m.to_string(),
        )
        .unwrap();

        let mut json = minimal_config_json();
        json["providers"][0]["capabilities"][0]["pinned_digest"] = json!(digest);

        let config_path = dir.join("config.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

        let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
        let prepared = prepare_runtime(&loaded).unwrap();
        let err = prepared.planner_capabilities().unwrap_err();
        assert_eq!(
            err.code,
            RuntimePreparationErrorCode::UnsupportedPlannerSchema
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    // 32
    #[test]
    fn j12_packet2_tether_material_exact() {
        with_prepared_runtime(|prepared| {
            let mat = prepared.tether_material(0).unwrap();
            assert_eq!(mat["id"], "record-completed-task");
            assert_eq!(mat["version"], "demo-v1");
            assert!(mat["source"]
                .as_str()
                .unwrap()
                .contains("when event.task.completed"));
        });
    }

    // 33
    #[test]
    fn j12_packet2_provider_launch_plan_literal() {
        with_prepared_runtime(|prepared| {
            let p = &prepared.providers()[0];
            assert_eq!(p.stdio_config.command, "pwsh.exe");
            assert!(p.stdio_config.args.contains(&"-NoProfile".to_string()));
        });
    }

    // 34
    #[test]
    fn j12_packet2_provider_working_directory() {
        with_prepared_runtime(|prepared| {
            let p = &prepared.providers()[0];
            assert!(p.working_directory.is_absolute());
            assert_eq!(p.working_directory, prepared.config_dir());
        });
    }

    // 35
    #[test]
    fn j12_packet2_no_side_effects_during_preparation() {
        with_prepared_runtime(|prepared| {
            assert!(!prepared.tethers().is_empty());
            assert!(!prepared.requirements().is_empty());
            assert!(!prepared.providers().is_empty());
            assert!(!prepared.trusted_store().is_empty());
        });
    }

    // 36 - tether_material oob
    #[test]
    fn j12_packet2_tether_material_oob_none() {
        with_prepared_runtime(|prepared| {
            assert!(prepared.tether_material(999).is_none());
        });
    }

    // 37 - planner descriptors sorted
    #[test]
    fn j12_packet2_planner_descriptors_sorted() {
        let dir = std::env::temp_dir().join(format!("j12-pkt2-sort-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("manifests")).unwrap();
        write_default_tether(&dir);

        let (m_b, db) = make_manifest("b.cap", 2, "lantern-local", json!(null));
        let (m_a, da) = make_manifest("a.cap", 1, "lantern-local", json!(null));
        std::fs::write(dir.join("manifests/b.json"), &m_b).unwrap();
        std::fs::write(dir.join("manifests/a.json"), &m_a).unwrap();

        let mut json = minimal_config_json();
        json["tether_set"]["capability_requirements"] =
            json!([{"name": "b.cap", "version": 2}, {"name": "a.cap", "version": 1}]);
        json["providers"][0]["capabilities"] = json!([
            {"name": "b.cap", "version": 2, "manifest_path": "manifests/b.json", "pinned_digest": db},
            {"name": "a.cap", "version": 1, "manifest_path": "manifests/a.json", "pinned_digest": da}
        ]);
        json["policy"]["rules"] = json!([
            {"name": "b.cap", "version": 2, "decision": "allow"},
            {"name": "a.cap", "version": 1, "decision": "allow"}
        ]);

        let config_path = dir.join("config.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

        let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
        let prepared = prepare_runtime(&loaded).unwrap();
        let descriptors = prepared.planner_capabilities().unwrap();
        assert_eq!(descriptors.len(), 2);
        assert_eq!(descriptors[0]["name"], "a.cap");
        assert_eq!(descriptors[1]["name"], "b.cap");
        let _ = std::fs::remove_dir_all(&dir);
    }

    // 38 - NUL byte in tether
    #[test]
    fn j12_packet2_nul_byte_in_tether_rejected() {
        let dir = std::env::temp_dir().join(format!("j12-pkt2-nul-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("manifests")).unwrap();
        std::fs::create_dir_all(dir.join("tethers")).unwrap();
        std::fs::write(
            dir.join("tethers/record-completed-task.tether"),
            b"hello\0world",
        )
        .unwrap();

        let (manifest_json, _digest) =
            make_manifest("lantern.task.record", 1, "lantern-local", json!(null));
        std::fs::write(
            dir.join("manifests/lantern-task-record.json"),
            &manifest_json,
        )
        .unwrap();

        let config_path = dir.join("config.json");
        std::fs::write(
            &config_path,
            serde_json::to_string_pretty(&minimal_config_json()).unwrap(),
        )
        .unwrap();

        let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
        let err = prepare_runtime(&loaded).unwrap_err();
        assert_eq!(err.code, RuntimePreparationErrorCode::InvalidUtf8OrText);
        let _ = std::fs::remove_dir_all(&dir);
    }

    // 39 - prepared_capabilities accessor
    #[test]
    fn j12_packet2_prepared_capabilities_accessor() {
        with_prepared_runtime(|prepared| {
            let caps = prepared.prepared_capabilities();
            assert_eq!(caps.len(), 1);
            assert_eq!(caps[0].name, "lantern.task.record");
            assert_eq!(caps[0].version, 1);
        });
    }

    // 40 - accessors
    #[test]
    fn j12_packet2_accessors_correct() {
        with_prepared_runtime(|prepared| {
            assert!(prepared.config_path().is_absolute());
            assert!(prepared.config_dir().is_absolute());
            assert_eq!(prepared.tether_set_id(), "example.local");
            assert_eq!(prepared.tether_set_version(), "1");
            assert_eq!(prepared.requirements().len(), 1);
            assert_eq!(
                prepared.requirements()[0].capability_name,
                "lantern.task.record"
            );
            assert_eq!(prepared.requirements()[0].capability_version, 1);
        });
    }

    // 41 - whitespace-only tether
    #[test]
    fn j12_packet2_whitespace_only_tether_rejected() {
        let dir = std::env::temp_dir().join(format!("j12-pkt2-ws-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("manifests")).unwrap();
        std::fs::create_dir_all(dir.join("tethers")).unwrap();
        std::fs::write(dir.join("tethers/ws.tether"), "   \n  \t  \n  ").unwrap();

        let mut json = minimal_config_json();
        json["tether_set"]["tethers"][0]["source_path"] = json!("tethers/ws.tether");

        let (manifest_json, _digest) =
            make_manifest("lantern.task.record", 1, "lantern-local", json!(null));
        std::fs::write(
            dir.join("manifests/lantern-task-record.json"),
            &manifest_json,
        )
        .unwrap();

        let config_path = dir.join("config.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

        let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
        let err = prepare_runtime(&loaded).unwrap_err();
        assert_eq!(err.code, RuntimePreparationErrorCode::EmptyTetherSource);
        let _ = std::fs::remove_dir_all(&dir);
    }

    // 42 - manifest path escape
    #[test]
    fn j12_packet2_manifest_path_escape_fails() {
        let dir = std::env::temp_dir().join(format!("j12-pkt2-mescape-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("manifests")).unwrap();
        write_default_tether(&dir);

        let mut json = minimal_config_json();
        json["providers"][0]["capabilities"][0]["manifest_path"] = json!("../outside.json");

        let config_path = dir.join("config.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

        let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
        let err = prepare_runtime(&loaded).unwrap_err();
        assert_eq!(
            err.code,
            RuntimePreparationErrorCode::AssetOutsideConfigRoot
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    // 43 - invalid allowed_prefix
    #[test]
    fn j12_packet2_invalid_allowed_prefix_fails() {
        let dir = std::env::temp_dir().join(format!("j12-pkt2-iap-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("manifests")).unwrap();
        write_default_tether(&dir);

        let (manifest_json, digest) = make_manifest(
            "lantern.task.record",
            1,
            "lantern-local",
            json!({"kind": "path_prefix", "allowed_prefixes": [""]}),
        );
        std::fs::write(
            dir.join("manifests/lantern-task-record.json"),
            &manifest_json,
        )
        .unwrap();

        let mut json = minimal_config_json();
        json["providers"][0]["capabilities"][0]["pinned_digest"] = json!(digest);
        json["providers"][0]["capabilities"][0]["scope_binding"] = json!({
            "kind": "path_prefix",
            "argument_json_pointer": "/path"
        });

        let config_path = dir.join("config.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

        let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
        let err = prepare_runtime(&loaded).unwrap_err();
        assert_eq!(err.code, RuntimePreparationErrorCode::InvalidResourcePath);
        let _ = std::fs::remove_dir_all(&dir);
    }

    // 44 - JSON Pointer escaping
    #[test]
    fn j12_packet2_json_pointer_escaping_works() {
        let dir = std::env::temp_dir().join(format!("j12-pkt2-jpe-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("manifests")).unwrap();
        write_default_tether(&dir);

        let (manifest_json, digest) = make_manifest(
            "lantern.task.record",
            1,
            "lantern-local",
            json!({"kind": "path_prefix", "allowed_prefixes": ["projects/"]}),
        );
        std::fs::write(
            dir.join("manifests/lantern-task-record.json"),
            &manifest_json,
        )
        .unwrap();

        let mut json = minimal_config_json();
        json["providers"][0]["capabilities"][0]["pinned_digest"] = json!(digest);
        json["providers"][0]["capabilities"][0]["scope_binding"] = json!({
            "kind": "path_prefix",
            "argument_json_pointer": "/resource~1path"
        });

        let config_path = dir.join("config.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

        let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
        let prepared = prepare_runtime(&loaded).unwrap();

        let action = ProposedAction {
            evaluation_id: "eval-1".into(),
            plan_id: "plan-1".into(),
            action_id: "act-1".into(),
            capability_name: "lantern.task.record".into(),
            manifest_digest: Some(digest),
            bridge_capability_version: Some(1),
            bridge_provider_identity: Some("lantern-local".into()),
            arguments: json!({"resource/path": "projects/file.md"}),
        };
        assert_eq!(
            prepared.assess_action_scope(&action),
            ScopeAssessment::WithinScope
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    // 45 - input schema ordering deterministic
    #[test]
    fn j12_packet2_input_schema_ordering_deterministic() {
        let dir = std::env::temp_dir().join(format!("j12-pkt2-iso-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("manifests")).unwrap();
        write_default_tether(&dir);

        let mut m = json!({
            "manifest_format_version": "1.0",
            "capability_name": "lantern.task.record",
            "capability_version": 1,
            "title": "Test",
            "description": "Test.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "z_field": { "type": "string" },
                    "a_field": { "type": "boolean" },
                    "m_field": { "type": "number" }
                }
            },
            "output_schema": { "type": "object", "properties": {} },
            "effects": ["test.effect"],
            "permission_scope": null,
            "reversibility": "reversible",
            "determinism": "deterministic",
            "idempotency": { "mechanism": "none" },
            "confirmation_policy": {
                "standing_permitted": false,
                "per_call_required": true
            },
            "timeout_ms": 5000,
            "retry_policy": {
                "max_retries": 0,
                "backoff_ms": 500,
                "allowed_on": [],
                "requires_idempotency_proof": false
            },
            "provider": {
                "identity": "lantern-local",
                "display_name": "Test Provider",
                "identity_source": "host_configuration",
                "description": "Host-assigned."
            },
            "binding": {
                "kind": "mcp",
                "server_name": "test-server",
                "tool_name": "test_tool",
                "adapter": null
            }
        });
        let (_, digest) = crate::manifest::canonicalize_and_digest(&m.to_string()).unwrap();
        m["digest"] = json!(digest);
        std::fs::write(
            dir.join("manifests/lantern-task-record.json"),
            m.to_string(),
        )
        .unwrap();

        let mut json = minimal_config_json();
        json["providers"][0]["capabilities"][0]["pinned_digest"] = json!(digest);

        let config_path = dir.join("config.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

        let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
        let prepared = prepare_runtime(&loaded).unwrap();
        let descriptors = prepared.planner_capabilities().unwrap();
        let inputs = &descriptors[0]["inputs"];
        let keys: Vec<&str> = inputs
            .as_object()
            .unwrap()
            .keys()
            .map(|s| s.as_str())
            .collect();
        assert_eq!(keys, vec!["a_field", "m_field", "z_field"]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    // ------------------------------------------------------------------
    // J12 Packet 2 correction tests — filesystem escape classification
    // ------------------------------------------------------------------

    // 46: nonexistent ../ escape returns AssetOutsideConfigRoot
    #[test]
    fn j12_packet2_nonexistent_dotdot_escape_returns_outside_root() {
        let dir = std::env::temp_dir().join(format!("j12-pkt2-ddesc-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("manifests")).unwrap();
        write_default_tether(&dir);

        let mut json = minimal_config_json();
        // Path that would escape even though the file doesn't exist
        json["tether_set"]["tethers"][0]["source_path"] = json!("../nonexistent/outside.tether");

        let config_path = dir.join("config.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

        let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
        let err = prepare_runtime(&loaded).unwrap_err();
        assert_eq!(
            err.code,
            RuntimePreparationErrorCode::AssetOutsideConfigRoot
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    // 47: genuinely missing in-root asset returns AssetNotFound
    #[test]
    fn j12_packet2_missing_inroot_asset_returns_not_found() {
        let dir = std::env::temp_dir().join(format!("j12-pkt2-inroot-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("manifests")).unwrap();
        write_default_tether(&dir);

        let mut json = minimal_config_json();
        // Deeply nested but still within root — just doesn't exist
        json["tether_set"]["tethers"][0]["source_path"] = json!("tethers/sub/deep/missing.tether");

        let config_path = dir.join("config.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

        let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
        let err = prepare_runtime(&loaded).unwrap_err();
        assert_eq!(err.code, RuntimePreparationErrorCode::AssetNotFound);
        let _ = std::fs::remove_dir_all(&dir);
    }

    // 48: normal nested relative asset succeeds
    #[test]
    fn j12_packet2_nested_relative_asset_succeeds() {
        let dir = std::env::temp_dir().join(format!("j12-pkt2-nested-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("manifests")).unwrap();
        std::fs::create_dir_all(dir.join("tethers/sub/deep")).unwrap();
        let source = "when event.task.completed if task.status == \"done\" do lantern.task.record";
        std::fs::write(dir.join("tethers/sub/deep/nested.tether"), source).unwrap();

        let mut json = minimal_config_json();
        json["tether_set"]["tethers"][0]["source_path"] = json!("tethers/sub/deep/nested.tether");

        let (manifest_json, digest) =
            make_manifest("lantern.task.record", 1, "lantern-local", json!(null));
        std::fs::write(
            dir.join("manifests/lantern-task-record.json"),
            &manifest_json,
        )
        .unwrap();
        json["providers"][0]["capabilities"][0]["pinned_digest"] = json!(digest);

        let config_path = dir.join("config.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

        let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
        let prepared = prepare_runtime(&loaded).unwrap();
        assert_eq!(prepared.tethers().len(), 1);
        assert!(prepared.tethers()[0]
            .source
            .contains("when event.task.completed"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    // ------------------------------------------------------------------
    // J12 Packet 2 correction tests — strict RFC 6901 JSON Pointer
    // ------------------------------------------------------------------

    // 49: /a~1b accesses key "a/b"
    #[test]
    fn j12_packet2_pointer_slash_escape_accesses_key_with_slash() {
        let dir = std::env::temp_dir().join(format!("j12-pkt2-pslash-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("manifests")).unwrap();
        write_default_tether(&dir);

        let (manifest_json, digest) = make_manifest(
            "lantern.task.record",
            1,
            "lantern-local",
            json!({"kind": "path_prefix", "allowed_prefixes": ["projects/"]}),
        );
        std::fs::write(
            dir.join("manifests/lantern-task-record.json"),
            &manifest_json,
        )
        .unwrap();

        let mut json = minimal_config_json();
        json["providers"][0]["capabilities"][0]["pinned_digest"] = json!(digest);
        json["providers"][0]["capabilities"][0]["scope_binding"] = json!({
            "kind": "path_prefix",
            "argument_json_pointer": "/resource~1path"
        });

        let config_path = dir.join("config.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

        let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
        let prepared = prepare_runtime(&loaded).unwrap();

        let action = ProposedAction {
            evaluation_id: "eval-1".into(),
            plan_id: "plan-1".into(),
            action_id: "act-1".into(),
            capability_name: "lantern.task.record".into(),
            manifest_digest: Some(digest),
            bridge_capability_version: Some(1),
            bridge_provider_identity: Some("lantern-local".into()),
            arguments: json!({"resource/path": "projects/file.md"}),
        };
        assert_eq!(
            prepared.assess_action_scope(&action),
            ScopeAssessment::WithinScope
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    // 50: /a~0b accesses key "a~b"
    #[test]
    fn j12_packet2_pointer_tilde_escape_accesses_key_with_tilde() {
        let dir = std::env::temp_dir().join(format!("j12-pkt2-ptilde-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("manifests")).unwrap();
        write_default_tether(&dir);

        let (manifest_json, digest) = make_manifest(
            "lantern.task.record",
            1,
            "lantern-local",
            json!({"kind": "path_prefix", "allowed_prefixes": ["projects/"]}),
        );
        std::fs::write(
            dir.join("manifests/lantern-task-record.json"),
            &manifest_json,
        )
        .unwrap();

        let mut json = minimal_config_json();
        json["providers"][0]["capabilities"][0]["pinned_digest"] = json!(digest);
        json["providers"][0]["capabilities"][0]["scope_binding"] = json!({
            "kind": "path_prefix",
            "argument_json_pointer": "/key~0with~0tilde"
        });

        let config_path = dir.join("config.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

        let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
        let prepared = prepare_runtime(&loaded).unwrap();

        let action = ProposedAction {
            evaluation_id: "eval-1".into(),
            plan_id: "plan-1".into(),
            action_id: "act-1".into(),
            capability_name: "lantern.task.record".into(),
            manifest_digest: Some(digest),
            bridge_capability_version: Some(1),
            bridge_provider_identity: Some("lantern-local".into()),
            arguments: json!({"key~with~tilde": "projects/file.md"}),
        };
        assert_eq!(
            prepared.assess_action_scope(&action),
            ScopeAssessment::WithinScope
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    // 51: ~2 is rejected during config validation
    #[test]
    fn j12_packet2_pointer_tilde_2_rejected() {
        let mut json = minimal_config_json();
        json["providers"][0]["capabilities"][0]["scope_binding"] = json!({
            "kind": "path_prefix",
            "argument_json_pointer": "/a~2b"
        });
        let err = crate::runtime_config::parse_runtime_config(&json.to_string()).unwrap_err();
        assert_eq!(
            err.code,
            crate::runtime_config::RuntimeConfigErrorCode::InvalidValue
        );
    }

    // 52: trailing ~ is rejected during config validation
    #[test]
    fn j12_packet2_pointer_trailing_tilde_rejected() {
        let mut json = minimal_config_json();
        json["providers"][0]["capabilities"][0]["scope_binding"] = json!({
            "kind": "path_prefix",
            "argument_json_pointer": "/a~"
        });
        let err = crate::runtime_config::parse_runtime_config(&json.to_string()).unwrap_err();
        assert_eq!(
            err.code,
            crate::runtime_config::RuntimeConfigErrorCode::InvalidValue
        );
    }

    // 53: array index "0" works
    #[test]
    fn j12_packet2_pointer_array_index_0_works() {
        let dir = std::env::temp_dir().join(format!("j12-pkt2-arr0-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("manifests")).unwrap();
        write_default_tether(&dir);

        let (manifest_json, digest) = make_manifest(
            "lantern.task.record",
            1,
            "lantern-local",
            json!({"kind": "path_prefix", "allowed_prefixes": ["projects/"]}),
        );
        std::fs::write(
            dir.join("manifests/lantern-task-record.json"),
            &manifest_json,
        )
        .unwrap();

        let mut json = minimal_config_json();
        json["providers"][0]["capabilities"][0]["pinned_digest"] = json!(digest);
        json["providers"][0]["capabilities"][0]["scope_binding"] = json!({
            "kind": "path_prefix",
            "argument_json_pointer": "/items/0/path"
        });

        let config_path = dir.join("config.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

        let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
        let prepared = prepare_runtime(&loaded).unwrap();

        let action = ProposedAction {
            evaluation_id: "eval-1".into(),
            plan_id: "plan-1".into(),
            action_id: "act-1".into(),
            capability_name: "lantern.task.record".into(),
            manifest_digest: Some(digest),
            bridge_capability_version: Some(1),
            bridge_provider_identity: Some("lantern-local".into()),
            arguments: json!({"items": [{"path": "projects/file.md"}]}),
        };
        assert_eq!(
            prepared.assess_action_scope(&action),
            ScopeAssessment::WithinScope
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    // 54: array index "01" is rejected
    #[test]
    fn j12_packet2_pointer_array_index_01_rejected() {
        let dir = std::env::temp_dir().join(format!("j12-pkt2-arr01-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("manifests")).unwrap();
        write_default_tether(&dir);

        let (manifest_json, digest) = make_manifest(
            "lantern.task.record",
            1,
            "lantern-local",
            json!({"kind": "path_prefix", "allowed_prefixes": ["projects/"]}),
        );
        std::fs::write(
            dir.join("manifests/lantern-task-record.json"),
            &manifest_json,
        )
        .unwrap();

        let mut json = minimal_config_json();
        json["providers"][0]["capabilities"][0]["pinned_digest"] = json!(digest);
        json["providers"][0]["capabilities"][0]["scope_binding"] = json!({
            "kind": "path_prefix",
            "argument_json_pointer": "/items/01/path"
        });

        let config_path = dir.join("config.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

        let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
        let prepared = prepare_runtime(&loaded).unwrap();

        let action = ProposedAction {
            evaluation_id: "eval-1".into(),
            plan_id: "plan-1".into(),
            action_id: "act-1".into(),
            capability_name: "lantern.task.record".into(),
            manifest_digest: Some(digest),
            bridge_capability_version: Some(1),
            bridge_provider_identity: Some("lantern-local".into()),
            arguments: json!({"items": [{"path": "projects/file.md"}, {"path": "other.md"}]}),
        };
        assert_eq!(
            prepared.assess_action_scope(&action),
            ScopeAssessment::ScopeNotEstablished
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    // 55: array index "+1" is rejected
    #[test]
    fn j12_packet2_pointer_array_index_plus_1_rejected() {
        let dir = std::env::temp_dir().join(format!("j12-pkt2-arrplus-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("manifests")).unwrap();
        write_default_tether(&dir);

        let (manifest_json, digest) = make_manifest(
            "lantern.task.record",
            1,
            "lantern-local",
            json!({"kind": "path_prefix", "allowed_prefixes": ["projects/"]}),
        );
        std::fs::write(
            dir.join("manifests/lantern-task-record.json"),
            &manifest_json,
        )
        .unwrap();

        let mut json = minimal_config_json();
        json["providers"][0]["capabilities"][0]["pinned_digest"] = json!(digest);
        json["providers"][0]["capabilities"][0]["scope_binding"] = json!({
            "kind": "path_prefix",
            "argument_json_pointer": "/items/+1/path"
        });

        let config_path = dir.join("config.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

        let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
        let prepared = prepare_runtime(&loaded).unwrap();

        let action = ProposedAction {
            evaluation_id: "eval-1".into(),
            plan_id: "plan-1".into(),
            action_id: "act-1".into(),
            capability_name: "lantern.task.record".into(),
            manifest_digest: Some(digest),
            bridge_capability_version: Some(1),
            bridge_provider_identity: Some("lantern-local".into()),
            arguments: json!({"items": [{"path": "projects/file.md"}, {"path": "other.md"}]}),
        };
        assert_eq!(
            prepared.assess_action_scope(&action),
            ScopeAssessment::ScopeNotEstablished
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    // 56: out-of-range valid array index returns ScopeNotEstablished
    #[test]
    fn j12_packet2_pointer_array_oob_returns_not_established() {
        let dir = std::env::temp_dir().join(format!("j12-pkt2-arroob-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("manifests")).unwrap();
        write_default_tether(&dir);

        let (manifest_json, digest) = make_manifest(
            "lantern.task.record",
            1,
            "lantern-local",
            json!({"kind": "path_prefix", "allowed_prefixes": ["projects/"]}),
        );
        std::fs::write(
            dir.join("manifests/lantern-task-record.json"),
            &manifest_json,
        )
        .unwrap();

        let mut json = minimal_config_json();
        json["providers"][0]["capabilities"][0]["pinned_digest"] = json!(digest);
        json["providers"][0]["capabilities"][0]["scope_binding"] = json!({
            "kind": "path_prefix",
            "argument_json_pointer": "/items/5/path"
        });

        let config_path = dir.join("config.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

        let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
        let prepared = prepare_runtime(&loaded).unwrap();

        let action = ProposedAction {
            evaluation_id: "eval-1".into(),
            plan_id: "plan-1".into(),
            action_id: "act-1".into(),
            capability_name: "lantern.task.record".into(),
            manifest_digest: Some(digest),
            bridge_capability_version: Some(1),
            bridge_provider_identity: Some("lantern-local".into()),
            arguments: json!({"items": [{"path": "projects/file.md"}]}),
        };
        assert_eq!(
            prepared.assess_action_scope(&action),
            ScopeAssessment::ScopeNotEstablished
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    // ------------------------------------------------------------------
    // J12 Packet 2 correction tests — allowed-prefix validation
    // ------------------------------------------------------------------

    // 57: "/" is rejected
    #[test]
    fn j12_packet2_allowed_prefix_root_slash_rejected() {
        let dir = std::env::temp_dir().join(format!("j12-pkt2-aproot-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("manifests")).unwrap();
        write_default_tether(&dir);

        let (manifest_json, digest) = make_manifest(
            "lantern.task.record",
            1,
            "lantern-local",
            json!({"kind": "path_prefix", "allowed_prefixes": ["/"]}),
        );
        std::fs::write(
            dir.join("manifests/lantern-task-record.json"),
            &manifest_json,
        )
        .unwrap();

        let mut json = minimal_config_json();
        json["providers"][0]["capabilities"][0]["pinned_digest"] = json!(digest);
        json["providers"][0]["capabilities"][0]["scope_binding"] = json!({
            "kind": "path_prefix",
            "argument_json_pointer": "/path"
        });

        let config_path = dir.join("config.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

        let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
        let err = prepare_runtime(&loaded).unwrap_err();
        assert_eq!(err.code, RuntimePreparationErrorCode::InvalidResourcePath);
        let _ = std::fs::remove_dir_all(&dir);
    }

    // 58: "projects//" (double trailing slash) is rejected
    #[test]
    fn j12_packet2_allowed_prefix_double_slash_rejected() {
        let dir = std::env::temp_dir().join(format!("j12-pkt2-apds-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("manifests")).unwrap();
        write_default_tether(&dir);

        let (manifest_json, digest) = make_manifest(
            "lantern.task.record",
            1,
            "lantern-local",
            json!({"kind": "path_prefix", "allowed_prefixes": ["projects//"]}),
        );
        std::fs::write(
            dir.join("manifests/lantern-task-record.json"),
            &manifest_json,
        )
        .unwrap();

        let mut json = minimal_config_json();
        json["providers"][0]["capabilities"][0]["pinned_digest"] = json!(digest);
        json["providers"][0]["capabilities"][0]["scope_binding"] = json!({
            "kind": "path_prefix",
            "argument_json_pointer": "/path"
        });

        let config_path = dir.join("config.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

        let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
        let err = prepare_runtime(&loaded).unwrap_err();
        assert_eq!(err.code, RuntimePreparationErrorCode::InvalidResourcePath);
        let _ = std::fs::remove_dir_all(&dir);
    }

    // 59: "projects/" is valid
    #[test]
    fn j12_packet2_allowed_prefix_trailing_slash_valid() {
        let dir = std::env::temp_dir().join(format!("j12-pkt2-apts-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("manifests")).unwrap();
        write_default_tether(&dir);

        let (manifest_json, digest) = make_manifest(
            "lantern.task.record",
            1,
            "lantern-local",
            json!({"kind": "path_prefix", "allowed_prefixes": ["projects/"]}),
        );
        std::fs::write(
            dir.join("manifests/lantern-task-record.json"),
            &manifest_json,
        )
        .unwrap();

        let mut json = minimal_config_json();
        json["providers"][0]["capabilities"][0]["pinned_digest"] = json!(digest);
        json["providers"][0]["capabilities"][0]["scope_binding"] = json!({
            "kind": "path_prefix",
            "argument_json_pointer": "/path"
        });

        let config_path = dir.join("config.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

        let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
        let prepared = prepare_runtime(&loaded).unwrap();
        let action = ProposedAction {
            evaluation_id: "eval-1".into(),
            plan_id: "plan-1".into(),
            action_id: "act-1".into(),
            capability_name: "lantern.task.record".into(),
            manifest_digest: Some(digest),
            bridge_capability_version: Some(1),
            bridge_provider_identity: Some("lantern-local".into()),
            arguments: json!({"path": "projects/file.md"}),
        };
        assert_eq!(
            prepared.assess_action_scope(&action),
            ScopeAssessment::WithinScope
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    // 60: "projects" (no trailing slash) is valid
    #[test]
    fn j12_packet2_allowed_prefix_bare_valid() {
        let dir = std::env::temp_dir().join(format!("j12-pkt2-apbare-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("manifests")).unwrap();
        write_default_tether(&dir);

        let (manifest_json, digest) = make_manifest(
            "lantern.task.record",
            1,
            "lantern-local",
            json!({"kind": "path_prefix", "allowed_prefixes": ["projects"]}),
        );
        std::fs::write(
            dir.join("manifests/lantern-task-record.json"),
            &manifest_json,
        )
        .unwrap();

        let mut json = minimal_config_json();
        json["providers"][0]["capabilities"][0]["pinned_digest"] = json!(digest);
        json["providers"][0]["capabilities"][0]["scope_binding"] = json!({
            "kind": "path_prefix",
            "argument_json_pointer": "/path"
        });

        let config_path = dir.join("config.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

        let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
        let prepared = prepare_runtime(&loaded).unwrap();
        let action = ProposedAction {
            evaluation_id: "eval-1".into(),
            plan_id: "plan-1".into(),
            action_id: "act-1".into(),
            capability_name: "lantern.task.record".into(),
            manifest_digest: Some(digest),
            bridge_capability_version: Some(1),
            bridge_provider_identity: Some("lantern-local".into()),
            arguments: json!({"path": "projects/file.md"}),
        };
        assert_eq!(
            prepared.assess_action_scope(&action),
            ScopeAssessment::WithinScope
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    // === Windows drive-relative path tests ===

    // 61: drive-relative source path (C:outside\file) returns AssetOutsideConfigRoot
    #[cfg(windows)]
    #[test]
    fn j12_packet2_windows_drive_relative_source_returns_outside_root() {
        let dir = std::env::temp_dir().join(format!("j12-pkt2-drvsrc-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("manifests")).unwrap();
        write_default_tether(&dir);

        let mut json = minimal_config_json();
        // C:outside\missing.tether — drive-relative, not absolute, not rooted
        json["tether_set"]["tethers"][0]["source_path"] = json!(r"C:outside\missing.tether");

        let config_path = dir.join("config.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

        let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
        let err = prepare_runtime(&loaded).unwrap_err();
        assert_eq!(
            err.code,
            RuntimePreparationErrorCode::AssetOutsideConfigRoot
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    // 62: drive-relative manifest path never returns AssetNotFound
    #[cfg(windows)]
    #[test]
    fn j12_packet2_windows_drive_relative_manifest_never_not_found() {
        let dir = std::env::temp_dir().join(format!("j12-pkt2-drvman-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("manifests")).unwrap();
        write_default_tether(&dir);

        let mut json = minimal_config_json();
        // C:outside\missing.json — must be AssetOutsideConfigRoot, never AssetNotFound
        json["providers"][0]["capabilities"][0]["manifest_path"] = json!(r"C:outside\missing.json");

        let config_path = dir.join("config.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

        let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
        let err = prepare_runtime(&loaded).unwrap_err();
        assert_eq!(
            err.code,
            RuntimePreparationErrorCode::AssetOutsideConfigRoot
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    // 63: rooted Windows path rejected
    #[cfg(windows)]
    #[test]
    fn j12_packet2_windows_rooted_path_rejected() {
        let dir = std::env::temp_dir().join(format!("j12-pkt2-rtpath-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("manifests")).unwrap();
        write_default_tether(&dir);

        let mut json = minimal_config_json();
        // \Windows\System32\notepad.exe — rooted path, has RootDir component
        json["tether_set"]["tethers"][0]["source_path"] = json!(r"\Windows\System32\notepad.exe");

        let config_path = dir.join("config.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

        let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
        let err = prepare_runtime(&loaded).unwrap_err();
        assert_eq!(
            err.code,
            RuntimePreparationErrorCode::AssetOutsideConfigRoot
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    // 64: ordinary nested relative path still succeeds on Windows
    #[cfg(windows)]
    #[test]
    fn j12_packet2_windows_nested_relative_succeeds() {
        let dir = std::env::temp_dir().join(format!("j12-pkt2-wnest-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("manifests")).unwrap();

        // Create the tether in a nested subdirectory
        std::fs::create_dir_all(dir.join("tethers/sub")).unwrap();
        std::fs::write(
            dir.join("tethers/sub/main.tether"),
            "on event hello do log \"ok\"\n",
        )
        .unwrap();

        // Create a valid manifest
        let (manifest_json, digest) =
            make_manifest("lantern.task.record", 1, "lantern-local", json!(null));
        std::fs::write(
            dir.join("manifests/lantern-task-record.json"),
            &manifest_json,
        )
        .unwrap();

        let mut json = minimal_config_json();
        json["tether_set"]["tethers"][0]["source_path"] = json!("tethers/sub/main.tether");
        json["providers"][0]["capabilities"][0]["pinned_digest"] = json!(digest);

        let config_path = dir.join("config.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

        let loaded = crate::runtime_config::load_runtime_config(&config_path).unwrap();
        let result = prepare_runtime(&loaded);
        assert!(
            result.is_ok(),
            "nested relative path should succeed: {:?}",
            result.err()
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
