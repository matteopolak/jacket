//! Code actions, fix-all, import fixes and lint configuration.
use super::{TWO_LINTS, at_nth, everything, file_uri, notify, opened, request, session, uri};
use jai_language_server::{DiagnosticCode, DiagnosticSeverity, JsonSession, Position, Range};

pub(super) fn code_actions(
    json: &mut JsonSession,
    range: Range,
    context: serde_json::Value,
) -> Vec<serde_json::Value> {
    request(
        json,
        "textDocument/codeAction",
        serde_json::json!({
            "textDocument": {"uri": uri().as_str()}, "range": range, "context": context}),
    )
    .as_array()
    .unwrap()
    .clone()
}

fn top() -> Range {
    let start = Position {
        line: 0,
        character: 0,
    };
    Range {
        start,
        end: start,
    }
}

#[test]
fn fix_all_merges_every_safe_fix() {
    let mut json = opened(TWO_LINTS);
    let all = code_actions(
        &mut json,
        top(),
        serde_json::json!({"diagnostics": [], "only": ["source.fixAll"]}),
    );
    assert_eq!(all.len(), 1, "{all:?}");
    assert_eq!(all[0]["kind"], "source.fixAll.jailint");
    assert_eq!(all[0]["title"], "Fix 2 lint problems");
    let edits = all[0]["edit"]["changes"][uri().as_str()]
        .as_array()
        .unwrap();
    let texts: Vec<&str> = edits
        .iter()
        .map(|e| e["newText"].as_str().unwrap())
        .collect();
    assert_eq!(texts, ["a", "!b"]);
    assert_eq!(all[0]["diagnostics"].as_array().unwrap().len(), 2);
    // `only: quickfix` leaves the source action out, and no lint touches the range.
    let quick = code_actions(
        &mut json,
        top(),
        serde_json::json!({"diagnostics": [], "only": ["quickfix"]}),
    );
    assert!(quick.is_empty(), "{quick:?}");
}

#[test]
fn quick_fixes_follow_the_diagnostics_they_are_asked_for() {
    let mut json = opened(TWO_LINTS);
    let lint = Range {
        start: at_nth(TWO_LINTS, "b == false", 0, 0),
        end: at_nth(TWO_LINTS, "b == false", 0, "b == false".len()),
    };
    let diagnostic = serde_json::json!({
        "range": lint, "code": "bool_comparison", "source": "jailint", "message": ""});
    let found = code_actions(
        &mut json,
        top(),
        serde_json::json!({"diagnostics": [diagnostic], "only": ["quickfix"]}),
    );
    assert_eq!(found.len(), 1, "{found:?}");
    let fix = &found[0];
    assert_eq!(fix["kind"], "quickfix");
    assert_eq!(fix["title"], "Write `!b`");
    assert_eq!(fix["isPreferred"], true);
    assert_eq!(fix["data"]["rule"], "bool_comparison");
    assert_eq!(fix["diagnostics"][0]["code"], "bool_comparison");
    assert_eq!(fix["diagnostics"][0]["range"], serde_json::json!(lint));
}

#[test]
fn an_open_jailint_toml_configures_the_lints() {
    let mut json = opened(TWO_LINTS);
    let lint_count = |messages: &[serde_json::Value]| {
        messages
            .iter()
            .find(|m| {
                m["method"] == "textDocument/publishDiagnostics"
                    && m["params"]["uri"] == uri().as_str()
            })
            .map(|m| {
                m["params"]["diagnostics"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|d| d["source"] == "jailint")
                    .count()
            })
            .unwrap()
    };
    let settings = "file:///lsp-test/jacket.toml";
    let published = notify(
        &mut json,
        "textDocument/didOpen",
        serde_json::json!({"textDocument": {"uri": settings, "languageId": "toml",
            "version": 1, "text": "[lint.rules]\nbool_comparison = \"allow\"\n"}}),
    );
    assert_eq!(lint_count(&published), 0);
    // The settings file is not Jai: nothing is published for it.
    assert!(
        published.iter().all(|m| m["params"]["uri"] != settings),
        "{published:?}"
    );
    let published = notify(
        &mut json,
        "textDocument/didChange",
        serde_json::json!({"textDocument": {"uri": settings, "version": 2},
            "contentChanges": [{"text": "[lint.rules]\nbool_comparison = \"deny\"\n"}]}),
    );
    assert!(published.is_empty(), "typing must not apply settings");
    let published = notify(
        &mut json,
        "textDocument/didSave",
        serde_json::json!({"textDocument":{"uri":settings}}),
    );
    assert_eq!(lint_count(&published), 2);
    let published = notify(
        &mut json,
        "textDocument/didClose",
        serde_json::json!({"textDocument": {"uri": settings}}),
    );
    assert_eq!(lint_count(&published), 2);
}

