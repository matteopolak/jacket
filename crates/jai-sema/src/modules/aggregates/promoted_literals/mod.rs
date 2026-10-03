//! Named literals resolve to real physical paths before grouping aggregate data.
use crate::{Diagnostic, Span};
use jai_syntax as syntax;
mod paths;
mod positional;
mod source;
mod tree;
pub(crate) use source::concrete_literal;
pub(super) use tree::{BuildError, FieldDefaults, PreparedLiteral};

pub(super) fn path_diagnostic(
    error: paths::PathError,
    literal: &syntax::StructLiteral,
    span: Span,
) -> Diagnostic {
    use paths::PathError;
    let (index, message) = match error {
        PathError::Empty(index) => (
            index,
            "record literal initializer requires a field path".to_owned(),
        ),
        PathError::Budget(index) => (
            index,
            "record literal initializer exceeds compiler path budget".to_owned(),
        ),
        PathError::Canonical { index, error } => (index, error.to_string()),
        PathError::LeafType {
            index,
            expected,
            actual,
        } => (
            index,
            format!("record literal canonical field type mismatch: {expected:?} and {actual:?}"),
        ),
        PathError::Duplicate(index) => (index, "duplicate record literal field".to_owned()),
        PathError::Ancestor(index) => (
            index,
            "record literal whole field and nested field overlap".to_owned(),
        ),
        PathError::CompetingUnion(index) => (
            index,
            "record literal selects competing union alternatives".to_owned(),
        ),
    };
    Diagnostic::new(
        literal.fields.get(index).map_or(span, |field| field.span),
        message,
    )
}

pub(super) fn build_diagnostic<E>(error: BuildError<E>, span: Span) -> Result<Diagnostic, E> {
    let message = match error {
        BuildError::Default(error) => return Err(error),
        BuildError::Type(error) => error.to_string(),
        BuildError::DefaultType { expected, actual } => {
            format!("record construction canonical type mismatch: {expected:?} and {actual:?}")
        }
        BuildError::MalformedDefault => {
            "record construction has a malformed checked default".into()
        }
        BuildError::MissingUnionAlternative => {
            "union literal requires exactly one explicit alternative".into()
        }
        BuildError::Budget => "record construction exceeds compiler constant budget".into(),
    };
    Ok(Diagnostic::new(span, message))
}
