//! Strict Resolve v1 HTTPS transport for the existing P2 host seams.
//!
//! This module is deliberately a client, not a coordination framework.  It
//! sends only the bounded opaque projections already produced by Tethers,
//! validates the complete response before returning a closed result, and has
//! no provider, replay, Trail, or Resolve-database access.

use crate::approval;
use crate::resolve_guard::{
    ResolveGuardAdapter, ResolveGuardAdapterError, ResolveGuardAdmission, ResolveGuardRef,
    ResolveGuardRequired,
};
use crate::resolve_outcome::{
    ResolveOutcomeAdapter, ResolveOutcomeAdapterError, ResolveOutcomeDeliveryAck,
    ResolveOutcomeRequest,
};
use serde_json::{Map, Value};
use std::fmt;
use std::io::Read;
use std::time::Duration;

pub const RESOLVE_GUARD_PROTOCOL_VERSION: &str = "resolve.tethers-guard/1";
pub const DEFAULT_RESOLVE_TIMEOUT: Duration = Duration::from_secs(5);
pub const MAX_RESOLVE_BODY_BYTES: usize = 16 * 1024;
pub const MAX_RESOLVE_SCOPE_KEYS: usize = 64;
const MAX_RESOLVE_TIMEOUT: Duration = Duration::from_secs(30);
const MIN_RESOLVE_TIMEOUT: Duration = Duration::from_millis(100);
const ADMISSION_PATH: &str = "/internal/tethers/v1/guard/admit";
const OUTCOME_PATH: &str = "/internal/tethers/v1/outcome";
const BRIDGE_KEY_HEADER: &str = "X-Resolve-Tethers-Key";

/// Host-owned configuration.  The bridge key is intentionally absent from
/// Debug and Display output.  Providers, Tether source, and project config do
/// not construct or control this value.
pub struct ResolveGuardTransportConfig {
    base_url: String,
    bridge_key: String,
    timeout: Duration,
    allow_insecure_local: bool,
}

impl fmt::Debug for ResolveGuardTransportConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResolveGuardTransportConfig")
            .field("base_url", &self.base_url)
            .field("bridge_key", &"<redacted>")
            .field("timeout", &self.timeout)
            .field("allow_insecure_local", &self.allow_insecure_local)
            .finish()
    }
}

impl ResolveGuardTransportConfig {
    pub fn new(
        base_url: impl Into<String>,
        bridge_key: impl Into<String>,
        timeout: Duration,
    ) -> Result<Self, ResolveTransportError> {
        Self::new_checked(base_url.into(), bridge_key.into(), timeout, false)
    }

    /// Explicit local/test-only construction.  It accepts HTTP only for the
    /// loopback hosts so a test server cannot accidentally become a general
    /// insecure remote bridge.
    pub fn new_for_local_test(
        base_url: impl Into<String>,
        bridge_key: impl Into<String>,
        timeout: Duration,
    ) -> Result<Self, ResolveTransportError> {
        Self::new_checked(base_url.into(), bridge_key.into(), timeout, true)
    }

