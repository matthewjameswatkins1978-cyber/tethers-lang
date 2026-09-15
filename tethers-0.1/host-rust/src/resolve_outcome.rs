//! Tethers-owned delivery of a durable provider outcome to Resolve.
//!
//! This is deliberately smaller than a transport or message bus.  The only
//! value crossing the boundary is an opaque action reference and one of the
//! three already-classified provider outcomes.  Provider execution and its
//! durable outcome remain entirely outside this module.

use crate::dispatch::{CoordinationDeliveryEntry, Trail, TrailError};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const MAX_ACTION_REF_BYTES: usize = 512;

/// Opaque host-owned identity of a Tethers action as seen by Resolve.
#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd)]
pub struct TethersActionRef(String);

impl TethersActionRef {
    /// Validate an action reference from the host integration boundary.
    pub fn from_host_value(value: &str) -> Result<Self, ActionRefError> {
        if value.is_empty() {
            return Err(ActionRefError::Empty);
        }
        if value.len() > MAX_ACTION_REF_BYTES {
            return Err(ActionRefError::TooLong);
        }
        if value.chars().any(char::is_control) {
            return Err(ActionRefError::ControlCharacter);
        }
        Ok(Self(value.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Evidence uses a digest so the reference value itself never enters
    /// coordination Trail records.
    pub fn digest(&self) -> String {
        crate::approval::digest(&serde_json::Value::String(self.0.clone()))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionRefError {
    Empty,
    TooLong,
    ControlCharacter,
}

impl fmt::Display for ActionRefError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::Empty => "Tethers action reference is empty",
            Self::TooLong => "Tethers action reference exceeds the bounded length",
            Self::ControlCharacter => "Tethers action reference contains a control character",
        };
        f.write_str(message)
    }
}

impl std::error::Error for ActionRefError {}

/// The only provider truth that Tethers may deliver to Resolve.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ResolveOutcome {
    Succeeded,
    Failed,
    Uncertain,
}

impl ResolveOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Succeeded => "SUCCEEDED",
            Self::Failed => "FAILED",
            Self::Uncertain => "UNCERTAIN",
        }
    }
}

/// Exactly the two values accepted by the Resolve outcome boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolveOutcomeRequest {
    action_ref: TethersActionRef,
    outcome: ResolveOutcome,
}

impl ResolveOutcomeRequest {
    pub fn new(action_ref: TethersActionRef, outcome: ResolveOutcome) -> Self {
        Self {
            action_ref,
            outcome,
        }
    }

    pub fn action_ref(&self) -> &TethersActionRef {
        &self.action_ref
    }

    pub fn outcome(&self) -> ResolveOutcome {
        self.outcome
    }
}

/// Recovery state for delivery, separate from the provider's canonical truth.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ResolveOutcomeDeliveryState {
    Pending,
    Delivered,
    Indeterminate,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredResolveOutcome {
    request: ResolveOutcomeRequest,
    state: ResolveOutcomeDeliveryState,
}

impl StoredResolveOutcome {
    pub fn request(&self) -> &ResolveOutcomeRequest {
        &self.request
    }

    pub fn action_ref(&self) -> &TethersActionRef {
        self.request.action_ref()
    }

    pub fn outcome(&self) -> ResolveOutcome {
        self.request.outcome()
    }

    pub fn state(&self) -> ResolveOutcomeDeliveryState {
        self.state
    }
}

/// The narrow persistence seam for delivery state. It is not a queue: each
/// action has one immutable provider outcome and an append-only delivery
/// status history.
pub trait ResolveOutcomeDeliveryStore {
    fn current(
        &self,
        action_ref: &TethersActionRef,
    ) -> Result<Option<StoredResolveOutcome>, ResolveOutcomeStoreError>;

    fn append(&mut self, entry: &StoredResolveOutcome) -> Result<(), ResolveOutcomeStoreError>;
}

#[derive(Debug)]
pub enum ResolveOutcomeStoreError {
    Io(io::Error),
    InvalidRecord,
}

