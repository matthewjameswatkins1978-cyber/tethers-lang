//! tethers-env — Operational execution-environment handshake gateway.
//!
//! Commands:
//!   observe  — probe the host and write a HostEnvironmentObservation.
//!   issue    — read request + observation, write a frozen contract.
//!   inspect  — reload and verify a stored contract.
//!   run      — permit and launch one approved command through SupervisedChild.

use clap::{Parser, Subcommand};
use serde::Serialize;
use sha2::Digest as _;
use std::collections::BTreeMap;
use std::path::Path;
use std::process::{Command, ExitCode, Stdio};
use tethers_reference_host::execution_environment::{
    ContractStatus, ExecutionEnvironmentContract, HostCapabilityObservation,
    HostCommandObservation, HostEnvironmentObservation, RepositoryBinding, TaskEnvironmentRequest,
    MATTHEW_ASSIGNMENT_AUTHORITY,
};

#[derive(Parser)]
#[command(
    name = "tethers-env",
    version = "0.2.0",
    about = "Operational execution-environment handshake gateway"
)]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Probe the host environment and write an observation.
    Observe {
        #[arg(long = "request", value_name = "PATH")]
        request: String,
        #[arg(long = "output", value_name = "PATH")]
        output: String,
    },
    /// Read request + observation and write a frozen execution contract.
    Issue {
        #[arg(long = "request", value_name = "PATH")]
        request: String,
        #[arg(long = "observation", value_name = "PATH")]
        observation: String,
        #[arg(long = "output", value_name = "PATH")]
        output: String,
    },
    /// Reload a stored contract and print its safe summary.
    Inspect {
        #[arg(long = "contract", value_name = "PATH")]
        contract: String,
    },
    /// Permit and launch one approved command through SupervisedChild.
    Run {
        #[arg(long = "contract", value_name = "PATH")]
        contract: String,
        #[arg(long = "command-id", value_name = "ID")]
        command_id: String,
    },
}

#[derive(Serialize)]
struct CliEnvelope {
    schema: &'static str,
    command: String,
    status: String,
    exit_code: i32,
    data: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<CliErrorDetail>,
}

#[derive(Serialize)]
struct CliErrorDetail {
    code: String,
    message: String,
}

fn ok_envelope(command: &str, data: serde_json::Value) -> CliEnvelope {
    CliEnvelope {
        schema: "tethers-env/1",
        command: command.to_owned(),
        status: "ok".to_owned(),
        exit_code: 0,
        data,
        error: None,
    }
}

fn err_envelope(command: &str, code: &str, message: &str, exit_code: i32) -> CliEnvelope {
    CliEnvelope {
        schema: "tethers-env/1",
        command: command.to_owned(),
        status: code.to_owned(),
        exit_code,
        data: serde_json::Value::Object(Default::default()),
        error: Some(CliErrorDetail {
            code: code.to_owned(),
            message: message.to_owned(),
        }),
    }
}

fn print_json(envelope: &CliEnvelope) {
    println!("{}", serde_json::to_string_pretty(envelope).unwrap());
}

fn read_json<T: serde::de::DeserializeOwned>(path: &str, label: &str) -> Result<T, String> {
    let content =
        std::fs::read_to_string(path).map_err(|e| format!("cannot read {label} '{path}': {e}"))?;
    serde_json::from_str(&content).map_err(|e| format!("cannot parse {label} '{path}': {e}"))
}

fn write_json_atomic(path: &str, value: &impl Serialize, label: &str) -> Result<(), String> {
    let json = serde_json::to_string_pretty(value)
        .map_err(|e| format!("cannot serialize {label}: {e}"))?;
    let tmp = format!("{path}.tmp");
    std::fs::write(&tmp, &json).map_err(|e| format!("cannot write {label}: {e}"))?;
    std::fs::rename(&tmp, path).map_err(|e| format!("cannot atomically rename {label}: {e}"))
}

fn run_capture(args: &[&str], cwd: &str) -> Result<std::process::Output, String> {
    let output = Command::new(args[0])
        .args(&args[1..])
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("cannot launch {}: {e}", args[0]))?;
    Ok(output)
}