    pub fn from_environment() -> Result<Self, ResolveTransportError> {
        let base_url = std::env::var("TETHERS_RESOLVE_BASE_URL")
            .map_err(|_| ResolveTransportError::MissingConfiguration("TETHERS_RESOLVE_BASE_URL"))?;
        let bridge_key = std::env::var("RESOLVE_TETHERS_BRIDGE_KEY").map_err(|_| {
            ResolveTransportError::MissingConfiguration("RESOLVE_TETHERS_BRIDGE_KEY")
        })?;
        let timeout = match std::env::var("TETHERS_RESOLVE_TIMEOUT_MS") {
            Ok(value) => {
                let millis = value
                    .parse::<u64>()
                    .map_err(|_| ResolveTransportError::InvalidConfiguration("timeout"))?;
                Duration::from_millis(millis)
            }
            Err(_) => DEFAULT_RESOLVE_TIMEOUT,
        };
        let local_mode = std::env::var("TETHERS_RESOLVE_LOCAL_MODE").as_deref() == Ok("1");
        Self::new_checked(base_url, bridge_key, timeout, local_mode)
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    fn new_checked(
        mut base_url: String,
        bridge_key: String,
        timeout: Duration,
        allow_insecure_local: bool,
    ) -> Result<Self, ResolveTransportError> {
        if bridge_key.is_empty() || bridge_key.chars().any(char::is_control) {
            return Err(ResolveTransportError::InvalidConfiguration("bridge key"));
        }
        if !(MIN_RESOLVE_TIMEOUT..=MAX_RESOLVE_TIMEOUT).contains(&timeout) {
            return Err(ResolveTransportError::InvalidConfiguration("timeout"));
        }
        if base_url.ends_with('/') {
            base_url.pop();
        }
        if base_url.is_empty()
            || base_url.chars().any(char::is_whitespace)
            || base_url.contains('?')
            || base_url.contains('#')
        {
            return Err(ResolveTransportError::InvalidConfiguration("base URL"));
        }
        let secure = base_url.starts_with("https://");
        let local_http = base_url.starts_with("http://") && is_loopback_url(&base_url);
        if !secure && !(allow_insecure_local && local_http) {
            return Err(ResolveTransportError::InvalidConfiguration(
                "base URL must use HTTPS",
            ));
        }
        Ok(Self {
            base_url,
            bridge_key,
            timeout,
            allow_insecure_local,
        })
    }
}

fn is_loopback_url(url: &str) -> bool {
    let authority = url
        .strip_prefix("http://")
        .or_else(|| url.strip_prefix("https://"))
        .unwrap_or_default()
        .split('/')
        .next()
        .unwrap_or_default();
    let host = authority
        .rsplit_once(':')
        .map_or(authority, |(host, _)| host);
    matches!(host, "127.0.0.1" | "localhost" | "[::1]")
}

/// A configured client implementing both existing P2/P4 adapter seams.
pub struct ResolveGuardHttpClient {
    agent: ureq::Agent,
    config: ResolveGuardTransportConfig,
}

impl fmt::Debug for ResolveGuardHttpClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResolveGuardHttpClient")
            .field("config", &self.config)
            .finish()
    }
}

impl ResolveGuardHttpClient {
    pub fn new(config: ResolveGuardTransportConfig) -> Self {
        let ureq_config = ureq::Agent::config_builder()
            .timeout_global(Some(config.timeout))
            .max_redirects(0)
            .proxy(None)
            .https_only(!config.allow_insecure_local)
            .build();
        Self {
            agent: ureq::Agent::new_with_config(ureq_config),
            config,
        }
    }

    pub fn from_environment() -> Result<Self, ResolveTransportError> {
        Ok(Self::new(ResolveGuardTransportConfig::from_environment()?))
    }

    pub fn admit_guard_detailed(
        &mut self,
        guard_ref: &ResolveGuardRef,
        required: &ResolveGuardRequired,
    ) -> Result<ResolveGuardAdmission, ResolveTransportError> {
        let request = admission_request(guard_ref, required)?;
        let expected_digest = approval::digest(&request);
        let body = encode_bounded(&request)?;
        let response = self.post_json(ADMISSION_PATH, &body)?;
        parse_admission_response(&response, &expected_digest)
    }

    pub fn deliver_outcome_detailed(
        &mut self,
        request: &ResolveOutcomeRequest,
    ) -> Result<ResolveOutcomeDeliveryAck, ResolveTransportError> {
        let payload = outcome_request(request)?;
        let expected_digest = approval::digest(&payload);
        let body = encode_bounded(&payload)?;
        let response = self.post_json(OUTCOME_PATH, &body)?;
        parse_outcome_response(&response, &expected_digest)
    }

