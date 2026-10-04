//! Pure overload applicability and baking use the same canonical path plan as lowering.
use super::*;
use crate::modules::aggregates::promoted_literals::{
    self, FieldDefaults, PathStep, PreparedLiteral,
};
use jai_ir::ConstantValue;
use jai_types::FieldId;

fn prepare(
    types: &dyn TypeView,
    nominals: &dyn NominalView,
    root: TypeId,
    fields: &[RecordArgumentField],
    span: Span,
) -> Result<(PreparedLiteral, Vec<TypeId>), Diagnostic> {
    crate::record_placements::require_record_storage_ready(types, root, span)?;
    if fields.len() > crate::constant_limits::MAX_CONSTANT_CELLS {
        return Err(Diagnostic::new(
            span,
            "literal paths exceed compiler cell budget",
        ));
    }
    let mut leaves = Vec::with_capacity(fields.len());
    let mut targets = Vec::with_capacity(fields.len());
    let mut cells = 0usize;
    for field in fields {
        let mut owner = root;
        let mut path = Vec::new();
        for step in &field.target.steps {
            if path.len() >= crate::constant_limits::MAX_CONSTANT_DEPTH {
                return Err(Diagnostic::new(
                    field.span,
                    "literal path exceeds compiler depth",
                ));
            }
            match step {
                RecordArgumentStep::Field(name) => {
                    let nested = nominals.record_field_path(owner, *name, field.span)?;
                    for id in nested {
                        owner = types
                            .validate_field(owner, id)
                            .map_err(|e| Diagnostic::new(field.span, e.to_string()))?;
                        path.push(PathStep::Field(id));
                    }
                }
                RecordArgumentStep::Index {
                    value,
                    span,
                } => {
                    let integer = match &value.constant {
                        Some(ConstantArgument::IntegerLiteral(value)) => *value,
                        Some(ConstantArgument::Value(value)) if value.as_integer().is_some() => {
                            value.as_integer().unwrap().value()
                        }
                        _ => {
                            return Err(Diagnostic::new(
                                *span,
                                "literal index requires an already checked integer constant",
                            ));
                        }
                    };
                    let index = u64::try_from(integer)
                        .map_err(|_| Diagnostic::new(*span, "literal index must be nonnegative"))?;
                    let TypeKind::FixedArray {
                        element,
                        count,
                    } = *types
                        .kind(owner)
                        .map_err(|e| Diagnostic::new(*span, e.to_string()))?
                    else {
                        return Err(Diagnostic::new(
                            *span,
                            "literal index requires a fixed array",
                        ));
                    };
                    if index >= count {
                        return Err(Diagnostic::new(
                            *span,
                            "literal index exceeds fixed array count",
                        ));
                    }
                    path.push(PathStep::Element {
                        owner,
                        index,
                    });
                    owner = element;
                }
            }
            if path.len() >= crate::constant_limits::MAX_CONSTANT_DEPTH {
                return Err(Diagnostic::new(
                    field.span,
                    "literal path exceeds compiler depth",
                ));
            }
        }
        cells = cells
            .checked_add(path.len())
            .filter(|cells| *cells <= crate::constant_limits::MAX_CONSTANT_CELLS)
            .ok_or_else(|| {
                Diagnostic::new(field.span, "literal paths exceed compiler cell budget")
            })?;
        targets.push(owner);
        leaves.push((path, owner));
    }
    let plan = PreparedLiteral::with_paths(types, root, leaves).map_err(|e| {
        promoted_literals::path_diagnostic_at(
            e,
            &fields.iter().map(|f| f.span).collect::<Vec<_>>(),
            span,
        )
    })?;
    plan.validate_defaults(
        types,
        &mut Defaults {
            nominals,
            span,
        },
    )
    .map_err(|e| promoted_literals::build_diagnostic(e, span).unwrap_or_else(|e| e))?;
    Ok((plan, targets))
}

pub(super) fn compatible(
    types: &dyn TypeView,
    nominals: &dyn NominalView,
    root: TypeId,
    fields: &[RecordArgumentField],
    explicit: bool,
    span: Span,
) -> Result<ConversionRank, Diagnostic> {
    let (_, targets) = prepare(types, nominals, root, fields, span)?;
    let mut rank = if explicit {
        ConversionRank::Exact
    } else {
        ConversionRank::Literal
    };
    for (field, target) in fields.iter().zip(targets) {
        rank = rank.max(super::compatible(
            types,
            nominals,
            &TypePattern::Concrete(target),
            &field.value.ty,
            &Substitution::default(),
            true,
            field.span,
        )?);
    }
    Ok(rank)
}

pub(super) fn bake(
    types: &dyn TypeView,
    nominals: &dyn NominalView,
    root: TypeId,
    fields: &[RecordArgumentField],
    span: Span,
) -> Result<ConstantValue, Diagnostic> {
    let (plan, targets) = prepare(types, nominals, root, fields, span)?;
    let values = fields
        .iter()
        .zip(targets)
        .map(|(field, target)| {
            super::bake_with_nominals(
                types,
                nominals,
                &TypePattern::Concrete(target),
                &field.value,
                &Substitution::default(),
                field.span,
            )?
            .into_runtime(target, types)
            .map_err(|e| Diagnostic::new(field.span, e.to_string()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    plan.compose_constants(
        types,
        values,
        &mut Defaults {
            nominals,
            span,
        },
    )
    .map_err(|e| promoted_literals::build_diagnostic(e, span).unwrap_or_else(|e| e))
}

struct Defaults<'a> {
    nominals: &'a dyn NominalView,
    span: Span,
}
impl FieldDefaults for Defaults<'_> {
    type Error = Diagnostic;
    fn complete(&mut self, field: FieldId) -> Result<ConstantValue, Diagnostic> {
        self.nominals.field_default(field, self.span)
    }
    fn partial(&mut self, field: FieldId) -> Result<Option<ConstantValue>, Diagnostic> {
        self.nominals.field_construction_overlay(field, self.span)
    }
    fn array_element(&mut self, field: FieldId, ty: TypeId) -> Result<ConstantValue, Diagnostic> {
        self.nominals.literal_element_default(field, ty, self.span)
    }
}
