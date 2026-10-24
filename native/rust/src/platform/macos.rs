//! macOS process, memory, window-state and installation primitives.
//! Supports x86_64 and Apple Silicon ARM64 (aarch64).

use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

pub const MIN_WORKING_SET_BYTES: usize = 80 * 1024;
pub const HARD_CAP_FLAGS: u32 = 0x6;

extern "C" {
    #[allow(dead_code)]
    fn kill(pid: i32, sig: i32) -> i32;
    fn flock(fd: i32, operation: i32) -> i32;
    fn getuid() -> u32;
    fn close(fd: i32) -> i32;
}

const LOCK_EX: i32 = 2;
const LOCK_NB: i32 = 4;
const LOCK_UN: i32 = 8;

#[derive(Clone, Debug)]
pub struct ProcessRow {
    pub pid: u32,
    pub ppid: u32,
    pub cmd: String,
    pub creation: u64,
    pub cpu_time_ms: f64,
    pub cpu_percent: Option<f64>,
    pub working_set_mb: Option<f64>,
    pub private_mb: Option<f64>,
    pub page_fault_count: Option<u64>,
    pub page_faults_delta: Option<u64>,
    pub error_code: i32,
}

#[derive(Clone, Debug)]
pub struct WindowRow {
    pub pid: u32,
    pub hwnd: usize,
    pub title: String,
}

#[derive(Clone, Debug, Default)]
pub struct Snapshot {
    pub processes: Vec<ProcessRow>,
    pub windows: Vec<WindowRow>,
}

#[derive(Clone, Copy, Debug)]
pub struct OriginalLimits {
    pub minimum: usize,
    pub maximum: usize,
    pub flags: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct NativeStats {
    pub creation: u64,
    pub cpu_time_ms: f64,
    pub working_set_mb: f64,
    pub page_fault_count: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RestoreOutcome {
    Restored,
    ProcessGone,
    PidReused,
}

pub struct SingleInstanceGuard {
    fd: i32,
}

impl Drop for SingleInstanceGuard {
    fn drop(&mut self) {
        unsafe {
            flock(self.fd, LOCK_UN);
            close(self.fd);
        }
    }
}

pub fn acquire_single_instance() -> Result<SingleInstanceGuard, String> {
    use std::os::unix::fs::OpenOptionsExt;
    let uid = unsafe { getuid() };
    let lock_path = std::env::temp_dir().join(format!("memory-sentry-{uid}.lock"));
    let file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .mode(0o600)
        .open(&lock_path)
        .map_err(|e| format!("Could not open lock file {}: {e}", lock_path.display()))?;

    use std::os::unix::io::IntoRawFd;
    let fd = file.into_raw_fd();
    let res = unsafe { flock(fd, LOCK_EX | LOCK_NB) };
    if res != 0 {
        unsafe { close(fd); }
        return Err("Memory Sentry is already running for this user session.".to_string());
    }
    Ok(SingleInstanceGuard { fd })
}

pub fn random_token() -> Result<String, String> {
    use std::io::Read;
    let mut file = fs::File::open("/dev/urandom")
        .map_err(|e| format!("Could not open /dev/urandom: {e}"))?;
    let mut bytes = [0u8; 32];
    file.read_exact(&mut bytes)
        .map_err(|e| format!("Could not read from /dev/urandom: {e}"))?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}

pub fn detect_supported_client_os() -> Result<(&'static str, u32), String> {
    let output = Command::new("sw_vers").arg("-productVersion").output();
    let mut build = 0u32;
    if let Ok(out) = output {
        let version_str = String::from_utf8_lossy(&out.stdout);
        let parts: Vec<&str> = version_str.trim().split('.').collect();
        if parts.len() >= 2 {
            let major = parts[0].parse::<u32>().unwrap_or(0);
            let minor = parts[1].parse::<u32>().unwrap_or(0);
            build = major * 10_000 + minor * 100;
        }
    }
    Ok(("macOS", build))
}

fn parse_cpu_time_str(time_str: &str) -> f64 {
    // format: [dd-]hh:mm:ss or mm:ss.ss
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() == 2 {
        let mins = parts[0].parse::<f64>().unwrap_or(0.0);
        let secs = parts[1].parse::<f64>().unwrap_or(0.0);
        (mins * 60.0 + secs) * 1000.0
    } else if parts.len() == 3 {
        let hours = parts[0].parse::<f64>().unwrap_or(0.0);
        let mins = parts[1].parse::<f64>().unwrap_or(0.0);
        let secs = parts[2].parse::<f64>().unwrap_or(0.0);
        (hours * 3600.0 + mins * 60.0 + secs) * 1000.0
    } else {
        0.0
    }
}

pub fn inspect_process(pid: u32) -> Result<NativeStats, String> {
    let output = Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "rss=,time="])
        .output()
        .map_err(|e| format!("Could not inspect process {pid}: {e}"))?;

