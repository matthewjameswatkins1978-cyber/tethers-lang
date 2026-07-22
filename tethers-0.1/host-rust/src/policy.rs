// Columbo — Effective capability permission resolution
//
// Combines Tether Set capability declarations, live admitted capability
// resolution, and host-local policy into one of four effective
// decisions: allow, ask, deny, or unavailable.
//
// A capability is usable only when:
// - the Tether Set declares it as a required capability (exact name + version);
// - it resolves as admitted, verified, and currently available;
// - host-local policy permits it.
//
// Missing any of the three → not permitted (deny for undeclared,
// unavailable for no current provider, deny for explicit prohibition).

use crate::resolver::{self, ProviderAvailability, ResolvedCapability};
use crate::trusted_store::TrustedManifestStore;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Capability requirement — what a Tether Set declares
// ---------------------------------------------------------------------------

/// One capability a Tether Set declares it requires at an exact version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityRequirement {
    /// Exact capability name.
    pub capability_name: String,
    /// Exact capability version.
    pub capability_version: u32,
    /// Optional human-readable reason the set needs this capability.
    pub reason: Option<String>,
}

impl CapabilityRequirement {
    pub fn new(name: impl Into<String>, version: u32) -> Self {
        Self {
            capability_name: name.into(),
            capability_version: version,
            reason: None,
        }
    }

    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }
}

// ---------------------------------------------------------------------------
// Host-local policy
// ---------------------------------------------------------------------------

/// A per-capability host-local policy rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyRule {
    /// Pre-authorised — dispatch may proceed without per-call confirmation.
    Allow,
    /// Per-call confirmation required before dispatch.
    Ask,
    /// Explicitly prohibited — must not dispatch.
    Deny,
}

/// Host-local policy: default posture plus per-capability overrides.
///
/// Matches by exact capability name (not version — the version is
/// governed by the Tether Set declaration and admission, not by
/// host policy).  Overrides take precedence over the default.
///
/// This is host-controlled, in-memory policy.  It does not grant
/// permission to an undeclared or unavailable capability, and does
/// not bypass the trusted manifest store.
#[derive(Debug, Clone)]
pub struct HostLocalPolicy {
    /// Default posture for capabilities not listed in `overrides`.
    default_posture: PolicyRule,
    /// Per-capability overrides keyed by exact capability name.
    overrides: HashMap<String, PolicyRule>,
}

impl HostLocalPolicy {
    /// Create a policy with a default posture and no overrides.
    pub fn new(default_posture: PolicyRule) -> Self {
        Self {
            default_posture,
            overrides: HashMap::new(),
        }
    }

    /// Insert a per-capability override.
    pub fn insert(&mut self, capability_name: impl Into<String>, rule: PolicyRule) {
        self.overrides.insert(capability_name.into(), rule);
    }

    /// What rule applies to a given capability name?
    pub fn rule_for(&self, capability_name: &str) -> PolicyRule {
        self.overrides
            .get(capability_name)
            .copied()
            .unwrap_or(self.default_posture)
    }
}

// ---------------------------------------------------------------------------
// Permission decision
// ---------------------------------------------------------------------------

/// The effective permission outcome for one requested capability.
///
/// These match the canonical architecture §4.7:
///
/// - `Allow` — pre-authorised; dispatch may proceed.
/// - `Ask` — per-call confirmation required; dispatch must pause.
/// - `Deny` — explicitly prohibited; dispatch must not occur.
/// - `Unavailable` — no currently valid provider binding exists
///   for this exact-version capability.
///
/// `Unavailable` is *not* the same as `Deny`.  Deny means the
/// capability is known but disallowed by policy.  Unavailable means
/// the capability cannot be resolved as currently usable — the
/// provider is not reachable, the manifest was never admitted, or
/// the exact version does not exist.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionDecision {
    Allow,
    Ask,
    Deny,
    Unavailable,
}

// ---------------------------------------------------------------------------
// Effective permission evaluation
// ---------------------------------------------------------------------------

