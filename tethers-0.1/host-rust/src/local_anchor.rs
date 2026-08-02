//! Host-owned admission of one bounded local inbound event.
//!
//! Provider notifications are untrusted transport data.  This module validates
//! the reviewed envelope, persists the admission before evaluation or
//! acknowledgement, and only then exposes a generation-zero root Anchor.

use serde::de::{Deserializer, MapAccess, Visitor};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

pub const EVENT_FORMAT_VERSION: &str = "1";
pub const EVENT_NAME: &str = "file.received@1";
pub const MAX_PAYLOAD_BYTES: usize = 64 * 1024;
const RECORD_SCHEMA: &str = "tethers.local-event-admission.v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventError {
    Invalid(String),
    Io(String),
    Corrupt(String),
}

impl std::fmt::Display for EventError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for EventError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InboundEvent {
    pub event_id: String,
    pub event_name: String,
    pub provider_identity: String,
    pub installed_plug_id: String,
    pub session_id: String,
    pub occurred_at_unix_ms: u64,
    pub payload: Value,
    pub payload_digest: String,
    pub source_relative_path: Option<String>,
    pub generation: u32,
}

impl InboundEvent {
    pub fn from_json(input: &str) -> Result<Self, EventError> {
        let mut deserializer = serde_json::Deserializer::from_str(input);
        let value = deserializer
            .deserialize_map(StrictObjectVisitor)
            .map_err(|e| EventError::Invalid(format!("event JSON: {e}")))?;
        deserializer
            .end()
            .map_err(|e| EventError::Invalid(format!("event JSON trailing data: {e}")))?;
        let object = value
            .as_object()
            .ok_or_else(|| EventError::Invalid("event envelope must be an object".into()))?;
        let allowed: HashSet<&str> = [
            "event_format_version",
            "event_id",
            "event_name",
            "provider_identity",
            "installed_plug_id",
            "session_id",
            "occurred_at_unix_ms",
            "payload",
            "payload_digest",
            "source_relative_path",
            "generation",
        ]
        .into_iter()
        .collect();
        if object.keys().any(|key| !allowed.contains(key.as_str())) {
            return Err(EventError::Invalid("unknown event envelope field".into()));
        }
        if object.get("event_format_version").and_then(Value::as_str) != Some(EVENT_FORMAT_VERSION)
        {
            return Err(EventError::Invalid(
                "unsupported event format version".into(),
            ));
        }
        let event_name = string_field(object, "event_name")?;
        if event_name != EVENT_NAME {
            return Err(EventError::Invalid("unsupported event name".into()));
        }
        let event_id = string_field(object, "event_id")?;
        validate_id(&event_id, "event_id")?;
        let provider_identity = string_field(object, "provider_identity")?;
        let installed_plug_id = string_field(object, "installed_plug_id")?;
        let session_id = string_field(object, "session_id")?;
        validate_id(&session_id, "session_id")?;
        let occurred_at_unix_ms = object
            .get("occurred_at_unix_ms")
            .and_then(Value::as_u64)
            .ok_or_else(|| EventError::Invalid("occurred_at_unix_ms must be an integer".into()))?;
        let payload = object
            .get("payload")
            .cloned()
            .ok_or_else(|| EventError::Invalid("missing payload".into()))?;
        let canonical_payload = serde_json_canonicalizer::to_vec(&payload)
            .map_err(|e| EventError::Invalid(format!("payload canonicalization: {e}")))?;
        let digest = sha256_digest(&canonical_payload);
        if canonical_payload.len() > MAX_PAYLOAD_BYTES {
            return Err(EventError::Invalid("payload exceeds bound".into()));
        }
        let payload_digest = string_field(object, "payload_digest")?;
        if payload_digest != digest {
            return Err(EventError::Invalid("payload digest mismatch".into()));
        }
        let source_relative_path = object
            .get("source_relative_path")
            .map(|value| {
                value
                    .as_str()
                    .ok_or_else(|| {
                        EventError::Invalid("source_relative_path must be a string".into())
                    })
                    .and_then(validate_relative_path)
            })
            .transpose()?;
        let generation = object
            .get("generation")
            .and_then(Value::as_u64)
            .unwrap_or(0);
        if generation > 8 {
            return Err(EventError::Invalid("causal generation exceeds 8".into()));
        }
        Ok(Self {
            event_id,
            event_name,
            provider_identity,
            installed_plug_id,
            session_id,
            occurred_at_unix_ms,
            payload,
            payload_digest,
            source_relative_path,
            generation: generation as u32,
        })
    }

