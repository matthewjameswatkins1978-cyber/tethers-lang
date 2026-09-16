//! Bounded OpenShell effect adapter for the Lantern Keeper demo capability.
//!
//! Tethers remains the authority boundary. This adapter only receives a
//! `DispatchReadyAction`, validates the already-resolved demo identity again,
//! and invokes one fixed OpenShell sandbox path. It never decides policy,
//! grants authority, or falls back to an unsandboxed process.

use crate::child_process::{ChildConfig, SupervisedChild};
use crate::dispatch::DispatchReadyAction;
use crate::executor::CapabilityExecutor;
use crate::outcome::ProviderDiagnostic;
use serde_json::{json, Value};
use std::time::Duration;
use std::time::Instant;

pub const DEMO_CAPABILITY_NAME: &str = "demo.export_summary";
pub const DEMO_CAPABILITY_VERSION: u32 = 1;
pub const APPROVED_SANDBOX_PATH: &str = "/sandbox/outbox/approved/summary.txt";

const DEFAULT_DISTRIBUTION: &str = "Ubuntu";
const DEFAULT_OPEN_SHELL_BINARY: &str = "/home/matmus/.local/openshell-0.0.116/usr/bin/openshell";
const DEFAULT_GATEWAY_ENDPOINT: &str = "https://127.0.0.1:17670";
const DEFAULT_SANDBOX_NAME: &str = "lantern-m4-demo";
const MAX_SUMMARY_BYTES: usize = 4096;

/// Explicit local configuration for the OpenShell demo lane.
#[derive(Debug, Clone)]
pub struct OpenShellExecutorConfig {
    pub wsl_command: String,
    pub distribution: String,
    pub openshell_binary: String,
    pub gateway_endpoint: String,
    pub gateway_insecure: bool,
    pub sandbox_name: String,
    pub provider_identity: String,
    pub approved_path: String,
    pub timeout: Duration,
}

impl Default for OpenShellExecutorConfig {
    fn default() -> Self {
        Self {
            wsl_command: "wsl.exe".to_owned(),
            distribution: DEFAULT_DISTRIBUTION.to_owned(),
            openshell_binary: DEFAULT_OPEN_SHELL_BINARY.to_owned(),
            gateway_endpoint: DEFAULT_GATEWAY_ENDPOINT.to_owned(),
            gateway_insecure: true,
            sandbox_name: DEFAULT_SANDBOX_NAME.to_owned(),
            provider_identity: "openshell-lantern-local".to_owned(),
            approved_path: APPROVED_SANDBOX_PATH.to_owned(),
            timeout: Duration::from_secs(30),
        }
    }
}

impl OpenShellExecutorConfig {
    /// Load only non-secret, explicit demo settings from the environment.
    pub fn from_env() -> Result<Self, String> {
        let mut config = Self::default();
        for (name, target) in [
            ("LANTERN_OPENSHELL_WSL_COMMAND", &mut config.wsl_command),
            ("LANTERN_OPENSHELL_DISTRIBUTION", &mut config.distribution),
            ("LANTERN_OPENSHELL_BINARY", &mut config.openshell_binary),
            (
                "LANTERN_OPENSHELL_GATEWAY_ENDPOINT",
                &mut config.gateway_endpoint,
            ),
            ("LANTERN_OPENSHELL_SANDBOX", &mut config.sandbox_name),
            (
                "LANTERN_OPENSHELL_PROVIDER_IDENTITY",
                &mut config.provider_identity,
            ),
        ] {
            if let Some(value) = std::env::var_os(name) {
                *target = value
                    .into_string()
                    .map_err(|_| format!("{name} is not UTF-8"))?;
            }
        }
        if config.approved_path != APPROVED_SANDBOX_PATH {
            return Err("OpenShell demo path is not the fixed approved path".to_owned());
        }
        if config.wsl_command.is_empty()
            || config.distribution.is_empty()
            || config.openshell_binary.is_empty()
            || config.gateway_endpoint.is_empty()
            || config.sandbox_name.is_empty()
            || config.provider_identity.is_empty()
        {
            return Err("OpenShell demo configuration contains an empty value".to_owned());
        }
        Ok(config)
    }
}

pub struct OpenShellExecutor {
    config: OpenShellExecutorConfig,
}

