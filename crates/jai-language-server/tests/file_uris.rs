//! Original URI/path boundary fixtures; no library API assumptions or program execution.
use jai_language_server::{DocumentUri, Environment, Limits, Position, Session};
use std::path::{Path, PathBuf};

fn session() -> Session {
    Session::with_environment(
        Limits::default(),
        Environment {
            fs: std::rc::Rc::new(jaic::sema::NativeFs),
            options: Box::new(|_| {
                let mut options = jaic::sema::Options::host();
                options.runtime_support = false;
                options.stack_trace = false;
                options
            }),
        },
    )
}

fn at(text: &str, needle: &str) -> Position {
    let byte = text.rfind(needle).unwrap();
    Position {
        line: text[..byte].matches('\n').count() as u32,
        character: text[..byte]
            .rsplit('\n')
            .next()
            .unwrap()
            .encode_utf16()
            .count() as u32,
    }
}

#[test]
fn definition_and_hierarchy_encode_reserved_native_filename_characters() {
    let mut s = session();
    let uri = DocumentUri::parse("file:///file-uris/main%20%23%25%3F%C3%A9.jai").unwrap();
    let text = "target :: () {}\nmain :: () { target(); }";
    s.open(uri.clone(), 1, text.into()).unwrap();
    assert!(s.hover(&uri, at(text, "target()")).unwrap().is_some());
    let definitions = s.definition(&uri, at(text, "target()")).unwrap();
    assert_eq!(definitions.len(), 1, "{definitions:?}");
    assert_eq!(definitions[0].uri, uri.as_str());
    let hierarchy = s
        .prepare_call_hierarchy(&uri, at(text, "target()"))
        .unwrap();
    assert_eq!(hierarchy.len(), 1, "{hierarchy:?}");
    assert_eq!(hierarchy[0].uri, uri.as_str());
}

#[test]
fn load_links_and_cross_file_definitions_use_encoded_destinations() {
    let mut s = session();
    let main = DocumentUri::parse("file:///file-uris/main.jai").unwrap();
    let helper = DocumentUri::parse("file:///file-uris/helper%20%23%25%C3%A9.jai").unwrap();
    s.open(helper.clone(), 1, "answer :: 42;".into()).unwrap();
    let text = "#load \"helper #%é.jai\";\nmain :: () { value := answer; }";
    s.open(main.clone(), 1, text.into()).unwrap();
    assert_eq!(
        s.document_links(&main)
            .unwrap()
            .iter()
            .map(|(_, target)| target.as_str())
            .collect::<Vec<_>>(),
        [helper.as_str()]
    );
    let definitions = s.definition(&main, at(text, "answer")).unwrap();
    assert_eq!(definitions.len(), 1, "{definitions:?}");
    assert_eq!(definitions[0].uri, helper.as_str());
}

#[test]
fn auto_import_keeps_authored_docs_for_encoded_module_directory() {
    let mut s = session();
    let module =
        DocumentUri::parse("file:///file-uris/modules/Documented%20%23%25%C3%A9/module.jai")
            .unwrap();
    s.open(
        module,
        1,
        "/// A visible record.\nVisible_Record :: struct { value: int; }".into(),
    )
    .unwrap();
    let main = DocumentUri::parse("file:///file-uris/main.jai").unwrap();
    let text = "main :: () { value: Visible_; }";
    s.open(main.clone(), 1, text.into()).unwrap();
    let start = at(text, "Visible_");
    let items = s
        .completion(
            &main,
            Position {
                character: start.character + 8,
                ..start
            },
        )
        .unwrap();
    let item = items
        .items
        .iter()
        .find(|item| item.label == "Visible_Record")
        .expect("actual public module export");
    assert!(
        item.additional_edits
            .iter()
            .any(|edit| edit.new_text.contains("#import \"Documented #%é\";")),
        "{item:?}"
    );
    assert!(
        item.documentation
            .as_ref()
            .unwrap()
            .contains("A visible record."),
        "{item:?}"
    );
}

struct ProvenAlias;
impl jaic::sema::FileSystem for ProvenAlias {
    fn read(&self, _: &Path) -> Option<Vec<u8>> {
        None
    }
    fn is_file(&self, _: &Path) -> bool {
        false
    }
    fn is_dir(&self, _: &Path) -> bool {
        false
    }
    fn canonical(&self, path: &Path) -> PathBuf {
        if path == Path::new("/file-uris/Helper.jai") {
            PathBuf::from("/file-uris/helper.jai")
        } else {
            path.to_owned()
        }
    }
}

