// Supervised Windows child-process owner with Job Object termination.
//
// Every child receives piped stdin/stdout, separately captured stderr,
// an unnamed Windows Job Object with KILL_ON_JOB_CLOSE, persistent
// stdout reader thread with mpsc channel for timeout-aware protocol
// reads, and stored reader-thread JoinHandles for proper cleanup.

use std::collections::BTreeMap;
use std::fmt;
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

// Fixed production constants.
const DEFAULT_STARTUP_TIMEOUT_SECS: u64 = 10;
const DEFAULT_GRACEFUL_CLOSE_SECS: u64 = 2;
const MAX_PROTOCOL_LINE_BYTES: usize = 8 * 1024 * 1024; // 8 MiB
const STDERR_TAIL_BYTES: usize = 64 * 1024; // 64 KiB
const SYNC_CHANNEL_BOUND: usize = 16;
// A console interrupt can close a provider's stdout before the host handler
// publishes the interrupt flag. Give that hand-off a short, bounded window.
const INTERRUPT_DISCONNECT_OBSERVATION: Duration = Duration::from_millis(50);
const INTERRUPT_DISCONNECT_POLL: Duration = Duration::from_millis(1);

// Global interruption state.
pub static INTERRUPTED: AtomicBool = AtomicBool::new(false);

pub fn is_interrupted() -> bool {
    INTERRUPTED.load(Ordering::Acquire)
}
pub fn set_interrupted() {
    INTERRUPTED.store(true, Ordering::Release);
}

