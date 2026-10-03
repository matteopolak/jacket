//! Bind placement cursors to earlier declared source fields before publication.
use crate::{Diagnostic, Symbol, syntax};
use std::collections::HashMap;

pub(crate) enum PlacementMember<'a> {
    Field(Option<Symbol>),
    Place(&'a syntax::PlaceSyntax),
}

/// The registry creates actual owner-bound FieldIds transactionally. This pass
/// returns only source ordinals; anonymous embeddings consume a physical field
/// ordinal but do not invent a name or expose promoted members as anchors.
pub(crate) fn placement_ordinals<'a>(
    members: impl IntoIterator<Item = PlacementMember<'a>>,
) -> Result<Box<[Option<usize>]>, Diagnostic> {
    let mut names = HashMap::new();
    let mut anchors = Vec::new();
    let mut cursor = None;
    for member in members {
        match member {
            PlacementMember::Field(name) => {
                if let Some(name) = name {
                    names.entry(name).or_insert(anchors.len());
                }
                anchors.push(cursor.take());
            }
            PlacementMember::Place(target) => {
                let syntax::PlaceKind::Name(name) = target.kind else {
                    return Err(Diagnostic::new(
                        target.span,
                        "#place currently requires a directly declared field name; qualified, indexed, and dereferenced anchors are unsupported",
                    ));
                };
                cursor = Some(*names.get(&name).ok_or_else(|| {
                    Diagnostic::new(
                        target.span,
                        "#place requires an earlier directly declared record field",
                    )
                })?);
            }
        }
    }
    // Preserve the natural-layout fast path when no directive changes a field.
    Ok(if anchors.iter().any(Option::is_some) {
        anchors.into_boxed_slice()
    } else {
        Box::new([])
    })
}

/// Preserve selected body order and field-prefix overlays in one source journal.
pub(crate) fn source_placement_ordinals(
    members: &[syntax::RecordMember],
) -> Result<Box<[Option<usize>]>, Diagnostic> {
    let mut ordered = Vec::new();
    for member in members {
        match member {
            syntax::RecordMember::Placement(placement) => {
                ordered.push(PlacementMember::Place(&placement.target))
            }
            syntax::RecordMember::Field(field) => {
                for attribute in &field.attributes {
                    if let syntax::FieldAttribute::Placement(
                        syntax::FieldPlacementSyntax::Overlay {
                            target, ..
                        },
                    ) = attribute
                    {
                        return Err(Diagnostic::new(
                            target.span,
                            "field #overlay requires a checked overlay cursor policy; source parsing preserves its distinct placement kind",
                        ));
                    }
                }
                ordered.push(PlacementMember::Field(Some(field.name)));
            }
            syntax::RecordMember::AnonymousRecord(_) => ordered.push(PlacementMember::Field(None)),
            _ => {}
        }
    }
    placement_ordinals(ordered)
}

pub(crate) fn source_reflection_policy(
    attributes: &[syntax::RecordAttribute],
) -> jai_types::RecordReflectionPolicy {
    jai_types::RecordReflectionPolicy::from_flags(attributes.iter().filter_map(|attribute| {
        match attribute {
            syntax::RecordAttribute::TypeInfoNone => {
                Some(jai_types::RecordReflectionFlag::NoTypeInfo)
            }
            syntax::RecordAttribute::Reflection(setting) => Some(setting.flag),
            _ => None,
        }
    }))
}

