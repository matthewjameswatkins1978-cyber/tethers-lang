//! Structured coding capabilities for the Agent Essentials Phase C provider.
//!
//! This module deliberately does not expose a shell or arbitrary Git command
//! path. The host supplies a strict scope object; every operation revalidates
//! its own arguments and the provider executes only explicit argv vectors.

use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

pub const GIT_STATUS: &str = "git_status";
pub const GIT_DIFF: &str = "git_diff";
pub const GIT_LOG: &str = "git_log";
pub const GIT_SHOW: &str = "git_show";
pub const GIT_BRANCH_LIST: &str = "git_branch_list";
pub const GIT_BRANCH_CURRENT: &str = "git_branch_current";
pub const GIT_ADD: &str = "git_add";
pub const GIT_BRANCH_CREATE: &str = "git_branch_create";
pub const GIT_CHECKOUT: &str = "git_checkout";
pub const GIT_COMMIT: &str = "git_commit";
pub const PROCESS_EXECUTE: &str = "process_execute";
pub const VERIFICATION_RUN: &str = "verification_run";

const MAX_ARGS: usize = 256;
const MAX_ARGUMENT_BYTES: usize = 16 * 1024;
const MAX_BRANCH_BYTES: usize = 255;
const MAX_CHECKS: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodingError {
    pub code: &'static str,
    pub message: String,
}

impl CodingError {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for CodingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for CodingError {}

type Result<T> = std::result::Result<T, CodingError>;

#[derive(Debug, Clone)]
pub struct VerificationCheck {
    pub program: String,
    pub args: Vec<String>,
    pub cwd: String,
    pub environment: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct CodingScope {
    pub repository_root: PathBuf,
    pub process_cwd_root: PathBuf,
    pub allowed_programs: BTreeSet<String>,
    pub max_runtime_ms: u64,
    pub max_output_bytes: u64,
    pub allowed_environment_keys: BTreeSet<String>,
    pub verification_checks: BTreeMap<String, VerificationCheck>,
}

fn object<'a>(
    arguments: &'a Value,
    required: &[&str],
    optional: &[&str],
) -> Result<&'a Map<String, Value>> {
    let value = arguments
        .as_object()
        .ok_or_else(|| CodingError::new("arguments_invalid", "arguments must be an object"))?;
    if required.iter().any(|key| !value.contains_key(*key))
        || value
            .keys()
            .any(|key| !required.contains(&key.as_str()) && !optional.contains(&key.as_str()))
    {
        return Err(CodingError::new(
            "arguments_invalid",
            "unknown or missing coding argument",
        ));
    }
    Ok(value)
}

fn text(value: &Value, field: &'static str, allow_empty: bool) -> Result<String> {
    let value = value.as_str().ok_or_else(|| {
        CodingError::new("arguments_invalid", format!("{field} must be a string"))
    })?;
    if !allow_empty && value.is_empty() {
        return Err(CodingError::new(
            "arguments_invalid",
            format!("{field} must not be empty"),
        ));
    }
    if value.len() > MAX_ARGUMENT_BYTES || value.contains('\0') {
        return Err(CodingError::new(
            "resource_limit",
            format!("{field} is too large or contains NUL"),
        ));
    }
    Ok(value.to_owned())
}

fn bounded_u64(value: &Value, field: &'static str, max: u64) -> Result<u64> {
    let value = value.as_u64().ok_or_else(|| {
        CodingError::new(
            "arguments_invalid",
            format!("{field} must be an unsigned integer"),
        )
    })?;
    if value == 0 || value > max {
        return Err(CodingError::new(
            "resource_limit",
            format!("{field} exceeds its configured bound"),
        ));
    }
    Ok(value)
}

fn array_of_strings(value: &Value, field: &'static str, max: usize) -> Result<Vec<String>> {
    let values = value.as_array().ok_or_else(|| {
        CodingError::new("arguments_invalid", format!("{field} must be an array"))
    })?;
    if values.len() > max {
        return Err(CodingError::new(
            "resource_limit",
            format!("{field} contains too many items"),
        ));
    }
    values
        .iter()
        .map(|value| text(value, field, true))
        .collect()
}

fn safe_relative(raw: &str, field: &'static str, allow_root: bool) -> Result<String> {
    if raw.len() > MAX_ARGUMENT_BYTES
        || raw.contains('\\')
        || raw.contains(':')
        || raw.contains('\0')
        || raw.starts_with('/')
    {
        return Err(CodingError::new(
            "path_invalid",
            format!("{field} must be a bounded relative slash path"),
        ));
    }
    if allow_root && (raw.is_empty() || raw == ".") {
        return Ok(String::new());
    }
    if raw.is_empty()
        || raw
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err(CodingError::new(
            "path_invalid",
            format!("{field} contains an unsafe path segment"),
        ));
    }
    Ok(raw.to_owned())
}

