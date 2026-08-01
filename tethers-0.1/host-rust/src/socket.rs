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

/// Accepted Socket v1 operations over a serial retained session.
pub trait Socket: Sized {
    fn establish(spec: SocketEstablishment<'_>) -> Result<Self, StdioProviderError>;
    fn discover(&mut self) -> Result<Vec<Value>, StdioProviderError>;
    fn invoke(
        &mut self,
        operation: &str,
        arguments: &Value,
        remaining: Duration,
    ) -> Result<Value, StdioProviderError>;
    fn observe_result(&mut self, result: Value) -> Value;
    fn observe_catalogue_change(&mut self);
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
}

impl RetainedProviderSession {
    /// Wrap a provider that has already completed initialize and tools/list.
    pub(crate) fn from_discovered(provider: ManagedProvider, identity: String) -> Self {
        Self {
            provider,
            next_request_id: 3,
            identity,
            catalogue_stale: false,
        }
    }

    pub fn identity(&self) -> &str {
        &self.identity
    }

    pub fn catalogue_is_stale(&self) -> bool {
        self.catalogue_stale
    }

    pub(crate) fn invalidate_catalogue(&mut self) {
        self.catalogue_stale = true;
    }

    pub fn stderr_tail(&self) -> String {
        self.provider.stderr_tail()
    }

    pub fn discover_tools(&mut self) -> Result<Vec<Value>, StdioProviderError> {
        <Self as Socket>::discover(self)
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
}

fn require_fresh_catalogue(stale: bool) -> Result<(), StdioProviderError> {
    if stale {
        Err(StdioProviderError::ProtocolError(
            "Socket catalogue is stale; exact rediscovery is required before invocation".to_owned(),
        ))
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
        })
    }

    fn discover(&mut self) -> Result<Vec<Value>, StdioProviderError> {
        let request_id = self.take_request_id()?;
        let tools = self.provider.list_tools_with_id(request_id)?;
        self.catalogue_stale = false;
        Ok(tools)
    }

    fn invoke(
        &mut self,
        operation: &str,
        arguments: &Value,
        remaining: Duration,
    ) -> Result<Value, StdioProviderError> {
        require_fresh_catalogue(self.catalogue_stale)?;
        let request_id = self.take_request_id()?;
        let result = self
            .provider
            .tools_call_with_timeout(request_id, operation, arguments, remaining)?;
        Ok(self.observe_result(result))
    }

    fn observe_result(&mut self, result: Value) -> Value {
        result
    }

    fn observe_catalogue_change(&mut self) {
        self.catalogue_stale = true;
    }

    fn probe(&mut self) -> Result<Value, StdioProviderError> {
        let request_id = self.take_request_id()?;
        self.provider.ping(request_id)
    }

    fn close(&mut self) {
        self.provider.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalogue_change_blocks_invocation_before_transport() {
        let error = require_fresh_catalogue(true).unwrap_err();
        assert!(error.to_string().contains("catalogue is stale"));
        assert!(require_fresh_catalogue(false).is_ok());
    }
}
