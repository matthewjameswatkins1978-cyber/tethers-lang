//! `tethers gate --stdio` — persistent local stdio Authority Gate loop.
//!
//! Newline-delimited JSON frames on stdin/stdout. Diagnostics go to bounded
//! stderr. No prompts, no busy loops, no provider invocation.

use crate::authority_gate::{AuthorityGate, GateConfig};
use crate::cli::{CliEnvelope, OutcomeStatus};
use crate::gate_protocol::{
    parse_frame, AuthorityResponse, FrameError, RequestIdMemory, MAX_FRAME_BYTES,
};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

/// Outer bound waiting for a complete inbound frame before the Gate treats
/// the connection as stalled and fails closed for that wait.
const FRAME_WAIT: Duration = Duration::from_secs(30);

/// Bound on stderr diagnostic lines per session.
const MAX_STDERR_LINES: u32 = 64;

pub struct GateCommandArgs {
    pub stdio: bool,
    pub config: PathBuf,
    pub engine: PathBuf,
    pub trail: PathBuf,
    pub host_data_root: PathBuf,
}

pub struct GateCommandResult {
    pub envelope: CliEnvelope,
    pub exit_code: i32,
}

pub fn run_gate(args: GateCommandArgs) -> GateCommandResult {
    if !args.stdio {
        return failure(
            "GATE_STDIO_REQUIRED",
            "tethers gate requires --stdio for the machine transport",
        );
    }
    for (label, path) in [
        ("--config", &args.config),
        ("--engine", &args.engine),
        ("--trail", &args.trail),
        ("--host-data-root", &args.host_data_root),
    ] {
        if !path.is_absolute() {
            return failure(
                "GATE_PATH_NOT_ABSOLUTE",
                &format!("{label} must be absolute"),
            );
        }
    }
    if !args.config.is_file() {
        return failure(
            "GATE_CONFIG_NOT_FOUND",
            "runtime configuration was not found",
        );
    }
    if !args.engine.is_file() {
        return failure("GATE_ENGINE_NOT_FOUND", "engine binary was not found");
    }
    if let Some(parent) = args.trail.parent() {
        if let Err(error) = std::fs::create_dir_all(parent) {
            return failure(
                "GATE_TRAIL_UNAVAILABLE",
                &format!("cannot create trail directory: {error}"),
            );
        }
    }
    if let Err(error) = std::fs::create_dir_all(&args.host_data_root) {
        return failure(
            "GATE_HOST_DATA_UNAVAILABLE",
            &format!("cannot create host data root: {error}"),
        );
    }
    // Prefer an already-provisioned root (caller-owned ACL). Harden and
    // provision only when durable replay state is not yet established.
    if crate::replay_store::provision_replay(&args.host_data_root).is_err() {
        if let Err(error) = harden_host_data_acl(&args.host_data_root) {
            return failure("GATE_HOST_DATA_UNAVAILABLE", &error);
        }
        if let Err(error) = crate::replay_store::provision_replay(&args.host_data_root) {
            return failure(
                "GATE_REPLAY_UNAVAILABLE",
                &format!("cannot provision durable replay state: {error}"),
            );
        }
    }

    let mut gate = AuthorityGate::new(GateConfig {
        config_path: args.config.clone(),
        engine_path: args.engine.clone(),
        trail_path: args.trail.clone(),
        host_data_root: args.host_data_root.clone(),
    });

    match serve_stdio(&mut gate) {
        Ok(()) => GateCommandResult {
            envelope: CliEnvelope::ok(
                "gate",
                serde_json::json!({
                    "schema": "tethers.gate/1",
                    "transport": "stdio",
                    "shutdown": true,
                    "gate_instance_id": gate.gate_instance_id(),
                    "provider_invocations": gate.provider_invocations(),
                }),
            ),
            exit_code: 0,
        },
        Err(error) => failure("GATE_IO_FAILED", &error),
    }
}

fn failure(code: &str, message: &str) -> GateCommandResult {
    let envelope = CliEnvelope::error(
        "gate",
        OutcomeStatus::Failed,
        code,
        message.to_owned(),
        None,
    );
    GateCommandResult {
        exit_code: envelope.exit_code,
        envelope,
    }
}

