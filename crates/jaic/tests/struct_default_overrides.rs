//! Struct defaults retain expanded assignments and validate every target.
use jaic::sema::{Compiler, NativeFs, Options, ProgramSource};
use std::rc::Rc;

fn check(source: &str) -> Result<(), String> {
    let mut options = Options::host();
    let stdlib = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    options.import_paths.push(stdlib.clone());
    options.set_stdlib(&NativeFs, &stdlib);
    let mut c = Compiler::new(options, Rc::new(NativeFs));
    c.compile_sources(&[ProgramSource::String(source.into())])
        .map_err(|e| e.render(&c.src.sources))
}

#[test]
fn expanded_and_nested_defaults_share_the_constant_image() {
    check(include_str!(
        "../../../tests/compiler/regressions/struct-default-overrides.jai"
    ))
    .unwrap();
}

#[test]
fn generated_declarations_and_assignments_keep_their_order() {
    check(
        r#"
R :: struct(N:int) {
    #insert "values: [2]int; values[1] = 11;";
    #if N == 1 { #insert "values[1] = 17;"; }
    values[0] = N;
}
probe :: () -> bool { a:R(1); b:R(2); return a.values[1]==17 && b.values[1]==11 && b.values[0]==2; }
#assert probe();
main :: () {}
"#,
    )
    .unwrap();
}

#[test]
fn default_targets_follow_using_aliases_and_filters() {
    check(
        r#"
Inner :: struct { value:int; }
R :: struct { item:Inner; using item; value = 17; }
probe :: () -> bool { r:R; return r.item.value==17; }
#assert probe();
main :: () {}
"#,
    )
    .unwrap();
}

#[test]
fn invalid_default_assignments_are_not_silently_discarded() {
    for (body, message) in [
        ("x:int; missing = 17;", "not a member"),
        ("x:[2]int; x[2] = 17;", "out of bounds"),
        ("x:[2]int; x[-1] = 17;", "out of bounds"),
        ("x:[2]int; x[1] = \"bad\";", "type mismatch"),
        ("x:int; x += 17;", "struct default"),
        ("x,y:int; x,y = 1,2,3;", "2 names but 3 values"),
        ("x:[]int; x[0] = 17;", "fixed array"),
        ("x:*int; x[0] = 17;", "fixed array"),
    ] {
        let source = format!("R :: struct {{ {body} }} main :: () {{ r:R; }}");
        let error = check(&source).expect_err(&source);
        assert!(error.contains(message), "{source}\n{error}");
    }
}

#[test]
fn multiple_default_targets_consume_each_result_once() {
    check(
        r#"
pair :: () -> int, int { return 17,29; }
R :: struct { a,b:int; a,b = #run pair(); }
probe :: () -> bool { r:R; return r.a==17 && r.b==29; }
#assert probe();
main :: () {}
"#,
    )
    .unwrap();
}
