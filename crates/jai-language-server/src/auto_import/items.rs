//! Project symbols and the completion items an auto import offers.
use super::names::{builds_for, module_of};
use super::{MAX_ITEMS, MAX_OVERLOADS, MIN_PREFIX, Modules, Source, hash};
use crate::analysis::Span;
use crate::document::DocumentUri;
use crate::exports::{self, Decl};
use crate::model::{CompletionItem, CompletionKind, TextEdit};
use crate::project::{self};
use crate::session::Session;
use jaic::fxhash::HashMap;
use jaic::sema::{ModuleRanking, Rank};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

impl Session {
    /// Reads the module indexes auto-import completion offers from (the stdlib's and the
    /// project's), which cost a few hundred milliseconds the first time and nothing after, so
    /// the first completion of a name does not pay for them. A transport calls this when it
    /// has nothing else to do; it does nothing without auto-import or an open document.
    pub fn warm_auto_import(&self) {
        let Some(env) = self.environment.as_ref().filter(|_| self.auto_import) else {
            return;
        };
        let Some(uri) = self.documents.keys().next() else {
            return;
        };
        let sources = self.sources(env);
        let file = sources.key(Path::new(uri.path()));
        let mut index = self.index.borrow_mut();
        let program = sources.program(&mut index, &file, &self.workspace_folders);
        for dir in &program.module_dirs {
            sources.user_modules(&mut index, dir);
        }
        if let Some(stdlib) = &program.stdlib {
            sources.stdlib(&mut index, stdlib);
        }
    }

    /// Auto-import items for `prefix` at the cursor of `uri` (whose text is `text`), leaving out
    /// `visible` names; and whether the list is incomplete (more matches, or too short a prefix
    /// yet, so the client asks again as the word grows).
    /// The `jai.toml` build files whose run is still unknown and could attribute `uri` to a
    /// program, with a hash of what each would be run from (see `Index::probed`).
    pub(crate) fn unrecorded_builds(&self, uri: &DocumentUri) -> Vec<(PathBuf, u64)> {
        let Some(env) = self.environment.as_ref() else {
            return Vec::new();
        };
        let sources = self.sources(env);
        let mut index = self.index.borrow_mut();
        let program = sources.program(&mut index, Path::new(uri.path()), &self.workspace_folders);
        let mut out = Vec::new();
        for entry in program.unattributed {
            if index.builds.contains_key(&entry) {
                continue;
            }
            // Everything the run reads: the metaprogram, what it loads, the settings file.
            let mut files: Vec<PathBuf> = sources.closure(&mut index, &entry).into_iter().collect();
            if let Some((folder, _)) = sources.config(&mut index, &entry) {
                files.extend(project::CONFIG_NAMES.iter().map(|n| folder.join(n)));
            }
            let mut texts = String::new();
            for file in files {
                texts.push_str(&file.to_string_lossy());
                texts.push('\0');
                texts.push_str(&sources.read(&file).unwrap_or_default());
                texts.push('\0');
            }
            let hash = hash(&texts);
            if index.probed.get(&entry) != Some(&hash) {
                out.push((entry, hash));
            }
        }
        out
    }

    /// The entry file of the project program `uri` belongs to (`jai.toml`'s `build_files`, an
    /// inferred entry, or the file that loads it), read from disk and the open documents.
    pub(crate) fn project_root(&self, uri: &DocumentUri) -> Option<PathBuf> {
        let env = self.environment.as_ref()?;
        let sources = self.sources(env);
        let mut index = self.index.borrow_mut();
        let program = sources.program(&mut index, Path::new(uri.path()), &self.workspace_folders);
        Some(program.root)
    }

    /// `root` compiles other files by running a build metaprogram (`add_build_file("x", w)`),
    /// which an untrusted workspace does not: those files are compiled as their own programs.
    pub(crate) fn is_build_metaprogram(&self, root: &Path) -> bool {
        let Some(env) = self.environment.as_ref() else {
            return false;
        };
        let sources = self.sources(env);
        let mut index = self.index.borrow_mut();
        !sources.scan(&mut index, root).builds.is_empty()
    }

