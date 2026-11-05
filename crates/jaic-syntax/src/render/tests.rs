use super::*;

const SOURCE: &str = "main :: () {\n    counter := 1;\n    print(\"%\\n\", countr);\n}\n";

fn leak_path(path: &str) -> &'static SourcePath {
    Box::leak(Box::new(SourcePath::from(path)))
}

fn label(text: &str, needle: &str, message: Option<&str>) -> Label<'static> {
    let text: &'static str = Box::leak(text.to_string().into_boxed_str());
    let start = text.find(needle).unwrap();
    Label {
        path: leak_path("u.jai"),
        text,
        start,
        end: start + needle.len(),
        message: message.map(str::to_string),
        lines: None,
    }
}

fn unknown_name() -> Report<'static> {
    let primary = label(SOURCE, "countr)", Some("not found in this scope"));
    let at = primary.start;
    let mut report = Report::new(Code::UnknownIdentifier, "unknown identifier `countr`");
    report.primary = Some(Label {
        end: at + 6,
        ..primary
    });
    report.help.push(Help {
        message: "a similar name exists: `counter`".into(),
        fix: Some(Fix {
            text: SOURCE,
            edits: vec![(at, at + 6, "counter".into())],
            lines: None,
        }),
    });
    report.notes.push("names are case-sensitive".into());
    report
}

const ASCII_STYLE: Style = Style {
    layout: Layout::Ascii,
    color: false,
};

const UNICODE_STYLE: Style = Style {
    layout: Layout::Unicode,
    color: false,
};

/// A lint finding shows its rule where other diagnostics show their code.
#[test]
fn a_lint_finding_is_named_by_its_rule() {
    let mut report = unknown_name();
    report.code = Code::LintFinding;
    report.severity = Code::LintFinding.severity();
    report.name = Some("magic_number");
    let text = report.render_with(Style::PLAIN);
    assert!(
        text.starts_with("u.jai:3:18: warning[magic_number]: unknown identifier"),
        "{text}"
    );
}

#[test]
fn plain_compact_is_the_long_standing_layout() {
    let text = unknown_name().render_with(Style::PLAIN);
    assert_eq!(
        text,
        "u.jai:3:18: error[unknown-identifier]: unknown identifier `countr`\n\
         \x20       print(\"%\\n\", countr);\n\
         \x20                    ^^^^^^\n\
         note: names are case-sensitive\n\
         help: a similar name exists: `counter`\n"
    );
}

#[test]
fn msvc_layout_is_one_located_line_per_item_and_never_coloured() {
    let style = Style {
        layout: Layout::Msvc,
        color: true,
    };
    assert_eq!(
        unknown_name().render_with(style),
        "u.jai(3,18): error[unknown-identifier]: unknown identifier `countr`\n\
         u.jai(3,18): note: names are case-sensitive\n\
         u.jai(3,18): note: help: a similar name exists: `counter`\n"
    );
    let bare = Report::new(Code::TypeMismatch, "no place").render_with(style);
    assert_eq!(bare, "error[type-mismatch]: no place\n");
}

#[test]
fn ascii_block_has_line_numbers_context_labels_and_the_fix() {
    let text = unknown_name().render_with(ASCII_STYLE);
    assert_eq!(
        text,
        "error[unknown-identifier]: unknown identifier `countr`\n\
         \x20 --> u.jai:3:18\n\
         \x20  |\n\
         \x202 |     counter := 1;\n\
         \x203 |     print(\"%\\n\", countr);\n\
         \x20  |                  ^^^^^^ not found in this scope\n\
         \x204 | }\n\
         \x20  |\n\
         help: a similar name exists: `counter`\n\
         \x20  |\n\
         \x202       counter := 1;\n\
         \x203 ~     print(\"%\\n\", counter);\n\
         \x204   }\n\
         \x20  |\n\
         \x20  = note: names are case-sensitive\n"
    );
}

#[test]
fn unicode_block_draws_boxes() {
    let text = unknown_name().render_with(UNICODE_STYLE);
    assert_eq!(
        text,
        "error[unknown-identifier]: unknown identifier `countr`\n\
         \x20  ╭─[u.jai:3:18]\n\
         \x202 │     counter := 1;\n\
         \x203 │     print(\"%\\n\", countr);\n\
         \x20  ·                  ━━━━━━ not found in this scope\n\
         \x204 │ }\n\
         \x20  ╰─\n\
         help: a similar name exists: `counter`\n\
         \x20  │\n\
         \x202       counter := 1;\n\
         \x203 ~     print(\"%\\n\", counter);\n\
         \x204   }\n\
         \x20  │\n\
         \x20  = note: names are case-sensitive\n"
    );
}

