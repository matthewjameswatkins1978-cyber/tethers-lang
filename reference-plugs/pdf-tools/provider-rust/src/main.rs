//! Independently owned reference MCP provider for bounded PDF inspection.

use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{self, BufRead, Read, Write};
use std::path::{Path, PathBuf};

const PROVIDER_IDENTITY: &str = "tethers-pdf-provider";
const PROVIDER_VERSION: &str = "1.0.0";
const PROTOCOL_VERSION: &str = "2025-11-25";
const INSPECT_OPERATION: &str = "pdf_inspect";
const MAX_PDF_BYTES: u64 = 64 * 1024 * 1024;
const MAX_PAGE_SCAN: usize = 8 * 1024 * 1024;

#[derive(Debug)]
struct PdfError {
    code: &'static str,
    message: String,
}

impl PdfError {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for PdfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

struct PdfScope {
    query_root: PathBuf,
    max_bytes: u64,
}

impl PdfScope {
    fn new(query_root: &Path, max_bytes: u64) -> Result<Self, PdfError> {
        let query_root = canonical_directory(query_root, "query_root")?;
        if max_bytes == 0 || max_bytes > MAX_PDF_BYTES {
            return Err(PdfError::new(
                "scope_invalid",
                format!("max_bytes {max_bytes} is outside [1, {MAX_PDF_BYTES}]"),
            ));
        }
        Ok(Self {
            query_root,
            max_bytes,
        })
    }
}

fn canonical_directory(path: &Path, label: &'static str) -> Result<PathBuf, PdfError> {
    if !path.is_absolute() {
        return Err(PdfError::new(
            "scope_invalid",
            format!("{label} must be absolute"),
        ));
    }
    let canonical = fs::canonicalize(path)
        .map_err(|e| PdfError::new("scope_invalid", format!("{label}: {e}")))?;
    if !canonical.is_dir() {
        return Err(PdfError::new(
            "scope_invalid",
            format!("{label} is not a directory"),
        ));
    }
    Ok(canonical)
}

fn require_exact_object<'a>(
    value: &'a Value,
    fields: &'a [&'a str],
) -> Result<&'a Map<String, Value>, PdfError> {
    let object = value
        .as_object()
        .ok_or_else(|| PdfError::new("arguments_invalid", "arguments must be an object"))?;
    if object.len() != fields.len() || fields.iter().any(|field| !object.contains_key(*field)) {
        return Err(PdfError::new(
            "arguments_invalid",
            "unknown or missing pdf.inspect argument",
        ));
    }
    Ok(object)
}

fn scoped_path(scope: &PdfScope, raw: &Value) -> Result<(String, PathBuf), PdfError> {
    let relative = raw
        .as_str()
        .ok_or_else(|| PdfError::new("arguments_invalid", "path must be a string"))?;
    if relative.is_empty()
        || relative.len() > 240
        || relative.contains('\\')
        || relative.contains(':')
        || relative.starts_with('/')
        || relative.contains('\0')
    {
        return Err(PdfError::new(
            "path_invalid",
            "path must be a bounded relative slash path",
        ));
    }
    if relative
        .split('/')
        .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err(PdfError::new(
            "path_invalid",
            "path contains an unsafe segment",
        ));
    }
    let canonical = fs::canonicalize(
        scope
            .query_root
            .join(relative.replace('/', std::path::MAIN_SEPARATOR_STR)),
    )
    .map_err(|e| PdfError::new("path_unavailable", format!("path: {e}")))?;
    if !canonical.starts_with(&scope.query_root) || canonical == scope.query_root {
        return Err(PdfError::new(
            "scope_violation",
            "path is outside its approved root",
        ));
    }
    Ok((relative.to_owned(), canonical))
}