fn git_path(raw: &str, field: &'static str) -> Result<String> {
    let path = safe_relative(raw, field, false)?;
    if path
        .chars()
        .any(|character| matches!(character, '*' | '?' | '[' | ']'))
    {
        return Err(CodingError::new(
            "path_invalid",
            format!("{field} must identify one literal path; glob syntax is not accepted"),
        ));
    }
    Ok(path)
}

fn canonical_directory(value: &Value, field: &'static str) -> Result<PathBuf> {
    let raw = text(value, field, false)?;
    let path = PathBuf::from(raw);
    if !path.is_absolute() {
        return Err(CodingError::new(
            "scope_invalid",
            format!("{field} must be absolute"),
        ));
    }
    let canonical = fs::canonicalize(&path).map_err(|error| {
        CodingError::new("scope_invalid", format!("{field} is unavailable: {error}"))
    })?;
    if !canonical.is_dir() {
        return Err(CodingError::new(
            "scope_invalid",
            format!("{field} must be a directory"),
        ));
    }
    Ok(canonical)
}

fn string_set(
    value: &Value,
    field: &'static str,
    require_nonempty: bool,
) -> Result<BTreeSet<String>> {
    let values = array_of_strings(value, field, MAX_CHECKS)?;
    if require_nonempty && values.is_empty() {
        return Err(CodingError::new(
            "scope_invalid",
            format!("{field} must contain at least one item"),
        ));
    }
    if values.iter().any(|value| value.is_empty()) {
        return Err(CodingError::new(
            "scope_invalid",
            format!("{field} contains an empty item"),
        ));
    }
    Ok(values.into_iter().collect())
}

fn parse_check(name: &str, value: &Value) -> Result<VerificationCheck> {
    let map = value.as_object().ok_or_else(|| {
        CodingError::new(
            "scope_invalid",
            format!("verification check '{name}' must be an object"),
        )
    })?;
    let allowed = ["program", "args", "cwd", "environment"];
    if map.keys().any(|key| !allowed.contains(&key.as_str())) {
        return Err(CodingError::new(
            "scope_invalid",
            format!("verification check '{name}' has an unknown field"),
        ));
    }
    let program = text(
        map.get("program").ok_or_else(|| {
            CodingError::new("scope_invalid", format!("check '{name}' lacks program"))
        })?,
        "verification program",
        false,
    )?;
    let args = array_of_strings(
        map.get("args").ok_or_else(|| {
            CodingError::new("scope_invalid", format!("check '{name}' lacks args"))
        })?,
        "verification args",
        MAX_ARGS,
    )?;
    let cwd = text(
        map.get("cwd").ok_or_else(|| {
            CodingError::new("scope_invalid", format!("check '{name}' lacks cwd"))
        })?,
        "verification cwd",
        false,
    )?;
    safe_relative(&cwd, "verification cwd", true)?;
    let environment = match map.get("environment") {
        None => BTreeMap::new(),
        Some(value) => {
            let environment = value.as_object().ok_or_else(|| {
                CodingError::new(
                    "scope_invalid",
                    "verification environment must be an object",
                )
            })?;
            if environment.len() > MAX_CHECKS {
                return Err(CodingError::new(
                    "resource_limit",
                    "verification environment contains too many keys",
                ));
            }
            environment
                .iter()
                .map(|(key, value)| {
                    Ok((
                        key.clone(),
                        text(value, "verification environment value", true)?,
                    ))
                })
                .collect::<Result<BTreeMap<_, _>>>()?
        }
    };
    Ok(VerificationCheck {
        program,
        args,
        cwd,
        environment,
    })
}

