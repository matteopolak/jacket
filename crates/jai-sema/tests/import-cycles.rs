//! Semantic/runtime evidence for actual declarations connected by module cycles.
use jai_modules::{GraphOptions, ModuleGraph, SourceOverlay};
use jai_sema::ResolveOptions;
use jai_types::LayoutPolicy;
use jai_vm::{Limits, NoEffects, Outcome, Value};
use std::path::Path;

fn check(files: &[(&str, &str)]) {
    let mut sources = SourceOverlay::new();
    for (name, source) in files {
        sources
            .insert(
                Path::new(&format!("/semantic-cycles/{name}")),
                source.as_bytes().to_vec(),
            )
            .unwrap();
    }
    let graph = ModuleGraph::load_with_provider(
        Path::new("/semantic-cycles/main.jai"),
        GraphOptions {
            import_dirs: vec!["/semantic-cycles".into()],
        },
        &sources,
    )
    .unwrap();
    let program = jai_sema::resolve_graph_with_options(
        &graph,
        &ResolveOptions {
            layout: Some(LayoutPolicy::lp64()),
            ..Default::default()
        },
        &mut NoEffects,
    )
    .unwrap();
    let execution = jai_vm::execute(&program, Limits::default());
    assert!(
        matches!(&execution.outcome, Outcome::Complete(values) if matches!(values.as_slice(), [Value::Int(value)] if value.value()==42)),
        "cycle fixture must return42: {execution:?}"
    );
}

#[test]
fn imported_procedures_resolve_names_in_their_original_cyclic_modules() {
    check(&[
        (
            "main.jai",
            "A::#import \"A\"; main::()->int{return A.entry();}",
        ),
        (
            "A.jai",
            "B::#import \"B\"; base::40; entry::()->int{return B.answer();}",
        ),
        (
            "B.jai",
            "A::#import \"A\"; answer::()->int{return A.base+2;}",
        ),
    ]);
}

#[test]
fn mutually_recursive_nominal_pointer_fields_keep_real_type_identities() {
    check(&[
        (
            "main.jai",
            "A::#import \"A\"; main::()->int{return A.answer();}",
        ),
        (
            "A.jai",
            "B::#import \"B\"; Left::struct{link:*B.Right;value:int;} answer::()->int{return size_of(Left)+size_of(B.Right)+10;}",
        ),
        (
            "B.jai",
            "A::#import \"A\"; Right::struct{link:*A.Left;value:int;}",
        ),
    ]);
}

#[test]
fn late_reexports_across_cycles_retain_the_original_constants() {
    check(&[
        (
            "main.jai",
            "#import \"A\"; #import \"B\"; main::()->int{return left+right;}",
        ),
        ("A.jai", "#import \"B\"; #load \"late.jai\";"),
        ("B.jai", "#import \"A\"; right::2;"),
        ("late.jai", "left::40;"),
    ]);
}
