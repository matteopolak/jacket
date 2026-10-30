//! A name used only in an `#if` arm the target skips is described the same however often, and
//! after whatever else, it is asked about.
use jai_language_server::{Limits, Position, Session};
use jaic::target::TargetOs;
use test_support::lsp::{file_uri, modules_environment, uri};

const SOURCE: &str = r#"#import "Basic";

#if OS == .MACOS {
    I :: #import "Input";
}

main :: () {
    #if OS == .MACOS {
        for 1..3 {
            I.update_window_events();
        }
    }
}
"#;

/// The hover at the start of every identifier, in order.
fn hovers(s: &Session) -> Vec<(String, Option<String>)> {
    let mut out = Vec::new();
    for (line, text) in SOURCE.lines().enumerate() {
        let mut from = None;
        for (i, c) in text.char_indices().chain([(text.len(), ' ')]) {
            let word = c.is_alphanumeric() || c == '_';
            match (from, word) {
                (None, true) => from = Some(i),
                (Some(start), false) => {
                    from = None;
                    let name = &text[start..i];
                    let quoted = text[..start].ends_with(['"', '#']);
                    if quoted || name == "for" || name.starts_with(|c: char| c.is_ascii_digit()) {
                        continue;
                    }
                    let at = Position {
                        line: line as u32,
                        character: start as u32,
                    };
                    let hover = s.hover(&uri(), at).ok().flatten();
                    s.definition(&uri(), at).unwrap();
                    out.push((name.to_owned(), hover.map(|h| h.contents.value)));
                }
                _ => {}
            }
        }
    }
    out
}

#[test]
fn an_alias_in_a_skipped_arm_is_described_the_same_each_time() {
    // Linux skips the `.MACOS` arms; asking about the call's name looks the callee up, which
    // must not bind the alias before the hover on it is asked for.
    let mut s = Session::with_environment(
        Limits::default(),
        modules_environment(Some(TargetOs::Linux)),
    );
    s.open(
        file_uri("/lsp-test/jacket.toml"),
        1,
        "[project]\nentry = [\"main.jai\"]\n".into(),
    )
    .unwrap();
    s.set_auto_import(false);
    s.open(uri(), 1, SOURCE.into()).unwrap();
    let _ = s.diagnostics(&uri());

    let first = hovers(&s);
    let alias: Vec<_> = first.iter().filter(|(name, _)| name == "I").collect();
    assert_eq!(alias.len(), 2);
    for (_, hover) in alias {
        assert_eq!(hover.as_deref(), Some("I :: #import \"Input\""));
    }
    assert_eq!(hovers(&s), first, "a second pass over the same text");
}
