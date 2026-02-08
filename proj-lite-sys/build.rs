use flate2::read::GzDecoder;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use tar::Archive;

const PROJ_VERSION: &str = "9.7.1";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-env-changed=SQLITE3_BIN");
    println!("cargo:rerun-if-changed=../proj-9.7.1.tar.gz");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let tarball = manifest_dir.join("..").join("proj-9.7.1.tar.gz");
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let src_root = out_dir.join("PROJSRC").join("proj");
    let proj_src = src_root.join(format!("proj-{PROJ_VERSION}"));

    unpack_tarball(&tarball, &src_root)?;

    let mut config = cmake::Config::new(&proj_src);
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
    config.define("ENABLE_CURL", "OFF");
    config.define("ENABLE_TIFF", "OFF");
    config.define("ENABLE_EMSCRIPTEN_FETCH", "OFF");
    config.define("EMBED_RESOURCE_FILES", "ON");
    config.define("USE_ONLY_EMBEDDED_RESOURCE_FILES", "ON");
    let sqlite3_exe = match env::var("SQLITE3_BIN") {
        Ok(val) => PathBuf::from(val),
        Err(_) => {
            let sqlite3_bin = find_in_path("sqlite3").or_else(|| find_in_path("sqlite3.exe"));
            match sqlite3_bin {
                Some(path) => path,
                None => create_sqlite3_shim(&out_dir)?,
            }
        }
    };
    config.define("EXE_SQLITE3", sqlite3_exe.display().to_string());

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

    let target = env::var("TARGET").unwrap_or_default();
    if target == "wasm32-unknown-emscripten" {
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
        println!("cargo:rustc-link-lib=shell32");
        println!("cargo:rustc-link-lib=ole32");
    }

    Ok(())
}

fn unpack_tarball(tarball: &Path, dst: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if dst.exists() {
        return Ok(());
    }

    std::fs::create_dir_all(dst)?;
    let tar_gz = File::open(tarball)?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    archive.unpack(dst)?;
    Ok(())
}

fn create_sqlite3_shim(out_dir: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let py = out_dir.join("sqlite3_shim.py");
    std::fs::write(
        &py,
        r#"import sqlite3
import sys

if len(sys.argv) != 2:
    sys.stderr.write("usage: sqlite3_shim.py <db-path>\n")
    sys.exit(2)

conn = sqlite3.connect(sys.argv[1])
try:
    conn.executescript(sys.stdin.read())
    conn.commit()
finally:
    conn.close()
"#,
    )?;

    if cfg!(windows) {
        let bat = out_dir.join("sqlite3_shim.bat");
        let mut file = std::fs::File::create(&bat)?;
        writeln!(file, "@echo off")?;
        writeln!(file, "python \"{}\" %*", py.display())?;
        Ok(bat)
    } else {
        let sh = out_dir.join("sqlite3_shim.sh");
        let mut file = std::fs::File::create(&sh)?;
        writeln!(file, "#!/bin/sh")?;
        writeln!(file, "python3 \"{}\" \"$@\"", py.display())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&sh)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&sh, perms)?;
        }
        Ok(sh)
    }
}

fn find_in_path(name: &str) -> Option<PathBuf> {
    let path = env::var_os("PATH")?;
    env::split_paths(&path)
        .map(|dir| dir.join(name))
        .find(|candidate| candidate.exists())
}
