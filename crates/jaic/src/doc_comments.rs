//! The doc comments of a library, read from its parsed files: the docs above each declaration
//! (also those inside top-level `#if` branches) and the comment that opens a module. `jaic
//! -doc_comments DIR` prints them for `jacket test` to run the examples in them.
//!
//! The library is the `.jai` files of the folder, without its `tests/` folder, hidden folders or
//! folders that are modules of their own (they hold a `module.jai`), then every file those
//! `#load`, wherever it lives, and the `internal/...` modules of the standard library that the
//! folder's own files import.
use crate::docs;
use crate::parser::ide_facts::{self, TopLevelKind};
use crate::source::FileId;
use std::collections::{BTreeSet, VecDeque};
use std::path::{Path, PathBuf};

/// One doc comment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LibraryDoc {
    pub file: PathBuf,
    /// The line of the first line of `text`.
    pub line: usize,
    /// The constant it documents, or `None` for a module's opening comment.
    pub symbol: Option<String>,
    pub text: String,
}

/// The `.jai` files of the library in `dir`.
fn own_files(dir: &Path, top: bool, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut paths: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
    paths.sort();
    for path in paths {
        if crate::paths::is_hidden(&path) {
            continue;
        }
        if path.is_dir() {
            let name = path.file_name().is_some_and(|n| n == "tests");
            if (top && name) || crate::paths::module_entry(&path).is_file() {
                continue;
            }
            own_files(&path, false, out);
        } else if path.extension().is_some_and(|e| e == "jai") {
            out.push(path);
        }
    }
}

fn canonical(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

/// Every doc comment of the library in `dir`, in the order its files are reached. `stdlib` is
/// where the `internal/` modules are.
pub fn library_docs(dir: &Path, stdlib: &Path) -> Vec<LibraryDoc> {
    let mut own = Vec::new();
    own_files(dir, true, &mut own);
    let module_file = canonical(&crate::paths::module_entry(dir));
    let mut queue: VecDeque<(PathBuf, bool)> = own.into_iter().map(|f| (f, true)).collect();
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    while let Some((file, is_own)) = queue.pop_front() {
        let file = canonical(&file);
        if !seen.insert(file.clone()) {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&file) else {
            continue;
        };
        let parsed = crate::parser::parse_file_ide(FileId(0), &text);
        let base = file.parent().unwrap_or(Path::new("."));
        let mut documented_from = None;
        for item in ide_facts::top_level(&parsed.file) {
            match item.kind {
                TopLevelKind::Decl(decl) if decl.kind == crate::ast::DeclKind::Const => {
                    let start = item.span.start as usize;
                    if let (Some(name), Some(found)) =
                        (decl.names.first(), docs::located_above(&text, start))
                    {
                        documented_from = Some(found.line);
                        out.push(LibraryDoc {
                            file: file.clone(),
                            line: found.line,
                            symbol: Some(name.name.as_str().to_owned()),
                            text: found.text,
                        });
                    }
                }
                TopLevelKind::Load(path, _) => {
                    let loaded = base.join(path);
                    if loaded.is_file() {
                        queue.push_back((loaded, false));
                    }
                }
                TopLevelKind::Import(import) if is_own => {
                    if let crate::ast::ImportSource::Module(name) = &import.source
                        && name.starts_with("internal/")
                    {
                        let base = stdlib.join(&**name);
                        let found = [
                            base.with_extension("jai"),
                            crate::paths::module_entry(&base),
                        ]
                        .into_iter()
                        .find(|p| p.is_file());
                        queue.extend(found.map(|f| (f, false)));
                    }
                }
                _ => {}
            }
        }
        // Only the module's own file documents the module, and a comment that sits directly
        // above its first declaration documents that declaration instead.
        if file == module_file
            && let Some(opening) = docs::opening(&text)
            && documented_from != Some(opening.line)
        {
            out.push(LibraryDoc {
                file: file.clone(),
                line: opening.line,
                symbol: None,
                text: opening.text,
            });
        }
    }
    out
}

/// `field` with `\`, tab, newline and carriage return written as escapes, so a record is one
/// line of tab-separated fields.
fn escape(field: &str) -> String {
    let mut out = String::with_capacity(field.len());
    for c in field.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\t' => out.push_str("\\t"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            c => out.push(c),
        }
    }
    out
}

/// The records `jaic -doc_comments` prints: `file<TAB>line<TAB>symbol<TAB>text`, one per doc
/// comment, with the symbol empty for a module's own docs and the text escaped (`\n` for a
/// newline, `\t`, `\r` and `\\`).
pub fn records(docs: &[LibraryDoc]) -> String {
    let mut out = String::new();
    for doc in docs {
        out.push_str(&escape(&doc.file.to_string_lossy()));
        out.push('\t');
        out.push_str(&doc.line.to_string());
        out.push('\t');
        out.push_str(&escape(doc.symbol.as_deref().unwrap_or("")));
        out.push('\t');
        out.push_str(&escape(&doc.text));
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn library(files: &[(&str, &str)]) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "jaic-doc-comments-{}-{}",
            std::process::id(),
            files.len()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        for (name, text) in files {
            let path = dir.join(name);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        }
        dir
    }

    #[test]
    fn declarations_in_if_branches_and_with_long_headers_have_docs() {
        let dir = library(&[
            (
                "module.jai",
                "// The module.\n\n#load \"more.jai\";\n\n#if OS == .LINUX {\n    // Linux only.\n    only_here :: () {}\n}\n\n// Spread over lines.\nsplit :: (\n    s: string,\n    on: string,\n) -> int { return 0; }\n",
            ),
            ("more.jai", "// Loaded.\nextra :: 1;\n"),
            ("tests/t.jai", "// Not read.\nt :: 1;\n"),
        ]);
        let found = library_docs(&dir, Path::new("/nowhere"));
        let described: Vec<(Option<&str>, &str)> = found
            .iter()
            .map(|d| (d.symbol.as_deref(), d.text.as_str()))
            .collect();
        assert_eq!(
            described,
            [
                (Some("only_here"), "Linux only."),
                (Some("split"), "Spread over lines."),
                (None, "The module."),
                (Some("extra"), "Loaded."),
            ]
        );
        assert_eq!(found[1].line, 10);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn records_escape_the_text() {
        let doc = LibraryDoc {
            file: PathBuf::from("/a.jai"),
            line: 3,
            symbol: Some("f".into()),
            text: "one\ttwo\nthree\\".into(),
        };
        assert_eq!(records(&[doc]), "/a.jai\t3\tf\tone\\ttwo\\nthree\\\\\n");
    }
}
