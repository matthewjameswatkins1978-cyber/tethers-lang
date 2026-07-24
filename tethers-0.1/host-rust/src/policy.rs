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

use crate::resolver::{self, CapabilityIdentity, ProviderAvailability, ResolvedCapability};
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
/// Matches by exact capability identity: name + version. Overrides
/// take precedence over the default for that exact identity only.
///
/// This is host-controlled, in-memory policy.  It does not grant
/// permission to an undeclared or unavailable capability, and does
/// not bypass the trusted manifest store.
#[derive(Debug, Clone)]
pub struct HostLocalPolicy {
    /// Default posture for capabilities not listed in `overrides`.
    default_posture: PolicyRule,
    /// Per-capability overrides keyed by exact capability name + version.
    overrides: HashMap<CapabilityIdentity, PolicyRule>,
}

impl HostLocalPolicy {
    /// Create a policy with a default posture and no overrides.
    pub fn new(default_posture: PolicyRule) -> Self {
        Self {
            default_posture,
            overrides: HashMap::new(),
        }
    }

    /// Insert a per-capability override for an exact name/version pair.
    pub fn insert(
        &mut self,
        capability_name: impl Into<String>,
        capability_version: u32,
        rule: PolicyRule,
    ) {
        self.overrides.insert(
            CapabilityIdentity::new(capability_name, capability_version),
            rule,
        );
    }

    /// What rule applies to a given exact capability identity?
    pub fn rule_for(&self, capability_name: &str, capability_version: u32) -> PolicyRule {
        self.overrides
            .get(&CapabilityIdentity::new(
                capability_name,
                capability_version,
            ))
            .copied()
            .unwrap_or(self.default_posture)
    }
}

// ---------------------------------------------------------------------------
// Permission decision
// ---------------------------------------------------------------------------

/// Policy-created proof that one exact capability identity was allowed.
///
/// The readiness-establishing field is private: callers can inspect or
/// clone a token they were given, but only this policy module can create
/// one from an effective permission evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AllowedCapability {
    identity: CapabilityIdentity,
}

impl AllowedCapability {
    fn new(identity: CapabilityIdentity) -> Self {
        Self { identity }
    }

    pub fn capability_name(&self) -> &str {
        &self.identity.name
    }

    pub fn capability_version(&self) -> u32 {
        self.identity.version
    }
}

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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionDecision {
    Allow(AllowedCapability),
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
/// 3. Host policy deny → `Deny`.  Once a declared capability has a
///    current provider binding, an explicit denial prevents dispatch.
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
            match policy.rule_for(capability_name, capability_version) {
                PolicyRule::Deny => PermissionDecision::Deny,
                PolicyRule::Ask => PermissionDecision::Ask,
                PolicyRule::Allow => PermissionDecision::Allow(AllowedCapability::new(
                    CapabilityIdentity::new(capability_name, capability_version),
                )),
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
    let capability_name = resolved.capability_name();
    let capability_version = resolved.capability_version();

    // 1. Tether Set declaration check.
    let requirement = requirements.iter().find(|r| {
        r.capability_name.as_str() == capability_name && r.capability_version == capability_version
    });

    if requirement.is_none() {
        return PermissionDecision::Deny;
    }

    // 2. Already resolved — skip to policy.
    match policy.rule_for(capability_name, capability_version) {
        PolicyRule::Deny => PermissionDecision::Deny,
        PolicyRule::Ask => PermissionDecision::Ask,
        PolicyRule::Allow => {
            PermissionDecision::Allow(AllowedCapability::new(resolved.identity().clone()))
        }
    }
}

// ---------------------------------------------------------------------------
// J04 — complete effective-policy resolution (docs/DECISIONS.md J03/J03a/J03b)
// ---------------------------------------------------------------------------

/// Host-owned scope assessment supplied by the caller (J03b).
///
/// The policy resolver never inspects raw Action arguments to derive this
/// itself. A trusted host/binding-specific assessor produces it from the
/// verified manifest's declared `permission_scope`, the resolved non-secret
/// Action arguments, and the configured provider binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeAssessment {
    /// The host checked the declared scope against the resolved arguments
    /// and the Action is within it.
    WithinScope,
    /// The host checked the declared scope and the Action falls outside it.
    ScopeViolation,
    /// No trusted binding-specific assessor exists, the required argument is
    /// absent or ambiguous, or the assessment otherwise cannot be made.
    ScopeNotEstablished,
}

