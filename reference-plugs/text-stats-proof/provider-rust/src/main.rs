use std::env;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Component, Path, PathBuf};

use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

const HARD_MAX_BYTES: u64 = 8 * 1024 * 1024;
const PROTOCOL_VERSION: &str = "2025-11-25";
const SERVER_NAME: &str = "tethers-text-stats-provider";
const TOOL_NAME: &str = "text_stats";

fn main() -> std::process::ExitCode {
    let scope = match resolve_scope() {
        Ok(scope) => scope,
        Err(error) => {
            eprintln!("{SERVER_NAME}: configuration refused: {error}");
            return std::process::ExitCode::from(1);
        }
    };
    if let Err(error) = run_mcp_loop(scope) {
        eprintln!("{SERVER_NAME}: stdio failure: {error}");
        return std::process::ExitCode::from(1);
    }
    std::process::ExitCode::SUCCESS
}

/// Operational scope carries the generic host's constraint on *where* and
/// *how much* this provider may read. The host validates and binds the scope;
/// the provider interprets its own application-specific meaning.
#[derive(Debug)]
struct Scope {
    query_root: PathBuf,
    max_bytes: u64,
}

#[derive(Debug, PartialEq, Eq)]
enum ScopeError {
    NoScope,
    Malformed(String),
    MaxBytesAboveHardLimit(u64),
    RootNotDirectory(PathBuf),
}

impl std::fmt::Display for ScopeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScopeError::NoScope => write!(
                f,
                "no operational scope: TETHERS_OPERATIONAL_SCOPE_JSON absent and conformance mode inactive"
            ),
            ScopeError::Malformed(detail) => write!(f, "malformed operational scope: {detail}"),
            ScopeError::MaxBytesAboveHardLimit(bytes) => {
                write!(f, "max_bytes {bytes} exceeds the {HARD_MAX_BYTES}-byte hard maximum")
            }
            ScopeError::RootNotDirectory(path) => {
                write!(f, "query_root is not a directory: {}", path.display())
            }
        }
    }
}

fn resolve_scope() -> Result<Scope, ScopeError> {
    match env::var("TETHERS_OPERATIONAL_SCOPE_JSON") {
        Ok(raw) => parse_scope(&raw),
        Err(_) if env::var("TETHERS_CONFORMANCE").as_deref() == Ok("1") => {
            let temp = env::var("TEMP")
                .or_else(|_| env::var("TMP"))
                .map_err(|_| ScopeError::NoScope)?;
            let query_root = PathBuf::from(temp);
            if !query_root.is_dir() {
                return Err(ScopeError::RootNotDirectory(query_root));
            }
            Ok(Scope {
                query_root,
                max_bytes: HARD_MAX_BYTES,
            })
        }
        Err(_) => Err(ScopeError::NoScope),
    }
}

fn parse_scope(raw: &str) -> Result<Scope, ScopeError> {
    let value: Value =
        serde_json::from_str(raw).map_err(|error| ScopeError::Malformed(error.to_string()))?;
    let query_root = value
        .get("query_root")
        .and_then(Value::as_str)
        .ok_or_else(|| ScopeError::Malformed("query_root must be a string".to_string()))?;
    let max_bytes = value
        .get("max_bytes")
        .and_then(Value::as_u64)
        .ok_or_else(|| ScopeError::Malformed("max_bytes must be an integer".to_string()))?;
    if max_bytes > HARD_MAX_BYTES {
        return Err(ScopeError::MaxBytesAboveHardLimit(max_bytes));
    }
    let query_root = PathBuf::from(query_root);
    if !query_root.is_dir() {
        return Err(ScopeError::RootNotDirectory(query_root));
    }
    Ok(Scope {
        query_root,
        max_bytes,
    })
}

#[derive(Debug, PartialEq, Eq)]
struct Stats {
    path: String,
    size_bytes: u64,
    sha256: String,
    line_count: usize,
    word_count: usize,
    character_count: usize,
}

#[derive(Debug)]
enum StatsError {
    AbsolutePath(String),
    Traversal(String),
    NotRegularFile,
    TooLarge { size_bytes: u64, max_bytes: u64 },
    InvalidUtf8,
    ReadFailed(io::Error),
}

