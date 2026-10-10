//! Metaprogramming, format-string and navigation features against the repository's stdlib.
use jai_language_server::{
    DiagnosticCode, DiagnosticSeverity, DocumentUri, Environment, InlayHintKind, JsonSession,
    Limits, MarkupKind, Position, Range, SemanticTokenKind, Session,
};
use std::path::PathBuf;

fn environment() -> Environment {
    let stdlib = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    Environment {
        fs: std::rc::Rc::new(jaic::sema::NativeFs),
        options: Box::new(move |_| {
            let mut options = jaic::sema::Options::host();
            options.import_paths = vec![stdlib.clone()];
            options.preload = Some(stdlib.join("prelude/module.jai"));
            options
        }),
    }
}

fn session() -> Session {
    Session::with_environment(Limits::default(), environment())
}

fn uri() -> DocumentUri {
    DocumentUri::parse("file:///lsp-features-test/main.jai").unwrap()
}

/// Position of the `n`th (0-based) occurrence of `marker`, `offset` bytes into it.
fn at(text: &str, marker: &str, n: usize, offset: usize) -> Position {
    let byte = text.match_indices(marker).nth(n).unwrap().0 + offset;
    let prefix = &text[..byte];
    Position {
        line: prefix.matches('\n').count() as u32,
        character: prefix.rsplit('\n').next().unwrap().len() as u32,
    }
}

fn everything(text: &str) -> Range {
    Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: text.lines().count() as u32,
            character: 0,
        },
    }
}

fn hover(s: &Session, position: Position) -> String {
    s.hover(&uri(), position)
        .unwrap()
        .expect("hover")
        .contents
        .value
}

fn markdown(s: &Session, position: Position) -> String {
    let hover = s
        .hover_as(&uri(), position, MarkupKind::Markdown)
        .unwrap()
        .expect("hover");
    assert_eq!(hover.contents.kind, MarkupKind::Markdown);
    hover.contents.value
}

const PROGRAM: &str = r#"#import "Basic";
square :: (x: int) -> int #expand {
    `total += x;
    return x * x;
}
make_code :: () -> string { return "inserted := 40 + 2;"; }
SIZE :: #run compute(3);
compute :: (n: int) -> int { print("computing\n"); return n * 10; }
scale :: (value: int, factor: int) -> int { return value * factor; }
Kind :: enum { ONE; TWO; }
identity :: (v: $T) -> T { return v; }
main :: () {
    total := 1;
    s := square(total + 2);
    #insert #run make_code();
    #if SIZE > 10 { big := true; } else { big := false; }
    scaled := scale(5, factor = 2);
    kind := Kind.TWO;
    a := identity(3);
    b := identity("x");
    print("% and %2 of %1\n", total, s);
    print("% %\n", inserted, s);
}
"#;

#[test]
fn hovers_show_what_metaprograms_produced() {
    let mut s = session();
    s.open(uri(), 1, PROGRAM.into()).unwrap();
    let macro_hover = hover(&s, at(PROGRAM, "square(total", 0, 2));
    assert!(
        macro_hover.starts_with("square :: (x: int) -> int #expand"),
        "{macro_hover}"
    );
    assert!(
        macro_hover.contains(
            "\n─── expands to ───\ntotal += (total + 2);\nreturn (total + 2) * (total + 2);"
        ),
        "{macro_hover}"
    );
    let insert = hover(&s, at(PROGRAM, "#insert", 0, 2));
    assert_eq!(insert, "#insert\n─── expands to ───\ninserted := 40 + 2;");
    let run = hover(&s, at(PROGRAM, "#run compute", 0, 2));
    assert_eq!(run, "#run = 30: s64\n─── prints ───\ncomputing");
    let condition = hover(&s, at(PROGRAM, "#if SIZE", 0, 1));
    assert_eq!(
        condition,
        "#if: the condition is true, the first branch is compiled"
    );
}

#[test]
fn format_string_hover_lists_each_directive() {
    let mut s = session();
    s.open(uri(), 1, PROGRAM.into()).unwrap();
    let summary = hover(&s, at(PROGRAM, "\"% and", 0, 4));
    assert_eq!(
        summary,
        "\"% and %2 of %1\\n\"\n  %  → total: s64\n  %2 → s: s64\n  %1 → total: s64"
    );
    let marked = hover(&s, at(PROGRAM, "%2 of", 0, 0));
    assert!(marked.contains("▸ %2 → s: s64"), "{marked}");
}

#[test]
fn markdown_hovers_fence_code_and_break_sections() {
    let mut s = session();
    s.open(uri(), 1, PROGRAM.into()).unwrap();
    let macro_hover = markdown(&s, at(PROGRAM, "square(total", 0, 2));
    assert!(
        macro_hover.starts_with("```jai\nsquare :: (x: int) -> int #expand"),
        "{macro_hover}"
    );
    assert!(
        macro_hover.ends_with(
            "\n```\n\n---\n\n*expands to*\n\n```jai\n\
             total += (total + 2);\nreturn (total + 2) * (total + 2);\n```"
        ),
        "{macro_hover}"
    );
    assert_eq!(
        markdown(&s, at(PROGRAM, "#insert", 0, 2)),
        "```jai\n#insert\n```\n\n---\n\n*expands to*\n\n```jai\ninserted := 40 + 2;\n```"
    );
    assert_eq!(
        markdown(&s, at(PROGRAM, "#run compute", 0, 2)),
        "```jai\n#run = 30: s64\n```\n\n---\n\n*prints*\n\n```text\ncomputing\n```"
    );
    assert_eq!(
        markdown(&s, at(PROGRAM, "#if SIZE", 0, 1)),
        "`#if`: the condition is true, the first branch is compiled"
    );
    assert_eq!(
        markdown(&s, at(PROGRAM, "return x * x", 0, 1)),
        "keyword `return`"
    );
    assert_eq!(
        markdown(&s, at(PROGRAM, "scale(5", 0, 1)),
        "```jai\nscale :: (value: int, factor: int) -> int\n```"
    );
    // An overload set: one declaration per overload, the documented ones followed by
    // their docs.
    let overloads = markdown(&s, at(PROGRAM, "print(\"% and", 0, 1));
    assert!(overloads.starts_with("```jai\nprint :: ("), "{overloads}");
    assert!(
        overloads.contains("Writes the formatted text to standard output"),
        "{overloads}"
    );
    assert_eq!(overloads.matches("\nprint :: (").count(), 2, "{overloads}");
}

