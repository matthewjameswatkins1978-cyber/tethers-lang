//! `tethers.authority/1` frame contract for the Tethers Authority Gate.
//!
//! Newline-delimited JSON over persistent local stdio. Every frame carries an
//! explicit schema, a request identity, and either a result or a structured
//! error. Unknown required semantics are rejected; nothing is guessed.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashSet;
use std::fmt;

/// Authority protocol identity. Independent of product, Core, and CLI versions.
pub const AUTHORITY_PROTOCOL: &str = "tethers.authority/1";

/// Bound on one raw frame (including the trailing newline handling budget).
pub const MAX_FRAME_BYTES: usize = 1_048_576;

/// Bound on the serialised `payload` object alone.
pub const MAX_PAYLOAD_BYTES: usize = 524_288;

/// Bound on `request_id` length.
pub const MAX_REQUEST_ID_BYTES: usize = 128;

/// Bound on a structured error message.
pub const MAX_ERROR_MESSAGE_BYTES: usize = 2_048;

/// Maximum remembered request identities before the session refuses new ones.
pub const MAX_REMEMBERED_REQUEST_IDS: usize = 8_192;

/// Bound on collections returned by `status`.
pub const MAX_STATUS_ENTRIES: usize = 128;

/// Operations understood by the Gate.
pub const OP_HELLO: &str = "hello";
pub const OP_PREPARE: &str = "prepare";
pub const OP_APPROVAL_DECISION: &str = "approval_decision";
pub const OP_COMMIT: &str = "commit";
pub const OP_OUTCOME: &str = "outcome";
pub const OP_STATUS: &str = "status";
pub const OP_SHUTDOWN: &str = "shutdown";

/// Host-authority booleans a caller must never supply as if Tethers believed them.
/// Checked on `prepare` payloads (including nested plan material). The
/// `approval_decision` operation legitimately carries its own `decision` field.
pub const FORBIDDEN_AUTHORITY_KEYS: &[&str] = &[
    "permission",
    "within_scope",
    "trusted",
    "approved",
    "authority_granted",
    "granted",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrameError {
    EmptyFrame,
    Oversized { bytes: usize },
    InvalidJson,
    DuplicateField(&'static str),
    NotAnObject,
    MissingField(&'static str),
    WrongType { field: &'static str },
    SchemaUnsupported { schema: String },
    RequestIdInvalid,
    RequestIdDuplicate,
    RequestIdSpaceExhausted,
    UnknownOperation { operation: String },
    PayloadOversized { bytes: usize },
    PayloadInvalid { code: &'static str, message: String },
    ForbiddenAuthorityKey { field: String },
}

impl fmt::Display for FrameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFrame => write!(f, "empty frame"),
            Self::Oversized { bytes } => write!(f, "frame exceeds {bytes} bytes"),
            Self::InvalidJson => write!(f, "frame is not valid JSON"),
            Self::DuplicateField(field) => write!(f, "duplicate field: {field}"),
            Self::NotAnObject => write!(f, "frame must be a JSON object"),
            Self::MissingField(field) => write!(f, "missing field: {field}"),
            Self::WrongType { field } => write!(f, "field has the wrong type: {field}"),
            Self::SchemaUnsupported { schema } => {
                write!(f, "unsupported schema: {schema}")
            }
            Self::RequestIdInvalid => write!(f, "request_id is invalid"),
            Self::RequestIdDuplicate => write!(f, "request_id was already used"),
            Self::RequestIdSpaceExhausted => write!(f, "request_id space exhausted"),
            Self::UnknownOperation { operation } => {
                write!(f, "unknown operation: {operation}")
            }
            Self::PayloadOversized { bytes } => write!(f, "payload exceeds {bytes} bytes"),
            Self::PayloadInvalid { code, message } => write!(f, "{code}: {message}"),
            Self::ForbiddenAuthorityKey { field } => {
                write!(f, "caller must not supply authority field: {field}")
            }
        }
    }
}

impl FrameError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::EmptyFrame => "frame.empty",
            Self::Oversized { .. } => "frame.oversized",
            Self::InvalidJson => "frame.invalid_json",
            Self::DuplicateField(_) => "frame.duplicate_field",
            Self::NotAnObject => "frame.not_object",
            Self::MissingField(_) => "frame.missing_field",
            Self::WrongType { .. } => "frame.wrong_type",
            Self::SchemaUnsupported { .. } => "frame.unsupported_schema",
            Self::RequestIdInvalid => "frame.request_id_invalid",
            Self::RequestIdDuplicate => "frame.request_id_duplicate",
            Self::RequestIdSpaceExhausted => "frame.request_id_space_exhausted",
            Self::UnknownOperation { .. } => "frame.unknown_operation",
            Self::PayloadOversized { .. } => "frame.payload_oversized",
            Self::PayloadInvalid { .. } => "frame.payload_invalid",
            Self::ForbiddenAuthorityKey { .. } => "frame.forbidden_authority_key",
        }
    }
}

