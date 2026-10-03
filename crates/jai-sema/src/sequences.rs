//! Sequence values retain their element identities and ordered initialization.
use super::*;
use jai_ir::SequenceField;
use jai_types::TypeKind;
mod positional_literals;

impl Resolver<'_> {
    pub(crate) fn string_literal(&self, bytes: &[u8], span: Span) -> Result<Expr, Diagnostic> {
        let ty = self.types.string();
        self.typed_value(
            ValueExpr::StringBytes {
                ty,
                bytes: bytes.to_vec(),
            },
            ty,
            span,
        )
    }

    pub(crate) fn array_literal(
        &mut self,
        literal: &syntax::ArrayLiteral,
        expected: Option<TypeId>,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        let contextual_element = match expected {
            Some(ty) => match self
                .types
                .kind(ty)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?
            {
                TypeKind::FixedArray { element, .. } | TypeKind::Slice(element) => Some(*element),
                _ => {
                    return Err(Diagnostic::new(
                        span,
                        "array literal requires a fixed array context",
                    ));
                }
            },
            None => None,
        };
        let explicit_element = match &literal.element_type {
            Some(annotation) => Some(
                self.graph_scope
                    .ok_or_else(|| {
                        Diagnostic::new(span, "typed array literal requires a graph type scope")
                    })?
                    .annotation(annotation, self.types, span)?,
            ),
            None => None,
        };
        if explicit_element
            .zip(contextual_element)
            .is_some_and(|(actual, expected)| actual != expected)
        {
            return Err(Diagnostic::new(
                span,
                "array literal element type differs from its context",
            ));
        }
        let mut first_resolved = None;
        let element = match explicit_element.or(contextual_element) {
            Some(element) => element,
            None => {
                let first = literal.elements.first().ok_or_else(|| {
                    Diagnostic::new(span, "empty array literal requires an element type")
                })?;
                let value = self.expr(first)?;
                let element = self.expression_type(&value, first.span)?;
                first_resolved = Some(value);
                element
            }
        };
        let count = u64::try_from(literal.elements.len())
            .map_err(|_| Diagnostic::new(span, "array literal has too many elements"))?;
        let ty = self
            .types
            .fixed_array(element, count)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        if expected.is_some_and(|expected| {
            matches!(self.types.kind(expected), Ok(TypeKind::FixedArray { .. })) && expected != ty
        }) {
            return Err(Diagnostic::new(
                span,
                "array literal length differs from its context",
            ));
        }
        let mut elements = Vec::with_capacity(literal.elements.len());
        for (index, expression) in literal.elements.iter().enumerate() {
            let value = if index == 0 {
                first_resolved.take()
            } else {
                None
            };
            let value = match value {
                Some(value) => value,
                None => self.expr_expected(expression, element)?,
            };
            elements.push(self.coerce_value(value, element, expression.span)?);
        }
        let array = ValueExpr::Array { ty, elements };
        if let Some(view) =
            expected.filter(|expected| matches!(self.types.kind(*expected), Ok(TypeKind::Slice(_))))
        {
            return self.typed_value(
                ValueExpr::ArrayView {
                    array: Box::new(array),
                    ty: view,
                },
                view,
                span,
            );
        }
        self.typed_value(array, ty, span)
    }

    /// Descriptor field identity is distinct from nominal record field identity.
    pub(crate) fn sequence_member(
        &mut self,
        base: Expr,
        member: Symbol,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        let base_ty = self.expression_type(&base, span)?;
        let kind = self
            .types
            .kind(base_ty)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?
            .clone();
        let element = match kind {
            TypeKind::String => self.types.scalar(ScalarType::Int(IntegerType::U8)),
            TypeKind::FixedArray { element, .. }
            | TypeKind::Slice(element)
            | TypeKind::DynamicArray(element) => element,
            _ => return Err(Diagnostic::new(span, "value is not a sequence")),
        };
        let name = self.symbols.name(member);
        let (field, ty) = match name {
            "count" => (
                SequenceField::Count,
                self.types.scalar(ScalarType::Int(IntegerType::S64)),
            ),
            "data" => (
                SequenceField::Data,
                self.types
                    .pointer(element)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?,
            ),
            "allocated" if matches!(kind, TypeKind::DynamicArray(_)) => (
                SequenceField::Allocated,
                self.types.scalar(ScalarType::Int(IntegerType::S64)),
            ),
            _ => return Err(Diagnostic::new(span, "unknown sequence member")),
        };
        let base = self.coerce_value(base, base_ty, span)?;
        if field == SequenceField::Data
            && matches!(kind, TypeKind::FixedArray { .. })
            && let ValueExpr::Load(array) = &base
        {
            self.reject_iteration_write(*array, span)?;
        }
        if field == SequenceField::Count
            && let TypeKind::FixedArray { count, .. } = kind
            && matches!(&base, ValueExpr::Load(place) if matches!(place.kind(), jai_ir::PlaceKind::Local(_) | jai_ir::PlaceKind::Global(_)))
        {
            let count =
                IntegerValue::checked(IntegerType::S64, i128::from(count)).ok_or_else(|| {
                    Diagnostic::new(span, "fixed array count exceeds signed descriptor range")
                })?;
            return Ok(Expr::Int(IntExpr::constant(count)));
        }
        self.typed_value(
            ValueExpr::SequenceField {
                base: Box::new(base),
                field,
                ty,
            },
            ty,
            span,
        )
    }

    pub(crate) fn sequence_coercion(
        &self,
        value: Expr,
        ty: TypeId,
        span: Span,
    ) -> Result<ValueExpr, Diagnostic> {
        let Expr::Typed { ty: actual, value } = value else {
            return Err(Diagnostic::new(
                span,
                "sequence value requires a compatible sequence type",
            ));
        };
        if actual == ty {
            return Ok(value);
        }
        if let (Ok(TypeKind::FixedArray { element, .. }), Ok(TypeKind::Slice(expected))) =
            (self.types.kind(actual), self.types.kind(ty))
            && element == expected
        {
            return Ok(match value {
                ValueExpr::Load(array) => {
                    self.reject_iteration_write(array, span)?;
                    ValueExpr::ArrayToSlice { array, ty }
                }
                array => ValueExpr::ArrayView {
                    array: Box::new(array),
                    ty,
                },
            });
        }
        if let Ok(TypeKind::Slice(expected)) = self.types.kind(ty) {
            let element = match self.types.kind(actual) {
                Ok(TypeKind::Slice(element) | TypeKind::DynamicArray(element)) => Some(*element),
                Ok(TypeKind::String) => Some(self.types.scalar(ScalarType::Int(IntegerType::U8))),
                _ => None,
            };
            if element == Some(*expected) {
                return Ok(ValueExpr::SequenceView {
                    sequence: Box::new(value),
                    ty,
                });
            }
        }
        Err(Diagnostic::new(
            span,
            "sequence value has a different element type or length",
        ))
    }
    pub(crate) fn sequence_literal(
        &mut self,
        literal: &syntax::StructLiteral,
        ty: TypeId,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        let mut initializers = Vec::new();
        let mut fields = std::collections::HashSet::new();
        for initializer in &literal.fields {
            let (field, field_ty) =
                self.sequence_field_type(ty, initializer.name, initializer.span)?;
            if !fields.insert(field) {
                return Err(Diagnostic::new(
                    initializer.span,
                    "duplicate sequence descriptor field",
                ));
            }
            let value = self.expr_expected(&initializer.value, field_ty)?;
            initializers.push((field, self.coerce_value(value, field_ty, initializer.span)?));
        }
        self.typed_value(ValueExpr::SequenceBuild { ty, initializers }, ty, span)
    }

    fn sequence_field_type(
        &mut self,
        ty: TypeId,
        member: Symbol,
        span: Span,
    ) -> Result<(SequenceField, TypeId), Diagnostic> {
        let kind = self
            .types
            .kind(ty)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?
            .clone();
        let element = match kind {
            TypeKind::String => self.types.scalar(ScalarType::Int(IntegerType::U8)),
            TypeKind::Slice(element) | TypeKind::DynamicArray(element) => element,
            _ => {
                return Err(Diagnostic::new(
                    span,
                    "mutable descriptor field requires a string or array view",
                ));
            }
        };
        match self.symbols.name(member) {
            "count" => Ok((
                SequenceField::Count,
                self.types.scalar(ScalarType::Int(IntegerType::S64)),
            )),
            "data" => Ok((
                SequenceField::Data,
                self.types
                    .pointer(element)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?,
            )),
            "allocated" if matches!(kind, TypeKind::DynamicArray(_)) => Ok((
                SequenceField::Allocated,
                self.types.scalar(ScalarType::Int(IntegerType::S64)),
            )),
            _ => Err(Diagnostic::new(span, "unknown sequence descriptor field")),
        }
    }

    pub(crate) fn sequence_member_place(
        &mut self,
        base: Place,
        member: Symbol,
        span: Span,
    ) -> Result<Place, Diagnostic> {
        let (field, _) = self.sequence_field_type(base.ty(), member, span)?;
        self.places
            .sequence_field(base, field, self.types)
            .map_err(|error| Diagnostic::new(span, error.to_string()))
    }
}

