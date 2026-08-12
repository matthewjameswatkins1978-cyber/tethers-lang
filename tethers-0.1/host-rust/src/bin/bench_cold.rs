//! B0-D: Cold Full Production Execution Benchmark
//!
//! ==================================================================
//! PERFORMANCE HARNESS
//! NOT A NORMAL TEST
//! FULL MODE MAY BE SLOW
//!
//! B0 semantic/runtime baseline: 1ce6b10f1de3cd10fef619483df444f83899c870
//! ==================================================================
//!
//! Measures from dead processes:
//!   HostExecutionService invocation → engine launch/init → provider launch/init/catalogue
//!   → Core planning → policy/replay/dispatch → provider result → cleanup
//!
//! Each sample creates fresh temp isolated roots, provisioned replay, then times
//! a single HostExecutionService::run() that cold-launches engine+provider.

use clap::Parser;
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;
use tethers_reference_host::configured_runtime::prepare_runtime;
use tethers_reference_host::host_execution::{
    ExecutionServiceResult, HostExecutionService, PreparedEvaluationInput,
};
use tethers_reference_host::manifest;
use tethers_reference_host::runtime_config::load_runtime_config;

#[derive(Parser)]
#[command(
    name = "bench_cold",
    about = "B0-D: Cold Full Production Execution Benchmark"
)]
struct Args {
    #[arg(short = 'c', long, default_value = "P1")]
    case: String,
    #[arg(short = 'n', long, default_value_t = 20)]
    iterations: usize,
}

fn tether_ping_n(n: usize) -> String {
    let mut s =
        String::from("tether \"benchmark ping\"\n\nanchor\n    fixture.start\n\nwhen\ndo\n");
    for _ in 0..n {
        s.push_str("    fixture.ping\n        message: anchor.message\n        path: \"projects/bench.txt\"\n");
    }
    s
}
fn tether_pc10(n: usize) -> String {
    let mut s = String::from("tether \"benchmark conditions\"\n\nanchor\n    fixture.start\n\nwhen\n    project.type is \"software\"\n    and task.count greater_than 0\ndo\n");
    for _ in 0..n {
        s.push_str("    fixture.ping\n        message: anchor.message\n        path: \"projects/bench.txt\"\n");
    }
    s
}
fn tether_not_matched() -> String {
    String::from("tether \"benchmark ping\"\n\nanchor\n    fixture.wrong_anchor\n\nwhen\ndo\n    fixture.ping\n        message: anchor.message\n        path: \"projects/bench.txt\"\n")
}
fn build_runtime_config(
    fixture_script_path: &str,
    manifest_path: &str,
    pinned_digest: &str,
    case: &str,
) -> serde_json::Value {
    let input_facts = if case == "PC10" {
        json!([{"source_name": "project.type","fact_id": "fact.project_type","host_snapshot_key": "project.type","scalar_type": "string","schema_description": "project type"},{"source_name": "task.count","fact_id": "fact.task_count","host_snapshot_key": "task.count","scalar_type": "integer","schema_description": "task count"}])
    } else {
        json!([])
    };
    json!({
        "format_version": "0.1",
        "tether_set": {"id": "benchmark.b0c","version": "1","tethers": [{"id": "benchmark-ping","version": "1","source_path": "tethers/benchmark.tether","core_environment": {"program_id": "program.benchmark","core_version": "1","capabilities": [{"source_name": "fixture.ping","capability_id": "cap.benchmark.ping","contract_digest": "BENCH-CONTRACT-0","runtime_name": "fixture.ping"}],"input_facts": input_facts}}],"capability_requirements": [{"name": "fixture.ping","version": 1,"reason": "B0-C benchmark"}]},
        "providers": [{"id": "tethers-stdio-fixture","display_name": "Tethers Stdio Fixture","transport": {"kind": "stdio","command": "pwsh.exe","args": ["-NoProfile","-File",fixture_script_path,"-Mode","run-success"],"protocol_version": "2025-11-25"},"capabilities": [{"name": "fixture.ping","version": 1,"manifest_path": manifest_path,"pinned_digest": pinned_digest,"scope_binding": {"kind": "path_prefix","argument_json_pointer": "/path"}}]}],
        "policy": {"default": "deny","rules": [{"name": "fixture.ping","version": 1,"decision": "allow"}]}
    })
}
fn build_input(
    eval_id: &str,
    event_name: &str,
    event_data: serde_json::Value,
    facts: serde_json::Value,
) -> PreparedEvaluationInput {
    PreparedEvaluationInput {
        tether_id: "benchmark-ping".to_string(),
        tether_version: "1".to_string(),
        evaluation_id: eval_id.to_string(),
        anchor_event: json!({"id": format!("evt_{eval_id}"),"name": event_name,"data": event_data}),
        facts,
    }
}
fn make_input(eval_id: &str, case: &str) -> PreparedEvaluationInput {
    match case {
        "P0" => build_input(
            eval_id,
            "fixture.start",
            json!({"message":"hello"}),
            json!({}),
        ),
        "PC10" => build_input(
            eval_id,
            "fixture.start",
            json!({"message":"hello"}),
            json!({"project.type":"software","task.count":5}),
        ),
        _ => build_input(
            eval_id,
            "fixture.start",
            json!({"message":"hello"}),
            json!({}),
        ),
    }
}
fn expected_status(case: &str) -> &'static str {
    match case {
        "P0" => "not_matched",
        _ => "matched",
    }
}

