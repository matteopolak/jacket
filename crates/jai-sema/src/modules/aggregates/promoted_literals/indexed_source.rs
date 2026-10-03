//! Preserve relative source projections before any literal RHS is lowered.
use super::indexed_paths::PathStep;
use crate::{Diagnostic, Span};
use jai_source::Symbol;
use jai_syntax::{Expression, ExpressionKind, PlaceKind, PlaceSyntax};
use jai_types::{FieldId, Integer, TypeId, TypeKind, TypeView};

enum Pending<'a> {
    Place(&'a PlaceSyntax),
    Expression(&'a Expression),
    Name(Symbol, Span),
    Index(&'a Expression, Span),
}

/// Index facts must already be ready in the defining environment. The callback
/// may inspect checked constants, but must not run source effects or `#run`.
pub(super) fn resolve(
    root: TypeId,
    source: &PlaceSyntax,
    types: &dyn TypeView,
    mut field_path: impl FnMut(TypeId, Symbol, Span) -> Result<Vec<FieldId>, Diagnostic>,
    mut ready_index: impl FnMut(&Expression) -> Result<Option<Integer>, Diagnostic>,
) -> Result<Vec<PathStep>, Diagnostic> {
    let mut pending = vec![Pending::Place(source)];
    let mut path = Vec::new();
    let mut owner = root;
    let mut remaining = crate::constant_limits::MAX_CONSTANT_CELLS;
    while let Some(step) = pending.pop() {
        if remaining == 0 || path.len() >= crate::constant_limits::MAX_CONSTANT_DEPTH {
            return Err(Diagnostic::new(
                source.span,
                "literal initializer exceeds compiler path budget",
            ));
        }
        remaining -= 1;
        match step {
            Pending::Place(place) => match &place.kind {
                PlaceKind::Name(name) => pending.push(Pending::Name(*name, place.span)),
                PlaceKind::Qualified(names) => {
                    push_names(&mut pending, names.root, &names.members, place.span)?;
                }
                PlaceKind::Member { base, member } => {
                    pending.push(Pending::Name(*member, place.span));
                    pending.push(Pending::Expression(base));
                }
                PlaceKind::Index { base, index } => {
                    pending.push(Pending::Index(index, place.span));
                    pending.push(Pending::Expression(base));
                }
                _ => return Err(relative_path_error(place.span)),
            },
            Pending::Expression(expression) => match &expression.kind {
                ExpressionKind::Name(name) => {
                    pending.push(Pending::Name(*name, expression.span));
                }
                ExpressionKind::QualifiedName(names) => {
                    push_names(&mut pending, names.root, &names.members, expression.span)?;
                }
                ExpressionKind::Member { base, member } => {
                    pending.push(Pending::Name(*member, expression.span));
                    pending.push(Pending::Expression(base));
                }
                ExpressionKind::Index { base, index } => {
                    pending.push(Pending::Index(index, expression.span));
                    pending.push(Pending::Expression(base));
                }
                _ => return Err(relative_path_error(expression.span)),
            },
            Pending::Name(name, span) => {
                let fields = field_path(owner, name, span)?;
                if fields.is_empty() {
                    return Err(relative_path_error(span));
                }
                if path.len().saturating_add(fields.len())
                    >= crate::constant_limits::MAX_CONSTANT_DEPTH
                {
                    return Err(Diagnostic::new(
                        span,
                        "literal initializer exceeds compiler path budget",
                    ));
                }
                for field in fields {
                    owner = types
                        .validate_field(owner, field)
                        .map_err(|error| Diagnostic::new(span, error.to_string()))?;
                    path.push(PathStep::Field(field));
                }
            }
            Pending::Index(expression, span) => {
                let TypeKind::FixedArray { element, count } = *types
                    .kind(owner)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?
                else {
                    return Err(Diagnostic::new(
                        span,
                        "literal initializer index requires an actual fixed array",
                    ));
                };
                let value = ready_index(expression)?.ok_or_else(|| {
                    Diagnostic::new(
                        expression.span,
                        "literal initializer index requires a ready checked integer constant",
                    )
                })?;
                let index = u64::try_from(value.value()).map_err(|_| {
                    Diagnostic::new(expression.span, "literal initializer index is negative")
                })?;
                if index >= count {
                    return Err(Diagnostic::new(
                        expression.span,
                        format!(
                            "literal initializer index {index} exceeds fixed array count {count}"
                        ),
                    ));
                }
                path.push(PathStep::Element { owner, index });
                owner = element;
            }
        }
        if pending.len() >= crate::constant_limits::MAX_CONSTANT_DEPTH {
            return Err(Diagnostic::new(
                source.span,
                "literal initializer exceeds compiler path budget",
            ));
        }
    }
    if path.is_empty() {
        return Err(relative_path_error(source.span));
    }
    Ok(path)
}

fn push_names<'a>(
    pending: &mut Vec<Pending<'a>>,
    root: Symbol,
    members: &[Symbol],
    span: Span,
) -> Result<(), Diagnostic> {
    if pending
        .len()
        .saturating_add(members.len())
        .saturating_add(1)
        >= crate::constant_limits::MAX_CONSTANT_DEPTH
    {
        return Err(Diagnostic::new(
            span,
            "literal initializer exceeds compiler path budget",
        ));
    }
    pending.extend(members.iter().rev().map(|name| Pending::Name(*name, span)));
    pending.push(Pending::Name(root, span));
    Ok(())
}

fn relative_path_error(span: Span) -> Diagnostic {
    Diagnostic::new(
        span,
        "literal initializer requires a relative field and fixed-array path",
    )
}

#[cfg(test)]
#[path = "indexed_source/tests.rs"]
mod tests;
