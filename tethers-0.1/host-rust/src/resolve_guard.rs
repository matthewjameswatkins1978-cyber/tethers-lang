//! Tethers-owned preparation evidence for the future Resolve guard seam.
//!
//! This module deliberately stops before guard admission.  It projects facts
//! that have already been resolved and authorised by the Rust host into typed,
//! deterministic, opaque evidence.  No type in this module can invoke a
//! provider, change replay state, grant permission, or contact Resolve.

use crate::approval;
use crate::configured_runtime::PreparedRuntime;
use crate::manifest::{BindingKind, IdentitySource};
use crate::policy::{PermissionDecision, ProposedAction, ScopeAssessment};
use crate::resolver::{self, ProviderAvailability, ResolvedCapability};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fmt;

pub const GUARD_PREPARATION_FORMAT_VERSION: &str = "tethers.resolve.guard-preparation/1";
pub const SCOPE_KEY_FORMAT_VERSION: &str = "tethers.resolve.scope-key/1";
pub const BINDING_DIGEST_FORMAT_VERSION: &str = "tethers.resolve.binding/1";
const RESOLVE_GUARD_REF_MAX_BYTES: usize = 512;

/// Opaque equality evidence for one reviewed Tethers scope dimension.
///
/// The original scope value is intentionally not retained.  Resolve may
/// compare keys, but it has no representation with which to interpret them.
#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct ScopeKey {
    digest: String,
}

impl ScopeKey {
    pub fn as_str(&self) -> &str {
        &self.digest
    }

    fn from_projection(projection: Value) -> Self {
        Self {
            digest: approval::digest(&projection),
        }
    }
}

impl fmt::Debug for ScopeKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ScopeKey").field(&self.digest).finish()
    }
}

/// Identity of one complete preparation proof.  It is evidence, not a token.
#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct GuardPreparationProofDigest {
    digest: String,
}

impl GuardPreparationProofDigest {
    pub fn as_str(&self) -> &str {
        &self.digest
    }
}

impl fmt::Debug for GuardPreparationProofDigest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("GuardPreparationProofDigest")
            .field(&self.digest)
            .finish()
    }
}

/// The controlled host-owned proof frozen by Resolve01 P0.
///
/// Fields are private so callers cannot fabricate a proof from arbitrary
/// strings. Construction is available only through the current prepared
/// runtime and its current capability, policy, and approval authorities.
#[derive(Clone, PartialEq, Eq)]
pub struct GuardPreparationProof {
    format_version: String,
    evaluation_id: String,
    plan_id: String,
    action_id: String,
    capability_name: String,
    capability_version: u32,
    argument_digest: String,
    manifest_digest: String,
    provider_identity: String,
    resolved_scope_digest: String,
    scope_keys: Vec<ScopeKey>,
    binding_digest: String,
}

impl fmt::Debug for GuardPreparationProof {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GuardPreparationProof")
            .field("format_version", &self.format_version)
            .field("evaluation_id", &self.evaluation_id)
            .field("plan_id", &self.plan_id)
            .field("action_id", &self.action_id)
            .field("capability_name", &self.capability_name)
            .field("capability_version", &self.capability_version)
            .field("argument_digest", &self.argument_digest)
            .field("manifest_digest", &self.manifest_digest)
            .field("provider_identity", &self.provider_identity)
            .field("resolved_scope_digest", &self.resolved_scope_digest)
            .field("scope_keys", &self.scope_keys)
            .field("binding_digest", &self.binding_digest)
            .finish()
    }
}

impl GuardPreparationProof {
    pub fn format_version(&self) -> &str {
        &self.format_version
    }
    pub fn evaluation_id(&self) -> &str {
        &self.evaluation_id
    }
    pub fn plan_id(&self) -> &str {
        &self.plan_id
    }
    pub fn action_id(&self) -> &str {
        &self.action_id
    }
    pub fn capability_name(&self) -> &str {
        &self.capability_name
    }
    pub fn capability_version(&self) -> u32 {
        self.capability_version
    }
    pub fn argument_digest(&self) -> &str {
        &self.argument_digest
    }
    pub fn manifest_digest(&self) -> &str {
        &self.manifest_digest
    }
    pub fn provider_identity(&self) -> &str {
        &self.provider_identity
    }
    pub fn resolved_scope_digest(&self) -> &str {
        &self.resolved_scope_digest
    }
    pub fn scope_keys(&self) -> &[ScopeKey] {
        &self.scope_keys
    }
    pub fn binding_digest(&self) -> &str {
        &self.binding_digest
    }

    /// The safe canonical projection.  It contains digests and identities,
    /// never the raw arguments or resolved scope value.
    pub fn canonical_projection(&self) -> Value {
        json!({
            "format_version": self.format_version,
            "evaluation_id": self.evaluation_id,
            "plan_id": self.plan_id,
            "action_id": self.action_id,
            "capability_name": self.capability_name,
            "capability_version": self.capability_version,
            "argument_digest": self.argument_digest,
            "manifest_digest": self.manifest_digest,
            "provider_identity": self.provider_identity,
            "resolved_scope_digest": self.resolved_scope_digest,
            "scope_keys": self.scope_keys.iter().map(|key| key.as_str()).collect::<Vec<_>>(),
            "binding_digest": self.binding_digest,
        })
    }

