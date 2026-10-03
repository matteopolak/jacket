//! Generic restrictions inspect declared using ancestry without converting values.
use super::*;
use jai_types::TypeKind;

const MAX_RESTRICTION_DEPTH: usize = 128;
const MAX_RESTRICTION_STEPS: usize = 4096;

struct RestrictionNode {
    ty: TypeId,
    ancestors: Vec<TypeId>,
}

impl Resolver<'_> {
    pub(crate) fn restricted_nominal_ancestor(
        &self,
        actual: TypeId,
        target: TypeId,
        span: Span,
    ) -> Result<bool, Diagnostic> {
        nominal_ancestor(self.types, actual, target, span, |ty| {
            self.restriction_fields(ty, span)
        })
    }

    pub(crate) fn restricted_interface_member(
        &self,
        actual: TypeId,
        name: Symbol,
        span: Span,
    ) -> Result<Option<TypeId>, Diagnostic> {
        if !matches!(self.types.kind(actual), Ok(TypeKind::Record(_))) {
            return Ok(None);
        }
        if let Some(path) = self.reflection_field_path(actual, name, span)? {
            let mut ty = actual;
            for field in path {
                ty = self
                    .types
                    .validate_field(ty, field)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?;
            }
            return Ok(Some(ty));
        }
        let mut found = None;
        self.visit_restricted_using(actual, span, |_, fields| {
            for field in fields {
                if field.name == Some(name) && found.replace(field.ty).is_some() {
                    return Err(Diagnostic::new(
                        span,
                        "ambiguous using member in interface restriction",
                    ));
                }
            }
            Ok(())
        })?;
        Ok(found)
    }

    fn restriction_fields(
        &self,
        ty: TypeId,
        span: Span,
    ) -> Result<Vec<RestrictionField>, Diagnostic> {
        Ok(self
            .record_metadata(ty, span)?
            .fields
            .iter()
            .map(|field| RestrictionField {
                name: field.name,
                id: field.id,
                ty: field.ty,
                using: field.syntax.using(),
            })
            .collect())
    }

    fn visit_restricted_using(
        &self,
        actual: TypeId,
        span: Span,
        visit: impl FnMut(TypeId, &[RestrictionField]) -> Result<(), Diagnostic>,
    ) -> Result<(), Diagnostic> {
        visit_using(
            self.types,
            actual,
            span,
            |ty| self.restriction_fields(ty, span),
            visit,
        )
    }
}

/// Facts must refer to published fields in the same canonical type registry.
/// The using marker comes from each field's original source declaration.
pub(crate) struct RestrictionField {
    pub name: Option<Symbol>,
    pub id: jai_types::FieldId,
    pub ty: TypeId,
    pub using: bool,
}

pub(crate) fn nominal_ancestor(
    types: &dyn jai_types::TypeView,
    actual: TypeId,
    target: TypeId,
    span: Span,
    fields: impl FnMut(TypeId) -> Result<Vec<RestrictionField>, Diagnostic>,
) -> Result<bool, Diagnostic> {
    if actual == target {
        return Ok(true);
    }
    if !matches!(types.kind(actual), Ok(TypeKind::Record(_)))
        || !matches!(types.kind(target), Ok(TypeKind::Record(_)))
    {
        return Ok(false);
    }
    let mut found = false;
    visit_using(types, actual, span, fields, |ty, _| {
        if ty == target {
            if found {
                return Err(Diagnostic::new(
                    span,
                    "ambiguous using ancestry in type restriction",
                ));
            }
            found = true;
        }
        Ok(())
    })?;
    Ok(found)
}

fn visit_using(
    types: &dyn jai_types::TypeView,
    actual: TypeId,
    span: Span,
    mut fields: impl FnMut(TypeId) -> Result<Vec<RestrictionField>, Diagnostic>,
    mut visit: impl FnMut(TypeId, &[RestrictionField]) -> Result<(), Diagnostic>,
) -> Result<(), Diagnostic> {
    let mut pending = vec![RestrictionNode {
        ty: actual,
        ancestors: Vec::new(),
    }];
    let mut remaining = MAX_RESTRICTION_STEPS;
    while let Some(mut node) = pending.pop() {
        if remaining == 0 || node.ancestors.len() >= MAX_RESTRICTION_DEPTH {
            return Err(Diagnostic::new(
                span,
                "using restriction exceeds compiler traversal budget",
            ));
        }
        remaining -= 1;
        if node.ancestors.contains(&node.ty) {
            return Err(Diagnostic::new(
                span,
                "cyclic using ancestry in type restriction",
            ));
        }
        node.ancestors.push(node.ty);
        let fields = fields(node.ty)?;
        if fields.len() > remaining {
            return Err(Diagnostic::new(
                span,
                "using restriction exceeds compiler traversal budget",
            ));
        }
        remaining -= fields.len();
        for field in &fields {
            let actual = types
                .validate_field(node.ty, field.id)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?;
            if actual != field.ty {
                return Err(Diagnostic::new(
                    span,
                    "restriction field metadata has a different canonical type",
                ));
            }
        }
        visit(node.ty, &fields)?;
        for field in fields.iter().rev().filter(|field| field.using) {
            if !matches!(types.kind(field.ty), Ok(TypeKind::Record(_))) {
                return Err(Diagnostic::new(
                    span,
                    "using restriction requires a declared record field",
                ));
            }
            if pending.len() >= remaining {
                return Err(Diagnostic::new(
                    span,
                    "using restriction exceeds compiler traversal budget",
                ));
            }
            pending.push(RestrictionNode {
                ty: field.ty,
                ancestors: node.ancestors.clone(),
            });
        }
    }
    Ok(())
}
