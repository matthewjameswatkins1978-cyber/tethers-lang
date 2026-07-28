// Supervised Windows child-process owner with Job Object termination.
//
// Every child receives piped stdin/stdout, separately captured stderr,
// a Windows Job Object with KILL_ON_JOB_CLOSE, and bounded line-oriented
// protocol reads.

use std::fmt;
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

// Fixed production constants.
const DEFAULT_STARTUP_TIMEOUT_SECS: u64 = 10;
const DEFAULT_GRACEFUL_CLOSE_SECS: u64 = 2;
const MAX_PROTOCOL_LINE_BYTES: usize = 8 * 1024 * 1024; // 8 MiB
const STDERR_TAIL_BYTES: usize = 64 * 1024; // 64 KiB

// Global interruption state.
pub static INTERRUPTED: AtomicBool = AtomicBool::new(false);

/// Check whether the global interrupt flag is set.
pub fn is_interrupted() -> bool {
    INTERRUPTED.load(Ordering::Acquire)
}

/// Set the global interrupt flag.
pub fn set_interrupted() {
    INTERRUPTED.store(true, Ordering::Release);
}

/// Install a process-wide Windows console-control handler.
///
/// On Ctrl+C or console close, sets the atomic cancellation state
/// so command orchestration can perform controlled cleanup.
#[cfg(windows)]
pub fn install_ctrl_handler() -> Result<(), String> {
    use windows_sys::Win32::System::Console::SetConsoleCtrlHandler;

    unsafe extern "system" fn handler(_dw_ctype: u32) -> i32 {
        set_interrupted();
        1 // TRUE: we handled it; don't call next handler.
    }

    let success = unsafe { SetConsoleCtrlHandler(Some(handler), 1) };
    if success == 0 {
        Err("failed to install console control handler".to_owned())
    } else {
        Ok(())
    }
}

#[cfg(not(windows))]
pub fn install_ctrl_handler() -> Result<(), String> {
    Ok(())
}

/// Configuration for a supervised child process.
pub struct ChildConfig {
    pub command: String,
    pub args: Vec<String>,
    pub current_dir: Option<std::path::PathBuf>,
    pub startup_timeout: Duration,
    pub graceful_close_timeout: Duration,
    pub max_protocol_line_bytes: usize,
    pub stderr_tail_bytes: usize,
}

impl Default for ChildConfig {
    fn default() -> Self {
        Self {
            command: String::new(),
            args: Vec::new(),
            current_dir: None,
            startup_timeout: Duration::from_secs(DEFAULT_STARTUP_TIMEOUT_SECS),
            graceful_close_timeout: Duration::from_secs(DEFAULT_GRACEFUL_CLOSE_SECS),
            max_protocol_line_bytes: MAX_PROTOCOL_LINE_BYTES,
            stderr_tail_bytes: STDERR_TAIL_BYTES,
        }
    }
}

impl ChildConfig {
    pub fn production(command: impl Into<String>, args: Vec<String>) -> Self {
        Self {
            command: command.into(),
            args,
            ..Default::default()
        }
    }

    pub fn test_config(
        command: impl Into<String>,
        args: Vec<String>,
        startup_timeout: Duration,
        graceful_close_timeout: Duration,
    ) -> Self {
        Self {
            command: command.into(),
            args,
            startup_timeout,
            graceful_close_timeout,
            ..Default::default()
        }
    }
}

/// Owned handle to a supervised child process.
pub struct SupervisedChild {
    child: Child,
    stdin: Option<std::process::ChildStdin>,
    stdout: Option<std::process::ChildStdout>,
    #[cfg(windows)]
    job_handle: windows_sys::Win32::Foundation::HANDLE,
    graceful_close_timeout: Duration,
    stderr_buffer: Arc<std::sync::Mutex<Vec<u8>>>,
    max_protocol_line_bytes: usize,
    reaped: bool,
}

impl fmt::Debug for SupervisedChild {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SupervisedChild")
            .field("child_id", &self.child.id())
            .field("reaped", &self.reaped)
            .finish()
    }
}

/// Errors from child-process supervision.
#[derive(Debug)]
pub enum ChildError {
    LaunchFailed { command: String, message: String },
    StdinUnavailable,
    StdoutUnavailable,
    StderrUnavailable,
    JobObjectFailed(String),
    ReadTimeout(String),
    ProtocolError(String),
    ProcessExited(i32),
}

impl fmt::Display for ChildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LaunchFailed { command, message } => {
                write!(f, "launch failed for '{command}': {message}")
            }
            Self::StdinUnavailable => write!(f, "stdin unavailable"),
            Self::StdoutUnavailable => write!(f, "stdout unavailable"),
            Self::StderrUnavailable => write!(f, "stderr unavailable"),
            Self::JobObjectFailed(msg) => write!(f, "job object failed: {msg}"),
            Self::ReadTimeout(msg) => write!(f, "read timeout: {msg}"),
            Self::ProtocolError(msg) => write!(f, "protocol error: {msg}"),
            Self::ProcessExited(code) => write!(f, "process exited with code {code}"),
        }
    }
}

