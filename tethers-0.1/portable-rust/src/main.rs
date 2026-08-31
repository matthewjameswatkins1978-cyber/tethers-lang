use serde::Deserialize;
use serde_json::{json, Value};
use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tethers_portable::{
    evaluate_text_with_config, evaluate_text_with_options, validate_manifest, validate_policy_text,
    Manifest, Policy, Response,
};

const EXIT_ALLOW: i32 = 0;
const EXIT_ASK: i32 = 10;
const EXIT_DENY: i32 = 20;
const EXIT_USAGE: i32 = 64;
const EXIT_POLICY: i32 = 65;
const EXIT_UNAVAILABLE: i32 = 66;
const EXIT_INTERNAL: i32 = 70;
const ROOT_POLICY: &str = include_str!("../policies/coding-agent-default.json");

fn main() {
    std::process::exit(run(env::args().skip(1).collect()));
}

fn run(args: Vec<String>) -> i32 {
    if args == ["--version"] || args == ["-V"] {
        println!("tethers portable {}", tethers_portable::version());
        return EXIT_ALLOW;
    }
    if args.is_empty() || args == ["--help"] || args == ["-h"] {
        println!("{}", top_help());
        return if args.is_empty() {
            EXIT_USAGE
        } else {
            EXIT_ALLOW
        };
    }
    match args.first().map(String::as_str) {
        Some("check") => check_command(&args[1..]),
        Some("validate") => validate_command(&args[1..]),
        Some("version") => version_command(&args[1..]),
        Some("doctor") => doctor_command(&args[1..]),
        Some("evaluate") => evaluate_command(&args[1..]),
        Some("explain") => {
            let mut forwarded = args[1..].to_vec();
            forwarded.push("--explain".to_owned());
            evaluate_command(&forwarded)
        }
        Some("test") => test_command(&args[1..]),
        Some("validate-manifest") => manifest_command(&args[1..]),
        Some("lint") => lint_command(&args[1..]),
        Some("init") => init_command(&args[1..]),
        _ => cli_error("unknown command", EXIT_USAGE),
    }
}

fn top_help() -> &'static str {
    "usage:
  tethers check [REQUEST|-] [--action ACTION] [--policy POLICY] [--json|--quiet|--explain]
  tethers validate POLICY [--json]
  tethers version [--json]
  tethers doctor [--json]
  tethers evaluate [--input PATH|-] [--policy PATH] [--manifest PATH] [--explain] [--trace] [--audit PATH]
  tethers explain [--input PATH|-] [--policy PATH] [--manifest PATH]
  tethers test POLICY CORPUS [--json]
  tethers lint POLICY [--json]
  tethers init [--profile PROFILE] [--output DIR]
  tethers validate-manifest MANIFEST

Use tethers check --help, validate --help, doctor --help, or version --help for examples."
}

fn check_help() -> &'static str {
    "usage:
  tethers check REQUEST.json
  cat REQUEST.json | tethers check -
  tethers check --action git.push
  tethers check REQUEST.json --policy POLICY.json
  tethers check REQUEST.json --json
  tethers check REQUEST.json --explain
  tethers check REQUEST.json --quiet

Exit status: ALLOW=0, ASK=10, DENY=20; usage=64, invalid policy/input=65, unavailable file=66, internal failure=70.
--quiet suppresses stdout and cannot be combined with --json or --explain."
}

fn validate_help() -> &'static str {
    "usage: tethers validate POLICY [--json]\n\nChecks policy structure and loadability only. It never executes an action. Use - for stdin."
}

fn doctor_help() -> &'static str {
    "usage: tethers doctor [--json]\n\nRuns deterministic local checks for version metadata, the bundled policy, the evaluator, and the parity corpus."
}

fn version_help() -> &'static str {
    "usage: tethers version [--json]\n\nPrints stable compatibility metadata without timestamps."
}

fn cli_error(message: impl AsRef<str>, code: i32) -> i32 {
    eprintln!("tethers: {}", message.as_ref());
    code
}

