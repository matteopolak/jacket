//! Record self-pointer annotations share canonical identities in VM and native code.
#[path = "support/native_tools.rs"]
mod native_tools;
use jai_codegen::{
    optimization::Optimization,
    target::{NativeTarget, TargetOptions},
};
use jai_types::BitcodeOptimization;
use std::{
    fs,
    process::Command,
    time::{Duration, Instant},
};

#[test]
fn generic_record_self_pointers_execute() {
    struct Scratch(std::path::PathBuf);
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let scratch = Scratch(std::env::temp_dir().join(format!(
        "jai-source-record-self-types-{}",
        std::process::id()
    )));
    fs::create_dir(&scratch.0).unwrap();
    let source = scratch.0.join("main.jai");
    fs::write(
        &source,
        r#"
        Node :: struct(T:Type) { next:*#this; value:T; }
        main :: ()->int {
            first:Node(int);
            second:Node(int);
            first.next=*second;
            first.next.value=42;
            return second.value;
        }
    "#,
    )
    .unwrap();
    let graph =
        jai_modules::ModuleGraph::load(&source, jai_modules::GraphOptions::default()).unwrap();
    for bitcode in [BitcodeOptimization::O0, BitcodeOptimization::O2] {
        let target = NativeTarget::select(&TargetOptions {
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
        let outcome = jai_vm::execute(&program, jai_vm::Limits::default()).outcome;
        let jai_vm::Outcome::Complete(values) = outcome else {
            panic!("{outcome:?}");
        };
        assert_eq!(values[0].integer().unwrap().value(), 42);
        let context = jai_codegen::Context::create();
        let module = jai_codegen::lower_for_target(&context, &program, &target).unwrap();
        let object = scratch.0.join("program.o");
        let executable = scratch.0.join("program");
        target.write_object(&module, &object).unwrap();
        let mut command = native_tools::clang_command();
        command
            .arg(&object)
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
        let output = command.output().unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stderr),
            module.print_to_string()
        );
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
                panic!("record self-pointer fixture timed out");
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }
}
