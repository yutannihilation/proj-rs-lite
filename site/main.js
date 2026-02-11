import init from "./npm/proj_lite_web.js";
import { __setWasiMemory } from "wasi_snapshot_preview1";

const resultEl = document.getElementById("result");
const runBtn = document.getElementById("run");
const textEncoder = new TextEncoder();
const textDecoder = new TextDecoder();

const wasm = await init();
if (typeof wasm._initialize === "function") {
  wasm._initialize();
}
__setWasiMemory(wasm.memory);

if (
  typeof wasm.transform2_known_crs_raw !== "function" ||
  typeof wasm.last_error_message_ptr !== "function" ||
  typeof wasm.last_error_message_len !== "function" ||
  typeof wasm.malloc !== "function" ||
  !(wasm.memory instanceof WebAssembly.Memory)
) {
  resultEl.textContent =
    "Error: required raw wasm exports are missing. Rebuild npm artifacts and reload.";
}

function allocUtf8(text) {
  const bytes = textEncoder.encode(text);
  const ptr = wasm.malloc(bytes.length);
  const view = new Uint8Array(wasm.memory.buffer, ptr, bytes.length);
  view.set(bytes);
  return { ptr, len: bytes.length };
}

function getLastErrorMessage() {
  const ptr = wasm.last_error_message_ptr();
  const len = wasm.last_error_message_len();
  if (!ptr || !len) return "unknown wasm error";
  const bytes = new Uint8Array(wasm.memory.buffer, ptr, len);
  return textDecoder.decode(bytes);
}

runBtn.addEventListener("click", () => {
  const from = document.getElementById("from").value;
  const to = document.getElementById("to").value;
  const x = Number(document.getElementById("x").value);
  const y = Number(document.getElementById("y").value);

  try {
    const fromArg = allocUtf8(from);
    const toArg = allocUtf8(to);
    const outPtr = wasm.malloc(16);
    const rc = wasm.transform2_known_crs_raw(fromArg.ptr, fromArg.len, toArg.ptr, toArg.len, x, y, outPtr);
    if (rc !== 0) {
      throw new Error(getLastErrorMessage());
    }
    const out = new Float64Array(wasm.memory.buffer, outPtr, 2);
    resultEl.textContent = JSON.stringify({ input: [x, y], output: [out[0], out[1]] }, null, 2);
  } catch (err) {
    resultEl.textContent = `Error: ${err}`;
  }
});