/// A proposed Action's identity, bridge pins, and resolved non-secret
/// arguments, as supplied by the caller for effective-policy resolution.
///
/// Every capability resolved through this host is bridge-backed (MCP
/// binding), so all three bridge pins are required for a `Deny`-free
/// evaluation; a missing pin is treated as a malformed Action identity.
#[derive(Debug, Clone)]
pub struct ProposedAction {
    pub evaluation_id: String,
    pub plan_id: String,
    pub action_id: String,
    pub capability_name: String,
    /// Opaque verified manifest digest pinned by the Plan.
    pub manifest_digest: Option<String>,
    /// Exact resolvable capability version pinned by the Plan.
    pub bridge_capability_version: Option<u32>,
    /// Provider identity pinned by the Plan.
    pub bridge_provider_identity: Option<String>,
    /// Resolved non-secret Action arguments.
    pub arguments: serde_json::Value,
}

/// A distinct, inspectable reason for an effective-policy outcome.
///
/// Carried alongside `PermissionDecision` rather than inside it, so every
/// existing `PermissionDecision` caller (including `dispatch.rs`) remains
/// unchanged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyReason {
    /// A required Action identifier was missing or empty.
    EmptyIdentifier(&'static str),
    /// A required bridge pin was missing.
    MissingBridgePin(&'static str),
    /// The capability/version was not declared by the selected Tether Set.
    UndeclaredCapability,
    /// Resolved Action arguments failed the manifest's `input_schema`.
    InputSchemaViolation(String),
    /// The host-owned scope assessment reported a violation.
    ScopeViolation,
    /// The host-owned scope assessment could not establish scope.
    ScopeNotEstablished,
    /// No admitted manifest exists for the exact capability identity.
    NoAdmittedManifest,
    /// The admitted manifest's provider is not currently available.
    ProviderUnavailable,
    /// The Action's pinned provider identity does not match the admitted
    /// manifest's provider identity.
    ProviderIdentityMismatch,
    /// The Action's pinned manifest digest does not match the current
    /// verified manifest digest.
    ManifestDigestMismatch,
    /// An exact host-local `Deny` rule applied.
    HostPolicyDeny,
    /// The manifest's `confirmation_policy.per_call_required` is `true`.
    ManifestRequiresConfirmation,
    /// An exact host-local `Ask` rule applied.
    HostPolicyAsk,
    /// An exact host-local `Allow` rule applied.
    HostPolicyAllow,
    /// Every other omitted, malformed, or unsupported policy configuration.
    UnsupportedPolicyConfiguration,
}

/// The complete effective-policy result: a `PermissionDecision` plus the
/// distinct reason that produced it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyEvaluation {
    pub decision: PermissionDecision,
    pub reason: PolicyReason,
}

impl PolicyEvaluation {
    fn deny(reason: PolicyReason) -> Self {
        Self {
            decision: PermissionDecision::Deny,
            reason,
        }
    }

    fn unavailable(reason: PolicyReason) -> Self {
        Self {
            decision: PermissionDecision::Unavailable,
            reason,
        }
    }

    fn ask(reason: PolicyReason) -> Self {
        Self {
            decision: PermissionDecision::Ask,
            reason,
        }
    }

    fn allow(reason: PolicyReason, allowed: AllowedCapability) -> Self {
        Self {
            decision: PermissionDecision::Allow(allowed),
            reason,
        }
    }
}

