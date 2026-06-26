#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJ_REPO_URL="https://github.com/OSGeo/PROJ.git"

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <version|commit-hash>" >&2
  echo "  version:     e.g. 9.8.1  (downloads release archive)" >&2
  echo "  commit-hash: e.g. abc123 (builds dist from git checkout)" >&2
  exit 1
fi

ARG="$1"

WORK_DIR="$(mktemp -d)"
trap 'rm -rf "${WORK_DIR}"' EXIT

# Detect whether the argument is a version (digits and dots) or a commit hash (hex).
if [[ "${ARG}" =~ ^[0-9]+\.[0-9]+(\.[0-9]+)?$ ]]; then
  # --- Version: download the official release archive ---
  VERSION="${ARG}"
  ARCHIVE_URL="https://github.com/OSGeo/PROJ/releases/download/${VERSION}/proj-${VERSION}.tar.gz"
  ARCHIVE="${WORK_DIR}/proj-${VERSION}.tar.gz"

  echo "Downloading PROJ ${VERSION} release archive..."
  echo "  ${ARCHIVE_URL}"
  curl -fSL -o "${ARCHIVE}" "${ARCHIVE_URL}"

elif [[ "${ARG}" =~ ^[0-9a-fA-F]{7,40}$ ]]; then
  # --- Commit hash: shallow-clone and build dist ---
  COMMIT="${ARG}"
  SRC_DIR="${WORK_DIR}/proj"
  BUILD_DIR="${WORK_DIR}/build"

  echo "Fetching PROJ commit ${COMMIT} from ${PROJ_REPO_URL}..."
  git init "${SRC_DIR}" >/dev/null
  git -C "${SRC_DIR}" remote add origin "${PROJ_REPO_URL}"
  git -C "${SRC_DIR}" fetch --depth 1 origin "${COMMIT}"
  git -C "${SRC_DIR}" checkout --detach FETCH_HEAD >/dev/null

  echo "Building official PROJ dist archive..."
  cmake -S "${SRC_DIR}" -B "${BUILD_DIR}" -D BUILD_TESTING=OFF
  cmake --build "${BUILD_DIR}" --target dist

  ARCHIVE="$(ls -1 "${BUILD_DIR}"/proj-*.tar.gz | head -n 1 || true)"
  if [[ -z "${ARCHIVE}" ]]; then
    echo "error: dist archive not produced under ${BUILD_DIR}" >&2
    exit 1
  fi

else
  echo "error: argument must be a version (e.g. 9.8.1) or a commit hash (hex, 7-40 chars)" >&2
  exit 1
fi

ARCHIVE_BASENAME="$(basename "${ARCHIVE}")"
DEST_ARCHIVE="${SCRIPT_DIR}/${ARCHIVE_BASENAME}"

echo "Updating ${DEST_ARCHIVE} ..."
cp -f "${ARCHIVE}" "${DEST_ARCHIVE}"

echo "Done: ${DEST_ARCHIVE}"
