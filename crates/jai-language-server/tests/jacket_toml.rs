//! `jacket.toml` in the editor: its problems, key and path completion, hover, dependency hints,
//! and what a file that does not read does to the Jai files under it.
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

use jai_language_server::{
    CompletionItem, DependencyRequest, DependencySource, DependencyStanding, DiagnosticCode,
    DiagnosticSeverity, DocumentUri, Position, Range, Session, TextChange,
};

fn open(files: &[(&str, &str)], config: &str) -> (test_support::TempDir, Session, DocumentUri) {
    let mut all = files.to_vec();

    all.push(("jacket.toml", config));
    let dir = test_support::lsp::project("jacket-toml", &all);
    let mut s = test_support::lsp::workspace_session(&dir);
    let uri = test_support::lsp::uri_of(&dir.join("jacket.toml"));

    s.open(uri.clone(), 1, config.into()).unwrap();
    (dir, s, uri)
}

fn edit(s: &mut Session, uri: &DocumentUri, version: i32, text: &str) {
    s.change(
        uri,
        version,
        &[TextChange {
            range: None,
            range_length: None,
            text: text.into(),
        }],
    )
    .unwrap();
}

fn config_diagnostics(s: &Session, uri: &DocumentUri) -> Vec<(String, String)> {
    s.diagnostics(uri)
        .unwrap()
        .into_iter()
        .filter(|d| d.code == DiagnosticCode::Config)
        .map(|d| {
            let text = s.document_text(uri).unwrap().to_string();
            (test_support::lsp::text_in_range(&text, d.range), d.message)
        })
        .collect()
}

fn labels(list: &[CompletionItem]) -> Vec<&str> {
    list.iter().map(|i| i.label.as_str()).collect()
}

fn complete(config: &str, extra: &[(&str, &str)]) -> Vec<CompletionItem> {
    let (text, at) = test_support::lsp::cursor(config);
    let (_dir, s, uri) = open(extra, &text);

    s.completion(&uri, at).unwrap().items
}

#[test]
fn an_unknown_key_is_reported_at_the_key_with_the_nearest_known_one() {
    let (_dir, s, uri) = open(&[], "[lint]\nmax-nestng = 3\n");
    let found = config_diagnostics(&s, &uri);

    assert_eq!(found.len(), 1);
    assert_eq!(found[0].0, "max-nestng");
    assert!(found[0].1.contains("did you mean `lint.max-nesting`?"));
    let d = &s.diagnostics(&uri).unwrap()[0];

    assert_eq!(d.severity, DiagnosticSeverity::Error);
}

#[test]
fn a_value_of_the_wrong_type_and_a_bad_lint_rule_are_reported_where_they_are() {
    let (_dir, s, uri) = open(&[], "[lint]\nmax-nesting = \"deep\"\n");

    assert_eq!(config_diagnostics(&s, &uri)[0].0, "\"deep\"");
    let (_dir, s, uri) = open(&[], "[lint.rules]\nunused_varible = \"deny\"\n");
    let found = config_diagnostics(&s, &uri);

    assert_eq!(found.len(), 1);
    assert_eq!(found[0].0, "unused_varible");
}

#[test]
fn a_valid_file_has_no_problems_and_fixing_one_clears_it() {
    let (_dir, mut s, uri) = open(&[], "[lint]\nmax-nestng = 3\n");

    assert_eq!(config_diagnostics(&s, &uri).len(), 1);
    edit(&mut s, &uri, 2, "[lint]\nmax-nesting = 3\n");
    assert!(config_diagnostics(&s, &uri).is_empty());
}

#[test]
fn the_problems_are_published_with_the_file() {
    let (_dir, s, uri) = open(&[], "[lint]\nmax-nestng = 3\n");
    let published = s.publications();
    let (_, version, diagnostics) = published
        .iter()
        .find(|(name, ..)| name == uri.as_str())
        .expect("the file is published");

    assert_eq!(*version, 1);
    assert_eq!(diagnostics.len(), 1);
}

#[test]
fn a_broken_file_stops_the_diagnostics_that_depend_on_it_until_it_is_fixed() {
    let source = "main :: () { x: int = \"text\"; }";
    let (dir, mut s, config) = open(&[("main.jai", source)], "[lint]\nmax-nestng = 3\n");
    let main = test_support::lsp::uri_of(&dir.join("main.jai"));

    s.open(main.clone(), 1, source.into()).unwrap();
    let checked = |s: &Session| {
        s.diagnostics(&main)
            .unwrap()
            .iter()
            .any(|d| d.code == DiagnosticCode::Check)
    };

    assert!(
        !checked(&s),
        "the compile is off while the settings are wrong"
    );
    edit(&mut s, &config, 2, "[lint]\nmax-nesting = 3\n");
    s.save(&config).unwrap();
    assert!(checked(&s), "fixed settings bring the compile back");
}

#[test]
fn a_broken_file_does_not_hide_syntax_errors() {
    let (dir, mut s, _config) = open(&[("main.jai", "")], "[lint]\nmax-nestng = 3\n");
    let main = test_support::lsp::uri_of(&dir.join("main.jai"));

    s.open(main.clone(), 1, "main :: () {".into()).unwrap();
    assert!(
        s.diagnostics(&main)
            .unwrap()
            .iter()
            .any(|d| d.code == DiagnosticCode::Parser)
    );
}

