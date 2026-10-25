//! A module alias names its module wherever it is used, also in a body the checker could not
//! finish (here the Windows procedures on a host without them): the answer must not depend on
//! which statements happened to be checked, or the editor's answers change while typing.
use jai_language_server::{Position, Range, TextChange};
mod common;
use common::{file_uri, session};

fn probe() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let main = root.join("Input/tests/windows-device-decoding.jai");
    let module = root.join("internal/input/windows-devices/module.jai");
    let (main, module) = (main.canonicalize().unwrap(), module.canonicalize().unwrap());
    let windows = root.join("Windows/module.jai").canonicalize().unwrap();
    let text = std::fs::read_to_string(&module).unwrap();
    let mut session = session();
    let config = file_uri(main.parent().unwrap().join("jacket.toml").to_str().unwrap());
    session
        .open(
            config,
            1,
            "[project]\nentry = [\"windows-device-decoding.jai\"]\n".into(),
        )
        .unwrap();
    let main_uri = file_uri(main.to_str().unwrap());
    session
        .open(main_uri, 1, std::fs::read_to_string(&main).unwrap())
        .unwrap();
    let uri = file_uri(module.to_str().unwrap());
    session.open(uri.clone(), 1, text.clone()).unwrap();

    let check = |session: &jai_language_server::Session, text: &str, label: &str| {
        let _ = session.diagnostics(&uri);
        let mut uses = 0;
        for (line, content) in text.lines().enumerate() {
            for (column, _) in content.match_indices("W.") {
                if column > 0 && content.as_bytes()[column - 1].is_ascii_alphanumeric() {
                    continue;
                }
                let position = Position {
                    line: line as u32,
                    character: column as u32,
                };
                let found = session.definition(&uri, position).unwrap();
                assert_eq!(found.len(), 1, "{label}: definition of `W` at {position:?}");
                assert!(
                    found[0]
                        .uri
                        .ends_with(windows.file_name().unwrap().to_str().unwrap())
                        && found[0].uri.contains("/Windows/"),
                    "{label}: `W` at {position:?} went to {}",
                    found[0].uri
                );
                let hover = session.hover(&uri, position).unwrap().unwrap();
                assert_eq!(
                    hover.contents.value, "W :: #import \"Windows\"",
                    "{label}: hover of `W` at {position:?}"
                );
                uses += 1;
            }
        }
        assert!(uses > 20, "{label}: only {uses} uses found");
    };
    check(&session, &text, "as written");
    // A stray statement in the first procedure fails that body; the others must not change.
    let at = Position {
        line: 10,
        character: 0,
    };
    session
        .change(
            &uri,
            2,
            &[TextChange {
                range: Some(Range {
                    start: at,
                    end: at,
                }),
                range_length: None,
                text: "    __\n".into(),
            }],
        )
        .unwrap();
    let mut edited: Vec<&str> = text.lines().collect();
    edited.insert(10, "    __");
    check(&session, &edited.join("\n"), "after a stray statement");
}

#[test]
fn module_alias_answers_do_not_depend_on_the_checked_statements() {
    // The compiler thread of `jailsp` has a large stack; the parser's depth budget needs it.
    std::thread::Builder::new()
        .stack_size(1 << 30)
        .spawn(probe)
        .unwrap()
        .join()
        .unwrap();
}
