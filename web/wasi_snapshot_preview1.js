// Browser shims for WASI imports used by the wasm module.
// We intentionally return ENOSYS for unsupported host features in browser.

const WASI_ERRNO_ENOSYS = 52;
let wasmMemory = null;
const decoder = new TextDecoder();

export function __setWasiMemory(memory) {
  wasmMemory = memory;
}

function dataView() {
  if (!(wasmMemory instanceof WebAssembly.Memory)) return null;
  return new DataView(wasmMemory.buffer);
}

export function proc_exit(code) {
  throw new Error(`WASI proc_exit(${code})`);
}

export function environ_sizes_get(environ_count_ptr, environ_buf_size_ptr) {
  const dv = dataView();
  if (!dv) return WASI_ERRNO_ENOSYS;
  dv.setUint32(environ_count_ptr, 0, true);
  dv.setUint32(environ_buf_size_ptr, 0, true);
  return 0;
}

export function environ_get(_environ_ptr, _environ_buf_ptr) {
  return 0;
}

export function fd_close(_fd) {
  return 0;
}

export function clock_time_get(_clock_id, _precision, time_ptr) {
  const dv = dataView();
  if (!dv) return WASI_ERRNO_ENOSYS;
  const nowNs = BigInt(Date.now()) * 1000000n;
  dv.setBigUint64(time_ptr, nowNs, true);
  return 0;
}

export function fd_sync(_fd) {
  return WASI_ERRNO_ENOSYS;
}

export function fd_seek(_fd, _offset, _whence, new_offset_ptr) {
  const dv = dataView();
  if (!dv) return WASI_ERRNO_ENOSYS;
  dv.setBigUint64(new_offset_ptr, 0n, true);
  return 0;
}

export function fd_read(_fd, _iovs, _iovs_len, nread_ptr) {
  const dv = dataView();
  if (!dv) return WASI_ERRNO_ENOSYS;
  dv.setUint32(nread_ptr, 0, true);
  return 0;
}

export function fd_write(_fd, iovs, iovs_len, nwritten_ptr) {
  const dv = dataView();
  if (!dv) return WASI_ERRNO_ENOSYS;
  let total = 0;
  for (let i = 0; i < iovs_len; i += 1) {
    const base = iovs + i * 8;
    const ptr = dv.getUint32(base, true);
    const len = dv.getUint32(base + 4, true);
    total += len;
    if (len > 0) {
      const bytes = new Uint8Array(wasmMemory.buffer, ptr, len);
      // Keep browser behavior deterministic without leaking to console by default.
      decoder.decode(bytes, { stream: true });
    }
  }
  dv.setUint32(nwritten_ptr, total >>> 0, true);
  return 0;
}

export function fd_fdstat_get(_fd, stat_ptr) {
  const dv = dataView();
  if (!dv) return WASI_ERRNO_ENOSYS;
  for (let i = 0; i < 24; i += 1) {
    dv.setUint8(stat_ptr + i, 0);
  }
  return 0;
}
