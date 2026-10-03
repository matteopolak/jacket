//! Independently authored source contracts for anonymous nominal field schemas.
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
            "jai-inline-types-{}-{}",
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
fn inline_nested_using_fields_and_contextual_enum_defaults_execute() {
    assert_eq!(
        run(r#"
        Options :: struct {
            output: enum u8 { OMIT; EXECUTABLE; OBJECT; } = .EXECUTABLE;
            using common: struct {
                support: enum u8 { AUTO; INIT; OMIT; } = .INIT;
                using inner: struct { amount: int = 40; }
            }
        }
        main :: () -> int { options: Options; return options.amount + cast(int)options.output + cast(int)options.support; }
    "#),
        42
    );
}

#[test]
fn inline_specializations_reuse_identity_and_keep_baked_defaults() {
    assert_eq!(
        run(r#"
        Box :: struct(T:Type, Fill:T=19) {
            entry: struct { value:T=Fill; next:*Box(T,Fill); }
            mode: enum u8 { OFF; ON; } = .ON;
        }
        main :: () -> int {
            first:Box(int); second:Box(int)=first;
            second.entry=first.entry;
            if second.entry.next != null return 1;
            small:Box(u8,3);
            return second.entry.value+cast(int)small.entry.value+cast(int)small.mode+19;
        }
    "#),
        42
    );
}

#[test]
fn inline_enum_defaults_apply_contextual_checked_and_wrapping_casts() {
    assert_eq!(
        run(r#"
        Options :: struct {
            bits:enum_flags u8 { READ; WRITE; } = xx 3;
            mode:enum u8 { ZERO; ONE; } = xx 1;
            wrapped:enum u8 { ZERO; } = xx,no_check 256;
        }
        main :: () -> int { options:Options;
            return cast(int)options.bits + cast(int)options.mode
                 + cast(int)options.wrapped + 38; }
        "#),
        42
    );
}

#[test]
fn inline_enums_keep_flags_aliases_and_explicit_width() {
    assert_eq!(
        run(r#"
        Options :: struct {
            bits: enum_flags u16 { READ; WRITE; BOTH :: READ | WRITE; } = .BOTH;
            mode: enum u8 #specified { ZERO :: 0; LAST :: 39; ALIAS :: LAST; } = .ALIAS;
        }
        main :: () -> int { value:Options; return cast(int)value.bits+cast(int)value.mode; }
    "#),
        42
    );
}

#[test]
fn inline_array_elements_keep_recursive_pointers_and_defaults() {
    assert_eq!(
        run(r#"
        Holder :: struct { values:[2] struct { amount:int=21; next:*Holder; }; }
        main :: () -> int { holder:Holder;
            if holder.values[1].next != null return 1;
            return holder.values[0].amount+holder.values[1].amount; }
    "#),
        42
    );
}

#[test]
fn inline_union_fields_use_real_overlapping_storage() {
    assert_eq!(
        run(r#"
        Holder :: struct { value:union { small:u8; wide:int; }; }
        main :: () -> int { holder:Holder=.{value=.{wide=42}}; return holder.value.wide; }
    "#),
        42
    );
}

#[test]
fn inline_nominal_type_identity_rejects_equal_spelled_distinct_sites() {
    for (source, expected) in [
        (
            "Box :: struct(N:int) { inner:struct { value:int=N; }; } main :: () { a:Box(1); b:Box(2); a.inner=b.inner; }",
            "nominal",
        ),
        (
            "Owner :: struct { left:struct { value:int; }; right:struct { value:int; }; } main :: () { x:Owner; x.left=x.right; }",
            "nominal",
        ),
        (
            "Owner :: struct { left:enum { ZERO; }; right:enum { ZERO; }; } main :: () { x:Owner; x.left=x.right; }",
            "nominal",
        ),
        (
            "Owner :: struct { value:struct { repeated:int; repeated:int; }; } main :: () {}",
            "duplicate record",
        ),
        (
            "Owner :: struct { value:enum u8 #specified { ZERO; }; } main :: () {}",
            "explicit value",
        ),
        (
            "Owner :: struct { value:enum bool { ZERO; }; } main :: () {}",
            "integer type",
        ),
        (
            "Owner :: struct { value:enum u8 { ZERO; } = xx 256; } main :: () {}",
            "out of range",
        ),
        (
            "Owner :: struct { value:enum u8 { LAST :: 256; }; } main :: () {}",
            "out of range",
        ),
        (
            "Owner :: struct { value:struct { self:Owner; }; } main :: () {}",
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
