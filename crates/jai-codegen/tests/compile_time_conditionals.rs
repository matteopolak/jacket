//! Execute only independently authored source and newly emitted host objects.
#[path = "support/native_tools.rs"]
mod native_tools;
use jai_codegen::target::NativeTarget;
use jai_modules::{GraphOptions, ModuleGraph, SourceOverlay};
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

#[test]
fn static_selection_scope_cleanup_and_source_target_tags_agree_natively() {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let root = std::env::temp_dir().join(format!(
        "jai-static-if-native-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    struct Scratch(PathBuf);
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let scratch = Scratch(root);
    let input = scratch.0.join("main.jai");
    let native = NativeTarget::new().unwrap();
    let target = native.build_target().unwrap();
    let source = r#"
        Operating_System_Tag :: enum u32 #specified { WINDOWS::2; LINUX::3; MACOS::6; }
        CPU_Tag :: enum u32 #specified { X64::3; ARM64::4; }
        counter: int = 0;
        fact :: (n:int) -> int { if n <= 1 return 1; return n * fact(n-1); }
        mode :: ($Enabled: bool) -> int {
            #if Enabled { #assert Enabled; return 1; }
            else { #assert !Enabled "selected false specialization"; return 2; }
        }
        #assert #run fact(5) == 120 "checked source factorial";
        main :: () -> int {
            #assert !(2 & 1) "selected compile-time flag";
            if mode(true) + mode(false) != 3 return 1;
            #if OS == .WINDOWS || OS == .LINUX || OS == .MACOS {
                select :: () -> int { return 40; }
            } else { select :: () -> int { return missing_platform(); } }
            #if CPU == .X64 || CPU == .ARM64 { offset := 2; }
            else { offset := missing_architecture; }
            { #if #run fact(5) > 100 { defer counter += offset; }
              else { unavailable(); } counter += 5; if counter != 5 return 1; }
            return select() + counter - 5;
        }
    "#;
    let mut overlay = SourceOverlay::new();
    overlay.insert(&input, source.as_bytes().to_vec()).unwrap();
    let graph =
        ModuleGraph::load_with_target(&input, GraphOptions::default(), &overlay, target.clone())
            .unwrap();
    let program = jai_sema::resolve_graph_with_options(
        &graph,
        &jai_sema::ResolveOptions {
            target: Some(target),
            ..Default::default()
        },
        &mut jai_vm::NoEffects,
    )
    .unwrap();
    assert!(
        matches!(jai_vm::execute(&program, jai_vm::Limits::default()).outcome, jai_vm::Outcome::Complete(values) if matches!(values.as_slice(), [jai_vm::Value::Int(value)] if value.value()==42))
    );
    let context = jai_codegen::Context::create();
    let module = jai_codegen::lower_for_target(&context, &program, &native).unwrap();
    let object = scratch.0.join("program.o");
    let executable = scratch.0.join("program");
    native.write_object(&module, &object).unwrap();
    let output = native_tools::clang_command()
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
    let mut child = Command::new(&executable).spawn().unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert_eq!(status.code(), Some(42));
            break;
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("generated static-if fixture timed out");
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}
