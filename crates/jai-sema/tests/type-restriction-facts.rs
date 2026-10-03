//! Restrictions inspect canonical record facts while retaining supplied types.
use jai_modules::{GraphOptions, ModuleGraph, SourceOverlay};
use jai_vm::{Limits, Outcome, Value};
use std::path::Path;

fn compile(source: &str) -> Result<jai_ir::Program, String> {
    let path = Path::new("/type-restriction-facts/main.jai");
    let mut overlay = SourceOverlay::new();
    overlay.insert(path, source.as_bytes().to_vec()).unwrap();
    let graph = ModuleGraph::load_with_provider(path, GraphOptions::default(), &overlay)
        .map_err(|error| error.to_string())?;
    jai_sema::resolve_graph(&graph).map_err(|error| error.render(graph.sources()))
}

fn run(source: &str) {
    let program = compile(source).unwrap();
    let execution = jai_vm::execute(&program, Limits::default());
    assert!(
        matches!(execution.outcome, Outcome::Complete(ref values)
            if matches!(values.as_slice(), [Value::Int(value)] if value.value() == 42)),
        "{execution:?}"
    );
}

fn reject(source: &str, expected: &str) {
    let error = compile(source).unwrap_err();
    assert!(error.contains(expected), "expected {expected:?}: {error}");
    assert!(
        error.contains("main.jai:"),
        "missing source origin: {error}"
    );
}

#[test]
fn using_ancestry_retains_the_complete_supplied_record() {
    run(r#"
Base::struct{value:int;}
Middle::struct{using base:Base;}
Derived::struct{using middle:Middle;extra:int;}
read::(item:$T/Base)->int{return item.value+item.extra;}
main::()->int{item:Derived;item.value=40;item.extra=2;return read(item);}
"#);
}

#[test]
fn identical_nominal_type_and_pointer_pattern_keep_the_actual_type() {
    run(r#"
Base::struct{value:int;}
Derived::struct{using base:Base;extra:int;}
read::(item:*$T/Base)->int{return item.value+item.extra;}
identity::(item:$T/Base)->int{return item.value;}
main::()->int{base:Base;base.value=1;item:Derived;item.value=40;item.extra=1;
return read(*item)+identity(base);}
"#);
}

#[test]
fn interface_members_include_promoted_fields_and_ignore_order() {
    run(r#"
Required::struct{value:int;extra:int;}
Base::struct{value:int;}
Actual::struct{extra:int;using base:Base;unused:bool;}
read::(item:$T/interface Required)->int{return item.value+item.extra;}
main::()->int{item:Actual;item.value=40;item.extra=2;return read(item);}
"#);
}

#[test]
fn local_record_metadata_establishes_real_using_ancestry() {
    run(r#"
Base::struct{value:int;}
read::(item:$T/Base)->int{return item.value+item.extra;}
main::()->int{Local::struct{using base:Base;extra:int;}
item:Local;item.value=40;item.extra=2;return read(item);}
"#);
}

#[test]
fn ordinary_and_conversion_only_fields_do_not_establish_using_ancestry() {
    for field in ["base:Base;", "#as base:Base;"] {
        reject(
            &format!(
                "Base::struct{{value:int;}} Other::struct{{{field}}} \
                 read::(item:$T/Base)->int{{return 0;}} \
                 main::()->int{{item:Other;return read(item);}}"
            ),
            "nominal restriction",
        );
    }
}

#[test]
fn interfaces_require_each_member_with_its_exact_canonical_type() {
    for (actual, expected) in [
        ("other:int;", "interface requires member 'value'"),
        (
            "value:u32;",
            "interface member 'value' has a different type",
        ),
    ] {
        reject(
            &format!(
                "Required::struct{{value:int;}} Actual::struct{{{actual}}} \
                 read::(item:$T/interface Required)->int{{return 0;}} \
                 main::()->int{{item:Actual;return read(item);}}"
            ),
            expected,
        );
    }
}

#[test]
fn duplicate_promoted_member_paths_remain_ambiguous() {
    reject(
        "Required::struct{value:int;} Base::struct{value:int;} \
         Actual::struct{using first:Base;using second:Base;} \
         read::(item:$T/interface Required)->int{return 0;} \
         main::()->int{item:Actual;return read(item);}",
        "value",
    );
}
