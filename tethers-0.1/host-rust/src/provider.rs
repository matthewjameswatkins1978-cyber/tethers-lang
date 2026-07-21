// Columbo — Provider configuration and admission
//
// Binds a locally configured provider identity to a set of allowed
// capability names with optional pinned digests.  Admits verified
// manifests through the Trusted Manifest Store only after checking
// that the manifest matches local configuration.
//
// This connects "configured local provider binding" to "verified
// manifest → trusted store admission" per the joint canonical
// architecture §4.3.

use crate::manifest::VerifiedManifest;
use crate::trusted_store::{InsertOutcome, ManifestStoreError, TrustedManifestStore};

// ---------------------------------------------------------------------------
// Provider configuration
// ---------------------------------------------------------------------------

/// Capability-level configuration for one provider capability.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AllowedCapability {
    /// Exact capability name the provider is permitted to present.
    pub capability_name: String,
    /// Optional locally pinned digest.  When present, only a manifest
    /// whose verified digest matches this exact value is admitted.
    pub pinned_digest: Option<String>,
}

/// A host-configured provider binding.
///
/// Binds a host-assigned provider identity to a display name and the
/// exact set of capability names the provider is permitted to expose.
/// Each capability may carry an optional pinned digest for defence in
/// depth against manifest changes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderConfig {
    /// Host-assigned identity.  Must match `manifest.provider.identity`.
    pub identity: String,
    /// Human-readable name for diagnostics and Trails.
    pub display_name: String,
    /// Capabilities this provider is permitted to expose, with optional
    /// pinned digests.
    pub allowed_capabilities: Vec<AllowedCapability>,
}

// ---------------------------------------------------------------------------
// Admission outcome
// ---------------------------------------------------------------------------

