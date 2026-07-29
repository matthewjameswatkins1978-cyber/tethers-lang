//! Strict public input boundary for `tethers-reference-host run`.
//!
//! The public format is intentionally smaller than the internal planner
//! request.  Callers supply one event, one immutable Facts object, and one
//! configured Tether identity; host-owned policy, capability, causal, replay,
//! and execution data never cross this boundary.

use crate::runtime_config::parse_value_no_dupes;
use serde_json::{Map, Value};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct RunInput {
    pub evaluation_id: String,
    pub tether: RunTether,
    pub event: RunEvent,
    pub facts: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunTether {
    pub id: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunEvent {
    pub id: String,
    pub name: String,
    pub data: Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunInputErrorCode {
    InvalidJson,
    UnknownField,
    MissingField,
    InvalidType,
    InvalidValue,
}

impl RunInputErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidJson => "RUN_INPUT_INVALID_JSON",
            Self::UnknownField => "RUN_INPUT_UNKNOWN_FIELD",
            Self::MissingField => "RUN_INPUT_MISSING_FIELD",
            Self::InvalidType => "RUN_INPUT_INVALID_TYPE",
            Self::InvalidValue => "RUN_INPUT_INVALID_VALUE",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunInputError {
    pub code: RunInputErrorCode,
    pub message: &'static str,
    pub field: Option<String>,
}

impl RunInputError {
    fn new(
        code: RunInputErrorCode,
        message: &'static str,
        field: impl Into<Option<String>>,
    ) -> Self {
        Self {
            code,
            message,
            field: field.into(),
        }
    }
}

impl fmt::Display for RunInputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code.as_str(), self.message)
    }
}

impl std::error::Error for RunInputError {}

/// Parse the exact public run-input format.  The shared duplicate-key parser
/// runs before any field access, including inside arbitrary event data/Facts.
pub fn parse_run_input(text: &str) -> Result<RunInput, RunInputError> {
    let value = parse_value_no_dupes(text).map_err(|_| {
        RunInputError::new(
            RunInputErrorCode::InvalidJson,
            "input must be valid JSON with no duplicate keys",
            None,
        )
    })?;
    let root = required_object(&value, "")?;
    reject_unknown(
        root,
        "",
        &[
            "format_version",
            "evaluation_id",
            "tether",
            "event",
            "facts",
        ],
    )?;

    let format_version = required_string(root, "format_version", "")?;
    if format_version != "1" {
        return Err(RunInputError::new(
            RunInputErrorCode::InvalidValue,
            "format_version must be exactly \"1\"",
            Some("/format_version".to_owned()),
        ));
    }

    let evaluation_id = required_string(root, "evaluation_id", "")?;
    require_identifier_without_whitespace(&evaluation_id, "/evaluation_id")?;

    let tether_value = required_value(root, "tether", "")?;
    let tether_object = required_object(tether_value, "/tether")?;
    reject_unknown(tether_object, "/tether", &["id", "version"])?;
    let tether_id = required_string(tether_object, "id", "/tether")?;
    require_non_empty(&tether_id, "/tether/id")?;
    let tether_version = required_string(tether_object, "version", "/tether")?;
    require_non_empty(&tether_version, "/tether/version")?;

    let event_value = required_value(root, "event", "")?;
    let event_object = required_object(event_value, "/event")?;
    reject_unknown(event_object, "/event", &["id", "name", "data"])?;
    let event_id = required_string(event_object, "id", "/event")?;
    require_identifier_without_whitespace(&event_id, "/event/id")?;
    let event_name = required_string(event_object, "name", "/event")?;
    require_non_empty(&event_name, "/event/name")?;
    let event_data = required_value(event_object, "data", "/event")?;
    required_object(event_data, "/event/data")?;

    let facts = required_value(root, "facts", "")?;
    required_object(facts, "/facts")?;

    Ok(RunInput {
        evaluation_id,
        tether: RunTether {
            id: tether_id,
            version: tether_version,
        },
        event: RunEvent {
            id: event_id,
            name: event_name,
            data: event_data.clone(),
        },
        facts: facts.clone(),
    })
}

fn required_object<'a>(
    value: &'a Value,
    pointer: &str,
) -> Result<&'a Map<String, Value>, RunInputError> {
    value.as_object().ok_or_else(|| {
        RunInputError::new(
            RunInputErrorCode::InvalidType,
            "value must be an object",
            Some(pointer.to_owned()),
        )
    })
}

fn required_value<'a>(
    object: &'a Map<String, Value>,
    name: &str,
    pointer: &str,
) -> Result<&'a Value, RunInputError> {
    object.get(name).ok_or_else(|| {
        RunInputError::new(
            RunInputErrorCode::MissingField,
            "required field is missing",
            Some(child_pointer(pointer, name)),
        )
    })
}

fn required_string(
    object: &Map<String, Value>,
    name: &str,
    pointer: &str,
) -> Result<String, RunInputError> {
    required_value(object, name, pointer)?
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| {
            RunInputError::new(
                RunInputErrorCode::InvalidType,
                "value must be a string",
                Some(child_pointer(pointer, name)),
            )
        })
}

