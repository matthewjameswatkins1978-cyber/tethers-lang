// J13 CLI: strict clap 4 command-line parsing with explicit routes.
// Outcome vocabulary with matching status/exit_code always consistent.

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "tethers-reference-host",
    version = env!("CARGO_PKG_VERSION"),
    about = "Tethers Reference Host",
    disable_help_subcommand = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Inspect a Plug package without extracting, installing, or executing it.
    Plug {
        #[command(subcommand)]
        command: PlugCommand,
    },
    /// Validate Tether source, engine, and provider availability.
    Check {
        #[arg(long = "config", value_name = "PATH")]
        config: PathBuf,
        #[arg(long = "engine", value_name = "PATH")]
        engine: PathBuf,
    },

    /// Evaluate one explicit external event against one configured Tether.
    Run {
        #[arg(long = "config", value_name = "PATH")]
        config: PathBuf,
        #[arg(long = "engine", value_name = "PATH")]
        engine: PathBuf,
        #[arg(long = "input", value_name = "PATH")]
        input: PathBuf,
        #[arg(long = "trail", value_name = "ABSOLUTE_PATH")]
        trail: PathBuf,
        #[arg(long = "host-data-root", value_name = "ABSOLUTE_PATH")]
        host_data_root: PathBuf,
    },

    /// Hidden legacy positional compatibility route.
    #[command(hide = true)]
    #[clap(name = "__legacy")]
    Legacy {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true, num_args = 0..)]
        args: Vec<String>,
    },

    /// Hidden provision-replay administrative command.
    #[command(hide = true)]
    #[clap(name = "provision-replay")]
    ProvisionReplay {
        #[arg(value_name = "ABSOLUTE_HOST_DATA_ROOT")]
        root: PathBuf,
    },

    /// Debug-only event-admission probe.
    #[command(hide = true)]
    #[clap(name = "event-admission-probe")]
    EventAdmissionProbe {
        #[arg(value_name = "PROBE_MODE")]
        mode: String,
    },

    /// Debug-only event-admission trail probe.
    #[command(hide = true)]
    #[clap(name = "event-admission-trail-probe")]
    EventAdmissionTrailProbe {
        #[arg(value_name = "PROBE_MODE")]
        mode: String,
        #[arg(value_name = "ABSOLUTE_TRAIL_PATH")]
        trail_path: PathBuf,
    },

    /// Read and filter a Trail by execution identity.
    Trail {
        #[arg(long = "trail", value_name = "ABSOLUTE_PATH")]
        trail: PathBuf,
        #[arg(long = "execution-id", value_name = "exec_UUID")]
        execution_id: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum PlugCommand {
    /// Inspect one .tetherplug package as hostile, read-only data.
    Inspect {
        #[arg(long = "package", value_name = "PATH")]
        package: PathBuf,
    },
    /// List installed Plug identities without changing lifecycle state.
    List {
        #[arg(long = "host-data-root", value_name = "ABSOLUTE_PATH")]
        host_data_root: PathBuf,
    },
    /// Disable one exact currently-enabled installed Plug.
    Disable {
        #[arg(long = "host-data-root", value_name = "ABSOLUTE_PATH")]
        host_data_root: PathBuf,
        #[arg(long = "installed-id", value_name = "UUID")]
        installed_id: String,
    },
    /// Enable one installed Plug with a permission scope file.
    Enable {
        #[arg(long = "host-data-root", value_name = "ABSOLUTE_PATH")]
        host_data_root: PathBuf,
        #[arg(long = "installed-id", value_name = "UUID")]
        installed_id: String,
        #[arg(long = "scope", value_name = "ABSOLUTE_JSON_PATH")]
        scope: PathBuf,
    },
    /// Prepare or reuse an installation candidate without installing or enabling it.
    Stage {
        #[arg(long = "host-data-root", value_name = "ABSOLUTE_PATH")]
        host_data_root: PathBuf,
        #[arg(long = "package", value_name = "ABSOLUTE_TETHERPLUG_PATH")]
        package: PathBuf,
    },
    /// Install an already-staged candidate through conformance to a disabled installed Plug.
    Install {
        #[arg(long = "host-data-root", value_name = "ABSOLUTE_PATH")]
        host_data_root: PathBuf,
        #[arg(long = "request", value_name = "ABSOLUTE_JSON_PATH")]
        request: PathBuf,
    },
    /// Pack an author source directory into a deterministic .tetherplug.
    Pack {
        #[arg(long = "source", value_name = "ABSOLUTE_DIRECTORY")]
        source: PathBuf,
        #[arg(long = "output", value_name = "ABSOLUTE_FILE")]
        output: PathBuf,
    },
}

/// Outcome status vocabulary with exit codes.
///
/// | exit | status           |
/// |------|------------------|
/// | 0    | ok               |
/// | 0    | completed        |
/// | 0    | denied           |
/// | 0    | no_actions       |
/// | 2    | invalid_cli_usage|
/// | 3    | invalid_data     |
/// | 4    | unavailable      |
/// | 5    | approval_required|
/// | 6    | failed           |
/// | 7    | uncertain        |
/// | 8    | audit_failed     |
/// | 9    | not_found        |
/// | 10   | interrupted      |
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutcomeStatus {
    #[serde(rename = "ok")]
    Ok,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "denied")]
    Denied,
    #[serde(rename = "no_actions")]
    NoActions,
    #[serde(rename = "invalid_cli_usage")]
    InvalidCliUsage,
    #[serde(rename = "invalid_data")]
    InvalidData,
    #[serde(rename = "unavailable")]
    Unavailable,
    #[serde(rename = "approval_required")]
    ApprovalRequired,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "uncertain")]
    Uncertain,
    #[serde(rename = "audit_failed")]
    AuditFailed,
    #[serde(rename = "not_found")]
    NotFound,
    #[serde(rename = "interrupted")]
    Interrupted,
}

