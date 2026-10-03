//! Leading-dot enum members take their identity from the checked context.
use super::*;

impl Resolver<'_> {
    pub(crate) fn enum_is_flags(&self, ty: TypeId) -> bool {
        if let Some(enumeration) = self.meta.record_specializations.member_enum(ty) {
            return enumeration.flags;
        }
        self.meta
            .local_declarations
            .enum_is_flags(ty)
            .unwrap_or_else(|| self.graph_scope.is_some_and(|scope| scope.enum_flags(ty)))
    }
    pub(crate) fn inferred_enum_member(
        &self,
        ty: TypeId,
        name: Symbol,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        let value = self
            .meta
            .local_declarations
            .enum_member_value(ty, name)
            .or_else(|| {
                self.meta
                    .record_specializations
                    .member_enum(ty)?
                    .values
                    .iter()
                    .find(|(member, _)| *member == name)
                    .map(|(_, value)| *value)
            })
            .or_else(|| {
                self.graph_scope
                    .and_then(|scope| scope.enum_member_value(ty, name))
            })
            .ok_or_else(|| {
                Diagnostic::new(
                    span,
                    "leading-dot member does not belong to the contextual enum",
                )
            })?;
        self.typed_value(ValueExpr::Enum { ty, value }, ty, span)
    }
}