fn read_source(path: Option<&PathBuf>, label: &str) -> Result<String, i32> {
    match path {
        Some(path) if path.as_os_str() == "-" => {
            let mut value = String::new();
            io::stdin().read_to_string(&mut value).map_err(|error| {
                cli_error(
                    format!("cannot read {label} from stdin: {error}"),
                    EXIT_UNAVAILABLE,
                )
            })?;
            Ok(value)
        }
        Some(path) => fs::read_to_string(path).map_err(|error| {
            cli_error(
                format!("cannot read {label} {}: {error}", path.display()),
                EXIT_UNAVAILABLE,
            )
        }),
        None => {
            let mut value = String::new();
            io::stdin().read_to_string(&mut value).map_err(|error| {
                cli_error(
                    format!("cannot read {label} from stdin: {error}"),
                    EXIT_UNAVAILABLE,
                )
            })?;
            Ok(value)
        }
    }
}

fn read_optional(path: Option<&PathBuf>, label: &str) -> Result<Option<String>, i32> {
    path.map(|path| read_source(Some(path), label)).transpose()
}

fn check_command(args: &[String]) -> i32 {
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("{}", check_help());
        return EXIT_ALLOW;
    }
    let mut input_path = None;
    let mut policy_path = None;
    let mut manifest_path = None;
    let mut action = None;
    let mut actor = "agent".to_owned();
    let mut resource = "workspace".to_owned();
    let mut json_output = false;
    let mut quiet = false;
    let mut explain = false;
    let mut trace = false;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--json" => {
                json_output = true;
                index += 1;
            }
            "--quiet" => {
                quiet = true;
                index += 1;
            }
            "--explain" => {
                explain = true;
                index += 1;
            }
            "--trace" => {
                trace = true;
                index += 1;
            }
            "--action" | "--policy" | "--manifest" | "--actor" | "--resource" => {
                let flag = args[index].clone();
                let Some(value) = args.get(index + 1) else {
                    return cli_error(format!("missing value for {flag}"), EXIT_USAGE);
                };
                match flag.as_str() {
                    "--action" => {
                        if action.is_some() {
                            return cli_error("duplicate option: --action", EXIT_USAGE);
                        }
                        action = Some(value.clone());
                    }
                    "--policy" => {
                        if policy_path.is_some() {
                            return cli_error("duplicate option: --policy", EXIT_USAGE);
                        }
                        policy_path = Some(PathBuf::from(value));
                    }
                    "--manifest" => {
                        if manifest_path.is_some() {
                            return cli_error("duplicate option: --manifest", EXIT_USAGE);
                        }
                        manifest_path = Some(PathBuf::from(value));
                    }
                    "--actor" => actor = value.clone(),
                    "--resource" => resource = value.clone(),
                    _ => unreachable!(),
                }
                index += 2;
            }
            "-" => {
                if input_path.is_some() {
                    return cli_error("multiple request inputs", EXIT_USAGE);
                }
                input_path = Some(PathBuf::from("-"));
                index += 1;
            }
            value if !value.starts_with('-') => {
                if input_path.is_some() {
                    return cli_error("multiple request inputs", EXIT_USAGE);
                }
                input_path = Some(PathBuf::from(value));
                index += 1;
            }
            flag => return cli_error(format!("unknown option: {flag}"), EXIT_USAGE),
        }
    }
    if quiet && (json_output || explain) {
        return cli_error(
            "--quiet cannot be combined with --json or --explain",
            EXIT_USAGE,
        );
    }
    if action.is_some() && input_path.is_some() {
        return cli_error(
            "--action cannot be combined with a request input",
            EXIT_USAGE,
        );
    }
    let (input, policy) = if let Some(action) = action {
        let request = json!({"schema_version":"1","actor":actor,"action":action,"resource":resource,"context":{}});
        let policy = match read_optional(policy_path.as_ref(), "policy") {
            Ok(Some(value)) => value,
            Ok(None) => ROOT_POLICY.to_owned(),
            Err(code) => return code,
        };
        (request.to_string(), Some(policy))
    } else {
        let input = match read_source(input_path.as_ref(), "request") {
            Ok(value) => value,
            Err(code) => return code,
        };
        let policy = match read_optional(policy_path.as_ref(), "policy") {
            Ok(value) => value,
            Err(code) => return code,
        };
        (input, policy)
    };
    let manifest = match read_optional(manifest_path.as_ref(), "manifest") {
        Ok(value) => value,
        Err(code) => return code,
    };
    let response = evaluate_text_with_config(
        &input,
        policy.as_deref(),
        manifest.as_deref(),
        explain,
        trace,
    );
    if quiet {
        if let Some(error) = &response.error {
            eprintln!("tethers: {error}");
        }
    } else if json_output {
        println!("{}", tethers_portable::response_json(&response));
    } else if explain {
        print_human_explanation(&response);
    } else {
        println!("{}", response.decision);
    }
    if response.error.is_some() {
        EXIT_POLICY
    } else {
        decision_exit(response.decision)
    }
}

