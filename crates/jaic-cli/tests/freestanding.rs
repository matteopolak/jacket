//! `-os none` (`Operating_System_Tag.NONE`): freestanding aarch64 programs linked by `ld.lld` and
//! run on QEMU's `virt` machine, which reports the exit code through semihosting
//! (docs/native/embedded.md). Tests that need QEMU or Clang are skipped, with a note, when the
//! tool is missing, unless `JAIC_REQUIRE_TOOLCHAIN` names it (CI sets it).
#![cfg(feature = "llvm")]

use std::path::Path;
use std::process::{Command, Output};
use test_support::{TempDir, toolchain};

const JAIC: &str = env!("CARGO_BIN_EXE_jaic");

fn fixture(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/freestanding");
    std::fs::read_to_string(path.join(name)).unwrap()
}

/// How a fixture project is put together.
struct Setup<'a> {
    /// The program's source file in `tests/freestanding`.
    program: &'a str,
    /// The linker script, `ram.ld` or `flash.ld`.
    script: &'a str,
    /// A `Optimization_Type` member.
    optimization: &'a str,
    /// `OMIT` for the bare programs, `ONLY_INIT` for the ones on the board.
    runtime: &'a str,
    /// The `exit` code of `bare.jai`.
    exit_code: u8,
    /// Statements of the metaprogram that pass the linker script.
    linker_arguments: &'a str,
}

impl Default for Setup<'_> {
    fn default() -> Self {
        Self {
            program: "bare.jai",
            script: "ram.ld",
            optimization: "DEBUG",
            runtime: "OMIT",
            exit_code: 0,
            linker_arguments: SCRIPT,
        }
    }
}

/// Writes the fixture project into `dir`. `start.o` is assembled when `clang` is given.
fn project(dir: &TempDir, setup: &Setup, clang: Option<&Path>) {
    std::fs::write(
        dir.join("prog.jai"),
        fixture(setup.program).replace("EXIT_CODE", &setup.exit_code.to_string()),
    )
    .unwrap();
    let board = if setup.runtime == "OMIT" {
        ""
    } else {
        r#"add_build_file("board.jai", w);"#
    };
    std::fs::write(
        dir.join("build.jai"),
        fixture("build.jai")
            .replace("LINKER_ARGUMENTS", setup.linker_arguments)
            .replace("OPTIMIZATION", &format!(".{}", setup.optimization))
            .replace(".RUNTIME", &format!(".{}", setup.runtime))
            .replace(
                "    set_build_options(options, w);",
                &format!("    set_build_options(options, w);\n    {board}"),
            ),
    )
    .unwrap();
    std::fs::write(dir.join("board.jai"), fixture("board.jai")).unwrap();
    std::fs::write(dir.join("board.ld"), fixture(setup.script)).unwrap();
    std::fs::write(dir.join("start.S"), fixture("start.S")).unwrap();
    if let Some(clang) = clang {
        let status = Command::new(clang)
            .args([
                "--target=aarch64-none-elf",
                "-c",
                "start.S",
                "-o",
                "start.o",
            ])
            .current_dir(dir.path())
            .status()
            .unwrap();
        assert!(status.success(), "assembling start.S");
    }
}

fn build(dir: &TempDir) -> Output {
    Command::new(JAIC)
        .arg("build.jai")
        .current_dir(dir.path())
        .output()
        .unwrap()
}

/// The linker script made by `Embedded.link_for`: all RAM, or flash for the code and RAM for the data.
const RAM_MAP: &str =
    "Embedded.link_for(*options, Embedded.Memory_Map.{ ram = .{ 0x4000_0000, 0x100_0000 } });";

const FLASH_MAP: &str = "Embedded.link_for(*options, Embedded.Memory_Map.{ flash = .{ 0x4000_0000, 0x10_0000 }, ram = .{ 0x4010_0000, 0xf0_0000 } });";

const SCRIPT: &str = r#"options.additional_linker_arguments = string.["-T", "board.ld"];"#;

/// Whether `ld.lld` is installed, as jaic looks for it. When it is not, a link names it.
fn lld_missing(stderr: &str) -> bool {
    stderr.contains("ld.lld not found") || stderr.contains("could not find the linker")
}

#[test]
fn a_none_program_runs_on_qemu_and_exits_through_semihosting() {
    let Some(clang) = toolchain::require("clang") else {
        return;
    };
    let Some(qemu) = toolchain::optional("qemu-system-aarch64") else {
        return;
    };
    for code in [0, 7] {
        let dir = TempDir::new(&format!("freestanding-exit-{code}-{}", std::process::id()));
        project(
            &dir,
            &Setup {
                exit_code: code,
                ..Setup::default()
            },
            Some(&clang),
        );
        let built = build(&dir);
        let stderr = String::from_utf8_lossy(&built.stderr);
        if lld_missing(&stderr) && !toolchain::strict("ld.lld") {
            toolchain::skip(&stderr);
            return;
        }
        assert!(built.status.success(), "jaic build.jai failed:\n{stderr}");
        let run = Command::new(&qemu)
            .args(["-M", "virt", "-cpu", "cortex-a53", "-nographic"])
            .args(["-semihosting-config", "enable=on,target=native"])
            .arg("-kernel")
            .arg(dir.join("prog.elf"))
            .output()
            .unwrap();
        assert_eq!(run.status.code(), Some(i32::from(code)), "{run:?}");
    }
}

