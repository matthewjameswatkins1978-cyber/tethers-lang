//! M4 File Tools contract and host-owned filesystem adapter.
//!
//! The provider receives only scoped relative paths.  This module repeats the
//! checks at the host boundary so policy and provider validation do not depend
//! on either side trusting the other.

use serde_json::{json, Map, Value};
use std::fs;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use std::time::Duration;

pub const METADATA_CAPABILITY: &str = "file.metadata";
pub const MOVE_CAPABILITY: &str = "file.move";
pub const METADATA_OPERATION: &str = "file_metadata";
pub const MOVE_OPERATION: &str = "file_move";
pub const METADATA_V2_OPERATION: &str = "file_metadata_v2";
pub const MAX_CONTENT_BYTES: u64 = 64 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OperationalScopeBinding {
    pub installed_id: String,
    pub capability_name: String,
    pub capability_version: u32,
    pub query_root: PathBuf,
    pub move_source_root: PathBuf,
    pub move_destination_root: PathBuf,
    pub max_content_bytes: u64,
    pub authority: String,
    pub integrity_digest: String,
}

impl OperationalScopeBinding {
    pub fn create(
        installed_id: &str,
        capability_name: &str,
        capability_version: u32,
        query_root: &Path,
        move_source_root: &Path,
        move_destination_root: &Path,
        max_content_bytes: u64,
        authority: &str,
    ) -> Result<Self, FileToolsError> {
        let mut binding = Self {
            installed_id: installed_id.into(),
            capability_name: capability_name.into(),
            capability_version,
            query_root: canonical_directory(query_root, "query_root")?,
            move_source_root: canonical_directory(move_source_root, "move_source_root")?,
            move_destination_root: canonical_directory(
                move_destination_root,
                "move_destination_root",
            )?,
            max_content_bytes,
            authority: authority.into(),
            integrity_digest: String::new(),
        };
        let roots = [
            &binding.query_root,
            &binding.move_source_root,
            &binding.move_destination_root,
        ];
        for (index, root) in roots.iter().enumerate() {
            for other in roots.iter().skip(index + 1) {
                if root == other || root.starts_with(other) || other.starts_with(root) {
                    return Err(FileToolsError::new(
                        "scope_invalid",
                        "operational roots must be distinct and non-nested",
                    ));
                }
            }
        }
        let mut covered = binding.clone();
        covered.integrity_digest.clear();
        let bytes = serde_json_canonicalizer::to_vec(&covered)
            .map_err(|e| FileToolsError::new("scope_invalid", e.to_string()))?;
        use sha2::{Digest, Sha256};
        binding.integrity_digest = format!("sha256:{:x}", Sha256::digest(bytes));
        binding.validate()?;
        Ok(binding)
    }

    pub fn validate(&self) -> Result<(), FileToolsError> {
        if self.installed_id.is_empty()
            || self.capability_name.is_empty()
            || self.capability_version == 0
            || self.authority.is_empty()
            || self.max_content_bytes == 0
            || self.max_content_bytes > MAX_CONTENT_BYTES
        {
            return Err(FileToolsError::new(
                "scope_invalid",
                "invalid operational scope binding",
            ));
        }
        let mut covered = self.clone();
        let digest = covered.integrity_digest.clone();
        covered.integrity_digest.clear();
        let bytes = serde_json_canonicalizer::to_vec(&covered)
            .map_err(|e| FileToolsError::new("scope_invalid", e.to_string()))?;
        use sha2::{Digest, Sha256};
        if digest != format!("sha256:{:x}", Sha256::digest(bytes)) {
            return Err(FileToolsError::new(
                "scope_invalid",
                "scope integrity evidence is invalid",
            ));
        }
        for root in [
            &self.query_root,
            &self.move_source_root,
            &self.move_destination_root,
        ] {
            reject_reparse_chain(root)?;
        }
        Ok(())
    }

    pub fn scope(&self) -> Result<FileScope, FileToolsError> {
        self.validate()?;
        FileScope::new(
            &self.query_root,
            &self.move_source_root,
            &self.move_destination_root,
        )?
        .with_max_content_bytes(self.max_content_bytes)
    }