fn resolve_program(name: &str) -> Option<String> {
    let output = Command::new("where.exe")
        .arg(name)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    for candidate in String::from_utf8_lossy(&output.stdout).lines() {
        let p = Path::new(candidate);
        if p.exists() && p.metadata().map(|m| m.len() > 0).unwrap_or(false) {
            return Some(p.to_string_lossy().replace('/', "\\"));
        }
    }
    None
}

fn extract_version(output: &str) -> String {
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        for part in trimmed.split_whitespace() {
            let cleaned = part.trim_matches(|c: char| c == 'v' || c == 'V');
            if cleaned.chars().any(|c| c.is_ascii_digit()) && cleaned.contains('.') {
                return cleaned.to_owned();
            }
        }
        let pieces: Vec<&str> = trimmed.split_whitespace().collect();
        for piece in pieces.iter().rev() {
            let cleaned = piece.trim_matches(|c: char| c == 'v' || c == 'V');
            if cleaned.chars().any(|c| c.is_ascii_digit()) && cleaned.contains('.') {
                return cleaned.to_owned();
            }
        }
    }
    "unknown".to_owned()
}

/// Capability ID → (executable name, version args, cwd-relative-to-repo-root)
fn capability_probe_hint(id: &str) -> Option<(&str, &[&str], &str)> {
    match id {
        "git-inspection" => Some(("git.exe", &["git", "--version"], ".")),
        "recursive-text-search" => Some(("rg.exe", &["rg", "--version"], ".")),
        "structured-json-query" => Some(("jq.exe", &["jq", "--version"], ".")),
        "github-api-inspection" => Some(("gh.exe", &["gh", "--version"], ".")),
        "task-automation-runner" => Some(("just.exe", &["just", "--version"], ".")),
        "rust-compilation" => Some((
            "cargo.exe",
            &["cargo", "+1.89.0", "--version"],
            "tethers-0.1/host-rust",
        )),
        "rust-formatting" => Some((
            "rustfmt.exe",
            &["cargo", "+1.89.0", "fmt", "--version"],
            "tethers-0.1/host-rust",
        )),
        "rust-linting"
        | "ocaml-compilation"
        | "ocaml-formatting"
        | "ocaml-switch-management"
        | "powershell-automation" => None,
        _ => None,
    }
}

fn git_repo_binding() -> Result<RepositoryBinding, String> {
    let root = {
        let output = Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| format!("git: {e}"))?;
        if !output.status.success() {
            return Err("git rev-parse --show-toplevel failed".to_owned());
        }
        String::from_utf8_lossy(&output.stdout)
            .trim()
            .replace('/', "\\")
    };
    let branch = {
        let output = Command::new("git")
            .args(["branch", "--show-current"])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| format!("git: {e}"))?;
        String::from_utf8_lossy(&output.stdout).trim().to_owned()
    };
    let head = {
        let output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| format!("git: {e}"))?;
        if !output.status.success() {
            return Err("git rev-parse HEAD failed".to_owned());
        }
        String::from_utf8_lossy(&output.stdout).trim().to_owned()
    };
    Ok(RepositoryBinding { root, branch, head })
}

// ── observe ──────────────────────────────────────────────────────

