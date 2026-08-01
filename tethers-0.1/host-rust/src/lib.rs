extern crate self as tethers_reference_host;

pub mod application;
pub mod approval;
pub mod candidate;
pub mod check_command;
pub mod child_process;
pub mod cli;
pub mod configured_runtime;
pub mod dispatch;
pub mod engine_stdio;
pub mod event_admission;
mod event_queue;
pub mod executor;
pub mod host_execution;
mod manifest;
mod outcome;
pub mod package;
pub mod policy;
pub mod provider;
pub mod replay;
mod replay_runtime;
#[cfg(windows)]
pub mod replay_windows;
pub mod resolver;
mod result_anchor;
pub mod run_command;
pub mod run_input;
pub mod runtime_config;
pub mod socket;
pub mod stdio_provider;
pub mod trail_command;
pub mod trusted_store;
mod validation;
pub(crate) use application::{
    build_event_admission_entry, execute_shared_boundary, extract_proposed_action,
    inject_bridge_projection_into_request, now_unix_ms, request_exact_approval, InputEventContext,
    ResponseResultAnchorWriter, SharedExecutionOutcome, SharedExecutionResult,
};
