//! Bounded workspace operations for the Agent Essentials provider.
//!
//! This module is intentionally provider-local. The host still owns Plug
//! trust, policy, operational scope, and Trail admission; these functions only
//! enforce the provider's second boundary over already-approved roots.

use crate::file_tools::FileScope;
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

pub const FILESYSTEM_READ: &str = "filesystem_read";
pub const FILESYSTEM_LIST: &str = "filesystem_list";
pub const FILESYSTEM_STAT: &str = "filesystem_stat";
pub const TEXT_SEARCH: &str = "text_search";
pub const TEXT_READ_RANGE: &str = "text_read_range";
pub const TEXT_REPLACE_EXACT: &str = "text_replace_exact";
pub const TEXT_COMPARE: &str = "text_compare";
pub const PATCH_APPLY: &str = "patch_apply";
pub const HASH_SHA256: &str = "hash_sha256";
pub const HASH_VERIFY: &str = "hash_verify";
pub const HASH_DIRECTORY_MANIFEST: &str = "hash_directory_manifest";
pub const MAX_ENTRIES: usize = 10_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceError {
    pub code: &'static str,
    pub message: String,
}

impl WorkspaceError {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for WorkspaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}
impl std::error::Error for WorkspaceError {}

fn object<'a>(
    arguments: &'a Value,
    required: &[&str],
    optional: &[&str],
) -> Result<&'a Map<String, Value>, WorkspaceError> {
    let value = arguments
        .as_object()
        .ok_or_else(|| WorkspaceError::new("arguments_invalid", "arguments must be an object"))?;
    if required.iter().any(|key| !value.contains_key(*key))
        || value
            .keys()
            .any(|key| !required.contains(&key.as_str()) && !optional.contains(&key.as_str()))
    {
        return Err(WorkspaceError::new(
            "arguments_invalid",
            "unknown or missing workspace argument",
        ));
    }
    Ok(value)
}

fn string<'a>(value: &'a Value, field: &'static str) -> Result<&'a str, WorkspaceError> {
    let text = value.as_str().ok_or_else(|| {
        WorkspaceError::new("arguments_invalid", format!("{field} must be a string"))
    })?;
    if text.is_empty() {
        return Err(WorkspaceError::new(
            "arguments_invalid",
            format!("{field} must not be empty"),
        ));
    }
    Ok(text)
}

fn text_value<'a>(value: &'a Value, field: &'static str) -> Result<&'a str, WorkspaceError> {
    value.as_str().ok_or_else(|| {
        WorkspaceError::new("arguments_invalid", format!("{field} must be a string"))
    })
}

fn bounded_u64(value: &Value, field: &'static str, max: u64) -> Result<u64, WorkspaceError> {
    let number = value.as_u64().ok_or_else(|| {
        WorkspaceError::new(
            "arguments_invalid",
            format!("{field} must be an unsigned integer"),
        )
    })?;
    if number == 0 || number > max {
        return Err(WorkspaceError::new(
            "resource_limit",
            format!("{field} exceeds the configured bound"),
        ));
    }
    Ok(number)
}

fn safe_relative(
    raw: &str,
    field: &'static str,
    allow_root: bool,
) -> Result<String, WorkspaceError> {
    if raw.len() > 240
        || raw.contains('\\')
        || raw.contains(':')
        || raw.contains('\0')
        || raw.starts_with('/')
    {
        return Err(WorkspaceError::new(
            "path_invalid",
            format!("{field} must be a bounded relative slash path"),
        ));
    }
    if raw.is_empty() || (allow_root && raw == ".") {
        return Ok(String::new());
    }
    if raw
        .split('/')
        .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err(WorkspaceError::new(
            "path_invalid",
            format!("{field} contains an unsafe segment"),
        ));
    }
    Ok(raw.to_owned())
}

fn existing(
    root: &Path,
    raw: &Value,
    field: &'static str,
    allow_root: bool,
) -> Result<(String, PathBuf), WorkspaceError> {
    let relative = safe_relative(string(raw, field)?, field, allow_root)?;
    let full = if relative.is_empty() {
        root.to_path_buf()
    } else {
        root.join(relative.replace('/', std::path::MAIN_SEPARATOR_STR))
    };
    let canonical = fs::canonicalize(&full)
        .map_err(|e| WorkspaceError::new("path_unavailable", format!("{field}: {e}")))?;
    if !canonical.starts_with(root) {
        return Err(WorkspaceError::new(
            "scope_violation",
            format!("{field} is outside its approved root"),
        ));
    }
    reject_reparse_chain(&canonical)?;
    Ok((relative, canonical))
}

