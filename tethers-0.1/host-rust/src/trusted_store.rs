// Columbo C2b — Trusted Manifest Store
//
// Stores verified manifests with identity and digest indexes.
// Insertion accepts only VerifiedManifest.
//
// Conflict rules (per docs/CAPABILITY_BRIDGE.md):
// A. Same identity + same digest → AlreadyPresent (idempotent).
// B. Same identity + different digest → IdentityConflict.
// C. Same digest + different identity → DigestConflict.
// D. Same identity + different digest already assigned elsewhere → DigestConflict.
// E. All mutations are atomic: on error, both indexes remain unchanged.

use crate::manifest::VerifiedManifest;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Store types
// ---------------------------------------------------------------------------

/// Identity key for the primary index.
type IdentityKey = (String, u32);

/// A store that admits only `VerifiedManifest` values and indexes them
/// by identity and by digest.
///
/// No public mutation bypass exists. Every insertion goes through
/// `insert`, which preflights both indexes before any mutation.
#[derive(Debug, Clone, Default)]
pub struct TrustedManifestStore {
    /// Primary index: (capability_name, capability_version) → verified manifest.
    entries: HashMap<IdentityKey, VerifiedManifest>,
    /// Secondary index: verified digest → primary identity key.
    digest_index: HashMap<String, IdentityKey>,
}

/// Result of a successful insertion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertOutcome {
    /// The manifest was added to both indexes.
    Inserted,
    /// The same identity and digest were already present; no change.
    AlreadyPresent,
}

/// An error that prevents a manifest from being admitted to the store.
///
/// These are distinct from `ManifestError` (which describes parsing and
/// verification failures). Store admission is a separate boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestStoreError {
    /// Same identity (name, version) with a different digest.
    /// Existing content must not be replaced silently.
    IdentityConflict {
        capability_name: String,
        capability_version: u32,
        existing_digest: String,
        attempted_digest: String,
    },
    /// Same digest with a different identity.
    /// Represents a SHA-256 collision, corrupted state, or implementation
    /// fault.
    DigestConflict {
        digest: String,
        existing_capability_name: String,
        existing_capability_version: u32,
        attempted_capability_name: String,
        attempted_capability_version: u32,
    },
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

impl TrustedManifestStore {
    /// An empty store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of manifests currently stored.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the store contains no manifests.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Retrieve a manifest by exact (capability_name, capability_version).
    pub fn get_by_name_version(&self, name: &str, version: u32) -> Option<&VerifiedManifest> {
        self.entries.get(&(name.to_owned(), version))
    }

    /// Retrieve a manifest by exact verified digest.
    pub fn get_by_digest(&self, digest: &str) -> Option<&VerifiedManifest> {
        let key = self.digest_index.get(digest)?;
        self.entries.get(key)
    }

    /// Admit a `VerifiedManifest` to the store.
    ///
    /// Preflights both indexes before any mutation. On error, neither
    /// index is changed.
    pub fn insert(
        &mut self,
        verified: VerifiedManifest,
    ) -> Result<InsertOutcome, ManifestStoreError> {
        let name = verified.capability_name().to_owned();
        let version = verified.capability_version();
        let digest = verified.verified_digest().to_owned();

        let identity_key: IdentityKey = (name.clone(), version);

        // --- preflight: check both indexes ---

        match (
            self.entries.get(&identity_key),
            self.digest_index.get(&digest),
        ) {
            (None, None) => {
                // Fresh insertion.
                self.digest_index.insert(digest, identity_key.clone());
                self.entries.insert(identity_key, verified);
                Ok(InsertOutcome::Inserted)
            }
            (Some(_existing), Some(existing_key)) if existing_key == &identity_key => {
                // Same identity, same digest — idempotent.
                Ok(InsertOutcome::AlreadyPresent)
            }
            (Some(_existing), Some(existing_key)) => {
                // Same identity with a different attempted digest, and that
                // digest is already assigned to another identity. Classify by
                // digest ownership because the attempted digest cannot be
                // admitted for this identity.
                let (ref existing_name, existing_version) = *existing_key;
                Err(ManifestStoreError::DigestConflict {
                    digest,
                    existing_capability_name: existing_name.clone(),
                    existing_capability_version: existing_version,
                    attempted_capability_name: name,
                    attempted_capability_version: version,
                })
            }
            (Some(existing), None) => {
                // Same identity, different digest.
                Err(ManifestStoreError::IdentityConflict {
                    capability_name: name,
                    capability_version: version,
                    existing_digest: existing.verified_digest().to_owned(),
                    attempted_digest: digest,
                })
            }
            (None, Some(existing_key)) => {
                // Same digest, different identity.
                let (ref existing_name, existing_version) = *existing_key;
                Err(ManifestStoreError::DigestConflict {
                    digest,
                    existing_capability_name: existing_name.clone(),
                    existing_capability_version: existing_version,
                    attempted_capability_name: name,
                    attempted_capability_version: version,
                })
            }
        }
    }
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

