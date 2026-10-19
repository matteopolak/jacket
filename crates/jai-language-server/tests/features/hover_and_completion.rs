//! Completion, signature help and hovers over polymorphs, baked procedures and other targets.
use super::{at_nth, file_uri, hover, session, uri};
use jai_language_server::{Position, Session};

/// Complete at the end of `typed` (the text before the cursor) inside `main`, with a
/// `table` array and a `lo`/`hi` pair in scope.
fn complete_in_main(typed: &str) -> Vec<String> {
    let text = format!(
        "#import \"Basic\";\nmain :: () {{\n    table: [8] int;\n    lo := 0;\n    hi := 3;\n    {typed}"
    );
    let rest = " {\n        print(\"%\", i);\n    }\n}\n";
    let cursor = text.len();
    let text = format!("{text}{rest}");
    let mut s = session();
    s.open(uri(), 1, text.clone()).unwrap();
    let end = Position {
        line: text[..cursor].matches('\n').count() as u32,
        character: text[..cursor].rsplit('\n').next().unwrap().len() as u32,
    };
    s.completion(&uri(), end)
        .unwrap()
        .items
        .into_iter()
        .map(|i| i.label)
        .collect()
}

#[test]
fn completion_works_inside_range_expressions() {
    for typed in [
        "for i: 0..ta",
        "for i: 0 .. ta",
        "for 0..ta",
        "for i: lo..ta",
        "x := lo..ta",
    ] {
        let labels = complete_in_main(typed);
        assert!(labels.iter().any(|l| l == "table"), "{typed}: {labels:?}");
    }
    for typed in [
        "for i: 0..table.co",
        "for i: 0 .. table.co",
        "for 0..table.co",
        "for i: lo..table.co",
    ] {
        let labels = complete_in_main(typed);
        assert!(labels.iter().any(|l| l == "count"), "{typed}: {labels:?}");
        assert!(!labels.iter().any(|l| l == "table"), "{typed}: {labels:?}");
    }
    // The end of a range also offers the names after it.
    let labels = complete_in_main("for i: lo..");
    assert!(labels.iter().any(|l| l == "hi"), "{labels:?}");
}

#[test]
fn signature_help_for_procedure_typed_variables_and_members() {
    let program = "\
My_Func_Type :: #type (a: int, b: string) -> int;
Holder :: struct { callback: (x: float, y: float) -> float; }
twice :: (n: int, label: string) -> int { return n; }
run :: () {
    f: My_Func_Type = twice;
    s: Holder;
    CALL
}
";
    for (call, label, params, active) in [
        ("f(1, ", "f: (s64, string) -> s64", vec!["s64", "string"], 1),
        (
            "s.callback(1.0, ",
            "callback: (float32, float32) -> float32",
            vec!["float32", "float32"],
            1,
        ),
        ("f(", "f: (s64, string) -> s64", vec!["s64", "string"], 0),
    ] {
        let text = program.replace("CALL", call);
        let mut s = session();
        s.open(uri(), 1, text.clone()).unwrap();
        let help = s
            .signature_help(&uri(), at_nth(&text, call, 0, call.len()))
            .unwrap()
            .unwrap_or_else(|| panic!("no signature help for {call}"));
        assert_eq!(help.signatures.len(), 1);
        assert_eq!(help.signatures[0].label, label);
        assert_eq!(help.signatures[0].parameters, params);
        assert_eq!(help.active_parameter, active, "{call}");
    }
}

#[test]
fn hover_of_a_baked_procedure_is_its_remaining_header() {
    let text = "scale_by :: (value: int, factor: int) -> int { return value * factor; }\ndouble_it :: #bake_arguments scale_by(factor = 2);\nmain :: () { x := double_it(4); }\n";
    let mut s = session();
    s.open(uri(), 1, text.into()).unwrap();
    let decl = hover(&s, at_nth(text, "double_it ::", 0, 2));
    let used = hover(&s, at_nth(text, "double_it(4", 0, 2));
    assert_eq!(decl, "double_it :: (value: int) -> int");
    assert_eq!(used, "double_it :: (value: int) -> int");
    let help = s
        .signature_help(&uri(), at_nth(text, "double_it(4", 0, 10))
        .unwrap()
        .expect("signature help");
    assert_eq!(help.signatures[0].label, "double_it :: (value: int) -> int");
}

