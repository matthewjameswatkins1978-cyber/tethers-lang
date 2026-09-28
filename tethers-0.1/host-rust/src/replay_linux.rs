//! Native POSIX (Linux and macOS) replay persistence for the J09 record model.
//!
//! The Windows backend uses handle-bound Win32 operations. This backend uses
//! the corresponding POSIX guarantees:
//!
//! - every structural directory below the host-data root is opened
//!   `O_DIRECTORY | O_NOFOLLOW` and validated by `fstat` on the retained
//!   descriptor (owner is the effective UID, no group/other write access), so
//!   the object validated is the object later used;
//! - child entries are opened relative to retained descriptors with
//!   `openat`/`mkdirat`/`unlinkat`/`linkat` and `O_NOFOLLOW`, so a pathname
//!   substitution between validation and use cannot redirect an operation;
//! - records are canonical, immutable, and published with create-new
//!   semantics (`linkat` fails `EEXIST`); publication visibility is atomic
//!   and never replaces an existing record;
//! - publication staging uses a uniquely nonced temporary name, so a stranded
//!   temporary from an interrupted publication can never block a later
//!   publication attempt, and strict scans recognise (without trusting)
//!   exactly that internal residue shape;
//! - macOS data-file durability uses `F_FULLFSYNC`, matching the durability
//!   model the directory metadata sync already promises;
//! - exclusive per-key `flock` locks are held for the lifetime of an
//!   admission;
//! - every failure records a structured machine-readable diagnostic with the
//!   same phase/reason/recovery contract as the Windows backend.
//!
//! Ancestral directories above the host-data root are only checked for
//! symlink traversal and canonicalisation (with the narrowly verified macOS
//! `/private` alias rules). A private, correctly owned root under a shared
//! parent such as `/tmp` is safe storage; requiring every ancestor to be
//! user-owned would reject legitimate system layouts.

use crate::replay::{
    canonical_generation_number, is_lower_hex, is_recognized_staging_residue, resolve_audit_line,
    staging_temp_name, validate_chain, Claim, ExecutionBinding, ExecutionId, Generation,
    LogicalExecutionKey, ReplayError, ReplayResolveReport, ReplayState, StagingContext,
    REPLAY_QUARANTINE_DIR_NAME, REPLAY_RESOLVE_LOG_NAME,
};
use std::collections::{HashMap, HashSet};
use std::ffi::CString;
use std::fs::File;
use std::io::{Read, Write};
use std::os::fd::{AsRawFd, FromRawFd};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::rc::Rc;

const FORMAT_BYTES: &[u8] = br#"{"replay_format_version":1}"#;
const MAX_REPLAY_RECORD_BYTES: u64 = 64 * 1024;

// ---------------------------------------------------------------------------
// Structured diagnostics (DEF-04 parity with the Windows backend)
// ---------------------------------------------------------------------------

fn fail<T>(phase: &str, reason: &str, recovery: &str) -> Result<T, ReplayError> {
    crate::replay::record_replay_diagnostic(phase, reason, recovery, None);
    Err(ReplayError::PersistenceUnavailable)
}

fn fail_with<T>(
    phase: &str,
    reason: &str,
    recovery: &str,
    detail: String,
) -> Result<T, ReplayError> {
    crate::replay::record_replay_diagnostic(phase, reason, recovery, Some(detail));
    Err(ReplayError::PersistenceUnavailable)
}

fn io_error(phase: &str, reason: &str, recovery: &str, error: &std::io::Error) -> ReplayError {
    let detail = error
        .raw_os_error()
        .map(|code| format!("os_error:{code}"))
        .unwrap_or_else(|| format!("io_error:{error}"));
    crate::replay::record_replay_diagnostic(phase, reason, recovery, Some(detail));
    ReplayError::PersistenceUnavailable
}

fn io_fail<T>(
    phase: &str,
    reason: &str,
    recovery: &str,
    error: &std::io::Error,
) -> Result<T, ReplayError> {
    Err(io_error(phase, reason, recovery, error))
}

fn record_error(phase: &str, reason: &str) -> ReplayError {
    crate::replay::record_replay_diagnostic(phase, reason, RECOVERY_INSPECT, None);
    ReplayError::PersistenceUnavailable
}

const RECOVERY_INSPECT: &str =
    "Inspect the replay subtree; remove or repair only through an authorised operator route.";
const RECOVERY_REPROVISION: &str = "Run provision-replay against a valid absolute host-data root.";
const RECOVERY_PERMISSIONS: &str =
    "Ensure the host-data root and replay subtree are owned by the current user with no group or other write access.";

// ---------------------------------------------------------------------------
// Deterministic fault injection (test-only; mirrors the Windows seam set)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // DuringPartialWrite is armed only from cfg(test) code.
enum UnixPersistenceFaultPoint {
    BeforeTemporaryCreate,
    AfterTemporaryCreate,
    DuringPartialWrite,
    AfterFileSync,
    BeforePublication,
    AfterPublication,
    BeforeDirectorySync,
    BeforeTemporaryCleanup,
    AfterCleanup,
    DuringRestartScan,
}

#[cfg(test)]
thread_local! {
    static INJECTED_PERSISTENCE_FAULT: std::cell::Cell<Option<UnixPersistenceFaultPoint>> =
        const { std::cell::Cell::new(None) };
}

fn persistence_fault(point: UnixPersistenceFaultPoint) -> Result<(), ReplayError> {
    #[cfg(test)]
    if INJECTED_PERSISTENCE_FAULT.with(|fault| fault.get() == Some(point)) {
        return fail(
            "fault_injection",
            "injected_persistence_fault",
            "Test-only deterministic interruption; no operator action.",
        );
    }
    #[cfg(not(test))]
    let _ = point;
    Ok(())
}

fn fault_partial_write_armed() -> bool {
    #[cfg(test)]
    {
        INJECTED_PERSISTENCE_FAULT
            .with(|fault| fault.get() == Some(UnixPersistenceFaultPoint::DuringPartialWrite))
    }
    #[cfg(not(test))]
    {
        false
    }
}

#[cfg(test)]
fn with_persistence_fault<T>(point: UnixPersistenceFaultPoint, operation: impl FnOnce() -> T) -> T {
    INJECTED_PERSISTENCE_FAULT.with(|fault| {
        let previous = fault.replace(Some(point));
        let result = operation();
        fault.set(previous);
        result
    })
}

// ---------------------------------------------------------------------------
// Path-level root validation (ancestors, canonicalisation, macOS aliases)
// ---------------------------------------------------------------------------

fn verify_chain(path: &Path) -> Result<(), ReplayError> {
    if !path.is_absolute() {
        return fail(
            "path_validation",
            "path_not_absolute",
            "Supply an absolute host-data root path.",
        );
    }
    for ancestor in path.ancestors() {
        match std::fs::symlink_metadata(ancestor) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                #[cfg(target_os = "macos")]
                if crate::path_safety::is_macos_system_path_alias(ancestor) {
                    continue;
                }
                return fail_with(
                    "path_traversal",
                    "symlink_component_rejected",
                    RECOVERY_INSPECT,
                    ancestor.display().to_string(),
                );
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return io_fail(
                    "path_traversal",
                    "ancestor_stat_failed",
                    RECOVERY_INSPECT,
                    &error,
                )
            }
        }
    }
    Ok(())
}

fn validate_directory(path: &Path) -> Result<PathBuf, ReplayError> {
    if !path.is_absolute() {
        return fail(
            "path_validation",
            "path_not_absolute",
            "Supply an absolute host-data root path.",
        );
    }
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) => {
            return io_fail(
                "path_traversal",
                "directory_stat_failed",
                RECOVERY_INSPECT,
                &error,
            )
        }
    };
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return fail("path_traversal", "not_a_plain_directory", RECOVERY_INSPECT);
    }
    let canonical = match std::fs::canonicalize(path) {
        Ok(canonical) => canonical,
        Err(error) => {
            return io_fail(
                "path_traversal",
                "canonicalization_failed",
                RECOVERY_INSPECT,
                &error,
            )
        }
    };

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
            return fail(
                "path_traversal",
                "canonical_path_mismatch",
                RECOVERY_INSPECT,
            );
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        verify_chain(path)?;
        if canonical != path {
            return fail(
                "path_traversal",
                "canonical_path_mismatch",
                RECOVERY_INSPECT,
            );
        }
    }

    Ok(canonical)
}

// ---------------------------------------------------------------------------
// Descriptor-bound trusted directories and entries (RSK-01)
// ---------------------------------------------------------------------------

fn current_euid() -> libc::uid_t {
    // SAFETY: geteuid takes no arguments and cannot fail.
    unsafe { libc::geteuid() }
}

fn metadata_is_trusted(metadata: &std::fs::Metadata, want_dir: bool) -> bool {
    let kind_ok = if want_dir {
        metadata.is_dir()
    } else {
        metadata.is_file()
    };
    kind_ok && metadata.uid() == current_euid() && (metadata.mode() & 0o022) == 0
}

fn stat_is_trusted(stat: &libc::stat, want_dir: bool) -> bool {
    let type_mask = stat.st_mode & libc::S_IFMT;
    let kind_ok = if want_dir {
        type_mask == libc::S_IFDIR
    } else {
        type_mask == libc::S_IFREG
    };
    kind_ok && stat.st_uid == current_euid() && (stat.st_mode & 0o022) == 0
}

/// A directory descriptor that was opened `O_DIRECTORY | O_NOFOLLOW` and whose
/// `fstat` proved ownership by the effective UID with no group/other write
/// access. All child operations are performed relative to this descriptor.
struct TrustedDir {
    file: File,
    path: PathBuf,
}

