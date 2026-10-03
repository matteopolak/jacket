//! Native checks compile only independently authored source and newly emitted LLVM.
#[path = "support/native_tools.rs"]
mod native_tools;
use jai_modules::{GraphOptions, ModuleGraph};
use jai_sema::{ResolveOptions, resolve_graph_with_options};
use jai_types::LayoutPolicy;
use jai_vm::{Limits, NoEffects, Outcome};
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "jai-any-native-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        Self(root)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn check(source: &str) {
    let fixture = Fixture::new();
    let input = fixture.0.join("main.jai");
    fs::write(&input, source).unwrap();
    let graph = ModuleGraph::load(&input, GraphOptions::default()).unwrap();
    let program = resolve_graph_with_options(
        &graph,
        &ResolveOptions {
            layout: Some(LayoutPolicy::lp64()),
            ..ResolveOptions::default()
        },
        &mut NoEffects,
    )
    .unwrap();
    let Outcome::Complete(values) = jai_vm::execute(&program, Limits::default()).outcome else {
        panic!("Any fixture must complete in the VM");
    };
    assert_eq!(values[0].integer().unwrap().value(), 42);
    let llvm = jai_codegen::emit(&program).unwrap();
    let llvm_path = fixture.0.join("main.ll");
    let executable = fixture.0.join("main");
    fs::write(&llvm_path, &llvm).unwrap();
    let output = native_tools::clang_command()
        .arg(&llvm_path)
        .arg("-o")
        .arg(&executable)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{llvm}",
        String::from_utf8_lossy(&output.stderr)
    );
    let mut child = Command::new(executable).spawn().unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert_eq!(status.code(), Some(42), "{llvm}");
            return;
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("newly generated Any fixture exceeded its deadline");
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn borrowed_storage_and_descriptor_copy_keep_native_aliases() {
    check(
        "main :: () -> int { value:s32=1; first:Any=value; second:Any=first; value=42; if first.value_pointer!=second.value_pointer return 1; if first.type!=second.type return 2; return (cast(*s32)second.value_pointer).*; }",
    );
}

#[test]
fn rvalue_boxing_runs_once_in_its_conditional_branch() {
    check(
        "calls:int=0; next :: () -> int { calls+=1; return 41; } main :: () -> int { value:Any=ifx true then next() else 1/0; return (cast(*int)value.value_pointer).*+calls; }",
    );
}

#[test]
fn jai_variadic_forwarding_retains_descriptors_and_caller_temporaries() {
    check(
        "sum :: (args:..Any) -> int { result:int=0; for i:0..args.count-1 { result+=(cast(*int)args[i].value_pointer).*; } return result; } forward :: (args:..Any) -> int { return sum(..args); } main :: () -> int { value:=20; boxed:Any=value; return forward(boxed,22); }",
    );
}

#[test]
fn mixed_any_packs_keep_native_payload_aliases_and_caller_temporaries() {
    check(
        r#"
        original:int=9;
        calls:int=0;
        prefix :: () -> int { original=10; calls+=1; return 41; }
        read :: (args:..Any) -> int {
            if args.count!=3 return 1;
            if args[1].value_pointer!=args[2].value_pointer return 2;
            if args[1].type!=args[2].type return 3;
            return (cast(*int)args[0].value_pointer).*
                 + (cast(*int)args[1].value_pointer).* - 9;
        }
        forward :: (args:..Any) -> int { return read(prefix(),..args); }
        main :: () -> int {
            descriptor:Any=original;
            result:=forward(descriptor,descriptor);
            if calls!=1 return 4;
            return result;
        }
    "#,
    );
}

#[test]
fn universal_reflection_and_manual_descriptor_fields_use_real_addresses() {
    check(
        "main :: () -> int { value:=42; boxed:Any; boxed.type=cast(*Type_Info)type_info(int); boxed.value_pointer=cast(*void)*value; info:=type_info(Any); if cast(int)info.type!=10 return 1; if info.runtime_size!=16 return 2; return (cast(*int)boxed.value_pointer).*; }",
    );
}

#[test]
fn uninitialized_descriptor_field_writes_create_a_complete_native_value() {
    check(
        "main :: () -> int { value:=42; boxed:Any=---; boxed.type=cast(*Type_Info)type_info(int); boxed.value_pointer=cast(*void)*value; copy:Any=boxed; return (cast(*int)copy.value_pointer).*; }",
    );
}

#[test]
fn source_storage_mirror_pointer_casts_preserve_universal_identity() {
    check(
        "Mirror :: struct { type:*Type_Info; value_pointer:*void; } main :: () -> int { original:=1; replacement:=42; descriptor:Any=original; mirror:=cast(*Mirror)*descriptor; mirror.value_pointer=cast(*void)*replacement; mirror_copy:=mirror.*; boxed_mirror:Any=mirror_copy; if cast(*void)boxed_mirror.type!=cast(*void)type_info(Mirror) return 1; return (cast(*int)descriptor.value_pointer).*; }",
    );
}

#[test]
fn descriptor_record_crosses_source_c_call_abi_as_two_pointer_fields() {
    check(
        "identity :: (value:Any) -> Any #c_call { return value; } main :: () -> int { original:=42; boxed:Any=original; result:=identity(boxed); return (cast(*int)result.value_pointer).*; }",
    );
}

#[test]
fn any_fields_in_format_style_records_use_regular_aggregate_storage() {
    check(
        "Format :: struct { value:Any; width:int; } main :: () -> int { original:=20; item:Format=.{value=original,width=22}; original=21; return (cast(*int)item.value.value_pointer).*+item.width-1; }",
    );
}

#[test]
fn runtime_type_boxing_uses_actual_canonical_native_pointer_cells() {
    check(
        r#"
        main :: () -> int {
            direct:Any=s32;
            if cast(int)direct.type.type!=13 return 1;
            if direct.type.runtime_size!=size_of(Type) return 2;
            descriptor:=(cast(**Type_Info)direct.value_pointer).*;
            if cast(*void)descriptor!=cast(*void)type_info(s32) return 3;
            chosen:Type=s32;
            borrowed:Any=chosen;
            chosen=s64;
            updated:=(cast(**Type_Info)borrowed.value_pointer).*;
            if cast(*void)updated!=cast(*void)type_info(s64) return 4;
            if updated.runtime_size!=8 return 5;
            return 42;
        }
    "#,
    );
}

#[test]
fn procedure_payloads_recover_the_actual_native_callable_cell() {
    check(
        r#"
        Increment::#type (value:s32)->s32;
        increment::(value:s32)->s32{return value+1;}
        main::()->int {
            boxed:Any=increment;
            if cast(*void)boxed.type!=cast(*void)type_info(Increment) return 1;
            callable:=(cast(*Increment)boxed.value_pointer).*;
            return callable(41);
        }
    "#,
    );
}