    pub fn from_canonical_scope_json(
        json: &str,
        installed_id: &str,
        capability_name: &str,
        capability_version: u32,
    ) -> Result<Self, FileToolsError> {
        let value: serde_json::Value = serde_json::from_str(json)
            .map_err(|e| FileToolsError::new("scope_invalid", format!("parse scope JSON: {e}")))?;
        let obj = value
            .as_object()
            .ok_or_else(|| FileToolsError::new("scope_invalid", "scope must be a JSON object"))?;
        let query_root = obj
            .get("query_root")
            .and_then(|v| v.as_str())
            .ok_or_else(|| FileToolsError::new("scope_invalid", "query_root is required"))?;
        let move_source_root = obj
            .get("move_source_root")
            .and_then(|v| v.as_str())
            .ok_or_else(|| FileToolsError::new("scope_invalid", "move_source_root is required"))?;
        let move_destination_root = obj
            .get("move_destination_root")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                FileToolsError::new("scope_invalid", "move_destination_root is required")
            })?;
        let max_content_bytes: u64 = obj
            .get("max_content_bytes")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| FileToolsError::new("scope_invalid", "max_content_bytes is required"))?;
        Self::create(
            installed_id,
            capability_name,
            capability_version,
            Path::new(query_root),
            Path::new(move_source_root),
            Path::new(move_destination_root),
            max_content_bytes,
            "generic-scope",
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileToolsError {
    pub code: &'static str,
    pub message: String,
}

impl FileToolsError {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for FileToolsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}
impl std::error::Error for FileToolsError {}

#[derive(Debug, Clone)]
pub struct FileScope {
    pub query_root: PathBuf,
    pub source_root: PathBuf,
    pub destination_root: PathBuf,
    pub max_content_bytes: u64,
}

impl FileScope {
    pub fn new(
        query_root: &Path,
        source_root: &Path,
        destination_root: &Path,
    ) -> Result<Self, FileToolsError> {
        let query_root = canonical_directory(query_root, "query_root")?;
        let source_root = canonical_directory(source_root, "source_root")?;
        let destination_root = canonical_directory(destination_root, "destination_root")?;
        Ok(Self {
            query_root,
            source_root,
            destination_root,
            max_content_bytes: MAX_CONTENT_BYTES,
        })
    }

    pub fn with_max_content_bytes(mut self, limit: u64) -> Result<Self, FileToolsError> {
        if limit == 0 || limit > MAX_CONTENT_BYTES {
            return Err(FileToolsError::new(
                "file_limit_invalid",
                "content limit is outside the M4 bound",
            ));
        }
        self.max_content_bytes = limit;
        Ok(self)
    }
}

fn canonical_directory(path: &Path, label: &'static str) -> Result<PathBuf, FileToolsError> {
    if !path.is_absolute() {
        return Err(FileToolsError::new(
            "scope_invalid",
            format!("{label} must be absolute"),
        ));
    }
    reject_reparse_chain(path)?;
    let canonical = fs::canonicalize(path)
        .map_err(|e| FileToolsError::new("scope_invalid", format!("{label}: {e}")))?;
    if !canonical.is_dir() {
        return Err(FileToolsError::new(
            "scope_invalid",
            format!("{label} is not a directory"),
        ));
    }
    reject_reparse_chain(&canonical)?;
    Ok(canonical)
}

fn reject_reparse_chain(path: &Path) -> Result<(), FileToolsError> {
    if !path.is_absolute() {
        return Err(FileToolsError::new(
            "reparse_refused",
            "path must be absolute",
        ));
    }
    for ancestor in path.ancestors() {
        if let Ok(metadata) = fs::symlink_metadata(ancestor) {
            if metadata.file_type().is_symlink() {
                return Err(FileToolsError::new(
                    "reparse_refused",
                    "symbolic link encountered",
                ));
            }
            #[cfg(windows)]
            {
                use std::os::windows::ffi::OsStrExt;
                use windows_sys::Win32::Storage::FileSystem::{
                    GetFileAttributesW, FILE_ATTRIBUTE_REPARSE_POINT, INVALID_FILE_ATTRIBUTES,
                };
                let wide: Vec<u16> = ancestor.as_os_str().encode_wide().chain(Some(0)).collect();
                let attributes = unsafe { GetFileAttributesW(wide.as_ptr()) };
                if attributes == INVALID_FILE_ATTRIBUTES
                    || attributes & FILE_ATTRIBUTE_REPARSE_POINT != 0
                {
                    return Err(FileToolsError::new(
                        "reparse_refused",
                        "Windows reparse point encountered",
                    ));
                }
            }
        }
    }
    Ok(())
}

fn relative_path<'a>(raw: &'a Value, field: &'static str) -> Result<&'a str, FileToolsError> {
    let path = raw.as_str().ok_or_else(|| {
        FileToolsError::new("arguments_invalid", format!("{field} must be a string"))
    })?;
    if path.is_empty()
        || path.len() > 240
        || path.contains('\\')
        || path.contains(':')
        || path.starts_with('/')
        || path.contains('\0')
    {
        return Err(FileToolsError::new(
            "path_invalid",
            format!("{field} must be a bounded relative slash path"),
        ));
    }
    if path
        .split('/')
        .any(|segment| segment.is_empty() || segment == "." || segment == "..")
    {
        return Err(FileToolsError::new(
            "path_invalid",
            format!("{field} contains an unsafe segment"),
        ));
    }
    Ok(path)
}

fn scoped_path(
    root: &Path,
    raw: &Value,
    field: &'static str,
) -> Result<(String, PathBuf), FileToolsError> {
    let relative = relative_path(raw, field)?.to_owned();
    let path = root.join(relative.replace('/', std::path::MAIN_SEPARATOR_STR));
    let full = fs::canonicalize(&path)
        .map_err(|e| FileToolsError::new("path_unavailable", format!("{field}: {e}")))?;
    reject_reparse_chain(&full)?;
    let root = root.to_path_buf();
    if !full.starts_with(&root) || full == root {
        return Err(FileToolsError::new(
            "scope_violation",
            format!("{field} is outside its approved root"),
        ));
    }
    Ok((relative, full))
}

fn require_exact_object<'a>(
    value: &'a Value,
    fields: &[&str],
) -> Result<&'a Map<String, Value>, FileToolsError> {
    let object = value
        .as_object()
        .ok_or_else(|| FileToolsError::new("arguments_invalid", "arguments must be an object"))?;
    if object.len() != fields.len() || fields.iter().any(|field| !object.contains_key(*field)) {
        return Err(FileToolsError::new(
            "arguments_invalid",
            "unknown or missing file operation argument",
        ));
    }
    Ok(object)
}

pub fn metadata(scope: &FileScope, arguments: &Value) -> Result<Value, FileToolsError> {
    let object = require_exact_object(arguments, &["path", "include_content"])?;
    let (relative, path) = scoped_path(&scope.query_root, object.get("path").unwrap(), "path")?;
    let include_content = object
        .get("include_content")
        .unwrap()
        .as_bool()
        .ok_or_else(|| {
            FileToolsError::new("arguments_invalid", "include_content must be boolean")
        })?;
    let metadata =
        fs::metadata(&path).map_err(|e| FileToolsError::new("file_unavailable", e.to_string()))?;
    if !metadata.is_file() {
        return Err(FileToolsError::new(
            "wrong_type",
            "path is not a regular file",
        ));
    }
    if metadata.len() > scope.max_content_bytes && include_content {
        return Err(FileToolsError::new(
            "file_too_large",
            "content exceeds the host read bound",
        ));
    }
    let mut result = json!({"path": relative, "is_file": true, "size_bytes": metadata.len()});
    if include_content {
        let bytes =
            fs::read(&path).map_err(|e| FileToolsError::new("file_read_failed", e.to_string()))?;
        let content = String::from_utf8(bytes)
            .map_err(|_| FileToolsError::new("invalid_utf8", "content is not valid UTF-8"))?;
        result["content"] = Value::String(content);
    }
    Ok(result)
}

