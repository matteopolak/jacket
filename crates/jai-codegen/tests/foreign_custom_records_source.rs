//! Source declarations cross the custom C ABI and pass a generated C callback.
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
fn packed_source_foreign_call_and_generated_c_callback_execute() {
    struct Scratch(std::path::PathBuf);
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let scratch =
        Scratch(std::env::temp_dir().join(format!("jai-source-custom-c-{}", std::process::id())));
    fs::create_dir(&scratch.0).unwrap();
    let source = scratch.0.join("main.jai");
    fs::write(
        &source,
        r#"
        #add_context bias:int = 2;
        Packed :: struct { tag:u8; value:u64; } #no_padding
        Callback :: #type (input:Packed)->Packed #c_call;
        mutate :: (input:Packed)->Packed #foreign "mutate_packed";
        bridge :: (input:Packed, callback:Callback)->Packed #foreign "call_callback";
        callback :: (input:Packed)->Packed #c_call {
            input.tag += 3;
            input.value += 20;
            return input;
        }
        main :: ()->int {
            input:Packed = .{tag=7,value=20};
            direct := mutate(input);
            returned := bridge(input,callback);
            if input.tag != 7 || input.value != 20 return 1;
            if direct.tag != 9 || returned.tag != 10 return 2;
            if direct.value != 30 || returned.value != 40 return 3;
            return cast(int) direct.value + cast(int) returned.value - 30 + context.bias;
        }
    "#,
    )
    .unwrap();
    let c = scratch.0.join("fixture.c");
    fs::write(&c,r#"
        struct __attribute__((packed)) Packed { unsigned char tag; unsigned long long value; };
        _Static_assert(sizeof(struct Packed)==9,"size");
        _Static_assert(_Alignof(struct Packed)==1,"alignment");
        _Static_assert(__builtin_offsetof(struct Packed,value)==1,"offset");
        struct Packed mutate_packed(struct Packed input) { input.tag+=2; input.value+=10; return input; }
        struct Packed call_callback(struct Packed input, struct Packed (*callback)(struct Packed)) { return callback(input); }
    "#).unwrap();
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
        let context = jai_codegen::Context::create();
        let module = jai_codegen::lower_for_target(&context, &program, &target).unwrap();
        let object = scratch.0.join("program.o");
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
                panic!("source C ABI fixture timed out");
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }
}
