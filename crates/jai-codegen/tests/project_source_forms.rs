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
            "jai-project-source-forms-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        fs::write(root.join("main.jai"), source).unwrap();
        Self(root)
    }

    fn graph(&self) -> jai_modules::ModuleGraph {
        jai_modules::ModuleGraph::load(&self.0.join("main.jai"), Default::default()).unwrap()
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
                panic!("generated project syntax fixture timed out");
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
fn semicolon_free_run_declarations_produce_real_checked_constants() {
    complete(
        r#"
        VERSION :: #run -> string { return "version"; }
        ANSWER :: #run -> int { return 34; }
        main :: () -> int {
            offset := #run -> int { return 1; }
            return ANSWER + VERSION.count + offset;
        }
    "#,
    );
    let fixture = Fixture::new("VALUE::#run->int {return Missing;} main::()->int{return VALUE;}");
    let error = jai_sema::resolve_graph(&fixture.graph())
        .unwrap_err()
        .to_string();
    assert!(error.contains("Missing"), "{error}");
}

#[test]
fn parenthesized_dereference_reads_and_writes_once_with_unary_precedence() {
    complete(
        r#"
        calls: int;
        pointer :: (value: *int) -> *int { calls += 1; return value; }
        main :: () -> int {
            value := 17;
            address := *value;
            ((.*) address) = 41;
            result := (.*)pointer(address) + 1;
            return result + (calls - 1);
        }
    "#,
    );
    complete(
        r#"
        choose :: (data: *void) -> string {
            return ifx (.*)(cast(*bool)data) "true" else "false";
        }
        main :: () -> int {
            yes := true;
            no := false;
            return choose(cast(*void)*yes).count + choose(cast(*void)*no).count + 33;
        }
    "#,
    );
}

#[test]
fn parenthesized_dereference_retains_type_and_null_validation() {
    let fixture = Fixture::new("main::()->int{return (.*)42;}");
    let error = jai_sema::resolve_graph(&fixture.graph())
        .unwrap_err()
        .to_string();
    assert!(error.contains("pointer"), "{error}");
    let fixture = Fixture::new("main::()->int { address:*int=null; return (.*)address; }");
    let program = fixture.program();
    assert_eq!(
        jai_vm::execute(&program, Limits::default()).outcome,
        Outcome::Failed(Error::NullPointer)
    );
    assert!(!fixture.native(&program).success());
}
