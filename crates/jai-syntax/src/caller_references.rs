//! Caller references mark only the root name, leaving postfix operands in place.
use super::*;

#[derive(Clone, Debug)]
pub struct CallerReferenceSyntax {
    pub name: Symbol,
    pub span: Span,
}

impl Parser<'_> {
    pub(super) fn caller_reference(&mut self) -> Result<CallerReferenceSyntax, Diagnostic> {
        let start = self.token().span.start;
        self.need(Punct::Backtick)?;
        if !self.allow_qualified {
            return Err(Diagnostic::new(
                self.tokens[self.at - 1].span,
                "caller references require checked invocation scope",
            ));
        }
        let name = self.name()?;
        Ok(CallerReferenceSyntax {
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
    fn only_the_root_name_is_consumed_before_member_index_and_call_operands() {
        for (text, next) in [
            ("`table.allocated - 1", Punct::Dot),
            ("`items[index]", Punct::OpenBracket),
            ("`function(argument)", Punct::OpenParen),
        ] {
            let mut parser = parser(text);
            let reference = parser.caller_reference().unwrap();
            assert_eq!(
                reference.span.text(text),
                &text[..text.find(['.', '[', '(']).unwrap()]
            );
            assert_eq!(parser.token().kind, Kind::Punctuation(next));
            assert_eq!(
                parser.symbols.name(reference.name),
                &text[1..reference.span.end]
            );
        }
        let text = "`time\\     _report.member";
        let mut parser = parser(text);
        let reference = parser.caller_reference().unwrap();
        assert_eq!(parser.symbols.name(reference.name), "time_report");
        assert_eq!(reference.span.text(text), "`time\\     _report");
        assert!(parser.is(Punct::Dot));
    }

    #[test]
    fn computed_roots_and_legacy_execution_do_not_erase_invocation_scope() {
        for text in ["`42", "`(value)", "`"] {
            assert!(parser(text).caller_reference().is_err());
        }
        let text = "`table";
        let mut parser = parser(text);
        parser.allow_qualified = false;
        let error = parser.caller_reference().unwrap_err();
        assert_eq!(error.span.text(text), "`");
        assert_eq!(
            error.message,
            "caller references require checked invocation scope"
        );
    }
}
