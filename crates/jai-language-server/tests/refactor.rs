//! Refactoring code actions: extract, inline, fill in cases and fields, `ifx` rewrites.
use jai_language_server::{
    CodeAction, DiagnosticSeverity, DocumentUri, Environment, Limits, Position, Range, Session,
};
use std::path::PathBuf;

fn environment() -> Environment {
    let stdlib = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    Environment {
        fs: std::rc::Rc::new(jaic::sema::NativeFs),
        options: Box::new(move |_| {
            let mut options = jaic::sema::Options::host();
            options.import_paths = vec![stdlib.clone()];
            options.preload = Some(stdlib.join(jaic::PRELOAD_ENTRY));
            options
        }),
    }
}

fn uri() -> DocumentUri {
    DocumentUri::parse("file:///lsp-refactor-test/main.jai").unwrap()
}

fn open(text: &str) -> Session {
    let mut s = Session::with_environment(Limits::default(), environment());
    s.open(uri(), 1, text.into()).unwrap();
    s
}

fn position(text: &str, byte: usize) -> Position {
    let prefix = &text[..byte];
    Position {
        line: prefix.matches('\n').count() as u32,
        character: prefix.rsplit('\n').next().unwrap().len() as u32,
    }
}

/// The range of the first occurrence of `marker`.
fn select(text: &str, marker: &str) -> Range {
    let at = text.find(marker).unwrap_or_else(|| panic!("no {marker:?}"));
    Range {
        start: position(text, at),
        end: position(text, at + marker.len()),
    }
}

/// A cursor `offset` bytes into the first occurrence of `marker`.
fn cursor(text: &str, marker: &str, offset: usize) -> Range {
    let at = text.find(marker).unwrap() + offset;
    Range {
        start: position(text, at),
        end: position(text, at),
    }
}

fn byte(text: &str, p: Position) -> usize {
    let start: usize = text
        .split_inclusive('\n')
        .take(p.line as usize)
        .map(str::len)
        .sum();
    start + p.character as usize
}

/// The text after applying the action's edits.
fn apply(text: &str, action: &CodeAction) -> String {
    let (_, edits) = action.edit.as_ref().expect("an edit");
    let mut spans: Vec<(usize, usize, &str)> = edits
        .iter()
        .map(|e| {
            (
                byte(text, e.range.start),
                byte(text, e.range.end),
                e.new_text.as_str(),
            )
        })
        .collect();
    spans.sort();
    for pair in spans.windows(2) {
        assert!(pair[0].1 <= pair[1].0, "overlapping edits");
    }
    let mut out = text.to_string();
    for (start, end, new) in spans.into_iter().rev() {
        out.replace_range(start..end, new);
    }
    out
}

fn actions(text: &str, range: Range) -> Vec<CodeAction> {
    open(text)
        .code_actions(&uri(), range)
        .unwrap()
        .into_iter()
        .filter(|a| a.kind.is_some_and(|k| k.starts_with("refactor")))
        .collect()
}

fn titled<'a>(list: &'a [CodeAction], title: &str) -> Option<&'a CodeAction> {
    list.iter().find(|a| a.title.starts_with(title))
}

/// The edited program still compiles.
fn compiles(text: &str) {
    let s = open(text);
    let errors: Vec<_> = s
        .diagnostics(&uri())
        .unwrap()
        .into_iter()
        .filter(|d| d.severity == DiagnosticSeverity::Error)
        .collect();
    assert!(errors.is_empty(), "{text}\n{errors:?}");
}

/// Apply the action `title` for `range` and return the new text.
fn run(text: &str, range: Range, title: &str) -> String {
    let list = actions(text, range);
    let action = titled(&list, title).unwrap_or_else(|| panic!("no {title:?} in {list:?}"));
    let result = apply(text, action);
    compiles(&result);
    result
}

fn declined(text: &str, range: Range, title: &str) {
    let list = actions(text, range);
    assert!(
        titled(&list, title).is_none(),
        "{title:?} offered: {list:?}"
    );
}

// -------------------------------------------------------------------------------------------
// Extract into variable
// -------------------------------------------------------------------------------------------

