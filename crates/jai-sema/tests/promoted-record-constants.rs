//! Declaration-site constants construct selected physical branches and defaults.
use jai_modules::{GraphOptions, ModuleGraph, SourceOverlay};
use std::path::Path;

fn graph(source: &str) -> ModuleGraph {
    let mut overlay = SourceOverlay::new();
    overlay
        .insert(
            Path::new("/jai-promoted-constants/main.jai"),
            source.as_bytes().to_vec(),
        )
        .unwrap();
    ModuleGraph::load_with_provider(
        Path::new("/jai-promoted-constants/main.jai"),
        GraphOptions::default(),
        &overlay,
    )
    .unwrap()
}

fn run(source: &str) {
    let program = jai_sema::resolve_graph(&graph(source))
        .unwrap_or_else(|error| panic!("{source}\n{error:?}"));
    let execution = jai_vm::execute(&program, jai_vm::Limits::default());
    assert!(
        matches!(execution.outcome, jai_vm::Outcome::Complete(ref values) if matches!(values.as_slice(), [jai_vm::Value::Int(value)] if value.value() == 42)),
        "{execution:?}"
    );
}

#[test]
fn jaison_number_and_string_defaults_use_the_selected_union_field() {
    for source in [
        r#"JSON_Type::enum u8{NULL::0;NUMBER::3;STRING::2;}
        JSON_Value::struct{type:JSON_Type;union{number:float64;str:string;}}
        Holder::struct{value:JSON_Value=.{type=.NUMBER,number=42.0};}
        main::()->int{holder:Holder;copy:=holder.value;if copy.type!=.NUMBER return 1;return cast(int)copy.number;}"#,
        r#"JSON_Type::enum u8{NULL::0;STRING::2;}
        JSON_Value::struct{type:JSON_Type;union{number:float64;str:string;}}
        Holder::struct{value:JSON_Value=.{type=.STRING,str="forty-two"};}
        main::()->int{holder:Holder;copy:=holder.value;if copy.type!=.STRING || copy.str!="forty-two" return 1;return 42;}"#,
    ] {
        run(source);
    }
}

#[test]
fn selected_branch_defaults_do_not_demand_unrelated_union_alternatives() {
    run(
        r#"Owner::struct{union{struct{x:int=1;y:int=22;}other:int=99;}}
        Holder::struct{value:Owner=.{x=20};}
        main::()->int{holder:Holder;return holder.value.x+holder.value.y;}"#,
    );
}

#[test]
fn explicit_using_initializer_and_ordered_overrides_supply_partial_defaults() {
    for source in [
        r#"Child::struct{x:int=1;y:int=2;}
        Owner::struct{using child:Child=.{x=7,y=22};}
        Holder::struct{value:Owner=.{x=20};}
        main::()->int{holder:Holder;return holder.value.x+holder.value.y;}"#,
        r#"Owner::struct{struct{x:int=1;y:int=2;}x=5;y=22;}
        Holder::struct{value:Owner=.{x=20};}
        main::()->int{holder:Holder;return holder.value.x+holder.value.y;}"#,
    ] {
        run(source);
    }
}

#[test]
fn promoted_constant_paths_reject_aliases_and_competing_union_alternatives() {
    for (source, expected) in [
        (
            "Owner::struct{struct{x:int;}} Holder::struct{value:Owner=.{x=20,x=22};} main::(){holder:Holder;}",
            "duplicate",
        ),
        (
            "Child::struct{x:int;} Owner::struct{using child:Child;} Holder::struct{value:Owner=.{child=.{x=20},x=22};} main::(){holder:Holder;}",
            "overlap",
        ),
        (
            "Owner::struct{union{x:int;y:int;}} Holder::struct{value:Owner=.{x=20,y=22};} main::(){holder:Holder;}",
            "competing union",
        ),
    ] {
        let error = jai_sema::resolve_graph(&graph(source)).unwrap_err();
        assert!(error.message.contains(expected), "{error:?}");
    }
}
