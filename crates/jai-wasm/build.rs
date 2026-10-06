//! Generates the table of bundled `stdlib/` files embedded into the wasm module.
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

fn collect(dir: &Path, root: &Path, out: &mut Vec<PathBuf>) {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()))
        .filter_map(Result::ok)
        .collect();
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let path = entry.path();
        if entry.file_name().to_string_lossy().starts_with('.') {
            continue;
        }
        if path.is_dir() {
            collect(&path, root, out);
        } else if path.is_file() {
            out.push(path.strip_prefix(root).unwrap().to_path_buf());
        }
    }
}

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    println!("cargo:rerun-if-changed=build.rs");
    let mut code = String::from("pub static BUNDLED: &[(&str, &[u8])] = &[\n");
    // `stdlib/prelude/module.jai` loads `../stdlib/prelude/module.jai`, so both trees are bundled.
    for tree in ["stdlib"] {
        let root = manifest
            .join("../..")
            .join(tree)
            .canonicalize()
            .expect("source tree");
        println!("cargo:rerun-if-changed={}", root.display());
        let mut files = Vec::new();
        collect(&root, &root, &mut files);
        for relative in &files {
            let name = format!("{tree}/{}", relative.to_string_lossy().replace('\\', "/"));
            let absolute = root.join(relative);
            println!("cargo:rerun-if-changed={}", absolute.display());
            let _ = writeln!(
                code,
                "    ({name:?}, include_bytes!({:?})),",
                absolute.to_string_lossy()
            );
        }
    }
    code.push_str("];\n");
    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("stdlib_files.rs");
    std::fs::write(out, code).unwrap();
    // cargo-fuzz builds this crate as a dependency of the fuzz harness (`--cfg fuzzing`); the
    // unused cdylib then references libFuzzer's coverage hooks, which only the fuzz binaries
    // define. Linux shared objects may leave them undefined; macOS needs to be told.
    if std::env::var_os("CARGO_CFG_FUZZING").is_some()
        && std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos")
    {
        println!("cargo:rustc-link-arg-cdylib=-Wl,-undefined,dynamic_lookup");
    }
    // The compiler recurses deeply; the default 1 MiB wasm shadow stack is too small.
    if std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("wasm32") {
        println!("cargo:rustc-link-arg-cdylib=-zstack-size=268435456");
    }
}
