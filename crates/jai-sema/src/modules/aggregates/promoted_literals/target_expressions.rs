//! Staged dependency discovery visits index expressions, never relative field names.
use crate::{Diagnostic, Span};
use jai_syntax::{Expression, ExpressionKind, PlaceKind, PlaceSyntax};

enum Node<'a> {
    Place(&'a PlaceSyntax),
    Base(&'a Expression),
    Index(&'a Expression),
}

pub(crate) fn index_expressions(source: &PlaceSyntax) -> Result<Vec<&Expression>, Diagnostic> {
    let mut pending = vec![Node::Place(source)];
    let mut indices = Vec::new();
    let mut depth = 0usize;
    while let Some(node) = pending.pop() {
        if depth >= crate::constant_limits::MAX_CONSTANT_DEPTH && !matches!(&node, Node::Index(_)) {
            return Err(invalid(
                source.span,
                "literal dependency path exceeds source depth",
            ));
        }
        match node {
            Node::Place(place) => match &place.kind {
                PlaceKind::Name(_) => depth += 1,
                PlaceKind::Qualified(names) => {
                    depth = checked_names(depth, names.members.len(), place.span)?;
                }
                PlaceKind::Member { base, .. } => {
                    depth += 1;
                    pending.push(Node::Base(base));
                }
                PlaceKind::Index { base, index } => {
                    depth += 1;
                    pending.push(Node::Index(index));
                    pending.push(Node::Base(base));
                }
                _ => {
                    return Err(invalid(
                        place.span,
                        "literal target requires a relative field path",
                    ));
                }
            },
            Node::Base(base) => match &base.kind {
                ExpressionKind::Name(_) => depth += 1,
                ExpressionKind::QualifiedName(names) => {
                    depth = checked_names(depth, names.members.len(), base.span)?;
                }
                ExpressionKind::Member { base, .. } => {
                    depth += 1;
                    pending.push(Node::Base(base));
                }
                ExpressionKind::Index { base, index } => {
                    depth += 1;
                    pending.push(Node::Index(index));
                    pending.push(Node::Base(base));
                }
                _ => {
                    return Err(invalid(
                        base.span,
                        "literal target requires a relative field path",
                    ));
                }
            },
            Node::Index(index) => indices.push(index),
        }
    }
    Ok(indices)
}

fn checked_names(previous: usize, members: usize, span: Span) -> Result<usize, Diagnostic> {
    previous
        .checked_add(members)
        .and_then(|depth| depth.checked_add(1))
        .filter(|depth| *depth <= crate::constant_limits::MAX_CONSTANT_DEPTH)
        .ok_or_else(|| invalid(span, "literal dependency path exceeds source depth"))
}

fn invalid(span: Span, message: &str) -> Diagnostic {
    Diagnostic::new(span, message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_source::Symbols;

    #[test]
    fn source_indices_keep_order_and_field_names_are_not_dependencies() {
        let mut symbols = Symbols::default();
        let field = symbols.intern("values");
        let first = Expression {
            span: Span::new(10, 11),
            kind: ExpressionKind::Integer(1),
        };
        let second = Expression {
            span: Span::new(20, 21),
            kind: ExpressionKind::Integer(2),
        };
        let source = PlaceSyntax {
            span: Span::new(0, 22),
            kind: PlaceKind::Index {
                base: Box::new(Expression {
                    span: Span::new(0, 12),
                    kind: ExpressionKind::Index {
                        base: Box::new(Expression {
                            span: Span::new(0, 6),
                            kind: ExpressionKind::Name(field),
                        }),
                        index: Box::new(first),
                    },
                }),
                index: Box::new(second),
            },
        };
        let indices = index_expressions(&source).unwrap();
        assert_eq!(
            indices.iter().map(|index| index.span).collect::<Vec<_>>(),
            [Span::new(10, 11), Span::new(20, 21)]
        );
    }

    #[test]
    fn a_runtime_root_is_rejected_before_an_index_is_discovered() {
        let mut symbols = Symbols::default();
        let source = PlaceSyntax {
            span: Span::new(0, 8),
            kind: PlaceKind::Index {
                base: Box::new(Expression {
                    span: Span::new(0, 6),
                    kind: ExpressionKind::Call(symbols.intern("make"), vec![]),
                }),
                index: Box::new(Expression {
                    span: Span::new(7, 8),
                    kind: ExpressionKind::Integer(0),
                }),
            },
        };
        let error = index_expressions(&source).unwrap_err();
        assert_eq!(error.span, Span::new(0, 6));
    }

    #[test]
    fn oversized_relative_names_are_bounded_before_dependency_collection() {
        let mut symbols = Symbols::default();
        let name = symbols.intern("field");
        let source = PlaceSyntax {
            span: Span::new(0, 500),
            kind: PlaceKind::Qualified(jai_syntax::NamePath {
                root: name,
                members: vec![name; crate::constant_limits::MAX_CONSTANT_DEPTH],
            }),
        };
        assert!(
            index_expressions(&source)
                .unwrap_err()
                .message
                .contains("source depth")
        );
    }
}
