//! Host-owned execution-environment handshake for development tasks.
//!
//! This is intentionally outside the OCaml Core.  It binds one selected worker,
//! one task/session, one repository state, and exact command arrays to a single
//! host observation.  Command launch goes through `SupervisedChild`; matching an
//! executable name alone never grants execution authority.
//!
//! ## Contract integrity
//!
//! `ContractData` fields are private.  `issue()` is the only constructor.
//! `ExecutionEnvironmentContract::from_stored()` recomputes and verifies the
//! digest during reload.  `permit()` recomputes and verifies it again before
//! every command lookup, so a deserialised or mutated contract cannot bypass
//! integrity validation.
//!
//! ## Substitution
//!
//! Capability substitution is explicitly deferred from executable v1.  The
//! shared workbench profile documents host-named substitutes but the issuer
//! does not resolve them at runtime.  A required/preferred capability whose
//! host probe fails gates the contract as specified; no replacement is
//! invented.
//!
//! ## PowerShell enforcement
//!
//! One exact approved form is permitted:
//!
//! pwsh.exe -NoLogo -NoProfile -NonInteractive -File <absolute-script> [args]
//!
//! `-Command` and `-EncodedCommand` are unconditionally refused.  The issuer
//! hashes the canonical script file; `permit()` rehashes and re-canonicalises
//! the path; `CommandPermit::launch()` canonicalises and rehashes immediately
//! before `SupervisedChild::launch` to close the permit-to-launch race.

use crate::child_process::{ChildConfig, ChildError, SupervisedChild};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub const WORKBENCH_PROFILE_ID: &str = "tethers-development-workbench-v1";
pub const MATTHEW_ASSIGNMENT_AUTHORITY: &str = "Matthew";

const WORKBENCH_CAPABILITIES: &[&str] = &[
    "git-inspection",
    "recursive-text-search",
    "structured-json-query",
    "github-api-inspection",
    "task-automation-runner",
    "rust-compilation",
    "rust-formatting",
    "rust-linting",
    "ocaml-compilation",
    "ocaml-formatting",
    "ocaml-switch-management",
    "powershell-automation",
];

const REQUIRED_PLATFORM: &str = "windows";
const REQUIRED_SHELL: &str = "pwsh";

