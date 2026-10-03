//! Source-backed forms use independently authored VM/native behavior fixtures.
#[path = "support/native_tools.rs"]
mod native_tools;
use jai_vm::{Error, Limits, Outcome, Value};
use std::{
    fs,
    path::PathBuf,
    process::{Command, ExitStatus},
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Fixture(PathBuf);

impl Fixture {
    fn new(source: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "jai-using-declarations-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        fs::write(root.join("main.jai"), source).unwrap();
        Self(root)
    }

    fn graph(&self) -> jai_modules::ModuleGraph {
        let mut discovery = jai_modules::GraphDiscovery::new(
            &self.0.join("main.jai"),
            Default::default(),
            &jai_modules::Filesystem,
        )
        .unwrap();
        for _ in 0..32 {
            if discovery.advance().unwrap().is_complete() {
                break;
            }
            let requests = discovery.pending_using_requests();
            let outcome = jai_sema::resolve_discovery_using(
                discovery.graph(),
                &requests,
                &Default::default(),
                &mut jai_vm::NoEffects,
            )
            .unwrap();
            assert!(outcome.pending.is_empty(), "{:?}", outcome.pending);
            assert!(
                !outcome.decisions.is_empty(),
                "using source preparation did not progress"
            );
            for (id, decision) in outcome.decisions {
                discovery.resolve_using(id, decision).unwrap();
            }
        }
        discovery.into_graph().unwrap()
    }

    fn program(&self) -> jai_ir::Program {
        jai_sema::resolve_graph(&self.graph()).unwrap()
    }

    fn native(&self, program: &jai_ir::Program) -> ExitStatus {
        let context = jai_codegen::Context::create();
        let target = jai_codegen::target::NativeTarget::new().unwrap();
        let module = jai_codegen::lower_for_target(&context, program, &target).unwrap();
        let object = self.0.join("program.o");
        target.write_object(&module, &object).unwrap();
        let executable = self.0.join("program");
        let linked = native_tools::clang_command()
            .arg(object)
            .arg("-o")
            .arg(&executable)
            .output()
            .unwrap();
        assert!(
            linked.status.success(),
            "{}",
            String::from_utf8_lossy(&linked.stderr)
        );
        let mut child = Command::new(executable).spawn().unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(status) = child.try_wait().unwrap() {
                return status;
            }
            if Instant::now() >= deadline {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("generated using declaration fixture timed out");
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

fn complete(source: &str) {
    let fixture = Fixture::new(source);
    let program = fixture.program();
    assert!(matches!(
        jai_vm::execute(&program, Limits::default()).outcome,
        Outcome::Complete(values) if matches!(values.as_slice(), [Value::Int(value)] if value.value() == 42)
    ));
    assert_eq!(fixture.native(&program).code(), Some(42));
}

#[test]
fn nested_and_exported_aliases_mutate_the_defining_global() {
    complete(
        "Inner::struct{value:int=1;} Outer::struct{inner:Inner;} state:Outer; using state.inner; main::()->int{value+=41;return state.inner.value;}",
    );
    let fixture = Fixture::new(
        "Lib::#import,file \"library.jai\"; main::()->int{Lib.value+=41;return Lib.read();}",
    );
    fs::write(
        fixture.0.join("library.jai"),
        "Pair::struct{value:int=1;} #scope_file state:Pair; #scope_export using state; read::()->int{return state.value;}",
    ).unwrap();
    let program = fixture.program();
    assert!(matches!(
        jai_vm::execute(&program, Limits::default()).outcome,
        Outcome::Complete(values) if matches!(values.as_slice(), [Value::Int(value)] if value.value() == 42)
    ));
    assert_eq!(fixture.native(&program).code(), Some(42));
}

#[test]
fn real_child_storage_initializes_once_before_mutable_aliases() {
    complete(
        r#"
        Pair::struct {value:int;}
        calls:int;
        producer::(item:*Pair)->*Pair {calls+=1;return item;}
        main::()->int {
            item:Pair;item.value=41;
            using alias:=producer(*item);
            value+=1;
            return item.value+(calls-1);
        }
    "#,
    );
    complete(
        r#"
        Pair::struct {value:int;extra:int;}
        main::()->int {using item:=Pair.{value=40,extra=1};value+=1;return item.value+extra;}
    "#,
    );
}

#[test]
fn canonical_file_and_local_members_reach_actual_storage() {
    complete(
        r#"
        using Choice::enum s32 {answer::40;}
        Pair::struct {value:int=1;}
        using state:Pair;
        main::()->int {value+=1;return cast(int)answer+state.value;}
    "#,
    );
    complete(
        r#"
        main::()->int {using Choice::enum s32 {answer::41;} offset::1;return cast(int)answer+offset;}
    "#,
    );
}

#[test]
fn discarded_file_nominal_owners_keep_distinct_nested_types_and_source_members() {
    complete(
        r#"
        using _ :: struct { First :: struct { value:int = 20; } }
        using _ :: struct { Second :: struct { value:int = 22; } }
        main :: () -> int { first:First; second:Second; return first.value + second.value; }
        "#,
    );
}

#[test]
fn retained_selection_and_mapper_alias_actual_fields() {
    complete(
        r#"
        Pair::struct {value:int=42;extra:int=7;}
        extra::0;
        main::()->int {using,except(extra) item:Pair;return value+extra;}
    "#,
    );
    complete(
        r#"
        Pair::struct {value:int=42;extra:int=7;}
        selected::()->[]string {return .["value"];}
        main::()->int {using,only(selected()) item:Pair;return value;}
    "#,
    );
    complete(
        r#"
        Pair::struct {value:int;}
        rename::(names:[]string) {names[0]="renamed";}
        main::()->int {using,map(rename) item:Pair;renamed=42;return item.value;}
    "#,
    );
}

#[test]
fn pointer_declaration_keeps_vm_and_native_null_guards() {
    let fixture =
        Fixture::new("Pair::struct{value:int;} main::()->int{using item:*Pair=null;return value;}");
    let program = fixture.program();
    assert_eq!(
        jai_vm::execute(&program, Limits::default()).outcome,
        Outcome::Failed(Error::NullPointer)
    );
    assert!(!fixture.native(&program).success());
}