pub fn move_file(scope: &FileScope, arguments: &Value) -> Result<Value, FileToolsError> {
    let object = require_exact_object(arguments, &["source_path", "destination_path"])?;
    let (source_relative, source) = scoped_path(
        &scope.source_root,
        object.get("source_path").unwrap(),
        "source_path",
    )?;
    let destination_relative =
        relative_path(object.get("destination_path").unwrap(), "destination_path")?.to_owned();
    let destination = scope
        .destination_root
        .join(destination_relative.replace('/', std::path::MAIN_SEPARATOR_STR));
    reject_reparse_chain(&scope.destination_root)?;
    if !destination.starts_with(&scope.destination_root) || destination == scope.destination_root {
        return Err(FileToolsError::new(
            "scope_violation",
            "destination_path is outside its approved root",
        ));
    }
    if !source.is_file() {
        return Err(FileToolsError::new(
            "wrong_type",
            "source_path is not a regular file",
        ));
    }
    if source == destination || destination.exists() {
        return Err(FileToolsError::new(
            "overwrite_refused",
            "destination exists or equals source",
        ));
    }
    let parent = destination
        .parent()
        .ok_or_else(|| FileToolsError::new("path_invalid", "destination has no parent"))?;
    reject_reparse_chain(parent)?;
    if !parent.is_dir() {
        return Err(FileToolsError::new(
            "destination_missing",
            "destination directory does not exist",
        ));
    }
    fs::rename(&source, &destination)
        .map_err(|e| FileToolsError::new("move_failed", e.to_string()))?;
    if source.exists() || !destination.is_file() {
        return Err(FileToolsError::new(
            "move_uncertain",
            "move completion could not be verified",
        ));
    }
    Ok(
        json!({"moved": true, "source_path": source_relative, "destination_path": destination_relative}),
    )
}

pub fn metadata_v2(scope: &FileScope, arguments: &Value) -> Result<Value, FileToolsError> {
    let object = arguments
        .as_object()
        .ok_or_else(|| FileToolsError::new("arguments_invalid", "arguments must be an object"))?;
    let allowed: &[&str] = &["path", "compute_hash"];
    if object.iter().any(|(k, _)| !allowed.contains(&k.as_str())) || !object.contains_key("path") {
        return Err(FileToolsError::new(
            "arguments_invalid",
            "unknown or missing file.metadata argument",
        ));
    }
    let compute_hash = object
        .get("compute_hash")
        .map(|v| {
            v.as_bool().ok_or_else(|| {
                FileToolsError::new("arguments_invalid", "compute_hash must be boolean")
            })
        })
        .transpose()?
        .unwrap_or(false);
    let (relative, path) = scoped_path(&scope.query_root, object.get("path").unwrap(), "path")?;
    let file_meta =
        fs::metadata(&path).map_err(|e| FileToolsError::new("file_unavailable", e.to_string()))?;
    let file_type = file_meta.file_type();
    let kind = if file_type.is_file() {
        "file"
    } else if file_type.is_dir() {
        "directory"
    } else {
        "other"
    };
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| FileToolsError::new("path_invalid", "unable to extract filename"))?;
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());
    let size_bytes: Option<u64> = if file_type.is_file() {
        Some(file_meta.len())
    } else {
        None
    };
    let modified_unix_ms = file_meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as i64);
    let created_unix_ms = file_meta
        .created()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as i64);
    let read_only = file_meta.permissions().readonly();
    let hidden = is_hidden(&path);
    let sha256: Option<String> = if compute_hash && file_type.is_file() {
        Some(compute_sha256_string(&path)?)
    } else if compute_hash && !file_type.is_file() {
        return Err(FileToolsError::new(
            "wrong_type",
            "hashing requires a regular file",
        ));
    } else {
        None
    };
    Ok(json!({
        "path": relative,
        "name": name,
        "extension": extension,
        "kind": kind,
        "size_bytes": size_bytes,
        "modified_unix_ms": modified_unix_ms,
        "created_unix_ms": created_unix_ms,
        "read_only": read_only,
        "hidden": hidden,
        "sha256": sha256
    }))
}

#[cfg(windows)]
fn is_hidden(path: &std::path::Path) -> bool {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{
        GetFileAttributesW, FILE_ATTRIBUTE_HIDDEN, INVALID_FILE_ATTRIBUTES,
    };
    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    let attributes = unsafe { GetFileAttributesW(wide.as_ptr()) };
    attributes != INVALID_FILE_ATTRIBUTES && (attributes & FILE_ATTRIBUTE_HIDDEN) != 0
}

#[cfg(not(windows))]
fn is_hidden(path: &std::path::Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.starts_with('.'))
        .unwrap_or(false)
}

fn compute_sha256_string(path: &std::path::Path) -> Result<String, FileToolsError> {
    use sha2::{Digest, Sha256};
    let mut file =
        fs::File::open(path).map_err(|e| FileToolsError::new("file_read_failed", e.to_string()))?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 65536];
    loop {
        let n = file
            .read(&mut buf)
            .map_err(|e| FileToolsError::new("file_read_failed", e.to_string()))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("sha256:{:x}", hasher.finalize()))
}

pub fn metadata_input_schema() -> Value {
    json!({"type":"object","properties":{"path":{"type":"string"},"include_content":{"type":"boolean"}},"required":["path","include_content"],"additionalProperties":false})
}