fn reject_reparse_chain(path: &Path) -> Result<(), WorkspaceError> {
    for ancestor in path.ancestors() {
        if let Ok(metadata) = fs::symlink_metadata(ancestor) {
            if metadata.file_type().is_symlink() {
                return Err(WorkspaceError::new(
                    "reparse_refused",
                    "symbolic link encountered",
                ));
            }
        }
    }
    Ok(())
}

fn read_bounded(path: &Path, max_bytes: u64) -> Result<Vec<u8>, WorkspaceError> {
    let metadata =
        fs::metadata(path).map_err(|e| WorkspaceError::new("file_unavailable", e.to_string()))?;
    if !metadata.is_file() {
        return Err(WorkspaceError::new(
            "wrong_type",
            "path is not a regular file",
        ));
    }
    if metadata.len() > max_bytes {
        return Err(WorkspaceError::new(
            "file_too_large",
            "file exceeds the configured output bound",
        ));
    }
    let mut file =
        fs::File::open(path).map_err(|e| WorkspaceError::new("file_read_failed", e.to_string()))?;
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.read_to_end(&mut bytes)
        .map_err(|e| WorkspaceError::new("file_read_failed", e.to_string()))?;
    Ok(bytes)
}

pub fn read(scope: &FileScope, arguments: &Value) -> Result<Value, WorkspaceError> {
    let args = object(arguments, &["path", "max_bytes"], &[])?;
    let max = bounded_u64(
        args.get("max_bytes").unwrap(),
        "max_bytes",
        scope.max_content_bytes,
    )?;
    let (path, full) = existing(&scope.query_root, args.get("path").unwrap(), "path", false)?;
    let bytes = read_bounded(&full, max)?;
    let content = String::from_utf8(bytes.clone())
        .map_err(|_| WorkspaceError::new("invalid_utf8", "file content is not valid UTF-8"))?;
    Ok(json!({"path": path, "content": content, "bytes_read": bytes.len()}))
}

fn kind(metadata: &fs::Metadata) -> &'static str {
    if metadata.is_file() {
        "file"
    } else if metadata.is_dir() {
        "directory"
    } else {
        "other"
    }
}

pub fn stat(scope: &FileScope, arguments: &Value) -> Result<Value, WorkspaceError> {
    let args = object(arguments, &["path"], &[])?;
    let (path, full) = existing(&scope.query_root, args.get("path").unwrap(), "path", true)?;
    let metadata =
        fs::metadata(&full).map_err(|e| WorkspaceError::new("path_unavailable", e.to_string()))?;
    Ok(
        json!({"path": path, "kind": kind(&metadata), "size_bytes": if metadata.is_file() { Some(metadata.len()) } else { None }}),
    )
}

pub fn list(scope: &FileScope, arguments: &Value) -> Result<Value, WorkspaceError> {
    let args = object(arguments, &[], &["path"])?;
    let (prefix, full) = match args.get("path") {
        Some(path) => existing(&scope.query_root, path, "path", true)?,
        None => (String::new(), scope.query_root.clone()),
    };
    let metadata =
        fs::metadata(&full).map_err(|e| WorkspaceError::new("path_unavailable", e.to_string()))?;
    if !metadata.is_dir() {
        return Err(WorkspaceError::new("wrong_type", "path is not a directory"));
    }
    let mut entries = Vec::new();
    for item in fs::read_dir(&full)
        .map_err(|e| WorkspaceError::new("directory_read_failed", e.to_string()))?
    {
        let item = item.map_err(|e| WorkspaceError::new("directory_read_failed", e.to_string()))?;
        let name = item
            .file_name()
            .to_str()
            .ok_or_else(|| {
                WorkspaceError::new("invalid_utf8", "directory entry name is not valid UTF-8")
            })?
            .to_owned();
        let child = item.path();
        reject_reparse_chain(&child)?;
        let metadata = fs::metadata(&child)
            .map_err(|e| WorkspaceError::new("path_unavailable", e.to_string()))?;
        let relative = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{prefix}/{name}")
        };
        entries.push((name, json!({"path": relative, "kind": kind(&metadata), "size_bytes": if metadata.is_file() { Some(metadata.len()) } else { None }})));
        if entries.len() > MAX_ENTRIES {
            return Err(WorkspaceError::new(
                "resource_limit",
                "directory entry count exceeds the configured bound",
            ));
        }
    }
    entries.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(
        json!({"path": prefix, "entries": entries.into_iter().map(|(_, value)| value).collect::<Vec<_>>() }),
    )
}

