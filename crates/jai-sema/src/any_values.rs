//! Universal conversion borrows actual storage and retains canonical type descriptors.
use super::*;
#[path = "any_values/typed_literals.rs"]
mod typed_literals;
use jai_types::{AnyConversion, AnyField, AnySchema, TypeKind};

impl Resolver<'_> {
    pub(crate) fn any_schema(&mut self, ty: TypeId, span: Span) -> Result<AnySchema, Diagnostic> {
        let header = self.schema_header_type(span)?;
        AnySchema::validate(self.types, ty, header)
            .map_err(|error| Diagnostic::new(span, error.to_string()))
    }

    /// Called before ordinary contextual literal handling: an array converted to
    /// Any remains an array, rather than taking Any as its element context.
    pub(crate) fn any_expected(
        &mut self,
        source: &syntax::Expression,
        ty: TypeId,
    ) -> Result<Expr, Diagnostic> {
        if let syntax::ExpressionKind::Conditional(conditional) = &source.kind {
            return self.value_conditional(conditional, ty, source.span);
        }
        if let syntax::ExpressionKind::StructLiteral(literal) = &source.kind {
            let descriptor_literal = match &literal.ty {
                None => true,
                Some(path) => self.lexical_annotation(path, source.span)? == ty,
            };
            if descriptor_literal {
                return self.any_literal(literal, ty, source.span);
            }
        }
        let value = self.expr(source)?;
        self.box_any_expression(value, ty, source.span)
    }

    pub(crate) fn box_any_expression(
        &mut self,
        value: Expr,
        ty: TypeId,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        let schema = self.any_schema(ty, span)?;
        let represented = self.expression_type(&value, span)?;
        if represented == schema.ty() {
            let value = self.coerce_value(value, ty, span)?;
            return self.typed_value(value, ty, span);
        }
        let policy = self.target_layout.ok_or_else(|| {
            Diagnostic::new(
                span,
                "Any boxing is waiting for the compilation target layout",
            )
        })?;
        let AnyConversion::Borrow {
            ..
        } = schema
            .conversion(self.types, represented, policy)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?
        else {
            unreachable!("identity conversion was handled before materialization")
        };
        let value = self.runtime_type_expression(value, span)?;
        let value = self.coerce_value(value, represented, span)?;
        let pointer_type = self
            .types
            .pointer(represented)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        let address = match self.boxed_value_place(&value, span)? {
            Some(place) => ValueExpr::AddressOf {
                place,
                ty: pointer_type,
            },
            None => ValueExpr::AddressOfValue {
                value: Box::new(value),
                ty: pointer_type,
            },
        };
        let descriptor = self
            .type_info_header_expression(represented, span)?
            .value(span)?;
        self.typed_value(
            ValueExpr::Record {
                ty,
                fields: vec![
                    descriptor,
                    ValueExpr::PointerCast {
                        value: Box::new(address),
                        ty: schema.field(AnyField::ValuePointer).ty,
                        mode: CastMode::Unchecked,
                    },
                ],
            },
            ty,
            span,
        )
    }

    /// Recover places from already resolved IR. Re-resolving source would run
    /// compile-time expressions twice and duplicate index/address side effects.
    pub(crate) fn boxed_value_place(
        &mut self,
        value: &ValueExpr,
        span: Span,
    ) -> Result<Option<Place>, Diagnostic> {
        let mut projections = Vec::new();
        let mut current = value;
        let base = loop {
            match current {
                ValueExpr::Load(place) => break Some(*place),
                ValueExpr::Int(integer) => match integer.kind() {
                    IntExprKind::Load(place) => break Some(place.place()),
                    IntExprKind::Value(value) => current = value,
                    _ => break None,
                },
                ValueExpr::Bool(BoolExpr::Load(place)) => break Some(place.place()),
                ValueExpr::Bool(BoolExpr::Value(value)) => current = value,
                ValueExpr::Float(float) => match float.kind() {
                    FloatExprKind::Load(place) => break Some(*place),
                    FloatExprKind::Value(value) => current = value,
                    _ => break None,
                },
                ValueExpr::Field {
                    base,
                    field,
                    ..
                } => {
                    projections.push(BoxedProjection::Field(*field));
                    current = base;
                }
                ValueExpr::SequenceField {
                    base,
                    field,
                    ..
                } if matches!(
                    self.types.kind(base.type_id(self.types)),
                    Ok(TypeKind::String | TypeKind::Slice(_) | TypeKind::DynamicArray(_))
                ) =>
                {
                    projections.push(BoxedProjection::Sequence(*field));
                    current = base;
                }
                ValueExpr::Index {
                    base,
                    index,
                    check,
                    ..
                } => {
                    let base_type = base.type_id(self.types);
                    if matches!(self.types.kind(base_type), Ok(TypeKind::Pointer(_))) {
                        let pointer = ValueExpr::PointerOffset {
                            pointer: base.clone(),
                            offset: index.clone(),
                            subtract: false,
                            ty: base_type,
                        };
                        break Some(
                            self.places
                                .dereference(pointer, self.types)
                                .map_err(|error| Diagnostic::new(span, error.to_string()))?,
                        );
                    }
                    if !matches!(
                        self.types.kind(base_type),
                        Ok(TypeKind::FixedArray { .. }
                            | TypeKind::Slice(_)
                            | TypeKind::DynamicArray(_))
                    ) {
                        break None;
                    }
                    projections.push(BoxedProjection::Index(index.clone(), *check));
                    current = base;
                }
                _ => break None,
            }
        };
        let Some(mut place) = base else {
            return Ok(None);
        };
        for projection in projections.into_iter().rev() {
            place = match projection {
                BoxedProjection::Field(field) => self.places.field(place, field, self.types),
                BoxedProjection::Index(index, check) => self
                    .places
                    .index_with_check(place, index, check, self.types),
                BoxedProjection::Sequence(field) => {
                    self.places.sequence_field(place, field, self.types)
                }
            }
            .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        }
        Ok(Some(place))
    }

    pub(crate) fn any_field(
        &mut self,
        ty: TypeId,
        member: Symbol,
        span: Span,
    ) -> Result<Option<jai_types::FieldId>, Diagnostic> {
        if !matches!(self.types.kind(ty), Ok(TypeKind::Any(_))) {
            return Ok(None);
        }
        let field = match self.symbols.name(member) {
            "type" => AnyField::Type,
            "value_pointer" => AnyField::ValuePointer,
            _ => return Err(Diagnostic::new(span, "unknown Any descriptor member")),
        };
        Ok(Some(self.any_schema(ty, span)?.field(field).id))
    }
}

enum BoxedProjection {
    Field(jai_types::FieldId),
    Index(IntExpr, jai_ir::CheckMode),
    Sequence(jai_ir::SequenceField),
}
