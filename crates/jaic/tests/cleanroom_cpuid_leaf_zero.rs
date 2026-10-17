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