/// Until ordered recipes are bound, reject whole placed constructors before a
/// semantic field map can erase overlapping write order. Field-default jobs
/// and shape registration may still finish independently of construction.
pub(crate) fn require_record_construction_recipe(
    types: &dyn jai_types::TypeView,
    ty: jai_types::TypeId,
    span: jai_source::Span,
) -> Result<(), Diagnostic> {
    let kind = types
        .kind(ty)
        .map_err(|error| Diagnostic::new(span, error.to_string()))?;
    if kind.record_storage_id().is_some()
        && types
            .record_storage_definition(ty)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?
            .layout
            .field_placements
            .iter()
            .any(Option::is_some)
    {
        return Err(Diagnostic::new(
            span,
            "placed record construction requires an ordered storage initialization recipe",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_source::{Span, Symbols};

    fn name(symbol: Symbol, span: Span) -> syntax::PlaceSyntax {
        syntax::PlaceSyntax {
            kind: syntax::PlaceKind::Name(symbol),
            span,
        }
    }

    #[test]
    fn ordinal_binding_retains_anonymous_storage_and_cursor_source_order() {
        let mut symbols = Symbols::default();
        let original = symbols.intern("original");
        let overlay = symbols.intern("overlay");
        let target = name(original, Span::new(12, 20));
        let overlay_target = name(overlay, Span::new(31, 38));
        assert_eq!(
            placement_ordinals([
                PlacementMember::Field(Some(original)),
                PlacementMember::Field(None),
                PlacementMember::Place(&target),
                PlacementMember::Field(Some(overlay)),
                PlacementMember::Place(&overlay_target),
                PlacementMember::Field(None),
                PlacementMember::Field(None),
            ])
            .unwrap()
            .as_ref(),
            &[None, None, Some(0), Some(2), None]
        );
    }

    #[test]
    fn forward_and_nonfield_names_fail_at_the_actual_target_span() {
        let mut symbols = Symbols::default();
        let missing = symbols.intern("not_an_earlier_field");
        let target = name(missing, Span::new(12, 31));
        let error = placement_ordinals([
            PlacementMember::Field(None),
            PlacementMember::Place(&target),
            PlacementMember::Field(Some(missing)),
        ])
        .unwrap_err();
        assert_eq!(error.span, target.span);
        assert!(error.message.contains("earlier directly declared"));
    }

    #[test]
    fn complex_anchor_is_rejected_without_resolving_its_expression() {
        let mut symbols = Symbols::default();
        let target = syntax::PlaceSyntax {
            kind: syntax::PlaceKind::Member {
                base: Box::new(syntax::Expression {
                    kind: syntax::ExpressionKind::Name(symbols.intern("info")),
                    span: Span::new(0, 4),
                }),
                member: symbols.intern("value"),
            },
            span: Span::new(0, 10),
        };
        let error = placement_ordinals([PlacementMember::Place(&target)]).unwrap_err();
        assert_eq!(error.span, target.span);
        assert!(error.message.contains("unsupported"));
    }

    #[test]
    fn unused_cursor_and_ordinary_fields_preserve_default_layout_metadata() {
        let mut symbols = Symbols::default();
        let field = symbols.intern("field");
        let target = name(field, Span::new(10, 15));
        assert!(
            placement_ordinals([
                PlacementMember::Field(Some(field)),
                PlacementMember::Place(&target),
            ])
            .unwrap()
            .is_empty()
        );
    }

    #[test]
    fn complete_placed_construction_is_guarded_without_suppressing_shape_readiness() {
        use jai_types::{IntegerType, RecordKind, RecordLayout, ScalarType, TypeRegistry};
        let mut types = TypeRegistry::new();
        let word = types.scalar(ScalarType::Int(IntegerType::U64));
        let placed = types.reserve_record(RecordKind::Struct);
        types
            .define_record_with_placements(
                placed,
                [word, word],
                RecordLayout::default(),
                [None, Some(0)],
            )
            .unwrap();
        let ordinary = types.reserve_record(RecordKind::Struct);
        types.define_record(ordinary, [word]).unwrap();
        let span = Span::new(12, 24);
        let error = require_record_construction_recipe(&types, placed, span).unwrap_err();
        assert_eq!(error.span, span);
        assert!(error.message.contains("ordered storage initialization"));
        assert!(types.record_definition(placed).is_ok());
        require_record_construction_recipe(&types, ordinary, span).unwrap();
        require_record_construction_recipe(&types, word, span).unwrap();
    }
}
