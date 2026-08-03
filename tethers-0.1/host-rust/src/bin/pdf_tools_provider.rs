//! Credential-free native stdio provider for the M6A PDF inspection capability.
//!
//! The binary is a thin MCP transport around `pdf_tools::inspect`. All PDF
//! semantics, scope enforcement, and bounds live in the library so the provider
//! process cannot drift from the reviewed capability contract.

use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use tethers_reference_host::pdf_tools::{self, PdfScope};

const PROVIDER_IDENTITY: &str = "tethers-pdf-provider";
const PROVIDER_VERSION: &str = "1.0.0";
const PROTOCOL_VERSION: &str = "2025-11-25";

/// Package-declared query root. The installed launcher replaces it with the
/// exact operational root; during host conformance the provider resolves it
/// from the host-owned TEMP scratch directory.
const PDF_QUERY_ROOT_PLACEHOLDER: &str = "__TETHERS_PDF_QUERY_ROOT__";

fn argument(name: &str) -> Option<String> {
    let mut args = std::env::args().skip(1);
    while let Some(value) = args.next() {
        if value == name {
            return args.next();
        }
    }
    None
}

fn response(id: &Value, result: Value) -> Value {
    json!({"jsonrpc":"2.0","id":id,"result":result})
}

fn error(id: &Value, code: i32, message: &str) -> Value {
    json!({"jsonrpc":"2.0","id":id,"error":{"code":code,"message":message}})
}

fn resolve_query_root(arg: &str) -> PathBuf {
    if arg == PDF_QUERY_ROOT_PLACEHOLDER {
        return conformance_query_root();
    }
    // Any other reviewed placeholder is unsupported and must not silently widen
    // scope to the current directory, profile, repository, or arbitrary env.
    if arg.starts_with("__TETHERS_PDF_") {
        eprintln!("pdf provider configuration refused: unsupported query-root placeholder {arg}");
        std::process::exit(2);
    }
    PathBuf::from(arg)
}

/// Resolve the reviewed placeholder only during host conformance. The host
/// conformance launcher supplies TETHERS_CONFORMANCE=1 and a clean TEMP scratch
/// directory; outside that contract the placeholder is refused.
fn conformance_query_root() -> PathBuf {
    let conformance = std::env::var("TETHERS_CONFORMANCE").unwrap_or_default();
    if conformance != "1" {
        eprintln!(
            "pdf provider configuration refused: query-root placeholder is only valid during host conformance (TETHERS_CONFORMANCE=1)"
        );
        std::process::exit(2);
    }
    let temp = match std::env::var("TEMP") {
        Ok(value) if !value.is_empty() => value,
        _ => {
            eprintln!(
                "pdf provider configuration refused: TEMP is not set during host conformance"
            );
            std::process::exit(2);
        }
    };
    let path = PathBuf::from(temp);
    match PdfScope::new(&path, pdf_tools::MAX_PDF_BYTES) {
        Ok(scope) => scope.query_root,
        Err(failure) => {
            eprintln!(
                "pdf provider configuration refused: TEMP is not a valid query root: {failure}"
            );
            std::process::exit(2);
        }
    }
}

fn main() {
    // The query root is host configuration, never provider-inferred: an absent
    // or unusable root must refuse the session rather than widen scope.
    let Some(root_arg) = argument("--query-root") else {
        eprintln!(
            "pdf provider configuration refused: --query-root <absolute directory> is required"
        );
        std::process::exit(2);
    };
    let root = resolve_query_root(&root_arg);
    let scope = match PdfScope::new(&root, pdf_tools::MAX_PDF_BYTES) {
        Ok(scope) => scope,
        Err(failure) => {
            eprintln!("pdf provider configuration refused: {failure}");
            std::process::exit(2);
        }
    };

    let stdin = io::stdin();
    let mut initialized = false;
    let mut client_initialized = false;
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        let request: Value = match serde_json::from_str(&line) {
            Ok(value) => value,
            Err(_) => {
                eprintln!("malformed JSON request");
                continue;
            }
        };
        let id = request.get("id").cloned().unwrap_or(Value::Null);
        let method = request.get("method").and_then(Value::as_str).unwrap_or("");
        let result = match method {
            "initialize" => {
                initialized = true;
                Ok(json!({
                    "protocolVersion": PROTOCOL_VERSION,
                    "capabilities": {"tools": {}},
                    "serverInfo": {"name": PROVIDER_IDENTITY, "version": PROVIDER_VERSION}
                }))
            }
            "notifications/initialized" => {
                if initialized {
                    client_initialized = true;
                }
                Ok(Value::Null)
            }
            "tools/list" if client_initialized => Ok(json!({"tools":[{
                "name": pdf_tools::INSPECT_OPERATION,
                "inputSchema": pdf_tools::inspect_input_schema(),
                "outputSchema": pdf_tools::inspect_output_schema()
            }]})),
            "tools/call" if client_initialized => {
                let params = request.get("params").cloned().unwrap_or(Value::Null);
                let name = params.get("name").and_then(Value::as_str).unwrap_or("");
                let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                match name {
                    pdf_tools::INSPECT_OPERATION => pdf_tools::inspect(&scope, &arguments),
                    _ => Err(pdf_tools::PdfToolsError {
                        code: "unknown_operation",
                        message: "operation is not part of the reviewed PDF contract".into(),
                    }),
                }
            }
            "tools/list" | "tools/call" => Err(pdf_tools::PdfToolsError {
                code: "not_initialized",
                message: "MCP session is not initialized".into(),
            }),
            _ => Err(pdf_tools::PdfToolsError {
                code: "method_not_found",
                message: "unsupported MCP method".into(),
            }),
        };
        if method == "notifications/initialized" {
            continue;
        }
        let output = match result {
            Ok(value) => response(&id, if value.is_null() { json!({}) } else { value }),
            Err(failure) => error(
                &id,
                if failure.code == "unknown_operation" || failure.code == "method_not_found" {
                    -32601
                } else {
                    -32602
                },
                &failure.message,
            ),
        };
        println!(
            "{}",
            serde_json::to_string(&output).expect("provider response is serializable")
        );
        io::stdout().flush().ok();
    }
}
