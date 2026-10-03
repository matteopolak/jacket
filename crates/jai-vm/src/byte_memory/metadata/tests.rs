use crate::{AddressProvenance, ByteImage, ByteTarget, Error, LimitKind, Number, Value};
use jai_types::{Integer, IntegerType, ScalarType, TypeRegistry};

fn origin_image(limit: usize) -> ByteImage {
    let types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let number = Number::address(
        Integer::wrapping(IntegerType::U64, 42),
        AddressProvenance::Derived {
            memory: 7,
            allocations: (0..12).collect(),
        },
    );
    ByteImage::encode(
        &types,
        ByteTarget::default(),
        word,
        &number.into_value(),
        limit,
    )
    .unwrap()
}
#[test]
fn batch_origin_budget_is_checked_before_cloning_metadata() {
    let image = origin_image(64);
    assert_eq!(image.metadata_cells(), 12);
    assert!(matches!(
        ByteImage::concatenate(
            &[image.clone(), image.clone(), image.clone()],
            image.target(),
            32
        ),
        Err(Error::Limit(LimitKind::ValueCells))
    ));
    let result =
        ByteImage::concatenate(&[image.clone(), image], ByteTarget::default(), 32).unwrap();
    assert_eq!(result.metadata_cells(), 24);
    assert_eq!(result.len(), 16);
}
#[test]
fn origin_splits_copies_and_typed_stores_fail_atomically_at_metadata_budget() {
    let target = ByteTarget::default();
    let types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let mut image = origin_image(16);
    let before = image.bytes().to_vec();
    assert!(matches!(
        image.write_range(3, &[0]),
        Err(Error::Limit(LimitKind::ValueCells))
    ));
    assert!(matches!(
        image.fill_range(3, 1, 0),
        Err(Error::Limit(LimitKind::ValueCells))
    ));
    assert!(matches!(
        image.write(
            &types,
            target,
            3,
            types.scalar(ScalarType::Int(IntegerType::U8)),
            &Value::Int(Integer::wrapping(IntegerType::U8, 0))
        ),
        Err(Error::Limit(LimitKind::ValueCells))
    ));
    assert_eq!(image.bytes(), before);
    assert_eq!(image.metadata_cells(), 12);
    let mut destination = ByteImage::concatenate(
        &[
            origin_image(64),
            ByteImage::from_bytes(target, vec![0; 8], 64).unwrap(),
        ],
        target,
        16,
    )
    .unwrap();
    let before = destination.bytes().to_vec();
    assert!(matches!(
        destination.copy_range_from(&image, 0, 8, 8),
        Err(Error::Limit(LimitKind::ValueCells))
    ));
    assert!(matches!(
        destination.copy_range_within(0, 8, 8),
        Err(Error::Limit(LimitKind::ValueCells))
    ));
    let number = image
        .read(&types, target, 0, word)
        .unwrap()
        .number()
        .unwrap();
    assert!(matches!(
        destination.fill_range_number(8, 8, number),
        Err(Error::Limit(LimitKind::ValueCells))
    ));
    assert_eq!(destination.bytes(), before);
    assert_eq!(destination.metadata_cells(), 12);
    image.fill_range(0, 8, 0).unwrap();
    assert_eq!(image.metadata_cells(), 0);
}

fn deep_pointer_fixture(
    limit: usize,
) -> (
    TypeRegistry,
    crate::Memory,
    jai_types::TypeId,
    crate::Pointer,
) {
    let mut types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let mut root_ty = word;
    let pointer_ty = types.pointer(word).unwrap();
    let mut demanded_layouts = vec![word, pointer_ty];
    for _ in 0..64 {
        let record = types.reserve_record(jai_types::RecordKind::Struct);
        types.define_record(record, [root_ty]).unwrap();
        root_ty = record;
        demanded_layouts.push(record);
    }
    let preparation = crate::Memory::new(crate::Limits::default());
    for &ty in &demanded_layouts {
        preparation
            .prepare_layout(&types, ty, usize::MAX)
            .1
            .unwrap();
    }
    let mut memory = crate::Memory::new(crate::Limits {
        value_cells: limit + preparation.value_cells(),
        ..crate::Limits::default()
    });
    for ty in demanded_layouts {
        memory.prepare_layout(&types, ty, usize::MAX).1.unwrap();
    }
    let mut pointer = memory.allocate(&types, root_ty, None).unwrap();
    for _ in 0..64 {
        pointer = memory.field(&types, &pointer, 0).unwrap();
    }
    assert_eq!(pointer.metadata_cells(), 64);
    (types, memory, word, pointer)
}

