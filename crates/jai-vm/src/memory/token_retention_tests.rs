use super::*;
use jai_types::{CallingConvention, ContextMode, ProcedureType, TypeRegistry, Variadic};

fn int(bits: u64) -> Value {
    Value::Int(Integer::wrapping(IntegerType::U64, i128::from(bits)))
}
fn image(memory: &Memory, types: &dyn TypeView, ty: TypeId, value: &Value) -> ByteImage {
    let mut image =
        ByteImage::encode(types, memory.target(), ty, value, memory.limits.value_cells).unwrap();
    memory.retokenize_image(types, &mut image).unwrap();
    image
}
fn signature(types: &mut TypeRegistry) -> TypeId {
    types
        .procedure(ProcedureType {
            parameters: Box::new([]),
            results: Box::new([]),
            return_abi: jai_types::ForeignReturnAbi::Natural,
            convention: CallingConvention::Jai,
            context: ContextMode::None,
            variadic: Variadic::None,
        })
        .unwrap()
}

#[test]
fn retired_data_images_do_not_add_ledger_entries_or_retokenization_work() {
    let mut types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let pointer_ty = types.pointer(word).unwrap();
    let procedure_ty = signature(&mut types);
    let mut memory = Memory::new(Limits::default());
    let procedure = Value::Procedure {
        signature: procedure_ty,
        procedure: Some(jai_ir::ProcedureId::new(11)),
    };
    image(&memory, &types, procedure_ty, &procedure);
    let receipt = memory.code_pointer(&types, &procedure).unwrap();
    let entries = memory.handle_tokens.borrow().values.len();
    let work = memory.retokenize_work_cost();
    let mut previous = 0;
    let mut steady_store_work = None;
    let mut steady_intrinsic_work = None;
    for _ in 0..1_024 {
        let data = memory.allocate(&types, word, Some(int(42))).unwrap();
        let address = memory
            .pointer_to_integer(&types, &data, IntegerType::U64, CastMode::Checked)
            .unwrap();
        assert!(address.bits() > previous);
        previous = address.bits();
        let encoded = image(&memory, &types, pointer_ty, &Value::Pointer(data.clone()));
        assert_eq!(encoded.bytes(), &address.bits().to_le_bytes());
        let slot = memory
            .allocate(&types, pointer_ty, Some(Value::Pointer(data.clone())))
            .unwrap();
        memory
            .ensure_image(&types, memory.allocation(&slot).unwrap())
            .unwrap();
        let alias = memory
            .cast_pointer(&types, &slot, word, CastMode::Checked)
            .unwrap();
        let store_work = memory.store_work_cost(&types, &alias).unwrap();
        let intrinsic_work = memory
            .intrinsic_work_cost(&types, &[(&slot, true)], 1)
            .unwrap();
        assert_eq!(*steady_store_work.get_or_insert(store_work), store_work);
        assert_eq!(
            *steady_intrinsic_work.get_or_insert(intrinsic_work),
            intrinsic_work
        );
        memory.release(&slot).unwrap();
        memory.release(&data).unwrap();
        assert_eq!(memory.allocation_count(), 0);
        assert_eq!(memory.handle_tokens.borrow().values.len(), entries);
        assert_eq!(memory.retokenize_work_cost(), work);
        memory.validate_code_pointer(&types, receipt).unwrap();
    }
}

#[test]
fn canonical_data_bytes_match_aliases_but_distinguish_offsets_roots_and_code() {
    let mut types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let pointer_ty = types.pointer(word).unwrap();
    let void_pointer_ty = types.pointer(types.void()).unwrap();
    let array = types.fixed_array(word, 2).unwrap();
    let procedure_ty = signature(&mut types);
    let mut memory = Memory::new(Limits::default());
    let root = memory
        .allocate(
            &types,
            array,
            Some(Value::Array {
                ty: array,
                elements: vec![int(1), int(2)],
            }),
        )
        .unwrap();
    let first = memory.index(&types, &root, 0).unwrap();
    let second = memory.index(&types, &root, 1).unwrap();
    let other = memory.allocate(&types, word, Some(int(3))).unwrap();
    let alias = memory
        .cast_pointer(&types, &first, types.void(), CastMode::Checked)
        .unwrap();
    let first_image = image(&memory, &types, pointer_ty, &Value::Pointer(first));
    let alias_image = image(&memory, &types, void_pointer_ty, &Value::Pointer(alias));
    let second_image = image(&memory, &types, pointer_ty, &Value::Pointer(second));
    let other_image = image(&memory, &types, pointer_ty, &Value::Pointer(other));
    let code_image = image(
        &memory,
        &types,
        procedure_ty,
        &Value::Procedure {
            signature: procedure_ty,
            procedure: Some(jai_ir::ProcedureId::new(12)),
        },
    );
    assert_eq!(first_image.bytes(), alias_image.bytes());
    for distinct in [&second_image, &other_image, &code_image] {
        assert_ne!(first_image.bytes(), distinct.bytes());
    }
    assert_ne!(second_image.bytes(), other_image.bytes());
    assert_eq!(memory.handle_tokens.borrow().values.len(), 1);
}

