//! Scalar domains and evaluation retain one actual implicit subject.
use jai_eval::{CheckMode, DomainInference, ScalarDomain, Value};
use jai_source::Diagnostic;

fn expression(text: &str) -> jai_syntax::Expression {
    jai_syntax::parse(&format!("VALUE :: {text}; main :: () {{}}"))
        .unwrap()
        .constants()[0]
        .initializer
        .clone()
}
#[test]
fn constant_comparison_unary_and_omitted_default_keep_subject_values() {
    for (text, expected) in [
        ("ifx 42 > 5 else 0", Value::Literal(42)),
        ("ifx !0 else 42", Value::Literal(0)),
        ("ifx 0", Value::Literal(0)),
        ("ifx false", Value::Bool(false)),
        ("ifx 42", Value::Literal(42)),
    ] {
        let value = jai_eval::evaluate(&expression(text), |_, span| {
            Err(Diagnostic::new(span, "unexpected name"))
        })
        .unwrap();
        assert_eq!(value, expected, "{text}");
    }
}
#[test]
fn actual_scalar_subject_is_bound_once_by_both_evaluators() {
    let input = expression("ifx answer > 0 else 0");
    let mut reads = 0;
    let value = jai_eval::evaluate_paths(&input, |_, _| {
        reads += 1;
        Ok(Value::Literal(42))
    })
    .unwrap();
    assert_eq!(value, Value::Literal(42));
    assert_eq!(reads, 1);
    let mut domains = 0;
    let inferred = DomainInference::infer(&input, |_, _| {
        domains += 1;
        Ok(ScalarDomain::IntegerLiteral)
    })
    .unwrap();
    assert_eq!(domains, 1);
    reads = 0;
    let value = inferred
        .evaluate_paths(CheckMode::Enabled, |_, _| {
            reads += 1;
            Ok(Value::Literal(42))
        })
        .unwrap();
    assert_eq!(value, Value::Literal(42));
    assert_eq!(reads, 1);
}
#[test]
fn inactive_fallback_domain_is_checked_but_its_value_is_not_read() {
    let input = expression("ifx answer > 0 else unavailable");
    let inferred = DomainInference::infer(&input, |_, _| Ok(ScalarDomain::IntegerLiteral)).unwrap();
    let mut reads = 0;
    let value = inferred
        .evaluate_paths(CheckMode::Enabled, |_, span| {
            assert_eq!(
                span,
                match &input.kind {
                    jai_syntax::ExpressionKind::Conditional(value) => value.then_source().span,
                    _ => panic!(),
                }
            );
            reads += 1;
            Ok(Value::Literal(42))
        })
        .unwrap();
    assert_eq!(value, Value::Literal(42));
    assert_eq!(reads, 1);
}
#[test]
fn nested_subject_capture_and_short_circuit_preserve_actual_selection() {
    let input = expression("ifx (ifx answer > 0 else 0) > 1 else 0");
    let inferred = DomainInference::infer(&input, |_, _| Ok(ScalarDomain::IntegerLiteral)).unwrap();
    let mut reads = 0;
    assert_eq!(
        inferred
            .evaluate_paths(CheckMode::Enabled, |_, _| {
                reads += 1;
                Ok(Value::Literal(42))
            })
            .unwrap(),
        Value::Literal(42)
    );
    assert_eq!(reads, 1);
    let input = expression("ifx false && unavailable else false");
    let inferred = DomainInference::infer(&input, |_, _| Ok(ScalarDomain::Bool)).unwrap();
    assert_eq!(
        inferred
            .evaluate_paths(CheckMode::Enabled, |_, _| panic!(
                "short circuit must skip the right operand"
            ))
            .unwrap(),
        Value::Bool(false)
    );
}
