//! Staged typed source paths; activate with the literal target parser carrier.
use super::{
    FieldDefaults, PreparedLiteral, build_diagnostic, indexed_source, path_diagnostic_at,
    tree::PreparedLeaf,
};
use crate::{
    Binding, BoolExpr, Diagnostic, Expr, FloatExprKind, IntExprKind, Resolver, ScalarConstant,
    Span, ValueExpr,
};
use jai_syntax as syntax;
use jai_types::{FieldId, Integer, IntegerType, TypeId};

impl Resolver<'_> {
    pub(crate) fn promoted_record_literal(
        &mut self,
        literal: &syntax::StructLiteral,
        ty: TypeId,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        crate::record_placements::require_record_storage_ready(self.types, ty, span)?;
        let (plan, targets) = self.prepare_promoted_literal(literal, ty, span)?;
        let producers = literal
            .fields
            .iter()
            .zip(targets)
            .map(|(source, target)| (&source.value, target, source.span))
            .collect::<Vec<_>>();
        self.lower_prepared_record_literal(plan, &producers, ty, span)
    }

    pub(super) fn lower_prepared_record_literal(
        &mut self,
        plan: PreparedLiteral,
        producers: &[(&syntax::Expression, TypeId, Span)],
        ty: TypeId,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        plan.validate_defaults(
            self.types,
            &mut SourceDefaults {
                resolver: self,
                span,
            },
        )
        .map_err(|error| match build_diagnostic(error, span) {
            Ok(error) | Err(error) => error,
        })?;
        let mut bindings = Vec::with_capacity(producers.len());
        let mut values = Vec::with_capacity(producers.len());
        for &(source, target, source_span) in producers {
            let value = self.expr_expected(source, target)?;
            let value = self.coerce_value(value, target, source_span)?;
            if concrete_literal(&value, source_span)? {
                values.push(PreparedLeaf::constant(
                    self.literal_constant(value, source_span)?,
                ));
                continue;
            }
            let binding = self.allocate_expression_binding(source_span)?;
            self.capture_expression_value_contract(binding, source, &value, source_span)?;
            bindings.push((binding, value));
            values.push(PreparedLeaf::bound(binding, target));
        }
        let body = plan
            .compose(
                self.types,
                values,
                &mut SourceDefaults {
                    resolver: self,
                    span,
                },
            )
            .map_err(|error| match build_diagnostic(error, span) {
                Ok(error) | Err(error) => error,
            })?;
        Ok(Expr::Typed {
            ty,
            value: if bindings.is_empty() {
                body
            } else {
                ValueExpr::Bind {
                    bindings,
                    body: Box::new(body),
                    ty,
                }
            },
        })
    }

    fn prepare_promoted_literal(
        &self,
        literal: &syntax::StructLiteral,
        ty: TypeId,
        span: Span,
    ) -> Result<(PreparedLiteral, Vec<TypeId>), Diagnostic> {
        if literal.fields.len() > crate::constant_limits::MAX_CONSTANT_CELLS {
            return Err(Diagnostic::new(
                span,
                "literal initializer exceeds compiler cell budget",
            ));
        }
        let mut leaves = Vec::with_capacity(literal.fields.len());
        let mut targets = Vec::with_capacity(literal.fields.len());
        let mut path_cells = 0usize;
        for source in &literal.fields {
            let path = indexed_source::resolve(
                ty,
                &source.target,
                self.types,
                |owner, name, span| self.field_path(owner, name, span),
                |index| self.ready_literal_index(index),
            )?;
            path_cells = path_cells.saturating_add(path.len());
            if path_cells > crate::constant_limits::MAX_CONSTANT_CELLS {
                return Err(Diagnostic::new(
                    source.span,
                    "literal paths exceed compiler cell budget",
                ));
            }
            let target = path.iter().try_fold(ty, |owner, step| match *step {
                super::paths::PathStep::Field(field) => self
                    .types
                    .validate_field(owner, field)
                    .map_err(|error| Diagnostic::new(source.span, error.to_string())),
                super::paths::PathStep::Element {
                    owner: array, ..
                } => {
                    let jai_types::TypeKind::FixedArray {
                        element, ..
                    } = *self
                        .types
                        .kind(array)
                        .map_err(|error| Diagnostic::new(source.span, error.to_string()))?
                    else {
                        return Err(Diagnostic::new(
                            source.span,
                            "checked literal path is not a fixed array",
                        ));
                    };
                    Ok(element)
                }
            })?;
            targets.push(target);
            leaves.push((path, target));
        }
        let plan = PreparedLiteral::with_paths(self.types, ty, leaves).map_err(|error| {
            path_diagnostic_at(
                error,
                &literal
                    .fields
                    .iter()
                    .map(|field| field.span)
                    .collect::<Vec<_>>(),
                span,
            )
        })?;
        Ok((plan, targets))
    }

    fn ready_literal_index(
        &self,
        source: &syntax::Expression,
    ) -> Result<Option<Integer>, Diagnostic> {
        let value =
            jai_eval::evaluate_paths(source, |path, span| match self.lookup_path(path, span)? {
                Binding::Constant(value) => Ok(value),
                Binding::TypedConstant(id) => {
                    let value = self.meta.constants.get(id).ok_or_else(|| {
                        Diagnostic::new(
                            span,
                            "literal index constant belongs to another semantic context",
                        )
                    })?;
                    match &value.kind {
                        jai_ir::ConstantKind::Int(value) => Ok(ScalarConstant::Int(*value)),
                        _ => Err(Diagnostic::new(
                            span,
                            "literal index requires an integer constant",
                        )),
                    }
                }
                _ => Err(Diagnostic::new(
                    span,
                    "literal index requires a ready compile-time integer",
                )),
            })?;
        match value {
            ScalarConstant::Int(value) => Ok(Some(value)),
            ScalarConstant::Literal(value) => {
                let ty = if value < 0 {
                    IntegerType::S64
                } else {
                    IntegerType::U64
                };
                Integer::checked(ty, value).map(Some).ok_or_else(|| {
                    Diagnostic::new(
                        source.span,
                        "literal index integer is outside the supported range",
                    )
                })
            }
            _ => Err(Diagnostic::new(
                source.span,
                "literal index requires an integer constant",
            )),
        }
    }
}