/// One inbound authority request.
#[derive(Debug, Clone)]
pub struct AuthorityRequest {
    pub schema: String,
    pub request_id: String,
    pub operation: String,
    pub payload: Map<String, Value>,
}

/// One outbound authority response. Exactly one of `result` / `error` is set.
#[derive(Debug, Clone, Serialize)]
pub struct AuthorityResponse {
    pub schema: String,
    pub request_id: String,
    pub status: ResponseStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ResponseError>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStatus {
    Ok,
    Error,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResponseError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl AuthorityResponse {
    pub fn ok(request_id: impl Into<String>, result: Value) -> Self {
        Self {
            schema: AUTHORITY_PROTOCOL.to_owned(),
            request_id: request_id.into(),
            status: ResponseStatus::Ok,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(
        request_id: impl Into<String>,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        let message = message.into();
        Self {
            schema: AUTHORITY_PROTOCOL.to_owned(),
            request_id: request_id.into(),
            status: ResponseStatus::Error,
            result: None,
            error: Some(ResponseError {
                code: code.into(),
                message: truncate_message(&message),
                data: None,
            }),
        }
    }

    pub fn error_with_data(
        request_id: impl Into<String>,
        code: impl Into<String>,
        message: impl Into<String>,
        data: Value,
    ) -> Self {
        let message = message.into();
        Self {
            schema: AUTHORITY_PROTOCOL.to_owned(),
            request_id: request_id.into(),
            status: ResponseStatus::Error,
            result: None,
            error: Some(ResponseError {
                code: code.into(),
                message: truncate_message(&message),
                data: Some(data),
            }),
        }
    }

    pub fn from_frame_error(request_id: &str, error: &FrameError) -> Self {
        Self::error(request_id, error.code(), error.to_string())
    }

    pub fn to_json_line(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| {
            r#"{"schema":"tethers.authority/1","request_id":"","status":"error","error":{"code":"frame.response_serialize_failed","message":"response could not be serialised"}}"#.to_owned()
        })
    }
}

fn truncate_message(message: &str) -> String {
    if message.len() <= MAX_ERROR_MESSAGE_BYTES {
        message.to_owned()
    } else {
        let mut end = MAX_ERROR_MESSAGE_BYTES;
        while !message.is_char_boundary(end) {
            end -= 1;
        }
        message[..end].to_owned()
    }
}

/// Session-scoped request identity memory. Rejects exact duplicates.
#[derive(Debug, Default)]
pub struct RequestIdMemory {
    seen: HashSet<String>,
}

impl RequestIdMemory {
    pub fn accept(&mut self, request_id: &str) -> Result<(), FrameError> {
        if self.seen.len() >= MAX_REMEMBERED_REQUEST_IDS && !self.seen.contains(request_id) {
            return Err(FrameError::RequestIdSpaceExhausted);
        }
        if !self.seen.insert(request_id.to_owned()) {
            return Err(FrameError::RequestIdDuplicate);
        }
        Ok(())
    }
}

fn parse_string_field(
    object: &Map<String, Value>,
    field: &'static str,
) -> Result<String, FrameError> {
    let value = object.get(field).ok_or(FrameError::MissingField(field))?;
    value
        .as_str()
        .map(str::to_owned)
        .ok_or(FrameError::WrongType { field })
}

/// Parse and strictly validate one raw frame line.
pub fn parse_frame(line: &str) -> Result<AuthorityRequest, FrameError> {
    let trimmed = line.trim_end_matches(['\r', '\n']);
    if trimmed.is_empty() {
        return Err(FrameError::EmptyFrame);
    }
    if trimmed.len() > MAX_FRAME_BYTES {
        return Err(FrameError::Oversized {
            bytes: trimmed.len(),
        });
    }
    let value: Value = serde_json::from_str(trimmed).map_err(|_| FrameError::InvalidJson)?;
    let object = value.as_object().ok_or(FrameError::NotAnObject)?;

    reject_duplicate_top_level(trimmed, object)?;

    let schema = parse_string_field(object, "schema")?;
    if schema != AUTHORITY_PROTOCOL {
        return Err(FrameError::SchemaUnsupported { schema });
    }
    let request_id = parse_string_field(object, "request_id")?;
    validate_request_id(&request_id)?;
    let operation = parse_string_field(object, "operation")?;
    if !is_known_operation(&operation) {
        return Err(FrameError::UnknownOperation { operation });
    }
    let payload_value = object
        .get("payload")
        .ok_or(FrameError::MissingField("payload"))?;
    let payload = payload_value
        .as_object()
        .ok_or(FrameError::WrongType { field: "payload" })?
        .clone();
    let payload_bytes = serde_json::to_vec(&payload)
        .map_err(|_| FrameError::InvalidJson)?
        .len();
    if payload_bytes > MAX_PAYLOAD_BYTES {
        return Err(FrameError::PayloadOversized {
            bytes: payload_bytes,
        });
    }
    if operation == OP_PREPARE {
        reject_forbidden_authority_keys(&payload)?;
    }

    Ok(AuthorityRequest {
        schema,
        request_id,
        operation,
        payload,
    })
}

fn is_known_operation(operation: &str) -> bool {
    matches!(
        operation,
        OP_HELLO
            | OP_PREPARE
            | OP_APPROVAL_DECISION
            | OP_COMMIT
            | OP_OUTCOME
            | OP_STATUS
            | OP_SHUTDOWN
    )
}

fn validate_request_id(request_id: &str) -> Result<(), FrameError> {
    if request_id.is_empty()
        || request_id.len() > MAX_REQUEST_ID_BYTES
        || request_id.chars().any(char::is_control)
    {
        return Err(FrameError::RequestIdInvalid);
    }
    Ok(())
}

fn reject_forbidden_authority_keys(payload: &Map<String, Value>) -> Result<(), FrameError> {
    for key in payload.keys() {
        if FORBIDDEN_AUTHORITY_KEYS.contains(&key.as_str()) {
            return Err(FrameError::ForbiddenAuthorityKey { field: key.clone() });
        }
    }
    Ok(())
}

/// Detect duplicate top-level fields. serde_json keeps the last duplicate; the
/// Gate must refuse rather than silently reinterpret the frame.
fn reject_duplicate_top_level(raw: &str, _object: &Map<String, Value>) -> Result<(), FrameError> {
    struct FrameKeyProbe;

    impl<'de> serde::Deserialize<'de> for FrameKeyProbe {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            struct ProbeVisitor;

            impl<'de> serde::de::Visitor<'de> for ProbeVisitor {
                type Value = FrameKeyProbe;

                fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                    formatter.write_str("a JSON object")
                }

                fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::MapAccess<'de>,
                {
                    let mut seen: HashSet<String> = HashSet::new();
                    while let Some(key) = map.next_key::<String>()? {
                        if !seen.insert(key.clone()) {
                            let field: &'static str = match key.as_str() {
                                "schema" => "schema",
                                "request_id" => "request_id",
                                "operation" => "operation",
                                "payload" => "payload",
                                _ => "unknown",
                            };
                            return Err(serde::de::Error::custom(format!(
                                "duplicate field: {field}"
                            )));
                        }
                        map.next_value::<serde::de::IgnoredAny>()?;
                    }
                    Ok(FrameKeyProbe)
                }
            }

            deserializer.deserialize_map(ProbeVisitor)
        }
    }

