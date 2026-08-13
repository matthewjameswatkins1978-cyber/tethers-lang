//! PF1 Part B/D: Retained P10 Growth Benchmark.
//!
//! ==================================================================
//! PERFORMANCE HARNESS
//! NOT A NORMAL TEST
//! FULL MODE MAY BE SLOW
//!
//! B0 semantic/runtime baseline: 1ce6b10f1de3cd10fef619483df444f83899c870
//! ==================================================================
//!
//! Uses ONE retained production session (engine + provider launched once)
//! and records every P10 evaluation individually — no batching, no averaging
//! across evaluations. Each evaluation uses a fresh evaluation ID and is
//! verified to have made EXACTLY 10 provider tools/call invocations; on any
//! mismatch the benchmark stops rather than analysing invalid timing.
//!
//! Per-evaluation state is observed before/after each run:
//!   - wall-clock execution time (single evaluation)
//!   - exact provider tools/call delta and cumulative count
//!   - Trail line count and bytes
//!   - replay/state file count and total bytes (claims, locks, chain files)
//!
//! With the `bench-timing` feature enabled, stage durations recorded by the
//! feature-gated hooks in the production path are captured per evaluation
//! (raw samples for evaluations 1, 3, 6, 12; aggregates for all).

use clap::Parser;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tethers_reference_host::approval::ApprovalStore;
use tethers_reference_host::bench_timing;
use tethers_reference_host::configured_runtime::prepare_runtime;
use tethers_reference_host::host_execution::{
    ExecutionServiceResult, HostExecutionService, PreparedEvaluationInput,
};
use tethers_reference_host::manifest;
use tethers_reference_host::replay_runtime::FileReplayAuthority;
use tethers_reference_host::runtime_config::load_runtime_config;

#[derive(Parser)]
#[command(name = "bench_retained", about = "PF1: Retained P10 growth benchmark")]
struct Args {
    /// Number of measured retained evaluations
    #[arg(short = 'n', long, default_value_t = 12)]
    evaluations: usize,

    /// Number of not_matched warmup evaluations (no dispatch, no replay state)
    #[arg(short = 'w', long, default_value_t = 3)]
    warmup: usize,

    /// Marker file path override (default: under the temp runtime dir)
    #[arg(long, default_value = "")]
    marker: String,
}

// ---------------------------------------------------------------------------
// Tether source: P10 (10 sequential ping actions, identical signatures)
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Runtime config (mirrors B0-C setup)
// ---------------------------------------------------------------------------