impl OutcomeStatus {
    pub const fn exit_code(self) -> i32 {
        match self {
            Self::Ok => 0,
            Self::Completed => 0,
            Self::Denied => 0,
            Self::NoActions => 0,
            Self::InvalidCliUsage => 2,
            Self::InvalidData => 3,
            Self::Unavailable => 4,
            Self::ApprovalRequired => 5,
            Self::Failed => 6,
            Self::Uncertain => 7,
            Self::AuditFailed => 8,
            Self::NotFound => 9,
            Self::Interrupted => 10,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Completed => "completed",
            Self::Denied => "denied",
            Self::NoActions => "no_actions",
            Self::InvalidCliUsage => "invalid_cli_usage",
            Self::InvalidData => "invalid_data",
            Self::Unavailable => "unavailable",
            Self::ApprovalRequired => "approval_required",
            Self::Failed => "failed",
            Self::Uncertain => "uncertain",
            Self::AuditFailed => "audit_failed",
            Self::NotFound => "not_found",
            Self::Interrupted => "interrupted",
        }
    }
}

/// Stable JSON output envelope.
#[derive(Debug, Clone, Serialize)]
pub struct CliEnvelope {
    pub schema: &'static str,
    pub command: String,
    pub status: OutcomeStatus,
    pub exit_code: i32,
    pub data: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<CliError>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CliError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
}

impl CliEnvelope {
    /// Success envelope. Status is Ok, exit_code 0.
    pub fn ok(command: impl Into<String>, data: serde_json::Value) -> Self {
        Self {
            schema: "tethers.cli/1",
            command: command.into(),
            status: OutcomeStatus::Ok,
            exit_code: 0,
            data,
            error: None,
        }
    }

