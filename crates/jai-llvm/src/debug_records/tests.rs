use super::*;
fn source() -> DebugSource<'static> {
    DebugSource {
        file: "own.jai",
        directory: ".",
        line: NonZeroU32::new(2).unwrap(),
        column: NonZeroU32::new(1).unwrap(),
    }
}
#[test]
fn repeated_debug_records_remain_metadata_after_constant_folding() {
    let other = Context::create();
    let context = Context::create();
    let module = context.create_module("own.debug.records");
    let debug = DebugSession::new(
        &module,
        &context,
        "own.jai",
        ".",
        "jai-rs",
        false,
        DebugEmission::Variables,
    )
    .unwrap();
    let function = module.add_function("main", context.i32_type().fn_type(&[], false), None);
    let builder = context.create_builder();
    builder.position_at_end(context.append_basic_block(function, "entry"));
    let scope = debug
        .function_scope(&builder, function, source(), "main")
        .unwrap();
    let oversized_column = DebugSource {
        column: NonZeroU32::new(65_536).unwrap(),
        ..source()
    };
    assert!(matches!(
        debug.lexical_scope(scope, oversized_column),
        Err(Error::InvalidDebugCoordinates)
    ));
    assert!(matches!(
        debug.set_location(&builder, scope, oversized_column),
        Err(Error::InvalidDebugCoordinates)
    ));
    assert!(matches!(
        debug.variable(
            scope,
            source(),
            "invalid.argument",
            DebugVariableKind::Parameter(NonZeroU32::new(65_536).unwrap()),
            DebugPrimitive::Signed32,
            4
        ),
        Err(Error::InvalidDebugParameter)
    ));
    let storage = builder.build_alloca(context.i32_type(), "answer").unwrap();
    let folded = builder
        .build_int_add(
            context.i32_type().const_int(20, false),
            context.i32_type().const_int(22, false),
            "folded",
        )
        .unwrap();
    assert!(
        folded.is_const(),
        "fixture must cross the constant-folding boundary"
    );
    builder.build_store(storage, folded).unwrap();
    let variable = debug
        .variable(
            scope,
            source(),
            "answer",
            DebugVariableKind::Automatic,
            DebugPrimitive::Signed32,
            4,
        )
        .unwrap();
    let unset = context.create_builder();
    assert!(matches!(
        debug.declare(&unset, storage, variable),
        Err(Error::Build(BuilderError::UnsetPosition))
    ));
    let other_module = other.create_module("own.other.context");
    let other_function =
        other_module.add_function("other", other.void_type().fn_type(&[], false), None);
    let other_builder = other.create_builder();
    other_builder.position_at_end(other.append_basic_block(other_function, "entry"));
    let foreign_storage = other_builder
        .build_alloca(other.i32_type(), "other")
        .unwrap();
    assert!(matches!(
        debug.declare(&builder, foreign_storage, variable),
        Err(Error::ContextMismatch)
    ));
    other_builder.build_return(None).unwrap();
    other_module.verify().unwrap();
    for _ in 0..256 {
        debug.declare(&builder, storage, variable).unwrap();
    }
    debug.set_location(&builder, scope, source()).unwrap();
    builder.build_return(Some(&folded)).unwrap();
    debug.finish();
    module.verify().unwrap();
    let ir = module.print_to_string().to_string();
    assert_eq!(ir.matches("#dbg_declare").count(), 256, "{ir}");
}
#[test]
fn sessions_reject_same_context_foreign_modules_and_foreign_session_metadata() {
    let context = Context::create();
    let module = context.create_module("own.session");
    let other_module = context.create_module("own.other.module");
    let first = DebugSession::new(
        &module,
        &context,
        "own.jai",
        ".",
        "jai-rs",
        false,
        DebugEmission::Variables,
    )
    .unwrap();
    let second = DebugSession::new(
        &module,
        &context,
        "own.jai",
        ".",
        "jai-rs",
        false,
        DebugEmission::Variables,
    )
    .unwrap();
    let function = module.add_function("own", context.void_type().fn_type(&[], false), None);
    let builder = context.create_builder();
    builder.position_at_end(context.append_basic_block(function, "entry"));
    let scope = first
        .function_scope(&builder, function, source(), "own")
        .unwrap();
    let variable = first
        .variable(
            scope,
            source(),
            "answer",
            DebugVariableKind::Automatic,
            DebugPrimitive::Signed32,
            4,
        )
        .unwrap();
    let storage = builder.build_alloca(context.i32_type(), "answer").unwrap();
    assert!(matches!(
        second.lexical_scope(scope, source()),
        Err(Error::DebugOwnership)
    ));
    assert!(matches!(
        second.set_location(&builder, scope, source()),
        Err(Error::DebugOwnership)
    ));
    assert!(matches!(
        second.variable(
            scope,
            source(),
            "bad",
            DebugVariableKind::Automatic,
            DebugPrimitive::Signed32,
            4
        ),
        Err(Error::DebugOwnership)
    ));
    assert!(matches!(
        second.declare(&builder, storage, variable),
        Err(Error::DebugOwnership)
    ));
    let other_function =
        other_module.add_function("foreign", context.void_type().fn_type(&[], false), None);
    let other_builder = context.create_builder();
    other_builder.position_at_end(context.append_basic_block(other_function, "entry"));
    assert!(matches!(
        first.function_scope(&other_builder, other_function, source(), "foreign"),
        Err(Error::ContextMismatch)
    ));
    assert!(matches!(
        first.set_location(&other_builder, scope, source()),
        Err(Error::ContextMismatch)
    ));
    assert!(matches!(
        first.declare(&other_builder, storage, variable),
        Err(Error::ContextMismatch)
    ));
    let foreign_global = other_module.add_global(context.i32_type(), None, "foreign.storage");
    foreign_global.set_initializer(&context.i32_type().const_zero());
    assert!(matches!(
        first.declare(&builder, foreign_global.as_pointer_value(), variable),
        Err(Error::DebugStorageOwnership)
    ));
    assert!(matches!(
        first.declare(
            &builder,
            context
                .ptr_type(inkwell::AddressSpace::default())
                .const_null(),
            variable
        ),
        Err(Error::DebugStorageOwnership)
    ));
    let owned_global = module.add_global(context.i32_type(), None, "owned.storage");
    owned_global.set_initializer(&context.i32_type().const_zero());
    first
        .declare(&builder, owned_global.as_pointer_value(), variable)
        .unwrap();
    let array_type = context.i32_type().array_type(2);
    let owned_array = module.add_global(array_type, None, "owned.array");
    owned_array.set_initializer(&array_type.const_zero());
    let constant_gep = crate::const_gep(
        array_type.into(),
        owned_array.as_pointer_value(),
        &[
            context.i32_type().const_zero(),
            context.i32_type().const_int(1, false),
        ],
    )
    .unwrap();
    assert!(matches!(
        first.declare(&builder, constant_gep, variable),
        Err(Error::DebugStorageOwnership)
    ));
    let sibling = module.add_function(
        "sibling",
        context.void_type().fn_type(
            &[context.ptr_type(inkwell::AddressSpace::default()).into()],
            false,
        ),
        None,
    );
    let sibling_builder = context.create_builder();
    sibling_builder.position_at_end(context.append_basic_block(sibling, "entry"));
    let sibling_storage = sibling_builder
        .build_alloca(context.i32_type(), "sibling.answer")
        .unwrap();
    assert!(matches!(
        first.declare(&builder, sibling_storage, variable),
        Err(Error::DebugStorageOwnership)
    ));
    assert!(matches!(
        first.declare(
            &builder,
            sibling.get_first_param().unwrap().into_pointer_value(),
            variable
        ),
        Err(Error::DebugStorageOwnership)
    ));
    sibling_builder.build_return(None).unwrap();
    builder.build_return(None).unwrap();
    other_builder.build_return(None).unwrap();
    first.finish();
    second.finish();
    module.verify().unwrap();
    other_module.verify().unwrap();
}