/// Evaluate the complete fail-closed effective policy for one proposed
/// Action (J03, corrected by J03a and J03b).
///
/// Precedence, in order (first match wins); the Ask-approval-resume step
/// (J03/J03a precedence step 4) is intentionally absent — it is reserved
/// for J05:
///
/// 1. Malformed/missing Action identity or a missing required bridge pin
///    → `Deny`.
/// 2. Capability/version not declared by the selected Tether Set → `Deny`.
/// 3. No admitted manifest, provider absent from availability, or a stale
///    manifest/provider pin against the admitted manifest → `Unavailable`.
/// 4. Resolved Action arguments fail the manifest `input_schema` → `Deny`.
/// 5. Host-owned scope assessment: `scope_violation` or
///    `scope_not_established` for a structured scope → `Deny`; `within_scope`
///    continues; `Unrestricted` relies on its structural
///    `per_call_required: true` invariant.
/// 6. An exact host-local `Deny` rule → `Deny`.
/// 7. Manifest `confirmation_policy.per_call_required: true`, or an exact
///    host-local `Ask` rule → `Ask`.
/// 8. An exact host-local `Allow` rule → `Allow`.
/// 9. Every other omitted, malformed, or unsupported policy configuration
///    → `Deny`.
///
/// Pure and deterministic: identical `action`, `store`, and `availability`
/// content always yields an identical `PolicyEvaluation`. Performs no I/O,
/// dispatch, executor call, or Trail write.
pub fn evaluate_effective_policy(
    action: &ProposedAction,
    requirements: &[CapabilityRequirement],
    store: &TrustedManifestStore,
    availability: &ProviderAvailability,
    policy: &HostLocalPolicy,
    scope_assessment: ScopeAssessment,
) -> PolicyEvaluation {
    // 1. Action identity and bridge-pin well-formedness.
    if action.evaluation_id.is_empty() {
        return PolicyEvaluation::deny(PolicyReason::EmptyIdentifier("evaluation_id"));
    }
    if action.plan_id.is_empty() {
        return PolicyEvaluation::deny(PolicyReason::EmptyIdentifier("plan_id"));
    }
    if action.action_id.is_empty() {
        return PolicyEvaluation::deny(PolicyReason::EmptyIdentifier("action_id"));
    }
    let Some(manifest_digest) = action.manifest_digest.as_deref() else {
        return PolicyEvaluation::deny(PolicyReason::MissingBridgePin("manifest_digest"));
    };
    if manifest_digest.is_empty() {
        return PolicyEvaluation::deny(PolicyReason::MissingBridgePin("manifest_digest"));
    }
    let Some(capability_version) = action.bridge_capability_version else {
        return PolicyEvaluation::deny(PolicyReason::MissingBridgePin("bridge_capability_version"));
    };
    let Some(provider_identity) = action.bridge_provider_identity.as_deref() else {
        return PolicyEvaluation::deny(PolicyReason::MissingBridgePin("bridge_provider_identity"));
    };
    if provider_identity.is_empty() {
        return PolicyEvaluation::deny(PolicyReason::MissingBridgePin("bridge_provider_identity"));
    }

    // 2. Tether Set declaration check.
    let declared = requirements.iter().any(|r| {
        r.capability_name == action.capability_name && r.capability_version == capability_version
    });
    if !declared {
        return PolicyEvaluation::deny(PolicyReason::UndeclaredCapability);
    }

    // 3. Live admitted resolution. The pinned provider identity is checked
    //    here, against the manifest's actual provider — a mismatch is a
    //    binding fact (`Unavailable`), not a policy override.
    let resolved = match resolver::resolve_capability(
        store,
        availability,
        &action.capability_name,
        capability_version,
        Some(provider_identity),
    ) {
        Ok(resolved) => resolved,
        Err(resolver::ResolutionError::NoAdmittedManifest { .. }) => {
            return PolicyEvaluation::unavailable(PolicyReason::NoAdmittedManifest);
        }
        Err(resolver::ResolutionError::ProviderUnavailable { .. }) => {
            return PolicyEvaluation::unavailable(PolicyReason::ProviderUnavailable);
        }
        Err(resolver::ResolutionError::ProviderIdentityMismatch { .. }) => {
            return PolicyEvaluation::unavailable(PolicyReason::ProviderIdentityMismatch);
        }
    };

    // The Plan must remain pinned to this exact verified manifest. A
    // non-empty digest is not sufficient: admitting a newer manifest under
    // the same capability identity must invalidate the older Plan.
    if manifest_digest != resolved.manifest_digest() {
        return PolicyEvaluation::unavailable(PolicyReason::ManifestDigestMismatch);
    }

    let manifest = resolved.manifest().manifest();

    // 4. Action input-schema validation.
    if let Err(err) =
        crate::validation::validate_against_schema(&manifest.input_schema, &action.arguments)
    {
        return PolicyEvaluation::deny(PolicyReason::InputSchemaViolation(err.message));
    }

    // 5. Host-owned scope assessment (J03b). `Unrestricted` scopes rely on
    //    their existing structural `per_call_required: true` invariant and
    //    are not subject to this assessment.
    if !matches!(
        manifest.permission_scope,
        crate::manifest::PermissionScope::Unrestricted
    ) {
        match scope_assessment {
            ScopeAssessment::ScopeViolation => {
                return PolicyEvaluation::deny(PolicyReason::ScopeViolation);
            }
            ScopeAssessment::ScopeNotEstablished => {
                return PolicyEvaluation::deny(PolicyReason::ScopeNotEstablished);
            }
            ScopeAssessment::WithinScope => {}
        }
    }

    // 6. Exact host-local Deny rule.
    let rule = policy.rule_for(&action.capability_name, capability_version);
    if matches!(rule, PolicyRule::Deny) {
        return PolicyEvaluation::deny(PolicyReason::HostPolicyDeny);
    }

    // 7. Mandatory manifest confirmation, or an exact host-local Ask rule.
    if manifest.confirmation_policy.per_call_required {
        return PolicyEvaluation::ask(PolicyReason::ManifestRequiresConfirmation);
    }
    if matches!(rule, PolicyRule::Ask) {
        return PolicyEvaluation::ask(PolicyReason::HostPolicyAsk);
    }

    // 8. Exact host-local Allow rule.
    if matches!(rule, PolicyRule::Allow) {
        return PolicyEvaluation::allow(
            PolicyReason::HostPolicyAllow,
            AllowedCapability::new(resolved.identity().clone()),
        );
    }

    // 9. Every other omitted, malformed, or unsupported policy configuration.
    PolicyEvaluation::deny(PolicyReason::UnsupportedPolicyConfiguration)
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

    fn verified_read_version(version: u32) -> VerifiedManifest {
        let mut m = read_manifest_json();
        m["capability_version"] = json!(version);
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

    fn notes_read_v2_requirement() -> CapabilityRequirement {
        CapabilityRequirement::new("notes.note.read", 2)
            .with_reason("Read notes from the vault using the v2 contract")
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

    fn assert_allow(decision: &PermissionDecision, name: &str, version: u32) {
        match decision {
            PermissionDecision::Allow(allowed) => {
                assert_eq!(allowed.capability_name(), name);
                assert_eq!(allowed.capability_version(), version);
            }
            other => panic!("expected Allow, got {other:?}"),
        }
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

        assert_allow(&decision, "notes.note.read", 1);
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
        policy.insert("notes.note.read", 1, PolicyRule::Deny);

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
        policy.insert("notes.note.read", 1, PolicyRule::Allow);

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
        assert_allow(&decision, "notes.note.read", 1);
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

    #[test]
    fn declared_same_name_different_version_but_not_admitted_returns_unavailable() {
        let requirements = vec![notes_read_v2_requirement()];
        let store = admitted_store(); // contains notes.note.read@1 only
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
        assert_allow(&d1, "notes.note.read", 1);
        assert_eq!(store.len(), len_before);
    }

    // -- exact-version policy overrides --

    #[test]
    fn exact_version_override_applies_to_its_identity() {
        let requirements = vec![notes_read_requirement()];
        let store = admitted_store();
        let availability = obsidian_available();
        let mut policy = HostLocalPolicy::new(PolicyRule::Deny);
        policy.insert("notes.note.read", 1, PolicyRule::Allow);

        let decision = evaluate_permission(
            &requirements,
            &store,
            &availability,
            &policy,
            "notes.note.read",
            1,
            Some("obsidian-local"),
        );

        assert_allow(&decision, "notes.note.read", 1);
    }

    #[test]
    fn same_name_other_version_does_not_inherit_override_and_uses_default() {
        let requirements = vec![notes_read_requirement(), notes_read_v2_requirement()];
        let mut store = admitted_store();
        store.insert(verified_read_version(2)).unwrap();
        let availability = obsidian_available();
        let mut policy = HostLocalPolicy::new(PolicyRule::Ask);
        policy.insert("notes.note.read", 1, PolicyRule::Allow);

        let v1 = evaluate_permission(
            &requirements,
            &store,
            &availability,
            &policy,
            "notes.note.read",
            1,
            Some("obsidian-local"),
        );
        let v2 = evaluate_permission(
            &requirements,
            &store,
            &availability,
            &policy,
            "notes.note.read",
            2,
            Some("obsidian-local"),
        );

        assert_allow(&v1, "notes.note.read", 1);
        assert_eq!(v2, PermissionDecision::Ask);
    }

    #[test]
    fn exact_version_ask_override_works() {
        let requirements = vec![notes_read_requirement()];
        let store = admitted_store();
        let availability = obsidian_available();
        let mut policy = HostLocalPolicy::new(PolicyRule::Deny);
        policy.insert("notes.note.read", 1, PolicyRule::Ask);

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

    #[test]
    fn exact_version_deny_override_works() {
        let requirements = vec![notes_read_requirement()];
        let store = admitted_store();
        let availability = obsidian_available();
        let mut policy = HostLocalPolicy::new(PolicyRule::Allow);
        policy.insert("notes.note.read", 1, PolicyRule::Deny);

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

    #[test]
    fn unavailable_takes_precedence_over_explicit_deny_for_declared_capability() {
        let requirements = vec![notes_read_requirement()];
        let store = admitted_store();
        let availability = ProviderAvailability::empty();
        let mut policy = HostLocalPolicy::new(PolicyRule::Allow);
        policy.insert("notes.note.read", 1, PolicyRule::Deny);

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

        assert_allow(&decision, "notes.note.read", 1);
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

    #[test]
    fn resolved_same_name_different_version_does_not_authorise_other_requirement() {
        let requirements = vec![notes_read_requirement()];
        let mut store = admitted_store();
        store.insert(verified_read_version(2)).unwrap();
        let availability = obsidian_available();
        let resolved = resolver::resolve_capability(
            &store,
            &availability,
            "notes.note.read",
            2,
            Some("obsidian-local"),
        )
        .unwrap();

        let policy = allow_all_policy();
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
            assert_allow(&decision, "notes.note.read", 1);
        }
        // No process spawn, no file I/O, no Trail append — the store
        // is unchanged.
        assert_eq!(store.len(), 1);
    }

    // -- HostLocalPolicy default posture --

    #[test]
    fn default_posture_applies_to_unknown_capabilities() {
        let policy = HostLocalPolicy::new(PolicyRule::Ask);
        assert_eq!(policy.rule_for("anything", 1), PolicyRule::Ask);
    }

    #[test]
    fn insert_override_is_retrievable() {
        let mut policy = HostLocalPolicy::new(PolicyRule::Ask);
        policy.insert("notes.note.read", 1, PolicyRule::Allow);
        assert_eq!(policy.rule_for("notes.note.read", 1), PolicyRule::Allow);
        assert_eq!(policy.rule_for("notes.note.read", 2), PolicyRule::Ask);
        assert_eq!(policy.rule_for("other", 1), PolicyRule::Ask);
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

    // -----------------------------------------------------------------
    // J04 — evaluate_effective_policy
    // -----------------------------------------------------------------

    fn read_manifest_digest() -> String {
        let m = read_manifest_json();
        let (_, digest) = crate::manifest::canonicalize_and_digest(&m.to_string()).unwrap();
        digest
    }

    fn verified_read_confirmation_required() -> VerifiedManifest {
        let mut m = read_manifest_json();
        m["confirmation_policy"] = json!({
            "standing_permitted": true,
            "per_call_required": true
        });
        let (_, digest) = crate::manifest::canonicalize_and_digest(&m.to_string()).unwrap();
        m["digest"] = json!(digest);
        verify_manifest(&m.to_string()).unwrap()
    }

    fn unrestricted_manifest_json() -> serde_json::Value {
        json!({
            "manifest_format_version": "1.0",
            "capability_name": "chaos.write",
            "capability_version": 1,
            "title": "Write without a declared scope",
            "description": "Write without a declared scope.",
            "input_schema": {
                "type": "object",
                "properties": { "value": { "type": "string" } },
                "required": ["value"],
                "additionalProperties": false
            },
            "output_schema": {
                "type": "object",
                "properties": { "status": { "type": "string" } },
                "required": ["status"]
            },
            "effects": ["filesystem.write"],
            "permission_scope": null,
            "reversibility": "irreversible",
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
                "allowed_on": ["outcome_unknown"],
                "requires_idempotency_proof": false
            },
            "provider": {
                "identity": "chaos-local",
                "display_name": "Chaos (local)",
                "identity_source": "host_configuration",
                "description": "Host-assigned."
            },
            "binding": {
                "kind": "mcp",
                "server_name": "chaos",
                "tool_name": "chaos_write",
                "adapter": null
            }
        })
    }

    fn verified_unrestricted() -> VerifiedManifest {
        let mut m = unrestricted_manifest_json();
        let (_, digest) = crate::manifest::canonicalize_and_digest(&m.to_string()).unwrap();
        m["digest"] = json!(digest);
        verify_manifest(&m.to_string()).unwrap()
    }

    fn valid_read_action() -> ProposedAction {
        ProposedAction {
            evaluation_id: "eval_1".into(),
            plan_id: "eval_1/plan".into(),
            action_id: "action_1".into(),
            capability_name: "notes.note.read".into(),
            manifest_digest: Some(read_manifest_digest()),
            bridge_capability_version: Some(1),
            bridge_provider_identity: Some("obsidian-local".into()),
            arguments: json!({"path": "projects/x"}),
        }
    }

    fn assert_deny_reason(evaluation: &PolicyEvaluation, expected_reason: &PolicyReason) {
        assert_eq!(evaluation.decision, PermissionDecision::Deny);
        assert_eq!(&evaluation.reason, expected_reason);
    }

    fn assert_unavailable_reason(evaluation: &PolicyEvaluation, expected_reason: &PolicyReason) {
        assert_eq!(evaluation.decision, PermissionDecision::Unavailable);
        assert_eq!(&evaluation.reason, expected_reason);
    }

    #[test]
    fn effective_policy_denies_empty_evaluation_id() {
        let mut action = valid_read_action();
        action.evaluation_id = String::new();
        let evaluation = evaluate_effective_policy(
            &action,
            &[notes_read_requirement()],
            &admitted_store(),
            &obsidian_available(),
            &allow_all_policy(),
            ScopeAssessment::WithinScope,
        );
        assert_deny_reason(&evaluation, &PolicyReason::EmptyIdentifier("evaluation_id"));
    }

    #[test]
    fn effective_policy_denies_empty_plan_id() {
        let mut action = valid_read_action();
        action.plan_id = String::new();
        let evaluation = evaluate_effective_policy(
            &action,
            &[notes_read_requirement()],
            &admitted_store(),
            &obsidian_available(),
            &allow_all_policy(),
            ScopeAssessment::WithinScope,
        );
        assert_deny_reason(&evaluation, &PolicyReason::EmptyIdentifier("plan_id"));
    }

    #[test]
    fn effective_policy_denies_empty_action_id() {
        let mut action = valid_read_action();
        action.action_id = String::new();
        let evaluation = evaluate_effective_policy(
            &action,
            &[notes_read_requirement()],
            &admitted_store(),
            &obsidian_available(),
            &allow_all_policy(),
            ScopeAssessment::WithinScope,
        );
        assert_deny_reason(&evaluation, &PolicyReason::EmptyIdentifier("action_id"));
    }

    #[test]
    fn effective_policy_denies_missing_manifest_digest_pin() {
        let mut action = valid_read_action();
        action.manifest_digest = None;
        let evaluation = evaluate_effective_policy(
            &action,
            &[notes_read_requirement()],
            &admitted_store(),
            &obsidian_available(),
            &allow_all_policy(),
            ScopeAssessment::WithinScope,
        );
        assert_deny_reason(
            &evaluation,
            &PolicyReason::MissingBridgePin("manifest_digest"),
        );
    }

    #[test]
    fn effective_policy_denies_missing_bridge_capability_version_pin() {
        let mut action = valid_read_action();
        action.bridge_capability_version = None;
        let evaluation = evaluate_effective_policy(
            &action,
            &[notes_read_requirement()],
            &admitted_store(),
            &obsidian_available(),
            &allow_all_policy(),
            ScopeAssessment::WithinScope,
        );
        assert_deny_reason(
            &evaluation,
            &PolicyReason::MissingBridgePin("bridge_capability_version"),
        );
    }

    #[test]
    fn effective_policy_denies_missing_bridge_provider_identity_pin() {
        let mut action = valid_read_action();
        action.bridge_provider_identity = None;
        let evaluation = evaluate_effective_policy(
            &action,
            &[notes_read_requirement()],
            &admitted_store(),
            &obsidian_available(),
            &allow_all_policy(),
            ScopeAssessment::WithinScope,
        );
        assert_deny_reason(
            &evaluation,
            &PolicyReason::MissingBridgePin("bridge_provider_identity"),
        );
    }

    #[test]
    fn effective_policy_denies_undeclared_capability() {
        let action = valid_read_action();
        let evaluation = evaluate_effective_policy(
            &action,
            &[], // Tether Set declares nothing.
            &admitted_store(),
            &obsidian_available(),
            &allow_all_policy(),
            ScopeAssessment::WithinScope,
        );
        assert_deny_reason(&evaluation, &PolicyReason::UndeclaredCapability);
    }

    #[test]
    fn effective_policy_reports_unavailable_for_no_admitted_manifest() {
        let action = valid_read_action();
        let empty_store = TrustedManifestStore::new();
        let evaluation = evaluate_effective_policy(
            &action,
            &[notes_read_requirement()],
            &empty_store,
            &obsidian_available(),
            &allow_all_policy(),
            ScopeAssessment::WithinScope,
        );
        assert_unavailable_reason(&evaluation, &PolicyReason::NoAdmittedManifest);
    }

    #[test]
    fn effective_policy_reports_unavailable_for_absent_provider() {
        let action = valid_read_action();
        let evaluation = evaluate_effective_policy(
            &action,
            &[notes_read_requirement()],
            &admitted_store(),
            &ProviderAvailability::empty(),
            &allow_all_policy(),
            ScopeAssessment::WithinScope,
        );
        assert_unavailable_reason(&evaluation, &PolicyReason::ProviderUnavailable);
    }

    #[test]
    fn effective_policy_reports_unavailable_for_provider_identity_mismatch() {
        let mut action = valid_read_action();
        action.bridge_provider_identity = Some("wrong-provider".into());
        let evaluation = evaluate_effective_policy(
            &action,
            &[notes_read_requirement()],
            &admitted_store(),
            &obsidian_available(),
            &allow_all_policy(),
            ScopeAssessment::WithinScope,
        );
        assert_unavailable_reason(&evaluation, &PolicyReason::ProviderIdentityMismatch);
    }

    #[test]
    fn effective_policy_reports_unavailable_for_stale_manifest_digest() {
        let mut action = valid_read_action();
        action.manifest_digest =
            Some("sha256:0000000000000000000000000000000000000000000000000000000000000000".into());
        let evaluation = evaluate_effective_policy(
            &action,
            &[notes_read_requirement()],
            &admitted_store(),
            &obsidian_available(),
            &allow_all_policy(),
            ScopeAssessment::WithinScope,
        );
        assert_unavailable_reason(&evaluation, &PolicyReason::ManifestDigestMismatch);
    }

    #[test]
    fn effective_policy_denies_input_schema_violation() {
        let mut action = valid_read_action();
        action.arguments = json!({}); // missing required "path"
        let evaluation = evaluate_effective_policy(
            &action,
            &[notes_read_requirement()],
            &admitted_store(),
            &obsidian_available(),
            &allow_all_policy(),
            ScopeAssessment::WithinScope,
        );
        assert_eq!(evaluation.decision, PermissionDecision::Deny);
        assert!(matches!(
            evaluation.reason,
            PolicyReason::InputSchemaViolation(_)
        ));
    }

    #[test]
    fn effective_policy_valid_arguments_reach_allow() {
        let action = valid_read_action();
        let evaluation = evaluate_effective_policy(
            &action,
            &[notes_read_requirement()],
            &admitted_store(),
            &obsidian_available(),
            &allow_all_policy(),
            ScopeAssessment::WithinScope,
        );
        assert_allow(&evaluation.decision, "notes.note.read", 1);
        assert_eq!(evaluation.reason, PolicyReason::HostPolicyAllow);
    }

    #[test]
    fn effective_policy_valid_arguments_reach_ask() {
        let action = valid_read_action();
        let evaluation = evaluate_effective_policy(
            &action,
            &[notes_read_requirement()],
            &admitted_store(),
            &obsidian_available(),
            &ask_all_policy(),
            ScopeAssessment::WithinScope,
        );
        assert_eq!(evaluation.decision, PermissionDecision::Ask);
        assert_eq!(evaluation.reason, PolicyReason::HostPolicyAsk);
    }

    #[test]
    fn effective_policy_denies_scope_violation_before_local_allow() {
        let action = valid_read_action();
        let evaluation = evaluate_effective_policy(
            &action,
            &[notes_read_requirement()],
            &admitted_store(),
            &obsidian_available(),
            &allow_all_policy(),
            ScopeAssessment::ScopeViolation,
        );
        assert_deny_reason(&evaluation, &PolicyReason::ScopeViolation);
    }

    #[test]
    fn effective_policy_denies_scope_not_established_before_local_allow() {
        let action = valid_read_action();
        let evaluation = evaluate_effective_policy(
            &action,
            &[notes_read_requirement()],
            &admitted_store(),
            &obsidian_available(),
            &allow_all_policy(),
            ScopeAssessment::ScopeNotEstablished,
        );
        assert_deny_reason(&evaluation, &PolicyReason::ScopeNotEstablished);
    }

    #[test]
    fn effective_policy_exact_deny_overrides_default_allow() {
        let action = valid_read_action();
        let mut policy = HostLocalPolicy::new(PolicyRule::Allow);
        policy.insert("notes.note.read", 1, PolicyRule::Deny);
        let evaluation = evaluate_effective_policy(
            &action,
            &[notes_read_requirement()],
            &admitted_store(),
            &obsidian_available(),
            &policy,
            ScopeAssessment::WithinScope,
        );
        assert_deny_reason(&evaluation, &PolicyReason::HostPolicyDeny);
    }

    #[test]
    fn effective_policy_manifest_confirmation_overrides_local_allow() {
        let mut store = TrustedManifestStore::new();
        store.insert(verified_read_confirmation_required()).unwrap();

        let mut m = read_manifest_json();
        m["confirmation_policy"] = json!({
            "standing_permitted": true,
            "per_call_required": true
        });
        let (_, digest) = crate::manifest::canonicalize_and_digest(&m.to_string()).unwrap();

        let mut action = valid_read_action();
        action.manifest_digest = Some(digest);

        let evaluation = evaluate_effective_policy(
            &action,
            &[notes_read_requirement()],
            &store,
            &obsidian_available(),
            &allow_all_policy(),
            ScopeAssessment::WithinScope,
        );
        assert_eq!(evaluation.decision, PermissionDecision::Ask);
        assert_eq!(
            evaluation.reason,
            PolicyReason::ManifestRequiresConfirmation
        );
    }

    #[test]
    fn effective_policy_unrestricted_scope_ignores_scope_assessment() {
        let mut store = TrustedManifestStore::new();
        store.insert(verified_unrestricted()).unwrap();

        let (_, digest) =
            crate::manifest::canonicalize_and_digest(&unrestricted_manifest_json().to_string())
                .unwrap();

        let action = ProposedAction {
            evaluation_id: "eval_2".into(),
            plan_id: "eval_2/plan".into(),
            action_id: "action_2".into(),
            capability_name: "chaos.write".into(),
            manifest_digest: Some(digest),
            bridge_capability_version: Some(1),
            bridge_provider_identity: Some("chaos-local".into()),
            arguments: json!({"value": "anything"}),
        };
        let requirements = vec![CapabilityRequirement::new("chaos.write", 1)];
        let availability = ProviderAvailability::from_identities(["chaos-local"]);

        // Unrestricted scope must not consult the (deliberately hostile)
        // scope assessment; its mandatory confirmation invariant applies.
        let evaluation = evaluate_effective_policy(
            &action,
            &requirements,
            &store,
            &availability,
            &allow_all_policy(),
            ScopeAssessment::ScopeNotEstablished,
        );
        assert_eq!(evaluation.decision, PermissionDecision::Ask);
        assert_eq!(
            evaluation.reason,
            PolicyReason::ManifestRequiresConfirmation
        );
    }

    #[test]
    fn effective_policy_is_deterministic_across_repeated_calls() {
        let action = valid_read_action();
        let requirements = vec![notes_read_requirement()];
        let store = admitted_store();
        let availability = obsidian_available();
        let policy = allow_all_policy();

        let first = evaluate_effective_policy(
            &action,
            &requirements,
            &store,
            &availability,
            &policy,
            ScopeAssessment::WithinScope,
        );
        let second = evaluate_effective_policy(
            &action,
            &requirements,
            &store,
            &availability,
            &policy,
            ScopeAssessment::WithinScope,
        );
        assert_eq!(first, second);
        // No I/O, dispatch, or Trail side effects: repeating resolution does
        // not mutate the store or availability snapshot.
        assert_eq!(store.len(), 1);
    }
}