    pub fn digest(&self) -> GuardPreparationProofDigest {
        GuardPreparationProofDigest {
            digest: approval::digest(&self.canonical_projection()),
        }
    }
}

/// Minimal opaque projection handed to the future Resolve adapter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolveGuardRequired {
    preparation_digest: GuardPreparationProofDigest,
    action_id: String,
    scope_keys: Vec<ScopeKey>,
}

impl ResolveGuardRequired {
    pub fn preparation_digest(&self) -> &GuardPreparationProofDigest {
        &self.preparation_digest
    }
    pub fn action_id(&self) -> &str {
        &self.action_id
    }
    pub fn scope_keys(&self) -> &[ScopeKey] {
        &self.scope_keys
    }
}

/// Opaque identity of a Resolve guard request.
///
/// P2 deliberately does not interpret the value or expose it through
/// formatting.  The future transport adapter receives the validated object;
/// host evidence records only its existing SHA-256 digest.
#[derive(Clone, Eq, PartialEq)]
pub struct ResolveGuardRef {
    digest: String,
}

impl ResolveGuardRef {
    /// Construct a reference supplied by a host-owned integration boundary.
    /// The value is only checked as bounded opaque UTF-8; P2 does not assign
    /// semantics to Resolve identifiers.
    pub fn from_host_value(value: &str) -> Result<Self, ResolveGuardRefError> {
        if value.is_empty() {
            return Err(ResolveGuardRefError::Empty);
        }
        if value.len() > RESOLVE_GUARD_REF_MAX_BYTES {
            return Err(ResolveGuardRefError::TooLong);
        }
        if value.chars().any(char::is_control) {
            return Err(ResolveGuardRefError::ControlCharacter);
        }
        Ok(Self {
            digest: approval::digest(&Value::String(value.to_owned())),
        })
    }

    pub fn digest(&self) -> &str {
        &self.digest
    }
}

impl fmt::Debug for ResolveGuardRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ResolveGuardRef")
            .field(&self.digest)
            .finish()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResolveGuardRefError {
    Empty,
    TooLong,
    ControlCharacter,
}

impl fmt::Display for ResolveGuardRefError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::Empty => "guard reference is empty",
            Self::TooLong => "guard reference exceeds the bounded length",
            Self::ControlCharacter => "guard reference contains a control character",
        };
        f.write_str(message)
    }
}

impl std::error::Error for ResolveGuardRefError {}

/// The only decisions a P2 adapter may return.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResolveGuardAdmission {
    Admitted,
    Rejected,
    Indeterminate,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolveGuardAdapterError;

/// One deliberately narrow future Resolve boundary.  P2 supplies no live
/// transport implementation; adapter errors are mapped to Indeterminate by
/// the host seam.
pub trait ResolveGuardAdapter {
    fn admit_guard(
        &mut self,
        guard_ref: &ResolveGuardRef,
        required: &ResolveGuardRequired,
    ) -> Result<ResolveGuardAdmission, ResolveGuardAdapterError>;
}

/// Request state that has been rebuilt from current Tethers authorities and is
/// safe to present at the future adapter boundary.
pub struct GuardAdmissionContext<'a> {
    guard_ref: ResolveGuardRef,
    required: ResolveGuardRequired,
    adapter: &'a mut dyn ResolveGuardAdapter,
}

/// Explicit host-selected guard inputs for a Together group. The plan is
/// keyed by already validated Action identity; it never discovers guard
/// references from arguments, provider labels, or free text.
pub(crate) struct GuardAdmissionPlan<'a> {
    adapter: &'a mut dyn ResolveGuardAdapter,
    requests: HashMap<String, GuardAdmissionRequest>,
}

pub(crate) struct GuardAdmissionRequest {
    pub(crate) guard_ref: ResolveGuardRef,
    pub(crate) previous: Option<GuardPreparationProof>,
}

impl<'a> GuardAdmissionPlan<'a> {
    #[allow(dead_code)]
    // The default production route remains unguarded until an explicit host
    // integration selects this P2 Together seam.
    pub(crate) fn new(
        adapter: &'a mut dyn ResolveGuardAdapter,
        requests: HashMap<String, GuardAdmissionRequest>,
    ) -> Self {
        Self { adapter, requests }
    }

    pub(crate) fn prepare_for(
        &mut self,
        runtime: &PreparedRuntime,
        action: &ProposedAction,
        availability: &ProviderAvailability,
        approval: Option<(&approval::ApprovalStore, &str)>,
    ) -> Result<GuardAdmissionContext<'_>, GuardPreparationError> {
        let request = self
            .requests
            .get(&action.action_id)
            .ok_or(GuardPreparationError::MissingGuardReference)?;
        prepare_guard_admission(
            runtime,
            action,
            availability,
            approval,
            request.guard_ref.clone(),
            request.previous.as_ref(),
            self.adapter,
        )
    }
}