const EXTRACT_VARIABLE: &str = "scale :: (a: int, b: int) -> int { return a * b; }
main :: () {
    n := 4;
    total := scale(n + 1, 2) + (n * 3);
    if n > 2 {
        total += scale(n, 3);
    }
}
";

#[test]
fn extracts_an_expression_into_a_variable() {
    let out = run(
        EXTRACT_VARIABLE,
        select(EXTRACT_VARIABLE, "n + 1"),
        "Extract into variable",
    );
    assert_eq!(
        out,
        "scale :: (a: int, b: int) -> int { return a * b; }
main :: () {
    n := 4;
    value := n + 1;
    total := scale(value, 2) + (n * 3);
    if n > 2 {
        total += scale(n, 3);
    }
}
"
    );
}

#[test]
fn extracts_a_parenthesized_operand_and_names_it_after_the_call() {
    let out = run(
        EXTRACT_VARIABLE,
        select(EXTRACT_VARIABLE, "(n * 3)"),
        "Extract into variable",
    );
    assert!(
        out.contains("    value := n * 3;\n    total := scale(n + 1, 2) + value;\n"),
        "{out}"
    );
    let out = run(
        EXTRACT_VARIABLE,
        select(EXTRACT_VARIABLE, "scale(n, 3)"),
        "Extract into variable",
    );
    assert!(
        out.contains("        scale := scale(n, 3);\n        total += scale;\n")
            || out.contains("        scale2 := scale(n, 3);\n        total += scale2;\n"),
        "{out}"
    );
}

#[test]
fn declines_to_extract_what_cannot_move() {
    let text = EXTRACT_VARIABLE;
    // A whole initializer, and text that is not one expression.
    declined(
        text,
        select(text, "scale(n + 1, 2) + (n * 3)"),
        "Extract into variable",
    );
    declined(text, select(text, "n + 1, 2"), "Extract into variable");
}

#[test]
fn declines_to_extract_conditions_that_run_each_time() {
    let text = "main :: () {
    n := 4;
    while n > 0 {
        n -= 1;
    }
    a := n > 1 && n < 9;
    b := ifx n > 3 then n * 2 else n;
}
";
    declined(text, select(text, "n > 0"), "Extract into variable");
    declined(text, select(text, "n < 9"), "Extract into variable");
    declined(text, select(text, "n * 2"), "Extract into variable");
    let out = run(text, select(text, "n > 1"), "Extract into variable");
    assert!(
        out.contains("    value := n > 1;\n    a := value && n < 9;\n"),
        "{out}"
    );
}

// -------------------------------------------------------------------------------------------
// Inline variable
// -------------------------------------------------------------------------------------------

#[test]
fn inlines_a_variable() {
    let text = "main :: () {
    n := 4;
    doubled := n * 2;
    total := doubled + 1;
    other := doubled * doubled;
}
";
    let out = run(text, cursor(text, "doubled :=", 2), "Inline variable");
    assert_eq!(
        out,
        "main :: () {
    n := 4;
    total := (n * 2) + 1;
    other := (n * 2) * (n * 2);
}
"
    );
    // From a use, too.
    let out = run(text, cursor(text, "doubled + 1", 1), "Inline variable");
    assert!(out.contains("total := (n * 2) + 1;"), "{out}");
}

#[test]
fn inlines_without_parentheses_where_none_are_needed() {
    let text = "show :: (v: int) {}
main :: () {
    n := 4;
    sum := n + 1;
    show(sum);
}
";
    let out = run(text, cursor(text, "sum :=", 1), "Inline variable");
    assert!(out.contains("    show(n + 1);\n"), "{out}");
    assert!(!out.contains("sum"), "{out}");
}

#[test]
fn declines_to_inline_a_variable_that_changes() {
    let text = "main :: () {
    n := 4;
    count := n * 2;
    count += 1;
    other := count;
}
";
    declined(text, cursor(text, "count :=", 1), "Inline variable");
    let text = "ptr :: (p: *int) {}
main :: () {
    n := 4;
    m := n * 2;
    ptr(*m);
}
";
    declined(text, cursor(text, "m :=", 0), "Inline variable");
    // A call run twice would do its work twice.
    let text = "next :: () -> int { return 1; }