#[test]
fn colour_marks_severity_gutter_and_labels() {
    let text = unknown_name().render_with(Style {
        layout: Layout::Unicode,
        color: true,
    });
    assert!(
        text.starts_with("\x1b[1;31merror[unknown-identifier]\x1b[0m\x1b[1m: unknown identifier"),
        "{text}"
    );
    assert!(text.contains("\x1b[34m│\x1b[0m"), "{text}");
    assert!(text.contains("\x1b[1;31m━━━━━━\x1b[0m"), "{text}");
    assert!(!unknown_name().render_with(UNICODE_STYLE).contains('\x1b'));
}

#[test]
fn secondary_labels_and_other_files() {
    let source = "x: int = 1;\nmain :: () {\n    x = \"text\";\n}\n";
    let mut report = Report::new(
        Code::TypeMismatch,
        "type mismatch: expected `int`, found `string`",
    );
    report.primary = Some(label(source, "\"text\"", Some("this is a `string`")));
    report
        .secondary
        .push(label(source, "int", Some("`x` is declared as `int` here")));
    let mut other = label("helper :: () {}\n", "helper", Some("also here"));
    other.path = leak_path("other.jai");
    report.secondary.push(other);
    let text = report.render_with(ASCII_STYLE);
    assert_eq!(
        text,
        "error[type-mismatch]: type mismatch: expected `int`, found `string`\n\
         \x20 --> u.jai:3:9\n\
         \x20  |\n\
         \x201 | x: int = 1;\n\
         \x20  |    --- `x` is declared as `int` here\n\
         \x202 | main :: () {\n\
         \x203 |     x = \"text\";\n\
         \x20  |         ^^^^^^ this is a `string`\n\
         \x204 | }\n\
         \x20  |\n\
         \x20 ::: other.jai:1:1\n\
         \x20  |\n\
         \x201 | helper :: () {}\n\
         \x20  | ------ also here\n\
         \x20  |\n"
    );
    // The plain compact layout keeps one item per place.
    let plain = report.render_with(Style::PLAIN);
    assert!(
        plain.contains("u.jai:1:4: note: `x` is declared as `int` here\n"),
        "{plain}"
    );
}

#[test]
fn multi_line_spans_get_a_connector() {
    let source = "f :: () {\n    a := 1;\n    b := 2;\n}\nmain :: () {}\n";
    let mut report = Report::new(Code::ShadowsBuiltinType, "`f` is never used");
    let start = 0;
    let end = source.find("}\n").unwrap() + 1;
    report.primary = Some(Label {
        path: leak_path("m.jai"),
        text: source,
        start,
        end,
        message: Some("this procedure".into()),
        lines: None,
    });
    let text = report.render_with(UNICODE_STYLE);
    assert_eq!(
        text,
        "warning[shadows-builtin-type]: `f` is never used\n\
         \x20  ╭─[m.jai:1:1]\n\
         \x201 │ ╭ f :: () {\n\
         \x202 │ │     a := 1;\n\
         \x203 │ │     b := 2;\n\
         \x204 │ │ }\n\
         \x20  · ╰── this procedure\n\
         \x205 │   main :: () {}\n\
         \x20  ╰─\n"
    );
    let ascii = report.render_with(ASCII_STYLE);
    assert!(ascii.contains(" 1 | / f :: () {\n"), "{ascii}");
    assert!(ascii.contains("   | \\__ this procedure\n"), "{ascii}");
}

#[test]
fn long_spans_show_their_edges_and_long_lines_are_cut() {
    let mut source = String::from("f :: () {\n");
    for i in 0..20 {
        source += &format!("    x{i} := {i};\n");
    }
    source += "}\n";
    let mut report = Report::new(Code::TypeMismatch, "long");
    report.primary = Some(Label {
        path: leak_path("l.jai"),
        text: Box::leak(source.clone().into_boxed_str()),
        start: 0,
        end: source.len() - 1,
        message: None,
        lines: None,
    });
    let text = report.render_with(ASCII_STYLE);
    assert!(text.contains("\n   ...\n"), "{text}");
    assert!(!text.contains("x10"), "{text}");
    assert!(text.contains("x19"), "{text}");

    let long = format!("x := {}needle{};\n", "a".repeat(200), "b".repeat(200));
    let mut report = Report::new(Code::TypeMismatch, "cut");
    report.primary = Some(label(&long, "needle", None));
    let text = report.render_with(UNICODE_STYLE);
    let line = text.lines().find(|l| l.contains("needle")).unwrap();
    assert!(line.chars().count() < 150, "{line}");
    assert!(line.contains('…'), "{line}");
    let marks = text.lines().find(|l| l.contains('━')).unwrap();
    assert_eq!(
        marks.find('━').map(|i| marks[..i].chars().count()),
        line.find("needle").map(|i| line[..i].chars().count())
    );
}

