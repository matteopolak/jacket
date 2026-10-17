//! Auto-import completion: stdlib modules, the project's modules and files, where the edit goes,
//! what is left out, ranking and the setting.
use jai_language_server::{
    CompletionItem, CompletionList, DocumentUri, Environment, JsonSession, Limits, Position,
    Session,
};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

fn environment(os: jaic::sema::TargetOs) -> Environment {
    let stdlib = std::env::var_os("JAIC_STDLIB")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib"));
    Environment {
        fs: std::rc::Rc::new(jaic::sema::NativeFs),
        options: Box::new(move |main| {
            let mut options = jaic::sema::Options::host();
            options.os = os;
            let dir = main.parent().map(PathBuf::from).unwrap_or_default();
            options.import_paths = vec![dir.join("modules"), stdlib.clone()];
            options.preload = Some(stdlib.join(jaic::PRELOAD_ENTRY));
            options
        }),
    }
}

fn session() -> Session {
    Session::with_environment(Limits::default(), environment(jaic::sema::TargetOs::Linux))
}

/// A fresh folder with `files` (path, text) written into it.
fn project(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let dir =
        std::env::temp_dir().join(format!("jailsp-auto-import-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    for (path, text) in files {
        let path = dir.join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }
    dir
}

fn uri_of(path: &Path) -> DocumentUri {
    DocumentUri::from_path(path).unwrap()
}

fn end_of(text: &str, marker: &str) -> Position {
    let byte = text.find(marker).unwrap() + marker.len();
    let prefix = &text[..byte];
    Position {
        line: prefix.matches('\n').count() as u32,
        character: prefix.rsplit('\n').next().unwrap().len() as u32,
    }
}

/// Completion at the end of `marker` in `text`, opened as `path`.
fn complete_at(s: &mut Session, path: &Path, text: &str, marker: &str) -> CompletionList {
    let uri = uri_of(path);
    s.open(uri.clone(), 1, text.into()).unwrap();
    s.completion(&uri, end_of(text, marker)).unwrap()
}

fn auto<'a>(list: &'a CompletionList, label: &str) -> Vec<&'a CompletionItem> {
    list.items
        .iter()
        .filter(|i| i.label == label && !i.additional_edits.is_empty())
        .collect()
}

