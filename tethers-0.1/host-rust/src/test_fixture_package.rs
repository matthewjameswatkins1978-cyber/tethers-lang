//! Neutral synthetic Plug package helper for host lifecycle tests.
//!
//! This deliberately carries no real capability-family implementation. Tests
//! supply provider bytes; package mechanics stay exercised through the same
//! public package parser used by installed Plug lifecycle coverage.

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::io::{Cursor, Write};

pub fn build_fixture_package(provider_bytes: &[u8]) -> Result<Vec<u8>, String> {
    let manifest: Value = serde_json::from_str(include_str!(
        "../../protocol/capability-manifests/fixture-ping.json"
    ))
    .map_err(|error| error.to_string())?;
    let manifest_bytes = serde_json::to_vec(&manifest).map_err(|error| error.to_string())?;
    let digest = |bytes: &[u8]| format!("sha256:{:x}", Sha256::digest(bytes));
    let plug = json!({
        "package_format_version":"1", "package_id":"tethers.fixture", "package_version":"1.0.0",
        "display_name":"Tethers stdio fixture", "description":"Neutral test fixture Plug", "publisher":"Tethers test material", "licence":"MIT", "socket_major":1,
        "protocol_bindings":[{"protocol":"MCP","version":"2025-11-25","transport":"stdio"}], "platforms":[{"os":"windows","architecture":"x86_64"}],
        "provider":{"provider_id":"tethers-stdio-fixture","provider_version":"0.1.0","launch":{"path":"provider/tethers-stdio-fixture.exe","arguments":[]},"working_directory":"provider","capability_operation_namespace":"fixture","operational_scope_schema":{"type":"object","properties":{"query_root":{"type":"string","x-tethers-path":"canonical-directory"},"max_bytes":{"type":"integer","minimum":1,"maximum":67108864}},"required":["query_root","max_bytes"],"additionalProperties":false}},
        "capabilities":[{"capability_name":"fixture.ping","capability_version":1,"manifest_path":"manifests/fixture-ping.json","manifest_digest":manifest["digest"],"provider_operation_name":"fixture_ping"}],
        "payload_index":[{"path":"manifests/fixture-ping.json","sha256":digest(&manifest_bytes),"size_bytes":manifest_bytes.len(),"role":"capability_manifest"},{"path":"provider/tethers-stdio-fixture.exe","sha256":digest(provider_bytes),"size_bytes":provider_bytes.len(),"role":"provider_executable"}]
    });
    let plug_bytes = serde_json_canonicalizer::to_vec(&plug).map_err(|error| error.to_string())?;
    let options = zip::write::FileOptions::<()>::default().last_modified_time(
        zip::DateTime::from_date_and_time(1980, 1, 1, 0, 0, 0)
            .map_err(|error| error.to_string())?,
    );
    let mut archive = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (path, bytes) in [
        ("plug.json", plug_bytes.as_slice()),
        ("manifests/fixture-ping.json", manifest_bytes.as_slice()),
        ("provider/tethers-stdio-fixture.exe", provider_bytes),
    ] {
        archive
            .start_file(path, options)
            .map_err(|error| error.to_string())?;
        archive
            .write_all(bytes)
            .map_err(|error| error.to_string())?;
    }
    archive
        .finish()
        .map(|cursor| cursor.into_inner())
        .map_err(|error| error.to_string())
}
