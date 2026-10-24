#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VERSION="${1:-1.0.0-rc.1}"
OS_NAME="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH_NAME="$(uname -m)"

mkdir -p "${ROOT_DIR}/dist"
STAGE_DIR="${ROOT_DIR}/dist/stage-unix"
rm -rf "${STAGE_DIR}"
mkdir -p "${STAGE_DIR}"

BIN_SRC="${ROOT_DIR}/native/rust/target/release/memory-sentry-native"
if [ ! -f "${BIN_SRC}" ]; then
  cargo build --release --manifest-path "${ROOT_DIR}/native/rust/Cargo.toml"
fi

cp "${BIN_SRC}" "${STAGE_DIR}/MemorySentry"
chmod +x "${STAGE_DIR}/MemorySentry"

mkdir -p "${STAGE_DIR}/app"
cp "${ROOT_DIR}/app/index.js" "${STAGE_DIR}/app/index.js"
cp "${ROOT_DIR}/app/manifest.json" "${STAGE_DIR}/app/manifest.json"
cp "${ROOT_DIR}/scripts/install.sh" "${STAGE_DIR}/install.sh"
cp "${ROOT_DIR}/LICENSE" "${STAGE_DIR}/LICENSE"
cp "${ROOT_DIR}/README.md" "${STAGE_DIR}/README.md"

ARCHIVE_NAME="MemorySentry-${VERSION}-${OS_NAME}-${ARCH_NAME}.tar.gz"
tar -czf "${ROOT_DIR}/dist/${ARCHIVE_NAME}" -C "${STAGE_DIR}" .

if command -v sha256sum &>/dev/null; then
  (cd "${ROOT_DIR}/dist" && sha256sum "${ARCHIVE_NAME}" > "${ARCHIVE_NAME}.sha256")
elif command -v shasum &>/dev/null; then
  (cd "${ROOT_DIR}/dist" && shasum -a 256 "${ARCHIVE_NAME}" > "${ARCHIVE_NAME}.sha256")
fi

rm -rf "${STAGE_DIR}"
echo "Packaged dist/${ARCHIVE_NAME}"
