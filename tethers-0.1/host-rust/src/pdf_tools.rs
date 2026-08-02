//! J21 M6A PDF inspection contract and host-owned filesystem adapter.
//!
//! Provides read-only structural inspection of PDF files without a PDF library.
//! Byte-level header parsing and page-count scanning are constrained by fixed
//! host bounds; this is a deterministic inspection capability, not a parser.

use serde_json::{json, Map, Value};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub const INSPECT_CAPABILITY: &str = "pdf.inspect";
pub const INSPECT_OPERATION: &str = "pdf_inspect";
pub const MAX_PDF_BYTES: u64 = 64 * 1024 * 1024;
const MAX_PAGE_SCAN: usize = 8 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PdfToolsError {
    pub code: &'static str,
    pub message: String,
}

impl PdfToolsError {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for PdfToolsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}
impl std::error::Error for PdfToolsError {}

#[derive(Debug, Clone)]
pub struct PdfScope {
    pub query_root: PathBuf,
    pub max_bytes: u64,
}

impl PdfScope {
    pub fn new(query_root: &Path, max_bytes: u64) -> Result<Self, PdfToolsError> {
        let root = canonical_directory(query_root, "query_root")?;
        if max_bytes == 0 || max_bytes > MAX_PDF_BYTES {
            return Err(PdfToolsError::new(
                "scope_invalid",
                format!("max_bytes {max_bytes} is outside [1, {MAX_PDF_BYTES}]"),
            ));
        }
        Ok(Self {
            query_root: root,
            max_bytes,
        })
    }
}

fn canonical_directory(path: &Path, label: &'static str) -> Result<PathBuf, PdfToolsError> {
    if !path.is_absolute() {
        return Err(PdfToolsError::new(
            "scope_invalid",
            format!("{label} must be absolute"),
        ));
    }
    let canonical = fs::canonicalize(path)
        .map_err(|e| PdfToolsError::new("scope_invalid", format!("{label}: {e}")))?;
    if !canonical.is_dir() {
        return Err(PdfToolsError::new(
            "scope_invalid",
            format!("{label} is not a directory"),
        ));
    }
    Ok(canonical)
}

fn scoped_path(scope: &PdfScope, raw: &Value) -> Result<(String, PathBuf), PdfToolsError> {
    let relative = raw
        .as_str()
        .ok_or_else(|| PdfToolsError::new("arguments_invalid", "path must be a string"))?;
    if relative.is_empty()
        || relative.len() > 240
        || relative.contains('\\')
        || relative.contains(':')
        || relative.starts_with('/')
        || relative.contains('\0')
    {
        return Err(PdfToolsError::new(
            "path_invalid",
            "path must be a bounded relative slash path",
        ));
    }
    if relative
        .split('/')
        .any(|segment| segment.is_empty() || segment == "." || segment == "..")
    {
        return Err(PdfToolsError::new(
            "path_invalid",
            "path contains an unsafe segment",
        ));
    }
    let full = scope
        .query_root
        .join(relative.replace('/', std::path::MAIN_SEPARATOR_STR));
    let canonical = fs::canonicalize(&full)
        .map_err(|e| PdfToolsError::new("path_unavailable", format!("path: {e}")))?;
    if !canonical.starts_with(&scope.query_root) || canonical == scope.query_root {
        return Err(PdfToolsError::new(
            "scope_violation",
            "path is outside its approved root",
        ));
    }
    Ok((relative.to_owned(), canonical))
}

fn require_exact_object<'a>(
    value: &'a Value,
    fields: &'a [&'a str],
) -> Result<&'a Map<String, Value>, PdfToolsError> {
    let object = value
        .as_object()
        .ok_or_else(|| PdfToolsError::new("arguments_invalid", "arguments must be an object"))?;
    if object.len() != fields.len() || fields.iter().any(|field| !object.contains_key(*field)) {
        return Err(PdfToolsError::new(
            "arguments_invalid",
            "unknown or missing pdf.inspect argument",
        ));
    }
    Ok(object)
}

#[derive(Debug, Clone, serde::Serialize)]
struct InspectResult {
    path: String,
    size_bytes: u64,
    is_pdf: bool,
    pdf_version: Option<String>,
    page_count: Option<u32>,
}

