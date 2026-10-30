//! `#symmetric` on an ordinary two-argument procedure accepts the arguments in either order.
use jaic::sema::ProgramSource;
use test_support::compiler::bare_compiler as compiler;

fn compiles(source: &str) -> Result<(), String> {
    let mut c = compiler();
    c.compile_sources(&[ProgramSource::String(source.into())])
        .map_err(|e| e.render(&c.src.sources))
}

const TYPES: &str = "A :: struct { x: float; }\nB :: struct { y: float; }\n";

#[test]
fn swapped_call_of_a_symmetric_procedure_compiles() {
    let source = format!(
        "{TYPES}f :: (a: A, b: B) -> float #symmetric {{ return a.x + b.y; }}\n\
         main :: () {{ a: A; b: B; f(a, b); f(b, a); }}"
    );
    compiles(&source).expect("both argument orders are accepted");
}

#[test]
fn swapped_call_of_an_unmarked_procedure_is_still_rejected() {
    let source = format!(
        "{TYPES}f :: (a: A, b: B) -> float {{ return a.x + b.y; }}\n\
         main :: () {{ a: A; b: B; f(b, a); }}"
    );
    let error = compiles(&source).expect_err("no #symmetric, no swapping");
    assert!(error.contains("does not match"), "{error}");
}
