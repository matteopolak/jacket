//! What the site documents: modules, their items and overloads, read from the JSON
//! `jaic -doc_model` prints for each library of the standard library.
use serde::Deserialize;
mod wire;
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
    pub signature: String,
    pub platforms: Vec<String>,
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
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
}

/// One compiler-authored definition and its verified private source identity.
#[derive(Clone, Debug)]
pub struct Definition {
    pub kind: Kind,
    pub docs: String,
    pub file: String,
    pub native_file: PathBuf,
    pub input_sha256: String,
    pub line: usize,
    pub start: u32,
    pub end: u32,
    pub platforms: Vec<String>,
    pub guards: Vec<CompileGuard>,
    pub scope: Vec<String>,
    pub tokens: Vec<DeclarationToken>,
    pub members: Vec<Member>,
    pub refs: Vec<TypeRef>,
}

#[derive(Clone, Debug)]
pub struct CompileGuard {
    pub native_file: PathBuf,
    pub input_sha256: String,
    pub start: u32,
    pub end: u32,
    pub taken: bool,
}

#[derive(Clone, Debug)]
pub struct DeclarationToken {
    pub text: String,
    pub start: u32,
    pub end: u32,
    pub reference: bool,
    pub names: Vec<String>,
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
    pub definitions: Vec<Definition>,
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
    pub dependencies: Vec<(PathBuf, String)>,
    /// The module's route and display identity under the selected documentation library.
    pub id: String,
    /// Compiler documentation identity; the site's selected library root does not change it.
    pub doc_key: String,
    /// Executable import path: actual stdlib-relative identity, or the selected external path.
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
    /// The import used by examples and probes, such as `Extensions/Widget` on a `Widget` page.
    /// Synthetic modules without an explicit context keep their selected-library import.
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

    pub fn item(&self, name: &str) -> Option<&Item> {
        self.items.iter().find(|i| i.name == name)
    }
}

/// `file` below `stdlib`, `/`-separated; the path itself when it is elsewhere.
fn below(stdlib: &Path, file: &str) -> String {
    let path = PathBuf::from(file);
    path.strip_prefix(stdlib)
        .map_or_else(|_| path.clone(), Path::to_path_buf)
        .to_string_lossy()
        .replace('\\', "/")
}

