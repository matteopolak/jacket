//! A panic in one request leaves the server running; settings files that do not parse are
//! reported to the client once.
use jai_language_server::{JsonSession, Limits};
use serde_json::{Value, json};
use test_support::lsp::{environment, utf16_len};

fn send(session: &mut JsonSession, message: Value) -> Vec<Value> {
    session
        .handle_json(&message.to_string())
        .unwrap()
        .iter()
        .map(|s| serde_json::from_str(s).unwrap())
        .collect()
}

fn open(session: &mut JsonSession, uri: &str, version: i32, text: &str) -> Vec<Value> {
    let method = if version == 1 {
        "textDocument/didOpen"
    } else {
        "textDocument/didChange"
    };
    let params = if version == 1 {
        json!({ "textDocument": { "uri": uri, "languageId": "jai", "version": 1, "text": text } })
    } else {
        json!({
            "textDocument": { "uri": uri, "version": version },
            "contentChanges": [{ "text": text }],
        })
    };
    let mut out = send(
        session,
        json!({ "jsonrpc": "2.0", "method": method, "params": params }),
    );

    if version > 1 && uri.ends_with(".toml") {
        out.extend(send(session, json!({ "jsonrpc":"2.0", "method":"textDocument/didSave", "params":{"textDocument":{"uri":uri}} })));
    }

    out
}

#[cfg(debug_assertions)]
#[test]
fn a_panic_is_an_error_response_and_the_server_keeps_working() {
    let mut s = JsonSession::new(Limits::default());

    send(
        &mut s,
        json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {} }),
    );
    let uri = "file:///resilience/main.jai";

    open(&mut s, uri, 1, "answer :: 42;\nmain :: () {}\n");
    let out = send(
        &mut s,
        json!({ "jsonrpc": "2.0", "id": 2, "method": "jai/debugPanic" }),
    );

    assert_eq!(out[0]["error"]["code"], -32603, "{out:?}");
    assert!(
        out[0]["error"]["message"]
            .as_str()
            .unwrap()
            .contains("jai/debugPanic")
    );
    // As a notification: logged, no response.
    let out = send(
        &mut s,
        json!({ "jsonrpc": "2.0", "method": "jai/debugPanic" }),
    );

    assert_eq!(out[0]["method"], "window/logMessage", "{out:?}");
    // The open document is still there and requests still work.
    let out = send(
        &mut s,
        json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "textDocument/documentSymbol",
            "params": { "textDocument": { "uri": uri } },
        }),
    );

    assert_eq!(out[0]["result"].as_array().unwrap().len(), 2, "{out:?}");
}

fn with_environment() -> JsonSession {
    JsonSession::with_environment(Limits::default(), environment())
}

fn messages(out: &[Value]) -> Vec<String> {
    out.iter()
        .filter(|m| m["method"] == "window/showMessage")
        .map(|m| m["params"]["message"].as_str().unwrap().to_string())
        .collect()
}

/// The last publication for `uri` in `out`.
fn published<'a>(out: &'a [Value], uri: &str) -> Option<&'a Value> {
    out.iter()
        .rev()
        .find(|m| m["method"] == "textDocument/publishDiagnostics" && m["params"]["uri"] == uri)
}

/// The diagnostics of the last publication for `uri`, from `source`.
fn from_source(out: &[Value], uri: &str, source: &str) -> Vec<Value> {
    published(out, uri)
        .and_then(|m| m["params"]["diagnostics"].as_array())
        .into_iter()
        .flatten()
        .filter(|d| d["source"] == source)
        .cloned()
        .collect()
}

#[test]
fn a_broken_settings_file_is_published_and_holds_back_the_jai_diagnostics() {
    let mut s = with_environment();

    send(
        &mut s,
        json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {} }),
    );
    let dir = "file:///resilience-config";
    let (config, main) = (format!("{dir}/jacket.toml"), format!("{dir}/main.jai"));
    let broken = "[lint.rules\nunused_variable = \"deny\"\n";
    let out = open(&mut s, &config, 1, broken);

    assert!(messages(&out).is_empty(), "{out:?}");
    assert_eq!(from_source(&out, &config, "jacket").len(), 1, "{out:?}");
    let out = open(&mut s, &main, 1, "main :: () {\n    x := 1;\n}\n");

    assert!(messages(&out).is_empty(), "{out:?}");
    assert!(from_source(&out, &main, "jailint").is_empty(), "{out:?}");
    // Fixed: the lints are back.
    let out = open(
        &mut s,
        &config,
        2,
        "[lint.rules]\nunused_variable = \"deny\"\n",
    );

    assert!(from_source(&out, &config, "jacket").is_empty(), "{out:?}");
    assert_eq!(from_source(&out, &main, "jailint").len(), 1, "{out:?}");
    // Broken again: held back again.
    let out = open(&mut s, &config, 3, broken);

    assert_eq!(from_source(&out, &config, "jacket").len(), 1, "{out:?}");
    assert!(from_source(&out, &main, "jailint").is_empty(), "{out:?}");
}

