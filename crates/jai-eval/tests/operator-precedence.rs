//! The evaluator consumes the canonical syntax tree rather than parsing operators again.
use jai_eval::{Value, evaluate};
use jai_source::Diagnostic;

#[test]
fn integer_masks_keep_bitwise_shift_arithmetic_and_comparison_precedence() {
    for expression in [
        "1 & 1 != 0",
        "1 | 2 ^ 3 & 4 << 1 == 3",
        "8 >> 1 & 1 + 3 == 4 && 0 | 1 != 0",
        "8 & 1 + 3 < 5",
    ] {
        let module = jai_syntax::parse(&format!("value :: {expression}; main :: () {{}}")).unwrap();
        let value = evaluate(&module.constants()[0].initializer, |_, span| {
            Err(Diagnostic::new(
                span,
                "unexpected name in integer precedence fixture",
            ))
        })
        .unwrap();
        assert_eq!(value, Value::Bool(true), "{expression}");
    }
}