    fn post_json(&self, path: &str, body: &[u8]) -> Result<Vec<u8>, ResolveTransportError> {
        let url = format!("{}{}", self.config.base_url, path);
        let response = self
            .agent
            .post(url)
            .content_type("application/json")
            .header(BRIDGE_KEY_HEADER, &self.config.bridge_key)
            .send(body)
            .map_err(|_| ResolveTransportError::Transport)?;
        if response.status() != 200 {
            return Err(ResolveTransportError::NonSuccessStatus);
        }
        if response
            .headers()
            .get("content-length")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<usize>().ok())
            .is_some_and(|length| length > MAX_RESOLVE_BODY_BYTES)
        {
            return Err(ResolveTransportError::OversizedResponse);
        }
        let mut bytes = Vec::new();
        response
            .into_body()
            .into_reader()
            .take((MAX_RESOLVE_BODY_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(|_| ResolveTransportError::Transport)?;
        if bytes.len() > MAX_RESOLVE_BODY_BYTES {
            return Err(ResolveTransportError::OversizedResponse);
        }
        Ok(bytes)
    }
}

impl ResolveGuardAdapter for ResolveGuardHttpClient {
    fn admit_guard(
        &mut self,
        guard_ref: &ResolveGuardRef,
        required: &ResolveGuardRequired,
    ) -> Result<ResolveGuardAdmission, ResolveGuardAdapterError> {
        self.admit_guard_detailed(guard_ref, required)
            .map_err(|_| ResolveGuardAdapterError)
    }
}

impl ResolveOutcomeAdapter for ResolveGuardHttpClient {
    fn deliver_outcome(
        &mut self,
        request: &ResolveOutcomeRequest,
    ) -> Result<ResolveOutcomeDeliveryAck, ResolveOutcomeAdapterError> {
        self.deliver_outcome_detailed(request)
            .map_err(|_| ResolveOutcomeAdapterError)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResolveTransportError {
    MissingConfiguration(&'static str),
    InvalidConfiguration(&'static str),
    InvalidRequest,
    RequestTooLarge,
    Transport,
    NonSuccessStatus,
    InvalidResponse,
    OversizedResponse,
    InvalidUtf8,
    UnsupportedProtocol,
    InvalidDigest,
    RequestDigestMismatch,
    UnknownDecision,
    UnknownOutcomeResult,
}

impl fmt::Display for ResolveTransportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::MissingConfiguration(field) => match *field {
                "TETHERS_RESOLVE_BASE_URL" => "Resolve base URL is not configured",
                "RESOLVE_TETHERS_BRIDGE_KEY" => "Resolve bridge key is not configured",
                _ => "Resolve transport configuration is incomplete",
            },
            Self::InvalidConfiguration(field) => {
                if *field == "bridge key" {
                    "Resolve bridge key configuration is invalid"
                } else {
                    "Resolve transport configuration is invalid"
                }
            }
            Self::InvalidRequest => "Resolve request is invalid",
            Self::RequestTooLarge => "Resolve request exceeds the protocol body limit",
            Self::Transport => "Resolve transport failed",
            Self::NonSuccessStatus => "Resolve returned a non-success HTTP status",
            Self::InvalidResponse => "Resolve response is invalid",
            Self::OversizedResponse => "Resolve response exceeds the protocol body limit",
            Self::InvalidUtf8 => "Resolve response is not valid UTF-8",
            Self::UnsupportedProtocol => "Resolve protocol version is unsupported",
            Self::InvalidDigest => "Resolve response contains an invalid digest",
            Self::RequestDigestMismatch => "Resolve response does not match the request digest",
            Self::UnknownDecision => "Resolve returned an unknown admission decision",
            Self::UnknownOutcomeResult => "Resolve returned an unknown delivery result",
        };
        f.write_str(message)
    }
}

impl std::error::Error for ResolveTransportError {}

fn admission_request(
    guard_ref: &ResolveGuardRef,
    required: &ResolveGuardRequired,
) -> Result<Value, ResolveTransportError> {
    validate_digest(guard_ref.digest())?;
    validate_digest(required.preparation_digest().as_str())?;
    validate_action_id(required.action_id())?;
    if required.scope_keys().len() > MAX_RESOLVE_SCOPE_KEYS
        || required
            .scope_keys()
            .windows(2)
            .any(|window| window[0] >= window[1])
    {
        return Err(ResolveTransportError::InvalidRequest);
    }
    for key in required.scope_keys() {
        validate_digest(key.as_str())?;
    }
    Ok(serde_json::json!({
        "protocol_version": RESOLVE_GUARD_PROTOCOL_VERSION,
        "guard_ref": guard_ref.digest(),
        "action_id": required.action_id(),
        "preparation_digest": required.preparation_digest().as_str(),
        "scope_keys": required.scope_keys().iter().map(|key| key.as_str()).collect::<Vec<_>>(),
    }))
}

fn outcome_request(request: &ResolveOutcomeRequest) -> Result<Value, ResolveTransportError> {
    validate_action_id(request.action_ref().as_str())?;
    validate_digest(request.preparation_digest().as_str())?;
    Ok(serde_json::json!({
        "protocol_version": RESOLVE_GUARD_PROTOCOL_VERSION,
        "action_id": request.action_ref().as_str(),
        "preparation_digest": request.preparation_digest().as_str(),
        "outcome": request.outcome().as_str(),
    }))
}

fn encode_bounded(value: &Value) -> Result<Vec<u8>, ResolveTransportError> {
    let bytes = serde_json::to_vec(value).map_err(|_| ResolveTransportError::InvalidRequest)?;
    if bytes.len() > MAX_RESOLVE_BODY_BYTES {
        Err(ResolveTransportError::RequestTooLarge)
    } else {
        Ok(bytes)
    }
}

fn parse_admission_response(
    bytes: &[u8],
    expected_digest: &str,
) -> Result<ResolveGuardAdmission, ResolveTransportError> {
    let value = parse_strict_json(bytes)?;
    let object = exact_object(
        &value,
        &[
            "protocol_version",
            "request_digest",
            "decision",
            "reason_code",
        ],
    )?;
    if object.get("protocol_version").and_then(Value::as_str)
        != Some(RESOLVE_GUARD_PROTOCOL_VERSION)
    {
        return Err(ResolveTransportError::UnsupportedProtocol);
    }
    let request_digest = object
        .get("request_digest")
        .and_then(Value::as_str)
        .ok_or(ResolveTransportError::InvalidResponse)?;
    validate_digest(request_digest)?;
    if request_digest != expected_digest {
        return Err(ResolveTransportError::RequestDigestMismatch);
    }
    let decision = object
        .get("decision")
        .and_then(Value::as_str)
        .ok_or(ResolveTransportError::UnknownDecision)?;
    let reason = object
        .get("reason_code")
        .and_then(Value::as_str)
        .ok_or(ResolveTransportError::InvalidResponse)?;
    let valid_reason = matches!(
        reason,
        "admitted"
            | "guard_not_found"
            | "guard_revoked"
            | "guard_already_bound"
            | "guard_expired"
            | "task_not_active"
            | "ownership_changed"
            | "fence_changed"
            | "action_mismatch"
            | "preparation_mismatch"
            | "scope_keys_mismatch"
            | "state_unavailable"
            | "internal_integrity"
    );
    if !valid_reason {
        return Err(ResolveTransportError::InvalidResponse);
    }
    match decision {
        "ADMITTED" if reason == "admitted" => Ok(ResolveGuardAdmission::Admitted),
        "REJECTED" if reason != "admitted" => Ok(ResolveGuardAdmission::Rejected),
        "INDETERMINATE" if reason != "admitted" => Ok(ResolveGuardAdmission::Indeterminate),
        "ADMITTED" | "REJECTED" | "INDETERMINATE" => Err(ResolveTransportError::InvalidResponse),
        _ => Err(ResolveTransportError::UnknownDecision),
    }
}

fn parse_outcome_response(
    bytes: &[u8],
    expected_digest: &str,
) -> Result<ResolveOutcomeDeliveryAck, ResolveTransportError> {
    let value = parse_strict_json(bytes)?;
    let object = exact_object(&value, &["protocol_version", "delivery_digest", "result"])?;
    if object.get("protocol_version").and_then(Value::as_str)
        != Some(RESOLVE_GUARD_PROTOCOL_VERSION)
    {
        return Err(ResolveTransportError::UnsupportedProtocol);
    }
    let delivery_digest = object
        .get("delivery_digest")
        .and_then(Value::as_str)
        .ok_or(ResolveTransportError::InvalidResponse)?;
    validate_digest(delivery_digest)?;
    if delivery_digest != expected_digest {
        return Err(ResolveTransportError::RequestDigestMismatch);
    }
    match object.get("result").and_then(Value::as_str) {
        Some("RECORDED") => Ok(ResolveOutcomeDeliveryAck::Recorded),
        Some("ALREADY_RECORDED") => Ok(ResolveOutcomeDeliveryAck::AlreadyRecorded),
        Some("CONFLICT") => Ok(ResolveOutcomeDeliveryAck::Conflict),
        Some(_) => Err(ResolveTransportError::UnknownOutcomeResult),
        None => Err(ResolveTransportError::InvalidResponse),
    }
}

fn parse_strict_json(bytes: &[u8]) -> Result<Value, ResolveTransportError> {
    let text = std::str::from_utf8(bytes).map_err(|_| ResolveTransportError::InvalidUtf8)?;
    crate::manifest::parse_value_no_dupes(text).map_err(|_| ResolveTransportError::InvalidResponse)
}

fn exact_object<'a>(
    value: &'a Value,
    expected: &[&str],
) -> Result<&'a Map<String, Value>, ResolveTransportError> {
    let object = value
        .as_object()
        .ok_or(ResolveTransportError::InvalidResponse)?;
    if object.len() != expected.len() || object.keys().any(|key| !expected.contains(&key.as_str()))
    {
        return Err(ResolveTransportError::InvalidResponse);
    }
    Ok(object)
}