#[test]
fn a_bad_project_file_is_a_diagnostic_not_a_popup() {
    let mut s = with_environment();

    send(
        &mut s,
        json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {} }),
    );
    let dir = "file:///resilience-project";
    let config = format!("{dir}/jacket.toml");
    let out = open(&mut s, &config, 1, "[project]\nentry = 3\n");
    let problems = from_source(&out, &config, "jacket");

    assert_eq!(problems.len(), 1, "{out:?}");
    assert!(
        problems[0]["message"]
            .as_str()
            .unwrap()
            .contains("an array of strings"),
        "{problems:?}"
    );
    let main = format!("{dir}/main.jai");
    let text = "main :: () {\n    prin\n}\n";
    let mut found = messages(&out);

    found.extend(messages(&open(&mut s, &main, 1, text)));
    let out = send(
        &mut s,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/completion",
            "params": {
                "textDocument": { "uri": main },
                "position": { "line": 1, "character": 8 },
            },
        }),
    );

    found.extend(messages(&out));
    assert!(found.is_empty(), "{found:?}");
}

fn complete(s: &mut JsonSession, uri: &str, line: u32, character: u32) -> Vec<Value> {
    let out = send(
        s,
        json!({
            "jsonrpc": "2.0",
            "id": 9,
            "method": "textDocument/completion",
            "params": {
                "textDocument": { "uri": uri },
                "position": { "line": line, "character": character },
            },
        }),
    );
    out[0]["result"]["items"].as_array().unwrap().clone()
}

#[test]
fn completion_items_replace_the_whole_word_in_utf16() {
    let mut s = with_environment();

    send(
        &mut s,
        json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {} }),
    );
    let uri = "file:///resilience-edit/main.jai";
    // The emoji is two UTF-16 units; the cursor is in the middle of `render_all`.
    let text = "render_all :: () {}\nmain :: () {\n    s := \"😀\"; rend_er_all();\n}\n#imp\n";

    open(&mut s, uri, 1, text);
    let start = text.lines().nth(2).unwrap().find("rend_er_all").unwrap();
    let units = |bytes: usize| utf16_len(&text.lines().nth(2).unwrap()[..bytes]);
    let items = complete(&mut s, uri, 2, units(start + 4));
    let item = items
        .iter()
        .find(|i| i["label"] == "render_all")
        .unwrap_or_else(|| panic!("{items:?}"));
    let edit = &item["textEdit"];

    assert_eq!(edit["newText"], "render_all");
    assert_eq!(edit["range"]["start"]["character"], units(start));
    assert_eq!(
        edit["range"]["end"]["character"],
        units(start + "rend_er_all".len())
    );
    assert!(item.get("insertText").is_none());
    // A directive's range includes its `#`.
    let items = complete(&mut s, uri, 4, 4);
    let import = items.iter().find(|i| i["label"] == "#import").unwrap();

    assert_eq!(import["textEdit"]["range"]["start"]["character"], 0);
    assert_eq!(import["textEdit"]["range"]["end"]["character"], 4);
}

#[test]
fn unfinished_config_edits_do_not_apply_or_warn_until_saved() {
    let mut s = with_environment();

    send(
        &mut s,
        json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}),
    );
    let config = "file:///saved-config/jacket.toml";
    let main = "file:///saved-config/main.jai";

    open(
        &mut s,
        config,
        1,
        "[lint.rules]\nunused_variable=\"deny\"\n",
    );
    open(&mut s, main, 1, "main :: () { x:=1; }\n");

    for (version, text) in [
        (2, "import-path ="),
        (3, "import-path = a"),
        (4, "import-path = ab"),
    ] {
        let out = send(
            &mut s,
            json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{"textDocument":{"uri":config,"version":version},"contentChanges":[{"text":text}]}}),
        );

        assert!(messages(&out).is_empty(), "{out:?}");
        let out = open(&mut s, main, version, "main :: () { x:=1; }\n");

        assert!(messages(&out).is_empty(), "{out:?}");
        assert!(
            out.iter().any(
                |m| m["params"]["diagnostics"].as_array().is_some_and(|ds| ds
                    .iter()
                    .any(|d| d["source"] == "jailint" && d["severity"] == 1))
            ),
            "{out:?}"
        );
    }

    let save = json!({"jsonrpc":"2.0","method":"textDocument/didSave","params":{"textDocument":{"uri":config}}});

    for _ in 0..2 {
        let out = send(&mut s, save.clone());

        assert!(messages(&out).is_empty(), "{out:?}");
        assert_eq!(from_source(&out, config, "jacket").len(), 1, "{out:?}");
        assert!(from_source(&out, main, "jailint").is_empty(), "{out:?}");
    }

    let out = open(
        &mut s,
        config,
        5,
        "[lint.rules]\nunused_variable=\"allow\"\n",
    );

    assert!(messages(&out).is_empty());
    assert!(!out.iter().any(|m| {
        m["params"]["diagnostics"]
            .as_array()
            .is_some_and(|ds| ds.iter().any(|d| d["source"] == "jailint"))
    }));
}
