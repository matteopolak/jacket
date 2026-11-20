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

/// Writes the fixture project into `dir`: the program with `exit_code`, and a metaprogram whose
/// linker arguments are `linker_arguments`. `start.o` is assembled when `clang` is given.
fn project(dir: &TempDir, exit_code: u8, linker_arguments: &str, clang: Option<&Path>) {
    std::fs::write(
        dir.join("prog.jai"),
        fixture("prog.jai").replace("EXIT_CODE", &exit_code.to_string()),
    )
    .unwrap();
    std::fs::write(
        dir.join("build.jai"),
        fixture("build.jai").replace("LINKER_ARGUMENTS", linker_arguments),
    )
    .unwrap();
    std::fs::write(dir.join("board.ld"), fixture("board.ld")).unwrap();
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
        project(&dir, code, SCRIPT, Some(&clang));
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
    project(&dir, 0, SCRIPT, Some(&clang));
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
    project(&dir, 0, "", None);
    let built = build(&dir);
    let stderr = String::from_utf8_lossy(&built.stderr);
    assert!(!built.status.success());
    assert!(stderr.contains("needs a linker script"), "{stderr}");
    assert!(stderr.contains("-T <script>"), "{stderr}");
}

#[test]
fn a_missing_lld_fails_with_an_install_hint_and_no_other_linker_runs() {
    let dir = TempDir::new(&format!("freestanding-lld-{}", std::process::id()));
    project(&dir, 0, SCRIPT, None);
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
    project(&dir, 0, SCRIPT, None);
    std::fs::write(
        dir.join("prog.jai"),
        "libc :: #system_library \"libc\";\nputs :: (text: *u8) -> s32 #foreign libc;\n\
         #program_export \"jai_start\"\njai_start :: () #c_call #no_context { puts(\"hi\"); }\n",
    )
    .unwrap();
    let built = build(&dir);
    let stderr = String::from_utf8_lossy(&built.stderr);
    assert!(!built.status.success());
    assert!(stderr.contains("`#library \"libc\"`"), "{stderr}");
    assert!(stderr.contains("no libc"), "{stderr}");
}

#[test]
fn a_system_entry_point_is_refused_on_a_target_with_no_system() {
    let dir = TempDir::new(&format!("freestanding-entry-{}", std::process::id()));
    project(&dir, 0, SCRIPT, None);
    let build_file = fixture("build.jai")
        .replace("LINKER_ARGUMENTS", SCRIPT)
        .replace(".OMIT", ".ENTRY_POINT_AND_INIT");
    std::fs::write(dir.join("build.jai"), build_file).unwrap();
    let built = build(&dir);
    let stderr = String::from_utf8_lossy(&built.stderr);
    assert!(!built.status.success());
    assert!(stderr.contains("has none"), "{stderr}");
    assert!(stderr.contains(".ONLY_INIT"), "{stderr}");
}
