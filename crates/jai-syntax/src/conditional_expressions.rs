//! Implicit arms retain the actual subject inside their written condition.
use super::*;

impl ConditionalExpression {
    pub fn is_implicit(&self) -> bool {
        matches!(self.then_value, ConditionalThenValue::ImplicitSubject)
    }
    pub fn explicit_then(&self) -> Option<&Expression> {
        match &self.then_value {
            ConditionalThenValue::Expression(value) => Some(value),
            ConditionalThenValue::ImplicitSubject => None,
        }
    }
    /// Source navigation only: this never clones or evaluates the subject.
    pub fn then_source(&self) -> &Expression {
        self.explicit_then()
            .unwrap_or_else(|| implicit_subject(&self.condition))
    }
    pub fn expressions(&self) -> impl Iterator<Item = &Expression> {
        [
            Some(self.condition.as_ref()),
            self.explicit_then(),
            self.else_value.as_deref(),
        ]
        .into_iter()
        .flatten()
    }
    pub fn expressions_mut(&mut self) -> impl Iterator<Item = &mut Expression> {
        let yes = match &mut self.then_value {
            ConditionalThenValue::Expression(value) => Some(value.as_mut()),
            ConditionalThenValue::ImplicitSubject => None,
        };
        [
            Some(self.condition.as_mut()),
            yes,
            self.else_value.as_deref_mut(),
        ]
        .into_iter()
        .flatten()
    }
}

/// One boolean operator may expose its operand, followed by first call arguments.
fn implicit_subject(mut source: &Expression) -> &Expression {
    let mut boolean_available = true;
    loop {
        source = match &source.kind {
            ExpressionKind::Unary(UnaryOp::LogicalNot, value) if boolean_available => {
                boolean_available = false;
                value
            }
            ExpressionKind::Binary(op, left, _)
                if boolean_available
                    && matches!(
                        op,
                        BinaryOp::Equal
                            | BinaryOp::NotEqual
                            | BinaryOp::Less
                            | BinaryOp::LessEqual
                            | BinaryOp::Greater
                            | BinaryOp::GreaterEqual
                            | BinaryOp::LogicalAnd
                            | BinaryOp::LogicalOr
                    ) =>
            {
                boolean_available = false;
                left
            }
            ExpressionKind::Call(_, args) | ExpressionKind::QualifiedCall(_, args)
                if !args.is_empty() =>
            {
                &args[0].value
            }
            ExpressionKind::CallHint {
                call, ..
            } => call,
            _ => return source,
        };
    }
}
impl Parser<'_> {
    pub(super) fn conditional_expression(&mut self, span: Span) -> Result<Expression, Diagnostic> {
        let condition = Box::new(self.expression(0)?);
        if self.is(Punct::OpenBrace) {
            return Err(self.error("ifx branch blocks require statement-valued expression support"));
        }
        let written_then = self.keyword(Keyword::Then);
        let implicit = !written_then
            && (self.token().kind == Kind::Keyword(Keyword::Else)
                || matches!(
                    self.token().kind,
                    Kind::Eof
                        | Kind::Punctuation(
                            Punct::Semicolon
                                | Punct::Comma
                                | Punct::CloseParen
                                | Punct::CloseBracket
                                | Punct::CloseBrace
                        )
                ));
        let then_value = if implicit {
            ConditionalThenValue::ImplicitSubject
        } else {
            ConditionalThenValue::Expression(Box::new(self.expression(0)?))
        };
        let else_value = if self.keyword(Keyword::Else) {
            Some(Box::new(self.expression(0)?))
        } else {
            None
        };
        let end = else_value.as_ref().map_or_else(
            || match &then_value {
                ConditionalThenValue::Expression(value) => value.span.end,
                ConditionalThenValue::ImplicitSubject => condition.span.end,
            },
            |value| value.span.end,
        );
        Ok(Expression {
            span: Span::new(span.start, end),
            kind: ExpressionKind::Conditional(ConditionalExpression {
                condition,
                then_value,
                else_value,
            }),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn conditional(text: &str) -> ConditionalExpression {
        let parsed = parse(&format!("VALUE :: {text}; main :: () {{}}")).unwrap();
        let ExpressionKind::Conditional(value) = &parsed.constants()[0].initializer.kind else {
            panic!("conditional AST required");
        };
        value.clone()
    }
    #[test]
    fn omitted_arms_retain_actual_subject_and_distinct_written_children() {
        for text in ["ifx value", "ifx value else 1"] {
            let value = conditional(text);
            assert!(value.is_implicit());
            assert!(std::ptr::eq(value.then_source(), value.condition.as_ref()));
            assert!(value.explicit_then().is_none());
            assert_eq!(
                value.expressions().count(),
                if value.else_value.is_some() {
                    2
                } else {
                    1
                }
            );
        }
    }
    #[test]
    fn comparison_and_nested_direct_calls_select_the_written_leaf() {
        let text = "ifx accept(add(value, later())) > 0 else 7";
        let value = conditional(text);
        assert!(matches!(value.then_source().kind, ExpressionKind::Name(_)));
        let full = format!("VALUE :: {text}; main :: () {{}}");
        assert_eq!(value.then_source().span.text(&full), "value");
    }
    #[test]
    fn only_one_boolean_operator_is_unwrapped() {
        let value = conditional("ifx value > 0 && enabled else false");
        assert!(matches!(
            value.then_source().kind,
            ExpressionKind::Binary(BinaryOp::Greater, _, _)
        ));
        let value = conditional("ifx !accept(value) else 9");
        assert!(matches!(value.then_source().kind, ExpressionKind::Name(_)));
    }
    #[test]
    fn explicit_optional_keyword_and_nearest_else_keep_their_ast() {
        for text in ["ifx true then 41 else 42", "ifx true 41 else 42"] {
            let value = conditional(text);
            assert!(!value.is_implicit());
            assert_eq!(value.expressions().count(), 3);
            assert!(matches!(
                value.then_source().kind,
                ExpressionKind::Integer(41)
            ));
        }
        let value = conditional("ifx true then ifx count else 41 else 42");
        assert!(matches!(
            value.then_source().kind,
            ExpressionKind::Conditional(_)
        ));
        assert!(matches!(
            value.else_value.unwrap().kind,
            ExpressionKind::Integer(42)
        ));
    }
    #[test]
    fn implicit_arms_stop_at_real_expression_delimiters() {
        for source in [
            "main::(){ value:=f(ifx x, ifx y); }",
            "main::(){ value:=.[ifx x, ifx y]; }",
            "main::(){ value:=(ifx x); }",
            "main::(){ value:=ifx x; }",
        ] {
            let mut sources = jai_source::SourceMap::default();
            let id = sources.insert("implicit-delimiters.jai".into(), source.into());
            assert!(
                parse_file(sources.get(id).unwrap(), &mut Symbols::default()).is_ok(),
                "{source}"
            );
        }
    }
    #[test]
    fn malformed_written_then_and_branch_blocks_remain_errors() {
        for source in [
            "VALUE::ifx true then;",
            "VALUE::ifx true then else 1;",
            "VALUE::ifx true else;",
            "VALUE::ifx;",
        ] {
            assert!(parse(source).is_err(), "{source}");
        }
        let error = parse("VALUE::ifx true { 1; } else { 2; }").unwrap_err();
        assert!(
            error
                .message
                .contains("statement-valued expression support"),
            "{error}"
        );
    }
}