    /// Top-level declarations of the project's `.jai` files that are not open, whose name
    /// contains `query` (lower case), at most `room` of them. The files come from the
    /// workspace folders and the folders of the open documents' programs, are scanned from
    /// tokens (`exports.rs`, cached with the auto-import index), and nothing is compiled.
    pub(crate) fn project_symbols(
        &self,
        query: &str,
        room: usize,
    ) -> Vec<crate::model::SymbolInformation> {
        let Some(env) = self.environment.as_ref().filter(|_| room > 0) else {
            return Vec::new();
        };
        let sources = self.sources(env);
        let mut index = self.index.borrow_mut();
        let mut folders: BTreeSet<PathBuf> = self.workspace_folders.iter().cloned().collect();
        for uri in self.documents.keys() {
            folders.insert(
                sources
                    .program(&mut index, Path::new(uri.path()), &self.workspace_folders)
                    .dir,
            );
        }
        let mut files: BTreeSet<PathBuf> = BTreeSet::new();
        for folder in &folders {
            files.extend(sources.project_files(&mut index, folder));
        }
        let mut out = Vec::new();
        for file in files {
            // Listed files are keys, as are the open documents'.
            if sources.open.contains_key(&file) {
                continue;
            }
            let scan = sources.scan_keyed(&mut index, &file);
            let hits: Vec<_> = scan
                .symbols
                .iter()
                .filter(|s| s.name.to_lowercase().contains(query))
                .collect();
            if hits.is_empty() {
                continue;
            }
            let Some(bytes) = env.fs.read(&file) else {
                continue;
            };
            let text = String::from_utf8_lossy(&bytes);
            let lines = crate::position::LineIndex::new(&text);
            let Ok(uri) = DocumentUri::from_path(&file) else {
                continue;
            };
            for symbol in hits {
                if out.len() >= room {
                    return out;
                }
                let Ok(range) = lines.range(
                    &text,
                    Span::new(symbol.offset, symbol.offset + symbol.name.len()),
                ) else {
                    continue;
                };
                out.push(crate::model::SymbolInformation {
                    name: symbol.name.clone(),
                    kind: match symbol.kind {
                        CompletionKind::Function => crate::model::SymbolKind::Function,
                        CompletionKind::Struct => crate::model::SymbolKind::Struct,
                        CompletionKind::Enum => crate::model::SymbolKind::Enum,
                        CompletionKind::TypeAlias => crate::model::SymbolKind::TypeAlias,
                        CompletionKind::Constant => crate::model::SymbolKind::Constant,
                        _ => crate::model::SymbolKind::Variable,
                    },
                    location: crate::model::Location {
                        uri: uri.as_str().into(),
                        range,
                    },
                    container: None,
                });
            }
        }
        out
    }

