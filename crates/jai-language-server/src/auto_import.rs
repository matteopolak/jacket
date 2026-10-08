//! Auto-import completion: names the document cannot see yet, each with the `#import` or
//! `#load` that brings it in as an additional edit (TypeScript-style auto-imports).
//!
//! Three sources, all matched case-insensitively by prefix:
//! - **Standard-library modules** (and `Extensions/<Name>`): their exported top-level names.
//!   Indexed once per stdlib folder, on first use.
//! - **The project's modules**: the `modules/` folder next to the entry file and next to the
//!   project root, `jai.toml`'s `import_path`, and the environment's import paths. A module
//!   there hides a stdlib module of the same name, as for the compiler.
//! - **The project's other files**: declarations of `.jai` files under the project folder that
//!   the program does not load yet get `#load "relative/path.jai";` into the document; files
//!   it already loads offer their names with no edit.
//!
//! Every file is scanned from tokens (`exports.rs`) and cached by path: files on disk until a
//! watched-file or save notification names them, open documents by a hash of their text.
//! Folder listings are cached until a file is created or deleted under them.
use crate::analysis::Span;
use crate::document::DocumentUri;
use crate::exports::{self, Decl, FileScan, Visibility};
use crate::model::{CompletionItem, CompletionKind, TextEdit};
use crate::project::{self, CONFIG_FILE, INFERRED_ENTRIES, ProjectConfig};
use crate::semantic::Environment;
use crate::session::Session;
use jaic::sema::TargetOs;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// The most auto-import items one completion offers.
pub const MAX_ITEMS: usize = 50;

/// The most overload signatures one item's documentation shows.
const MAX_OVERLOADS: usize = 4;

/// Characters to type before auto-import items are offered.
pub const MIN_PREFIX: usize = 2;

/// The most `.jai` files a project folder walk collects, and how deep it goes.
const MAX_PROJECT_FILES: usize = 4000;

const MAX_DEPTH: usize = 8;

/// The most files one load graph (a program's or a module's) follows.
const MAX_LOADED: usize = 4000;

/// How many folders above a document its loader is looked for in.
const LOADER_LEVELS: usize = 3;

/// Folders a project walk skips besides hidden ones (module folders are indexed as modules).
const SKIPPED_FOLDERS: &[&str] = &["modules", "node_modules", "target", "build", "bin"];

/// A module and the names it exports.
pub(crate) struct Module {
    /// As imported: `Basic`, `Extensions/Long_Double`.
    name: String,
    only_os: Option<Vec<String>>,
    decls: Vec<Decl>,
}

#[derive(Default)]
pub(crate) struct Index {
    stdlib: Option<(PathBuf, Rc<Vec<Module>>)>,
    module_dirs: BTreeMap<PathBuf, Rc<Vec<Module>>>,
    /// Scans of files read from the environment.
    files: BTreeMap<PathBuf, Rc<FileScan>>,
    /// Scans of open documents, with the hash of the text scanned.
    open: BTreeMap<PathBuf, (u64, Rc<FileScan>)>,
    /// The `.jai` files on disk under each walked folder.
    walked: BTreeMap<PathBuf, Rc<Vec<PathBuf>>>,
}

impl Index {
    /// `path` changed: its scan and the module index it is part of are stale.
    pub(crate) fn changed(&mut self, path: &Path) {
        self.files.remove(path);
        self.module_dirs.retain(|dir, _| !path.starts_with(dir));
        if self
            .stdlib
            .as_ref()
            .is_some_and(|(dir, _)| path.starts_with(dir))
        {
            self.stdlib = None;
        }
    }

    /// `path` was created or deleted: listings of the folders above it are stale too.
    pub(crate) fn created_or_deleted(&mut self, path: &Path) {
        self.changed(path);
        self.walked.retain(|dir, _| !path.starts_with(dir));
    }
}

