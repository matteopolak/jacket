//! Which public declarations of a library have docs and an example, for `jaic -doc_gaps` and
//! the check that no new public declaration of the standard library goes undocumented.
//!
//! Public means what an importer of the module can name: a procedure, struct, enum or alias at
//! the top level of the module's files (the ones `#load`ed too) that is not under
//! `#scope_file` or `#scope_module`. Docs are the ones the editor shows, so an alias of a
//! documented declaration counts as documented (`docs::docs_site`). Fields, members, variables
//! and plain constants are not listed.
use super::forward::Files;
use crate::ast::DeclKind;
use crate::docs;
use crate::parser::ide_facts::{self, DeclClass, TopLevelKind, Visibility};
use crate::source::FileId;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// One public declaration, or a module.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub file: PathBuf,
    pub line: usize,
    /// `module`, `procedure`, `struct`, `enum` or `alias`.
    pub kind: &'static str,
    /// Empty for a module.
    pub symbol: String,
    pub documented: bool,
    /// The docs hold a code block that is not marked to be skipped.
    pub example: bool,
}

/// Whether `docs` hold a fenced block of Jai code.
fn has_example(docs: &str) -> bool {
    docs.lines().any(|line| {
        let Some(info) = line.trim_start().strip_prefix("```") else {
            return false;
        };
        let info = info.trim();
        info.is_empty() || info.split([',', ' ']).next() == Some("jai")
    })
}

fn has_body(decl: &crate::ast::Decl) -> bool {
    match decl.value.as_ref().map(|v| &v.kind) {
        Some(crate::ast::ExprKind::Proc(lit)) => lit.body.is_some(),
        _ => true,
    }
}

fn kind_of(decl: &crate::ast::Decl) -> Option<&'static str> {
    if decl.kind != DeclKind::Const {
        return None;
    }
    if ide_facts::alias_path(decl).is_some() {
        return Some("alias");
    }
    match ide_facts::decl_class(decl) {
        DeclClass::Procedure => Some("procedure"),
        DeclClass::Struct => Some("struct"),
        DeclClass::Enum => Some("enum"),
        _ => None,
    }
}

/// Every public procedure, struct, enum and alias of the library in `dir`, and the module
/// itself, with whether each has docs and an example. Sorted by file and line.
pub fn gaps(dir: &Path, stdlib: &Path) -> Vec<Entry> {
    let mut own = Vec::new();
    super::own_files(dir, true, &mut own);
    let module_file = super::canonical(&crate::paths::module_entry(dir));
    let mut queue: Vec<PathBuf> = own;
    let mut seen = BTreeSet::new();
    let mut files = Files::new(stdlib);
    let mut out = Vec::new();
    while let Some(file) = queue.pop() {
        let file = super::canonical(&file);
        if !seen.insert(file.clone()) {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&file) else {
            continue;
        };
        let parsed = crate::parser::parse_file_ide(FileId(0), &text);
        let base = file.parent().unwrap_or(Path::new("."));
        // A file that declares foreign procedures (no body) mirrors a native API: its types and
        // procedures are documented by their vendor, and are not listed.
        let binding = ide_facts::top_level(&parsed.file).iter().any(|item| {
            matches!(item.kind, TopLevelKind::Decl(decl)
                if decl.kind == DeclKind::Const
                    && ide_facts::decl_class(decl) == DeclClass::Procedure
                    && !has_body(decl))
        });
        let mut first_statement = None;
        for item in ide_facts::top_level(&parsed.file) {
            first_statement.get_or_insert(item.span.start as usize);
            match item.kind {
                TopLevelKind::Decl(decl) => {
                    let (Some(kind), Some(name)) = (kind_of(decl), decl.names.first()) else {
                        continue;
                    };
                    let start = item.span.start as usize;
                    if binding || item.visibility != Visibility::Export {
                        continue;
                    }
                    let (site, at) =
                        docs::docs_site(&mut files, file.clone(), name.span.start as usize);
                    let doc = docs::Resolver::text(&mut files, &site)
                        .and_then(|t| docs::declaration(&t, at));
                    out.push(Entry {
                        file: file.clone(),
                        line: text[..start].matches('\n').count() + 1,
                        kind,
                        symbol: name.name.as_str().to_owned(),
                        documented: doc.is_some(),
                        example: doc.as_deref().is_some_and(has_example),
                    });
                }
                TopLevelKind::Load(path, _) => queue.push(base.join(path)),
                _ => {}
            }
        }
        if file == module_file {
            // A comment directly above the first statement documents that, not the module.
            let opening = docs::opening(&text).filter(|o| {
                first_statement
                    .and_then(|at| docs::located_above(&text, at))
                    .is_none_or(|first| first.line != o.line)
            });
            out.push(Entry {
                file: file.clone(),
                line: 1,
                kind: "module",
                symbol: String::new(),
                documented: opening.is_some(),
                example: opening.as_ref().is_some_and(|o| has_example(&o.text)),
            });
        }
    }
    out.sort_by(|a, b| (&a.file, a.line).cmp(&(&b.file, b.line)));
    out
}

