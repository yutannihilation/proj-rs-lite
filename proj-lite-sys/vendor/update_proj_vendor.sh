#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SUBMODULE_DIR="${SCRIPT_DIR}/proj"

if [[ ! -f "${SUBMODULE_DIR}/CMakeLists.txt" ]]; then
  echo "error: missing submodule at ${SUBMODULE_DIR}" >&2
  echo "run: git submodule update --init --recursive" >&2
  exit 1
fi

BUILD_DIR="$(mktemp -d)"
trap 'rm -rf "${BUILD_DIR}"' EXIT

echo "Building official PROJ dist archive from submodule..."
cmake -S "${SUBMODULE_DIR}" -B "${BUILD_DIR}" -D BUILD_TESTING=OFF
cmake --build "${BUILD_DIR}" --target dist

ARCHIVE="$(ls -1 "${BUILD_DIR}"/proj-*.tar.gz | head -n 1 || true)"
if [[ -z "${ARCHIVE}" ]]; then
  echo "error: dist archive not produced under ${BUILD_DIR}" >&2
  exit 1
fi

ARCHIVE_BASENAME="$(basename "${ARCHIVE}")"
DEST_ARCHIVE="${SCRIPT_DIR}/${ARCHIVE_BASENAME}"

echo "Updating ${DEST_ARCHIVE} ..."
cp -f "${ARCHIVE}" "${DEST_ARCHIVE}"

echo "Done: ${DEST_ARCHIVE}"
