//! Strict source checking of documented storage reexports without a foreign runtime.
use std::{path::PathBuf, process::Command};

#[test]
fn foreign_storage_facades_lint_all_bodies_for_each_target_without_loading_data() {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("storage-alias-lint");
    if root.exists() {
        std::fs::remove_dir_all(&root).unwrap();
    }
    for dir in [
        "stdlib/prelude",
        "stdlib/Runtime_Support",
        "Backing",
        "Facade",
    ] {
        std::fs::create_dir_all(root.join(dir)).unwrap();
    }
    // Source-only minimal bootstrap. No target/runtime execution is claimed.
    std::fs::write(root.join("stdlib/prelude/module.jai"), "").unwrap();
    std::fs::write(root.join("stdlib/Runtime_Support/module.jai"),"#module_parameters(DEFINE_SYSTEM_ENTRY_POINT:bool, DEFINE_INITIALIZATION:bool, ENABLE_BACKTRACE_ON_CRASH:bool);").unwrap();
    std::fs::write(
        root.join("Backing/module.jai"),
        include_bytes!("../../../tests/compiler/semantics/storage-aliases/Backing/module.jai"),
    )
    .unwrap();
    let facade =
        include_str!("../../../tests/compiler/semantics/storage-aliases/Facade/module.jai");
    std::fs::write(root.join("Facade/module.jai"), facade).unwrap();
    for target in [
        "aarch64-apple-darwin",
        "x86_64-pc-windows-msvc",
        "aarch64-unknown-linux-gnu",
        "wasm64-unknown-unknown",
    ] {
        for invalid in [false, true] {
            std::fs::write(
                root.join("Facade/module.jai"),
                format!(
                    "{facade} {}",
                    if invalid {
                        "unreferenced_invalid :: () {not_declared();}"
                    } else {
                        ""
                    }
                ),
            )
            .unwrap();
            let output = Command::new(env!("CARGO_BIN_EXE_jailint"))
                .current_dir(&root)
                .env("JAIC_STDLIB", root.join("stdlib"))
                .args([
                    "-target",
                    target,
                    "-I",
                    ".",
                    "-D",
                    "warnings",
                    "-v",
                    "Facade/module.jai",
                ])
                .output()
                .unwrap();
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert_eq!(
                output.status.code(),
                Some(i32::from(invalid)),
                "{target}\n{stderr}\n{stdout}"
            );
            if invalid {
                assert!(
                    stderr.contains("not_declared") || stdout.contains("not_declared"),
                    "{stderr}\n{stdout}"
                );
            }
        }
    }
}
