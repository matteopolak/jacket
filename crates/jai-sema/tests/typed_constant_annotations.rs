//! Explicit constant annotations check canonical types in their defining scopes.
use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

struct Fixture(PathBuf);
impl Fixture {
    fn new(source: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "jai-typed-constant-annotation-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("main.jai"), source).unwrap();
        Self(root)
    }
    fn program(&self) -> Result<jai_ir::Program, String> {
        let graph = jai_modules::ModuleGraph::load(&self.0.join("main.jai"), Default::default())
            .map_err(|error| error.to_string())?;
        jai_sema::resolve_graph_with_options(
            &graph,
            &jai_sema::ResolveOptions {
                layout: Some(jai_types::LayoutPolicy::lp64()),
                ..Default::default()
            },
            &mut jai_vm::NoEffects,
        )
        .map_err(|error| error.to_string())
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn check(source: &str) {
    let program = Fixture::new(source).program().unwrap();
    let outcome = jai_vm::execute(&program, jai_vm::Limits::default()).outcome;
    let jai_vm::Outcome::Complete(values) = outcome else {
        panic!("{outcome:?}");
    };
    assert_eq!(values[0].integer().unwrap().value(), 42);
}

#[test]
fn named_integer_alias() {
    check("Word::u16; answer:Word:42; main::()->int{return cast(int)answer;}");
}

#[test]
fn float_annotation() {
    check("Small::float32; answer:Small:42.0; main::()->int{return cast(int)answer;}");
}

#[test]
fn string_annotation() {
    check("Text::string; answer:Text:\"x\"; main::()->int{return answer.count+41;}");
}

#[test]
fn contextual_record_annotation() {
    check("Pair::struct{value:int;} answer:Pair:.{value=42}; main::()->int{return answer.value;}");
}

#[test]
fn array_annotation() {
    check(
        "Values::[2]s16; answer:Values:.[20,22]; main::()->int{return cast(int)(answer[0]+answer[1]);}",
    );
}

#[test]
fn callback_annotation() {
    check(
        "Callback::(x:int)->int; identity::(x:int)->int{return x;} target:Callback:identity; main::()->int{return target(42);}",
    );
}

#[test]
fn local_alias_annotation() {
    check("Word::bool; main::()->int{Word::u16; answer:Word:42; return cast(int)answer;}");
}

#[test]
fn local_record_annotation() {
    check("main::()->int{Pair::struct{value:int;} answer:Pair:.{value=42}; return answer.value;}");
}

#[test]
fn distinct_annotation() {
    check("Counter::#type,distinct int; answer:Counter:42; main::()->int{return cast(int)answer;}");
}

#[test]
fn annotation_checks_integer_range() {
    let error = Fixture::new("Byte::u8; answer:Byte:256; main::()->int{return 0;}")
        .program()
        .unwrap_err();
    assert!(error.contains("range"), "{error}");
}

#[test]
fn annotation_checks_record_fields() {
    let error = Fixture::new(
        "Pair::struct{value:int;} answer:Pair:.{missing=42}; main::()->int{return 0;}",
    )
    .program()
    .unwrap_err();
    assert!(error.contains("field"), "{error}");
}

#[test]
fn annotation_checks_array_count() {
    let error = Fixture::new("Values::[2]int; answer:Values:.[1,2,3]; main::()->int{return 0;}")
        .program()
        .unwrap_err();
    assert!(error.contains("array"), "{error}");
}

#[test]
fn annotation_checks_callback_signature() {
    let error = Fixture::new("Callback::(x:int)->int; wrong::(x:int)->bool{return true;} target:Callback:wrong; main::()->int{return 0;}").program().unwrap_err();
    assert!(
        error.contains("typed constant has a different type"),
        "{error}"
    );
}

#[test]
fn annotated_record_constant_is_readonly() {
    let error = Fixture::new("Pair::struct{value:int;} answer:Pair:.{value=42}; main::()->int{answer.value=43;return answer.value;}").program().unwrap_err();
    assert!(error.contains("constant"), "{error}");
}

#[test]
fn declaration_annotations_supply_read_only_header_queries() {
    check(
        "Word::u16; answer:Word:42; copy::(value:type_of(answer))->int{return cast(int)value;} main::()->int{return copy(answer);}",
    );
}

#[test]
fn annotated_alias_constants_supply_array_type_counts() {
    check(
        "Word::u8; count:Word:2; Values::[count]int; answer:Values:.[20,22]; main::()->int{return answer[0]+answer[1];}",
    );
}

#[test]
fn typed_run_constants_keep_their_expected_type() {
    check(
        "Word::u16; make::()->Word{return 42;} answer:Word:#run make(); main::()->int{return cast(int)answer;}",
    );
}
