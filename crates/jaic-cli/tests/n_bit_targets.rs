//! Programs built for targets whose pointers are narrower than the host's, run in the
//! interpreter (docs/compiler/targets.md, docs/native/linux-cpu-bindings.md).
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

/// The same module under wasmtime. Skipped when `wasmtime` is not installed: there is no fallback
/// to node.
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

/// `armv7` pointers are 4 bytes and its `s64` aligns to 8, so `string` is 16 bytes there.
#[test]
fn an_armv7_program_has_4_byte_pointers() {
    assert_eq!(
        run("armv7-unknown-linux-gnueabihf", &pointer_sizes()),
        "pointer 4\nstring 16\narray 32\nany 8\nallocator 8\npair 16 8\nheap-42\n"
    );
}

/// A plugin that sets the workspace's options back (`jacket test`'s discovery does) keeps an
/// i686, s390x or armv7 build as it was: the spec's `CPU_Tag.CUSTOM` names them and wasm alike.
#[test]
fn a_plugin_setting_the_cpu_back_keeps_the_target() {
    for target in [
        "i686-unknown-linux-gnu",
        "s390x-unknown-linux-gnu",
        "armv7-unknown-linux-gnueabihf",
        "powerpc64-unknown-linux-gnu",
    ] {
        let output = Command::new(JAIC)
            .arg("-check")
            .arg(
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../tests/compiler/targets/pointer-sizes.jai"),
            )
            .args(["-target", target, "+Extensions/Testing/Discovery"])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{target}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
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
        "armv7-unknown-linux-gnueabihf",
        "powerpc64-unknown-linux-gnu",
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
    for target in ["s390x-unknown-linux-gnu", "powerpc64-unknown-linux-gnu"] {
        assert_eq!(
            run(target, &source),
            "1 2 3 4\n16909060\n1069547520\n",
            "{target}"
        );
    }
}

/// `powerpc64` keeps 8-byte pointers and runs the same program.
#[test]
fn a_powerpc64_program_has_8_byte_pointers() {
    assert_eq!(
        run("powerpc64-unknown-linux-gnu", &pointer_sizes()),
        "pointer 8\nstring 16\narray 40\nany 16\nallocator 16\npair 24 8\nheap-42\n"
    );
}

/// `volatile_load` and `volatile_store` at compile time use the program's byte order, like a
/// plain load or store.
#[test]
fn volatile_accesses_use_the_targets_byte_order() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/compiler/targets/volatile-order.jai");
    assert_eq!(
        run("x86_64-unknown-linux-gnu", &source),
        "16909060\n168496141 13 3085\n"
    );
    for target in ["s390x-unknown-linux-gnu", "powerpc64-unknown-linux-gnu"] {
        assert_eq!(
            run(target, &source),
            "16909060\n168496141 10 2571\n",
            "{target}"
        );
    }
}

/// A program for the Arduino Uno board (2-byte pointers) passes compile-time checks: the
/// compiler's own compile-time buffers fit the 16-bit address window. A check writes no
/// executable, so the board writes no linker script beside it.
#[test]
fn an_avr_program_checks_at_compile_time() {
    let dir = test_support::tmp::TempDir::new("avr-check");
    let output = Command::new(JAIC)
        .arg("-check")
        .arg("-exe")
        .arg(dir.join("avr-hello"))
        .arg(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/compiler/targets/avr-hello.jai"),
        )
        .arg("+Extensions/Embedded/Boards/Arduino_Uno")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!dir.join("avr-hello.ld").exists());
}

