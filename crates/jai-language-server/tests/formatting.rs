//! `textDocument/formatting` and `textDocument/rangeFormatting`, answered by running `jaifmt`.
//! The tests that format use the built `jaifmt` (`<target dir>/jaifmt`, as `dev.py` and CI
//! build it); the rest drive the server with a stand-in script.
use jai_language_server::{JsonSession, Limits};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use test_support::TempDir;
use test_support::toolchain;

/// The built `jaifmt`: in the cargo target directory (a folder above this test, whatever the
/// layout below it), else on `PATH`. Skips (or fails, where tools are
/// required) when there is none.
fn jaifmt() -> Option<PathBuf> {
    let name = format!("jaifmt{}", std::env::consts::EXE_SUFFIX);
    let above = std::env::current_exe().ok().into_iter().flat_map(|exe| {
        exe.ancestors()
            .skip(1)
            .take(8)
            .map(Path::to_path_buf)
            .collect::<Vec<_>>()
    });
    above
        .map(|dir| dir.join(&name))
        .find(|candidate| candidate.is_file())
        .or_else(|| toolchain::require("jaifmt"))
}

struct Editor {
    session: JsonSession,
    next: u64,
}

impl Editor {
    /// A session with `jaifmt` at `formatter` (none: the server looks for it), with `files`
    /// written under `dir`; `main.jai` is open.
    fn open(dir: &Path, formatter: Option<&Path>, main: &str) -> Self {
        std::fs::write(dir.join("jacket.toml"), "").unwrap();
        std::fs::write(dir.join("main.jai"), main).unwrap();
        let mut editor = Self {
            session: JsonSession::new(Limits::default()),
            next: 1,
        };
        let options = formatter.map_or(json!({}), |p| json!({ "formatter": { "path": p } }));
        let reply = editor.send(json!({"jsonrpc": "2.0", "id": 0, "method": "initialize",
            "params": { "capabilities": {}, "initializationOptions": options }}));
        let capabilities = &reply[0]["result"]["capabilities"];
        assert_eq!(capabilities["documentFormattingProvider"], true);
        assert_eq!(capabilities["documentRangeFormattingProvider"], true);
        editor.send(
            json!({"jsonrpc": "2.0", "method": "textDocument/didOpen", "params": {
            "textDocument": { "uri": uri(dir), "languageId": "jai", "version": 1, "text": main }}}),
        );
        editor
    }

