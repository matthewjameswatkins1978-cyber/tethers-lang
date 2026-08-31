use serde::Deserialize;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    Allow,
    Ask,
    Deny,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evaluation {
    pub decision: Decision,
    pub rule: Option<String>,
    pub reason: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WireResponse {
    schema_version: Option<String>,
    decision: String,
    rule: Option<String>,
    matched_rule: Option<String>,
    reason: Option<String>,
    error: Option<String>,
}

pub fn evaluate(binary: impl AsRef<Path>, request_json: &str, timeout: Duration) -> Evaluation {
    let mut child = match Command::new(binary.as_ref())
        .arg("evaluate")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(error) => return denied(format!("cannot start Tethers: {error}")),
    };
    if let Some(mut stdin) = child.stdin.take() {
        if let Err(error) = stdin.write_all(request_json.as_bytes()) {
            let _ = child.kill();
            return denied(format!("cannot send request: {error}"));
        }
    }
    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let output = match child.wait_with_output() {
                    Ok(output) => output,
                    Err(error) => return denied(format!("cannot collect Tethers output: {error}")),
                };
                if !status.success() {
                    return denied(format!("Tethers exited with {status}"));
                }
                return parse_response(&output.stdout);
            }
            Ok(None) if started.elapsed() < timeout => std::thread::sleep(Duration::from_millis(5)),
            Ok(None) => {
                let _ = child.kill();
                return denied("Tethers evaluation timed out".to_owned());
            }
            Err(error) => {
                let _ = child.kill();
                return denied(format!("cannot poll Tethers: {error}"));
            }
        }
    }
}

fn parse_response(bytes: &[u8]) -> Evaluation {
    let response: WireResponse = match serde_json::from_slice(bytes) {
        Ok(value) => value,
        Err(error) => return denied(format!("invalid Tethers response: {error}")),
    };
    if response.schema_version.as_deref() != Some("1") {
        return denied("Tethers response schema mismatch".to_owned());
    }
    let decision = match response.decision.as_str() {
        "ALLOW" => Decision::Allow,
        "ASK" => Decision::Ask,
        "DENY" => Decision::Deny,
        _ => return denied("unknown Tethers decision".to_owned()),
    };
    Evaluation {
        decision,
        rule: response.matched_rule.or(response.rule),
        reason: response.reason,
        error: response.error,
    }
}
fn denied(error: String) -> Evaluation {
    Evaluation {
        decision: Decision::Deny,
        rule: None,
        reason: None,
        error: Some(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_binary_fails_closed() {
        assert_eq!(
            evaluate(
                "definitely-missing-tethers",
                "{}",
                Duration::from_millis(10)
            )
            .decision,
            Decision::Deny
        );
    }
    #[test]
    fn malformed_response_fails_closed() {
        assert_eq!(parse_response(b"not json").decision, Decision::Deny);
    }
    #[test]
    fn schema_mismatch_fails_closed() {
        assert_eq!(
            parse_response(br#"{"schema_version":"2","decision":"ALLOW"}"#).decision,
            Decision::Deny
        );
    }
}
