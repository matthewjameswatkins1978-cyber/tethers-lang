use serde::Deserialize;
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use tethers_portable::{
    evaluate_text_with_options, validate_manifest, validate_policy_text, Manifest, Response,
};

fn usage() -> &'static str {
    "usage:\n  tethers evaluate [--input PATH] [--policy PATH] [--manifest PATH] [--explain]\n  tethers explain [--input PATH] [--policy PATH] [--manifest PATH]\n  tethers test POLICY CORPUS [--json]\n  tethers validate-manifest MANIFEST\n  tethers --version"
}

fn main() {
    std::process::exit(run(env::args().skip(1).collect()));
}

fn run(args: Vec<String>) -> i32 {
    if args == ["--version"] || args == ["-V"] {
        println!("tethers portable {}", tethers_portable::version());
        return 0;
    }
    if args == ["--help"] || args == ["-h"] || args.is_empty() {
        println!("{}", usage());
        return if args.is_empty() { 2 } else { 0 };
    }
    match args.first().map(String::as_str) {
        Some("evaluate") => evaluate_command(&args[1..]),
        Some("explain") => {
            let mut explain = args[1..].to_vec();
            explain.push("--explain".to_owned());
            evaluate_command(&explain)
        }
        Some("test") => test_command(&args[1..]),
        Some("validate-manifest") => manifest_command(&args[1..]),
        _ => {
            emit(Response::deny_error_for_cli("unknown command"));
            2
        }
    }
}

fn evaluate_command(args: &[String]) -> i32 {
    let mut input_path = None;
    let mut policy_path = None;
    let mut manifest_path = None;
    let mut explain = false;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--explain" => {
                explain = true;
                index += 1;
            }
            "--input" | "--policy" | "--manifest" => {
                let flag = args[index].clone();
                let Some(path) = args.get(index + 1) else {
                    emit(Response::deny_error_for_cli(format!(
                        "missing value for {flag}"
                    )));
                    return 2;
                };
                let destination = match flag.as_str() {
                    "--input" => &mut input_path,
                    "--policy" => &mut policy_path,
                    _ => &mut manifest_path,
                };
                if destination.is_some() {
                    emit(Response::deny_error_for_cli(format!(
                        "duplicate option: {flag}"
                    )));
                    return 2;
                }
                *destination = Some(PathBuf::from(path));
                index += 2;
            }
            flag => {
                emit(Response::deny_error_for_cli(format!(
                    "unknown option: {flag}"
                )));
                return 2;
            }
        }
    }
    let input = match input_path {
        Some(path) => match fs::read_to_string(path) {
            Ok(value) => value,
            Err(error) => {
                emit(Response::deny_error_for_cli(format!(
                    "cannot read input: {error}"
                )));
                return 3;
            }
        },
        None => {
            let mut value = String::new();
            if let Err(error) = io::stdin().read_to_string(&mut value) {
                emit(Response::deny_error_for_cli(format!(
                    "cannot read stdin: {error}"
                )));
                return 3;
            }
            value
        }
    };
    let policy = match read_optional(policy_path, "policy") {
        Ok(value) => value,
        Err(response) => {
            emit(response);
            return 3;
        }
    };
    let manifest = match read_optional(manifest_path, "manifest") {
        Ok(value) => value,
        Err(response) => {
            emit(response);
            return 3;
        }
    };
    emit(evaluate_text_with_options(
        &input,
        policy.as_deref(),
        manifest.as_deref(),
        explain,
    ));
    0
}

fn read_optional(path: Option<PathBuf>, label: &str) -> Result<Option<String>, Response> {
    path.map(|path| {
        fs::read_to_string(path)
            .map_err(|error| Response::deny_error_for_cli(format!("cannot read {label}: {error}")))
    })
    .transpose()
}

#[derive(Debug, Deserialize)]
struct TestCase {
    name: String,
    request: Value,
    expect: String,
}