/// `-target` names the program's target, not the plugins' metaprogram's: the board's
/// `get_plugin`, which only a host build declares, is found with a freestanding `-target`.
#[test]
fn a_board_plugin_runs_on_the_host_with_a_freestanding_target() {
    let dir = test_support::tmp::TempDir::new("avr-target");
    let output = Command::new(JAIC)
        .arg("-check")
        .arg("-exe")
        .arg(dir.join("avr-hello"))
        .arg(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/compiler/targets/avr-hello.jai"),
        )
        .args(["-target", "avr-none-elf"])
        .arg("+Extensions/Embedded/Boards/Arduino_Uno")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// A string builder holds 4 KiB, or 256 bytes where pointers are 2 bytes wide, so a builder on a
/// 2 KiB part's stack fits. `avr-hello.jai` asserts the 256 bytes.
#[test]
fn a_string_builder_is_sized_by_the_pointer_width() {
    let source =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/compiler/targets/builder-size.jai");
    for target in [
        "x86_64-unknown-linux-gnu",
        "i686-unknown-linux-gnu",
        "wasm32-unknown-unknown",
    ] {
        let output = Command::new(JAIC)
            .arg("-check")
            .arg(&source)
            .args(["-target", target])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{target}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

/// A pointer written to a stack slot is as wide as the target's, so it leaves the next member
/// alone on a 4-byte-pointer target.
#[test]
fn a_pointer_stored_in_a_frame_slot_has_the_targets_width() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/compiler/targets/frame-pointer-width.jai");
    for target in [
        "i686-unknown-linux-gnu",
        "armv7-unknown-linux-gnueabihf",
        "wasm32-wasi",
    ] {
        assert_eq!(run(target, &source), "aabbccdd true\n", "{target}");
    }
}

/// Atomic loads, stores, exchanges and read-modify-writes at compile time use the program's
/// byte order: a big-endian program's counters count.
#[test]
fn atomics_use_the_targets_byte_order() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/compiler/targets/atomics-order.jai");
    for target in [
        "x86_64-unknown-linux-gnu",
        "s390x-unknown-linux-gnu",
        "powerpc64-unknown-linux-gnu",
    ] {
        assert_eq!(
            run(target, &source),
            "1ff 200 1ff fff f f false 1ff true 7 9\n\
             102ffff 1030000 102ffff 102ffff 102f00f 102f00f false 102ffff true 7 9\n\
             1020304ffffffff 102030500000000 1020304ffffffff 1020304ffffffff 1020304fffff00f \
             1020304fffff00f false 1020304ffffffff true 7 9\n",
            "{target}"
        );
    }
}

/// The emulated C library fills `struct stat`, `struct tm`, `struct timespec` and an `strto*`
/// end pointer at the program's own widths, alignments and byte order, so a narrow or
/// big-endian program reads back what a native one would.
#[test]
fn the_emulated_c_library_writes_records_at_the_targets_layout() {
    let source =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/compiler/targets/libc-records.jai");
    for target in [
        "i686-unknown-linux-gnu",
        "armv7-unknown-linux-gnueabihf",
        "s390x-unknown-linux-gnu",
        "powerpc64-unknown-linux-gnu",
    ] {
        assert_eq!(
            run(target, &source),
            "file 0 11 true\ndirectory 0 true\ndated true\nlisted true\nstrtod 12.5 4 aabbccdd\n\
             strtol 77 aabbccdd\nclock true true\ntm 71 0 1\n",
            "{target}"
        );
    }
}

/// Integers cast to pointers at compile time stay integers in a 32-bit program: its
/// compile-time memory ends before them, so they are not frozen as addresses.
#[test]
fn integer_handles_made_at_compile_time_keep_their_values() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/compiler/targets/integer-handles.jai");
    for target in [
        "x86_64-unknown-linux-gnu",
        "i686-unknown-linux-gnu",
        "armv7-unknown-linux-gnueabihf",
        "wasm32-wasi",
    ] {
        assert_eq!(run(target, &source), "true true true true\n", "{target}");
    }
}

/// Pointer arithmetic and integers cast to pointers wrap at the target's pointer width, and a
/// procedure value read back from a struct equals the inline constant it was set from.
#[test]
fn pointers_wrap_at_the_targets_width() {
    let source =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/compiler/targets/pointer-wrap.jai");
    let wide = "18446744073709551600 16 16 18446744073709551600\ntrue 18446744073709551615\n";
    let narrow = "4294967280 16 16 4294967280\ntrue 4294967295\n";
    for (target, expected) in [
        ("x86_64-unknown-linux-gnu", wide),
        ("s390x-unknown-linux-gnu", wide),
        ("i686-unknown-linux-gnu", narrow),
        ("armv7-unknown-linux-gnueabihf", narrow),
        ("wasm32-wasi", narrow),
    ] {
        assert_eq!(run(target, &source), expected, "{target}");
    }
}

/// A compile-time fault in a narrow program names the address the program used, not where
/// the interpreter keeps that memory.
#[test]
fn a_compile_time_fault_names_the_programs_address() {
    let source =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/compiler/targets/wild-read.jai");
    for target in ["x86_64-unknown-linux-gnu", "i686-unknown-linux-gnu"] {
        let output = Command::new(JAIC)
            .arg("-check")
            .arg(&source)
            .args(["-target", target])
            .output()
            .unwrap();
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("read from address 0x80000000 faulted"),
            "{target}: {stderr}"
        );
    }
}
