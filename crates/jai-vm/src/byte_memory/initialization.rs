//! Initialization follows written byte extents and never fabricates a value.
use super::*;
use std::ops::Range;

impl ByteImage {
    pub(crate) fn uninitialized(
        target: ByteTarget,
        length: usize,
        limit: usize,
    ) -> Result<Self, Error> {
        if length > limit {
            return Err(Error::Limit(LimitKind::ValueCells));
        }
        Ok(Self {
            bytes: vec![0; length],
            initialized: vec![false; length],
            provenance: vec![],
            relocations: vec![],
            unions: vec![],
            target,
            limit,
        })
    }

    pub(super) fn initialized_range(
        &self,
        offset: usize,
        length: usize,
    ) -> Result<Range<usize>, Error> {
        let range = self.range(offset, length)?;
        if self.initialized[range.clone()]
            .iter()
            .any(|written| !written)
        {
            return Err(Error::Uninitialized);
        }
        Ok(range)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Limits, Memory};
    use jai_types::{CastMode, Integer, IntegerType, RecordKind, ScalarType, TypeRegistry};

    fn integer(value: i128) -> Value {
        Value::Int(Integer::wrapping(IntegerType::S64, value))
    }

    #[test]
    fn any_pointer_fields_initialize_independently_without_fabricating_nulls() {
        let mut types = TypeRegistry::new();
        let int = types.scalar(ScalarType::Int(IntegerType::S64));
        let header = types.reserve_record(RecordKind::Struct);
        types.define_record(header, [int]).unwrap();
        let any = types.reserve_any();
        types.define_any(any, header).unwrap();
        let mut memory = Memory::new(Limits::default());
        let descriptor_header = memory
            .allocate(
                &types,
                header,
                Some(Value::Record {
                    ty: header,
                    fields: vec![integer(77)],
                }),
            )
            .unwrap();
        let value = memory.allocate(&types, int, Some(integer(42))).unwrap();
        let payload = memory
            .cast_pointer(&types, &value, types.void(), CastMode::Unchecked)
            .unwrap();
        let descriptor = memory.allocate(&types, any, None).unwrap();
        let type_field = memory.field(&types, &descriptor, 0).unwrap();
        let value_field = memory.field(&types, &descriptor, 1).unwrap();
        memory
            .store(
                &types,
                &type_field,
                Value::Pointer(descriptor_header.clone()),
            )
            .unwrap();
        assert_eq!(
            memory.load(&types, &type_field).unwrap(),
            Value::Pointer(descriptor_header.clone())
        );
        assert!(matches!(
            memory.load(&types, &value_field),
            Err(Error::Uninitialized)
        ));
        assert!(matches!(
            memory.load(&types, &descriptor),
            Err(Error::Uninitialized)
        ));
        memory
            .store(&types, &value_field, Value::Pointer(payload.clone()))
            .unwrap();
        assert_eq!(
            memory.load(&types, &descriptor).unwrap(),
            Value::Record {
                ty: any,
                fields: vec![Value::Pointer(descriptor_header), Value::Pointer(payload)]
            }
        );
    }

    #[test]
    fn copy_and_memmove_preserve_holes_and_fill_initializes_only_its_range() {
        let target = ByteTarget::default();
        let mut source = ByteImage::uninitialized(target, 8, 8).unwrap();
        source.write_range(0, &[1, 2]).unwrap();
        let mut copy = ByteImage::from_bytes(target, vec![9; 8], 8).unwrap();
        copy.copy_range_from(&source, 0, 0, 8).unwrap();
        assert_eq!(copy.read_range(0, 2).unwrap(), &[1, 2]);
        assert!(matches!(copy.read_range(2, 1), Err(Error::Uninitialized)));
        copy.copy_range_within(0, 1, 7).unwrap();
        assert_eq!(copy.read_range(0, 3).unwrap(), &[1, 1, 2]);
        assert!(matches!(copy.read_range(3, 1), Err(Error::Uninitialized)));
        copy.fill_range(3, 4, 0).unwrap();
        assert_eq!(copy.read_range(0, 7).unwrap(), &[1, 1, 2, 0, 0, 0, 0]);
        assert!(matches!(copy.read_range(7, 1), Err(Error::Uninitialized)));
        assert!(ByteImage::uninitialized(target, 9, 8).is_err());
    }

