// Columbo — Live exact-version capability resolution
//
// Resolves an admitted, verified capability as currently usable only
// when both the trusted manifest store and the host-supplied provider
// availability snapshot agree on the exact identity, name, and version.
//
// An admitted manifest alone does not prove availability.  A live
// provider alone does not prove trust or admission.  Both must agree.

use crate::manifest::VerifiedManifest;
use crate::trusted_store::TrustedManifestStore;
use std::collections::HashSet;

// ---------------------------------------------------------------------------
// Capability identity
// ---------------------------------------------------------------------------

/// Exact (name, version) pair identifying a capability.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CapabilityIdentity {
    pub name: String,
    pub version: u32,
}

impl CapabilityIdentity {
    pub fn new(name: impl Into<String>, version: u32) -> Self {
        Self {
            name: name.into(),
            version,
        }
    }
}

// ---------------------------------------------------------------------------
// Provider availability
// ---------------------------------------------------------------------------

/// A host-supplied snapshot of currently available provider identities.
///
/// This is an explicit, host-controlled set.  Availability is reported
/// by the host at the start of each evaluation/dispatch cycle; it does
/// not automatically discover providers and does not promise a
/// connection will remain live.
///
/// An empty set means no providers are currently reachable, even if
/// manifests have been admitted for them.
#[derive(Debug, Clone, Default)]
pub struct ProviderAvailability {
    available: HashSet<String>,
}

impl ProviderAvailability {
    /// No providers available.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Build from a list of currently available provider identities.
    pub fn from_identities(identities: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            available: identities.into_iter().map(Into::into).collect(),
        }
    }

    /// Whether a provider with the given identity is currently available.
    pub fn is_available(&self, identity: &str) -> bool {
        self.available.contains(identity)
    }

    /// Number of available providers.
    pub fn len(&self) -> usize {
        self.available.len()
    }

    /// Whether no providers are available.
    pub fn is_empty(&self) -> bool {
        self.available.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Resolved capability
// ---------------------------------------------------------------------------

/// A capability that has been resolved as currently usable.
///
/// Resolution requires:
/// - an admitted verified manifest in the trusted store for the exact
///   (name, version);
/// - the manifest's provider identity appearing in the current
///   availability snapshot;
/// - the caller's expected provider identity matching the manifest's.
///
/// A resolved capability carries everything later dispatch needs:
/// the verified manifest digest, provider identity, binding metadata,
/// input/output schemas, effects, and execution policy.
///
/// It does **not** grant execution permission. That remains a separate
/// host decision before dispatch.
#[derive(Debug, Clone)]
pub struct ResolvedCapability {
    /// Exact (capability_name, capability_version).
    identity: CapabilityIdentity,
    /// Host-assigned provider identity from the manifest.
    provider_identity: String,
    /// Verified cryptographic digest of the manifest.
    manifest_digest: String,
    /// The complete underlying verified manifest for later dispatch use.
    manifest: VerifiedManifest,
}

impl ResolvedCapability {
    pub fn identity(&self) -> &CapabilityIdentity {
        &self.identity
    }

    pub fn capability_name(&self) -> &str {
        &self.identity.name
    }

    pub fn capability_version(&self) -> u32 {
        self.identity.version
    }

    pub fn provider_identity(&self) -> &str {
        &self.provider_identity
    }

    pub fn manifest_digest(&self) -> &str {
        &self.manifest_digest
    }

    pub fn manifest(&self) -> &VerifiedManifest {
        &self.manifest
    }
}

// ---------------------------------------------------------------------------
// Resolution error
// ---------------------------------------------------------------------------

/// Why a capability could not be resolved as currently usable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolutionError {
    /// No manifest in the trusted store for this exact (name, version).
    NoAdmittedManifest {
        capability_name: String,
        capability_version: u32,
    },
    /// An admitted manifest exists, but the provider that owns it is
    /// not in the current availability snapshot.
    ProviderUnavailable {
        capability_name: String,
        capability_version: u32,
        provider_identity: String,
    },
    /// The caller expected a capability from one provider, but the
    /// admitted manifest belongs to a different provider identity.
    ProviderIdentityMismatch {
        capability_name: String,
        capability_version: u32,
        expected_provider: String,
        actual_provider: String,
    },
}

