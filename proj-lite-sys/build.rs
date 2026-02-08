use flate2::read::GzDecoder;
use std::env;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use tar::Archive;

const PROJ_VERSION: &str = "9.7.1";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-env-changed=SQLITE3_BIN");
    println!("cargo:rerun-if-changed=vendor/proj-9.7.1.tar.gz");
    println!("cargo:rerun-if-changed=shim");
    println!("cargo:rerun-if-changed=sqlite3");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let tarball = manifest_dir.join("vendor").join("proj-9.7.1.tar.gz");
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let src_root = out_dir.join("PROJSRC").join("proj");
    let proj_src = src_root.join(format!("proj-{PROJ_VERSION}"));
    let target = env::var("TARGET").unwrap_or_default();

    unpack_tarball(&tarball, &src_root)?;

    // IMPORTANT:
    // This crate intentionally always builds PROJ from bundled source.
    // We do not probe for a system PROJ installation.
    let mut config = cmake::Config::new(&proj_src);

    // Build only the static library and disable extra tools/tests to keep
    // the build minimal and deterministic.
    config.define("BUILD_SHARED_LIBS", "OFF");
    config.define("BUILD_TESTING", "OFF");
    config.define("BUILD_APPS", "OFF");
    config.define("BUILD_CCT", "OFF");
    config.define("BUILD_CS2CS", "OFF");
    config.define("BUILD_GEOD", "OFF");
    config.define("BUILD_GIE", "OFF");
    config.define("BUILD_PROJ", "OFF");
    config.define("BUILD_PROJINFO", "OFF");
    config.define("BUILD_PROJSYNC", "OFF");

    // IMPORTANT:
    // For proj-lite we intentionally omit optional runtime deps.
    // - ENABLE_CURL=OFF disables network/grid-download path.
    // - ENABLE_TIFF=OFF avoids TIFF grid dependency.
    config.define("ENABLE_CURL", "OFF");
    config.define("ENABLE_TIFF", "OFF");
    config.define("ENABLE_EMSCRIPTEN_FETCH", "OFF");

    // IMPORTANT:
    // Embed proj.db into the static library and use only embedded resources.
    // This avoids external PROJ data lookup at runtime for core operations.
    config.define("EMBED_RESOURCE_FILES", "ON");
    config.define("USE_ONLY_EMBEDDED_RESOURCE_FILES", "ON");

    // PROJ needs a sqlite3 CLI during build to generate proj.db.
    // Selection order:
    // 1) SQLITE3_BIN override
    // 2) Platform default binary in PATH
    let sqlite3_exe = match env::var("SQLITE3_BIN") {
        Ok(val) => PathBuf::from(val),
        Err(_) if cfg!(windows) => find_in_path("sqlite3.exe").ok_or_else(|| {
            "sqlite3.exe not found in PATH; set SQLITE3_BIN or install sqlite3".to_string()
        })?,
        Err(_) => find_in_path("sqlite3").ok_or_else(|| {
            "sqlite3 not found in PATH; set SQLITE3_BIN or install sqlite3".to_string()
        })?,
    };
    config.define("EXE_SQLITE3", sqlite3_exe.display().to_string());

    if target == "wasm32-unknown-unknown" {
        // IMPORTANT:
        // For wasm32-unknown-unknown, compile sqlite with local shims that provide
        // libc-like functions required by sqlite amalgamation.
        let (sqlite_include, sqlite_library) = build_sqlite_with_shim(&manifest_dir, &out_dir);
        config.define("SQLite3_INCLUDE_DIR", sqlite_include.display().to_string());
        config.define("SQLite3_LIBRARY", sqlite_library.display().to_string());

        // Cross-compile PROJ C/C++ code to wasm32-unknown-unknown with clang.
        // Use absolute paths so CMake does not mis-resolve tool names.
        let clang = find_required_tool(&["clang"])?;
        let clangxx = find_required_tool(&["clang++"])?;
        let llvm_ar = find_required_tool(&["llvm-ar", "llvm-ar-18", "llvm-ar-17"])?;
        let llvm_ranlib = find_required_tool(&["llvm-ranlib", "llvm-ranlib-18", "llvm-ranlib-17"])?;
        config.define("CMAKE_C_COMPILER", clang.display().to_string());
        config.define("CMAKE_CXX_COMPILER", clangxx.display().to_string());
        config.define("CMAKE_AR", llvm_ar.display().to_string());
        config.define("CMAKE_RANLIB", llvm_ranlib.display().to_string());
        config.define("CMAKE_TRY_COMPILE_TARGET_TYPE", "STATIC_LIBRARY");
        let shim_include = manifest_dir.join("shim").join("musl").join("include");
        let shim_arch_include = manifest_dir
            .join("shim")
            .join("musl")
            .join("arch")
            .join("generic");
        // Use local shim headers so libc headers like string.h/stdio.h are available
        // under wasm32-unknown-unknown where no libc sysroot is provided by default.
        let wasm_flags = format!(
            "--target=wasm32-unknown-unknown -I{} -I{}",
            shim_include.display(),
            shim_arch_include.display()
        );
        config.define("CMAKE_C_FLAGS", wasm_flags.clone());
        config.define("CMAKE_CXX_FLAGS", wasm_flags);
    } else {
        // IMPORTANT:
        // Prefer libsqlite3-sys bundled outputs when available so CMake links
        // against the same sqlite build Cargo produced.
        if let Ok(sqlite_include) = env::var("DEP_SQLITE3_INCLUDE") {
            config.define("SQLite3_INCLUDE_DIR", sqlite_include);
        }
        if let Ok(sqlite_lib_dir) = env::var("DEP_SQLITE3_LIB_DIR") {
            let lib_dir = PathBuf::from(sqlite_lib_dir);
            let sqlite3_msvc = lib_dir.join("sqlite3.lib");
            let sqlite3_gnu = lib_dir.join("libsqlite3.a");
            let sqlite3_lib = if sqlite3_msvc.exists() {
                sqlite3_msvc
            } else {
                sqlite3_gnu
            };
            config.define("SQLite3_LIBRARY", sqlite3_lib.display().to_string());
        }
    }

    if target == "wasm32-unknown-emscripten" {
        // Keep in sync with PROJ's own Emscripten build recommendations.
        let flags = "-pthread -matomics -mbulk-memory -fexceptions";
        config.define("CMAKE_C_FLAGS", flags);
        config.define("CMAKE_CXX_FLAGS", flags);
    }

    if cfg!(target_env = "msvc") {
        config.profile("Release");
    }

    let proj = config.build();

    if proj.join("lib").join("proj_d.lib").exists() {
        println!("cargo:rustc-link-lib=static=proj_d");
    } else {
        println!("cargo:rustc-link-lib=static=proj");
    }
    println!(
        "cargo:rustc-link-search=native={}",
        proj.join("lib").display()
    );

    if target.contains("windows") {
        // Required by PROJ on Windows for known-folder and COM allocator APIs.
        println!("cargo:rustc-link-lib=shell32");
        println!("cargo:rustc-link-lib=ole32");
    }

    Ok(())
}

