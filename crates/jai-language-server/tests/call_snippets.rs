//! Completion that inserts call parentheses and statement skeletons as snippets.
use jai_language_server::{
    CallSnippets, CompletionItem, DocumentUri, Environment, JsonSession, Limits, Position, Range,
    Session,
};
use serde_json::{Value, json};
mod common;
use common::{lean_environment, position};

fn session(snippets: bool, mode: CallSnippets) -> Session {
    let mut s = Session::with_environment(Limits::default(), lean_environment());
    s.set_snippet_support(snippets);
    s.set_call_snippets(mode);
    s
}

const URI: &str = "file:///call-snippets/main.jai";

/// The completion item `label` where `|` marks the cursor.
fn item(s: &mut Session, source: &str, label: &str) -> Option<CompletionItem> {
    let byte = source.find('|').unwrap();
    let text = source.replacen('|', "", 1);
    let uri = DocumentUri::parse(URI).unwrap();
    let _ = s.close(&uri);
    s.open(uri.clone(), 1, text.clone()).unwrap();
    let at = position(&text, byte);
    s.completion(&uri, at)
        .unwrap()
        .items
        .into_iter()
        .find(|i| i.label == label)
}

fn inserted(source: &str, label: &str) -> Option<String> {
    let mut s = session(true, CallSnippets::AddParentheses);
    item(&mut s, source, label).and_then(|i| i.insert_text)
}

const PROCEDURES: &str = "add :: (a: int, b: int) -> int { return a + b; }\n\
    now :: () -> int { return 1; }\n\
    greet :: (name: string, loud := false, rest: ..Any) {}\n\
    twice :: () {}\ntwice :: (n: int) {}\n\
    none_a :: () {}\nnone_a :: (flag: bool) {}\n\
    zero :: () {}\nzero :: () -> int { return 1; }\n";

fn body(statement: &str) -> String {
    format!("{PROCEDURES}main :: () {{\n    {statement}\n}}\n")
}

#[test]
fn a_procedure_with_parameters_puts_the_cursor_inside_the_parentheses() {
    let mut s = session(true, CallSnippets::AddParentheses);
    let add = item(&mut s, &body("x := ad|"), "add").unwrap();
    assert_eq!(add.insert_text.as_deref(), Some("add($0)"));
    assert!(add.snippet);
    assert!(add.trigger_signature_help);
}

#[test]
fn a_parameterless_procedure_gets_empty_parentheses() {
    let mut s = session(true, CallSnippets::AddParentheses);
    let now = item(&mut s, &body("x := no|"), "now").unwrap();
    assert_eq!(now.insert_text.as_deref(), Some("now()$0"));
    assert!(!now.trigger_signature_help);
}

#[test]
fn overloads_are_empty_only_when_every_overload_is() {
    assert_eq!(inserted(&body("zer|"), "zero").as_deref(), Some("zero()$0"));
    assert_eq!(
        inserted(&body("twic|"), "twice").as_deref(),
        Some("twice($0)")
    );
}

#[test]
fn fill_arguments_adds_a_placeholder_for_each_required_parameter() {
    let mut s = session(true, CallSnippets::FillArguments);
    let add = item(&mut s, &body("x := ad|"), "add").unwrap();
    assert_eq!(add.insert_text.as_deref(), Some("add(${1:a}, ${2:b})$0"));
    // Parameters with a default or `..` varargs are left for the caller.
    let greet = item(&mut s, &body("gre|"), "greet").unwrap();
    assert_eq!(greet.insert_text.as_deref(), Some("greet(${1:name})$0"));
    // Overloads cannot name their arguments.
    let twice = item(&mut s, &body("twic|"), "twice").unwrap();
    assert_eq!(twice.insert_text.as_deref(), Some("twice($0)"));
}

#[test]
fn the_setting_none_and_a_client_without_snippets_leave_the_name_alone() {
    let mut s = session(true, CallSnippets::None);
    assert_eq!(
        item(&mut s, &body("x := ad|"), "add").unwrap().insert_text,
        None
    );
    let mut s = session(false, CallSnippets::AddParentheses);
    let add = item(&mut s, &body("x := ad|"), "add").unwrap();
    assert_eq!((add.insert_text, add.snippet), (None, false));
}

