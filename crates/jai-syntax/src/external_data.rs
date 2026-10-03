//! External data metadata identifies an existing storage provider.
use super::*;

#[derive(Clone, Debug)]
pub enum ExternalDataSource {
    Program,
    Library(NamePath),
}

#[derive(Clone, Debug)]
pub struct ExternalDataBinding {
    pub source: ExternalDataSource,
    pub symbol: Option<String>,
    pub span: Span,
}

impl Parser<'_> {
    pub(super) fn external_data_binding(
        &mut self,
    ) -> Result<Option<ExternalDataBinding>, Diagnostic> {
        if self.token().kind != Kind::Directive(Directive::Elsewhere) {
            return Ok(None);
        }
        if !self.allow_qualified {
            return Err(self.error("external data requires checked storage binding"));
        }
        let start = self.token().span.start;
        self.at += 1;
        let source = if self.token().kind == Kind::Ident {
            ExternalDataSource::Library(self.name_path()?)
        } else {
            ExternalDataSource::Program
        };
        let symbol = if self.token().kind == Kind::String {
            if matches!(source, ExternalDataSource::Program) {
                return Err(self.error("an external symbol alias requires a library"));
            }
            let span = self.token().span;
            let bytes = literals::string(self.text(), span)?;
            let symbol = String::from_utf8(bytes)
                .map_err(|_| Diagnostic::new(span, "external symbol name must be UTF-8"))?;
            self.at += 1;
            Some(symbol)
        } else {
            None
        };
        if self.token().kind == Kind::Directive(Directive::Elsewhere) {
            return Err(self.error("duplicate external data binding"));
        }
        Ok(Some(ExternalDataBinding {
            source,
            symbol,
            span: Span::new(start, self.tokens[self.at - 1].span.end),
        }))
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
    fn program_and_library_bindings_preserve_provider_paths_symbols_and_spans() {
        let mut parser = parser("#elsewhere;");
        let binding = parser.external_data_binding().unwrap().unwrap();
        assert!(matches!(binding.source, ExternalDataSource::Program));
        assert!(binding.symbol.is_none());
        assert_eq!(binding.span.text(parser.source), "#elsewhere");
        assert!(parser.is(Punct::Semicolon));

        let text = r#"#elsewhere libc "native_\x73tate";"#;
        let mut parser = self::parser(text);
        let binding = parser.external_data_binding().unwrap().unwrap();
        assert_eq!(binding.symbol.as_deref(), Some("native_state"));
        assert_eq!(binding.span.text(text), &text[..text.len() - 1]);

        let text = "#elsewhere SDK.library \"OBJC_CLASS_$_View\";";
        let mut parser = self::parser(text);
        let binding = parser.external_data_binding().unwrap().unwrap();
        let ExternalDataSource::Library(path) = &binding.source else {
            panic!()
        };
        assert_eq!(parser.symbols.name(path.root), "SDK");
        assert_eq!(parser.symbols.name(path.members[0]), "library");
        assert_eq!(binding.symbol.as_deref(), Some("OBJC_CLASS_$_View"));
        assert_eq!(binding.span.text(text), &text[..text.len() - 1]);
        assert!(parser.is(Punct::Semicolon));
    }

    #[test]
    fn ordinary_declarations_keep_their_boundary_and_unsupported_bindings_are_located() {
        let mut parser = parser("= 42;");
        assert!(parser.external_data_binding().unwrap().is_none());
        assert_eq!(parser.at, 0);
        for (text, spelling, message) in [
            (
                "#elsewhere \"renamed\";",
                "\"renamed\"",
                "an external symbol alias requires a library",
            ),
            (
                "#elsewhere libc #elsewhere other;",
                "#elsewhere",
                "duplicate external data binding",
            ),
            (
                r#"#elsewhere libc "\xff";"#,
                r#""\xff""#,
                "external symbol name must be UTF-8",
            ),
        ] {
            let error = self::parser(text).external_data_binding().unwrap_err();
            assert_eq!(error.span.text(text), spelling);
            assert_eq!(error.message, message);
        }
        let mut parser = self::parser("#elsewhere;");
        parser.allow_qualified = false;
        let error = parser.external_data_binding().unwrap_err();
        assert_eq!(error.span.text(parser.source), "#elsewhere");
        assert_eq!(
            error.message,
            "external data requires checked storage binding"
        );
    }
}
