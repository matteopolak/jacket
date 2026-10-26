//! What the standard library's modules declare, worked out once and shared by every compile in
//! the process: the "import it" help of an unknown name or operator reads this instead of
//! opening and parsing the modules (see `docs/compiler/stdlib-index.md`).
//!
//! The index is keyed by the stdlib folder and valid while the `FileSystem::stamp` of every
//! path it was built from is unchanged, so editing a stdlib file (or adding one) builds a new
//! one. A file system without stamps gets an index of its own for each compile.
use super::FileSystem;
use super::modules::module_listed;
use crate::ast::{ScopeKind, Stmt, StmtKind};
use crate::fxhash::{HashMap, HashSet};
use crate::lang_items::modules;
use crate::source::FileId;
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

/// Indexes kept for reuse (several stdlib folders, or one that changed under a long-running
/// process).
const KEPT: usize = 4;

/// A module and the parameter words of each of its declarations of an operator.
type ModuleHeads = (usize, Vec<Vec<String>>);

/// How many modules import each module.
type Importers = Arc<HashMap<String, u32>>;

/// The paths asked about, and each one's stamp.
type Seen = (HashSet<PathBuf>, Vec<(PathBuf, Option<u64>)>);

/// One standard-library module: `Name.jai` or the folder `Name/module.jai`.
pub(super) struct ModuleEntry {
    /// The name an `#import` writes (`Extensions/Name` for jaic's own modules).
    pub name: String,
    /// The module's entry file, canonicalized, to tell the prelude module from the others.
    entry: PathBuf,
    /// The modules its files `#import`, in order.
    imports: Vec<String>,
}

/// A file of a module, and what its top level exports once it has been parsed.
struct FileEntry {
    module: usize,
    path: PathBuf,
    parsed: OnceLock<Parsed>,
}

enum Parsed {
    /// The names declared where importers see them.
    Names(HashSet<String>),
    /// The file did not parse: any line that looks like a declaration counts.
    Unparsed,
    Unreadable,
}

pub(super) struct StdlibIndex {
    /// Every path the index was built from and its stamp then; `None` when the file system
    /// has no stamps, which makes the index good for the compile that built it only.
    deps: Option<Vec<(PathBuf, u64)>>,
    modules: Vec<ModuleEntry>,
    by_name: HashMap<String, Vec<usize>>,
    files: Vec<FileEntry>,
    /// Names with a line that looks like a top-level declaration, to the files with one.
    declaring: HashMap<String, Vec<usize>>,
    /// Operators to the modules that declare them, with the words of each declaration's
    /// parameter list.
    operators: HashMap<String, Vec<ModuleHeads>>,
    /// How many modules import each module, by the modules left out.
    importers: Mutex<HashMap<Vec<usize>, Importers>>,
}

/// An index as one compile sees it: without the module the compile preloads in its place.
pub(in crate::sema) struct StdlibView {
    pub index: Arc<StdlibIndex>,
    excluded: Vec<usize>,
    pub importers: Importers,
}

impl StdlibView {
    pub fn new(index: Arc<StdlibIndex>, preload: Option<&Path>, fs: &dyn FileSystem) -> Self {
        let preload = preload.map(|p| fs.canonical(p));
        let excluded: Vec<usize> = index
            .modules
            .iter()
            .enumerate()
            .filter(|(_, m)| preload.as_ref().is_some_and(|p| *p == m.entry))
            .map(|(i, _)| i)
            .collect();
        let importers = index.importers_without(&excluded);
        Self {
            index,
            excluded,
            importers,
        }
    }

    /// The module named `name` is there to import.
    pub fn has_module(&self, name: &str) -> bool {
        self.index
            .by_name
            .get(name)
            .is_some_and(|all| all.iter().any(|m| !self.excluded.contains(m)))
    }

    /// Whether module `m` is the one its name stands for: the first of several entries with
    /// that name (a file and a folder) that is not left out.
    fn is_first(&self, m: usize) -> bool {
        !self.excluded.contains(&m)
            && self.index.by_name[&self.index.modules[m].name]
                .iter()
                .find(|other| !self.excluded.contains(other))
                == Some(&m)
    }

    pub fn module_name(&self, m: usize) -> &str {
        &self.index.modules[m].name
    }

    /// The modules that export `name`, in no particular order.
    pub fn modules_exporting_name(&self, name: &str, fs: &dyn FileSystem) -> Vec<usize> {
        let mut found: Vec<usize> = Vec::new();
        // A name with other characters is no declaration in the library.
        if !name.chars().all(is_name_char) {
            return found;
        }
        for &file in self.index.declaring.get(name).into_iter().flatten() {
            let module = self.index.files[file].module;
            if self.is_first(module)
                && !found.contains(&module)
                && self.index.file_exports(file, name, fs)
            {
                found.push(module);
            }
        }
        found
    }

