//! Source requests for bounded compile-time execution.
use super::*;

#[derive(Clone, Debug)]
pub struct CompileTimeRun {
    pub flags: RunFlags,
    pub body: CompileTimeBody,
}

#[derive(Clone, Debug)]
pub enum CompileTimeBody {
    Expression(Box<Expression>),
    Block(Vec<Statement>),
    Procedure {
        result: TypeSyntax,
        body: Vec<Statement>,
    },
}

impl Parser<'_> {
    pub(super) fn compile_time(&mut self) -> Result<Expression, Diagnostic> {
        let start = self.token().span.start;
        self.at += 1;
        let flags = self.run_flags()?;
        if self.is(Punct::OpenParen)
            && self
                .tokens
                .get(self.at + 1)
                .is_some_and(|token| token.kind == Kind::Punctuation(Punct::CloseParen))
        {
            self.at += 2;
        }
        let body = if self.is(Punct::OpenBrace) {
            CompileTimeBody::Block(self.block()?)
        } else if self.take(Punct::Arrow) {
            let result = self.type_syntax()?;
            if !self.allow_qualified
                && result.as_scalar().is_none()
                && !matches!(result, TypeSyntax::Builtin(BuiltinType::Void))
            {
                return Err(self.error("anonymous #run result requires aggregate type resolution"));
            }
            CompileTimeBody::Procedure {
                result,
                body: self.block()?,
            }
        } else {
            CompileTimeBody::Expression(Box::new(self.expression(21)?))
        };
        Ok(Expression {
            span: Span::new(start, self.tokens[self.at - 1].span.end),
            kind: ExpressionKind::CompileTime(CompileTimeRun { flags, body }),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_time_calls_preserve_result_and_precedence() {
        let module = parse("answer :: #run factorial(5) + 2; main :: () {}").unwrap();
        let ExpressionKind::Binary(BinaryOp::Add, lhs, _) = &module.constants()[0].initializer.kind
        else {
            panic!("addition follows the compile-time call");
        };
        assert!(matches!(
            lhs.kind,
            ExpressionKind::CompileTime(CompileTimeRun {
                body: CompileTimeBody::Expression(_),
                ..
            })
        ));
        assert_eq!(lhs.span, Span::new(10, 27));
    }

    #[test]
    fn anonymous_run_bodies_retain_explicit_result_type() {
        let module =
            parse("main :: () { value := #run -> int { return 42; }; #run { value := 3; }; }")
                .unwrap();
        let StatementKind::Declare(Declaration::Inferred { initializer, .. }) =
            &module.procedures()[0].body[0].kind
        else {
            panic!("expected inferred declaration");
        };
        assert!(matches!(
            initializer.kind,
            ExpressionKind::CompileTime(CompileTimeRun {
                body: CompileTimeBody::Procedure { .. },
                ..
            })
        ));
        assert!(matches!(
            module.procedures()[0].body[1].kind,
            StatementKind::Expression(Expression {
                kind: ExpressionKind::CompileTime(CompileTimeRun {
                    body: CompileTimeBody::Block(_),
                    ..
                }),
                ..
            })
        ));
    }
}
