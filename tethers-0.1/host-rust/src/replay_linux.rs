//! Native GNU/Linux replay persistence for the J09 record model.
//!
//! The Windows backend uses handle-bound Win32 operations. Linux uses the
//! corresponding filesystem properties: every existing path component is
//! rejected when it is a symlink, records are canonical and immutable, and
//! exclusive per-key locks are held for the lifetime of an admission.

use crate::replay::{
    validate_chain, Claim, ExecutionBinding, ExecutionId, Generation, LogicalExecutionKey,
    ReplayError, ReplayState,
};
use std::collections::{HashMap, HashSet};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::os::fd::AsRawFd;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::rc::Rc;

const FORMAT_BYTES: &[u8] = br#"{"replay_format_version":1}"#;
const MAX_REPLAY_RECORD_BYTES: u64 = 64 * 1024;

fn unavailable<T>() -> Result<T, ReplayError> {
    Err(ReplayError::PersistenceUnavailable)
}

fn valid_leaf(value: &str) -> bool {
    !value.is_empty()
        && value != "."
        && value != ".."
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
}

fn is_lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

fn verify_chain(path: &Path) -> Result<(), ReplayError> {
    if !path.is_absolute() {
        return unavailable();
    }
    for ancestor in path.ancestors() {
        match fs::symlink_metadata(ancestor) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                #[cfg(target_os = "macos")]
                if crate::path_safety::is_macos_system_path_alias(ancestor) {
                    continue;
                }
                return unavailable();
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return unavailable(),
        }
    }
    Ok(())
}

fn validate_directory(path: &Path) -> Result<PathBuf, ReplayError> {
    if !path.is_absolute() {
        return unavailable();
    }
    let metadata = fs::symlink_metadata(path).map_err(|_| ReplayError::PersistenceUnavailable)?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return unavailable();
    }
    let canonical = fs::canonicalize(path).map_err(|_| ReplayError::PersistenceUnavailable)?;

    #[cfg(target_os = "macos")]
    {
        // On macOS, Darwin links /var -> private/var and /tmp -> private/tmp at system root.
        // We verify that the canonical path has no symlink components, and that the only difference
        // between path and canonical is the standard macOS /private prefix.
        verify_chain(&canonical)?;
        let without_private: PathBuf = match canonical.strip_prefix("/private") {
            Ok(rest) => Path::new("/").join(rest),
            Err(_) => canonical.clone(),
        };
        if without_private != path && canonical != path {
            return unavailable();
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        verify_chain(path)?;
        if canonical != path {
            return unavailable();
        }
    }

    Ok(canonical)
}

fn names(path: &Path) -> Result<Vec<String>, ReplayError> {
    let mut result = Vec::new();
    for entry in fs::read_dir(path).map_err(|_| ReplayError::PersistenceUnavailable)? {
        let entry = entry.map_err(|_| ReplayError::PersistenceUnavailable)?;
        result.push(entry.file_name().to_string_lossy().into_owned());
    }
    result.sort();
    Ok(result)
}

fn exact_entries(path: &Path, expected: &[&str]) -> Result<(), ReplayError> {
    let mut expected = expected
        .iter()
        .map(|value| (*value).to_owned())
        .collect::<Vec<_>>();
    expected.sort();
    if names(path)? == expected {
        Ok(())
    } else {
        unavailable()
    }
}

fn regular_file(path: &Path) -> Result<bool, ReplayError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                unavailable()
            } else {
                Ok(true)
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(_) => unavailable(),
    }
}

fn read_record(path: &Path) -> Result<Vec<u8>, ReplayError> {
    if !regular_file(path)? {
        return unavailable();
    }
    let metadata = fs::metadata(path).map_err(|_| ReplayError::PersistenceUnavailable)?;
    if metadata.len() == 0 || metadata.len() > MAX_REPLAY_RECORD_BYTES {
        return unavailable();
    }
    let mut file = File::open(path).map_err(|_| ReplayError::PersistenceUnavailable)?;
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.read_to_end(&mut bytes)
        .map_err(|_| ReplayError::PersistenceUnavailable)?;
    if bytes.len() as u64 != metadata.len() {
        return unavailable();
    }
    Ok(bytes)
}

