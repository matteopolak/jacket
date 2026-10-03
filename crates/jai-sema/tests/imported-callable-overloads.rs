//! Imported overloads retain original declarations and hermetic body scopes.
use jai_modules::{GraphOptions, ModuleGraph, SourceOverlay};
use jai_vm::{Limits, Outcome, Value};
use std::path::Path;

fn compile(files: &[(&str, &str)]) -> Result<jai_ir::Program, String> {
    let mut sources = SourceOverlay::new();
    for (name, text) in files {
        sources
            .insert(Path::new(name), text.as_bytes().to_vec())
            .unwrap();
    }
    let graph = ModuleGraph::load_with_provider(
        Path::new("/imported-overloads/main.jai"),
        GraphOptions::default(),
        &sources,
    )
    .map_err(|error| error.to_string())?;
    jai_sema::resolve_graph(&graph).map_err(|error| error.render(graph.sources()))
}

fn run(files: &[(&str, &str)]) {
    let program = compile(files).unwrap();
    let result = jai_vm::execute(&program, Limits::default());
    assert!(
        matches!(result.outcome, Outcome::Complete(ref values)
        if matches!(values.as_slice(), [Value::Int(value)] if value.value()==42)),
        "{result:?}"
    );
}

#[test]
fn local_and_imported_candidates_keep_definition_site_defaults_and_bodies() {
    run(&[
        (
            "/imported-overloads/main.jai",
            r#"
#import,file "library.jai";
DEFAULT::1;
pick::(value:bool)->int{return ifx value then 1 else 0;}
helper::(value:bool)->int{return 0;}
main::()->int{return pick()+pick(true);}
"#,
        ),
        (
            "/imported-overloads/library.jai",
            r#"
pick::(value:int=DEFAULT)->int{return helper(value);}
#scope_module
DEFAULT::41;
helper::(value:int)->int{return value;}
"#,
        ),
    ]);
}

#[test]
fn application_candidates_cannot_change_a_library_internal_call() {
    run(&[
        (
            "/imported-overloads/main.jai",
            r#"
#import,file "library.jai";
pick::(value:bool)->int{return 100;}
main::()->int{return inside()+20;}
"#,
        ),
        (
            "/imported-overloads/library.jai",
            r#"
pick::(value:int)->int{return 20;}
pick::(value:$T)->int{return 22;}
inside::()->int{return pick(true);}
"#,
        ),
    ]);
}

#[test]
fn imported_aliases_merge_without_wrapper_procedures_or_ambiguous_duplicates() {
    run(&[
        (
            "/imported-overloads/main.jai",
            r#"
#import,file "outer.jai";
#import,file "inner.jai";
pick::(value:bool)->int{return 1;}
main::()->int{return pick()+pick(true);}
"#,
        ),
        (
            "/imported-overloads/outer.jai",
            "#import,file \"inner.jai\";",
        ),
        (
            "/imported-overloads/inner.jai",
            "pick::original;#scope_module original::(value:int=41)->int{return value;}",
        ),
    ]);
}

#[test]
fn a_private_imported_overload_is_not_a_candidate_in_the_importing_scope() {
    let error = compile(&[
        ("/imported-overloads/main.jai", "#import,file \"library.jai\";main::()->int{return pick(true);}"),
        ("/imported-overloads/library.jai", "pick::(value:int)->int{return value;}pick::(value:u8)->int{return value;}#scope_module pick::(value:bool)->int{return 42;}"),
    ]).unwrap_err();
    assert!(error.contains("no overload matches"), "{error}");
    assert!(error.contains("main.jai:"), "{error}");
}

#[test]
fn two_imported_modules_contribute_distinct_callable_candidates() {
    run(&[
        (
            "/imported-overloads/main.jai",
            "#import,file \"left.jai\";#import,file \"right.jai\";main::()->int{return pick(20)+pick(true);}",
        ),
        (
            "/imported-overloads/left.jai",
            "pick::(value:int)->int{return value;}",
        ),
        (
            "/imported-overloads/right.jai",
            "pick::(value:bool)->int{return ifx value then 22 else 0;}",
        ),
    ]);
}

#[test]
fn equally_matching_imported_candidates_remain_ambiguous() {
    let error = compile(&[
        (
            "/imported-overloads/main.jai",
            "#import,file \"left.jai\";#import,file \"right.jai\";main::()->int{return pick(1);}",
        ),
        (
            "/imported-overloads/left.jai",
            "pick::(value:int)->int{return 20;}",
        ),
        (
            "/imported-overloads/right.jai",
            "pick::(value:int)->int{return 22;}",
        ),
    ])
    .unwrap_err();
    assert!(error.contains("ambiguous"), "{error}");
    assert!(error.contains("main.jai:"), "{error}");
}
