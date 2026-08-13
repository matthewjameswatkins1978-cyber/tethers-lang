//! Semantic operations over a retained provider session.
//!
//! Socket translates host lifecycle operations to the retained MCP binding. It
//! returns protocol observations only: trust, policy, approval, canonical
//! outcomes, replay, Result Anchors, Trails and retry remain host-owned.

use crate::stdio_provider::{ManagedProvider, StdioProviderError};
use serde_json::Value;
use std::path::PathBuf;
use std::time::Duration;

/// Host-supplied launch and MCP-binding details for one local stdio Socket.
pub struct SocketEstablishment<'a> {
    pub command: &'a str,
    pub args: &'a [String],
    pub working_directory: &'a PathBuf,
    pub protocol_version: &'a str,
    pub server_name: &'a str,
    pub identity: &'a str,
}

/// Complete untrusted operation catalogue observed from one discovery cycle.
/// Descriptions, annotations and any provider extensions remain observations;
/// the host compares only trusted binding fields before making operations
/// available.
#[derive(Debug, Clone, PartialEq)]
pub struct SocketCatalogue {
    operations: Vec<Value>,
}

impl SocketCatalogue {
    pub fn operations(&self) -> &[Value] {
        &self.operations
    }
}

/// Accepted Socket v1 operations over a serial retained session.
pub trait Socket: Sized {
    fn establish(spec: SocketEstablishment<'_>) -> Result<Self, StdioProviderError>;
    fn discover(&mut self) -> Result<SocketCatalogue, StdioProviderError>;
    fn invoke(
        &mut self,
        operation: &str,
        arguments: &Value,
        remaining: Duration,
    ) -> Result<Value, StdioProviderError>;
    fn observe_result(&mut self, result: Value) -> Value;
    fn observe_catalogue_change(&mut self) -> Result<bool, StdioProviderError>;
    fn probe(&mut self) -> Result<Value, StdioProviderError>;
    fn close(&mut self);
}

/// One retained MCP stdio Socket with a single monotonically increasing
/// request-ID sequence and no retry or restart queue.
pub struct RetainedProviderSession {
    provider: ManagedProvider,
    next_request_id: u64,
    identity: String,
    catalogue_stale: bool,
    catalogue: Option<SocketCatalogue>,
}

impl RetainedProviderSession {
    /// Enter the normal retained MCP session after the generic installed
    /// launch boundary has already produced a supervised child.
    pub fn from_supervised_child(
        child: crate::child_process::SupervisedChild,
        protocol_version: &str,
        server_name: &str,
        identity: String,
    ) -> Result<Self, StdioProviderError> {
        let mut provider = ManagedProvider::from_supervised_child(child);
        provider.initialize(protocol_version, server_name)?;
        Ok(Self {
            provider,
            next_request_id: 2,
            identity,
            catalogue_stale: true,
            catalogue: None,
        })
    }

    /// Wrap a provider that has already completed initialize and tools/list.
    #[cfg(test)]
    pub(crate) fn from_discovered(provider: ManagedProvider, identity: String) -> Self {
        Self {
            provider,
            next_request_id: 3,
            identity,
            catalogue_stale: false,
            catalogue: None,
        }
    }

    pub fn identity(&self) -> &str {
        &self.identity
    }

    pub fn catalogue_is_stale(&self) -> bool {
        self.catalogue_stale
    }

    pub fn catalogue(&self) -> Option<&SocketCatalogue> {
        self.catalogue.as_ref()
    }

    pub(crate) fn invalidate_catalogue(&mut self) {
        self.catalogue_stale = true;
        self.catalogue = None;
    }

    pub fn stderr_tail(&self) -> String {
        self.provider.stderr_tail()
    }

    pub fn discover_tools(&mut self) -> Result<Vec<Value>, StdioProviderError> {
        <Self as Socket>::discover(self).map(|catalogue| catalogue.operations)
    }

    pub fn tools_call(
        &mut self,
        operation: &str,
        arguments: &Value,
        remaining: Duration,
    ) -> Result<Value, StdioProviderError> {
        <Self as Socket>::invoke(self, operation, arguments, remaining)
    }

