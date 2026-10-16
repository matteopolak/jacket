//! Doc comments: hover, completion, signature help and intra-doc links, against the
//! repository's stdlib.
use jai_language_server::{
    CompletionKind, DocumentUri, Environment, Limits, MarkupKind, Position, Session, TextChange,
};
use std::path::PathBuf;

fn session() -> Session {
    let stdlib = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    Session::with_environment(
        Limits::default(),
        Environment {
            fs: std::rc::Rc::new(jaic::sema::NativeFs),
            options: Box::new(move |_| {
                let mut options = jaic::sema::Options::host();
                options.import_paths = vec![stdlib.clone()];
                options.preload = Some(stdlib.join(jaic::PRELOAD_ENTRY));
                options
            }),
        },
    )
}

fn uri() -> DocumentUri {
    DocumentUri::parse("file:///lsp-doc-test/main.jai").unwrap()
}

/// Position just after the `n`th (0-based) occurrence of `marker`, minus `back` characters.
fn after(text: &str, marker: &str, n: usize, back: usize) -> Position {
    let byte = text.match_indices(marker).nth(n).unwrap().0 + marker.len() - back;
    let prefix = &text[..byte];
    Position {
        line: prefix.matches('\n').count() as u32,
        character: prefix.rsplit('\n').next().unwrap().len() as u32,
    }
}

fn markdown_hover(session: &Session, at: Position) -> String {
    session
        .hover_as(&uri(), at, MarkupKind::Markdown)
        .unwrap()
        .expect("hover")
        .contents
        .value
}

fn plain_hover(session: &Session, at: Position) -> String {
    session
        .hover(&uri(), at)
        .unwrap()
        .expect("hover")
        .contents
        .value
}

/// The 1-based line of the first line containing `marker`.
fn line_of(text: &str, marker: &str) -> usize {
    text.lines().position(|l| l.contains(marker)).unwrap() + 1
}

const PROGRAM: &str = r#"#import "Basic";

// A point on the screen.
//
// ```
// p: Point;
// ```
Point :: struct {
    x: int; // Horizontal position.
    // Vertical position, like [Point.x].
    y: int;
}

// The ways to move.
Mode :: enum {
    // Do nothing.
    IDLE;
    RUNNING; // Keep going.
}

/// Scales a [Point] by a factor, starting from [`Mode.IDLE`].
///
/// - p: the point to scale
/// - `factor` — how much to scale by
/// - not_a_parameter: ignored
scale :: (
    p: Point,
    factor: int,
    // Printed when set.
    verbose := false,
) -> Point {
    return .{ p.x * factor, p.y * factor };
}

// Adds integers.
add :: (a: int, b: int) -> int { return a + b; }

// Adds floats.
add :: (a: float, b: float) -> float { return a + b; }

/* Block docs
 * for the runner.
 */
run :: () {
    point: Point;
    scaled := scale(point, 2);
    sum := add(1, 2);
    mode := Mode.IDLE;
    print("%\n", scaled.x + sum);
    nothing(1);
}

nothing :: (value: int) {}
"#;

fn open() -> Session {
    let mut s = session();
    s.open(uri(), 1, PROGRAM.into()).unwrap();
    s
}

#[test]
fn hover_shows_the_docs_of_procedures_structs_fields_and_members() {
    let s = open();
    let proc = markdown_hover(&s, after(PROGRAM, "scale(point", 0, 7));
    assert!(proc.starts_with("```jai\nscale :: ("), "{proc}");
    assert!(proc.contains("Scales a "), "{proc}");
    let strukt = markdown_hover(&s, after(PROGRAM, "point: Point", 0, 3));
    assert!(strukt.contains("A point on the screen."), "{strukt}");
    let field = markdown_hover(&s, after(PROGRAM, "scaled.x", 0, 0));
    assert!(field.contains("Horizontal position."), "{field}");
    let above = markdown_hover(&s, after(PROGRAM, "p.y *", 0, 2));
    assert!(above.contains("Vertical position"), "{above}");
    let member = markdown_hover(&s, after(PROGRAM, "Mode.IDLE;", 0, 2));
    assert!(member.contains("Do nothing."), "{member}");
    let trailing = markdown_hover(&s, after(PROGRAM, "RUNNING", 0, 2));
    assert!(trailing.contains("Keep going."), "{trailing}");
    let block = markdown_hover(&s, after(PROGRAM, "run :: ", 0, 5));
    assert!(block.contains("Block docs\nfor the runner."), "{block}");
}

