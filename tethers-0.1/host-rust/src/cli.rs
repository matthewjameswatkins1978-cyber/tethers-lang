// J13 CLI: strict clap 4 command-line parsing with explicit routes.
// Outcome vocabulary with matching status/exit_code always consistent.

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "tethers-reference-host",
    version = "0.1.0",
    about = "Tethers Reference Host",
    disable_help_subcommand = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
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
}
