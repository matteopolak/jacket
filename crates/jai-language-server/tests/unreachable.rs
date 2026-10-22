//! Code that can never run is reported as an `unreachable` hint, which editors fade.
use jai_language_server::{DiagnosticCode, DiagnosticSeverity};
mod common;
use common::{at_offset, lean_session, session, uri};

/// The text of every faded range of `source`, without its final `;`, in order.
fn faded(source: &str) -> Vec<String> {
    faded_in(lean_session(), source)
}

/// `faded` in session `s`.
fn faded_in(mut s: jai_language_server::Session, source: &str) -> Vec<String> {
    s.open(uri(), 1, source.into()).unwrap();
    let lines: Vec<&str> = source.split('\n').collect();
    let offset = |line: u32, character: u32| -> usize {
        lines[..line as usize]
            .iter()
            .map(|l| l.len() + 1)
            .sum::<usize>()
            + character as usize
    };
    let mut found = Vec::new();
    for d in s.diagnostics(&uri()).unwrap() {
        if d.code != DiagnosticCode::Unreachable {
            continue;
        }
        assert_eq!(d.severity, DiagnosticSeverity::Hint);
        let start = offset(d.range.start.line, d.range.start.character);
        let end = offset(d.range.end.line, d.range.end.character);
        found.push(source[start..end].trim().trim_end_matches(';').to_string());
    }
    found
}

#[test]
fn statements_after_return_break_and_continue_fade_once_per_block() {
    let faded = faded(
        r"main :: () {
    n := 0;
    while n < 3 {
        n += 1;
        if n == 2 { continue; first_skipped := 1; second_skipped := 2; }
        if n == 3 { break; after_break := 1; }
    }
    return;
    after_return := 1;
    more := 2;
}
",
    );
    assert_eq!(
        faded,
        [
            "first_skipped := 1; second_skipped := 2",
            "after_break := 1",
            "after_return := 1;\n    more := 2",
        ]
    );
}

#[test]
fn a_return_in_one_branch_leaves_the_rest_reachable() {
    let source = r"f :: (flag: bool) -> int {
    if flag { return 1; }
    live := 2;
    return live;
}
main :: () { f(true); }
";
    assert_eq!(faded(source), Vec::<String>::new());
}

#[test]
fn code_after_an_if_whose_branches_both_exit_fades() {
    let source = r"f :: (flag: bool) -> int {
    if flag { return 1; } else { return 2; }
    dead := 3;
}
main :: () { f(true); }
";
    assert_eq!(faded(source), ["dead := 3"]);
}

#[test]
fn loops_that_nothing_breaks_out_of_end_the_block() {
    let source = r"spin :: () {
    while true { }
    after_spin := 1;
}
spin_int :: () {
    while 1 { }
    after_int := 1;
}
leaves :: () {
    while true { break; }
    reached := 1;
}
leaves_deep :: (flag: bool) {
    while true { if flag { break; } }
    still_reached := 1;
}
inner_break_only :: () {
    while true { for 0..3 { break; } }
    after_inner := 1;
}
main :: () { leaves(); leaves_deep(true); }
";
    assert_eq!(
        faded(source),
        ["after_spin := 1", "after_int := 1", "after_inner := 1"]
    );
}

#[test]
fn a_labelled_break_counts_for_the_loop_it_names() {
    let source = r"main :: (flag: bool) {
    while :outer true {
        while true {
            if flag { break outer; }
        }
        never := 1;
    }
    after_outer := 2;
}
";
    assert_eq!(faded(source), ["never := 1"]);
}

#[test]
fn defer_bodies_are_not_after_the_return() {
    let source = r"main :: () {
    count := 0;
    defer count += 1;
    return;
    dead := 1;
}
";
    assert_eq!(faded(source), ["dead := 1"]);
}

#[test]
fn static_if_fades_the_arm_that_was_not_taken_with_its_header() {
    let source = r"main :: () {
    #if false {
        off := 1;
    } else {
        on := 2;
    }
    #if true {
        shown := 3;
    } else {
        hidden := 4;
    }
    #if 1 == 2 { a := 5; } else #if true { b := 6; } else { c := 7; }
    #if true { d := 8; } else #if false { e := 9; } else { f := 10; }
    #if false { g := 11; } else #if false { h := 12; } else { i := 13; }
}
";
    assert_eq!(
        faded(source),
        [
            "#if false {\n        off := 1;\n    }",
            "else {\n        hidden := 4;\n    }",
            "#if 1 == 2 { a := 5; }",
            "else { c := 7; }",
            "else #if false { e := 9; } else { f := 10; }",
            "#if false { g := 11; }",
            "else #if false { h := 12; }",
        ]
    );
}