impl TrustedDir {
    fn fd(&self) -> std::os::fd::RawFd {
        self.file.as_raw_fd()
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

fn trust_violation<T>(path: &Path) -> Result<T, ReplayError> {
    fail_with(
        "security_validation",
        "unsafe_ownership_or_permissions",
        RECOVERY_PERMISSIONS,
        path.display().to_string(),
    )
}

fn check_dir_trust(file: &File, path: &Path) -> Result<(), ReplayError> {
    let metadata = match file.metadata() {
        Ok(metadata) => metadata,
        Err(error) => {
            return io_fail(
                "security_validation",
                "directory_fstat_failed",
                RECOVERY_INSPECT,
                &error,
            )
        }
    };
    if !metadata_is_trusted(&metadata, true) {
        return trust_violation(path);
    }
    Ok(())
}

fn check_file_trust(file: &File, path: &Path) -> Result<(), ReplayError> {
    let metadata = match file.metadata() {
        Ok(metadata) => metadata,
        Err(error) => {
            return io_fail(
                "security_validation",
                "record_fstat_failed",
                RECOVERY_INSPECT,
                &error,
            )
        }
    };
    if !metadata_is_trusted(&metadata, false) {
        return trust_violation(path);
    }
    Ok(())
}

fn c_name(name: &str) -> Result<CString, ReplayError> {
    match CString::new(name.as_bytes()) {
        Ok(value) => Ok(value),
        Err(_) => fail_with(
            "path_validation",
            "invalid_entry_name",
            RECOVERY_INSPECT,
            "name contains an interior NUL".to_owned(),
        ),
    }
}

/// Open the host-data root itself: path-level ancestor validation first, then
/// an `O_NOFOLLOW` descriptor with `fstat` trust validation.
fn open_trusted_root(path: &Path) -> Result<TrustedDir, ReplayError> {
    let canonical = validate_directory(path)?;
    let cname = match CString::new(canonical.as_os_str().as_bytes()) {
        Ok(value) => value,
        Err(_) => {
            return fail(
                "path_validation",
                "invalid_path_bytes",
                "The host-data root path contains an interior NUL byte.",
            )
        }
    };
    // SAFETY: the returned descriptor from a successful open is owned by this
    // function and transferred into a File exactly once.
    let fd = unsafe {
        libc::open(
            cname.as_ptr(),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
        )
    };
    if fd < 0 {
        return io_fail(
            "path_traversal",
            "root_open_failed",
            RECOVERY_INSPECT,
            &std::io::Error::last_os_error(),
        );
    }
    let file = unsafe { File::from_raw_fd(fd) };
    check_dir_trust(&file, &canonical)?;
    Ok(TrustedDir {
        file,
        path: canonical,
    })
}

/// Open a child directory relative to a retained trusted descriptor.
fn open_dir_at(parent: &TrustedDir, name: &str) -> Result<TrustedDir, ReplayError> {
    let cname = c_name(name)?;
    // SAFETY: parent.fd() is live and owned by the retained TrustedDir; cname
    // is NUL-terminated; a successful openat returns an owned descriptor that
    // is transferred into a File exactly once.
    let fd = unsafe {
        libc::openat(
            parent.fd(),
            cname.as_ptr(),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
        )
    };
    if fd < 0 {
        return io_fail(
            "hierarchy_validation",
            "directory_open_failed",
            RECOVERY_INSPECT,
            &std::io::Error::last_os_error(),
        );
    }
    let file = unsafe { File::from_raw_fd(fd) };
    let path = parent.path().join(name);
    check_dir_trust(&file, &path)?;
    Ok(TrustedDir { file, path })
}

enum StatAtOutcome {
    Missing,
    Entry(libc::stat),
    Error,
}

/// Classify a child entry by `fstatat` relative to a retained descriptor
/// without opening it. `AT_SYMLINK_NOFOLLOW` ensures a substituted symlink is
/// reported as an entry whose type check fails, never followed.
fn stat_at(parent: &TrustedDir, name: &str) -> StatAtOutcome {
    let Ok(cname) = c_name(name) else {
        return StatAtOutcome::Error;
    };
    let mut stat: libc::stat = unsafe { std::mem::zeroed() };
    // SAFETY: parent.fd() is live; cname is NUL-terminated; stat is a valid
    // zeroed structure receiving the result.
    let rc = unsafe {
        libc::fstatat(
            parent.fd(),
            cname.as_ptr(),
            &mut stat,
            libc::AT_SYMLINK_NOFOLLOW,
        )
    };
    if rc == 0 {
        return StatAtOutcome::Entry(stat);
    }
    if std::io::Error::last_os_error().kind() == std::io::ErrorKind::NotFound {
        StatAtOutcome::Missing
    } else {
        StatAtOutcome::Error
    }
}

fn missing_at(parent: &TrustedDir, name: &str) -> bool {
    matches!(stat_at(parent, name), StatAtOutcome::Missing)
}

fn open_file_at(
    parent: &TrustedDir,
    name: &str,
    flags: libc::c_int,
    mode: libc::mode_t,
) -> Result<File, std::io::Error> {
    let cname = CString::new(name.as_bytes())
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "interior NUL"))?;
    // SAFETY: parent.fd() is live; cname is NUL-terminated; O_NOFOLLOW and
    // O_CLOEXEC are always added; mode applies only with O_CREAT and is
    // widened to c_uint because Darwin's mode_t is u16 and variadic
    // arguments must not be narrower than c_int; a successful openat
    // returns an owned descriptor transferred into a File exactly once.
    let fd = unsafe {
        libc::openat(
            parent.fd(),
            cname.as_ptr(),
            flags | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            mode as libc::c_uint,
        )
    };
    if fd < 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(unsafe { File::from_raw_fd(fd) })
}

/// Read one authoritative record through a descriptor-bound, trust-checked
/// open. Symlinks are rejected by `O_NOFOLLOW`.
fn read_record_at(dir: &TrustedDir, name: &str) -> Result<Vec<u8>, ReplayError> {
    let path = dir.path().join(name);
    let mut file = match open_file_at(dir, name, libc::O_RDONLY, 0) {
        Ok(file) => file,
        Err(error) => {
            return io_fail(
                "record_validation",
                "record_open_failed",
                RECOVERY_INSPECT,
                &error,
            )
        }
    };
    check_file_trust(&file, &path)?;
    let metadata = match file.metadata() {
        Ok(metadata) => metadata,
        Err(error) => {
            return io_fail(
                "record_validation",
                "record_fstat_failed",
                RECOVERY_INSPECT,
                &error,
            )
        }
    };
    if metadata.len() == 0 || metadata.len() > MAX_REPLAY_RECORD_BYTES {
        return fail_with(
            "record_validation",
            "record_size_out_of_bounds",
            RECOVERY_INSPECT,
            path.display().to_string(),
        );
    }
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.read_to_end(&mut bytes).map_err(|error| {
        io_error(
            "record_validation",
            "record_read_failed",
            RECOVERY_INSPECT,
            &error,
        )
    })?;
    if bytes.len() as u64 != metadata.len() {
        return fail_with(
            "record_validation",
            "record_length_changed_during_read",
            RECOVERY_INSPECT,
            path.display().to_string(),
        );
    }
    Ok(bytes)
}

fn reset_errno() {
    // SAFETY: writing zero to this thread's errno storage is always valid.
    #[cfg(target_os = "linux")]
    unsafe {
        *libc::__errno_location() = 0;
    }
    #[cfg(target_os = "macos")]
    unsafe {
        *libc::__error() = 0;
    }
}

/// Enumerate directory entry names through a duplicated descriptor bound to
/// the exact directory that was validated. Names are sorted. Non-UTF-8 names
/// become lossy strings that can never match an authoritative or recognised
/// staging pattern, so they fail closed in every scan.
fn names_at(dir: &TrustedDir) -> Result<Vec<String>, ReplayError> {
    // SAFETY: dup creates a new descriptor we own; fdopendir transfers
    // ownership of that duplicate (never the retained ledger descriptor) to
    // the DIR handle; closedir releases it exactly once on every exit path.
    let dup = unsafe { libc::dup(dir.fd()) };
    if dup < 0 {
        return io_fail(
            "hierarchy_validation",
            "directory_read_failed",
            RECOVERY_INSPECT,
            &std::io::Error::last_os_error(),
        );
    }
    let handle = unsafe { libc::fdopendir(dup) };
    if handle.is_null() {
        let error = std::io::Error::last_os_error();
        // SAFETY: fdopendir failed, so the duplicate descriptor is still ours.
        unsafe { libc::close(dup) };
        return io_fail(
            "hierarchy_validation",
            "directory_read_failed",
            RECOVERY_INSPECT,
            &error,
        );
    }
    // The duplicate SHARES the directory offset with the retained descriptor,
    // and earlier enumerations may have consumed the stream to EOF. Rewind so
    // every scan reads the directory from the beginning. (The retained
    // descriptor is only used for openat/fstatat, which ignore the offset.)
    // SAFETY: the DIR handle is exclusively owned by this function.
    unsafe { libc::rewinddir(handle) };
    let mut names = Vec::new();
    loop {
        reset_errno();
        // SAFETY: the DIR handle is exclusively owned by this function and
        // used single-threaded; readdir returns NULL at end-of-directory or
        // on error, distinguished by the zeroed errno above.
        let entry = unsafe { libc::readdir(handle) };
        if entry.is_null() {
            let errno = std::io::Error::last_os_error();
            // SAFETY: closedir on the exclusively owned handle; this is the
            // only close on every exit path (exactly-once ownership).
            unsafe { libc::closedir(handle) };
            if errno.raw_os_error() != Some(0) {
                return io_fail(
                    "hierarchy_validation",
                    "directory_read_failed",
                    RECOVERY_INSPECT,
                    &errno,
                );
            }
            names.sort();
            return Ok(names);
        }
        // SAFETY: d_name is a NUL-terminated C string inside a valid entry.
        let name = unsafe { std::ffi::CStr::from_ptr((*entry).d_name.as_ptr()) };
        let text = name.to_string_lossy().into_owned();
        if text == "." || text == ".." {
            continue;
        }
        names.push(text);
    }
}

