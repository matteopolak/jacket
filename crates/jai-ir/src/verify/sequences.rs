use super::*;

fn array_view(types: &dyn TypeView, source: TypeId, target: TypeId) -> Result<(), IrError> {
    let TypeKind::FixedArray { element, .. } = *types.kind(source)? else {
        return Err(IrError::InvalidValue(source));
    };
    storage::runtime_type(types, element)?;
    let expected = match *types.kind(target)? {
        TypeKind::Slice(element) => element,
        TypeKind::String => types.scalar(ScalarType::Int(IntegerType::U8)),
        _ => return Err(IrError::InvalidValue(target)),
    };
    same_type(expected, element)
}

impl Context<'_> {
    pub(super) fn sequence_concat(
        &self,
        ty: TypeId,
        element: TypeId,
        parts: &[SequencePackPart],
    ) -> Result<(), IrError> {
        let _depth = self.enter()?;
        let TypeKind::Slice(actual) = *self.types.kind(ty)? else {
            return Err(IrError::InvalidValue(ty));
        };
        same_type(element, actual)?;
        storage::runtime_type(self.types, element)?;
        for part in parts {
            match part {
                SequencePackPart::Element(value) => same_type(element, self.value(value)?)?,
                SequencePackPart::Spread(value) => same_type(ty, self.value(value)?)?,
            }
        }
        Ok(())
    }

    pub(super) fn sequence_value(&self, value: &ValueExpr, ty: TypeId) -> Result<(), IrError> {
        let _depth = self.enter()?;
        let types = self.types;
        storage::runtime_type(types, ty)?;
        same_type(ty, value.type_id(types))?;
        match value {
            ValueExpr::Array { elements, .. } => {
                let TypeKind::FixedArray { element, count } = *types.kind(ty)? else {
                    return Err(IrError::InvalidValue(ty));
                };
                storage::runtime_type(types, element)?;
                let count = usize::try_from(count).map_err(|_| IrError::InvalidValue(ty))?;
                arity("array elements", count, elements.len())?;
                for value in elements {
                    same_type(element, self.value(value)?)?;
                }
            }
            ValueExpr::StringBytes { .. } => {
                if !matches!(types.kind(ty)?, TypeKind::String) {
                    return Err(IrError::InvalidValue(ty));
                }
            }
            ValueExpr::SequenceField { base, field, .. } => {
                let base = self.value(base)?;
                same_type(crate::sequences::field_type(types, base, *field)?, ty)?;
            }
            ValueExpr::ArrayToSlice { array, .. } => {
                array_view(types, self.place(*array)?, ty)?;
            }
            ValueExpr::ArrayView { array, .. } => {
                array_view(types, self.value(array)?, ty)?;
            }
            ValueExpr::SequenceView { sequence, .. } => {
                let source = self.value(sequence)?;
                if !matches!(
                    types.kind(source)?,
                    TypeKind::Slice(_) | TypeKind::DynamicArray(_) | TypeKind::String
                ) {
                    return Err(IrError::InvalidValue(source));
                }
                let TypeKind::Slice(element) = *types.kind(ty)? else {
                    return Err(IrError::InvalidValue(ty));
                };
                same_type(element, crate::sequences::element(types, source)?)?;
            }
            ValueExpr::Index { base, index, .. } => {
                let base = self.value(base)?;
                let element = crate::sequences::element(types, base)?;
                storage::runtime_type(types, element)?;
                self.integer(index)?;
                same_integer(crate::canonical_index_type(index.ty()), index.ty())?;
                same_type(element, ty)?;
            }
            ValueExpr::SequenceBuild { initializers, .. } => {
                let element = match *types.kind(ty)? {
                    TypeKind::Slice(element) | TypeKind::DynamicArray(element) => element,
                    TypeKind::String => types.scalar(ScalarType::Int(IntegerType::U8)),
                    _ => return Err(IrError::InvalidValue(ty)),
                };
                storage::runtime_type(types, element)?;
                let mut fields = [false; 3];
                // Retain source order; omitted descriptor fields have zero defaults.
                for (field, value) in initializers {
                    let expected = crate::sequences::field_type(types, ty, *field)?;
                    if std::mem::replace(&mut fields[field.index()], true) {
                        return Err(IrError::DuplicateIdentity {
                            kind: "sequence field",
                            index: field.index(),
                        });
                    }
                    same_type(expected, self.value(value)?)?;
                }
            }
            _ => return Err(IrError::InvalidValue(ty)),
        }
        Ok(())
    }
}
