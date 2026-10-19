//! `jacket.toml` loading and the migration of the legacy files (`jai.toml`, `jailint.toml`,
//! `jaifmt.toml`), on arbitrary text.
//!
//! The input is up to four files separated by NUL bytes, in the order `jai.toml`, `jailint.toml`,
//! `jaifmt.toml`, `jacket.toml`; an empty one is absent. Every input must load without panicking
//! (a bad file is an error), and for the legacy files:
//!
//! - the migrated text is valid TOML whenever the plan is `Ok`, and each section holds exactly
//!   the legacy keys (renamed, empty tables ignored) with the same values;
//! - reading the legacy files and reading the migrated `jacket.toml` give the same settings, or
//!   the same refusal;
//! - settings serialize and read back to the same settings;
//! - with a `jacket.toml` present the legacy files are ignored.
use jacket_config::migrate;
use jacket_config::{Config, Files, load, parse};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const NAMES: [&str; 4] = ["jai.toml", "jailint.toml", "jaifmt.toml", "jacket.toml"];

/// A legacy file, the section it becomes and its renamed keys, as `migrate` maps them.
type Mapping = (
    &'static str,
    &'static str,
    &'static [(&'static str, &'static str)],
);

const MAPPING: [Mapping; 3] = [
    ("jai.toml", "project", &[("build_files", "entry")]),
    ("jailint.toml", "lint", &[]),
    ("jaifmt.toml", "format", &[]),
];

struct Mem(BTreeMap<PathBuf, String>);

impl Files for Mem {
    fn is_file(&self, path: &Path) -> bool {
        self.0.contains_key(path)
    }

    fn read(&self, path: &Path) -> Option<String> {
        self.0.get(path).cloned()
    }
}

fn files(parts: &[&str], names: &[&str]) -> Mem {
    Mem(parts
        .iter()
        .zip(names)
        .filter(|(text, _)| !text.is_empty())
        .map(|(text, name)| (Path::new("/p").join(name), text.to_string()))
        .collect())
}

pub fn jacket_config(data: &[u8]) {
    let text = String::from_utf8_lossy(data);
    let mut parts: Vec<&str> = text.splitn(4, '\0').collect();
    parts.resize(4, "");
    let dir = Path::new("/p");

    // Everything together: the `jacket.toml` wins, or the legacy files are read.
    let all = files(&parts, &NAMES);
    let loaded = load(&all, dir);
    if !parts[3].is_empty() {
        match (parse(parts[3]), loaded) {
            (Ok(config), Ok(Some(loaded))) => assert_eq!(config, loaded.config),
            (Err(_), Err(_)) => {}
            (parsed, loaded) => panic!("jacket.toml read two ways: {parsed:?} / {loaded:?}"),
        }
    }

    let legacy = files(&parts[..3], &NAMES[..3]);
    let Ok(plan) = migrate::plan(&legacy, dir) else {
        // A legacy file that is not TOML: loading refuses it too.
        if !legacy.0.is_empty() {
            assert!(load(&legacy, dir).is_err(), "plan failed but load did not");
        }
        return;
    };
    if legacy.0.is_empty() {
        return;
    }
    let migrated: toml::Table = toml::from_str(&plan.text)
        .unwrap_or_else(|e| panic!("the migrated text is not TOML: {e}\n{}", plan.text));
    check_sections(&parts, &migrated, &plan.text);

    // Same settings from the legacy files and from the file migration wrote.
    let from_legacy = load(&legacy, dir).map(|l| l.map(|l| l.config));
    let from_jacket =
        load(&files(&[plan.text.as_str()], &["jacket.toml"]), dir).map(|l| l.map(|l| l.config));
    match (&from_legacy, &from_jacket) {
        (Ok(a), Ok(b)) => assert_eq!(a, b, "legacy and migrated settings differ:\n{}", plan.text),
        (Err(_), Err(_)) => {}
        _ => panic!(
            "legacy {from_legacy:?} but migrated {from_jacket:?}:\n{}",
            plan.text
        ),
    }
    if let Ok(Some(config)) = from_jacket {
        round_trip(&config);
    }
}

/// Tables without keys: migration keeps one when a comment sits above it, so neither side counts
/// them.
fn without_empty_tables(mut table: toml::Table) -> toml::Table {
    table.retain(|_, v| !matches!(v, toml::Value::Table(t) if t.is_empty()));
    table
}

/// Each migrated section equals its legacy file's table, keys renamed.
fn check_sections(parts: &[&str], migrated: &toml::Table, text: &str) {
    for (i, (_, section, renames)) in MAPPING.iter().enumerate() {
        if parts[i].is_empty() {
            continue;
        }
        let Ok(legacy) = toml::from_str::<toml::Table>(parts[i]) else {
            continue;
        };
        let mut expected = toml::Table::new();
        let mut clash = false;
        for (key, value) in legacy {
            let name = renames
                .iter()
                .find(|(from, _)| *from == key)
                .map_or(key, |(_, to)| (*to).to_string());
            clash |= expected.insert(name, value).is_some();
        }
        if clash {
            continue;
        }
        let got = match migrated.get(*section) {
            Some(toml::Value::Table(t)) => t.clone(),
            None => toml::Table::new(),
            Some(other) => panic!("section {section} is {other:?}:\n{text}"),
        };
        assert_eq!(
            without_empty_tables(expected).to_string(),
            without_empty_tables(got).to_string(),
            "section [{section}] differs:\n{text}"
        );
    }
}

/// Written out and read back, the settings are the same.
fn round_trip(config: &Config) {
    let written = toml::to_string(config)
        .unwrap_or_else(|e| panic!("settings do not serialize: {e}\n{config:?}"));
    let again =
        parse(&written).unwrap_or_else(|e| panic!("settings do not read back: {e}\n{written}"));
    let twice = toml::to_string(&again).expect("serializes again");
    assert_eq!(written, twice, "serialization is not stable");
}
