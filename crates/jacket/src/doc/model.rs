//! What the site documents: modules, their items and overloads, read from the JSON
//! `jaic -doc_model` prints for each library of the standard library.
use serde_json::Value;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Default)]
pub struct Param {
    pub name: String,
    pub ty: String,
    pub default: String,
    pub doc: String,
}

#[derive(Clone, Debug, Default)]
pub struct Overload {
    /// Below the standard library folder: `internal/string/bytes.jai`.
    pub file: String,
    pub line: usize,
    pub params: Vec<Param>,
    pub returns: Vec<Param>,
    pub docs: String,
    pub flags: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct Member {
    pub name: String,
    pub ty: String,
    pub doc: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Procedure,
    Struct,
    Union,
    Enum,
    EnumFlags,
    Type,
    Constant,
}

impl Kind {
    fn parse(text: &str) -> Kind {
        match text {
            "procedure" => Kind::Procedure,
            "struct" => Kind::Struct,
            "union" => Kind::Union,
            "enum" => Kind::Enum,
            "enum_flags" => Kind::EnumFlags,
            "type" => Kind::Type,
            _ => Kind::Constant,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Kind::Procedure => "procedure",
            Kind::Struct => "struct",
            Kind::Union => "union",
            Kind::Enum => "enum",
            Kind::EnumFlags => "enum flags",
            Kind::Type => "type",
            Kind::Constant => "constant",
        }
    }

    /// Whether other signatures can link to it as a type.
    pub fn is_type(self) -> bool {
        matches!(
            self,
            Kind::Struct | Kind::Union | Kind::Enum | Kind::EnumFlags | Kind::Type
        )
    }
}

#[derive(Clone, Debug)]
pub struct Item {
    pub name: String,
    pub kind: Kind,
    pub file: String,
    pub line: usize,
    pub docs: String,
    pub value: String,
    pub overloads: Vec<Overload>,
    pub members: Vec<Member>,
    /// The heading it is listed under on the module page; empty for none.
    pub group: String,
}

#[derive(Clone, Debug)]
pub struct Module {
    /// `String`, `Extensions/Http`: what `#import` takes.
    pub id: String,
    pub file: String,
    pub docs: String,
    pub platforms: Vec<String>,
    pub items: Vec<Item>,
}

impl Module {
    /// `Http` for `Extensions/Http`.
    pub fn title(&self) -> &str {
        self.id.rsplit('/').next().unwrap_or(&self.id)
    }

    pub fn category(&self) -> &'static str {
        if self.id.starts_with("Extensions/") {
            "Extensions"
        } else {
            "Standard library"
        }
    }

    pub fn item(&self, name: &str) -> Option<&Item> {
        self.items.iter().find(|i| i.name == name)
    }
}

fn text(v: &Value, key: &str) -> String {
    v.get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned()
}

fn number(v: &Value, key: &str) -> usize {
    v.get(key).and_then(Value::as_u64).unwrap_or(0) as usize
}

fn list<'a>(v: &'a Value, key: &str) -> &'a [Value] {
    v.get(key)
        .and_then(Value::as_array)
        .map_or(&[], Vec::as_slice)
}

fn params(v: &Value, key: &str) -> Vec<Param> {
    list(v, key)
        .iter()
        .map(|p| Param {
            name: text(p, "name"),
            ty: text(p, "type"),
            default: text(p, "default"),
            doc: text(p, "doc"),
        })
        .collect()
}

/// `file` below `stdlib`, `/`-separated; the path itself when it is elsewhere.
fn below(stdlib: &Path, file: &str) -> String {
    let path = PathBuf::from(file);
    path.strip_prefix(stdlib)
        .map_or_else(|_| path.clone(), Path::to_path_buf)
        .to_string_lossy()
        .replace('\\', "/")
}

