//! Win32 process, memory, window-state and installation primitives.
//! All operations validate the process creation timestamp immediately before mutation.

use std::ffi::c_void;
use std::io;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

pub const PROCESS_QUERY_INFORMATION: u32 = 0x0400;
pub const PROCESS_SET_QUOTA: u32 = 0x0100;
pub const QUOTA_LIMITS_HARDWS_MIN_DISABLE: u32 = 0x0000_0002;
pub const QUOTA_LIMITS_HARDWS_MAX_ENABLE: u32 = 0x0000_0004;
pub const HARD_CAP_FLAGS: u32 = QUOTA_LIMITS_HARDWS_MIN_DISABLE | QUOTA_LIMITS_HARDWS_MAX_ENABLE;
pub const MIN_WORKING_SET_BYTES: usize = 80 * 1024;

type Handle = *mut c_void;

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct FileTime {
    low: u32,
    high: u32,
}

impl FileTime {
    fn value(self) -> u64 {
        ((self.high as u64) << 32) | self.low as u64
    }
}

#[repr(C)]
struct ProcessMemoryCounters {
    cb: u32,
    page_fault_count: u32,
    peak_working_set_size: usize,
    working_set_size: usize,
    quota_peak_paged_pool_usage: usize,
    quota_paged_pool_usage: usize,
    quota_peak_non_paged_pool_usage: usize,
    quota_non_paged_pool_usage: usize,
    pagefile_usage: usize,
    peak_pagefile_usage: usize,
}

#[repr(C)]
struct RtlOsVersionInfoExW {
    size: u32,
    major: u32,
    minor: u32,
    build: u32,
    platform_id: u32,
    service_pack: [u16; 128],
    service_pack_major: u16,
    service_pack_minor: u16,
    suite_mask: u16,
    product_type: u8,
    reserved: u8,
}

#[link(name = "kernel32")]
extern "system" {
    fn OpenProcess(access: u32, inherit: i32, process_id: u32) -> Handle;
    fn CloseHandle(handle: Handle) -> i32;
    fn GetLastError() -> u32;
    fn SetLastError(error: u32);
    fn GetProcessTimes(
        process: Handle,
        creation: *mut FileTime,
        exit: *mut FileTime,
        kernel: *mut FileTime,
        user: *mut FileTime,
    ) -> i32;
    fn GetProcessWorkingSetSizeEx(
        process: Handle,
        minimum: *mut usize,
        maximum: *mut usize,
        flags: *mut u32,
    ) -> i32;
    fn SetProcessWorkingSetSizeEx(process: Handle, minimum: usize, maximum: usize, flags: u32) -> i32;
    fn GetCurrentProcessId() -> u32;
    fn CreateMutexW(attributes: *mut c_void, initial_owner: i32, name: *const u16) -> Handle;
    fn ReleaseMutex(handle: Handle) -> i32;
    fn GetExitCodeProcess(process: Handle, exit_code: *mut u32) -> i32;
    fn QueryFullProcessImageNameW(process: Handle, flags: u32, exe_name: *mut u16, size: *mut u32) -> i32;
}

#[link(name = "psapi")]
extern "system" {
    fn GetProcessMemoryInfo(process: Handle, counters: *mut ProcessMemoryCounters, size: u32) -> i32;
    fn EmptyWorkingSet(process: Handle) -> i32;
}

#[link(name = "user32")]
extern "system" {
    fn IsWindowVisible(hwnd: Handle) -> i32;
    fn IsIconic(hwnd: Handle) -> i32;
    fn GetForegroundWindow() -> Handle;
}

#[link(name = "bcrypt")]
extern "system" {
    fn BCryptGenRandom(algorithm: Handle, buffer: *mut u8, length: u32, flags: u32) -> i32;
}