#[test]
fn markdown_format_hover_is_a_list_with_the_hovered_row_bold() {
    let mut s = session();
    s.open(uri(), 1, PROGRAM.into()).unwrap();
    assert_eq!(
        markdown(&s, at(PROGRAM, "%2 of", 0, 0)),
        "```jai\n\"% and %2 of %1\\n\"\n```\n\n\
         - `%` → `total: s64`\n\
         - **`%2`** → `s: s64`\n\
         - `%1` → `total: s64`"
    );
    assert_eq!(
        markdown(&s, at(PROGRAM, "% %\\n", 0, 0)),
        "```jai\n\"% %\\n\"\n```\n\n\
         - **`%`** → `inserted: s64`\n\
         - `%` → `s: s64`"
    );
}

#[test]
fn inlay_hints_show_types_parameters_and_run_values() {
    let mut s = session();
    s.open(uri(), 1, PROGRAM.into()).unwrap();
    let hints = s.inlay_hints(&uri(), everything(PROGRAM)).unwrap();
    let find = |label: &str| {
        hints
            .iter()
            .find(|h| h.label == label)
            .unwrap_or_else(|| panic!("no hint {label}: {hints:?}"))
    };
    let total = find(": s64");
    assert_eq!(total.position, at(PROGRAM, "total := 1", 0, 5));
    assert_eq!(total.kind, Some(InlayHintKind::Type));
    assert_eq!(
        find(": string").position,
        at(PROGRAM, "b := identity", 0, 1)
    );
    let value = find("value:");
    assert_eq!(value.position, at(PROGRAM, "5, factor", 0, 0));
    assert_eq!(value.kind, Some(InlayHintKind::Parameter));
    // A named argument needs no hint, and `kind := Kind.TWO` says its type.
    assert!(!hints.iter().any(|h| h.label == "factor:"), "{hints:?}");
    assert!(!hints.iter().any(|h| h.label == ": Kind"), "{hints:?}");
    // Only parameters sharing a type get names: `scale` has two ints, but `print`'s
    // format string and `identity`'s single parameter are unambiguous.
    assert!(
        !hints
            .iter()
            .any(|h| h.kind == Some(InlayHintKind::Parameter) && h.label != "value:"),
        "{hints:?}"
    );
    assert_eq!(
        find("= 30").position,
        at(PROGRAM, "SIZE :: #run compute(3)", 0, 23)
    );
}

