//! Descriptor slots require a direct field; projected record paths stay distinct.
use crate::Diagnostic;
use jai_source::Symbol;
use jai_syntax::{NamePath, PlaceKind, PlaceSyntax};

pub(crate) fn descriptor_target(source: &PlaceSyntax) -> Result<Symbol, Diagnostic> {
    match &source.kind {
        PlaceKind::Name(name) => Ok(*name),
        PlaceKind::Qualified(NamePath { root, members }) if members.is_empty() => Ok(*root),
        _ => Err(Diagnostic::new(
            source.span,
            "descriptor literal initializer requires a direct descriptor field",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_source::{Span, Symbols};

    #[test]
    fn qualified_descriptor_target_is_rejected_without_dropping_tail() {
        let mut symbols = Symbols::default();
        let root = symbols.intern("data");
        let member = symbols.intern("count");
        let span = Span::default();
        assert_eq!(
            descriptor_target(&PlaceSyntax {
                kind: PlaceKind::Name(root),
                span
            })
            .unwrap(),
            root
        );
        assert!(
            descriptor_target(&PlaceSyntax {
                kind: PlaceKind::Qualified(NamePath {
                    root,
                    members: vec![member]
                }),
                span,
            })
            .is_err()
        );
    }
}
