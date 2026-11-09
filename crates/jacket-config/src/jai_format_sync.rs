//! The Jai formatter reads the same `[format]` keys when it runs without `jacket` (standalone
//! `jaifmt`, the browser playground). Its keys, ranges, defaults and messages must be the ones in
//! `schema::FORMAT_SETTINGS`; these tests read its source and compare.
use crate::schema::{FORMAT_SETTINGS, Fallback, Kind};

const MODULE: &str = include_str!("../../../stdlib/Extensions/Jai_Format/module.jai");
const CONFIG: &str = include_str!("../../../stdlib/Extensions/Jai_Format/config.jai");

/// Each field of `Format_Config`: its name, its declaration (type or default, without the
/// trailing comment) and the text of its `@toml(...)` note.
fn fields() -> Vec<(String, String, String)> {
    let body = MODULE
        .split("Format_Config :: struct")
        .nth(1)
        .and_then(|rest| rest.split_once('{'))
        .and_then(|(_, rest)| rest.split("\n}").next())
        .expect("Format_Config struct");
    body.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with("//"))
        .map(|line| {
            let (declaration, rest) = line.split_once(';').expect("field ends with `;`");
            let note = rest
                .trim()
                .strip_prefix("@toml(")
                .and_then(|n| n.rsplit_once(')'))
                .map_or("", |(n, _)| n);
            let at = declaration.find([':', ' ']).unwrap();
            (
                declaration[..at].to_string(),
                declaration[at..]
                    .trim_start_matches([':', ' ', '='])
                    .to_string(),
                note.to_string(),
            )
        })
        .collect()
}

#[test]
fn struct_fields_are_the_schema_keys() {
    let fields = fields();
    let names: Vec<&str> = fields.iter().map(|(n, ..)| n.as_str()).collect();
    let mut want: Vec<&str> = FORMAT_SETTINGS.iter().map(|s| s.name).collect();
    want.extend(["unknown_keys", "doc_format", "root"]);
    assert_eq!(names, want);
}

#[test]
fn defaults_are_the_schema_defaults() {
    let fields = fields();
    for setting in FORMAT_SETTINGS {
        let (_, declared, _) = fields.iter().find(|(n, ..)| n == setting.name).unwrap();
        let want = match setting.default {
            Fallback::Int(v) => v.to_string(),
            Fallback::IndentWidth => "-1".into(),
            Fallback::Flag(v) => v.to_string(),
            Fallback::Choice(v) => format!("Brace_Style.{}", v.to_uppercase()),
            Fallback::Empty => "[..] string".into(),
        };
        assert_eq!(*declared, want, "default of {}", setting.name);
    }
}

#[test]
fn notes_are_the_schema_types() {
    let names: Vec<&str> = FORMAT_SETTINGS.iter().map(|s| s.name).collect();
    let keys = CONFIG
        .lines()
        .find(|l| l.starts_with("FORMAT_KEYS"))
        .expect("FORMAT_KEYS");
    let keys: Vec<&str> = keys.split('"').skip(1).step_by(2).collect();
    assert_eq!(keys, names, "FORMAT_KEYS");

    let fields = fields();
    for (name, _, note) in &fields {
        if !names.contains(&name.as_str()) {
            assert_eq!(
                note, "skip",
                "{name} is not a setting, so the decoder skips it"
            );
        }
    }
    for setting in FORMAT_SETTINGS {
        let (_, _, note) = fields.iter().find(|(n, ..)| n == setting.name).unwrap();
        match setting.kind {
            Kind::Int {
                min,
                max,
            } => {
                assert!(
                    note.contains(&format!("range={min}..{max}")),
                    "{} range: {note}",
                    setting.name
                );
                let text = format!("{} must be an integer from {min} to {max}", setting.name);
                assert!(note.contains(&text), "{} message", setting.name);
            }
            Kind::Flag => {
                let text = format!("{} must be true or false", setting.name);
                assert!(note.contains(&text), "{} message", setting.name);
            }
            Kind::Choice(choices) => {
                for choice in choices {
                    assert!(
                        note.contains(&format!("\\\"{choice}\\\"")),
                        "{choice}: {note}"
                    );
                }
            }
            Kind::Globs => assert!(note.contains("ignore must be an array"), "ignore"),
        }
    }
}

#[test]
fn validate_words_errors_like_the_jai_parser() {
    let config = crate::parse("[format]\nindent_width = 17\n").unwrap();
    assert_eq!(
        config.format.validate().unwrap_err(),
        "indent_width must be an integer from 1 to 16"
    );
    let config = crate::parse("[format]\nbrace_style = \"x\"\n").unwrap();
    assert_eq!(
        config.format.validate().unwrap_err(),
        "brace_style must be \"same_line\" or \"preserve\""
    );
    assert!(
        crate::parse("[format]\n")
            .unwrap()
            .format
            .validate()
            .is_ok()
    );
}

#[test]
fn resolved_fills_in_defaults() {
    let config = crate::parse("[format]\nindent_width = 2\nmax_width = 80\n").unwrap();
    let got = config.format.resolved();
    assert!(got.contains(&("case_indent", "2".to_string())));
    assert!(got.contains(&("max_width", "80".to_string())));
    assert!(got.contains(&("brace_style", "\"same_line\"".to_string())));
    assert!(got.contains(&("blank_lines_between_items", "true".to_string())));
    assert_eq!(got.len(), FORMAT_SETTINGS.len() - 1);
}
