//! P3 crucible for the independently built PDF reference Plug.

#![cfg(windows)]

use serde_json::{json, Value};
use sha2::Digest;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;
use tethers_reference_host::candidate::{extract_to_quarantine, CandidateRegistry};
use tethers_reference_host::conformance::{run_host_conformance, ConformanceEvidenceStore};
use tethers_reference_host::enablement::EnablementStore;
use tethers_reference_host::host_execution::execute_enabled_installed_action;
use tethers_reference_host::installed::{InstallationApprovalStore, InstalledPlugRegistry};
use tethers_reference_host::installed_provider_executor::InstalledProviderExecutor;
use tethers_reference_host::launch_profile::PreparedSupervisedLaunch;
use tethers_reference_host::operational_scope::OperationalScopeEvidence;
use tethers_reference_host::policy::CapabilityRequirement;
use tethers_reference_host::resolver;
use tethers_reference_host::trust::{
    DeveloperApprovalStore, PackageTrustEvidence, PublisherTrustStore,
};
use tethers_reference_host::trusted_store::TrustedManifestStore;

fn host_binary() -> PathBuf {
    std::env::var_os("CARGO_BIN_EXE_tethers-reference-host")
        .or_else(|| std::env::var_os("CARGO_BIN_EXE_tethers_reference_host"))
        .map(PathBuf::from)
        .or_else(|| {
            std::env::current_exe().ok().and_then(|path| {
                path.parent()?
                    .parent()
                    .map(|dir| dir.join("tethers-reference-host.exe"))
            })
        })
        .expect("compiled reference host binary")
}

fn provider_binary() -> PathBuf {
    PathBuf::from(
        std::env::var_os("TETHERS_PDF_REFERENCE_PROVIDER_EXE")
            .expect("P3 provider path is required"),
    )
}

fn parse(output: &std::process::Output) -> Value {
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
    let text = String::from_utf8(output.stdout.clone()).unwrap();
    assert_eq!(text.lines().count(), 1);
    serde_json::from_str(text.trim()).unwrap()
}

fn author_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("reference-plugs/pdf-tools/author")
}

fn make_source(root: &Path) -> PathBuf {
    let source = root.join("source");
    fs::create_dir_all(source.join("manifests")).unwrap();
    fs::create_dir_all(source.join("provider")).unwrap();
    fs::copy(author_root().join("plug.json"), source.join("plug.json")).unwrap();
    fs::copy(
        author_root().join("manifests/pdf-inspect-v1.json"),
        source.join("manifests/pdf-inspect-v1.json"),
    )
    .unwrap();
    fs::copy(
        provider_binary(),
        source.join("provider/pdf_tools_provider.exe"),
    )
    .unwrap();
    source
}

fn package_from_author(root: &Path) -> PathBuf {
    let source = make_source(root);
    let package = root.join("pdf-tools.tetherplug");
    let output = Command::new(host_binary())
        .args(["plug", "pack", "--source"])
        .arg(source)
        .args(["--output"])
        .arg(&package)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    package
}