pub fn search(scope: &FileScope, arguments: &Value) -> Result<Value, WorkspaceError> {
    let args = object(arguments, &["path", "query", "mode", "max_matches"], &[])?;
    let query = string(args.get("query").unwrap(), "query")?;
    let mode = string(args.get("mode").unwrap(), "mode")?;
    if mode != "literal" && mode != "regex" {
        return Err(WorkspaceError::new(
            "arguments_invalid",
            "mode must be literal or regex",
        ));
    }
    let max_matches = bounded_u64(
        args.get("max_matches").unwrap(),
        "max_matches",
        MAX_ENTRIES as u64,
    )? as usize;
    let (path, full) = existing(&scope.query_root, args.get("path").unwrap(), "path", false)?;
    let bytes = read_bounded(&full, scope.max_content_bytes)?;
    let text = String::from_utf8(bytes)
        .map_err(|_| WorkspaceError::new("invalid_utf8", "file content is not valid UTF-8"))?;
    let mut matches = Vec::new();
    for (line_index, line) in text.lines().enumerate() {
        let mut start = 0usize;
        while start <= line.len() {
            let found = if mode == "literal" {
                line[start..].find(query)
            } else {
                regex::Regex::new(query)
                    .map_err(|e| WorkspaceError::new("query_invalid", e.to_string()))?
                    .find(&line[start..])
                    .map(|m| m.start())
            };
            let Some(offset) = found else { break };
            let absolute = start + offset;
            let matched = if mode == "literal" {
                query
            } else {
                regex::Regex::new(query)
                    .map_err(|e| WorkspaceError::new("query_invalid", e.to_string()))?
                    .find(&line[absolute..])
                    .map(|m| m.as_str())
                    .unwrap_or("")
            };
            matches.push(json!({"line": line_index + 1, "column": absolute + 1, "text": matched}));
            if matches.len() >= max_matches {
                return Ok(
                    json!({"path": path, "mode": mode, "matches": matches, "truncated": true}),
                );
            }
            let width = matched.len().max(1);
            start = absolute + width;
        }
    }
    Ok(json!({"path": path, "mode": mode, "matches": matches, "truncated": false}))
}

pub fn read_range(scope: &FileScope, arguments: &Value) -> Result<Value, WorkspaceError> {
    let args = object(arguments, &["path", "start_line", "end_line"], &[])?;
    let start = bounded_u64(
        args.get("start_line").unwrap(),
        "start_line",
        MAX_ENTRIES as u64,
    )? as usize;
    let end = bounded_u64(
        args.get("end_line").unwrap(),
        "end_line",
        MAX_ENTRIES as u64,
    )? as usize;
    if end < start {
        return Err(WorkspaceError::new(
            "arguments_invalid",
            "end_line must not precede start_line",
        ));
    }
    let (path, full) = existing(&scope.query_root, args.get("path").unwrap(), "path", false)?;
    let bytes = read_bounded(&full, scope.max_content_bytes)?;
    let text = String::from_utf8(bytes)
        .map_err(|_| WorkspaceError::new("invalid_utf8", "file content is not valid UTF-8"))?;
    let lines: Vec<_> = text.lines().collect();
    let selected = lines
        .get(start - 1..end.min(lines.len()))
        .unwrap_or(&[])
        .join("\n");
    if selected.len() as u64 > scope.max_content_bytes {
        return Err(WorkspaceError::new(
            "resource_limit",
            "range output exceeds the configured bound",
        ));
    }
    Ok(
        json!({"path": path, "start_line": start, "end_line": end, "content": selected, "line_count": lines.len()}),
    )
}

