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
            "jai-local-declarations-native-{}-{}",
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
                panic!("generated local declaration fixture timed out");
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
fn local_record_method_default_waits_for_its_own_run_initializer() {
    Fixture::new().check(
        "main::()->int{R::struct{value:int=#run seed();seed::()->int{return 21;}read::(input:R=R.{}) ->int{return input.value;}} return R.read()+R.read();}",
        42,
    );
}

#[test]
fn local_record_method_default_observes_its_own_completed_field_initializers() {
    Fixture::new().check(
        "main::()->int{R::struct{value:int=21;read::(input:R=R.{}) ->int{return input.value;}} return R.read()+R.read();}",
        42,
    );
}

#[test]
fn exported_method_alias_keeps_private_owner_defaults_and_seed_body() {
    let fixture = Fixture::new();
    fs::write(
        fixture.0.join("library.jai"),
        "#scope_module; R::struct{value:int=#run seed();seed::()->int{return 21;}read::(input:R=R.{}) ->int{return input.value;}} #scope_export; answer::()->int{return R.read();} alias::R.read;",
    )
    .unwrap();
    fixture.check(
        "Lib::#import,file \"library.jai\"; main::()->int{return Lib.answer()+Lib.alias();}",
        42,
    );
}

#[test]
fn local_record_method_default_observes_completed_construction_overrides() {
    Fixture::new().check(
        "main::()->int{R::struct{value:int=7; value=21; read::(input:R=R.{}) ->int{return input.value;}} return R.read()+R.read();}",
        42,
    );
}

#[test]
fn local_inferred_run_field_waits_for_the_source_seed_body() {
    Fixture::new().check(
        "main::()->int{R::struct{value:=#run seed();seed::()->int{return 21;}read::(input:R=R.{}) ->int{return input.value;}} return R.read()+R.read();}",
        42,
    );
}

#[test]
fn record_method_aliases_materialize_record_and_context_defaults_after_type_reservation() {
    Fixture::new().check(
        "#add_context marker:int=7; R::struct{value:int=14;} Methods::struct{read::(value:R=R.{},saved:#Context=.{})->int{return value.value+saved.marker;}} alias::Methods.read; main::()->int{return alias()+Methods.read();}",
        42,
    );
}

#[test]
fn record_method_run_defaults_wait_for_actual_source_procedure_bodies() {
    Fixture::new().check(
        "R::struct{value:int=7;} make::()->R{return R.{value=21};} Methods::struct{read::(value:R=#run make())->int{return value.value;}} alias::Methods.read; main::()->int{return alias()+Methods.read();}",
        42,
    );
}

#[test]
fn local_record_static_selection_and_completed_shape_assertions_execute_natively() {
    Fixture::new().check(
        r#"
        main :: () -> int {
            Record :: struct {
                #assert size_of(Record) == 16;
                before: int = 17;
                #if CHOICE { after: int = 25; }
                else { after: MissingType; #assert false; }
                #if true { CHOICE :: true; }
                #assert CHOICE;
            }
            record: Record;
            return record.before + record.after;
        }
        "#,
        42,
    );
}

#[test]
fn local_record_baked_conditions_run_guards_and_selected_namespaces_execute_natively() {
    Fixture::new().check(
        r#"
        enabled :: () -> bool { return true; }
        choose :: ($Enabled: bool) -> int {
            Record :: struct {
                #if Enabled {
                    LIMIT :: 17;
                    Inner :: struct { value: int = LIMIT; }
                    value: Inner;
                } else {
                    #if #run enabled() {
                        LIMIT :: 25;
                        Inner :: struct { value: int = LIMIT; }
                        value: Inner;
                    } else { value: MissingType; }
                }
                #assert size_of(Record) == 8;
            }
            record: Record;
            return record.value.value;
        }
        main :: () -> int { return choose(true) + choose(false); }
        "#,
        42,
    );
}

#[test]
fn local_records_enums_constants_and_nested_default_calls_agree_natively() {
    Fixture::new().check(
        r#"
        Node :: struct { value: u8; }
        main :: () -> int {
            Count :: Later + 1;
            Later :: 2;
            Node :: struct { next: *Node; values: [Count] int; value: int = 9; }
            Fruit :: enum u32 #specified { APPLE :: 5; PEAR :: APPLE + 2; }
            Choice :: struct { fruit := Fruit.PEAR; }
            BASE :: 7;
            add :: (value: int = 5) -> int { return value + BASE; }
            Alias :: add;
            node: Node;
            choice: Choice;
            if node.next != null return 1;
            node.values[2] = 7;
            shadow_value := 0;
            {
                Node :: struct { value: int = 4; }
                shadow: Node;
                shadow_value = shadow.value;
            }
            return Alias(value=5) + node.value + node.values[2]
                + node.values.count + cast(int) choice.fruit + shadow_value;
        }
        "#,
        42,
    );
}

#[test]
fn local_nominal_types_and_nested_bodies_work_inside_source_runs() {
    Fixture::new().check(
        r#"
        computed :: () -> int {
            Point :: struct { x: int = 42; }
            read :: (point: *Point) -> int { return point.x; }
            point: Point;
            return read(*point);
        }
        ANSWER :: #run computed();
        main :: () -> int { return ANSWER; }
        "#,
        42,
    );
}

#[test]
fn local_record_methods_preserve_definition_constants_and_named_defaults() {
    Fixture::new().check(
        r#"
        main :: () -> int {
            BASE :: 6;
            Box :: struct {
                value: int = #run default_value();
                default_value :: () -> int { return BASE; }
                add :: (self: *Box, amount: int = 36) -> int {
                    return self.value + amount;
                }
            }
            Alias :: Box.add;
            value: Box;
            return Alias(*value, amount=36);
        }
        "#,
        42,
    );
}

#[test]
fn local_record_namespaces_keep_nested_types_constants_and_enum_defaults() {
    Fixture::new().check(
        r#"
        main :: () -> int {
            Outer :: struct {
                LIMIT :: 7;
                Inner :: struct { value: int = LIMIT; }
                State :: enum u32 #specified { READY :: LIMIT; }
                Alias :: Inner;
                node: Alias;
                state: State = State.READY;
            }
            outer: Outer;
            inner: Outer.Inner;
            return outer.node.value + inner.value + cast(int) outer.state
                + Outer.LIMIT + 14;
        }
        "#,
        42,
    );
}

#[test]
fn local_reflection_retains_source_notes_and_initializer_defaults() {
    Fixture::new().check(
        r#"
        main :: () -> int {
            Pair :: struct { value: int = 42; @JsonIgnore }
            info := type_info(Pair);
            if info.members[0].notes.count != 1 return 1;
            if info.members[0].notes[0].count != 10 return 2;
            if info.members[0].notes[0][4] != #char "I" return 3;
            if (cast(int) info.status_flags & 4) == 0 return 4;
            pair := initializer_of(Pair);
            return pair.value;
        }
        "#,
        42,
    );
}
