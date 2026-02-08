import { __getWasiDebug, __setWasiMemory } from "wasi_snapshot_preview1";
import { __getEnvDebug } from "env";
import * as wasi from "wasi_snapshot_preview1";
import * as env from "env";

const resultEl = document.getElementById("result");
const runBtn = document.getElementById("run");
const textEncoder = new TextEncoder();
const textDecoder = new TextDecoder();
const BUILD_TAG = "2026-02-09-d";

const wasmUrl = new URL(`./npm/proj_lite_web_bg.wasm?v=${BUILD_TAG}`, import.meta.url);
const response = await fetch(wasmUrl);
const wasmBytes = await response.arrayBuffer();
const wasmView = new Uint8Array(wasmBytes);
const wasmHead = Array.from(wasmView.slice(0, 16))
  .map((b) => b.toString(16).padStart(2, "0"))
  .join("");
let wasmSha256 = "unavailable";
if (globalThis.crypto?.subtle) {
  const digest = await globalThis.crypto.subtle.digest("SHA-256", wasmBytes);
  wasmSha256 = Array.from(new Uint8Array(digest))
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
}
const { instance } = await WebAssembly.instantiate(wasmBytes, {
  wasi_snapshot_preview1: wasi,
  env,
});
const wasm = instance.exports;
if (typeof wasm._initialize === "function") {
  wasm._initialize();
}
__setWasiMemory(wasm.memory);

function getExport(name) {
  return wasm[name] ?? wasm[`_${name}`];
}

const transform2KnownCrsRaw = getExport("transform2_known_crs_raw");
const lastErrorPtrFn = getExport("last_error_message_ptr");
const lastErrorLenFn = getExport("last_error_message_len");
const malloc = wasm.malloc;
const memory = wasm.memory;

if (
  typeof transform2KnownCrsRaw !== "function" ||
  typeof lastErrorPtrFn !== "function" ||
  typeof lastErrorLenFn !== "function" ||
  typeof malloc !== "function" ||
  !(memory instanceof WebAssembly.Memory)
) {
  resultEl.textContent =
    "Error: required raw wasm exports are missing. Rebuild npm artifacts and reload.";
}

function allocUtf8(text) {
  const bytes = textEncoder.encode(text);
  const ptr = malloc(bytes.length);
  const view = new Uint8Array(memory.buffer, ptr, bytes.length);
  view.set(bytes);
  return { ptr, len: bytes.length };
}

function getLastErrorMessage() {
  const ptr = lastErrorPtrFn();
  const len = lastErrorLenFn();
  if (!ptr || !len) return "unknown wasm error";
  const bytes = new Uint8Array(memory.buffer, ptr, len);
  return textDecoder.decode(bytes);
}

function bytesHex(text) {
  return Array.from(textEncoder.encode(text))
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
}

runBtn.addEventListener("click", () => {
  const normalizeCrs = (s) => {
    const t = String(s ?? "").trim();
    if (t.toLowerCase().startsWith("epsg:")) {
      return `EPSG:${t.slice(5).trim()}`;
    }
    return t;
  };

  const from = normalizeCrs(document.getElementById("from").value);
  const to = normalizeCrs(document.getElementById("to").value);
  const x = Number(document.getElementById("x").value);
  const y = Number(document.getElementById("y").value);

  try {
    if (typeof transform2KnownCrsRaw !== "function") {
      throw new Error("Missing transform2_known_crs_raw export");
    }
    const fromArg = allocUtf8(from);
    const toArg = allocUtf8(to);
    const outPtr = malloc(16);

    const rc = transform2KnownCrsRaw(fromArg.ptr, fromArg.len, toArg.ptr, toArg.len, x, y, outPtr);
    if (rc !== 0) {
      throw new Error(
        `${getLastErrorMessage()} [from=${JSON.stringify(from)}, to=${JSON.stringify(to)}, from_hex=${bytesHex(from)}, to_hex=${bytesHex(to)}, wasi=${JSON.stringify(__getWasiDebug())}, env=${JSON.stringify(__getEnvDebug())}, wasm_size=${wasmView.byteLength}, wasm_head=${wasmHead}, wasm_sha256=${wasmSha256}, build=${BUILD_TAG}]`,
      );
    }

    const out = new Float64Array(memory.buffer, outPtr, 2);
    resultEl.textContent = JSON.stringify({ input: [x, y], output: [out[0], out[1]] }, null, 2);
  } catch (err) {
    resultEl.textContent = `Error: ${err}`;
  }
});
