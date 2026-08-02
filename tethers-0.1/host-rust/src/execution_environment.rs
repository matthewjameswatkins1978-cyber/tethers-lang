//! Host-owned execution-environment handshake for development tasks.
//!
//! This is intentionally outside the OCaml Core.  It binds one selected worker,
//! one task/session, one repository state, and exact command arrays to a single
//! host observation.  Command launch goes through `SupervisedChild`; matching an
//! executable name alone never grants execution authority.

use crate::child_process::{ChildConfig, ChildError, SupervisedChild};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::PathBuf;
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
    /// Only predeclared substitutes are eligible. The host never invents one.
    #[serde(default)]
    pub substitutes: Vec<String>,
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
    /// Required for a PowerShell `-File` command, and must identify reviewed bytes.
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
    pub environment: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContractStatus {
    Agreed,
    Degraded,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractBody {
    pub schema: String,
    pub request_id: String,
    pub observation_id: String,
    pub task: TaskBinding,
    pub worker_assignment: WorkerAssignment,
    pub status: ContractStatus,
    pub capability_resolutions: Vec<CapabilityResolution>,
    pub approved_commands: BTreeMap<String, ApprovedCommand>,
    pub granted_permissions: PermissionScopes,
    pub process_tree_supervision_required: bool,
    pub request_digest: String,
    pub observation_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionEnvironmentContract {
    #[serde(flatten)]
    pub body: ContractBody,
    /// Digest of `body`; this is the immutable contract identity.
    pub contract_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandInvocation {
    pub command_id: String,
    pub program_path: String,
    pub args: Vec<String>,
    pub cwd: String,
}

/// A non-cloneable permission to launch exactly one contract-approved process.
#[derive(Debug)]
pub struct CommandPermit {
    command: ApprovedCommand,
}

impl CommandPermit {
    pub fn child_config(&self) -> ChildConfig {
        ChildConfig {
            command: self.command.program_path.clone(),
            args: self.command.args.clone(),
            current_dir: Some(PathBuf::from(&self.command.cwd)),
            startup_timeout: Duration::from_secs(10),
            graceful_close_timeout: Duration::from_secs(2),
            max_protocol_line_bytes: 8 * 1024 * 1024,
            stderr_tail_bytes: 64 * 1024,
            // A frozen contract does not inherit accidental shell state.
            clear_environment: true,
            environment: self.command.environment.clone(),
            max_processes: 1,
            process_memory_limit_bytes: 512 * 1024 * 1024,
            // On Windows this puts the process in the Job Object before code runs.
            assign_before_execution: true,
        }
    }

    pub fn launch(self) -> Result<SupervisedChild, ChildError> {
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

fn version_satisfies(actual: &str, policy: &VersionPolicy) -> bool {
    match policy {
        VersionPolicy::Any => true,
        VersionPolicy::Exact { version } => actual == version,
        // Tool versions are host-probed and dotted numeric versions in this profile.
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

fn validates_powershell(command: &HostCommandObservation) -> bool {
    let is_powershell = command
        .program_path
        .to_ascii_lowercase()
        .ends_with("pwsh.exe")
        || command.program_path.eq_ignore_ascii_case("pwsh");
    if !is_powershell {
        return true;
    }
    let arguments = command
        .args
        .iter()
        .map(|argument| argument.to_ascii_lowercase())
        .collect::<Vec<_>>();
    !arguments
        .iter()
        .any(|argument| argument == "-command" || argument == "-encodedcommand")
        && arguments.iter().any(|argument| argument == "-file")
        && command.script_digest.is_some()
}

fn capability_failure(class: &RequirementClass) -> CapabilityState {
    match class {
        RequirementClass::Required => CapabilityState::Unavailable,
        RequirementClass::Preferred | RequirementClass::Replaceable => CapabilityState::Degraded,
        RequirementClass::Optional => CapabilityState::Unavailable,
    }
}

impl ExecutionEnvironmentContract {
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
                || !validates_powershell(observed)
            {
                resolutions.push(CapabilityResolution {
                    id: requirement.id.clone(),
                    class: requirement.class.clone(),
                    state: capability_failure(&requirement.class),
                    reason: Some("command is not an exact reviewed host command".to_owned()),
                });
                continue;
            }
            approved_commands.insert(
                requested.command_id.clone(),
                ApprovedCommand {
                    command_id: requested.command_id.clone(),
                    program_path: observed.program_path.clone(),
                    args: observed.args.clone(),
                    cwd: observed.cwd.clone(),
                    script_digest: observed.script_digest.clone(),
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
        let body = ContractBody {
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
        };
        let contract_digest = canonical_digest(&body)?;
        Ok(Self {
            body,
            contract_digest,
        })
    }

    pub fn permit(&self, invocation: CommandInvocation) -> Result<CommandPermit, EnvironmentError> {
        if self.body.status == ContractStatus::Blocked {
            return Err(EnvironmentError::new(
                "contract_blocked",
                "blocked contracts cannot launch a process",
            ));
        }
        if !self.body.process_tree_supervision_required {
            return Err(EnvironmentError::new(
                "supervision",
                "contract lacks required process-tree supervision",
            ));
        }
        let Some(command) = self.body.approved_commands.get(&invocation.command_id) else {
            return Err(EnvironmentError::new(
                "command",
                "command id is not in the frozen contract",
            ));
        };
        if command.program_path != invocation.program_path
            || command.args != invocation.args
            || command.cwd != invocation.cwd
        {
            return Err(EnvironmentError::new(
                "command_mismatch",
                "program, arguments, and working directory must exactly match the contract",
            ));
        }
        Ok(CommandPermit {
            command: command.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn permissions() -> PermissionScopes {
        PermissionScopes {
            filesystem_read: BTreeSet::from(["D:/Tethers".to_owned()]),
            filesystem_write: BTreeSet::from(["D:/Tethers".to_owned()]),
            network_hosts: BTreeSet::from(["github.com".to_owned()]),
            network_allowed: true,
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
                substitutes: vec![],
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

    #[test]
    fn issues_an_immutable_contract_with_all_three_digests() {
        let contract =
            ExecutionEnvironmentContract::issue(request(RequirementClass::Required), observation())
                .unwrap();
        assert_eq!(contract.body.status, ContractStatus::Agreed);
        assert!(contract.body.request_digest.starts_with("sha256:"));
        assert!(contract.body.observation_digest.starts_with("sha256:"));
        assert!(contract.contract_digest.starts_with("sha256:"));
        assert_eq!(
            contract.contract_digest,
            canonical_digest(&contract.body).unwrap()
        );
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
        assert_eq!(blocked.body.status, ContractStatus::Blocked);
        let optional =
            ExecutionEnvironmentContract::issue(request(RequirementClass::Optional), unavailable)
                .unwrap();
        assert_eq!(optional.body.status, ContractStatus::Agreed);
        assert_eq!(
            optional.body.capability_resolutions[0].state,
            CapabilityState::Unavailable
        );
    }

    #[test]
    fn preferred_absence_is_degraded() {
        let mut unavailable = observation();
        unavailable.capabilities.clear();
        let contract =
            ExecutionEnvironmentContract::issue(request(RequirementClass::Preferred), unavailable)
                .unwrap();
        assert_eq!(contract.body.status, ContractStatus::Degraded);
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
        let contract =
            ExecutionEnvironmentContract::issue(request(RequirementClass::Required), observation())
                .unwrap();
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
        let mut altered = exact;
        altered.args.push("--release".to_owned());
        assert_eq!(
            contract.permit(altered).unwrap_err().code,
            "command_mismatch"
        );
    }

    #[test]
    fn powershell_requires_file_and_reviewed_script_digest() {
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
        let contract =
            ExecutionEnvironmentContract::issue(request(RequirementClass::Required), observed)
                .unwrap();
        assert_eq!(contract.body.status, ContractStatus::Blocked);
    }
}