    let mut deserializer = serde_json::Deserializer::from_str(raw);
    match FrameKeyProbe::deserialize(&mut deserializer) {
        Ok(_) => Ok(()),
        Err(err) => {
            let message = err.to_string();
            if message.contains("duplicate field: schema") {
                Err(FrameError::DuplicateField("schema"))
            } else if message.contains("duplicate field: request_id") {
                Err(FrameError::DuplicateField("request_id"))
            } else if message.contains("duplicate field: operation") {
                Err(FrameError::DuplicateField("operation"))
            } else if message.contains("duplicate field: payload") {
                Err(FrameError::DuplicateField("payload"))
            } else if message.contains("duplicate field: unknown") {
                Err(FrameError::DuplicateField("payload"))
            } else {
                Ok(())
            }
        }
    }
}

/// Parse a `prepare` payload.
#[derive(Debug, Clone)]
pub struct PreparePayload {
    pub tether_id: String,
    pub tether_version: String,
    pub evaluation_id: String,
    pub event_id: String,
    pub plan: Value,
    pub action_id: String,
    pub observations: Observations,
}

/// Bounded external physical observations supplied by the Host as input facts.
/// These are never authority decisions.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Observations {
    /// Physically observed available provider identities. `None` means the
    /// Gate uses configured provider identities as the availability set.
    pub available_provider_identities: Option<Vec<String>>,
}

