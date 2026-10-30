//! "Run test" and "Debug test" lenses over `@test` procedures.
use jai_language_server::{CodeLens, JsonSession, Limits};
use serde_json::{Value, json};
use std::path::Path;
use test_support::lsp::{at_nth, uri_of, virtual_session};

const TESTS: &str = r#"#import "Extensions/Testing";

helper :: () {}

adds :: () {
    expect(1 + 1 == 2);
} @test

holds :: (n: s64) {
    expect(n > 0);
} @property(20) @test

builds :: () {
} @compile_time @test

squares :: (n: s64) {
} @test @cases(s64.[1, 2])

timing :: (b: *Bench) {
} @bench

waits :: () {
} @test @slow
"#;

fn lenses(files: &[(&str, &str)], file: &str) -> Vec<CodeLens> {
    let mut s = virtual_session(files, &[], None);
    let uri = uri_of(Path::new(file));
    let text = files.iter().find(|(path, _)| *path == file).unwrap().1;
    s.open(uri.clone(), 1, text.into()).unwrap();
    s.code_lenses(&uri).unwrap()
}

const PROJECT: [(&str, &str); 2] = [
    ("/proj/jacket.toml", "[project]\n"),
    ("/proj/tests/strings.jai", TESTS),
];

#[test]
fn each_test_gets_a_run_and_a_debug_lens() {
    let found = lenses(&PROJECT, "/proj/tests/strings.jai");
    let summary: Vec<(&str, &str, &str)> = found
        .iter()
        .map(|l| {
            (
                l.command.title.as_str(),
                l.command.arguments[0]["name"].as_str().unwrap(),
                l.command.arguments[0]["kind"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        summary,
        [
            ("Run test", "adds", "unit"),
            ("Debug test", "adds", "unit"),
            ("Run test", "holds", "property"),
            ("Debug test", "holds", "property"),
            ("Run test", "builds", "compile_time"),
            ("Debug test", "builds", "compile_time"),
            ("Run test", "squares", "cases"),
            ("Debug test", "squares", "cases"),
            ("Run test", "waits", "unit"),
            ("Debug test", "waits", "unit"),
        ],
        "helpers and @bench get none"
    );
    assert_eq!(found[0].command.command, "jai.runTest");
    assert_eq!(found[1].command.command, "jai.debugTest");
    assert_eq!(found[0].range.start, at_nth(TESTS, "adds ::", 0, 0));
}

#[test]
fn the_argument_names_the_file_roots_and_the_selecting_filter() {
    let found = lenses(&PROJECT, "/proj/tests/strings.jai");
    let argument = &found[0].command.arguments[0];
    assert_eq!(argument["file"], "/proj/tests/strings.jai");
    assert_eq!(argument["projectRoot"], "/proj");
    assert_eq!(argument["testRoot"], "/proj");
    assert_eq!(
        argument["filter"],
        json!(["-E", "test(=adds) and file(tests/strings.jai)"])
    );
}

#[test]
fn a_path_a_filter_expression_cannot_hold_selects_by_name() {
    let files = [
        ("/proj/jacket.toml", ""),
        ("/proj/tests/a(b).jai", "f :: () {} @test"),
    ];
    let found = lenses(&files, "/proj/tests/a(b).jai");
    assert_eq!(
        found[0].command.arguments[0]["filter"],
        json!(["--exact", "f"])
    );
}

#[test]
fn a_module_inside_the_project_runs_from_the_project_with_its_own_test_root() {
    let files = [
        ("/proj/jacket.toml", ""),
        ("/proj/lib/Sort/tests/order.jai", "sorted :: () {} @test"),
    ];
    let found = lenses(&files, "/proj/lib/Sort/tests/order.jai");
    let argument = &found[0].command.arguments[0];
    assert_eq!(argument["projectRoot"], "/proj");
    assert_eq!(argument["testRoot"], "/proj/lib/Sort");
    assert_eq!(
        argument["filter"],
        json!(["-E", "test(=sorted) and file(tests/order.jai)"])
    );
}

#[test]
fn a_file_without_tests_has_no_lenses() {
    let files = [("/proj/main.jai", "main :: () {}\n")];
    assert!(lenses(&files, "/proj/main.jai").is_empty());
}

#[test]
fn the_setting_turns_the_lenses_off_and_asks_the_client_to_fetch_again() {
    let mut server = JsonSession::new(Limits::default());
    let mut send = |message: Value| -> Vec<Value> {
        server
            .handle_json(&message.to_string())
            .unwrap()
            .iter()
            .map(|m| serde_json::from_str(m).unwrap())
            .collect()
    };
    send(json!({
        "jsonrpc": "2.0", "id": 1, "method": "initialize",
        "params": { "capabilities": { "workspace": { "codeLens": { "refreshSupport": true } } } },
    }));
    send(json!({
        "jsonrpc": "2.0", "method": "textDocument/didOpen",
        "params": { "textDocument": {
            "uri": "file:///proj/tests/a.jai", "languageId": "jai", "version": 1,
            "text": "f :: () {} @test",
        } },
    }));
    let ask = |send: &mut dyn FnMut(Value) -> Vec<Value>, id: u64| {
        let reply = send(json!({
            "jsonrpc": "2.0", "id": id, "method": "textDocument/codeLens",
            "params": { "textDocument": { "uri": "file:///proj/tests/a.jai" } },
        }));
        reply.last().unwrap()["result"].as_array().unwrap().len()
    };
    assert_eq!(ask(&mut send, 2), 2);
    let changed = send(json!({
        "jsonrpc": "2.0", "method": "workspace/didChangeConfiguration",
        "params": { "settings": { "jai": { "tests": { "codeLens": false } } } },
    }));
    assert!(
        changed
            .iter()
            .any(|m| m["method"] == "workspace/codeLens/refresh"),
        "{changed:?}"
    );
    assert_eq!(ask(&mut send, 3), 0);
}
