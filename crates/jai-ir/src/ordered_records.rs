//! Ordered, nominal record paths shared by verifier and execution consumers.
use jai_types::{FieldId, RecordKind, TypeError, TypeId, TypeView};

/// Backing state before the first ordered write. Uninitialized storage retains
/// the source `=---` contract, including unread padding and omitted fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrderedRecordBacking {
    Zeroed,
    Uninitialized,
}

/// Validate every actual owner along a source field path. Intermediate unions
/// have no ordered active-member contract and cannot be traversed here.
/// A path never crosses a pointer, array, distinct type, or printed-name alias.
pub fn ordered_record_path_type(
    types: &dyn TypeView,
    root: TypeId,
    path: &[FieldId],
) -> Result<TypeId, TypeError> {
    if path.is_empty() || path.len() > 128 {
        return Err(TypeError::WrongKind(root));
    }
    let mut owner = root;
    for &field in path {
        if types.record_storage_definition(owner)?.kind != RecordKind::Struct {
            return Err(TypeError::WrongKind(owner));
        }
        owner = types.validate_field(owner, field)?;
    }
    Ok(owner)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{IntExpr, ValueExpr, verify_expression};
    use jai_types::{Integer, IntegerType, ScalarType, TypeRegistry};
    use std::collections::HashMap;

    #[test]
    fn nominal_paths_accept_repeated_writes_and_reject_cross_owner_fields() {
        let mut types = TypeRegistry::new();
        let word = types.scalar(ScalarType::Int(IntegerType::U32));
        let inner = types.reserve_record(RecordKind::Struct);
        types.define_record(inner, [word]).unwrap();
        let other = types.reserve_record(RecordKind::Struct);
        types.define_record(other, [word]).unwrap();
        let outer = types.reserve_record(RecordKind::Struct);
        types.define_record(outer, [inner]).unwrap();
        let parent = types.field(outer, 0).unwrap().id;
        let child = types.field(inner, 0).unwrap().id;
        let foreign = types.field(other, 0).unwrap().id;
        let types = types.freeze().unwrap();
        assert_eq!(
            ordered_record_path_type(&types, outer, &[parent, child]).unwrap(),
            word
        );
        assert!(matches!(
            ordered_record_path_type(&types, outer, &[parent, foreign]),
            Err(TypeError::FieldOwner { .. })
        ));
        assert!(ordered_record_path_type(&types, outer, &[]).is_err());
        let expression = ValueExpr::OrderedRecord {
            ty: outer,
            backing: OrderedRecordBacking::Zeroed,
            initializers: [1, 2, 42]
                .into_iter()
                .map(|value| {
                    (
                        Box::from([parent, child]),
                        ValueExpr::Int(IntExpr::constant(
                            Integer::checked(IntegerType::U32, value).unwrap(),
                        )),
                    )
                })
                .collect(),
        };
        assert!(
            verify_expression(
                &types,
                &expression,
                &HashMap::new(),
                &[],
                &crate::Places::default()
            )
            .is_ok()
        );
    }

    #[test]
    fn intermediate_union_and_pointer_paths_are_rejected() {
        let mut types = TypeRegistry::new();
        let word = types.scalar(ScalarType::Int(IntegerType::U32));
        let inner = types.reserve_record(RecordKind::Union);
        types.define_record(inner, [word]).unwrap();
        let pointer = types.pointer(inner).unwrap();
        let outer = types.reserve_record(RecordKind::Struct);
        types.define_record(outer, [inner, pointer]).unwrap();
        let union = types.field(outer, 0).unwrap().id;
        let pointer = types.field(outer, 1).unwrap().id;
        let child = types.field(inner, 0).unwrap().id;
        let types = types.freeze().unwrap();
        assert!(ordered_record_path_type(&types, outer, &[union, child]).is_err());
        assert!(ordered_record_path_type(&types, outer, &[pointer, child]).is_err());
        assert_eq!(
            ordered_record_path_type(&types, outer, &[union]).unwrap(),
            inner
        );
    }
}