    #[test]
    fn record_reconstruction_checks_semantic_fields_and_leaves_padding_unknown() {
        let mut types = TypeRegistry::new();
        let byte = types.scalar(ScalarType::Int(IntegerType::U8));
        let int = types.scalar(ScalarType::Int(IntegerType::S64));
        let record = types.reserve_record(RecordKind::Struct);
        types.define_record(record, [byte, int]).unwrap();
        let target = ByteTarget::default();
        let mut image = ByteImage::uninitialized(target, 16, 64).unwrap();
        let first = Value::Int(Integer::wrapping(IntegerType::U8, 7));
        image.write(&types, target, 0, byte, &first).unwrap();
        image.write(&types, target, 8, int, &integer(42)).unwrap();
        assert_eq!(
            image.read(&types, target, 0, record).unwrap(),
            Value::Record {
                ty: record,
                fields: vec![first, integer(42)]
            }
        );
        assert!(matches!(image.read_range(1, 1), Err(Error::Uninitialized)));
    }

    #[test]
    fn opaque_union_copy_keeps_its_selected_view_and_inactive_bytes_unknown() {
        let mut types = TypeRegistry::new();
        let byte = types.scalar(ScalarType::Int(IntegerType::U8));
        let int = types.scalar(ScalarType::Int(IntegerType::S64));
        let union = types.reserve_record(RecordKind::Union);
        types.define_record(union, [byte, int]).unwrap();
        let target = ByteTarget::default();
        let mut source = ByteImage::uninitialized(target, 8, 64).unwrap();
        let first = Value::Int(Integer::wrapping(IntegerType::U8, 42));
        source.write(&types, target, 0, byte, &first).unwrap();
        source.note_union_field(&types, 0, union, 0).unwrap();
        let mut copy = ByteImage::uninitialized(target, 8, 64).unwrap();
        copy.copy_range_from(&source, 0, 0, 8).unwrap();
        assert_eq!(
            copy.read(&types, target, 0, union).unwrap(),
            Value::Union {
                ty: union,
                field: 0,
                value: Box::new(first)
            }
        );
        assert!(matches!(
            copy.read_union_field(&types, 0, union, 1),
            Err(Error::Uninitialized)
        ));
    }

    #[test]
    fn first_partial_store_failure_keeps_the_original_allocation_uninitialized() {
        let mut types = TypeRegistry::new();
        let int = types.scalar(ScalarType::Int(IntegerType::S64));
        let record = types.reserve_record(RecordKind::Struct);
        types.define_record(record, [int, int]).unwrap();
        let preparation = Memory::new(Limits::default());
        for ty in [record, int] {
            preparation
                .prepare_layout(&types, ty, usize::MAX)
                .1
                .unwrap();
        }
        let mut memory = Memory::new(Limits {
            value_cells: preparation.value_cells() + 15,
            ..Limits::default()
        });
        for ty in [record, int] {
            memory.prepare_layout(&types, ty, usize::MAX).1.unwrap();
        }
        let root = memory.allocate(&types, record, None).unwrap();
        let field = memory.field(&types, &root, 0).unwrap();
        let baseline = memory.value_cells();
        assert!(matches!(
            memory.store(&types, &field, integer(42)),
            Err(Error::Limit(LimitKind::ValueCells))
        ));
        assert_eq!(memory.value_cells(), baseline);
        assert!(matches!(
            memory.load(&types, &field),
            Err(Error::Uninitialized)
        ));
    }

    #[test]
    fn string_backing_reads_do_not_bypass_holes_copied_from_partial_storage() {
        let mut types = TypeRegistry::new();
        let byte = types.scalar(ScalarType::Int(IntegerType::U8));
        let record = types.reserve_record(RecordKind::Struct);
        types.define_record(record, [byte, byte]).unwrap();
        let mut memory = Memory::new(Limits::default());
        let partial = memory.allocate(&types, record, None).unwrap();
        let field = memory.field(&types, &partial, 0).unwrap();
        memory
            .store(
                &types,
                &field,
                Value::Int(Integer::wrapping(IntegerType::U8, 7)),
            )
            .unwrap();
        let text = memory
            .allocate(&types, types.string(), Some(Value::String(vec![1, 2])))
            .unwrap();
        memory.byte_copy(&types, &text, &partial, 2).unwrap();
        assert!(matches!(
            memory.load(&types, &text),
            Err(Error::Uninitialized)
        ));
    }
}
