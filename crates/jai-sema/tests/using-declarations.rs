//! Original declaration wrappers publish canonical names and actual storage aliases.
use jai_modules::{GraphDiscovery, GraphOptions, SourceOverlay};
use jai_vm::{Limits, NoEffects, Outcome, Value};
use std::path::Path;

fn program(source: &str) -> Result<jai_ir::Program, String> {
    let path = Path::new("/jai-using-declarations/main.jai");
    let mut overlay = SourceOverlay::new();
    overlay.insert(path, source.as_bytes().to_vec()).unwrap();
    let mut discovery = GraphDiscovery::new(path, GraphOptions::default(), &overlay)
        .map_err(|error| error.to_string())?;
    for _ in 0..32 {
        if discovery
            .advance()
            .map_err(|error| error.to_string())?
            .is_complete()
        {
            let graph = discovery
                .into_graph()
                .map_err(|_| "using discovery has unresolved source work".to_owned())?;
            return jai_sema::resolve_graph(&graph).map_err(|error| error.render(graph.sources()));
        }
        let requests = discovery.pending_using_requests();
        if requests.is_empty() {
            return Err("using fixture has no ready source request".into());
        }
        let outcome = jai_sema::resolve_discovery_using(
            discovery.graph(),
            &requests,
            &Default::default(),
            &mut NoEffects,
        )
        .map_err(|error| error.to_string())?;
        if let Some(pending) = outcome.pending.first() {
            return Err(pending.diagnostic.render(discovery.graph().sources()));
        }
        for (id, decision) in outcome.decisions {
            discovery
                .resolve_using(id, decision)
                .map_err(|error| error.to_string())?;
        }
    }
    Err("using fixture exceeded bounded discovery iterations".into())
}

fn answer(source: &str) {
    let program = program(source).unwrap();
    let outcome = jai_vm::execute(&program, Limits::default()).outcome;
    assert!(
        matches!(outcome, Outcome::Complete(ref values) if matches!(values.as_slice(), [Value::Int(value)] if value.value()==42)),
        "{outcome:?}"
    );
}

#[test]
fn inferred_pointer_initializer_runs_once_and_aliases_original_storage() {
    answer(
        r#"
        Pair::struct {value:int;}
        calls:int;
        producer::(item:*Pair)->*Pair {calls+=1;return item;}
        main::()->int {
            item:Pair; item.value=41;
            using alias:=producer(*item);
            value+=1;
            return item.value+(calls-1);
        }
    "#,
    );
}

#[test]
fn explicit_and_inferred_values_keep_actual_child_storage() {
    answer(
        r#"
        Pair::struct {value:int=41;}
        main::()->int {using item:Pair; value+=1;return item.value;}
    "#,
    );
    answer(
        r#"
        Pair::struct {value:int;extra:int;}
        main::()->int {using item:=Pair.{value=40,extra=1}; value+=1;return item.value+extra;}
    "#,
    );
}

#[test]
fn file_and_local_nominal_declarations_preserve_canonical_members() {
    answer(
        r#"
        using Choice::enum s32 {answer::40;}
        Pair::struct {value:int=1;}
        using state:Pair;
        main::()->int {value+=1;return cast(int)answer+state.value;}
    "#,
    );
    answer(
        r#"
        main::()->int {using Choice::enum s32 {answer::41;} offset::1;return cast(int)answer+offset;}
    "#,
    );
}

#[test]
fn discarded_file_nominal_owners_promote_from_independent_real_declarations() {
    answer(
        r#"
        using _ :: struct { FIRST :: 20; }
        using _ :: struct { SECOND :: 22; }
        main :: () -> int { return FIRST + SECOND; }
        "#,
    );
    answer(
        r#"
        using _ :: struct { First :: struct { value:int = 20; } }
        using _ :: struct { Second :: struct { value:int = 22; } }
        main :: () -> int { first:First; second:Second; return first.value + second.value; }
        "#,
    );
    let error = program("using _ :: struct { FIRST :: 42; } main :: () -> int { return _.FIRST; }")
        .unwrap_err();
    assert!(error.contains("unknown") && error.contains("_"), "{error}");
}

#[test]
fn declaration_filters_and_maps_retain_original_members() {
    answer(
        r#"
        Pair::struct {value:int=42;extra:int=7;}
        extra::0;
        main::()->int {using,except(extra) item:Pair;return value+extra;}
    "#,
    );
    answer(
        r#"
        Pair::struct {value:int=42;extra:int=7;}
        selected::()->[]string {return .["value"];}
        main::()->int {using,only(selected()) item:Pair;return value;}
    "#,
    );
    answer(
        r#"
        Pair::struct {value:int;}
        rename::(names:[]string) {names[0]="renamed";}
        main::()->int {using,map(rename) item:Pair;renamed=42;return item.value;}
    "#,
    );
}

#[test]
fn source_scope_conflicts_and_immutable_storage_do_not_disappear() {
    for (source, expected) in [
        (
            "Pair::struct{value:int;} main::()->int{{using item:Pair;} return value;}",
            "value",
        ),
        (
            "Pair::struct{value:int;} main::(){using item:Pair; value:=1;}",
            "value",
        ),
        (
            "Pair::struct{value:int;} main::(){using item::Pair.{value=42}; value=1;}",
            "read-only",
        ),
        ("main::(){using item:=42;}", "using"),
        (
            "Pair::struct{value:int;} main::(){using item:Pair; item:Pair;}",
            "duplicate",
        ),
        (
            "using Choice::enum{answer::42;} Other::enum{answer::42;} main::(){value:Choice=Other.answer;}",
            "type",
        ),
    ] {
        let error = program(source).unwrap_err();
        assert!(error.contains(expected), "expected {expected}: {error}");
        assert!(error.contains("main.jai"), "source location lost: {error}");
    }
}

#[test]
fn null_pointer_promotion_keeps_the_actual_vm_guard() {
    let program =
        program("Pair::struct{value:int;} main::()->int {using item:*Pair=null;return value;}")
            .unwrap();
    assert_eq!(
        jai_vm::execute(&program, Limits::default()).outcome,
        Outcome::Failed(jai_vm::Error::NullPointer)
    );
}