impl Observations {
    pub fn parse(payload: &Map<String, Value>) -> Result<Self, FrameError> {
        let Some(value) = payload.get("observations") else {
            return Ok(Self::default());
        };
        let object = value.as_object().ok_or(FrameError::WrongType {
            field: "observations",
        })?;
        for key in object.keys() {
            if key != "available_provider_identities" {
                return Err(FrameError::PayloadInvalid {
                    code: "prepare.unknown_observation",
                    message: format!("unknown observation field: {key}"),
                });
            }
        }
        let available = match object.get("available_provider_identities") {
            None => None,
            Some(Value::Null) => None,
            Some(Value::Array(items)) => {
                let mut identities = Vec::with_capacity(items.len());
                for item in items {
                    let identity = item.as_str().ok_or(FrameError::PayloadInvalid {
                        code: "prepare.invalid_observation",
                        message: "available_provider_identities must be strings".to_owned(),
                    })?;
                    if identity.is_empty() || identity.len() > 256 {
                        return Err(FrameError::PayloadInvalid {
                            code: "prepare.invalid_observation",
                            message: "provider identity length is out of bounds".to_owned(),
                        });
                    }
                    identities.push(identity.to_owned());
                }
                if identities.len() > 64 {
                    return Err(FrameError::PayloadInvalid {
                        code: "prepare.invalid_observation",
                        message: "too many provider identities".to_owned(),
                    });
                }
                Some(identities)
            }
            Some(_) => {
                return Err(FrameError::WrongType {
                    field: "observations.available_provider_identities",
                })
            }
        };
        Ok(Self {
            available_provider_identities: available,
        })
    }
}

pub fn parse_prepare_payload(payload: &Map<String, Value>) -> Result<PreparePayload, FrameError> {
    let tether_id = required_nonempty_string(payload, "tether_id")?;
    let tether_version = required_nonempty_string(payload, "tether_version")?;
    let evaluation_id = required_nonempty_string(payload, "evaluation_id")?;
    let event_id = required_nonempty_string(payload, "event_id")?;
    let action_id = required_nonempty_string(payload, "action_id")?;
    let plan = payload
        .get("plan")
        .cloned()
        .ok_or(FrameError::MissingField("plan"))?;
    if !plan.is_object() {
        return Err(FrameError::WrongType { field: "plan" });
    }
    reject_nested_authority_keys(&plan)?;
    let observations = Observations::parse(payload)?;
    Ok(PreparePayload {
        tether_id,
        tether_version,
        evaluation_id,
        event_id,
        plan,
        action_id,
        observations,
    })
}

pub fn parse_approval_payload(
    payload: &Map<String, Value>,
) -> Result<(String, String), FrameError> {
    let approval_id = required_nonempty_string(payload, "approval_id")?;
    let decision = required_nonempty_string(payload, "decision")?;
    if !matches!(decision.as_str(), "approve" | "deny" | "cancel") {
        return Err(FrameError::PayloadInvalid {
            code: "approval.invalid_decision",
            message: "decision must be approve, deny, or cancel".to_owned(),
        });
    }
    Ok((approval_id, decision))
}

pub fn parse_commit_payload(
    payload: &Map<String, Value>,
) -> Result<(String, Option<String>, Observations), FrameError> {
    parse_commit_payload_with_observations(payload)
}

