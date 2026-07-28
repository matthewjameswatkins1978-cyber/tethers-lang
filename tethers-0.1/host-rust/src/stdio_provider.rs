// Host-owned stdio MCP provider binding.
//
// The provider advertises untrusted MCP tool metadata. A separately authored,
// host-owned trusted manifest defines the authoritative capability contract.
// Discovery may prove that the live advertisement matches that contract; it
// may never create or modify the contract.
//
// J13A: Refactored ManagedProvider to support retained provider sessions.
// One provider process may remain alive after initialize/tools-list for
// the duration of the check command.

use crate::child_process::{ChildConfig, ChildError, SupervisedChild};
use crate::manifest::{self, BindingKind, VerifiedManifest};
use crate::provider::{self, AdmissionError, ProviderConfig};
use crate::trusted_store::TrustedManifestStore;
use std::collections::HashSet;
use std::fmt;
use std::path::PathBuf;

const INITIALIZE_REQUEST_ID: u64 = 1;
const TOOLS_LIST_REQUEST_ID: u64 = 2;

/// A running provider process owned by the host.
///
/// J13A: Uses SupervisedChild for Job Object supervision and
/// supports retained sessions.
pub struct ManagedProvider {
    child: SupervisedChild,
    next_request_id: u64,
}

impl ManagedProvider {
    /// Launch the exact executable and arguments supplied by host configuration,
    /// with an explicit current directory.
    pub fn launch(
        command: &str,
        args: &[String],
        working_dir: &PathBuf,
        startup_timeout_override: Option<std::time::Duration>,
        graceful_close_timeout_override: Option<std::time::Duration>,
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

        let mut child =
            SupervisedChild::launch(config).map_err(|e| StdioProviderError::LaunchFailed {
                command: command.to_owned(),
                message: e.to_string(),
            })?;

        Ok(Self {
            child,
            next_request_id: INITIALIZE_REQUEST_ID + 1,
        })
    }

    fn write_message(&mut self, message: &serde_json::Value) -> Result<(), StdioProviderError> {
        let line = serde_json::to_string(message)
            .map_err(|error| StdioProviderError::SerializeFailed(error.to_string()))?;
        self.child
            .write_line(&line)
            .map_err(|_| StdioProviderError::WriteFailed("write to provider failed".to_owned()))
    }

    fn read_message(&mut self) -> Result<serde_json::Value, StdioProviderError> {
        let line = self.child.read_protocol_line().map_err(|e| match e {
            ChildError::ReadTimeout(msg) => StdioProviderError::ReadFailed(msg),
            ChildError::ProtocolError(msg) => StdioProviderError::MalformedResponse(msg),
            ChildError::ProcessExited(_) => StdioProviderError::EmptyResponse,
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
        self.write_message(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        }))?;

        let response = self.read_message()?;
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
            return Err(StdioProviderError::ProtocolError(format!(
                "provider returned JSON-RPC error for {method}: {error}"
            )));
        }

        object.get("result").cloned().ok_or_else(|| {
            StdioProviderError::ProtocolError(format!(
                "JSON-RPC response for {method} had no result"
            ))
        })
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

    /// Initialize the provider, verifying protocol version and server identity.
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
                    "version": "0.1.0"
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
        let result = self.request(TOOLS_LIST_REQUEST_ID, "tools/list", serde_json::json!({}))?;
        result
            .get("tools")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .ok_or_else(|| {
                StdioProviderError::ProtocolError(
                    "tools/list result must contain a tools array".to_owned(),
                )
            })
    }

    /// Get the retained stderr diagnostic tail.
    pub fn stderr_tail(&self) -> String {
        self.child.stderr_tail()
    }

    /// Close the provider (shut down gracefully).
    /// Drops stdin to signal EOF; caller should drop this value
    /// to trigger full Job Object cleanup through Drop.
    pub fn close(&mut self) {
        let _ = self.child.write_line("");
    }
}

impl Drop for ManagedProvider {
    fn drop(&mut self) {
        // SupervisedChild's Drop handles Job Object termination.
    }
}

/// Errors produced by the fail-closed stdio MCP admission boundary.
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
    TrustedManifestInvalid(String),
    AdmissionFailed(String),
}

