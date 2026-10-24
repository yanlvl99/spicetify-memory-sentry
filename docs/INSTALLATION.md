# Installation guide

**Supported systems: Windows 10/11 (x64, ARM64), Linux (x86_64, aarch64, arm), and macOS 11+ (Intel, Apple Silicon).**

**Spicetify is required by the current release.** Spotify Desktop and Spicetify CLI must be installed for the same user. The dashboard is a Spicetify Custom App, while the native Rust companion performs process-memory management.

## The only supported installation method on Windows

1. Open the official release page: https://github.com/yanlvl99/spicetify-memory-sentry/releases
2. Download **Memory Sentry Setup** and the `.sha256` checksum attached to that same release.
3. Verify the checksum. It confirms file integrity, not publisher identity.
4. Run Setup as the same Windows user that runs Spotify and Spicetify. Do not run as Administrator.
5. Setup checks the supported Windows edition and detects Spicetify before replacing installed files. You do not have to close Spotify manually.
6. Setup installs the native companion, writes its local settings/token, starts and health-checks the local API, then applies the Custom App through `spicetify apply`. Spicetify may restart Spotify automatically.
7. When Spotify returns, select Memory Sentry in the sidebar. The helper is also registered to start at login for the current user.

## Installation on Linux and macOS (x64 & ARM)

For Linux and macOS, install from source using the helper script:

1. Ensure `git`, `cargo` (Rust) and `spicetify` are installed.
2. Clone and run the automated installer:
   ```bash
   git clone https://github.com/yanlvl99/spicetify-memory-sentry.git
   cd spicetify-memory-sentry
   ./scripts/install.sh
   ```
3. The script compiles the companion, configures Spicetify, starts the daemon and configures autostart.

The **Marketplace entry is for discovery only** and points to this same official release page. It is not a second installation method: Marketplace cannot install the native companion, and there is no separate Marketplace build or custom portable ZIP. GitHub-generated source archives are for code review, not installation.

## Upgrade from the Node.js preview

The official Setup EXE is also the supported upgrade route. Before replacing a previous installation, it asks the old companion to restore recorded limits and preserves recovery data if restoration cannot be confirmed. If a previous Node-based version has an outstanding `working-set-backup.json`, Setup stops only the matching legacy helper and attempts its restore command. It may require Node.js 22+ temporarily for that migration. Never delete a recovery backup manually to force an upgrade.

## Uninstall

- **Windows:** Use Windows Settings → Apps → Installed apps → Memory Sentry → Uninstall. The native pre-uninstall command restores recorded working-set limits, removes the current-user startup entry and deregisters the Custom App before the files are removed. Spotify does not need to be closed manually; `spicetify apply` may refresh/restart it.
- **Linux/macOS:** Run `MemorySentry --pre-uninstall` to restore any limits, remove autostart entries and deregister the Custom App.

Logs are preserved in the user logs directory after uninstall.

## Security and unsupported systems

The installer is per-user and should not request administrator privileges. It does not disable antivirus, SmartScreen, the firewall or system security controls. Unsigned setup files may still show reputation warnings; verify the download source and SHA-256 instead of blindly bypassing warnings.

Unsupported: Windows Server, Windows 7/8/8.1, 32-bit legacy Windows.
