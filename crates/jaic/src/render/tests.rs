use super::*;

const SOURCE: &str = "main :: () {\n    counter := 1;\n    print(\"%\\n\", countr);\n}\n";

fn label(text: &str, needle: &str, message: Option<&str>) -> Label<'static> {
    let text: &'static str = Box::leak(text.to_string().into_boxed_str());
    let start = text.find(needle).unwrap();
    Label {
        path: "u.jai",
        text,
        start,
        end: start + needle.len(),
        message: message.map(str::to_string),
    }
}

fn unknown_name() -> Report<'static> {
    let primary = label(SOURCE, "countr)", Some("not found in this scope"));
    let at = primary.start;
    let mut report = Report::new(Severity::Error, "unknown identifier `countr`");
    report.primary = Some(Label {
        end: at + 6,
        ..primary
    });
    report.help.push(Help {
        message: "a similar name exists: `counter`".into(),
        fix: Some(Fix {
            text: SOURCE,
            edits: vec![(at, at + 6, "counter".into())],
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

#[test]
fn plain_compact_is_the_long_standing_layout() {
    let text = unknown_name().render_with(Style::PLAIN);
    assert_eq!(
        text,
        "u.jai:3:18: error: unknown identifier `countr`\n\
         \x20       print(\"%\\n\", countr);\n\
         \x20                    ^^^^^^\n\
         note: names are case-sensitive\n\
         help: a similar name exists: `counter`\n"
    );
}

#[test]
fn ascii_block_has_line_numbers_context_labels_and_the_fix() {
    let text = unknown_name().render_with(ASCII_STYLE);
    assert_eq!(
        text,
        "error: unknown identifier `countr`\n\
         \x20 --> u.jai:3:18\n\
         \x20  |\n\
         \x202 |     counter := 1;\n\
         \x203 |     print(\"%\\n\", countr);\n\
         \x20  |                  ^^^^^^ not found in this scope\n\
         \x204 | }\n\
         \x20  |\n\
         help: a similar name exists: `counter`\n\
         \x20  |\n\
         \x203 ~     print(\"%\\n\", counter);\n\
         \x20  |\n\
         \x20  = note: names are case-sensitive\n"
    );
}

#[test]
fn unicode_block_draws_boxes() {
    let text = unknown_name().render_with(UNICODE_STYLE);
    assert_eq!(
        text,
        "error: unknown identifier `countr`\n\
         \x20  ╭─[u.jai:3:18]\n\
         \x202 │     counter := 1;\n\
         \x203 │     print(\"%\\n\", countr);\n\
         \x20  ·                  ━━━━━━ not found in this scope\n\
         \x204 │ }\n\
         \x20  ╰─\n\
         help: a similar name exists: `counter`\n\
         \x20  │\n\
         \x203 ~     print(\"%\\n\", counter);\n\
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
        text.starts_with("\x1b[1;31merror\x1b[0m\x1b[1m: unknown identifier"),
        "{text}"
    );
    assert!(text.contains("\x1b[34m│\x1b[0m"), "{text}");
    assert!(text.contains("\x1b[1;31m━━━━━━\x1b[0m"), "{text}");
    assert!(!Style::PLAIN.color);
    assert!(!unknown_name().render_with(UNICODE_STYLE).contains('\x1b'));
}

#[test]
fn secondary_labels_and_other_files() {
    let source = "x: int = 1;\nmain :: () {\n    x = \"text\";\n}\n";
    let mut report = Report::new(
        Severity::Error,
        "type mismatch: expected `int`, found `string`",
    );
    report.primary = Some(label(source, "\"text\"", Some("this is a `string`")));
    report
        .secondary
        .push(label(source, "int", Some("`x` is declared as `int` here")));
    let mut other = label("helper :: () {}\n", "helper", Some("also here"));
    other.path = "other.jai";
    report.secondary.push(other);
    let text = report.render_with(ASCII_STYLE);
    assert_eq!(
        text,
        "error: type mismatch: expected `int`, found `string`\n\
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
    let mut report = Report::new(Severity::Warning, "`f` is never used");
    let start = 0;
    let end = source.find("}\n").unwrap() + 1;
    report.primary = Some(Label {
        path: "m.jai",
        text: source,
        start,
        end,
        message: Some("this procedure".into()),
    });
    let text = report.render_with(UNICODE_STYLE);
    assert_eq!(
        text,
        "warning: `f` is never used\n\
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
    let mut report = Report::new(Severity::Error, "long");
    report.primary = Some(Label {
        path: "l.jai",
        text: Box::leak(source.clone().into_boxed_str()),
        start: 0,
        end: source.len() - 1,
        message: None,
    });
    let text = report.render_with(ASCII_STYLE);
    assert!(text.contains("\n   ...\n"), "{text}");
    assert!(!text.contains("x10"), "{text}");
    assert!(text.contains("x19"), "{text}");

    let long = format!("x := {}needle{};\n", "a".repeat(200), "b".repeat(200));
    let mut report = Report::new(Severity::Error, "cut");
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
    let mut report = Report::new(Severity::Error, "unknown identifier `y`");
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
    let report = Report::new(Severity::Error, "file `nope.jai` does not exist")
        .help("did you mean `nop.jai`?");
    for style in [Style::PLAIN, ASCII_STYLE, UNICODE_STYLE] {
        assert_eq!(
            report.render_with(style),
            "error: file `nope.jai` does not exist\nhelp: did you mean `nop.jai`?\n"
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
    assert_eq!(detect_with(auto, false, &utf8), Style::PLAIN);
    // A UTF-8 terminal.
    assert_eq!(
        detect_with(auto, true, &utf8),
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
    assert!(!detect_with(ColorChoice::Never, true, &utf8).color);
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
