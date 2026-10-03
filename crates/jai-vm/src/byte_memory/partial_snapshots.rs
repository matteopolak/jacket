//! Preserve aggregate storage without decoding undefined or overlapping fields.
use super::*;
use std::collections::HashSet;

impl ByteImage {
    pub(super) fn partial_snapshot(
        &self,
        types: &dyn TypeView,
        layouts: &mut LayoutEngine<'_>,
        offset: usize,
        ty: TypeId,
        limit: usize,
        allow_unknown: bool,
    ) -> Result<Option<Value>, Error> {
        let mut representation = ty;
        let mut seen = HashSet::new();
        while let TypeKind::Distinct(id) = types.kind(representation)? {
            if !seen.insert(representation) || seen.len() > limit {
                return Err(Error::Limit(LimitKind::ValueCells));
            }
            representation = types.distinct(*id)?.representation;
        }
        if !matches!(
            types.kind(representation)?,
            TypeKind::Record(_) | TypeKind::Any(_) | TypeKind::FixedArray { .. }
        ) {
            return Ok(None);
        }
        let layout = layouts.layout(ty)?.clone();
        let length = size(layout.size, limit)?;
        let range = self.range(offset, length)?;
        if !(allow_unknown && self.initialized[range.clone()].iter().any(|byte| !*byte))
            && !has_placements(types, ty, limit)?
        {
            return Ok(None);
        }
        length
            .checked_add(self.range_metadata_cells(&range))
            .and_then(|cells| cells.checked_add(1))
            .filter(|cells| *cells <= limit)
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        let mut image = self.extract_range(offset, length)?;
        image.limit = limit;
        Ok(Some(Value::StoredAggregate(StoredAggregate::opaque(
            types, ty, image, &layout, limit,
        )?)))
    }
}