/// Inspect a PDF file and return structural metadata.
///
/// Reads the file header to validate the PDF signature and extract the version,
/// then scans for `/Type /Page` markers to estimate the page count.  Both scans
/// are bounded; a truncated file or a scan that hits the bound is reported
/// honestly rather than silently guessing.
pub fn inspect(scope: &PdfScope, arguments: &Value) -> Result<Value, PdfToolsError> {
    let object = require_exact_object(arguments, &["path"])?;
    let (relative, path) = scoped_path(scope, object.get("path").unwrap())?;

    let metadata =
        fs::metadata(&path).map_err(|e| PdfToolsError::new("file_unavailable", e.to_string()))?;
    if !metadata.is_file() {
        return Err(PdfToolsError::new(
            "wrong_type",
            "path is not a regular file",
        ));
    }
    let size_bytes = metadata.len();
    if size_bytes > scope.max_bytes {
        return Err(PdfToolsError::new(
            "file_too_large",
            format!(
                "file {} bytes exceeds {} byte host bound",
                size_bytes, scope.max_bytes
            ),
        ));
    }

    let mut file =
        fs::File::open(&path).map_err(|e| PdfToolsError::new("file_read_failed", e.to_string()))?;

    let mut buf = Vec::new();
    file.read_to_end(&mut buf)
        .map_err(|e| PdfToolsError::new("file_read_failed", e.to_string()))?;

    let pdf_version = extract_pdf_version(&buf);
    let is_pdf = pdf_version.is_some();

    let page_count = if is_pdf { count_pages(&buf) } else { None };

    let result = InspectResult {
        path: relative,
        size_bytes,
        is_pdf,
        pdf_version,
        page_count,
    };
    serde_json::to_value(result).map_err(|e| PdfToolsError::new("result_serialise", e.to_string()))
}

fn extract_pdf_version(bytes: &[u8]) -> Option<String> {
    let header = std::str::from_utf8(bytes).ok()?;
    let header = header.lines().next()?;
    if !header.starts_with("%PDF-") {
        return None;
    }
    let version = header.strip_prefix("%PDF-")?;
    let version = version.split(|c: char| c.is_whitespace()).next()?;
    if version.len() > 8 || version.is_empty() {
        return None;
    }
    if version.chars().all(|c| c.is_ascii_digit() || c == '.') {
        Some(version.to_owned())
    } else {
        None
    }
}

fn count_pages(bytes: &[u8]) -> Option<u32> {
    if bytes.len() > MAX_PAGE_SCAN {
        return None;
    }
    let text = std::str::from_utf8(bytes).ok()?;
    let mut count: u32 = 0;
    let mut pos = 0usize;
    while let Some(idx) = text[pos..].find("/Type") {
        pos += idx + "/Type".len();
        let after = text[pos..].trim_start();
        if after.starts_with("/Page") {
            let next_char = after.chars().nth("/Page".len());
            match next_char {
                None | Some(' ') | Some('\n') | Some('\r') | Some('\t') | Some('/') | Some('>')
                | Some('<') => {
                    count = count.saturating_add(1);
                }
                _ => {}
            }
        }
    }
    if count == 0 {
        None
    } else {
        Some(count)
    }
}

// -- Manifest builders --

pub fn inspect_input_schema() -> Value {
    json!({"type":"object","properties":{"path":{"type":"string"}},"required":["path"],"additionalProperties":false})
}

pub fn inspect_output_schema() -> Value {
    json!({"type":"object","properties":{"path":{"type":"string"},"size_bytes":{"type":"integer","minimum":0},"is_pdf":{"type":"boolean"},"pdf_version":{"type":"string"},"page_count":{"type":"integer","minimum":0}},"required":["path","size_bytes","is_pdf"],"additionalProperties":false})
}