#[test]
fn the_output_is_a_static_aarch64_elf_executable() {
    let Some(clang) = toolchain::require("clang") else {
        return;
    };
    let dir = TempDir::new(&format!("freestanding-elf-{}", std::process::id()));
    project(&dir, &Setup::default(), Some(&clang));
    let built = build(&dir);
    let stderr = String::from_utf8_lossy(&built.stderr);
    if lld_missing(&stderr) && !toolchain::strict("ld.lld") {
        toolchain::skip(&stderr);
        return;
    }
    assert!(built.status.success(), "{stderr}");
    let bytes = std::fs::read(dir.join("prog.elf")).unwrap();
    assert_eq!(&bytes[..4], b"\x7fELF");
    assert_eq!(bytes[4], 2, "ELF64");
    assert_eq!(bytes[5], 1, "little-endian");
    assert_eq!(u16::from_le_bytes([bytes[16], bytes[17]]), 2, "ET_EXEC");
    assert_eq!(
        u16::from_le_bytes([bytes[18], bytes[19]]),
        183,
        "EM_AARCH64"
    );
    let entry = u64::from_le_bytes(bytes[24..32].try_into().unwrap());
    assert_eq!(entry, 0x4000_0000, "the script puts the reset code first");
}

#[test]
fn a_freestanding_link_without_a_linker_script_says_how_to_add_one() {
    let dir = TempDir::new(&format!("freestanding-script-{}", std::process::id()));
    project(
        &dir,
        &Setup {
            linker_arguments: "",
            ..Setup::default()
        },
        None,
    );
    let built = build(&dir);
    let stderr = String::from_utf8_lossy(&built.stderr);
    assert!(!built.status.success());
    assert!(stderr.contains("needs a linker script"), "{stderr}");
    assert!(stderr.contains("-T <script>"), "{stderr}");
}

#[test]
fn a_missing_lld_fails_with_an_install_hint_and_no_other_linker_runs() {
    let dir = TempDir::new(&format!("freestanding-lld-{}", std::process::id()));
    project(&dir, &Setup::default(), None);
    std::fs::write(
        dir.join("prog.jai"),
        "#program_export \"jai_start\"\njai_start :: () #c_call #no_context {}\n",
    )
    .unwrap();
    let built = Command::new(JAIC)
        .arg("build.jai")
        .env("JAIC_LLD", dir.join("no-such-lld"))
        .current_dir(dir.path())
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&built.stderr);
    assert!(!built.status.success());
    assert!(stderr.contains("could not find the linker"), "{stderr}");
    assert!(stderr.contains("install LLD"), "{stderr}");
    assert!(!dir.join("prog.elf").exists());
}

#[test]
fn a_library_that_needs_an_os_is_a_link_error_naming_it() {
    let dir = TempDir::new(&format!("freestanding-libc-{}", std::process::id()));
    project(&dir, &Setup::default(), None);
    std::fs::write(
        dir.join("prog.jai"),
        "libc :: #system_library \"libc\";\nputs :: (text: *u8) -> s32 #foreign libc;\n\
         #program_export \"jai_start\"\njai_start :: () #c_call #no_context { puts(\"hi\"); }\n",
    )
    .unwrap();
    let built = build(&dir);
    let stderr = String::from_utf8_lossy(&built.stderr);
    assert!(!built.status.success());
    assert!(stderr.contains("cannot call `puts`"), "{stderr}");
    assert!(stderr.contains("no operating system"), "{stderr}");
}

#[test]
fn a_system_entry_point_is_refused_on_a_target_with_no_system() {
    let dir = TempDir::new(&format!("freestanding-entry-{}", std::process::id()));
    project(
        &dir,
        &Setup {
            runtime: "ENTRY_POINT_AND_INIT",
            ..Setup::default()
        },
        None,
    );
    let built = build(&dir);
    let stderr = String::from_utf8_lossy(&built.stderr);
    assert!(!built.status.success());
    assert!(stderr.contains("has none"), "{stderr}");
    assert!(stderr.contains(".ONLY_INIT"), "{stderr}");
}

/// Builds `setup` and runs it on QEMU. `None` when Clang, LLD or QEMU is missing and not required.
fn run(name: &str, setup: &Setup) -> Option<Output> {
    let clang = toolchain::require("clang")?;
    let qemu = toolchain::optional("qemu-system-aarch64")?;
    let dir = TempDir::new(&format!("freestanding-{name}-{}", std::process::id()));
    project(&dir, setup, Some(&clang));
    let built = build(&dir);
    let stderr = String::from_utf8_lossy(&built.stderr);
    if lld_missing(&stderr) && !toolchain::strict("ld.lld") {
        toolchain::skip(&stderr);
        return None;
    }
    assert!(
        built.status.success(),
        "{name}: jaic build.jai failed:\n{stderr}"
    );
    Some(qemu_run(&qemu, &dir.join("prog.elf")))
}