/// Verify that a recognised staging-residue name is a trusted regular file.
/// Residue content is never read or parsed: it is interrupted internal
/// publication state, not an authoritative record. A symlink or non-regular
/// object wearing a residue-shaped name fails closed.
fn verify_residue_trust(dir: &TrustedDir, name: &str) -> Result<(), ReplayError> {
    match stat_at(dir, name) {
        StatAtOutcome::Entry(stat) if stat_is_trusted(&stat, false) => Ok(()),
        StatAtOutcome::Entry(_) => trust_violation(&dir.path().join(name)),
        StatAtOutcome::Missing => fail_with(
            "hierarchy_validation",
            "staging_residue_disappeared_during_scan",
            RECOVERY_INSPECT,
            name.to_owned(),
        ),
        StatAtOutcome::Error => fail_with(
            "hierarchy_validation",
            "staging_residue_stat_failed",
            RECOVERY_INSPECT,
            name.to_owned(),
        ),
    }
}

// ---------------------------------------------------------------------------
// Durability primitives (RSK-02)
// ---------------------------------------------------------------------------

/// Flush a record file to durable storage. On macOS a plain `fsync` only
/// reaches the drive cache, so the promised crash-durability model requires
/// `F_FULLFSYNC`; a failed synchronization is never downgraded into success.
fn sync_file_durable(file: &File) -> Result<(), ReplayError> {
    #[cfg(target_os = "macos")]
    {
        // SAFETY: the descriptor is live and owned by `file`.
        let ret = unsafe { libc::fcntl(file.as_raw_fd(), libc::F_FULLFSYNC) };
        if ret == -1 {
            return io_fail(
                "publication",
                "record_fullfsync_failed",
                RECOVERY_INSPECT,
                &std::io::Error::last_os_error(),
            );
        }
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    {
        file.sync_all().map_err(|error| {
            io_error(
                "publication",
                "record_sync_failed",
                RECOVERY_INSPECT,
                &error,
            )
        })
    }
}

fn sync_dir(dir: &TrustedDir) -> Result<(), ReplayError> {
    #[cfg(target_os = "macos")]
    {
        // On macOS/Darwin, standard fsync() on a directory fd returns EINVAL.
        // F_FULLFSYNC provides physical barrier synchronization for metadata.
        // SAFETY: the descriptor is live and owned by the retained TrustedDir.
        let ret = unsafe { libc::fcntl(dir.fd(), libc::F_FULLFSYNC) };
        if ret == -1 {
            // A successful open proves only that the directory exists. It does
            // not prove that the directory entry update reached durable storage.
            return io_fail(
                "publication",
                "directory_fullfsync_failed",
                RECOVERY_INSPECT,
                &std::io::Error::last_os_error(),
            );
        }
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    {
        dir.file.sync_all().map_err(|error| {
            io_error(
                "publication",
                "directory_sync_failed",
                RECOVERY_INSPECT,
                &error,
            )
        })
    }
}

/// Publish without replacing an existing destination (DEF-01 crash-safe form).
///
/// Sequence: uniquely nonced create-new temporary (mode 0600) -> write ->
/// durable file sync -> `linkat` create-new publication (atomic visibility,
/// `EEXIST` refusal) -> directory sync -> temporary unlink -> directory sync.
///
/// An interruption at any point leaves only recognised staging residue,
/// which strict scans tolerate without trusting and which can never block a
/// later publication because every attempt uses a fresh nonce. Failed
/// attempts never delete anything: residue from this or any earlier attempt
/// is evidence of ambiguity and is recovered by scan tolerance alone.
fn publish_new(dir: &TrustedDir, stem: &str, bytes: &[u8]) -> Result<(), ReplayError> {
    if stem.is_empty()
        || stem.contains('/')
        || !stem
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
    {
        return fail("publication", "invalid_record_name", RECOVERY_INSPECT);
    }
    persistence_fault(UnixPersistenceFaultPoint::BeforeTemporaryCreate)?;
    let temporary = staging_temp_name(stem);
    let c_temp = c_name(&temporary)?;
    let c_dest = c_name(stem)?;

    // SAFETY: dir.fd() is live and retained; c_temp is NUL-terminated;
    // O_CREAT|O_EXCL gives create-new semantics; the returned descriptor is
    // owned and transferred into a File exactly once.
    let fd = unsafe {
        libc::openat(
            dir.fd(),
            c_temp.as_ptr(),
            libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            0o600,
        )
    };
    if fd < 0 {
        return io_fail(
            "publication",
            "temporary_create_failed",
            RECOVERY_INSPECT,
            &std::io::Error::last_os_error(),
        );
    }
    let mut file = unsafe { File::from_raw_fd(fd) };
    // umask may have stripped bits from the create mode; fchmod is exact.
    // SAFETY: the descriptor is live and owned by `file`.
    if unsafe { libc::fchmod(file.as_raw_fd(), 0o600) } != 0 {
        return io_fail(
            "publication",
            "temporary_mode_failed",
            RECOVERY_INSPECT,
            &std::io::Error::last_os_error(),
        );
    }
    persistence_fault(UnixPersistenceFaultPoint::AfterTemporaryCreate)?;

    if fault_partial_write_armed() {
        let half = bytes.len() / 2;
        let _ = file.write_all(&bytes[..half]);
        return fail(
            "fault_injection",
            "injected_persistence_fault",
            "Test-only deterministic interruption; no operator action.",
        );
    }
    file.write_all(bytes).map_err(|error| {
        io_error(
            "publication",
            "temporary_write_failed",
            RECOVERY_INSPECT,
            &error,
        )
    })?;
    sync_file_durable(&file)?;
    persistence_fault(UnixPersistenceFaultPoint::AfterFileSync)?;
    persistence_fault(UnixPersistenceFaultPoint::BeforePublication)?;

    // SAFETY: both names are NUL-terminated and relative to the same live
    // retained directory descriptor. linkat creates a new directory entry and
    // fails with EEXIST when the destination already exists; it never
    // replaces. POSIX link() visibility is atomic on the supported local
    // filesystems. Investigated alternative: Linux renameat2(RENAME_NOREPLACE)
    // offers the same create-new guarantee but has no macOS equivalent;
    // linkat is the portable atomic create-new primitive, so both platforms
    // use it.
    let linked = unsafe { libc::linkat(dir.fd(), c_temp.as_ptr(), dir.fd(), c_dest.as_ptr(), 0) };
    if linked != 0 {
        let error = std::io::Error::last_os_error();
        if error.raw_os_error() == Some(libc::EEXIST) {
            return fail_with(
                "publication",
                "destination_already_exists",
                "The immutable record already exists; a second publication is refused.",
                dir.path().join(stem).display().to_string(),
            );
        }
        return io_fail(
            "publication",
            "publication_link_failed",
            RECOVERY_INSPECT,
            &error,
        );
    }
    persistence_fault(UnixPersistenceFaultPoint::AfterPublication)?;
    persistence_fault(UnixPersistenceFaultPoint::BeforeDirectorySync)?;
    sync_dir(dir)?;
    persistence_fault(UnixPersistenceFaultPoint::BeforeTemporaryCleanup)?;

    // The record is published and durable at this point. A failed unlink of
    // our own recognised staging residue must not report the publication as
    // failed: the residue is tolerated by every strict scan and can never be
    // mistaken for an authoritative record.
    if let Err(error) = unlink_entry(dir, &temporary) {
        crate::replay::record_replay_diagnostic(
            "publication",
            "staging_residue_retained",
            "Recognised interrupted staging residue was retained; it is tolerated and never trusted.",
            Some(format!(
                "{}: os_error:{}",
                temporary,
                error.raw_os_error().unwrap_or(-1)
            )),
        );
    }
    persistence_fault(UnixPersistenceFaultPoint::AfterCleanup)?;
    sync_dir(dir)?;
    Ok(())
}

fn unlink_entry(dir: &TrustedDir, name: &str) -> Result<(), std::io::Error> {
    let cname = CString::new(name.as_bytes())
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "interior NUL"))?;
    // SAFETY: dir.fd() is live; cname is NUL-terminated; flags 0 removes a
    // file entry only (AT_REMOVEDIR is used explicitly for directories).
    if unsafe { libc::unlinkat(dir.fd(), cname.as_ptr(), 0) } != 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

fn remove_empty_dir_at(dir: &TrustedDir, name: &str) -> Result<(), std::io::Error> {
    let cname = CString::new(name.as_bytes())
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "interior NUL"))?;
    // SAFETY: dir.fd() is live; cname is NUL-terminated; AT_REMOVEDIR removes
    // a directory and fails when it is not empty, which is exactly the
    // guarantee required before dropping recognised interrupted state.
    if unsafe { libc::unlinkat(dir.fd(), cname.as_ptr(), libc::AT_REMOVEDIR) } != 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

fn make_dir_at(parent: &TrustedDir, name: &str) -> Result<TrustedDir, ReplayError> {
    let cname = c_name(name)?;
    // SAFETY: parent.fd() is live; cname is NUL-terminated; mode 0700 is
    // masked by umask, so the exact mode is re-established with fchmod below.
    let rc = unsafe { libc::mkdirat(parent.fd(), cname.as_ptr(), 0o700) };
    if rc != 0 {
        return io_fail(
            "provision",
            "directory_create_failed",
            RECOVERY_REPROVISION,
            &std::io::Error::last_os_error(),
        );
    }
    let dir = open_dir_at(parent, name)?;
    // SAFETY: the descriptor is live and owned by the freshly opened TrustedDir.
    if unsafe { libc::fchmod(dir.fd(), 0o700) } != 0 {
        return io_fail(
            "provision",
            "directory_mode_failed",
            RECOVERY_PERMISSIONS,
            &std::io::Error::last_os_error(),
        );
    }
    Ok(dir)
}

