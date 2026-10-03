use super::*;
use jai_types::TypeRegistry;

#[test]
fn lazily_cached_images_share_the_memory_budget_and_release_their_charge() {
    let mut types = TypeRegistry::new();
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let pointer_type = types.pointer(byte).unwrap();
    let mut memory = Memory::new(Limits::default());
    for ty in [pointer_type, word] {
        memory.prepare_layout(&types, ty, usize::MAX).1.unwrap();
    }
    let layout_cells = memory.value_cells();
    memory.limits.value_cells = layout_cells + 20;
    let roots = (0..3)
        .map(|_| {
            memory
                .allocate(
                    &types,
                    pointer_type,
                    Some(Value::Pointer(Pointer::null(byte))),
                )
                .unwrap()
        })
        .collect::<Vec<_>>();
    let views = roots
        .iter()
        .map(|root| {
            memory
                .cast_pointer(&types, root, word, CastMode::Checked)
                .unwrap()
        })
        .collect::<Vec<_>>();
    let before = memory.snapshot();
    let baseline = memory.value_cells();
    assert_eq!(baseline, layout_cells + 3);
    memory.load(&types, &views[0]).unwrap();
    memory.load(&types, &views[1]).unwrap();
    assert_eq!(memory.value_cells(), baseline + 14);
    assert_eq!(
        memory.load(&types, &views[2]),
        Err(Error::Limit(LimitKind::ValueCells))
    );
    assert_eq!(memory.value_cells(), baseline + 14);
    assert!(
        memory
            .allocation(&roots[2])
            .unwrap()
            .image
            .borrow()
            .is_none()
    );
    memory.release(&roots[0]).unwrap();
    memory.load(&types, &views[2]).unwrap();
    assert_eq!(memory.value_cells(), layout_cells + 16);
    memory.restore(before);
    assert_eq!(memory.value_cells(), baseline);
    assert!(
        memory
            .allocations
            .values()
            .all(|allocation| allocation.image.borrow().is_none())
    );
}

#[test]
fn opaque_relocation_metadata_is_part_of_the_cumulative_image_charge() {
    let mut types = TypeRegistry::new();
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let pointer_type = types.pointer(byte).unwrap();
    let mut memory = Memory::new(Limits::default());
    for ty in [byte, pointer_type, word] {
        memory.prepare_layout(&types, ty, usize::MAX).1.unwrap();
    }
    let layout_cells = memory.value_cells();
    memory.limits.value_cells = layout_cells + 10;
    let payload = memory
        .allocate(
            &types,
            byte,
            Some(Value::Int(Integer::wrapping(IntegerType::U8, 42))),
        )
        .unwrap();
    let slot = memory
        .allocate(&types, pointer_type, Some(Value::Pointer(payload)))
        .unwrap();
    let view = memory
        .cast_pointer(&types, &slot, word, CastMode::Checked)
        .unwrap();
    let baseline = memory.value_cells();
    assert_eq!(baseline, layout_cells + 2);
    // Eight bytes + one relocation + one origin plus the payload's cell = eleven.
    // Prepared layout facts have their own retained charge above this storage cap.
    assert_eq!(
        memory.load(&types, &view),
        Err(Error::Limit(LimitKind::ValueCells))
    );
    assert_eq!(memory.value_cells(), baseline);
    assert!(memory.allocation(&slot).unwrap().image.borrow().is_none());
}
