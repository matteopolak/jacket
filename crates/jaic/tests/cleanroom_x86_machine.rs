//! ISA prerequisites and operand diagnostics; ARM hosts never report synthetic x86 results.
use jaic::sema::{Compiler, Options, ProgramSource, TargetCpu, VirtualFs};
use std::rc::Rc;

fn compile(cpu: TargetCpu, source: &str) -> (Compiler, jaic::sema::Result<()>) {
    let mut options = Options::host();
    options.cpu = cpu;
    options.runtime_support = false;
    let mut c = Compiler::new(options, Rc::new(VirtualFs::default()));
    let result = c.compile_sources(&[ProgramSource::String(source.into())]);
    (c, result)
}

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
            let error = result.expect_err("x86 ISA needs x64, not a synthetic alternate target");
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
// The independent ISA oracle uses unsafe std intrinsics only after real CPUID checks.
#[allow(unsafe_code)]
fn interpreter_uses_real_timestamps_aux_flags_and_optional_feature_diagnostics() {
    use std::arch::x86_64::{__cpuid_count, __rdtscp, _mm_lfence, _rdtsc};
    let (mut c, result) = compile(
        TargetCpu::X64,
        include_str!("../../../tests/compiler-semantics/x86-machine.jai"),
    );
    result.unwrap();
    let function = |c: &Compiler, name: &str| {
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