#[test]
fn code_actions_show_and_inline_expansions() {
    let mut s = session();
    s.open(uri(), 1, PROGRAM.into()).unwrap();
    let cursor = at(PROGRAM, "#insert", 0, 1);
    let actions = s
        .code_actions(
            &uri(),
            Range {
                start: cursor,
                end: cursor,
            },
        )
        .unwrap();
    let titles: Vec<&str> = actions.iter().map(|a| a.title.as_str()).collect();
    assert_eq!(titles, ["Show #insert expansion", "Inline #insert"]);
    let (_, edits) = actions[1].edit.as_ref().unwrap();
    assert_eq!(edits[0].new_text, "inserted := 40 + 2;");
    assert_eq!(edits[0].range.start, at(PROGRAM, "#insert", 0, 0));
    assert_eq!(edits[0].range.end, at(PROGRAM, "();\n    #if", 0, 3));
    let command = actions[0].command.as_ref().unwrap();
    assert_eq!(command.command, "jai.showExpansion");
    let (target, position) = command.target.clone().unwrap();
    let expansion = s
        .expansion(&DocumentUri::parse(&target).unwrap(), position)
        .unwrap()
        .unwrap();
    assert_eq!(expansion.kind, "insert");
    assert!(
        expansion
            .uri
            .starts_with("jai-expansion:///lsp-features-test/main.jai?")
    );
    assert_eq!(
        expansion.text,
        "// Expansion of the #insert at main.jai:15:5\ninserted := 40 + 2;\n"
    );
    assert_eq!(s.expansion_source(&expansion.uri), Some(expansion.text));
    let cursor = at(PROGRAM, "#run compute", 0, 1);
    let actions = s
        .code_actions(
            &uri(),
            Range {
                start: cursor,
                end: cursor,
            },
        )
        .unwrap();
    // It printed something: replacing it would drop the output.
    assert_eq!(actions.len(), 1, "{actions:?}");
}

#[test]
fn semantic_tokens_distinguish_jai_constructs() {
    let mut s = session();
    s.open(uri(), 1, PROGRAM.into()).unwrap();
    let tokens = s.semantic_tokens(&uri()).unwrap();
    let kind_at = |position: Position| {
        tokens
            .iter()
            .find(|t| t.position == position)
            .unwrap_or_else(|| panic!("no token at {position:?}"))
    };
    let square = kind_at(at(PROGRAM, "square(total", 0, 0));
    assert_eq!(square.kind, SemanticTokenKind::Function);
    assert!(square.expand);
    assert_eq!(
        kind_at(at(PROGRAM, "$T", 0, 1)).kind,
        SemanticTokenKind::TypeParameter
    );
    assert_eq!(
        kind_at(at(PROGRAM, "-> T", 0, 3)).kind,
        SemanticTokenKind::TypeParameter
    );
    assert_eq!(
        kind_at(at(PROGRAM, "Kind.TWO", 0, 0)).kind,
        SemanticTokenKind::Type
    );
    assert_eq!(
        kind_at(at(PROGRAM, "Kind.TWO", 0, 5)).kind,
        SemanticTokenKind::EnumMember
    );
    let size = kind_at(at(PROGRAM, "SIZE > 10", 0, 0));
    assert_eq!(size.kind, SemanticTokenKind::Variable);
    assert!(size.readonly);
    assert_eq!(
        kind_at(at(PROGRAM, "%2 of", 0, 0)).kind,
        SemanticTokenKind::FormatSpecifier
    );
    assert_eq!(kind_at(at(PROGRAM, "%2 of", 0, 0)).length, 2);
    assert_eq!(
        kind_at(at(PROGRAM, "#insert", 0, 0)).kind,
        SemanticTokenKind::Macro
    );
}

#[test]
fn semantic_tokens_leave_language_tagged_here_strings_to_the_editor() {
    const TEXT: &str =
        "SHADER :: #string,cr wgsl\n@vertex fn vs() {}\nwgsl;\nTEXT :: #string END\nplain\nEND;\n";
    let mut s = Session::new(Limits::default());
    s.open(uri(), 1, TEXT.into()).unwrap();
    let tokens = s.semantic_tokens(&uri()).unwrap();
    let strings: Vec<u32> = tokens
        .iter()
        .filter(|t| t.kind == SemanticTokenKind::String)
        .map(|t| t.position.line)
        .collect();
    // Only the untagged here-string, one token per line.
    assert_eq!(strings, [3, 4, 5], "{tokens:?}");
    assert!(
        tokens
            .iter()
            .any(|t| t.position == at(TEXT, "SHADER", 0, 0))
    );
}