impl CodingScope {
    pub fn from_json(value: &Value) -> Result<Self> {
        let map = value
            .as_object()
            .ok_or_else(|| CodingError::new("scope_invalid", "coding scope must be an object"))?;
        let allowed = [
            "repository_root",
            "process_cwd_root",
            "allowed_programs",
            "max_runtime_ms",
            "max_output_bytes",
            "allowed_environment_keys",
            "verification_checks",
        ];
        if map.keys().any(|key| !allowed.contains(&key.as_str())) {
            return Err(CodingError::new(
                "scope_invalid",
                "coding scope contains an unknown field",
            ));
        }
        let repository_root = canonical_directory(
            map.get("repository_root")
                .ok_or_else(|| CodingError::new("scope_invalid", "repository_root is required"))?,
            "repository_root",
        )?;
        let process_cwd_root = canonical_directory(
            map.get("process_cwd_root")
                .ok_or_else(|| CodingError::new("scope_invalid", "process_cwd_root is required"))?,
            "process_cwd_root",
        )?;
        let allowed_programs = string_set(
            map.get("allowed_programs")
                .ok_or_else(|| CodingError::new("scope_invalid", "allowed_programs is required"))?,
            "allowed_programs",
            true,
        )?;
        let max_runtime_ms = bounded_u64(
            map.get("max_runtime_ms")
                .ok_or_else(|| CodingError::new("scope_invalid", "max_runtime_ms is required"))?,
            "max_runtime_ms",
            120_000,
        )?;
        let max_output_bytes = bounded_u64(
            map.get("max_output_bytes")
                .ok_or_else(|| CodingError::new("scope_invalid", "max_output_bytes is required"))?,
            "max_output_bytes",
            16 * 1024 * 1024,
        )?;
        let allowed_environment_keys = string_set(
            map.get("allowed_environment_keys").ok_or_else(|| {
                CodingError::new("scope_invalid", "allowed_environment_keys is required")
            })?,
            "allowed_environment_keys",
            false,
        )?;
        let checks = map
            .get("verification_checks")
            .ok_or_else(|| CodingError::new("scope_invalid", "verification_checks is required"))?
            .as_object()
            .ok_or_else(|| {
                CodingError::new("scope_invalid", "verification_checks must be an object")
            })?;
        if checks.len() > MAX_CHECKS {
            return Err(CodingError::new(
                "resource_limit",
                "verification_checks contains too many entries",
            ));
        }
        let verification_checks = checks
            .iter()
            .map(|(name, value)| {
                if name.is_empty() || name.len() > 128 {
                    return Err(CodingError::new(
                        "scope_invalid",
                        "verification check name is invalid",
                    ));
                }
                Ok((name.clone(), parse_check(name, value)?))
            })
            .collect::<Result<BTreeMap<_, _>>>()?;
        for check in verification_checks.values() {
            if !allowed_programs.contains(&check.program) {
                return Err(CodingError::new(
                    "scope_invalid",
                    format!(
                        "verification program '{}' is not allow-listed",
                        check.program
                    ),
                ));
            }
            for key in check.environment.keys() {
                if !allowed_environment_keys.contains(key) {
                    return Err(CodingError::new(
                        "scope_invalid",
                        format!("verification environment key '{key}' is not allow-listed"),
                    ));
                }
            }
        }
        Ok(Self {
            repository_root,
            process_cwd_root,
            allowed_programs,
            max_runtime_ms,
            max_output_bytes,
            allowed_environment_keys,
            verification_checks,
        })
    }
}

fn resolve_relative(root: &Path, raw: &str, field: &'static str) -> Result<PathBuf> {
    let relative = safe_relative(raw, field, true)?;
    let candidate = if relative.is_empty() {
        root.to_path_buf()
    } else {
        root.join(relative.replace('/', std::path::MAIN_SEPARATOR_STR))
    };
    let canonical = fs::canonicalize(&candidate)
        .map_err(|error| CodingError::new("path_unavailable", format!("{field}: {error}")))?;
    if !canonical.starts_with(root) {
        return Err(CodingError::new(
            "scope_violation",
            format!("{field} is outside its approved root"),
        ));
    }
    Ok(canonical)
}

fn program_allowed(scope: &CodingScope, program: &str) -> bool {
    scope.allowed_programs.contains(program)
        || scope
            .allowed_programs
            .iter()
            .any(|allowed| cfg!(windows) && allowed.eq_ignore_ascii_case(program))
}

#[derive(Debug)]
struct ProcessResult {
    exit_code: Option<i32>,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    stdout_truncated: bool,
    stderr_truncated: bool,
    timed_out: bool,
    duration_ms: u64,
}

fn drain<R: Read + Send + 'static>(
    mut reader: R,
    limit: usize,
) -> thread::JoinHandle<(Vec<u8>, bool)> {
    thread::spawn(move || {
        let mut output = Vec::with_capacity(limit.min(8192));
        let mut truncated = false;
        let mut buffer = [0_u8; 8192];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(count) => {
                    if output.len() < limit {
                        let remaining = limit - output.len();
                        output.extend_from_slice(&buffer[..count.min(remaining)]);
                    }
                    if output.len() >= limit && count > limit.saturating_sub(output.len()) {
                        truncated = true;
                    }
                }
                Err(_) => {
                    truncated = true;
                    break;
                }
            }
        }
        (output, truncated)
    })
}

