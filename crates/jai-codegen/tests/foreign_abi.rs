//! Compiles only self-written C fixture source with the installed trusted Clang.
#[path = "support/native_tools.rs"]
mod native_tools;
use inkwell::{context::Context, values::BasicValueEnum};
use jai_codegen::{abi, foreign, types::TypeLowerer, unions};
use jai_types::*;
use std::{
    fs,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};

fn signature(
    registry: &mut TypeRegistry,
    parameters: Vec<TypeId>,
    result: Option<TypeId>,
    variadic: bool,
) -> TypeId {
    let fixed_parameters = parameters.len();
    registry
        .procedure(ProcedureType {
            parameters: parameters.into_boxed_slice(),
            results: result.into_iter().collect(),
            convention: CallingConvention::C,
            context: ContextMode::None,
            variadic: if variadic {
                Variadic::C { fixed_parameters }
            } else {
                Variadic::None
            },
        })
        .unwrap()
}
fn record(registry: &mut TypeRegistry, fields: &[TypeId], kind: RecordKind) -> TypeId {
    let ty = registry.reserve_record(kind);
    registry.define_record(ty, fields.to_vec()).unwrap();
    ty
}
fn aggregate<'ctx>(
    ty: TypeId,
    values: Vec<BasicValueEnum<'ctx>>,
    lowerer: &mut TypeLowerer<'ctx, '_>,
) -> inkwell::values::StructValue<'ctx> {
    lowerer
        .basic(ty)
        .unwrap()
        .into_struct_type()
        .const_named_struct(&values)
}
fn execute(module: &inkwell::module::Module<'_>, fixture: &str) -> i32 {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Scratch(std::path::PathBuf);
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let scratch = Scratch(std::env::temp_dir().join(format!(
        "jai-foreign-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )));
    fs::create_dir(&scratch.0).unwrap();
    let object = scratch.0.join("generated.o");
    let c = scratch.0.join("fixture.c");
    let executable = scratch.0.join("program");
    module.verify().unwrap();
    jai_codegen::target::NativeTarget::new()
        .unwrap()
        .write_object(module, &object)
        .unwrap();
    fs::write(&c, fixture).unwrap();
    let compiled = native_tools::clang_command()
        .arg(&object)
        .arg(&c)
        .arg("-o")
        .arg(&executable)
        .output()
        .unwrap();
    assert!(
        compiled.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&compiled.stderr),
        module.print_to_string()
    );
    let status = Command::new(&executable).status().unwrap();
    status.code().expect("fixture terminated by signal")
}

#[test]
fn scalar_foreign_calls_narrow_extensions_varargs_pointer_and_indirect() {
    let mut registry = TypeRegistry::new();
    let s8 = registry.scalar(ScalarType::Int(IntegerType::S8));
    let u8 = registry.scalar(ScalarType::Int(IntegerType::U8));
    let s32 = registry.scalar(ScalarType::Int(IntegerType::S32));
    let s64 = registry.scalar(ScalarType::Int(IntegerType::S64));
    let boolean = registry.scalar(ScalarType::Bool);
    let f32 = registry.float(FloatType::F32);
    let f64 = registry.float(FloatType::F64);
    let pointer = registry.pointer(s64).unwrap();
    let narrow = signature(&mut registry, vec![s8, u8, boolean], Some(s32), false);
    let floating = signature(&mut registry, vec![f32, f64], Some(f64), false);
    let mutation = signature(&mut registry, vec![pointer], None, false);
    let variadic = signature(&mut registry, vec![s32], Some(s32), true);
    let types = registry.freeze().unwrap();
    let context = Context::create();
    let target = abi::NativeTarget::new().unwrap();
    let module = context.create_module("foreign.scalars");
    module.set_triple(&target.triple);
    module.set_data_layout(&target.data.get_data_layout());
    let mut lowerer = TypeLowerer::with_target(&context, &types, &target.data);
    let builder = context.create_builder();
    let main = module.add_function("main", context.i32_type().fn_type(&[], false), None);
    builder.position_at_end(context.append_basic_block(main, "entry"));
    let mut checks = vec![];
    let function = foreign::declare(
        &module,
        &types,
        &mut lowerer,
        target.c_platform().unwrap(),
        &target.data,
        narrow,
        "narrow",
    )
    .unwrap();
    let result = function
        .call(
            &builder,
            &types,
            &mut lowerer,
            &target.data,
            &[
                (s8, context.i8_type().const_int((-7i8) as u64, true).into()),
                (u8, context.i8_type().const_int(250, false).into()),
                (boolean, context.bool_type().const_int(1, false).into()),
            ],
        )
        .unwrap()
        .value
        .unwrap()
        .into_int_value();
    checks.push(
        builder
            .build_int_compare(
                inkwell::IntPredicate::EQ,
                result,
                context.i32_type().const_int(244, false),
                "narrow.ok",
            )
            .unwrap(),
    );
    let function = foreign::declare(
        &module,
        &types,
        &mut lowerer,
        target.c_platform().unwrap(),
        &target.data,
        floating,
        "floating",
    )
    .unwrap();
    let result = foreign::call(
        &builder,
        &types,
        &mut lowerer,
        &target.data,
        &function.signature,
        foreign::Callee::Indirect(function.value.as_global_value().as_pointer_value()),
        &[
            (f32, context.f32_type().const_float(1.25).into()),
            (f64, context.f64_type().const_float(2.5).into()),
        ],
    )
    .unwrap()
    .value
    .unwrap()
    .into_float_value();
    checks.push(
        builder
            .build_float_compare(
                inkwell::FloatPredicate::OEQ,
                result,
                context.f64_type().const_float(3.75),
                "float.ok",
            )
            .unwrap(),
    );
    let slot = builder.build_alloca(context.i64_type(), "pointed").unwrap();
    builder
        .build_store(slot, context.i64_type().const_zero())
        .unwrap();
    let function = foreign::declare(
        &module,
        &types,
        &mut lowerer,
        target.c_platform().unwrap(),
        &target.data,
        mutation,
        "mutation",
    )
    .unwrap();
    function
        .call(
            &builder,
            &types,
            &mut lowerer,
            &target.data,
            &[(pointer, slot.into())],
        )
        .unwrap();
    let value = builder
        .build_load(context.i64_type(), slot, "changed")
        .unwrap()
        .into_int_value();
    checks.push(
        builder
            .build_int_compare(
                inkwell::IntPredicate::EQ,
                value,
                context.i64_type().const_int(42, false),
                "pointer.ok",
            )
            .unwrap(),
    );
    let function = foreign::declare(
        &module,
        &types,
        &mut lowerer,
        target.c_platform().unwrap(),
        &target.data,
        variadic,
        "variadic",
    )
    .unwrap();
    let result = function
        .call(
            &builder,
            &types,
            &mut lowerer,
            &target.data,
            &[
                (s32, context.i32_type().const_int(3, false).into()),
                (s8, context.i8_type().const_int((-7i8) as u64, true).into()),
                (f32, context.f32_type().const_float(1.25).into()),
                (pointer, slot.into()),
            ],
        )
        .unwrap()
        .value
        .unwrap()
        .into_int_value();
    checks.push(
        builder
            .build_int_compare(
                inkwell::IntPredicate::EQ,
                result,
                context.i32_type().const_int(1, false),
                "variadic.ok",
            )
            .unwrap(),
    );
    let passed = checks
        .into_iter()
        .reduce(|a, b| builder.build_and(a, b, "checks").unwrap())
        .unwrap();
    let exit = builder
        .build_select(
            passed,
            context.i32_type().const_zero(),
            context.i32_type().const_int(1, false),
            "exit",
        )
        .unwrap();
    builder.build_return(Some(&exit)).unwrap();
    assert_eq!(
        execute(
            &module,
            r#"
#include <stdint.h>
#include <stdbool.h>
#include <stdarg.h>
int narrow(int8_t a,uint8_t b,bool c) { return a+b+c; }
double floating(float a,double b) { return a+b; }
void mutation(int64_t *p) { *p=42; }
int variadic(int count,...) { va_list args; va_start(args,count); int a=va_arg(args,int); double b=va_arg(args,double); int64_t *p=va_arg(args,int64_t*); va_end(args); return count==3 && a==-7 && b==1.25 && *p==42; }
"#
        ),
        0
    );
}

#[test]
fn aggregates_roundtrip_through_real_c_abi_including_unions_hfa_and_sret() {
    let mut registry = TypeRegistry::new();
    let byte = registry.scalar(ScalarType::Int(IntegerType::U8));
    let int = registry.scalar(ScalarType::Int(IntegerType::S32));
    let long = registry.scalar(ScalarType::Int(IntegerType::S64));
    let float = registry.float(FloatType::F32);
    let double = registry.float(FloatType::F64);
    let odd = record(&mut registry, &[byte, byte, byte], RecordKind::Struct);
    let pair = record(&mut registry, &[long, long], RecordKind::Struct);
    let mixed = record(&mut registry, &[double, int], RecordKind::Struct);
    let hfa = record(&mut registry, &[float, float, float], RecordKind::Struct);
    let large = record(&mut registry, &[long, long, long], RecordKind::Struct);
    let union = record(&mut registry, &[long, double], RecordKind::Union);
    let union_hfa = record(&mut registry, &[double, double], RecordKind::Union);
    let double_hfa = record(
        &mut registry,
        &[double, double, double, double],
        RecordKind::Struct,
    );
    let shapes = [
        (odd, "odd"),
        (pair, "pair"),
        (mixed, "mixed"),
        (hfa, "hfa"),
        (large, "large"),
        (union, "union_value"),
        (union_hfa, "union_hfa"),
        (double_hfa, "double_hfa"),
    ];
    let signatures = shapes.map(|(ty, _)| signature(&mut registry, vec![ty], Some(ty), false));
    let types = registry.freeze().unwrap();
    let context = Context::create();
    let target = abi::NativeTarget::new().unwrap();
    let module = context.create_module("foreign.aggregates");
    module.set_triple(&target.triple);
    module.set_data_layout(&target.data.get_data_layout());
    let mut lowerer = TypeLowerer::with_target(&context, &types, &target.data);
    let builder = context.create_builder();
    let main = module.add_function("main", context.i32_type().fn_type(&[], false), None);
    builder.position_at_end(context.append_basic_block(main, "entry"));
    let inputs = [
        aggregate(
            odd,
            vec![
                context.i8_type().const_int(2, false).into(),
                context.i8_type().const_int(3, false).into(),
                context.i8_type().const_int(4, false).into(),
            ],
            &mut lowerer,
        )
        .into(),
        aggregate(
            pair,
            vec![
                context.i64_type().const_int(10, false).into(),
                context.i64_type().const_int(20, false).into(),
            ],
            &mut lowerer,
        )
        .into(),
        aggregate(
            mixed,
            vec![
                context.f64_type().const_float(2.5).into(),
                context.i32_type().const_int(5, false).into(),
            ],
            &mut lowerer,
        )
        .into(),
        aggregate(
            hfa,
            vec![
                context.f32_type().const_float(1.0).into(),
                context.f32_type().const_float(2.0).into(),
                context.f32_type().const_float(3.0).into(),
            ],
            &mut lowerer,
        )
        .into(),
        aggregate(
            large,
            vec![
                context.i64_type().const_int(10, false).into(),
                context.i64_type().const_int(20, false).into(),
                context.i64_type().const_int(30, false).into(),
            ],
            &mut lowerer,
        )
        .into(),
        unions::construct(
            &context,
            &builder,
            lowerer.basic(union).unwrap().into_struct_type(),
            context.i64_type().const_int(42, false).into(),
        )
        .unwrap()
        .into(),
        unions::construct(
            &context,
            &builder,
            lowerer.basic(union_hfa).unwrap().into_struct_type(),
            context.f64_type().const_float(4.5).into(),
        )
        .unwrap()
        .into(),
        aggregate(
            double_hfa,
            vec![
                context.f64_type().const_float(1.0).into(),
                context.f64_type().const_float(2.0).into(),
                context.f64_type().const_float(3.0).into(),
                context.f64_type().const_float(4.0).into(),
            ],
            &mut lowerer,
        )
        .into(),
    ];
    let mut checks = vec![];
    for (index, ((ty, symbol), signature)) in shapes.into_iter().zip(signatures).enumerate() {
        let function = foreign::declare(
            &module,
            &types,
            &mut lowerer,
            target.c_platform().unwrap(),
            &target.data,
            signature,
            symbol,
        )
        .unwrap();
        let value = function
            .call(
                &builder,
                &types,
                &mut lowerer,
                &target.data,
                &[(ty, inputs[index])],
            )
            .unwrap()
            .value
            .unwrap()
            .into_struct_value();
        let (member, expected) = match index {
            0 => (
                builder.build_extract_value(value, 2, "odd").unwrap(),
                context.i8_type().const_int(5, false).into(),
            ),
            1 => (
                builder.build_extract_value(value, 1, "pair").unwrap(),
                context.i64_type().const_int(22, false).into(),
            ),
            2 => (
                builder.build_extract_value(value, 0, "mixed").unwrap(),
                context.f64_type().const_float(3.0).into(),
            ),
            3 => (
                builder.build_extract_value(value, 2, "hfa").unwrap(),
                context.f32_type().const_float(6.0).into(),
            ),
            4 => (
                builder.build_extract_value(value, 2, "large").unwrap(),
                context.i64_type().const_int(33, false).into(),
            ),
            5 => (
                unions::extract(&context, &builder, value, context.i64_type().into()).unwrap(),
                context.i64_type().const_int(43, false).into(),
            ),
            6 => (
                unions::extract(&context, &builder, value, context.f64_type().into()).unwrap(),
                context.f64_type().const_float(5.0).into(),
            ),
            _ => (
                builder.build_extract_value(value, 3, "double_hfa").unwrap(),
                context.f64_type().const_float(8.0).into(),
            ),
        };
        checks.push(match (member, expected) {
            (BasicValueEnum::IntValue(a), BasicValueEnum::IntValue(b)) => builder
                .build_int_compare(inkwell::IntPredicate::EQ, a, b, "aggregate.ok")
                .unwrap(),
            (BasicValueEnum::FloatValue(a), BasicValueEnum::FloatValue(b)) => builder
                .build_float_compare(inkwell::FloatPredicate::OEQ, a, b, "aggregate.ok")
                .unwrap(),
            _ => panic!("wrong test member kind"),
        });
    }
    let passed = checks
        .into_iter()
        .reduce(|a, b| builder.build_and(a, b, "checks").unwrap())
        .unwrap();
    let exit = builder
        .build_select(
            passed,
            context.i32_type().const_zero(),
            context.i32_type().const_int(1, false),
            "exit",
        )
        .unwrap();
    builder.build_return(Some(&exit)).unwrap();
    assert_eq!(
        execute(
            &module,
            r#"
#include <stdint.h>
struct Odd { uint8_t a,b,c; }; struct Odd odd(struct Odd x) { ++x.c; return x; }
struct Pair { int64_t a,b; }; struct Pair pair(struct Pair x) { x.b+=2; return x; }
struct Mixed { double a; int32_t b; }; struct Mixed mixed(struct Mixed x) { x.a+=0.5; return x; }
struct Hfa { float a,b,c; }; struct Hfa hfa(struct Hfa x) { x.c+=3; return x; }
struct Large { int64_t a,b,c; }; struct Large large(struct Large x) { x.c+=3; return x; }
union Union { int64_t a; double b; }; union Union union_value(union Union x) { ++x.a; return x; }
union UnionHfa { double a,b; }; union UnionHfa union_hfa(union UnionHfa x) { x.b+=0.5; return x; }
struct DoubleHfa {double a,b,c,d;}; struct DoubleHfa double_hfa(struct DoubleHfa x) {x.d+=4;return x;}
"#
        ),
        0
    );
}

#[test]
fn linux_sysv_shapes_and_register_exhaustion_are_verified_without_host_execution() {
    use inkwell::{
        OptimizationLevel,
        targets::{CodeModel, InitializationConfig, RelocMode, Target, TargetTriple},
    };
    Target::initialize_all(&InitializationConfig::default());
    let triple = TargetTriple::create("x86_64-unknown-linux-gnu");
    let machine = Target::from_triple(&triple)
        .unwrap()
        .create_target_machine(
            &triple,
            "generic",
            "",
            OptimizationLevel::None,
            RelocMode::Default,
            CodeModel::Default,
        )
        .unwrap();
    let data = machine.get_target_data();
    let context = Context::create();
    let mut registry = TypeRegistry::new();
    let int = registry.scalar(ScalarType::Int(IntegerType::S32));
    let long = registry.scalar(ScalarType::Int(IntegerType::S64));
    let float = registry.float(FloatType::F32);
    let double = registry.float(FloatType::F64);
    let pair = record(&mut registry, &[long, long], RecordKind::Struct);
    let mixed = record(&mut registry, &[double, int], RecordKind::Struct);
    let hfa = record(&mut registry, &[float, float, float], RecordKind::Struct);
    let large = record(&mut registry, &[long, long, long], RecordKind::Struct);
    let sig_pair = signature(&mut registry, vec![pair], Some(pair), false);
    let sig_mixed = signature(&mut registry, vec![mixed], Some(mixed), false);
    let sig_hfa = signature(&mut registry, vec![hfa], Some(hfa), false);
    let sig_large = signature(&mut registry, vec![large], Some(large), false);
    let exhausted = signature(
        &mut registry,
        vec![long, long, long, long, long, pair],
        Some(int),
        false,
    );
    let types = registry.freeze().unwrap();
    let mut lowerer = TypeLowerer::with_target(&context, &types, &data);
    let mut classify = |signature| {
        abi::Signature::classify(
            &context,
            &types,
            &mut lowerer,
            abi::Platform::LinuxX86_64,
            &data,
            signature,
            false,
        )
        .unwrap()
    };
    let pair = classify(sig_pair);
    assert_eq!(
        pair.llvm.get_param_types(),
        vec![context.i64_type().into(), context.i64_type().into()]
    );
    assert_eq!(
        pair.llvm.get_return_type(),
        Some(
            context
                .struct_type(
                    &[context.i64_type().into(), context.i64_type().into()],
                    false
                )
                .into()
        )
    );
    let mixed = classify(sig_mixed);
    assert_eq!(
        mixed.llvm.get_param_types(),
        vec![context.f64_type().into(), context.i32_type().into()]
    );
    let hfa = classify(sig_hfa);
    assert_eq!(
        hfa.llvm.get_param_types(),
        vec![
            context.f32_type().vec_type(2).into(),
            context.f32_type().into()
        ]
    );
    let large = classify(sig_large);
    assert!(large.llvm.get_return_type().is_none());
    assert_eq!(large.llvm.count_param_types(), 2);
    assert!(matches!(large.result, abi::Value::Indirect { .. }));
    assert!(matches!(
        large.parameters[0],
        abi::Value::Indirect { by_value: true, .. }
    ));
    let exhausted = classify(exhausted);
    assert!(matches!(
        exhausted.parameters[5],
        abi::Value::Indirect { by_value: true, .. }
    ));
    let module = context.create_module("linux.abi.classification");
    module.set_triple(&triple);
    module.set_data_layout(&data.get_data_layout());
    for (name, signature) in [
        ("pair", pair),
        ("mixed", mixed),
        ("hfa", hfa),
        ("large", large),
        ("exhausted", exhausted),
    ] {
        let f = module.add_function(name, signature.llvm, None);
        signature.attributes_on_function(f);
    }
    module.verify().unwrap();
}

#[test]
fn c_calls_generated_definitions_with_aggregate_parameters_and_returns() {
    let mut registry = TypeRegistry::new();
    let long = registry.scalar(ScalarType::Int(IntegerType::S64));
    let int = registry.scalar(ScalarType::Int(IntegerType::S32));
    let float = registry.float(FloatType::F32);
    let double = registry.float(FloatType::F64);
    let pair = record(&mut registry, &[long, long], RecordKind::Struct);
    let hfa = record(&mut registry, &[float, float, float], RecordKind::Struct);
    let large = record(&mut registry, &[long, long, long], RecordKind::Struct);
    let union = record(&mut registry, &[long, double], RecordKind::Union);
    let shapes = [
        (pair, "callback_pair"),
        (hfa, "callback_hfa"),
        (large, "callback_large"),
        (union, "callback_union"),
    ];
    let signatures = shapes.map(|(ty, _)| signature(&mut registry, vec![ty], Some(ty), false));
    let check = signature(&mut registry, vec![], Some(int), false);
    let types = registry.freeze().unwrap();
    let context = Context::create();
    let target = abi::NativeTarget::new().unwrap();
    let module = context.create_module("c.generated.callbacks");
    module.set_triple(&target.triple);
    module.set_data_layout(&target.data.get_data_layout());
    let mut lowerer = TypeLowerer::with_target(&context, &types, &target.data);
    for ((ty, symbol), signature) in shapes.into_iter().zip(signatures) {
        let function = foreign::declare(
            &module,
            &types,
            &mut lowerer,
            target.c_platform().unwrap(),
            &target.data,
            signature,
            symbol,
        )
        .unwrap();
        let builder = context.create_builder();
        builder.position_at_end(context.append_basic_block(function.value, "entry"));
        let incoming = foreign::parameters(
            &builder,
            &types,
            &mut lowerer,
            &target.data,
            &function.signature,
            function.value,
        )
        .unwrap();
        assert_eq!(incoming.len(), 1);
        assert_eq!(incoming[0].get_type(), lowerer.basic(ty).unwrap());
        foreign::return_value(
            &builder,
            &context,
            &target.data,
            &function.signature,
            function.value,
            Some(incoming[0]),
        )
        .unwrap();
    }
    let main = module.add_function("main", context.i32_type().fn_type(&[], false), None);
    let builder = context.create_builder();
    builder.position_at_end(context.append_basic_block(main, "entry"));
    let check = foreign::declare(
        &module,
        &types,
        &mut lowerer,
        target.c_platform().unwrap(),
        &target.data,
        check,
        "check_callbacks",
    )
    .unwrap();
    let result = check
        .call(&builder, &types, &mut lowerer, &target.data, &[])
        .unwrap()
        .value
        .unwrap();
    builder.build_return(Some(&result)).unwrap();
    assert_eq!(
        execute(
            &module,
            r#"
#include <stdint.h>
struct Pair { int64_t a,b; }; struct Hfa { float a,b,c; }; struct Large { int64_t a,b,c; }; union Union { int64_t a; double b; };
extern struct Pair callback_pair(struct Pair); extern struct Hfa callback_hfa(struct Hfa); extern struct Large callback_large(struct Large); extern union Union callback_union(union Union);
int check_callbacks(void) {
 struct Pair p=callback_pair((struct Pair){11,22}); struct Hfa f=callback_hfa((struct Hfa){1.25f,2.5f,3.75f}); struct Large l=callback_large((struct Large){33,44,55}); union Union u; u.a=66; u=callback_union(u);
 return !(p.a==11 && p.b==22 && f.a==1.25f && f.b==2.5f && f.c==3.75f && l.a==33 && l.b==44 && l.c==55 && u.a==66);
}
"#
        ),
        0
    );
}

#[test]
fn classification_skips_huge_zero_sized_arrays_and_compacts_shared_union_graphs() {
    let context = Context::create();
    let target = abi::NativeTarget::new().unwrap();
    let mut registry = TypeRegistry::new();
    let double = registry.float(FloatType::F64);
    let empty = record(&mut registry, &[], RecordKind::Struct);
    let huge = registry.fixed_array(empty, u64::from(u32::MAX)).unwrap();
    let wrapper = record(&mut registry, &[huge, double], RecordKind::Struct);
    let mut repeated = double;
    for _ in 0..256 {
        repeated = record(&mut registry, &[repeated, repeated], RecordKind::Union);
    }
    let array_signature = signature(&mut registry, vec![wrapper], Some(wrapper), false);
    let union_signature = signature(&mut registry, vec![repeated], Some(repeated), false);
    let types = registry.freeze().unwrap();
    let mut lowerer = TypeLowerer::with_target(&context, &types, &target.data);
    for signature in [array_signature, union_signature] {
        let signature = abi::Signature::classify(
            &context,
            &types,
            &mut lowerer,
            target.c_platform().unwrap(),
            &target.data,
            signature,
            false,
        )
        .unwrap();
        assert!(matches!(
            signature.parameters.as_slice(),
            [abi::Value::Coerce { .. }]
        ));
        assert!(matches!(signature.result, abi::Value::Coerce { .. }));
        let module = context.create_module("compact.abi");
        let function = module.add_function("fixture", signature.llvm, None);
        signature.attributes_on_function(function);
        module.verify().unwrap();
    }
}

#[test]
fn universal_any_descriptor_roundtrips_through_actual_c_argument_and_result_abi() {
    let mut registry = TypeRegistry::new();
    let integer = registry.scalar(ScalarType::Int(IntegerType::S64));
    let header = record(&mut registry, &[integer], RecordKind::Struct);
    let any = registry.reserve_any();
    registry.define_any(any, header).unwrap();
    let prototype = signature(&mut registry, vec![any], Some(any), false);
    let types = registry.freeze().unwrap();
    let context = Context::create();
    let target = abi::NativeTarget::new().unwrap();
    let module = context.create_module("foreign.any");
    module.set_triple(&target.triple);
    module.set_data_layout(&target.data.get_data_layout());
    let mut lowerer = TypeLowerer::with_target(&context, &types, &target.data);
    let builder = context.create_builder();
    let main = module.add_function("main", context.i32_type().fn_type(&[], false), None);
    builder.position_at_end(context.append_basic_block(main, "entry"));
    let descriptor = builder
        .build_alloca(lowerer.basic(header).unwrap(), "header")
        .unwrap();
    builder
        .build_store(
            descriptor,
            aggregate(
                header,
                vec![context.i64_type().const_int(77, false).into()],
                &mut lowerer,
            ),
        )
        .unwrap();
    let payload = builder.build_alloca(context.i64_type(), "payload").unwrap();
    builder
        .build_store(payload, context.i64_type().const_int(41, false))
        .unwrap();
    let storage = lowerer.basic(any).unwrap().into_struct_type();
    let argument = builder
        .build_insert_value(storage.get_undef(), descriptor, 0, "any.type")
        .unwrap();
    let argument = builder
        .build_insert_value(argument, payload, 1, "any.payload")
        .unwrap()
        .into_struct_value();
    let function = foreign::declare(
        &module,
        &types,
        &mut lowerer,
        target.c_platform().unwrap(),
        &target.data,
        prototype,
        "any_roundtrip",
    )
    .unwrap();
    let returned = function
        .call(
            &builder,
            &types,
            &mut lowerer,
            &target.data,
            &[(any, argument.into())],
        )
        .unwrap()
        .value
        .unwrap()
        .into_struct_value();
    let returned_descriptor = builder
        .build_extract_value(returned, 0, "returned.type")
        .unwrap()
        .into_pointer_value();
    let returned_payload = builder
        .build_extract_value(returned, 1, "returned.payload")
        .unwrap()
        .into_pointer_value();
    let pointer_type = context.ptr_sized_int_type(&target.data, None);
    let equal_type = builder
        .build_int_compare(
            inkwell::IntPredicate::EQ,
            builder
                .build_ptr_to_int(returned_descriptor, pointer_type, "returned.type.address")
                .unwrap(),
            builder
                .build_ptr_to_int(descriptor, pointer_type, "original.type.address")
                .unwrap(),
            "type.same",
        )
        .unwrap();
    let equal_payload = builder
        .build_int_compare(
            inkwell::IntPredicate::EQ,
            builder
                .build_ptr_to_int(returned_payload, pointer_type, "returned.payload.address")
                .unwrap(),
            builder
                .build_ptr_to_int(payload, pointer_type, "original.payload.address")
                .unwrap(),
            "payload.same",
        )
        .unwrap();
    let mutated = builder
        .build_load(context.i64_type(), payload, "mutated")
        .unwrap()
        .into_int_value();
    let changed = builder
        .build_int_compare(
            inkwell::IntPredicate::EQ,
            mutated,
            context.i64_type().const_int(42, false),
            "payload.changed",
        )
        .unwrap();
    let passed = builder
        .build_and(
            builder
                .build_and(equal_type, equal_payload, "descriptor.same")
                .unwrap(),
            changed,
            "checks",
        )
        .unwrap();
    let result = builder
        .build_select(
            passed,
            context.i32_type().const_zero(),
            context.i32_type().const_int(1, false),
            "exit",
        )
        .unwrap();
    builder.build_return(Some(&result)).unwrap();
    assert_eq!(
        execute(
            &module,
            r#"
#include <stdint.h>
struct Any { void *type; void *value_pointer; };
struct Any any_roundtrip(struct Any value) {
    if (!value.type || *(int64_t *)value.type != 77) { value.type = 0; return value; }
    ++*(int64_t *)value.value_pointer;
    return value;
}
"#
        ),
        0
    );
}
