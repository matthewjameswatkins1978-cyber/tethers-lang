// Supervised Windows child-process owner with Job Object termination.
//
// Every child receives piped stdin/stdout, separately captured stderr,
// an unnamed Windows Job Object with KILL_ON_JOB_CLOSE, persistent
// stdout reader thread with mpsc channel for timeout-aware protocol
// reads, and stored reader-thread JoinHandles for proper cleanup.

use std::collections::BTreeMap;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
#[cfg(not(windows))]
use std::process::{Child, Command, Stdio};
#[cfg(windows)]
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver};
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
    /// M3 direct-provider launches must join their Job Object while suspended,
    /// before provider code can run. Legacy host children retain their frozen
    /// creation path.
    pub assign_before_execution: bool,
}

#[cfg(windows)]
struct ManagedChild {
    process_handle: windows_sys::Win32::Foundation::HANDLE,
    process_id: u32,
}

#[cfg(windows)]
impl ManagedChild {
    fn id(&self) -> u32 {
        self.process_id
    }

    fn try_wait(&mut self) -> std::result::Result<Option<i32>, ()> {
        use windows_sys::Win32::Foundation::STILL_ACTIVE;
        use windows_sys::Win32::System::Threading::GetExitCodeProcess;
        let mut code = 0u32;
        // SAFETY: process_handle is owned by this object until wait/Drop.
        if unsafe { GetExitCodeProcess(self.process_handle, &mut code) } == 0 {
            return Err(());
        }
        Ok((code != STILL_ACTIVE as u32).then_some(code as i32))
    }

    fn kill(&mut self) -> std::result::Result<(), ()> {
        use windows_sys::Win32::System::Threading::TerminateProcess;
        // SAFETY: process_handle is an owned valid process handle.
        (unsafe { TerminateProcess(self.process_handle, 1) } != 0)
            .then_some(())
            .ok_or(())
    }

    fn wait(&mut self) -> std::result::Result<i32, ()> {
        use windows_sys::Win32::System::Threading::{
            GetExitCodeProcess, WaitForSingleObject, INFINITE,
        };
        // SAFETY: process_handle is owned and WaitForSingleObject accepts it.
        unsafe { WaitForSingleObject(self.process_handle, INFINITE) };
        let mut code = 0u32;
        // SAFETY: process_handle remains valid until SupervisedChild cleanup.
        if unsafe { GetExitCodeProcess(self.process_handle, &mut code) } == 0 {
            return Err(());
        }
        Ok(code as i32)
    }
}

#[cfg(windows)]
impl Drop for ManagedChild {
    fn drop(&mut self) {
        close_handle(self.process_handle);
    }
}

#[cfg(not(windows))]
struct ManagedChild(Child);

#[cfg(not(windows))]
impl ManagedChild {
    fn id(&self) -> u32 {
        self.0.id()
    }
    fn try_wait(&mut self) -> std::result::Result<Option<i32>, ()> {
        self.0
            .try_wait()
            .map(|status| status.map(|status| status.code().unwrap_or(-1)))
            .map_err(|_| ())
    }
    fn kill(&mut self) -> std::result::Result<(), ()> {
        self.0.kill().map_err(|_| ())
    }
    fn wait(&mut self) -> std::result::Result<i32, ()> {
        self.0
            .wait()
            .map(|status| status.code().unwrap_or(-1))
            .map_err(|_| ())
    }
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
            assign_before_execution: false,
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
    child: ManagedChild,
    stdin: Option<Box<dyn Write + Send>>,
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

        // M3 direct providers are created suspended until they are in the Job
        // Object. Other pre-existing host children retain their frozen launch
        // path and are assigned immediately after creation.
        #[cfg(windows)]
        let (child, stdin, stdout, stderr_r) = if config.assign_before_execution {
            match spawn_suspended_in_job(&config, job_handle) {
                Ok(process) => process,
                Err(error) => {
                    close_handle(job_handle);
                    return Err(error);
                }
            }
        } else {
            match spawn_then_assign_to_job(&config, job_handle) {
                Ok(process) => process,
                Err(error) => {
                    close_handle(job_handle);
                    return Err(error);
                }
            }
        };

