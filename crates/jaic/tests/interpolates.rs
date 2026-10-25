//! `@Interpolates("mode")`: holes in a literal become positional arguments, checked in the
//! caller's scope with spans inside the string.
use jaic::sema::{Compiler, NativeFs, Options, ProgramSource};
use std::{path::PathBuf, rc::Rc};

const PRELUDE: &str = r#"
#import "Basic";
Info :: struct { text: string; count: s64; first: s64; second: s64; }

describe :: (s: string, args: ..Any) -> Info {
    info: Info;
    info.text = s;
    info.count = args.count;
    if args.count > 0 info.first = <<cast(*s64) args[0].value_pointer;
    if args.count > 1 info.second = <<cast(*s64) args[1].value_pointer;
    return info;
} @Interpolates("plain")

query :: ($sql: string, args: ..Any) -> Info #expand {
    return describe(sql, ..args);
} @Interpolates("sql")

same :: (a: string, b: string) -> bool {
    if a.count != b.count return false;
    for 0..a.count-1 if a[it] != b[it] return false;
    return true;
}
"#;

fn run(body: &str) -> Result<(), String> {
    let source = format!("{PRELUDE}\n{body}");
    let stdlib = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let mut options = Options::host();
    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
    let mut compiler = Compiler::new(options, Rc::new(NativeFs));
    compiler
        .compile_sources(&[ProgramSource::String(source)])
        .map_err(|e| e.render(&compiler.src.sources))
}

fn ok(body: &str) {
    if let Err(e) = run(body) {
        panic!("{e}");
    }
}

fn error(body: &str) -> String {
    run(body).expect_err("the program must not compile")
}

#[test]
fn holes_become_positional_arguments() {
    ok(r#"
user_id := 7;
#run {
    info := describe("id = %user_id and %(user_id + 1)");
    assert(same(info.text, "id = % and %"));
    assert(info.count == 2);
    assert(info.first == 7);
    assert(info.second == 8);
}
"#);
}

