# proj-lite

Minimal Rust bindings for PROJ focused on point transforms.

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

- PROJ source is always built from `proj-lite-sys/vendor/proj-9.7.1.tar.gz`.
- `libsqlite3-sys` is used with `bundled` enabled.
- `libcurl` and `libtiff` are disabled in the PROJ CMake build.

### Bundled PROJ source license

This repository vendors `proj-9.7.1.tar.gz` from OSGeo/PROJ.

- The bundled PROJ source is licensed under the PROJ upstream license.
- See the bundled source `COPYING` file in the PROJ archive for the exact terms.
- Keep that license text when redistributing builds that include bundled PROJ.

## sqlite3 for proj.db generation

PROJ generates `proj.db` at build time.

- If `SQLITE3_BIN` is set, that executable is used.
- Otherwise:
  - On Windows: `sqlite3.exe` is searched in `PATH`.
  - On non-Windows: `sqlite3` is searched in `PATH`.
  - If not found, the build fails.

## WASM support

Current supported WASM target:

- `wasm32-unknown-emscripten`

CI includes a dedicated Emscripten check job.

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
  - Emscripten/WASM-oriented build ideas.

## Quick start

```bash
cargo check
cargo test
```

## Build npm package from Rust WASM

```bash
rm -rf npm
cd proj-lite-web
RUSTFLAGS="-C link-arg=--no-entry" wasm-pack build \
  --release \
  --target web \
  --out-dir ../npm \
  --out-name proj_lite_web \
  -- \
  --target wasm32-unknown-emscripten
cd ..
```

This generates JS/WASM package artifacts under `npm/`.

## Simple web demo

After generating `npm/` artifacts, serve the repository root with any static server and open `web/index.html`.
The page imports `../npm/proj_lite_web.js` and runs a single-point CRS transform.

## GitHub Pages deployment

Workflow: `.github/workflows/pages.yml`

- Builds `proj-lite-web` for `wasm32-unknown-emscripten` using `wasm-pack`
- Publishes `web/` + generated package files to GitHub Pages
