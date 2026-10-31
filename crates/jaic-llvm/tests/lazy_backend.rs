//! The code generation library (`jaic-backend`) writes the same object a direct `jaic-llvm` call
//! does, and the loader refuses what is not that library.
use jaic_native::backend;
use std::path::PathBuf;
use std::process::Command;
use test_support::{TempDir, backend as support};

/// `libjaic_backend` of this build, built here when the test run did not (`cargo test` builds a
/// cdylib crate's tests, not its library).
fn library() -> PathBuf {
    let status = Command::new(env!("CARGO"))
        .args(["build", "--quiet", "--locked", "-p", "jaic-backend"])
        .status()
        .expect("run cargo");
    assert!(status.success(), "cargo build -p jaic-backend failed");
    let exe = std::env::current_exe().expect("test executable");
    // The profile directory (`target/debug`) is an ancestor of the test executable.
    exe.ancestors()
        .map(|dir| dir.join(backend::library_name()))
        .find(|library| library.is_file())
        .unwrap_or_else(|| panic!("no {} above {}", backend::library_name(), exe.display()))
}

#[test]
fn the_library_emits_what_a_direct_call_emits() {
    let loaded = backend::load_from(&library()).unwrap_or_else(|e| panic!("{e}"));
    let variant = support::VARIANTS[1];
    let compiler = variant
        .compile(r#"#program_export "answer" answer :: () -> s64 #c_call { return 6 * 7; }"#);
    let options = variant.emit_options(None);
    let dir = TempDir::new("lazy-backend");
    let direct = dir.join("direct.o");
    let through_library = dir.join("library.o");
    jaic_llvm::emit_object(&compiler.out.program, &options, &direct).unwrap();
    loaded
        .emit_object(&compiler.out.program, &options, &through_library)
        .unwrap();
    assert_eq!(
        std::fs::read(&direct).unwrap(),
        std::fs::read(&through_library).unwrap()
    );
    let objects = loaded
        .emit_objects(&compiler.out.program, &options, &dir.join("many.o"))
        .unwrap();
    assert_eq!(objects, [dir.join("many.o")]);
}

#[test]
fn a_failure_in_the_library_comes_back_as_text() {
    let loaded = backend::load_from(&library()).unwrap_or_else(|e| panic!("{e}"));
    let variant = support::VARIANTS[0];
    let compiler =
        variant.compile(r#"#program_export "one" one :: () -> s64 #c_call { return 1; }"#);
    let dir = TempDir::new("lazy-backend-failure");
    // A directory cannot be written as an object file.
    let message = loaded
        .emit_object(
            &compiler.out.program,
            &variant.emit_options(None),
            dir.path(),
        )
        .unwrap_err();
    assert!(!message.is_empty());
}

#[test]
fn a_file_that_is_not_the_library_is_refused() {
    let dir = TempDir::new("lazy-backend-refused");
    let missing = dir.join("nothing");
    let message = backend::load_from(&missing).err().expect("no such file");
    assert!(message.contains("could not open"), "{message}");
    // A real library without the backend's entry points.
    let system = if cfg!(target_os = "macos") {
        "/usr/lib/libSystem.B.dylib"
    } else if cfg!(target_os = "linux") {
        "libm.so.6"
    } else {
        return;
    };
    let message = backend::load_from(std::path::Path::new(system))
        .err()
        .expect("not a backend");
    assert!(
        message.contains("is not a jaic code generation library"),
        "{message}"
    );
}