/// A temporary view borrows compiler-created function-local backing storage.
/// Returning that backing address directly would manufacture a dangling view.
pub(crate) fn reject_returned_temporary(
    value: &ValueExpr,
    types: &dyn jai_types::TypeView,
    span: Span,
) -> Result<(), Diagnostic> {
    let mut pending = vec![value];
    let mut escapes = false;
    while let Some(value) = pending.pop() {
        match value {
            ValueExpr::Bind { bindings, body, .. } => {
                pending.push(body);
                pending.extend(bindings.iter().map(|(_, value)| value));
            }
            ValueExpr::ArrayView { array, .. } => {
                if !jai_ir::is_static_value(array) {
                    escapes = true;
                    break;
                }
                pending.push(array);
            }
            ValueExpr::SequenceField {
                base,
                field: SequenceField::Data,
                ..
            } => {
                if matches!(
                    types.kind(base.type_id(types)),
                    Ok(TypeKind::FixedArray { .. })
                ) && !matches!(base.as_ref(), ValueExpr::Load(_))
                    && !jai_ir::is_static_value(base)
                {
                    escapes = true;
                    break;
                }
                pending.push(base);
            }
            ValueExpr::Conditional { expression, .. } => {
                pending.push(&expression.then_value);
                pending.push(&expression.else_value);
            }
            ValueExpr::Array { elements, .. } => pending.extend(elements),
            ValueExpr::Record { fields, .. } => pending.extend(fields),
            ValueExpr::OrderedRecord { initializers, .. } => {
                pending.extend(initializers.iter().map(|(_, value)| value));
            }
            ValueExpr::RecordBuild { initializers, .. } => {
                pending.extend(initializers.iter().map(|(_, value)| value));
            }
            ValueExpr::SequenceBuild { initializers, .. } => {
                pending.extend(initializers.iter().map(|(_, value)| value));
            }
            ValueExpr::Union { value, .. }
            | ValueExpr::Distinct { value, .. }
            | ValueExpr::UnwrapDistinct { value, .. }
            | ValueExpr::PointerCast { value, .. } => pending.push(value),
            ValueExpr::SequenceView { sequence, .. } => pending.push(sequence),
            ValueExpr::PointerOffset { pointer, .. } => pending.push(pointer),
            ValueExpr::Field { base, .. } | ValueExpr::Index { base, .. } => pending.push(base),
            // Loads and calls follow the source language's explicit storage lifetime.
            // Scalar fields and conditions cannot expose their temporary backing.
            _ => {}
        }
    }
    if escapes {
        Err(Diagnostic::new(
            span,
            "cannot return a view or data address of temporary array storage",
        ))
    } else {
        Ok(())
    }
}

