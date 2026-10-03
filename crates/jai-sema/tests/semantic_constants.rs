//! Typed reflection operands compose into immutable scalar values without effects.
use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

struct Fixture(PathBuf);
impl Fixture {
    fn new(source: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "jai-semantic-constants-{}-{}",
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
fn reflected_sizes_and_default_fields_compose_into_typed_scalar_constants() {
    check(
        "Pair::struct{value:int=40;} main::()->int{base::initializer_of(Pair).value;extra::size_of(s32)-2;ready::size_of(Pair)==size_of(int);if !ready return 1;return base+extra;}",
    );
}

#[test]
fn explicit_annotations_constrain_the_materialized_reflection_expression() {
    check("main::()->int{value:s32:cast(s32)(size_of(s32)+38);return cast(int)value;}");
}

#[test]
fn typed_reflection_arithmetic_keeps_the_ordinary_integer_conversion_rule() {
    let error = Fixture::new("main::()->int{value:s32:size_of(s32)+38;return cast(int)value;}")
        .program()
        .unwrap_err();
    assert!(
        error.contains("does not preserve the source type's entire range"),
        "{error}"
    );
}

#[test]
fn semantic_arithmetic_does_not_execute_an_implicit_runtime_call() {
    let error = Fixture::new("calls:int;tick::()->int{calls+=1;return 34;} main::()->int{value::size_of(int)+tick();return value;}")
        .program().unwrap_err();
    assert!(
        error.contains("procedure calls require explicit #run"),
        "{error}"
    );
}

#[test]
fn promoted_literal_bindings_keep_all_guard_producers_pure() {
    check(
        r#"Owner::struct{struct{x,y:int;}}
        main::()->int {
            #if Owner.{x=size_of(s32)+16,y=size_of(s32)+18}.x==20 {
                #assert Owner.{x=size_of(s32)+16,y=size_of(s32)+18}.y==22;
                return 42;
            } else { return missing_inactive_value; }
        }"#,
    );
    for producer in ["runtime_value", "tick()"] {
        let error = Fixture::new(&format!(
            "Owner::struct{{struct{{x,y:int;}}}} runtime_value:int=22; tick::()->int{{return 22;}} main::()->int{{#if Owner.{{x=size_of(s32)+16,y={producer}}}.x==20 return 42;else return 0;}}"
        ))
        .program()
        .unwrap_err();
        assert!(error.contains("requires compile-time values"), "{error}");
    }
}

#[test]
fn quoted_pure_arithmetic_errors_keep_the_original_source_in_both_insert_modes() {
    for insertion in ["#insert BODY;", "#insert,scope() BODY;"] {
        let fixture = Fixture::new(&format!(
            "#load \"quote.jai\"; main::()->int{{{insertion}return 0;}}"
        ));
        let quote = fixture.0.join("quote.jai");
        std::fs::write(&quote, "BODY::#code{bad::size_of(int)/0;};").unwrap();
        let graph = jai_modules::ModuleGraph::load(&fixture.0.join("main.jai"), Default::default())
            .unwrap();
        let error = jai_sema::resolve_graph_with_options(
            &graph,
            &jai_sema::ResolveOptions {
                layout: Some(jai_types::LayoutPolicy::lp64()),
                ..Default::default()
            },
            &mut jai_vm::NoEffects,
        )
        .unwrap_err();
        let original = graph.sources().get(error.location.source).unwrap();
        assert_eq!(original.path(), std::fs::canonicalize(&quote).unwrap());
        assert!(error.message.contains("ZeroDivisor"), "{error}");
    }
}