pub fn parse_commit_payload_with_observations(
    payload: &Map<String, Value>,
) -> Result<(String, Option<String>, Observations), FrameError> {
    let prepared_id = required_nonempty_string(payload, "prepared_id")?;
    let approval_id = match payload.get("approval_id") {
        None | Some(Value::Null) => None,
        Some(Value::String(value)) if value.is_empty() => {
            return Err(FrameError::PayloadInvalid {
                code: "commit.invalid_approval",
                message: "approval_id must not be empty".to_owned(),
            })
        }
        Some(Value::String(value)) => Some(value.clone()),
        Some(_) => {
            return Err(FrameError::WrongType {
                field: "approval_id",
            })
        }
    };
    let observations = Observations::parse(payload)?;
    Ok((prepared_id, approval_id, observations))
}

pub fn parse_outcome_payload(payload: &Map<String, Value>) -> Result<OutcomePayload, FrameError> {
    let execution_id = required_nonempty_string(payload, "execution_id")?;
    let classification = required_nonempty_string(payload, "classification")?;
    if !matches!(
        classification.as_str(),
        "succeeded" | "failed" | "uncertain"
    ) {
        return Err(FrameError::PayloadInvalid {
            code: "outcome.invalid_classification",
            message: "classification must be succeeded, failed, or uncertain".to_owned(),
        });
    }
    let attempted = match payload.get("attempted") {
        None => true,
        Some(Value::Bool(value)) => *value,
        Some(_) => return Err(FrameError::WrongType { field: "attempted" }),
    };
    let external_execution_identity = optional_string(payload, "external_execution_identity")?;
    let result = payload
        .get("result")
        .cloned()
        .filter(|value| !value.is_null());
    let error = optional_string(payload, "error")?;
    let reason_code = optional_string(payload, "reason_code")?;
    let evidence = optional_string(payload, "evidence")?;

    match classification.as_str() {
        "succeeded" if result.is_none() => {
            return Err(FrameError::PayloadInvalid {
                code: "outcome.missing_result",
                message: "succeeded outcome requires result".to_owned(),
            })
        }
        "failed" if error.as_deref().unwrap_or_default().is_empty() => {
            return Err(FrameError::PayloadInvalid {
                code: "outcome.missing_error",
                message: "failed outcome requires error".to_owned(),
            })
        }
        _ => {}
    }
    if result.is_some() && error.is_some() {
        return Err(FrameError::PayloadInvalid {
            code: "outcome.conflicting_fields",
            message: "result and error are mutually exclusive".to_owned(),
        });
    }

    Ok(OutcomePayload {
        execution_id,
        classification,
        attempted,
        external_execution_identity,
        result,
        error,
        reason_code,
        evidence,
    })
}

#[derive(Debug, Clone)]
pub struct OutcomePayload {
    pub execution_id: String,
    pub classification: String,
    pub attempted: bool,
    pub external_execution_identity: Option<String>,
    pub result: Option<Value>,
    pub error: Option<String>,
    pub reason_code: Option<String>,
    pub evidence: Option<String>,
}

fn required_nonempty_string(
    payload: &Map<String, Value>,
    field: &'static str,
) -> Result<String, FrameError> {
    let value = payload.get(field).ok_or(FrameError::MissingField(field))?;
    let text = value.as_str().ok_or(FrameError::WrongType { field })?;
    if text.is_empty() {
        return Err(FrameError::PayloadInvalid {
            code: "frame.empty_field",
            message: format!("{field} must not be empty"),
        });
    }
    if text.len() > 4_096 {
        return Err(FrameError::PayloadOversized { bytes: text.len() });
    }
    Ok(text.to_owned())
}

fn optional_string(
    payload: &Map<String, Value>,
    field: &'static str,
) -> Result<Option<String>, FrameError> {
    match payload.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(text)) => {
            if text.len() > 16_384 {
                return Err(FrameError::PayloadOversized { bytes: text.len() });
            }
            Ok(Some(text.clone()))
        }
        Some(_) => Err(FrameError::WrongType { field }),
    }
}

