//! Folded integer constants, `#run` and compiled code give the same results for shifts,
//! division and narrowing.
mod common;
use common::bare_compiler as compiler;
use jaic::sema::ProgramSource;

const SOURCE: &str =
    include_str!("../../../tests/compiler/semantics/integer-constant-arithmetic.jai");

#[test]
fn folded_constants_match_the_interpreter() {
    let mut c = compiler();
    if let Err(error) = c.compile_sources(&[ProgramSource::String(format!(
        "{SOURCE}\n#assert(check()==0);"
    ))]) {
        panic!("{}", error.render(&c.src.sources));
    }
}

#[test]
fn a_negative_constant_shift_is_rejected() {
    let mut c = compiler();
    let error = c
        .compile_sources(&[ProgramSource::String("X :: (cast(u32) 1) << -1;".into())])
        .expect_err("negative shift amounts in constants are errors");
    assert!(
        error.message.contains("negative shift"),
        "{}",
        error.message
    );
}
