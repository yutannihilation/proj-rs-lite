// Browser shims for Emscripten libc syscall imports.
// Return negative errno values expected by the musl/Emscripten syscall layer.

const ERRNO_ENOSYS_NEG = -38;
const envCalls = {};

function record(name) {
  envCalls[name] = (envCalls[name] || 0) + 1;
}

export function __getEnvDebug() {
  return { ...envCalls };
}

export function __syscall_faccessat(_dirfd, _path, _mode, _flags) {
  record("__syscall_faccessat");
  return ERRNO_ENOSYS_NEG;
}

export function __syscall_chmod(_path, _mode) {
  record("__syscall_chmod");
  return ERRNO_ENOSYS_NEG;
}

export function __syscall_fchmod(_fd, _mode) {
  record("__syscall_fchmod");
  return ERRNO_ENOSYS_NEG;
}

export function __syscall_fchown32(_fd, _owner, _group) {
  record("__syscall_fchown32");
  return ERRNO_ENOSYS_NEG;
}

export function __syscall_ftruncate64(_fd, _length) {
  record("__syscall_ftruncate64");
  return ERRNO_ENOSYS_NEG;
}

export function __syscall_getcwd(_buf, _size) {
  record("__syscall_getcwd");
  return ERRNO_ENOSYS_NEG;
}

export function __syscall_readlinkat(_dirfd, _path, _buf, _bufsiz) {
  record("__syscall_readlinkat");
  return ERRNO_ENOSYS_NEG;
}

export function __syscall_rmdir(_path) {
  record("__syscall_rmdir");
  return ERRNO_ENOSYS_NEG;
}

export function __syscall_unlinkat(_dirfd, _path, _flags) {
  record("__syscall_unlinkat");
  return ERRNO_ENOSYS_NEG;
}

export function __syscall_utimensat(_dirfd, _path, _times, _flags) {
  record("__syscall_utimensat");
  return ERRNO_ENOSYS_NEG;
}
