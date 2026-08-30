//! Tethers Portable 0.1 decision boundary.
//!
//! This is deliberately an additive host-facing façade.  It reuses the
//! current Tethers host policy identity (`name` + `version`) and does not
//! alter the OCaml Core language, parser, evaluator, or public `run` route.
//! It proposes a deterministic policy decision; it never executes an Action.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashSet;
use std::fmt;

const PORTABLE_VERSION: &str = "0.1";

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub action: Action,
    pub context: Map<String, Value>,
    #[serde(default)]
    pub policy: Option<Policy>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Action {
    pub name: String,
    pub version: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub default: PolicyDecision,
    pub rules: Vec<PolicyRule>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyRule {
    pub name: String,
    pub version: u32,
    pub decision: PolicyDecision,
    #[serde(default)]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PolicyDecision {
    Allow,
    Ask,
    Deny,
}

impl PolicyDecision {
    fn as_str(self) -> &'static str {
        match self {
            Self::Allow => "ALLOW",
            Self::Ask => "ASK",
            Self::Deny => "DENY",
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Response {
    pub decision: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl Response {
    fn decision(decision: PolicyDecision, rule: String, reason: String) -> Self {
        Self {
            decision: decision.as_str(),
            rule: Some(rule),
            reason: Some(reason),
            error: None,
        }
    }

    fn deny_error(error: impl Into<String>) -> Self {
        Self {
            decision: "DENY",
            rule: None,
            reason: None,
            error: Some(error.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputError(String);

impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for InputError {}

/// Parse and evaluate one portable request.  Every error is represented as a
/// DENY response so callers never need to infer authority from process failure.
pub fn evaluate_text(input: &str, external_policy: Option<&str>) -> Response {
    let request: Request = match serde_json::from_str(input) {
        Ok(request) => request,
        Err(error) => return Response::deny_error(format!("invalid request JSON: {error}")),
    };

    evaluate_request(&request, external_policy)
}

/// Evaluate one already-decoded request.  `external_policy`, when present,
/// replaces the embedded policy and is parsed with the same strict schema.
pub fn evaluate_request(request: &Request, external_policy: Option<&str>) -> Response {
    if let Err(error) = validate_action(&request.action) {
        return Response::deny_error(error);
    }

    let policy = match external_policy {
        Some(policy_text) => match serde_json::from_str::<Policy>(policy_text) {
            Ok(policy) => policy,
            Err(error) => {
                return Response::deny_error(format!("invalid policy JSON: {error}"));
            }
        },
        None => match request.policy.clone() {
            Some(policy) => policy,
            None => return Response::deny_error("missing policy"),
        },
    };

    if let Err(error) = validate_policy(&policy) {
        return Response::deny_error(error);
    }

    // Context is intentionally opaque at this seam.  The current Tethers
    // host owns fact interpretation and execution; this façade only resolves
    // the exact host-local policy identity supplied by the caller.
    let _context = &request.context;
    let identity = format!("{}@{}", request.action.name, request.action.version);
    if let Some(rule) = policy
        .rules
        .iter()
        .find(|rule| rule.name == request.action.name && rule.version == request.action.version)
    {
        let reason = rule
            .reason
            .clone()
            .unwrap_or_else(|| format!("matched policy rule {identity}"));
        return Response::decision(rule.decision, identity, reason);
    }

    Response::decision(
        policy.default,
        "default".to_owned(),
        format!("used policy default for {identity}"),
    )
}

fn validate_action(action: &Action) -> Result<(), String> {
    if action.name.trim().is_empty() {
        return Err("invalid action: name must be non-empty".to_owned());
    }
    if action.name.chars().any(char::is_whitespace) {
        return Err("invalid action: name must not contain whitespace".to_owned());
    }
    if action.version == 0 {
        return Err("invalid action: version must be greater than zero".to_owned());
    }
    Ok(())
}

fn validate_policy(policy: &Policy) -> Result<(), String> {
    let mut identities = HashSet::new();
    for rule in &policy.rules {
        if rule.name.trim().is_empty() || rule.name.chars().any(char::is_whitespace) {
            return Err(
                "invalid policy: rule name must be non-empty and contain no whitespace".to_owned(),
            );
        }
        if rule.version == 0 {
            return Err("invalid policy: rule version must be greater than zero".to_owned());
        }
        let identity = (rule.name.as_str(), rule.version);
        if !identities.insert(identity) {
            return Err(format!(
                "invalid policy: duplicate rule {}@{}",
                rule.name, rule.version
            ));
        }
    }
    Ok(())
}

pub fn response_json(response: &Response) -> String {
    // Response contains only infallible JSON scalar values, so this is a
    // defensive fallback rather than a reachable evaluator failure.
    serde_json::to_string(response).unwrap_or_else(|_| {
        "{\"decision\":\"DENY\",\"error\":\"response serialization failed\"}".to_owned()
    })
}

pub fn version() -> &'static str {
    PORTABLE_VERSION
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(policy: &str) -> String {
        format!(
            r#"{{"action":{{"name":"git.push","version":1}},"context":{{"branch":"main"}},"policy":{policy}}}"#
        )
    }

    #[test]
    fn allow_rule_is_deterministic() {
        let input = request(
            r#"{"default":"deny","rules":[{"name":"git.push","version":1,"decision":"allow","reason":"trusted branch"}]}"#,
        );
        let response = evaluate_text(&input, None);
        assert_eq!(response.decision, "ALLOW");
        assert_eq!(response.rule.as_deref(), Some("git.push@1"));
        assert_eq!(
            response_json(&response),
            response_json(&evaluate_text(&input, None))
        );
    }

    #[test]
    fn ask_rule_is_returned_without_execution() {
        let input = request(
            r#"{"default":"deny","rules":[{"name":"git.push","version":1,"decision":"ask"}]}"#,
        );
        assert_eq!(evaluate_text(&input, None).decision, "ASK");
    }

    #[test]
    fn deny_rule_is_returned_as_a_valid_decision() {
        let input = request(
            r#"{"default":"allow","rules":[{"name":"git.push","version":1,"decision":"deny"}]}"#,
        );
        assert_eq!(evaluate_text(&input, None).decision, "DENY");
    }

    #[test]
    fn malformed_request_fails_closed() {
        let response = evaluate_text("{", None);
        assert_eq!(response.decision, "DENY");
        assert!(response.error.unwrap().starts_with("invalid request JSON:"));
    }

    #[test]
    fn missing_policy_fails_closed() {
        let response = evaluate_text(
            r#"{"action":{"name":"git.push","version":1},"context":{}}"#,
            None,
        );
        assert_eq!(response.decision, "DENY");
        assert_eq!(response.error.as_deref(), Some("missing policy"));
    }

    #[test]
    fn bad_policy_fails_closed() {
        let input = request(
            r#"{"default":"deny","rules":[{"name":"git.push","version":1,"decision":"allow"},{"name":"git.push","version":1,"decision":"ask"}]}"#,
        );
        let response = evaluate_text(&input, None);
        assert_eq!(response.decision, "DENY");
        assert!(response.error.unwrap().contains("duplicate rule"));
    }

    #[test]
    fn unknown_condition_or_field_fails_closed() {
        let input = request(r#"{"default":"allow","rules":[],"when":{"branch":"main"}}"#);
        let response = evaluate_text(&input, None);
        assert_eq!(response.decision, "DENY");
        assert!(response.error.unwrap().contains("unknown field"));
    }

    #[test]
    fn evaluator_error_fails_closed_for_invalid_action() {
        let input = request(r#"{"default":"allow","rules":[]}"#).replace("git.push", "bad action");
        let response = evaluate_text(&input, None);
        assert_eq!(response.decision, "DENY");
        assert!(response.error.unwrap().contains("whitespace"));
    }

    #[test]
    fn external_policy_overrides_embedded_policy() {
        let input = request(r#"{"default":"allow","rules":[]}"#);
        let external = r#"{"default":"ask","rules":[]}"#;
        assert_eq!(evaluate_text(&input, Some(external)).decision, "ASK");
    }
}