#[test]
fn recursive_record_handles_resolve_replaced_slots_and_reject_foreign_children() {
    let context = Context::create();
    let module = context.create_module("own.recursive.debug");
    let other_module = context.create_module("own.foreign.debug.types");
    let debug = DebugSession::new(
        &module,
        &context,
        "own.jai",
        ".",
        "jai-rs",
        false,
        DebugEmission::Variables,
    )
    .unwrap();
    let foreign = DebugSession::new(
        &other_module,
        &context,
        "other.jai",
        ".",
        "jai-rs",
        false,
        DebugEmission::Variables,
    )
    .unwrap();
    let number = debug.primitive_type(DebugPrimitive::Signed32).unwrap();
    let foreign_number = foreign.primitive_type(DebugPrimitive::Signed32).unwrap();
    assert!(matches!(
        debug.pointer_type(Some(foreign_number), 64, 64),
        Err(Error::DebugOwnership)
    ));
    assert!(matches!(
        debug.array_type(number, u64::MAX, 64, 32),
        Err(Error::InvalidDebugType)
    ));
    assert!(matches!(
        debug.array_type(number, 2, 80, 64),
        Err(Error::InvalidDebugType)
    ));
    assert!(matches!(
        debug.begin_record(source(), Some("Unpadded"), DebugRecordKind::Struct, 8, 64),
        Err(Error::InvalidDebugType)
    ));
    let aligned = debug
        .begin_record(
            source(),
            Some("AlignedMember"),
            DebugRecordKind::Struct,
            64,
            64,
        )
        .unwrap();
    // Member over-alignment affects its placement; its type keeps its own size.
    debug
        .finish_record(
            aligned,
            &[DebugMember {
                name: "value",
                source: source(),
                ty: number,
                offset_bits: 0,
                alignment_bits: 64,
            }],
        )
        .unwrap();
    let node = debug
        .begin_record(source(), Some("Node"), DebugRecordKind::Struct, 128, 64)
        .unwrap();
    let retained_handle = node;
    let pointer = debug.pointer_type(Some(node), 64, 64).unwrap();
    let recursive_array = debug.array_type(node, 1, 128, 64).unwrap();
    for child in [node, recursive_array] {
        assert!(matches!(
            debug.finish_record(
                node,
                &[DebugMember {
                    name: "recursive_value",
                    source: source(),
                    ty: child,
                    offset_bits: 0,
                    alignment_bits: 64,
                }]
            ),
            Err(Error::InvalidDebugType)
        ));
    }
    let left = debug
        .begin_record(source(), Some("Left"), DebugRecordKind::Struct, 64, 64)
        .unwrap();
    let right = debug
        .begin_record(source(), Some("Right"), DebugRecordKind::Struct, 64, 64)
        .unwrap();
    debug
        .finish_record(
            left,
            &[DebugMember {
                name: "right",
                source: source(),
                ty: right,
                offset_bits: 0,
                alignment_bits: 64,
            }],
        )
        .unwrap();
    assert!(matches!(
        debug.finish_record(
            right,
            &[DebugMember {
                name: "left",
                source: source(),
                ty: left,
                offset_bits: 0,
                alignment_bits: 64,
            }]
        ),
        Err(Error::InvalidDebugType)
    ));
    assert!(matches!(
        debug.finish_record(
            node,
            &[DebugMember {
                name: "bad",
                source: source(),
                ty: number,
                offset_bits: 120,
                alignment_bits: 8
            }]
        ),
        Err(Error::InvalidDebugType)
    ));
    debug
        .finish_record(
            node,
            &[
                DebugMember {
                    name: "value",
                    source: source(),
                    ty: number,
                    offset_bits: 0,
                    alignment_bits: 32,
                },
                DebugMember {
                    name: "next",
                    source: source(),
                    ty: pointer,
                    offset_bits: 64,
                    alignment_bits: 64,
                },
            ],
        )
        .unwrap();
    assert!(matches!(
        debug.finish_record(node, &[]),
        Err(Error::InvalidDebugType)
    ));
    // The copied pre-completion token resolves the updated table slot, not a
    // deleted temporary metadata pointer.
    let function = module.add_function("main", context.i32_type().fn_type(&[], false), None);
    let builder = context.create_builder();
    builder.position_at_end(context.append_basic_block(function, "entry"));
    let scope = debug
        .function_scope(&builder, function, source(), "main")
        .unwrap();
    let storage = builder
        .build_alloca(context.i8_type().array_type(16), "node")
        .unwrap();
    let variable = debug
        .typed_variable(
            scope,
            source(),
            "node",
            DebugVariableKind::Automatic,
            retained_handle,
            8,
        )
        .unwrap();
    debug.declare(&builder, storage, variable).unwrap();
    builder
        .build_return(Some(&context.i32_type().const_int(42, false)))
        .unwrap();
    debug.finish();
    foreign.finish();
    module.verify().unwrap();
    other_module.verify().unwrap();
    let ir = module.print_to_string().to_string();
    assert!(ir.contains("name: \"Node\""), "{ir}");
    assert!(ir.contains("name: \"next\""), "{ir}");
    assert!(ir.contains("DW_TAG_pointer_type"), "{ir}");
    assert!(!ir.contains("<temporary!>"), "{ir}");
}

