#![cfg(windows)]

use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Duration;
use tethers_reference_host::candidate::{extract_to_quarantine, CandidateRegistry};
use tethers_reference_host::conformance::{run_host_conformance, ConformanceEvidenceStore};
use tethers_reference_host::enablement::EnablementStore;
use tethers_reference_host::host_execution::execute_enabled_installed_action;
use tethers_reference_host::installed::{InstallationApprovalStore, InstalledPlugRegistry};
use tethers_reference_host::launch_profile::PreparedSupervisedLaunch;
use tethers_reference_host::operational_scope::OperationalScopeEvidence;
use tethers_reference_host::package;
use tethers_reference_host::pdf_tools::{self, InstalledPdfToolsExecutor};

fn make_pdf_scope(
    installed_id: &str,
    query_root: &Path,
    max_bytes: u64,
) -> OperationalScopeEvidence {
    OperationalScopeEvidence::create(
        installed_id,
        "tethers.pdf-tools",
        "tethers-pdf-provider",
        "sha256:eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
        &serde_json::json!({"query_root": query_root.to_string_lossy(), "max_bytes": max_bytes}),
        "Matthew",
    )
    .unwrap()
}
use tethers_reference_host::policy::CapabilityRequirement;
use tethers_reference_host::resolver;
use tethers_reference_host::trust::{
    DeveloperApprovalStore, PackageTrustEvidence, PublisherTrustStore,
};
use tethers_reference_host::trusted_store::TrustedManifestStore;
use uuid::Uuid;