/// The module `id` from what `jaic -doc_model` printed.
pub fn parse(id: &str, json: &str, stdlib: &Path) -> Result<Module, String> {
    let v: Value = serde_json::from_str(json).map_err(|e| format!("{id}: {e}"))?;
    let items = list(&v, "items")
        .iter()
        .map(|i| {
            let overloads: Vec<Overload> = list(i, "overloads")
                .iter()
                .map(|o| Overload {
                    file: below(stdlib, &text(o, "file")),
                    line: number(o, "line"),
                    params: params(o, "params"),
                    returns: params(o, "returns"),
                    docs: text(o, "docs"),
                    flags: list(o, "flags")
                        .iter()
                        .filter_map(|f| f.as_str().map(str::to_owned))
                        .collect(),
                })
                .collect();
            Item {
                name: text(i, "name"),
                kind: Kind::parse(&text(i, "kind")),
                file: below(stdlib, &text(i, "file")),
                line: number(i, "line"),
                docs: text(i, "docs"),
                value: text(i, "value"),
                overloads,
                members: list(i, "members")
                    .iter()
                    .map(|m| Member {
                        name: text(m, "name"),
                        ty: text(m, "type"),
                        doc: text(m, "doc"),
                    })
                    .collect(),
                group: String::new(),
            }
        })
        .collect();
    let mut module = Module {
        id: id.to_owned(),
        file: below(stdlib, &text(&v, "file")),
        docs: text(&v, "docs"),
        platforms: list(&v, "platforms")
            .iter()
            .filter_map(|p| p.as_str().map(str::to_owned))
            .collect(),
        items,
    };
    assign_groups(&mut module);
    Ok(module)
}

/// `bytes.jai` is "Bytes", `self-test.jai` is "Self test".
fn title_case(stem: &str) -> String {
    let words = stem.replace(['-', '_'], " ");
    let mut chars = words.chars();
    chars
        .next()
        .map(|c| c.to_uppercase().collect::<String>() + chars.as_str())
        .unwrap_or_default()
}

/// Procedures are grouped by the file that declares them when that tells more than one heading
/// would (two or more files); other items by what they are.
fn assign_groups(module: &mut Module) {
    let stem = |file: &str| {
        let name = file.rsplit('/').next().unwrap_or(file);
        let stem = name.strip_suffix(".jai").unwrap_or(name);
        if stem == "module" {
            "General".to_owned()
        } else {
            title_case(stem)
        }
    };
    let mut stems: Vec<String> = Vec::new();
    for item in module.items.iter().filter(|i| i.kind == Kind::Procedure) {
        let s = stem(&item.file);
        if !stems.contains(&s) {
            stems.push(s);
        }
    }
    let split = stems.len() >= 2;
    for item in &mut module.items {
        item.group = if item.kind == Kind::Procedure && split {
            stem(&item.file)
        } else {
            String::new()
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn items_come_out_with_groups_and_relative_files() {
        let json = r#"{"file":"/s/String/module.jai","docs":"A module.","docs_line":1,"platforms":["MACOS"],"items":[
{"name":"trim","kind":"procedure","file":"/s/internal/string/bytes.jai","line":3,"docs":"","value":"","overloads":[{"file":"/s/internal/string/bytes.jai","line":3,"signature":"","params":[{"name":"s","type":"string","default":"","doc":"the text"}],"returns":[],"docs":"Trims.","docs_line":2,"flags":[]}],"members":[]},
{"name":"scan","kind":"procedure","file":"/s/internal/string/scan.jai","line":9,"docs":"","value":"","overloads":[],"members":[]},
{"name":"LIMIT","kind":"constant","file":"/s/internal/string/bytes.jai","line":1,"docs":"","value":"4","overloads":[],"members":[]}]}"#;
        let module = parse("String", json, Path::new("/s")).unwrap();
        assert_eq!(module.file, "String/module.jai");
        assert_eq!(module.items[0].file, "internal/string/bytes.jai");
        assert_eq!(module.items[0].overloads[0].params[0].doc, "the text");
        let groups: Vec<_> = module.items.iter().map(|i| i.group.as_str()).collect();
        assert_eq!(groups, ["Bytes", "Scan", ""]);
        assert_eq!(module.title(), "String");
    }
}
