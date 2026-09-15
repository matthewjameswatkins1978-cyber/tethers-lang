//! Host-owned Lantern authority and audit bridge.
//!
//! Tethers remains the policy and dispatch authority.  This module only asks
//! Lantern for an independent persistent grant decision after Tethers has
//! resolved the capability, validated the schema, and assessed its scope.

use crate::{
    approval, policy::ProposedAction, resolve_guard::ResolvedActionScope,
    resolver::ResolvedCapability,
};
use serde_json::{json, Value};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

pub const AUTHORITY_CHECK_WIRE_VERSION: &str = "lantern.authority.check/1";
pub const RECEIPT_INTENT_WIRE_VERSION: &str = "lantern.receipt.intent/1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorityRequest {
    pub action_id: String,
    pub principal_id: String,
    pub capability_id: String,
    pub capability_version: String,
    pub scope: std::collections::BTreeMap<String, String>,
    pub constraints: std::collections::BTreeMap<String, String>,
}

impl AuthorityRequest {
    pub(crate) fn from_resolved(
        action: &ProposedAction,
        resolved: &ResolvedCapability,
        scope: &ResolvedActionScope,
        principal_id: &str,
    ) -> Self {
        Self {
            action_id: action.action_id.clone(),
            principal_id: principal_id.to_owned(),
            capability_id: resolved.capability_name().to_owned(),
            capability_version: resolved.capability_version().to_string(),
            scope: scope.authority_scope(),
            constraints: std::collections::BTreeMap::new(),
        }
    }