fn reject_nested_authority_keys(value: &Value) -> Result<(), FrameError> {
    match value {
        Value::Object(map) => {
            for (key, nested) in map {
                if FORBIDDEN_AUTHORITY_KEYS.contains(&key.as_str()) {
                    return Err(FrameError::ForbiddenAuthorityKey { field: key.clone() });
                }
                reject_nested_authority_keys(nested)?;
            }
            Ok(())
        }
        Value::Array(items) => {
            for item in items {
                reject_nested_authority_keys(item)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

/// Deserialised helper used by tests and the stdio loop for empty payloads.
pub fn ensure_empty_payload(
    payload: &Map<String, Value>,
    operation: &str,
) -> Result<(), FrameError> {
    if payload.is_empty() {
        Ok(())
    } else {
        Err(FrameError::PayloadInvalid {
            code: "frame.unexpected_payload",
            message: format!("{operation} does not accept payload fields"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(operation: &str, payload: &str) -> String {
        format!(
            r#"{{"schema":"tethers.authority/1","request_id":"req-1","operation":"{operation}","payload":{payload}}}"#
        )
    }

    #[test]
    fn parses_hello() {
        let request = parse_frame(&frame(OP_HELLO, "{}")).unwrap();
        assert_eq!(request.operation, OP_HELLO);
        assert_eq!(request.request_id, "req-1");
    }

    #[test]
    fn rejects_unknown_operation() {
        let error = parse_frame(&frame("launch_missiles", "{}")).unwrap_err();
        assert_eq!(error.code(), "frame.unknown_operation");
    }

    #[test]
    fn rejects_wrong_schema() {
        let line =
            r#"{"schema":"tethers.plan/1","request_id":"r","operation":"hello","payload":{}}"#;
        let error = parse_frame(line).unwrap_err();
        assert_eq!(error.code(), "frame.unsupported_schema");
    }

    #[test]
    fn rejects_duplicate_request_id_field() {
        let line = r#"{"schema":"tethers.authority/1","request_id":"a","request_id":"b","operation":"hello","payload":{}}"#;
        let error = parse_frame(line).unwrap_err();
        assert_eq!(error.code(), "frame.duplicate_field");
    }

    #[test]
    fn rejects_forbidden_authority_boolean() {
        let line = frame(OP_PREPARE, r#"{"permission":true}"#);
        let error = parse_frame(&line).unwrap_err();
        assert_eq!(error.code(), "frame.forbidden_authority_key");
    }

    #[test]
    fn rejects_oversized_frame() {
        let padded = format!(
            r#"{{"schema":"tethers.authority/1","request_id":"r","operation":"hello","payload":{{"pad":"{}"}}}}"#,
            "x".repeat(MAX_FRAME_BYTES)
        );
        let error = parse_frame(&padded).unwrap_err();
        assert_eq!(error.code(), "frame.oversized");
    }

    #[test]
    fn rejects_malformed_json() {
        let error = parse_frame("{not json").unwrap_err();
        assert_eq!(error.code(), "frame.invalid_json");
    }

    #[test]
    fn rejects_empty_frame() {
        assert_eq!(parse_frame("").unwrap_err(), FrameError::EmptyFrame);
    }

    #[test]
    fn request_id_memory_rejects_duplicates() {
        let mut memory = RequestIdMemory::default();
        memory.accept("a").unwrap();
        assert_eq!(
            memory.accept("a").unwrap_err(),
            FrameError::RequestIdDuplicate
        );
    }

    #[test]
    fn prepare_payload_requires_identity_fields() {
        let line = frame(OP_PREPARE, r#"{"plan":{}}"#);
        let request = parse_frame(&line).unwrap();
        let error = parse_prepare_payload(&request.payload).unwrap_err();
        assert_eq!(error.code(), "frame.missing_field");
    }

    #[test]
    fn commit_accepts_optional_approval_and_observations() {
        let line = frame(
            OP_COMMIT,
            r#"{"prepared_id":"prep_1","observations":{"available_provider_identities":["p"]}}"#,
        );
        let request = parse_frame(&line).unwrap();
        let (prepared_id, approval, observations) = parse_commit_payload(&request.payload).unwrap();
        assert_eq!(prepared_id, "prep_1");
        assert!(approval.is_none());
        assert_eq!(
            observations.available_provider_identities,
            Some(vec!["p".to_owned()])
        );
    }

    #[test]
    fn response_serialises_ok_and_error() {
        let ok = AuthorityResponse::ok("r1", serde_json::json!({"hello": true}));
        let line = ok.to_json_line();
        assert!(line.contains("\"status\":\"ok\""));
        assert!(line.contains("tethers.authority/1"));
        let err = AuthorityResponse::error("r2", "boom", "failed");
        assert!(err.to_json_line().contains("\"status\":\"error\""));
    }
}