/// Where a name comes from.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Source {
    /// A file the program loads already: no edit.
    Loaded(PathBuf),
    /// A project file to `#load`.
    Load(PathBuf),
    /// A module to `#import`.
    Import(String),
}

/// What the document's program is made of.
struct Program {
    /// Every file the entry loads (the document included).
    closure: BTreeSet<PathBuf>,
    /// The folder whose other files may be loaded.
    dir: PathBuf,
    /// The project's entry files: never offered for `#load`.
    entries: BTreeSet<PathBuf>,
    /// Folders of user modules, searched in order.
    module_dirs: Vec<PathBuf>,
    os: TargetOs,
    stdlib: Option<PathBuf>,
}

/// Files as the server sees them: open documents over the environment's file system.
struct Sources<'a> {
    env: &'a Environment,
    open: BTreeMap<PathBuf, &'a str>,
}

fn hash(text: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    text.hash(&mut h);
    h.finish()
}

impl Sources<'_> {
    fn read(&self, path: &Path) -> Option<String> {
        match self.open.get(path) {
            Some(text) => Some((*text).to_string()),
            None => self
                .env
                .fs
                .read(path)
                .map(|b| String::from_utf8_lossy(&b).into_owned()),
        }
    }

    fn is_file(&self, path: &Path) -> bool {
        self.open.contains_key(path) || self.env.fs.is_file(path)
    }

    fn is_dir(&self, path: &Path) -> bool {
        self.open.keys().any(|p| p.starts_with(path) && p != path) || self.env.fs.is_dir(path)
    }

    /// Entries of `dir` (name, is a folder): on disk and open documents below it.
    fn list_dir(&self, dir: &Path) -> Vec<(String, bool)> {
        let mut entries: BTreeSet<(String, bool)> = self.env.fs.list_dir(dir).into_iter().collect();
        for open in self.open.keys() {
            if let Ok(rest) = open.strip_prefix(dir) {
                let mut parts = rest.iter();
                if let Some(first) = parts.next() {
                    entries.insert((first.to_string_lossy().into_owned(), parts.next().is_some()));
                }
            }
        }
        entries.into_iter().collect()
    }

    fn scan(&self, index: &mut Index, path: &Path) -> Rc<FileScan> {
        if let Some(text) = self.open.get(path) {
            let h = hash(text);
            if let Some((seen, scan)) = index.open.get(path)
                && *seen == h
            {
                return scan.clone();
            }
            let scan = Rc::new(exports::scan(text));
            index.open.insert(path.to_path_buf(), (h, scan.clone()));
            return scan;
        }
        if let Some(scan) = index.files.get(path) {
            return scan.clone();
        }
        let scan = Rc::new(
            self.read(path)
                .map(|text| exports::scan(&text))
                .unwrap_or_default(),
        );
        index.files.insert(path.to_path_buf(), scan.clone());
        scan
    }

    /// `root` and every file its `#load`s reach.
    fn closure(&self, index: &mut Index, root: &Path) -> BTreeSet<PathBuf> {
        let mut seen = BTreeSet::new();
        let mut pending = vec![root.to_path_buf()];
        while let Some(file) = pending.pop() {
            if seen.len() >= MAX_LOADED || !seen.insert(file.clone()) {
                continue;
            }
            let dir = file.parent().unwrap_or(Path::new("/")).to_path_buf();
            for load in &self.scan(index, &file).loads {
                pending.push(project::normalize(&dir.join(&load.path)));
            }
        }
        seen
    }

    /// The modules in `dir`: `Name.jai` and `Name/module.jai`, imported as `prefix` + `Name`.
    fn modules_in(&self, index: &mut Index, dir: &Path, prefix: &str) -> Vec<Module> {
        let mut modules = Vec::new();
        for (entry, is_dir) in self.list_dir(dir) {
            if entry.starts_with('.') {
                continue;
            }
            let (name, file) = if is_dir {
                let file = dir.join(&entry).join("module.jai");
                if !self.is_file(&file) {
                    continue;
                }
                (entry, file)
            } else if let Some(stem) = entry.strip_suffix(".jai") {
                (stem.to_string(), dir.join(&entry))
            } else {
                continue;
            };
            let only_os = self.scan(index, &file).only_os.clone();
            let mut decls = Vec::new();
            for part in self.closure(index, &file) {
                decls.extend(
                    self.scan(index, &part)
                        .decls
                        .iter()
                        .filter(|d| d.visibility == Visibility::Export)
                        .cloned(),
                );
            }
            modules.push(Module {
                name: format!("{prefix}{name}"),
                only_os,
                decls,
            });
        }
        modules
    }

    fn stdlib(&self, index: &mut Index, dir: &Path) -> Rc<Vec<Module>> {
        if let Some((cached, modules)) = &index.stdlib
            && cached == dir
        {
            return modules.clone();
        }
        let mut modules: Vec<Module> = self
            .modules_in(index, dir, "")
            .into_iter()
            .filter(|m| !jaic::sema::COMPILER_INTERNAL_MODULES.contains(&m.name.as_str()))
            .collect();
        let extensions = jaic::STDLIB_EXTENSIONS_DIR;
        modules.extend(self.modules_in(index, &dir.join(extensions), &format!("{extensions}/")));
        // The stdlib's files are only read through its modules: drop their scans.
        index.files.retain(|path, _| !path.starts_with(dir));
        let modules = Rc::new(modules);
        index.stdlib = Some((dir.to_path_buf(), modules.clone()));
        modules
    }

    fn user_modules(&self, index: &mut Index, dir: &Path) -> Rc<Vec<Module>> {
        if let Some(modules) = index.module_dirs.get(dir) {
            return modules.clone();
        }
        let modules = Rc::new(self.modules_in(index, dir, ""));
        index.module_dirs.insert(dir.to_path_buf(), modules.clone());
        modules
    }

    /// The `.jai` files under `dir` (cached walk of the disk, plus open documents).
    fn project_files(&self, index: &mut Index, dir: &Path) -> Vec<PathBuf> {
        let walked = match index.walked.get(dir) {
            Some(files) => files.clone(),
            None => {
                let mut files = Vec::new();
                let mut pending = vec![(dir.to_path_buf(), 0)];
                while let Some((folder, depth)) = pending.pop() {
                    for (entry, is_dir) in self.env.fs.list_dir(&folder) {
                        if files.len() >= MAX_PROJECT_FILES {
                            break;
                        }
                        let path = folder.join(&entry);
                        if is_dir {
                            if depth < MAX_DEPTH
                                && !entry.starts_with('.')
                                && !SKIPPED_FOLDERS.contains(&entry.as_str())
                            {
                                pending.push((path, depth + 1));
                            }
                        } else if entry.ends_with(".jai") {
                            files.push(path);
                        }
                    }
                }
                let files = Rc::new(files);
                index.walked.insert(dir.to_path_buf(), files.clone());
                files
            }
        };
        let mut all: BTreeSet<PathBuf> = walked.iter().cloned().collect();
        all.extend(
            self.open
                .keys()
                .filter(|p| {
                    p.starts_with(dir)
                        && p.extension().is_some_and(|e| e == "jai")
                        && !p
                            .strip_prefix(dir)
                            .unwrap_or(p)
                            .parent()
                            .is_some_and(|rest| {
                                rest.iter().any(|c| {
                                    let c = c.to_string_lossy();
                                    c.starts_with('.') || SKIPPED_FOLDERS.contains(&&*c)
                                })
                            })
                })
                .cloned(),
        );
        all.into_iter().collect()
    }

    /// The nearest `jai.toml` at or above `file`: its folder and settings (the defaults when it
    /// does not parse).
    fn config(&self, file: &Path) -> Option<(PathBuf, ProjectConfig)> {
        let mut dir = file.parent();
        while let Some(d) = dir {
            let path = d.join(CONFIG_FILE);
            if self.is_file(&path) {
                let text = self.read(&path).unwrap_or_default();
                return Some((
                    d.to_path_buf(),
                    ProjectConfig::parse(&text).unwrap_or_default(),
                ));
            }
            dir = d.parent();
        }
        None
    }

    fn program(&self, index: &mut Index, file: &Path, folders: &[PathBuf]) -> Program {
        let config = self.config(file);
        let base = config.as_ref().map(|(dir, _)| dir.clone()).or_else(|| {
            folders
                .iter()
                .filter(|f| file.starts_with(f))
                .max_by_key(|f| f.as_os_str().len())
                .cloned()
        });
        let entries: BTreeSet<PathBuf> = match (&base, &config) {
            (Some(base), Some((_, config))) if !config.build_files.is_empty() => config
                .build_files
                .iter()
                .map(|f| project::normalize(&base.join(f)))
                .collect(),
            (Some(base), _) => INFERRED_ENTRIES
                .iter()
                .map(|f| base.join(f))
                .filter(|f| self.is_file(f))
                .collect(),
            (None, _) => BTreeSet::new(),
        };
        let mut found = None;
        for entry in &entries {
            let closure = self.closure(index, entry);
            if closure.contains(file) {
                found = Some((entry.clone(), closure));
                break;
            }
        }
        let (root, closure) = match found {
            Some(found) => found,
            None => {
                // Up the `#load`s among the files of the document's folder and the folders
                // above it (a loader lives next to or above what it loads).
                let mut files = Vec::new();
                let mut dir = file.parent();
                for _ in 0..=LOADER_LEVELS {
                    let Some(d) = dir else {
                        break;
                    };
                    files.extend(
                        self.list_dir(d)
                            .into_iter()
                            .filter(|(name, is_dir)| !is_dir && name.ends_with(".jai"))
                            .map(|(name, _)| d.join(name)),
                    );
                    if base.as_deref() == Some(d) {
                        break;
                    }
                    dir = d.parent();
                }
                let mut root = file.to_path_buf();
                let mut visited = BTreeSet::from([root.clone()]);
                'up: loop {
                    for f in &files {
                        let dir = f.parent().unwrap_or(Path::new("/"));
                        let loads_root = self
                            .scan(index, f)
                            .loads
                            .iter()
                            .any(|l| project::normalize(&dir.join(&l.path)) == root);
                        if loads_root && visited.insert(f.clone()) {
                            root = f.clone();
                            continue 'up;
                        }
                    }
                    break;
                }
                let closure = self.closure(index, &root);
                (root, closure)
            }
        };
        let root_dir = root.parent().unwrap_or(Path::new("/")).to_path_buf();
        let dir = match &base {
            Some(base) if entries.contains(&root) => base.clone(),
            _ => root_dir.clone(),
        };
        let options = (self.env.options)(&root);
        let stdlib = options
            .preload
            .as_deref()
            .and_then(Path::parent)
            .map(Path::to_path_buf);
        let mut module_dirs = vec![root_dir.join("modules")];
        if let Some(base) = &base {
            module_dirs.push(base.join("modules"));
            if let Some((_, config)) = &config {
                module_dirs.extend(
                    config
                        .import_path
                        .iter()
                        .map(|p| project::normalize(&base.join(p))),
                );
            }
        }
        module_dirs.extend(
            options.import_paths.into_iter().filter(|p| {
                stdlib.as_deref().map(project::normalize) != Some(project::normalize(p))
            }),
        );
        let mut seen = BTreeSet::new();
        module_dirs.retain(|d| seen.insert(d.clone()) && self.is_dir(d));
        Program {
            closure,
            dir,
            entries,
            module_dirs,
            os: options.os,
            stdlib,
        }
    }
}

