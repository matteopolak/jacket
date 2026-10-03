//! End-to-end checks for our own generated record specializations.
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
            "jai-generic-native-{}-{}",
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
                panic!("generated record specialization timed out");
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }
}

#[test]
fn baked_type_count_and_value_defaults_execute_natively() {
    check(
        r#"
    Buffer :: struct(T:=int, N:int=3, Fill:T=7) { values:[N]T; tag:T=Fill; }
    main :: ()->int { a:Buffer(); b:Buffer(N=2,Fill=5); b.values[1]=11;
        return a.tag+b.tag+b.values[1]+a.values.count+b.values.count; }
    "#,
        28,
    );
}
#[test]
fn specialized_field_run_defaults_wait_for_real_provider_bodies() {
    check(
        "Owner::struct(T:Type){value:T=#run seed();seed::()->T{return 21;}}main::()->int{first:Owner(int);second:Owner(int);return first.value+second.value;}",
        42,
    );
    check(
        "#add_context amount:int=21;Owner::struct(T:Type){value:T=#run seed();seed::()->T{return context.amount;}}main::()->int{first:Owner(int);second:Owner(int);return first.value+second.value;}",
        42,
    );
}

#[test]
fn record_field_run_jobs_complete_before_global_initialization() {
    check(
        "#add_context amount:int=21;Owner::struct{value:int=#run seed();seed::()->int{return context.amount;}}saved:Owner;read::(next:Owner=Owner.{}) ->int{return next.value;}main::()->int{return saved.value+read();}",
        42,
    );
}

#[test]
fn recursive_generic_records_and_inferred_results_execute_natively() {
    check(
        r#"
    Node :: struct(T:Type) { next:*Node(T); value:T=19; }
    pass :: (node:Node($T))->Node(T) { return node; }
    main :: ()->int { node:Node(int); copy:=pass(node); node.value=99;
        if copy.next!=null return 1; return copy.value+23; }
    "#,
        42,
    );
}
#[test]
fn nested_member_types_and_baked_default_constraints_execute_natively() {
    check(
        r#"
    Table :: struct(T:Type, Fill:T=9) { values:[Capacity]Entry;
        Capacity :: Later; Later :: 2; Item :: T;
        Entry :: struct { next:*Entry; value:Item=Fill; }
    }
    read :: (table:Table($T))->T { return table.values[0].value; }
    main :: ()->int { table:Table(int); table.values[1].value=31;
        return read(table)+table.values[1].value+table.values.count; }
    "#,
        42,
    );
}
#[test]
fn source_nested_enum_defaults_execute_with_their_declared_representation() {
    check(
        r#"
    Owner :: struct(T:Type) { flags:Flags=Flags.SECOND; mode:=Saved;
        Saved :: Mode.VIEW; Flags :: enum_flags u32 { FIRST; SECOND; }
        Mode :: enum u16 { FIXED; VIEW; }
    }
    main :: ()->int { value:Owner(int); first:Owner(int)=value;
        return cast(int)first.flags+cast(int)first.mode+39; }
    "#,
        42,
    );
}

#[test]
fn inserted_fields_and_checked_record_methods_execute_natively() {
    check(
        r#"
    Box :: struct(T:Type, Fill:T=19) {
        #insert #code { value:T=Fill; };
        read :: (value:Box)->T { return value.value; }
        seed :: ()->T { return Fill; }
    }
    IntBox :: Box(int);
    main :: ()->int { value:IntBox; return IntBox.read(value)+IntBox.seed()+4; }
    "#,
        42,
    );
}

#[test]
fn selected_record_members_and_aggregate_constants_execute_natively() {
    check(
        r#"
    Pair :: struct { left:int; right:int; }
    Holder :: struct(T:Type,N:int=2) {
        #assert N>0 && N<4;
        #if N==2 { Saved :: Pair.{19,23}; value:=Saved; }
        else { count:int=N; }
    }
    main :: ()->int { first:Holder(int); copy:Holder(int)=first;
        return copy.value.left+copy.value.right; }
    "#,
        42,
    );
}

#[test]
fn inherited_default_overrides_and_inserted_assignments_execute_natively() {
    check(
        r#"
    Pair :: struct { left:int=1; right:int=2; }
    Derived :: struct(T:Type,Fill:T=19) {
        #as using base:Pair;
        #insert #code { base.left=Fill; };
        #if Fill==19 { base.right=23; }
        #assert Fill>0 "positive fill required";
    }
    GLOBAL:Derived(int);
    main :: ()->int { original:Pair; other:Derived(int,7);
        if original.left!=1 || original.right!=2 return 1;
        if other.left!=7 || other.right!=2 return 2;
        return GLOBAL.left+GLOBAL.right; }
    "#,
        42,
    );
}

#[test]
fn typed_record_case_tables_execute_natively() {
    check(
        r#"
    Choice::struct(T:Type,Flag:bool=true) {
        #if T == {
            case int;
                #if #complete Flag == {
                    case false; value:T=1;
                    case true; value:T=42;
                }
            case; other:T;
        }
    }
    main::()->int {value:Choice(int); return value.value;}
    "#,
        42,
    );
}

#[test]
fn completed_record_method_alias_defaults_execute_natively() {
    check(
        "Methods::struct{read::(value:int=#run Seed.make())->int{return value;}} Seed::struct{make::()->int{return 21;}} alias::Methods.read; forwarded::alias; main::()->int{return forwarded()+alias();}",
        42,
    );
    check(
        "#add_context marker:int=7; R::struct{value:int=14;} Methods::struct{read::(value:R=R.{},saved:#Context=.{})->int{return value.value+saved.marker;}} alias::Methods.read; forwarded::alias; main::()->int{return forwarded()+alias(saved=.{});}",
        42,
    );
}