    pub(crate) fn auto_import_items(
        &self,
        uri: &DocumentUri,
        text: &str,
        prefix: &str,
        visible: &BTreeSet<String>,
    ) -> (Vec<CompletionItem>, bool) {
        let Some(env) = self.environment.as_ref().filter(|_| self.auto_import) else {
            return (Vec::new(), false);
        };
        if prefix.chars().count() < MIN_PREFIX {
            return (Vec::new(), true);
        }
        let sources = self.sources(env);
        let file = sources.key(Path::new(uri.path()));
        let here = file.parent().unwrap_or(Path::new("/")).to_path_buf();
        let lower = prefix.to_lowercase();
        let mut index = self.index.borrow_mut();
        let program = sources.program(&mut index, &file, &self.workspace_folders);
        let current = sources.scan(&mut index, &file);
        let mut import_counts: HashMap<String, u32> = HashMap::default();
        for f in &program.closure {
            for import in &sources.scan(&mut index, f).imports {
                *import_counts.entry(import.path.clone()).or_insert(0) += 1;
            }
        }
        let imported: BTreeSet<&String> = import_counts.keys().collect();
        // (rank, source, declaration)
        let mut found: Vec<(Rank, Source, Decl)> = Vec::new();
        let matches = |d: &Decl| d.lower.starts_with(&lower) && !visible.contains(&d.name);
        // The project's other files. A stdlib the workspace holds is offered through its modules
        // below, not file by file; only a program inside one of its folders (a module's own
        // files) sees the files there.
        let module_dirs = program.module_dirs.clone();
        let stdlib = program
            .stdlib
            .as_deref()
            .map(|s| sources.key(s))
            .filter(|s| !program.dir.starts_with(s) || program.dir == *s);
        for f in sources.project_files(&mut index, &program.dir) {
            if f == file
                || module_dirs.iter().any(|d| f.starts_with(d))
                || stdlib.as_ref().is_some_and(|s| f.starts_with(s))
            {
                continue;
            }
            let scan = sources.scan(&mut index, &f);
            let source = if program.closure.contains(&f) {
                Source::Loaded(f.clone())
            } else if program.entries.contains(&f) || scan.declares_main {
                continue;
            } else {
                Source::Load(f.clone())
            };
            let rank = Rank::from(!matches!(source, Source::Loaded(_)));
            let declaration_root = if program.closure.contains(&f) {
                &program.root
            } else {
                &f
            };
            for d in scan.decls.iter().filter(|d| matches(d)) {
                for declaration in sources.alias_documentation(
                    &mut index,
                    declaration_root,
                    &f,
                    d,
                    &program.dir.join("modules"),
                ) {
                    found.push((rank, source.clone(), declaration));
                }
            }
        }
        // Modules: the project's, then the stdlib's (one the project has hides it), never the
        // module the document is part of.
        let mut seen_modules: BTreeSet<String> = module_dirs
            .iter()
            .cloned()
            .chain(program.stdlib.as_deref().map(|s| sources.key(s)))
            .filter_map(|d| module_of(&file, &d))
            .collect();
        let mut sets: Vec<(Modules, bool)> = module_dirs
            .iter()
            .map(|d| (sources.user_modules(&mut index, d), true))
            .collect();
        let mut ranking = ModuleRanking::default();
        if let Some(stdlib) = &program.stdlib {
            let modules = sources.stdlib(&mut index, stdlib);
            ranking = sources
                .stdlib_ranking(&mut index, &modules)
                .with_project(import_counts.clone());
            sets.push((modules, false));
        }
        for (modules, user) in &sets {
            for module in modules.iter() {
                if !seen_modules.insert(module.name.clone())
                    || imported.contains(&module.name)
                    || !builds_for(module.only_os.as_deref(), program.os)
                {
                    continue;
                }
                let rank = if *user {
                    5
                } else {
                    ranking.rank(&module.name)
                };
                for d in module.decls.iter().filter(|d| matches(d)) {
                    found.push((rank, Source::Import(module.name.clone()), d.clone()));
                }
            }
        }
        drop(index);
        found.sort_by(|a, b| {
            (&a.2.lower, &a.2.name, a.0, &a.1).cmp(&(&b.2.lower, &b.2.name, b.0, &b.1))
        });
        // Overloads (and `#if` variants) of one name in one place are one item showing every
        // signature; a procedure's kind wins over `print :: print_to_builder;`.
        let mut merged: Vec<(Rank, Source, Decl)> = Vec::with_capacity(found.len());
        for (rank, source, decl) in found {
            match merged.last_mut() {
                Some((_, s, d)) if *s == source && d.name == decl.name => {
                    if !d.signature.lines().any(|l| l == decl.signature)
                        && d.signature.lines().count() < MAX_OVERLOADS
                    {
                        d.signature = format!("{}\n{}", d.signature, decl.signature);
                    }
                    if d.doc.is_empty() {
                        d.doc = decl.doc;
                    }
                    if decl.kind == CompletionKind::Function {
                        d.kind = decl.kind;
                    }
                    d.call = match (d.call.take(), decl.call) {
                        (Some(a), Some(b)) => Some(a.overloaded(&b)),
                        (a, b) => a.or(b),
                    };
                }
                _ => merged.push((rank, source, decl)),
            }
        }
        let mut found = merged;
        let incomplete = found.len() > MAX_ITEMS;
        found.truncate(MAX_ITEMS);
        // Where a new `#import` and `#load` go: after the last of their kind at the top level.
        let last =
            |list: &[exports::Directive]| list.iter().filter(|d| !d.nested).map(|d| d.end).max();
        let after_imports = last(&current.imports);
        let after_loads = last(&current.loads).or(after_imports);
        let items = found
            .into_iter()
            .filter_map(|(rank, source, decl)| {
                let fence = format!("```jai\n{}\n```", decl.signature);
                let doc = if decl.doc.is_empty() {
                    String::new()
                } else {
                    format!("\n\n{}", decl.doc)
                };
                let (statement, after, description, detail) = match &source {
                    Source::Loaded(path) => {
                        let shown = project::relative(&here, path);
                        return Some(CompletionItem {
                            label: decl.name.clone(),
                            kind: decl.kind,
                            detail: decl.signature.lines().next().unwrap_or_default().into(),
                            documentation: Some(format!(
                                "From `{shown}`, which the program already loads.\n\n{fence}{doc}"
                            )),
                            label_description: Some(shown),
                            call: decl.call.clone(),
                            ..CompletionItem::default()
                        });
                    }
                    Source::Load(path) => {
                        let shown = project::relative(&here, path);
                        (
                            format!("#load \"{shown}\";"),
                            after_loads,
                            shown.clone(),
                            format!("auto-import: #load \"{shown}\""),
                        )
                    }
                    Source::Import(module) => (
                        format!("#import \"{module}\";"),
                        after_imports,
                        module.clone(),
                        format!("auto-import from {module}"),
                    ),
                };
                let (offset, new_text) =
                    crate::imports::insertion(text, after, current.first_code, &statement);
                let range = self.range_of(uri, Span::new(offset, offset))?;
                Some(CompletionItem {
                    label: decl.name.clone(),
                    kind: decl.kind,
                    detail,
                    documentation: Some(format!("Adds `{statement}`\n\n{fence}{doc}")),
                    label_description: Some(description),
                    // After every in-scope name; then by name, closest source first.
                    sort_text: Some(format!("~{}\u{1}{rank:014}", decl.name)),
                    additional_edits: vec![TextEdit {
                        range,
                        new_text,
                    }],
                    call: decl.call.clone(),
                    ..CompletionItem::default()
                })
            })
            .collect();
        (items, incomplete)
    }
}
