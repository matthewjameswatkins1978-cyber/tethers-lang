use serde::Deserialize;
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::fs::OpenOptions;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tethers_portable::{
    evaluate_text_with_config, evaluate_text_with_options, validate_manifest, validate_policy_text,
    Manifest, Policy, Response,
};

fn usage() -> &'static str {
    "usage:\n  tethers evaluate [--input PATH] [--policy PATH] [--manifest PATH] [--explain] [--trace] [--audit PATH]\n  tethers explain [--input PATH] [--policy PATH] [--manifest PATH] [--trace]\n  tethers lint POLICY [--json]\n  tethers init [--profile PROFILE] [--output DIR]\n  tethers doctor [--json]\n  tethers test POLICY CORPUS [--json]\n  tethers validate-manifest MANIFEST\n  tethers --version"
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
        Some("lint") => lint_command(&args[1..]),
        Some("init") => init_command(&args[1..]),
        Some("doctor") => doctor_command(&args[1..]),
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
    let mut trace = false;
    let mut audit_path = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--explain" => {
                explain = true;
                index += 1;
            }
            "--trace" => {
                trace = true;
                index += 1;
            }
            "--input" | "--policy" | "--manifest" | "--audit" => {
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
                    "--manifest" => &mut manifest_path,
                    _ => &mut audit_path,
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
    let mut response = evaluate_text_with_config(
        &input,
        policy.as_deref(),
        manifest.as_deref(),
        explain,
        trace,
    );
    if let Some(path) = audit_path {
        if let Err(error) = append_audit(&path, &input, &response) {
            response = Response::deny_error(format!("audit write failed closed: {error}"));
        }
    }
    emit(response);
    0
}

fn append_audit(path: &PathBuf, input: &str, response: &Response) -> io::Result<()> {
    let request = serde_json::from_str::<Value>(input).unwrap_or(Value::Null);
    let action = request.get("action").and_then(|value| {
        value
            .as_str()
            .or_else(|| value.get("name").and_then(Value::as_str))
    });
    let record = json!({
        "timestamp_unix": SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
        "decision_id": response.decision_id,
        "tethers_version": response.tethers_version,
        "decision": response.decision,
        "rule": response.matched_rule,
        "policy_sha256": response.policy_sha256,
        "actor": request.get("actor").and_then(Value::as_str),
        "action": action,
        "resource": request.get("resource").and_then(Value::as_str),
        "error": response.error,
    });
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(file, "{}", serde_json::to_string(&record).unwrap())
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

fn lint_command(args: &[String]) -> i32 {
    let json_output = args.iter().any(|arg| arg == "--json");
    let positional: Vec<_> = args.iter().filter(|arg| *arg != "--json").collect();
    if positional.len() != 1 {
        emit(Response::deny_error_for_cli(
            "usage: tethers lint POLICY [--json]",
        ));
        return 2;
    }
    let text = match fs::read_to_string(positional[0]) {
        Ok(text) => text,
        Err(error) => {
            emit(Response::deny_error_for_cli(format!(
                "cannot read policy: {error}"
            )));
            return 3;
        }
    };
    let policy: Policy = match serde_json::from_str(&text) {
        Ok(policy) => policy,
        Err(error) => {
            emit(Response::deny_error(format!(
                "invalid policy JSON: {error}"
            )));
            return 1;
        }
    };
    if let Err(error) = validate_policy_text(&text) {
        emit(Response::deny_error(error));
        return 1;
    }
    let mut warnings = Vec::new();
    if policy.default == tethers_portable::PolicyDecision::Allow {
        warnings.push("default ALLOW is broad; prefer DENY with explicit rules".to_owned());
    }
    for rule in &policy.rules {
        if rule.actors.iter().any(|actor| actor == "*")
            || rule.resources.iter().any(|resource| resource == "*")
        {
            warnings.push(format!("rule {} uses a wildcard selector", rule.name));
        }
    }
    if json_output {
        println!(
            "{}",
            serde_json::to_string(&json!({"valid": true, "warnings": warnings})).unwrap()
        );
    } else {
        println!("valid: true");
        for warning in warnings {
            println!("warning: {warning}");
        }
    }
    0
}

fn init_command(args: &[String]) -> i32 {
    let mut profile = "coding-agent-default".to_owned();
    let mut output = PathBuf::from(".tethers");
    let mut index = 0;
    while index < args.len() {
        let flag = args[index].as_str();
        match flag {
            "--profile" | "--output" => {
                let Some(value) = args.get(index + 1) else {
                    emit(Response::deny_error_for_cli(format!(
                        "missing value for {flag}"
                    )));
                    return 2;
                };
                if flag == "--profile" {
                    profile = value.clone();
                } else {
                    output = PathBuf::from(value);
                }
                index += 2;
            }
            _ => {
                emit(Response::deny_error_for_cli(format!(
                    "unknown option: {flag}"
                )));
                return 2;
            }
        }
    }
    let policy = match profile.as_str() {
        "coding-agent-default" => include_str!("../policies/coding-agent-default.json"),
        "read-only-agent" => include_str!("../policies/read-only-agent.json"),
        "ci-worker" => include_str!("../policies/ci-worker.json"),
        "gary-worker" => include_str!("../policies/gary-worker.json"),
        _ => {
            emit(Response::deny_error_for_cli("unknown profile"));
            return 2;
        }
    };
    if let Err(error) = fs::create_dir_all(&output) {
        emit(Response::deny_error_for_cli(format!(
            "cannot create output: {error}"
        )));
        return 3;
    }
    let files = [
        ("policy.json", policy),
        (
            "manifest.json",
            r#"{"schema_version":"1","tool":"tethers-agent","version":"0.2.1","capabilities":["filesystem.read","filesystem.write","git.status","git.diff","test.run"]}"#,
        ),
        (
            "request.json",
            r#"{"schema_version":"1","actor":"agent","action":"workspace.read","resource":"workspace","context":{}}"#,
        ),
        (
            "tests.json",
            r#"{"cases":[{"name":"workspace-read","request":{"schema_version":"1","actor":"agent","action":"workspace.read","resource":"workspace","context":{}},"expect":"ALLOW"}]}"#,
        ),
    ];
    for (name, content) in files {
        if let Err(error) = fs::write(output.join(name), content) {
            emit(Response::deny_error_for_cli(format!(
                "cannot write {name}: {error}"
            )));
            return 3;
        }
    }
    println!("initialized profile {profile} in {}", output.display());
    0
}

fn doctor_command(args: &[String]) -> i32 {
    let json_output = args.iter().any(|arg| arg == "--json");
    if args.iter().any(|arg| arg != "--json") {
        emit(Response::deny_error_for_cli(
            "usage: tethers doctor [--json]",
        ));
        return 2;
    }
    let bundled = include_str!("../policies/coding-agent-default.json");
    let policy_ok = validate_policy_text(bundled).is_ok();
    let result = json!({"version": tethers_portable::version(), "policy_valid": policy_ok, "platform": std::env::consts::OS, "architecture": std::env::consts::ARCH});
    if json_output {
        println!("{}", result);
    } else {
        println!("{}", serde_json::to_string_pretty(&result).unwrap());
    }
    if policy_ok {
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