pub fn compare(scope: &FileScope, arguments: &Value) -> Result<Value, WorkspaceError> {
    let args = object(arguments, &["left_path", "right_path"], &[])?;
    let (left, left_full) = existing(
        &scope.query_root,
        args.get("left_path").unwrap(),
        "left_path",
        false,
    )?;
    let (right, right_full) = existing(
        &scope.query_root,
        args.get("right_path").unwrap(),
        "right_path",
        false,
    )?;
    let left_bytes = read_bounded(&left_full, scope.max_content_bytes)?;
    let right_bytes = read_bounded(&right_full, scope.max_content_bytes)?;
    Ok(
        json!({"equal": left_bytes == right_bytes, "left_path": left, "right_path": right, "left_sha256": digest(&left_bytes), "right_sha256": digest(&right_bytes)}),
    )
}

pub fn replace_exact(scope: &FileScope, arguments: &Value) -> Result<Value, WorkspaceError> {
    let args = object(
        arguments,
        &["path", "old_text", "new_text", "expected_matches"],
        &[],
    )?;
    let old_text = string(args.get("old_text").unwrap(), "old_text")?;
    let new_text = text_value(args.get("new_text").unwrap(), "new_text")?;
    let expected = args
        .get("expected_matches")
        .and_then(Value::as_u64)
        .ok_or_else(|| {
            WorkspaceError::new(
                "arguments_invalid",
                "expected_matches must be an unsigned integer",
            )
        })?;
    if expected > MAX_ENTRIES as u64 {
        return Err(WorkspaceError::new(
            "resource_limit",
            "expected_matches exceeds the configured bound",
        ));
    }
    let (path, full) = existing(&scope.source_root, args.get("path").unwrap(), "path", false)?;
    let bytes = read_bounded(&full, scope.max_content_bytes)?;
    let text = String::from_utf8(bytes)
        .map_err(|_| WorkspaceError::new("invalid_utf8", "file content is not valid UTF-8"))?;
    let matches = text.match_indices(old_text).count() as u64;
    if matches != expected {
        return Err(WorkspaceError::new(
            "match_count_mismatch",
            format!("expected {expected} exact matches, found {matches}"),
        ));
    }
    let replaced = text.replace(old_text, new_text);
    if replaced.len() as u64 > scope.max_content_bytes {
        return Err(WorkspaceError::new(
            "resource_limit",
            "replacement output exceeds the configured bound",
        ));
    }
    fs::write(&full, replaced.as_bytes())
        .map_err(|e| WorkspaceError::new("file_write_failed", e.to_string()))?;
    Ok(
        json!({"path": path, "changed": true, "changed_count": matches, "bytes_written": replaced.len()}),
    )
}

fn patch_path(line: &str, prefix: &str) -> Result<String, WorkspaceError> {
    let path = line
        .strip_prefix(prefix)
        .ok_or_else(|| WorkspaceError::new("patch_invalid", "patch file header is malformed"))?;
    let path = path
        .strip_prefix("a/")
        .or_else(|| path.strip_prefix("b/"))
        .unwrap_or(path);
    safe_relative(path, "patch path", false)
}

fn parse_hunk_header(line: &str) -> Result<(usize, usize, usize, usize), WorkspaceError> {
    let body = line
        .strip_prefix("@@ -")
        .and_then(|value| value.strip_suffix(" @@"))
        .ok_or_else(|| {
            WorkspaceError::new(
                "patch_invalid",
                "patch must contain a complete unified hunk header",
            )
        })?;
    let mut sides = body.split(" +");
    let old = sides
        .next()
        .ok_or_else(|| WorkspaceError::new("patch_invalid", "patch old range is missing"))?;
    let new = sides
        .next()
        .ok_or_else(|| WorkspaceError::new("patch_invalid", "patch new range is missing"))?;
    if sides.next().is_some() {
        return Err(WorkspaceError::new(
            "patch_invalid",
            "patch hunk header has extra ranges",
        ));
    }
    let parse_range = |value: &str| -> Result<(usize, usize), WorkspaceError> {
        let mut parts = value.split(',');
        let start = parts
            .next()
            .and_then(|item| item.parse::<usize>().ok())
            .ok_or_else(|| WorkspaceError::new("patch_invalid", "patch range start is invalid"))?;
        let count = match parts.next() {
            Some(item) => item.parse::<usize>().map_err(|_| {
                WorkspaceError::new("patch_invalid", "patch range count is invalid")
            })?,
            None => 1,
        };
        if parts.next().is_some() || start == 0 || count > MAX_ENTRIES {
            return Err(WorkspaceError::new(
                "patch_invalid",
                "patch range is invalid",
            ));
        }
        Ok((start, count))
    };
    let (old_start, old_count) = parse_range(old)?;
    let (new_start, new_count) = parse_range(new)?;
    Ok((old_start, old_count, new_start, new_count))
}

