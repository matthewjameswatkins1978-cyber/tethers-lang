// executor.rs - Capability executor trait for dispatch-ready actions.
//
// Defines the single trait that adapters and retained provider sessions
// implement.  This is the narrow boundary between the dispatch proof and
// the actual provider invocation.

use crate::dispatch::DispatchReadyAction;
use crate::outcome;
use serde_json::Value;
use std::time::Duration;

/// A typed executor that can invoke a capability Action described by a
/// `DispatchReadyAction`.
///
/// Implementors receive the exact capability name, version, provider
/// identity, manifest digest, arguments, and stable execution/action
/// identifiers from the readiness token.  They must not use independently
/// supplied identity fields.
pub trait CapabilityExecutor {
    /// Honest provider identity.  Callers must verify this matches the
    /// resolved capability's `provider_identity()` before invoking
    /// `execute()`.
    fn provider_identity(&self) -> &str;

    /// Execute the capability Action described by `ready`.
    ///
    /// The executor receives the exact capability name, version,
    /// provider identity, manifest digest, arguments, and stable
    /// execution/action identifiers from the readiness token.  It must
    /// not use independently supplied identity fields.
    fn execute(&mut self, ready: &DispatchReadyAction) -> Result<Value, String>;

    /// Execute with the host-computed remaining monotonic deadline.  Adapters
    /// must bound their wait by `remaining` and report a typed ambiguity when
    /// no trustworthy final response is available in time.
    ///
    /// The compatibility implementation never treats an untyped string error
    /// as provider-declared failure: it is post-invocation uncertainty.
    /// Adapters with a trusted explicit provider error must override this
    /// method and return `ExplicitProviderError` themselves.
    fn execute_classified(
        &mut self,
        ready: &DispatchReadyAction,
        _remaining: Duration,
    ) -> Result<Value, outcome::ProviderDiagnostic> {
        self.execute(ready)
            .map_err(|_| outcome::ProviderDiagnostic::NoFinalResponse)
    }
}