pub fn inspect_manifest_without_digest() -> Value {
    json!({"manifest_format_version":"1.0","capability_name":INSPECT_CAPABILITY,"capability_version":1,"title":"Bounded PDF Inspection","description":"Read structural PDF metadata (version, page count) from one host-approved root without a PDF library.","input_schema":inspect_input_schema(),"output_schema":inspect_output_schema(),"effects":["data.read","metadata.read"],"permission_scope":{"kind":"path_prefix","allowed_prefixes":["query/"]},"reversibility":"reversible","determinism":"deterministic","idempotency":{"mechanism":"none"},"confirmation_policy":{"standing_permitted":true,"per_call_required":false},"timeout_ms":5000,"retry_policy":{"max_retries":0,"backoff_ms":0,"allowed_on":[],"requires_idempotency_proof":false},"provider":{"identity":"tethers-pdf-provider","display_name":"Tethers PDF Provider","identity_source":"host_configuration","description":"Host-owned direct PDF inspector; no external provider process."},"binding":{"kind":"mcp","server_name":"tethers-pdf-provider","tool_name":INSPECT_OPERATION,"adapter":null}})
}

pub fn manifest_with_digest(mut manifest: Value) -> Result<Value, PdfToolsError> {
    let mut covered = manifest.clone();
    let covered_object = covered
        .as_object_mut()
        .ok_or_else(|| PdfToolsError::new("manifest_invalid", "manifest must be an object"))?;
    covered_object.remove("digest");
    covered_object.remove("title");
    covered_object.remove("description");
    let bytes = serde_json_canonicalizer::to_vec(&covered)
        .map_err(|e| PdfToolsError::new("manifest_invalid", e.to_string()))?;
    use sha2::{Digest, Sha256};
    manifest.as_object_mut().unwrap().insert(
        "digest".into(),
        Value::String(format!("sha256:{:x}", Sha256::digest(bytes))),
    );
    Ok(manifest)
}

/// Host-owned direct PDF inspector that reads files locally.
/// Does not spawn a provider subprocess; implements `CapabilityExecutor`
/// directly so the existing dispatch boundary works without M5/M6 integration.
pub struct PdfToolsExecutor {
    scope: PdfScope,
}

impl PdfToolsExecutor {
    pub fn new(scope: PdfScope) -> Self {
        Self { scope }
    }
}

impl crate::executor::CapabilityExecutor for PdfToolsExecutor {
    fn provider_identity(&self) -> &str {
        "tethers-pdf-provider"
    }

    fn execute(&mut self, ready: &crate::dispatch::DispatchReadyAction) -> Result<Value, String> {
        inspect(&self.scope, ready.arguments()).map_err(|e| e.to_string())
    }

