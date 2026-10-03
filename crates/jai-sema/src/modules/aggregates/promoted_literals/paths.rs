//! Canonical physical paths prove aliasing and active union alternatives.
use jai_types::{FieldId, RecordKind, TypeError, TypeId, TypeView};
use std::collections::HashMap;

#[derive(Debug)]
pub(crate) enum PathError {
    Empty(usize),
    Budget(usize),
    Canonical {
        index: usize,
        error: TypeError,
    },
    LeafType {
        index: usize,
        expected: TypeId,
        actual: TypeId,
    },
    Duplicate(usize),
    Ancestor(usize),
    CompetingUnion(usize),
}

#[derive(Default)]
struct Prefix {
    initialized: bool,
    union_choice: Option<FieldId>,
    children: HashMap<FieldId, Prefix>,
}

pub(super) fn check_paths(
    types: &dyn TypeView,
    root: TypeId,
    initializers: &[(&[FieldId], TypeId)],
) -> Result<(), PathError> {
    let mut prefix = Prefix::default();
    let mut cells = 0usize;
    for (index, &(path, leaf)) in initializers.iter().enumerate() {
        if path.is_empty() {
            return Err(PathError::Empty(index));
        }
        cells = cells.saturating_add(path.len());
        if path.len() > 128 || cells > 65_536 {
            return Err(PathError::Budget(index));
        }
        let mut ty = root;
        let mut node = &mut prefix;
        for &field in path {
            if node.initialized {
                return Err(PathError::Ancestor(index));
            }
            let child = types
                .validate_field(ty, field)
                .map_err(|error| PathError::Canonical {
                    index,
                    error,
                })?;
            if types
                .record_storage_definition(ty)
                .map_err(|error| PathError::Canonical {
                    index,
                    error,
                })?
                .kind
                == RecordKind::Union
                && let Some(previous) = node.union_choice.replace(field)
                && previous != field
            {
                return Err(PathError::CompetingUnion(index));
            }
            node = node.children.entry(field).or_default();
            ty = child;
        }
        if ty != leaf {
            return Err(PathError::LeafType {
                index,
                expected: ty,
                actual: leaf,
            });
        }
        if node.initialized {
            return Err(PathError::Duplicate(index));
        }
        if !node.children.is_empty() {
            return Err(PathError::Ancestor(index));
        }
        node.initialized = true;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_types::{IntegerType, ScalarType, TypeRegistry};

    #[test]
    fn actual_field_owner_and_leaf_type_are_required() {
        let mut types = TypeRegistry::new();
        let int = types.scalar(ScalarType::Int(IntegerType::S64));
        let boolean = types.scalar(ScalarType::Bool);
        let owner = types.reserve_record(RecordKind::Struct);
        let foreign = types.reserve_record(RecordKind::Struct);
        types.define_record(owner, vec![int]).unwrap();
        types.define_record(foreign, vec![int]).unwrap();
        let correct = [types.field(owner, 0).unwrap().id];
        let wrong = [types.field(foreign, 0).unwrap().id];
        assert!(check_paths(&types, owner, &[(&correct, int)]).is_ok());
        assert!(matches!(
            check_paths(&types, owner, &[(&wrong, int)]),
            Err(PathError::Canonical { .. })
        ));
        assert!(matches!(
            check_paths(&types, owner, &[(&correct, boolean)]),
            Err(PathError::LeafType { .. })
        ));
        assert!(matches!(
            check_paths(&types, owner, &[(&[], int)]),
            Err(PathError::Empty(0))
        ));
    }

    #[test]
    fn physical_prefix_distinguishes_same_typed_union_instances() {
        let mut types = TypeRegistry::new();
        let int = types.scalar(ScalarType::Int(IntegerType::S64));
        let boolean = types.scalar(ScalarType::Bool);
        let union = types.reserve_record(RecordKind::Union);
        types.define_record(union, vec![int, boolean]).unwrap();
        let owner = types.reserve_record(RecordKind::Struct);
        types.define_record(owner, vec![union, union]).unwrap();
        let first = types.field(owner, 0).unwrap().id;
        let second = types.field(owner, 1).unwrap().id;
        let a = types.field(union, 0).unwrap().id;
        let b = types.field(union, 1).unwrap().id;
        let one = [first, a];
        let two = [second, b];
        let competing = [first, b];
        let ancestor = [first];
        assert!(check_paths(&types, owner, &[(&one, int), (&two, boolean)]).is_ok());
        assert!(matches!(
            check_paths(&types, owner, &[(&one, int), (&competing, boolean)]),
            Err(PathError::CompetingUnion(1))
        ));
        assert!(matches!(
            check_paths(&types, owner, &[(&one, int), (&one, int)]),
            Err(PathError::Duplicate(1))
        ));
        assert!(matches!(
            check_paths(&types, owner, &[(&ancestor, union), (&one, int)]),
            Err(PathError::Ancestor(1))
        ));
    }

    #[test]
    fn large_distinct_paths_are_bounded_by_cells_without_pairwise_comparisons() {
        let mut types = TypeRegistry::new();
        let int = types.scalar(ScalarType::Int(IntegerType::S64));
        let owner = types.reserve_record(RecordKind::Struct);
        types.define_record(owner, vec![int; 65_537]).unwrap();
        let paths: Vec<_> = (0..65_537)
            .map(|index| [types.field(owner, index).unwrap().id])
            .collect();
        let initializers: Vec<_> = paths.iter().map(|path| (path.as_slice(), int)).collect();
        assert!(check_paths(&types, owner, &initializers[..65_536]).is_ok());
        assert!(matches!(
            check_paths(&types, owner, &initializers),
            Err(PathError::Budget(65_536))
        ));
    }
}
