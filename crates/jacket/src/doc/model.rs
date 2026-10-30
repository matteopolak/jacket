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
    /// The line of the doc comment above it, 0 when it has none.
    pub docs_line: usize,
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
    /// The systems an `#if` limits it to (`WINDOWS`); empty for any.
    pub platforms: Vec<String>,
    /// The release it first appeared in (`0.24.0`); empty when unknown. See `since.rs`.
    pub since: String,
}

#[derive(Clone, Debug)]
pub struct Module {
    /// `String`, `Extensions/Http`: what `#import` takes.
    pub id: String,
    pub file: String,
    pub docs: String,
    /// The line the module's own doc comment starts on, 0 when it has none.
    pub docs_line: usize,
    pub platforms: Vec<String>,
    pub items: Vec<Item>,
    /// From `stdlib/categories.toml`; set once every module is read.
    pub category: String,
    /// Whether the module builds for `-os wasm` and has something to use there.
    pub wasm: bool,
    /// The release its items first appeared in, when they all did after the oldest one known.
    pub since: String,
}

impl Module {
    /// `Http` for `Extensions/Http`.
    pub fn title(&self) -> &str {
        self.id.rsplit('/').next().unwrap_or(&self.id)
    }

    /// The heading of the sidebar and the home page it is listed under.
    pub fn section(&self) -> &'static str {
        if self.id.starts_with("Extensions/") {
            "Extensions"
        } else {
            "Standard library"
        }
    }

    /// The systems it is for, in the order the badges show them: the targets it asserts, else
    /// the three desktop ones, and `wasm` when it builds there.
    pub fn systems(&self) -> Vec<&'static str> {
        let mut out: Vec<&'static str> = if self.platforms.is_empty() {
            vec!["macOS", "Linux", "Windows"]
        } else {
            let mut named: Vec<&'static str> = self
                .platforms
                .iter()
                .filter_map(|p| system_name(p))
                .collect();
            named.sort_by_key(|n| SYSTEMS.iter().position(|s| s == n));
            named
        };
        if self.wasm && !out.contains(&"wasm") {
            out.push("wasm");
        }
        out
    }

    /// The badges for `item`: the systems it is available on, when that is fewer than its
    /// module's. Empty when it matches the module.
    pub fn item_systems(&self, item: &Item) -> Vec<&'static str> {
        if item.platforms.is_empty() {
            return Vec::new();
        }
        let theirs: Vec<&'static str> = item
            .platforms
            .iter()
            .filter_map(|p| system_name(p))
            .collect();
        let ours = self.systems();
        let shown: Vec<&'static str> = ours
            .iter()
            .copied()
            .filter(|s| theirs.contains(s))
            .collect();
        if shown.len() == ours.len() {
            Vec::new()
        } else {
            shown
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
                    docs_line: number(o, "docs_line"),
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
                platforms: list(i, "platforms")
                    .iter()
                    .filter_map(|p| p.as_str().map(str::to_owned))
                    .collect(),
                since: String::new(),
            }
        })
        .collect();
    let mut module = Module {
        id: id.to_owned(),
        file: below(stdlib, &text(&v, "file")),
        docs: text(&v, "docs"),
        docs_line: number(&v, "docs_line"),
        platforms: list(&v, "platforms")
            .iter()
            .filter_map(|p| p.as_str().map(str::to_owned))
            .collect(),
        items,
        category: String::new(),
        wasm: false,
        since: String::new(),
    };
    assign_groups(&mut module);
    Ok(module)
}

/// The systems a badge can name, in the order they are shown.
const SYSTEMS: [&str; 6] = ["macOS", "Linux", "Windows", "Android", "iOS", "wasm"];

/// The badge for the compiler's name of a system (`MACOS`).
fn system_name(os: &str) -> Option<&'static str> {
    Some(match os {
        "MACOS" => "macOS",
        "LINUX" => "Linux",
        "WINDOWS" => "Windows",
        "ANDROID" => "Android",
        "IOS" => "iOS",
        "WASM" => "wasm",
        _ => return None,
    })
}

/// Words with their own casing in a heading made from a file name. The rest is sentence case.
const WORDS: &[&str] = &[
    "ABI", "ADB", "ADPCM", "API", "APK", "ASCII", "ASN1", "BMP", "COFF", "CPU", "CSV", "DFA",
    "DLL", "DWARF", "ELF", "GL", "GPU", "GUI", "HID", "HTML", "HTTP", "HTTPS", "IDCT", "IMA", "IO",
    "IOCP", "ID", "IP", "JKS", "JNI", "JPEG", "JSON", "LZ4", "MD5", "MSF", "NDK", "NFA", "ORM",
    "OS", "PCM", "PDB", "PNG", "PNM", "POSIX", "QOI", "RNG", "SCRAM", "SDK", "SDL", "SHA", "SIMD",
    "SQL", "TBD", "TCP", "TLS", "UI", "URL", "UTF8", "VM", "WAV", "X11", "X64", "X86", "X509",
    "XML", "ZIP", "CodeView", "Cocoa", "Linux", "Mach-O", "macOS", "Unix", "Windows",
];

/// `bytes.jai` is "Bytes", `self-test.jai` is "Self test", `jks-generation.jai` is "JKS
/// generation": sentence case, with the words of `WORDS` as spelled there.
fn title_case(stem: &str) -> String {
    let words: Vec<String> = stem
        .split(['-', '_', ' '])
        .filter(|w| !w.is_empty())
        .enumerate()
        .map(|(i, w)| {
            let lower = w.to_lowercase();
            let lower = if lower == "macho" {
                "mach-o".to_owned()
            } else {
                lower
            };
            if let Some(known) = WORDS.iter().find(|k| k.to_lowercase() == lower) {
                (*known).to_owned()
            } else if i == 0 {
                let mut chars = lower.chars();
                chars
                    .next()
                    .map(|c| c.to_uppercase().collect::<String>() + chars.as_str())
                    .unwrap_or_default()
            } else {
                lower
            }
        })
        .collect();
    words.join(" ")
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
        assert_eq!(title_case("jks-generation"), "JKS generation");
        assert_eq!(title_case("self-test"), "Self test");
        assert_eq!(title_case("utf8_tables"), "UTF8 tables");
    }

    #[test]
    fn items_get_badges_only_where_they_differ_from_the_module() {
        let json = r#"{"file":"/s/File/module.jai","docs":"","docs_line":1,"platforms":[],"items":[
{"name":"open","kind":"procedure","file":"/s/File/a.jai","line":1,"docs":"","value":"","platforms":[],"overloads":[],"members":[]},
{"name":"map","kind":"procedure","file":"/s/File/a.jai","line":2,"docs":"","value":"","platforms":["WINDOWS"],"overloads":[],"members":[]},
{"name":"all","kind":"procedure","file":"/s/File/a.jai","line":3,"docs":"","value":"","platforms":["MACOS","LINUX","WINDOWS","WASM"],"overloads":[],"members":[]}]}"#;
        let mut module = parse("File", json, Path::new("/s")).unwrap();
        assert_eq!(module.systems(), ["macOS", "Linux", "Windows"]);
        assert!(module.item_systems(&module.items[0]).is_empty());
        assert_eq!(module.item_systems(&module.items[1]), ["Windows"]);
        module.wasm = true;
        assert_eq!(module.systems(), ["macOS", "Linux", "Windows", "wasm"]);
        assert!(module.item_systems(&module.items[2]).is_empty());
        assert_eq!(module.item_systems(&module.items[1]), ["Windows"]);
    }
}