fn extract_pdf_version(bytes: &[u8]) -> Option<String> {
    let prefix = b"%PDF-";
    if bytes.len() < prefix.len() + 1 || &bytes[..prefix.len()] != prefix {
        return None;
    }
    let start = prefix.len();
    let end = bytes[start..]
        .iter()
        .position(|&b| matches!(b, b'\n' | b'\r' | b' ' | b'\t'))
        .map(|n| start + n)
        .unwrap_or_else(|| (start + 1).min(bytes.len()));
    let version = std::str::from_utf8(&bytes[start..end]).ok()?;
    (version.len() <= 8
        && !version.is_empty()
        && version.chars().all(|c| c.is_ascii_digit() || c == '.'))
    .then(|| version.to_owned())
}

fn count_pages(bytes: &[u8]) -> Option<u32> {
    if bytes.len() > MAX_PAGE_SCAN {
        return None;
    }
    let mut count = 0u32;
    let mut position = 0usize;
    while let Some(index) = bytes[position..]
        .windows(5)
        .position(|window| window == b"/Type")
    {
        position += index + 5;
        let after = &bytes[position..];
        let whitespace = after
            .iter()
            .take_while(|b| matches!(**b, b' ' | b'\t' | b'\n' | b'\r'))
            .count();
        let value = &after[whitespace..];
        if value.starts_with(b"/Page")
            && matches!(
                value.get(5),
                None | Some(b' ')
                    | Some(b'\n')
                    | Some(b'\r')
                    | Some(b'\t')
                    | Some(b'/')
                    | Some(b'>')
                    | Some(b'<')
            )
        {
            count = count.saturating_add(1);
        }
        position += 1;
    }
    (count != 0).then_some(count)
}

fn inspect(scope: &PdfScope, arguments: &Value) -> Result<Value, PdfError> {
    let object = require_exact_object(arguments, &["path"])?;
    let (relative, path) = scoped_path(scope, &object["path"])?;
    let metadata =
        fs::metadata(&path).map_err(|e| PdfError::new("file_unavailable", e.to_string()))?;
    if !metadata.is_file() {
        return Err(PdfError::new("wrong_type", "path is not a regular file"));
    }
    if metadata.len() > scope.max_bytes {
        return Err(PdfError::new(
            "file_too_large",
            format!(
                "file {} bytes exceeds {} byte host bound",
                metadata.len(),
                scope.max_bytes
            ),
        ));
    }
    let mut bytes = Vec::new();
    fs::File::open(path)
        .map_err(|e| PdfError::new("file_read_failed", e.to_string()))?
        .read_to_end(&mut bytes)
        .map_err(|e| PdfError::new("file_read_failed", e.to_string()))?;
    let pdf_version = extract_pdf_version(&bytes);
    let is_pdf = pdf_version.is_some();
    Ok(
        json!({"path":relative,"size_bytes":metadata.len(),"sha256":format!("sha256:{:x}", Sha256::digest(&bytes)),"is_pdf":is_pdf,"pdf_version":pdf_version,"page_count":if is_pdf { count_pages(&bytes) } else { None }}),
    )
}

fn input_schema() -> Value {
    json!({"type":"object","properties":{"path":{"type":"string"}},"required":["path"],"additionalProperties":false})
}
fn output_schema() -> Value {
    json!({"type":"object","properties":{"path":{"type":"string"},"size_bytes":{"type":"integer","minimum":0},"sha256":{"type":"string","pattern":"^sha256:[a-f0-9]{64}$"},"is_pdf":{"type":"boolean"},"pdf_version":{"type":"string"},"page_count":{"type":"integer","minimum":0}},"required":["path","size_bytes","sha256","is_pdf"],"additionalProperties":false})
}

