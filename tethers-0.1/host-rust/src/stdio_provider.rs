// Host-owned stdio MCP provider binding with retained session support.
//
// ManagedProvider owns a SupervisedChild in an Option so that close()
// can move the child into shutdown.  Drop remains the emergency fallback.

use crate::child_process::{ChildConfig, ChildError, SupervisedChild};
use crate::manifest::{self, BindingKind, VerifiedManifest};
use crate::provider::{self, AdmissionError, ProviderConfig};
use crate::trusted_store::TrustedManifestStore;
use std::collections::HashSet;
use std::fmt;
use std::path::PathBuf;
use std::time::{Duration, Instant};

const INITIALIZE_REQUEST_ID: u64 = 1;
const TOOLS_LIST_REQUEST_ID: u64 = 2;

/// A running provider process owned by the host.
pub struct ManagedProvider {
    child: Option<SupervisedChild>,
    read_timeout: Duration,
    catalogue_change_observed: bool,
}

impl ManagedProvider {
    /// Launch the provider with explicit current directory.
    pub fn launch(
        command: &str,
        args: &[String],
        working_dir: &PathBuf,
        startup_timeout_override: Option<Duration>,
        graceful_close_timeout_override: Option<Duration>,
    ) -> Result<Self, StdioProviderError> {
        let config = if let (Some(startup), Some(graceful)) =
            (startup_timeout_override, graceful_close_timeout_override)
        {
            ChildConfig::test_config(command, args.to_vec(), startup, graceful)
        } else {
            let mut cfg = ChildConfig::production(command, args.to_vec());
            cfg.current_dir = Some(working_dir.clone());
            cfg
        };

        let child =
            SupervisedChild::launch(config).map_err(|e| StdioProviderError::LaunchFailed {
                command: command.to_owned(),
                message: e.to_string(),
            })?;

        Ok(Self {
            child: Some(child),
            read_timeout: Duration::from_secs(10),
            catalogue_change_observed: false,
        })
    }

    fn child_mut(&mut self) -> Result<&mut SupervisedChild, StdioProviderError> {
        self.child
            .as_mut()
            .ok_or(StdioProviderError::StdinUnavailable)
    }

    fn write_message(&mut self, message: &serde_json::Value) -> Result<(), StdioProviderError> {
        let line = serde_json::to_string(message)
            .map_err(|e| StdioProviderError::SerializeFailed(e.to_string()))?;
        self.child_mut()?
            .write_line(&line)
            .map_err(|_| StdioProviderError::WriteFailed("write failed".to_owned()))
    }

    fn read_message(&mut self, timeout: Duration) -> Result<serde_json::Value, StdioProviderError> {
        let line = self
            .child_mut()?
            .read_protocol_line(timeout)
            .map_err(|e| match e {
                ChildError::ReadTimeout(msg) => StdioProviderError::ReadFailed(msg),
                ChildError::ProtocolError(msg) => StdioProviderError::MalformedResponse(msg),
                ChildError::ProcessExited(_) => StdioProviderError::EmptyResponse,
                ChildError::NotUtf8 => {
                    StdioProviderError::MalformedResponse("not valid UTF-8".to_owned())
                }
                ChildError::LineTooLarge { .. } => {
                    StdioProviderError::MalformedResponse("line too large".to_owned())
                }
                ChildError::Interrupted => StdioProviderError::Interrupted,
                _ => StdioProviderError::ReadFailed(e.to_string()),
            })?;

        let line = line.trim();
        if line.is_empty() {
            return Err(StdioProviderError::EmptyResponse);
        }

        serde_json::from_str(line).map_err(|error| {
            StdioProviderError::MalformedResponse(format!(
                "provider stdout was not valid JSON: {error}"
            ))
        })
    }

    fn request(
        &mut self,
        id: u64,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, StdioProviderError> {
        self.request_with_timeout(id, method, params, self.read_timeout)
    }

    fn request_with_timeout(
        &mut self,
        id: u64,
        method: &str,
        params: serde_json::Value,
        timeout: Duration,
    ) -> Result<serde_json::Value, StdioProviderError> {
        self.write_message(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        }))?;

