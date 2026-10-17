mod common;
#[path = "../../jaic/tests/support/register_source.rs"]
mod registers_source;

mod cpuid {
    //! Real x64 instruction execution, compared to independent native C SDK queries.
    use crate::common::checked;
    use jaic::sema::{Compiler, Options, ProgramSource, TargetCpu, VirtualFs};
    use jaic_llvm::{OptLevel, emit_object};
    use std::{path::PathBuf, process::Command, rc::Rc};

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
                    target: Some(target.parse().unwrap()),
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
                        target: Some(target.parse().unwrap()),
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
    use crate::common::run as checked;
    use jaic::sema::{Compiler, Options, ProgramSource, TargetCpu, VirtualFs};
    use jaic_llvm::{OptLevel, emit_object};
    use std::{path::PathBuf, process::Command, rc::Rc};

    #[cfg(any(target_arch = "x86_64", target_os = "macos"))]
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
                    target: Some(target.parse().unwrap()),
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
}

mod xgetbv {
    //! Real XGETBV object code across x64 ABIs, plus CPUID-guarded native controls.
    use crate::common::checked;
    use jaic::sema::{Compiler, Options, ProgramSource, TargetCpu, VirtualFs};
    use jaic_llvm::{OptLevel, emit_object};
    use std::{path::PathBuf, process::Command, rc::Rc};

    fn out_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("xgetbv-controls")
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
                        target: Some(target.parse().unwrap()),
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
                    target: Some(target.parse().unwrap()),
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
}

mod registers {
    //! Real native/WASM execution of mixed register operand forwarding.
    use crate::common::run;
    use jaic::sema::{Compiler, Options, ProgramSource, TargetCpu, TargetOs, VirtualFs};
    use jaic_llvm::{OptLevel, emit_object};
    use std::{process::Command, rc::Rc};


    #[test]
    fn register_operands_execute_native_and_wasm_at_o0_and_o2() {
        execute("registers", super::registers_source::SOURCE);
    }

    fn execute(name: &str, source: &str) {
        for wasm in [false, true] {
            for optimized in [false, true] {
                let dir = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
                    .join(format!("register-backend/{name}-{wasm}-{optimized}"));
                std::fs::create_dir_all(&dir).unwrap();
                let mut options = Options::host();
                options.runtime_support = false;
                if wasm {
                    options.os = TargetOs::Wasm;
                    options.cpu = TargetCpu::Wasm;
                }
                let mut c = Compiler::new(options, Rc::new(VirtualFs::default()));
                if let Err(e) = c.compile_sources(&[ProgramSource::String(source.into())]) {
                    panic!("{}", e.render(&c.sources));
                }
                let object = dir.join("variadics.o");
                emit_object(
                    &c.program,
                    &jaic_llvm::Options {
                        target: wasm.then(|| jaic::target::Target::wasm_module()),
                        opt_level: if optimized {
                            OptLevel::O2
                        } else {
                            OptLevel::O0
                        },
                        ..Default::default()
                    },
                    &object,
                )
                .unwrap();
                if wasm {
                    let module = dir.join("variadics.wasm");
                    run(Command::new("wasm-ld")
                        .args([
                            "-mwasm64",
                            "--no-entry",
                            "--export=register_check",
                            "--export-memory",
                            "--allow-undefined",
                        ])
                        .arg(object)
                        .arg("-o")
                        .arg(&module));
                    run(Command::new("node").args(["-e",r#"const fs=require('fs');const m=new WebAssembly.Module(fs.readFileSync(process.argv[1]));let memory;const env={};for(const i of WebAssembly.Module.imports(m)){if(i.kind!=='function')throw Error('unexpected import');env[i.name]=()=>{throw Error('unexpected runtime failure '+i.name);};}env.memset=(d,v,n)=>{new Uint8Array(memory.buffer).fill(v,Number(d),Number(d+n));return d;};env.memcpy=(d,s,n)=>{const bytes=new Uint8Array(memory.buffer);bytes.copyWithin(Number(d),Number(s),Number(s+n));return d;};const x=new WebAssembly.Instance(m,{env});memory=x.exports.memory;const result=x.exports.register_check();if(result!==0)throw Error('variadic failure '+result);"#]).arg(module));
                } else {
                    let driver = dir.join("driver.c");
                    std::fs::write(
                        &driver,
                        "extern int register_check(void); int main(void){return register_check();}\n",
                    )
                    .unwrap();
                    let exe = dir.join("driver");
                    run(Command::new("clang")
                        .arg(driver)
                        .arg(object)
                        .arg("-o")
                        .arg(&exe));
                    run(&mut Command::new(exe));
                }
            }
        }
    }
}

mod x86_machine {
    //! Explicit architectural opcodes and actual CPUID-guarded native x64 execution.
    use crate::common::checked;
    use jaic::sema::{Compiler, Options, ProgramSource, TargetCpu, VirtualFs};
    use jaic_llvm::{OptLevel, emit_object};
    use std::{path::PathBuf, process::Command, rc::Rc};

    #[test]
    fn random_parity_outputs_are_zero_without_eliminating_architectural_samples() {
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
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let dir = root.join("target/x86-parity-source");
        std::fs::create_dir_all(&dir).unwrap();
        let ir = dir.join("parity.ll");
        emit_object(
            &c.program,
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
            Command::new("opt")
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
            assert!(
                body.lines()
                    .any(|line| line.contains(instruction) && line.contains("asm sideeffect")),
                "clearing the flag must not remove the real architectural sample: {body}"
            );
        }
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
                        target: Some(target.parse().unwrap()),
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
                    target: Some(target.parse().unwrap()),
                    ..Default::default()
                },
                &root.join("target/x86-machine/rejected.o"),
            )
            .expect_err("architectural intrinsics must not silently disappear on another backend");
            assert!(error.contains("x86-64 target"), "{error}");
        }
    }
}
