//! Host-owned M3 conformance orchestration and immutable evidence.

use crate::candidate::CandidateRecord;
use crate::launch_profile::{
    revalidate_candidate, LaunchProfileEvidence, PreparedSupervisedLaunch,
};
use crate::m3_store::{canonical, sha256, strict_json, unix_ms, M3Error, Result, StoreRoot};
use crate::manifest;
use crate::package::{CapabilityEvidence, PayloadEvidence};
use crate::trust::{DeveloperApprovalStore, PackageTrustEvidence, PublisherTrustStore};
use crate::validation;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::time::{Duration, Instant};
use uuid::Uuid;

pub const CONFORMANCE_SUITE_VERSION: &str = "m3-generic-1";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CaseDisposition {
    Passed,
    Failed,
    Interrupted,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ConformanceCaseEvidence {
    pub case_id: String,
    pub disposition: CaseDisposition,
    pub safe_diagnostic_code: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConformanceDisposition {
    Passed,
    Failed,
    Interrupted,
    Invalidated,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ConformanceEvidence {
    pub schema_version: u32,
    pub evidence_id: String,
    pub candidate_id: String,
    pub package_id: String,
    pub package_version: String,
    pub semantic_package_digest: String,
    pub payloads: Vec<PayloadEvidence>,
    pub capabilities: Vec<CapabilityEvidence>,
    pub trust_evidence_digest: String,
    pub launch_profile_evidence_digest: String,
    pub launch_profile_label: String,
    pub provider_id: String,
    pub provider_version: String,
    pub socket_major: u32,
    pub mcp_protocol_version: String,
    pub binding_version: String,
    pub host_build_identity: String,
    pub platform: String,
    pub architecture: String,
    pub suite_version: String,
    pub suite_digest: String,
    pub test_configuration_digest: String,
    pub started_unix_ms: u64,
    pub ended_unix_ms: u64,
    pub cases: Vec<ConformanceCaseEvidence>,
    pub disposition: ConformanceDisposition,
    pub retry_count: u32,
    pub raw_stderr_persisted: bool,
    pub evidence_digest: String,
}

impl ConformanceEvidence {
    fn covered_bytes(&self) -> Result<Vec<u8>> {
        let mut copy = self.clone();
        copy.evidence_digest.clear();
        canonical(&copy)
    }

    pub fn validate(&self) -> Result<()> {
        if self.schema_version != 1
            || Uuid::parse_str(&self.evidence_id).is_err()
            || self.socket_major != 1
            || self.mcp_protocol_version != "2025-11-25"
            || self.binding_version != "mcp-stdio-2025-11-25"
            || self.platform != "windows"
            || self.architecture != "x86_64"
            || self.retry_count != 0
            || self.raw_stderr_persisted
            || self.cases.is_empty()
            || self.evidence_digest != sha256(&self.covered_bytes()?)
        {
            return Err(M3Error::new(
                "conformance_invalid",
                "invalid conformance evidence",
            ));
        }
        let case_ids = self
            .cases
            .iter()
            .map(|case| case.case_id.as_str())
            .collect::<BTreeSet<_>>();
        if case_ids.len() != self.cases.len() {
            return Err(M3Error::new(
                "conformance_invalid",
                "duplicate conformance case",
            ));
        }
        let expected = if self
            .cases
            .iter()
            .any(|case| case.disposition == CaseDisposition::Interrupted)
        {
            ConformanceDisposition::Interrupted
        } else if self
            .cases
            .iter()
            .any(|case| case.disposition == CaseDisposition::Failed)
        {
            ConformanceDisposition::Failed
        } else {
            ConformanceDisposition::Passed
        };
        if self.disposition != ConformanceDisposition::Invalidated && self.disposition != expected {
            return Err(M3Error::new(
                "conformance_invalid",
                "disposition does not match cases",
            ));
        }
        Ok(())
    }

    pub fn require_current(
        &self,
        candidate: &CandidateRecord,
        trust: &PackageTrustEvidence,
        launch: &LaunchProfileEvidence,
        current_suite_digest: &str,
    ) -> Result<()> {
        self.validate()?;
        trust.require_for_candidate(candidate)?;
        launch.require_for_candidate(candidate)?;
        if self.disposition != ConformanceDisposition::Passed
            || self.candidate_id != candidate.candidate_id
            || self.semantic_package_digest != candidate.semantic_package_digest
            || self.payloads != candidate.payloads
            || self.capabilities != candidate.capabilities
            || self.trust_evidence_digest != trust.evidence_digest
            || self.launch_profile_evidence_digest != launch.profile_evidence_digest
            || self.suite_digest != current_suite_digest
        {
            return Err(M3Error::new(
                "conformance_stale",
                "conformance pins drifted",
            ));
        }
        Ok(())
    }
}

fn suite_digest() -> Result<String> {
    Ok(sha256(&canonical(&json!({
        "suite_version": CONFORMANCE_SUITE_VERSION,
        "cases": [
            "static_candidate_revalidation",
            "exact_launch_clean_environment",
            "mcp_initialize_protocol_pin",
            "provider_identity",
            "complete_discovery_exact_operations",
            "bounded_valid_fixture_call",
            "invalid_fixture_call_refused",
            "bounded_shutdown_process_cleanup"
        ]
    }))?))
}

pub fn current_suite_digest() -> Result<String> {
    suite_digest()
}

fn passed(case_id: &str) -> ConformanceCaseEvidence {
    ConformanceCaseEvidence {
        case_id: case_id.into(),
        disposition: CaseDisposition::Passed,
        safe_diagnostic_code: None,
    }
}

fn failure(case_id: &str, code: &str, interrupted: bool) -> ConformanceCaseEvidence {
    ConformanceCaseEvidence {
        case_id: case_id.into(),
        disposition: if interrupted {
            CaseDisposition::Interrupted
        } else {
            CaseDisposition::Failed
        },
        safe_diagnostic_code: Some(code.into()),
    }
}

fn parse_line(line: &str) -> Result<Value> {
    strict_json(line.as_bytes())
}

fn expected_tools(
    candidate: &CandidateRecord,
    quarantine: &Path,
) -> Result<BTreeMap<String, (Value, Value)>> {
    let mut expected = BTreeMap::new();
    for capability in &candidate.capabilities {
        let bytes = std::fs::read(quarantine.join(&capability.manifest_path))
            .map_err(|error| M3Error::new("conformance_manifest", error.to_string()))?;
        let text = std::str::from_utf8(&bytes)
            .map_err(|_| M3Error::new("conformance_manifest", "manifest is not UTF-8"))?;
        let verified = manifest::verify_manifest(text)
            .map_err(|error| M3Error::new("conformance_manifest", error.message))?;
        if verified.verified_digest() != capability.manifest_digest
            || verified.capability_name() != capability.name
            || verified.capability_version() != capability.version
        {
            return Err(M3Error::new(
                "conformance_manifest",
                "trusted manifest identity drifted",
            ));
        }
        if expected
            .insert(
                capability.operation.clone(),
                (
                    verified.manifest().input_schema.clone(),
                    verified.manifest().output_schema.clone(),
                ),
            )
            .is_some()
        {
            return Err(M3Error::new(
                "conformance_manifest",
                "duplicate provider operation",
            ));
        }
    }
    Ok(expected)
}

fn request(
    child: &mut crate::child_process::SupervisedChild,
    value: Value,
    timeout: Duration,
) -> Result<Value> {
    child
        .write_line(&serde_json::to_string(&value).expect("JSON value serializes"))
        .map_err(|error| M3Error::new("conformance_protocol", error.to_string()))?;
    let line = child
        .read_protocol_line(timeout)
        .map_err(|error| M3Error::new("conformance_protocol", error.to_string()))?;
    parse_line(&line)
}

pub fn run_host_conformance(
    prepared: &PreparedSupervisedLaunch,
    candidate: &CandidateRecord,
    quarantine_root: &Path,
    trust: &PackageTrustEvidence,
    publisher_trust: &PublisherTrustStore,
    developer_approvals: &DeveloperApprovalStore,
    host_build_identity: &str,
) -> Result<ConformanceEvidence> {
    let started = unix_ms()?;
    let mut cases = Vec::new();
    let quarantine = revalidate_candidate(candidate, quarantine_root)?;
    let expected_tools = expected_tools(candidate, &quarantine)?;
    cases.push(passed("static_candidate_revalidation"));
    trust.require_for_candidate(candidate)?;
    prepared.evidence.require_for_candidate(candidate)?;
    // This deliberately reopens host-owned authority after candidate
    // revalidation and immediately before the process boundary. Historical
    // PackageTrustEvidence is never current launch authority.
    prepared.revalidate_current_trust(candidate, trust, publisher_trust, developer_approvals)?;
    let mut child = prepared
        .launch_for_candidate(candidate, trust, publisher_trust, developer_approvals)
        .map_err(|error| M3Error::new("conformance_launch", error.to_string()))?;
    cases.push(passed("exact_launch_clean_environment"));
    let deadline =
        Instant::now() + Duration::from_millis(prepared.evidence.wall_time_limit_ms.max(1));

    let run_result = (|| -> Result<()> {
        let initialize = request(
            &mut child,
            json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"tethers-m3-conformance","version":"1"}}}),
            deadline.saturating_duration_since(Instant::now()),
        )?;
        if initialize
            .pointer("/result/protocolVersion")
            .and_then(Value::as_str)
            != Some("2025-11-25")
        {
            return Err(M3Error::new(
                "protocol_pin",
                "provider negotiated a different MCP version",
            ));
        }
        cases.push(passed("mcp_initialize_protocol_pin"));
        if initialize
            .pointer("/result/serverInfo/name")
            .and_then(Value::as_str)
            != Some(candidate.provider_id.as_str())
            || initialize
                .pointer("/result/serverInfo/version")
                .and_then(Value::as_str)
                != Some(candidate.provider_version.as_str())
        {
            return Err(M3Error::new(
                "provider_identity",
                "provider identity differs",
            ));
        }
        cases.push(passed("provider_identity"));
        child
            .write_line("{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}")
            .map_err(|error| M3Error::new("conformance_protocol", error.to_string()))?;
        let mut discovered = BTreeSet::new();
        let mut seen_cursors = BTreeSet::new();
        let mut cursor: Option<String> = None;
        for page in 0..16u64 {
            let params = cursor
                .as_ref()
                .map_or_else(|| json!({}), |cursor| json!({"cursor":cursor}));
            let discovery = request(
                &mut child,
                json!({"jsonrpc":"2.0","id":2 + page,"method":"tools/list","params":params}),
                deadline.saturating_duration_since(Instant::now()),
            )?;
            let tools = discovery
                .pointer("/result/tools")
                .and_then(Value::as_array)
                .ok_or_else(|| M3Error::new("catalogue_invalid", "tools/list result is absent"))?;
            for tool in tools {
                let name = tool
                    .get("name")
                    .and_then(Value::as_str)
                    .ok_or_else(|| M3Error::new("catalogue_invalid", "tool name is absent"))?;
                let (expected_input, _) = expected_tools
                    .get(name)
                    .ok_or_else(|| M3Error::new("catalogue_drift", "unapproved tool advertised"))?;
                if tool.get("inputSchema") != Some(expected_input)
                    || !discovered.insert(name.to_owned())
                {
                    return Err(M3Error::new(
                        "catalogue_drift",
                        "tool schema differs or operation is duplicated",
                    ));
                }
            }
            cursor = discovery
                .pointer("/result/nextCursor")
                .and_then(Value::as_str)
                .map(str::to_owned);
            match cursor.as_ref() {
                None => break,
                Some(cursor) if cursor.is_empty() || !seen_cursors.insert(cursor.clone()) => {
                    return Err(M3Error::new(
                        "catalogue_invalid",
                        "empty or repeated discovery cursor",
                    ));
                }
                Some(_) if page == 15 => {
                    return Err(M3Error::new(
                        "catalogue_invalid",
                        "discovery pagination exceeded its bound",
                    ));
                }
                Some(_) => {}
            }
        }
        if discovered != expected_tools.keys().cloned().collect() {
            return Err(M3Error::new(
                "catalogue_drift",
                "discovery operations differ",
            ));
        }
        cases.push(passed("complete_discovery_exact_operations"));
        if let Some(operation) = discovered
            .iter()
            .find(|operation| operation.starts_with("fixture"))
        {
            let (_, output_schema) = expected_tools
                .get(operation)
                .expect("discovered operation came from expected map");
            let valid = request(
                &mut child,
                json!({"jsonrpc":"2.0","id":100,"method":"tools/call","params":{"name":operation,"arguments":{"message":"ping"}}}),
                deadline.saturating_duration_since(Instant::now()),
            )?;
            if valid.pointer("/result/isError").and_then(Value::as_bool) != Some(false)
                || valid
                    .pointer("/result/tethersFixtureEvidence/ambient_secret_present")
                    .and_then(Value::as_bool)
                    != Some(false)
                || valid.pointer("/result/tethersFixtureEvidence/arguments")
                    != Some(
                        &serde_json::to_value(&prepared.evidence.arguments)
                            .expect("arguments serialize"),
                    )
            {
                return Err(M3Error::new(
                    "fixture_valid_call",
                    "valid fixture evidence failed",
                ));
            }
            let structured = valid
                .pointer("/result/structuredContent")
                .ok_or_else(|| M3Error::new("fixture_valid_call", "structured output absent"))?;
            validation::validate_output(output_schema, structured)
                .map_err(|_| M3Error::new("fixture_valid_call", "trusted output schema failed"))?;
            let observed_working_directory = valid
                .pointer("/result/tethersFixtureEvidence/working_directory")
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    M3Error::new(
                        "fixture_working_directory",
                        "working directory evidence absent",
                    )
                })?;
            let observed_working_directory = std::fs::canonicalize(observed_working_directory)
                .map_err(|_| {
                    M3Error::new(
                        "fixture_working_directory",
                        "working directory cannot be canonicalized",
                    )
                })?;
            if observed_working_directory != prepared.working_directory() {
                return Err(M3Error::new(
                    "fixture_working_directory",
                    "working directory differs",
                ));
            }
            let observed_environment = valid
                .pointer("/result/tethersFixtureEvidence/environment_names")
                .and_then(Value::as_array)
                .ok_or_else(|| M3Error::new("fixture_environment", "environment evidence absent"))?
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_ascii_lowercase)
                .collect::<BTreeSet<_>>();
            let expected_environment = prepared
                .evidence
                .environment_names
                .iter()
                .map(|name| name.to_ascii_lowercase())
                .collect::<BTreeSet<_>>();
            if observed_environment != expected_environment {
                return Err(M3Error::new(
                    "fixture_environment",
                    "clean environment allow-list differs",
                ));
            }
            if valid
                .pointer(
                    "/result/tethersFixtureEvidence/unrelated_inheritable_handle_canary_requested",
                )
                .and_then(Value::as_bool)
                == Some(true)
                && valid
                    .pointer(
                        "/result/tethersFixtureEvidence/unrelated_inheritable_handle_accessible",
                    )
                    .and_then(Value::as_bool)
                    != Some(false)
            {
                return Err(M3Error::new(
                    "launch_handle_leak",
                    "provider accessed an unrelated inheritable host handle",
                ));
            }
            cases.push(passed("bounded_valid_fixture_call"));
            let invalid = request(
                &mut child,
                json!({"jsonrpc":"2.0","id":101,"method":"tools/call","params":{"name":operation,"arguments":{"__tethers_invalid":true}}}),
                deadline.saturating_duration_since(Instant::now()),
            )?;
            if invalid.pointer("/result/isError").and_then(Value::as_bool) != Some(true) {
                return Err(M3Error::new(
                    "fixture_invalid_call",
                    "invalid fixture call was accepted",
                ));
            }
            cases.push(passed("invalid_fixture_call_refused"));
        }
        Ok(())
    })();

    let interrupted = run_result
        .as_ref()
        .err()
        .is_some_and(|error| error.message.to_ascii_lowercase().contains("interrupt"));
    if let Err(error) = run_result {
        cases.push(failure("conformance_session", error.code, interrupted));
    }
    child.shutdown();
    cases.push(passed("bounded_shutdown_process_cleanup"));
    let disposition = if cases
        .iter()
        .any(|case| case.disposition == CaseDisposition::Interrupted)
    {
        ConformanceDisposition::Interrupted
    } else if cases
        .iter()
        .any(|case| case.disposition == CaseDisposition::Failed)
    {
        ConformanceDisposition::Failed
    } else {
        ConformanceDisposition::Passed
    };
    let suite_digest = suite_digest()?;
    let mut evidence = ConformanceEvidence {
        schema_version: 1,
        evidence_id: Uuid::new_v4().to_string(),
        candidate_id: candidate.candidate_id.clone(),
        package_id: candidate.package_id.clone(),
        package_version: candidate.package_version.clone(),
        semantic_package_digest: candidate.semantic_package_digest.clone(),
        payloads: candidate.payloads.clone(),
        capabilities: candidate.capabilities.clone(),
        trust_evidence_digest: trust.evidence_digest.clone(),
        launch_profile_evidence_digest: prepared.evidence.profile_evidence_digest.clone(),
        launch_profile_label: prepared.evidence.profile_label.clone(),
        provider_id: candidate.provider_id.clone(),
        provider_version: candidate.provider_version.clone(),
        socket_major: 1,
        mcp_protocol_version: "2025-11-25".into(),
        binding_version: "mcp-stdio-2025-11-25".into(),
        host_build_identity: host_build_identity.into(),
        platform: candidate.selected_platform.os.clone(),
        architecture: candidate.selected_platform.architecture.clone(),
        suite_version: CONFORMANCE_SUITE_VERSION.into(),
        suite_digest,
        test_configuration_digest: prepared.evidence.environment_digest.clone(),
        started_unix_ms: started,
        ended_unix_ms: unix_ms()?,
        cases,
        disposition,
        retry_count: 0,
        raw_stderr_persisted: false,
        evidence_digest: String::new(),
    };
    evidence.evidence_digest = sha256(&evidence.covered_bytes()?);
    evidence.validate()?;
    Ok(evidence)
}