#[test]
#[ignore = "requires TETHERS_PDF_REFERENCE_PROVIDER_EXE from the standalone provider build"]
fn p3_pdf_reference_plug_public_crucible() {
    let root = std::env::temp_dir().join(format!("tethers-p3-pdf-{}", uuid::Uuid::new_v4()));
    let temp = root.join("temp");
    fs::create_dir_all(&temp).unwrap();
    let source = make_source(&root);
    let package = root.join("pdf-tools.tetherplug");
    let plug_before = fs::read(source.join("plug.json")).unwrap();
    let manifest_before = fs::read(source.join("manifests/pdf-inspect-v1.json")).unwrap();
    let provider_before = fs::read(source.join("provider/pdf_tools_provider.exe")).unwrap();

    let packed = Command::new(host_binary())
        .args(["plug", "pack", "--source"])
        .arg(&source)
        .args(["--output"])
        .arg(&package)
        .output()
        .unwrap();
    assert_eq!(packed.status.code(), Some(0));
    let packed = parse(&packed);
    assert_eq!(packed["data"]["package_id"], "tethers.pdf-tools");
    assert_eq!(packed["data"]["provider_id"], "tethers-pdf-provider");
    let semantic = packed["data"]["semantic_package_digest"]
        .as_str()
        .unwrap()
        .to_owned();
    let package_before = fs::read(&package).unwrap();

    let inspected = Command::new(host_binary())
        .args(["plug", "inspect", "--package"])
        .arg(&package)
        .output()
        .unwrap();
    assert_eq!(inspected.status.code(), Some(0));
    let inspected = parse(&inspected);
    let inspection = &inspected["data"]["inspection"];
    assert_eq!(inspection["capabilities"][0]["name"], "pdf.inspect");
    assert_eq!(
        inspection["capabilities"][0]["manifest_digest"],
        "sha256:26da081128608859c1259da7ddd784d343241504cb47339ca54a9b5979b6297c"
    );
    assert_eq!(inspection["package"]["semantic_digest"], semantic);

    let denied = Command::new(host_binary())
        .args(["plug", "conform", "--package"])
        .arg(&package)
        .env("TEMP", &temp)
        .env("TMP", &temp)
        .output()
        .unwrap();
    assert_eq!(denied.status.code(), Some(5));
    let denied = parse(&denied);
    assert_eq!(denied["status"], "approval_required");
    assert_eq!(
        denied["error"]["code"],
        "conformance_execution_approval_required"
    );

    let conformed = Command::new(host_binary())
        .args(["plug", "conform", "--package"])
        .arg(&package)
        .arg("--allow-non-isolated-supervised-execution")
        .env("TEMP", &temp)
        .env("TMP", &temp)
        .output()
        .unwrap();
    assert_eq!(conformed.status.code(), Some(0));
    let conformed = parse(&conformed);
    assert_eq!(conformed["data"]["conformance"]["disposition"], "passed");
    assert_eq!(conformed["data"]["launch_profile"]["isolated"], false);
    assert!(conformed["data"]["launch_profile"]["limitation"]
        .as_str()
        .unwrap()
        .contains("not isolated"));
    assert_eq!(conformed["data"]["semantic_package_digest"], semantic);

    assert_eq!(fs::read(source.join("plug.json")).unwrap(), plug_before);
    assert_eq!(
        fs::read(source.join("manifests/pdf-inspect-v1.json")).unwrap(),
        manifest_before
    );
    assert_eq!(
        fs::read(source.join("provider/pdf_tools_provider.exe")).unwrap(),
        provider_before
    );
    assert_eq!(fs::read(&package).unwrap(), package_before);
    assert!(!json!([packed, inspected, conformed])
        .to_string()
        .contains("conformance-scratch"));
    let _ = fs::remove_dir_all(root);
}

