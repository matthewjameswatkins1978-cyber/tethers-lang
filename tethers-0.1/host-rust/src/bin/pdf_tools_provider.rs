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
const TETHERS_CONFORMANCE_ENV: &str = "TETHERS_CONFORMANCE";
const TEMP_ENV: &str = "TEMP";

fn response(id: &Value, result: Value) -> Value {
    json!({"jsonrpc":"2.0","id":id,"result":result})
}

fn error(id: &Value, code: i32, message: &str) -> Value {
    json!({"jsonrpc":"2.0","id":id,"error":{"code":code,"message":message}})
}

fn is_conformance_mode(value: Option<&str>) -> bool {
    value == Some("1")
}

fn conformance_scope(temp: Option<&str>) -> Result<PdfScope, String> {
    let temp = temp
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "TEMP is required during host conformance".to_owned())?;
    PdfScope::new(&PathBuf::from(temp), pdf_tools::MAX_PDF_BYTES)
        .map_err(|failure| failure.to_string())
}

fn scope_from_configuration(
    conformance: bool,
    operational_scope_json: Option<&str>,
    temp: Option<&str>,
) -> Result<PdfScope, String> {
    let json = match operational_scope_json.filter(|value| !value.is_empty()) {
        Some(json) => json,
        None if conformance => return conformance_scope(temp),
        None => {
            return Err(
                "TETHERS_OPERATIONAL_SCOPE_JSON is required in installed operational mode".into(),
            )
        }
    };
    let scope: Value =
        serde_json::from_str(json).map_err(|_| "malformed operational scope JSON".to_owned())?;
    let query_root = scope
        .get("query_root")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "query_root is required and must be a non-empty string".to_owned())?;
    let max_bytes = if conformance {
        pdf_tools::MAX_PDF_BYTES
    } else {
        scope
            .get("max_bytes")
            .and_then(Value::as_u64)
            .ok_or_else(|| "max_bytes is required and must be an integer".to_owned())?
    };
    PdfScope::new(&PathBuf::from(query_root), max_bytes).map_err(|failure| failure.to_string())
}

fn resolve_scope() -> Result<PdfScope, String> {
    let scope = match std::env::var_os(OPERATIONAL_SCOPE_JSON_ENV) {
        Some(value) => Some(
            value
                .into_string()
                .map_err(|_| "TETHERS_OPERATIONAL_SCOPE_JSON must be valid UTF-8".to_owned())?,
        ),
        None => None,
    };
    scope_from_configuration(
        is_conformance_mode(std::env::var(TETHERS_CONFORMANCE_ENV).ok().as_deref()),
        scope.as_deref(),
        std::env::var(TEMP_ENV).ok().as_deref(),
    )
}

fn main() {
    let scope = match resolve_scope() {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn scope_json(root: &std::path::Path, max_bytes: Value) -> String {
        json!({"query_root": root, "max_bytes": max_bytes}).to_string()
    }

    #[test]
    fn valid_normal_scope_uses_exact_max_bytes() {
        let root = std::env::temp_dir();
        let scope =
            scope_from_configuration(false, Some(&scope_json(&root, json!(4096))), None).unwrap();
        assert_eq!(scope.max_bytes, 4096);
    }

    #[test]
    fn conformance_unset_does_not_activate_fallback() {
        assert!(!is_conformance_mode(None));
        assert!(scope_from_configuration(is_conformance_mode(None), None, None).is_err());
    }

    #[test]
    fn conformance_zero_does_not_activate_fallback() {
        assert!(!is_conformance_mode(Some("0")));
        assert!(scope_from_configuration(is_conformance_mode(Some("0")), None, None).is_err());
    }

    #[test]
    fn other_conformance_values_do_not_activate_fallback() {
        assert!(!is_conformance_mode(Some("true")));
        assert!(scope_from_configuration(is_conformance_mode(Some("true")), None, None).is_err());
    }

    #[test]
    fn missing_max_bytes_in_normal_mode_refuses() {
        let root = std::env::temp_dir();
        let scope = json!({"query_root": root}).to_string();
        assert!(scope_from_configuration(false, Some(&scope), None).is_err());
    }

    #[test]
    fn malformed_scope_refuses() {
        assert!(scope_from_configuration(false, Some("{"), None).is_err());
    }

    #[test]
    fn wrong_max_bytes_type_refuses() {
        let root = std::env::temp_dir();
        let scope = scope_json(&root, json!("4096"));
        assert!(scope_from_configuration(false, Some(&scope), None).is_err());
    }

    #[test]
    fn out_of_range_max_bytes_refuses() {
        let root = std::env::temp_dir();
        let scope = scope_json(&root, json!(pdf_tools::MAX_PDF_BYTES + 1));
        assert!(scope_from_configuration(false, Some(&scope), None).is_err());
    }

    #[test]
    fn conformance_one_preserves_maximum_bound_fallback() {
        let temp = std::env::temp_dir();
        let scope =
            scope_from_configuration(is_conformance_mode(Some("1")), None, temp.to_str()).unwrap();
        assert_eq!(scope.max_bytes, pdf_tools::MAX_PDF_BYTES);
    }
}