impl Resolver<'_> {
    pub(crate) fn literal_constant(
        &self,
        value: ValueExpr,
        span: Span,
    ) -> Result<jai_ir::ConstantValue, Diagnostic> {
        fn convert(
            value: ValueExpr,
            types: &TypeRegistry,
            span: Span,
            depth: usize,
        ) -> Result<jai_ir::ConstantValue, Diagnostic> {
            if depth >= crate::constant_limits::MAX_CONSTANT_DEPTH {
                return Err(Diagnostic::new(
                    span,
                    "constant exceeds compiler constant depth budget",
                ));
            }
            let ty = value.type_id(types);
            let kind = match value {
                ValueExpr::NativePointer(value) => jai_ir::ConstantKind::NativePointer(value),
                ValueExpr::RuntimeType(value) => jai_ir::ConstantKind::RuntimeType(value),
                ValueExpr::Int(value) => match value.kind() {
                    IntExprKind::Constant(value) => jai_ir::ConstantKind::Int(*value),
                    _ => {
                        return Err(Diagnostic::new(
                            span,
                            "literal constant requires compile-time evaluation",
                        ));
                    }
                },
                ValueExpr::Bool(BoolExpr::Constant(value)) => jai_ir::ConstantKind::Bool(value),
                ValueExpr::Float(value) => match value.kind() {
                    FloatExprKind::Constant(value) => jai_ir::ConstantKind::Float(*value),
                    _ => {
                        return Err(Diagnostic::new(
                            span,
                            "literal constant requires compile-time evaluation",
                        ));
                    }
                },
                ValueExpr::Zero(_) => jai_ir::ConstantKind::Zero,
                ValueExpr::ProcedureValue { procedure, .. } => {
                    jai_ir::ConstantKind::Procedure(procedure)
                }
                ValueExpr::StringBytes { bytes, .. } => jai_ir::ConstantKind::StringBytes(bytes),
                ValueExpr::Array { elements, .. } => jai_ir::ConstantKind::Array(
                    elements
                        .into_iter()
                        .map(|value| convert(value, types, span, depth + 1))
                        .collect::<Result<Vec<_>, _>>()?,
                ),
                ValueExpr::Record { fields, .. } => jai_ir::ConstantKind::Record(
                    fields
                        .into_iter()
                        .map(|value| convert(value, types, span, depth + 1))
                        .collect::<Result<Vec<_>, _>>()?,
                ),
                ValueExpr::RecordBuild { initializers, .. } => {
                    let count = types
                        .record_definition(ty)
                        .map_err(|error| Diagnostic::new(span, error.to_string()))?
                        .fields
                        .len();
                    let mut fields = vec![None; count];
                    for (field, value) in initializers {
                        fields[field.index()] = Some(convert(value, types, span, depth + 1)?);
                    }
                    jai_ir::ConstantKind::Record(
                        fields
                            .into_iter()
                            .collect::<Option<Vec<_>>>()
                            .ok_or_else(|| {
                                Diagnostic::new(span, "record constant is missing an initializer")
                            })?,
                    )
                }
                ValueExpr::Enum { value, .. } => jai_ir::ConstantKind::Enum(value),
                ValueExpr::Distinct { value, .. } => jai_ir::ConstantKind::Distinct(Box::new(
                    convert(*value, types, span, depth + 1)?,
                )),
                ValueExpr::Union { field, value, .. } => jai_ir::ConstantKind::Union {
                    field,
                    value: Box::new(convert(*value, types, span, depth + 1)?),
                },
                _ => {
                    return Err(Diagnostic::new(
                        span,
                        "literal constant requires compile-time evaluation",
                    ));
                }
            };
            Ok(jai_ir::ConstantValue { ty, kind })
        }
        let constant = convert(value, self.types, span, 0)?;
        crate::constant_limits::cells(&constant).ok_or_else(|| {
            Diagnostic::new(span, "constant exceeds compiler constant cell budget")
        })?;
        Ok(constant)
    }
}
