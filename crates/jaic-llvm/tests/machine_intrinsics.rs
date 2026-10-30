#[path = "../../jaic/tests/support/register_source.rs"]
mod registers_source;

mod cpuid {
    //! Real x64 instruction execution, compared to independent native C SDK queries.
    use jaic::sema::{Compiler, Options, ProgramSource, VirtualFs};
    use jaic::target::TargetCpu;
    use jaic_llvm::{OptLevel, emit_object};
    use std::{process::Command, rc::Rc};
    use test_support::{TempDir, process::checked, toolchain};

    // macOS ARM64 supports executing independently built x64 programs under translation.
    // Other ARM64 hosts receive metadata/target-diagnostic coverage below, not x64 execution claims.
    #[cfg(any(target_arch = "x86_64", target_os = "macos"))]
    #[test]
    fn cpuid_native_outputs_match_c_at_o0_and_o2() {
        if !toolchain::require_all(&["clang"]) {
            return;
        }
        let root = test_support::repo::root();
        let mut options = Options::host();
        options.cpu = TargetCpu::X64;
        options.runtime_support = false;
        let mut c = Compiler::new(options, Rc::new(VirtualFs::default()));
        if let Err(e) = c.compile_sources(&[ProgramSource::String(
            include_str!("../../../tests/compiler/semantics/cpuid.jai").into(),
        )]) {
            panic!("{}", e.render(&c.src.sources));
        }
        let target = if cfg!(target_os = "macos") {
            "x86_64-apple-darwin"
        } else if cfg!(windows) {
            "x86_64-pc-windows-msvc"
        } else {
            "x86_64-unknown-linux-gnu"
        };
        for level in [OptLevel::O0, OptLevel::O2] {
            let dir = TempDir::new(&format!("cpuid-{level:?}"));
            let object = dir.join("cpuid.o");
            emit_object(
                &c.out.program,
                &jaic_llvm::Options {
                    target: Some(target.parse().unwrap()),
                    opt_level: level,
                    ..Default::default()
                },
                &object,
            )
            .unwrap();
            let exe = dir.join(format!("cpuid{}", std::env::consts::EXE_SUFFIX));
            let mut driver = toolchain::command("clang");
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
                .arg(root.join("tests/compiler/semantics/cpuid-driver.c"))
                .arg(object)
                .arg("-o")
                .arg(&exe);
            checked(&mut driver);
            assert!(checked(&mut Command::new(exe)).contains("CPUID hardware registers"));
        }
    }

    #[test]
    fn cpuid_emits_real_instruction_for_x64_abis_and_rejects_other_architectures() {
        if !toolchain::require_all(&["llvm-objdump", "llvm-readobj"]) {
            return;
        }
        let mut options = Options::host();
        options.cpu = TargetCpu::X64;
        options.runtime_support = false;
        let mut c = Compiler::new(options, Rc::new(VirtualFs::default()));
        c.compile_sources(&[ProgramSource::String(
            include_str!("../../../tests/compiler/semantics/cpuid.jai").into(),
        )])
        .unwrap();
        let dir = TempDir::new("cpuid-metadata");
        for target in [
            "x86_64-apple-darwin",
            "x86_64-unknown-linux-gnu",
            "x86_64-pc-windows-msvc",
            "x86_64-pc-windows-gnu",
        ] {
            for level in [OptLevel::O0, OptLevel::O2] {
                let object = dir.join(format!("{target}-{level:?}.o"));
                emit_object(
                    &c.out.program,
                    &jaic_llvm::Options {
                        target: Some(target.parse().unwrap()),
                        opt_level: level,
                        ..Default::default()
                    },
                    &object,
                )
                .unwrap();
                let disassembly =
                    checked(toolchain::command("llvm-objdump").arg("-d").arg(&object));
                assert!(
                    disassembly
                        .lines()
                        .any(|line| line.split_whitespace().any(|word| word == "cpuid")),
                    "{target} {level:?}: {disassembly}"
                );
                if target.contains("windows") {
                    let unwind = checked(
                        toolchain::command("llvm-readobj")
                            .arg("--unwind")
                            .arg(&object),
                    );
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
                &c.out.program,
                &jaic_llvm::Options {
                    target: Some(target.parse().unwrap()),
                    ..Default::default()
                },
                &dir.join("rejected.o"),
            )
            .expect_err("CPUID requires a real x64 target");
            assert!(error.contains("x86-64 target"), "{target}: {error}");
        }
    }
}

mod cpuid_leaf_zero {
    //! O0/O2 CPUID codegen and native comparison against the public C intrinsic.
    #[cfg(any(target_arch = "x86_64", target_os = "macos"))]
    use jaic::sema::{Compiler, Options, ProgramSource, VirtualFs};
    use jaic::target::TargetCpu;
    #[cfg(any(target_arch = "x86_64", target_os = "macos"))]
    use jaic_llvm::{OptLevel, emit_object};
    #[cfg(any(target_arch = "x86_64", target_os = "macos"))]
    use std::{process::Command, rc::Rc};
    #[cfg(any(target_arch = "x86_64", target_os = "macos"))]
    use test_support::process::run as checked;
    #[cfg(any(target_arch = "x86_64", target_os = "macos"))]
    use test_support::{TempDir, ir, toolchain};