/// Evaluate the effective permission for a capability request.
///
/// Combines three inputs:
/// 1. **Tether Set declaration** — does the set require this exact
///    (name, version)?
/// 2. **Live admitted resolution** — is a verified, admitted,
///    currently-available provider binding present?
/// 3. **Host-local policy** — does host policy allow, ask, or deny
///    this capability?
///
/// Precedence:
/// 1. Not declared by the set → `Deny`.  A set cannot request
///    capabilities it did not declare.
/// 2. Not admitted or unavailable → `Unavailable`.  Honest report:
///    the binding cannot currently be obtained.
/// 3. Host policy deny → `Deny`.  An explicit denial overrides
///    everything.
/// 4. Host policy ask → `Ask`.
/// 5. Host policy allow → `Allow`.
///
/// Read-only.  No side effects.  No Trail, dispatch, or Anchor.
pub fn evaluate_permission(
    requirements: &[CapabilityRequirement],
    store: &TrustedManifestStore,
    availability: &ProviderAvailability,
    policy: &HostLocalPolicy,
    capability_name: &str,
    capability_version: u32,
    expected_provider: Option<&str>,
) -> PermissionDecision {
    // 1. Tether Set declaration check.
    let requirement = requirements.iter().find(|r| {
        r.capability_name == capability_name && r.capability_version == capability_version
    });

    if requirement.is_none() {
        return PermissionDecision::Deny;
    }

    // 2. Live admitted resolution.
    match resolver::resolve_capability(
        store,
        availability,
        capability_name,
        capability_version,
        expected_provider,
    ) {
        Ok(_resolved) => {
            // 3. Host-local policy.
            match policy.rule_for(capability_name) {
                PolicyRule::Deny => PermissionDecision::Deny,
                PolicyRule::Ask => PermissionDecision::Ask,
                PolicyRule::Allow => PermissionDecision::Allow,
            }
        }
        Err(_) => PermissionDecision::Unavailable,
    }
}