fn sync_directory(path: &Path) -> Result<(), ReplayError> {
    let file = File::open(path).map_err(|_| ReplayError::PersistenceUnavailable)?;
    #[cfg(target_os = "macos")]
    {
        // On macOS/Darwin, standard fsync() on a directory fd returns EINVAL.
        // F_FULLFSYNC provides physical barrier synchronization for metadata.
        let fd = file.as_raw_fd();
        let ret = unsafe { libc::fcntl(fd, libc::F_FULLFSYNC) };
        if ret == -1 {
            // A successful open proves only that the directory exists. It does
            // not prove that the directory entry update reached durable storage.
            return Err(ReplayError::PersistenceUnavailable);
        }
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    {
        file.sync_all()
            .map_err(|_| ReplayError::PersistenceUnavailable)
    }
}

/// Publish without replacing an existing destination. A hard link from a
/// synced temporary file provides create-new visibility on the same filesystem.
fn publish_new(path: &Path, stem: &str, bytes: &[u8]) -> Result<(), ReplayError> {
    if !valid_leaf(stem) || !path.is_dir() {
        return unavailable();
    }
    let temporary = path.join(format!(".{stem}.tmp"));
    let destination = path.join(stem);
    if regular_file(&temporary)? || destination.exists() {
        return unavailable();
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|_| ReplayError::PersistenceUnavailable)?;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|_| ReplayError::PersistenceUnavailable)?;
    if fs::hard_link(&temporary, &destination).is_err() {
        return unavailable();
    }
    sync_directory(path)?;
    fs::remove_file(&temporary).map_err(|_| ReplayError::PersistenceUnavailable)?;
    Ok(())
}

fn validate_hierarchy(root: &Path) -> Result<(PathBuf, PathBuf, PathBuf), ReplayError> {
    let root = validate_directory(root)?;
    if !root.join("replay").is_dir() {
        return unavailable();
    }
    let replay = validate_directory(&root.join("replay"))?;
    exact_entries(&replay, &["v1"])?;
    let version = validate_directory(&replay.join("v1"))?;
    exact_entries(&version, &["FORMAT.json", "chains", "claims", "locks"])?;
    if read_record(&version.join("FORMAT.json"))? != FORMAT_BYTES {
        return unavailable();
    }
    let chains = validate_directory(&version.join("chains"))?;
    let claims = validate_directory(&version.join("claims"))?;
    let locks = validate_directory(&version.join("locks"))?;
    Ok((locks, claims, chains))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProvisionReplayOutcome {
    Provisioned,
    AlreadyProvisioned,
}

impl ProvisionReplayOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Provisioned => "Provisioned",
            Self::AlreadyProvisioned => "AlreadyProvisioned",
        }
    }
}

pub fn provision_replay(root_path: &Path) -> Result<ProvisionReplayOutcome, ReplayError> {
    let root = validate_directory(root_path)?;
    if root.join("replay").exists() {
        validate_hierarchy(&root)?;
        ReplayLedger::open(&root)?;
        return Ok(ProvisionReplayOutcome::AlreadyProvisioned);
    }
    fs::create_dir(root.join("replay")).map_err(|_| ReplayError::PersistenceUnavailable)?;
    let version = root.join("replay/v1");
    fs::create_dir(&version).map_err(|_| ReplayError::PersistenceUnavailable)?;
    for name in ["locks", "claims", "chains"] {
        fs::create_dir(version.join(name)).map_err(|_| ReplayError::PersistenceUnavailable)?;
    }
    publish_new(&version, "FORMAT.json", FORMAT_BYTES)?;
    sync_directory(&root.join("replay"))?;
    sync_directory(&root)?;
    validate_hierarchy(&root)?;
    Ok(ProvisionReplayOutcome::Provisioned)
}

pub struct LogicalKeyLock {
    _file: File,
}

impl LogicalKeyLock {
    fn acquire(directory: &Path, logical_key: &LogicalExecutionKey) -> Result<Self, ReplayError> {
        let path = directory.join(format!("{}.lock", logical_key.filename_digest()));
        if let Ok(metadata) = fs::symlink_metadata(&path) {
            if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
                return unavailable();
            }
        }
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .custom_flags(libc::O_NOFOLLOW)
            .open(&path)
            .map_err(|_| ReplayError::PersistenceUnavailable)?;
        if !file
            .metadata()
            .map_err(|_| ReplayError::PersistenceUnavailable)?
            .is_file()
        {
            return unavailable();
        }
        // SAFETY: the descriptor is owned by this guard and remains live until
        // the lock is released by Drop.
        if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX) } != 0 {
            return unavailable();
        }
        Ok(Self { _file: file })
    }
}

pub struct ReplayLedger {
    locks: PathBuf,
    claims: PathBuf,
    chains: PathBuf,
}

