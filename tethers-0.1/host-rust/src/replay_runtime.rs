//! Narrow host-runtime boundary for J09 replay admission.
//!
//! Storage remains in `replay_windows`; orchestration sees only a held
//! admission guard and the four redacted replay results.

use crate::replay::{ExecutionBinding, LogicalExecutionKey, ReplayError, ReplayState};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayDispatchResult {
    PersistenceUnavailable,
    BlockedCompletedSuccess,
    BlockedCompletedFailure,
    RequiresManualResolution,
}

impl ReplayDispatchResult {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PersistenceUnavailable => "replay_persistence_unavailable",
            Self::BlockedCompletedSuccess => "replay_blocked_completed_success",
            Self::BlockedCompletedFailure => "replay_blocked_completed_failure",
            Self::RequiresManualResolution => "replay_requires_manual_resolution",
        }
    }

    pub fn from_recovered_state(state: ReplayState) -> Self {
        match state {
            ReplayState::Succeeded => Self::BlockedCompletedSuccess,
            ReplayState::Failed => Self::BlockedCompletedFailure,
            ReplayState::ClaimedNoState
            | ReplayState::IntentRecorded
            | ReplayState::InvocationArmed
            | ReplayState::Uncertain => Self::RequiresManualResolution,
        }
    }
}

pub trait ReplayAdmissionGuard {
    fn execution_id(&self) -> &str;
    fn state(&self) -> ReplayState;
    fn is_fresh(&self) -> bool;
    fn publish_intent(&mut self) -> Result<(), ReplayError>;
    fn publish_armed(&mut self) -> Result<(), ReplayError>;
    fn publish_terminal(
        &mut self,
        state: ReplayState,
        durable_outcome_digest: String,
    ) -> Result<(), ReplayError>;
}

pub trait ReplayAuthority {
    fn admit(
        &self,
        logical_key: &LogicalExecutionKey,
        binding: &ExecutionBinding,
    ) -> Result<Box<dyn ReplayAdmissionGuard>, ReplayError>;
}

use std::cell::RefCell;
use std::rc::Rc;

/// Lazy normal-execution authority. Merely constructing this value neither
/// opens nor provisions replay storage.
pub struct FileReplayAuthority {
    root: Option<PathBuf>,
    #[cfg(windows)]
    ledger: RefCell<Option<Rc<crate::replay_windows::ReplayLedger>>>,
}

impl FileReplayAuthority {
    pub fn new(root: Option<&Path>) -> Self {
        Self {
            root: root.map(Path::to_path_buf),
            #[cfg(windows)]
            ledger: RefCell::new(None),
        }
    }
}

#[cfg(windows)]
impl ReplayAdmissionGuard for crate::replay_windows::ReplayAdmission {
    fn execution_id(&self) -> &str {
        crate::replay_windows::ReplayAdmission::execution_id(self)
    }

    fn state(&self) -> ReplayState {
        self.state()
    }

    fn is_fresh(&self) -> bool {
        self.is_fresh()
    }

    fn publish_intent(&mut self) -> Result<(), ReplayError> {
        self.publish_intent()
    }

    fn publish_armed(&mut self) -> Result<(), ReplayError> {
        self.publish_armed()
    }

    fn publish_terminal(
        &mut self,
        state: ReplayState,
        durable_outcome_digest: String,
    ) -> Result<(), ReplayError> {
        self.publish_terminal(state, durable_outcome_digest)
    }
}

impl ReplayAuthority for FileReplayAuthority {
    fn admit(
        &self,
        logical_key: &LogicalExecutionKey,
        binding: &ExecutionBinding,
    ) -> Result<Box<dyn ReplayAdmissionGuard>, ReplayError> {
        let root = self
            .root
            .as_deref()
            .ok_or(ReplayError::PersistenceUnavailable)?;
        #[cfg(windows)]
        {
            let mut ledger_ref = self.ledger.borrow_mut();
            if ledger_ref.is_none() {
                *ledger_ref = Some(Rc::new(crate::replay_windows::ReplayLedger::open(root)?));
            }
            let ledger = Rc::clone(
                ledger_ref
                    .as_ref()
                    .ok_or(ReplayError::PersistenceUnavailable)?,
            );
            let admission = crate::replay_windows::ReplayLedger::admit_or_recover_owned(
                &ledger,
                logical_key.clone(),
                binding.clone(),
            )?;
            Ok(Box::new(admission))
        }
        #[cfg(not(windows))]
        {
            let _ = (root, logical_key, binding);
            Err(ReplayError::PersistenceUnavailable)
        }
    }
}

