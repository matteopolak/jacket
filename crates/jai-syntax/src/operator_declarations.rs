//! Operator tokens have typed identities independent of procedure spellings.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OperatorKind {
    Unary(UnaryOp),
    Binary(BinaryOp),
    Compound(BinaryOp),
    Index,
    IndexAssign,
    IndexAddress,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OperatorDeclaration {
    pub kind: OperatorKind,
    pub symmetric: bool,
}

impl OperatorKind {
    pub fn arity(self) -> usize {
        match self {
            Self::Unary(_) => 1,
            Self::Binary(_) | Self::Compound(_) | Self::Index | Self::IndexAddress => 2,
            Self::IndexAssign => 3,
        }
    }

    pub fn shares_token(self, other: Self) -> bool {
        self == other
            || matches!(
                (self, other),
                (Self::Unary(UnaryOp::Positive), Self::Binary(BinaryOp::Add))
                    | (Self::Binary(BinaryOp::Add), Self::Unary(UnaryOp::Positive))
                    | (
                        Self::Unary(UnaryOp::Negate),
                        Self::Binary(BinaryOp::Subtract)
                    )
                    | (
                        Self::Binary(BinaryOp::Subtract),
                        Self::Unary(UnaryOp::Negate)
                    )
            )
    }
}

impl Parser<'_> {
    pub(super) fn operator_prefix(&mut self) -> Result<Option<(OperatorKind, Symbol)>, Diagnostic> {
        if !self.keyword(Keyword::Operator) {
            return Ok(None);
        }
        let start = self.token().span.start;
        let kind = if self.take(Punct::OpenBracket) {
            self.need(Punct::CloseBracket)?;
            if self.take(Punct::Assign) {
                OperatorKind::IndexAssign
            } else {
                OperatorKind::Index
            }
        } else if self.is(Punct::Mul)
            && self
                .tokens
                .get(self.at + 1)
                .is_some_and(|token| token.kind == Kind::Punctuation(Punct::OpenBracket))
        {
            self.at += 2;
            self.need(Punct::CloseBracket)?;
            OperatorKind::IndexAddress
        } else {
            let Kind::Punctuation(punctuation) = self.token().kind else {
                return Err(self.error("expected an overloadable operator token"));
            };
            let kind = if let Some(operation) = BinaryOp::compound(punctuation) {
                OperatorKind::Compound(operation)
            } else if let Some((operation, _)) = BinaryOp::parse(punctuation) {
                OperatorKind::Binary(operation)
            } else {
                OperatorKind::Unary(match punctuation {
                    Punct::Not => UnaryOp::LogicalNot,
                    Punct::Complement => UnaryOp::Complement,
                    _ => return Err(self.error("operator token cannot be overloaded")),
                })
            };
            self.at += 1;
            kind
        };
        let end = self.tokens[self.at - 1].span.end;
        // This label is for diagnostics and debug names; no name binding uses it.
        let label = self
            .symbols
            .intern(&format!("operator {}", &self.source[start..end]));
        Ok(Some((kind, label)))
    }

    pub(super) fn finish_operator_declaration(
        &self,
        mut kind: OperatorKind,
        parameters: &[Parameter],
        symmetric: bool,
        span: Span,
    ) -> Result<OperatorDeclaration, Diagnostic> {
        if let Some(parameter) = parameters
            .iter()
            .find(|parameter| parameter.evaluation != ParameterEvaluation::Evaluate)
        {
            return Err(Diagnostic::new(
                parameter.span,
                "discarded operator parameters require source-aware captured argument binding",
            ));
        }
        if parameters.len() == 1 {
            kind = match kind {
                OperatorKind::Binary(BinaryOp::Add) => OperatorKind::Unary(UnaryOp::Positive),
                OperatorKind::Binary(BinaryOp::Subtract) => OperatorKind::Unary(UnaryOp::Negate),
                other => other,
            };
        }
        if parameters.len() < kind.arity()
            || parameters.iter().any(|parameter| parameter.variadic)
            || parameters.iter().skip(kind.arity()).any(|parameter| {
                !matches!(
                    parameter.binding,
                    ParameterBinding::Defaulted { .. } | ParameterBinding::DefaultedType { .. }
                )
            })
        {
            return Err(Diagnostic::new(
                span,
                "operator declaration has an invalid operand signature",
            ));
        }
        if symmetric && !matches!(kind, OperatorKind::Binary(_)) {
            return Err(Diagnostic::new(
                span,
                "#symmetric requires a binary operator",
            ));
        }
        Ok(OperatorDeclaration { kind, symmetric })
    }
}
