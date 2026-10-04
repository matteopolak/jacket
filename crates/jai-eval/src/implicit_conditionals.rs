//! A capture borrows the identity of an actual written source node.
use super::*;
use std::ptr::NonNull;

pub(super) struct SourceCapture {
    pub(super) node: NonNull<Expression>,
    pub(super) value: Expr,
}

pub(super) fn captured(source: &Expression, captures: &[SourceCapture]) -> Option<Expr> {
    let node = NonNull::from(source);
    captures
        .iter()
        .rev()
        .find(|value| value.node == node)
        .map(|value| value.value.clone())
}

pub(super) fn bind_subject(
    source: &jai_syntax::ConditionalExpression,
    overflow_check: CheckMode,
    lookup: &mut impl FnMut(&NamePath, Span) -> Result<Value, Diagnostic>,
    captures: &mut Vec<SourceCapture>,
) -> Result<(BoolExpr, Expr), Diagnostic> {
    let subject = source.then_source();
    let value = bind_captured(subject, overflow_check, lookup, captures)?;
    let value = literal(finish_bound(value, subject.span)?);
    let checkpoint = captures.len();
    captures.push(SourceCapture {
        node: NonNull::from(subject),
        value: value.clone(),
    });
    let condition = bind_captured(&source.condition, overflow_check, lookup, captures);
    captures.truncate(checkpoint);
    Ok((condition?.condition(), value))
}
