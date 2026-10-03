//! Independent trusted C definitions exercise both directions of custom C ABI.
#[path = "support/abi_carriers.rs"]
mod abi_carriers;
#[path = "support/native_tools.rs"]
mod native_tools;
#[path = "support/object_headers.rs"]
mod object_headers;
use abi_carriers::function_shape;
use inkwell::{
    attributes::{Attribute, AttributeLoc},
    context::Context,
    memory_buffer::MemoryBuffer,
    values::{BasicValueEnum, FunctionValue},
};
use jai_codegen::{
    foreign,
    optimization::Optimization,
    target::{NativeTarget, TargetOptions, TargetSelection, Triple},
    types::TypeLowerer,
};
use jai_types::*;
use std::{
    fs,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

const C_SOURCE: &str = r#"
typedef unsigned char u8; typedef unsigned long long u64;
struct __attribute__((packed)) Packed { u8 tag; u64 value; };
struct __attribute__((packed,aligned(4))) Reduced { u8 tag; u64 value __attribute__((aligned(4))); };
struct __attribute__((aligned(32))) Aligned { u64 a,b; };
struct __attribute__((packed)) PackedHfa { float a,b; };
struct __attribute__((aligned(16))) PaddedHfa { float a,b; };
struct __attribute__((packed)) Nested { u8 tag; struct Packed child; };
struct Array { struct Packed values[2]; };
struct __attribute__((packed)) PackedSingle { u64 value; };
struct __attribute__((aligned(16))) PaddedInteger { u64 value; };
struct SingleFloat { float value; }; struct NestedFloat { struct SingleFloat child; };
#define LAYOUT(T,S,A,F,O) _Static_assert(sizeof(struct T)==S,"size");_Static_assert(_Alignof(struct T)==A,"alignment");_Static_assert(__builtin_offsetof(struct T,F)==O,"offset");
LAYOUT(Packed,9,1,value,1) LAYOUT(Reduced,12,4,value,4) LAYOUT(Aligned,32,32,b,8)
LAYOUT(PackedHfa,8,1,b,4) LAYOUT(PaddedHfa,16,16,b,4) LAYOUT(Nested,10,1,child,1) LAYOUT(Array,18,1,values,0)
LAYOUT(PackedSingle,8,1,value,0) LAYOUT(PaddedInteger,16,16,value,0) LAYOUT(NestedFloat,4,4,child,0)
struct Packed mutate_packed(struct Packed x){x.tag+=2;x.value+=10;return x;}
struct Reduced mutate_reduced(struct Reduced x){x.tag+=2;x.value+=10;return x;}
struct Aligned mutate_aligned(struct Aligned x){x.a+=2;x.b+=10;return x;}
struct PackedHfa mutate_packed_hfa(struct PackedHfa x){x.a+=2;x.b+=10;return x;}
struct PaddedHfa mutate_padded_hfa(struct PaddedHfa x){x.a+=2;x.b+=10;return x;}
struct Nested mutate_nested(struct Nested x){x.tag+=2;x.child.tag+=2;x.child.value+=10;return x;}
struct Array mutate_array(struct Array x){x.values[0].tag+=2;x.values[0].value+=10;x.values[1].tag+=2;x.values[1].value+=10;return x;}
struct PackedSingle mutate_packed_single(struct PackedSingle x){x.value+=10;return x;}
struct PaddedInteger mutate_padded_integer(struct PaddedInteger x){x.value+=10;return x;}
struct NestedFloat mutate_nested_float(struct NestedFloat x){x.child.value+=10;return x;}
int check_nested_float(struct NestedFloat x){return x.child.value==30?42:10;}
int check_packed(struct Packed x){return x.tag==9 && x.value==30 ? 42:1;}
int check_reduced(struct Reduced x){return x.tag==9 && x.value==30 ? 42:2;}
int check_aligned(struct Aligned x){return x.a==9 && x.b==30 ? 42:3;}
int check_packed_hfa(struct PackedHfa x){return x.a==9 && x.b==30 ? 42:4;}
int check_padded_hfa(struct PaddedHfa x){return x.a==9 && x.b==30 ? 42:5;}
int check_nested(struct Nested x){return x.tag==9 && x.child.tag==5 && x.child.value==30 ? 42:6;}
int check_array(struct Array x){return x.values[0].tag==9 && x.values[0].value==30 && x.values[1].tag==5 && x.values[1].value==20 ? 42:7;}
int check_packed_single(struct PackedSingle x){return x.value==30 ? 42:8;}
int check_padded_integer(struct PaddedInteger x){return x.value==30 ? 42:9;}
extern struct Packed generated_packed(struct Packed);
extern struct Reduced generated_reduced(struct Reduced);
extern struct Aligned generated_aligned(struct Aligned);
extern struct PackedHfa generated_packed_hfa(struct PackedHfa);
extern struct PaddedHfa generated_padded_hfa(struct PaddedHfa);
extern struct Nested generated_nested(struct Nested);
extern struct Array generated_array(struct Array);
extern struct PackedSingle generated_packed_single(struct PackedSingle);
extern struct PaddedInteger generated_padded_integer(struct PaddedInteger);
extern struct NestedFloat generated_nested_float(struct NestedFloat);
int drive_generated(void) {
 if(check_packed(generated_packed((struct Packed){7,20}))!=42)return 1;
 if(check_reduced(generated_reduced((struct Reduced){7,20}))!=42)return 2;
 if(check_aligned(generated_aligned((struct Aligned){7,20}))!=42)return 3;
 if(check_packed_hfa(generated_packed_hfa((struct PackedHfa){7,20}))!=42)return 4;
 if(check_padded_hfa(generated_padded_hfa((struct PaddedHfa){7,20}))!=42)return 5;
 if(check_nested(generated_nested((struct Nested){7,{3,20}}))!=42)return 6;
 if(check_array(generated_array((struct Array){{{7,20},{3,10}}}))!=42)return 7;
 if(check_packed_single(generated_packed_single((struct PackedSingle){20}))!=42)return 8;
 if(check_padded_integer(generated_padded_integer((struct PaddedInteger){20}))!=42)return 9;
 if(check_nested_float(generated_nested_float((struct NestedFloat){{20}}))!=42)return 10;
 return 42;
}
"#;

#[derive(Clone)]
enum Literal {
    Int(u64),
    Float(f64),
    Fields(Vec<Self>),
}
struct Case {
    name: &'static str,
    ty: TypeId,
    transform: TypeId,
    check: TypeId,
    input: Literal,
    expected: (u64, u32, Vec<u64>),
}
fn signature(registry: &mut TypeRegistry, parameters: Vec<TypeId>, result: TypeId) -> TypeId {
    registry
        .procedure(ProcedureType {
            parameters: parameters.into(),
            results: Box::new([result]),
            return_abi: jai_types::ForeignReturnAbi::Natural,
            convention: CallingConvention::C,
            context: ContextMode::None,
            variadic: Variadic::None,
        })
        .unwrap()
}
fn cases() -> (Types, Vec<Case>, TypeId) {
    use Literal::{Fields as R, Float as F, Int as I};
    let mut types = TypeRegistry::new();
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let int = types.scalar(ScalarType::Int(IntegerType::S32));
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let float = types.float(FloatType::F32);
    let packed = types.reserve_record(RecordKind::Struct);
    types
        .define_record_with_layout(
            packed,
            [byte, word],
            RecordLayout {
                packed: true,
                ..Default::default()
            },
        )
        .unwrap();
    let reduced = types.reserve_record(RecordKind::Struct);
    types
        .define_record_with_layout(
            reduced,
            [byte, word],
            RecordLayout {
                packed: true,
                field_alignments: Box::new([None, Some(4)]),
                ..Default::default()
            },
        )
        .unwrap();
    let aligned = types.reserve_record(RecordKind::Struct);
    types
        .define_record_with_layout(
            aligned,
            [word, word],
            RecordLayout {
                minimum_alignment: Some(32),
                ..Default::default()
            },
        )
        .unwrap();
    let packed_hfa = types.reserve_record(RecordKind::Struct);
    types
        .define_record_with_layout(
            packed_hfa,
            [float, float],
            RecordLayout {
                packed: true,
                ..Default::default()
            },
        )
        .unwrap();
    let padded_hfa = types.reserve_record(RecordKind::Struct);
    types
        .define_record_with_layout(
            padded_hfa,
            [float, float],
            RecordLayout {
                minimum_alignment: Some(16),
                ..Default::default()
            },
        )
        .unwrap();
    let nested = types.reserve_record(RecordKind::Struct);
    types
        .define_record_with_layout(
            nested,
            [byte, packed],
            RecordLayout {
                packed: true,
                ..Default::default()
            },
        )
        .unwrap();
    let elements = types.fixed_array(packed, 2).unwrap();
    let array = types.reserve_record(RecordKind::Struct);
    types.define_record(array, [elements]).unwrap();
    let packed_single = types.reserve_record(RecordKind::Struct);
    types
        .define_record_with_layout(
            packed_single,
            [word],
            RecordLayout {
                packed: true,
                ..Default::default()
            },
        )
        .unwrap();
    let padded_integer = types.reserve_record(RecordKind::Struct);
    types
        .define_record_with_layout(
            padded_integer,
            [word],
            RecordLayout {
                minimum_alignment: Some(16),
                ..Default::default()
            },
        )
        .unwrap();
    let single_float = types.reserve_record(RecordKind::Struct);
    types.define_record(single_float, [float]).unwrap();
    let nested_float = types.reserve_record(RecordKind::Struct);
    types.define_record(nested_float, [single_float]).unwrap();
    let specs = [
        (
            "nested_float",
            nested_float,
            R(vec![R(vec![F(20.)])]),
            (4, 4, vec![0]),
        ),
        ("packed", packed, R(vec![I(7), I(20)]), (9, 1, vec![0, 1])),
        (
            "reduced",
            reduced,
            R(vec![I(7), I(20)]),
            (12, 4, vec![0, 4]),
        ),
        (
            "aligned",
            aligned,
            R(vec![I(7), I(20)]),
            (32, 32, vec![0, 8]),
        ),
        (
            "packed_hfa",
            packed_hfa,
            R(vec![F(7.), F(20.)]),
            (8, 1, vec![0, 4]),
        ),
        (
            "padded_hfa",
            padded_hfa,
            R(vec![F(7.), F(20.)]),
            (16, 16, vec![0, 4]),
        ),
        (
            "nested",
            nested,
            R(vec![I(7), R(vec![I(3), I(20)])]),
            (10, 1, vec![0, 1]),
        ),
        (
            "array",
            array,
            R(vec![R(vec![R(vec![I(7), I(20)]), R(vec![I(3), I(10)])])]),
            (18, 1, vec![0]),
        ),
        (
            "packed_single",
            packed_single,
            R(vec![I(20)]),
            (8, 1, vec![0]),
        ),
        (
            "padded_integer",
            padded_integer,
            R(vec![I(20)]),
            (16, 16, vec![0]),
        ),
    ];
    let cases = specs
        .into_iter()
        .map(|(name, ty, input, expected)| Case {
            name,
            ty,
            transform: signature(&mut types, vec![ty], ty),
            check: signature(&mut types, vec![ty], int),
            input,
            expected,
        })
        .collect();
    let driver = signature(&mut types, vec![], int);
    (types.freeze().unwrap(), cases, driver)
}
struct Scratch(std::path::PathBuf);
impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "jai-foreign-custom-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn compile(command: &mut Command) {
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
fn value<'ctx>(
    context: &'ctx Context,
    builder: &inkwell::builder::Builder<'ctx>,
    lowerer: &mut TypeLowerer<'ctx, '_>,
    target: &NativeTarget,
    ty: TypeId,
    literal: &Literal,
) -> BasicValueEnum<'ctx> {
    let storage = lowerer.basic(ty).unwrap();
    match literal {
        Literal::Int(value) => storage.into_int_type().const_int(*value, false).into(),
        Literal::Float(value) => storage.into_float_type().const_float(*value).into(),
        Literal::Fields(fields) => match *lowerer.registry().kind(ty).unwrap() {
            TypeKind::FixedArray {
                element, ..
            } => {
                let mut result = storage.into_array_type().const_zero();
                for (index, field) in fields.iter().enumerate() {
                    let field = value(context, builder, lowerer, target, element, field);
                    result = builder
                        .build_insert_value(result, field, index as u32, "array.field")
                        .unwrap()
                        .into_array_value();
                }
                result.into()
            }
            TypeKind::Record(record) => {
                let types = lowerer.registry();
                let declared = types.record(record).unwrap().fields.to_vec();
                let layout = LayoutEngine::new(types, target.layout_policy().unwrap())
                    .layout(ty)
                    .unwrap()
                    .clone();
                assert_eq!(fields.len(), declared.len());
                let pointer = builder.build_alloca(storage, "source.record").unwrap();
                builder.build_store(pointer, storage.const_zero()).unwrap();
                for ((field, ty), offset) in fields.iter().zip(declared).zip(layout.field_offsets) {
                    let field = value(context, builder, lowerer, target, ty, field);
                    let address = jai_llvm::gep(
                        builder,
                        context.i8_type().into(),
                        pointer,
                        &[context.i64_type().const_int(offset, false)],
                        "source.field",
                    )
                    .unwrap();
                    builder
                        .build_store(address, field)
                        .unwrap()
                        .set_alignment(1)
                        .unwrap();
                }
                builder
                    .build_load(storage, pointer, "source.value")
                    .unwrap()
            }
            _ => panic!("fixture literal is not an aggregate"),
        },
    }
}
fn adapter_module<'ctx>(
    context: &'ctx Context,
    types: &Types,
    cases: &[Case],
    driver: TypeId,
    target: &NativeTarget,
) -> inkwell::module::Module<'ctx> {
    let module = context.create_module("foreign.custom");
    module.set_triple(&target.triple);
    module.set_data_layout(&target.data.get_data_layout());
    let mut lowerer = TypeLowerer::with_target(context, types, &target.data);
    let builder = context.create_builder();
    for case in cases {
        let layout = lowerer.verify_layout(case.ty, &target.data).unwrap();
        assert_eq!(
            (
                layout.size,
                layout.alignment,
                layout.field_offsets.into_vec()
            ),
            case.expected
        );
        let function = foreign::declare(
            &module,
            types,
            &mut lowerer,
            target.c_platform().unwrap(),
            &target.data,
            case.transform,
            &format!("generated_{}", case.name),
        )
        .unwrap();
        builder.position_at_end(context.append_basic_block(function.value, "entry"));
        let arguments = foreign::parameters(
            &builder,
            types,
            &mut lowerer,
            &target.data,
            &function.signature,
            function.value,
        )
        .unwrap();
        let c_function = foreign::declare(
            &module,
            types,
            &mut lowerer,
            target.c_platform().unwrap(),
            &target.data,
            case.transform,
            &format!("mutate_{}", case.name),
        )
        .unwrap();
        let result = foreign::call(
            &builder,
            types,
            &mut lowerer,
            &target.data,
            &c_function.signature,
            foreign::Callee::Indirect(c_function.value.as_global_value().as_pointer_value()),
            &[(case.ty, arguments[0])],
        )
        .unwrap();
        foreign::return_value(
            &builder,
            context,
            &target.data,
            &function.signature,
            function.value,
            result.value,
        )
        .unwrap();
    }
    let main = module.add_function("main", context.i32_type().fn_type(&[], false), None);
    builder.position_at_end(context.append_basic_block(main, "entry"));
    let mut checks = vec![];
    for case in cases {
        let input = value(
            context,
            &builder,
            &mut lowerer,
            target,
            case.ty,
            &case.input,
        );
        let transform = foreign::declare(
            &module,
            types,
            &mut lowerer,
            target.c_platform().unwrap(),
            &target.data,
            case.transform,
            &format!("mutate_{}", case.name),
        )
        .unwrap();
        let result = transform
            .call(
                &builder,
                types,
                &mut lowerer,
                &target.data,
                &[(case.ty, input)],
            )
            .unwrap()
            .value
            .unwrap();
        let checker = foreign::declare(
            &module,
            types,
            &mut lowerer,
            target.c_platform().unwrap(),
            &target.data,
            case.check,
            &format!("check_{}", case.name),
        )
        .unwrap();
        let result = checker
            .call(
                &builder,
                types,
                &mut lowerer,
                &target.data,
                &[(case.ty, result)],
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
                    context.i32_type().const_int(42, false),
                    case.name,
                )
                .unwrap(),
        );
    }
    let driver = foreign::declare(
        &module,
        types,
        &mut lowerer,
        target.c_platform().unwrap(),
        &target.data,
        driver,
        "drive_generated",
    )
    .unwrap();
    let result = driver
        .call(&builder, types, &mut lowerer, &target.data, &[])
        .unwrap()
        .value
        .unwrap()
        .into_int_value();
    checks.push(
        builder
            .build_int_compare(
                inkwell::IntPredicate::EQ,
                result,
                context.i32_type().const_int(42, false),
                "reverse.ok",
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
            context.i32_type().const_int(42, false),
            context.i32_type().const_int(1, false),
            "exit",
        )
        .unwrap();
    builder.build_return(Some(&exit)).unwrap();
    module.verify().unwrap();
    module
}
#[test]
fn native_custom_records_call_and_return_in_both_c_directions() {
    let (types, cases, driver) = cases();
    let scratch = Scratch::new();
    let c = scratch.0.join("fixture.c");
    fs::write(&c, C_SOURCE).unwrap();
    for bitcode in [BitcodeOptimization::O0, BitcodeOptimization::O2] {
        let target = NativeTarget::select(&TargetOptions {
            optimization: Optimization {
                bitcode,
                ..Default::default()
            },
            ..Default::default()
        })
        .unwrap();
        let context = Context::create();
        let module = adapter_module(&context, &types, &cases, driver, &target);
        let object = scratch.0.join("generated.o");
        let executable = scratch.0.join("program");
        target.write_object(&module, &object).unwrap();
        let mut command = native_tools::clang_command();
        command
            .arg(&object)
            .arg(&c)
            .arg(if bitcode == BitcodeOptimization::O0 {
                "-O0"
            } else {
                "-O2"
            })
            .arg("-o")
            .arg(&executable);
        if cfg!(target_os = "macos") {
            command.arg("-Wl,-no_fixup_chains");
        }
        compile(&mut command);
        let mut child = Command::new(&executable).spawn().unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(status) = child.try_wait().unwrap() {
                assert_eq!(
                    status.code(),
                    Some(42),
                    "{bitcode:?}\n{}",
                    module.print_to_string()
                );
                break;
            }
            if Instant::now() >= deadline {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("generated C fixture timed out");
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }
}
/// Object emission exercises the parameter decoder, calls, and return encoder on
/// each target; cross-target code is never linked or executed on this host.
#[test]
fn canonical_custom_adapters_emit_objects_on_all_proven_targets() {
    let (types, cases, driver) = cases();
    let scratch = Scratch::new();
    for triple in [
        "aarch64-apple-darwin",
        "x86_64-apple-darwin",
        "aarch64-unknown-linux-gnu",
        "x86_64-unknown-linux-gnu",
        "x86_64-pc-windows-msvc",
        "aarch64-pc-windows-msvc",
        "wasm32-unknown-unknown",
        "wasm64-unknown-unknown",
        "aarch64-linux-android",
        "arm64-apple-ios",
    ] {
        for bitcode in [BitcodeOptimization::O0, BitcodeOptimization::O2] {
            let target = NativeTarget::select(&TargetOptions {
                selection: TargetSelection::Triple(Triple::new(triple).unwrap()),
                optimization: Optimization {
                    bitcode,
                    ..Default::default()
                },
                ..Default::default()
            })
            .unwrap();
            let context = Context::create();
            let module = adapter_module(&context, &types, &cases, driver, &target);
            let object = scratch.0.join("adapters.o");
            target.write_object(&module, &object).unwrap();
            object_headers::check(&fs::read(&object).unwrap(), triple);
        }
    }
}
fn attributes(function: FunctionValue<'_>, index: u32) -> (bool, bool, Option<u64>) {
    let location = AttributeLoc::Param(index);
    let attribute =
        |name| function.get_enum_attribute(location, Attribute::get_named_enum_kind_id(name));
    (
        attribute("sret").is_some(),
        attribute("byval").is_some(),
        attribute("align").map(|attribute| attribute.get_enum_value()),
    )
}
#[test]
fn canonical_custom_carriers_and_memory_attributes_match_trusted_clang_oracle() {
    let scratch = Scratch::new();
    let c = scratch.0.join("oracle.c");
    let llvm = scratch.0.join("oracle.ll");
    fs::write(&c, C_SOURCE).unwrap();
    let (types, cases, _) = cases();
    for triple in [
        "aarch64-apple-darwin",
        "x86_64-apple-darwin",
        "aarch64-unknown-linux-gnu",
        "x86_64-unknown-linux-gnu",
        "x86_64-pc-windows-msvc",
        "aarch64-pc-windows-msvc",
        "wasm32-unknown-unknown",
        "wasm64-unknown-unknown",
        "aarch64-linux-android",
        "arm64-apple-ios",
    ] {
        compile(
            native_tools::clang_command()
                .arg(format!("--target={triple}"))
                .args(["-S", "-emit-llvm", "-O0", "-ffreestanding"])
                .arg(&c)
                .arg("-o")
                .arg(&llvm),
        );
        let context = Context::create();
        let oracle = context
            .create_module_from_ir(MemoryBuffer::create_from_file(&llvm).unwrap())
            .unwrap();
        oracle.verify().unwrap();
        let target = NativeTarget::select(&TargetOptions {
            selection: TargetSelection::Triple(Triple::new(triple).unwrap()),
            ..Default::default()
        })
        .unwrap();
        let mut lowerer = TypeLowerer::with_target(&context, &types, &target.data);
        let module = context.create_module("classified.desktop");
        for case in &cases {
            let classified = foreign::declare(
                &module,
                &types,
                &mut lowerer,
                target.c_platform().unwrap(),
                &target.data,
                case.transform,
                &format!("mutate_{}", case.name),
            )
            .unwrap();
            let expected = oracle
                .get_function(&format!("mutate_{}", case.name))
                .unwrap();
            assert_eq!(
                function_shape(classified.value.get_type()),
                function_shape(expected.get_type()),
                "{}",
                case.name
            );
            for index in 0..expected.count_params() {
                assert_eq!(
                    attributes(classified.value, index),
                    attributes(expected, index),
                    "{} parameter {index}",
                    case.name
                );
            }
        }
        module.verify().unwrap();
    }
}
