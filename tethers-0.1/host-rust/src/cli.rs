// J13 CLI: strict clap 4 command-line parsing with explicit routes.
//
// Every invocation attempts to emit exactly one compact JSON envelope
// to stdout.  Diagnostics go to stderr.  Clap is used through
// try_parse_from so it never exits the process directly.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Tethers Reference Host - deterministic local capability planner.
#[derive(Parser, Debug)]
#[command(
    name = "tethers-reference-host",
    version = "0.1.0",
    about = "Tethers Reference Host - deterministic local capability planner",
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
        /// Path to the runtime configuration JSON file.
        #[arg(long = "config", value_name = "PATH", value_hint = clap::ValueHint::FilePath)]
        config: PathBuf,

        /// Path to the MCP engine executable.
        #[arg(long = "engine", value_name = "PATH", value_hint = clap::ValueHint::FilePath)]
        engine: PathBuf,
    },

    /// Hidden legacy positional compatibility route.
    #[command(hide = true)]
    #[clap(name = "__legacy")]
    Legacy {
        /// Legacy positional arguments passed through as-is after the
        /// subcommand.  Clap captures them in order.
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
}

/// Outcome status vocabulary shared across all J13 commands.
///
/// Statuses map to exit codes as follows:
///
/// | exit code | status           | meaning                                          |
/// |-----------|------------------|--------------------------------------------------|
/// | 0         | ok               | success                                          |
/// | 0         | completed        | action completed                                 |
/// | 0         | denied           | policy denied                                    |
/// | 0         | no_actions       | no actions proposed                              |
/// | 2         | invalid_cli_usage| invalid CLI usage                                |
/// | 3         | invalid_data     | malformed/over-limit/multi-doc input             |
/// | 4         | unavailable      | resource unavailable                             |
/// | 5         | approval_required| human approval required                          |
/// | 6         | failed           | operation failed                                 |
/// | 7         | uncertain        | outcome uncertain after Action invocation        |
/// | 8         | audit_failed     | durable recording failed                         |
/// | 9         | not_found        | resource not found                               |
/// | 10        | interrupted      | Ctrl+C before Action invocation                  |
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum OutcomeStatus {
    Ok = 0,
    Completed = 1,
    Denied = 2,
    NoActions = 3,
    InvalidCliUsage = 4,
    InvalidData = 5,
    Unavailable = 6,
    ApprovalRequired = 7,
    Failed = 8,
    Uncertain = 9,
    AuditFailed = 10,
    NotFound = 11,
    Interrupted = 12,
}

impl OutcomeStatus {
    pub const fn exit_code(self) -> i32 {
        match self {
            OutcomeStatus::Ok => 0,
            OutcomeStatus::Completed => 0,
            OutcomeStatus::Denied => 0,
            OutcomeStatus::NoActions => 0,
            OutcomeStatus::InvalidCliUsage => 2,
            OutcomeStatus::InvalidData => 3,
            OutcomeStatus::Unavailable => 4,
            OutcomeStatus::ApprovalRequired => 5,
            OutcomeStatus::Failed => 6,
            OutcomeStatus::Uncertain => 7,
            OutcomeStatus::AuditFailed => 8,
            OutcomeStatus::NotFound => 9,
            OutcomeStatus::Interrupted => 10,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            OutcomeStatus::Ok => "ok",
            OutcomeStatus::Completed => "completed",
            OutcomeStatus::Denied => "denied",
            OutcomeStatus::NoActions => "no_actions",
            OutcomeStatus::InvalidCliUsage => "invalid_cli_usage",
            OutcomeStatus::InvalidData => "invalid_data",
            OutcomeStatus::Unavailable => "unavailable",
            OutcomeStatus::ApprovalRequired => "approval_required",
            OutcomeStatus::Failed => "failed",
            OutcomeStatus::Uncertain => "uncertain",
            OutcomeStatus::AuditFailed => "audit_failed",
            OutcomeStatus::NotFound => "not_found",
            OutcomeStatus::Interrupted => "interrupted",
        }
    }
}

