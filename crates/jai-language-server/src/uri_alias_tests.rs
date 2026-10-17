//! Original fixtures force syntax fallbacks directly, independently of compiler recovery.
use super::*;

struct AliasFs;

impl jaic::sema::FileSystem for AliasFs {
    fn read(&self, _: &Path) -> Option<Vec<u8>> {
        None
    }

    fn is_file(&self, path: &Path) -> bool {
        path == Path::new("/alias/helper.jai")
    }

    fn is_dir(&self, path: &Path) -> bool {
        path == Path::new("/alias")
    }

    fn canonical(&self, path: &Path) -> PathBuf {
        if path == Path::new("/alias/Helper.jai") {
            PathBuf::from("/alias/helper.jai")
        } else {
            path.to_owned()
        }
    }

    fn list_dir(&self, path: &Path) -> Vec<(String, bool)> {
        if self.is_dir(path) {
            vec![("helper.jai".into(), false)]
        } else {
            vec![]
        }
    }
}

fn fixture(text: &str) -> (Session, DocumentUri, DocumentUri) {
    let mut s = Session::with_environment(
        Limits::default(),
        Environment {
            fs: Rc::new(AliasFs),
            options: Box::new(|_| {
                let mut o = jaic::sema::Options::host();
                o.runtime_support = false;
                o.stack_trace = false;
                o
            }),
        },
    );
    let stored = DocumentUri::parse("file:///alias/Helper.jai").unwrap();
    let alias = DocumentUri::parse("file:///alias/helper.jai").unwrap();
    s.open(stored.clone(), 1, text.into()).unwrap();
    (s, stored, alias)
}

fn pos(s: &Session, uri: &DocumentUri, needle: &str) -> Position {
    let d = s.document(uri).unwrap();
    d.index
        .position(&d.text, d.text.rfind(needle).unwrap() + 1)
        .unwrap()
}

#[test]
fn alias_lexical_declaration_and_local_bindings_use_stored_key() {
    let text = "answer :: 42;\nmain :: () { local := 1; value := local; }";
    let (s, stored, alias) = fixture(text);
    for needle in ["answer", "local;"] {
        let rows = s.bindings(&alias, pos(&s, &alias, needle)).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(*rows[0].0, stored);
        assert!(!Session::source_detail(s.document(rows[0].0).unwrap(), rows[0].1).is_empty());
    }
}

#[test]
fn alias_lexical_private_eligibility_is_own_file_only() {
    let text = "#scope_file\nsecret :: 42;\nmain :: () { value := secret; }";
    let (mut s, stored, alias) = fixture(text);
    let rows = s.bindings(&alias, pos(&s, &alias, "secret;")).unwrap();
    assert_eq!(rows.len(), 1, "own-file private binding remains eligible");
    assert_eq!(*rows[0].0, stored);
    let other = DocumentUri::parse("file:///alias/other.jai").unwrap();
    s.open(
        other.clone(),
        1,
        "#load \"helper.jai\";\nmain :: () { value := secret; }".into(),
    )
    .unwrap();
    assert!(
        s.bindings(&other, pos(&s, &other, "secret;"))
            .unwrap()
            .is_empty()
    );
    let unproven = DocumentUri::parse("file:///alias/HELPER.jai").unwrap();
    assert!(s.document(&unproven).is_err(), "no filename case folding");
}

#[test]
fn alias_document_symbol_source_details_use_stored_key() {
    let (s, stored, alias) = fixture("answer :: 42;");
    let symbols = s.document_symbols(&alias).unwrap();
    assert_eq!(
        symbols[0].detail,
        s.document_symbols(&stored).unwrap()[0].detail
    );
    assert_eq!(symbols[0].detail, "answer :: 42");
}

#[test]
fn alias_public_hover_definition_and_completion_force_lexical_fallback() {
    let text = "#scope_file\nsecret :: 42;\nmain :: () { local := 1; value := local; private_value := secret; }";
    let (mut s, stored, alias) = fixture(text);
    s.environment.as_mut().unwrap().options = Box::new(|_| {
        let mut o = jaic::sema::Options::host();
        o.preload = Some(PathBuf::from("/alias/unavailable-prelude.jai"));
        o
    });
    for needle in ["secret ::", "local;"] {
        let at = pos(&s, &alias, needle);
        let hover = s.hover(&alias, at).unwrap().unwrap();
        assert!(
            hover.contents.value.contains("Source syntax declaration."),
            "{hover:?}"
        );
        let definitions = s.definition(&alias, at).unwrap();
        assert_eq!(definitions.len(), 1);
        assert_eq!(definitions[0].uri, stored.as_str());
    }
    let at = pos(&s, &alias, "secret;");
    assert_eq!(s.definition(&alias, at).unwrap().len(), 1);
    let d = s.document(&alias).unwrap();
    let at = d
        .index
        .position(text, text.rfind("secret;").unwrap() + 3)
        .unwrap();
    assert!(
        s.completion(&alias, at)
            .unwrap()
            .items
            .iter()
            .any(|i| i.label == "secret")
    );
}

#[test]
fn alias_assembly_completion_retains_lexical_scope_names() {
    // A genuinely unavailable prelude prevents checked scope metadata: syntax fallback is required.
    let text = "main :: () {\ncount: s64 = 0;\n#asm {\nr: gpr;\npopcnt r, co\n}\n}";
    let (mut s, _, alias) = fixture(text);
    s.environment.as_mut().unwrap().options = Box::new(|_| {
        let mut o = jaic::sema::Options::host();
        o.preload = Some(PathBuf::from("/alias/unavailable-prelude.jai"));
        o
    });
    let d = s.document(&alias).unwrap();
    let at = text.find(", co").unwrap() + 4;
    let items = s
        .completion(&alias, d.index.position(text, at).unwrap())
        .unwrap();
    assert!(items.items.iter().any(|i| i.label == "count"), "{items:?}");
}

#[test]
fn alias_workspace_symbols_exclude_canonical_disk_copy() {
    let (mut s, stored, _) = fixture("Unique_Answer :: 42;");
    s.set_workspace_folders(vec![PathBuf::from("/alias")]);
    let symbols = s.workspace_symbols("Unique_Answer");
    assert_eq!(symbols.len(), 1, "{symbols:?}");
    assert_eq!(symbols[0].location.uri, stored.as_str());
}