// This admits only actual literal storage. Arithmetic, casts, calls and loads
// keep their evaluation position even when another visitor calls them static.
pub(crate) fn concrete_literal(value: &ValueExpr, span: Span) -> Result<bool, Diagnostic> {
    let mut pending = vec![(value, 0)];
    let mut remaining = crate::constant_limits::MAX_CONSTANT_CELLS;
    while let Some((value, depth)) = pending.pop() {
        if remaining == 0 || depth >= crate::constant_limits::MAX_CONSTANT_DEPTH {
            return Err(Diagnostic::new(
                span,
                "literal constant exceeds compiler budget",
            ));
        }
        remaining -= 1;
        match value {
            ValueExpr::Int(value) if matches!(value.kind(), IntExprKind::Constant(_)) => {}
            ValueExpr::Float(value) if matches!(value.kind(), FloatExprKind::Constant(_)) => {}
            ValueExpr::Bool(BoolExpr::Constant(_))
            | ValueExpr::Enum {
                ..
            }
            | ValueExpr::ProcedureValue {
                ..
            }
            | ValueExpr::NativePointer(_)
            | ValueExpr::RuntimeType(_)
            | ValueExpr::Zero(_)
            | ValueExpr::StringBytes {
                ..
            } => {}
            ValueExpr::Record {
                fields, ..
            }
            | ValueExpr::Array {
                elements: fields, ..
            } => {
                pending.extend(fields.iter().map(|field| (field, depth + 1)));
            }
            ValueExpr::RecordBuild {
                initializers, ..
            } => {
                pending.extend(initializers.iter().map(|(_, value)| (value, depth + 1)));
            }
            ValueExpr::Union {
                value, ..
            }
            | ValueExpr::Distinct {
                value, ..
            } => {
                pending.push((value, depth + 1));
            }
            _ => return Ok(false),
        }
    }
    Ok(true)
}

struct SourceDefaults<'r, 's> {
    resolver: &'r Resolver<'s>,
    span: Span,
}
impl FieldDefaults for SourceDefaults<'_, '_> {
    type Error = Diagnostic;
    fn complete(&mut self, field: FieldId) -> Result<jai_ir::ConstantValue, Diagnostic> {
        self.resolver.field_default_value(field, self.span)
    }
    fn partial(&mut self, field: FieldId) -> Result<Option<jai_ir::ConstantValue>, Diagnostic> {
        self.resolver.field_construction_overlay(field, self.span)
    }
    fn array_element(
        &mut self,
        _field: FieldId,
        ty: TypeId,
    ) -> Result<jai_ir::ConstantValue, Diagnostic> {
        self.resolver.default_value(ty, self.span)
    }
}