#[test]
fn projected_pointer_paths_count_in_every_value_and_exact_number_origin() {
    let (mut types, memory, word, pointer) = deep_pointer_fixture(4096);
    let slice = types.slice(word).unwrap();
    let dynamic = types.dynamic_array(word).unwrap();
    let address = memory
        .pointer_to_integer(
            &types,
            &pointer,
            IntegerType::U64,
            jai_types::CastMode::Checked,
        )
        .unwrap();
    assert_eq!(address.origin_count(), 1);
    assert_eq!(address.metadata_cells(), 65);
    let header = pointer.pointee();
    // Type and StringView accounting examines retained pointer metadata independently
    // of representation validation; the matching typed fixtures cover validation.
    for value in [
        Value::Pointer(pointer.clone()),
        Value::Slice {
            ty: slice,
            pointer: pointer.clone(),
            count: 0,
        },
        Value::DynamicArray {
            ty: dynamic,
            pointer: pointer.clone(),
            count: 0,
            allocated: 0,
            allocator: None,
        },
        Value::StringView {
            pointer: pointer.clone(),
            count: 0,
        },
        Value::Type {
            descriptor: Some(pointer.clone()),
        },
    ] {
        assert_eq!(value.cells(65).unwrap(), 65);
        assert_eq!(value.cells(64), Err(Error::Limit(LimitKind::ValueCells)));
    }
    assert_eq!(header, word);
    assert_eq!(address.into_value().cells(66).unwrap(), 66);
    let pointer_ty = types.pointer(word).unwrap();
    let array = types.fixed_array(pointer_ty, 16).unwrap();
    let value = Value::Array {
        ty: array,
        elements: vec![Value::Pointer(pointer); 16],
    };
    assert_eq!(value.cells(1041).unwrap(), 1041);
    assert_eq!(value.cells(1024), Err(Error::Limit(LimitKind::ValueCells)));
}

#[test]
fn encoder_and_cumulative_copy_charge_both_pointer_path_clones_before_mutation() {
    let (mut types, _memory, word, pointer) = deep_pointer_fixture(4096);
    let pointer_ty = types.pointer(word).unwrap();
    let array = types.fixed_array(pointer_ty, 8).unwrap();
    let value = Value::Array {
        ty: array,
        elements: vec![Value::Pointer(pointer.clone()); 8],
    };
    assert_eq!(value.cells(1024).unwrap(), 521);
    assert!(matches!(
        ByteImage::encode(&types, ByteTarget::default(), array, &value, 1024),
        Err(Error::Limit(LimitKind::ValueCells))
    ));
    let source = ByteImage::encode(
        &types,
        ByteTarget::default(),
        pointer_ty,
        &Value::Pointer(pointer.clone()),
        512,
    )
    .unwrap();
    assert_eq!(source.metadata_cells(), 130);
    let mut destination = ByteImage::from_bytes(source.target(), vec![0; 32], 512).unwrap();
    for slot in 0..3 {
        destination
            .copy_range_from(&source, 0, slot * 8, 8)
            .unwrap();
    }
    assert_eq!(destination.metadata_cells(), 390);
    let before = destination.bytes().to_vec();
    assert!(matches!(
        destination.copy_range_from(&source, 0, 24, 8),
        Err(Error::Limit(LimitKind::ValueCells))
    ));
    assert_eq!(destination.bytes(), before);
    let partial = source.extract_range(0, 1).unwrap();
    assert_eq!(partial.metadata_cells(), 1);
    destination.copy_range_from(&partial, 0, 24, 1).unwrap();
    assert_eq!(destination.metadata_cells(), 391);
    assert_eq!(
        destination
            .read(&types, source.target(), 0, pointer_ty)
            .unwrap(),
        Value::Pointer(pointer)
    );
}