impl std::fmt::Display for StatsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StatsError::AbsolutePath(path) => {
                write!(f, "path must be relative to query_root: {path:?}")
            }
            StatsError::Traversal(path) => write!(f, "path escapes query_root: {path:?}"),
            StatsError::NotRegularFile => write!(f, "path is not a regular file"),
            StatsError::TooLarge {
                size_bytes,
                max_bytes,
            } => {
                write!(
                    f,
                    "file is {size_bytes} bytes but the approved maximum is {max_bytes}"
                )
            }
            StatsError::InvalidUtf8 => write!(f, "file is not valid UTF-8"),
            StatsError::ReadFailed(error) => write!(f, "read failure: {error}"),
        }
    }
}

/// Reject absolute/rooted and parent-traversal paths before any filesystem
/// access. `..` is refused outright rather than resolved, so a relative path
/// can never be expressed as an escape.
fn validate_relative_path(rel: &str) -> Result<(), StatsError> {
    let path = Path::new(rel);
    for component in path.components() {
        match component {
            Component::Prefix(_) | Component::RootDir => {
                return Err(StatsError::AbsolutePath(rel.to_string()));
            }
            Component::ParentDir => return Err(StatsError::Traversal(rel.to_string())),
            Component::CurDir | Component::Normal(_) => {}
        }
    }
    Ok(())
}

fn compute_stats(root: &Path, rel: &str, max_bytes: u64) -> Result<Stats, StatsError> {
    validate_relative_path(rel)?;
    let root_canon = fs::canonicalize(root).map_err(StatsError::ReadFailed)?;
    let candidate = root.join(rel);
    let candidate_canon = fs::canonicalize(&candidate).map_err(StatsError::ReadFailed)?;
    if !candidate_canon.starts_with(&root_canon) {
        return Err(StatsError::Traversal(rel.to_string()));
    }
    let metadata = fs::metadata(&candidate_canon).map_err(StatsError::ReadFailed)?;
    if !metadata.is_file() {
        return Err(StatsError::NotRegularFile);
    }
    let size_bytes = metadata.len();
    if size_bytes > max_bytes {
        return Err(StatsError::TooLarge {
            size_bytes,
            max_bytes,
        });
    }
    let bytes = fs::read(&candidate_canon).map_err(StatsError::ReadFailed)?;
    let text = String::from_utf8(bytes).map_err(|_| StatsError::InvalidUtf8)?;
    Ok(Stats {
        path: rel.to_string(),
        size_bytes,
        sha256: format!("sha256:{}", hex_lower(&Sha256::digest(text.as_bytes()))),
        line_count: logical_line_count(&text),
        word_count: text.split_whitespace().count(),
        character_count: text.chars().count(),
    })
}

/// Logical lines are separated by `\n`. A single trailing `\n` does not add a
/// phantom final empty line, and an empty file has zero lines.
fn logical_line_count(text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }
    let mut lines = text.split('\n').count();
    if text.ends_with('\n') {
        lines -= 1;
    }
    lines
}

fn hex_lower(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(out, "{byte:02x}");
    }
    out
}

fn run_mcp_loop(scope: Scope) -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let message: Value = match serde_json::from_str(&line) {
            Ok(message) => message,
            Err(_) => continue,
        };
        let request: RpcRequest = match serde_json::from_value(message) {
            Ok(request) => request,
            Err(_) => continue,
        };
        let Some(id) = request.id.clone() else {
            continue; // notification: no response on stdout
        };
        if request.jsonrpc != "2.0" {
            write_response(&mut out, &id, Err(RpcError::invalid_request()))?;
            continue;
        }
        let outcome = handle_request(&request, &scope);
        write_response(&mut out, &id, outcome)?;
    }
    Ok(())
}

#[derive(Deserialize)]
struct RpcRequest {
    jsonrpc: String,
    #[serde(default)]
    id: Option<Value>,
    #[serde(default)]
    method: Option<String>,
    #[serde(default)]
    params: Value,
}

fn handle_request(request: &RpcRequest, scope: &Scope) -> Result<Value, RpcError> {
    match request.method.as_deref() {
        Some("initialize") => Ok(json!({
            "protocolVersion": PROTOCOL_VERSION,
            "capabilities": { "tools": {} },
            "serverInfo": {
                "name": SERVER_NAME,
                "version": env!("CARGO_PKG_VERSION")
            }
        })),
        Some("ping") => Ok(Value::Null),
        Some("tools/list") => Ok(json!({ "tools": [tool_definition()] })),
        Some("tools/call") => handle_tools_call(&request.params, scope),
        Some(_) => Err(RpcError::method_not_found()),
        None => Err(RpcError::invalid_request()),
    }
}

