use super::*;
use jai_types::TypeKind;

fn address(f: &Fixture, place: Place, pointer_type: TypeId) -> IntExpr {
    assert_eq!(
        f.types.kind(pointer_type).unwrap(),
        &TypeKind::Pointer(place.ty())
    );
    IntExpr::new(
        IntegerType::S64,
        IntExprKind::FromPointer {
            value: Box::new(ValueExpr::AddressOf {
                place,
                ty: pointer_type,
            }),
            mode: jai_types::CastMode::Checked,
        },
    )
}
#[test]
fn address_result_publication_fails_inside_the_transaction_and_restores_globals() {
    let mut f = fixture();
    let scalar = f.types.scalar(ScalarType::Int(IntegerType::S64));
    let pointer = f.types.pointer(scalar).unwrap();
    let global = Global::new(
        0,
        GlobalInitializer::Int(Integer::wrapping(IntegerType::S64, 0)),
        &f.types,
    );
    let place = global.place();
    let typed = IntPlace::try_from_place(place, &f.types).unwrap();
    f.globals.push(global);
    let expression = address(&f, place, pointer);
    let body = procedure(
        &mut f,
        0,
        vec![],
        vec![],
        vec![Statement::StoreInt(typed, int(5)), ret(expression)],
    );
    f.procedures.push(body);
    let mut vm = Vm::new(&f, NoEffects, Limits::default()).unwrap();
    let outcome = vm
        .evaluate_validated(&ValueExpr::Int(call_int(0, vec![])), |vm, values| {
            vm.materialize_value(&values[0]).map(|_| ())
        })
        .outcome;
    assert_eq!(
        outcome,
        Outcome::Failed(Error::UnsupportedPointerOperation(
            "address-derived integer cannot be published as a native constant"
        ))
    );
    assert_eq!(vm.memory().allocation_count(), 0);
    assert_eq!(
        vm.evaluate(&ValueExpr::Load(place)).outcome,
        Outcome::Complete(vec![value(0)])
    );
}
#[test]
fn affine_offsets_and_same_allocation_differences_are_proven_without_numeric_pattern_guesses() {
    let mut f = fixture();
    let scalar = f.types.scalar(ScalarType::Int(IntegerType::S64));
    let byte = f.types.scalar(ScalarType::Int(IntegerType::U8));
    let pointer = f.types.pointer(scalar).unwrap();
    let global = Global::new(
        0,
        GlobalInitializer::Int(Integer::wrapping(IntegerType::S64, 0x1122334455667788)),
        &f.types,
    );
    let place = global.place();
    f.globals.push(global);
    let root = address(&f, place, pointer);
    let next = binary(IntOp::Add, root.clone(), int(1));
    let difference = binary(IntOp::Subtract, next.clone(), root);
    let mut vm = Vm::new(&f, NoEffects, Limits::default()).unwrap();
    assert_eq!(
        vm.evaluate(&ValueExpr::Int(difference)).outcome,
        Outcome::Complete(vec![value(1)])
    );
    let Outcome::Complete(values) = vm.evaluate(&ValueExpr::Int(next)).outcome else {
        panic!("address evaluation failed")
    };
    let number = values[0].number().unwrap();
    assert!(matches!(
        number.provenance(),
        Some(AddressProvenance::Pointer(_))
    ));
    let recovered = vm
        .memory()
        .integer_to_pointer(&f.types, number.clone(), byte, jai_types::CastMode::Checked)
        .unwrap();
    assert_eq!(
        vm.memory().load(&f.types, &recovered).unwrap(),
        Value::Int(Integer::wrapping(IntegerType::U8, 0x77))
    );
    let ordinary = ValueExpr::Int(IntExpr::constant(number.integer()));
    assert_eq!(
        vm.evaluate_validated(&ordinary, |vm, values| vm
            .materialize_value(&values[0])
            .map(|_| ()))
            .outcome,
        Outcome::Complete(vec![Value::Int(number.integer())])
    );
}
#[test]
fn unproven_address_bits_cannot_choose_an_array_element_or_become_a_null_pointer() {
    let mut f = fixture();
    let scalar = f.types.scalar(ScalarType::Int(IntegerType::S64));
    let pointer = f.types.pointer(scalar).unwrap();
    let array = f.types.fixed_array(scalar, 3).unwrap();
    let global = Global::new(
        0,
        GlobalInitializer::Int(Integer::wrapping(IntegerType::S64, 42)),
        &f.types,
    );
    let place = global.place();
    f.globals.push(global);
    let root = address(&f, place, pointer);
    let index = binary(IntOp::Remainder, root.clone(), int(3));
    let expression = ValueExpr::Index {
        base: Box::new(ValueExpr::Array {
            ty: array,
            elements: vec![ValueExpr::Int(int(42)); 3],
        }),
        index,
        ty: scalar,
        check: CheckMode::Enabled,
    };
    let mut vm = Vm::new(&f, NoEffects, Limits::default()).unwrap();
    assert!(matches!(
        vm.evaluate(&expression).outcome,
        Outcome::Failed(Error::UnsupportedPointerOperation(_))
    ));
    let shifted = binary(IntOp::ShiftRight, root, int(63));
    let expression = ValueExpr::PointerFromInteger {
        value: shifted,
        ty: pointer,
        mode: jai_types::CastMode::Unchecked,
    };
    assert_eq!(
        vm.evaluate(&expression).outcome,
        Outcome::Failed(Error::UnsupportedPointerOperation(
            "transformed address integer has no proven affine pointer identity"
        ))
    );
}