pub fn patch_apply(scope: &FileScope, arguments: &Value) -> Result<Value, WorkspaceError> {
    let args = object(arguments, &["patch"], &["expected_base_sha256"])?;
    let patch = string(args.get("patch").unwrap(), "patch")?;
    if patch.len() as u64 > scope.max_content_bytes * 2 {
        return Err(WorkspaceError::new(
            "resource_limit",
            "patch exceeds the configured bound",
        ));
    }
    let mut lines = patch.lines();
    let old_header = lines
        .next()
        .ok_or_else(|| WorkspaceError::new("patch_invalid", "patch is empty"))?;
    let new_header = lines
        .next()
        .ok_or_else(|| WorkspaceError::new("patch_invalid", "patch has no new-file header"))?;
    let old_path = patch_path(old_header, "--- ")?;
    let new_path = patch_path(new_header, "+++ ")?;
    if old_path != new_path {
        return Err(WorkspaceError::new(
            "patch_invalid",
            "patch must update exactly one unchanged path",
        ));
    }
    let hunk = lines
        .next()
        .ok_or_else(|| WorkspaceError::new("patch_invalid", "patch has no hunk"))?;
    let (old_start, old_count, new_start, new_count) = parse_hunk_header(hunk)?;
    let mut old_lines = Vec::new();
    let mut new_lines = Vec::new();
    for line in lines {
        if line.starts_with(' ') {
            let content = line[1..].to_owned();
            old_lines.push(content.clone());
            new_lines.push(content);
        } else if line.starts_with('-') {
            old_lines.push(line[1..].to_owned());
        } else if line.starts_with('+') {
            new_lines.push(line[1..].to_owned());
        } else if line == "\\ No newline at end of file" {
            continue;
        } else {
            return Err(WorkspaceError::new(
                "patch_invalid",
                "patch contains a malformed hunk line",
            ));
        }
    }
    if old_lines.len() != old_count || new_lines.len() != new_count {
        return Err(WorkspaceError::new(
            "patch_invalid",
            "patch hunk line counts do not match its header",
        ));
    }
    let (path, full) = existing(
        &scope.source_root,
        &Value::String(old_path.clone()),
        "patch path",
        false,
    )?;
    let bytes = read_bounded(&full, scope.max_content_bytes)?;
    if let Some(expected) = args.get("expected_base_sha256").and_then(Value::as_str) {
        if expected != digest(&bytes) {
            return Err(WorkspaceError::new(
                "stale_base",
                "patch base digest does not match the current file",
            ));
        }
    }
    let text = String::from_utf8(bytes)
        .map_err(|_| WorkspaceError::new("invalid_utf8", "patch target is not valid UTF-8"))?;
    let had_trailing_newline = text.ends_with('\n');
    let mut file_lines: Vec<String> = text.lines().map(str::to_owned).collect();
    let offset = old_start - 1;
    if offset > file_lines.len()
        || file_lines.get(offset..offset + old_lines.len()).is_none()
        || file_lines[offset..offset + old_lines.len()] != old_lines
    {
        return Err(WorkspaceError::new(
            "patch_stale_or_mismatch",
            "patch context does not match the current file",
        ));
    }
    file_lines.splice(offset..offset + old_lines.len(), new_lines);
    let mut output = file_lines.join("\n");
    if had_trailing_newline {
        output.push('\n');
    }
    if output.len() as u64 > scope.max_content_bytes {
        return Err(WorkspaceError::new(
            "resource_limit",
            "patched output exceeds the configured bound",
        ));
    }
    if new_start == 0 {
        return Err(WorkspaceError::new(
            "patch_invalid",
            "new range start is invalid",
        ));
    }
    fs::write(&full, output.as_bytes())
        .map_err(|e| WorkspaceError::new("file_write_failed", e.to_string()))?;
    Ok(json!({"changed_files":[path], "changed_hunks":1, "bytes_written":output.len()}))
}