fn handle_tools_call(params: &Value, scope: &Scope) -> Result<Value, RpcError> {
    let name = params
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| RpcError::invalid_params("missing tool name"))?;
    if name != TOOL_NAME {
        return Err(RpcError::method_not_found());
    }
    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
    let args: TextStatsArgs = serde_json::from_value(arguments)
        .map_err(|_| RpcError::invalid_params("text_stats arguments must be {\"path\": string}"))?;
    match compute_stats(&scope.query_root, &args.path, scope.max_bytes) {
        Ok(stats) => {
            let result = json!({
                "path": stats.path,
                "size_bytes": stats.size_bytes,
                "sha256": stats.sha256,
                "line_count": stats.line_count,
                "word_count": stats.word_count,
                "character_count": stats.character_count,
            });
            Ok(json!({
                "content": [{ "type": "text", "text": result.to_string() }],
                "structuredContent": result,
                "isError": false
            }))
        }
        Err(error) => Ok(json!({
            "content": [{ "type": "text", "text": error.to_string() }],
            "structuredContent": Value::Null,
            "isError": true
        })),
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TextStatsArgs {
    path: String,
}

fn output_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "path": { "type": "string" },
            "size_bytes": { "type": "integer", "minimum": 0 },
            "sha256": { "type": "string", "pattern": "^sha256:[a-f0-9]{64}$" },
            "line_count": { "type": "integer", "minimum": 0 },
            "word_count": { "type": "integer", "minimum": 0 },
            "character_count": { "type": "integer", "minimum": 0 }
        },
        "required": ["path", "size_bytes", "sha256", "line_count", "word_count", "character_count"],
        "additionalProperties": false
    })
}

fn tool_definition() -> Value {
    json!({
        "name": TOOL_NAME,
        "description": "Bounded Text Statistics",
        "inputSchema": {
            "type": "object",
            "properties": { "path": { "type": "string" } },
            "required": ["path"],
            "additionalProperties": false
        },
        "outputSchema": output_schema()
    })
}

#[derive(Debug, Clone, Copy)]
struct RpcError {
    code: i64,
    message: &'static str,
}

impl RpcError {
    fn invalid_request() -> Self {
        RpcError {
            code: -32600,
            message: "Invalid Request",
        }
    }
    fn method_not_found() -> Self {
        RpcError {
            code: -32601,
            message: "Method not found",
        }
    }
    fn invalid_params(message: &'static str) -> Self {
        RpcError {
            code: -32602,
            message,
        }
    }
}