        let started = Instant::now();
        let response = loop {
            let remaining = timeout.saturating_sub(started.elapsed());
            if remaining.is_zero() {
                return Err(StdioProviderError::ReadFailed(
                    "timeout waiting for JSON-RPC response".to_owned(),
                ));
            }
            let message = self.read_message(remaining)?;
            if self.observe_server_notification(&message)? {
                continue;
            }
            break message;
        };
        let object = response.as_object().ok_or_else(|| {
            StdioProviderError::ProtocolError("JSON-RPC response must be an object".to_owned())
        })?;

        if object.get("jsonrpc").and_then(serde_json::Value::as_str) != Some("2.0") {
            return Err(StdioProviderError::ProtocolError(
                "JSON-RPC response must declare jsonrpc 2.0".to_owned(),
            ));
        }
        if object.get("id") != Some(&serde_json::json!(id)) {
            return Err(StdioProviderError::ProtocolError(format!(
                "JSON-RPC response id did not match request {id}"
            )));
        }
        if let Some(error) = object.get("error") {
            return Err(StdioProviderError::ExplicitProviderError(format!(
                "provider returned JSON-RPC error for {method}: {error}"
            )));
        }

        object.get("result").cloned().ok_or_else(|| {
            StdioProviderError::ProtocolError(format!(
                "JSON-RPC response for {method} had no result"
            ))
        })
    }

    fn observe_server_notification(
        &mut self,
        message: &serde_json::Value,
    ) -> Result<bool, StdioProviderError> {
        let Some(object) = message.as_object() else {
            return Ok(false);
        };
        if object.contains_key("id") {
            return Ok(false);
        }
        let Some(method) = object.get("method").and_then(serde_json::Value::as_str) else {
            return Ok(false);
        };
        if object.get("jsonrpc").and_then(serde_json::Value::as_str) != Some("2.0") {
            return Err(StdioProviderError::ProtocolError(
                "JSON-RPC notification must declare jsonrpc 2.0".to_owned(),
            ));
        }
        if method == "notifications/tools/list_changed" {
            self.catalogue_change_observed = true;
        }
        Ok(true)
    }

    /// Drain already-buffered server notifications before the next serial
    /// request. A response outside an active request is a protocol error.
    pub(crate) fn poll_notifications(&mut self) -> Result<(), StdioProviderError> {
        loop {
            let line = match self
                .child_mut()?
                .try_read_protocol_line()
                .map_err(|error| StdioProviderError::ReadFailed(error.to_string()))?
            {
                Some(line) => line,
                None => return Ok(()),
            };
            let message: serde_json::Value =
                serde_json::from_str(line.trim()).map_err(|error| {
                    StdioProviderError::MalformedResponse(format!(
                        "provider stdout was not valid JSON: {error}"
                    ))
                })?;
            if !self.observe_server_notification(&message)? {
                return Err(StdioProviderError::ProtocolError(
                    "provider emitted a response outside an active request".to_owned(),
                ));
            }
        }
    }

    pub(crate) fn take_catalogue_change_observed(&mut self) -> bool {
        std::mem::take(&mut self.catalogue_change_observed)
    }

    fn notify(
        &mut self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<(), StdioProviderError> {
        self.write_message(&serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        }))
    }

    /// Initialize the provider.
    pub fn initialize(
        &mut self,
        protocol_version: &str,
        expected_server_name: &str,
    ) -> Result<(), StdioProviderError> {
        let result = self.request(
            INITIALIZE_REQUEST_ID,
            "initialize",
            serde_json::json!({
                "protocolVersion": protocol_version,
                "capabilities": {},
                "clientInfo": {
                    "name": "tethers-reference-host",
                    "version": "0.2.0"
                }
            }),
        )?;

        if result
            .get("protocolVersion")
            .and_then(serde_json::Value::as_str)
            != Some(protocol_version)
        {
            return Err(StdioProviderError::ProtocolError(
                "provider selected an incompatible MCP protocol version".to_owned(),
            ));
        }
        if result
            .pointer("/serverInfo/name")
            .and_then(serde_json::Value::as_str)
            != Some(expected_server_name)
        {
            return Err(StdioProviderError::ProtocolError(format!(
                "provider server name did not match trusted binding '{expected_server_name}'"
            )));
        }
        if !result
            .pointer("/capabilities/tools")
            .is_some_and(serde_json::Value::is_object)
        {
            return Err(StdioProviderError::ProtocolError(
                "provider did not advertise MCP tools capability".to_owned(),
            ));
        }

        self.notify("notifications/initialized", serde_json::json!({}))
    }

    /// List tools from the provider.
    pub fn list_tools(&mut self) -> Result<Vec<serde_json::Value>, StdioProviderError> {
        let mut next_request_id = TOOLS_LIST_REQUEST_ID;
        self.list_tools_paginated(&mut next_request_id)
    }

    pub(crate) fn list_tools_paginated(
        &mut self,
        next_request_id: &mut u64,
    ) -> Result<Vec<serde_json::Value>, StdioProviderError> {
        let mut tools = Vec::new();
        let mut operation_names = HashSet::new();
        let mut observed_cursors = HashSet::new();
        let mut cursor: Option<String> = None;
        loop {
            let request_id = *next_request_id;
            *next_request_id = request_id.checked_add(1).ok_or_else(|| {
                StdioProviderError::ProtocolError("Socket request id exhausted".to_owned())
            })?;
            let params = cursor.as_ref().map_or_else(
                || serde_json::json!({}),
                |value| serde_json::json!({"cursor": value}),
            );
            let result = self.request(request_id, "tools/list", params)?;
            let page = result
                .get("tools")
                .and_then(serde_json::Value::as_array)
                .ok_or_else(|| {
                    StdioProviderError::ProtocolError(
                        "tools/list result must contain a tools array".to_owned(),
                    )
                })?;
            for tool in page {
                let object = tool.as_object().ok_or_else(|| {
                    StdioProviderError::ProtocolError(
                        "every tools/list entry must be an object".to_owned(),
                    )
                })?;
                let name = object
                    .get("name")
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| {
                        StdioProviderError::ProtocolError(
                            "every tools/list entry must have a string name".to_owned(),
                        )
                    })?;
                if !operation_names.insert(name.to_owned()) {
                    return Err(StdioProviderError::ProtocolError(format!(
                        "tools/list contained duplicate tool name '{name}'"
                    )));
                }
                tools.push(tool.clone());
            }

            cursor = match result.get("nextCursor") {
                None | Some(serde_json::Value::Null) => None,
                Some(serde_json::Value::String(value)) => Some(value.clone()),
                Some(_) => {
                    return Err(StdioProviderError::ProtocolError(
                        "tools/list nextCursor must be a string when present".to_owned(),
                    ))
                }
            };
            match cursor.as_ref() {
                Some(value) if observed_cursors.insert(value.clone()) => {}
                Some(_) => {
                    return Err(StdioProviderError::ProtocolError(
                        "tools/list cursor repeated or looped".to_owned(),
                    ))
                }
                None => return Ok(tools),
            }
        }
    }

    pub fn ping(&mut self, id: u64) -> Result<serde_json::Value, StdioProviderError> {
        self.request(id, "ping", serde_json::json!({}))
    }

    /// MCP tools/call invocation for retained-session dispatch.
    ///
    /// Sends a `tools/call` request with the given `id`, `tool_name`, and
    /// `arguments`.  Validates JSON-RPC version, matching response ID, and
    /// maps JSON-RPC errors to `ProtocolError`.
    pub fn tools_call(
        &mut self,
        id: u64,
        tool_name: &str,
        arguments: &serde_json::Value,
    ) -> Result<serde_json::Value, StdioProviderError> {
        self.request(
            id,
            "tools/call",
            serde_json::json!({
                "name": tool_name,
                "arguments": arguments
            }),
        )
    }

    /// Invoke one provider tool while bounding the response wait by the host's
    /// exact remaining monotonic deadline.
    pub fn tools_call_with_timeout(
        &mut self,
        id: u64,
        tool_name: &str,
        arguments: &serde_json::Value,
        timeout: Duration,
    ) -> Result<serde_json::Value, StdioProviderError> {
        self.request_with_timeout(
            id,
            "tools/call",
            serde_json::json!({
                "name": tool_name,
                "arguments": arguments
            }),
            timeout,
        )
    }

    /// Get retained stderr tail.
    pub fn stderr_tail(&self) -> String {
        self.child
            .as_ref()
            .map(|c| c.stderr_tail())
            .unwrap_or_default()
    }

    /// Graceful close: take the child and call shutdown.
    /// Drop is the emergency fallback only.
    pub fn close(&mut self) {
        if let Some(child) = self.child.take() {
            child.shutdown();
        }
    }
}

