# proj-lite

> [!WARNING]
> The current codebase was initially generated with assistance from Codex.

This repository is an experiment to answer one question:

Can a Rust crate that wraps **PROJ** be built and run in a web browser?

## Result

Short answer: partially.

- It **does run in browsers** ([demo](https://yutannihilation.github.io/proj-rs-lite/)).
- But it does **not run as a clean, standalone browser module**.
- It still needs browser-side runtime shims for imports like `wasi_snapshot_preview1` and `env`.

So this experiment is a practical success, but architecturally a partial failure.
Please refer to [`docs/web-build.md`](docs/web-build.md) for the details about build configurations.

## Why this happened

The main reason is that PROJ is a large C++ codebase, not an amalgamated C library like SQLite.

- C++ runtime and exception behavior pull in more host/runtime expectations.
- Those expectations appear as WASI/env imports that browsers do not provide directly.
- We therefore need shim modules in the web app.

## Why `wasm32-unknown-emscripten`

We also tried `wasm32-unknown-unknown`.
In practice, we ended up with the same class of runtime gap (host imports that need browser-side support), plus more fragile integration.

So the current project uses `wasm32-unknown-emscripten` as the more workable path.

## Current status

- Browser demo works with shim modules.
- CI and GitHub Pages publish this shim-based setup.

## Usage

### Rust (native)

```rust
use proj_lite::Proj;

let tf = Proj::new_known_crs("EPSG:2230", "EPSG:26946")?;
let out = tf.transform2((4_760_096.421_921, 3_744_293.729_449))?;
println!("{out:?}");
# Ok::<(), proj_lite::ProjError>(())
```

```rust
use proj_lite::Proj;
use std::str::FromStr;
use wkt::Wkt;

let tf = Proj::new_known_crs("EPSG:4326", "EPSG:4979")?;
let point3d = match Wkt::<f64>::from_str("POINT Z (-122.4194 37.7749 10.0)")? {
    Wkt::Point(p) => p.coord().unwrap().clone(),
    _ => unreachable!(),
};

let xy = tf.transform2(point3d.clone())?;
let xyz = tf.transform3(point3d)?;
println!("{xy:?} {xyz:?}");
# Ok::<(), Box<dyn std::error::Error>>(())
```

### Rust (WASM entrypoint)

```rust
#[wasm_bindgen]
pub fn transform2_known_crs(
    from_crs: &str,
    to_crs: &str,
    x: f64,
    y: f64,
) -> Result<Vec<f64>, JsValue> {
    let proj = proj_lite::Proj::new_known_crs(from_crs, to_crs)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let (xo, yo) = proj
        .transform2((x, y))
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(vec![xo, yo])
}
```

### JavaScript (browser)

```js
import * as env from "./npm/env.js"; // Emscripten libc/syscall shim module
import * as wasi from "./npm/wasi_snapshot_preview1.js"; // WASI shim module for browser runtime

const wasmUrl = new URL("./npm/proj_lite_web_bg.wasm", import.meta.url);
const wasmBytes = await (await fetch(wasmUrl)).arrayBuffer();
const { instance } = await WebAssembly.instantiate(wasmBytes, {
  env, // Provides host functions imported under module name "env"
  wasi_snapshot_preview1: wasi, // Maps WASI imports to our browser shim implementation
});
const wasm = instance.exports;
if (typeof wasm._initialize === "function") wasm._initialize();
wasi.__setWasiMemory(wasm.memory);

const encode = new TextEncoder();
const alloc = (s) => {
  const b = encode.encode(s);
  const p = wasm.malloc(b.length);
  new Uint8Array(wasm.memory.buffer, p, b.length).set(b);
  return [p, b.length];
};

const [fromPtr, fromLen] = alloc("EPSG:2230");
const [toPtr, toLen] = alloc("EPSG:26946");
const outPtr = wasm.malloc(16);
const rc = wasm.transform2_known_crs_raw(
  fromPtr,
  fromLen,
  toPtr,
  toLen,
  4760096.421921,
  3744293.729449,
  outPtr,
);
if (rc !== 0) throw new Error("transform failed");

const out = new Float64Array(wasm.memory.buffer, outPtr, 2);
console.log(out[0], out[1]);
```

## Bundled PROJ source license

This repository vendors PROJ source distribution content from OSGeo/PROJ.

- The bundled PROJ source is licensed under the PROJ upstream license.
- See the bundled source `COPYING` file for the exact terms.
- Keep that license text when redistributing builds that include bundled PROJ.

## Credits

The implementation and build strategy were informed by these repositories:

- https://github.com/georust/proj
  - Rust API and FFI layering patterns.
- https://github.com/OSGeo/PROJ
  - PROJ source code and CMake/build configuration.
- https://github.com/Spxg/sqlite-wasm-rs
  - SQLite/WASM build ideas.
