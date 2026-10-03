//! Checked source prefixes preserve declarations, storage, and original names.
use jai_modules::{GraphDiscovery, GraphOptions, SourceOverlay};
use jai_vm::{Limits, NoEffects, Outcome, Value};
use std::path::Path;

fn compile(source: &str, library: &str) -> Result<jai_ir::Program, String> {
    let mut overlay = SourceOverlay::new();
    for (path, text) in [
        ("/using-prefix/main.jai", source),
        ("/using-prefix/library.jai", library),
    ] {
        overlay
            .insert(Path::new(path), text.as_bytes().to_vec())
            .unwrap();
    }
    let mut discovery = GraphDiscovery::new(
        Path::new("/using-prefix/main.jai"),
        GraphOptions::default(),
        &overlay,
    )
    .map_err(|error| error.to_string())?;
    for _ in 0..32 {
        if discovery
            .advance()
            .map_err(|error| error.to_string())?
            .is_complete()
        {
            let graph = discovery
                .into_graph()
                .map_err(|error| format!("{error:?}"))?;
            return jai_sema::resolve_graph(&graph).map_err(|error| error.render(graph.sources()));
        }
        let mut progress = false;
        let mut pending = Vec::new();
        let using = discovery.pending_using_requests();
        if !using.is_empty() {
            let outcome = jai_sema::resolve_discovery_using(
                discovery.graph(),
                &using,
                &Default::default(),
                &mut NoEffects,
            )
            .map_err(|error| error.to_string())?;
            for (id, decision) in outcome.decisions {
                discovery
                    .resolve_using(id, decision)
                    .map_err(|error| error.to_string())?;
                progress = true;
            }
            pending.extend(
                outcome
                    .pending
                    .into_iter()
                    .map(|pending| pending.diagnostic.to_string()),
            );
        }
        let conditions = discovery.pending_conditions().cloned().collect::<Vec<_>>();
        if !conditions.is_empty() {
            let outcome = jai_sema::resolve_discovery_conditions(
                discovery.graph(),
                &conditions,
                &Default::default(),
                &mut NoEffects,
            )
            .map_err(|error| error.to_string())?;
            for (id, selected) in outcome.decisions {
                discovery
                    .select_condition(id, selected)
                    .map_err(|error| error.to_string())?;
                progress = true;
            }
            pending.extend(
                outcome
                    .pending
                    .into_iter()
                    .map(|pending| pending.diagnostic.to_string()),
            );
        }
        if !progress {
            return Err(format!("source discovery pending: {pending:?}"));
        }
    }
    Err("source discovery did not converge".into())
}

fn execute(source: &str, library: &str, expected: i128) {
    let program = compile(source, library).unwrap();
    let execution = jai_vm::execute(&program, Limits::default());
    assert!(
        matches!(execution.outcome, Outcome::Complete(ref values)
        if matches!(values.as_slice(), [Value::Int(value)] if value.value() == expected)),
        "{execution:?}"
    );
}

#[test]
fn mapped_static_names_are_captured_only_by_later_declarations() {
    execute(
        r#"
renamed :: 1;
mapper :: (names:[]string) { names[0]="renamed"; }
main :: ()->int {
    before :: ()->int { return renamed; }
    Lib :: #import,file "library.jai";
    using,map(mapper) Lib;
    after :: ()->int { return renamed; }
    return before()+after();
}
"#,
        "value::41;",
        42,
    );
}

#[test]
fn checked_static_prefix_selects_guard_and_captures_selected_constant() {
    execute(
        r#"
ENABLED :: false;
Lib :: #import,file "library.jai";
main :: ()->int {
    using,only(ENABLED,VALUE) Lib;
    #if ENABLED { selected :: VALUE; } else { selected :: -1; }
    read :: ()->int { return selected; }
    return read();
}
"#,
        "ENABLED::true; VALUE::42;",
        42,
    );
}

#[test]
fn nested_using_prefix_restores_outer_names_after_local_procedure_resolution() {
    execute(
        r#"
value :: 2;
Lib :: #import,file "library.jai";
main :: ()->int {
    result:=0;
    { using Lib; read::()->int { return value; } result=read(); }
    return result+value;
}
"#,
        "value::40;",
        42,
    );
}

