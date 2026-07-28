// event_admission.rs - host-local event admission gate
//
// A process-local, in-memory gate that accepts each exact event ID once
// per host invocation and enforces a maximum causal generation depth.
//
// This is the pure safety component.  Runtime wiring into the coordinator
// belongs to a later packet.  The gate performs no logging, Trail writing,
// queue draining, evaluation, or dispatch.

use std::collections::HashSet;

/// Maximum causal generation the gate will admit.
///
/// Generations `0..=8` are valid.  Generation 9 and greater are rejected
/// with `CausalDepthExceeded`.
pub const MAX_CAUSAL_GENERATION: u32 = 8;

/// Reason an event was refused admission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventAdmissionRejection {
    /// The event ID has already been admitted during this host invocation.
    DuplicateEventId { event_id: String },
    /// The event's causal generation exceeds the maximum permitted depth.
    CausalDepthExceeded {
        event_id: String,
        generation: u32,
        maximum_generation: u32,
    },
}

/// Process-local, in-memory admission gate.
///
/// Accepts each exact event ID once per invocation.  Rejects duplicates and
/// events whose causal generation exceeds `MAX_CAUSAL_GENERATION`.
///
/// # Ordering
///
/// Depth validation runs before duplicate lookup.  An event that is already
/// beyond the causal limit always reports the structural depth violation,
/// regardless of whether the same ID happened to appear earlier.
#[derive(Debug, Default)]
pub struct EventAdmissionGate {
    admitted_event_ids: HashSet<String>,
}

impl EventAdmissionGate {
    /// Create a fresh admission gate with no admitted event IDs.
    pub fn new() -> Self {
        Self {
            admitted_event_ids: HashSet::new(),
        }
    }

    /// Attempt to admit an event.
    ///
    /// Returns `Ok(())` when the event ID is new and the generation is
    /// within the permitted range.  Returns an `EventAdmissionRejection`
    /// otherwise.
    ///
    /// Depth validation runs first.  The admitted-ID set is only mutated
    /// after every check passes.
    pub fn admit(
        &mut self,
        event_id: &str,
        generation: u32,
    ) -> Result<(), EventAdmissionRejection> {
        // 1. Depth check — always first.
        if generation > MAX_CAUSAL_GENERATION {
            return Err(EventAdmissionRejection::CausalDepthExceeded {
                event_id: event_id.to_owned(),
                generation,
                maximum_generation: MAX_CAUSAL_GENERATION,
            });
        }

        // 2. Duplicate check.
        if self.admitted_event_ids.contains(event_id) {
            return Err(EventAdmissionRejection::DuplicateEventId {
                event_id: event_id.to_owned(),
            });
        }

        // 3. All checks passed — record the admission.
        self.admitted_event_ids.insert(event_id.to_owned());
        Ok(())
    }

