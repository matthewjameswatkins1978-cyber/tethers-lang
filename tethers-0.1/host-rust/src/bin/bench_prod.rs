//! B0-C: Warm Full Production Execution Benchmark
//!
//! ==================================================================
//! PERFORMANCE HARNESS
//! NOT A NORMAL TEST
//! FULL MODE MAY BE SLOW
//!
//! B0 semantic/runtime baseline: 1ce6b10f1de3cd10fef619483df444f83899c870
//! ==================================================================
//!
//! Measures the real public production route:
//!   runtime config → prepare_runtime → HostExecutionService → retained engine
//!   → canonical Core → policy → replay → dispatch → fixture provider → Trail
//!
//! Uses fixture.ping with the existing deterministic successful fixture setup.
//! Launches engine + provider ONCE, warms, then measures repeated evaluations.

use clap::Parser;
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;
use tethers_reference_host::approval::ApprovalStore;
use tethers_reference_host::configured_runtime::prepare_runtime;
use tethers_reference_host::host_execution::{
    ExecutionServiceResult, HostExecutionService, PreparedEvaluationInput,
};
use tethers_reference_host::manifest;
use tethers_reference_host::runtime_config::load_runtime_config;

#[derive(Parser)]
#[command(
    name = "bench_prod",
    about = "B0-C: Warm Full Production Execution Benchmark"
)]
struct Args {
    /// Benchmark case: P0, P1, P3, P10, P25, P50, PC10, PA10
    #[arg(short = 'c', long, default_value = "P1")]
    case: String,

    /// Number of measured iterations
    #[arg(short = 'n', long, default_value_t = 200)]
    iterations: usize,

    /// Number of warmup iterations (not timed)
    #[arg(short = 'w', long, default_value_t = 20)]
    warmup: usize,

    /// Batch size for timing
    #[arg(short = 'b', long, default_value_t = 10)]
    batch_size: usize,

    /// Marker file for provider tools/call observation proof
    #[arg(long, default_value = "")]
    marker: String,
}

// ── Tether sources ──────────────────────────────────────────────────

fn tether_ping_n(n: usize) -> String {
    let mut s =
        String::from("tether \"benchmark ping\"\n\nanchor\n    fixture.start\n\nwhen\ndo\n");
    for _ in 0..n {
        s.push_str(
            "    fixture.ping\n        message: anchor.message\n        path: \"projects/bench.txt\"\n",
        );
    }
    s
}

fn tether_pc10(n: usize) -> String {
    let mut s = String::from(
        "tether \"benchmark conditions\"\n\nanchor\n    fixture.start\n\nwhen\n    project.type is \"software\"\n    and task.count greater_than 0\ndo\n",
    );
    for _ in 0..n {
        s.push_str(
            "    fixture.ping\n        message: anchor.message\n        path: \"projects/bench.txt\"\n",
        );
    }
    s
}

fn tether_not_matched() -> String {
    String::from(
        "tether \"benchmark ping\"\n\nanchor\n    fixture.wrong_anchor\n\nwhen\ndo\n    fixture.ping\n        message: anchor.message\n        path: \"projects/bench.txt\"\n",
    )
}

// ── Runtime config builder ──────────────────────────────────────────