    fn json(&self) -> Value {
        json!({
            "wire_version": AUTHORITY_CHECK_WIRE_VERSION,
            "action_id": self.action_id,
            "principal_id": self.principal_id,
            "capability_id": self.capability_id,
            "capability_version": self.capability_version,
            "scope": self.scope,
            "constraints": self.constraints,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthorityVerdict {
    Allow { grant_id: String },
    Deny { reason_code: String },
    Unavailable { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceiptIntent {
    pub kind: &'static str,
    pub request: AuthorityRequest,
    pub decision: &'static str,
    pub grant_id: Option<String>,
    pub executed: bool,
    pub outcome: Option<&'static str>,
    pub result_ref: Option<String>,
    pub reason_code: Option<String>,
}

pub trait AuthorityProvider: Send + Sync {
    fn check(&self, request: &AuthorityRequest)
        -> Result<AuthorityVerdict, AuthorityProviderError>;
    fn record_receipt(&self, intent: &ReceiptIntent) -> Result<(), AuthorityProviderError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorityProviderError {
    pub code: &'static str,
    pub message: String,
}

impl std::fmt::Display for AuthorityProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for AuthorityProviderError {}

#[derive(Debug, Clone)]
pub struct LanternHttpAuthorityProvider {
    endpoint: String,
    audit_token: String,
    timeout: Duration,
}

impl LanternHttpAuthorityProvider {
    pub fn from_config(
        endpoint: impl Into<String>,
        audit_token: String,
    ) -> Result<Self, AuthorityProviderError> {
        let endpoint = endpoint.into();
        if !endpoint.starts_with("http://") || audit_token.is_empty() {
            return Err(AuthorityProviderError {
                code: "AUTHORITY_CONFIGURATION_INVALID",
                message: "Lantern endpoint must be http:// and audit token must be present"
                    .to_owned(),
            });
        }
        Ok(Self {
            endpoint,
            audit_token,
            timeout: Duration::from_secs(3),
        })
    }

    fn post(&self, path: &str, body: &Value) -> Result<Value, AuthorityProviderError> {
        let (host, port, base_path) = parse_http_endpoint(&self.endpoint)?;
        let address = format!("{host}:{port}");
        let mut stream = TcpStream::connect(address).map_err(|e| unavailable(e.to_string()))?;
        stream
            .set_read_timeout(Some(self.timeout))
            .and_then(|_| stream.set_write_timeout(Some(self.timeout)))
            .map_err(|e| unavailable(e.to_string()))?;
        let payload = serde_json::to_vec(body).map_err(|e| invalid(e.to_string()))?;
        let target = if base_path == "/" {
            path.to_owned()
        } else {
            format!("{}{}", base_path.trim_end_matches('/'), path)
        };
        let request = format!(
            "POST {target} HTTP/1.1\r\nHost: {host}\r\nAuthorization: Bearer {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            self.audit_token,
            payload.len()
        );
        stream
            .write_all(request.as_bytes())
            .and_then(|_| stream.write_all(&payload))
            .map_err(|e| unavailable(e.to_string()))?;
        let mut response = Vec::new();
        stream
            .read_to_end(&mut response)
            .map_err(|e| unavailable(e.to_string()))?;
        if response.len() > 128 * 1024 {
            return Err(invalid(
                "Lantern response exceeded the bounded response size",
            ));
        }
        let separator = response
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .ok_or_else(|| unavailable("Lantern returned an invalid HTTP response"))?;
        let header = std::str::from_utf8(&response[..separator])
            .map_err(|_| unavailable("Lantern response headers were not UTF-8"))?;
        let status = header
            .lines()
            .next()
            .and_then(|line| line.split_whitespace().nth(1))
            .unwrap_or_default();
        let body = &response[separator + 4..];
        let value: Value = serde_json::from_slice(body)
            .map_err(|e| unavailable(format!("Lantern returned invalid JSON: {e}")))?;
        if status != "200" && status != "201" {
            return Err(unavailable(format!("Lantern returned HTTP {status}")));
        }
        Ok(value)
    }
}

impl AuthorityProvider for LanternHttpAuthorityProvider {
    fn check(
        &self,
        request: &AuthorityRequest,
    ) -> Result<AuthorityVerdict, AuthorityProviderError> {
        let value = self.post("/api/v1/tethers/authority/check", &request.json())?;
        if value.get("wire_version").and_then(Value::as_str) != Some("lantern.authority.decision/1")
        {
            return Err(unavailable(
                "Lantern returned an unsupported decision wire version",
            ));
        }
        match value.get("decision").and_then(Value::as_str) {
            Some("ALLOW") => Ok(AuthorityVerdict::Allow {
                grant_id: value
                    .get("grant_id")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned(),
            }),
            Some("DENY") => Ok(AuthorityVerdict::Deny {
                reason_code: value
                    .get("reason_code")
                    .and_then(Value::as_str)
                    .unwrap_or("AUTHORITY_DENIED")
                    .to_owned(),
            }),
            other => Err(unavailable(format!("invalid Lantern decision {:?}", other))),
        }
    }

    fn record_receipt(&self, intent: &ReceiptIntent) -> Result<(), AuthorityProviderError> {
        let value = json!({
            "wire_version": RECEIPT_INTENT_WIRE_VERSION,
            "kind": intent.kind,
            "action_id": intent.request.action_id,
            "principal_id": intent.request.principal_id,
            "capability_id": intent.request.capability_id,
            "capability_version": intent.request.capability_version,
            "decision": intent.decision,
            "grant_id": intent.grant_id,
            "scope": intent.request.scope,
            "executed": intent.executed,
            "outcome": intent.outcome,
            "result_ref": intent.result_ref,
            "reason_code": intent.reason_code,
        });
        self.post("/api/v1/tethers/receipts", &value).map(|_| ())
    }
}

pub fn decision_receipt(request: AuthorityRequest, verdict: &AuthorityVerdict) -> ReceiptIntent {
    match verdict {
        AuthorityVerdict::Allow { grant_id } => ReceiptIntent {
            kind: "decision",
            request,
            decision: "ALLOW",
            grant_id: Some(grant_id.clone()),
            executed: false,
            outcome: None,
            result_ref: None,
            reason_code: None,
        },
        AuthorityVerdict::Deny { reason_code } => ReceiptIntent {
            kind: "decision",
            request,
            decision: "DENY",
            grant_id: None,
            executed: false,
            outcome: None,
            result_ref: None,
            reason_code: Some(reason_code.clone()),
        },
        AuthorityVerdict::Unavailable { reason } => ReceiptIntent {
            kind: "decision",
            request,
            decision: "UNAVAILABLE",
            grant_id: None,
            executed: false,
            outcome: None,
            result_ref: None,
            reason_code: Some(reason.clone()),
        },
    }
}

pub fn outcome_receipt(
    request: AuthorityRequest,
    grant_id: String,
    execution_id: Option<String>,
    outcome: &'static str,
    reason_code: Option<String>,
) -> ReceiptIntent {
    ReceiptIntent {
        kind: "outcome",
        request,
        decision: "ALLOW",
        grant_id: Some(grant_id),
        executed: execution_id.is_some(),
        outcome: Some(outcome),
        result_ref: execution_id,
        reason_code,
    }
}

pub fn argument_digest(arguments: &Value) -> String {
    approval::digest(arguments)
}

fn parse_http_endpoint(endpoint: &str) -> Result<(String, u16, String), AuthorityProviderError> {
    let rest = endpoint
        .strip_prefix("http://")
        .ok_or_else(|| invalid("only http:// Lantern endpoints are supported"))?;
    let (host_port, path) = match rest.split_once('/') {
        Some((host, path)) => (host, format!("/{path}")),
        None => (rest, "/".to_owned()),
    };
    let (host, port) = host_port
        .rsplit_once(':')
        .ok_or_else(|| invalid("Lantern endpoint must include an explicit port"))?;
    let port = port
        .parse::<u16>()
        .map_err(|_| invalid("Lantern endpoint port is invalid"))?;
    if host.is_empty() {
        return Err(invalid("Lantern endpoint host is blank"));
    }
    Ok((host.to_owned(), port, path.to_owned()))
}

fn unavailable(message: impl Into<String>) -> AuthorityProviderError {
    AuthorityProviderError {
        code: "AUTHORITY_UNAVAILABLE",
        message: message.into(),
    }
}
fn invalid(message: impl Into<String>) -> AuthorityProviderError {
    AuthorityProviderError {
        code: "AUTHORITY_PROTOCOL_INVALID",
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;
    use std::thread;

    fn request() -> AuthorityRequest {
        AuthorityRequest {
            action_id: "action-1".to_owned(),
            principal_id: "agent:lucy".to_owned(),
            capability_id: "demo.export_summary".to_owned(),
            capability_version: "1".to_owned(),
            scope: [("kind".to_owned(), "unrestricted".to_owned())]
                .into_iter()
                .collect(),
            constraints: Default::default(),
        }
    }

    fn serve(listener: TcpListener) {
        for response in [
            r#"{"wire_version":"lantern.authority.decision/1","decision":"ALLOW","grant_id":"grant-1","action_id":"action-1"}"#,
            r#"{"receipt":{"receipt_id":"receipt-1"}}"#,
        ] {
            let (mut stream, _) = listener.accept().unwrap();
            let mut bytes = Vec::new();
            let mut buffer = [0_u8; 4096];
            loop {
                let count = stream.read(&mut buffer).unwrap();
                if count == 0 {
                    break;
                }
                bytes.extend_from_slice(&buffer[..count]);
                let Some(separator) = bytes.windows(4).position(|w| w == b"\r\n\r\n") else {
                    continue;
                };
                let headers = String::from_utf8_lossy(&bytes[..separator]);
                let length = headers
                    .lines()
                    .find_map(|line| line.strip_prefix("Content-Length: "))
                    .unwrap()
                    .parse::<usize>()
                    .unwrap();
                if bytes.len() >= separator + 4 + length {
                    break;
                }
            }
            let text = String::from_utf8_lossy(&bytes);
            assert!(text.contains("Authorization: Bearer test-token"));
            assert!(text.contains("lantern."));
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                response.len(), response
            );
            stream.write_all(response.as_bytes()).unwrap();
        }
    }

    #[test]
    fn http_provider_uses_bounded_versioned_exchange() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || serve(listener));
        let provider = LanternHttpAuthorityProvider::from_config(
            format!("http://127.0.0.1:{port}"),
            "test-token".to_owned(),
        )
        .unwrap();
        let request = request();
        let verdict = provider.check(&request).unwrap();
        assert_eq!(
            verdict,
            AuthorityVerdict::Allow {
                grant_id: "grant-1".to_owned()
            }
        );
        provider
            .record_receipt(&decision_receipt(request, &verdict))
            .unwrap();
        server.join().unwrap();
    }
}
