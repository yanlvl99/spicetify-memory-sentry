//! Platform-specific process inspection, memory manipulation, and environment primitives.

#[cfg(windows)]
pub mod windows;
#[cfg(windows)]
pub use windows::*;

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "linux")]
pub use linux::*;

#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "macos")]
pub use macos::*;

#[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
pub mod stub;
#[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
pub use stub::*;
