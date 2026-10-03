//! End-to-end checks for our own generated pointer and void callback aliases.
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
impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "jai-alias-native-{}-{}",
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

fn check(files: &[(&str, &str)]) {
    let expected = 42;
    let scratch = Scratch::new();
    let input = scratch.0.join("main.jai");
    for (name, source) in files {
        fs::write(scratch.0.join(name), source).unwrap();
    }
    let graph =
        jai_modules::ModuleGraph::load(&input, jai_modules::GraphOptions::default()).unwrap();
    for optimization in [BitcodeOptimization::O0, BitcodeOptimization::O2] {
        let target = NativeTarget::select(&TargetOptions {
            optimization: Optimization {
                bitcode: optimization,
                ..Optimization::default()
            },
            ..TargetOptions::default()
        })
        .unwrap();
        let program = jai_sema::resolve_graph_with_options(
            &graph,
            &jai_sema::ResolveOptions {
                layout: Some(target.layout_policy().unwrap()),
                ..jai_sema::ResolveOptions::default()
            },
            &mut jai_vm::NoEffects,
        )
        .unwrap();
        let execution = jai_vm::execute(&program, jai_vm::Limits::default());
        let jai_vm::Outcome::Complete(values) = execution.outcome else {
            panic!("VM failed: {:?}", execution.outcome);
        };
        let [jai_vm::Value::Int(value)] = values.as_slice() else {
            panic!("integer result required");
        };
        assert_eq!(value.value(), i128::from(expected));
        let context = jai_codegen::Context::create();
        let module = jai_codegen::lower_for_target(&context, &program, &target).unwrap();
        let object = scratch.0.join("program.o");
        let executable = scratch.0.join("program");
        target.write_object(&module, &object).unwrap();
        let mut command = native_tools::clang_command();
        command.arg(&object);
        if cfg!(target_os = "macos") {
            command.arg("-Wl,-no_fixup_chains");
        }
        let output = command.arg("-o").arg(&executable).output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let mut child = Command::new(&executable).spawn().unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(status) = child.try_wait().unwrap() {
                assert_eq!(status.code(), Some(expected), "{optimization:?}");
                break;
            }
            if Instant::now() >= deadline {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("generated alias fixture timed out");
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }
}

#[test]
fn builtin_pointer_aliases_and_c_void_callbacks_share_canonical_signatures() {
    check(&[(
        "main.jai",
        r#"
        marg_list :: *void;
        PointerPointer :: **void;
        IMP :: #type () -> void #c_call;
        EmptyCallback :: #type () #c_call;
        NoResult :: void;
        answer:int;
        notify :: () -> NoResult #c_call { answer=42; }
        Holder :: struct { callback:IMP=notify; raw:marg_list; }
        main :: () -> int {
            holder:Holder;
            nested:PointerPointer;
            if holder.raw != null return 1;
            if nested != null return 2;
            callback:EmptyCallback=holder.callback;
            callback();
            return answer;
        }
        "#,
    )]);
}

#[test]
fn local_pointer_and_void_callback_aliases_keep_storage_address_semantics() {
    check(&[(
        "main.jai",
        r#"
        main :: () -> int {
            Number :: int;
            Pointer :: *Number;
            Callback :: #type (value:Pointer) -> void #c_call;
            notify :: (value:Pointer) -> void #c_call { value.*=42; }
            answer:int;
            pointer:Pointer=*answer;
            callback:Callback=notify;
            callback(pointer);
            return pointer.*;
        }
        "#,
    )]);
}

#[test]
fn generic_void_result_patterns_produce_no_runtime_result_slot() {
    check(&[(
        "main.jai",
        r#"
        write :: (target:*$T, value:T) -> void { target.*=value; }
        main :: () -> int { answer:int; write(*answer,42); return answer; }
        "#,
    )]);
}

#[test]
fn imported_aliases_and_module_callback_arguments_keep_zero_results() {
    check(&[
        (
            "main.jai",
            r#"
            Types :: #import,file "types.jai";
            Calls :: #import,file "calls.jai"(T=Types.Callback);
            notify :: (target:*int) -> void #c_call { target.*=42; }
            main :: () -> int { answer:int; pointer:Types.Pointer=*answer;
                Calls.invoke(notify,pointer); return answer; }
            "#,
        ),
        (
            "types.jai",
            "Pointer :: *int; Callback :: #type (target:Pointer) -> void #c_call;",
        ),
        (
            "calls.jai",
            "#module_parameters(T:Type); invoke :: (callback:T,target:*int) -> void { callback(target); }",
        ),
    ]);
}

#[test]
fn imported_opaque_anonymous_pointer_alias_keeps_its_canonical_target() {
    check(&[
        (
            "main.jai",
            "Refs::#import,file \"refs.jai\"; accept::(value:Refs.EventStreamRef)->int{return ifx value==null then 42 else 1;} main::()->int{first:Refs.EventStreamRef;second:Refs.Again=first;return accept(second);}",
        ),
        (
            "refs.jai",
            "EventStreamRef::*struct {}; Again::#type EventStreamRef;",
        ),
    ]);
}

#[test]
fn local_anonymous_union_and_enum_pointer_aliases_use_real_nominals() {
    check(&[(
        "main.jai",
        "main::()->int{UnionRef::**union{integer:int;number:float64;};EnumRef::*enum u8{ZERO::0;};FlagsRef::*enum_flags u8{A::1;};a:UnionRef;b:UnionRef=a;c:EnumRef;d:FlagsRef;if b!=null || c!=null || d!=null return 1;return 42;}",
    )]);
}
