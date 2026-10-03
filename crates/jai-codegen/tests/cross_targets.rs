//! Cross objects prove target emission; only host fixtures are executed elsewhere.
#[path = "support/object_headers.rs"]
mod object_headers;
use jai_codegen::{
    optimization::Optimization,
    target::{NativeTarget, TargetOptions, TargetSelection, Triple},
};
use jai_types::{Architecture as Arch, BitcodeOptimization, OperatingSystem as Os};
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
            "jai-cross-target-{}-{}",
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
#[derive(Clone, Copy)]
enum Object {
    Elf(u16),
    MachO(u32),
    Coff(u16),
    Wasm,
}
fn check_object(bytes: &[u8], kind: Object) {
    match kind {
        Object::Elf(machine) => {
            assert_eq!(&bytes[..4], b"\x7fELF");
            assert_eq!(bytes[4], 2);
            assert_eq!(bytes[5], 1);
            assert_eq!(
                u16::from_le_bytes(bytes[18..20].try_into().unwrap()),
                machine
            );
        }
        Object::MachO(cpu) => {
            assert_eq!(&bytes[..4], b"\xcf\xfa\xed\xfe");
            assert_eq!(u32::from_le_bytes(bytes[4..8].try_into().unwrap()), cpu);
        }
        Object::Coff(machine) => {
            assert_eq!(u16::from_le_bytes(bytes[..2].try_into().unwrap()), machine)
        }
        Object::Wasm => assert_eq!(&bytes[..8], b"\0asm\x01\0\0\0"),
    }
}
#[test]
fn source_context_and_aggregate_objects_follow_desktop_windows_wasm_and_mobile_targets() {
    let scratch = Scratch::new();
    let path = scratch.0.join("main.jai");
    fs::write(
        &path,
        r#"
      #add_context number: int = 1;
      Pair :: struct { a:int; b:int; }
      bump :: (p:*Pair) { p.a += context.number; }
      main :: ()->int { p:Pair; p.a=40; p.b=1; bump(*p); return p.a+p.b; }
    "#,
    )
    .unwrap();
    let graph =
        jai_modules::ModuleGraph::load(&path, jai_modules::GraphOptions::default()).unwrap();
    for (triple, arch, os, kind, width) in [
        (
            "x86_64-unknown-linux-gnu",
            Arch::X86_64,
            Os::Linux,
            Object::Elf(62),
            8,
        ),
        (
            "aarch64-unknown-linux-gnu",
            Arch::Arm64,
            Os::Linux,
            Object::Elf(183),
            8,
        ),
        (
            "x86_64-apple-darwin",
            Arch::X86_64,
            Os::MacOS,
            Object::MachO(0x01000007),
            8,
        ),
        (
            "aarch64-apple-darwin",
            Arch::Arm64,
            Os::MacOS,
            Object::MachO(0x0100000c),
            8,
        ),
        (
            "x86_64-pc-windows-msvc",
            Arch::X86_64,
            Os::Windows,
            Object::Coff(0x8664),
            8,
        ),
        (
            "aarch64-pc-windows-msvc",
            Arch::Arm64,
            Os::Windows,
            Object::Coff(0xaa64),
            8,
        ),
        (
            "wasm32-unknown-unknown",
            Arch::WebAssembly32,
            Os::WebAssembly,
            Object::Wasm,
            4,
        ),
        (
            "wasm64-unknown-unknown",
            Arch::WebAssembly64,
            Os::WebAssembly,
            Object::Wasm,
            8,
        ),
        (
            "aarch64-linux-android",
            Arch::Arm64,
            Os::Android,
            Object::Elf(183),
            8,
        ),
        (
            "arm64-apple-ios",
            Arch::Arm64,
            Os::IOS,
            Object::MachO(0x0100000c),
            8,
        ),
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
            let selected = target.build_target().unwrap();
            assert_eq!(selected.architecture, arch);
            assert_eq!(selected.operating_system, os);
            assert_eq!(target.data.get_pointer_byte_size(None), width);
            let program = jai_sema::resolve_graph_with_options(
                &graph,
                &jai_sema::ResolveOptions {
                    target: Some(selected),
                    layout: Some(target.layout_policy().unwrap()),
                    ..Default::default()
                },
                &mut jai_vm::NoEffects,
            )
            .unwrap();
            let result = jai_vm::execute(&program, jai_vm::Limits::default()).outcome;
            assert!(
                matches!(result,jai_vm::Outcome::Complete(values) if values[0].integer().unwrap().value()==42),
                "{triple}"
            );
            let context = jai_codegen::Context::create();
            let module = jai_codegen::lower_for_target(&context, &program, &target).unwrap();
            module.verify().unwrap();
            let object = scratch.0.join("program.o");
            target.write_object(&module, &object).unwrap();
            let bytes = fs::read(&object).unwrap();
            assert!(bytes.len() > 32, "{triple}");
            check_object(&bytes, kind);
            assert_eq!(module.get_triple(), target.triple);
            assert_eq!(
                module.get_data_layout().as_str(),
                target.data.get_data_layout().as_str()
            );
        }
    }
}
#[test]
fn unsupported_foreign_abis_fail_explicitly_instead_of_reusing_desktop_rules() {
    let scratch = Scratch::new();
    let path = scratch.0.join("foreign.jai");
    fs::write(
        &path,
        "external :: ()->int #foreign \"unsupported_symbol\"; main :: ()->int { return external(); }",
    )
    .unwrap();
    let graph =
        jai_modules::ModuleGraph::load(&path, jai_modules::GraphOptions::default()).unwrap();
    for triple in [
        "i686-pc-windows-msvc",
        "armv7-linux-android",
        "armv7-apple-ios",
        "i686-unknown-linux-gnu",
    ] {
        let target = NativeTarget::select(&TargetOptions {
            selection: TargetSelection::Triple(Triple::new(triple).unwrap()),
            ..Default::default()
        })
        .unwrap();
        assert!(
            matches!(
                target.c_platform(),
                Err(jai_codegen::abi::Error::UnsupportedTarget(_))
            ),
            "{triple}"
        );
        let program = jai_sema::resolve_graph_with_options(
            &graph,
            &jai_sema::ResolveOptions {
                target: Some(target.build_target().unwrap()),
                layout: Some(target.layout_policy().unwrap()),
                ..Default::default()
            },
            &mut jai_vm::NoEffects,
        )
        .unwrap();
        let context = jai_codegen::Context::create();
        assert!(
            matches!(
                jai_codegen::lower_for_target(&context, &program, &target),
                Err(jai_codegen::Error::Foreign(
                    jai_codegen::abi::Error::UnsupportedTarget(_)
                ))
            ),
            "{triple}"
        );
    }
}

