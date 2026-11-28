//! A file several entries load is answered from every program that includes it: a declaration
//! the first entry's compile skips (`#if` on a constant that entry sets) is still known where
//! another entry's compile checks it.
use test_support::lsp::{at, open_file, project};

const LIB: &str = "#if MODE == 1 {\n    only_first :: () -> int { return 1; }\n} else {\n    \
    only_second :: () -> int { return 2; }\n}\n";

fn hover_in_lib(name: &str) -> String {
    let dir = project(
        "multi-entry-hover",
        &[
            ("jacket.toml", "[project]\nentry = [\"a.jai\", \"b.jai\"]\n"),
            (
                "a.jai",
                "MODE :: 1;\n#load \"lib.jai\";\nmain :: () { only_first(); }\n",
            ),
            (
                "b.jai",
                "MODE :: 2;\n#load \"lib.jai\";\nmain :: () { only_second(); }\n",
            ),
            ("lib.jai", LIB),
        ],
    );
    let (s, uri, text) = open_file(&dir, "lib.jai");

    s.hover(&uri, at(&text, name))
        .unwrap()
        .unwrap_or_else(|| panic!("{name}: no hover"))
        .contents
        .value
}

#[test]
fn a_declaration_the_first_entry_skips_is_answered_from_the_entry_that_checks_it() {
    let text = hover_in_lib("only_second");

    assert!(text.contains("-> int"), "{text}");
    assert!(!text.contains("Source syntax declaration"), "{text}");
}

#[test]
fn the_first_entrys_own_declaration_is_still_answered() {
    let text = hover_in_lib("only_first");

    assert!(text.contains("-> int"), "{text}");
    assert!(!text.contains("Source syntax declaration"), "{text}");
}
