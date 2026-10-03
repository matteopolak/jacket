//! Execute only independently authored source and objects emitted by this compiler.
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
            "jai-field-conversions-native-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        Self(root)
    }

    fn check(&self, source: &str, expected: i32) {
        let input = self.0.join("main.jai");
        fs::write(&input, source).unwrap();
        let graph =
            jai_modules::ModuleGraph::load(&input, jai_modules::GraphOptions::default()).unwrap();
        let target = NativeTarget::new().unwrap();
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
            panic!("VM did not complete: {execution:?}");
        };
        let [jai_vm::Value::Int(value)] = values.as_slice() else {
            panic!("expected an integer result: {values:?}");
        };
        assert_eq!(value.value(), i128::from(expected));

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
                panic!("generated field conversion fixture timed out");
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
fn marked_fields_copy_values_and_project_real_pointer_offsets() {
    Fixture::new().check(
        r#"
        Base :: struct { value: int; }
        Derived :: struct { prefix: int; unmarked: Base; #as base: Base; }
        calls := 0;
        make :: () -> Derived { calls += 1; return .{prefix=11, unmarked=.{value=3}, base=.{value=7}}; }
        read :: (value: Base) -> int { return value.value; }
        mutate :: (value: *Base) { value.value += 5; }
        convert :: (value: Derived) -> Base { return value; }
        main :: () -> int {
            derived := make();
            copy: Base = derived;
            mutate(*derived);
            returned := convert(derived);
            observed := read(make());
            return copy.value + derived.base.value + returned.value + observed + calls + derived.unmarked.value + derived.prefix;
        }
        "#,
        54,
    );
}

#[test]
fn local_and_specialized_marked_paths_preserve_identity() {
    Fixture::new().check(
        r#"
        Base :: struct { value: int; }
        Wrapper :: struct (T: Type) { prefix: int; #as base: T; }
        mutate :: (value: *Base) { value.value += 3; }
        read :: (value: Base) -> int { return value.value; }
        main :: () -> int {
            Local :: struct { #as embedded: Wrapper(Base); }
            value: Local = .{embedded=.{prefix=77, base=.{value=8}}};
            mutate(*value);
            return read(value) + value.embedded.base.value;
        }
        "#,
        22,
    );
}

#[test]
fn constants_convert_in_global_field_and_parameter_defaults() {
    Fixture::new().check(
        "Base::struct{value:int;} Derived::struct{prefix:int;#as base:Base;} SOURCE::Derived.{prefix=99,base=.{value=7}}; Holder::struct{base:Base=SOURCE;} global:Base=SOURCE; read::(value:Base=SOURCE)->int{return value.value;} main::()->int{holder:Holder;return global.value+holder.base.value+read();}",
        21,
    );
}