    fn send(&mut self, message: Value) -> Vec<Value> {
        let output: Vec<serde_json::Value> = self
            .session
            .handle_json(&message.to_string())
            .unwrap()
            .iter()
            .map(|s| serde_json::from_str(s).unwrap())
            .collect();
        if message["method"] == "initialize"
            && output.first().is_some_and(|m| m.get("result").is_some())
        {
            assert!(
                self.session
                    .handle_json(r#"{"jsonrpc":"2.0","method":"initialized","params":{}}"#)
                    .unwrap()
                    .is_empty()
            );
        }
        output
    }

    /// The whole reply (the response and any notifications) to a formatting request.
    fn format(&mut self, dir: &Path, range: Option<Value>) -> Vec<Value> {
        self.next += 1;
        let mut params = json!({ "textDocument": { "uri": uri(dir) },
            "options": { "tabSize": 2, "insertSpaces": false } });
        let method = if let Some(range) = range {
            params["range"] = range;
            "textDocument/rangeFormatting"
        } else {
            "textDocument/formatting"
        };
        self.send(json!({"jsonrpc": "2.0", "id": self.next, "method": method, "params": params}))
    }
}

fn uri(dir: &Path) -> String {
    jai_language_server::DocumentUri::from_path(&dir.join("main.jai"))
        .unwrap()
        .as_str()
        .to_string()
}

fn response(replies: &[Value]) -> &Value {
    replies.iter().find(|m| m.get("id").is_some()).unwrap()
}

fn logs(replies: &[Value]) -> Vec<String> {
    replies
        .iter()
        .filter(|m| m["method"] == "window/logMessage")
        .map(|m| m["params"]["message"].as_str().unwrap().to_string())
        .collect()
}

/// Applies `edits` (LSP text edits) to `text`, by line and UTF-16 column.
fn apply(text: &str, edits: &Value) -> String {
    let offset = |position: &Value| {
        let line = position["line"].as_u64().unwrap() as usize;
        let column = position["character"].as_u64().unwrap() as usize;
        let start: usize = text.split_inclusive('\n').take(line).map(str::len).sum();
        let mut units = 0;
        let mut bytes = 0;
        for c in text[start..].chars() {
            if units >= column {
                break;
            }
            units += c.len_utf16();
            bytes += c.len_utf8();
        }
        start + bytes
    };
    let mut out = text.to_string();
    let mut all: Vec<_> = edits.as_array().unwrap().iter().collect();
    all.sort_by_key(|e| std::cmp::Reverse(offset(&e["range"]["start"])));
    for edit in all {
        let (a, b) = (
            offset(&edit["range"]["start"]),
            offset(&edit["range"]["end"]),
        );
        out.replace_range(a..b, edit["newText"].as_str().unwrap());
    }
    out
}

const MESSY: &str =
    "first :: () {\nx:=1;\n}\n\nsecond :: () {\ny   :=   2;\n}\n\nthird :: () {\nz:=3;\n}\n";

#[test]
fn a_document_is_formatted_with_edits_for_the_changed_lines_only() {
    let Some(jaifmt) = jaifmt() else {
        return;
    };
    let dir = TempDir::new("jailsp-format-all");
    let mut editor = Editor::open(&dir, Some(&jaifmt), MESSY);
    let replies = editor.format(&dir, None);
    let edits = &response(&replies)["result"];
    assert!(response(&replies)["error"].is_null());
    assert!(logs(&replies).is_empty(), "{:?}", logs(&replies));
    let formatted = apply(MESSY, edits);
    assert_eq!(
        formatted,
        "first :: () {\n    x := 1;\n}\n\nsecond :: () {\n    y := 2;\n}\n\nthird :: () {\n    z := 3;\n}\n"
    );
    // The unchanged `}` lines between the changed ones are not in any edit's range, and the
    // first and last lines never are.
    let start = edits[0]["range"]["start"]["line"].as_u64().unwrap();
    assert_eq!(start, 1);
    let end = edits[0]["range"]["end"]["line"].as_u64().unwrap();
    assert!(end <= 10, "{edits}");
}

#[test]
fn formatted_text_has_no_edits() {
    let Some(jaifmt) = jaifmt() else {
        return;
    };
    let dir = TempDir::new("jailsp-format-clean");
    let clean = "main :: () {\n    x := 1;\n}\n";
    let mut editor = Editor::open(&dir, Some(&jaifmt), clean);
    let replies = editor.format(&dir, None);
    assert_eq!(response(&replies)["result"], json!([]));
}

#[test]
fn a_range_in_the_middle_of_a_statement_formats_only_that_statement() {
    let Some(jaifmt) = jaifmt() else {
        return;
    };
    let dir = TempDir::new("jailsp-format-range");
    let mut editor = Editor::open(&dir, Some(&jaifmt), MESSY);
    // Inside `y   :=   2;` of `second`, in the middle of the statement.
    let range =
        json!({ "start": { "line": 5, "character": 3 }, "end": { "line": 5, "character": 6 } });
    let replies = editor.format(&dir, Some(range));
    let edits = &response(&replies)["result"];
    assert!(logs(&replies).is_empty(), "{:?}", logs(&replies));
    let formatted = apply(MESSY, edits);
    assert_eq!(
        formatted,
        "first :: () {\nx:=1;\n}\n\nsecond :: () {\n    y := 2;\n}\n\nthird :: () {\nz:=3;\n}\n"
    );
    for edit in edits.as_array().unwrap() {
        let (a, b) = (
            edit["range"]["start"]["line"].as_u64().unwrap(),
            edit["range"]["end"]["line"].as_u64().unwrap(),
        );
        assert!((4..=6).contains(&a) && (4..=7).contains(&b), "{edit}");
    }
}

/// A stand-in `jaifmt` that records its arguments in `args` next to it, then runs `body`.
#[cfg(unix)]
fn stand_in(dir: &Path, body: &str) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;
    let path = dir.join("fake-jaifmt");
    let script = format!(
        "#!/bin/sh\nif [ \"$1\" = \"--is-ignored\" ]; then exit 1; fi\necho \"$@\" > \"{}\"\n{body}\n",
        dir.join("args").display()
    );
    std::fs::write(&path, script).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    path
}

