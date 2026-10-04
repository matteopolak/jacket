//! Source conditionals retain both branches until target-aware semantic selection.
use super::*;

impl Parser<'_> {
    /// Accept both `#assert condition "message";` and `#assert(condition, "message");`.
    pub(super) fn assertion_arguments(
        &mut self,
    ) -> Result<(Expression, Option<Expression>), Diagnostic> {
        let operands = self.assertion_operands(true)?;
        self.need(Punct::Semicolon)?;
        Ok(operands)
    }

    /// Replacement directives omit the terminator and reserve bare commas for their list.
    pub(super) fn assertion_operands(
        &mut self,
        comma_message: bool,
    ) -> Result<(Expression, Option<Expression>), Diagnostic> {
        if !self.allow_qualified {
            return Err(self.error("#assert requires source condition resolution"));
        }
        self.at += 1;
        let mut closes = Vec::new();
        let mut comma_arguments = false;
        if self.is(Punct::OpenParen) {
            for token in &self.tokens[self.at..] {
                match token.kind {
                    Kind::Punctuation(Punct::OpenParen) => closes.push(Punct::CloseParen),
                    Kind::Punctuation(Punct::OpenBracket | Punct::ArrayLiteral) => {
                        closes.push(Punct::CloseBracket)
                    }
                    Kind::Punctuation(Punct::OpenBrace | Punct::StructLiteral) => {
                        closes.push(Punct::CloseBrace)
                    }
                    Kind::Punctuation(
                        close @ (Punct::CloseParen | Punct::CloseBracket | Punct::CloseBrace),
                    ) => {
                        if closes.pop() != Some(close) {
                            break;
                        }
                        if closes.is_empty() {
                            break;
                        }
                    }
                    Kind::Punctuation(Punct::Comma) if closes.len() == 1 => comma_arguments = true,
                    _ => {}
                }
            }
        }
        if comma_arguments {
            self.need(Punct::OpenParen)?;
        }
        let condition = self.expression(0)?;
        let message = if comma_arguments {
            self.need(Punct::Comma)?;
            let message = self.expression(0)?;
            self.need(Punct::CloseParen)?;
            Some(message)
        } else if (comma_message && self.take(Punct::Comma)) || self.token().kind == Kind::String {
            Some(self.expression(0)?)
        } else {
            None
        };
        Ok((condition, message))
    }

    pub(super) fn statement_conditional(&mut self) -> Result<StatementKind, Diagnostic> {
        if !self.allow_qualified {
            return Err(self.error("#if requires source condition resolution"));
        }
        let start = self.token().span.start;
        self.at += 1;
        let (condition, operator, complete) = self.source_conditional_header()?;
        if let Some(operator) = operator {
            return Ok(StatementKind::CompileTimeCases(self.source_cases(
                start,
                condition,
                operator,
                complete,
                Self::source_case_statements,
            )?));
        }
        let then_body = self.body()?;
        let else_body = if self.keyword(Keyword::Else) {
            self.body()?
        } else {
            Vec::new()
        };
        Ok(StatementKind::CompileTimeIf {
            condition,
            then_body,
            else_body,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_source::SourceMap;

    #[test]
    fn runtime_support_target_branches_and_nested_selection_keep_source_spans() {
        let text = "write :: (s: string) #no_context { #if OS == .WINDOWS { WriteFile(s); } else #if OS == .ANDROID { android_write(s); } else { write_unix(s); } }";
        let mut sources = SourceMap::default();
        let id = sources.insert("runtime-selection.jai".into(), text.into());
        let parsed = parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap();
        let FileItem::Declaration(FileDeclaration {
            kind: FileDeclarationKind::Procedure(procedure),
            ..
        }) = &parsed.items()[0]
        else {
            panic!("expected procedure")
        };
        let branch = &procedure.body[0];
        let StatementKind::CompileTimeIf {
            condition,
            then_body,
            else_body,
        } = &branch.kind
        else {
            panic!("expected source conditional")
        };
        assert_eq!(condition.span.text(text), "OS == .WINDOWS");
        assert_eq!(then_body[0].span.text(text), "WriteFile(s);");
        assert!(branch.span.text(text).starts_with("#if OS"));
        let nested = &else_body[0];
        let StatementKind::CompileTimeIf {
            condition,
            then_body,
            else_body,
        } = &nested.kind
        else {
            panic!("expected nested conditional")
        };
        assert_eq!(condition.span.text(text), "OS == .ANDROID");
        assert_eq!(then_body[0].span.text(text), "android_write(s);");
        assert_eq!(else_body[0].span.text(text), "write_unix(s);");
        assert!(nested.span.text(text).starts_with("#if OS == .ANDROID"));
    }

    #[test]
    fn quoted_single_statement_conditionals_retain_both_branches() {
        let text = "selection :: #code #if ENABLED then chosen(); else rejected();";
        let mut sources = SourceMap::default();
        let id = sources.insert("quoted-selection.jai".into(), text.into());
        let parsed = parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap();
        let FileItem::Declaration(FileDeclaration {
            kind: FileDeclarationKind::Constant(constant),
            ..
        }) = &parsed.items()[0]
        else {
            panic!("expected constant")
        };
        let ExpressionKind::Code(CodeBody::Statement(statement)) = &constant.initializer.kind
        else {
            panic!("expected statement quote")
        };
        let StatementKind::CompileTimeIf {
            then_body,
            else_body,
            ..
        } = &statement.kind
        else {
            panic!("expected conditional")
        };
        assert_eq!(then_body.len(), 1);
        assert_eq!(else_body.len(), 1);
        assert_eq!(
            parse("main :: () { #if true {} }").unwrap_err().message,
            "#if requires source condition resolution"
        );
    }
}

#[cfg(test)]
mod assertion_tests {
    use super::*;
    use jai_source::SourceMap;

    #[test]
    fn file_and_body_assertions_retain_condition_message_and_directive_spans() {
        let source = "#assert CPU == .X64 \"unsupported architecture\"; main::(){ #assert(!(flags & .REVERSE)); #assert(false, \"unsupported flags\"); #assert (a) == b \"grouped comparison\"; }";
        let mut sources = SourceMap::default();
        let id = sources.insert("assertions.jai".into(), source.into());
        let parsed = parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap();
        let FileItem::Assert {
            condition,
            message: Some(message),
            location,
        } = &parsed.items()[0]
        else {
            panic!("file assertion");
        };
        assert_eq!(condition.span.text(source), "CPU == .X64");
        assert_eq!(message.span.text(source), "\"unsupported architecture\"");
        assert_eq!(
            location.span.text(source),
            "#assert CPU == .X64 \"unsupported architecture\";"
        );
        let FileItem::Declaration(FileDeclaration {
            kind: FileDeclarationKind::Procedure(procedure),
            ..
        }) = &parsed.items()[1]
        else {
            panic!("procedure");
        };
        let StatementKind::CompileTimeAssert {
            condition,
            message: None,
        } = &procedure.body[0].kind
        else {
            panic!("body assertion");
        };
        assert_eq!(condition.span.text(source), "(!(flags & .REVERSE))");
        assert_eq!(
            procedure.body[1].span.text(source),
            "#assert(false, \"unsupported flags\");"
        );
        let StatementKind::CompileTimeAssert {
            condition,
            message: Some(_),
        } = &procedure.body[2].kind
        else {
            panic!("grouped assertion");
        };
        assert_eq!(condition.span.text(source), "(a) == b");
    }
}