#[test]
fn a_broken_file_does_not_fall_back_to_default_lint_settings() {
    let source = "main :: () { x := 1; }";
    let (dir, mut s, config) = open(&[("main.jai", source)], "[lint]\nmax-nestng = 3\n");
    let main = test_support::lsp::uri_of(&dir.join("main.jai"));

    s.open(main.clone(), 1, source.into()).unwrap();
    let linted = |s: &Session| {
        s.diagnostics(&main)
            .unwrap()
            .iter()
            .any(|d| matches!(d.code, DiagnosticCode::Lint(_)))
    };

    assert!(!linted(&s));
    edit(&mut s, &config, 2, "[lint]\nmax-nesting = 3\n");
    s.save(&config).unwrap();
    assert!(linted(&s), "the same file lints once the settings read");
}

#[test]
fn keys_of_the_table_the_cursor_is_in_are_offered_with_their_description() {
    let items = complete("[lint]\nmax-|\n", &[]);

    assert!(labels(&items).contains(&"max-nesting"));
    assert!(!labels(&items).contains(&"entry"));
    let item = items.iter().find(|i| i.label == "max-nesting").unwrap();

    assert!(
        item.documentation
            .as_deref()
            .unwrap()
            .contains("excessive_nesting")
    );
}

#[test]
fn keys_already_written_are_not_offered_again() {
    let items = complete("[lint]\nmax-nesting = 3\nmax-|\n", &[]);

    assert!(!labels(&items).contains(&"max-nesting"));
    assert!(labels(&items).contains(&"max-arguments"));
}

#[test]
fn top_level_keys_are_the_tables() {
    let items = complete("|\n", &[]);

    assert!(labels(&items).contains(&"lint"));
    assert!(labels(&items).contains(&"dependencies"));
}

#[test]
fn table_headers_complete_the_table_names() {
    let items = complete("[li|]\n", &[]);

    assert!(labels(&items).contains(&"lint"));
    assert!(labels(&items).contains(&"lint.portability"));
    assert!(!labels(&items).contains(&"project"));
}

#[test]
fn a_named_table_offers_the_keys_of_its_kind() {
    let items = complete("[dependencies.jaison]\n|\n", &[]);

    assert!(labels(&items).contains(&"git"));
    assert!(labels(&items).contains(&"min-release-age"));
    let items = complete("[dependencies]\njaison = { git = \"a/b\", | }\n", &[]);

    assert!(labels(&items).contains(&"version"));
    assert!(!labels(&items).contains(&"git"));
}

#[test]
fn entries_are_completed_from_the_jai_files_of_the_project() {
    let items = complete(
        "[project]\nentry = [\"ma|\"]\n",
        &[("main.jai", ""), ("first.jai", ""), ("notes.txt", "")],
    );

    assert!(labels(&items).contains(&"main.jai"));
    assert!(labels(&items).contains(&"first.jai"));
    assert!(!labels(&items).contains(&"notes.txt"));
}

#[test]
fn a_package_module_is_completed_from_the_folders_that_hold_module_jai() {
    let items = complete(
        "[package]\nmodule = \"|\"\n",
        &[("Parser/module.jai", ""), ("docs/readme.md", "")],
    );

    assert!(labels(&items).contains(&"Parser"));
    assert!(!labels(&items).contains(&"docs"));
}

#[test]
fn import_paths_are_completed_from_folders() {
    let items = complete(
        "[project]\nimport-path = [\"|\"]\n",
        &[("modules/Parser/module.jai", "")],
    );

    assert!(labels(&items).contains(&"modules"));
}

#[test]
fn a_key_is_described_on_hover_and_so_is_its_table() {
    let (_dir, s, uri) = open(&[], "[lint]\nmax-nesting = 3\n");
    let key = s
        .hover(
            &uri,
            Position {
                line: 1,
                character: 3,
            },
        )
        .unwrap()
        .unwrap();

    assert!(key.contents.value.contains("excessive_nesting"));
    assert_eq!(
        key.range.start,
        Position {
            line: 1,
            character: 0
        }
    );
    assert_eq!(
        key.range.end,
        Position {
            line: 1,
            character: 11
        }
    );
    let table = s
        .hover(
            &uri,
            Position {
                line: 0,
                character: 2,
            },
        )
        .unwrap()
        .unwrap();

    assert!(table.contents.value.contains("jailint"));
    assert!(
        s.hover(
            &uri,
            Position {
                line: 1,
                character: 14
            }
        )
        .unwrap()
        .is_none()
    );
}

#[derive(Default)]
struct Fake {
    known: RefCell<Option<DependencyStanding>>,
    asked: RefCell<Vec<String>>,
    arrived: RefCell<bool>,
}

impl DependencySource for Fake {
    fn lookup(&self, request: &DependencyRequest) -> Option<DependencyStanding> {
        self.asked.borrow_mut().push(request.name.clone());
        self.known.borrow().clone()
    }