/// The console text of a run: QEMU's semihosting console writes to standard error.
fn console(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

/// Runs `elf` on QEMU's `virt` machine, killing it after 30 seconds: a program that never exits
/// is a failure, not a hung test run.
fn qemu_run(qemu: &Path, elf: &Path) -> Output {
    let mut child = Command::new(qemu)
        .args(["-M", "virt", "-cpu", "cortex-a53", "-nographic"])
        .args(["-semihosting-config", "enable=on,target=native"])
        .arg("-kernel")
        .arg(elf)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let started = std::time::Instant::now();
    while child.try_wait().unwrap().is_none() {
        if started.elapsed() > std::time::Duration::from_secs(30) {
            child.kill().unwrap();
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    child.wait_with_output().unwrap()
}

fn board(program: &str, optimization: &str) -> Setup<'static> {
    Setup {
        program: Box::leak(program.to_string().into_boxed_str()),
        optimization: Box::leak(optimization.to_string().into_boxed_str()),
        runtime: "ONLY_INIT",
        ..Setup::default()
    }
}

#[test]
fn basic_print_and_temporary_storage_work_with_no_operating_system() {
    for optimization in ["DEBUG", "OPTIMIZED"] {
        let Some(out) = run("print", &board("print.jai", optimization)) else {
            return;
        };
        assert_eq!(out.status.code(), Some(0), "{optimization}: {out:?}");
        assert_eq!(
            console(&out),
            "hello 42 world\n1-2\n3.5\n244837814094590\n",
            "{optimization}"
        );
    }
}

#[test]
fn growing_an_array_with_no_heap_reaches_the_panic_hook() {
    let Some(out) = run("heap", &board("heap.jai", "DEBUG")) else {
        return;
    };
    assert_eq!(out.status.code(), Some(70), "{out:?}");
    let stdout = console(&out);
    assert!(stdout.starts_with("before\n"), "{stdout}");
    assert!(
        stdout.contains("panic: allocation with no heap"),
        "{stdout}"
    );
    assert!(!stdout.contains("not reached"), "{stdout}");
}

#[test]
fn a_failed_assert_reports_and_panics() {
    let Some(out) = run("assert", &board("assertion.jai", "DEBUG")) else {
        return;
    };
    assert_eq!(out.status.code(), Some(70), "{out:?}");
    let stdout = console(&out);
    assert!(stdout.contains("assertion failed: value was 3"), "{stdout}");
    assert!(stdout.contains("panic: assertion failed"), "{stdout}");
}

#[test]
fn the_runtime_memory_functions_are_correct_at_every_optimization_level() {
    let expected = "copy true\nmove up true\nmove down true\nset true true\n\
                    struct 2 4\ncmp true true\n";
    for optimization in ["DEBUG", "OPTIMIZED", "VERY_OPTIMIZED"] {
        let Some(out) = run("memory", &board("memory.jai", optimization)) else {
            return;
        };
        assert_eq!(out.status.code(), Some(0), "{optimization}: {out:?}");
        assert_eq!(console(&out), expected, "{optimization}");
    }
}

#[test]
fn init_runtime_copies_data_to_ram_and_zeroes_bss() {
    for script in ["ram.ld", "flash.ld"] {
        let setup = Setup {
            script,
            ..board("data.jai", "DEBUG")
        };
        let Some(out) = run("data", &setup) else {
            return;
        };
        assert_eq!(out.status.code(), Some(0), "{script}: {out:?}");
        assert_eq!(console(&out), "12648430 30 freestanding 0\n5\n", "{script}");
    }
}

#[test]
fn link_for_writes_a_script_for_a_ram_map_and_a_flash_map() {
    for (name, map) in [("ram", RAM_MAP), ("flash", FLASH_MAP)] {
        for program in ["data.jai", "print.jai"] {
            let setup = Setup {
                linker_arguments: map,
                ..board(program, "DEBUG")
            };
            let Some(out) = run(&format!("map-{name}-{program}"), &setup) else {
                return;
            };
            assert_eq!(out.status.code(), Some(0), "{name} {program}: {out:?}");
        }
    }
}

#[test]
fn volatile_barriers_and_sections_work_on_the_target() {
    for optimization in ["DEBUG", "OPTIMIZED"] {
        let setup = Setup {
            linker_arguments: RAM_MAP,
            ..board("devices.jai", optimization)
        };
        let Some(out) = run("devices", &setup) else {
            return;
        };
        assert_eq!(out.status.code(), Some(0), "{optimization}: {out:?}");
        assert_eq!(console(&out), "42 8\n", "{optimization}");
    }
}
