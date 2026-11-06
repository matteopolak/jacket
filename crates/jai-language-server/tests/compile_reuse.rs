//! A compile is reused while only buffers it never read change.
use jai_language_server::{DiagnosticSeverity, DocumentUri, Session, TextChange};
use test_support::lsp::{project, session, uri_of};

fn replace(s: &mut Session, uri: &DocumentUri, version: i32, text: &str) {
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

/// The errors of `uri` (lints left out).
fn messages(s: &Session, uri: &DocumentUri) -> Vec<String> {
    s.diagnostics(uri)
        .unwrap()
        .into_iter()
        .filter(|d| d.severity == DiagnosticSeverity::Error)
        .map(|d| d.message)
        .collect()
}

#[test]
fn typing_in_one_program_does_not_recompile_another() {
    let a = "main :: () { x := 1; }\n";
    let b = "main :: () { y := 2; }\n";
    let dir = project("reuse-two", &[("a.jai", a), ("b.jai", b)]);
    let (ua, ub) = (uri_of(&dir.join("a.jai")), uri_of(&dir.join("b.jai")));
    let mut s = session();
    s.open(ua.clone(), 1, a.into()).unwrap();
    s.open(ub.clone(), 1, b.into()).unwrap();
    assert!(messages(&s, &ua).is_empty());
    assert!(messages(&s, &ub).is_empty());
    let (compiled, _) = s.compile_counts();
    for version in 2..6 {
        replace(&mut s, &ua, version, &a.replace('1', &version.to_string()));
        assert!(messages(&s, &ub).is_empty());
        assert_eq!(s.compile_counts().0, compiled, "b.jai never loads a.jai");
    }
    replace(&mut s, &ua, 6, "main :: () { x := ; }\n");
    assert!(!messages(&s, &ua).is_empty());
    assert_eq!(
        s.compile_counts().0,
        compiled + 1,
        "a.jai's own program does"
    );
}

#[test]
fn a_new_buffer_the_program_looks_for_recompiles_it() {
    let main = "#load \"gen.jai\";\nmain :: () { generated(); }\n";
    let dir = project("reuse-new-buffer", &[("main.jai", main)]);
    let um = uri_of(&dir.join("main.jai"));
    let mut s = session();
    s.open(um.clone(), 1, main.into()).unwrap();
    assert!(!messages(&s, &um).is_empty(), "gen.jai does not exist yet");
    // A buffer for the file the `#load` asked for, never saved: the program can load it now.
    let gen_uri = uri_of(&dir.join("gen.jai"));
    s.open(gen_uri, 1, "generated :: () {}\n".into()).unwrap();
    assert_eq!(messages(&s, &um), Vec::<String>::new());
}

#[test]
fn a_buffer_compile_time_code_read_recompiles_the_program() {
    let main = "#import \"Basic\";\n#import \"File\";\nDATA :: #run read_entire_file(\"#DIR/data.txt\");\nmain :: () {\n    #assert DATA == \"one\";\n}\n";
    let dir = project("reuse-run-read", &[("data.txt", "one")]);
    let main = main.replace("#DIR", &jacket_config::slashed(&dir));
    std::fs::write(dir.join("main.jai"), &main).unwrap();
    let um = uri_of(&dir.join("main.jai"));
    let ud = uri_of(&dir.join("data.txt"));
    let mut s = session();
    s.open(um.clone(), 1, main.clone()).unwrap();
    s.open(ud.clone(), 1, "one".into()).unwrap();
    assert_eq!(messages(&s, &um), Vec::<String>::new());
    replace(&mut s, &ud, 2, "two");
    let after = messages(&s, &um);
    assert!(
        after
            .iter()
            .any(|m| m.contains("#assert") || m.contains("assert")),
        "the edited data.txt must reach the `#run`: {after:?}"
    );
}

#[test]
fn missing_load_diagnostics_do_not_observe_other_tests_scratch_directories() {
    use jaic::sema::{FileSystem, NativeFs};
    use std::path::{Path, PathBuf};
    use std::{cell::RefCell, rc::Rc};
    struct ChurningFs {
        shared: PathBuf,
        sibling: RefCell<Option<test_support::TempDir>>,
    }
    impl FileSystem for ChurningFs {
        fn is_native(&self) -> bool {
            true
        }
        fn read(&self, path: &Path) -> Option<Vec<u8>> {
            NativeFs.read(path)
        }
        fn is_file(&self, path: &Path) -> bool {
            NativeFs.is_file(path)
        }
        fn is_dir(&self, path: &Path) -> bool {
            NativeFs.is_dir(path)
        }
        fn canonical(&self, path: &Path) -> PathBuf {
            NativeFs.canonical(path)
        }
        fn canonical_exact(&self, path: &Path) -> Option<PathBuf> {
            NativeFs.canonical_exact(path)
        }
        fn stamp(&self, path: &Path) -> Option<u64> {
            NativeFs.stamp(path)
        }
        fn list_dir(&self, path: &Path) -> Vec<(String, bool)> {
            let result = NativeFs.list_dir(path);
            if NativeFs.canonical(path) == self.shared && self.sibling.borrow().is_none() {
                *self.sibling.borrow_mut() =
                    Some(test_support::TempDir::new("unrelated-help-churn"));
            }
            result
        }
    }
    let main = "#load \"gen.jai\"; main :: () { generated(); }";
    let dir = project("help-lookup-isolation", &[("main.jai", main)]);
    let base = jacket_config::env::TEST_TMPDIR
        .os()
        .map_or_else(std::env::temp_dir, PathBuf::from);
    let fs = Rc::new(ChurningFs {
        shared: base.canonicalize().unwrap(),
        sibling: RefCell::default(),
    });
    let mut env = test_support::lsp::environment();
    env.fs = fs.clone();
    let mut session = Session::with_environment(jai_language_server::Limits::default(), env);
    let uri = uri_of(&dir.join("main.jai"));
    session.open(uri.clone(), 1, main.into()).unwrap();
    assert!(
        !messages(&session, &uri).is_empty(),
        "missing-load diagnostics need a stable private lookup scope"
    );
    assert!(
        fs.sibling.borrow().is_none(),
        "nearby-name help must stay inside this fixture's owned ancestry"
    );
}