    pub fn probe(&mut self) -> Result<Value, StdioProviderError> {
        <Self as Socket>::probe(self)
    }

    pub fn close(&mut self) {
        <Self as Socket>::close(self);
    }

    #[cfg(test)]
    pub(crate) fn next_request_id(&self) -> u64 {
        self.next_request_id
    }

    fn take_request_id(&mut self) -> Result<u64, StdioProviderError> {
        let current = self.next_request_id;
        self.next_request_id = current.checked_add(1).ok_or_else(|| {
            StdioProviderError::ProtocolError("Socket request id exhausted".to_owned())
        })?;
        Ok(current)
    }

    fn refresh_notification_state(&mut self) -> Result<bool, StdioProviderError> {
        self.provider.poll_notifications()?;
        if self.provider.take_catalogue_change_observed() {
            self.invalidate_catalogue();
        }
        Ok(self.catalogue_stale)
    }
}

fn require_fresh_catalogue(stale: bool) -> Result<(), StdioProviderError> {
    if stale {
        Err(StdioProviderError::CatalogueStale)
    } else {
        Ok(())
    }
}

impl Socket for RetainedProviderSession {
    fn establish(spec: SocketEstablishment<'_>) -> Result<Self, StdioProviderError> {
        let mut provider =
            ManagedProvider::launch(spec.command, spec.args, spec.working_directory, None, None)?;
        provider.initialize(spec.protocol_version, spec.server_name)?;
        Ok(Self {
            provider,
            next_request_id: 2,
            identity: spec.identity.to_owned(),
            catalogue_stale: true,
            catalogue: None,
        })
    }

    fn discover(&mut self) -> Result<SocketCatalogue, StdioProviderError> {
        self.refresh_notification_state()?;
        self.invalidate_catalogue();
        let tools = self
            .provider
            .list_tools_paginated(&mut self.next_request_id)?;
        if self.provider.take_catalogue_change_observed() {
            self.invalidate_catalogue();
            return Err(StdioProviderError::CatalogueChangedDuringDiscovery);
        }
        let catalogue = SocketCatalogue { operations: tools };
        self.catalogue_stale = false;
        self.catalogue = Some(catalogue.clone());
        Ok(catalogue)
    }

    fn invoke(
        &mut self,
        operation: &str,
        arguments: &Value,
        remaining: Duration,
    ) -> Result<Value, StdioProviderError> {
        self.refresh_notification_state()?;
        require_fresh_catalogue(self.catalogue_stale)?;
        let request_id = self.take_request_id()?;
        let result = self
            .provider
            .tools_call_with_timeout(request_id, operation, arguments, remaining);
        if self.provider.take_catalogue_change_observed() {
            self.invalidate_catalogue();
        }
        result.map(|result| self.observe_result(result))
    }

    fn observe_result(&mut self, result: Value) -> Value {
        result
    }

    fn observe_catalogue_change(&mut self) -> Result<bool, StdioProviderError> {
        self.refresh_notification_state()
    }

    fn probe(&mut self) -> Result<Value, StdioProviderError> {
        self.refresh_notification_state()?;
        let request_id = self.take_request_id()?;
        let result = self.provider.ping(request_id);
        if self.provider.take_catalogue_change_observed() {
            self.invalidate_catalogue();
        }
        result
    }

    fn close(&mut self) {
        self.provider.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TRUSTED_MANIFEST: &str =
        include_str!("../../protocol/capability-manifests/fixture-ping.json");

    fn fixture_script_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("scripts")
            .join("tethers-stdio-fixture.ps1")
    }

    fn establish_fixture(mode: &str) -> RetainedProviderSession {
        let script = fixture_script_path();
        let working_directory = script.parent().unwrap().to_path_buf();
        let args = vec![
            "-NoProfile".to_owned(),
            "-ExecutionPolicy".to_owned(),
            "Bypass".to_owned(),
            "-File".to_owned(),
            script.to_string_lossy().into_owned(),
            "-Mode".to_owned(),
            mode.to_owned(),
        ];
        RetainedProviderSession::establish(SocketEstablishment {
            command: "pwsh.exe",
            args: &args,
            working_directory: &working_directory,
            protocol_version: "2025-11-25",
            server_name: "tethers-stdio-fixture",
            identity: "tethers-stdio-fixture",
        })
        .unwrap()
    }