pub fn metadata_output_schema() -> Value {
    json!({"type":"object","properties":{"path":{"type":"string"},"is_file":{"type":"boolean"},"size_bytes":{"type":"integer","minimum":0},"content":{"type":"string"}},"required":["path","is_file","size_bytes"],"additionalProperties":false})
}

pub fn move_input_schema() -> Value {
    json!({"type":"object","properties":{"source_path":{"type":"string"},"destination_path":{"type":"string"}},"required":["source_path","destination_path"],"additionalProperties":false})
}

pub fn move_output_schema() -> Value {
    json!({"type":"object","properties":{"moved":{"type":"boolean"},"source_path":{"type":"string"},"destination_path":{"type":"string"}},"required":["moved","source_path","destination_path"],"additionalProperties":false})
}

/// Machine-readable contract projection used by package builders and conformance.
/// The returned value deliberately omits `digest`; callers compute and insert the
/// JCS digest after all authoritative fields are frozen.
pub fn metadata_manifest_without_digest() -> Value {
    json!({"manifest_format_version":"1.0","capability_name":METADATA_CAPABILITY,"capability_version":1,"title":"Bounded File Metadata","description":"Read bounded metadata and optional UTF-8 content from one host-approved root.","input_schema":metadata_input_schema(),"output_schema":metadata_output_schema(),"effects":["data.read","metadata.read"],"permission_scope":{"kind":"path_prefix","allowed_prefixes":["query/"]},"reversibility":"reversible","determinism":"deterministic","idempotency":{"mechanism":"none"},"confirmation_policy":{"standing_permitted":true,"per_call_required":false},"timeout_ms":5000,"retry_policy":{"max_retries":0,"backoff_ms":0,"allowed_on":[],"requires_idempotency_proof":false},"provider":{"identity":"tethers-file-tools","display_name":"Tethers File Tools","identity_source":"host_configuration","description":"Credential-free local reference provider."},"binding":{"kind":"mcp","server_name":"tethers-file-tools","tool_name":METADATA_OPERATION,"adapter":null}})
}

pub fn move_manifest_without_digest() -> Value {
    json!({"manifest_format_version":"1.0","capability_name":MOVE_CAPABILITY,"capability_version":1,"title":"Exact File Move","description":"Move one regular file between two host-approved roots without overwrite.","input_schema":move_input_schema(),"output_schema":move_output_schema(),"effects":["data.read","data.move"],"permission_scope":{"kind":"path_prefix","allowed_prefixes":["source/","destination/"]},"reversibility":"compensatable","determinism":"deterministic","idempotency":{"mechanism":"none"},"confirmation_policy":{"standing_permitted":true,"per_call_required":false},"timeout_ms":5000,"retry_policy":{"max_retries":0,"backoff_ms":0,"allowed_on":[],"requires_idempotency_proof":true},"provider":{"identity":"tethers-file-tools","display_name":"Tethers File Tools","identity_source":"host_configuration","description":"Credential-free local reference provider."},"binding":{"kind":"mcp","server_name":"tethers-file-tools","tool_name":MOVE_OPERATION,"adapter":null}})
}

pub fn metadata_v2_input_schema() -> Value {
    json!({"type":"object","properties":{"path":{"type":"string"},"compute_hash":{"type":"boolean"}},"required":["path"],"additionalProperties":false})
}

pub fn metadata_v2_output_schema() -> Value {
    json!({"type":"object","properties":{"path":{"type":"string"},"name":{"type":"string"},"extension":{"type":["string","null"]},"kind":{"type":"string","enum":["file","directory","other"]},"size_bytes":{"type":["integer","null"],"minimum":0},"modified_unix_ms":{"type":["integer","null"]},"created_unix_ms":{"type":["integer","null"]},"read_only":{"type":"boolean"},"hidden":{"type":"boolean"},"sha256":{"type":["string","null"],"pattern":"^sha256:[a-f0-9]{64}$"}},"required":["path","name","extension","kind","size_bytes","modified_unix_ms","created_unix_ms","read_only","hidden","sha256"],"additionalProperties":false})
}

/// Machine-readable contract projection for file.metadata version 2.
/// The returned value deliberately omits `digest`; callers compute and insert the
/// JCS digest after all authoritative fields are frozen.
pub fn metadata_v2_manifest_without_digest() -> Value {
    json!({"manifest_format_version":"1.0","capability_name":METADATA_CAPABILITY,"capability_version":2,"title":"File Properties Reception Desk","description":"Fast filesystem object identification and cheap property inspection without reading file contents unless a hash is explicitly requested.","input_schema":metadata_v2_input_schema(),"output_schema":metadata_v2_output_schema(),"effects":["metadata.read"],"permission_scope":{"kind":"path_prefix","allowed_prefixes":["query/"]},"reversibility":"reversible","determinism":"deterministic","idempotency":{"mechanism":"none"},"confirmation_policy":{"standing_permitted":true,"per_call_required":false},"timeout_ms":5000,"retry_policy":{"max_retries":0,"backoff_ms":0,"allowed_on":[],"requires_idempotency_proof":false},"provider":{"identity":"tethers-file-tools","display_name":"Tethers File Tools","identity_source":"host_configuration","description":"Credential-free local reference provider."},"binding":{"kind":"mcp","server_name":"tethers-file-tools","tool_name":METADATA_V2_OPERATION,"adapter":null}})
}

pub fn manifest_with_digest(mut manifest: Value) -> Result<Value, FileToolsError> {
    let mut covered = manifest.clone();
    let covered_object = covered
        .as_object_mut()
        .ok_or_else(|| FileToolsError::new("manifest_invalid", "manifest must be an object"))?;
    covered_object.remove("digest");
    covered_object.remove("title");
    covered_object.remove("description");
    let bytes = serde_json_canonicalizer::to_vec(&covered)
        .map_err(|e| FileToolsError::new("manifest_invalid", e.to_string()))?;
    use sha2::{Digest, Sha256};
    manifest.as_object_mut().unwrap().insert(
        "digest".into(),
        Value::String(format!("sha256:{:x}", Sha256::digest(bytes))),
    );
    Ok(manifest)
}

