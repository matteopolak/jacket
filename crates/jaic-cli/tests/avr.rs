//! Programs for the Arduino Uno (`ATmega328P`, 2-byte pointers) built with
//! `Extensions/Embedded/Boards/Arduino_Uno` and run under `qemu-system-avr`
//! (docs/native/embedded.md#avr-arduino-uno).
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use test_support::{TempDir, toolchain};

const JAIC: &str = env!("CARGO_BIN_EXE_jaic");
const BOARD: &str = "+Extensions/Embedded/Boards/Arduino_Uno";

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/compiler/targets")
        .join(name)
}

/// Runs `jaic` on a fixture with `flags` before the source (a board's arguments come after its
/// name, so every compiler flag goes first), and returns its stderr. `None`
/// (after a note) when this host cannot build for the Uno and the test may skip: the LLVM jaic
/// links has no AVR backend, or `ld.lld` or the compiler-rt builtins are missing. On CI each is
/// a failure.
fn build(
    dir: &TempDir,
    source: &str,
    flags: &[&str],
    output: &str,
) -> Option<std::process::Output> {
    if !toolchain::buildable("avr-none-elf") {
        return None;
    }

    let built = Command::new(JAIC)
        .args(flags)
        .args(["-exe", output])
        .arg(fixture(source))
        .arg(BOARD)
        .current_dir(dir.path())
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&built.stderr);

    if stderr.contains("has no backend for") && !toolchain::strict() {
        toolchain::skip(&stderr);
        return None;
    }

    Some(built)
}

/// Runs `elf` on the emulator's Uno until the program has written `expected` (it then loops for
/// ever, so a run never ends by itself), or fails after 20 seconds. Returns what it wrote.
fn run_on_uno(qemu: &Path, elf: &Path, expected: &str) -> String {
    let mut child = Command::new(qemu)
        .args([
            "-M",
            "uno",
            "-nographic",
            "-serial",
            "stdio",
            "-monitor",
            "none",
        ])
        .arg("-bios")
        .arg(elf)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut stdout = child.stdout.take().unwrap();
    let (send, receive) = mpsc::channel();

    std::thread::spawn(move || {
        let mut byte = [0u8; 1];

        while stdout.read(&mut byte).is_ok_and(|n| n == 1) {
            if send.send(byte[0]).is_err() {
                break;
            }
        }
    });
    let started = Instant::now();
    let mut text = String::new();

    while !text.ends_with(expected) && started.elapsed() < Duration::from_secs(20) {
        if let Ok(byte) = receive.recv_timeout(Duration::from_millis(100)) {
            text.push(char::from(byte));
        }
    }

    child.kill().unwrap();
    child.wait().unwrap();
    text
}

/// The program is built and run at each optimization level the board allows: the board builds
/// small whatever the command line asks, because an unoptimized image does not fit the part.
fn run_program(source: &str, expected: &str) {
    let Some(qemu) = toolchain::optional("qemu-system-avr") else {
        return;
    };

    for flags in [&[][..], &["-O0"], &["-O2"]] {
        let dir = TempDir::new(&format!("avr-{}-{}", source.replace('.', "-"), flags.len()));
        let Some(built) = build(&dir, source, flags, "prog.elf") else {
            return;
        };

        assert!(
            built.status.success(),
            "{flags:?}: {}",
            String::from_utf8_lossy(&built.stderr)
        );
        assert_eq!(
            run_on_uno(&qemu, &dir.join("prog.elf"), expected),
            expected,
            "{flags:?}"
        );
    }
}

/// `print("hello\n")` prints on the Uno's serial port.
#[test]
fn hello_runs_on_an_uno() {
    run_program("avr-hello.jai", "hello\n");
}

/// Calls through code pointers (`#c_call` and Jai procedures, from a table in RAM and from a
/// variable) reach the right procedure: the pointers are word addresses in program memory.
#[test]
fn calls_through_code_pointers_run_on_an_uno() {
    run_program("avr-callbacks.jai", "abc\nok\n");
}

/// The image fits the part: 32 KiB of flash, and `.data` and `.bss` leave room for a stack in
/// 2 KiB of RAM.
#[test]
fn the_hello_image_fits_the_atmega328p() {
    let Some(size) = toolchain::optional("llvm-size") else {
        return;
    };
    let dir = TempDir::new("avr-size");
    let Some(built) = build(&dir, "avr-hello.jai", &[], "prog.elf") else {
        return;
    };

    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    let out = Command::new(size)
        .arg(dir.join("prog.elf"))
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    let numbers: Vec<u64> = text
        .lines()
        .nth(1)
        .unwrap()
        .split_whitespace()
        .take(3)
        .map(|n| n.parse().unwrap())
        .collect();
    let (code, data, bss) = (numbers[0], numbers[1], numbers[2]);

    assert!(code + data <= 32 * 1024, "{text}");
    assert!(data + bss + 256 <= 2048, "{text}");
}

/// Code pointers are lowered through program memory: a pointer to a function is the word address
/// cast from address space 1, and a call through one casts it back.
#[test]
fn code_pointers_lower_through_address_space_1() {
    let dir = TempDir::new("avr-ir");
    let built = Command::new(JAIC)
        .args(["-emit_ir", "prog.ll", "-exe", "prog.elf"])
        .arg(fixture("avr-callbacks.jai"))
        .arg(BOARD)
        .current_dir(dir.path())
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&built.stderr);

    if stderr.contains("has no backend for") && !toolchain::strict() {
        toolchain::skip(&stderr);
        return;
    }

    let ir = std::fs::read_to_string(dir.join("prog.ll")).unwrap_or_else(|_| panic!("{stderr}"));

    assert!(
        ir.contains("addrspacecast (ptr addrspace(1) @step_a"),
        "a procedure's address is cast from program memory:\n{ir}"
    );
    let casts_back = ir
        .lines()
        .any(|line| line.contains("= addrspacecast ptr %") && line.contains("to ptr addrspace(1)"));
    let calls_it = ir.lines().any(|line| {
        line.trim_start().starts_with("call addrspace(1)")
            && line.contains(" %")
            && line.contains('(')
            && !line.contains('@')
    });

    assert!(
        casts_back && calls_it,
        "an indirect call goes through program memory"
    );
}

/// `JAIC_COMPILER_RT` naming something that is not there is an error that says so, not a link
/// failure on a missing file.
#[test]
fn a_compiler_rt_path_that_does_not_exist_is_named() {
    if !toolchain::buildable("avr-none-elf") {
        return;
    }

    let dir = TempDir::new("avr-builtins-missing");
    let built = Command::new(JAIC)
        .args(["-exe", "prog.elf"])
        .arg(fixture("avr-hello.jai"))
        .arg(BOARD)
        .env("JAIC_COMPILER_RT", dir.join("none"))
        .current_dir(dir.path())
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&built.stderr);

    if stderr.contains("has no backend for") && !toolchain::strict() {
        toolchain::skip(&stderr);
        return;
    }

    assert!(!built.status.success(), "{stderr}");
    assert!(stderr.contains("JAIC_COMPILER_RT names"), "{stderr}");
    assert!(stderr.contains("which does not exist"), "{stderr}");
}