fn digest(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

pub fn sha256(scope: &FileScope, arguments: &Value) -> Result<Value, WorkspaceError> {
    let args = object(arguments, &[], &["path", "text"])?;
    match (args.get("path"), args.get("text")) {
        (Some(path), None) => {
            let (_, full) = existing(&scope.query_root, path, "path", false)?;
            let bytes = read_bounded(&full, scope.max_content_bytes)?;
            Ok(json!({"sha256": digest(&bytes), "bytes": bytes.len()}))
        }
        (None, Some(text)) => {
            let text = string(text, "text")?;
            if text.len() as u64 > scope.max_content_bytes {
                return Err(WorkspaceError::new(
                    "resource_limit",
                    "text exceeds the configured bound",
                ));
            }
            Ok(json!({"sha256": digest(text.as_bytes()), "bytes": text.len()}))
        }
        _ => Err(WorkspaceError::new(
            "arguments_invalid",
            "exactly one of path or text is required",
        )),
    }
}

pub fn verify(scope: &FileScope, arguments: &Value) -> Result<Value, WorkspaceError> {
    let args = object(arguments, &["path", "sha256"], &[])?;
    let expected = string(args.get("sha256").unwrap(), "sha256")?;
    if expected.len() != 71
        || !expected.starts_with("sha256:")
        || !expected[7..]
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(WorkspaceError::new(
            "arguments_invalid",
            "sha256 must be a lowercase sha256 digest",
        ));
    }
    let (path, full) = existing(&scope.query_root, args.get("path").unwrap(), "path", false)?;
    let bytes = read_bounded(&full, scope.max_content_bytes)?;
    let actual = digest(&bytes);
    Ok(
        json!({"path": path, "expected_sha256": expected, "actual_sha256": actual, "verified": expected == actual}),
    )
}

fn manifest_walk(
    root: &Path,
    current: &Path,
    out: &mut Vec<Value>,
    max_file_bytes: u64,
) -> Result<(), WorkspaceError> {
    let mut children = Vec::new();
    for item in fs::read_dir(current)
        .map_err(|e| WorkspaceError::new("directory_read_failed", e.to_string()))?
    {
        children
            .push(item.map_err(|e| WorkspaceError::new("directory_read_failed", e.to_string()))?);
    }
    children.sort_by_key(|item| item.file_name());
    for item in children {
        let path = item.path();
        reject_reparse_chain(&path)?;
        let relative = path
            .strip_prefix(root)
            .map_err(|_| WorkspaceError::new("scope_violation", "manifest path escaped root"))?
            .to_string_lossy()
            .replace('\\', "/");
        let metadata = fs::metadata(&path)
            .map_err(|e| WorkspaceError::new("path_unavailable", e.to_string()))?;
        if metadata.is_dir() {
            out.push(json!({"path": relative, "type": "directory"}));
            manifest_walk(root, &path, out, max_file_bytes)?;
        } else if metadata.is_file() {
            let bytes = read_bounded(&path, max_file_bytes)?;
            out.push(json!({"path": relative, "type": "file", "sha256": digest(&bytes)}));
        } else {
            return Err(WorkspaceError::new(
                "unsupported_type",
                "directory manifest refuses special files",
            ));
        }
        if out.len() > MAX_ENTRIES {
            return Err(WorkspaceError::new(
                "resource_limit",
                "directory manifest entry count exceeds the configured bound",
            ));
        }
    }
    Ok(())
}

