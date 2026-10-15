//! Native constructor effects must agree between interpreted and linked programs.
#![cfg(unix)]
use std::path::Path;
use std::process::Command;

const JAIC: &str = env!("CARGO_BIN_EXE_jaic");

fn checked(command: &mut Command) {
    let result = command.output().unwrap();
    assert!(
        result.status.success(),
        "{command:?}\n{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn link_always_loads_real_constructor_without_a_referenced_symbol() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("native-link-always");
    std::fs::create_dir_all(&dir).unwrap();
    let provider = dir.join("provider.c");
    std::fs::write(
        &provider,
        "#include <stdio.h>\n#include <stdlib.h>\n\
         __attribute__((constructor)) static void original_constructor(void) {\n\
         const char *path = getenv(\"JAIC_TEST_LIBRARY_MARKER\");\n\
         if (!path) abort();\n\
         FILE *file = fopen(path, \"ab\"); if (!file) abort();\n\
         if (fputc('x', file) == EOF || fclose(file) != 0) abort();\n}\n",
    )
    .unwrap();
    let suffix = if cfg!(target_os = "macos") {
        "dylib"
    } else {
        "so"
    };
    checked(
        Command::new("cc")
            .args([
                if cfg!(target_os = "macos") {
                    "-dynamiclib"
                } else {
                    "-shared"
                },
                "-fPIC",
                "-Wall",
                "-Wextra",
                "-Werror",
            ])
            .arg(&provider)
            .arg("-o")
            .arg(dir.join(format!("libconstructor_marker.{suffix}"))),
    );
    let mut libraries = vec![dir.clone()];
    if let Some(existing) = std::env::var_os("JAIC_NATIVE_LIBS") {
        libraries.extend(std::env::split_paths(&existing));
    }
    let libraries = std::env::join_paths(libraries).unwrap();
    for named in [false, true] {
        for compile_time in [false, true] {
            let label = format!("{named}-{compile_time}");
            let source = dir.join(format!("{label}.jai"));
            let marker = dir.join(format!("{label}.marker"));
            if marker.exists() {
                std::fs::remove_file(&marker).unwrap();
            }
            let name = if named {
                "required :: "
            } else {
                ""
            };
            let run = if compile_time {
                "C :: #library,system \"c\";\nget_pid :: () -> s32 #foreign C \"getpid\";\n#run { get_pid(); }\n"
            } else {
                ""
            };
            std::fs::write(
                &source,
                format!(
                    "{name}#library,link_always \"constructor_marker\";\n{run}main :: () {{}}\n"
                ),
            )
            .unwrap();
            checked(
                Command::new(JAIC)
                    .arg(&source)
                    .arg("-interp")
                    .env("JAIC_NATIVE_LIBS", &libraries)
                    .env("JAIC_TEST_LIBRARY_MARKER", &marker),
            );
            assert_eq!(std::fs::read(&marker).unwrap(), b"x", "{label}");
            // The identical ordinary program also loads its constructor when linked.
            if !compile_time {
                let executable = dir.join(format!("{label}-native"));
                checked(
                    Command::new(JAIC)
                        .arg(&source)
                        .args(["-O0", "-exe"])
                        .arg(&executable)
                        .env("JAIC_NATIVE_LIBS", &libraries)
                        .env("JAIC_TEST_LIBRARY_MARKER", &marker),
                );
                std::fs::remove_file(&marker).unwrap();
                checked(Command::new(&executable).env("JAIC_TEST_LIBRARY_MARKER", &marker));
                assert_eq!(std::fs::read(&marker).unwrap(), b"x", "{label}");
            }
        }
    }
}

#[test]
fn missing_runtime_requirement_fails_without_loading_in_the_sandbox() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("missing-link-always");
    std::fs::create_dir_all(&dir).unwrap();
    let source = dir.join("main.jai");
    std::fs::write(
        &source,
        "required :: #library,link_always \"cleanroom_deliberately_missing_requirement\";\n\
         #assert #run -> int { return 3 + 4; } == 7;\nmain :: () {}\n",
    )
    .unwrap();
    // Pure compile-time evaluation does not run the native output's constructor.
    checked(Command::new(JAIC).arg(&source).arg("-check"));
    let native = Command::new(JAIC)
        .arg(&source)
        .arg("-interp")
        .output()
        .unwrap();
    assert!(!native.status.success());
    assert!(
        String::from_utf8_lossy(&native.stderr).contains(
            "required native library `cleanroom_deliberately_missing_requirement` could not be loaded"
        ),
        "{}",
        String::from_utf8_lossy(&native.stderr)
    );
    // This checks host isolation, not a native library or native WASI link pass.
    checked(
        Command::new(JAIC)
            .arg(&source)
            .args(["-os", "wasm", "-interp"]),
    );
}
