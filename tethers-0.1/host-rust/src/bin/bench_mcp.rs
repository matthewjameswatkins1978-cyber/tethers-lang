//! B0-B: Warm MCP Planning Benchmark
//!
//! ==================================================================
//! PERFORMANCE HARNESS
//! NOT A NORMAL TEST
//! FULL MODE MAY BE SLOW
//!
//! B0 semantic/runtime baseline: 1ce6b10f1de3cd10fef619483df444f83899c870
//! ==================================================================
//!
//! Measures the real normal client boundary:
//!   retained Rust EngineSession → real OCaml MCP → tethers.evaluate → response
//!
//! Launches EngineSession ONCE, warms it, then performs repeated
//! evaluate_tether calls through the same retained process.
//! Fresh evaluation_id/event_id per sample.

use clap::Parser;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::time::Instant;
use tethers_reference_host::engine_stdio::{EngineSession, PlannerResponseWire};

#[derive(Parser)]
#[command(name = "bench_mcp", about = "B0-B: Warm MCP Planning Benchmark")]
struct Args {
    /// Benchmark case: P0, P1, P3, P10, P25, P50, PC10, PA10
    #[arg(short = 'c', long, default_value = "P1")]
    case: String,

    /// Number of measured iterations
    #[arg(short = 'n', long, default_value_t = 500)]
    iterations: usize,

    /// Number of warmup iterations (not timed)
    #[arg(short = 'w', long, default_value_t = 100)]
    warmup: usize,

    /// Batch size for timing (divide batch time by this)
    #[arg(short = 'b', long, default_value_t = 10)]
    batch_size: usize,
}

// ── Tether sources ──────────────────────────────────────────────────

const TETHER_PING_1: &str = "tether \"benchmark ping\"\n\nanchor\n    fixture.start\n\nwhen\ndo\n    fixture.ping\n        message: anchor.message\n";

fn tether_ping_n(n: usize) -> String {
    let mut s =
        String::from("tether \"benchmark ping\"\n\nanchor\n    fixture.start\n\nwhen\ndo\n");
    for _ in 0..n {
        s.push_str("    fixture.ping\n        message: anchor.message\n");
    }
    s
}

const TETHER_NOT_MATCHED: &str = "tether \"benchmark ping\"\n\nanchor\n    fixture.wrong_anchor\n\nwhen\ndo\n    fixture.ping\n        message: anchor.message\n";

fn tether_pc10(n: usize) -> String {
    let mut s = String::from(
        "tether \"benchmark conditions\"\n\nanchor\n    fixture.start\n\nwhen\n    project.type is \"software\"\n    and task.count greater_than 0\ndo\n",
    );
    for _ in 0..n {
        s.push_str("    fixture.ping\n        message: anchor.message\n");
    }
    s
}

// ── Request builders ────────────────────────────────────────────────

fn build_request(
    evaluation_id: &str,
    tether_source: &str,
    event_name: &str,
    facts: Value,
) -> Value {
    json!({
        "protocol_version": "0.1",
        "language_version": "0.1",
        "evaluation_id": evaluation_id,
        "tether": {
            "id": "benchmark-mcp",
            "version": "1",
            "source": tether_source
        },
        "event": {
            "id": format!("evt_{evaluation_id}"),
            "name": event_name,
            "data": { "message": "hello" }
        },
        "facts": facts,
        "capabilities": [{
            "name": "fixture.ping",
            "version": "1.0.0",
            "inputs": {"message": "string"},
            "effects": ["fixture.test"],
            "reversibility": "compensatable"
        }],
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
    })
}

fn build_pc10_request(evaluation_id: &str, tether_source: &str) -> Value {
    json!({
        "protocol_version": "0.1",
        "language_version": "0.1",
        "evaluation_id": evaluation_id,
        "tether": {
            "id": "benchmark-mcp-pc10",
            "version": "1",
            "source": tether_source
        },
        "event": {
            "id": format!("evt_{evaluation_id}"),
            "name": "fixture.start",
            "data": { "message": "hello" }
        },
        "facts": {
            "project.type": "software",
            "task.count": 5
        },
        "capabilities": [{
            "name": "fixture.ping",
            "version": "1.0.0",
            "inputs": {"message": "string"},
            "effects": ["fixture.test"],
            "reversibility": "compensatable"
        }],
        "core_environment": {
            "program_id": "program.benchmark",
            "core_version": "1",
            "capabilities": [{
                "source_name": "fixture.ping",
                "capability_id": "cap.benchmark.ping",
                "contract_digest": "BENCH-CONTRACT-0",
                "runtime_name": "fixture.ping"
            }],
            "input_facts": [
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
            ]
        }
    })
}

// ── Case dispatch ───────────────────────────────────────────────────

fn make_request(evaluation_id: &str, case: &str) -> Value {
    match case {
        "P0" => build_request(
            evaluation_id,
            TETHER_NOT_MATCHED,
            "fixture.start",
            json!({}),
        ),
        "P1" => build_request(evaluation_id, TETHER_PING_1, "fixture.start", json!({})),
        "P3" => build_request(evaluation_id, &tether_ping_n(3), "fixture.start", json!({})),
        "P10" => build_request(
            evaluation_id,
            &tether_ping_n(10),
            "fixture.start",
            json!({}),
        ),
        "P25" => build_request(
            evaluation_id,
            &tether_ping_n(25),
            "fixture.start",
            json!({}),
        ),
        "P50" => build_request(
            evaluation_id,
            &tether_ping_n(50),
            "fixture.start",
            json!({}),
        ),
        "PC10" => build_pc10_request(evaluation_id, &tether_pc10(10)),
        "PA10" => build_request(
            evaluation_id,
            &tether_ping_n(10),
            "fixture.start",
            json!({}),
        ),
        _ => panic!("unknown case: {case}"),
    }
}