/// The records `jaic -doc_gaps` prints: `file<TAB>line<TAB>kind<TAB>symbol<TAB>doc<TAB>example`
/// with `doc` and `example` as `1` or `0`.
pub fn records(entries: &[Entry]) -> String {
    let mut out = String::new();
    for e in entries {
        out.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\n",
            e.file.to_string_lossy().replace(['\t', '\n'], " "),
            e.line,
            e.kind,
            e.symbol,
            u8::from(e.documented),
            u8::from(e.example)
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn library(name: &str, files: &[(&str, &str)]) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("jaic-doc-gaps-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        for (file, text) in files {
            let path = dir.join(file);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        }
        dir
    }

    fn rows(entries: &[Entry]) -> Vec<(&str, &str, bool, bool)> {
        entries
            .iter()
            .map(|e| (e.kind, e.symbol.as_str(), e.documented, e.example))
            .collect()
    }

    #[test]
    fn public_declarations_report_docs_and_examples() {
        let dir = library(
            "public",
            &[
                (
                    "module.jai",
                    "// A module.\n\nImpl :: #import,file \"impl.jai\";\n\n// Has an example.\n//\n// ```jai\n// documented();\n// ```\ndocumented :: () {}\n\nbare :: () {}\n\n// Types.\nPoint :: struct { x: int; }\n\nMode :: enum { A; }\n\nforwarded :: Impl.forwarded;\n\nmissing :: Impl.missing;\n\nCONSTANT :: 3;\n\n#scope_module\n\nhidden :: () {}\n\n#scope_file\n\nprivate :: () {}\n",
                ),
                (
                    "impl.jai",
                    "// Docs written at the target.\nforwarded :: () {}\n\nmissing :: () {}\n",
                ),
            ],
        );
        let found = gaps(&dir, Path::new("/nowhere"));
        let module = rows(&found);
        assert!(module.contains(&("module", "", true, false)), "{module:?}");
        assert!(module.contains(&("procedure", "documented", true, true)));
        assert!(module.contains(&("procedure", "bare", false, false)));
        assert!(module.contains(&("struct", "Point", true, false)));
        assert!(module.contains(&("enum", "Mode", false, false)));
        assert!(module.contains(&("alias", "forwarded", true, false)));
        assert!(module.contains(&("alias", "missing", false, false)));
        for hidden in ["hidden", "private", "CONSTANT"] {
            assert!(module.iter().all(|r| r.1 != hidden), "{hidden} is listed");
        }
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_comment_above_the_first_declaration_is_not_the_modules() {
        let dir = library(
            "opening",
            &[("module.jai", "// About the procedure.\nfirst :: () {}\n")],
        );
        let found = gaps(&dir, Path::new("/nowhere"));
        assert_eq!(
            rows(&found),
            [
                ("module", "", false, false),
                ("procedure", "first", true, false)
            ]
        );
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn files_of_foreign_declarations_are_not_listed() {
        let dir = library(
            "foreign",
            &[
                (
                    "module.jai",
                    "// Binding.\n\n#load \"native.jai\";\nwrapper :: () {}\n",
                ),
                (
                    "native.jai",
                    "native_call :: (x: int) -> int #foreign lib;\nNative_Struct :: struct { x: int; }\n",
                ),
            ],
        );
        let found = gaps(&dir, Path::new("/nowhere"));
        assert_eq!(
            rows(&found),
            [
                ("module", "", true, false),
                ("procedure", "wrapper", false, false)
            ]
        );
        let _ = std::fs::remove_dir_all(dir);
    }
}