impl fmt::Display for ResolveOutcomeStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "Resolve outcome store I/O failed: {error}"),
            Self::InvalidRecord => f.write_str("Resolve outcome store contains an invalid record"),
        }
    }
}

impl std::error::Error for ResolveOutcomeStoreError {}

/// A host-owned append-only recovery journal for Resolve delivery.
pub struct FileResolveOutcomeDeliveryStore {
    file: File,
    path: PathBuf,
    latest: BTreeMap<TethersActionRef, StoredResolveOutcome>,
}

impl FileResolveOutcomeDeliveryStore {
    pub fn open(path: impl Into<PathBuf>) -> Result<Self, ResolveOutcomeStoreError> {
        let path = path.into();
        let latest = read_latest(&path)?;
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(ResolveOutcomeStoreError::Io)?;
        Ok(Self { file, path, latest })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl ResolveOutcomeDeliveryStore for FileResolveOutcomeDeliveryStore {
    fn current(
        &self,
        action_ref: &TethersActionRef,
    ) -> Result<Option<StoredResolveOutcome>, ResolveOutcomeStoreError> {
        Ok(self.latest.get(action_ref).cloned())
    }

    fn append(&mut self, entry: &StoredResolveOutcome) -> Result<(), ResolveOutcomeStoreError> {
        let persisted = PersistedResolveOutcome {
            action_ref: entry.action_ref().as_str().to_owned(),
            outcome: entry.outcome(),
            state: entry.state(),
        };
        let line = serde_json::to_string(&persisted)
            .map_err(|_| ResolveOutcomeStoreError::InvalidRecord)?;
        writeln!(self.file, "{line}").map_err(ResolveOutcomeStoreError::Io)?;
        self.file.flush().map_err(ResolveOutcomeStoreError::Io)?;
        self.file
            .sync_data()
            .map_err(ResolveOutcomeStoreError::Io)?;
        self.latest
            .insert(entry.action_ref().clone(), entry.clone());
        Ok(())
    }
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct PersistedResolveOutcome {
    action_ref: String,
    outcome: ResolveOutcome,
    state: ResolveOutcomeDeliveryState,
}

fn read_latest(
    path: &Path,
) -> Result<BTreeMap<TethersActionRef, StoredResolveOutcome>, ResolveOutcomeStoreError> {
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(ResolveOutcomeStoreError::Io(error)),
    };
    let mut latest: BTreeMap<TethersActionRef, StoredResolveOutcome> = BTreeMap::new();
    for line in contents.lines() {
        if line.is_empty() {
            return Err(ResolveOutcomeStoreError::InvalidRecord);
        }
        let persisted: PersistedResolveOutcome =
            serde_json::from_str(line).map_err(|_| ResolveOutcomeStoreError::InvalidRecord)?;
        let action_ref = TethersActionRef::from_host_value(&persisted.action_ref)
            .map_err(|_| ResolveOutcomeStoreError::InvalidRecord)?;
        let entry = StoredResolveOutcome {
            request: ResolveOutcomeRequest::new(action_ref.clone(), persisted.outcome),
            state: persisted.state,
        };
        if let Some(previous) = latest.get(&action_ref) {
            if previous.outcome() != entry.outcome() {
                return Err(ResolveOutcomeStoreError::InvalidRecord);
            }
        }
        latest.insert(action_ref, entry);
    }
    Ok(latest)
}

/// The adapter errors are deliberately opaque. Adapter diagnostics do not
/// become provider truth or enter durable Trail evidence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResolveOutcomeAdapterError;

pub trait ResolveOutcomeAdapter {
    fn deliver_outcome(
        &mut self,
        action_ref: &TethersActionRef,
        outcome: ResolveOutcome,
    ) -> Result<(), ResolveOutcomeAdapterError>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResolveOutcomeDeliveryResult {
    Delivered,
    Indeterminate,
}

#[derive(Debug)]
pub enum ResolveOutcomeDeliveryError {
    MissingActionReference,
    InvalidActionReference(ActionRefError),
    NoRecordedOutcome {
        action_ref: TethersActionRef,
    },
    ContradictoryOutcome {
        action_ref: TethersActionRef,
        recorded: ResolveOutcome,
        supplied: ResolveOutcome,
    },
    Store(ResolveOutcomeStoreError),
    Trail(TrailError),
}

impl fmt::Display for ResolveOutcomeDeliveryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingActionReference => {
                f.write_str("provider outcome has no trusted Tethers action reference")
            }
            Self::InvalidActionReference(error) => error.fmt(f),
            Self::NoRecordedOutcome { .. } => {
                f.write_str("Resolve outcome retry has no recorded provider outcome")
            }
            Self::ContradictoryOutcome { .. } => {
                f.write_str("contradictory Resolve outcome rejected")
            }
            Self::Store(error) => error.fmt(f),
            Self::Trail(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for ResolveOutcomeDeliveryError {}

/// Delivers only a previously classified outcome. Retrying this operation
/// cannot access, and therefore cannot rerun, the provider.
pub struct ResolveOutcomeDeliveryCoordinator<S> {
    store: S,
}

impl<S: ResolveOutcomeDeliveryStore> ResolveOutcomeDeliveryCoordinator<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }

    pub fn store(&self) -> &S {
        &self.store
    }

    pub fn store_mut(&mut self) -> &mut S {
        &mut self.store
    }

    pub fn deliver(
        &mut self,
        request: ResolveOutcomeRequest,
        adapter: &mut dyn ResolveOutcomeAdapter,
        trail: &mut dyn Trail,
    ) -> Result<ResolveOutcomeDeliveryResult, ResolveOutcomeDeliveryError> {
        if let Some(previous) = self
            .store
            .current(request.action_ref())
            .map_err(ResolveOutcomeDeliveryError::Store)?
        {
            if previous.outcome() != request.outcome() {
                return Err(ResolveOutcomeDeliveryError::ContradictoryOutcome {
                    action_ref: request.action_ref().clone(),
                    recorded: previous.outcome(),
                    supplied: request.outcome(),
                });
            }
        }

        let pending = StoredResolveOutcome {
            request: request.clone(),
            state: ResolveOutcomeDeliveryState::Pending,
        };
        self.store
            .append(&pending)
            .map_err(ResolveOutcomeDeliveryError::Store)?;
        append_evidence(trail, &pending)?;

        if adapter
            .deliver_outcome(request.action_ref(), request.outcome())
            .is_err()
        {
            let indeterminate = StoredResolveOutcome {
                request,
                state: ResolveOutcomeDeliveryState::Indeterminate,
            };
            self.store
                .append(&indeterminate)
                .map_err(ResolveOutcomeDeliveryError::Store)?;
            append_evidence(trail, &indeterminate)?;
            return Ok(ResolveOutcomeDeliveryResult::Indeterminate);
        }

        let delivered = StoredResolveOutcome {
            request,
            state: ResolveOutcomeDeliveryState::Delivered,
        };
        self.store
            .append(&delivered)
            .map_err(ResolveOutcomeDeliveryError::Store)?;
        append_evidence(trail, &delivered)?;
        Ok(ResolveOutcomeDeliveryResult::Delivered)
    }

    /// Retry delivery using only the outcome already present in the delivery
    /// journal. There is no provider or classification input on this path.
    pub fn retry(
        &mut self,
        action_ref: &TethersActionRef,
        adapter: &mut dyn ResolveOutcomeAdapter,
        trail: &mut dyn Trail,
    ) -> Result<ResolveOutcomeDeliveryResult, ResolveOutcomeDeliveryError> {
        let stored = self
            .store
            .current(action_ref)
            .map_err(ResolveOutcomeDeliveryError::Store)?
            .ok_or_else(|| ResolveOutcomeDeliveryError::NoRecordedOutcome {
                action_ref: action_ref.clone(),
            })?;
        self.deliver(stored.request().clone(), adapter, trail)
    }
}

fn append_evidence(
    trail: &mut dyn Trail,
    entry: &StoredResolveOutcome,
) -> Result<(), ResolveOutcomeDeliveryError> {
    trail
        .append_coordination_delivery(&CoordinationDeliveryEntry {
            action_ref_digest: entry.action_ref().digest(),
            outcome: entry.outcome().as_str().to_owned(),
            state: entry.state().as_str().to_owned(),
        })
        .map_err(ResolveOutcomeDeliveryError::Trail)
}

impl ResolveOutcomeDeliveryState {
    fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Delivered => "delivered",
            Self::Indeterminate => "indeterminate",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dispatch::RecordingTrail;
    use std::cell::Cell;
    use std::rc::Rc;

    #[derive(Default)]
    struct MemoryStore {
        latest: BTreeMap<TethersActionRef, StoredResolveOutcome>,
    }

    impl ResolveOutcomeDeliveryStore for MemoryStore {
        fn current(
            &self,
            action_ref: &TethersActionRef,
        ) -> Result<Option<StoredResolveOutcome>, ResolveOutcomeStoreError> {
            Ok(self.latest.get(action_ref).cloned())
        }

        fn append(&mut self, entry: &StoredResolveOutcome) -> Result<(), ResolveOutcomeStoreError> {
            self.latest
                .insert(entry.action_ref().clone(), entry.clone());
            Ok(())
        }
    }

    struct TestAdapter {
        calls: Rc<Cell<usize>>,
        result: Result<(), ResolveOutcomeAdapterError>,
    }

    impl ResolveOutcomeAdapter for TestAdapter {
        fn deliver_outcome(
            &mut self,
            _action_ref: &TethersActionRef,
            _outcome: ResolveOutcome,
        ) -> Result<(), ResolveOutcomeAdapterError> {
            self.calls.set(self.calls.get() + 1);
            self.result
        }
    }

    fn request(outcome: ResolveOutcome) -> ResolveOutcomeRequest {
        ResolveOutcomeRequest::new(
            TethersActionRef::from_host_value("exec_p4-test").unwrap(),
            outcome,
        )
    }

    fn trail() -> RecordingTrail {
        RecordingTrail::new()
    }

    #[test]
    fn delivery_failure_preserves_known_outcome_and_recovery_state() {
        for outcome in [
            ResolveOutcome::Succeeded,
            ResolveOutcome::Failed,
            ResolveOutcome::Uncertain,
        ] {
            let calls = Rc::new(Cell::new(0));
            let mut adapter = TestAdapter {
                calls: Rc::clone(&calls),
                result: Err(ResolveOutcomeAdapterError),
            };
            let mut trail = trail();
            let mut coordinator = ResolveOutcomeDeliveryCoordinator::new(MemoryStore::default());
            assert!(matches!(
                coordinator.deliver(request(outcome), &mut adapter, &mut trail),
                Ok(ResolveOutcomeDeliveryResult::Indeterminate)
            ));
            assert_eq!(calls.get(), 1);
            let stored = coordinator
                .store()
                .current(&TethersActionRef::from_host_value("exec_p4-test").unwrap())
                .unwrap()
                .unwrap();
            assert_eq!(stored.outcome(), outcome);
            assert_eq!(stored.state(), ResolveOutcomeDeliveryState::Indeterminate);
            assert_eq!(trail.coordination_delivery_entries.len(), 2);
        }
    }

    #[test]
    fn exact_repeat_is_idempotent_and_never_reexecutes_provider() {
        let calls = Rc::new(Cell::new(0));
        let mut adapter = TestAdapter {
            calls: Rc::clone(&calls),
            result: Ok(()),
        };
        let mut trail = trail();
        let mut coordinator = ResolveOutcomeDeliveryCoordinator::new(MemoryStore::default());
        assert!(matches!(
            coordinator.deliver(request(ResolveOutcome::Failed), &mut adapter, &mut trail),
            Ok(ResolveOutcomeDeliveryResult::Delivered)
        ));
        assert!(matches!(
            coordinator.deliver(request(ResolveOutcome::Failed), &mut adapter, &mut trail),
            Ok(ResolveOutcomeDeliveryResult::Delivered)
        ));
        assert_eq!(calls.get(), 2);
        assert_eq!(
            coordinator
                .store()
                .current(&TethersActionRef::from_host_value("exec_p4-test").unwrap())
                .unwrap()
                .unwrap()
                .state(),
            ResolveOutcomeDeliveryState::Delivered
        );
    }

    #[test]
    fn exact_repeat_is_idempotent_for_each_closed_outcome() {
        for outcome in [
            ResolveOutcome::Succeeded,
            ResolveOutcome::Failed,
            ResolveOutcome::Uncertain,
        ] {
            let calls = Rc::new(Cell::new(0));
            let mut adapter = TestAdapter {
                calls: Rc::clone(&calls),
                result: Ok(()),
            };
            let mut trail = trail();
            let mut coordinator = ResolveOutcomeDeliveryCoordinator::new(MemoryStore::default());
            coordinator
                .deliver(request(outcome), &mut adapter, &mut trail)
                .unwrap();
            coordinator
                .retry(
                    &TethersActionRef::from_host_value("exec_p4-test").unwrap(),
                    &mut adapter,
                    &mut trail,
                )
                .unwrap();
            assert_eq!(calls.get(), 2);
            assert_eq!(
                coordinator
                    .store()
                    .current(&TethersActionRef::from_host_value("exec_p4-test").unwrap())
                    .unwrap()
                    .unwrap()
                    .outcome(),
                outcome
            );
        }
    }

    #[test]
    fn contradictory_outcome_is_rejected_before_adapter() {
        let calls = Rc::new(Cell::new(0));
        let mut adapter = TestAdapter {
            calls: Rc::clone(&calls),
            result: Ok(()),
        };
        let mut trail = trail();
        let mut coordinator = ResolveOutcomeDeliveryCoordinator::new(MemoryStore::default());
        coordinator
            .deliver(request(ResolveOutcome::Succeeded), &mut adapter, &mut trail)
            .unwrap();
        let before = trail.coordination_delivery_entries.len();
        assert!(matches!(
            coordinator.deliver(request(ResolveOutcome::Uncertain), &mut adapter, &mut trail),
            Err(ResolveOutcomeDeliveryError::ContradictoryOutcome { .. })
        ));
        assert_eq!(calls.get(), 1);
        assert_eq!(trail.coordination_delivery_entries.len(), before);
    }

    #[test]
    fn all_three_outcomes_are_closed_and_distinct() {
        assert_ne!(ResolveOutcome::Succeeded, ResolveOutcome::Failed);
        assert_ne!(ResolveOutcome::Failed, ResolveOutcome::Uncertain);
        assert_eq!(ResolveOutcome::Uncertain.as_str(), "UNCERTAIN");
    }

    #[test]
    fn file_store_reopens_delivery_state_without_provider_access() {
        let path = std::env::temp_dir().join(format!(
            "tethers-p4-outcome-{}-{}.jsonl",
            std::process::id(),
            crate::approval::digest(&serde_json::json!("p4"))
        ));
        let _ = fs::remove_file(&path);
        {
            let mut store = FileResolveOutcomeDeliveryStore::open(&path).unwrap();
            let entry = StoredResolveOutcome {
                request: request(ResolveOutcome::Uncertain),
                state: ResolveOutcomeDeliveryState::Indeterminate,
            };
            store.append(&entry).unwrap();
        }
        let store = FileResolveOutcomeDeliveryStore::open(&path).unwrap();
        let current = store
            .current(&TethersActionRef::from_host_value("exec_p4-test").unwrap())
            .unwrap()
            .unwrap();
        assert_eq!(current.outcome(), ResolveOutcome::Uncertain);
        assert_eq!(current.state(), ResolveOutcomeDeliveryState::Indeterminate);
        fs::remove_file(path).unwrap();
    }
}
