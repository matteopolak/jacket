//! Real XGETBV object code across x64 ABIs, plus CPUID-guarded native controls.
use jaic::sema::{Compiler, Options, ProgramSource, TargetCpu, VirtualFs};
use jaic_llvm::{OptLevel, emit_object};
use std::{path::PathBuf, process::Command, rc::Rc};

fn out_dir() -> PathBuf {
    std::env::var_os("CARGO_TARGET_DIR")
        .map_or_else(
            || PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target"),
            PathBuf::from,
        )
        .join("xgetbv-controls")
}

fn program() -> Compiler {
    let mut options = Options::host();
    options.cpu = TargetCpu::X64;
    options.runtime_support = false;
    let mut c = Compiler::new(options, Rc::new(VirtualFs::default()));
    if let Err(e) = c.compile_sources(&[ProgramSource::String(
        include_str!("../../../tests/compiler-semantics/xgetbv.jai").into(),
    )]) {
        panic!("{}", e.render(&c.sources));
    }
    c
}

fn checked(command: &mut Command) -> String {
    let out = command.output().expect("required XGETBV control command");
    assert!(
        out.status.success(),
        "{command:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn xgetbv_real_opcode_survives_optimization_for_x64_abis() {
    let c = program();
    let dir = out_dir().join("metadata");
    std::fs::create_dir_all(&dir).unwrap();
    for target in [
        "x86_64-apple-darwin",
        "x86_64-unknown-linux-gnu",
        "x86_64-pc-windows-msvc",
        "x86_64-pc-windows-gnu",
    ] {
        for level in [OptLevel::O0, OptLevel::O2] {
            let path = dir.join(format!("{target}-{level:?}.o"));
            emit_object(
                &c.program,
                &jaic_llvm::Options {
                    target: Some(target.into()),
                    opt_level: level,
                    ..Default::default()
                },
                &path,
            )
            .unwrap();
            let disassembly = checked(Command::new("llvm-objdump").arg("-d").arg(&path));
            std::fs::write(path.with_extension("disassembly.txt"), &disassembly).unwrap();
            let count = disassembly
                .lines()
                .filter(|line| line.split_whitespace().any(|word| word == "xgetbv"))
                .count();
            assert!(
                count >= 5,
                "all five exported reads, including discarded output, must survive: {target} {level:?}: {disassembly}"
            );
        }
    }
    for target in [
        "aarch64-unknown-linux-gnu",
        "aarch64-pc-windows-msvc",
        "wasm64-unknown-unknown",
    ] {
        let error = emit_object(
            &c.program,
            &jaic_llvm::Options {
                target: Some(target.into()),
                ..Default::default()
            },
            &dir.join("rejected.o"),
        )
        .expect_err("non-x64 backend must reject XGETBV");
        assert!(
            error.contains("XGETBV requires an x86-64 target"),
            "{target}: {error}"
        );
    }
}

// Actual x64 CI hosts run this required native instruction proof. ARM hosts
// receive object/diagnostic coverage above, not a fabricated native pass.
#[cfg(target_arch = "x86_64")]
#[test]
fn xgetbv_native_matches_guarded_independent_isa_queries_at_o0_and_o2() {
    let c = program();
    let target = if cfg!(target_os = "macos") {
        "x86_64-apple-darwin"
    } else if cfg!(windows) {
        "x86_64-pc-windows-msvc"
    } else {
        "x86_64-unknown-linux-gnu"
    };
    for level in [OptLevel::O0, OptLevel::O2] {
        let dir = out_dir().join(format!("native-{level:?}"));
        std::fs::create_dir_all(&dir).unwrap();
        let object = dir.join("xgetbv.o");
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
        let exe = dir.join(format!("xgetbv{}", std::env::consts::EXE_SUFFIX));
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut driver = Command::new("clang");
        driver
            .args(["-std=c11", "-Wall", "-Wextra", "-Werror"])
            .arg(if level == OptLevel::O0 {
                "-O0"
            } else {
                "-O2"
            })
            .arg(root.join("tests/compiler-semantics/xgetbv-driver.c"))
            .arg(&object)
            .arg("-o")
            .arg(&exe);
        checked(&mut driver);
        let result = Command::new(&exe).output().unwrap();
        assert!(
            result.status.success(),
            "required native XGETBV proof unavailable or failed: status={:?}, stdout={}, stderr={}",
            result.status.code(),
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(String::from_utf8_lossy(&result.stdout).contains("real index0 PASS"));
    }
}