/// Whether a module asserting `only_os` builds for `os`.
fn builds_for(only_os: &Option<Vec<String>>, os: TargetOs) -> bool {
    let name = match os {
        TargetOs::Windows => "WINDOWS",
        TargetOs::Linux => "LINUX",
        TargetOs::MacOS => "MACOS",
        TargetOs::Wasm => "WASM",
    };
    only_os
        .as_ref()
        .is_none_or(|targets| targets.iter().any(|t| t == name))
}

/// Ranks of stdlib modules: common ones (`Basic`, `String`, ...) first, `Extensions/` last.
fn stdlib_rank(module: &str) -> u32 {
    match jaic::sema::COMMON_MODULES.iter().position(|m| *m == module) {
        Some(at) => 10 + at as u32,
        None if module.contains('/') => 2000,
        None => 1000,
    }
}

impl Session {
    /// Auto-import items for `prefix` at the cursor of `uri` (whose text is `text`), leaving out
    /// `visible` names; and whether the list is incomplete (more matches, or too short a prefix
    /// yet, so the client asks again as the word grows).
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
        let sources = Sources {
            env,
            open: self
                .documents
                .iter()
                .chain(&self.lint_configs)
                .map(|(u, d)| (PathBuf::from(u.path()), d.text.as_str()))
                .collect(),
        };
        let file = PathBuf::from(uri.path());
        let here = file.parent().unwrap_or(Path::new("/")).to_path_buf();
        let lower = prefix.to_lowercase();
        let mut index = self.index.borrow_mut();
        let program = sources.program(&mut index, &file, &self.workspace_folders);
        let current = sources.scan(&mut index, &file);
        let mut imported: BTreeSet<String> = BTreeSet::new();
        for f in &program.closure {
            imported.extend(
                sources
                    .scan(&mut index, f)
                    .imports
                    .iter()
                    .map(|i| i.path.clone()),
            );
        }
        // (rank, source, declaration)
        let mut found: Vec<(u32, Source, Decl)> = Vec::new();
        let matches = |d: &Decl| d.lower.starts_with(&lower) && !visible.contains(&d.name);
        // The project's other files.
        let module_dirs = program.module_dirs.clone();
        for f in sources.project_files(&mut index, &program.dir) {
            if f == file || module_dirs.iter().any(|d| f.starts_with(d)) {
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
            let rank = if matches!(source, Source::Loaded(_)) {
                0
            } else {
                1
            };
            for d in scan.decls.iter().filter(|d| matches(d)) {
                found.push((rank, source.clone(), d.clone()));
            }
        }
        // Modules: the project's, then the stdlib's (one the project has hides it).
        let mut seen_modules = BTreeSet::new();
        let mut sets: Vec<(Rc<Vec<Module>>, bool)> = module_dirs
            .iter()
            .map(|d| (sources.user_modules(&mut index, d), true))
            .collect();
        if let Some(stdlib) = &program.stdlib {
            sets.push((sources.stdlib(&mut index, stdlib), false));
        }
        for (modules, user) in &sets {
            for module in modules.iter() {
                if !seen_modules.insert(module.name.clone())
                    || imported.contains(&module.name)
                    || !builds_for(&module.only_os, program.os)
                {
                    continue;
                }
                let rank = if *user {
                    5
                } else {
                    stdlib_rank(&module.name)
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
        let mut merged: Vec<(u32, Source, Decl)> = Vec::with_capacity(found.len());
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
                let (offset, new_text) = crate::imports::insertion(text, after, &statement);
                let range = self.range_of(uri, Span::new(offset, offset))?;
                Some(CompletionItem {
                    label: decl.name.clone(),
                    kind: decl.kind,
                    detail,
                    documentation: Some(format!("Adds `{statement}`\n\n{fence}{doc}")),
                    label_description: Some(description),
                    // After every in-scope name; then by name, closest source first.
                    sort_text: Some(format!("~{}\u{1}{rank:04}", decl.name)),
                    additional_edits: vec![TextEdit {
                        range,
                        new_text,
                    }],
                    ..CompletionItem::default()
                })
            })
            .collect();
        (items, incomplete)
    }
}