impl OpenShellExecutor {
    pub fn new(config: OpenShellExecutorConfig) -> Result<Self, String> {
        if config.approved_path != APPROVED_SANDBOX_PATH {
            return Err("OpenShell executor requires its fixed approved path".to_owned());
        }
        if config.provider_identity.is_empty() {
            return Err("OpenShell executor requires a provider identity".to_owned());
        }
        Ok(Self { config })
    }

    fn validate_ready(&self, ready: &DispatchReadyAction) -> Result<String, String> {
        if ready.capability_name() != DEMO_CAPABILITY_NAME
            || ready.capability_version() != DEMO_CAPABILITY_VERSION
            || ready.provider_identity() != self.config.provider_identity
        {
            return Err("OpenShell executor received a mismatched dispatch proof".to_owned());
        }

        let arguments = ready
            .arguments()
            .as_object()
            .ok_or_else(|| "demo summary arguments must be an object".to_owned())?;
        if arguments.len() != 1 || !arguments.contains_key("summary") {
            return Err("demo summary arguments contain an unsupported field".to_owned());
        }
        let summary = arguments["summary"]
            .as_str()
            .ok_or_else(|| "demo summary must be a string".to_owned())?;
        if summary.is_empty()
            || summary.len() > MAX_SUMMARY_BYTES
            || summary.contains('\r')
            || summary.contains('\n')
        {
            return Err("demo summary must be one bounded line".to_owned());
        }
        Ok(summary.to_owned())
    }

    fn command_args(&self) -> Vec<String> {
        let mut args = vec![
            "-d".to_owned(),
            self.config.distribution.clone(),
            "--".to_owned(),
            self.config.openshell_binary.clone(),
            "sandbox".to_owned(),
            "exec".to_owned(),
            "--gateway-endpoint".to_owned(),
            self.config.gateway_endpoint.clone(),
        ];
        if self.config.gateway_insecure {
            args.push("--gateway-insecure".to_owned());
        }
        args.extend([
            "--name".to_owned(),
            self.config.sandbox_name.clone(),
            "--no-tty".to_owned(),
            "--".to_owned(),
            "/usr/bin/tee".to_owned(),
            self.config.approved_path.clone(),
        ]);
        args
    }

    fn invoke(&self, summary: &str, remaining: Duration) -> Result<Value, String> {
        let mut config =
            ChildConfig::production(self.config.wsl_command.clone(), self.command_args());
        config.clear_environment = true;
        // wsl.exe needs the Windows installation root to locate its host
        // service. This allowlist contains only system paths, never tokens or
        // provider credentials.
        for name in ["SystemRoot", "WINDIR"] {
            if let Ok(value) = std::env::var(name) {
                config.environment.insert(name.to_owned(), value);
            }
        }
        config.startup_timeout = remaining.min(Duration::from_secs(10));
        config.graceful_close_timeout = Duration::from_secs(2);
        let mut child = SupervisedChild::launch(config)
            .map_err(|_| "OpenShell sandbox command could not be launched".to_owned())?;

        let result = (|| {
            child
                .write_line(summary)
                .map_err(|_| "OpenShell sandbox command input failed".to_owned())?;
            child.close_stdin();
            let deadline = Instant::now() + remaining;
            let mut confirmed = false;
            for _ in 0..8 {
                let wait = deadline.saturating_duration_since(Instant::now());
                if wait.is_zero() {
                    break;
                }
                let line = child
                    .read_protocol_line(wait)
                    .map_err(|_| "OpenShell sandbox result was not confirmed".to_owned())?;
                if line.trim_end_matches(['\r', '\n']) == summary {
                    confirmed = true;
                    break;
                }
            }
            if !confirmed {
                return Err(
                    "OpenShell sandbox result did not confirm the submitted summary".to_owned(),
                );
            }
            Ok(json!({
                "sandbox": self.config.sandbox_name,
                "path": APPROVED_SANDBOX_PATH,
                "summary": summary,
            }))
        })();
        let _cleanup = child.shutdown();
        result
    }
}

impl CapabilityExecutor for OpenShellExecutor {
    fn provider_identity(&self) -> &str {
        &self.config.provider_identity
    }