    /// The modules that declare `operator <op>` and qualify: any, with no `names` to match;
    /// otherwise one of `homes` or one with a declaration whose parameters name every one of
    /// `names`.
    pub fn modules_exporting_operator(
        &self,
        op: &str,
        names: &[String],
        homes: &[String],
    ) -> Vec<usize> {
        if op.is_empty() || op.contains(char::is_whitespace) || op.contains("::") {
            return Vec::new();
        }
        self.index
            .operators
            .get(op)
            .into_iter()
            .flatten()
            .filter(|(m, heads)| {
                self.is_first(*m)
                    && (names.is_empty()
                        || homes.iter().any(|h| *h == self.index.modules[*m].name)
                        || heads
                            .iter()
                            .any(|words| names.iter().all(|n| words.contains(n))))
            })
            .map(|(m, _)| *m)
            .collect()
    }
}

impl StdlibIndex {
    /// The index of the standard library under `stdlib`: one that is kept from an earlier
    /// compile when nothing it was built from changed, else a new one.
    pub fn shared(fs: &dyn FileSystem, stdlib: &Path) -> Arc<Self> {
        static KEPT_INDEXES: Mutex<Vec<(PathBuf, Arc<StdlibIndex>)>> = Mutex::new(Vec::new());
        let candidates: Vec<Arc<Self>> = KEPT_INDEXES
            .lock()
            .map(|kept| {
                kept.iter()
                    .rev()
                    .filter(|(root, _)| root == stdlib)
                    .map(|(_, index)| index.clone())
                    .collect()
            })
            .unwrap_or_default();
        if let Some(found) = candidates.into_iter().find(|index| index.is_current(fs)) {
            return found;
        }
        let built = Arc::new(Self::build(fs, stdlib));
        if built.deps.is_some()
            && let Ok(mut kept) = KEPT_INDEXES.lock()
        {
            kept.retain(|(root, old)| root != stdlib || old.is_current(fs));
            kept.push((stdlib.to_path_buf(), built.clone()));
            if kept.len() > KEPT {
                kept.remove(0);
            }
        }
        built
    }

    fn is_current(&self, fs: &dyn FileSystem) -> bool {
        self.deps.as_ref().is_some_and(|deps| {
            deps.iter()
                .all(|(path, stamp)| fs.stamp(path) == Some(*stamp))
        })
    }

    fn build(fs: &dyn FileSystem, stdlib: &Path) -> Self {
        let recorder = Recorder::new(fs);
        let listed = modules_under(&recorder, stdlib, "")
            .into_iter()
            .chain(modules_under(
                &recorder,
                stdlib,
                crate::STDLIB_EXTENSIONS_DIR,
            ))
            .filter(|(m, _)| !modules::INTERNAL.contains(&m.as_str()));
        let mut index = Self {
            deps: None,
            modules: Vec::new(),
            by_name: HashMap::default(),
            files: Vec::new(),
            declaring: HashMap::default(),
            operators: HashMap::default(),
            importers: Mutex::new(HashMap::default()),
        };
        for (name, is_dir) in listed {
            let (entry, files) = if is_dir {
                let mut files = Vec::new();
                module_files(&recorder, &stdlib.join(&name), &mut files, 0);
                (crate::paths::module_entry(&stdlib.join(&name)), files)
            } else {
                let entry = stdlib.join(format!("{name}.jai"));
                (entry.clone(), vec![entry])
            };
            let module = index.modules.len();
            let mut imports = Vec::new();
            for path in files {
                let Some(bytes) = recorder.read(&path) else {
                    continue;
                };
                let text = String::from_utf8_lossy(&bytes);
                imports.extend(imported_modules(&text));
                index.add_file(module, path, &text);
            }
            index.by_name.entry(name.clone()).or_default().push(module);
            index.modules.push(ModuleEntry {
                name,
                entry: fs.canonical(&entry),
                imports,
            });
        }
        index.deps = recorder.finish();
        index
    }