#[test]
fn an_inactive_os_branch_fades_with_its_header() {
    let other = if cfg!(windows) {
        "LINUX"
    } else {
        "WINDOWS"
    };
    let source = format!(
        "#if OS == .{other} {{\n    only_there :: 1;\n}}\nmain :: () {{\n    #if OS == .{other} {{\n        x := 1;\n    }}\n}}\n"
    );
    assert_eq!(
        faded_in(session(), &source),
        [
            format!("#if OS == .{other} {{\n    only_there :: 1;\n}}"),
            format!("#if OS == .{other} {{\n        x := 1;\n    }}"),
        ]
    );
}

#[test]
fn if_on_a_constant_fades_the_branch_it_rules_out() {
    let source = r"NEVER :: false;
ALWAYS :: true;
main :: (flag: bool) {
    if false { a := 1; }
    if NEVER { b := 2; } else { b_else := 2; }
    if ALWAYS { c := 3; } else { c_else := 3; }
    if flag { d := 4; } else { d_else := 4; }
    x := ifx ALWAYS then 1 else 2;
    while false { never_runs := 5; }
}
";
    assert_eq!(
        faded(source),
        ["a := 1", "b := 2", "c_else := 3", "2", "never_runs := 5"]
    );
}

#[test]
fn comparisons_a_type_decides_fade_the_branch_they_rule_out() {
    let source = r"main :: (u: u64, s: s64, byte: u8, small: s8, other: u64) {
    if u >= 0 { a := 1; } else { a_else := 1; }
    if s >= 0 { b := 2; } else { b_else := 2; }
    if byte > 255 { c := 3; }
    if small < -128 { d := 4; }
    if byte == 300 { e := 5; }
    if u < 0 { f := 6; }
    if 0 > u { g := 7; }
    if u >= other { h := 8; } else { h_else := 8; }
}
";
    assert_eq!(
        faded(source),
        [
            "a_else := 1",
            "c := 3",
            "d := 4",
            "e := 5",
            "f := 6",
            "g := 7"
        ]
    );
}

#[test]
fn intervals_follow_arithmetic_and_casts() {
    let source = r"main :: (x: u64, y: s64, z: u8) {
    if (x & 0xF) > 20 { masked := 1; }
    if (x & 0xF) > 14 { masked_live := 1; }
    if cast(u8) y < 300 { cast_live := 2; } else { cast_dead := 2; }
    if z + 1 > 255 { sum_wrapped := 3; }
    if (z >> 4) > 15 { shifted := 4; }
    if (z % 10) >= 10 { remainder := 5; }
    if !(z >= 0) { negated := 6; }
    if z >= 0 && x >= 0 { both := 7; } else { both_else := 7; }
    if z < 0 || x < 0 { either := 8; }
}
";
    assert_eq!(
        faded(source),
        [
            "masked := 1",
            "cast_dead := 2",
            "sum_wrapped := 3",
            "shifted := 4",
            "remainder := 5",
            "negated := 6",
            "both_else := 7",
            "either := 8",
        ]
    );
}

#[test]
fn a_branch_narrows_the_locals_its_condition_constrains() {
    let source = r"main :: (x: s64) {
    if x > 10 {
        if x < 5 { impossible := 1; }
        if x < 20 { possible := 2; }
    } else {
        if x > 10 { also_impossible := 3; }
    }
    while x < 3 {
        if x >= 3 { in_loop := 4; }
        break;
    }
}
";
    assert_eq!(
        faded(source),
        ["impossible := 1", "also_impossible := 3", "in_loop := 4"]
    );
}

#[test]
fn narrowing_stops_where_the_local_may_change() {
    let source = r"bump :: (p: *s64) { p.* += 1; }
main :: (x: s64, y: s64) {
    if x > 10 {
        x = 0;
        if x < 5 { reassigned := 1; }
    }
    if y > 10 {
        bump(*y);
        if y < 5 { by_pointer := 2; }
    }
}
";
    assert_eq!(faded(source), Vec::<String>::new());
}