#[test]
fn unfinished_record_is_retained_as_a_genuine_forward_declaration() {
    let context = Context::create();
    let module = context.create_module("own.forward.debug");
    let debug = DebugSession::new(
        &module,
        &context,
        "own.jai",
        ".",
        "jai-rs",
        false,
        DebugEmission::Variables,
    )
    .unwrap();
    let record = debug
        .begin_record(source(), Some("Forward"), DebugRecordKind::Struct, 64, 64)
        .unwrap();
    let pointer = debug.pointer_type(Some(record), 64, 64).unwrap();
    let function = module.add_function("main", context.i32_type().fn_type(&[], false), None);
    let builder = context.create_builder();
    builder.position_at_end(context.append_basic_block(function, "entry"));
    let scope = debug
        .function_scope(&builder, function, source(), "main")
        .unwrap();
    let storage = builder
        .build_alloca(
            context.ptr_type(inkwell::AddressSpace::default()),
            "pointer",
        )
        .unwrap();
    let variable = debug
        .typed_variable(
            scope,
            source(),
            "pointer",
            DebugVariableKind::Automatic,
            pointer,
            8,
        )
        .unwrap();
    debug.declare(&builder, storage, variable).unwrap();
    builder
        .build_return(Some(&context.i32_type().const_int(42, false)))
        .unwrap();
    drop(debug);
    module.verify().unwrap();
    let ir = module.print_to_string().to_string();
    assert!(ir.contains("name: \"Forward\""), "{ir}");
    assert!(ir.contains("DIFlagFwdDecl"), "{ir}");
    assert!(!ir.contains("<temporary!>"), "{ir}");
}
