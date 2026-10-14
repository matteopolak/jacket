//! Call hierarchy and selection ranges.
use jai_language_server::{
    DocumentUri, Environment, JsonSession, Limits, Position, Range, Session,
};
use serde_json::{Value, json};
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
    DocumentUri::parse("file:///lsp-hierarchy-test/main.jai").unwrap()
}

fn at(text: &str, marker: &str, offset: usize) -> Position {
    let byte = text.find(marker).unwrap_or_else(|| panic!("no {marker:?}")) + offset;
    let prefix = &text[..byte];
    Position {
        line: prefix.matches('\n').count() as u32,
        character: prefix.rsplit('\n').next().unwrap().len() as u32,
    }
}

fn open(text: &str) -> Session {
    let mut s = Session::with_environment(Limits::default(), environment());
    s.open(uri(), 1, text.into()).unwrap();
    s
}

const PROGRAM: &str = "leaf :: (n: int) -> int {
    return n + 1;
}
middle :: (n: int) -> int {
    a := leaf(n);
    b := leaf(a);
    return a + b;
}
other :: () -> int {
    return leaf(2) + middle(3);
}
main :: () {
    x := other();
    y := middle(x);
}
";

#[test]
fn prepares_the_procedure_at_a_declaration_or_a_use() {
    let s = open(PROGRAM);
    let at_decl = s
        .prepare_call_hierarchy(&uri(), at(PROGRAM, "middle ::", 2))
        .unwrap();
    assert_eq!(at_decl.len(), 1);
    assert_eq!(at_decl[0].name, "middle");
    assert_eq!(at_decl[0].detail, "(n: int) -> int");
    assert_eq!(at_decl[0].selection_range.start.line, 3);
    assert_eq!(at_decl[0].range.start.line, 3);
    assert_eq!(at_decl[0].range.end.line, 7);
    let at_use = s
        .prepare_call_hierarchy(&uri(), at(PROGRAM, "middle(x)", 2))
        .unwrap();
    assert_eq!(at_use, at_decl);
    // Not on a procedure.
    assert!(
        s.prepare_call_hierarchy(&uri(), at(PROGRAM, "x := other", 0))
            .unwrap()
            .is_empty()
    );
}

#[test]
fn lists_incoming_calls_with_their_sites() {
    let s = open(PROGRAM);
    let leaf = s
        .prepare_call_hierarchy(&uri(), at(PROGRAM, "leaf ::", 1))
        .unwrap();
    let calls = s.incoming_calls(&leaf[0]).unwrap();
    let summary: Vec<(&str, usize)> = calls
        .iter()
        .map(|c| (c.item.name.as_str(), c.from_ranges.len()))
        .collect();
    assert_eq!(summary, [("middle", 2), ("other", 1)]);
    assert_eq!(calls[0].from_ranges[0].start.line, 4);
    assert_eq!(calls[0].from_ranges[1].start.line, 5);
    // `main` is called by no one.
    let main = s
        .prepare_call_hierarchy(&uri(), at(PROGRAM, "main ::", 1))
        .unwrap();
    assert!(s.incoming_calls(&main[0]).unwrap().is_empty());
}

#[test]
fn lists_outgoing_calls_with_their_sites() {
    let s = open(PROGRAM);
    let other = s
        .prepare_call_hierarchy(&uri(), at(PROGRAM, "other ::", 1))
        .unwrap();
    let calls = s.outgoing_calls(&other[0]).unwrap();
    let summary: Vec<(&str, usize)> = calls
        .iter()
        .map(|c| (c.item.name.as_str(), c.from_ranges.len()))
        .collect();
    assert_eq!(summary, [("leaf", 1), ("middle", 1)]);
    let main = s
        .prepare_call_hierarchy(&uri(), at(PROGRAM, "main ::", 1))
        .unwrap();
    let names: Vec<_> = s
        .outgoing_calls(&main[0])
        .unwrap()
        .into_iter()
        .map(|c| c.item.name)
        .collect();
    assert_eq!(names, ["other", "middle"]);
    // A leaf calls nothing in this file.
    let leaf = s
        .prepare_call_hierarchy(&uri(), at(PROGRAM, "leaf ::", 1))
        .unwrap();
    assert!(s.outgoing_calls(&leaf[0]).unwrap().is_empty());
}