fn build_runtime_config(
    fixture_script_path: &str,
    manifest_path: &str,
    pinned_digest: &str,
    marker: &str,
) -> serde_json::Value {
    json!({
        "format_version": "0.1",
        "tether_set": {
            "id": "benchmark.pf1",
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
                    "input_facts": []
                }
            }],
            "capability_requirements": [{
                "name": "fixture.ping",
                "version": 1,
                "reason": "PF1 benchmark"
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

fn build_input(eval_id: &str, event_name: &str) -> PreparedEvaluationInput {
    PreparedEvaluationInput {
        tether_id: "benchmark-ping".to_string(),
        tether_version: "1".to_string(),
        evaluation_id: eval_id.to_string(),
        anchor_event: json!({
            "id": format!("evt_{eval_id}"),
            "name": event_name,
            "data": { "message": "hello" }
        }),
        facts: json!({}),
    }
}

// ---------------------------------------------------------------------------
// Engine / repo path helpers
// ---------------------------------------------------------------------------

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

fn provision_replay_root(root: &Path) {
    use std::process::Command;
    fs::create_dir_all(root).expect("create host-data");
    let acl_script = format!(
        "$p='{}'; $identity=[System.Security.Principal.WindowsIdentity]::GetCurrent().Name; $acl=[System.Security.AccessControl.DirectorySecurity]::new(); $acl.SetAccessRuleProtection($true,$false); $inherit=[System.Security.AccessControl.InheritanceFlags]::ContainerInherit -bor [System.Security.AccessControl.InheritanceFlags]::ObjectInherit; foreach($t in @($identity,'NT AUTHORITY\\SYSTEM','BUILTIN\\Administrators')) {{ $acl.AddAccessRule([System.Security.AccessControl.FileSystemAccessRule]::new($t,'FullControl',$inherit,'None','Allow')) }}; Set-Acl -LiteralPath $p -AclObject $acl",
        root.to_string_lossy()
    );
    let status = Command::new("pwsh.exe")
        .args(["-NoProfile", "-Command", &acl_script])
        .status()
        .expect("set host-data ACL");
    assert!(
        status.success(),
        "host-data root must receive protected ACL"
    );
    let _ =
        tethers_reference_host::replay_windows::provision_replay(root).expect("provision_replay");
}

// ---------------------------------------------------------------------------
// State observation
// ---------------------------------------------------------------------------

#[derive(Default)]
struct ReplayState {
    files: usize,
    bytes: u64,
    claims: usize,
    locks: usize,
    chain_files: usize,
    execution_dirs: usize,
    directories: usize,
}

fn walk_replay(root: &Path) -> ReplayState {
    let mut state = ReplayState::default();
    let mut stack: Vec<PathBuf> = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = match fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
            if is_dir {
                state.directories += 1;
                // Execution directory: chains/<2hex>/<64hex>
                let is_execution_dir = name.len() == 64
                    && name.chars().all(|c| c.is_ascii_hexdigit())
                    && dir.file_name().map(|d| d.len() == 2).unwrap_or(false);
                if is_execution_dir {
                    state.execution_dirs += 1;
                }
                stack.push(path);
            } else if let Ok(meta) = fs::metadata(&path) {
                state.files += 1;
                state.bytes += meta.len();
                if name.ends_with(".claim.json") {
                    state.claims += 1;
                } else if name.ends_with(".lock") {
                    state.locks += 1;
                } else {
                    state.chain_files += 1;
                }
            }
        }
    }
    state
}

fn trail_state(path: &Path) -> (usize, u64) {
    match fs::read_to_string(path) {
        Ok(content) => {
            let lines = content.matches('\n').count();
            (lines, content.len() as u64)
        }
        Err(_) => (0, 0),
    }
}

fn marker_calls(path: &Path) -> usize {
    match fs::read_to_string(path) {
        Ok(content) => content.lines().filter(|l| *l == "tools/call").count(),
        Err(_) => 0,
    }
}

// ---------------------------------------------------------------------------
// Stage timing aggregation
// ---------------------------------------------------------------------------

fn stage_aggregate(samples: &[(&'static str, u128)]) -> Value {
    let mut by_stage: std::collections::BTreeMap<&str, Vec<u128>> =
        std::collections::BTreeMap::new();
    for (stage, us) in samples {
        by_stage.entry(stage).or_default().push(*us);
    }
    let mut obj = serde_json::Map::new();
    for (stage, us) in by_stage {
        let total: u128 = us.iter().sum();
        let count = us.len() as u128;
        obj.insert(
            stage.to_owned(),
            json!({
                "count": count,
                "total_us": total,
                "mean_us": (total as f64) / (count as f64),
            }),
        );
    }
    Value::Object(obj)
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() {
    let args = Args::parse();
    let case = "P10";
    let actions_per_eval = 10;

    eprintln!("PF1: Retained P10 growth benchmark");
    eprintln!(
        "Case: {case}, evaluations: {}, warmup: {}, actions/eval: {actions_per_eval}",
        args.evaluations, args.warmup
    );

    let engine_path = engine_binary_path()
        .expect("engine binary not found at engine-ocaml/_build/default/bin/tethers_mcp_main.exe");
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir.parent().unwrap();

    // Temp runtime root that must survive across all retained evaluations.
    let temp_dir = std::env::temp_dir().join(format!("tethers-bench-pf1-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&temp_dir).expect("create temp dir");
    let host_data_root = temp_dir.join("host-data");
    provision_replay_root(&host_data_root);
    let tethers_dir = temp_dir.join("tethers");
    let manifests_dir = temp_dir.join("manifests");
    fs::create_dir_all(&tethers_dir).expect("create temp dirs");
    fs::create_dir_all(&manifests_dir).expect("create manifests dir");

    let src_manifest =
        repo_root.join("protocol/capability-manifests/fixture-ping-standing-allow.json");
    let manifest_json = fs::read_to_string(&src_manifest).expect("read standing-allow manifest");
    let (_canonical_bytes, computed_digest) =
        manifest::canonicalize_and_digest(&manifest_json).expect("canonicalize_and_digest");
    let expected_digest = "sha256:eb61b62bde489e00a4d15c37c83e6cdb1e9e378b8f13b910d4b68bd6d68c19da";
    assert_eq!(computed_digest, expected_digest, "pinned digest mismatch");
    fs::write(manifests_dir.join("fixture-ping.json"), &manifest_json).expect("copy manifest");

    fs::write(
        tethers_dir.join("benchmark.tether"),
        tether_ping_n(actions_per_eval),
    )
    .expect("write tether source");

    let fixture_script = repo_root.join("scripts/tethers-stdio-fixture.ps1");
    let marker_file = if args.marker.is_empty() {
        temp_dir.join("provider-marker.txt")
    } else {
        PathBuf::from(&args.marker)
    };
    let marker_str = marker_file.to_string_lossy().into_owned();
    let config = build_runtime_config(
        &fixture_script.to_string_lossy(),
        "manifests/fixture-ping.json",
        &computed_digest,
        &marker_str,
    );
    let config_path = temp_dir.join("tethers-config.json");
    fs::write(&config_path, serde_json::to_string_pretty(&config).unwrap()).expect("write config");

    eprintln!("Preparing runtime...");
    let loaded = load_runtime_config(&config_path).expect("load_runtime_config");
    let prepared = prepare_runtime(&loaded).expect("prepare_runtime");

    let trail_path = temp_dir.join("trail.jsonl");
    let service =
        HostExecutionService::new(&prepared, &engine_path, &trail_path, Some(&host_data_root));

    eprintln!("Warming up (launching engine + provider)...");
    let (mut engine, mut provider_sessions, provider_availability) =
        service.bench_warmup().expect("bench_warmup");
    eprintln!("Engine + provider warmed.");

    // Warm the evaluation path with not_matched inputs only: the engine MCP
    // path is exercised but NO Actions dispatch, so the replay ledger stays
    // empty and the growth curve starts from a clean base.
    let mut approvals = ApprovalStore::default();
    let mut replay_authority = FileReplayAuthority::new(Some(&host_data_root));
    for i in 0..args.warmup {
        let input = build_input(&format!("pf1_warmup_{i:02}"), "fixture.wrong_anchor");
        let result = service.bench_evaluate_one(
            &input,
            &mut engine,
            &mut provider_sessions,
            &provider_availability,
            &mut approvals,
            &mut replay_authority,
        );
        match result {
            ExecutionServiceResult::NoActions { .. } => {}
            other => {
                eprintln!("Warmup {i} not not_matched: {other:?}");
                std::process::exit(1);
            }
        }
    }
    eprintln!("Warmup complete (no replay state created).");

    // Verify replay ledger is still empty after not_matched warmup.
    let replay0 = walk_replay(&host_data_root);
    if replay0.claims != 0 {
        eprintln!(
            "Warmup created replay claims unexpectedly: {}",
            replay0.claims
        );
        std::process::exit(1);
    }

    let mut eval_records: Vec<Value> = Vec::with_capacity(args.evaluations);
    let mut total_provider_calls = 0usize;
    let raw_stage_evals = [1usize, 3, 6, 12];

    eprintln!(
        "Measuring {} retained evaluations individually...",
        args.evaluations
    );
    for i in 1..=args.evaluations {
        bench_timing::reset();

        let pre_marker = marker_calls(&marker_file);

        let eval_id = format!("pf1_retained_{i:02}_{}", uuid::Uuid::new_v4().simple());
        let input = build_input(&eval_id, "fixture.start");

        let t0 = Instant::now();
        let result = service.bench_evaluate_one(
            &input,
            &mut engine,
            &mut provider_sessions,
            &provider_availability,
            &mut approvals,
            &mut replay_authority,
        );
        let wall_us = t0.elapsed().as_micros();

        match &result {
            ExecutionServiceResult::Completed {
                evaluation_id,
                action_id,
                ..
            } => {
                let _ = (evaluation_id, action_id);
            }
            other => {
                eprintln!(
                    "STOP: evaluation {i} did not complete: {other:?} (evaluation_id {eval_id})"
                );
                std::process::exit(1);
            }
        }

        let post_trail = trail_state(&trail_path);
        let post_replay = walk_replay(&host_data_root);
        let post_marker = marker_calls(&marker_file);

        let delta_calls = post_marker - pre_marker;
        total_provider_calls += delta_calls;
        if delta_calls != actions_per_eval {
            eprintln!(
                "STOP: evaluation {i} ({eval_id}) made {delta_calls} provider tools/call, expected exactly {actions_per_eval}"
            );
            std::process::exit(1);
        }

        let stage_samples = bench_timing::snapshot();
        let include_raw = raw_stage_evals.contains(&i);
        let mut record = json!({
            "eval_number": i,
            "eval_id": eval_id,
            "wall_us": wall_us,
            "provider_tools_call_delta": delta_calls,
            "provider_tools_call_cumulative": total_provider_calls,
            "trail_lines": post_trail.0,
            "trail_bytes": post_trail.1,
            "replay_files": post_replay.files,
            "replay_bytes": post_replay.bytes,
            "replay_claims": post_replay.claims,
            "replay_locks": post_replay.locks,
            "replay_chain_files": post_replay.chain_files,
            "replay_execution_dirs": post_replay.execution_dirs,
            "replay_directories": post_replay.directories,
            "stages": stage_aggregate(&stage_samples),
        });
        if include_raw {
            let raw_obj: Vec<Value> = stage_samples
                .iter()
                .map(|(s, us)| json!({ "stage": s, "us": us }))
                .collect();
            record["stage_raw"] = Value::Array(raw_obj);
        }
        eval_records.push(record);

        let wall_ms = wall_us as f64 / 1000.0;
        eprintln!(
            "  eval {i:>2}: {wall_ms:>9.1} ms wall, calls+{delta_calls} (cum {total_provider_calls}), trail {} lines / {} B, replay {} files / {} B ({} claims, {} chains)",
            post_trail.0, post_trail.1, post_replay.files, post_replay.bytes, post_replay.claims, post_replay.chain_files
        );
    }

    engine.shutdown();
    for (_, mut session) in provider_sessions {
        session.close();
    }

    let output = json!({
        "benchmark": "PF1-RETAINED",
        "description": "Retained P10 production growth benchmark (single retained session)",
        "case": case,
        "actions_per_eval": actions_per_eval,
        "evaluations": args.evaluations,
        "warmup": args.warmup,
        "env": {
            "package_version": env!("CARGO_PKG_VERSION"),
            "date": unix_timestamp(),
        },
        "provider_call_proof": {
            "expected_per_eval": actions_per_eval,
            "all_exact": true,
            "total": total_provider_calls,
        },
        "trail_path": trail_path.to_string_lossy(),
        "replay_root": host_data_root.to_string_lossy(),
        "eval": eval_records,
    });
    println!("{}", serde_json::to_string_pretty(&output).unwrap());

    let _ = fs::remove_dir_all(&temp_dir);
    eprintln!("Done.");
}

fn unix_timestamp() -> String {
    // Small dependency-free UTC timestamp for the metadata block.
    let t = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}.{:03}", t.as_secs(), t.subsec_millis())
}