fn run_argv_in_dir(
    scope: &CodingScope,
    cwd: &Path,
    program: &str,
    args: &[String],
    timeout_ms: u64,
    max_output_bytes: u64,
    environment: &BTreeMap<String, String>,
) -> Result<ProcessResult> {
    if !program_allowed(scope, program) {
        return Err(CodingError::new(
            "program_not_allowed",
            format!("program '{program}' is not allow-listed"),
        ));
    }
    if args.len() > MAX_ARGS {
        return Err(CodingError::new(
            "resource_limit",
            "too many process arguments",
        ));
    }
    if !cwd.is_absolute()
        || !cwd.starts_with(&scope.process_cwd_root) && !cwd.starts_with(&scope.repository_root)
    {
        return Err(CodingError::new(
            "scope_violation",
            "process working directory is outside its approved root",
        ));
    }
    let timeout_ms = timeout_ms.min(scope.max_runtime_ms);
    if timeout_ms == 0 {
        return Err(CodingError::new(
            "resource_limit",
            "timeout_ms must not be zero",
        ));
    }
    let max_output_bytes = max_output_bytes.min(scope.max_output_bytes);
    if max_output_bytes == 0 {
        return Err(CodingError::new(
            "resource_limit",
            "max_output_bytes must not be zero",
        ));
    }
    for (key, value) in environment {
        if key.is_empty()
            || key.contains('=')
            || key.contains('\0')
            || !scope.allowed_environment_keys.contains(key)
        {
            return Err(CodingError::new(
                "environment_not_allowed",
                format!("environment key '{key}' is not allow-listed"),
            ));
        }
        if value.contains('\0') {
            return Err(CodingError::new(
                "environment_invalid",
                format!("environment value for '{key}' contains NUL"),
            ));
        }
    }

    let mut command = Command::new(program);
    command
        .args(args)
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env_clear();
    for key in &scope.allowed_environment_keys {
        if let Ok(value) = env::var(key) {
            command.env(key, value);
        }
    }
    for (key, value) in environment {
        command.env(key, value);
    }
    let started = Instant::now();
    let mut child: Child = command
        .spawn()
        .map_err(|error| CodingError::new("process_spawn_failed", error.to_string()))?;
    let stdout = drain(
        child
            .stdout
            .take()
            .ok_or_else(|| CodingError::new("process_spawn_failed", "stdout pipe unavailable"))?,
        max_output_bytes as usize,
    );
    let stderr = drain(
        child
            .stderr
            .take()
            .ok_or_else(|| CodingError::new("process_spawn_failed", "stderr pipe unavailable"))?,
        max_output_bytes as usize,
    );
    let deadline = started + Duration::from_millis(timeout_ms);
    let mut timed_out = false;
    let status = loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|error| CodingError::new("process_wait_failed", error.to_string()))?
        {
            break status;
        }
        if Instant::now() >= deadline {
            timed_out = true;
            child
                .kill()
                .map_err(|error| CodingError::new("process_kill_failed", error.to_string()))?;
            break child
                .wait()
                .map_err(|error| CodingError::new("process_wait_failed", error.to_string()))?;
        }
        thread::sleep(Duration::from_millis(5));
    };
    let (stdout, stdout_truncated) = stdout
        .join()
        .map_err(|_| CodingError::new("process_output_failed", "stdout reader panicked"))?;
    let (stderr, stderr_truncated) = stderr
        .join()
        .map_err(|_| CodingError::new("process_output_failed", "stderr reader panicked"))?;
    Ok(ProcessResult {
        exit_code: status.code(),
        stdout,
        stderr,
        stdout_truncated,
        stderr_truncated,
        timed_out,
        duration_ms: started.elapsed().as_millis().min(u64::MAX as u128) as u64,
    })
}

fn run_argv(
    scope: &CodingScope,
    program: &str,
    args: &[String],
    cwd: &str,
    timeout_ms: u64,
    max_output_bytes: u64,
    environment: &BTreeMap<String, String>,
) -> Result<ProcessResult> {
    let cwd = resolve_relative(&scope.process_cwd_root, cwd, "cwd")?;
    run_argv_in_dir(
        scope,
        &cwd,
        program,
        args,
        timeout_ms,
        max_output_bytes,
        environment,
    )
}

fn lossy(bytes: &[u8]) -> (String, bool) {
    match String::from_utf8(bytes.to_owned()) {
        Ok(value) => (value, true),
        Err(error) => (
            String::from_utf8_lossy(error.as_bytes()).into_owned(),
            false,
        ),
    }
}