#[test]
fn cached_images_and_value_allocations_share_the_projected_path_budget() {
    let (mut types, mut memory, word, pointer) = deep_pointer_fixture(512);
    let pointer_ty = types.pointer(word).unwrap();
    let before_roots = memory.value_cells();
    let pointer_cells = crate::Value::Pointer(pointer.clone()).cells(512).unwrap();
    assert_eq!(pointer_cells, 65);
    let roots = (0..4)
        .map(|_| {
            memory
                .allocate(&types, pointer_ty, Some(Value::Pointer(pointer.clone())))
                .unwrap()
        })
        .collect::<Vec<_>>();
    let baseline = memory.value_cells();
    assert_eq!(baseline, before_roots + 4 * pointer_cells);
    let views = roots
        .iter()
        .map(|root| {
            memory
                .cast_pointer(&types, root, word, jai_types::CastMode::Checked)
                .unwrap()
        })
        .collect::<Vec<_>>();
    for view in &views[..3] {
        memory.load(&types, view).unwrap();
    }
    assert_eq!(memory.value_cells(), baseline + 3 * 73);
    assert_eq!(
        memory.load(&types, &views[3]),
        Err(Error::Limit(LimitKind::ValueCells))
    );
    assert_eq!(memory.value_cells(), baseline + 3 * 73);
}

#[test]
fn projected_paths_keep_a_hard_cap_when_the_configured_depth_is_larger() {
    let mut types = TypeRegistry::new();
    let mut ty = types.scalar(ScalarType::Int(IntegerType::U64));
    for _ in 0..257 {
        let parent = types.reserve_record(jai_types::RecordKind::Struct);
        types.define_record(parent, [ty]).unwrap();
        ty = parent;
    }
    let mut memory = crate::Memory::new(crate::Limits {
        evaluation_depth: 512,
        ..crate::Limits::default()
    });
    let mut pointer = memory.allocate(&types, ty, None).unwrap();
    for _ in 0..256 {
        pointer = memory.field(&types, &pointer, 0).unwrap();
    }
    assert_eq!(pointer.metadata_cells(), 256);
    assert_eq!(
        memory.field(&types, &pointer, 0),
        Err(Error::Limit(LimitKind::EvaluationDepth))
    );
}

#[test]
fn selected_range_work_counts_origins_without_cloning_unrelated_metadata() {
    let target = ByteTarget::default();
    let tagged = origin_image(64);
    let plain = ByteImage::from_bytes(target, vec![0; 8], 64).unwrap();
    let image = ByteImage::concatenate(&[plain.clone(), tagged, plain], target, 64).unwrap();
    assert_eq!(image.range_metadata_work(0, 8).unwrap(), 0);
    assert_eq!(image.range_metadata_work(8, 8).unwrap(), 12);
    assert_eq!(image.range_metadata_work(10, 1).unwrap(), 12);
    assert_eq!(image.range_metadata_work(16, 8).unwrap(), 0);
    assert_eq!(image.range_metadata_work(image.len(), 0).unwrap(), 0);
    assert!(matches!(
        image.range_metadata_work(usize::MAX, 1),
        Err(Error::OutOfBounds { .. })
    ));
    assert!(matches!(
        image.range_metadata_work(20, 8),
        Err(Error::OutOfBounds { .. })
    ));
    let (mut types, _memory, word, pointer) = deep_pointer_fixture(4096);
    let pointer_ty = types.pointer(word).unwrap();
    let pointer_image =
        ByteImage::encode(&types, target, pointer_ty, &Value::Pointer(pointer), 512).unwrap();
    assert_eq!(pointer_image.range_metadata_work(0, 8).unwrap(), 130);
    assert_eq!(pointer_image.range_metadata_work(1, 4).unwrap(), 1);
    assert_eq!(pointer_image.metadata_cells(), 130);
}
