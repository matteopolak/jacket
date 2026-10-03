//! Field overlays remain distinct from record-body placement cursor directives.
use super::*;

#[derive(Clone, Debug)]
pub enum FieldPlacementSyntax {
    Overlay { target: PlaceSyntax, span: Span },
}

#[derive(Clone, Debug)]
pub struct PlacedFieldPrefix {
    pub qualifiers: FieldPrefix,
    pub placement: Option<FieldPlacementSyntax>,
}

impl Parser<'_> {
    pub(super) fn placed_field_prefix(&mut self) -> Result<PlacedFieldPrefix, Diagnostic> {
        let mut qualifiers = FieldPrefix::default();
        let mut placement = None;
        loop {
            let before = self.at;
            let next = field_prefix::field_prefix(&self.tokens, &mut self.at)?;
            if next.using {
                if qualifiers.using {
                    let span = self.tokens[before..self.at]
                        .iter()
                        .find(|token| token.kind == Kind::Keyword(Keyword::Using))
                        .unwrap()
                        .span;
                    return Err(Diagnostic::new(span, "duplicate using field qualifier"));
                }
                qualifiers.using = true;
            }
            if next.conversion == FieldConversion::Implicit {
                if qualifiers.conversion == FieldConversion::Implicit {
                    return Err(Diagnostic::new(
                        next.conversion_span.unwrap(),
                        "duplicate #as field qualifier",
                    ));
                }
                qualifiers.conversion = next.conversion;
                qualifiers.conversion_span = next.conversion_span;
            }
            let token = self.token();
            if token.kind != Kind::UnknownDirective || token.spelling(self.source) != "#overlay" {
                break;
            }
            if !self.allow_qualified {
                return Err(self.error("field overlays require checked layout binding"));
            }
            if placement.is_some() {
                return Err(self.error("duplicate field overlay"));
            }
            self.at += 1;
            self.need(Punct::OpenParen)?;
            let target = PlaceSyntax::try_from(self.expression(0)?).map_err(|error| {
                Diagnostic::new(error.span, "field overlay requires a field place")
            })?;
            self.need(Punct::CloseParen)?;
            placement = Some(FieldPlacementSyntax::Overlay {
                target,
                span: Span::new(token.span.start, self.tokens[self.at - 1].span.end),
            });
        }
        Ok(PlacedFieldPrefix {
            qualifiers,
            placement,
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
    fn overlay_parentheses_and_field_qualifiers_preserve_the_next_name() {
        for text in [
            "#overlay(anchor)\n#as using alias:int;",
            "using #overlay (anchor) #as alias:int;",
            "#as using #overlay(anchor) alias:int;",
        ] {
            let mut parser = parser(text);
            let prefix = parser.placed_field_prefix().unwrap();
            assert!(prefix.qualifiers.using);
            assert_eq!(prefix.qualifiers.conversion, FieldConversion::Implicit);
            let FieldPlacementSyntax::Overlay { target, span } = prefix.placement.unwrap();
            assert_eq!(target.span.text(text), "anchor");
            assert!(span.text(text).starts_with("#overlay"));
            assert!(span.text(text).ends_with(')'));
            assert_eq!(parser.text(), "alias");
        }
    }

    #[test]
    fn malformed_overlays_and_repeated_qualifiers_do_not_become_field_names() {
        for text in [
            "#overlay anchor field:int;",
            "#overlay(42) field:int;",
            "#overlay(call()) field:int;",
            "#overlay(anchor) #overlay(other) field:int;",
            "using #overlay(anchor) using field:int;",
            "#as #overlay(anchor) #as field:int;",
        ] {
            assert!(parser(text).placed_field_prefix().is_err());
        }
    }
}