#[cfg(test)]
pub mod test_support {
    use super::*;
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    pub const TEST_EXECUTION_ID: &str = "exec_00000000-0000-4000-8000-000000000001";

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum FailPoint {
        Admit,
        Intent,
        Armed,
        Terminal,
    }

    pub struct TestReplayAuthority {
        pub fresh: bool,
        pub recovered_state: ReplayState,
        pub fail_at: Option<FailPoint>,
        pub events: Rc<RefCell<Vec<&'static str>>>,
        pub guard_held: Rc<Cell<bool>>,
        pub admissions: Rc<RefCell<usize>>,
        pub logical_keys: Rc<RefCell<Vec<LogicalExecutionKey>>>,
        pub bindings: Rc<RefCell<Vec<ExecutionBinding>>>,
    }

    impl Default for TestReplayAuthority {
        fn default() -> Self {
            Self {
                fresh: true,
                recovered_state: ReplayState::ClaimedNoState,
                fail_at: None,
                events: Rc::new(RefCell::new(Vec::new())),
                guard_held: Rc::new(Cell::new(false)),
                admissions: Rc::new(RefCell::new(0)),
                logical_keys: Rc::new(RefCell::new(Vec::new())),
                bindings: Rc::new(RefCell::new(Vec::new())),
            }
        }
    }

    struct TestAdmission {
        fresh: bool,
        state: ReplayState,
        fail_at: Option<FailPoint>,
        events: Rc<RefCell<Vec<&'static str>>>,
        guard_held: Rc<Cell<bool>>,
    }

    impl Drop for TestAdmission {
        fn drop(&mut self) {
            self.events.borrow_mut().push("release_admission");
            self.guard_held.set(false);
        }
    }

    impl ReplayAdmissionGuard for TestAdmission {
        fn execution_id(&self) -> &str {
            TEST_EXECUTION_ID
        }

        fn state(&self) -> ReplayState {
            self.state
        }

        fn is_fresh(&self) -> bool {
            self.fresh
        }

        fn publish_intent(&mut self) -> Result<(), ReplayError> {
            self.events.borrow_mut().push("publish_g0");
            if self.fail_at == Some(FailPoint::Intent) {
                return Err(ReplayError::PersistenceUnavailable);
            }
            self.state = ReplayState::IntentRecorded;
            Ok(())
        }

        fn publish_armed(&mut self) -> Result<(), ReplayError> {
            self.events.borrow_mut().push("publish_g1");
            if self.fail_at == Some(FailPoint::Armed) {
                return Err(ReplayError::PersistenceUnavailable);
            }
            self.state = ReplayState::InvocationArmed;
            Ok(())
        }

        fn publish_terminal(
            &mut self,
            state: ReplayState,
            _durable_outcome_digest: String,
        ) -> Result<(), ReplayError> {
            self.events.borrow_mut().push("publish_g2");
            if self.fail_at == Some(FailPoint::Terminal) {
                return Err(ReplayError::PersistenceUnavailable);
            }
            self.state = state;
            Ok(())
        }
    }

    impl ReplayAuthority for TestReplayAuthority {
        fn admit(
            &self,
            logical_key: &LogicalExecutionKey,
            binding: &ExecutionBinding,
        ) -> Result<Box<dyn ReplayAdmissionGuard>, ReplayError> {
            self.events.borrow_mut().push("admit");
            *self.admissions.borrow_mut() += 1;
            self.logical_keys.borrow_mut().push(logical_key.clone());
            self.bindings.borrow_mut().push(binding.clone());
            if self.fail_at == Some(FailPoint::Admit) {
                return Err(ReplayError::PersistenceUnavailable);
            }
            self.guard_held.set(true);
            Ok(Box::new(TestAdmission {
                fresh: self.fresh,
                state: self.recovered_state,
                fail_at: self.fail_at,
                events: Rc::clone(&self.events),
                guard_held: Rc::clone(&self.guard_held),
            }))
        }
    }
}
