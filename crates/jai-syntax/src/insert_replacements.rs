//! Insertion modifiers retain source bodies for lazy loop-control substitution.
use super::*;

#[derive(Clone, Debug)]
pub struct LoopControlReplacement {
    pub kind: JumpKind,
    pub body: LoopControlReplacementBody,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum LoopControlReplacementBody {
    Code(CodeBody),
    Assert {
        condition: Box<Expression>,
        span: Span,
    },
}

impl Parser<'_> {
    pub(super) fn insert_replacements(
        &mut self,
    ) -> Result<Vec<LoopControlReplacement>, Diagnostic> {
        if !self.is(Punct::OpenParen)
            || self.tokens.get(self.at + 2).map(|token| token.kind)
                != Some(Kind::Punctuation(Punct::Assign))
        {
            return Ok(Vec::new());
        }
        self.at += 1;
        let mut replacements: Vec<LoopControlReplacement> = Vec::new();
        loop {
            let start = self.token().span.start;
            let kind = match self.token().kind {
                Kind::Keyword(Keyword::Break) => JumpKind::Break,
                Kind::Keyword(Keyword::Continue) => JumpKind::Continue,
                Kind::Keyword(Keyword::Remove) => JumpKind::Remove,
                _ => {
                    return Err(
                        self.error("insertion replacement must name break, continue, or remove")
                    );
                }
            };
            if replacements
                .iter()
                .any(|replacement| replacement.kind == kind)
            {
                return Err(self.error("duplicate loop-control insertion replacement"));
            }
            self.at += 1;
            self.need(Punct::Assign)?;
            let body = self.insert_replacement_body()?;
            replacements.push(LoopControlReplacement {
                kind,
                body,
                span: Span::new(start, self.tokens[self.at - 1].span.end),
            });
            if !self.take(Punct::Comma) {
                self.need(Punct::CloseParen)?;
                break;
            }
        }
        Ok(replacements)
    }

    fn insert_replacement_body(&mut self) -> Result<LoopControlReplacementBody, Diagnostic> {
        if self.is(Punct::OpenBrace) {
            return Ok(LoopControlReplacementBody::Code(CodeBody::Block(
                self.block()?,
            )));
        }
        if self.token().kind == Kind::Directive(Directive::Assert) {
            let start = self.token().span.start;
            self.at += 1;
            let condition = self.expression(0)?;
            return Ok(LoopControlReplacementBody::Assert {
                span: Span::new(start, condition.span.end),
                condition: Box::new(condition),
            });
        }
        let kind = match self.token().kind {
            Kind::Keyword(Keyword::Break) => Some(JumpKind::Break),
            Kind::Keyword(Keyword::Continue) => Some(JumpKind::Continue),
            Kind::Keyword(Keyword::Remove) => Some(JumpKind::Remove),
            _ => None,
        };
        if let Some(kind) = kind {
            let token_span = self.token().span;
            self.at += 1;
            let target = if self.token().kind == Kind::Ident {
                LoopTarget::Named(self.name()?)
            } else if kind == JumpKind::Remove {
                return Err(Diagnostic::new(
                    token_span,
                    "remove requires an array iterator name",
                ));
            } else {
                LoopTarget::Innermost
            };
            let span = Span::new(token_span.start, self.tokens[self.at - 1].span.end);
            return Ok(LoopControlReplacementBody::Code(CodeBody::Statement(
                Box::new(Statement::new(
                    span,
                    StatementKind::Jump {
                        kind,
                        target,
                        span: token_span,
                    },
                )),
            )));
        }
        Ok(LoopControlReplacementBody::Code(CodeBody::Expression(
            Box::new(self.expression(0)?),
        )))
    }
}
