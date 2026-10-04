//! Named literals resolve to real physical paths before grouping aggregate data.
use crate::{Diagnostic, Span};
use jai_syntax as syntax;
#[path = "indexed_paths.rs"]
mod paths;
use paths as indexed_paths;
pub(crate) use paths::PathStep;
mod descriptor_targets;
mod indexed_diagnostics;
pub(crate) mod indexed_source;
pub(crate) mod target_expressions;
pub(crate) use descriptor_targets::descriptor_target;
pub(crate) use indexed_diagnostics::path_diagnostic_at;
mod positional;
#[path = "typed_source.rs"]
mod source;
#[path = "indexed_tree.rs"]
mod tree;
pub(crate) use source::concrete_literal;
pub(crate) use tree::{BuildError, FieldDefaults, PreparedLiteral};

pub(super) fn path_diagnostic(
    error: paths::PathError,
    literal: &syntax::StructLiteral,
    span: Span,
) -> Diagnostic {
    path_diagnostic_at(
        error,
        &literal
            .fields
            .iter()
            .map(|field| field.span)
            .collect::<Vec<_>>(),
        span,
    )
}

pub(crate) fn build_diagnostic<E>(error: BuildError<E>, span: Span) -> Result<Diagnostic, E> {
    indexed_diagnostics::build_diagnostic(error, span)
}