    #[test]
    fn catalogue_change_blocks_invocation_before_transport() {
        let error = require_fresh_catalogue(true).unwrap_err();
        assert_eq!(error, StdioProviderError::CatalogueStale);
        assert!(require_fresh_catalogue(false).is_ok());
    }

    #[test]
    fn discovery_consumes_every_page_and_preserves_opaque_observations() {
        let mut session = establish_fixture("paginated-tools");
        let catalogue = session.discover().unwrap();
        assert_eq!(catalogue.operations().len(), 2);
        assert_eq!(
            catalogue.operations()[1]
                .get("name")
                .and_then(Value::as_str),
            Some("fixture_unapproved_addition")
        );
        assert_eq!(
            catalogue.operations()[1].pointer("/annotations/readOnlyHint"),
            Some(&Value::Bool(true))
        );
        assert_eq!(session.next_request_id(), 4);
        session.close();
    }

    #[test]
    fn repeated_opaque_cursor_fails_closed() {
        let mut session = establish_fixture("cursor-loop");
        let error = session.discover().unwrap_err();
        assert!(error.to_string().contains("cursor repeated or looped"));
        assert!(session.catalogue_is_stale());
        session.close();
    }

    #[test]
    fn duplicate_operation_across_pages_fails_closed() {
        let mut session = establish_fixture("paged-duplicate");
        let error = session.discover().unwrap_err();
        assert!(error.to_string().contains("duplicate tool name"));
        assert!(session.catalogue_is_stale());
        session.close();
    }

    #[test]
    fn list_changed_during_discovery_requires_bounded_rediscovery() {
        let mut session = establish_fixture("catalogue-change-unchanged");
        assert_eq!(
            session.discover().unwrap_err(),
            StdioProviderError::CatalogueChangedDuringDiscovery
        );
        assert!(session.catalogue_is_stale());
        let catalogue = session.discover().unwrap();
        assert_eq!(catalogue.operations().len(), 1);
        assert!(!session.catalogue_is_stale());
        session.close();
    }

    #[test]
    fn observed_catalogue_notification_prevents_affected_invocation() {
        let mut session = establish_fixture("catalogue-change-on-probe");
        session.discover().unwrap();
        session.probe().unwrap();
        assert!(session.catalogue_is_stale());
        let next_request_id = session.next_request_id();
        assert_eq!(
            session
                .invoke(
                    "fixture_ping",
                    &serde_json::json!({"message": "must-not-dispatch"}),
                    Duration::from_secs(1),
                )
                .unwrap_err(),
            StdioProviderError::CatalogueStale
        );
        assert_eq!(session.next_request_id(), next_request_id);
        session.close();
    }

    #[test]
    fn changed_schema_remains_unavailable_after_rediscovery() {
        let trusted = crate::manifest::verify_manifest(TRUSTED_MANIFEST).unwrap();
        let mut session = establish_fixture("catalogue-change-drift");
        assert_eq!(
            session.discover().unwrap_err(),
            StdioProviderError::CatalogueChangedDuringDiscovery
        );
        let catalogue = session.discover().unwrap();
        let error =
            crate::stdio_provider::compare_discovery_evidence(catalogue.operations(), &trusted)
                .unwrap_err();
        assert!(error.to_string().contains("input schema did not match"));
        session.invalidate_catalogue();
        assert!(session.catalogue_is_stale());
        session.close();
    }

    #[test]
    fn unapproved_addition_is_observed_but_does_not_change_trusted_binding() {
        let trusted = crate::manifest::verify_manifest(TRUSTED_MANIFEST).unwrap();
        let mut session = establish_fixture("paginated-tools");
        let catalogue = session.discover().unwrap();
        crate::stdio_provider::compare_discovery_evidence(catalogue.operations(), &trusted)
            .unwrap();
        assert!(catalogue
            .operations()
            .iter()
            .any(|operation| operation.get("name").and_then(Value::as_str)
                == Some("fixture_unapproved_addition")));
        assert_eq!(trusted.manifest().binding.tool_name, "fixture_ping");
        session.close();
    }
}
