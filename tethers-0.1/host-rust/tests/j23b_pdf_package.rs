//! J23B deterministic PDF Tools `.tetherplug` package conformance.
//!
//! Builds the real provider binary selected by Cargo and packages it through the
//! host-owned `pdf_tools::build_reference_package` builder, then inspects the
//! resulting archive with the same non-executing evidence path the host uses.
//! No installation, enablement, launch, or dispatch is performed here.

use std::collections::BTreeMap;
use std::fs;
use std::io::{Cursor, Read};
use std::path::Path;

use serde_json::Value;
use sha2::{Digest, Sha256};
use tethers_reference_host::manifest;
use tethers_reference_host::package;
use tethers_reference_host::pdf_tools;
use zip::ZipArchive;

fn provider_bytes() -> Vec<u8> {
    fs::read(env!("CARGO_BIN_EXE_pdf_tools_provider")).expect("compiled pdf_tools_provider binary")
}

fn digest(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn read_archive(archive: &Path) -> BTreeMap<String, Vec<u8>> {
    let data = fs::read(archive).expect("archive read");
    let mut za = ZipArchive::new(Cursor::new(data)).expect("archive must be a valid zip");
    let mut entries: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    for index in 0..za.len() {
        let mut entry = za.by_index(index).expect("archive entry");
        let name = entry.name().to_string();
        let mut buf = Vec::new();
        entry.read_to_end(&mut buf).expect("archive entry read");
        entries.insert(name, buf);
    }
    entries
}

#[test]
fn package_build_is_deterministic_and_matches_contract() {
    let bytes = provider_bytes();
    assert!(
        !bytes.is_empty(),
        "compiled provider binary must not be empty"
    );

    let first = pdf_tools::build_reference_package(&bytes).unwrap();
    let second = pdf_tools::build_reference_package(&bytes).unwrap();

    // 1. building twice from identical provider bytes returns identical bytes.
    assert_eq!(first, second, "package bytes must be deterministic");

    let dir = std::env::temp_dir().join(format!("tethers-j23b-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&dir).unwrap();
    let archive = dir.join("pdf-tools.tetherplug");
    fs::write(&archive, &first).unwrap();

    // 2. package::inspect accepts the generated package.
    let report = package::inspect(&archive).unwrap();

    // 3-6. identity.
    assert_eq!(report.package.package_id, "tethers.pdf-tools");
    assert_eq!(report.package.package_version, "1.0.0");
    assert_eq!(report.provider_id, "tethers-pdf-provider");
    assert_eq!(report.provider_version, "1.0.0");

    // 7-10. launch contract.
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

    // 11-14. exactly one capability, declared correctly.
    assert_eq!(report.capabilities.len(), 1);
    assert_eq!(report.capabilities[0].name, "pdf.inspect");
    assert_eq!(report.capabilities[0].version, 1);
    assert_eq!(report.capabilities[0].operation, "pdf_inspect");
    assert_eq!(
        report.capabilities[0].manifest_path,
        "manifests/pdf-inspect-v1.json"
    );

    // 15. capability manifest digest equals the newly frozen digest.
    let new_digest = "sha256:26da081128608859c1259da7ddd784d343241504cb47339ca54a9b5979b6297c";
    assert_eq!(report.capabilities[0].manifest_digest, new_digest);

    // 16-17. exactly two payload entries, sorted by path.
    assert_eq!(report.payloads.len(), 2);
    let paths: Vec<String> = report.payloads.iter().map(|p| p.path.clone()).collect();
    assert!(
        paths.windows(2).all(|w| w[0] <= w[1]),
        "payload index must be path sorted"
    );
    assert_eq!(
        paths,
        vec![
            "manifests/pdf-inspect-v1.json".to_string(),
            "provider/pdf_tools_provider.exe".to_string()
        ]
    );

    let entries = read_archive(&archive);

    // 18. both payload sizes are exact.
    assert_eq!(
        report.payloads[0].size_bytes,
        entries["manifests/pdf-inspect-v1.json"].len() as u64
    );
    assert_eq!(
        report.payloads[1].size_bytes,
        entries["provider/pdf_tools_provider.exe"].len() as u64
    );
    assert_eq!(report.payloads[1].size_bytes, bytes.len() as u64);

    // 19. both payload SHA-256 values match their archived bytes.
    assert_eq!(
        report.payloads[0].sha256,
        digest(&entries["manifests/pdf-inspect-v1.json"])
    );
    assert_eq!(
        report.payloads[1].sha256,
        digest(&entries["provider/pdf_tools_provider.exe"])
    );

    // 20. the archived provider bytes equal the compiled provider binary bytes.
    assert_eq!(entries["provider/pdf_tools_provider.exe"], bytes);

    // 21. the archived manifest verifies through manifest::verify_manifest().
    let manifest_text =
        String::from_utf8(entries["manifests/pdf-inspect-v1.json"].clone()).unwrap();
    let verified = manifest::verify_manifest(&manifest_text).unwrap();
    assert_eq!(verified.capability_name(), "pdf.inspect");
    assert_eq!(verified.verified_digest(), new_digest);

    // 22. archived manifest and committed frozen manifest agree on every field.
    let committed_text = include_str!("../../protocol/capability-manifests/pdf-inspect-v1.json");
    let committed = manifest::verify_manifest(committed_text).unwrap();
    assert_eq!(verified.capability_name(), committed.capability_name());
    assert_eq!(
        verified.capability_version(),
        committed.capability_version()
    );
    assert_eq!(verified.verified_digest(), committed.verified_digest());
    assert_eq!(
        verified.manifest().input_schema,
        committed.manifest().input_schema
    );
    assert_eq!(
        verified.manifest().output_schema,
        committed.manifest().output_schema
    );
    assert_eq!(
        verified.manifest().provider.identity,
        "tethers-pdf-provider"
    );
    assert_eq!(
        verified.manifest().provider.identity,
        committed.manifest().provider.identity
    );
    assert_eq!(verified.manifest().binding.tool_name, "pdf_inspect");
    assert_eq!(
        verified.manifest().binding.tool_name,
        committed.manifest().binding.tool_name
    );

    // 23-24. no signature files are present.
    assert!(!report.signatures_present);
    assert!(report.signature_files.is_empty());

    // 25. the package contains no unindexed or additional payload.
    let archived_payload_paths: Vec<String> = entries
        .keys()
        .filter(|k| *k != "plug.json" && !k.starts_with("signatures/"))
        .cloned()
        .collect();
    assert_eq!(archived_payload_paths.len(), 2);
    for path in &archived_payload_paths {
        assert!(
            report.payloads.iter().any(|pe| &pe.path == path),
            "unindexed payload entry: {path}"
        );
    }

    // 26. the old PDF manifest digest is no longer used.
    let old_digest = "sha256:fe8d4eb7a36f8961baea94175f0eff979364322534ca27a305486688e3b268b3";
    assert_ne!(report.capabilities[0].manifest_digest, old_digest);
    assert_ne!(verified.verified_digest(), old_digest);

    fs::remove_dir_all(&dir).unwrap();
}
