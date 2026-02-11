// Browser shims for Emscripten libc syscall imports.
// Return negative errno values expected by the musl/Emscripten syscall layer.

const ERRNO_ENOSYS_NEG = -38;

export function __syscall_faccessat(_dirfd, _path, _mode, _flags) {
  return ERRNO_ENOSYS_NEG;
}

export function __syscall_chmod(_path, _mode) {
  return ERRNO_ENOSYS_NEG;
}

export function __syscall_fchmod(_fd, _mode) {
  return ERRNO_ENOSYS_NEG;
}

export function __syscall_fchown32(_fd, _owner, _group) {
  return ERRNO_ENOSYS_NEG;
}

export function __syscall_ftruncate64(_fd, _length) {
  return ERRNO_ENOSYS_NEG;
}

export function __syscall_getcwd(_buf, _size) {
  return ERRNO_ENOSYS_NEG;
}

export function __syscall_readlinkat(_dirfd, _path, _buf, _bufsiz) {
  return ERRNO_ENOSYS_NEG;
}

export function __syscall_rmdir(_path) {
  return ERRNO_ENOSYS_NEG;
}

export function __syscall_unlinkat(_dirfd, _path, _flags) {
  return ERRNO_ENOSYS_NEG;
}

export function __syscall_utimensat(_dirfd, _path, _times, _flags) {
  return ERRNO_ENOSYS_NEG;
}