#[test]
fn outgoing_calls_reach_into_modules() {
    let text = "#import \"Basic\";
helper :: () { print(\"hi\\n\"); }
main :: () { helper(); }
";
    let s = open(text);
    let main = s
        .prepare_call_hierarchy(&uri(), at(text, "main ::", 1))
        .unwrap();
    let calls = s.outgoing_calls(&main[0]).unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].item.name, "helper");
    let helper = s.outgoing_calls(&calls[0].item).unwrap();
    assert_eq!(helper.len(), 1);
    assert_eq!(helper[0].item.name, "print");
    assert!(
        helper[0]
            .item
            .uri
            .ends_with("/stdlib/internal/basic/scalar.jai")
    );
}

#[test]
fn calls_of_nested_procedures_belong_to_them() {
    let text = "leaf :: () {}
outer :: () {
    inner :: () { leaf(); }
    inner();
}
";
    let s = open(text);
    let leaf = s
        .prepare_call_hierarchy(&uri(), at(text, "leaf ::", 1))
        .unwrap();
    let callers: Vec<_> = s
        .incoming_calls(&leaf[0])
        .unwrap()
        .into_iter()
        .map(|c| c.item.name)
        .collect();
    assert_eq!(callers, ["inner"]);
    let outer = s
        .prepare_call_hierarchy(&uri(), at(text, "outer ::", 1))
        .unwrap();
    let callees: Vec<_> = s
        .outgoing_calls(&outer[0])
        .unwrap()
        .into_iter()
        .map(|c| c.item.name)
        .collect();
    assert_eq!(callees, ["inner"]);
}

// -------------------------------------------------------------------------------------------
// The protocol
// -------------------------------------------------------------------------------------------

fn request(json: &mut JsonSession, id: u32, method: &str, params: Value) -> Value {
    let message = json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params});
    let out = json.handle_json(&message.to_string()).unwrap();
    let last: Value = serde_json::from_str(out.last().unwrap()).unwrap();
    last["result"].clone()
}

#[test]
fn the_server_advertises_and_answers_the_requests() {
    let mut json = JsonSession::with_environment(Limits::default(), environment());
    let init = request(&mut json, 1, "initialize", json!({}));
    assert_eq!(init["capabilities"]["callHierarchyProvider"], true);
    assert_eq!(init["capabilities"]["selectionRangeProvider"], true);
    let kinds = init["capabilities"]["codeActionProvider"]["codeActionKinds"]
        .as_array()
        .unwrap();
    for kind in ["refactor.extract", "refactor.inline", "refactor.rewrite"] {
        assert!(kinds.contains(&json!(kind)), "{kinds:?}");
    }
    json.handle_json(
        &json!({"jsonrpc": "2.0", "method": "textDocument/didOpen", "params": {"textDocument": {
            "uri": uri().as_str(), "languageId": "jai", "version": 1, "text": PROGRAM}}})
        .to_string(),
    )
    .unwrap();
    let items = request(
        &mut json,
        2,
        "textDocument/prepareCallHierarchy",
        json!({"textDocument": {"uri": uri().as_str()}, "position": at(PROGRAM, "leaf ::", 1)}),
    );
    assert_eq!(items[0]["name"], "leaf");
    assert_eq!(items[0]["kind"], 12);
    let incoming = request(
        &mut json,
        3,
        "callHierarchy/incomingCalls",
        json!({"item": items[0]}),
    );
    assert_eq!(incoming[0]["from"]["name"], "middle");
    assert_eq!(incoming[0]["fromRanges"].as_array().unwrap().len(), 2);
    let middle = request(
        &mut json,
        4,
        "textDocument/prepareCallHierarchy",
        json!({"textDocument": {"uri": uri().as_str()}, "position": at(PROGRAM, "middle ::", 1)}),
    );
    let outgoing = request(
        &mut json,
        5,
        "callHierarchy/outgoingCalls",
        json!({"item": middle[0]}),
    );
    assert_eq!(outgoing[0]["to"]["name"], "leaf");
    let nothing = request(
        &mut json,
        6,
        "textDocument/prepareCallHierarchy",
        json!({"textDocument": {"uri": uri().as_str()}, "position": at(PROGRAM, "x := other", 0)}),
    );
    assert!(nothing.is_null());
}

// -------------------------------------------------------------------------------------------
// Selection ranges
// -------------------------------------------------------------------------------------------

