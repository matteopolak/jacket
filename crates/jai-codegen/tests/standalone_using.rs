//! Compile independent source to checked IR, then execute only our emitted objects.
#[path = "support/native_tools.rs"]
mod native_tools;

use jai_codegen::target::NativeTarget;
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
            "jai-standalone-using-native-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        Self(root)
    }

    fn check(&self, source: &str, expected: i32) {
        self.check_with_library(source, "", expected);
    }

    fn check_with_library(&self, source: &str, library: &str, expected: i32) {
        let input = self.0.join("main.jai");
        fs::write(&input, source).unwrap();
        fs::write(self.0.join("library.jai"), library).unwrap();
        let target = NativeTarget::new().unwrap();
        let options = jai_sema::ResolveOptions {
            layout: Some(target.layout_policy().unwrap()),
            ..Default::default()
        };
        let mut discovery =
            jai_modules::GraphDiscovery::new(&input, Default::default(), &jai_modules::Filesystem)
                .unwrap();
        for _ in 0..32 {
            if discovery.advance().unwrap().is_complete() {
                break;
            }
            let requests = discovery.pending_using_requests();
            let outcome = jai_sema::resolve_discovery_using(
                discovery.graph(),
                &requests,
                &options,
                &mut jai_vm::NoEffects,
            )
            .unwrap();
            assert!(
                !outcome.decisions.is_empty(),
                "source using discovery did not progress: {:?}",
                outcome.pending
            );
            for (id, decision) in outcome.decisions {
                discovery.resolve_using(id, decision).unwrap();
            }
        }
        let graph = discovery.into_graph().unwrap();
        let program =
            jai_sema::resolve_graph_with_options(&graph, &options, &mut jai_vm::NoEffects).unwrap();
        let execution = jai_vm::execute(&program, jai_vm::Limits::default());
        let jai_vm::Outcome::Complete(values) = execution.outcome else {
            panic!("using VM did not complete: {execution:?}");
        };
        assert!(
            matches!(values.as_slice(), [jai_vm::Value::Int(value)] if value.value() == i128::from(expected))
        );

        let context = jai_codegen::Context::create();
        let module = jai_codegen::lower_for_target(&context, &program, &target).unwrap();
        let object = self.0.join("program.o");
        let executable = self.0.join("program");
        target.write_object(&module, &object).unwrap();
        let output = native_tools::clang_command()
            .arg(&object)
            .arg("-o")
            .arg(&executable)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stderr),
            module.print_to_string()
        );
        let mut child = Command::new(executable).spawn().unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(status) = child.try_wait().unwrap() {
                assert_eq!(status.code(), Some(expected));
                break;
            }
            if Instant::now() >= deadline {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("generated using fixture timed out");
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn computed_pointer_using_target_runs_once_and_mutates_original_fields() {
    Fixture::new().check(
        r#"
Box :: struct { value:int=7; other:int=11; }
choose :: (target:*Box, calls:*int)->*Box { calls.*+=1; return target; }
main :: ()->int {
    box:Box;
    calls:=0;
    using choose(*box,*calls);
    value+=3;
    other+=5;
    return calls*100+box.value+box.other;
}
"#,
        126,
    );
}

#[test]
fn selector_promotions_keep_actual_storage_and_enum_member_identity() {
    Fixture::new().check(
        r#"
State :: enum u8 #specified { NONE::0; READY::7; }
Box :: struct { x:int=7; y:int=11; z:int=13; }
names :: ()->[1]string { return string.["y"]; }
main :: ()->int {
    box:Box;
    total:=0;
    using State;
    { using,only(x,Missing) box; x+=3; total+=x; }
    { using,only #run names() box; y+=5; total+=y; }
    { using,except string.["x","y"] box; z+=7; total+=z; }
    return total+cast(int) READY;
}
"#,
        53,
    );
}

#[test]
fn compile_time_mapper_replaces_name_slots_without_copying_runtime_fields() {
    Fixture::new().check(
        r#"
Box :: struct { x:int=7; hidden:int=11; y:int=13; }
rename :: (names:[]string) {
    for i:0..names.count-1 {
        if names[i]=="x" names[i]="shown";
        if names[i]=="hidden" names[i]="";
    }
}

main :: ()->int {
    box:Box;
    using,map(rename) box;
    shown+=5;
    y+=3;
    return box.x+box.hidden+box.y;
}
"#,
        39,
    );
}

#[test]
fn mapped_static_prefix_is_captured_only_by_later_local_procedures() {
    Fixture::new().check_with_library(
        r#"
renamed :: 1;
mapper :: (names:[]string) { names[0]="renamed"; }
main :: ()->int {
    before :: ()->int { return renamed; }
    Lib :: #import,file "library.jai";
    using,map(mapper) Lib;
    after :: ()->int { return renamed; }
    return before()+after();
}
"#,
        "value::41;",
        42,
    );
}

#[test]
fn qualified_using_alias_mutates_the_original_nested_field_place() {
    Fixture::new().check(
        "Inner::struct{value:int=21;} Outer::struct{inner:Inner;} main::()->int{outer:Outer; using outer; inner.value+=21; return outer.inner.value;}",
        42,
    );
}

#[test]
fn mapped_single_field_exposes_only_source_members() {
    Fixture::new().check(
        "Record::struct{original:int;} mapper::(names:[]string){names[0]=\"renamed\";} main::()->int{record:Record; using,map(mapper) record; renamed=42; return record.original;}",
        42,
    );
}

#[test]
fn using_declaration_initializes_its_original_storage_once() {
    Fixture::new().check(
        r#"
Record::struct{value:int;}
make::(calls:*int)->Record{calls.*+=1; return .{41};}
main::()->int{
    calls:=0;
    using record:=make(*calls);
    value+=1;
    return calls*100+record.value;
}
"#,
        142,
    );
}

#[test]
fn using_declaration_static_members_preserve_local_procedure_prefixes() {
    Fixture::new().check(
        r#"
answer::1;
main::()->int{
    before::()->int{return answer;}
    using Choice::enum s32{answer::41;}
    after::()->int{return cast(int) answer;}
    return before()+after();
}
"#,
        42,
    );
}

#[test]
fn file_using_declaration_aliases_its_real_global_field() {
    Fixture::new().check(
        "Record::struct{value:int=41;} using record:Record; main::()->int{value+=1; return record.value;}",
        42,
    );
}

#[test]
fn scoped_import_keeps_exported_using_global_field_identity() {
    Fixture::new().check_with_library(
        "main::()->int{#import,file \"library.jai\"; value+=1; return record.value;}",
        "Record::struct{value:int=41;} using record:Record;",
        42,
    );
}

#[test]
fn repeated_module_using_reuses_the_canonical_global_field_view() {
    Fixture::new().check_with_library(
        "Lib::#import,file \"library.jai\"; main::()->int{using Lib; using Lib; value+=1; return Lib.record.value;}",
        "Record::struct{value:int=41;} using record:Record;",
        42,
    );
}