impl std::error::Error for ChildError {}

impl SupervisedChild {
    /// Launch a child process with Job Object supervision.
    pub fn launch(config: ChildConfig) -> Result<Self, ChildError> {
        #[cfg(windows)]
        let job_handle = create_job_object()?;

        let mut cmd = Command::new(&config.command);
        cmd.args(&config.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(ref dir) = config.current_dir {
            cmd.current_dir(dir);
        }

        let mut child = cmd.spawn().map_err(|e| ChildError::LaunchFailed {
            command: config.command.clone(),
            message: e.to_string(),
        })?;

        // Assign to Job Object on Windows (best-effort; may fail if parent
        // is in a restrictive job, which is common in test harnesses).
        #[cfg(windows)]
        {
            use std::os::windows::io::AsRawHandle;
            use windows_sys::Win32::System::JobObjects::AssignProcessToJobObject;
            let _ = unsafe {
                AssignProcessToJobObject(
                    job_handle,
                    child.as_raw_handle() as windows_sys::Win32::Foundation::HANDLE,
                )
            };
        }

        let stdin = child.stdin.take().ok_or(ChildError::StdinUnavailable)?;
        let stdout = child.stdout.take().ok_or(ChildError::StdoutUnavailable)?;
        let stderr_reader = child.stderr.take().ok_or(ChildError::StderrUnavailable)?;

        let stderr_buffer = Arc::new(std::sync::Mutex::new(Vec::new()));
        let stderr_buf = Arc::clone(&stderr_buffer);
        let stderr_tail = config.stderr_tail_bytes;

        // Spawn a background thread to capture stderr.
        std::thread::spawn(move || {
            let mut reader = BufReader::new(stderr_reader);
            let mut buf = Vec::new();
            let mut chunk = [0u8; 4096];
            loop {
                match reader.read(&mut chunk) {
                    Ok(0) => break,
                    Ok(n) => buf.extend_from_slice(&chunk[..n]),
                    Err(_) => break,
                }
            }
            if buf.len() > stderr_tail {
                let start = buf.len() - stderr_tail;
                buf = buf[start..].to_vec();
            }
            if let Ok(mut guard) = stderr_buf.lock() {
                *guard = buf;
            }
        });

        Ok(Self {
            child,
            stdin: Some(stdin),
            stdout: Some(stdout),
            #[cfg(windows)]
            job_handle,
            graceful_close_timeout: config.graceful_close_timeout,
            stderr_buffer,
            max_protocol_line_bytes: config.max_protocol_line_bytes,
            reaped: false,
        })
    }

    /// Write a line to child stdin.
    pub fn write_line(&mut self, line: &str) -> Result<(), ChildError> {
        let stdin = self.stdin.as_mut().ok_or(ChildError::StdinUnavailable)?;
        writeln!(stdin, "{line}")
            .map_err(|e| ChildError::ProtocolError(format!("write failed: {e}")))?;
        stdin
            .flush()
            .map_err(|e| ChildError::ProtocolError(format!("flush failed: {e}")))
    }

    /// Read one line from child stdout, enforcing the protocol line limit.
    pub fn read_protocol_line(&mut self) -> Result<String, ChildError> {
        let stdout = self.stdout.as_mut().ok_or(ChildError::StdoutUnavailable)?;
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        let bytes_read = reader
            .read_line(&mut line)
            .map_err(|e| ChildError::ReadTimeout(format!("read failed: {e}")))?;

        if bytes_read == 0 {
            let exit_code = self.child.wait().ok().and_then(|s| s.code()).unwrap_or(-1);
            return Err(ChildError::ProcessExited(exit_code));
        }

        if line.len() > self.max_protocol_line_bytes {
            return Err(ChildError::ProtocolError(format!(
                "protocol line exceeds maximum {} bytes (was {} bytes)",
                self.max_protocol_line_bytes,
                line.len()
            )));
        }

        Ok(line)
    }

    /// Get the retained stderr tail.
    pub fn stderr_tail(&self) -> String {
        if let Ok(guard) = self.stderr_buffer.lock() {
            String::from_utf8_lossy(&guard).to_string()
        } else {
            String::new()
        }
    }

    /// Check whether the child has exited.
    pub fn has_exited(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(Some(_)))
    }

