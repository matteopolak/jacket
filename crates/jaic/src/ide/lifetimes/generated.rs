//! Completion evidence for the single main call emitted by an entry wrapper.
use super::{Arc, Compiler, ProcId, ast, ir};
use crate::sema::procs::{BodyState, ProcTarget};

/// The actual hidden-context operand of the generated call.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LifetimeContextTransfer {
    ForwardHidden,
    Null,
    Absent,
}

/// A generated call edge, not a resource, default-context or result-origin fact.
#[derive(Clone, Debug)]
pub struct LifetimeGeneratedEntryCall {
    pub wrapper: ProcId,
    pub wrapper_func: ir::FuncId,
    pub wrapper_syntax: Arc<ast::ProcLit>,
    pub target: ProcId,
    pub target_func: ir::FuncId,
    pub target_syntax: Arc<ast::ProcLit>,
    pub context: LifetimeContextTransfer,
}

impl Compiler {
    pub(crate) fn lifetime_record_generated_entry(
        &mut self,
        wrapper: ProcId,
        wrapper_func: ir::FuncId,
        target: ProcId,
        target_func: ir::FuncId,
        context: LifetimeContextTransfer,
    ) {
        use crate::ide::Recorder;
        if !self.ed.facts().is_some_and(|f| f.lint) || self.ed.sketching() {
            return;
        }
        // Speculative lowering refuses live recording, so it cannot publish this edge.
        let edge = LifetimeGeneratedEntryCall {
            wrapper,
            wrapper_func,
            wrapper_syntax: self.proc(wrapper).lit.clone(),
            target,
            target_func,
            target_syntax: self.proc(target).lit.clone(),
            context,
        };
        self.ed
            .facts_mut()
            .unwrap()
            .lifetime_generated_entries
            .insert(wrapper, edge);
    }

    /// Return a generated edge only while its checked identities still match.
    /// For example, releasing executable IR preserves a valid edge; replacing its target does not.
    pub fn lifetime_generated_entry(&self, id: ProcId) -> Option<&LifetimeGeneratedEntryCall> {
        let facts = self.ed.facts()?;
        let edge = facts.lifetime_generated_entries.get(&id)?;
        let wrapper = self.pr.procs.get(id.0 as usize)?;
        let target = self.pr.procs.get(edge.target.0 as usize)?;
        let body = facts.lifetime_bodies.get(&edge.target)?;
        let context = match (
            wrapper.sig.as_ref()?.has_context,
            target.sig.as_ref()?.has_context,
        ) {
            (true, true) => LifetimeContextTransfer::ForwardHidden,
            (false, true) => LifetimeContextTransfer::Null,
            (_, false) => LifetimeContextTransfer::Absent,
        };
        (edge.wrapper == id
            && matches!(wrapper.target, Some(ProcTarget::Func(f)) if f == edge.wrapper_func)
            && matches!(target.target, Some(ProcTarget::Func(f)) if f == edge.target_func)
            && Arc::ptr_eq(&wrapper.lit, &edge.wrapper_syntax)
            && Arc::ptr_eq(&target.lit, &edge.target_syntax)
            && context == edge.context
            && target.body_state == BodyState::Done
            && body.procedure == edge.target
            && Arc::ptr_eq(&body.syntax, &target.lit))
        .then_some(edge)
    }
}
