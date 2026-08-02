//! J21 M6A PDF inspection contract and host-owned filesystem adapter.
//!
//! Provides read-only structural inspection of PDF files without a PDF library.
//! Byte-level header parsing and page-count scanning are constrained by fixed
//! host bounds; this is a deterministic inspection capability, not a parser.

use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{Cursor, Read};
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
    sha256: String,
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

    let sha256_digest = format!("sha256:{:x}", Sha256::digest(&buf));

    let pdf_version = extract_pdf_version(&buf);
    let is_pdf = pdf_version.is_some();

    let page_count = if is_pdf { count_pages(&buf) } else { None };

    let result = InspectResult {
        path: relative,
        size_bytes,
        sha256: sha256_digest,
        is_pdf,
        pdf_version,
        page_count,
    };
    serde_json::to_value(result).map_err(|e| PdfToolsError::new("result_serialise", e.to_string()))
}

fn extract_pdf_version(bytes: &[u8]) -> Option<String> {
    let prefix = b"%PDF-";
    if bytes.len() < prefix.len() + 1 || &bytes[..prefix.len()] != prefix {
        return None;
    }
    let version_start = prefix.len();
    let version_end = bytes[version_start..]
        .iter()
        .position(|&b| b == b'\n' || b == b'\r' || b == b' ' || b == b'\t')
        .map(|p| version_start + p)
        .unwrap_or_else(|| (version_start + 1).min(bytes.len()));
    let version_bytes = &bytes[version_start..version_end];
    let version = std::str::from_utf8(version_bytes).ok()?;
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
    let mut count: u32 = 0;
    let mut pos = 0usize;
    let slash_type = b"/Type";
    let slash_page = b"/Page";
    while let Some(idx) = bytes[pos..]
        .windows(slash_type.len())
        .position(|w| w == slash_type)
    {
        pos += idx + slash_type.len();
        let after = &bytes[pos..];
        let mut ws = 0;
        while ws < after.len()
            && (after[ws] == b' ' || after[ws] == b'\t' || after[ws] == b'\n' || after[ws] == b'\r')
        {
            ws += 1;
        }
        let after_ws = &after[ws..];
        if after_ws.len() >= slash_page.len() && &after_ws[..slash_page.len()] == slash_page {
            let next_byte = after_ws.get(slash_page.len());
            match next_byte {
                None | Some(b' ') | Some(b'\n') | Some(b'\r') | Some(b'\t') | Some(b'/')
                | Some(b'>') | Some(b'<') => {
                    count = count.saturating_add(1);
                }
                _ => {}
            }
        }
        pos += 1;
    }
    if count == 0 {
        None
    } else {
        Some(count)
    }
}

// -- Operational-scope evidence --

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PdfOperationalScopeBinding {
    pub installed_id: String,
    pub capability_name: String,
    pub capability_version: u32,
    pub query_root: PathBuf,
    pub max_bytes: u64,
    pub authority: String,
    pub integrity_digest: String,
}

impl PdfOperationalScopeBinding {
    pub fn create(
        installed_id: &str,
        query_root: &Path,
        max_bytes: u64,
        authority: &str,
    ) -> Result<Self, PdfToolsError> {
        if installed_id.is_empty() {
            return Err(PdfToolsError::new(
                "scope_invalid",
                "installed_id must not be empty",
            ));
        }
        if authority.is_empty() {
            return Err(PdfToolsError::new(
                "scope_invalid",
                "authority must not be empty",
            ));
        }
        if max_bytes == 0 || max_bytes > MAX_PDF_BYTES {
            return Err(PdfToolsError::new(
                "scope_invalid",
                format!("max_bytes {max_bytes} is outside [1, {MAX_PDF_BYTES}]"),
            ));
        }
        let mut binding = Self {
            installed_id: installed_id.into(),
            capability_name: INSPECT_CAPABILITY.into(),
            capability_version: 1,
            query_root: canonical_directory(query_root, "query_root")?,
            max_bytes,
            authority: authority.into(),
            integrity_digest: String::new(),
        };
        let mut covered = binding.clone();
        covered.integrity_digest.clear();
        let bytes = serde_json_canonicalizer::to_vec(&covered)
            .map_err(|e| PdfToolsError::new("scope_invalid", e.to_string()))?;
        use sha2::{Digest, Sha256};
        binding.integrity_digest = format!("sha256:{:x}", Sha256::digest(bytes));
        binding.validate()?;
        Ok(binding)
    }

