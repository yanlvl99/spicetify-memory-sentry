#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
echo "=== Memory Sentry: Multi-platform Installation (Linux / macOS / ARM) ==="

# Check Rust / Cargo
if ! command -v cargo &>/dev/null; then
  echo "Error: Cargo/Rust is not installed. Please install Rust (https://rustup.rs) first." >&2
  exit 1
fi

# Build native companion
echo "==> Building native companion..."
cargo build --release --manifest-path "${ROOT_DIR}/native/rust/Cargo.toml"

BINARY_SRC="${ROOT_DIR}/native/rust/target/release/memory-sentry-native"

# Determine target directory
OS="$(uname -s)"
ARCH="$(uname -m)"
echo "==> Detected OS: ${OS} (${ARCH})"

if [ "${OS}" = "Darwin" ]; then
  INSTALL_DIR="${HOME}/Library/Application Support/SpicetifyMemorySentry"
  SPICETIFY_CUSTOM_APPS="${HOME}/Library/Application Support/spicetify/CustomApps"
  if [ ! -d "${SPICETIFY_CUSTOM_APPS}" ]; then
    SPICETIFY_CUSTOM_APPS="${HOME}/.config/spicetify/CustomApps"
  fi
  if [ ! -d "${SPICETIFY_CUSTOM_APPS}" ]; then
    SPICETIFY_CUSTOM_APPS="${HOME}/spicetify_data/CustomApps"
  fi
else
  # Linux
  DATA_HOME="${XDG_DATA_HOME:-${HOME}/.local/share}"
  CONFIG_HOME="${XDG_CONFIG_HOME:-${HOME}/.config}"
  INSTALL_DIR="${DATA_HOME}/SpicetifyMemorySentry"
  SPICETIFY_CUSTOM_APPS="${CONFIG_HOME}/spicetify/CustomApps"
fi

echo "==> Installing companion to ${INSTALL_DIR}..."
mkdir -p "${INSTALL_DIR}" "${INSTALL_DIR}/logs"
cp -f "${BINARY_SRC}" "${INSTALL_DIR}/MemorySentry"
chmod +x "${INSTALL_DIR}/MemorySentry"

# Install Spicetify Custom App
TARGET_APP_DIR="${SPICETIFY_CUSTOM_APPS}/SpicetifyMemorySentry"
echo "==> Installing Custom App to ${TARGET_APP_DIR}..."
mkdir -p "${TARGET_APP_DIR}"
cp -f "${ROOT_DIR}/app/index.js" "${TARGET_APP_DIR}/index.js"
cp -f "${ROOT_DIR}/app/manifest.json" "${TARGET_APP_DIR}/manifest.json"

# Run configure
echo "==> Configuring Memory Sentry..."
"${INSTALL_DIR}/MemorySentry" --configure "${TARGET_APP_DIR}"

echo "=== Installation completed successfully! ==="
echo "Restart Spotify (or check Spicetify) to see Memory Sentry in the sidebar."
