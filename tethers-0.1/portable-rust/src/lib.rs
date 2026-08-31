//! Tethers Portable: a deterministic, fail-closed authority decision façade.
//!
//! The evaluator decides authority. It never performs the requested action,
//! calls an LLM, reads live state, or starts a service. The legacy 0.1 JSON
//! form remains accepted so the 0.2 workbench surface is additive.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fmt;

const PORTABLE_VERSION: &str = "0.2.1";
const SCHEMA_VERSION: &str = "1";

pub const KNOWN_ACTIONS: &[&str] = &[
    "filesystem.read",
    "filesystem.write",
    "filesystem.create",
    "filesystem.rename",
    "filesystem.delete",
    "filesystem.write_outside_workspace",
    "process.execute",
    "process.execute_shell",
    "process.kill",
    "process.elevated",
    "git.status",
    "git.diff",
    "git.commit",
    "git.branch",
    "git.push",
    "git.force_push",
    "git.merge",
    "git.tag",
    "git.reset_destructive",
    "network.http_get",
    "network.http_post",
    "network.download",
    "network.upload",
    "network.resolve",
    "network.connect",
    "network.http_head",
    "network.http_put",
    "network.http_patch",
    "network.http_delete",
    "network.websocket",
    "network.arbitrary",
    "secret.exists",
    "secret.read",
    "secret.use",
    "secret.write",
    "secret.export",
    "secret.expose",
    "container.build",
    "container.run",
    "container.exec",
    "container.stop",
    "container.remove",
    "container.image.pull",
    "container.image.push",
    "container.network",
    "container.mount",
    "container.mount_host",
    "container.privileged",
    "tool.call",
    "mcp.tool_call",
    "database.connect",
    "deploy.preview",
    "deploy.staging",
    "deploy.production",
    "message.draft",
    "message.send",
    "email.draft",
    "email.send",
    "database.read",
    "database.write",
    "database.insert",
    "database.update",
    "database.query_read",
    "database.bulk_write",
    "database.schema_read",
    "database.delete",
    "database.drop",
    "database.schema_change",
    "purchase.prepare",
    "purchase.execute",
    "subscription.change",
    "payment.execute",
    "message.read",
    "message.delete",
    "email.read",
    "email.forward",
    "email.delete",
    "deploy.inspect",
    "deploy.build",
    "deploy.rollback",
    "deploy.destroy",
    "filesystem.list",
    "filesystem.stat",
    "filesystem.append",
    "filesystem.move",
    "filesystem.copy",
    "filesystem.mkdir",
    "filesystem.rmdir",
    "filesystem.chmod",
    "filesystem.permission_change",
    "filesystem.outside_workspace",
    "process.spawn",
    "process.background",
    "shell.execute",
    "shell.pipeline",
    "shell.redirect",
    "shell.destructive",
    "git.log",
    "git.show",
    "git.fetch",
    "git.branch.read",
    "git.branch.create",
    "git.checkout",
    "git.switch",
    "git.add",
    "git.pull",
    "git.rebase",
    "git.reset",
    "git.clean",
    "git.remote.add",
    "git.remote.change",
    "workspace.read",
    "apply_patch",
    "test.run",
];