    pub fn validate(&self) -> Result<(), PdfToolsError> {
        if self.installed_id.is_empty()
            || self.capability_name != INSPECT_CAPABILITY
            || self.capability_version != 1
            || self.authority.is_empty()
        {
            return Err(PdfToolsError::new(
                "scope_invalid",
                "invalid operational scope binding",
            ));
        }
        let canonical = canonical_directory(&self.query_root, "query_root")?;
        if canonical != self.query_root {
            return Err(PdfToolsError::new(
                "scope_invalid",
                "query_root must be canonical",
            ));
        }
        if self.max_bytes == 0 || self.max_bytes > MAX_PDF_BYTES {
            return Err(PdfToolsError::new(
                "scope_invalid",
                format!(
                    "max_bytes {} is outside [1, {MAX_PDF_BYTES}]",
                    self.max_bytes
                ),
            ));
        }
        let mut covered = self.clone();
        let digest = covered.integrity_digest.clone();
        covered.integrity_digest.clear();
        let bytes = serde_json_canonicalizer::to_vec(&covered)
            .map_err(|e| PdfToolsError::new("scope_invalid", e.to_string()))?;
        use sha2::{Digest, Sha256};
        if digest != format!("sha256:{:x}", Sha256::digest(bytes)) {
            return Err(PdfToolsError::new(
                "scope_invalid",
                "scope integrity evidence is invalid",
            ));
        }
        Ok(())
    }

    pub fn scope(&self) -> Result<PdfScope, PdfToolsError> {
        self.validate()?;
        PdfScope::new(&self.query_root, self.max_bytes)
    }
}

// -- Manifest builders --

pub fn inspect_input_schema() -> Value {
    json!({"type":"object","properties":{"path":{"type":"string"}},"required":["path"],"additionalProperties":false})
}

pub fn inspect_output_schema() -> Value {
    json!({"type":"object","properties":{"path":{"type":"string"},"size_bytes":{"type":"integer","minimum":0},"sha256":{"type":"string","pattern":"^sha256:[a-f0-9]{64}$"},"is_pdf":{"type":"boolean"},"pdf_version":{"type":"string"},"page_count":{"type":"integer","minimum":0}},"required":["path","size_bytes","sha256","is_pdf"],"additionalProperties":false})
}

pub fn inspect_manifest_without_digest() -> Value {
    json!({"manifest_format_version":"1.0","capability_name":INSPECT_CAPABILITY,"capability_version":1,"title":"Bounded PDF Inspection","description":"Read structural PDF metadata (version, page count) from one host-approved root without a PDF library.","input_schema":inspect_input_schema(),"output_schema":inspect_output_schema(),"effects":["data.read","metadata.read"],"permission_scope":{"kind":"path_prefix","allowed_prefixes":["query/"]},"reversibility":"reversible","determinism":"deterministic","idempotency":{"mechanism":"none"},"confirmation_policy":{"standing_permitted":true,"per_call_required":false},"timeout_ms":5000,"retry_policy":{"max_retries":0,"backoff_ms":0,"allowed_on":[],"requires_idempotency_proof":false},"provider":{"identity":"tethers-pdf-provider","display_name":"Tethers PDF Provider","identity_source":"host_configuration","description":"Credential-free local PDF inspection provider."},"binding":{"kind":"mcp","server_name":"tethers-pdf-provider","tool_name":INSPECT_OPERATION,"adapter":null}})
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
    manifest.as_object_mut().unwrap().insert(
        "digest".into(),
        Value::String(format!("sha256:{:x}", Sha256::digest(bytes))),
    );
    Ok(manifest)
}