fn process_value(result: ProcessResult, program: &str, cwd: &str) -> Value {
    let (stdout, stdout_utf8) = lossy(&result.stdout);
    let (stderr, stderr_utf8) = lossy(&result.stderr);
    json!({
        "program": program,
        "cwd": cwd,
        "exit_code": result.exit_code,
        "stdout": stdout,
        "stderr": stderr,
        "stdout_utf8": stdout_utf8,
        "stderr_utf8": stderr_utf8,
        "stdout_truncated": result.stdout_truncated,
        "stderr_truncated": result.stderr_truncated,
        "timed_out": result.timed_out,
        "duration_ms": result.duration_ms
    })
}

fn git(scope: &CodingScope, args: Vec<String>, max_output_bytes: u64) -> Result<ProcessResult> {
    let repository = run_argv_in_dir(
        scope,
        &scope.repository_root,
        "git",
        &["rev-parse".to_owned(), "--show-toplevel".to_owned()],
        scope.max_runtime_ms,
        4096,
        &BTreeMap::new(),
    )?;
    if repository.timed_out || repository.exit_code != Some(0) {
        return Err(CodingError::new(
            "repository_unavailable",
            "repository_root is not a Git worktree",
        ));
    }
    let discovered = String::from_utf8(repository.stdout)
        .map_err(|_| CodingError::new("repository_invalid", "Git repository root is not UTF-8"))?;
    let discovered = fs::canonicalize(discovered.trim()).map_err(|error| {
        CodingError::new(
            "repository_invalid",
            format!("Git repository root is unavailable: {error}"),
        )
    })?;
    if discovered != scope.repository_root {
        return Err(CodingError::new(
            "scope_violation",
            "repository_root is not the actual Git worktree root",
        ));
    }
    run_argv_in_dir(
        scope,
        &scope.repository_root,
        "git",
        &args,
        scope.max_runtime_ms,
        max_output_bytes,
        &BTreeMap::new(),
    )
}

fn require_git_success(result: &ProcessResult) -> Result<()> {
    if result.timed_out {
        return Err(CodingError::new("git_timeout", "Git operation timed out"));
    }
    if result.exit_code != Some(0) {
        let (stderr, _) = lossy(&result.stderr);
        return Err(CodingError::new(
            "git_failed",
            if stderr.trim().is_empty() {
                "Git operation failed".to_owned()
            } else {
                stderr.trim().to_owned()
            },
        ));
    }
    if result.stdout_truncated || result.stderr_truncated {
        return Err(CodingError::new(
            "git_output_truncated",
            "Git output exceeded the operation bound",
        ));
    }
    Ok(())
}

fn git_revision(value: &Value) -> Result<String> {
    let revision = text(value, "revision", false)?;
    if revision.starts_with('-')
        || revision.chars().any(|character| {
            character.is_whitespace() || matches!(character, ';' | '|' | '&' | '`' | '$' | '\0')
        })
        || revision.contains("..")
    {
        return Err(CodingError::new(
            "revision_invalid",
            "revision contains unsupported or ambiguous syntax",
        ));
    }
    Ok(revision)
}

fn branch_name(value: &Value) -> Result<String> {
    let name = text(value, "branch", false)?;
    if name.len() > MAX_BRANCH_BYTES
        || name.starts_with('-')
        || name.starts_with('/')
        || name.ends_with('/')
        || name.starts_with('.')
        || name.ends_with('.')
        || name.contains("..")
        || name.contains("//")
        || name.contains("@{")
        || name.chars().any(|character| {
            character.is_whitespace()
                || character.is_control()
                || matches!(character, '~' | '^' | ':' | '?' | '*' | '[' | '\\')
        })
    {
        return Err(CodingError::new("branch_invalid", "branch name is unsafe"));
    }
    Ok(name)
}

pub fn process_execute(scope: &CodingScope, arguments: &Value) -> Result<Value> {
    let args = object(
        arguments,
        &["program", "args"],
        &["cwd", "timeout_ms", "max_output_bytes", "environment"],
    )?;
    let program = text(args.get("program").unwrap(), "program", false)?;
    let process_args = array_of_strings(args.get("args").unwrap(), "args", MAX_ARGS)?;
    let cwd = match args.get("cwd") {
        Some(value) => text(value, "cwd", false)?,
        None => ".".to_owned(),
    };
    safe_relative(&cwd, "cwd", true)?;
    let timeout_ms = match args.get("timeout_ms") {
        Some(value) => bounded_u64(value, "timeout_ms", scope.max_runtime_ms)?,
        None => scope.max_runtime_ms,
    };
    let max_output_bytes = match args.get("max_output_bytes") {
        Some(value) => bounded_u64(value, "max_output_bytes", scope.max_output_bytes)?,
        None => scope.max_output_bytes,
    };
    let environment = match args.get("environment") {
        None => BTreeMap::new(),
        Some(value) => value
            .as_object()
            .ok_or_else(|| CodingError::new("arguments_invalid", "environment must be an object"))?
            .iter()
            .map(|(key, value)| Ok((key.clone(), text(value, "environment value", true)?)))
            .collect::<Result<BTreeMap<_, _>>>()?,
    };
    let result = run_argv(
        scope,
        &program,
        &process_args,
        &cwd,
        timeout_ms,
        max_output_bytes,
        &environment,
    )?;
    Ok(process_value(result, &program, &cwd))
}