    #[cfg(any(target_arch = "x86_64", target_os = "macos"))]
    #[test]
    fn cpuid_leaf_zero_survives_optimization_and_matches_native_intrinsic() {
        if !toolchain::require_all(&["clang", "llvm-objdump"]) {
            return;
        }
        let root = test_support::repo::root();
        let mut options = Options::host();
        options.cpu = TargetCpu::X64;
        options.runtime_support = false;
        options.stack_trace = false;
        let mut c = Compiler::new(options, Rc::new(VirtualFs::default()));
        c.compile_sources(&[ProgramSource::String(
            include_str!("../../../tests/compiler/semantics/cpuid-leaf-zero.jai").into(),
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
            let dir = TempDir::new(&format!("cpuid-leaf-zero-{level:?}"));
            let object = dir.join("cpuid.o");
            let ir = dir.join("cpuid.ll");
            emit_object(
                &c.out.program,
                &jaic_llvm::Options {
                    target: Some(target.parse().unwrap()),
                    opt_level: level,
                    emit_ir: Some(ir.clone()),
                    ..Default::default()
                },
                &object,
            )
            .unwrap();
            let ir = std::fs::read_to_string(ir).unwrap();
            ir::assert_line(&ir, &["\"cpuid\"", "asm sideeffect", "~{memory}"]);
            let disassembly = toolchain::command("llvm-objdump")
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
                let mut clang = toolchain::command("clang");
                if cfg!(target_os = "macos") {
                    clang.args(["-arch", "x86_64"]);
                }
                checked(
                    clang
                        .args(["-std=c11", "-Wall", "-Wextra", "-Werror"])
                        .arg(root.join("tests/compiler/semantics/cpuid-leaf-zero-driver.c"))
                        .arg(&object)
                        .arg("-o")
                        .arg(&exe),
                );
                checked(&mut Command::new(exe));
            }
        }
    }
}

mod xgetbv {
    //! Real XGETBV object code across x64 ABIs, plus CPUID-guarded native controls.
    use jaic::sema::{Compiler, Options, ProgramSource, VirtualFs};
    use jaic::target::TargetCpu;
    use jaic_llvm::{OptLevel, emit_object};
    use std::rc::Rc;
    use test_support::{TempDir, process::checked, toolchain};

    fn out_dir() -> TempDir {
        TempDir::new("xgetbv-controls")
    }

