//! Canonical target carrier and ABI attributes match freshly compiled header-free C.
#[path = "support/abi_carriers.rs"]
mod abi_carriers;
#[path = "support/native_tools.rs"]
mod native_tools;
use abi_carriers::function_shape;
use inkwell::{
    attributes::{Attribute, AttributeLoc},
    context::Context,
    memory_buffer::MemoryBuffer,
    values::FunctionValue,
};
use jai_codegen::{
    foreign,
    target::{NativeTarget, TargetOptions, TargetSelection, Triple},
    types::TypeLowerer,
};
use jai_types::*;
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

const SOURCE: &str = r#"
typedef unsigned char u8; typedef signed char s8; typedef unsigned long long u64;
struct Small { u8 a; }; struct Pair { u64 a,b; };
struct Hfa { float a,b; }; struct __attribute__((aligned(16))) PaddedHfa { float a,b; };
struct __attribute__((aligned(16))) Aligned { u64 a,b; };
struct Pointer { void *a; }; union PointerUnion { void *a; u64 *b; };
struct Pointers { void *a,*b; }; struct Big { u64 a,b,c; };
struct SingleFloat { float x; }; struct SingleDouble { double x; };
struct TripleFloat { float a,b,c; }; struct Nested { struct SingleFloat x; };
struct __attribute__((aligned(16))) PadSingle { float x; };
struct OneArray { float x[1]; }; union SamePointer { void *a,*b; };
u8 unsigned_narrow(u8 x){return x;} s8 signed_narrow(s8 x){return x;}
_Bool boolean(_Bool x){return x;}
#define ID(T,N) struct T N(struct T x){return x;}
ID(Small,small) ID(Pair,pair) ID(Hfa,hfa) ID(PaddedHfa,padded_hfa)
ID(Aligned,aligned) ID(Pointer,pointer) ID(Pointers,pointers) ID(Big,big)
union PointerUnion pointer_union(union PointerUnion x){return x;}
struct Pair exhausted(u64 a,u64 b,u64 c,u64 d,u64 e,u64 f,struct Pair x){return x;}
ID(SingleFloat,single_float) ID(SingleDouble,single_double) ID(TripleFloat,triple_float)
ID(Nested,nested) ID(PadSingle,padded_single) ID(OneArray,one_array)
union SamePointer same_pointer(union SamePointer x){return x;}
struct Hfa variadic_hfa(struct Hfa x,...){return x;}
"#;
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "jai-desktop-abi-{}-{}",
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
fn record(types: &mut TypeRegistry, fields: Vec<TypeId>, alignment: Option<u32>) -> TypeId {
    let ty = types.reserve_record(RecordKind::Struct);
    types
        .define_record_with_layout(
            ty,
            fields,
            RecordLayout {
                minimum_alignment: alignment,
                ..Default::default()
            },
        )
        .unwrap();
    ty
}
fn cases() -> (Types, Vec<(&'static str, TypeId)>) {
    let mut types = TypeRegistry::new();
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let signed = types.scalar(ScalarType::Int(IntegerType::S8));
    let boolean = types.scalar(ScalarType::Bool);
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let float = types.float(FloatType::F32);
    let double = types.float(FloatType::F64);
    let void = types.void();
    let pointer = types.pointer(void).unwrap();
    let small = record(&mut types, vec![byte], None);
    let pair = record(&mut types, vec![word, word], None);
    let hfa = record(&mut types, vec![float, float], None);
    let padded_hfa = record(&mut types, vec![float, float], Some(16));
    let aligned = record(&mut types, vec![word, word], Some(16));
    let single_pointer = record(&mut types, vec![pointer], None);
    let word_pointer = types.pointer(word).unwrap();
    let pointer_union = types.reserve_record(RecordKind::Union);
    types
        .define_record(pointer_union, vec![pointer, word_pointer])
        .unwrap();
    let pointers = record(&mut types, vec![pointer, pointer], None);
    let big = record(&mut types, vec![word, word, word], None);
    let single_float = record(&mut types, vec![float], None);
    let single_double = record(&mut types, vec![double], None);
    let triple_float = record(&mut types, vec![float, float, float], None);
    let nested = record(&mut types, vec![single_float], None);
    let padded_single = record(&mut types, vec![float], Some(16));
    let array = types.fixed_array(float, 1).unwrap();
    let one_array = record(&mut types, vec![array], None);
    let same_pointer = types.reserve_record(RecordKind::Union);
    types
        .define_record(same_pointer, vec![pointer, pointer])
        .unwrap();
    let mut result = Vec::new();
    for (name, ty) in [
        ("unsigned_narrow", byte),
        ("signed_narrow", signed),
        ("boolean", boolean),
        ("small", small),
        ("pair", pair),
        ("hfa", hfa),
        ("padded_hfa", padded_hfa),
        ("aligned", aligned),
        ("pointer", single_pointer),
        ("pointer_union", pointer_union),
        ("pointers", pointers),
        ("big", big),
        ("exhausted", pair),
        ("single_float", single_float),
        ("single_double", single_double),
        ("triple_float", triple_float),
        ("nested", nested),
        ("padded_single", padded_single),
        ("one_array", one_array),
        ("same_pointer", same_pointer),
        ("variadic_hfa", hfa),
    ] {
        let mut parameters = if name == "exhausted" {
            vec![word; 6]
        } else {
            vec![]
        };
        parameters.push(ty);
        let signature = types
            .procedure(ProcedureType {
                parameters: parameters.into(),
                results: Box::new([ty]),
                return_abi: jai_types::ForeignReturnAbi::Natural,
                convention: CallingConvention::C,
                context: ContextMode::None,
                variadic: if name == "variadic_hfa" {
                    Variadic::C {
                        fixed_parameters: 1,
                    }
                } else {
                    Variadic::None
                },
            })
            .unwrap();
        result.push((name, signature));
    }
    (types.freeze().unwrap(), result)
}
fn attributes(function: FunctionValue<'_>, location: AttributeLoc) -> Vec<(&'static str, u64)> {
    ["sret", "byval", "align", "alignstack", "signext", "zeroext"]
        .into_iter()
        .filter_map(|name| {
            function
                .get_enum_attribute(location, Attribute::get_named_enum_kind_id(name))
                .map(|attribute| {
                    (
                        name,
                        if attribute.is_enum() {
                            attribute.get_enum_value()
                        } else {
                            0
                        },
                    )
                })
        })
        .collect()
}
fn check_shapes(triples: &[&str]) {
    let scratch = Scratch::new();
    let c = scratch.0.join("oracle.c");
    let llvm = scratch.0.join("oracle.ll");
    fs::write(&c, SOURCE).unwrap();
    let (types, cases) = cases();
    for &triple in triples {
        let output = native_tools::clang_command()
            .arg(format!("--target={triple}"))
            .args(["-S", "-emit-llvm", "-O0", "-ffreestanding"])
            .arg(&c)
            .arg("-o")
            .arg(&llvm)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
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
        let module = context.create_module("desktop.abi");
        for &(name, signature) in &cases {
            let actual = foreign::declare(
                &module,
                &types,
                &mut lowerer,
                target.c_platform().unwrap(),
                &target.data,
                signature,
                name,
            )
            .unwrap()
            .value;
            let expected = oracle.get_function(name).unwrap();
            assert_eq!(
                function_shape(actual.get_type()),
                function_shape(expected.get_type()),
                "{triple}: {name}"
            );
            for location in std::iter::once(AttributeLoc::Return)
                .chain((0..expected.count_params()).map(AttributeLoc::Param))
            {
                assert_eq!(
                    attributes(actual, location),
                    attributes(expected, location),
                    "{triple}: {name} {location:?}"
                );
            }
        }
        module.verify().unwrap();
    }
}

#[test]
fn four_desktop_scalar_aggregate_and_exhaustion_carriers_match_clang22() {
    check_shapes(&[
        "aarch64-apple-darwin",
        "x86_64-apple-darwin",
        "aarch64-unknown-linux-gnu",
        "x86_64-unknown-linux-gnu",
    ]);
}
#[test]
fn windows_wasm_and_mobile_canonical_carriers_match_clang22() {
    check_shapes(&[
        "x86_64-pc-windows-msvc",
        "aarch64-pc-windows-msvc",
        "wasm32-unknown-unknown",
        "wasm64-unknown-unknown",
        "aarch64-linux-android",
        "arm64-apple-ios",
    ]);
}
