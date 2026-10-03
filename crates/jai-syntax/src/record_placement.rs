//! A placement names the source field whose offset becomes the next layout cursor.
use super::*;

#[derive(Clone, Debug)]
pub struct RecordPlacementSyntax {
    pub target: PlaceSyntax,
    pub span: Span,
}

impl Parser<'_> {
    pub(super) fn record_placement(&mut self) -> Result<RecordPlacementSyntax, Diagnostic> {
        if self.token().kind != Kind::Directive(Directive::Place) {
            return Err(self.error("expected #place"));
        }
        let start = self.token().span.start;
        if !self.allow_qualified {
            return Err(self.error("record placement requires checked layout binding"));
        }
        self.at += 1;
        let expression = self.expression(0)?;
        let target = PlaceSyntax::try_from(expression).map_err(|error| {
            Diagnostic::new(error.span, "record placement requires a field place")
        })?;
        self.need(Punct::Semicolon)?;
        Ok(RecordPlacementSyntax {
            target,
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
    fn field_anchors_preserve_source_targets_and_leave_the_next_member_unread() {
        for (text, target) in [
            ("#place info; padding: [SIZE]u8 = ---;", "info"),
            ("#place base.member; next: int;", "base.member"),
        ] {
            let mut parser = parser(text);
            let placement = parser.record_placement().unwrap();
            assert_eq!(placement.target.span.text(text), target);
            assert_eq!(
                placement.span.text(text),
                &text[..text.find(';').unwrap() + 1]
            );
            assert_eq!(parser.token().kind, Kind::Ident);
            assert!(matches!(
                placement.target.kind,
                PlaceKind::Name(_) | PlaceKind::Qualified(_)
            ));
        }
        let mut parser = parser("#place info[0];");
        let placement = parser.record_placement().unwrap();
        assert!(matches!(placement.target.kind, PlaceKind::Index { .. }));
    }

    #[test]
    fn placement_values_initializers_and_missing_terminators_are_rejected() {
        for (text, spelling, message) in [
            (
                "#place 42;",
                "42",
                "record placement requires a field place",
            ),
            (
                "#place call();",
                "call()",
                "record placement requires a field place",
            ),
            ("#place info = 3;", "=", "expected ';', found '='"),
            (
                "#place info next: int;",
                "next",
                "expected ';', found 'next'",
            ),
        ] {
            let error = parser(text).record_placement().unwrap_err();
            assert_eq!(error.span.text(text), spelling);
            assert_eq!(error.message, message);
        }
        let mut parser = parser("#place info;");
        parser.allow_qualified = false;
        let error = parser.record_placement().unwrap_err();
        assert_eq!(error.span.text(parser.source), "#place");
        assert_eq!(
            error.message,
            "record placement requires checked layout binding"
        );
    }
}