// ---------------------------------------------------------------------------
// Live capability projection
// ---------------------------------------------------------------------------

/// One projected capability entry for planning.
///
/// Produced by projecting a Tether Set requirement through the trusted
/// manifest store and explicit provider availability.  Contains only
/// the fields planning needs: exact identity, required effects, and
/// the opaque manifest digest.
///
/// Projection is read-only and deterministic.  It carries no execution
/// permission, makes no dispatch decisions, and does not mutate the
/// store or availability snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectedCapability {
    /// Exact capability name.
    pub capability_name: String,
    /// Exact capability version.
    pub capability_version: u32,
    /// Required effects declared by the capability's manifest.
    pub required_effects: Vec<String>,
    /// Opaque verified manifest digest for later dispatch binding.
    pub manifest_digest: String,
    /// Provider identity currently owning this projected capability.
    pub provider_identity: String,
}

/// Project a list of Tether Set capability requirements into a
/// deterministic live capability view for planning.
///
/// Each requirement is independently resolved through the trusted
/// manifest store and provider availability snapshot.  Failures are
/// silently omitted — the projection fails closed per capability.
///
/// Resolution failures that cause omission:
/// - No admitted manifest for the exact (name, version).
/// - Admitted manifest exists but provider is not in the availability snapshot.
/// - Provider identity mismatch between manifest and expected provider.
///
/// The projection is read-only: no process launch, no dispatch, no
/// policy decision, no planner I/O, and no protocol mutation.
pub fn project_capabilities(
    requirements: &[(String, u32)],
    store: &TrustedManifestStore,
    availability: &ProviderAvailability,
) -> Vec<ProjectedCapability> {
    requirements
        .iter()
        .filter_map(|(name, version)| {
            let resolved = resolve_capability(store, availability, name, *version, None).ok()?;

            let effects = resolved.manifest().manifest().effects.clone();

            Some(ProjectedCapability {
                capability_name: resolved.capability_name().to_owned(),
                capability_version: resolved.capability_version(),
                required_effects: effects,
                manifest_digest: resolved.manifest_digest().to_owned(),
                provider_identity: resolved.provider_identity().to_owned(),
            })
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Resolution
// ---------------------------------------------------------------------------

/// Resolve an exact-version capability as currently usable.
///
/// Checks (in order):
/// 1. Trusted store — does an admitted verified manifest exist for the
///    exact (capability_name, capability_version)?
/// 2. Provider availability — is the manifest's provider identity in
///    the current availability snapshot?
/// 3. Caller identity — does the manifest's provider identity match the
///    expected provider identity (if supplied)?
///
/// Returns `ResolvedCapability` only when all three agree.  Any
/// disagreement returns a precise `ResolutionError`.
///
/// If `expected_provider` is `None`, step 3 is skipped — the caller
/// accepts whatever provider owns the manifest.  Callers that know the
/// required provider identity should supply it to catch configuration
/// errors early.
pub fn resolve_capability(
    store: &TrustedManifestStore,
    availability: &ProviderAvailability,
    name: &str,
    version: u32,
    expected_provider: Option<&str>,
) -> Result<ResolvedCapability, ResolutionError> {
    // 1. Admitted manifest.
    let verified = store.get_by_name_version(name, version).ok_or_else(|| {
        ResolutionError::NoAdmittedManifest {
            capability_name: name.to_owned(),
            capability_version: version,
        }
    })?;

    let manifest = verified.manifest();
    let provider_identity = &manifest.provider.identity;

    // 2. Provider available.
    if !availability.is_available(provider_identity) {
        return Err(ResolutionError::ProviderUnavailable {
            capability_name: name.to_owned(),
            capability_version: version,
            provider_identity: provider_identity.clone(),
        });
    }

    // 3. Caller identity check (when supplied).
    if let Some(expected) = expected_provider {
        if provider_identity != expected {
            return Err(ResolutionError::ProviderIdentityMismatch {
                capability_name: name.to_owned(),
                capability_version: version,
                expected_provider: expected.to_owned(),
                actual_provider: provider_identity.clone(),
            });
        }
    }

    Ok(ResolvedCapability {
        identity: CapabilityIdentity::new(name, version),
        provider_identity: provider_identity.clone(),
        manifest_digest: verified.verified_digest().to_owned(),
        manifest: verified.clone(),
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::verify_manifest;
    use serde_json::json;

    // -- helpers: manifest JSON fixtures --

    fn read_manifest_json() -> serde_json::Value {
        json!({
            "manifest_format_version": "1.0",
            "capability_name": "notes.note.read",
            "capability_version": 1,
            "title": "Read a note",
            "description": "Read a note.",
            "input_schema": {
                "type": "object",
                "properties": { "path": { "type": "string" } },
                "required": ["path"],
                "additionalProperties": false
            },
            "output_schema": {
                "type": "object",
                "properties": { "content": { "type": "string" } },
                "required": ["content"]
            },
            "effects": ["filesystem.read"],
            "permission_scope": {
                "kind": "path_prefix",
                "allowed_prefixes": ["projects/"]
            },
            "reversibility": "reversible",
            "determinism": "deterministic",
            "idempotency": { "mechanism": "none" },
            "confirmation_policy": {
                "standing_permitted": true,
                "per_call_required": false
            },
            "timeout_ms": 5000,
            "retry_policy": {
                "max_retries": 0,
                "backoff_ms": 500,
                "allowed_on": ["outcome_unknown"],
                "requires_idempotency_proof": false
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
                "tool_name": "obsidian_read_note",
                "adapter": null
            }
        })
    }

    fn write_manifest_json() -> serde_json::Value {
        json!({
            "manifest_format_version": "1.0",
            "capability_name": "notes.note.create",
            "capability_version": 1,
            "title": "Create a note",
            "description": "Create a note.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "title": { "type": "string" },
                    "content": { "type": "string" }
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
                "key_source": "evaluation_id/action_id"
            },
            "confirmation_policy": {
                "standing_permitted": false,
                "per_call_required": true
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

    fn verified_read() -> VerifiedManifest {
        let mut m = read_manifest_json();
        let (_, digest) = crate::manifest::canonicalize_and_digest(&m.to_string()).unwrap();
        m["digest"] = json!(digest);
        verify_manifest(&m.to_string()).unwrap()
    }

    fn verified_read_v2() -> VerifiedManifest {
        let mut m = read_manifest_json();
        m["capability_version"] = json!(2);
        let (_, digest) = crate::manifest::canonicalize_and_digest(&m.to_string()).unwrap();
        m["digest"] = json!(digest);
        verify_manifest(&m.to_string()).unwrap()
    }

    fn verified_write() -> VerifiedManifest {
        let mut m = write_manifest_json();
        let (_, digest) = crate::manifest::canonicalize_and_digest(&m.to_string()).unwrap();
        m["digest"] = json!(digest);
        verify_manifest(&m.to_string()).unwrap()
    }

    fn obsidian_available() -> ProviderAvailability {
        ProviderAvailability::from_identities(["obsidian-local"])
    }

    fn admitted_store() -> TrustedManifestStore {
        let mut store = TrustedManifestStore::new();
        store.insert(verified_read()).unwrap();
        store.insert(verified_write()).unwrap();
        store
    }

    // -- resolution succeeds --

    #[test]
    fn resolve_admitted_available_capability() {
        let store = admitted_store();
        let availability = obsidian_available();

        let resolved = resolve_capability(
            &store,
            &availability,
            "notes.note.read",
            1,
            Some("obsidian-local"),
        )
        .unwrap();

        assert_eq!(resolved.capability_name(), "notes.note.read");
        assert_eq!(resolved.capability_version(), 1);
        assert_eq!(resolved.provider_identity(), "obsidian-local");
        assert_eq!(
            resolved.manifest_digest(),
            verified_read().verified_digest()
        );
    }

    #[test]
    fn resolve_without_expected_provider_succeeds() {
        let store = admitted_store();
        let availability = obsidian_available();

        let resolved = resolve_capability(
            &store,
            &availability,
            "notes.note.read",
            1,
            None, // accept whatever provider owns the manifest
        )
        .unwrap();

        assert_eq!(resolved.capability_name(), "notes.note.read");
        assert_eq!(resolved.provider_identity(), "obsidian-local");
    }

    #[test]
    fn resolve_write_capability() {
        let store = admitted_store();
        let availability = obsidian_available();

        let resolved = resolve_capability(
            &store,
            &availability,
            "notes.note.create",
            1,
            Some("obsidian-local"),
        )
        .unwrap();

        assert_eq!(resolved.capability_name(), "notes.note.create");
        assert_eq!(resolved.provider_identity(), "obsidian-local");
    }

    // -- no admitted manifest --

    #[test]
    fn no_admitted_manifest_fails() {
        let store = TrustedManifestStore::new(); // empty
        let availability = obsidian_available();

        let err = resolve_capability(
            &store,
            &availability,
            "notes.note.read",
            1,
            Some("obsidian-local"),
        )
        .unwrap_err();

        match err {
            ResolutionError::NoAdmittedManifest {
                capability_name,
                capability_version,
            } => {
                assert_eq!(capability_name, "notes.note.read");
                assert_eq!(capability_version, 1);
            }
            other => panic!("expected NoAdmittedManifest, got {:?}", other),
        }
    }

    // -- different version does not resolve --

    #[test]
    fn different_version_does_not_resolve() {
        // Store has notes.note.read@1.  Asking for @2 must fail even
        // though the provider is available.
        let store = admitted_store();
        let availability = obsidian_available();

        let err = resolve_capability(
            &store,
            &availability,
            "notes.note.read",
            2,
            Some("obsidian-local"),
        )
        .unwrap_err();

        match err {
            ResolutionError::NoAdmittedManifest {
                capability_name,
                capability_version,
            } => {
                assert_eq!(capability_name, "notes.note.read");
                assert_eq!(capability_version, 2);
            }
            other => panic!("expected NoAdmittedManifest, got {:?}", other),
        }
    }

    #[test]
    fn same_name_v2_version_is_separate_admission() {
        // If version 2 is separately admitted, it resolves independently.
        let mut store = admitted_store();
        store.insert(verified_read_v2()).unwrap();
        let availability = obsidian_available();

        let v1 = resolve_capability(
            &store,
            &availability,
            "notes.note.read",
            1,
            Some("obsidian-local"),
        )
        .unwrap();
        assert_eq!(v1.capability_version(), 1);

        let v2 = resolve_capability(
            &store,
            &availability,
            "notes.note.read",
            2,
            Some("obsidian-local"),
        )
        .unwrap();
        assert_eq!(v2.capability_version(), 2);
        assert_ne!(v1.manifest_digest(), v2.manifest_digest());
    }

    // -- provider unavailable --

    #[test]
    fn available_provider_resolves_unavailable_does_not() {
        let store = admitted_store();

        // obsidian-local is available → resolves.
        let availability = obsidian_available();
        assert!(resolve_capability(
            &store,
            &availability,
            "notes.note.read",
            1,
            Some("obsidian-local"),
        )
        .is_ok());

        // Empty availability → provider not available.
        let empty = ProviderAvailability::empty();
        let err = resolve_capability(&store, &empty, "notes.note.read", 1, Some("obsidian-local"))
            .unwrap_err();

        match err {
            ResolutionError::ProviderUnavailable {
                capability_name,
                capability_version,
                provider_identity,
            } => {
                assert_eq!(capability_name, "notes.note.read");
                assert_eq!(capability_version, 1);
                assert_eq!(provider_identity, "obsidian-local");
            }
            other => panic!("expected ProviderUnavailable, got {:?}", other),
        }
    }

    #[test]
    fn different_provider_available_does_not_help() {
        // Admitted manifest belongs to obsidian-local.  If only
        // "other-provider" is live, obsidian-local is unavailable.
        let store = admitted_store();
        let availability = ProviderAvailability::from_identities(["other-provider"]);

        let err = resolve_capability(
            &store,
            &availability,
            "notes.note.read",
            1,
            Some("obsidian-local"),
        )
        .unwrap_err();

        match err {
            ResolutionError::ProviderUnavailable { .. } => {}
            other => panic!("expected ProviderUnavailable, got {:?}", other),
        }
    }

    // -- provider identity mismatch --

    #[test]
    fn provider_identity_mismatch() {
        // Verified manifest says "obsidian-local".  Caller expects
        // "notes-provider".  Resolution must flag the mismatch.
        let store = admitted_store();
        let availability = obsidian_available();

        let err = resolve_capability(
            &store,
            &availability,
            "notes.note.read",
            1,
            Some("notes-provider"),
        )
        .unwrap_err();

        match err {
            ResolutionError::ProviderIdentityMismatch {
                capability_name,
                capability_version,
                expected_provider,
                actual_provider,
            } => {
                assert_eq!(capability_name, "notes.note.read");
                assert_eq!(capability_version, 1);
                assert_eq!(expected_provider, "notes-provider");
                assert_eq!(actual_provider, "obsidian-local");
            }
            other => panic!("expected ProviderIdentityMismatch, got {:?}", other),
        }
    }

    // -- resolution does not mutate the store --

    #[test]
    fn resolution_is_read_only() {
        let store = admitted_store();
        let availability = obsidian_available();
        let len_before = store.len();

        let _ = resolve_capability(
            &store,
            &availability,
            "notes.note.read",
            1,
            Some("obsidian-local"),
        )
        .unwrap();

        let _ = resolve_capability(
            &store,
            &availability,
            "notes.note.read",
            2,
            Some("obsidian-local"),
        );

        assert_eq!(store.len(), len_before);
        assert!(store.get_by_name_version("notes.note.read", 1).is_some());
    }

    // -- repeated resolution is stable --

    #[test]
    fn repeated_resolution_is_stable() {
        let store = admitted_store();
        let availability = obsidian_available();

        let r1 = resolve_capability(
            &store,
            &availability,
            "notes.note.read",
            1,
            Some("obsidian-local"),
        )
        .unwrap();

        let r2 = resolve_capability(
            &store,
            &availability,
            "notes.note.read",
            1,
            Some("obsidian-local"),
        )
        .unwrap();

        assert_eq!(r1.manifest_digest(), r2.manifest_digest());
        assert_eq!(r1.provider_identity(), r2.provider_identity());
    }

    // -- live but unadmitted capability does not resolve --

    #[test]
    fn live_provider_without_admitted_manifest_does_not_resolve() {
        // Provider is live, but no manifest was ever admitted for this
        // capability.
        let mut store = TrustedManifestStore::new();
        store.insert(verified_read()).unwrap(); // notes.note.read@1 admitted.
        let availability = obsidian_available();

        // notes.note.delete was never admitted.
        let err = resolve_capability(
            &store,
            &availability,
            "notes.note.delete",
            1,
            Some("obsidian-local"),
        )
        .unwrap_err();

        match err {
            ResolutionError::NoAdmittedManifest {
                capability_name,
                capability_version,
            } => {
                assert_eq!(capability_name, "notes.note.delete");
                assert_eq!(capability_version, 1);
            }
            other => panic!("expected NoAdmittedManifest, got {:?}", other),
        }
    }

    // -- ProviderAvailability is explicit and empty-able --

    #[test]
    fn empty_availability_rejects_all() {
        let store = admitted_store();
        let availability = ProviderAvailability::empty();

        let err = resolve_capability(
            &store,
            &availability,
            "notes.note.read",
            1,
            Some("obsidian-local"),
        )
        .unwrap_err();

        assert!(matches!(err, ResolutionError::ProviderUnavailable { .. }));
    }

    #[test]
    fn availability_from_empty_iterator_is_empty() {
        let availability = ProviderAvailability::from_identities(Vec::<String>::new());
        assert!(availability.is_empty());
        assert_eq!(availability.len(), 0);
    }

    #[test]
    fn availability_contains_expected_identities() {
        let availability =
            ProviderAvailability::from_identities(["obsidian-local", "notes-provider"]);
        assert_eq!(availability.len(), 2);
        assert!(availability.is_available("obsidian-local"));
        assert!(availability.is_available("notes-provider"));
        assert!(!availability.is_available("unknown-provider"));
    }

    // -- projection: admitted + available → projected --

    #[test]
    fn declared_requirement_projects_when_admitted_and_available() {
        let store = admitted_store();
        let availability = obsidian_available();
        let requirements = vec![("notes.note.read".to_owned(), 1u32)];

        let projection = project_capabilities(&requirements, &store, &availability);

        assert_eq!(projection.len(), 1);
        assert_eq!(projection[0].capability_name, "notes.note.read");
        assert_eq!(projection[0].capability_version, 1);
        assert_eq!(projection[0].required_effects, vec!["filesystem.read"]);
        assert_eq!(
            projection[0].manifest_digest,
            verified_read().verified_digest()
        );
    }

    #[test]
    fn multiple_declared_requirements_all_project() {
        let store = admitted_store();
        let availability = obsidian_available();
        let requirements = vec![
            ("notes.note.read".to_owned(), 1u32),
            ("notes.note.create".to_owned(), 1u32),
        ];

        let projection = project_capabilities(&requirements, &store, &availability);

        assert_eq!(projection.len(), 2);
        assert_eq!(projection[0].capability_name, "notes.note.read");
        assert_eq!(projection[0].required_effects, vec!["filesystem.read"]);
        assert_eq!(projection[1].capability_name, "notes.note.create");
        assert_eq!(projection[1].required_effects, vec!["filesystem.write"]);
    }

    // -- projection: missing admission → omitted --

    #[test]
    fn missing_admission_omitted_from_projection() {
        let store = admitted_store();
        let availability = obsidian_available();
        let requirements = vec![
            ("notes.note.read".to_owned(), 1u32),
            ("notes.note.delete".to_owned(), 1u32), // never admitted
        ];

        let projection = project_capabilities(&requirements, &store, &availability);

        assert_eq!(projection.len(), 1);
        assert_eq!(projection[0].capability_name, "notes.note.read");
    }

    // -- projection: version mismatch → omitted --

    #[test]
    fn exact_version_mismatch_omitted_from_projection() {
        let store = admitted_store(); // has notes.note.read@1 only
        let availability = obsidian_available();
        let requirements = vec![
            ("notes.note.read".to_owned(), 2u32), // version 2 not admitted
        ];

        let projection = project_capabilities(&requirements, &store, &availability);

        assert!(projection.is_empty());
    }

    // -- projection: unavailable provider → omitted --

    #[test]
    fn unavailable_provider_omitted_from_projection() {
        let store = admitted_store();
        // Provider not in availability snapshot.
        let availability = ProviderAvailability::empty();
        let requirements = vec![("notes.note.read".to_owned(), 1u32)];

        let projection = project_capabilities(&requirements, &store, &availability);

        assert!(projection.is_empty());
    }

    // -- projection: unavailable provider identity → omitted --

    #[test]
    fn unavailable_provider_identity_omitted_from_projection() {
        // Admitted manifest belongs to obsidian-local.  If the
        // availability snapshot contains a different provider identity,
        // obsidian-local is unavailable and the requirement is omitted.
        let store = admitted_store();
        let availability = ProviderAvailability::from_identities(["other-provider"]);
        let requirements = vec![("notes.note.read".to_owned(), 1u32)];

        let projection = project_capabilities(&requirements, &store, &availability);

        // obsidian-local is not in the availability set → omitted.
        assert!(projection.is_empty());
    }

    // -- projection: projection is read-only --

    #[test]
    fn projection_is_read_only() {
        let store = admitted_store();
        let availability = obsidian_available();
        let len_before = store.len();

        let requirements = vec![("notes.note.read".to_owned(), 1u32)];
        let _projection = project_capabilities(&requirements, &store, &availability);

        assert_eq!(store.len(), len_before);
        assert!(store.get_by_name_version("notes.note.read", 1).is_some());
    }

    // -- projection: deterministic repeat --

    #[test]
    fn projection_is_deterministic() {
        let store = admitted_store();
        let availability = obsidian_available();
        let requirements = vec![
            ("notes.note.read".to_owned(), 1u32),
            ("notes.note.create".to_owned(), 1u32),
        ];

        let p1 = project_capabilities(&requirements, &store, &availability);
        let p2 = project_capabilities(&requirements, &store, &availability);

        assert_eq!(p1, p2);
    }

    // -- projection: empty requirements → empty projection --

    #[test]
    fn empty_requirements_produces_empty_projection() {
        let store = admitted_store();
        let availability = obsidian_available();

        let projection = project_capabilities(&[], &store, &availability);

        assert!(projection.is_empty());
    }

    // -- projection: non-empty requirements but all fail → empty --

    #[test]
    fn all_requirements_fail_produces_empty_projection() {
        let store = TrustedManifestStore::new(); // empty store
        let availability = ProviderAvailability::empty();
        let requirements = vec![
            ("notes.note.read".to_owned(), 1u32),
            ("notes.note.create".to_owned(), 1u32),
        ];

        let projection = project_capabilities(&requirements, &store, &availability);

        assert!(projection.is_empty());
    }
}