fn resolve_scope() -> Result<PdfScope, String> {
    let conformance = std::env::var("TETHERS_CONFORMANCE").ok().as_deref() == Some("1");
    let scope_json = std::env::var("TETHERS_OPERATIONAL_SCOPE_JSON").ok();
    let raw = match scope_json.as_deref() {
        Some(raw) if !raw.is_empty() => raw,
        None if conformance => {
            return PdfScope::new(
                &PathBuf::from(
                    std::env::var("TEMP")
                        .map_err(|_| "TEMP is required during host conformance")?,
                ),
                MAX_PDF_BYTES,
            )
            .map_err(|e| e.to_string())
        }
        _ => {
            return Err(
                "TETHERS_OPERATIONAL_SCOPE_JSON is required in installed operational mode".into(),
            )
        }
    };
    let scope: Value = serde_json::from_str(raw).map_err(|_| "malformed operational scope JSON")?;
    let root = scope
        .get("query_root")
        .and_then(Value::as_str)
        .filter(|v| !v.is_empty())
        .ok_or("query_root is required and must be a non-empty string")?;
    let max = if conformance {
        MAX_PDF_BYTES
    } else {
        scope
            .get("max_bytes")
            .and_then(Value::as_u64)
            .ok_or("max_bytes is required and must be an integer")?
    };
    PdfScope::new(&PathBuf::from(root), max).map_err(|e| e.to_string())
}

fn main() {
    let scope = resolve_scope().unwrap_or_else(|failure| {
        eprintln!("pdf provider configuration refused: {failure}");
        std::process::exit(2)
    });
    let stdin = io::stdin();
    let mut initialized = false;
    let mut client_initialized = false;
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        let Ok(request) = serde_json::from_str::<Value>(&line) else {
            eprintln!("malformed JSON request");
            continue;
        };
        let id = request.get("id").cloned().unwrap_or(Value::Null);
        let method = request.get("method").and_then(Value::as_str).unwrap_or("");
        let result = match method {
            "initialize" => {
                initialized = true;
                Ok(
                    json!({"protocolVersion":PROTOCOL_VERSION,"capabilities":{"tools":{}},"serverInfo":{"name":PROVIDER_IDENTITY,"version":PROVIDER_VERSION}}),
                )
            }
            "notifications/initialized" if initialized => {
                client_initialized = true;
                Ok(Value::Null)
            }
            "tools/list" if client_initialized => Ok(
                json!({"tools":[{"name":INSPECT_OPERATION,"inputSchema":input_schema(),"outputSchema":output_schema()}]}),
            ),
            "tools/call" if client_initialized => {
                match request.pointer("/params/name").and_then(Value::as_str) {
                    Some(INSPECT_OPERATION) => inspect(
                        &scope,
                        request.pointer("/params/arguments").unwrap_or(&Value::Null),
                    ),
                    _ => Err(PdfError::new(
                        "unknown_operation",
                        "operation is not part of the reviewed PDF contract",
                    )),
                }
            }
            "tools/list" | "tools/call" => Err(PdfError::new(
                "not_initialized",
                "MCP session is not initialized",
            )),
            _ => Err(PdfError::new("method_not_found", "unsupported MCP method")),
        };
        if method == "notifications/initialized" {
            continue;
        }
        let output = match result {
            Ok(value) => {
                json!({"jsonrpc":"2.0","id":id,"result":if value.is_null() { json!({}) } else { value }})
            }
            Err(error) => {
                json!({"jsonrpc":"2.0","id":id,"error":{"code":if matches!(error.code,"unknown_operation"|"method_not_found") { -32601 } else { -32602 },"message":error.message}})
            }
        };
        println!(
            "{}",
            serde_json::to_string(&output).expect("provider response serializable")
        );
        let _ = io::stdout().flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pdf_header_and_pages_are_detected() {
        let bytes = b"%PDF-1.4\n1 0 obj << /Type /Page >>\n";
        assert_eq!(extract_pdf_version(bytes).as_deref(), Some("1.4"));
        assert_eq!(count_pages(bytes), Some(1));
    }
    #[test]
    fn invalid_pdf_header_is_not_pdf() {
        assert_eq!(extract_pdf_version(b"not pdf"), None);
    }
    #[test]
    fn scope_bound_is_exact() {
        assert!(PdfScope::new(&std::env::temp_dir(), MAX_PDF_BYTES + 1).is_err());
    }
}
