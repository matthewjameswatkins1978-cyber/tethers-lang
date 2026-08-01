//! Small audited persistence primitives shared by the separate M3 stores.

use crate::manifest;
use serde::{de::DeserializeOwned, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;
#[cfg(windows)]
use windows_sys::Win32::Storage::FileSystem::{
    GetFileAttributesW, FILE_ATTRIBUTE_REPARSE_POINT, INVALID_FILE_ATTRIBUTES,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct M3Error {
    pub code: &'static str,
    pub message: String,
}

impl M3Error {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for M3Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for M3Error {}

pub type Result<T> = std::result::Result<T, M3Error>;

pub fn sha256(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

pub fn unix_ms() -> Result<u64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_millis() as u64)
        .map_err(|_| M3Error::new("clock_invalid", "clock is before the Unix epoch"))
}

pub fn canonical<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    serde_json_canonicalizer::to_vec(value)
        .map_err(|error| M3Error::new("record_invalid", error.to_string()))
}

pub fn strict_json<T: DeserializeOwned>(bytes: &[u8]) -> Result<T> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| M3Error::new("record_invalid", "record is not UTF-8"))?;
    let value = manifest::parse_value_no_dupes(text)
        .map_err(|error| M3Error::new("record_invalid", error.to_string()))?;
    serde_json::from_value(value).map_err(|error| M3Error::new("record_invalid", error.to_string()))
}

#[cfg(windows)]
fn wide(path: &Path) -> Vec<u16> {
    path.as_os_str().encode_wide().chain(Some(0)).collect()
}

pub fn reject_reparse(path: &Path) -> Result<()> {
    let metadata =
        fs::symlink_metadata(path).map_err(|error| M3Error::new("store_io", error.to_string()))?;
    if metadata.file_type().is_symlink() {
        return Err(M3Error::new("unsafe_store_path", "symbolic link refused"));
    }
    #[cfg(windows)]
    {
        let path_w = wide(path);
        // SAFETY: path_w is a live nul-terminated UTF-16 buffer and no pointer escapes.
        let attributes = unsafe { GetFileAttributesW(path_w.as_ptr()) };
        if attributes == INVALID_FILE_ATTRIBUTES || attributes & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            return Err(M3Error::new(
                "unsafe_store_path",
                "Windows reparse point refused",
            ));
        }
    }
    Ok(())
}

pub fn verify_chain(path: &Path) -> Result<()> {
    if !path.is_absolute() {
        return Err(M3Error::new(
            "unsafe_store_path",
            "store root must be absolute",
        ));
    }
    for ancestor in path.ancestors() {
        match fs::symlink_metadata(ancestor) {
            Ok(_) => reject_reparse(ancestor)?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(M3Error::new("store_io", error.to_string())),
        }
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub struct StoreRoot(PathBuf);

impl StoreRoot {
    pub fn open(path: &Path) -> Result<Self> {
        verify_chain(path)?;
        fs::create_dir_all(path).map_err(|error| M3Error::new("store_io", error.to_string()))?;
        verify_chain(path)?;
        let root =
            fs::canonicalize(path).map_err(|error| M3Error::new("store_io", error.to_string()))?;
        reject_reparse(&root)?;
        Ok(Self(root))
    }

    pub fn path(&self) -> &Path {
        &self.0
    }

    pub fn entries(&self) -> Result<Vec<PathBuf>> {
        verify_chain(&self.0)?;
        let mut paths = Vec::new();
        for entry in
            fs::read_dir(&self.0).map_err(|error| M3Error::new("store_io", error.to_string()))?
        {
            let path = entry
                .map_err(|error| M3Error::new("store_io", error.to_string()))?
                .path();
            reject_reparse(&path)?;
            paths.push(path);
        }
        paths.sort();
        Ok(paths)
    }

    pub fn read<T: DeserializeOwned>(&self, path: &Path) -> Result<T> {
        reject_reparse(path)?;
        let bytes = fs::read(path).map_err(|error| M3Error::new("store_io", error.to_string()))?;
        strict_json(&bytes)
    }

    pub fn create_json<T: Serialize>(&self, id: &str, value: &T) -> Result<PathBuf> {
        if id.is_empty()
            || !id
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        {
            return Err(M3Error::new("record_invalid", "unsafe record identity"));
        }
        verify_chain(&self.0)?;
        let destination = self.0.join(format!("{id}.json"));
        let temporary = self.0.join(format!(".{id}.tmp"));
        if destination.exists() || temporary.exists() {
            return Err(M3Error::new("record_conflict", "record already exists"));
        }
        let bytes = canonical(value)?;
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|error| M3Error::new("store_io", error.to_string()))?;
        file.write_all(&bytes)
            .map_err(|error| M3Error::new("store_io", error.to_string()))?;
        file.sync_all()
            .map_err(|error| M3Error::new("store_io", error.to_string()))?;
        verify_chain(&self.0)?;
        fs::rename(&temporary, &destination)
            .map_err(|error| M3Error::new("store_io", error.to_string()))?;
        Ok(destination)
    }
}
