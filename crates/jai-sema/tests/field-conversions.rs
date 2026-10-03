use jai_modules::{GraphOptions, ModuleGraph};
use jai_sema::resolve_graph;
use jai_vm::{Limits, Outcome, Value};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

struct Fixture(PathBuf);

impl Fixture {
    fn new(source: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "jai-field-conversions-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("main.jai"), source).unwrap();
        Self(root)
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

fn run(source: &str) -> i128 {
    let fixture = Fixture::new(source);
    let program = resolve_graph(&fixture.graph())
        .unwrap_or_else(|error| panic!("field conversion source must resolve: {error:?}"));
    let execution = jai_vm::execute(&program, Limits::default());
    let Outcome::Complete(values) = execution.outcome else {
        panic!("field conversion source must execute: {execution:?}");
    };
    let [Value::Int(result)] = values.as_slice() else {
        panic!("expected one integer result: {values:?}");
    };
    result.value()
}

fn reject(source: &str, expected: &str) {
    let fixture = Fixture::new(source);
    let error = resolve_graph(&fixture.graph()).unwrap_err();
    assert!(
        error.message.contains(expected),
        "expected {expected:?}, got {error:?}"
    );
}

#[test]
fn value_call_initialization_assignment_and_return_take_marked_field_snapshots() {
    assert_eq!(
        run(r#"
Base :: struct { value: int; }
Derived :: struct { prefix: int; unmarked: Base; #as base: Base; }
read :: (value: Base) -> int { return value.value; }
convert :: (value: Derived) -> Base { return value; }
main :: () -> int {
    derived: Derived = .{prefix=31, unmarked=.{value=4}, base=.{value=7}};
    projected: Base = derived;
    derived.base.value = 19;
    observed := read(derived);
    assigned: Base;
    assigned = derived;
    assigned.value = 23;
    returned := convert(derived);
    derived.base.value = 29;
    return observed + projected.value + assigned.value + returned.value
        + derived.base.value + derived.prefix + derived.unmarked.value;
}

"#),
        132
    );
}

#[test]
fn projecting_a_record_producing_call_evaluates_the_source_once() {
    assert_eq!(
        run(r#"
Base :: struct { value: int; }
Derived :: struct { prefix: int; #as base: Base; }
make :: (calls: *int) -> Derived { calls.* += 1; return .{prefix=99, base=.{value=7}}; }
read :: (value: Base) -> int { return value.value; }
convert :: (calls: *int) -> Base { return make(calls); }
main :: () -> int {
    calls := 0;
    first: Base = make(*calls);
    second := read(make(*calls));
    third := convert(*calls);
    return calls * 100 + first.value + second + third.value;
}
"#),
        321
    );
}

#[test]
fn explicitly_named_derived_literals_convert_to_the_expected_base() {
    assert_eq!(
        run(
            "Base :: struct { value:int; } Derived :: struct { prefix:int=31; #as base:Base; } read :: (value:Base)->int { return value.value; } main :: ()->int { copy:Base=Derived.{base=.{value=7}}; return copy.value+read(Derived.{base=.{value=11}}); }"
        ),
        18
    );
}

#[test]
fn record_value_can_project_a_marked_pointer_field_without_pointee_dereference() {
    assert_eq!(
        run(r#"
Base :: struct { value:int=7; }
Wrapper :: struct { prefix:int=31; #as pointer:*Base; }
bump :: (value:*Base) { value.value+=3; }
convert :: (value:Wrapper)->*Base { return value; }
main :: ()->int {
    base:Base;
    wrapper:Wrapper=.{pointer=*base};
    pointer:*Base=wrapper;
    pointer.value+=2;
    bump(wrapper);
    returned:=convert(wrapper);
    returned.value+=5;
    empty:Wrapper=.{pointer=null};
    absent:*Base=empty;
    if absent != null return 99;
    return base.value+wrapper.prefix;
}
"#),
        48
    );
    reject(
        "Base :: struct { value:int; } Wrapper :: struct { #as pointer:*Base; } main :: () { value:Wrapper; base:Base=value; }",
        "type",
    );
}
#[test]
fn pointer_conversion_mutates_the_actual_nonfirst_marked_subobject() {
    assert_eq!(
        run(r#"
Base :: struct { value: int; }
Derived :: struct { prefix: int; unmarked: Base; #as base: Base; }
bump :: (value: *Base) { value.value += 5; }
convert :: (value: *Derived) -> *Base { return value; }
main :: () -> int {
    derived: Derived = .{prefix=44, unmarked=.{value=99}, base=.{value=8}};
    pointer: *Base = *derived;
    bump(*derived);
    pointer.value += 2;
    returned := convert(*derived);
    returned.value += 3;
    return derived.base.value + derived.prefix + derived.unmarked.value;
}
"#),
        161
    );
}

#[test]
fn conversion_and_using_name_promotion_are_independent() {
    assert_eq!(
        run(
            "Base :: struct { value:int=7; } Derived :: struct { #as base:Base; } read :: (value:Base)->int { return value.value; } main :: ()->int { value:Derived; return read(value); }"
        ),
        7
    );
    assert_eq!(
        run(
            "Base :: struct { value:int=7; } Derived :: struct { using base:Base; } main :: ()->int { value:Derived; value.value+=2; return value.base.value; }"
        ),
        9
    );
    reject(
        "Base :: struct { value:int; } Derived :: struct { #as base:Base; } main :: ()->int { value:Derived; return value.value; }",
        "member",
    );
    reject(
        "Base :: struct { value:int; } Derived :: struct { using base:Base; } read :: (value:Base)->int { return value.value; } main :: ()->int { value:Derived; return read(value); }",
        "type",
    );
}

#[test]
fn combined_as_using_converts_and_promotes_names() {
    assert_eq!(
        run(
            "Base :: struct { value:int=7; } Derived :: struct { prefix:int=3; using #as base:Base; } read :: (value:Base)->int { return value.value; } main :: ()->int { value:Derived; value.value+=2; return read(value)+value.prefix; }"
        ),
        12
    );
}

#[test]
fn inverse_conversions_and_unmarked_pointer_projection_are_rejected() {
    reject(
        "Base :: struct { value:int; } Derived :: struct { #as base:Base; } main :: () { base:Base; derived:Derived=base; }",
        "type",
    );
    reject(
        "Base :: struct { value:int; } Derived :: struct { #as base:Base; } main :: () { base:Base; derived:*Derived=*base; }",
        "type",
    );
    reject(
        "Base :: struct { value:int; } Derived :: struct { using base:Base; } main :: () { derived:Derived; base:*Base=*derived; }",
        "type",
    );
}

#[test]
fn unique_transitive_paths_project_values_and_pointers() {
    assert_eq!(
        run(r#"
Base :: struct { value:int=7; }
Middle :: struct { prefix:int=31; #as base:Base; }
Derived :: struct { prefix:int=41; #as middle:Middle; }
read :: (value:Base)->int { return value.value; }
bump :: (value:*Base) { value.value+=3; }
main :: ()->int { derived:Derived; copy:Base=derived; bump(*derived); return copy.value+read(derived)+derived.prefix+derived.middle.prefix; }
"#),
        89
    );
}

#[test]
fn different_targets_are_selected_without_ambiguity() {
    assert_eq!(
        run(
            "First :: struct { value:int=7; } Second :: struct { value:int=11; } Both :: struct { #as first:First; #as second:Second; } read_first :: (value:First)->int { return value.value; } read_second :: (value:Second)->int { return value.value; } main :: ()->int { value:Both; return read_first(value)+read_second(value); }"
        ),
        18
    );
}

#[test]
fn multiple_paths_to_one_target_are_ambiguous_only_when_conversion_is_used() {
    let declarations =
        "Base :: struct { value:int=7; } Both :: struct { #as first:Base; #as second:Base; } ";
    assert_eq!(
        run(&format!(
            "{declarations} main :: ()->int {{ value:Both; return value.first.value+value.second.value; }}"
        )),
        14
    );
    reject(
        &format!("{declarations} main :: () {{ value:Both; base:Base=value; }}"),
        "ambiguous implicit field conversion",
    );
    reject(
        &format!("{declarations} main :: () {{ value:Both; base:*Base=*value; }}"),
        "ambiguous implicit field conversion",
    );
    reject(
        "Base :: struct { value:int; } Middle :: struct { #as base:Base; } Both :: struct { #as direct:Base; #as middle:Middle; } main :: () { value:Both; base:Base=value; }",
        "ambiguous implicit field conversion",
    );
}

#[test]
fn local_records_use_the_same_directional_projection() {
    assert_eq!(
        run(
            "main :: ()->int { Base :: struct { value:int=7; } Derived :: struct { prefix:int=11; #as base:Base; } derived:Derived; copy:Base=derived; pointer:*Base=*derived; pointer.value+=3; return copy.value+derived.base.value+derived.prefix; }"
        ),
        28
    );
}

#[test]
fn parameterized_records_preserve_conversion_fields_per_concrete_type() {
    assert_eq!(
        run(
            "Base :: struct(T:Type) { value:T; } Derived :: struct(T:Type) { prefix:int; #as base:Base(T); } read :: (value:Base(int))->int { return value.value; } bump :: (value:*Base(int)) { value.value+=3; } main :: ()->int { value:Derived(int)=.{prefix=41,base=.{value=7}}; copy:Base(int)=value; bump(*value); return copy.value+read(value)+value.prefix; }"
        ),
        58
    );
    reject(
        "Base :: struct(T:Type) { value:T; } Derived :: struct(T:Type) { #as base:Base(T); } main :: () { value:Derived(int); copy:Base(u8)=value; }",
        "type",
    );
}

#[test]
fn global_field_and_parameter_defaults_project_record_constants() {
    assert_eq!(
        run(r#"
Base::struct {value:int;}
Derived::struct {prefix:int; #as base:Base;}
SOURCE::Derived.{prefix=99,base=.{value=7}};
Holder::struct {base:Base=SOURCE;}
global:Base=SOURCE;
read::(value:Base=SOURCE)->int {return value.value;}
main::()->int {holder:Holder; return global.value+holder.base.value+read();}
"#),
        21
    );
}

#[test]
fn overload_matching_prefers_exact_types_and_accepts_declared_conversion() {
    assert_eq!(
        run(
            "Base::struct{value:int=7;} Derived::struct{#as base:Base;} exact::(value:Base)->int{return 1;} exact::(value:Derived)->int{return 2;} converted::(value:Base)->int{return value.value;} converted::(value:int)->int{return value;} main::()->int{value:Derived;return exact(value)+converted(value);}",
        ),
        9
    );
}

#[test]
fn scalar_marked_fields_bind_declarations_and_scalar_returns() {
    assert_eq!(
        run(
            "IntegerBox::struct{#as value:int=7;} BooleanBox::struct{#as value:bool=true;} result::(value:IntegerBox)->int{return value;} truth::(value:BooleanBox)->bool{return value;} main::()->int{box:IntegerBox;boolean:BooleanBox;value:int=box;flag:bool=boolean;if truth(boolean)&&flag return result(box)+value;return 0;}",
        ),
        14
    );
}

#[test]
fn multiple_pointer_results_convert_after_one_call_snapshot() {
    assert_eq!(
        run(
            "Base::struct{value:int;} Derived::struct{prefix:int;#as base:Base;} calls:=0; pair::(a:*Derived,b:*Derived)->(*Derived,*Derived){calls+=1;return a,b;} main::()->int{first:Derived=.{base=.{value=7}};second:Derived=.{base=.{value=9}};a,b:*Base=pair(*first,*second);a.value+=1;b.value+=2;x,y:*Base;x,y=pair(*first,*second);x.value+=3;y.value+=4;return first.base.value+second.base.value+calls;}",
        ),
        28
    );
}

#[test]
fn float_marked_constants_use_the_field_type_before_evaluation() {
    assert_eq!(
        run(
            "FloatBox::struct{#as value:float64;} SOURCE::FloatBox.{value=7.5}; Holder::struct{value:float64=SOURCE;} global:float64=SOURCE; read::(value:float64=SOURCE)->float64{return value;} main::()->int{holder:Holder;return cast(int)(global+holder.value+read());}",
        ),
        22
    );
}