#[test]
#[ignore = "requires TETHERS_PDF_REFERENCE_PROVIDER_EXE from the standalone provider build"]
fn p3_real_installed_pdf_execution_uses_generic_executor() {
    let root = std::env::temp_dir().join(format!("tethers-p3-installed-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    let package = package_from_author(&root);
    let report = tethers_reference_host::package::inspect(&package).unwrap();
    let quarantine = root.join("quarantine");
    let candidate = CandidateRegistry::open(&root.join("candidates"), &quarantine)
        .unwrap()
        .create(&extract_to_quarantine(&report, &quarantine).unwrap())
        .unwrap();
    let developers = DeveloperApprovalStore::open(&root.join("developers")).unwrap();
    let developer = developers
        .approve_exact_digest(&candidate.semantic_package_digest, "Matthew")
        .unwrap();
    let trust = PackageTrustEvidence::unsigned(&developer).unwrap();
    let publishers = PublisherTrustStore::open(&root.join("publishers")).unwrap();
    let prepared = PreparedSupervisedLaunch::prepare(
        &candidate,
        &quarantine,
        &root.join("scratch"),
        Duration::from_secs(10),
    )
    .unwrap();
    let conformance = run_host_conformance(
        &prepared,
        &candidate,
        &quarantine,
        &trust,
        &publishers,
        &developers,
        "tethers-reference-host@0.3-p3",
    )
    .unwrap();
    ConformanceEvidenceStore::open(&root.join("conformance"))
        .unwrap()
        .create(&conformance)
        .unwrap();
    let approval = InstallationApprovalStore::open(&root.join("approvals"))
        .unwrap()
        .approve(
            &candidate,
            &quarantine,
            &trust,
            &publishers,
            &developers,
            &prepared.evidence,
            &conformance,
            "Matthew",
        )
        .unwrap();
    let registry =
        InstalledPlugRegistry::open(&root.join("install"), &root.join("installed-records"))
            .unwrap();
    let installed = registry
        .install_disabled(
            &candidate,
            &quarantine,
            &trust,
            &publishers,
            &developers,
            &prepared.evidence,
            &conformance,
            &approval,
        )
        .unwrap();
    let query = root.join("query");
    fs::create_dir_all(&query).unwrap();
    let pdf = b"%PDF-1.4\n1 0 obj\n<< /Type /Page /Parent 99 0 R >>\nendobj\n%%EOF\n";
    fs::write(query.join("doc.pdf"), pdf).unwrap();
    let schema = json!({"type":"object","properties":{"query_root":{"type":"string","x-tethers-path":"canonical-directory"},"max_bytes":{"type":"integer","minimum":1,"maximum":67108864}},"required":["query_root","max_bytes"],"additionalProperties":false});
    let schema_digest = format!(
        "sha256:{:x}",
        sha2::Sha256::digest(serde_json_canonicalizer::to_vec(&schema).unwrap())
    );
    let scope = OperationalScopeEvidence::create(
        &installed.installed_id,
        "tethers.pdf-tools",
        "tethers-pdf-provider",
        &schema_digest,
        &json!({"query_root":query.to_string_lossy(),"max_bytes":67108864}),
        "Matthew",
    )
    .unwrap();
    let enablements = EnablementStore::open(&root.join("enablements")).unwrap();
    let enabled = enablements
        .enable(&installed, scope.clone(), "Matthew")
        .unwrap();
    let installation = registry.installation_directory(&installed).unwrap();
    let manifest = tethers_reference_host::manifest::verify_manifest(
        &fs::read_to_string(installation.join("manifests/pdf-inspect-v1.json")).unwrap(),
    )
    .unwrap();
    let mut executor = InstalledProviderExecutor::launch_from_installed(
        &installed,
        &installation,
        &trust,
        &publishers,
        &developers,
        &conformance,
        &approval,
        &enabled,
        &scope,
        manifest.clone(),
    )
    .unwrap();
    let mut manifests = TrustedManifestStore::new();
    manifests.insert(manifest).unwrap();
    let snapshot = enablements
        .snapshot(&installed.installed_id)
        .unwrap()
        .unwrap();
    let resolved = resolver::resolve_capability(
        &manifests,
        &snapshot.provider_availability(),
        "pdf.inspect",
        1,
        Some("tethers-pdf-provider"),
    )
    .unwrap();
    let mut response = json!({"status":"matched","protocol_version":"0.1","evaluation_id":"eval-p3-pdf-inspect","event_id":"evt-p3-pdf-inspect","tether_id":"p3-pdf-reference","tether_version":"1","trail":[],"plan":{"id":"plan-p3-pdf-inspect","actions":[{"action_id":"action-p3-pdf-inspect","capability":"pdf.inspect","capability_version":"1.0.0","bridge_capability_version":1,"bridge_provider_identity":"tethers-pdf-provider","manifest_digest":resolved.manifest_digest(),"arguments":{"path":"doc.pdf"}}]}});
    let replay_root = root.join("replay");
    fs::create_dir_all(&replay_root).unwrap();
    let acl_script = format!(
        "$p='{}'; $identity=[System.Security.Principal.WindowsIdentity]::GetCurrent().Name; $acl=[System.Security.AccessControl.DirectorySecurity]::new(); $acl.SetAccessRuleProtection($true,$false); $inherit=[System.Security.AccessControl.InheritanceFlags]::ContainerInherit -bor [System.Security.AccessControl.InheritanceFlags]::ObjectInherit; foreach($t in @($identity,'NT AUTHORITY\\SYSTEM','BUILTIN\\Administrators')) {{ $acl.AddAccessRule([System.Security.AccessControl.FileSystemAccessRule]::new($t,'FullControl',$inherit,'None','Allow')) }}; Set-Acl -LiteralPath $p -AclObject $acl",
        replay_root.to_string_lossy()
    );
    assert!(Command::new("pwsh")
        .args(["-NoProfile", "-Command", &acl_script])
        .status()
        .unwrap()
        .success());
    let provision = Command::new(host_binary())
        .args(["provision-replay", replay_root.to_str().unwrap()])
        .status()
        .unwrap();
    assert!(provision.success());
    let trail_path = root.join("trail.jsonl");
    let shared = execute_enabled_installed_action(
        &mut response,
        &[CapabilityRequirement::new("pdf.inspect", 1)],
        &resolved,
        &snapshot,
        &mut executor,
        &trail_path,
        &replay_root,
        "evt-p3-pdf-inspect",
    )
    .unwrap();
    assert_eq!(
        shared.outcome,
        tethers_reference_host::SharedExecutionOutcome::Completed
    );
    assert!(shared.execution_id.is_some());
    assert_eq!(response["execution_status"], "completed");
    assert_eq!(
        response["result_anchor"]["event_name"],
        "capability.succeeded"
    );
    let result = &response["result_anchor"]["facts"]["result"];
    assert_eq!(result["is_pdf"], true);
    assert_eq!(result["pdf_version"], "1.4");
    assert_eq!(result["page_count"], 1);
    assert_eq!(result["size_bytes"], pdf.len() as u64);
    assert_eq!(result["path"], "doc.pdf");
    assert!(result["sha256"].as_str().unwrap().starts_with("sha256:"));
    assert!(fs::read_to_string(&trail_path)
        .unwrap()
        .contains("pdf.inspect"));
    prepared.cleanup_scratch().unwrap();
    let _ = fs::remove_dir_all(root);
}