impl<'a> GuardAdmissionContext<'a> {
    #[allow(dead_code)]
    // Construction is controlled by the current-authority preparation route.
    pub(crate) fn new(
        guard_ref: ResolveGuardRef,
        prepared: PreparedResolveGuard,
        adapter: &'a mut dyn ResolveGuardAdapter,
    ) -> Self {
        Self {
            guard_ref,
            required: prepared.required,
            adapter,
        }
    }

    pub(crate) fn required(&self) -> &ResolveGuardRequired {
        &self.required
    }

    pub(crate) fn guard_ref_digest(&self) -> &str {
        self.guard_ref.digest()
    }

    pub(crate) fn admit(&mut self) -> ResolveGuardAdmission {
        self.adapter
            .admit_guard(&self.guard_ref, &self.required)
            .unwrap_or(ResolveGuardAdmission::Indeterminate)
    }
}

/// Build the guarded execution context from current Tethers-owned evidence.
/// An optional previous proof is comparison input only, never an authority
/// source.  `None` is the fresh preparation path; `Some` is explicit resume.
#[allow(dead_code)]
pub(crate) fn prepare_guard_admission<'a>(
    runtime: &PreparedRuntime,
    action: &ProposedAction,
    availability: &ProviderAvailability,
    approval: Option<(&approval::ApprovalStore, &str)>,
    guard_ref: ResolveGuardRef,
    previous: Option<&GuardPreparationProof>,
    adapter: &'a mut dyn ResolveGuardAdapter,
) -> Result<GuardAdmissionContext<'a>, GuardPreparationError> {
    let prepared = match previous {
        Some(previous) => {
            reconstruct_resolve_guard_evidence(previous, runtime, action, availability, approval)?
        }
        None => prepare_resolve_guard_evidence(runtime, action, availability, approval)?,
    };
    Ok(GuardAdmissionContext::new(guard_ref, prepared, adapter))
}

#[cfg(test)]
pub(crate) fn test_guard_admission_context<'a>(
    adapter: &'a mut dyn ResolveGuardAdapter,
) -> GuardAdmissionContext<'a> {
    let scope = ResolvedActionScope::unrestricted();
    let proof = GuardPreparationProof {
        format_version: GUARD_PREPARATION_FORMAT_VERSION.to_owned(),
        evaluation_id: "evaluation-1".to_owned(),
        plan_id: "plan-1".to_owned(),
        action_id: "action-1".to_owned(),
        capability_name: "lantern.task.record".to_owned(),
        capability_version: 1,
        argument_digest: approval::digest(&json!({})),
        manifest_digest: approval::digest(&json!({"manifest": 1})),
        provider_identity: "lantern-local".to_owned(),
        resolved_scope_digest: scope.digest(),
        scope_keys: scope_keys(&scope),
        binding_digest: approval::digest(&json!({"binding": 1})),
    };
    let required = ResolveGuardRequired {
        preparation_digest: proof.digest(),
        action_id: proof.action_id.clone(),
        scope_keys: proof.scope_keys.clone(),
    };
    GuardAdmissionContext::new(
        ResolveGuardRef::from_host_value("guard/test-1").expect("test guard reference is valid"),
        PreparedResolveGuard { proof, required },
        adapter,
    )
}

/// Both the complete proof and its bounded future-adapter projection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreparedResolveGuard {
    proof: GuardPreparationProof,
    required: ResolveGuardRequired,
}

impl PreparedResolveGuard {
    pub fn proof(&self) -> &GuardPreparationProof {
        &self.proof
    }
    pub fn required(&self) -> &ResolveGuardRequired {
        &self.required
    }
}

/// Preparation failures preserve the distinction between unsupported Tethers
/// scope, stale host identity, and a caller that has not established Allow.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GuardPreparationError {
    NotAuthorised,
    InvalidIdentifier(&'static str),
    MissingBridgePin(&'static str),
    CapabilityMismatch,
    InvalidArguments,
    ManifestMismatch,
    ProviderMismatch,
    BindingUnavailable,
    ApprovalRequired,
    ApprovalMismatch,
    MissingGuardReference,
    EvidenceMismatch(GuardPreparationMismatch),
    ScopeViolation,
    ScopeNotEstablished,
    UnsupportedScope,
}

impl fmt::Display for GuardPreparationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAuthorised => {
                write!(f, "guard preparation requires an existing Allow decision")
            }
            Self::InvalidIdentifier(field) => write!(f, "invalid preparation identifier: {field}"),
            Self::MissingBridgePin(field) => write!(f, "missing preparation bridge pin: {field}"),
            Self::CapabilityMismatch => write!(f, "resolved capability does not match the action"),
            Self::InvalidArguments => write!(f, "action arguments failed the capability schema"),
            Self::ManifestMismatch => write!(f, "resolved manifest does not match the action"),
            Self::ProviderMismatch => write!(f, "resolved provider does not match the action"),
            Self::BindingUnavailable => write!(f, "trusted execution binding is unavailable"),
            Self::ApprovalRequired => write!(f, "current policy requires exact approval"),
            Self::ApprovalMismatch => write!(f, "exact approval is not current for this action"),
            Self::MissingGuardReference => {
                write!(
                    f,
                    "guard preparation has no explicit guard reference for this action"
                )
            }
            Self::EvidenceMismatch(reason) => write!(f, "{reason}"),
            Self::ScopeViolation => {
                write!(f, "resolved action scope is outside the reviewed scope")
            }
            Self::ScopeNotEstablished => {
                write!(f, "resolved action scope was not established safely")
            }
            Self::UnsupportedScope => write!(
                f,
                "resolved scope form is unsupported for guard preparation"
            ),
        }
    }
}

