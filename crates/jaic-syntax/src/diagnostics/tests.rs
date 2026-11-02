use super::*;

#[test]
fn every_code_has_a_name_of_its_own() {
    let mut names: Vec<_> = Code::ALL.iter().map(|c| c.name()).collect();
    names.sort_unstable();
    let before = names.len();
    names.dedup();
    assert_eq!(names.len(), before, "two codes share a name");
    assert!(names.iter().all(|n| {
        !n.is_empty()
            && !n.starts_with('-')
            && !n.ends_with('-')
            && n.bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    }));
}

#[test]
fn names_parse_back_to_their_codes() {
    for &code in Code::ALL {
        assert_eq!(Code::parse(code.name()), Some(code));
    }
    assert_eq!(Code::parse("no-such-code"), None);
}

#[test]
fn leniencies_are_warnings() {
    for code in Code::LENIENCIES {
        assert_eq!(code.severity(), Severity::Warning, "{code}");
    }
}

#[test]
fn explicit_secondary_labels_render_in_plain_snippets_across_files() {
    use crate::source::SourcePath;
    let mut sources = SourceMap::default();
    let first = sources.add(SourcePath::from("origin.jai"), "p := *xs[0];\n".into());
    let second = sources.add(SourcePath::from("use.jai"), "grow(*xs);\np.*;\n".into());
    let d = Diagnostic::new(
        Code::LintFinding,
        Span::new(second, 11, 12),
        "invalid pointer",
    )
    .with_label("used here")
    .with_secondary_label(Span::new(first, 5, 11), "derived here")
    .with_secondary_label(Span::new(second, 0, 9), "invalidated here");
    let actual = d.report(&sources).render_with(crate::render::Style::PLAIN);
    assert_eq!(
        actual,
        "warning[lint-finding]: invalid pointer\n  --> use.jai:2:1\n   |\n 1 | grow(*xs);\n   | --------- invalidated here\n 2 | p.*;\n   | ^ used here\n   |\n  ::: origin.jai:1:6\n   |\n 1 | p := *xs[0];\n   |      ------ derived here\n   |\n"
    );
}


#[test]
fn clearing_fixes_preserves_labels_and_parser_error_size() {
    let mut d = Diagnostic::new(Code::LintFinding, Span::NONE, "message")
        .with_secondary_label(Span::NONE, "retained label")
        .with_replacement("fix", Span::NONE, "replacement");
    d.clear_suggestions();
    assert!(d.suggestions().is_empty());
    assert_eq!(
        d.secondary_labels(),
        [(Span::NONE, "retained label".into())]
    );
    let target = Span::new(crate::source::FileId(0), 3, 8);
    d.map_secondary_spans(|_| target);
    assert_eq!(d.secondary_labels()[0].0, target);
    assert!(std::mem::size_of::<Diagnostic>() < 128);
}