#[test]
fn no_parentheses_where_the_procedure_is_a_value() {
    let negatives = [
        // Already called.
        ("ad|(1, 2);", "add"),
        ("ad| (1, 2);", "add"),
        // Naming it, not calling it.
        ("t := type_of(ad|);", "add"),
        ("t := type_info(ad|);", "add"),
        ("f := *ad|;", "add"),
        ("g :: ad|;", "add"),
        // Being declared.
        ("ad| :: 3;", "add"),
        ("ad| := 3;", "add"),
        // A type is written here.
        ("v: ad|;", "add"),
        ("f :: () -> ad|;", "add"),
    ];
    for (statement, label) in negatives {
        let text = body(statement);
        assert_eq!(inserted(&text, label), None, "{statement}");
    }
    // A global initialiser cannot call at run time.
    let global = format!("{PROCEDURES}handler := ad|;\nmain :: () {{}}\n");
    assert_eq!(inserted(&global, "add"), None);
    let directive = body("#procedure_of_call ad|(1, 2);");
    assert_eq!(inserted(&directive, "add"), None);
}

#[test]
fn multiplication_is_not_an_address_of() {
    assert_eq!(
        inserted(&body("x := 2 * no|"), "now").as_deref(),
        Some("now()$0")
    );
    assert_eq!(
        inserted(&body("return ad|"), "add").as_deref(),
        Some("add($0)")
    );
}

#[test]
fn a_local_initialiser_still_calls() {
    assert_eq!(
        inserted(&body("x := no|"), "now").as_deref(),
        Some("now()$0")
    );
    assert_eq!(inserted(&body("ad|"), "add").as_deref(), Some("add($0)"));
}

#[test]
fn builtins_that_take_arguments_get_parentheses() {
    assert_eq!(
        inserted(&body("x := size_|"), "size_of").as_deref(),
        Some("size_of($0)")
    );
    // Inside another naming call the result is a type.
    assert_eq!(inserted(&body("x := type_of(size_|)"), "size_of"), None);
}

#[test]
fn named_arguments_and_paths_are_not_calls() {
    let source = "scale :: (factor: float, offset := 0.0) -> float { return factor; }\n\
        main :: () {\n    x := scale(fac|);\n}\n";
    let mut s = session(true, CallSnippets::AddParentheses);
    let named = item(&mut s, source, "factor").unwrap();
    assert_eq!(named.insert_text.as_deref(), Some("factor = "));
    assert!(!named.trigger_signature_help);
    let import = "#import \"Bas|\";\n";
    let mut s = session(true, CallSnippets::AddParentheses);
    assert!(item(&mut s, import, "Basic").is_none_or(|i| !i.snippet));
}

#[test]
fn statement_keywords_expand_to_a_skeleton() {
    let mut s = session(true, CallSnippets::AddParentheses);
    let for_loop = item(&mut s, &body("fo|"), "for").unwrap();
    assert_eq!(
        for_loop.insert_text.as_deref(),
        Some("for ${1:iterable} {\n    $0\n}")
    );
    let while_loop = item(&mut s, &body("whi|"), "while").unwrap();
    assert_eq!(
        while_loop.insert_text.as_deref(),
        Some("while ${1:condition} {\n    $0\n}")
    );
    let branch = item(&mut s, &body("i|"), "if").unwrap();
    assert!(branch.snippet);
    // In an expression the keyword stays a word, and so does one in front of existing code.
    assert_eq!(
        item(&mut s, &body("x := i|"), "if").unwrap().insert_text,
        None
    );
    assert_eq!(
        item(&mut s, &body("i| x == 1 {}"), "if")
            .unwrap()
            .insert_text,
        None
    );
    // Snippets are for clients that ask for them.
    let mut plain = session(false, CallSnippets::AddParentheses);
    assert_eq!(
        item(&mut plain, &body("fo|"), "for").unwrap().insert_text,
        None
    );
}

#[test]
fn case_expands_only_inside_the_braces_of_an_if_on_a_value() {
    let source = "main :: () {\n    x := 1;\n    if x == {\n        ca|\n    }\n}\n";
    let mut s = session(true, CallSnippets::AddParentheses);
    let case = item(&mut s, source, "case").unwrap();
    assert_eq!(case.insert_text.as_deref(), Some("case ${1:value};\n$0"));
    let elsewhere = item(&mut s, &body("ca|"), "case").unwrap();
    assert_eq!(elsewhere.insert_text, None);
}

fn request(session: &mut JsonSession, id: u64, method: &str, params: Value) -> Value {
    let message = json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params });
    let out = session.handle_json(&message.to_string()).unwrap();
    out.iter()
        .map(|m| serde_json::from_str::<Value>(m).unwrap())
        .find(|m| m["id"] == json!(id))
        .unwrap()
}