impl std::error::Error for GuardPreparationError {}

/// Exact comparison reasons.  These are diagnostic evidence, not policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuardPreparationMismatch {
    UnsupportedProofVersion,
    MalformedEvidence,
    EvaluationChanged,
    PlanChanged,
    ActionChanged,
    CapabilityChanged,
    ManifestChanged,
    ProviderChanged,
    ArgumentsChanged,
    BindingChanged,
    ScopeChanged,
    ScopeKeysChanged,
}

impl fmt::Display for GuardPreparationMismatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "preparation evidence mismatch: {self:?}")
    }
}

impl std::error::Error for GuardPreparationMismatch {}

/// A reviewed scope produced by `PreparedRuntime`; it cannot be constructed
/// from raw descriptive text by callers outside this crate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ResolvedActionScope {
    PathPrefix {
        value: String,
        argument_json_pointer: String,
        allowed_prefixes: Vec<String>,
        within_scope: bool,
    },
    Unrestricted,
}

impl ResolvedActionScope {
    pub(crate) fn path_prefix(
        value: String,
        argument_json_pointer: String,
        allowed_prefixes: Vec<String>,
        within_scope: bool,
    ) -> Self {
        Self::PathPrefix {
            value,
            argument_json_pointer,
            allowed_prefixes,
            within_scope,
        }
    }

    pub(crate) fn unrestricted() -> Self {
        Self::Unrestricted
    }

    pub(crate) fn is_within_scope(&self) -> bool {
        match self {
            Self::PathPrefix { within_scope, .. } => *within_scope,
            Self::Unrestricted => true,
        }
    }

    /// Canonical scalar map for the Lantern authority wire contract.  This is
    /// a projection of the already validated Tethers scope; it is not a
    /// second scope parser or an authority decision.
    pub(crate) fn authority_scope(&self) -> std::collections::BTreeMap<String, String> {
        let mut scope = std::collections::BTreeMap::new();
        match self {
            Self::PathPrefix {
                value,
                argument_json_pointer,
                allowed_prefixes,
                ..
            } => {
                scope.insert("kind".to_owned(), "path_prefix".to_owned());
                scope.insert("value".to_owned(), value.clone());
                scope.insert(
                    "argument_json_pointer".to_owned(),
                    argument_json_pointer.clone(),
                );
                let mut prefixes = allowed_prefixes.clone();
                prefixes.sort();
                prefixes.dedup();
                scope.insert("allowed_prefixes".to_owned(), prefixes.join("\u{001f}"));
            }
            Self::Unrestricted => {
                scope.insert("kind".to_owned(), "unrestricted".to_owned());
            }
        }
        scope
    }

    fn key(&self) -> ScopeKey {
        let projection = match self {
            Self::PathPrefix { value, .. } => json!({
                "format_version": SCOPE_KEY_FORMAT_VERSION,
                "kind": "path_prefix",
                "value": value,
            }),
            Self::Unrestricted => json!({
                "format_version": SCOPE_KEY_FORMAT_VERSION,
                "kind": "unrestricted",
            }),
        };
        ScopeKey::from_projection(projection)
    }

    fn digest(&self) -> String {
        let projection = match self {
            Self::PathPrefix {
                value,
                argument_json_pointer,
                allowed_prefixes,
                ..
            } => {
                let mut prefixes = allowed_prefixes.clone();
                prefixes.sort();
                prefixes.dedup();
                json!({
                    "format_version": SCOPE_KEY_FORMAT_VERSION,
                    "kind": "path_prefix",
                    "value": value,
                    "argument_json_pointer": argument_json_pointer,
                    "allowed_prefixes": prefixes,
                })
            }
            Self::Unrestricted => json!({
                "format_version": SCOPE_KEY_FORMAT_VERSION,
                "kind": "unrestricted",
            }),
        };
        approval::digest(&projection)
    }

    fn binding_scope_projection(&self) -> Value {
        match self {
            Self::PathPrefix {
                argument_json_pointer,
                ..
            } => json!({
                "kind": "path_prefix",
                "argument_json_pointer": argument_json_pointer,
            }),
            Self::Unrestricted => Value::Null,
        }
    }
}

