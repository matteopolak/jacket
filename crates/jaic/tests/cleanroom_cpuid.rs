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
