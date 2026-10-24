//! The `jacket.toml` schema. Every section is optional and every key in it too. Keys the schema
//! does not know are dropped while parsing (see [`crate::parse_noting`]), never kept.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct Config {
    #[serde(default, skip_serializing_if = "Project::is_empty")]
    pub project: Project,
    #[serde(default, skip_serializing_if = "Toolchain::is_empty")]
    pub toolchain: Toolchain,
    #[serde(default, skip_serializing_if = "Lint::is_empty")]
    pub lint: Lint,
    #[serde(default, skip_serializing_if = "Format::is_empty")]
    pub format: Format,
    #[serde(default, skip_serializing_if = "Experimental::is_empty")]
    pub experimental: Experimental,
}

/// A string or a list of strings (`entry = "main.jai"` is a list of one).
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Strings(pub Vec<String>);

impl<'de> Deserialize<'de> for Strings {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged, expecting = "a string or an array of strings")]
        enum One {
            One(String),
            Many(Vec<String>),
        }
        Ok(Strings(match One::deserialize(d)? {
            One::One(one) => vec![one],
            One::Many(many) => many,
        }))
    }
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct Project {
    /// The files given to `jaic` (a metaprogram, or the program), relative to the file.
    #[serde(default, skip_serializing_if = "is_empty_strings")]
    pub entry: Strings,
    /// More module folders, as `-import_dir` adds.
    #[serde(default, skip_serializing_if = "is_empty_strings")]
    pub import_path: Strings,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct Toolchain {
    /// The `x.y.z` release the project is written for.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct Lint {
    /// Glob patterns of paths not to lint, relative to the file.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exclude: Vec<String>,
    /// How many blocks deep code may nest before `excessive_nesting` fires.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_nesting: Option<usize>,
    /// The score above which `cognitive_complexity` reports a procedure.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_complexity: Option<usize>,
    /// How many parameters a procedure may take before `too_many_arguments` reports it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_arguments: Option<usize>,
    /// How many `bool` parameters a procedure may take before `excessive_bools` reports it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_bools: Option<usize>,
    /// How many bytes a local fixed array may take before `large_stack_array` reports it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_stack_array_bytes: Option<usize>,
    /// Numbers `magic_number` never reports, besides its built-in ones.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub magic_number_allow: Vec<f64>,
    /// Whole numbers smaller than this in magnitude are not magic (`magic_number`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub magic_number_min: Option<f64>,
    /// `magic_number` also checks files in `tests` and `test` folders.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub magic_number_in_tests: Option<bool>,
    /// Rule name and level text (`allow`, `warn`, `deny`).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub rules: BTreeMap<String, String>,
    /// Rule levels for some paths (`[[lint.overrides]]`); a later entry wins over an earlier one.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub overrides: Vec<LintOverride>,
    /// Which code must build on every operating system (`platform_leak`).
    #[serde(default, skip_serializing_if = "Portability::is_empty")]
    pub portability: Portability,
}

/// The code that is meant to build on every operating system, for `platform_leak`.
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct Portability {
    /// The operating systems portable code is built for: `linux`, `macos`, `windows`. Left out,
    /// all three.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub targets: Vec<String>,
    /// Globs (relative to the file) of the code that must build on every target.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub portable: Vec<String>,
    /// Globs of files that build only on the operating systems listed (none: on no target). When
    /// several match a file, the most specific one (the most characters besides `*` and `?`)
    /// decides.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub platforms: BTreeMap<String, Vec<String>>,
}

/// Rule levels for the files under some paths.
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct LintOverride {
    /// Globs of the files these levels apply to, relative to the file.
    #[serde(default)]
    pub paths: Vec<String>,
    /// Rule or category name and level text, as in `[lint.rules]`.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub rules: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct Format {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub indent_width: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub case_indent: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub case_body_indent: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_blank_lines: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_width: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub brace_style: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blank_lines_between_items: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blank_line_after_declarations: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blank_lines_around_blocks: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blank_line_between_cases: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blank_line_before_comments: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blank_line_after_guards: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blank_line_before_return: Option<bool>,
    /// Glob patterns of paths not to format, relative to the file.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ignore: Vec<String>,
}

/// Performance and debugging knobs; none changes what a program means.
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct Experimental {
    /// Threads that parse files ahead of the checker (`-parse_threads`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parse_threads: Option<usize>,
    /// Parse on the checking thread (`-serial_parse`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub serial_parse: Option<bool>,
    /// Object files a native build is split into (`-codegen_units`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub codegen_units: Option<usize>,
}

fn is_empty_strings(s: &Strings) -> bool {
    s.0.is_empty()
}

macro_rules! is_empty {
    ($($t:ty),*) => {$(impl $t { pub fn is_empty(&self) -> bool { *self == Self::default() } })*};
}

is_empty!(Project, Toolchain, Lint, Format, Experimental, Portability);

/// The keys each table accepts, to suggest the near one for a misspelling. A test keeps these in
/// step with the structs.
pub(crate) fn known_keys(table: &str) -> &'static [&'static str] {
    match table {
        "" => &["project", "toolchain", "lint", "format", "experimental"],
        "project" => &["entry", "import_path"],
        "toolchain" => &["version"],
        "lint" => &[
            "exclude",
            "max_nesting",
            "max_complexity",
            "max_arguments",
            "max_bools",
            "max_stack_array_bytes",
            "magic_number_allow",
            "magic_number_min",
            "magic_number_in_tests",
            "rules",
            "overrides",
            "portability",
        ],
        "lint.portability" => &["targets", "portable", "platforms"],
        "format" => &[
            "indent_width",
            "case_indent",
            "case_body_indent",
            "max_blank_lines",
            "max_width",
            "brace_style",
            "blank_lines_between_items",
            "blank_line_after_declarations",
            "blank_lines_around_blocks",
            "blank_line_between_cases",
            "blank_line_before_comments",
            "blank_line_after_guards",
            "blank_line_before_return",
            "ignore",
        ],
        "experimental" => &["parse_threads", "serial_parse", "codegen_units"],
        _ => &[],
    }
}