/// Derive a canonical, sorted, duplicate-free key vector from reviewed scope.
fn scope_keys(scope: &ResolvedActionScope) -> Vec<ScopeKey> {
    let mut keys = vec![scope.key()];
    keys.sort();
    keys.dedup();
    keys
}

/// Construct preparation evidence after current capability, binding scope,
/// and policy/approval authorities have succeeded.
pub fn prepare_resolve_guard_evidence(
    runtime: &PreparedRuntime,
    action: &ProposedAction,
    availability: &ProviderAvailability,
    approval: Option<(&approval::ApprovalStore, &str)>,
) -> Result<PreparedResolveGuard, GuardPreparationError> {
    let current = resolver::resolve_capability(
        runtime.trusted_store(),
        availability,
        &action.capability_name,
        action
            .bridge_capability_version
            .ok_or(GuardPreparationError::MissingBridgePin(
                "bridge_capability_version",
            ))?,
        action.bridge_provider_identity.as_deref(),
    )
    .map_err(|_| GuardPreparationError::BindingUnavailable)?;
    crate::validation::validate_against_schema(
        &current.manifest().manifest().input_schema,
        &action.arguments,
    )
    .map_err(|_| GuardPreparationError::InvalidArguments)?;

    let scope = runtime
        .resolve_action_scope(action)
        .map_err(|assessment| match assessment {
            ScopeAssessment::ScopeViolation => GuardPreparationError::ScopeViolation,
            ScopeAssessment::ScopeNotEstablished => GuardPreparationError::ScopeNotEstablished,
            ScopeAssessment::WithinScope => GuardPreparationError::BindingUnavailable,
        })?;
    if !scope.is_within_scope() {
        return Err(GuardPreparationError::ScopeViolation);
    }
    let prepared = runtime
        .providers()
        .iter()
        .flat_map(|provider| provider.capabilities.iter().map(move |cap| (provider, cap)))
        .find(|(provider, cap)| {
            provider.identity == current.provider_identity()
                && cap.name == current.capability_name()
                && cap.version == current.capability_version()
                && cap.verified_manifest.verified_digest() == current.manifest_digest()
        })
        .ok_or(GuardPreparationError::BindingUnavailable)?;
    let policy_evaluation = crate::policy::evaluate_effective_policy(
        action,
        runtime.requirements(),
        runtime.trusted_store(),
        availability,
        runtime.policy(),
        if scope.is_within_scope() {
            ScopeAssessment::WithinScope
        } else {
            ScopeAssessment::ScopeViolation
        },
    );
    let decision = match policy_evaluation.decision {
        PermissionDecision::Allow(allowed) => PermissionDecision::Allow(allowed),
        PermissionDecision::Ask => {
            let (approvals, approval_id) =
                approval.ok_or(GuardPreparationError::ApprovalRequired)?;
            let fresh_approval = approval::ApprovalProof::from_action(action)
                .map_err(|_| GuardPreparationError::ApprovalMismatch)?;
            let record = approvals
                .record(approval_id)
                .map_err(|_| GuardPreparationError::ApprovalMismatch)?;
            if record.state != approval::ApprovalState::Approved {
                return Err(GuardPreparationError::ApprovalMismatch);
            }
            if !record.proof.exactly_matches(&fresh_approval) {
                return Err(GuardPreparationError::ApprovalMismatch);
            }
            crate::policy::allow_after_exact_approval(&current)
        }
        PermissionDecision::Deny => return Err(GuardPreparationError::NotAuthorised),
        PermissionDecision::Unavailable => return Err(GuardPreparationError::BindingUnavailable),
    };
    prepare_from_parts(action, &current, &decision, &scope, prepared.0, prepared.1)
}

/// Rebuilds all evidence from current Tethers state and refuses to return it
/// unless it exactly matches the previous proof. The old proof is comparison
/// input only; no current authority is reconstructed from it.
pub fn reconstruct_resolve_guard_evidence(
    previous: &GuardPreparationProof,
    runtime: &PreparedRuntime,
    action: &ProposedAction,
    availability: &ProviderAvailability,
    approval: Option<(&approval::ApprovalStore, &str)>,
) -> Result<PreparedResolveGuard, GuardPreparationError> {
    let fresh = prepare_resolve_guard_evidence(runtime, action, availability, approval)?;
    compare_resolve_guard_preparation(previous, fresh.proof())
        .map_err(GuardPreparationError::EvidenceMismatch)?;
    Ok(fresh)
}