impl fmt::Display for StdioProviderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LaunchFailed { command, message } => {
                write!(formatter, "launch failed for '{command}': {message}")
            }
            Self::StdinUnavailable => write!(formatter, "stdin unavailable"),
            Self::StdoutUnavailable => write!(formatter, "stdout unavailable"),
            Self::SerializeFailed(message) => write!(formatter, "serialization failed: {message}"),
            Self::WriteFailed(message) => write!(formatter, "write failed: {message}"),
            Self::ReadFailed(message) => write!(formatter, "read failed: {message}"),
            Self::EmptyResponse => write!(formatter, "provider returned no response"),
            Self::MalformedResponse(message) => write!(formatter, "malformed response: {message}"),
            Self::ProtocolError(message) => write!(formatter, "MCP protocol error: {message}"),
            Self::TrustedManifestInvalid(message) => {
                write!(formatter, "trusted manifest invalid: {message}")
            }
            Self::AdmissionFailed(message) => write!(formatter, "admission failed: {message}"),
        }
    }
}

/// Host-owned configuration for one explicitly configured stdio provider.
#[derive(Debug, Clone)]
pub struct StdioProviderConfig {
    pub command: String,
    pub args: Vec<String>,
    pub protocol_version: String,
    pub provider_config: ProviderConfig,
}

fn admission_error(error: AdmissionError) -> StdioProviderError {
    StdioProviderError::AdmissionFailed(format!("{error:?}"))
}

