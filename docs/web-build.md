# Web Build Notes

This file describes the current shim-based web build configuration.

## Target and toolchain

- Rust target: `wasm32-unknown-emscripten`
- Toolchain: Emscripten (Emsdk)
- Binding generator: `wasm-bindgen`

## Build command

```bash
cargo build --release --target wasm32-unknown-emscripten -p proj-lite-web
wasm-bindgen \
  --out-dir ./npm \
  --typescript \
  --target web \
  ./target/wasm32-unknown-emscripten/release/proj_lite_web.wasm
```

For emscripten linking, `.cargo/config.toml` sets:

- `--no-entry`
- `-sSTANDALONE_WASM=0`
- `-sFILESYSTEM=0`

## Compiler and linker flags (detailed)

The current build has two flag layers:

- C/C++ compile flags passed to PROJ via `proj-lite-sys/build.rs` (`CMAKE_C_FLAGS`/`CMAKE_CXX_FLAGS`)
- Rust linker arguments for `wasm32-unknown-emscripten` via `.cargo/config.toml`

`proj-lite-sys/build.rs` now contains detailed inline comments next to each important
configuration block. Use that file as the source-of-truth when updating flags.

### C/C++ compile flags (`proj-lite-sys/build.rs`)

Active flags:

- `-pthread`
- `-matomics`
- `-mbulk-memory`
- `-fwasm-exceptions`

Why each flag exists:

- `-pthread`
  - Enables thread-aware libc++/Emscripten headers and ABI expectations.
  - PROJ is C++, and parts of the dependency/runtime surface assume thread-capable libc++ definitions even when browser code is not using worker-based pthreads directly.
  - Without this, toolchain/header mismatches can appear in C++ standard library code paths.

- `-matomics`
  - Enables wasm atomic instructions in generated code where required by thread-capable runtime paths.
  - Pairs with `-pthread` so the compilation model is internally consistent.
  - Missing this can produce compile/link incompatibilities when atomic operations are referenced.

- `-mbulk-memory`
  - Enables bulk-memory wasm instructions used by modern wasm runtimes/toolchains.
  - Keeps generated objects aligned with Emscripten’s expected feature set in this configuration.
  - If object feature sets diverge, link/runtime validation issues can occur.

- `-fwasm-exceptions`
  - Selects wasm exception handling model for C++.
  - This is critical for PROJ because it is C++ and uses exception-capable code paths.
  - Must match Rust/Emscripten final link mode; mismatch caused previous failures:
    - `undefined symbol: __resumeException`
    - `undefined symbol: llvm_eh_typeid_for`

Related build-script config (also documented inline in code):

- `BUILD_*` tool/test targets are disabled to keep outputs deterministic and minimal.
- `ENABLE_CURL=OFF`, `ENABLE_TIFF=OFF`, `ENABLE_EMSCRIPTEN_FETCH=OFF` to avoid
  optional network/TIFF runtime dependencies in browser-oriented builds.
- `EMBED_RESOURCE_FILES=ON`, `USE_ONLY_EMBEDDED_RESOURCE_FILES=ON` so runtime does not
  rely on external PROJ data files.
- `EXE_SQLITE3` is explicitly resolved because PROJ needs sqlite3 CLI to build `proj.db`.
- `SQLite3_INCLUDE_DIR` / `SQLite3_LIBRARY` are forwarded from `libsqlite3-sys` outputs
  to avoid CMake selecting a different host sqlite installation.

### Rust linker flags (`.cargo/config.toml`)

Active flags:

- `-C link-arg=--no-entry`
- `-C link-arg=-sSTANDALONE_WASM=0`
- `-C link-arg=-sFILESYSTEM=0`

Why each flag exists:

- `--no-entry`
  - Produces a module-style wasm artifact (no C `main` entrypoint required).
  - Required for library-style wasm output consumed by `wasm-bindgen`.
  - Without it, link fails with:
    - `wasm-ld: ... undefined symbol: main`

- `-sSTANDALONE_WASM=0`
  - Disables standalone-WASM mode and keeps Emscripten runtime integration model.
  - Ensures the resulting module follows the import/runtime shape expected by the generated JS glue and this project’s shim approach.

- `-sFILESYSTEM=0`
  - Disables Emscripten filesystem runtime.
  - Reduces runtime surface and avoids pulling in unnecessary FS machinery for this demo/library scenario.
  - Also helps keep host import requirements minimal and predictable.

### Practical rule: keep flag sets aligned

The most important operational rule is consistency across C++ compile and final link:

- Do not mix different exception models between C++ objects and final Rust/Emscripten link.
- Keep Emscripten feature flags coherent (`pthread`/atomics/bulk-memory) across the build.
- Keep no-entry linker mode enabled for browser library output.

If any of those drift, builds often still compile partially but fail late at link or at runtime with hard-to-debug module import/exception errors.

## Notes on native linking

Native C++ stdlib linking is now handled by `link-cplusplus` defaults.

`proj-lite-sys/build.rs` still adds Windows system libraries required by PROJ:

- `shell32`
- `ole32`

## Runtime model in browser

Generated JS currently imports bare module specifiers:

- `wasi_snapshot_preview1`
- `env`

Browsers do not provide these, so we map them via import map and local shim modules:

- `web/wasi_snapshot_preview1.js`
- `web/env.js`

`web/index.html` provides the import map, and `web/main.js` initializes wasm and calls raw exports.

## Vendored sources

- PROJ source is vendored as archive: `proj-lite-sys/vendor/proj-9.8.0.tar.gz`
- Build script extracts it into `OUT_DIR` at build time.
- Local bundled C shim/sqlite sources were intentionally removed; only PROJ source is vendored.

## CI / Pages

- CI WASM check: `.github/workflows/ci.yml` (`wasm32-unknown-emscripten`)
- Pages publish flow: `.github/workflows/pages.yml`
  - Builds wasm package
  - Copies `web/index.html`, `web/main.js`, and shim files into `site/npm`
  - Copies generated wasm/js artifacts

## Common failure signatures

- `Uncaught TypeError: Failed to resolve module specifier "env"`
  - Import map/shim files are missing from deployed static files.

- `undefined symbol: __resumeException` / `llvm_eh_typeid_for`
  - Exception model mismatch between compiled C++ objects and final link.
  - Current setup uses `-fwasm-exceptions` for emscripten C/C++ flags.

- `wasm-ld: ... undefined symbol: main`
  - Missing emscripten no-entry linker arguments (configured in `.cargo/config.toml`).
