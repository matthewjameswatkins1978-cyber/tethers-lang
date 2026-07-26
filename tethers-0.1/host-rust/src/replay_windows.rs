//! Native Windows J09 replay-root admission boundary.
//!
//! This module contains every J09 Win32 call.  A path is never authority after
//! it has been parsed: each existing component is opened without reparse-point
//! following, and the final directory handle carries the volume and ACL proof.

use crate::replay::ReplayError;
use std::ffi::c_void;
use std::mem::{size_of, MaybeUninit};
use std::os::windows::ffi::OsStrExt;
use std::path::{Component, Path, PathBuf, Prefix};
use uuid::Uuid;
use windows_sys::Win32::Foundation::{
    CloseHandle, GetLastError, GENERIC_ALL, GENERIC_READ, GENERIC_WRITE, HANDLE,
    INVALID_HANDLE_VALUE,
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
    GetFileInformationByHandle, GetVolumeInformationByHandleW, ReadFile,
    SetFileInformationByHandle, WriteFile, BY_HANDLE_FILE_INFORMATION, CREATE_NEW, DELETE,
    FILE_ADD_FILE, FILE_ADD_SUBDIRECTORY, FILE_APPEND_DATA, FILE_ATTRIBUTE_NORMAL,
    FILE_ATTRIBUTE_REPARSE_POINT, FILE_DELETE_CHILD, FILE_FLAG_BACKUP_SEMANTICS,
    FILE_FLAG_OPEN_REPARSE_POINT, FILE_FLAG_WRITE_THROUGH, FILE_GENERIC_READ, FILE_RENAME_INFO,
    FILE_RENAME_INFO_0, FILE_SHARE_READ, FILE_SHARE_WRITE, FILE_WRITE_ATTRIBUTES, FILE_WRITE_DATA,
    FILE_WRITE_EA, OPEN_EXISTING, READ_CONTROL, WRITE_DAC, WRITE_OWNER,
};
use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

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
    #[cfg(test)]
    clear_native_publish_diagnostic();
    let temporary = ValidatedLeafName::new(&format!(
        "{}.{}.tmp",
        final_name.as_str(),
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
        let child = version.child_directory(&ValidatedLeafName::new(name)?)?;
        exact_directory_entries(&child.path, &[])?;
    }
    Ok(())
}

/// Establish exactly the one permitted empty v1 hierarchy. Existing partial or
/// unrecognised state is deliberately not repaired, even when it looks benign.
pub fn provision_replay(root_path: &Path) -> Result<ProvisionReplayOutcome, ReplayError> {
    let root = validate_existing_root(root_path)?;
    if child_exists(root.path(), "replay") {
        validate_complete_hierarchy(root)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Barrier};

    fn fresh_native_test_root(label: &str) -> Option<PathBuf> {
        let base = std::env::var_os("TETHERS_J09_NATIVE_PROVISION_ROOT")?;
        let root = PathBuf::from(base).join(format!("{label}-{}", Uuid::new_v4().simple()));
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
}
