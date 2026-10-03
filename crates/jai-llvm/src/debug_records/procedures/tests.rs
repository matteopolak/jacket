use super::*;
use inkwell::{AddressSpace, context::Context};

#[test]
fn owned_procedure_signatures_and_artificial_result_tuples_reject_nonstorage_misuse() {
    let context = Context::create();
    let module = context.create_module("own.procedure.debug");
    let foreign_module = context.create_module("own.foreign.debug");
    let make = |module| {
        DebugSession::new(
            module,
            &context,
            "callback.jai",
            "/own-source",
            "jai-rs",
            false,
            DebugEmission::Variables,
        )
        .unwrap()
    };
    let debug = make(&module);
    let foreign = make(&foreign_module);
    let int = debug.primitive_type(DebugPrimitive::Signed64).unwrap();
    let foreign_int = foreign.primitive_type(DebugPrimitive::Signed64).unwrap();
    assert!(matches!(
        debug.subroutine_type(None, &[foreign_int], DebugVariadic::None),
        Err(Error::DebugOwnership)
    ));
    assert!(matches!(
        debug.subroutine_type(Some(foreign_int), &[], DebugVariadic::None),
        Err(Error::DebugOwnership)
    ));
    let members = [
        DebugResultMember {
            ty: int,
            offset_bits: 0,
            alignment_bits: 64,
        },
        DebugResultMember {
            ty: int,
            offset_bits: 64,
            alignment_bits: 64,
        },
    ];
    let tuple = debug.result_tuple_type(128, 64, &members).unwrap();
    assert!(debug.result_tuple_type(64, 64, &members).is_err());
    assert!(debug.result_tuple_type(192, 64, &members).is_err());
    assert!(
        debug
            .result_tuple_type(128, 64, &[members[0], members[0]])
            .is_err()
    );
    let signature = debug
        .subroutine_type(Some(tuple), &[int], DebugVariadic::C)
        .unwrap();
    assert!(debug.array_type(signature, 1, 0, 8).is_err());
    assert!(
        debug
            .subroutine_type(Some(signature), &[], DebugVariadic::None)
            .is_err()
    );
    assert!(
        debug
            .subroutine_type(None, &[signature], DebugVariadic::None)
            .is_err()
    );
    assert!(
        debug
            .result_tuple_type(
                128,
                64,
                &[
                    DebugResultMember {
                        ty: signature,
                        ..members[0]
                    },
                    members[1]
                ]
            )
            .is_err()
    );
    let pointer = debug.pointer_type(Some(signature), 64, 64).unwrap();
    let builder = context.create_builder();
    let function = module.add_function("main", context.i32_type().fn_type(&[], false), None);
    builder.position_at_end(context.append_basic_block(function, "entry"));
    let source = DebugSource {
        file: "callback.jai",
        directory: "/own-source",
        line: NonZeroU32::new(1).unwrap(),
        column: NonZeroU32::new(1).unwrap(),
    };
    let scope = debug
        .function_scope(&builder, function, source, "main")
        .unwrap();
    assert!(matches!(
        debug.set_function_type(scope, foreign_int),
        Err(Error::DebugOwnership)
    ));
    assert!(debug.set_function_type(scope, int).is_err());
    let lexical = debug.lexical_scope(scope, source).unwrap();
    assert!(debug.set_function_type(lexical, signature).is_err());
    let result = debug.primitive_type(DebugPrimitive::Signed32).unwrap();
    let stdcall = debug
        .subroutine_type_with_convention(
            Some(result),
            &[],
            DebugVariadic::None,
            DebugCallingConvention::X86Stdcall,
        )
        .unwrap();
    debug.set_function_type(scope, stdcall).unwrap();
    assert!(
        debug
            .typed_variable(
                scope,
                source,
                "invalid",
                DebugVariableKind::Automatic,
                signature,
                8
            )
            .is_err()
    );
    let variable = debug
        .typed_variable(
            scope,
            source,
            "callback",
            DebugVariableKind::Automatic,
            pointer,
            8,
        )
        .unwrap();
    let storage = builder
        .build_alloca(context.ptr_type(AddressSpace::default()), "callback")
        .unwrap();
    debug.declare(&builder, storage, variable).unwrap();
    builder
        .build_return(Some(&context.i32_type().const_zero()))
        .unwrap();
    debug.finish();
    foreign.finish();
    module.verify().unwrap();
    let ir = module.print_to_string().to_string();
    assert!(
        ir.contains("DISubroutineType(flags: DIFlagPrototyped"),
        "{ir}"
    );
    assert!(ir.contains("flags: DIFlagArtificial"), "{ir}");
    assert!(ir.contains("DW_TAG_pointer_type"), "{ir}");
    assert!(ir.contains("DW_CC_BORLAND_stdcall"), "{ir}");
    assert!(!ir.contains("name: \"result0\""), "{ir}");
}