fn validate_host_binding(
    config: &ProviderConfig,
    trusted: &VerifiedManifest,
) -> Result<(), StdioProviderError> {
    let allowed = config
        .allowed_capabilities
        .iter()
        .find(|allowed| {
            allowed.capability_name == trusted.capability_name()
                && allowed.capability_version == trusted.capability_version()
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

    // Validate the remainder of the existing provider-admission contract
    // without mutating the caller's Trusted Manifest Store.
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
    let mut matching = None;

    for tool in tools {
        let object = tool.as_object().ok_or_else(|| {
            StdioProviderError::ProtocolError("every tools/list entry must be an object".to_owned())
        })?;
        let name = object
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
            matching = Some(object);
        }
    }

    matching.ok_or_else(|| {
        StdioProviderError::ProtocolError(format!(
            "tools/list did not contain trusted binding tool '{expected_tool_name}'"
        ))
    })
}

/// Compare live discovery evidence against the trusted manifest.
/// J13A: shared function used by both discover_and_admit and the check command.
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

/// Admit a separately authored trusted manifest only after a live MCP provider
/// proves that its untrusted discovery evidence matches the host-owned binding.
///
/// J13A: Legacy compatibility wrapper that still launches/tears down per call.
/// The check command uses the retained-session path instead.
pub fn discover_and_admit(
    config: &StdioProviderConfig,
    trusted_manifest_json: &str,
    store: &mut TrustedManifestStore,
) -> Result<VerifiedManifest, StdioProviderError> {
    let trusted = manifest::verify_manifest(trusted_manifest_json)
        .map_err(|error| StdioProviderError::TrustedManifestInvalid(format!("{error:?}")))?;
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

    // Close retained provider session.
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
        let error =
            discover_and_admit(&fixture_config(mode), TRUSTED_MANIFEST, &mut store).unwrap_err();
        assert!(
            error.to_string().contains(expected),
            "unexpected error for mode {mode}: {error}"
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

    #[test]
    fn provider_description_cannot_rewrite_trusted_manifest() {
        let mut store = TrustedManifestStore::new();
        let verified = discover_and_admit(
            &fixture_config("changed-description"),
            TRUSTED_MANIFEST,
            &mut store,
        )
        .unwrap();

        assert_eq!(
            verified.manifest().description,
            "A deterministic test capability for stdio provider binding."
        );
        assert_eq!(verified.verified_digest(), TRUSTED_DIGEST);
    }

    #[test]
    fn admitted_fixture_resolves_only_when_host_reports_it_available() {
        let mut store = TrustedManifestStore::new();
        let verified =
            discover_and_admit(&fixture_config("valid"), TRUSTED_MANIFEST, &mut store).unwrap();
        let availability = ProviderAvailability::from_identities(["tethers-stdio-fixture"]);

        let resolved = resolver::resolve_capability(
            &store,
            &availability,
            "fixture.ping",
            1,
            Some("tethers-stdio-fixture"),
        )
        .unwrap();
        assert_eq!(resolved.manifest_digest(), verified.verified_digest());
    }

    #[test]
    fn admitted_fixture_is_unavailable_without_live_host_evidence() {
        let mut store = TrustedManifestStore::new();
        discover_and_admit(&fixture_config("valid"), TRUSTED_MANIFEST, &mut store).unwrap();

        let error = resolver::resolve_capability(
            &store,
            &ProviderAvailability::empty(),
            "fixture.ping",
            1,
            Some("tethers-stdio-fixture"),
        )
        .unwrap_err();
        assert!(matches!(
            error,
            resolver::ResolutionError::ProviderUnavailable { .. }
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
    fn input_schema_mismatch_fails_closed() {
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
        let mut config = fixture_config("valid");
        config.provider_config.allowed_capabilities[0].pinned_digest = None;
        let mut store = TrustedManifestStore::new();

        let error = discover_and_admit(&config, TRUSTED_MANIFEST, &mut store).unwrap_err();
        assert!(error.to_string().contains("must pin the digest"));
        assert!(store.is_empty());
    }

    #[test]
    fn wrong_host_digest_pin_fails_before_launch() {
        let mut config = fixture_config("valid");
        config.provider_config.allowed_capabilities[0].pinned_digest = Some(
            "sha256:0000000000000000000000000000000000000000000000000000000000000000".to_owned(),
        );
        let mut store = TrustedManifestStore::new();

        let error = discover_and_admit(&config, TRUSTED_MANIFEST, &mut store).unwrap_err();
        assert!(error.to_string().contains("host-pinned digest"));
        assert!(store.is_empty());
    }

    #[test]
    fn tampered_trusted_manifest_digest_fails_before_launch() {
        let tampered = TRUSTED_MANIFEST.replace(
            TRUSTED_DIGEST,
            "sha256:0000000000000000000000000000000000000000000000000000000000000000",
        );
        let mut store = TrustedManifestStore::new();

        let error =
            discover_and_admit(&fixture_config("valid"), &tampered, &mut store).unwrap_err();
        assert!(matches!(
            error,
            StdioProviderError::TrustedManifestInvalid(_)
        ));
        assert!(store.is_empty());
    }

    #[test]
    fn wrong_host_identity_fails_before_launch() {
        let mut config = fixture_config("valid");
        config.provider_config.identity = "wrong-provider".to_owned();
        let mut store = TrustedManifestStore::new();

        let error = discover_and_admit(&config, TRUSTED_MANIFEST, &mut store).unwrap_err();
        assert!(error.to_string().contains("IdentityMismatch"));
        assert!(store.is_empty());
    }

    #[test]
    fn wrong_host_capability_version_fails_before_launch() {
        let mut config = fixture_config("valid");
        config.provider_config.allowed_capabilities[0].capability_version = 2;
        let mut store = TrustedManifestStore::new();

        let error = discover_and_admit(&config, TRUSTED_MANIFEST, &mut store).unwrap_err();
        assert!(error
            .to_string()
            .contains("does not allow exact capability"));
        assert!(store.is_empty());
    }

    #[test]
    fn capability_absent_from_host_allow_list_fails_before_launch() {
        let mut config = fixture_config("valid");
        config.provider_config.allowed_capabilities[0].capability_name = "fixture.other".to_owned();
        let mut store = TrustedManifestStore::new();

        let error = discover_and_admit(&config, TRUSTED_MANIFEST, &mut store).unwrap_err();
        assert!(error
            .to_string()
            .contains("does not allow exact capability"));
        assert!(store.is_empty());
    }

    #[test]
    fn nonexistent_command_fails_closed_without_admission() {
        let mut config = fixture_config("valid");
        config.command = "nonexistent-command-hopefully-xyzzy".to_owned();
        let mut store = TrustedManifestStore::new();

        let error = discover_and_admit(&config, TRUSTED_MANIFEST, &mut store).unwrap_err();
        assert!(matches!(error, StdioProviderError::LaunchFailed { .. }));
        assert!(store.is_empty());
    }
}
