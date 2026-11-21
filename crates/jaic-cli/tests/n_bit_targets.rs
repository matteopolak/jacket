//! Programs built for targets whose pointers are narrower than the host's, run in the
//! interpreter (docs/compiler/targets.md, docs/native/n-bit-and-embedded.md).
use std::path::Path;
use std::process::Command;

const JAIC: &str = env!("CARGO_BIN_EXE_jaic");

fn run(target: &str, source: &Path) -> String {
    let output = Command::new(JAIC)
        .arg("-interp")
        .arg(source)
        .args(["-target", target])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{target}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn pointer_sizes() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/compiler/targets/pointer-sizes.jai")
}

/// A 64-bit target keeps 8-byte pointers.
#[test]
fn a_64_bit_target_has_8_byte_pointers() {
    assert_eq!(
        run("wasm64-unknown-unknown", &pointer_sizes()),
        "pointer 8\nstring 16\narray 40\nany 16\nallocator 16\npair 24 8\nheap-42\n"
    );
}

/// `wasm32` pointers are 4 bytes in the program's memory, and the program runs.
#[test]
fn a_wasm32_program_has_4_byte_pointers() {
    for target in ["wasm32-unknown-unknown", "wasm32-wasi"] {
        assert_eq!(
            run(target, &pointer_sizes()),
            "pointer 4\nstring 16\narray 32\nany 8\nallocator 8\npair 16 8\nheap-42\n",
            "{target}"
        );
    }
}

/// A `wasm32-wasi` executable gets stack traces and the arguments it is started with: the
/// compiler's trace layout and the runtime's argument table both follow the 4-byte pointers.
#[test]
fn a_wasm32_executable_keeps_traces_and_arguments() {
    use test_support::{tmp::TempDir, toolchain};

    if toolchain::wasm_ld().is_none() {
        return;
    }
    let Some(mut node) = toolchain::node() else {
        return;
    };
    let dir = TempDir::new("wasm32-runtime");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/compiler/targets/wasm32-runtime.jai");
    let module = dir.join("program.wasm");
    let build = Command::new(JAIC)
        .arg(&source)
        .args(["-target", "wasm32-wasi", "-exe"])
        .arg(&module)
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    let runner = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tools/wasi_run.ts");
    let output = node
        .arg(runner)
        .arg(&module)
        .args(["--list", "--run", "name"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "trace 2\narguments 3\n--list\n--run\nname\n"
    );
}
