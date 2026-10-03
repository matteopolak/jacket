//! Only independently authored C++ and our newly emitted objects are executed.
#![cfg(any(target_os = "linux", target_os = "macos"))]
#[path = "support/native_tools.rs"]
mod native_tools;
use jai_codegen::{native_reachability::Publication, target::NativeTarget};
use jai_modules::{GraphOptions, ModuleGraph, SourceOverlay};
use std::{
    fs,
    path::Path,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

#[test]
fn real_cpp_vtable_calls_cross_both_directions() {
    let source = r#"
Receiver :: struct { vtable: *VTable; base: s32; }
VTable :: struct { score: (this:*Receiver, delta:s32)->s32 #cpp_method; }
#program_export "jai_score" score :: (this:*Receiver,delta:s32)->s32 #cpp_method { return this.base+delta+1; }
#program_export "jai_invoke" invoke :: (this:*Receiver,delta:s32)->s32 #c_call { return this.vtable.score(this,delta); }
"#;
    let path = Path::new("/own-cpp-methods/main.jai");
    let mut overlay = SourceOverlay::new();
    overlay.insert(path, source.as_bytes().to_vec()).unwrap();
    let graph = ModuleGraph::load_with_provider(path, GraphOptions::default(), &overlay).unwrap();
    let library = jai_sema::resolve_library(&graph).unwrap();
    let target = NativeTarget::new().unwrap();
    let context = jai_codegen::Context::create();
    let module = jai_codegen::lower_library_for_target(
        &context,
        &library,
        &Publication::Selected(vec![]),
        &target,
    )
    .unwrap();
    module.verify().unwrap();
    let callback = module.get_function("jai_score").unwrap();
    assert_eq!(
        callback.count_params(),
        2,
        "C++ methods must not receive Jai context"
    );
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let dir = std::env::temp_dir().join(format!(
        "jai-own-cpp-method-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&dir).unwrap();
    let object = dir.join("generated.o");
    target.write_object(&module, &object).unwrap();
    let cpp = dir.join("own.cpp");
    fs::write(
        &cpp,
        r#"
class Receiver { public: virtual int score(int delta) { return base + delta; } int base = 40; };
extern "C" int jai_score(Receiver *, int);
extern "C" int jai_invoke(Receiver *, int);
__attribute__((noinline)) int invoke_virtual(Receiver *r, int delta) { return r->score(delta); }
int main() {
    Receiver receiver;
    if (jai_invoke(&receiver, 2) != 42) return 1;
    void *replacement[] = {nullptr, nullptr, reinterpret_cast<void *>(&jai_score)};
    *reinterpret_cast<void ***>(&receiver) = replacement + 2;
    if (invoke_virtual(&receiver, 2) != 43) return 2;
    if (jai_invoke(&receiver, 2) != 43) return 3;
    return 0;
}
"#,
    )
    .unwrap();
    let executable = dir.join("program");
    let output = native_tools::clang_command()
        .arg("--driver-mode=g++")
        .args(["-std=c++17", "-O0"])
        .arg(&cpp)
        .arg(&object)
        .arg("-o")
        .arg(&executable)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let status = Command::new(&executable).status().unwrap();
    assert!(status.success(), "C++ vtable oracle exited {status}");
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn cpp_aggregate_carriers_are_rejected_without_an_abi_proof() {
    let source = "Payload :: struct { value:s32; } #program_export method :: (this:*void,value:Payload)->s32 #cpp_method {return value.value;}";
    let path = Path::new("/own-cpp-aggregate/main.jai");
    let mut overlay = SourceOverlay::new();
    overlay.insert(path, source.as_bytes().to_vec()).unwrap();
    let graph = ModuleGraph::load_with_provider(path, GraphOptions::default(), &overlay).unwrap();
    let library = jai_sema::resolve_library(&graph).unwrap();
    let target = NativeTarget::new().unwrap();
    let context = jai_codegen::Context::create();
    assert!(matches!(
        jai_codegen::lower_library_for_target(
            &context,
            &library,
            &Publication::Selected(vec![]),
            &target
        ),
        Err(jai_codegen::Error::Foreign(
            jai_codegen::abi::Error::UnsupportedType(_)
        ))
    ));
}