/// One exact approved PowerShell argument sequence prefix.
const POWERSHELL_PREFIX: &[&str] = &["-nologo", "-noprofile", "-noninteractive"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequirementClass {
    Required,
    Preferred,
    Replaceable,
    Optional,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum VersionPolicy {
    Exact { version: String },
    Minimum { version: String },
    Any,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityRequirement {
    pub id: String,
    pub class: RequirementClass,
    pub version_policy: VersionPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryBinding {
    pub root: String,
    pub branch: String,
    pub head: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskBinding {
    pub task_id: String,
    pub session_id: String,
    pub scope: Vec<String>,
    pub repository: RepositoryBinding,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkerAssignment {
    pub selected_by: String,
    pub worker_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestedCommand {
    pub command_id: String,
    pub capability_id: String,
    pub args: Vec<String>,
    pub cwd: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PermissionScopes {
    #[serde(default)]
    pub filesystem_read: BTreeSet<String>,
    #[serde(default)]
    pub filesystem_write: BTreeSet<String>,
    #[serde(default)]
    pub network_hosts: BTreeSet<String>,
    pub network_allowed: bool,
    pub installation_allowed: bool,
}

impl PermissionScopes {
    fn contains(&self, requested: &Self) -> bool {
        (!requested.network_allowed || self.network_allowed)
            && (!requested.installation_allowed || self.installation_allowed)
            && requested.filesystem_read.is_subset(&self.filesystem_read)
            && requested.filesystem_write.is_subset(&self.filesystem_write)
            && requested.network_hosts.is_subset(&self.network_hosts)
    }

    fn contains_path(&self, canonical: &str) -> bool {
        for root in self
            .filesystem_write
            .iter()
            .chain(self.filesystem_read.iter())
        {
            if canonical_as_prefix(root, canonical) {
                return true;
            }
        }
        false
    }
}

fn canonical_as_prefix(scope: &str, path: &str) -> bool {
    fn normalise(s: &str) -> String {
        let stripped = s.strip_prefix("\\\\?\\").unwrap_or(s);
        stripped.replace('/', "\\").to_ascii_lowercase()
    }
    let scope_norm = normalise(scope);
    let path_norm = normalise(path);
    if !path_norm.starts_with(&scope_norm) {
        return false;
    }
    if path_norm.len() == scope_norm.len() {
        return true;
    }
    // After the scope prefix the next byte must be a separator,
    // unless the scope itself ends with one (e.g. drive root C:\).
    if scope_norm.ends_with('\\') {
        return true;
    }
    path_norm.as_bytes()[scope_norm.len()] == b'\\'
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskEnvironmentRequest {
    pub schema: String,
    pub request_id: String,
    pub workbench_profile: String,
    pub task: TaskBinding,
    pub worker_assignment: WorkerAssignment,
    pub capabilities: Vec<CapabilityRequirement>,
    pub commands: Vec<RequestedCommand>,
    pub requested_permissions: PermissionScopes,
    /// Must remain false: installation is a separate, explicitly authorised task.
    pub automatic_install: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostCapabilityObservation {
    pub version: String,
    pub verified: bool,
    pub command_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostCommandObservation {
    pub program_path: String,
    pub args: Vec<String>,
    pub cwd: String,
    /// Required for a PowerShell `-File` command; identifies reviewed bytes.
    pub script_digest: Option<String>,
    #[serde(default)]
    pub environment: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostEnvironmentObservation {
    pub observation_id: String,
    pub platform: String,
    pub shell: String,
    pub repository: RepositoryBinding,
    pub granted_permissions: PermissionScopes,
    pub capabilities: BTreeMap<String, HostCapabilityObservation>,
    pub commands: BTreeMap<String, HostCommandObservation>,
    pub process_tree_supervision_available: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityState {
    Available,
    Degraded,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityResolution {
    pub id: String,
    pub class: RequirementClass,
    pub state: CapabilityState,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovedCommand {
    pub command_id: String,
    pub program_path: String,
    pub args: Vec<String>,
    pub cwd: String,
    pub script_digest: Option<String>,
    /// Canonical path of the PowerShell script file, present only for
    /// commands whose `program_path` is pwsh.exe with `-File`.
    #[serde(default)]
    pub script_path: Option<String>,
    pub environment: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContractStatus {
    Agreed,
    Degraded,
    Blocked,
}

/// Host-owned maximum supervised process count.  Bounded in the contract.
const HOST_MAX_SUPERVISED_PROCESSES: u32 = 16;

/// Private immutable contract data.  Only `issue()` and `from_stored()` can
/// construct a valid instance; `permit()` recomputes the digest every time.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ContractData {
    schema: String,
    request_id: String,
    observation_id: String,
    task: TaskBinding,
    worker_assignment: WorkerAssignment,
    status: ContractStatus,
    capability_resolutions: Vec<CapabilityResolution>,
    approved_commands: BTreeMap<String, ApprovedCommand>,
    granted_permissions: PermissionScopes,
    process_tree_supervision_required: bool,
    request_digest: String,
    observation_digest: String,
    max_supervised_processes: u32,
}

#[derive(Debug, Clone)]
pub struct ExecutionEnvironmentContract {
    data: ContractData,
    contract_digest: String,
}

impl ExecutionEnvironmentContract {
    fn data_digest(data: &ContractData) -> Result<String, EnvironmentError> {
        canonical_digest(data)
    }

    /// Recompute and verify the stored digest.
    fn verify_integrity(&self) -> Result<(), EnvironmentError> {
        let recomputed = Self::data_digest(&self.data)?;
        if recomputed != self.contract_digest {
            return Err(EnvironmentError::new(
                "contract_integrity",
                "contract digest mismatch — contract bytes may have been tampered with",
            ));
        }
        Ok(())
    }

    /// Restore a contract from stored JSON.  The stored digest is recomputed
    /// and verified before the contract is accepted.
    pub fn from_stored(json: &str) -> Result<Self, EnvironmentError> {
        #[derive(Deserialize)]
        struct StoredContract {
            #[serde(rename = "contract_digest")]
            stored_digest: String,
            #[serde(flatten)]
            data: ContractData,
        }
        let stored: StoredContract = serde_json::from_str(json).map_err(|error| {
            EnvironmentError::new(
                "deserialize",
                format!("cannot parse stored contract: {error}"),
            )
        })?;
        let recomputed = Self::data_digest(&stored.data)?;
        if recomputed != stored.stored_digest {
            return Err(EnvironmentError::new(
                "contract_integrity",
                "stored contract digest does not match stored body — rejected at load",
            ));
        }
        Ok(Self {
            data: stored.data,
            contract_digest: stored.stored_digest,
        })
    }

    pub fn status(&self) -> &ContractStatus {
        &self.data.status
    }

    pub fn contract_digest(&self) -> &str {
        &self.contract_digest
    }

    pub fn request_digest(&self) -> &str {
        &self.data.request_digest
    }

    pub fn observation_digest(&self) -> &str {
        &self.data.observation_digest
    }

    pub fn issue(
        request: TaskEnvironmentRequest,
        observation: HostEnvironmentObservation,
    ) -> Result<Self, EnvironmentError> {
        if request.schema != "tethers-execution-environment-request-v1" {
            return Err(EnvironmentError::new(
                "schema",
                "unsupported task request schema",
            ));
        }
        if request.workbench_profile != WORKBENCH_PROFILE_ID {
            return Err(EnvironmentError::new(
                "profile",
                "unknown workbench profile",
            ));
        }
        if request.worker_assignment.selected_by != MATTHEW_ASSIGNMENT_AUTHORITY {
            return Err(EnvironmentError::new(
                "worker_assignment",
                "only Matthew may select or replace a worker",
            ));
        }
        if worker_overlay(&request.worker_assignment.worker_id).is_none() {
            return Err(EnvironmentError::new(
                "worker",
                "worker has no approved optional overlay",
            ));
        }
        if request.automatic_install || request.requested_permissions.installation_allowed {
            return Err(EnvironmentError::new(
                "automatic_install",
                "the workbench never installs software during a handshake",
            ));
        }
        if request.task.repository != observation.repository {
            return Err(EnvironmentError::new(
                "repository_binding",
                "host observation does not match the requested repository, branch, and HEAD",
            ));
        }
        if !observation
            .granted_permissions
            .contains(&request.requested_permissions)
        {
            return Err(EnvironmentError::new(
                "permission_scope",
                "host cannot grant the requested filesystem or network scope",
            ));
        }

        validate_no_duplicate_capability_ids(&request.capabilities)?;
        validate_no_duplicate_command_ids(&request.commands)?;
        validate_workbench_env(&observation)?;
        validate_command_paths_and_scope(
            &request.commands,
            &observation.commands,
            &observation.granted_permissions,
        )?;

        for command in &request.commands {
            if let Some(observed) = observation.commands.get(&command.command_id) {
                validate_powershell_command(observed)?;
            }
        }

        let request_digest = canonical_digest(&request)?;
        let observation_digest = canonical_digest(&observation)?;
        let mut resolutions = Vec::new();
        let mut approved_commands = BTreeMap::new();

        for requirement in &request.capabilities {
            if !WORKBENCH_CAPABILITIES.contains(&requirement.id.as_str()) {
                return Err(EnvironmentError::new(
                    "capability",
                    "capability is absent from shared profile",
                ));
            }
            let resolution = observation
                .capabilities
                .get(&requirement.id)
                .and_then(|capability| {
                    (capability.verified
                        && version_satisfies(&capability.version, &requirement.version_policy))
                    .then_some(capability)
                });
            let Some(capability) = resolution else {
                resolutions.push(CapabilityResolution {
                    id: requirement.id.clone(),
                    class: requirement.class.clone(),
                    state: capability_failure(&requirement.class),
                    reason: Some("missing, unverified, or version-policy mismatch".to_owned()),
                });
                continue;
            };
            let observed = request
                .commands
                .iter()
                .find(|command| command.capability_id == requirement.id)
                .and_then(|command| {
                    observation
                        .commands
                        .get(&command.command_id)
                        .map(|observed| (command, observed))
                });
            let Some((requested, observed)) = observed else {
                resolutions.push(CapabilityResolution {
                    id: requirement.id.clone(),
                    class: requirement.class.clone(),
                    state: capability_failure(&requirement.class),
                    reason: Some("no exact host command observation".to_owned()),
                });
                continue;
            };
            if capability.command_id != requested.command_id
                || requested.args != observed.args
                || requested.cwd != observed.cwd
            {
                resolutions.push(CapabilityResolution {
                    id: requirement.id.clone(),
                    class: requirement.class.clone(),
                    state: capability_failure(&requirement.class),
                    reason: Some("command is not an exact reviewed host command".to_owned()),
                });
                continue;
            }
            let canonical_script = extract_script_path(observed)
                .as_deref()
                .map(canonicalize_path)
                .transpose()?;
            approved_commands.insert(
                requested.command_id.clone(),
                ApprovedCommand {
                    command_id: requested.command_id.clone(),
                    program_path: observed.program_path.clone(),
                    // Preserve the exact reviewed argument array.  For a
                    // PowerShell -File invocation this may intentionally
                    // contain a junction path; script_path records its
                    // canonical target for permit and launch-time identity
                    // checks.
                    args: observed.args.clone(),
                    cwd: observed.cwd.clone(),
                    script_digest: observed.script_digest.clone(),
                    script_path: canonical_script,
                    environment: observed.environment.clone(),
                },
            );
            resolutions.push(CapabilityResolution {
                id: requirement.id.clone(),
                class: requirement.class.clone(),
                state: CapabilityState::Available,
                reason: None,
            });
        }

        let blocked = !observation.process_tree_supervision_available
            || resolutions.iter().any(|resolution| {
                resolution.class == RequirementClass::Required
                    && resolution.state != CapabilityState::Available
            });
        let degraded = resolutions
            .iter()
            .any(|resolution| resolution.state == CapabilityState::Degraded);
        let status = if blocked {
            ContractStatus::Blocked
        } else if degraded {
            ContractStatus::Degraded
        } else {
            ContractStatus::Agreed
        };
        let data = ContractData {
            schema: "tethers-execution-environment-contract-v1".to_owned(),
            request_id: request.request_id,
            observation_id: observation.observation_id,
            task: request.task,
            worker_assignment: request.worker_assignment,
            status,
            capability_resolutions: resolutions,
            approved_commands,
            granted_permissions: request.requested_permissions,
            process_tree_supervision_required: true,
            request_digest,
            observation_digest,
            max_supervised_processes: HOST_MAX_SUPERVISED_PROCESSES,
        };
        let contract_digest = canonical_digest(&data)?;
        Ok(Self {
            data,
            contract_digest,
        })
    }

    pub fn permit(&self, invocation: CommandInvocation) -> Result<CommandPermit, EnvironmentError> {
        self.verify_integrity()?;

        if self.data.status == ContractStatus::Blocked {
            return Err(EnvironmentError::new(
                "contract_blocked",
                "blocked contracts cannot launch a process",
            ));
        }
        if !self.data.process_tree_supervision_required {
            return Err(EnvironmentError::new(
                "supervision",
                "contract lacks required process-tree supervision",
            ));
        }
        let Some(command) = self.data.approved_commands.get(&invocation.command_id) else {
            return Err(EnvironmentError::new(
                "command",
                "command id is not in the frozen contract",
            ));
        };
        if command.program_path != invocation.program_path || command.cwd != invocation.cwd {
            return Err(EnvironmentError::new(
                "command_mismatch",
                "program, arguments, and working directory must exactly match the contract",
            ));
        }
        // Check canonical script-path identity before generic argument comparison
        // so a changed target returns script_redirected rather than command_mismatch.
        if let Some(script) = &command.script_path {
            let invocation_script_idx =
                extract_file_arg_index(&invocation.args).ok_or_else(|| {
                    EnvironmentError::new(
                        "script_redirected",
                        "PowerShell -File argument is missing from the invocation",
                    )
                })?;
            let inv_canonical = canonicalize_path(&invocation.args[invocation_script_idx])?;
            if &inv_canonical != script {
                return Err(EnvironmentError::new(
                    "script_redirected",
                    "PowerShell script path was redirected after contract issuance",
                ));
            }
        }
        if command.args != invocation.args {
            return Err(EnvironmentError::new(
                "command_mismatch",
                "arguments must exactly match the approved contract command",
            ));
        }
        if let Some(script) = &command.script_path {
            let current =
                hash_file(script).map_err(|error| EnvironmentError::new("script_digest", error))?;
            let expected = command
                .script_digest
                .as_deref()
                .and_then(|d| d.strip_prefix("sha256:"))
                .unwrap_or("");
            if current != expected {
                return Err(EnvironmentError::new(
                    "script_changed",
                    "script file bytes have changed since contract issuance",
                ));
            }
        }
        let max_supervised_processes = self.data.max_supervised_processes;
        Ok(CommandPermit {
            command: command.clone(),
            max_supervised_processes,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandInvocation {
    pub command_id: String,
    pub program_path: String,
    pub args: Vec<String>,
    pub cwd: String,
}

/// A non-cloneable permission to launch exactly one contract-approved process.
///
/// `child_config()` is private to prevent callers from altering the approved
/// configuration and launching it separately.  `launch()` is the only launch
/// path; it canonicalises and rehashes the script path immediately before
/// `SupervisedChild::launch` to close the permit-to-launch race.
#[derive(Debug)]
pub struct CommandPermit {
    command: ApprovedCommand,
    max_supervised_processes: u32,
}

impl CommandPermit {
    fn child_config(&self) -> ChildConfig {
        ChildConfig {
            command: self.command.program_path.clone(),
            args: self.command.args.clone(),
            current_dir: Some(PathBuf::from(&self.command.cwd)),
            startup_timeout: Duration::from_secs(10),
            graceful_close_timeout: Duration::from_secs(2),
            max_protocol_line_bytes: 8 * 1024 * 1024,
            stderr_tail_bytes: 64 * 1024,
            clear_environment: true,
            environment: self.command.environment.clone(),
            max_processes: self.max_supervised_processes,
            process_memory_limit_bytes: 512 * 1024 * 1024,
            assign_before_execution: true,
        }
    }

    pub fn launch(self) -> Result<SupervisedChild, ChildError> {
        if let Some(script) = &self.command.script_path {
            // Canonicalise the -File argument immediately before launch and
            // require exact equality with the stored script path to close
            // the permit-to-launch race.
            let file_idx = extract_file_arg_index(&self.command.args).ok_or_else(|| {
                ChildError::LaunchFailed {
                    command: self.command.program_path.clone(),
                    message: "script -File argument missing at launch".into(),
                }
            })?;
            let canonical_now = canonicalize_path(&self.command.args[file_idx]).map_err(|e| {
                ChildError::LaunchFailed {
                    command: self.command.program_path.clone(),
                    message: format!("cannot canonicalize script path at launch: {}", e.message),
                }
            })?;
            if &canonical_now != script {
                return Err(ChildError::LaunchFailed {
                    command: self.command.program_path.clone(),
                    message: "script path redirected between permit and launch".into(),
                });
            }
            match hash_file(&canonical_now) {
                Ok(current) => {
                    let expected = self
                        .command
                        .script_digest
                        .as_deref()
                        .and_then(|d| d.strip_prefix("sha256:"))
                        .unwrap_or("");
                    if current != expected {
                        return Err(ChildError::LaunchFailed {
                            command: self.command.program_path.clone(),
                            message: "script hash changed between permit and launch".into(),
                        });
                    }
                }
                Err(_) => {
                    return Err(ChildError::LaunchFailed {
                        command: self.command.program_path.clone(),
                        message: "script file missing at launch time".into(),
                    });
                }
            }
        }
        SupervisedChild::launch(self.child_config())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvironmentError {
    pub code: &'static str,
    pub message: String,
}

impl EnvironmentError {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl fmt::Display for EnvironmentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for EnvironmentError {}

/// Small worker overlays. They are role constraints, not duplicated tool profiles.
pub fn worker_overlay(worker_id: &str) -> Option<(&'static str, &'static [&'static str])> {
    match worker_id {
        "lucy-codex" => Some((
            "architecture-review-and-difficult-integration",
            &[
                "may recommend; Matthew selects worker",
                "requires independent Red sign-off",
            ],
        )),
        "luna-opencode" => Some((
            "autonomous-implementation-within-frozen-architecture",
            &[
                "must remain inside frozen scope",
                "must run required matrix",
            ],
        )),
        "deepseek-pro-v4-opencode" => Some((
            "bounded-implementation-correction-and-documentation",
            &["must not change architecture without review"],
        )),
        "hy3-opencode" => Some((
            "narrow-mechanical-implementation",
            &[
                "must report uncertainty",
                "no architecture or security decisions",
            ],
        )),
        _ => None,
    }
}

fn canonical_digest<T: Serialize>(value: &T) -> Result<String, EnvironmentError> {
    let json = serde_json::to_value(value)
        .map_err(|error| EnvironmentError::new("serialize", error.to_string()))?;
    let bytes = serde_json_canonicalizer::to_vec(&json)
        .map_err(|error| EnvironmentError::new("canonical_json", error.to_string()))?;
    Ok(format!("sha256:{:x}", Sha256::digest(bytes)))
}

fn hash_file(path: &str) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|error| format!("cannot read file: {error}"))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn version_satisfies(actual: &str, policy: &VersionPolicy) -> bool {
    match policy {
        VersionPolicy::Any => true,
        VersionPolicy::Exact { version } => actual == version,
        VersionPolicy::Minimum { version } => version_key(actual) >= version_key(version),
    }
}

fn version_key(value: &str) -> Vec<u64> {
    value
        .split(|character: char| !character.is_ascii_digit())
        .filter(|part| !part.is_empty())
        .map(|part| part.parse::<u64>().unwrap_or(0))
        .collect()
}

fn is_absolute_windows_path(value: &str) -> bool {
    if value.len() < 3 {
        return false;
    }
    let bytes = value.as_bytes();
    bytes[0].is_ascii_alphabetic() && bytes[1] == b':' && (bytes[2] == b'\\' || bytes[2] == b'/')
}

fn canonicalize_path(raw: &str) -> Result<String, EnvironmentError> {
    let p = Path::new(raw);
    if !is_absolute_windows_path(raw) {
        return Err(EnvironmentError::new(
            "path",
            format!("path is not absolute: {raw}"),
        ));
    }
    if p.exists() {
        p.canonicalize()
            .map(|cp| {
                let s = cp.to_string_lossy().replace('/', "\\");
                s.strip_prefix("\\\\?\\").unwrap_or(&s).to_owned()
            })
            .map_err(|error| {
                EnvironmentError::new("canonical", format!("cannot canonicalize '{raw}': {error}"))
            })
    } else {
        resolve_path_segments(raw)
    }
}

fn resolve_path_segments(raw: &str) -> Result<String, EnvironmentError> {
    let normalised = raw.replace('/', "\\");
    let components: Vec<&str> = normalised
        .split('\\')
        .filter(|c| !c.is_empty() && *c != ".")
        .collect();
    let mut resolved: Vec<&str> = Vec::new();
    for comp in components {
        if comp == ".." {
            if resolved.pop().is_none() {
                return Err(EnvironmentError::new(
                    "scope",
                    format!("path '{raw}' escapes beyond the root"),
                ));
            }
        } else {
            resolved.push(comp);
        }
    }
    let result = resolved.join("\\");
    if result.len() < 2 || !result.contains('\\') {
        return Err(EnvironmentError::new(
            "scope",
            format!("path '{raw}' resolves outside any granted root"),
        ));
    }
    Ok(result)
}

fn validate_workbench_env(
    observation: &HostEnvironmentObservation,
) -> Result<(), EnvironmentError> {
    let platform_lower = observation.platform.to_ascii_lowercase();
    if platform_lower != REQUIRED_PLATFORM {
        return Err(EnvironmentError::new(
            "platform",
            format!(
                "workbench requires platform '{REQUIRED_PLATFORM}'; observed '{platform_lower}'"
            ),
        ));
    }
    let shell_lower = observation.shell.to_ascii_lowercase();
    if shell_lower != REQUIRED_SHELL {
        return Err(EnvironmentError::new(
            "shell",
            format!("workbench requires shell '{REQUIRED_SHELL}'; observed '{shell_lower}'"),
        ));
    }
    for (_id, observed) in &observation.commands {
        if !is_absolute_windows_path(&observed.program_path) {
            return Err(EnvironmentError::new(
                "program_path",
                format!(
                    "program_path must be an absolute canonical path: {}",
                    observed.program_path
                ),
            ));
        }
        if !is_absolute_windows_path(&observed.cwd) {
            return Err(EnvironmentError::new(
                "cwd",
                format!(
                    "working directory must be an absolute canonical path: {}",
                    observed.cwd
                ),
            ));
        }
    }
    Ok(())
}

fn validate_no_duplicate_capability_ids(
    capabilities: &[CapabilityRequirement],
) -> Result<(), EnvironmentError> {
    let mut seen = BTreeSet::new();
    for cap in capabilities {
        if !seen.insert(&cap.id) {
            return Err(EnvironmentError::new(
                "duplicate_capability",
                format!("duplicate capability id: {}", cap.id),
            ));
        }
    }
    Ok(())
}

fn validate_no_duplicate_command_ids(
    commands: &[RequestedCommand],
) -> Result<(), EnvironmentError> {
    let mut seen = BTreeSet::new();
    for cmd in commands {
        if !seen.insert(&cmd.command_id) {
            return Err(EnvironmentError::new(
                "duplicate_command",
                format!("duplicate command id: {}", cmd.command_id),
            ));
        }
    }
    Ok(())
}

fn validate_command_paths_and_scope(
    requested: &[RequestedCommand],
    observed: &BTreeMap<String, HostCommandObservation>,
    permissions: &PermissionScopes,
) -> Result<(), EnvironmentError> {
    for cmd in requested {
        if let Some(obs) = observed.get(&cmd.command_id) {
            if !is_absolute_windows_path(&obs.program_path) {
                return Err(EnvironmentError::new(
                    "program_path",
                    format!(
                        "observed program_path must be absolute: {}",
                        obs.program_path
                    ),
                ));
            }
            if !is_absolute_windows_path(&obs.cwd) {
                return Err(EnvironmentError::new(
                    "cwd",
                    format!("observed cwd must be absolute: {}", obs.cwd),
                ));
            }
            let canonical_cwd = canonicalize_path(&obs.cwd)?;
            if !permissions.contains_path(&canonical_cwd) {
                return Err(EnvironmentError::new(
                    "scope",
                    format!("cwd '{canonical_cwd}' is outside the granted filesystem scope"),
                ));
            }
            if is_powershell_command(&obs.program_path) {
                if let Some(script) = extract_script_path(obs) {
                    if is_absolute_windows_path(&script) {
                        let canonical_script = canonicalize_path(&script)?;
                        if !permissions.contains_path(&canonical_script) {
                            return Err(EnvironmentError::new(
                                "scope",
                                format!(
                                    "PowerShell script '{canonical_script}' is outside the granted filesystem scope"
                                ),
                            ));
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn is_powershell_command(program_path: &str) -> bool {
    let lower = program_path.to_ascii_lowercase();
    lower.ends_with("pwsh.exe") || lower.ends_with("powershell.exe")
}

fn extract_file_arg_index(args: &[String]) -> Option<usize> {
    let lower: Vec<String> = args.iter().map(|a| a.to_ascii_lowercase()).collect();
    lower.iter().position(|a| a == "-file").and_then(|i| {
        if i + 1 < args.len() {
            Some(i + 1)
        } else {
            None
        }
    })
}

fn extract_script_path(observed: &HostCommandObservation) -> Option<String> {
    if !is_powershell_command(&observed.program_path) {
        return None;
    }
    extract_file_arg_index(&observed.args).map(|script_idx| observed.args[script_idx].clone())
}

fn validate_powershell_command(observed: &HostCommandObservation) -> Result<(), EnvironmentError> {
    if !is_powershell_command(&observed.program_path) {
        if observed.script_digest.is_some() {
            return Err(EnvironmentError::new(
                "powershell",
                "script_digest is only valid with an absolute pwsh.exe program_path",
            ));
        }
        return Ok(());
    }
    if observed.args.is_empty() {
        return Err(EnvironmentError::new(
            "powershell",
            "PowerShell command requires at least a -File argument",
        ));
    }
    let lower_args: Vec<String> = observed
        .args
        .iter()
        .map(|a| a.to_ascii_lowercase())
        .collect();

    // Reject -Command and -EncodedCommand unconditionally.
    for arg in &lower_args {
        if arg == "-command" || arg == "-encodedcommand" {
            return Err(EnvironmentError::new(
                "powershell",
                "-Command and -EncodedCommand are forbidden; only -File is permitted",
            ));
        }
    }

    // The exact approved prefix is -NoLogo -NoProfile -NonInteractive.
    if lower_args.len() < POWERSHELL_PREFIX.len() + 2 {
        return Err(EnvironmentError::new(
            "powershell",
            "PowerShell command must start with -NoLogo -NoProfile -NonInteractive, then -File <script>",
        ));
    }
    for (i, expected) in POWERSHELL_PREFIX.iter().enumerate() {
        if &lower_args[i] != expected {
            return Err(EnvironmentError::new(
                "powershell",
                format!(
                    "PowerShell command must start with -NoLogo -NoProfile -NonInteractive; at position {i} expected '{expected}' but got '{}'",
                    lower_args[i]
                ),
            ));
        }
    }
    if lower_args[POWERSHELL_PREFIX.len()] != "-file" {
        return Err(EnvironmentError::new(
            "powershell",
            format!(
                "PowerShell command must use -File after -NoLogo -NoProfile -NonInteractive; got '{}'",
                lower_args[POWERSHELL_PREFIX.len()]
            ),
        ));
    }

    let script_path = &observed.args[POWERSHELL_PREFIX.len() + 1];
    if !is_absolute_windows_path(script_path) {
        return Err(EnvironmentError::new(
            "powershell",
            format!("PowerShell -File script path must be absolute: {script_path}"),
        ));
    }
    if !Path::new(script_path).exists() {
        return Err(EnvironmentError::new(
            "powershell",
            format!("PowerShell script file does not exist: {script_path}"),
        ));
    }
    if observed.script_digest.is_none() {
        return Err(EnvironmentError::new(
            "powershell",
            "PowerShell command requires a reviewed script_digest",
        ));
    }
    let actual_digest = format!(
        "sha256:{}",
        hash_file(script_path).map_err(|error| EnvironmentError::new("powershell", error))?
    );
    if observed.script_digest.as_deref() != Some(&actual_digest) {
        return Err(EnvironmentError::new(
            "powershell",
            "PowerShell script_digest does not match the actual script file bytes",
        ));
    }
    Ok(())
}

fn capability_failure(class: &RequirementClass) -> CapabilityState {
    match class {
        RequirementClass::Required => CapabilityState::Unavailable,
        RequirementClass::Preferred | RequirementClass::Replaceable => CapabilityState::Degraded,
        RequirementClass::Optional => CapabilityState::Unavailable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn create_junction(link: &Path, target: &Path) -> Result<(), String> {
        let output = std::process::Command::new("cmd")
            .args([
                "/c",
                "mklink",
                "/J",
                &link.to_string_lossy(),
                &target.to_string_lossy(),
            ])
            .output()
            .map_err(|e| e.to_string())?;
        if output.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    fn permissions() -> PermissionScopes {
        PermissionScopes {
            filesystem_read: BTreeSet::from(["D:/Tethers".to_owned()]),
            filesystem_write: BTreeSet::from(["D:/Tethers".to_owned()]),
            network_hosts: BTreeSet::from(["github.com".to_owned()]),
            network_allowed: true,
            installation_allowed: false,
        }
    }

    fn broad_permissions() -> PermissionScopes {
        let temp = std::env::temp_dir();
        let temp_str = temp.to_string_lossy().replace('/', "\\");
        let scope_root = if let Some(drive) = temp_str.get(..3) {
            drive.to_owned()
        } else {
            temp_str
        };
        let system = "C:/Windows".to_owned();
        PermissionScopes {
            filesystem_read: BTreeSet::from([scope_root.clone(), system.clone()]),
            filesystem_write: BTreeSet::from([scope_root, system]),
            network_hosts: BTreeSet::new(),
            network_allowed: false,
            installation_allowed: false,
        }
    }

    fn request(class: RequirementClass) -> TaskEnvironmentRequest {
        TaskEnvironmentRequest {
            schema: "tethers-execution-environment-request-v1".to_owned(),
            request_id: "request-1".to_owned(),
            workbench_profile: WORKBENCH_PROFILE_ID.to_owned(),
            task: TaskBinding {
                task_id: "J20-ENV-P1".to_owned(),
                session_id: "session-1".to_owned(),
                scope: vec!["rust-host".to_owned()],
                repository: RepositoryBinding {
                    root: "D:/Tethers".to_owned(),
                    branch: "codex/example".to_owned(),
                    head: "a".repeat(40),
                },
            },
            worker_assignment: WorkerAssignment {
                selected_by: MATTHEW_ASSIGNMENT_AUTHORITY.to_owned(),
                worker_id: "deepseek-pro-v4-opencode".to_owned(),
            },
            capabilities: vec![CapabilityRequirement {
                id: "rust-compilation".to_owned(),
                class,
                version_policy: VersionPolicy::Exact {
                    version: "1.89.0".to_owned(),
                },
            }],
            commands: vec![RequestedCommand {
                command_id: "rust-check".to_owned(),
                capability_id: "rust-compilation".to_owned(),
                args: vec!["check".to_owned(), "--locked".to_owned()],
                cwd: "D:/Tethers/tethers-0.1/host-rust".to_owned(),
            }],
            requested_permissions: permissions(),
            automatic_install: false,
        }
    }

    fn observation() -> HostEnvironmentObservation {
        HostEnvironmentObservation {
            observation_id: "observation-1".to_owned(),
            platform: "windows".to_owned(),
            shell: "pwsh".to_owned(),
            repository: request(RequirementClass::Required).task.repository,
            granted_permissions: permissions(),
            capabilities: BTreeMap::from([(
                "rust-compilation".to_owned(),
                HostCapabilityObservation {
                    version: "1.89.0".to_owned(),
                    verified: true,
                    command_id: "rust-check".to_owned(),
                },
            )]),
            commands: BTreeMap::from([(
                "rust-check".to_owned(),
                HostCommandObservation {
                    program_path: "C:/Users/Matmus/.cargo/bin/cargo.exe".to_owned(),
                    args: vec!["check".to_owned(), "--locked".to_owned()],
                    cwd: "D:/Tethers/tethers-0.1/host-rust".to_owned(),
                    script_digest: None,
                    environment: BTreeMap::new(),
                },
            )]),
            process_tree_supervision_available: true,
        }
    }

    fn issue_agreed() -> ExecutionEnvironmentContract {
        ExecutionEnvironmentContract::issue(request(RequirementClass::Required), observation())
            .unwrap()
    }

    // ── existing issuer / permit tests ──────────────────────────

    #[test]
    fn issues_an_immutable_contract_with_all_three_digests() {
        let contract = issue_agreed();
        assert_eq!(contract.status(), &ContractStatus::Agreed);
        assert!(contract.request_digest().starts_with("sha256:"));
        assert!(contract.observation_digest().starts_with("sha256:"));
        assert!(contract.contract_digest().starts_with("sha256:"));
    }

    #[test]
    fn required_absence_blocks_but_optional_absence_does_not_degrade() {
        let mut unavailable = observation();
        unavailable.capabilities.clear();
        let blocked = ExecutionEnvironmentContract::issue(
            request(RequirementClass::Required),
            unavailable.clone(),
        )
        .unwrap();
        assert_eq!(blocked.status(), &ContractStatus::Blocked);
        let optional =
            ExecutionEnvironmentContract::issue(request(RequirementClass::Optional), unavailable)
                .unwrap();
        assert_eq!(optional.status(), &ContractStatus::Agreed);
    }

    #[test]
    fn preferred_absence_is_degraded() {
        let mut unavailable = observation();
        unavailable.capabilities.clear();
        let contract =
            ExecutionEnvironmentContract::issue(request(RequirementClass::Preferred), unavailable)
                .unwrap();
        assert_eq!(contract.status(), &ContractStatus::Degraded);
    }

    #[test]
    fn worker_cannot_self_appoint() {
        let mut request = request(RequirementClass::Required);
        request.worker_assignment.selected_by = "deepseek-pro-v4-opencode".to_owned();
        assert_eq!(
            ExecutionEnvironmentContract::issue(request, observation())
                .unwrap_err()
                .code,
            "worker_assignment"
        );
    }

    #[test]
    fn permit_requires_exact_program_arguments_and_directory() {
        let contract = issue_agreed();
        let exact = CommandInvocation {
            command_id: "rust-check".to_owned(),
            program_path: "C:/Users/Matmus/.cargo/bin/cargo.exe".to_owned(),
            args: vec!["check".to_owned(), "--locked".to_owned()],
            cwd: "D:/Tethers/tethers-0.1/host-rust".to_owned(),
        };
        let permit = contract.permit(exact.clone()).unwrap();
        let config = permit.child_config();
        assert!(config.clear_environment);
        assert!(config.assign_before_execution);
        assert_eq!(config.max_processes, HOST_MAX_SUPERVISED_PROCESSES);
        let mut altered = exact;
        altered.args.push("--release".to_owned());
        assert_eq!(
            contract.permit(altered).unwrap_err().code,
            "command_mismatch"
        );
    }

    #[test]
    fn powershell_command_is_refused_at_issue() {
        let mut observed = observation();
        observed.commands.insert(
            "rust-check".to_owned(),
            HostCommandObservation {
                program_path: "C:/Program Files/PowerShell/7/pwsh.exe".to_owned(),
                args: vec!["-Command".to_owned(), "cargo check".to_owned()],
                cwd: "D:/Tethers/tethers-0.1/host-rust".to_owned(),
                script_digest: None,
                environment: BTreeMap::new(),
            },
        );
        let err =
            ExecutionEnvironmentContract::issue(request(RequirementClass::Required), observed)
                .unwrap_err();
        assert_eq!(err.code, "powershell");
        assert!(err.message.contains("-Command"));
    }

    #[test]
    fn powershell_requires_absolute_script_path() {
        let mut observed = observation();
        observed.commands.insert(
            "rust-check".to_owned(),
            HostCommandObservation {
                program_path: "C:/Program Files/PowerShell/7/pwsh.exe".to_owned(),
                args: vec![
                    "-NoLogo".to_owned(),
                    "-NoProfile".to_owned(),
                    "-NonInteractive".to_owned(),
                    "-File".to_owned(),
                    "check-tethers-task-packet.ps1".to_owned(),
                ],
                cwd: "D:/Tethers".to_owned(),
                script_digest: Some(
                    "sha256:0000000000000000000000000000000000000000000000000000000000000000"
                        .to_owned(),
                ),
                environment: BTreeMap::new(),
            },
        );
        let err =
            ExecutionEnvironmentContract::issue(request(RequirementClass::Required), observed)
                .unwrap_err();
        assert_eq!(err.code, "powershell");
        assert!(err.message.to_ascii_lowercase().contains("absolute"));
    }

    // ── new contract integrity tests ────────────────────────────

    #[test]
    fn contract_integrity_digest_is_recomputed_in_permit() {
        let contract = issue_agreed();
        assert!(contract.permit(exact_invocation()).is_ok());
    }

    #[test]
    fn from_stored_rejects_digest_mismatch() {
        let contract = issue_agreed();
        let serialized =
            serde_json::to_string_pretty(&serde_json::json!({
                "contract_digest": "sha256:0000000000000000000000000000000000000000000000000000000000000000",
                "schema": "tethers-execution-environment-contract-v1",
                "request_id": "x",
                "observation_id": "x",
                "task": { "task_id": "t", "session_id": "s", "scope": [], "repository": { "root": "D:/", "branch": "b", "head": "a".repeat(40) } },
                "worker_assignment": { "selected_by": "Matthew", "worker_id": "luna-opencode" },
                "status": "agreed",
                "capability_resolutions": [],
                "approved_commands": {},
                "granted_permissions": { "filesystem_read": [], "filesystem_write": [], "network_hosts": [], "network_allowed": false, "installation_allowed": false },
                "process_tree_supervision_required": true,
                "request_digest": contract.request_digest(),
                "observation_digest": contract.observation_digest(),
                "max_supervised_processes": 16
            }))
            .unwrap();
        let err = ExecutionEnvironmentContract::from_stored(&serialized).unwrap_err();
        assert_eq!(err.code, "contract_integrity");
    }

    fn tamper_body_and_expect_rejection<F>(mutate: F, expected_code: &str)
    where
        F: FnOnce(&mut serde_json::Value),
    {
        let contract = issue_agreed();
        let mut json = serde_json::to_value(&serde_json::json!({
            "contract_digest": contract.contract_digest(),
            "schema": "tethers-execution-environment-contract-v1",
            "request_id": "request-1",
            "observation_id": "observation-1",
            "task": { "task_id": "J20-ENV-P1", "session_id": "session-1", "scope": ["rust-host"], "repository": { "root": "D:/Tethers", "branch": "codex/example", "head": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" } },
            "worker_assignment": { "selected_by": "Matthew", "worker_id": "deepseek-pro-v4-opencode" },
            "status": "agreed",
            "capability_resolutions": [{ "id": "rust-compilation", "class": "required", "state": "available", "reason": null }],
            "approved_commands": { "rust-check": { "command_id": "rust-check", "program_path": "C:/Users/Matmus/.cargo/bin/cargo.exe", "args": ["check", "--locked"], "cwd": "D:/Tethers/tethers-0.1/host-rust", "script_digest": null, "script_path": null, "environment": {} } },
            "granted_permissions": { "filesystem_read": ["D:/Tethers"], "filesystem_write": ["D:/Tethers"], "network_hosts": ["github.com"], "network_allowed": true, "installation_allowed": false },
            "process_tree_supervision_required": true,
            "request_digest": contract.request_digest(),
            "observation_digest": contract.observation_digest(),
            "max_supervised_processes": 16
        }))
        .unwrap();
        mutate(&mut json);
        let serialized = serde_json::to_string(&json).unwrap();
        let err = ExecutionEnvironmentContract::from_stored(&serialized).unwrap_err();
        assert_eq!(
            err.code, expected_code,
            "expected code '{expected_code}', got '{}' for {serialized}",
            err.code
        );
    }

    #[test]
    fn tampered_status_is_rejected() {
        tamper_body_and_expect_rejection(
            |json| {
                json["status"] = serde_json::Value::String("blocked".to_owned());
            },
            "contract_integrity",
        );
    }

    #[test]
    fn tampered_program_path_is_rejected() {
        tamper_body_and_expect_rejection(
            |json| {
                json["approved_commands"]["rust-check"]["program_path"] =
                    serde_json::Value::String("C:/evil.exe".to_owned());
            },
            "contract_integrity",
        );
    }

    #[test]
    fn tampered_arguments_are_rejected() {
        tamper_body_and_expect_rejection(
            |json| {
                json["approved_commands"]["rust-check"]["args"] = serde_json::json!(["malicious"]);
            },
            "contract_integrity",
        );
    }

    #[test]
    fn tampered_cwd_is_rejected() {
        tamper_body_and_expect_rejection(
            |json| {
                json["approved_commands"]["rust-check"]["cwd"] =
                    serde_json::Value::String("C:/hacked".to_owned());
            },
            "contract_integrity",
        );
    }

    #[test]
    fn tampered_request_digest_is_rejected() {
        tamper_body_and_expect_rejection(
            |json| {
                json["request_digest"] = serde_json::Value::String(
                    "sha256:0000000000000000000000000000000000000000000000000000000000000000"
                        .to_owned(),
                );
            },
            "contract_integrity",
        );
    }

    #[test]
    fn tampered_observation_digest_is_rejected() {
        tamper_body_and_expect_rejection(
            |json| {
                json["observation_digest"] = serde_json::Value::String(
                    "sha256:0000000000000000000000000000000000000000000000000000000000000000"
                        .to_owned(),
                );
            },
            "contract_integrity",
        );
    }

    // ── duplicate rejection tests ───────────────────────────────

    #[test]
    fn duplicate_capability_ids_are_rejected() {
        let mut req = request(RequirementClass::Required);
        req.capabilities.push(CapabilityRequirement {
            id: "rust-compilation".to_owned(),
            class: RequirementClass::Optional,
            version_policy: VersionPolicy::Any,
        });
        let err = ExecutionEnvironmentContract::issue(req, observation()).unwrap_err();
        assert_eq!(err.code, "duplicate_capability");
    }

    #[test]
    fn duplicate_command_ids_are_rejected() {
        let mut req = request(RequirementClass::Required);
        req.commands.push(RequestedCommand {
            command_id: "rust-check".to_owned(),
            capability_id: "rust-formatting".to_owned(),
            args: vec!["fmt".to_owned()],
            cwd: "D:/Tethers".to_owned(),
        });
        let err = ExecutionEnvironmentContract::issue(req, observation()).unwrap_err();
        assert_eq!(err.code, "duplicate_command");
    }

    // ── Windows workbench enforcement tests ─────────────────────

    #[test]
    fn non_windows_platform_is_rejected() {
        let mut obs = observation();
        obs.platform = "linux".to_owned();
        let err = ExecutionEnvironmentContract::issue(request(RequirementClass::Required), obs)
            .unwrap_err();
        assert_eq!(err.code, "platform");
    }

    #[test]
    fn non_pwsh_shell_is_rejected() {
        let mut obs = observation();
        obs.shell = "bash".to_owned();
        let err = ExecutionEnvironmentContract::issue(request(RequirementClass::Required), obs)
            .unwrap_err();
        assert_eq!(err.code, "shell");
    }

    #[test]
    fn relative_program_path_is_rejected() {
        let mut obs = observation();
        obs.commands.insert(
            "rust-check".to_owned(),
            HostCommandObservation {
                program_path: "cargo.exe".to_owned(),
                args: vec!["check".to_owned()],
                cwd: "D:/Tethers".to_owned(),
                script_digest: None,
                environment: BTreeMap::new(),
            },
        );
        let err = ExecutionEnvironmentContract::issue(request(RequirementClass::Required), obs)
            .unwrap_err();
        assert_eq!(err.code, "program_path");
    }

    #[test]
    fn relative_cwd_is_rejected() {
        let mut obs = observation();
        obs.commands.insert(
            "rust-check".to_owned(),
            HostCommandObservation {
                program_path: "C:/cargo.exe".to_owned(),
                args: vec!["check".to_owned()],
                cwd: "tethers-0.1/host-rust".to_owned(),
                script_digest: None,
                environment: BTreeMap::new(),
            },
        );
        let err = ExecutionEnvironmentContract::issue(request(RequirementClass::Required), obs)
            .unwrap_err();
        assert_eq!(err.code, "cwd");
    }

    // ── canonical scope enforcement tests ────────────────────────

    #[test]
    fn cwd_outside_granted_scope_is_rejected() {
        let mut obs = observation();
        obs.commands.insert(
            "rust-check".to_owned(),
            HostCommandObservation {
                program_path: "C:/Users/Matmus/.cargo/bin/cargo.exe".to_owned(),
                args: vec!["check".to_owned()],
                cwd: "C:/Elsewhere".to_owned(),
                script_digest: None,
                environment: BTreeMap::new(),
            },
        );
        let err = ExecutionEnvironmentContract::issue(request(RequirementClass::Required), obs)
            .unwrap_err();
        assert_eq!(err.code, "scope");
        assert!(err.message.to_ascii_lowercase().contains("outside"));
    }

    #[test]
    fn dot_dot_traversal_outside_scope_is_rejected() {
        let mut obs = observation();
        obs.commands.insert(
            "rust-check".to_owned(),
            HostCommandObservation {
                program_path: "C:/Users/Matmus/.cargo/bin/cargo.exe".to_owned(),
                args: vec!["check".to_owned()],
                cwd: "D:/Tethers/../Evil".to_owned(),
                script_digest: None,
                environment: BTreeMap::new(),
            },
        );
        let err = ExecutionEnvironmentContract::issue(request(RequirementClass::Required), obs)
            .unwrap_err();
        assert_eq!(err.code, "scope");
    }

    // ── substitute deferral tests ───────────────────────────────

    #[test]
    fn substitutes_are_deferred_not_resolved() {
        let obs = observation();
        let mut req = request(RequirementClass::Preferred);
        req.capabilities[0].id = "recursive-text-search".to_owned();
        let contract = ExecutionEnvironmentContract::issue(req, obs).unwrap();
        assert_eq!(contract.status(), &ContractStatus::Degraded);
        let resolution = &contract.data.capability_resolutions[0];
        assert_eq!(resolution.state, CapabilityState::Degraded);
    }

    // ── from_stored round-trip test ─────────────────────────────

    #[test]
    fn from_stored_round_trips_valid_contract() {
        let contract = issue_agreed();
        let serialized = serde_json::to_string_pretty(&serde_json::json!({
            "contract_digest": contract.contract_digest(),
            "schema": "tethers-execution-environment-contract-v1",
            "request_id": "request-1",
            "observation_id": "observation-1",
            "task": { "task_id": "J20-ENV-P1", "session_id": "session-1", "scope": ["rust-host"], "repository": { "root": "D:/Tethers", "branch": "codex/example", "head": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" } },
            "worker_assignment": { "selected_by": "Matthew", "worker_id": "deepseek-pro-v4-opencode" },
            "status": "agreed",
            "capability_resolutions": [{ "id": "rust-compilation", "class": "required", "state": "available", "reason": null }],
            "approved_commands": { "rust-check": { "command_id": "rust-check", "program_path": "C:/Users/Matmus/.cargo/bin/cargo.exe", "args": ["check", "--locked"], "cwd": "D:/Tethers/tethers-0.1/host-rust", "script_digest": null, "script_path": null, "environment": {} } },
            "granted_permissions": { "filesystem_read": ["D:/Tethers"], "filesystem_write": ["D:/Tethers"], "network_hosts": ["github.com"], "network_allowed": true, "installation_allowed": false },
            "process_tree_supervision_required": true,
            "request_digest": contract.request_digest(),
            "observation_digest": contract.observation_digest(),
            "max_supervised_processes": 16
        }))
        .unwrap();
        let restored = ExecutionEnvironmentContract::from_stored(&serialized).unwrap();
        assert_eq!(restored.contract_digest(), contract.contract_digest());
        assert_eq!(restored.status(), &ContractStatus::Agreed);
        assert!(restored.permit(exact_invocation()).is_ok());
    }

    fn exact_invocation() -> CommandInvocation {
        CommandInvocation {
            command_id: "rust-check".to_owned(),
            program_path: "C:/Users/Matmus/.cargo/bin/cargo.exe".to_owned(),
            args: vec!["check".to_owned(), "--locked".to_owned()],
            cwd: "D:/Tethers/tethers-0.1/host-rust".to_owned(),
        }
    }

    // ── native Windows supervised-launch tests ───────────────────

    #[cfg(windows)]
    #[test]
    fn supervised_launch_runs_descendant_completes_and_leaves_no_survivor() {
        let cmd_path = "C:/Windows/System32/cmd.exe";
        let temp = std::env::temp_dir().to_string_lossy().replace('/', "\\");
        let cwd = temp.clone();

        // The command spawns a descendant: cmd /c start /b /wait cmd /c exit 0
        let args = vec!["/c".to_owned(), "start /b /wait cmd /c exit 0".to_owned()];

        let mut req = request(RequirementClass::Required);
        req.capabilities[0] = CapabilityRequirement {
            id: "powershell-automation".to_owned(),
            class: RequirementClass::Required,
            version_policy: VersionPolicy::Any,
        };
        req.commands = vec![RequestedCommand {
            command_id: "cmd-descendant".to_owned(),
            capability_id: "powershell-automation".to_owned(),
            args: args.clone(),
            cwd: cwd.clone(),
        }];
        req.requested_permissions = broad_permissions();

        let mut obs = observation();
        obs.granted_permissions = broad_permissions();
        obs.capabilities = BTreeMap::from([(
            "powershell-automation".to_owned(),
            HostCapabilityObservation {
                version: "10.0".to_owned(),
                verified: true,
                command_id: "cmd-descendant".to_owned(),
            },
        )]);
        obs.commands = BTreeMap::from([(
            "cmd-descendant".to_owned(),
            HostCommandObservation {
                program_path: cmd_path.to_owned(),
                args: args.clone(),
                cwd: cwd.clone(),
                script_digest: None,
                environment: BTreeMap::from([("TETHERS_TEST".to_owned(), "present".to_owned())]),
            },
        )]);

        let contract = ExecutionEnvironmentContract::issue(req, obs).unwrap();
        assert_eq!(contract.status(), &ContractStatus::Agreed);

        let invocation = CommandInvocation {
            command_id: "cmd-descendant".to_owned(),
            program_path: cmd_path.to_owned(),
            args,
            cwd,
        };
        let permit = contract.permit(invocation).unwrap();
        let config = permit.child_config();
        assert!(config.clear_environment);
        assert_eq!(config.max_processes, HOST_MAX_SUPERVISED_PROCESSES);
        assert!(config.assign_before_execution);
        assert_eq!(
            config.environment.get("TETHERS_TEST"),
            Some(&"present".to_owned())
        );

        let child = permit.launch().unwrap();
        child.shutdown();
    }

    #[cfg(windows)]
    #[test]
    fn altered_command_is_refused_in_supervised_launch() {
        let contract = issue_agreed();
        let mut altered = exact_invocation();
        altered.args.push("--release".to_owned());
        let err = contract.permit(altered).unwrap_err();
        assert_eq!(err.code, "command_mismatch");
    }

    #[cfg(windows)]
    #[test]
    fn powershell_file_permit_and_launch_succeeds() {
        use std::io::Write;

        let Some(pwsh) = resolve_pwsh() else {
            panic!("pwsh.exe not found — integration test unavailable");
        };
        let temp = std::env::temp_dir();
        let script_path = temp.join("tethers-test-script.ps1");
        let script_content = "exit 0";
        {
            let mut file = fs::File::create(&script_path).unwrap();
            file.write_all(script_content.as_bytes()).unwrap();
        }
        let script_str = script_path.to_string_lossy().replace('/', "\\");
        let cwd = temp.to_string_lossy().replace('/', "\\");

        let script_hash = format!("sha256:{}", hash_file(&script_str).unwrap());

        let args = vec![
            "-NoLogo".to_owned(),
            "-NoProfile".to_owned(),
            "-NonInteractive".to_owned(),
            "-File".to_owned(),
            script_str.clone(),
        ];

        let mut req = request(RequirementClass::Required);
        req.capabilities[0] = CapabilityRequirement {
            id: "powershell-automation".to_owned(),
            class: RequirementClass::Required,
            version_policy: VersionPolicy::Any,
        };
        req.commands = vec![RequestedCommand {
            command_id: "pwsh-test".to_owned(),
            capability_id: "powershell-automation".to_owned(),
            args: args.clone(),
            cwd: cwd.clone(),
        }];
        req.requested_permissions = broad_permissions();

        let mut obs = observation();
        obs.granted_permissions = broad_permissions();
        obs.capabilities = BTreeMap::from([(
            "powershell-automation".to_owned(),
            HostCapabilityObservation {
                version: "7.6.4".to_owned(),
                verified: true,
                command_id: "pwsh-test".to_owned(),
            },
        )]);
        obs.commands = BTreeMap::from([(
            "pwsh-test".to_owned(),
            HostCommandObservation {
                program_path: pwsh.clone(),
                args: args.clone(),
                cwd: cwd.clone(),
                script_digest: Some(script_hash),
                environment: powershell_launch_environment(),
            },
        )]);

        let contract = ExecutionEnvironmentContract::issue(req, obs).unwrap();
        assert_eq!(contract.status(), &ContractStatus::Agreed);

        let invocation = CommandInvocation {
            command_id: "pwsh-test".to_owned(),
            program_path: pwsh,
            args,
            cwd,
        };
        let permit = contract.permit(invocation).unwrap();
        let child = permit.launch().unwrap();
        child.shutdown();

        let _ = fs::remove_file(&script_path);
    }

    #[cfg(windows)]
    #[test]
    fn changed_powershell_script_is_refused_by_permit() {
        use std::io::Write;

        let pwsh = "C:/Program Files/PowerShell/7/pwsh.exe";
        let temp = std::env::temp_dir();
        let script_path = temp.join("tethers-test-refused.ps1");
        let script_content = "exit 0";
        {
            let mut file = fs::File::create(&script_path).unwrap();
            file.write_all(script_content.as_bytes()).unwrap();
        }
        let script_str = script_path.to_string_lossy().replace('/', "\\");
        let cwd = temp.to_string_lossy().replace('/', "\\");
        let script_hash = format!("sha256:{}", hash_file(&script_str).unwrap());

        let args = vec![
            "-NoLogo".to_owned(),
            "-NoProfile".to_owned(),
            "-NonInteractive".to_owned(),
            "-File".to_owned(),
            script_str.clone(),
        ];

        let mut req = request(RequirementClass::Required);
        req.capabilities[0] = CapabilityRequirement {
            id: "powershell-automation".to_owned(),
            class: RequirementClass::Required,
            version_policy: VersionPolicy::Any,
        };
        req.commands = vec![RequestedCommand {
            command_id: "pwsh-refuse".to_owned(),
            capability_id: "powershell-automation".to_owned(),
            args: args.clone(),
            cwd: cwd.clone(),
        }];
        req.requested_permissions = broad_permissions();

        let mut obs = observation();
        obs.granted_permissions = broad_permissions();
        obs.capabilities = BTreeMap::from([(
            "powershell-automation".to_owned(),
            HostCapabilityObservation {
                version: "7.6.4".to_owned(),
                verified: true,
                command_id: "pwsh-refuse".to_owned(),
            },
        )]);
        obs.commands = BTreeMap::from([(
            "pwsh-refuse".to_owned(),
            HostCommandObservation {
                program_path: pwsh.to_owned(),
                args: args.clone(),
                cwd: cwd.clone(),
                script_digest: Some(script_hash),
                environment: BTreeMap::new(),
            },
        )]);

        let contract = ExecutionEnvironmentContract::issue(req, obs).unwrap();
        assert_eq!(contract.status(), &ContractStatus::Agreed);

        // Mutate the script before permit.
        {
            let mut file = fs::File::create(&script_path).unwrap();
            file.write_all(b"exit 1").unwrap();
        }

        let invocation = CommandInvocation {
            command_id: "pwsh-refuse".to_owned(),
            program_path: pwsh.to_owned(),
            args,
            cwd,
        };
        let err = contract.permit(invocation).unwrap_err();
        assert_eq!(err.code, "script_changed");

        let _ = fs::remove_file(&script_path);
    }

    #[cfg(windows)]
    #[test]
    fn missing_powershell_script_is_refused_by_permit() {
        use std::io::Write;

        let pwsh = "C:/Program Files/PowerShell/7/pwsh.exe";
        let temp = std::env::temp_dir();
        let script_path = temp.join("tethers-test-missing.ps1");
        let script_content = "exit 0";
        {
            // Create, hash, then delete before permit.
            let mut file = fs::File::create(&script_path).unwrap();
            file.write_all(script_content.as_bytes()).unwrap();
        }
        let script_str = script_path.to_string_lossy().replace('/', "\\");
        let cwd = temp.to_string_lossy().replace('/', "\\");
        let script_hash = format!("sha256:{}", hash_file(&script_str).unwrap());

        let args = vec![
            "-NoLogo".to_owned(),
            "-NoProfile".to_owned(),
            "-NonInteractive".to_owned(),
            "-File".to_owned(),
            script_str.clone(),
        ];

        let mut req = request(RequirementClass::Required);
        req.capabilities[0] = CapabilityRequirement {
            id: "powershell-automation".to_owned(),
            class: RequirementClass::Required,
            version_policy: VersionPolicy::Any,
        };
        req.commands = vec![RequestedCommand {
            command_id: "pwsh-missing".to_owned(),
            capability_id: "powershell-automation".to_owned(),
            args: args.clone(),
            cwd: cwd.clone(),
        }];
        req.requested_permissions = broad_permissions();

        let mut obs = observation();
        obs.granted_permissions = broad_permissions();
        obs.capabilities = BTreeMap::from([(
            "powershell-automation".to_owned(),
            HostCapabilityObservation {
                version: "7.6.4".to_owned(),
                verified: true,
                command_id: "pwsh-missing".to_owned(),
            },
        )]);
        obs.commands = BTreeMap::from([(
            "pwsh-missing".to_owned(),
            HostCommandObservation {
                program_path: pwsh.to_owned(),
                args: args.clone(),
                cwd: cwd.clone(),
                script_digest: Some(script_hash),
                environment: BTreeMap::new(),
            },
        )]);

        let contract = ExecutionEnvironmentContract::issue(req, obs).unwrap();
        assert_eq!(contract.status(), &ContractStatus::Agreed);

        // Delete the script before permit.
        let _ = fs::remove_file(&script_path);

        let invocation = CommandInvocation {
            command_id: "pwsh-missing".to_owned(),
            program_path: pwsh.to_owned(),
            args,
            cwd,
        };
        let err = contract.permit(invocation).unwrap_err();
        assert_eq!(err.code, "script_digest");
    }

    #[cfg(windows)]
    #[test]
    fn powershell_without_noprofile_is_refused() {
        let pwsh = "C:/Program Files/PowerShell/7/pwsh.exe";
        let mut obs = observation();
        obs.commands.insert(
            "rust-check".to_owned(),
            HostCommandObservation {
                program_path: pwsh.to_owned(),
                args: vec!["-File".to_owned(), "D:/Tethers/script.ps1".to_owned()],
                cwd: "D:/Tethers".to_owned(),
                script_digest: Some(
                    "sha256:0000000000000000000000000000000000000000000000000000000000000000"
                        .to_owned(),
                ),
                environment: BTreeMap::new(),
            },
        );
        let err = ExecutionEnvironmentContract::issue(request(RequirementClass::Required), obs)
            .unwrap_err();
        assert_eq!(err.code, "powershell");
        assert!(err.message.contains("-NoLogo"));
    }

    // ── canonical script-path tests ──────────────────────────────

    fn resolve_pwsh() -> Option<String> {
        // Use the same PATH-resolved executable that the local environment
        // probe reaches, rather than assuming a fixed PowerShell installation.
        let output = std::process::Command::new("where.exe")
            .arg("pwsh.exe")
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        for candidate in String::from_utf8_lossy(&output.stdout).lines() {
            let p = Path::new(candidate);
            if p.exists() && p.metadata().map(|m| m.len() > 0).unwrap_or(false) {
                return Some(p.to_string_lossy().replace('/', "\\"));
            }
        }
        None
    }

    fn powershell_launch_environment() -> BTreeMap<String, String> {
        let system_root = std::env::var("SystemRoot")
            .expect("SystemRoot is required for the verified local PowerShell launch");
        BTreeMap::from([("SystemRoot".to_owned(), system_root)])
    }

    #[cfg(windows)]
    #[test]
    fn stored_script_path_is_canonical_while_args_remain_exact() {
        use std::io::Write;

        let Some(pwsh) = resolve_pwsh() else {
            panic!("pwsh.exe not found — integration test unavailable");
        };
        let temp = std::env::temp_dir();
        let script_path = temp.join("tethers-canonical.ps1");
        {
            let mut file = fs::File::create(&script_path).unwrap();
            file.write_all(b"exit 0").unwrap();
        }
        let script_str = script_path.to_string_lossy().replace('/', "\\");
        let cwd = temp.to_string_lossy().replace('/', "\\");
        let canonical_str = canonicalize_path(&script_str).unwrap();
        let script_hash = format!("sha256:{}", hash_file(&canonical_str).unwrap());

        let args = vec![
            "-NoLogo".to_owned(),
            "-NoProfile".to_owned(),
            "-NonInteractive".to_owned(),
            "-File".to_owned(),
            script_str.clone(),
        ];

        let mut req = request(RequirementClass::Required);
        req.capabilities[0] = CapabilityRequirement {
            id: "powershell-automation".to_owned(),
            class: RequirementClass::Required,
            version_policy: VersionPolicy::Any,
        };
        req.commands = vec![RequestedCommand {
            command_id: "pwsh-canon".to_owned(),
            capability_id: "powershell-automation".to_owned(),
            args: args.clone(),
            cwd: cwd.clone(),
        }];
        req.requested_permissions = broad_permissions();

        let mut obs = observation();
        obs.granted_permissions = broad_permissions();
        obs.capabilities = BTreeMap::from([(
            "powershell-automation".to_owned(),
            HostCapabilityObservation {
                version: "7.6.4".to_owned(),
                verified: true,
                command_id: "pwsh-canon".to_owned(),
            },
        )]);
        obs.commands = BTreeMap::from([(
            "pwsh-canon".to_owned(),
            HostCommandObservation {
                program_path: pwsh,
                args: args.clone(),
                cwd: cwd.clone(),
                script_digest: Some(script_hash),
                environment: BTreeMap::new(),
            },
        )]);

        let contract = ExecutionEnvironmentContract::issue(req, obs).unwrap();
        let approved = &contract.data.approved_commands["pwsh-canon"];
        let stored = approved.script_path.as_ref().unwrap();
        assert_eq!(stored, &canonical_str);

        assert_eq!(approved.args, args);

        let _ = fs::remove_file(&script_path);
    }

    #[cfg(windows)]
    #[test]
    fn script_redirected_is_refused_by_permit() {
        use std::io::Write;

        let Some(pwsh) = resolve_pwsh() else {
            panic!("pwsh.exe not found — integration test unavailable");
        };
        let temp = std::env::temp_dir();
        let script_a = temp.join("tethers-redirect-a.ps1");
        let script_b = temp.join("tethers-redirect-b.ps1");
        {
            let mut file = fs::File::create(&script_a).unwrap();
            file.write_all(b"exit 0").unwrap();
        }
        {
            let mut file = fs::File::create(&script_b).unwrap();
            file.write_all(b"exit 0").unwrap();
        }
        let canonical_a = canonicalize_path(&script_a.to_string_lossy()).unwrap();
        let canonical_b = canonicalize_path(&script_b.to_string_lossy()).unwrap();
        let script_hash = format!("sha256:{}", hash_file(&canonical_a).unwrap());
        let cwd = temp.to_string_lossy().replace('/', "\\");

        let args_with_a = vec![
            "-NoLogo".to_owned(),
            "-NoProfile".to_owned(),
            "-NonInteractive".to_owned(),
            "-File".to_owned(),
            canonical_a.clone(),
        ];
        let args_with_b = vec![
            "-NoLogo".to_owned(),
            "-NoProfile".to_owned(),
            "-NonInteractive".to_owned(),
            "-File".to_owned(),
            canonical_b.clone(),
        ];

        let mut req = request(RequirementClass::Required);
        req.capabilities[0] = CapabilityRequirement {
            id: "powershell-automation".to_owned(),
            class: RequirementClass::Required,
            version_policy: VersionPolicy::Any,
        };
        req.commands = vec![RequestedCommand {
            command_id: "pwsh-redir".to_owned(),
            capability_id: "powershell-automation".to_owned(),
            args: args_with_a.clone(),
            cwd: cwd.clone(),
        }];
        req.requested_permissions = broad_permissions();

        let mut obs = observation();
        obs.granted_permissions = broad_permissions();
        obs.capabilities = BTreeMap::from([(
            "powershell-automation".to_owned(),
            HostCapabilityObservation {
                version: "7.6.4".to_owned(),
                verified: true,
                command_id: "pwsh-redir".to_owned(),
            },
        )]);
        obs.commands = BTreeMap::from([(
            "pwsh-redir".to_owned(),
            HostCommandObservation {
                program_path: pwsh.clone(),
                args: args_with_a,
                cwd: cwd.clone(),
                script_digest: Some(script_hash),
                environment: powershell_launch_environment(),
            },
        )]);

        let contract = ExecutionEnvironmentContract::issue(req, obs).unwrap();
        let invocation = CommandInvocation {
            command_id: "pwsh-redir".to_owned(),
            program_path: pwsh,
            args: args_with_b,
            cwd,
        };
        let err = contract.permit(invocation).unwrap_err();
        assert_eq!(err.code, "script_redirected");

        let _ = fs::remove_file(&script_a);
        let _ = fs::remove_file(&script_b);
    }

    #[cfg(windows)]
    #[test]
    fn script_path_redirect_between_permit_and_launch_is_refused() {
        use std::io::Write;

        let Some(pwsh) = resolve_pwsh() else {
            panic!("pwsh.exe not found — integration test unavailable");
        };
        let temp = std::env::temp_dir();
        let unique = format!(
            "{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );

        // Directories with identical script files at different canonical paths.
        let dir_a = temp.join(format!("tethers-junction-race-a-{unique}"));
        let dir_b = temp.join(format!("tethers-junction-race-b-{unique}"));
        let junction = temp.join(format!("tethers-junction-race-link-{unique}"));
        fs::create_dir_all(&dir_a).unwrap();
        fs::create_dir_all(&dir_b).unwrap();
        {
            let mut f = fs::File::create(dir_a.join("script.ps1")).unwrap();
            f.write_all(b"exit 0").unwrap();
        }
        {
            let mut f = fs::File::create(dir_b.join("script.ps1")).unwrap();
            f.write_all(b"exit 0").unwrap();
        }

        // Junction points to dir_a — canonicalise resolves to dir_a.
        create_junction(&junction, &dir_a).unwrap();

        let script_via_junction = junction
            .join("script.ps1")
            .to_string_lossy()
            .replace('/', "\\");
        let canonical_real = canonicalize_path(&script_via_junction).unwrap();
        // The canonical path must reflect the real target directory.
        assert!(
            canonical_real.contains("tethers-junction-race-a-"),
            "expected canonical to resolve to dir_a, got {canonical_real}"
        );

        let script_hash = format!("sha256:{}", hash_file(&canonical_real).unwrap());
        let cwd = temp.to_string_lossy().replace('/', "\\");

        let args = vec![
            "-NoLogo".to_owned(),
            "-NoProfile".to_owned(),
            "-NonInteractive".to_owned(),
            "-File".to_owned(),
            script_via_junction,
        ];

        let mut req = request(RequirementClass::Required);
        req.capabilities[0] = CapabilityRequirement {
            id: "powershell-automation".to_owned(),
            class: RequirementClass::Required,
            version_policy: VersionPolicy::Any,
        };
        req.commands = vec![RequestedCommand {
            command_id: "pwsh-junction-race".to_owned(),
            capability_id: "powershell-automation".to_owned(),
            args: args.clone(),
            cwd: cwd.clone(),
        }];
        req.requested_permissions = broad_permissions();

        let mut obs = observation();
        obs.granted_permissions = broad_permissions();
        obs.capabilities = BTreeMap::from([(
            "powershell-automation".to_owned(),
            HostCapabilityObservation {
                version: "7.6.4".to_owned(),
                verified: true,
                command_id: "pwsh-junction-race".to_owned(),
            },
        )]);
        obs.commands = BTreeMap::from([(
            "pwsh-junction-race".to_owned(),
            HostCommandObservation {
                program_path: pwsh.clone(),
                args: args.clone(),
                cwd: cwd.clone(),
                script_digest: Some(script_hash),
                environment: powershell_launch_environment(),
            },
        )]);

        let contract = ExecutionEnvironmentContract::issue(req, obs).unwrap();
        assert_eq!(
            contract.data.approved_commands["pwsh-junction-race"].args, args,
            "the frozen command must retain the reviewed junction path"
        );
        let invocation = CommandInvocation {
            command_id: "pwsh-junction-race".to_owned(),
            program_path: pwsh,
            args: contract.data.approved_commands["pwsh-junction-race"]
                .args
                .clone(),
            cwd,
        };
        let permit = contract.permit(invocation).unwrap();

        // Retarget the junction to dir_b between permit and launch.
        let _ = fs::remove_dir(&junction);
        create_junction(&junction, &dir_b).unwrap();

        let err = permit.launch().unwrap_err();
        match err {
            ChildError::LaunchFailed { message, .. } => {
                assert!(
                    message.contains("script path redirected"),
                    "expected path-redirect error, got: {message}"
                );
            }
            other => panic!("expected LaunchFailed, got: {other:?}"),
        }

        let _ = fs::remove_dir(&junction);
        let _ = fs::remove_dir_all(&dir_a);
        let _ = fs::remove_dir_all(&dir_b);
    }

    #[cfg(windows)]
    #[test]
    fn valid_canonical_powershell_launch_succeeds() {
        use std::io::Write;

        let Some(pwsh) = resolve_pwsh() else {
            panic!("pwsh.exe not found — integration test unavailable");
        };
        let temp = std::env::temp_dir();
        let script_path = temp.join("tethers-canonical-launch.ps1");
        {
            let mut file = fs::File::create(&script_path).unwrap();
            file.write_all(b"exit 0").unwrap();
        }
        let canonical_str = canonicalize_path(&script_path.to_string_lossy()).unwrap();
        let script_hash = format!("sha256:{}", hash_file(&canonical_str).unwrap());
        let cwd = temp.to_string_lossy().replace('/', "\\");

        let args = vec![
            "-NoLogo".to_owned(),
            "-NoProfile".to_owned(),
            "-NonInteractive".to_owned(),
            "-File".to_owned(),
            canonical_str.clone(),
        ];

        let mut req = request(RequirementClass::Required);
        req.capabilities[0] = CapabilityRequirement {
            id: "powershell-automation".to_owned(),
            class: RequirementClass::Required,
            version_policy: VersionPolicy::Any,
        };
        req.commands = vec![RequestedCommand {
            command_id: "pwsh-canon-launch".to_owned(),
            capability_id: "powershell-automation".to_owned(),
            args: args.clone(),
            cwd: cwd.clone(),
        }];
        req.requested_permissions = broad_permissions();

        let mut obs = observation();
        obs.granted_permissions = broad_permissions();
        obs.capabilities = BTreeMap::from([(
            "powershell-automation".to_owned(),
            HostCapabilityObservation {
                version: "7.6.4".to_owned(),
                verified: true,
                command_id: "pwsh-canon-launch".to_owned(),
            },
        )]);
        obs.commands = BTreeMap::from([(
            "pwsh-canon-launch".to_owned(),
            HostCommandObservation {
                program_path: pwsh.clone(),
                args,
                cwd,
                script_digest: Some(script_hash),
                environment: powershell_launch_environment(),
            },
        )]);

        let contract = ExecutionEnvironmentContract::issue(req, obs).unwrap();
        assert_eq!(contract.status(), &ContractStatus::Agreed);

        let approved = &contract.data.approved_commands["pwsh-canon-launch"];
        let invocation = CommandInvocation {
            command_id: "pwsh-canon-launch".to_owned(),
            program_path: pwsh,
            args: approved.args.clone(),
            cwd: approved.cwd.clone(),
        };
        let permit = contract.permit(invocation).unwrap();
        let child = permit.launch().unwrap();
        child.shutdown();

        let _ = fs::remove_file(&script_path);
    }
}