fn build_runtime_config(
    fixture_script_path: &str,
    manifest_path: &str,
    pinned_digest: &str,
    case: &str,
    marker: &str,
) -> serde_json::Value {
    let input_facts = if case == "PC10" {
        json!([
            {
                "source_name": "project.type",
                "fact_id": "fact.project_type",
                "host_snapshot_key": "project.type",
                "scalar_type": "string",
                "schema_description": "project type"
            },
            {
                "source_name": "task.count",
                "fact_id": "fact.task_count",
                "host_snapshot_key": "task.count",
                "scalar_type": "integer",
                "schema_description": "task count"
            }
        ])
    } else {
        json!([])
    };
    json!({
        "format_version": "0.1",
        "tether_set": {
            "id": "benchmark.b0c",
            "version": "1",
            "tethers": [{
                "id": "benchmark-ping",
                "version": "1",
                "source_path": "tethers/benchmark.tether",
                "core_environment": {
                    "program_id": "program.benchmark",
                    "core_version": "1",
                    "capabilities": [{
                        "source_name": "fixture.ping",
                        "capability_id": "cap.benchmark.ping",
                        "contract_digest": "BENCH-CONTRACT-0",
                        "runtime_name": "fixture.ping"
                    }],
                    "input_facts": input_facts
                }
            }],
            "capability_requirements": [{
                "name": "fixture.ping",
                "version": 1,
                "reason": "B0-C benchmark"
            }]
        },
        "providers": [{
            "id": "tethers-stdio-fixture",
            "display_name": "Tethers Stdio Fixture",
            "transport": {
                "kind": "stdio",
                "command": "pwsh.exe",
                "args": ["-NoProfile", "-File", fixture_script_path, "-Mode", "run-success", "-MarkerFile", marker],
                "protocol_version": "2025-11-25"
            },
            "capabilities": [{
                "name": "fixture.ping",
                "version": 1,
                "manifest_path": manifest_path,
                "pinned_digest": pinned_digest,
                "scope_binding": {
                    "kind": "path_prefix",
                    "argument_json_pointer": "/path"
                }
            }]
        }],
        "policy": {
            "default": "deny",
            "rules": [{
                "name": "fixture.ping",
                "version": 1,
                "decision": "allow"
            }]
        }
    })
}

// ── Evaluation input builder ────────────────────────────────────────

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
        anchor_event: json!({
            "id": format!("evt_{eval_id}"),
            "name": event_name,
            "data": event_data
        }),
        facts,
    }
}

