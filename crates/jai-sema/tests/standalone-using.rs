//! Source contracts execute only this compiler's checked IR in its Rust VM.
use jai_modules::{GraphDiscovery, GraphOptions, SourceOverlay};
use jai_sema::{ResolveOptions, resolve_graph_with_options};
use jai_types::LayoutPolicy;
use jai_vm::{Limits, NoEffects, Outcome, Value};
use std::path::Path;

const MAIN: &str = "/standalone-using/main.jai";
const LIBRARY: &str = "/standalone-using/library.jai";

fn compile(files: &[(&str, &str)]) -> Result<jai_ir::Program, String> {
    let mut sources = SourceOverlay::new();
    for &(path, source) in files {
        sources
            .insert(Path::new(path), source.as_bytes().to_vec())
            .unwrap();
    }
    let options = ResolveOptions {
        layout: Some(LayoutPolicy::lp64()),
        ..ResolveOptions::default()
    };
    let mut discovery =
        GraphDiscovery::new(Path::new(files[0].0), GraphOptions::default(), &sources)
            .map_err(|error| error.to_string())?;
    let mut ready = false;
    for _ in 0..32 {
        if discovery
            .advance()
            .map_err(|error| error.to_string())?
            .is_complete()
        {
            ready = true;
            break;
        }
        let requests = discovery.pending_using_requests();
        let outcome = jai_sema::resolve_discovery_using(
            discovery.graph(),
            &requests,
            &options,
            &mut NoEffects,
        )
        .map_err(|error| error.to_string())?;
        if outcome.decisions.is_empty() {
            return Err(format!(
                "using source discovery pending: {:?}",
                outcome.pending
            ));
        }
        for (id, decision) in outcome.decisions {
            discovery
                .resolve_using(id, decision)
                .map_err(|error| error.to_string())?;
        }
    }
    if !ready {
        return Err("using source discovery did not converge".into());
    }
    let graph = discovery
        .into_graph()
        .map_err(|_| "using source discovery is still incomplete".to_owned())?;
    resolve_graph_with_options(
        &graph,
        &ResolveOptions {
            layout: Some(LayoutPolicy::lp64()),
            ..ResolveOptions::default()
        },
        &mut NoEffects,
    )
    .map_err(|error| error.to_string())
}

fn run_files(files: &[(&str, &str)]) -> i128 {
    let program = compile(files).unwrap();
    let execution = jai_vm::execute(&program, Limits::default());
    let Outcome::Complete(values) = execution.outcome else {
        panic!("using fixture did not complete: {execution:?}");
    };
    let [Value::Int(value)] = values.as_slice() else {
        panic!("expected one integer result: {values:?}");
    };
    value.value()
}

fn run(source: &str) -> i128 {
    run_files(&[(MAIN, source)])
}

fn reject_files(files: &[(&str, &str)], message: &str) {
    let error = compile(files).unwrap_err();
    assert!(error.contains(message), "expected {message:?}, got {error}");
}

#[test]
fn enum_using_publishes_nominal_members_and_leaves_unrelated_enums_distinct() {
    assert_eq!(
        run(
            "State :: enum u8 #specified { NONE::0; READY::7; } read :: (state:State)->int { return cast(int) state; } main :: ()->int { using State; value:=READY; return read(value)+cast(int) NONE; }"
        ),
        7
    );
    reject_files(
        &[(
            MAIN,
            "State :: enum u8 #specified { NONE::0; READY::7; } Other :: enum u8 #specified { NONE::0; READY::7; } main :: () { using State; value:Other=READY; }",
        )],
        "type",
    );
}

#[test]
fn nested_record_using_mutates_the_original_place_and_restores_outer_names() {
    assert_eq!(
        run(
            "Inner :: struct { value:int=7; } Outer :: struct { prefix:int=11; inner:Inner; } main :: ()->int { value:=99; outer:Outer; { using outer.inner; value+=5; } return outer.inner.value+outer.prefix+value; }"
        ),
        122
    );
}

