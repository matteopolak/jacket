//! Relative literal targets retain each source field and index operation.
use super::ArgumentInfo;
use crate::Resolver;
use jai_source::{Diagnostic, Span, Symbol};
use jai_syntax::{Expression, ExpressionKind, PlaceKind, PlaceSyntax};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordArgumentTarget {
    pub steps: Vec<RecordArgumentStep>,
}

impl RecordArgumentTarget {
    pub fn field(name: Symbol) -> Self {
        Self {
            steps: vec![RecordArgumentStep::Field(name)],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecordArgumentStep {
    Field(Symbol),
    Index {
        value: Box<ArgumentInfo>,
        span: Span,
    },
}

impl Resolver<'_> {
    pub(crate) fn describe_record_argument_target(
        &mut self,
        source: &PlaceSyntax,
    ) -> Result<RecordArgumentTarget, Diagnostic> {
        let mut steps = Vec::new();
        match &source.kind {
            PlaceKind::Name(name) => steps.push(RecordArgumentStep::Field(*name)),
            PlaceKind::Qualified(path) => {
                steps.push(RecordArgumentStep::Field(path.root));
                steps.extend(path.members.iter().copied().map(RecordArgumentStep::Field));
            }
            PlaceKind::Member { base, member } => {
                self.describe_record_target_expression(base, &mut steps, 0)?;
                steps.push(RecordArgumentStep::Field(*member));
            }
            PlaceKind::Index { base, index } => {
                self.describe_record_target_expression(base, &mut steps, 0)?;
                steps.push(RecordArgumentStep::Index {
                    value: Box::new(self.describe_argument(index)?),
                    span: index.span,
                });
            }
            PlaceKind::Insert(_) | PlaceKind::Dereference(_) => {
                return Err(relative_path_diagnostic(source.span));
            }
        }
        check_depth(steps.len(), source.span)?;
        Ok(RecordArgumentTarget { steps })
    }

    fn describe_record_target_expression(
        &mut self,
        source: &Expression,
        steps: &mut Vec<RecordArgumentStep>,
        depth: usize,
    ) -> Result<(), Diagnostic> {
        check_depth(depth, source.span)?;
        match &source.kind {
            ExpressionKind::Name(name) => steps.push(RecordArgumentStep::Field(*name)),
            ExpressionKind::QualifiedName(path) => {
                steps.push(RecordArgumentStep::Field(path.root));
                steps.extend(path.members.iter().copied().map(RecordArgumentStep::Field));
            }
            ExpressionKind::Member { base, member } => {
                self.describe_record_target_expression(base, steps, depth + 1)?;
                steps.push(RecordArgumentStep::Field(*member));
            }
            ExpressionKind::Index { base, index } => {
                self.describe_record_target_expression(base, steps, depth + 1)?;
                steps.push(RecordArgumentStep::Index {
                    value: Box::new(self.describe_argument(index)?),
                    span: index.span,
                });
            }
            _ => return Err(relative_path_diagnostic(source.span)),
        }
        check_depth(steps.len(), source.span)
    }
}

fn check_depth(depth: usize, span: Span) -> Result<(), Diagnostic> {
    if depth >= crate::constant_limits::MAX_CONSTANT_DEPTH {
        return Err(Diagnostic::new(
            span,
            "record literal initializer exceeds field depth",
        ));
    }
    Ok(())
}

fn relative_path_diagnostic(span: Span) -> Diagnostic {
    Diagnostic::new(span, "record literal requires a relative field path")
}