    pub fn canonical_digest(&self) -> Result<String, EventError> {
        let envelope = serde_json::json!({
            "event_format_version": EVENT_FORMAT_VERSION,
            "event_id": self.event_id,
            "event_name": self.event_name,
            "provider_identity": self.provider_identity,
            "installed_plug_id": self.installed_plug_id,
            "session_id": self.session_id,
            "occurred_at_unix_ms": self.occurred_at_unix_ms,
            "payload": self.payload,
            "payload_digest": self.payload_digest,
            "source_relative_path": self.source_relative_path,
            "generation": self.generation,
        });
        let bytes = serde_json_canonicalizer::to_vec(&envelope)
            .map_err(|e| EventError::Invalid(format!("event canonicalization: {e}")))?;
        Ok(sha256_digest(&bytes))
    }
}

struct StrictObjectVisitor;

impl<'de> Visitor<'de> for StrictObjectVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a JSON object without duplicate fields")
    }

    fn visit_map<A>(self, mut access: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut object = serde_json::Map::new();
        while let Some(key) = access.next_key::<String>()? {
            if object.contains_key(&key) {
                return Err(serde::de::Error::custom(format!("duplicate field: {key}")));
            }
            object.insert(key, access.next_value()?);
        }
        Ok(Value::Object(object))
    }
}

fn string_field(object: &serde_json::Map<String, Value>, name: &str) -> Result<String, EventError> {
    object
        .get(name)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| EventError::Invalid(format!("{name} must be a string")))
}

fn validate_id(value: &str, name: &str) -> Result<(), EventError> {
    if value.is_empty()
        || value.len() > 256
        || value.chars().any(|c| c.is_control() || c.is_whitespace())
    {
        return Err(EventError::Invalid(format!("invalid {name}")));
    }
    Ok(())
}