main :: () {
    a := next();
    b := a + a;
}
";
    declined(text, cursor(text, "a :=", 0), "Inline variable");
}

// -------------------------------------------------------------------------------------------
// Fill in missing cases and fields
// -------------------------------------------------------------------------------------------

const KINDS: &str = "Kind :: enum { ONE; TWO; THREE; FOUR; }
describe :: (k: Kind) -> int {
    result := 0;
    if k == {
        case .ONE;
            result = 1;
        case .THREE;
            result = 3;
    }
    return result;
}
";

#[test]
fn fills_in_the_missing_cases_of_an_enum_switch() {
    let out = run(KINDS, cursor(KINDS, "if k ==", 1), "Add missing cases");
    assert_eq!(
        out,
        "Kind :: enum { ONE; TWO; THREE; FOUR; }
describe :: (k: Kind) -> int {
    result := 0;
    if k == {
        case .ONE;
            result = 1;
        case .THREE;
            result = 3;
        case .TWO;
        case .FOUR;
    }
    return result;
}
"
    );
}

#[test]
fn missing_cases_go_before_the_default_case() {
    let text = "Kind :: enum { ONE; TWO; THREE; }
f :: (k: Kind) -> int {
    if k == #complete {
        case .TWO; return 2;
        case; return 0;
    }
    return 1;
}
";
    let out = run(text, cursor(text, "if k", 0), "Add missing cases (2)");
    assert!(
        out.contains("        case .TWO; return 2;\n        case .ONE;\n        case .THREE;\n        case; return 0;\n"),
        "{out}"
    );
}

#[test]
fn offers_no_cases_when_nothing_is_missing_or_the_cursor_is_in_a_case() {
    let full = KINDS.replace(
        "        case .THREE;\n            result = 3;\n",
        "        case .THREE;\n            result = 3;\n        case .TWO; #through;\n        case .FOUR;\n",
    );
    declined(&full, cursor(&full, "if k ==", 1), "Add missing cases");
    declined(&full, cursor(&full, "result = 1", 1), "Add missing cases");
}

const POINTS: &str = "Point :: struct { x: int; y: int; name: string; ok: bool; label: *u8; }
main :: () {
    a := Point.{x = 1};
    b := Point.{};
    c := Point.{
        x = 1,
        y = 2,
    };
}
";

#[test]
fn fills_in_missing_struct_fields() {
    let out = run(POINTS, cursor(POINTS, "Point.{x", 3), "Add missing fields");
    assert!(
        out.contains("    a := Point.{x = 1, y = 0, name = \"\", ok = false, label = null};\n"),
        "{out}"
    );
    let out = run(POINTS, cursor(POINTS, "Point.{}", 6), "Add missing fields");
    assert!(
        out.contains("    b := Point.{ x = 0, y = 0, name = \"\", ok = false, label = null };\n"),
        "{out}"
    );
    let out = run(POINTS, cursor(POINTS, "y = 2", 0), "Add missing fields");
    assert!(
        out.contains(
            "        x = 1,\n        y = 2,\n        name = \"\",\n        ok = false,\n        label = null,\n    };\n"
        ),
        "{out}"
    );
}

#[test]
fn declines_to_fill_positional_literals() {
    let text = "Point :: struct { x: int; y: int; }
main :: () {
    a := Point.{1};
}
";
    declined(text, cursor(text, "Point.{1", 3), "Add missing fields");
}

// -------------------------------------------------------------------------------------------
// ifx <-> if / else
// -------------------------------------------------------------------------------------------

#[test]
fn converts_ifx_to_if_else() {
    let text = "main :: () {
    n := 4;
    big := ifx n > 3 then 10 else 20;
    big = ifx n > 5 then 1 else 2;
}
";
    let out = run(
        text,
        cursor(text, "big := ifx", 0),
        "Convert ifx to if/else",
    );
    assert_eq!(
        out,
        "main :: () {
    n := 4;
    big: s64;
    if n > 3 {
        big = 10;
    } else {
        big = 20;
    }
    big = ifx n > 5 then 1 else 2;
}
"
    );
    let out = run(text, cursor(text, "big = ifx", 0), "Convert ifx to if/else");
    assert!(
        out.contains("    if n > 5 {\n        big = 1;\n    } else {\n        big = 2;\n    }\n"),
        "{out}"
    );
}

