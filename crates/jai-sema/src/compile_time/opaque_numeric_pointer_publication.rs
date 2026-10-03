use super::*;
use jai_types::{CastMode, Integer, IntegerType, LayoutPolicy, ScalarLayout, TypeRegistry};
fn target(bytes: u64) -> jai_vm::ByteTarget {
    jai_vm::ByteTarget {
        policy: LayoutPolicy::new(
            ScalarLayout::new(bytes, bytes as u32),
            [
                ScalarLayout::new(1, 1),
                ScalarLayout::new(2, 2),
                ScalarLayout::new(4, 4),
                ScalarLayout::new(8, 4),
            ],
            [ScalarLayout::new(4, 4), ScalarLayout::new(8, 4)],
            ScalarLayout::new(1, 1),
        )
        .unwrap(),
        endian: jai_vm::Endian::Little,
    }
}
#[test]
fn numeric_run_result_is_a_real_native_pointer_recipe_in_the_selected_source_width() {
    let mut types = TypeRegistry::new();
    let ty = types.pointer(types.void()).unwrap();
    for bytes in [4, 8] {
        let memory = jai_vm::Memory::with_target(Limits::default(), target(bytes));
        for bits in [1, 0x80000001] {
            let pointer = memory
                .integer_to_pointer(
                    &types,
                    jai_vm::Number::plain(Integer::wrapping(IntegerType::U64, bits)),
                    types.void(),
                    CastMode::Checked,
                )
                .unwrap();
            let value = Value::Pointer(pointer);
            assert_eq!(scalar_type(&types, &value), Some(ty));
            materializable_type(&types, ty, 256).unwrap();
            let constant = materialize(&types, value, ty, 256).unwrap();
            let ConstantKind::NativePointer(recipe) = constant.kind else {
                panic!("numeric pointer recipe")
            };
            let jai_ir::NativePointerSource::Strong(source) = recipe.source() else {
                panic!("captured selected bits are strong")
            };
            assert_eq!(
                source.ty(),
                if bytes == 4 {
                    IntegerType::U32
                } else {
                    IntegerType::U64
                }
            );
            assert_eq!(source.bits(), bits as u64);
            assert_eq!(
                recipe.address(bytes as u32 * 8).unwrap().bits(),
                bits as u64
            );
        }
        let null = Value::Pointer(jai_vm::Pointer::null(types.void()));
        assert_eq!(scalar_type(&types, &null), Some(ty));
        assert_eq!(
            materialize(&types, null, ty, 256).unwrap().kind,
            ConstantKind::Zero
        );
    }
}
#[test]
fn an_actual_allocation_cannot_use_the_numeric_run_result_leaf() {
    let mut types = TypeRegistry::new();
    let byte = types.scalar(jai_types::ScalarType::Int(IntegerType::U8));
    let ty = types.pointer(byte).unwrap();
    let mut memory = jai_vm::Memory::new(Limits::default());
    let root = memory
        .allocate(
            &types,
            byte,
            Some(Value::Int(Integer::wrapping(IntegerType::U8, 9))),
        )
        .unwrap();
    assert_eq!(
        materialize(&types, Value::Pointer(root.clone()), ty, 256),
        Err(jai_vm::Error::UnsupportedType(ty))
    );
    assert_eq!(
        memory.load(&types, &root).unwrap(),
        Value::Int(Integer::wrapping(IntegerType::U8, 9))
    );
}