pub fn verification_run(scope: &CodingScope, arguments: &Value) -> Result<Value> {
    let args = object(arguments, &["check"], &[])?;
    let name = text(args.get("check").unwrap(), "check", false)?;
    let check = scope.verification_checks.get(&name).ok_or_else(|| {
        CodingError::new(
            "verification_not_configured",
            format!("verification check '{name}' is not configured"),
        )
    })?;
    let result = run_argv(
        scope,
        &check.program,
        &check.args,
        &check.cwd,
        scope.max_runtime_ms,
        scope.max_output_bytes,
        &check.environment,
    )?;
    let (stdout, stdout_utf8) = lossy(&result.stdout);
    let (stderr, stderr_utf8) = lossy(&result.stderr);
    Ok(json!({
        "check": name,
        "passed": !result.timed_out && result.exit_code == Some(0),
        "exit_code": result.exit_code,
        "duration_ms": result.duration_ms,
        "stdout": stdout,
        "stderr": stderr,
        "stdout_utf8": stdout_utf8,
        "stderr_utf8": stderr_utf8,
        "stdout_truncated": result.stdout_truncated,
        "stderr_truncated": result.stderr_truncated,
        "timed_out": result.timed_out
    }))
}

pub fn git_status(scope: &CodingScope, _arguments: &Value) -> Result<Value> {
    let result = git(
        scope,
        vec![
            "status".into(),
            "--porcelain=v1".into(),
            "-z".into(),
            "--branch".into(),
        ],
        scope.max_output_bytes,
    )?;
    require_git_success(&result)?;
    let text = String::from_utf8(result.stdout)
        .map_err(|_| CodingError::new("git_invalid_utf8", "Git status contains invalid UTF-8"))?;
    let mut branch = None;
    let mut entries = Vec::new();
    let tokens: Vec<&str> = text.split('\0').filter(|token| !token.is_empty()).collect();
    for token in tokens {
        if let Some(value) = token.strip_prefix("## ") {
            let value = value
                .strip_prefix("No commits yet on ")
                .unwrap_or(value)
                .split("...")
                .next()
                .unwrap_or(value)
                .split(" [")
                .next()
                .unwrap_or(value);
            if !value.starts_with("(HEAD detached") {
                branch = Some(value.to_owned());
            }
            continue;
        }
        if token.len() < 4 || token.as_bytes()[2] != b' ' {
            return Err(CodingError::new(
                "git_status_invalid",
                "Git status record is malformed",
            ));
        }
        entries.push(json!({
            "index_status": token[0..1].to_owned(),
            "worktree_status": token[1..2].to_owned(),
            "path": token[3..].to_owned()
        }));
    }
    Ok(json!({"branch": branch, "clean": entries.is_empty(), "entries": entries}))
}

pub fn git_diff(scope: &CodingScope, arguments: &Value) -> Result<Value> {
    let args = object(arguments, &["staged", "max_bytes"], &["path"])?;
    let staged = args
        .get("staged")
        .unwrap()
        .as_bool()
        .ok_or_else(|| CodingError::new("arguments_invalid", "staged must be a boolean"))?;
    let max_bytes = bounded_u64(
        args.get("max_bytes").unwrap(),
        "max_bytes",
        scope.max_output_bytes,
    )?;
    let path = match args.get("path") {
        None => None,
        Some(value) => Some(git_path(&text(value, "path", false)?, "path")?),
    };
    let mut command = vec!["diff".to_owned()];
    if staged {
        command.push("--cached".to_owned());
    }
    command.extend(["--no-ext-diff".into(), "--no-color".into(), "--".into()]);
    if let Some(path) = &path {
        command.push(path.clone());
    }
    let result = git(scope, command, max_bytes)?;
    require_git_success(&result)?;
    let (diff, utf8) = lossy(&result.stdout);
    Ok(
        json!({"staged": staged, "path": path, "diff": diff, "utf8": utf8, "truncated": result.stdout_truncated}),
    )
}

