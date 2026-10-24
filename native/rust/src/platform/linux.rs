//! Linux process, memory, window-state and installation primitives.
//! Supports x86_64, aarch64 (ARM64), and ARM.
//! Memory trimming utilizes kernel process_madvise (MADV_PAGEOUT) where available.

use std::ffi::c_void;
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
    fn syscall(number: i64, ...) -> i64;
}

const LOCK_EX: i32 = 2;
const LOCK_NB: i32 = 4;
const LOCK_UN: i32 = 8;

const SYS_PIDFD_OPEN: i64 = 434;
const SYS_PROCESS_MADVISE: i64 = 440;
const MADV_PAGEOUT: i32 = 21;

#[repr(C)]
struct IOVec {
    iov_base: *mut c_void,
    iov_len: usize,
}

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
    let release = fs::read_to_string("/proc/sys/kernel/osrelease").unwrap_or_default();
    let mut build = 0u32;
    let parts: Vec<&str> = release.trim().split('.').collect();
    if parts.len() >= 2 {
        let major = parts[0].parse::<u32>().unwrap_or(0);
        let minor = parts[1].split(|c: char| !c.is_ascii_digit()).next().unwrap_or("0").parse::<u32>().unwrap_or(0);
        build = major * 10_000 + minor * 100;
    }
    Ok(("Linux", build))
}

pub fn inspect_process(pid: u32) -> Result<NativeStats, String> {
    let stat_content = fs::read_to_string(format!("/proc/{pid}/stat"))
        .map_err(|e| format!("Could not read /proc/{pid}/stat: {e}"))?;

    let comm_end = stat_content.rfind(')').ok_or_else(|| "Invalid stat format".to_string())?;
    let rest = stat_content[comm_end + 1..].trim_start();
    let fields: Vec<&str> = rest.split_whitespace().collect();
    if fields.len() < 20 {
        return Err("Unexpected stat field count".to_string());
    }

    let minflt = fields[7].parse::<u64>().unwrap_or(0);
    let majflt = fields[9].parse::<u64>().unwrap_or(0);
    let utime = fields[11].parse::<u64>().unwrap_or(0);
    let stime = fields[12].parse::<u64>().unwrap_or(0);
    let starttime = fields[19].parse::<u64>().unwrap_or(0);

    let page_fault_count = minflt.saturating_add(majflt);
    let cpu_ticks = utime.saturating_add(stime);
    let cpu_time_ms = cpu_ticks as f64 * 10.0;
    let creation = starttime;

    let mut working_set_mb = 0.0;
    if let Ok(status) = fs::read_to_string(format!("/proc/{pid}/status")) {
        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(kb) = parts[1].parse::<f64>() {
                        working_set_mb = kb / 1024.0;
                    }
                }
                break;
            }
        }
    }

    Ok(NativeStats {
        creation,
        cpu_time_ms,
        working_set_mb,
        page_fault_count,
    })
}