#[test]
fn tabs_are_expanded_outside_the_plain_layout() {
    let source = "main :: () {\n\tx := y;\n}\n";
    let mut report = Report::new(Code::TypeMismatch, "unknown identifier `y`");
    report.primary = Some(label(source, "y;", None));
    report.primary.as_mut().unwrap().end -= 1;
    let text = report.render_with(ASCII_STYLE);
    assert!(
        text.contains(" 2 |     x := y;\n   |          ^\n"),
        "{text}"
    );
    let plain = report.render_with(Style::PLAIN);
    assert!(plain.contains("    \tx := y;\n    \t     ^\n"), "{plain}");
}

#[test]
fn reports_without_a_place() {
    let report = Report::new(Code::TypeMismatch, "file `nope.jai` does not exist")
        .help("did you mean `nop.jai`?");
    for style in [Style::PLAIN, ASCII_STYLE, UNICODE_STYLE] {
        assert_eq!(
            report.render_with(style),
            "error[type-mismatch]: file `nope.jai` does not exist\nhelp: did you mean `nop.jai`?\n"
        );
    }
}

#[test]
fn detection_follows_terminal_flags_and_environment() {
    let env = |pairs: &'static [(&'static str, &'static str)]| {
        move |key: &str| {
            pairs
                .iter()
                .find(|(k, _)| *k == key)
                .map(|(_, v)| v.to_string())
        }
    };
    let utf8 = env(&[("LANG", "en_US.UTF-8"), ("TERM", "xterm-256color")]);
    let auto = ColorChoice::Auto;
    // Pipes: the plain layout, no colour.
    assert_eq!(detect_with(auto, false, utf8), Style::PLAIN);
    // A UTF-8 terminal.
    assert_eq!(
        detect_with(auto, true, utf8),
        Style {
            layout: Layout::Unicode,
            color: true
        }
    );
    // No UTF-8 locale: ASCII.
    assert_eq!(
        detect_with(auto, true, env(&[("TERM", "xterm")])).layout,
        Layout::Ascii
    );
    // NO_COLOR, whatever its value; --color always wins over it.
    assert!(!detect_with(auto, true, env(&[("NO_COLOR", ""), ("TERM", "xterm")])).color);
    assert!(detect_with(ColorChoice::Always, false, env(&[("NO_COLOR", "1")])).color);
    assert!(!detect_with(ColorChoice::Never, true, utf8).color);
    // Forcing colour into a pipe.
    assert!(detect_with(auto, false, env(&[("FORCE_COLOR", "1")])).color);
    assert!(detect_with(auto, false, env(&[("CLICOLOR_FORCE", "1")])).color);
    assert!(!detect_with(auto, false, env(&[("FORCE_COLOR", "0")])).color);
    // A dumb terminal.
    assert_eq!(
        detect_with(auto, true, env(&[("TERM", "dumb")])),
        Style::PLAIN
    );
    // The layout override.
    assert_eq!(
        detect_with(auto, false, env(&[("JAIC_DIAGNOSTICS", "unicode")])).layout,
        Layout::Unicode
    );
    assert_eq!(
        detect_with(
            auto,
            true,
            env(&[("JAIC_DIAGNOSTICS", "plain"), ("LANG", "C.UTF-8")])
        )
        .layout,
        Layout::Plain
    );
    assert_eq!(ColorChoice::parse("sometimes"), None);
}

#[test]
fn reports_from_text_split_help_and_note_lines() {
    let report = Report::from_text(
        Code::CliFailure,
        "linking failed\nld: oops\nhelp: install it\nnote: why\n",
    );
    assert_eq!(report.message, "linking failed\nld: oops");
    assert_eq!(report.help.len(), 1);
    assert_eq!(report.help[0].message, "install it");
    assert_eq!(report.notes, ["why"]);
    assert_eq!(
        report.render_with(Style::PLAIN),
        "error[cli-failure]: linking failed\nld: oops\nnote: why\nhelp: install it\n"
    );
}

