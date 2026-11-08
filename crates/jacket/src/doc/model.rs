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
    pub refs: Vec<TypeRef>,
}

/// A name written in a type, and where the compiler found its declaration: the file below
/// `stdlib` and the line, as an item records them.
#[derive(Clone, Debug, Default)]
pub struct TypeRef {
    /// `Table` or `Hash.Table`, as written.
    pub path: String,
    pub file: String,
    pub line: usize,
    pub private: bool,
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
    pub refs: Vec<TypeRef>,
}

impl Item {
    /// For each member, the line of `declaration` that declares it: a line at the body's own
    /// indentation that starts with the name (or with a list of names, `a, b: int`). `None` when the line is not found, and for every
    /// member when a line holds an unfinished string.
    pub fn member_lines(&self) -> Vec<Option<usize>> {
        let lines: Vec<&str> = self.declaration.split('\n').collect();
        let none = vec![None; self.members.len()];
        if lines.iter().any(|l| l.matches('"').count() % 2 == 1) {
            return none;
        }
        let indent = |l: &str| l.len() - l.trim_start().len();
        let Some(base) = lines
            .iter()
            .skip(1)
            .find(|l| !l.trim().is_empty())
            .map(|l| indent(l))
        else {
            return none;
        };
        let mut from = 1;
        self.members
            .iter()
            .map(|m| {
                let found = (from..lines.len()).find(|&i| {
                    let written = lines[i].trim_start();
                    let written = written.strip_prefix("using ").unwrap_or(written);
                    let head = written.split([':', ';', '=']).next().unwrap_or_default();
                    indent(lines[i]) == base
                        && !m.name.is_empty()
                        && head.split(',').any(|name| name.trim() == m.name)
                });
                // `a, b: int;` declares both on one line.
                from = found.unwrap_or(from);
                found
            })
            .collect()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Kind {
    #[default]
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

#[derive(Clone, Debug, Default)]
pub struct Item {
    pub name: String,
    pub kind: Kind,
    pub file: String,
    pub line: usize,
    pub docs: String,
    pub value: String,
    pub scope: Vec<String>,
    pub overloads: Vec<Overload>,
    pub members: Vec<Member>,
    /// The declarations the names in `value` stand for.
    pub refs: Vec<TypeRef>,
    /// The heading it is listed under on the module page; empty for none.
    /// Other declarations (file below `stdlib`, line) the item stands for: the same type
    /// declared in each branch of an `#if`.
    pub also: Vec<(String, usize)>,
    pub group: String,
    /// The systems an `#if` limits it to (`WINDOWS`); empty for any.
    pub platforms: Vec<String>,
    /// The release it first appeared in (`0.24.0`); empty when unknown. See `since.rs`.
    pub since: String,
    /// Tagged `@experimental` in its docs.
    pub experimental: bool,
    /// Declared by a native binding: badged "foreign" and listed after the module's own items.
    pub binding: bool,
    /// A struct, union or enum as written, without comments; empty for anything else.
    pub declaration: String,
    /// The declarations the type names in `declaration` stand for.
    pub decl_refs: Vec<TypeRef>,
}

/// A parameter of a module (`#module_parameters`).
#[derive(Clone, Debug, Default)]
pub struct Parameter {
    pub name: String,
    pub ty: String,
    pub default: String,
    pub doc: String,
    /// What the names in the type and the default resolve to.
    pub refs: Vec<TypeRef>,
}

#[derive(Clone, Debug, Default)]
pub struct Module {
    /// `String`, `Extensions/Http`: the page's place below the documented folder. Documenting
    /// `stdlib/Extensions/Http` alone makes it `Http`.
    pub id: String,
    /// The name the compiler's `jaidoc:` links use for the module; empty means `id`. It does not
    /// change with the folder being documented: `Extensions/Http` stays so for `Http` alone.
    pub doc_key: String,
    /// What an example imports the module as; empty means `id`.
    pub import_path: String,
    pub file: String,
    pub docs: String,
    /// The line the module's own doc comment starts on, 0 when it has none.
    pub docs_line: usize,
    pub platforms: Vec<String>,
    pub items: Vec<Item>,
    /// Types that items name but the module does not export: they have pages, so the types
    /// that name them can link there, but no list, search or sitemap has them.
    pub private: Vec<Item>,
    /// Every program gets it without importing it.
    pub implicit: bool,
    /// The `[[doc.category]]` it is listed under, as an index; set once every module is read.
    pub category: Option<usize>,
    /// The `[[doc.group]]` it is in; empty for none.
    pub group: String,
    /// Whether the module builds for `-os wasm` and has something to use there.
    pub wasm: bool,
    /// The release its items first appeared in, when they all did after the oldest one known.
    pub since: String,
    /// Tagged `@experimental` in its opening comment.
    pub experimental: bool,
    pub parameters: Vec<Parameter>,
    /// Links in its doc comments that read as intra-doc links and name nothing, as
    /// `file:line: [text]`.
    pub unresolved: Vec<String>,
    /// The notes of the `// test:` lines that open its `module.jai`, which every example of the
    /// module takes as `jacket test` gives them to its doc tests.
    pub test_notes: Vec<String>,
}

impl Module {
    /// What `jaidoc:` links call the module.
    pub fn key(&self) -> &str {
        if self.doc_key.is_empty() {
            &self.id
        } else {
            &self.doc_key
        }
    }

    /// The path an example's `#import` names: `Extensions/Http`, also when only that folder is
    /// documented.
    pub fn import_name(&self) -> &str {
        if self.import_path.is_empty() {
            &self.id
        } else {
            &self.import_path
        }
    }

    /// `Http` for `Extensions/Http`.
    pub fn title(&self) -> &str {
        self.id.rsplit('/').next().unwrap_or(&self.id)
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
        if self.wasm && !out.contains(&"Wasm") {
            out.push("Wasm");
        }
        out
    }

    /// Every system a module page lists, with how well this module runs there: the desktop
    /// systems and Wasm always, Android and iOS when the module names them. A system is partial
    /// when some items are missing on it; the count is how many.
    pub fn support(&self) -> Vec<(&'static str, Support)> {
        let ours = self.systems();
        SYSTEMS
            .iter()
            .copied()
            .filter(|s| !matches!(*s, "Android" | "iOS") || ours.contains(s))
            .map(|s| {
                if !ours.contains(&s) {
                    return (s, Support::None);
                }
                let missing = self
                    .items
                    .iter()
                    .filter(|it| self.item_missing(it).contains(&s))
                    .count();
                (
                    s,
                    if missing == 0 {
                        Support::Full
                    } else {
                        Support::Partial(missing)
                    },
                )
            })
            .collect()
    }

    /// The systems its module runs on that `item` does not, in badge order. Empty when it runs
    /// everywhere its module does.
    pub fn item_missing(&self, item: &Item) -> Vec<&'static str> {
        if item.platforms.is_empty() {
            return Vec::new();
        }
        let theirs: Vec<&'static str> = item
            .platforms
            .iter()
            .filter_map(|p| system_name(p))
            .collect();
        self.systems()
            .into_iter()
            .filter(|s| !theirs.contains(s))
            .collect()
    }

    /// The systems an item page lists, the same ones as its module's page, each full or none.
    pub fn item_support(&self, item: &Item) -> Vec<(&'static str, Support)> {
        let missing = self.item_missing(item);
        self.support()
            .into_iter()
            .map(|(s, module)| match module {
                Support::None => (s, Support::None),
                _ if missing.contains(&s) => (s, Support::None),
                _ => (s, Support::Full),
            })
            .collect()
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

fn type_refs(v: &Value, stdlib: &Path) -> Vec<TypeRef> {
    type_refs_of(v, "refs", stdlib)
}

fn type_refs_of(v: &Value, key: &str, stdlib: &Path) -> Vec<TypeRef> {
    list(v, key)
        .iter()
        .filter_map(|r| {
            let r = r.as_array()?;
            Some(TypeRef {
                path: r.first()?.as_str()?.to_owned(),
                file: below(stdlib, r.get(1)?.as_str()?),
                line: r.get(2)?.as_u64()? as usize,
                private: r.get(3).and_then(Value::as_bool).unwrap_or(false),
            })
        })
        .collect()
}

fn params(v: &Value, key: &str, stdlib: &Path) -> Vec<Param> {
    list(v, key)
        .iter()
        .map(|p| Param {
            name: text(p, "name"),
            ty: text(p, "type"),
            default: text(p, "default"),
            doc: text(p, "doc"),
            refs: type_refs(p, stdlib),
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

/// An item and whether the module exports it.
type Flagged = (bool, Item);

/// The compiler's name for the module folder `root/id` in links, and the path that imports it.
/// Inside `stdlib` both are the path below it; elsewhere a link names the folder alone, and an
/// import is `id`, found through `-import_dir root`.
fn identity(id: &str, root: &Path, stdlib: &Path) -> (String, String) {
    let canonical = |p: &Path| p.canonicalize().unwrap_or_else(|_| p.to_path_buf());
    let dir = canonical(&root.join(id));
    match dir.strip_prefix(canonical(stdlib)) {
        Ok(below) => {
            let key = jacket_config::slashed(below);
            (key.clone(), key)
        }
        Err(_) => (
            dir.file_name()
                .map_or_else(String::new, |n| n.to_string_lossy().into_owned()),
            id.to_owned(),
        ),
    }
}

/// The module `id` (a folder below `root`) from what `jaic -doc_model` printed. `stdlib` is the
/// standard library folder, which names a module for links and imports whatever `root` is.
pub fn parse(id: &str, json: &str, root: &Path, stdlib: &Path) -> Result<Module, String> {
    let (doc_key, import_path) = identity(id, root, stdlib);
    let v: Value = serde_json::from_str(json).map_err(|e| format!("{id}: {e}"))?;
    let (private, items): (Vec<Flagged>, Vec<Flagged>) = list(&v, "items")
        .iter()
        .map(|i| {
            let overloads: Vec<Overload> = list(i, "overloads")
                .iter()
                .map(|o| Overload {
                    file: below(stdlib, &text(o, "file")),
                    line: number(o, "line"),
                    params: params(o, "params", stdlib),
                    returns: params(o, "returns", stdlib),
                    docs: text(o, "docs"),
                    docs_line: number(o, "docs_line"),
                    flags: list(o, "flags")
                        .iter()
                        .filter_map(|f| f.as_str().map(str::to_owned))
                        .collect(),
                })
                .collect();
            let item = Item {
                name: text(i, "name"),
                kind: Kind::parse(&text(i, "kind")),
                file: below(stdlib, &text(i, "file")),
                line: number(i, "line"),
                docs: text(i, "docs"),
                value: text(i, "value"),
                scope: list(i, "scope")
                    .iter()
                    .filter_map(|n| n.as_str().map(str::to_owned))
                    .collect(),
                refs: type_refs(i, stdlib),
                binding: i.get("binding").and_then(Value::as_bool).unwrap_or(false),
                overloads,
                members: list(i, "members")
                    .iter()
                    .map(|m| Member {
                        name: text(m, "name"),
                        ty: text(m, "type"),
                        doc: text(m, "doc"),
                        refs: type_refs(m, stdlib),
                    })
                    .collect(),
                also: list(i, "also")
                    .iter()
                    .filter_map(|a| {
                        let a = a.as_array()?;
                        Some((
                            below(stdlib, a.first()?.as_str()?),
                            a.get(1)?.as_u64()? as usize,
                        ))
                    })
                    .collect(),
                declaration: text(i, "declaration"),
                decl_refs: type_refs_of(i, "decl_refs", stdlib),
                group: String::new(),
                platforms: list(i, "platforms")
                    .iter()
                    .filter_map(|p| p.as_str().map(str::to_owned))
                    .collect(),
                since: String::new(),
                experimental: i.get("experimental").and_then(Value::as_bool) == Some(true),
            };
            (
                i.get("private").and_then(Value::as_bool) == Some(true),
                item,
            )
        })
        .partition(|(private, _)| *private);
    let (private, items): (Vec<Item>, Vec<Item>) = (
        private.into_iter().map(|(_, i)| i).collect(),
        items.into_iter().map(|(_, i)| i).collect(),
    );
    let mut module = Module {
        id: id.to_owned(),
        doc_key,
        import_path,
        file: below(stdlib, &text(&v, "file")),
        docs: text(&v, "docs"),
        docs_line: number(&v, "docs_line"),
        platforms: list(&v, "platforms")
            .iter()
            .filter_map(|p| p.as_str().map(str::to_owned))
            .collect(),
        items,
        private,
        implicit: v.get("implicit").and_then(Value::as_bool) == Some(true),
        category: None,
        group: String::new(),
        wasm: false,
        since: String::new(),
        experimental: v.get("experimental").and_then(Value::as_bool) == Some(true),
        parameters: list(&v, "parameters")
            .iter()
            .map(|p| Parameter {
                name: text(p, "name"),
                ty: text(p, "type"),
                default: text(p, "default"),
                doc: text(p, "doc"),
                refs: type_refs(p, stdlib),
            })
            .collect(),
        unresolved: list(&v, "unresolved")
            .iter()
            .map(|u| {
                format!(
                    "{}:{}: {}",
                    below(stdlib, &text(u, "file")),
                    number(u, "line"),
                    text(u, "text")
                )
            })
            .collect(),
        test_notes: crate::test::options::header_notes(
            &std::fs::read_to_string(stdlib.join(id).join("module.jai")).unwrap_or_default(),
        ),
    };
    assign_groups(&mut module);
    Ok(module)
}

/// The systems a badge can name, in the order they are shown.
/// How well a module runs on one system.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Support {
    Full,
    /// Available, but this many items are not.
    Partial(usize),
    None,
}

const SYSTEMS: [&str; 6] = ["macOS", "Linux", "Windows", "Android", "iOS", "Wasm"];

/// The badge for the compiler's name of a system (`MACOS`).
fn system_name(os: &str) -> Option<&'static str> {
    Some(match os {
        "MACOS" => "macOS",
        "LINUX" => "Linux",
        "WINDOWS" => "Windows",
        "ANDROID" => "Android",
        "IOS" => "iOS",
        "WASM" => "Wasm",
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
        let module = parse("String", json, Path::new("/s"), Path::new("/s")).unwrap();
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
    fn a_module_documented_alone_keeps_the_names_the_compiler_uses() {
        let json = r#"{"file":"/s/Extensions/Json/module.jai","docs":"","docs_line":1,"platforms":[],"items":[]}"#;
        let inside = parse("Json", json, Path::new("/s/Extensions"), Path::new("/s")).unwrap();
        assert_eq!(inside.id, "Json");
        assert_eq!(inside.key(), "Extensions/Json");
        assert_eq!(inside.import_name(), "Extensions/Json");
        let outside = parse("Acme/Json", json, Path::new("/lib"), Path::new("/s")).unwrap();
        assert_eq!(outside.key(), "Json");
        assert_eq!(outside.import_name(), "Acme/Json");
    }

    #[test]
    fn links_that_name_nothing_keep_their_place() {
        let json = r#"{"file":"/s/String/module.jai","docs":"","docs_line":1,"platforms":[],"items":[],"unresolved":[{"file":"/s/String/module.jai","line":7,"text":"[Gone]"}]}"#;
        let module = parse("String", json, Path::new("/s"), Path::new("/s")).unwrap();
        assert_eq!(module.unresolved, ["String/module.jai:7: [Gone]"]);
    }

    #[test]
    fn items_get_badges_only_where_they_differ_from_the_module() {
        let json = r#"{"file":"/s/File/module.jai","docs":"","docs_line":1,"platforms":[],"items":[
{"name":"open","kind":"procedure","file":"/s/File/a.jai","line":1,"docs":"","value":"","platforms":[],"overloads":[],"members":[]},
{"name":"map","kind":"procedure","file":"/s/File/a.jai","line":2,"docs":"","value":"","platforms":["WINDOWS"],"overloads":[],"members":[]},
{"name":"all","kind":"procedure","file":"/s/File/a.jai","line":3,"docs":"","value":"","platforms":["MACOS","LINUX","WINDOWS","WASM"],"overloads":[],"members":[]}]}"#;
        let mut module = parse("File", json, Path::new("/s"), Path::new("/s")).unwrap();
        assert_eq!(module.systems(), ["macOS", "Linux", "Windows"]);
        assert!(module.item_missing(&module.items[0]).is_empty());
        assert_eq!(module.item_missing(&module.items[1]), ["macOS", "Linux"]);
        module.wasm = true;
        assert_eq!(module.systems(), ["macOS", "Linux", "Windows", "Wasm"]);
        assert!(module.item_missing(&module.items[2]).is_empty());
        assert_eq!(
            module.item_missing(&module.items[1]),
            ["macOS", "Linux", "Wasm"]
        );
        let support = module.item_support(&module.items[1]);
        assert_eq!(support[2], ("Windows", Support::Full));
        assert_eq!(support[0], ("macOS", Support::None));
    }
}