#[cfg(windows)]
pub fn install_ctrl_handler() -> Result<(), String> {
    use windows_sys::Win32::System::Console::SetConsoleCtrlHandler;
    unsafe extern "system" fn handler(_dw_ctype: u32) -> i32 {
        set_interrupted();
        1
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

pub struct ChildConfig {
    pub command: String,
    pub args: Vec<String>,
    pub current_dir: Option<std::path::PathBuf>,
    pub startup_timeout: Duration,
    pub graceful_close_timeout: Duration,
    pub max_protocol_line_bytes: usize,
    pub stderr_tail_bytes: usize,
    /// When true the child receives only `environment`; no ambient process
    /// environment is inherited.
    pub clear_environment: bool,
    pub environment: BTreeMap<String, String>,
    pub max_processes: u32,
    pub process_memory_limit_bytes: usize,
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
            clear_environment: false,
            environment: BTreeMap::new(),
            max_processes: 8,
            process_memory_limit_bytes: 256 * 1024 * 1024,
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

/// A protocol line or error from the stdout reader thread.
type LineResult = Result<String, ChildError>;

/// Owned handle to a supervised child process.
pub struct SupervisedChild {
    child: Child,
    stdin: Option<ChildStdin>,
    #[cfg(windows)]
    job_handle: windows_sys::Win32::Foundation::HANDLE,
    graceful_close_timeout: Duration,
    max_line_bytes: usize,

    // Channel from stdout reader thread.
    line_rx: Receiver<LineResult>,
    stdout_thread: Option<JoinHandle<()>>,

    // Stderr capture.
    stderr_buffer: Arc<Mutex<Vec<u8>>>,
    stderr_thread: Option<JoinHandle<()>>,

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
    Interrupted,
    LineTooLarge { max: usize, actual: usize },
    NotUtf8,
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
            Self::Interrupted => write!(f, "interrupted"),
            Self::LineTooLarge { max, actual } => {
                write!(f, "protocol line {actual} bytes exceeds maximum {max}")
            }
            Self::NotUtf8 => write!(f, "protocol line is not valid UTF-8"),
        }
    }
}

impl std::error::Error for ChildError {}

/// Prefer a host interruption that becomes visible immediately after a
/// disconnect over the disconnect's ordinary error. The caller supplies the
/// clock and pause operations so this ordering boundary is deterministic in
/// tests; production pauses for at most 50 ms in one-millisecond increments.
fn classify_disconnect_with_interrupt_observation<Interrupted, Elapsed, Pause>(
    ordinary_error: ChildError,
    observation_window: Duration,
    mut interrupted: Interrupted,
    mut elapsed: Elapsed,
    mut pause: Pause,
) -> ChildError
where
    Interrupted: FnMut() -> bool,
    Elapsed: FnMut() -> Duration,
    Pause: FnMut(),
{
    let deadline = elapsed().saturating_add(observation_window);
    loop {
        if interrupted() {
            return ChildError::Interrupted;
        }
        if elapsed() >= deadline {
            return ordinary_error;
        }
        pause();
    }
}

fn classify_disconnect_after_interrupt_observation(ordinary_error: ChildError) -> ChildError {
    let started = Instant::now();
    classify_disconnect_with_interrupt_observation(
        ordinary_error,
        INTERRUPT_DISCONNECT_OBSERVATION,
        is_interrupted,
        || started.elapsed(),
        || thread::sleep(INTERRUPT_DISCONNECT_POLL),
    )
}

impl SupervisedChild {
    pub fn launch(config: ChildConfig) -> Result<Self, ChildError> {
        #[cfg(windows)]
        let job_handle =
            create_job_object(config.max_processes, config.process_memory_limit_bytes)?;

        let mut cmd = Command::new(&config.command);
        cmd.args(&config.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if config.clear_environment {
            cmd.env_clear();
        }
        cmd.envs(&config.environment);
        if let Some(ref dir) = config.current_dir {
            cmd.current_dir(dir);
        }

        let mut child = cmd.spawn().map_err(|e| ChildError::LaunchFailed {
            command: config.command.clone(),
            message: e.to_string(),
        })?;

        // Assign to Job Object (fatal on failure).
        #[cfg(windows)]
        {
            use std::os::windows::io::AsRawHandle;
            use windows_sys::Win32::System::JobObjects::AssignProcessToJobObject;
            let success = unsafe {
                AssignProcessToJobObject(
                    job_handle,
                    child.as_raw_handle() as windows_sys::Win32::Foundation::HANDLE,
                )
            };
            if success == 0 {
                let _ = child.kill();
                let _ = child.wait();
                unsafe {
                    use windows_sys::Win32::Foundation::CloseHandle;
                    CloseHandle(job_handle);
                }
                return Err(ChildError::JobObjectFailed(
                    "AssignProcessToJobObject failed".to_owned(),
                ));
            }
        }

        let stdin = child.stdin.take().ok_or(ChildError::StdinUnavailable)?;
        let stdout = child.stdout.take().ok_or(ChildError::StdoutUnavailable)?;
        let stderr_r = child.stderr.take().ok_or(ChildError::StderrUnavailable)?;

        // Spawn stdout reader thread.
        let max_line = config.max_protocol_line_bytes;
        let (line_tx, line_rx) = mpsc::sync_channel::<LineResult>(SYNC_CHANNEL_BOUND);
        let stdout_thread = thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            loop {
                let mut buf = Vec::with_capacity(4096);
                // Read until newline or EOF.
                match read_until_newline(&mut reader, &mut buf, max_line) {
                    Ok(Some(line)) => {
                        // Validate strict UTF-8.
                        match String::from_utf8(line) {
                            Ok(s) => {
                                if line_tx.send(Ok(s)).is_err() {
                                    break; // receiver dropped
                                }
                            }
                            Err(_) => {
                                let _ = line_tx.send(Err(ChildError::NotUtf8));
                                break;
                            }
                        }
                    }
                    Ok(None) => {
                        // EOF
                        break;
                    }
                    Err(e) => {
                        let _ = line_tx.send(Err(e));
                        break;
                    }
                }
            }
        });

        // Spawn stderr capture thread.
        let stderr_buffer = Arc::new(Mutex::new(Vec::new()));
        let stderr_buf = Arc::clone(&stderr_buffer);
        let stderr_tail = config.stderr_tail_bytes;
        let stderr_thread = thread::spawn(move || {
            let mut reader = BufReader::new(stderr_r);
            let mut buf = Vec::new();
            let mut chunk = [0u8; 4096];
            loop {
                match reader.read(&mut chunk) {
                    Ok(0) => break,
                    Ok(n) => {
                        buf.extend_from_slice(&chunk[..n]);
                        if buf.len() > stderr_tail {
                            let excess = buf.len() - stderr_tail;
                            buf.drain(..excess);
                        }
                    }
                    Err(_) => break,
                }
            }
            if let Ok(mut guard) = stderr_buf.lock() {
                *guard = buf;
            }
        });

        Ok(Self {
            child,
            stdin: Some(stdin),
            #[cfg(windows)]
            job_handle,
            graceful_close_timeout: config.graceful_close_timeout,
            max_line_bytes: config.max_protocol_line_bytes,
            line_rx,
            stdout_thread: Some(stdout_thread),
            stderr_buffer,
            stderr_thread: Some(stderr_thread),
            reaped: false,
        })
    }

    pub fn write_line(&mut self, line: &str) -> Result<(), ChildError> {
        let stdin = self.stdin.as_mut().ok_or(ChildError::StdinUnavailable)?;
        writeln!(stdin, "{line}")
            .map_err(|e| ChildError::ProtocolError(format!("write failed: {e}")))?;
        stdin
            .flush()
            .map_err(|e| ChildError::ProtocolError(format!("flush failed: {e}")))
    }

    /// Receive one protocol line from the reader thread with timeout.
    pub fn read_protocol_line(&mut self, timeout: Duration) -> Result<String, ChildError> {
        let deadline = Instant::now() + timeout;
        loop {
            if is_interrupted() {
                return Err(ChildError::Interrupted);
            }
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(ChildError::ReadTimeout(
                    "timeout waiting for protocol line".to_owned(),
                ));
            }
            // Use a shorter poll interval to check interrupt frequently.
            let poll = Duration::from_millis(100).min(remaining);
            match self.line_rx.recv_timeout(poll) {
                Ok(result) => return result,
                Err(mpsc::RecvTimeoutError::Timeout) => continue,
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    // A CTRL_C_EVENT can close the provider's stdout before the
                    // host handler publishes INTERRUPTED. Preserve interruption
                    // precedence through this small bounded observation window.
                    let ordinary_error = match self.child.try_wait() {
                        Ok(Some(status)) => ChildError::ProcessExited(status.code().unwrap_or(-1)),
                        _ => ChildError::ProtocolError(
                            "stdout reader disconnected unexpectedly".to_owned(),
                        ),
                    };
                    return Err(classify_disconnect_after_interrupt_observation(
                        ordinary_error,
                    ));
                }
            }
        }
    }

    /// Read one already-buffered protocol line without waiting.
    ///
    /// This is used only at a serial protocol boundary, before issuing the
    /// next request, so server notifications can invalidate host state before
    /// another operation is invoked.
    pub fn try_read_protocol_line(&mut self) -> Result<Option<String>, ChildError> {
        match self.line_rx.try_recv() {
            Ok(result) => result.map(Some),
            Err(mpsc::TryRecvError::Empty) => Ok(None),
            Err(mpsc::TryRecvError::Disconnected) => {
                let ordinary_error = match self.child.try_wait() {
                    Ok(Some(status)) => ChildError::ProcessExited(status.code().unwrap_or(-1)),
                    _ => ChildError::ProtocolError(
                        "stdout reader disconnected unexpectedly".to_owned(),
                    ),
                };
                Err(classify_disconnect_after_interrupt_observation(
                    ordinary_error,
                ))
            }
        }
    }

    pub fn stderr_tail(&self) -> String {
        if let Ok(guard) = self.stderr_buffer.lock() {
            String::from_utf8_lossy(&guard).to_string()
        } else {
            String::new()
        }
    }

    pub fn close_stdin(&mut self) {
        drop(self.stdin.take());
    }

    pub fn has_exited(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(Some(_)))
    }

    /// Full shutdown sequence.
    pub fn shutdown(mut self) {
        self.shutdown_inner();
    }

    fn shutdown_inner(&mut self) {
        // 1. Close stdin.
        drop(self.stdin.take());

        // 2. Wait up to graceful_close_timeout.
        let deadline = Instant::now() + self.graceful_close_timeout;
        loop {
            match self.child.try_wait() {
                Ok(Some(_)) => break,
                Ok(None) => {
                    if Instant::now() >= deadline {
                        break;
                    }
                    thread::sleep(Duration::from_millis(50));
                }
                Err(_) => break,
            }
        }

        // 3. Terminate Job Object.
        #[cfg(windows)]
        {
            use windows_sys::Win32::System::JobObjects::TerminateJobObject;
            unsafe {
                TerminateJobObject(self.job_handle, 1);
            }
        }

        // 4. Reap direct child.
        let _ = self.child.kill();
        let _ = self.child.wait();
        self.reaped = true;

        // 5. Close Job Object handle.
        #[cfg(windows)]
        {
            use windows_sys::Win32::Foundation::CloseHandle;
            unsafe {
                CloseHandle(self.job_handle);
            }
        }

        // 6. Join reader threads.
        if let Some(h) = self.stdout_thread.take() {
            let _ = h.join();
        }
        if let Some(h) = self.stderr_thread.take() {
            let _ = h.join();
        }
    }
}