#[test]
fn references_type_definitions_and_signature_help() {
    let mut s = session();
    s.open(uri(), 1, PROGRAM.into()).unwrap();
    let refs = s
        .references(&uri(), at(PROGRAM, "total := 1", 0, 1), true)
        .unwrap();
    let starts: Vec<Position> = refs.iter().map(|r| r.range.start).collect();
    assert!(
        starts.contains(&at(PROGRAM, "total := 1", 0, 0)),
        "{starts:?}"
    );
    assert!(
        starts.contains(&at(PROGRAM, "total + 2", 0, 0)),
        "{starts:?}"
    );
    assert!(
        starts.contains(&at(PROGRAM, "total, s)", 0, 0)),
        "{starts:?}"
    );
    let calls = s
        .references(&uri(), at(PROGRAM, "scale ::", 0, 1), false)
        .unwrap();
    assert_eq!(calls.len(), 1, "{calls:?}");
    assert_eq!(calls[0].range.start, at(PROGRAM, "scale(5", 0, 0));
    let types = s
        .type_definition(&uri(), at(PROGRAM, "kind :=", 0, 1))
        .unwrap();
    assert_eq!(types.len(), 1, "{types:?}");
    assert_eq!(types[0].range.start, at(PROGRAM, "Kind :: enum", 0, 0));
    let help = s
        .signature_help(&uri(), at(PROGRAM, "factor = 2", 0, 0))
        .unwrap()
        .unwrap();
    assert_eq!(
        help.signatures[help.active_signature].label,
        "scale :: (value: int, factor: int) -> int"
    );
    assert_eq!(help.active_parameter, 1);
    // While typing: the call does not parse yet.
    let typing = PROGRAM.replace("    kind := Kind.TWO;\n", "    x := scale(1, \n");
    let mut s = session();
    s.open(uri(), 1, typing.clone()).unwrap();
    let help = s
        .signature_help(&uri(), at(&typing, "scale(1, ", 0, 9))
        .unwrap()
        .unwrap();
    assert_eq!(help.signatures[0].parameters, ["value: int", "factor: int"]);
    assert_eq!(help.active_parameter, 1);
}

#[test]
fn outline_folding_lenses_and_workspace_symbols() {
    let mut s = session();
    s.open(uri(), 1, PROGRAM.into()).unwrap();
    let symbols = s.workspace_symbols("SCA");
    assert_eq!(symbols.len(), 1);
    assert_eq!(symbols[0].name, "scale");
    let folds = s.folding_ranges(&uri()).unwrap();
    let main = at(PROGRAM, "main ::", 0, 0).line;
    assert!(folds.iter().any(|f| f.start_line == main), "{folds:?}");
    let lenses = s.code_lenses(&uri()).unwrap();
    assert_eq!(lenses.len(), 1, "{lenses:?}");
    assert_eq!(lenses[0].range.start, at(PROGRAM, "identity ::", 0, 0));
    assert_eq!(lenses[0].command.title, "2 polymorphs: T = s64; T = string");
    assert_eq!(
        s.polymorphs(&uri(), lenses[0].range.start),
        ["T = s64", "T = string"]
    );
}

