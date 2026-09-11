//! The small, host-owned Agent Core surface introduced for Tethers 0.7.
//!
//! This module deliberately sits above the deterministic OCaml planner. It
//! resolves a workspace, checks mechanical scope, performs one explicit local
//! operation, and emits a structured receipt. It is not a second Tethers
//! language or a hidden Threadmoth adapter.

use crate::cli::{
    CapabilityCommand, CliEnvelope, GitCommand, OutcomeStatus, ThreadmothCommand, WorkspaceCommand,
};
use base64::Engine as _;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

const CONFIG_DIR: &str = ".tethers";
const CONFIG_FILE: &str = "config.json";
const DEFAULT_MAX_READ_BYTES: usize = 10 * 1024 * 1024;
const DEFAULT_MAX_WRITE_BYTES: usize = 10 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectConfig {
    schema: String,
    config_version: u32,
    workspace_root: String,
}

#[derive(Debug, Clone)]
pub struct WorkspaceContext {
    pub root: PathBuf,
    pub config_path: PathBuf,
    pub state_root: PathBuf,
    pub workspace_id: String,
}

#[derive(Debug)]
struct CoreError {
    code: &'static str,
    message: String,
    status: OutcomeStatus,
}

impl CoreError {
    fn invalid(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            status: OutcomeStatus::InvalidData,
        }
    }
    fn unavailable(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            status: OutcomeStatus::Unavailable,
        }
    }
    fn denied(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            status: OutcomeStatus::Denied,
        }
    }
    fn failed(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            status: OutcomeStatus::Failed,
        }
    }
}

fn error(command: &str, error: CoreError) -> CliEnvelope {
    CliEnvelope::error(command, error.status, error.code, error.message, None)
}

pub struct CoreResult {
    pub envelope: CliEnvelope,
    pub exit_code: i32,
}

fn result(envelope: CliEnvelope) -> CoreResult {
    let exit_code = match envelope.status {
        OutcomeStatus::Ok
        | OutcomeStatus::Completed
        | OutcomeStatus::Denied
        | OutcomeStatus::NoActions => 0,
        status => status.exit_code(),
    };
    CoreResult {
        envelope,
        exit_code,
    }
}