fn expected_status(case: &str) -> &'static str {
    match case {
        "P0" => "not_matched",
        _ => "matched",
    }
}

// ── Engine binary path (same logic as tests) ────────────────────────

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

// ── Statistics ──────────────────────────────────────────────────────

fn compute_stats(times_us: &[f64]) -> Value {
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

// ── Main ────────────────────────────────────────────────────────────

fn main() {
    let args = Args::parse();
    let case = &args.case;
    let expected = expected_status(case);

    eprintln!("B0-B: Warm MCP Planning Benchmark");
    eprintln!(
        "Case: {case}, iterations: {}, warmup: {}, batch: {}",
        args.iterations, args.warmup, args.batch_size
    );

    // Find engine binary
    let engine_path = engine_binary_path().expect(
        "engine binary not found at engine-ocaml/_build/default/bin/tethers_mcp_main.exe; \
         build with: opam exec -- dune build --profile release @all",
    );
    let working_dir = engine_path.parent().unwrap().to_path_buf();

    // Launch ONCE
    eprintln!("Launching MCP engine...");
    let mut session =
        EngineSession::launch(&engine_path, &working_dir).expect("failed to launch EngineSession");
    eprintln!("Engine launched and initialized.");

    // Warmup
    eprintln!("Warming up ({})...", args.warmup);
    for i in 0..args.warmup {
        let eval_id = format!("warmup_{i}");
        let req = make_request(&eval_id, case);
        let _ = session.evaluate_tether(&eval_id, &req);
    }
    eprintln!("Warmup complete.");

    // Pre-build all request envelopes (outside timed region)
    let requests: Vec<(String, Value)> = (0..args.iterations)
        .map(|i| {
            let eval_id = format!("bench_{case}_{i:06}");
            let req = make_request(&eval_id, case);
            (eval_id, req)
        })
        .collect();

    // Measure in batches
    let num_batches = (args.iterations + args.batch_size - 1) / args.batch_size;
    let mut batch_times_us: Vec<f64> = Vec::with_capacity(num_batches);
    let mut all_correct = true;
    let mut first_program_digest: Option<String> = None;

    eprintln!("Measuring {num_batches} batches of {}...", args.batch_size);
    for batch_idx in 0..num_batches {
        let start = batch_idx * args.batch_size;
        let end = (start + args.batch_size).min(args.iterations);
        let t0 = Instant::now();
        for i in start..end {
            let (eval_id, req) = &requests[i];
            let wire = session
                .evaluate_tether(eval_id, req)
                .expect("evaluate_tether failed");
            match wire {
                PlannerResponseWire::Matched(resp) => {
                    let status = resp
                        .get("status")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");
                    if status != expected {
                        eprintln!("WRONG STATUS at {eval_id}: expected {expected}, got {status}");
                        all_correct = false;
                    }
                    if first_program_digest.is_none() {
                        if let Some(pd) = resp.get("program_digest").and_then(|v| v.as_str()) {
                            first_program_digest = Some(pd.to_string());
                        }
                    }
                }
                PlannerResponseWire::NotMatched(_) => {
                    if expected != "not_matched" {
                        eprintln!(
                            "WRONG STATUS at {eval_id}: expected {expected}, got not_matched"
                        );
                        all_correct = false;
                    }
                }
                PlannerResponseWire::Error(v) => {
                    eprintln!("ERROR at {eval_id}: {v}");
                    all_correct = false;
                }
                PlannerResponseWire::Unknown { status, .. } => {
                    eprintln!("UNKNOWN status at {eval_id}: {status}");
                    all_correct = false;
                }
            }
        }
        let elapsed_us = t0.elapsed().as_micros() as f64;
        batch_times_us.push(elapsed_us / args.batch_size as f64);
    }

    // Verify ProgramDigest stability: re-evaluate first case and compare
    eprintln!("Verifying ProgramDigest stability...");
    let verify_req = make_request("verify_pd_001", case);
    if let PlannerResponseWire::Matched(resp) = session
        .evaluate_tether("verify_pd_001", &verify_req)
        .expect("verify")
    {
        let pd = resp
            .get("program_digest")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if let Some(ref first) = first_program_digest {
            if pd == first {
                eprintln!("ProgramDigest stability: PASS ({pd})");
            } else {
                eprintln!("ProgramDigest stability: FAIL (expected {first}, got {pd})");
                all_correct = false;
            }
        }
    }

    session.shutdown();

    let stats = compute_stats(&batch_times_us);

    // Output JSON to stdout with raw batch times for recomputation
    let output = json!({
        "benchmark": "B0-B",
        "description": "Warm MCP Planning Benchmark",
        "case": case,
        "correctness": all_correct,
        "stats": stats,
        "raw_us": batch_times_us,
    });
    println!("{}", serde_json::to_string_pretty(&output).unwrap());

    if !all_correct {
        eprintln!("CORRECTNESS FAILURE");
        std::process::exit(1);
    }
    eprintln!("Done.");
}
