use super::*;
use crate::{
    foreign,
    target::{TargetOptions, TargetSelection, Triple},
};
use jai_types::{CallingConvention, ForeignReturnAbi, ProcedureType, TypeRegistry, Variadic};

#[test]
fn microsoft_record_results_share_exact_call_and_definition_slots() {
    for triple in ["x86_64-pc-windows-msvc", "aarch64-pc-windows-msvc"] {
        for (convention, policy, result_index) in [
            (CallingConvention::C, ForeignReturnAbi::CppNonPod, 0),
            (CallingConvention::CppMethod, ForeignReturnAbi::CppNonPod, 1),
            (CallingConvention::CppMethod, ForeignReturnAbi::Natural, 1),
        ] {
            let context = Context::create();
            let target = NativeTarget::select(&TargetOptions {
                selection: TargetSelection::Triple(Triple::new(triple).unwrap()),
                ..Default::default()
            })
            .unwrap();
            let mut registry = TypeRegistry::new();
            let record = registry.reserve_record(RecordKind::Struct);
            let float = registry.float(jai_types::FloatType::F32);
            registry.define_record(record, [float, float]).unwrap();
            let receiver = registry.pointer(registry.void()).unwrap();
            let signature_ty = registry
                .procedure(ProcedureType {
                    parameters: Box::new([receiver, float]),
                    results: Box::new([record]),
                    convention,
                    return_abi: policy,
                    context: ContextMode::None,
                    variadic: Variadic::None,
                })
                .unwrap();
            let opposite = registry
                .procedure(ProcedureType {
                    parameters: Box::new([receiver, float]),
                    results: Box::new([record]),
                    convention: if convention == CallingConvention::CppMethod {
                        CallingConvention::C
                    } else {
                        CallingConvention::CppMethod
                    },
                    return_abi: ForeignReturnAbi::CppNonPod,
                    context: ContextMode::None,
                    variadic: Variadic::None,
                })
                .unwrap();
            let types = registry.freeze().unwrap();
            let mut lowerer = TypeLowerer::with_target(&context, &types, &target.data);
            let signature = Signature::classify(
                &context,
                &types,
                &mut lowerer,
                target.c_platform().unwrap(),
                &target.data,
                signature_ty,
                false,
            )
            .unwrap();
            assert!(matches!(signature.result, Value::Indirect { .. }));
            assert_eq!(signature.result_parameter, Some(result_index));
            assert_eq!(
                signature.parameter_indices,
                if result_index == 1 {
                    vec![0, 2]
                } else {
                    vec![1, 2]
                }
            );
            assert!(signature.llvm.get_return_type().is_none());
            let inreg = Attribute::get_named_enum_kind_id("inreg");
            assert_eq!(
                signature
                    .attributes
                    .iter()
                    .any(|(loc, attr)| *loc == AttributeLoc::Param(result_index)
                        && attr.is_enum()
                        && attr.get_enum_kind_id() == inreg),
                triple.starts_with("aarch64")
            );
            let module = context.create_module("actual-return-carriers");
            module.set_triple(&target.triple);
            module.set_data_layout(&target.data.get_data_layout());
            let function = module.add_function("method", signature.llvm, None);
            signature.attributes_on_function(function);
            assert!(matches!(
                foreign::declare(
                    &module,
                    &types,
                    &mut lowerer,
                    target.c_platform().unwrap(),
                    &target.data,
                    opposite,
                    "method"
                ),
                Err(Error::InvalidSymbol(_))
            ));
            let builder = context.create_builder();
            builder.position_at_end(context.append_basic_block(function, "entry"));
            let recovered = foreign::parameters(
                &builder,
                &types,
                &mut lowerer,
                &target.data,
                &signature,
                function,
            )
            .unwrap();
            assert_eq!(
                recovered[0],
                function
                    .get_nth_param(if result_index == 1 {
                        0
                    } else {
                        1
                    })
                    .unwrap()
            );
            let storage = lowerer.basic(record).unwrap();
            foreign::return_value(
                &builder,
                &context,
                &target.data,
                &signature,
                function,
                Some(storage.const_zero()),
            )
            .unwrap();
            let caller = module.add_function(
                "caller",
                storage.fn_type(
                    &[
                        context.ptr_type(AddressSpace::default()).into(),
                        context.f32_type().into(),
                        context.ptr_type(AddressSpace::default()).into(),
                    ],
                    false,
                ),
                None,
            );
            builder.position_at_end(context.append_basic_block(caller, "entry"));
            let args = [
                (receiver, caller.get_nth_param(0).unwrap()),
                (float, caller.get_nth_param(1).unwrap()),
            ];
            for callee in [
                foreign::Callee::Direct(function),
                foreign::Callee::Indirect(caller.get_nth_param(2).unwrap().into_pointer_value()),
            ] {
                let result = foreign::call(
                    &builder,
                    &types,
                    &mut lowerer,
                    &target.data,
                    &signature,
                    callee,
                    &args,
                )
                .unwrap();
                assert_eq!(result.value.unwrap().get_type(), storage);
            }
            builder.build_return(Some(&storage.const_zero())).unwrap();
            module.verify().unwrap();
            let ir = module.print_to_string().to_string();
            assert!(ir.contains("sret("));
            assert!(ir.contains("foreign.result"));
        }
    }
}

#[test]
fn explicit_non_pod_result_is_not_an_itanium_nontrivial_receipt() {
    let context = Context::create();
    let target = NativeTarget::select(&TargetOptions {
        selection: TargetSelection::Triple(Triple::new("x86_64-unknown-linux-gnu").unwrap()),
        ..Default::default()
    })
    .unwrap();
    let mut registry = TypeRegistry::new();
    let record = registry.reserve_record(RecordKind::Struct);
    let float = registry.float(jai_types::FloatType::F32);
    registry.define_record(record, [float, float]).unwrap();
    let forced = registry
        .procedure(ProcedureType {
            parameters: Box::new([]),
            results: Box::new([record]),
            convention: CallingConvention::C,
            return_abi: ForeignReturnAbi::CppNonPod,
            context: ContextMode::None,
            variadic: Variadic::None,
        })
        .unwrap();
    let natural = registry
        .procedure(ProcedureType {
            parameters: Box::new([]),
            results: Box::new([record]),
            convention: CallingConvention::C,
            return_abi: ForeignReturnAbi::Natural,
            context: ContextMode::None,
            variadic: Variadic::None,
        })
        .unwrap();
    let types = registry.freeze().unwrap();
    let mut lowerer = TypeLowerer::with_target(&context, &types, &target.data);
    assert!(matches!(
        Signature::classify(
            &context,
            &types,
            &mut lowerer,
            target.c_platform().unwrap(),
            &target.data,
            forced,
            false
        ),
        Err(Error::UnsupportedTarget(_))
    ));
    assert!(!matches!(
        Signature::classify(
            &context,
            &types,
            &mut lowerer,
            target.c_platform().unwrap(),
            &target.data,
            natural,
            false
        )
        .unwrap()
        .result,
        Value::Indirect { .. }
    ));
    assert!(
        crate::cpp_methods::validate_triple(
            &types,
            types.procedure_definition(forced).unwrap(),
            Platform::WindowsX86_64,
            "x86_64-w64-windows-gnu"
        )
        .is_err()
    );
}
