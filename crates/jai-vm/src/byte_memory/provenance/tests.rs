use crate::{
    AddressProvenance, ByteImage, ByteTarget, Error, Limits, Memory, Number, Pointer, Value,
};
use jai_types::{FloatType, Integer, IntegerType, RecordKind, ScalarType, TypeRegistry};

fn integer(ty: IntegerType, bits: i128) -> Value {
    Value::Int(Integer::wrapping(ty, bits))
}
fn setup() -> (
    TypeRegistry,
    Memory,
    jai_types::TypeId,
    jai_types::TypeId,
    Pointer,
) {
    let mut types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let pointer_ty = types.pointer(word).unwrap();
    let mut memory = Memory::new(Limits::default());
    let pointer = memory
        .allocate(&types, word, Some(integer(IntegerType::U64, 7)))
        .unwrap();
    (types, memory, word, pointer_ty, pointer)
}
fn pointer_image(
    types: &TypeRegistry,
    pointer_ty: jai_types::TypeId,
    pointer: &Pointer,
) -> ByteImage {
    let mut image = ByteImage::encode(
        types,
        ByteTarget::default(),
        pointer_ty,
        &Value::Pointer(pointer.clone()),
        128,
    )
    .unwrap();
    image.retokenize_handles(|_| Ok(42)).unwrap();
    image
}
fn derived(value: Value, pointer: &Pointer) {
    let number = value.number().unwrap();
    let Some(AddressProvenance::Derived {
        memory: actual,
        allocations,
    }) = number.provenance()
    else {
        panic!("expected derived provenance")
    };
    assert_eq!(*actual, pointer.memory_identity());
    assert_eq!(allocations.as_ref(), &[pointer.allocation_key().1]);
}

#[test]
fn integer_views_preserve_complete_pointer_and_derive_partial_provenance() {
    let (types, _memory, word, pointer_ty, pointer) = setup();
    let image = pointer_image(&types, pointer_ty, &pointer);
    let number = image
        .read(&types, ByteTarget::default(), 0, word)
        .unwrap()
        .number()
        .unwrap();
    assert_eq!(number.bits(), 42);
    assert_eq!(
        number.provenance(),
        Some(&AddressProvenance::Pointer(pointer.clone()))
    );
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    derived(
        image.read(&types, ByteTarget::default(), 1, byte).unwrap(),
        &pointer,
    );
    let plain = ByteImage::encode(
        &types,
        ByteTarget::default(),
        word,
        &integer(IntegerType::U64, 42),
        128,
    )
    .unwrap();
    assert_eq!(plain.bytes(), image.bytes());
    assert_eq!(
        plain.read(&types, ByteTarget::default(), 0, word).unwrap(),
        integer(IntegerType::U64, 42)
    );
}

#[test]
fn float_bool_and_enum_views_cannot_launder_address_bytes() {
    let (mut types, _memory, _word, pointer_ty, pointer) = setup();
    let enumeration = types.reserve_enum(IntegerType::U64);
    types
        .define_enum(enumeration, [Integer::wrapping(IntegerType::U64, 42)])
        .unwrap();
    let image = pointer_image(&types, pointer_ty, &pointer);
    for ty in [
        types.float(FloatType::F64),
        types.float(FloatType::F32),
        types.scalar(ScalarType::Bool),
        enumeration,
    ] {
        assert!(matches!(
            image.read(&types, ByteTarget::default(), 0, ty),
            Err(Error::UnsupportedPointerOperation(_))
        ));
    }
}

#[test]
fn typed_address_integer_store_keeps_bits_and_never_becomes_an_opaque_pointer() {
    let (types, _memory, word, pointer_ty, pointer) = setup();
    let number = Number::address(
        Integer::wrapping(IntegerType::U64, 42),
        AddressProvenance::Pointer(pointer.clone()),
    );
    let value = number.clone().into_value();
    let mut image = ByteImage::encode(&types, ByteTarget::default(), word, &value, 128).unwrap();
    image
        .retokenize_handles(|_| panic!("address integer bits must not be retokenized"))
        .unwrap();
    assert_eq!(
        image.read(&types, ByteTarget::default(), 0, word).unwrap(),
        value
    );
    assert!(matches!(
        image.read(&types, ByteTarget::default(), 0, pointer_ty),
        Err(Error::UnsupportedPointerOperation(_))
    ));
    let marked = Number::address(
        Integer::wrapping(IntegerType::U64, 43),
        AddressProvenance::Derived {
            memory: pointer.memory_identity(),
            allocations: vec![pointer.allocation_key().1].into_boxed_slice(),
        },
    );
    image
        .write(
            &types,
            ByteTarget::default(),
            0,
            word,
            &marked.clone().into_value(),
        )
        .unwrap();
    assert_eq!(
        image.read(&types, ByteTarget::default(), 0, word).unwrap(),
        marked.into_value()
    );
}