fn unpack_tarball(tarball: &Path, dst: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if dst.exists() {
        return Ok(());
    }

    // Extract once into OUT_DIR; repeated builds reuse extracted sources.
    std::fs::create_dir_all(dst)?;
    let tar_gz = File::open(tarball)?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    archive.unpack(dst)?;
    Ok(())
}

fn find_in_path(name: &str) -> Option<PathBuf> {
    let path = env::var_os("PATH")?;
    env::split_paths(&path)
        .map(|dir| dir.join(name))
        .find(|candidate| candidate.exists())
}

fn find_required_tool(candidates: &[&str]) -> Result<PathBuf, io::Error> {
    for name in candidates {
        if let Some(path) = find_in_path(name) {
            return Ok(path);
        }
    }
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        format!("required tool not found in PATH: {}", candidates.join(" or ")),
    ))
}

fn build_sqlite_with_shim(manifest_dir: &Path, out_dir: &Path) -> (PathBuf, PathBuf) {
    const SQLITE_FEATURES: [&str; 23] = [
        "-DSQLITE_OS_OTHER",
        "-DSQLITE_USE_URI",
        "-DSQLITE_THREADSAFE=0",
        "-DSQLITE_TEMP_STORE=2",
        "-DSQLITE_DEFAULT_CACHE_SIZE=-16384",
        "-DSQLITE_DEFAULT_PAGE_SIZE=8192",
        "-DSQLITE_OMIT_DEPRECATED",
        "-DSQLITE_OMIT_LOAD_EXTENSION",
        "-DSQLITE_OMIT_SHARED_CACHE",
        "-DSQLITE_ENABLE_UNLOCK_NOTIFY",
        "-DSQLITE_ENABLE_API_ARMOR",
        "-DSQLITE_ENABLE_BYTECODE_VTAB",
        "-DSQLITE_ENABLE_DBPAGE_VTAB",
        "-DSQLITE_ENABLE_DBSTAT_VTAB",
        "-DSQLITE_ENABLE_FTS5",
        "-DSQLITE_ENABLE_MATH_FUNCTIONS",
        "-DSQLITE_ENABLE_OFFSET_SQL_FUNC",
        "-DSQLITE_ENABLE_PREUPDATE_HOOK",
        "-DSQLITE_ENABLE_RTREE",
        "-DSQLITE_ENABLE_SESSION",
        "-DSQLITE_ENABLE_STMTVTAB",
        "-DSQLITE_ENABLE_UNKNOWN_SQL_FUNCTION",
        "-DSQLITE_ENABLE_COLUMN_METADATA",
    ];
    const MUSL_SOURCES: [&str; 36] = [
        "string/memchr.c",
        "string/memrchr.c",
        "string/stpcpy.c",
        "string/stpncpy.c",
        "string/strcat.c",
        "string/strchr.c",
        "string/strchrnul.c",
        "string/strcmp.c",
        "string/strcpy.c",
        "string/strcspn.c",
        "string/strlen.c",
        "string/strncat.c",
        "string/strncmp.c",
        "string/strncpy.c",
        "string/strrchr.c",
        "string/strspn.c",
        "stdlib/atoi.c",
        "stdlib/bsearch.c",
        "stdlib/qsort.c",
        "stdlib/qsort_nr.c",
        "stdlib/strtod.c",
        "stdlib/strtol.c",
        "math/__fpclassifyl.c",
        "math/acosh.c",
        "math/asinh.c",
        "math/atanh.c",
        "math/fmodl.c",
        "math/scalbn.c",
        "math/scalbnl.c",
        "math/sqrt.c",
        "math/trunc.c",
        "errno/__errno_location.c",
        "stdio/__toread.c",
        "stdio/__uflow.c",
        "internal/floatscan.c",
        "internal/shgetc.c",
    ];

    let shim_dir = manifest_dir.join("shim");
    let sqlite_dir = manifest_dir.join("sqlite3");

    let mut cc = cc::Build::new();
    cc.warnings(false)
        .flag("-Wno-macro-redefined")
        .include(&shim_dir)
        .include(shim_dir.join("musl/arch/generic"))
        .include(shim_dir.join("musl/include"))
        .file(shim_dir.join("printf/printf.c"))
        .file(sqlite_dir.join("sqlite3.c"))
        .flag("-DPRINTF_ALIAS_STANDARD_FUNCTION_NAMES_HARD")
        .flag("-include")
        .flag(shim_dir.join("wasm-shim.h").display().to_string());

    for src in MUSL_SOURCES {
        cc.file(shim_dir.join("musl").join(src));
    }
    cc.flags(&SQLITE_FEATURES);
    cc.compile("sqlite3shim");

    let include = sqlite_dir;
    let lib = out_dir.join("libsqlite3shim.a");
    (include, lib)
}
