//! Original atomic intrinsic regressions; no module-specific implementation.
use jaic::sema::{Compiler, Options, ProgramSource, VirtualFs};
use std::rc::Rc;

const DECLS: &str = r#"
atomic_load :: (pointer: *$T) -> T #intrinsic;
atomic_store :: (pointer: *$T, new: T) #intrinsic;
atomic_exchange :: (pointer: *$T, new: T) -> T #intrinsic;
compare_and_swap :: (pointer: *$T, old: T, new: T) -> (bool, T) #intrinsic;
"#;

fn compile(source: &str) -> (Compiler, jaic::sema::Result<()>) {
    let mut options = Options::host();
    options.runtime_support = false;
    let mut c = Compiler::new(options, Rc::new(VirtualFs::default()));
    let result = c.compile_sources(&[ProgramSource::String(format!("{DECLS}\n{source}"))]);
    (c, result)
}

#[test]
fn bootstrap_atomics_preserve_scalar_bits_and_previous_values() {
    let (c, result) = compile(
        r#"
        check :: () -> bool {
            a: u8; b: s16; c: u32; d: u64; f: float32; g: float64; flag: bool;
            atomic_store(*a, 255); atomic_store(*b, -7);
            atomic_store(*c, 0xf1234567); atomic_store(*d, 0xfedcba9876543210);
            atomic_store(*f, 0h80000000); atomic_store(*g, 0h7ff8000000001234);
            atomic_store(*flag, true);
            ok_nan, nan_old := compare_and_swap(*g, 0h7ff8000000001234, 0h7ff8000000005678);
            if !ok_nan || cast,force(u64) nan_old != 0x7ff8000000001234 return false;
            atomic_store(*g, 0h7ff8000000001234);
            if atomic_load(*a) != 255 || atomic_exchange(*b, 9) != -7 return false;
            if atomic_load(*c) != 0xf1234567 || atomic_exchange(*d, 17) != 0xfedcba9876543210 return false;
            if cast,force(u32) atomic_load(*f) != 0x80000000 return false;
            if cast,force(u64) atomic_exchange(*g, 1) != 0x7ff8000000001234 return false;
            if !atomic_exchange(*flag, false) || atomic_load(*flag) return false;
            p: *u32; atomic_store(*p, *c);
            if atomic_load(*p) != *c || atomic_exchange(*p, null) != *c return false;
            ok, previous := compare_and_swap(*d, 17, 23);
            return ok && previous == 17 && d == 23;
        }
        #assert(check());
    "#,
    );
    if let Err(e) = result {
        panic!("{}", e.render(&c.sources));
    }
}

#[test]
fn atomic_enum_and_procedure_values_keep_their_types_and_identity() {
    let (c, result) = compile(
        r#"
        Token :: enum u32 { FIRST :: 7; SECOND :: 29; }
        first :: () -> int #no_context { return 13; }
        second :: () -> int #no_context { return 47; }
        check :: () -> bool {
            token: Token = .FIRST;
            if atomic_exchange(*token, .SECOND) != .FIRST || atomic_load(*token) != .SECOND return false;
            procedure: () -> int #no_context = first;
            previous := atomic_exchange(*procedure, second);
            if previous() != 13 || atomic_load(*procedure)() != 47 return false;
            ok, old := compare_and_swap(*procedure, second, first);
            return ok && old() == 47 && procedure() == 13;
        }
        #assert(check());
    "#,
    );
    if let Err(e) = result {
        panic!("{}", e.render(&c.sources));
    }
}

#[test]
fn atomic_aggregate_and_wide_operands_are_rejected() {
    for ty in ["[2]u32", "[16]u8", "string", "Record"] {
        let (c, result) = compile(&format!(
            "Record :: struct {{ x: u64; }} check :: () {{ value: {ty}; atomic_load(*value); }} #run check();"
        ));
        let error = result.expect_err("aggregate/wide atomic access must fail");
        assert!(
            error.message.contains("atomic scalar"),
            "{}",
            error.render(&c.sources)
        );
    }
}

#[test]
fn atomic_misalignment_is_an_explicit_runtime_error() {
    let (c, result) = compile(
        "check :: () { storage: [24]u8; p := cast(*u64) (storage.data + 1); atomic_store(p, 7); } #run check();",
    );
    let error = result.expect_err("unaligned atomic access must fail");
    assert!(
        error.message.contains("aligned"),
        "{}",
        error.render(&c.sources)
    );
}

#[test]
fn intrinsic_signatures_cannot_change_the_accessed_scalar_type() {
    let declarations = DECLS.replace("(pointer: *$T) -> T", "(pointer: *$T) -> u32");
    let mut options = Options::host();
    options.runtime_support = false;
    let mut c = Compiler::new(options, Rc::new(VirtualFs::default()));
    let source =
        format!("{declarations} check :: () {{ value: u64; atomic_load(*value); }} #run check();");
    let error = c
        .compile_sources(&[ProgramSource::String(source)])
        .expect_err("incorrect atomic result type must fail");
    assert!(
        error.message.contains("signature"),
        "{}",
        error.render(&c.sources)
    );
}

#[test]
fn ordered_atomics_reject_invalid_and_stronger_failure_orders() {
    let decls = "atomic_load_order :: (p: *$T, order: s64) -> T #intrinsic; atomic_store_order :: (p: *$T, value: T, order: s64) #intrinsic; atomic_compare_exchange_order :: (p: *$T, old: T, new: T, orders: s64) -> (bool,T) #intrinsic;";
    for call in [
        "atomic_load_order(*v, 256)",
        "atomic_store_order(*v, 1, 16)",
        "atomic_load_order(*v, 2)",
        "atomic_compare_exchange_order(*v, 0, 1, 0x10)",
        "atomic_compare_exchange_order(*v, 0, 1, 0x42)",
    ] {
        let (c, result) = compile(&format!(
            "{decls} check :: () {{ v: u64; {call}; }} #run check();"
        ));
        let error = result.expect_err("invalid order must fail");
        assert!(
            error.message.contains("order"),
            "{}",
            error.render(&c.sources)
        );
    }
}