impl Drop for ManagedProvider {
    fn drop(&mut self) {
        // Emergency fallback: close if not already closed.
        if let Some(child) = self.child.take() {
            child.shutdown();
        }
    }
}

// ===========================================================================
// Error types, config, and discovery helpers (unchanged)
// ===========================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StdioProviderError {
    LaunchFailed { command: String, message: String },
    StdinUnavailable,
    StdoutUnavailable,
    SerializeFailed(String),
    WriteFailed(String),
    ReadFailed(String),
    EmptyResponse,
    MalformedResponse(String),
    ProtocolError(String),
    ExplicitProviderError(String),
    CatalogueStale,
    CatalogueChangedDuringDiscovery,
    Interrupted,
    TrustedManifestInvalid(String),
    AdmissionFailed(String),
}

impl fmt::Display for StdioProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LaunchFailed { command, message } => {
                write!(f, "launch failed for '{command}': {message}")
            }
            Self::StdinUnavailable => write!(f, "stdin unavailable"),
            Self::StdoutUnavailable => write!(f, "stdout unavailable"),
            Self::SerializeFailed(m) => write!(f, "serialization failed: {m}"),
            Self::WriteFailed(m) => write!(f, "write failed: {m}"),
            Self::ReadFailed(m) => write!(f, "read failed: {m}"),
            Self::EmptyResponse => write!(f, "provider returned no response"),
            Self::MalformedResponse(m) => write!(f, "malformed response: {m}"),
            Self::ProtocolError(m) => write!(f, "MCP protocol error: {m}"),
            Self::ExplicitProviderError(m) => write!(f, "MCP JSON-RPC error: {m}"),
            Self::CatalogueStale => {
                write!(
                    f,
                    "Socket catalogue is stale; exact rediscovery is required"
                )
            }
            Self::CatalogueChangedDuringDiscovery => {
                write!(f, "catalogue changed during discovery")
            }
            Self::Interrupted => write!(f, "provider call interrupted"),
            Self::TrustedManifestInvalid(m) => write!(f, "trusted manifest invalid: {m}"),
            Self::AdmissionFailed(m) => write!(f, "admission failed: {m}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StdioProviderConfig {
    pub command: String,
    pub args: Vec<String>,
    pub protocol_version: String,
    pub provider_config: ProviderConfig,
}

