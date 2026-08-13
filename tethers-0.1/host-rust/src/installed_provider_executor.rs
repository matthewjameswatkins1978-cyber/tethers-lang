//! Generic retained executor for a trusted installed MCP provider.

use crate::dispatch::DispatchReadyAction;
use crate::enablement::EnablementRecord;
use crate::executor::CapabilityExecutor;
use crate::installed::{InstallationApprovalRecord, InstalledPlugRecord};
use crate::launch_profile::launch_installed_provider;
use crate::manifest::VerifiedManifest;
use crate::operational_scope::OperationalScopeEvidence;
use crate::outcome::ProviderDiagnostic;
use crate::socket::RetainedProviderSession;
use crate::stdio_provider::{compare_discovery_evidence, StdioProviderError};
use crate::trust::{DeveloperApprovalStore, PackageTrustEvidence, PublisherTrustStore};
use serde_json::Value;
use std::path::Path;
use std::time::Duration;

/// A generic installed-provider executor. It knows only the trusted package
/// record and verified manifest supplied by the host; capability family meaning
/// remains with the external provider.
pub struct InstalledProviderExecutor {
    provider_identity: String,
    operation: String,
    session: RetainedProviderSession,
}

impl InstalledProviderExecutor {
    #[allow(clippy::too_many_arguments)]
    pub fn launch_from_installed(
        record: &InstalledPlugRecord,
        installed_directory: &Path,
        trust: &PackageTrustEvidence,
        publisher_trust: &PublisherTrustStore,
        developer_approvals: &DeveloperApprovalStore,
        conformance: &crate::conformance::ConformanceEvidence,
        approval: &InstallationApprovalRecord,
        enablement: &EnablementRecord,
        scope: &OperationalScopeEvidence,
        trusted: VerifiedManifest,
    ) -> Result<Self, StdioProviderError> {
        let manifest = trusted.manifest();
        if record.provider_id != manifest.provider.identity {
            return Err(StdioProviderError::AdmissionFailed(
                "installed provider identity did not match trusted manifest".into(),
            ));
        }
        let child = launch_installed_provider(
            record,
            installed_directory,
            trust,
            publisher_trust,
            developer_approvals,
            conformance,
            approval,
            enablement,
            scope,
        )
        .map_err(|error| StdioProviderError::LaunchFailed {
            command: record.launch_path.clone(),
            message: error.to_string(),
        })?;
        let mut session = RetainedProviderSession::from_supervised_child(
            child,
            "2025-11-25",
            &manifest.binding.server_name,
            record.provider_id.clone(),
        )?;
        let tools = session.discover_tools()?;
        if let Err(error) = compare_discovery_evidence(&tools, &trusted) {
            session.close();
            return Err(error);
        }
        Ok(Self {
            provider_identity: record.provider_id.clone(),
            operation: manifest.binding.tool_name.clone(),
            session,
        })
    }

    pub fn call(
        &mut self,
        operation: &str,
        arguments: &Value,
        remaining: Duration,
    ) -> Result<Value, ProviderDiagnostic> {
        if operation != self.operation {
            return Err(ProviderDiagnostic::NoFinalResponse);
        }
        self.session
            .tools_call(operation, arguments, remaining)
            .map_err(|error| match error {
                StdioProviderError::ExplicitProviderError(_) => {
                    ProviderDiagnostic::ExplicitProviderError
                }
                _ => ProviderDiagnostic::NoFinalResponse,
            })
    }
}

impl CapabilityExecutor for InstalledProviderExecutor {
    fn provider_identity(&self) -> &str {
        &self.provider_identity
    }

    fn execute(&mut self, ready: &DispatchReadyAction) -> Result<Value, String> {
        self.call(
            &ready.verified_manifest().manifest().binding.tool_name,
            ready.arguments(),
            Duration::from_secs(5),
        )
        .map_err(|error| format!("{error:?}"))
    }

    fn execute_classified(
        &mut self,
        ready: &DispatchReadyAction,
        remaining: Duration,
    ) -> Result<Value, ProviderDiagnostic> {
        self.call(
            &ready.verified_manifest().manifest().binding.tool_name,
            ready.arguments(),
            remaining,
        )
    }
}

impl Drop for InstalledProviderExecutor {
    fn drop(&mut self) {
        self.session.close();
    }
}
