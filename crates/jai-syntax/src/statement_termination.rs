//! Only a direct, syntactically closed value can terminate its owning statement.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum StatementTerminator {
    Semicolon,
    ClosedValue,
}

impl Parser<'_> {
    pub(super) fn expression_terminator(&self, value: &Expression) -> StatementTerminator {
        let closing = self.tokens[self.at - 1].kind;
        let closed = match &value.kind {
            ExpressionKind::HereString(_) => closing == Kind::HereString,
            ExpressionKind::Type(TypeSyntax::InlineRecord(_) | TypeSyntax::InlineEnum(_))
            | ExpressionKind::AnonymousProcedure(_)
            | ExpressionKind::Code(CodeBody::Block(_))
            | ExpressionKind::CompileTime(CompileTimeRun {
                body:
                    CompileTimeBody::Block(_)
                    | CompileTimeBody::Procedure {
                        ..
                    },
                ..
            }) => closing == Kind::Punctuation(Punct::CloseBrace),
            _ => false,
        };
        if closed {
            StatementTerminator::ClosedValue
        } else {
            StatementTerminator::Semicolon
        }
    }

    pub(super) fn declaration_terminator(
        &self,
        declaration: &StatementKind,
    ) -> StatementTerminator {
        match declaration {
            StatementKind::Declare(Declaration::Inferred {
                initializer, ..
            })
            | StatementKind::Constant(ConstantDeclaration {
                initializer, ..
            })
            | StatementKind::Declare(
                Declaration::Explicit {
                    initializer: Some(initializer),
                    ..
                }
                | Declaration::UnresolvedExplicit {
                    initializer: Some(initializer),
                    ..
                },
            ) => self.expression_terminator(initializer),
            StatementKind::Declare(Declaration::UnresolvedExplicit {
                ty: TypeSyntax::InlineRecord(_) | TypeSyntax::InlineEnum(_),
                initializer: None,
                ..
            }) if self.tokens[self.at - 1].kind == Kind::Punctuation(Punct::CloseBrace) => {
                StatementTerminator::ClosedValue
            }
            _ => StatementTerminator::Semicolon,
        }
    }

    pub(super) fn finish_statement(
        &mut self,
        terminator: StatementTerminator,
    ) -> Result<(), Diagnostic> {
        match terminator {
            StatementTerminator::Semicolon => self.need(Punct::Semicolon),
            StatementTerminator::ClosedValue => {
                self.take(Punct::Semicolon);
                Ok(())
            }
        }
    }

    pub(super) fn finish_value_statement(
        &mut self,
        values: &[Expression],
    ) -> Result<(), Diagnostic> {
        let terminator = match values {
            [value] => self.expression_terminator(value),
            _ => StatementTerminator::Semicolon,
        };
        self.finish_statement(terminator)
    }

    pub(super) fn finish_expression_statement(
        &mut self,
        value: &Expression,
    ) -> Result<(), Diagnostic> {
        self.finish_statement(self.expression_terminator(value))
    }
}
