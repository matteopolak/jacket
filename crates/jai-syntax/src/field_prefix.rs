//! Preserve source field qualifiers without conflating conversion with `using`.
use jai_lexer::{Directive, Keyword, Kind, Token};
use jai_source::{Diagnostic, Span};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FieldConversion {
    #[default]
    None,
    /// `#as` permits directional conversion through this embedded field.
    Implicit,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FieldPrefix {
    pub using: bool,
    pub conversion: FieldConversion,
    pub conversion_span: Option<Span>,
}

/// Consume only field prefix qualifiers. The canonical parser still owns names,
/// types, initializers and the resulting field declaration.
pub fn field_prefix(tokens: &[Token], at: &mut usize) -> Result<FieldPrefix, Diagnostic> {
    let mut prefix = FieldPrefix::default();
    while let Some(token) = tokens.get(*at) {
        match token.kind {
            Kind::Keyword(Keyword::Using) => {
                if prefix.using {
                    return Err(Diagnostic::new(
                        token.span,
                        "duplicate using field qualifier",
                    ));
                }
                prefix.using = true;
            }
            Kind::Directive(Directive::As) => {
                if prefix.conversion == FieldConversion::Implicit {
                    return Err(Diagnostic::new(token.span, "duplicate #as field qualifier"));
                }
                prefix.conversion = FieldConversion::Implicit;
                prefix.conversion_span = Some(token.span);
            }
            _ => break,
        }
        *at += 1;
    }
    Ok(prefix)
}
