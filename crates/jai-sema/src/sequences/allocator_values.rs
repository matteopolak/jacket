//! Descriptor allocator fields preserve the adopted source record identity.
use super::*;

impl Resolver<'_> {
    pub(super) fn sequence_allocator_type(&self, span: Span) -> Result<TypeId, Diagnostic> {
        let schema = self.types.allocator_schema().ok_or_else(|| {
            Diagnostic::new(
                span,
                "dynamic allocator field requires the selected Preload allocator role",
            )
        })?;
        let validated =
            jai_types::AllocatorSchema::validate(self.types, schema.ty(), schema.mode_type())
                .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        if validated != schema {
            return Err(Diagnostic::new(
                span,
                "allocator field proof belongs to another type registry",
            ));
        }
        Ok(schema.ty())
    }
}
