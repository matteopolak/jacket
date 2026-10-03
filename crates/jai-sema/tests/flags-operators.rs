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
            "jai-flags-operators-{}-{}",
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
        .unwrap_or_else(|error| panic!("flags operator source must resolve: {error:?}"));
    let execution = jai_vm::execute(&program, Limits::default());
    let Outcome::Complete(values) = execution.outcome else {
        panic!("flags operator source must execute: {execution:?}");
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
fn weak_zero_defaults_preserve_flags_identity_without_a_zero_member() {
    assert_eq!(
        run(r#"
Flags :: enum_flags u16 { FIRST :: 1; SECOND :: 2; }
ZERO :: 1 - 1;
stored: Flags = ZERO;
State :: struct { flags: Flags = 0; }
read :: (flags: Flags = ZERO) -> int { if flags == 0 return 40; return 1; }
main :: () -> int {
    state: State;
    empty: Flags = ZERO;
    if stored == 0 && state.flags == 0 && empty == 0 return read() + 2;
    return 1;
}
"#),
        42
    );
}

#[test]
fn integer_defaults_do_not_gain_general_nominal_enum_conversion() {
    for source in [
        "Flags :: enum_flags u16 { FIRST :: 1; } read :: (flags:Flags=1) {} main :: () {}",
        "Flags :: enum_flags u16 { FIRST :: 1; } ZERO:u16:0; read :: (flags:Flags=ZERO) {} main :: () {}",
        "Choice :: enum u16 { NONE :: 0; FIRST :: 1; } read :: (choice:Choice=0) {} main :: () {}",
        "Choice :: enum u16 { FIRST :: 1; } read :: (choice:Choice=0) {} main :: () {}",
    ] {
        reject(source, "nominal type");
    }
}

#[test]
fn typed_run_constants_keep_nominal_flags_instead_of_integer_bits() {
    assert_eq!(
        run(
            "Flags::enum_flags u16{FIRST::1;} make::()->Flags{return .FIRST;} answer:Flags:#run make(); main::()->int{return cast(int)answer+41;}"
        ),
        42
    );
    reject(
        "Flags::enum_flags u16{FIRST::1;} make::()->u16{return 0;} answer:Flags:#run make(); main::()->int{return 0;}",
        "nominal type",
    );
}

#[test]
fn masks_bind_before_comparisons_and_logical_operators() {
    assert_eq!(
        run(r#"
Flags :: enum_flags u16 { FIRST :: 1; SECOND :: 2; THIRD :: 4; }
main :: () -> int {
    flags: Flags = .FIRST;
    if flags & .FIRST != 0 && flags & .SECOND == 0 return 42;
    return 1;
}
"#),
        42
    );
}

#[test]
fn weak_zero_equality_supports_both_orders_and_aliases() {
    assert_eq!(
        run(r#"
Flags :: enum_flags u64 { FIRST :: 1; HIGH :: 0x8000000000000000; }
ZERO :: 1 - 1;
main :: () -> int {
    empty: Flags;
    full: Flags = .HIGH;
    if empty == ZERO && ZERO == empty && full != 0 && 0 != full
       && !(full == 0) && !(0 == full) return 42;
    return 1;
}
"#),
        42
    );
}

#[test]
fn complements_keep_representation_width_and_contextual_members() {
    assert_eq!(
        run(r#"
Flags :: enum_flags u8 { FIRST :: 1; SECOND :: 2; THIRD :: 4; }
State :: struct { flags: Flags; }
main :: () -> int {
    flags: Flags = Flags.FIRST | .SECOND | .THIRD;
    flags &= ~.SECOND;
    state: State = .{flags=flags};
    state.flags &= ~.THIRD;
    inverted: Flags = ~.FIRST;
    explicit := ~Flags.FIRST;
    combined := ~.FIRST & Flags.THIRD;
    repeated: Flags = ~~.THIRD;
    if cast(u8) inverted != 254 || cast(u8) explicit != 254 return 1;
    if cast(u8) flags != 5 || cast(u8) state.flags != 1 return 2;
    if cast(u8) combined != 4 || cast(u8) repeated != 4 return 3;
    return 42;
}
"#),
        42
    );
}

#[test]
fn module_constants_and_run_share_flags_comparison_lowering() {
    assert_eq!(
        run(r#"
Flags :: enum_flags u16 { FIRST :: 1; SECOND :: 2; }
MASK :: Flags.FIRST | .SECOND;
HAS_FIRST :: MASK & .FIRST != 0;
ZERO :: MASK & .SECOND == 0;
probe :: () -> int { if HAS_FIRST && !ZERO return 42; return 1; }
ANSWER :: #run probe();
main :: () -> int { return ANSWER; }
"#),
        42
    );
}

#[test]
fn nonzero_weak_integers_and_strong_zero_remain_incompatible() {
    for comparison in [
        "flags == 1",
        "1 != flags",
        "flags == zero",
        "zero != flags",
        "flags > 0",
    ] {
        reject(
            &format!(
                r#"
Flags :: enum_flags u16 {{ FIRST :: 1; }}
main :: () -> int {{ flags: Flags = .FIRST; zero: u16 = 0; if {comparison} return 1; return 0; }}
"#
            ),
            "same nominal type",
        );
    }
}

#[test]
fn calls_bind_masks_and_boolean_comparisons_in_the_same_domains() {
    assert_eq!(
        run(r#"
Flags :: enum_flags u8 { FIRST :: 1; SECOND :: 2; }
read :: (flags: Flags) -> int { return cast(int)flags; }
check :: (condition: bool) -> int { if condition return 1; return 0; }
main :: () -> int {
    flags: Flags = .FIRST;
    if read(flags & .FIRST) != 1 || read(~.FIRST) != 254 return 1;
    if check(Flags.FIRST != 0) != 1 || check(flags & .SECOND == 0) != 1 return 2;
    return 42;
}
"#),
        42
    );
}

#[test]
fn local_flags_and_signed_complements_keep_their_declared_types() {
    assert_eq!(
        run(r#"
main :: () -> int {
    Flags :: enum_flags s8 { FIRST :: 1; SECOND :: 2; }
    flags: Flags = .FIRST;
    inverted: Flags = ~.FIRST;
    flags &= ~.FIRST;
    if flags == 0 && 0 == flags && cast(s8)inverted == -2 return 42;
    return 1;
}
"#),
        42
    );
}

#[test]
fn ordinary_and_unrelated_enums_do_not_gain_integer_conversions() {
    reject(
        r#"Mode :: enum u16 { FIRST; } main :: () -> int { mode: Mode; if mode == 0 return 1; return 0; }"#,
        "same nominal type",
    );
    reject(
        r#"Flags :: enum_flags u16 { FIRST; } Other :: enum_flags u16 { FIRST; } main :: () -> int { a:Flags=.FIRST; b:Other=.FIRST; if a==b return 1; return 0; }"#,
        "same nominal type",
    );
    reject(
        r#"Mode :: enum u16 { FIRST; } main :: () { mode: Mode = ~.FIRST; }"#,
        "enum_flags",
    );
}

#[test]
fn leading_dot_does_not_search_through_binary_arithmetic() {
    reject(
        r#"Flags :: enum_flags u16 { FIRST; } main :: () { flags: Flags = Flags.FIRST | (.FIRST + 1); }"#,
        "contextual enum",
    );
}

#[test]
fn missing_contextual_members_are_checked_in_lazy_branches() {
    reject(
        r#"Flags :: enum_flags u16 { FIRST; } main :: () -> int { flags: Flags = .FIRST; if true || flags & .MISSING != 0 return 1; return 0; }"#,
        "contextual enum",
    );
}

#[test]
fn overload_selection_describes_flags_operations_without_erasing_their_types() {
    assert_eq!(
        run(r#"
Flags :: enum_flags u8 { FIRST :: 1; SECOND :: 2; }
read :: (flags: Flags) -> int { return cast(int)flags; }
read :: (integer: int) -> int { return 99; }
check :: (condition: bool) -> int { if condition return 1; return 0; }
check :: (integer: int) -> int { return 99; }
main :: () -> int {
    flags: Flags = .FIRST;
    if read(flags & .FIRST) != 1 || read(~.FIRST) != 254 return 1;
    if check(Flags.FIRST != 0) != 1 || check(flags & .SECOND == 0) != 1 return 2;
    return 42;
}
"#),
        42
    );
}
