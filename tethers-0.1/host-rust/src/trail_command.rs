//! Public `trail` command coordinator.
//!
//! This module owns the read-only Trail inspection route: path validation,
//! strict JSONL parsing, execution-ID filtering, and envelope mapping.
//! It never executes a Tether, starts the OCaml engine, starts a provider,
//! consults replay persistence, or mutates any file.

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
    pub envelope: CliEnvelope,
    pub exit_code: i32,
}

impl TrailResult {
    fn from_envelope(envelope: CliEnvelope) -> Self {
        let exit_code = envelope.exit_code;
        Self {
            envelope,
            exit_code,
        }
    }
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
    let entries = match read_and_filter(reader, &execution_id_value) {
        Ok(entries) => entries,
        Err(result) => return result,
    };

    // --- 5. Build envelope ---
    let entry_count = entries.len();
    if entry_count == 0 {
        let data = json!({
            "execution_id": execution_id_value,
            "trail_path": canonical_path.display().to_string(),
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
        return TrailResult::from_envelope(envelope);
    }

    let entries_array: Vec<Value> = entries;
    let data = json!({
        "execution_id": execution_id_value,
        "trail_path": canonical_path.display().to_string(),
        "entry_count": entry_count,
        "entries": entries_array,
    });
    TrailResult::from_envelope(CliEnvelope::ok("trail", data))
}

/// Read a JSONL file, parse each line as strict JSON, and collect entries
/// whose top-level `execution_id` matches the supplied value.
///
/// Any malformed line, duplicate key, blank line, non-object JSON, or line
/// exceeding `MAX_LINE_BYTES` invalidates the entire inspection.
fn read_and_filter(
    mut reader: impl BufRead,
    execution_id: &str,
) -> Result<Vec<Value>, TrailResult> {
    let mut entries: Vec<Value> = Vec::new();
    let mut line_count: u64 = 0;
    let mut line_buf = String::new();

    loop {
        line_count += 1;
        line_buf.clear();

        let bytes_read = read_line_limited(&mut line_buf, &mut reader, line_count)?;
        if bytes_read == 0 {
            break; // EOF
        }

        // Strip trailing CR for CRLF files, then check if line is empty.
        let mut line = line_buf.as_str();
        line = line.strip_suffix('\r').unwrap_or(line);
        line = line.strip_suffix('\n').unwrap_or(line);

        if line.is_empty() {
            return Err(invalid_trail(line_count));
        }

        // Strict JSON parsing with duplicate-key rejection.
        let value = match manifest::parse_value_no_dupes(line) {
            Ok(v) => v,
            Err(_) => return Err(invalid_trail(line_count)),
        };

        // Must be a JSON object.
        match value {
            Value::Object(obj) => {
                // Check top-level execution_id field.
                match obj.get("execution_id") {
                    Some(Value::String(id)) if id == execution_id => {
                        entries.push(Value::Object(obj));
                    }
                    Some(Value::String(_)) => {
                        // Different execution ID: skip silently.
                    }
                    Some(_) => {
                        // Non-string execution_id: invalid.
                        return Err(invalid_trail(line_count));
                    }
                    None => {
                        // No execution_id: skip silently.
                    }
                }
            }
            _ => return Err(invalid_trail(line_count)), // non-object
        }
    }

    Ok(entries)
}

/// Read one logical line, enforcing the 8 MiB size limit.
/// Returns the number of bytes read (including line terminator).
fn read_line_limited(
    buf: &mut String,
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
                    None,
                ));
            }
        };
        if available.is_empty() {
            return Ok(total as usize);
        }

        // Find newline in the buffer.
        let consume = match available.iter().position(|&b| b == b'\n') {
            Some(pos) => pos + 1, // Include the LF
            None => available.len(),
        };

        let chunk_len = consume as u64;
        if total + chunk_len > MAX_LINE_BYTES {
            return Err(invalid_trail(line_number));
        }

        // UTF-8 validation.
        let chunk = &available[..consume];
        if std::str::from_utf8(chunk).is_err() {
            return Err(invalid_trail(line_number));
        }

        // SAFETY: validated as UTF-8 above.
        buf.push_str(unsafe { std::str::from_utf8_unchecked(chunk) });

        total += chunk_len;

        let found_newline = chunk.last() == Some(&b'\n');
        reader.consume(consume);

        if found_newline {
            break;
        }
    }
    Ok(total as usize)
}

fn invalid_trail(line_number: u64) -> TrailResult {
    let mut data = serde_json::Map::new();
    data.insert("line".to_owned(), json!(line_number));
    let mut result = failure(
        OutcomeStatus::AuditFailed,
        "TRAIL_INVALID",
        "trail file content is invalid",
        None,
    );
    result.envelope.data = Value::Object(data);
    result
}