#[link(name = "ntdll")]
extern "system" {
    fn RtlGetVersion(info: *mut RtlOsVersionInfoExW) -> i32;
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

fn last_error() -> u32 {
    unsafe { GetLastError() }
}

fn open_process(pid: u32, access: u32) -> Result<Handle, String> {
    let handle = unsafe { OpenProcess(access, 0, pid) };
    if handle.is_null() {
        return Err(format!("OpenProcess({pid}) failed with Win32 error {}", last_error()));
    }
    Ok(handle)
}

fn process_times(handle: Handle) -> Result<(u64, f64), String> {
    let mut creation = FileTime::default();
    let mut exit = FileTime::default();
    let mut kernel = FileTime::default();
    let mut user = FileTime::default();
    let ok = unsafe {
        GetProcessTimes(handle, &mut creation, &mut exit, &mut kernel, &mut user)
    };
    if ok == 0 {
        return Err(format!("GetProcessTimes failed with Win32 error {}", last_error()));
    }
    let cpu_ticks = kernel.value().saturating_add(user.value());
    Ok((creation.value(), cpu_ticks as f64 / 10_000.0))
}

fn process_stats_with_handle(handle: Handle) -> Result<(f64, u64), String> {
    let mut counters = ProcessMemoryCounters {
        cb: std::mem::size_of::<ProcessMemoryCounters>() as u32,
        page_fault_count: 0,
        peak_working_set_size: 0,
        working_set_size: 0,
        quota_peak_paged_pool_usage: 0,
        quota_paged_pool_usage: 0,
        quota_peak_non_paged_pool_usage: 0,
        quota_non_paged_pool_usage: 0,
        pagefile_usage: 0,
        peak_pagefile_usage: 0,
    };
    let ok = unsafe {
        GetProcessMemoryInfo(
            handle,
            &mut counters,
            std::mem::size_of::<ProcessMemoryCounters>() as u32,
        )
    };
    if ok == 0 {
        return Err(format!("GetProcessMemoryInfo failed with Win32 error {}", last_error()));
    }
    Ok((
        counters.working_set_size as f64 / (1024.0 * 1024.0),
        counters.page_fault_count as u64,
    ))
}

pub fn inspect_process(pid: u32) -> Result<NativeStats, String> {
    let handle = open_process(pid, PROCESS_QUERY_INFORMATION)?;
    let result = (|| {
        let (creation, cpu_time_ms) = process_times(handle)?;
        let (working_set_mb, page_fault_count) = process_stats_with_handle(handle)?;
        Ok(NativeStats { creation, cpu_time_ms, working_set_mb, page_fault_count })
    })();
    unsafe { CloseHandle(handle); }
    result
}

pub fn read_original_limits(pid: u32, expected_creation: u64) -> Result<OriginalLimits, String> {
    let handle = open_process(pid, PROCESS_QUERY_INFORMATION | PROCESS_SET_QUOTA)?;
    let result = (|| {
        let (creation, _) = process_times(handle)?;
        if creation != expected_creation {
            return Err("Process identity changed before reading working-set limits.".to_string());
        }
        let mut minimum = 0usize;
        let mut maximum = 0usize;
        let mut flags = 0u32;
        let ok = unsafe {
            GetProcessWorkingSetSizeEx(handle, &mut minimum, &mut maximum, &mut flags)
        };
        if ok == 0 {
            return Err(format!("GetProcessWorkingSetSizeEx failed with Win32 error {}", last_error()));
        }
        Ok(OriginalLimits { minimum, maximum, flags })
    })();
    unsafe { CloseHandle(handle); }
    result
}

pub fn set_working_set_limits(
    pid: u32,
    expected_creation: u64,
    minimum: usize,
    maximum: usize,
    flags: u32,
) -> Result<(), String> {
    let handle = open_process(pid, PROCESS_QUERY_INFORMATION | PROCESS_SET_QUOTA)?;
    let result = (|| {
        let (creation, _) = process_times(handle)?;
        if creation != expected_creation {
            return Err("Process identity changed before setting working-set limits; skipped.".to_string());
        }
        let ok = unsafe { SetProcessWorkingSetSizeEx(handle, minimum, maximum, flags) };
        if ok == 0 {
            return Err(format!("SetProcessWorkingSetSizeEx failed with Win32 error {}", last_error()));
        }
        Ok(())
    })();
    unsafe { CloseHandle(handle); }
    result
}

pub fn empty_working_set(pid: u32, expected_creation: u64) -> Result<(), String> {
    let handle = open_process(pid, PROCESS_QUERY_INFORMATION | PROCESS_SET_QUOTA)?;
    let result = (|| {
        let (creation, _) = process_times(handle)?;
        if creation != expected_creation {
            return Err("Process identity changed before trimming; skipped.".to_string());
        }
        let ok = unsafe { EmptyWorkingSet(handle) };
        if ok == 0 {
            return Err(format!("EmptyWorkingSet failed with Win32 error {}", last_error()));
        }
        Ok(())
    })();
    unsafe { CloseHandle(handle); }
    result
}

pub fn restore_working_set_limits(
    pid: u32,
    expected_creation: u64,
    minimum: usize,
    maximum: usize,
    flags: u32,
) -> Result<RestoreOutcome, String> {
    let handle = match open_process(pid, PROCESS_QUERY_INFORMATION | PROCESS_SET_QUOTA) {
        Ok(handle) => handle,
        Err(error) if error.contains("error 87") || error.contains("error 1168") => {
            return Ok(RestoreOutcome::ProcessGone);
        }
        Err(error) => return Err(error),
    };
    let result = (|| {
        let (creation, _) = process_times(handle)?;
        if creation != expected_creation {
            return Ok(RestoreOutcome::PidReused);
        }
        let ok = unsafe { SetProcessWorkingSetSizeEx(handle, minimum, maximum, flags) };
        if ok == 0 {
            return Err(format!("Could not restore PID {pid}: Win32 error {}", last_error()));
        }
        Ok(RestoreOutcome::Restored)
    })();
    unsafe { CloseHandle(handle); }
    result
}

pub fn window_state(hwnd: usize, has_processes: bool) -> String {
    if hwnd == 0 {
        return if has_processes { "window-not-found" } else { "not-running" }.to_string();
    }
    let window = hwnd as Handle;
    if unsafe { IsIconic(window) } != 0 {
        return "minimized".to_string();
    }
    if unsafe { IsWindowVisible(window) } == 0 {
        return "hidden".to_string();
    }
    if unsafe { GetForegroundWindow() } == window {
        "foreground".to_string()
    } else {
        "background-visible".to_string()
    }
}

pub fn random_token() -> Result<String, String> {
    let mut bytes = [0u8; 32];
    let status = unsafe {
        BCryptGenRandom(std::ptr::null_mut(), bytes.as_mut_ptr(), bytes.len() as u32, 0x0000_0002)
    };
    if status != 0 {
        return Err(format!("BCryptGenRandom failed with NTSTATUS 0x{:08X}", status as u32));
    }
    Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
}

pub fn detect_supported_client_os() -> Result<(&'static str, u32), String> {
    let mut info = RtlOsVersionInfoExW {
        size: std::mem::size_of::<RtlOsVersionInfoExW>() as u32,
        major: 0,
        minor: 0,
        build: 0,
        platform_id: 0,
        service_pack: [0; 128],
        service_pack_major: 0,
        service_pack_minor: 0,
        suite_mask: 0,
        product_type: 0,
        reserved: 0,
    };
    let status = unsafe { RtlGetVersion(&mut info) };
    if status != 0 {
        return Err(format!("RtlGetVersion returned NTSTATUS 0x{:08X}", status as u32));
    }
    if info.product_type != 1 {
        return Err("Windows Server editions are not supported".to_string());
    }
    if info.major != 10 || info.minor != 0 || info.build < 10_240 {
        return Err("unsupported Windows version".to_string());
    }
    if info.build >= 22_000 {
        Ok(("Windows 11", info.build))
    } else {
        Ok(("Windows 10", info.build))
    }
}

pub fn powershell_snapshot() -> Result<Snapshot, String> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let script = r#"
$ErrorActionPreference = 'Stop'
try {
  [Console]::OutputEncoding = New-Object System.Text.UTF8Encoding($false)
  $utf8 = New-Object System.Text.UTF8Encoding($false)
  $culture = [Globalization.CultureInfo]::InvariantCulture
  foreach ($p in @(Get-CimInstance Win32_Process -Filter "Name='Spotify.exe'")) {
    $created = '0'
    try { if ($p.CreationDate) { $created = [string](([datetime]$p.CreationDate).ToFileTimeUtc()) } } catch {}
    $cpuMs = 0.0
    try { $cpuMs = ([double]$p.KernelModeTime + [double]$p.UserModeTime) / 10000.0 } catch {}
    $cmd64 = [Convert]::ToBase64String($utf8.GetBytes([string]$p.CommandLine))
    $ws = 0.0; $private = 0.0; $faults = 0.0
    try { $ws = [double]$p.WorkingSetSize } catch {}
    try { $private = [double]$p.PrivatePageCount } catch {}
    try { $faults = [double]$p.PageFaults } catch {}
    [Console]::Out.WriteLine(('P|{0}|{1}|{2}|{3}|{4}|{5}|{6}|{7}' -f
      $p.ProcessId, $p.ParentProcessId, $created,
      $cpuMs.ToString($culture), $ws.ToString($culture),
      $private.ToString($culture), $faults.ToString($culture), $cmd64))
  }
  foreach ($w in @(Get-Process -Name Spotify -ErrorAction SilentlyContinue)) {
    $hwnd = 0
    try { $hwnd = [long]$w.MainWindowHandle.ToInt64() } catch {}
    if ($hwnd -ne 0) {
      $title64 = [Convert]::ToBase64String($utf8.GetBytes([string]$w.MainWindowTitle))
      [Console]::Out.WriteLine(('W|{0}|{1}|{2}' -f $w.Id, $hwnd, $title64))
    }
  }
} catch {
  [Console]::Error.WriteLine($_.Exception.Message)
  exit 2
}
"#;
    let output = Command::new("powershell.exe")
        .args(["-NoLogo", "-NoProfile", "-NonInteractive", "-Command", script])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|error| format!("Could not launch Windows PowerShell: {error}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Process discovery failed: {}", stderr.trim()));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut snapshot = Snapshot::default();
    for raw in stdout.lines() {
        let line = raw.trim_start_matches('\u{feff}').trim();
        let parts: Vec<&str> = line.split('|').collect();
        match parts.first().copied() {
            Some("P") if parts.len() == 9 => {
                let pid = parts[1].parse::<u32>().unwrap_or(0);
                let ppid = parts[2].parse::<u32>().unwrap_or(0);
                if pid == 0 { continue; }
                let creation_from_wmi = parts[3].parse::<u64>().unwrap_or(0);
                let cpu_from_wmi = parts[4].parse::<f64>().unwrap_or(0.0);
                let ws = parts[5].parse::<f64>().unwrap_or(0.0);
                let private = parts[6].parse::<f64>().unwrap_or(0.0);
                let faults = parts[7].parse::<f64>().unwrap_or(0.0);
                let cmd = String::from_utf8_lossy(&decode_base64(parts[8])).into_owned();
                let mut creation = creation_from_wmi;
                let mut cpu_time_ms = cpu_from_wmi;
                let mut working_set_mb = (ws > 0.0).then_some(ws / (1024.0 * 1024.0));
                let mut page_fault_count = (parts[7].parse::<u64>().is_ok()).then_some(faults as u64);
                let mut error_code = 0;
                if let Ok(native) = inspect_process(pid) {
                    // The native timestamp is the identity later revalidated before every mutation.
                    creation = native.creation;
                    cpu_time_ms = native.cpu_time_ms;
                    working_set_mb = Some(native.working_set_mb);
                    page_fault_count = Some(native.page_fault_count);
                } else {
                    error_code = -1;
                }
                snapshot.processes.push(ProcessRow {
                    pid, ppid, cmd, creation, cpu_time_ms, cpu_percent: None,
                    working_set_mb, private_mb: (private > 0.0).then_some(private / (1024.0 * 1024.0)),
                    page_fault_count, page_faults_delta: None, error_code,
                });
            }
            Some("W") if parts.len() == 4 => {
                let pid = parts[1].parse::<u32>().unwrap_or(0);
                let hwnd = parts[2].parse::<usize>().unwrap_or(0);
                let title = String::from_utf8_lossy(&decode_base64(parts[3])).into_owned();
                if pid > 0 && hwnd != 0 {
                    snapshot.windows.push(WindowRow { pid, hwnd, title });
                }
            }
            _ => {}
        }
    }
    Ok(snapshot)
}

