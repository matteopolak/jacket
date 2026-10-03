//! Member procedures retain source ownership until a checked namespace binds them.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct RecordMethodId {
    pub(crate) owner: TypeId,
    pub(crate) member: usize,
}
#[derive(Clone)]
pub(crate) enum RecordMethodSource {
    Procedure(syntax::Procedure),
    Prototype(syntax::ProcedurePrototype),
    Constant(syntax::ConstantDeclaration),
}
impl RecordMethodSource {
    pub(crate) fn name(&self) -> jai_source::Symbol {
        match self {
            Self::Procedure(source) => source.name,
            Self::Prototype(source) => source.name,
            Self::Constant(source) => source.name,
        }
    }
    pub(crate) fn span(&self) -> Span {
        match self {
            Self::Procedure(source) => source.span,
            Self::Prototype(source) => source.span,
            Self::Constant(source) => source.span,
        }
    }
}
#[derive(Clone)]
pub(crate) struct RecordMethod {
    pub(crate) id: RecordMethodId,
    pub(crate) file: FileInstanceId,
    pub(crate) source: RecordMethodSource,
}