fn validate_relative_path(value: &str) -> Result<String, EventError> {
    if value.is_empty()
        || value.starts_with('/')
        || value.contains(':')
        || value.contains('\\')
        || value
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err(EventError::Invalid(
            "source path is not a safe relative path".into(),
        ));
    }
    Ok(value.to_owned())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AdmissionState {
    Admitted,
    Conflict,
    Refused,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AdmissionRecord {
    schema: String,
    event_id: String,
    canonical_event_digest: String,
    provider_identity: String,
    installed_plug_id: String,
    session_id: String,
    event_name: String,
    state: AdmissionState,
    reason: String,
    root_anchor_id: Option<String>,
    admitted_at_unix_ms: u64,
    record_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdmissionResult {
    Admitted { root_anchor_id: String },
    Duplicate { root_anchor_id: String },
    Conflict,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmissionBinding {
    pub installed_plug_id: String,
    pub provider_identity: String,
    pub session_id: String,
    pub event_name: String,
    pub source_root: PathBuf,
}

pub struct AdmissionStore {
    root: PathBuf,
    records: HashMap<String, AdmissionRecord>,
}

impl AdmissionStore {
    pub fn open(root: impl Into<PathBuf>) -> Result<Self, EventError> {
        let root = root.into();
        fs::create_dir_all(&root).map_err(io_error)?;
        let mut records = HashMap::new();
        for entry in fs::read_dir(&root).map_err(io_error)? {
            let entry = entry.map_err(io_error)?;
            if !entry.file_type().map_err(io_error)?.is_file() {
                return Err(EventError::Corrupt("unexpected admission entry".into()));
            }
            let mut bytes = Vec::new();
            File::open(entry.path())
                .map_err(io_error)?
                .read_to_end(&mut bytes)
                .map_err(io_error)?;
            let record: AdmissionRecord = serde_json::from_slice(&bytes)
                .map_err(|e| EventError::Corrupt(format!("record JSON: {e}")))?;
            if record.schema != RECORD_SCHEMA || record.record_digest != digest_record(&record)? {
                return Err(EventError::Corrupt(
                    "admission record integrity failure".into(),
                ));
            }
            if record.state == AdmissionState::Conflict {
                continue;
            }
            if records.insert(record.event_id.clone(), record).is_some() {
                return Err(EventError::Corrupt("duplicate event record".into()));
            }
        }
        Ok(Self { root, records })
    }

    pub fn admit(
        &mut self,
        event: &InboundEvent,
        now_unix_ms: u64,
    ) -> Result<AdmissionResult, EventError> {
        let digest = event.canonical_digest()?;
        if let Some(existing) = self.records.get(&event.event_id) {
            if existing.canonical_event_digest == digest {
                return Ok(AdmissionResult::Duplicate {
                    root_anchor_id: existing.root_anchor_id.clone().ok_or_else(|| {
                        EventError::Corrupt("admitted record lacks root Anchor".into())
                    })?,
                });
            }
            self.write_record(&record_for(
                event,
                &digest,
                AdmissionState::Conflict,
                "same event ID with different digest",
                None,
                now_unix_ms,
            )?)?;
            return Ok(AdmissionResult::Conflict);
        }
        let root_anchor_id = format!("anchor/{}/0", event.event_id);
        let record = record_for(
            event,
            &digest,
            AdmissionState::Admitted,
            "accepted",
            Some(root_anchor_id.clone()),
            now_unix_ms,
        )?;
        self.write_record(&record)?;
        Ok(AdmissionResult::Admitted { root_anchor_id })
    }

    pub fn admit_verified(
        &mut self,
        event: &InboundEvent,
        binding: &AdmissionBinding,
        now_unix_ms: u64,
    ) -> Result<AdmissionResult, EventError> {
        if event.installed_plug_id != binding.installed_plug_id
            || event.provider_identity != binding.provider_identity
            || event.session_id != binding.session_id
            || event.event_name != binding.event_name
        {
            return Err(EventError::Invalid(
                "event binding identity mismatch".into(),
            ));
        }
        if let Some(relative) = &event.source_relative_path {
            let path = binding.source_root.join(relative);
            let canonical_root = fs::canonicalize(&binding.source_root).map_err(io_error)?;
            let canonical_path = fs::canonicalize(&path).map_err(io_error)?;
            if !canonical_path.starts_with(&canonical_root) {
                return Err(EventError::Invalid(
                    "event source escapes approved scope".into(),
                ));
            }
        }
        self.admit(event, now_unix_ms)
    }

    fn write_record(&mut self, record: &AdmissionRecord) -> Result<(), EventError> {
        let name = format!("{}.json", safe_filename(&record.event_id));
        let destination = self.root.join(name);
        if destination.exists() && record.state == AdmissionState::Conflict {
            let conflict = self.root.join(format!(
                "{}.conflict-{}.json",
                safe_filename(&record.event_id),
                safe_filename(&record.record_digest)
            ));
            return atomic_create(&conflict, record).and_then(|_| Ok(()));
        }
        if destination.exists() {
            return Err(EventError::Corrupt(
                "admission record already exists".into(),
            ));
        }
        atomic_create(&destination, record)?;
        self.records.insert(record.event_id.clone(), record.clone());
        Ok(())
    }
}

fn record_for(
    event: &InboundEvent,
    digest: &str,
    state: AdmissionState,
    reason: &str,
    root_anchor_id: Option<String>,
    now: u64,
) -> Result<AdmissionRecord, EventError> {
    let mut record = AdmissionRecord {
        schema: RECORD_SCHEMA.into(),
        event_id: event.event_id.clone(),
        canonical_event_digest: digest.into(),
        provider_identity: event.provider_identity.clone(),
        installed_plug_id: event.installed_plug_id.clone(),
        session_id: event.session_id.clone(),
        event_name: event.event_name.clone(),
        state,
        reason: reason.into(),
        root_anchor_id,
        admitted_at_unix_ms: now,
        record_digest: String::new(),
    };
    record.record_digest = digest_record(&record)?;
    Ok(record)
}

fn digest_record(record: &AdmissionRecord) -> Result<String, EventError> {
    let mut value = serde_json::to_value(record).map_err(|e| EventError::Corrupt(e.to_string()))?;
    value
        .as_object_mut()
        .ok_or_else(|| EventError::Corrupt("record not object".into()))?
        .remove("record_digest");
    let bytes = serde_json_canonicalizer::to_vec(&value)
        .map_err(|e| EventError::Corrupt(format!("record canonicalization: {e}")))?;
    Ok(sha256_digest(&bytes))
}

fn atomic_create(path: &Path, record: &AdmissionRecord) -> Result<(), EventError> {
    let temp = path.with_extension("tmp");
    let bytes = serde_json::to_vec(record).map_err(|e| EventError::Io(e.to_string()))?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)
        .map_err(io_error)?;
    file.write_all(&bytes).map_err(io_error)?;
    file.sync_all().map_err(io_error)?;
    drop(file);
    fs::rename(&temp, path).map_err(io_error)
}

fn safe_filename(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
fn sha256_digest(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("sha256:{:x}", hasher.finalize())
}
fn io_error(error: std::io::Error) -> EventError {
    EventError::Io(error.to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RootAnchor {
    pub event_id: String,
    pub event_name: String,
    pub generation: u32,
    pub occurred_at_unix_ms: u64,
    pub facts: Value,
}

pub fn root_anchor(event: &InboundEvent, anchor_id: &str) -> RootAnchor {
    RootAnchor {
        event_id: anchor_id.to_owned(),
        event_name: event.event_name.clone(),
        generation: 0,
        occurred_at_unix_ms: event.occurred_at_unix_ms,
        facts: event.payload.clone(),
    }
}

/// Host boundary used by a local Socket/session adapter.  The acknowledgement
/// callback is deliberately owned by this boundary and is invoked only after
/// the admission record has been atomically published.
pub struct LocalAnchorCoordinator {
    store: AdmissionStore,
    binding: AdmissionBinding,
}

impl LocalAnchorCoordinator {
    pub fn open(
        store_root: impl Into<PathBuf>,
        binding: AdmissionBinding,
    ) -> Result<Self, EventError> {
        Ok(Self {
            store: AdmissionStore::open(store_root)?,
            binding,
        })
    }

    pub fn admit_notification<A>(
        &mut self,
        event_json: &str,
        now_unix_ms: u64,
        acknowledge: A,
    ) -> Result<(AdmissionResult, RootAnchor), EventError>
    where
        A: FnOnce(&str) -> Result<(), EventError>,
    {
        let event = InboundEvent::from_json(event_json)?;
        let admission = self
            .store
            .admit_verified(&event, &self.binding, now_unix_ms)?;
        let anchor_id = match &admission {
            AdmissionResult::Admitted { root_anchor_id }
            | AdmissionResult::Duplicate { root_anchor_id } => root_anchor_id.clone(),
            AdmissionResult::Conflict => {
                return Err(EventError::Invalid("conflicting event identity".into()))
            }
        };
        acknowledge(&anchor_id)?;
        Ok((admission, root_anchor(&event, &anchor_id)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;
    use std::time::UNIX_EPOCH;

    fn event(id: &str, payload: Value) -> InboundEvent {
        let digest = sha256_digest(&serde_json_canonicalizer::to_vec(&payload).unwrap());
        InboundEvent {
            event_id: id.into(),
            event_name: EVENT_NAME.into(),
            provider_identity: "file-tools".into(),
            installed_plug_id: "plug-1".into(),
            session_id: "session-1".into(),
            occurred_at_unix_ms: 1,
            payload,
            payload_digest: digest,
            source_relative_path: Some("in/a.txt".into()),
            generation: 0,
        }
    }
    fn temp() -> PathBuf {
        std::env::temp_dir().join(format!(
            "tethers-m5-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn same_id_same_digest_is_duplicate_after_restart() {
        let root = temp();
        let e = event("evt-1", serde_json::json!({"path":"in/a.txt"}));
        let mut store = AdmissionStore::open(&root).unwrap();
        assert!(matches!(
            store.admit(&e, 1).unwrap(),
            AdmissionResult::Admitted { .. }
        ));
        drop(store);
        let mut reloaded = AdmissionStore::open(&root).unwrap();
        assert!(matches!(
            reloaded.admit(&e, 2).unwrap(),
            AdmissionResult::Duplicate { .. }
        ));
        let _ = fs::remove_dir_all(root);
    }
    #[test]
    fn same_id_different_digest_is_conflict() {
        let root = temp();
        let e = event("evt-1", serde_json::json!({"path":"in/a.txt"}));
        let mut store = AdmissionStore::open(&root).unwrap();
        store.admit(&e, 1).unwrap();
        let changed = event("evt-1", serde_json::json!({"path":"in/b.txt"}));
        assert_eq!(store.admit(&changed, 2).unwrap(), AdmissionResult::Conflict);
        let _ = fs::remove_dir_all(root);
    }
    #[test]
    fn root_anchor_is_generation_zero() {
        let e = event("evt-1", serde_json::json!({"path":"in/a.txt"}));
        assert_eq!(root_anchor(&e, "anchor/evt-1/0").generation, 0);
    }

    #[test]
    fn notification_acknowledges_only_after_admission() {
        let root = temp();
        let source = root.join("source");
        fs::create_dir_all(&source).unwrap();
        fs::write(source.join("a.txt"), b"fixture").unwrap();
        let event = event("evt-ack", serde_json::json!({ "path": "in/a.txt" }));
        let payload_digest = event.payload_digest.clone();
        let json = serde_json::json!({
            "event_format_version": EVENT_FORMAT_VERSION,
            "event_id": event.event_id,
            "event_name": EVENT_NAME,
            "provider_identity": event.provider_identity,
            "installed_plug_id": event.installed_plug_id,
            "session_id": event.session_id,
            "occurred_at_unix_ms": 1,
            "payload": event.payload,
            "payload_digest": payload_digest,
            "source_relative_path": "a.txt",
            "generation": 0
        });
        let binding = AdmissionBinding {
            installed_plug_id: "plug-1".into(),
            provider_identity: "file-tools".into(),
            session_id: "session-1".into(),
            event_name: EVENT_NAME.into(),
            source_root: source,
        };
        let mut coordinator =
            LocalAnchorCoordinator::open(root.join("admission"), binding).unwrap();
        let mut acked = false;
        let (result, anchor) = coordinator
            .admit_notification(&json.to_string(), 2, |id| {
                assert!(root
                    .join("admission")
                    .join(format!("{}.json", safe_filename("evt-ack")))
                    .exists());
                assert_eq!(id, "anchor/evt-ack/0");
                acked = true;
                Ok(())
            })
            .unwrap();
        assert!(acked);
        assert!(matches!(result, AdmissionResult::Admitted { .. }));
        assert_eq!(anchor.generation, 0);
        let _ = fs::remove_dir_all(root);
    }
}
