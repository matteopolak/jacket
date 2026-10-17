//! Raw pointer values are not requests to read/freeze their pointees.
use jaic::sema::{Compiler, Options, ProgramSource, TargetCpu, VirtualFs};
use std::rc::Rc;

fn compile(source: &str, cpu: TargetCpu) -> (Compiler, jaic::sema::Result<()>) {
    let mut options = Options::host();
    options.cpu = cpu;
    options.runtime_support = false;
    let mut c = Compiler::new(options, Rc::new(VirtualFs::default()));
    let result = c.compile_sources(&[ProgramSource::String(source.into())]);
    (c, result)
}

#[test]
fn raw_pointer_constants_defaults_globals_and_run_results_are_values() {
    for cpu in [TargetCpu::X64, TargetCpu::Arm64, TargetCpu::Wasm] {
        let (c, result) = compile(
            &format!(
                "{} #assert check()==0; #assert size_of(HKEY)==8;",
                include_str!("../../../tests/compiler/semantics/raw-pointer-constants.jai")
            ),
            cpu,
        );
        if let Err(e) = result {
            panic!("{}", e.render(&c.sources));
        }
        assert!(
            !c.program.globals.iter().any(|g| g.name == "frozen"),
            "raw handles must not acquire fabricated pointee storage"
        );
    }
}

#[test]
fn checked_integer_narrowing_still_rejects_out_of_range_pointer_bits() {
    for source in [
        "HANDLE :: cast(*void) 0x80000001; BYTE :: cast(u8) HANDLE; main :: () {}",
        "make :: () -> *void { return cast(*void) 0x80000001; } HANDLE :: #run make(); BYTE :: cast(u8) HANDLE; main :: () {}",
    ] {
        let (c, result) = compile(source, TargetCpu::X64);
        let e = result.expect_err("raw pointer representation must not bypass checked narrowing");
        assert!(
            e.message.contains("out of range"),
            "{}",
            e.render(&c.sources)
        );
    }
}

#[test]
fn invalid_pointer_access_still_traps_and_forged_views_cannot_capture_memory() {
    for source in [
        "read :: () -> u32 { pointer:=cast(*u32)1; return pointer.*; } #run read(); main :: () {}",
        "read :: () -> u32 { pointer:=cast(*u32)1; pointer.*=2; return 0; } #run read(); main :: () {}",
        "make :: () -> string { value:string; value.count=4; value.data=cast(*u8)0x80000001; return value; } BAD :: #run make(); main :: () {}",
    ] {
        let (c, result) = compile(source, TargetCpu::X64);
        let e =
            result.expect_err("representing a raw pointer cannot license invalid memory access");
        assert!(
            e.message.contains("null pointer") || e.message.contains("cannot be read"),
            "{}",
            e.render(&c.sources)
        );
    }
}

#[test]
fn baked_pointer_constants_retain_their_pointer_type() {
    let (c, result) = compile(
        r#"
        HKEY :: *void;
        HANDLE :: cast(HKEY)0x80000001;
        inspect :: ($value: $T) -> bool { return T==HKEY && type_of(value)==HKEY; }
        inspect_untyped :: ($value:) -> bool { return type_of(value)==HKEY; }
        #assert inspect(HANDLE);
        #assert inspect_untyped(HANDLE);
        kind :: ($value:) -> int { if type_of(value)==HKEY return 1; if type_of(value)==u32 return 2; if type_of(value)==*u32 return 3; return 0; }
        #assert kind(0x80000001)==0;
        #assert kind(HANDLE)==1;
        #assert kind(cast(u32)0x80000001)==2;
        #assert kind(cast(*u32)0x80000001)==3;
        #assert kind(HANDLE)==1;
        Holder :: struct(value:) { pointer:*type_of(value); }
        #assert Holder(HANDLE)!=Holder(0x80000001);
        main :: () {}
    "#,
        TargetCpu::X64,
    );
    if let Err(e) = result {
        panic!("{}", e.render(&c.sources));
    }
}

#[test]
fn pointer_bit_patterns_preserve_full_64_bit_width() {
    let (c, result) = compile(
        r#"
        POSITIVE :: cast(*void) 0x80000001;
        HIGH :: cast(*void) 0x1234567880000001;
        NEGATIVE :: cast(*void) -1;
        #assert (cast(u64) POSITIVE)==0x80000001;
        #assert (cast(u64) HIGH)==0x1234567880000001;
        #assert (cast(u64) NEGATIVE)==0xffffffffffffffff;
        #assert POSITIVE!=HIGH;
        #assert (cast,trunc(u8) POSITIVE)==1;
        #assert (cast,no_check(u32) HIGH)==0x80000001;
        #assert (cast(*void) 0)==null;
        main :: () {}
    "#,
        TargetCpu::X64,
    );
    if let Err(e) = result {
        panic!("{}", e.render(&c.sources));
    }
}