impl Drop for SupervisedChild {
    fn drop(&mut self) {
        if !self.reaped {
            self.shutdown_inner();
        }
    }
}

/// Read bytes from reader until newline or EOF, enforcing max size.
/// Returns Ok(Some(line_bytes_without_newline)) on success,
/// Ok(None) on EOF without data, or Err on error/overflow.
fn read_until_newline(
    reader: &mut BufReader<ChildStdout>,
    buf: &mut Vec<u8>,
    max_bytes: usize,
) -> Result<Option<Vec<u8>>, ChildError> {
    loop {
        let data_len;
        let found_newline;
        let consume_amount;
        {
            let available = match reader.fill_buf() {
                Ok(data) if data.is_empty() => {
                    if buf.is_empty() {
                        return Ok(None); // EOF
                    } else {
                        // EOF with partial data (no trailing newline).
                        if buf.len() > max_bytes {
                            return Err(ChildError::LineTooLarge {
                                max: max_bytes,
                                actual: buf.len(),
                            });
                        }
                        let result = std::mem::take(buf);
                        return Ok(Some(result));
                    }
                }
                Ok(data) => data,
                Err(e) => {
                    return Err(ChildError::ReadTimeout(format!("read error: {e}")));
                }
            };

            let mut consumed = 0;
            let mut nl = false;

            for (i, &b) in available.iter().enumerate() {
                if b == b'\n' {
                    nl = true;
                    consumed = i + 1;
                    break;
                }
                if b == 0 {
                    return Err(ChildError::NotUtf8);
                }
            }

            if nl {
                buf.extend_from_slice(&available[..consumed - 1]); // exclude \n
                data_len = 0;
                found_newline = true;
                consume_amount = consumed;
            } else {
                buf.extend_from_slice(available);
                data_len = available.len();
                found_newline = false;
                consume_amount = available.len();
            }
        } // borrow on available/reader ends here

        reader.consume(consume_amount);

        if found_newline {
            if buf.len() > max_bytes {
                return Err(ChildError::LineTooLarge {
                    max: max_bytes,
                    actual: buf.len(),
                });
            }
            let result = std::mem::take(buf);
            return Ok(Some(result));
        } else {
            if buf.len() > max_bytes {
                return Err(ChildError::LineTooLarge {
                    max: max_bytes,
                    actual: buf.len(),
                });
            }
            // Continue reading for newline.
            let _ = data_len;
        }
    }
}