fn decision_exit(decision: &str) -> i32 {
    match decision {
        "ALLOW" => EXIT_ALLOW,
        "ASK" => EXIT_ASK,
        "DENY" => EXIT_DENY,
        _ => EXIT_INTERNAL,
    }
}

fn print_human_explanation(response: &Response) {
    println!("Decision: {}", response.decision);
    if let Some(rule) = &response.matched_rule {
        println!("Rule: {rule}");
    }
    if let Some(reason) = &response.reason {
        println!("Reason: {reason}");
    }
    if let Some(conditions) = &response.evaluated_conditions {
        for condition in conditions {
            println!("Condition: {} => {}", condition.condition, condition.result);
        }
    }
    if let Some(trace) = &response.trace {
        for entry in trace {
            println!("Trace: {entry}");
        }
    }
    if let Some(error) = &response.error {
        println!("Error: {error}");
    }
}

fn evaluate_command(args: &[String]) -> i32 {
    let mut input_path = None;
    let mut policy_path = None;
    let mut manifest_path = None;
    let mut audit_path = None;
    let mut explain = false;
    let mut trace = false;
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
            "--json" => {
                index += 1;
            }
            "--input" | "--policy" | "--manifest" | "--audit" => {
                let flag = args[index].clone();
                let Some(value) = args.get(index + 1) else {
                    emit(Response::deny_error_for_cli(format!(
                        "missing value for {flag}"
                    )));
                    return EXIT_ALLOW;
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
                    return EXIT_ALLOW;
                }
                *destination = Some(PathBuf::from(value));
                index += 2;
            }
            "-" if input_path.is_none() => {
                input_path = Some(PathBuf::from("-"));
                index += 1;
            }
            flag => {
                emit(Response::deny_error_for_cli(format!(
                    "unknown option: {flag}"
                )));
                return EXIT_ALLOW;
            }
        }
    }
    let input = match read_source(input_path.as_ref(), "input") {
        Ok(value) => value,
        Err(code) => {
            emit(Response::deny_error_for_cli("input unavailable"));
            return code;
        }
    };
    let policy = match read_optional(policy_path.as_ref(), "policy") {
        Ok(value) => value,
        Err(code) => {
            emit(Response::deny_error_for_cli("policy unavailable"));
            return code;
        }
    };
    let manifest = match read_optional(manifest_path.as_ref(), "manifest") {
        Ok(value) => value,
        Err(code) => {
            emit(Response::deny_error_for_cli("manifest unavailable"));
            return code;
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
    EXIT_ALLOW
}

fn append_audit(path: &PathBuf, input: &str, response: &Response) -> io::Result<()> {
    let request = serde_json::from_str::<Value>(input).unwrap_or(Value::Null);
    let action = request.get("action").and_then(|value| {
        value
            .as_str()
            .or_else(|| value.get("name").and_then(Value::as_str))
    });
    let record = json!({"timestamp_unix":SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),"decision_id":response.decision_id,"tethers_version":response.tethers_version,"decision":response.decision,"rule":response.matched_rule,"policy_sha256":response.policy_sha256,"actor":request.get("actor").and_then(Value::as_str),"action":action,"resource":request.get("resource").and_then(Value::as_str),"error":response.error});
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(file, "{}", serde_json::to_string(&record).unwrap())
}