        #[cfg(not(windows))]
        let (child, stdin, stdout, stderr_r) = {
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
            let stdin = child.stdin.take().ok_or(ChildError::StdinUnavailable)?;
            let stdout = child.stdout.take().ok_or(ChildError::StdoutUnavailable)?;
            let stderr_r = child.stderr.take().ok_or(ChildError::StderrUnavailable)?;
            (
                ManagedChild(child),
                Box::new(stdin) as Box<dyn Write + Send>,
                Box::new(stdout) as Box<dyn Read + Send>,
                Box::new(stderr_r) as Box<dyn Read + Send>,
            )
        };

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
                        // Update shared buffer incrementally for live visibility.
                        if let Ok(mut guard) = stderr_buf.lock() {
                            guard.clone_from(&buf);
                        }
                    }
                    Err(_) => break,
                }
            }
            if let Ok(mut guard) = stderr_buf.lock() {
                guard.clone_from(&buf);
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
                        Ok(Some(status)) => ChildError::ProcessExited(status),
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
                    Ok(Some(status)) => ChildError::ProcessExited(status),
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
fn read_until_newline<R: BufRead>(
    reader: &mut R,
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
fn quoted_windows_argument(value: &str) -> String {
    if !value.is_empty()
        && !value
            .bytes()
            .any(|byte| byte == b' ' || byte == b'\t' || byte == b'"')
    {
        return value.to_owned();
    }
    let mut quoted = String::from("\"");
    let mut slashes = 0usize;
    for character in value.chars() {
        match character {
            '\\' => slashes += 1,
            '"' => {
                quoted.push_str(&"\\".repeat(slashes.saturating_mul(2).saturating_add(1)));
                quoted.push('"');
                slashes = 0;
            }
            _ => {
                quoted.push_str(&"\\".repeat(slashes));
                quoted.push(character);
                slashes = 0;
            }
        }
    }
    quoted.push_str(&"\\".repeat(slashes.saturating_mul(2)));
    quoted.push('"');
    quoted
}

#[cfg(windows)]
fn nul_terminated_wide(value: &str, field: &str) -> Result<Vec<u16>, ChildError> {
    if value.contains('\0') {
        return Err(ChildError::LaunchFailed {
            command: field.to_owned(),
            message: "embedded NUL is not permitted".to_owned(),
        });
    }
    Ok(value.encode_utf16().chain(std::iter::once(0)).collect())
}

#[cfg(windows)]
fn windows_environment_block(config: &ChildConfig) -> Result<Option<Vec<u16>>, ChildError> {
    if !config.clear_environment && config.environment.is_empty() {
        return Ok(None);
    }
    let mut values = BTreeMap::<String, (String, String)>::new();
    if !config.clear_environment {
        for (name, value) in std::env::vars() {
            values.insert(name.to_ascii_uppercase(), (name, value));
        }
    }
    for (name, value) in &config.environment {
        if name.is_empty() || name.contains('=') || name.contains('\0') || value.contains('\0') {
            return Err(ChildError::LaunchFailed {
                command: config.command.clone(),
                message: "invalid environment entry".to_owned(),
            });
        }
        values.insert(name.to_ascii_uppercase(), (name.clone(), value.clone()));
    }
    let mut block = Vec::new();
    for (_, (name, value)) in values {
        block.extend(format!("{name}={value}").encode_utf16());
        block.push(0);
    }
    block.push(0);
    Ok(Some(block))
}

#[cfg(windows)]
fn close_handle(handle: windows_sys::Win32::Foundation::HANDLE) {
    if !handle.is_null() && handle != windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE {
        // SAFETY: this helper is called only for owned Win32 handles and never twice.
        unsafe { windows_sys::Win32::Foundation::CloseHandle(handle) };
    }
}

/// Owns the extended-startup handle allow-list for an M3 direct-provider launch.
/// The value buffer must remain live through `CreateProcessW`; its destructor
/// also guarantees `DeleteProcThreadAttributeList` on every return path.
#[cfg(windows)]
struct InheritedHandleList {
    storage: Vec<usize>,
    handles: Vec<windows_sys::Win32::Foundation::HANDLE>,
    initialized: bool,
}

#[cfg(windows)]
impl InheritedHandleList {
    fn new(
        handles: [windows_sys::Win32::Foundation::HANDLE; 3],
        command: &str,
    ) -> Result<Self, ChildError> {
        use windows_sys::Win32::System::Threading::{
            InitializeProcThreadAttributeList, UpdateProcThreadAttribute,
            PROC_THREAD_ATTRIBUTE_HANDLE_LIST,
        };

        let mut bytes = 0usize;
        // SAFETY: this sizing call deliberately has a null list pointer.
        unsafe {
            InitializeProcThreadAttributeList(std::ptr::null_mut(), 1, 0, &mut bytes);
        }
        if bytes == 0 {
            return Err(ChildError::LaunchFailed {
                command: command.to_owned(),
                message: "InitializeProcThreadAttributeList sizing failed".to_owned(),
            });
        }
        let words = bytes.div_ceil(std::mem::size_of::<usize>());
        let mut list = Self {
            storage: vec![0; words],
            handles: handles.to_vec(),
            initialized: false,
        };
        let pointer = list.storage.as_mut_ptr()
            as windows_sys::Win32::System::Threading::LPPROC_THREAD_ATTRIBUTE_LIST;
        // SAFETY: the aligned storage is at least the size reported by Windows.
        if unsafe { InitializeProcThreadAttributeList(pointer, 1, 0, &mut bytes) } == 0 {
            return Err(ChildError::LaunchFailed {
                command: command.to_owned(),
                message: "InitializeProcThreadAttributeList failed".to_owned(),
            });
        }
        list.initialized = true;
        // SAFETY: the three handles are valid inheritable child pipe endpoints;
        // the handle vector and attribute list outlive CreateProcessW.
        if unsafe {
            UpdateProcThreadAttribute(
                pointer,
                0,
                PROC_THREAD_ATTRIBUTE_HANDLE_LIST as usize,
                list.handles.as_ptr() as *const std::ffi::c_void,
                std::mem::size_of_val(list.handles.as_slice()),
                std::ptr::null_mut(),
                std::ptr::null(),
            )
        } == 0
        {
            return Err(ChildError::LaunchFailed {
                command: command.to_owned(),
                message: "UpdateProcThreadAttribute handle allow-list failed".to_owned(),
            });
        }
        Ok(list)
    }

    fn pointer(&mut self) -> windows_sys::Win32::System::Threading::LPPROC_THREAD_ATTRIBUTE_LIST {
        self.storage.as_mut_ptr()
            as windows_sys::Win32::System::Threading::LPPROC_THREAD_ATTRIBUTE_LIST
    }
}

#[cfg(windows)]
impl Drop for InheritedHandleList {
    fn drop(&mut self) {
        if self.initialized {
            // SAFETY: this is the one matching destructor call after successful
            // InitializeProcThreadAttributeList for the owned storage.
            unsafe {
                windows_sys::Win32::System::Threading::DeleteProcThreadAttributeList(
                    self.pointer(),
                );
            }
        }
    }
}

#[cfg(windows)]
fn spawn_then_assign_to_job(
    config: &ChildConfig,
    job_handle: windows_sys::Win32::Foundation::HANDLE,
) -> Result<
    (
        ManagedChild,
        Box<dyn Write + Send>,
        Box<dyn Read + Send>,
        Box<dyn Read + Send>,
    ),
    ChildError,
> {
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::System::JobObjects::AssignProcessToJobObject;

    let mut command = Command::new(&config.command);
    command
        .args(&config.args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if config.clear_environment {
        command.env_clear();
    }
    command.envs(&config.environment);
    if let Some(directory) = &config.current_dir {
        command.current_dir(directory);
    }
    let mut process = command.spawn().map_err(|error| ChildError::LaunchFailed {
        command: config.command.clone(),
        message: error.to_string(),
    })?;
    let stdin = process.stdin.take().ok_or(ChildError::StdinUnavailable)?;
    let stdout = process.stdout.take().ok_or(ChildError::StdoutUnavailable)?;
    let stderr = process.stderr.take().ok_or(ChildError::StderrUnavailable)?;
    let process_id = process.id();
    let process_handle = process.as_raw_handle() as windows_sys::Win32::Foundation::HANDLE;
    // SAFETY: the handle belongs to `process` until it is deliberately
    // forgotten below after successful assignment, transferring ownership to
    // ManagedChild. On failure the normal Child destructor retains ownership.
    if unsafe { AssignProcessToJobObject(job_handle, process_handle) } == 0 {
        let _ = process.kill();
        let _ = process.wait();
        return Err(ChildError::JobObjectFailed(
            "AssignProcessToJobObject failed".to_owned(),
        ));
    }
    std::mem::forget(process);
    Ok((
        ManagedChild {
            process_handle,
            process_id,
        },
        Box::new(stdin),
        Box::new(stdout),
        Box::new(stderr),
    ))
}

#[cfg(windows)]
fn spawn_suspended_in_job(
    config: &ChildConfig,
    job_handle: windows_sys::Win32::Foundation::HANDLE,
) -> Result<
    (
        ManagedChild,
        Box<dyn Write + Send>,
        Box<dyn Read + Send>,
        Box<dyn Read + Send>,
    ),
    ChildError,
> {
    use std::os::windows::io::FromRawHandle;
    use windows_sys::Win32::Foundation::{SetHandleInformation, HANDLE_FLAG_INHERIT};
    use windows_sys::Win32::Security::SECURITY_ATTRIBUTES;
    use windows_sys::Win32::System::JobObjects::{AssignProcessToJobObject, TerminateJobObject};
    use windows_sys::Win32::System::Pipes::CreatePipe;
    use windows_sys::Win32::System::Threading::{
        CreateProcessW, ResumeThread, TerminateProcess, CREATE_SUSPENDED,
        CREATE_UNICODE_ENVIRONMENT, EXTENDED_STARTUPINFO_PRESENT, PROCESS_INFORMATION,
        STARTF_USESTDHANDLES, STARTUPINFOEXW,
    };

    let failure = |message: &str| ChildError::LaunchFailed {
        command: config.command.clone(),
        message: message.to_owned(),
    };
    let security = SECURITY_ATTRIBUTES {
        nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: std::ptr::null_mut(),
        bInheritHandle: 1,
    };
    let (mut stdin_read, mut stdin_write) = (std::ptr::null_mut(), std::ptr::null_mut());
    let (mut stdout_read, mut stdout_write) = (std::ptr::null_mut(), std::ptr::null_mut());
    let (mut stderr_read, mut stderr_write) = (std::ptr::null_mut(), std::ptr::null_mut());
    // SAFETY: the output pointers are valid owned local storage and the security
    // descriptor remains live for each call.
    let pipes_ready = unsafe {
        CreatePipe(&mut stdin_read, &mut stdin_write, &security, 0) != 0
            && CreatePipe(&mut stdout_read, &mut stdout_write, &security, 0) != 0
            && CreatePipe(&mut stderr_read, &mut stderr_write, &security, 0) != 0
    };
    if !pipes_ready {
        close_handle(stdin_read);
        close_handle(stdin_write);
        close_handle(stdout_read);
        close_handle(stdout_write);
        close_handle(stderr_read);
        close_handle(stderr_write);
        return Err(failure("CreatePipe failed"));
    }
    // SAFETY: the parent ends are valid handles. Clearing inheritance ensures
    // only the three child ends cross the process boundary.
    let parent_ends_private = unsafe {
        SetHandleInformation(stdin_write, HANDLE_FLAG_INHERIT, 0) != 0
            && SetHandleInformation(stdout_read, HANDLE_FLAG_INHERIT, 0) != 0
            && SetHandleInformation(stderr_read, HANDLE_FLAG_INHERIT, 0) != 0
    };
    if !parent_ends_private {
        close_handle(stdin_read);
        close_handle(stdin_write);
        close_handle(stdout_read);
        close_handle(stdout_write);
        close_handle(stderr_read);
        close_handle(stderr_write);
        return Err(failure("SetHandleInformation failed"));
    }
    let application = std::path::Path::new(&config.command)
        .is_absolute()
        .then(|| nul_terminated_wide(&config.command, &config.command))
        .transpose()?;
    let command_line = std::iter::once(quoted_windows_argument(&config.command))
        .chain(
            config
                .args
                .iter()
                .map(|argument| quoted_windows_argument(argument)),
        )
        .collect::<Vec<_>>()
        .join(" ");
    let mut command_line = nul_terminated_wide(&command_line, &config.command)?;
    let current_directory = config
        .current_dir
        .as_ref()
        .map(|path| nul_terminated_wide(&path.to_string_lossy(), &config.command))
        .transpose()?;
    let environment = windows_environment_block(config)?;
    let mut inherited_handles =
        match InheritedHandleList::new([stdin_read, stdout_write, stderr_write], &config.command) {
            Ok(list) => list,
            Err(error) => {
                close_handle(stdin_read);
                close_handle(stdin_write);
                close_handle(stdout_read);
                close_handle(stdout_write);
                close_handle(stderr_read);
                close_handle(stderr_write);
                return Err(error);
            }
        };
    let mut startup: STARTUPINFOEXW = unsafe { std::mem::zeroed() };
    startup.StartupInfo.cb = std::mem::size_of::<STARTUPINFOEXW>() as u32;
    startup.StartupInfo.dwFlags = STARTF_USESTDHANDLES;
    startup.StartupInfo.hStdInput = stdin_read;
    startup.StartupInfo.hStdOutput = stdout_write;
    startup.StartupInfo.hStdError = stderr_write;
    startup.lpAttributeList = inherited_handles.pointer();
    let mut process: PROCESS_INFORMATION = unsafe { std::mem::zeroed() };
    // SAFETY: all UTF-16 buffers are NUL-terminated and live for this call;
    // startup and process storage are correctly initialised; only child pipe
    // endpoints are inheritable; CREATE_SUSPENDED prevents provider code from
    // running until Job Object assignment succeeds.
    let created = unsafe {
        CreateProcessW(
            application
                .as_ref()
                .map_or(std::ptr::null(), |path| path.as_ptr()),
            command_line.as_mut_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            1,
            CREATE_SUSPENDED | CREATE_UNICODE_ENVIRONMENT | EXTENDED_STARTUPINFO_PRESENT,
            environment
                .as_ref()
                .map_or(std::ptr::null(), |block| block.as_ptr() as *const _),
            current_directory
                .as_ref()
                .map_or(std::ptr::null(), |path| path.as_ptr()),
            &startup.StartupInfo,
            &mut process,
        )
    };
    close_handle(stdin_read);
    close_handle(stdout_write);
    close_handle(stderr_write);
    if created == 0 {
        close_handle(stdin_write);
        close_handle(stdout_read);
        close_handle(stderr_read);
        return Err(failure("CreateProcessW failed"));
    }
    // SAFETY: CreateProcessW returned owned process/thread handles. The process
    // is still suspended, so no provider instruction has executed.
    let assigned = unsafe { AssignProcessToJobObject(job_handle, process.hProcess) };
    if assigned == 0 {
        unsafe { TerminateProcess(process.hProcess, 1) };
        close_handle(process.hThread);
        close_handle(process.hProcess);
        close_handle(stdin_write);
        close_handle(stdout_read);
        close_handle(stderr_read);
        return Err(ChildError::JobObjectFailed(
            "AssignProcessToJobObject failed".to_owned(),
        ));
    }
    // SAFETY: the primary thread is suspended by CreateProcessW and belongs to
    // the Job Object before this call. A u32::MAX return reports failure.
    let resumed = unsafe { ResumeThread(process.hThread) };
    close_handle(process.hThread);
    if resumed == u32::MAX {
        unsafe { TerminateJobObject(job_handle, 1) };
        close_handle(process.hProcess);
        close_handle(stdin_write);
        close_handle(stdout_read);
        close_handle(stderr_read);
        return Err(ChildError::JobObjectFailed(
            "ResumeThread failed".to_owned(),
        ));
    }
    // SAFETY: ownership of these parent handles transfers exactly once to the
    // standard-library process/pipe wrappers; child endpoints were closed above.
    let child = ManagedChild {
        process_handle: process.hProcess,
        process_id: process.dwProcessId,
    };
    let stdin = unsafe { File::from_raw_handle(stdin_write as _) };
    let stdout = unsafe { File::from_raw_handle(stdout_read as _) };
    let stderr = unsafe { File::from_raw_handle(stderr_read as _) };
    Ok((
        child,
        Box::new(stdin) as Box<dyn Write + Send>,
        Box::new(stdout) as Box<dyn Read + Send>,
        Box::new(stderr) as Box<dyn Read + Send>,
    ))
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
    fn j13a_named_powershell_child_preserves_piped_protocol() {
        let mut child = launch_fixture("valid", 5, 2).expect("launch");
        child
            .write_line(
                r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","clientInfo":{"name":"test","version":"1"},"capabilities":{}}}"#,
            )
            .expect("write initialize");
        let response = child
            .read_protocol_line(Duration::from_secs(5))
            .expect("read initialize response");
        assert!(response.contains(r#""id":1"#), "response: {response}");
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

    // ── F2a: live stderr visibility ─────────────────────────────────

    fn launch_live_stderr_fixture(
        startup: u64,
        close: u64,
    ) -> Result<SupervisedChild, ChildError> {
        let config = ChildConfig::test_config(
            "pwsh.exe",
            vec![
                "-NoProfile".to_owned(),
                "-Command".to_owned(),
                "& { [Console]::Error.WriteLine('STDERR_MARKER_READY'); [Console]::Error.Flush(); [Console]::Out.WriteLine('READY'); [Console]::Out.Flush(); Start-Sleep -Seconds 30 }"
                    .to_owned(),
            ],
            Duration::from_secs(startup),
            Duration::from_secs(close),
        );
        SupervisedChild::launch(config)
    }

    /// Poll `stderr_tail()` until `marker` is present or `deadline` passes.
    fn poll_stderr_tail(child: &SupervisedChild, marker: &str, deadline: Instant) -> String {
        loop {
            let tail = child.stderr_tail();
            if tail.contains(marker) {
                return tail;
            }
            if Instant::now() >= deadline {
                return tail;
            }
            thread::sleep(Duration::from_millis(5));
        }
    }

    #[test]
    fn f2a_regression_live_stderr_not_visible_before_exit() {
        let mut child = launch_live_stderr_fixture(5, 2).expect("launch");
        let line = child
            .read_protocol_line(Duration::from_secs(5))
            .expect("stdout ready");
        assert!(line.contains("READY"), "expected READY, got: {line}");

        let deadline = Instant::now() + Duration::from_secs(3);
        let tail = poll_stderr_tail(&child, "STDERR_MARKER_READY", deadline);
        assert!(
            tail.contains("STDERR_MARKER_READY"),
            "stderr must be visible while child is alive; got: {tail}"
        );

        child.shutdown();
    }

    #[test]
    fn f2a_live_stderr_visible_before_exit() {
        let mut child = launch_live_stderr_fixture(5, 2).expect("launch");
        let line = child
            .read_protocol_line(Duration::from_secs(5))
            .expect("stdout ready");
        assert!(line.contains("READY"), "expected READY, got: {line}");

        let deadline = Instant::now() + Duration::from_secs(3);
        let tail = poll_stderr_tail(&child, "STDERR_MARKER_READY", deadline);
        assert!(
            tail.contains("STDERR_MARKER_READY"),
            "stderr must be visible while child is alive; got: {tail}"
        );
        assert!(
            !child.has_exited(),
            "child must still be alive during stderr observation"
        );

        child.shutdown();
    }

    #[test]
    fn f2a_bounded_stderr_tail() {
        let small_tail: usize = 100;
        let config = ChildConfig {
            stderr_tail_bytes: small_tail,
            ..ChildConfig::test_config(
                "pwsh.exe",
                vec![
                    "-NoProfile".to_owned(),
                    "-Command".to_owned(),
                    "& { 1..50 | ForEach-Object { [Console]::Error.WriteLine(('LINE_' + ($_ -as [string]).PadLeft(4,'0') + '_MARKER')); Start-Sleep -Milliseconds 1 }; [Console]::Error.Flush(); [Console]::Out.WriteLine('READY'); [Console]::Out.Flush(); Start-Sleep -Seconds 30 }"
                        .to_owned(),
                ],
                Duration::from_secs(10),
                Duration::from_secs(2),
            )
        };
        let mut child = SupervisedChild::launch(config).expect("launch");
        let line = child
            .read_protocol_line(Duration::from_secs(10))
            .expect("stdout ready");
        assert!(line.contains("READY"), "expected READY, got: {line}");

        let deadline = Instant::now() + Duration::from_secs(5);
        let tail = poll_stderr_tail(&child, "LINE_0050_MARKER", deadline);
        assert!(
            tail.contains("LINE_0050_MARKER"),
            "newest stderr line must be retained; got: {tail}"
        );
        assert!(
            tail.len() <= small_tail + 64,
            "tail must respect configured byte limit ({small_tail}); actual: {}",
            tail.len()
        );
        assert!(
            !tail.contains("LINE_0001_MARKER"),
            "oldest stderr must be evicted; got: {tail}"
        );

        child.shutdown();
    }

    #[test]
    fn f2a_timeout_remains_timeout_with_stderr_available() {
        let mut child = launch_live_stderr_fixture(5, 2).expect("launch");
        let line = child
            .read_protocol_line(Duration::from_secs(5))
            .expect("first line");
        assert!(line.contains("READY"), "expected READY, got: {line}");

        let result = child.read_protocol_line(Duration::from_millis(100));
        assert!(
            matches!(result, Err(ChildError::ReadTimeout(_))),
            "second read must timeout; got: {result:?}"
        );

        let tail = child.stderr_tail();
        assert!(
            tail.contains("STDERR_MARKER_READY"),
            "stderr emitted before timeout must remain available; got: {tail}"
        );

        child.shutdown();
    }

    #[test]
    fn f2a_exit_distinguishable_from_timeout_and_disconnect() {
        let config = ChildConfig::test_config(
            "pwsh.exe",
            vec![
                "-NoProfile".to_owned(),
                "-Command".to_owned(),
                "& { [Console]::Out.WriteLine('READY'); [Console]::Out.Flush(); exit 7 }"
                    .to_owned(),
            ],
            Duration::from_secs(5),
            Duration::from_secs(2),
        );
        let mut child = SupervisedChild::launch(config).expect("launch");
        let line = child
            .read_protocol_line(Duration::from_secs(5))
            .expect("first line");
        assert!(line.contains("READY"), "expected READY, got: {line}");

        let result = child.read_protocol_line(Duration::from_millis(200));
        assert!(
            matches!(result, Err(ChildError::ProcessExited(7))),
            "child exit with code 7 must produce ProcessExited; got: {result:?}"
        );

        assert!(child.has_exited(), "child must be reaped after exit");
        child.shutdown();
    }

    #[test]
    fn f2a_windows_cleanup_reaps_child_and_joins_threads() {
        let child = launch_live_stderr_fixture(5, 2).expect("launch");
        let child_id = child.child.id();
        assert!(child_id > 0, "child must have a valid PID");
        child.shutdown();

        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            let state = std::process::Command::new("pwsh")
                .args([
                    "-NoProfile",
                    "-Command",
                    &format!(
                        "if (Get-Process -Id {child_id} -ErrorAction SilentlyContinue) {{ exit 1 }}"
                    ),
                ])
                .status();
            match state {
                Ok(status) if status.code() == Some(0) => break,
                Ok(_) => {
                    if Instant::now() >= deadline {
                        panic!("child process {child_id} still alive after shutdown");
                    }
                    thread::sleep(Duration::from_millis(50));
                }
                Err(_) => break,
            }
        }
    }
}
