#[path = "support/register_source.rs"]
mod registers_source;

mod cpuid {
    //! Architectural CPU data must never be substituted with fabricated zero features.
    use jaic::sema::{Compiler, Options, ProgramSource, TargetCpu, VirtualFs};
    use std::rc::Rc;

    const SOURCE: &str = include_str!("../../../tests/compiler/semantics/cpuid.jai");

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
                panic!("{}", e.render(&compiler.src.sources));
            }
        } else {
            let error = result.expect_err("non-x64 host must not report fake features");
            assert!(
                error.message.contains("x86-64 interpreter host")
                    || error.message.contains("requires an x86-64 target"),
                "{}",
                error.render(&compiler.src.sources)
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
                error.render(&c.src.sources)
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
            panic!("{}", e.render(&compiler.src.sources));
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
                error.render(&compiler.src.sources)
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
                include_str!("../../../tests/compiler/semantics/cpuid-leaf-zero.jai").into(),
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
            include_str!("../../../tests/compiler/semantics/cpuid-leaf-zero.jai").into(),
        )])
        .unwrap();
        let function = jaic::ir::FuncId(
            c.out
                .program
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
            let effects = c.ct.interp.effects;
            let actual =
                c.ct.interp
                    .call(&c.out.program, function, &[0, 0, index as u64])
                    .unwrap()[0];
            assert_eq!(actual, u64::from(value), "leaf zero register {index}");
            assert!(c.ct.interp.effects > effects);
        }
    }
}

mod xgetbv {
    //! XGETBV source/IR controls; no unguarded optional host instruction is executed.
    use jaic::ir::{Inst, Intrinsic, Linkage};
    use jaic::sema::TargetCpu;
    use test_support::compiler::compile_for_cpu as compile;


    #[test]
    fn xgetbv_consumes_index_and_produces_an_effectful_instruction() {
        let (c, result) = compile(
            TargetCpu::X64,
            "#program_export \"read_xcr\" read_xcr :: (index:u64) -> u64 #no_context { hi:u64;lo:u64; #asm {xgetbv hi,lo,index;} return (hi<<32)|lo; }",
        );
        result.unwrap();
        let function = c
            .out
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
                error.render(&c.src.sources)
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
            error.message.contains("x86-64 interpreter host")
                || error.message.contains("requires an x86-64 target"),
            "{}",
            error.render(&c.src.sources)
        );
    }


    #[cfg(target_arch = "x86_64")]
    #[test]
    #[allow(unsafe_code)]
    fn native_x64_interpreter_queries_are_guarded_and_not_cached_as_pure_results() {
        use std::arch::x86_64::{__cpuid_count, _xgetbv};
        let (mut c, result) = compile(
            TargetCpu::X64,
            include_str!("../../../tests/compiler/semantics/xgetbv.jai"),
        );
        result.unwrap();
        let function = |c: &jaic::sema::Compiler, name: &str| {
            jaic::ir::FuncId(
                c.out
                    .program
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
        let pointers = [(&raw mut high) as u64, (&raw mut low) as u64];
        let invalid =
            c.ct.interp
                .call(&c.out.program, read, &[2, pointers[0], pointers[1]])
                .unwrap_err();
        assert!(invalid.message.contains("index"), "{invalid:?}");
        if features & (3 << 26) != (3 << 26) {
            let error =
                c.ct.interp
                    .call(&c.out.program, read, &[0, pointers[0], pointers[1]])
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
            let effects = c.ct.interp.effects;
            let actual =
                c.ct.interp
                    .call(&c.out.program, read, &[index, pointers[0], pointers[1]])
                    .unwrap()[0];
            assert_eq!((actual ^ expected) & defined, 0);
            assert_eq!(actual, (high << 32) | low);
            assert!(u32::try_from(high).is_ok() && u32::try_from(low).is_ok());
            assert!(
                c.ct.interp.effects > effects,
                "architectural reads cannot use pure result caching"
            );
            assert_eq!(
                (c.ct.interp.call(&c.out.program, alias, &[index]).unwrap()[0] ^ expected)
                    & defined,
                0
            );
        }
        assert_eq!(
            (c.ct.interp.call(&c.out.program, read32, &[0]).unwrap()[0] ^ expected) & defined,
            0
        );
        let index1 = maximum >= 0xd && __cpuid_count(0xd, 1).eax & 4 != 0;
        if index1 {
            let actual =
                c.ct.interp
                    .call(&c.out.program, read, &[1, pointers[0], pointers[1]])
                    .unwrap()[0];
            assert_eq!(
                actual & !expected & defined,
                0,
                "XINUSE result must be within XCR0"
            );
            assert!(u32::try_from(high).is_ok() && u32::try_from(low).is_ok());
        } else {
            let error =
                c.ct.interp
                    .call(&c.out.program, read, &[1, pointers[0], pointers[1]])
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
                    c.ct.interp
                        .call(&c.out.program, flags, &[index, lhs, rhs])
                        .unwrap()[0],
                    1
                );
            }
            let effects = c.ct.interp.effects;
            c.ct.interp.call(&c.out.program, discard, &[index]).unwrap();
            assert!(c.ct.interp.effects > effects);
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
            panic!("{}", e.render(&c.src.sources));
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
            panic!("{}", e.render(&c.src.sources));
        }
        let ty = c
            .preload_type(jaic::sema::PreludeType::Reg, jaic::source::Span::NONE)
            .unwrap();
        assert_eq!(c.ty.types.name(ty), "__reg");
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
                error.render(&c.src.sources)
            );
        }
    }
}

