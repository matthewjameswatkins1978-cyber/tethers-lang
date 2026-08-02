#![cfg(windows)]

use serde_json::Value;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::time::Duration;
use tethers_reference_host::candidate::{extract_to_quarantine, CandidateRegistry};
use tethers_reference_host::conformance::{run_host_conformance, ConformanceEvidenceStore};
use tethers_reference_host::enablement::EnablementStore;
use tethers_reference_host::file_tools::FileToolsExecutor;
use tethers_reference_host::file_tools::{
    build_reference_package, manifest_with_digest, metadata_manifest_without_digest,
    move_manifest_without_digest, OperationalScopeBinding,
};
use tethers_reference_host::host_execution::execute_enabled_file_tools_action;
use tethers_reference_host::installed::{InstallationApprovalStore, InstalledPlugRegistry};
use tethers_reference_host::launch_profile::PreparedSupervisedLaunch;
use tethers_reference_host::package;
use tethers_reference_host::policy::CapabilityRequirement;
use tethers_reference_host::resolver;
use tethers_reference_host::trust::{
    DeveloperApprovalStore, PackageTrustEvidence, PublisherTrustStore,
};
use tethers_reference_host::trusted_store::TrustedManifestStore;
use uuid::Uuid;

fn request(child: &mut std::process::Child, message: Value) -> Value {
    let stdin = child.stdin.as_mut().unwrap();
    writeln!(stdin, "{}", serde_json::to_string(&message).unwrap()).unwrap();
    stdin.flush().unwrap();
    let stdout = child.stdout.as_mut().unwrap();
    let mut line = String::new();
    BufReader::new(stdout).read_line(&mut line).unwrap();
    serde_json::from_str(&line).unwrap()
}

