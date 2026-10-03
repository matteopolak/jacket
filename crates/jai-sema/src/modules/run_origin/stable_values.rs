//! Storage constraints encode physical ordinals, never record arena identities.
use jai_types::{IntegerType, RecordId, RecordLayout};

pub(super) fn integer_type_key(ty: IntegerType) -> [u8; 2] {
    [u8::from(ty.signed()), ty.bits() as u8]
}

pub(super) fn record_layout_key(
    owner: RecordId,
    fields: usize,
    layout: &RecordLayout,
) -> Result<Vec<u8>, jai_vm::Error> {
    if (!layout.field_alignments.is_empty() && layout.field_alignments.len() != fields)
        || (!layout.field_placements.is_empty() && layout.field_placements.len() != fields)
    {
        return Err(jai_vm::Error::InvalidIr(
            "replay layout field count mismatch",
        ));
    }
    let mut key = vec![u8::from(layout.packed)];
    match layout.minimum_alignment {
        Some(alignment) => {
            key.push(1);
            key.extend_from_slice(&alignment.to_le_bytes());
        }
        None => key.push(0),
    }
    key.extend_from_slice(&(fields as u64).to_le_bytes());
    for index in 0..fields {
        match layout.field_alignments.get(index).copied().flatten() {
            Some(alignment) => {
                key.push(1);
                key.extend_from_slice(&alignment.to_le_bytes());
            }
            None => key.push(0),
        }
        match layout.field_placements.get(index).copied().flatten() {
            Some(anchor) => {
                if anchor.record() != owner || anchor.index() >= index {
                    return Err(jai_vm::Error::InvalidIr(
                        "replay placement anchor belongs to another field layout",
                    ));
                }
                key.push(1);
                key.extend_from_slice(&(anchor.index() as u64).to_le_bytes());
            }
            None => key.push(0),
        }
    }
    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_types::{RecordKind, ScalarType, TypeKind, TypeRegistry};

    fn layout(skew: bool) -> (RecordId, RecordLayout) {
        let mut types = TypeRegistry::new();
        let integer = types.scalar(ScalarType::Int(IntegerType::S64));
        if skew {
            let unused = types.reserve_record(RecordKind::Struct);
            types.define_record(unused, vec![integer]).unwrap();
        }
        let owner = types.reserve_record(RecordKind::Struct);
        types.define_record(owner, vec![integer, integer]).unwrap();
        let TypeKind::Record(id) = *types.kind(owner).unwrap() else {
            unreachable!()
        };
        (
            id,
            RecordLayout {
                field_placements: vec![None, Some(types.field(owner, 0).unwrap().id)].into(),
                ..RecordLayout::default()
            },
        )
    }

    #[test]
    fn placement_keys_ignore_record_allocation_order() {
        let (first, first_layout) = layout(false);
        let (second, second_layout) = layout(true);
        assert_ne!(first, second);
        assert_eq!(
            record_layout_key(first, 2, &first_layout).unwrap(),
            record_layout_key(second, 2, &second_layout).unwrap()
        );
        assert!(record_layout_key(first, 2, &second_layout).is_err());
    }

    #[test]
    fn layout_keys_keep_storage_constraints_and_normalize_natural_fields() {
        let (owner, mut layout) = layout(false);
        let baseline = record_layout_key(owner, 2, &layout).unwrap();
        layout.field_alignments = vec![None, None].into();
        assert_eq!(baseline, record_layout_key(owner, 2, &layout).unwrap());
        layout.field_alignments = vec![Some(16), None].into();
        assert_ne!(baseline, record_layout_key(owner, 2, &layout).unwrap());
        layout.field_alignments = Box::new([]);
        layout.minimum_alignment = Some(32);
        assert_ne!(baseline, record_layout_key(owner, 2, &layout).unwrap());
        layout.minimum_alignment = None;
        layout.packed = true;
        assert_ne!(baseline, record_layout_key(owner, 2, &layout).unwrap());
    }
}