/// Build the deterministic unsigned reference package from the exact provider
/// bytes selected by the host build. ZIP metadata is fixed and never enters
/// semantic identity; the complete payload index does.
pub fn build_reference_package(provider_bytes: &[u8]) -> Result<Vec<u8>, FileToolsError> {
    use sha2::{Digest, Sha256};
    let metadata = serde_json::to_vec(&manifest_with_digest(metadata_manifest_without_digest())?)
        .map_err(|e| FileToolsError::new("package_invalid", e.to_string()))?;
    let metadata_v2 =
        serde_json::to_vec(&manifest_with_digest(metadata_v2_manifest_without_digest())?)
            .map_err(|e| FileToolsError::new("package_invalid", e.to_string()))?;
    let movement = serde_json::to_vec(&manifest_with_digest(move_manifest_without_digest())?)
        .map_err(|e| FileToolsError::new("package_invalid", e.to_string()))?;
    let digest = |bytes: &[u8]| format!("sha256:{:x}", Sha256::digest(bytes));
    let v2_manifest = manifest_with_digest(metadata_v2_manifest_without_digest())?;
    let plug = json!({
        "package_format_version":"1","package_id":"tethers.file-tools","package_version":"1.1.0","display_name":"Tethers File Tools","description":"Credential-free bounded local File Tools reference Plug","publisher":"Tethers reference material","licence":"MIT","socket_major":1,
        "protocol_bindings":[{"protocol":"MCP","version":"2025-11-25","transport":"stdio"}],"platforms":[{"os":"windows","architecture":"x86_64"}],
        "provider":{"provider_id":"tethers-file-tools","provider_version":"1.0.0","launch":{"path":"provider/file_tools_provider.exe","arguments":[]},"working_directory":"provider","capability_operation_namespace":"file","operational_scope_schema":{"type":"object","properties":{"query_root":{"type":"string","x-tethers-path":"canonical-directory"},"move_source_root":{"type":"string","x-tethers-path":"canonical-directory"},"move_destination_root":{"type":"string","x-tethers-path":"canonical-directory"},"max_content_bytes":{"type":"integer","minimum":1,"maximum":65536}},"required":["query_root","move_source_root","move_destination_root","max_content_bytes"],"additionalProperties":false}},
        "capabilities":[{"capability_name":"file.metadata","capability_version":1,"manifest_path":"manifests/file-metadata-local.json","manifest_digest":"sha256:369f4034f702847bb82d1ef82e93f2c5661cad4ad2d7496c3685b406747db09a","provider_operation_name":"file_metadata"},{"capability_name":"file.metadata","capability_version":2,"manifest_path":"manifests/file-metadata-v2.json","manifest_digest":v2_manifest["digest"].as_str().unwrap(),"provider_operation_name":"file_metadata_v2"},{"capability_name":"file.move","capability_version":1,"manifest_path":"manifests/file-move-m4.json","manifest_digest":"sha256:2ac3793d4b61725fd130dac531d9690b93881341245f0c2f7c3aca2fd2dd2311","provider_operation_name":"file_move"}],
        "payload_index":[{"path":"manifests/file-metadata-local.json","sha256":digest(&metadata),"size_bytes":metadata.len(),"role":"capability_manifest"},{"path":"manifests/file-metadata-v2.json","sha256":digest(&metadata_v2),"size_bytes":metadata_v2.len(),"role":"capability_manifest"},{"path":"manifests/file-move-m4.json","sha256":digest(&movement),"size_bytes":movement.len(),"role":"capability_manifest"},{"path":"provider/file_tools_provider.exe","sha256":digest(provider_bytes),"size_bytes":provider_bytes.len(),"role":"provider_executable"}]
    });
    let plug_bytes = serde_json_canonicalizer::to_vec(&plug)
        .map_err(|e| FileToolsError::new("package_invalid", e.to_string()))?;
    let mut archive = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let options = zip::write::FileOptions::<()>::default().last_modified_time(
        zip::DateTime::from_date_and_time(1980, 1, 1, 0, 0, 0)
            .map_err(|e| FileToolsError::new("package_invalid", e.to_string()))?,
    );
    for (path, bytes) in [
        ("plug.json", plug_bytes.as_slice()),
        ("manifests/file-metadata-local.json", metadata.as_slice()),
        ("manifests/file-metadata-v2.json", metadata_v2.as_slice()),
        ("manifests/file-move-m4.json", movement.as_slice()),
        ("provider/file_tools_provider.exe", provider_bytes),
    ] {
        use std::io::Write;
        archive
            .start_file(path, options)
            .map_err(|e| FileToolsError::new("package_invalid", e.to_string()))?;
        archive
            .write_all(bytes)
            .map_err(|e| FileToolsError::new("package_invalid", e.to_string()))?;
    }
    archive
        .finish()
        .map(|cursor| cursor.into_inner())
        .map_err(|e| FileToolsError::new("package_invalid", e.to_string()))
}

/// MCP adapter used only after the normal resolver/policy/intent boundary has
/// produced a `DispatchReadyAction`. It does not classify permission or write
/// Trail records; those remain owned by the shared host execution boundary.
pub struct FileToolsExecutor {
    provider: crate::child_process::SupervisedChild,
    next_request_id: u64,
}

