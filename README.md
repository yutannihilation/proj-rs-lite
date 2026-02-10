# proj-lite

Minimal Rust bindings for PROJ focused on point transforms.

## WASM Notes (Current Setup)

This repository is currently focused on `wasm32-unknown-unknown`.

- `proj-lite-sys` builds bundled PROJ from `proj-lite-sys/vendor/proj-9.8.0/`.
- For `wasm32-unknown-unknown`, sqlite is compiled from the bundled amalgamation
  with local shim sources under `proj-lite-sys/shim/` and `proj-lite-sys/sqlite3/`.
- The browser demo uses `wasm-bindgen` output (`npm/proj_lite_web.js` + `npm/proj_lite_web_bg.wasm`)
  and does not rely on Emscripten runtime shims.

### Why this works without WASI shims

`wasm32-unknown-unknown` has no default libc/sysroot. PROJ itself is C/C++, so the build uses two paths:

- `proj-lite-sys/build.rs` compiles PROJ C/C++ with Emscripten's Clang + sysroot
  (`--target=wasm32-unknown-unknown --sysroot=<emsdk>/upstream/emscripten/cache/sysroot`).
- SQLite (used by PROJ's `proj.db` build step) is compiled from bundled amalgamation with local shims
  (`proj-lite-sys/shim/` + `proj-lite-sys/sqlite3/`).

This avoids importing `wasi_snapshot_preview1` in browser runtime and avoids JS-side WASI/env shim files.

### Required build environment for wasm32-unknown-unknown

- Emsdk installed and available in `PATH` (for `emcc`, `emar`, `emranlib`, Emscripten sysroot).
- `sqlite3` CLI available in `PATH` (or set `SQLITE3_BIN`).
- `wasm-bindgen-cli` for generating JS glue.

Recommended target compiler env (used in CI too):

```bash
export CC_wasm32_unknown_unknown="$EMSDK/upstream/bin/clang"
export CXX_wasm32_unknown_unknown="$EMSDK/upstream/bin/clang++"
```

### Troubleshooting

- `warning: proj-lite-sys@... Compiler family detection failed ... detect_compiler_family.c`
  - These warnings come from `cc-rs` probing the compiler and are expected in this setup.
  - If the build continues and finishes, they can be ignored.

- `unable to create target: wasm32-unknown-unknown` or host `clang` errors
  - `CC_wasm32_unknown_unknown` / `CXX_wasm32_unknown_unknown` are not pointing to Emsdk clang.
  - Set:
    - `CC_wasm32_unknown_unknown=$EMSDK/upstream/bin/clang`
    - `CXX_wasm32_unknown_unknown=$EMSDK/upstream/bin/clang++`

- `sqlite3 not found in PATH`
  - Install `sqlite3` or set `SQLITE3_BIN` to the sqlite3 executable path.

- Linker error like `unable to find library -lstdc++`
  - Ensure you are using current `proj-lite-sys` configuration (manual `link-cplusplus` mode + Emscripten libc++ link path from `build.rs`).

## Disclaimer

The current codebase was initially generated with assistance from Codex.
Ongoing development is expected to include substantial hand-written changes.

## Scope

- Minimal high-level API:
  - `Proj::new(definition)`
  - `Proj::new_known_crs(from, to)`
  - `transform2(coord)`
  - `transform3(coord)`
- Bundled build only (no system `pkg-config` lookup path)
- Optional/network features intentionally omitted

## Bundled dependencies

- PROJ source is always built from `proj-lite-sys/vendor/proj-9.8.0/` (official `dist` archive content).
- `libsqlite3-sys` is used with `bundled` enabled.
- `libcurl` and `libtiff` are disabled in the PROJ CMake build.

### Bundled PROJ source license

This repository vendors PROJ source distribution content from OSGeo/PROJ.

- The bundled PROJ source is licensed under the PROJ upstream license.
- See the bundled source `COPYING` file for the exact terms.
- Keep that license text when redistributing builds that include bundled PROJ.

### Updating bundled PROJ source

- Submodule checkout: `proj-lite-sys/vendor/proj`
- Build + extract official source distribution:
  - `proj-lite-sys/vendor/update_proj_vendor.sh`
- The crate build consumes the extracted `proj-lite-sys/vendor/proj-<version>/` directory.
- `proj-lite-sys/vendor/proj/**` is excluded from crate packaging to avoid shipping the entire git repository.

## sqlite3 for proj.db generation

PROJ generates `proj.db` at build time.

- If `SQLITE3_BIN` is set, that executable is used.
- Otherwise:
  - On Windows: `sqlite3.exe` is searched in `PATH`.
  - On non-Windows: `sqlite3` is searched in `PATH`.
  - If not found, the build fails.

## WASM support

Current supported WASM target:

- `wasm32-unknown-unknown` (for npm/web packaging via `proj-lite-web`)

## Examples

### Convert coordinates between known CRS

```rust
use proj_lite::Proj;

let tf = Proj::new_known_crs("EPSG:2230", "EPSG:26946")?;
let out = tf.transform2((4_760_096.421_921, 3_744_293.729_449))?;
println!("{out:?}");
# Ok::<(), proj_lite::ProjError>(())
```

### Use a custom PROJ pipeline

```rust
use proj_lite::Proj;

let tf = Proj::new(
    "+proj=pipeline \
     +step +proj=longlat +datum=WGS84 \
     +step +proj=merc +datum=WGS84"
)?;
let out = tf.transform2((-122.4194, 37.7749))?;
println!("{out:?}");
# Ok::<(), proj_lite::ProjError>(())
```

### `transform2` / `transform3` with WKT coordinates

```rust
use proj_lite::Proj;
use std::str::FromStr;
use wkt::Wkt;

let tf = Proj::new_known_crs("EPSG:4326", "EPSG:4979")?;
let point3d = match Wkt::<f64>::from_str("POINT Z (-122.4194 37.7749 10.0)")? {
    Wkt::Point(p) => p.coord().unwrap().clone(),
    _ => unreachable!(),
};

// transform2:
// - accepts 2D or 3D+ CoordTrait input
// - uses x/y and discards z (if present)
let xy = tf.transform2(point3d.clone())?;
println!("{xy:?}");

// transform3:
// - accepts 2D or 3D+ CoordTrait input
// - if input is 2D, z is filled with 0.0
let xyz_from_2d = tf.transform3((-122.4194, 37.7749))?;
let xyz_from_3d = tf.transform3(point3d)?;
println!("{xyz_from_2d:?} {xyz_from_3d:?}");
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Credits

The implementation and build strategy were informed by these repositories:

- https://github.com/georust/proj
  - Rust API and FFI layering patterns.
- https://github.com/OSGeo/PROJ
  - PROJ source code and CMake/build configuration.
- https://github.com/Spxg/sqlite-wasm-rs
  - SQLite/WASM build ideas and shim approach.

## Quick start

```bash
cargo check
cargo test
```

## Build npm package from Rust WASM

```bash
export CC_wasm32_unknown_unknown="$EMSDK/upstream/bin/clang"
export CXX_wasm32_unknown_unknown="$EMSDK/upstream/bin/clang++"
cargo build --release --target wasm32-unknown-unknown -p proj-lite-web
wasm-bindgen \
  --out-dir ./npm \
  --typescript \
  --target web \
  ./target/wasm32-unknown-unknown/release/proj_lite_web.wasm
```

This generates JS/TS bindings and `.wasm` under `npm/` (with metadata in `npm/package.json`).

## Simple web demo

After generating `npm/` artifacts, serve the repository root with any static server and open `web/index.html`.
The page imports `../npm/proj_lite_web.js` and runs a single-point CRS transform.

## GitHub Pages deployment

Workflow: `.github/workflows/pages.yml`

- Builds `proj-lite-web` for `wasm32-unknown-unknown`
- Runs `wasm-bindgen` to generate the npm package files
- Publishes `web/` + generated package files to GitHub Pages
