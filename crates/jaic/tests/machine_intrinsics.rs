mod common;

#[path = "support/register_source.rs"]
mod registers_source;

mod cpuid {
    //! Architectural CPU data must never be substituted with fabricated zero features.
    use jaic::sema::{Compiler, Options, ProgramSource, TargetCpu, VirtualFs};
    use std::rc::Rc;

    const SOURCE: &str = include_str!("../../../tests/compiler-semantics/cpuid.jai");

    #[test]
    fn cpuid_executes_real_host_query_or_diagnoses_unsupported_host() {
        let mut options = Options::host();
        options.cpu = TargetCpu::X64;
        options.runtime_support = false;
        let mut compiler = Compiler::new(options, Rc::new(VirtualFs::default()));
        let source = format!(
            "{SOURCE}\nprobe :: () -> bool #no_context {{ data:[4]u64; cpu_query(0,0,*data); return (data[1]|data[2]|data[3])!=0; }} #assert probe();"
        );
        let result = compiler.compile_sources(&[ProgramSource::String(source)]);
        if cfg!(target_arch = "x86_64") {
            if let Err(e) = result {
                panic!("{}", e.render(&compiler.sources));
            }
        } else {
            let error = result.expect_err("non-x64 host must not report fake features");
            assert!(
                error.message.contains("x86-64 interpreter host"),
                "{}",
                error.render(&compiler.sources)
            );
        }
    }

