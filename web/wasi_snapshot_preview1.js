// Browser shims for WASI imports used by the wasm module.
// We intentionally return ENOSYS for unsupported host features in browser.

const WASI_ERRNO_ENOSYS = 52;
let wasmMemory = null;
const wasiCalls = {};
let lastFdWrite = "";
const decoder = new TextDecoder();

function record(name) {
  wasiCalls[name] = (wasiCalls[name] || 0) + 1;
}

export function __setWasiMemory(memory) {
  wasmMemory = memory;
}

export function __getWasiDebug() {
  return { ...wasiCalls, last_fd_write: lastFdWrite };
}

function dataView() {
  if (!(wasmMemory instanceof WebAssembly.Memory)) return null;
  return new DataView(wasmMemory.buffer);
}

export function proc_exit(code) {
  record("proc_exit");
  throw new Error(`WASI proc_exit(${code})`);
}

export function environ_sizes_get(environ_count_ptr, environ_buf_size_ptr) {
  record("environ_sizes_get");
  const dv = dataView();
  if (!dv) return WASI_ERRNO_ENOSYS;
  dv.setUint32(environ_count_ptr, 0, true);
  dv.setUint32(environ_buf_size_ptr, 0, true);
  return 0;
}

export function environ_get(_environ_ptr, _environ_buf_ptr) {
  record("environ_get");
  return 0;
}

export function fd_close(_fd) {
  record("fd_close");
  return 0;
}

export function clock_time_get(_clock_id, _precision, time_ptr) {
  record("clock_time_get");
  const dv = dataView();
  if (!dv) return WASI_ERRNO_ENOSYS;
  const nowNs = BigInt(Date.now()) * 1000000n;
  dv.setBigUint64(time_ptr, nowNs, true);
  return 0;
}

export function fd_sync(_fd) {
  record("fd_sync");
  return WASI_ERRNO_ENOSYS;
}

export function fd_seek(_fd, _offset, _whence, new_offset_ptr) {
  record("fd_seek");
  const dv = dataView();
  if (!dv) return WASI_ERRNO_ENOSYS;
  dv.setBigUint64(new_offset_ptr, 0n, true);
  return 0;
}

export function fd_read(_fd, _iovs, _iovs_len, nread_ptr) {
  record("fd_read");
  const dv = dataView();
  if (!dv) return WASI_ERRNO_ENOSYS;
  dv.setUint32(nread_ptr, 0, true);
  return 0;
}

export function fd_write(_fd, iovs, iovs_len, nwritten_ptr) {
  record("fd_write");
  const dv = dataView();
  if (!dv) return WASI_ERRNO_ENOSYS;
  let total = 0;
  let text = "";
  for (let i = 0; i < iovs_len; i += 1) {
    const base = iovs + i * 8;
    const ptr = dv.getUint32(base, true);
    const len = dv.getUint32(base + 4, true);
    total += len;
    if (len > 0) {
      const bytes = new Uint8Array(wasmMemory.buffer, ptr, len);
      text += decoder.decode(bytes, { stream: true });
    }
  }
  if (text) {
    lastFdWrite = text.trim().slice(0, 500);
  }
  dv.setUint32(nwritten_ptr, total >>> 0, true);
  return 0;
}

export function fd_fdstat_get(_fd, stat_ptr) {
  record("fd_fdstat_get");
  const dv = dataView();
  if (!dv) return WASI_ERRNO_ENOSYS;
  for (let i = 0; i < 24; i += 1) {
    dv.setUint8(stat_ptr + i, 0);
  }
  return 0;
}
