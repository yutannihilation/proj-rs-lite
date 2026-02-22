#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJ_REPO_URL="https://github.com/OSGeo/PROJ.git"

if [[ $# -gt 1 ]]; then
  echo "usage: $0 [tag-or-commit]" >&2
  exit 1
fi

REF="${1:-master}"

WORK_DIR="$(mktemp -d)"
SRC_DIR="${WORK_DIR}/proj"
BUILD_DIR="${WORK_DIR}/build"
trap 'rm -rf "${WORK_DIR}"' EXIT

echo "Fetching PROJ (${REF}) from ${PROJ_REPO_URL}..."
git init "${SRC_DIR}" >/dev/null
git -C "${SRC_DIR}" remote add origin "${PROJ_REPO_URL}"
git -C "${SRC_DIR}" fetch --depth 1 origin "${REF}"
git -C "${SRC_DIR}" checkout --detach FETCH_HEAD >/dev/null

echo "Building official PROJ dist archive..."
cmake -S "${SRC_DIR}" -B "${BUILD_DIR}" -D BUILD_TESTING=OFF
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
