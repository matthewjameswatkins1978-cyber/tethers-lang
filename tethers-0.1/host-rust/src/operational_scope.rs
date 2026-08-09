//! Generic operational-scope evidence.
//!
//! Carries the plug-declared schema, validated canonical scope, and integrity
//! proof without any plug-subject-specific knowledge.  A new Plug must not
//! require editing this module.

use serde::{Deserialize, Serialize};
use serde_json::Value;

const SCOPE_SCHEMA_VERSION: u32 = 1;
const DIGEST_LEN: usize = 71;

pub(crate) fn is_strict_lowercase_hex_digest(value: &str) -> bool {
    if value.len() != DIGEST_LEN || !value.starts_with("sha256:") {
        return false;
    }
    value[7..]
        .bytes()
        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OperationalScopeEvidence {
    pub schema_version: u32,
    pub installed_identity: String,
    pub package_identity: String,
    pub provider_identity: String,
    pub scope_schema_digest: String,
    pub canonical_scope_json: String,
    pub authority: String,
    pub integrity_digest: String,
}

impl OperationalScopeEvidence {
    pub fn create(
        installed_identity: &str,
        package_identity: &str,
        provider_identity: &str,
        scope_schema_digest: &str,
        scope: &Value,
        authority: &str,
    ) -> Result<Self, crate::m3_store::M3Error> {
        if installed_identity.is_empty()
            || package_identity.is_empty()
            || provider_identity.is_empty()
            || authority.is_empty()
        {
            return Err(crate::m3_store::M3Error::new(
                "scope_invalid",
                "required operational scope fields must not be empty",
            ));
        }
        if !is_strict_lowercase_hex_digest(scope_schema_digest) {
            return Err(crate::m3_store::M3Error::new(
                "scope_invalid",
                "scope_schema_digest must be sha256: hex form",
            ));
        }
        let canonical_bytes = serde_json_canonicalizer::to_vec(scope).map_err(|e| {
            crate::m3_store::M3Error::new("scope_invalid", format!("JCS canonicalisation: {e}"))
        })?;
        let canonical_scope_json = String::from_utf8(canonical_bytes).map_err(|_| {
            crate::m3_store::M3Error::new("scope_invalid", "corrupt canonical scope")
        })?;
        let integrity_digest = {
            use sha2::{Digest, Sha256};
            let copy = Self {
                schema_version: SCOPE_SCHEMA_VERSION,
                installed_identity: installed_identity.to_owned(),
                package_identity: package_identity.to_owned(),
                provider_identity: provider_identity.to_owned(),
                scope_schema_digest: scope_schema_digest.to_owned(),
                canonical_scope_json: canonical_scope_json.clone(),
                authority: authority.to_owned(),
                integrity_digest: String::new(),
            };
            let covered = serde_json_canonicalizer::to_vec(&copy)
                .map_err(|e| crate::m3_store::M3Error::new("scope_invalid", format!("JCS: {e}")))?;
            format!("sha256:{:x}", Sha256::digest(covered))
        };
        let evidence = Self {
            schema_version: SCOPE_SCHEMA_VERSION,
            installed_identity: installed_identity.to_owned(),
            package_identity: package_identity.to_owned(),
            provider_identity: provider_identity.to_owned(),
            scope_schema_digest: scope_schema_digest.to_owned(),
            canonical_scope_json,
            authority: authority.to_owned(),
            integrity_digest,
        };
        evidence.validate()?;
        Ok(evidence)
    }

    pub fn installed_id(&self) -> &str {
        &self.installed_identity
    }

    pub fn integrity_digest(&self) -> &str {
        &self.integrity_digest
    }

    pub fn canonical_scope(&self) -> Result<Value, crate::m3_store::M3Error> {
        serde_json::from_str(&self.canonical_scope_json).map_err(|e| {
            crate::m3_store::M3Error::new("scope_invalid", format!("parse canonical scope: {e}"))
        })
    }

    pub fn validate(&self) -> crate::m3_store::Result<()> {
        use crate::m3_store::M3Error;
        if self.schema_version != SCOPE_SCHEMA_VERSION {
            return Err(M3Error::new(
                "scope_invalid",
                "unsupported scope schema version",
            ));
        }
        if self.installed_identity.is_empty()
            || self.package_identity.is_empty()
            || self.provider_identity.is_empty()
            || self.authority.is_empty()
        {
            return Err(M3Error::new(
                "scope_invalid",
                "required scope fields missing",
            ));
        }
        if !is_strict_lowercase_hex_digest(&self.scope_schema_digest) {
            return Err(M3Error::new(
                "scope_invalid",
                "scope_schema_digest is malformed",
            ));
        }
        if self.canonical_scope_json.is_empty() || self.integrity_digest.len() != DIGEST_LEN {
            return Err(M3Error::new(
                "scope_invalid",
                "canonical scope or digest is incomplete",
            ));
        }
        // Re-verify tamper evidence: recompute digest from covered fields and compare.
        let mut copy = self.clone();
        let stored_digest = copy.integrity_digest.clone();
        copy.integrity_digest = String::new();
        let covered = serde_json_canonicalizer::to_vec(&copy)
            .map_err(|_| M3Error::new("scope_invalid", "scope JCS failure"))?;
        use sha2::{Digest, Sha256};
        let recomputed = format!("sha256:{:x}", Sha256::digest(covered));
        if stored_digest != recomputed {
            return Err(M3Error::new(
                "scope_invalid",
                "scope integrity evidence tampered",
            ));
        }
        // Validate canonical_scope_json round-trips through JCS unchanged.
        let parsed: Value = serde_json::from_str(&self.canonical_scope_json)
            .map_err(|_| M3Error::new("scope_invalid", "canonical scope is not valid JSON"))?;
        let re_canonical = serde_json_canonicalizer::to_vec(&parsed)
            .map_err(|_| M3Error::new("scope_invalid", "scope re-canonicalisation failed"))?;
        if re_canonical.as_slice() != self.canonical_scope_json.as_bytes() {
            return Err(M3Error::new(
                "scope_invalid",
                "canonical scope round-trip mismatch",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_and_validates_minimal_scope() {
        let scope = serde_json::json!({"key": "value"});
        let evidence = OperationalScopeEvidence::create(
            "00000000-0000-0000-0000-000000000001",
            "example.package",
            "example-provider",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            &scope,
            "test-authority",
        )
        .unwrap();
        assert_eq!(evidence.schema_version, 1);
        assert_eq!(
            evidence.installed_id(),
            "00000000-0000-0000-0000-000000000001"
        );
        assert_eq!(evidence.package_identity, "example.package");
        assert_eq!(evidence.provider_identity, "example-provider");
        assert!(evidence.integrity_digest.starts_with("sha256:"));
        evidence.validate().unwrap();
    }

    #[test]
    fn rejects_empty_fields() {
        let scope = serde_json::json!({"k": "v"});
        assert!(OperationalScopeEvidence::create(
            "",
            "pkg",
            "prv",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            &scope,
            "auth",
        )
        .is_err());
        assert!(OperationalScopeEvidence::create(
            "id",
            "",
            "prv",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            &scope,
            "auth",
        )
        .is_err());
        assert!(
            OperationalScopeEvidence::create("id", "pkg", "prv", "short", &scope, "auth",).is_err()
        );
    }

    #[test]
    fn tampered_digest_detected_by_validate() {
        let scope = serde_json::json!({"k": "v"});
        let mut evidence = OperationalScopeEvidence::create(
            "00000000-0000-0000-0000-000000000001",
            "example.package",
            "example-provider",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            &scope,
            "test-authority",
        )
        .unwrap();
        evidence.integrity_digest =
            "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string();
        assert!(evidence.validate().is_err());
    }

    #[test]
    fn tampered_json_detected_by_validate() {
        let scope = serde_json::json!({"k": "v"});
        let mut evidence = OperationalScopeEvidence::create(
            "00000000-0000-0000-0000-000000000001",
            "example.package",
            "example-provider",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            &scope,
            "test-authority",
        )
        .unwrap();
        evidence.canonical_scope_json.push('x');
        assert!(evidence.validate().is_err());
    }

    #[test]
    fn digest_deterministic_across_calls() {
        let scope = serde_json::json!({"key": "value"});
        let a = OperationalScopeEvidence::create(
            "00000000-0000-0000-0000-000000000001",
            "example.package",
            "example-provider",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            &scope,
            "test-authority",
        )
        .unwrap();
        let b = OperationalScopeEvidence::create(
            "00000000-0000-0000-0000-000000000001",
            "example.package",
            "example-provider",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            &scope,
            "test-authority",
        )
        .unwrap();
        assert_eq!(a.integrity_digest, b.integrity_digest);
        assert_eq!(a.canonical_scope_json, b.canonical_scope_json);
    }

    #[test]
    fn different_scope_yields_different_digest() {
        let s1 = serde_json::json!({"a": 1});
        let s2 = serde_json::json!({"a": 2});
        let a = OperationalScopeEvidence::create(
            "00000000-0000-0000-0000-000000000001",
            "example.package",
            "example-provider",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            &s1,
            "test-authority",
        )
        .unwrap();
        let b = OperationalScopeEvidence::create(
            "00000000-0000-0000-0000-000000000001",
            "example.package",
            "example-provider",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            &s2,
            "test-authority",
        )
        .unwrap();
        assert_ne!(a.integrity_digest, b.integrity_digest);
    }
}
