//! Internal callback object emission is independent of foreign ABI availability.
use jai_codegen::target::{NativeTarget, TargetOptions, TargetSelection, Triple};
use jai_modules::{GraphOptions, ModuleGraph, SourceOverlay};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

fn program(source: &str, target: &NativeTarget) -> jai_ir::Program {
    let mut sources = SourceOverlay::new();
    let path = Path::new("/jai-call-abi/main.jai");
    sources.insert(path, source.as_bytes().to_vec()).unwrap();
    let graph = ModuleGraph::load_with_provider(path, GraphOptions::default(), &sources).unwrap();
    jai_sema::resolve_graph_with_options(
        &graph,
        &jai_sema::ResolveOptions {
            target: Some(target.build_target().unwrap()),
            layout: Some(target.layout_policy().unwrap()),
            ..Default::default()
        },
        &mut jai_vm::NoEffects,
    )
    .unwrap()
}
fn target(triple: &str) -> NativeTarget {
    NativeTarget::select(&TargetOptions {
        selection: TargetSelection::Triple(Triple::new(triple).unwrap()),
        ..Default::default()
    })
    .unwrap()
}
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "jai-internal-call-abi-{}-{}",
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

#[test]
fn scalar_aggregate_and_multiple_result_jai_callbacks_emit_on_i686() {
    let scratch = Scratch::new();
    for triple in ["i686-unknown-linux-gnu", "x86_64-unknown-linux-gnu"] {
        let target = target(triple);
        if triple.starts_with("i686-") {
            assert!(matches!(
                target.c_platform(),
                Err(jai_codegen::abi::Error::UnsupportedTarget(_))
            ));
        }
        for (name, source) in [
            (
                "scalar",
                "answer::(value:int)->int #no_context{return value;}main::()->int{callback:=answer;return callback(42);}",
            ),
            (
                "aggregate",
                "Pair::struct{left:int;right:int;}answer::(value:Pair)->Pair #no_context{return value;}main::()->int{callback:=answer;value:Pair=callback(.{left=20,right=22});return value.left+value.right;}",
            ),
            (
                "multiple",
                "answer::(value:int)->(int,int) #no_context{return value,2;}main::()->int{callback:=answer;left,right:=callback(40);return left+right;}",
            ),
        ] {
            let program = program(source, &target);
            let context = jai_codegen::Context::create();
            let module = jai_codegen::lower_for_target(&context, &program, &target).unwrap();
            module.verify().unwrap();
            let text = module.print_to_string().to_string();
            assert!(
                text.lines().any(|line| line
                    .split_once("call ")
                    .is_some_and(|(_, call)| call.contains(" %") && !call.contains('@'))),
                "a real loaded callback call is required: {text}"
            );
            assert!(
                !text.contains("byval("),
                "internal arguments must retain the Jai signature: {text}"
            );
            let object = scratch.0.join(format!("{triple}-{name}.o"));
            target.write_object(&module, &object).unwrap();
            let bytes = fs::read(object).unwrap();
            assert!(bytes.len() > 64);
            assert_eq!(&bytes[..4], b"\x7fELF");
            assert_eq!(
                bytes[4],
                if triple.starts_with("i686-") {
                    1
                } else {
                    2
                }
            );
            let machine = u16::from_le_bytes([bytes[18], bytes[19]]);
            assert_eq!(
                machine,
                if triple.starts_with("i686-") {
                    3
                } else {
                    62
                }
            );
        }
    }
}

#[test]
fn c_callback_still_requires_its_real_foreign_target_classifier() {
    let source = "Callback::#type(value:int)->int #c_call;invoke::(callback:Callback)->int #no_context{return callback(42);}main::()->int{return invoke(null);}";
    let unsupported = target("i686-unknown-linux-gnu");
    let checked = program(source, &unsupported);
    let context = jai_codegen::Context::create();
    let error = jai_codegen::lower_for_target(&context, &checked, &unsupported).unwrap_err();
    assert!(
        matches!(error, jai_codegen::Error::Foreign(jai_codegen::abi::Error::UnsupportedTarget(ref triple)) if triple == "i686-unknown-linux-gnu"),
        "{error}"
    );
    let supported = target("x86_64-unknown-linux-gnu");
    let checked = program(source, &supported);
    let context = jai_codegen::Context::create();
    let module = jai_codegen::lower_for_target(&context, &checked, &supported).unwrap();
    module.verify().unwrap();
    // This shape test emits the call without executing its null callback.
    assert!(module.print_to_string().to_string().contains("call i64 %"));
}

#[test]
fn indirect_non_pod_result_keeps_the_explicit_microsoft_environment_guard() {
    let source = "Pair::struct{left:int;right:int;}Callback::#type(value:int)->Pair #c_call #cpp_return_type_is_non_pod;invoke::(callback:Callback)->Pair #no_context{return callback(42);}main::()->int{value:=invoke(null);return value.left;}";
    let unsupported = target("x86_64-w64-windows-gnu");
    let checked = program(source, &unsupported);
    let context = jai_codegen::Context::create();
    let error = jai_codegen::lower_for_target(&context, &checked, &unsupported).unwrap_err();
    assert!(
        matches!(error, jai_codegen::Error::Foreign(jai_codegen::abi::Error::UnsupportedTarget(ref message)) if message.contains("explicit msvc environment")),
        "{error}"
    );
    let supported = target("x86_64-pc-windows-msvc");
    let checked = program(source, &supported);
    let context = jai_codegen::Context::create();
    let module = jai_codegen::lower_for_target(&context, &checked, &supported).unwrap();
    module.verify().unwrap();
    assert!(module.print_to_string().to_string().contains("sret("));
}
