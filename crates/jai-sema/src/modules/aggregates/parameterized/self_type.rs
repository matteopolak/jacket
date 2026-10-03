//! Record self types use the actual enclosing reservation, with explicit bans.
use super::*;

#[derive(Clone, Copy, Default)]
pub(crate) enum NominalAnnotationContext {
    #[default]
    None,
    Record(TypeId),
    Forbidden,
}

impl NominalAnnotationContext {
    pub(crate) fn record(self, owner: TypeId) -> Self {
        match self {
            Self::Forbidden => Self::Forbidden,
            _ => Self::Record(owner),
        }
    }

    pub(crate) fn this_type(self, span: Span) -> Result<TypeId, Diagnostic> {
        match self {
            Self::Record(owner) => Ok(owner),
            Self::None => Err(Diagnostic::new(
                span,
                "#this type requires an enclosing record field annotation",
            )),
            Self::Forbidden => Err(Diagnostic::new(
                span,
                "#this is not allowed in procedure headers or record parameter lists",
            )),
        }
    }
}

impl<F> TypeResolver<'_, '_, F>
where
    F: FnMut(FileInstanceId, &syntax::Expression) -> Result<ScalarConstant, LocatedDiagnostic>,
{
    pub(super) fn in_record_annotation<R>(
        &mut self,
        owner: TypeId,
        operation: impl FnOnce(&mut Self) -> R,
    ) -> R {
        let context = self.nominal_context.record(owner);
        self.in_nominal_context(context, operation)
    }

    pub(super) fn without_record_annotation<R>(
        &mut self,
        operation: impl FnOnce(&mut Self) -> R,
    ) -> R {
        self.in_nominal_context(NominalAnnotationContext::Forbidden, operation)
    }

    pub(super) fn in_nominal_context<R>(
        &mut self,
        context: NominalAnnotationContext,
        operation: impl FnOnce(&mut Self) -> R,
    ) -> R {
        let previous = std::mem::replace(&mut self.nominal_context, context);
        let result = operation(self);
        self.nominal_context = previous;
        result
    }
}