    fn add_file(&mut self, module: usize, path: PathBuf, text: &str) {
        let file = self.files.len();
        let mut names: Vec<&str> = Vec::new();
        let mut ops: Vec<&str> = Vec::new();
        eligible_lines(text, |line| {
            let end = line.find(|c: char| !is_name_char(c)).unwrap_or(line.len());
            if end > 0 && line[end..].trim_start().starts_with(':') {
                names.push(&line[..end]);
            }
            if let Some(rest) = line.strip_prefix("operator") {
                ops.extend(operator_texts(rest.trim_start()));
            }
        });
        names.sort_unstable();
        names.dedup();
        for name in names {
            self.declaring
                .entry(name.to_string())
                .or_default()
                .push(file);
        }
        ops.sort_unstable();
        ops.dedup();
        for op in ops {
            let heads = operator_heads(text, op);
            if heads.is_empty() {
                continue;
            }
            let found = self.operators.entry(op.to_string()).or_default();
            match found.last_mut() {
                Some((m, all)) if *m == module => all.extend(heads),
                _ => found.push((module, heads)),
            }
        }
        self.files.push(FileEntry {
            module,
            path,
            parsed: OnceLock::new(),
        });
    }

    /// Whether `file` exports `name`, given that a line of it looks like such a declaration.
    fn file_exports(&self, file: usize, name: &str, fs: &dyn FileSystem) -> bool {
        let entry = &self.files[file];
        let parsed = entry.parsed.get_or_init(|| match fs.read(&entry.path) {
            None => Parsed::Unreadable,
            Some(bytes) => match exported_names(&String::from_utf8_lossy(&bytes)) {
                Some(names) => Parsed::Names(names),
                None => Parsed::Unparsed,
            },
        });
        match parsed {
            Parsed::Names(names) => names.contains(name),
            Parsed::Unparsed => true,
            Parsed::Unreadable => false,
        }
    }

    /// How many modules, leaving out those in `excluded`, import each module.
    fn importers_without(&self, excluded: &[usize]) -> Importers {
        let mut memo = self
            .importers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        memo.entry(excluded.to_vec())
            .or_insert_with(|| {
                Arc::new(super::module_rank::count_importers(
                    self.modules
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| !excluded.contains(i))
                        .map(|(_, m)| (m.name.as_str(), m.imports.iter().map(String::as_str))),
                ))
            })
            .clone()
    }
}

/// The modules in `stdlib/folder` as (import name, is a folder): `Name` for the stdlib
/// itself (`folder` empty), `folder/Name` below it.
pub(super) fn modules_under(
    fs: &dyn FileSystem,
    stdlib: &Path,
    folder: &str,
) -> Vec<(String, bool)> {
    let dir = stdlib.join(folder);
    let prefix = if folder.is_empty() {
        String::new()
    } else {
        format!("{folder}/")
    };
    fs.list_dir(&dir)
        .into_iter()
        .filter_map(|(entry, is_dir)| {
            module_listed(fs, &dir, &entry, is_dir).map(|m| (format!("{prefix}{m}"), is_dir))
        })
        .collect()
}

/// The `.jai` files of a module directory, leaving out tests and examples.
fn module_files(fs: &dyn FileSystem, dir: &Path, files: &mut Vec<PathBuf>, depth: usize) {
    if depth > 3 {
        return;
    }
    for (entry, is_dir) in fs.list_dir(dir) {
        let path = dir.join(&entry);
        if is_dir {
            if !matches!(entry.as_str(), "tests" | "examples" | "modules") {
                module_files(fs, &path, files, depth + 1);
            }
        } else if entry.ends_with(".jai") {
            files.push(path);
        }
    }
}

/// A file system that notes the stamp of every path it is asked about, before answering.
struct Recorder<'a> {
    fs: &'a dyn FileSystem,
    seen: RefCell<Seen>,
}

impl<'a> Recorder<'a> {
    fn new(fs: &'a dyn FileSystem) -> Self {
        Self {
            fs,
            seen: RefCell::default(),
        }
    }

    fn note(&self, path: &Path) {
        let mut seen = self.seen.borrow_mut();
        if seen.0.insert(path.to_path_buf()) {
            seen.1.push((path.to_path_buf(), self.fs.stamp(path)));
        }
    }

    /// The stamps of everything asked about, when the file system has them all.
    fn finish(self) -> Option<Vec<(PathBuf, u64)>> {
        self.seen
            .into_inner()
            .1
            .into_iter()
            .map(|(path, stamp)| stamp.map(|s| (path, s)))
            .collect()
    }
}

impl FileSystem for Recorder<'_> {
    fn read(&self, path: &Path) -> Option<Vec<u8>> {
        self.note(path);
        self.fs.read(path)
    }

    fn is_file(&self, path: &Path) -> bool {
        self.note(path);
        self.fs.is_file(path)
    }

    fn is_dir(&self, path: &Path) -> bool {
        self.note(path);
        self.fs.is_dir(path)
    }

    fn canonical(&self, path: &Path) -> PathBuf {
        self.fs.canonical(path)
    }

    fn list_dir(&self, path: &Path) -> Vec<(String, bool)> {
        self.note(path);
        self.fs.list_dir(path)
    }
}