    /// Error envelope. Status and exit_code derive from the same OutcomeStatus.
    /// `data` should contain partial evidence when available.
    pub fn error(
        command: impl Into<String>,
        status: OutcomeStatus,
        code: impl Into<String>,
        message: impl Into<String>,
        field: Option<String>,
    ) -> Self {
        let exit = status.exit_code();
        Self {
            schema: "tethers.cli/1",
            command: command.into(),
            status,
            exit_code: exit,
            data: serde_json::Value::Object(Default::default()),
            error: Some(CliError {
                code: code.into(),
                message: message.into(),
                field,
            }),
        }
    }

    /// Error with partial data evidence.
    pub fn error_with_data(
        command: impl Into<String>,
        status: OutcomeStatus,
        code: impl Into<String>,
        message: impl Into<String>,
        field: Option<String>,
        data: serde_json::Value,
    ) -> Self {
        let exit = status.exit_code();
        Self {
            schema: "tethers.cli/1",
            command: command.into(),
            status,
            exit_code: exit,
            data,
            error: Some(CliError {
                code: code.into(),
                message: message.into(),
                field,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn parse_cli(args: &[&str]) -> Result<Cli, clap::Error> {
        Cli::try_parse_from(std::iter::once("tethers-reference-host").chain(args.iter().copied()))
    }

    #[test]
    fn j13a_valid_check_command() {
        let cli = parse_cli(&["check", "--config", "c.json", "--engine", "e.exe"]).unwrap();
        match cli.command {
            Some(Command::Check { config, engine }) => {
                assert_eq!(config, PathBuf::from("c.json"));
                assert_eq!(engine, PathBuf::from("e.exe"));
            }
            _ => panic!("expected Check"),
        }
    }

    #[test]
    fn j13a_reordered_options() {
        let cli = parse_cli(&["check", "--engine", "e.exe", "--config", "c.json"]).unwrap();
        match cli.command {
            Some(Command::Check { config, engine }) => {
                assert_eq!(config, PathBuf::from("c.json"));
                assert_eq!(engine, PathBuf::from("e.exe"));
            }
            _ => panic!(),
        }
    }

    #[test]
    fn j13a_duplicate_config_rejected() {
        assert!(
            parse_cli(&["check", "--config", "a.json", "--config", "b.json", "--engine", "e"])
                .is_err()
        );
    }

    #[test]
    fn j13a_duplicate_engine_rejected() {
        assert!(
            parse_cli(&["check", "--config", "c.json", "--engine", "a", "--engine", "b"]).is_err()
        );
    }

    #[test]
    fn j13a_missing_config_rejected() {
        assert!(parse_cli(&["check", "--engine", "e.exe"]).is_err());
    }

    #[test]
    fn j13a_missing_engine_rejected() {
        assert!(parse_cli(&["check", "--config", "c.json"]).is_err());
    }

    #[test]
    fn j13a_unknown_option_rejected() {
        assert!(parse_cli(&[
            "check",
            "--config",
            "c.json",
            "--engine",
            "e.exe",
            "--unknown"
        ])
        .is_err());
    }

    #[test]
    fn j13a_unknown_command_rejected() {
        assert!(parse_cli(&["nonexistent"]).is_err());
    }

    #[test]
    fn j13a_misspelled_runn_rejected() {
        let err = parse_cli(&["runn"]).unwrap_err();
        assert!(
            !err.to_string().contains("legacy"),
            "runn must not reach legacy"
        );
    }

    #[test]
    fn j13a_explicit_legacy() {
        let cli = parse_cli(&["__legacy", "engine.exe", "req.json"]).unwrap();
        match cli.command {
            Some(Command::Legacy { args }) => assert_eq!(args, vec!["engine.exe", "req.json"]),
            _ => panic!(),
        }
    }

    #[test]
    fn j13a_hidden_not_in_help() {
        let err = parse_cli(&["--help"]).unwrap_err();
        let msg = err.to_string();
        assert!(!msg.contains("__legacy"));
        assert!(!msg.contains("provision-replay"));
    }

    #[test]
    fn j13a_provision_replay_valid_parse() {
        let cli = parse_cli(&["provision-replay", "C:\\host-data"]).unwrap();
        match cli.command {
            Some(Command::ProvisionReplay { root }) => {
                assert_eq!(root, PathBuf::from("C:\\host-data"))
            }
            _ => panic!("expected ProvisionReplay"),
        }
    }

    #[test]
    fn j13a_provision_replay_missing_root_rejected() {
        assert!(parse_cli(&["provision-replay"]).is_err());
    }

    #[test]
    fn j13a_provision_replay_extra_positional_rejected() {
        assert!(parse_cli(&["provision-replay", "C:\\host-data", "extra"]).is_err());
    }

    #[test]
    fn j13a_provision_replay_unknown_option_rejected() {
        assert!(parse_cli(&["provision-replay", "--host-data-root", "C:\\host-data"]).is_err());
    }

    #[test]
    fn j13a_no_command() {
        assert!(parse_cli(&[]).is_ok()); // returns Ok with command=None
    }

    #[test]
    fn j13a_extra_positional_rejected() {
        assert!(parse_cli(&["check", "--config", "c.json", "--engine", "e.exe", "extra"]).is_err());
    }

    #[test]
    fn j13a_status_exit_code_consistent() {
        // Every status's as_str + exit_code must be consistent.
        for (status, expected_exit, expected_str) in [
            (OutcomeStatus::Ok, 0, "ok"),
            (OutcomeStatus::InvalidCliUsage, 2, "invalid_cli_usage"),
            (OutcomeStatus::InvalidData, 3, "invalid_data"),
            (OutcomeStatus::Unavailable, 4, "unavailable"),
            (OutcomeStatus::Failed, 6, "failed"),
            (OutcomeStatus::Uncertain, 7, "uncertain"),
            (OutcomeStatus::AuditFailed, 8, "audit_failed"),
            (OutcomeStatus::NotFound, 9, "not_found"),
            (OutcomeStatus::Interrupted, 10, "interrupted"),
        ] {
            assert_eq!(status.exit_code(), expected_exit, "{expected_str}");
            assert_eq!(status.as_str(), expected_str);
        }
    }

    #[test]
    fn j13a_envelope_status_match_exit() {
        let env = CliEnvelope::error("test", OutcomeStatus::Interrupted, "INT", "msg", None);
        let json = serde_json::to_string(&env).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["exit_code"].as_i64().unwrap(), 10);
        assert_eq!(v["status"], "interrupted");
        assert!(v["error"]["code"].as_str().unwrap() == "INT");
    }

    #[test]
    fn j13a_envelope_no_timestamp() {
        let env = CliEnvelope::ok("check", serde_json::json!({"x": 1}));
        let json = serde_json::to_string(&env).unwrap();
        assert!(!json.contains("timestamp"));
    }

    #[test]
    fn j13a_unknown_subcommand_remains_rejected() {
        assert!(parse_cli(&["runn"]).is_err());
    }

    #[test]
    fn j13b_run_public_command_requires_its_five_options() {
        assert!(matches!(
            parse_cli(&[
                "run",
                "--config",
                "c.json",
                "--engine",
                "engine.exe",
                "--input",
                "input.json",
                "--trail",
                "C:\\trail.jsonl",
                "--host-data-root",
                "C:\\host-data"
            ])
            .unwrap()
            .command,
            Some(Command::Run { .. })
        ));
        assert!(parse_cli(&["run", "--config", "c.json"]).is_err());
    }

    #[test]
    fn j13a_equal_sign_accepted() {
        let cli = parse_cli(&["check", "--config=c.json", "--engine", "e.exe"]).unwrap();
        match cli.command {
            Some(Command::Check { config, .. }) => assert_eq!(config, PathBuf::from("c.json")),
            _ => panic!(),
        }
    }
    #[test]
    fn j13c_valid_trail_command() {
        let cli = parse_cli(&[
            "trail",
            "--trail",
            "C:\\t.jsonl",
            "--execution-id",
            "exec_00000000-0000-4000-8000-000000000000",
        ])
        .unwrap();
        match cli.command {
            Some(Command::Trail {
                trail,
                execution_id,
            }) => {
                assert_eq!(trail, PathBuf::from("C:\\t.jsonl"));
                assert_eq!(execution_id, "exec_00000000-0000-4000-8000-000000000000");
            }
            _ => panic!("expected Trail"),
        }
    }

    #[test]
    fn j13c_trail_reordered_options() {
        let cli = parse_cli(&[
            "trail",
            "--execution-id",
            "exec_00000000-0000-4000-8000-000000000000",
            "--trail",
            "C:\\t.jsonl",
        ])
        .unwrap();
        match cli.command {
            Some(Command::Trail { trail, .. }) => {
                assert_eq!(trail, PathBuf::from("C:\\t.jsonl"));
            }
            _ => panic!(),
        }
    }

    #[test]
    fn j13c_duplicate_trail_rejected() {
        assert!(parse_cli(&[
            "trail",
            "--trail",
            "a.jsonl",
            "--trail",
            "b.jsonl",
            "--execution-id",
            "exec_00000000-0000-4000-8000-000000000000"
        ])
        .is_err());
    }

    #[test]
    fn j13c_duplicate_execution_id_rejected() {
        assert!(parse_cli(&[
            "trail",
            "--trail",
            "a.jsonl",
            "--execution-id",
            "exec_00000000-0000-4000-8000-000000000000",
            "--execution-id",
            "exec_00000000-0000-4000-8000-000000000001"
        ])
        .is_err());
    }

    #[test]
    fn j13c_missing_trail_rejected() {
        assert!(parse_cli(&[
            "trail",
            "--execution-id",
            "exec_00000000-0000-4000-8000-000000000000"
        ])
        .is_err());
    }

    #[test]
    fn j13c_missing_execution_id_rejected() {
        assert!(parse_cli(&["trail", "--trail", "C:\\t.jsonl"]).is_err());
    }

    #[test]
    fn j13c_unknown_trail_option_rejected() {
        assert!(parse_cli(&[
            "trail",
            "--trail",
            "C:\\t.jsonl",
            "--execution-id",
            "exec_00000000-0000-4000-8000-000000000000",
            "--unknown"
        ])
        .is_err());
    }

    #[test]
    fn j24a_plug_inspect_syntax_is_strict() {
        let cli = parse_cli(&["plug", "inspect", "--package", "package.tetherplug"]).unwrap();
        assert!(matches!(
            cli.command,
            Some(Command::Plug {
                command: PlugCommand::Inspect { package }
            }) if package == PathBuf::from("package.tetherplug")
        ));
        assert!(parse_cli(&["plug", "inspect", "--package=package.tetherplug"]).is_ok());
        assert!(parse_cli(&["plug"]).is_err());
        assert!(parse_cli(&["plug", "inspect"]).is_err());
        assert!(parse_cli(&["plug", "inspect", "--package", "a", "--package", "b"]).is_err());
        assert!(parse_cli(&["plug", "inspect", "--unknown", "a"]).is_err());
        assert!(parse_cli(&["plug", "inspect", "--package", "a", "extra"]).is_err());
    }

    #[test]
    fn j24b_plug_list_syntax_is_strict() {
        assert!(parse_cli(&["plug", "list", "--host-data-root", "C:\\host"]).is_ok());
        assert!(parse_cli(&["plug", "list", "--host-data-root=C:\\host"]).is_ok());
        assert!(parse_cli(&["plug", "list"]).is_err());
        assert!(parse_cli(&[
            "plug",
            "list",
            "--host-data-root",
            "a",
            "--host-data-root",
            "b"
        ])
        .is_err());
        assert!(parse_cli(&["plug", "list", "--host-data-root", "a", "extra"]).is_err());
    }

    #[test]
    fn j24c_plug_disable_syntax_is_strict() {
        assert!(parse_cli(&[
            "plug",
            "disable",
            "--host-data-root",
            "C:\\host",
            "--installed-id",
            "00000000-0000-4000-8000-000000000000"
        ])
        .is_ok());
        assert!(parse_cli(&[
            "plug",
            "disable",
            "--host-data-root=C:\\host",
            "--installed-id=00000000-0000-4000-8000-000000000000"
        ])
        .is_ok());
        assert!(parse_cli(&["plug", "disable"]).is_err());
        assert!(parse_cli(&["plug", "disable", "--host-data-root", "C:\\host"]).is_err());
        assert!(parse_cli(&[
            "plug",
            "disable",
            "--installed-id",
            "00000000-0000-4000-8000-000000000000"
        ])
        .is_err());
        assert!(parse_cli(&[
            "plug",
            "disable",
            "--host-data-root",
            "a",
            "--host-data-root",
            "b",
            "--installed-id",
            "00000000-0000-4000-8000-000000000000"
        ])
        .is_err());
        assert!(parse_cli(&[
            "plug",
            "disable",
            "--host-data-root",
            "C:\\host",
            "--installed-id",
            "a",
            "--installed-id",
            "b"
        ])
        .is_err());
        assert!(parse_cli(&[
            "plug",
            "disable",
            "--host-data-root",
            "C:\\host",
            "--installed-id",
            "00000000-0000-4000-8000-000000000000",
            "extra"
        ])
        .is_err());
        assert!(parse_cli(&[
            "plug",
            "disable",
            "--host-data-root",
            "C:\\host",
            "--installed-id",
            "00000000-0000-4000-8000-000000000000",
            "--unknown"
        ])
        .is_err());
    }

    #[test]
    fn j24d_plug_enable_syntax_is_strict() {
        assert!(parse_cli(&[
            "plug",
            "enable",
            "--host-data-root",
            "C:\\host",
            "--installed-id",
            "00000000-0000-4000-8000-000000000000",
            "--scope",
            "C:\\scope.json"
        ])
        .is_ok());
        assert!(parse_cli(&[
            "plug",
            "enable",
            "--host-data-root=C:\\host",
            "--installed-id=00000000-0000-4000-8000-000000000000",
            "--scope=C:\\scope.json"
        ])
        .is_ok());
        assert!(parse_cli(&["plug", "enable"]).is_err());
        assert!(parse_cli(&[
            "plug",
            "enable",
            "--host-data-root",
            "C:\\host",
            "--installed-id",
            "00000000-0000-4000-8000-000000000000"
        ])
        .is_err());
        assert!(parse_cli(&[
            "plug",
            "enable",
            "--host-data-root",
            "C:\\host",
            "--scope",
            "C:\\scope.json"
        ])
        .is_err());
        assert!(parse_cli(&[
            "plug",
            "enable",
            "--installed-id",
            "00000000-0000-4000-8000-000000000000",
            "--scope",
            "C:\\scope.json"
        ])
        .is_err());
        assert!(parse_cli(&[
            "plug",
            "enable",
            "--host-data-root",
            "a",
            "--host-data-root",
            "b",
            "--installed-id",
            "00000000-0000-4000-8000-000000000000",
            "--scope",
            "C:\\scope.json"
        ])
        .is_err());
        assert!(parse_cli(&[
            "plug",
            "enable",
            "--host-data-root",
            "C:\\host",
            "--installed-id",
            "a",
            "--installed-id",
            "b",
            "--scope",
            "C:\\scope.json"
        ])
        .is_err());
        assert!(parse_cli(&[
            "plug",
            "enable",
            "--host-data-root",
            "C:\\host",
            "--installed-id",
            "00000000-0000-4000-8000-000000000000",
            "--scope",
            "a",
            "--scope",
            "b"
        ])
        .is_err());
        assert!(parse_cli(&[
            "plug",
            "enable",
            "--host-data-root",
            "C:\\host",
            "--installed-id",
            "00000000-0000-4000-8000-000000000000",
            "--scope",
            "C:\\scope.json",
            "extra"
        ])
        .is_err());
        assert!(parse_cli(&[
            "plug",
            "enable",
            "--host-data-root",
            "C:\\host",
            "--installed-id",
            "00000000-0000-4000-8000-000000000000",
            "--scope",
            "C:\\scope.json",
            "--unknown"
        ])
        .is_err());
    }

    #[test]
    fn j24f_plug_stage_syntax_is_strict() {
        assert!(matches!(
            parse_cli(&[
                "plug",
                "stage",
                "--host-data-root",
                "C:\\host",
                "--package",
                "C:\\package.tetherplug"
            ])
            .unwrap()
            .command,
            Some(Command::Plug {
                command: PlugCommand::Stage {
                    host_data_root,
                    package
                }
            }) if host_data_root == PathBuf::from("C:\\host")
                && package == PathBuf::from("C:\\package.tetherplug")
        ));
        assert!(parse_cli(&[
            "plug",
            "stage",
            "--host-data-root=C:\\host",
            "--package=C:\\package.tetherplug"
        ])
        .is_ok());
        assert!(parse_cli(&["plug", "stage"]).is_err());
        assert!(parse_cli(&["plug", "stage", "--host-data-root", "C:\\host"]).is_err());
        assert!(parse_cli(&["plug", "stage", "--package", "C:\\package.tetherplug"]).is_err());
        assert!(parse_cli(&[
            "plug",
            "stage",
            "--host-data-root",
            "a",
            "--host-data-root",
            "b",
            "--package",
            "C:\\package.tetherplug"
        ])
        .is_err());
        assert!(parse_cli(&[
            "plug",
            "stage",
            "--host-data-root",
            "C:\\host",
            "--package",
            "a",
            "--package",
            "b"
        ])
        .is_err());
        assert!(parse_cli(&[
            "plug",
            "stage",
            "--host-data-root",
            "C:\\host",
            "--package",
            "C:\\package.tetherplug",
            "extra"
        ])
        .is_err());
        assert!(parse_cli(&[
            "plug",
            "stage",
            "--host-data-root",
            "C:\\host",
            "--package",
            "C:\\package.tetherplug",
            "--unknown"
        ])
        .is_err());
    }

    #[test]
    fn p2a_plug_pack_syntax_is_strict() {
        assert!(matches!(
            parse_cli(&[
                "plug",
                "pack",
                "--source",
                "C:\\my-plug",
                "--output",
                "C:\\my-plug.tetherplug"
            ])
            .unwrap()
            .command,
            Some(Command::Plug {
                command: PlugCommand::Pack { source, output }
            }) if source == PathBuf::from("C:\\my-plug")
                && output == PathBuf::from("C:\\my-plug.tetherplug")
        ));
        assert!(parse_cli(&[
            "plug",
            "pack",
            "--source=C:\\my-plug",
            "--output=C:\\my-plug.tetherplug"
        ])
        .is_ok());
        assert!(parse_cli(&["plug", "pack"]).is_err());
        assert!(parse_cli(&["plug", "pack", "--source", "C:\\my-plug"]).is_err());
        assert!(parse_cli(&["plug", "pack", "--output", "C:\\my-plug.tetherplug"]).is_err());
        assert!(parse_cli(&[
            "plug",
            "pack",
            "--source",
            "a",
            "--source",
            "b",
            "--output",
            "C:\\my-plug.tetherplug"
        ])
        .is_err());
        assert!(parse_cli(&[
            "plug",
            "pack",
            "--source",
            "C:\\my-plug",
            "--output",
            "a",
            "--output",
            "b"
        ])
        .is_err());
        assert!(parse_cli(&[
            "plug",
            "pack",
            "--source",
            "C:\\my-plug",
            "--output",
            "C:\\my-plug.tetherplug",
            "extra"
        ])
        .is_err());
        assert!(parse_cli(&[
            "plug",
            "pack",
            "--source",
            "C:\\my-plug",
            "--output",
            "C:\\my-plug.tetherplug",
            "--unknown"
        ])
        .is_err());
    }
}
