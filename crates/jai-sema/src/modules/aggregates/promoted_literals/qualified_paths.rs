//! Relative source members resolve through actual nested physical record fields.
use crate::{Diagnostic, Span, local_declarations::RecordMetadata};
use jai_source::Symbol;
use jai_syntax::{Expression, ExpressionKind, PlaceKind, PlaceSyntax};
use jai_types::{FieldId, TypeId, TypeView};

pub(super) fn resolve(
    root: TypeId,
    source: &PlaceSyntax,
    types: &dyn TypeView,
    mut metadata: impl FnMut(TypeId) -> Result<RecordMetadata, Diagnostic>,
) -> Result<Vec<FieldId>, Diagnostic> {
    let mut names = Vec::new();
    place_names(source, &mut names)?;
    let mut path = Vec::new();
    let mut ty = root;
    for name in names {
        let nested = crate::record_default_overrides::find_field_path(
            ty,
            name,
            source.span,
            types,
            &mut metadata,
        )?;
        if path.len() + nested.len() >= crate::constant_limits::MAX_CONSTANT_DEPTH {
            return Err(Diagnostic::new(
                source.span,
                "record literal initializer exceeds field depth",
            ));
        }
        for field in nested {
            ty = types
                .validate_field(ty, field)
                .map_err(|error| Diagnostic::new(source.span, error.to_string()))?;
            path.push(field);
        }
    }
    Ok(path)
}

fn place_names(source: &PlaceSyntax, names: &mut Vec<Symbol>) -> Result<(), Diagnostic> {
    match &source.kind {
        PlaceKind::Name(name) => names.push(*name),
        PlaceKind::Qualified(path) => {
            names.push(path.root);
            names.extend(&path.members);
        }
        PlaceKind::Member { base, member } => {
            expression_names(base, names)?;
            names.push(*member);
        }
        PlaceKind::Index { .. } => return Err(indexed_diagnostic(source.span)),
        _ => {
            return Err(Diagnostic::new(
                source.span,
                "record literal requires a relative field path",
            ));
        }
    }
    check_depth(names, source.span)
}

fn expression_names(mut source: &Expression, names: &mut Vec<Symbol>) -> Result<(), Diagnostic> {
    let mut suffix = Vec::new();
    loop {
        check_depth(&suffix, source.span)?;
        match &source.kind {
            ExpressionKind::Name(name) => {
                names.push(*name);
                break;
            }
            ExpressionKind::QualifiedName(path) => {
                names.push(path.root);
                names.extend(&path.members);
                break;
            }
            ExpressionKind::Member { base, member } => {
                suffix.push(*member);
                source = base;
            }
            ExpressionKind::Index { .. } => return Err(indexed_diagnostic(source.span)),
            _ => {
                return Err(Diagnostic::new(
                    source.span,
                    "record literal requires a relative field path",
                ));
            }
        }
    }
    names.extend(suffix.into_iter().rev());
    check_depth(names, source.span)
}

fn check_depth(names: &[Symbol], span: Span) -> Result<(), Diagnostic> {
    if names.len() >= crate::constant_limits::MAX_CONSTANT_DEPTH {
        return Err(Diagnostic::new(
            span,
            "record literal initializer exceeds field depth",
        ));
    }
    Ok(())
}

fn indexed_diagnostic(span: Span) -> Diagnostic {
    Diagnostic::new(
        span,
        "indexed literal initializers require checked array construction paths",
    )
}
