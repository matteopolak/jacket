//! Capture ordinals are shared by real source owners across resolver retries.
use super::*;

#[derive(Default)]
pub(crate) struct ExpressionBindingAllocator {
    next: HashMap<ProcedureId, usize>,
}
impl ExpressionBindingAllocator {
    fn allocate(
        &mut self,
        owner: ProcedureId,
        span: Span,
    ) -> Result<jai_ir::ExpressionBindingId, Diagnostic> {
        let next = self.next.entry(owner).or_default();
        let id = jai_ir::ExpressionBindingId::new(owner, *next);
        *next = next
            .checked_add(1)
            .ok_or_else(|| Diagnostic::new(span, "expression capture identity space exhausted"))?;
        Ok(id)
    }
}
impl Resolver<'_> {
    pub(crate) fn allocate_expression_binding(
        &mut self,
        span: Span,
    ) -> Result<jai_ir::ExpressionBindingId, Diagnostic> {
        let owner = authorized_owner(
            self.expression_owner,
            self.compile_time.map(|context| context.owner),
            span,
        )?;
        self.meta.expression_bindings.allocate(owner, span)
    }
}

fn authorized_owner(
    source: Option<ProcedureId>,
    compile_time: Option<ProcedureId>,
    span: Span,
) -> Result<ProcedureId, Diagnostic> {
    let source = source.ok_or_else(|| {
        Diagnostic::new(
            span,
            "expression capture requires an actual source procedure or compile-time owner",
        )
    })?;
    Ok(compile_time.unwrap_or(source))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn owner_ordinals_survive_interleaved_allocations() {
        let mut allocator = ExpressionBindingAllocator::default();
        let first = ProcedureId::new(7);
        let second = ProcedureId::new(8);
        for (owner, expected) in [(first, 0), (second, 0), (first, 1), (second, 1)] {
            let id = allocator.allocate(owner, Span::default()).unwrap();
            assert_eq!((id.procedure(), id.index()), (owner, expected));
        }
    }

    #[test]
    fn preview_denial_cannot_be_overridden_by_a_retained_context() {
        let source = ProcedureId::new(7);
        let context = ProcedureId::new(8);
        let span = Span::new(12, 18);
        assert!(authorized_owner(None, None, span).is_err());
        assert!(authorized_owner(None, Some(context), span).is_err());
        assert_eq!(authorized_owner(Some(source), None, span).unwrap(), source);
        assert_eq!(
            authorized_owner(Some(source), Some(context), span).unwrap(),
            context
        );
    }

    #[test]
    fn exhausted_ordinals_do_not_issue_or_reuse_an_identity() {
        let owner = ProcedureId::new(7);
        let mut allocator = ExpressionBindingAllocator::default();
        allocator.next.insert(owner, usize::MAX);
        assert!(allocator.allocate(owner, Span::default()).is_err());
        assert_eq!(allocator.next.get(&owner), Some(&usize::MAX));
        assert!(allocator.allocate(owner, Span::default()).is_err());
    }
}
