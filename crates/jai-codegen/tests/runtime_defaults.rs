//! VM/native parity uses authored sources and freshly linked trusted-tool output.
#[path = "support/native_tools.rs"]
mod native_tools;
use jai_codegen::{
    optimization::Optimization,
    target::{NativeTarget, TargetOptions},
};
use jai_types::BitcodeOptimization;
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

struct Scratch(PathBuf);
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn check(source: &str, c: Option<&str>) {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let scratch = Scratch(std::env::temp_dir().join(format!(
        "jai-runtime-default-native-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )));
    fs::create_dir(&scratch.0).unwrap();
    let input = scratch.0.join("main.jai");
    fs::write(&input, source).unwrap();
    let c_file = scratch.0.join("fixture.c");
    if let Some(c) = c {
        fs::write(&c_file, c).unwrap();
    }
    let graph =
        jai_modules::ModuleGraph::load(&input, jai_modules::GraphOptions::default()).unwrap();
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
        .unwrap_or_else(|error| panic!("{}", error.render(graph.sources())));
        if c.is_none() {
            let outcome = jai_vm::execute(&program, jai_vm::Limits::default()).outcome;
            assert!(
                matches!(&outcome,jai_vm::Outcome::Complete(values) if matches!(values.as_slice(),[jai_vm::Value::Int(value)] if value.value()==42)),
                "{outcome:?}"
            );
        }
        let context = jai_codegen::Context::create();
        let module = jai_codegen::lower_for_target(&context, &program, &target).unwrap();
        module.verify().unwrap();
        let object = scratch.0.join("program.o");
        let executable = scratch.0.join("program");
        target.write_object(&module, &object).unwrap();
        let mut link = native_tools::clang_command();
        link.arg(&object);
        if c.is_some() {
            link.arg(&c_file);
        }
        let output = link
            .arg(if bitcode == BitcodeOptimization::O0 {
                "-O0"
            } else {
                "-O2"
            })
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
                assert_eq!(status.code(), Some(42), "{bitcode:?}");
                break;
            }
            if Instant::now() >= deadline {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("authored runtime default fixture exceeded deadline");
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }
}
#[test]
fn global_aggregate_default_retains_live_callback_pointer_and_defining_scope() {
    check(
        include_str!("../../jai-sema/tests/fixtures/runtime-default-global.jai"),
        None,
    );
}
#[test]
fn default_reads_two_active_contexts_without_freezing_either_value() {
    check(
        include_str!("../../jai-sema/tests/fixtures/runtime-default-context.jai"),
        None,
    );
}
#[test]
fn indirect_context_override_loads_omitted_default_inside_generated_context() {
    check(
        include_str!("../../jai-sema/tests/fixtures/runtime-default-context-override.jai"),
        None,
    );
}
#[test]
fn selected_generic_context_call_uses_actual_nonbaked_parameter_ids() {
    check(
        "#add_context number:int; read::($before:int,prefix:int,$after:int,selected:int=context.number)->int{return prefix+selected;} main::()->int{return read(before=100,prefix=2,after=200,,number=40);}",
        None,
    );
}
#[test]
fn forwarded_pack_and_discarded_argument_keep_the_default_destination() {
    check(
        "#add_context number:int; read::(#discard ignored:int,prefix:int,values:..int,selected:int=context.number)->int{if values.count!=2 return 1;return prefix+selected;} main::()->int{values:[2]int=.[1,2];return read(100,2,..values,,number=40);}",
        None,
    );
}
#[test]
fn authored_foreign_abi_receives_the_current_aggregate_default() {
    check(
        "Allocator::struct{marker:s64;} Context::struct{allocator:Allocator;} context:Context; default_allocator_read::(allocator:=context.allocator)->s64 #c_call #foreign; main::()->int{context.allocator.marker=40;if default_allocator_read()!=40 return 1;context.allocator.marker=42;return default_allocator_read();}",
        Some(
            "#include <stdint.h>\nstruct Allocator { int64_t marker; };\nint64_t default_allocator_read(struct Allocator allocator) { return allocator.marker; }\n",
        ),
    );
}
