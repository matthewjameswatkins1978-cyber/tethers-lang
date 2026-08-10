//! Public deterministic `.tetherplug` packer.
//!
//! Turns an author source directory into a valid `.tetherplug` without
//! requiring the author to hand-maintain hashes, sizes, or manifest digests.
//! The generated package must pass `package::inspect` unchanged.

use crate::manifest;
use crate::package;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use zip::write::SimpleFileOptions;

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackError {
    pub code: &'static str,
    pub message: String,
}

impl PackError {
    pub(crate) fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for PackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for PackError {}

type Result<T> = std::result::Result<T, PackError>;

// ---------------------------------------------------------------------------
// Digest helper
// ---------------------------------------------------------------------------

fn digest(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

// ---------------------------------------------------------------------------
// Windows reparse safety
// ---------------------------------------------------------------------------

#[cfg(windows)]
fn is_reparse_or_symlink(meta: &std::fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
    meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
fn is_reparse_or_symlink(meta: &std::fs::Metadata) -> bool {
    meta.file_type().is_symlink()
}

// ---------------------------------------------------------------------------
// Author plug.json types (payloads, not payload_index)
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AuthorPlugJson {
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
    capabilities: Vec<AuthorCapability>,
    payloads: Vec<AuthorPayload>,
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
struct AuthorCapability {
    capability_name: String,
    capability_version: u32,
    manifest_path: String,
    #[serde(default)]
    manifest_digest: Option<String>,
    provider_operation_name: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AuthorPayload {
    path: String,
    role: String,
}

// ---------------------------------------------------------------------------
// Public report
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct PackReport {
    pub output_path: String,
    pub package_id: String,
    pub package_version: String,
    pub semantic_package_digest: String,
    pub raw_archive_digest: String,
    pub raw_archive_size: u64,
    pub provider_id: String,
    pub capability_count: usize,
}

// ---------------------------------------------------------------------------
// Source tree top-level validation (roots only, no traversal yet)
// ---------------------------------------------------------------------------

const VALID_ROOTS: &[&str] = &[
    "plug.json",
    "provider",
    "manifests",
    "tests",
    "docs",
    "assets",
    "licenses",
];
const FORBIDDEN_ROOTS: &[&str] = &["signatures"];

fn validate_source_top_level(source: &Path) -> Result<()> {
    if !source.is_dir() {
        return Err(PackError::new(
            "source_unavailable",
            "source is not a directory",
        ));
    }

    let mut seen = Vec::new();
    for entry in fs::read_dir(source)
        .map_err(|e| PackError::new("source_read", format!("cannot read source: {e}")))?
    {
        let entry = entry.map_err(|e| PackError::new("source_read", format!("entry: {e}")))?;
        let meta = entry
            .metadata()
            .map_err(|e| PackError::new("source_read", format!("metadata: {e}")))?;
        if is_reparse_or_symlink(&meta) {
            return Err(PackError::new(
                "unsafe_source",
                "symlinks/junctions/reparse points in source tree are refused",
            ));
        }
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| PackError::new("unsafe_source", "non-UTF-8 source entry name"))?;
        if seen.contains(&name) {
            return Err(PackError::new(
                "invalid_source_layout",
                format!("duplicate top-level entry: {name}"),
            ));
        }
        seen.push(name);
    }

    let has_plug_json = seen.contains(&"plug.json".to_owned());
    let has_provider = seen.iter().any(|r| r == "provider");
    let has_manifests = seen.iter().any(|r| r == "manifests");
    if !has_plug_json || !has_provider || !has_manifests {
        return Err(PackError::new(
            "invalid_source_layout",
            "source must contain plug.json, provider/, and manifests/",
        ));
    }

    for root in &seen {
        if root.ends_with(".tetherplug") {
            return Err(PackError::new(
                "unsafe_source",
                "nested .tetherplug files are refused",
            ));
        }
        if root.starts_with('.') {
            return Err(PackError::new(
                "unsafe_source",
                "hidden entries in source tree are refused",
            ));
        }
        if FORBIDDEN_ROOTS.contains(&root.as_str()) {
            return Err(PackError::new(
                "unsupported_authoring_feature",
                "signatures/ is not supported for public pack",
            ));
        }
        if !VALID_ROOTS.contains(&root.as_str()) {
            return Err(PackError::new(
                "unknown_source_root",
                format!("unknown top-level source entry: {root}"),
            ));
        }
    }

    let plug_path = source.join("plug.json");
    let meta = fs::symlink_metadata(&plug_path)
        .map_err(|_| PackError::new("source_read", "cannot read plug.json"))?;
    if !meta.is_file() || is_reparse_or_symlink(&meta) {
        return Err(PackError::new(
            "invalid_source_layout",
            "plug.json must be an ordinary file",
        ));
    }

    for dir_name in &["provider", "manifests"] {
        let path = source.join(dir_name);
        let meta = fs::symlink_metadata(&path)
            .map_err(|_| PackError::new("source_read", format!("cannot read {dir_name}/")))?;
        if !meta.is_dir() || is_reparse_or_symlink(&meta) {
            return Err(PackError::new(
                "invalid_source_layout",
                format!("{dir_name}/ must be an ordinary directory"),
            ));
        }
    }

    for optional in &["tests", "docs", "assets", "licenses"] {
        if seen.iter().any(|r| r == optional) {
            let path = source.join(optional);
            let meta = fs::symlink_metadata(&path)
                .map_err(|_| PackError::new("source_read", format!("cannot read {optional}/")))?;
            if !meta.is_dir() || is_reparse_or_symlink(&meta) {
                return Err(PackError::new(
                    "invalid_source_layout",
                    format!("{optional}/ must be an ordinary directory"),
                ));
            }
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Source inventory: single bounded traversal producing exact file set
// ---------------------------------------------------------------------------

struct SourceInventory {
    files: BTreeMap<String, (PathBuf, u64)>, // relative package path -> (disk path, size)
}

fn build_source_inventory(source: &Path) -> Result<SourceInventory> {
    let mut inventory = SourceInventory {
        files: BTreeMap::new(),
    };
    for dir_name in &[
        "provider",
        "manifests",
        "tests",
        "docs",
        "assets",
        "licenses",
    ] {
        let dir = source.join(dir_name);
        if !dir.is_dir() {
            continue;
        }
        walk_source_dir(&dir, dir_name, &mut inventory)?;
    }
    if inventory.files.len() > package::MAX_ENTRIES {
        return Err(PackError::new(
            "resource_limit",
            format!("source exceeds {} files", package::MAX_ENTRIES),
        ));
    }
    let total: u64 = inventory.files.values().map(|(_, s)| s).sum();
    if total > package::MAX_TOTAL_UNCOMPRESSED_BYTES {
        return Err(PackError::new(
            "resource_limit",
            "source exceeds total uncompressed bound",
        ));
    }
    Ok(inventory)
}

fn walk_source_dir(dir: &Path, prefix: &str, inventory: &mut SourceInventory) -> Result<()> {
    for entry in fs::read_dir(dir)
        .map_err(|e| PackError::new("source_read", format!("read dir {prefix}: {e}")))?
    {
        let entry = entry.map_err(|e| PackError::new("source_read", format!("entry: {e}")))?;
        let meta = entry
            .metadata()
            .map_err(|e| PackError::new("source_read", format!("metadata: {e}")))?;
        if is_reparse_or_symlink(&meta) {
            return Err(PackError::new(
                "unsafe_source",
                "symlinks/junctions/reparse points in source tree are refused",
            ));
        }
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| PackError::new("unsafe_source", "non-UTF-8 source entry name"))?;
        let relative = format!("{prefix}/{name}");

        if relative.ends_with(".tetherplug") {
            return Err(PackError::new(
                "unsafe_source",
                "nested .tetherplug files are refused",
            ));
        }

        if meta.is_dir() {
            if name.starts_with('.') {
                return Err(PackError::new(
                    "unsafe_source",
                    format!("hidden directory refused: {relative}"),
                ));
            }
            walk_source_dir(&entry.path(), &relative, inventory)?;
        } else if meta.is_file() {
            if meta.len() > package::MAX_ENTRY_BYTES {
                return Err(PackError::new(
                    "resource_limit",
                    format!("file exceeds entry limit: {relative}"),
                ));
            }
            let pkg_path = validate_source_path(&relative)?;
            if inventory.files.contains_key(&pkg_path) {
                return Err(PackError::new(
                    "duplicate_path",
                    format!("case-colliding path: {relative}"),
                ));
            }
            inventory.files.insert(pkg_path, (entry.path(), meta.len()));
        } else {
            return Err(PackError::new(
                "unsafe_source",
                format!("unsupported file type in source: {relative}"),
            ));
        }
    }
    Ok(())
}

fn validate_source_path(raw: &str) -> Result<String> {
    package::validate_path(raw.as_bytes()).map_err(|e| {
        PackError::new(
            match e.code {
                "invalid_path" => "invalid_path",
                _ => "invalid_path",
            },
            e.message,
        )
    })
}

// ---------------------------------------------------------------------------
// Author plug.json parsing & validation
// ---------------------------------------------------------------------------

fn parse_author_plug_json(json_str: &str) -> Result<(AuthorPlugJson, serde_json::Value)> {
    if json_str.len() > package::MAX_JSON_BYTES {
        return Err(PackError::new(
            "invalid_plug_json",
            "plug.json exceeds size limit",
        ));
    }
    if json_str.as_bytes().starts_with(&[0xEF, 0xBB, 0xBF]) {
        return Err(PackError::new(
            "invalid_plug_json",
            "plug.json must be UTF-8 without BOM",
        ));
    }
    let value = manifest::parse_value_no_dupes(json_str)
        .map_err(|e| PackError::new("invalid_plug_json", e.to_string()))?;
    let author: AuthorPlugJson = serde_json::from_value(value.clone())
        .map_err(|e| PackError::new("invalid_plug_json", e.to_string()))?;
    Ok((author, value))
}

fn validate_author_plug_json(author: &AuthorPlugJson) -> Result<()> {
    if author.package_format_version != "1"
        || author.socket_major != 1
        || author.protocol_bindings.len() != 1
        || author.protocol_bindings[0].protocol != "MCP"
        || author.protocol_bindings[0].version != "2025-11-25"
        || author.protocol_bindings[0].transport != "stdio"
        || author.platforms.len() != 1
        || author.platforms[0].os != "windows"
        || author.platforms[0].architecture != "x86_64"
    {
        return Err(PackError::new(
            "incompatible_package",
            "unsupported package compatibility",
        ));
    }

    if let Err(e) = package::validate_dotted(&author.package_id, "package_id") {
        return Err(PackError::new(e.code, e.message));
    }
    if !package::is_version(&author.package_version)
        || author.display_name.is_empty()
        || author.description.is_empty()
        || author.publisher.is_empty()
        || author.licence.is_empty()
    {
        return Err(PackError::new(
            "invalid_metadata",
            "invalid package metadata",
        ));
    }

    if let Err(e) = package::validate_dotted(&author.provider.provider_id, "provider_id") {
        return Err(PackError::new(e.code, e.message));
    }
    if !package::is_version(&author.provider.provider_version)
        || author.provider.capability_operation_namespace.is_empty()
        || author
            .provider
            .launch
            .arguments
            .iter()
            .any(|a| a.contains("cmd /c") || a.contains("-Command") || a.contains('\0'))
    {
        return Err(PackError::new(
            "invalid_launch",
            "unsupported provider launch",
        ));
    }

    let launch_path = validate_source_path(&author.provider.launch.path)?;
    let working = validate_source_path(&author.provider.working_directory)?;
    if !working.starts_with("provider") || !launch_path.starts_with("provider/") {
        return Err(PackError::new(
            "invalid_launch",
            "launch must remain under provider",
        ));
    }

    if author.payloads.is_empty() {
        return Err(PackError::new(
            "invalid_payloads",
            "at least one payload is required",
        ));
    }
    if author.capabilities.is_empty() || author.capabilities.len() > package::MAX_MANIFESTS {
        return Err(PackError::new(
            "invalid_capability",
            "invalid capability count",
        ));
    }

    let mut cap_ids = BTreeSet::new();
    let mut operations = BTreeSet::new();
    for cap in &author.capabilities {
        let key = (cap.capability_name.clone(), cap.capability_version);
        if !cap_ids.insert(key) {
            return Err(PackError::new(
                "invalid_capability",
                "duplicate capability identity",
            ));
        }
        if !operations.insert(cap.provider_operation_name.clone()) {
            return Err(PackError::new(
                "invalid_capability",
                "duplicate provider operation name",
            ));
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Manifest processing
// ---------------------------------------------------------------------------

fn process_manifest(
    json_str: &str,
    author_declared_digest: Option<&str>,
) -> Result<(Vec<u8>, String)> {
    let parsed = manifest::parse_value_no_dupes(json_str)
        .map_err(|e| PackError::new("invalid_manifest", e.to_string()))?;

    let has_top_level_digest = parsed
        .as_object()
        .and_then(|o| o.get("digest"))
        .map(|v| v.is_string())
        .unwrap_or(false);

    let calculated_digest = if has_top_level_digest {
        let verified = manifest::verify_manifest(json_str)
            .map_err(|e| PackError::new("invalid_manifest", e.message))?;
        verified.verified_digest().to_owned()
    } else {
        let (_, digest_val) = manifest::canonicalize_and_digest(json_str)
            .map_err(|e| PackError::new("invalid_manifest", e.message))?;
        digest_val
    };

    if let Some(declared) = author_declared_digest {
        if declared != calculated_digest {
            return Err(PackError::new(
                "manifest_digest_mismatch",
                format!(
                    "declared manifest digest {} does not match calculated {}",
                    declared, calculated_digest
                ),
            ));
        }
    }

    let mut final_value = parsed;
    if let serde_json::Value::Object(ref mut obj) = final_value {
        obj.insert(
            "digest".to_owned(),
            serde_json::Value::String(calculated_digest.clone()),
        );
    }

    let final_bytes = serde_json_canonicalizer::to_vec(&final_value)
        .map_err(|e| PackError::new("invalid_manifest", e.to_string()))?;

    Ok((final_bytes, calculated_digest))
}

// ---------------------------------------------------------------------------
// Payload reconciliation
// ---------------------------------------------------------------------------

struct ProcessedPayload {
    path: String,
    sha256: String,
    size_bytes: u64,
    role: String,
    bytes: Vec<u8>,
}

fn reconcile_payloads(
    _source: &Path,
    author: &AuthorPlugJson,
    inventory: &SourceInventory,
    manifest_outputs: &BTreeMap<String, (Vec<u8>, String)>,
) -> Result<Vec<ProcessedPayload>> {
    let mut declared: BTreeMap<String, String> = BTreeMap::new();
    for p in &author.payloads {
        let path = validate_source_path(&p.path)?;
        if !package::payload_role_ok(&path, &p.role) {
            return Err(PackError::new(
                "invalid_payload_role",
                format!("role {} is not valid for path {}", p.role, path),
            ));
        }
        if declared.contains_key(&path) {
            return Err(PackError::new(
                "duplicate_payload",
                format!("duplicate payload declaration: {path}"),
            ));
        }
        declared.insert(path, p.role.clone());
    }

    let declared_paths: BTreeSet<&String> = declared.keys().collect();
    let inventory_paths: BTreeSet<&String> = inventory.files.keys().collect();

    for path in &declared_paths {
        if !inventory_paths.contains(path) {
            return Err(PackError::new(
                "payload_missing",
                format!("declared payload not found on disk: {path}"),
            ));
        }
    }

    for path in &inventory_paths {
        if !declared_paths.contains(path) {
            return Err(PackError::new(
                "undeclared_payload",
                format!("file on disk not declared in payloads: {path}"),
            ));
        }
    }

    let launch_path = validate_source_path(&author.provider.launch.path)?;
    if !declared.contains_key(&launch_path) {
        return Err(PackError::new(
            "invalid_launch",
            "launch payload is not declared",
        ));
    }

    let mut results = Vec::new();
    for (path, role) in &declared {
        if let Some((manifest_bytes, _)) = manifest_outputs.get(path) {
            results.push(ProcessedPayload {
                path: path.clone(),
                sha256: digest(manifest_bytes),
                size_bytes: manifest_bytes.len() as u64,
                role: role.clone(),
                bytes: manifest_bytes.clone(),
            });
        } else {
            let (disk_path, _expected_size) = &inventory.files[path];
            let file_bytes = fs::read(disk_path)
                .map_err(|_| PackError::new("payload_read", format!("cannot read: {path}")))?;
            results.push(ProcessedPayload {
                path: path.clone(),
                sha256: digest(&file_bytes),
                size_bytes: file_bytes.len() as u64,
                role: role.clone(),
                bytes: file_bytes,
            });
        }
    }

    results.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(results)
}

// ---------------------------------------------------------------------------
// Final plug.json generation (payload_index, sorted capabilities)
// ---------------------------------------------------------------------------

fn generate_final_plug_json(
    author: &AuthorPlugJson,
    capability_digests: &BTreeMap<String, String>,
    payloads: &[ProcessedPayload],
) -> Result<Vec<u8>> {
    let mut capabilities: Vec<(String, u32, String, String, String)> = author
        .capabilities
        .iter()
        .map(|cap| {
            let digest = capability_digests
                .get(&cap.manifest_path)
                .cloned()
                .ok_or_else(|| {
                    PackError::new(
                        "invalid_capability",
                        format!("no digest for manifest: {}", cap.manifest_path),
                    )
                })?;
            Ok((
                cap.capability_name.clone(),
                cap.capability_version,
                cap.manifest_path.clone(),
                digest,
                cap.provider_operation_name.clone(),
            ))
        })
        .collect::<Result<Vec<_>>>()?;

    capabilities.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

    let capabilities_json: Vec<serde_json::Value> = capabilities
        .iter()
        .map(|(name, version, path, digest, op)| {
            serde_json::json!({
                "capability_name": name,
                "capability_version": version,
                "manifest_path": path,
                "manifest_digest": digest,
                "provider_operation_name": op,
            })
        })
        .collect();

    let payload_index: Vec<serde_json::Value> = payloads
        .iter()
        .map(|p| {
            serde_json::json!({
                "path": p.path,
                "sha256": p.sha256,
                "size_bytes": p.size_bytes,
                "role": p.role,
            })
        })
        .collect();

    let mut plug = serde_json::json!({
        "package_format_version": author.package_format_version,
        "package_id": author.package_id,
        "package_version": author.package_version,
        "display_name": author.display_name,
        "description": author.description,
        "publisher": author.publisher,
        "licence": author.licence,
        "socket_major": author.socket_major,
        "protocol_bindings": [{
            "protocol": "MCP",
            "version": "2025-11-25",
            "transport": "stdio",
        }],
        "platforms": [{
            "os": "windows",
            "architecture": "x86_64",
        }],
        "provider": {
            "provider_id": author.provider.provider_id,
            "provider_version": author.provider.provider_version,
            "launch": {
                "path": author.provider.launch.path,
                "arguments": &author.provider.launch.arguments,
            },
            "working_directory": author.provider.working_directory,
            "capability_operation_namespace": author.provider.capability_operation_namespace,
        },
        "capabilities": capabilities_json,
        "payload_index": payload_index,
    });

    if let Some(ref schema) = author.provider.operational_scope_schema {
        if !schema.is_null() {
            plug["provider"]["operational_scope_schema"] = schema.clone();
        }
    }

    serde_json_canonicalizer::to_vec(&plug)
        .map_err(|e| PackError::new("invalid_plug_json", e.to_string()))
}

// ---------------------------------------------------------------------------
// Deterministic ZIP (writes to temp, returns SHA + size)
// ---------------------------------------------------------------------------

fn write_deterministic_zip(
    output_path: &Path,
    plug_json_bytes: &[u8],
    payloads: &[ProcessedPayload],
) -> std::io::Result<(String, u64)> {
    let file = fs::File::create(output_path)?;
    let mut zip = zip::ZipWriter::new(file);

    let dt = zip::DateTime::default();
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .last_modified_time(dt);

    let mut entries: Vec<(&str, &[u8])> = vec![("plug.json", plug_json_bytes)];
    for payload in payloads {
        entries.push((&payload.path, &payload.bytes));
    }
    entries.sort_by_key(|(path, _)| *path);

    for (name, bytes) in &entries {
        zip.start_file(*name, options)?;
        zip.write_all(bytes)?;
    }
    zip.finish()?;

    let written = fs::read(output_path)?;
    Ok((digest(&written), written.len() as u64))
}

// ---------------------------------------------------------------------------
// Main pack entry point
// ---------------------------------------------------------------------------

pub fn pack(source: &Path, output: &Path) -> Result<PackReport> {
    // ---- CLI validation ----
    if !source.is_absolute() {
        return Err(PackError::new(
            "invalid_cli_usage",
            "--source must be absolute",
        ));
    }
    if !output.is_absolute() {
        return Err(PackError::new(
            "invalid_cli_usage",
            "--output must be absolute",
        ));
    }
    if output.extension().and_then(|x| x.to_str()) != Some("tetherplug") {
        return Err(PackError::new(
            "invalid_cli_usage",
            "--output must end in .tetherplug",
        ));
    }
    if output.exists() {
        return Err(PackError::new(
            "output_exists",
            "output file already exists",
        ));
    }
    let parent = output
        .parent()
        .ok_or_else(|| PackError::new("output_unavailable", "output has no parent directory"))?;
    if !parent.is_dir() {
        return Err(PackError::new(
            "output_unavailable",
            "output parent directory does not exist",
        ));
    }

    let source_meta = fs::symlink_metadata(source)
        .map_err(|e| PackError::new("source_unavailable", format!("cannot access source: {e}")))?;
    if is_reparse_or_symlink(&source_meta) {
        return Err(PackError::new(
            "unsafe_source",
            "source root must be an ordinary directory; symlinks/junctions/reparse points are refused",
        ));
    }
    if !source_meta.is_dir() {
        return Err(PackError::new(
            "source_unavailable",
            "source is not a directory",
        ));
    }

    // ---- Source tree top-level checks ----
    validate_source_top_level(source)?;

    // ---- Build source inventory ----
    let inventory = build_source_inventory(source)?;

    // ---- Parse author plug.json ----
    let plug_path = source.join("plug.json");
    let plug_bytes =
        fs::read(&plug_path).map_err(|_| PackError::new("source_read", "cannot read plug.json"))?;
    let plug_str = std::str::from_utf8(&plug_bytes)
        .map_err(|_| PackError::new("invalid_plug_json", "plug.json is not UTF-8"))?;
    let (author, _) = parse_author_plug_json(plug_str)?;
    validate_author_plug_json(&author)?;

    // ---- Process capability manifests ----
    let mut manifest_outputs: BTreeMap<String, (Vec<u8>, String)> = BTreeMap::new();
    let mut capability_digests: BTreeMap<String, String> = BTreeMap::new();
    for cap in &author.capabilities {
        let manifest_path = validate_source_path(&cap.manifest_path)?;
        let manifest_disk = source.join(&manifest_path);
        let manifest_bytes = fs::read(&manifest_disk).map_err(|_| {
            PackError::new(
                "payload_missing",
                format!("manifest missing: {manifest_path}"),
            )
        })?;
        let manifest_str = std::str::from_utf8(&manifest_bytes)
            .map_err(|_| PackError::new("invalid_manifest", "manifest is not UTF-8"))?;

        let (final_manifest_bytes, calculated_digest) =
            process_manifest(manifest_str, cap.manifest_digest.as_deref())?;

        if manifest_outputs.contains_key(&manifest_path) {
            return Err(PackError::new(
                "duplicate_manifest",
                format!("duplicate manifest path: {manifest_path}"),
            ));
        }
        capability_digests.insert(manifest_path.clone(), calculated_digest.clone());
        manifest_outputs.insert(manifest_path, (final_manifest_bytes, calculated_digest));
    }

    // ---- Reconcile payloads ----
    let payloads = reconcile_payloads(source, &author, &inventory, &manifest_outputs)?;

    // ---- Generate final plug.json ----
    let final_plug_bytes = generate_final_plug_json(&author, &capability_digests, &payloads)?;

    // ---- Write to temp sibling file ----
    let temp_path = temp_sibling_path(output)?;
    let (raw_archive_digest, raw_archive_size) =
        write_deterministic_zip(&temp_path, &final_plug_bytes, &payloads).map_err(|e| {
            let _ = fs::remove_file(&temp_path);
            PackError::new("archive_write", format!("cannot write archive: {e}"))
        })?;

    // ---- Validate with package::inspect ----
    let inspection = package::inspect(&temp_path).map_err(|e| {
        let _ = fs::remove_file(&temp_path);
        PackError::new(
            "pack_validation_failed",
            format!("generated package rejected by inspection: {e}"),
        )
    })?;

    let semantic_digest = inspection.package.semantic_digest.clone();

    // ---- Publish with atomic hard-link (no-replace, no partial write) ----
    #[cfg(test)]
    {
        if let Some(ref hook) = *publication_hook::BEFORE_PUBLICATION.lock().unwrap() {
            hook(output);
        }
    }

    match fs::hard_link(&temp_path, output) {
        Ok(()) => {
            let _ = fs::remove_file(&temp_path);
        }
        Err(e) => {
            let _ = fs::remove_file(&temp_path);
            if e.kind() == std::io::ErrorKind::AlreadyExists {
                return Err(PackError::new(
                    "output_collision",
                    format!("output appeared after initial check; refusing to replace: {e}"),
                ));
            }
            return Err(PackError::new(
                "archive_write",
                format!("cannot publish final package: {e}"),
            ));
        }
    }

    Ok(PackReport {
        output_path: output.to_string_lossy().to_string(),
        package_id: inspection.package.package_id,
        package_version: inspection.package.package_version,
        semantic_package_digest: semantic_digest,
        raw_archive_digest,
        raw_archive_size,
        provider_id: inspection.provider_id,
        capability_count: inspection.capabilities.len(),
    })
}

fn temp_sibling_path(output: &Path) -> Result<PathBuf> {
    let parent = output
        .parent()
        .ok_or_else(|| PackError::new("output_unavailable", "output has no parent directory"))?;
    let uuid = uuid::Uuid::new_v4();
    Ok(parent.join(format!(".tmp-pack-{uuid}.tetherplug")))
}

#[cfg(test)]
mod publication_hook {
    use std::path::Path;
    use std::sync::Mutex;
    pub(crate) static BEFORE_PUBLICATION: Mutex<Option<Box<dyn Fn(&Path) + Send + Sync>>> =
        Mutex::new(None);
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn example_plug_json() -> serde_json::Value {
        serde_json::json!({
            "package_format_version": "1",
            "package_id": "example.text-tools",
            "package_version": "0.1.0",
            "display_name": "Text Tools",
            "description": "Example text processing Plug",
            "publisher": "tethers-example",
            "licence": "MIT",
            "socket_major": 1,
            "protocol_bindings": [{
                "protocol": "MCP",
                "version": "2025-11-25",
                "transport": "stdio"
            }],
            "platforms": [{
                "os": "windows",
                "architecture": "x86_64"
            }],
            "provider": {
                "provider_id": "example.text-provider",
                "provider_version": "0.1.0",
                "launch": {
                    "path": "provider/example-text-provider.exe",
                    "arguments": ["--serve"]
                },
                "working_directory": "provider",
                "capability_operation_namespace": "text"
            },
            "capabilities": [{
                "capability_name": "text.inspect",
                "capability_version": 1,
                "manifest_path": "manifests/text-inspect.json",
                "provider_operation_name": "inspect"
            }],
            "payloads": [
                {"path": "manifests/text-inspect.json", "role": "capability_manifest"},
                {"path": "provider/example-text-provider.exe", "role": "provider_executable"}
            ]
        })
    }

    fn example_manifest_json(capability_name: &str) -> serde_json::Value {
        serde_json::json!({
            "manifest_format_version": "1.0",
            "capability_name": capability_name,
            "capability_version": 1,
            "title": "Inspect Text",
            "description": "Inspect text content deterministically",
            "input_schema": {
                "type": "object",
                "properties": {
                    "path": {"type": "string"}
                },
                "required": ["path"],
                "additionalProperties": false
            },
            "output_schema": {
                "type": "object",
                "properties": {
                    "size_bytes": {"type": "integer"},
                    "encoding": {"type": "string"}
                },
                "required": ["size_bytes", "encoding"]
            },
            "effects": ["filesystem.read"],
            "permission_scope": {
                "kind": "path_prefix",
                "allowed_prefixes": ["projects/"]
            },
            "reversibility": "reversible",
            "determinism": "deterministic",
            "idempotency": {
                "mechanism": "none"
            },
            "confirmation_policy": {
                "standing_permitted": true,
                "per_call_required": false
            },
            "timeout_ms": 5000,
            "retry_policy": {
                "max_retries": 0,
                "backoff_ms": 500,
                "allowed_on": ["outcome_unknown"],
                "requires_idempotency_proof": false
            },
            "provider": {
                "identity": "example-text-provider",
                "display_name": "Example Text Provider",
                "identity_source": "host_configuration",
                "description": "Example fixture provider"
            },
            "binding": {
                "kind": "mcp",
                "server_name": "example-text-provider",
                "tool_name": "inspect",
                "adapter": null
            }
        })
    }

    fn example_source_tree(source: &Path, with_manifest_digest: bool) {
        fs::create_dir_all(source.join("provider")).unwrap();
        fs::create_dir_all(source.join("manifests")).unwrap();

        let plug = example_plug_json();
        fs::write(source.join("plug.json"), serde_json::to_vec(&plug).unwrap()).unwrap();

        let mut manifest = example_manifest_json("text.inspect");
        if with_manifest_digest {
            let manifest_str = serde_json::to_string(&manifest).unwrap();
            let (_, digest_val) = manifest::canonicalize_and_digest(&manifest_str).unwrap();
            manifest["digest"] = serde_json::json!(digest_val);
        }
        fs::write(
            source.join("manifests/text-inspect.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();

        fs::write(
            source.join("provider/example-text-provider.exe"),
            b"deterministic fixture provider bytes",
        )
        .unwrap();
    }

    #[test]
    fn p2a_pack_then_inspect_succeeds() {
        let dir = std::env::temp_dir().join(format!("tethers-p2a-{}", uuid::Uuid::new_v4()));
        let source = dir.join("my-plug");
        let output = dir.join("my-plug.tetherplug");
        fs::create_dir_all(&source).unwrap();

        example_source_tree(&source, false);
        let report = pack(&source, &output).unwrap();
        assert!(output.exists());
        assert_eq!(report.package_id, "example.text-tools");
        assert_eq!(report.provider_id, "example.text-provider");
        assert_eq!(report.capability_count, 1);
        assert!(!report.semantic_package_digest.is_empty());
        assert!(!report.raw_archive_digest.is_empty());
        assert!(report.raw_archive_size > 0);

        let inspection = package::inspect(&output).unwrap();
        assert_eq!(inspection.package.package_id, "example.text-tools");
        assert_eq!(inspection.capabilities.len(), 1);

        let source_plug_bytes = fs::read(source.join("plug.json")).unwrap();
        let orig_plug = example_plug_json();
        assert_eq!(source_plug_bytes, serde_json::to_vec(&orig_plug).unwrap());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn p2a_pack_is_deterministic() {
        let dir = std::env::temp_dir().join(format!("tethers-p2a-det-{}", uuid::Uuid::new_v4()));
        let source = dir.join("my-plug");
        let first = dir.join("first.tetherplug");
        let second = dir.join("second.tetherplug");
        fs::create_dir_all(&source).unwrap();

        example_source_tree(&source, false);
        pack(&source, &first).unwrap();
        pack(&source, &second).unwrap();

        let a = fs::read(&first).unwrap();
        let b = fs::read(&second).unwrap();
        assert_eq!(a, b);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn p2a_refuses_relative_source() {
        let err = pack(Path::new("relative"), Path::new("C:\\out.tetherplug")).unwrap_err();
        assert_eq!(err.code, "invalid_cli_usage");
    }

    #[test]
    fn p2a_refuses_relative_output() {
        let err = pack(Path::new("C:\\src"), Path::new("relative.tetherplug")).unwrap_err();
        assert_eq!(err.code, "invalid_cli_usage");
    }

    #[test]
    fn p2a_refuses_wrong_extension() {
        let err = pack(Path::new("C:\\src"), Path::new("C:\\out.zip")).unwrap_err();
        assert_eq!(err.code, "invalid_cli_usage");
        assert!(err.message.contains(".tetherplug"));
    }

    #[test]
    fn p2a_refuses_existing_output() {
        let dir = std::env::temp_dir().join(format!("tethers-p2a-exist-{}", uuid::Uuid::new_v4()));
        let source = dir.join("my-plug");
        let output = dir.join("my-plug.tetherplug");
        fs::create_dir_all(&source).unwrap();
        example_source_tree(&source, false);
        fs::write(&output, b"pre-existing").unwrap();

        let err = pack(&source, &output).unwrap_err();
        assert_eq!(err.code, "output_exists");
        assert_eq!(fs::read(&output).unwrap(), b"pre-existing");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn p2a_refuses_malformed_plug_json() {
        let dir =
            std::env::temp_dir().join(format!("tethers-p2a-malform-{}", uuid::Uuid::new_v4()));
        let source = dir.join("my-plug");
        let output = dir.join("my-plug.tetherplug");
        fs::create_dir_all(&source).unwrap();
        example_source_tree(&source, false);
        fs::write(source.join("plug.json"), b"not json").unwrap();

        let err = pack(&source, &output).unwrap_err();
        assert_eq!(err.code, "invalid_plug_json");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn p2a_refuses_duplicate_json_key() {
        let dir = std::env::temp_dir().join(format!("tethers-p2a-dupkey-{}", uuid::Uuid::new_v4()));
        let source = dir.join("my-plug");
        let output = dir.join("my-plug.tetherplug");
        fs::create_dir_all(&source).unwrap();
        example_source_tree(&source, false);
        fs::write(
            source.join("plug.json"),
            br#"{"package_format_version":"1","package_format_version":"1"}"#,
        )
        .unwrap();

        let err = pack(&source, &output).unwrap_err();
        assert_eq!(err.code, "invalid_plug_json");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn p2a_refuses_missing_payload() {
        let dir = std::env::temp_dir().join(format!("tethers-p2a-miss-{}", uuid::Uuid::new_v4()));
        let source = dir.join("my-plug");
        let output = dir.join("my-plug.tetherplug");
        fs::create_dir_all(&source).unwrap();
        example_source_tree(&source, false);
        let _ = fs::remove_file(source.join("provider/example-text-provider.exe"));

        let err = pack(&source, &output).unwrap_err();
        assert_eq!(err.code, "payload_missing");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn p2a_refuses_undeclared_payload() {
        let dir = std::env::temp_dir().join(format!("tethers-p2a-undec-{}", uuid::Uuid::new_v4()));
        let source = dir.join("my-plug");
        let output = dir.join("my-plug.tetherplug");
        fs::create_dir_all(&source).unwrap();
        example_source_tree(&source, false);
        fs::create_dir_all(source.join("docs")).unwrap();
        fs::write(source.join("docs/readme.md"), b"extra file").unwrap();

        let err = pack(&source, &output).unwrap_err();
        assert_eq!(err.code, "undeclared_payload");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn p2a_refuses_signatures_root() {
        let dir = std::env::temp_dir().join(format!("tethers-p2a-sig-{}", uuid::Uuid::new_v4()));
        let source = dir.join("my-plug");
        let output = dir.join("my-plug.tetherplug");
        fs::create_dir_all(&source).unwrap();
        example_source_tree(&source, false);
        fs::create_dir_all(source.join("signatures")).unwrap();

        let err = pack(&source, &output).unwrap_err();
        assert_eq!(err.code, "unsupported_authoring_feature");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn p2a_refuses_wrong_supplied_manifest_digest() {
        let dir =
            std::env::temp_dir().join(format!("tethers-p2a-wrongdig-{}", uuid::Uuid::new_v4()));
        let source = dir.join("my-plug");
        let output = dir.join("my-plug.tetherplug");
        fs::create_dir_all(&source).unwrap();
        example_source_tree(&source, false);

        let mut plug = example_plug_json();
        plug["capabilities"][0]["manifest_digest"] = serde_json::json!(
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        );
        fs::write(source.join("plug.json"), serde_json::to_vec(&plug).unwrap()).unwrap();

        let err = pack(&source, &output).unwrap_err();
        assert_eq!(err.code, "manifest_digest_mismatch");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn p2a_pack_with_manifest_digest_in_manifest_succeeds() {
        let dir = std::env::temp_dir().join(format!("tethers-p2a-md-{}", uuid::Uuid::new_v4()));
        let source = dir.join("my-plug");
        let output = dir.join("my-plug.tetherplug");
        fs::create_dir_all(&source).unwrap();
        example_source_tree(&source, true);

        let report = pack(&source, &output).unwrap();
        assert!(output.exists());
        assert_eq!(report.capability_count, 1);

        let inspection = package::inspect(&output).unwrap();
        let cap = &inspection.capabilities[0];
        assert_eq!(cap.name, "text.inspect");
        assert!(!cap.manifest_digest.is_empty());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn p2a_wrong_digest_in_manifest_json_is_refused() {
        let dir = std::env::temp_dir().join(format!("tethers-p2a-baddig-{}", uuid::Uuid::new_v4()));
        let source = dir.join("my-plug");
        let output = dir.join("my-plug.tetherplug");
        fs::create_dir_all(&source).unwrap();
        example_source_tree(&source, false);

        let mut manifest = example_manifest_json("text.inspect");
        manifest["digest"] = serde_json::json!(
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        );
        fs::write(
            source.join("manifests/text-inspect.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();

        let err = pack(&source, &output).unwrap_err();
        assert!(err.code == "invalid_manifest" || err.code == "manifest_digest_mismatch");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn p2a_source_plug_json_unchanged() {
        let dir = std::env::temp_dir().join(format!("tethers-p2a-unch-{}", uuid::Uuid::new_v4()));
        let source = dir.join("my-plug");
        let output = dir.join("my-plug.tetherplug");
        fs::create_dir_all(&source).unwrap();
        example_source_tree(&source, false);

        let source_plug_before = fs::read(source.join("plug.json")).unwrap();
        let source_manifest_before = fs::read(source.join("manifests/text-inspect.json")).unwrap();

        pack(&source, &output).unwrap();

        assert_eq!(
            fs::read(source.join("plug.json")).unwrap(),
            source_plug_before
        );
        assert_eq!(
            fs::read(source.join("manifests/text-inspect.json")).unwrap(),
            source_manifest_before
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn p2a_no_output_left_on_failure() {
        let dir = std::env::temp_dir().join(format!("tethers-p2a-nout-{}", uuid::Uuid::new_v4()));
        let source = dir.join("my-plug");
        let output = dir.join("my-plug.tetherplug");
        fs::create_dir_all(&source).unwrap();
        example_source_tree(&source, false);
        let _ = fs::remove_file(source.join("provider/example-text-provider.exe"));

        let _ = pack(&source, &output);
        assert!(!output.exists());

        let _ = fs::remove_dir_all(&dir);
    }

    #[cfg(windows)]
    #[test]
    fn p2a_refuses_source_root_junction() {
        let dir = std::env::temp_dir().join(format!("tethers-p2a-junc-{}", uuid::Uuid::new_v4()));
        let real_source = dir.join("real-plug");
        let junction_source = dir.join("junction-plug");
        let output = dir.join("out.tetherplug");
        fs::create_dir_all(&real_source).unwrap();
        example_source_tree(&real_source, false);

        let junction_created = std::process::Command::new("cmd")
            .args(["/c", "mklink", "/J"])
            .arg(&junction_source)
            .arg(&real_source)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !junction_created {
            let _ = fs::remove_dir_all(&dir);
            return;
        }

        let err = pack(&junction_source, &output).unwrap_err();
        assert_eq!(err.code, "unsafe_source");
        assert!(err.message.contains("source root"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[cfg(not(windows))]
    #[test]
    fn p2a_refuses_source_root_symlink() {
        use std::os::unix::fs as unix_fs;
        let dir = std::env::temp_dir().join(format!("tethers-p2a-sym-{}", uuid::Uuid::new_v4()));
        let real_source = dir.join("real-plug");
        let symlink_source = dir.join("symlink-plug");
        let output = dir.join("out.tetherplug");
        fs::create_dir_all(&real_source).unwrap();
        example_source_tree(&real_source, false);

        unix_fs::symlink(&real_source, &symlink_source).unwrap();
        let err = pack(&symlink_source, &output).unwrap_err();
        assert_eq!(err.code, "unsafe_source");
        assert!(err.message.contains("source root"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn p2a_refuses_publication_collision() {
        let dir = std::env::temp_dir().join(format!("tethers-p2a-col-{}", uuid::Uuid::new_v4()));
        let source = dir.join("my-plug");
        let output = dir.join("my-plug.tetherplug");
        fs::create_dir_all(&source).unwrap();
        example_source_tree(&source, false);

        let output_clone = output.clone();
        *super::publication_hook::BEFORE_PUBLICATION.lock().unwrap() = Some(Box::new(move |p| {
            if p == output_clone {
                fs::write(p, b"collision-bytes").unwrap();
            }
        }));

        let err = pack(&source, &output).unwrap_err();
        assert_eq!(err.code, "output_collision");
        assert_eq!(
            fs::read(&output).unwrap(),
            b"collision-bytes",
            "pre-existing collision bytes must remain untouched"
        );

        let temp_pattern = ".tmp-pack-";
        let parent_dir = output.parent().unwrap();
        let temps: Vec<_> = fs::read_dir(parent_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_str()
                    .map(|n| n.starts_with(temp_pattern) && n.ends_with(".tetherplug"))
                    .unwrap_or(false)
            })
            .map(|e| e.path())
            .collect();
        assert!(
            temps.is_empty(),
            "no temporary .tetherplug must remain after collision"
        );

        *super::publication_hook::BEFORE_PUBLICATION.lock().unwrap() = None;

        let _ = fs::remove_dir_all(&dir);
    }
}