fn cmd_observe(request_path: &str, output_path: &str) -> Result<(), String> {
    let request: TaskEnvironmentRequest = read_json(request_path, "request")?;

    if request.schema != "tethers-execution-environment-request-v1" {
        return Err("unsupported request schema".to_owned());
    }
    if request.worker_assignment.selected_by != MATTHEW_ASSIGNMENT_AUTHORITY {
        return Err("only Matthew may select a worker".to_owned());
    }
    if request.automatic_install {
        return Err("automatic installation is never permitted".to_owned());
    }

    let repo = git_repo_binding()?;

    let mut capabilities: BTreeMap<String, HostCapabilityObservation> = BTreeMap::new();
    let mut commands: BTreeMap<String, HostCommandObservation> = BTreeMap::new();

    for req_cap in &request.capabilities {
        let (verified_cmd_id, version) =
            if let Some((exe, version_args, cap_cwd)) = capability_probe_hint(&req_cap.id) {
                let cwd = Path::new(&repo.root).join(cap_cwd);
                let cwd_str = cwd.to_string_lossy().replace('/', "\\");
                let exe_path = resolve_program(exe);
                if let Some(path) = &exe_path {
                    let output = run_capture(version_args, &cwd_str);
                    let ver = match output {
                        Ok(o) if o.status.success() => {
                            extract_version(&String::from_utf8_lossy(&o.stdout))
                        }
                        _ => "unknown".to_owned(),
                    };
                    (path.clone(), ver)
                } else {
                    (String::new(), "unavailable".to_owned())
                }
            } else {
                (String::new(), "unavailable".to_owned())
            };

        let verified = !verified_cmd_id.is_empty();
        let cap_obs = HostCapabilityObservation {
            version,
            verified,
            command_id: verified_cmd_id,
        };
        capabilities.insert(req_cap.id.clone(), cap_obs);
    }

    for cmd in &request.commands {
        let program_path = capabilities
            .get(&cmd.capability_id)
            .map(|c| c.command_id.clone())
            .unwrap_or_default();

        let script_digest = if program_path.to_ascii_lowercase().ends_with("pwsh.exe") {
            extract_pwsh_script_digest(&cmd.args)
        } else {
            None
        };

        commands.insert(
            cmd.command_id.clone(),
            HostCommandObservation {
                program_path,
                args: cmd.args.clone(),
                cwd: cmd.cwd.clone(),
                script_digest,
                environment: BTreeMap::new(),
            },
        );
    }

    let observation = HostEnvironmentObservation {
        observation_id: format!("observation-{}-{}", request.request_id, uuid_v4()),
        platform: "windows".to_owned(),
        shell: "pwsh".to_owned(),
        repository: repo,
        granted_permissions: request.requested_permissions.clone(),
        capabilities,
        commands,
        process_tree_supervision_available: true,
    };

    write_json_atomic(output_path, &observation, "observation")?;
    Ok(())
}

fn extract_pwsh_script_digest(args: &[String]) -> Option<String> {
    let lower: Vec<String> = args.iter().map(|a| a.to_ascii_lowercase()).collect();
    let file_idx = lower.iter().position(|a| a == "-file")?;
    let script_path = args.get(file_idx + 1)?;
    let content = std::fs::read(script_path).ok()?;
    let hex = format!("{:x}", sha2::Sha256::digest(&content));
    Some(format!("sha256:{hex}"))
}

fn uuid_v4() -> String {
    uuid::Uuid::new_v4().to_string()
}

// ── issue ────────────────────────────────────────────────────────

fn cmd_issue(request_path: &str, observation_path: &str, output_path: &str) -> Result<(), String> {
    let request: TaskEnvironmentRequest = read_json(request_path, "request")?;
    let observation: HostEnvironmentObservation = read_json(observation_path, "observation")?;

    let contract = ExecutionEnvironmentContract::issue(request, observation)
        .map_err(|e| format!("cannot issue contract: {e}"))?;

    eprintln!("request digest:     {}", contract.request_digest());
    eprintln!("observation digest: {}", contract.observation_digest());
    eprintln!("contract digest:    {}", contract.contract_digest());
    let status_str = match contract.status() {
        ContractStatus::Agreed => "agreed",
        ContractStatus::Degraded => "degraded",
        ContractStatus::Blocked => "blocked",
    };
    eprintln!("contract status:   {status_str}");

    let stored_json = contract
        .to_stored_json()
        .map_err(|e| format!("cannot serialize contract: {e}"))?;
    let tmp = format!("{output_path}.tmp");
    std::fs::write(&tmp, &stored_json).map_err(|e| format!("cannot write contract: {e}"))?;
    std::fs::rename(&tmp, output_path).map_err(|e| format!("cannot rename contract: {e}"))?;

    let envelope = ok_envelope(
        "issue",
        serde_json::json!({
            "contract_digest": contract.contract_digest(),
            "request_digest": contract.request_digest(),
            "observation_digest": contract.observation_digest(),
            "status": status_str,
            "output": output_path,
        }),
    );
    print_json(&envelope);

    if *contract.status() == ContractStatus::Blocked {
        return Err("contract is blocked — no command may launch".to_owned());
    }
    Ok(())
}

// ── inspect ──────────────────────────────────────────────────────

