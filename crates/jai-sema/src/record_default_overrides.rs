//! Construction defaults update an owned constant through canonical field paths.
use crate::{Diagnostic, Span};
use jai_ir::{ConstantKind, ConstantValue};
use jai_types::{FieldId, RecordKind, TypeId, TypeView};

/// Resolve direct and promoted names using physical source-owned field metadata.
pub(crate) fn find_field_path(
    ty: TypeId,
    name: jai_source::Symbol,
    span: Span,
    types: &dyn TypeView,
    mut metadata: impl FnMut(TypeId) -> Result<crate::local_declarations::RecordMetadata, Diagnostic>,
) -> Result<Vec<FieldId>, Diagnostic> {
    let mut pending = vec![(ty, Vec::new(), std::collections::HashSet::new())];
    let mut found = None;
    let mut visited = 0usize;
    while let Some((ty, prefix, mut ancestors)) = pending.pop() {
        visited += 1;
        if visited > 65_536
            || pending.len() > 65_536
            || prefix.len() > crate::constant_limits::MAX_CONSTANT_DEPTH
        {
            return Err(Diagnostic::new(
                span,
                "record field projection exceeds compiler declaration budget",
            ));
        }
        if !ancestors.insert(ty) {
            return Err(Diagnostic::new(span, "cyclic using field promotion"));
        }
        let record = metadata(ty)?;
        for field in record.fields.iter().rev() {
            let canonical = types
                .validate_field(ty, field.id)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?;
            if canonical != field.ty {
                return Err(Diagnostic::new(
                    span,
                    "record field projection differs from its canonical field type",
                ));
            }
            if field.name == Some(name) {
                let mut path = prefix.clone();
                path.push(field.id);
                if found.replace(path).is_some() {
                    return Err(Diagnostic::new(span, "ambiguous promoted record member"));
                }
            }
            if field.syntax.using() {
                let mut path = prefix.clone();
                path.push(field.id);
                pending.push((field.ty, path, ancestors.clone()));
            }
        }
    }
    found.ok_or_else(|| Diagnostic::new(span, "unknown record default field"))
}

pub(crate) fn replace_constant(
    mut value: ConstantValue,
    path: &[FieldId],
    replacement: ConstantValue,
    types: &dyn TypeView,
    span: Span,
) -> Result<ConstantValue, Diagnostic> {
    if path.len() > crate::constant_limits::MAX_CONSTANT_DEPTH {
        return Err(Diagnostic::new(
            span,
            "record default path exceeds compiler depth budget",
        ));
    }
    let mut ancestors = Vec::with_capacity(path.len());
    for &field in path {
        let target = types
            .validate_field(value.ty, field)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        let record = types
            .record_definition(value.ty)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        if record.kind != RecordKind::Struct {
            return Err(Diagnostic::new(
                span,
                "record default overrides require a struct field path",
            ));
        }
        let mut fields = match value.kind {
            ConstantKind::Record(fields) => fields,
            ConstantKind::Zero => record
                .fields
                .iter()
                .map(|&ty| ConstantValue {
                    ty,
                    kind: ConstantKind::Zero,
                })
                .collect(),
            _ => {
                return Err(Diagnostic::new(
                    span,
                    "record default path requires an immutable record constant",
                ));
            }
        };
        if fields.len() != record.fields.len()
            || fields
                .iter()
                .zip(&record.fields)
                .any(|(field, ty)| field.ty != *ty)
        {
            return Err(Diagnostic::new(
                span,
                "record default constant differs from its canonical field schema",
            ));
        }
        let index = field.index();
        let child = std::mem::replace(
            &mut fields[index],
            ConstantValue {
                ty: target,
                kind: ConstantKind::Zero,
            },
        );
        ancestors.push((value.ty, fields, index));
        value = child;
    }
    if value.ty != replacement.ty {
        return Err(Diagnostic::new(
            span,
            "record default override differs from its canonical field type",
        ));
    }
    value = replacement;
    while let Some((ty, mut fields, index)) = ancestors.pop() {
        fields[index] = value;
        value = ConstantValue {
            ty,
            kind: ConstantKind::Record(fields),
        };
    }
    Ok(value)
}