#[test]
fn a_switch_fades_cases_its_value_cannot_select_and_ends_when_all_exit() {
    let source = r"Mode :: enum { A; B; }
pick :: (m: Mode) -> int {
    if #complete m == {
        case .A; return 1;
        case .B; return 2;
    }
    dead_after := 3;
}
narrow :: (x: s64) {
    if x > 10 {
        if x == {
            case 3; excluded := 1;
            case 20; selected := 2;
        }
    }
}
main :: () { pick(.A); narrow(1); }
";
    assert_eq!(faded(source), ["dead_after := 3", "excluded := 1"]);
}

#[test]
fn a_through_case_flows_into_the_next() {
    let source = r"main :: (x: s64) {
    if x > 10 {
        if x == {
            case 20; first := 1; #through;
            case 3; reached_by_falling := 2;
        }
    }
}
";
    assert_eq!(faded(source), Vec::<String>::new());
}

#[test]
fn a_polymorphic_body_fades_only_what_every_instance_rules_out() {
    let source = r"check :: (v: $T) {
    if v >= 0 { positive := 1; } else { negative := 2; }
}
only_unsigned :: (v: $T) {
    if v >= 0 { positive_only := 1; } else { never_negative := 2; }
}
main :: () {
    check(cast(u8) 1);
    check(cast(s32) 1);
    only_unsigned(cast(u8) 1);
    only_unsigned(cast(u16) 1);
}
";
    assert_eq!(faded(source), ["never_negative := 2"]);
}

#[test]
fn faded_branches_keep_hover_and_definition() {
    let source = r"main :: (u: u64) {
    outer := 5;
    if false {
        inside_false := outer + 1;
        broken_but_dead := missing_name;
    }
    if u >= 0 { live := 1; } else {
        inside_else := outer * 2;
    }
}
";
    let mut s = lean_session();
    s.open(uri(), 1, source.into()).unwrap();
    for (needle, expected) in [
        ("inside_false", "inside_false: s64"),
        ("inside_else", "inside_else: s64"),
    ] {
        let hover = s
            .hover(&uri(), at_offset(source, needle, 2))
            .unwrap()
            .expect(needle);
        assert!(hover.contents.value.contains(expected), "{hover:?}");
    }
    for needle in ["outer + 1", "outer * 2"] {
        let defs = s.definition(&uri(), at_offset(source, needle, 1)).unwrap();
        assert_eq!(defs.len(), 1, "{needle}");
        assert_eq!(defs[0].range.start.line, 1);
    }
    let errors: Vec<_> = s
        .diagnostics(&uri())
        .unwrap()
        .into_iter()
        .filter(|d| d.severity == DiagnosticSeverity::Error)
        .collect();
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn an_early_exit_narrows_the_rest_of_the_block() {
    let source = r"main :: (x: s64, y: s64, z: s64) {
    if x < 0 { return; }
    if x < 0 { after_return := 1; }
    while true {
        if y > 5 { break; }
        if y > 5 { after_break := 2; }
        if y < 3 { y_live := 3; }
    }
    if z < 0 { return; }
    z = -1;
    if z < 0 { reassigned_live := 4; }
}
";
    assert_eq!(faded(source), ["after_return := 1", "after_break := 2"]);
}

#[test]
fn an_if_else_where_one_side_exits_narrows_too() {
    let source = r"main :: (x: s64) {
    if x > 3 { } else { return; }
    if x < 2 { impossible := 1; }
}
";
    assert_eq!(faded(source), ["impossible := 1"]);
}

#[test]
fn a_for_over_an_empty_range_fades_its_body() {
    let source = r"main :: (n: s64) {
    for 1..0 { empty := 1; }
    for 0..1 { once := 2; }
    for 5..n { unknown := 3; }
}
";
    assert_eq!(faded(source), ["empty := 1"]);
}

#[test]
fn the_editor_does_not_warn_of_a_missing_return_the_walk_rules_out() {
    let source = r"f :: (u: u8) -> int {
    if u >= 0 { return 1; }
}
main :: () { f(1); }
";
    let mut s = lean_session();
    s.open(uri(), 1, source.into()).unwrap();
    let warned = s
        .diagnostics(&uri())
        .unwrap()
        .iter()
        .any(|d| d.message.contains("not all control paths"));
    assert!(!warned);
}

#[test]
fn an_inactive_static_if_branch_still_has_syntax_tokens() {
    let source = "main :: () {\n    #if false {\n        value := 1 + other;\n    }\n}\n";
    let mut s = lean_session();
    s.open(uri(), 1, source.into()).unwrap();
    let tokens = s.semantic_tokens(&uri()).unwrap();
    assert!(tokens.iter().any(|t| t.position.line == 2), "{tokens:?}");
}