#[test]
fn converts_ifx_in_a_return() {
    let text = "pick :: (n: int) -> int {
    return ifx n > 3 then 10 else 20;
}
";
    let out = run(text, cursor(text, "return", 0), "Convert ifx to if/else");
    assert_eq!(
        out,
        "pick :: (n: int) -> int {
    if n > 3 {
        return 10;
    } else {
        return 20;
    }
}
"
    );
}

#[test]
fn converts_if_else_to_ifx() {
    let text = "main :: () {
    n := 4;
    big := 0;
    if n > 3 {
        big = 10;
    } else {
        big = 20;
    }
}
pick :: (n: int) -> int {
    if n > 3 return 10; else return 20;
}
";
    let out = run(
        text,
        cursor(text, "if n > 3 {", 0),
        "Convert if/else to ifx",
    );
    assert!(
        out.contains("    big = ifx n > 3 then 10 else 20;\n"),
        "{out}"
    );
    let out = run(
        text,
        cursor(text, "if n > 3 return", 0),
        "Convert if/else to ifx",
    );
    assert!(
        out.contains("    return ifx n > 3 then 10 else 20;\n"),
        "{out}"
    );
}

#[test]
fn declines_to_convert_if_else_that_differs() {
    let text = "main :: () {
    a := 0;
    b := 0;
    n := 4;
    if n > 3 {
        a = 1;
    } else {
        b = 2;
    }
    if n > 3 {
        a = 1;
        b = 1;
    } else {
        a = 2;
    }
}
";
    declined(text, cursor(text, "if n > 3", 0), "Convert if/else to ifx");
    let last = text.rfind("if n > 3").unwrap();
    let range = Range {
        start: position(text, last),
        end: position(text, last),
    };
    declined(text, range, "Convert if/else to ifx");
}

// -------------------------------------------------------------------------------------------
// Extract into procedure
// -------------------------------------------------------------------------------------------

const EXTRACT_PROCEDURE: &str = "compute :: (base: int, items: []int) -> int {
    sum := 0;
    for items {
        sum += it;
    }
    scaled := sum * base;
    label := \"total\";
    return scaled + label.count;
}
";

#[test]
fn extracts_statements_into_a_procedure() {
    let selection = "    scaled := sum * base;\n    label := \"total\";";
    let range = select(EXTRACT_PROCEDURE, selection.trim_start());
    let out = run(EXTRACT_PROCEDURE, range, "Extract into procedure");
    assert_eq!(
        out,
        "compute :: (base: int, items: []int) -> int {
    sum := 0;
    for items {
        sum += it;
    }
    scaled, label := extracted(sum, base);
    return scaled + label.count;
}

extracted :: (sum: s64, base: int) -> s64, string {
    scaled := sum * base;
    label := \"total\";
    return scaled, label;
}
"
    );
}

#[test]
fn extracts_a_loop_that_uses_a_parameter() {
    let text = "show :: (v: int) {}
run :: (items: []int) {
    for items {
        show(it);
    }
}
";
    let out = run(
        text,
        select(text, "for items {\n        show(it);\n    }"),
        "Extract into procedure",
    );
    assert_eq!(
        out,
        "show :: (v: int) {}
run :: (items: []int) {
    extracted(items);
}

extracted :: (items: []int) {
    for items {
        show(it);
    }
}
"
    );
}

#[test]
fn declines_to_extract_what_cannot_be_a_procedure() {
    let text = "f :: (items: []int) -> int {
    total := 0;
    for items {
        if it < 0 break;
        total += it;
    }
    if total > 9 return total;
    count := 0;
    count += 1;
    return count;
}
";
    // Assigns an outer variable.
    declined(text, select(text, "total += it;"), "Extract into procedure");
    // A `return` that would leave the caller.
    declined(
        text,
        select(text, "if total > 9 return total;"),
        "Extract into procedure",
    );
    // A `break` whose loop is outside the selection.
    declined(
        text,
        select(text, "if it < 0 break;"),
        "Extract into procedure",
    );
    // Half a statement.
    declined(
        text,
        select(text, "count := 0;\n    count"),
        "Extract into procedure",
    );
    // Changes a variable declared before it.
    declined(text, select(text, "count += 1;"), "Extract into procedure");
}