impl ReplayLedger {
    pub fn open(root_path: &Path) -> Result<Self, ReplayError> {
        let (locks, claims, chains) = validate_hierarchy(root_path)?;
        for name in names(&locks)? {
            if !name
                .strip_suffix(".lock")
                .is_some_and(|value| is_lower_hex(value, 64))
                || !regular_file(&locks.join(&name))?
            {
                return unavailable();
            }
        }
        let mut claims_by_execution = HashMap::new();
        for name in names(&claims)? {
            let Some(digest) = name.strip_suffix(".claim.json") else {
                return unavailable();
            };
            if !is_lower_hex(digest, 64) {
                return unavailable();
            }
            let logical_key = LogicalExecutionKey::from_digest(format!("sha256:{digest}"))
                .map_err(|_| ReplayError::PersistenceUnavailable)?;
            let claim = read_claim(&claims, &logical_key)?;
            if claims_by_execution
                .insert(claim.execution_id.filename_digest(), claim)
                .is_some()
            {
                return unavailable();
            }
        }
        let mut seen = HashSet::new();
        for prefix_name in names(&chains)? {
            if !is_lower_hex(&prefix_name, 2) {
                return unavailable();
            }
            let prefix = validate_directory(&chains.join(&prefix_name))?;
            for execution_name in names(&prefix)? {
                if !is_lower_hex(&execution_name, 64)
                    || !execution_name.starts_with(&prefix_name)
                    || !seen.insert(execution_name.clone())
                {
                    return unavailable();
                }
                let Some(claim) = claims_by_execution.get(&execution_name) else {
                    return unavailable();
                };
                let execution = validate_directory(&prefix.join(&execution_name))?;
                let mut generation_names = names(&execution)?;
                generation_names.sort();
                let mut generations = Vec::new();
                for (expected, name) in generation_names.into_iter().enumerate() {
                    let number = parse_generation_filename(&name)?;
                    if number != expected as u64 {
                        return unavailable();
                    }
                    generations.push(
                        Generation::from_canonical_bytes(&read_record(&execution.join(name))?)
                            .map_err(|_| ReplayError::PersistenceUnavailable)?,
                    );
                }
                validate_chain(claim, &generations)
                    .map_err(|_| ReplayError::PersistenceUnavailable)?;
            }
        }
        Ok(Self {
            locks,
            claims,
            chains,
        })
    }

    /// Recover logical key and binding for one durable execution identity.
    /// Used by Authority Gate restart reconciliation; read-only.
    pub fn claim_material_for(
        &self,
        execution_id: &str,
    ) -> Option<(LogicalExecutionKey, ExecutionBinding)> {
        for name in names(&self.claims).ok()? {
            let digest = name.strip_suffix(".claim.json")?;
            if !is_lower_hex(digest, 64) {
                return None;
            }
            let logical_key = LogicalExecutionKey::from_digest(format!("sha256:{digest}")).ok()?;
            let claim = read_claim(&self.claims, &logical_key).ok()?;
            if claim.execution_id.as_str() == execution_id {
                return Some((claim.logical_key, claim.binding));
            }
        }
        None
    }

    /// Enumerate every durable claim with its reconstructed chain state.
    /// Bounded, read-only; used by Authority Gate durable reconciliation.
    pub fn inspect_durable(&self) -> Result<Vec<crate::replay::DurableReplayClaim>, ReplayError> {
        let mut inspected = Vec::new();
        for name in names(&self.claims)? {
            let Some(digest) = name.strip_suffix(".claim.json") else {
                return unavailable();
            };
            if !is_lower_hex(digest, 64) {
                return unavailable();
            }
            let logical_key = LogicalExecutionKey::from_digest(format!("sha256:{digest}"))
                .map_err(|_| ReplayError::PersistenceUnavailable)?;
            let claim = read_claim(&self.claims, &logical_key)?;
            let (state, generations) = reconstruct(self, &claim)?;
            inspected.push(crate::replay::DurableReplayClaim {
                execution_id: claim.execution_id.as_str().to_owned(),
                logical_key: claim.logical_key.clone(),
                binding: claim.binding.clone(),
                state,
                durable_outcome_digest: generations
                    .last()
                    .and_then(|generation| generation.durable_outcome_digest.clone()),
            });
        }
        inspected.sort_by(|left, right| left.execution_id.cmp(&right.execution_id));
        Ok(inspected)
    }