fn prepare_from_parts(
    action: &ProposedAction,
    resolved: &ResolvedCapability,
    decision: &PermissionDecision,
    scope: &ResolvedActionScope,
    provider: &crate::configured_runtime::PreparedProvider,
    prepared: &crate::configured_runtime::PreparedCapability,
) -> Result<PreparedResolveGuard, GuardPreparationError> {
    let allowed = match decision {
        PermissionDecision::Allow(allowed) => allowed,
        PermissionDecision::Ask | PermissionDecision::Deny | PermissionDecision::Unavailable => {
            return Err(GuardPreparationError::NotAuthorised)
        }
    };
    for (name, value) in [
        ("evaluation_id", action.evaluation_id.as_str()),
        ("plan_id", action.plan_id.as_str()),
        ("action_id", action.action_id.as_str()),
        ("capability_name", action.capability_name.as_str()),
    ] {
        if value.is_empty() {
            return Err(GuardPreparationError::InvalidIdentifier(name));
        }
    }
    let action_version =
        action
            .bridge_capability_version
            .ok_or(GuardPreparationError::MissingBridgePin(
                "bridge_capability_version",
            ))?;
    let action_manifest = action
        .manifest_digest
        .as_deref()
        .filter(|value| !value.is_empty())
        .ok_or(GuardPreparationError::MissingBridgePin("manifest_digest"))?;
    let action_provider = action
        .bridge_provider_identity
        .as_deref()
        .filter(|value| !value.is_empty())
        .ok_or(GuardPreparationError::MissingBridgePin(
            "bridge_provider_identity",
        ))?;

    if allowed.capability_name() != resolved.capability_name()
        || allowed.capability_version() != resolved.capability_version()
    {
        return Err(GuardPreparationError::NotAuthorised);
    }
    if action.capability_name != resolved.capability_name()
        || action_version != resolved.capability_version()
    {
        return Err(GuardPreparationError::CapabilityMismatch);
    }
    if action_manifest != resolved.manifest_digest()
        || prepared.verified_manifest.verified_digest() != resolved.manifest_digest()
    {
        return Err(GuardPreparationError::ManifestMismatch);
    }
    if action_provider != resolved.provider_identity()
        || provider.identity != resolved.provider_identity()
    {
        return Err(GuardPreparationError::ProviderMismatch);
    }
    crate::validation::validate_against_schema(
        &resolved.manifest().manifest().input_schema,
        &action.arguments,
    )
    .map_err(|_| GuardPreparationError::InvalidArguments)?;

    let binding_digest = binding_digest(provider, prepared, resolved, scope)?;
    let proof = GuardPreparationProof {
        format_version: GUARD_PREPARATION_FORMAT_VERSION.to_owned(),
        evaluation_id: action.evaluation_id.clone(),
        plan_id: action.plan_id.clone(),
        action_id: action.action_id.clone(),
        capability_name: resolved.capability_name().to_owned(),
        capability_version: resolved.capability_version(),
        argument_digest: approval::digest(&action.arguments),
        manifest_digest: resolved.manifest_digest().to_owned(),
        provider_identity: resolved.provider_identity().to_owned(),
        resolved_scope_digest: scope.digest(),
        scope_keys: scope_keys(scope),
        binding_digest,
    };
    let digest = proof.digest();
    let required = ResolveGuardRequired {
        preparation_digest: digest,
        action_id: proof.action_id.clone(),
        scope_keys: proof.scope_keys.clone(),
    };
    Ok(PreparedResolveGuard { proof, required })
}

fn binding_digest(
    provider: &crate::configured_runtime::PreparedProvider,
    prepared: &crate::configured_runtime::PreparedCapability,
    resolved: &ResolvedCapability,
    scope: &ResolvedActionScope,
) -> Result<String, GuardPreparationError> {
    let manifest = resolved.manifest().manifest();
    let binding = &manifest.binding;
    let binding_kind = match binding.kind {
        BindingKind::Mcp => "mcp",
    };
    let provider_source = match manifest.provider.identity_source {
        IdentitySource::HostConfiguration => "host_configuration",
    };
    let adapter = binding.adapter.as_ref().map(
        |value| json!({ "name": value.name, "version": value.version, "digest": value.digest }),
    );
    let scope_binding = prepared.scope_binding.as_ref().map(|value| {
        json!({
            "kind": "path_prefix",
            "argument_json_pointer": value.argument_json_pointer,
        })
    });
    if provider.identity != manifest.provider.identity {
        return Err(GuardPreparationError::ProviderMismatch);
    }
    Ok(approval::digest(&json!({
        "format_version": BINDING_DIGEST_FORMAT_VERSION,
        "capability_name": resolved.capability_name(),
        "capability_version": resolved.capability_version(),
        "manifest_digest": resolved.manifest_digest(),
        "provider_identity": manifest.provider.identity,
        "provider_identity_source": provider_source,
        "binding": {
            "kind": binding_kind,
            "server_name": binding.server_name,
            "tool_name": binding.tool_name,
            "adapter": adapter,
        },
        "scope_binding": scope_binding,
        "resolved_scope": scope.binding_scope_projection(),
        "provider_launch": {
            "command": provider.stdio_config.command,
            "args": provider.stdio_config.args,
            "working_directory": provider.working_directory,
        },
    })))
}

