//! Stub platform implementation for unsupported operating systems.

use std::io;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub const MIN_WORKING_SET_BYTES: usize = 80 * 1024;
pub const HARD_CAP_FLAGS: u32 = 0x6;

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

pub struct SingleInstanceGuard;

pub fn acquire_single_instance() -> Result<SingleInstanceGuard, String> {
    Ok(SingleInstanceGuard)
}

pub fn random_token() -> Result<String, String> {
    Ok("0000000000000000000000000000000000000000000000000000000000000000".to_string())
}

pub fn detect_supported_client_os() -> Result<(&'static str, u32), String> {
    Err("Unsupported operating system".to_string())
}

pub fn inspect_process(_pid: u32) -> Result<NativeStats, String> {
    Err("Unsupported platform".to_string())
}

pub fn process_snapshot() -> Result<Snapshot, String> {
    Ok(Snapshot::default())
}

pub fn powershell_snapshot() -> Result<Snapshot, String> {
    process_snapshot()
}

pub fn window_state(_hwnd: usize, _has_processes: bool) -> String {
    "not-running".to_string()
}

pub fn read_original_limits(_pid: u32, _expected_creation: u64) -> Result<OriginalLimits, String> {
    Err("Unsupported platform".to_string())
}

pub fn set_working_set_limits(
    _pid: u32,
    _expected_creation: u64,
    _minimum: usize,
    _maximum: usize,
    _flags: u32,
) -> Result<(), String> {
    Err("Unsupported platform".to_string())
}

pub fn empty_working_set(_pid: u32, _expected_creation: u64) -> Result<(), String> {
    Err("Unsupported platform".to_string())
}

pub fn restore_working_set_limits(
    _pid: u32,
    _expected_creation: u64,
    _minimum: usize,
    _maximum: usize,
    _flags: u32,
) -> Result<RestoreOutcome, String> {
    Err("Unsupported platform".to_string())
}

pub fn process_exists(_pid: u32) -> bool {
    false
}

pub fn process_matches_executable(_pid: u32, _expected_path: &Path) -> bool {
    false
}

pub fn current_process_id() -> u32 {
    std::process::id()
}

pub fn epoch_millis() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64
}

pub fn io_error(error: io::Error) -> String {
    error.to_string()
}