impl FileToolsExecutor {
    pub fn launch_from_installed(
        record: &crate::installed::InstalledPlugRecord,
        installed_directory: &Path,
        trust: &crate::trust::PackageTrustEvidence,
        publisher_trust: &crate::trust::PublisherTrustStore,
        developer_approvals: &crate::trust::DeveloperApprovalStore,
        conformance: &crate::conformance::ConformanceEvidence,
        approval: &crate::installed::InstallationApprovalRecord,
        enablement: &crate::enablement::EnablementRecord,
        scope: &crate::operational_scope::OperationalScopeEvidence,
    ) -> Result<Self, FileToolsError> {
        let mut provider = crate::launch_profile::launch_installed_provider(
            record,
            installed_directory,
            trust,
            publisher_trust,
            developer_approvals,
            conformance,
            approval,
            enablement,
            scope,
        )
        .map_err(|e| FileToolsError::new("provider_launch", e.to_string()))?;
        let initialize = json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"tethers-reference-host","version":"0.2.0"}}});
        let initialized = Self::request_child(&mut provider, &initialize)?;
        if initialized
            .pointer("/result/protocolVersion")
            .and_then(Value::as_str)
            != Some("2025-11-25")
            || initialized
                .pointer("/result/serverInfo/name")
                .and_then(Value::as_str)
                != Some("tethers-file-tools")
        {
            provider.shutdown();
            return Err(FileToolsError::new(
                "provider_protocol",
                "installed provider identity or protocol drifted",
            ));
        }
        provider
            .write_line("{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}")
            .map_err(|e| FileToolsError::new("provider_protocol", e.to_string()))?;
        let tools = Self::request_child(
            &mut provider,
            &json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}),
        )?;
        let tools = tools
            .pointer("/result/tools")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                FileToolsError::new("provider_discovery", "tools/list result missing")
            })?;
        let names = tools
            .iter()
            .filter_map(|tool| tool.get("name").and_then(Value::as_str))
            .collect::<std::collections::BTreeSet<_>>();
        if names.len() != 3
            || !names.contains(METADATA_OPERATION)
            || !names.contains(METADATA_V2_OPERATION)
            || !names.contains(MOVE_OPERATION)
        {
            provider.shutdown();
            return Err(FileToolsError::new(
                "provider_drift",
                "provider advertised an operation outside the reviewed File Tools contract",
            ));
        }
        Ok(Self {
            provider,
            next_request_id: 10,
        })
    }

    fn request_child(
        child: &mut crate::child_process::SupervisedChild,
        request: &Value,
    ) -> Result<Value, FileToolsError> {
        child
            .write_line(
                &serde_json::to_string(request)
                    .map_err(|e| FileToolsError::new("provider_protocol", e.to_string()))?,
            )
            .map_err(|e| FileToolsError::new("provider_protocol", e.to_string()))?;
        let line = child
            .read_protocol_line(Duration::from_secs(5))
            .map_err(|e| FileToolsError::new("provider_protocol", e.to_string()))?;
        serde_json::from_str(&line)
            .map_err(|e| FileToolsError::new("provider_protocol", e.to_string()))
    }

    pub fn call(&mut self, operation: &str, arguments: &Value) -> Result<Value, FileToolsError> {
        if operation != METADATA_OPERATION
            && operation != METADATA_V2_OPERATION
            && operation != MOVE_OPERATION
        {
            return Err(FileToolsError::new(
                "provider_drift",
                "operation is not reviewed",
            ));
        }
        let id = self.next_request_id;
        self.next_request_id = self
            .next_request_id
            .checked_add(1)
            .ok_or_else(|| FileToolsError::new("provider_protocol", "request id exhausted"))?;
        Self::request_child(
            &mut self.provider,
            &json!({"jsonrpc":"2.0","id":id,"method":"tools/call","params":{"name":operation,"arguments":arguments}}),
        )
    }
}