/// Apply the accepted protected DACL to a Windows replay root before
/// provisioning. Non-Windows platforms return Ok without changes.
fn harden_host_data_acl(root: &std::path::Path) -> Result<(), String> {
    #[cfg(windows)]
    {
        let acl_script = format!(
            "$p='{}'; $identity=[System.Security.Principal.WindowsIdentity]::GetCurrent().Name; $acl=[System.Security.AccessControl.DirectorySecurity]::new(); $acl.SetAccessRuleProtection($true,$false); $inherit=[System.Security.AccessControl.InheritanceFlags]::ContainerInherit -bor [System.Security.AccessControl.InheritanceFlags]::ObjectInherit; foreach($t in @($identity,'NT AUTHORITY\\SYSTEM','BUILTIN\\Administrators')) {{ $acl.AddAccessRule([System.Security.AccessControl.FileSystemAccessRule]::new($t,'FullControl',$inherit,'None','Allow')) }}; Set-Acl -LiteralPath $p -AclObject $acl",
            root.display()
        );
        let status = std::process::Command::new("pwsh.exe")
            .args(["-NoProfile", "-Command", &acl_script])
            .status()
            .map_err(|error| format!("cannot harden host-data ACL: {error}"))?;
        if !status.success() {
            return Err("host-data root must receive the accepted protected ACL".to_owned());
        }
    }
    #[cfg(not(windows))]
    let _ = root;
    Ok(())
}