fn validate_command(args: &[String]) -> i32 {
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("{}", validate_help());
        return EXIT_ALLOW;
    }
    let json_output = args.iter().any(|arg| arg == "--json");
    let positional: Vec<_> = args.iter().filter(|arg| *arg != "--json").collect();
    if positional.len() != 1 {
        return cli_error("usage: tethers validate POLICY [--json]", EXIT_USAGE);
    }
    let path = PathBuf::from(positional[0]);
    let text = match read_source(Some(&path), "policy") {
        Ok(value) => value,
        Err(code) => return code,
    };
    match serde_json::from_str::<Policy>(&text) {
        Ok(_) => match validate_policy_text(&text) {
            Ok(()) => {
                if json_output {
                    println!("{{\"valid\":true}}");
                } else {
                    println!("VALID");
                }
                EXIT_ALLOW
            }
            Err(error) => validation_failure(json_output, "policy", error),
        },
        Err(error) => validation_failure(json_output, "json", error.to_string()),
    }
}

fn validation_failure(json_output: bool, category: &str, message: String) -> i32 {
    if json_output {
        println!(
            "{}",
            json!({"valid":false,"errors":[{"category":category,"message":message}]})
        );
    } else {
        eprintln!("INVALID: {message}");
    }
    EXIT_POLICY
}

fn version_command(args: &[String]) -> i32 {
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("{}", version_help());
        return EXIT_ALLOW;
    }
    if args.iter().any(|arg| arg != "--json") {
        return cli_error("usage: tethers version [--json]", EXIT_USAGE);
    }
    if args.iter().any(|arg| arg == "--json") {
        println!(
            "{}",
            json!({"name":"tethers","version":tethers_portable::version(),"engine":"portable-rust","policy_schema":"1","target":target_triple()})
        );
    } else {
        println!("tethers {}", tethers_portable::version());
    }
    EXIT_ALLOW
}

fn target_triple() -> &'static str {
    if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        "x86_64-pc-windows-msvc"
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        "x86_64-unknown-linux-musl"
    } else {
        "unknown-target"
    }
}

