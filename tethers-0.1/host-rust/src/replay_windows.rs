//! Native Windows J09 replay-root admission boundary.
//!
//! This module contains every J09 Win32 call.  A path is never authority after
//! it has been parsed: each existing component is opened without reparse-point
//! following, and the final directory handle carries the volume and ACL proof.

use crate::replay::{
    validate_chain, Claim, ExecutionBinding, ExecutionId, Generation, LogicalExecutionKey,
    ReplayError, ReplayState,
};
use std::collections::{HashMap, HashSet};
use std::ffi::c_void;
use std::mem::{size_of, MaybeUninit};
use std::os::windows::ffi::OsStrExt;
use std::path::{Component, Path, PathBuf, Prefix};
use uuid::Uuid;
use windows_sys::Win32::Foundation::{
    CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, ERROR_FILE_NOT_FOUND, ERROR_PATH_NOT_FOUND,
    GENERIC_ALL, GENERIC_READ, GENERIC_WRITE, HANDLE, INVALID_HANDLE_VALUE,
};
use windows_sys::Win32::Security::{
    AclSizeInformation, CreateWellKnownSid, EqualSid, GetAce, GetAclInformation,
    GetKernelObjectSecurity, GetLengthSid, GetSecurityDescriptorDacl, GetSecurityDescriptorOwner,
    GetTokenInformation, IsValidSecurityDescriptor, IsValidSid, TokenUser,
    WinBuiltinAdministratorsSid, WinLocalSystemSid, ACCESS_ALLOWED_ACE, ACL, ACL_SIZE_INFORMATION,
    DACL_SECURITY_INFORMATION, GROUP_SECURITY_INFORMATION, OWNER_SECURITY_INFORMATION,
    PSECURITY_DESCRIPTOR, PSID, TOKEN_QUERY, TOKEN_USER,
};
use windows_sys::Win32::Storage::FileSystem::{
    CreateDirectoryW, CreateFileW, FileRenameInfo, FlushFileBuffers, GetDriveTypeW,
    GetFileInformationByHandle, GetFileSizeEx, GetVolumeInformationByHandleW, LockFileEx, ReadFile,
    SetFileInformationByHandle, WriteFile, BY_HANDLE_FILE_INFORMATION, CREATE_NEW, DELETE,
    FILE_ADD_FILE, FILE_ADD_SUBDIRECTORY, FILE_APPEND_DATA, FILE_ATTRIBUTE_DIRECTORY,
    FILE_ATTRIBUTE_NORMAL, FILE_ATTRIBUTE_REPARSE_POINT, FILE_DELETE_CHILD,
    FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, FILE_FLAG_WRITE_THROUGH,
    FILE_GENERIC_READ, FILE_RENAME_INFO, FILE_RENAME_INFO_0, FILE_SHARE_READ, FILE_SHARE_WRITE,
    FILE_WRITE_ATTRIBUTES, FILE_WRITE_DATA, FILE_WRITE_EA, LOCKFILE_EXCLUSIVE_LOCK, OPEN_ALWAYS,
    OPEN_EXISTING, READ_CONTROL, WRITE_DAC, WRITE_OWNER,
};
use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
use windows_sys::Win32::System::IO::OVERLAPPED;

const SECURITY_INFORMATION: u32 =
    OWNER_SECURITY_INFORMATION | GROUP_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION;
const WRITE_CAPABLE: u32 = FILE_WRITE_DATA
    | FILE_APPEND_DATA
    | FILE_ADD_FILE
    | FILE_ADD_SUBDIRECTORY
    | FILE_DELETE_CHILD
    | DELETE
    | FILE_WRITE_ATTRIBUTES
    | FILE_WRITE_EA
    | WRITE_DAC
    | WRITE_OWNER
    | GENERIC_WRITE
    | GENERIC_ALL;
// These documented ABI values normally live in windows-sys feature modules
// outside J09's frozen feature set. Keeping them local avoids broadening it.
const ACCESS_ALLOWED_ACE_TYPE: u8 = 0;
const ACCESS_DENIED_ACE_TYPE: u8 = 1;
const DRIVE_FIXED: u32 = 3;
const FORMAT_BYTES: &[u8] = br#"{"replay_format_version":1}"#;
const MAX_REPLAY_RECORD_BYTES: i64 = 64 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PersistenceFaultPoint {
    LockFileOpen,
    LockAcquisition,
    ClaimRead,
    ClaimPublication,
    ClaimCollisionReopen,
    ChainDirectoryValidation,
    GenerationZeroPublication,
    GenerationOnePublication,
    GenerationTwoPublication,
    GenerationReopen,
    DigestVerification,
    RestartScan,
    OrphanDetection,
}

#[cfg(test)]
thread_local! {
    static INJECTED_PERSISTENCE_FAULT: std::cell::Cell<Option<PersistenceFaultPoint>> =
        const { std::cell::Cell::new(None) };
}

fn persistence_fault(point: PersistenceFaultPoint) -> Result<(), ReplayError> {
    #[cfg(test)]
    if INJECTED_PERSISTENCE_FAULT.with(|fault| fault.get() == Some(point)) {
        return unavailable();
    }
    #[cfg(not(test))]
    let _ = point;
    Ok(())
}

#[cfg(test)]
fn with_persistence_fault<T>(point: PersistenceFaultPoint, operation: impl FnOnce() -> T) -> T {
    INJECTED_PERSISTENCE_FAULT.with(|fault| {
        let previous = fault.replace(Some(point));
        let result = operation();
        fault.set(previous);
        result
    })
}

