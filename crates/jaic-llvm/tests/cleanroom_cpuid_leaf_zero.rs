//! O0/O2 CPUID codegen and native comparison against the public C intrinsic.
use jaic::sema::{Compiler, Options, ProgramSource, TargetCpu, VirtualFs};
use jaic_llvm::{OptLevel, emit_object};
use std::{path::PathBuf, process::Command, rc::Rc};

#[cfg(any(target_arch = "x86_64", target_os = "macos"))]
fn checked(command: &mut Command) {
    let out = command.output().expect("required CPUID regression tool");
    assert!(
        out.status.success(),
        "{command:?}: {:?}\n{}\n{}",
        out.status.code(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn cpuid_leaf_zero_survives_optimization_and_matches_native_intrinsic() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut options = Options::host();
    options.cpu = TargetCpu::X64;
    options.runtime_support = false;
    options.stack_trace = false;
    let mut c = Compiler::new(options, Rc::new(VirtualFs::default()));
    c.compile_sources(&[ProgramSource::String(
        include_str!("../../../tests/compiler-semantics/cpuid-leaf-zero.jai").into(),
    )])
    .unwrap();
    let target = if cfg!(target_os = "macos") {
        "x86_64-apple-darwin"
    } else if cfg!(windows) {
        "x86_64-pc-windows-msvc"
    } else {
        "x86_64-unknown-linux-gnu"
    };
    for level in [OptLevel::O0, OptLevel::O2] {
        let dir = root.join(format!("target/cpuid-leaf-zero/{level:?}"));
        std::fs::create_dir_all(&dir).unwrap();
        let object = dir.join("cpuid.o");
        let ir = dir.join("cpuid.ll");
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
        let ir = std::fs::read_to_string(ir).unwrap();
        assert!(ir.lines().any(|line| {
            line.contains("\"cpuid\"")
                && line.contains("asm sideeffect")
                && line.contains("~{memory}")
        }));
        let disassembly = Command::new("llvm-objdump")
            .arg("-d")
            .arg(&object)
            .output()
            .unwrap();
        assert!(disassembly.status.success());
        assert!(
            String::from_utf8(disassembly.stdout)
                .unwrap()
                .lines()
                .any(|line| line.split_whitespace().any(|word| word == "cpuid"))
        );
        #[cfg(any(target_arch = "x86_64", target_os = "macos"))]
        {
            let exe = dir.join(format!("cpuid-driver{}", std::env::consts::EXE_SUFFIX));
            let mut clang = Command::new("clang");
            if cfg!(target_os = "macos") {
                clang.args(["-arch", "x86_64"]);
            }
            checked(
                clang
                    .args(["-std=c11", "-Wall", "-Wextra", "-Werror"])
                    .arg(root.join("tests/compiler-semantics/cpuid-leaf-zero-driver.c"))
                    .arg(&object)
                    .arg("-o")
                    .arg(&exe),
            );
            checked(&mut Command::new(exe));
        }
    }
}