/// Compare two proofs field by field, rejecting malformed or future evidence.
pub fn compare_resolve_guard_preparation(
    old: &GuardPreparationProof,
    fresh: &GuardPreparationProof,
) -> Result<(), GuardPreparationMismatch> {
    for proof in [old, fresh] {
        if proof.format_version != GUARD_PREPARATION_FORMAT_VERSION {
            return Err(GuardPreparationMismatch::UnsupportedProofVersion);
        }
        if [
            proof.evaluation_id.as_str(),
            proof.plan_id.as_str(),
            proof.action_id.as_str(),
            proof.capability_name.as_str(),
        ]
        .iter()
        .any(|value| value.is_empty())
            || !is_digest(&proof.argument_digest)
            || !is_digest(&proof.manifest_digest)
            || !is_digest(&proof.resolved_scope_digest)
            || !is_digest(&proof.binding_digest)
            || proof.scope_keys.iter().any(|key| !is_digest(key.as_str()))
        {
            return Err(GuardPreparationMismatch::MalformedEvidence);
        }
    }
    if old.evaluation_id != fresh.evaluation_id {
        return Err(GuardPreparationMismatch::EvaluationChanged);
    }
    if old.plan_id != fresh.plan_id {
        return Err(GuardPreparationMismatch::PlanChanged);
    }
    if old.action_id != fresh.action_id {
        return Err(GuardPreparationMismatch::ActionChanged);
    }
    if old.capability_name != fresh.capability_name
        || old.capability_version != fresh.capability_version
    {
        return Err(GuardPreparationMismatch::CapabilityChanged);
    }
    if old.argument_digest != fresh.argument_digest {
        return Err(GuardPreparationMismatch::ArgumentsChanged);
    }
    if old.manifest_digest != fresh.manifest_digest {
        return Err(GuardPreparationMismatch::ManifestChanged);
    }
    if old.provider_identity != fresh.provider_identity {
        return Err(GuardPreparationMismatch::ProviderChanged);
    }
    if old.binding_digest != fresh.binding_digest {
        return Err(GuardPreparationMismatch::BindingChanged);
    }
    if old.resolved_scope_digest != fresh.resolved_scope_digest {
        return Err(GuardPreparationMismatch::ScopeChanged);
    }
    if old.scope_keys != fresh.scope_keys {
        return Err(GuardPreparationMismatch::ScopeKeysChanged);
    }
    Ok(())
}