#[test]
fn source_prefix_replay_does_not_evaluate_a_runtime_target_twice() {
    execute(
        r#"
Box :: struct { value:int=7; other:int=11; }
choose :: (target:*Box,calls:*int)->*Box { calls.*+=1; return target; }
main :: ()->int {
    box:Box;
    calls:=0;
    using choose(*box,*calls);
    value+=3;
    other+=5;
    return calls*100+box.value+box.other;
}
"#,
        "",
        126,
    );
}

#[test]
fn pending_place_alias_cannot_bind_a_same_named_file_constant() {
    let error = compile(
        r#"
value :: 42;
Box :: struct { value:int=7; }
main :: ()->int {
    box:Box;
    using box;
    read :: ()->int { return value; }
    return read();
}

"#,
        "",
    )
    .unwrap_err();
    assert!(
        error.contains("using place alias") || error.contains("capture runtime local"),
        "{error}"
    );
}

#[test]
fn nested_local_procedure_consumes_its_real_owning_source_publication() {
    execute(
        r#"
Lib :: #import,file "library.jai";
mapper :: (names:[]string) { names[0]="renamed"; }
main :: ()->int {
    read :: ()->int { using,map(mapper) Lib; return renamed; }
    return read();
}
"#,
        "value::42;",
        42,
    );
}

#[test]
fn qualified_using_alias_keeps_the_original_nested_field_place() {
    execute(
        "Inner::struct{value:int=21;} Outer::struct{inner:Inner;} main::()->int{outer:Outer; using outer; inner.value+=21; return outer.inner.value;}",
        "",
        42,
    );
}

#[test]
fn mapped_zero_initialized_field_keeps_mutable_storage() {
    execute(
        "Record::struct{original:int;} mapper::(names:[]string){names[0]=\"renamed\";} main::()->int{record:Record; using,map(mapper) record; renamed=42; return record.original;}",
        "",
        42,
    );
}

#[test]
fn using_declaration_shadows_its_target_only_inside_the_original_scope() {
    execute(
        r#"
Pair::struct{value:int=40;}
main::()->int {
    item:=17;
    result:=0;
    { using item:Pair; value+=2; result=item.value; }
    return result+item-17;
}
"#,
        "",
        42,
    );
}

#[test]
fn using_declaration_static_members_follow_each_declarations_original_prefix() {
    execute(
        r#"
answer::1;
main::()->int {
    before::()->int{return answer;}
    using Choice::enum s32{answer::41;}
    after::()->int{return cast(int) answer;}
    return before()+after();
}
"#,
        "",
        42,
    );
}

#[test]
fn earlier_procedure_does_not_capture_a_later_using_declaration_alias() {
    execute(
        r#"
value::42;
Pair::struct{value:int=7;}
main::()->int {
    read::()->int{return value;}
    using item:Pair;
    return read();
}
"#,
        "",
        42,
    );
}

#[test]
fn later_procedure_cannot_replace_a_using_place_capture_with_a_file_constant() {
    let error = compile(
        r#"
value::42;
Pair::struct{value:int=7;}
main::()->int {
    using item:Pair;
    read::()->int{return value;}
    return read();
}

"#,
        "",
    )
    .unwrap_err();
    assert!(
        error.contains("using place alias") || error.contains("capture runtime local"),
        "{error}"
    );
    assert!(error.contains("main.jai"), "source identity lost: {error}");
}

#[test]
fn using_declaration_child_initializer_does_not_see_its_own_mapped_publication() {
    execute(
        r#"
answer::1;
mapper::(names:[]string){names[0]="answer";}
main::()->int {
    using,map(mapper) Local::struct{value::answer+40;}
    after::()->int{return answer;}
    return after()+1;
}
"#,
        "",
        42,
    );
}

#[test]
fn using_declaration_static_member_selects_only_the_original_active_import_branch() {
    execute(
        r#"
enabled::false;
main::()->int {
    using Flags::struct{enabled::true;}
    #if enabled {
        Lib::#import,file "library.jai";
        return Lib.VALUE;
    } else {
        Missing::#import,file "does-not-exist.jai";
        return Missing.VALUE;
    }
}
"#,
        "VALUE::42;",
        42,
    );
}

#[test]
fn repeated_all_promotions_reuse_the_canonical_global_field_view() {
    execute(
        "Lib::#import,file \"library.jai\"; main::()->int{using Lib; using Lib; value+=1; return Lib.record.value;}",
        "Record::struct{value:int=41;} using record:Record;",
        42,
    );
}
