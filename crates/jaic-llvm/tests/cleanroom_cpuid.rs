//! Real x64 instruction execution, compared to independent native C SDK queries.
use jaic::sema::{Compiler, Options, ProgramSource, TargetCpu, VirtualFs};
use jaic_llvm::{OptLevel, emit_object};
use std::{path::PathBuf, process::Command, rc::Rc};

fn checked(command: &mut Command) -> String {
    let output = command
        .output()
        .expect("required source-built backend test command");
    assert!(
        output.status.success(),
        "{command:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

// macOS ARM64 supports executing independently built x64 programs under translation.
// Other ARM64 hosts receive metadata/target-diagnostic coverage below, not x64 execution claims.
#[cfg(any(target_arch = "x86_64", target_os = "macos"))]
#[test]
fn cpuid_native_outputs_match_c_at_o0_and_o2() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut options = Options::host();
    options.cpu = TargetCpu::X64;
    options.runtime_support = false;
    let mut c = Compiler::new(options, Rc::new(VirtualFs::default()));
    if let Err(e) = c.compile_sources(&[ProgramSource::String(
        include_str!("../../../tests/compiler-semantics/cpuid.jai").into(),
    )]) {
        panic!("{}", e.render(&c.sources));
    }
    let target = if cfg!(target_os = "macos") {
        "x86_64-apple-darwin"
    } else if cfg!(windows) {
        "x86_64-pc-windows-msvc"
    } else {
        "x86_64-unknown-linux-gnu"
    };
    for level in [OptLevel::O0, OptLevel::O2] {
        let dir = root.join(format!("target/cpuid/{level:?}"));
        std::fs::create_dir_all(&dir).unwrap();
        let object = dir.join("cpuid.o");
        emit_object(
            &c.program,
            &jaic_llvm::Options {
                target: Some(target.into()),
                opt_level: level,
                ..Default::default()
            },
            &object,
        )
        .unwrap();
        let exe = dir.join(format!("cpuid{}", std::env::consts::EXE_SUFFIX));
        let mut driver = Command::new("clang");
        if cfg!(target_os = "macos") {
            driver.args(["-arch", "x86_64"]);
        }
        driver
            .args(["-std=c11", "-Wall", "-Wextra", "-Werror"])
            .arg(if level == OptLevel::O0 {
                "-O0"
            } else {
                "-O2"
            })
            .arg(root.join("tests/compiler-semantics/cpuid-driver.c"))
            .arg(object)
            .arg("-o")
            .arg(&exe);
        checked(&mut driver);
        assert!(checked(&mut Command::new(exe)).contains("CPUID hardware registers"));
    }
}

#[test]
fn cpuid_emits_real_instruction_for_x64_abis_and_rejects_other_architectures() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut options = Options::host();
    options.cpu = TargetCpu::X64;
    options.runtime_support = false;
    let mut c = Compiler::new(options, Rc::new(VirtualFs::default()));
    c.compile_sources(&[ProgramSource::String(
        include_str!("../../../tests/compiler-semantics/cpuid.jai").into(),
    )])
    .unwrap();
    let dir = root.join("target/cpuid/metadata");
    std::fs::create_dir_all(&dir).unwrap();
    for target in [
        "x86_64-apple-darwin",
        "x86_64-unknown-linux-gnu",
        "x86_64-pc-windows-msvc",
        "x86_64-pc-windows-gnu",
    ] {
        for level in [OptLevel::O0, OptLevel::O2] {
            let object = dir.join(format!("{target}-{level:?}.o"));
            emit_object(
                &c.program,
                &jaic_llvm::Options {
                    target: Some(target.into()),
                    opt_level: level,
                    ..Default::default()
                },
                &object,
            )
            .unwrap();
            let disassembly = checked(Command::new("llvm-objdump").arg("-d").arg(&object));
            assert!(
                disassembly
                    .lines()
                    .any(|line| line.split_whitespace().any(|word| word == "cpuid")),
                "{target} {level:?}: {disassembly}"
            );
            if target.contains("windows") {
                let unwind = checked(Command::new("llvm-readobj").arg("--unwind").arg(&object));
                assert!(
                    unwind.contains("RBX"),
                    "CPUID clobbers callee-saved RBX: {unwind}"
                );
            }
        }
    }
    for target in [
        "wasm64-unknown-unknown",
        "aarch64-unknown-linux-gnu",
        "aarch64-pc-windows-msvc",
    ] {
        let error = emit_object(
            &c.program,
            &jaic_llvm::Options {
                target: Some(target.into()),
                ..Default::default()
            },
            &dir.join("rejected.o"),
        )
        .expect_err("CPUID requires a real x64 target");
        assert!(error.contains("x86-64 target"), "{target}: {error}");
    }
}