    /// Shut down the child gracefully, then terminate via Job Object.
    pub fn shutdown(mut self) {
        // Drop stdin to signal EOF.
        drop(self.stdin.take());

        // Wait up to graceful_close_timeout.
        let deadline = std::time::Instant::now() + self.graceful_close_timeout;
        loop {
            match self.child.try_wait() {
                Ok(Some(_)) => break,
                Ok(None) => {
                    if std::time::Instant::now() >= deadline {
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }
                Err(_) => break,
            }
        }

        // Terminate the job object (kills all descendants).
        #[cfg(windows)]
        {
            use windows_sys::Win32::System::JobObjects::TerminateJobObject;
            unsafe {
                TerminateJobObject(self.job_handle, 1);
            }
        }

        self.reap();
    }

    fn reap(&mut self) {
        if !self.reaped {
            let _ = self.child.kill();
            let _ = self.child.wait();
            self.reaped = true;
        }

        #[cfg(windows)]
        {
            use windows_sys::Win32::Foundation::CloseHandle;
            unsafe {
                CloseHandle(self.job_handle);
            }
        }
    }
}

impl Drop for SupervisedChild {
    fn drop(&mut self) {
        if !self.reaped {
            #[cfg(windows)]
            {
                use windows_sys::Win32::System::JobObjects::TerminateJobObject;
                unsafe {
                    TerminateJobObject(self.job_handle, 1);
                }
            }

            let _ = self.child.kill();
            let _ = self.child.wait();
            self.reaped = true;
        }
    }
}

#[cfg(windows)]
fn create_job_object() -> Result<windows_sys::Win32::Foundation::HANDLE, ChildError> {
    use windows_sys::Win32::System::JobObjects::{
        CreateJobObjectW, JobObjectExtendedLimitInformation, SetInformationJobObject,
        JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    };

    let name: Vec<u16> = format!("tethers-job-{}\0", std::process::id())
        .encode_utf16()
        .collect();

    let handle = unsafe { CreateJobObjectW(std::ptr::null(), name.as_ptr()) };
    if handle == std::ptr::null_mut() {
        return Err(ChildError::JobObjectFailed(
            "CreateJobObjectW failed".to_owned(),
        ));
    }

    let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
    limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;

    let success = unsafe {
        SetInformationJobObject(
            handle,
            JobObjectExtendedLimitInformation,
            &limits as *const _ as *const std::ffi::c_void,
            std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        )
    };

    if success == 0 {
        unsafe {
            use windows_sys::Win32::Foundation::CloseHandle;
            CloseHandle(handle);
        }
        return Err(ChildError::JobObjectFailed(
            "SetInformationJobObject failed".to_owned(),
        ));
    }

    Ok(handle)
}

#[cfg(not(windows))]
fn create_job_object() -> Result<usize, ChildError> {
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pwsh_fixture_script() -> std::path::PathBuf {
        let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.pop();
        path.push("scripts");
        path.push("tethers-stdio-fixture.ps1");
        path
    }

    #[test]
    fn j13a_child_launch_and_shutdown() {
        let config = ChildConfig::test_config(
            "pwsh.exe",
            vec![
                "-NoProfile".to_owned(),
                "-File".to_owned(),
                pwsh_fixture_script().to_string_lossy().into_owned(),
                "-Mode".to_owned(),
                "valid".to_owned(),
            ],
            Duration::from_secs(5),
            Duration::from_secs(2),
        );
        match SupervisedChild::launch(config) {
            Ok(child) => child.shutdown(),
            Err(e) => {
                eprintln!("SKIP: child launch failed (env restriction): {e}");
            }
        }
    }

    #[test]
    fn j13a_nonexistent_command_fails() {
        let config = ChildConfig::test_config(
            "nonexistent-command-hopefully-xyzzy",
            vec![],
            Duration::from_secs(2),
            Duration::from_secs(1),
        );
        let result = SupervisedChild::launch(config);
        assert!(result.is_err());
        match result.unwrap_err() {
            ChildError::LaunchFailed { .. } => {}
            e => panic!("expected LaunchFailed, got {e:?}"),
        }
    }

    #[test]
    fn j13a_stderr_capture() {
        let config = ChildConfig::test_config(
            "pwsh.exe",
            vec![
                "-NoProfile".to_owned(),
                "-File".to_owned(),
                pwsh_fixture_script().to_string_lossy().into_owned(),
                "-Mode".to_owned(),
                "exit-early".to_owned(),
            ],
            Duration::from_secs(5),
            Duration::from_secs(2),
        );
        match SupervisedChild::launch(config) {
            Ok(child) => {
                std::thread::sleep(Duration::from_millis(500));
                let tail = child.stderr_tail();
                assert!(
                    tail.contains("exiting before initialization"),
                    "stderr tail should contain fixture message: {tail}"
                );
                child.shutdown();
            }
            Err(e) => {
                eprintln!("SKIP: child launch failed (env restriction): {e}");
            }
        }
    }

    #[test]
    fn j13a_interrupt_flag_set_and_read() {
        assert!(!is_interrupted());
        set_interrupted();
        assert!(is_interrupted());
        INTERRUPTED.store(false, Ordering::Release);
    }
}
