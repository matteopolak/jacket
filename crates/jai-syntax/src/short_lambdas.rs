//! Short procedure expressions retain source parameters until contextual inference.
use super::*;

#[derive(Clone, Debug)]
pub struct ShortLambdaParameter {
    pub name: Symbol,
    pub ty: Option<TypeSyntax>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct ShortLambda {
    pub parameters: Vec<ShortLambdaParameter>,
    pub body: ShortLambdaBody,
}

#[derive(Clone, Debug)]
pub struct ShortLambdaBody {
    pub span: Span,
    pub kind: ShortLambdaBodyKind,
}

#[derive(Clone, Debug)]
pub enum ShortLambdaBodyKind {
    Expression(Expression),
    Block(Vec<Statement>),
}

impl Parser<'_> {
    pub(super) fn starts_short_lambda(&self) -> bool {
        if self.token().kind == Kind::Ident {
            return self
                .tokens
                .get(self.at + 1)
                .is_some_and(|next| next.kind == Kind::Punctuation(Punct::QuickLambda));
        }
        if !self.is(Punct::OpenParen) {
            return false;
        }
        let mut depth = 0;
        for (offset, token) in self.tokens[self.at..].iter().enumerate() {
            match token.kind {
                Kind::Punctuation(Punct::OpenParen) => depth += 1,
                Kind::Punctuation(Punct::CloseParen) => {
                    depth -= 1;
                    if depth == 0 {
                        return self.tokens.get(self.at + offset + 1).is_some_and(|next| {
                            next.kind == Kind::Punctuation(Punct::QuickLambda)
                        });
                    }
                }
                Kind::Eof => return false,
                _ => {}
            }
        }
        false
    }

    pub(super) fn short_lambda(&mut self) -> Result<Expression, Diagnostic> {
        let start = self.token().span.start;
        let mut parameters = Vec::new();
        if self.token().kind == Kind::Ident {
            let span = self.token().span;
            parameters.push(ShortLambdaParameter {
                name: self.name()?,
                ty: None,
                span,
            });
        } else {
            self.need(Punct::OpenParen)?;
            if !self.take(Punct::CloseParen) {
                loop {
                    let parameter_start = self.token().span.start;
                    let name = self.name()?;
                    let ty = if self.take(Punct::Colon) {
                        Some(if self.allow_qualified {
                            self.parameter_type_syntax()?
                        } else {
                            TypeSyntax::Builtin(BuiltinType::Scalar(self.scalar_type()?))
                        })
                    } else {
                        None
                    };
                    parameters.push(ShortLambdaParameter {
                        name,
                        ty,
                        span: Span::new(parameter_start, self.tokens[self.at - 1].span.end),
                    });
                    if self.take(Punct::CloseParen) {
                        break;
                    }
                    self.need(Punct::Comma)?;
                }
            }
        }
        self.need(Punct::QuickLambda)?;
        let body = if self.is(Punct::OpenBrace) {
            let start = self.token().span.start;
            let statements = self.block()?;
            ShortLambdaBody {
                span: Span::new(start, self.tokens[self.at - 1].span.end),
                kind: ShortLambdaBodyKind::Block(statements),
            }
        } else {
            let expression = self.expression(0)?;
            ShortLambdaBody {
                span: expression.span,
                kind: ShortLambdaBodyKind::Expression(expression),
            }
        };
        Ok(Expression {
            span: Span::new(start, body.span.end),
            kind: ExpressionKind::ShortLambda(Box::new(ShortLambda {
                parameters,
                body,
            })),
        })
    }
}