/// `with_style` applies to its own thread and ends with the closure; another thread keeps the
/// process-wide style meanwhile.
#[test]
fn scoped_style_stays_on_its_thread() {
    let unicode = Style {
        layout: Layout::Unicode,
        color: true,
    };
    let global = style();
    with_style(unicode, || {
        assert_eq!(style(), unicode);
        assert_eq!(std::thread::spawn(style).join().unwrap(), global);
        assert!(unknown_name().render().contains('\x1b'));
    });
    assert_eq!(style(), global);
}

/// Fix previews retain neutral boundaries and each side's line numbers.
#[test]
fn fix_diff_keeps_neutral_context() {
    for newline in ["\n", "\r\n", "\r"] {
        let text = ["keep;", "dead;", "stay;", "old;", "end;"].join(newline);
        let dead = text.find("dead;").unwrap();
        let old = text.find("old;").unwrap();
        let edits = [
            (dead - newline.len(), dead + 5, String::new()),
            (old, old + 4, "new;".into()),
        ];
        assert_eq!(
            changed_lines(&text, &LineIndex::new(&text), &edits),
            vec![
                ("1".into(), ' ', "keep;".into()),
                ("2".into(), '-', "dead;".into()),
                ("3".into(), ' ', "stay;".into()),
                ("3".into(), '~', "new;".into()),
                ("5".into(), ' ', "end;".into()),
            ]
        );
    }
    // Removing the separator before the final line also removes keep's final newline.
    let text = "keep;\ndead;";
    assert_eq!(
        changed_lines(
            text,
            &LineIndex::new(text),
            &[(5, text.len(), String::new())]
        ),
        vec![
            ("1".into(), '~', "keep;".into()),
            ("2".into(), '-', "dead;".into())
        ]
    );
}

