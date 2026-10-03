//! Captured source values take precedence over a quote's namespace fallback.
use super::*;

impl FileScope<'_> {
    pub(super) fn inserted_value(
        &self,
        path: &NamePath,
        span: Span,
    ) -> Result<Option<Binding>, Diagnostic> {
        let Some(value) = self
            .declarations
            .graph
            .insertion_capture_value(self.file, path.root)
        else {
            return Ok(None);
        };
        if !path.members.is_empty() {
            return Err(Diagnostic::new(
                span,
                "captured source members require lowering from their captured value",
            ));
        }
        use jai_modules::SourceCaptureValue;
        let binding = match value {
            SourceCaptureValue::Scalar(value) => Binding::Constant(value.clone()),
            SourceCaptureValue::Type(_) => Binding::Type(
                self.inserted_capture_type(path.root, span)?
                    .ok_or_else(|| {
                        Diagnostic::new(span, "captured source type has no canonical binding")
                    })?,
            ),
            SourceCaptureValue::Enumeration(value) => {
                let ty = self
                    .declarations
                    .nominals
                    .declarations
                    .get(&value.declaration)
                    .copied()
                    .ok_or_else(|| {
                        Diagnostic::new(span, "captured enumeration declaration is not ready")
                    })?;
                Binding::Enum(aggregates::EnumConstant {
                    ty,
                    value: value.value,
                })
            }
            SourceCaptureValue::String(_) => {
                return Err(Diagnostic::new(
                    span,
                    "captured source string requires string lowering",
                ));
            }
        };
        Ok(Some(binding))
    }

    pub(super) fn inserted_string(&self, path: &NamePath) -> Option<(Vec<u8>, Vec<Symbol>)> {
        match self
            .declarations
            .graph
            .insertion_capture_value(self.file, path.root)?
        {
            jai_modules::SourceCaptureValue::String(bytes) => {
                Some((bytes.to_vec(), path.members.clone()))
            }
            _ => None,
        }
    }
}