fn write_response(
    out: &mut impl Write,
    id: &Value,
    outcome: Result<Value, RpcError>,
) -> io::Result<()> {
    let body = match outcome {
        Ok(result) => json!({ "jsonrpc": "2.0", "id": id, "result": result }),
        Err(error) => json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": error.code, "message": error.message }
        }),
    };
    writeln!(out, "{body}")?;
    out.flush()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static DIR_SEQ: AtomicU64 = AtomicU64::new(0);

    struct TempDir(PathBuf);

    impl TempDir {
        fn new(tag: &str) -> TempDir {
            let seq = DIR_SEQ.fetch_add(1, Ordering::Relaxed);
            let path = env::temp_dir().join(format!(
                "text-stats-provider-test-{tag}-{}-{seq}",
                std::process::id()
            ));
            fs::create_dir_all(&path).expect("create temp dir");
            TempDir(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn valid_utf8_file_returns_correct_stats() {
        let dir = TempDir::new("valid");
        let content = "hello world\nsecond line\nthird with café ☕\n";
        let file = dir.path().join("sample.txt");
        fs::write(&file, content).expect("write fixture");

        let stats = compute_stats(dir.path(), "sample.txt", HARD_MAX_BYTES).expect("stats");

        assert_eq!(stats.path, "sample.txt");
        assert_eq!(stats.size_bytes, content.len() as u64);
        assert_eq!(stats.line_count, 3);
        assert_eq!(stats.word_count, 8);
        assert_eq!(stats.character_count, content.chars().count());
        assert_eq!(stats.sha256.len(), "sha256:".len() + 64);
        let expected_digest = format!("sha256:{}", hex_lower(&Sha256::digest(content.as_bytes())));
        assert_eq!(stats.sha256, expected_digest);
    }

    #[test]
    fn path_traversal_and_absolute_paths_refuse() {
        let dir = TempDir::new("traversal");
        let root = dir.path().join("root");
        fs::create_dir_all(&root).expect("create root");
        fs::write(dir.path().join("secret.txt"), "secret").expect("write outside");

        assert!(matches!(
            compute_stats(&root, "..\\secret.txt", HARD_MAX_BYTES),
            Err(StatsError::Traversal(ref p)) if p == "..\\secret.txt"
        ));
        assert!(matches!(
            compute_stats(&root, "sub/..\\..\\secret.txt", HARD_MAX_BYTES),
            Err(StatsError::Traversal(_))
        ));
        let absolute = root.join("secret.txt").to_string_lossy().into_owned();
        assert!(matches!(
            compute_stats(&root, &absolute, HARD_MAX_BYTES),
            Err(StatsError::AbsolutePath(_))
        ));
    }

    #[test]
    fn oversized_file_refuses() {
        let dir = TempDir::new("oversize");
        let root = dir.path().join("root");
        fs::create_dir_all(&root).expect("create root");
        let content = vec![b'a'; 2048];
        fs::write(root.join("big.txt"), &content).expect("write fixture");

        let error = compute_stats(&root, "big.txt", 1024).expect_err("must refuse");
        assert!(matches!(
            error,
            StatsError::TooLarge {
                size_bytes: 2048,
                max_bytes: 1024
            }
        ));
    }

    #[test]
    fn malformed_utf8_refuses() {
        let dir = TempDir::new("malformed");
        let root = dir.path().join("root");
        fs::create_dir_all(&root).expect("create root");
        let bytes = vec![0x48, 0x69, 0xFF, 0xFE, 0x80];
        fs::write(root.join("bad.bin"), &bytes).expect("write fixture");

        let error = compute_stats(&root, "bad.bin", HARD_MAX_BYTES).expect_err("must refuse");
        assert!(matches!(error, StatsError::InvalidUtf8));
    }

    #[test]
    fn directory_input_refuses_as_not_regular_file() {
        let dir = TempDir::new("directory");
        let root = dir.path().join("root");
        fs::create_dir_all(&root).expect("create root");
        fs::create_dir_all(root.join("subdir")).expect("create subdir");

        let error = compute_stats(&root, "subdir", HARD_MAX_BYTES).expect_err("must refuse");
        assert!(matches!(error, StatsError::NotRegularFile));
    }

    #[test]
    fn scope_above_hard_max_refuses() {
        let raw = json!({
            "query_root": "C:\\does\\not\\matter",
            "max_bytes": HARD_MAX_BYTES + 1
        })
        .to_string();
        let error = parse_scope(&raw).expect_err("must refuse");
        assert_eq!(
            error,
            ScopeError::MaxBytesAboveHardLimit(HARD_MAX_BYTES + 1)
        );
    }

    #[test]
    fn scope_at_hard_max_parses_when_root_exists() {
        let dir = TempDir::new("scope-ok");
        let raw = json!({
            "query_root": dir.path().to_string_lossy(),
            "max_bytes": HARD_MAX_BYTES
        })
        .to_string();
        let scope = parse_scope(&raw).expect("valid scope");
        assert_eq!(scope.max_bytes, HARD_MAX_BYTES);
        assert_eq!(scope.query_root, dir.path());
    }

    #[test]
    fn unknown_or_missing_arguments_refuse() {
        let dir = TempDir::new("args");
        let scope = Scope {
            query_root: dir.path().to_path_buf(),
            max_bytes: HARD_MAX_BYTES,
        };

        let missing_path = json!({ "name": TOOL_NAME, "arguments": {} });
        assert!(matches!(
            handle_tools_call(&missing_path, &scope),
            Err(RpcError { code: -32602, .. })
        ));

        let unknown_field =
            json!({ "name": TOOL_NAME, "arguments": { "path": "a.txt", "extra": 1 } });
        assert!(matches!(
            handle_tools_call(&unknown_field, &scope),
            Err(RpcError { code: -32602, .. })
        ));

        let non_string_path = json!({ "name": TOOL_NAME, "arguments": { "path": 42 } });
        assert!(matches!(
            handle_tools_call(&non_string_path, &scope),
            Err(RpcError { code: -32602, .. })
        ));

        let unknown_tool = json!({ "name": "other_tool", "arguments": { "path": "a.txt" } });
        assert!(matches!(
            handle_tools_call(&unknown_tool, &scope),
            Err(RpcError { code: -32601, .. })
        ));
    }
}