#[test]
fn installed_pdf_plug_lifecycle() {
    let base = std::env::temp_dir().join(format!("tethers-j23c3-{}", Uuid::new_v4()));
    fs::create_dir_all(&base).unwrap();

    // -- Build the package --
    let archive = base.join("pdf-tools.tetherplug");
    let provider_bytes =
        fs::read(env!("CARGO_BIN_EXE_pdf_tools_provider")).expect("compiled provider");
    fs::write(
        &archive,
        pdf_tools::build_reference_package(&provider_bytes).unwrap(),
    )
    .unwrap();

    // -- Inspect --
    let report = package::inspect(&archive).unwrap();
    assert_eq!(report.package.package_id, "tethers.pdf-tools");
    assert_eq!(report.package.package_version, "1.0.0");
    assert_eq!(report.provider_id, "tethers-pdf-provider");

    // -- Quarantine --
    let quarantined = extract_to_quarantine(&report, &base.join("quarantine")).unwrap();

    // -- Candidate --
    let candidates =
        CandidateRegistry::open(&base.join("candidates"), &base.join("quarantine")).unwrap();
    let candidate = candidates.create(&quarantined).unwrap();
    assert_eq!(
        candidate.launch_arguments,
        vec!["--query-root", "__TETHERS_PDF_QUERY_ROOT__"]
    );

    // -- Developer trust --
    let developers = DeveloperApprovalStore::open(&base.join("developer")).unwrap();
    let developer = developers
        .approve_exact_digest(&candidate.semantic_package_digest, "Matthew")
        .unwrap();
    let trust = PackageTrustEvidence::unsigned(&developer).unwrap();
    let publishers = PublisherTrustStore::open(&base.join("publishers")).unwrap();

    // -- Conformance --
    let prepared = PreparedSupervisedLaunch::prepare(
        &candidate,
        &base.join("quarantine"),
        &base.join("scratch"),
        Duration::from_secs(10),
    )
    .unwrap();
    assert_eq!(
        prepared.evidence.arguments,
        vec!["--query-root", "__TETHERS_PDF_QUERY_ROOT__"]
    );
    let conformance = run_host_conformance(
        &prepared,
        &candidate,
        &base.join("quarantine"),
        &trust,
        &publishers,
        &developers,
        "tethers-reference-host@0.2.0+j23c3",
    )
    .unwrap();
    assert_eq!(
        conformance.disposition,
        tethers_reference_host::conformance::ConformanceDisposition::Passed
    );
    assert_eq!(conformance.retry_count, 0);
    ConformanceEvidenceStore::open(&base.join("conformance"))
        .unwrap()
        .create(&conformance)
        .unwrap();

    // -- Installation approval --
    let approval = InstallationApprovalStore::open(&base.join("approvals"))
        .unwrap()
        .approve(
            &candidate,
            &base.join("quarantine"),
            &trust,
            &publishers,
            &developers,
            &prepared.evidence,
            &conformance,
            "Matthew",
        )
        .unwrap();

    // -- Install disabled --
    let registry =
        InstalledPlugRegistry::open(&base.join("install"), &base.join("installed-records"))
            .unwrap();
    let installed = registry
        .install_disabled(
            &candidate,
            &base.join("quarantine"),
            &trust,
            &publishers,
            &developers,
            &prepared.evidence,
            &conformance,
            &approval,
        )
        .unwrap();
    assert_eq!(installed.state, "present_disabled");
    assert_eq!(installed.active_binding_count(), 0);
    assert_eq!(installed.disabled_bindings.len(), 1);
    assert_eq!(
        installed.disabled_bindings[0].capability_name,
        "pdf.inspect"
    );
    assert_eq!(installed.disabled_bindings[0].capability_version, 1);

    let installation_dir = registry.installation_directory(&installed).unwrap();
    let provider_exe = installation_dir.join("provider/pdf_tools_provider.exe");
    let metadata = fs::metadata(&provider_exe).unwrap();
    assert!(metadata.permissions().readonly());

    // -- Operational scope --
    let query_dir = base.join("scope");
    fs::create_dir_all(&query_dir).unwrap();

    // Write a tiny generated PDF
    let pdf_path = query_dir.join("doc.pdf");
    let pdf_bytes = b"%PDF-1.4\n1 0 obj\n<< /Type /Page /Parent 99 0 R >>\nendobj\n99 0 obj\n<< /Type /Pages /Kids [1 0 R] /Count 1 >>\nendobj\nxref\n0 2\ntrailer\n<< /Root 99 0 R >>\nstartxref\n0\n%%EOF\n";
    fs::write(&pdf_path, pdf_bytes).unwrap();

    // Write an oversized file
    let big_path = query_dir.join("big.pdf");
    let oversized_bytes = vec![b'x'; 2000];
    let operational_max: u64 = 1024;
    fs::write(&big_path, oversized_bytes).unwrap();

    let scope = make_pdf_scope(&installed.installed_id, &query_dir, operational_max);
    assert_eq!(scope.schema_version, 1);
    assert_eq!(scope.installed_id(), &installed.installed_id);

    // -- Enablement --
    let enablements = EnablementStore::open(&base.join("enablements")).unwrap();
    assert!(!enablements.is_available(&installed.installed_id).unwrap());
    let enabled = enablements
        .enable(&installed, scope.clone(), "Matthew")
        .unwrap();
    assert_eq!(enabled.operational_scope_digest, scope.integrity_digest);

    let snapshot = enablements
        .snapshot(&installed.installed_id)
        .unwrap()
        .unwrap();
    assert!(snapshot
        .provider_availability()
        .is_available("tethers-pdf-provider"));
    assert!(snapshot.contains(
        "pdf.inspect",
        1,
        "pdf_inspect",
        "sha256:26da081128608859c1259da7ddd784d343241504cb47339ca54a9b5979b6297c"
    ));

    // -- Wrong scope refused --
    let alt_dir = base.join("alt-scope");
    fs::create_dir_all(&alt_dir).unwrap();
    let alt_scope = make_pdf_scope(&installed.installed_id, &alt_dir, operational_max);
    assert_ne!(alt_scope.integrity_digest, scope.integrity_digest);
    let alt_launch = InstalledPdfToolsExecutor::launch_from_installed(
        &installed,
        &installation_dir,
        &trust,
        &publishers,
        &developers,
        &conformance,
        &approval,
        &enabled,
        &alt_scope,
    );
    assert!(alt_launch.is_err());

    // -- Mismatched operational scope refused --
    let mismatched_scope = make_pdf_scope(&installed.installed_id, &query_dir, operational_max + 1);
    assert_eq!(mismatched_scope.integrity_digest, scope.integrity_digest);
    let mismatched_launch = InstalledPdfToolsExecutor::launch_from_installed(
        &installed,
        &installation_dir,
        &trust,
        &publishers,
        &developers,
        &conformance,
        &approval,
        &enabled,
        &mismatched_scope,
    );
    let mismatched_error = match mismatched_launch {
        Ok(_) => panic!("mismatched operational scope was accepted"),
        Err(error) => error.to_string(),
    };
    assert!(mismatched_error.contains("enablement scope does not match supplied scope"));

    // -- Successful installed launch --
    let mut executor = InstalledPdfToolsExecutor::launch_from_installed(
        &installed,
        &installation_dir,
        &trust,
        &publishers,
        &developers,
        &conformance,
        &approval,
        &enabled,
        &scope,
    )
    .unwrap();

    // -- Direct operational call --
    let inspect_result = executor
        .call("pdf_inspect", &serde_json::json!({"path":"doc.pdf"}))
        .unwrap();
    assert_eq!(inspect_result["result"]["path"], "doc.pdf");
    assert!(inspect_result["result"]["size_bytes"].as_u64().is_some());
    let sha = inspect_result["result"]["sha256"].as_str().unwrap();
    assert!(sha.starts_with("sha256:"));
    assert_eq!(sha.len(), 71);
    assert_eq!(inspect_result["result"]["is_pdf"], true);
    assert_eq!(inspect_result["result"]["pdf_version"], "1.4");
    assert!(inspect_result["result"]["page_count"].as_u64().unwrap() >= 1);

    // -- Oversized file refused --
    let oversized = executor
        .call("pdf_inspect", &serde_json::json!({"path":"big.pdf"}))
        .unwrap();
    assert!(oversized.get("error").is_some());
    assert!(oversized.get("result").is_none());

    // -- Manifest resolution --
    let manifest_text =
        fs::read_to_string(installation_dir.join("manifests/pdf-inspect-v1.json")).unwrap();
    let verified = tethers_reference_host::manifest::verify_manifest(&manifest_text).unwrap();
    assert_eq!(verified.manifest().binding.tool_name, "pdf_inspect");
    let mut manifests = TrustedManifestStore::new();
    manifests.insert(verified).unwrap();
    let availability = snapshot.provider_availability();
    let resolved = resolver::resolve_capability(
        &manifests,
        &availability,
        "pdf.inspect",
        1,
        Some("tethers-pdf-provider"),
    )
    .unwrap();
    assert_eq!(
        resolved.manifest_digest(),
        "sha256:26da081128608859c1259da7ddd784d343241504cb47339ca54a9b5979b6297c"
    );

    // -- Shared execution --
    let mut response = serde_json::json!({
        "status":"matched",
        "protocol_version":"0.1",
        "evaluation_id":"eval-j23c3-pdf-inspect",
        "event_id":"evt-j23c3-pdf-inspect",
        "tether_id":"j23c3-pdf-tools",
        "tether_version":"1",
        "trail":[],
        "plan":{
            "id":"plan-j23c3-pdf-inspect",
            "actions":[{
                "action_id":"action-j23c3-pdf-inspect",
                "capability":"pdf.inspect",
                "capability_version":"1.0.0",
                "bridge_capability_version":1,
                "bridge_provider_identity":"tethers-pdf-provider",
                "manifest_digest":resolved.manifest_digest(),
                "arguments":{"path":"doc.pdf"}
            }]
        }
    });

    let replay_root = base.join("replay");
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
    let provision = Command::new(env!("CARGO_BIN_EXE_tethers-reference-host"))
        .args(["provision-replay", replay_root.to_str().unwrap()])
        .status()
        .unwrap();
    assert!(provision.success());

    let trail_path = base.join("trail.jsonl");
    let shared = execute_enabled_installed_action(
        &mut response,
        &[CapabilityRequirement::new("pdf.inspect", 1)],
        &resolved,
        &snapshot,
        &mut executor,
        &trail_path,
        &replay_root,
        "evt-j23c3-pdf-inspect",
    )
    .unwrap();
    assert_eq!(
        shared.outcome,
        tethers_reference_host::SharedExecutionOutcome::Completed
    );
    assert_eq!(
        response["result_anchor"]["event_name"],
        "capability.succeeded"
    );
    assert!(shared.execution_id.is_some());

    // -- Trail proof --
    let trail_text = fs::read_to_string(&trail_path).unwrap();
    let entries: Vec<Value> = trail_text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .collect();
    assert!(entries.len() >= 2);

    let intent = entries
        .iter()
        .find(|entry| {
            entry.get("capability_name").and_then(Value::as_str) == Some("pdf.inspect")
                && entry.get("provider_identity").and_then(Value::as_str)
                    == Some("tethers-pdf-provider")
                && entry.get("action_id").and_then(Value::as_str)
                    == Some("action-j23c3-pdf-inspect")
                && entry.pointer("/arguments/path").and_then(Value::as_str) == Some("doc.pdf")
        })
        .expect("intent Trail entry not found");

    let intent_index = entries.iter().position(|e| e == intent).unwrap();

    let outcome = entries
        .iter()
        .skip(intent_index + 1)
        .find(|entry| {
            entry.get("action_id").and_then(Value::as_str) == Some("action-j23c3-pdf-inspect")
                && entry.get("status").and_then(Value::as_str) == Some("succeeded")
        })
        .expect("succeeded outcome Trail entry not found");

    assert_eq!(
        intent["manifest_digest"].as_str().unwrap(),
        resolved.manifest_digest()
    );
    assert_eq!(outcome["result"]["is_pdf"], true);
    assert_eq!(outcome["result"]["pdf_version"], "1.4");
    assert!(outcome["result"]["sha256"]
        .as_str()
        .unwrap()
        .starts_with("sha256:"));

    // -- Replay --
    let mut replay_response = response.clone();
    let replayed = execute_enabled_installed_action(
        &mut replay_response,
        &[CapabilityRequirement::new("pdf.inspect", 1)],
        &resolved,
        &snapshot,
        &mut executor,
        &trail_path,
        &replay_root,
        "evt-j23c3-pdf-inspect",
    )
    .unwrap();
    assert_eq!(
        replayed.outcome,
        tethers_reference_host::SharedExecutionOutcome::Replay(
            tethers_reference_host::replay_runtime::ReplayDispatchResult::BlockedCompletedSuccess
        )
    );

    // -- Disablement --
    drop(executor);
    enablements.disable(&installed, "Matthew").unwrap();
    assert!(!enablements.is_available(&installed.installed_id).unwrap());
    assert!(enablements
        .snapshot(&installed.installed_id)
        .unwrap()
        .is_none());

    // -- Cleanup --
    prepared.cleanup_scratch().unwrap();
    fs::remove_dir_all(base).unwrap();
}
