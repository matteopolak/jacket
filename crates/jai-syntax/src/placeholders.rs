//! Placeholder declarations reserve source names without fabricating types or values.
use super::*;

#[derive(Clone, Copy, Debug)]
pub struct PlaceholderDeclaration {
    pub name: Symbol,
    pub span: Span,
}

impl Parser<'_> {
    pub(super) fn placeholder_declaration(&mut self) -> Result<PlaceholderDeclaration, Diagnostic> {
        let token = self.token();
        if token.kind != Kind::UnknownDirective || token.spelling(self.source) != "#placeholder" {
            return Err(self.error("expected #placeholder"));
        }
        let start = token.span.start;
        if !self.allow_qualified {
            return Err(self.error("placeholders require checked declaration fulfillment"));
        }
        self.at += 1;
        let name = self.name()?;
        self.need(Punct::Semicolon)?;
        Ok(PlaceholderDeclaration {
            name,
            span: Span::new(start, self.tokens[self.at - 1].span.end),
        })
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
    fn placeholder_retains_its_source_name_and_leaves_the_following_item_unread() {
        let text = "#placeholder TRUTH;\nactual::true;";
        let mut parser = parser(text);
        let declaration = parser.placeholder_declaration().unwrap();
        assert_eq!(parser.symbols.name(declaration.name), "TRUTH");
        assert_eq!(declaration.span.text(text), "#placeholder TRUTH;");
        assert_eq!(parser.text(), "actual");
    }

    #[test]
    fn placeholder_is_a_single_name_declaration_with_a_required_terminator() {
        for text in [
            "#placeholder;",
            "#placeholder TRUTH = true;",
            "#placeholder TRUTH",
            "#placeholders TRUTH;",
        ] {
            assert!(parser(text).placeholder_declaration().is_err());
        }
    }
}
