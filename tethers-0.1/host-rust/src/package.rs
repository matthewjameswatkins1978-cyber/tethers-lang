//! Host-owned, non-executing `.tetherplug` inspection boundary.
//!
//! ZIP convenience extraction is deliberately never used here.  The report is
//! an evidence value; it confers neither installation nor launch authority.

use crate::manifest;
use serde::Deserialize;
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageIdentity {
    pub package_id: String,
    pub package_version: String,
    pub semantic_digest: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PayloadEvidence {
    pub path: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub role: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityEvidence {
    pub name: String,
    pub version: u32,
    pub operation: String,
    pub manifest_path: String,
    pub manifest_digest: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InspectionReport {
    pub package: PackageIdentity,
    pub raw_archive_digest: String,
    pub raw_archive_size: u64,
    pub provider_id: String,
    pub provider_version: String,
    pub provider_launch_path: String,
    pub provider_working_directory: String,
    pub payloads: Vec<PayloadEvidence>,
    pub capabilities: Vec<CapabilityEvidence>,
    pub signatures_present: bool,
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
fn zip64_or_multidisk(bytes: &[u8]) -> bool {
    bytes
        .windows(4)
        .any(|x| x == b"PK\x06\x06" || x == b"PK\x06\x07")
        || bytes
            .windows(4)
            .rev()
            .find(|x| *x == b"PK\x05\x06")
            .is_some_and(|eocd| {
                eocd.len() >= 22
                    && (eocd[4] != 0 || eocd[5] != 0 || eocd[6] != eocd[8] || eocd[7] != eocd[9])
            })
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

/// Inspect a file by reading it as hostile data. This function does not write,
/// extract, launch, bind, or mutate runtime configuration.
pub fn inspect(path: &Path) -> Result<InspectionReport> {
    if path.extension().and_then(|x| x.to_str()) != Some("tetherplug") {
        return Err(refusal("invalid_archive", "source must use .tetherplug"));
    }
    let bytes = fs::read(path).map_err(|e| refusal("archive_read", e.to_string()))?;
    if bytes.len() as u64 > MAX_ARCHIVE_BYTES
        || !bytes.starts_with(b"PK")
        || zip64_or_multidisk(&bytes)
    {
        return Err(refusal(
            "invalid_archive",
            "archive profile is unsupported or exceeds bound",
        ));
    }
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
        if !folds.insert(name.to_ascii_lowercase()) || entries.contains_key(&name) {
            return Err(refusal(
                "duplicate_path",
                "duplicate or case-colliding archive path",
            ));
        }
        if entry.size() > MAX_ENTRY_BYTES
            || entry.compressed_size() == 0 && entry.size() > 0
            || entry.compressed_size() > 0
                && entry.size() / entry.compressed_size() > MAX_COMPRESSION_RATIO
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
    if plug.len() > MAX_JSON_BYTES || plug.starts_with(&[0xef, 0xbb, 0xbf]) {
        return Err(refusal(
            "invalid_json",
            "plug.json must be bounded UTF-8 without BOM",
        ));
    }
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
    let canonical = serde_json_canonicalizer::to_vec(&value)
        .map_err(|e| refusal("invalid_json", e.to_string()))?;
    Ok(InspectionReport {
        package: PackageIdentity {
            package_id: model.package_id,
            package_version: model.package_version,
            semantic_digest: digest(&canonical),
        },
        raw_archive_digest,
        raw_archive_size,
        provider_id: model.provider.provider_id,
        provider_version: model.provider.provider_version,
        provider_launch_path: launch_path,
        provider_working_directory: working,
        payloads,
        capabilities,
        signatures_present: entries.keys().any(|p| p.starts_with("signatures/")),
        archive_path: path.to_path_buf(),
    })
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
}
