# Memory Sentry for Spotify Desktop

[![Release](https://img.shields.io/github/v/release/yanlvl99/spicetify-memory-sentry?include_prereleases&sort=semver&label=release)](https://github.com/yanlvl99/spicetify-memory-sentry/releases) [![Version](https://img.shields.io/github/v/tag/yanlvl99/spicetify-memory-sentry?include_prereleases&sort=semver&label=version)](https://github.com/yanlvl99/spicetify-memory-sentry/tags) [![Downloads](https://img.shields.io/github/downloads/yanlvl99/spicetify-memory-sentry/total?label=all%20release%20downloads)](https://github.com/yanlvl99/spicetify-memory-sentry/releases) [![Validation](https://github.com/yanlvl99/spicetify-memory-sentry/actions/workflows/validate.yml/badge.svg?branch=main)](https://github.com/yanlvl99/spicetify-memory-sentry/actions/workflows/validate.yml) [![License](https://img.shields.io/github/license/yanlvl99/spicetify-memory-sentry?label=license)](LICENSE) ![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-blue) ![Architecture](https://img.shields.io/badge/arch-x64%20%7C%20ARM64-lightgrey) ![Status](https://img.shields.io/badge/status-experimental-orange) [![Requires Spicetify](https://img.shields.io/badge/requires-Spicetify-1DB954)](docs/INSTALLATION.md)

**Windows 10/11, Linux, and macOS (x64 and ARM). Requires Spotify Desktop and Spicetify CLI.** Memory Sentry is an experimental open-source companion. When Spotify is minimized or hidden, its native companion adaptively changes eligible child-process working-set ceilings, protects audio-service processes, and attempts to restore original limits when Spotify becomes visible.

## Spicetify is required by the current release

The supported release is **not a standalone Spotify app**. It requires Spotify Desktop and Spicetify CLI installed for the same Windows user. The interface runs as a Spicetify Custom App; a separate native Rust companion performs the Windows process-memory operations. A standalone interface could be added as a future adaptation, but a no-Spicetify installation mode is **not implemented or supported today**. For the supported setup, use the official installer and keep Spicetify installed.

## Preview: Task Manager with and without Memory Sentry

<table>
  <thead>
    <tr>
      <th width="50%">With Memory Sentry</th>
      <th width="50%">Without Memory Sentry</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td align="center"><img src="assets/COM-sharp.svg" alt="Task Manager capture with Memory Sentry enabled" width="100%" /></td>
      <td align="center"><img src="assets/SEM-sharp.svg" alt="Task Manager capture without Memory Sentry" width="100%" /></td>
    </tr>
  </tbody>
</table>

*These previews use the supplied `COM.png` and `SEM.png` captures with restrained SVG sharpening; the original PNG files remain unchanged. This is an illustrative comparison, not a controlled benchmark or a promise of the same RAM reduction on another PC.*

## Observed memory and restore behavior

Memory Sentry is designed to work across supported Windows 10/11 x64 desktop PCs and to **substantially reduce Spotify's resident working set while the window is minimized or hidden**. The amount varies by PC, Spotify workload, process activity, and memory conditions; it is not a fixed RAM cap or a guarantee of identical numbers. In one measured setup, the combined Spotify working set fell to about **40 MB during the initial trim** and later settled near **100 MB**. These are observed resident-working-set figures, not total committed memory. Summed working sets can double-count shared pages.

When Spotify is restored, the intended behavior is to return to the previous interface state without deliberately reloading the page, while playback continues. On very low-end PCs, the window can sometimes appear black temporarily while the interface becomes visible again. **A temporary black screen does not by itself prove that the page reloaded**: if the previous page and state return as they were, the UI state was retained. Keyboard or mouse input can also take a few seconds to respond. Persistent black screens, crashes, playback interruption, or state loss should still be reported and tested. The Windows 10/11 compatibility and recovery test matrix remains open.

> **1.0.0-rc.1 release candidate — still experimental.** On supported Windows 10/11 x64 desktop PCs, optimization is intended to cut Spotify's resident memory substantially while minimized or hidden. The amount and restore behavior vary by machine. Very low-end PCs may briefly show a black window during restore; this does not necessarily mean the page reloaded if it returns to the previous state. The full Windows 10/11 release test matrix remains open, so this is not a stable-release claim.

## The only supported installation method

1. Open the [official GitHub Releases page](https://github.com/yanlvl99/spicetify-memory-sentry/releases).
2. Download **Memory Sentry Setup** and its `.sha256` checksum from the same release.
3. Verify the checksum before running Setup. A checksum confirms file integrity, not the publisher's identity.
4. Run Setup as your normal Windows user, not as Administrator. It checks Windows and Spicetify before changing installed files; you do not have to close Spotify manually.
5. Setup installs the native companion, writes local settings/token, starts and health-checks the local API, then applies the Custom App through Spicetify. Spicetify may restart Spotify automatically.
6. When Spotify returns, select Memory Sentry in the sidebar. The helper is also registered to start at login for the current Windows user.

### Installation on Linux and macOS (x64 and ARM64)

Clone the repository and run the automated installer:
```bash
git clone https://github.com/yanlvl99/spicetify-memory-sentry.git
cd spicetify-memory-sentry
./scripts/install.sh
```

The Marketplace entry is for **discovery only** and points to the same official GitHub release. There is no separate Marketplace installer, Marketplace-specific build, or custom portable ZIP. GitHub's automatically generated source archive is for inspecting the source and is not an installation package.

The Inno Setup installer is per-user and does not require a machine-wide MSI installation. The executable is not code-signed, so Windows SmartScreen or antivirus may still show a reputation warning. Do not bypass security warnings blindly.

## What the adaptive policy does

The current policy keeps an individual working-set ceiling for each eligible Spotify process:

- Starting ceilings: main process 36 MB; GPU and renderer 28 MB; utility/helper 24 MB; crash handler 16 MB.
- If one process uses at least 10% of one logical processor for six seconds, its ceiling can increase by 8 MB.
- If CPU remains at or below 3% and the working set stays at least 6 MB below its ceiling for ten seconds, the ceiling can decrease by 4 MB.
- Changes have an eight-second cooldown. The global policy range is 12–64 MB, with a 24 MB floor for the main process and lower floors for background processes.
- Audio-service processes are excluded. The policy runs after the configured hide/minimize delay. The helper saves original Windows limits before changing them and attempts to restore them when Spotify becomes visible.

CPU is estimated from process-time deltas and expressed as a percentage of one logical processor, capped at 100% per process. High CPU is only a heuristic; it does not prove that a process needs more memory. Working-set MB is resident RAM, not private committed memory, and summed working sets can double-count shared pages.

## Requirements

- Windows 10/11 (x64, ARM64), Linux (x86_64, aarch64, arm), or macOS 11+ (Intel, Apple Silicon).
- Spotify Desktop and Spicetify CLI installed for the same user.
- The official setup program contains the Rust native companion. A global Node.js installation is not required for new installations.
- When migrating a legacy Node.js installation with an outstanding recovery file, setup may request Node.js 22+ temporarily to run the old helper's restore command. If legacy recovery cannot be verified, setup stops and preserves the backup instead of deleting it.

## Installed components and safety

- Custom App: %APPDATA%\spicetify\CustomApps\SpicetifyMemorySentry
- Native companion, settings and recovery state: %LOCALAPPDATA%\SpicetifyMemorySentry
- User startup registration: HKCU\Software\Microsoft\Windows\CurrentVersion\Run
- Logs: %LOCALAPPDATA%\MemorySentry\logs after uninstall

The companion runs as a separate per-user process under the current Windows user, exposes its dashboard API only on 127.0.0.1, requires a random per-install token, validates the Spotify HTTPS origin and loopback Host header, and does not upload telemetry. The installer launches it automatically and verifies the local status API. Early startup failures are recorded in %LOCALAPPDATA%\\SpicetifyMemorySentry\\logs\\startup.log. Dashboard failures now include the underlying local API error, and uninstall attempts safe recovery without requiring Spotify to be closed. It checks the process creation timestamp before changing or restoring limits, persists recovery data before applying a cap, and stops upgrade/uninstall if a limit cannot be safely restored.

## Upgrade and uninstall

Setup and uninstall do not require you to close Spotify manually; Spicetify apply manages its own refresh/restart. Setup attempts to restore the old helper's limits before replacing it and preserves recovery data if restoration cannot be confirmed. Windows Settings → Apps → Installed apps → Memory Sentry → Uninstall runs a pre-uninstall recovery check. If recovery fails or recovery records exist while the executable is missing, uninstallation stops and preserves data. Do not manually delete recovery files to bypass an error.

## Build from source

Install the stable Rust toolchain and Inno Setup 6. From the repository root, run cargo test with manifest path native/rust/Cargo.toml and --locked, then cargo build --release for target x86_64-pc-windows-msvc. A complete setup also needs the Custom App and docs staged as described in installer/inno/MemorySentry.iss. GitHub Actions builds the single supported Setup EXE and its SHA-256 checksum. GitHub's automatically generated source archive is for source review, not an alternate installer. CI compilation is not the same as an actual Spotify/audio/restore test on a Windows 10/11 desktop.

## Open source, license and antivirus notices

Source and release packaging are public under GPL-3.0. The helper emits no remote telemetry and does not read playlists, track names, listening history or Spotify credentials. The installer does not disable Defender, SmartScreen, the firewall or execution-policy controls.

**No unsigned release can guarantee zero SmartScreen or antivirus warnings.** A new Rust executable can still trigger warnings due to publisher/file reputation. Do not bypass security warnings blindly. Verify the download source and SHA-256, inspect the open source, and report suspected false positives with the specific file hash.

See [PRIVACY.md](PRIVACY.md), [SECURITY.md](SECURITY.md), [docs/INSTALLATION.md](docs/INSTALLATION.md), [docs/TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md) and [docs/RUST-MIGRATION.md](docs/RUST-MIGRATION.md).