    fn execute_classified(
        &mut self,
        ready: &crate::dispatch::DispatchReadyAction,
        _remaining: Duration,
    ) -> Result<Value, crate::outcome::ProviderDiagnostic> {
        match inspect(&self.scope, ready.arguments()) {
            Ok(value) => Ok(value),
            Err(_) => Err(crate::outcome::ProviderDiagnostic::NoFinalResponse),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scope() -> (std::path::PathBuf, PdfScope) {
        use uuid::Uuid;
        let root = std::env::temp_dir().join(format!("tethers-j21-pdf-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        let scope = PdfScope::new(&root, MAX_PDF_BYTES).unwrap();
        (root, scope)
    }

    fn minimal_pdf_bytes(page_count: u32) -> Vec<u8> {
        let mut pdf = String::from("%PDF-1.4\n%binary comment\n");
        pdf.push_str("1 0 obj\n<< >>\nendobj\n");
        for i in 1..=page_count {
            pdf.push_str(&format!(
                "{o} 0 obj\n<< /Type /Page /Parent 99 0 R >>\nendobj\n",
                o = i + 1
            ));
        }
        pdf.push_str("99 0 obj\n<< /Type /Pages /Kids [");
        for i in 1..=page_count {
            if i > 1 {
                pdf.push(' ');
            }
            pdf.push_str(&format!("{} 0 R", i + 1));
        }
        pdf.push_str(&format!(
            "] /Count {page_count} >>\nendobj\nxref\n0 {total}\ntrailer\n<< /Root 99 0 R >>\nstartxref\n0\n%%EOF\n",
            total = page_count + 2
        ));
        pdf.into_bytes()
    }

    #[test]
    fn header_detection_and_version_extraction() {
        let (root, scope) = scope();
        let pdf = minimal_pdf_bytes(3);
        let path = scope.query_root.join("test.pdf");
        fs::write(&path, &pdf).unwrap();
        let result = inspect(&scope, &json!({"path":"test.pdf"})).unwrap();
        assert_eq!(result["is_pdf"], true);
        assert_eq!(result["pdf_version"], "1.4");
        assert_eq!(result["page_count"], 3);
        assert_eq!(result["size_bytes"], pdf.len() as u64);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn non_pdf_file_reports_false() {
        let (root, scope) = scope();
        fs::write(scope.query_root.join("note.txt"), b"hello world").unwrap();
        let result = inspect(&scope, &json!({"path":"note.txt"})).unwrap();
        assert_eq!(result["is_pdf"], false);
        assert!(result.get("pdf_version").unwrap().is_null());
        assert!(result.get("page_count").unwrap().is_null());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn missing_file_is_unavailable() {
        let (root, scope) = scope();
        let err = inspect(&scope, &json!({"path":"missing.pdf"})).unwrap_err();
        assert_eq!(err.code, "path_unavailable");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn unsafe_segment_is_refused() {
        use uuid::Uuid;
        let root = std::env::temp_dir().join(format!("tethers-j21-scope-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        let scope = PdfScope::new(&root, MAX_PDF_BYTES).unwrap();
        let attempt = inspect(&scope, &json!({"path":"../escape.pdf"}));
        assert_eq!(attempt.unwrap_err().code, "path_invalid");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn oversized_file_refused_by_bound() {
        let (root, _) = scope();
        let tiny = PdfScope::new(&root, 100).unwrap();
        let path = tiny.query_root.join("big.pdf");
        fs::write(&path, vec![0u8; 200]).unwrap();
        let err = inspect(&tiny, &json!({"path":"big.pdf"})).unwrap_err();
        assert_eq!(err.code, "file_too_large");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn manifest_digest_is_stable() {
        let manifest = manifest_with_digest(inspect_manifest_without_digest()).unwrap();
        let text = serde_json::to_string(&manifest).unwrap();
        let verified = crate::manifest::verify_manifest(&text).unwrap();
        assert_eq!(verified.capability_name(), "pdf.inspect");
        assert_eq!(verified.capability_version(), 1);
        assert_eq!(
            verified.manifest().provider.identity,
            "tethers-pdf-provider"
        );
        assert!(!verified.verified_digest().is_empty());
    }

    #[test]
    fn manifest_round_trip_matches_frozen_digest() {
        let first = manifest_with_digest(inspect_manifest_without_digest()).unwrap();
        let second = manifest_with_digest(inspect_manifest_without_digest()).unwrap();
        assert_eq!(first["digest"], second["digest"]);
    }

    #[test]
    fn page_count_zero_returns_none() {
        let (root, scope) = scope();
        let pdf = minimal_pdf_bytes(0);
        let path = scope.query_root.join("empty.pdf");
        fs::write(&path, &pdf).unwrap();
        let result = inspect(&scope, &json!({"path":"empty.pdf"})).unwrap();
        assert_eq!(result["is_pdf"], true);
        assert!(result.get("page_count").unwrap().is_null());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn single_page_pdf() {
        let (root, scope) = scope();
        let pdf = minimal_pdf_bytes(1);
        let path = scope.query_root.join("one.pdf");
        fs::write(&path, &pdf).unwrap();
        let result = inspect(&scope, &json!({"path":"one.pdf"})).unwrap();
        assert_eq!(result["page_count"], 1);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn unknown_argument_is_refused() {
        let (root, scope) = scope();
        let err = inspect(&scope, &json!({"path":"test.pdf","extra":"bad"})).unwrap_err();
        assert_eq!(err.code, "arguments_invalid");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn missing_path_argument_is_refused() {
        let (root, scope) = scope();
        let err = inspect(&scope, &json!({})).unwrap_err();
        assert_eq!(err.code, "arguments_invalid");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn committed_manifest_matches_programmatic_builder() {
        let built = manifest_with_digest(inspect_manifest_without_digest()).unwrap();
        let file = crate::manifest::verify_manifest(include_str!(
            "../../protocol/capability-manifests/pdf-inspect-v1.json"
        ))
        .unwrap();
        assert_eq!(file.verified_digest(), built["digest"].as_str().unwrap());
    }
}