pub fn git_log(scope: &CodingScope, arguments: &Value) -> Result<Value> {
    let args = object(arguments, &["max_count"], &[])?;
    let max_count = bounded_u64(args.get("max_count").unwrap(), "max_count", 1000)?;
    let result = git(
        scope,
        vec![
            "log".into(),
            "--no-decorate".into(),
            "--format=%H%x00%an%x00%aI%x00%s%x00".into(),
            format!("--max-count={max_count}"),
        ],
        scope.max_output_bytes,
    )?;
    require_git_success(&result)?;
    let text = String::from_utf8(result.stdout)
        .map_err(|_| CodingError::new("git_invalid_utf8", "Git log contains invalid UTF-8"))?;
    let fields: Vec<&str> = text.split('\0').filter(|field| !field.is_empty()).collect();
    if fields.len() % 4 != 0 {
        return Err(CodingError::new(
            "git_log_invalid",
            "Git log record is malformed",
        ));
    }
    let commits = fields
        .chunks_exact(4)
        .map(|fields| json!({"commit":fields[0],"author":fields[1],"timestamp":fields[2],"subject":fields[3]}))
        .collect::<Vec<_>>();
    Ok(json!({"commits": commits}))
}

pub fn git_show(scope: &CodingScope, arguments: &Value) -> Result<Value> {
    let args = object(arguments, &["revision", "max_bytes"], &[])?;
    let revision = git_revision(args.get("revision").unwrap())?;
    let max_bytes = bounded_u64(
        args.get("max_bytes").unwrap(),
        "max_bytes",
        scope.max_output_bytes,
    )?;
    let result = git(
        scope,
        vec![
            "show".into(),
            "--no-ext-diff".into(),
            "--no-color".into(),
            "--end-of-options".into(),
            revision.clone(),
        ],
        max_bytes,
    )?;
    require_git_success(&result)?;
    let (content, utf8) = lossy(&result.stdout);
    Ok(
        json!({"revision": revision, "content": content, "utf8": utf8, "truncated": result.stdout_truncated}),
    )
}

pub fn git_branch_list(scope: &CodingScope, _arguments: &Value) -> Result<Value> {
    let result = git(
        scope,
        vec![
            "for-each-ref".into(),
            "--format=%(refname:short)%x00%(objectname)%x00%(upstream:short)%x00".into(),
            "refs/heads".into(),
            "refs/remotes".into(),
        ],
        scope.max_output_bytes,
    )?;
    require_git_success(&result)?;
    let text = String::from_utf8(result.stdout).map_err(|_| {
        CodingError::new("git_invalid_utf8", "Git branch list contains invalid UTF-8")
    })?;
    let fields: Vec<&str> = text.split('\0').filter(|field| !field.is_empty()).collect();
    if fields.len() % 3 != 0 {
        return Err(CodingError::new(
            "git_branch_list_invalid",
            "Git branch record is malformed",
        ));
    }
    let branches = fields
        .chunks_exact(3)
        .map(|fields| json!({"name":fields[0],"commit":fields[1],"upstream":if fields[2].is_empty(){Value::Null}else{Value::String(fields[2].to_owned())}}))
        .collect::<Vec<_>>();
    Ok(json!({"branches": branches}))
}

pub fn git_branch_current(scope: &CodingScope, _arguments: &Value) -> Result<Value> {
    let result = git(scope, vec!["branch".into(), "--show-current".into()], 4096)?;
    require_git_success(&result)?;
    let current = String::from_utf8(result.stdout)
        .map_err(|_| {
            CodingError::new("git_invalid_utf8", "Git branch name contains invalid UTF-8")
        })?
        .trim()
        .to_owned();
    Ok(json!({"branch": if current.is_empty(){Value::Null}else{Value::String(current)}}))
}

pub fn git_add(scope: &CodingScope, arguments: &Value) -> Result<Value> {
    let args = object(arguments, &["paths"], &[])?;
    let paths = array_of_strings(args.get("paths").unwrap(), "paths", MAX_ARGS)?;
    if paths.is_empty() {
        return Err(CodingError::new(
            "arguments_invalid",
            "paths must not be empty",
        ));
    }
    let paths = paths
        .into_iter()
        .map(|path| git_path(&path, "path"))
        .collect::<Result<Vec<_>>>()?;
    let mut command = vec!["add".into(), "--".into()];
    command.extend(paths.iter().cloned());
    let result = git(scope, command, scope.max_output_bytes)?;
    require_git_success(&result)?;
    Ok(json!({"added_paths": paths}))
}

