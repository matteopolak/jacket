//! The pure matcher validates storage readiness; selected lowering validates
//! the exact target layout and creates the represented type's descriptor.
use super::*;

pub(super) fn runtime_payload(
    types: &dyn TypeView,
    root: TypeId,
    span: Span,
) -> Result<(), Diagnostic> {
    let mut active = HashSet::new();
    let mut complete = HashSet::new();
    let mut pending = vec![(root, false)];
    while let Some((ty, leaving)) = pending.pop() {
        if leaving {
            active.remove(&ty);
            complete.insert(ty);
            continue;
        }
        if complete.contains(&ty) {
            continue;
        }
        if !active.insert(ty) {
            return Err(Diagnostic::new(
                span,
                "cyclic value type cannot be boxed into Any",
            ));
        }
        let kind = types
            .kind(ty)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        pending.push((ty, true));
        match kind {
            TypeKind::Void | TypeKind::Code => {
                return Err(Diagnostic::new(
                    span,
                    "compile-time metadata has no runtime payload for Any boxing",
                ));
            }
            TypeKind::Record(_) | TypeKind::Any(_) => {
                let record = types
                    .record_storage_definition(ty)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?;
                pending.extend(record.fields.iter().rev().map(|&field| (field, false)));
            }
            TypeKind::Distinct(_) => {
                let definition = types
                    .distinct_definition(ty)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?;
                pending.push((definition.representation, false));
            }
            TypeKind::FixedArray { element, .. } => pending.push((*element, false)),
            TypeKind::Enum(_) => {
                types
                    .enum_definition(ty)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?;
            }
            TypeKind::Procedure(_) => {
                types
                    .procedure_definition(ty)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?;
            }
            TypeKind::Pointer(element)
            | TypeKind::Slice(element)
            | TypeKind::DynamicArray(element) => {
                // Descriptor storage is sized independently of its pointed-to
                // nominal definition; type-info readiness remains a dependency.
                types
                    .kind(*element)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?;
            }
            TypeKind::Bool
            | TypeKind::Integer(_)
            | TypeKind::Float(_)
            | TypeKind::String
            | TypeKind::Type => {}
        }
    }
    Ok(())
}