/// Source declarations exercise hidden-context exclusion at C boundaries and a
/// generated C callback; no cross-target object is linked or executed here.
#[test]
fn source_c_prototypes_and_packed_callback_adapters_emit_on_all_proven_targets() {
    let scratch = Scratch::new();
    let path = scratch.0.join("foreign-callback.jai");
    fs::write(
        &path,
        r#"
        #add_context bias:int = 2;
        Packed :: struct { tag:u8; value:u64; } #no_padding
        Callback :: #type (input:Packed)->Packed #c_call;
        mutate :: (input:Packed)->Packed #foreign "mutate_packed";
        bridge :: (input:Packed, callback:Callback)->Packed #foreign "call_callback";
        callback :: (input:Packed)->Packed #c_call {
            input.tag += 3; input.value += 20; return input;
        }
        main :: ()->int {
            input:Packed = .{tag=7,value=20};
            direct := mutate(input);
            returned := bridge(input,callback);
            return cast(int) direct.value + cast(int) returned.value - 30 + context.bias;
        }
    "#,
    )
    .unwrap();
    let graph =
        jai_modules::ModuleGraph::load(&path, jai_modules::GraphOptions::default()).unwrap();
    for triple in [
        "x86_64-unknown-linux-gnu",
        "aarch64-unknown-linux-gnu",
        "x86_64-apple-darwin",
        "aarch64-apple-darwin",
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
            let program = jai_sema::resolve_graph_with_options(
                &graph,
                &jai_sema::ResolveOptions {
                    target: Some(target.build_target().unwrap()),
                    layout: Some(target.layout_policy().unwrap()),
                    ..Default::default()
                },
                &mut jai_vm::NoEffects,
            )
            .unwrap();
            let context = jai_codegen::Context::create();
            let module = jai_codegen::lower_for_target(&context, &program, &target).unwrap();
            let object = scratch.0.join("callback.o");
            target.write_object(&module, &object).unwrap();
            object_headers::check(&fs::read(&object).unwrap(), triple);
        }
    }
}