#[test]
fn native_file_tools_provider_performs_query_and_non_overwriting_move() {
    let root = std::env::temp_dir().join(format!("tethers-m4-provider-{}", Uuid::new_v4()));
    let query = root.join("query");
    let source = root.join("source");
    let destination = root.join("destination");
    fs::create_dir_all(&query).unwrap();
    fs::create_dir_all(&source).unwrap();
    fs::create_dir_all(&destination).unwrap();
    fs::write(query.join("hello.txt"), b"hello").unwrap();
    fs::write(source.join("move.txt"), b"move").unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_file_tools_provider"))
        .args([
            "--query-root",
            query.to_str().unwrap(),
            "--source-root",
            source.to_str().unwrap(),
            "--destination-root",
            destination.to_str().unwrap(),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let initialize = request(
        &mut child,
        serde_json::json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25"}}),
    );
    assert_eq!(initialize["result"]["protocolVersion"], "2025-11-25");
    let stdin = child.stdin.as_mut().unwrap();
    writeln!(
        stdin,
        "{{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}}"
    )
    .unwrap();
    stdin.flush().unwrap();
    let tools = request(
        &mut child,
        serde_json::json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}),
    );
    assert_eq!(tools["result"]["tools"].as_array().unwrap().len(), 2);
    let metadata = request(
        &mut child,
        serde_json::json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"file_metadata","arguments":{"path":"hello.txt","include_content":true}}}),
    );
    assert_eq!(metadata["result"]["content"], "hello");
    fs::write(destination.join("move.txt"), b"existing").unwrap();
    let moved = request(
        &mut child,
        serde_json::json!({"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"file_move","arguments":{"source_path":"move.txt","destination_path":"move.txt"}}}),
    );
    assert_eq!(moved["error"]["code"], -32602);
    assert!(source.join("move.txt").exists());
    drop(child.stdin.take());
    let _ = child.wait();
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn m4_contract_and_package_digest_material_is_stable() {
    let metadata = manifest_with_digest(metadata_manifest_without_digest()).unwrap();
    let movement = manifest_with_digest(move_manifest_without_digest()).unwrap();
    assert_eq!(metadata["capability_name"], "file.metadata");
    assert_eq!(movement["capability_name"], "file.move");
    assert_eq!(
        build_reference_package(b"provider"),
        build_reference_package(b"provider")
    );
}

#[test]
fn operational_scope_is_host_owned_and_integrity_bound() {
    let root = std::env::temp_dir().join(format!("tethers-m4-scope-{}", Uuid::new_v4()));
    let query = root.join("query");
    let source = root.join("source");
    let destination = root.join("destination");
    fs::create_dir_all(&query).unwrap();
    fs::create_dir_all(&source).unwrap();
    fs::create_dir_all(&destination).unwrap();
    let binding = OperationalScopeBinding::create(
        "installed-file-tools",
        "file.move",
        1,
        &query,
        &source,
        &destination,
        4096,
        "Matthew",
    )
    .unwrap();
    assert!(binding.validate().is_ok());
    let mut tampered = binding.clone();
    tampered.max_content_bytes = 4095;
    assert_eq!(tampered.validate().unwrap_err().code, "scope_invalid");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn installed_state_drives_native_operational_launch_and_disable() {
    let root = std::env::temp_dir().join(format!("tethers-m4-e2e-{}", Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    let archive = root.join("file-tools.tetherplug");
    let provider_bytes = fs::read(env!("CARGO_BIN_EXE_file_tools_provider")).unwrap();
    fs::write(&archive, build_reference_package(&provider_bytes).unwrap()).unwrap();
    let report = package::inspect(&archive).unwrap();
    let quarantined = extract_to_quarantine(&report, &root.join("quarantine")).unwrap();
    let candidates =
        CandidateRegistry::open(&root.join("candidates"), &root.join("quarantine")).unwrap();
    let candidate = candidates.create(&quarantined).unwrap();
    let developers = DeveloperApprovalStore::open(&root.join("developer")).unwrap();
    let developer = developers
        .approve_exact_digest(&candidate.semantic_package_digest, "Matthew")
        .unwrap();
    let trust = PackageTrustEvidence::unsigned(&developer).unwrap();
    let publishers = PublisherTrustStore::open(&root.join("publishers")).unwrap();
    let prepared = PreparedSupervisedLaunch::prepare(
        &candidate,
        &root.join("quarantine"),
        &root.join("scratch"),
        Duration::from_secs(5),
    )
    .unwrap();
    let conformance = run_host_conformance(
        &prepared,
        &candidate,
        &root.join("quarantine"),
        &trust,
        &publishers,
        &developers,
        "tethers-reference-host@0.2.0+m4",
    )
    .unwrap();
    assert_eq!(
        conformance.disposition,
        tethers_reference_host::conformance::ConformanceDisposition::Passed
    );
    ConformanceEvidenceStore::open(&root.join("conformance"))
        .unwrap()
        .create(&conformance)
        .unwrap();
    let approval = InstallationApprovalStore::open(&root.join("approvals"))
        .unwrap()
        .approve(
            &candidate,
            &root.join("quarantine"),
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
            &root.join("quarantine"),
            &trust,
            &publishers,
            &developers,
            &prepared.evidence,
            &conformance,
            &approval,
        )
        .unwrap();
    let scope_root = root.join("operational-scope");
    fs::create_dir_all(scope_root.join("query")).unwrap();
    fs::create_dir_all(scope_root.join("source")).unwrap();
    fs::create_dir_all(scope_root.join("destination")).unwrap();
    fs::write(scope_root.join("query/read.txt"), b"query").unwrap();
    fs::write(scope_root.join("source/move.txt"), b"move").unwrap();
    let scope = OperationalScopeBinding::create(
        &installed.installed_id,
        "file.move",
        1,
        &scope_root.join("query"),
        &scope_root.join("source"),
        &scope_root.join("destination"),
        4096,
        "Matthew",
    )
    .unwrap();
    let enablements = EnablementStore::open(&root.join("enablements")).unwrap();
    assert!(!enablements.is_available(&installed.installed_id).unwrap());
    let enabled = enablements
        .enable(&installed, scope.clone(), "Matthew")
        .unwrap();
    assert!(enablements
        .snapshot(&installed.installed_id)
        .unwrap()
        .unwrap()
        .provider_availability()
        .is_available("tethers-file-tools"));
    let directory = registry.installation_directory(&installed).unwrap();
    let mut executor = FileToolsExecutor::launch_from_installed(
        &installed,
        &directory,
        &trust,
        &publishers,
        &developers,
        &conformance,
        &approval,
        &enabled,
        &scope,
    )
    .unwrap();
    let metadata = executor
        .call(
            "file_metadata",
            &serde_json::json!({"path":"read.txt","include_content":true}),
        )
        .unwrap();
    assert_eq!(metadata["result"]["content"], "query");
    let moved = executor
        .call(
            "file_move",
            &serde_json::json!({"source_path":"move.txt","destination_path":"moved.txt"}),
        )
        .unwrap();
    assert_eq!(moved["result"]["moved"], true);
    fs::write(scope_root.join("source/host-path.txt"), b"host-path").unwrap();
    let manifest_text = fs::read_to_string(directory.join("manifests/file-move-m4.json")).unwrap();
    let verified = tethers_reference_host::manifest::verify_manifest(&manifest_text).unwrap();
    let mut manifests = TrustedManifestStore::new();
    manifests.insert(verified).unwrap();
    let snapshot = enablements
        .snapshot(&installed.installed_id)
        .unwrap()
        .unwrap();
    let availability = snapshot.provider_availability();
    let resolved = resolver::resolve_capability(
        &manifests,
        &availability,
        "file.move",
        1,
        Some("tethers-file-tools"),
    )
    .unwrap();
    let mut response = serde_json::json!({"status":"matched","protocol_version":"0.1","evaluation_id":"eval-m4-file-move","event_id":"evt-m4-file-move","tether_id":"m4-file-tools","tether_version":"1","trail":[],"plan":{"id":"plan-m4-file-move","actions":[{"action_id":"action-m4-file-move","capability":"file.move","capability_version":"1.0.0","bridge_capability_version":1,"bridge_provider_identity":"tethers-file-tools","manifest_digest":resolved.manifest_digest(),"arguments":{"source_path":"host-path.txt","destination_path":"host-path-moved.txt"}}]}});
    let replay_root = std::env::temp_dir().join(format!("tethers-m4-replay-{}", Uuid::new_v4()));
    fs::create_dir_all(&replay_root).unwrap();
    let acl_script = format!("$p='{}'; $identity=[System.Security.Principal.WindowsIdentity]::GetCurrent().Name; $acl=[System.Security.AccessControl.DirectorySecurity]::new(); $acl.SetAccessRuleProtection($true,$false); $inherit=[System.Security.AccessControl.InheritanceFlags]::ContainerInherit -bor [System.Security.AccessControl.InheritanceFlags]::ObjectInherit; foreach($t in @($identity,'NT AUTHORITY\\SYSTEM','BUILTIN\\Administrators')) {{ $acl.AddAccessRule([System.Security.AccessControl.FileSystemAccessRule]::new($t,'FullControl',$inherit,'None','Allow')) }}; Set-Acl -LiteralPath $p -AclObject $acl", replay_root.to_string_lossy());
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
    let trail_path = root.join("trail.jsonl");
    let shared = execute_enabled_file_tools_action(
        &mut response,
        &[CapabilityRequirement::new("file.move", 1)],
        &resolved,
        &snapshot,
        &mut executor,
        &trail_path,
        &replay_root,
        "evt-m4-file-move",
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
    assert!(scope_root.join("destination/host-path-moved.txt").exists());
    assert!(fs::read_to_string(&trail_path)
        .unwrap()
        .contains("host-path.txt"));
    let mut replay_response = response.clone();
    let replayed = execute_enabled_file_tools_action(
        &mut replay_response,
        &[CapabilityRequirement::new("file.move", 1)],
        &resolved,
        &snapshot,
        &mut executor,
        &trail_path,
        &replay_root,
        "evt-m4-file-move",
    )
    .unwrap();
    assert_eq!(
        replayed.outcome,
        tethers_reference_host::SharedExecutionOutcome::Replay(
            tethers_reference_host::replay_runtime::ReplayDispatchResult::BlockedCompletedSuccess
        )
    );
    drop(executor);
    enablements.disable(&installed, "Matthew").unwrap();
    assert!(!enablements.is_available(&installed.installed_id).unwrap());
    assert!(enablements
        .snapshot(&installed.installed_id)
        .unwrap()
        .is_none());
    prepared.cleanup_scratch().unwrap();
    fs::remove_dir_all(root).unwrap();
    fs::remove_dir_all(replay_root).unwrap();
}