fn make_input(eval_id: &str, case: &str) -> PreparedEvaluationInput {
    match case {
        "P0" => build_input(
            eval_id,
            "fixture.start",
            json!({"message": "hello"}),
            json!({}),
        ),
        "PC10" => build_input(
            eval_id,
            "fixture.start",
            json!({"message": "hello"}),
            json!({"project.type": "software", "task.count": 5}),
        ),
        _ => build_input(
            eval_id,
            "fixture.start",
            json!({"message": "hello"}),
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

fn actions_per_evaluation(case: &str) -> usize {
    match case {
        "P0" => 0,
        "P3" => 3,
        "P10" | "PC10" | "PA10" => 10,
        "P25" => 25,
        "P50" => 50,
        _ => 1,
    }
}

// ── Statistics ──────────────────────────────────────────────────────

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
    let ops_per_sec = if mean > 0.0 { 1_000_000.0 / mean } else { 0.0 };

    json!({
        "sample_count": n,
        "median_us": median,
        "p95_us": p95,
        "min_us": min,
        "max_us": max,
        "mean_us": mean,
        "stddev_us": stddev,
        "ops_per_sec": ops_per_sec
    })
}

// ── Engine binary path ──────────────────────────────────────────────

fn engine_binary_path() -> Option<PathBuf> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.push("engine-ocaml");
    path.push("_build");
    path.push("default");
    path.push("bin");
    path.push("tethers_mcp_main.exe");
    if path.exists() {
        Some(path)
    } else {
        None
    }
}

// ── Main ────────────────────────────────────────────────────────────

fn main() {
    let args = Args::parse();
    let case = &args.case;
    let expected = expected_status(case);

    eprintln!("B0-C: Warm Full Production Execution Benchmark");
    eprintln!(
        "Case: {case}, iterations: {}, warmup: {}, batch: {}",
        args.iterations, args.warmup, args.batch_size
    );

    // Find paths
    let engine_path = engine_binary_path()
        .expect("engine binary not found at engine-ocaml/_build/default/bin/tethers_mcp_main.exe");
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir.parent().unwrap();

    // Create temp directory with runtime config under system temp (validated replay root)
    let temp_dir = std::env::temp_dir().join(format!("tethers-bench-b0c-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&temp_dir).expect("create temp dir");
    // Provision replay root as temp_dir/host-data with protected ACL (CORE-9C pattern)
    let host_data_root = temp_dir.join("host-data");
    {
        use std::process::Command;
        fs::create_dir_all(&host_data_root).expect("create host-data root");
        let acl_script = format!(
            "$p='{}'; $identity=[System.Security.Principal.WindowsIdentity]::GetCurrent().Name; $acl=[System.Security.AccessControl.DirectorySecurity]::new(); $acl.SetAccessRuleProtection($true,$false); $inherit=[System.Security.AccessControl.InheritanceFlags]::ContainerInherit -bor [System.Security.AccessControl.InheritanceFlags]::ObjectInherit; foreach($t in @($identity,'NT AUTHORITY\\SYSTEM','BUILTIN\\Administrators')) {{ $acl.AddAccessRule([System.Security.AccessControl.FileSystemAccessRule]::new($t,'FullControl',$inherit,'None','Allow')) }}; Set-Acl -LiteralPath $p -AclObject $acl",
            host_data_root.to_string_lossy()
        );
        let status = Command::new("pwsh.exe")
            .args(["-NoProfile", "-Command", &acl_script])
            .status()
            .expect("set host-data ACL");
        assert!(
            status.success(),
            "host-data root must receive protected ACL"
        );
        let outcome = tethers_reference_host::replay_windows::provision_replay(&host_data_root)
            .expect("provision_replay");
        eprintln!("Replay provisioned: {:?}", outcome);
    }
    let tethers_dir = temp_dir.join("tethers");
    let manifests_dir = temp_dir.join("manifests");
    fs::create_dir_all(&tethers_dir).expect("create temp dirs");
    fs::create_dir_all(&manifests_dir).expect("create manifests dir");

    // Copy standing-allow manifest into temp dir as manifests/fixture-ping.json
    // and compute pinned_digest from exact manifest using canonicalize_and_digest
    let src_manifest =
        repo_root.join("protocol/capability-manifests/fixture-ping-standing-allow.json");
    let manifest_json = fs::read_to_string(&src_manifest).expect("read standing-allow manifest");
    let (_canonical_bytes, computed_digest) =
        manifest::canonicalize_and_digest(&manifest_json).expect("canonicalize_and_digest");
    // Assert digest matches accepted base, but computed value is authority
    let expected_digest = "sha256:eb61b62bde489e00a4d15c37c83e6cdb1e9e378b8f13b910d4b68bd6d68c19da";
    if computed_digest != expected_digest {
        eprintln!(
            "WARNING: computed digest {} != expected {}",
            computed_digest, expected_digest
        );
    } else {
        eprintln!("Pinned digest verified: {}", computed_digest);
    }
    fs::write(manifests_dir.join("fixture-ping.json"), &manifest_json).expect("copy manifest");

    // Write tether source
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
    fs::write(tethers_dir.join("benchmark.tether"), &tether_source).expect("write tether source");

    // Write runtime config with computed pinned_digest and scope_binding
    let fixture_script = repo_root.join("scripts/tethers-stdio-fixture.ps1");
    let marker_file = temp_dir.join("provider-marker.txt");
    let marker_str = marker_file.to_string_lossy().into_owned();
    let config = build_runtime_config(
        &fixture_script.to_string_lossy(),
        "manifests/fixture-ping.json",
        &computed_digest,
        case,
        &marker_str,
    );
    let config_path = temp_dir.join("tethers-config.json");
    fs::write(&config_path, serde_json::to_string_pretty(&config).unwrap()).expect("write config");

    // Prepare runtime
    eprintln!("Preparing runtime...");
    let loaded = load_runtime_config(&config_path).expect("load_runtime_config");
    let prepared = prepare_runtime(&loaded).expect("prepare_runtime");
    eprintln!(
        "Runtime prepared: {} tethers, {} providers",
        prepared.tethers().len(),
        prepared.providers().len()
    );

    // Create service with isolated replay/Trail roots (replay = host-data)
    let trail_path = temp_dir.join("trail.jsonl");
    let service =
        HostExecutionService::new(&prepared, &engine_path, &trail_path, Some(&host_data_root));

    // Warm up: launch engine + provider
    eprintln!("Warming up (launching engine + provider)...");
    let (mut engine, mut provider_sessions, provider_availability) =
        service.bench_warmup().expect("bench_warmup");
    eprintln!("Engine + provider warmed.");

    // Warm evaluation path
    eprintln!("Warming evaluation path ({})...", args.warmup);
    let mut approvals = ApprovalStore::default();
    for i in 0..args.warmup {
        let input = make_input(&format!("warmup_{i}"), case);
        let _ = service.bench_evaluate_one(
            &input,
            &mut engine,
            &mut provider_sessions,
            &provider_availability,
            &mut approvals,
        );
    }
    eprintln!("Evaluation warmup complete.");

    // Pre-build all evaluation inputs (outside timed region)
    let inputs: Vec<PreparedEvaluationInput> = (0..args.iterations)
        .map(|i| make_input(&format!("bench_{case}_{i:06}"), case))
        .collect();

    // Measure in batches
    let num_batches = (args.iterations + args.batch_size - 1) / args.batch_size;
    let mut batch_times_us: Vec<f64> = Vec::with_capacity(num_batches);
    let mut all_correct = true;

    eprintln!("Measuring {num_batches} batches of {}...", args.batch_size);
    for batch_idx in 0..num_batches {
        let start = batch_idx * args.batch_size;
        let end = (start + args.batch_size).min(args.iterations);
        let t0 = Instant::now();
        for i in start..end {
            let result = service.bench_evaluate_one(
                &inputs[i],
                &mut engine,
                &mut provider_sessions,
                &provider_availability,
                &mut approvals,
            );
            match &result {
                ExecutionServiceResult::Completed {
                    evaluation_id,
                    action_id,
                    response,
                    ..
                } => {
                    if expected == "not_matched" {
                        eprintln!(
                            "WRONG at {}: expected not_matched, got completed",
                            inputs[i].evaluation_id
                        );
                        all_correct = false;
                    } else {
                        // Verify bridge pins are present in the plan
                        let plan = response.get("plan");
                        let has_manifest_digest = response
                            .get("plan")
                            .and_then(|p| p.get("actions"))
                            .and_then(|a| a.as_array())
                            .and_then(|arr| arr.first())
                            .and_then(|act| act.get("manifest_digest"))
                            .is_some();
                        if !has_manifest_digest {
                            // Try alternate location: action may be at top level?
                            eprintln!(
                                "WARNING: Completed {} missing manifest_digest in plan: {:?}",
                                evaluation_id, plan
                            );
                        }
                        let _ = (evaluation_id, action_id);
                    }
                }
                ExecutionServiceResult::NoActions { .. } => {
                    if expected != "not_matched" {
                        eprintln!(
                            "WRONG at {}: expected matched, got no_actions",
                            inputs[i].evaluation_id
                        );
                        all_correct = false;
                    }
                }
                ExecutionServiceResult::PlannerError { message, .. } => {
                    eprintln!("ERROR at {}: {message}", inputs[i].evaluation_id);
                    all_correct = false;
                }
                ExecutionServiceResult::InvalidData { message } => {
                    eprintln!("INVALID at {}: {message}", inputs[i].evaluation_id);
                    all_correct = false;
                }
                other => {
                    eprintln!("UNEXPECTED at {}: {other:?}", inputs[i].evaluation_id);
                    all_correct = false;
                }
            }
        }
        let elapsed_us = t0.elapsed().as_micros() as f64;
        batch_times_us.push(elapsed_us / args.batch_size as f64);
    }

    // Verify provider tools/call observed (only for matched cases)
    let mut marker_calls: usize = 0;
    if expected != "not_matched" {
        if let Ok(content) = fs::read_to_string(&marker_file) {
            marker_calls = content.lines().filter(|l| *l == "tools/call").count();
        }
        let expected_calls = actions_per_evaluation(case) * (args.warmup + inputs.len());
        eprintln!(
            "Provider tools/call observed: {marker_calls} ({expected_calls} expected for {case})",
        );
        if marker_calls != expected_calls {
            eprintln!("PROVIDER tools/call count mismatch (got {marker_calls}, expected {expected_calls})");
            all_correct = false;
        }
    }

    // Cleanup
    engine.shutdown();
    for (_, mut session) in provider_sessions {
        session.close();
    }

    // Clean up temp dir
    let _ = fs::remove_dir_all(&temp_dir);

    let stats = compute_stats(&batch_times_us);

    let output = json!({
        "benchmark": "B0-C",
        "description": "Warm Full Production Execution Benchmark",
        "case": case,
        "correctness": all_correct,
        "stats": stats,
        "raw_us": batch_times_us,
        "provider_tools_call_observed": marker_calls,
    });
    println!("{}", serde_json::to_string_pretty(&output).unwrap());

    if !all_correct {
        eprintln!("CORRECTNESS FAILURE");
        std::process::exit(1);
    }
    eprintln!("Done.");
}
