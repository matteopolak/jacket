//! Diagnostics for modules and files that cannot be found or read.
use super::filesystem::{module_file_in, modules_in};
use crate::diagnostics::Code;
use crate::sema::SourceState;
use crate::sema::{Compiler, Diagnostic, PathBuf, Span};
use std::path::Path;

impl Compiler {
    /// For a missing module that is one of jaic's `stdlib/Extensions/` modules imported without
    /// the `Extensions/` path (or the folder itself, or the old `Jaic_Extensions`): `d` with that
    /// help, which replaces the general search help. `Err(d)` unchanged otherwise.
    fn with_extension_help(
        &self,
        d: Diagnostic,
        name: &str,
        span: Span,
    ) -> std::result::Result<Diagnostic, Diagnostic> {
        let folder = crate::STDLIB_EXTENSIONS_DIR;
        let Some(stdlib) = self.options.stdlib_root() else {
            return Err(d);
        };
        if name == folder {
            let mut modules: Vec<String> = self
                .src
                .stdlib_modules_under(stdlib, folder)
                .into_iter()
                .map(|(m, _)| format!("`{m}`"))
                .collect();
            modules.sort();
            if modules.is_empty() {
                return Err(d);
            }
            return Ok(d.with_help(format!(
                "`{folder}` is the folder of jaic's extension modules, not a module: import one of them by its path: {}",
                modules.join(", ")
            )));
        }
        if name.contains('/') {
            return Err(d);
        }
        // `Jaic_Extensions` held only Long_Double, which is now a module of its own.
        let wanted = if name == "Jaic_Extensions" {
            "Long_Double"
        } else {
            name
        };
        let path = format!("{folder}/{wanted}");
        let exists = module_file_in(&*self.src.fs, &stdlib.join(folder), wanted).is_some();
        if !exists {
            return Err(d);
        }
        let message = if wanted == name {
            format!(
                "`{name}` is a jaic extension (not official Jai), imported by its path: `#import \"{path}\";`"
            )
        } else {
            format!("`{name}` was replaced by `{path}`: `#import \"{path}\";`")
        };
        Ok(
            match self.src.sources.snippet(span).find(&format!("\"{name}\"")) {
                Some(at) => {
                    let start = span.start as usize + at + 1;
                    d.with_replacement(
                        message,
                        Span::new(span.file, start, start + name.len()),
                        path,
                    )
                }
                None => d.with_help(message),
            },
        )
    }

    /// `module `X` not found`, with where jaic looked and the closest module name there.
    pub(super) fn module_not_found(&self, name: &str, from_dir: &Path, span: Span) -> Diagnostic {
        let mut dirs = vec![from_dir.join("modules")];
        dirs.extend(self.options.import_paths.iter().cloned());
        dirs.dedup();
        let mut available: Vec<String> = dirs
            .iter()
            .flat_map(|d| modules_in(&*self.src.fs, d))
            .collect();
        available.sort();
        available.dedup();
        let d = Diagnostic::error(span, format!("module `{name}` not found"))
            .with_label("no module of this name");
        let mut d = match self.with_extension_help(d, name, span) {
            Ok(d) => return d,
            Err(d) => d,
        };
        // A misspelled `Extensions/Name`: the closest extension module.
        if name.starts_with(&format!("{}/", crate::STDLIB_EXTENSIONS_DIR))
            && let Some(stdlib) = self.options.stdlib_root()
        {
            available.extend(
                self.src
                    .stdlib_modules_under(stdlib, crate::STDLIB_EXTENSIONS_DIR)
                    .into_iter()
                    .map(|(m, _)| m),
            );
        }
        let near =
            crate::suggest::closest(name, available.iter().map(String::as_str)).map(str::to_string);
        let near_found = near.is_some();
        if let Some(near) = near {
            let quoted = self.src.sources.snippet(span).find(&format!("\"{name}\""));
            d = match quoted {
                Some(at) => {
                    let start = span.start as usize + at + 1;
                    let at = Span::new(span.file, start, start + name.len());
                    d.with_replacement(
                        format!("a module with a similar name exists: `{near}`"),
                        at,
                        near,
                    )
                }
                None => d.with_help(format!("a module with a similar name exists: `{near}`")),
            };
        }
        let searched: Vec<String> = dirs
            .iter()
            .map(|d| format!("    {}", self.src.canon(d).display()))
            .collect();
        d = d.with_note(
            Span::NONE,
            format!(
                "looked for `{name}.jai` and `{name}/module.jai` in:\n{}",
                searched.join("\n")
            ),
        );
        if near_found {
            return d;
        }
        // The module may be nearby, in a folder jaic does not search.
        if let Some(found) = self.module_nearby(name, from_dir) {
            let shown = crate::display_path(&found);
            let shown = if shown.is_empty() {
                ".".to_string()
            } else {
                shown
            };
            return d.with_help(format!(
                "a module `{name}` exists in `{shown}`, which is not searched: pass `-import_dir {shown}`, or move it into a `modules` folder next to the main file"
            ));
        }
        d.with_help(
            "a module of your own goes in a `modules` folder next to the main file, or in a directory passed with `-import_dir`",
        )
    }

