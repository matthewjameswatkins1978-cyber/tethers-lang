// dispatch.rs — Intent-first dispatch preparation
//
// The single narrow boundary between permission and execution.
// Proves that no dispatch-ready action escapes before its intent
// has been durably recorded.
//
// ---------------------------------------------------------------------------
// Enforcement boundary
// ---------------------------------------------------------------------------
//
// `prepare_and_record()` establishes a usable dispatch-intent proof
// boundary.  It returns `DispatchReadyAction` only after successful
// durable intent recording.
//
// `authorise_and_execute()` in `main.rs` now requires `&DispatchReadyAction`
// for every provider/executor invocation.  The compiler enforces that no
// production effectful path can bypass durable intent preparation.
//
// ---------------------------------------------------------------------------
// Missing validation (explicitly deferred)
// ---------------------------------------------------------------------------
//
// - Action arguments are preserved raw.  Full JSON Schema validation
//   against the manifest's `input_schema` is a later boundary.
// - `PermissionDecision::Allow` carries a policy-created
//   `AllowedCapability` token.  `prepare_and_record` verifies that
//   token against the resolved capability before recording intent.
// ---------------------------------------------------------------------------

use crate::policy::PermissionDecision;
use crate::resolver::ResolvedCapability;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Identifiers
// ---------------------------------------------------------------------------

/// Caller-supplied stable execution identifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionId(pub String);

/// Caller-supplied stable action identifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionId(pub String);

// ---------------------------------------------------------------------------
// Intent entry
// ---------------------------------------------------------------------------

/// A durable intent record written before any effectful call.
///
/// This is the smallest provisional intent record for the current
/// proof boundary.  It is not yet the complete execution Trail
/// envelope described by the architecture docs.
#[derive(Debug, Clone, serde::Serialize)]
pub struct IntentEntry {
    pub execution_id: String,
    pub action_id: String,
    pub capability_name: String,
    pub capability_version: u32,
    pub provider_identity: String,
    pub manifest_digest: String,
    /// Raw action arguments, preserved exactly as received.
    ///
    /// Full JSON Schema validation against the manifest's `input_schema`
    /// is a later boundary.  This increment preserves arguments without
    /// validating them.
    pub arguments: serde_json::Value,
}

// ---------------------------------------------------------------------------
// Outcome entry
// ---------------------------------------------------------------------------

/// A durable execution-outcome record written after the executor returns.
///
/// Written after `IntentEntry` in the durable Trail.  The status field
/// is `"succeeded"` or `"failed"`.  The `result` and `error_message`
/// fields are mutually exclusive: a succeeded outcome carries a result;
/// a failed outcome carries an error message.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct OutcomeEntry {
    pub execution_id: String,
    pub action_id: String,
    /// `"succeeded"` or `"failed"`.
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    /// Host-supplied wall-clock timestamp in milliseconds since Unix epoch.
    pub timestamp_unix_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct AuthorisationEntry {
    pub execution_id: String,
    pub action_id: String,
    pub capability_name: String,
    pub capability_version: u32,
    pub provider_identity: String,
    pub manifest_digest: String,
    pub kind: String,
    pub reason_code: String,
    pub argument_digest: String,
}

// ---------------------------------------------------------------------------
// DispatchReadyAction — proof token
// ---------------------------------------------------------------------------

/// Proof that dispatch intent has been durably recorded.
///
/// Fields are private — only [`prepare_and_record`] can construct this
/// type.  Accessors provide read-only inspection.  No other module can
/// construct a `DispatchReadyAction` via a struct literal; the compiler
/// enforces this.
///
/// # Guarantees
///
/// - Intent has been durably appended and flushed.
/// - The exact capability name, version, provider identity, manifest
///   digest, and arguments are bound to stable execution/action
///   identifiers.
///
/// # Does NOT guarantee
///
/// - A provider has been contacted.
/// - The Action completed or will complete.
/// - Arguments were validated against the manifest's `input_schema`.
/// - Permission was re-checked after intent recording.
#[derive(Debug)]
pub struct DispatchReadyAction {
    execution_id: ExecutionId,
    action_id: ActionId,
    capability_name: String,
    capability_version: u32,
    provider_identity: String,
    manifest_digest: String,
    verified_manifest: crate::manifest::VerifiedManifest,
    arguments: serde_json::Value,
}

impl DispatchReadyAction {
    pub fn execution_id(&self) -> &ExecutionId {
        &self.execution_id
    }

    pub fn action_id(&self) -> &ActionId {
        &self.action_id
    }

    pub fn capability_name(&self) -> &str {
        &self.capability_name
    }

    pub fn capability_version(&self) -> u32 {
        self.capability_version
    }

    pub fn provider_identity(&self) -> &str {
        &self.provider_identity
    }

    pub fn manifest_digest(&self) -> &str {
        &self.manifest_digest
    }

    pub fn verified_manifest(&self) -> &crate::manifest::VerifiedManifest {
        &self.verified_manifest
    }

