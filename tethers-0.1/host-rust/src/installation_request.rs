use crate::manifest::parse_value_no_dupes;
use serde_json::{Map, Value};
use std::fmt;
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;
use uuid::Uuid;

pub const INSTALLATION_REQUEST_SCHEMA: &str = "tethers.plug-install/1";
pub const INSTALLATION_REQUEST_MAX_BYTES: usize = 16 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallationRequest {
    pub schema: String,
    pub candidate_id: String,
    pub trust: InstallationTrustRequest,
    pub conformance: InstallationConformanceRequest,
    pub installation: InstallationTargetRequest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallationTrustRequest {
    pub scope: InstallationTrustScope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallationTrustScope {
    ExactCandidate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallationConformanceRequest {
    pub allow_non_isolated_supervised_execution: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallationTargetRequest {
    pub target_state: InstallationTargetState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallationTargetState {
    Disabled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallationRequestError {
    pub code: &'static str,
    pub message: &'static str,
    pub field: Option<String>,
}

impl InstallationRequestError {
    fn invalid(message: &'static str, field: Option<String>) -> Self {
        Self {
            code: "installation_request_invalid",
            message,
            field,
        }
    }

    fn io() -> Self {
        Self {
            code: "installation_request_io",
            message: "cannot read installation request",
            field: None,
        }
    }
}

impl fmt::Display for InstallationRequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for InstallationRequestError {}

pub fn load_installation_request(
    path: &Path,
) -> Result<InstallationRequest, InstallationRequestError> {
    if !path.is_absolute() {
        return Err(InstallationRequestError::invalid(
            "installation request path must be absolute",
            None,
        ));
    }

    let metadata = fs::symlink_metadata(path).map_err(|_| InstallationRequestError::io())?;
    if !metadata.is_file() {
        return Err(InstallationRequestError::invalid(
            "installation request path must name an ordinary file",
            None,
        ));
    }

    let file = File::open(path).map_err(|_| InstallationRequestError::io())?;
    let mut bytes = Vec::with_capacity(INSTALLATION_REQUEST_MAX_BYTES + 1);
    file.take((INSTALLATION_REQUEST_MAX_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| InstallationRequestError::io())?;

    parse_installation_request_bytes(&bytes)
}

pub fn parse_installation_request_bytes(
    bytes: &[u8],
) -> Result<InstallationRequest, InstallationRequestError> {
    if bytes.len() > INSTALLATION_REQUEST_MAX_BYTES {
        return Err(InstallationRequestError::invalid(
            "installation request exceeds 16 KiB limit",
            None,
        ));
    }
    if bytes.starts_with(&[0xef, 0xbb, 0xbf]) {
        return Err(InstallationRequestError::invalid(
            "installation request contains UTF-8 BOM",
            None,
        ));
    }
    let text = std::str::from_utf8(bytes).map_err(|_| {
        InstallationRequestError::invalid("installation request is not valid UTF-8", None)
    })?;
    let value = parse_value_no_dupes(text).map_err(|_| {
        InstallationRequestError::invalid(
            "installation request must be valid JSON with no duplicate keys or trailing content",
            None,
        )
    })?;

    let root = required_object(&value, "")?;
    reject_unknown(
        root,
        "",
        &[
            "schema",
            "candidate_id",
            "trust",
            "conformance",
            "installation",
        ],
    )?;

    let schema = required_string(root, "schema", "")?;
    if schema != INSTALLATION_REQUEST_SCHEMA {
        return Err(InstallationRequestError::invalid(
            "schema must be exactly \"tethers.plug-install/1\"",
            Some("/schema".to_owned()),
        ));
    }

    let candidate_id = required_string(root, "candidate_id", "")?;
    let parsed_candidate_id = Uuid::parse_str(&candidate_id).map_err(|_| {
        InstallationRequestError::invalid(
            "candidate_id must be a canonical lowercase hyphenated UUID",
            Some("/candidate_id".to_owned()),
        )
    })?;
    if parsed_candidate_id.hyphenated().to_string() != candidate_id {
        return Err(InstallationRequestError::invalid(
            "candidate_id must be a canonical lowercase hyphenated UUID",
            Some("/candidate_id".to_owned()),
        ));
    }

    let trust_value = required_value(root, "trust", "")?;
    let trust = required_object(trust_value, "/trust")?;
    reject_unknown(trust, "/trust", &["scope"])?;
    let scope = required_string(trust, "scope", "/trust")?;
    if scope != "exact_candidate" {
        return Err(InstallationRequestError::invalid(
            "trust scope must be exactly \"exact_candidate\"",
            Some("/trust/scope".to_owned()),
        ));
    }

    let conformance_value = required_value(root, "conformance", "")?;
    let conformance = required_object(conformance_value, "/conformance")?;
    reject_unknown(
        conformance,
        "/conformance",
        &["allow_non_isolated_supervised_execution"],
    )?;
    let allow_non_isolated_supervised_execution = required_bool(
        conformance,
        "allow_non_isolated_supervised_execution",
        "/conformance",
    )?;
    if !allow_non_isolated_supervised_execution {
        return Err(InstallationRequestError::invalid(
            "non-isolated supervised execution must be explicitly approved",
            Some("/conformance/allow_non_isolated_supervised_execution".to_owned()),
        ));
    }

    let installation_value = required_value(root, "installation", "")?;
    let installation = required_object(installation_value, "/installation")?;
    reject_unknown(installation, "/installation", &["target_state"])?;
    let target_state = required_string(installation, "target_state", "/installation")?;
    if target_state != "disabled" {
        return Err(InstallationRequestError::invalid(
            "installation target_state must be exactly \"disabled\"",
            Some("/installation/target_state".to_owned()),
        ));
    }

    Ok(InstallationRequest {
        schema,
        candidate_id,
        trust: InstallationTrustRequest {
            scope: InstallationTrustScope::ExactCandidate,
        },
        conformance: InstallationConformanceRequest {
            allow_non_isolated_supervised_execution,
        },
        installation: InstallationTargetRequest {
            target_state: InstallationTargetState::Disabled,
        },
    })
}

fn required_object<'a>(
    value: &'a Value,
    pointer: &str,
) -> Result<&'a Map<String, Value>, InstallationRequestError> {
    value.as_object().ok_or_else(|| {
        InstallationRequestError::invalid("value must be an object", Some(pointer.to_owned()))
    })
}

fn required_value<'a>(
    object: &'a Map<String, Value>,
    name: &str,
    pointer: &str,
) -> Result<&'a Value, InstallationRequestError> {
    object.get(name).ok_or_else(|| {
        InstallationRequestError::invalid(
            "required field is missing",
            Some(child_pointer(pointer, name)),
        )
    })
}

fn required_string(
    object: &Map<String, Value>,
    name: &str,
    pointer: &str,
) -> Result<String, InstallationRequestError> {
    required_value(object, name, pointer)?
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| {
            InstallationRequestError::invalid(
                "value must be a string",
                Some(child_pointer(pointer, name)),
            )
        })
}

fn required_bool(
    object: &Map<String, Value>,
    name: &str,
    pointer: &str,
) -> Result<bool, InstallationRequestError> {
    required_value(object, name, pointer)?
        .as_bool()
        .ok_or_else(|| {
            InstallationRequestError::invalid(
                "value must be a boolean",
                Some(child_pointer(pointer, name)),
            )
        })
}

fn reject_unknown(
    object: &Map<String, Value>,
    pointer: &str,
    allowed: &[&str],
) -> Result<(), InstallationRequestError> {
    for name in object.keys() {
        if !allowed.contains(&name.as_str()) {
            return Err(InstallationRequestError::invalid(
                "field is not permitted in installation request",
                Some(child_pointer(pointer, name)),
            ));
        }
    }
    Ok(())
}

fn child_pointer(pointer: &str, name: &str) -> String {
    format!("{}/{}", pointer, name.replace('~', "~0").replace('/', "~1"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID: &[u8] = br#"{
        "schema":"tethers.plug-install/1",
        "candidate_id":"3d846d40-01fc-4e1e-b77d-83944dbed76f",
        "trust":{"scope":"exact_candidate"},
        "conformance":{"allow_non_isolated_supervised_execution":true},
        "installation":{"target_state":"disabled"}
    }"#;

    #[test]
    fn parses_exact_request() {
        let request = parse_installation_request_bytes(VALID).unwrap();
        assert_eq!(request.schema, INSTALLATION_REQUEST_SCHEMA);
        assert_eq!(request.candidate_id, "3d846d40-01fc-4e1e-b77d-83944dbed76f");
        assert_eq!(request.trust.scope, InstallationTrustScope::ExactCandidate);
        assert!(request.conformance.allow_non_isolated_supervised_execution);
        assert_eq!(
            request.installation.target_state,
            InstallationTargetState::Disabled
        );
    }

    #[test]
    fn rejects_bom_before_json_parsing() {
        let mut bytes = vec![0xef, 0xbb, 0xbf];
        bytes.extend_from_slice(VALID);
        let error = parse_installation_request_bytes(&bytes).unwrap_err();
        assert_eq!(error.message, "installation request contains UTF-8 BOM");
        assert_eq!(error.field, None);
    }
}