fn admission_error(e: AdmissionError) -> StdioProviderError {
    StdioProviderError::AdmissionFailed(format!("{e:?}"))
}

fn validate_host_binding(
    config: &ProviderConfig,
    trusted: &VerifiedManifest,
) -> Result<(), StdioProviderError> {
    let allowed = config
        .allowed_capabilities
        .iter()
        .find(|a| {
            a.capability_name == trusted.capability_name()
                && a.capability_version == trusted.capability_version()
        })
        .ok_or_else(|| {
            StdioProviderError::AdmissionFailed(format!(
                "host configuration does not allow exact capability {} v{}",
                trusted.capability_name(),
                trusted.capability_version()
            ))
        })?;

    let pinned = allowed.pinned_digest.as_deref().ok_or_else(|| {
        StdioProviderError::AdmissionFailed(format!(
            "host configuration must pin the digest for {} v{}",
            trusted.capability_name(),
            trusted.capability_version()
        ))
    })?;
    if pinned != trusted.verified_digest() {
        return Err(StdioProviderError::AdmissionFailed(format!(
            "host-pinned digest for {} did not match reviewed manifest",
            trusted.capability_name()
        )));
    }

    let mut scratch = TrustedManifestStore::new();
    provider::admit_provider_manifest(config, trusted.clone(), &mut scratch)
        .map_err(admission_error)?;
    Ok(())
}

