use crate::installed::InstalledPlugRecord;
use crate::m3_store::{canonical, reject_reparse, sha256, strict_json, M3Error, Result, StoreRoot};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Component, Path, PathBuf};
use uuid::Uuid;

const INTENT_ROOT: &str = "installation-intent";

fn invalid() -> M3Error {
    M3Error::new(
        "installation_intent_invalid",
        "installation publication intent is invalid",
    )
}

fn conflict() -> M3Error {
    M3Error::new(
        "installation_intent_conflict",
        "installation publication intent conflicts with current state",
    )
}

fn io_error() -> M3Error {
    M3Error::new(
        "installation_intent_io",
        "installation publication intent could not be persisted",
    )
}

fn to_intent_error(error: M3Error) -> M3Error {
    match error.code {
        "unsafe_store_path" => error,
        "store_io" | "install_io" | "install_review_io" | "record_conflict" => io_error(),
        _ => invalid(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct InstallationPublicationIntent {
    pub schema_version: u32,
    pub transaction_id: String,
    pub candidate_id: String,
    pub destination_relative_path: String,
    pub installed_record: InstalledPlugRecord,
    pub installed_record_digest: String,
    pub intent_digest: String,
}

impl InstallationPublicationIntent {
    fn covered_bytes(&self) -> Result<Vec<u8>> {
        let mut covered = self.clone();
        covered.intent_digest.clear();
        canonical(&covered)
    }

    fn expected_destination(transaction_id: &str) -> String {
        format!("plug-{transaction_id}")
    }

    fn valid_destination(path: &str, transaction_id: &str) -> bool {
        if path != Self::expected_destination(transaction_id)
            || path.is_empty()
            || Path::new(path).is_absolute()
        {
            return false;
        }
        let mut components = Path::new(path).components();
        matches!(components.next(), Some(Component::Normal(value))
            if value.to_string_lossy() == path)
            && components.next().is_none()
    }

    pub(crate) fn from_precomputed_record(record: InstalledPlugRecord) -> Result<Self> {
        record.validate().map_err(|_| invalid())?;
        let mut intent = Self {
            schema_version: 1,
            transaction_id: record.installed_id.clone(),
            candidate_id: record.source_candidate_id.clone(),
            destination_relative_path: record.installation_relative_path.clone(),
            installed_record: record.clone(),
            installed_record_digest: record.record_digest.clone(),
            intent_digest: String::new(),
        };
        intent.intent_digest = sha256(&intent.covered_bytes().map_err(|_| invalid())?);
        intent.validate()?;
        Ok(intent)
    }

    pub(crate) fn validate(&self) -> Result<()> {
        let transaction = Uuid::parse_str(&self.transaction_id).map_err(|_| invalid())?;
        if self.schema_version != 1
            || transaction.to_string() != self.transaction_id
            || self.transaction_id != self.installed_record.installed_id
            || self.candidate_id != self.installed_record.source_candidate_id
            || self.destination_relative_path != self.installed_record.installation_relative_path
            || !Self::valid_destination(&self.destination_relative_path, &self.transaction_id)
            || self.installed_record_digest != self.installed_record.record_digest
        {
            return Err(invalid());
        }
        self.installed_record.validate().map_err(|_| invalid())?;
        let digest = sha256(&self.covered_bytes().map_err(|_| invalid())?);
        if self.intent_digest != digest {
            return Err(invalid());
        }
        Ok(())
    }
}

pub(crate) struct InstallationPublicationIntentStore {
    root: StoreRoot,
}

impl InstallationPublicationIntentStore {
    pub(crate) fn open(executor_state_root: &Path) -> Result<Self> {
        let state_root = StoreRoot::open(executor_state_root).map_err(to_intent_error)?;
        Ok(Self {
            root: StoreRoot::open(&state_root.path().join(INTENT_ROOT)).map_err(to_intent_error)?,
        })
    }

    pub(crate) fn open_existing(executor_state_root: &Path) -> Result<Self> {
        let state_root = StoreRoot::open_existing(executor_state_root).map_err(to_intent_error)?;
        Ok(Self {
            root: StoreRoot::open_existing(&state_root.path().join(INTENT_ROOT))
                .map_err(to_intent_error)?,
        })
    }

    fn current_path(&self) -> PathBuf {
        self.root.path().join("current.json")
    }

    fn validate_entry(path: &Path) -> Result<()> {
        reject_reparse(path).map_err(to_intent_error)?;
        let metadata = fs::symlink_metadata(path).map_err(|_| io_error())?;
        if !metadata.is_file() {
            return Err(invalid());
        }
        Ok(())
    }

    pub(crate) fn create(&self, intent: &InstallationPublicationIntent) -> Result<()> {
        intent.validate()?;
        let entries = self.root.entries().map_err(to_intent_error)?;
        if !entries.is_empty() {
            match self.load() {
                Ok(Some(_)) => return Err(conflict()),
                Ok(None) => return Err(invalid()),
                Err(error) => return Err(error),
            }
        }
        self.root
            .create_json("current", intent)
            .map_err(to_intent_error)?;
        Ok(())
    }

    pub(crate) fn load(&self) -> Result<Option<InstallationPublicationIntent>> {
        let entries = self.root.entries().map_err(to_intent_error)?;
        if entries.is_empty() {
            return Ok(None);
        }
        if entries.len() != 1
            || entries[0].file_name().and_then(|name| name.to_str()) != Some("current.json")
        {
            return Err(invalid());
        }
        let path = &entries[0];
        Self::validate_entry(path)?;
        let bytes = fs::read(path).map_err(|_| io_error())?;
        let intent: InstallationPublicationIntent = strict_json(&bytes).map_err(|_| invalid())?;
        intent.validate()?;
        Ok(Some(intent))
    }

    pub(crate) fn root_path(&self) -> &Path {
        self.root.path()
    }

    pub(crate) fn remove_if_matches(
        &self,
        expected: &InstallationPublicationIntent,
    ) -> Result<bool> {
        expected.validate()?;
        let Some(current) = self.load()? else {
            return Ok(false);
        };
        if current != *expected {
            return Err(conflict());
        }
        let path = self.current_path();
        Self::validate_entry(&path)?;
        fs::remove_file(path).map_err(|_| io_error())?;
        Ok(true)
    }
}