pub fn directory_manifest(scope: &FileScope, arguments: &Value) -> Result<Value, WorkspaceError> {
    let args = object(arguments, &[], &["path"])?;
    let (path, full) = match args.get("path") {
        Some(value) => existing(&scope.query_root, value, "path", true)?,
        None => (String::new(), scope.query_root.clone()),
    };
    if !full.is_dir() {
        return Err(WorkspaceError::new("wrong_type", "path is not a directory"));
    }
    let mut entries = Vec::new();
    manifest_walk(&full, &full, &mut entries, scope.max_content_bytes)?;
    Ok(json!({"path": path, "entries": entries, "entry_count": entries.len()}))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> (PathBuf, FileScope) {
        let root =
            std::env::temp_dir().join(format!("tethers-agent-workspace-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        let scope = FileScope::new(&root, &root, &root)
            .unwrap()
            .with_max_content_bytes(4096)
            .unwrap();
        (root, scope)
    }

    #[test]
    fn traversal_is_rejected_before_filesystem_access() {
        let (root, scope) = fixture();
        let result = read(&scope, &json!({"path":"../secret.txt","max_bytes":100}));
        assert_eq!(result.unwrap_err().code, "path_invalid");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn listing_is_sorted_and_stat_is_typed() {
        let (root, scope) = fixture();
        fs::write(root.join("z.txt"), b"z").unwrap();
        fs::write(root.join("a.txt"), b"a").unwrap();
        let listed = list(&scope, &json!({})).unwrap();
        assert_eq!(listed["entries"][0]["path"], "a.txt");
        assert_eq!(listed["entries"][1]["path"], "z.txt");
        let metadata = stat(&scope, &json!({"path":"a.txt"})).unwrap();
        assert_eq!(metadata["kind"], "file");
        assert_eq!(metadata["size_bytes"], 1);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn search_supports_explicit_literal_and_regex_modes() {
        let (root, scope) = fixture();
        fs::write(root.join("notes.txt"), "alpha 12\nbeta 34\n").unwrap();
        let literal = search(
            &scope,
            &json!({"path":"notes.txt","query":"alpha","mode":"literal","max_matches":10}),
        )
        .unwrap();
        assert_eq!(literal["matches"][0]["line"], 1);
        let regex = search(
            &scope,
            &json!({"path":"notes.txt","query":"[0-9]+","mode":"regex","max_matches":10}),
        )
        .unwrap();
        assert_eq!(regex["matches"].as_array().unwrap().len(), 2);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn exact_replace_refuses_unexpected_match_count() {
        let (root, scope) = fixture();
        fs::write(root.join("notes.txt"), "x x\n").unwrap();
        let refused = replace_exact(
            &scope,
            &json!({"path":"notes.txt","old_text":"x","new_text":"y","expected_matches":1}),
        );
        assert_eq!(refused.unwrap_err().code, "match_count_mismatch");
        assert_eq!(fs::read_to_string(root.join("notes.txt")).unwrap(), "x x\n");
        let changed = replace_exact(
            &scope,
            &json!({"path":"notes.txt","old_text":"x","new_text":"y","expected_matches":2}),
        )
        .unwrap();
        assert_eq!(changed["changed_count"], 2);
        assert_eq!(fs::read_to_string(root.join("notes.txt")).unwrap(), "y y\n");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn patch_requires_exact_context_and_base_digest() {
        let (root, scope) = fixture();
        fs::write(root.join("notes.txt"), "one\ntwo\n").unwrap();
        let patch = "--- a/notes.txt\n+++ b/notes.txt\n@@ -1,2 +1,2 @@\n one\n-two\n+three\n";
        let stale = patch_apply(
            &scope,
            &json!({"patch":patch,"expected_base_sha256":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}),
        );
        assert_eq!(stale.unwrap_err().code, "stale_base");
        let changed = patch_apply(&scope, &json!({"patch":patch})).unwrap();
        assert_eq!(changed["changed_hunks"], 1);
        assert_eq!(
            fs::read_to_string(root.join("notes.txt")).unwrap(),
            "one\nthree\n"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn hash_verify_reports_mismatch_without_claiming_success() {
        let (root, scope) = fixture();
        fs::write(root.join("data.txt"), b"data").unwrap();
        let hash = sha256(&scope, &json!({"path":"data.txt"})).unwrap()["sha256"]
            .as_str()
            .unwrap()
            .to_owned();
        assert_eq!(
            verify(&scope, &json!({"path":"data.txt","sha256":hash})).unwrap()["verified"],
            true
        );
        assert_eq!(verify(&scope, &json!({"path":"data.txt","sha256":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"})).unwrap()["verified"], false);
        fs::remove_dir_all(root).unwrap();
    }
}