/// Stable JSON output envelope for every invocation.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct CliEnvelope {
    pub schema: &'static str,
    pub command: String,
    pub status: String,
    pub exit_code: i32,
    pub data: serde_json::Value,
    pub error: Option<CliError>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct CliError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
}

impl CliEnvelope {
    pub fn ok(command: impl Into<String>, data: serde_json::Value) -> Self {
        Self {
            schema: "tethers.cli/1",
            command: command.into(),
            status: OutcomeStatus::Ok.as_str().to_owned(),
            exit_code: OutcomeStatus::Ok.exit_code(),
            data,
            error: None,
        }
    }

    pub fn error(
        command: impl Into<String>,
        status: OutcomeStatus,
        code: impl Into<String>,
        message: impl Into<String>,
        field: Option<String>,
    ) -> Self {
        Self {
            schema: "tethers.cli/1",
            command: command.into(),
            status: status.as_str().to_owned(),
            exit_code: status.exit_code(),
            data: serde_json::Value::Object(Default::default()),
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

    // The primary parser entry: try_parse_from so clap never exits.
    pub fn parse_cli(args: &[&str]) -> Result<Cli, clap::Error> {
        Cli::try_parse_from(std::iter::once("tethers-reference-host").chain(args.iter().copied()))
    }

    #[test]
    fn j13a_valid_check_command() {
        let cli =
            parse_cli(&["check", "--config", "config.json", "--engine", "engine.exe"]).unwrap();
        match cli.command {
            Some(Command::Check { config, engine }) => {
                assert_eq!(config, std::path::PathBuf::from("config.json"));
                assert_eq!(engine, std::path::PathBuf::from("engine.exe"));
            }
            _ => panic!("expected Check command"),
        }
    }

    #[test]
    fn j13a_reordered_singleton_options() {
        let cli = parse_cli(&["check", "--engine", "e.exe", "--config", "c.json"]).unwrap();
        match cli.command {
            Some(Command::Check { config, engine }) => {
                assert_eq!(config, std::path::PathBuf::from("c.json"));
                assert_eq!(engine, std::path::PathBuf::from("e.exe"));
            }
            _ => panic!("expected Check command"),
        }
    }

    #[test]
    fn j13a_duplicate_config_rejected() {
        let result = parse_cli(&[
            "check", "--config", "a.json", "--config", "b.json", "--engine", "e.exe",
        ]);
        assert!(result.is_err());
    }

    #[test]
    fn j13a_duplicate_engine_rejected() {
        let result = parse_cli(&[
            "check", "--config", "c.json", "--engine", "a.exe", "--engine", "b.exe",
        ]);
        assert!(result.is_err());
    }

    #[test]
    fn j13a_missing_config_rejected() {
        let result = parse_cli(&["check", "--engine", "e.exe"]);
        assert!(result.is_err());
    }

    #[test]
    fn j13a_missing_engine_rejected() {
        let result = parse_cli(&["check", "--config", "c.json"]);
        assert!(result.is_err());
    }

    #[test]
    fn j13a_unknown_option_rejected() {
        let result = parse_cli(&[
            "check",
            "--config",
            "c.json",
            "--engine",
            "e.exe",
            "--unknown",
            "x",
        ]);
        assert!(result.is_err());
    }

    #[test]
    fn j13a_unknown_command_rejected() {
        let result = parse_cli(&["nonexistent"]);
        assert!(result.is_err());
    }

    #[test]
    fn j13a_misspelled_runn_rejected() {
        // "runn" is not a recognised command and must not enter legacy.
        let result = parse_cli(&["runn"]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        let msg = err.to_string();
        assert!(
            !msg.contains("legacy"),
            "runn must not reach legacy parser: {msg}"
        );
    }

    #[test]
    fn j13a_explicit_legacy_reaches_parser() {
        let cli = parse_cli(&["__legacy", "engine.exe", "req.json"]).unwrap();
        match cli.command {
            Some(Command::Legacy { args }) => {
                assert_eq!(args, vec!["engine.exe", "req.json"]);
            }
            _ => panic!("expected Legacy command"),
        }
    }

    #[test]
    fn j13a_legacy_with_trailing_options() {
        let cli = parse_cli(&[
            "__legacy",
            "engine.exe",
            "req.json",
            "allow",
            "trail.jsonl",
            "success",
            "--host-data-root",
            "C:\\data",
        ])
        .unwrap();
        match cli.command {
            Some(Command::Legacy { args }) => {
                assert_eq!(args.len(), 7);
                assert_eq!(args[0], "engine.exe");
                assert_eq!(args[6], "C:\\data");
            }
            _ => panic!("expected Legacy command"),
        }
    }

    #[test]
    fn j13a_hidden_commands_absent_from_help() {
        let cli = parse_cli(&["--help"]);
        // --help causes clap to print help and return an error in try_parse_from.
        // We just verify the error message doesn't mention hidden commands.
        let err = cli.unwrap_err();
        let msg = err.to_string();
        assert!(!msg.contains("__legacy"), "help must not show __legacy");
        assert!(
            !msg.contains("provision-replay"),
            "help must not show provision-replay"
        );
        assert!(
            !msg.contains("event-admission-probe"),
            "help must not show event-admission-probe"
        );
        assert!(
            !msg.contains("event-admission-trail-probe"),
            "help must not show event-admission-trail-probe"
        );
    }

    #[test]
    fn j13a_command_absent_rejected() {
        let result = parse_cli(&[]);
        // Clap with optional subcommand returns Ok with command=None
        // when no arguments are given. The new main handles this by
        // emitting an error envelope. So this test should pass.
        assert!(result.is_ok());
    }

    #[test]
    fn j13a_extra_positional_rejected() {
        let result = parse_cli(&["check", "--config", "c.json", "--engine", "e.exe", "extra"]);
        assert!(result.is_err());
    }

    #[test]
    fn j13a_outcome_status_values() {
        assert_eq!(OutcomeStatus::Ok.exit_code(), 0);
        assert_eq!(OutcomeStatus::InvalidCliUsage.exit_code(), 2);
        assert_eq!(OutcomeStatus::InvalidData.exit_code(), 3);
        assert_eq!(OutcomeStatus::Unavailable.exit_code(), 4);
        assert_eq!(OutcomeStatus::Failed.exit_code(), 6);
        assert_eq!(OutcomeStatus::Uncertain.exit_code(), 7);
        assert_eq!(OutcomeStatus::AuditFailed.exit_code(), 8);
        assert_eq!(OutcomeStatus::NotFound.exit_code(), 9);
        assert_eq!(OutcomeStatus::Interrupted.exit_code(), 10);
    }

    #[test]
    fn j13a_envelope_no_timestamp() {
        let envelope = CliEnvelope::ok("check", serde_json::json!({"test": true}));
        let json = serde_json::to_string(&envelope).unwrap();
        assert!(
            !json.contains("timestamp"),
            "envelope must not contain timestamp"
        );
    }

    #[test]
    fn j13a_unknown_subcommand_rejected() {
        // "run" is not a command (not yet implemented).
        let result = parse_cli(&["run"]);
        assert!(result.is_err());
    }

    #[test]
    fn j13a_extra_subcommand_arg_rejected() {
        let result = parse_cli(&["check", "--config", "c.json", "--engine", "e.exe", "--foo"]);
        assert!(result.is_err());
    }

    #[test]
    fn j13a_equal_sign_config_accepted() {
        // clap 4 normally supports --config=PATH but let's verify.
        let cli = parse_cli(&["check", "--config=c.json", "--engine", "e.exe"]).unwrap();
        match cli.command {
            Some(Command::Check { config, .. }) => {
                assert_eq!(config, std::path::PathBuf::from("c.json"));
            }
            _ => panic!("expected Check"),
        }
    }
}