fn serve_stdio(gate: &mut AuthorityGate) -> Result<(), String> {
    let stdin = std::io::stdin();
    let (tx, rx) = mpsc::channel::<io_chunk::Chunk>();
    std::thread::spawn(move || {
        let mut reader = BufReader::new(stdin.lock());
        loop {
            match reader.fill_buf() {
                Ok([]) => {
                    let _ = tx.send(io_chunk::Chunk::Eof);
                    break;
                }
                Ok(buffer) => {
                    let len = buffer.len();
                    if tx
                        .send(io_chunk::Chunk::Bytes(buffer[..len].to_vec()))
                        .is_err()
                    {
                        break;
                    }
                    reader.consume(len);
                }
                Err(error) => {
                    let _ = tx.send(io_chunk::Chunk::Error(error.to_string()));
                    break;
                }
            }
        }
    });

    let mut stdout = std::io::stdout();
    let mut buffer: Vec<u8> = Vec::new();
    let mut stderr_lines: u32 = 0;
    let mut memory = RequestIdMemory::default();
    let mut saw_bytes_since_deadline_reset = false;
    let mut discarding_oversized = false;

    loop {
        if gate.shutdown_requested() {
            break;
        }

        // Extract complete lines first.
        while let Some(index) = buffer.iter().position(|byte| *byte == b'\n') {
            let line: Vec<u8> = buffer.drain(..=index).collect();
            saw_bytes_since_deadline_reset = false;
            if discarding_oversized {
                // This newline terminates the already-refused oversized frame.
                discarding_oversized = false;
                continue;
            }
            let text = String::from_utf8_lossy(&line);
            handle_line(gate, &mut stdout, &mut memory, &mut stderr_lines, &text)?;
            if gate.shutdown_requested() {
                return Ok(());
            }
        }

        if discarding_oversized {
            // Keep dropping bytes until the oversized frame's newline arrives.
            // The refusal was already emitted when discard mode began.
            if let Some(index) = buffer.iter().position(|byte| *byte == b'\n') {
                let _line: Vec<u8> = buffer.drain(..=index).collect();
                saw_bytes_since_deadline_reset = false;
                discarding_oversized = false;
            } else {
                buffer.clear();
            }
        } else if buffer.len() > MAX_FRAME_BYTES {
            // Oversized without a newline yet: refuse now, drain the rest.
            discarding_oversized = true;
            buffer.clear();
            saw_bytes_since_deadline_reset = false;
            write_response(
                &mut stdout,
                &mut stderr_lines,
                &AuthorityResponse::error(
                    "",
                    FrameError::Oversized {
                        bytes: MAX_FRAME_BYTES + 1,
                    }
                    .code(),
                    "inbound frame exceeded the maximum size; discarding until newline",
                ),
            )?;
        }

        match rx.recv_timeout(FRAME_WAIT) {
            Ok(io_chunk::Chunk::Bytes(bytes)) => {
                saw_bytes_since_deadline_reset = true;
                buffer.extend_from_slice(&bytes);
            }
            Ok(io_chunk::Chunk::Eof) => {
                if !buffer.is_empty() {
                    buffer.clear();
                    write_response(
                        &mut stdout,
                        &mut stderr_lines,
                        &AuthorityResponse::error(
                            "",
                            FrameError::InvalidJson.code(),
                            "connection closed mid-frame; partial frame discarded",
                        ),
                    )?;
                }
                return Ok(());
            }
            Ok(io_chunk::Chunk::Error(message)) => {
                return Err(format!("stdin read failed: {message}"));
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if !buffer.is_empty() && saw_bytes_since_deadline_reset {
                    buffer.clear();
                    saw_bytes_since_deadline_reset = false;
                    write_response(
                        &mut stdout,
                        &mut stderr_lines,
                        &AuthorityResponse::error(
                            "",
                            "frame.timeout",
                            "partial frame stalled beyond the request lifetime; discarded",
                        ),
                    )?;
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                if !buffer.is_empty() {
                    buffer.clear();
                }
                return Ok(());
            }
        }
    }
    Ok(())
}

mod io_chunk {
    pub enum Chunk {
        Bytes(Vec<u8>),
        Eof,
        Error(String),
    }
}

fn handle_line(
    gate: &mut AuthorityGate,
    stdout: &mut impl Write,
    memory: &mut RequestIdMemory,
    stderr_lines: &mut u32,
    line: &str,
) -> Result<(), String> {
    match parse_frame(line) {
        Ok(request) => {
            if let Err(error) = memory.accept(&request.request_id) {
                let response =
                    AuthorityResponse::error(&request.request_id, error.code(), error.to_string());
                return write_response(stdout, stderr_lines, &response);
            }
            let response = gate.handle(&request.operation, &request.request_id, &request.payload);
            write_response(stdout, stderr_lines, &response)?;
            if gate.shutdown_requested() {
                let bye = AuthorityResponse::ok(
                    &request.request_id,
                    serde_json::json!({"shutdown": true}),
                );
                // Primary response already written; no second frame required.
                let _ = bye;
            }
            Ok(())
        }
        Err(error) => {
            // Recover a request_id when the frame is otherwise well-formed.
            let request_id = extract_request_id(line).unwrap_or_default();
            let response = AuthorityResponse::error(&request_id, error.code(), error.to_string());
            write_response(stdout, stderr_lines, &response)
        }
    }
}

fn extract_request_id(line: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(line.trim()).ok()?;
    value.get("request_id")?.as_str().map(str::to_owned)
}

fn write_response(
    stdout: &mut impl Write,
    stderr_lines: &mut u32,
    response: &AuthorityResponse,
) -> Result<(), String> {
    let line = response.to_json_line();
    writeln!(stdout, "{line}").map_err(|error| format!("stdout write failed: {error}"))?;
    stdout
        .flush()
        .map_err(|error| format!("stdout flush failed: {error}"))?;
    if response.status == crate::gate_protocol::ResponseStatus::Error
        && *stderr_lines < MAX_STDERR_LINES
    {
        if let Some(error) = &response.error {
            let _ = writeln!(
                std::io::stderr(),
                "tethers-gate: {} ({})",
                error.message,
                error.code
            );
            *stderr_lines += 1;
        }
    }
    Ok(())
}

// Silence unused import in some feature combinations.
#[allow(unused_imports)]
use std::io::BufRead as _;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gate_requires_stdio_flag() {
        let result = run_gate(GateCommandArgs {
            stdio: false,
            config: PathBuf::from("/abs/config.json"),
            engine: PathBuf::from("/abs/engine"),
            trail: PathBuf::from("/abs/trail.jsonl"),
            host_data_root: PathBuf::from("/abs/host-data"),
        });
        assert_eq!(result.exit_code, OutcomeStatus::Failed.exit_code());
        assert_eq!(
            result
                .envelope
                .error
                .as_ref()
                .map(|error| error.code.as_str()),
            Some("GATE_STDIO_REQUIRED")
        );
    }

    #[test]
    fn gate_rejects_relative_paths() {
        let result = run_gate(GateCommandArgs {
            stdio: true,
            config: PathBuf::from("relative/config.json"),
            engine: PathBuf::from("relative/engine"),
            trail: PathBuf::from("relative/trail.jsonl"),
            host_data_root: PathBuf::from("relative/host-data"),
        });
        assert_eq!(
            result
                .envelope
                .error
                .as_ref()
                .map(|error| error.code.as_str()),
            Some("GATE_PATH_NOT_ABSOLUTE")
        );
    }
}