mod x86_machine {
    //! ISA prerequisites and operand diagnostics; ARM hosts never report synthetic x86 results.
    use jaic::sema::TargetCpu;
    use test_support::compiler::compile_for_cpu as compile;


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
                    error.render(&c.src.sources)
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
                error.message.contains("x86-64 interpreter host")
                    || error.message.contains("requires an x86-64 target"),
                "{}",
                error.render(&c.src.sources)
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
            include_str!("../../../tests/compiler/semantics/x86-machine.jai"),
        );
        result.unwrap();
        let function = |c: &jaic::sema::Compiler, name: &str| {
            jaic::ir::FuncId(
                c.out
                    .program
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
            let effects = c.ct.interp.effects;
            let actual = c.ct.interp.call(&c.out.program, tsc, &[]).unwrap()[0];
            // SAFETY: as above, `rdtsc` is available on this CPU.
            let after = unsafe {
                _mm_lfence();
                _rdtsc()
            };
            assert!(
                before <= actual && actual <= after,
                "{before} <= {actual} <= {after}"
            );
            assert!(
                c.ct.interp.effects > effects,
                "architectural samples cannot enter pure compile-time result caching"
            );
        } else {
            assert!(
                c.ct.interp
                    .call(&c.out.program, tsc, &[])
                    .unwrap_err()
                    .message
                    .contains("unavailable")
            );
        }
        let tscp = function(&c, "machine_tscp");
        let mut auxiliary = 0u32;
        if __cpuid_count(0x8000_0000, 0).eax >= 0x8000_0001
            && __cpuid_count(0x8000_0001, 0).edx & (1 << 27) != 0
        {
            let mut matched = false;
            for _ in 0..100 {
                let mut a = 0;
                let mut b = 0;
                // SAFETY: `rdtscp` is available (checked above) and `a` is a live local.
                let before = unsafe { __rdtscp(&raw mut a) };
                let actual =
                    c.ct.interp
                        .call(&c.out.program, tscp, &[(&raw mut auxiliary) as u64])
                        .unwrap()[0];
                // SAFETY: as above, with `b`.
                let after = unsafe { __rdtscp(&raw mut b) };
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
                c.ct.interp
                    .call(&c.out.program, tscp, &[(&raw mut auxiliary) as u64])
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
                let result =
                    c.ct.interp
                        .call(&c.out.program, id, &[(&raw mut value) as u64]);
                if available {
                    let flags = result.unwrap()[0];
                    assert!(flags <= 1, "other arithmetic flags must clear: {flags}");
                    if width == 16 {
                        assert_eq!(value & !0xffff, 0x1234_5678_9abc_0000);
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
        c.ct.interp.call(&c.out.program, fences, &[]).unwrap();
        let prefetch = function(&c, "machine_prefetch");
        let mut value = 0x1234_5678_9abc_def0_u64;
        c.ct.interp
            .call(&c.out.program, prefetch, &[(&raw mut value) as u64])
            .unwrap();
        assert_eq!(value, 0x1234_5678_9abc_def0);
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
                c.out
                    .program
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
            let input = 0xabcd_ef12_3456_7800 | low;
            for count in [0, 1, 7, 63, 64] {
                let expected = u64::from((low as u8).count_ones().is_multiple_of(2));
                assert_eq!(
                    c.ct.interp
                        .call(&c.out.program, result_parity, &[input, count])
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
                    c.ct.interp
                        .call(&c.out.program, cleared_parity, &[input, count])
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
    fn string_compares_check_their_operand_count_and_control_constant() {
        for (body, message) in [
            (
                "#asm SSE4_2 { a:vec; b:vec; pcmpistri ecx:, a, b; }",
                "`pcmpistri` takes 4 operands",
            ),
            (
                "#asm SSE4_2 { a:vec; b:vec; pcmpistrm m:, a, b, 0, 0; }",
                "`pcmpistrm` takes 4 operands",
            ),
            (
                "#asm SSE4_2 { a:vec; b:vec; pcmpestri ecx:, a, b, 0; }",
                "`pcmpestri` takes 6 operands",
            ),
            (
                "#asm SSE4_2 { a:vec; b:vec; m:vec; pcmpestrm m, a, b, 0; }",
                "`pcmpestrm` takes 6 operands",
            ),
            (
                "#asm SSE4_2 { a:vec; b:vec; pcmpistri ecx:, a, b, b; }",
                "`pcmpistri` needs an 8-bit control constant",
            ),
            (
                "#asm SSE4_2 { a:vec; b:vec; pcmpistri ecx:, a, b, 256; }",
                "`pcmpistri` needs an 8-bit control constant",
            ),
            (
                "#asm SSE4_2 { a:vec; b:vec; pcmpistri ecx:, a, b, -1; }",
                "`pcmpistri` needs an 8-bit control constant",
            ),
        ] {
            let (c, result) = compile(TargetCpu::X64, &format!("main :: () {{ {body} }}"));
            let error = result.expect_err("a malformed string compare must diagnose");
            assert!(
                error.message.contains(message),
                "{message}: {}",
                error.render(&c.src.sources)
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