fn matching_tool<'a>(
    tools: &'a [serde_json::Value],
    expected_tool_name: &str,
) -> Result<&'a serde_json::Map<String, serde_json::Value>, StdioProviderError> {
    let mut names = HashSet::new();
    let mut found = None;
    for tool in tools {
        let obj = tool.as_object().ok_or_else(|| {
            StdioProviderError::ProtocolError("every tools/list entry must be an object".to_owned())
        })?;
        let name = obj
            .get("name")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| {
                StdioProviderError::ProtocolError(
                    "every tools/list entry must have a string name".to_owned(),
                )
            })?;
        if !names.insert(name.to_owned()) {
            return Err(StdioProviderError::ProtocolError(format!(
                "tools/list contained duplicate tool name '{name}'"
            )));
        }
        if name == expected_tool_name {
            found = Some(obj);
        }
    }
    found.ok_or_else(|| {
        StdioProviderError::ProtocolError(format!(
            "tools/list did not contain trusted binding tool '{expected_tool_name}'"
        ))
    })
}

pub fn compare_discovery_evidence(
    tools: &[serde_json::Value],
    trusted: &VerifiedManifest,
) -> Result<(), StdioProviderError> {
    let manifest = trusted.manifest();
    if manifest.binding.kind != BindingKind::Mcp {
        return Err(StdioProviderError::AdmissionFailed(
            "trusted manifest binding is not MCP".to_owned(),
        ));
    }
    let tool = matching_tool(tools, &manifest.binding.tool_name)?;
    if tool.get("inputSchema") != Some(&manifest.input_schema) {
        return Err(StdioProviderError::ProtocolError(format!(
            "discovered input schema did not match trusted manifest for '{}'",
            manifest.binding.tool_name
        )));
    }
    if tool.get("outputSchema") != Some(&manifest.output_schema) {
        return Err(StdioProviderError::ProtocolError(format!(
            "discovered output schema did not match trusted manifest for '{}'",
            manifest.binding.tool_name
        )));
    }
    Ok(())
}