fn has_placements(types: &dyn TypeView, root: TypeId, limit: usize) -> Result<bool, Error> {
    let mut pending = vec![root];
    let mut seen = HashSet::new();
    while let Some(ty) = pending.pop() {
        if !seen.insert(ty) {
            continue;
        }
        if seen.len().saturating_add(pending.len()) > limit {
            return Err(Error::Limit(LimitKind::ValueCells));
        }
        match types.kind(ty)? {
            TypeKind::Record(_) | TypeKind::Any(_) => {
                let record = types.record_storage_definition(ty)?;
                if record.layout.field_placements.iter().any(Option::is_some) {
                    return Ok(true);
                }
                if record.fields.len()
                    > limit
                        .saturating_sub(seen.len())
                        .saturating_sub(pending.len())
                {
                    return Err(Error::Limit(LimitKind::ValueCells));
                }
                pending.extend(record.fields.iter().copied());
            }
            TypeKind::Distinct(id) => pending.push(types.distinct(*id)?.representation),
            TypeKind::FixedArray {
                element,
                count,
            } if *count != 0 => pending.push(*element),
            _ => {}
        }
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_types::{RecordLayout, TypeRegistry};

    #[test]
    fn placed_copy_keeps_unknown_bytes_and_reads_only_initialized_subranges() {
        let mut types = TypeRegistry::new();
        let word = types.scalar(ScalarType::Int(IntegerType::U64));
        let byte = types.scalar(ScalarType::Int(IntegerType::U8));
        let info = types.reserve_record(RecordKind::Struct);
        types.define_record(info, [word, word]).unwrap();
        let padding = types.fixed_array(byte, 64).unwrap();
        let slice = types.slice(byte).unwrap();
        let worker = types.reserve_record(RecordKind::Struct);
        types
            .define_record_with_placements(
                worker,
                [info, padding, slice],
                RecordLayout::default(),
                [None, Some(0), None],
            )
            .unwrap();
        let target = ByteTarget::default();
        let mut image = ByteImage::uninitialized(target, 80, 4096).unwrap();
        image
            .write(
                &types,
                target,
                0,
                word,
                &Value::Int(Integer::wrapping(IntegerType::U64, 41)),
            )
            .unwrap();
        image
            .write(
                &types,
                target,
                8,
                word,
                &Value::Int(Integer::wrapping(IntegerType::U64, 1)),
            )
            .unwrap();
        image
            .write(
                &types,
                target,
                63,
                byte,
                &Value::Int(Integer::wrapping(IntegerType::U8, 99)),
            )
            .unwrap();
        image
            .write(
                &types,
                target,
                64,
                slice,
                &Value::Slice {
                    ty: slice,
                    count: 0,
                    pointer: crate::Pointer::null(byte),
                },
            )
            .unwrap();
        let value = image.read_preserving(&types, target, 0, worker).unwrap();
        let Value::StoredAggregate(snapshot) = &value else {
            panic!("placed storage must retain its byte carrier");
        };
        assert!(snapshot.decoded_semantic().is_none());
        assert!(std::ptr::eq(value.semantic(), &value));
        value.validate(&types, worker, 128).unwrap();
        let Value::Record {
            fields, ..
        } = snapshot.field(&types, 0, 4096).unwrap()
        else {
            panic!("initialized info is readable");
        };
        assert_eq!(
            fields[0].integer().unwrap().value() + fields[1].integer().unwrap().value(),
            42
        );
        let Value::StoredAggregate(padding) = snapshot.field(&types, 1, 4096).unwrap() else {
            panic!("padding retains its own initialization mask");
        };
        assert_eq!(
            padding
                .index(&types, 63, 4096)
                .unwrap()
                .integer()
                .unwrap()
                .value(),
            99
        );
        assert!(padding.index(&types, 62, 4096).is_err());
        assert_eq!(
            ByteImage::encode(&types, target, worker, &value, 4096).unwrap(),
            image
        );
        assert!(snapshot.publication_semantic(&types, 4096).is_err());
        assert!(snapshot.field(&types, 0, 4).is_err());
    }

    #[test]
    fn opaque_copy_preserves_data_pointer_provenance_and_undefined_tail() {
        let mut types = TypeRegistry::new();
        let word = types.scalar(ScalarType::Int(IntegerType::U64));
        let byte = types.scalar(ScalarType::Int(IntegerType::U8));
        let pointer_ty = types.pointer(word).unwrap();
        let bytes = types.fixed_array(byte, 8).unwrap();
        let record = types.reserve_record(RecordKind::Struct);
        types
            .define_record_with_placements(
                record,
                [pointer_ty, bytes, bytes],
                RecordLayout::default(),
                [None, Some(0), None],
            )
            .unwrap();
        let mut memory = crate::Memory::new(crate::Limits::default());
        let pointee = memory
            .allocate(
                &types,
                word,
                Some(Value::Int(Integer::wrapping(IntegerType::U64, 42))),
            )
            .unwrap();
        let pointer = Value::Pointer(pointee.clone());
        let target = ByteTarget::default();
        let mut image = ByteImage::uninitialized(target, 16, 4096).unwrap();
        image
            .write(&types, target, 0, pointer_ty, &pointer)
            .unwrap();
        let value = image.read_preserving(&types, target, 0, record).unwrap();
        let Value::StoredAggregate(snapshot) = &value else {
            panic!("pointer overlay requires a byte carrier");
        };
        assert_eq!(snapshot.field(&types, 0, 4096).unwrap(), pointer);
        let destination = memory.allocate(&types, record, Some(value)).unwrap();
        let copied = memory.load(&types, &destination).unwrap();
        let Value::StoredAggregate(copy) = copied else {
            panic!("copy must retain unknown bytes");
        };
        assert_eq!(copy.field(&types, 0, 4096).unwrap(), pointer);
        let Value::StoredAggregate(tail) = copy.field(&types, 2, 4096).unwrap() else {
            panic!("undefined tail must retain its mask");
        };
        assert!(tail.index(&types, 0, 4096).is_err());
        assert_eq!(
            memory
                .load(&types, &pointee)
                .unwrap()
                .integer()
                .unwrap()
                .value(),
            42
        );
        assert!(copy.publication_semantic(&types, 4096).is_err());
    }
}