/// Result of attempting to admit a verified manifest under a provider
/// configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdmissionError {
    /// The manifest's `provider.identity` does not match the configured
    /// identity.
    IdentityMismatch {
        configured_identity: String,
        manifest_identity: String,
    },
    /// The manifest's capability name is not in the configured allowed
    /// capability list.
    CapabilityNotAllowed {
        capability_name: String,
        configured_identity: String,
    },
    /// The configured provider pins a digest for this capability, and
    /// the manifest's verified digest does not match the pin.
    PinnedDigestMismatch {
        capability_name: String,
        pinned_digest: String,
        verified_digest: String,
    },
    /// The configuration has an entry for this capability name but it
    /// lacks a required pinned digest.  (When a pin is expected but
    /// absent, the configuration must be explicit.)
    DigestPinRequired {
        capability_name: String,
    },
    /// Admission to the trusted store failed (identity conflict, digest
    /// conflict).
    StoreError(ManifestStoreError),
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Admit a verified manifest through a configured provider binding.
///
/// Performs (in order):
/// 1. Identity check — manifest's `provider.identity` must match the
///    configured identity.
/// 2. Capability allow-list check — the manifest's `capability_name`
///    must appear in the configured allowed list.
/// 3. Pinned digest check — if the configured capability carries a
///    `pinned_digest`, the manifest's verified digest must match it
///    exactly.
/// 4. Trusted-store insertion — atomic insert with existing conflict
///    and idempotency semantics.
///
/// Returns `InsertOutcome` on success (with `AlreadyPresent`
/// distinguishing a no-op reinsertion).
pub fn admit_provider_manifest(
    config: &ProviderConfig,
    verified: VerifiedManifest,
    store: &mut TrustedManifestStore,
) -> Result<InsertOutcome, AdmissionError> {
    let manifest = verified.manifest();

    // -- 1. identity check --
    if manifest.provider.identity != config.identity {
        return Err(AdmissionError::IdentityMismatch {
            configured_identity: config.identity.clone(),
            manifest_identity: manifest.provider.identity.clone(),
        });
    }

    // -- 2. capability allow-list check --
    let allowed = config
        .allowed_capabilities
        .iter()
        .find(|c| c.capability_name == manifest.capability_name)
        .ok_or_else(|| AdmissionError::CapabilityNotAllowed {
            capability_name: manifest.capability_name.clone(),
            configured_identity: config.identity.clone(),
        })?;

    // -- 3. pinned digest check --
    if let Some(pinned) = &allowed.pinned_digest {
        if pinned != verified.verified_digest() {
            return Err(AdmissionError::PinnedDigestMismatch {
                capability_name: manifest.capability_name.clone(),
                pinned_digest: pinned.clone(),
                verified_digest: verified.verified_digest().to_owned(),
            });
        }
    }

    // -- 4. trusted-store insertion --
    store.insert(verified).map_err(AdmissionError::StoreError)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::verify_manifest;
    use serde_json::json;

    // -- helpers --

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
        let (_, digest) =
            crate::manifest::canonicalize_and_digest(&m.to_string()).unwrap();
        m["digest"] = json!(digest);
        verify_manifest(&m.to_string()).unwrap()
    }

    fn verified_write() -> VerifiedManifest {
        let mut m = write_manifest_json();
        let (_, digest) =
            crate::manifest::canonicalize_and_digest(&m.to_string()).unwrap();
        m["digest"] = json!(digest);
        verify_manifest(&m.to_string()).unwrap()
    }

    fn obsidian_config() -> ProviderConfig {
        ProviderConfig {
            identity: "obsidian-local".to_string(),
            display_name: "Obsidian (local vault)".to_string(),
            allowed_capabilities: vec![
                AllowedCapability {
                    capability_name: "notes.note.read".to_string(),
                    pinned_digest: None,
                },
                AllowedCapability {
                    capability_name: "notes.note.create".to_string(),
                    pinned_digest: None,
                },
            ],
        }
    }

    // -- successful admission --

    #[test]
    fn admit_read_manifest_succeeds() {
        let config = obsidian_config();
        let mut store = TrustedManifestStore::new();
        let v = verified_read();

        let outcome = admit_provider_manifest(&config, v, &mut store).unwrap();
        assert_eq!(outcome, InsertOutcome::Inserted);
        assert_eq!(store.len(), 1);
        assert!(store
            .get_by_name_version("notes.note.read", 1)
            .is_some());
    }

    #[test]
    fn admit_write_manifest_succeeds() {
        let config = obsidian_config();
        let mut store = TrustedManifestStore::new();
        let v = verified_write();

        let outcome = admit_provider_manifest(&config, v, &mut store).unwrap();
        assert_eq!(outcome, InsertOutcome::Inserted);
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn admit_both_capabilities_succeeds() {
        let config = obsidian_config();
        let mut store = TrustedManifestStore::new();

        admit_provider_manifest(&config, verified_read(), &mut store).unwrap();
        admit_provider_manifest(&config, verified_write(), &mut store).unwrap();

        assert_eq!(store.len(), 2);
    }

    #[test]
    fn idempotent_readmission_returns_already_present() {
        let config = obsidian_config();
        let mut store = TrustedManifestStore::new();

        let outcome1 = admit_provider_manifest(&config, verified_read(), &mut store).unwrap();
        assert_eq!(outcome1, InsertOutcome::Inserted);

        let outcome2 = admit_provider_manifest(&config, verified_read(), &mut store).unwrap();
        assert_eq!(outcome2, InsertOutcome::AlreadyPresent);
        assert_eq!(store.len(), 1);
    }

    // -- identity mismatch --

    #[test]
    fn reject_manifest_with_wrong_provider_identity() {
        let config = obsidian_config();
        let mut store = TrustedManifestStore::new();

        // Build a manifest with a different provider identity.
        let mut m = read_manifest_json();
        m["provider"]["identity"] = json!("other-provider");
        let (_, digest) =
            crate::manifest::canonicalize_and_digest(&m.to_string()).unwrap();
        m["digest"] = json!(digest);
        let v = verify_manifest(&m.to_string()).unwrap();

        let err = admit_provider_manifest(&config, v, &mut store).unwrap_err();
        match err {
            AdmissionError::IdentityMismatch {
                configured_identity,
                manifest_identity,
            } => {
                assert_eq!(configured_identity, "obsidian-local");
                assert_eq!(manifest_identity, "other-provider");
            }
            _ => panic!("expected IdentityMismatch, got {:?}", err),
        }
        assert!(store.is_empty());
    }

    // -- capability not allowed --

    #[test]
    fn reject_manifest_for_capability_not_in_allow_list() {
        // Config allows only "notes.note.read" and "notes.note.create".
        // Build a manifest for a different capability.
        let mut m = read_manifest_json();
        m["capability_name"] = json!("notes.note.delete");
        m["binding"]["tool_name"] = json!("obsidian_delete_note");
        // Effects must stay read-only to keep semantics valid.
        let (_, digest) =
            crate::manifest::canonicalize_and_digest(&m.to_string()).unwrap();
        m["digest"] = json!(digest);
        let v = verify_manifest(&m.to_string()).unwrap();

        let config = obsidian_config();
        let mut store = TrustedManifestStore::new();

        let err = admit_provider_manifest(&config, v, &mut store).unwrap_err();
        match err {
            AdmissionError::CapabilityNotAllowed {
                capability_name,
                configured_identity,
            } => {
                assert_eq!(capability_name, "notes.note.delete");
                assert_eq!(configured_identity, "obsidian-local");
            }
            _ => panic!("expected CapabilityNotAllowed, got {:?}", err),
        }
        assert!(store.is_empty());
    }

    // -- pinned digest match --

    #[test]
    fn pinned_digest_match_admits() {
        let v = verified_read();
        let correct_digest = v.verified_digest().to_owned();

        let config = ProviderConfig {
            identity: "obsidian-local".to_string(),
            display_name: "Obsidian (local vault)".to_string(),
            allowed_capabilities: vec![AllowedCapability {
                capability_name: "notes.note.read".to_string(),
                pinned_digest: Some(correct_digest),
            }],
        };
        let mut store = TrustedManifestStore::new();

        let outcome = admit_provider_manifest(&config, v, &mut store).unwrap();
        assert_eq!(outcome, InsertOutcome::Inserted);
    }

    #[test]
    fn pinned_digest_mismatch_rejects() {
        let v = verified_read();
        let correct_digest = v.verified_digest().to_owned();

        let config = ProviderConfig {
            identity: "obsidian-local".to_string(),
            display_name: "Obsidian (local vault)".to_string(),
            allowed_capabilities: vec![AllowedCapability {
                capability_name: "notes.note.read".to_string(),
                pinned_digest: Some("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string()),
            }],
        };
        let mut store = TrustedManifestStore::new();

        let err = admit_provider_manifest(&config, v, &mut store).unwrap_err();
        match err {
            AdmissionError::PinnedDigestMismatch {
                capability_name,
                pinned_digest,
                verified_digest,
            } => {
                assert_eq!(capability_name, "notes.note.read");
                assert_ne!(pinned_digest, verified_digest);
                assert_eq!(verified_digest, correct_digest);
            }
            _ => panic!("expected PinnedDigestMismatch, got {:?}", err),
        }
        assert!(store.is_empty());
    }

    // -- identity conflict on readmission with different content --

    #[test]
    fn identity_conflict_through_admission_preserves_store() {
        let config = obsidian_config();
        let mut store = TrustedManifestStore::new();

        // Admit the canonical read manifest.
        admit_provider_manifest(&config, verified_read(), &mut store).unwrap();

        // Build a different read manifest with same identity.
        let mut m = read_manifest_json();
        m["effects"] = json!(["filesystem.read", "network.access"]);
        m["retry_policy"] = json!({
            "max_retries": 0,
            "backoff_ms": 500,
            "allowed_on": ["outcome_unknown"],
            "requires_idempotency_proof": false
        });
        let (_, digest) =
            crate::manifest::canonicalize_and_digest(&m.to_string()).unwrap();
        m["digest"] = json!(digest);
        let v2 = verify_manifest(&m.to_string()).unwrap();

        let err = admit_provider_manifest(&config, v2, &mut store).unwrap_err();
        match err {
            AdmissionError::StoreError(ManifestStoreError::IdentityConflict { .. }) => {}
            _ => panic!("expected StoreError::IdentityConflict, got {:?}", err),
        }
        assert_eq!(store.len(), 1);
    }

    // -- no capabilities configured --

    #[test]
    fn empty_allow_list_rejects_everything() {
        let config = ProviderConfig {
            identity: "obsidian-local".to_string(),
            display_name: "Obsidian (local vault)".to_string(),
            allowed_capabilities: vec![],
        };
        let mut store = TrustedManifestStore::new();
        let v = verified_read();

        let err = admit_provider_manifest(&config, v, &mut store).unwrap_err();
        match err {
            AdmissionError::CapabilityNotAllowed { .. } => {}
            _ => panic!("expected CapabilityNotAllowed, got {:?}", err),
        }
    }

    // -- identity check is case-sensitive --

    #[test]
    fn identity_comparison_is_case_sensitive() {
        let config = ProviderConfig {
            identity: "Obsidian-Local".to_string(),
            display_name: "Obsidian".to_string(),
            allowed_capabilities: vec![AllowedCapability {
                capability_name: "notes.note.read".to_string(),
                pinned_digest: None,
            }],
        };
        let mut store = TrustedManifestStore::new();
        let v = verified_read(); // identity is "obsidian-local"

        let err = admit_provider_manifest(&config, v, &mut store).unwrap_err();
        match err {
            AdmissionError::IdentityMismatch {
                configured_identity,
                manifest_identity,
            } => {
                assert_eq!(configured_identity, "Obsidian-Local");
                assert_eq!(manifest_identity, "obsidian-local");
            }
            _ => panic!("expected IdentityMismatch, got {:?}", err),
        }
    }
}