fn is_name_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Calls `f` with each line of `text`, trimmed at the start, that is indented at most four
/// spaces and outside a `#scope_file` section (a line-based test, as the declarations of a
/// module start their line).
fn eligible_lines<'a>(text: &'a str, mut f: impl FnMut(&'a str)) {
    let mut file_scope = false;
    for line in text.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("#scope_file") {
            file_scope = true;
        } else if trimmed.starts_with("#scope_export") || trimmed.starts_with("#scope_module") {
            file_scope = trimmed.starts_with("#scope_module");
        } else if !file_scope && line.len() - trimmed.len() <= 4 {
            f(trimmed);
        }
    }
}

/// The operators an `operator` line declares, given what follows the keyword (without
/// leading blanks): the text before a `::`, and none that holds a blank.
fn operator_texts(rest: &str) -> Vec<&str> {
    let Some(first) = rest.find("::") else {
        return Vec::new();
    };
    (0..=first)
        .filter(|&k| rest.is_char_boundary(k) && rest[k..].trim_start().starts_with("::"))
        .map(|k| &rest[..k])
        .filter(|op| !op.is_empty() && !op.contains(char::is_whitespace))
        .collect()
}

/// The words of the parameter list of each `operator <op>` declaration in `text` outside its
/// `#scope_file` sections: the line, and the lines after it up to the opening `{` or `->`,
/// split at everything that is not part of a name.
fn operator_heads(text: &str, op: &str) -> Vec<Vec<String>> {
    let mut heads = Vec::new();
    let mut file_scope = false;
    let mut lines = text.lines();
    while let Some(line) = lines.next() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("#scope_file") {
            file_scope = true;
        } else if trimmed.starts_with("#scope_export") || trimmed.starts_with("#scope_module") {
            file_scope = trimmed.starts_with("#scope_module");
        } else if !file_scope
            && line.len() - trimmed.len() <= 4
            && trimmed
                .strip_prefix("operator")
                .and_then(|rest| rest.trim_start().strip_prefix(op))
                .is_some_and(|rest| rest.trim_start().starts_with("::"))
        {
            let mut head = trimmed.to_string();
            while !head.contains('{') && !head.contains("->") && head.len() < 400 {
                match lines.next() {
                    Some(next) => head.push_str(next),
                    None => break,
                }
            }
            let end = head
                .find("->")
                .or_else(|| head.find('{'))
                .unwrap_or(head.len());
            let mut words: Vec<String> = head[..end]
                .split(|c: char| !is_name_char(c))
                .map(str::to_string)
                .collect();
            words.sort_unstable();
            words.dedup();
            heads.push(words);
        }
    }
    heads
}

/// The names `text` declares for its importers: at its top level (or in a top-level `#if`),
/// outside `#scope_file` and `#scope_module` sections. `None` when it does not parse.
fn exported_names(text: &str) -> Option<HashSet<String>> {
    fn collect(stmts: &[Stmt], exported: &mut bool, names: &mut HashSet<String>) {
        for stmt in stmts {
            match &stmt.kind {
                StmtKind::Scope(kind) => *exported = *kind == ScopeKind::Export,
                StmtKind::Decl(decl) if *exported => {
                    names.extend(decl.names.iter().map(|n| n.name.as_str().to_string()));
                }
                StmtKind::StaticIf(boxed) => {
                    let crate::ast::StaticIfStmt {
                        then_branch,
                        else_branch,
                        ..
                    } = &**boxed;
                    let (mut then_exported, mut else_exported) = (*exported, *exported);
                    collect(then_branch, &mut then_exported, names);
                    collect(else_branch, &mut else_exported, names);
                }
                _ => {}
            }
        }
    }
    let file = crate::parser::parse_file(FileId(u32::MAX), text).ok()?;
    let mut names = HashSet::default();
    collect(&file.stmts, &mut true, &mut names);
    Some(names)
}

/// The modules `#import "Name"` names in `text`, found from its tokens.
fn imported_modules(text: &str) -> Vec<String> {
    if !text.contains("#import") {
        return Vec::new();
    }
    let Ok(tokens) = crate::lexer::lex(FileId(u32::MAX), text) else {
        return Vec::new();
    };
    crate::project::scan_directives(&tokens)
        .into_iter()
        .filter_map(|link| match link.source {
            crate::project::LinkSource::Import(crate::ast::ImportSource::Module(name)) => {
                Some(name.to_string())
            }
            _ => None,
        })
        .collect()
}
