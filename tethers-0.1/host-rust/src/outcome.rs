//! Host-owned execution deadline, outcome classification, and redaction.
//!
//! This module deliberately has no provider or Trail dependency.  It is the
//! narrow pure boundary which prevents private adapter diagnostics from being
//! mistaken for durable execution facts.

#[cfg(test)]
use std::cell::Cell;
use std::time::{Duration, Instant};

use serde_json::Value;

pub trait MonotonicClock {
    fn now(&self) -> Duration;
}

pub struct ProductionMonotonicClock {
    origin: Instant,
}

impl ProductionMonotonicClock {
    pub fn new() -> Self {
        Self {
            origin: Instant::now(),
        }
    }
}

impl MonotonicClock for ProductionMonotonicClock {
    fn now(&self) -> Duration {
        self.origin.elapsed()
    }
}

/// A deterministic clock for host tests.  It advances only when the test
/// directs it to, so deadline classification cannot depend on scheduler time.
#[cfg(test)]
pub struct TestMonotonicClock {
    now: Cell<Duration>,
}

#[cfg(test)]
impl TestMonotonicClock {
    pub fn new() -> Self {
        Self {
            now: Cell::new(Duration::ZERO),
        }
    }

    pub fn advance(&self, by: Duration) {
        self.now.set(self.now.get().saturating_add(by));
    }
}

#[cfg(test)]
impl MonotonicClock for TestMonotonicClock {
    fn now(&self) -> Duration {
        self.now.get()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderDiagnostic {
    ExplicitProviderError,
    ProcessLost,
    ResponseMalformed,
    ResponseTruncated,
    ProtocolInterrupted,
    NoFinalResponse,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionOutcome {
    Succeeded(Value),
    Failed { reason: PublicReason },
    Uncertain { reason: PublicReason },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicReason {
    pub code: &'static str,
    pub message: &'static str,
}

/// The sole durable-reason boundary.  Input diagnostics intentionally contain
/// no adapter text; any raw stderr, payload, path, credential, argument, or
/// stack detail stays process-local.
pub fn redact(diagnostic: ProviderDiagnostic) -> PublicReason {
    match diagnostic {
        ProviderDiagnostic::ExplicitProviderError => PublicReason {
            code: "provider_error",
            message: "provider reported an error",
        },
        ProviderDiagnostic::ProcessLost => PublicReason {
            code: "provider_process_lost",
            message: "provider process was lost",
        },
        ProviderDiagnostic::ResponseMalformed | ProviderDiagnostic::ResponseTruncated => {
            PublicReason {
                code: "provider_response_invalid",
                message: "provider response was invalid",
            }
        }
        ProviderDiagnostic::ProtocolInterrupted => PublicReason {
            code: "provider_protocol_interrupted",
            message: "provider protocol was interrupted",
        },
        ProviderDiagnostic::NoFinalResponse => PublicReason {
            code: "provider_outcome_uncertain",
            message: "provider outcome is uncertain",
        },
    }
}

pub fn deadline_reason() -> PublicReason {
    PublicReason {
        code: "deadline_exceeded",
        message: "execution deadline exceeded",
    }
}

pub fn validation_reason() -> PublicReason {
    PublicReason {
        code: "result_validation_failed",
        message: "provider result failed validation",
    }
}

pub fn audit_failure_reason() -> PublicReason {
    PublicReason {
        code: "audit_write_failed",
        message: "outcome audit write failed",
    }
}

pub fn deadline_expired(clock: &dyn MonotonicClock, start: Duration, deadline: Duration) -> bool {
    clock.now().saturating_sub(start) >= deadline
}

/// The host computes this immediately before the provider invocation boundary.
/// `None` means the deadline has already elapsed and the Action is still
/// unattempted, so no adapter call is permitted.
pub fn remaining_until_deadline(
    clock: &dyn MonotonicClock,
    start: Duration,
    deadline: Duration,
) -> Option<Duration> {
    let elapsed = clock.now().saturating_sub(start);
    deadline
        .checked_sub(elapsed)
        .filter(|remaining| !remaining.is_zero())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_clock_advances_only_when_directed() {
        let clock = TestMonotonicClock::new();
        assert_eq!(clock.now(), Duration::ZERO);
        clock.advance(Duration::from_millis(9));
        assert_eq!(clock.now(), Duration::from_millis(9));
    }

    #[test]
    fn redaction_is_stable_bounded_and_contains_no_private_diagnostic() {
        let reason = redact(ProviderDiagnostic::ProtocolInterrupted);
        assert_eq!(reason.code, "provider_protocol_interrupted");
        assert_eq!(reason.message, "provider protocol was interrupted");
        assert!(reason.message.len() < 128);
    }
}