#[test]
fn plain_text_clients_get_the_comment_as_written() {
    let s = open();
    let hover = plain_hover(&s, after(PROGRAM, "point: Point", 0, 3));
    assert!(hover.contains("A point on the screen."), "{hover}");
    assert!(hover.contains("```\np: Point;\n```"), "{hover}");
}

#[test]
fn untagged_fences_are_jai() {
    let s = open();
    let hover = markdown_hover(&s, after(PROGRAM, "point: Point", 0, 3));
    assert!(hover.contains("```jai\np: Point;\n```"), "{hover}");
}

#[test]
fn each_overload_shows_its_own_docs() {
    let s = open();
    let hover = markdown_hover(&s, after(PROGRAM, "add(1", 0, 3));
    let ints = hover.find("Adds integers.").expect(&hover);
    let floats = hover.find("Adds floats.").expect(&hover);
    assert!(hover.find("add :: (a: int").unwrap() < ints, "{hover}");
    assert!(ints < hover.find("add :: (a: float").unwrap(), "{hover}");
    assert!(hover.find("add :: (a: float").unwrap() < floats, "{hover}");
}

#[test]
fn parameters_have_docs_from_the_list_or_their_own_comments() {
    let s = open();
    let factor = markdown_hover(&s, after(PROGRAM, "p.x * factor", 0, 2));
    assert!(factor.contains("how much to scale by"), "{factor}");
    let declared = markdown_hover(&s, after(PROGRAM, "    p: Point", 0, 8));
    assert!(declared.contains("the point to scale"), "{declared}");
    let own = markdown_hover(&s, after(PROGRAM, "verbose := false", 0, 10));
    assert!(own.contains("Printed when set."), "{own}");
}

#[test]
fn signature_help_documents_the_active_parameter() {
    let s = open();
    let help = s
        .signature_help_as(
            &uri(),
            after(PROGRAM, "scale(point, ", 0, 0),
            MarkupKind::Markdown,
        )
        .unwrap()
        .expect("signature help");
    assert_eq!(help.active_parameter, 1);
    let signature = &help.signatures[help.active_signature];
    assert!(
        signature
            .documentation
            .as_deref()
            .unwrap()
            .contains("Scales a ")
    );
    assert_eq!(
        signature.parameter_docs[0].as_deref(),
        Some("the point to scale")
    );
    assert_eq!(
        signature.parameter_docs[help.active_parameter].as_deref(),
        Some("how much to scale by")
    );
    assert_eq!(
        signature.parameter_docs[2].as_deref(),
        Some("Printed when set.")
    );
}

#[test]
fn completion_items_carry_documentation() {
    let s = open();
    let list = s
        .completion(&uri(), after(PROGRAM, "    nothing", 0, 0))
        .unwrap();
    let item = list
        .items
        .iter()
        .find(|i| i.label == "nothing")
        .expect("nothing");
    assert_eq!(item.documentation, None);
    let scaled = s
        .completion(&uri(), after(PROGRAM, "scaled := sc", 0, 0))
        .unwrap();
    let item = scaled
        .items
        .iter()
        .find(|i| i.label == "scale")
        .expect("scale");
    assert_eq!(item.kind, CompletionKind::Function);
    assert!(item.documentation.as_deref().unwrap().contains("Scales a "));
    let member = s.completion(&uri(), after(PROGRAM, "Mode.", 1, 0)).unwrap();
    let idle = member
        .items
        .iter()
        .find(|i| i.label == "IDLE")
        .expect("IDLE");
    assert!(
        idle.documentation
            .as_deref()
            .unwrap()
            .contains("Do nothing.")
    );
}

#[test]
fn long_completion_lists_resolve_documentation_on_demand() {
    let s = open();
    let list = s
        .completion(&uri(), after(PROGRAM, "    nothing", 0, 7))
        .unwrap();
    assert!(list.items.len() > 24);
    let item = list
        .items
        .iter()
        .find(|i| i.label == "scale")
        .expect("scale");
    assert_eq!(item.documentation, None);
    let docs = s
        .completion_documentation(item.data.as_ref().expect("data"))
        .expect("docs");
    assert!(docs.contains("Scales a "), "{docs}");
}

