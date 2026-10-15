//! Explicit architectural opcodes and actual CPUID-guarded native x64 execution.
use jaic::sema::{Compiler, Options, ProgramSource, TargetCpu, VirtualFs};
use jaic_llvm::{OptLevel, emit_object};
use std::{path::PathBuf, process::Command, rc::Rc};

fn checked(command: &mut Command) -> String {
    let output = command
        .output()
        .expect("required native ISA regression tool");
    assert!(
        output.status.success(),
        "{command:?}: status {:?}\n{}\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn opcodes_and_architecture_rejection_survive_optimization() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut options = Options::host();
    options.cpu = TargetCpu::X64;
    options.runtime_support = false;
    options.stack_trace = false;
    let mut c = Compiler::new(options, Rc::new(VirtualFs::default()));
    if let Err(e) = c.compile_sources(&[ProgramSource::String(
        include_str!("../../../tests/compiler-semantics/x86-machine.jai").into(),
    )]) {
        panic!("{}", e.render(&c.sources));
    }
    for target in [
        "x86_64-apple-darwin",
        "x86_64-unknown-linux-gnu",
        "x86_64-pc-windows-msvc",
        "x86_64-pc-windows-gnu",
    ] {
        for level in [OptLevel::O0, OptLevel::O2] {
            let dir = root.join(format!("target/x86-machine/{target}-{level:?}"));
            std::fs::create_dir_all(&dir).unwrap();
            let object = dir.join("machine.o");
            let ir = dir.join("machine.ll");
            emit_object(
                &c.program,
                &jaic_llvm::Options {
                    target: Some(target.into()),
                    opt_level: level,
                    emit_ir: Some(ir.clone()),
                    ..Default::default()
                },
                &object,
            )
            .unwrap();
            let disassembly = checked(Command::new("llvm-objdump").arg("-d").arg(&object));
            for instruction in [
                "rdtsc",
                "rdtscp",
                "rdrandw",
                "rdrandl",
                "rdrandq",
                "rdseedw",
                "rdseedl",
                "rdseedq",
                "mfence",
                "lfence",
                "sfence",
                "pause",
                "prefetcht0",
                "prefetcht1",
                "prefetcht2",
                "prefetchnta",
            ] {
                assert!(
                    disassembly
                        .lines()
                        .any(|line| line.split_whitespace().any(|word| word == instruction)),
                    "{target} {level:?} missing {instruction}: {disassembly}"
                );
            }
            let ir = std::fs::read_to_string(ir).unwrap();
            for instruction in ["mfence", "lfence", "sfence", "rdtsc", "rdtscp"] {
                assert!(
                    ir.lines()
                        .any(|line| line.contains(&format!("\"{instruction}\""))
                            && line.contains("asm sideeffect")
                            && line.contains("~{memory}")),
                    "{instruction} must retain side effects and a compiler memory clobber"
                );
            }
            #[cfg(any(target_arch = "x86_64", target_os = "macos"))]
            {
                let native = if cfg!(target_os = "macos") {
                    "x86_64-apple-darwin"
                } else if cfg!(windows) {
                    "x86_64-pc-windows-msvc"
                } else {
                    "x86_64-unknown-linux-gnu"
                };
                if target == native {
                    let exe = dir.join(format!("isa-driver{}", std::env::consts::EXE_SUFFIX));
                    let mut clang = Command::new("clang");
                    if cfg!(target_os = "macos") {
                        clang.args(["-arch", "x86_64"]);
                    }
                    checked(
                        clang
                            .args(["-std=c11", "-Wall", "-Wextra", "-Werror"])
                            .arg(root.join("tests/compiler-semantics/x86-machine-driver.c"))
                            .arg(&object)
                            .arg("-o")
                            .arg(&exe),
                    );
                    let evidence = checked(&mut Command::new(exe));
                    std::fs::write(dir.join("executed.txt"), &evidence).unwrap();
                    println!("{target} {level:?}: {evidence}");
                }
            }
        }
    }
    for target in [
        "aarch64-apple-darwin",
        "aarch64-pc-windows-msvc",
        "wasm64-unknown-unknown",
    ] {
        let error = emit_object(
            &c.program,
            &jaic_llvm::Options {
                target: Some(target.into()),
                ..Default::default()
            },
            &root.join("target/x86-machine/rejected.o"),
        )
        .expect_err("architectural intrinsics must not silently disappear on another backend");
        assert!(error.contains("x86-64 target"), "{error}");
    }
}
