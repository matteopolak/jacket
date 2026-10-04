//! Note syntax retains selector words independently of expression name lookup.
use super::*;

impl Parser<'_> {
    pub(super) fn source_note_name(&mut self) -> Result<Symbol, Diagnostic> {
        let canonical = self.token().spelling(self.source);
        let spelling = canonical.strip_prefix('@').unwrap_or_default();
        self.at += 1;
        if !spelling.is_empty() {
            return Ok(self.symbols.intern(spelling));
        }
        if self.token().kind != Kind::String {
            return Err(self.error("expected a note name after '@'"));
        }
        let token = self.token();
        let bytes = literals::string(token.span.text(self.source), token.span)?;
        let text = std::str::from_utf8(&bytes)
            .map_err(|_| Diagnostic::new(token.span, "quoted note name must be UTF-8"))?;
        self.at += 1;
        Ok(self.symbols.intern(text))
    }

    pub(super) fn source_note_selector(&mut self) -> Result<NoteSelectorSyntax, Diagnostic> {
        let start = self.token().span.start;
        let mut components = Vec::new();
        loop {
            if !matches!(self.token().kind, Kind::Ident | Kind::Keyword(_)) {
                return Err(self.error("expected a selector component"));
            }
            let component = self.token();
            components.push(self.symbols.intern(&component.spelling(self.source)));
            self.at += 1;
            self.need(Punct::Colon)?;
            if self.is(Punct::Comma) || self.is(Punct::CloseParen) {
                break;
            }
        }
        Ok(NoteSelectorSyntax {
            components,
            span: Span::new(start, self.tokens[self.at - 1].span.end),
        })
    }
}