fn test_command(args: &[String]) -> i32 {
    let json_output = args.iter().any(|arg| arg == "--json");
    let positional: Vec<_> = args.iter().filter(|arg| *arg != "--json").collect();
    if positional.len() != 2 {
        emit(Response::deny_error_for_cli(
            "usage: tethers test POLICY CORPUS [--json]",
        ));
        return 2;
    }
    let policy = match fs::read_to_string(positional[0]) {
        Ok(value) => value,
        Err(error) => {
            emit(Response::deny_error_for_cli(format!(
                "cannot read policy: {error}"
            )));
            return 3;
        }
    };
    if let Err(error) = validate_policy_text(&policy) {
        emit(Response::deny_error(error));
        return 3;
    }
    let corpus_text = match fs::read_to_string(positional[1]) {
        Ok(value) => value,
        Err(error) => {
            emit(Response::deny_error_for_cli(format!(
                "cannot read corpus: {error}"
            )));
            return 3;
        }
    };
    let cases: Vec<TestCase> =
        match serde_json::from_str::<Value>(&corpus_text)
            .ok()
            .and_then(|value| {
                if value.is_array() {
                    serde_json::from_value::<Vec<TestCase>>(value).ok()
                } else {
                    value
                        .get("cases")
                        .cloned()
                        .and_then(|cases| serde_json::from_value::<Vec<TestCase>>(cases).ok())
                }
            }) {
            Some(cases) if !cases.is_empty() => cases,
            _ => {
                emit(Response::deny_error("invalid test corpus"));
                return 3;
            }
        };
    let mut results = Vec::new();
    let mut all_passed = true;
    for case in cases {
        let request = serde_json::to_string(&case.request).unwrap_or_default();
        let actual = evaluate_text_with_options(&request, Some(&policy), None, true);
        let expected = case.expect.to_ascii_uppercase();
        let passed =
            matches!(expected.as_str(), "ALLOW" | "ASK" | "DENY") && actual.decision == expected;
        all_passed &= passed;
        results.push(json!({"name": case.name, "passed": passed, "expected": expected, "actual": actual.decision, "matched_rule": actual.matched_rule, "error": actual.error}));
    }
    if json_output {
        println!(
            "{}",
            serde_json::to_string(&json!({"passed": all_passed, "cases": results})).unwrap()
        );
    } else {
        for result in &results {
            println!(
                "{}: {} (expected {}, actual {}, rule {})",
                result["name"].as_str().unwrap_or("case"),
                if result["passed"].as_bool().unwrap_or(false) {
                    "PASS"
                } else {
                    "FAIL"
                },
                result["expected"],
                result["actual"],
                result["matched_rule"].as_str().unwrap_or("<none>")
            );
        }
        println!(
            "{} case(s): {}",
            results.len(),
            if all_passed { "PASS" } else { "FAIL" }
        );
    }
    if all_passed {
        0
    } else {
        1
    }
}

fn manifest_command(args: &[String]) -> i32 {
    if args.len() != 1 {
        emit(Response::deny_error_for_cli(
            "usage: tethers validate-manifest MANIFEST",
        ));
        return 2;
    }
    let text = match fs::read_to_string(&args[0]) {
        Ok(value) => value,
        Err(error) => {
            emit(Response::deny_error_for_cli(format!(
                "cannot read manifest: {error}"
            )));
            return 3;
        }
    };
    let manifest: Manifest = match serde_json::from_str(&text) {
        Ok(value) => value,
        Err(error) => {
            emit(Response::deny_error(format!(
                "invalid manifest JSON: {error}"
            )));
            return 1;
        }
    };
    match validate_manifest(&manifest) {
        Ok(()) => {
            println!(
                "{{\"valid\":true,\"tool\":{}}}",
                serde_json::to_string(&manifest.tool).unwrap()
            );
            0
        }
        Err(error) => {
            emit(Response::deny_error(error));
            1
        }
    }
}

fn emit(response: Response) {
    println!("{}", tethers_portable::response_json(&response));
}