#[test]
fn stdlib_declarations_show_their_docs() {
    let s = open();
    let hover = markdown_hover(&s, after(PROGRAM, "print(\"%", 0, 5));
    assert!(
        hover.contains("Writes the formatted text to standard output"),
        "{hover}"
    );
    assert!(
        hover.contains("```jai\nprint(\"% + % = %\\n\", 1, 2, 1 + 2);\n```"),
        "{hover}"
    );
    let help = s
        .signature_help_as(
            &uri(),
            after(PROGRAM, "print(\"%\\n\", ", 0, 0),
            MarkupKind::Markdown,
        )
        .unwrap()
        .expect("signature help");
    let signature = &help.signatures[help.active_signature];
    assert_eq!(
        signature.parameter_docs[help.active_parameter].as_deref(),
        Some("the values to print, in order")
    );
}

#[test]
fn imported_declarations_show_their_authored_docs() {
    // Public signatures do not prescribe library prose or private source layout.
    let mut s = session();
    let imported = DocumentUri::parse("file:///lsp-doc-test/documented.jai").unwrap();
    s.open(imported, 1, "/// Visible docs across imports.\n/// ```jai\n/// M.exposed(7);\n/// ```\n/// - value: the fixture input\nexposed :: (value: int) {}".into()).unwrap();
    let text = "M :: #import,file \"documented.jai\";\nmain :: () { M.exposed(7); }";
    s.open(uri(), 1, text.into()).unwrap();
    let hover = markdown_hover(&s, after(text, "M.exposed", 0, 2));
    assert!(hover.contains("Visible docs across imports."), "{hover}");
    assert!(hover.contains("```jai\nM.exposed(7);\n```"), "{hover}");
    let help = s
        .signature_help_as(
            &uri(),
            after(text, "M.exposed(", 0, 0),
            MarkupKind::Markdown,
        )
        .unwrap()
        .expect("signature help");
    let signature = &help.signatures[help.active_signature];
    assert_eq!(
        signature.parameter_docs[help.active_parameter].as_deref(),
        Some("the fixture input")
    );
}

#[test]
fn links_in_docs_become_markdown_links() {
    let s = open();
    let hover = markdown_hover(&s, after(PROGRAM, "scale(point", 0, 7));
    let point = line_of(PROGRAM, "Point :: struct");
    assert!(
        hover.contains(&format!("[Point](file:///lsp-doc-test/main.jai#L{point})")),
        "{hover}"
    );
    let idle = line_of(PROGRAM, "IDLE;");
    assert!(
        hover.contains(&format!(
            "[`Mode.IDLE`](file:///lsp-doc-test/main.jai#L{idle})"
        )),
        "{hover}"
    );
    let field = markdown_hover(&s, after(PROGRAM, "p.y *", 0, 2));
    let x = line_of(PROGRAM, "x: int;");
    assert!(field.contains(&format!("#L{x})")), "{field}");
}

#[test]
fn unresolved_links_stay_text() {
    let mut s = session();
    let text = "// See [missing] and [Basic.nothing_here].\nf :: () {}\nmain :: () { f(); }\n";
    s.open(uri(), 1, text.into()).unwrap();
    let hover = markdown_hover(&s, after(text, "f(", 0, 2));
    assert!(
        hover.contains("See [missing] and [Basic.nothing_here]."),
        "{hover}"
    );
}

#[test]
fn definition_hover_and_document_links_work_inside_comments() {
    let s = open();
    let in_comment = after(PROGRAM, "[Point] by", 0, 6);
    let definition = s.definition(&uri(), in_comment).unwrap();
    assert_eq!(definition.len(), 1);
    assert_eq!(
        definition[0].range.start.line as usize + 1,
        line_of(PROGRAM, "Point :: struct")
    );
    let hover = markdown_hover(&s, in_comment);
    assert!(hover.contains("Point :: struct"), "{hover}");
    assert!(hover.contains("A point on the screen."), "{hover}");
    let qualified = s
        .definition(&uri(), after(PROGRAM, "[`Mode.IDLE`", 0, 3))
        .unwrap();
    assert_eq!(
        qualified[0].range.start.line as usize + 1,
        line_of(PROGRAM, "IDLE;")
    );
    let links = s.document_links(&uri()).unwrap();
    let targets: Vec<&str> = links.iter().map(|(_, t)| t.as_str()).collect();
    let point = format!(
        "file:///lsp-doc-test/main.jai#L{}",
        line_of(PROGRAM, "Point :: struct")
    );
    assert!(targets.contains(&point.as_str()), "{targets:?}");
    assert_eq!(targets.iter().filter(|t| **t == point).count(), 1);
    assert!(
        links
            .iter()
            .any(|(r, t)| t == &point && r.start.line as usize + 1 == line_of(PROGRAM, "Scales"))
    );
}