/// Likely spellings of a section's key, written at the top level of the file: the spelling, the
/// section and the key it means.
pub(crate) const ALIASES: &[(&str, &str, &str)] = &[
    ("build_files", "project", "entry"),
    ("build_file", "project", "entry"),
    ("entries", "project", "entry"),
    ("entry_files", "project", "entry"),
    ("import_dir", "project", "import_path"),
    ("import_dirs", "project", "import_path"),
    ("import_paths", "project", "import_path"),
];

/// Where a top-level key was meant: the section and key, found by exact name or by alias.
pub(crate) fn misplaced_home(key: &str) -> Option<(&'static str, &'static str)> {
    let section = ["project", "toolchain", "lint", "format", "experimental"]
        .into_iter()
        .find_map(|table| {
            known_keys(table)
                .iter()
                .find(|k| **k == key)
                .map(|k| (table, *k))
        });
    section.or_else(|| {
        ALIASES
            .iter()
            .find(|(a, ..)| *a == key)
            .map(|(_, table, key)| (*table, *key))
    })
}

/// The known name nearest to a misspelled `wanted`: a case-only difference first, then the
/// fewest edits (a swap of neighbours counts one), within a third of the name's length.
pub(crate) fn closest<'a>(wanted: &str, known: &[&'a str]) -> Option<&'a str> {
    let limit = (wanted.chars().count() / 3).max(1);
    known
        .iter()
        .copied()
        .filter(|name| *name != wanted)
        .filter_map(|name| {
            let d = if name.eq_ignore_ascii_case(wanted) {
                0
            } else {
                edits(wanted, name)
            };
            (d <= limit).then_some((d, name))
        })
        .min()
        .map(|(_, name)| name)
}

/// Optimal-string-alignment distance.
fn edits(a: &str, b: &str) -> usize {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    let mut d = vec![vec![0usize; b.len() + 1]; a.len() + 1];
    for (i, row) in d.iter_mut().enumerate() {
        row[0] = i;
    }
    for (j, cell) in d[0].iter_mut().enumerate() {
        *cell = j;
    }
    for i in 1..=a.len() {
        for j in 1..=b.len() {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            d[i][j] = (d[i - 1][j] + 1)
                .min(d[i][j - 1] + 1)
                .min(d[i - 1][j - 1] + cost);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                d[i][j] = d[i][j].min(d[i - 2][j - 2] + 1);
            }
        }
    }
    d[a.len()][b.len()]
}