    pub fn arguments(&self) -> &serde_json::Value {
        &self.arguments
    }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Why intent preparation failed.
#[derive(Debug, PartialEq, Eq)]
pub enum PrepareError {
    /// Permission is Ask — confirmation required before dispatch.
    Ask,
    /// Permission is Deny — explicitly prohibited.
    Deny,
    /// Permission is Unavailable — no current provider binding.
    Unavailable,
    /// The Allow identity does not match the resolved capability.
    CapabilityIdentityMismatch {
        allowed_name: String,
        allowed_version: u32,
        resolved_name: String,
        resolved_version: u32,
    },
    /// Empty execution or action identifier.
    EmptyIdentifier { field: &'static str },
    /// Serialization or write failure during intent append.  No
    /// dispatch-ready token was returned; the file may contain no bytes,
    /// a partial record, or an unconfirmed complete record.
    IntentWriteFailed { message: String },
    /// Flush/sync/durability failure after write.  No dispatch-ready
    /// token was returned; the file may contain no bytes, a partial
    /// record, or an unconfirmed complete record.
    IntentFlushFailed { message: String },
}

// ---------------------------------------------------------------------------
// Trail abstraction
// ---------------------------------------------------------------------------

mod sealed {
    pub trait Sealed {}
}

/// Durable intent and outcome recording.
///
/// Implementations must ensure that a successful `append_and_flush_intent`
/// or `append_outcome` means the record has reached durable storage.
pub trait Trail: sealed::Sealed {
    /// Serialize, append, flush userspace buffers, and sync to durable
    /// storage.  Returns `Ok(())` only when the intent is durable.
    ///
    /// On any error, callers must not dispatch.  A failed write, flush,
    /// or sync may still leave no bytes, a partial record, or an
    /// unconfirmed complete record at the tail.
    fn append_and_flush_intent(&mut self, entry: &IntentEntry) -> Result<(), TrailError>;
    fn append_authorisation(&mut self, entry: &AuthorisationEntry) -> Result<(), TrailError>;

    /// Serialize, append, flush, and sync an execution outcome to durable
    /// storage.  Returns `Ok(())` only when the outcome is durable.
    ///
    /// Called after the executor returns.  On failure, the Action has
    /// already occurred; callers must preserve the known execution status
    /// and record an audit-failure entry in the response Trail.
    fn append_outcome(&mut self, entry: &OutcomeEntry) -> Result<(), TrailError>;
}

#[derive(Debug, PartialEq, Eq)]
pub enum TrailError {
    /// Serialization or write failed.  The file may contain no bytes,
    /// a partial record, or an unconfirmed complete record.
    WriteFailed(String),
    /// Flush/sync/durability failed.  The file may contain no bytes,
    /// a partial record, or an unconfirmed complete record.
    FlushFailed(String),
}

impl std::fmt::Display for TrailError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "trail error: {self:?}")
    }
}

impl std::error::Error for TrailError {}

// ---------------------------------------------------------------------------
// File-backed Trail (real durable)
// ---------------------------------------------------------------------------

/// Append-only JSONL Trail backed by a filesystem file.
///
/// Each intent is serialized as one JSON line, written, buffered,
/// and `sync_data()`ed before reporting success.  A failed write,
/// flush, or sync returns no dispatch-ready token, but this is a
/// serial-use append helper, not an atomic multi-writer JSONL store
/// or crash-recovery mechanism.  Uses only the Rust standard library
/// — no database or external dependency.
pub struct FileTrail {
    file: fs::File,
    path: PathBuf,
}