    #[test]
    fn cpuid_rejects_non_x64_target() {
        for cpu in [TargetCpu::Arm64, TargetCpu::Wasm] {
            let mut options = Options::host();
            options.cpu = cpu;
            options.runtime_support = false;
            let mut c = Compiler::new(options, Rc::new(VirtualFs::default()));
            let error = c
                .compile_sources(&[ProgramSource::String(SOURCE.into())])
                .expect_err("CPUID has no alternate ISA fallback");
            assert!(
                error.message.contains("x86-64 target"),
                "{}",
                error.render(&c.sources)
            );
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn cpuid_interpreter_matches_all_hardware_registers_and_subleaves() {
        let mut options = Options::host();
        options.runtime_support = false;
        let mut compiler = Compiler::new(options, Rc::new(VirtualFs::default()));
        let mut source = SOURCE.to_owned();
        for (leaf, subleaf) in [(0u32, 0u32), (7, 0), (7, 1), (0x8000_0000, 0)] {
            let r = std::arch::x86_64::__cpuid_count(leaf, subleaf);
            source.push_str(&format!("\nprobe_{leaf}_{subleaf} :: () -> bool #no_context {{ data:[4]u64; cpu_query({leaf},{subleaf},*data); return data[0]=={} && data[1]=={} && data[2]=={} && data[3]=={}; }} #assert probe_{leaf}_{subleaf}();",r.eax,r.ebx,r.ecx,r.edx));
        }
        if let Err(e) = compiler.compile_sources(&[ProgramSource::String(source)]) {
            panic!("{}", e.render(&compiler.sources));
        }
    }

    #[test]
    fn cpuid_requires_four_integer_register_operands() {
        for (body, expected) in [
            (
                "a:u8; b:u32; c:u32; d:u32; #asm { cpuid a,b,c,d; }",
                "32- or 64-bit integer registers",
            ),
            (
                "b:u32; c:u32; d:u32; #asm { cpuid 0,b,c,d; }",
                "32- or 64-bit integer registers",
            ),
            ("a:u32; b:u32; c:u32; #asm { cpuid a,b,c; }", "operand"),
        ] {
            let mut options = Options::host();
            options.cpu = TargetCpu::X64;
            options.runtime_support = false;
            let mut compiler = Compiler::new(options, Rc::new(VirtualFs::default()));
            let error = compiler
                .compile_sources(&[ProgramSource::String(format!("main :: () {{ {body} }}"))])
                .expect_err("invalid architectural operand must fail");
            assert!(
                error.message.contains(expected),
                "{}",
                error.render(&compiler.sources)
            );
        }
    }
}

mod cpuid_leaf_zero {
    //! Stable CPUID fields compared with an independent host intrinsic.
    use jaic::sema::{Compiler, Options, ProgramSource, TargetCpu, VirtualFs};
    use std::rc::Rc;

    #[test]
    fn cpuid_leaf_zero_compiles_for_x64_and_rejects_other_targets() {
        for cpu in [TargetCpu::X64, TargetCpu::Arm64, TargetCpu::Wasm] {
            let mut options = Options::host();
            options.cpu = cpu;
            options.runtime_support = false;
            let mut c = Compiler::new(options, Rc::new(VirtualFs::default()));
            let result = c.compile_sources(&[ProgramSource::String(
                include_str!("../../../tests/compiler-semantics/cpuid-leaf-zero.jai").into(),
            )]);
            if cpu == TargetCpu::X64 {
                result.unwrap();
            } else {
                assert!(result.unwrap_err().message.contains("x86-64 target"));
            }
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn interpreter_returns_all_real_leaf_zero_vendor_fields() {
        use std::arch::x86_64::__cpuid_count;
        let mut options = Options::host();
        options.cpu = TargetCpu::X64;
        options.runtime_support = false;
        let mut c = Compiler::new(options, Rc::new(VirtualFs::default()));
        c.compile_sources(&[ProgramSource::String(
            include_str!("../../../tests/compiler-semantics/cpuid-leaf-zero.jai").into(),
        )])
        .unwrap();
        let function = jaic::ir::FuncId(
            c.program
                .funcs
                .iter()
                .position(|f| {
                    f.as_ref().is_some_and(|f| {
                        f.linkage == jaic::ir::Linkage::Export("cpuid_register".into())
                    })
                })
                .unwrap() as u32,
        );
        let expected = __cpuid_count(0, 0);
        assert!(expected.eax >= 1);
        assert_ne!(expected.ebx | expected.ecx | expected.edx, 0);
        for (index, value) in [expected.eax, expected.ebx, expected.ecx, expected.edx]
            .into_iter()
            .enumerate()
        {
            let effects = c.interp.effects;
            let actual = c
                .interp
                .call(&c.program, function, &[0, 0, index as u64])
                .unwrap()[0];
            assert_eq!(actual, u64::from(value), "leaf zero register {index}");
            assert!(c.interp.effects > effects);
        }
    }
}

mod xgetbv {
    //! XGETBV source/IR controls; no unguarded optional host instruction is executed.
    use crate::common::compile_for_cpu as compile;
    use jaic::ir::{Inst, Intrinsic, Linkage};
    use jaic::sema::TargetCpu;


    #[test]
    fn xgetbv_consumes_index_and_produces_an_effectful_instruction() {
        let (c, result) = compile(
            TargetCpu::X64,
            "#program_export \"read_xcr\" read_xcr :: (index:u64) -> u64 #no_context { hi:u64;lo:u64; #asm {xgetbv hi,lo,index;} return (hi<<32)|lo; }",
        );
        result.unwrap();
        let function = c
            .program
            .funcs
            .iter()
            .flatten()
            .find(|f| f.linkage == Linkage::Export("read_xcr".into()))
            .unwrap();
        let operations = function
            .blocks
            .iter()
            .flat_map(|b| &b.insts)
            .filter_map(|inst| match inst {
                Inst::Intrinsic(i) => Some(i),
                _ => None,
            })
            .filter(|i| i.op == Intrinsic::X86Xgetbv)
            .collect::<Vec<_>>();
        assert_eq!(operations.len(), 1, "XGETBV must not fabricate constants");
        assert_eq!(operations[0].args.len(), 1, "ECX input must be consumed");
        assert_eq!(operations[0].results.len(), 2, "EDX:EAX result pair");
    }

    #[test]
    fn xgetbv_rejects_non_x64_targets() {
        for cpu in [TargetCpu::Arm64, TargetCpu::Wasm] {
            let (c, result) = compile(
                cpu,
                "main :: () {hi:u32;lo:u32;index:u32;#asm{xgetbv hi,lo,index;}}",
            );
            let error = result.expect_err("XGETBV requires x64");
            assert!(
                error.message.contains("x86-64 target"),
                "{}",
                error.render(&c.sources)
            );
        }
    }

    #[test]
    fn xgetbv_rejects_invalid_implicit_registers_and_lock() {
        for body in [
            "hi:u8;lo:u32;index:u32;#asm{xgetbv hi,lo,index;}",
            "hi:float32;lo:u32;index:u32;#asm{xgetbv hi,lo,index;}",
            "hi:u32;lo:u32;index:float32;#asm{xgetbv hi,lo,index;}",
            "hi:u32;lo:u32;index:u8;#asm{xgetbv hi,lo,index;}",
            "hi:u32;lo:u32;index:u32;#asm{xgetbv 1,lo,index;}",
            "hi:u32;lo:u32;index:u32;#asm{lock_xgetbv hi,lo,index;}",
        ] {
            let (_, result) = compile(TargetCpu::X64, &format!("main :: () {{ {body} }}"));
            result.expect_err("invalid XGETBV operand must diagnose");
        }
    }

    #[cfg(not(target_arch = "x86_64"))]
    #[test]
    fn non_x64_interpreter_never_returns_a_fabricated_xcr_value() {
        let (c, result) = compile(
            TargetCpu::X64,
            "read :: () -> u64 {hi:u32;lo:u32;index:u32;#asm{xgetbv hi,lo,index;}return (cast(u64)hi<<32)|lo;} #run read(); main :: () {}",
        );
        let error = result.expect_err("non-x64 host must reject architectural execution");
        assert!(
            error.message.contains("x86-64 interpreter host"),
            "{}",
            error.render(&c.sources)
        );
    }


    #[cfg(target_arch = "x86_64")]
    #[test]
    #[allow(unsafe_code)]
    fn native_x64_interpreter_queries_are_guarded_and_not_cached_as_pure_results() {
        use std::arch::x86_64::{__cpuid_count, _xgetbv};
        let (mut c, result) = compile(
            TargetCpu::X64,
            include_str!("../../../tests/compiler-semantics/xgetbv.jai"),
        );
        result.unwrap();
        let function = |c: &jaic::sema::Compiler, name: &str| {
            jaic::ir::FuncId(
                c.program
                    .funcs
                    .iter()
                    .position(|f| {
                        f.as_ref()
                            .is_some_and(|f| f.linkage == Linkage::Export(name.into()))
                    })
                    .unwrap() as u32,
            )
        };
        let read = function(&c, "xgetbv_read");
        let read32 = function(&c, "xgetbv_read32");
        let alias = function(&c, "xgetbv_alias");
        let flags = function(&c, "xgetbv_flags");
        let discard = function(&c, "xgetbv_discard");
        let maximum = __cpuid_count(0, 0).eax;
        let features = if maximum >= 1 {
            __cpuid_count(1, 0).ecx
        } else {
            0
        };
        let mut high = u64::MAX;
        let mut low = u64::MAX;
        let pointers = [
            (&mut high as *mut u64) as u64,
            (&mut low as *mut u64) as u64,
        ];
        let invalid = c
            .interp
            .call(&c.program, read, &[2, pointers[0], pointers[1]])
            .unwrap_err();
        assert!(invalid.message.contains("index"), "{invalid:?}");
        if features & (3 << 26) != (3 << 26) {
            let error = c
                .interp
                .call(&c.program, read, &[0, pointers[0], pointers[1]])
                .unwrap_err();
            assert!(error.message.contains("XSAVE"), "{error:?}");
            eprintln!(
                "XGETBV execution UNAVAILABLE: XSAVE/OSXSAVE absent; managed diagnostic PASS"
            );
            return; // This is the required unsupported-host diagnostic test.
        }
        // SAFETY: the real processor exposes both XSAVE and OSXSAVE.
        let expected = unsafe { _xgetbv(0) };
        assert_ne!(expected & 1, 0, "XCR0.x87 must be enabled");
        let defined = if maximum >= 0xd {
            let fields = __cpuid_count(0xd, 0);
            (u64::from(fields.edx) << 32) | u64::from(fields.eax)
        } else {
            1
        };
        for index in [0, 1u64 << 32] {
            let effects = c.interp.effects;
            let actual = c
                .interp
                .call(&c.program, read, &[index, pointers[0], pointers[1]])
                .unwrap()[0];
            assert_eq!((actual ^ expected) & defined, 0);
            assert_eq!(actual, (high << 32) | low);
            assert!(high <= u32::MAX as u64 && low <= u32::MAX as u64);
            assert!(
                c.interp.effects > effects,
                "architectural reads cannot use pure result caching"
            );
            assert_eq!(
                (c.interp.call(&c.program, alias, &[index]).unwrap()[0] ^ expected) & defined,
                0
            );
        }
        assert_eq!(
            (c.interp.call(&c.program, read32, &[0]).unwrap()[0] ^ expected) & defined,
            0
        );
        let index1 = maximum >= 0xd && __cpuid_count(0xd, 1).eax & 4 != 0;
        if index1 {
            let actual = c
                .interp
                .call(&c.program, read, &[1, pointers[0], pointers[1]])
                .unwrap()[0];
            assert_eq!(
                actual & !expected & defined,
                0,
                "XINUSE result must be within XCR0"
            );
            assert!(high <= u32::MAX as u64 && low <= u32::MAX as u64);
        } else {
            let error = c
                .interp
                .call(&c.program, read, &[1, pointers[0], pointers[1]])
                .unwrap_err();
            assert!(error.message.contains("index 1"), "{error:?}");
        }
        for index in 0..=u64::from(index1) {
            for (lhs, rhs) in [
                (0, 0),
                (0, 1),
                (u64::MAX, 1),
                (1u64 << 63, 1),
                (i64::MAX as u64, u64::MAX),
            ] {
                assert_eq!(
                    c.interp
                        .call(&c.program, flags, &[index, lhs, rhs])
                        .unwrap()[0],
                    1
                );
            }
            let effects = c.interp.effects;
            c.interp.call(&c.program, discard, &[index]).unwrap();
            assert!(c.interp.effects > effects);
        }
    }
}

mod registers {
    //! Original register operand forwarding and scalar-result regressions.
    use jaic::sema::{Compiler, Options, ProgramSource, VirtualFs};
    use std::rc::Rc;


    use super::registers_source::SOURCE;

    #[test]
    fn registers_forward_through_expansion_and_extract_scalar_lanes() {
        let mut options = Options::host();
        options.runtime_support = false;
        let mut c = Compiler::new(options, Rc::new(VirtualFs::default()));
        if let Err(e) = c.compile_sources(&[ProgramSource::String(format!(
            "{SOURCE}\n#assert(check()==0);"
        ))]) {
            panic!("{}", e.render(&c.sources));
        }
    }

    #[test]
    fn bootstrap_register_identity_and_scalar_misuse_are_checked() {
        let mut fs = VirtualFs::default();
        fs.insert(
            "/lib/prelude/module.jai",
            "__reg :: #type,distinct u16;".as_bytes(),
        );
        let fs = Rc::new(fs);
        let mut options = Options::host();
        options.runtime_support = false;
        options.preload = Some("/lib/prelude/module.jai".into());
        let mut c = Compiler::new(options.clone(), fs.clone());
        if let Err(e) = c.compile_sources(&[ProgramSource::String(format!(
            "{SOURCE}\n#assert(check()==0);"
        ))]) {
            panic!("{}", e.render(&c.sources));
        }
        let ty = c.preload_type("__reg", jaic::source::Span::NONE).unwrap();
        assert_eq!(c.types.name(ty), "__reg");
        for bad in [
            "use :: (v:__reg) #expand {} main :: () { n:u16; use(n); }",
            "use :: (v:__reg) #expand {} main :: () { n:[16]u8; use(n); }",
            "main :: () { #asm { v:vec; } n:=v; }",
            "main :: () { #asm { v:vec; add v,1; } }",
            "main :: () { value:__reg; }",
            "main :: () { value:[2]__reg; }",
            "Record :: struct { value:__reg; } main :: () { value:Record; }",
            "value:__reg; main :: () { value=cast(__reg)0; }",
            "use :: (v:__reg) {} main :: () { #asm { v:vec; } use(v); }",
        ] {
            let mut c = Compiler::new(options.clone(), fs.clone());
            let error = c
                .compile_sources(&[ProgramSource::String(bad.into())])
                .expect_err("register misuse must be rejected");
            assert!(
                error.message.contains("register"),
                "{}",
                error.render(&c.sources)
            );
        }
    }
}

mod x86_machine {
    //! ISA prerequisites and operand diagnostics; ARM hosts never report synthetic x86 results.
    use crate::common::compile_for_cpu as compile;
    use jaic::sema::TargetCpu;


    #[test]
    fn architectural_operations_reject_non_x64_targets() {
        for cpu in [TargetCpu::Arm64, TargetCpu::Wasm] {
            for body in [
                "hi:u32;lo:u32;#asm {rdtsc hi,lo;}",
                "hi:u32;lo:u32;aux:u32;#asm {rdtscp hi,lo,aux;}",
                "v:u64;#asm {rdrand v;}",
                "v:u64;#asm {rdseed v;}",
                "#asm {mfence;}",
                "#asm {lfence;}",
                "#asm {sfence;}",
                "#asm {pause;}",
                "value:u64; pointer:=*value; #asm {prefetcht0 [pointer];}",
            ] {
                let (c, result) = compile(cpu, &format!("main :: () {{ {body} }}"));
                let error =
                    result.expect_err("x86 ISA needs x64, not a synthetic alternate target");
                assert!(
                    error.message.contains("x86-64 target"),
                    "{}",
                    error.render(&c.sources)
                );
            }
        }
    }

    #[cfg(not(target_arch = "x86_64"))]
    #[test]
    fn non_x64_interpreter_rejects_x86_execution() {
        for body in [
            "hi:u32;lo:u32;#asm {rdtsc hi,lo;}",
            "hi:u32;lo:u32;aux:u32;#asm {rdtscp hi,lo,aux;}",
            "v:u64;#asm {rdrand v;}",
            "v:u64;#asm {rdseed v;}",
            "#asm {mfence;}",
            "#asm {lfence;}",
            "#asm {sfence;}",
            "#asm {pause;}",
            "value:u64; pointer:=*value; #asm {prefetcht0 [pointer];}",
        ] {
            let (c, result) = compile(
                TargetCpu::X64,
                &format!("probe :: () {{ {body} }} #run probe(); main :: () {{}}"),
            );
            let error = result.expect_err("host cannot execute x86 without actual translation");
            assert!(
                error.message.contains("x86-64 interpreter host"),
                "{}",
                error.render(&c.sources)
            );
        }
    }

    #[test]
    fn instruction_outputs_have_architectural_width_and_register_constraints() {
        for body in [
            "v:u8;#asm {rdrand.b v;}",
            "v:u64;#asm {rdseed [v];}",
            "#asm {rdrand 1;}",
            "hi:u8;lo:u32;#asm {rdtsc hi,lo;}",
            "hi:float32;lo:u32;#asm {rdtsc hi,lo;}",
            "hi:u32;lo:u32;#asm {rdtscp hi,lo,1;}",
            "#asm {mfence 1;}",
        ] {
            let (_, result) = compile(TargetCpu::X64, &format!("main :: () {{ {body} }}"));
            result.expect_err("invalid architectural operand must diagnose");
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[test]
    // The expected values come from std intrinsics, used only after real CPUID checks.
    #[allow(unsafe_code)]
    fn interpreter_uses_real_timestamps_aux_flags_and_optional_feature_diagnostics() {
        use std::arch::x86_64::{__cpuid_count, __rdtscp, _mm_lfence, _rdtsc};
        let (mut c, result) = compile(
            TargetCpu::X64,
            include_str!("../../../tests/compiler-semantics/x86-machine.jai"),
        );
        result.unwrap();
        let function = |c: &jaic::sema::Compiler, name: &str| {
            jaic::ir::FuncId(
                c.program
                    .funcs
                    .iter()
                    .position(|f| {
                        f.as_ref()
                            .is_some_and(|f| f.linkage == jaic::ir::Linkage::Export(name.into()))
                    })
                    .unwrap() as u32,
            )
        };
        let tsc = function(&c, "machine_tsc");
        if __cpuid_count(1, 0).edx & (1 << 4) != 0 {
            // SAFETY: real CPUID guards timestamp instructions; fences are baseline x64.
            let before = unsafe {
                _mm_lfence();
                _rdtsc()
            };
            let effects = c.interp.effects;
            let actual = c.interp.call(&c.program, tsc, &[]).unwrap()[0];
            let after = unsafe {
                _mm_lfence();
                _rdtsc()
            };
            assert!(
                before <= actual && actual <= after,
                "{before} <= {actual} <= {after}"
            );
            assert!(
                c.interp.effects > effects,
                "architectural samples cannot enter pure compile-time result caching"
            );
        } else {
            assert!(
                c.interp
                    .call(&c.program, tsc, &[])
                    .unwrap_err()
                    .message
                    .contains("unavailable")
            );
        }
        let tscp = function(&c, "machine_tscp");
        let mut auxiliary = 0u32;
        if __cpuid_count(0x80000000, 0).eax >= 0x80000001
            && __cpuid_count(0x80000001, 0).edx & (1 << 27) != 0
        {
            let mut matched = false;
            for _ in 0..100 {
                let mut a = 0;
                let mut b = 0;
                let before = unsafe { __rdtscp(&mut a) };
                let actual = c
                    .interp
                    .call(&c.program, tscp, &[(&mut auxiliary as *mut u32) as u64])
                    .unwrap()[0];
                let after = unsafe { __rdtscp(&mut b) };
                if a == b && auxiliary == a && before <= actual && actual <= after {
                    matched = true;
                    break;
                }
            }
            assert!(
                matched,
                "RDTSCP must match a real AUX/timestamp bracket; CPU migration cannot silently skip the assertion"
            );
        } else {
            assert!(
                c.interp
                    .call(&c.program, tscp, &[(&mut auxiliary as *mut u32) as u64])
                    .unwrap_err()
                    .message
                    .contains("unavailable")
            );
        }
        let maximum = __cpuid_count(0, 0).eax;
        for (name, width, available) in [
            (
                "machine_random16",
                16,
                __cpuid_count(1, 0).ecx & (1 << 30) != 0,
            ),
            (
                "machine_random32",
                32,
                __cpuid_count(1, 0).ecx & (1 << 30) != 0,
            ),
            (
                "machine_random64",
                64,
                __cpuid_count(1, 0).ecx & (1 << 30) != 0,
            ),
            (
                "machine_seed16",
                16,
                maximum >= 7 && __cpuid_count(7, 0).ebx & (1 << 18) != 0,
            ),
            (
                "machine_seed32",
                32,
                maximum >= 7 && __cpuid_count(7, 0).ebx & (1 << 18) != 0,
            ),
            (
                "machine_seed64",
                64,
                maximum >= 7 && __cpuid_count(7, 0).ebx & (1 << 18) != 0,
            ),
        ] {
            let id = function(&c, name);
            for _ in 0..64 {
                let mut value = 0u64;
                let result = c
                    .interp
                    .call(&c.program, id, &[(&mut value as *mut u64) as u64]);
                if available {
                    let flags = result.unwrap()[0];
                    assert!(flags <= 1, "other arithmetic flags must clear: {flags}");
                    if width == 16 {
                        assert_eq!(value & !0xffff, 0x123456789abc0000);
                    }
                    if width == 32 {
                        assert_eq!(value >> 32, 0);
                    }
                    if flags == 0 {
                        assert_eq!(
                            value
                                & if width == 64 {
                                    u64::MAX
                                } else {
                                    (1u64 << width) - 1
                                },
                            0
                        );
                    }
                } else {
                    assert!(result.unwrap_err().message.contains("unavailable"));
                }
            }
        }
        let fences = function(&c, "machine_fences");
        c.interp.call(&c.program, fences, &[]).unwrap();
        let prefetch = function(&c, "machine_prefetch");
        let mut value = 0x123456789abcdef0u64;
        c.interp
            .call(&c.program, prefetch, &[(&mut value as *mut u64) as u64])
            .unwrap();
        assert_eq!(value, 0x123456789abcdef0);
    }

    #[test]
    fn counter_intrinsic_signatures_are_validated() {
        for name in ["rdtsc", "get_cpu_cycle_count"] {
            for signature in ["(value: u64) -> u64", "() -> u32", "()", "() -> float64"] {
                let call = if signature.starts_with("(value") {
                    "1"
                } else {
                    ""
                };
                let (_, result) = compile(
                    TargetCpu::X64,
                    &format!("{name} :: {signature} #intrinsic; main :: () {{ {name}({call}); }}"),
                );
                assert!(
                    result
                        .expect_err("counter signature must match architectural result")
                        .message
                        .contains("requires no arguments and one 64-bit integer result")
                );
            }
        }
    }

    #[test]
    fn parity_result_bytes_and_explicit_clears_survive_dynamic_shifts() {
        let (mut c, result) = compile(
            TargetCpu::X64,
            r#"
    #program_export "result_parity"
    result_parity :: (input: u64, count: u64) -> u8 #c_call {
        value := input;
        parity: u8;
        #asm { test.q value, value; rol.q value, count; setp parity; }
        return parity;
    }
    #program_export "cleared_parity"
    cleared_parity :: (input: u64, count: u64) -> u8 #c_call {
        value: u64;
        parity: u8;
        #asm { popcnt.q value, input; shl.q value, count; setp parity; }
        return parity;
    }
    main :: () {}
    "#,
        );
        result.unwrap();
        let function = |name: &str| {
            jaic::ir::FuncId(
                c.program
                    .funcs
                    .iter()
                    .position(|f| {
                        f.as_ref()
                            .is_some_and(|f| f.linkage == jaic::ir::Linkage::Export(name.into()))
                    })
                    .unwrap() as u32,
            )
        };
        let result_parity = function("result_parity");
        let cleared_parity = function("cleared_parity");
        for low in 0u64..=255 {
            let input = 0xabcdef1234567800 | low;
            for count in [0, 1, 7, 63, 64] {
                let expected = u64::from((low as u8).count_ones().is_multiple_of(2));
                assert_eq!(
                    c.interp
                        .call(&c.program, result_parity, &[input, count])
                        .unwrap()[0],
                    expected
                );
                let masked = count & 63;
                let expected = if masked == 0 {
                    0
                } else {
                    u64::from(
                        ((u64::from(input.count_ones()) << masked) as u8)
                            .count_ones()
                            .is_multiple_of(2),
                    )
                };
                assert_eq!(
                    c.interp
                        .call(&c.program, cleared_parity, &[input, count])
                        .unwrap()[0],
                    expected
                );
            }
        }
    }

    #[test]
    fn unimplemented_cache_operations_diagnose_instead_of_succeeding_as_noops() {
        for instruction in [
            "prefetchw",
            "prefetchwt1",
            "clflush",
            "clflushopt",
            "clwb",
            "cldemote",
        ] {
            let (_, result) = compile(
                TargetCpu::X64,
                &format!(
                    "main :: () {{ value:u64; pointer:=*value; #asm {{ {instruction} [pointer]; }} }}"
                ),
            );
            assert!(
                result
                    .unwrap_err()
                    .message
                    .contains("cache operation is not implemented")
            );
        }
    }

    #[test]
    fn wasm_cycle_counter_is_an_explicit_missing_capability() {
        let (_, result) = compile(
            TargetCpu::Wasm,
            "get_cpu_cycle_count :: () -> u64 #intrinsic; main :: () { value:=get_cpu_cycle_count(); }",
        );
        assert!(
            result
                .unwrap_err()
                .message
                .contains("no architectural cycle counter")
        );
    }
}