    fn program() -> Compiler {
        let mut options = Options::host();
        options.cpu = TargetCpu::X64;
        options.runtime_support = false;
        let mut c = Compiler::new(options, Rc::new(VirtualFs::default()));
        if let Err(e) = c.compile_sources(&[ProgramSource::String(
            include_str!("../../../tests/compiler/semantics/xgetbv.jai").into(),
        )]) {
            panic!("{}", e.render(&c.src.sources));
        }
        c
    }

    #[test]
    fn xgetbv_real_opcode_survives_optimization_for_x64_abis() {
        if !toolchain::require_all(&["llvm-objdump"]) {
            return;
        }
        let c = program();
        let out = out_dir();
        let dir = out.join("metadata");
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
                    &c.out.program,
                    &jaic_llvm::Options {
                        target: Some(target.parse().unwrap()),
                        opt_level: level,
                        ..Default::default()
                    },
                    &path,
                )
                .unwrap();
                let disassembly = checked(toolchain::command("llvm-objdump").arg("-d").arg(&path));
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
                &c.out.program,
                &jaic_llvm::Options {
                    target: Some(target.parse().unwrap()),
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
        if !toolchain::require_all(&["clang"]) {
            return;
        }
        let c = program();
        let target = if cfg!(target_os = "macos") {
            "x86_64-apple-darwin"
        } else if cfg!(windows) {
            "x86_64-pc-windows-msvc"
        } else {
            "x86_64-unknown-linux-gnu"
        };
        for level in [OptLevel::O0, OptLevel::O2] {
            let out = out_dir();
            let dir = out.join(format!("native-{level:?}"));
            std::fs::create_dir_all(&dir).unwrap();
            let object = dir.join("xgetbv.o");
            emit_object(
                &c.out.program,
                &jaic_llvm::Options {
                    target: Some(target.parse().unwrap()),
                    opt_level: level,
                    ..Default::default()
                },
                &object,
            )
            .unwrap();
            let exe = dir.join(format!("xgetbv{}", std::env::consts::EXE_SUFFIX));
            let root = test_support::repo::root();
            let mut driver = toolchain::command("clang");
            driver
                .args(["-std=c11", "-Wall", "-Wextra", "-Werror"])
                .arg(if level == OptLevel::O0 {
                    "-O0"
                } else {
                    "-O2"
                })
                .arg(root.join("tests/compiler/semantics/xgetbv-driver.c"))
                .arg(&object)
                .arg("-o")
                .arg(&exe);
            checked(&mut driver);
            let result = std::process::Command::new(&exe).output().unwrap();
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
}

mod registers {
    //! Real native/WASM execution of mixed register operand forwarding.
    use test_support::backend;

    #[test]
    fn register_operands_execute_native_and_wasm_at_o0_and_o2() {
        backend::execute(
            "register-backend",
            "register_check",
            super::registers_source::SOURCE,
            &[],
            None,
        );
    }
}

mod x86_machine {
    //! Explicit architectural opcodes and actual CPUID-guarded native x64 execution.
    use jaic::sema::{Compiler, Options, ProgramSource, VirtualFs};
    use jaic::target::TargetCpu;
    use jaic_llvm::{OptLevel, emit_object};
    use std::{process::Command, rc::Rc};
    use test_support::{TempDir, ir, process::checked, toolchain};

    #[test]
    fn random_parity_outputs_are_zero_without_eliminating_architectural_samples() {
        if !toolchain::require_all(&["opt"]) {
            return;
        }
        let mut options = Options::host();
        options.cpu = TargetCpu::X64;
        options.runtime_support = false;
        options.stack_trace = false;
        let mut c = Compiler::new(options, Rc::new(VirtualFs::default()));
        c.compile_sources(&[ProgramSource::String(
            r#"
    #program_export "random_parity"
    random_parity :: () -> u8 #c_call {
        value: u64; parity: u8;
        #asm { rdrand.q value; setp parity; }
        return parity;
    }
    #program_export "seed_parity"
    seed_parity :: () -> u8 #c_call {
        value: u64; parity: u8;
        #asm { rdseed.q value; setp parity; }
        return parity;
    }
    main :: () {}
    "#
            .into(),
        )])
        .unwrap();
        let dir = TempDir::new("x86-parity-source");
        let ir = dir.join("parity.ll");
        emit_object(
            &c.out.program,
            &jaic_llvm::Options {
                target: Some("x86_64-unknown-linux-gnu".parse().unwrap()),
                opt_level: OptLevel::O2,
                emit_ir: Some(ir.clone()),
                ..Default::default()
            },
            &dir.join("parity.o"),
        )
        .unwrap();
        // emit_ir records the pre-optimization module. Evaluate its constant flag
        // dataflow with the same official LLVM optimizer, without executing ISA.
        let optimized = dir.join("parity-opt.ll");
        checked(
            toolchain::command("opt")
                .arg("-passes=default<O2>")
                .arg("-S")
                .arg(&ir)
                .arg("-o")
                .arg(&optimized),
        );
        let ir = std::fs::read_to_string(optimized).unwrap();
        for (name, instruction) in [("random_parity", "rdrand"), ("seed_parity", "rdseed")] {
            let body = ir
                .lines()
                .skip_while(|line| {
                    !line.starts_with("define ") || !line.contains(&format!("@{name}("))
                })
                .take_while(|line| *line != "}")
                .collect::<Vec<_>>()
                .join("\n");
            assert!(body.contains("ret i8 0"), "{name} must expose PF=0: {body}");
            ir::assert_line(&body, &[instruction, "asm sideeffect"]);
        }
    }