pub fn git_branch_create(scope: &CodingScope, arguments: &Value) -> Result<Value> {
    let args = object(arguments, &["branch"], &["start_point"])?;
    let branch = branch_name(args.get("branch").unwrap())?;
    let mut command = vec!["branch".into(), branch.clone()];
    let start_point = match args.get("start_point") {
        None => None,
        Some(value) => {
            let revision = git_revision(value)?;
            command.push(revision.clone());
            Some(revision)
        }
    };
    let result = git(scope, command, scope.max_output_bytes)?;
    require_git_success(&result)?;
    Ok(json!({"created": branch, "start_point": start_point}))
}

pub fn git_checkout(scope: &CodingScope, arguments: &Value) -> Result<Value> {
    let args = object(arguments, &["branch"], &[])?;
    let branch = branch_name(args.get("branch").unwrap())?;
    let result = git(
        scope,
        vec!["checkout".into(), "--quiet".into(), branch.clone()],
        scope.max_output_bytes,
    )?;
    require_git_success(&result)?;
    Ok(json!({"branch": branch, "checked_out": true}))
}

pub fn git_commit(scope: &CodingScope, arguments: &Value) -> Result<Value> {
    let args = object(arguments, &["message"], &[])?;
    let message = text(args.get("message").unwrap(), "message", false)?;
    let result = git(
        scope,
        vec!["commit".into(), "-m".into(), message],
        scope.max_output_bytes,
    )?;
    require_git_success(&result)?;
    let head = git(
        scope,
        vec!["rev-parse".into(), "--verify".into(), "HEAD".into()],
        4096,
    )?;
    require_git_success(&head)?;
    let commit = String::from_utf8(head.stdout)
        .map_err(|_| CodingError::new("git_invalid_utf8", "commit id contains invalid UTF-8"))?
        .trim()
        .to_owned();
    Ok(json!({"committed": true, "commit": commit}))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scope(root: &Path) -> CodingScope {
        let root = fs::canonicalize(root).unwrap();
        CodingScope {
            repository_root: root.clone(),
            process_cwd_root: root,
            allowed_programs: ["git".to_owned(), "cmd".to_owned()].into_iter().collect(),
            max_runtime_ms: 5000,
            max_output_bytes: 4096,
            allowed_environment_keys: ["PATH".to_owned()].into_iter().collect(),
            verification_checks: BTreeMap::new(),
        }
    }

    fn fixture() -> PathBuf {
        let root = env::temp_dir().join(format!("tethers-agent-coding-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn argv_process_does_not_allow_program_or_cwd_escape() {
        let root = fixture();
        let scope = scope(&root);
        let denied = process_execute(&scope, &json!({"program":"powershell","args":[]}));
        assert_eq!(denied.unwrap_err().code, "program_not_allowed");
        let escaped = process_execute(
            &scope,
            &json!({"program":"cmd","args":["/c","echo","ok"],"cwd":".."}),
        );
        assert_eq!(escaped.unwrap_err().code, "path_invalid");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn verification_rejects_unknown_check_without_command_input() {
        let root = fixture();
        let scope = scope(&root);
        let error = verification_run(&scope, &json!({"check":"tests"})).unwrap_err();
        assert_eq!(error.code, "verification_not_configured");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn branch_and_revision_validation_refuse_flag_injection() {
        assert_eq!(
            branch_name(&json!("--delete")).unwrap_err().code,
            "branch_invalid"
        );
        assert_eq!(
            git_revision(&json!("--help")).unwrap_err().code,
            "revision_invalid"
        );
        assert_eq!(
            git_revision(&json!("HEAD..HEAD")).unwrap_err().code,
            "revision_invalid"
        );
        assert_eq!(
            git_path("src/*.rs", "path").unwrap_err().code,
            "path_invalid"
        );
    }

    #[test]
    fn process_returns_structured_bounded_result() {
        let root = fixture();
        let scope = scope(&root);
        let result =
            process_execute(&scope, &json!({"program":"git","args":["--version"]})).unwrap();
        assert_eq!(result["exit_code"], 0);
        assert_eq!(result["timed_out"], false);
        assert_eq!(result["stdout_utf8"], true);
        assert_eq!(result["stdout_truncated"], false);
        assert!(result["stdout"]
            .as_str()
            .unwrap()
            .starts_with("git version "));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn git_refuses_a_scope_inside_a_parent_repository() {
        let outer = fixture();
        let nested = outer.join("nested");
        fs::create_dir_all(&nested).unwrap();
        let status = Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(&outer)
            .status()
            .unwrap();
        assert!(status.success());
        let scope = scope(&nested);
        let error = git_status(&scope, &json!({})).unwrap_err();
        assert_eq!(error.code, "scope_violation");
        fs::remove_dir_all(outer).unwrap();
    }
}