#[test]
fn released_complete_images_and_address_numbers_stay_dangling_after_new_allocations() {
    let mut types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let pointer_ty = types.pointer(word).unwrap();
    let mut memory = Memory::new(Limits::default());
    let data = memory.allocate(&types, word, Some(int(42))).unwrap();
    let mut old_image = image(&memory, &types, pointer_ty, &Value::Pointer(data.clone()));
    let number = old_image
        .read(&types, memory.target(), 0, word)
        .unwrap()
        .number()
        .unwrap();
    let copied = old_image.extract_range(0, 8).unwrap();
    memory.release(&data).unwrap();
    let replacement = memory.allocate(&types, word, Some(int(77))).unwrap();
    assert_ne!(replacement.allocation_key(), data.allocation_key());
    let replacement_address = memory
        .pointer_to_integer(&types, &replacement, IntegerType::U64, CastMode::Checked)
        .unwrap();
    assert_ne!(replacement_address.bits(), number.bits());
    let stale = copied
        .read(&types, memory.target(), 0, pointer_ty)
        .unwrap()
        .pointer()
        .unwrap()
        .clone();
    assert_eq!(memory.load(&types, &stale), Err(Error::DanglingPointer));
    assert_eq!(
        memory.integer_to_pointer(&types, number, word, CastMode::Checked),
        Err(Error::DanglingPointer)
    );
    let before = old_image.clone();
    assert_eq!(
        memory.retokenize_image(&types, &mut old_image),
        Err(Error::DanglingPointer)
    );
    assert_eq!(old_image, before);
    assert!(memory.handle_tokens.borrow().values.is_empty());
}

#[test]
fn procedure_limits_and_rollback_keep_receipts_without_data_history() {
    let mut types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let pointer_ty = types.pointer(word).unwrap();
    let procedure_ty = signature(&mut types);
    let mut memory = Memory::new(Limits {
        value_cells: 16,
        ..Limits::default()
    });
    let root = memory.allocate(&types, word, Some(int(42))).unwrap();
    image(&memory, &types, pointer_ty, &Value::Pointer(root.clone()));
    let baseline = memory.snapshot();
    let procedure = Value::Procedure {
        signature: procedure_ty,
        procedure: Some(jai_ir::ProcedureId::new(31)),
    };
    image(&memory, &types, procedure_ty, &procedure);
    let receipt = memory.code_pointer(&types, &procedure).unwrap();
    let abandoned_highwater = memory.next_virtual_address.get();
    memory.restore(baseline);
    assert_eq!(
        memory.validate_code_pointer(&types, receipt),
        Err(Error::DanglingPointer)
    );
    assert_eq!(memory.next_virtual_address.get(), abandoned_highwater);
    image(&memory, &types, pointer_ty, &Value::Pointer(root));
    assert!(memory.handle_tokens.borrow().values.is_empty());
    for id in 0..16 {
        image(
            &memory,
            &types,
            procedure_ty,
            &Value::Procedure {
                signature: procedure_ty,
                procedure: Some(jai_ir::ProcedureId::new(id)),
            },
        );
    }
    let value = Value::Procedure {
        signature: procedure_ty,
        procedure: Some(jai_ir::ProcedureId::new(100)),
    };
    let mut denied = ByteImage::encode(&types, memory.target(), procedure_ty, &value, 16).unwrap();
    let before = denied.clone();
    let highwater = memory.next_virtual_address.get();
    assert_eq!(
        memory.retokenize_image(&types, &mut denied),
        Err(Error::Limit(LimitKind::ValueCells))
    );
    assert_eq!(denied, before);
    assert_eq!(memory.handle_tokens.borrow().values.len(), 16);
    assert_eq!(memory.next_virtual_address.get(), highwater);
}

#[test]
fn private_branch_data_addresses_keep_isolation_without_ledger_entries() {
    let mut types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let pointer_ty = types.pointer(word).unwrap();
    let mut parent = Memory::new(Limits::default());
    let root = parent.allocate(&types, word, Some(int(42))).unwrap();
    let parent_image = image(&parent, &types, pointer_ty, &Value::Pointer(root.clone()));
    let mut child = parent
        .fork_private_branch(parent.snapshot_work_cost().unwrap())
        .unwrap();
    let child_image = image(&child, &types, pointer_ty, &Value::Pointer(root.clone()));
    assert_eq!(parent_image.bytes(), child_image.bytes());
    child.store(&types, &root, int(77)).unwrap();
    assert_eq!(parent.load(&types, &root).unwrap(), int(42));
    child.release(&root).unwrap();
    assert_eq!(
        child.retokenize_image(&types, &mut child_image.clone()),
        Err(Error::DanglingPointer)
    );
    assert!(parent.handle_tokens.borrow().values.is_empty());
    assert!(child.handle_tokens.borrow().values.is_empty());
    assert_eq!(parent.retokenize_work_cost(), 0);
    assert_eq!(child.retokenize_work_cost(), 0);
}
