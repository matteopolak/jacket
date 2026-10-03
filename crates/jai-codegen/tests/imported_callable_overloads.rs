//! Original imported declarations execute through the shared native pipeline.
#[path = "support/native_tools.rs"]
mod native_tools;
use std::{
    fs,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

fn check(application: &str, library: &str) {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    struct Fixture(std::path::PathBuf);
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let fixture = Fixture(std::env::temp_dir().join(format!(
        "jai-imported-overloads-native-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed),
    )));
    fs::create_dir_all(&fixture.0).unwrap();
    let input = fixture.0.join("main.jai");
    fs::write(&input, application).unwrap();
    fs::write(fixture.0.join("library.jai"), library).unwrap();
    let graph = jai_modules::ModuleGraph::load(&input, Default::default()).unwrap();
    let target = jai_codegen::target::NativeTarget::new().unwrap();
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
    let execution = jai_vm::execute(&program, jai_vm::Limits::default());
    assert!(
        matches!(execution.outcome, jai_vm::Outcome::Complete(ref values)
        if values[0].integer().is_ok_and(|value|value.value()==42)),
        "{execution:?}"
    );
    let context = jai_codegen::Context::create();
    let module = jai_codegen::lower_for_target(&context, &program, &target).unwrap();
    let object = fixture.0.join("program.o");
    let executable = fixture.0.join("program");
    target.write_object(&module, &object).unwrap();
    let compiled = native_tools::clang_command()
        .arg(&object)
        .arg("-o")
        .arg(&executable)
        .output()
        .unwrap();
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
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
            panic!("generated imported overload fixture timed out");
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn imported_alias_and_local_overload_keep_named_default_definition_scope() {
    check(
        "#import,file \"library.jai\";DEFAULT::1;pick::(value:bool)->int{return 1;}main::()->int{return pick()+pick(value=true);}",
        "pick::original;#scope_module DEFAULT::41;original::(value:int=DEFAULT)->int{return value;}",
    );
}

#[test]
fn application_overload_does_not_affect_original_generic_library_call() {
    check(
        "#import,file \"library.jai\";pick::(value:bool)->int{return 100;}main::()->int{return inside()+20;}",
        "pick::(value:int)->int{return 20;}pick::(value:$T)->int{return 22;}inside::()->int{return pick(true);}",
    );
}