#[test]
fn partial_copies_carry_only_intersecting_provenance_and_full_copies_retain_handles() {
    let (types, _memory, word, pointer_ty, pointer) = setup();
    let source = pointer_image(&types, pointer_ty, &pointer);
    let target = ByteTarget::default();
    let mut full = ByteImage::from_bytes(target, vec![0; 8], 128).unwrap();
    full.copy_range_from(&source, 0, 0, 8).unwrap();
    assert_eq!(
        full.read(&types, target, 0, pointer_ty).unwrap(),
        Value::Pointer(pointer.clone())
    );
    let mut partial = ByteImage::from_bytes(target, vec![0; 8], 128).unwrap();
    partial.copy_range_from(&source, 0, 0, 1).unwrap();
    // Numerically identical bytes do not replace the missing opaque relocation.
    assert_eq!(partial.bytes(), source.bytes());
    derived(partial.read(&types, target, 0, word).unwrap(), &pointer);
    assert!(matches!(
        partial.read(&types, target, 0, pointer_ty),
        Err(Error::UnsupportedPointerOperation(_))
    ));
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    assert_eq!(
        partial.read(&types, target, 1, byte).unwrap(),
        integer(IntegerType::U8, 0)
    );
    partial.copy_range_from(&source, 1, 1, 7).unwrap();
    derived(partial.read(&types, target, 0, word).unwrap(), &pointer);
}

#[test]
fn writes_clear_only_their_extent_and_tagged_memset_preserves_origin() {
    let (types, _memory, word, pointer_ty, pointer) = setup();
    let target = ByteTarget::default();
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let mut image = pointer_image(&types, pointer_ty, &pointer);
    image
        .write(&types, target, 0, byte, &integer(IntegerType::U8, 0))
        .unwrap();
    assert_eq!(
        image.read(&types, target, 0, byte).unwrap(),
        integer(IntegerType::U8, 0)
    );
    derived(image.read(&types, target, 1, byte).unwrap(), &pointer);
    derived(image.read(&types, target, 0, word).unwrap(), &pointer);
    image.fill_range(0, 8, 0).unwrap();
    assert_eq!(
        image.read(&types, target, 0, word).unwrap(),
        integer(IntegerType::U64, 0)
    );
    let fill = Number::address(
        Integer::wrapping(IntegerType::U8, 0),
        AddressProvenance::Derived {
            memory: pointer.memory_identity(),
            allocations: vec![pointer.allocation_key().1].into_boxed_slice(),
        },
    );
    image.fill_range_number(0, 8, fill).unwrap();
    derived(image.read(&types, target, 0, word).unwrap(), &pointer);
    assert!(matches!(
        image.read(&types, target, 0, pointer_ty),
        Err(Error::UnsupportedPointerOperation(_))
    ));
}

#[test]
fn union_views_and_uninitialized_copies_keep_provenance_and_holes() {
    let (mut types, _memory, word, _pointer_ty, pointer) = setup();
    let float = types.float(FloatType::F64);
    let union = types.reserve_record(RecordKind::Union);
    types.define_record(union, [word, float]).unwrap();
    let address = Number::address(
        Integer::wrapping(IntegerType::U64, 42),
        AddressProvenance::Pointer(pointer.clone()),
    )
    .into_value();
    let value = Value::Union {
        ty: union,
        field: 0,
        value: Box::new(address.clone()),
    };
    let source = ByteImage::encode(&types, ByteTarget::default(), union, &value, 128).unwrap();
    assert_eq!(
        source.read_union_field(&types, 0, union, 0).unwrap(),
        address
    );
    assert!(matches!(
        source.read_union_field(&types, 0, union, 1),
        Err(Error::UnsupportedPointerOperation(_))
    ));
    let mut partial = ByteImage::uninitialized(ByteTarget::default(), 8, 128).unwrap();
    partial.copy_range_from(&source, 0, 0, 4).unwrap();
    let half = types.scalar(ScalarType::Int(IntegerType::U32));
    derived(
        partial
            .read(&types, ByteTarget::default(), 0, half)
            .unwrap(),
        &pointer,
    );
    assert!(matches!(
        partial.read(&types, ByteTarget::default(), 0, word),
        Err(Error::Uninitialized)
    ));
}