#[test]
fn protocol_exposes_the_new_requests() {
    let mut json = JsonSession::with_environment(Limits::default(), environment());
    let send = |json: &mut JsonSession, message: serde_json::Value| -> serde_json::Value {
        let out = json.handle_json(&message.to_string()).unwrap();
        out.last()
            .map(|m| serde_json::from_str(m).unwrap())
            .unwrap_or(serde_json::Value::Null)
    };
    let init = send(
        &mut json,
        serde_json::json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}}),
    );
    let caps = &init["result"]["capabilities"];
    for key in [
        "inlayHintProvider",
        "codeActionProvider",
        "executeCommandProvider",
        "referencesProvider",
        "signatureHelpProvider",
        "foldingRangeProvider",
        "codeLensProvider",
        "workspaceSymbolProvider",
        "typeDefinitionProvider",
        "documentHighlightProvider",
        "renameProvider",
    ] {
        assert!(!caps[key].is_null(), "{key}");
    }
    assert!(
        caps["semanticTokensProvider"]["legend"]["tokenTypes"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!("formatSpecifier"))
    );
    send(
        &mut json,
        serde_json::json!({"jsonrpc": "2.0", "method": "textDocument/didOpen", "params": {
            "textDocument": {"uri": uri().as_str(), "languageId": "jai", "version": 1, "text": PROGRAM}}}),
    );
    let document = serde_json::json!({"uri": uri().as_str()});
    let hints = send(
        &mut json,
        serde_json::json!({"jsonrpc": "2.0", "id": 2, "method": "textDocument/inlayHint", "params": {
            "textDocument": document, "range": everything(PROGRAM)}}),
    );
    assert!(
        hints["result"]
            .as_array()
            .unwrap()
            .iter()
            .any(|h| h["label"] == ": s64" && h["kind"] == 1),
        "{hints}"
    );
    let position = at(PROGRAM, "#insert", 0, 1);
    let actions = send(
        &mut json,
        serde_json::json!({"jsonrpc": "2.0", "id": 3, "method": "textDocument/codeAction", "params": {
            "textDocument": document, "range": {"start": position, "end": position}, "context": {"diagnostics": []}}}),
    );
    let show = &actions["result"][0]["command"];
    assert_eq!(show["command"], "jai.showExpansion");
    let inline = &actions["result"][1]["edit"]["changes"][uri().as_str()][0];
    assert_eq!(inline["newText"], "inserted := 40 + 2;");
    let executed = send(
        &mut json,
        serde_json::json!({"jsonrpc": "2.0", "id": 4, "method": "workspace/executeCommand", "params": {
            "command": "jai.showExpansion", "arguments": show["arguments"]}}),
    );
    let expansion_uri = executed["result"]["uri"].as_str().unwrap().to_string();
    assert!(expansion_uri.starts_with("jai-expansion:"));
    let direct = send(
        &mut json,
        serde_json::json!({"jsonrpc": "2.0", "id": 5, "method": "jai/expansion", "params": {
            "textDocument": document, "position": position}}),
    );
    assert_eq!(direct["result"], executed["result"]);
    let source = send(
        &mut json,
        serde_json::json!({"jsonrpc": "2.0", "id": 6, "method": "jai/source", "params": {"uri": expansion_uri}}),
    );
    assert_eq!(source["result"], executed["result"]["text"]);
    for (id, method) in [
        (7, "textDocument/references"),
        (8, "textDocument/signatureHelp"),
        (9, "textDocument/typeDefinition"),
        (10, "textDocument/documentHighlight"),
    ] {
        let reply = send(
            &mut json,
            serde_json::json!({"jsonrpc": "2.0", "id": id, "method": method, "params": {
                "textDocument": document, "position": at(PROGRAM, "total := 1", 0, 1),
                "context": {"includeDeclaration": true}}}),
        );
        assert!(reply.get("error").is_none(), "{method}: {reply}");
    }
    for (id, method) in [
        (11, "textDocument/foldingRange"),
        (12, "textDocument/codeLens"),
    ] {
        let reply = send(
            &mut json,
            serde_json::json!({"jsonrpc": "2.0", "id": id, "method": method, "params": {"textDocument": document}}),
        );
        assert!(
            reply["result"].as_array().is_some_and(|a| !a.is_empty()),
            "{method}: {reply}"
        );
    }
    let symbols = send(
        &mut json,
        serde_json::json!({"jsonrpc": "2.0", "id": 13, "method": "workspace/symbol", "params": {"query": "identity"}}),
    );
    assert_eq!(symbols["result"][0]["name"], "identity");
}