    fn execute(&mut self, ready: &DispatchReadyAction) -> Result<Value, String> {
        let summary = self.validate_ready(ready)?;
        self.invoke(&summary, self.config.timeout)
    }

    fn execute_classified(
        &mut self,
        ready: &DispatchReadyAction,
        remaining: Duration,
    ) -> Result<Value, ProviderDiagnostic> {
        let summary = self
            .validate_ready(ready)
            .map_err(|_| ProviderDiagnostic::ExplicitProviderError)?;
        self.invoke(&summary, remaining)
            .map_err(|_| ProviderDiagnostic::NoFinalResponse)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(windows)]
    use crate::enablement::{EnabledBindingSnapshot, EnabledCapability};
    #[cfg(windows)]
    use crate::policy::CapabilityRequirement;
    #[cfg(windows)]
    use crate::resolver::{self, ProviderAvailability};
    #[cfg(windows)]
    use crate::trusted_store::TrustedManifestStore;
    #[cfg(windows)]
    use serde_json::json;
    #[cfg(windows)]
    use std::path::PathBuf;
    #[cfg(windows)]
    use std::process::Command;

    #[cfg(windows)]
    fn demo_manifest() -> Value {
        json!({
            "manifest_format_version": "1.0",
            "capability_name": DEMO_CAPABILITY_NAME,
            "capability_version": DEMO_CAPABILITY_VERSION,
            "title": "Export a synthetic summary",
            "description": "Write one bounded synthetic summary into the OpenShell outbox.",
            "input_schema": {
                "type": "object",
                "properties": { "summary": { "type": "string", "maxLength": MAX_SUMMARY_BYTES } },
                "required": ["summary"],
                "additionalProperties": false
            },
            "output_schema": {
                "type": "object",
                "properties": {
                    "sandbox": { "type": "string" },
                    "path": { "type": "string" },
                    "summary": { "type": "string" }
                },
                "required": ["sandbox", "path", "summary"],
                "additionalProperties": false
            },
            "effects": ["filesystem.write"],
            "permission_scope": {
                "kind": "path_prefix",
                "allowed_prefixes": ["sandbox/"]
            },
            "reversibility": "reversible",
            "determinism": "deterministic",
            "idempotency": { "mechanism": "none" },
            "confirmation_policy": { "standing_permitted": true, "per_call_required": false },
            "timeout_ms": 30000,
            "retry_policy": {
                "max_retries": 0,
                "backoff_ms": 0,
                "allowed_on": [],
                "requires_idempotency_proof": false
            },
            "provider": {
                "identity": "openshell-lantern-local",
                "display_name": "OpenShell Lantern demo",
                "identity_source": "host_configuration",
                "description": "Explicit local OpenShell sandbox lane."
            },
            "binding": {
                "kind": "mcp",
                "server_name": "openshell",
                "tool_name": DEMO_CAPABILITY_NAME,
                "adapter": null
            }
        })
    }

    #[cfg(windows)]
    fn resolved_demo() -> (TrustedManifestStore, resolver::ResolvedCapability) {
        let mut manifest = demo_manifest();
        let (_, digest) = crate::manifest::canonicalize_and_digest(&manifest.to_string()).unwrap();
        manifest["digest"] = json!(digest);
        let mut store = TrustedManifestStore::new();
        store
            .insert(crate::manifest::verify_manifest(&manifest.to_string()).unwrap())
            .unwrap();
        let availability = ProviderAvailability::from_identities(["openshell-lantern-local"]);
        let resolved = resolver::resolve_capability(
            &store,
            &availability,
            DEMO_CAPABILITY_NAME,
            DEMO_CAPABILITY_VERSION,
            Some("openshell-lantern-local"),
        )
        .unwrap();
        (store, resolved)
    }

    #[cfg(windows)]
    fn response_for(resolved: &resolver::ResolvedCapability) -> Value {
        json!({
            "evaluation_id": "eval-openshell-m4",
            "plan": {
                "id": "plan-openshell-m4",
                "required_effects": ["filesystem.write"],
                "actions": [{
                    "action_id": "action-openshell-m4",
                    "idempotency_key": "eval-openshell-m4/action-openshell-m4",
                    "capability": DEMO_CAPABILITY_NAME,
                    "capability_version": "1.0.0",
                    "bridge_capability_version": DEMO_CAPABILITY_VERSION,
                    "manifest_digest": resolved.manifest_digest(),
                    "bridge_provider_identity": resolved.provider_identity(),
                    "arguments": { "summary": "Lantern synthetic summary" }
                }]
            },
            "trail": []
        })
    }