fn sha256_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn sha256_file(path: &Path) -> Result<String, CoreError> {
    let mut file =
        File::open(path).map_err(|e| CoreError::failed("FILE_OPEN_FAILED", e.to_string()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|e| CoreError::failed("FILE_READ_FAILED", e.to_string()))?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn canonical_existing(path: &Path) -> Result<PathBuf, CoreError> {
    let canonical = fs::canonicalize(path).map_err(|e| {
        CoreError::failed("PATH_RESOLUTION_FAILED", format!("{}: {e}", path.display()))
    })?;
    reject_reparse_chain(&canonical)?;
    Ok(canonical)
}

fn reject_reparse_chain(path: &Path) -> Result<(), CoreError> {
    for ancestor in path.ancestors() {
        let metadata = fs::symlink_metadata(ancestor).map_err(|e| {
            CoreError::failed(
                "PATH_INSPECTION_FAILED",
                format!("{}: {e}", ancestor.display()),
            )
        })?;
        if metadata.file_type().is_symlink() {
            return Err(CoreError::denied(
                "REPARSE_REFUSED",
                format!("reparse point is not permitted: {}", ancestor.display()),
            ));
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
            if metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
                return Err(CoreError::denied(
                    "REPARSE_REFUSED",
                    format!(
                        "Windows reparse point is not permitted: {}",
                        ancestor.display()
                    ),
                ));
            }
        }
    }
    Ok(())
}

fn current_directory() -> Result<PathBuf, CoreError> {
    std::env::current_dir().map_err(|e| CoreError::failed("CWD_UNAVAILABLE", e.to_string()))
}

fn git_root(start: &Path) -> Option<PathBuf> {
    let output = Command::new("git")
        .args([
            "-C",
            &start.to_string_lossy(),
            "rev-parse",
            "--show-toplevel",
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8(output.stdout).ok()?;
    let value = PathBuf::from(text.trim());
    canonical_existing(&value).ok()
}

fn find_config(start: &Path) -> Option<PathBuf> {
    for directory in start.ancestors() {
        let candidate = directory.join(CONFIG_DIR).join(CONFIG_FILE);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn default_state_root() -> Result<PathBuf, CoreError> {
    if let Some(value) = std::env::var_os("TETHERS_HOST_DATA_ROOT") {
        let path = PathBuf::from(value);
        if !path.is_absolute() {
            return Err(CoreError::invalid(
                "HOST_STATE_INVALID",
                "TETHERS_HOST_DATA_ROOT must be absolute",
            ));
        }
        return Ok(path);
    }
    #[cfg(windows)]
    let base = std::env::var_os("LOCALAPPDATA").map(PathBuf::from);
    #[cfg(not(windows))]
    let base = std::env::var_os("XDG_STATE_HOME").map(PathBuf::from);
    let base = base.ok_or_else(|| {
        CoreError::unavailable(
            "HOST_STATE_UNAVAILABLE",
            "no local application-data directory is available",
        )
    })?;
    Ok(base.join("Tethers"))
}

fn normalize_for_identity(path: &Path) -> String {
    let mut value = path.to_string_lossy().replace('\\', "/");
    #[cfg(windows)]
    {
        value = value.to_ascii_lowercase();
    }
    value
}

fn workspace_identity(root: &Path) -> String {
    let git_identity = Command::new("git")
        .args(["-C", &root.to_string_lossy(), "rev-parse", "--git-dir"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|value| {
            let path = PathBuf::from(value.trim());
            let path = if path.is_absolute() {
                path
            } else {
                root.join(path)
            };
            normalize_for_identity(&fs::canonicalize(path).unwrap_or_else(|_| root.to_path_buf()))
        })
        .unwrap_or_else(|| "no-git-worktree".to_owned());
    let mut hasher = Sha256::new();
    hasher.update(b"tethers-workspace-v1\0");
    hasher.update(normalize_for_identity(root).as_bytes());
    hasher.update(b"\0");
    hasher.update(git_identity.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn resolve_workspace() -> Result<WorkspaceContext, CoreError> {
    let cwd = canonical_existing(&current_directory()?)?;
    let config_path = find_config(&cwd).unwrap_or_else(|| cwd.join(CONFIG_DIR).join(CONFIG_FILE));
    let root = if config_path.is_file() {
        let raw = fs::read_to_string(&config_path)
            .map_err(|e| CoreError::invalid("CONFIG_READ_FAILED", e.to_string()))?;
        let config: ProjectConfig = serde_json::from_str(&raw).map_err(|e| {
            CoreError::invalid(
                "CONFIG_INVALID",
                format!("cannot parse {}: {e}", config_path.display()),
            )
        })?;
        if config.schema != "tethers.project/1"
            || config.config_version != 1
            || config.workspace_root != "."
        {
            return Err(CoreError::invalid(
                "CONFIG_INVALID",
                "project config must use schema tethers.project/1 and workspace_root '.'",
            ));
        }
        canonical_existing(config_path.parent().and_then(Path::parent).ok_or_else(|| {
            CoreError::invalid("CONFIG_INVALID", "project config has no workspace root")
        })?)?
    } else {
        git_root(&cwd).unwrap_or(cwd)
    };
    let host_root = default_state_root()?;
    let id = workspace_identity(&root);
    Ok(WorkspaceContext {
        root,
        config_path,
        state_root: host_root.join("data").join("workspaces").join(&id),
        workspace_id: id,
    })
}

fn append_receipt(
    context: &WorkspaceContext,
    capability: &str,
    operation: &str,
    status: &str,
    data: &Value,
) {
    let _ = fs::create_dir_all(&context.state_root);
    let path = context.state_root.join("trail.jsonl");
    let receipt = json!({
        "schema": "tethers.trail/1",
        "receipt_id": format!("receipt_{}", Uuid::new_v4()),
        "timestamp_unix_ms": SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or_default(),
        "workspace_id": context.workspace_id,
        "capability": capability,
        "operation": operation,
        "status": status,
        "data": data,
    });
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "{}", receipt);
    }
}

fn scoped_existing(context: &WorkspaceContext, relative: &Path) -> Result<PathBuf, CoreError> {
    if relative.is_absolute() {
        return Err(CoreError::denied(
            "SCOPE_ESCAPE",
            "workspace paths must be relative",
        ));
    }
    let candidate = context.root.join(relative);
    let resolved = canonical_existing(&candidate)?;
    if !resolved.starts_with(&context.root) {
        return Err(CoreError::denied(
            "SCOPE_ESCAPE",
            "path is outside the workspace root",
        ));
    }
    Ok(resolved)
}

fn scoped_working_directory(context: &WorkspaceContext, path: &Path) -> Result<PathBuf, CoreError> {
    let candidate = if path.is_absolute() {
        path.to_path_buf()
    } else {
        context.root.join(path)
    };
    let resolved = canonical_existing(&candidate)?;
    if !resolved.starts_with(&context.root) {
        return Err(CoreError::denied(
            "SCOPE_ESCAPE",
            "exec cwd is outside the workspace root",
        ));
    }
    Ok(resolved)
}

fn scoped_target(context: &WorkspaceContext, relative: &Path) -> Result<PathBuf, CoreError> {
    if relative.is_absolute() {
        return Err(CoreError::denied(
            "SCOPE_ESCAPE",
            "workspace paths must be relative",
        ));
    }
    let candidate = context.root.join(relative);
    if candidate.exists() {
        return scoped_existing(context, relative);
    }
    let parent = candidate
        .parent()
        .ok_or_else(|| CoreError::denied("SCOPE_ESCAPE", "target has no parent"))?;
    let parent = canonical_existing(parent)?;
    if !parent.starts_with(&context.root) {
        return Err(CoreError::denied(
            "SCOPE_ESCAPE",
            "target parent is outside the workspace root",
        ));
    }
    let name = candidate
        .file_name()
        .ok_or_else(|| CoreError::invalid("PATH_INVALID", "target has no filename"))?;
    Ok(parent.join(name))
}

fn relative_display(context: &WorkspaceContext, path: &Path) -> String {
    path.strip_prefix(&context.root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn content_bytes(
    content: Option<String>,
    content_file: Option<PathBuf>,
) -> Result<Vec<u8>, CoreError> {
    match (content, content_file) {
        (Some(value), None) => Ok(value.into_bytes()),
        (None, Some(path)) => {
            fs::read(path).map_err(|e| CoreError::failed("CONTENT_READ_FAILED", e.to_string()))
        }
        (None, None) => Ok(Vec::new()),
        (Some(_), Some(_)) => Err(CoreError::invalid(
            "INPUT_AMBIGUOUS",
            "choose content or content-file",
        )),
    }
}

fn workspace_stat(context: &WorkspaceContext, path: &Path) -> Result<Value, CoreError> {
    let resolved = match scoped_existing(context, path) {
        Ok(resolved) => resolved,
        Err(error) if error.code == "PATH_RESOLUTION_FAILED" => {
            let target = scoped_target(context, path)?;
            return Ok(json!({
                "path": relative_display(context, &target),
                "exists": false,
                "kind": Value::Null,
                "size_bytes": Value::Null,
                "sha256": Value::Null,
                "readonly": false
            }));
        }
        Err(error) => return Err(error),
    };
    let metadata =
        fs::metadata(&resolved).map_err(|e| CoreError::failed("METADATA_FAILED", e.to_string()))?;
    let kind = if metadata.is_file() {
        "file"
    } else if metadata.is_dir() {
        "directory"
    } else {
        "other"
    };
    let sha = if metadata.is_file() {
        Some(sha256_file(&resolved)?)
    } else {
        None
    };
    Ok(
        json!({"path": relative_display(context, &resolved), "exists": true, "kind": kind, "size_bytes": metadata.is_file().then_some(metadata.len()), "sha256": sha, "readonly": metadata.permissions().readonly()}),
    )
}

fn workspace_list(
    context: &WorkspaceContext,
    path: &Path,
    recursive: bool,
    max_depth: u32,
    max_entries: usize,
) -> Result<Value, CoreError> {
    if max_entries == 0 || max_entries > 100_000 {
        return Err(CoreError::invalid(
            "LIMIT_INVALID",
            "max_entries must be between 1 and 100000",
        ));
    }
    let root = scoped_existing(context, path)?;
    if !root.is_dir() {
        return Err(CoreError::invalid(
            "NOT_DIRECTORY",
            "workspace.list requires a directory",
        ));
    }
    let mut entries = Vec::new();
    let mut pending = vec![(root.clone(), 0u32)];
    while let Some((directory, depth)) = pending.pop() {
        let mut children: Vec<_> = fs::read_dir(&directory)
            .map_err(|e| CoreError::failed("LIST_FAILED", e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| CoreError::failed("LIST_FAILED", e.to_string()))?;
        children.sort_by_key(|entry| entry.file_name());
        for entry in children {
            if entries.len() >= max_entries {
                return Ok(
                    json!({"path": relative_display(context, &root), "entries": entries, "truncated": true, "max_entries": max_entries}),
                );
            }
            let child = entry.path();
            reject_reparse_chain(&child)?;
            let metadata = fs::metadata(&child)
                .map_err(|e| CoreError::failed("LIST_FAILED", e.to_string()))?;
            let is_dir = metadata.is_dir();
            entries.push(json!({"path": relative_display(context, &child), "kind": if is_dir {"directory"} else if metadata.is_file() {"file"} else {"other"}, "size_bytes": metadata.is_file().then_some(metadata.len())}));
            if recursive && is_dir && depth < max_depth {
                pending.push((child, depth + 1));
            }
        }
    }
    Ok(
        json!({"path": relative_display(context, &root), "entries": entries, "truncated": false, "max_entries": max_entries}),
    )
}

fn workspace_read(
    context: &WorkspaceContext,
    path: &Path,
    offset: u64,
    max_bytes: usize,
) -> Result<Value, CoreError> {
    if max_bytes == 0 || max_bytes > DEFAULT_MAX_READ_BYTES {
        return Err(CoreError::invalid(
            "LIMIT_INVALID",
            "max_bytes must be between 1 and 10485760",
        ));
    }
    let resolved = scoped_existing(context, path)?;
    let metadata =
        fs::metadata(&resolved).map_err(|e| CoreError::failed("READ_FAILED", e.to_string()))?;
    if !metadata.is_file() {
        return Err(CoreError::invalid(
            "NOT_FILE",
            "workspace.read requires a file",
        ));
    }
    if offset > metadata.len() {
        return Err(CoreError::invalid(
            "OFFSET_INVALID",
            "offset is beyond end of file",
        ));
    }
    let mut file =
        File::open(&resolved).map_err(|e| CoreError::failed("READ_FAILED", e.to_string()))?;
    file.seek(SeekFrom::Start(offset))
        .map_err(|e| CoreError::failed("READ_FAILED", e.to_string()))?;
    let mut bytes = vec![0u8; max_bytes.saturating_add(1)];
    let count = file
        .read(&mut bytes)
        .map_err(|e| CoreError::failed("READ_FAILED", e.to_string()))?;
    let truncated = count > max_bytes;
    bytes.truncate(count.min(max_bytes));
    let (encoding, content) = match String::from_utf8(bytes.clone()) {
        Ok(value) => ("utf8", Value::String(value)),
        Err(_) => (
            "base64",
            Value::String(base64::engine::general_purpose::STANDARD.encode(bytes)),
        ),
    };
    Ok(
        json!({"path": relative_display(context, &resolved), "total_size_bytes": metadata.len(), "offset": offset, "bytes_returned": count.min(max_bytes), "sha256": sha256_file(&resolved)?, "truncated": truncated, "encoding": encoding, "content": content}),
    )
}

fn workspace_create(
    context: &WorkspaceContext,
    path: &Path,
    content: Vec<u8>,
) -> Result<Value, CoreError> {
    if content.len() > DEFAULT_MAX_WRITE_BYTES {
        return Err(CoreError::invalid(
            "WRITE_LIMIT_EXCEEDED",
            "content exceeds the 10 MiB write limit",
        ));
    }
    let target = scoped_target(context, path)?;
    if target.exists() {
        return Err(CoreError::failed(
            "ALREADY_EXISTS",
            "workspace.create never overwrites an existing path",
        ));
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&target)
        .map_err(|e| CoreError::failed("CREATE_FAILED", e.to_string()))?;
    file.write_all(&content)
        .map_err(|e| CoreError::failed("CREATE_FAILED", e.to_string()))?;
    file.sync_all()
        .map_err(|e| CoreError::failed("CREATE_FAILED", e.to_string()))?;
    Ok(
        json!({"path": relative_display(context, &target), "bytes_written": content.len(), "sha256": sha256_bytes(&content)}),
    )
}

fn normalize_hash(value: &str) -> &str {
    value.strip_prefix("sha256:").unwrap_or(value)
}

fn workspace_replace(
    context: &WorkspaceContext,
    path: &Path,
    expected: &str,
    content: Vec<u8>,
) -> Result<Value, CoreError> {
    if content.len() > DEFAULT_MAX_WRITE_BYTES {
        return Err(CoreError::invalid(
            "WRITE_LIMIT_EXCEEDED",
            "content exceeds the 10 MiB write limit",
        ));
    }
    let target = scoped_existing(context, path)?;
    if !target.is_file() {
        return Err(CoreError::invalid(
            "NOT_FILE",
            "workspace.replace requires an existing file",
        ));
    }
    let actual = sha256_file(&target)?;
    if normalize_hash(expected) != actual {
        return Err(CoreError::denied(
            "PREIMAGE_MISMATCH",
            format!(
                "expected sha256:{}, found sha256:{}; file was not changed",
                normalize_hash(expected),
                actual
            ),
        ));
    }
    let parent = target
        .parent()
        .ok_or_else(|| CoreError::failed("REPLACE_FAILED", "target has no parent"))?;
    let temporary = parent.join(format!(".tethers-replace-{}.tmp", Uuid::new_v4()));
    let write_result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|e| CoreError::failed("REPLACE_FAILED", e.to_string()))?;
        file.write_all(&content)
            .map_err(|e| CoreError::failed("REPLACE_FAILED", e.to_string()))?;
        file.sync_all()
            .map_err(|e| CoreError::failed("REPLACE_FAILED", e.to_string()))?;
        fs::rename(&temporary, &target)
            .map_err(|e| CoreError::failed("REPLACE_FAILED", e.to_string()))?;
        Ok::<(), CoreError>(())
    })();
    if write_result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    write_result?;
    Ok(
        json!({"path": relative_display(context, &target), "bytes_written": content.len(), "sha256": sha256_bytes(&content), "expected_preimage_sha256": actual}),
    )
}

fn workspace_rename(
    context: &WorkspaceContext,
    from: &Path,
    to: &Path,
) -> Result<Value, CoreError> {
    let source = scoped_existing(context, from)?;
    let destination = scoped_target(context, to)?;
    if destination.exists() {
        return Err(CoreError::failed(
            "ALREADY_EXISTS",
            "workspace.rename never overwrites a destination",
        ));
    }
    fs::rename(&source, &destination)
        .map_err(|e| CoreError::failed("RENAME_FAILED", e.to_string()))?;
    Ok(
        json!({"from": relative_display(context, &source), "to": relative_display(context, &destination)}),
    )
}

fn workspace_delete(
    context: &WorkspaceContext,
    path: &Path,
    recursive: bool,
) -> Result<Value, CoreError> {
    let target = scoped_existing(context, path)?;
    if target == context.root {
        return Err(CoreError::denied(
            "ROOT_DELETE_REFUSED",
            "the workspace root cannot be deleted",
        ));
    }
    if target.is_dir() {
        if recursive {
            return Err(CoreError::denied(
                "RECURSIVE_DELETE_UNAVAILABLE",
                "recursive deletion requires a separate future authority",
            ));
        }
        fs::remove_dir(&target).map_err(|e| CoreError::failed("DELETE_FAILED", e.to_string()))?;
    } else {
        fs::remove_file(&target).map_err(|e| CoreError::failed("DELETE_FAILED", e.to_string()))?;
    }
    Ok(json!({"path": relative_display(context, &target), "deleted": true, "recursive": recursive}))
}

fn operation<F>(context: &WorkspaceContext, capability: &str, name: &str, work: F) -> CoreResult
where
    F: FnOnce() -> Result<Value, CoreError>,
{
    match work() {
        Ok(data) => {
            append_receipt(context, capability, name, "success", &data);
            result(CliEnvelope::ok(format!("{capability}.{name}"), data))
        }
        Err(e) => {
            let data = json!({
                "workspace_id": context.workspace_id,
                "error_code": e.code,
                "error_message": e.message,
                "error_status": e.status.as_str()
            });
            append_receipt(context, capability, name, e.status.as_str(), &data);
            result(error(&format!("{capability}.{name}"), e))
        }
    }
}

pub fn run_workspace(command: WorkspaceCommand) -> CoreResult {
    let context = match resolve_workspace() {
        Ok(value) => value,
        Err(e) => return result(error("workspace", e)),
    };
    match command {
        WorkspaceCommand::Stat { path } => operation(&context, "workspace", "stat", || {
            workspace_stat(&context, &path)
        }),
        WorkspaceCommand::List {
            path,
            recursive,
            max_depth,
            max_entries,
        } => operation(&context, "workspace", "list", || {
            workspace_list(&context, &path, recursive, max_depth, max_entries)
        }),
        WorkspaceCommand::Read {
            path,
            offset,
            max_bytes,
        } => operation(&context, "workspace", "read", || {
            workspace_read(&context, &path, offset, max_bytes)
        }),
        WorkspaceCommand::Create {
            path,
            content,
            content_file,
        } => operation(&context, "workspace", "create", || {
            workspace_create(&context, &path, content_bytes(content, content_file)?)
        }),
        WorkspaceCommand::Replace {
            path,
            expected_preimage_sha256,
            content,
            content_file,
        } => operation(&context, "workspace", "replace", || {
            workspace_replace(
                &context,
                &path,
                &expected_preimage_sha256,
                content_bytes(content, content_file)?,
            )
        }),
        WorkspaceCommand::Rename { from, to } => operation(&context, "workspace", "rename", || {
            workspace_rename(&context, &from, &to)
        }),
        WorkspaceCommand::Delete { path, recursive } => {
            operation(&context, "workspace", "delete", || {
                workspace_delete(&context, &path, recursive)
            })
        }
    }
}

fn command_output(mut command: Command) -> Result<(Vec<u8>, Vec<u8>), CoreError> {
    let output = command
        .output()
        .map_err(|e| CoreError::failed("PROCESS_LAUNCH_FAILED", e.to_string()))?;
    if !output.status.success() {
        return Err(CoreError::failed(
            "GIT_FAILED",
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ));
    }
    Ok((output.stdout, output.stderr))
}

fn git_command(
    context: &WorkspaceContext,
    args: &[String],
) -> Result<(Vec<u8>, Vec<u8>), CoreError> {
    let mut command = Command::new("git");
    command
        .current_dir(&context.root)
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env("GIT_TERMINAL_PROMPT", "0");
    command
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    command_output(command)
}

fn hooks_path(context: &WorkspaceContext) -> Result<PathBuf, CoreError> {
    let path = context.state_root.join("empty-hooks");
    fs::create_dir_all(&path)
        .map_err(|e| CoreError::failed("STATE_WRITE_FAILED", e.to_string()))?;
    Ok(path)
}

fn git_status(context: &WorkspaceContext) -> Result<Value, CoreError> {
    let args = vec![
        "status".to_owned(),
        "--porcelain=v1".to_owned(),
        "-z".to_owned(),
        "--branch".to_owned(),
    ];
    let (stdout, _) = git_command(context, &args)?;
    let mut branch = Value::Null;
    let mut staged = Vec::new();
    let mut unstaged = Vec::new();
    let mut untracked = Vec::new();
    for token in stdout.split(|b| *b == 0).filter(|v| !v.is_empty()) {
        let text = String::from_utf8_lossy(token);
        if let Some(value) = text.strip_prefix("## ") {
            branch = json!(value);
            continue;
        }
        if text.len() < 4 {
            continue;
        }
        let code = &text[..2];
        let path = text[3..].to_owned();
        if code == "??" {
            untracked.push(path);
        } else {
            if code.as_bytes()[0] != b' ' {
                staged.push(path.clone());
            }
            if code.as_bytes()[1] != b' ' {
                unstaged.push(path);
            }
        }
    }
    Ok(
        json!({"branch": branch, "staged_paths": staged, "unstaged_paths": unstaged, "untracked_paths": untracked, "conflict": false, "clean": staged.is_empty() && unstaged.is_empty() && untracked.is_empty()}),
    )
}

fn git_diff(
    context: &WorkspaceContext,
    staged: bool,
    path: Option<PathBuf>,
    max_bytes: usize,
) -> Result<Value, CoreError> {
    if max_bytes == 0 || max_bytes > DEFAULT_MAX_READ_BYTES {
        return Err(CoreError::invalid(
            "LIMIT_INVALID",
            "max_bytes must be between 1 and 10485760",
        ));
    }
    let mut args = vec![
        "-c".to_owned(),
        "core.pager=cat".to_owned(),
        "--no-pager".to_owned(),
        "diff".to_owned(),
        "--no-ext-diff".to_owned(),
        "--no-textconv".to_owned(),
    ];
    if staged {
        args.push("--cached".to_owned());
    }
    if let Some(path) = path {
        if path.is_absolute() {
            return Err(CoreError::denied(
                "SCOPE_ESCAPE",
                "Git path must be relative",
            ));
        }
        args.extend(["--".to_owned(), path.to_string_lossy().into_owned()]);
    }
    let (stdout, _) = git_command(context, &args)?;
    let truncated = stdout.len() > max_bytes;
    let bytes = &stdout[..stdout.len().min(max_bytes)];
    Ok(
        json!({"staged": staged, "bytes_returned": bytes.len(), "truncated": truncated, "encoding": "utf8", "diff": String::from_utf8_lossy(bytes)}),
    )
}

fn git_log(context: &WorkspaceContext, limit: usize) -> Result<Value, CoreError> {
    if limit == 0 || limit > 1000 {
        return Err(CoreError::invalid(
            "LIMIT_INVALID",
            "log limit must be between 1 and 1000",
        ));
    }
    let format = "%H%x00%an%x00%aI%x00%s%x1e";
    let args = vec![
        "--no-pager".to_owned(),
        "log".to_owned(),
        format!("--format={format}"),
        "-n".to_owned(),
        limit.to_string(),
    ];
    let (stdout, _) = git_command(context, &args)?;
    let mut commits = Vec::new();
    for record in stdout.split(|b| *b == 0x1e).filter(|v| !v.is_empty()) {
        let fields: Vec<_> = record
            .split(|b| *b == 0)
            .map(|v| String::from_utf8_lossy(v).into_owned())
            .collect();
        if fields.len() >= 4 {
            commits.push(json!({"sha": fields[0], "author": fields[1], "timestamp": fields[2], "subject": fields[3]}));
        }
    }
    Ok(json!({"commits": commits, "limit": limit}))
}

fn git_branch_current(context: &WorkspaceContext) -> Result<Value, CoreError> {
    let args = vec![
        "symbolic-ref".to_owned(),
        "--quiet".to_owned(),
        "--short".to_owned(),
        "HEAD".to_owned(),
    ];
    let output = Command::new("git")
        .current_dir(&context.root)
        .args(&args)
        .output()
        .map_err(|e| CoreError::failed("PROCESS_LAUNCH_FAILED", e.to_string()))?;
    if output.status.success() {
        Ok(json!({"detached": false, "branch": String::from_utf8_lossy(&output.stdout).trim()}))
    } else {
        let (stdout, _) = git_command(context, &["rev-parse".to_owned(), "HEAD".to_owned()])?;
        Ok(
            json!({"detached": true, "branch": Value::Null, "head": String::from_utf8_lossy(&stdout).trim()}),
        )
    }
}

fn git_stage(context: &WorkspaceContext, paths: &[PathBuf]) -> Result<Value, CoreError> {
    if paths.is_empty() {
        return Err(CoreError::invalid(
            "PATHS_REQUIRED",
            "git.stage requires at least one path",
        ));
    }
    for path in paths {
        if path.is_absolute() {
            return Err(CoreError::denied(
                "SCOPE_ESCAPE",
                "Git paths must be relative",
            ));
        }
        let args = vec![
            "check-attr".to_owned(),
            "filter".to_owned(),
            "--".to_owned(),
            path.to_string_lossy().into_owned(),
        ];
        let (stdout, _) = git_command(context, &args)?;
        let text = String::from_utf8_lossy(&stdout);
        if text.lines().any(|line| {
            line.split(':').nth(2).map(str::trim).is_some_and(|value| {
                !value.is_empty() && value != "unspecified" && value != "unset"
            })
        }) {
            return Err(CoreError::unavailable(
                "EXTERNAL_FILTER_UNAVAILABLE",
                format!(
                    "git.stage refused because {} declares an external clean/process filter",
                    path.display()
                ),
            ));
        }
    }
    let hooks = hooks_path(context)?;
    let mut args = vec![
        "-c".to_owned(),
        format!("core.hooksPath={}", hooks.display()),
        "add".to_owned(),
        "--".to_owned(),
    ];
    args.extend(paths.iter().map(|path| path.to_string_lossy().into_owned()));
    let _ = git_command(context, &args)?;
    Ok(
        json!({"staged_paths": paths.iter().map(|p| p.to_string_lossy().replace('\\', "/")).collect::<Vec<_>>(), "hooks_disabled": true, "filters_executed": false}),
    )
}

fn git_commit(context: &WorkspaceContext, message: &str) -> Result<Value, CoreError> {
    if message.trim().is_empty() {
        return Err(CoreError::invalid(
            "MESSAGE_REQUIRED",
            "git.commit requires a non-empty message",
        ));
    }
    let hooks = hooks_path(context)?;
    let args = vec![
        "-c".to_owned(),
        format!("core.hooksPath={}", hooks.display()),
        "-c".to_owned(),
        "commit.gpgSign=false".to_owned(),
        "commit".to_owned(),
        "--no-verify".to_owned(),
        "-m".to_owned(),
        message.to_owned(),
    ];
    let _ = git_command(context, &args)?;
    let (stdout, _) = git_command(context, &["rev-parse".to_owned(), "HEAD".to_owned()])?;
    Ok(
        json!({"commit": String::from_utf8_lossy(&stdout).trim(), "hooks_disabled": true, "signing": false}),
    )
}

fn git_branch_create(context: &WorkspaceContext, name: &str) -> Result<Value, CoreError> {
    let check = Command::new("git")
        .current_dir(&context.root)
        .args(["check-ref-format", "--branch", name])
        .output()
        .map_err(|e| CoreError::failed("PROCESS_LAUNCH_FAILED", e.to_string()))?;
    if !check.status.success() {
        return Err(CoreError::invalid(
            "BRANCH_NAME_INVALID",
            format!("invalid branch name: {name}"),
        ));
    }
    let _ = git_command(
        context,
        &["branch".to_owned(), "--".to_owned(), name.to_owned()],
    )?;
    Ok(json!({"branch": name, "checked_out": false}))
}

pub fn run_git(command: GitCommand) -> CoreResult {
    let context = match resolve_workspace() {
        Ok(value) => value,
        Err(e) => return result(error("git", e)),
    };
    match command {
        GitCommand::Status => operation(&context, "git", "status", || git_status(&context)),
        GitCommand::Diff {
            staged,
            path,
            max_bytes,
        } => operation(&context, "git", "diff", || {
            git_diff(&context, staged, path, max_bytes)
        }),
        GitCommand::Log { limit } => operation(&context, "git", "log", || git_log(&context, limit)),
        GitCommand::BranchCurrent => operation(&context, "git", "branch_current", || {
            git_branch_current(&context)
        }),
        GitCommand::Stage { paths } => {
            operation(&context, "git", "stage", || git_stage(&context, &paths))
        }
        GitCommand::Commit { message } => {
            operation(&context, "git", "commit", || git_commit(&context, &message))
        }
        GitCommand::BranchCreate { name } => operation(&context, "git", "branch_create", || {
            git_branch_create(&context, &name)
        }),
    }
}

fn resolve_executable(program: &str) -> Result<PathBuf, CoreError> {
    let candidate = PathBuf::from(program);
    if candidate.is_absolute() {
        return canonical_existing(&candidate);
    }
    let path = std::env::var_os("PATH").ok_or_else(|| {
        CoreError::unavailable(
            "EXECUTABLE_NOT_FOUND",
            format!("cannot resolve executable '{program}' because PATH is empty"),
        )
    })?;
    for directory in std::env::split_paths(&path) {
        let direct = directory.join(program);
        if direct.is_file() {
            return canonical_existing(&direct);
        }
        #[cfg(windows)]
        if direct.extension().is_none() {
            let exe = direct.with_extension("exe");
            if exe.is_file() {
                return canonical_existing(&exe);
            }
        }
    }
    Err(CoreError::unavailable(
        "EXECUTABLE_NOT_FOUND",
        format!("could not resolve '{program}' to a concrete executable"),
    ))
}

const THREADMOTH_VERSION: &str = "1.9.1";
const THREADMOTH_PROTOCOL: &str = "1.3.1";

fn validate_threadmoth_request(
    context: &WorkspaceContext,
    request: &Path,
) -> Result<(PathBuf, Value), CoreError> {
    let request_path = scoped_existing(context, request)?;
    let metadata = fs::metadata(&request_path)
        .map_err(|e| CoreError::failed("REQUEST_METADATA_FAILED", e.to_string()))?;
    if !metadata.is_file() || metadata.len() > 8 * 1024 * 1024 {
        return Err(CoreError::invalid(
            "REQUEST_INVALID",
            "Threadmoth request must be a regular file no larger than 8 MiB",
        ));
    }
    let raw = fs::read_to_string(&request_path)
        .map_err(|e| CoreError::failed("REQUEST_READ_FAILED", e.to_string()))?;
    let document: Value = serde_json::from_str(&raw)
        .map_err(|e| CoreError::invalid("REQUEST_INVALID", e.to_string()))?;
    let requests: Vec<&Value> =
        if let Some(requests) = document.get("requests").and_then(Value::as_array) {
            requests.iter().collect()
        } else if document.get("file_path").is_some() {
            vec![&document]
        } else {
            return Err(CoreError::invalid(
                "REQUEST_INVALID",
                "request must contain file_path or an array of requests",
            ));
        };
    if requests.is_empty() || requests.len() > 256 {
        return Err(CoreError::invalid(
            "REQUEST_INVALID",
            "Threadmoth request must contain between 1 and 256 operations",
        ));
    }
    for item in requests {
        let file_path = item
            .get("file_path")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                CoreError::invalid("REQUEST_INVALID", "every request needs file_path")
            })?;
        let relative = PathBuf::from(file_path);
        scoped_target(context, &relative)?;
    }
    Ok((request_path, document))
}

fn threadmoth_metadata(executable: &Path) -> Result<(String, String), CoreError> {
    let version = Command::new(executable)
        .arg("--version")
        .output()
        .map_err(|e| CoreError::failed("THREADMOTH_UNAVAILABLE", e.to_string()))?;
    let version_text = String::from_utf8_lossy(&version.stdout).trim().to_owned();
    let version = version_text
        .split_whitespace()
        .last()
        .filter(|value| !value.is_empty())
        .unwrap_or(THREADMOTH_VERSION)
        .to_owned();
    let capabilities = Command::new(executable)
        .args(["capabilities", "--json"])
        .output()
        .map_err(|e| CoreError::failed("THREADMOTH_UNAVAILABLE", e.to_string()))?;
    if !capabilities.status.success() {
        return Err(CoreError::unavailable(
            "THREADMOTH_UNAVAILABLE",
            String::from_utf8_lossy(&capabilities.stderr)
                .trim()
                .to_owned(),
        ));
    }
    let document: Value = serde_json::from_slice(&capabilities.stdout)
        .map_err(|e| CoreError::failed("THREADMOTH_PROTOCOL_INVALID", e.to_string()))?;
    let protocol = document
        .get("protocol_version")
        .and_then(Value::as_str)
        .unwrap_or(THREADMOTH_PROTOCOL)
        .to_owned();
    Ok((version, protocol))
}

fn run_threadmoth_operation(
    context: &WorkspaceContext,
    request: &Path,
    action: &str,
) -> Result<Value, CoreError> {
    let (request_path, document) = validate_threadmoth_request(context, request)?;
    let executable = resolve_executable("threadmoth")?;
    let (version, protocol) = threadmoth_metadata(&executable)?;
    let provider_command = if action == "preview" {
        "preview"
    } else {
        "mutate"
    };
    let output = Command::new(&executable)
        .args([provider_command, "--request"])
        .arg(&request_path)
        .output()
        .map_err(|e| CoreError::failed("THREADMOTH_FAILED", e.to_string()))?;
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    let certificate =
        serde_json::from_str::<Value>(&stdout).unwrap_or_else(|_| json!({"raw_output": stdout}));
    if !output.status.success() {
        return Err(CoreError::failed(
            "THREADMOTH_REFUSED",
            if stderr.is_empty() {
                certificate.to_string()
            } else {
                stderr
            },
        ));
    }
    Ok(json!({
        "provider": "threadmoth",
        "provider_version": version,
        "protocol_version": protocol,
        "action": action,
        "request_path": relative_display(context, &request_path),
        "request_schema": document.get("version"),
        "certificate": certificate
    }))
}

pub fn run_threadmoth(command: ThreadmothCommand) -> CoreResult {
    let context = match resolve_workspace() {
        Ok(value) => value,
        Err(e) => return result(error("threadmoth", e)),
    };
    match command {
        ThreadmothCommand::Preview { request } => {
            operation(&context, "threadmoth", "preview", || {
                run_threadmoth_operation(&context, &request, "preview")
            })
        }
        ThreadmothCommand::Apply { request } => operation(&context, "threadmoth", "apply", || {
            run_threadmoth_operation(&context, &request, "apply")
        }),
    }
}

pub fn run_trail_recent(limit: usize) -> CoreResult {
    let context = match resolve_workspace() {
        Ok(value) => value,
        Err(e) => return result(error("trail", e)),
    };
    if limit == 0 || limit > 1000 {
        return result(error(
            "trail",
            CoreError::invalid("LIMIT_INVALID", "limit must be between 1 and 1000"),
        ));
    }
    let path = context.state_root.join("trail.jsonl");
    let mut recent = Vec::new();
    if let Ok(raw) = fs::read_to_string(&path) {
        for line in raw.lines().filter(|line| !line.trim().is_empty()) {
            if let Ok(value) = serde_json::from_str::<Value>(line) {
                recent.push(value);
                if recent.len() > limit {
                    recent.remove(0);
                }
            }
        }
    }
    result(CliEnvelope::ok(
        "trail",
        json!({"schema":"tethers.trail/1", "limit":limit, "executions":recent}),
    ))
}

pub fn run_trail_id(execution_id: &str) -> CoreResult {
    let context = match resolve_workspace() {
        Ok(value) => value,
        Err(e) => return result(error("trail", e)),
    };
    let trail = context.state_root.join("trail.jsonl");
    if execution_id.starts_with("receipt_") {
        let receipt = fs::read_to_string(&trail).ok().and_then(|raw| {
            raw.lines().rev().find_map(|line| {
                let value = serde_json::from_str::<Value>(line).ok()?;
                (value.get("receipt_id").and_then(Value::as_str) == Some(execution_id))
                    .then_some(value)
            })
        });
        return match receipt {
            Some(value) => result(CliEnvelope::ok("trail", json!({"receipt": value}))),
            None => result(error(
                "trail",
                CoreError::invalid(
                    "RECEIPT_NOT_FOUND",
                    "receipt ID was not found in the workspace Trail",
                ),
            )),
        };
    }
    let inspection = crate::trail_command::run_trail(&trail, execution_id);
    let data = serde_json::from_str::<Value>(&inspection.json_output)
        .unwrap_or_else(|_| json!({"raw": inspection.json_output}));
    result(CliEnvelope {
        schema: "tethers.cli/1",
        command: "trail".to_owned(),
        status: if inspection.exit_code == 0 {
            OutcomeStatus::Ok
        } else {
            OutcomeStatus::Failed
        },
        exit_code: inspection.exit_code,
        data,
        error: None,
    })
}

pub fn run_simple_trail(arguments: &[String]) -> CoreResult {
    let mut limit = 20usize;
    let mut execution_id = None;
    let mut index = 0;
    while index < arguments.len() {
        let argument = &arguments[index];
        if argument == "--limit" {
            index += 1;
            let value = arguments
                .get(index)
                .ok_or_else(|| CoreError::invalid("LIMIT_INVALID", "--limit requires a value"));
            match value.and_then(|value| {
                value
                    .parse::<usize>()
                    .map_err(|_| CoreError::invalid("LIMIT_INVALID", "--limit must be an integer"))
            }) {
                Ok(value) => limit = value,
                Err(core_error) => return result(error("trail", core_error)),
            }
        } else if let Some(value) = argument.strip_prefix("--limit=") {
            match value.parse::<usize>() {
                Ok(value) => limit = value,
                Err(_) => {
                    return result(error(
                        "trail",
                        CoreError::invalid("LIMIT_INVALID", "--limit must be an integer"),
                    ))
                }
            }
        } else if argument == "--help" || argument == "-h" {
            return result(CliEnvelope::ok(
                "trail",
                json!({"usage":"tethers trail [EXECUTION_ID] [--limit N]"}),
            ));
        } else if argument.starts_with('-') {
            return result(error(
                "trail",
                CoreError::invalid(
                    "INVALID_CLI_USAGE",
                    format!("unexpected argument '{argument}'"),
                ),
            ));
        } else if execution_id.replace(argument.clone()).is_some() {
            return result(error(
                "trail",
                CoreError::invalid("INVALID_CLI_USAGE", "only one execution ID is accepted"),
            ));
        }
        index += 1;
    }
    match execution_id {
        Some(id) => run_trail_id(&id),
        None => run_trail_recent(limit),
    }
}

fn capture_bounded(mut reader: impl Read, limit: usize) -> (Vec<u8>, bool) {
    let mut bytes = Vec::with_capacity(limit.saturating_add(1));
    let mut buffer = [0u8; 16 * 1024];
    let mut total = 0usize;
    let mut truncated = false;
    loop {
        match reader.read(&mut buffer) {
            Ok(0) | Err(_) => break,
            Ok(count) => {
                total = total.saturating_add(count);
                if bytes.len() < limit {
                    let remaining = limit - bytes.len();
                    bytes.extend_from_slice(&buffer[..count.min(remaining)]);
                }
                truncated = total > limit;
            }
        }
    }
    (bytes, truncated)
}

fn run_exec(
    context: &WorkspaceContext,
    program: &str,
    argv: &[String],
    cwd: Option<PathBuf>,
    timeout_ms: u64,
    environment: &str,
    env: &[String],
    max_output_bytes: usize,
) -> Result<Value, CoreError> {
    if timeout_ms == 0 || timeout_ms > 86_400_000 {
        return Err(CoreError::invalid(
            "TIMEOUT_INVALID",
            "timeout-ms must be between 1 and 86400000",
        ));
    }
    if max_output_bytes == 0 || max_output_bytes > DEFAULT_MAX_READ_BYTES {
        return Err(CoreError::invalid(
            "LIMIT_INVALID",
            "max-output-bytes must be between 1 and 10485760",
        ));
    }
    let executable = resolve_executable(program)?;
    let working_directory = match cwd {
        Some(path) => scoped_working_directory(context, &path)?,
        None => context.root.clone(),
    };
    if !working_directory.is_dir() {
        return Err(CoreError::invalid(
            "CWD_INVALID",
            "exec cwd must be a directory",
        ));
    }
    if !matches!(environment, "inherit" | "clean" | "explicit") {
        return Err(CoreError::invalid(
            "ENVIRONMENT_INVALID",
            "environment must be inherit, clean, or explicit",
        ));
    }
    let mut explicit = BTreeMap::new();
    for item in env {
        let (key, value) = item.split_once('=').ok_or_else(|| {
            CoreError::invalid(
                "ENVIRONMENT_INVALID",
                format!("invalid --env entry: {item}"),
            )
        })?;
        if key.is_empty() {
            return Err(CoreError::invalid(
                "ENVIRONMENT_INVALID",
                "environment names cannot be empty",
            ));
        }
        explicit.insert(key.to_owned(), value.to_owned());
    }
    if environment == "explicit" && explicit.is_empty() {
        return Err(CoreError::invalid(
            "ENVIRONMENT_INVALID",
            "explicit environment mode requires at least one --env entry",
        ));
    }
    let mut command = Command::new(&executable);
    command
        .args(argv)
        .current_dir(&working_directory)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if environment == "clean" || environment == "explicit" {
        command.env_clear();
    }
    for (key, value) in &explicit {
        command.env(key, value);
    }
    let mut child = command
        .spawn()
        .map_err(|e| CoreError::failed("PROCESS_LAUNCH_FAILED", e.to_string()))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| CoreError::failed("PROCESS_IO_FAILED", "stdout was unavailable"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| CoreError::failed("PROCESS_IO_FAILED", "stderr was unavailable"))?;
    let output_limit = max_output_bytes;
    let stdout_thread = thread::spawn(move || capture_bounded(stdout, output_limit));
    let stderr_thread = thread::spawn(move || capture_bounded(stderr, output_limit));
    let deadline = std::time::Instant::now() + Duration::from_millis(timeout_ms);
    let mut timed_out = false;
    loop {
        if child
            .try_wait()
            .map_err(|e| CoreError::failed("PROCESS_WAIT_FAILED", e.to_string()))?
            .is_some()
        {
            break;
        }
        if std::time::Instant::now() >= deadline {
            timed_out = true;
            #[cfg(windows)]
            {
                let pid = child.id().to_string();
                let windir = std::env::var_os("WINDIR").unwrap_or_else(|| "C:\\Windows".into());
                let _ = Command::new(PathBuf::from(windir).join("System32\\taskkill.exe"))
                    .args(["/PID", &pid, "/T", "/F"])
                    .status();
            }
            let _ = child.kill();
            let _ = child.wait();
            break;
        }
        thread::sleep(Duration::from_millis(5));
    }
    let status = child
        .wait()
        .map_err(|e| CoreError::failed("PROCESS_WAIT_FAILED", e.to_string()))?;
    let (stdout, stdout_truncated) = stdout_thread.join().unwrap_or_default();
    let (stderr, stderr_truncated) = stderr_thread.join().unwrap_or_default();
    let executable_sha256 = sha256_file(&executable).ok();
    Ok(
        json!({"requested_program": program, "resolved_program": executable, "resolved_program_sha256": executable_sha256, "argv": argv, "cwd": working_directory, "environment_mode": environment, "exit_code": status.code(), "timed_out": timed_out, "supervision": if cfg!(windows) {"process_tree_best_effort_taskkill"} else {"child_process_only"}, "stdout": String::from_utf8_lossy(&stdout), "stderr": String::from_utf8_lossy(&stderr), "stdout_bytes": stdout.len(), "stderr_bytes": stderr.len(), "stdout_truncated": stdout_truncated, "stderr_truncated": stderr_truncated}),
    )
}

pub fn run_exec_command(
    program: String,
    argv: Vec<String>,
    cwd: Option<PathBuf>,
    timeout_ms: u64,
    environment: String,
    env: Vec<String>,
    max_output_bytes: usize,
) -> CoreResult {
    let context = match resolve_workspace() {
        Ok(value) => value,
        Err(e) => return result(error("exec", e)),
    };
    operation(&context, "exec", "run", || {
        run_exec(
            &context,
            &program,
            &argv,
            cwd,
            timeout_ms,
            &environment,
            &env,
            max_output_bytes,
        )
    })
}

pub fn run_init(engine_override: Option<PathBuf>) -> CoreResult {
    let cwd = match current_directory().and_then(|value| canonical_existing(&value)) {
        Ok(value) => value,
        Err(e) => return result(error("init", e)),
    };
    let root = git_root(&cwd).unwrap_or(cwd);
    let directory = root.join(CONFIG_DIR);
    let path = directory.join(CONFIG_FILE);
    if let Err(e) = fs::create_dir_all(&directory) {
        return result(error(
            "init",
            CoreError::failed("CONFIG_CREATE_FAILED", e.to_string()),
        ));
    }
    let initialized = if path.exists() {
        let raw = match fs::read_to_string(&path) {
            Ok(value) => value,
            Err(e) => {
                return result(error(
                    "init",
                    CoreError::invalid("CONFIG_READ_FAILED", e.to_string()),
                ))
            }
        };
        if let Err(e) = serde_json::from_str::<ProjectConfig>(&raw)
            .map_err(|e| e.to_string())
            .and_then(|config| {
                if config.schema == "tethers.project/1"
                    && config.config_version == 1
                    && config.workspace_root == "."
                {
                    Ok(())
                } else {
                    Err("unsupported project config".to_owned())
                }
            })
        {
            return result(error("init", CoreError::invalid("CONFIG_INVALID", e)));
        }
        false
    } else {
        let config = ProjectConfig {
            schema: "tethers.project/1".to_owned(),
            config_version: 1,
            workspace_root: ".".to_owned(),
        };
        let bytes = serde_json::to_vec_pretty(&config).unwrap_or_else(|_| b"{}".to_vec());
        if let Err(e) = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .and_then(|mut file| file.write_all(&bytes))
        {
            return result(error(
                "init",
                CoreError::failed("CONFIG_CREATE_FAILED", e.to_string()),
            ));
        }
        true
    };
    let host_root = match default_state_root() {
        Ok(value) => value,
        Err(e) => return result(error("init", e)),
    };
    let id = workspace_identity(&root);
    let state = host_root.join("data").join("workspaces").join(&id);
    if let Err(e) = fs::create_dir_all(&state) {
        return result(error(
            "init",
            CoreError::failed("STATE_CREATE_FAILED", e.to_string()),
        ));
    }
    let engine = discover_engine(engine_override.as_deref());
    result(CliEnvelope::ok(
        "init",
        json!({"workspace_root": root, "workspace_id": id, "config_path": path, "config_created": initialized, "host_state_root": state, "engine": engine, "ready": true}),
    ))
}

pub fn discover_engine(explicit: Option<&Path>) -> Value {
    let mut candidates = Vec::new();
    if let Some(path) = explicit {
        candidates.push(("explicit_override", path.to_path_buf()));
    }
    if let Some(value) = std::env::var_os("TETHERS_ENGINE") {
        candidates.push(("environment_override", PathBuf::from(value)));
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            candidates.push((
                "sibling",
                parent.join(if cfg!(windows) {
                    "tethers-engine.exe"
                } else {
                    "tethers-engine"
                }),
            ));
        }
    }
    for (source, path) in &candidates {
        if path.is_file() {
            return json!({"available": true, "source": source, "path": path, "sha256": sha256_file(path).ok()});
        }
    }
    let expected = candidates
        .last()
        .map(|(_, path)| path.to_string_lossy().into_owned())
        .unwrap_or_else(|| "tethers-engine beside the host executable".to_owned());
    json!({"available": false, "expected": expected, "looked_at": candidates.iter().map(|(_, path)| path).collect::<Vec<_>>(), "repair": "Reinstall Tethers with tethers-engine.exe beside tethers.exe or provide --engine <path>."})
}

pub fn run_doctor(engine_override: Option<PathBuf>) -> CoreResult {
    let context = match resolve_workspace() {
        Ok(value) => value,
        Err(e) => return result(error("doctor", e)),
    };
    let config = if context.config_path.is_file() {
        match fs::read_to_string(&context.config_path)
            .ok()
            .and_then(|raw| serde_json::from_str::<ProjectConfig>(&raw).ok())
        {
            Some(_) => json!({"path": context.config_path, "valid": true}),
            None => {
                json!({"path": context.config_path, "valid": false, "repair": "Run tethers init after repairing .tethers/config.json."})
            }
        }
    } else {
        json!({"path": context.config_path, "valid": false, "repair": "Run: tethers init"})
    };
    let state_probe = context
        .state_root
        .join(format!(".doctor-write-{}", Uuid::new_v4()));
    let writable = if context.state_root.is_dir() {
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&state_probe)
        {
            Ok(_) => {
                let _ = fs::remove_file(&state_probe);
                true
            }
            Err(_) => false,
        }
    } else {
        false
    };
    let state = json!({"path": context.state_root, "exists": context.state_root.is_dir(), "writable": writable});
    let engine = discover_engine(engine_override.as_deref());
    let healthy = config["valid"] == true && state["exists"] == true && engine["available"] == true;
    result(CliEnvelope::ok(
        "doctor",
        json!({"healthy": healthy, "host_version": env!("CARGO_PKG_VERSION"), "engine": engine, "project_configuration": config, "host_state": state, "workspace": {"root": context.root, "id": context.workspace_id}, "capability_registry": {"families": 3, "threadmoth": "optional"}, "trail": {"path": context.state_root.join("trail.jsonl")}, "repair": (!healthy).then_some("Run tethers init and ensure the packaged sibling engine is present.")}),
    ))
}

pub fn run_describe(engine_override: Option<PathBuf>) -> CoreResult {
    let context = match resolve_workspace() {
        Ok(value) => value,
        Err(e) => return result(error("describe", e)),
    };
    let engine = discover_engine(engine_override.as_deref());
    result(CliEnvelope::ok(
        "describe",
        json!({
            "schema":"tethers.describe/1",
            "name":"Tethers",
            "version":env!("CARGO_PKG_VERSION"),
            "mode":"execution-boundary",
            "engine":engine,
            "workspace":{"root":context.root,"id":context.workspace_id},
            "host_state":context.state_root,
            "capabilities":["workspace","git","exec","threadmoth (optional)"]
        }),
    ))
}

fn capability_descriptor(name: &str) -> Option<Value> {
    let workspace = json!({"name":"workspace","family":"tethers.workspace","description":"Bounded filesystem operations rooted at the resolved workspace.","operations":["workspace.stat","workspace.list","workspace.read","workspace.create","workspace.replace","workspace.rename","workspace.delete"],"mechanical_scope":{"allowed_roots":["<canonical workspace root>"],"max_read_bytes":DEFAULT_MAX_READ_BYTES,"max_write_bytes":DEFAULT_MAX_WRITE_BYTES,"allow_recursive_delete":false}});
    let git = json!({"name":"git","family":"tethers.git","description":"Structured local Git inspection and conservative mutation.","operations":["git.status","git.diff","git.log","git.branch_current","git.stage","git.commit","git.branch_create"],"repository_scope":"resolved workspace worktree","remote_mutation":false,"repository_hooks":"disabled through a Tethers-owned empty hooks path"});
    let exec = json!({"name":"exec","family":"tethers.exec","description":"Direct program plus argv execution with explicit environment and bounded output.","operations":["exec.run"],"shell_inserted":false,"environment_modes":["inherit","clean","explicit"],"timeout_ms_max":86400000});
    let threadmoth = threadmoth_descriptor();
    let normalized = name.strip_prefix("tethers.").unwrap_or(name);
    match normalized {
        "workspace" => Some(workspace),
        "git" => Some(git),
        "exec" => Some(exec),
        "threadmoth" => Some(threadmoth),
        "workspace.stat" => Some(
            json!({"name":"workspace.stat","family":"tethers.workspace","input":{"path":"relative workspace path"},"output":{"exists":"boolean","kind":"file|directory|other|null","size_bytes":"integer|null","sha256":"string|null","readonly":"boolean"}}),
        ),
        "workspace.list" => Some(
            json!({"name":"workspace.list","family":"tethers.workspace","input":{"path":"relative path","recursive":"boolean","max_depth":"integer","max_entries":"integer"},"output":{"entries":"bounded array","truncated":"boolean"}}),
        ),
        "workspace.read" => Some(
            json!({"name":"workspace.read","family":"tethers.workspace","input":{"path":"relative file path","offset":"integer","max_bytes":"integer <= 10485760"},"output":{"content":"utf8 or base64","bytes_returned":"integer","sha256":"string","truncated":"boolean"}}),
        ),
        "workspace.create" | "workspace.replace" | "workspace.rename" | "workspace.delete" => Some(
            json!({"name":name,"family":"tethers.workspace","authority":"host scope plus policy","note":"Use the matching workspace subcommand; replace requires expected_preimage_sha256 and delete does not grant recursive authority."}),
        ),
        "git.status" | "git.diff" | "git.log" | "git.branch_current" | "git.stage"
        | "git.commit" | "git.branch_create" => Some(
            json!({"name":name,"family":"tethers.git","authority":"resolved local worktree","remote_mutation":false,"hidden_hooks":false}),
        ),
        "threadmoth.preview" | "threadmoth.apply" => Some(
            json!({"name":name,"family":"threadmoth","input":{"request":"workspace-relative Threadmoth protocol JSON"},"output":{"provider":"threadmoth","provider_version":"string","protocol_version":"string","certificate":"object"},"guarded":true,"hidden_fallback":false}),
        ),
        "exec.run" => Some(exec),
        _ => None,
    }
}

fn threadmoth_descriptor() -> Value {
    match resolve_executable("threadmoth").ok().and_then(|path| {
        threadmoth_metadata(&path)
            .ok()
            .map(|(version, protocol)| (version, protocol))
    }) {
        Some((version, protocol)) => json!({
            "name":"threadmoth",
            "family":"threadmoth",
            "description":"Optional explicit structural mutation adapter.",
            "operations":["threadmoth.preview","threadmoth.apply"],
            "status":"available",
            "provider_version":version,
            "protocol_version":protocol,
            "guarded":true,
            "hidden_fallback":false
        }),
        None => json!({
            "name":"threadmoth",
            "family":"threadmoth",
            "description":"Optional explicit structural mutation adapter.",
            "operations":["threadmoth.preview","threadmoth.apply"],
            "status":"unavailable",
            "repair":"Install a compatible Threadmoth protocol provider."
        }),
    }
}

pub fn run_capability(command: CapabilityCommand) -> CoreResult {
    match command {
        CapabilityCommand::List => result(CliEnvelope::ok(
            "capability list",
            json!({"schema":"tethers.capabilities/1","policy_decisions":["ALLOW","ASK","DENY","UNAVAILABLE"],"families":[capability_descriptor("workspace").unwrap(), capability_descriptor("git").unwrap(), capability_descriptor("exec").unwrap()],"optional":[capability_descriptor("threadmoth").unwrap()]}),
        )),
        CapabilityCommand::Inspect { name } => match capability_descriptor(&name) {
            Some(data) => result(CliEnvelope::ok("capability inspect", data)),
            None => result(error(
                "capability inspect",
                CoreError::invalid(
                    "CAPABILITY_NOT_FOUND",
                    format!("unknown capability operation: {name}"),
                ),
            )),
        },
    }
}