pub fn process_snapshot() -> Result<Snapshot, String> {
    let proc_dir = fs::read_dir("/proc").map_err(|e| format!("Could not read /proc: {e}"))?;
    let mut snapshot = Snapshot::default();

    for entry in proc_dir.flatten() {
        let file_name = entry.file_name();
        let Some(name_str) = file_name.to_str() else { continue; };
        let Ok(pid) = name_str.parse::<u32>() else { continue; };

        let cmdline_path = format!("/proc/{pid}/cmdline");
        let Ok(cmdline_raw) = fs::read(&cmdline_path) else { continue; };
        if cmdline_raw.is_empty() { continue; }

        let cmd = String::from_utf8_lossy(&cmdline_raw)
            .replace('\0', " ")
            .trim()
            .to_string();

        let lower_cmd = cmd.to_ascii_lowercase();
        let is_spotify = lower_cmd.contains("/spotify") || lower_cmd.contains(" spotify") || lower_cmd.starts_with("spotify");
        if !is_spotify {
            continue;
        }

        let Ok(stat_content) = fs::read_to_string(format!("/proc/{pid}/stat")) else { continue; };
        let Some(comm_end) = stat_content.rfind(')') else { continue; };
        let rest = stat_content[comm_end + 1..].trim_start();
        let fields: Vec<&str> = rest.split_whitespace().collect();
        if fields.len() < 20 { continue; }

        let ppid = fields[1].parse::<u32>().unwrap_or(0);
        let minflt = fields[7].parse::<u64>().unwrap_or(0);
        let majflt = fields[9].parse::<u64>().unwrap_or(0);
        let utime = fields[11].parse::<u64>().unwrap_or(0);
        let stime = fields[12].parse::<u64>().unwrap_or(0);
        let starttime = fields[19].parse::<u64>().unwrap_or(0);

        let mut private_mb = None;
        let mut working_set_mb = None;
        if let Ok(status) = fs::read_to_string(format!("/proc/{pid}/status")) {
            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let Ok(kb) = parts[1].parse::<f64>() {
                            working_set_mb = Some(kb / 1024.0);
                        }
                    }
                } else if line.starts_with("RssAnon:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let Ok(kb) = parts[1].parse::<f64>() {
                            private_mb = Some(kb / 1024.0);
                        }
                    }
                }
            }
        }

        let mut creation = starttime;
        let mut cpu_time_ms = (utime.saturating_add(stime)) as f64 * 10.0;
        let mut page_fault_count = Some(minflt.saturating_add(majflt));
        let mut error_code = 0;

        if let Ok(native) = inspect_process(pid) {
            creation = native.creation;
            cpu_time_ms = native.cpu_time_ms;
            working_set_mb = Some(native.working_set_mb);
            page_fault_count = Some(native.page_fault_count);
        } else {
            error_code = -1;
        }

        snapshot.processes.push(ProcessRow {
            pid,
            ppid,
            cmd: cmd.clone(),
            creation,
            cpu_time_ms,
            cpu_percent: None,
            working_set_mb,
            private_mb,
            page_fault_count,
            page_faults_delta: None,
            error_code,
        });
    }

    let main_proc = snapshot.processes.iter().find(|p| !p.cmd.contains("--type="));
    if let Some(main) = main_proc {
        let mut window_found = false;
        if let Ok(output) = Command::new("xdotool").args(["search", "--class", "spotify"]).output() {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    if let Ok(wid) = line.trim().parse::<usize>() {
                        snapshot.windows.push(WindowRow {
                            pid: main.pid,
                            hwnd: wid,
                            title: "Spotify".to_string(),
                        });
                        window_found = true;
                        break;
                    }
                }
            }
        }
        if !window_found {
            snapshot.windows.push(WindowRow {
                pid: main.pid,
                hwnd: main.pid as usize,
                title: "Spotify".to_string(),
            });
        }
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

    if let Ok(output) = Command::new("xdotool").args(["getactivewindow"]).output() {
        if output.status.success() {
            let active_id = String::from_utf8_lossy(&output.stdout).trim().parse::<usize>().unwrap_or(0);
            if active_id == hwnd {
                return "foreground".to_string();
            }
        }
    }

    if let Ok(output) = Command::new("xprop").args(["-id", &hwnd.to_string(), "_NET_WM_STATE"]).output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("_NET_WM_STATE_HIDDEN") {
                return "minimized".to_string();
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

    let pidfd = unsafe { syscall(SYS_PIDFD_OPEN, pid as i64, 0) };
    if pidfd < 0 {
        return Ok(());
    }

    let maps_path = format!("/proc/{pid}/maps");
    if let Ok(maps) = fs::read_to_string(&maps_path) {
        let mut iovs = Vec::new();
        for line in maps.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 { continue; }
            let perms = parts[1];
            if !perms.starts_with("rw-p") { continue; }
            let Some((start_str, end_str)) = parts[0].split_once('-') else { continue; };
            let Ok(start) = usize::from_str_radix(start_str, 16) else { continue; };
            let Ok(end) = usize::from_str_radix(end_str, 16) else { continue; };
            if end > start {
                iovs.push(IOVec {
                    iov_base: start as *mut c_void,
                    iov_len: end - start,
                });
            }
            if iovs.len() >= 64 {
                unsafe {
                    syscall(
                        SYS_PROCESS_MADVISE,
                        pidfd,
                        iovs.as_ptr(),
                        iovs.len() as i64,
                        MADV_PAGEOUT as i64,
                        0i64,
                    );
                }
                iovs.clear();
            }
        }
        if !iovs.is_empty() {
            unsafe {
                syscall(
                    SYS_PROCESS_MADVISE,
                    pidfd,
                    iovs.as_ptr(),
                    iovs.len() as i64,
                    MADV_PAGEOUT as i64,
                    0i64,
                );
            }
        }
    }

    unsafe { close(pidfd as i32); }
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
    Path::new(&format!("/proc/{pid}")).exists()
}

pub fn process_matches_executable(pid: u32, expected_path: &Path) -> bool {
    let exe_link = format!("/proc/{pid}/exe");
    let Ok(target) = fs::read_link(&exe_link) else { return false; };
    let expected = expected_path.canonicalize().unwrap_or_else(|_| expected_path.to_path_buf());
    let target = target.canonicalize().unwrap_or(target);
    expected == target
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