    #[test]
    fn opcodes_and_architecture_rejection_survive_optimization() {
        if !toolchain::require_all(&["clang", "llvm-objdump"]) {
            return;
        }
        let root = test_support::repo::root();
        let mut options = Options::host();
        options.cpu = TargetCpu::X64;
        options.runtime_support = false;
        options.stack_trace = false;
        let mut c = Compiler::new(options, Rc::new(VirtualFs::default()));
        if let Err(e) = c.compile_sources(&[ProgramSource::String(
            include_str!("../../../tests/compiler/semantics/x86-machine.jai").into(),
        )]) {
            panic!("{}", e.render(&c.src.sources));
        }
        for target in [
            "x86_64-apple-darwin",
            "x86_64-unknown-linux-gnu",
            "x86_64-pc-windows-msvc",
            "x86_64-pc-windows-gnu",
        ] {
            for level in [OptLevel::O0, OptLevel::O2] {
                let dir = TempDir::new(&format!("x86-machine-{target}-{level:?}"));
                let object = dir.join("machine.o");
                let ir = dir.join("machine.ll");
                emit_object(
                    &c.out.program,
                    &jaic_llvm::Options {
                        target: Some(target.parse().unwrap()),
                        opt_level: level,
                        emit_ir: Some(ir.clone()),
                        ..Default::default()
                    },
                    &object,
                )
                .unwrap();
                let disassembly =
                    checked(toolchain::command("llvm-objdump").arg("-d").arg(&object));
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
                    ir::assert_line(
                        &ir,
                        &[&format!("\"{instruction}\""), "asm sideeffect", "~{memory}"],
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
                        let mut clang = toolchain::command("clang");
                        if cfg!(target_os = "macos") {
                            clang.args(["-arch", "x86_64"]);
                        }
                        checked(
                            clang
                                .args(["-std=c11", "-Wall", "-Wextra", "-Werror"])
                                .arg(root.join("tests/compiler/semantics/x86-machine-driver.c"))
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
                &c.out.program,
                &jaic_llvm::Options {
                    target: Some(target.parse().unwrap()),
                    ..Default::default()
                },
                &TempDir::new("x86-machine-rejected").join("rejected.o"),
            )
            .expect_err("architectural intrinsics must not silently disappear on another backend");
            assert!(error.contains("x86-64 target"), "{error}");
        }
    }
}
