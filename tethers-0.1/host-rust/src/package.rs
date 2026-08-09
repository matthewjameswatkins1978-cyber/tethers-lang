//! Host-owned, non-executing `.tetherplug` inspection boundary.
//!
//! ZIP convenience extraction is deliberately never used here.  The report is
//! an evidence value; it confers neither installation nor launch authority.

use crate::manifest;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use zip::CompressionMethod;

pub const MAX_ARCHIVE_BYTES: u64 = 64 * 1024 * 1024;
pub const MAX_ENTRIES: usize = 512;
pub const MAX_ENTRY_BYTES: u64 = 16 * 1024 * 1024;
pub const MAX_TOTAL_UNCOMPRESSED_BYTES: u64 = 64 * 1024 * 1024;
pub const MAX_COMPRESSION_RATIO: u64 = 100;
pub const MAX_JSON_BYTES: usize = 256 * 1024;
pub const MAX_MANIFESTS: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageError {
    pub code: &'static str,
    pub message: String,
}
impl PackageError {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}
impl std::fmt::Display for PackageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}
impl std::error::Error for PackageError {}
type Result<T> = std::result::Result<T, PackageError>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageIdentity {
    pub package_id: String,
    pub package_version: String,
    pub semantic_digest: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PayloadEvidence {
    pub path: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub role: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityEvidence {
    pub name: String,
    pub version: u32,
    pub operation: String,
    pub manifest_path: String,
    pub manifest_digest: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformEvidence {
    pub os: String,
    pub architecture: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct InspectionReport {
    pub inspection_format_version: u32,
    pub inspection_evidence_digest: String,
    pub package: PackageIdentity,
    pub raw_archive_digest: String,
    pub raw_archive_size: u64,
    pub provider_id: String,
    pub provider_version: String,
    pub provider_launch_path: String,
    pub provider_launch_arguments: Vec<String>,
    pub provider_working_directory: String,
    pub provider_operation_namespace: String,
    pub selected_platform: PlatformEvidence,
    pub plug_json: PayloadEvidence,
    pub payloads: Vec<PayloadEvidence>,
    pub capabilities: Vec<CapabilityEvidence>,
    pub signature_files: Vec<PayloadEvidence>,
    pub signatures_present: bool,
    #[serde(skip)]
    archive_path: PathBuf,
}
impl InspectionReport {
    pub fn archive_path(&self) -> &Path {
        &self.archive_path
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PlugJson {
    package_format_version: String,
    package_id: String,
    package_version: String,
    display_name: String,
    description: String,
    publisher: String,
    licence: String,
    socket_major: u32,
    protocol_bindings: Vec<ProtocolBinding>,
    platforms: Vec<Platform>,
    provider: Provider,
    capabilities: Vec<Capability>,
    payload_index: Vec<Payload>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProtocolBinding {
    protocol: String,
    version: String,
    transport: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Platform {
    os: String,
    architecture: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Provider {
    provider_id: String,
    provider_version: String,
    launch: Launch,
    working_directory: String,
    capability_operation_namespace: String,
    #[serde(default)]
    #[allow(dead_code)]
    operational_scope_schema: Option<serde_json::Value>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Launch {
    path: String,
    arguments: Vec<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Capability {
    capability_name: String,
    capability_version: u32,
    manifest_path: String,
    manifest_digest: String,
    provider_operation_name: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Payload {
    path: String,
    sha256: String,
    size_bytes: u64,
    role: String,
}

fn digest(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}
fn refusal(code: &'static str, msg: impl Into<String>) -> PackageError {
    PackageError::new(code, msg)
}

fn is_reserved(segment: &str) -> bool {
    let stem = segment
        .split('.')
        .next()
        .unwrap_or(segment)
        .to_ascii_uppercase();
    matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || stem
            .strip_prefix("COM")
            .is_some_and(|n| matches!(n, "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9"))
        || stem
            .strip_prefix("LPT")
            .is_some_and(|n| matches!(n, "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9"))
}
fn validate_path(raw: &[u8]) -> Result<String> {
    if raw.len() > 240 || !raw.is_ascii() {
        return Err(refusal(
            "invalid_path",
            "path must be bounded lowercase ASCII",
        ));
    }
    let path =
        std::str::from_utf8(raw).map_err(|_| refusal("invalid_path", "path is not UTF-8"))?;
    if path.is_empty()
        || path.starts_with('/')
        || path.contains('\\')
        || path.contains(':')
        || path.contains('\0')
    {
        return Err(refusal("invalid_path", "path is not a relative slash path"));
    }
    for segment in path.split('/') {
        let valid = !segment.is_empty()
            && segment.as_bytes()[0]
                .is_ascii_lowercase()
                .then_some(())
                .is_some()
            || !segment.is_empty() && segment.as_bytes()[0].is_ascii_digit();
        if !valid
            || segment.ends_with('.')
            || segment.ends_with(' ')
            || is_reserved(segment)
            || !segment.bytes().all(|b| {
                b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'.' | b'_' | b'-')
            })
        {
            return Err(refusal(
                "invalid_path",
                format!("unsafe package path: {path}"),
            ));
        }
    }
    Ok(path.to_owned())
}
fn validate_dotted(value: &str, field: &'static str) -> Result<()> {
    if value.is_empty()
        || value.split('.').any(|s| {
            s.is_empty()
                || !s
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        })
    {
        Err(refusal("invalid_metadata", format!("invalid {field}")))
    } else {
        Ok(())
    }
}
fn validate_digest(value: &str) -> Result<()> {
    if value.len() == 71
        && value.starts_with("sha256:")
        && value[7..]
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        Ok(())
    } else {
        Err(refusal("invalid_digest", "digest must be lowercase sha256"))
    }
}
fn le_u16(bytes: &[u8]) -> u16 {
    u16::from_le_bytes([bytes[0], bytes[1]])
}
fn le_u32(bytes: &[u8]) -> u32 {
    u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}
/// Validate the complete 22-byte EOCD fixed record. ZIP's central-directory
/// parser remains the archive authority; this independent fixed-profile check
/// rejects features that must never reach it.
fn validate_eocd_profile(bytes: &[u8]) -> Result<()> {
    const EOCD: &[u8; 4] = b"PK\x05\x06";
    if bytes.len() < 22 {
        return Err(refusal("invalid_archive", "archive has no complete EOCD"));
    }
    let start = bytes.len().saturating_sub(22 + u16::MAX as usize);
    let eocd = (start..=bytes.len() - 22)
        .rev()
        .find(|offset| {
            let fixed = &bytes[*offset..*offset + 22];
            fixed[..4] == EOCD[..] && *offset + 22 + le_u16(&fixed[20..22]) as usize == bytes.len()
        })
        .ok_or_else(|| refusal("invalid_archive", "archive has no valid EOCD"))?;
    let fixed = &bytes[eocd..eocd + 22];
    if le_u16(&fixed[20..22]) != 0 {
        return Err(refusal(
            "unsupported_archive_feature",
            "ZIP archive comments are refused",
        ));
    }
    let disk = le_u16(&fixed[4..6]);
    let directory_disk = le_u16(&fixed[6..8]);
    let entries_on_disk = le_u16(&fixed[8..10]);
    let entries_total = le_u16(&fixed[10..12]);
    if disk != 0 || directory_disk != 0 || entries_on_disk != entries_total {
        return Err(refusal(
            "unsupported_archive_feature",
            "multi-disk ZIP is refused",
        ));
    }
    if entries_on_disk == u16::MAX
        || entries_total == u16::MAX
        || le_u32(&fixed[12..16]) == u32::MAX
        || le_u32(&fixed[16..20]) == u32::MAX
    {
        return Err(refusal("unsupported_archive_feature", "Zip64 is refused"));
    }
    Ok(())
}
fn payload_role_ok(path: &str, role: &str) -> bool {
    match role {
        "provider_executable" | "provider_script" => path.starts_with("provider/"),
        "capability_manifest" => path.starts_with("manifests/"),
        "conformance" => path.starts_with("tests/"),
        "documentation" => path.starts_with("docs/"),
        "asset" => path.starts_with("assets/"),
        "licence" | "notice" => path.starts_with("licenses/"),
        _ => false,
    }
}
fn bounded_read<R: Read>(reader: &mut R, limit: u64) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    reader
        .take(limit + 1)
        .read_to_end(&mut out)
        .map_err(|e| refusal("archive_read", e.to_string()))?;
    if out.len() as u64 > limit {
        Err(refusal("resource_limit", "entry exceeds bound"))
    } else {
        Ok(out)
    }
}

pub(crate) fn semantic_digest_for_plug_json(bytes: &[u8]) -> Result<String> {
    if bytes.len() > MAX_JSON_BYTES || bytes.starts_with(&[0xef, 0xbb, 0xbf]) {
        return Err(refusal(
            "invalid_json",
            "plug.json must be bounded UTF-8 without BOM",
        ));
    }
    let text = std::str::from_utf8(bytes)
        .map_err(|_| refusal("invalid_json", "plug.json is not UTF-8"))?;
    let value =
        manifest::parse_value_no_dupes(text).map_err(|e| refusal("invalid_json", e.to_string()))?;
    let _: PlugJson = serde_json::from_value(value.clone())
        .map_err(|e| refusal("invalid_json", e.to_string()))?;
    let canonical = serde_json_canonicalizer::to_vec(&value)
        .map_err(|e| refusal("invalid_json", e.to_string()))?;
    Ok(digest(&canonical))
}

/// Inspect a file by reading it as hostile data. This function does not write,
/// extract, launch, bind, or mutate runtime configuration.
pub fn inspect(path: &Path) -> Result<InspectionReport> {
    if path.extension().and_then(|x| x.to_str()) != Some("tetherplug") {
        return Err(refusal("invalid_archive", "source must use .tetherplug"));
    }
    let bytes = fs::read(path).map_err(|e| refusal("archive_read", e.to_string()))?;
    if bytes.len() as u64 > MAX_ARCHIVE_BYTES || !bytes.starts_with(b"PK") {
        return Err(refusal(
            "invalid_archive",
            "archive profile is unsupported or exceeds bound",
        ));
    }
    validate_eocd_profile(&bytes)?;
    let raw_archive_digest = digest(&bytes);
    let raw_archive_size = bytes.len() as u64;
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))
        .map_err(|e| refusal("invalid_archive", e.to_string()))?;
    if archive.len() == 0 || archive.len() > MAX_ENTRIES {
        return Err(refusal("resource_limit", "invalid entry count"));
    }
    let mut entries = BTreeMap::new();
    let mut folds = HashSet::new();
    let mut total = 0u64;
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|e| refusal("archive_read", e.to_string()))?;
        if entry.is_dir()
            || !entry.is_file()
            || entry.is_symlink()
            || entry.encrypted()
            || entry
                .unix_mode()
                .is_some_and(|mode| mode & 0o170_000 != 0 && mode & 0o170_000 != 0o100_000)
            || !matches!(
                entry.compression(),
                CompressionMethod::Stored | CompressionMethod::Deflated
            )
        {
            return Err(refusal(
                "unsupported_archive_feature",
                "only ordinary stored or deflated files are allowed",
            ));
        }
        let name = validate_path(entry.name_raw())?;
        if name.ends_with(".tetherplug") {
            return Err(refusal(
                "unsupported_archive_feature",
                "nested .tetherplug payloads are refused",
            ));
        }
        if !folds.insert(name.to_ascii_lowercase()) || entries.contains_key(&name) {
            return Err(refusal(
                "duplicate_path",
                "duplicate or case-colliding archive path",
            ));
        }
        if entry.size() > MAX_ENTRY_BYTES
            || entry.compressed_size() == 0 && entry.size() > 0
            || entry.compressed_size() > 0
                && entry.size()
                    > entry
                        .compressed_size()
                        .saturating_mul(MAX_COMPRESSION_RATIO)
        {
            return Err(refusal("resource_limit", "entry resource limit exceeded"));
        }
        total = total
            .checked_add(entry.size())
            .ok_or_else(|| refusal("resource_limit", "size overflow"))?;
        if total > MAX_TOTAL_UNCOMPRESSED_BYTES {
            return Err(refusal(
                "resource_limit",
                "archive exceeds uncompressed bound",
            ));
        }
        let expected_size = entry.size();
        let contents = bounded_read(&mut entry, expected_size)?;
        if contents.len() as u64 != entry.size() {
            return Err(refusal("archive_read", "unexpected decompressed size"));
        }
        entries.insert(name, contents);
    }
    if entries.keys().any(|p| {
        !matches!(
            p.split('/').next(),
            Some(
                "plug.json"
                    | "provider"
                    | "manifests"
                    | "tests"
                    | "docs"
                    | "assets"
                    | "licenses"
                    | "signatures"
            )
        )
    }) || !entries.keys().any(|p| p.starts_with("provider/"))
        || !entries.keys().any(|p| p.starts_with("manifests/"))
    {
        return Err(refusal(
            "invalid_layout",
            "missing required roots or unknown root",
        ));
    }
    for path in entries.keys() {
        if entries
            .keys()
            .any(|other| other != path && other.starts_with(&(path.clone() + "/")))
        {
            return Err(refusal("path_conflict", "file/directory prefix conflict"));
        }
    }
    let plug = entries
        .get("plug.json")
        .ok_or_else(|| refusal("invalid_layout", "exactly one root plug.json required"))?;
    let semantic_digest = semantic_digest_for_plug_json(plug)?;
    let text =
        std::str::from_utf8(plug).map_err(|_| refusal("invalid_json", "plug.json is not UTF-8"))?;
    let value =
        manifest::parse_value_no_dupes(text).map_err(|e| refusal("invalid_json", e.to_string()))?;
    let model: PlugJson = serde_json::from_value(value.clone())
        .map_err(|e| refusal("invalid_json", e.to_string()))?;
    if model.package_format_version != "1"
        || model.socket_major != 1
        || model.protocol_bindings.len() != 1
        || model.protocol_bindings[0].protocol != "MCP"
        || model.protocol_bindings[0].version != "2025-11-25"
        || model.protocol_bindings[0].transport != "stdio"
        || model.platforms.len() != 1
        || model.platforms[0].os != "windows"
        || model.platforms[0].architecture != "x86_64"
    {
        return Err(refusal(
            "incompatible_package",
            "unsupported package compatibility",
        ));
    }
    validate_dotted(&model.package_id, "package_id")?;
    if !is_version(&model.package_version)
        || model.display_name.is_empty()
        || model.description.is_empty()
        || model.publisher.is_empty()
        || model.licence.is_empty()
    {
        return Err(refusal("invalid_metadata", "invalid package metadata"));
    }
    validate_dotted(&model.provider.provider_id, "provider_id")?;
    if !is_version(&model.provider.provider_version)
        || model.provider.capability_operation_namespace.is_empty()
        || model
            .provider
            .launch
            .arguments
            .iter()
            .any(|a| a.contains("cmd /c") || a.contains("-Command") || a.contains('\0'))
    {
        return Err(refusal("invalid_launch", "unsupported provider launch"));
    }
    let launch_path = validate_path(model.provider.launch.path.as_bytes())?;
    let working = validate_path(model.provider.working_directory.as_bytes())?;
    if !working.starts_with("provider") || !launch_path.starts_with("provider/") {
        return Err(refusal(
            "invalid_launch",
            "launch must remain under provider",
        ));
    }
    let mut indexed = BTreeMap::new();
    for p in &model.payload_index {
        let path = validate_path(p.path.as_bytes())?;
        validate_digest(&p.sha256)?;
        if !payload_role_ok(&path, &p.role) || indexed.insert(path.clone(), p).is_some() {
            return Err(refusal(
                "invalid_payload_index",
                "invalid or duplicate payload entry",
            ));
        }
    }
    let archive_payloads = entries
        .keys()
        .filter(|path| path.as_str() != "plug.json" && !path.starts_with("signatures/"))
        .collect::<BTreeSet<_>>();
    let indexed_payloads = indexed.keys().collect::<BTreeSet<_>>();
    if archive_payloads != indexed_payloads {
        return Err(refusal(
            "payload_index_mismatch",
            "archive payloads and payload index must match exactly",
        ));
    }
    for (path, data) in &entries {
        if path == "plug.json" || path.starts_with("signatures/") {
            continue;
        }
        let p = indexed
            .get(path)
            .ok_or_else(|| refusal("unindexed_payload", format!("{path} is not indexed")))?;
        if p.size_bytes != data.len() as u64 || p.sha256 != digest(data) {
            return Err(refusal(
                "payload_mismatch",
                format!("payload evidence mismatch: {path}"),
            ));
        }
    }
    if !indexed.contains_key(&launch_path) {
        return Err(refusal("invalid_launch", "launch payload is not indexed"));
    }
    let mut capabilities = Vec::new();
    let mut cap_ids = BTreeSet::new();
    let mut operations = BTreeSet::new();
    if model.capabilities.is_empty() || model.capabilities.len() > MAX_MANIFESTS {
        return Err(refusal("invalid_capability", "invalid capability count"));
    }
    let mut previous = None;
    for cap in &model.capabilities {
        let key = (cap.capability_name.clone(), cap.capability_version);
        if previous.as_ref().is_some_and(|p: &(String, u32)| p >= &key)
            || !cap_ids.insert(key.clone())
            || !operations.insert(cap.provider_operation_name.clone())
        {
            return Err(refusal(
                "invalid_capability",
                "capabilities must be ordered and unique",
            ));
        }
        previous = Some(key);
        let mp = validate_path(cap.manifest_path.as_bytes())?;
        if !mp.starts_with("manifests/")
            || indexed
                .get(&mp)
                .is_none_or(|p| p.role != "capability_manifest")
        {
            return Err(refusal(
                "invalid_capability",
                "manifest path is not indexed",
            ));
        }
        validate_digest(&cap.manifest_digest)?;
        let manifest_bytes = entries
            .get(&mp)
            .ok_or_else(|| refusal("invalid_capability", "manifest missing"))?;
        let verified = manifest::verify_manifest(
            std::str::from_utf8(manifest_bytes)
                .map_err(|_| refusal("invalid_capability", "manifest UTF-8"))?,
        )
        .map_err(|e| refusal("manifest_refusal", e.message))?;
        if verified.capability_name() != cap.capability_name
            || verified.capability_version() != cap.capability_version
            || verified.verified_digest() != cap.manifest_digest
        {
            return Err(refusal(
                "manifest_mismatch",
                "manifest identity or digest mismatch",
            ));
        }
        capabilities.push(CapabilityEvidence {
            name: cap.capability_name.clone(),
            version: cap.capability_version,
            operation: cap.provider_operation_name.clone(),
            manifest_path: mp,
            manifest_digest: cap.manifest_digest.clone(),
        });
    }
    let mut payloads = Vec::new();
    let mut last = None;
    for p in &model.payload_index {
        if last.as_ref().is_some_and(|x: &String| x >= &p.path) {
            return Err(refusal(
                "invalid_payload_index",
                "payload index must be path sorted",
            ));
        }
        last = Some(p.path.clone());
        payloads.push(PayloadEvidence {
            path: p.path.clone(),
            sha256: p.sha256.clone(),
            size_bytes: p.size_bytes,
            role: p.role.clone(),
        });
    }
    let plug_json = PayloadEvidence {
        path: "plug.json".to_owned(),
        sha256: digest(plug),
        size_bytes: plug.len() as u64,
        role: "package_descriptor".to_owned(),
    };
    let signature_files = entries
        .iter()
        .filter(|(path, _)| path.starts_with("signatures/"))
        .map(|(path, bytes)| PayloadEvidence {
            path: path.clone(),
            sha256: digest(bytes),
            size_bytes: bytes.len() as u64,
            role: "signature_evidence".to_owned(),
        })
        .collect::<Vec<_>>();
    let mut report = InspectionReport {
        inspection_format_version: 1,
        inspection_evidence_digest: String::new(),
        package: PackageIdentity {
            package_id: model.package_id,
            package_version: model.package_version,
            semantic_digest,
        },
        raw_archive_digest,
        raw_archive_size,
        provider_id: model.provider.provider_id,
        provider_version: model.provider.provider_version,
        provider_launch_path: launch_path,
        provider_launch_arguments: model.provider.launch.arguments,
        provider_working_directory: working,
        provider_operation_namespace: model.provider.capability_operation_namespace,
        selected_platform: PlatformEvidence {
            os: model.platforms[0].os.clone(),
            architecture: model.platforms[0].architecture.clone(),
        },
        plug_json,
        payloads,
        capabilities,
        signatures_present: !signature_files.is_empty(),
        signature_files,
        archive_path: path.to_path_buf(),
    };
    let evidence = serde_json::json!({
        "inspection_format_version": report.inspection_format_version,
        "package": &report.package,
        "raw_archive_digest": report.raw_archive_digest,
        "raw_archive_size": report.raw_archive_size,
        "provider_id": report.provider_id,
        "provider_version": report.provider_version,
        "provider_launch_path": report.provider_launch_path,
        "provider_launch_arguments": &report.provider_launch_arguments,
        "provider_working_directory": report.provider_working_directory,
        "provider_operation_namespace": report.provider_operation_namespace,
        "selected_platform": &report.selected_platform,
        "plug_json": &report.plug_json,
        "payloads": &report.payloads,
        "capabilities": &report.capabilities,
        "signature_files": &report.signature_files,
    });
    let evidence_bytes = serde_json_canonicalizer::to_vec(&evidence)
        .map_err(|e| refusal("invalid_json", e.to_string()))?;
    report.inspection_evidence_digest = digest(&evidence_bytes);
    Ok(report)
}
fn is_version(value: &str) -> bool {
    let mut n = 0;
    for part in value.split('.') {
        n += 1;
        if part.is_empty() || !part.bytes().all(|b| b.is_ascii_digit()) {
            return false;
        }
    }
    n == 3
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs::File;
    use std::io::Write;
    use zip::write::SimpleFileOptions;

    fn temporary(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("tethers-m2-{}-{name}", uuid::Uuid::new_v4()))
    }

    fn manifest_bytes() -> Vec<u8> {
        let mut manifest = json!({
            "manifest_format_version":"1.0", "capability_name":"notes.note.read", "capability_version":1,
            "title":"Read", "description":"Read", "input_schema":{"type":"object"},
            "output_schema":{"type":"object"}, "effects":["filesystem.read"],
            "permission_scope":{"kind":"path_prefix","allowed_prefixes":["projects/"]},
            "reversibility":"reversible", "determinism":"deterministic",
            "idempotency":{"mechanism":"none"},
            "confirmation_policy":{"standing_permitted":true,"per_call_required":false},
            "timeout_ms":1000, "retry_policy":{"max_retries":0,"backoff_ms":0,"allowed_on":[],"requires_idempotency_proof":false},
            "provider":{"identity":"local-provider","display_name":"Local","identity_source":"host_configuration","description":null},
            "binding":{"kind":"mcp","server_name":"local","tool_name":"read","adapter":null}
        });
        let (_, digest) = manifest::canonicalize_and_digest(&manifest.to_string()).unwrap();
        manifest["digest"] = json!(digest);
        serde_json::to_vec(&manifest).unwrap()
    }

    fn valid_archive(path: &Path, method: CompressionMethod) {
        let provider = b"harmless provider marker";
        let manifest = manifest_bytes();
        let manifest_digest = manifest::verify_manifest(std::str::from_utf8(&manifest).unwrap())
            .unwrap()
            .verified_digest()
            .to_owned();
        let plug = json!({
            "package_format_version":"1", "package_id":"tethers.file-tools", "package_version":"0.1.0",
            "display_name":"File Tools", "description":"fixture", "publisher":"fixture", "licence":"MIT", "socket_major":1,
            "protocol_bindings":[{"protocol":"MCP","version":"2025-11-25","transport":"stdio"}],
            "platforms":[{"os":"windows","architecture":"x86_64"}],
            "provider":{"provider_id":"tethers.file-provider","provider_version":"0.1.0","launch":{"path":"provider/tool.exe","arguments":["--serve"]},"working_directory":"provider","capability_operation_namespace":"file"},
            "capabilities":[{"capability_name":"notes.note.read","capability_version":1,"manifest_path":"manifests/read.json","manifest_digest":manifest_digest,"provider_operation_name":"read"}],
            "payload_index":[
              {"path":"manifests/read.json","sha256":digest(&manifest),"size_bytes":manifest.len(),"role":"capability_manifest"},
              {"path":"provider/tool.exe","sha256":digest(provider),"size_bytes":provider.len(),"role":"provider_executable"}
            ]
        });
        let file = File::create(path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let options = SimpleFileOptions::default().compression_method(method);
        for (name, bytes) in [
            ("plug.json", serde_json::to_vec(&plug).unwrap()),
            ("manifests/read.json", manifest),
            ("provider/tool.exe", provider.to_vec()),
        ] {
            zip.start_file(name, options).unwrap();
            zip.write_all(&bytes).unwrap();
        }
        zip.finish().unwrap();
    }

    fn hostile_archive(path: &Path, entries: &[(&str, &[u8])]) {
        let file = File::create(path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        for (name, bytes) in entries {
            zip.start_file(*name, SimpleFileOptions::default()).unwrap();
            zip.write_all(bytes).unwrap();
        }
        zip.finish().unwrap();
    }
    fn archive_with_document_mismatch(path: &Path, include_document: bool, index_document: bool) {
        let provider = b"harmless provider marker";
        let manifest = manifest_bytes();
        let document = b"package documentation";
        let manifest_digest = manifest::verify_manifest(std::str::from_utf8(&manifest).unwrap())
            .unwrap()
            .verified_digest()
            .to_owned();
        let mut payload_index = vec![
            json!({"path":"manifests/read.json","sha256":digest(&manifest),"size_bytes":manifest.len(),"role":"capability_manifest"}),
            json!({"path":"provider/tool.exe","sha256":digest(provider),"size_bytes":provider.len(),"role":"provider_executable"}),
        ];
        if index_document {
            payload_index.push(json!({"path":"docs/readme.md","sha256":digest(document),"size_bytes":document.len(),"role":"documentation"}));
            payload_index.sort_by_key(|value| value["path"].as_str().unwrap().to_owned());
        }
        let plug = json!({
            "package_format_version":"1", "package_id":"tethers.file-tools", "package_version":"0.1.0",
            "display_name":"File Tools", "description":"fixture", "publisher":"fixture", "licence":"MIT", "socket_major":1,
            "protocol_bindings":[{"protocol":"MCP","version":"2025-11-25","transport":"stdio"}],
            "platforms":[{"os":"windows","architecture":"x86_64"}],
            "provider":{"provider_id":"tethers.file-provider","provider_version":"0.1.0","launch":{"path":"provider/tool.exe","arguments":["--serve"]},"working_directory":"provider","capability_operation_namespace":"file"},
            "capabilities":[{"capability_name":"notes.note.read","capability_version":1,"manifest_path":"manifests/read.json","manifest_digest":manifest_digest,"provider_operation_name":"read"}],
            "payload_index":payload_index
        });
        let file = File::create(path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        for (name, bytes) in [
            ("plug.json", serde_json::to_vec(&plug).unwrap()),
            ("manifests/read.json", manifest),
            ("provider/tool.exe", provider.to_vec()),
        ] {
            zip.start_file(name, SimpleFileOptions::default()).unwrap();
            zip.write_all(&bytes).unwrap();
        }
        if include_document {
            zip.start_file("docs/readme.md", SimpleFileOptions::default())
                .unwrap();
            zip.write_all(document).unwrap();
        }
        zip.finish().unwrap();
    }
    fn eocd_offset(bytes: &[u8]) -> usize {
        (0..=bytes.len() - 22)
            .rev()
            .find(|offset| bytes[*offset..*offset + 4] == *b"PK\x05\x06")
            .unwrap()
    }
    fn patch_zip(path: &Path, patch: impl FnOnce(&mut [u8])) {
        let mut bytes = fs::read(path).unwrap();
        patch(&mut bytes);
        fs::write(path, bytes).unwrap();
    }
    #[test]
    fn package_paths_reject_windows_escape_forms() {
        for p in [
            b"../x".as_slice(),
            b"C:/x",
            b"x\\y",
            b"con/x",
            b"x /y",
            b"x/../y",
        ] {
            assert!(validate_path(p).is_err(), "{:?}", p);
        }
        assert_eq!(
            validate_path(b"provider/tool.exe").unwrap(),
            "provider/tool.exe"
        );
    }
    #[test]
    fn digest_syntax_is_strict() {
        assert!(validate_digest(&format!("sha256:{}", "a".repeat(64))).is_ok());
        assert!(validate_digest("sha256:ABC").is_err());
    }

    #[test]
    fn stored_and_deflated_archives_have_distinct_raw_but_equal_semantic_identity() {
        let stored = temporary("stored.tetherplug");
        let deflated = temporary("deflated.tetherplug");
        valid_archive(&stored, CompressionMethod::Stored);
        valid_archive(&deflated, CompressionMethod::Deflated);
        let a = inspect(&stored).unwrap();
        let b = inspect(&deflated).unwrap();
        assert_ne!(a.raw_archive_digest, b.raw_archive_digest);
        assert_eq!(a.package, b.package);
        fs::remove_file(stored).unwrap();
        fs::remove_file(deflated).unwrap();
    }

    #[test]
    fn hostile_archive_paths_and_collisions_fail_closed() {
        for (name, entries) in [
            ("traversal", vec![("../provider/tool.exe", b"x" as &[u8])]),
            (
                "case",
                vec![("provider/a", b"x" as &[u8]), ("provider/A", b"x")],
            ),
            (
                "prefix",
                vec![("provider/a", b"x" as &[u8]), ("provider/a/b", b"x")],
            ),
            ("unknown", vec![("evil/x", b"x")]),
        ] {
            let path = temporary(&format!("{name}.tetherplug"));
            hostile_archive(&path, &entries);
            assert!(inspect(&path).is_err());
            fs::remove_file(path).unwrap();
        }
    }

    #[test]
    fn payload_index_is_a_bidirectional_complete_file_set() {
        let unindexed = temporary("unindexed-doc.tetherplug");
        archive_with_document_mismatch(&unindexed, true, false);
        assert_eq!(
            inspect(&unindexed).unwrap_err().code,
            "payload_index_mismatch"
        );
        fs::remove_file(unindexed).unwrap();

        let missing = temporary("missing-doc.tetherplug");
        archive_with_document_mismatch(&missing, false, true);
        assert_eq!(
            inspect(&missing).unwrap_err().code,
            "payload_index_mismatch"
        );
        fs::remove_file(missing).unwrap();
    }

    #[test]
    fn frozen_zip_profile_refuses_multidisk_zip64_encryption_compression_and_directories() {
        for (name, patch) in [
            (
                "multidisk",
                Box::new(|bytes: &mut [u8]| {
                    let eocd = eocd_offset(bytes);
                    bytes[eocd + 4] = 1;
                }) as Box<dyn Fn(&mut [u8])>,
            ),
            (
                "zip64",
                Box::new(|bytes: &mut [u8]| {
                    let eocd = eocd_offset(bytes);
                    bytes[eocd + 10] = 0xff;
                    bytes[eocd + 11] = 0xff;
                }),
            ),
        ] {
            let source = temporary(&format!("{name}.tetherplug"));
            valid_archive(&source, CompressionMethod::Stored);
            patch_zip(&source, |bytes| patch(bytes));
            assert!(inspect(&source).is_err(), "{name} must be refused");
            fs::remove_file(source).unwrap();
        }
        let directory = temporary("directory.tetherplug");
        let file = File::create(&directory).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        zip.add_directory("provider/", SimpleFileOptions::default())
            .unwrap();
        zip.finish().unwrap();
        assert!(inspect(&directory).is_err());
        fs::remove_file(directory).unwrap();

        for (name, encryption, method) in [
            ("encrypted", true, None),
            ("unsupported-compression", false, Some(12u8)),
        ] {
            let source = temporary(&format!("{name}.tetherplug"));
            valid_archive(&source, CompressionMethod::Stored);
            patch_zip(&source, |bytes| {
                for offset in 0..=bytes.len() - 4 {
                    if bytes[offset..offset + 4] == *b"PK\x03\x04" {
                        if encryption {
                            bytes[offset + 6] |= 1;
                        }
                        if let Some(method) = method {
                            bytes[offset + 8] = method;
                        }
                    }
                    if bytes[offset..offset + 4] == *b"PK\x01\x02" {
                        if encryption {
                            bytes[offset + 8] |= 1;
                        }
                        if let Some(method) = method {
                            bytes[offset + 10] = method;
                        }
                    }
                }
            });
            assert!(inspect(&source).is_err(), "{name} must be refused");
            fs::remove_file(source).unwrap();
        }
    }

    #[test]
    fn archive_resource_boundaries_and_nested_packages_refuse_before_use() {
        let count = temporary("count.tetherplug");
        let file = File::create(&count).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        for index in 0..=MAX_ENTRIES {
            zip.start_file(format!("docs/{index}"), SimpleFileOptions::default())
                .unwrap();
            zip.write_all(b"x").unwrap();
        }
        zip.finish().unwrap();
        assert!(inspect(&count).is_err());
        fs::remove_file(count).unwrap();

        let too_large = temporary("archive-limit.tetherplug");
        File::create(&too_large)
            .unwrap()
            .set_len(MAX_ARCHIVE_BYTES + 1)
            .unwrap();
        assert!(inspect(&too_large).is_err());
        fs::remove_file(too_large).unwrap();

        let per_entry = temporary("entry-limit.tetherplug");
        hostile_archive(
            &per_entry,
            &[("docs/large", &vec![0; MAX_ENTRY_BYTES as usize + 1])],
        );
        assert_eq!(inspect(&per_entry).unwrap_err().code, "resource_limit");
        fs::remove_file(per_entry).unwrap();

        let ratio = temporary("ratio-limit.tetherplug");
        let file = File::create(&ratio).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        zip.start_file(
            "docs/repeated",
            SimpleFileOptions::default().compression_method(CompressionMethod::Deflated),
        )
        .unwrap();
        zip.write_all(&vec![0; 1024 * 1024]).unwrap();
        zip.finish().unwrap();
        assert_eq!(inspect(&ratio).unwrap_err().code, "resource_limit");
        fs::remove_file(ratio).unwrap();

        let total = temporary("total-limit.tetherplug");
        let file = File::create(&total).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        for index in 0..5 {
            zip.start_file(
                format!("docs/{index}"),
                SimpleFileOptions::default().compression_method(CompressionMethod::Deflated),
            )
            .unwrap();
            zip.write_all(&vec![0; 14 * 1024 * 1024]).unwrap();
        }
        zip.finish().unwrap();
        assert_eq!(inspect(&total).unwrap_err().code, "resource_limit");
        fs::remove_file(total).unwrap();

        let nested = temporary("nested.tetherplug");
        hostile_archive(&nested, &[("assets/inner.tetherplug", b"PK\x03\x04")]);
        assert_eq!(
            inspect(&nested).unwrap_err().code,
            "unsupported_archive_feature"
        );
        fs::remove_file(nested).unwrap();

        assert!(bounded_read(&mut Cursor::new(vec![0; 2]), 1).is_err());
    }

    #[test]
    fn frozen_zip_profile_refuses_comments_and_nonordinary_unix_metadata() {
        let commented = temporary("commented.tetherplug");
        let file = File::create(&commented).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        zip.start_file("docs/readme", SimpleFileOptions::default())
            .unwrap();
        zip.write_all(b"x").unwrap();
        zip.set_comment("unsupported metadata");
        zip.finish().unwrap();
        assert_eq!(
            inspect(&commented).unwrap_err().code,
            "unsupported_archive_feature"
        );
        fs::remove_file(commented).unwrap();

        let symlink = temporary("unix-symlink.tetherplug");
        let file = File::create(&symlink).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        zip.start_file(
            "provider/link",
            SimpleFileOptions::default().unix_permissions(0o120_777),
        )
        .unwrap();
        zip.write_all(b"target").unwrap();
        zip.finish().unwrap();
        patch_zip(&symlink, |bytes| {
            for offset in 0..=bytes.len() - 42 {
                if bytes[offset..offset + 4] == *b"PK\x01\x02" {
                    bytes[offset + 5] = 3; // Unix "version made by" platform.
                    bytes[offset + 38..offset + 42]
                        .copy_from_slice(&(0o120_777u32 << 16).to_le_bytes());
                }
            }
        });
        assert_eq!(
            inspect(&symlink).unwrap_err().code,
            "unsupported_archive_feature"
        );
        fs::remove_file(symlink).unwrap();
    }

    #[test]
    fn quarantine_and_candidate_stay_host_owned_and_detect_mutation_and_conflict() {
        let source = temporary("candidate.tetherplug");
        let root = temporary("quarantine");
        let registry_root = temporary("registry");
        valid_archive(&source, CompressionMethod::Stored);
        let report = inspect(&source).unwrap();
        let quarantined = crate::candidate::extract_to_quarantine(&report, &root).unwrap();
        assert!(quarantined
            .directory
            .starts_with(fs::canonicalize(&root).unwrap()));
        assert_eq!(
            fs::read(quarantined.directory.join("provider/tool.exe")).unwrap(),
            b"harmless provider marker"
        );
        let registry = crate::candidate::CandidateRegistry::open(&registry_root, &root).unwrap();
        let candidate = registry.create(&quarantined).unwrap();
        assert_eq!(candidate.state, "quarantined_installation_candidate");
        assert_eq!(candidate.launch_arguments, vec!["--serve"]);
        assert_eq!(candidate.provider_working_directory, "provider");
        assert_eq!(candidate.capability_operation_namespace, "file");
        assert_eq!(candidate.selected_platform.architecture, "x86_64");
        assert_eq!(candidate.payloads[0].role, "capability_manifest");
        assert!(!candidate.inspection_evidence_digest.is_empty());
        assert!(
            fs::metadata(quarantined.directory.join("provider/tool.exe"))
                .unwrap()
                .permissions()
                .readonly()
        );
        assert_eq!(registry.load_all().unwrap().len(), 1);
        let unexpected = quarantined.directory.join("provider/unexpected.exe");
        fs::write(&unexpected, b"not in the package evidence").unwrap();
        assert_eq!(registry.load_all().unwrap_err().code, "record_invalid");
        fs::remove_file(unexpected).unwrap();

        let mut conflicting = quarantined.clone();
        conflicting.report.package.semantic_digest = format!("sha256:{}", "b".repeat(64));
        assert_eq!(
            registry.create(&conflicting).unwrap_err().code,
            "semantic_conflict"
        );
        let original_plug = fs::read(quarantined.directory.join("plug.json")).unwrap();
        for relative in ["plug.json", "manifests/read.json", "provider/tool.exe"] {
            let file = quarantined.directory.join(relative);
            let mut permissions = fs::metadata(&file).unwrap().permissions();
            permissions.set_readonly(false);
            fs::set_permissions(file, permissions).unwrap();
        }
        fs::write(quarantined.directory.join("plug.json"), b"{}").unwrap();
        assert_eq!(registry.load_all().unwrap_err().code, "record_invalid");
        fs::write(quarantined.directory.join("plug.json"), original_plug).unwrap();
        fs::write(quarantined.directory.join("provider/tool.exe"), b"mutated").unwrap();
        assert_eq!(registry.load_all().unwrap_err().code, "record_invalid");
        fs::remove_file(source).unwrap();
        fs::remove_dir_all(root).unwrap();
        fs::remove_dir_all(registry_root).unwrap();
    }
}