#[test]
fn field_and_index_chains_are_holes() {
    ok(r#"
Point :: struct { x: s64; y: s64; }
main :: () {
    p: Point;
    p.x = 3; p.y = 4;
    values := s64.[10, 20, 30];
    i := 1;
    info := describe("%p.y then %values[i + 1]. done");
    assert(same(info.text, "% then %. done"));
    assert(info.first == 4);
    assert(info.second == 30);
}
#run main();
"#);
}

#[test]
fn percent_signs_that_start_no_hole_stay() {
    ok(r#"
#run {
    info := describe("50% of 10 % 3 and %% and %1");
    assert(info.count == 0);
    assert(same(info.text, "50% of 10 % 3 and %% and %1"));
}
"#);
}

#[test]
fn a_string_without_holes_keeps_positional_arguments() {
    ok(r#"
#run {
    info := describe("a % b %", 1, 2);
    assert(info.count == 2 && info.first == 1 && info.second == 2);
}
"#);
}

#[test]
fn a_non_literal_string_is_left_alone() {
    ok(r#"
#run {
    s := "has %x";
    info := describe(s);
    assert(info.count == 0);
    assert(same(info.text, "has %x"));
}
"#);
}

#[test]
fn holes_and_positional_arguments_do_not_mix() {
    let e = error(
        r#"
x := 1;
#run describe("a %x", 2);
"#,
    );
    assert!(
        e.contains("a mix of inline holes and positional arguments"),
        "{e}"
    );
}

#[test]
fn an_empty_hole_is_an_error() {
    let e = error(
        r#"
#run describe("a %( ) b");
"#,
    );
    assert!(e.contains("an inline hole is empty"), "{e}");
}

#[test]
fn a_hole_that_is_not_utf8_is_an_error() {
    let e = error(
        r#"
#run describe("a %(\xff) b");
"#,
    );
    assert!(e.contains("an inline hole must be valid UTF-8 text"), "{e}");
}

#[test]
fn sql_mode_skips_quotes_comments_and_dollar_quotes() {
    ok(r#"
a := 1; b := 2;
#run {
    info := query("select '%a', E'\\'%a', \"%a\", $t$ %a $t$ -- %a
        /* %a /* %a */ %a */ where x % 3 = %b and y = %a");
    assert(info.count == 2);
    assert(info.first == 2);
    assert(info.second == 1);
}
"#);
}

#[test]
fn plain_mode_skips_nothing() {
    ok(r#"
a := 5;
#run {
    info := describe("'%a'");
    assert(info.count == 1 && info.first == 5);
}
"#);
}

#[test]
fn here_strings_are_scanned_too() {
    ok(r"
a := 5; b := 6;
#run {
    info := query(#string SQL
select %a, '%b'
  from t where c = %b
SQL
    );
    assert(info.count == 2 && info.first == 5 && info.second == 6);
}
");
}

#[test]
fn an_undefined_name_points_into_the_string() {
    // `missing` starts at column 21 of the `#run` line below.
    let e = error("#run describe(\"id = %missing x\");\n");
    assert!(e.contains("missing"), "{e}");
    let line = e.lines().find(|l| l.contains('^')).unwrap_or("");
    assert!(e.contains("describe(\"id = %missing x\")"), "{e}");
    let caret = line.find('^').unwrap();
    let source_line = e.lines().find(|l| l.contains("describe(")).unwrap();
    let want = source_line.find("missing").unwrap();
    assert_eq!(caret, want, "{e}");
}

#[test]
fn a_type_error_points_at_the_hole() {
    let e = error("#run describe(\"pad %(1 + \\\"x\\\") end\");\n");
    let source_line = e.lines().find(|l| l.contains("describe(")).unwrap();
    let carets = e.lines().find(|l| l.contains('^')).unwrap();
    let from = carets.find('^').unwrap();
    let to = carets.rfind('^').unwrap();
    let hole = source_line.find("1 + ").unwrap();
    assert!(
        from >= hole && to < source_line.find(" end").unwrap(),
        "{e}"
    );
}

#[test]
fn escapes_map_to_the_right_columns() {
    // The tab and the `\n` make the value offsets differ from the source's.
    let e = error("#run describe(\"a\\tb\\n%nope\");\n");
    let source_line = e.lines().find(|l| l.contains("describe(")).unwrap();
    let line = e.lines().find(|l| l.contains('^')).unwrap();
    assert_eq!(
        line.find('^').unwrap(),
        source_line.find("nope").unwrap(),
        "{e}"
    );
}

#[test]
fn macros_with_constant_string_parameters_take_holes() {
    ok(r#"
id := 9;
#run {
    info := query("select %id");
    assert(info.first == 9 && same(info.text, "select %"));
}
"#);
}

#[test]
fn the_language_fixture_checks_out() {
    let source = include_str!("../../../tests/compiler/semantics/interpolates.jai");
    let stdlib = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let mut options = Options::host();
    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
    let mut compiler = Compiler::new(options, Rc::new(NativeFs));
    if let Err(e) = compiler.compile_sources(&[ProgramSource::String(source.into())]) {
        panic!("{}", e.render(&compiler.src.sources));
    }
}

#[test]
fn an_overload_set_with_one_noted_candidate_is_rewritten() {
    ok(r#"
pick :: (n: s64) -> s64 { return n; }
pick :: (s: string, args: ..Any) -> Info { return describe(s, ..args); } @Interpolates("plain")
a := 3;
#run {
    info := pick("x %a");
    assert(info.count == 1 && info.first == 3);
    assert(pick(5) == 5);
}
"#);
}

#[test]
fn a_named_string_argument_is_rewritten() {
    ok(r#"
a := 3;
#run {
    info := describe(s = "x %a");
    assert(info.count == 1 && info.first == 3);
}
"#);
}