#[test]
fn fix_diff_handles_inline_insertions_and_blank_lines() {
    let cases = [
        (
            "a\nb\nc",
            vec![(0, 5, "A\nc\nb".into())],
            vec![("1", '~', "A"), ("2", '~', "c"), ("3", '~', "b")],
        ),
        ("a;", vec![(2, 2, "\n".into())], vec![("1", '~', "a;")]),
        (
            "a; dead; b;",
            vec![(3, 9, String::new())],
            vec![("1", '~', "a; b;")],
        ),
        (
            "a;\nb;",
            vec![(3, 3, "new;\n".into())],
            vec![("1", ' ', "a;"), ("2", '+', "new;"), ("2", ' ', "b;")],
        ),
        (
            "a;\n\nb;",
            vec![(3, 4, String::new())],
            vec![("1", ' ', "a;"), ("2", '-', ""), ("3", ' ', "b;")],
        ),
        (
            "dead;",
            vec![(0, 5, String::new())],
            vec![("1", '-', "dead;")],
        ),
        (
            "a;\nb;",
            vec![(3, 5, String::new())],
            vec![("1", ' ', "a;"), ("2", '-', "b;")],
        ),
    ];
    for (text, edits, expected) in cases {
        assert_eq!(
            changed_lines(text, &LineIndex::new(text), &edits),
            expected
                .into_iter()
                .map(|(n, mark, text)| (n.into(), mark, text.into()))
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn warning_fix_deletion_marker_is_red() {
    let mut report = unknown_name();
    report.severity = Severity::Warning;
    let start = SOURCE.find("    counter").unwrap();
    let end = SOURCE[start..].find('\n').unwrap() + start + 1;
    report.help[0].fix.as_mut().unwrap().edits = vec![(start, end, String::new())];
    let text = report.render_with(Style {
        layout: Layout::Ascii,
        color: true,
    });
    assert!(text.contains("\x1b[1;33mwarning"), "{text}");
    assert!(text.contains("\x1b[31m-\x1b[0m"), "{text}");
}

/// Separate edits are anchored to their source lines; unchanged rows have a blank marker.
#[test]
fn fix_context_preserves_edit_order_and_elides_distant_gaps() {
    for newline in ["\n", "\r\n", "\r"] {
        let text = [
            "before", "drop", "keep", "old", "middle", "far1", "far2", "far3", "after",
        ]
        .join(newline);
        let drop = text.find("drop").unwrap();
        let old = text.find("old").unwrap();
        let after = text.find("after").unwrap();
        let edits = [
            (drop, drop + 4 + newline.len(), String::new()),
            (old, old + 3, "new".into()),
            (after, after, format!("insert{newline}")),
        ];
        assert_eq!(
            changed_lines(&text, &LineIndex::new(&text), &edits),
            vec![
                ("1".into(), ' ', "before".into()),
                ("2".into(), '-', "drop".into()),
                ("3".into(), ' ', "keep".into()),
                ("3".into(), '~', "new".into()),
                ("5".into(), ' ', "middle".into()),
                ("...".into(), ' ', String::new()),
                ("8".into(), ' ', "far3".into()),
                ("8".into(), '+', "insert".into()),
                ("9".into(), ' ', "after".into()),
            ]
        );
    }
}

#[test]
fn empty_neutral_fix_context_has_no_trailing_whitespace() {
    let source = "before;\n\ndead;\n\nafter;";
    let mut report = Report::new(Code::TypeMismatch, "remove dead statement");
    report.compact = false;
    report.primary = Some(label(source, "dead;", None));
    let start = source.find("dead;").unwrap();
    report.help.push(Help {
        message: "remove it".into(),
        fix: Some(Fix {
            text: source,
            edits: vec![(start, start + 6, String::new())],
            lines: None,
        }),
    });
    let rendered = report.render_with(Style::PLAIN);
    let help = rendered.split_once("help:").unwrap().1;
    assert!(!help.lines().any(|line| line.ends_with(' ')), "{help}");
}

#[test]
fn fix_context_keeps_the_existing_changed_row_limit() {
    let text = (0..20)
        .map(|n| format!("old{n}"))
        .collect::<Vec<_>>()
        .join("\n");
    let fixed = (0..20)
        .map(|n| format!("new{n}"))
        .collect::<Vec<_>>()
        .join("\n");
    let rows = changed_lines(&text, &LineIndex::new(&text), &[(0, text.len(), fixed)]);
    assert_eq!(rows.len(), MAX_FIX_LINES + 1);
    assert_eq!(rows.last(), Some(&("...".into(), '~', String::new())));
    assert!(rows[..MAX_FIX_LINES].iter().all(|row| row.1 == '~'));
}

#[test]
fn line_ending_edit_marks_only_the_changed_line() {
    let text = "a\nb\nc\n";
    let rows = changed_lines(
        text,
        &LineIndex::new(text),
        &[(0, text.len(), "a\nb\r\nc\n".into())],
    );
    assert_eq!(
        rows,
        vec![
            ("1".into(), ' ', "a".into()),
            ("2".into(), '~', "b".into()),
            ("3".into(), ' ', "c".into())
        ]
    );
}

#[test]
fn fix_boundary_edits_measure_complete_logical_lines() {
    let cases = [
        (
            "a;\nb;",
            (2, 3, String::new()),
            vec![("1", '~', "a;b;"), ("2", '-', "b;")],
        ),
        (
            "a\rb",
            (2, 2, "\n".into()),
            vec![("1", '~', "a"), ("2", ' ', "b")],
        ),
        (
            "a;\r\nb;",
            (2, 4, String::new()),
            vec![("1", '~', "a;b;"), ("2", '-', "b;")],
        ),
    ];
    for (text, edit, expected) in cases {
        assert_eq!(
            changed_lines(text, &LineIndex::new(text), &[edit]),
            expected
                .into_iter()
                .map(|(n, mark, text)| (n.into(), mark, text.into()))
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn broad_edit_elides_unchanged_interior_gaps() {
    let text = "before\nold1\nkeep1\nkeep2\nkeep3\nold2\nafter";
    let fixed = "before\nnew1\nkeep1\nkeep2\nkeep3\nnew2\nafter";
    assert_eq!(
        changed_lines(
            text,
            &LineIndex::new(text),
            &[(0, text.len(), fixed.into())]
        ),
        vec![
            ("1".into(), ' ', "before".into()),
            ("2".into(), '~', "new1".into()),
            ("3".into(), ' ', "keep1".into()),
            ("...".into(), ' ', String::new()),
            ("5".into(), ' ', "keep3".into()),
            ("6".into(), '~', "new2".into()),
            ("7".into(), ' ', "after".into()),
        ]
    );
}

#[test]
fn mixed_content_and_line_ending_edits_preserve_each_change() {
    let text = "a\nb\nc\n";
    assert_eq!(
        changed_lines(
            text,
            &LineIndex::new(text),
            &[(1, 2, "\r\n".into()), (4, 5, "C".into())]
        ),
        vec![
            ("1".into(), '~', "a".into()),
            ("2".into(), ' ', "b".into()),
            ("3".into(), '~', "C".into()),
        ]
    );
}

#[test]
fn cancelling_edits_do_not_invent_a_context_only_preview() {
    let text = "a;";
    assert!(
        changed_lines(
            text,
            &LineIndex::new(text),
            &[(0, 1, String::new()), (1, 1, "a".into())]
        )
        .is_empty()
    );
}
