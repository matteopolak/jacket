//! The `jacket.toml` schema. Every section is optional and every key in it too; keys the schema
//! does not know are kept in `extra` so the tools can warn about them instead of failing.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub type Extra = BTreeMap<String, toml::Value>;

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
    /// Tables the schema does not know.
    #[serde(flatten)]
    pub extra: Extra,
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
    #[serde(flatten)]
    pub extra: Extra,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct Toolchain {
    /// The `x.y.z` release the project is written for.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(flatten)]
    pub extra: Extra,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct Lint {
    /// Glob patterns of paths not to lint, relative to the file.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exclude: Vec<String>,
    /// How many blocks deep code may nest before `excessive_nesting` fires.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_nesting: Option<usize>,
    /// Rule name and level text (`allow`, `warn`, `deny`).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub rules: BTreeMap<String, String>,
    #[serde(flatten)]
    pub extra: Extra,
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
    #[serde(flatten)]
    pub extra: Extra,
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
    #[serde(flatten)]
    pub extra: Extra,
}

fn is_empty_strings(s: &Strings) -> bool {
    s.0.is_empty()
}

macro_rules! is_empty {
    ($($t:ty),*) => {$(impl $t { pub fn is_empty(&self) -> bool { *self == Self::default() } })*};
}
is_empty!(Project, Toolchain, Lint, Format, Experimental);

impl Config {
    /// The dotted names of every key and table the schema does not know.
    pub fn unknown_keys(&self) -> Vec<String> {
        let mut out: Vec<String> = self.extra.keys().cloned().collect();
        let sections: [(&str, &Extra); 5] = [
            ("project", &self.project.extra),
            ("toolchain", &self.toolchain.extra),
            ("lint", &self.lint.extra),
            ("format", &self.format.extra),
            ("experimental", &self.experimental.extra),
        ];
        for (name, extra) in sections {
            out.extend(extra.keys().map(|k| format!("{name}.{k}")));
        }
        out
    }
}
