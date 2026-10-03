//! Quoted source subtrees and insertion requests retain their defining syntax.
use super::*;

#[derive(Clone, Debug)]
pub enum CodeBody {
    Null,
    Expression(Box<Expression>),
    Block(Vec<Statement>),
    Statement(Box<Statement>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InsertScope {
    Captured,
    Current,
}
#[derive(Clone, Debug)]
pub struct InsertDirective {
    pub value: Expression,
    pub scope: InsertScope,
    pub replacements: Vec<LoopControlReplacement>,
    pub span: Span,
}
impl Parser<'_> {
    pub(super) fn insert_terminator(
        &mut self,
        directive: &InsertDirective,
    ) -> Result<(), Diagnostic> {
        if matches!(
            &directive.value.kind,
            ExpressionKind::CompileTime(CompileTimeRun {
                body: CompileTimeBody::Block(_) | CompileTimeBody::Procedure { .. },
                ..
            })
        ) {
            self.take(Punct::Semicolon);
            Ok(())
        } else {
            self.need(Punct::Semicolon)
        }
    }
    pub(super) fn code_syntax(&mut self) -> Result<Expression, Diagnostic> {
        let start = self.token().span.start;
        self.at += 1;
        if self.take(Punct::Comma) {
            if !self.keyword(Keyword::Null) {
                return Err(self.error("expected null code modifier after '#code,'"));
            }
            return Ok(Expression {
                kind: ExpressionKind::Code(CodeBody::Null),
                span: Span::new(start, self.tokens[self.at - 1].span.end),
            });
        }
        let payload_start = self.at;
        let statement = self.import_prefix()
            || self.is(Punct::Backtick)
            || matches!(
                self.token().kind,
                Kind::Directive(
                    Directive::Import
                        | Directive::If
                        | Directive::Assert
                        | Directive::AddContext
                        | Directive::NoArrayBoundsCheck
                        | Directive::NoArithmeticOverflowCheck
                ) | Kind::Keyword(
                    Keyword::Return
                        | Keyword::If
                        | Keyword::For
                        | Keyword::While
                        | Keyword::Defer
                        | Keyword::Using
                        | Keyword::Break
                        | Keyword::Continue
                        | Keyword::Remove
                        | Keyword::PushContext
                )
            )
            || matches!(
                self.tokens.get(self.at + 1).map(|token| token.kind),
                Some(Kind::Punctuation(
                    Punct::Infer | Punct::Colon | Punct::Constant | Punct::Assign
                ))
            )
            || matches!(self.tokens.get(self.at+1).map(|token| token.kind),Some(Kind::Punctuation(p)) if BinaryOp::compound(p).is_some());
        let body = if self.is(Punct::OpenBrace) {
            CodeBody::Block(self.block()?)
        } else if statement {
            CodeBody::Statement(Box::new(self.statement()?))
        } else {
            let value = self.expression(0)?;
            if self.is(Punct::Assign)
                || matches!(self.token().kind, Kind::Punctuation(p) if BinaryOp::compound(p).is_some())
            {
                self.at = payload_start;
                CodeBody::Statement(Box::new(self.statement()?))
            } else {
                CodeBody::Expression(Box::new(value))
            }
        };
        let end = self.tokens[self.at - 1].span.end;
        if matches!(body, CodeBody::Statement(_))
            && self.tokens[self.at - 1].kind == Kind::Punctuation(Punct::Semicolon)
        {
            self.at -= 1;
        }
        Ok(Expression {
            kind: ExpressionKind::Code(body),
            span: Span::new(start, end),
        })
    }
    pub(super) fn insert_directive(&mut self, minimum: u8) -> Result<InsertDirective, Diagnostic> {
        let start = self.token().span.start;
        self.at += 1;
        let scope = if self.take(Punct::Comma) {
            if self.token().kind != Kind::Ident || self.text() != "scope" {
                return Err(self.error("expected scope() insertion modifier"));
            }
            self.at += 1;
            self.need(Punct::OpenParen)?;
            self.need(Punct::CloseParen)?;
            InsertScope::Current
        } else {
            InsertScope::Captured
        };
        let replacements = self.insert_replacements()?;
        let value = if self.take(Punct::Arrow) {
            let procedure_start = self.tokens[self.at - 1].span.start;
            let result = self.type_syntax()?;
            let body = self.block()?;
            Expression {
                span: Span::new(procedure_start, self.tokens[self.at - 1].span.end),
                kind: ExpressionKind::CompileTime(CompileTimeRun {
                    flags: RunFlags::default(),
                    body: CompileTimeBody::Procedure {
                        result,
                        body,
                    },
                }),
            }
        } else {
            self.expression(minimum)?
        };
        Ok(InsertDirective {
            span: Span::new(start, value.span.end),
            value,
            scope,
            replacements,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_source::SourceMap;

    fn parse_source(source: &str) -> ParsedFile {
        let mut sources = SourceMap::default();
        let id = sources.insert("code.jai".into(), source.into());
        parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap()
    }

    #[test]
    fn null_code_has_a_distinct_marker_from_a_quoted_null_expression() {
        let text = "absent :: #code,null; quoted :: #code null;";
        let parsed = parse_source(text);
        let constants: Vec<_> = parsed
            .items()
            .iter()
            .map(|item| {
                let FileItem::Declaration(FileDeclaration {
                    kind: FileDeclarationKind::Constant(constant),
                    ..
                }) = item
                else {
                    panic!("expected constant")
                };
                constant
            })
            .collect();
        assert!(matches!(
            constants[0].initializer.kind,
            ExpressionKind::Code(CodeBody::Null)
        ));
        assert_eq!(constants[0].initializer.span.text(text), "#code,null");
        assert!(
            matches!(&constants[1].initializer.kind, ExpressionKind::Code(CodeBody::Expression(value)) if matches!(value.kind, ExpressionKind::Null))
        );
    }

    #[test]
    fn quoted_expressions_blocks_and_place_statements_retain_the_subtree() {
        let parsed = parse_source(
            "increment :: #code point.x += 1; body :: #code { return 3; }; call :: #code work(3); main :: () { #insert body; #insert,scope() call; result := #insert make_code(); }",
        );
        let FileItem::Declaration(FileDeclaration {
            kind: FileDeclarationKind::Constant(increment),
            ..
        }) = &parsed.items()[0]
        else {
            panic!("expected constant")
        };
        assert!(
            matches!(&increment.initializer.kind, ExpressionKind::Code(CodeBody::Statement(statement)) if matches!(&statement.kind, StatementKind::UpdatePlace { .. }))
        );
        let FileItem::Declaration(FileDeclaration {
            kind: FileDeclarationKind::Constant(body),
            ..
        }) = &parsed.items()[1]
        else {
            panic!("expected constant")
        };
        assert!(
            matches!(&body.initializer.kind, ExpressionKind::Code(CodeBody::Block(statements)) if statements.len()==1)
        );
        let FileItem::Declaration(FileDeclaration {
            kind: FileDeclarationKind::Constant(call),
            ..
        }) = &parsed.items()[2]
        else {
            panic!("expected constant")
        };
        assert!(
            matches!(&call.initializer.kind, ExpressionKind::Code(CodeBody::Expression(expression)) if matches!(expression.kind, ExpressionKind::Call(_, _)))
        );
        let FileItem::Declaration(FileDeclaration {
            kind: FileDeclarationKind::Procedure(main),
            ..
        }) = &parsed.items()[3]
        else {
            panic!("expected procedure")
        };
        assert!(matches!(
            &main.body[0].kind,
            StatementKind::Insert(InsertDirective {
                scope: InsertScope::Captured,
                ..
            })
        ));
        assert!(matches!(
            &main.body[1].kind,
            StatementKind::Insert(InsertDirective {
                scope: InsertScope::Current,
                ..
            })
        ));
        assert!(matches!(
            &main.body[2].kind,
            StatementKind::Declare(Declaration::Inferred {
                initializer: Expression {
                    kind: ExpressionKind::Insert(_),
                    ..
                },
                ..
            })
        ));
    }

    #[test]
    fn anonymous_insert_sugar_retains_a_typed_compile_time_body() {
        let parsed = parse_source(
            "main :: () { #insert -> string { return \"x *= x;\"; } #insert -> Code { return #code x = (x * 10) + 3495; } }",
        );
        let FileItem::Declaration(FileDeclaration {
            kind: FileDeclarationKind::Procedure(main),
            ..
        }) = &parsed.items()[0]
        else {
            panic!("expected procedure")
        };
        let StatementKind::Insert(first) = &main.body[0].kind else {
            panic!("expected insertion")
        };
        assert!(matches!(
            &first.value.kind,
            ExpressionKind::CompileTime(CompileTimeRun {
                body: CompileTimeBody::Procedure {
                    result: TypeSyntax::Builtin(BuiltinType::String),
                    ..
                },
                ..
            })
        ));
        let StatementKind::Insert(second) = &main.body[1].kind else {
            panic!("expected insertion")
        };
        let ExpressionKind::CompileTime(CompileTimeRun {
            body: CompileTimeBody::Procedure {
                body, ..
            },
            ..
        }) = &second.value.kind
        else {
            panic!("expected anonymous compiletime body")
        };
        assert!(
            matches!(&body[0].kind, StatementKind::Return(Some(Expression { kind: ExpressionKind::Code(CodeBody::Statement(statement)), .. })) if matches!(&statement.kind, StatementKind::Assign(_, _)))
        );
    }

    #[test]
    fn quoted_arithmetic_is_one_expression_tree_inside_call_delimiters() {
        let parsed = parse_source(
            "integer_code :: #code 41 + 1; main :: () { consume(#code left + right * 2, 3); }",
        );
        let FileItem::Declaration(FileDeclaration {
            kind: FileDeclarationKind::Constant(constant),
            ..
        }) = &parsed.items()[0]
        else {
            panic!("expected constant")
        };
        assert!(
            matches!(&constant.initializer.kind, ExpressionKind::Code(CodeBody::Expression(expression)) if matches!(expression.kind, ExpressionKind::Binary(BinaryOp::Add, _, _)))
        );
        let FileItem::Declaration(FileDeclaration {
            kind: FileDeclarationKind::Procedure(main),
            ..
        }) = &parsed.items()[1]
        else {
            panic!("expected procedure")
        };
        let StatementKind::Expression(Expression {
            kind: ExpressionKind::Call(_, arguments),
            ..
        }) = &main.body[0].kind
        else {
            panic!("expected call")
        };
        assert_eq!(arguments.len(), 2);
        assert!(
            matches!(&arguments[0].value.kind, ExpressionKind::Code(CodeBody::Expression(expression)) if matches!(&expression.kind, ExpressionKind::Binary(BinaryOp::Add, _, right) if matches!(right.kind, ExpressionKind::Binary(BinaryOp::Multiply, _, _))))
        );
    }

    #[test]
    fn unknown_compile_directive_has_a_specific_located_diagnostic() {
        let text = "value := #invented_compile_directive 3;";
        let mut sources = SourceMap::default();
        let id = sources.insert("unknown.jai".into(), text.into());
        let error = parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap_err();
        assert_eq!(
            error.message,
            "unknown directive '#invented_compile_directive'"
        );
        assert_eq!(
            error.location.span.text(text),
            "#invented_compile_directive"
        );
    }
}