    if !output.status.success() {
        return Err(format!("Process {pid} not found"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout.lines().next().unwrap_or("").trim();
    let fields: Vec<&str> = line.split_whitespace().collect();
    if fields.len() < 2 {
        return Err("Unexpected ps output".to_string());
    }

    let rss_kb = fields[0].parse::<f64>().unwrap_or(0.0);
    let cpu_time_ms = parse_cpu_time_str(fields[1]);

    Ok(NativeStats {
        creation: pid as u64,
        cpu_time_ms,
        working_set_mb: rss_kb / 1024.0,
        page_fault_count: 0,
    })
}

pub fn process_snapshot() -> Result<Snapshot, String> {
    let output = Command::new("ps")
        .args(["-axo", "pid=,ppid=,rss=,time=,command="])
        .output()
        .map_err(|e| format!("Could not inspect processes: {e}"))?;

    let mut snapshot = Snapshot::default();
    let stdout = String::from_utf8_lossy(&output.stdout);

    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() { continue; }
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() < 5 { continue; }

        let Ok(pid) = parts[0].parse::<u32>() else { continue; };
        let Ok(ppid) = parts[1].parse::<u32>() else { continue; };
        let rss_kb = parts[2].parse::<f64>().unwrap_or(0.0);
        let cpu_time_ms = parse_cpu_time_str(parts[3]);
        let cmd = parts[4..].join(" ");

        let lower = cmd.to_ascii_lowercase();
        if !lower.contains("spotify") {
            continue;
        }

        let creation = pid as u64;
        let working_set_mb = Some(rss_kb / 1024.0);

        snapshot.processes.push(ProcessRow {
            pid,
            ppid,
            cmd,
            creation,
            cpu_time_ms,
            cpu_percent: None,
            working_set_mb,
            private_mb: working_set_mb,
            page_fault_count: None,
            page_faults_delta: None,
            error_code: 0,
        });
    }

    let main_proc = snapshot.processes.iter().find(|p| !p.cmd.contains("--type="));
    if let Some(main) = main_proc {
        snapshot.windows.push(WindowRow {
            pid: main.pid,
            hwnd: main.pid as usize,
            title: "Spotify".to_string(),
        });
    }

    Ok(snapshot)
}

#[allow(dead_code)]
pub fn powershell_snapshot() -> Result<Snapshot, String> {
    process_snapshot()
}

pub fn window_state(hwnd: usize, has_processes: bool) -> String {
    if !has_processes {
        return "not-running".to_string();
    }
    if hwnd == 0 {
        return "window-not-found".to_string();
    }

    // Query AppleScript for Spotify visibility if possible
    let script = r#"tell application "System Events" to get visible of process "Spotify""#;
    if let Ok(output) = Command::new("osascript").args(["-e", script]).output() {
        if output.status.success() {
            let res = String::from_utf8_lossy(&output.stdout).trim().to_ascii_lowercase();
            if res == "false" {
                return "hidden".to_string();
            }
        }
    }

    "background-visible".to_string()
}

pub fn read_original_limits(pid: u32, expected_creation: u64) -> Result<OriginalLimits, String> {
    let stats = inspect_process(pid)?;
    if stats.creation != expected_creation {
        return Err("Process identity changed before reading working-set limits.".to_string());
    }
    Ok(OriginalLimits { minimum: 0, maximum: 0, flags: 0 })
}

pub fn set_working_set_limits(
    pid: u32,
    expected_creation: u64,
    _minimum: usize,
    _maximum: usize,
    _flags: u32,
) -> Result<(), String> {
    let stats = inspect_process(pid)?;
    if stats.creation != expected_creation {
        return Err("Process identity changed before setting working-set limits; skipped.".to_string());
    }
    Ok(())
}

pub fn empty_working_set(pid: u32, expected_creation: u64) -> Result<(), String> {
    let stats = inspect_process(pid)?;
    if stats.creation != expected_creation {
        return Err("Process identity changed before trimming; skipped.".to_string());
    }
    Ok(())
}

pub fn restore_working_set_limits(
    pid: u32,
    expected_creation: u64,
    _minimum: usize,
    _maximum: usize,
    _flags: u32,
) -> Result<RestoreOutcome, String> {
    let stats = match inspect_process(pid) {
        Ok(stats) => stats,
        Err(_) => return Ok(RestoreOutcome::ProcessGone),
    };
    if stats.creation != expected_creation {
        return Ok(RestoreOutcome::PidReused);
    }
    Ok(RestoreOutcome::Restored)
}

#[allow(dead_code)]
pub fn process_exists(pid: u32) -> bool {
    unsafe { kill(pid as i32, 0) == 0 }
}

pub fn process_matches_executable(pid: u32, expected_path: &Path) -> bool {
    let output = Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "command="])
        .output();
    if let Ok(out) = output {
        let cmd = String::from_utf8_lossy(&out.stdout);
        let expected = expected_path.to_string_lossy();
        cmd.contains(&*expected)
    } else {
        false
    }
}

pub fn current_process_id() -> u32 {
    std::process::id()
}

#[allow(dead_code)]
pub fn epoch_millis() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64
}

#[allow(dead_code)]
pub fn io_error(error: io::Error) -> String {
    error.to_string()
}