/// The `quickfix` actions at `marker` in `text` (open as `uri()`).
fn quick_fixes_at(text: &str, marker: &str) -> Vec<jai_language_server::CodeAction> {
    let mut s = session();
    s.open(uri(), 1, text.into()).unwrap();
    let cursor = at_nth(text, marker, 0, 0);
    s.code_actions(
        &uri(),
        Range {
            start: cursor,
            end: cursor,
        },
    )
    .unwrap()
    .into_iter()
    .filter(|a| a.kind == Some("quickfix"))
    .collect()
}

/// The single edit of `action` as (start, new text).
fn single_edit(action: &jai_language_server::CodeAction) -> (Position, &str) {
    let (target, edits) = action.edit.as_ref().unwrap();
    assert_eq!(target, uri().as_str());
    assert_eq!(edits.len(), 1, "{edits:?}");
    assert_eq!(edits[0].range.start, edits[0].range.end);
    (edits[0].range.start, edits[0].new_text.as_str())
}

#[test]
fn an_unknown_name_offers_the_import_that_declares_it() {
    let text = "// A program.\n\nmain :: () {\n    print(\"hi\\n\");\n}\n";
    let actions = quick_fixes_at(text, "print");
    assert_eq!(actions.len(), 1, "{actions:?}");
    assert_eq!(actions[0].title, "Add `#import \"Basic\";`");
    assert!(actions[0].is_preferred);
    assert_eq!(actions[0].diagnostics.len(), 1);
    assert_eq!(actions[0].diagnostics[0].code, DiagnosticCode::Check);
    // Below the leading comment, with a blank line before the code.
    assert_eq!(
        single_edit(&actions[0]),
        (at_nth(text, "main", 0, 0), "#import \"Basic\";\n\n")
    );
    // Away from the error, nothing.
    assert!(quick_fixes_at(text, "main").is_empty());
}

#[test]
fn an_unknown_extension_name_offers_its_extensions_import() {
    let text = "main :: () {\n    x: Long_Double;\n}\n";
    let actions = quick_fixes_at(text, "Long_Double");
    assert_eq!(actions.len(), 1, "{actions:?}");
    assert_eq!(
        actions[0].title,
        "Add `#import \"Extensions/Long_Double\";`"
    );
}

#[test]
fn an_import_goes_after_the_existing_imports() {
    let text = "#import \"Math\";\n#load \"other.jai\";\n\nmain :: () {\n    print(\"%\\n\", sqrt(2.0));\n}\n";
    let mut s = session();
    let other = file_uri("/lsp-test/other.jai");
    s.open(other, 1, "helper :: () {}\n".into()).unwrap();
    s.open(uri(), 1, text.into()).unwrap();
    let cursor = at_nth(text, "print", 0, 0);
    let actions = s
        .code_actions(
            &uri(),
            Range {
                start: cursor,
                end: cursor,
            },
        )
        .unwrap();
    assert_eq!(actions.len(), 1, "{actions:?}");
    assert_eq!(
        single_edit(&actions[0]),
        (at_nth(text, "#load", 0, 0), "#import \"Basic\";\n")
    );
}

#[test]
fn a_name_several_modules_declare_offers_each_without_preferring_one() {
    let text = "main :: () {\n    x := log(2.0);\n}\n";
    let actions = quick_fixes_at(text, "log");
    let titles: Vec<&str> = actions.iter().map(|a| a.title.as_str()).collect();
    assert_eq!(
        titles,
        ["Add `#import \"Basic\";`", "Add `#import \"Math\";`"],
        "{actions:?}"
    );
    assert!(actions.iter().all(|a| !a.is_preferred));
}

