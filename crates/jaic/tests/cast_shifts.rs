//! Explicit widening and SDK encoding of shifts.
mod common;
use common::bare_compiler as compiler;
use jaic::sema::ProgramSource;

const SOURCE: &str = include_str!("../../../tests/compiler/semantics/cast-shift-widths.jai");


#[test]
fn explicit_widening_preserves_constant_and_runtime_shift_bits() {
    let mut c = compiler();
    if let Err(error) = c.compile_sources(&[ProgramSource::String(format!(
        "{SOURCE}\n#assert(check()==0);"
    ))]) {
        panic!("{}", error.render(&c.sources));
    }
}

#[test]
fn shift_use_does_not_remove_checked_cast_errors() {
    for (source, expected) in [
        ("#assert((cast(u8) 256) << 1 == 0);", "out of range"),
        ("#assert((cast(u32) -1) >> 1 == 0);", "out of range"),
        ("#assert((cast(s8) 128) << 1 == 0);", "out of range"),
        (
            "narrow :: (value:u32) -> u8 {return cast(u8) value;} #assert(narrow(256) << 1 == 0);",
            "overflows",
        ),
    ] {
        let mut c = compiler();
        let error = c
            .compile_sources(&[ProgramSource::String(source.into())])
            .expect_err("checked out-of-range cast must still reject or trap");
        let rendered = error.render(&c.sources);
        assert!(error.message.contains(expected), "{rendered}");
    }
}