    #[test]
    fn command_is_fixed_to_the_approved_sandbox_path() {
        let executor = OpenShellExecutor::new(OpenShellExecutorConfig::default()).unwrap();
        let args = executor.command_args();
        assert_eq!(args.last().map(String::as_str), Some(APPROVED_SANDBOX_PATH));
        assert!(!args
            .iter()
            .any(|arg| arg == "/sandbox/outbox/forbidden/summary.txt"));
    }

    #[test]
    fn child_configuration_does_not_inherit_ambient_environment() {
        let mut config = ChildConfig::production("wsl.exe", Vec::new());
        config.clear_environment = true;
        assert!(config.clear_environment);
    }

    /// Real OpenShell/Tethers boundary test. It is opt-in because ordinary
    /// repository tests must not require Docker, WSL, or a running gateway.
    #[cfg(windows)]
    #[test]
    fn valid_grant_executes_once_in_openshell() {
        if std::env::var("LANTERN_OPENSHELL_E2E").ok().as_deref() != Some("1") {
            return;
        }

        let (_store, resolved) = resolved_demo();
        let mut response = response_for(&resolved);
        let action = response["plan"]["actions"][0].clone();
        let requirements = vec![CapabilityRequirement::new(
            DEMO_CAPABILITY_NAME,
            DEMO_CAPABILITY_VERSION,
        )];
        let enabled = EnabledBindingSnapshot {
            installed_id: "installed-openshell-demo".to_owned(),
            provider_id: "openshell-lantern-local".to_owned(),
            capabilities: vec![EnabledCapability {
                name: DEMO_CAPABILITY_NAME.to_owned(),
                version: DEMO_CAPABILITY_VERSION,
                manifest_digest: resolved.manifest_digest().to_owned(),
                provider_operation_name: DEMO_CAPABILITY_NAME.to_owned(),
            }],
        };
        let mut executor =
            OpenShellExecutor::new(OpenShellExecutorConfig::from_env().unwrap()).unwrap();
        let base =
            std::env::temp_dir().join(format!("tethers-openshell-m4-{}", std::process::id()));
        let root = base.join("replay-root");
        let trail_path = base.join("trail.jsonl");
        let replay_root = root.clone();
        std::fs::create_dir_all(&root).unwrap();
        let acl_script = format!(
            "$p='{}'; $identity=[System.Security.Principal.WindowsIdentity]::GetCurrent().Name; $acl=[System.Security.AccessControl.DirectorySecurity]::new(); $acl.SetAccessRuleProtection($true,$false); $inherit=[System.Security.AccessControl.InheritanceFlags]::ContainerInherit -bor [System.Security.AccessControl.InheritanceFlags]::ObjectInherit; foreach($t in @($identity,'NT AUTHORITY\\SYSTEM','BUILTIN\\Administrators')) {{ $acl.AddAccessRule([System.Security.AccessControl.FileSystemAccessRule]::new($t,'FullControl',$inherit,'None','Allow')) }}; Set-Acl -LiteralPath $p -AclObject $acl",
            root.to_string_lossy()
        );
        assert!(Command::new("pwsh.exe")
            .args(["-NoProfile", "-Command", &acl_script])
            .status()
            .unwrap()
            .success());
        crate::replay_windows::provision_replay(&root).unwrap();

        let result = crate::host_execution::execute_enabled_installed_action(
            &mut response,
            &requirements,
            &resolved,
            &enabled,
            &mut executor,
            &trail_path,
            &replay_root,
            "event-openshell-m4",
        )
        .unwrap();

        assert_eq!(
            result.outcome,
            crate::SharedExecutionOutcome::Completed,
            "OpenShell/Tethers execution response: {response}"
        );
        assert_eq!(response["execution_status"], "completed");
        assert_eq!(action["capability"], DEMO_CAPABILITY_NAME);
        let _ = std::fs::remove_dir_all(PathBuf::from(base));
    }
}
