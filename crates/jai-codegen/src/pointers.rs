//! Native pointer operations use LLVM pointers and typed element strides.
use super::*;
use jai_ir::CheckMode;
mod index_values;

impl<'ctx> Generator<'ctx, '_, '_> {
    pub(super) fn pointer_difference(
        &mut self,
        left: &ValueExpr,
        right: &ValueExpr,
    ) -> Result<IntValue<'ctx>, Error> {
        let ty = left.type_id(self.types);
        if right.type_id(self.types) != ty {
            return Err(Error::Invariant);
        }
        let TypeKind::Pointer(element) = *self.types.kind(ty)? else {
            return Err(Error::Invariant);
        };
        let element = self.lowerer.basic(element)?;
        let left = self.value(left)?.into_pointer_value();
        let right = self.value(right)?.into_pointer_value();
        Ok(self
            .builder
            .build_ptr_diff(element, left, right, "pointer.difference")?)
    }
    pub(super) fn conditional_value(
        &mut self,
        ty: TypeId,
        expression: &Conditional<ValueExpr>,
    ) -> Result<BasicValueEnum<'ctx>, Error> {
        if let Some(selected) = self.native_condition(&expression.condition) {
            self.boolean(&expression.condition)?;
            return self.value(if selected {
                &expression.then_value
            } else {
                &expression.else_value
            });
        }
        let [(yes, yes_end), (no, no_end)] = self.conditional_values(expression, Self::value)?;
        let phi = self
            .builder
            .build_phi(self.lowerer.basic(ty)?, "ifx.value")?;
        phi.add_incoming(&[(&yes, yes_end), (&no, no_end)]);
        Ok(phi.as_basic_value())
    }
    pub(super) fn dereference_slot(
        &mut self,
        value: &ValueExpr,
        ty: TypeId,
    ) -> Result<Slot<'ctx>, Error> {
        if !matches!(self.types.kind(value.type_id(self.types))?, TypeKind::Pointer(element) if *element == ty)
        {
            return Err(Error::Invariant);
        }
        let pointer = self.value(value)?.into_pointer_value();
        let valid = self.builder.build_is_not_null(pointer, "pointer.nonnull")?;
        self.check_cast(Bit(valid))?;
        Ok(Slot {
            pointer,
            ty,
            alignment: 1,
        })
    }

    pub(super) fn index_slot(
        &mut self,
        base: Place,
        index: &IntExpr,
        ty: TypeId,
        check: CheckMode,
    ) -> Result<Slot<'ctx>, Error> {
        let kind = self.types.kind(base.ty())?.clone();
        let base_slot = self.slot(base)?;
        let (pointer, count, element) = match kind {
            TypeKind::FixedArray { element, count } => (
                base_slot.pointer,
                Some(self.context.i64_type().const_int(count, false)),
                element,
            ),
            kind @ (TypeKind::Slice(_) | TypeKind::DynamicArray(_) | TypeKind::String) => {
                let element = match kind {
                    TypeKind::Slice(element) | TypeKind::DynamicArray(element) => element,
                    TypeKind::String => self
                        .types
                        .scalar(jai_types::ScalarType::Int(IntegerType::U8)),
                    _ => return Err(Error::Invariant),
                };
                let descriptor = crate::memory::load(
                    &self.builder,
                    self.lowerer.basic(base.ty())?,
                    base_slot.pointer,
                    "index.descriptor",
                    base_slot.alignment,
                )?
                .into_struct_value();
                let count = self
                    .builder
                    .build_extract_value(descriptor, 0, "index.count")?
                    .into_int_value();
                let pointer = self
                    .builder
                    .build_extract_value(descriptor, 1, "index.data")?
                    .into_pointer_value();
                (pointer, Some(count), element)
            }
            TypeKind::Pointer(element) => {
                let pointer = crate::memory::load(
                    &self.builder,
                    self.lowerer.basic(base.ty())?,
                    base_slot.pointer,
                    "index.pointer",
                    base_slot.alignment,
                )?
                .into_pointer_value();
                (pointer, None, element)
            }
            _ => return Err(Error::Invariant),
        };
        if element != ty {
            return Err(Error::Invariant);
        }
        let domain = index.ty();
        let index = self.int(index)?.0;
        if check.enabled()
            && let Some(count) = count
        {
            self.check_index(index, count)?;
        }
        let index = self.index_displacement(index, domain, element)?;
        let nonnull = self.builder.build_is_not_null(pointer, "index.nonnull")?;
        self.check_cast(Bit(nonnull))?;
        let pointer = jai_llvm::gep(
            &self.builder,
            self.lowerer.basic(element)?,
            pointer,
            &[index],
            "index.address",
        )?;
        Ok(Slot {
            pointer,
            ty,
            alignment: 1,
        })
    }

    fn check_index(&mut self, index: IntValue<'ctx>, count: IntValue<'ctx>) -> Result<(), Error> {
        // Unsigned comparison also rejects every negative signed index.
        let in_bounds =
            self.builder
                .build_int_compare(IntPredicate::ULT, index, count, "index.in.bounds")?;
        let nonnegative_count = self.builder.build_int_compare(
            IntPredicate::SGE,
            count,
            count.get_type().const_zero(),
            "index.valid.count",
        )?;
        let valid = self
            .builder
            .build_and(in_bounds, nonnegative_count, "index.valid")?;
        self.check_cast(Bit(valid))
    }

    pub(super) fn index_value(
        &mut self,
        base: &ValueExpr,
        index: &IntExpr,
        ty: TypeId,
        check: CheckMode,
    ) -> Result<BasicValueEnum<'ctx>, Error> {
        let kind = self.types.kind(base.type_id(self.types))?.clone();
        // Snapshot the base before evaluating the index. In particular, a call
        // in the index must not change the array value we already evaluated.
        let snapshot = self.value(base)?;
        let (pointer, count, element) = match kind {
            TypeKind::FixedArray { element, count } => {
                let pointer = crate::unions::entry_alloca(
                    self.context,
                    &self.builder,
                    snapshot.get_type(),
                    "index.snapshot",
                )?;
                crate::memory::store(&self.builder, pointer, snapshot, 1)?;
                (
                    pointer,
                    Some(self.context.i64_type().const_int(count, false)),
                    element,
                )
            }
            TypeKind::Slice(element) | TypeKind::DynamicArray(element) => {
                let descriptor = snapshot.into_struct_value();
                let count = self
                    .builder
                    .build_extract_value(descriptor, 0, "index.count")?
                    .into_int_value();
                let pointer = self
                    .builder
                    .build_extract_value(descriptor, 1, "index.data")?
                    .into_pointer_value();
                (pointer, Some(count), element)
            }
            TypeKind::String => {
                let descriptor = snapshot.into_struct_value();
                let count = self
                    .builder
                    .build_extract_value(descriptor, 0, "index.count")?
                    .into_int_value();
                let pointer = self
                    .builder
                    .build_extract_value(descriptor, 1, "index.data")?
                    .into_pointer_value();
                (
                    pointer,
                    Some(count),
                    self.types
                        .scalar(jai_types::ScalarType::Int(IntegerType::U8)),
                )
            }
            TypeKind::Pointer(element) => (snapshot.into_pointer_value(), None, element),
            _ => return Err(Error::Invariant),
        };
        if element != ty {
            return Err(Error::Invariant);
        }
        let domain = index.ty();
        let index = self.int(index)?.0;
        if check.enabled()
            && let Some(count) = count
        {
            self.check_index(index, count)?;
        }
        let index = self.index_displacement(index, domain, element)?;
        let nonnull = self.builder.build_is_not_null(pointer, "index.nonnull")?;
        self.check_cast(Bit(nonnull))?;
        let element = self.lowerer.basic(element)?;
        let pointer = jai_llvm::gep(&self.builder, element, pointer, &[index], "index.address")?;
        crate::memory::load(&self.builder, element, pointer, "index.value", 1)
    }
    pub(super) fn address_value(
        &mut self,
        place: Place,
        ty: TypeId,
    ) -> Result<BasicValueEnum<'ctx>, Error> {
        if !matches!(self.types.kind(ty)?, TypeKind::Pointer(element) if *element == place.ty()) {
            return Err(Error::Invariant);
        }
        Ok(self.slot(place)?.pointer.into())
    }

    pub(super) fn cast_pointer(
        &mut self,
        value: &ValueExpr,
        ty: TypeId,
        mode: CastMode,
    ) -> Result<BasicValueEnum<'ctx>, Error> {
        Self::numeric_pointer_mode(mode)?;
        if !matches!(
            self.types.kind(value.type_id(self.types))?,
            TypeKind::Pointer(_)
        ) || !matches!(self.types.kind(ty)?, TypeKind::Pointer(_))
        {
            return Err(Error::Invariant);
        }
        // LLVM opaque pointers retain the address while the shared IR retains
        // the pointee identity that governs every subsequent load and GEP.
        Ok(self.value(value)?.into_pointer_value().into())
    }

    pub(super) fn offset_pointer(
        &mut self,
        value: &ValueExpr,
        offset: &IntExpr,
        subtract: bool,
        ty: TypeId,
    ) -> Result<BasicValueEnum<'ctx>, Error> {
        let TypeKind::Pointer(element) = *self.types.kind(ty)? else {
            return Err(Error::Invariant);
        };
        if value.type_id(self.types) != ty || offset.ty() != IntegerType::S64 {
            return Err(Error::Invariant);
        }
        let pointer = self.value(value)?.into_pointer_value();
        let offset = self.int(offset)?.0;
        let offset = if subtract {
            self.builder.build_int_neg(offset, "pointer.offset.neg")?
        } else {
            offset
        };
        let element = self.lowerer.basic(element)?;
        Ok(jai_llvm::gep(&self.builder, element, pointer, &[offset], "pointer.offset")?.into())
    }

    pub(super) fn pointer_truth(&mut self, value: &ValueExpr) -> Result<IntValue<'ctx>, Error> {
        if !matches!(
            self.types.kind(value.type_id(self.types))?,
            TypeKind::Pointer(_) | TypeKind::Procedure(_)
        ) {
            return Err(Error::Invariant);
        }
        let pointer = self.value(value)?.into_pointer_value();
        Ok(self.builder.build_is_not_null(pointer, "pointer.truth")?)
    }

    pub(super) fn compare_pointers(
        &mut self,
        operation: Equality,
        left: &ValueExpr,
        right: &ValueExpr,
    ) -> Result<IntValue<'ctx>, Error> {
        if left.type_id(self.types) != right.type_id(self.types)
            || !matches!(
                self.types.kind(left.type_id(self.types))?,
                TypeKind::Pointer(_) | TypeKind::Procedure(_)
            )
        {
            return Err(Error::Invariant);
        }
        let left = self.value(left)?.into_pointer_value();
        let right = self.value(right)?.into_pointer_value();
        Ok(jai_llvm::compare_pointers(
            &self.builder,
            left,
            right,
            match operation {
                Equality::Equal => IntPredicate::EQ,
                Equality::NotEqual => IntPredicate::NE,
            },
            "pointer.compare",
        )?)
    }
}