#[test]
fn rename_edits_every_reference() {
    let mut s = session();
    s.open(uri(), 1, PROGRAM.into()).unwrap();
    let position = at(PROGRAM, "scale(5", 0, 2);
    let range = s.prepare_rename(&uri(), position).unwrap().unwrap();
    assert_eq!(range.start, at(PROGRAM, "scale(5", 0, 0));
    let edits = s.rename(&uri(), position, "resize").unwrap().unwrap();
    assert_eq!(edits.len(), 1);
    let starts: Vec<Position> = edits[0].1.iter().map(|e| e.range.start).collect();
    assert_eq!(
        starts,
        [at(PROGRAM, "scale ::", 0, 0), at(PROGRAM, "scale(5", 0, 0)]
    );
    assert!(edits[0].1.iter().all(|e| e.new_text == "resize"));
    // Locals and enum members rename too.
    let local = s
        .rename(&uri(), at(PROGRAM, "total := 1", 0, 1), "sum")
        .unwrap()
        .unwrap();
    // The declaration, two uses, and `` `total `` in the macro body (the caller's local).
    assert_eq!(local[0].1.len(), 4, "{local:?}");
    assert_eq!(local[0].1[0].range.start, at(PROGRAM, "total += x", 0, 0));
    assert!(s.rename(&uri(), position, "two words").is_err());
    assert!(
        s.prepare_rename(&uri(), at(PROGRAM, "Kind.TWO", 0, 6))
            .unwrap()
            .is_some()
    );
}

#[test]
fn keywords_are_described_without_the_language_name() {
    let mut s = session();
    s.open(uri(), 1, PROGRAM.into()).unwrap();
    assert_eq!(
        hover(&s, at(PROGRAM, "return x * x", 0, 1)),
        "keyword return"
    );
    let items = s
        .completion(&uri(), at(PROGRAM, "total := 1", 0, 0))
        .unwrap()
        .items;
    let keyword = items.iter().find(|i| i.label == "return").unwrap();
    assert_eq!(keyword.detail, "keyword");
}

const CODE_MACRO: &str = r#"#import "Basic";
twice :: (body: Code) #expand {
    #insert body;
    #insert body;
}
describe :: (v: $T) {
    #if T == string { print("text\n"); } else { print("other\n"); }
}
main :: () {
    twice(#code print("hi\n"));
    describe(1);
    describe("x");
    #if OS == .NONE { never := 1; }
}
"#;

#[test]
fn code_arguments_instances_and_untaken_branches() {
    let mut s = session();
    s.open(uri(), 1, CODE_MACRO.into()).unwrap();
    let twice = hover(&s, at(CODE_MACRO, "twice(#code", 0, 1));
    assert!(
        twice.ends_with("\n─── expands to ───\nprint(\"hi\\n\");\nprint(\"hi\\n\");"),
        "{twice}"
    );
    assert_eq!(
        hover(&s, at(CODE_MACRO, "#if T", 0, 1)),
        "#if: the condition is true for some instances and false for others"
    );
    assert_eq!(
        hover(&s, at(CODE_MACRO, "#if OS", 0, 1)),
        "#if: the condition is false, the else branch is compiled"
    );
}

const LINTED: &str = r#"#import "Basic";
main :: () {
    leftover := 5;
    ready := leftover > 0;
    if ready == true print("ready\n");
}
"#;

#[test]
fn lints_are_diagnostics_with_quick_fixes() {
    let mut s = session();
    s.open(uri(), 1, LINTED.into()).unwrap();
    let lints: Vec<_> = s
        .diagnostics(&uri())
        .unwrap()
        .into_iter()
        .filter(|d| matches!(d.code, DiagnosticCode::Lint(_)))
        .collect();
    assert_eq!(lints.len(), 1, "{lints:?}");
    assert_eq!(lints[0].code, DiagnosticCode::Lint("bool_comparison"));
    assert_eq!(lints[0].severity, DiagnosticSeverity::Warning);
    assert_eq!(lints[0].range.start, at(LINTED, "ready == true", 0, 0));
    let cursor = at(LINTED, "== true", 0, 0);
    let actions = s
        .code_actions(
            &uri(),
            Range {
                start: cursor,
                end: cursor,
            },
        )
        .unwrap();
    // The quick fix, then the action that applies every safe fix.
    assert_eq!(actions.len(), 2, "{actions:?}");
    assert_eq!(actions[0].kind, Some("quickfix"));
    assert_eq!(actions[0].title, "Write `ready`");
    assert_eq!(actions[0].rule, Some("bool_comparison"));
    assert!(actions[0].is_preferred);
    assert_eq!(actions[0].diagnostics, lints);
    let (_, edits) = actions[0].edit.as_ref().unwrap();
    assert_eq!(edits[0].new_text, "ready");
    assert_eq!(actions[1].kind, Some("source.fixAll.jailint"));
    // Over the wire: the rule is the code and jailint the source.
    let mut json = JsonSession::with_environment(Limits::default(), environment());
    let init = serde_json::json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}});
    json.handle_json(&init.to_string()).unwrap();
    let open = serde_json::json!({"jsonrpc": "2.0", "method": "textDocument/didOpen", "params": {
        "textDocument": {"uri": uri().as_str(), "languageId": "jai", "version": 1, "text": LINTED}}});
    let published: serde_json::Value = json
        .handle_json(&open.to_string())
        .unwrap()
        .iter()
        .map(|m| serde_json::from_str::<serde_json::Value>(m).unwrap())
        .find(|m| m["method"] == "textDocument/publishDiagnostics")
        .unwrap();
    let diagnostic = &published["params"]["diagnostics"][0];
    assert_eq!(diagnostic["code"], "bool_comparison");
    assert_eq!(diagnostic["source"], "jailint");
    assert_eq!(
        diagnostic["codeDescription"]["href"],
        "https://github.com/matteopolak/jai/blob/main/docs/tools/jailint.md#bool_comparison"
    );
}