    fn minimal_read_json() -> serde_json::Value {
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

    /// The correct digest for minimal_read_json(), computed once.
    fn read_correct_digest() -> String {
        let m = minimal_read_json();
        crate::manifest::canonicalize_and_digest(&m.to_string())
            .unwrap()
            .1
    }

    fn verified_minimal_read() -> VerifiedManifest {
        let mut m = minimal_read_json();
        let digest = read_correct_digest();
        m["digest"] = json!(&digest);
        verify_manifest(&m.to_string()).unwrap()
    }

    fn minimal_write_json() -> serde_json::Value {
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

    fn verified_minimal_write() -> VerifiedManifest {
        let mut m = minimal_write_json();
        let (_, correct_digest) = crate::manifest::canonicalize_and_digest(&m.to_string()).unwrap();
        m["digest"] = json!(correct_digest);
        verify_manifest(&m.to_string()).unwrap()
    }

    // -- empty store --

    #[test]
    fn store_empty_on_creation() {
        let store = TrustedManifestStore::new();
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);
        assert!(store.get_by_name_version("notes.note.read", 1).is_none());
        assert!(store.get_by_digest("sha256:aaaa").is_none());
    }

    // -- fresh insertion --

    #[test]
    fn fresh_insert_returns_inserted() {
        let mut store = TrustedManifestStore::new();
        let v = verified_minimal_read();
        let outcome = store.insert(v.clone()).unwrap();
        assert_eq!(outcome, InsertOutcome::Inserted);
        assert_eq!(store.len(), 1);
        assert!(!store.is_empty());
    }

    #[test]
    fn fresh_insert_retrievable_by_identity() {
        let mut store = TrustedManifestStore::new();
        let v = verified_minimal_read();
        store.insert(v.clone()).unwrap();

        let retrieved = store.get_by_name_version("notes.note.read", 1).unwrap();
        assert_eq!(retrieved.capability_name(), "notes.note.read");
        assert_eq!(retrieved.capability_version(), 1);
        let expected_digest = read_correct_digest();
        assert_eq!(retrieved.verified_digest(), expected_digest);
    }

    #[test]
    fn fresh_insert_retrievable_by_digest() {
        let mut store = TrustedManifestStore::new();
        let v = verified_minimal_read();
        let digest = v.verified_digest().to_owned();
        store.insert(v).unwrap();

        let retrieved = store.get_by_digest(&digest).unwrap();
        assert_eq!(retrieved.capability_name(), "notes.note.read");
    }

    #[test]
    fn both_retrieval_paths_agree() {
        let mut store = TrustedManifestStore::new();
        let v = verified_minimal_read();
        let digest = v.verified_digest().to_owned();
        store.insert(v).unwrap();

        // Retrieve the manifest through both paths.
        let by_name = store.get_by_name_version("notes.note.read", 1);
        let by_digest = store.get_by_digest(&digest);
        // Compare identity fields.
        assert_eq!(
            by_name.unwrap().capability_name(),
            by_digest.unwrap().capability_name()
        );
    }

    // -- idempotent reinsertion --

    #[test]
    fn reinsert_same_identity_same_digest_returns_already_present() {
        let mut store = TrustedManifestStore::new();
        let v1 = verified_minimal_read();
        store.insert(v1).unwrap();

        // Independently verify a second copy.
        let v2 = verified_minimal_read();
        let outcome = store.insert(v2).unwrap();
        assert_eq!(outcome, InsertOutcome::AlreadyPresent);
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn reinsertion_preserves_original_data() {
        let mut store = TrustedManifestStore::new();
        let v1 = verified_minimal_read();
        store.insert(v1).unwrap();

        // Reinsert — the stored manifest is unchanged.
        let v2 = verified_minimal_read();
        store.insert(v2).unwrap();

        let retrieved = store.get_by_name_version("notes.note.read", 1).unwrap();
        assert_eq!(retrieved.capability_name(), "notes.note.read");
    }

    #[test]
    fn reinsertion_keeps_both_indexes_consistent() {
        let mut store = TrustedManifestStore::new();
        let v = verified_minimal_read();
        let digest = v.verified_digest().to_owned();
        store.insert(v).unwrap();

        // Reinsert — the digest index must still point to the same entry.
        let v2 = verified_minimal_read();
        store.insert(v2).unwrap();

        assert!(store.get_by_digest(&digest).is_some());
        assert_eq!(store.len(), 1);
    }

    // -- identity conflict --

    #[test]
    fn identity_conflict_rejected() {
        let mut store = TrustedManifestStore::new();
        let v1 = verified_minimal_read();
        let existing_digest = v1.verified_digest().to_owned();
        store.insert(v1).unwrap();

        // Same identity with a valid but different digest.
        // Change a covered field so the correct digest differs.
        let mut m = minimal_read_json();
        m["effects"] = json!(["filesystem.read", "network.access"]);
        m["retry_policy"] = json!({
            "max_retries": 0,
            "backoff_ms": 500,
            "allowed_on": ["outcome_unknown"],
            "requires_idempotency_proof": false
        });
        // Compute the correct digest for this modified manifest.
        let (_, correct_digest) = crate::manifest::canonicalize_and_digest(&m.to_string()).unwrap();
        m["digest"] = json!(&correct_digest);
        let v2 = verify_manifest(&m.to_string()).unwrap();
        let attempted_digest = v2.verified_digest().to_owned();

        let err = store.insert(v2).unwrap_err();

        match err {
            ManifestStoreError::IdentityConflict {
                capability_name,
                capability_version,
                existing_digest: err_existing,
                attempted_digest: err_attempted,
            } => {
                assert_eq!(capability_name, "notes.note.read");
                assert_eq!(capability_version, 1);
                assert_eq!(err_existing, existing_digest);
                assert_eq!(err_attempted, attempted_digest);
                assert_ne!(existing_digest, attempted_digest);
            }
            _ => panic!("expected IdentityConflict"),
        }
    }

    #[test]
    fn identity_conflict_preserves_store() {
        let mut store = TrustedManifestStore::new();
        let v1 = verified_minimal_read();
        let orig_digest = v1.verified_digest().to_owned();
        store.insert(v1).unwrap();

        // Attempt conflicting insertion — same identity, different valid content.
        let mut m = minimal_read_json();
        m["effects"] = json!(["filesystem.read", "network.access"]);
        m["retry_policy"] = json!({
            "max_retries": 0,
            "backoff_ms": 500,
            "allowed_on": ["outcome_unknown"],
            "requires_idempotency_proof": false
        });
        let (_, correct_digest) = crate::manifest::canonicalize_and_digest(&m.to_string()).unwrap();
        m["digest"] = json!(&correct_digest);
        let conflicting_digest = correct_digest.clone();
        let _ = store.insert(verify_manifest(&m.to_string()).unwrap());

        // Store unchanged.
        assert_eq!(store.len(), 1);
        let retrieved = store.get_by_name_version("notes.note.read", 1).unwrap();
        assert_eq!(retrieved.verified_digest(), orig_digest);
        // Conflicting digest not in index.
        assert!(store.get_by_digest(&conflicting_digest).is_none());
    }

    // -- digest conflict --

    #[test]
    fn digest_conflict_without_attempted_identity_rejected() {
        let mut store = TrustedManifestStore::new();
        let read = verified_minimal_read();
        let read_digest = read.verified_digest().to_owned();
        let write = verified_minimal_write();
        let write_identity = (
            write.capability_name().to_owned(),
            write.capability_version(),
        );
        store.insert(write).unwrap();

        // Simulate the collision preflight state privately: the attempted
        // digest is already assigned to a different identity, while the
        // attempted identity is absent.
        store
            .digest_index
            .insert(read_digest.clone(), write_identity.clone());
        let entries_before = store.entries.len();
        let digest_index_before = store.digest_index.clone();

        let err = store.insert(read).unwrap_err();

        match err {
            ManifestStoreError::DigestConflict {
                digest,
                existing_capability_name,
                existing_capability_version,
                attempted_capability_name,
                attempted_capability_version,
            } => {
                assert_eq!(digest, read_digest);
                assert_eq!(existing_capability_name, "notes.note.create");
                assert_eq!(existing_capability_version, 1);
                assert_eq!(attempted_capability_name, "notes.note.read");
                assert_eq!(attempted_capability_version, 1);
            }
            _ => panic!("expected DigestConflict"),
        }

        assert_eq!(store.entries.len(), entries_before);
        assert_eq!(store.digest_index, digest_index_before);
        assert!(store.entries.get(&write_identity).is_some());
        assert!(store
            .entries
            .get(&("notes.note.read".to_owned(), 1))
            .is_none());
    }

    #[test]
    fn dual_identity_and_digest_conflict_returns_digest_conflict_without_mutation() {
        let mut store = TrustedManifestStore::new();
        let read = verified_minimal_read();
        let read_digest = read.verified_digest().to_owned();
        let write = verified_minimal_write();
        let write_identity = (
            write.capability_name().to_owned(),
            write.capability_version(),
        );

        store.insert(read.clone()).unwrap();
        store.insert(write).unwrap();

        // Simulate the SHA-collision admission state privately: the attempted
        // identity is already occupied, and the attempted digest is already
        // assigned to a different identity.
        store
            .digest_index
            .insert(read_digest.clone(), write_identity.clone());
        let entries_before = store.entries.len();
        let digest_index_before = store.digest_index.clone();

        let err = store.insert(read).unwrap_err();

        match err {
            ManifestStoreError::DigestConflict {
                digest,
                existing_capability_name,
                existing_capability_version,
                attempted_capability_name,
                attempted_capability_version,
            } => {
                assert_eq!(digest, read_digest);
                assert_eq!(existing_capability_name, "notes.note.create");
                assert_eq!(existing_capability_version, 1);
                assert_eq!(attempted_capability_name, "notes.note.read");
                assert_eq!(attempted_capability_version, 1);
            }
            _ => panic!("expected DigestConflict"),
        }

        assert_eq!(store.entries.len(), entries_before);
        assert_eq!(store.digest_index, digest_index_before);
        assert!(store
            .entries
            .get(&("notes.note.read".to_owned(), 1))
            .is_some());
        assert!(store.entries.get(&write_identity).is_some());
    }

    // -- multiple entries --

    #[test]
    fn multiple_entries_coexist() {
        let mut store = TrustedManifestStore::new();
        let read = verified_minimal_read();
        let write = verified_minimal_write();

        store.insert(read).unwrap();
        store.insert(write).unwrap();

        assert_eq!(store.len(), 2);

        let r = store.get_by_name_version("notes.note.read", 1).unwrap();
        assert_eq!(r.capability_name(), "notes.note.read");

        let w = store.get_by_name_version("notes.note.create", 1).unwrap();
        assert_eq!(w.capability_name(), "notes.note.create");
    }

    #[test]
    fn digest_retrieval_works_with_multiple_entries() {
        let mut store = TrustedManifestStore::new();
        let read = verified_minimal_read();
        let read_digest = read.verified_digest().to_owned();
        let write = verified_minimal_write();
        let write_digest = write.verified_digest().to_owned();

        store.insert(read).unwrap();
        store.insert(write).unwrap();

        let by_read_digest = store.get_by_digest(&read_digest).unwrap();
        assert_eq!(by_read_digest.capability_name(), "notes.note.read");

        let by_write_digest = store.get_by_digest(&write_digest).unwrap();
        assert_eq!(by_write_digest.capability_name(), "notes.note.create");
    }

    #[test]
    fn case_change_does_not_silently_match() {
        let mut store = TrustedManifestStore::new();
        store.insert(verified_minimal_read()).unwrap();

        assert!(store.get_by_name_version("Notes.Note.Read", 1).is_none());
        assert!(store
            .get_by_digest(
                "SHA256:AD5C8E3CD5430588CAF083E367A452F01D4F6C5DA6EE7EABDF39C47919B27401"
            )
            .is_none());
    }

    #[test]
    fn unknown_identity_returns_none() {
        let mut store = TrustedManifestStore::new();
        store.insert(verified_minimal_read()).unwrap();

        assert!(store.get_by_name_version("nonexistent.cap", 1).is_none());
        assert!(store
            .get_by_digest(
                "sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
            )
            .is_none());
    }

    // -- atomicity: rejected insertions leave store unchanged --

    #[test]
    fn identity_conflict_does_not_change_len() {
        let mut store = TrustedManifestStore::new();
        store.insert(verified_minimal_read()).unwrap();
        let len_before = store.len();

        // Same identity, different valid content.
        let mut m = minimal_read_json();
        m["effects"] = json!(["filesystem.read", "network.access"]);
        m["retry_policy"] = json!({
            "max_retries": 0,
            "backoff_ms": 500,
            "allowed_on": ["outcome_unknown"],
            "requires_idempotency_proof": false
        });
        let (_, correct_digest) = crate::manifest::canonicalize_and_digest(&m.to_string()).unwrap();
        m["digest"] = json!(&correct_digest);
        let _ = store.insert(verify_manifest(&m.to_string()).unwrap());

        assert_eq!(store.len(), len_before);
    }

    #[test]
    fn identity_conflict_leaves_existing_lookup_intact() {
        let mut store = TrustedManifestStore::new();
        let v1 = verified_minimal_read();
        let orig_digest = v1.verified_digest().to_owned();
        store.insert(v1).unwrap();

        // Same identity, different valid content.
        let mut m = minimal_read_json();
        m["effects"] = json!(["filesystem.read", "network.access"]);
        m["retry_policy"] = json!({
            "max_retries": 0,
            "backoff_ms": 500,
            "allowed_on": ["outcome_unknown"],
            "requires_idempotency_proof": false
        });
        let (_, correct_digest) = crate::manifest::canonicalize_and_digest(&m.to_string()).unwrap();
        m["digest"] = json!(&correct_digest);
        let _ = store.insert(verify_manifest(&m.to_string()).unwrap());

        let retrieved = store.get_by_name_version("notes.note.read", 1).unwrap();
        assert_eq!(retrieved.verified_digest(), orig_digest);
    }

    // -- one-to-one index consistency --

    #[test]
    fn one_primary_entry_per_digest_index_entry() {
        let mut store = TrustedManifestStore::new();
        let v1 = verified_minimal_read();
        let v2 = verified_minimal_write();
        let write_digest = v2.verified_digest().to_owned();
        store.insert(v1).unwrap();
        store.insert(v2).unwrap();

        // For each digest in the digest_index, the corresponding identity key
        // must resolve back to a manifest with that same digest.
        // We verify indirectly through the public API.
        let read_digest = read_correct_digest();
        let by_digest_read = store.get_by_digest(&read_digest).unwrap();
        assert_eq!(by_digest_read.capability_name(), "notes.note.read");
        assert_eq!(by_digest_read.capability_version(), 1);

        let by_digest_write = store.get_by_digest(&write_digest).unwrap();
        assert_eq!(by_digest_write.capability_name(), "notes.note.create");
        assert_eq!(by_digest_write.capability_version(), 1);
    }

    // -- type boundary: no raw TrustedManifest insertion --

    #[test]
    fn insertion_accepts_only_verified_manifest() {
        // This is a compile-time guarantee: the `insert` method accepts
        // `VerifiedManifest`. There is no public method accepting
        // `TrustedManifest`, `serde_json::Value`, `&str`, or raw fields.
        // This test obtains a VerifiedManifest through `verify_manifest`,
        // which is the only construction route.
        let v = verified_minimal_read();
        let mut store = TrustedManifestStore::new();
        assert!(store.insert(v).is_ok());
    }
}