    pub fn admit_or_recover_owned(
        ledger: &Rc<ReplayLedger>,
        logical_key: LogicalExecutionKey,
        binding: ExecutionBinding,
    ) -> Result<ReplayAdmission, ReplayError> {
        let lock = LogicalKeyLock::acquire(&ledger.locks, &logical_key)?;
        if let Some(claim) = existing_claim(&ledger.claims, &logical_key)? {
            claim.require_binding(&binding)?;
            let (state, generations) = reconstruct(ledger, &claim)?;
            return Ok(ReplayAdmission {
                ledger: Rc::clone(ledger),
                _lock: lock,
                claim,
                generations,
                state,
                fresh: false,
            });
        }
        let claim = Claim::new(logical_key, ExecutionId::generate(), binding)
            .map_err(|_| ReplayError::PersistenceUnavailable)?;
        let name = format!("{}.claim.json", claim.logical_key.filename_digest());
        let bytes = claim
            .canonical_bytes()
            .map_err(|_| ReplayError::PersistenceUnavailable)?;
        publish_new(&ledger.claims, &name, &bytes)?;
        let published = existing_claim(&ledger.claims, &claim.logical_key)?
            .ok_or(ReplayError::PersistenceUnavailable)?;
        if published != claim {
            return unavailable();
        }
        Ok(ReplayAdmission {
            ledger: Rc::clone(ledger),
            _lock: lock,
            claim,
            generations: Vec::new(),
            state: ReplayState::ClaimedNoState,
            fresh: true,
        })
    }
}

fn read_claim(claims: &Path, key: &LogicalExecutionKey) -> Result<Claim, ReplayError> {
    Claim::from_canonical_bytes(
        &read_record(&claims.join(format!("{}.claim.json", key.filename_digest())))?,
        key,
    )
    .map_err(|_| ReplayError::PersistenceUnavailable)
}

fn existing_claim(claims: &Path, key: &LogicalExecutionKey) -> Result<Option<Claim>, ReplayError> {
    let path = claims.join(format!("{}.claim.json", key.filename_digest()));
    if !path.exists() {
        return Ok(None);
    }
    read_claim(claims, key).map(Some)
}

fn parse_generation_filename(value: &str) -> Result<u64, ReplayError> {
    if value.len() != 22 || !value.starts_with('g') || !value.ends_with(".json") {
        return unavailable();
    }
    let number = value[1..17]
        .parse::<u64>()
        .map_err(|_| ReplayError::PersistenceUnavailable)?;
    if number > 2 || format!("g{number:016}.json") != value {
        return unavailable();
    }
    Ok(number)
}

fn execution_directory(ledger: &ReplayLedger, claim: &Claim) -> Result<PathBuf, ReplayError> {
    let digest = claim.execution_id.filename_digest();
    let prefix = ledger.chains.join(&digest[..2]);
    if !prefix.exists() {
        fs::create_dir(&prefix).map_err(|_| ReplayError::PersistenceUnavailable)?;
    }
    let prefix = validate_directory(&prefix)?;
    let execution = prefix.join(digest);
    if !execution.exists() {
        fs::create_dir(&execution).map_err(|_| ReplayError::PersistenceUnavailable)?;
    }
    validate_directory(&execution)
}

fn reconstruct(
    ledger: &ReplayLedger,
    claim: &Claim,
) -> Result<(ReplayState, Vec<Generation>), ReplayError> {
    let digest = claim.execution_id.filename_digest();
    let execution = ledger.chains.join(&digest[..2]).join(digest);
    if !execution.exists() {
        return Ok((ReplayState::ClaimedNoState, Vec::new()));
    }
    let execution = validate_directory(&execution)?;
    let mut records = Vec::new();
    for name in names(&execution)? {
        records.push((parse_generation_filename(&name)?, name));
    }
    records.sort_by_key(|(number, _)| *number);
    let mut generations = Vec::new();
    for (expected, (number, name)) in records.into_iter().enumerate() {
        if expected as u64 != number {
            return unavailable();
        }
        generations.push(
            Generation::from_canonical_bytes(&read_record(&execution.join(name))?)
                .map_err(|_| ReplayError::PersistenceUnavailable)?,
        );
    }
    let state =
        validate_chain(claim, &generations).map_err(|_| ReplayError::PersistenceUnavailable)?;
    Ok((state, generations))
}

pub struct ReplayAdmission {
    ledger: Rc<ReplayLedger>,
    _lock: LogicalKeyLock,
    claim: Claim,
    generations: Vec<Generation>,
    state: ReplayState,
    fresh: bool,
}

