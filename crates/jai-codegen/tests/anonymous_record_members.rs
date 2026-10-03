//! End-to-end checks for our own generated unnamed aggregate members.
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
            "jai-anonymous-members-native-{}-{}",
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

fn check(source: &str, expected: i32) {
    let scratch = Scratch::new();
    let input = scratch.0.join("main.jai");
    fs::write(&input, source).unwrap();
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
                assert_eq!(status.code(), Some(expected), "{optimization:?}\n{source}");
                break;
            }
            if Instant::now() >= deadline {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("generated anonymous field fixture timed out");
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }
}

#[test]
fn jaison_shaped_promoted_constant_defaults_select_actual_union_storage() {
    for source in [
        r#"JSON_Type::enum u8{NULL::0;NUMBER::3;STRING::2;}
        JSON_Value::struct{type:JSON_Type;union{number:float64;str:string;}}
        Holder::struct{value:JSON_Value=.{type=.NUMBER,number=42.0};}
        main::()->int{holder:Holder;copy:=holder.value;if copy.type!=.NUMBER return 1;return cast(int)copy.number;}"#,
        r#"JSON_Type::enum u8{NULL::0;STRING::2;}
        JSON_Value::struct{type:JSON_Type;union{number:float64;str:string;}}
        Holder::struct{value:JSON_Value=.{type=.STRING,str="forty-two"};}
        main::()->int{holder:Holder;copy:=holder.value;if copy.type!=.STRING || copy.str!="forty-two" return 1;return 42;}"#,
    ] {
        check(source, 42);
    }
}

#[test]
fn promoted_constant_defaults_preserve_selected_branch_and_source_overlays() {
    for source in [
        r#"Owner::struct{union{struct{x:int=1;y:int=22;}other:int=99;}}
        Holder::struct{value:Owner=.{x=20};}
        main::()->int{holder:Holder;return holder.value.x+holder.value.y;}"#,
        r#"Child::struct{x:int=1;y:int=2;}
        Owner::struct{using child:Child=.{x=7,y=22};}
        Holder::struct{value:Owner=.{x=20};}
        main::()->int{holder:Holder;return holder.value.x+holder.value.y;}"#,
        r#"Owner::struct{struct{x:int=1;y:int=2;}x=5;y=22;}
        Holder::struct{value:Owner=.{x=20};}
        main::()->int{holder:Holder;return holder.value.x+holder.value.y;}"#,
    ] {
        check(source, 42);
    }
}

#[test]
fn anonymous_struct_members_keep_defaults_and_promote_real_fields() {
    check(
        r#"
Owner :: struct { before:u8; struct { x:int=20; y:int=22; } after:u8; }
main :: () -> int { value:Owner; value.before=7; value.after=9; return value.x+value.y; }
"#,
        42,
    );
}

#[test]
fn anonymous_nested_union_views_share_bytes_and_keep_surrounding_fields() {
    check(
        r#"
Vector :: struct { before:int; union { struct { x,y:int; } struct { r,g:int; } } after:int; }
main :: () -> int {
    value:Vector=---; value.before=5; value.after=7; value.x=20; value.y=22;
    if value.before!=5 || value.after!=7 return 1;
    return value.r+value.g;
}
"#,
        42,
    );
}

#[test]
fn anonymous_member_paths_remain_projected_storage_for_using_parameters() {
    check(
        r#"
Owner :: struct { struct { amount:int=21; } }
add :: (using value:*Owner) { amount+=21; }
main :: () -> int { value:Owner; add(*value); return value.amount; }
"#,
        42,
    );
}

#[test]
fn anonymous_members_keep_specialization_defaults_and_recursive_pointers() {
    check(
        r#"
Box :: struct(T:Type, Fill:T) { struct { amount:T=Fill; next:*Box(T,Fill); } }
main :: () -> int {
    first:Box(int,20); second:Box(int,20)=first; small:Box(u8,22);
    if second.next!=null return 1;
    return second.amount+cast(int)small.amount;
}
"#,
        42,
    );
}

#[test]
fn local_anonymous_members_capture_definition_scope() {
    check(
        r#"
main :: () -> int {
    Count::20;
    Local::struct { struct { value:int=Count; } }
    one:Local; two:Local=one;
    { Count::22; Inner::struct { struct { value:int=Count; } } value:Inner; return two.value+value.value; }
}
"#,
        42,
    );
}

#[test]
fn anonymous_union_uniform_zero_defaults_initialize_checked_storage() {
    check(
        r#"
Owner::struct { union { flag:bool; number:float64; pointer:*Owner; } }
main::()->int { value:Owner; if value.flag || value.pointer!=null || value.number!=0.0 return 1; value.number=42.0; return cast(int)value.number; }
"#,
        42,
    );
}

#[test]
fn positional_initialization_uses_real_unnamed_physical_field_ids() {
    check(
        r#"
Owner::struct { struct { x:int; y:int; } tail:int=7; }
main::()->int { first:Owner=.{.{20,22}}; second:Owner=first; if second.tail!=7 return 1; return second.x+second.y; }
"#,
        42,
    );
}

#[test]
fn construction_overrides_follow_promoted_physical_field_paths() {
    check(
        r#"
Owner::struct { struct { x:int=1; y:int=2; } x=20; y=22; }
main::()->int { value:Owner; return value.x+value.y; }
"#,
        42,
    );
}