#[test]
fn resolved_links_are_semantic_tokens() {
    let s = open();
    let tokens = s.semantic_tokens(&uri()).unwrap();
    let line = (line_of(PROGRAM, "Scales a") - 1) as u32;
    let on_line: Vec<_> = tokens.iter().filter(|t| t.position.line == line).collect();
    assert_eq!(on_line.len(), 2, "{on_line:?}");
    assert_eq!(
        on_line[0].kind,
        jai_language_server::SemanticTokenKind::Type
    );
    assert_eq!(on_line[0].length, "Point".len() as u32);
    assert_eq!(
        on_line[1].kind,
        jai_language_server::SemanticTokenKind::EnumMember
    );
}

const STRUCT_MEMBERS: &str = r#"#import "Basic";

Outer :: struct {
    SIDES :: 4;
    Inner :: struct {
        DEPTH :: 2;
    }
    area :: (o: Outer) -> int { return o.count * SIDES; }
    count: int;
}

// Counts [Outer.SIDES] with [Outer.area()], inside [Outer.Inner], see [`Outer.Inner.DEPTH`],
// [Outer.count] and [Outer.missing] and [Outer.Inner.nothing].
walk :: () {}

main :: () {
    walk();
    o: Outer;
    print("%\n", Outer.area(o));
}
"#;

fn open_with(text: &str) -> Session {
    let mut s = session();
    s.open(uri(), 1, text.into()).unwrap();
    s
}

#[test]
fn struct_constants_procedures_and_types_resolve_in_links() {
    let s = open_with(STRUCT_MEMBERS);
    let line_of_definition = |at: Position| {
        let found = s.definition(&uri(), at).unwrap();
        assert_eq!(found.len(), 1, "{found:?}");
        found[0].range.start.line as usize + 1
    };
    assert_eq!(
        line_of_definition(after(STRUCT_MEMBERS, "[Outer.SIDES", 0, 3)),
        line_of(STRUCT_MEMBERS, "SIDES ::")
    );
    assert_eq!(
        line_of_definition(after(STRUCT_MEMBERS, "[Outer.area", 0, 2)),
        line_of(STRUCT_MEMBERS, "area ::")
    );
    assert_eq!(
        line_of_definition(after(STRUCT_MEMBERS, "[Outer.Inner]", 0, 4)),
        line_of(STRUCT_MEMBERS, "Inner ::")
    );
    // A constant of a type nested in the struct, written in code quotes.
    assert_eq!(
        line_of_definition(after(STRUCT_MEMBERS, "Outer.Inner.DEPTH", 0, 2)),
        line_of(STRUCT_MEMBERS, "DEPTH ::")
    );
    // Fields still work next to them.
    assert_eq!(
        line_of_definition(after(STRUCT_MEMBERS, "[Outer.count", 0, 2)),
        line_of(STRUCT_MEMBERS, "count: int")
    );
    // What is not declared in the struct is not found.
    assert!(
        s.definition(&uri(), after(STRUCT_MEMBERS, "[Outer.missing", 0, 2))
            .unwrap()
            .is_empty()
    );
    assert!(
        s.definition(&uri(), after(STRUCT_MEMBERS, "Inner.nothing", 0, 2))
            .unwrap()
            .is_empty()
    );
}

const POLY_STRUCT: &str = r#"Ring :: struct(Item: Type, CAPACITY: int) {
    items: [CAPACITY] Item;
    count: int;
}

// Holds [Ring.items], at most [Ring.CAPACITY], not [Ring.missing].
fill :: () {}

main :: () { fill(); }
"#;

#[test]
fn polymorphic_struct_fields_and_parameters_resolve_in_links() {
    let s = open_with(POLY_STRUCT);
    let line_of_definition = |at: Position| {
        let found = s.definition(&uri(), at).unwrap();
        assert_eq!(found.len(), 1, "{found:?}");
        found[0].range.start.line as usize + 1
    };
    // Resolved from the source: the program never instantiates `Ring`.
    assert_eq!(
        line_of_definition(after(POLY_STRUCT, "[Ring.items", 0, 2)),
        line_of(POLY_STRUCT, "items:")
    );
    assert_eq!(
        line_of_definition(after(POLY_STRUCT, "[Ring.CAPACITY", 0, 2)),
        line_of(POLY_STRUCT, "Ring ::")
    );
    assert!(
        s.definition(&uri(), after(POLY_STRUCT, "[Ring.missing", 0, 2))
            .unwrap()
            .is_empty()
    );
}

