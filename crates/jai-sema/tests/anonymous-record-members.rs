//! Independently authored source contracts for unnamed aggregate member contracts.
use jai_modules::{GraphOptions, ModuleGraph};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
struct Fixture(PathBuf);
impl Fixture {
    fn new(source: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "jai-anonymous-members-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
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
fn run(source: &str) -> i128 {
    let fixture = Fixture::new(source);
    let program = jai_sema::resolve_graph(&fixture.graph())
        .unwrap_or_else(|error| panic!("{source}\n{error:?}"));
    let execution = jai_vm::execute(&program, jai_vm::Limits::default());
    let jai_vm::Outcome::Complete(values) = execution.outcome else {
        panic!("{execution:?}");
    };
    let [jai_vm::Value::Int(value)] = values.as_slice() else {
        panic!("integer result required");
    };
    value.value()
}

#[test]
fn anonymous_struct_members_keep_defaults_and_promote_real_fields() {
    assert_eq!(
        run(r#"
Owner :: struct { before:u8; struct { x:int=20; y:int=22; } after:u8; }
main :: () -> int { value:Owner; value.before=7; value.after=9; return value.x+value.y; }
"#),
        42
    );
}

#[test]
fn anonymous_nested_union_views_share_bytes_and_keep_surrounding_fields() {
    assert_eq!(
        run(r#"
Vector :: struct { before:int; union { struct { x,y:int; } struct { r,g:int; } } after:int; }
main :: () -> int {
    value:Vector=---; value.before=5; value.after=7; value.x=20; value.y=22;
    if value.before!=5 || value.after!=7 return 1;
    return value.r+value.g;
}
"#),
        42
    );
}

#[test]
fn anonymous_member_paths_remain_projected_storage_for_using_parameters() {
    assert_eq!(
        run(r#"
Owner :: struct { struct { amount:int=21; } }
add :: (using value:*Owner) { amount+=21; }
main :: () -> int { value:Owner; add(*value); return value.amount; }
"#),
        42
    );
}

#[test]
fn anonymous_members_keep_specialization_defaults_and_recursive_pointers() {
    assert_eq!(
        run(r#"
Box :: struct(T:Type, Fill:T) { struct { amount:T=Fill; next:*Box(T,Fill); } }
main :: () -> int {
    first:Box(int,20); second:Box(int,20)=first; small:Box(u8,22);
    if second.next!=null return 1;
    return second.amount+cast(int)small.amount;
}
"#),
        42
    );
}

#[test]
fn local_anonymous_members_capture_definition_scope() {
    assert_eq!(
        run(r#"
main :: () -> int {
    Count::20;
    Local::struct { struct { value:int=Count; } }
    one:Local; two:Local=one;
    { Count::22; Inner::struct { struct { value:int=Count; } } value:Inner; return two.value+value.value; }
}
"#),
        42
    );
}

#[test]
fn duplicate_promotions_and_by_value_cycles_are_rejected() {
    for (source, expected) in [
        (
            "Bad::struct { union { a:int=4; b:int; } } main::()->int{value:Bad;return value.a;}",
            "identical zero storage",
        ),
        (
            "Bad::union { a:int; b:int; } main::()->int{value:Bad=.{};return value.a;}",
            "exactly one explicit alternative",
        ),
        (
            "main::(){enabled::true; R::struct{union{enabled:bool;} #if enabled{extra:int;}}}",
            "runtime",
        ),
        (
            "Bad::struct { a:int; struct { a:int; } } main::(){}",
            "ambiguous",
        ),
        (
            "Bad::struct { struct { a:int; } struct { a:int; } } main::(){}",
            "ambiguous",
        ),
        (
            "Bad::struct { struct { self:Bad; } } main::(){}",
            "itself by value",
        ),
    ] {
        let fixture = Fixture::new(source);
        let error = jai_sema::resolve_graph(&fixture.graph()).unwrap_err();
        assert!(
            error.message.contains(expected),
            "expected {expected}: {error:?}"
        );
    }
}

#[test]
fn anonymous_union_uniform_zero_defaults_initialize_checked_storage() {
    assert_eq!(
        run(r#"
Owner::struct { union { flag:bool; number:float64; pointer:*Owner; } }
main::()->int { value:Owner; if value.flag || value.pointer!=null || value.number!=0.0 return 1; value.number=42.0; return cast(int)value.number; }
"#),
        42
    );
}

#[test]
fn inactive_anonymous_child_fields_do_not_shadow_outer_guard_constants() {
    assert_eq!(
        run(
            r#"main::()->int { enabled::true; R::struct { struct{#if false{enabled:bool;} value:int=42;} #if enabled{marker:u8;} } value:R; return value.value; }"#
        ),
        42
    );
}

#[test]
fn positional_initialization_uses_real_unnamed_physical_field_ids() {
    assert_eq!(
        run(r#"
Owner::struct { struct { x:int; y:int; } tail:int=7; }
main::()->int { first:Owner=.{.{20,22}}; second:Owner=first; if second.tail!=7 return 1; return second.x+second.y; }
"#),
        42
    );
}

#[test]
fn construction_overrides_follow_promoted_physical_field_paths() {
    assert_eq!(
        run(r#"
Owner::struct { struct { x:int=1; y:int=2; } x=20; y=22; }
main::()->int { value:Owner; return value.x+value.y; }
"#),
        42
    );
}
