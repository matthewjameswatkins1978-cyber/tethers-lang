//! Public `trail` command coordinator.
//!
//! This module owns the read-only Trail inspection route: path validation,
//! strict JSONL parsing, execution-ID filtering, and envelope mapping.
//! It never executes a Tether, starts the OCaml engine, starts a provider,
//! consults replay persistence, or mutates any file.

use crate::dispatch;
use crate::manifest;
use crate::replay::ExecutionId;
use serde_json::{json, Value};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use tethers_reference_host::cli::{CliEnvelope, OutcomeStatus};

const MAX_LINE_BYTES: u64 = 8 * 1024 * 1024; // 8 MiB

/// Result of a trail command invocation.
#[derive(Debug)]
pub struct TrailResult {
    pub json_output: String,
    pub exit_code: i32,
}

/// Execute the public trail command.  All untrusted file and JSON processing
/// stays inside this boundary.  No engine, provider, or replay access.
pub fn run_trail(trail_path: &Path, execution_id_str: &str) -> TrailResult {
    // --- 1. Validate execution ID before opening the file ---
    let execution_id = match ExecutionId::parse(execution_id_str.to_owned()) {
        Ok(id) => id,
        Err(_) => {
            return failure(
                OutcomeStatus::InvalidData,
                "EXECUTION_ID_INVALID",
                "execution ID format is invalid",
                Some("--execution-id".to_string()),
            );
        }
    };
    let execution_id_value = execution_id.as_str().to_owned();

    // --- 2. Validate trail path ---
    if !trail_path.is_absolute() {
        return failure(
            OutcomeStatus::InvalidData,
            "TRAIL_NOT_ABSOLUTE",
            "trail path must be absolute",
            Some("--trail".to_string()),
        );
    }

    if !trail_path.exists() {
        return failure(
            OutcomeStatus::NotFound,
            "TRAIL_NOT_FOUND",
            "trail file was not found",
            Some("--trail".to_string()),
        );
    }

    if !trail_path.is_file() {
        return failure(
            OutcomeStatus::InvalidData,
            "TRAIL_NOT_FILE",
            "trail path must be a regular file",
            Some("--trail".to_string()),
        );
    }

    // Canonicalise after validation.
    let canonical_path = match trail_path.canonicalize() {
        Ok(p) => p,
        Err(_) => {
            return failure(
                OutcomeStatus::Unavailable,
                "TRAIL_UNAVAILABLE",
                "cannot access trail file",
                Some("--trail".to_string()),
            );
        }
    };

    // --- 3. Open the file read-only ---
    let file = match fs::File::open(&canonical_path) {
        Ok(f) => f,
        Err(_) => {
            return failure(
                OutcomeStatus::Unavailable,
                "TRAIL_UNAVAILABLE",
                "cannot open trail file",
                Some("--trail".to_string()),
            );
        }
    };

    // --- 4. Read and filter entries ---
    let reader = BufReader::new(file);
    let raw_entries = match read_and_filter(reader, &execution_id_value) {
        Ok(entries) => entries,
        Err(result) => return result,
    };

    // --- 5. Build output ---
    let entry_count = raw_entries.len();
    let trail_path_display = canonical_path.display().to_string();

    if entry_count == 0 {
        let data = json!({
            "execution_id": execution_id_value,
            "trail_path": trail_path_display,
            "entry_count": 0,
        });
        let envelope = CliEnvelope::error_with_data(
            "trail",
            OutcomeStatus::NotFound,
            "EXECUTION_NOT_FOUND",
            "no trail entries matched the supplied execution ID",
            Some("--execution-id".to_string()),
            data,
        );
        let json_output = serde_json::to_string(&envelope)
            .unwrap_or_else(|_| r#"{"schema":"tethers.cli/1"}"#.into());
        return TrailResult {
            json_output,
            exit_code: envelope.exit_code,
        };
    }

    // Build success envelope manually to preserve original entry text.
    // Only execution_id and trail_path are escaped by serde_json; raw
    // entries are already validated single-object JSON strings inserted
    // directly.
    let escaped_exec_id =
        serde_json::to_string(&execution_id_value).unwrap_or_else(|_| "\"\"".into());
    let escaped_trail =
        serde_json::to_string(&trail_path_display).unwrap_or_else(|_| "\"\"".into());
    let entries_joined = raw_entries.join(",");

    let json_output = format!(
        concat!(
            r#"{{"schema":"tethers.cli/1","#,
            r#""command":"trail","#,
            r#""status":"ok","#,
            r#""exit_code":0,"#,
            r#""error":null,"#,
            r#""data":{{"#,
            r#""execution_id":{},"#,
            r#""trail_path":{},"#,
            r#""entry_count":{},"#,
            r#""entries":[{}]"#,
            r#"}}}}"#
        ),
        escaped_exec_id, escaped_trail, entry_count, entries_joined,
    );

    TrailResult {
        json_output,
        exit_code: 0,
    }
}

/// Read a JSONL file, parse each line as strict JSON, and collect raw text
/// of entries whose top-level `execution_id` matches the supplied value.
///
/// Each line is accumulated as raw bytes, validated as UTF-8 once on the
/// complete line (so multibyte characters split across reader-buffer
/// boundaries are handled correctly), then structurally validated.
///
/// Any malformed line, duplicate key, blank line, non-object JSON, or line
/// exceeding `MAX_LINE_BYTES` invalidates the entire inspection.
fn read_and_filter(
    mut reader: impl BufRead,
    execution_id: &str,
) -> Result<Vec<String>, TrailResult> {
    let mut entries: Vec<String> = Vec::new();
    let mut line_count: u64 = 0;
    let mut line_bytes: Vec<u8> = Vec::new();

    loop {
        line_count += 1;
        line_bytes.clear();

        let bytes_read = read_line_limited(&mut line_bytes, &mut reader, line_count)?;
        if bytes_read == 0 {
            break; // EOF
        }

        // Strip physical line terminator: trailing LF, then one preceding CR.
        let mut content: &[u8] = &line_bytes;
        if content.last() == Some(&b'\n') {
            content = &content[..content.len() - 1];
        }
        if content.last() == Some(&b'\r') {
            content = &content[..content.len() - 1];
        }

        // Empty lines are invalid.
        if content.is_empty() {
            return Err(invalid_trail(line_count));
        }

        // Validate UTF-8 once on the complete accumulated line.
        let line_str = match std::str::from_utf8(content) {
            Ok(s) => s,
            Err(_) => return Err(invalid_trail(line_count)),
        };

        // Strict JSON parsing with duplicate-key rejection.
        let value = match manifest::parse_value_no_dupes(line_str) {
            Ok(v) => v,
            Err(_) => return Err(invalid_trail(line_count)),
        };

        // Must be a JSON object.
        match value {
            Value::Object(obj) => {
                match obj.get("execution_id") {
                    Some(Value::String(id)) if id == execution_id => {
                        // Validate semantic_position structure when present.
                        // Old records without the field are accepted.
                        // Null is treated as absent (matches skip_serializing_if).
                        match obj.get("semantic_position") {
                            Some(Value::Null) | None => {}
                            Some(sp) => {
                                if let Err(e) = dispatch::validate_semantic_position_json(sp) {
                                    return Err(invalid_trail_reason(line_count, &e.to_string()));
                                }
                            }
                        }
                        // Store the exact raw validated text.
                        entries.push(line_str.to_owned());
                    }
                    Some(Value::String(_)) => {
                        // Different execution ID: skip silently.
                    }
                    Some(_) => {
                        // Non-string execution_id: invalid.
                        return Err(invalid_trail(line_count));
                    }
                    None => {
                        // No execution_id field: skip silently (audit entry).
                    }
                }
            }
            _ => return Err(invalid_trail(line_count)), // non-object
        }
    }

    Ok(entries)
}

/// Read one physical line (up to and including LF) into the supplied byte
/// buffer, enforcing the 8 MiB size limit.
///
/// Bytes are accumulated across `fill_buf` / `consume` cycles without
/// intermediate UTF‑8 validation so that a multibyte character split across
/// internal reader‑buffer boundaries is not rejected prematurely.
///
/// Returns the number of bytes read (including the line terminator), or 0 at
/// EOF with no data.
fn read_line_limited(
    buf: &mut Vec<u8>,
    reader: &mut impl BufRead,
    line_number: u64,
) -> Result<usize, TrailResult> {
    let mut total: u64 = 0;
    loop {
        let available = match reader.fill_buf() {
            Ok(b) => b,
            Err(_) => {
                return Err(failure(
                    OutcomeStatus::AuditFailed,
                    "TRAIL_INVALID",
                    "cannot read trail file",
                    None::<String>,
                ));
            }
        };
        if available.is_empty() {
            return Ok(total as usize);
        }

        // Find LF in the available chunk.
        let newline_pos = available.iter().position(|&b| b == b'\n');
        let consume = match newline_pos {
            Some(pos) => pos + 1, // include the LF
            None => available.len(),
        };

        let chunk_len = consume as u64;
        if total + chunk_len > MAX_LINE_BYTES {
            return Err(invalid_trail(line_number));
        }

        buf.extend_from_slice(&available[..consume]);
        total += chunk_len;

        reader.consume(consume);

        if newline_pos.is_some() {
            break;
        }
    }
    Ok(total as usize)
}

fn invalid_trail(line_number: u64) -> TrailResult {
    // Build the complete output manually so the line number survives.
    let envelope_with_data = json!({
        "schema": "tethers.cli/1",
        "command": "trail",
        "status": "audit_failed",
        "exit_code": 8,
        "error": {
            "code": "TRAIL_INVALID",
            "message": "trail file content is invalid"
        },
        "data": {
            "line": line_number
        }
    });
    let json_output = serde_json::to_string(&envelope_with_data)
        .unwrap_or_else(|_| r#"{"schema":"tethers.cli/1"}"#.into());
    TrailResult {
        json_output,
        exit_code: 8,
    }
}

fn invalid_trail_reason(line_number: u64, reason: &str) -> TrailResult {
    let envelope_with_data = json!({
        "schema": "tethers.cli/1",
        "command": "trail",
        "status": "audit_failed",
        "exit_code": 8,
        "error": {
            "code": "TRAIL_INVALID",
            "message": format!("trail file content is invalid: {reason}")
        },
        "data": {
            "line": line_number
        }
    });
    let json_output = serde_json::to_string(&envelope_with_data)
        .unwrap_or_else(|_| r#"{"schema":"tethers.cli/1"}"#.into());
    TrailResult {
        json_output,
        exit_code: 8,
    }
}

fn failure(
    status: OutcomeStatus,
    code: impl Into<String>,
    message: impl Into<String>,
    field: Option<String>,
) -> TrailResult {
    let envelope = CliEnvelope::error("trail", status, code, message, field);
    let exit_code = envelope.exit_code;
    let json_output =
        serde_json::to_string(&envelope).unwrap_or_else(|_| r#"{"schema":"tethers.cli/1"}"#.into());
    TrailResult {
        json_output,
        exit_code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    // Helper: build a JSONL string from a slice of raw JSON strings.
    fn jsonl(lines: &[&str]) -> String {
        let mut out = String::new();
        for line in lines {
            out.push_str(line);
            out.push('\n');
        }
        out
    }

    fn entry_with_id(execution_id: &str, extra: &str) -> String {
        format!(
            r#"{{"execution_id":"{}","kind":"test","extra":"{}"}}"#,
            execution_id, extra
        )
    }

    fn entry_no_id(extra: &str) -> String {
        format!(r#"{{"kind":"audit","extra":"{}"}}"#, extra)
    }

    #[test]
    fn j13c_valid_execution_id_accepted() {
        assert!(ExecutionId::parse("exec_00000000-0000-4000-8000-000000000000".to_owned()).is_ok());
    }

    #[test]
    fn j13c_malformed_execution_id_rejected() {
        for bad in [
            "not-exec_00000000-0000-4000-8000-000000000000",
            "exec_not-a-uuid",
            "exec_00000000-0000-4000-8000-00000000000", // too short
            "",
            "exec_",
        ] {
            assert!(
                ExecutionId::parse(bad.to_owned()).is_err(),
                "should reject: {bad}"
            );
        }
    }

    #[test]
    fn j13c_matching_entries_returned_in_original_order() {
        let target = "exec_00000000-0000-4000-8000-000000000000";
        let other = "exec_00000000-0000-4000-8000-000000000001";
        let input = jsonl(&[
            &entry_with_id(other, "1"),
            &entry_with_id(target, "2"),
            &entry_no_id("3"),
            &entry_with_id(target, "4"),
            &entry_with_id(other, "5"),
            &entry_with_id(target, "6"),
        ]);
        let reader = BufReader::new(Cursor::new(input.as_bytes()));
        let raw = read_and_filter(reader, target).unwrap();
        assert_eq!(raw.len(), 3);
        // Parse each back to check ordering.
        let extra_vals: Vec<String> = raw
            .iter()
            .map(|s| {
                let v: Value = serde_json::from_str(s).unwrap();
                v["extra"].as_str().unwrap().to_owned()
            })
            .collect();
        assert_eq!(extra_vals, vec!["2", "4", "6"]);
    }

    #[test]
    fn j13c_unrelated_execution_ids_omitted() {
        let target = "exec_00000000-0000-4000-8000-000000000000";
        let other = "exec_00000000-0000-4000-8000-000000000001";
        let input = jsonl(&[&entry_with_id(other, "1"), &entry_with_id(other, "2")]);
        let reader = BufReader::new(Cursor::new(input.as_bytes()));
        let raw = read_and_filter(reader, target).unwrap();
        assert!(raw.is_empty());
    }

    #[test]
    fn j13c_valid_audit_entries_without_execution_id_skipped() {
        let target = "exec_00000000-0000-4000-8000-000000000000";
        let input = jsonl(&[
            &entry_no_id("a"),
            &entry_with_id(target, "b"),
            &entry_no_id("c"),
        ]);
        let reader = BufReader::new(Cursor::new(input.as_bytes()));
        let raw = read_and_filter(reader, target).unwrap();
        assert_eq!(raw.len(), 1);
        let v: Value = serde_json::from_str(&raw[0]).unwrap();
        assert_eq!(v["extra"], "b");
    }

    #[test]
    fn j13c_zero_matches_maps_to_not_found() {
        let tmp =
            std::env::temp_dir().join(format!("j13c-zero-match-{}.jsonl", uuid::Uuid::new_v4()));
        let content = jsonl(&[&entry_with_id(
            "exec_00000000-0000-4000-8000-000000000001",
            "x",
        )]);
        fs::write(&tmp, &content).unwrap();
        let result = run_trail(&tmp, "exec_00000000-0000-4000-8000-000000000000");
        let envelope: Value = serde_json::from_str(&result.json_output).unwrap();
        assert_eq!(result.exit_code, 9);
        assert_eq!(envelope["status"], "not_found");
        assert_eq!(envelope["error"]["code"], "EXECUTION_NOT_FOUND");
        assert_eq!(envelope["data"]["entry_count"], 0);
        assert!(envelope["data"].get("entries").is_none());
        let _ = fs::remove_file(&tmp);
    }

    #[test]
    fn j13c_relative_trail_path_rejected() {
        let result = run_trail(
            Path::new("relative/path.jsonl"),
            "exec_00000000-0000-4000-8000-000000000000",
        );
        let envelope: Value = serde_json::from_str(&result.json_output).unwrap();
        assert_eq!(envelope["status"], "invalid_data");
        assert_eq!(result.exit_code, 3);
        assert_eq!(envelope["error"]["code"], "TRAIL_NOT_ABSOLUTE");
    }

    #[test]
    fn j13c_missing_trail_file_maps_to_not_found() {
        let missing = Path::new("C:\\does-not-exist-j13c-test.jsonl");
        let result = run_trail(missing, "exec_00000000-0000-4000-8000-000000000000");
        let envelope: Value = serde_json::from_str(&result.json_output).unwrap();
        assert_eq!(envelope["status"], "not_found");
        assert_eq!(result.exit_code, 9);
        assert_eq!(envelope["error"]["code"], "TRAIL_NOT_FOUND");
    }

    #[test]
    fn j13c_directory_path_rejected() {
        let dir = std::env::temp_dir();
        let result = run_trail(&dir, "exec_00000000-0000-4000-8000-000000000000");
        let envelope: Value = serde_json::from_str(&result.json_output).unwrap();
        assert_eq!(envelope["status"], "invalid_data");
        assert_eq!(result.exit_code, 3);
        assert_eq!(envelope["error"]["code"], "TRAIL_NOT_FILE");
    }

    #[test]
    fn j13c_malformed_json_maps_to_audit_failed() {
        let target = "exec_00000000-0000-4000-8000-000000000000";
        let input = format!("{{\"execution_id\": \"{}\"}}\n{{not json\n", target);
        let reader = BufReader::new(Cursor::new(input.as_bytes()));
        let result = read_and_filter(reader, target);
        assert!(result.is_err());
        let envelope: Value = serde_json::from_str(&result.unwrap_err().json_output).unwrap();
        assert_eq!(envelope["status"], "audit_failed");
    }

    #[test]
    fn j13c_duplicate_json_key_maps_to_audit_failed() {
        let target = "exec_00000000-0000-4000-8000-000000000000";
        let input = format!("{{\"execution_id\": \"{target}\", \"execution_id\": \"{target}\"}}\n");
        let reader = BufReader::new(Cursor::new(input.as_bytes()));
        let result = read_and_filter(reader, target);
        assert!(result.is_err());
        let envelope: Value = serde_json::from_str(&result.unwrap_err().json_output).unwrap();
        assert_eq!(envelope["status"], "audit_failed");
    }

    #[test]
    fn j13c_blank_line_maps_to_audit_failed() {
        let target = "exec_00000000-0000-4000-8000-000000000000";
        let input = format!("{{\"execution_id\": \"{target}\"}}\n\n");
        let reader = BufReader::new(Cursor::new(input.as_bytes()));
        let result = read_and_filter(reader, target);
        assert!(result.is_err());
        let envelope: Value = serde_json::from_str(&result.unwrap_err().json_output).unwrap();
        assert_eq!(envelope["status"], "audit_failed");
    }

    #[test]
    fn j13c_non_object_json_maps_to_audit_failed() {
        let target = "exec_00000000-0000-4000-8000-000000000000";
        let input = "\"just a string\"\n";
        let reader = BufReader::new(Cursor::new(input.as_bytes()));
        let result = read_and_filter(reader, target);
        assert!(result.is_err());
        let envelope: Value = serde_json::from_str(&result.unwrap_err().json_output).unwrap();
        assert_eq!(envelope["status"], "audit_failed");
    }

    #[test]
    fn j13c_non_string_execution_id_maps_to_audit_failed() {
        let target = "exec_00000000-0000-4000-8000-000000000000";
        let input = "{\"execution_id\": 42}\n";
        let reader = BufReader::new(Cursor::new(input.as_bytes()));
        let result = read_and_filter(reader, target);
        assert!(result.is_err());
        let envelope: Value = serde_json::from_str(&result.unwrap_err().json_output).unwrap();
        assert_eq!(envelope["status"], "audit_failed");
    }

    #[test]
    fn j13c_oversize_line_maps_to_audit_failed() {
        let target = "exec_00000000-0000-4000-8000-000000000000";
        // Build a line that exceeds 8 MiB.
        let padding = "x".repeat(9 * 1024 * 1024); // 9 MiB of padding
        let input = format!("{{\"execution_id\": \"{target}\", \"pad\": \"{padding}\"}}\n");
        let reader = BufReader::new(Cursor::new(input.as_bytes()));
        let result = read_and_filter(reader, target);
        assert!(result.is_err());
        let envelope: Value = serde_json::from_str(&result.unwrap_err().json_output).unwrap();
        assert_eq!(envelope["status"], "audit_failed");
    }

    #[test]
    fn j13c_lf_and_crlf_produce_equivalent_entry_selection() {
        let target = "exec_00000000-0000-4000-8000-000000000000";
        let other = "exec_00000000-0000-4000-8000-000000000001";

        let lf = format!(
            "{{\"execution_id\": \"{other}\"}}\n{{\"execution_id\": \"{target}\"}}\n{{\"execution_id\": \"{other}\"}}\n"
        );
        let crlf = format!(
            "{{\"execution_id\": \"{other}\"}}\r\n{{\"execution_id\": \"{target}\"}}\r\n{{\"execution_id\": \"{other}\"}}\r\n"
        );

        let lf_entries =
            read_and_filter(BufReader::new(Cursor::new(lf.as_bytes())), target).unwrap();
        let crlf_entries =
            read_and_filter(BufReader::new(Cursor::new(crlf.as_bytes())), target).unwrap();

        assert_eq!(lf_entries.len(), 1);
        assert_eq!(crlf_entries.len(), 1);
        // Both should contain the same raw text (CRLF stripped to the same content).
        assert_eq!(lf_entries[0], crlf_entries[0]);
    }

    #[test]
    fn j13c_success_envelope_has_no_timestamp() {
        let tmp = std::env::temp_dir().join(format!("j13c-no-ts-{}.jsonl", uuid::Uuid::new_v4()));
        let target = "exec_00000000-0000-4000-8000-000000000000";
        let content = format!("{{\"execution_id\": \"{target}\"}}\n");
        fs::write(&tmp, &content).unwrap();
        let result = run_trail(&tmp, target);
        assert!(!result.json_output.contains("timestamp"));
        let _ = fs::remove_file(&tmp);
    }

    #[test]
    fn j13c_not_found_envelope_contains_no_invented_entries() {
        let tmp =
            std::env::temp_dir().join(format!("j13c-no-invent-{}.jsonl", uuid::Uuid::new_v4()));
        let other = "exec_00000000-0000-4000-8000-000000000001";
        let content = format!("{{\"execution_id\": \"{other}\"}}\n");
        fs::write(&tmp, &content).unwrap();
        let result = run_trail(&tmp, "exec_00000000-0000-4000-8000-000000000000");
        let envelope: Value = serde_json::from_str(&result.json_output).unwrap();
        assert_eq!(envelope["data"]["entry_count"], 0);
        assert!(envelope["data"].get("entries").is_none());
        let _ = fs::remove_file(&tmp);
    }

    #[test]
    fn j13c_no_partial_success_when_later_line_malformed() {
        let target = "exec_00000000-0000-4000-8000-000000000000";
        let input = format!("{{\"execution_id\": \"{target}\"}}\n{{bad json\n");
        let reader = BufReader::new(Cursor::new(input.as_bytes()));
        let result = read_and_filter(reader, target);
        assert!(result.is_err());
        let envelope: Value = serde_json::from_str(&result.unwrap_err().json_output).unwrap();
        assert_eq!(envelope["status"], "audit_failed");
    }

    #[test]
    fn j13c_execution_id_validated_before_file_access() {
        // Use a path that doesn't exist, but with an invalid execution ID.
        // The execution-ID error should take precedence.
        let missing = Path::new("C:\\does-not-exist-j13c-precedence.jsonl");
        let result = run_trail(missing, "not-a-valid-exec-id");
        let envelope: Value = serde_json::from_str(&result.json_output).unwrap();
        assert_eq!(envelope["status"], "invalid_data");
        assert_eq!(result.exit_code, 3);
        assert_eq!(envelope["error"]["code"], "EXECUTION_ID_INVALID");
        assert_eq!(envelope["error"]["field"], "--execution-id");
    }

    #[test]
    fn j13c_success_envelope_has_correct_shape() {
        let tmp = std::env::temp_dir().join(format!("j13c-shape-{}.jsonl", uuid::Uuid::new_v4()));
        let target = "exec_00000000-0000-4000-8000-000000000000";
        let content = format!(
            "{{\"execution_id\": \"{target}\", \"kind\": \"event_admitted\"}}\n{{\"execution_id\": \"{target}\", \"kind\": \"action_intent\"}}\n"
        );
        fs::write(&tmp, &content).unwrap();
        let result = run_trail(&tmp, target);
        let envelope: Value = serde_json::from_str(&result.json_output).unwrap();
        assert_eq!(envelope["schema"], "tethers.cli/1");
        assert_eq!(envelope["command"], "trail");
        assert_eq!(envelope["status"], "ok");
        assert_eq!(result.exit_code, 0);
        assert_eq!(envelope["data"]["entry_count"], 2);
        assert_eq!(envelope["data"]["entries"].as_array().unwrap().len(), 2);
        assert!(envelope["error"].is_null());
        let _ = fs::remove_file(&tmp);
    }

    // -------------------------------------------------------------------
    // J13C-A corrected tests
    // -------------------------------------------------------------------

    /// Multibyte UTF-8 character split across 1-byte reader buffers must
    /// succeed — the old per-chunk validation would reject this.
    #[test]
    fn j13c_utf8_split_across_buffer_boundary() {
        let target = "exec_00000000-0000-4000-8000-000000000000";
        // U+00E9 (é) is 0xC3 0xA9 in UTF-8 — two bytes.
        // Include it inside a value so the JSON is still valid.
        let line = format!("{{\"execution_id\":\"{target}\",\"note\":\"caf\u{00E9}\"}}\n");
        // BufReader with capacity 1 forces one-byte fill_buf() chunks.
        let reader = BufReader::with_capacity(1, Cursor::new(line.as_bytes()));
        let raw = read_and_filter(reader, target).unwrap();
        assert_eq!(raw.len(), 1, "multibyte entry must be accepted");
        let v: Value = serde_json::from_str(&raw[0]).unwrap();
        assert_eq!(v["note"], "caf\u{00E9}");
    }

    /// Non-alphabetical key order must be preserved in output.
    #[test]
    fn j13c_preserves_non_alphabetical_key_order() {
        let target = "exec_00000000-0000-4000-8000-000000000000";
        let line = format!("{{\"z\":1,\"execution_id\":\"{target}\",\"a\":2}}\n");
        let reader = BufReader::new(Cursor::new(line.as_bytes()));
        let raw = read_and_filter(reader, target).unwrap();
        assert_eq!(raw.len(), 1);
        // The raw text must contain "z" before "a".
        let z_pos = raw[0].find("\"z\"").unwrap();
        let a_pos = raw[0].find("\"a\"").unwrap();
        assert!(
            z_pos < a_pos,
            "key order must be preserved: z before a, got: {}",
            raw[0]
        );
    }

    /// Spaces inside a matching object must be unchanged.
    #[test]
    fn j13c_preserves_internal_spaces() {
        let target = "exec_00000000-0000-4000-8000-000000000000";
        let line = format!("{{  \"execution_id\" : \"{target}\" , \"x\" :  1  }}\n");
        let reader = BufReader::new(Cursor::new(line.as_bytes()));
        let raw = read_and_filter(reader, target).unwrap();
        assert_eq!(raw.len(), 1);
        // The raw text should contain the exact spacing (excluding the \n).
        let expected_no_lf = line.trim_end_matches('\n');
        assert_eq!(raw[0], expected_no_lf, "spaces must be preserved exactly");
    }

    /// The success JSON output must parse as valid JSON.
    #[test]
    fn j13c_success_output_is_valid_json() {
        let tmp =
            std::env::temp_dir().join(format!("j13c-valid-json-{}.jsonl", uuid::Uuid::new_v4()));
        let target = "exec_00000000-0000-4000-8000-000000000000";
        let content = format!("{{\"execution_id\":\"{target}\",\"k\":\"v\"}}\n");
        fs::write(&tmp, &content).unwrap();
        let result = run_trail(&tmp, target);
        // Must parse without error.
        let _envelope: Value =
            serde_json::from_str(&result.json_output).expect("output must be valid JSON");
        let _ = fs::remove_file(&tmp);
    }

    /// The exact original matching object must appear inside the entries array.
    #[test]
    fn j13c_exact_original_text_in_entries() {
        let tmp = std::env::temp_dir().join(format!("j13c-exact-{}.jsonl", uuid::Uuid::new_v4()));
        let target = "exec_00000000-0000-4000-8000-000000000000";
        let entry_text = format!("{{\"z\":1,\"execution_id\":\"{target}\",\"a\":2}}");
        let content = format!("{entry_text}\n");
        fs::write(&tmp, &content).unwrap();
        let result = run_trail(&tmp, target);
        // The raw output string must contain the exact original entry text.
        assert!(
            result.json_output.contains(&entry_text),
            "raw output must contain original entry text:\n{}",
            result.json_output
        );
        // The output must still parse as valid JSON.
        let _envelope: Value =
            serde_json::from_str(&result.json_output).expect("output must be valid JSON");
        let _ = fs::remove_file(&tmp);
    }

    /// CRLF removes only the physical terminator, not internal data.
    #[test]
    fn j13c_crlf_preserves_internal_data() {
        let target = "exec_00000000-0000-4000-8000-000000000000";
        let data = format!("{{\"execution_id\":\"{target}\",\"value\":\"hello\"}}\r\n");
        let reader = BufReader::new(Cursor::new(data.as_bytes()));
        let raw = read_and_filter(reader, target).unwrap();
        assert_eq!(raw.len(), 1);
        // The \r is removed, the rest is intact.
        let v: Value = serde_json::from_str(&raw[0]).unwrap();
        assert_eq!(v["value"], "hello");
        // Raw text must not contain \r.
        assert!(!raw[0].contains('\r'), "CR must be stripped");
    }

    /// Malformed later data prevents all successful output (fail-closed).
    #[test]
    fn j13c_malformed_later_prevents_all_output() {
        let target = "exec_00000000-0000-4000-8000-000000000000";
        let input = format!("{{\"execution_id\":\"{target}\",\"ok\":true}}\n{{broken\n");
        let reader = BufReader::new(Cursor::new(input.as_bytes()));
        let result = read_and_filter(reader, target);
        assert!(result.is_err(), "must fail on later malformed line");
    }

    // -------------------------------------------------------------------
    // F3e1: production reader classification of truncated final entry
    // -------------------------------------------------------------------

    #[test]
    fn f3e1_truncated_final_line_maps_to_audit_failed() {
        let target = "exec_00000000-0000-4000-8000-000000000000";
        let complete = format!("{{\"execution_id\":\"{target}\",\"idx\":1}}");
        let truncated = format!("{{\"execution_id\":\"{target}\",\"idx");
        let input = format!("{complete}\n{truncated}"); // no trailing LF
        let reader = BufReader::new(Cursor::new(input.as_bytes()));
        let result = read_and_filter(reader, target);
        assert!(
            result.is_err(),
            "F3e1: truncated final line without trailing LF must fail entire inspection"
        );
        let envelope: Value = serde_json::from_str(&result.unwrap_err().json_output).unwrap();
        assert_eq!(
            envelope["status"], "audit_failed",
            "F3e1: production reader classifies truncated final entry as TRAIL_INVALID"
        );
        assert_eq!(
            envelope["error"]["code"], "TRAIL_INVALID",
            "F3e1: truncated final line -> TRAIL_INVALID (fail-closed, entire file rejected)"
        );
    }
}