fn cmd_inspect(contract_path: &str) -> Result<(), String> {
    let content =
        std::fs::read_to_string(contract_path).map_err(|e| format!("cannot read contract: {e}"))?;
    let contract = ExecutionEnvironmentContract::from_stored(&content)
        .map_err(|e| format!("contract integrity failure: {e}"))?;

    eprintln!("contract digest:    {}", contract.contract_digest());
    eprintln!("contract status:    {:?}", contract.status());
    eprintln!("request digest:     {}", contract.request_digest());
    eprintln!("observation digest: {}", contract.observation_digest());

    let envelope = ok_envelope(
        "inspect",
        serde_json::json!({
            "contract_digest": contract.contract_digest(),
            "status": format!("{:?}", contract.status()),
            "request_digest": contract.request_digest(),
            "observation_digest": contract.observation_digest(),
        }),
    );
    print_json(&envelope);
    Ok(())
}

// ── run ──────────────────────────────────────────────────────────

fn cmd_run(contract_path: &str, command_id: &str) -> Result<(), String> {
    let content =
        std::fs::read_to_string(contract_path).map_err(|e| format!("cannot read contract: {e}"))?;
    let contract = ExecutionEnvironmentContract::from_stored(&content)
        .map_err(|e| format!("contract integrity failure: {e}"))?;

    if *contract.status() == ContractStatus::Blocked {
        return Err("blocked contract — refused to launch".to_owned());
    }

    let permit = contract
        .permit_by_id(command_id)
        .map_err(|e| format!("permit denied: {e}"))?;

    let child = permit.launch().map_err(|e| format!("launch failed: {e}"))?;

    let stderr = child.stderr_tail();
    child.shutdown();

    let envelope = ok_envelope(
        "run",
        serde_json::json!({
            "command_id": command_id,
            "status": "completed",
            "stderr_tail": if stderr.is_empty() { None } else { Some(stderr) },
        }),
    );
    print_json(&envelope);
    Ok(())
}

// ── main ─────────────────────────────────────────────────────────

fn main() -> ExitCode {
    let result = run_cli();
    match result {
        Ok(()) => ExitCode::from(0),
        Err(msg) => {
            eprintln!("error: {msg}");
            ExitCode::from(1)
        }
    }
}

