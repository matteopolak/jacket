//! Scalar enum guards use the same integer/bool truth convention as source record guards.
use jai_source::Diagnostic;
pub(crate) fn truth(value: jai_eval::Value, span: jai_source::Span) -> Result<bool, Diagnostic> {
    match value {
        jai_eval::Value::Bool(value) => Ok(value),
        jai_eval::Value::Literal(value) => Ok(value != 0),
        jai_eval::Value::Int(value) => Ok(value.value() != 0),
        _ => Err(Diagnostic::new(
            span,
            "enum #if condition requires a scalar boolean or integer",
        )),
    }
}