impl FileTrail {
    /// Open or create the Trail file at `path` in append mode.
    pub fn open(path: impl Into<PathBuf>) -> io::Result<Self> {
        let path = path.into();
        let file = OpenOptions::new().create(true).append(true).open(&path)?;
        Ok(Self { file, path })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl sealed::Sealed for FileTrail {}

impl Trail for FileTrail {
    fn append_and_flush_intent(&mut self, entry: &IntentEntry) -> Result<(), TrailError> {
        let line = serde_json::to_string(entry)
            .map_err(|e| TrailError::WriteFailed(format!("serialization failed: {e}")))?;

        writeln!(self.file, "{line}")
            .map_err(|e| TrailError::WriteFailed(format!("write failed: {e}")))?;

        self.file
            .flush()
            .map_err(|e| TrailError::FlushFailed(format!("flush failed: {e}")))?;

        self.file
            .sync_data()
            .map_err(|e| TrailError::FlushFailed(format!("sync_data failed: {e}")))?;

        Ok(())
    }

    fn append_authorisation(&mut self, entry: &AuthorisationEntry) -> Result<(), TrailError> {
        let line =
            serde_json::to_string(entry).map_err(|e| TrailError::WriteFailed(e.to_string()))?;
        writeln!(self.file, "{line}").map_err(|e| TrailError::WriteFailed(e.to_string()))?;
        self.file
            .flush()
            .map_err(|e| TrailError::FlushFailed(e.to_string()))?;
        self.file
            .sync_data()
            .map_err(|e| TrailError::FlushFailed(e.to_string()))
    }

    fn append_outcome(&mut self, entry: &OutcomeEntry) -> Result<(), TrailError> {
        let line = serde_json::to_string(entry)
            .map_err(|e| TrailError::WriteFailed(format!("serialization failed: {e}")))?;

        writeln!(self.file, "{line}")
            .map_err(|e| TrailError::WriteFailed(format!("write failed: {e}")))?;

        self.file
            .flush()
            .map_err(|e| TrailError::FlushFailed(format!("flush failed: {e}")))?;

        self.file
            .sync_data()
            .map_err(|e| TrailError::FlushFailed(format!("sync_data failed: {e}")))?;

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Recording Trail (test double — NOT durable)
// ---------------------------------------------------------------------------

/// In-memory test Trail.  Records entries for inspection but provides
/// **no durability**.  Used in tests to verify intent content and
/// ordering without touching the filesystem.
///
/// `injected_intent_error` simulates an intent write or flush failure.
/// `injected_outcome_error` simulates an outcome write or flush failure.
#[cfg(test)]
pub struct RecordingTrail {
    pub entries: Vec<IntentEntry>,
    pub authorisation_entries: Vec<AuthorisationEntry>,
    pub outcome_entries: Vec<OutcomeEntry>,
    pub injected_intent_error: Option<TrailError>,
    pub injected_authorisation_error: Option<TrailError>,
    pub injected_outcome_error: Option<TrailError>,
}

#[cfg(test)]
impl RecordingTrail {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            authorisation_entries: Vec::new(),
            outcome_entries: Vec::new(),
            injected_intent_error: None,
            injected_authorisation_error: None,
            injected_outcome_error: None,
        }
    }
}

#[cfg(test)]
impl sealed::Sealed for RecordingTrail {}

#[cfg(test)]
impl Trail for RecordingTrail {
    fn append_and_flush_intent(&mut self, entry: &IntentEntry) -> Result<(), TrailError> {
        if let Some(err) = self.injected_intent_error.take() {
            return Err(err);
        }
        self.entries.push(entry.clone());
        Ok(())
    }

    fn append_authorisation(&mut self, entry: &AuthorisationEntry) -> Result<(), TrailError> {
        if let Some(err) = self.injected_authorisation_error.take() {
            return Err(err);
        }
        self.authorisation_entries.push(entry.clone());
        Ok(())
    }

    fn append_outcome(&mut self, entry: &OutcomeEntry) -> Result<(), TrailError> {
        if let Some(err) = self.injected_outcome_error.take() {
            return Err(err);
        }
        self.outcome_entries.push(entry.clone());
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// prepare_and_record — the boundary
// ---------------------------------------------------------------------------

/// Prepare a dispatch-ready action, recording its intent durably first.
///
/// If this function returns `Ok(DispatchReadyAction)`, the matching
/// intent record has already been durably appended and flushed.
/// No provider call occurs.
///
/// # Arguments
///
/// * `decision` — effective permission decision from policy evaluation,
///   including a policy-created `AllowedCapability` token for Allow.
/// * `resolved` — the resolved capability (admitted, verified, available).
/// * `execution_id` — caller-supplied stable execution identifier.
/// * `action_id` — caller-supplied stable action identifier.
/// * `arguments` — proposed action arguments (preserved raw).
/// * `trail` — durable intent recorder.
pub fn prepare_and_record(
    decision: PermissionDecision,
    resolved: &ResolvedCapability,
    execution_id: ExecutionId,
    action_id: ActionId,
    arguments: serde_json::Value,
    trail: &mut dyn Trail,
) -> Result<DispatchReadyAction, PrepareError> {
    // 1. Decision must be Allow and must carry a policy-created token.
    let allowed = match decision {
        PermissionDecision::Allow(allowed) => allowed,
        PermissionDecision::Ask => return Err(PrepareError::Ask),
        PermissionDecision::Deny => return Err(PrepareError::Deny),
        PermissionDecision::Unavailable => return Err(PrepareError::Unavailable),
    };

    // 2. Validate non-empty identifiers.
    if execution_id.0.is_empty() {
        return Err(PrepareError::EmptyIdentifier {
            field: "execution_id",
        });
    }
    if action_id.0.is_empty() {
        return Err(PrepareError::EmptyIdentifier { field: "action_id" });
    }

    // 3. Identity binding: the AllowedIdentity must match the resolved
    //    capability.
    if allowed.capability_name() != resolved.capability_name()
        || allowed.capability_version() != resolved.capability_version()
    {
        return Err(PrepareError::CapabilityIdentityMismatch {
            allowed_name: allowed.capability_name().to_owned(),
            allowed_version: allowed.capability_version(),
            resolved_name: resolved.capability_name().to_owned(),
            resolved_version: resolved.capability_version(),
        });
    }

    let capability_name = resolved.capability_name();
    let capability_version = resolved.capability_version();
    let provider_identity = resolved.provider_identity();
    let manifest_digest = resolved.manifest_digest();

    debug_assert_eq!(resolved.manifest().capability_name(), capability_name);
    debug_assert_eq!(resolved.manifest().capability_version(), capability_version);
    debug_assert_eq!(resolved.manifest().verified_digest(), manifest_digest);

    // 4. Build intent entry.
    let entry = IntentEntry {
        execution_id: execution_id.0.clone(),
        action_id: action_id.0.clone(),
        capability_name: capability_name.to_owned(),
        capability_version,
        provider_identity: provider_identity.to_owned(),
        manifest_digest: manifest_digest.to_owned(),
        arguments: arguments.clone(),
    };

    // 5. Durably append and flush intent.
    trail.append_and_flush_intent(&entry).map_err(|e| match e {
        TrailError::WriteFailed(msg) => PrepareError::IntentWriteFailed { message: msg },
        TrailError::FlushFailed(msg) => PrepareError::IntentFlushFailed { message: msg },
    })?;

    // 6. Return proof token.
    Ok(DispatchReadyAction {
        execution_id,
        action_id,
        capability_name: capability_name.to_owned(),
        capability_version,
        provider_identity: provider_identity.to_owned(),
        manifest_digest: manifest_digest.to_owned(),
        verified_manifest: resolved.manifest().clone(),
        arguments,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::VerifiedManifest;
    use crate::policy::{
        evaluate_permission_resolved, CapabilityRequirement, HostLocalPolicy, PermissionDecision,
        PolicyRule,
    };
    use crate::resolver::{self, ProviderAvailability};
    use crate::trusted_store::TrustedManifestStore;
    use serde_json::json;
    use std::io::Read;

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

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
        crate::manifest::verify_manifest(&m.to_string()).unwrap()
    }

    fn notes_read_requirement() -> CapabilityRequirement {
        CapabilityRequirement::new("notes.note.read", 1).with_reason("Read notes")
    }

    fn resolved_read() -> (
        TrustedManifestStore,
        ProviderAvailability,
        ResolvedCapability,
    ) {
        let mut store = TrustedManifestStore::new();
        store.insert(verified_read()).unwrap();
        let availability = ProviderAvailability::from_identities(["obsidian-local"]);
        let resolved = resolver::resolve_capability(
            &store,
            &availability,
            "notes.note.read",
            1,
            Some("obsidian-local"),
        )
        .unwrap();
        (store, availability, resolved)
    }

    fn allow_all_policy() -> HostLocalPolicy {
        HostLocalPolicy::new(PolicyRule::Allow)
    }

    fn allow_decision_for(resolved: &ResolvedCapability) -> PermissionDecision {
        let requirements = vec![CapabilityRequirement::new(
            resolved.capability_name().to_owned(),
            resolved.capability_version(),
        )];
        evaluate_permission_resolved(&requirements, resolved, &allow_all_policy())
    }

    fn assert_allowed(decision: &PermissionDecision, name: &str, version: u32) {
        match decision {
            PermissionDecision::Allow(allowed) => {
                assert_eq!(allowed.capability_name(), name);
                assert_eq!(allowed.capability_version(), version);
            }
            other => panic!("expected Allow, got {other:?}"),
        }
    }

    // -----------------------------------------------------------------------
    // Test 1: Allow + valid resolved exact capability + successful durable
    //         intent write returns DispatchReadyAction.
    // -----------------------------------------------------------------------

    #[test]
    fn allow_with_valid_resolved_capability_returns_dispatch_ready_action() {
        let (_store, _availability, resolved) = resolved_read();
        let policy = allow_all_policy();
        let requirements = vec![notes_read_requirement()];

        let decision = evaluate_permission_resolved(&requirements, &resolved, &policy);
        assert_allowed(&decision, "notes.note.read", 1);

        let mut trail = RecordingTrail::new();
        let args = json!({"path": "projects/test.md"});
        let exec_id = ExecutionId("exec-001".into());
        let action_id = ActionId("action_1".into());

        let ready = prepare_and_record(
            decision,
            &resolved,
            exec_id.clone(),
            action_id.clone(),
            args.clone(),
            &mut trail,
        )
        .unwrap();

        assert_eq!(ready.execution_id().0, "exec-001");
        assert_eq!(ready.action_id().0, "action_1");
        assert_eq!(ready.capability_name(), "notes.note.read");
        assert_eq!(ready.capability_version(), 1);
        assert_eq!(ready.provider_identity(), "obsidian-local");
        assert_eq!(ready.manifest_digest(), resolved.manifest_digest());
        assert_eq!(ready.arguments(), &args);
    }

    // -----------------------------------------------------------------------
    // Test 2: The matching intent exists before the ready action is returned.
    // -----------------------------------------------------------------------

    #[test]
    fn intent_is_recorded_before_ready_action_is_returned() {
        let (_store, _availability, resolved) = resolved_read();
        let policy = allow_all_policy();
        let requirements = vec![notes_read_requirement()];
        let decision = evaluate_permission_resolved(&requirements, &resolved, &policy);

        let mut trail = RecordingTrail::new();
        assert!(trail.entries.is_empty());

        let _ready = prepare_and_record(
            decision,
            &resolved,
            ExecutionId("exec-001".into()),
            ActionId("action_1".into()),
            json!({"path": "projects/test.md"}),
            &mut trail,
        )
        .unwrap();

        assert_eq!(trail.entries.len(), 1);
        let intent = &trail.entries[0];
        assert_eq!(intent.execution_id, "exec-001");
        assert_eq!(intent.action_id, "action_1");
        assert_eq!(intent.capability_name, "notes.note.read");
        assert_eq!(intent.capability_version, 1);
        assert_eq!(intent.provider_identity, "obsidian-local");
    }

    // -----------------------------------------------------------------------
    // Test 3: Ask returns a typed not-ready result and writes no intent.
    // -----------------------------------------------------------------------

    #[test]
    fn ask_returns_not_ready_and_writes_no_intent() {
        let (_store, _availability, resolved) = resolved_read();
        let policy = HostLocalPolicy::new(PolicyRule::Ask);
        let requirements = vec![notes_read_requirement()];
        let decision = evaluate_permission_resolved(&requirements, &resolved, &policy);
        assert_eq!(decision, PermissionDecision::Ask);

        let mut trail = RecordingTrail::new();
        let err = prepare_and_record(
            decision,
            &resolved,
            ExecutionId("exec-001".into()),
            ActionId("action_1".into()),
            json!({}),
            &mut trail,
        )
        .unwrap_err();

        assert_eq!(err, PrepareError::Ask);
        assert!(trail.entries.is_empty());
    }

    // -----------------------------------------------------------------------
    // Test 4: Deny returns no ready action and writes no intent.
    // -----------------------------------------------------------------------

    #[test]
    fn deny_returns_no_ready_action_and_writes_no_intent() {
        let (_store, _availability, resolved) = resolved_read();
        let policy = HostLocalPolicy::new(PolicyRule::Deny);
        let requirements = vec![notes_read_requirement()];
        let decision = evaluate_permission_resolved(&requirements, &resolved, &policy);
        assert_eq!(decision, PermissionDecision::Deny);

        let mut trail = RecordingTrail::new();
        let err = prepare_and_record(
            decision,
            &resolved,
            ExecutionId("exec-001".into()),
            ActionId("action_1".into()),
            json!({}),
            &mut trail,
        )
        .unwrap_err();

        assert_eq!(err, PrepareError::Deny);
        assert!(trail.entries.is_empty());
    }

    // -----------------------------------------------------------------------
    // Test 5: Unavailable returns no ready action and writes no intent.
    // -----------------------------------------------------------------------

    #[test]
    fn unavailable_returns_no_ready_action_and_writes_no_intent() {
        let (_store, _availability, resolved) = resolved_read();
        let mut trail = RecordingTrail::new();
        let err = prepare_and_record(
            PermissionDecision::Unavailable,
            &resolved,
            ExecutionId("exec-001".into()),
            ActionId("action_1".into()),
            json!({}),
            &mut trail,
        )
        .unwrap_err();

        assert_eq!(err, PrepareError::Unavailable);
        assert!(trail.entries.is_empty());
    }

    // -----------------------------------------------------------------------
    // Test 6: Allow for capability A cannot prepare capability B.
    // -----------------------------------------------------------------------

    #[test]
    fn allow_for_capability_a_cannot_prepare_capability_b() {
        let (_store, _availability, resolved_b) = resolved_read();
        let mut store = TrustedManifestStore::new();
        let mut create_json = read_manifest_json();
        create_json["capability_name"] = json!("notes.note.create");
        create_json["effects"] = json!(["filesystem.write"]);
        create_json["binding"]["tool_name"] = json!("obsidian_create_note");
        let (_, digest) =
            crate::manifest::canonicalize_and_digest(&create_json.to_string()).unwrap();
        create_json["digest"] = json!(digest);
        store
            .insert(crate::manifest::verify_manifest(&create_json.to_string()).unwrap())
            .unwrap();
        let availability = ProviderAvailability::from_identities(["obsidian-local"]);
        let resolved_a = resolver::resolve_capability(
            &store,
            &availability,
            "notes.note.create",
            1,
            Some("obsidian-local"),
        )
        .unwrap();
        let decision_for_a = allow_decision_for(&resolved_a);
        assert_allowed(&decision_for_a, "notes.note.create", 1);
        let mut trail = RecordingTrail::new();

        let err = prepare_and_record(
            decision_for_a,
            &resolved_b,
            ExecutionId("exec-001".into()),
            ActionId("action_1".into()),
            json!({}),
            &mut trail,
        )
        .unwrap_err();

        assert_eq!(
            err,
            PrepareError::CapabilityIdentityMismatch {
                allowed_name: "notes.note.create".into(),
                allowed_version: 1,
                resolved_name: "notes.note.read".into(),
                resolved_version: 1,
            }
        );
        assert!(trail.entries.is_empty());
    }

    // -----------------------------------------------------------------------
    // Test 7: Mismatched capability version (AllowedIdentity vs resolved)
    //         cannot produce a ready action.
    // -----------------------------------------------------------------------

    #[test]
    fn mismatched_allow_identity_version_rejected() {
        let (_store, _availability, resolved) = resolved_read();
        let mut store = TrustedManifestStore::new();
        let mut read_v2_json = read_manifest_json();
        read_v2_json["capability_version"] = json!(2);
        let (_, digest) =
            crate::manifest::canonicalize_and_digest(&read_v2_json.to_string()).unwrap();
        read_v2_json["digest"] = json!(digest);
        store
            .insert(crate::manifest::verify_manifest(&read_v2_json.to_string()).unwrap())
            .unwrap();
        let availability = ProviderAvailability::from_identities(["obsidian-local"]);
        let resolved_v2 = resolver::resolve_capability(
            &store,
            &availability,
            "notes.note.read",
            2,
            Some("obsidian-local"),
        )
        .unwrap();
        let decision_for_v2 = allow_decision_for(&resolved_v2);
        assert_allowed(&decision_for_v2, "notes.note.read", 2);
        let mut trail = RecordingTrail::new();

        let err = prepare_and_record(
            decision_for_v2,
            &resolved,
            ExecutionId("exec-001".into()),
            ActionId("action_1".into()),
            json!({}),
            &mut trail,
        )
        .unwrap_err();

        assert_eq!(
            err,
            PrepareError::CapabilityIdentityMismatch {
                allowed_name: "notes.note.read".into(),
                allowed_version: 2,
                resolved_name: "notes.note.read".into(),
                resolved_version: 1,
            }
        );
    }

    // -----------------------------------------------------------------------
    // Test 8: Ready action carries resolved provider and digest, not
    //         substituted values.
    // -----------------------------------------------------------------------

    #[test]
    fn ready_action_carries_resolved_provider_and_digest_not_substituted_values() {
        let (_store, _availability, resolved) = resolved_read();
        let policy = allow_all_policy();
        let requirements = vec![notes_read_requirement()];
        let decision = evaluate_permission_resolved(&requirements, &resolved, &policy);

        let mut trail = RecordingTrail::new();
        let ready = prepare_and_record(
            decision,
            &resolved,
            ExecutionId("exec-001".into()),
            ActionId("action_1".into()),
            json!({"path": "test.md"}),
            &mut trail,
        )
        .unwrap();

        assert_eq!(ready.provider_identity(), "obsidian-local");
        assert_eq!(ready.manifest_digest(), resolved.manifest_digest());
        let intent = &trail.entries[0];
        assert_eq!(intent.provider_identity, "obsidian-local");
        assert_eq!(intent.manifest_digest, resolved.manifest_digest());
    }

    // -----------------------------------------------------------------------
    // Test 9: Trail append failure returns an error and no ready action.
    // -----------------------------------------------------------------------

    #[test]
    fn trail_write_failure_returns_error_and_no_ready_action() {
        let (_store, _availability, resolved) = resolved_read();
        let decision = allow_decision_for(&resolved);

        let mut trail = RecordingTrail::new();
        trail.injected_intent_error = Some(TrailError::WriteFailed("disk full".into()));

        let err = prepare_and_record(
            decision,
            &resolved,
            ExecutionId("exec-001".into()),
            ActionId("action_1".into()),
            json!({}),
            &mut trail,
        )
        .unwrap_err();

        assert_eq!(
            err,
            PrepareError::IntentWriteFailed {
                message: "disk full".into()
            }
        );
    }

    // -----------------------------------------------------------------------
    // Test 10: Flush/durability failure returns an error and no ready action.
    // -----------------------------------------------------------------------

    #[test]
    fn trail_flush_failure_returns_error_and_no_ready_action() {
        let (_store, _availability, resolved) = resolved_read();
        let decision = allow_decision_for(&resolved);

        let mut trail = RecordingTrail::new();
        trail.injected_intent_error = Some(TrailError::FlushFailed("sync_data failed".into()));

        let err = prepare_and_record(
            decision,
            &resolved,
            ExecutionId("exec-001".into()),
            ActionId("action_1".into()),
            json!({}),
            &mut trail,
        )
        .unwrap_err();

        assert_eq!(
            err,
            PrepareError::IntentFlushFailed {
                message: "sync_data failed".into()
            }
        );
    }

    // -----------------------------------------------------------------------
    // Test 11: Stable supplied identifiers appear unchanged in both the
    //          intent and ready action.
    // -----------------------------------------------------------------------

    #[test]
    fn stable_identifiers_appear_unchanged() {
        let (_store, _availability, resolved) = resolved_read();
        let policy = allow_all_policy();
        let requirements = vec![notes_read_requirement()];
        let decision = evaluate_permission_resolved(&requirements, &resolved, &policy);

        let mut trail = RecordingTrail::new();
        let exec_id = ExecutionId("eval-042".into());
        let action_id = ActionId("action_3".into());

        let ready = prepare_and_record(
            decision,
            &resolved,
            exec_id.clone(),
            action_id.clone(),
            json!({"path": "x.md"}),
            &mut trail,
        )
        .unwrap();

        assert_eq!(ready.execution_id().0, "eval-042");
        assert_eq!(ready.action_id().0, "action_3");

        let intent = &trail.entries[0];
        assert_eq!(intent.execution_id, "eval-042");
        assert_eq!(intent.action_id, "action_3");
    }

    // -----------------------------------------------------------------------
    // Test 12: Repeated tests are deterministic.
    // -----------------------------------------------------------------------

    #[test]
    fn repeated_prepare_and_record_is_deterministic() {
        let (_store, _availability, resolved) = resolved_read();
        let policy = allow_all_policy();
        let requirements = vec![notes_read_requirement()];

        for _ in 0..3 {
            let decision = evaluate_permission_resolved(&requirements, &resolved, &policy);
            let mut trail = RecordingTrail::new();
            let ready = prepare_and_record(
                decision,
                &resolved,
                ExecutionId("exec-001".into()),
                ActionId("action_1".into()),
                json!({"path": "projects/test.md"}),
                &mut trail,
            )
            .unwrap();

            assert_eq!(ready.capability_name(), "notes.note.read");
            assert_eq!(ready.capability_version(), 1);
            assert_eq!(trail.entries.len(), 1);
        }
    }

    // -----------------------------------------------------------------------
    // Test 13: No provider invocation, outcome Trail entry or result Anchor.
    // -----------------------------------------------------------------------

    #[test]
    fn prepare_and_record_produces_no_provider_invocation() {
        let (_store, _availability, resolved) = resolved_read();
        let policy = allow_all_policy();
        let requirements = vec![notes_read_requirement()];
        let decision = evaluate_permission_resolved(&requirements, &resolved, &policy);

        let mut trail = RecordingTrail::new();
        let _ready = prepare_and_record(
            decision,
            &resolved,
            ExecutionId("exec-001".into()),
            ActionId("action_1".into()),
            json!({"path": "test.md"}),
            &mut trail,
        )
        .unwrap();

        assert_eq!(trail.entries.len(), 1);
        let entry = &trail.entries[0];
        let as_value = serde_json::to_value(entry).unwrap();
        assert!(as_value.get("outcome").is_none());
        assert!(as_value.get("result").is_none());
    }

    // -----------------------------------------------------------------------
    // Test 14: File-backed durability — intent survives close and re-read.
    // -----------------------------------------------------------------------

    #[test]
    fn file_trail_writes_durable_jsonl_intent() {
        let dir = std::env::temp_dir().join("tethers-dispatch-test-file-trail");
        let _ = std::fs::create_dir_all(&dir);
        let trail_path = dir.join("trail.jsonl");

        let (_store, _availability, resolved) = resolved_read();
        let decision = allow_decision_for(&resolved);

        {
            let mut trail = FileTrail::open(&trail_path).unwrap();
            let _ready = prepare_and_record(
                decision,
                &resolved,
                ExecutionId("exec-file-001".into()),
                ActionId("action_file_1".into()),
                json!({"path": "projects/file-test.md"}),
                &mut trail,
            )
            .unwrap();
        }

        let mut contents = String::new();
        fs::File::open(&trail_path)
            .unwrap()
            .read_to_string(&mut contents)
            .unwrap();

        let parsed: Vec<serde_json::Value> = contents
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| serde_json::from_str(l).unwrap())
            .collect();

        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0]["execution_id"], "exec-file-001");
        assert_eq!(parsed[0]["action_id"], "action_file_1");
        assert_eq!(parsed[0]["capability_name"], "notes.note.read");
        assert_eq!(parsed[0]["capability_version"], 1);
        assert_eq!(parsed[0]["provider_identity"], "obsidian-local");
        assert_eq!(parsed[0]["manifest_digest"], resolved.manifest_digest());
        assert_eq!(parsed[0]["arguments"]["path"], "projects/file-test.md");

        let _ = fs::remove_file(&trail_path);
        let _ = fs::remove_dir(&dir);
    }

    // -----------------------------------------------------------------------
    // Test 15: Empty execution identifier is rejected.
    // -----------------------------------------------------------------------

    #[test]
    fn empty_execution_id_rejected() {
        let (_store, _availability, resolved) = resolved_read();
        let decision = allow_decision_for(&resolved);
        let mut trail = RecordingTrail::new();

        let err = prepare_and_record(
            decision,
            &resolved,
            ExecutionId(String::new()),
            ActionId("action_1".into()),
            json!({}),
            &mut trail,
        )
        .unwrap_err();

        assert_eq!(
            err,
            PrepareError::EmptyIdentifier {
                field: "execution_id"
            }
        );
    }

    // -----------------------------------------------------------------------
    // Test 16: Empty action identifier is rejected.
    // -----------------------------------------------------------------------

    #[test]
    fn empty_action_id_rejected() {
        let (_store, _availability, resolved) = resolved_read();
        let decision = allow_decision_for(&resolved);
        let mut trail = RecordingTrail::new();

        let err = prepare_and_record(
            decision,
            &resolved,
            ExecutionId("exec-001".into()),
            ActionId(String::new()),
            json!({}),
            &mut trail,
        )
        .unwrap_err();

        assert_eq!(err, PrepareError::EmptyIdentifier { field: "action_id" });
    }

    // -----------------------------------------------------------------------
    // Outcome tests
    // -----------------------------------------------------------------------

    // Test O1: Intent is recorded before outcome.
    #[test]
    fn intent_recorded_before_outcome() {
        let mut trail = RecordingTrail::new();
        trail
            .append_and_flush_intent(&IntentEntry {
                execution_id: "exec-001".into(),
                action_id: "action_1".into(),
                capability_name: "test.cap".into(),
                capability_version: 1,
                provider_identity: "test-prov".into(),
                manifest_digest: "sha256:abc".into(),
                arguments: json!({}),
            })
            .unwrap();

        trail
            .append_outcome(&OutcomeEntry {
                execution_id: "exec-001".into(),
                action_id: "action_1".into(),
                status: "succeeded".into(),
                result: Some(json!({"ok": true})),
                error_message: None,
                timestamp_unix_ms: 1000,
            })
            .unwrap();

        assert_eq!(trail.entries.len(), 1);
        assert_eq!(trail.outcome_entries.len(), 1);
        assert_eq!(trail.entries[0].execution_id, "exec-001");
        assert_eq!(trail.outcome_entries[0].execution_id, "exec-001");
    }

    // Test O2: Success outcome carries correct identifiers, status, and result.
    #[test]
    fn success_outcome_carries_correct_content() {
        let mut trail = RecordingTrail::new();
        trail
            .append_outcome(&OutcomeEntry {
                execution_id: "eval-abc".into(),
                action_id: "action_1".into(),
                status: "succeeded".into(),
                result: Some(json!({"status": "recorded", "project": "p", "task": "t"})),
                error_message: None,
                timestamp_unix_ms: 42,
            })
            .unwrap();

        let outcome = &trail.outcome_entries[0];
        assert_eq!(outcome.execution_id, "eval-abc");
        assert_eq!(outcome.action_id, "action_1");
        assert_eq!(outcome.status, "succeeded");
        assert_eq!(
            outcome.result,
            Some(json!({"status": "recorded", "project": "p", "task": "t"}))
        );
        assert_eq!(outcome.error_message, None);
        assert_eq!(outcome.timestamp_unix_ms, 42);
    }

    // Test O3: Failure outcome carries error message, no result.
    #[test]
    fn failure_outcome_carries_error_message_not_result() {
        let mut trail = RecordingTrail::new();
        trail
            .append_outcome(&OutcomeEntry {
                execution_id: "eval-abc".into(),
                action_id: "action_1".into(),
                status: "failed".into(),
                result: None,
                error_message: Some("executor failed as requested".into()),
                timestamp_unix_ms: 99,
            })
            .unwrap();

        let outcome = &trail.outcome_entries[0];
        assert_eq!(outcome.status, "failed");
        assert_eq!(outcome.result, None);
        assert_eq!(
            outcome.error_message,
            Some("executor failed as requested".into())
        );
        assert_eq!(outcome.timestamp_unix_ms, 99);
    }

    // Test O4: Outcome write failure does not call the executor again.
    // The RecordingTrail's injected_outcome_error simulates this.
    #[test]
    fn outcome_write_failure_returns_error_and_preserves_entries() {
        let mut trail = RecordingTrail::new();
        trail
            .append_and_flush_intent(&IntentEntry {
                execution_id: "exec-001".into(),
                action_id: "action_1".into(),
                capability_name: "test.cap".into(),
                capability_version: 1,
                provider_identity: "test-prov".into(),
                manifest_digest: "sha256:abc".into(),
                arguments: json!({}),
            })
            .unwrap();

        trail.injected_outcome_error = Some(TrailError::WriteFailed("outcome disk full".into()));

        let err = trail
            .append_outcome(&OutcomeEntry {
                execution_id: "exec-001".into(),
                action_id: "action_1".into(),
                status: "succeeded".into(),
                result: Some(json!({"ok": true})),
                error_message: None,
                timestamp_unix_ms: 1000,
            })
            .unwrap_err();

        assert_eq!(err, TrailError::WriteFailed("outcome disk full".into()));
        // Intent still present, no outcome recorded.
        assert_eq!(trail.entries.len(), 1);
        assert!(trail.outcome_entries.is_empty());
    }

    // Test O5: Deterministic OutcomeEntry serialization.
    #[test]
    fn outcome_entry_serialization_is_deterministic() {
        let entry = OutcomeEntry {
            execution_id: "exec-001".into(),
            action_id: "action_1".into(),
            status: "succeeded".into(),
            result: Some(json!({"ok": true})),
            error_message: None,
            timestamp_unix_ms: 1000,
        };

        let line1 = serde_json::to_string(&entry).unwrap();
        let line2 = serde_json::to_string(&entry).unwrap();
        assert_eq!(line1, line2);
    }

    // Test O6: FileTrail durability — intent + outcome survive close and re-read.
    #[test]
    fn file_trail_writes_durable_intent_and_outcome() {
        let dir = std::env::temp_dir().join("tethers-dispatch-test-intent-outcome");
        let _ = std::fs::create_dir_all(&dir);
        let trail_path = dir.join("trail.jsonl");

        let (_store, _availability, resolved) = resolved_read();
        let decision = allow_decision_for(&resolved);

        {
            let mut trail = FileTrail::open(&trail_path).unwrap();
            let _ready = prepare_and_record(
                decision,
                &resolved,
                ExecutionId("exec-out-001".into()),
                ActionId("action_out_1".into()),
                json!({"path": "projects/outcome-test.md"}),
                &mut trail,
            )
            .unwrap();

            trail
                .append_outcome(&OutcomeEntry {
                    execution_id: "exec-out-001".into(),
                    action_id: "action_out_1".into(),
                    status: "succeeded".into(),
                    result: Some(json!({"status": "recorded"})),
                    error_message: None,
                    timestamp_unix_ms: 5000,
                })
                .unwrap();
        }

        let mut contents = String::new();
        fs::File::open(&trail_path)
            .unwrap()
            .read_to_string(&mut contents)
            .unwrap();

        let parsed: Vec<serde_json::Value> = contents
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| serde_json::from_str(l).unwrap())
            .collect();

        assert_eq!(parsed.len(), 2, "expected intent + outcome");

        // First line: intent
        assert_eq!(parsed[0]["execution_id"], "exec-out-001");
        assert_eq!(parsed[0]["capability_name"], "notes.note.read");

        // Second line: outcome
        assert_eq!(parsed[1]["execution_id"], "exec-out-001");
        assert_eq!(parsed[1]["action_id"], "action_out_1");
        assert_eq!(parsed[1]["status"], "succeeded");
        assert_eq!(parsed[1]["result"]["status"], "recorded");
        assert_eq!(parsed[1]["timestamp_unix_ms"], 5000);

        let _ = fs::remove_file(&trail_path);
        let _ = fs::remove_dir(&dir);
    }
}