    /// A directory near `from_dir` (it, its parents up to three levels, and their immediate
    /// subdirectories) that holds the module `name`.
    fn module_nearby(&self, name: &str, from_dir: &Path) -> Option<PathBuf> {
        let holds = |dir: &Path| module_file_in(&*self.src.fs, dir, name).is_some();
        let start = self.src.canon(from_dir);
        for ancestor in start.ancestors().take(4) {
            if holds(ancestor) {
                return Some(ancestor.to_path_buf());
            }
            let mut subdirs: Vec<String> = self
                .src
                .fs
                .list_dir(ancestor)
                .into_iter()
                .filter(|(n, is_dir)| *is_dir && !n.starts_with('.'))
                .map(|(n, _)| n)
                .collect();
            subdirs.sort();
            for sub in subdirs {
                let dir = ancestor.join(sub);
                if holds(&dir) {
                    return Some(dir);
                }
            }
        }
        None
    }
}

impl SourceState {
    /// `#load` (or another read) of a file that is not there, with a close name if one is.
    pub(super) fn unreadable_file(&self, path: &Path, span: Span) -> Diagnostic {
        let snippet = self.sources.snippet(span);
        // An `#import,file` or `#import,dir` span covers the directive, not only its path.
        let import = snippet.starts_with("#import");
        let shown = match snippet.split_once('"') {
            Some((_, rest)) if import => rest.split('"').next().unwrap_or(rest).to_string(),
            _ => snippet.trim_matches('"').to_string(),
        };
        let shown = if shown.is_empty() || shown.contains('\n') {
            path.display().to_string()
        } else {
            shown
        };
        let relative = if import {
            "`#import,file` and `#import,dir` paths are relative to the file that imports them"
        } else {
            "`#load` paths are relative to the file that loads them"
        };
        if import && snippet.contains(",dir") {
            let folder = path.parent().unwrap_or(Path::new("."));
            let message = if self.fs.is_dir(folder) {
                format!("`{shown}` has no module.jai")
            } else {
                format!("folder `{shown}` does not exist")
            };
            return Diagnostic::error(span, message)
                .with_code(Code::MissingFile)
                .with_note(Span::NONE, format!("looked for {}", path.display()))
                .with_help(relative);
        }
        if self.fs.is_dir(path) {
            return Diagnostic::error(span, format!("`{shown}` is a directory, not a file"))
                .with_help(
                    "`#load` takes a .jai file; `#import,dir` imports a directory's module.jai",
                );
        }
        let mut d = Diagnostic::error(span, format!("file `{shown}` does not exist"))
            .with_code(Code::MissingFile);
        let dir = path.parent().unwrap_or(Path::new("."));
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let files: Vec<String> = self
            .fs
            .list_dir(dir)
            .into_iter()
            .filter(|(n, is_dir)| !is_dir && n.ends_with(".jai"))
            .map(|(n, _)| n)
            .collect();
        if let Some(near) = crate::suggest::similar_entry(&name, files.iter().map(String::as_str)) {
            d = d.with_help(format!("a file with a similar name exists: `{near}`"));
        }
        d.with_note(Span::NONE, format!("looked for {}", path.display()))
            .with_help(relative)
    }
}