/// Build the deterministic unsigned reference package from the exact provider
/// bytes selected by the host build. ZIP metadata is fixed and never enters
/// semantic identity; the complete payload index does.
///
/// The archived manifest is the J23B-corrected `pdf.inspect@1` manifest; its
/// digest is computed here and embedded in `capabilities[].manifest_digest`,
/// so the package inspector's `verify_manifest` round-trip remains exact.
pub fn build_reference_package(provider_bytes: &[u8]) -> Result<Vec<u8>, PdfToolsError> {
    use sha2::{Digest, Sha256};
    let manifest_value = manifest_with_digest(inspect_manifest_without_digest())
        .map_err(|e| PdfToolsError::new("package_invalid", e.to_string()))?;
    let manifest_bytes = serde_json::to_vec(&manifest_value)
        .map_err(|e| PdfToolsError::new("package_invalid", e.to_string()))?;
    let manifest_digest = manifest_value["digest"]
        .as_str()
        .ok_or_else(|| PdfToolsError::new("package_invalid", "manifest digest missing"))?
        .to_owned();
    let digest = |bytes: &[u8]| format!("sha256:{:x}", Sha256::digest(bytes));
    let plug = json!({
        "package_format_version":"1",
        "package_id":"tethers.pdf-tools",
        "package_version":"1.0.0",
        "display_name":"Tethers PDF Tools",
        "description":"Credential-free bounded local PDF inspection Plug",
        "publisher":"Tethers reference material",
        "licence":"MIT",
        "socket_major":1,
        "protocol_bindings":[{"protocol":"MCP","version":"2025-11-25","transport":"stdio"}],
        "platforms":[{"os":"windows","architecture":"x86_64"}],
        "provider":{"provider_id":"tethers-pdf-provider","provider_version":"1.0.0","launch":{"path":"provider/pdf_tools_provider.exe","arguments":["--query-root","__TETHERS_PDF_QUERY_ROOT__"]},"working_directory":"provider","capability_operation_namespace":"pdf"},
        "capabilities":[{"capability_name":"pdf.inspect","capability_version":1,"manifest_path":"manifests/pdf-inspect-v1.json","manifest_digest":manifest_digest,"provider_operation_name":"pdf_inspect"}],
        "payload_index":[{"path":"manifests/pdf-inspect-v1.json","sha256":digest(&manifest_bytes),"size_bytes":manifest_bytes.len(),"role":"capability_manifest"},{"path":"provider/pdf_tools_provider.exe","sha256":digest(provider_bytes),"size_bytes":provider_bytes.len(),"role":"provider_executable"}]
    });
    let plug_bytes = serde_json_canonicalizer::to_vec(&plug)
        .map_err(|e| PdfToolsError::new("package_invalid", e.to_string()))?;
    let mut archive = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let options = zip::write::FileOptions::<()>::default().last_modified_time(
        zip::DateTime::from_date_and_time(1980, 1, 1, 0, 0, 0)
            .map_err(|e| PdfToolsError::new("package_invalid", e.to_string()))?,
    );
    for (path, bytes) in [
        ("plug.json", plug_bytes.as_slice()),
        ("manifests/pdf-inspect-v1.json", manifest_bytes.as_slice()),
        ("provider/pdf_tools_provider.exe", provider_bytes),
    ] {
        use std::io::Write;
        archive
            .start_file(path, options)
            .map_err(|e| PdfToolsError::new("package_invalid", e.to_string()))?;
        archive
            .write_all(bytes)
            .map_err(|e| PdfToolsError::new("package_invalid", e.to_string()))?;
    }
    archive
        .finish()
        .map(|cursor| cursor.into_inner())
        .map_err(|e| PdfToolsError::new("package_invalid", e.to_string()))
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

    #[test]
    fn binary_pdf_with_non_utf8_bytes_is_recognized() {
        let (root, scope) = scope();
        let mut pdf = b"%PDF-1.7\n".to_vec();
        pdf.extend((0x80u8..0xFFu8).cycle().take(4096));
        pdf.extend(b"\nxref\n0 0\ntrailer\n<< >>\nstartxref\n0\n%%EOF\n");
        let path = scope.query_root.join("binary.pdf");
        fs::write(&path, &pdf).unwrap();
        let result = inspect(&scope, &json!({"path":"binary.pdf"})).unwrap();
        assert_eq!(result["is_pdf"], true);
        assert_eq!(result["pdf_version"], "1.7");
        assert!(result.get("sha256").unwrap().is_string());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn sha256_is_deterministic() {
        let (root, scope) = scope();
        let pdf = minimal_pdf_bytes(2);
        let path = scope.query_root.join("hash.pdf");
        fs::write(&path, &pdf).unwrap();
        let first = inspect(&scope, &json!({"path":"hash.pdf"})).unwrap();
        let second = inspect(&scope, &json!({"path":"hash.pdf"})).unwrap();
        assert_eq!(first["sha256"], second["sha256"]);
        assert_eq!(first["sha256"].as_str().unwrap().len(), 71);
        assert!(first["sha256"].as_str().unwrap().starts_with("sha256:"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn page_marker_scanning_works_with_binary_bytes() {
        let (root, scope) = scope();
        let mut pdf = b"%PDF-1.4\n".to_vec();
        pdf.extend((0x90u8..0xAFu8).cycle().take(1024));
        pdf.extend(b"\n/Type /Page\n");
        pdf.extend((0xB0u8..0xCFu8).cycle().take(2048));
        pdf.extend(b"\n/Type /Page\n");
        pdf.extend(b"%%EOF\n");
        let path = scope.query_root.join("binary_pages.pdf");
        fs::write(&path, &pdf).unwrap();
        let result = inspect(&scope, &json!({"path":"binary_pages.pdf"})).unwrap();
        assert_eq!(result["is_pdf"], true);
        assert_eq!(result["pdf_version"], "1.4");
        assert_eq!(result["page_count"], 2);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn new_pdf_manifest_digest_is_stable() {
        let built = manifest_with_digest(inspect_manifest_without_digest()).unwrap();
        let digest = built["digest"].as_str().unwrap().to_owned();
        let second = manifest_with_digest(inspect_manifest_without_digest()).unwrap();
        assert_eq!(digest, second["digest"].as_str().unwrap());
        let text = serde_json::to_string(&built).unwrap();
        let verified = crate::manifest::verify_manifest(&text).unwrap();
        assert_eq!(verified.capability_name(), "pdf.inspect");
        assert_eq!(verified.capability_version(), 1);
        assert_eq!(
            verified.manifest().provider.identity,
            "tethers-pdf-provider"
        );
        assert_eq!(verified.manifest().binding.tool_name, "pdf_inspect");
    }

    #[test]
    fn old_pdf_manifest_digest_is_not_returned() {
        let built = manifest_with_digest(inspect_manifest_without_digest()).unwrap();
        assert_ne!(
            built["digest"].as_str().unwrap(),
            "sha256:fe8d4eb7a36f8961baea94175f0eff979364322534ca27a305486688e3b268b3"
        );
    }

    #[test]
    fn committed_pdf_manifest_matches_programmatic_builder() {
        let built = manifest_with_digest(inspect_manifest_without_digest()).unwrap();
        let file = crate::manifest::verify_manifest(include_str!(
            "../../protocol/capability-manifests/pdf-inspect-v1.json"
        ))
        .unwrap();
        assert_eq!(file.verified_digest(), built["digest"].as_str().unwrap());
        assert_eq!(
            file.manifest().provider.description.as_deref(),
            Some("Credential-free local PDF inspection provider.")
        );
    }

    #[test]
    fn reference_package_bytes_are_deterministic_and_inspectable() {
        let bytes = b"compiled-pdf-provider-placeholder";
        let first = build_reference_package(bytes).unwrap();
        let second = build_reference_package(bytes).unwrap();
        assert_eq!(first, second);
        let root =
            std::env::temp_dir().join(format!("tethers-j23b-package-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        let archive = root.join("pdf-tools.tetherplug");
        fs::write(&archive, first).unwrap();
        let report = crate::package::inspect(&archive).unwrap();
        assert_eq!(report.package.package_id, "tethers.pdf-tools");
        assert_eq!(report.package.package_version, "1.0.0");
        assert_eq!(report.provider_id, "tethers-pdf-provider");
        assert_eq!(report.provider_version, "1.0.0");
        assert_eq!(
            report.provider_launch_path,
            "provider/pdf_tools_provider.exe"
        );
        assert_eq!(
            report.provider_launch_arguments,
            vec![
                "--query-root".to_string(),
                "__TETHERS_PDF_QUERY_ROOT__".to_string()
            ]
        );
        assert_eq!(report.provider_working_directory, "provider");
        assert_eq!(report.provider_operation_namespace, "pdf");
        assert_eq!(report.capabilities.len(), 1);
        assert_eq!(report.capabilities[0].name, "pdf.inspect");
        assert_eq!(report.capabilities[0].version, 1);
        assert_eq!(report.capabilities[0].operation, "pdf_inspect");
        assert_eq!(
            report.capabilities[0].manifest_path,
            "manifests/pdf-inspect-v1.json"
        );
        assert_eq!(report.payloads.len(), 2);
        assert_eq!(report.payloads[0].path, "manifests/pdf-inspect-v1.json");
        assert_eq!(report.payloads[1].path, "provider/pdf_tools_provider.exe");
        assert!(!report.signatures_present);
        fs::remove_dir_all(root).unwrap();
    }
}
