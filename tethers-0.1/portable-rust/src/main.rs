use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use tethers_portable::{evaluate_text, response_json, version, Response};

fn usage() -> &'static str {
    "usage: tethers evaluate [--input PATH] [--policy PATH]\n       tethers --version\n\nReads one JSON request from stdin unless --input is supplied."
}

fn main() {
    let exit_code = run(env::args().skip(1).collect());
    std::process::exit(exit_code);
}

fn run(args: Vec<String>) -> i32 {
    if args == ["--version"] || args == ["-V"] {
        println!("tethers portable {}", version());
        return 0;
    }
    if args == ["--help"] || args == ["-h"] {
        println!("{}", usage());
        return 0;
    }

    let mut input_path: Option<PathBuf> = None;
    let mut policy_path: Option<PathBuf> = None;
    let mut index = 0;
    if args.get(index).map(String::as_str) != Some("evaluate") {
        emit(Response::deny_error_for_cli(
            "expected the evaluate command",
        ));
        return 2;
    }
    index += 1;
    while index < args.len() {
        let flag = &args[index];
        let destination = match flag.as_str() {
            "--input" => &mut input_path,
            "--policy" => &mut policy_path,
            _ => {
                emit(Response::deny_error_for_cli(format!(
                    "unknown option: {flag}"
                )));
                return 2;
            }
        };
        index += 1;
        let Some(path) = args.get(index) else {
            emit(Response::deny_error_for_cli(format!(
                "missing value for {flag}"
            )));
            return 2;
        };
        if destination.is_some() {
            emit(Response::deny_error_for_cli(format!(
                "duplicate option: {flag}"
            )));
            return 2;
        }
        *destination = Some(PathBuf::from(path));
        index += 1;
    }

    let input = match input_path {
        Some(path) => match fs::read_to_string(path) {
            Ok(input) => input,
            Err(error) => {
                emit(Response::deny_error_for_cli(format!(
                    "cannot read input: {error}"
                )));
                return 3;
            }
        },
        None => {
            let mut input = String::new();
            if let Err(error) = io::stdin().read_to_string(&mut input) {
                emit(Response::deny_error_for_cli(format!(
                    "cannot read stdin: {error}"
                )));
                return 3;
            }
            input
        }
    };
    let policy = match policy_path {
        Some(path) => match fs::read_to_string(path) {
            Ok(policy) => Some(policy),
            Err(error) => {
                emit(Response::deny_error_for_cli(format!(
                    "cannot read policy: {error}"
                )));
                return 3;
            }
        },
        None => None,
    };

    emit(evaluate_text(&input, policy.as_deref()));
    0
}

fn emit(response: Response) {
    println!("{}", response_json(&response));
}

trait CliResponse {
    fn deny_error_for_cli(message: impl Into<String>) -> Self;
}

impl CliResponse for Response {
    fn deny_error_for_cli(message: impl Into<String>) -> Self {
        Response {
            decision: "DENY",
            rule: None,
            reason: None,
            error: Some(message.into()),
        }
    }
}