    fn waiting(&self) -> bool {
        self.known.borrow().is_none()
    }

    fn take_arrivals(&self) -> bool {
        std::mem::take(&mut *self.arrived.borrow_mut())
    }
}

const DEPENDENCIES: &str =
    "[dependencies]\njaison = { git = \"someone/jaison\", version = \"^1.2\" }\n";

fn hints(s: &Session, uri: &DocumentUri) -> Vec<(u32, String)> {
    let all = Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: 99,
            character: 0,
        },
    };

    s.inlay_hints(uri, all)
        .unwrap()
        .into_iter()
        .map(|h| (h.position.line, h.label))
        .collect()
}

#[test]
fn a_dependency_shows_its_pin_a_newer_release_and_one_still_waiting() {
    let (_dir, mut s, uri) = open(&[], DEPENDENCIES);
    let source = Rc::new(Fake::default());

    s.set_dependency_source(source.clone());
    assert!(
        hints(&s, &uri).is_empty(),
        "nothing is known until the lookup is back"
    );
    assert_eq!(*source.asked.borrow(), ["jaison"]);
    assert!(s.dependency_lookups_waiting());
    *source.known.borrow_mut() = Some(DependencyStanding {
        pinned: Some("v1.2.0".into()),
        newer: Some("v1.4.0".into()),
        waiting: Some(("v1.5.0".into(), "3 days".into())),
        problem: None,
    });
    assert_eq!(
        hints(&s, &uri),
        [
            (1, "pinned v1.2.0".to_string()),
            (1, "v1.4.0 available".to_string()),
            (1, "v1.5.0 in 3 days".to_string()),
        ]
    );
}

#[test]
fn a_table_dependency_is_hinted_on_its_header() {
    let (_dir, mut s, uri) = open(&[], "[dependencies.jaison]\ngit = \"someone/jaison\"\n");
    let source = Rc::new(Fake::default());

    *source.known.borrow_mut() = Some(DependencyStanding {
        pinned: Some("v1.2.0".into()),
        ..DependencyStanding::default()
    });
    s.set_dependency_source(source);
    assert_eq!(hints(&s, &uri), [(0, "pinned v1.2.0".to_string())]);
}

#[test]
fn an_arrived_lookup_asks_for_the_hints_again() {
    let (_dir, mut s, _uri) = open(&[], DEPENDENCIES);
    let source = Rc::new(Fake::default());

    s.set_dependency_source(source.clone());
    assert!(!s.take_dependency_arrivals());
    *source.arrived.borrow_mut() = true;
    assert!(s.take_dependency_arrivals());
    assert!(!s.take_dependency_arrivals());
}

#[test]
fn a_file_that_does_not_read_gets_no_dependency_lookups() {
    let (_dir, mut s, uri) = open(&[], "[dependencies]\njaison = { gitt = \"a/b\" }\n");
    let source = Rc::new(Fake::default());

    s.set_dependency_source(source.clone());
    assert!(hints(&s, &uri).is_empty());
    assert!(source.asked.borrow().is_empty());
}

#[test]
fn the_request_carries_the_project_folder_and_the_entry() {
    let (dir, mut s, uri) = open(&[], DEPENDENCIES);

    struct Records(RefCell<Vec<(std::path::PathBuf, Option<String>)>>);

    impl DependencySource for Records {
        fn lookup(&self, request: &DependencyRequest) -> Option<DependencyStanding> {
            self.0
                .borrow_mut()
                .push((request.dir.clone(), request.entry.version.clone()));
            None
        }

        fn waiting(&self) -> bool {
            false
        }

        fn take_arrivals(&self) -> bool {
            false
        }
    }

    let records = Rc::new(Records(RefCell::default()));

    s.set_dependency_source(records.clone());
    hints(&s, &uri);
    let seen = records.0.borrow();

    assert_eq!(seen[0].1.as_deref(), Some("^1.2"));
    assert!(Path::new(&seen[0].0).ends_with(dir.file_name().unwrap()));
}

#[cfg(feature = "dependency-hints")]
#[test]
fn a_live_lookup_answers_at_once_and_announces_what_it_found() {
    use jai_language_server::manifest::live::Live;
    // A folder that is no repository: the lookup fails at its first git call, offline.
    let dir = test_support::lsp::project("live-dependencies", &[]);
    let live = Live::default();
    let request = DependencyRequest {
        dir: dir.to_path_buf(),
        name: "jaison".into(),
        entry: jacket_config::Dependency {
            git: "someone/jaison".into(),
            version: Some("^1".into()),
            ..jacket_config::Dependency::default()
        },
        resolver: jacket_config::Resolver::default(),
    };

    assert_eq!(
        live.lookup(&request),
        None,
        "the first look starts the fetch"
    );
    let started = std::time::Instant::now();

    while !live.take_arrivals() {
        assert!(
            started.elapsed().as_secs() < 30,
            "the lookup never finished"
        );
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    assert!(!live.waiting());
    let found = live.lookup(&request).expect("the answer is kept");

    assert!(found.problem.is_some(), "{found:?}");
    assert!(!live.take_arrivals(), "a kept answer starts no new fetch");
}