// ---------------------------------------------------------------------------
// Hierarchy validation and provisioning
// ---------------------------------------------------------------------------

struct ValidatedHierarchy {
    #[allow(dead_code)]
    replay: TrustedDir,
    #[allow(dead_code)]
    v1: TrustedDir,
    locks: TrustedDir,
    claims: TrustedDir,
    chains: TrustedDir,
}

fn validate_hierarchy_at(root: &TrustedDir) -> Result<ValidatedHierarchy, ReplayError> {
    let replay = match open_dir_at(root, "replay") {
        Ok(replay) => replay,
        Err(error) => {
            if missing_at(root, "replay") {
                return fail(
                    "hierarchy_validation",
                    "missing_replay_subtree",
                    "Run provision-replay against this host-data root first.",
                );
            }
            return Err(error);
        }
    };
    if names_at(&replay)? != vec!["v1".to_owned()] {
        return fail_with(
            "hierarchy_validation",
            "unexpected_directory_entry",
            "Remove unrecognized files or directories from the replay subtree.",
            replay.path().display().to_string(),
        );
    }
    let v1 = open_dir_at(&replay, "v1")?;
    let expected = ["FORMAT.json", "chains", "claims", "locks"];
    for name in names_at(&v1)? {
        if expected.contains(&name.as_str()) {
            continue;
        }
        if is_recognized_staging_residue(StagingContext::Format, &name) {
            verify_residue_trust(&v1, &name)?;
            continue;
        }
        return fail_with(
            "hierarchy_validation",
            "unexpected_directory_entry",
            "Remove unrecognized files or directories from the replay subtree.",
            v1.path().join(&name).display().to_string(),
        );
    }
    for name in expected {
        if missing_at(&v1, name) {
            return fail_with(
                "hierarchy_validation",
                "missing_required_entry",
                RECOVERY_REPROVISION,
                name.to_owned(),
            );
        }
    }
    if read_record_at(&v1, "FORMAT.json")? != FORMAT_BYTES {
        return fail(
            "hierarchy_validation",
            "invalid_format_file",
            "The replay format marker is missing or hostile; reprovision from a trusted backup.",
        );
    }
    let chains = open_dir_at(&v1, "chains")?;
    let claims = open_dir_at(&v1, "claims")?;
    let locks = open_dir_at(&v1, "locks")?;
    Ok(ValidatedHierarchy {
        replay,
        v1,
        locks,
        claims,
        chains,
    })
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
    crate::replay::clear_replay_diagnostic();
    let root = open_trusted_root(root_path)?;
    match stat_at(&root, "replay") {
        StatAtOutcome::Entry(stat) => {
            if stat.st_mode & libc::S_IFMT != libc::S_IFDIR {
                return fail_with(
                    "hierarchy_validation",
                    "replay_subtree_not_a_directory",
                    "Remove the blocking file or reprovision from a trusted backup.",
                    root.path().join("replay").display().to_string(),
                );
            }
            if !stat_is_trusted(&stat, true) {
                return trust_violation(&root.path().join("replay"));
            }
            validate_hierarchy_at(&root)?;
            ReplayLedger::open_with_root(root)?;
            return Ok(ProvisionReplayOutcome::AlreadyProvisioned);
        }
        StatAtOutcome::Error => {
            return fail(
                "hierarchy_validation",
                "directory_read_failed",
                RECOVERY_INSPECT,
            )
        }
        StatAtOutcome::Missing => {}
    }
    let replay = make_dir_at(&root, "replay")?;
    let v1 = make_dir_at(&replay, "v1")?;
    for name in ["locks", "claims", "chains"] {
        make_dir_at(&v1, name)?;
    }
    sync_dir(&v1)?;
    publish_new(&v1, "FORMAT.json", FORMAT_BYTES)?;
    sync_dir(&replay)?;
    sync_dir(&root)?;
    validate_hierarchy_at(&root)?;
    Ok(ProvisionReplayOutcome::Provisioned)
}

// ---------------------------------------------------------------------------
// Locks
// ---------------------------------------------------------------------------

pub struct LogicalKeyLock {
    _file: File,
}

impl LogicalKeyLock {
    fn acquire(
        directory: &TrustedDir,
        logical_key: &LogicalExecutionKey,
    ) -> Result<Self, ReplayError> {
        let name = format!("{}.lock", logical_key.filename_digest());
        let file = match open_file_at(directory, &name, libc::O_RDWR | libc::O_CREAT, 0o600) {
            Ok(file) => file,
            Err(error) => return io_fail("locking", "lock_open_failed", RECOVERY_INSPECT, &error),
        };
        // umask-independent exact mode for a freshly created lock file.
        // SAFETY: the descriptor is live and owned by `file`.
        if unsafe { libc::fchmod(file.as_raw_fd(), 0o600) } != 0 {
            return io_fail(
                "locking",
                "lock_mode_failed",
                RECOVERY_PERMISSIONS,
                &std::io::Error::last_os_error(),
            );
        }
        check_file_trust(&file, &directory.path().join(&name))?;
        // SAFETY: the descriptor is owned by this guard and remains live until
        // the lock is released by Drop.
        if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX) } != 0 {
            return io_fail(
                "locking",
                "lock_acquisition_failed",
                "Another process holds the logical-key lock; retry only after it exits.",
                &std::io::Error::last_os_error(),
            );
        }
        Ok(Self { _file: file })
    }
}

// ---------------------------------------------------------------------------
// Ledger
// ---------------------------------------------------------------------------

pub struct ReplayLedger {
    root: TrustedDir,
    locks: TrustedDir,
    claims: TrustedDir,
    chains: TrustedDir,
}

impl std::fmt::Debug for ReplayLedger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReplayLedger")
            .field("root", &self.root.path)
            .finish()
    }
}

fn read_claim_at(claims: &TrustedDir, key: &LogicalExecutionKey) -> Result<Claim, ReplayError> {
    let name = format!("{}.claim.json", key.filename_digest());
    let bytes = read_record_at(claims, &name)?;
    Claim::from_canonical_bytes(&bytes, key)
        .map_err(|_| record_error("record_validation", "claim_parse_failed"))
}

fn existing_claim_at(
    claims: &TrustedDir,
    key: &LogicalExecutionKey,
) -> Result<Option<Claim>, ReplayError> {
    let name = format!("{}.claim.json", key.filename_digest());
    match stat_at(claims, &name) {
        StatAtOutcome::Missing => Ok(None),
        StatAtOutcome::Error => fail("record_validation", "claim_stat_failed", RECOVERY_INSPECT),
        StatAtOutcome::Entry(_) => read_claim_at(claims, key).map(Some),
    }
}

impl ReplayLedger {
    pub fn open(root_path: &Path) -> Result<Self, ReplayError> {
        crate::replay::clear_replay_diagnostic();
        let root = open_trusted_root(root_path)?;
        Self::open_with_root(root)
    }

    fn open_with_root(root: TrustedDir) -> Result<Self, ReplayError> {
        let hierarchy = validate_hierarchy_at(&root)?;
        persistence_fault(UnixPersistenceFaultPoint::DuringRestartScan)?;
        for name in names_at(&hierarchy.locks)? {
            let valid_lock = name
                .strip_suffix(".lock")
                .is_some_and(|value| is_lower_hex(value, 64));
            if !valid_lock {
                return fail_with(
                    "hierarchy_validation",
                    "unexpected_directory_entry",
                    "Remove unrecognized files or directories from the replay subtree.",
                    hierarchy.locks.path().join(&name).display().to_string(),
                );
            }
            match stat_at(&hierarchy.locks, &name) {
                StatAtOutcome::Entry(stat) if stat_is_trusted(&stat, false) => {}
                _ => return trust_violation(&hierarchy.locks.path().join(&name)),
            }
        }
        let mut claims_by_execution = HashMap::new();
        for name in names_at(&hierarchy.claims)? {
            if is_recognized_staging_residue(StagingContext::Claims, &name) {
                verify_residue_trust(&hierarchy.claims, &name)?;
                continue;
            }
            let Some(digest) = name.strip_suffix(".claim.json") else {
                return fail_with(
                    "hierarchy_validation",
                    "unexpected_directory_entry",
                    "Remove unrecognized files or directories from the replay subtree.",
                    hierarchy.claims.path().join(&name).display().to_string(),
                );
            };
            if !is_lower_hex(digest, 64) {
                return fail_with(
                    "hierarchy_validation",
                    "unexpected_directory_entry",
                    "Remove unrecognized files or directories from the replay subtree.",
                    hierarchy.claims.path().join(&name).display().to_string(),
                );
            }
            let logical_key = LogicalExecutionKey::from_digest(format!("sha256:{digest}"))
                .map_err(|_| record_error("record_validation", "claim_name_parse_failed"))?;
            let claim = read_claim_at(&hierarchy.claims, &logical_key)?;
            if claims_by_execution
                .insert(claim.execution_id.filename_digest(), claim)
                .is_some()
            {
                return fail(
                    "hierarchy_validation",
                    "duplicate_execution_identity",
                    RECOVERY_INSPECT,
                );
            }
        }
        let mut seen = HashSet::new();
        for prefix_name in names_at(&hierarchy.chains)? {
            if !is_lower_hex(&prefix_name, 2) {
                return fail_with(
                    "hierarchy_validation",
                    "unexpected_directory_entry",
                    "Remove unrecognized files or directories from the replay subtree.",
                    hierarchy
                        .chains
                        .path()
                        .join(&prefix_name)
                        .display()
                        .to_string(),
                );
            }
            let prefix = open_dir_at(&hierarchy.chains, &prefix_name)?;
            let execution_names = names_at(&prefix)?;
            if execution_names.is_empty() {
                // An empty prefix directory is recognised interrupted internal
                // state (an interrupted publication or a completed operator
                // resolve); it holds no records and cannot affect authority.
                continue;
            }
            for execution_name in execution_names {
                if !is_lower_hex(&execution_name, 64)
                    || !execution_name.starts_with(&prefix_name)
                    || !seen.insert(execution_name.clone())
                {
                    return fail_with(
                        "hierarchy_validation",
                        "unexpected_directory_entry",
                        "Remove unrecognized files or directories from the replay subtree.",
                        prefix.path().join(&execution_name).display().to_string(),
                    );
                }
                let Some(claim) = claims_by_execution.get(&execution_name) else {
                    return fail_with(
                        "hierarchy_validation",
                        "orphan_execution_chain",
                        RECOVERY_INSPECT,
                        execution_name,
                    );
                };
                let execution = open_dir_at(&prefix, &execution_name)?;
                let generations = read_generation_chain(&execution, claim)?;
                validate_chain(claim, &generations)
                    .map_err(|_| record_error("record_validation", "chain_validation_failed"))?;
            }
        }
        Ok(Self {
            root,
            locks: hierarchy.locks,
            claims: hierarchy.claims,
            chains: hierarchy.chains,
        })
    }