#[test]
fn struct_constant_links_are_document_links_hovers_and_tokens() {
    let s = open_with(STRUCT_MEMBERS);
    let links = s.document_links(&uri()).unwrap();
    let walk_line = line_of(STRUCT_MEMBERS, "Counts");
    let targets: Vec<&str> = links
        .iter()
        .filter(|(r, _)| r.start.line as usize + 1 == walk_line)
        .map(|(_, t)| t.as_str())
        .collect();
    for marker in ["SIDES ::", "area ::", "Inner ::", "DEPTH ::"] {
        let want = format!(
            "file:///lsp-doc-test/main.jai#L{}",
            line_of(STRUCT_MEMBERS, marker)
        );
        assert!(targets.contains(&want.as_str()), "{marker}: {targets:?}");
    }
    let hover = markdown_hover(&s, after(STRUCT_MEMBERS, "[Outer.SIDES", 0, 3));
    assert!(hover.contains("SIDES :: 4"), "{hover}");
    let tokens = s.semantic_tokens(&uri()).unwrap();
    let line = (walk_line - 1) as u32;
    let kinds: Vec<_> = tokens
        .iter()
        .filter(|t| t.position.line == line)
        .map(|t| t.kind)
        .collect();
    use jai_language_server::SemanticTokenKind as K;
    assert_eq!(
        kinds,
        [K::Variable, K::Function, K::Type, K::Variable],
        "{kinds:?}"
    );
}