#[test]
fn pointer_producing_using_target_is_evaluated_once_for_all_promoted_fields() {
    assert_eq!(
        run(r#"
Box :: struct { value:int=7; other:int=11; }
choose :: (target:*Box, calls:*int)->*Box { calls.*+=1; return target; }
main :: ()->int {
    box:Box;
    calls:=0;
    using choose(*box,*calls);
    value+=3;
    other+=5;
    return calls*100+box.value+box.other;
}
"#),
        126
    );
}

#[test]
fn record_producing_using_target_is_evaluated_once_and_read_only() {
    assert_eq!(
        run(
            "Box :: struct { value:int=7; other:int=11; } make :: (calls:*int)->Box { calls.*+=1; return .{}; } main :: ()->int { calls:=0; using make(*calls); return calls*100+value+other; }"
        ),
        118
    );
    assert_eq!(
        run("Box :: struct { value:int=7; } main :: ()->int { using Box.{}; return value; }"),
        7
    );
    reject_files(
        &[(
            MAIN,
            "Box :: struct { value:int=7; } main :: () { using Box.{}; value+=1; }",
        )],
        "read-only",
    );
}

#[test]
fn using_read_only_array_iterator_preserves_its_storage_policy() {
    assert_eq!(
        run(
            "Box :: struct { value:int; } main :: ()->int { values:[2]Box=.[.{value=7},.{value=11}]; total:=0; for element:values { using element; total+=value; } return total; }"
        ),
        18
    );
    reject_files(
        &[(
            MAIN,
            "Box :: struct { value:int; } main :: () { values:[1]Box=.[.{value=7}]; for element:values { using element; value+=1; } }",
        )],
        "read-only",
    );
}

#[test]
fn direct_only_and_except_names_filter_promotions_and_allow_missing_selectors() {
    assert_eq!(
        run(
            "Box :: struct { x:int=7; y:int=11; z:int=13; } main :: ()->int { box:Box; total:=0; { using,only(x,Missing) box; x+=3; total+=x; } { using,except(x,Missing) box; y+=5; z+=7; total+=y+z; } return total+box.x+box.y+box.z; }"
        ),
        92
    );
    reject_files(
        &[(
            MAIN,
            "Box :: struct { x:int=7; y:int=11; } main :: ()->int { box:Box; using,only(x) box; return y; }",
        )],
        "y",
    );
    reject_files(
        &[(
            MAIN,
            "Box :: struct { x:int=7; y:int=11; } main :: ()->int { box:Box; using,except(x) box; return x; }",
        )],
        "x",
    );
}

#[test]
fn typed_string_array_selectors_and_source_run_results_select_real_members() {
    assert_eq!(
        run(r#"
Box :: struct { x:int=7; y:int=11; z:int=13; }
names :: ()->[1]string { return string.["y"]; }
main :: ()->int {
    box:Box;
    total:=0;
    PICK :: string.["x","Missing"];
    { using,only PICK box; total+=x; }
    { using,only #run names() box; total+=y; }
    { using,except string.["x","y"] box; total+=z; }
    return total;
}
"#),
        31
    );
    reject_files(
        &[(
            MAIN,
            "Box :: struct { x:int; } main :: () { box:Box; using,only int.[1] box; }",
        )],
        "string",
    );
}

#[test]
fn mapper_mutates_string_slots_to_rename_members_and_omit_empty_names() {
    assert_eq!(
        run(r#"
Box :: struct { x:int=7; hidden:int=11; y:int=13; }
rename :: (names:[]string) {
    for i:0..names.count-1 {
        if names[i]=="x" names[i]="shown";
        if names[i]=="hidden" names[i]="";
    }
}
main :: ()->int {
    box:Box;
    using,map(rename) box;
    shown+=5;
    y+=3;
    return box.x+box.hidden+box.y;
}
"#),
        39
    );
}

#[test]
fn map_does_not_publish_original_or_omitted_member_names() {
    let declarations = "Box :: struct { x:int=7; hidden:int=11; } rename :: (names:[]string) { for i:0..names.count-1 { if names[i]==\"x\" names[i]=\"shown\"; if names[i]==\"hidden\" names[i]=\"\"; } } ";
    for name in ["x", "hidden"] {
        reject_files(
            &[(
                MAIN,
                &format!(
                    "{declarations} main :: ()->int {{ box:Box; using,map(rename) box; return {name}; }}"
                ),
            )],
            name,
        );
    }
}

#[test]
fn duplicate_promoted_names_are_rejected_in_one_lexical_scope() {
    let error = compile(&[(MAIN, "Box :: struct { value:int; } main :: () { first:Box; second:Box; using first; using second; }")]).unwrap_err();
    assert!(
        error.contains("duplicate") || error.contains("redecl") || error.contains("conflict"),
        "unexpected promotion conflict: {error}"
    );
}

#[test]
fn module_namespace_using_obeys_export_privacy_and_selector_filters() {
    let library = "#scope_file; Hidden::99; #scope_export; VISIBLE::7; EXTRA::11;";
    assert_eq!(
        run_files(&[
            (
                MAIN,
                "Library :: #import,file \"library.jai\"; main :: ()->int { using,only(VISIBLE,Missing) Library; return VISIBLE; }"
            ),
            (LIBRARY, library),
        ]),
        7
    );
    for (source, missing) in [
        (
            "Library :: #import,file \"library.jai\"; main :: ()->int { using Library; return Hidden; }",
            "Hidden",
        ),
        (
            "Library :: #import,file \"library.jai\"; main :: ()->int { using,except(VISIBLE) Library; return VISIBLE; }",
            "VISIBLE",
        ),
    ] {
        reject_files(&[(MAIN, source), (LIBRARY, library)], missing);
    }
}

#[test]
fn bare_module_using_deduplicates_but_modified_publication_conflicts() {
    assert_eq!(
        run_files(&[
            (
                MAIN,
                "Library :: #import,file \"library.jai\"; main :: ()->int { using Library; using Library; return VISIBLE; }"
            ),
            (LIBRARY, "VISIBLE::7;"),
        ]),
        7
    );
    assert!(compile(&[
        (MAIN, "Library :: #import,file \"library.jai\"; main :: ()->int { using,only(VISIBLE) Library; using,only(VISIBLE) Library; return VISIBLE; }"),
        (LIBRARY, "VISIBLE::7;"),
    ]).is_err());
}

#[test]
fn module_operator_selectors_publish_original_operator_ids_in_lexical_scope() {
    let library = "Box :: struct { value:int; } operator + :: (a:Box,b:Box)->Box { return .{value=a.value+b.value}; } operator - :: (a:Box,b:Box)->Box { return .{value=a.value-b.value}; }";
    assert_eq!(
        run_files(&[
            (
                MAIN,
                "Library :: #import,file \"library.jai\"; main :: ()->int { using,only string.[\"+\"] Library; a:Library.Box=.{value=7}; b:Library.Box=.{value=11}; result:=a+b; return result.value; }"
            ),
            (LIBRARY, library),
        ]),
        18
    );
    assert!(compile(&[
        (MAIN, "Library :: #import,file \"library.jai\"; main :: ()->int { using,only string.[\"+\"] Library; a:Library.Box=.{value=7}; b:Library.Box=.{value=11}; result:=a-b; return result.value; }"),
        (LIBRARY, library),
    ]).is_err());
    assert!(compile(&[
        (MAIN, "Library :: #import,file \"library.jai\"; main :: ()->int { { using Library; } a:Library.Box=.{value=7}; b:Library.Box=.{value=11}; result:=a+b; return result.value; }"),
        (LIBRARY, library),
    ]).is_err());
}

#[test]
fn module_using_never_publishes_private_operators() {
    let library = "Box :: struct { value:int; } #scope_file; operator + :: (a:Box,b:Box)->Box { return .{value=a.value+b.value}; }";
    assert!(compile(&[
        (MAIN, "Library :: #import,file \"library.jai\"; main :: ()->int { using Library; a:Library.Box=.{value=7}; b:Library.Box=.{value=11}; result:=a+b; return result.value; }"),
        (LIBRARY, library),
    ]).is_err());
}

#[test]
fn nested_same_named_iterators_keep_their_original_loop_types() {
    assert_eq!(
        run(
            "Outer::struct{outer:int;} Inner::struct{inner:int;} main::()->int{a:[1]Outer=.[.{outer=7}]; b:[1]Inner=.[.{inner=35}]; total:=0; for element:a {using element; total+=outer; for element:b {using element; total+=inner;}} return total;}"
        ),
        42
    );
}

#[test]
fn stallable_selector_requires_a_live_result_continuation() {
    reject_files(
        &[(
            MAIN,
            "Box::struct{value:int;} names::()->[]string{return .[\"value\"];} main::(){box:Box; using,only #run,stallable names() box;}",
        )],
        "live-result continuation consumer",
    );
}

#[test]
fn file_pointer_storage_using_requires_a_real_initialized_capture() {
    reject_files(
        &[(
            MAIN,
            "Pair::struct{value:int;} pointer:*Pair=null; using pointer; main::(){}",
        )],
        "initialized pointer capture",
    );
}
