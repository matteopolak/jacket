//! Ordinary call arguments and call-scoped context overrides are separate groups.
use super::*;

pub(super) struct ParsedCallArguments {
    pub arguments: Vec<CallArgument>,
    pub overrides: Vec<CallArgument>,
}

impl Parser<'_> {
    pub(super) fn parsed_call_arguments(&mut self) -> Result<ParsedCallArguments, Diagnostic> {
        self.need(Punct::OpenParen)?;
        let mut arguments = Vec::new();
        let mut overrides = Vec::new();
        if self.take(Punct::CloseParen) {
            return Ok(ParsedCallArguments {
                arguments,
                overrides,
            });
        }
        if !self.take(Punct::DoubleComma) {
            loop {
                arguments.push(self.source_call_argument(true)?);
                if self.take(Punct::CloseParen) {
                    return Ok(ParsedCallArguments {
                        arguments,
                        overrides,
                    });
                }
                if self.take(Punct::DoubleComma) {
                    break;
                }
                self.need(Punct::Comma)?;
                if self.take(Punct::CloseParen) {
                    return Ok(ParsedCallArguments {
                        arguments,
                        overrides,
                    });
                }
            }
        }
        if !self.allow_qualified {
            return Err(self.error("call context overrides require context resolution"));
        }
        if self.is(Punct::CloseParen) {
            return Err(self.error("expected a context override after ',,'"));
        }
        loop {
            overrides.push(self.source_call_argument(false)?);
            if self.take(Punct::CloseParen) {
                break;
            }
            self.need(Punct::Comma)?;
            if self.take(Punct::CloseParen) {
                break;
            }
        }
        Ok(ParsedCallArguments {
            arguments,
            overrides,
        })
    }

    fn source_call_argument(&mut self, allow_spread: bool) -> Result<CallArgument, Diagnostic> {
        let name = if self.named_prefix(Punct::Assign) {
            let name = self.name()?;
            self.need(Punct::Assign)?;
            Some(name)
        } else {
            None
        };
        let spread = self.take(Punct::Range);
        if spread && !allow_spread {
            return Err(self.error("spread arguments require an ordinary call argument"));
        }
        if spread && !self.allow_qualified {
            return Err(self.error("spread arguments require variadic resolution"));
        }
        Ok(CallArgument {
            name,
            value: self.expression(0)?,
            spread,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_source::SourceMap;

    fn body(text: &str, symbols: &mut Symbols) -> Vec<Statement> {
        let mut sources = SourceMap::default();
        let id = sources.insert("call-arguments.jai".into(), text.into());
        let file = parse_file(sources.get(id).unwrap(), symbols).unwrap();
        let FileItem::Declaration(FileDeclaration {
            kind: FileDeclarationKind::Procedure(main),
            ..
        }) = &file.items()[0]
        else {
            panic!("expected procedure")
        };
        main.body.clone()
    }

    #[test]
    fn context_overrides_are_separate_from_ordinary_and_spread_arguments() {
        let mut symbols = Symbols::default();
        let statements = body(
            "main :: () { log(\"%\", ..args, location,, allocator=temp,logger=custom); Math.call(,, pool_allocator); }",
            &mut symbols,
        );
        let StatementKind::Expression(Expression {
            kind: ExpressionKind::ContextCall {
                args, overrides, ..
            },
            ..
        }) = &statements[0].kind
        else {
            panic!("expected context call")
        };
        assert_eq!(args.len(), 3);
        assert!(!args[0].spread);
        assert!(args[1].spread);
        assert!(!args[2].spread);
        assert_eq!(overrides.len(), 2);
        assert_eq!(symbols.name(overrides[0].name.unwrap()), "allocator");
        assert_eq!(symbols.name(overrides[1].name.unwrap()), "logger");
        assert!(overrides.iter().all(|argument| !argument.spread));
        let StatementKind::Expression(Expression {
            kind:
                ExpressionKind::ContextCall {
                    callee,
                    args,
                    overrides,
                },
            ..
        }) = &statements[1].kind
        else {
            panic!("expected context call")
        };
        assert!(matches!(callee.kind, ExpressionKind::QualifiedName(_)));
        assert!(args.is_empty());
        assert_eq!(overrides.len(), 1);
        assert!(overrides[0].name.is_none());
    }

    #[test]
    fn basic_print_pack_forwarding_preserves_the_descriptor_boundary() {
        let statements = body(
            "main :: () { print_to_builder(builder, format, ..args); log(..args, location); }",
            &mut Symbols::default(),
        );
        let StatementKind::Expression(Expression {
            kind: ExpressionKind::Call(_, args),
            ..
        }) = &statements[0].kind
        else {
            panic!("expected call")
        };
        assert_eq!(args.len(), 3);
        assert!(args[2].spread);
        let StatementKind::Expression(Expression {
            kind: ExpressionKind::Call(_, args),
            ..
        }) = &statements[1].kind
        else {
            panic!("expected call")
        };
        assert!(args[0].spread);
        assert!(!args[1].spread);
    }

    #[test]
    fn named_ordinary_spreads_retain_their_parameter_identity() {
        let mut symbols = Symbols::default();
        let statements = body("main::(){ f(values=..args, extra=2); }", &mut symbols);
        let StatementKind::Expression(Expression {
            kind: ExpressionKind::Call(_, arguments),
            ..
        }) = &statements[0].kind
        else {
            panic!("expected call");
        };
        assert_eq!(symbols.name(arguments[0].name.unwrap()), "values");
        assert!(arguments[0].spread);
        assert!(!arguments[1].spread);
    }

    #[test]
    fn malformed_context_and_spread_groups_have_located_errors() {
        for text in [
            "main::(){ f(1,,); }",
            "main::(){ f(1,,a=2,,b=3); }",
            "main::(){ f(1,,..args); }",
        ] {
            let mut sources = SourceMap::default();
            let id = sources.insert("bad-call.jai".into(), text.into());
            let error = parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap_err();
            assert!(!error.message.is_empty());
            assert!(error.location.span.end <= text.len());
        }
        assert!(parse("main::(){ f(..args); }").is_err());
        assert!(parse("main::(){ f(1,, allocator=temp); }").is_err());
    }
}
