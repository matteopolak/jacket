//! Validate pointer identities and pointees without interpreting host addresses.
use super::*;

impl Context<'_> {
    pub(super) fn pointer_value(&self, value: &ValueExpr, ty: TypeId) -> Result<(), IrError> {
        let _depth = self.enter()?;
        let TypeKind::Pointer(pointee) = *self.types.kind(ty)? else {
            return Err(IrError::InvalidValue(ty));
        };
        match value {
            ValueExpr::AddressOf {
                place,
                ty: declared,
            } => {
                same_type(ty, *declared)?;
                same_type(pointee, self.place(*place)?)?;
            }
            ValueExpr::AddressOfValue {
                value,
                ty: declared,
            } => {
                same_type(ty, *declared)?;
                same_type(pointee, self.value(value)?)?;
            }
            ValueExpr::PointerCast {
                value,
                ty: declared,
                mode,
            } => {
                if matches!(mode, CastMode::Force(_)) {
                    return Err(IrError::InvalidValue(ty));
                }
                same_type(ty, *declared)?;
                let source = self.value(value)?;
                if !matches!(self.types.kind(source)?, TypeKind::Pointer(_)) {
                    return Err(IrError::InvalidValue(source));
                }
                // Both cast modes preserve pointer values; the source language
                // permits changing the pointee but never integer addresses.
            }
            ValueExpr::PointerOffset {
                pointer,
                offset,
                ty: declared,
                ..
            } => {
                same_type(ty, *declared)?;
                same_type(ty, self.value(pointer)?)?;
                // Arithmetic needs runtime storage for a pointee, unlike a
                // pointer cast or null comparison, which can use *void.
                storage::runtime_type(self.types, pointee)?;
                self.integer(offset)?;
                same_integer(IntegerType::S64, offset.ty())?;
            }
            ValueExpr::PointerOffsetLeft {
                offset,
                pointer,
                ty: declared,
            } => {
                same_type(ty, *declared)?;
                self.integer(offset)?;
                same_integer(IntegerType::S64, offset.ty())?;
                same_type(ty, self.value(pointer)?)?;
                storage::runtime_type(self.types, pointee)?;
            }
            ValueExpr::PointerFromInteger {
                value,
                ty: declared,
                mode,
            } => {
                if matches!(mode, CastMode::Force(_)) {
                    return Err(IrError::InvalidValue(ty));
                }
                same_type(ty, *declared)?;
                self.integer(value)?;
            }
            _ => return Err(IrError::InvalidValue(ty)),
        }
        Ok(())
    }

    pub(super) fn pointer_compare(
        &self,
        left: &ValueExpr,
        right: &ValueExpr,
    ) -> Result<(), IrError> {
        let _depth = self.enter()?;
        let left_ty = self.value(left)?;
        if !matches!(
            self.types.kind(left_ty)?,
            TypeKind::Pointer(_) | TypeKind::Procedure(_)
        ) {
            return Err(IrError::InvalidValue(left_ty));
        }
        same_type(left_ty, self.value(right)?)
    }

    pub(super) fn pointer_truth(&self, value: &ValueExpr) -> Result<(), IrError> {
        let _depth = self.enter()?;
        let ty = self.value(value)?;
        if matches!(
            self.types.kind(ty)?,
            TypeKind::Pointer(_) | TypeKind::Procedure(_)
        ) {
            Ok(())
        } else {
            Err(IrError::InvalidValue(ty))
        }
    }
}
