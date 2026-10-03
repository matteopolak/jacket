//! Root-owned byte projections retain their exact non-widening permissions.
use super::*;

impl Memory {
    pub(crate) fn static_byte_view(
        &self,
        types: &dyn TypeView,
        pointer: &Pointer,
        view: &jai_ir::StaticByteView,
    ) -> Result<Pointer, Error> {
        view.validate_target(self.target.policy)
            .map_err(|error| Error::IrValidation(error.to_string()))?;
        let allocation = self.allocation(pointer)?;
        if !pointer.data()?.path.is_empty() || allocation.ty != view.backing_type() {
            return Err(Error::InvalidIr(
                "static byte view requires its exact typed root",
            ));
        }
        let end = view
            .offset()
            .checked_add(view.length())
            .ok_or(Error::CheckedCast)?;
        let layout = self.prepared_layout(types, allocation.ty)?;
        if end > layout.size || layout.size != allocation.virtual_extent {
            return Err(Error::InvalidIr(
                "static byte view differs from its prepared backing layout",
            ));
        }
        let byte = types.scalar(ScalarType::Int(IntegerType::U8));
        let mut result = pointer.clone();
        result.data_mut()?.path = vec![Projection::Bytes {
            offset: view.offset(),
            ty: byte,
        }];
        result.pointee = byte;
        result.data_mut()?.region = Some((view.offset(), end));
        result.data_mut()?.restricted_region = true;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_ir::{ConstantKind, ConstantValue, StaticDataBuilder, StaticDataLimits, StaticValue};
    use jai_types::{LayoutEngine, LayoutPolicy, RecordKind, TypeRegistry};

    fn policy32() -> LayoutPolicy {
        LayoutPolicy::new(
            jai_types::ScalarLayout::new(4, 4),
            [
                jai_types::ScalarLayout::new(1, 1),
                jai_types::ScalarLayout::new(2, 2),
                jai_types::ScalarLayout::new(4, 4),
                jai_types::ScalarLayout::new(8, 4),
            ],
            [
                jai_types::ScalarLayout::new(4, 4),
                jai_types::ScalarLayout::new(8, 4),
            ],
            jai_types::ScalarLayout::new(1, 1),
        )
        .unwrap()
    }
    #[test]
    fn sealed_static_byte_views_never_regain_owner_bytes_or_writable_storage() {
        for policy in [policy32(), LayoutPolicy::lp64()] {
            let mut types = TypeRegistry::new();
            let word = types.scalar(ScalarType::Int(IntegerType::U32));
            let record = types.reserve_record(RecordKind::Struct);
            types.define_record(record, [word, word]).unwrap();
            let mut builder = StaticDataBuilder::new();
            let object = builder.reserve(record, &types).unwrap();
            let view = builder.byte_view(object, policy, 0, 4, &types).unwrap();
            builder
                .define(
                    object,
                    StaticValue {
                        ty: record,
                        kind: jai_ir::StaticValueKind::Record(
                            [20, 22]
                                .into_iter()
                                .map(|n| {
                                    StaticValue::constant(ConstantValue {
                                        ty: word,
                                        kind: ConstantKind::Int(Integer::wrapping(
                                            IntegerType::U32,
                                            n,
                                        )),
                                    })
                                })
                                .collect(),
                        ),
                    },
                )
                .unwrap();
            builder.finish(&types, StaticDataLimits::default()).unwrap();
            let mut memory = Memory::with_layout(Limits::default(), policy);
            let root = memory
                .allocate(
                    &types,
                    record,
                    Some(Value::Record {
                        ty: record,
                        fields: [20, 22]
                            .into_iter()
                            .map(|n| Value::Int(Integer::wrapping(IntegerType::U32, n)))
                            .collect(),
                    }),
                )
                .unwrap();
            memory.freeze(&root).unwrap();
            let byte = memory.static_byte_view(&types, &root, &view).unwrap();
            let owner = memory
                .cast_pointer(&types, &byte, record, CastMode::Checked)
                .unwrap();
            assert!(matches!(
                memory.load(&types, &owner),
                Err(Error::OutOfBounds { .. })
            ));
            assert!(memory.field(&types, &owner, 1).is_err());
            let byte_pointer = types.pointer(byte.pointee()).unwrap();
            let image = ByteImage::encode(
                &types,
                memory.target(),
                byte_pointer,
                &Value::Pointer(byte.clone()),
                1024,
            )
            .unwrap();
            let mut copied = ByteImage::uninitialized(memory.target(), image.len(), 1024).unwrap();
            copied.copy_range_from(&image, 0, 0, image.len()).unwrap();
            let recovered = copied
                .read(&types, memory.target(), 0, byte_pointer)
                .unwrap()
                .pointer()
                .unwrap()
                .clone();
            let recovered_owner = memory
                .cast_pointer(&types, &recovered, record, CastMode::Checked)
                .unwrap();
            assert!(memory.load(&types, &recovered_owner).is_err());
            let saved = memory
                .allocate(&types, byte_pointer, Some(Value::Pointer(byte.clone())))
                .unwrap();
            let saved = memory
                .load(&types, &saved)
                .unwrap()
                .pointer()
                .unwrap()
                .clone();
            let saved_owner = memory
                .cast_pointer(&types, &saved, record, CastMode::Checked)
                .unwrap();
            assert!(memory.load(&types, &saved_owner).is_err());
            let number = memory
                .pointer_to_integer(&types, &byte, IntegerType::U64, CastMode::Checked)
                .unwrap();
            let owner = memory
                .integer_to_pointer(&types, number, record, CastMode::Checked)
                .unwrap();
            assert!(memory.load(&types, &owner).is_err());
            assert!(memory.offset(&types, &byte, 5).is_err());
            let one_past = memory.offset(&types, &byte, 4).unwrap();
            assert!(memory.load(&types, &one_past).is_err());
            assert!(memory.validate_slice(&types, &byte, 5).is_err());
            assert!(matches!(
                memory.store(
                    &types,
                    &byte,
                    Value::Int(Integer::wrapping(IntegerType::U8, 0))
                ),
                Err(Error::ReadOnlyStorage)
            ));
            let wrong = if policy == policy32() {
                LayoutPolicy::lp64()
            } else {
                policy32()
            };
            assert!(view.validate_target(wrong).is_err());
        }
    }

    #[test]
    fn sealed_static_byte_views_keep_real_typed_relocations_and_empty_bounds() {
        for policy in [policy32(), LayoutPolicy::lp64()] {
            let mut types = TypeRegistry::new();
            let word = types.scalar(ScalarType::Int(IntegerType::U32));
            let pointer_ty = types.pointer(word).unwrap();
            let record = types.reserve_record(RecordKind::Struct);
            types.define_record(record, [word, pointer_ty]).unwrap();
            let mut layouts = LayoutEngine::new(&types, policy);
            let layout = layouts.layout(record).unwrap();
            let mut builder = StaticDataBuilder::new();
            let object = builder.reserve(record, &types).unwrap();
            let view = builder
                .byte_view(object, policy, 0, layout.size, &types)
                .unwrap();
            let empty = builder
                .byte_view(object, policy, layout.size, 0, &types)
                .unwrap();
            let mut memory = Memory::with_layout(Limits::default(), policy);
            let referent = memory
                .allocate(
                    &types,
                    word,
                    Some(Value::Int(Integer::wrapping(IntegerType::U32, 22))),
                )
                .unwrap();
            let root = memory
                .allocate(
                    &types,
                    record,
                    Some(Value::Record {
                        ty: record,
                        fields: vec![
                            Value::Int(Integer::wrapping(IntegerType::U32, 20)),
                            Value::Pointer(referent.clone()),
                        ],
                    }),
                )
                .unwrap();
            memory.freeze(&root).unwrap();
            let byte = memory.static_byte_view(&types, &root, &view).unwrap();
            let cell = memory
                .offset(&types, &byte, layout.field_offsets[1] as isize)
                .unwrap();
            let cell = memory
                .cast_pointer(&types, &cell, pointer_ty, CastMode::Checked)
                .unwrap();
            let recovered = memory
                .load(&types, &cell)
                .unwrap()
                .pointer()
                .unwrap()
                .clone();
            assert!(memory.same_address(&types, &recovered, &referent).unwrap());
            let one_past = memory.static_byte_view(&types, &root, &empty).unwrap();
            memory.validate_slice(&types, &one_past, 0).unwrap();
            assert!(memory.load(&types, &one_past).is_err());
            assert!(memory.offset(&types, &one_past, -1).is_err());
        }
    }
}