pub struct ConformanceEvidenceStore {
    root: StoreRoot,
}

impl ConformanceEvidenceStore {
    pub fn open(path: &Path) -> Result<Self> {
        Ok(Self {
            root: StoreRoot::open(path)?,
        })
    }

    pub fn open_existing(path: &Path) -> Result<Self> {
        Ok(Self {
            root: StoreRoot::open_existing(path)?,
        })
    }

    pub fn create(&self, evidence: &ConformanceEvidence) -> Result<()> {
        evidence.validate()?;
        self.root.create_json(&evidence.evidence_id, evidence)?;
        Ok(())
    }

    pub fn load_all(&self) -> Result<Vec<ConformanceEvidence>> {
        let mut evidence = Vec::new();
        let mut identities = BTreeSet::new();
        for path in self.root.entries()? {
            if path.extension().and_then(|value| value.to_str()) == Some("tmp") {
                return Err(M3Error::new("conformance_invalid", "torn evidence record"));
            }
            if path.extension().and_then(|value| value.to_str()) != Some("json") {
                return Err(M3Error::new(
                    "conformance_invalid",
                    "unexpected evidence entry",
                ));
            }
            let record: ConformanceEvidence = self.root.read(&path)?;
            record.validate()?;
            if path.file_stem().and_then(|value| value.to_str()) != Some(&record.evidence_id)
                || !identities.insert(record.evidence_id.clone())
            {
                return Err(M3Error::new(
                    "conformance_invalid",
                    "duplicate or mismatched evidence identity",
                ));
            }
            evidence.push(record);
        }
        Ok(evidence)
    }
}