fn compute_stats(times_us: &[f64]) -> serde_json::Value {
    let mut sorted = times_us.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = sorted.len();
    let median = if n % 2 == 1 {
        sorted[n / 2]
    } else {
        (sorted[n / 2 - 1] + sorted[n / 2]) / 2.0
    };
    let p95_idx = ((n as f64) * 0.95).floor() as usize;
    let p95 = sorted[p95_idx.min(n - 1)];
    let min = sorted[0];
    let max = sorted[n - 1];
    let mean: f64 = sorted.iter().sum::<f64>() / n as f64;
    let variance: f64 = sorted.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n as f64;
    let stddev = variance.sqrt();
    let ops = if mean > 0.0 { 1_000_000.0 / mean } else { 0.0 };
    json!({"sample_count": n,"median_us": median,"p95_us": p95,"min_us": min,"max_us": max,"mean_us": mean,"stddev_us": stddev,"ops_per_sec": ops})
}
fn engine_binary_path() -> Option<PathBuf> {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop();
    p.push("engine-ocaml");
    p.push("_build");
    p.push("default");
    p.push("bin");
    p.push("tethers_mcp_main.exe");
    if p.exists() {
        Some(p)
    } else {
        None
    }
}
fn provision_replay_root(root: &PathBuf) {
    use std::process::Command;
    fs::create_dir_all(root).expect("create host-data");
    let acl_script = format!("$p='{}'; $identity=[System.Security.Principal.WindowsIdentity]::GetCurrent().Name; $acl=[System.Security.AccessControl.DirectorySecurity]::new(); $acl.SetAccessRuleProtection($true,$false); $inherit=[System.Security.AccessControl.InheritanceFlags]::ContainerInherit -bor [System.Security.AccessControl.InheritanceFlags]::ObjectInherit; foreach($t in @($identity,'NT AUTHORITY\\SYSTEM','BUILTIN\\Administrators')) {{ $acl.AddAccessRule([System.Security.AccessControl.FileSystemAccessRule]::new($t,'FullControl',$inherit,'None','Allow')) }}; Set-Acl -LiteralPath $p -AclObject $acl", root.to_string_lossy());
    let status = Command::new("pwsh.exe")
        .args(["-NoProfile", "-Command", &acl_script])
        .status()
        .expect("set ACL");
    assert!(status.success(), "ACL");
    let outcome =
        tethers_reference_host::replay_windows::provision_replay(root).expect("provision_replay");
    // quiet
    let _ = outcome;
}