/// Read a compiler model under the selected library's `root`, keeping its compiler link key.
/// For example, `Widget` below a selected `Extensions` root links as `Extensions/Widget`.
/// Missing fields and malformed references fail before any site data is assembled.
pub fn parse(id: &str, json: &str, root: &Path, stdlib: &Path) -> Result<Module, String> {
    let wire: wire::Module = serde_json::from_str(json)
        .map_err(|e| format!("{id}: compiler documentation model: {e}"))?;
    if wire.schema != "jai-doc-model" || wire.schema_version != 1 {
        return Err(format!(
            "{id}: unsupported compiler documentation model {}/{}, expected jai-doc-model/1",
            wire.schema, wire.schema_version
        ));
    }
    for (index, dependency) in wire.dependencies.iter().enumerate() {
        if dependency.sha256.len() != 64
            || !dependency
                .sha256
                .bytes()
                .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
        {
            return Err(format!(
                "{id}: /dependencies/{index}/sha256: expected lowercase SHA-256"
            ));
        }
    }
    let refs = |rows: Vec<wire::TypeRef>| {
        rows.into_iter()
            .map(|(path, file, line, private)| TypeRef {
                path,
                file: below(root, &file),
                line,
                private,
            })
            .collect()
    };
    let param = |p: wire::Param| Param {
        name: p.name,
        ty: p.ty,
        default: p.default,
        doc: p.doc,
        refs: refs(p.refs),
    };
    let mut items = Vec::new();
    let mut private = Vec::new();
    for i in wire.items {
        if matches!(
            i.kind,
            Kind::Struct | Kind::Union | Kind::Enum | Kind::EnumFlags
        ) && i.definitions.is_empty()
        {
            return Err(format!(
                "{id}: item {}: definitions are required for a written declaration",
                i.name
            ));
        }
        let mut definitions = Vec::new();
        for definition in i.definitions {
            if definition.line == 0
                || definition.start >= definition.end
                || definition.tokens.is_empty()
            {
                return Err(format!(
                    "{id}: item {}: definition source bounds and tokens must be nonempty",
                    i.name
                ));
            }
            if definition
                .guards
                .iter()
                .any(|guard| guard.tokens.is_empty() || guard.tokens.iter().any(String::is_empty))
            {
                return Err(format!(
                    "{id}: item {}: compile guards must contain lexical tokens",
                    i.name
                ));
            }
            let mut previous = definition.start;
            for token in &definition.tokens {
                if token.start < previous
                    || token.start >= token.end
                    || token.end > definition.end
                    || token.text.is_empty()
                {
                    return Err(format!(
                        "{id}: item {}: definition token has invalid source bounds",
                        i.name
                    ));
                }
                previous = token.end;
            }
            let mut guards = Vec::new();
            for guard in definition.guards {
                if guard.start >= guard.end {
                    return Err(format!(
                        "{id}: item {}: compile guard has invalid source bounds",
                        i.name
                    ));
                }
                let digest=wire.dependencies.iter().find(|input|input.file==guard.file).map(|input|input.sha256.clone())
                    .ok_or_else(||format!("{id}: item {}: compile guard source is absent from compiler dependencies",i.name))?;
                guards.push(CompileGuard {
                    native_file: PathBuf::from(guard.file),
                    input_sha256: digest,
                    start: guard.start,
                    end: guard.end,
                    taken: guard.taken,
                });
            }
            let input_sha256 = wire
                .dependencies
                .iter()
                .find(|input| input.file == definition.file)
                .map(|input| input.sha256.clone())
                .ok_or_else(|| {
                    format!(
                        "{id}: item {}: definition source is absent from compiler dependencies",
                        i.name
                    )
                })?;
            definitions.push(Definition {
                kind: definition.kind,
                docs: definition.docs,
                file: below(root, &definition.file),
                native_file: PathBuf::from(&definition.file),
                input_sha256,
                line: definition.line,
                start: definition.start,
                end: definition.end,
                platforms: definition.platforms,
                scope: definition.scope,
                guards,
                tokens: definition
                    .tokens
                    .into_iter()
                    .map(|token| DeclarationToken {
                        text: token.text,
                        start: token.start,
                        end: token.end,
                        reference: token.reference,
                        names: token.names,
                    })
                    .collect(),
                members: definition
                    .members
                    .into_iter()
                    .map(|member| Member {
                        name: member.name,
                        ty: member.ty,
                        doc: member.doc,
                        refs: refs(member.refs),
                    })
                    .collect(),
                refs: refs(definition.refs),
            });
        }
        let item = Item {
            definitions,
            name: i.name,
            kind: i.kind,
            file: below(root, &i.file),
            line: i.line,
            docs: i.docs,
            value: i.value,
            scope: i.scope,
            overloads: i
                .overloads
                .into_iter()
                .map(|o| Overload {
                    signature: o.signature,
                    platforms: o.platforms,
                    file: below(root, &o.file),
                    line: o.line,
                    params: o.params.into_iter().map(&param).collect(),
                    returns: o.returns.into_iter().map(&param).collect(),
                    docs: o.docs,
                    docs_line: o.docs_line,
                    flags: o.flags,
                })
                .collect(),
            members: i
                .members
                .into_iter()
                .map(|m| Member {
                    name: m.name,
                    ty: m.ty,
                    doc: m.doc,
                    refs: refs(m.refs),
                })
                .collect(),
            refs: refs(i.refs),
            also: i
                .also
                .into_iter()
                .map(|(f, l)| (below(root, &f), l))
                .collect(),
            group: String::new(),
            platforms: i.platforms,
            since: String::new(),
            experimental: i.experimental,
            binding: i.binding,
        };
        if i.private {
            private.push(item);
        } else {
            items.push(item);
        }
    }
    let mut module = Module {
        dependencies: wire
            .dependencies
            .into_iter()
            .map(|d| (PathBuf::from(d.file), d.sha256))
            .collect(),
        id: id.to_owned(),
        doc_key: jaic::doc_comments::module_key(&root.join(id), stdlib),
        import_path: jaic::doc_comments::stdlib_module_key(&root.join(id), stdlib)
            .unwrap_or_else(|| id.to_owned()),
        file: below(root, &wire.file),
        docs: wire.docs,
        docs_line: wire.docs_line,
        platforms: wire.platforms,
        items,
        private,
        implicit: wire.implicit,
        category: None,
        group: String::new(),
        wasm: false,
        since: String::new(),
        experimental: wire.experimental,
        parameters: wire
            .parameters
            .into_iter()
            .map(|p| Parameter {
                name: p.name,
                ty: p.ty,
                default: p.default,
                doc: p.doc,
                refs: refs(p.refs),
            })
            .collect(),
        unresolved: wire
            .unresolved
            .into_iter()
            .map(|u| format!("{}:{}: {}", below(root, &u.file), u.line, u.text))
            .collect(),
        test_notes: crate::test::options::header_notes(
            &std::fs::read_to_string(root.join(id).join("module.jai")).unwrap_or_default(),
        ),
    };
    assign_groups(&mut module);
    Ok(module)
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
    fn import_context_defaults_to_the_selected_path_for_synthetic_modules() {
        let mut module = Module {
            id: "Acme/Widget".into(),
            ..Module::default()
        };
        assert_eq!(module.import_name(), "Acme/Widget");
        module.import_path = "Extensions/Widget".into();
        assert_eq!(module.import_name(), "Extensions/Widget");
        assert_eq!(module.id, "Acme/Widget");
    }

    #[test]
    fn definition_facts_require_ordered_spans_guards_and_pinned_source() {
        let definition = serde_json::json!({"kind":"struct","docs":"","file":"/s/Example/module.jai","line":1,"start":0,"end":5,"platforms":["MACOS"],"guards":[{"file":"/s/Example/module.jai","start":0,"end":5,"tokens":["OS","==",".","MACOS"],"taken":true}],"scope":[],"tokens":[{"text":"Entry","start":0,"end":5,"statement":false,"reference":false,"names":[],"docs":""}],"members":[]});
        let item = serde_json::json!({"name":"Entry","kind":"struct","file":"/s/Example/module.jai","line":1,"docs":"","value":"","platforms":[],"experimental":false,"overloads":[],"members":[],"scope":[],"definitions":[definition]});
        let valid = serde_json::json!({"schema":"jai-doc-model","schemaVersion":1,"file":"/s/Example/module.jai","implicit":false,"docs":"","docs_line":0,"platforms":[],"experimental":false,"parameters":[],"items":[item],"unresolved":[],"dependencies":[{"file":"/s/Example/module.jai","sha256":"a".repeat(64)}]});
        let parsed = parse(
            "Example",
            &valid.to_string(),
            Path::new("/s"),
            Path::new("/s"),
        )
        .unwrap();
        assert_eq!(parsed.items[0].definitions[0].input_sha256, "a".repeat(64));
        for (pointer, value) in [
            ("/items/0/definitions/0/start", serde_json::json!(5)),
            ("/items/0/definitions/0/line", serde_json::json!(0)),
            ("/items/0/definitions/0/tokens/0/end", serde_json::json!(6)),
            (
                "/items/0/definitions/0/tokens/0/reference",
                serde_json::json!("false"),
            ),
            (
                "/items/0/definitions/0/guards/0/tokens",
                serde_json::json!([]),
            ),
            ("/dependencies", serde_json::json!([])),
        ] {
            let mut broken = valid.clone();
            *broken.pointer_mut(pointer).unwrap() = value;
            assert!(
                parse(
                    "Example",
                    &broken.to_string(),
                    Path::new("/s"),
                    Path::new("/s")
                )
                .is_err(),
                "accepted invalid definition field {pointer}"
            );
        }
        let mut unknown = valid;
        unknown["items"][0]["definitions"][0]["source_text"] =
            serde_json::json!("original author spacing");
        assert!(
            parse(
                "Example",
                &unknown.to_string(),
                Path::new("/s"),
                Path::new("/s")
            )
            .is_err()
        );
    }

    /// A compiler payload is a versioned contract, not best-effort presentation data.
    #[test]
    fn malformed_compiler_models_are_rejected() {
        let valid = serde_json::json!({
            "schema": "jai-doc-model", "schemaVersion": 1,
            "file": "Example/module.jai", "implicit": false, "docs": "",
            "docs_line": 0, "platforms": [], "experimental": false,
            "parameters": [], "items": [], "unresolved": [], "dependencies": [{"file":"Example/module.jai","sha256":"a".repeat(64)}]
        });
        assert!(
            parse(
                "Example",
                &valid.to_string(),
                Path::new("/s"),
                Path::new("/s")
            )
            .is_ok()
        );
        for (field, bad) in [
            ("schemaVersion", serde_json::json!(2)),
            ("schemaVersion", serde_json::json!("1")),
            ("implicit", serde_json::json!("false")),
            ("items", serde_json::json!(null)),
        ] {
            let mut model = valid.clone();
            model[field] = bad;
            assert!(
                parse(
                    "Example",
                    &model.to_string(),
                    Path::new("/s"),
                    Path::new("/s")
                )
                .is_err(),
                "{field}"
            );
        }
        let mut missing = valid.clone();
        missing.as_object_mut().unwrap().remove("file");
        assert!(
            parse(
                "Example",
                &missing.to_string(),
                Path::new("/s"),
                Path::new("/s")
            )
            .is_err()
        );
        let item = serde_json::json!({
            "name": "Entry", "kind": "struct", "file": "Example/module.jai",
            "line": 1, "docs": "", "value": "", "platforms": [],
            "experimental": false, "overloads": [], "members": [], "scope": [],
            "definitions": [{"kind":"struct","docs":"","file":"Example/module.jai","line":1,"start":0,"end":5,"platforms":[],"guards":[],"scope":[],"tokens":[{"text":"Entry","start":0,"end":5,"statement":false,"reference":false,"names":[],"docs":""}],"members":[]}]
        });
        let mut complete = valid.clone();
        complete["items"] = serde_json::json!([item.clone()]);
        assert!(
            parse(
                "Example",
                &complete.to_string(),
                Path::new("/s"),
                Path::new("/s")
            )
            .is_ok()
        );
        for (field, bad) in [
            ("kind", serde_json::json!("unknown")),
            ("binding", serde_json::json!("false")),
            (
                "declaration",
                serde_json::json!("Entry :: struct { aligned: int; }"),
            ),
            ("definitions", serde_json::json!([])),
            (
                "declaration_tokens",
                serde_json::json!([["Entry", false, [], ""]]),
            ),
            ("decl_refs", serde_json::json!([])),
            (
                "refs",
                serde_json::json!([["Entry", "Example/module.jai", "one", false]]),
            ),
            (
                "refs",
                serde_json::json!([["Entry", "Example/module.jai", 1]]),
            ),
        ] {
            let mut model = valid.clone();
            let mut broken = item.clone();
            broken[field] = bad;
            model["items"] = serde_json::json!([broken]);
            assert!(
                parse(
                    "Example",
                    &model.to_string(),
                    Path::new("/s"),
                    Path::new("/s")
                )
                .is_err(),
                "{field}"
            );
        }
    }

    #[test]
    fn items_come_out_with_groups_and_relative_files() {
        let json = r#"{"file":"/s/String/module.jai","docs":"A module.","docs_line":1,"platforms":["MACOS"],"items":[{"name":"trim","kind":"procedure","file":"/s/internal/string/bytes.jai","line":3,"docs":"","value":"","overloads":[{"file":"/s/internal/string/bytes.jai","line":3,"signature":"","params":[{"name":"s","type":"string","default":"","doc":"the text"}],"returns":[],"docs":"Trims.","docs_line":2,"flags":[],"platforms":[]}],"members":[],"platforms":[],"experimental":false,"scope":[]},{"name":"scan","kind":"procedure","file":"/s/internal/string/scan.jai","line":9,"docs":"","value":"","overloads":[],"members":[],"platforms":[],"experimental":false,"scope":[]},{"name":"LIMIT","kind":"constant","file":"/s/internal/string/bytes.jai","line":1,"docs":"","value":"4","overloads":[],"members":[],"platforms":[],"experimental":false,"scope":[]}],"schema":"jai-doc-model","schemaVersion":1,"implicit":false,"experimental":false,"parameters":[],"unresolved":[],"dependencies":[]}"#;
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
    fn links_that_name_nothing_keep_their_place() {
        let json = r#"{"file":"/s/String/module.jai","docs":"","docs_line":1,"platforms":[],"items":[],"unresolved":[{"file":"/s/String/module.jai","line":7,"text":"[Gone]"}],"schema":"jai-doc-model","schemaVersion":1,"implicit":false,"experimental":false,"parameters":[],"dependencies":[]}"#;
        let module = parse("String", json, Path::new("/s"), Path::new("/s")).unwrap();
        assert_eq!(module.unresolved, ["String/module.jai:7: [Gone]"]);
    }
}
