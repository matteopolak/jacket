//! The Jai formatter reads the same `[format]` keys when it runs without `jacket` (standalone
//! `jaifmt`, the browser playground). Its keys, ranges, defaults and messages must be the ones in
//! `schema::FORMAT_SETTINGS`; these tests read its source and compare.
use crate::schema::{FORMAT_SETTINGS, Fallback, Kind};

const MODULE: &str = include_str!("../../../stdlib/Extensions/Jai_Format/module.jai");
const CONFIG: &str = include_str!("../../../stdlib/Extensions/Jai_Format/config.jai");

/// The name and declaration (without its comment) of each field of `Format_Config`.
fn fields() -> Vec<(String, String)> {
    let body = MODULE
        .split("Format_Config :: struct {")
        .nth(1)
        .and_then(|rest| rest.split("\n}").next())
        .expect("Format_Config struct");
    body.lines()
        .map(|line| {
            line.split("//")
                .next()
                .unwrap()
                .trim()
                .trim_end_matches(';')
        })
        .filter(|line| !line.is_empty())
        .map(|line| {
            let at = line.find([':', ' ']).unwrap();
            (
                line[..at].to_string(),
                line[at..].trim_start_matches([':', ' ', '=']).to_string(),
            )
        })
        .collect()
}

/// The text of `apply_key`'s `case "name";` arm.
fn arm(name: &str) -> &'static str {
    let from = CONFIG
        .find(&format!("case \"{name}\";"))
        .unwrap_or_else(|| panic!("no arm for {name}"));
    let rest = &CONFIG[from + 1..];
    let end = rest.find("\n        case").unwrap_or(rest.len());
    &rest[..end]
}

#[test]
fn struct_fields_are_the_schema_keys() {
    let fields = fields();
    let names: Vec<&str> = fields.iter().map(|(n, _)| n.as_str()).collect();
    let mut want: Vec<&str> = FORMAT_SETTINGS.iter().map(|s| s.name).collect();
    want.extend(["unknown_keys", "root"]);
    assert_eq!(names, want);
}

#[test]
fn defaults_are_the_schema_defaults() {
    let fields = fields();
    for setting in FORMAT_SETTINGS {
        let (_, declared) = fields.iter().find(|(n, _)| n == setting.name).unwrap();
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
fn parser_arms_are_the_schema_types() {
    let names: Vec<&str> = FORMAT_SETTINGS.iter().map(|s| s.name).collect();
    let arms: Vec<&str> = CONFIG
        .lines()
        .filter_map(|l| l.trim().strip_prefix("case \""))
        .filter_map(|l| l.split('"').next())
        .collect();
    assert_eq!(arms, names, "apply_key arms");
    let keys = CONFIG
        .lines()
        .find(|l| l.starts_with("FORMAT_KEYS"))
        .expect("FORMAT_KEYS");
    let keys: Vec<&str> = keys.split('"').skip(1).step_by(2).collect();
    assert_eq!(keys, names, "FORMAT_KEYS");

    for setting in FORMAT_SETTINGS {
        let arm = arm(setting.name);
        match setting.kind {
            Kind::Int {
                min,
                max,
            } => {
                let call = format!("*config.{}, {min}, {max})", setting.name);
                assert!(arm.contains(&call), "{} range: {arm}", setting.name);
                let text = format!("{} must be an integer from {min} to {max}", setting.name);
                assert!(arm.contains(&text), "{} message", setting.name);
            }
            Kind::Flag => {
                let text = format!("{} must be true or false", setting.name);
                assert!(arm.contains(&text), "{} message", setting.name);
            }
            Kind::Choice(choices) => {
                for choice in choices {
                    assert!(arm.contains(&format!("style == \"{choice}\"")), "{choice}");
                }
            }
            Kind::Globs => assert!(arm.contains("ignore must be an array"), "ignore"),
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