fn doctor_command(args: &[String]) -> i32 {
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("{}", doctor_help());
        return EXIT_ALLOW;
    }
    if args.iter().any(|arg| arg != "--json") {
        return cli_error("usage: tethers doctor [--json]", EXIT_USAGE);
    }
    let policy_valid = validate_policy_text(ROOT_POLICY).is_ok();
    let corpus_valid =
        serde_json::from_str::<Value>(include_str!("../tests/parity-corpus.json")).is_ok();
    let request = json!({"schema_version":"1","actor":"doctor","action":"workspace.read","resource":"workspace","context":{},"policy":serde_json::from_str::<Value>(ROOT_POLICY).unwrap_or(Value::Null)}).to_string();
    let result = evaluate_text_with_config(&request, None, None, false, false);
    let engine_ok = result.decision == "ALLOW" && result.error.is_none();
    let ok = policy_valid && corpus_valid && engine_ok;
    if args.iter().any(|arg| arg == "--json") {
        println!(
            "{}",
            json!({"version":tethers_portable::version(),"checks":{"version_metadata":true,"policy_parser":policy_valid,"decision_engine":engine_ok,"parity_corpus":corpus_valid},"ok":ok})
        );
    } else {
        println!("Tethers {}", tethers_portable::version());
        println!("version metadata: OK");
        println!(
            "policy parser: {}",
            if policy_valid { "OK" } else { "FAIL" }
        );
        println!("decision engine: {}", if engine_ok { "OK" } else { "FAIL" });
        println!(
            "parity corpus: {}",
            if corpus_valid { "OK" } else { "FAIL" }
        );
        println!("{}", if ok { "OK" } else { "FAIL" });
    }
    if ok {
        EXIT_ALLOW
    } else {
        EXIT_INTERNAL
    }
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
        return EXIT_USAGE;
    }
    let policy = match fs::read_to_string(positional[0]) {
        Ok(value) => value,
        Err(error) => {
            emit(Response::deny_error_for_cli(format!(
                "cannot read policy: {error}"
            )));
            return EXIT_UNAVAILABLE;
        }
    };
    if let Err(error) = validate_policy_text(&policy) {
        emit(Response::deny_error(error));
        return EXIT_POLICY;
    }
    let corpus_text = match fs::read_to_string(positional[1]) {
        Ok(value) => value,
        Err(error) => {
            emit(Response::deny_error_for_cli(format!(
                "cannot read corpus: {error}"
            )));
            return EXIT_UNAVAILABLE;
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
                return EXIT_POLICY;
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
        results.push(json!({"name":case.name,"passed":passed,"expected":expected,"actual":actual.decision,"matched_rule":actual.matched_rule,"error":actual.error}));
    }
    if json_output {
        println!("{}", json!({"passed":all_passed,"cases":results}));
    } else {
        for result in &results {
            println!(
                "{}: {} (expected {}, actual {}, rule {})",
                result["name"],
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
        EXIT_ALLOW
    } else {
        EXIT_DENY
    }
}

fn lint_command(args: &[String]) -> i32 {
    let json_output = args.iter().any(|arg| arg == "--json");
    let positional: Vec<_> = args.iter().filter(|arg| *arg != "--json").collect();
    if positional.len() != 1 {
        return cli_error("usage: tethers lint POLICY [--json]", EXIT_USAGE);
    }
    let text = match fs::read_to_string(positional[0]) {
        Ok(text) => text,
        Err(error) => return cli_error(format!("cannot read policy: {error}"), EXIT_UNAVAILABLE),
    };
    let policy: Policy = match serde_json::from_str(&text) {
        Ok(policy) => policy,
        Err(error) => return validation_failure(json_output, "json", error.to_string()),
    };
    if let Err(error) = validate_policy_text(&text) {
        return validation_failure(json_output, "policy", error);
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
        println!("{}", json!({"valid":true,"warnings":warnings}));
    } else {
        println!("valid: true");
        for warning in warnings {
            println!("warning: {warning}");
        }
    }
    EXIT_ALLOW
}

fn init_command(args: &[String]) -> i32 {
    let mut profile = "coding-agent-default".to_owned();
    let mut output = PathBuf::from(".tethers");
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--profile" | "--output" => {
                let flag = args[index].clone();
                let Some(value) = args.get(index + 1) else {
                    return cli_error(format!("missing value for {flag}"), EXIT_USAGE);
                };
                if flag == "--profile" {
                    profile = value.clone();
                } else {
                    output = PathBuf::from(value);
                }
                index += 2;
            }
            flag => return cli_error(format!("unknown option: {flag}"), EXIT_USAGE),
        }
    }
    let policy = match profile.as_str() {
        "coding-agent-default" => include_str!("../policies/coding-agent-default.json"),
        "read-only-agent" => include_str!("../policies/read-only-agent.json"),
        "ci-worker" => include_str!("../policies/ci-worker.json"),
        "gary-worker" => include_str!("../policies/gary-worker.json"),
        _ => return cli_error("unknown profile", EXIT_USAGE),
    };
    if let Err(error) = fs::create_dir_all(&output) {
        return cli_error(format!("cannot create output: {error}"), EXIT_UNAVAILABLE);
    }
    let files = [
        ("policy.json", policy),
        (
            "manifest.json",
            r#"{"schema_version":"1","tool":"tethers-agent","version":"0.2.2","capabilities":["filesystem.read","filesystem.write","git.status","git.diff","test.run"]}"#,
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
            return cli_error(format!("cannot write {name}: {error}"), EXIT_UNAVAILABLE);
        }
    }
    println!("initialized profile {profile} in {}", output.display());
    EXIT_ALLOW
}

fn manifest_command(args: &[String]) -> i32 {
    if args.len() != 1 {
        emit(Response::deny_error_for_cli(
            "usage: tethers validate-manifest MANIFEST",
        ));
        return EXIT_USAGE;
    }
    let text = match fs::read_to_string(&args[0]) {
        Ok(value) => value,
        Err(error) => {
            emit(Response::deny_error_for_cli(format!(
                "cannot read manifest: {error}"
            )));
            return EXIT_UNAVAILABLE;
        }
    };
    let manifest: Manifest = match serde_json::from_str(&text) {
        Ok(value) => value,
        Err(error) => {
            emit(Response::deny_error(format!(
                "invalid manifest JSON: {error}"
            )));
            return EXIT_POLICY;
        }
    };
    match validate_manifest(&manifest) {
        Ok(()) => {
            println!(
                "{{\"valid\":true,\"tool\":{}}}",
                serde_json::to_string(&manifest.tool).unwrap()
            );
            EXIT_ALLOW
        }
        Err(error) => {
            emit(Response::deny_error(error));
            EXIT_POLICY
        }
    }
}

fn emit(response: Response) {
    println!("{}", tethers_portable::response_json(&response));
}
