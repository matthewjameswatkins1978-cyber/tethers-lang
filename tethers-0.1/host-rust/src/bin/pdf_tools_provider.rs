//! Credential-free native stdio provider for the M6A PDF inspection capability.
//!
//! The binary is a thin MCP transport around `pdf_tools::inspect`. All PDF
//! semantics, scope enforcement, and bounds live in the library so the provider
//! process cannot drift from the reviewed capability contract.
//!
//! In installed operational mode, scope is delivered through
//! `TETHERS_OPERATIONAL_SCOPE_JSON` with exact SHA-256 integrity via
//! `TETHERS_OPERATIONAL_SCOPE_DIGEST`. During host conformance
//! (TETHERS_CONFORMANCE=1) a TEMP query root is used.

use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use tethers_reference_host::pdf_tools::{self, PdfScope};

const PROVIDER_IDENTITY: &str = "tethers-pdf-provider";
const PROVIDER_VERSION: &str = "1.0.0";
const PROTOCOL_VERSION: &str = "2025-11-25";

const OPERATIONAL_SCOPE_JSON_ENV: &str = "TETHERS_OPERATIONAL_SCOPE_JSON";

fn response(id: &Value, result: Value) -> Value {
    json!({"jsonrpc":"2.0","id":id,"result":result})
}

fn error(id: &Value, code: i32, message: &str) -> Value {
    json!({"jsonrpc":"2.0","id":id,"error":{"code":code,"message":message}})
}

fn resolve_query_root() -> PathBuf {
    let conformance = std::env::var("TETHERS_CONFORMANCE").unwrap_or_default();
    let json = match std::env::var(OPERATIONAL_SCOPE_JSON_ENV) {
        Ok(v) if !v.is_empty() => v,
        _ => {
            if conformance == "1" {
                return conformance_query_root();
            }
            eprintln!("pdf provider configuration refused: TETHERS_OPERATIONAL_SCOPE_JSON is required in installed operational mode");
            std::process::exit(2);
        }
    };
    let value: Value = match serde_json::from_str(&json) {
        Ok(v) => v,
        Err(_) => {
            eprintln!("pdf provider configuration refused: malformed operational scope JSON");
            std::process::exit(2);
        }
    };
    let root = match value.get("query_root").and_then(|v| v.as_str()) {
        Some(s) => s,
        None => {
            eprintln!("pdf provider configuration refused: query_root is not present in operational scope");
            std::process::exit(2);
        }
    };
    if root.is_empty() {
        eprintln!("pdf provider configuration refused: query_root is empty");
        std::process::exit(2);
    }
    PathBuf::from(root)
}

fn conformance_query_root() -> PathBuf {
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

fn resolve_max_bytes() -> u64 {
    let conformance = std::env::var("TETHERS_CONFORMANCE").unwrap_or_default();
    if conformance != "0" {
        return pdf_tools::MAX_PDF_BYTES;
    }
    let json = match std::env::var(OPERATIONAL_SCOPE_JSON_ENV) {
        Ok(v) if !v.is_empty() => v,
        _ => return pdf_tools::MAX_PDF_BYTES,
    };
    let value: Value = match serde_json::from_str(&json) {
        Ok(v) => v,
        Err(_) => return pdf_tools::MAX_PDF_BYTES,
    };
    match value.get("max_bytes").and_then(|v| v.as_u64()) {
        Some(n) if n >= 1 && n <= pdf_tools::MAX_PDF_BYTES => n,
        Some(n) => {
            eprintln!(
                "pdf provider configuration refused: max_bytes {n} is outside [1, {}]",
                pdf_tools::MAX_PDF_BYTES
            );
            std::process::exit(2);
        }
        None => pdf_tools::MAX_PDF_BYTES,
    }
}

fn main() {
    let root = resolve_query_root();
    let max_bytes = resolve_max_bytes();
    let scope = match PdfScope::new(&root, max_bytes) {
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