#[test]
fn module_qualified_struct_constants_resolve() {
    let dir = std::env::temp_dir().join(format!("jai-doc-links-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let module = dir.join("shapes.jai");
    std::fs::write(
        &module,
        "Square :: struct {\n    SIDES :: 4;\n    side: int;\n}\nmake :: () -> Square { return .{}; }\n",
    )
    .unwrap();
    let text = format!(
        "Shapes :: #import,file \"{}\";\n\n// Uses [Shapes.Square.SIDES] and [Shapes.Square.side] and [Shapes.Square.nothing].\nf :: () {{ s := Shapes.make(); }}\n",
        module.display()
    );
    let s = open_with(&text);
    let found = s
        .definition(&uri(), after(&text, "[Shapes.Square.SIDES", 0, 2))
        .unwrap();
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].range.start.line, 1);
    let field = s
        .definition(&uri(), after(&text, "[Shapes.Square.side", 0, 2))
        .unwrap();
    assert_eq!(field[0].range.start.line, 2);
    assert!(
        s.definition(&uri(), after(&text, "Square.nothing", 0, 2))
            .unwrap()
            .is_empty()
    );
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn broken_doc_links_are_lint_diagnostics_with_a_quick_fix() {
    use jai_language_server::{DiagnosticCode, Range};
    let text = "#import \"Basic\";\n// Prints with [prnt].\nf :: () {}\nmain :: () { f(); print(\"x\\n\"); }\n";
    let s = open_with(text);
    let lints: Vec<_> = s
        .diagnostics(&uri())
        .unwrap()
        .into_iter()
        .filter(|d| d.code == DiagnosticCode::Lint("broken_doc_link"))
        .collect();
    assert_eq!(lints.len(), 1, "{lints:?}");
    assert_eq!(lints[0].range.start, after(text, "[prnt", 0, 5));
    let cursor = after(text, "[prnt", 0, 2);
    let actions = s
        .code_actions(
            &uri(),
            Range {
                start: cursor,
                end: cursor,
            },
        )
        .unwrap();
    let fix = actions
        .iter()
        .find(|a| a.rule == Some("broken_doc_link"))
        .expect("a quick fix");
    assert_eq!(fix.kind, Some("quickfix"));
    assert_eq!(fix.title, "Link to `print`");
    assert!(!fix.is_preferred);
    let (_, edits) = fix.edit.as_ref().unwrap();
    assert_eq!(edits[0].new_text, "print");
}

const BELOW: &str = r#"#import "Basic";

main :: () {
    q, r := divide(47, 5);
    print("%\n", hail("x"));
    print("%\n", q);
    print("%\n", r);
    print("%\n", q + r);
    print("%\n", q - r);
    print("%\n", q * r);
    print("%\n", q / r);
}

// Whole-number division.
//
// Returns the `quotient` and the `remainder`.
divide :: (a: int, b: int) -> quotient: int, remainder: int {
    return a / b, a % b;
}

// Greets `who`; see [divide].
hail :: (who: string, times := 1) -> string {
    return who;
}
"#;

/// `BELOW` compiled, then edited (as an editor does) to `text`.
fn edited(text: &str) -> Session {
    let mut s = open_with(BELOW);
    let change = TextChange {
        range: None,
        range_length: None,
        text: text.into(),
    };
    s.change(&uri(), 2, &[change]).unwrap();
    s
}

#[test]
fn completion_docs_of_declarations_below_an_edit_are_whole() {
    // The compile completion answers from predates the new line, so `divide` sits a line
    // further down in the document than the compile says.
    let text = BELOW.replace("    q, r :=", "    x := divi\n    q, r :=");
    let s = edited(&text);
    let list = s
        .completion(&uri(), after(&text, "x := divi", 0, 0))
        .unwrap();
    let item = list.items.iter().find(|i| i.label == "divide").unwrap();
    let docs = item.documentation.as_deref().unwrap();
    assert!(docs.starts_with("Whole-number division."), "{docs}");
    assert!(
        docs.ends_with("Returns the `quotient` and the `remainder`."),
        "{docs}"
    );
}

#[test]
fn signature_help_of_an_open_call_has_docs_and_links() {
    // Unclosed, the call swallows the rest of the file: the parse error is at its end.
    let text = BELOW.replace("hail(\"x\"));", "hail(\"x\", ");
    let s = edited(&text);
    let help = s
        .signature_help_as(
            &uri(),
            after(&text, "hail(\"x\", ", 0, 0),
            MarkupKind::Markdown,
        )
        .unwrap()
        .expect("signature help");
    assert_eq!(help.active_parameter, 1);
    let docs = help.signatures[0].documentation.as_deref().unwrap();
    let line = line_of(&text, "divide ::");
    assert!(
        docs.contains(&format!("[divide](file:///lsp-doc-test/main.jai#L{line})")),
        "{docs}"
    );
}

#[test]
fn code_in_comment_fences_is_highlighted_as_jai() {
    use jai_language_server::SemanticTokenKind as K;
    let text = "// Greets `who`:\n//\n// ```\n// s := hail(\"Ford\", 2); // \"Hey Hey Ford\"\n// ```\nhail :: (who: string, times := 1) -> string { return who; }\n";
    let s = open_with(text);
    let tokens = s.semantic_tokens(&uri()).unwrap();
    let kinds: Vec<(u32, K)> = tokens
        .iter()
        .filter(|t| t.position.line == 3)
        .map(|t| (t.position.character, t.kind))
        .collect();
    let at = |word: &str| text.lines().nth(3).unwrap().find(word).unwrap() as u32;
    assert_eq!(kinds[0], (at("s :="), K::Variable), "{kinds:?}");
    assert!(kinds.contains(&(at("hail"), K::Function)), "{kinds:?}");
    assert!(kinds.contains(&(at("\"Ford\""), K::String)), "{kinds:?}");
    assert!(kinds.contains(&(at("2)"), K::Number)), "{kinds:?}");
    // The comment inside the code stays a comment.
    assert!(kinds.iter().all(|&(c, _)| c < at("// \"Hey")), "{kinds:?}");
    // Prose lines are not code.
    assert!(tokens.iter().all(|t| t.position.line != 0), "{tokens:?}");
}

#[test]
fn completion_docs_carry_links_eagerly_and_when_resolved() {
    let text = BELOW.replace("    q, r :=", "    w := hai\n    q, r :=");
    let s = edited(&text);
    let list = s
        .completion(&uri(), after(&text, "w := hai", 0, 0))
        .unwrap();
    let item = list.items.iter().find(|i| i.label == "hail").unwrap();
    let link = format!(
        "[divide](file:///lsp-doc-test/main.jai#L{})",
        line_of(&text, "divide ::")
    );
    let eager = item.documentation.as_deref().unwrap();
    assert!(eager.contains(&link), "{eager}");
    let resolved = s
        .completion_documentation(item.data.as_ref().unwrap())
        .unwrap();
    assert!(resolved.contains(&link), "{resolved}");
}
