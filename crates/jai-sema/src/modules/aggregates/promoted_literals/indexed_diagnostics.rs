//! Located path errors shared by source lowering and pure applicability.
use super::{paths::PathError, tree::BuildError};
use crate::{Diagnostic, Span};

pub(crate) fn path_diagnostic_at(error: PathError, spans: &[Span], span: Span) -> Diagnostic {
    let (index, message) = match error {
        PathError::Empty(index) => (
            index,
            "literal initializer requires a physical path".to_owned(),
        ),
        PathError::Budget(index) => (
            index,
            "literal initializer exceeds compiler path budget".to_owned(),
        ),
        PathError::Canonical { index, error } => (index, error.to_string()),
        PathError::ArrayOwner { index, .. } => (
            index,
            "literal array path belongs to another canonical owner".to_owned(),
        ),
        PathError::NotFixedArray(index) => (
            index,
            "literal index requires an actual fixed array".to_owned(),
        ),
        PathError::ElementBounds {
            index,
            element,
            count,
        } => (
            index,
            format!("literal index {element} exceeds fixed array count {count}"),
        ),
        PathError::LeafType { index, .. } => (
            index,
            "literal initializer has a different canonical target type".to_owned(),
        ),
        PathError::Duplicate(index) => (index, "duplicate literal initializer path".to_owned()),
        PathError::Ancestor(index) => (
            index,
            "literal whole value and projected initializer overlap".to_owned(),
        ),
        PathError::CompetingUnion(index) => (
            index,
            "literal selects competing union alternatives".to_owned(),
        ),
    };
    Diagnostic::new(spans.get(index).copied().unwrap_or(span), message)
}

pub(super) fn build_diagnostic<E>(error: BuildError<E>, span: Span) -> Result<Diagnostic, E> {
    let message = match error {
        BuildError::Default(error) => return Err(error),
        BuildError::Type(error) => error.to_string(),
        BuildError::DefaultType { .. } => {
            "literal construction has a different canonical default type".into()
        }
        BuildError::MalformedDefault => {
            "literal construction has a malformed checked default".into()
        }
        BuildError::MissingUnionAlternative => {
            "union literal requires exactly one explicit alternative".into()
        }
        BuildError::MissingArrayDefault => {
            "array construction requires a checked declaring-field default environment".into()
        }
        BuildError::Budget => "literal construction exceeds compiler constant budget".into(),
    };
    Ok(Diagnostic::new(span, message))
}
