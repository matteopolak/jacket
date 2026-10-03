//! Immutable expression captures have a real semantic procedure owner.
use crate::ProcedureId;

pub(crate) const MAX_EXPRESSION_BINDINGS: usize = 65_536;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ExpressionBindingId {
    procedure: ProcedureId,
    index: usize,
}
impl ExpressionBindingId {
    #[doc(hidden)]
    pub fn new(procedure: ProcedureId, index: usize) -> Self {
        Self {
            procedure,
            index,
        }
    }
    pub fn procedure(self) -> ProcedureId {
        self.procedure
    }
    pub fn index(self) -> usize {
        self.index
    }
}