#[test]
fn names_from_a_using_parameter_hover_and_go_to_it() {
    let text = "#import \"Basic\";\nVec2 :: struct { x: float; y: float; }\nCreature :: struct { name: string; health: int; position: Vec2; }\nreport :: (using creature: Creature) {\n    print(\"% % %\\n\", name, health, position.x);\n}\nmain :: () { report(.{}); }\n";
    let mut s = session();
    s.open(uri(), 1, text.into()).unwrap();
    let name = s
        .hover(&uri(), at_nth(text, "name, health", 0, 1))
        .unwrap()
        .map(|h| h.contents.value);
    let pos = s
        .hover(&uri(), at_nth(text, "position.x", 0, 1))
        .unwrap()
        .map(|h| h.contents.value);
    let def = s
        .definition(&uri(), at_nth(text, "name, health", 0, 1))
        .unwrap();
    assert_eq!(name.as_deref(), Some("name: string"));
    assert_eq!(pos.as_deref(), Some("position: Vec2"));
    assert_eq!(def.len(), 1, "{def:?}");
    assert_eq!(def[0].range.start, at_nth(text, "name: string", 0, 0));
}

/// Names that only the type checker can place: each hovers with its type and goes to its
/// declaration (found by `examples/ide_coverage.rs`).
#[test]
fn a_polymorph_nothing_calls_hovers_with_its_parameters_as_types() {
    let text = r"Value :: struct { n: int; }
convert :: (v: Value, $Target: Type) -> ok: bool, result: Target {
    target: Target;
    count := v.n + 1;
    return count > 0, target;
}
first :: (items: [$N] $T) -> T {
    out := items[0];
    return out;
}
find :: (key: string) -> found := false, index: int = -1 {
    return true, 0;
}
main :: () {
    for < it_index: 0..3 {}
}
";
    let mut s = session();
    s.open(uri(), 1, text.into()).unwrap();
    let hover = |s: &mut Session, needle: &str, offset: usize| {
        s.hover(&uri(), at_nth(text, needle, 0, offset))
            .unwrap()
            .map(|h| h.contents.value)
            .unwrap_or_default()
    };
    let mut wrong = Vec::new();
    for (needle, offset, want) in [
        ("target: Target", 1, "target: Target"),
        ("count :=", 1, "count: s64"),
        ("n + 1", 0, "n: s64"),
        ("out :=", 1, "out: T"),
        ("found :=", 1, "found: bool"),
        ("index: int", 1, "index: s64"),
        ("result: Target", 1, "result: Target"),
    ] {
        let got = hover(&mut s, needle, offset);
        if !got.contains(want) {
            wrong.push(format!("{needle}: {got:?}"));
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
    assert_eq!(
        s.definition(&uri(), at_nth(text, "Target;", 0, 1))
            .unwrap()
            .first()
            .map(|d| d.range.start),
        Some(at_nth(text, "Target: Type", 0, 0))
    );
    // The sketch's errors (`items[0]` of a stand-in `T` is fine, but nothing else is checked
    // against real types) are not the program's.
    let diagnostics = s.diagnostics(&uri()).unwrap();
    assert!(diagnostics.is_empty(), "{diagnostics:#?}");
}

#[test]
fn a_sketch_carries_on_past_members_of_its_stand_ins() {
    let text = r"Matrix :: struct ($T: Type, $M: int, $N: int) { rows: [M] [N] T; }
Counter :: struct { hits: int; }
describe :: (x: $T, counter: *Counter) -> int {
    label := x.label;
    total := x.size * 2;
    for item: x.items {
        counter.hits += 1;
    }
    if x.ready {
        counter.hits += 2;
    }
    #if OS == .WINDOWS {
        windows_hits := counter.hits;
    } else {
        other_hits := counter.hits;
    }
    #assert size_of(T) == 4;
    missing := x.size + nothing_here;
    after := counter.hits;
    return total;
}
trace :: (m: $M/Matrix) -> M.T {
    first := m.rows[0][0];
    return first;
}
bump :: (counter: *Counter) #expand {
    bumped := counter.hits + 1;
}
main :: () {}
";
    let mut s = session();
    s.open(uri(), 1, text.into()).unwrap();
    let hover = |s: &mut Session, needle: &str, offset: usize| {
        s.hover(&uri(), at_nth(text, needle, 0, offset))
            .unwrap()
            .map(|h| h.contents.value)
            .unwrap_or_default()
    };
    let mut wrong = Vec::new();
    for (needle, offset, want) in [
        ("label :=", 1, "label: T.label"),
        ("total :=", 1, "total: s64"),
        ("item: x", 1, "item:"),
        ("hits += 1", 0, "hits: s64"),
        ("hits += 2", 0, "hits: s64"),
        ("windows_hits", 1, "windows_hits: s64"),
        ("other_hits", 1, "other_hits: s64"),
        ("missing :=", 1, "missing:"),
        ("after :=", 1, "after: s64"),
        ("first :=", 1, "first:"),
        ("bumped :=", 1, "bumped: s64"),
    ] {
        let got = hover(&mut s, needle, offset);
        if !got.contains(want) {
            wrong.push(format!("{needle}: {got:?}"));
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
    assert_eq!(
        s.definition(&uri(), at_nth(text, "hits += 2", 0, 0))
            .unwrap()
            .first()
            .map(|d| d.range.start),
        Some(at_nth(text, "hits: int", 0, 0))
    );
    let diagnostics = s.diagnostics(&uri()).unwrap();
    assert!(diagnostics.is_empty(), "{diagnostics:#?}");
}

#[test]
fn baked_names_and_bare_polymorphic_struct_parameters_hover() {
    let text = r"Vector :: struct (T: Type, N: int) { x: T; #if N > 1 { y: T; } }
Vec2 :: #bake_arguments Vector(N = 2);
Vec2f :: Vec2(float32);
scale :: (v: $T, by: int) -> T { return v * by; }
double :: #bake_arguments scale(by = 2);
twice :: #bake_constants scale(T = int);
flatten :: (v: Vec2) -> Vec2f { return .{xx v.x, xx v.y}; }
main :: () {}
";
    let mut s = session();
    s.open(uri(), 1, text.into()).unwrap();
    let hover = |s: &mut Session, needle: &str, offset: usize| {
        s.hover(&uri(), at_nth(text, needle, 0, offset))
            .unwrap()
            .map(|h| h.contents.value)
            .unwrap_or_default()
    };
    let mut wrong = Vec::new();
    for (needle, offset, want) in [
        ("N = 2", 0, "N: s64"),
        ("by = 2", 0, "by: s64"),
        ("T = int", 0, "T: Type"),
        ("v.x", 0, "v:"),
        ("x, xx", 0, "x:"),
        ("y}", 0, "y:"),
    ] {
        let got = hover(&mut s, needle, offset);
        if !got.contains(want) {
            wrong.push(format!("{needle}: {got:?}"));
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
    for (needle, decl) in [
        ("N = 2", "N: int"),
        ("by = 2", "by: int"),
        ("T = int", "$T"),
    ] {
        assert_eq!(
            s.definition(&uri(), at_nth(text, needle, 0, 0))
                .unwrap()
                .first()
                .map(|d| d.range.start),
            Some(at_nth(text, decl, 0, usize::from(decl == "$T"))),
            "{needle}"
        );
    }
    let diagnostics = s.diagnostics(&uri()).unwrap();
    assert!(diagnostics.is_empty(), "{diagnostics:#?}");
}

#[test]
fn foreign_parameters_and_libraries_hover() {
    let text = r#"libc :: #system_library "libc";
length :: (text: *u8) -> u64 #foreign libc "strlen";
main :: () {}
"#;
    let mut s = session();
    s.open(uri(), 1, text.into()).unwrap();
    let hover = |s: &mut Session, needle: &str| {
        s.hover(&uri(), at_nth(text, needle, 0, 0))
            .unwrap()
            .map(|h| h.contents.value)
            .unwrap_or_default()
    };
    assert!(hover(&mut s, "text:").contains("text: *u8"));
    assert!(hover(&mut s, "libc \"strlen").contains("libc"));
    assert_eq!(
        s.definition(&uri(), at_nth(text, "libc \"strlen", 0, 0))
            .unwrap()
            .first()
            .map(|d| d.range.start),
        Some(at_nth(text, "libc ::", 0, 0))
    );
}

#[test]
fn code_another_target_compiles_hovers_and_goes_to_definition() {
    let main = r#"#if OS == .WASM {
    #load "web.jai";
    scale :: (by: int) -> int { return by * web_factor(); }
}
main :: () {}
"#;
    let web_text = "web_factor :: () -> int { return 2; }\n";
    let web = file_uri("/lsp-test/web.jai");
    let mut s = session();
    s.open(uri(), 1, main.into()).unwrap();
    s.open(web.clone(), 1, web_text.into()).unwrap();
    let shown = s
        .hover(&uri(), at_nth(main, "by *", 0, 0))
        .unwrap()
        .map(|h| h.contents.value)
        .unwrap_or_default();
    assert!(shown.contains("by: s64"), "{shown}");
    let found = s
        .definition(&uri(), at_nth(main, "web_factor", 0, 0))
        .unwrap();
    assert_eq!(found.first().map(|d| d.uri.as_str()), Some(web.as_str()));
    let shown = s
        .hover(&web, at_nth(web_text, "int", 0, 0))
        .unwrap()
        .map(|h| h.contents.value)
        .unwrap_or_default();
    assert!(shown.contains("int"), "{shown}");
}

#[test]
fn a_failed_load_leaves_bare_polymorphic_struct_parameters_polymorphic() {
    let text = r#"#load "missing.jai";
Pair :: struct (T: Type) { a: T; b: T; }
swap :: (p: *Pair) { p.a, p.b = p.b, p.a; }
main :: () {}
"#;
    let mut s = session();
    s.open(uri(), 1, text.into()).unwrap();
    let diagnostics = s.diagnostics(&uri()).unwrap();
    assert!(
        diagnostics.iter().all(|d| d.range.start.line == 0),
        "{diagnostics:#?}"
    );
}

#[test]
fn names_outside_plain_expressions_hover_and_go_to_their_declarations() {
    let text = r#"#import "Basic";
Pair :: struct { first: int; second: int; }
Ring :: struct ($N: int) { items: [N] int; }
Countdown :: struct { from: int; }
for_expansion :: (c: Countdown, body: Code, flags: For_Flags) #expand {
    for i: 0..c.from - 1 { `it := c.from - i; `it_index := i; #insert body; }
}
half :: (x: $T) -> T { y: T = x / 2; return y; }
scale :: (x: int, by := 2) -> int { return x * by; }
Callback :: #type (count: int) -> int;
two :: () -> bool, int { return true, 2; }
main :: () {
    p := Pair.{first = 1, second = 2};
    r: Ring(3);
    print("% % % %\n", half(5), scale(1, by = 3), p, r.items.count);
    for value: Countdown.{3} print("%", value);
    for outer: 0..2 { for 0..2 if it == 1 continue outer; }
    ok: bool;
    ok=, n := two();
    log("%", n,, log_level = .VERBOSE);
    byte := cast,trunc(u8) 300;
    print("% %\n", ok, byte);
}
"#;
    let mut s = session();
    s.open(uri(), 1, text.into()).unwrap();
    let hover = |s: &mut Session, needle: &str, nth: usize, offset: usize| {
        s.hover(&uri(), at_nth(text, needle, nth, offset))
            .unwrap()
            .map(|h| h.contents.value)
            .unwrap_or_default()
    };
    let definition = |s: &mut Session, needle: &str, nth: usize, offset: usize| {
        s.definition(&uri(), at_nth(text, needle, nth, offset))
            .unwrap()
            .first()
            .map(|d| d.range.start)
    };
    let mut wrong = Vec::new();
    for (needle, nth, offset, want) in [
        ("first = 1", 0, 1, "first: s64"),
        ("by = 3", 0, 1, "by: s64"),
        ("items: [N]", 0, 1, "items: [N] int"),
        ("value: Countdown", 0, 1, "value: s64"),
        ("outer;", 0, 1, "outer: s64"),
        ("ok=,", 0, 0, "ok: bool"),
        ("count: int) ->", 0, 1, "count: s64"),
        ("log_level", 0, 1, "log_level: Log_Level"),
    ] {
        let got = hover(&mut s, needle, nth, offset);
        if !got.starts_with(want) {
            wrong.push(format!("{needle}: {got:?}"));
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
    assert!(hover(&mut s, "trunc", 0, 1).contains("Cast flag"));
    assert!(hover(&mut s, "#expand", 0, 1).contains("#expand"));
    assert_eq!(
        definition(&mut s, "first = 1", 0, 1),
        Some(at_nth(text, "first: int", 0, 0))
    );
    assert_eq!(
        definition(&mut s, "by = 3", 0, 1),
        Some(at_nth(text, "by := 2", 0, 0))
    );
    assert_eq!(
        definition(&mut s, "T = x", 0, 0),
        Some(at_nth(text, "T) -> T", 0, 0))
    );
    assert_eq!(
        definition(&mut s, "outer;", 0, 1),
        Some(at_nth(text, "outer: 0", 0, 0))
    );
}

#[test]
fn hover_of_a_pointer_constant_leaves_out_its_address() {
    let text = "#import \"Basic\";\nraw :: () -> *void { return alloc(32); }\nnothing :: () -> *void { return null; }\nBLOCK :: #run raw();\nNONE :: #run nothing();\nmain :: () { b := BLOCK; n := NONE; }\n";
    let mut s = session();
    s.open(uri(), 1, text.into()).unwrap();
    assert_eq!(hover(&s, at_nth(text, "BLOCK ::", 0, 1)), "BLOCK :: *void");
    assert_eq!(
        hover(&s, at_nth(text, "NONE ::", 0, 1)),
        "NONE :: *void = null"
    );
}