const HARD_DENY_ACTIONS: &[&str] = &[
    "filesystem.delete",
    "filesystem.write_outside_workspace",
    "filesystem.outside_workspace",
    "filesystem.rmdir",
    "process.elevated",
    "shell.destructive",
    "git.force_push",
    "git.reset_destructive",
    "secret.read",
    "secret.write",
    "secret.export",
    "secret.expose",
    "deploy.production",
    "database.delete",
    "database.schema_change",
    "database.drop",
    "deploy.destroy",
    "container.privileged",
    "container.mount_host",
    "payment.execute",
    "purchase.execute",
];

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    #[serde(default)]
    pub schema_version: Option<String>,
    #[serde(default)]
    pub actor: Option<String>,
    pub action: ActionInput,
    #[serde(default)]
    pub resource: Option<String>,
    pub context: Map<String, Value>,
    #[serde(default)]
    pub policy: Option<Policy>,
    #[serde(default)]
    pub scope: Option<Scope>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum ActionInput {
    Name(String),
    Legacy(LegacyAction),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LegacyAction {
    pub name: String,
    pub version: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    #[serde(default)]
    pub allowed_files: Option<Vec<String>>,
    #[serde(default)]
    pub allowed_actions: Option<Vec<String>>,
    #[serde(default)]
    pub workspace_root: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub default: PolicyDecision,
    pub rules: Vec<PolicyRule>,
    #[serde(default)]
    pub schema_version: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub policy_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyRule {
    pub name: String,
    #[serde(default = "one")]
    pub version: u32,
    pub decision: PolicyDecision,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub action: Option<String>,
    #[serde(default)]
    pub actions: Vec<String>,
    #[serde(default)]
    pub actors: Vec<String>,
    #[serde(default)]
    pub resources: Vec<String>,
    #[serde(default)]
    pub conditions: Vec<Condition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Condition {
    pub field: String,
    #[serde(default)]
    pub equals: Option<Value>,
    #[serde(default, rename = "in")]
    pub one_of: Option<Vec<Value>>,
    #[serde(default)]
    pub exists: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub schema_version: String,
    pub tool: String,
    pub version: String,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
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
pub struct EvaluatedCondition {
    pub condition: String,
    pub result: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Response {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_version: Option<String>,
    pub decision: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched_rule: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evaluated_conditions: Option<Vec<EvaluatedCondition>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tethers_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputError(String);
impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for InputError {}
fn one() -> u32 {
    1
}

impl Response {
    fn decision(
        protocol: bool,
        decision: PolicyDecision,
        rule: String,
        reason: String,
        policy: &Policy,
        conditions: Vec<EvaluatedCondition>,
        trace: Vec<String>,
    ) -> Self {
        let policy_sha256 = policy_fingerprint(policy);
        let decision_id = decision_id(&policy_sha256, decision.as_str(), &rule);
        Self {
            schema_version: protocol.then(|| SCHEMA_VERSION.to_owned()),
            decision: decision.as_str(),
            rule: Some(rule.clone()),
            matched_rule: Some(rule),
            reason: Some(reason),
            policy: policy.name.clone(),
            policy_version: policy.policy_version.clone(),
            evaluated_conditions: (!conditions.is_empty()).then_some(conditions),
            error: None,
            tethers_version: Some(PORTABLE_VERSION.to_owned()),
            decision_id: Some(decision_id),
            policy_sha256: Some(policy_sha256),
            trace: (!trace.is_empty()).then_some(trace),
        }
    }
    pub fn deny_error(error: impl Into<String>) -> Self {
        Self {
            schema_version: Some(SCHEMA_VERSION.to_owned()),
            decision: "DENY",
            rule: None,
            matched_rule: None,
            reason: None,
            policy: None,
            policy_version: None,
            evaluated_conditions: None,
            error: Some(error.into()),
            tethers_version: Some(PORTABLE_VERSION.to_owned()),
            decision_id: None,
            policy_sha256: None,
            trace: None,
        }
    }
    pub fn deny_error_for_cli(error: impl Into<String>) -> Self {
        Self::deny_error(error)
    }
}

fn policy_fingerprint(policy: &Policy) -> String {
    let bytes = serde_json::to_vec(policy).unwrap_or_default();
    format!("{:x}", Sha256::digest(bytes))
}

fn decision_id(policy_sha256: &str, decision: &str, rule: &str) -> String {
    format!(
        "{:x}",
        Sha256::digest(format!("{policy_sha256}|{decision}|{rule}").as_bytes())
    )
}

/// Backwards-compatible 0.1 entry point.
pub fn evaluate_text(input: &str, external_policy: Option<&str>) -> Response {
    evaluate_text_with_options(input, external_policy, None, false)
}

pub fn evaluate_text_with_options(
    input: &str,
    external_policy: Option<&str>,
    manifest: Option<&str>,
    explain: bool,
) -> Response {
    evaluate_text_with_config(input, external_policy, manifest, explain, false)
}

/// Evaluates a request with optional human-readable condition explanation and
/// a redacted decision trace. Trace entries never contain context values.
pub fn evaluate_text_with_config(
    input: &str,
    external_policy: Option<&str>,
    manifest: Option<&str>,
    explain: bool,
    trace: bool,
) -> Response {
    let request: Request = match serde_json::from_str(input) {
        Ok(request) => request,
        Err(error) => return Response::deny_error(format!("invalid request JSON: {error}")),
    };
    evaluate_request_with_config(&request, external_policy, manifest, explain, trace)
}

pub fn evaluate_request(request: &Request, external_policy: Option<&str>) -> Response {
    evaluate_request_with_options(request, external_policy, None, false)
}

pub fn evaluate_request_with_options(
    request: &Request,
    external_policy: Option<&str>,
    manifest_text: Option<&str>,
    explain: bool,
) -> Response {
    evaluate_request_with_config(request, external_policy, manifest_text, explain, false)
}

pub fn evaluate_request_with_config(
    request: &Request,
    external_policy: Option<&str>,
    manifest_text: Option<&str>,
    _explain: bool,
    trace_enabled: bool,
) -> Response {
    let protocol = request.schema_version.is_some();
    if let Some(version) = &request.schema_version {
        if version != SCHEMA_VERSION {
            return Response::deny_error(format!("unsupported schema_version: {version}"));
        }
    }
    let (action, legacy_version) = match &request.action {
        ActionInput::Name(name) => (name.clone(), None),
        ActionInput::Legacy(action) => (action.name.clone(), Some(action.version)),
    };
    if let Err(error) = validate_action(&action, legacy_version, protocol) {
        return Response::deny_error(error);
    }
    if protocol
        && request
            .actor
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .is_none()
    {
        return Response::deny_error("missing actor");
    }
    if protocol
        && request
            .resource
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .is_none()
    {
        return Response::deny_error("missing resource");
    }
    let policy = match external_policy {
        Some(text) => match serde_json::from_str::<Policy>(text) {
            Ok(policy) => policy,
            Err(error) => return Response::deny_error(format!("invalid policy JSON: {error}")),
        },
        None => match request.policy.clone() {
            Some(policy) => policy,
            None => return Response::deny_error("missing policy"),
        },
    };
    if let Err(error) = validate_policy(&policy) {
        return Response::deny_error(error);
    }
    if let Some(text) = manifest_text {
        let manifest: Manifest = match serde_json::from_str(text) {
            Ok(manifest) => manifest,
            Err(error) => return Response::deny_error(format!("invalid manifest JSON: {error}")),
        };
        if let Err(error) = validate_manifest(&manifest) {
            return Response::deny_error(error);
        }
        if !manifest
            .capabilities
            .iter()
            .any(|capability| capability == &action)
        {
            return Response::deny_error(format!("manifest does not declare action: {action}"));
        }
    }
    if protocol && !KNOWN_ACTIONS.contains(&action.as_str()) {
        return Response::deny_error(format!("unknown action: {action}"));
    }
    if let Err(error) = validate_scope(request, &action) {
        return Response::deny_error(error);
    }
    if HARD_DENY_ACTIONS.contains(&action.as_str()) {
        return Response::decision(
            protocol,
            PolicyDecision::Deny,
            "global_hard_deny".to_owned(),
            "this high-risk action is globally prohibited".to_owned(),
            &policy,
            Vec::new(),
            vec![format!("global_hard_deny: {action}")],
        );
    }
    let mut selected: Option<(&PolicyRule, Vec<EvaluatedCondition>)> = None;
    let mut trace = Vec::new();
    for rule in &policy.rules {
        if !rule_applies(rule, &action, request) {
            if trace_enabled {
                trace.push(format!("rule {}: not applicable", rule.name));
            }
            continue;
        }
        if trace_enabled {
            trace.push(format!("rule {}: applicable", rule.name));
        }
        let evaluated: Vec<_> = rule
            .conditions
            .iter()
            .map(|condition| EvaluatedCondition {
                condition: describe_condition(condition),
                result: condition_matches(condition, request, &action),
            })
            .collect();
        if trace_enabled {
            for condition in &evaluated {
                trace.push(format!(
                    "rule {} condition {}: {}",
                    rule.name,
                    condition.condition,
                    if condition.result {
                        "matched"
                    } else {
                        "not matched"
                    }
                ));
            }
        }
        if evaluated.iter().all(|condition| condition.result) {
            selected = Some((rule, evaluated));
            break;
        }
    }
    match selected {
        Some((rule, conditions)) => Response::decision(
            protocol,
            rule.decision,
            rule.name.clone(),
            rule.reason
                .clone()
                .unwrap_or_else(|| format!("matched rule {}", rule.name)),
            &policy,
            conditions,
            trace,
        ),
        None => Response::decision(
            protocol,
            policy.default,
            "default".to_owned(),
            format!("used policy default for {action}"),
            &policy,
            Vec::new(),
            trace,
        ),
    }
}

fn validate_action(
    action: &str,
    legacy_version: Option<u32>,
    protocol: bool,
) -> Result<(), String> {
    if action.trim().is_empty() {
        return Err("invalid action: name must be non-empty".to_owned());
    }
    if action.chars().any(char::is_whitespace) {
        return Err("invalid action: name must not contain whitespace".to_owned());
    }
    if legacy_version == Some(0) {
        return Err("invalid action: version must be greater than zero".to_owned());
    }
    if protocol
        && !action
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
    {
        return Err("invalid action: unsupported characters".to_owned());
    }
    Ok(())
}

fn validate_policy(policy: &Policy) -> Result<(), String> {
    if let Some(version) = &policy.schema_version {
        if version != SCHEMA_VERSION {
            return Err(format!("unsupported policy schema_version: {version}"));
        }
    }
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
        if !identities.insert((rule.name.as_str(), rule.version)) {
            return Err(format!(
                "invalid policy: duplicate rule {}@{}",
                rule.name, rule.version
            ));
        }
        for action in rule.actions.iter().chain(rule.action.iter()) {
            if action.trim().is_empty() || action.chars().any(char::is_whitespace) {
                return Err("invalid policy: invalid action selector".to_owned());
            }
        }
        for condition in &rule.conditions {
            let comparators = condition.equals.is_some() as u8
                + condition.one_of.is_some() as u8
                + condition.exists.is_some() as u8;
            if condition.field.trim().is_empty() || comparators != 1 {
                return Err(
                    "invalid policy: each condition needs exactly one comparator".to_owned(),
                );
            }
            if !matches!(
                condition.field.as_str(),
                "actor" | "action" | "resource" | "scope.allowed_actions"
            ) && !condition.field.starts_with("context.")
            {
                return Err(format!(
                    "invalid policy: unknown condition field {}",
                    condition.field
                ));
            }
        }
    }
    Ok(())
}

pub fn validate_policy_text(text: &str) -> Result<(), String> {
    let policy: Policy =
        serde_json::from_str(text).map_err(|error| format!("invalid policy JSON: {error}"))?;
    validate_policy(&policy)
}

pub fn validate_manifest(manifest: &Manifest) -> Result<(), String> {
    if manifest.schema_version != SCHEMA_VERSION {
        return Err(format!(
            "unsupported manifest schema_version: {}",
            manifest.schema_version
        ));
    }
    if manifest.tool.trim().is_empty() || manifest.version.trim().is_empty() {
        return Err("invalid manifest: tool and version are required".to_owned());
    }
    let mut seen = HashSet::new();
    for capability in &manifest.capabilities {
        if !KNOWN_ACTIONS.contains(&capability.as_str()) {
            return Err(format!("unknown capability: {capability}"));
        }
        if !seen.insert(capability) {
            return Err(format!("duplicate capability: {capability}"));
        }
    }
    Ok(())
}

fn rule_applies(rule: &PolicyRule, action: &str, request: &Request) -> bool {
    let action_matches = if rule.actions.is_empty() && rule.action.is_none() {
        rule.name == action
    } else {
        rule.action.as_deref() == Some(action)
            || rule.actions.iter().any(|candidate| candidate == action)
    };
    let actor_matches = rule.actors.is_empty()
        || request
            .actor
            .as_deref()
            .map(|actor| {
                rule.actors
                    .iter()
                    .any(|candidate| candidate == "*" || candidate == actor)
            })
            .unwrap_or(false);
    let resource_matches = rule.resources.is_empty()
        || request
            .resource
            .as_deref()
            .map(|resource| {
                rule.resources
                    .iter()
                    .any(|candidate| candidate == "*" || candidate == resource)
            })
            .unwrap_or(false);
    action_matches && actor_matches && resource_matches
}

fn lookup(request: &Request, action: &str, field: &str) -> Option<Value> {
    match field {
        "actor" => request.actor.clone().map(Value::String),
        "action" => Some(Value::String(action.to_owned())),
        "resource" => request.resource.clone().map(Value::String),
        "scope.allowed_actions" => request
            .scope
            .as_ref()
            .and_then(|scope| scope.allowed_actions.clone())
            .map(|values| Value::Array(values.into_iter().map(Value::String).collect())),
        value if value.starts_with("context.") => request
            .context
            .get(value.trim_start_matches("context."))
            .cloned(),
        _ => None,
    }
}

fn condition_matches(condition: &Condition, request: &Request, action: &str) -> bool {
    let value = lookup(request, action, &condition.field);
    match (&condition.equals, &condition.one_of, condition.exists) {
        (Some(expected), None, None) => value.as_ref() == Some(expected),
        (None, Some(expected), None) => value
            .as_ref()
            .map(|actual| expected.iter().any(|candidate| candidate == actual))
            .unwrap_or(false),
        (None, None, Some(expected)) => value.is_some() == expected,
        _ => false,
    }
}

fn describe_condition(condition: &Condition) -> String {
    if condition.equals.is_some() {
        return format!("{} == <value>", condition.field);
    }
    if let Some(values) = &condition.one_of {
        return format!("{} in array[{}]", condition.field, values.len());
    }
    format!(
        "{} exists == {}",
        condition.field,
        condition.exists.unwrap_or(false)
    )
}

fn validate_scope(request: &Request, action: &str) -> Result<(), String> {
    let Some(scope) = &request.scope else {
        return Ok(());
    };
    if let Some(actions) = &scope.allowed_actions {
        if !actions.iter().any(|allowed| allowed == action) {
            return Err(format!("scope does not allow action: {action}"));
        }
    }
    let file_action = action.starts_with("filesystem.") || action == "apply_patch";
    if file_action {
        let path = request.context.get("path").and_then(Value::as_str);
        if let Some(path) = path {
            let normal = path.replace('\\', "/");
            if normal.split('/').any(|part| part == "..")
                || normal.starts_with('/')
                || normal.contains(':')
            {
                return Err("scope path escapes workspace".to_owned());
            }
            if let Some(files) = &scope.allowed_files {
                if !files
                    .iter()
                    .map(|value| value.replace('\\', "/"))
                    .any(|allowed| allowed == normal)
                {
                    return Err("scope does not allow file".to_owned());
                }
            }
        } else if scope.allowed_files.is_some() {
            return Err("filesystem scope requires context.path".to_owned());
        }
    }
    Ok(())
}

pub fn response_json(response: &Response) -> String {
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
    fn legacy(policy: &str) -> String {
        format!(
            r#"{{"action":{{"name":"git.push","version":1}},"context":{{"branch":"main"}},"policy":{policy}}}"#
        )
    }
    fn modern(action: &str, policy: &str) -> String {
        format!(
            r#"{{"schema_version":"1","actor":"gary.worker","action":"{action}","resource":"workspace","context":{{}},"policy":{policy}}}"#
        )
    }
    fn default_policy(decision: &str) -> String {
        format!(
            r#"{{"schema_version":"1","name":"test","policy_version":"1.0.0","default":"{decision}","rules":[]}}"#
        )
    }
    #[test]
    fn legacy_allow_ask_deny_remain_supported() {
        assert_eq!(evaluate_text(&legacy(r#"{"default":"deny","rules":[{"name":"git.push","version":1,"decision":"allow"}]}"#), None).decision, "ALLOW");
        assert_eq!(evaluate_text(&legacy(r#"{"default":"deny","rules":[{"name":"git.push","version":1,"decision":"ask"}]}"#), None).decision, "ASK");
        assert_eq!(evaluate_text(&legacy(r#"{"default":"allow","rules":[{"name":"git.push","version":1,"decision":"deny"}]}"#), None).decision, "DENY");
    }
    #[test]
    fn modern_protocol_contains_identity_and_rule() {
        let policy = r#"{"schema_version":"1","name":"workbench-default","policy_version":"1.0.0","default":"deny","rules":[{"name":"git.push","decision":"ask","reason":"human approval","conditions":[{"field":"context.branch","equals":"main"}]}]}"#;
        let response = evaluate_text(
            &modern("git.push", policy)
                .replace("\"context\":{}", "\"context\":{\"branch\":\"main\"}"),
            None,
        );
        assert_eq!(response.decision, "ASK");
        assert_eq!(response.matched_rule.as_deref(), Some("git.push"));
        assert!(response.evaluated_conditions.as_ref().unwrap()[0].result);
    }
    #[test]
    fn unknown_modern_action_fails_closed() {
        assert_eq!(
            evaluate_text(&modern("made.up", &default_policy("allow")), None).decision,
            "DENY"
        );
    }
    #[test]
    fn hard_denies_cannot_be_broadened_by_policy() {
        assert_eq!(
            evaluate_text(&modern("git.force_push", &default_policy("allow")), None).decision,
            "DENY"
        );
    }
    #[test]
    fn scope_narrows_action_and_file() {
        let input = r#"{"schema_version":"1","actor":"worker","action":"filesystem.write","resource":"workspace","context":{"path":"src/parser.rs"},"scope":{"allowed_files":["src/parser.rs"],"allowed_actions":["filesystem.write"]},"policy":{"default":"allow","rules":[]}}"#;
        assert_eq!(evaluate_text(input, None).decision, "ALLOW");
        assert_eq!(
            evaluate_text(&input.replacen("src/parser.rs", "README.md", 1), None).decision,
            "DENY"
        );
    }
    #[test]
    fn manifest_is_checked_before_policy_default() {
        let input = modern("git.push", &default_policy("allow"));
        let manifest =
            r#"{"schema_version":"1","tool":"worker","version":"1","capabilities":["git.status"]}"#;
        assert_eq!(
            evaluate_text_with_options(&input, None, Some(manifest), false).decision,
            "DENY"
        );
    }
    #[test]
    fn malformed_and_unknown_schema_fail_closed() {
        assert_eq!(evaluate_text("{", None).decision, "DENY");
        assert_eq!(
            evaluate_text(
                &modern("git.status", &default_policy("allow")).replace("\"1\"", "\"2\""),
                None
            )
            .decision,
            "DENY"
        );
    }
    #[test]
    fn modern_request_without_context_fails_closed() {
        let request = r#"{"schema_version":"1","actor":"worker","action":"git.status","resource":"workspace","policy":{"default":"allow","rules":[]}}"#;
        assert_eq!(evaluate_text(request, None).decision, "DENY");
    }
    #[test]
    fn unknown_condition_field_invalidates_policy() {
        let policy = r#"{"default":"allow","rules":[{"name":"git.status","decision":"allow","conditions":[{"field":"invented.fact","exists":true}]}]}"#;
        assert_eq!(
            evaluate_text(&modern("git.status", policy), None).decision,
            "DENY"
        );
    }
    #[test]
    fn explanation_does_not_echo_sensitive_string_values() {
        let policy = r#"{"default":"deny","rules":[{"name":"git.status","decision":"allow","conditions":[{"field":"context.token","equals":"secret-value"}]}]}"#;
        let response = evaluate_text(
            &modern("git.status", policy).replace("\"{}\"", "{\"token\":\"secret-value\"}"),
            None,
        );
        assert!(!response_json(&response).contains("secret-value"));
    }
}
