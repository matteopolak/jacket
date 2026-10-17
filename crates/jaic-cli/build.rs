//! Builds the private POSIX crash runtime (`stdlib/internal/runtime/crash-native/runtime.c`)
//! for the target jaic is compiled for and embeds the result in the binary.
//!
//! The static archive and shared library are produced in `OUT_DIR` (so nothing is written to
//! the source tree) and re-made only when the C sources change. `src/crash_runtime.rs` unpacks
//! them into the user cache on first use and adds that directory to the library search path.
//! Targets without a POSIX crash runtime (Windows, cross builds) embed nothing.
//! See docs/native/crash-runtime-build.md.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const SOURCE_DIR: &str = "../../stdlib/internal/runtime/crash-native";
const WATCH_DIR: &str = "../../stdlib/internal/watching/native";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={SOURCE_DIR}");
    println!("cargo:rerun-if-changed={WATCH_DIR}");
    println!("cargo:rerun-if-env-changed=CC");
    println!("cargo:rerun-if-env-changed=AR");
    println!("cargo:rerun-if-env-changed=MACOSX_DEPLOYMENT_TARGET");

    let out = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR"));
    let mut files = build(&out).unwrap_or_else(|reason| {
        println!("cargo:warning=crash runtime not embedded: {reason}");
        Vec::new()
    });
    match build_watcher(&out) {
        Ok(found) => files.extend(found),
        Err(reason) => println!("cargo:warning=file watcher backend not embedded: {reason}"),
    }

    let mut digest: u64 = 0xcbf2_9ce4_8422_2325;
    let mut list = String::new();
    for (name, path) in &files {
        let bytes = fs::read(path).expect("read built crash runtime file");
        for byte in name.bytes().chain(bytes.iter().copied()) {
            digest = (digest ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3);
        }
        list.push_str(&format!(
            "    ({name:?}, include_bytes!({:?}) as &[u8]),\n",
            path.display().to_string()
        ));
    }
    let generated = format!(
        "pub const DIGEST: &str = \"{digest:016x}\";\n\
         pub const FILES: &[(&str, &[u8])] = &[\n{list}];\n"
    );
    fs::write(out.join("crash_runtime.rs"), generated).expect("write crash_runtime.rs");
}

/// Compile `runtime.c` once and make the archive and shared library from the one object.
fn build(out: &Path) -> Result<Vec<(String, PathBuf)>, String> {
    let os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    if !matches!(os.as_str(), "macos" | "linux") || !matches!(arch.as_str(), "x86_64" | "aarch64") {
        return Err(format!("no POSIX crash runtime for {os}-{arch}"));
    }
    if env::var("HOST") != env::var("TARGET") {
        return Err("cross builds do not embed the host's crash runtime".into());
    }
    let source = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap())
        .join(SOURCE_DIR)
        .join("runtime.c");
    let dir = out.join("crash-runtime");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let object = dir.join("portable-crash.o");
    let archive = dir.join("libportable_crash.a");
    let (shared_name, link_flags): (&str, &[&str]) = if os == "macos" {
        (
            "libportable_crash.dylib",
            &[
                "-dynamiclib",
                "-Wl,-install_name,@rpath/libportable_crash.dylib",
            ],
        )
    } else {
        (
            "libportable_crash.so",
            &["-shared", "-ldl", "-Wl,-soname,libportable_crash.so"],
        )
    };
    let shared = dir.join(shared_name);

    let cc = env::var("CC").unwrap_or_else(|_| "cc".into());
    let ar = env::var("AR").unwrap_or_else(|_| "ar".into());
    run(Command::new(&cc)
        .args([
            "-std=c11",
            "-O2",
            "-fno-omit-frame-pointer",
            "-fvisibility=hidden",
            "-fPIC",
        ])
        .args(["-Wall", "-Wextra", "-c"])
        .arg(&source)
        .arg("-o")
        .arg(&object))?;
    let _ = fs::remove_file(&archive);
    run(Command::new(&ar).arg("rcs").arg(&archive).arg(&object))?;
    run(Command::new(&cc)
        .arg(&object)
        .arg("-o")
        .arg(&shared)
        .arg("-pthread")
        .args(link_flags))?;
    Ok(vec![
        ("libportable_crash.a".into(), archive),
        (shared_name.into(), shared),
    ])
}

/// The macOS FSEvents file watcher backend (`stdlib/internal/watching/mac.jai` binds it as
/// `native/portable_watch`): one universal dylib. Other targets embed nothing.
fn build_watcher(out: &Path) -> Result<Vec<(String, PathBuf)>, String> {
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("macos") {
        return Ok(Vec::new());
    }
    if env::var("HOST") != env::var("TARGET") {
        return Err("cross builds do not embed the host's watcher backend".into());
    }
    let source = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap())
        .join(WATCH_DIR)
        .join("watch.c");
    let dir = out.join("watch-backend");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let shared = dir.join("libportable_watch.dylib");
    let cc = env::var("CC").unwrap_or_else(|_| "cc".into());
    run(Command::new(&cc)
        .args(["-std=c11", "-O2", "-fblocks", "-fvisibility=hidden"])
        .args(["-arch", "arm64", "-arch", "x86_64", "-Wall", "-Wextra"])
        .args([
            "-dynamiclib",
            "-Wl,-install_name,@rpath/libportable_watch.dylib",
        ])
        .args(["-framework", "CoreServices", "-pthread"])
        .arg(&source)
        .arg("-o")
        .arg(&shared))?;
    Ok(vec![("libportable_watch.dylib".into(), shared)])
}

fn run(command: &mut Command) -> Result<(), String> {
    let status = command
        .status()
        .map_err(|e| format!("{:?}: {e}", command.get_program()))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{command:?} failed with {status}"))
    }
}