fn validate_digest(value: &str) -> Result<(), ResolveTransportError> {
    if value.len() == 71
        && value.starts_with("sha256:")
        && value[7..].bytes().all(|byte| byte.is_ascii_hexdigit())
        && value[7..].bytes().all(|byte| !byte.is_ascii_uppercase())
    {
        Ok(())
    } else {
        Err(ResolveTransportError::InvalidDigest)
    }
}

fn validate_action_id(value: &str) -> Result<(), ResolveTransportError> {
    if !value.is_empty()
        && value.len() <= 256
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._:/-".contains(&byte))
    {
        Ok(())
    } else {
        Err(ResolveTransportError::InvalidRequest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolve_guard::{test_guard_admission_context, GuardPreparationProofDigest};
    use crate::resolve_outcome::{ResolveOutcome, TethersActionRef};
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::mpsc;
    use std::thread;

    fn response_for_request(body: &[u8], decision: &str, reason: &str) -> Vec<u8> {
        let request: Value = serde_json::from_slice(body).unwrap();
        let response = serde_json::json!({
            "protocol_version": RESOLVE_GUARD_PROTOCOL_VERSION,
            "request_digest": approval::digest(&request),
            "decision": decision,
            "reason_code": reason,
        });
        serde_json::to_vec(&response).unwrap()
    }

    fn spawn_server<F>(handler: F) -> (String, mpsc::Receiver<Vec<u8>>, thread::JoinHandle<()>)
    where
        F: FnOnce(Vec<u8>) -> Vec<u8> + Send + 'static,
    {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let (tx, rx) = mpsc::channel();
        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut bytes = Vec::new();
            let mut header = [0; 4096];
            let read = stream.read(&mut header).unwrap();
            bytes.extend_from_slice(&header[..read]);
            let header_end = bytes
                .windows(4)
                .position(|window| window == b"\r\n\r\n")
                .unwrap()
                + 4;
            let content_length = String::from_utf8_lossy(&bytes[..header_end])
                .lines()
                .find_map(|line| {
                    line.strip_prefix("Content-Length:")
                        .or_else(|| line.strip_prefix("content-length:"))
                })
                .and_then(|value| value.trim().parse::<usize>().ok())
                .unwrap_or_default();
            while bytes.len() < header_end + content_length {
                let mut chunk = [0; 4096];
                let read = stream.read(&mut chunk).unwrap();
                if read == 0 {
                    break;
                }
                bytes.extend_from_slice(&chunk[..read]);
            }
            let body = bytes[header_end..header_end + content_length].to_vec();
            tx.send(body.clone()).unwrap();
            let response = handler(body);
            let _ = write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                response.len()
            );
            let _ = stream.write_all(&response);
        });
        (url, rx, handle)
    }

    #[test]
    fn valid_admission_serialises_exact_request_and_verifies_digest() {
        let (url, rx, handle) =
            spawn_server(|body| response_for_request(&body, "ADMITTED", "admitted"));
        let config = ResolveGuardTransportConfig::new_for_local_test(
            url,
            "bridge-secret",
            Duration::from_secs(5),
        )
        .unwrap();
        let mut client = ResolveGuardHttpClient::new(config);
        let mut context = test_guard_admission_context(&mut client);
        assert_eq!(context.admit(), ResolveGuardAdmission::Admitted);
        let body = rx.recv().unwrap();
        let value: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(value["protocol_version"], RESOLVE_GUARD_PROTOCOL_VERSION);
        assert_eq!(value["action_id"], "action-1");
        assert!(value["guard_ref"].as_str().unwrap().starts_with("sha256:"));
        assert!(body.len() < MAX_RESOLVE_BODY_BYTES);
        drop(context);
        handle.join().unwrap();
    }

    #[test]
    fn malformed_and_mismatched_responses_never_admit() {
        for response in [
            br#"{"protocol_version":"resolve.tethers-guard/1","request_digest":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","decision":"ADMITTED","reason_code":"admitted","extra":1}"#.to_vec(),
            br#"{"protocol_version":"resolve.tethers-guard/2","request_digest":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","decision":"ADMITTED","reason_code":"admitted"}"#.to_vec(),
            br#"{"protocol_version":"resolve.tethers-guard/1","request_digest":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","decision":"ADMITTED","reason_code":"admitted","decision":"REJECTED"}"#.to_vec(),
            br#"{"protocol_version":"resolve.tethers-guard/1","protocol_version":"resolve.tethers-guard/2","request_digest":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","decision":"ADMITTED","reason_code":"admitted"}"#.to_vec(),
            b"{".to_vec(),
            vec![b'{', 0xff, b'}'],
        ] {
            let (url, _rx, handle) = spawn_server(move |_| response.clone());
            let config = ResolveGuardTransportConfig::new_for_local_test(
                url,
                "bridge-secret",
                Duration::from_secs(5),
            )
            .unwrap();
            let mut client = ResolveGuardHttpClient::new(config);
            let mut context = test_guard_admission_context(&mut client);
            assert_ne!(context.admit(), ResolveGuardAdmission::Admitted);
            drop(context);
            handle.join().unwrap();
        }
    }

    #[test]
    fn outcome_response_ack_is_strict_and_digest_bound() {
        let request = ResolveOutcomeRequest::new(
            TethersActionRef::from_host_value("action-1").unwrap(),
            GuardPreparationProofDigest::from_host_value(&format!("sha256:{}", "b".repeat(64)))
                .unwrap(),
            ResolveOutcome::Succeeded,
        );
        let (url, rx, handle) = spawn_server(|body| {
            let payload: Value = serde_json::from_slice(&body).unwrap();
            serde_json::to_vec(&serde_json::json!({
                "protocol_version": RESOLVE_GUARD_PROTOCOL_VERSION,
                "delivery_digest": approval::digest(&payload),
                "result": "ALREADY_RECORDED"
            }))
            .unwrap()
        });
        let config = ResolveGuardTransportConfig::new_for_local_test(
            url,
            "bridge-secret",
            Duration::from_secs(5),
        )
        .unwrap();
        let mut client = ResolveGuardHttpClient::new(config);
        assert_eq!(
            client.deliver_outcome_detailed(&request).unwrap(),
            ResolveOutcomeDeliveryAck::AlreadyRecorded
        );
        let body = rx.recv().unwrap();
        assert!(!String::from_utf8(body).unwrap().contains("bridge-secret"));
        handle.join().unwrap();
    }

    #[test]
    fn secure_mode_rejects_http_and_redirects_are_not_followed() {
        assert!(matches!(
            ResolveGuardTransportConfig::new(
                "http://127.0.0.1:1",
                "bridge-secret",
                DEFAULT_RESOLVE_TIMEOUT,
            ),
            Err(ResolveTransportError::InvalidConfiguration(_))
        ));
        let (url, _rx, handle) = spawn_server(|_| {
            serde_json::to_vec(&serde_json::json!({"redirect": "https://other.invalid"})).unwrap()
        });
        let config = ResolveGuardTransportConfig::new_for_local_test(
            url,
            "bridge-secret",
            DEFAULT_RESOLVE_TIMEOUT,
        )
        .unwrap();
        assert!(!format!("{config:?}").contains("bridge-secret"));
        let mut client = ResolveGuardHttpClient::new(config);
        assert!(!format!("{client:?}").contains("bridge-secret"));
        let mut context = test_guard_admission_context(&mut client);
        assert_ne!(context.admit(), ResolveGuardAdmission::Admitted);
        drop(context);
        handle.join().unwrap();
    }

    #[test]
    fn response_bound_and_timeout_fail_closed() {
        let (url, _rx, handle) = spawn_server(|_| vec![b'x'; MAX_RESOLVE_BODY_BYTES + 1]);
        let config = ResolveGuardTransportConfig::new_for_local_test(
            url,
            "bridge-secret",
            DEFAULT_RESOLVE_TIMEOUT,
        )
        .unwrap();
        let mut client = ResolveGuardHttpClient::new(config);
        let mut context = test_guard_admission_context(&mut client);
        assert_eq!(context.admit(), ResolveGuardAdmission::Indeterminate);
        drop(context);
        handle.join().unwrap();

        let (url, _rx, handle) = spawn_server(|_| {
            std::thread::sleep(Duration::from_millis(250));
            Vec::new()
        });
        let config = ResolveGuardTransportConfig::new_for_local_test(
            url,
            "bridge-secret",
            Duration::from_millis(100),
        )
        .unwrap();
        let mut client = ResolveGuardHttpClient::new(config);
        let mut context = test_guard_admission_context(&mut client);
        assert_eq!(context.admit(), ResolveGuardAdmission::Indeterminate);
        drop(context);
        handle.join().unwrap();
    }

    #[test]
    fn accepted_resolve_fixtures_share_the_digest_convention() {
        let admission = serde_json::json!({
            "protocol_version": RESOLVE_GUARD_PROTOCOL_VERSION,
            "guard_ref": format!("sha256:{}", "0".repeat(64)),
            "action_id": "action-fixture-1",
            "preparation_digest": format!("sha256:{}", "1".repeat(64)),
            "scope_keys": [
                format!("sha256:{}", "2".repeat(64)),
                format!("sha256:{}", "3".repeat(64)),
            ],
        });
        assert_eq!(
            approval::digest(&admission),
            "sha256:a643f675c945f85b213398f85a97400bec3332178705c67fbebcfeb898c08ce2"
        );
        let outcome = serde_json::json!({
            "protocol_version": RESOLVE_GUARD_PROTOCOL_VERSION,
            "action_id": "action-fixture-1",
            "preparation_digest": format!("sha256:{}", "1".repeat(64)),
            "outcome": "SUCCEEDED",
        });
        assert_eq!(
            approval::digest(&outcome),
            "sha256:d39c779993ff590d913cadd1db4dcaf7233d652e97866cb518bb4d8c5ed8ff4f"
        );
    }

    #[test]
    #[ignore = "requires the accepted Resolve service and Firestore emulator"]
    fn live_resolve_s3_smoke_records_and_redelivers_exact_outcome() {
        let mut client = ResolveGuardHttpClient::from_environment().unwrap();
        let mut context = test_guard_admission_context(&mut client);
        assert_eq!(context.admit(), ResolveGuardAdmission::Admitted);
        let preparation_digest = context.required().preparation_digest().clone();
        drop(context);

        let request = ResolveOutcomeRequest::new(
            TethersActionRef::from_host_value("action-1").unwrap(),
            preparation_digest.clone(),
            ResolveOutcome::Succeeded,
        );
        assert_eq!(
            client.deliver_outcome_detailed(&request).unwrap(),
            ResolveOutcomeDeliveryAck::Recorded
        );
        assert_eq!(
            client.deliver_outcome_detailed(&request).unwrap(),
            ResolveOutcomeDeliveryAck::AlreadyRecorded
        );
        let conflict = ResolveOutcomeRequest::new(
            TethersActionRef::from_host_value("action-1").unwrap(),
            preparation_digest,
            ResolveOutcome::Failed,
        );
        assert_eq!(
            client.deliver_outcome_detailed(&conflict).unwrap(),
            ResolveOutcomeDeliveryAck::Conflict
        );
    }

    #[test]
    #[ignore = "requires the accepted Resolve service and a revoked guard"]
    fn live_resolve_s3_rejects_revoked_guard() {
        let mut client = ResolveGuardHttpClient::from_environment().unwrap();
        let mut context = test_guard_admission_context(&mut client);
        assert_eq!(context.admit(), ResolveGuardAdmission::Rejected);
    }
}
