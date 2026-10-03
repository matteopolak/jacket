//! Compile only the authored LLVM shim with independently installed LLVM 22.1.
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

fn command(tool: &Path) -> Command {
    let mut command = Command::new(tool);
    for name in [
        "LD_PRELOAD",
        "LD_LIBRARY_PATH",
        "DYLD_INSERT_LIBRARIES",
        "DYLD_LIBRARY_PATH",
        "DYLD_FALLBACK_LIBRARY_PATH",
        "DYLD_FRAMEWORK_PATH",
        "DYLD_FALLBACK_FRAMEWORK_PATH",
        "CCC_OVERRIDE_OPTIONS",
    ] {
        command.env_remove(name);
    }
    command
}
fn run(command: &mut Command) -> Output {
    let output = command.output().expect("run installed LLVM tool");
    assert!(
        output.status.success(),
        "LLVM shim tool failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}
fn guarded(path: PathBuf, root: &Path) -> PathBuf {
    let path = path
        .canonicalize()
        .expect("installed LLVM shim tool/header path");
    for protected in ["reference", "corpus", ".git"] {
        let protected = root.join(protected);
        let protected = protected.canonicalize().unwrap_or(protected);
        assert!(
            !path.starts_with(protected),
            "LLVM tools and headers cannot come from supplied native inputs"
        );
    }
    path
}
fn executable(prefix: Option<&Path>, name: &str, root: &Path) -> PathBuf {
    let suffix = if cfg!(windows) { ".exe" } else { "" };
    let file = format!("{name}{suffix}");
    let path = if let Some(prefix) = prefix {
        prefix.join("bin").join(file)
    } else {
        env::split_paths(&env::var_os("PATH").unwrap_or_default())
            .map(|entry| entry.join(&file))
            .find(|path| path.is_file())
            .expect("LLVM 22.1 tools required; set LLVM_SYS_221_PREFIX")
    };
    guarded(path, root)
}
fn main() {
    let manifest = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let root = manifest.parent().unwrap().parent().unwrap();
    let prefix = env::var_os("LLVM_SYS_221_PREFIX").map(PathBuf::from);
    let config = executable(prefix.as_deref(), "llvm-config", root);
    let version = String::from_utf8(run(command(&config).arg("--version")).stdout).unwrap();
    assert!(
        version.trim().starts_with("22.1."),
        "LLVM shim requires LLVM 22.1, found {version}"
    );
    let prefix = match prefix {
        Some(prefix) => prefix,
        None => PathBuf::from(
            String::from_utf8(run(command(&config).arg("--prefix")).stdout)
                .unwrap()
                .trim(),
        ),
    };
    let compiler = executable(Some(&prefix), "clang++", root);
    let compiler_version =
        String::from_utf8(run(command(&compiler).arg("--version")).stdout).unwrap();
    assert!(
        compiler_version.contains("clang version 22.1."),
        "LLVM shim requires Clang 22.1"
    );
    let archiver = executable(Some(&prefix), "llvm-ar", root);
    let archiver_version =
        String::from_utf8(run(command(&archiver).arg("--version")).stdout).unwrap();
    assert!(
        archiver_version.contains("LLVM version 22.1."),
        "LLVM shim requires llvm-ar 22.1"
    );
    let include = guarded(
        PathBuf::from(
            String::from_utf8(run(command(&config).arg("--includedir")).stdout)
                .unwrap()
                .trim(),
        ),
        root,
    );
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let source = manifest.join("src/debug_shim.cpp");
    let object = out.join("debug_shim.o");
    let target = env::var("TARGET").unwrap();
    let msvc = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default() == "msvc";
    let archive = out.join(if msvc {
        "jai_llvm_debug_shim.lib"
    } else {
        "libjai_llvm_debug_shim.a"
    });
    run(command(&compiler)
        .arg(format!("--target={target}"))
        .args(["-std=c++17", "-fno-exceptions", "-fno-rtti", "-c"])
        .arg(&source)
        .arg("-I")
        .arg(&include)
        .arg("-o")
        .arg(&object));
    let mut archive_command = command(&archiver);
    if msvc {
        archive_command.arg("--format=coff");
    }
    run(archive_command.arg("crs").arg(&archive).arg(&object));
    fs::write(
        out.join("debug-shim-toolchain.txt"),
        format!(
            "LLVM={}\ncompiler={}\narchiver={}\nRust host={}\nRust target={target}\nsource={}\n",
            version.trim(),
            compiler.display(),
            archiver.display(),
            env::var("HOST").unwrap(),
            source.display()
        ),
    )
    .unwrap();
    for path in [
        &source,
        &compiler,
        &archiver,
        &config,
        &include.join("llvm/Config/llvm-config.h"),
    ] {
        println!("cargo:rerun-if-changed={}", path.display());
    }
    for name in ["LLVM_SYS_221_PREFIX", "PATH"] {
        println!("cargo:rerun-if-env-changed={name}");
    }
    println!("cargo:rustc-link-search=native={}", out.display());
    println!("cargo:rustc-link-lib=static=jai_llvm_debug_shim");
    match env::var("CARGO_CFG_TARGET_OS").unwrap().as_str() {
        "macos" | "ios" | "freebsd" => println!("cargo:rustc-link-lib=c++"),
        "windows" if msvc => {}
        _ => println!("cargo:rustc-link-lib=stdc++"),
    }
}
