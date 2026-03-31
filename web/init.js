// Minimal loader for the Emscripten-compiled wasm module.
// Provides env and wasi_snapshot_preview1 imports via the import-map
// defined in index.html, then returns the wasm instance exports.

import * as env from "env";
import * as wasi from "wasi_snapshot_preview1";

export default async function init(wasmPath) {
  const response = await fetch(wasmPath);
  const { instance } = await WebAssembly.instantiateStreaming(response, {
    env,
    wasi_snapshot_preview1: wasi,
  });
  return instance.exports;
}
