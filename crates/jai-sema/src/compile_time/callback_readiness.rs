//! Exact called-body receipts for graph and local callback rechecks.
use super::{Context, Dependency, ProcedureId};
use crate::local_declarations::LocalDeclarationRegistry;
use jai_source::{Diagnostic, SourceSpan};
use std::cell::{Cell, RefCell};
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct CallbackRevision {
    generic: usize,
    local: usize,
}

pub(super) enum CallbackFailure {
    Pending(Vec<Dependency>),
    Failed(Diagnostic),
}

impl CallbackRevision {
    pub(super) fn current(context: &Context<'_>, locals: &LocalDeclarationRegistry) -> Self {
        Self {
            generic: context.generics.borrow().callback_readiness_revision(),
            local: locals.callback_readiness_revision(),
        }
    }

    /// Recheck the actual bodies queried by this execution. Waiting for every
    /// pending body would make an independent cached recipe inside a rechecked
    /// enclosing body wait for that enclosing body itself.
    fn check(
        self,
        context: &Context<'_>,
        locals: &LocalDeclarationRegistry,
        procedures: &[ProcedureId],
    ) -> Result<Self, CallbackFailure> {
        let generics = context.generics.borrow();
        let current = Self {
            generic: generics.callback_readiness_revision(),
            local: locals.callback_readiness_revision(),
        };
        for &id in procedures {
            if let Some(error) = locals.callback_body_failure(id) {
                return Err(CallbackFailure::Failed(error.clone()));
            }
            if let Some(error) = generics.callback_body_failure(id) {
                return Err(CallbackFailure::Failed(error.clone()));
            }
        }
        if self == current {
            return Ok(current);
        }
        let mut pending = procedures
            .iter()
            .copied()
            .filter(|&id| {
                generics.callback_body_ready(id) == Some(false)
                    || locals.callback_body_ready(id) == Some(false)
            })
            .collect::<Vec<_>>();
        drop(generics);
        pending.sort_by_key(|id| id.index());
        pending.dedup();
        if pending.is_empty() {
            return Ok(current);
        }
        Err(CallbackFailure::Pending(
            pending.into_iter().map(Dependency::Procedure).collect(),
        ))
    }
}

pub(super) struct CallbackProof {
    revision: Cell<CallbackRevision>,
    procedures: RefCell<HashSet<ProcedureId>>,
    maximum: usize,
}
impl CallbackProof {
    pub(super) fn new(context: &Context<'_>, locals: &LocalDeclarationRegistry) -> Self {
        Self {
            revision: Cell::new(CallbackRevision::current(context, locals)),
            procedures: RefCell::new(HashSet::new()),
            maximum: context.limits.value_cells,
        }
    }
    pub(super) fn record(&self, id: ProcedureId) -> Result<(), jai_vm::Error> {
        let mut procedures = self.procedures.borrow_mut();
        if procedures.contains(&id) {
            return Ok(());
        }
        if procedures.len() >= self.maximum {
            return Err(jai_vm::Error::Limit(jai_vm::LimitKind::ValueCells));
        }
        procedures.insert(id);
        Ok(())
    }
    pub(super) fn check(
        &self,
        context: &Context<'_>,
        locals: &LocalDeclarationRegistry,
    ) -> Result<(), CallbackFailure> {
        let procedures = self.procedures.borrow().iter().copied().collect::<Vec<_>>();
        let revision = self.revision.get().check(context, locals, &procedures)?;
        self.revision.set(revision);
        Ok(())
    }
    pub(super) fn validate(
        &self,
        context: &Context<'_>,
        locals: &LocalDeclarationRegistry,
        location: SourceSpan,
    ) -> Result<(), Diagnostic> {
        self.check(context, locals).map_err(|error| match error {
            CallbackFailure::Pending(dependencies) => {
                context.record_pending(dependencies);
                Diagnostic::at_source(
                    location,
                    "cached #run is waiting for checked callback contracts",
                )
            }
            CallbackFailure::Failed(error) => {
                context
                    .cache
                    .callback_failure
                    .borrow_mut()
                    .get_or_insert_with(|| error.clone());
                error
            }
        })
    }
}

#[derive(Clone, Copy)]
pub(super) struct CallbackCheck<'a> {
    pub(super) proof: &'a CallbackProof,
    pub(super) context: &'a Context<'a>,
    pub(super) locals: &'a LocalDeclarationRegistry,
}
impl CallbackCheck<'_> {
    pub(super) fn validate(self) -> Result<(), jai_vm::Error> {
        self.proof
            .check(self.context, self.locals)
            .map_err(|error| {
                match error {
                    CallbackFailure::Pending(dependencies) => {
                        self.context.record_pending(dependencies)
                    }
                    CallbackFailure::Failed(error) => {
                        self.context
                            .cache
                            .callback_failure
                            .borrow_mut()
                            .get_or_insert(error);
                    }
                }
                jai_vm::Error::InvalidIr("#run callback body proof is pending before publication")
            })
    }
}