// -------------------------------------------------------------------------------------------
// Whatever is selected, an offered edit leaves a program that parses
// -------------------------------------------------------------------------------------------

#[test]
fn offered_refactorings_always_parse() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let mut offered = 0;
    for name in [
        "internal/build/autorun.jai",
        "internal/developer-tools/performance-report.jai",
        "internal/tooling/bundle.jai",
    ] {
        let text = std::fs::read_to_string(dir.join(name)).unwrap();
        let s = open(&text);
        let starts: Vec<usize> = std::iter::once(0)
            .chain(text.match_indices('\n').map(|(i, _)| i + 1))
            .collect();
        for (n, &start) in starts.iter().enumerate() {
            let end = text[start..].find('\n').map_or(text.len(), |i| start + i);
            let last = starts
                .get(n + 2)
                .map_or(text.len(), |&e| e.saturating_sub(1));
            // The line, two lines, and the cursor at the first word.
            let word = start + (text[start..end].len() - text[start..end].trim_start().len());
            for (a, b) in [
                (start, end),
                (start, last.max(end)),
                (word, word),
                (word + 1, end),
            ] {
                let (a, b) = (a.min(b), b);
                if !text.is_char_boundary(a) || !text.is_char_boundary(b) {
                    continue;
                }
                let range = Range {
                    start: position(&text, a),
                    end: position(&text, b),
                };
                for action in s.code_actions(&uri(), range).unwrap() {
                    if !action.kind.is_some_and(|k| k.starts_with("refactor")) {
                        continue;
                    }
                    offered += 1;
                    let result = apply(&text, &action);
                    assert!(
                        jaic::parser::parse_file(jaic::source::FileId(0), &result).is_ok(),
                        "{} in {name} at {a}..{b} breaks the program:\n{result}",
                        action.title
                    );
                }
            }
        }
    }
    assert!(offered > 10, "{offered}");
}

const INCOMPLETE: &str = "#import \"Basic\";
Kind :: enum { NOTHING; INDEX; NULL; }
CHOICE :: Kind.NOTHING;
main :: () {
    if #complete CHOICE == {
        case .NOTHING;
            print(\"nothing\\n\");
        case .INDEX;
            three := int.[1, 2, 3];
            index := three.count;
            print(\"%\\n\", three[index]);
    }
}
";

#[test]
fn an_incomplete_switch_keeps_types_and_offers_its_missing_cases() {
    let s = open(INCOMPLETE);
    let hover = s
        .hover(&uri(), cursor(INCOMPLETE, "index :=", 1).start)
        .unwrap()
        .expect("hover")
        .contents
        .value;
    assert_eq!(hover, "index: s64");
    let end = position(INCOMPLETE, INCOMPLETE.len());
    let hints = s
        .inlay_hints(
            &uri(),
            Range {
                start: position(INCOMPLETE, 0),
                end,
            },
        )
        .unwrap();
    assert!(hints.iter().any(|h| h.label == ": s64"), "{hints:?}");
    // On the error: a preferred quick fix, and no second copy among the refactorings.
    let all = s
        .code_actions(&uri(), cursor(INCOMPLETE, "CHOICE ==", 1))
        .unwrap();
    let fills: Vec<_> = all
        .iter()
        .filter(|a| a.title.starts_with("Add missing cases"))
        .collect();
    assert_eq!(fills.len(), 1, "{all:?}");
    assert_eq!(fills[0].kind, Some("quickfix"));
    assert!(fills[0].is_preferred);
    assert_eq!(fills[0].diagnostics.len(), 1);
    assert!(
        apply(INCOMPLETE, fills[0])
            .contains("            print(\"%\\n\", three[index]);\n        case .NULL;\n    }")
    );
}