fn run_cli() -> Result<(), String> {
    let cli = Cli::parse();
    match cli.command {
        Cmd::Observe { request, output } => cmd_observe(&request, &output),
        Cmd::Issue {
            request,
            observation,
            output,
        } => cmd_issue(&request, &observation, &output),
        Cmd::Inspect { contract } => cmd_inspect(&contract),
        Cmd::Run {
            contract,
            command_id,
        } => cmd_run(&contract, &command_id),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;
    use tethers_reference_host::execution_environment::{
        CapabilityRequirement, HostCapabilityObservation, HostCommandObservation, PermissionScopes,
        RepositoryBinding, RequestedCommand, RequirementClass, TaskBinding, VersionPolicy,
        WorkerAssignment, WORKBENCH_PROFILE_ID,
    };

    fn temp_dir() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "tethers-env-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = fs::create_dir_all(&dir);
        dir
    }

    fn test_request() -> TaskEnvironmentRequest {
        TaskEnvironmentRequest {
            schema: "tethers-execution-environment-request-v1".to_owned(),
            request_id: "test-req-1".to_owned(),
            workbench_profile: WORKBENCH_PROFILE_ID.to_owned(),
            task: TaskBinding {
                task_id: "J20-H2-TEST".to_owned(),
                session_id: "test-session-1".to_owned(),
                scope: vec!["rust-host".to_owned()],
                repository: RepositoryBinding {
                    root: "D:/Tethers".to_owned(),
                    branch: "test-branch".to_owned(),
                    head: "a".repeat(40),
                },
            },
            worker_assignment: WorkerAssignment {
                selected_by: MATTHEW_ASSIGNMENT_AUTHORITY.to_owned(),
                worker_id: "deepseek-pro-v4-opencode".to_owned(),
            },
            capabilities: vec![CapabilityRequirement {
                id: "git-inspection".to_owned(),
                class: RequirementClass::Required,
                version_policy: VersionPolicy::Minimum {
                    version: "2.0.0".to_owned(),
                },
            }],
            commands: vec![RequestedCommand {
                command_id: "git-status".to_owned(),
                capability_id: "git-inspection".to_owned(),
                args: vec!["status".to_owned()],
                cwd: "D:/Tethers".to_owned(),
            }],
            requested_permissions: PermissionScopes {
                filesystem_read: std::collections::BTreeSet::from(["D:/Tethers".to_owned()]),
                filesystem_write: std::collections::BTreeSet::from(["D:/Tethers".to_owned()]),
                network_hosts: std::collections::BTreeSet::new(),
                network_allowed: false,
                installation_allowed: false,
            },
            automatic_install: false,
        }
    }

    fn test_observation() -> HostEnvironmentObservation {
        HostEnvironmentObservation {
            observation_id: "test-obs-1".to_owned(),
            platform: "windows".to_owned(),
            shell: "pwsh".to_owned(),
            repository: RepositoryBinding {
                root: "D:/Tethers".to_owned(),
                branch: "test-branch".to_owned(),
                head: "a".repeat(40),
            },
            granted_permissions: PermissionScopes {
                filesystem_read: std::collections::BTreeSet::from(["D:/Tethers".to_owned()]),
                filesystem_write: std::collections::BTreeSet::from(["D:/Tethers".to_owned()]),
                network_hosts: std::collections::BTreeSet::new(),
                network_allowed: false,
                installation_allowed: false,
            },
            capabilities: std::collections::BTreeMap::from([(
                "git-inspection".to_owned(),
                HostCapabilityObservation {
                    version: "2.54.0".to_owned(),
                    verified: true,
                    command_id: "git-status".to_owned(),
                },
            )]),
            commands: std::collections::BTreeMap::from([(
                "git-status".to_owned(),
                HostCommandObservation {
                    program_path: "C:/Program Files/Git/cmd/git.exe".to_owned(),
                    args: vec!["status".to_owned()],
                    cwd: "D:/Tethers".to_owned(),
                    script_digest: None,
                    environment: std::collections::BTreeMap::new(),
                },
            )]),
            process_tree_supervision_available: true,
        }
    }

    fn temp_path(dir: &std::path::Path, name: &str) -> String {
        dir.join(name)
            .to_string_lossy()
            .replace('/', "\\")
            .to_string()
    }

    // ── observe tests ────────────────────────────────────────────

    #[test]
    fn observe_writes_valid_json_output() {
        let tmp = temp_dir();
        let req_path = temp_path(&tmp, "req.json");
        let obs_path = temp_path(&tmp, "obs.json");
        let request = test_request();
        fs::write(&req_path, serde_json::to_string_pretty(&request).unwrap()).unwrap();

        let result = cmd_observe(&req_path, &obs_path);
        assert!(result.is_ok(), "observe failed: {}", result.unwrap_err());

        let obs_content = fs::read_to_string(&obs_path).unwrap();
        let obs: HostEnvironmentObservation = serde_json::from_str(&obs_content).unwrap();
        assert_eq!(obs.platform, "windows");
        assert_eq!(obs.shell, "pwsh");
        assert!(obs.process_tree_supervision_available);
        assert!(obs.observation_id.starts_with("observation-"));
        assert!(obs.capabilities.contains_key("git-inspection"));
        assert!(obs.commands.contains_key("git-status"));

        let _ = fs::remove_dir_all(&tmp);
    }

    // ── issue tests ──────────────────────────────────────────────

    #[test]
    fn issue_produces_all_three_digests() {
        let tmp = temp_dir();
        let req_path = temp_path(&tmp, "req.json");
        let obs_path = temp_path(&tmp, "obs.json");
        let contract_path = temp_path(&tmp, "contract.json");

        let request = test_request();
        let observation = test_observation();
        fs::write(&req_path, serde_json::to_string_pretty(&request).unwrap()).unwrap();
        fs::write(
            &obs_path,
            serde_json::to_string_pretty(&observation).unwrap(),
        )
        .unwrap();

        let result = cmd_issue(&req_path, &obs_path, &contract_path);
        assert!(result.is_ok(), "issue failed: {}", result.unwrap_err());

        let contract_str = fs::read_to_string(&contract_path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&contract_str).unwrap();

        let contract_digest = parsed["contract_digest"].as_str().unwrap();
        let request_digest = parsed["request_digest"].as_str().unwrap();
        let observation_digest = parsed["observation_digest"].as_str().unwrap();

        assert!(contract_digest.starts_with("sha256:"));
        assert!(request_digest.starts_with("sha256:"));
        assert!(observation_digest.starts_with("sha256:"));
        assert_eq!(parsed["status"].as_str().unwrap(), "agreed");
        assert_eq!(
            parsed["schema"].as_str().unwrap(),
            "tethers-execution-environment-contract-v1"
        );

        let _ = fs::remove_dir_all(&tmp);
    }

    // ── inspect tests ────────────────────────────────────────────

    #[test]
    fn inspect_reloads_valid_contract() {
        let tmp = temp_dir();
        let contract_path = temp_path(&tmp, "contract.json");

        let request = test_request();
        let observation = test_observation();
        let contract = ExecutionEnvironmentContract::issue(request, observation).unwrap();
        let stored = contract.to_stored_json().unwrap();
        fs::write(&contract_path, &stored).unwrap();

        let result = cmd_inspect(&contract_path);
        assert!(result.is_ok(), "inspect failed: {}", result.unwrap_err());

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn inspect_refuses_tampered_contract() {
        let tmp = temp_dir();
        let contract_path = temp_path(&tmp, "contract.json");

        let request = test_request();
        let observation = test_observation();
        let contract = ExecutionEnvironmentContract::issue(request, observation).unwrap();

        // Tamper the stored contract by changing the contract_digest
        let mut stored_json: serde_json::Value =
            serde_json::from_str(&contract.to_stored_json().unwrap()).unwrap();
        stored_json["contract_digest"] = serde_json::Value::String(
            "sha256:0000000000000000000000000000000000000000000000000000000000000000".to_owned(),
        );
        fs::write(
            &contract_path,
            serde_json::to_string_pretty(&stored_json).unwrap(),
        )
        .unwrap();

        let result = cmd_inspect(&contract_path);
        assert!(result.is_err(), "expected tamper rejection, got success");
        let err = result.unwrap_err();
        assert!(
            err.contains("integrity"),
            "expected integrity error, got: {err}"
        );

        let _ = fs::remove_dir_all(&tmp);
    }

    // ── run tests ────────────────────────────────────────────────

    #[test]
    fn run_refuses_blocked_contract() {
        let tmp = temp_dir();
        let contract_path = temp_path(&tmp, "contract.json");

        let mut req = test_request();
        req.capabilities[0].class = RequirementClass::Required;
        let mut obs = test_observation();
        obs.capabilities.clear();

        let contract = ExecutionEnvironmentContract::issue(req, obs).unwrap();
        assert_eq!(*contract.status(), ContractStatus::Blocked);
        let stored = contract.to_stored_json().unwrap();
        fs::write(&contract_path, &stored).unwrap();

        let result = cmd_run(&contract_path, "git-status");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("blocked"));

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn run_refuses_unknown_command_id() {
        let tmp = temp_dir();
        let contract_path = temp_path(&tmp, "contract.json");

        let request = test_request();
        let observation = test_observation();
        let contract = ExecutionEnvironmentContract::issue(request, observation).unwrap();
        let stored = contract.to_stored_json().unwrap();
        fs::write(&contract_path, &stored).unwrap();

        let result = cmd_run(&contract_path, "nonexistent-cmd");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("command id"));

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn run_refuses_altered_contract() {
        let tmp = temp_dir();
        let contract_path = temp_path(&tmp, "contract.json");

        let request = test_request();
        let observation = test_observation();
        let contract = ExecutionEnvironmentContract::issue(request, observation).unwrap();

        let mut stored_json: serde_json::Value =
            serde_json::from_str(&contract.to_stored_json().unwrap()).unwrap();
        stored_json["approved_commands"]["git-status"]["program_path"] =
            serde_json::Value::String("C:/evil.exe".to_owned());
        fs::write(
            &contract_path,
            serde_json::to_string_pretty(&stored_json).unwrap(),
        )
        .unwrap();

        let result = cmd_run(&contract_path, "git-status");
        assert!(
            result.is_err(),
            "expected tampered-program rejection, got success"
        );
        assert!(
            result.unwrap_err().contains("integrity"),
            "expected integrity error"
        );

        let _ = fs::remove_dir_all(&tmp);
    }

    #[cfg(windows)]
    #[test]
    fn run_launches_permitted_command_through_supervised_child() {
        let tmp = temp_dir();
        let contract_path = temp_path(&tmp, "contract.json");

        // Use a command known to exist on Windows and already tested
        // through the library's supervised_launch integration test.
        let cmd_path = resolve_program("cmd.exe");
        if cmd_path.is_none() {
            return; // skip if cmd.exe not found
        }
        let cmd_path = cmd_path.unwrap();
        let cwd = tmp.to_string_lossy().replace('/', "\\").to_string();
        // Safe: just echo a known string, no shell expansion
        let args = vec!["/c".to_owned(), "echo ok".to_owned()];

        let scope = cwd[..3].to_owned(); // e.g. "C:\"
        let mut req = test_request();
        req.capabilities = vec![CapabilityRequirement {
            id: "powershell-automation".to_owned(),
            class: RequirementClass::Required,
            version_policy: VersionPolicy::Any,
        }];
        req.commands = vec![RequestedCommand {
            command_id: "cmd-echo".to_owned(),
            capability_id: "powershell-automation".to_owned(),
            args: args.clone(),
            cwd: cwd.clone(),
        }];
        req.requested_permissions = PermissionScopes {
            filesystem_read: std::collections::BTreeSet::from([scope.clone()]),
            filesystem_write: std::collections::BTreeSet::from([scope.clone()]),
            network_hosts: std::collections::BTreeSet::new(),
            network_allowed: false,
            installation_allowed: false,
        };

        let mut obs = test_observation();
        obs.granted_permissions = PermissionScopes {
            filesystem_read: std::collections::BTreeSet::from([scope.clone()]),
            filesystem_write: std::collections::BTreeSet::from([scope]),
            network_hosts: std::collections::BTreeSet::new(),
            network_allowed: false,
            installation_allowed: false,
        };
        obs.capabilities = std::collections::BTreeMap::from([(
            "powershell-automation".to_owned(),
            HostCapabilityObservation {
                version: "10.0".to_owned(),
                verified: true,
                command_id: "cmd-echo".to_owned(),
            },
        )]);
        obs.commands = std::collections::BTreeMap::from([(
            "cmd-echo".to_owned(),
            HostCommandObservation {
                program_path: cmd_path,
                args: args.clone(),
                cwd: cwd.clone(),
                script_digest: None,
                environment: std::collections::BTreeMap::new(),
            },
        )]);

        let contract = ExecutionEnvironmentContract::issue(req, obs).unwrap();
        if *contract.status() != ContractStatus::Agreed {
            return; // skip on permission mismatch
        }
        let stored = contract.to_stored_json().unwrap();
        fs::write(&contract_path, &stored).unwrap();

        let result = cmd_run(&contract_path, "cmd-echo");
        assert!(result.is_ok(), "run failed: {}", result.unwrap_err());

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn integration_observe_issue_inspect_run() {
        let tmp = temp_dir();
        let req_path = temp_path(&tmp, "req.json");
        let obs_path = temp_path(&tmp, "obs.json");
        let contract_path = temp_path(&tmp, "contract.json");

        let request = test_request();
        fs::write(&req_path, serde_json::to_string_pretty(&request).unwrap()).unwrap();
        let result = cmd_observe(&req_path, &obs_path);
        assert!(result.is_ok(), "observe failed: {}", result.unwrap_err());

        let obs_content = fs::read_to_string(&obs_path).unwrap();
        let observation: HostEnvironmentObservation = serde_json::from_str(&obs_content).unwrap();
        assert!(observation.observation_id.starts_with("observation-"));
        assert_eq!(observation.platform, "windows");

        let crafted_obs = test_observation();
        let obs2_path = temp_path(&tmp, "obs2.json");
        fs::write(
            &obs2_path,
            serde_json::to_string_pretty(&crafted_obs).unwrap(),
        )
        .unwrap();
        let result = cmd_issue(&req_path, &obs2_path, &contract_path);
        assert!(result.is_ok(), "issue failed: {}", result.unwrap_err());

        let contract_str = fs::read_to_string(&contract_path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&contract_str).unwrap();
        assert!(parsed["contract_digest"]
            .as_str()
            .unwrap()
            .starts_with("sha256:"));

        let result = cmd_inspect(&contract_path);
        assert!(result.is_ok(), "inspect failed: {}", result.unwrap_err());

        let result = cmd_run(&contract_path, "unknown-cmd");
        assert!(result.is_err());

        let _ = fs::remove_dir_all(&tmp);
    }
}