pub fn discover_and_admit(
    config: &StdioProviderConfig,
    trusted_manifest_json: &str,
    store: &mut TrustedManifestStore,
) -> Result<VerifiedManifest, StdioProviderError> {
    let trusted = manifest::verify_manifest(trusted_manifest_json)
        .map_err(|e| StdioProviderError::TrustedManifestInvalid(format!("{e:?}")))?;
    validate_host_binding(&config.provider_config, &trusted)?;

    let binding = &trusted.manifest().binding;
    let mut provider = ManagedProvider::launch(
        &config.command,
        &config.args,
        &std::env::current_dir().unwrap_or_default(),
        None,
        None,
    )?;
    provider.initialize(&config.protocol_version, &binding.server_name)?;
    let tools = provider.list_tools()?;
    compare_discovery_evidence(&tools, &trusted)?;

    provider::admit_provider_manifest(&config.provider_config, trusted.clone(), store)
        .map_err(admission_error)?;

    provider.close();
    Ok(trusted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::AllowedCapability;
    use crate::resolver::{self, ProviderAvailability};

    const TRUSTED_MANIFEST: &str =
        include_str!("../../protocol/capability-manifests/fixture-ping.json");
    const TRUSTED_DIGEST: &str =
        "sha256:01fed7a4b877dd82abe91a1b6cfcd476b02e4c115489e70cbb285b8bf2d32d8b";

    fn fixture_script_path() -> std::path::PathBuf {
        let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.pop();
        path.push("scripts");
        path.push("tethers-stdio-fixture.ps1");
        path
    }

    fn fixture_config(mode: &str) -> StdioProviderConfig {
        StdioProviderConfig {
            command: "pwsh.exe".to_owned(),
            args: vec![
                "-NoProfile".to_owned(),
                "-File".to_owned(),
                fixture_script_path().to_string_lossy().into_owned(),
                "-Mode".to_owned(),
                mode.to_owned(),
            ],
            protocol_version: "2025-11-25".to_owned(),
            provider_config: ProviderConfig {
                identity: "tethers-stdio-fixture".to_owned(),
                display_name: "Tethers Stdio Fixture".to_owned(),
                allowed_capabilities: vec![AllowedCapability {
                    capability_name: "fixture.ping".to_owned(),
                    capability_version: 1,
                    pinned_digest: Some(TRUSTED_DIGEST.to_owned()),
                }],
            },
        }
    }

    fn assert_mode_fails_closed(mode: &str, expected: &str) {
        let mut store = TrustedManifestStore::new();
        let err =
            discover_and_admit(&fixture_config(mode), TRUSTED_MANIFEST, &mut store).unwrap_err();
        assert!(
            err.to_string().contains(expected),
            "unexpected error for mode {mode}: {err}"
        );
        assert!(store.is_empty());
    }

    #[test]
    fn real_mcp_discovery_admits_only_host_owned_manifest() {
        let mut store = TrustedManifestStore::new();
        let verified =
            discover_and_admit(&fixture_config("valid"), TRUSTED_MANIFEST, &mut store).unwrap();
        assert_eq!(verified.capability_name(), "fixture.ping");
        assert_eq!(verified.capability_version(), 1);
        assert_eq!(verified.verified_digest(), TRUSTED_DIGEST);
        assert_eq!(
            verified.manifest().provider.identity,
            "tethers-stdio-fixture"
        );
        assert!(store.get_by_name_version("fixture.ping", 1).is_some());
    }

    // ... remaining existing tests identical to before ...
    #[test]
    fn provider_description_cannot_rewrite_trusted_manifest() {
        let mut store = TrustedManifestStore::new();
        let v = discover_and_admit(
            &fixture_config("changed-description"),
            TRUSTED_MANIFEST,
            &mut store,
        )
        .unwrap();
        assert_eq!(
            v.manifest().description,
            "A deterministic test capability for stdio provider binding."
        );
    }

    #[test]
    fn admitted_fixture_resolves_only_when_host_reports_it_available() {
        let mut store = TrustedManifestStore::new();
        let v = discover_and_admit(&fixture_config("valid"), TRUSTED_MANIFEST, &mut store).unwrap();
        let a = ProviderAvailability::from_identities(["tethers-stdio-fixture"]);
        let r = resolver::resolve_capability(
            &store,
            &a,
            "fixture.ping",
            1,
            Some("tethers-stdio-fixture"),
        )
        .unwrap();
        assert_eq!(r.manifest_digest(), v.verified_digest());
    }

    #[test]
    fn j13b_provider_is_unavailable_without_matching_live_discovery_evidence() {
        let mut store = TrustedManifestStore::new();
        discover_and_admit(&fixture_config("valid"), TRUSTED_MANIFEST, &mut store).unwrap();
        assert!(matches!(
            resolver::resolve_capability(
                &store,
                &ProviderAvailability::empty(),
                "fixture.ping",
                1,
                Some("tethers-stdio-fixture"),
            ),
            Err(resolver::ResolutionError::ProviderUnavailable { .. })
        ));
    }

    #[test]
    fn initialization_error_fails_closed() {
        assert_mode_fails_closed("initialization-error", "JSON-RPC error");
    }
    #[test]
    fn incompatible_protocol_version_fails_closed() {
        assert_mode_fails_closed("incompatible-version", "incompatible MCP protocol");
    }
    #[test]
    fn server_name_mismatch_fails_closed() {
        assert_mode_fails_closed("server-name-mismatch", "server name did not match");
    }
    #[test]
    fn malformed_json_rpc_fails_closed() {
        assert_mode_fails_closed("malformed-json", "not valid JSON");
    }
    #[test]
    fn missing_tool_fails_closed() {
        assert_mode_fails_closed("missing-tool", "did not contain trusted binding tool");
    }
    #[test]
    fn duplicate_tool_fails_closed() {
        assert_mode_fails_closed("duplicate-tool", "duplicate tool name");
    }
    #[test]
    fn wrong_tool_name_fails_closed() {
        assert_mode_fails_closed("wrong-tool", "did not contain trusted binding tool");
    }
    #[test]
    fn j13b_live_tools_list_schema_mismatch_is_not_admitted_available() {
        assert_mode_fails_closed("input-schema-mismatch", "input schema did not match");
    }
    #[test]
    fn output_schema_mismatch_fails_closed() {
        assert_mode_fails_closed("output-schema-mismatch", "output schema did not match");
    }
    #[test]
    fn premature_process_exit_fails_closed() {
        assert_mode_fails_closed("exit-early", "");
    }
    #[test]
    fn missing_host_digest_pin_fails_before_launch() {
        let mut cfg = fixture_config("valid");
        cfg.provider_config.allowed_capabilities[0].pinned_digest = None;
        let mut store = TrustedManifestStore::new();
        let e = discover_and_admit(&cfg, TRUSTED_MANIFEST, &mut store).unwrap_err();
        assert!(e.to_string().contains("must pin the digest"));
        assert!(store.is_empty());
    }
    #[test]
    fn wrong_host_digest_pin_fails_before_launch() {
        let mut cfg = fixture_config("valid");
        cfg.provider_config.allowed_capabilities[0].pinned_digest = Some(
            "sha256:0000000000000000000000000000000000000000000000000000000000000000".to_owned(),
        );
        let mut store = TrustedManifestStore::new();
        let e = discover_and_admit(&cfg, TRUSTED_MANIFEST, &mut store).unwrap_err();
        assert!(e.to_string().contains("host-pinned digest"));
        assert!(store.is_empty());
    }
    #[test]
    fn tampered_trusted_manifest_digest_fails_before_launch() {
        let tampered = TRUSTED_MANIFEST.replace(
            TRUSTED_DIGEST,
            "sha256:0000000000000000000000000000000000000000000000000000000000000000",
        );
        let mut store = TrustedManifestStore::new();
        let e = discover_and_admit(&fixture_config("valid"), &tampered, &mut store).unwrap_err();
        assert!(matches!(e, StdioProviderError::TrustedManifestInvalid(_)));
        assert!(store.is_empty());
    }
    #[test]
    fn wrong_host_identity_fails_before_launch() {
        let mut cfg = fixture_config("valid");
        cfg.provider_config.identity = "wrong-provider".to_owned();
        let mut store = TrustedManifestStore::new();
        let e = discover_and_admit(&cfg, TRUSTED_MANIFEST, &mut store).unwrap_err();
        assert!(e.to_string().contains("IdentityMismatch"));
        assert!(store.is_empty());
    }
    #[test]
    fn wrong_host_capability_version_fails_before_launch() {
        let mut cfg = fixture_config("valid");
        cfg.provider_config.allowed_capabilities[0].capability_version = 2;
        let mut store = TrustedManifestStore::new();
        let e = discover_and_admit(&cfg, TRUSTED_MANIFEST, &mut store).unwrap_err();
        assert!(e.to_string().contains("does not allow exact capability"));
        assert!(store.is_empty());
    }
    #[test]
    fn capability_absent_from_host_allow_list_fails_before_launch() {
        let mut cfg = fixture_config("valid");
        cfg.provider_config.allowed_capabilities[0].capability_name = "fixture.other".to_owned();
        let mut store = TrustedManifestStore::new();
        let e = discover_and_admit(&cfg, TRUSTED_MANIFEST, &mut store).unwrap_err();
        assert!(e.to_string().contains("does not allow exact capability"));
        assert!(store.is_empty());
    }
    #[test]
    fn nonexistent_command_fails_closed_without_admission() {
        let mut cfg = fixture_config("valid");
        cfg.command = "nonexistent-command-hopefully-xyzzy".to_owned();
        let mut store = TrustedManifestStore::new();
        let e = discover_and_admit(&cfg, TRUSTED_MANIFEST, &mut store).unwrap_err();
        assert!(matches!(e, StdioProviderError::LaunchFailed { .. }));
        assert!(store.is_empty());
    }
}
