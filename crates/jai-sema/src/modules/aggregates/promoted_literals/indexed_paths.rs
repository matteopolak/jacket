//! Canonical physical paths prove aliasing and active union alternatives.
use jai_types::{FieldId, RecordKind, TypeError, TypeId, TypeKind, TypeView};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum PathStep {
    Field(FieldId),
    Element { owner: TypeId, index: u64 },
}

#[derive(Debug)]
pub(crate) enum PathError {
    Empty(usize),
    ArrayOwner {
        index: usize,
        expected: TypeId,
        actual: TypeId,
    },
    NotFixedArray(usize),
    ElementBounds {
        index: usize,
        element: u64,
        count: u64,
    },
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
    children: HashMap<PathStep, Prefix>,
}

pub(super) fn check_paths(
    types: &dyn TypeView,
    root: TypeId,
    initializers: &[(&[FieldId], TypeId)],
) -> Result<(), PathError> {
    check_budget(initializers.iter().map(|(path, _)| path.len()))?;
    let paths = initializers
        .iter()
        .map(|(path, ty)| {
            (
                path.iter()
                    .copied()
                    .map(PathStep::Field)
                    .collect::<Vec<_>>(),
                *ty,
            )
        })
        .collect::<Vec<_>>();
    check_steps(
        types,
        root,
        &paths
            .iter()
            .map(|(path, ty)| (path.as_slice(), *ty))
            .collect::<Vec<_>>(),
    )
}

pub(super) fn check_budget(lengths: impl Iterator<Item = usize>) -> Result<(), PathError> {
    let mut cells = 0usize;
    for (index, length) in lengths.enumerate() {
        if length == 0 {
            return Err(PathError::Empty(index));
        }
        cells = cells.saturating_add(length);
        if length > 128 || cells > 65_536 {
            return Err(PathError::Budget(index));
        }
    }
    Ok(())
}

pub(crate) fn check_steps(
    types: &dyn TypeView,
    root: TypeId,
    initializers: &[(&[PathStep], TypeId)],
) -> Result<(), PathError> {
    check_budget(initializers.iter().map(|(path, _)| path.len()))?;
    let mut prefix = Prefix::default();
    for (index, &(path, leaf)) in initializers.iter().enumerate() {
        let mut ty = root;
        let mut node = &mut prefix;
        for &step in path {
            if node.initialized {
                return Err(PathError::Ancestor(index));
            }
            let child = match step {
                PathStep::Field(field) => {
                    let child = types
                        .validate_field(ty, field)
                        .map_err(|error| PathError::Canonical { index, error })?;
                    if types
                        .record_storage_definition(ty)
                        .map_err(|error| PathError::Canonical { index, error })?
                        .kind
                        == RecordKind::Union
                        && let Some(previous) = node.union_choice.replace(field)
                        && previous != field
                    {
                        return Err(PathError::CompetingUnion(index));
                    }
                    child
                }
                PathStep::Element {
                    owner,
                    index: element,
                } => {
                    if owner != ty {
                        return Err(PathError::ArrayOwner {
                            index,
                            expected: ty,
                            actual: owner,
                        });
                    }
                    let TypeKind::FixedArray {
                        element: child,
                        count,
                    } = *types
                        .kind(ty)
                        .map_err(|error| PathError::Canonical { index, error })?
                    else {
                        return Err(PathError::NotFixedArray(index));
                    };
                    if element >= count {
                        return Err(PathError::ElementBounds {
                            index,
                            element,
                            count,
                        });
                    }
                    child
                }
            };
            node = node.children.entry(step).or_default();
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
    #[test]
    fn array_projection_validates_owner_bounds_and_whole_array_overlap() {
        let mut types = TypeRegistry::new();
        let int = types.scalar(ScalarType::Int(IntegerType::S64));
        let array = types.fixed_array(int, 2).unwrap();
        let foreign = types.fixed_array(int, 3).unwrap();
        let root = types.reserve_record(RecordKind::Struct);
        types.define_record(root, vec![array]).unwrap();
        let field = PathStep::Field(types.field(root, 0).unwrap().id);
        let leaf = [
            field,
            PathStep::Element {
                owner: array,
                index: 1,
            },
        ];
        assert!(check_steps(&types, root, &[(&leaf, int)]).is_ok());
        let invalid_owner = [
            field,
            PathStep::Element {
                owner: foreign,
                index: 1,
            },
        ];
        assert!(matches!(
            check_steps(&types, root, &[(&invalid_owner, int)]),
            Err(PathError::ArrayOwner { index: 0, .. })
        ));
        let invalid_index = [
            field,
            PathStep::Element {
                owner: array,
                index: 2,
            },
        ];
        assert!(matches!(
            check_steps(&types, root, &[(&invalid_index, int)]),
            Err(PathError::ElementBounds { index: 0, .. })
        ));
        assert!(matches!(
            check_steps(&types, root, &[(&[field], array), (&leaf, int)]),
            Err(PathError::Ancestor(1))
        ));
        assert!(matches!(
            check_steps(&types, root, &[(&leaf, int), (&[field], array)]),
            Err(PathError::Ancestor(1))
        ));
    }

    #[test]
    fn union_choices_are_scoped_to_actual_array_element_prefixes() {
        let mut types = TypeRegistry::new();
        let int = types.scalar(ScalarType::Int(IntegerType::S64));
        let boolean = types.scalar(ScalarType::Bool);
        let union = types.reserve_record(RecordKind::Union);
        types.define_record(union, vec![int, boolean]).unwrap();
        let array = types.fixed_array(union, 2).unwrap();
        let first = PathStep::Element {
            owner: array,
            index: 0,
        };
        let second = PathStep::Element {
            owner: array,
            index: 1,
        };
        let a = PathStep::Field(types.field(union, 0).unwrap().id);
        let b = PathStep::Field(types.field(union, 1).unwrap().id);
        assert!(
            check_steps(
                &types,
                array,
                &[(&[first, a], int), (&[second, b], boolean)]
            )
            .is_ok()
        );
        assert!(matches!(
            check_steps(&types, array, &[(&[first, a], int), (&[first, b], boolean)]),
            Err(PathError::CompetingUnion(1))
        ));
    }
}
