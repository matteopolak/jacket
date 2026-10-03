//! Shared lexical using grammar retains targets and computed filters.
use super::*;

impl Parser<'_> {
    #[cfg(test)]
    pub(super) fn using_directive(&mut self) -> Result<UsingDirective, Diagnostic> {
        let start = self.token().span.start;
        if !self.keyword(Keyword::Using) {
            return Err(self.error("expected using"));
        }
        let selection = self.using_selection()?;
        self.using_target(start, selection)
    }

    pub(super) fn using_target(
        &mut self,
        start: usize,
        selection: UsingSelection,
    ) -> Result<UsingDirective, Diagnostic> {
        let target = self.expression(0)?;
        self.finish_expression_statement(&target)?;
        Ok(UsingDirective {
            target,
            selection,
            span: Span::new(start, self.tokens[self.at - 1].span.end),
        })
    }

    /// Called after the owning declaration parser consumes `using`.
    pub(super) fn using_selection(&mut self) -> Result<UsingSelection, Diagnostic> {
        if !self.take(Punct::Comma) {
            return Ok(UsingSelection::All);
        }
        let modifier = self.token().spelling(self.source).into_owned();
        if self.token().kind != Kind::Ident {
            return Err(self.error("expected only, except, or map using modifier"));
        }
        let span = self.token().span;
        self.at += 1;
        match modifier.as_str() {
            "only" => Ok(UsingSelection::Only(self.using_names()?)),
            "except" => Ok(UsingSelection::Except(self.using_names()?)),
            "map" => {
                self.need(Punct::OpenParen)?;
                let mapper = self.expression(0)?;
                self.need(Punct::CloseParen)?;
                Ok(UsingSelection::Map(Box::new(mapper)))
            }
            _ => Err(Diagnostic::new(span, "unknown using modifier")),
        }
    }

    fn using_names(&mut self) -> Result<UsingNames, Diagnostic> {
        if !self.take(Punct::OpenParen) {
            return Ok(UsingNames::Expression(Box::new(self.expression(21)?)));
        }
        let bare_name = matches!(self.token().kind, Kind::Ident | Kind::Keyword(_))
            && matches!(
                self.tokens.get(self.at + 1).map(|token| token.kind),
                Some(Kind::Punctuation(Punct::Comma | Punct::CloseParen))
            );
        if bare_name {
            let mut names = Vec::new();
            loop {
                let token = self.token();
                if !matches!(token.kind, Kind::Ident | Kind::Keyword(_)) {
                    return Err(self.error("expected a using selector name"));
                }
                let name = self.symbols.intern(&token.spelling(self.source));
                names.push(UsingName {
                    name,
                    span: token.span,
                });
                self.at += 1;
                if self.take(Punct::CloseParen) {
                    break;
                }
                self.need(Punct::Comma)?;
            }
            Ok(UsingNames::Names(names))
        } else {
            let expression = self.expression(0)?;
            self.need(Punct::CloseParen)?;
            Ok(UsingNames::Expression(Box::new(expression)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parser(text: &str) -> Parser<'_> {
        Parser {
            source: text,
            tokens: lex(text).unwrap(),
            at: 0,
            symbols: Symbols::default(),
            allow_qualified: true,
            record_conditional_depth: 0,
            file_conditional_depth: 0,
        }
    }

    #[test]
    fn compiler_enum_and_field_using_keep_target_paths_and_selector_names() {
        let text =
            "using Operator_Type; using,except(preserve_debug_info) build_options.llvm_options;";
        let mut parser = parser(text);
        let enum_using = parser.using_directive().unwrap();
        assert!(matches!(enum_using.selection, UsingSelection::All));
        assert_eq!(enum_using.span.text(text), "using Operator_Type;");
        let directive = parser.using_directive().unwrap();
        let UsingSelection::Except(UsingNames::Names(names)) = directive.selection else {
            panic!()
        };
        assert_eq!(parser.symbols.name(names[0].name), "preserve_debug_info");
        assert_eq!(names[0].span.text(text), "preserve_debug_info");
        assert_eq!(
            directive.target.span.text(text),
            "build_options.llvm_options"
        );
        assert!(matches!(
            directive.target.kind,
            ExpressionKind::QualifiedName(_)
        ));
    }

    #[test]
    fn vk_operator_names_and_source_generated_name_lists_are_expressions() {
        let text = "using,only(.[\"+\",\"-\",\"*\"]) Basic; using,except #run skipped_names() instance; using,map(prefix_with_gl) procs;";
        let mut parser = parser(text);
        let directive = parser.using_directive().unwrap();
        assert!(
            matches!(directive.selection,UsingSelection::Only(UsingNames::Expression(expression)) if matches!(expression.kind,ExpressionKind::ArrayLiteral(_)))
        );
        let directive = parser.using_directive().unwrap();
        assert!(
            matches!(directive.selection,UsingSelection::Except(UsingNames::Expression(expression)) if matches!(expression.kind,ExpressionKind::CompileTime(_)))
        );
        let directive = parser.using_directive().unwrap();
        assert!(
            matches!(directive.selection,UsingSelection::Map(expression) if matches!(expression.kind,ExpressionKind::Name(_)))
        );
    }

    #[test]
    fn malformed_using_modifiers_do_not_become_name_promotions() {
        for text in [
            "using,unknown(field) object;",
            "using,only(x,1) object;",
            "using,map() object;",
            "using,except(field) ;",
            "using,only re\\ turn object;",
        ] {
            assert!(parser(text).using_directive().is_err(), "{text}");
        }
    }

    #[test]
    fn unparenthesized_filters_stop_at_the_target_and_keep_source_spellings() {
        let text = "using,except .[\"x\", \"y\"] orientation; using,except(time\\    _report, context) settings;";
        let mut parser = parser(text);
        let directive = parser.using_directive().unwrap();
        assert_eq!(directive.target.span.text(text), "orientation");
        assert!(matches!(
            directive.selection,
            UsingSelection::Except(UsingNames::Expression(expression))
                if matches!(expression.kind, ExpressionKind::ArrayLiteral(_))
        ));
        let directive = parser.using_directive().unwrap();
        let UsingSelection::Except(UsingNames::Names(names)) = directive.selection else {
            panic!()
        };
        assert_eq!(parser.symbols.name(names[0].name), "time_report");
        assert_eq!(names[0].span.text(parser.source), "time\\    _report");
        assert_eq!(parser.symbols.name(names[1].name), "context");
    }
}