    /// Return the number of distinct event IDs admitted so far.
    pub fn admitted_count(&self) -> usize {
        self.admitted_event_ids.len()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // 1. Fresh gate has zero admitted events.
    #[test]
    fn fresh_gate_has_zero_admitted_events() {
        let gate = EventAdmissionGate::new();
        assert_eq!(gate.admitted_count(), 0);
    }

    // 2. Unique generation-0 event is accepted.
    #[test]
    fn unique_generation_zero_event_accepted() {
        let mut gate = EventAdmissionGate::new();
        let result = gate.admit("evt/alpha", 0);
        assert!(result.is_ok(), "gen-0 event must be accepted");
        assert_eq!(gate.admitted_count(), 1);
    }

    // 3. Distinct exact event IDs are accepted.
    #[test]
    fn distinct_event_ids_accepted() {
        let mut gate = EventAdmissionGate::new();
        assert!(gate.admit("evt/alpha", 0).is_ok());
        assert!(gate.admit("evt/beta", 0).is_ok());
        assert!(gate.admit("evt/gamma", 0).is_ok());
        assert_eq!(gate.admitted_count(), 3);
    }

    // 4. Second admission of same ID rejected as duplicate.
    #[test]
    fn second_admission_of_same_id_rejected_as_duplicate() {
        let mut gate = EventAdmissionGate::new();
        assert!(gate.admit("evt/one", 0).is_ok());

        let result = gate.admit("evt/one", 0);
        assert_eq!(
            result,
            Err(EventAdmissionRejection::DuplicateEventId {
                event_id: "evt/one".to_string(),
            })
        );
    }

    // 5. Duplicate matching is case-sensitive.
    #[test]
    fn duplicate_matching_is_case_sensitive() {
        let mut gate = EventAdmissionGate::new();
        assert!(gate.admit("EventA", 0).is_ok());
        // Different case — distinct ID.
        assert!(gate.admit("eventa", 0).is_ok());
        assert_eq!(gate.admitted_count(), 2);

        // Original case still blocked.
        assert_eq!(
            gate.admit("EventA", 0),
            Err(EventAdmissionRejection::DuplicateEventId {
                event_id: "EventA".to_string(),
            })
        );
    }

    // 6. Generation 8 is accepted.
    #[test]
    fn generation_eight_accepted() {
        let mut gate = EventAdmissionGate::new();
        assert!(gate.admit("evt/deep", 8).is_ok());
        assert_eq!(gate.admitted_count(), 1);
    }

    // 7. Generation 9 is rejected.
    #[test]
    fn generation_nine_rejected() {
        let mut gate = EventAdmissionGate::new();
        let result = gate.admit("evt/too-deep", 9);
        assert_eq!(
            result,
            Err(EventAdmissionRejection::CausalDepthExceeded {
                event_id: "evt/too-deep".to_string(),
                generation: 9,
                maximum_generation: 8,
            })
        );
    }

    // 8. Generation above 9 is rejected.
    #[test]
    fn generation_above_nine_rejected() {
        let mut gate = EventAdmissionGate::new();
        let result = gate.admit("evt/very-deep", 42);
        assert_eq!(
            result,
            Err(EventAdmissionRejection::CausalDepthExceeded {
                event_id: "evt/very-deep".to_string(),
                generation: 42,
                maximum_generation: 8,
            })
        );
    }

    // 9. Depth rejection does not reserve the event ID.
    #[test]
    fn depth_rejection_does_not_reserve_event_id() {
        let mut gate = EventAdmissionGate::new();

        // First, try at generation 9 — must be rejected.
        let result = gate.admit("evt/later-ok", 9);
        assert!(result.is_err());

        // The ID must not have been recorded.
        assert_eq!(gate.admitted_count(), 0);

        // Now admit at a valid generation — must succeed.
        assert!(gate.admit("evt/later-ok", 0).is_ok());
        assert_eq!(gate.admitted_count(), 1);
    }

    // 10. Duplicate rejection does not change admitted count.
    #[test]
    fn duplicate_rejection_does_not_change_admitted_count() {
        let mut gate = EventAdmissionGate::new();
        assert!(gate.admit("evt/count", 0).is_ok());
        assert_eq!(gate.admitted_count(), 1);

        let _ = gate.admit("evt/count", 0); // rejected
        assert_eq!(
            gate.admitted_count(),
            1,
            "count must not change after duplicate rejection"
        );
    }

    // 11. Depth rejection takes precedence over duplicate rejection.
    #[test]
    fn depth_rejection_precedes_duplicate_rejection() {
        let mut gate = EventAdmissionGate::new();

        // First, admit at generation 0.
        assert!(gate.admit("evt/precedence", 0).is_ok());

        // Now try the same ID but at generation 9.
        // Depth violation must be reported, not duplicate.
        let result = gate.admit("evt/precedence", 9);
        assert_eq!(
            result,
            Err(EventAdmissionRejection::CausalDepthExceeded {
                event_id: "evt/precedence".to_string(),
                generation: 9,
                maximum_generation: 8,
            }),
            "depth rejection must take precedence over duplicate"
        );
    }

    // 12. Accepted ID remains recorded with no removal surface.
    #[test]
    fn accepted_id_remains_recorded_with_no_removal_surface() {
        let mut gate = EventAdmissionGate::new();
        assert!(gate.admit("evt/forever", 5).is_ok());
        assert_eq!(gate.admitted_count(), 1);

        // The gate provides no removal method.  The ID stays.
        // Subsequent duplicate attempts prove it is still there.
        for _ in 0..5 {
            assert_eq!(
                gate.admit("evt/forever", 5),
                Err(EventAdmissionRejection::DuplicateEventId {
                    event_id: "evt/forever".to_string(),
                })
            );
        }
        assert_eq!(gate.admitted_count(), 1);
    }

    // 13. Consecutive distinct events at maximum depth are accepted.
    #[test]
    fn consecutive_distinct_events_at_max_depth_accepted() {
        let mut gate = EventAdmissionGate::new();
        assert!(gate.admit("evt/max-a", 8).is_ok());
        assert!(gate.admit("evt/max-b", 8).is_ok());
        assert!(gate.admit("evt/max-c", 8).is_ok());
        assert_eq!(gate.admitted_count(), 3);
    }

    // 14. Multiple rejections do not alter admitted set.
    #[test]
    fn multiple_rejections_do_not_alter_admitted_set() {
        let mut gate = EventAdmissionGate::new();
        assert!(gate.admit("evt/stable", 0).is_ok());

        // Various rejections.
        let _ = gate.admit("evt/stable", 0); // duplicate
        let _ = gate.admit("evt/other", 99); // depth
        let _ = gate.admit("evt/stable", 99); // depth (precedence)

        assert_eq!(gate.admitted_count(), 1);
        // The stable ID is still admitted.
        assert_eq!(
            gate.admit("evt/stable", 0),
            Err(EventAdmissionRejection::DuplicateEventId {
                event_id: "evt/stable".to_string(),
            })
        );
    }

    // 15. Max depth gate rejects generation 9 despite no prior admission.
    #[test]
    fn max_depth_rejects_generation_nine_fresh() {
        let gate = &mut EventAdmissionGate::new();
        // Test the boundary independently of any admission history.
        for gen in 0..=8 {
            let id = format!("evt/gen-{}", gen);
            assert!(gate.admit(&id, gen).is_ok(), "gen {} must be accepted", gen);
        }
        assert!(
            gate.admit("evt/gen-9", 9).is_err(),
            "gen 9 must be rejected"
        );
    }
}
