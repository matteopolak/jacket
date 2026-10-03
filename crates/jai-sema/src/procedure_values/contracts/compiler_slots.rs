//! A staged boundary for native compiler slots without source-contract transport.
use super::*;
use jai_types::TypeView;

impl Resolver<'_> {
    pub(crate) fn compiler_slot_contains_callback(
        &self,
        root: TypeId,
        span: Span,
    ) -> Result<bool, Diagnostic> {
        let mut pending = vec![root];
        let mut visited = std::collections::HashSet::new();
        let mut scheduled = 1usize;
        while let Some(ty) = pending.pop() {
            if !visited.insert(ty) {
                continue;
            }
            match self
                .types
                .kind(ty)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?
            {
                jai_types::TypeKind::Procedure(_) | jai_types::TypeKind::Any(_) => return Ok(true),
                jai_types::TypeKind::Record(_) => {
                    let record = self
                        .types
                        .record_definition(ty)
                        .map_err(|error| Diagnostic::new(span, error.to_string()))?;
                    scheduled = compiler_slot_budget(scheduled, record.fields.len(), span)?;
                    pending.extend(record.fields.iter().copied());
                }
                jai_types::TypeKind::Distinct(id) => {
                    let representation = self
                        .types
                        .distinct(*id)
                        .map_err(|error| Diagnostic::new(span, error.to_string()))?
                        .representation;
                    scheduled = compiler_slot_budget(scheduled, 1, span)?;
                    pending.push(representation);
                }
                jai_types::TypeKind::Pointer(element)
                | jai_types::TypeKind::Slice(element)
                | jai_types::TypeKind::DynamicArray(element)
                | jai_types::TypeKind::FixedArray { element, .. } => {
                    scheduled = compiler_slot_budget(scheduled, 1, span)?;
                    pending.push(*element);
                }
                _ => {}
            }
        }
        Ok(false)
    }
}

fn compiler_slot_budget(previous: usize, next: usize, span: Span) -> Result<usize, Diagnostic> {
    previous
        .checked_add(next)
        .filter(|count| *count <= crate::constant_limits::MAX_CONSTANT_CELLS)
        .ok_or_else(|| {
            Diagnostic::new(
                span,
                "compiler local slot type exceeds source contract budget",
            )
        })
}