const EXIT: &str = "exit :: (status: s32) #foreign;\n";

#[test]
fn code_after_a_call_to_exit_fades() {
    let source = format!(
        "{EXIT}main :: () {{
    exit(1);
    dead := 1;
}}
"
    );
    assert_eq!(faded(&source), ["dead := 1"]);
}

#[test]
fn a_procedure_that_always_exits_ends_its_callers_block() {
    let source = format!(
        "{EXIT}fatal :: () {{ exit(2); }}
wrapper :: (flag: bool) {{
    if flag {{ fatal(); }} else {{ exit(3); }}
    after_both := 1;
}}
main :: () {{
    fatal();
    after_fatal := 2;
}}
other :: () {{
    wrapper(true);
    after_wrapper := 3;
}}
"
    );
    assert_eq!(
        faded(&source),
        ["after_both := 1", "after_fatal := 2", "after_wrapper := 3"]
    );
}

#[test]
fn a_procedure_with_a_path_that_returns_is_not_a_dead_end() {
    let source = format!(
        "{EXIT}sometimes :: (flag: bool) {{
    if flag exit(1);
}}
recursive :: () {{ recursive(); }}
main :: () {{
    sometimes(false);
    live := 1;
    recursive();
    also_live := 2;
}}
"
    );
    assert_eq!(faded(&source), Vec::<String>::new());
}

#[test]
fn a_procedure_that_returns_on_some_path_does_not_diverge_even_after_exit_calls() {
    let source = format!(
        "{EXIT}maybe :: (flag: bool) {{
    if flag {{ return; }}
    exit(1);
}}
main :: () {{
    maybe(true);
    live := 1;
}}
"
    );
    assert_eq!(faded(&source), Vec::<String>::new());
}

#[test]
fn a_loop_without_a_break_never_ends_but_one_that_breaks_does() {
    let source = format!(
        "{EXIT}spin :: () {{ while true {{ }} }}
leaves :: () {{ while true {{ break; }} }}
main :: () {{
    leaves();
    live := 1;
    spin();
    dead := 2;
}}
"
    );
    assert_eq!(faded(&source), ["dead := 2"]);
}


#[test]
fn os_tests_without_a_prelude_do_not_crash() {
    // Builtins are declared in no file; looking one up as a file must not crash.
    let source = "#if OS == .WINDOWS {\n    only_there :: 1;\n}\nmain :: () {\n    #if OS == .WINDOWS {\n        x := 1;\n    }\n}\n";
    assert_eq!(faded(source), Vec::<String>::new());
}

#[test]
fn empty_switch_cases_leave_following_code_reachable() {
    let source = r"Mode :: enum { A; B; }
other_stuff_here :: () {}
default_only :: (mode: Mode) {
    if mode == { case; }
    other_stuff_here();
    after_default := 1;
}
exhaustive :: (mode: Mode) {
    if #complete mode == {
        case .A;
        case .B; return;
    }
    after_empty_case := 2;
}
main :: () {
    default_only(.A);
    exhaustive(.A);
    after_calls := 3;
}
";
    assert_eq!(faded(source), Vec::<String>::new());
}

#[test]
fn an_empty_through_case_can_enter_an_otherwise_excluded_case() {
    let source = r"main :: (x: s64) {
    if x > 10 {
        if x == {
            case 20; #through;
            case 3; reached_by_falling := 1;
            case;
        }
        after_switch := 2;
    }
}
";
    assert_eq!(faded(source), Vec::<String>::new());
}

#[test]
fn excluded_empty_cases_do_not_make_terminating_switches_fall_through() {
    let source = r"main :: (x: s64) {
    if x > 10 {
        if x == {
            case 3;
            case; return;
        }
        after_terminating_switch := 1;
    }
}
";
    assert_eq!(faded(source), ["after_terminating_switch := 1"]);
}

#[test]
fn an_empty_exhaustive_case_does_not_suppress_a_missing_return_warning() {
    let source = r"Mode :: enum { A; B; }
pick :: (mode: Mode) -> int {
    if #complete mode == {
        case .A;
        case .B; return 2;
    }
}
main :: () { pick(.A); }
";
    let mut session = lean_session();
    session.open(uri(), 1, source.into()).unwrap();
    assert!(
        session
            .diagnostics(&uri())
            .unwrap()
            .iter()
            .any(|d| d.message.contains("not all control paths return a value"))
    );
}