fn decode_base64(input: &str) -> Vec<u8> {
    let mut output = Vec::with_capacity(input.len() * 3 / 4);
    let mut accumulator = 0u32;
    let mut bits = 0u32;
    for byte in input.bytes() {
        let value = match byte {
            b'A'..=b'Z' => (byte - b'A') as u32,
            b'a'..=b'z' => (byte - b'a' + 26) as u32,
            b'0'..=b'9' => (byte - b'0' + 52) as u32,
            b'+' => 62,
            b'/' => 63,
            b'=' => break,
            _ => continue,
        };
        accumulator = (accumulator << 6) | value;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            output.push(((accumulator >> bits) & 0xff) as u8);
        }
    }
    output
}

pub fn process_exists(pid: u32) -> bool {
    let Ok(handle) = open_process(pid, 0x1000) else { return false; };
    let mut code = 0u32;
    let alive = unsafe { GetExitCodeProcess(handle, &mut code) } != 0 && code == 259;
    unsafe { CloseHandle(handle); }
    alive
}

pub fn process_matches_executable(pid: u32, expected_path: &Path) -> bool {
    let Ok(handle) = open_process(pid, 0x1000) else { return false; };
    let mut buffer = vec![0u16; 32_768];
    let mut length = buffer.len() as u32;
    let queried = unsafe {
        QueryFullProcessImageNameW(handle, 0, buffer.as_mut_ptr(), &mut length)
    } != 0;
    unsafe { CloseHandle(handle); }
    if !queried || length == 0 || length as usize > buffer.len() { return false; }

    let actual = String::from_utf16_lossy(&buffer[..length as usize]);
    let expected = expected_path.canonicalize().unwrap_or_else(|_| expected_path.to_path_buf());
    let normalize = |value: &str| value.replace('/', "\\").trim_start_matches(r"\\?\").to_ascii_lowercase();
    normalize(&actual) == normalize(&expected.to_string_lossy())
}

pub struct SingleInstanceGuard {
    handle: Handle,
}

impl Drop for SingleInstanceGuard {
    fn drop(&mut self) {
        unsafe {
            ReleaseMutex(self.handle);
            CloseHandle(self.handle);
        }
    }
}

pub fn acquire_single_instance() -> Result<SingleInstanceGuard, String> {
    let name: Vec<u16> = "Local\\MemorySentry-Companion".encode_utf16().chain(std::iter::once(0)).collect();
    unsafe { SetLastError(0); }
    let handle = unsafe { CreateMutexW(std::ptr::null_mut(), 1, name.as_ptr()) };
    if handle.is_null() {
        return Err(format!("Could not create single-instance mutex (Win32 error {}).", last_error()));
    }
    if last_error() == 183 {
        unsafe { CloseHandle(handle); }
        return Err("Memory Sentry is already running for this Windows session.".to_string());
    }
    Ok(SingleInstanceGuard { handle })
}

pub fn current_process_id() -> u32 {
    unsafe { GetCurrentProcessId() }
}

pub fn epoch_millis() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64
}

pub fn process_snapshot() -> Result<Snapshot, String> {
    powershell_snapshot()
}

pub fn io_error(error: io::Error) -> String {
    error.to_string()
}
