//! Directional #as conversions project declared fields without changing identity.
use super::*;
use jai_types::{FieldId, RecordKind, TypeKind};
use std::collections::HashSet;

const MAX_CONVERSION_DEPTH: usize = 128;
const MAX_CONVERSION_STEPS: usize = 4096;

impl Resolver<'_> {
    pub(crate) fn has_implicit_field_conversion(
        &self,
        source: TypeId,
        target: TypeId,
        span: Span,
    ) -> Result<bool, Diagnostic> {
        let (source, target) = match (self.types.kind(source), self.types.kind(target)) {
            (Ok(TypeKind::Record(_)), _) => (source, target),
            (Ok(TypeKind::Pointer(source)), Ok(TypeKind::Pointer(target))) => (*source, *target),
            _ => return Ok(false),
        };
        Ok(self.conversion_path(source, Some(target), span)?.is_some())
    }

    fn conversion_fields(
        &self,
        ty: TypeId,
        span: Span,
    ) -> Result<Vec<(FieldId, TypeId)>, Diagnostic> {
        if !matches!(self.types.kind(ty), Ok(TypeKind::Record(_))) {
            return Ok(Vec::new());
        }
        let Ok(record) = self.record_metadata(ty, span) else {
            // Compiler-owned reflection schemas retain their separate canonical adapter.
            return Ok(Vec::new());
        };
        let mut fields = Vec::new();
        for field in record.fields {
            if field.syntax.conversion() != syntax::FieldConversion::Implicit {
                continue;
            }
            if record.kind == RecordKind::Union {
                return Err(Diagnostic::new(
                    span,
                    "union #as fields require active-alternative conversion semantics",
                ));
            }
            let actual = self
                .types
                .validate_field(ty, field.id)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?;
            if actual != field.ty {
                return Err(Diagnostic::new(
                    span,
                    "implicit conversion field metadata has a different canonical type",
                ));
            }
            fields.push((field.id, field.ty));
        }
        Ok(fields)
    }

    pub(crate) fn conversion_path(
        &self,
        source: TypeId,
        target: Option<TypeId>,
        span: Span,
    ) -> Result<Option<Vec<FieldId>>, Diagnostic> {
        find_conversion_path(source, target, span, |ty| self.conversion_fields(ty, span))
    }

    pub(crate) fn validate_field_conversion_record(
        &self,
        ty: TypeId,
        span: Span,
    ) -> Result<(), Diagnostic> {
        self.conversion_path(ty, None, span).map(|_| ())
    }

    pub(crate) fn implicit_field_value(
        &self,
        expression: Expr,
        target: TypeId,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        let Expr::Typed {
            ty: source, ..
        } = &expression
        else {
            return Ok(expression);
        };
        if *source == target || !matches!(self.types.kind(*source), Ok(TypeKind::Record(_))) {
            return Ok(expression);
        }
        let Some(path) = self.conversion_path(*source, Some(target), span)? else {
            return Ok(expression);
        };
        let mut value = expression.value(span)?;
        for field in path {
            let ty = self
                .types
                .field_type(field)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?;
            value = ValueExpr::Field {
                base: Box::new(value),
                field,
                ty,
            };
        }
        self.typed_value(value, target, span)
    }

    pub(crate) fn implicit_field_pointer(
        &mut self,
        expression: Expr,
        target: TypeId,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        let Expr::Pointer {
            ty: source, ..
        } = &expression
        else {
            return Ok(expression);
        };
        if *source == target {
            return Ok(expression);
        }
        let (Ok(TypeKind::Pointer(source_pointee)), Ok(TypeKind::Pointer(target_pointee))) =
            (self.types.kind(*source), self.types.kind(target))
        else {
            return Ok(expression);
        };
        let Some(path) = self.conversion_path(*source_pointee, Some(*target_pointee), span)? else {
            return Ok(expression);
        };
        let mut place = self
            .places
            .dereference(expression.value(span)?, self.types)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        for field in path {
            place = self
                .places
                .field(place, field, self.types)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        }
        Ok(Expr::Pointer {
            ty: target,
            value: ValueExpr::AddressOf {
                place,
                ty: target,
            },
        })
    }
}

pub(crate) fn find_conversion_path(
    source: TypeId,
    target: Option<TypeId>,
    span: Span,
    mut fields: impl FnMut(TypeId) -> Result<Vec<(FieldId, TypeId)>, Diagnostic>,
) -> Result<Option<Vec<FieldId>>, Diagnostic> {
    let mut pending = vec![(source, Vec::new(), HashSet::new())];
    let mut found = None;
    let mut steps = 0;
    while let Some((ty, path, mut ancestors)) = pending.pop() {
        steps += 1;
        if steps > MAX_CONVERSION_STEPS || path.len() > MAX_CONVERSION_DEPTH {
            return Err(Diagnostic::new(
                span,
                "implicit field conversion exceeds compiler traversal budget",
            ));
        }
        if !ancestors.insert(ty) {
            return Err(Diagnostic::new(span, "cyclic implicit field conversions"));
        }
        if target == Some(ty) && !path.is_empty() {
            if found.replace(path).is_some() {
                return Err(Diagnostic::new(
                    span,
                    "ambiguous implicit field conversion: multiple declared #as paths reach the target type",
                ));
            }
            continue;
        }
        for (field, field_ty) in fields(ty)?.into_iter().rev() {
            if steps + pending.len() >= MAX_CONVERSION_STEPS {
                return Err(Diagnostic::new(
                    span,
                    "implicit field conversion exceeds compiler traversal budget",
                ));
            }
            let mut next = path.clone();
            next.push(field);
            pending.push((field_ty, next, ancestors.clone()));
        }
    }
    Ok(found)
}

pub(crate) fn project_constant(
    mut value: jai_ir::ConstantValue,
    path: &[FieldId],
    types: &dyn jai_types::TypeView,
    span: Span,
) -> Result<jai_ir::ConstantValue, Diagnostic> {
    for &field in path {
        let ty = types
            .validate_field(value.ty, field)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        value = match value.kind {
            jai_ir::ConstantKind::Record(mut fields) => {
                let index = field.index();
                if index >= fields.len() {
                    return Err(Diagnostic::new(
                        span,
                        "record conversion constant is missing its declared field",
                    ));
                }
                let projected = fields.swap_remove(index);
                if projected.ty != ty {
                    return Err(Diagnostic::new(
                        span,
                        "record conversion constant field has a different canonical type",
                    ));
                }
                projected
            }
            jai_ir::ConstantKind::Zero => jai_ir::ConstantValue {
                ty,
                kind: jai_ir::ConstantKind::Zero,
            },
            _ => {
                return Err(Diagnostic::new(
                    span,
                    "implicit field conversion requires a record constant",
                ));
            }
        };
    }
    Ok(value)
}
