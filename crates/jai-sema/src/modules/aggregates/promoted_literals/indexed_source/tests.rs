use super::*;
use jai_source::Symbols;
use jai_types::{IntegerType, RecordKind, ScalarType, TypeRegistry};

fn expression(kind: ExpressionKind) -> Expression {
    Expression {
        kind,
        span: Span::default(),
    }
}
fn index(base: Expression, value: i128) -> Expression {
    expression(ExpressionKind::Index {
        base: Box::new(base),
        index: Box::new(expression(ExpressionKind::Integer(value))),
    })
}
fn ready(expression: &Expression) -> Result<Option<Integer>, Diagnostic> {
    Ok(match expression.kind {
        ExpressionKind::Integer(value) => Integer::checked(IntegerType::S64, value),
        _ => None,
    })
}

#[test]
fn nested_array_member_indexes_keep_actual_owners_and_written_order() {
    let mut types = TypeRegistry::new();
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let values = types.fixed_array(int, 2).unwrap();
    let row = types.reserve_record(RecordKind::Struct);
    types.define_record(row, [values]).unwrap();
    let rows = types.fixed_array(row, 3).unwrap();
    let root = types.reserve_record(RecordKind::Struct);
    types.define_record(root, [rows]).unwrap();
    let first = types.field(root, 0).unwrap().id;
    let second = types.field(row, 0).unwrap().id;
    let mut symbols = Symbols::default();
    let rows_name = symbols.intern("rows");
    let values_name = symbols.intern("values");
    let source = PlaceSyntax::try_from(index(
        expression(ExpressionKind::Member {
            base: Box::new(index(expression(ExpressionKind::Name(rows_name)), 1)),
            member: values_name,
        }),
        0,
    ))
    .unwrap();
    let mut seen = Vec::new();
    let path = resolve(
        root,
        &source,
        &types,
        |owner, name, _| {
            assert!(owner == root && name == rows_name || owner == row && name == values_name);
            Ok(vec![if owner == root { first } else { second }])
        },
        |expression| {
            let value = ready(expression)?;
            seen.push(value.unwrap().value());
            Ok(value)
        },
    )
    .unwrap();
    assert_eq!(seen, [1, 0]);
    assert_eq!(
        path,
        [
            PathStep::Field(first),
            PathStep::Element {
                owner: rows,
                index: 1
            },
            PathStep::Field(second),
            PathStep::Element {
                owner: values,
                index: 0
            }
        ]
    );
}

#[test]
fn bad_field_owner_rejects_before_querying_index_facts() {
    let mut types = TypeRegistry::new();
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let array = types.fixed_array(int, 2).unwrap();
    let root = types.reserve_record(RecordKind::Struct);
    let foreign = types.reserve_record(RecordKind::Struct);
    types.define_record(root, [array]).unwrap();
    types.define_record(foreign, [array]).unwrap();
    let wrong = types.field(foreign, 0).unwrap().id;
    let name = Symbols::default().intern("values");
    let source = PlaceSyntax::try_from(index(expression(ExpressionKind::Name(name)), 0)).unwrap();
    assert!(
        resolve(
            root,
            &source,
            &types,
            |_, _, _| Ok(vec![wrong]),
            |_| panic!("invalid owner must not query an index")
        )
        .is_err()
    );
}

#[test]
fn negative_equal_count_and_large_unsigned_indices_cannot_be_truncated() {
    let mut types = TypeRegistry::new();
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let array = types.fixed_array(int, 2).unwrap();
    let root = types.reserve_record(RecordKind::Struct);
    types.define_record(root, [array]).unwrap();
    let field = types.field(root, 0).unwrap().id;
    let name = Symbols::default().intern("values");
    for value in [-1, 2, i128::from(u64::MAX)] {
        let source =
            PlaceSyntax::try_from(index(expression(ExpressionKind::Name(name)), value)).unwrap();
        let result = resolve(
            root,
            &source,
            &types,
            |_, _, _| Ok(vec![field]),
            |_| {
                let ty = if value < 0 {
                    IntegerType::S64
                } else {
                    IntegerType::U64
                };
                Ok(Integer::checked(ty, value))
            },
        );
        assert!(result.is_err(), "{value}");
    }
}

#[test]
fn slice_storage_and_runtime_indices_do_not_become_constructor_paths() {
    let mut types = TypeRegistry::new();
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let slice = types.slice(int).unwrap();
    let array = types.fixed_array(int, 2).unwrap();
    let root = types.reserve_record(RecordKind::Struct);
    types.define_record(root, [slice, array]).unwrap();
    let slice_field = types.field(root, 0).unwrap().id;
    let array_field = types.field(root, 1).unwrap().id;
    let name = Symbols::default().intern("values");
    let source = PlaceSyntax::try_from(index(expression(ExpressionKind::Name(name)), 0)).unwrap();
    assert!(
        resolve(
            root,
            &source,
            &types,
            |_, _, _| Ok(vec![slice_field]),
            |_| panic!("slice target must not query an index")
        )
        .is_err()
    );
    assert!(
        resolve(
            root,
            &source,
            &types,
            |_, _, _| Ok(vec![array_field]),
            |_| Ok(None)
        )
        .is_err()
    );
}

#[test]
fn oversized_qualified_path_is_rejected_before_metadata_or_constant_queries() {
    let types = TypeRegistry::new();
    let name = Symbols::default().intern("field");
    let source = PlaceSyntax {
        kind: PlaceKind::Qualified(jai_syntax::NamePath {
            root: name,
            members: vec![name; crate::constant_limits::MAX_CONSTANT_DEPTH],
        }),
        span: Span::default(),
    };
    assert!(
        resolve(
            types.void(),
            &source,
            &types,
            |_, _, _| panic!("path budget must precede metadata"),
            |_| panic!("path budget must precede index lookup")
        )
        .is_err()
    );
}
