//! Where a program's modules are, worked out without a `jai.toml`.
//!
//! The compiler looks for `#import "Name"` in the importing file's `modules/` folder, then in
//! its import paths: its own module folder (here the standard library) and whatever the build
//! options add (`Build_Options.import_path`, a metaprogram's `array_add(*import_path, ...)`).
//! An editor does not run the metaprogram, so the folders it would add are found three ways,
//! best first, and every one that is a folder is searched before the standard library:
//!
//! 1. `modules/` next to the root file, then next to each folder above it up to the project
//!    base (the folder of the nearest `jai.toml`, else the workspace folder holding the file).
//! 2. `import_path` of the nearest `jai.toml`, relative to it.
//! 3. The string literals a project entry file (`build_files`, an inferred entry, or the root
//!    itself) writes in a statement that mentions `import_path`: `array_add(*import_path,
//!    "modules");`, `options.import_path = .["a", "b"];`. Relative paths are taken from the
//!    entry's folder and from the project base. Computed paths (a `%` format, a variable) are
//!    not followed.
use crate::auto_import::{Index, Sources};
use crate::project::{self, CONFIG_FILE, INFERRED_ENTRIES};
use jaic::lexer::{P, Tok};
use jaic::source::FileId;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// Entry files of a metaprogram scanned at most.
const MAX_META_FILES: usize = 8;

/// The string literals written next to an `import_path` in `text`.
pub(crate) fn declared_import_paths(text: &str) -> Vec<String> {
    let Ok(tokens) = jaic::lexer::lex(FileId(0), text) else {
        return Vec::new();
    };
    let mut found = Vec::new();
    for (i, token) in tokens.iter().enumerate() {
        if !matches!(&token.tok, Tok::Ident(name) if name.as_str() == "import_path") {
            continue;
        }
        // The rest of the statement: up to the `;` that is not inside brackets.
        let mut depth = 0i32;
        for next in &tokens[i + 1..] {
            match &next.tok {
                Tok::Punct(P::LParen | P::LBracket | P::LBrace | P::DotBrace | P::DotBracket) => {
                    depth += 1
                }
                Tok::Punct(P::RParen | P::RBracket | P::RBrace) => {
                    depth -= 1;
                    if depth < 0 {
                        break;
                    }
                }
                Tok::Punct(P::Semi) if depth <= 0 => break,
                Tok::Str(s) => {
                    if let Ok(s) = std::str::from_utf8(s)
                        && !s.is_empty()
                        && !s.contains(['%', '\n'])
                    {
                        found.push(s.to_string());
                    }
                }
                _ => {}
            }
        }
    }
    found
}

impl Sources<'_> {
    /// The module folders to search before the standard library for the program rooted at
    /// `root` (see the module documentation), and the files they were read from.
    pub(crate) fn import_dirs(
        &self,
        index: &mut Index,
        root: &Path,
        folders: &[PathBuf],
    ) -> (Vec<PathBuf>, Vec<PathBuf>) {
        let config = self.config(index, root);
        let base = config.as_ref().map(|(dir, _)| dir.clone()).or_else(|| {
            folders
                .iter()
                .filter(|f| root.starts_with(f))
                .max_by_key(|f| f.as_os_str().len())
                .cloned()
        });
        let mut dirs: Vec<PathBuf> = Vec::new();
        let mut deps: Vec<PathBuf> = Vec::new();
        // 1. `modules/` here and in the folders above, up to the base.
        let mut dir = root.parent();
        while let Some(d) = dir {
            dirs.push(d.join("modules"));
            if base.as_deref().is_none_or(|b| b == d) {
                break;
            }
            dir = d.parent();
        }
        // 2. `jai.toml`'s `import_path`.
        if let Some((folder, config)) = &config {
            deps.push(folder.join(CONFIG_FILE));
            dirs.extend(
                config
                    .import_path
                    .iter()
                    .map(|p| project::normalize(&folder.join(p))),
            );
        }
        // 3. What the build metaprogram adds.
        let mut entries: Vec<PathBuf> = Vec::new();
        if let Some(base) = &base {
            match &config {
                Some((_, config)) if !config.build_files.is_empty() => entries.extend(
                    config
                        .build_files
                        .iter()
                        .map(|f| project::normalize(&base.join(f))),
                ),
                _ => entries.extend(INFERRED_ENTRIES.iter().map(|f| base.join(f))),
            }
        }
        entries.push(root.to_path_buf());
        let mut seen = BTreeSet::new();
        for entry in entries
            .into_iter()
            .filter(|e| seen.insert(e.clone()))
            .take(MAX_META_FILES)
        {
            let Some(text) = self.read(&entry) else {
                continue;
            };
            deps.push(entry.clone());
            let here = entry.parent().unwrap_or(Path::new("/")).to_path_buf();
            for path in declared_import_paths(&text) {
                let path = Path::new(&path);
                if path.is_absolute() {
                    dirs.push(project::normalize(path));
                    continue;
                }
                dirs.push(project::normalize(&here.join(path)));
                if let Some(base) = &base {
                    dirs.push(project::normalize(&base.join(path)));
                }
            }
        }
        let mut seen = BTreeSet::new();
        dirs.retain(|d| seen.insert(d.clone()) && self.is_dir(d));
        (dirs, deps)
    }
}

/// `dirs` placed before the last import path (the standard library, by the environment's
/// convention) of `paths`, without repeating one already there.
pub(crate) fn splice(paths: &mut Vec<PathBuf>, dirs: &[PathBuf]) {
    let at = paths.len().saturating_sub(1);
    let fresh: Vec<PathBuf> = dirs
        .iter()
        .filter(|d| !paths.iter().any(|p| p == *d))
        .cloned()
        .collect();
    paths.splice(at..at, fresh);
}

impl crate::Session {
    /// The module folders found for the program rooted at `root`, cached until a file they were
    /// read from changes or a file is created or deleted.
    pub(crate) fn import_dirs(&self, root: &Path) -> Vec<PathBuf> {
        let Some(env) = self.environment.as_ref() else {
            return Vec::new();
        };
        let mut index = self.index.borrow_mut();
        if let Some((dirs, _)) = index.import_dirs.get(root) {
            return dirs.clone();
        }
        let sources = Sources {
            env,
            open: self
                .documents
                .iter()
                .chain(&self.lint_configs)
                .map(|(u, d)| (PathBuf::from(u.path()), d.text.as_str()))
                .collect(),
        };
        let (dirs, deps) = sources.import_dirs(&mut index, root, &self.workspace_folders);
        index
            .import_dirs
            .insert(root.to_path_buf(), (dirs.clone(), deps));
        dirs
    }

    /// The compiler options for the program rooted at `root`: the environment's, with the
    /// module folders `import_dirs` finds in front of the standard library.
    pub(crate) fn options_for(&self, root: &Path) -> Option<jaic::sema::Options> {
        let env = self.environment.as_ref()?;
        let mut options = (env.options)(root);
        splice(&mut options.import_paths, &self.import_dirs(root));
        Some(options)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_literal_import_paths_from_a_metaprogram() {
        let text = r#"
            options.import_path = .["vendor", "libs/%"];
            import_path: [..] string;
            array_add(*import_path, "modules");
            array_add(*import_path, ..options.import_path);
            log("import_path is %", import_path);
            other(".");
        "#;
        assert_eq!(declared_import_paths(text), ["vendor", "modules"]);
    }
}