fn notify(session: &mut JsonSession, method: &str, params: Value) {
    let message = json!({ "jsonrpc": "2.0", "method": method, "params": params });
    session.handle_json(&message.to_string()).unwrap();
}

fn complete(session: &mut JsonSession, id: u64, text: &str, line: u32, character: u32) -> Value {
    let uri = "file:///call-snippets/wire.jai";
    notify(
        session,
        "textDocument/didOpen",
        json!({ "textDocument": { "uri": uri, "languageId": "jai", "version": id, "text": text } }),
    );
    request(
        session,
        id,
        "textDocument/completion",
        json!({ "textDocument": { "uri": uri }, "position": { "line": line, "character": character } }),
    )
}

fn find<'a>(reply: &'a Value, label: &str) -> &'a Value {
    reply["result"]["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["label"] == label)
        .unwrap_or_else(|| panic!("no {label} in {reply}"))
}

#[test]
fn the_wire_item_is_a_snippet_that_asks_for_parameter_hints() {
    let text = "add :: (a: int, b: int) -> int { return a + b; }\nmain :: () {\n    ad\n}\n";
    let mut s = JsonSession::with_environment(Limits::default(), lean_environment());
    request(
        &mut s,
        1,
        "initialize",
        json!({ "capabilities": { "textDocument": { "completion": {
            "completionItem": { "snippetSupport": true } } } } }),
    );
    notify(&mut s, "initialized", json!({}));
    let reply = complete(&mut s, 2, text, 2, 6);
    let add = find(&reply, "add");
    assert_eq!(add["textEdit"]["newText"], "add($0)");
    assert_eq!(add["insertTextFormat"], 2);
    assert_eq!(
        add["command"]["command"],
        "editor.action.triggerParameterHints"
    );
    // The setting arrives with `workspace/didChangeConfiguration`.
    notify(
        &mut s,
        "workspace/didChangeConfiguration",
        json!({ "settings": { "jai": { "completion": { "callSnippets": "fillArguments" } } } }),
    );
    let reply = complete(&mut s, 3, text, 2, 6);
    assert_eq!(
        find(&reply, "add")["textEdit"]["newText"],
        "add(${1:a}, ${2:b})$0"
    );
    notify(
        &mut s,
        "workspace/didChangeConfiguration",
        json!({ "settings": { "jai": { "completion": { "callSnippets": "none" } } } }),
    );
    let reply = complete(&mut s, 4, text, 2, 6);
    let add = find(&reply, "add");
    assert_eq!(add["textEdit"]["newText"], "add");
    assert!(add.get("command").is_none());
}

#[test]
fn a_client_without_snippet_support_gets_plain_text() {
    let text = "add :: (a: int) {}\nmain :: () {\n    ad\n}\n";
    let mut s = JsonSession::with_environment(Limits::default(), lean_environment());
    request(&mut s, 1, "initialize", json!({ "capabilities": {} }));
    notify(&mut s, "initialized", json!({}));
    let reply = complete(&mut s, 2, text, 2, 6);
    let add = find(&reply, "add");
    assert_eq!(add["textEdit"]["newText"], "add");
    assert!(add.get("insertTextFormat").is_none());
}

#[test]
fn an_auto_import_is_inserted_as_a_call_too() {
    let stdlib = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let mut s = Session::with_environment(
        Limits::default(),
        Environment {
            fs: std::rc::Rc::new(jaic::sema::NativeFs),
            options: Box::new(move |_| {
                let mut o = jaic::sema::Options::host();
                o.import_paths = vec![stdlib.clone()];
                o.preload = Some(stdlib.join(jaic::PRELOAD_ENTRY));
                o
            }),
        },
    );
    s.set_snippet_support(true);
    let print = item(&mut s, "main :: () {\n    prin|\n}\n", "print").unwrap();
    assert!(!print.additional_edits.is_empty(), "{print:?}");
    assert_eq!(print.insert_text.as_deref(), Some("print($0)"));
}

fn labels(s: &mut Session, source: &str, prefix: &str) -> Vec<String> {
    let byte = source.find('|').unwrap();
    let text = source.replacen('|', "", 1);
    let uri = DocumentUri::parse(URI).unwrap();
    let _ = s.close(&uri);
    s.open(uri.clone(), 1, text.clone()).unwrap();
    let at = position(&text, byte);
    s.completion(&uri, at)
        .unwrap()
        .items
        .into_iter()
        .filter(|i| i.label.starts_with(prefix) && i.insert_text.is_some())
        .map(|i| i.label)
        .collect()
}

