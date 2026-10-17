//! XGETBV source/IR controls; no unguarded optional host instruction is executed.
use jaic::ir::{Inst, Intrinsic, Linkage};
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
    let function = |c: &Compiler, name: &str| {
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
        eprintln!("XGETBV execution UNAVAILABLE: XSAVE/OSXSAVE absent; managed diagnostic PASS");
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