const NESTED: &str = "compute :: (a: int, b: int) -> int {
    total := (a + b) * 2;
    if total > 10 {
        total -= scale(a, 3);
    }
    return total;
}
";

/// The text of each range from the innermost out.
fn chain(text: &str, position: Position) -> Vec<String> {
    let s = open(text);
    let mut selection = Some(s.selection_ranges(&uri(), &[position]).unwrap().remove(0));
    let mut out = Vec::new();
    while let Some(sel) = selection {
        out.push(slice(text, sel.range));
        selection = sel.parent.map(|p| *p);
    }
    out
}

fn slice(text: &str, range: Range) -> String {
    let byte = |p: Position| -> usize {
        text.split_inclusive('\n')
            .take(p.line as usize)
            .map(str::len)
            .sum::<usize>()
            + p.character as usize
    };
    text[byte(range.start)..byte(range.end)].to_string()
}

#[test]
fn selection_grows_from_the_token_to_the_file() {
    let chain = chain(NESTED, at(NESTED, "a, 3", 0));
    assert_eq!(chain[0], "a");
    assert_eq!(chain[1], "scale(a, 3)");
    assert_eq!(chain[2], "total -= scale(a, 3);");
    // The block holds only that statement, so its text between braces is the same range.
    assert_eq!(chain[3], "{\n        total -= scale(a, 3);\n    }");
    assert!(chain[4].starts_with("if total > 10 {"), "{chain:?}");
    assert_eq!(chain.last().unwrap(), NESTED);
    // Every range holds the one before it.
    for pair in chain.windows(2) {
        assert!(pair[1].len() > pair[0].len(), "{chain:?}");
    }
}

#[test]
fn selection_steps_through_blocks_and_the_declaration() {
    let chain = chain(NESTED, at(NESTED, "total > 10", 0));
    assert_eq!(chain[0], "total");
    assert_eq!(chain[1], "total > 10");
    assert!(chain.iter().any(|c| c.starts_with("if total > 10 {")));
    // The body between the braces, without them, then with them.
    let body = chain
        .iter()
        .position(|c| c.starts_with("total := (a + b) * 2;"))
        .unwrap();
    assert!(chain[body].ends_with("return total;"));
    assert!(chain[body + 1].starts_with('{') && chain[body + 1].ends_with('}'));
    assert!(
        chain[body + 2..]
            .iter()
            .any(|c| c.starts_with("compute :: ("))
    );
}

#[test]
fn selection_includes_parenthesized_operands_and_string_text() {
    let chain_a = chain(NESTED, at(NESTED, "a + b", 0));
    assert_eq!(chain_a[0], "a");
    assert_eq!(chain_a[1], "a + b");
    assert!(chain_a.contains(&"(a + b) * 2".to_string()), "{chain_a:?}");
    let text = "main :: () {\n    s := \"hello world\";\n}\n";
    let c = chain(text, at(text, "world", 1));
    assert!(c.contains(&"\"hello world\"".to_string()), "{c:?}");
    assert!(c.contains(&"hello world".to_string()), "{c:?}");
}

#[test]
fn selection_works_without_a_parse() {
    let text = "main :: () {\n    x := ;\n";
    let c = chain(text, at(text, "x :=", 0));
    assert_eq!(c.first().unwrap(), "x");
    assert_eq!(c.last().unwrap(), text);
}

#[test]
fn selection_answers_every_position() {
    let s = open(NESTED);
    let positions = [at(NESTED, "compute", 0), at(NESTED, "return", 7)];
    let ranges = s.selection_ranges(&uri(), &positions).unwrap();
    assert_eq!(ranges.len(), 2);
    let mut json = JsonSession::with_environment(Limits::default(), environment());
    request(&mut json, 1, "initialize", json!({}));
    json.handle_json(
        &json!({"jsonrpc": "2.0", "method": "textDocument/didOpen", "params": {"textDocument": {
            "uri": uri().as_str(), "languageId": "jai", "version": 1, "text": NESTED}}})
        .to_string(),
    )
    .unwrap();
    let wire = request(
        &mut json,
        2,
        "textDocument/selectionRange",
        json!({"textDocument": {"uri": uri().as_str()}, "positions": positions}),
    );
    assert_eq!(wire.as_array().unwrap().len(), 2);
    assert!(wire[0]["range"]["start"].is_object());
    assert!(wire[0]["parent"]["range"].is_object());
}
