//! Staged TypeSyntax target dispatcher; activate after original annotation preparation.
use super::*;
impl Resolver<'_> {
    pub(crate) fn positional_record_literal(
        &mut self,
        literal: &syntax::PositionalStructLiteral,
        expected: Option<TypeId>,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        let ty = match &literal.ty {
            Some(source) => self.lexical_annotation(source, span)?,
            None => expected.ok_or_else(|| {
                Diagnostic::new(
                    span,
                    "positional record literal requires an explicit or contextual type",
                )
            })?,
        };
        if expected.is_some_and(|expected| expected != ty) {
            return Err(Diagnostic::new(
                span,
                "record literal has a different nominal type",
            ));
        }
        if matches!(
            self.types.kind(ty),
            Ok(TypeKind::String | TypeKind::Slice(_) | TypeKind::DynamicArray(_))
        ) {
            return self.positional_sequence_literal(literal, ty, span);
        }
        let record = self.record_metadata(ty, span)?;
        if record.kind != jai_types::RecordKind::Struct {
            return Err(Diagnostic::new(
                span,
                "positional union literals require an explicit alternative",
            ));
        }
        if literal.values.len() > record.fields.len() {
            return Err(Diagnostic::new(
                span,
                "positional record literal has too many values",
            ));
        }
        self.collect_record_field_default_jobs()?;
        self.prepared_positional_record_literal(literal, ty, span)
    }

    pub(crate) fn record_literal(
        &mut self,
        literal: &syntax::StructLiteral,
        expected: Option<TypeId>,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        if let Some(ty) = expected.filter(|ty| {
            literal.ty.is_none()
                && matches!(
                    self.types.kind(*ty),
                    Ok(TypeKind::String | TypeKind::Slice(_) | TypeKind::DynamicArray(_))
                )
        }) {
            return self.sequence_literal(literal, ty, span);
        }
        let ty = match &literal.ty {
            Some(source) => self.lexical_annotation(source, span)?,
            None => expected.ok_or_else(|| {
                Diagnostic::new(
                    span,
                    "record literal requires an explicit or contextual type",
                )
            })?,
        };
        self.collect_record_field_default_jobs()?;
        if expected.is_some_and(|expected| expected != ty) {
            return Err(Diagnostic::new(
                span,
                "record literal has a different nominal type",
            ));
        }
        if matches!(
            self.types.kind(ty),
            Ok(TypeKind::String | TypeKind::Slice(_) | TypeKind::DynamicArray(_))
        ) {
            return self.sequence_literal(literal, ty, span);
        }
        if matches!(self.types.kind(ty), Ok(TypeKind::Any(_))) {
            return self.any_literal(literal, ty, span);
        }
        self.promoted_record_literal(literal, ty, span)
    }
}