impl ReplayAdmission {
    pub fn execution_id(&self) -> &str {
        self.claim.execution_id.as_str()
    }
    pub fn state(&self) -> ReplayState {
        self.state
    }
    pub fn is_fresh(&self) -> bool {
        self.fresh
    }
    pub fn publish_intent(&mut self) -> Result<(), ReplayError> {
        if !self.fresh || self.state != ReplayState::ClaimedNoState || !self.generations.is_empty()
        {
            return unavailable();
        }
        let generation =
            Generation::intent(&self.claim).map_err(|_| ReplayError::PersistenceUnavailable)?;
        self.publish_generation(generation)
    }
    pub fn publish_armed(&mut self) -> Result<(), ReplayError> {
        if !self.fresh || self.state != ReplayState::IntentRecorded || self.generations.len() != 1 {
            return unavailable();
        }
        let generation = Generation::armed(&self.claim, &self.generations[0])
            .map_err(|_| ReplayError::PersistenceUnavailable)?;
        self.publish_generation(generation)
    }
    pub fn publish_terminal(
        &mut self,
        state: ReplayState,
        durable_outcome_digest: String,
    ) -> Result<(), ReplayError> {
        // Fresh armed admissions complete normally. Recovered (non-fresh)
        // armed admissions complete durable outcome after Gate restart.
        if self.state != ReplayState::InvocationArmed || self.generations.len() != 2 {
            return unavailable();
        }
        let generation = Generation::terminal(
            &self.claim,
            &self.generations[1],
            state,
            durable_outcome_digest,
        )
        .map_err(|_| ReplayError::PersistenceUnavailable)?;
        self.publish_generation(generation)
    }
    fn publish_generation(&mut self, generation: Generation) -> Result<(), ReplayError> {
        let (_, durable) = reconstruct(&self.ledger, &self.claim)?;
        if durable != self.generations {
            return unavailable();
        }
        let directory = execution_directory(&self.ledger, &self.claim)?;
        let name = format!("g{:016}.json", generation.number);
        let bytes = generation
            .canonical_bytes()
            .map_err(|_| ReplayError::PersistenceUnavailable)?;
        publish_new(&directory, &name, &bytes)?;
        let reopened = Generation::from_canonical_bytes(&read_record(&directory.join(&name))?)
            .map_err(|_| ReplayError::PersistenceUnavailable)?;
        if reopened != generation {
            return unavailable();
        }
        self.state = generation.state;
        self.generations.push(generation);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_replay_requires_directory_full_sync_support() {
        sync_directory(&std::env::temp_dir())
            .expect("macOS replay persistence requires directory full-sync support");
    }

    fn binding() -> ExecutionBinding {
        ExecutionBinding {
            evaluation_id: "eval-linux".into(),
            action_id: "action-linux".into(),
            capability_name: "test.capability".into(),
            capability_version: 1,
            manifest_digest:
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
            provider_identity: "provider-linux".into(),
            argument_digest:
                "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".into(),
        }
    }

    #[test]
    fn linux_replay_lifecycle_reopens_with_identical_terminal_state() {
        let root =
            std::env::temp_dir().join(format!("tethers-replay-linux-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&root).unwrap();
        provision_replay(&root).unwrap();
        let ledger = Rc::new(ReplayLedger::open(&root).unwrap());
        let key =
            LogicalExecutionKey::derive("anchor-linux", "eval-linux", "action-linux").unwrap();
        let mut admission =
            ReplayLedger::admit_or_recover_owned(&ledger, key.clone(), binding()).unwrap();
        assert!(admission.is_fresh());
        admission.publish_intent().unwrap();
        admission.publish_armed().unwrap();
        admission
            .publish_terminal(
                ReplayState::Succeeded,
                "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc".into(),
            )
            .unwrap();
        assert_eq!(admission.state(), ReplayState::Succeeded);
        drop(admission);

        let reopened = Rc::new(ReplayLedger::open(&root).unwrap());
        let recovered = ReplayLedger::admit_or_recover_owned(&reopened, key, binding()).unwrap();
        assert!(!recovered.is_fresh());
        assert_eq!(recovered.state(), ReplayState::Succeeded);
        drop(recovered);
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn linux_replay_rejects_symlinked_root() {
        use std::os::unix::fs::symlink;

        let parent =
            std::env::temp_dir().join(format!("tethers-replay-symlink-{}", uuid::Uuid::new_v4()));
        let real = parent.join("real");
        let linked = parent.join("linked");
        fs::create_dir_all(&real).unwrap();
        symlink(&real, &linked).unwrap();
        assert_eq!(
            provision_replay(&linked).unwrap_err(),
            ReplayError::PersistenceUnavailable
        );
        fs::remove_dir_all(parent).unwrap();
    }
}