#[test]
fn public_alias_documentation_resolves_loaded_overloads_and_private_qualifiers() {
    let dir = project(
        "alias-signatures",
        &[
            ("main.jai", "main :: () { meas }"),
            ("modules/Facade/module.jai", "#load \"exports.jai\";"),
            (
                "modules/Facade/exports.jai",
                "measure :: Impl.measure; #scope_file Impl :: #import \"Helper\"; secret :: (value:u8) {}",
            ),
            ("modules/Helper/module.jai", "#load \"overloads.jai\";"),
            (
                "modules/Helper/overloads.jai",
                "measure :: (count:int=7) -> int { return count; } measure :: (text:string) -> int { return text.count; }",
            ),
        ],
    );
    let mut s = session();
    let list = complete_at(&mut s, &dir.join("main.jai"), "main :: () { meas }", "meas");
    let candidates = auto(&list, "measure");
    let facade = candidates
        .iter()
        .find(|item| item.label_description.as_deref() == Some("Facade"))
        .unwrap();
    let doc = facade.documentation.as_deref().unwrap();
    assert!(doc.contains("measure :: (count:int=7) -> int"), "{doc}");
    assert!(doc.contains("measure :: (text:string) -> int"), "{doc}");
    assert!(
        facade.additional_edits[0]
            .new_text
            .contains("#import \"Facade\";")
    );
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn alias_documentation_renames_headers_without_leaking_private_names_or_looping() {
    let dir = project(
        "alias-privacy",
        &[
            ("main.jai", "main :: () { good }"),
            (
                "modules/Facade/module.jai",
                "good :: Impl.exported; broken :: Impl.hidden; cycle :: again; again :: cycle; #scope_file Impl :: #import \"Helper\";",
            ),
            (
                "modules/Helper/module.jai",
                "exported :: hidden; #scope_file hidden :: (value:u8=2) -> u8 { return value; }",
            ),
        ],
    );
    for (prefix, expected, forbidden) in [
        ("good", "good :: (value:u8=2) -> u8", "hidden ::"),
        ("broken", "broken :: Impl.hidden", "value:u8"),
        ("cycle", "cycle :: again", "value:u8"),
    ] {
        let mut s = session();
        let text = format!("main :: () {{ {prefix} }}");
        let list = complete_at(&mut s, &dir.join("main.jai"), &text, prefix);
        let candidates = auto(&list, prefix);
        let item = candidates
            .iter()
            .find(|item| item.label_description.as_deref() == Some("Facade"))
            .unwrap();
        let doc = item.documentation.as_deref().unwrap();
        assert!(doc.contains(expected), "{doc}");
        assert!(!doc.contains(forbidden), "{doc}");
    }
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn unloaded_project_file_alias_keeps_its_own_declaration_environment() {
    let dir = project(
        "alias-load",
        &[
            ("main.jai", "main :: () { public_c }"),
            (
                "utility.jai",
                "public_call :: implementation; implementation :: (number:int=12) -> int { return number; }",
            ),
        ],
    );
    let mut s = session();
    let list = complete_at(
        &mut s,
        &dir.join("main.jai"),
        "main :: () { public_c }",
        "public_c",
    );
    let item = auto(&list, "public_call")[0];
    let doc = item.documentation.as_deref().unwrap();
    assert!(
        doc.contains("public_call :: (number:int=12) -> int"),
        "{doc}"
    );
    assert!(
        item.additional_edits[0]
            .new_text
            .contains("#load \"utility.jai\";")
    );
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn offers_stdlib_names_with_the_import() {
    let mut s = session();
    let text = "main :: () {\n    prin\n}\n";
    let list = complete_at(&mut s, Path::new("/auto-import/a/main.jai"), text, "prin");
    let print = auto(&list, "print");
    assert_eq!(print.len(), 1, "{:#?}", list.items);
    let print = print[0];
    assert_eq!(print.label_description.as_deref(), Some("Basic"));
    assert_eq!(print.detail, "auto-import from Basic");
    assert!(print.sort_text.as_deref().unwrap().starts_with("~print"));
    assert!(
        print
            .documentation
            .as_deref()
            .unwrap()
            .contains("print :: (")
    );
    let edit = &print.additional_edits[0];
    assert_eq!(edit.new_text, "#import \"Basic\";\n\n");
    assert_eq!(
        edit.range.start,
        Position {
            line: 0,
            character: 0
        }
    );
    // Prefix matching: `print_*` names too, nothing that does not start with `prin`.
    assert!(
        list.items
            .iter()
            .all(|i| i.label.to_lowercase().starts_with("prin"))
    );
}

#[test]
fn facade_aliases_show_underlying_overloads_without_exporting_helpers() {
    let dir = project(
        "facade-metadata",
        &[
            ("main.jai", "main :: () { emi }"),
            (
                "modules/Facade/module.jai",
                "emit :: Impl.write;\n#scope_file\nImpl :: #import \"implementation\";",
            ),
            (
                "modules/implementation/module.jai",
                "// Writes a number.\nwrite :: (number: s64) {}\nwrite :: (text: string) {}\n#scope_module\nhidden :: () {}",
            ),
        ],
    );
    let mut s = session();
    let list = complete_at(&mut s, &dir.join("main.jai"), "main :: () { emi }", "emi");
    let items = auto(&list, "emit");
    assert_eq!(items.len(), 1);
    let item = items[0];
    assert_eq!(item.kind, jai_language_server::CompletionKind::Function);
    let doc = item.documentation.as_deref().unwrap();
    assert!(doc.contains("emit :: (number: s64)"), "{doc}");
    assert!(doc.contains("emit :: (text: string)"), "{doc}");
    assert!(doc.contains("Writes a number."), "{doc}");
    assert!(!doc.contains("hidden"));
}

#[test]
fn facade_metadata_alias_cycles_terminate_and_chains_keep_the_public_name() {
    let dir = project(
        "facade-chain",
        &[
            ("main.jai", "main :: () { emi }"),
            (
                "modules/Facade/module.jai",
                "emit :: M.relay;\n#scope_file\nM :: #import \"Middle\";",
            ),
            (
                "modules/Middle/module.jai",
                "relay :: T.target;\n#scope_file\nT :: #import \"Target\";",
            ),
            ("modules/Target/module.jai", "target :: (value: s64) {}"),
            (
                "modules/Cycle/module.jai",
                "emit_cycle :: M.emit_cycle;\n#scope_file\nM :: #import \"Cycle\";",
            ),
        ],
    );
    let mut s = session();
    let list = complete_at(&mut s, &dir.join("main.jai"), "main :: () { emi }", "emi");
    let items = auto(&list, "emit");
    assert_eq!(items.len(), 1);
    assert!(
        items[0]
            .documentation
            .as_deref()
            .unwrap()
            .contains("emit :: (value: s64)")
    );
    assert_eq!(auto(&list, "emit_cycle").len(), 1);
    assert!(auto(&list, "relay").is_empty());
}

#[test]
fn import_goes_after_the_existing_imports() {
    let mut s = session();
    let text = "// A program.\n#import \"Math\";\n#if OS == .WINDOWS {\n    #import \"Windows\";\n}\n\nmain :: () {\n    x := sqrt(2.0);\n    prin\n}\n";
    let list = complete_at(
        &mut s,
        Path::new("/auto-import/b/main.jai"),
        text,
        "    prin",
    );
    let edit = &auto(&list, "print")[0].additional_edits[0];
    // On the line after `#import "Math";`, not inside the `#if` block.
    assert_eq!(
        edit.range.start,
        Position {
            line: 2,
            character: 0
        }
    );
    assert_eq!(edit.new_text, "#import \"Basic\";\n");
}

#[test]
fn leaves_out_imported_modules_and_visible_names() {
    let mut s = session();
    let text = "#import \"Basic\";\nM :: #import \"Math\";\n\nmain :: () {\n    prin\n    M.sqrt(2.0);\n    sqr\n}\n";
    let list = complete_at(
        &mut s,
        Path::new("/auto-import/c/main.jai"),
        text,
        "    prin",
    );
    assert!(auto(&list, "print").is_empty(), "print is visible");
    assert!(list.items.iter().any(|i| i.label == "print"));
    assert!(
        list.items
            .iter()
            .all(|i| i.label_description.as_deref() != Some("Basic"))
    );
    let uri = uri_of(Path::new("/auto-import/c/main.jai"));
    let list = s.completion(&uri, end_of(text, "    sqr")).unwrap();
    // Math is imported under a name: not offered again.
    assert!(
        list.items
            .iter()
            .all(|i| i.label_description.as_deref() != Some("Math")),
        "{:#?}",
        list.items
    );
}

#[test]
fn ranks_after_names_in_scope_and_common_modules_first() {
    let mut s = session();
    let text = "logarithm_table :: 1;\nmain :: () {\n    log\n}\n";
    let list = complete_at(
        &mut s,
        Path::new("/auto-import/d/main.jai"),
        text,
        "    log",
    );
    let local = list
        .items
        .iter()
        .find(|i| i.label == "logarithm_table")
        .unwrap();
    assert!(local.sort_text.is_none());
    let logs = auto(&list, "log");
    let modules: Vec<&str> = logs
        .iter()
        .map(|i| i.label_description.as_deref().unwrap())
        .collect();
    assert_eq!(&modules[..2], ["Basic", "Math"]);
    let sort: Vec<&str> = logs
        .iter()
        .map(|i| i.sort_text.as_deref().unwrap())
        .collect();
    assert!(sort[0] < sort[1]);
    // In-scope names sort by label, before every `~` sort text.
    assert!("logarithm_table" < sort[0]);
}

#[test]
fn needs_two_characters_and_caps_the_list() {
    let mut s = session();
    let text = "main :: () {\n    p\n    ge\n}\n";
    let list = complete_at(&mut s, Path::new("/auto-import/e/main.jai"), text, "    p");
    assert!(list.items.iter().all(|i| i.additional_edits.is_empty()));
    assert!(
        list.is_incomplete,
        "the client must ask again as the word grows"
    );
    let uri = uri_of(Path::new("/auto-import/e/main.jai"));
    let list = s.completion(&uri, end_of(text, "    ge")).unwrap();
    let count = list
        .items
        .iter()
        .filter(|i| !i.additional_edits.is_empty())
        .count();
    assert_eq!(count, 50);
    assert!(list.is_incomplete);
}

#[test]
fn only_modules_for_the_target_os() {
    let text = "main :: () {\n    GetModuleHandl\n}\n";
    let path = Path::new("/auto-import/f/main.jai");
    let mut linux = session();
    let list = complete_at(&mut linux, path, text, "GetModuleHandl");
    assert!(auto(&list, "GetModuleHandleW").is_empty());
    let mut windows = Session::with_environment(
        Limits::default(),
        environment(jaic::sema::TargetOs::Windows),
    );
    let list = complete_at(&mut windows, path, text, "GetModuleHandl");
    assert_eq!(
        auto(&list, "GetModuleHandleW")[0]
            .label_description
            .as_deref(),
        Some("Windows")
    );
}

#[test]
fn the_setting_turns_it_off() {
    let mut s = session();
    s.set_auto_import(false);
    let text = "main :: () {\n    prin\n}\n";
    let list = complete_at(&mut s, Path::new("/auto-import/g/main.jai"), text, "prin");
    assert!(list.items.iter().all(|i| i.additional_edits.is_empty()));
}

#[test]
fn user_module_and_project_files() {
    let dir = project(
        "user",
        &[
            ("modules/Greeting/module.jai", "#load \"parts.jai\";\n"),
            (
                "modules/Greeting/parts.jai",
                "// Says hello.\ngreet_world :: () {}\n#scope_module\ngreet_secret :: () {}\n",
            ),
            (
                "util/strings.jai",
                "shout_text :: (s: string) -> string { return s; }\n",
            ),
            ("loaded.jai", "loaded_helper :: () {}\n"),
            ("tool.jai", "main :: () {}\ntool_only :: () {}\n"),
        ],
    );
    let mut s = session();
    s.set_workspace_folders(vec![dir.clone()]);
    let main = dir.join("main.jai");
    let text = "#import \"Basic\";\n#load \"loaded.jai\";\n\nmain :: () {\n    greet\n    shout\n    tool_o\n}\n";
    let list = complete_at(&mut s, &main, text, "    greet");
    let greet = &auto(&list, "greet_world")[0];
    assert_eq!(greet.label_description.as_deref(), Some("Greeting"));
    assert_eq!(
        greet.additional_edits[0].new_text,
        "#import \"Greeting\";\n"
    );
    assert!(
        greet
            .documentation
            .as_deref()
            .unwrap()
            .contains("Says hello.")
    );
    assert!(list.items.iter().all(|i| i.label != "greet_secret"));
    let uri = uri_of(&main);
    let list = s.completion(&uri, end_of(text, "    shout")).unwrap();
    let shout = &auto(&list, "shout_text")[0];
    assert_eq!(shout.label_description.as_deref(), Some("util/strings.jai"));
    let edit = &shout.additional_edits[0];
    // After the last `#load`.
    assert_eq!(
        edit.range.start,
        Position {
            line: 2,
            character: 0
        }
    );
    assert_eq!(edit.new_text, "#load \"util/strings.jai\";\n");
    // A file with its own `main` is another program.
    let list = s.completion(&uri, end_of(text, "    tool_o")).unwrap();
    assert!(list.items.iter().all(|i| i.label != "tool_only"));
}

#[test]
fn names_of_files_the_program_already_loads_need_no_edit() {
    // `main.jai` is the inferred entry; it loads `a.jai` and `b.jai`. Editing `b.jai` alone, the
    // program is checked from the entry, so the names of `a.jai` are offered without an edit; a file nothing loads gets a `#load`
    // relative to `b.jai`.
    let dir = project(
        "loaded",
        &[
            (
                "main.jai",
                "#load \"src/a.jai\";\n#load \"src/b.jai\";\nmain :: () {}\n",
            ),
            ("src/a.jai", "from_a :: () {}\n"),
            ("src/b.jai", ""),
            ("extra/c.jai", "from_c :: () {}\n"),
        ],
    );
    let mut s = session();
    s.set_workspace_folders(vec![dir.clone()]);
    let text = "use_them :: () {\n    from_\n}\n";
    let list = complete_at(&mut s, &dir.join("src/b.jai"), text, "from_");
    let a = list.items.iter().find(|i| i.label == "from_a").unwrap();
    assert!(a.additional_edits.is_empty());
    let c = &auto(&list, "from_c")[0];
    assert_eq!(
        c.additional_edits[0].new_text,
        "#load \"../extra/c.jai\";\n\n"
    );
}

#[test]
fn jai_toml_names_the_entry_and_import_path() {
    let dir = project(
        "config",
        &[
            (
                "jai.toml",
                "build_files = [\"app/entry.jai\"]\nimport_path = [\"vendor\"]\n",
            ),
            ("app/entry.jai", "#load \"part.jai\";\nmain :: () {}\n"),
            ("app/part.jai", ""),
            ("app/other.jai", "other_thing :: 1;\n"),
            ("vendor/Widgets.jai", "widget_count :: 3;\n"),
            // An inferred entry that the config overrides: it is offered like any file.
            ("main.jai", "root_thing :: 2;\n"),
        ],
    );
    let mut s = session();
    let text = "f :: () {\n    widget_c\n    other_t\n    root_t\n}\n";
    let path = dir.join("app/part.jai");
    let list = complete_at(&mut s, &path, text, "widget_c");
    let widget = &auto(&list, "widget_count")[0];
    assert_eq!(
        widget.additional_edits[0].new_text,
        "#import \"Widgets\";\n\n"
    );
    let uri = uri_of(&path);
    let list = s.completion(&uri, end_of(text, "other_t")).unwrap();
    assert_eq!(
        auto(&list, "other_thing")[0].additional_edits[0].new_text,
        "#load \"other.jai\";\n\n"
    );
    let list = s.completion(&uri, end_of(text, "root_t")).unwrap();
    assert_eq!(
        auto(&list, "root_thing")[0].additional_edits[0].new_text,
        "#load \"../main.jai\";\n\n"
    );
}

#[test]
fn files_changed_on_disk_are_read_again() {
    let dir = project("watch", &[("main.jai", "main :: () {}\n")]);
    let mut s = session();
    s.set_workspace_folders(vec![dir.clone()]);
    let text = "#load \"helpers.jai\";\nmain :: () {\n    fresh_\n}\n";
    let main = dir.join("main.jai");
    let list = complete_at(&mut s, &main, text, "fresh_");
    assert!(list.items.iter().all(|i| i.label != "fresh_name"));
    std::fs::write(dir.join("helpers.jai"), "fresh_name :: 1;\n").unwrap();
    s.file_changed(&dir.join("helpers.jai"), true);
    let list = s
        .completion(&uri_of(&main), end_of(text, "fresh_"))
        .unwrap();
    let fresh = list.items.iter().find(|i| i.label == "fresh_name").unwrap();
    assert!(fresh.additional_edits.is_empty(), "main.jai loads it");
}

fn request(session: &mut JsonSession, id: i64, method: &str, params: Value) -> Value {
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

#[test]
fn json_protocol_and_settings() {
    let mut s =
        JsonSession::with_environment(Limits::default(), environment(jaic::sema::TargetOs::Linux));
    request(
        &mut s,
        1,
        "initialize",
        json!({ "initializationOptions": { "completion": { "autoImport": true } } }),
    );
    notify(&mut s, "initialized", json!({}));
    let uri = "file:///auto-import/json/main.jai";
    notify(
        &mut s,
        "textDocument/didOpen",
        json!({ "textDocument": { "uri": uri, "languageId": "jai", "version": 1, "text": "main :: () {\n    prin\n}\n" } }),
    );
    let complete = |s: &mut JsonSession, id| {
        request(
            s,
            id,
            "textDocument/completion",
            json!({ "textDocument": { "uri": uri }, "position": { "line": 1, "character": 8 } }),
        )
    };
    let result = complete(&mut s, 2);
    let print = result["result"]["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["label"] == "print" && i["additionalTextEdits"].is_array())
        .unwrap()
        .clone();
    assert_eq!(print["labelDetails"]["description"], "Basic");
    assert_eq!(
        print["additionalTextEdits"][0]["newText"],
        "#import \"Basic\";\n\n"
    );
    assert_eq!(
        print["additionalTextEdits"][0]["range"]["start"],
        json!({ "line": 0, "character": 0 })
    );
    assert!(print["sortText"].as_str().unwrap().starts_with('~'));
    notify(
        &mut s,
        "workspace/didChangeConfiguration",
        json!({ "settings": { "jai": { "completion": { "autoImport": false } } } }),
    );
    let result = complete(&mut s, 3);
    assert!(
        result["result"]["items"]
            .as_array()
            .unwrap()
            .iter()
            .all(|i| i.get("additionalTextEdits").is_none())
    );
}

#[test]
fn workspace_symbols_include_project_files_that_are_not_open() {
    let main = "#load \"util/shapes.jai\";\nmain :: () {}\n";
    let shapes = "Circle :: struct { radius: float; }\ncircle_area :: (c: Circle) -> float { return c.radius; }\n#scope_file\ncircle_secret := 3;\n";
    let dir = project(
        "symbols",
        &[
            ("main.jai", main),
            ("util/shapes.jai", shapes),
            ("util/notes.jai", "Circle_Note :: 1;\n"),
        ],
    );
    let mut s = session();
    s.set_workspace_folders(vec![dir.clone()]);
    s.open(uri_of(&dir.join("main.jai")), 1, main.into())
        .unwrap();
    let names = |s: &Session, query: &str| -> Vec<(String, String, u32)> {
        let mut found: Vec<_> = s
            .workspace_symbols(query)
            .into_iter()
            .map(|x| {
                (
                    x.name,
                    x.location.uri.rsplit('/').next().unwrap().to_string(),
                    x.location.range.start.line,
                )
            })
            .collect();
        found.sort();
        found
    };
    assert_eq!(
        names(&s, "CIRCLE"),
        [
            ("Circle".into(), "shapes.jai".into(), 0),
            ("Circle_Note".into(), "notes.jai".into(), 0),
            ("circle_area".into(), "shapes.jai".into(), 1),
            ("circle_secret".into(), "shapes.jai".into(), 3),
        ]
    );
    // Open documents are not listed twice, and the limit holds.
    s.open(uri_of(&dir.join("util/shapes.jai")), 1, shapes.into())
        .unwrap();
    assert_eq!(names(&s, "circle").len(), 4);
    let mut small = Session::with_environment(
        Limits {
            symbols: 2,
            ..Limits::default()
        },
        environment(jaic::sema::TargetOs::Linux),
    );
    small.set_workspace_folders(vec![dir.clone()]);
    small
        .open(uri_of(&dir.join("main.jai")), 1, main.into())
        .unwrap();
    assert_eq!(small.workspace_symbols("circle").len(), 2);
    let _ = std::fs::remove_dir_all(&dir);
}

#[cfg(unix)]
#[test]
fn filesystem_aliases_keep_relative_loads_and_refresh_missing_files() {
    let dir = project(
        "canonical-alias",
        &[
            ("main.jai", "#load \"loaded.jai\"; main :: () {}"),
            ("loaded.jai", "loaded_name :: 1;"),
            ("extra.jai", "extra_name :: 2;"),
        ],
    );
    let alias = dir.with_extension("alias");
    let _ = std::fs::remove_file(&alias);
    std::os::unix::fs::symlink(&dir, &alias).unwrap();
    let mut s = session();
    s.set_workspace_folders(vec![dir.clone()]);
    let text =
        "#load \"loaded.jai\"; #load \"new.jai\"; main :: () { loaded_n; extra_n; fresh_n; }";
    let main = alias.join("main.jai");
    let list = complete_at(&mut s, &main, text, "loaded_n");
    assert!(
        list.items
            .iter()
            .find(|i| i.label == "loaded_name")
            .unwrap()
            .additional_edits
            .is_empty()
    );
    let uri = DocumentUri::from_path(&main).unwrap();
    let list = s.completion(&uri, end_of(text, "extra_n")).unwrap();
    assert!(
        auto(&list, "extra_name")[0].additional_edits[0]
            .new_text
            .contains("#load \"extra.jai\";")
    );
    let list = s.completion(&uri, end_of(text, "fresh_n")).unwrap();
    assert!(!list.items.iter().any(|i| i.label == "fresh_name"));
    std::fs::write(dir.join("new.jai"), "fresh_name :: 3;").unwrap();
    s.file_changed(&dir.join("new.jai"), true);
    let list = s.completion(&uri, end_of(text, "fresh_n")).unwrap();
    assert!(
        list.items
            .iter()
            .find(|i| i.label == "fresh_name")
            .unwrap()
            .additional_edits
            .is_empty()
    );
    std::fs::write(dir.join("extra.jai"), "extra_changed :: 4;").unwrap();
    s.file_changed(&alias.join("extra.jai"), false);
    let list = s.completion(&uri, end_of(text, "extra_n")).unwrap();
    assert!(!list.items.iter().any(|i| i.label == "extra_name"));
    std::fs::remove_file(alias).unwrap();
    std::fs::remove_dir_all(dir).unwrap();
}
