# Changelog

## Unreleased

- Add cross-platform support for Linux (x86_64, aarch64, arm) using procfs process tracking and `process_madvise` (`MADV_PAGEOUT`) memory trimming.
- Add cross-platform support for macOS (x86_64, Apple Silicon ARM64) with process monitoring and autostart LaunchAgent management.
- Add support for Windows on ARM64 (`aarch64-pc-windows-msvc`).
- Add unified Unix installation script (`scripts/install.sh`) for compiling and configuring Memory Sentry in one step.
- Update GitHub Actions validation matrix to test across Linux, macOS, and Windows.

- Clarify that the supported release requires Spotify Desktop and Spicetify CLI, distinguish the Spicetify Custom App from the native Rust companion, and add dynamic release/version/download/CI/license badges to the README.

- Convert the README's privacy, security, installation, troubleshooting and migration references into working relative Markdown links; fix the development guide's same-directory documentation links.

- Clarify that substantial resident-memory reduction while Spotify is minimized/hidden is the intended behavior across supported Windows 10/11 x64 desktop PCs, while actual savings vary by machine and workload.
- Document the approximately 40 MB initial and 100 MB settled combined Spotify working-set figures as observations from one setup, not guaranteed memory targets.
- Explain that a very low-end PC can briefly show a black window during restore without that alone proving the page reloaded; verify whether the previous UI state returns.
- Move the Task Manager comparison previews near the top of the README and expand restore testing to include playback, UI reload/state retention, black-screen recovery, and first-input latency.
- Keep the broader Windows 10/11 release-validation checklist open until that behavior has been tested across real desktop configurations.

## 1.0.0-rc.1 — Release candidate and installation consolidation

- Restore the Marketplace banner asset expected by package validation; keep all listing artwork derived from the supplied COM/SEM screenshots with restrained SVG sharpening only.
- Fix the screenshot disclaimer required by validation, and retain explicit references to the original COM.png and SEM.png captures.
- Consolidate supported distribution to one per-user Setup EXE and its SHA-256 checksum; remove the custom portable installer ZIP and redundant PowerShell/batch installation entry points.
- Make Marketplace discovery-only in the README, dashboard and installation docs; the same official GitHub Setup is the only supported install method.
- Align Cargo, runtime and Setup versions on 1.0.0-rc.1 and mark it as a prerelease pending the real-desktop Windows test matrix.

## 0.3.4 — Installer diagnostics and startup ordering (experimental)

- Remove the incorrect requirement to close Spotify manually; Spicetify apply handles its refresh/restart during install and uninstall.
- Start and health-check the native local API before applying the Custom App, preventing the dashboard's initial render from racing helper startup.
- Give missing Spicetify a dedicated preflight error and show native setup diagnostics in the Inno Setup failure dialog.
- Log configure failures locally to setup.log, guard maintenance PID waits by checking the executable path, and remove stale helper.pid when the API cannot bind.
- Retain recovery-first behavior and the experimental prerelease label.

## 0.3.3 — Early-launch diagnostics (experimental)

- Create `logs/startup.log` before reading settings, so missing or invalid configuration is diagnosed instead of exiting without a log.
- Record launcher invocation, settings-load errors, monitor spawn failures, readiness timeouts, and successful API health checks locally.
- Include the startup-log path when launcher initialization fails; keep the release experimental pending real Windows desktop validation.

## 0.3.2 — Connection diagnostics and non-blocking uninstall (experimental)

- Render the actual dashboard API error instead of only the generic disconnected-state copy.
- Clear the Win32 last-error value before creating the single-instance mutex to avoid false duplicate-instance detection.
- Allow uninstall while Spotify is open by restoring/stopping the companion first, removing startup registration, and then applying the Spicetify removal.
- Wait up to five seconds for a gracefully stopped companion before preserving recovery data and blocking unsafe file removal.

## 0.3.1 — Companion startup diagnostics (experimental)

- Make the installer launch the native companion automatically instead of relying on an optional post-install checkbox.
- Verify the authenticated local status API before the launcher reports success.
- Capture detached monitor startup errors in a local startup log.
- Show specific HTTP/API failure details in the dashboard instead of hiding them behind a generic connection message.
- Keep the release experimental until real Windows 10/11 audio, recovery, and restore-latency tests are complete.

## 0.3.0 — Native Windows companion (experimental)

- Add a native Rust companion and per-user Inno Setup installer for Windows 10/11 x64.
- Add guarded install, upgrade, recovery and uninstall flows that preserve recovery data if limits cannot be restored safely.
- Build a Windows Setup EXE and portable x64 ZIP with SHA-256 checksums in GitHub Actions.
- Keep adaptive per-process working-set policy, audio-service exclusions, loopback-only dashboard API and no-telemetry behavior.
- Align native runtime and Cargo lockfile versions with 0.3.0, and derive validation package versions from Cargo metadata.
- Publish as an experimental prerelease: the complete real-desktop Windows 10/11 behavior and restore-latency test matrix remains outstanding.

## 0.2.0 — Adaptive policy preview

- Replaced the user-selected fixed per-process cap with a role-based adaptive policy.
- Raised a process ceiling by 8 MB after sustained CPU usage at or above 10% of one logical processor for six seconds.
- Lowered a ceiling by 4 MB after sustained CPU usage at or below 3% and at least 6 MB of working-set headroom for ten seconds.
- Added hysteresis, per-role bounds, and an adjustment cooldown to prevent rapid oscillation.
- Added local CPU estimates and per-process adaptive ceilings to the dashboard.
- Added automatic English/Portuguese (Brazil) UI localization based on the system/browser language.
- Added a GitHub Releases action when the Windows companion cannot be reached.
- Switched the project license to GPL-3.0 to require redistributions of covered derivative works to retain the copyleft terms.
- Kept a 24 MB minimum ceiling for the main process while allowing low-activity background renderers to shrink to 12 MB, balancing savings with window restoration.
- Added adaptive policy unit tests and release checksum workflow.
- Prevented new limits from being applied using stale process snapshots after discovery failures, and throttled retries for inaccessible targets.
- Clarified the unsigned-installer/SmartScreen limitations and the reason for the separate native companion.

## 0.1.4

- Reduced window-state polling to 100 ms and moved recurring process discovery off the synchronous focus path.
- Added process-creation identity checks, page-fault metrics, and restore-operation timing.
