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

const RUNTIME_OUTPUT: &str = "trace 2\narguments 3\n--list\n--run\nname\n";

/// Builds the wasm32 runtime program into `dir`; `None` without `wasm-ld`.
fn build_wasm32_runtime(dir: &test_support::tmp::TempDir) -> Option<std::path::PathBuf> {
    test_support::toolchain::wasm_ld()?;
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
    Some(module)
}

fn assert_runtime_output(output: &std::process::Output) {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), RUNTIME_OUTPUT);
}

/// A `wasm32-wasi` executable gets stack traces and the arguments it is started with: the
/// compiler's trace layout and the runtime's argument table both follow the 4-byte pointers.
#[test]
fn a_wasm32_executable_keeps_traces_and_arguments() {
    use test_support::{tmp::TempDir, toolchain};

    let Some(mut node) = toolchain::node() else {
        return;
    };
    let dir = TempDir::new("wasm32-runtime");
    let Some(module) = build_wasm32_runtime(&dir) else {
        return;
    };
    let runner = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tools/wasi_run.ts");
    let output = node
        .arg(runner)
        .arg(&module)
        .args(["--list", "--run", "name"])
        .output()
        .unwrap();
    assert_runtime_output(&output);
}

/// The same module under wasmtime. Skipped unless `JAIC_REQUIRE_TOOLCHAIN` names `wasmtime`,
/// which then fails with an install hint when it is missing: there is no fallback to node.
#[test]
fn a_wasm32_executable_runs_under_wasmtime() {
    use test_support::{tmp::TempDir, toolchain};

    let Some(wasmtime) = toolchain::optional("wasmtime") else {
        return;
    };
    let dir = TempDir::new("wasm32-wasmtime");
    let Some(module) = build_wasm32_runtime(&dir) else {
        return;
    };
    let output = Command::new(wasmtime)
        .arg("run")
        .arg(&module)
        .args(["--list", "--run", "name"])
        .output()
        .unwrap();
    assert_runtime_output(&output);
}

/// A plugin that sets the workspace's options back (`jacket test`'s discovery does) keeps an
/// i686 build i686: the spec's `CPU_Tag.CUSTOM` names it and wasm alike.
#[test]
fn a_plugin_setting_the_cpu_back_keeps_i686() {
    let output = Command::new(JAIC)
        .arg("-check")
        .arg(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/compiler/targets/pointer-sizes.jai"),
        )
        .args([
            "-target",
            "i686-unknown-linux-gnu",
            "+Extensions/Testing/Discovery",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// The workspace a plugin creates for a 32-bit target runs its compile-time code on the
/// interpreter's host, not the compiler's: a native `fopen` would get a truncated address.
#[test]
fn a_plugin_workspace_reads_files_at_compile_time_on_i686() {
    let output = Command::new(JAIC)
        .arg("-check")
        .arg(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/compiler/targets/compile-time-file-read.jai"),
        )
        .args([
            "-target",
            "i686-unknown-linux-gnu",
            "+Extensions/Testing/Discovery",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Type information lists enum names at the target's own string size: i686 strings are 12
/// bytes, not 16.
#[test]
fn enum_names_in_type_information_use_the_targets_string_size() {
    let source =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/compiler/targets/enum-names.jai");
    for target in [
        "x86_64-unknown-linux-gnu",
        "i686-unknown-linux-gnu",
        "wasm32-unknown-unknown",
    ] {
        assert_eq!(
            run(target, &source),
            "[0:ALPHA] [1:BETA] [2:GAMMA] \n",
            "{target}"
        );
    }
}

/// Memory holds integers in the target's byte order: a big-endian program sees the high byte
/// first, though the interpreter runs on a little-endian host.
#[test]
fn a_big_endian_program_sees_its_own_byte_order() {
    let source =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/compiler/targets/byte-order.jai");
    assert_eq!(
        run("x86_64-unknown-linux-gnu", &source),
        "4 3 2 1\n84281096\n1069547520\n"
    );
    assert_eq!(
        run("s390x-unknown-linux-gnu", &source),
        "1 2 3 4\n16909060\n1069547520\n"
    );
}