#[test]
fn filesystem_proven_alias_uses_one_current_open_document_and_edit_identity() {
    let mut s = Session::with_environment(
        Limits::default(),
        Environment {
            fs: std::rc::Rc::new(ProvenAlias),
            options: Box::new(|_| {
                let mut o = jaic::sema::Options::host();
                o.runtime_support = false;
                o.stack_trace = false;
                o
            }),
        },
    );
    let original = DocumentUri::parse("file:///file-uris/Helper.jai").unwrap();
    let alias = DocumentUri::parse("file:///file-uris/helper.jai").unwrap();
    let text = "/* 😀 */ answer :: 42;";
    s.open(original.clone(), 1, text.into()).unwrap();
    assert_eq!(s.source(&alias).as_deref(), Some(text));
    assert!(matches!(
        s.open(alias.clone(), 1, text.into()),
        Err(jai_language_server::Error::AlreadyOpen)
    ));
    let start = at(text, "42");
    s.change(
        &alias,
        2,
        &[jai_language_server::TextChange {
            range: Some(jai_language_server::Range {
                start,
                end: Position {
                    character: start.character + 2,
                    ..start
                },
            }),
            range_length: Some(2),
            text: "99".into(),
        }],
    )
    .unwrap();
    assert_eq!(s.version(&original).unwrap(), 2);
    assert_eq!(s.document_text(&alias).unwrap(), "/* 😀 */ answer :: 99;");
    assert!(s.hover(&alias, at(text, "answer")).unwrap().is_some());
    let main = DocumentUri::parse("file:///file-uris/main.jai").unwrap();
    let main_text = "#load \"helper.jai\";\nmain :: () { value := answer; }";
    s.open(main.clone(), 1, main_text.into()).unwrap();
    let hover = s.hover(&main, at(main_text, "answer")).unwrap().unwrap();
    assert!(hover.contents.value.contains("99"), "{hover:?}");
    assert_eq!(s.document_links(&main).unwrap().len(), 1);
    s.close(&alias).unwrap();
    assert!(s.version(&original).is_err());
}

#[cfg(windows)]
#[test]
fn native_windows_drive_uri_resolves_production_print() {
    // This exercises native Windows paths and real production-library file access when
    // the coordinator runs it on Windows. It is not counted as macOS confirmation.
    let stdlib =
        jaic::canonicalize(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib")).unwrap();
    let configured = stdlib.clone();
    let mut s = Session::with_environment(
        Limits::default(),
        Environment {
            fs: std::rc::Rc::new(jaic::sema::NativeFs),
            options: Box::new(move |_| {
                let mut o = jaic::sema::Options::host();
                o.import_paths = vec![configured.clone()];
                o.preload = Some(configured.join(jaic::PRELOAD_ENTRY));
                o
            }),
        },
    );
    let native = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("native-windows-virtual.jai");
    let uri = DocumentUri::from_path(&native).unwrap();
    let canonical = uri.clone();
    let mut vscode = uri.as_str().to_owned();
    let drive = vscode[8..9].to_ascii_lowercase();
    vscode.replace_range(8..9, &drive);
    assert_eq!(vscode.as_bytes()[9], b':', "a local drive path is required");
    vscode.replace_range(9..10, "%3A");
    let uri = DocumentUri::parse(&vscode).unwrap();
    assert_eq!(uri, canonical);
    assert!(Path::new(uri.path()).is_absolute());
    let text = "main :: () { print(\"hello world\\n\"); }\n#import \"Basic\";";
    s.open(uri.clone(), 1, text.into()).unwrap();
    assert!(
        s.diagnostics(&uri)
            .unwrap()
            .iter()
            .all(|d| d.severity != jai_language_server::DiagnosticSeverity::Error)
    );
    assert!(s.hover(&uri, at(text, "print(")).unwrap().is_some());
    let definitions = s.definition(&uri, at(text, "print(")).unwrap();
    assert!(!definitions.is_empty());
    for definition in definitions {
        let destination = DocumentUri::parse(&definition.uri).unwrap();
        assert!(Path::new(destination.path()).is_absolute());
        assert!(
            jaic::canonicalize(destination.path())
                .unwrap()
                .starts_with(&stdlib)
        );
    }
}