fn is_digest(value: &str) -> bool {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return false;
    };
    hex.len() == 64
        && hex
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path(path: &str) -> ResolvedActionScope {
        ResolvedActionScope::path_prefix(
            path.to_owned(),
            "/path".to_owned(),
            vec!["/repo".to_owned()],
            true,
        )
    }

    fn proof() -> GuardPreparationProof {
        let scope = path("/repo/file.txt");
        GuardPreparationProof {
            format_version: GUARD_PREPARATION_FORMAT_VERSION.to_owned(),
            evaluation_id: "evaluation-1".to_owned(),
            plan_id: "plan-1".to_owned(),
            action_id: "action-1".to_owned(),
            capability_name: "files.write".to_owned(),
            capability_version: 1,
            argument_digest: approval::digest(
                &json!({ "path": "/repo/file.txt", "secret": "never in proof" }),
            ),
            manifest_digest: approval::digest(&json!({ "manifest": 1 })),
            provider_identity: "provider-1".to_owned(),
            resolved_scope_digest: scope.digest(),
            scope_keys: scope_keys(&scope),
            binding_digest: approval::digest(&json!({ "binding": 1 })),
        }
    }

    #[test]
    fn scope_keys_are_stable_opaque_and_explicit() {
        let a = path("/repo/file.txt");
        let b = path("/repo/file.txt");
        assert_eq!(a.key(), b.key());
        assert_ne!(a.key(), path("/other/file.txt").key());
        assert_eq!(
            ResolvedActionScope::unrestricted().key(),
            ResolvedActionScope::unrestricted().key()
        );
        assert_eq!(
            approval::digest(&json!({ "a": 1, "b": 2 })),
            approval::digest(&json!({ "b": 2, "a": 1 }))
        );
        assert!(!format!("{:?}", a.key()).contains("/repo/file.txt"));
    }

    #[test]
    fn scope_keys_are_sorted_and_duplicate_free() {
        let first = path("/repo/a").key();
        let second = path("/repo/b").key();
        let mut keys = vec![second.clone(), first.clone(), second, first];
        keys.sort();
        keys.dedup();
        assert_eq!(keys.len(), 2);
        assert!(keys[0] < keys[1]);
    }

    #[test]
    fn proof_digest_and_projection_are_deterministic_without_raw_values() {
        let first = proof();
        let second = proof();
        assert_eq!(first.digest(), second.digest());
        let text = format!("{:?} {:?}", first, first.canonical_projection());
        assert!(!text.contains("never in proof"));
        assert!(!text.contains("/repo/file.txt"));
    }

    #[test]
    fn every_material_proof_field_is_exactly_compared() {
        let base = proof();
        let cases = [
            (
                "evaluation",
                {
                    let mut p = base.clone();
                    p.evaluation_id.push('x');
                    p
                },
                GuardPreparationMismatch::EvaluationChanged,
            ),
            (
                "plan",
                {
                    let mut p = base.clone();
                    p.plan_id.push('x');
                    p
                },
                GuardPreparationMismatch::PlanChanged,
            ),
            (
                "action",
                {
                    let mut p = base.clone();
                    p.action_id.push('x');
                    p
                },
                GuardPreparationMismatch::ActionChanged,
            ),
            (
                "capability",
                {
                    let mut p = base.clone();
                    p.capability_name.push('x');
                    p
                },
                GuardPreparationMismatch::CapabilityChanged,
            ),
            (
                "arguments",
                {
                    let mut p = base.clone();
                    p.argument_digest = approval::digest(&json!({"changed": true}));
                    p
                },
                GuardPreparationMismatch::ArgumentsChanged,
            ),
            (
                "manifest",
                {
                    let mut p = base.clone();
                    p.manifest_digest = approval::digest(&json!({"changed": true}));
                    p
                },
                GuardPreparationMismatch::ManifestChanged,
            ),
            (
                "provider",
                {
                    let mut p = base.clone();
                    p.provider_identity.push('x');
                    p
                },
                GuardPreparationMismatch::ProviderChanged,
            ),
            (
                "scope",
                {
                    let mut p = base.clone();
                    p.resolved_scope_digest = approval::digest(&json!({"changed": true}));
                    p
                },
                GuardPreparationMismatch::ScopeChanged,
            ),
            (
                "keys",
                {
                    let mut p = base.clone();
                    p.scope_keys = vec![ResolvedActionScope::unrestricted().key()];
                    p
                },
                GuardPreparationMismatch::ScopeKeysChanged,
            ),
            (
                "binding",
                {
                    let mut p = base.clone();
                    p.binding_digest = approval::digest(&json!({"changed": true}));
                    p
                },
                GuardPreparationMismatch::BindingChanged,
            ),
        ];
        for (label, changed, expected) in cases {
            assert_eq!(
                compare_resolve_guard_preparation(&base, &changed),
                Err(expected),
                "{label}"
            );
        }
    }

    #[test]
    fn malformed_and_future_proofs_fail_closed() {
        let base = proof();
        let mut future = base.clone();
        future.format_version = "tethers.resolve.guard-preparation/99".to_owned();
        assert_eq!(
            compare_resolve_guard_preparation(&base, &future),
            Err(GuardPreparationMismatch::UnsupportedProofVersion)
        );
        let mut malformed = base;
        malformed.argument_digest = "not-a-digest".to_owned();
        assert_eq!(
            compare_resolve_guard_preparation(&malformed, &proof()),
            Err(GuardPreparationMismatch::MalformedEvidence)
        );
    }

    struct TestAdapter {
        result: Result<ResolveGuardAdmission, ResolveGuardAdapterError>,
        calls: usize,
    }

    impl ResolveGuardAdapter for TestAdapter {
        fn admit_guard(
            &mut self,
            _guard_ref: &ResolveGuardRef,
            _required: &ResolveGuardRequired,
        ) -> Result<ResolveGuardAdmission, ResolveGuardAdapterError> {
            self.calls += 1;
            self.result.clone()
        }
    }

    fn test_context(adapter: &mut TestAdapter) -> GuardAdmissionContext<'_> {
        let proof = proof();
        let required = ResolveGuardRequired {
            preparation_digest: proof.digest(),
            action_id: proof.action_id.clone(),
            scope_keys: proof.scope_keys.clone(),
        };
        let prepared = PreparedResolveGuard { proof, required };
        GuardAdmissionContext::new(
            ResolveGuardRef::from_host_value("guard/opaque-1").unwrap(),
            prepared,
            adapter,
        )
    }

    #[test]
    fn guard_reference_is_bounded_and_never_debug_leaks_value() {
        assert_eq!(
            ResolveGuardRef::from_host_value(""),
            Err(ResolveGuardRefError::Empty)
        );
        assert_eq!(
            ResolveGuardRef::from_host_value("bad\nref"),
            Err(ResolveGuardRefError::ControlCharacter)
        );
        let value = "guard/secret-looking-value";
        let guard_ref = ResolveGuardRef::from_host_value(value).unwrap();
        assert!(is_digest(guard_ref.digest()));
        assert!(!format!("{guard_ref:?}").contains(value));
    }

    #[test]
    fn adapter_error_is_closed_to_indeterminate() {
        let mut adapter = TestAdapter {
            result: Err(ResolveGuardAdapterError),
            calls: 0,
        };
        let mut context = test_context(&mut adapter);
        assert_eq!(context.admit(), ResolveGuardAdmission::Indeterminate);
        drop(context);
        assert_eq!(adapter.calls, 1);
    }

    #[test]
    fn adapter_receives_only_bounded_required_projection() {
        let mut adapter = TestAdapter {
            result: Ok(ResolveGuardAdmission::Admitted),
            calls: 0,
        };
        let mut context = test_context(&mut adapter);
        assert_eq!(context.admit(), ResolveGuardAdmission::Admitted);
        drop(context);
        assert_eq!(adapter.calls, 1);
        assert!(!format!(
            "{:?}",
            ResolveGuardRef::from_host_value("guard/opaque-1").unwrap()
        )
        .contains("opaque-1"));
    }
}