#[test]
fn empty_writes_do_not_shred_existing_handle_or_provenance() {
    let (types, _memory, word, pointer_ty, pointer) = setup();
    let mut image = pointer_image(&types, pointer_ty, &pointer);
    image.write_range(4, &[]).unwrap();
    image.fill_range(4, 0, 0).unwrap();
    assert_eq!(
        image
            .read(&types, ByteTarget::default(), 0, pointer_ty)
            .unwrap(),
        Value::Pointer(pointer.clone())
    );
    assert_eq!(
        image
            .read(&types, ByteTarget::default(), 0, word)
            .unwrap()
            .number()
            .unwrap()
            .provenance(),
        Some(&AddressProvenance::Pointer(pointer))
    );
}

#[test]
fn intersecting_origin_sets_merge_exactly_and_never_accept_a_foreign_memory_family() {
    let (types, mut memory, word, pointer_ty, first) = setup();
    let second = memory
        .allocate(&types, word, Some(integer(IntegerType::U64, 9)))
        .unwrap();
    let one = pointer_image(&types, pointer_ty, &first);
    let two = pointer_image(&types, pointer_ty, &second);
    let target = ByteTarget::default();
    let mut combined = ByteImage::from_bytes(target, vec![0; 8], 128).unwrap();
    combined.copy_range_from(&one, 0, 0, 4).unwrap();
    combined.copy_range_from(&two, 0, 4, 4).unwrap();
    let number = combined
        .read(&types, target, 0, word)
        .unwrap()
        .number()
        .unwrap();
    assert_eq!(
        number.provenance(),
        Some(&AddressProvenance::Derived {
            memory: first.memory_identity(),
            allocations: vec![first.allocation_key().1, second.allocation_key().1]
                .into_boxed_slice(),
        })
    );
    let mut foreign_memory = Memory::new(Limits::default());
    let foreign = foreign_memory
        .allocate(&types, word, Some(integer(IntegerType::U64, 9)))
        .unwrap();
    let foreign_image = pointer_image(&types, pointer_ty, &foreign);
    combined.copy_range_from(&foreign_image, 0, 4, 4).unwrap();
    assert!(matches!(
        combined.read(&types, target, 0, word),
        Err(Error::UnsupportedPointerOperation(_))
    ));
}

#[test]
fn exact_wider_integer_storage_preserves_pointer_origin_but_split_fragments_do_not() {
    let (types, _memory, word, _pointer_ty, pointer) = setup();
    let half = types.scalar(ScalarType::Int(IntegerType::U32));
    let policy = jai_types::LayoutPolicy::new(
        jai_types::ScalarLayout::new(4, 4),
        [
            jai_types::ScalarLayout::new(1, 1),
            jai_types::ScalarLayout::new(2, 2),
            jai_types::ScalarLayout::new(4, 4),
            jai_types::ScalarLayout::new(8, 4),
        ],
        [
            jai_types::ScalarLayout::new(4, 4),
            jai_types::ScalarLayout::new(8, 4),
        ],
        jai_types::ScalarLayout::new(1, 1),
    )
    .unwrap();
    let target = ByteTarget {
        policy,
        ..ByteTarget::default()
    };
    let value = Number::address(
        Integer::wrapping(IntegerType::U64, 42),
        AddressProvenance::Pointer(pointer.clone()),
    )
    .into_value();
    let mut image = ByteImage::encode(&types, target, word, &value, 128).unwrap();
    assert_eq!(image.read(&types, target, 0, word).unwrap(), value);
    // Reading only the target-width low half is still a partial origin view.
    derived(image.read(&types, target, 0, half).unwrap(), &pointer);
    image.write_range(4, &[0; 4]).unwrap();
    // The surviving four-byte fragment cannot acquire complete-pointer authority.
    derived(image.read(&types, target, 0, half).unwrap(), &pointer);
}