fn reject_unknown(
    object: &Map<String, Value>,
    pointer: &str,
    allowed: &[&str],
) -> Result<(), RunInputError> {
    for name in object.keys() {
        if !allowed.contains(&name.as_str()) {
            return Err(RunInputError::new(
                RunInputErrorCode::UnknownField,
                "field is not permitted in public run input",
                Some(child_pointer(pointer, name)),
            ));
        }
    }
    Ok(())
}

fn require_non_empty(value: &str, pointer: &str) -> Result<(), RunInputError> {
    if value.is_empty() || value.trim().is_empty() {
        return Err(RunInputError::new(
            RunInputErrorCode::InvalidValue,
            "value must be a non-empty string",
            Some(pointer.to_owned()),
        ));
    }
    Ok(())
}

fn require_identifier_without_whitespace(value: &str, pointer: &str) -> Result<(), RunInputError> {
    require_non_empty(value, pointer)?;
    if value.chars().any(char::is_whitespace) {
        return Err(RunInputError::new(
            RunInputErrorCode::InvalidValue,
            "identifier must not contain whitespace",
            Some(pointer.to_owned()),
        ));
    }
    Ok(())
}

fn child_pointer(pointer: &str, name: &str) -> String {
    format!("{}/{}", pointer, name.replace('~', "~0").replace('/', "~1"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID: &str = r#"{
        "format_version":"1",
        "evaluation_id":"eval_demo_001",
        "tether":{"id":"record-completed-task","version":"demo-v1"},
        "event":{"id":"evt_demo_001","name":"coding.task_completed","data":{"project":"lantern-keeper","task":"LK-39"}},
        "facts":{"project.type":"software","task.changed_files":3}
    }"#;

    #[test]
    fn j13b_run_strict_input_shape_preserves_supplied_identifiers() {
        let input = parse_run_input(VALID).unwrap();
        assert_eq!(input.evaluation_id, "eval_demo_001");
        assert_eq!(input.event.id, "evt_demo_001");
        assert_eq!(input.tether.id, "record-completed-task");
    }

    #[test]
    fn j13b_run_rejects_duplicate_keys_at_every_depth() {
        for input in [
            r#"{"format_version":"1","format_version":"1"}"#,
            r#"{"format_version":"1","evaluation_id":"e","tether":{"id":"t","id":"t","version":"v"},"event":{"id":"x","name":"n","data":{}},"facts":{}}"#,
            r#"{"format_version":"1","evaluation_id":"e","tether":{"id":"t","version":"v"},"event":{"id":"x","name":"n","data":{"x":1,"x":2}},"facts":{}}"#,
        ] {
            assert_eq!(
                parse_run_input(input).unwrap_err().code,
                RunInputErrorCode::InvalidJson
            );
        }
    }

    #[test]
    fn j13b_run_rejects_unknown_and_host_owned_fields() {
        for (field, value) in [
            ("source", "\"untrusted\""),
            ("capabilities", "[]"),
            ("provider_identity", "\"p\""),
            ("policy", "{}"),
            ("scope", "{}"),
            ("manifest_digest", "\"sha256:x\""),
            ("approval", "{}"),
            ("generation", "0"),
            ("correlation_id", "\"x\""),
            ("causation_id", "\"x\""),
            ("replay_identity", "\"x\""),
        ] {
            let input = VALID.replacen("\n    }", &format!(",\"{field}\":{value}\n    }}"), 1);
            assert_eq!(
                parse_run_input(&input).unwrap_err().code,
                RunInputErrorCode::UnknownField
            );
        }
    }

    #[test]
    fn j13b_run_validates_every_declared_field_boundary() {
        for (input, field) in [
            (
                VALID.replace("\"format_version\":\"1\"", "\"format_version\":\"2\""),
                "/format_version",
            ),
            (
                VALID.replace(
                    "\"evaluation_id\":\"eval_demo_001\"",
                    "\"evaluation_id\":\"eval id\"",
                ),
                "/evaluation_id",
            ),
            (
                VALID.replace("\"id\":\"record-completed-task\"", "\"id\":\"\""),
                "/tether/id",
            ),
            (
                VALID.replace("\"version\":\"demo-v1\"", "\"version\":\"\""),
                "/tether/version",
            ),
            (
                VALID.replace("\"id\":\"evt_demo_001\"", "\"id\":\"evt id\""),
                "/event/id",
            ),
            (
                VALID.replace("\"name\":\"coding.task_completed\"", "\"name\":\"\""),
                "/event/name",
            ),
            (
                VALID.replace(
                    "\"data\":{\"project\":\"lantern-keeper\",\"task\":\"LK-39\"}",
                    "\"data\":[]",
                ),
                "/event/data",
            ),
            (
                VALID.replace(
                    "\"facts\":{\"project.type\":\"software\",\"task.changed_files\":3}",
                    "\"facts\":[]",
                ),
                "/facts",
            ),
        ] {
            let error = parse_run_input(&input).unwrap_err();
            assert_eq!(error.field.as_deref(), Some(field));
        }
    }
}
