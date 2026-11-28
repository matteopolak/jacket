//! Code a build metaprogram generates (`add_build_string`) has no file: go to definition lands
//! in a `jai-generated:` document the server serves through `jai/source`.
use test_support::lsp::{at, open_file, project};

const BUILD: &str = "#import \"Basic\";\n#import \"Compiler\";\n#run build();\nbuild :: () {\n    \
    set_build_options_dc(.{do_output = false});\n    w := compiler_create_workspace(\"app\");\n    \
    options := get_build_options(w);\n    options.output_type = .NO_OUTPUT;\n    \
    set_build_options(options, w);\n    add_build_string(\"made_here :: () -> int { return 7; }\\n\", w);\n    \
    add_build_file(\"main.jai\", w);\n}\n";

const MAIN: &str =
    "#import \"Basic\";\nmain :: () {\n    value := made_here();\n    print(\"%\\n\", value);\n}\n";

#[test]
fn definition_into_a_generated_string_points_at_a_served_document() {
    let dir = project(
        "generated-code",
        &[
            ("jacket.toml", "[project]\nentry = [\"build.jai\"]\n"),
            ("build.jai", BUILD),
            ("main.jai", MAIN),
        ],
    );
    let (s, uri, text) = open_file(&dir, "main.jai");
    let found = s.definition(&uri, at(&text, "made_here()")).unwrap();
    let generated = found
        .iter()
        .find(|l| l.uri.as_str().starts_with("jai-generated:"))
        .unwrap_or_else(|| panic!("{found:?}"));
    let served = s.generated_source(generated.uri.as_str()).expect("served");

    assert!(served.contains("made_here :: () -> int"), "{served}");
    assert_eq!(generated.range.start.line, 0);
}

#[test]
fn an_error_that_names_generated_code_points_into_the_served_document() {
    let dir = project(
        "generated-note",
        &[
            ("jacket.toml", "[project]\nentry = [\"build.jai\"]\n"),
            ("build.jai", BUILD),
            (
                "main.jai",
                "#import \"Basic\";\nmain :: () {\n    value := made_here(1, 2);\n    print(\"%\\n\", value);\n}\n",
            ),
        ],
    );
    let (s, uri, _) = open_file(&dir, "main.jai");
    let diagnostics = s.diagnostics(&uri).unwrap();
    let related: Vec<_> = diagnostics
        .iter()
        .flat_map(|d| d.related.iter())
        .filter(|r| r.uri.starts_with("jai-generated:"))
        .collect();

    assert!(!related.is_empty(), "{diagnostics:?}");
    assert!(s.generated_source(&related[0].uri).is_some());
}

fn erroring_project() -> test_support::TempDir {
    let build = BUILD.replace(
        "made_here :: () -> int { return 7; }",
        "made_here :: () -> int { return \\\"seven\\\"; }",
    );

    project(
        "generated-own-error",
        &[
            ("jacket.toml", "[project]\nentry = [\"build.jai\"]\n"),
            ("build.jai", &build),
            ("main.jai", MAIN),
        ],
    )
}

#[test]
fn an_error_inside_generated_code_is_published_on_its_document() {
    let dir = erroring_project();
    let (s, main, text) = open_file(&dir, "main.jai");
    let generated = s
        .definition(&main, at(&text, "made_here()"))
        .unwrap()
        .into_iter()
        .find(|l| l.uri.starts_with("jai-generated:"))
        .expect("a generated location")
        .uri;
    let diagnostics = s.generated_diagnostics(&generated).expect("served");

    assert!(
        diagnostics
            .iter()
            .any(|d| d.message.contains("type mismatch") && d.range.start.line == 0),
        "{diagnostics:?}"
    );
}

/// A client that opened the generated document pulls its diagnostics like any other's.
#[test]
fn the_pull_for_a_generated_document_answers_with_its_errors() {
    use jai_language_server::{JsonSession, Limits};
    use serde_json::{Value, json};
    use test_support::lsp::{environment, uri_of};

    let dir = erroring_project();
    let mut json = JsonSession::with_environment(Limits::default(), environment());
    let mut send = |message: Value| -> Vec<Value> {
        json.handle_json(&message.to_string())
            .unwrap()
            .iter()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    };
    let main = uri_of(&dir.join("main.jai"));

    send(
        json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
        "capabilities": {"textDocument": {"diagnostic": {}}}, "workspaceFolders": [{"uri": uri_of(dir.path()).as_str(), "name": "w"}]}}),
    );
    send(json!({"jsonrpc": "2.0", "method": "initialized", "params": {}}));
    send(
        json!({"jsonrpc": "2.0", "method": "textDocument/didOpen", "params": {"textDocument": {
        "uri": main.as_str(), "languageId": "jai", "version": 1, "text": MAIN}}}),
    );
    let found = send(
        json!({"jsonrpc": "2.0", "id": 2, "method": "textDocument/definition", "params": {
        "textDocument": {"uri": main.as_str()}, "position": {"line": 2, "character": 14}}}),
    );
    let generated = found
        .iter()
        .find_map(|m| m["result"].as_array())
        .and_then(|locations| {
            locations.iter().find_map(|l| {
                l["uri"]
                    .as_str()
                    .filter(|u| u.starts_with("jai-generated:"))
            })
        })
        .expect("a generated location")
        .to_owned();

    // The client opens the document it was sent, and closes it again.
    send(
        json!({"jsonrpc": "2.0", "method": "textDocument/didOpen", "params": {"textDocument": {
        "uri": generated, "languageId": "jai", "version": 1, "text": "made_here :: () -> int { return \"seven\"; }\n"}}}),
    );
    let report = send(
        json!({"jsonrpc": "2.0", "id": 3, "method": "textDocument/diagnostic", "params": {
        "textDocument": {"uri": generated}}}),
    );
    let reply = report
        .iter()
        .find(|m| m["id"] == 3)
        .unwrap_or_else(|| panic!("no reply: {report:?}"));

    assert!(reply.to_string().contains("type mismatch"), "{reply}");
    send(
        json!({"jsonrpc": "2.0", "method": "textDocument/didClose", "params": {"textDocument": {
        "uri": generated}}}),
    );
}