/// Send one request over the JSON protocol and return its result.
fn request(json: &mut JsonSession, method: &str, params: serde_json::Value) -> serde_json::Value {
    let message =
        serde_json::json!({"jsonrpc": "2.0", "id": 7, "method": method, "params": params});
    let replies = json.handle_json(&message.to_string()).unwrap();
    replies
        .iter()
        .map(|m| serde_json::from_str::<serde_json::Value>(m).unwrap())
        .find(|m| m["id"] == 7)
        .expect("a reply")["result"]
        .clone()
}

/// Send a notification; the messages the server sends back.
fn notify(
    json: &mut JsonSession,
    method: &str,
    params: serde_json::Value,
) -> Vec<serde_json::Value> {
    let message = serde_json::json!({"jsonrpc": "2.0", "method": method, "params": params});
    json.handle_json(&message.to_string())
        .unwrap()
        .iter()
        .map(|m| serde_json::from_str::<serde_json::Value>(m).unwrap())
        .collect()
}

const TWO_LINTS: &str = r#"#import "Basic";
flag :: () -> bool { return true; }
main :: () {
    a := flag();
    b := flag();
    if a == true print("a\n");
    if b == false print("b\n");
}
"#;

/// A JSON session with `text` open as `uri()`.
fn opened(text: &str) -> JsonSession {
    let mut json = JsonSession::with_environment(Limits::default(), environment());
    let init = request(&mut json, "initialize", serde_json::json!({}));
    let kinds = &init["capabilities"]["codeActionProvider"]["codeActionKinds"];
    assert!(
        kinds
            .as_array()
            .unwrap()
            .contains(&serde_json::json!("source.fixAll.jailint")),
        "{kinds}"
    );
    notify(
        &mut json,
        "textDocument/didOpen",
        serde_json::json!({"textDocument": {
            "uri": uri().as_str(), "languageId": "jai", "version": 1, "text": text}}),
    );
    json
}

fn code_actions(
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
        start: at(TWO_LINTS, "b == false", 0, 0),
        end: at(TWO_LINTS, "b == false", 0, "b == false".len()),
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
    let settings = "file:///lsp-features-test/jailint.toml";
    let published = notify(
        &mut json,
        "textDocument/didOpen",
        serde_json::json!({"textDocument": {"uri": settings, "languageId": "toml",
            "version": 1, "text": "[rules]\nbool_comparison = \"allow\"\n"}}),
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
            "contentChanges": [{"text": "[rules]\nbool_comparison = \"deny\"\n"}]}),
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
    let cursor = at(text, marker, 0, 0);
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
        (at(text, "main", 0, 0), "#import \"Basic\";\n\n")
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
    let other = DocumentUri::parse("file:///lsp-features-test/other.jai").unwrap();
    s.open(other, 1, "helper :: () {}\n".into()).unwrap();
    s.open(uri(), 1, text.into()).unwrap();
    let cursor = at(text, "print", 0, 0);
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
        (at(text, "#load", 0, 0), "#import \"Basic\";\n")
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
    assert_eq!(actions.len(), 1, "{actions:?}");
    assert_eq!(actions[0].title, "Add `Math :: #import \"Math\";`");
    assert_eq!(single_edit(&actions[0]).1, "Math :: #import \"Math\";\n\n");
}