fn failure(
    status: OutcomeStatus,
    code: impl Into<String>,
    message: impl Into<String>,
    field: Option<String>,
) -> TrailResult {
    TrailResult::from_envelope(CliEnvelope::error("trail", status, code, message, field))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    // Helper: build a JSONL string from a slice of JSON values.
    fn jsonl(entries: &[Value]) -> String {
        entries
            .iter()
            .map(|v| serde_json::to_string(v).unwrap())
            .collect::<Vec<_>>()
            .join("\n")
            + "\n"
    }

    fn entry_with_id(execution_id: &str, extra: &str) -> Value {
        json!({"execution_id": execution_id, "kind": "test", "extra": extra})
    }

    fn entry_no_id(extra: &str) -> Value {
        json!({"kind": "audit", "extra": extra})
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
            entry_with_id(other, "1"),
            entry_with_id(target, "2"),
            entry_no_id("3"),
            entry_with_id(target, "4"),
            entry_with_id(other, "5"),
            entry_with_id(target, "6"),
        ]);
        let reader = BufReader::new(Cursor::new(input.as_bytes()));
        let entries = read_and_filter(reader, target).unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0]["extra"], "2");
        assert_eq!(entries[1]["extra"], "4");
        assert_eq!(entries[2]["extra"], "6");
    }

    #[test]
    fn j13c_unrelated_execution_ids_omitted() {
        let target = "exec_00000000-0000-4000-8000-000000000000";
        let other = "exec_00000000-0000-4000-8000-000000000001";
        let input = jsonl(&[entry_with_id(other, "1"), entry_with_id(other, "2")]);
        let reader = BufReader::new(Cursor::new(input.as_bytes()));
        let entries = read_and_filter(reader, target).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn j13c_valid_audit_entries_without_execution_id_skipped() {
        let target = "exec_00000000-0000-4000-8000-000000000000";
        let input = jsonl(&[
            entry_no_id("a"),
            entry_with_id(target, "b"),
            entry_no_id("c"),
        ]);
        let reader = BufReader::new(Cursor::new(input.as_bytes()));
        let entries = read_and_filter(reader, target).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0]["extra"], "b");
    }

    #[test]
    fn j13c_zero_matches_maps_to_not_found() {
        let tmp =
            std::env::temp_dir().join(format!("j13c-zero-match-{}.jsonl", uuid::Uuid::new_v4()));
        let content = jsonl(&[entry_with_id(
            "exec_00000000-0000-4000-8000-000000000001",
            "x",
        )]);
        fs::write(&tmp, &content).unwrap();
        let result = run_trail(&tmp, "exec_00000000-0000-4000-8000-000000000000");
        assert_eq!(result.envelope.status, OutcomeStatus::NotFound);
        assert_eq!(result.exit_code, 9);
        assert_eq!(
            result.envelope.error.as_ref().unwrap().code,
            "EXECUTION_NOT_FOUND"
        );
        assert_eq!(result.envelope.data["entry_count"], 0);
        assert!(!result
            .envelope
            .data
            .as_object()
            .unwrap()
            .contains_key("entries"));
        let _ = fs::remove_file(&tmp);
    }

    #[test]
    fn j13c_relative_trail_path_rejected() {
        let result = run_trail(
            Path::new("relative/path.jsonl"),
            "exec_00000000-0000-4000-8000-000000000000",
        );
        assert_eq!(result.envelope.status, OutcomeStatus::InvalidData);
        assert_eq!(result.exit_code, 3);
        assert_eq!(
            result.envelope.error.as_ref().unwrap().code,
            "TRAIL_NOT_ABSOLUTE"
        );
    }

    #[test]
    fn j13c_missing_trail_file_maps_to_not_found() {
        let missing = Path::new("C:\\does-not-exist-j13c-test.jsonl");
        let result = run_trail(missing, "exec_00000000-0000-4000-8000-000000000000");
        assert_eq!(result.envelope.status, OutcomeStatus::NotFound);
        assert_eq!(result.exit_code, 9);
        assert_eq!(
            result.envelope.error.as_ref().unwrap().code,
            "TRAIL_NOT_FOUND"
        );
    }

    #[test]
    fn j13c_directory_path_rejected() {
        let dir = std::env::temp_dir();
        let result = run_trail(&dir, "exec_00000000-0000-4000-8000-000000000000");
        assert_eq!(result.envelope.status, OutcomeStatus::InvalidData);
        assert_eq!(result.exit_code, 3);
        assert_eq!(
            result.envelope.error.as_ref().unwrap().code,
            "TRAIL_NOT_FILE"
        );
    }

    #[test]
    fn j13c_malformed_json_maps_to_audit_failed() {
        let target = "exec_00000000-0000-4000-8000-000000000000";
        let input =
            "{\"execution_id\": \"exec_00000000-0000-4000-8000-000000000000\"}\n{not json\n";
        let reader = BufReader::new(Cursor::new(input.as_bytes()));
        let result = read_and_filter(reader, target);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().envelope.status,
            OutcomeStatus::AuditFailed
        );
    }

    #[test]
    fn j13c_duplicate_json_key_maps_to_audit_failed() {
        let target = "exec_00000000-0000-4000-8000-000000000000";
        let input = format!("{{\"execution_id\": \"{target}\", \"execution_id\": \"{target}\"}}\n");
        let reader = BufReader::new(Cursor::new(input.as_bytes()));
        let result = read_and_filter(reader, target);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().envelope.status,
            OutcomeStatus::AuditFailed
        );
    }

    #[test]
    fn j13c_blank_line_maps_to_audit_failed() {
        let target = "exec_00000000-0000-4000-8000-000000000000";
        let input = format!("{{\"execution_id\": \"{target}\"}}\n\n");
        let reader = BufReader::new(Cursor::new(input.as_bytes()));
        let result = read_and_filter(reader, target);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().envelope.status,
            OutcomeStatus::AuditFailed
        );
    }

    #[test]
    fn j13c_non_object_json_maps_to_audit_failed() {
        let target = "exec_00000000-0000-4000-8000-000000000000";
        let input = "\"just a string\"\n";
        let reader = BufReader::new(Cursor::new(input.as_bytes()));
        let result = read_and_filter(reader, target);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().envelope.status,
            OutcomeStatus::AuditFailed
        );
    }

    #[test]
    fn j13c_non_string_execution_id_maps_to_audit_failed() {
        let target = "exec_00000000-0000-4000-8000-000000000000";
        let input = "{\"execution_id\": 42}\n";
        let reader = BufReader::new(Cursor::new(input.as_bytes()));
        let result = read_and_filter(reader, target);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().envelope.status,
            OutcomeStatus::AuditFailed
        );
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
        assert_eq!(
            result.unwrap_err().envelope.status,
            OutcomeStatus::AuditFailed
        );
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
        assert_eq!(lf_entries[0]["execution_id"], target);
        assert_eq!(crlf_entries[0]["execution_id"], target);
    }

    #[test]
    fn j13c_success_envelope_has_no_timestamp() {
        let tmp = std::env::temp_dir().join(format!("j13c-no-ts-{}.jsonl", uuid::Uuid::new_v4()));
        let target = "exec_00000000-0000-4000-8000-000000000000";
        let content = format!("{{\"execution_id\": \"{target}\"}}\n");
        fs::write(&tmp, &content).unwrap();
        let result = run_trail(&tmp, target);
        let json_str = serde_json::to_string(&result.envelope).unwrap();
        assert!(!json_str.contains("timestamp"));
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
        assert_eq!(result.envelope.data["entry_count"], 0);
        assert!(!result
            .envelope
            .data
            .as_object()
            .unwrap()
            .contains_key("entries"));
        let _ = fs::remove_file(&tmp);
    }

    #[test]
    fn j13c_no_partial_success_when_later_line_malformed() {
        let target = "exec_00000000-0000-4000-8000-000000000000";
        let input = format!("{{\"execution_id\": \"{target}\"}}\n{{bad json\n");
        let reader = BufReader::new(Cursor::new(input.as_bytes()));
        let result = read_and_filter(reader, target);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().envelope.status,
            OutcomeStatus::AuditFailed
        );
    }

    #[test]
    fn j13c_execution_id_validated_before_file_access() {
        // Use a path that doesn't exist, but with an invalid execution ID.
        // The execution-ID error should take precedence.
        let missing = Path::new("C:\\does-not-exist-j13c-precedence.jsonl");
        let result = run_trail(missing, "not-a-valid-exec-id");
        // Even though file doesn't exist, execution-ID format fails first.
        assert_eq!(result.envelope.status, OutcomeStatus::InvalidData);
        assert_eq!(result.exit_code, 3);
        assert_eq!(
            result.envelope.error.as_ref().unwrap().code,
            "EXECUTION_ID_INVALID"
        );
        assert_eq!(
            result.envelope.error.as_ref().unwrap().field.as_deref(),
            Some("--execution-id")
        );
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
        assert_eq!(result.envelope.schema, "tethers.cli/1");
        assert_eq!(result.envelope.command, "trail");
        assert_eq!(result.envelope.status, OutcomeStatus::Ok);
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.envelope.data["entry_count"], 2);
        assert_eq!(result.envelope.data["entries"].as_array().unwrap().len(), 2);
        assert!(result.envelope.error.is_none());
        let _ = fs::remove_file(&tmp);
    }
}