#[cfg(unix)]
#[test]
fn a_range_is_sent_to_jaifmt_as_byte_offsets_of_its_utf16_positions() {
    let dir = TempDir::new("jailsp-format-utf16");
    let formatter = stand_in(&dir, "cat");
    // Line 0 holds a 4-byte (2 UTF-16 units) character and a 2-byte (1 unit) one before `x`.
    let text = "// \u{1F600}\u{e9}\nx := 1;\ny := \"\u{e9}\u{e9}\" + 2;\n";
    let mut editor = Editor::open(&dir, Some(&formatter), text);
    let start_of_y = text.find('y').unwrap();
    // Line 2 is `y := "éé" + 2;`; columns count UTF-16 units, so columns 5..9 hold `"éé"`,
    // which is 1 + 2 + 2 + 1 bytes.
    let range =
        json!({ "start": { "line": 2, "character": 5 }, "end": { "line": 2, "character": 9 } });
    let replies = editor.format(&dir, Some(range));
    assert!(response(&replies)["error"].is_null());
    let args = std::fs::read_to_string(dir.join("args")).unwrap();
    assert_eq!(
        args.trim(),
        format!("--stdin --range {}:{}", start_of_y + 5, start_of_y + 11)
    );
    // The same on the first line, where the character before the range is outside the BMP.
    let range =
        json!({ "start": { "line": 0, "character": 5 }, "end": { "line": 0, "character": 5 } });
    editor.format(&dir, Some(range));
    let args = std::fs::read_to_string(dir.join("args")).unwrap();
    assert_eq!(args.trim(), "--stdin --range 7:7");
}

#[test]
fn a_missing_formatter_gives_no_edits_and_a_log_line() {
    let dir = TempDir::new("jailsp-format-missing");
    let nowhere = dir.path().join("no-such-jaifmt");
    let mut editor = Editor::open(&dir, Some(&nowhere), MESSY);
    let replies = editor.format(&dir, None);
    let reply = response(&replies);
    assert!(reply["error"].is_null(), "{reply}");
    assert_eq!(reply["result"], json!([]));
    let logs = logs(&replies);
    assert_eq!(logs.len(), 1, "{logs:?}");
    assert!(logs[0].contains("does not exist"), "{logs:?}");
}

#[cfg(unix)]
#[test]
fn a_failing_formatter_gives_no_edits_and_a_log_line() {
    let dir = TempDir::new("jailsp-format-failure");
    let formatter = stand_in(
        &dir,
        "echo '<stdin>:1:1: unbalanced bracket' >&2\ncat > /dev/null\nexit 2",
    );
    let mut editor = Editor::open(&dir, Some(&formatter), "main :: () {\n");
    for _ in 0..2 {
        let replies = editor.format(&dir, None);
        let reply = response(&replies);
        assert!(reply["error"].is_null(), "{reply}");
        assert_eq!(reply["result"], json!([]));
        let logs = logs(&replies);
        assert_eq!(logs.len(), 1, "every failure is logged: {logs:?}");
        assert!(logs[0].contains("unbalanced bracket"), "{logs:?}");
    }
}

#[cfg(unix)]
#[test]
fn output_larger_than_the_pipe_buffers_is_read_in_full() {
    let dir = TempDir::new("jailsp-format-large");
    let formatter = stand_in(&dir, "cat\nyes 'a := 1;' | head -c 400000");
    let mut editor = Editor::open(&dir, Some(&formatter), "x := 1;\n");
    let replies = editor.format(&dir, None);
    let edits = &response(&replies)["result"];
    assert!(apply("x := 1;\n", edits).len() > 400_000);
}