#[test]
fn a_loaded_file_gets_the_import_itself() {
    let main = "#load \"helper.jai\";\nmain :: () { helper(); }\n";
    let helper_text = "// Helpers.\nhelper :: () {\n    print(\"x\\n\");\n}\n";
    let helper = DocumentUri::parse("file:///lsp-features-test/helper.jai").unwrap();
    let mut s = session();
    s.open(uri(), 1, main.into()).unwrap();
    s.open(helper.clone(), 1, helper_text.into()).unwrap();
    let cursor = at(helper_text, "print", 0, 0);
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
    assert_eq!(edits[0].range.start, at(helper_text, "helper ::", 0, 0));
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
        start: at(text, "print", 0, 0),
        end: at(text, "print", 0, "print".len()),
    };
    let diagnostic = serde_json::json!({
        "range": error, "code": "jai-check", "source": "jai", "message": ""});
    let found = code_actions(
        &mut json,
        top(),
        serde_json::json!({"diagnostics": [diagnostic], "only": ["quickfix"]}),
    );
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0]["kind"], "quickfix");
    assert_eq!(found[0]["title"], "Add `#import \"Basic\";`");
    assert_eq!(found[0]["isPreferred"], true);
    assert_eq!(found[0]["diagnostics"][0]["code"], "jai-check");
    assert_eq!(found[0]["diagnostics"][0]["source"], "jai");
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

/// Complete at the end of `typed` (the text before the cursor) inside `main`, with a
/// `table` array and a `lo`/`hi` pair in scope.
fn complete_in_main(typed: &str) -> Vec<String> {
    let text = format!(
        "#import \"Basic\";\nmain :: () {{\n    table: [8] int;\n    lo := 0;\n    hi := 3;\n    {typed}"
    );
    let rest = " {\n        print(\"%\", i);\n    }\n}\n";
    let cursor = text.len();
    let text = format!("{text}{rest}");
    let mut s = session();
    s.open(uri(), 1, text.clone()).unwrap();
    let end = Position {
        line: text[..cursor].matches('\n').count() as u32,
        character: text[..cursor].rsplit('\n').next().unwrap().len() as u32,
    };
    s.completion(&uri(), end)
        .unwrap()
        .items
        .into_iter()
        .map(|i| i.label)
        .collect()
}

#[test]
fn completion_works_inside_range_expressions() {
    for typed in [
        "for i: 0..ta",
        "for i: 0 .. ta",
        "for 0..ta",
        "for i: lo..ta",
        "x := lo..ta",
    ] {
        let labels = complete_in_main(typed);
        assert!(labels.iter().any(|l| l == "table"), "{typed}: {labels:?}");
    }
    for typed in [
        "for i: 0..table.co",
        "for i: 0 .. table.co",
        "for 0..table.co",
        "for i: lo..table.co",
    ] {
        let labels = complete_in_main(typed);
        assert!(labels.iter().any(|l| l == "count"), "{typed}: {labels:?}");
        assert!(!labels.iter().any(|l| l == "table"), "{typed}: {labels:?}");
    }
    // The end of a range also offers the names after it.
    let labels = complete_in_main("for i: lo..");
    assert!(labels.iter().any(|l| l == "hi"), "{labels:?}");
}

#[test]
fn signature_help_for_procedure_typed_variables_and_members() {
    let program = "\
My_Func_Type :: #type (a: int, b: string) -> int;
Holder :: struct { callback: (x: float, y: float) -> float; }
twice :: (n: int, label: string) -> int { return n; }
run :: () {
    f: My_Func_Type = twice;
    s: Holder;
    CALL
}
";
    for (call, label, params, active) in [
        ("f(1, ", "f: (s64, string) -> s64", vec!["s64", "string"], 1),
        (
            "s.callback(1.0, ",
            "callback: (float32, float32) -> float32",
            vec!["float32", "float32"],
            1,
        ),
        ("f(", "f: (s64, string) -> s64", vec!["s64", "string"], 0),
    ] {
        let text = program.replace("CALL", call);
        let mut s = session();
        s.open(uri(), 1, text.clone()).unwrap();
        let help = s
            .signature_help(&uri(), at(&text, call, 0, call.len()))
            .unwrap()
            .unwrap_or_else(|| panic!("no signature help for {call}"));
        assert_eq!(help.signatures.len(), 1);
        assert_eq!(help.signatures[0].label, label);
        assert_eq!(help.signatures[0].parameters, params);
        assert_eq!(help.active_parameter, active, "{call}");
    }
}