#[test]
fn postfix_templates_replace_the_receiver_and_the_word() {
    let mut s = session(true, CallSnippets::AddParentheses);
    let source = "main :: () {\n    flag := true;\n    flag.i|\n}\n";
    let item = item(&mut s, source, "if").unwrap();
    assert_eq!(item.insert_text.as_deref(), Some("if flag {\n    $0\n}"));
    assert_eq!(item.filter_text.as_deref(), Some("flag.if"));
    let range = item.replace.unwrap();
    assert_eq!(
        (range.start.line, range.start.character, range.end.character),
        (2, 4, 10)
    );
}

#[test]
fn statement_postfixes_stay_out_of_expressions() {
    let mut s = session(true, CallSnippets::AddParentheses);
    let statement = "main :: () {\n    flag := true;\n    flag.|\n}\n";
    let all = labels(&mut s, statement, "");
    for want in ["if", "while", "for", "return", "not", "ref"] {
        assert!(all.iter().any(|l| l == want), "{want} in {all:?}");
    }
    let expression = "main :: () {\n    flag := true;\n    other := flag.|\n}\n";
    assert_eq!(labels(&mut s, expression, ""), ["not", "ref"]);
}

#[test]
fn postfix_templates_need_a_value_receiver_and_a_snippet_client() {
    let mut s = session(true, CallSnippets::AddParentheses);
    // A type's members are its constants, not something to wrap.
    let ty = "Vec :: struct { x: int; }\nmain :: () {\n    Vec.|\n}\n";
    assert!(labels(&mut s, ty, "").is_empty());
    // A call's result is a value.
    let call = "make :: () -> bool { return true; }\nmain :: () {\n    make().|\n}\n";
    assert!(labels(&mut s, call, "").contains(&"if".to_owned()));
    let mut plain = session(false, CallSnippets::AddParentheses);
    let source = "main :: () {\n    flag := true;\n    flag.|\n}\n";
    assert!(labels(&mut plain, source, "").is_empty());
}

fn long_procedure() -> String {
    let body = "    x := 1;\n".repeat(30);
    format!("big :: () {{\n{body}}}\n\nshort :: () {{\n    y := 2;\n}}\n")
}

fn closing_labels(s: &Session, uri: &DocumentUri, text: &str) -> Vec<(u32, String)> {
    let end = Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: text.lines().count() as u32,
            character: 0,
        },
    };
    s.inlay_hints(uri, end)
        .unwrap()
        .into_iter()
        .filter(|h| h.label.starts_with("// "))
        .map(|h| (h.position.line, h.label))
        .collect()
}

#[test]
fn a_long_procedure_names_its_closing_brace() {
    let mut s = session(false, CallSnippets::AddParentheses);
    let uri = DocumentUri::parse(URI).unwrap();
    let text = long_procedure();
    s.open(uri.clone(), 1, text.clone()).unwrap();
    assert_eq!(closing_labels(&s, &uri, &text), [(31, "// big".to_owned())]);
    // The threshold is a setting, and 0 turns the hints off.
    s.set_closing_brace_min_lines(3);
    let labels = closing_labels(&s, &uri, &text);
    assert_eq!(labels.len(), 2, "{labels:?}");
    s.set_closing_brace_min_lines(0);
    assert!(closing_labels(&s, &uri, &text).is_empty());
}

#[test]
fn the_closing_brace_setting_arrives_over_the_protocol() {
    let mut s = JsonSession::with_environment(Limits::default(), lean_environment());
    request(
        &mut s,
        1,
        "initialize",
        json!({ "initializationOptions": { "inlayHints": { "closingBraceMinLines": 0 } } }),
    );
    notify(&mut s, "initialized", json!({}));
    let uri = "file:///call-snippets/closing.jai";
    notify(
        &mut s,
        "textDocument/didOpen",
        json!({ "textDocument": { "uri": uri, "languageId": "jai", "version": 1, "text": long_procedure() } }),
    );
    let hints = |s: &mut JsonSession, id| {
        request(
            s,
            id,
            "textDocument/inlayHint",
            json!({ "textDocument": { "uri": uri }, "range": {
                "start": { "line": 0, "character": 0 }, "end": { "line": 36, "character": 0 } } }),
        )
    };
    let off = hints(&mut s, 2);
    assert!(
        off["result"].is_array() && !off.to_string().contains("// big"),
        "{off}"
    );
    notify(
        &mut s,
        "workspace/didChangeConfiguration",
        json!({ "settings": { "jai": { "inlayHints": { "closingBraceMinLines": 25 } } } }),
    );
    let third = hints(&mut s, 3).to_string();
    assert!(third.contains("// big"), "{third}");
}