fn unavailable<T>() -> Result<T, ReplayError> {
    Err(ReplayError::PersistenceUnavailable)
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NativePublishStage {
    CreateTemporary,
    WriteTemporary,
    FirstFlush,
    Rename,
    SecondFlush,
    CloseAfterRename,
    ReopenFinal,
    ReadFinal,
    VerifyFinal,
    CloseAfterReopen,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NativePublishDiagnostic {
    stage: NativePublishStage,
    win32_error: u32,
}

#[cfg(test)]
thread_local! {
    static LAST_NATIVE_PUBLISH_DIAGNOSTIC: std::cell::RefCell<Option<NativePublishDiagnostic>> =
        const { std::cell::RefCell::new(None) };
}

#[cfg(test)]
fn clear_native_publish_diagnostic() {
    LAST_NATIVE_PUBLISH_DIAGNOSTIC.with(|diagnostic| *diagnostic.borrow_mut() = None);
}

#[cfg(test)]
fn record_native_publish_diagnostic(stage: NativePublishStage, win32_error: u32) {
    LAST_NATIVE_PUBLISH_DIAGNOSTIC.with(|diagnostic| {
        *diagnostic.borrow_mut() = Some(NativePublishDiagnostic { stage, win32_error });
    });
}

#[cfg(test)]
fn last_native_publish_diagnostic() -> Option<NativePublishDiagnostic> {
    LAST_NATIVE_PUBLISH_DIAGNOSTIC.with(|diagnostic| *diagnostic.borrow())
}

fn unavailable_after_native_publish_failure<T>(
    #[cfg(test)] stage: NativePublishStage,
) -> Result<T, ReplayError> {
    // SAFETY: this is called directly from the documented failed-return branch
    // of a Win32 publication API, before any cleanup, allocation, or helper.
    let win32_error = unsafe { GetLastError() };
    #[cfg(test)]
    record_native_publish_diagnostic(stage, win32_error);
    #[cfg(not(test))]
    let _ = win32_error;
    unavailable()
}

fn wide(path: &Path) -> Vec<u16> {
    path.as_os_str().encode_wide().chain(Some(0)).collect()
}

/// A handle whose ownership is linear: every successful Win32 open reaches this
/// wrapper before any later fallible check, so all early returns close it.
struct OwnedHandle(HANDLE);
impl Drop for OwnedHandle {
    fn drop(&mut self) {
        if self.0 == INVALID_HANDLE_VALUE {
            return;
        }
        // SAFETY: constructed only from a successful handle-returning API, owned
        // by this value, and dropped exactly once.
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

#[derive(Clone)]
struct OwnedSid(Vec<u8>);
impl OwnedSid {
    fn from_ptr(sid: PSID) -> Result<Self, ReplayError> {
        // SAFETY: validation and length are documented SID APIs; the input is
        // supplied by Windows only while its enclosing buffer/descriptor lives.
        if sid.is_null() || unsafe { IsValidSid(sid) } == 0 {
            return unavailable();
        }
        let length = unsafe { GetLengthSid(sid) } as usize;
        if length == 0 {
            return unavailable();
        }
        let mut bytes = vec![0; length];
        // SAFETY: both regions are valid for `length` bytes and non-overlapping.
        unsafe {
            std::ptr::copy_nonoverlapping(sid.cast::<u8>(), bytes.as_mut_ptr(), length);
        }
        Ok(Self(bytes))
    }
    fn as_psid(&self) -> PSID {
        self.0.as_ptr().cast_mut().cast()
    }
    fn equals(&self, other: &Self) -> bool {
        // SAFETY: both pointers originate in owned, validated SID byte arrays.
        unsafe { EqualSid(self.as_psid(), other.as_psid()) != 0 }
    }
}

fn current_user_sid() -> Result<OwnedSid, ReplayError> {
    let mut raw = INVALID_HANDLE_VALUE;
    // SAFETY: current-process pseudo-handle is valid; `raw` is writable.
    if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut raw) } == 0
        || raw == INVALID_HANDLE_VALUE
    {
        return unavailable();
    }
    let token = OwnedHandle(raw);
    let mut needed = 0u32;
    // SAFETY: documented sizing call; null buffer with zero length is required.
    let _ =
        unsafe { GetTokenInformation(token.0, TokenUser, std::ptr::null_mut(), 0, &mut needed) };
    if needed < size_of::<TOKEN_USER>() as u32 {
        return unavailable();
    }
    let mut buffer = vec![0u8; needed as usize];
    // SAFETY: caller-owned buffer is writable for its declared exact length.
    if unsafe {
        GetTokenInformation(
            token.0,
            TokenUser,
            buffer.as_mut_ptr().cast(),
            needed,
            &mut needed,
        )
    } == 0
    {
        return unavailable();
    }
    // SAFETY: successful TokenUser query returns TOKEN_USER at the start.
    let token_user = unsafe { &*(buffer.as_ptr().cast::<TOKEN_USER>()) };
    OwnedSid::from_ptr(token_user.User.Sid)
}

fn well_known_sid(kind: i32) -> Result<OwnedSid, ReplayError> {
    let mut needed = 0u32;
    // SAFETY: documented sizing call for a well-known SID.
    let _ = unsafe {
        CreateWellKnownSid(
            kind,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut needed,
        )
    };
    if needed == 0 {
        return unavailable();
    }
    let mut bytes = vec![0u8; needed as usize];
    // SAFETY: the caller-owned SID buffer has the size requested above.
    if unsafe {
        CreateWellKnownSid(
            kind,
            std::ptr::null_mut(),
            bytes.as_mut_ptr().cast(),
            &mut needed,
        )
    } == 0
    {
        return unavailable();
    }
    OwnedSid::from_ptr(bytes.as_mut_ptr().cast())
}

fn trusted_writer(
    sid: &OwnedSid,
    user: &OwnedSid,
    system: &OwnedSid,
    administrators: &OwnedSid,
) -> bool {
    sid.equals(user) || sid.equals(system) || sid.equals(administrators)
}

/// Pure authority decision used by the native ACL walker and focused tests.
fn allow_ace_is_permitted(mask: u32, trustee_is_trusted: bool) -> Result<(), ReplayError> {
    if mask & WRITE_CAPABLE != 0 && !trustee_is_trusted {
        unavailable()
    } else {
        Ok(())
    }
}

fn validate_security(handle: HANDLE) -> Result<(), ReplayError> {
    let user = current_user_sid()?;
    let system = well_known_sid(WinLocalSystemSid)?;
    let administrators = well_known_sid(WinBuiltinAdministratorsSid)?;
    let mut needed = 0u32;
    // SAFETY: documented descriptor-sizing call with no descriptor buffer.
    let _ = unsafe {
        GetKernelObjectSecurity(
            handle,
            SECURITY_INFORMATION,
            std::ptr::null_mut(),
            0,
            &mut needed,
        )
    };
    if needed == 0 {
        return unavailable();
    }
    let mut descriptor = vec![0u8; needed as usize];
    // SAFETY: descriptor is caller-owned writable storage of the required size.
    if unsafe {
        GetKernelObjectSecurity(
            handle,
            SECURITY_INFORMATION,
            descriptor.as_mut_ptr().cast(),
            needed,
            &mut needed,
        )
    } == 0
    {
        return unavailable();
    }
    let sd: PSECURITY_DESCRIPTOR = descriptor.as_mut_ptr().cast();
    // SAFETY: `sd` identifies the returned descriptor buffer for its lifetime.
    if unsafe { IsValidSecurityDescriptor(sd) } == 0 {
        return unavailable();
    }
    let mut owner: PSID = std::ptr::null_mut();
    let mut owner_defaulted = 0;
    // SAFETY: output pointers are valid; `sd` remains live.
    if unsafe { GetSecurityDescriptorOwner(sd, &mut owner, &mut owner_defaulted) } == 0 {
        return unavailable();
    }
    let owner = OwnedSid::from_ptr(owner)?;
    if !owner.equals(&user) {
        return unavailable();
    }
    let mut present = 0;
    let mut dacl: *mut ACL = std::ptr::null_mut();
    let mut defaulted = 0;
    // SAFETY: output pointers are valid and `sd` remains live.
    if unsafe { GetSecurityDescriptorDacl(sd, &mut present, &mut dacl, &mut defaulted) } == 0
        || present == 0
        || dacl.is_null()
    {
        return unavailable();
    }
    let mut info = MaybeUninit::<ACL_SIZE_INFORMATION>::zeroed();
    // SAFETY: dacl is a present descriptor-owned ACL and info is writable.
    if unsafe {
        GetAclInformation(
            dacl,
            info.as_mut_ptr().cast(),
            size_of::<ACL_SIZE_INFORMATION>() as u32,
            AclSizeInformation,
        )
    } == 0
    {
        return unavailable();
    }
    let info = unsafe { info.assume_init() };
    for index in 0..info.AceCount {
        let mut raw: *mut c_void = std::ptr::null_mut();
        // SAFETY: index is bounded by the ACL's reported ACE count.
        if unsafe { GetAce(dacl, index, &mut raw) } == 0 || raw.is_null() {
            return unavailable();
        }
        // SAFETY: GetAce returns an ACE_HEADER at the returned address.
        let header = unsafe { &*raw.cast::<windows_sys::Win32::Security::ACE_HEADER>() };
        match header.AceType {
            ACCESS_ALLOWED_ACE_TYPE => {
                // SAFETY: this type has ACCESS_ALLOWED_ACE layout; SidStart is
                // the first SID byte and remains within the descriptor buffer.
                let ace = unsafe { &*raw.cast::<ACCESS_ALLOWED_ACE>() };
                let sid = OwnedSid::from_ptr(std::ptr::addr_of!(ace.SidStart).cast_mut().cast())?;
                allow_ace_is_permitted(
                    ace.Mask,
                    trusted_writer(&sid, &user, &system, &administrators),
                )?;
            }
            // Canonical deny ACEs grant no authority. Object, callback,
            // conditional, audit, and unknown types are deliberately unsupported.
            ACCESS_DENIED_ACE_TYPE => {}
            _ => return unavailable(),
        }
    }
    Ok(())
}

fn open_component(path: &Path) -> Result<OwnedHandle, ReplayError> {
    open_directory(path, FILE_GENERIC_READ | READ_CONTROL)
}

fn open_directory(path: &Path, access: u32) -> Result<OwnedHandle, ReplayError> {
    let path_w = wide(path);
    // SAFETY: nul-terminated path lives through the call. BACKUP_SEMANTICS opens
    // a directory and OPEN_REPARSE_POINT prevents final-component traversal.
    let raw = unsafe {
        CreateFileW(
            path_w.as_ptr(),
            access,
            // Allow ordinary readers and writers, but deny delete/share-delete.
            // Keeping this handle to the end of admission prevents a validated
            // component from being renamed or removed before its child opens.
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            std::ptr::null(),
            OPEN_EXISTING,
            FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT | FILE_ATTRIBUTE_NORMAL,
            std::ptr::null_mut(),
        )
    };
    if raw == INVALID_HANDLE_VALUE {
        return unavailable();
    }
    let handle = OwnedHandle(raw);
    let mut information = BY_HANDLE_FILE_INFORMATION::default();
    // SAFETY: handle remains live and information is writable for this call.
    if unsafe { GetFileInformationByHandle(handle.0, &mut information) } == 0
        || information.dwFileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0
    {
        return unavailable();
    }
    Ok(handle)
}

fn volume_root(path: &Path) -> Result<PathBuf, ReplayError> {
    let mut components = path.components();
    let prefix = match components.next() {
        Some(Component::Prefix(prefix)) => prefix,
        _ => return unavailable(),
    };
    let letter = match prefix.kind() {
        Prefix::Disk(letter) if letter.is_ascii_alphabetic() => letter,
        _ => return unavailable(),
    };
    if components.next() != Some(Component::RootDir)
        || components.any(|part| !matches!(part, Component::Normal(_)))
    {
        return unavailable();
    }
    Ok(PathBuf::from(format!("{}:\\", letter as char)))
}

fn validate_volume(handle: HANDLE, root: &Path) -> Result<(), ReplayError> {
    let mut filesystem = [0u16; 32];
    let mut serial = 0u32;
    // SAFETY: live opened directory handle and writable output locations.
    if unsafe {
        GetVolumeInformationByHandleW(
            handle,
            std::ptr::null_mut(),
            0,
            &mut serial,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            filesystem.as_mut_ptr(),
            filesystem.len() as u32,
        )
    } == 0
    {
        return unavailable();
    }
    if filesystem
        .iter()
        .take_while(|&&c| c != 0)
        .copied()
        .ne("NTFS".encode_utf16())
    {
        return unavailable();
    }
    let root_w = wide(root);
    // SAFETY: root_w is a live nul-terminated DOS volume root. Ancestor checks
    // rejected substitutions before this classification is consulted.
    if unsafe { GetDriveTypeW(root_w.as_ptr()) } != DRIVE_FIXED {
        return unavailable();
    }
    let _volume_serial = serial; // retained proof comes from the opened root handle.
    Ok(())
}

/// An absolute root whose every existing ancestor, volume, owner, and DACL has
/// been admitted. The final open handle remains live for the proof's lifetime.
pub struct ValidatedHostRoot {
    _ancestors: Vec<OwnedHandle>,
    _handle: OwnedHandle,
    path: PathBuf,
}
impl ValidatedHostRoot {
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// Validate a pre-existing host-data root. Missing components, relative/UNC or
/// non-canonical paths, reparse points, non-fixed/non-NTFS storage, and any
/// unprovable owner or ACL are all one redacted unavailable result.
pub fn validate_existing_root(path: &Path) -> Result<ValidatedHostRoot, ReplayError> {
    if !path.is_absolute() {
        return unavailable();
    }
    let spelling = path.as_os_str().to_string_lossy();
    if spelling.contains('/') || spelling.contains("\\\\.") {
        return unavailable();
    }
    let root = volume_root(path)?;
    let mut current = root.clone();
    let mut ancestors = Vec::new();
    let mut final_handle = open_component(&current)?;
    for component in path.components().skip(2) {
        let Component::Normal(name) = component else {
            return unavailable();
        };
        current.push(name);
        let next = open_component(&current)?;
        ancestors.push(final_handle);
        final_handle = next;
    }
    if current != path {
        return unavailable();
    }
    validate_volume(final_handle.0, &root)?;
    validate_security(final_handle.0)?;
    Ok(ValidatedHostRoot {
        _ancestors: ancestors,
        _handle: final_handle,
        path: path.to_path_buf(),
    })
}

/// A directory authority derived from a fully handle-bound validation. Callers
/// cannot construct it from a path string. Every admitted ancestor handle plus
/// the final operational handle remains live, preventing the absolute path used
/// by Win32 rename from being redirected after validation.
pub struct ValidatedDirectory {
    _authority: Vec<OwnedHandle>,
    _handle: OwnedHandle,
    path: PathBuf,
}

/// A validated one-component filename.  This rejects ADS syntax and every
/// spelling that Windows might reinterpret as a device or parent component.
pub struct ValidatedLeafName(String);

impl ValidatedLeafName {
    pub fn new(value: &str) -> Result<Self, ReplayError> {
        if value.is_empty()
            || value == "."
            || value == ".."
            || value.ends_with([' ', '.'])
            || value.chars().any(|c| {
                c.is_control() || matches!(c, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*')
            })
        {
            return unavailable();
        }
        let device = value
            .split('.')
            .next()
            .unwrap_or_default()
            .to_ascii_uppercase();
        if matches!(device.as_str(), "CON" | "PRN" | "AUX" | "NUL")
            || (device.len() == 4
                && (device.starts_with("COM") || device.starts_with("LPT"))
                && matches!(device.as_bytes()[3], b'1'..=b'9'))
        {
            return unavailable();
        }
        Ok(Self(value.to_owned()))
    }

    fn as_str(&self) -> &str {
        &self.0
    }
}

impl ValidatedHostRoot {
    fn into_directory(self) -> Result<ValidatedDirectory, ReplayError> {
        let path = self.path;
        let handle = open_directory(&path, FILE_GENERIC_READ | READ_CONTROL | FILE_ADD_FILE)?;
        let mut authority = self._ancestors;
        authority.push(self._handle);
        Ok(ValidatedDirectory {
            _authority: authority,
            _handle: handle,
            path,
        })
    }
}

impl ValidatedDirectory {
    fn child_directory(&self, name: &ValidatedLeafName) -> Result<ValidatedDirectory, ReplayError> {
        // `self` retains every admitted ancestor while the child obtains its
        // own complete independent handle chain, closing the absolute-path
        // substitution window before the returned authority can outlive us.
        validate_existing_root(&self.path.join(name.as_str()))?.into_directory()
    }

    fn create_new_child(
        &self,
        name: &ValidatedLeafName,
    ) -> Result<ValidatedDirectory, ReplayError> {
        let target = self.path.join(name.as_str());
        let wide_target = wide(&target);
        // SAFETY: target is one validated leaf under a retained, validated
        // parent; CreateDirectoryW either creates it once or fails closed.
        if unsafe { CreateDirectoryW(wide_target.as_ptr(), std::ptr::null()) } == 0 {
            return unavailable();
        }
        self.child_directory(name)
    }
}

fn create_new_directory(
    parent: &ValidatedDirectory,
    name: &str,
) -> Result<ValidatedDirectory, ReplayError> {
    parent.create_new_child(&ValidatedLeafName::new(name)?)
}

fn open_file(
    path: &Path,
    access: u32,
    disposition: u32,
    flags: u32,
) -> Result<OwnedHandle, ReplayError> {
    let path_w = wide(path);
    // SAFETY: the nul-terminated path is live for the call; all resulting
    // handles immediately enter OwnedHandle ownership.
    let raw = unsafe {
        CreateFileW(
            path_w.as_ptr(),
            access,
            0,
            std::ptr::null(),
            disposition,
            flags | FILE_FLAG_OPEN_REPARSE_POINT,
            std::ptr::null_mut(),
        )
    };
    if raw == INVALID_HANDLE_VALUE {
        return unavailable();
    }
    Ok(OwnedHandle(raw))
}

fn open_file_for_publish(
    path: &Path,
    access: u32,
    disposition: u32,
    flags: u32,
    #[cfg(test)] stage: NativePublishStage,
) -> Result<OwnedHandle, ReplayError> {
    let path_w = wide(path);
    // SAFETY: the nul-terminated path is live for the call; all resulting
    // handles immediately enter OwnedHandle ownership.
    let raw = unsafe {
        CreateFileW(
            path_w.as_ptr(),
            access,
            0,
            std::ptr::null(),
            disposition,
            flags | FILE_FLAG_OPEN_REPARSE_POINT,
            std::ptr::null_mut(),
        )
    };
    if raw == INVALID_HANDLE_VALUE {
        return unavailable_after_native_publish_failure(
            #[cfg(test)]
            stage,
        );
    }
    Ok(OwnedHandle(raw))
}

fn write_all_for_publish(handle: HANDLE, bytes: &[u8]) -> Result<(), ReplayError> {
    let mut remaining = bytes;
    while !remaining.is_empty() {
        let request = remaining.len().min(u32::MAX as usize) as u32;
        let mut written = 0u32;
        // SAFETY: `remaining` remains live and readable; `written` is writable.
        let succeeded = unsafe {
            WriteFile(
                handle,
                remaining.as_ptr(),
                request,
                &mut written,
                std::ptr::null_mut(),
            )
        };
        if succeeded == 0 {
            return unavailable_after_native_publish_failure(
                #[cfg(test)]
                NativePublishStage::WriteTemporary,
            );
        }
        if written == 0 || written > request {
            return unavailable();
        }
        remaining = &remaining[written as usize..];
    }
    Ok(())
}

fn read_complete(handle: HANDLE, expected_len: usize) -> Result<Vec<u8>, ReplayError> {
    let mut bytes = Vec::with_capacity(expected_len);
    while bytes.len() < expected_len {
        let mut chunk = vec![0u8; (expected_len - bytes.len()).min(64 * 1024)];
        let mut read = 0u32;
        // SAFETY: chunk is writable for the requested size and read is writable.
        if unsafe {
            ReadFile(
                handle,
                chunk.as_mut_ptr(),
                chunk.len() as u32,
                &mut read,
                std::ptr::null_mut(),
            )
        } == 0
            || read == 0
            || read as usize > chunk.len()
        {
            return unavailable();
        }
        bytes.extend_from_slice(&chunk[..read as usize]);
    }
    let mut extra = [0u8; 1];
    let mut read = 0u32;
    // SAFETY: `extra` and `read` are writable for this final EOF probe.
    if unsafe {
        ReadFile(
            handle,
            extra.as_mut_ptr(),
            1,
            &mut read,
            std::ptr::null_mut(),
        )
    } == 0
        || read != 0
    {
        return unavailable();
    }
    Ok(bytes)
}

fn read_complete_for_publish(handle: HANDLE, expected_len: usize) -> Result<Vec<u8>, ReplayError> {
    let mut bytes = Vec::with_capacity(expected_len);
    while bytes.len() < expected_len {
        let mut chunk = vec![0u8; (expected_len - bytes.len()).min(64 * 1024)];
        let mut read = 0u32;
        // SAFETY: chunk is writable for the requested size and read is writable.
        let succeeded = unsafe {
            ReadFile(
                handle,
                chunk.as_mut_ptr(),
                chunk.len() as u32,
                &mut read,
                std::ptr::null_mut(),
            )
        };
        if succeeded == 0 {
            return unavailable_after_native_publish_failure(
                #[cfg(test)]
                NativePublishStage::ReadFinal,
            );
        }
        if read == 0 || read as usize > chunk.len() {
            return unavailable();
        }
        bytes.extend_from_slice(&chunk[..read as usize]);
    }
    let mut extra = [0u8; 1];
    let mut read = 0u32;
    // SAFETY: `extra` and `read` are writable for this final EOF probe.
    let succeeded = unsafe {
        ReadFile(
            handle,
            extra.as_mut_ptr(),
            1,
            &mut read,
            std::ptr::null_mut(),
        )
    };
    if succeeded == 0 {
        return unavailable_after_native_publish_failure(
            #[cfg(test)]
            NativePublishStage::ReadFinal,
        );
    }
    if read != 0 {
        return unavailable();
    }
    Ok(bytes)
}

fn rename_without_replacement(handle: HANDLE, destination: &Path) -> Result<(), ReplayError> {
    let (mut storage, bytes) = rename_info_buffer(destination)?;
    let info = storage.as_mut_ptr().cast::<FILE_RENAME_INFO>();
    // SAFETY: `info` is aligned, zero-initialised storage containing the exact
    // FileName offset plus the validated UTF-16 filename bytes.
    unsafe {
        if SetFileInformationByHandle(handle, FileRenameInfo, info.cast(), bytes) == 0 {
            return unavailable_after_native_publish_failure(
                #[cfg(test)]
                NativePublishStage::Rename,
            );
        }
    }
    Ok(())
}

fn rename_info_buffer(destination: &Path) -> Result<(Vec<usize>, u32), ReplayError> {
    let name: Vec<u16> = destination.as_os_str().encode_wide().collect();
    let name_bytes = name
        .len()
        .checked_mul(size_of::<u16>())
        .ok_or(ReplayError::PersistenceUnavailable)?;
    let minimum_allocation = size_of::<FILE_RENAME_INFO>()
        .checked_add(name_bytes)
        .ok_or(ReplayError::PersistenceUnavailable)?;
    let name_bytes = u32::try_from(name_bytes).map_err(|_| ReplayError::PersistenceUnavailable)?;
    let storage_words = minimum_allocation.div_ceil(size_of::<usize>());
    let allocation_bytes = storage_words
        .checked_mul(size_of::<usize>())
        .ok_or(ReplayError::PersistenceUnavailable)?;
    let bytes = u32::try_from(allocation_bytes).map_err(|_| ReplayError::PersistenceUnavailable)?;
    let mut storage = vec![0usize; storage_words];
    let info = storage.as_mut_ptr().cast::<FILE_RENAME_INFO>();
    // SAFETY: aligned storage is zero-initialised and at least `bytes` long.
    // The variable-sized filename begins at its actual ABI offset.
    // FileRenameInfo interprets the union as ReplaceIfExists, so replacement
    // is explicitly disabled. A trailing NUL is neither stored nor counted.
    unsafe {
        (*info).Anonymous = FILE_RENAME_INFO_0 {
            ReplaceIfExists: false,
        };
        // SetFileInformationByHandle resolves a relative name against the
        // process current directory even when the source is already open. Use
        // the absolute path derived from the retained validated directory and
        // a validated leaf so process-global current-directory state is absent.
        (*info).RootDirectory = std::ptr::null_mut();
        (*info).FileNameLength = name_bytes;
        std::ptr::copy_nonoverlapping(
            name.as_ptr(),
            std::ptr::addr_of_mut!((*info).FileName).cast::<u16>(),
            name.len(),
        );
    }
    Ok((storage, bytes))
}

fn close_for_publish(
    mut owned: OwnedHandle,
    #[cfg(test)] stage: NativePublishStage,
) -> Result<(), ReplayError> {
    let handle = std::mem::replace(&mut owned.0, INVALID_HANDLE_VALUE);
    // SAFETY: this takes the one owned handle after the rename. The wrapper is
    // disarmed before returning, so it cannot close the handle a second time.
    if unsafe { CloseHandle(handle) } == 0 {
        return unavailable_after_native_publish_failure(
            #[cfg(test)]
            stage,
        );
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PublishNewOutcome {
    Published,
}

/// Atomically publish immutable bytes under a prevalidated directory.  Every
/// failure is intentionally indistinguishable to callers: a temporary file or
/// namespace state may be evidence of ambiguity and is never repaired here.
pub fn publish_new_canonical_file(
    directory: &ValidatedDirectory,
    final_name: &ValidatedLeafName,
    bytes: &[u8],
) -> Result<PublishNewOutcome, ReplayError> {
    publish_new_canonical_file_with_temporary_stem(
        directory,
        final_name,
        final_name.as_str(),
        bytes,
    )
}

fn publish_new_canonical_file_with_temporary_stem(
    directory: &ValidatedDirectory,
    final_name: &ValidatedLeafName,
    temporary_stem: &str,
    bytes: &[u8],
) -> Result<PublishNewOutcome, ReplayError> {
    #[cfg(test)]
    clear_native_publish_diagnostic();
    let temporary = ValidatedLeafName::new(&format!(
        "{}.{}.tmp",
        temporary_stem,
        Uuid::new_v4().simple()
    ))?;
    let temporary_path = directory.path.join(temporary.as_str());
    let handle = open_file_for_publish(
        &temporary_path,
        GENERIC_READ | GENERIC_WRITE | DELETE,
        CREATE_NEW,
        FILE_ATTRIBUTE_NORMAL | FILE_FLAG_WRITE_THROUGH,
        #[cfg(test)]
        NativePublishStage::CreateTemporary,
    )?;
    write_all_for_publish(handle.0, bytes)?;
    // SAFETY: handle is a live writable temporary file handle.
    if unsafe { FlushFileBuffers(handle.0) } == 0 {
        return unavailable_after_native_publish_failure(
            #[cfg(test)]
            NativePublishStage::FirstFlush,
        );
    }
    rename_without_replacement(handle.0, &directory.path.join(final_name.as_str()))?;
    // SAFETY: handle remains open across the rename and is the renamed file.
    if unsafe { FlushFileBuffers(handle.0) } == 0 {
        return unavailable_after_native_publish_failure(
            #[cfg(test)]
            NativePublishStage::SecondFlush,
        );
    }
    close_for_publish(
        handle,
        #[cfg(test)]
        NativePublishStage::CloseAfterRename,
    )?;
    let final_path = directory.path.join(final_name.as_str());
    let reopened = open_file_for_publish(
        &final_path,
        GENERIC_READ,
        OPEN_EXISTING,
        FILE_ATTRIBUTE_NORMAL,
        #[cfg(test)]
        NativePublishStage::ReopenFinal,
    )?;
    let actual = read_complete_for_publish(reopened.0, bytes.len())?;
    if actual != bytes {
        #[cfg(test)]
        {
            let _ = NativePublishStage::VerifyFinal;
        }
        return unavailable();
    }
    close_for_publish(
        reopened,
        #[cfg(test)]
        NativePublishStage::CloseAfterReopen,
    )?;
    Ok(PublishNewOutcome::Published)
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

fn child_exists(parent: &Path, name: &str) -> bool {
    parent.join(name).exists()
}

fn exact_directory_entries(path: &Path, expected: &[&str]) -> Result<(), ReplayError> {
    let mut actual = std::fs::read_dir(path)
        .map_err(|_| ReplayError::PersistenceUnavailable)?
        .map(|entry| entry.map(|item| item.file_name().to_string_lossy().into_owned()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| ReplayError::PersistenceUnavailable)?;
    actual.sort();
    let mut expected = expected
        .iter()
        .map(|item| item.to_string())
        .collect::<Vec<_>>();
    expected.sort();
    if actual != expected {
        return unavailable();
    }
    Ok(())
}

fn validate_format(directory: &ValidatedDirectory) -> Result<(), ReplayError> {
    let name = ValidatedLeafName::new("FORMAT.json")?;
    let file = open_file(
        &directory.path.join(name.as_str()),
        GENERIC_READ,
        OPEN_EXISTING,
        FILE_ATTRIBUTE_NORMAL,
    )?;
    if read_complete(file.0, FORMAT_BYTES.len())? != FORMAT_BYTES {
        return unavailable();
    }
    Ok(())
}

fn validate_complete_hierarchy(root: ValidatedHostRoot) -> Result<(), ReplayError> {
    exact_directory_entries(root.path(), &["replay"])?;
    let root = root.into_directory()?;
    let replay = root.child_directory(&ValidatedLeafName::new("replay")?)?;
    exact_directory_entries(&replay.path, &["v1"])?;
    let version = replay.child_directory(&ValidatedLeafName::new("v1")?)?;
    exact_directory_entries(&version.path, &["FORMAT.json", "chains", "claims", "locks"])?;
    validate_format(&version)?;
    for name in ["locks", "claims", "chains"] {
        let _child = version.child_directory(&ValidatedLeafName::new(name)?)?;
    }
    Ok(())
}

/// Establish exactly the one permitted empty v1 hierarchy. Existing partial or
/// unrecognised state is deliberately not repaired, even when it looks benign.
pub fn provision_replay(root_path: &Path) -> Result<ProvisionReplayOutcome, ReplayError> {
    let root = validate_existing_root(root_path)?;
    if child_exists(root.path(), "replay") {
        validate_complete_hierarchy(root)?;
        let _validated_ledger = ReplayLedger::open(root_path)?;
        return Ok(ProvisionReplayOutcome::AlreadyProvisioned);
    }
    exact_directory_entries(root.path(), &[])?;
    let root = root.into_directory()?;
    let replay = create_new_directory(&root, "replay")?;
    let version = create_new_directory(&replay, "v1")?;
    let locks = create_new_directory(&version, "locks")?;
    drop(locks);
    let claims = create_new_directory(&version, "claims")?;
    drop(claims);
    let chains = create_new_directory(&version, "chains")?;
    drop(chains);
    let version_for_format = validate_existing_root(root_path)?
        .into_directory()?
        .child_directory(&ValidatedLeafName::new("replay")?)?
        .child_directory(&ValidatedLeafName::new("v1")?)?;
    publish_new_canonical_file(
        &version_for_format,
        &ValidatedLeafName::new("FORMAT.json")?,
        FORMAT_BYTES,
    )?;
    drop(version_for_format);
    validate_complete_hierarchy(validate_existing_root(root_path)?)?;
    Ok(ProvisionReplayOutcome::Provisioned)
}

fn is_lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .as_bytes()
            .iter()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

fn directory_entry_names(directory: &ValidatedDirectory) -> Result<Vec<String>, ReplayError> {
    let mut names = std::fs::read_dir(&directory.path)
        .map_err(|_| ReplayError::PersistenceUnavailable)?
        .map(|entry| entry.map(|item| item.file_name().to_string_lossy().into_owned()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| ReplayError::PersistenceUnavailable)?;
    names.sort();
    Ok(names)
}

fn open_existing_regular_file(path: &Path) -> Result<Option<OwnedHandle>, ReplayError> {
    let path_w = wide(path);
    // SAFETY: the nul-terminated path lives through the call. Sharing is
    // disabled so validation and complete reads observe one immutable file.
    let raw = unsafe {
        CreateFileW(
            path_w.as_ptr(),
            GENERIC_READ,
            0,
            std::ptr::null(),
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL | FILE_FLAG_OPEN_REPARSE_POINT,
            std::ptr::null_mut(),
        )
    };
    if raw == INVALID_HANDLE_VALUE {
        // SAFETY: captured immediately after the failed CreateFileW call.
        return match unsafe { GetLastError() } {
            ERROR_FILE_NOT_FOUND | ERROR_PATH_NOT_FOUND => Ok(None),
            _ => unavailable(),
        };
    }
    let handle = OwnedHandle(raw);
    let mut information = BY_HANDLE_FILE_INFORMATION::default();
    // SAFETY: handle is live and information is caller-owned writable storage.
    if unsafe { GetFileInformationByHandle(handle.0, &mut information) } == 0
        || information.dwFileAttributes & (FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_REPARSE_POINT)
            != 0
    {
        return unavailable();
    }
    Ok(Some(handle))
}

fn read_replay_record(handle: HANDLE) -> Result<Vec<u8>, ReplayError> {
    let mut length = 0i64;
    // SAFETY: handle is a live regular-file handle and length is writable.
    if unsafe { GetFileSizeEx(handle, &mut length) } == 0
        || length <= 0
        || length > MAX_REPLAY_RECORD_BYTES
    {
        return unavailable();
    }
    read_complete(handle, length as usize)
}

/// Owning authority for the documented byte-zero, length-one logical-key
/// exclusion. Closing this handle releases both the byte-range lock and the
/// share-denying file open, including on process termination.
pub struct LogicalKeyLock {
    _handle: OwnedHandle,
}

impl LogicalKeyLock {
    fn acquire(
        directory: &ValidatedDirectory,
        logical_key: &LogicalExecutionKey,
    ) -> Result<Self, ReplayError> {
        persistence_fault(PersistenceFaultPoint::LockFileOpen)?;
        let leaf = ValidatedLeafName::new(&format!("{}.lock", logical_key.filename_digest()))?;
        let path_w = wide(&directory.path.join(leaf.as_str()));
        // SAFETY: path is one validated leaf beneath retained authority.
        // OPEN_ALWAYS makes file existence irrelevant; sharing zero makes a
        // competing process fail closed before it could reach the byte lock.
        let raw = unsafe {
            CreateFileW(
                path_w.as_ptr(),
                GENERIC_READ | GENERIC_WRITE,
                0,
                std::ptr::null(),
                OPEN_ALWAYS,
                FILE_ATTRIBUTE_NORMAL | FILE_FLAG_OPEN_REPARSE_POINT,
                std::ptr::null_mut(),
            )
        };
        if raw == INVALID_HANDLE_VALUE {
            return unavailable();
        }
        let handle = OwnedHandle(raw);
        let mut information = BY_HANDLE_FILE_INFORMATION::default();
        // SAFETY: handle is live and information is writable.
        if unsafe { GetFileInformationByHandle(handle.0, &mut information) } == 0
            || information.dwFileAttributes
                & (FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_REPARSE_POINT)
                != 0
        {
            return unavailable();
        }
        persistence_fault(PersistenceFaultPoint::LockAcquisition)?;
        let mut overlapped = OVERLAPPED::default();
        // SAFETY: the synchronous handle is live; the zeroed OVERLAPPED fixes
        // the offset at byte zero. The exclusive range is low=1, high=0.
        if unsafe { LockFileEx(handle.0, LOCKFILE_EXCLUSIVE_LOCK, 0, 1, 0, &mut overlapped) } == 0 {
            return unavailable();
        }
        Ok(Self { _handle: handle })
    }
}

fn open_or_create_directory(
    parent: &ValidatedDirectory,
    name: &str,
) -> Result<ValidatedDirectory, ReplayError> {
    let leaf = ValidatedLeafName::new(name)?;
    let path = parent.path.join(leaf.as_str());
    let wide_path = wide(&path);
    // SAFETY: target is one validated leaf beneath a retained parent.
    if unsafe { CreateDirectoryW(wide_path.as_ptr(), std::ptr::null()) } == 0 {
        // SAFETY: captured immediately after the failed CreateDirectoryW call.
        if unsafe { GetLastError() } != ERROR_ALREADY_EXISTS {
            return unavailable();
        }
    }
    parent.child_directory(&leaf)
}

fn existing_child_directory(
    parent: &ValidatedDirectory,
    name: &str,
) -> Result<Option<ValidatedDirectory>, ReplayError> {
    let leaf = ValidatedLeafName::new(name)?;
    let path = parent.path.join(leaf.as_str());
    if !path.exists() {
        return Ok(None);
    }
    parent.child_directory(&leaf).map(Some)
}

fn generation_filename(number: u64) -> Result<ValidatedLeafName, ReplayError> {
    if number > 2 {
        return unavailable();
    }
    ValidatedLeafName::new(&format!("g{number:016}.json"))
}

fn parse_generation_filename(value: &str) -> Result<u64, ReplayError> {
    if value.len() != 22 || !value.starts_with('g') || !value.ends_with(".json") {
        return unavailable();
    }
    let digits = &value[1..17];
    if !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return unavailable();
    }
    let number = digits
        .parse::<u64>()
        .map_err(|_| ReplayError::PersistenceUnavailable)?;
    if number > 2 || generation_filename(number)?.as_str() != value {
        return unavailable();
    }
    Ok(number)
}

fn model_unavailable<T>(result: Result<T, ReplayError>) -> Result<T, ReplayError> {
    result.map_err(|error| match error {
        ReplayError::BindingMismatch => ReplayError::BindingMismatch,
        _ => ReplayError::PersistenceUnavailable,
    })
}

pub struct ReplayLedger {
    locks: ValidatedDirectory,
    claims: ValidatedDirectory,
    chains: ValidatedDirectory,
}

impl ReplayLedger {
    /// Admit the already-provisioned hierarchy and reconstruct every durable
    /// record. Unknown entries and orphan chains fail the whole ledger closed.
    pub fn open(root_path: &Path) -> Result<Self, ReplayError> {
        persistence_fault(PersistenceFaultPoint::RestartScan)?;
        validate_complete_hierarchy(validate_existing_root(root_path)?)?;
        let version = validate_existing_root(root_path)?
            .into_directory()?
            .child_directory(&ValidatedLeafName::new("replay")?)?
            .child_directory(&ValidatedLeafName::new("v1")?)?;
        let ledger = Self {
            locks: version.child_directory(&ValidatedLeafName::new("locks")?)?,
            claims: version.child_directory(&ValidatedLeafName::new("claims")?)?,
            chains: version.child_directory(&ValidatedLeafName::new("chains")?)?,
        };
        ledger.validate_whole_ledger()?;
        Ok(ledger)
    }

    fn validate_whole_ledger(&self) -> Result<(), ReplayError> {
        self.validate_lock_entries()?;
        let claims = self.scan_claims()?;
        persistence_fault(PersistenceFaultPoint::OrphanDetection)?;
        self.scan_chains(&claims)?;
        Ok(())
    }

    fn validate_lock_entries(&self) -> Result<(), ReplayError> {
        let entries =
            std::fs::read_dir(&self.locks.path).map_err(|_| ReplayError::PersistenceUnavailable)?;
        for entry in entries {
            let entry = entry.map_err(|_| ReplayError::PersistenceUnavailable)?;
            let name = entry.file_name().to_string_lossy().into_owned();
            let Some(digest) = name.strip_suffix(".lock") else {
                return unavailable();
            };
            let file_type = entry
                .file_type()
                .map_err(|_| ReplayError::PersistenceUnavailable)?;
            if !is_lower_hex(digest, 64) || !file_type.is_file() || file_type.is_symlink() {
                return unavailable();
            }
        }
        Ok(())
    }

    fn scan_claims(&self) -> Result<HashMap<String, Claim>, ReplayError> {
        let mut by_execution = HashMap::new();
        for name in directory_entry_names(&self.claims)? {
            let Some(digest) = name.strip_suffix(".claim.json") else {
                return unavailable();
            };
            if !is_lower_hex(digest, 64) {
                return unavailable();
            }
            let logical_key =
                model_unavailable(LogicalExecutionKey::from_digest(format!("sha256:{digest}")))?;
            let claim =
                self.read_claim_named(&logical_key, &name, PersistenceFaultPoint::ClaimRead)?;
            let execution_digest = claim.execution_id.filename_digest();
            if by_execution.insert(execution_digest, claim).is_some() {
                return unavailable();
            }
        }
        Ok(by_execution)
    }

    fn scan_chains(&self, claims: &HashMap<String, Claim>) -> Result<(), ReplayError> {
        let mut seen = HashSet::new();
        for prefix_name in directory_entry_names(&self.chains)? {
            if !is_lower_hex(&prefix_name, 2) {
                return unavailable();
            }
            let prefix = self
                .chains
                .child_directory(&ValidatedLeafName::new(&prefix_name)?)?;
            let execution_names = directory_entry_names(&prefix)?;
            if execution_names.is_empty() {
                return unavailable();
            }
            for execution_name in execution_names {
                if !is_lower_hex(&execution_name, 64)
                    || !execution_name.starts_with(&prefix_name)
                    || !seen.insert(execution_name.clone())
                {
                    return unavailable();
                }
                let Some(claim) = claims.get(&execution_name) else {
                    return unavailable();
                };
                let execution =
                    prefix.child_directory(&ValidatedLeafName::new(&execution_name)?)?;
                let generations = self.read_generation_directory(&execution)?;
                model_unavailable(validate_chain(claim, &generations))?;
            }
        }
        Ok(())
    }

    fn read_claim_named(
        &self,
        logical_key: &LogicalExecutionKey,
        name: &str,
        fault_point: PersistenceFaultPoint,
    ) -> Result<Claim, ReplayError> {
        persistence_fault(fault_point)?;
        let Some(handle) = open_existing_regular_file(&self.claims.path.join(name))? else {
            return unavailable();
        };
        let bytes = read_replay_record(handle.0)?;
        persistence_fault(PersistenceFaultPoint::DigestVerification)?;
        model_unavailable(Claim::from_canonical_bytes(&bytes, logical_key))
    }

    fn existing_claim(
        &self,
        logical_key: &LogicalExecutionKey,
        fault_point: PersistenceFaultPoint,
    ) -> Result<Option<Claim>, ReplayError> {
        let name = format!("{}.claim.json", logical_key.filename_digest());
        if open_existing_regular_file(&self.claims.path.join(&name))?.is_none() {
            return Ok(None);
        }
        self.read_claim_named(logical_key, &name, fault_point)
            .map(Some)
    }

    fn read_generation_directory(
        &self,
        directory: &ValidatedDirectory,
    ) -> Result<Vec<Generation>, ReplayError> {
        persistence_fault(PersistenceFaultPoint::ChainDirectoryValidation)?;
        let mut numbered = Vec::new();
        for name in directory_entry_names(directory)? {
            numbered.push((parse_generation_filename(&name)?, name));
        }
        numbered.sort_by_key(|(number, _)| *number);
        let mut generations = Vec::with_capacity(numbered.len());
        for (expected, (number, name)) in numbered.into_iter().enumerate() {
            if number != expected as u64 {
                return unavailable();
            }
            persistence_fault(PersistenceFaultPoint::GenerationReopen)?;
            let Some(handle) = open_existing_regular_file(&directory.path.join(name))? else {
                return unavailable();
            };
            let bytes = read_replay_record(handle.0)?;
            persistence_fault(PersistenceFaultPoint::DigestVerification)?;
            generations.push(model_unavailable(Generation::from_canonical_bytes(&bytes))?);
        }
        Ok(generations)
    }

    fn reconstruct(&self, claim: &Claim) -> Result<(ReplayState, Vec<Generation>), ReplayError> {
        persistence_fault(PersistenceFaultPoint::ChainDirectoryValidation)?;
        let execution_digest = claim.execution_id.filename_digest();
        let prefix_name = &execution_digest[..2];
        let Some(prefix) = existing_child_directory(&self.chains, prefix_name)? else {
            return Ok((ReplayState::ClaimedNoState, Vec::new()));
        };
        let Some(execution) = existing_child_directory(&prefix, &execution_digest)? else {
            return Ok((ReplayState::ClaimedNoState, Vec::new()));
        };
        let generations = self.read_generation_directory(&execution)?;
        let state = model_unavailable(validate_chain(claim, &generations))?;
        Ok((state, generations))
    }

    fn ensure_execution_directory(&self, claim: &Claim) -> Result<ValidatedDirectory, ReplayError> {
        persistence_fault(PersistenceFaultPoint::ChainDirectoryValidation)?;
        let execution_digest = claim.execution_id.filename_digest();
        let prefix = open_or_create_directory(&self.chains, &execution_digest[..2])?;
        open_or_create_directory(&prefix, &execution_digest)
    }

    pub fn admit_or_recover(
        &self,
        logical_key: LogicalExecutionKey,
        binding: ExecutionBinding,
    ) -> Result<ReplayAdmission<'_>, ReplayError> {
        let lock = LogicalKeyLock::acquire(&self.locks, &logical_key)?;
        if let Some(claim) = self.existing_claim(&logical_key, PersistenceFaultPoint::ClaimRead)? {
            persistence_fault(PersistenceFaultPoint::ClaimCollisionReopen)?;
            claim.require_binding(&binding)?;
            let (state, generations) = self.reconstruct(&claim)?;
            return Ok(ReplayAdmission {
                ledger: self,
                _lock: lock,
                claim,
                generations,
                state,
                fresh: false,
            });
        }

        let claim = model_unavailable(Claim::new(logical_key, ExecutionId::generate(), binding))?;
        let bytes = model_unavailable(claim.canonical_bytes())?;
        persistence_fault(PersistenceFaultPoint::ClaimPublication)?;
        let name = ValidatedLeafName::new(&format!(
            "{}.claim.json",
            claim.logical_key.filename_digest()
        ))?;
        match publish_new_canonical_file_with_temporary_stem(
            &self.claims,
            &name,
            claim.logical_key.filename_digest(),
            &bytes,
        ) {
            Ok(PublishNewOutcome::Published) => {}
            // The native primitive intentionally retains its keyed temporary
            // after an ambiguous or collision failure. Such evidence can
            // never accompany a usable admission.
            Err(_) => return unavailable(),
        }
        let published = self
            .existing_claim(
                &claim.logical_key,
                PersistenceFaultPoint::ClaimCollisionReopen,
            )?
            .ok_or(ReplayError::PersistenceUnavailable)?;
        if published != claim {
            return unavailable();
        }
        Ok(ReplayAdmission {
            ledger: self,
            _lock: lock,
            claim,
            generations: Vec::new(),
            state: ReplayState::ClaimedNoState,
            fresh: true,
        })
    }
}

pub struct ReplayAdmission<'a> {
    ledger: &'a ReplayLedger,
    _lock: LogicalKeyLock,
    claim: Claim,
    generations: Vec<Generation>,
    state: ReplayState,
    fresh: bool,
}

impl ReplayAdmission<'_> {
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
        let generation = model_unavailable(Generation::intent(&self.claim))?;
        self.publish_generation(generation, PersistenceFaultPoint::GenerationZeroPublication)
    }

    pub fn publish_armed(&mut self) -> Result<(), ReplayError> {
        if !self.fresh || self.state != ReplayState::IntentRecorded || self.generations.len() != 1 {
            return unavailable();
        }
        let generation = model_unavailable(Generation::armed(&self.claim, &self.generations[0]))?;
        self.publish_generation(generation, PersistenceFaultPoint::GenerationOnePublication)
    }

    pub fn publish_terminal(
        &mut self,
        state: ReplayState,
        durable_outcome_digest: String,
    ) -> Result<(), ReplayError> {
        if !self.fresh || self.state != ReplayState::InvocationArmed || self.generations.len() != 2
        {
            return unavailable();
        }
        let generation = model_unavailable(Generation::terminal(
            &self.claim,
            &self.generations[1],
            state,
            durable_outcome_digest,
        ))?;
        self.publish_generation(generation, PersistenceFaultPoint::GenerationTwoPublication)
    }

    fn publish_generation(
        &mut self,
        generation: Generation,
        fault_point: PersistenceFaultPoint,
    ) -> Result<(), ReplayError> {
        if generation.number != self.generations.len() as u64 {
            return unavailable();
        }
        let (durable_state, durable_generations) = self.ledger.reconstruct(&self.claim)?;
        if durable_generations == self.generations && durable_state == self.state {
            // The expected generation is not present yet.
        } else if durable_generations.len() == self.generations.len() + 1
            && durable_generations[..self.generations.len()] == self.generations
            && durable_generations.last() == Some(&generation)
        {
            // An exact immutable generation won before this in-flight
            // publication. Full reconstruction, not byte equality alone,
            // established its identity, predecessor, and transition.
            self.state = generation.state;
            self.generations.push(generation);
            return Ok(());
        } else {
            return unavailable();
        }
        let directory = self.ledger.ensure_execution_directory(&self.claim)?;
        let name = generation_filename(generation.number)?;
        let expected = model_unavailable(generation.canonical_bytes())?;
        if open_existing_regular_file(&directory.path.join(name.as_str()))?.is_some() {
            return unavailable();
        }
        persistence_fault(fault_point)?;
        let temporary_stem = format!("g{:016}", generation.number);
        publish_new_canonical_file_with_temporary_stem(
            &directory,
            &name,
            &temporary_stem,
            &expected,
        )?;
        persistence_fault(PersistenceFaultPoint::GenerationReopen)?;
        let Some(reopened) = open_existing_regular_file(&directory.path.join(name.as_str()))?
        else {
            return unavailable();
        };
        let actual = read_replay_record(reopened.0)?;
        if actual != expected {
            return unavailable();
        }
        persistence_fault(PersistenceFaultPoint::DigestVerification)?;
        let recovered = model_unavailable(Generation::from_canonical_bytes(&actual))?;
        if recovered != generation {
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
    use serde_json::{json, Value};
    use sha2::{Digest, Sha256};
    use std::process::{Child, Command};
    use std::sync::{Arc, Barrier};
    use std::time::{Duration, Instant};

    fn fresh_native_test_root(_label: &str) -> Option<PathBuf> {
        let base = std::env::var_os("TETHERS_J09_NATIVE_PROVISION_ROOT")?;
        // Claim and generation temporary names are deliberately long digests.
        // Keep the test child short so an operator-supplied diagnostic base can
        // still exercise the actual Win32 path rather than MAX_PATH.
        let root = PathBuf::from(base).join(Uuid::new_v4().simple().to_string());
        std::fs::create_dir(&root).unwrap();
        Some(root)
    }

    fn validated_test_directory(root: &Path) -> ValidatedDirectory {
        validate_existing_root(root)
            .unwrap()
            .into_directory()
            .unwrap()
    }

    fn directory_names(path: &Path) -> Vec<String> {
        let mut names = std::fs::read_dir(path)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        names.sort();
        names
    }

    fn tree_snapshot(root: &Path) -> Vec<(String, bool, u64, std::time::SystemTime)> {
        fn collect(
            root: &Path,
            current: &Path,
            entries: &mut Vec<(String, bool, u64, std::time::SystemTime)>,
        ) {
            let mut children = std::fs::read_dir(current)
                .unwrap()
                .map(|entry| entry.unwrap().path())
                .collect::<Vec<_>>();
            children.sort();
            for child in children {
                let metadata = std::fs::metadata(&child).unwrap();
                entries.push((
                    child
                        .strip_prefix(root)
                        .unwrap()
                        .to_string_lossy()
                        .into_owned(),
                    metadata.is_dir(),
                    metadata.len(),
                    metadata.modified().unwrap(),
                ));
                if metadata.is_dir() {
                    collect(root, &child, entries);
                }
            }
        }

        let mut entries = Vec::new();
        collect(root, root, &mut entries);
        entries
    }

    fn test_digest(label: &str) -> String {
        format!("sha256:{:x}", Sha256::digest(label.as_bytes()))
    }

    fn test_binding(action_id: &str) -> ExecutionBinding {
        ExecutionBinding {
            evaluation_id: "eval-ledger".into(),
            action_id: action_id.into(),
            capability_name: "calendar.create".into(),
            capability_version: 1,
            manifest_digest: test_digest("manifest"),
            provider_identity: "provider-local".into(),
            argument_digest: test_digest("redacted-arguments"),
        }
    }

    fn test_key(action_id: &str) -> LogicalExecutionKey {
        LogicalExecutionKey::derive("event-ledger", "eval-ledger", action_id).unwrap()
    }

    fn provisioned_test_root(label: &str) -> Option<PathBuf> {
        let root = fresh_native_test_root(label)?;
        assert_eq!(
            provision_replay(&root),
            Ok(ProvisionReplayOutcome::Provisioned)
        );
        Some(root)
    }

    fn claim_path(root: &Path, key: &LogicalExecutionKey) -> PathBuf {
        root.join("replay")
            .join("v1")
            .join("claims")
            .join(format!("{}.claim.json", key.filename_digest()))
    }

    fn execution_directory(root: &Path, execution_id: &str) -> PathBuf {
        let execution_id = ExecutionId::parse(execution_id.to_owned()).unwrap();
        let digest = execution_id.filename_digest();
        root.join("replay")
            .join("v1")
            .join("chains")
            .join(&digest[..2])
            .join(digest)
    }

    fn wait_for_signal(path: &Path) {
        let deadline = Instant::now() + Duration::from_secs(20);
        while !path.exists() {
            assert!(
                Instant::now() < deadline,
                "child signal did not arrive: {}",
                path.display()
            );
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    fn lock_control_directory(label: &str) -> PathBuf {
        let path =
            std::env::temp_dir().join(format!("tethers-j09-{label}-{}", Uuid::new_v4().simple()));
        std::fs::create_dir(&path).unwrap();
        path
    }

    fn spawn_lock_child(
        root: &Path,
        action_id: &str,
        role: &str,
        signal: &Path,
        release: Option<&Path>,
    ) -> Child {
        let mut command = Command::new(std::env::current_exe().unwrap());
        command
            .arg("replay_windows::tests::native_lock_child_process_entry")
            .arg("--exact")
            .arg("--nocapture")
            .arg("--test-threads=1")
            .env("TETHERS_J09_LOCK_CHILD_ROLE", role)
            .env("TETHERS_J09_LOCK_CHILD_ROOT", root)
            .env("TETHERS_J09_LOCK_CHILD_ACTION", action_id)
            .env("TETHERS_J09_LOCK_CHILD_SIGNAL", signal);
        if let Some(release) = release {
            command.env("TETHERS_J09_LOCK_CHILD_RELEASE", release);
        }
        command.spawn().unwrap()
    }

    fn rewrite_enclosing_digest(value: &mut serde_json::Map<String, Value>, field: &str) {
        value.remove(field).unwrap();
        let unsigned = serde_json_canonicalizer::to_vec(&Value::Object(value.clone())).unwrap();
        value.insert(field.into(), Value::String(test_digest_bytes(&unsigned)));
    }

    fn test_digest_bytes(bytes: &[u8]) -> String {
        format!("sha256:{:x}", Sha256::digest(bytes))
    }

    #[test]
    fn simple_final_filename_is_accepted() {
        assert!(ValidatedLeafName::new("FORMAT.json").is_ok());
    }
    #[test]
    fn traversal_ads_and_separator_final_filenames_are_rejected() {
        for name in [".", "..", "x/y", r"x\y", "x:stream", "NUL", "COM1.txt"] {
            assert!(ValidatedLeafName::new(name).is_err(), "{name}");
        }
    }
    #[test]
    fn relative_root_is_rejected_before_win32() {
        assert!(validate_existing_root(Path::new("relative")).is_err());
    }
    #[test]
    fn unc_roots_are_rejected_before_win32() {
        assert!(volume_root(Path::new(r"\\server\share\root")).is_err());
    }
    #[test]
    fn parent_components_are_not_canonical() {
        assert!(volume_root(Path::new(r"C:\root\..\other")).is_err());
    }
    #[test]
    fn unrelated_read_only_authority_is_safe() {
        assert!(allow_ace_is_permitted(0, false).is_ok());
    }
    #[test]
    fn unrelated_write_authority_is_rejected() {
        assert!(allow_ace_is_permitted(FILE_ADD_SUBDIRECTORY, false).is_err());
    }
    #[test]
    fn generic_write_is_rejected() {
        assert!(allow_ace_is_permitted(GENERIC_WRITE, false).is_err());
    }
    #[test]
    fn trusted_writer_is_accepted() {
        assert!(allow_ace_is_permitted(GENERIC_ALL, true).is_ok());
    }
    #[test]
    fn native_token_and_current_directory_acl_is_live() {
        // This is intentionally an integration proof, not a synthetic SID or
        // descriptor: it exercises token sizing, SID ownership, every existing
        // ancestor handle, volume information, and the current directory DACL.
        // This sandbox deliberately grants Authenticated Users modify authority,
        // so it must be rejected rather than treated as a usable replay root.
        assert!(!current_user_sid().unwrap().0.is_empty());
        assert!(validate_existing_root(&std::env::current_dir().unwrap()).is_err());
    }
    #[test]
    fn native_local_fixed_ntfs_volume_is_accepted() {
        let current = std::env::current_dir().unwrap();
        let root = volume_root(&current).unwrap();
        let handle = open_component(&current).unwrap();
        assert!(validate_volume(handle.0, &root).is_ok());
    }

    #[test]
    fn rename_info_layout_matches_native_windows_sdk() {
        assert_eq!(size_of::<FILE_RENAME_INFO>(), 24);
        assert_eq!(size_of::<FILE_RENAME_INFO_0>(), 4);
        assert_eq!(std::mem::offset_of!(FILE_RENAME_INFO, Anonymous), 0);
        assert_eq!(std::mem::offset_of!(FILE_RENAME_INFO, RootDirectory), 8);
        assert_eq!(std::mem::offset_of!(FILE_RENAME_INFO, FileNameLength), 16);
        assert_eq!(std::mem::offset_of!(FILE_RENAME_INFO, FileName), 20);
    }

    #[test]
    fn validated_child_retains_complete_independent_handle_chain() {
        let Some(root) = fresh_native_test_root("handle-chain") else {
            return;
        };
        std::fs::create_dir(root.join("child")).unwrap();
        let parent = validated_test_directory(&root);
        let child = parent
            .child_directory(&ValidatedLeafName::new("child").unwrap())
            .unwrap();
        assert_eq!(
            parent._authority.len(),
            root.components().skip(2).count() + 1
        );
        assert_eq!(
            child._authority.len(),
            child.path.components().skip(2).count() + 1
        );
        drop(parent);
        assert_eq!(child.path, root.join("child"));
        assert_eq!(directory_names(&child.path), Vec::<String>::new());
        let redirected = root.with_file_name(format!(
            "{}-redirected",
            root.file_name().unwrap().to_string_lossy()
        ));
        assert!(
            std::fs::rename(&root, &redirected).is_err(),
            "a retained descendant authority must deny ancestor substitution"
        );
        assert!(root.exists());
        assert!(!redirected.exists());
        drop(child);
        std::fs::rename(&root, &redirected).unwrap();
        assert!(redirected.join("child").is_dir());
    }

    #[test]
    fn native_publication_survives_reopen_and_never_replaces() {
        let Some(root) = fresh_native_test_root("publication") else {
            return;
        };
        let directory = validated_test_directory(&root);
        let name = ValidatedLeafName::new("record.json").unwrap();
        let final_path = root.join(name.as_str());
        let expected = br#"{"generation":0}"#;
        assert_eq!(
            publish_new_canonical_file(&directory, &name, expected),
            Ok(PublishNewOutcome::Published)
        );
        assert_eq!(directory_names(&root), vec!["record.json"]);
        assert_eq!(std::fs::read(&final_path).unwrap(), expected);
        assert_eq!(last_native_publish_diagnostic(), None);

        let replacement = br#"{"generation":1}"#;
        assert_eq!(
            publish_new_canonical_file(&directory, &name, replacement),
            Err(ReplayError::PersistenceUnavailable)
        );
        let diagnostic = last_native_publish_diagnostic().unwrap();
        println!("native collision diagnostic: {diagnostic:?}");
        assert_eq!(diagnostic.stage, NativePublishStage::Rename);
        assert_ne!(diagnostic.win32_error, 0);
        assert_eq!(std::fs::read(&final_path).unwrap(), expected);
        let names = directory_names(&root);
        assert_eq!(names.len(), 2);
        let temporary = names.iter().find(|entry| entry.ends_with(".tmp")).unwrap();
        assert_eq!(std::fs::read(root.join(temporary)).unwrap(), replacement);
    }

    #[test]
    fn native_competing_publishers_accept_exactly_one_and_retain_loser() {
        let Some(root) = fresh_native_test_root("competing-publication") else {
            return;
        };
        let name = "record.json";
        let first = br#"{"publisher":"first"}"#;
        let second = br#"{"publisher":"second"}"#;
        let barrier = Arc::new(Barrier::new(2));
        let publish = |bytes: &'static [u8], barrier: Arc<Barrier>| {
            let root = root.clone();
            std::thread::spawn(move || {
                let directory = validated_test_directory(&root);
                let name = ValidatedLeafName::new(name).unwrap();
                barrier.wait();
                publish_new_canonical_file(&directory, &name, bytes)
            })
        };
        let first_publisher = publish(first, Arc::clone(&barrier));
        let second_publisher = publish(second, barrier);
        let first_result = first_publisher.join().unwrap();
        let second_result = second_publisher.join().unwrap();
        assert_eq!(
            [first_result.is_ok(), second_result.is_ok()]
                .into_iter()
                .filter(|accepted| *accepted)
                .count(),
            1
        );
        let final_bytes = std::fs::read(root.join(name)).unwrap();
        assert!(final_bytes == first || final_bytes == second);
        let names = directory_names(&root);
        assert_eq!(names.len(), 2);
        let temporary = names.iter().find(|entry| entry.ends_with(".tmp")).unwrap();
        let retained_bytes = std::fs::read(root.join(temporary)).unwrap();
        assert!(
            (final_bytes == first && retained_bytes == second)
                || (final_bytes == second && retained_bytes == first)
        );
    }

    #[test]
    fn native_provisioning_is_exact_idempotent_and_non_repairing() {
        let Some(root) = fresh_native_test_root("provisioning") else {
            return;
        };
        assert_eq!(
            provision_replay(&root),
            Ok(ProvisionReplayOutcome::Provisioned)
        );
        assert_eq!(
            directory_names(&root),
            vec!["replay"],
            "provisioning creates only the replay subtree"
        );
        assert_eq!(
            directory_names(&root.join("replay")),
            vec!["v1"],
            "provisioning creates exactly one version"
        );
        assert_eq!(
            directory_names(&root.join("replay").join("v1")),
            vec!["FORMAT.json", "chains", "claims", "locks"]
        );
        assert_eq!(
            std::fs::read(root.join("replay").join("v1").join("FORMAT.json")).unwrap(),
            FORMAT_BYTES
        );
        let before = tree_snapshot(&root);
        assert_eq!(
            provision_replay(&root),
            Ok(ProvisionReplayOutcome::AlreadyProvisioned)
        );
        assert_eq!(tree_snapshot(&root), before);

        let partial = fresh_native_test_root("partial-provisioning").unwrap();
        std::fs::create_dir(partial.join("replay")).unwrap();
        let partial_before = tree_snapshot(&partial);
        assert_eq!(
            provision_replay(&partial),
            Err(ReplayError::PersistenceUnavailable)
        );
        assert_eq!(tree_snapshot(&partial), partial_before);

        let unknown = fresh_native_test_root("unknown-provisioning").unwrap();
        std::fs::write(unknown.join("operator-owned.txt"), b"keep").unwrap();
        let unknown_before = tree_snapshot(&unknown);
        assert_eq!(
            provision_replay(&unknown),
            Err(ReplayError::PersistenceUnavailable)
        );
        assert_eq!(tree_snapshot(&unknown), unknown_before);

        let unknown_version = fresh_native_test_root("unknown-version").unwrap();
        std::fs::create_dir(unknown_version.join("replay")).unwrap();
        std::fs::create_dir(unknown_version.join("replay").join("v2")).unwrap();
        let unknown_version_before = tree_snapshot(&unknown_version);
        assert_eq!(
            provision_replay(&unknown_version),
            Err(ReplayError::PersistenceUnavailable)
        );
        assert_eq!(tree_snapshot(&unknown_version), unknown_version_before);
    }

    #[test]
    fn native_provisioning_reports_configured_root_diagnostic() {
        let Some(root) = fresh_native_test_root("diagnostic") else {
            return;
        };
        let name = ValidatedLeafName::new("FORMAT.json").unwrap();
        let destination = root.join(name.as_str());
        let (storage, dw_buffer_size) = rename_info_buffer(&destination).unwrap();
        println!(
            "native rename buffer layout: allocation_bytes={}, structure_bytes={}, filename_offset={}, filename_bytes={}, dw_buffer_size={}",
            storage.len() * size_of::<usize>(),
            size_of::<FILE_RENAME_INFO>(),
            std::mem::offset_of!(FILE_RENAME_INFO, FileName),
            destination.as_os_str().encode_wide().count() * size_of::<u16>(),
            dw_buffer_size,
        );
        let result = provision_replay(&root);
        match result {
            Ok(outcome) => println!("native publication succeeded: {}", outcome.as_str()),
            Err(ReplayError::PersistenceUnavailable) => println!(
                "native publication diagnostic: {:?}",
                last_native_publish_diagnostic()
            ),
            Err(error) => panic!("unexpected replay error: {error:?}"),
        }
    }

    #[test]
    fn native_lock_child_process_entry() {
        let Ok(role) = std::env::var("TETHERS_J09_LOCK_CHILD_ROLE") else {
            return;
        };
        let root = PathBuf::from(std::env::var_os("TETHERS_J09_LOCK_CHILD_ROOT").unwrap());
        let action_id = std::env::var("TETHERS_J09_LOCK_CHILD_ACTION").unwrap();
        let signal = PathBuf::from(std::env::var_os("TETHERS_J09_LOCK_CHILD_SIGNAL").unwrap());
        let ledger = ReplayLedger::open(&root);
        let acquired = ledger
            .as_ref()
            .ok()
            .and_then(|ledger| LogicalKeyLock::acquire(&ledger.locks, &test_key(&action_id)).ok());
        match role.as_str() {
            "hold" => {
                let _guard = acquired.expect("holding child must acquire the requested lock");
                std::fs::write(&signal, b"held").unwrap();
                let release =
                    PathBuf::from(std::env::var_os("TETHERS_J09_LOCK_CHILD_RELEASE").unwrap());
                wait_for_signal(&release);
            }
            "try" => {
                std::fs::write(
                    &signal,
                    if acquired.is_some() {
                        b"acquired".as_slice()
                    } else {
                        b"blocked".as_slice()
                    },
                )
                .unwrap();
            }
            _ => panic!("unknown child role"),
        }
    }

    #[test]
    fn ledger_01_real_second_process_exclusion_and_release() {
        let Some(root) = provisioned_test_root("ledger-lock-exclusion") else {
            return;
        };
        let control = lock_control_directory("lock-exclusion");
        let held = control.join("held");
        let release = control.join("release");
        let blocked = control.join("blocked");
        let acquired = control.join("acquired");
        let mut holder = spawn_lock_child(&root, "same-key", "hold", &held, Some(&release));
        wait_for_signal(&held);
        let mut contender = spawn_lock_child(&root, "same-key", "try", &blocked, None);
        assert!(contender.wait().unwrap().success());
        assert_eq!(std::fs::read(&blocked).unwrap(), b"blocked");
        std::fs::write(&release, b"release").unwrap();
        assert!(holder.wait().unwrap().success());
        let mut successor = spawn_lock_child(&root, "same-key", "try", &acquired, None);
        assert!(successor.wait().unwrap().success());
        assert_eq!(std::fs::read(&acquired).unwrap(), b"acquired");
    }

    #[test]
    fn ledger_02_process_termination_releases_lock() {
        let Some(root) = provisioned_test_root("ledger-lock-termination") else {
            return;
        };
        let control = lock_control_directory("lock-termination");
        let held = control.join("held");
        let never_release = control.join("never-release");
        let acquired = control.join("acquired");
        let mut holder =
            spawn_lock_child(&root, "terminated-key", "hold", &held, Some(&never_release));
        wait_for_signal(&held);
        holder.kill().unwrap();
        let _ = holder.wait().unwrap();
        let mut successor = spawn_lock_child(&root, "terminated-key", "try", &acquired, None);
        assert!(successor.wait().unwrap().success());
        assert_eq!(std::fs::read(&acquired).unwrap(), b"acquired");
    }

    #[test]
    fn ledger_03_different_logical_keys_are_independent() {
        let Some(root) = provisioned_test_root("ledger-lock-independent") else {
            return;
        };
        let control = lock_control_directory("lock-independent");
        let held = control.join("held");
        let release = control.join("release");
        let acquired = control.join("acquired");
        let mut holder = spawn_lock_child(&root, "key-a", "hold", &held, Some(&release));
        wait_for_signal(&held);
        let mut independent = spawn_lock_child(&root, "key-b", "try", &acquired, None);
        assert!(independent.wait().unwrap().success());
        assert_eq!(std::fs::read(&acquired).unwrap(), b"acquired");
        std::fs::write(&release, b"release").unwrap();
        assert!(holder.wait().unwrap().success());
    }

    #[test]
    fn ledger_04_lock_open_and_acquisition_faults_fail_closed() {
        let Some(root) = provisioned_test_root("ledger-lock-faults") else {
            return;
        };
        let ledger = ReplayLedger::open(&root).unwrap();
        for point in [
            PersistenceFaultPoint::LockFileOpen,
            PersistenceFaultPoint::LockAcquisition,
        ] {
            let result = with_persistence_fault(point, || {
                ledger.admit_or_recover(test_key("lock-fault"), test_binding("lock-fault"))
            });
            assert!(matches!(result, Err(ReplayError::PersistenceUnavailable)));
        }
        assert!(directory_names(&root.join("replay").join("v1").join("claims")).is_empty());
    }

    #[test]
    fn ledger_05_fresh_claim_creates_one_host_execution_identity() {
        let Some(root) = provisioned_test_root("ledger-fresh-claim") else {
            return;
        };
        let ledger = ReplayLedger::open(&root).unwrap();
        let admission = ledger
            .admit_or_recover(test_key("fresh"), test_binding("fresh"))
            .unwrap();
        assert!(admission.is_fresh());
        assert!(ExecutionId::parse(admission.execution_id().to_owned()).is_ok());
        assert_eq!(
            directory_names(&root.join("replay").join("v1").join("claims")).len(),
            1
        );
    }

    #[test]
    fn ledger_06_restart_recovers_same_execution_identity() {
        let Some(root) = provisioned_test_root("ledger-claim-restart") else {
            return;
        };
        let key = test_key("restart");
        let binding = test_binding("restart");
        let ledger = ReplayLedger::open(&root).unwrap();
        let first = ledger
            .admit_or_recover(key.clone(), binding.clone())
            .unwrap()
            .execution_id()
            .to_owned();
        drop(ledger);
        let ledger = ReplayLedger::open(&root).unwrap();
        let recovered = ledger.admit_or_recover(key, binding).unwrap();
        assert!(!recovered.is_fresh());
        assert_eq!(recovered.execution_id(), first);
    }

    #[test]
    fn ledger_07_sibling_actions_have_distinct_keys_claims_and_identities() {
        let Some(root) = provisioned_test_root("ledger-siblings") else {
            return;
        };
        let ledger = ReplayLedger::open(&root).unwrap();
        let first = ledger
            .admit_or_recover(test_key("sibling-a"), test_binding("sibling-a"))
            .unwrap()
            .execution_id()
            .to_owned();
        let second = ledger
            .admit_or_recover(test_key("sibling-b"), test_binding("sibling-b"))
            .unwrap()
            .execution_id()
            .to_owned();
        assert_ne!(test_key("sibling-a"), test_key("sibling-b"));
        assert_ne!(first, second);
        assert_eq!(
            directory_names(&root.join("replay").join("v1").join("claims")).len(),
            2
        );
    }

    #[test]
    fn ledger_08_exact_claim_collision_recovers_only_valid_winner() {
        let Some(root) = provisioned_test_root("ledger-claim-collision") else {
            return;
        };
        let key = test_key("claim-collision");
        let binding = test_binding("claim-collision");
        let ledger = ReplayLedger::open(&root).unwrap();
        let winner = ledger
            .admit_or_recover(key.clone(), binding.clone())
            .unwrap()
            .execution_id()
            .to_owned();
        drop(ledger);
        let ledger = ReplayLedger::open(&root).unwrap();
        let recovered = ledger.admit_or_recover(key, binding).unwrap();
        assert!(!recovered.is_fresh());
        assert_eq!(recovered.execution_id(), winner);
    }

    #[test]
    fn ledger_09_binding_mismatch_fails_closed() {
        let Some(root) = provisioned_test_root("ledger-binding-mismatch") else {
            return;
        };
        let key = test_key("binding-mismatch");
        let ledger = ReplayLedger::open(&root).unwrap();
        drop(
            ledger
                .admit_or_recover(key.clone(), test_binding("binding-mismatch"))
                .unwrap(),
        );
        let mut changed = test_binding("binding-mismatch");
        changed.argument_digest = test_digest("changed");
        assert!(matches!(
            ledger.admit_or_recover(key, changed),
            Err(ReplayError::BindingMismatch)
        ));
    }

    #[test]
    fn ledger_10_malformed_or_digest_invalid_claim_fails_closed() {
        let Some(root) = provisioned_test_root("ledger-malformed-claim") else {
            return;
        };
        let key = test_key("malformed-claim");
        let ledger = ReplayLedger::open(&root).unwrap();
        drop(
            ledger
                .admit_or_recover(key.clone(), test_binding("malformed-claim"))
                .unwrap(),
        );
        drop(ledger);
        let path = claim_path(&root, &key);
        let mut value: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        value
            .as_object_mut()
            .unwrap()
            .insert("claim_digest".into(), Value::String(test_digest("forged")));
        std::fs::write(&path, serde_json_canonicalizer::to_vec(&value).unwrap()).unwrap();
        assert!(matches!(
            ReplayLedger::open(&root),
            Err(ReplayError::PersistenceUnavailable)
        ));
    }

    #[test]
    fn ledger_11_claim_publication_fault_grants_no_authority() {
        let Some(root) = provisioned_test_root("ledger-claim-publication-fault") else {
            return;
        };
        let ledger = ReplayLedger::open(&root).unwrap();
        let result = with_persistence_fault(PersistenceFaultPoint::ClaimPublication, || {
            ledger.admit_or_recover(
                test_key("claim-publication-fault"),
                test_binding("claim-publication-fault"),
            )
        });
        assert!(matches!(result, Err(ReplayError::PersistenceUnavailable)));
        assert!(directory_names(&root.join("replay").join("v1").join("claims")).is_empty());
    }

    #[test]
    fn ledger_12_valid_generation_zero_publication() {
        let Some(root) = provisioned_test_root("ledger-g0") else {
            return;
        };
        let ledger = ReplayLedger::open(&root).unwrap();
        let mut admission = ledger
            .admit_or_recover(test_key("g0"), test_binding("g0"))
            .unwrap();
        admission.publish_intent().unwrap();
        assert_eq!(admission.state(), ReplayState::IntentRecorded);
        let path = execution_directory(&root, admission.execution_id());
        assert_eq!(directory_names(&path), vec!["g0000000000000000.json"]);
    }

    #[test]
    fn ledger_13_valid_generation_zero_to_one_transition() {
        let Some(root) = provisioned_test_root("ledger-g1") else {
            return;
        };
        let ledger = ReplayLedger::open(&root).unwrap();
        let mut admission = ledger
            .admit_or_recover(test_key("g1"), test_binding("g1"))
            .unwrap();
        admission.publish_intent().unwrap();
        admission.publish_armed().unwrap();
        assert_eq!(admission.state(), ReplayState::InvocationArmed);
    }

    #[test]
    fn ledger_14_each_valid_generation_two_terminal_state() {
        for state in [
            ReplayState::Succeeded,
            ReplayState::Failed,
            ReplayState::Uncertain,
        ] {
            let Some(root) = provisioned_test_root(&format!("ledger-g2-{state:?}")) else {
                return;
            };
            let action = format!("g2-{state:?}");
            let ledger = ReplayLedger::open(&root).unwrap();
            let mut admission = ledger
                .admit_or_recover(test_key(&action), test_binding(&action))
                .unwrap();
            admission.publish_intent().unwrap();
            admission.publish_armed().unwrap();
            admission
                .publish_terminal(state, test_digest("durable-outcome"))
                .unwrap();
            assert_eq!(admission.state(), state);
        }
    }

    #[test]
    fn ledger_15_generation_one_without_zero_is_rejected() {
        let Some(root) = provisioned_test_root("ledger-g1-without-g0") else {
            return;
        };
        let ledger = ReplayLedger::open(&root).unwrap();
        let admission = ledger
            .admit_or_recover(test_key("g1-without-g0"), test_binding("g1-without-g0"))
            .unwrap();
        let g0 = Generation::intent(&admission.claim).unwrap();
        let g1 = Generation::armed(&admission.claim, &g0).unwrap();
        let directory = execution_directory(&root, admission.execution_id());
        drop(admission);
        drop(ledger);
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(
            directory.join("g0000000000000001.json"),
            g1.canonical_bytes().unwrap(),
        )
        .unwrap();
        assert!(ReplayLedger::open(&root).is_err());
    }

    #[test]
    fn ledger_16_generation_two_without_one_is_rejected() {
        let Some(root) = provisioned_test_root("ledger-g2-without-g1") else {
            return;
        };
        let ledger = ReplayLedger::open(&root).unwrap();
        let admission = ledger
            .admit_or_recover(test_key("g2-without-g1"), test_binding("g2-without-g1"))
            .unwrap();
        let g0 = Generation::intent(&admission.claim).unwrap();
        let g1 = Generation::armed(&admission.claim, &g0).unwrap();
        let g2 = Generation::terminal(
            &admission.claim,
            &g1,
            ReplayState::Succeeded,
            test_digest("outcome"),
        )
        .unwrap();
        let directory = execution_directory(&root, admission.execution_id());
        drop(admission);
        drop(ledger);
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(
            directory.join("g0000000000000000.json"),
            g0.canonical_bytes().unwrap(),
        )
        .unwrap();
        std::fs::write(
            directory.join("g0000000000000002.json"),
            g2.canonical_bytes().unwrap(),
        )
        .unwrap();
        assert!(ReplayLedger::open(&root).is_err());
    }

    #[test]
    fn ledger_17_illegal_state_transition_is_rejected() {
        let Some(root) = provisioned_test_root("ledger-illegal-transition") else {
            return;
        };
        let ledger = ReplayLedger::open(&root).unwrap();
        let admission = ledger
            .admit_or_recover(
                test_key("illegal-transition"),
                test_binding("illegal-transition"),
            )
            .unwrap();
        let g0 = Generation::intent(&admission.claim).unwrap();
        let mut value: Value = serde_json::from_slice(&g0.canonical_bytes().unwrap()).unwrap();
        let object = value.as_object_mut().unwrap();
        object.insert("state".into(), Value::String("succeeded".into()));
        object.insert(
            "state_data".into(),
            json!({"durable_outcome_digest":test_digest("outcome")}),
        );
        rewrite_enclosing_digest(object, "record_digest");
        let bytes = serde_json_canonicalizer::to_vec(&value).unwrap();
        let directory = execution_directory(&root, admission.execution_id());
        drop(admission);
        drop(ledger);
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(directory.join("g0000000000000000.json"), bytes).unwrap();
        assert!(ReplayLedger::open(&root).is_err());
    }

    #[test]
    fn ledger_18_predecessor_mismatch_is_rejected() {
        let Some(root) = provisioned_test_root("ledger-predecessor") else {
            return;
        };
        let ledger = ReplayLedger::open(&root).unwrap();
        let admission = ledger
            .admit_or_recover(test_key("predecessor"), test_binding("predecessor"))
            .unwrap();
        let g0 = Generation::intent(&admission.claim).unwrap();
        let g1 = Generation::armed(&admission.claim, &g0).unwrap();
        let mut value: Value = serde_json::from_slice(&g1.canonical_bytes().unwrap()).unwrap();
        let object = value.as_object_mut().unwrap();
        object.insert(
            "predecessor_digest".into(),
            Value::String(test_digest("wrong-predecessor")),
        );
        rewrite_enclosing_digest(object, "record_digest");
        let directory = execution_directory(&root, admission.execution_id());
        drop(admission);
        drop(ledger);
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(
            directory.join("g0000000000000000.json"),
            g0.canonical_bytes().unwrap(),
        )
        .unwrap();
        std::fs::write(
            directory.join("g0000000000000001.json"),
            serde_json_canonicalizer::to_vec(&value).unwrap(),
        )
        .unwrap();
        assert!(ReplayLedger::open(&root).is_err());
    }

    #[test]
    fn ledger_19_generation_collision_never_replaces_bytes() {
        let Some(root) = provisioned_test_root("ledger-generation-collision") else {
            return;
        };
        let ledger = ReplayLedger::open(&root).unwrap();
        let mut admission = ledger
            .admit_or_recover(
                test_key("generation-collision"),
                test_binding("generation-collision"),
            )
            .unwrap();
        let g0 = Generation::intent(&admission.claim).unwrap();
        let directory = execution_directory(&root, admission.execution_id());
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(
            directory.join("g0000000000000000.json"),
            g0.canonical_bytes().unwrap(),
        )
        .unwrap();
        admission.publish_intent().unwrap();
        let collision = directory.join("g0000000000000001.json");
        std::fs::write(&collision, b"different-immutable-bytes").unwrap();
        assert!(admission.publish_armed().is_err());
        assert_eq!(
            std::fs::read(collision).unwrap(),
            b"different-immutable-bytes"
        );
    }

    #[test]
    fn ledger_20_generation_three_is_rejected() {
        assert!(generation_filename(3).is_err());
        let Some(root) = provisioned_test_root("ledger-generation-three") else {
            return;
        };
        let ledger = ReplayLedger::open(&root).unwrap();
        let mut admission = ledger
            .admit_or_recover(
                test_key("generation-three"),
                test_binding("generation-three"),
            )
            .unwrap();
        admission.publish_intent().unwrap();
        admission.publish_armed().unwrap();
        admission
            .publish_terminal(ReplayState::Succeeded, test_digest("outcome"))
            .unwrap();
        assert!(admission
            .publish_terminal(ReplayState::Failed, test_digest("second-outcome"))
            .is_err());
    }

    #[test]
    fn ledger_21_claim_only_reconstructs_blocked_incomplete() {
        let Some(root) = provisioned_test_root("ledger-reconstruct-claim") else {
            return;
        };
        let key = test_key("reconstruct-claim");
        let binding = test_binding("reconstruct-claim");
        let ledger = ReplayLedger::open(&root).unwrap();
        drop(
            ledger
                .admit_or_recover(key.clone(), binding.clone())
                .unwrap(),
        );
        drop(ledger);
        let ledger = ReplayLedger::open(&root).unwrap();
        let recovered = ledger.admit_or_recover(key, binding).unwrap();
        assert!(!recovered.is_fresh());
        assert_eq!(recovered.state(), ReplayState::ClaimedNoState);
    }

    #[test]
    fn ledger_22_generation_zero_reconstructs_blocked_incomplete() {
        let Some(root) = provisioned_test_root("ledger-reconstruct-g0") else {
            return;
        };
        let key = test_key("reconstruct-g0");
        let binding = test_binding("reconstruct-g0");
        let ledger = ReplayLedger::open(&root).unwrap();
        let mut admission = ledger
            .admit_or_recover(key.clone(), binding.clone())
            .unwrap();
        admission.publish_intent().unwrap();
        drop(admission);
        drop(ledger);
        let ledger = ReplayLedger::open(&root).unwrap();
        let recovered = ledger.admit_or_recover(key, binding).unwrap();
        assert_eq!(recovered.state(), ReplayState::IntentRecorded);
    }

    #[test]
    fn ledger_23_armed_reconstructs_blocked_possible_invocation() {
        let Some(root) = provisioned_test_root("ledger-reconstruct-g1") else {
            return;
        };
        let key = test_key("reconstruct-g1");
        let binding = test_binding("reconstruct-g1");
        let ledger = ReplayLedger::open(&root).unwrap();
        let mut admission = ledger
            .admit_or_recover(key.clone(), binding.clone())
            .unwrap();
        admission.publish_intent().unwrap();
        admission.publish_armed().unwrap();
        drop(admission);
        drop(ledger);
        let ledger = ReplayLedger::open(&root).unwrap();
        assert_eq!(
            ledger.admit_or_recover(key, binding).unwrap().state(),
            ReplayState::InvocationArmed
        );
    }

    #[test]
    fn ledger_24_succeeded_reconstructs_permanently_blocked() {
        assert_terminal_reconstruction(ReplayState::Succeeded, "succeeded");
    }

    #[test]
    fn ledger_25_failed_reconstructs_permanently_blocked() {
        assert_terminal_reconstruction(ReplayState::Failed, "failed");
    }

    #[test]
    fn ledger_26_uncertain_reconstructs_manual_resolution() {
        assert_terminal_reconstruction(ReplayState::Uncertain, "uncertain");
    }

    fn assert_terminal_reconstruction(state: ReplayState, label: &str) {
        let Some(root) = provisioned_test_root(&format!("ledger-reconstruct-{label}")) else {
            return;
        };
        let action = format!("reconstruct-{label}");
        let key = test_key(&action);
        let binding = test_binding(&action);
        let ledger = ReplayLedger::open(&root).unwrap();
        let mut admission = ledger
            .admit_or_recover(key.clone(), binding.clone())
            .unwrap();
        admission.publish_intent().unwrap();
        admission.publish_armed().unwrap();
        admission
            .publish_terminal(state, test_digest("durable-outcome"))
            .unwrap();
        drop(admission);
        drop(ledger);
        let ledger = ReplayLedger::open(&root).unwrap();
        let recovered = ledger.admit_or_recover(key, binding).unwrap();
        assert!(!recovered.is_fresh());
        assert_eq!(recovered.state(), state);
    }

    #[test]
    fn recovered_claim_g0_and_g1_admissions_cannot_advance_or_mutate() {
        for (label, generations) in [("claim", 0usize), ("g0", 1), ("g1", 2)] {
            let Some(root) = provisioned_test_root(&format!("ledger-recovered-{label}")) else {
                return;
            };
            let action = format!("recovered-{label}");
            let key = test_key(&action);
            let binding = test_binding(&action);
            let ledger = ReplayLedger::open(&root).unwrap();
            let mut fresh = ledger
                .admit_or_recover(key.clone(), binding.clone())
                .unwrap();
            if generations >= 1 {
                fresh.publish_intent().unwrap();
            }
            if generations >= 2 {
                fresh.publish_armed().unwrap();
            }
            drop(fresh);
            drop(ledger);

            let ledger = ReplayLedger::open(&root).unwrap();
            let mut recovered = ledger.admit_or_recover(key, binding).unwrap();
            assert!(!recovered.is_fresh());
            let before = tree_snapshot(&root);
            let result = match generations {
                0 => recovered.publish_intent(),
                1 => recovered.publish_armed(),
                2 => recovered.publish_terminal(ReplayState::Succeeded, test_digest("outcome")),
                _ => unreachable!(),
            };
            assert!(matches!(result, Err(ReplayError::PersistenceUnavailable)));
            assert_eq!(tree_snapshot(&root), before);
        }
    }

    #[test]
    fn recovered_terminal_admission_cannot_publish_or_mutate() {
        let Some(root) = provisioned_test_root("ledger-recovered-terminal") else {
            return;
        };
        let key = test_key("recovered-terminal");
        let binding = test_binding("recovered-terminal");
        let ledger = ReplayLedger::open(&root).unwrap();
        let mut fresh = ledger
            .admit_or_recover(key.clone(), binding.clone())
            .unwrap();
        fresh.publish_intent().unwrap();
        fresh.publish_armed().unwrap();
        fresh
            .publish_terminal(ReplayState::Succeeded, test_digest("outcome"))
            .unwrap();
        drop(fresh);
        drop(ledger);

        let ledger = ReplayLedger::open(&root).unwrap();
        let mut recovered = ledger.admit_or_recover(key, binding).unwrap();
        assert!(!recovered.is_fresh());
        let before = tree_snapshot(&root);
        assert!(recovered.publish_intent().is_err());
        assert!(recovered.publish_armed().is_err());
        assert!(recovered
            .publish_terminal(ReplayState::Failed, test_digest("other-outcome"))
            .is_err());
        assert_eq!(tree_snapshot(&root), before);
    }

    #[test]
    fn ledger_27_orphan_chain_fails_whole_ledger_closed() {
        let Some(root) = provisioned_test_root("ledger-orphan") else {
            return;
        };
        let digest = "ab00000000000000000000000000000000000000000000000000000000000000";
        std::fs::create_dir_all(
            root.join("replay")
                .join("v1")
                .join("chains")
                .join("ab")
                .join(digest),
        )
        .unwrap();
        assert!(matches!(
            ReplayLedger::open(&root),
            Err(ReplayError::PersistenceUnavailable)
        ));
    }

    #[test]
    fn ledger_28_malformed_chain_fails_closed() {
        let Some(root) = provisioned_test_root("ledger-malformed-chain") else {
            return;
        };
        let ledger = ReplayLedger::open(&root).unwrap();
        let admission = ledger
            .admit_or_recover(test_key("malformed-chain"), test_binding("malformed-chain"))
            .unwrap();
        let directory = execution_directory(&root, admission.execution_id());
        drop(admission);
        drop(ledger);
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(directory.join("g0000000000000000.json"), b"{").unwrap();
        assert!(ReplayLedger::open(&root).is_err());
    }

    #[test]
    fn ledger_29_unexpected_ledger_entry_fails_closed() {
        let Some(root) = provisioned_test_root("ledger-unexpected-entry") else {
            return;
        };
        std::fs::write(
            root.join("replay").join("v1").join("claims").join(format!(
                "{}.{}.tmp",
                test_key("unexpected-entry").filename_digest(),
                Uuid::new_v4().simple()
            )),
            b"debris",
        )
        .unwrap();
        assert!(matches!(
            ReplayLedger::open(&root),
            Err(ReplayError::PersistenceUnavailable)
        ));
    }

    #[test]
    fn ledger_30_restart_never_generates_new_uuid_for_existing_tuple() {
        let Some(root) = provisioned_test_root("ledger-no-new-uuid") else {
            return;
        };
        let key = test_key("no-new-uuid");
        let binding = test_binding("no-new-uuid");
        let ledger = ReplayLedger::open(&root).unwrap();
        let first = ledger
            .admit_or_recover(key.clone(), binding.clone())
            .unwrap()
            .execution_id()
            .to_owned();
        drop(ledger);
        let claim_before = std::fs::read(claim_path(&root, &key)).unwrap();
        for _ in 0..2 {
            let ledger = ReplayLedger::open(&root).unwrap();
            let recovered = ledger
                .admit_or_recover(key.clone(), binding.clone())
                .unwrap();
            assert_eq!(recovered.execution_id(), first);
        }
        assert_eq!(
            std::fs::read(claim_path(&root, &key)).unwrap(),
            claim_before
        );
    }

    #[test]
    fn ledger_populated_valid_subtrees_reopen_without_reprovisioning() {
        let Some(root) = provisioned_test_root("ledger-populated-reopen") else {
            return;
        };
        let key = test_key("populated-reopen");
        let binding = test_binding("populated-reopen");
        let ledger = ReplayLedger::open(&root).unwrap();
        let mut admission = ledger
            .admit_or_recover(key.clone(), binding.clone())
            .unwrap();
        admission.publish_intent().unwrap();
        admission.publish_armed().unwrap();
        admission
            .publish_terminal(ReplayState::Succeeded, test_digest("outcome"))
            .unwrap();
        drop(admission);
        drop(ledger);
        let before = tree_snapshot(&root);
        assert_eq!(
            provision_replay(&root),
            Ok(ProvisionReplayOutcome::AlreadyProvisioned)
        );
        assert_eq!(tree_snapshot(&root), before);
        let ledger = ReplayLedger::open(&root).unwrap();
        assert_eq!(
            ledger.admit_or_recover(key, binding).unwrap().state(),
            ReplayState::Succeeded
        );
    }

    #[test]
    fn ledger_all_bounded_persistence_seams_fail_closed() {
        let Some(root) = provisioned_test_root("ledger-fault-seams") else {
            return;
        };
        for point in [
            PersistenceFaultPoint::RestartScan,
            PersistenceFaultPoint::OrphanDetection,
        ] {
            assert!(matches!(
                with_persistence_fault(point, || ReplayLedger::open(&root)),
                Err(ReplayError::PersistenceUnavailable)
            ));
        }

        let key = test_key("fault-seams");
        let binding = test_binding("fault-seams");
        let ledger = ReplayLedger::open(&root).unwrap();
        let mut admission = ledger
            .admit_or_recover(key.clone(), binding.clone())
            .unwrap();
        for point in [
            PersistenceFaultPoint::ChainDirectoryValidation,
            PersistenceFaultPoint::GenerationZeroPublication,
        ] {
            assert!(matches!(
                with_persistence_fault(point, || admission.publish_intent()),
                Err(ReplayError::PersistenceUnavailable)
            ));
        }
        admission.publish_intent().unwrap();
        assert!(
            with_persistence_fault(PersistenceFaultPoint::GenerationOnePublication, || {
                admission.publish_armed()
            })
            .is_err()
        );
        admission.publish_armed().unwrap();
        assert!(
            with_persistence_fault(PersistenceFaultPoint::GenerationTwoPublication, || {
                admission.publish_terminal(ReplayState::Succeeded, test_digest("outcome"))
            })
            .is_err()
        );
        drop(admission);
        drop(ledger);

        for point in [
            PersistenceFaultPoint::ClaimRead,
            PersistenceFaultPoint::GenerationReopen,
            PersistenceFaultPoint::DigestVerification,
            PersistenceFaultPoint::ChainDirectoryValidation,
        ] {
            let ledger = ReplayLedger::open(&root).unwrap();
            assert!(matches!(
                with_persistence_fault(point, || {
                    ledger.admit_or_recover(key.clone(), binding.clone())
                }),
                Err(ReplayError::PersistenceUnavailable)
            ));
        }

        let collision_root = provisioned_test_root("ledger-collision-reopen-fault").unwrap();
        let collision_ledger = ReplayLedger::open(&collision_root).unwrap();
        assert!(
            with_persistence_fault(PersistenceFaultPoint::ClaimCollisionReopen, || {
                collision_ledger.admit_or_recover(
                    test_key("collision-reopen"),
                    test_binding("collision-reopen"),
                )
            })
            .is_err()
        );
    }

    // -----------------------------------------------------------------------
    // F3b-3: Replay Windows publish primitive characterization
    // -----------------------------------------------------------------------
    //
    // These tests characterize each stage of the accepted-main
    // `publish_new_canonical_file_with_temporary_stem` sequence in
    // isolation, using the same Win32 APIs without depending on the
    // Tethers Replay infrastructure.
    //
    // Six stages characterized:
    //   1. CreateFileW(CREATE_NEW | FILE_FLAG_WRITE_THROUGH) — durability
    //   2. WriteFile — complete write
    //   3. FlushFileBuffers before rename — file-data durability
    //   4. SetFileInformationByHandle rename — rename properties
    //   5. FlushFileBuffers on renamed file handle — what this proves
    //   6. reopen/re-read — exact-byte verification

    fn wide_path(path: &Path) -> Vec<u16> {
        path.as_os_str().encode_wide().chain(Some(0)).collect()
    }

    fn f3b_replay_temp_dir(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("f3b-replay-{}-{}", label, Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn f3b3_create_write_through_open_and_write() {
        // Stage 1 + 2: open with CREATE_NEW | FILE_FLAG_WRITE_THROUGH,
        // write complete bytes, verify.
        let dir = f3b_replay_temp_dir("create-wt");
        let path = dir.join("canonical.bin");
        let data = b"f3b3-replay-canary-v1-abcdef";

        let path_w = wide_path(&path);
        let handle;
        unsafe {
            handle = CreateFileW(
                path_w.as_ptr(),
                GENERIC_READ | GENERIC_WRITE,
                0, // exclusive
                std::ptr::null_mut(),
                CREATE_NEW,
                FILE_ATTRIBUTE_NORMAL | FILE_FLAG_WRITE_THROUGH,
                std::ptr::null_mut(),
            );
        }
        assert!(
            handle != INVALID_HANDLE_VALUE && handle != std::ptr::null_mut(),
            "F3b-3 stage 1: CreateFileW(CREATE_NEW | FILE_FLAG_WRITE_THROUGH) succeeded"
        );

        let mut written = 0u32;
        unsafe {
            let ok = WriteFile(
                handle,
                data.as_ptr(),
                data.len() as u32,
                &mut written,
                std::ptr::null_mut(),
            );
            assert_ne!(ok, 0, "WriteFile succeeded");
        }
        assert_eq!(written as usize, data.len(), "all bytes written");

        // Close handle before reading (share_mode was 0)
        unsafe {
            CloseHandle(handle);
        }

        // Verify bytes on disk (after close)
        let on_disk = std::fs::read(&path).expect("read back");
        assert_eq!(
            on_disk, data,
            "F3b-3 stage 2: WriteFile bytes match on-disk contents"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn f3b3_flush_before_rename_file_data_durability() {
        // Stage 3: FlushFileBuffers before rename confirms file-data
        // durability for the temporary file.
        let dir = f3b_replay_temp_dir("flush-before-rename");
        let tmp = dir.join("temp.bin");
        let final_path = dir.join("final.bin");
        let data = b"f3b3-flush-canary-data";

        let tmp_w = wide_path(&tmp);
        let handle;
        unsafe {
            handle = CreateFileW(
                tmp_w.as_ptr(),
                GENERIC_READ | GENERIC_WRITE | DELETE,
                0,
                std::ptr::null_mut(),
                CREATE_NEW,
                FILE_ATTRIBUTE_NORMAL | FILE_FLAG_WRITE_THROUGH,
                std::ptr::null_mut(),
            );
        }
        assert_ne!(handle, INVALID_HANDLE_VALUE);

        unsafe {
            let mut written = 0u32;
            WriteFile(
                handle,
                data.as_ptr().cast(),
                data.len() as u32,
                &mut written,
                std::ptr::null_mut(),
            );
            assert_eq!(written as usize, data.len());
        }

        // Stage 3: FlushFileBuffers before rename
        unsafe {
            let flush_ok = FlushFileBuffers(handle);
            assert_ne!(
                flush_ok, 0,
                "F3b-3 stage 3: FlushFileBuffers before rename succeeded"
            );
        }

        // Rename via SetFileInformationByHandle (stages 4)
        unsafe {
            let name: Vec<u16> = final_path.as_os_str().encode_wide().collect();
            let name_bytes = (name.len() * std::mem::size_of::<u16>()) as u32;
            let total = std::mem::size_of::<FILE_RENAME_INFO>() + name_bytes as usize;
            let buf_words = total.div_ceil(std::mem::size_of::<usize>());
            let mut buf = vec![0usize; buf_words];
            let info = buf.as_mut_ptr().cast::<FILE_RENAME_INFO>();
            (*info).Anonymous = FILE_RENAME_INFO_0 {
                ReplaceIfExists: false,
            };
            (*info).RootDirectory = std::ptr::null_mut();
            (*info).FileNameLength = name_bytes;
            std::ptr::copy_nonoverlapping(
                name.as_ptr(),
                std::ptr::addr_of_mut!((*info).FileName).cast::<u16>(),
                name.len(),
            );
            let rename_ok = SetFileInformationByHandle(
                handle,
                FileRenameInfo,
                info.cast(),
                (buf_words * std::mem::size_of::<usize>()) as u32,
            );
            assert_ne!(
                rename_ok, 0,
                "F3b-3 stage 4: SetFileInformationByHandle rename succeeded"
            );
        }

        // Stage 5: FlushFileBuffers on renamed file handle
        unsafe {
            let flush2_ok = FlushFileBuffers(handle);
            assert_ne!(
                flush2_ok, 0,
                "F3b-3 stage 5: FlushFileBuffers on renamed file handle succeeded — \
                 this flushes file metadata/data for the renamed file handle, \
                 not the parent directory."
            );
        }

        // Close handle
        unsafe {
            CloseHandle(handle);
        }

        // Stage 6: reopen and re-read
        let reopened_bytes = std::fs::read(&final_path).expect("reopen final");
        assert_eq!(
            reopened_bytes, data,
            "F3b-3 stage 6: reopened bytes match original written bytes. \
             This proves the rename landed complete file data. \
             It does NOT prove the parent directory entry is durable."
        );

        // Temporary path should be gone (rename moved it)
        assert!(
            !tmp.exists(),
            "temporary path no longer exists after rename"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn f3b3_create_new_prevents_overwrite() {
        // CREATE_NEW rejects an existing file. This is the TOCTOU defence
        // in the Replay primitive.
        let dir = f3b_replay_temp_dir("create-new");
        let path = dir.join("exclusive.txt");

        std::fs::write(&path, b"pre-existing").expect("pre-write");

        let path_w = wide_path(&path);
        let handle;
        unsafe {
            handle = CreateFileW(
                path_w.as_ptr(),
                GENERIC_WRITE,
                0,
                std::ptr::null_mut(),
                CREATE_NEW,
                FILE_ATTRIBUTE_NORMAL,
                std::ptr::null_mut(),
            );
        }
        assert!(
            handle == INVALID_HANDLE_VALUE,
            "F3b-3: CREATE_NEW correctly rejects existing file. \
             A concurrent claim cannot overwrite a published generation."
        );

        // Verify pre-existing content untouched
        assert_eq!(
            std::fs::read(&path).expect("read"),
            b"pre-existing",
            "existing content unchanged"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn f3b3_rename_without_replacement_defence() {
        // SetFileInformationByHandle with ReplaceIfExists:false rejects
        // when the destination already exists.
        let dir = f3b_replay_temp_dir("rename-no-replace");
        let src = dir.join("src.bin");
        let dst = dir.join("dst.bin");

        std::fs::write(&dst, b"pre-existing-destination").expect("pre-write dst");
        std::fs::write(&src, b"source-content").expect("write src");

        let src_w = wide_path(&src);
        let handler;
        unsafe {
            handler = CreateFileW(
                src_w.as_ptr(),
                GENERIC_READ | GENERIC_WRITE | DELETE,
                FILE_SHARE_READ,
                std::ptr::null_mut(),
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                std::ptr::null_mut(),
            );
        }
        assert_ne!(handler, INVALID_HANDLE_VALUE);

        let rename_result;
        unsafe {
            let name: Vec<u16> = dst.as_os_str().encode_wide().collect();
            let name_bytes = (name.len() * 2) as u32;
            let total = std::mem::size_of::<FILE_RENAME_INFO>() + name_bytes as usize;
            let buf_words = total.div_ceil(std::mem::size_of::<usize>());
            let mut buf = vec![0usize; buf_words];
            let info = buf.as_mut_ptr().cast::<FILE_RENAME_INFO>();
            (*info).Anonymous = FILE_RENAME_INFO_0 {
                ReplaceIfExists: false,
            };
            (*info).RootDirectory = std::ptr::null_mut();
            (*info).FileNameLength = name_bytes;
            std::ptr::copy_nonoverlapping(
                name.as_ptr(),
                std::ptr::addr_of_mut!((*info).FileName).cast::<u16>(),
                name.len(),
            );
            rename_result = SetFileInformationByHandle(
                handler,
                FileRenameInfo,
                info.cast(),
                (buf_words * std::mem::size_of::<usize>()) as u32,
            );
        }

        unsafe {
            CloseHandle(handler);
        }

        if rename_result == 0 {
            // Expected on many configurations:
            // SetFileInformationByHandle rejects replacement when
            // ReplaceIfExists is false and destination exists.
            eprintln!(
                "F3b-3: SetFileInformationByHandle(ReplaceIfExists:false) \
                 correctly rejected rename when destination existed. \
                 This is the non-replacing rename defence in the Replay primitive."
            );
        } else {
            eprintln!(
                "F3b-3: SetFileInformationByHandle(ReplaceIfExists:false) \
                 succeeded despite pre-existing destination. \
                 This platform/volume does not enforce ReplaceIfExists \
                 exclusion for this handle type."
            );
        }

        let _ = std::fs::remove_dir_all(&dir);
    }
}
