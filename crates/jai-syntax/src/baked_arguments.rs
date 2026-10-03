//! Retain partial-application source without manufacturing a smaller procedure.
use super::*;

#[derive(Clone, Debug)]
pub struct BakedArgumentsSyntax {
    pub callee: Box<Expression>,
    pub arguments: Vec<CallArgument>,
    pub call_span: Span,
    pub span: Span,
}

impl Parser<'_> {
    pub(super) fn baked_arguments(&mut self) -> Result<BakedArgumentsSyntax, Diagnostic> {
        let directive = self.token();
        if directive.kind != Kind::Directive(Directive::BakeArguments) {
            return Err(self.error("expected #bake_arguments"));
        }
        if !self.allow_qualified {
            return Err(self.error("#bake_arguments requires checked callable source resolution"));
        }
        self.at += 1;
        let call_start = self.at;
        let call = self.expression_until_call(23, true)?;
        let call_span = call.span;
        let named_span = self.baked_named_callee_span(call_start, call_span);
        let (callee, arguments) = match call.kind {
            ExpressionKind::Call(name, arguments) => (
                Expression {
                    span: named_span,
                    kind: ExpressionKind::Name(name),
                },
                arguments,
            ),
            ExpressionKind::QualifiedCall(path, arguments) => (
                Expression {
                    span: named_span,
                    kind: ExpressionKind::QualifiedName(path),
                },
                arguments,
            ),
            ExpressionKind::IndirectCall {
                callee,
                args,
            } => (*callee, args),
            ExpressionKind::ContextCall {
                ..
            } => {
                return Err(Diagnostic::new(
                    call_span,
                    "#bake_arguments cannot bake a call-context override group",
                ));
            }
            _ => {
                return Err(Diagnostic::new(
                    call_span,
                    "#bake_arguments requires a callable target with an argument list",
                ));
            }
        };
        Ok(BakedArgumentsSyntax {
            callee: Box::new(callee),
            arguments,
            call_span,
            span: Span::new(directive.span.start, call_span.end),
        })
    }

    fn baked_named_callee_span(&self, start: usize, call_span: Span) -> Span {
        let mut depth = 0;
        for index in (start..self.at).rev() {
            match self.tokens[index].kind {
                Kind::Punctuation(Punct::CloseParen) => depth += 1,
                Kind::Punctuation(Punct::OpenParen) => {
                    depth -= 1;
                    if depth == 0 && index > start {
                        return Span::new(call_span.start, self.tokens[index - 1].span.end);
                    }
                }
                _ => {}
            }
        }
        // Used only after a checked Call/QualifiedCall; another expression is
        // diagnosed by the caller rather than becoming a fabricated callee.
        call_span
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parser(source: &str) -> Parser<'_> {
        Parser {
            source,
            tokens: lex(source).unwrap(),
            at: 0,
            symbols: Symbols::default(),
            allow_qualified: true,
            record_conditional_depth: 0,
            file_conditional_depth: 0,
        }
    }

    #[test]
    fn named_and_computed_targets_keep_original_callee_and_argument_ranges() {
        for (source, callee) in [
            (
                "#bake_arguments only_set_or_unset(target_value = true)",
                "only_set_or_unset",
            ),
            (
                "#bake_arguments Library.walk(target_value = FLAG)",
                "Library.walk",
            ),
            (
                "#bake_arguments callbacks[index](target_value = true)",
                "callbacks[index]",
            ),
            ("#bake_arguments (walk)(target_value = true)", "(walk)"),
        ] {
            let mut parser = parser(source);
            let value = parser.baked_arguments().unwrap();
            assert_eq!(value.span.text(source), source);
            assert_eq!(
                value.call_span.text(source),
                source.trim_start_matches("#bake_arguments ")
            );
            assert_eq!(value.callee.span.text(source), callee);
            assert_eq!(value.arguments.len(), 1);
            assert_eq!(
                parser.symbols.name(value.arguments[0].name.unwrap()),
                "target_value"
            );
            assert_eq!(parser.token().kind, Kind::Eof);
        }
    }

    #[test]
    fn baked_application_stops_before_a_later_invocation_or_binary_operand() {
        for (source, next) in [
            ("#bake_arguments walk(flag=true)(body)", Punct::OpenParen),
            ("#bake_arguments walk(flag=true) + offset", Punct::Add),
        ] {
            let mut parser = parser(source);
            let value = parser.baked_arguments().unwrap();
            assert_eq!(value.span.text(source), "#bake_arguments walk(flag=true)");
            assert_eq!(parser.token().kind, Kind::Punctuation(next));
        }
    }

    #[test]
    fn missing_application_and_call_context_are_explicit_errors() {
        for source in [
            "#bake_arguments walk",
            "#bake_arguments walk(flag=true,,allocator=pool)",
        ] {
            assert!(parser(source).baked_arguments().is_err());
        }
        let mut parser = parser("#bake_arguments walk(flag=true)");
        parser.allow_qualified = false;
        assert_eq!(
            parser
                .baked_arguments()
                .unwrap_err()
                .span
                .text(parser.source),
            "#bake_arguments"
        );
    }
}
