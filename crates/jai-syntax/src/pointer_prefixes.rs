//! The parenthesized dereference operator uses the ordinary typed pointer node.
use super::*;

impl Parser<'_> {
    pub(super) fn starts_parenthesized_dereference(&self) -> bool {
        self.is(Punct::OpenParen)
            && self
                .tokens
                .get(self.at + 1)
                .is_some_and(|token| token.kind == Kind::Punctuation(Punct::PostfixDeref))
            && self
                .tokens
                .get(self.at + 2)
                .is_some_and(|token| token.kind == Kind::Punctuation(Punct::CloseParen))
    }

    pub(super) fn parenthesized_dereference(&mut self) -> Result<Expression, Diagnostic> {
        let start = self.token().span.start;
        if !self.allow_qualified {
            return Err(self.error("dereference requires pointer and storage resolution"));
        }
        self.at += 3;
        let value = self.expression(21)?;
        Ok(Expression {
            span: Span::new(start, value.span.end),
            kind: ExpressionKind::Dereference(Box::new(value)),
        })
    }
}
