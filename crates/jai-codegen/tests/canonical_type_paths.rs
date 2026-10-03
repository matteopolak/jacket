use inkwell::{context::Context, targets::TargetData};
use jai_codegen::types::{self, TypeLowerer};
use jai_types::*;
#[test]
fn physical_paths_keep_source_nominal_field_owners_and_offsets() {
    let mut registry = TypeRegistry::new();
    let byte = registry.scalar(ScalarType::Int(IntegerType::U8));
    let word = registry.scalar(ScalarType::Int(IntegerType::U32));
    let first = registry.reserve_record(RecordKind::Struct);
    registry.define_record(first, [byte, word]).unwrap();
    let other = registry.reserve_record(RecordKind::Struct);
    registry.define_record(other, [byte, word]).unwrap();
    let first_field = registry.field(first, 1).unwrap().id;
    let other_field = registry.field(other, 1).unwrap().id;
    let registry = registry.freeze().unwrap();
    let context = Context::create();
    let target = jai_codegen::target::NativeTarget::new().unwrap();
    let mut lowerer = TypeLowerer::with_target(&context, &registry, &target.data);
    assert_eq!(
        lowerer.record_field_path(first, first_field).unwrap(),
        [1, 2]
    );
    assert!(matches!(
        lowerer.record_field_path(first, other_field),
        Err(types::Error::Type(TypeError::FieldOwner { .. }))
    ));
    let layout = lowerer.verify_layout(first, &target.data).unwrap();
    assert_eq!(layout.field_offsets.as_ref(), [0, 4]);
    assert_eq!((layout.size, layout.alignment), (8, 4));
    let constant = lowerer
        .record_constant(
            first,
            &[
                context.i8_type().const_int(42, false).into(),
                context.i32_type().const_int(7, false).into(),
            ],
        )
        .unwrap();
    assert_eq!(
        constant.get_type(),
        lowerer.basic(first).unwrap().into_struct_type()
    );
    let payload = constant.get_field_at_index(1).unwrap().into_struct_value();
    assert_eq!(payload.get_type().count_fields(), 3);
    assert_eq!(
        payload
            .get_type()
            .get_field_type_at_index(1)
            .unwrap()
            .into_array_type()
            .len(),
        3
    );
    assert_eq!(
        target.data.offset_of_element(&payload.get_type(), 2),
        Some(4)
    );
}
#[test]
fn composed_record_exceeding_ilp32_extent_is_rejected_before_body_construction() {
    let mut registry = TypeRegistry::new();
    let byte = registry.scalar(ScalarType::Int(IntegerType::U8));
    let huge = registry.fixed_array(byte, 2_147_483_648).unwrap();
    let record = registry.reserve_record(RecordKind::Struct);
    registry.define_record(record, [huge, huge]).unwrap();
    let registry = registry.freeze().unwrap();
    let context = Context::create();
    let target = TargetData::create("e-p:32:32-i64:32-f64:32");
    let mut lowerer = TypeLowerer::with_target(&context, &registry, &target);
    assert!(lowerer.basic(huge).unwrap().is_array_type());
    assert!(
        matches!(lowerer.basic(record),Err(types::Error::RecordStorageTooLarge{ty,size:4_294_967_296,address_bits:32}) if ty==record)
    );
}
#[test]
fn named_records_require_a_selected_target_but_scalars_remain_targetless() {
    let mut registry = TypeRegistry::new();
    let byte = registry.scalar(ScalarType::Int(IntegerType::U8));
    let record = registry.reserve_record(RecordKind::Struct);
    registry.define_record(record, [byte]).unwrap();
    let registry = registry.freeze().unwrap();
    let context = Context::create();
    let mut lowerer = TypeLowerer::new(&context, &registry);
    assert!(lowerer.basic(byte).unwrap().is_int_type());
    assert!(
        matches!(lowerer.basic(record),Err(types::Error::MissingRecordTarget(ty)) if ty==record)
    );
}