impl crate::executor::CapabilityExecutor for FileToolsExecutor {
    fn provider_identity(&self) -> &str {
        "tethers-file-tools"
    }
    fn execute(&mut self, ready: &crate::dispatch::DispatchReadyAction) -> Result<Value, String> {
        let operation = ready
            .verified_manifest()
            .manifest()
            .binding
            .tool_name
            .as_str();
        self.call(operation, ready.arguments())
            .map_err(|e| e.to_string())
    }
    fn execute_classified(
        &mut self,
        ready: &crate::dispatch::DispatchReadyAction,
        _remaining: Duration,
    ) -> Result<Value, crate::outcome::ProviderDiagnostic> {
        match self.execute(ready) {
            Ok(value) if value.get("error").is_some() => {
                Err(crate::outcome::ProviderDiagnostic::ExplicitProviderError)
            }
            Ok(value) => Ok(value.get("result").cloned().unwrap_or(value)),
            Err(_) => Err(crate::outcome::ProviderDiagnostic::NoFinalResponse),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn scope() -> (std::path::PathBuf, FileScope) {
        let root =
            std::env::temp_dir().join(format!("tethers-m4-file-tools-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        for name in ["query", "source", "destination"] {
            fs::create_dir(root.join(name)).unwrap();
        }
        let scope = FileScope::new(
            &root.join("query"),
            &root.join("source"),
            &root.join("destination"),
        )
        .unwrap();
        (root, scope)
    }

    #[test]
    fn metadata_is_bounded_and_utf8_checked() {
        let (root, scope) = scope();
        let path = scope.query_root.join("note.txt");
        fs::write(&path, b"hello").unwrap();
        assert_eq!(
            metadata(&scope, &json!({"path":"note.txt","include_content":true})).unwrap()
                ["content"],
            "hello"
        );
        assert_eq!(
            metadata(&scope, &json!({"path":"missing","include_content":false}))
                .unwrap_err()
                .code,
            "path_unavailable"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn move_refuses_overwrite_and_preserves_source_on_refusal() {
        let (root, scope) = scope();
        let source = scope.source_root.join("a.txt");
        let destination = scope.destination_root.join("a.txt");
        fs::write(&source, b"source").unwrap();
        fs::write(&destination, b"existing").unwrap();
        assert_eq!(
            move_file(
                &scope,
                &json!({"source_path":"a.txt","destination_path":"a.txt"})
            )
            .unwrap_err()
            .code,
            "overwrite_refused"
        );
        assert_eq!(fs::read(&source).unwrap(), b"source");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn move_success_is_exact_and_non_recursive() {
        let (root, scope) = scope();
        let source = scope.source_root.join("a.txt");
        fs::File::create(&source)
            .unwrap()
            .write_all(b"source")
            .unwrap();
        let result = move_file(
            &scope,
            &json!({"source_path":"a.txt","destination_path":"b.txt"}),
        )
        .unwrap();
        assert_eq!(result["moved"], true);
        assert!(!source.exists());
        assert_eq!(
            fs::read(scope.destination_root.join("b.txt")).unwrap(),
            b"source"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn committed_contract_projections_are_valid_manifests() {
        for manifest in [
            metadata_manifest_without_digest(),
            metadata_v2_manifest_without_digest(),
            move_manifest_without_digest(),
        ] {
            let manifest = manifest_with_digest(manifest).unwrap();
            let text = serde_json::to_string(&manifest).unwrap();
            let verified = crate::manifest::verify_manifest(&text).unwrap();
            println!(
                "{}@{} {}",
                verified.capability_name(),
                verified.capability_version(),
                verified.verified_digest()
            );
        }
        let metadata = crate::manifest::verify_manifest(include_str!(
            "../../protocol/capability-manifests/file-metadata-local.json"
        ))
        .unwrap();
        let metadata_v2 = crate::manifest::verify_manifest(include_str!(
            "../../protocol/capability-manifests/file-metadata-v2.json"
        ))
        .unwrap();
        let movement = crate::manifest::verify_manifest(include_str!(
            "../../protocol/capability-manifests/file-move-m4.json"
        ))
        .unwrap();
        assert_eq!(
            metadata.verified_digest(),
            "sha256:369f4034f702847bb82d1ef82e93f2c5661cad4ad2d7496c3685b406747db09a"
        );
        assert_eq!(metadata_v2.capability_name(), METADATA_CAPABILITY);
        assert_eq!(metadata_v2.capability_version(), 2);
        assert_eq!(
            movement.verified_digest(),
            "sha256:2ac3793d4b61725fd130dac531d9690b93881341245f0c2f7c3aca2fd2dd2311"
        );
    }

    #[test]
    fn reference_package_bytes_are_deterministic_and_inspectable() {
        let first = build_reference_package(b"native-provider-placeholder").unwrap();
        let second = build_reference_package(b"native-provider-placeholder").unwrap();
        assert_eq!(first, second);
        let root =
            std::env::temp_dir().join(format!("tethers-m4-package-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        let archive = root.join("file-tools.tetherplug");
        fs::write(&archive, first).unwrap();
        let report = crate::package::inspect(&archive).unwrap();
        assert_eq!(report.package.package_id, "tethers.file-tools");
        assert_eq!(report.capabilities.len(), 3);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn metadata_v2_reports_kind_file() {
        let (root, scope) = scope();
        fs::write(scope.query_root.join("data.txt"), b"content").unwrap();
        let result = metadata_v2(&scope, &json!({"path":"data.txt"})).unwrap();
        assert_eq!(result["kind"], "file");
        assert_eq!(result["name"], "data.txt");
        assert_eq!(result["extension"], "txt");
        assert!(result["size_bytes"].as_u64().is_some());
        assert!(result["modified_unix_ms"].as_i64().is_some());
        assert!(result["sha256"].is_null());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn metadata_v2_reports_kind_directory() {
        let (root, scope) = scope();
        let dir = scope.query_root.join("subdir");
        fs::create_dir(&dir).unwrap();
        let result = metadata_v2(&scope, &json!({"path":"subdir"})).unwrap();
        assert_eq!(result["kind"], "directory");
        assert_eq!(result["name"], "subdir");
        assert!(result["extension"].is_null());
        assert!(result["size_bytes"].is_null());
        assert!(result["sha256"].is_null());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn metadata_v2_extension_lowercase_no_dot() {
        let (root, scope) = scope();
        fs::write(scope.query_root.join("Readme.MD"), b"hello").unwrap();
        let result = metadata_v2(&scope, &json!({"path":"Readme.MD"})).unwrap();
        assert_eq!(result["extension"], "md");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn metadata_v2_no_extension_returns_null() {
        let (root, scope) = scope();
        fs::write(scope.query_root.join("Makefile"), b"all:").unwrap();
        let result = metadata_v2(&scope, &json!({"path":"Makefile"})).unwrap();
        assert!(result["extension"].is_null());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn metadata_v2_size_null_for_directory() {
        let (root, scope) = scope();
        let dir = scope.query_root.join("emptydir");
        fs::create_dir(&dir).unwrap();
        let result = metadata_v2(&scope, &json!({"path":"emptydir"})).unwrap();
        assert!(result["size_bytes"].is_null());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn metadata_v2_size_exact_for_file() {
        let (root, scope) = scope();
        fs::write(scope.query_root.join("exact.bin"), b"1234567890").unwrap();
        let result = metadata_v2(&scope, &json!({"path":"exact.bin"})).unwrap();
        assert_eq!(result["size_bytes"], 10);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn metadata_v2_modified_time_present() {
        let (root, scope) = scope();
        fs::write(scope.query_root.join("mod.txt"), b"x").unwrap();
        let result = metadata_v2(&scope, &json!({"path":"mod.txt"})).unwrap();
        let ms = result["modified_unix_ms"].as_i64().unwrap();
        assert!(ms > 0);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn metadata_v2_created_time_valid_or_null() {
        let (root, scope) = scope();
        fs::write(scope.query_root.join("created.txt"), b"x").unwrap();
        let result = metadata_v2(&scope, &json!({"path":"created.txt"})).unwrap();
        if let Some(ms) = result["created_unix_ms"].as_i64() {
            assert!(ms > 0);
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn metadata_v2_compute_hash_omitted_defaults_false() {
        let (root, scope) = scope();
        fs::write(scope.query_root.join("default.txt"), b"hello").unwrap();
        let result = metadata_v2(&scope, &json!({"path":"default.txt"})).unwrap();
        assert!(result["sha256"].is_null());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn metadata_v2_compute_hash_false_returns_null() {
        let (root, scope) = scope();
        fs::write(scope.query_root.join("nohash.txt"), b"world").unwrap();
        let result =
            metadata_v2(&scope, &json!({"path":"nohash.txt","compute_hash":false})).unwrap();
        assert!(result["sha256"].is_null());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn metadata_v2_compute_hash_true_returns_deterministic_sha256() {
        let (root, scope) = scope();
        fs::write(scope.query_root.join("hashme.txt"), b"deterministic").unwrap();
        let result =
            metadata_v2(&scope, &json!({"path":"hashme.txt","compute_hash":true})).unwrap();
        let sha = result["sha256"].as_str().unwrap();
        assert!(sha.starts_with("sha256:"));
        assert_eq!(sha.len(), 71);
        assert_eq!(
            sha,
            "sha256:0badac3c6df445ad3aea62da1350683923aba37c685978afed96a515d12921a3"
        );
        let second =
            metadata_v2(&scope, &json!({"path":"hashme.txt","compute_hash":true})).unwrap();
        assert_eq!(result["sha256"], second["sha256"]);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn metadata_v2_hash_directory_refused() {
        let (root, scope) = scope();
        let dir = scope.query_root.join("hashdir");
        fs::create_dir(&dir).unwrap();
        let err = metadata_v2(&scope, &json!({"path":"hashdir","compute_hash":true})).unwrap_err();
        assert_eq!(err.code, "wrong_type");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn metadata_v2_missing_path_refused() {
        let (root, scope) = scope();
        let err = metadata_v2(&scope, &json!({"path":"noexist.txt"})).unwrap_err();
        assert_eq!(err.code, "path_unavailable");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn metadata_v2_scope_violation_refused() {
        let (root, scope) = scope();
        let err = metadata_v2(&scope, &json!({"path":"../escape.txt"})).unwrap_err();
        assert_eq!(err.code, "path_invalid");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn metadata_v2_traversal_refused() {
        let (root, scope) = scope();
        let err = metadata_v2(&scope, &json!({"path":"a/../../escape.txt"})).unwrap_err();
        assert_eq!(err.code, "path_invalid");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn metadata_v2_unknown_field_refused() {
        let (root, scope) = scope();
        let err = metadata_v2(&scope, &json!({"path":"test.txt","extra":"nope"})).unwrap_err();
        assert_eq!(err.code, "arguments_invalid");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn metadata_v2_missing_path_refused_in_validation() {
        let (root, scope) = scope();
        let err = metadata_v2(&scope, &json!({"compute_hash":true})).unwrap_err();
        assert_eq!(err.code, "arguments_invalid");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn metadata_v2_manifest_digest_is_stable() {
        let manifest = manifest_with_digest(metadata_v2_manifest_without_digest()).unwrap();
        let first = manifest["digest"].as_str().unwrap().to_owned();
        let second_manifest = manifest_with_digest(metadata_v2_manifest_without_digest()).unwrap();
        assert_eq!(first, second_manifest["digest"]);
        let text = serde_json::to_string(&manifest).unwrap();
        let verified = crate::manifest::verify_manifest(&text).unwrap();
        assert_eq!(verified.capability_name(), METADATA_CAPABILITY);
        assert_eq!(verified.capability_version(), 2);
        assert_eq!(verified.manifest().binding.tool_name, METADATA_V2_OPERATION);
    }

    #[test]
    fn metadata_v2_committed_frozen_manifest_matches_builder() {
        let built = manifest_with_digest(metadata_v2_manifest_without_digest()).unwrap();
        let file = crate::manifest::verify_manifest(include_str!(
            "../../protocol/capability-manifests/file-metadata-v2.json"
        ))
        .unwrap();
        assert_eq!(file.verified_digest(), built["digest"].as_str().unwrap());
    }

    #[test]
    fn metadata_v2_read_only_reported() {
        let (root, scope) = scope();
        let file_path = scope.query_root.join("ro.txt");
        fs::write(&file_path, b"readonly").unwrap();
        let mut perms = fs::metadata(&file_path).unwrap().permissions();
        perms.set_readonly(true);
        fs::set_permissions(&file_path, perms).unwrap();
        let result = metadata_v2(&scope, &json!({"path":"ro.txt"})).unwrap();
        assert_eq!(result["read_only"], true);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn metadata_v2_file_tools_v1_unchanged() {
        let (root, scope) = scope();
        let path = scope.query_root.join("legacy.txt");
        fs::write(&path, b"v1-data").unwrap();
        let result = metadata(
            &scope,
            &json!({"path":"legacy.txt","include_content":false}),
        )
        .unwrap();
        assert_eq!(result["path"], "legacy.txt");
        assert_eq!(result["is_file"], true);
        assert_eq!(result["size_bytes"], 7);
        assert!(result.get("content").is_none());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn metadata_v2_move_file_not_regressed() {
        let (root, scope) = scope();
        let source = scope.source_root.join("x.txt");
        fs::File::create(&source)
            .unwrap()
            .write_all(b"move-me")
            .unwrap();
        let result = move_file(
            &scope,
            &json!({"source_path":"x.txt","destination_path":"y.txt"}),
        )
        .unwrap();
        assert_eq!(result["moved"], true);
        assert!(!source.exists());
        assert!(scope.destination_root.join("y.txt").exists());
        fs::remove_dir_all(root).unwrap();
    }
}
