//! Header-free Clang++ proves receiver-first scalar method carriers on each target.
#[path = "support/native_tools.rs"]
mod native_tools;
use inkwell::{
    attributes::{Attribute, AttributeLoc},
    context::Context,
    memory_buffer::MemoryBuffer,
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
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "jai-cpp-target-{}-{}",
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
fn extensions(
    function: inkwell::values::FunctionValue<'_>,
    location: AttributeLoc,
) -> Vec<&'static str> {
    ["signext", "zeroext"]
        .into_iter()
        .filter(|name| {
            function
                .get_enum_attribute(location, Attribute::get_named_enum_kind_id(name))
                .is_some()
        })
        .collect()
}
#[test]
fn canonical_receiver_first_method_shape_matches_clang22() {
    let scratch = Scratch::new();
    let cpp = scratch.0.join("oracle.cpp");
    let llvm = scratch.0.join("oracle.ll");
    fs::write(&cpp,"struct Receiver { long long value; long long calculate(signed char, unsigned char, bool, float); }; long long Receiver::calculate(signed char a,unsigned char b,bool c,float d){return value+a+b+c+(long long)d;}").unwrap();
    let mut types = TypeRegistry::new();
    let void = types.void();
    let receiver = types.pointer(void).unwrap();
    let s8 = types.scalar(ScalarType::Int(IntegerType::S8));
    let u8 = types.scalar(ScalarType::Int(IntegerType::U8));
    let boolean = types.scalar(ScalarType::Bool);
    let float = types.float(FloatType::F32);
    let result = types.scalar(ScalarType::Int(IntegerType::S64));
    let signature = types
        .procedure(ProcedureType {
            parameters: Box::new([receiver, s8, u8, boolean, float]),
            results: Box::new([result]),
            convention: CallingConvention::CppMethod,
            context: ContextMode::None,
            variadic: Variadic::None,
        })
        .unwrap();
    let types = types.freeze().unwrap();
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
        let output = native_tools::clang_command()
            .arg(format!("--target={triple}"))
            .args([
                "-x",
                "c++",
                "-S",
                "-emit-llvm",
                "-O0",
                "-ffreestanding",
                "-nostdinc",
                "-nostdinc++",
            ])
            .arg(&cpp)
            .arg("-o")
            .arg(&llvm)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{triple}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let context = Context::create();
        let oracle = context
            .create_module_from_ir(MemoryBuffer::create_from_file(&llvm).unwrap())
            .unwrap();
        oracle.verify().unwrap();
        let expected = oracle
            .get_functions()
            .find(|function| function.get_name().to_string_lossy().contains("calculate"))
            .unwrap();
        let target = NativeTarget::select(&TargetOptions {
            selection: TargetSelection::Triple(Triple::new(triple).unwrap()),
            ..Default::default()
        })
        .unwrap();
        let mut lowerer = TypeLowerer::with_target(&context, &types, &target.data);
        let module = context.create_module("cpp.target.abi");
        let actual = foreign::declare(
            &module,
            &types,
            &mut lowerer,
            target.c_platform().unwrap(),
            &target.data,
            signature,
            "generated_method",
        )
        .unwrap()
        .value;
        assert_eq!(
            actual.get_call_conventions(),
            expected.get_call_conventions(),
            "{triple}"
        );
        assert_eq!(actual.get_type(), expected.get_type(), "{triple}");
        for location in std::iter::once(AttributeLoc::Return)
            .chain((0..expected.count_params()).map(AttributeLoc::Param))
        {
            assert_eq!(
                extensions(actual, location),
                extensions(expected, location),
                "{triple}: {location:?}"
            );
        }
        assert_eq!(
            actual.count_params(),
            5,
            "Jai context must not be hidden in a C++ method"
        );
        module.verify().unwrap();
    }
}