fn main() {
    let args = Args::parse();
    let case = &args.case;
    let expected = expected_status(case);
    eprintln!("B0-D: Cold Full Production Execution Benchmark");
    eprintln!("Case: {case}, cold_iterations: {}", args.iterations);
    let engine_path = engine_binary_path().expect("engine binary not found");
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir.parent().unwrap();
    let src_manifest =
        repo_root.join("protocol/capability-manifests/fixture-ping-standing-allow.json");
    let manifest_json = fs::read_to_string(&src_manifest).expect("read manifest");
    let (_b, computed_digest) = manifest::canonicalize_and_digest(&manifest_json).expect("digest");
    let expected_digest = "sha256:eb61b62bde489e00a4d15c37c83e6cdb1e9e378b8f13b910d4b68bd6d68c19da";
    assert_eq!(
        computed_digest, expected_digest,
        "digest must match accepted base"
    );
    eprintln!("Pinned digest: {}", computed_digest);
    let fixture_script = repo_root
        .join("scripts/tethers-stdio-fixture.ps1")
        .to_string_lossy()
        .into_owned();
    let tether_source = match case.as_str() {
        "P0" => tether_not_matched(),
        "PC10" => tether_pc10(10),
        _ => tether_ping_n(match case.as_str() {
            "P3" => 3,
            "P10" => 10,
            "P25" => 25,
            "P50" => 50,
            _ => 1,
        }),
    };
    let mut times_us: Vec<f64> = Vec::with_capacity(args.iterations);
    let mut all_correct = true;
    for i in 0..args.iterations {
        let eval_id = format!("bench_cold_{case}_{i:04}");
        let temp_dir = std::env::temp_dir().join(format!(
            "tethers-bench-b0d-{}-{}",
            case,
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&temp_dir).expect("create temp");
        let host_data_root = temp_dir.join("host-data");
        provision_replay_root(&host_data_root);
        let tethers_dir = temp_dir.join("tethers");
        let manifests_dir = temp_dir.join("manifests");
        fs::create_dir_all(&tethers_dir).expect("tethers");
        fs::create_dir_all(&manifests_dir).expect("manifests");
        fs::write(manifests_dir.join("fixture-ping.json"), &manifest_json).expect("manifest");
        fs::write(tethers_dir.join("benchmark.tether"), &tether_source).expect("tether");
        let config = build_runtime_config(
            &fixture_script,
            "manifests/fixture-ping.json",
            &computed_digest,
            case,
        );
        let config_path = temp_dir.join("tethers-config.json");
        fs::write(&config_path, serde_json::to_string_pretty(&config).unwrap()).expect("config");
        let loaded = load_runtime_config(&config_path).expect("load");
        let prepared = prepare_runtime(&loaded).expect("prepare");
        let trail_path = temp_dir.join("trail.jsonl");
        let service =
            HostExecutionService::new(&prepared, &engine_path, &trail_path, Some(&host_data_root));
        let input = make_input(&eval_id, case);
        let t0 = Instant::now();
        let results = service.run(&[input]).expect("run");
        let elapsed = t0.elapsed().as_micros() as f64;
        times_us.push(elapsed);
        // Verify single result
        if results.len() != 1 {
            eprintln!("WRONG {}: expected 1 result", eval_id);
            all_correct = false;
        } else {
            match &results[0] {
                ExecutionServiceResult::Completed { .. } => {
                    if expected == "not_matched" {
                        eprintln!("WRONG {}: expected not_matched got completed", eval_id);
                        all_correct = false;
                    }
                }
                ExecutionServiceResult::NoActions { .. } => {
                    if expected != "not_matched" {
                        eprintln!("WRONG {}: expected matched got no_actions", eval_id);
                        all_correct = false;
                    }
                }
                other => {
                    eprintln!("ERROR {}: {:?}", eval_id, other);
                    all_correct = false;
                }
            }
        }
        let _ = fs::remove_dir_all(&temp_dir);
        eprint!(".");
    }
    eprintln!("\nDone {} samples.", times_us.len());
    let stats = compute_stats(&times_us);
    let output = json!({"benchmark":"B0-D","description":"Cold Full Production Execution Benchmark","case":case,"correctness":all_correct,"stats":stats,"raw_us": times_us});
    println!("{}", serde_json::to_string_pretty(&output).unwrap());
    if !all_correct {
        eprintln!("CORRECTNESS FAILURE");
        std::process::exit(1);
    }
}