/// Convenience: evaluate permission with a resolved capability already
/// in hand.  Skips the resolution step but still checks declaration
/// and policy.
pub fn evaluate_permission_resolved(
    requirements: &[CapabilityRequirement],
    resolved: &ResolvedCapability,
    policy: &HostLocalPolicy,
) -> PermissionDecision {
    let capability_name = &resolved.identity.name;
    let capability_version = resolved.identity.version;

    // 1. Tether Set declaration check.
    let requirement = requirements.iter().find(|r| {
        r.capability_name.as_str() == capability_name.as_str()
            && r.capability_version == capability_version
    });

    if requirement.is_none() {
        return PermissionDecision::Deny;
    }

    // 2. Already resolved — skip to policy.
    match policy.rule_for(capability_name) {
        PolicyRule::Deny => PermissionDecision::Deny,
        PolicyRule::Ask => PermissionDecision::Ask,
        PolicyRule::Allow => PermissionDecision::Allow,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{verify_manifest, VerifiedManifest};
    use crate::resolver::ProviderAvailability;
    use crate::trusted_store::TrustedManifestStore;
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

    fn verified_read() -> VerifiedManifest {
        let mut m = read_manifest_json();
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
        store
    }

    fn notes_read_requirement() -> CapabilityRequirement {
        CapabilityRequirement::new("notes.note.read", 1).with_reason("Read notes from the vault")
    }

    fn notes_write_requirement() -> CapabilityRequirement {
        CapabilityRequirement::new("notes.note.create", 1).with_reason("Create notes in the vault")
    }

    fn allow_all_policy() -> HostLocalPolicy {
        HostLocalPolicy::new(PolicyRule::Allow)
    }

    fn ask_all_policy() -> HostLocalPolicy {
        HostLocalPolicy::new(PolicyRule::Ask)
    }

    fn deny_all_policy() -> HostLocalPolicy {
        HostLocalPolicy::new(PolicyRule::Deny)
    }

    // -- declared + live admitted + local allow → allow --

    #[test]
    fn declared_live_admitted_allow_policy_returns_allow() {
        let requirements = vec![notes_read_requirement()];
        let store = admitted_store();
        let availability = obsidian_available();
        let policy = allow_all_policy();

        let decision = evaluate_permission(
            &requirements,
            &store,
            &availability,
            &policy,
            "notes.note.read",
            1,
            Some("obsidian-local"),
        );

        assert_eq!(decision, PermissionDecision::Allow);
    }

    // -- declared + live admitted + local ask → ask --

    #[test]
    fn declared_live_admitted_ask_policy_returns_ask() {
        let requirements = vec![notes_read_requirement()];
        let store = admitted_store();
        let availability = obsidian_available();
        let policy = ask_all_policy();

        let decision = evaluate_permission(
            &requirements,
            &store,
            &availability,
            &policy,
            "notes.note.read",
            1,
            Some("obsidian-local"),
        );

        assert_eq!(decision, PermissionDecision::Ask);
    }

    // -- declared + live admitted + local deny → deny --

    #[test]
    fn declared_live_admitted_deny_policy_returns_deny() {
        let requirements = vec![notes_read_requirement()];
        let store = admitted_store();
        let availability = obsidian_available();
        let policy = deny_all_policy();

        let decision = evaluate_permission(
            &requirements,
            &store,
            &availability,
            &policy,
            "notes.note.read",
            1,
            Some("obsidian-local"),
        );

        assert_eq!(decision, PermissionDecision::Deny);
    }

    // -- deny cannot be overridden by declaration --

    #[test]
    fn deny_cannot_be_overridden_by_declaration() {
        // The set declares the requirement.  The capability is admitted
        // and live.  But host policy says deny — deny wins.
        let requirements = vec![notes_read_requirement()];
        let store = admitted_store();
        let availability = obsidian_available();

        let mut policy = HostLocalPolicy::new(PolicyRule::Allow);
        policy.insert("notes.note.read", PolicyRule::Deny);

        let decision = evaluate_permission(
            &requirements,
            &store,
            &availability,
            &policy,
            "notes.note.read",
            1,
            Some("obsidian-local"),
        );

        assert_eq!(decision, PermissionDecision::Deny);
    }

    // -- per-capability override takes precedence over default --

    #[test]
    fn per_capability_override_takes_precedence() {
        let requirements = vec![notes_read_requirement()];
        let store = admitted_store();
        let availability = obsidian_available();

        let mut policy = HostLocalPolicy::new(PolicyRule::Deny);
        policy.insert("notes.note.read", PolicyRule::Allow);

        let decision = evaluate_permission(
            &requirements,
            &store,
            &availability,
            &policy,
            "notes.note.read",
            1,
            Some("obsidian-local"),
        );

        // Override says Allow even though default is Deny.
        assert_eq!(decision, PermissionDecision::Allow);
    }

    // -- declared but not live or admitted → unavailable --

    #[test]
    fn declared_but_provider_unavailable_returns_unavailable() {
        let requirements = vec![notes_read_requirement()];
        let store = admitted_store();
        // No providers available.
        let availability = ProviderAvailability::empty();
        let policy = allow_all_policy();

        let decision = evaluate_permission(
            &requirements,
            &store,
            &availability,
            &policy,
            "notes.note.read",
            1,
            Some("obsidian-local"),
        );

        assert_eq!(decision, PermissionDecision::Unavailable);
    }

    #[test]
    fn declared_but_not_admitted_returns_unavailable() {
        let requirements = vec![notes_read_requirement()];
        let store = TrustedManifestStore::new(); // empty store
        let availability = obsidian_available();
        let policy = allow_all_policy();

        let decision = evaluate_permission(
            &requirements,
            &store,
            &availability,
            &policy,
            "notes.note.read",
            1,
            Some("obsidian-local"),
        );

        assert_eq!(decision, PermissionDecision::Unavailable);
    }

    // -- live admitted but undeclared → not permitted --

    #[test]
    fn live_admitted_but_undeclared_returns_deny() {
        // Set declares only notes.note.read@1.  notes.note.create@1 is
        // admitted and live but not declared.
        let requirements = vec![notes_read_requirement()];
        let mut store = admitted_store();
        // Also admit a write manifest so it exists in the store.
        let mut write_json = json!({
            "manifest_format_version": "1.0",
            "capability_name": "notes.note.create",
            "capability_version": 1,
            "title": "Create a note",
            "description": "Create.",
            "input_schema": { "type": "object", "properties": {}, "additionalProperties": false },
            "output_schema": { "type": "object", "properties": { "path": { "type": "string" } }, "required": ["path"] },
            "effects": ["filesystem.write"],
            "permission_scope": { "kind": "path_prefix", "allowed_prefixes": ["projects/"] },
            "reversibility": "compensatable",
            "determinism": "deterministic",
            "idempotency": { "mechanism": "argument_key", "argument_name": "idempotency_key", "key_source": "evaluation_id/action_id" },
            "confirmation_policy": { "standing_permitted": false, "per_call_required": true },
            "timeout_ms": 10000,
            "retry_policy": { "max_retries": 0, "backoff_ms": 500, "allowed_on": ["outcome_unknown"], "requires_idempotency_proof": false },
            "provider": { "identity": "obsidian-local", "display_name": "Obsidian", "identity_source": "host_configuration", "description": null },
            "binding": { "kind": "mcp", "server_name": "obsidian", "tool_name": "obsidian_create_note", "adapter": null }
        });
        let (_, digest) =
            crate::manifest::canonicalize_and_digest(&write_json.to_string()).unwrap();
        write_json["digest"] = json!(digest);
        store
            .insert(verify_manifest(&write_json.to_string()).unwrap())
            .unwrap();

        let availability = obsidian_available();
        let policy = allow_all_policy();

        let decision = evaluate_permission(
            &requirements,
            &store,
            &availability,
            &policy,
            "notes.note.create",
            1,
            Some("obsidian-local"),
        );

        // Admitted and live, but not declared → Deny.
        assert_eq!(decision, PermissionDecision::Deny);
    }

    // -- wrong exact version → unavailable --

    #[test]
    fn declared_version_1_but_requesting_version_2_returns_deny() {
        // Set declares notes.note.read@1.  Request is for @2.  @2 is
        // not in the declaration, so it falls to the "not declared" path.
        let requirements = vec![notes_read_requirement()];
        let store = admitted_store();
        let availability = obsidian_available();
        let policy = allow_all_policy();

        let decision = evaluate_permission(
            &requirements,
            &store,
            &availability,
            &policy,
            "notes.note.read",
            2,
            Some("obsidian-local"),
        );

        // Version 2 is not declared in the requirements → Deny.
        assert_eq!(decision, PermissionDecision::Deny);
    }

    // -- evaluation is read-only and stable --

    #[test]
    fn evaluation_is_read_only() {
        let requirements = vec![notes_read_requirement()];
        let store = admitted_store();
        let availability = obsidian_available();
        let policy = allow_all_policy();
        let len_before = store.len();

        let d1 = evaluate_permission(
            &requirements,
            &store,
            &availability,
            &policy,
            "notes.note.read",
            1,
            Some("obsidian-local"),
        );

        let d2 = evaluate_permission(
            &requirements,
            &store,
            &availability,
            &policy,
            "notes.note.read",
            1,
            Some("obsidian-local"),
        );

        assert_eq!(d1, d2);
        assert_eq!(d1, PermissionDecision::Allow);
        assert_eq!(store.len(), len_before);
    }

    // -- evaluate_permission_resolved with allowed policy --

    #[test]
    fn resolved_capability_with_allow_policy_returns_allow() {
        let requirements = vec![notes_read_requirement()];
        let store = admitted_store();
        let availability = obsidian_available();
        let resolved = resolver::resolve_capability(
            &store,
            &availability,
            "notes.note.read",
            1,
            Some("obsidian-local"),
        )
        .unwrap();

        let policy = allow_all_policy();
        let decision = evaluate_permission_resolved(&requirements, &resolved, &policy);

        assert_eq!(decision, PermissionDecision::Allow);
    }

    #[test]
    fn resolved_capability_with_deny_policy_returns_deny() {
        let requirements = vec![notes_read_requirement()];
        let store = admitted_store();
        let availability = obsidian_available();
        let resolved = resolver::resolve_capability(
            &store,
            &availability,
            "notes.note.read",
            1,
            Some("obsidian-local"),
        )
        .unwrap();

        let policy = deny_all_policy();
        let decision = evaluate_permission_resolved(&requirements, &resolved, &policy);

        assert_eq!(decision, PermissionDecision::Deny);
    }

    #[test]
    fn resolved_but_undeclared_returns_deny() {
        let requirements = vec![notes_read_requirement()]; // does NOT include create
        let availability = obsidian_available();

        // Build a store that has both notes.note.read@1 (declared) and
        // notes.note.create@1 (not declared).
        let mut store2 = TrustedManifestStore::new();
        store2.insert(verified_read()).unwrap();
        // Also admit create.
        let mut write_json = json!({
            "manifest_format_version": "1.0",
            "capability_name": "notes.note.create",
            "capability_version": 1,
            "title": "Create",
            "description": "Create.",
            "input_schema": { "type": "object", "properties": {}, "additionalProperties": false },
            "output_schema": { "type": "object", "properties": { "path": { "type": "string" } }, "required": ["path"] },
            "effects": ["filesystem.write"],
            "permission_scope": { "kind": "path_prefix", "allowed_prefixes": ["projects/"] },
            "reversibility": "compensatable",
            "determinism": "deterministic",
            "idempotency": { "mechanism": "argument_key", "argument_name": "idempotency_key", "key_source": "evaluation_id/action_id" },
            "confirmation_policy": { "standing_permitted": false, "per_call_required": true },
            "timeout_ms": 10000,
            "retry_policy": { "max_retries": 0, "backoff_ms": 500, "allowed_on": ["outcome_unknown"], "requires_idempotency_proof": false },
            "provider": { "identity": "obsidian-local", "display_name": "Obsidian", "identity_source": "host_configuration", "description": null },
            "binding": { "kind": "mcp", "server_name": "obsidian", "tool_name": "obsidian_create_note", "adapter": null }
        });
        let (_, digest) =
            crate::manifest::canonicalize_and_digest(&write_json.to_string()).unwrap();
        write_json["digest"] = json!(digest);
        let write_vm = verify_manifest(&write_json.to_string()).unwrap();
        store2.insert(write_vm).unwrap();

        let resolved = resolver::resolve_capability(
            &store2,
            &availability,
            "notes.note.create",
            1,
            Some("obsidian-local"),
        )
        .unwrap();

        let policy = allow_all_policy();
        // Requirements only has notes.note.read@1, not notes.note.create@1.
        let decision = evaluate_permission_resolved(&requirements, &resolved, &policy);

        assert_eq!(decision, PermissionDecision::Deny);
    }

    // -- no dispatch, Trail or Anchor side effect --

    #[test]
    fn permission_evaluation_has_no_side_effects() {
        let requirements = vec![notes_read_requirement()];
        let store = admitted_store();
        let availability = obsidian_available();
        let policy = allow_all_policy();

        // Repeated calls produce identical results and the store is untouched.
        for _ in 0..5 {
            let decision = evaluate_permission(
                &requirements,
                &store,
                &availability,
                &policy,
                "notes.note.read",
                1,
                Some("obsidian-local"),
            );
            assert_eq!(decision, PermissionDecision::Allow);
        }
        // No process spawn, no file I/O, no Trail append — the store
        // is unchanged.
        assert_eq!(store.len(), 1);
    }

    // -- HostLocalPolicy default posture --

    #[test]
    fn default_posture_applies_to_unknown_capabilities() {
        let policy = HostLocalPolicy::new(PolicyRule::Ask);
        assert_eq!(policy.rule_for("anything"), PolicyRule::Ask);
    }

    #[test]
    fn insert_override_is_retrievable() {
        let mut policy = HostLocalPolicy::new(PolicyRule::Ask);
        policy.insert("notes.note.read", PolicyRule::Allow);
        assert_eq!(policy.rule_for("notes.note.read"), PolicyRule::Allow);
        assert_eq!(policy.rule_for("other"), PolicyRule::Ask);
    }

    // -- unavailable is distinct from deny --

    #[test]
    fn unavailable_is_not_deny() {
        assert_ne!(PermissionDecision::Unavailable, PermissionDecision::Deny);
    }

    // -- requirement matching --

    #[test]
    fn requirement_matches_exact_name_and_version() {
        let req = CapabilityRequirement::new("notes.note.read", 1);
        assert_eq!(req.capability_name, "notes.note.read");
        assert_eq!(req.capability_version, 1);
    }
}