#[test]
fn a_module_used_as_a_namespace_is_imported_under_its_name() {
    let text = "main :: () {\n    x := Math.sqrt(2.0);\n}\n";
    let actions = quick_fixes_at(text, "Math");
    // Codex also publicly reexports Math in the reviewed replacement library.
    assert_eq!(actions.len(), 2, "{actions:?}");
    assert_eq!(actions[0].title, "Add `Math :: #import \"Math\";`");
    assert_eq!(single_edit(&actions[0]).1, "Math :: #import \"Math\";\n\n");
    assert_eq!(actions[1].title, "Add `#import \"Codex\";`");
    assert_eq!(single_edit(&actions[1]).1, "#import \"Codex\";\n\n");
    for action in &actions {
        let (start, insertion) = single_edit(action);
        assert_eq!(
            start,
            Position {
                line: 0,
                character: 0
            }
        );
        let mut session = session();
        session
            .open(uri(), 1, format!("{insertion}{text}"))
            .unwrap();
        let diagnostics = session.diagnostics(&uri()).unwrap();
        assert!(
            !diagnostics
                .iter()
                .any(|d| d.severity == DiagnosticSeverity::Error),
            "{action:?}: {diagnostics:?}"
        );
    }
}

#[test]
fn a_loaded_file_gets_the_import_itself() {
    let main = "#load \"helper.jai\";\nmain :: () { helper(); }\n";
    let helper_text = "// Helpers.\nhelper :: () {\n    print(\"x\\n\");\n}\n";
    let helper = file_uri("/lsp-test/helper.jai");
    let mut s = session();
    s.open(uri(), 1, main.into()).unwrap();
    s.open(helper.clone(), 1, helper_text.into()).unwrap();
    let cursor = at_nth(helper_text, "print", 0, 0);
    let actions = s
        .code_actions(
            &helper,
            Range {
                start: cursor,
                end: cursor,
            },
        )
        .unwrap();
    assert_eq!(actions.len(), 1, "{actions:?}");
    let (target, edits) = actions[0].edit.as_ref().unwrap();
    assert_eq!(target, helper.as_str());
    assert_eq!(edits[0].range.start, at_nth(helper_text, "helper ::", 0, 0));
    assert_eq!(edits[0].new_text, "#import \"Basic\";\n\n");
    // The file that loads it has no error of its own to fix.
    let top_actions = s.code_actions(&uri(), everything(main)).unwrap();
    assert!(top_actions.is_empty(), "{top_actions:?}");
}

#[test]
fn an_import_fix_follows_the_check_diagnostic_it_is_asked_for() {
    let text = "main :: () {\n    print(\"hi\\n\");\n}\n";
    let mut json = opened(text);
    let error = Range {
        start: at_nth(text, "print", 0, 0),
        end: at_nth(text, "print", 0, "print".len()),
    };
    let diagnostic = serde_json::json!({
        "range": error, "code": "check", "source": "jaic", "message": ""});
    let found = code_actions(
        &mut json,
        top(),
        serde_json::json!({"diagnostics": [diagnostic], "only": ["quickfix"]}),
    );
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0]["kind"], "quickfix");
    assert_eq!(found[0]["title"], "Add `#import \"Basic\";`");
    assert_eq!(found[0]["isPreferred"], true);
    assert_eq!(found[0]["diagnostics"][0]["code"], "check");
    assert_eq!(found[0]["diagnostics"][0]["source"], "jaic");
    assert_eq!(
        found[0]["edit"]["changes"][uri().as_str()][0]["newText"],
        "#import \"Basic\";\n\n"
    );
    // Not part of fixing everything: an import is a choice about the program.
    let all = code_actions(
        &mut json,
        top(),
        serde_json::json!({"diagnostics": [], "only": ["source.fixAll"]}),
    );
    assert!(all.is_empty(), "{all:?}");
}