    /// Recover logical key and binding for one durable execution identity.
    /// Used by Authority Gate restart reconciliation; read-only.
    pub fn claim_material_for(
        &self,
        execution_id: &str,
    ) -> Option<(LogicalExecutionKey, ExecutionBinding)> {
        for name in names_at(&self.claims).ok()? {
            if is_recognized_staging_residue(StagingContext::Claims, &name) {
                continue;
            }
            let digest = name.strip_suffix(".claim.json")?;
            if !is_lower_hex(digest, 64) {
                return None;
            }
            let logical_key = LogicalExecutionKey::from_digest(format!("sha256:{digest}")).ok()?;
            let claim = read_claim_at(&self.claims, &logical_key).ok()?;
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
        for name in names_at(&self.claims)? {
            if is_recognized_staging_residue(StagingContext::Claims, &name) {
                verify_residue_trust(&self.claims, &name)?;
                continue;
            }
            let Some(digest) = name.strip_suffix(".claim.json") else {
                return fail_with(
                    "hierarchy_validation",
                    "unexpected_directory_entry",
                    "Remove unrecognized files or directories from the replay subtree.",
                    self.claims.path().join(&name).display().to_string(),
                );
            };
            if !is_lower_hex(digest, 64) {
                return fail_with(
                    "hierarchy_validation",
                    "unexpected_directory_entry",
                    "Remove unrecognized files or directories from the replay subtree.",
                    self.claims.path().join(&name).display().to_string(),
                );
            }
            let logical_key = LogicalExecutionKey::from_digest(format!("sha256:{digest}"))
                .map_err(|_| record_error("record_validation", "claim_name_parse_failed"))?;
            let claim = read_claim_at(&self.claims, &logical_key)?;
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
        if let Some(claim) = existing_claim_at(&ledger.claims, &logical_key)? {
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
        let published = existing_claim_at(&ledger.claims, &claim.logical_key)?
            .ok_or(ReplayError::PersistenceUnavailable)?;
        if published != claim {
            return fail("publication", "claim_reopen_mismatch", RECOVERY_INSPECT);
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

/// Read and parse the generation records of one execution directory,
/// tolerating (without trusting) recognised staging residue.
fn read_generation_chain(
    execution: &TrustedDir,
    claim: &Claim,
) -> Result<Vec<Generation>, ReplayError> {
    let mut records = Vec::new();
    for name in names_at(execution)? {
        if is_recognized_staging_residue(StagingContext::Generations, &name) {
            verify_residue_trust(execution, &name)?;
            continue;
        }
        let Some(number) = canonical_generation_number(&name) else {
            return fail_with(
                "hierarchy_validation",
                "unexpected_directory_entry",
                "Remove unrecognized files or directories from the replay subtree.",
                execution.path().join(&name).display().to_string(),
            );
        };
        records.push((number, name));
    }
    records.sort_by_key(|(number, _)| *number);
    let mut generations = Vec::new();
    for (expected, (number, name)) in records.into_iter().enumerate() {
        if expected as u64 != number {
            return fail(
                "record_validation",
                "generation_sequence_gap",
                RECOVERY_INSPECT,
            );
        }
        let generation = Generation::from_canonical_bytes(&read_record_at(execution, &name)?)
            .map_err(|_| record_error("record_validation", "generation_parse_failed"))?;
        if generation.execution_id_digest != claim.execution_id.digest() {
            return fail(
                "record_validation",
                "generation_identity_mismatch",
                RECOVERY_INSPECT,
            );
        }
        generations.push(generation);
    }
    Ok(generations)
}

fn execution_directories(
    ledger: &ReplayLedger,
    claim: &Claim,
) -> Result<Option<(TrustedDir, TrustedDir)>, ReplayError> {
    let digest = claim.execution_id.filename_digest();
    let prefix_name = &digest[..2];
    if missing_at(&ledger.chains, prefix_name) {
        return Ok(None);
    }
    let prefix = open_dir_at(&ledger.chains, prefix_name)?;
    if missing_at(&prefix, &digest) {
        return Ok(None);
    }
    let execution = open_dir_at(&prefix, &digest)?;
    Ok(Some((prefix, execution)))
}

fn reconstruct(
    ledger: &ReplayLedger,
    claim: &Claim,
) -> Result<(ReplayState, Vec<Generation>), ReplayError> {
    persistence_fault(UnixPersistenceFaultPoint::DuringRestartScan)?;
    let Some((_prefix, execution)) = execution_directories(ledger, claim)? else {
        return Ok((ReplayState::ClaimedNoState, Vec::new()));
    };
    let generations = read_generation_chain(&execution, claim)?;
    let state = validate_chain(claim, &generations)
        .map_err(|_| record_error("record_validation", "chain_validation_failed"))?;
    Ok((state, generations))
}

fn ensure_execution_directories(
    ledger: &ReplayLedger,
    claim: &Claim,
) -> Result<(TrustedDir, TrustedDir), ReplayError> {
    let digest = claim.execution_id.filename_digest();
    let prefix_name = &digest[..2];
    let prefix = if missing_at(&ledger.chains, prefix_name) {
        let prefix = make_dir_at(&ledger.chains, prefix_name)?;
        sync_dir(&ledger.chains)?;
        prefix
    } else {
        open_dir_at(&ledger.chains, prefix_name)?
    };
    let execution = if missing_at(&prefix, &digest) {
        let execution = make_dir_at(&prefix, &digest)?;
        sync_dir(&prefix)?;
        execution
    } else {
        open_dir_at(&prefix, &digest)?
    };
    Ok((prefix, execution))
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
            return fail(
                "publication",
                "intent_requires_fresh_claimed_admission",
                "A recovered admission can never advance; manual resolution only.",
            );
        }
        let generation =
            Generation::intent(&self.claim).map_err(|_| ReplayError::PersistenceUnavailable)?;
        self.publish_generation(generation)
    }
    pub fn publish_armed(&mut self) -> Result<(), ReplayError> {
        if !self.fresh || self.state != ReplayState::IntentRecorded || self.generations.len() != 1 {
            return fail(
                "publication",
                "armed_requires_fresh_intent_admission",
                "A recovered admission can never advance; manual resolution only.",
            );
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
            return fail(
                "publication",
                "terminal_requires_armed_admission",
                "Only an armed admission may publish a terminal generation.",
            );
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
            return fail(
                "publication",
                "durable_chain_diverged_from_admission",
                "The durable chain no longer matches this admission; manual resolution only.",
            );
        }
        let (_prefix, directory) = ensure_execution_directories(&self.ledger, &self.claim)?;
        let name = format!("g{:016}.json", generation.number);
        let bytes = generation
            .canonical_bytes()
            .map_err(|_| ReplayError::PersistenceUnavailable)?;
        publish_new(&directory, &name, &bytes)?;
        let reopened = Generation::from_canonical_bytes(&read_record_at(&directory, &name)?)
            .map_err(|_| record_error("record_validation", "generation_reopen_parse_failed"))?;
        if reopened != generation {
            return fail(
                "publication",
                "generation_reopen_mismatch",
                RECOVERY_INSPECT,
            );
        }
        self.state = generation.state;
        self.generations.push(generation);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Operator reconciliation of an abandoned `claimed_no_state` admission (DEF-03)
// ---------------------------------------------------------------------------

/// Explicitly authorised operator route that releases one logical key whose
/// fresh admission was abandoned at `claimed_no_state` (approval-consumption
/// failure, Trail authorisation failure, intent-publication failure, or host
/// termination before generation zero became durable).
///
/// Safety facts enforced before any mutation:
///
/// - the durable state is exactly `claimed_no_state`: the claim exists and no
///   generation record was ever published. Because provider dispatch requires
///   a durable armed generation, no external effect can have started;
/// - the claim record is first preserved byte-identically under quarantine in
///   the host-data root (`replay-quarantine/`), with an append-only audit
///   line, before the original is released. Historical evidence and the
///   original execution identity are never deleted or reused;
/// - every other durable state (intent, armed, uncertain, terminal) is
///   refused; those remain manual-resolution-only;
/// - the logical-key lock is held across the whole transition, and the
///   operation is idempotent across crashes at every intermediate point.
pub fn resolve_claimed_no_state(
    root_path: &Path,
    execution_id: &str,
) -> Result<ReplayResolveReport, ReplayError> {
    crate::replay::clear_replay_diagnostic();
    let parsed = ExecutionId::parse(execution_id.to_owned()).map_err(|_| {
        crate::replay::record_replay_diagnostic(
            "resolution",
            "invalid_execution_id",
            "Supply the exact exec_UUID reported by gate STATUS recovery_required entries.",
            None,
        );
        ReplayError::InvalidIdentifier
    })?;
    let ledger = ReplayLedger::open(root_path)?;
    let logical_key = find_claim_key_by_execution(&ledger, parsed.as_str())?;
    let claim = read_claim_at(&ledger.claims, &logical_key)?;
    let (state, _) = reconstruct(&ledger, &claim)?;
    if state != ReplayState::ClaimedNoState {
        return refuse_unresolvable_state(parsed.as_str(), state);
    }

    // Hold the logical-key exclusion across the whole reconciliation.
    let _lock = LogicalKeyLock::acquire(&ledger.locks, &logical_key)?;

    // Re-establish the facts under the lock: nothing may have advanced.
    let claim = read_claim_at(&ledger.claims, &logical_key)?;
    let (state, _) = reconstruct(&ledger, &claim)?;
    if state != ReplayState::ClaimedNoState {
        return refuse_unresolvable_state(parsed.as_str(), state);
    }

    let bytes = claim
        .canonical_bytes()
        .map_err(|_| ReplayError::PersistenceUnavailable)?;
    let record_name = format!("{}.claim.json", claim.logical_key.filename_digest());

    // 1. Quarantine copy first: evidence is preserved before anything moves.
    let quarantine = ensure_quarantine_dir(&ledger.root)?;
    let already_quarantined = match stat_at(&quarantine, &record_name) {
        StatAtOutcome::Missing => {
            publish_new(&quarantine, &record_name, &bytes)?;
            let reopened = read_record_at(&quarantine, &record_name)?;
            if reopened != bytes {
                return fail("resolution", "quarantine_reopen_mismatch", RECOVERY_INSPECT);
            }
            false
        }
        StatAtOutcome::Entry(_) => {
            // An interrupted earlier resolve already preserved the record.
            let existing = read_record_at(&quarantine, &record_name)?;
            if existing != bytes {
                return fail_with(
                    "resolution",
                    "quarantine_conflict",
                    "The quarantine already holds different bytes for this logical key; manual inspection required.",
                    record_name.clone(),
                );
            }
            true
        }
        StatAtOutcome::Error => {
            return fail("resolution", "quarantine_stat_failed", RECOVERY_INSPECT)
        }
    };

    // 2. Append the audit line before the release so a crash cannot leave an
    //    unaudited removal behind. A repeated resolve appends a second honest
    //    line marked already_quarantined.
    append_resolve_audit(&quarantine, &claim, &record_name, already_quarantined)?;

    // 3. Remove the recognised interrupted chain state (guaranteed empty of
    //    records because the durable state is claimed_no_state), then release
    //    the claim. Order matters: while the claim exists an empty chain
    //    directory is consistent; after the claim is gone it would be an
    //    orphan that every strict scan must reject.
    if let Some((prefix, execution)) = execution_directories(&ledger, &claim)? {
        if !names_at(&execution)?.is_empty() {
            return fail("resolution", "chain_directory_not_empty", RECOVERY_INSPECT);
        }
        let execution_name = claim.execution_id.filename_digest();
        // The open descriptor must be released before the directory can be
        // removed; rmdir refuses non-empty directories regardless.
        drop(execution);
        remove_empty_dir_at(&prefix, &execution_name).map_err(|error| {
            io_error(
                "resolution",
                "chain_directory_remove_failed",
                RECOVERY_INSPECT,
                &error,
            )
        })?;
        sync_dir(&prefix)?;
        let prefix_name = execution_name[..2].to_owned();
        drop(prefix);
        // The prefix may still hold other executions; ENOTEMPTY (or EBUSY on a
        // still-open descriptor) is the expected, tolerated outcome. An empty
        // prefix directory is recognised interrupted state and is tolerated
        // by every strict scan.
        if remove_empty_dir_at(&ledger.chains, &prefix_name).is_ok() {
            sync_dir(&ledger.chains)?;
        }
    }
    match unlink_entry(&ledger.claims, &record_name) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            // An interrupted earlier resolve already released the claim.
        }
        Err(error) => {
            return io_fail(
                "resolution",
                "claim_release_failed",
                RECOVERY_INSPECT,
                &error,
            )
        }
    }
    sync_dir(&ledger.claims)?;

    Ok(ReplayResolveReport {
        execution_id: claim.execution_id.as_str().to_owned(),
        logical_key_digest: claim.logical_key.as_digest().to_owned(),
        binding_digest: claim.binding_digest.clone(),
        claim_digest: claim.claim_digest.clone(),
        quarantined_record: record_name,
        already_quarantined,
    })
}

fn refuse_unresolvable_state<T>(execution_id: &str, state: ReplayState) -> Result<T, ReplayError> {
    fail_with(
        "resolution",
        "state_not_resolvable",
        "Only an abandoned claimed_no_state admission may be resolved; every other state is manual-resolution-only.",
        format!("execution_id:{execution_id} state:{state:?}"),
    )
}

fn find_claim_key_by_execution(
    ledger: &ReplayLedger,
    execution_id: &str,
) -> Result<LogicalExecutionKey, ReplayError> {
    for name in names_at(&ledger.claims)? {
        if is_recognized_staging_residue(StagingContext::Claims, &name) {
            continue;
        }
        let Some(digest) = name.strip_suffix(".claim.json") else {
            continue;
        };
        if !is_lower_hex(digest, 64) {
            continue;
        }
        let logical_key = LogicalExecutionKey::from_digest(format!("sha256:{digest}"))
            .map_err(|_| record_error("record_validation", "claim_name_parse_failed"))?;
        let claim = read_claim_at(&ledger.claims, &logical_key)?;
        if claim.execution_id.as_str() == execution_id {
            return Ok(logical_key);
        }
    }
    fail_with(
        "resolution",
        "execution_not_found",
        "No durable claim carries this execution identity; check gate STATUS recovery_required entries.",
        execution_id.to_owned(),
    )
}

fn ensure_quarantine_dir(root: &TrustedDir) -> Result<TrustedDir, ReplayError> {
    match stat_at(root, REPLAY_QUARANTINE_DIR_NAME) {
        StatAtOutcome::Missing => {
            let dir = make_dir_at(root, REPLAY_QUARANTINE_DIR_NAME)?;
            sync_dir(root)?;
            Ok(dir)
        }
        StatAtOutcome::Entry(_) => open_dir_at(root, REPLAY_QUARANTINE_DIR_NAME),
        StatAtOutcome::Error => fail("resolution", "quarantine_stat_failed", RECOVERY_INSPECT),
    }
}

fn append_resolve_audit(
    quarantine: &TrustedDir,
    claim: &Claim,
    record_name: &str,
    already_quarantined: bool,
) -> Result<(), ReplayError> {
    let line = resolve_audit_line(claim, record_name, already_quarantined)?;
    let mut file = match open_file_at(
        quarantine,
        REPLAY_RESOLVE_LOG_NAME,
        libc::O_WRONLY | libc::O_APPEND | libc::O_CREAT,
        0o600,
    ) {
        Ok(file) => file,
        Err(error) => return io_fail("resolution", "audit_open_failed", RECOVERY_INSPECT, &error),
    };
    // SAFETY: the descriptor is live and owned by `file`.
    if unsafe { libc::fchmod(file.as_raw_fd(), 0o600) } != 0 {
        return io_fail(
            "resolution",
            "audit_mode_failed",
            RECOVERY_PERMISSIONS,
            &std::io::Error::last_os_error(),
        );
    }
    check_file_trust(&file, &quarantine.path().join(REPLAY_RESOLVE_LOG_NAME))?;
    file.write_all(&line)
        .map_err(|error| io_error("resolution", "audit_write_failed", RECOVERY_INSPECT, &error))?;
    sync_file_durable(&file)?;
    sync_dir(quarantine)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_replay_requires_directory_full_sync_support() {
        let root = temp_root("macos-fullsync");
        std::fs::create_dir(&root).unwrap();
        sync_dir(&open_trusted_root(&root).unwrap())
            .expect("macOS replay persistence requires directory full-sync support");
        std::fs::remove_dir_all(root).unwrap();
    }

    fn temp_root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!("tethers-replay-{label}-{}", uuid::Uuid::new_v4()))
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

    fn provisioned_root(label: &str) -> PathBuf {
        let root = temp_root(label);
        std::fs::create_dir(&root).unwrap();
        provision_replay(&root).unwrap();
        root
    }

    fn claims_dir(root: &Path) -> PathBuf {
        root.join("replay/v1/claims")
    }

    fn nonce() -> String {
        uuid::Uuid::new_v4().simple().to_string()
    }

    fn digest64(fill: char) -> String {
        fill.to_string().repeat(64)
    }

    #[test]
    fn linux_replay_lifecycle_reopens_with_identical_terminal_state() {
        let root = provisioned_root("lifecycle");
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
        let execution = admission.execution_id().to_owned();
        drop(admission);

        let reopened = Rc::new(ReplayLedger::open(&root).unwrap());
        let recovered = ReplayLedger::admit_or_recover_owned(&reopened, key, binding()).unwrap();
        assert!(!recovered.is_fresh());
        assert_eq!(recovered.state(), ReplayState::Succeeded);
        assert_eq!(recovered.execution_id(), execution);
        drop(recovered);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn linux_replay_rejects_symlinked_root() {
        use std::os::unix::fs::symlink;

        let parent = temp_root("symlink-root");
        let real = parent.join("real");
        let linked = parent.join("linked");
        std::fs::create_dir_all(&real).unwrap();
        symlink(&real, &linked).unwrap();
        assert_eq!(
            provision_replay(&linked).unwrap_err(),
            ReplayError::PersistenceUnavailable
        );
        let diagnostic = crate::replay::last_replay_diagnostic().expect("diagnostic recorded");
        assert_eq!(diagnostic.phase, "path_traversal");
        assert_eq!(diagnostic.reason, "not_a_plain_directory");
        std::fs::remove_dir_all(parent).unwrap();
    }

    // -----------------------------------------------------------------------
    // DEF-01: recognised staging residue is tolerated; unrecognised is not
    // -----------------------------------------------------------------------

    #[test]
    fn stranded_recognised_residue_never_blocks_open_or_republish() {
        let root = provisioned_root("residue-tolerated");
        // Hostile-content residue wearing the exact internal staging shape in
        // claims/, v1/ (FORMAT context) must never be parsed or trusted.
        let hostile = b"not-a-record".to_vec();
        let claim_residue = format!("{}.claim.json.{}.tmp", digest64('a'), nonce());
        let residue_path = claims_dir(&root).join(&claim_residue);
        std::fs::write(&residue_path, &hostile).unwrap();
        std::fs::set_permissions(&residue_path, std::fs::Permissions::from_mode(0o600)).unwrap();
        let format_residue = format!("FORMAT.json.{}.tmp", nonce());
        std::fs::write(root.join("replay/v1").join(&format_residue), &hostile).unwrap();

        // The ledger still opens, and a fresh admission publishes normally.
        let ledger = Rc::new(ReplayLedger::open(&root).unwrap());
        let key = LogicalExecutionKey::derive("anchor", "eval", "action").unwrap();
        let mut admission =
            ReplayLedger::admit_or_recover_owned(&ledger, key.clone(), binding()).unwrap();
        assert!(admission.is_fresh());
        admission.publish_intent().unwrap();
        drop(admission);

        // A second stranded residue for the SAME authoritative stem still does
        // not block anything: every attempt uses a fresh nonce.
        std::fs::write(
            claims_dir(&root).join(format!("{}.claim.json.{}.tmp", digest64('b'), nonce())),
            &hostile,
        )
        .unwrap();
        let reopened = Rc::new(ReplayLedger::open(&root).unwrap());
        let recovered = ReplayLedger::admit_or_recover_owned(&reopened, key, binding()).unwrap();
        assert!(!recovered.is_fresh());
        assert_eq!(recovered.state(), ReplayState::IntentRecorded);

        // The residue files were never modified, deleted, or parsed.
        assert_eq!(std::fs::read(&residue_path).unwrap(), hostile);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn unrecognised_tmp_entries_still_fail_closed() {
        let root = provisioned_root("residue-hostile");
        // Deterministic legacy-style temporary without a nonce: unrecognised.
        std::fs::write(
            claims_dir(&root).join(format!(".{}.claim.json.tmp", digest64('a'))),
            b"x",
        )
        .unwrap();
        assert_eq!(
            ReplayLedger::open(&root).unwrap_err(),
            ReplayError::PersistenceUnavailable
        );
        let diagnostic = crate::replay::last_replay_diagnostic().expect("diagnostic recorded");
        assert_eq!(diagnostic.phase, "hierarchy_validation");
        assert_eq!(diagnostic.reason, "unexpected_directory_entry");
        std::fs::remove_dir_all(root).unwrap();

        // Nonce-shaped residue with an invalid stem is also unrecognised.
        let root = provisioned_root("residue-hostile-stem");
        std::fs::write(
            claims_dir(&root).join(format!("hostile.claim.json.{}.tmp", nonce())),
            b"x",
        )
        .unwrap();
        assert!(ReplayLedger::open(&root).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn symlink_wearing_a_residue_name_fails_closed() {
        use std::os::unix::fs::symlink;
        let root = provisioned_root("residue-symlink");
        let target = temp_root("residue-symlink-target");
        std::fs::write(&target, b"elsewhere").unwrap();
        let link = claims_dir(&root).join(format!("{}.claim.json.{}.tmp", digest64('a'), nonce()));
        symlink(&target, &link).unwrap();
        assert_eq!(
            ReplayLedger::open(&root).unwrap_err(),
            ReplayError::PersistenceUnavailable
        );
        let diagnostic = crate::replay::last_replay_diagnostic().expect("diagnostic recorded");
        assert_eq!(diagnostic.phase, "security_validation");
        std::fs::remove_file(&target).unwrap();
        std::fs::remove_dir_all(root).unwrap();
    }

    // -----------------------------------------------------------------------
    // DEF-01: deterministic fault injection at every publication boundary
    // -----------------------------------------------------------------------

    fn admit_expect_err(ledger: &Rc<ReplayLedger>, key: &LogicalExecutionKey) {
        let result = ReplayLedger::admit_or_recover_owned(ledger, key.clone(), binding());
        assert!(result.is_err(), "admission must fail at the injected fault");
    }

    fn count_authoritative(path: &Path) -> usize {
        std::fs::read_dir(path)
            .unwrap()
            .filter(|entry| {
                !entry
                    .as_ref()
                    .unwrap()
                    .file_name()
                    .to_string_lossy()
                    .ends_with(".tmp")
            })
            .count()
    }

    #[test]
    fn faults_before_publication_leave_only_tolerated_residue_and_allow_retry() {
        for point in [
            UnixPersistenceFaultPoint::BeforeTemporaryCreate,
            UnixPersistenceFaultPoint::AfterTemporaryCreate,
            UnixPersistenceFaultPoint::DuringPartialWrite,
            UnixPersistenceFaultPoint::AfterFileSync,
            UnixPersistenceFaultPoint::BeforePublication,
        ] {
            let root = provisioned_root("fault-pre");
            let ledger = Rc::new(ReplayLedger::open(&root).unwrap());
            let key = LogicalExecutionKey::derive("anchor", "eval", "action").unwrap();
            with_persistence_fault(point, || admit_expect_err(&ledger, &key));

            // Operation definitely not committed: no claim record exists.
            assert_eq!(
                count_authoritative(&claims_dir(&root)),
                0,
                "no authoritative claim may exist before publication"
            );

            // Restart: the ledger reopens despite any stranded residue.
            let reopened = Rc::new(ReplayLedger::open(&root).unwrap());
            // Retry with a fresh admission succeeds: residue never blocks.
            let mut admission =
                ReplayLedger::admit_or_recover_owned(&reopened, key, binding()).unwrap();
            assert!(admission.is_fresh());
            admission.publish_intent().unwrap();
            assert_eq!(admission.state(), ReplayState::IntentRecorded);
            std::fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn partial_write_residue_is_never_accepted_as_a_record() {
        let root = provisioned_root("fault-partial");
        let ledger = Rc::new(ReplayLedger::open(&root).unwrap());
        let key = LogicalExecutionKey::derive("anchor", "eval", "action").unwrap();
        with_persistence_fault(UnixPersistenceFaultPoint::DuringPartialWrite, || {
            admit_expect_err(&ledger, &key)
        });
        // Exactly one residue file exists and it is not the claim name.
        let entries: Vec<String> = std::fs::read_dir(claims_dir(&root))
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(entries.len(), 1);
        assert!(entries[0].ends_with(".tmp"), "residue: {entries:?}");
        let reopened = Rc::new(ReplayLedger::open(&root).unwrap());
        // The partial residue grants no state: a fresh admission is issued.
        let admission = ReplayLedger::admit_or_recover_owned(&reopened, key, binding()).unwrap();
        assert!(admission.is_fresh());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn faults_after_publication_leave_a_durable_record_the_restart_sees() {
        for point in [
            UnixPersistenceFaultPoint::AfterPublication,
            UnixPersistenceFaultPoint::BeforeDirectorySync,
            UnixPersistenceFaultPoint::BeforeTemporaryCleanup,
            UnixPersistenceFaultPoint::AfterCleanup,
        ] {
            let root = provisioned_root("fault-post");
            let ledger = Rc::new(ReplayLedger::open(&root).unwrap());
            let key = LogicalExecutionKey::derive("anchor", "eval", "action").unwrap();
            with_persistence_fault(point, || admit_expect_err(&ledger, &key));

            // The claim became durable before the injected interruption: the
            // restart sees it, and the state requires manual resolution or the
            // operator route. It is never silently retried.
            let reopened = Rc::new(ReplayLedger::open(&root).unwrap());
            let recovered =
                ReplayLedger::admit_or_recover_owned(&reopened, key, binding()).unwrap();
            assert!(!recovered.is_fresh());
            assert_eq!(recovered.state(), ReplayState::ClaimedNoState);
            std::fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn restart_scan_fault_fails_open_closed() {
        let root = provisioned_root("fault-scan");
        let result = with_persistence_fault(UnixPersistenceFaultPoint::DuringRestartScan, || {
            ReplayLedger::open(&root)
        });
        assert!(result.is_err());
        std::fs::remove_dir_all(root).unwrap();
    }

    // -----------------------------------------------------------------------
    // RSK-01: ownership and permission contract
    // -----------------------------------------------------------------------

    #[test]
    fn group_writable_claims_directory_fails_closed() {
        let root = provisioned_root("perm-claims");
        std::fs::set_permissions(claims_dir(&root), std::fs::Permissions::from_mode(0o775))
            .unwrap();
        assert_eq!(
            ReplayLedger::open(&root).unwrap_err(),
            ReplayError::PersistenceUnavailable
        );
        let diagnostic = crate::replay::last_replay_diagnostic().expect("diagnostic recorded");
        assert_eq!(diagnostic.phase, "security_validation");
        assert_eq!(diagnostic.reason, "unsafe_ownership_or_permissions");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn other_writable_record_file_fails_closed() {
        let root = provisioned_root("perm-record");
        let ledger = Rc::new(ReplayLedger::open(&root).unwrap());
        let key = LogicalExecutionKey::derive("anchor", "eval", "action").unwrap();
        let admission =
            ReplayLedger::admit_or_recover_owned(&ledger, key.clone(), binding()).unwrap();
        let claim_file = claims_dir(&root).join(format!("{}.claim.json", key.filename_digest()));
        drop(admission);
        std::fs::set_permissions(&claim_file, std::fs::Permissions::from_mode(0o606)).unwrap();
        assert!(ReplayLedger::open(&root).is_err());
        let diagnostic = crate::replay::last_replay_diagnostic().expect("diagnostic recorded");
        assert_eq!(diagnostic.phase, "security_validation");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn provision_creates_private_directories_and_records() {
        let root = provisioned_root("perm-created");
        for relative in [
            "replay",
            "replay/v1",
            "replay/v1/claims",
            "replay/v1/locks",
            "replay/v1/chains",
        ] {
            let mode = std::fs::metadata(root.join(relative))
                .unwrap()
                .permissions()
                .mode();
            assert_eq!(mode & 0o777, 0o700, "{relative} must be private");
        }
        let ledger = Rc::new(ReplayLedger::open(&root).unwrap());
        let key = LogicalExecutionKey::derive("anchor", "eval", "action").unwrap();
        let admission = ReplayLedger::admit_or_recover_owned(&ledger, key, binding()).unwrap();
        let entries: Vec<String> = std::fs::read_dir(claims_dir(&root))
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(entries.len(), 1);
        let mode = std::fs::metadata(claims_dir(&root).join(&entries[0]))
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600);
        drop(admission);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn empty_prefix_directory_is_tolerated() {
        let root = provisioned_root("empty-prefix");
        std::fs::create_dir(root.join("replay/v1/chains/ab")).unwrap();
        std::fs::set_permissions(
            root.join("replay/v1/chains/ab"),
            std::fs::Permissions::from_mode(0o700),
        )
        .unwrap();
        ReplayLedger::open(&root).unwrap();
        std::fs::remove_dir_all(root).unwrap();
    }

    // -----------------------------------------------------------------------
    // DEF-03: abandoned admission and the operator resolve route
    // -----------------------------------------------------------------------

    #[test]
    fn abandoned_fresh_admission_blocks_until_resolved_then_admits_fresh() {
        let root = provisioned_root("resolve");
        let key = LogicalExecutionKey::derive("anchor", "eval", "action").unwrap();
        let abandoned;
        {
            let ledger = Rc::new(ReplayLedger::open(&root).unwrap());
            let admission =
                ReplayLedger::admit_or_recover_owned(&ledger, key.clone(), binding()).unwrap();
            assert!(admission.is_fresh());
            abandoned = admission.execution_id().to_owned();
            // Host dies before any generation: the claim stays durable.
        }
        {
            // Recovery across restart: the abandoned claim blocks execution.
            let ledger = Rc::new(ReplayLedger::open(&root).unwrap());
            let mut recovered =
                ReplayLedger::admit_or_recover_owned(&ledger, key.clone(), binding()).unwrap();
            assert!(!recovered.is_fresh());
            assert_eq!(recovered.state(), ReplayState::ClaimedNoState);
            assert!(recovered.publish_intent().is_err());
        }

        let report = resolve_claimed_no_state(&root, &abandoned).unwrap();
        assert_eq!(report.execution_id, abandoned);
        assert!(!report.already_quarantined);

        // Evidence preserved: the quarantine holds the byte-identical claim
        // and the audit log names the exact execution identity.
        let quarantine = root.join(REPLAY_QUARANTINE_DIR_NAME);
        let preserved = std::fs::read(quarantine.join(&report.quarantined_record)).unwrap();
        let claim = Claim::from_canonical_bytes(
            &preserved,
            &LogicalExecutionKey::from_digest(report.logical_key_digest.clone()).unwrap(),
        )
        .unwrap();
        assert_eq!(claim.execution_id.as_str(), abandoned);
        let audit = std::fs::read_to_string(quarantine.join(REPLAY_RESOLVE_LOG_NAME)).unwrap();
        let line: serde_json::Value = serde_json::from_str(audit.trim()).unwrap();
        assert_eq!(line["schema"], "tethers.replay_resolve/1");
        assert_eq!(line["execution_id"], abandoned);
        assert_eq!(line["resolved_state"], "claimed_no_state");
        assert_eq!(line["already_quarantined"], false);

        // The original claim is released; a fresh admission with a NEW
        // execution identity completes the whole lifecycle.
        let ledger = Rc::new(ReplayLedger::open(&root).unwrap());
        let mut admission = ReplayLedger::admit_or_recover_owned(&ledger, key, binding()).unwrap();
        assert!(admission.is_fresh());
        assert_ne!(admission.execution_id(), abandoned);
        admission.publish_intent().unwrap();
        admission.publish_armed().unwrap();
        admission
            .publish_terminal(
                ReplayState::Succeeded,
                "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc".into(),
            )
            .unwrap();
        drop(admission);

        // Resolving again reports the truth: nothing carries that identity.
        let error = resolve_claimed_no_state(&root, &abandoned).unwrap_err();
        assert_eq!(error, ReplayError::PersistenceUnavailable);
        let diagnostic = crate::replay::last_replay_diagnostic().expect("diagnostic recorded");
        assert_eq!(diagnostic.reason, "execution_not_found");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn resolve_refuses_every_state_beyond_claimed_no_state() {
        let root = provisioned_root("resolve-refuse");
        let key = LogicalExecutionKey::derive("anchor", "eval", "action").unwrap();
        let ledger = Rc::new(ReplayLedger::open(&root).unwrap());
        let mut admission =
            ReplayLedger::admit_or_recover_owned(&ledger, key.clone(), binding()).unwrap();
        admission.publish_intent().unwrap();
        let execution = admission.execution_id().to_owned();
        drop(admission);

        // IntentRecorded is durable evidence of a recorded intent: refused.
        let error = resolve_claimed_no_state(&root, &execution).unwrap_err();
        assert_eq!(error, ReplayError::PersistenceUnavailable);
        let diagnostic = crate::replay::last_replay_diagnostic().expect("diagnostic recorded");
        assert_eq!(diagnostic.reason, "state_not_resolvable");
        // The generation record is untouched.
        let ledger = Rc::new(ReplayLedger::open(&root).unwrap());
        let recovered =
            ReplayLedger::admit_or_recover_owned(&ledger, key.clone(), binding()).unwrap();
        assert_eq!(recovered.state(), ReplayState::IntentRecorded);

        // Armed and terminal states are refused the same way.
        let root2 = provisioned_root("resolve-refuse-armed");
        let ledger2 = Rc::new(ReplayLedger::open(&root2).unwrap());
        let mut admission =
            ReplayLedger::admit_or_recover_owned(&ledger2, key.clone(), binding()).unwrap();
        admission.publish_intent().unwrap();
        admission.publish_armed().unwrap();
        let execution = admission.execution_id().to_owned();
        drop(admission);
        assert!(resolve_claimed_no_state(&root2, &execution).is_err());
        let diagnostic = crate::replay::last_replay_diagnostic().expect("diagnostic recorded");
        assert_eq!(diagnostic.reason, "state_not_resolvable");

        let root3 = provisioned_root("resolve-refuse-terminal");
        let ledger3 = Rc::new(ReplayLedger::open(&root3).unwrap());
        let mut admission = ReplayLedger::admit_or_recover_owned(&ledger3, key, binding()).unwrap();
        admission.publish_intent().unwrap();
        admission.publish_armed().unwrap();
        admission
            .publish_terminal(
                ReplayState::Uncertain,
                "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc".into(),
            )
            .unwrap();
        let execution = admission.execution_id().to_owned();
        drop(admission);
        assert!(resolve_claimed_no_state(&root3, &execution).is_err());
        for root in [root, root2, root3] {
            std::fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn resolve_rejects_invalid_execution_identity() {
        let root = provisioned_root("resolve-invalid");
        let error = resolve_claimed_no_state(&root, "not-an-execution-id").unwrap_err();
        assert_eq!(error, ReplayError::InvalidIdentifier);
        let diagnostic = crate::replay::last_replay_diagnostic().expect("diagnostic recorded");
        assert_eq!(diagnostic.reason, "invalid_execution_id");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn resolve_is_idempotent_across_an_interrupted_earlier_attempt() {
        let root = provisioned_root("resolve-idempotent");
        let key = LogicalExecutionKey::derive("anchor", "eval", "action").unwrap();
        let abandoned;
        {
            let ledger = Rc::new(ReplayLedger::open(&root).unwrap());
            let admission =
                ReplayLedger::admit_or_recover_owned(&ledger, key.clone(), binding()).unwrap();
            abandoned = admission.execution_id().to_owned();
            // Simulate a crash between the quarantine copy and the release:
            // copy the claim bytes into the quarantine by hand first.
            let claim_path =
                claims_dir(&root).join(format!("{}.claim.json", key.filename_digest()));
            let quarantine = root.join(REPLAY_QUARANTINE_DIR_NAME);
            std::fs::create_dir(&quarantine).unwrap();
            std::fs::set_permissions(&quarantine, std::fs::Permissions::from_mode(0o700)).unwrap();
            std::fs::copy(&claim_path, quarantine.join(file_name_of(&claim_path))).unwrap();
        }
        let report = resolve_claimed_no_state(&root, &abandoned).unwrap();
        assert!(report.already_quarantined);
        // The claim is released and the ledger stays healthy.
        let ledger = Rc::new(ReplayLedger::open(&root).unwrap());
        let admission = ReplayLedger::admit_or_recover_owned(&ledger, key, binding()).unwrap();
        assert!(admission.is_fresh());
        drop(admission);
        let audit = std::fs::read_to_string(
            root.join(REPLAY_QUARANTINE_DIR_NAME)
                .join(REPLAY_RESOLVE_LOG_NAME),
        )
        .unwrap();
        let line: serde_json::Value = serde_json::from_str(audit.trim()).unwrap();
        assert_eq!(line["already_quarantined"], true);
        std::fs::remove_dir_all(root).unwrap();
    }

    fn file_name_of(path: &Path) -> String {
        path.file_name().unwrap().to_string_lossy().into_owned()
    }
}
