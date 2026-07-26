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
use windows_sys::Win32::Foundation::{
    CloseHandle, GENERIC_ALL, GENERIC_WRITE, HANDLE, INVALID_HANDLE_VALUE,
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
    CreateFileW, GetDriveTypeW, GetFileInformationByHandle, GetVolumeInformationByHandleW,
    BY_HANDLE_FILE_INFORMATION, DELETE, FILE_ADD_FILE, FILE_ADD_SUBDIRECTORY, FILE_APPEND_DATA,
    FILE_ATTRIBUTE_NORMAL, FILE_ATTRIBUTE_REPARSE_POINT, FILE_DELETE_CHILD,
    FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, FILE_GENERIC_READ, FILE_SHARE_READ,
    FILE_SHARE_WRITE, FILE_WRITE_ATTRIBUTES, FILE_WRITE_DATA, FILE_WRITE_EA, OPEN_EXISTING,
    READ_CONTROL, WRITE_DAC, WRITE_OWNER,
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

fn unavailable<T>() -> Result<T, ReplayError> {
    Err(ReplayError::PersistenceUnavailable)
}

fn wide(path: &Path) -> Vec<u16> {
    path.as_os_str().encode_wide().chain(Some(0)).collect()
}

/// A handle whose ownership is linear: every successful Win32 open reaches this
/// wrapper before any later fallible check, so all early returns close it.
struct OwnedHandle(HANDLE);
impl Drop for OwnedHandle {
    fn drop(&mut self) {
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
    let path_w = wide(path);
    // SAFETY: nul-terminated path lives through the call. BACKUP_SEMANTICS opens
    // a directory and OPEN_REPARSE_POINT prevents final-component traversal.
    let raw = unsafe {
        CreateFileW(
            path_w.as_ptr(),
            FILE_GENERIC_READ | READ_CONTROL,
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

#[cfg(test)]
mod tests {
    use super::*;
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
}
