//! Source modifiers must execute before typed specialization publication.
use jai_modules::{GraphOptions, ModuleGraph};
use jai_sema::{ResolveOptions, resolve_graph_with_options};
use jai_types::{IntegerType, ScalarType, TypeKind, TypeView};
use jai_vm::{Limits, NoEffects, Outcome, Value};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new(source: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "jai-modifier-source-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).unwrap();
        std::fs::write(path.join("main.jai"), source).unwrap();
        Self(path)
    }
    fn graph(&self) -> ModuleGraph {
        ModuleGraph::load(&self.0.join("main.jai"), GraphOptions::default()).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn resolve(source: &str) -> Result<jai_ir::Program, jai_source::LocatedDiagnostic> {
    let fixture = Fixture::new(source);
    resolve_graph_with_options(
        &fixture.graph(),
        &ResolveOptions {
            layout: Some(jai_types::LayoutPolicy::lp64()),
            ..ResolveOptions::default()
        },
        &mut NoEffects,
    )
}
fn result(program: &jai_ir::Program) -> i128 {
    match jai_vm::execute(program, Limits::default()).outcome {
        Outcome::Complete(values) => match &values[..] {
            [Value::Int(value)] => value.value(),
            other => panic!("unexpected results {other:?}"),
        },
        other => panic!("modifier program did not execute: {other:?}"),
    }
}
#[test]
fn baked_modifier_changes_the_published_body_binding() {
    let program=resolve("count :: ($N:int) -> int #modify { if N < 8 N = 8; return true; } { return N; } main :: () -> int { return count(1) + count(9); }").unwrap();
    assert_eq!(result(&program), 17);
}
#[test]
fn modified_type_controls_coercion_and_final_specialization_deduplication() {
    let program=resolve("identity :: (a:$T) -> T #modify { T = s64; return true; } { return a; } main :: () -> int { a:u8=3; b:u16=4; return identity(a)+identity(b); }").unwrap();
    assert_eq!(result(&program), 7);
    let s64 = program.types().scalar(ScalarType::Int(IntegerType::S64));
    let matching = program
        .procedures()
        .iter()
        .filter(|procedure| {
            let Ok(TypeKind::Procedure(id)) = program.types().kind(procedure.signature) else {
                return false;
            };
            let signature = program.types().procedure_type(*id).unwrap();
            signature.parameters.as_ref() == [s64] && signature.results.as_ref() == [s64]
        })
        .count();
    assert_eq!(
        matching, 1,
        "modified equivalent bindings must share one runtime body"
    );
}
#[test]
fn modifier_introduces_a_typed_result_binding() {
    let program=resolve("widen :: (a:$T) -> $R #modify { R = s64; return true; } { return a; } main :: () -> int { n:u8=8; return widen(n); }").unwrap();
    assert_eq!(result(&program), 8);
}
#[test]
fn rejected_modifier_reports_its_actual_reason() {
    let error=resolve("reject :: (a:$T) -> T #modify { return false, \"blocked by modifier\"; } { return a; } main :: () -> int { return reject(1); }").unwrap_err();
    assert!(error.message.contains("blocked by modifier"), "{error:?}");
}
#[test]
fn modified_signature_must_recheck_original_argument_coercion() {
    assert!(resolve("numeric :: (a:$T) -> T #modify { T = s64; return true; } { return a; } main :: () -> int { return numeric(true); }").is_err());
}

#[test]
fn rejected_modifier_is_removed_before_overload_ranking() {
    let program = resolve("choose :: (a:$T) -> int #modify { return false, \"excluded\"; } { return 1; } choose :: (a:int) -> int { return 11; } main :: () -> int { return choose(7); }").unwrap();
    assert_eq!(result(&program), 11);
}

#[test]
fn modified_conversion_ranks_against_the_original_concrete_overload() {
    let program = resolve("choose :: (a:$T) -> int #modify { T=s64; return true; } { return 1; } choose :: (a:u8) -> int { return 12; } main :: () -> int { value:u8=7; return choose(value); }").unwrap();
    assert_eq!(result(&program), 12);
}

#[test]
fn discarded_baked_parameters_are_unreadable_in_modifier_bodies() {
    for baking in ["$", "$$"] {
        for operation in [
            "N=ignored;",
            "address:=*ignored;",
            "ignored=9;",
            "observed:=type_of(ignored);",
        ] {
            let source = format!(
                "probe::(#discard {baking}ignored:int,$N:int)->int \
                 #modify{{{operation} return true;}}{{return N;}} \
                 main::()->int{{return probe(7,1);}}"
            );
            let error = resolve(&source).unwrap_err();
            assert!(
                error.message.contains("#discard parameter cannot"),
                "{baking} {operation}: {error:?}"
            );
        }
    }
}

#[test]
fn modifiers_omit_discarded_baked_values_from_the_checked_call() {
    for baking in ["$", "$$"] {
        let program = resolve(&format!(
            "probe::(#discard {baking}ignored:int,$N:int)->int \
             #modify{{N=42;return true;}}{{return N;}} \
             main::()->int{{return probe(7,1);}}"
        ))
        .unwrap();
        assert_eq!(result(&program), 42);
        assert_eq!(program.procedures().len(), 2);
    }
    let program = resolve(
        "probe::(#discard $T:Type,$N:int)->int #modify{N=42;return true;}{return N;} \
         main::()->int{return probe(int,1);}",
    )
    .unwrap();
    assert_eq!(result(&program), 42);
}