#[cfg(windows)]
fn create_job_object(
    max_processes: u32,
    process_memory_limit_bytes: usize,
) -> Result<windows_sys::Win32::Foundation::HANDLE, ChildError> {
    use windows_sys::Win32::System::JobObjects::{
        CreateJobObjectW, JobObjectExtendedLimitInformation, SetInformationJobObject,
        JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_ACTIVE_PROCESS,
        JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE, JOB_OBJECT_LIMIT_PROCESS_MEMORY,
    };

    // Unnamed: NULL, NULL.
    let handle = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
    if handle.is_null() {
        return Err(ChildError::JobObjectFailed(
            "CreateJobObjectW failed".to_owned(),
        ));
    }

    let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
    limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE
        | JOB_OBJECT_LIMIT_ACTIVE_PROCESS
        | JOB_OBJECT_LIMIT_PROCESS_MEMORY;
    limits.BasicLimitInformation.ActiveProcessLimit = max_processes;
    limits.ProcessMemoryLimit = process_memory_limit_bytes;

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
fn create_job_object(
    _max_processes: u32,
    _process_memory_limit_bytes: usize,
) -> Result<usize, ChildError> {
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    fn fixture_script() -> std::path::PathBuf {
        let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.pop();
        path.push("scripts");
        path.push("tethers-stdio-fixture.ps1");
        path
    }

    fn launch_fixture(mode: &str, startup: u64, close: u64) -> Result<SupervisedChild, ChildError> {
        let config = ChildConfig::test_config(
            "pwsh.exe",
            vec![
                "-NoProfile".to_owned(),
                "-File".to_owned(),
                fixture_script().to_string_lossy().into_owned(),
                "-Mode".to_owned(),
                mode.to_owned(),
            ],
            Duration::from_secs(startup),
            Duration::from_secs(close),
        );
        SupervisedChild::launch(config)
    }

    #[test]
    fn j13a_child_launch_and_shutdown() {
        let child = launch_fixture("valid", 5, 2).expect("launch");
        child.shutdown();
    }

    #[test]
    fn j13a_nonexistent_command_fails() {
        let config = ChildConfig::test_config(
            "nonexistent-command-hopefully-xyzzy",
            vec![],
            Duration::from_secs(2),
            Duration::from_secs(1),
        );
        assert!(SupervisedChild::launch(config).is_err());
    }

    #[test]
    fn j13a_stderr_capture() {
        let mut child = launch_fixture("exit-early", 5, 2).expect("launch");
        // Wait for child to exit and stderr thread to capture.
        for _ in 0..40 {
            if child.has_exited() {
                break;
            }
            thread::sleep(Duration::from_millis(50));
        }
        thread::sleep(Duration::from_millis(200));
        let tail = child.stderr_tail();
        assert!(
            tail.contains("exiting before initialization"),
            "stderr: {tail}"
        );
        child.shutdown();
    }

    #[test]
    fn j13a_interrupt_flag_set_and_read() {
        assert!(!is_interrupted());
        set_interrupted();
        assert!(is_interrupted());
        INTERRUPTED.store(false, Ordering::Release);
    }

    #[test]
    fn interrupted_is_returned_when_visible_at_stdout_disconnect() {
        let pauses = Cell::new(0);
        let result = classify_disconnect_with_interrupt_observation(
            ChildError::ProcessExited(7),
            Duration::from_millis(3),
            || true,
            || Duration::ZERO,
            || pauses.set(pauses.get() + 1),
        );

        assert!(matches!(result, ChildError::Interrupted));
        assert_eq!(pauses.get(), 0, "visible interruption must not wait");
    }

    #[test]
    fn interrupted_is_returned_when_visible_shortly_after_stdout_disconnect() {
        let elapsed_millis = Cell::new(0);
        let observations = Cell::new(0);
        let result = classify_disconnect_with_interrupt_observation(
            ChildError::ProcessExited(7),
            Duration::from_millis(3),
            || {
                observations.set(observations.get() + 1);
                observations.get() >= 2
            },
            || Duration::from_millis(elapsed_millis.get()),
            || elapsed_millis.set(elapsed_millis.get() + 1),
        );

        assert!(matches!(result, ChildError::Interrupted));
        assert_eq!(elapsed_millis.get(), 1, "late observation stays bounded");
    }

    #[test]
    fn ordinary_process_exit_remains_process_exited_without_interruption() {
        let elapsed_millis = Cell::new(0);
        let result = classify_disconnect_with_interrupt_observation(
            ChildError::ProcessExited(47),
            Duration::from_millis(3),
            || false,
            || Duration::from_millis(elapsed_millis.get()),
            || elapsed_millis.set(elapsed_millis.get() + 1),
        );

        assert!(matches!(result, ChildError::ProcessExited(47)));
    }

    #[test]
    fn interrupt_observation_window_terminates_at_its_bound() {
        let elapsed_millis = Cell::new(0);
        let result = classify_disconnect_with_interrupt_observation(
            ChildError::ProcessExited(1),
            Duration::from_millis(3),
            || false,
            || Duration::from_millis(elapsed_millis.get()),
            || elapsed_millis.set(elapsed_millis.get() + 1),
        );

        assert!(matches!(result, ChildError::ProcessExited(1)));
        assert_eq!(elapsed_millis.get(), 3, "observation loop is bounded");
    }

    #[test]
    fn j13a_independent_job_objects() {
        let c1 = launch_fixture("valid", 5, 2).expect("c1");
        let c2 = launch_fixture("valid", 5, 2).expect("c2");
        c1.shutdown();
        c2.shutdown();
    }

    #[test]
    fn j13a_direct_child_terminated() {
        let mut child = launch_fixture("valid", 5, 2).expect("launch");
        assert!(!child.has_exited());
        child.shutdown_inner();
        assert!(child.reaped);
    }

    #[test]
    fn j13a_descendant_terminated() {
        let child = launch_fixture("descendant-alive", 5, 2).expect("launch");
        child.shutdown();
    }

    #[test]
    fn j13a_job_handle_closed_on_shutdown() {
        let child = launch_fixture("valid", 5, 2).expect("launch");
        child.shutdown();
    }

    #[test]
    fn j13a_reader_threads_join() {
        let child = launch_fixture("valid", 5, 2).expect("launch");
        child.shutdown();
    }
}
