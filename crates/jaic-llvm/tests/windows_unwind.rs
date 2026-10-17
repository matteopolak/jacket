//! Required Windows ABI unwind tables survive optimization and PE linking.
mod common;
use common::checked;
use jaic::sema::{Compiler, Options, ProgramSource, TargetCpu, TargetOs, VirtualFs};
use jaic_llvm::{Codegen, OptLevel, emit_object};
use std::{
    path::{Path, PathBuf},
    process::Command,
    rc::Rc,
};

const SOURCE: &str = include_str!("../../../tests/compiler-semantics/windows-unwind.jai");

fn emit(triple: &str, cpu: TargetCpu, level: OptLevel, dir: &Path) -> PathBuf {
    std::fs::create_dir_all(dir).unwrap();
    let mut options = Options::host();
    options.os = TargetOs::Windows;
    options.cpu = cpu;
    options.runtime_support = false;
    options.stack_trace = false;
    let mut compiler = Compiler::new(options, Rc::new(VirtualFs::default()));
    if let Err(error) = compiler.compile_sources(&[ProgramSource::String(SOURCE.into())]) {
        panic!("{}", error.render(&compiler.sources));
    }
    let object = dir.join("frames.obj");
    emit_object(
        &compiler.program,
        &jaic_llvm::Options {
            target: Some(jaic::target::Target::parse(triple).unwrap()),
            opt_level: level,
            codegen: Codegen {
                frame_pointers: Some(false),
                single_unit: true,
                ir_after: Some(dir.join("optimized.ll")),
                ..Codegen::default()
            },
            ..Default::default()
        },
        &object,
    )
    .unwrap();
    object
}

fn hex_address(text: &str) -> u64 {
    let digits = text.split_once("0x").expect("LLVM hexadecimal address").1;
    let end = digits
        .find(|c: char| !c.is_ascii_hexdigit())
        .unwrap_or(digits.len());
    u64::from_str_radix(&digits[..end], 16).unwrap()
}

fn check_linked_entries(text: &str) {
    let image_base = hex_address(
        text.lines()
            .find(|l| l.trim_start().starts_with("ImageBase:"))
            .unwrap(),
    );
    for name in [
        "unwind_leaf",
        "unwind_middle",
        "unwind_entry",
        "unwind_capture_leaf",
        "unwind_capture_middle",
    ] {
        let export = text
            .split("Export {")
            .find(|part| part.lines().any(|l| l.trim() == format!("Name: {name}")))
            .expect("named export");
        let rva = hex_address(
            export
                .lines()
                .find(|l| l.trim_start().starts_with("RVA:"))
                .unwrap(),
        );
        let address = image_base + rva;
        assert!(
            text.split("RuntimeFunction {").skip(1).any(|part| {
                part.lines()
                    .find(|l| {
                        let line = l.trim_start();
                        line.starts_with("StartAddress:") || line.starts_with("Function:")
                    })
                    .is_some_and(|l| hex_address(l) == address)
            }),
            "{name} has no linked runtime function entry at {address:#x}: {text}"
        );
    }
}

#[test]
fn windows_nonleaf_frames_have_unwind_entries_at_o0_and_o2() {
    for (triple, cpu, machine) in [
        ("x86_64-pc-windows-msvc", TargetCpu::X64, "X64"),
        ("x86_64-w64-windows-gnu", TargetCpu::X64, "X64"),
        ("aarch64-pc-windows-msvc", TargetCpu::Arm64, "ARM64"),
    ] {
        for level in [OptLevel::O0, OptLevel::O2] {
            let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
                .join(format!("windows-unwind/{triple}-{level:?}"));
            let object = emit(triple, cpu, level, &dir);
            let unwind = checked(Command::new("llvm-readobj").arg("--unwind").arg(&object));
            std::fs::write(dir.join("object-unwind.txt"), &unwind).unwrap();
            for name in ["unwind_leaf", "unwind_middle", "unwind_entry"] {
                assert!(
                    unwind.contains(name),
                    "{triple} {level:?}: missing {name}: {unwind}"
                );
            }
            let disassembly = checked(Command::new("llvm-objdump").arg("-d").arg(&object));
            std::fs::write(dir.join("disassembly.txt"), disassembly).unwrap();
            let pe = dir.join("frames.exe");
            checked(
                Command::new("lld-link")
                    .arg("/entry:unwind_entry")
                    .args(["/subsystem:console", "/nodefaultlib"])
                    .arg(format!("/machine:{machine}"))
                    .arg(format!("/out:{}", pe.display()))
                    .arg(&object),
            );
            let linked = checked(
                Command::new("llvm-readobj")
                    .args(["--unwind", "--coff-exports", "--file-headers"])
                    .arg(&pe),
            );
            std::fs::write(dir.join("pe-unwind.txt"), &linked).unwrap();
            check_linked_entries(&linked);
            let ir = std::fs::read_to_string(dir.join("optimized.ll")).unwrap();
            assert!(
                ir.contains("uwtable") && ir.contains("!\"uwtable\", i32 2"),
                "{ir}"
            );
            assert!(ir.contains("\"frame-pointer\"=\"none\""), "{ir}");
            if level == OptLevel::O2 {
                assert!(
                    ir.contains("nounwind"),
                    "optimizer must retain valid inference: {ir}"
                );
            }
        }
    }
}

#[test]
fn sdk_backtrace_driver_links_and_executes_on_native_windows() {
    // Off Windows the driver is linked with MinGW-w64 (and run, where the host can).
    if !cfg!(windows)
        && Command::new("x86_64-w64-mingw32-gcc")
            .arg("--version")
            .output()
            .is_err()
    {
        eprintln!("skipped: x86_64-w64-mingw32-gcc is not installed");
        return;
    }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let native_arm = cfg!(all(windows, target_arch = "aarch64"));
    let (triple, cpu) = if native_arm {
        ("aarch64-pc-windows-msvc", TargetCpu::Arm64)
    } else if cfg!(windows) {
        ("x86_64-pc-windows-msvc", TargetCpu::X64)
    } else {
        ("x86_64-w64-windows-gnu", TargetCpu::X64)
    };
    for level in [OptLevel::O0, OptLevel::O2] {
        let dir = root.join(format!("target/windows-unwind/sdk-{triple}-{level:?}"));
        let object = emit(triple, cpu, level, &dir);
        let exe = dir.join("backtrace.exe");
        checked(
            Command::new(if cfg!(windows) {
                "clang"
            } else {
                "x86_64-w64-mingw32-gcc"
            })
            .args(["-Wall", "-Wextra", "-Werror"])
            .arg(if level == OptLevel::O0 {
                "-O0"
            } else {
                "-O2"
            })
            .arg(root.join("tests/compiler-semantics/windows-unwind-driver.c"))
            .arg(object)
            .arg("-o")
            .arg(&exe),
        );
        if cfg!(windows) {
            assert_eq!(
                checked(&mut Command::new(exe)).trim(),
                "real Windows frames and unwind tables: ok"
            );
        }
    }
}
