//! Type-checked answers for hover and completion: the open documents are compiled with jaic
//! (bundled or on-disk stdlib) and the compiler's editor facts (`jaic::sema::ide`) are queried.
//! Compilation happens on demand and is cached per source text.
use jaic::build::{BuildEnv, WorkspaceObserver, Workspaces};
use jaic::intern::Sym;
use jaic::interp::{SandboxHost, SharedHost};
use jaic::sema::ide::{IdeFacts, IdeLayout, IdeName};
use jaic::sema::{Compiler, FileSystem, Options, TargetCpu, TargetOs};
use jaic::source::{FileId, ImportSuggestion};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// Where the language server finds modules: a file system for everything that is not an open
/// document, and compiler options for a main file (import paths, Preload, target).
pub struct Environment {
    pub fs: Rc<dyn FileSystem>,
    pub options: Box<dyn Fn(&Path) -> Options>,
}

/// A declaration in a compiled file: (path, text of the file, start, end of its name).
pub type Decl = (String, Rc<str>, usize, usize);

/// What the type checker says about a hover.
pub struct HoverFacts {
    pub start: usize,
    pub end: usize,
    pub text: String,
    pub layout: Option<IdeLayout>,
    pub decls: Vec<Decl>,
}

/// Basic blocks compile-time code may run per analysis (an edit can make a `#run` loop forever).
const BLOCK_BUDGET: u64 = 200_000_000;

/// Compilers kept for reuse: the newest one that checked (what completion falls back to while
/// the text does not parse) and the newest of all. A compile is hundreds of MiB for a large
/// program, so room is made before the next one starts: at most `CACHED` are alive at once,
/// the one being built included.
const CACHED: usize = 2;

/// Compiles for another target kept besides those (for code an `#if OS == ...` skipped), the
/// one being built included.
const ALTERNATES: usize = 3;

/// The targets asked, in order, about code the environment's target skips.
pub const OTHER_TARGETS: [TargetOs; 4] = [
    TargetOs::Linux,
    TargetOs::Windows,
    TargetOs::MacOS,
    TargetOs::Wasm,
];

/// Open documents over the environment's file system.
struct OverlayFs {
    base: Rc<dyn FileSystem>,
    /// Where relative paths start (the root file's folder): the server's own working directory
    /// is wherever the editor started it, and a build metaprogram names its files and module
    /// folders relative to itself.
    dir: PathBuf,
    files: BTreeMap<PathBuf, Rc<[u8]>>,
    /// `canonical` answers so far: a compile asks about the same paths over and over.
    canonical: RefCell<HashMap<PathBuf, PathBuf>>,
}

impl OverlayFs {
    fn at<'a>(&self, path: &'a Path) -> std::borrow::Cow<'a, Path> {
        if path.is_relative() {
            self.dir.join(path).into()
        } else {
            path.into()
        }
    }

    fn canonical_of(&self, path: &Path) -> PathBuf {
        if let Some(known) = self.canonical.borrow().get(path) {
            return known.clone();
        }
        let found = self.base.canonical(&self.at(path));
        self.canonical
            .borrow_mut()
            .insert(path.to_path_buf(), found.clone());
        found
    }
}

impl FileSystem for OverlayFs {
    fn read(&self, path: &Path) -> Option<Vec<u8>> {
        match self.files.get(&self.canonical_of(path)) {
            Some(bytes) => Some(bytes.to_vec()),
            None => self.base.read(&self.at(path)),
        }
    }

    fn is_file(&self, path: &Path) -> bool {
        self.files.contains_key(&self.canonical_of(path)) || self.base.is_file(&self.at(path))
    }

    fn is_dir(&self, path: &Path) -> bool {
        let p = self.canonical_of(path);
        self.files.keys().any(|k| k.starts_with(&p) && *k != p) || self.base.is_dir(&self.at(path))
    }

    fn canonical(&self, path: &Path) -> PathBuf {
        self.canonical_of(path)
    }

    fn list_dir(&self, path: &Path) -> Vec<(String, bool)> {
        self.base.list_dir(&self.at(path))
    }
}

/// The open documents' texts by path, shared so that asking again with the same inputs is
/// known by address alone.
pub type Buffers = Rc<BTreeMap<PathBuf, Rc<[u8]>>>;

pub struct Analysis {
    pub compiler: Compiler,
    key: u64,
    /// `quick_key` of the inputs last answered from this compile, with those inputs, which
    /// keeps their buffers alive so no other text can take their addresses.
    quick: Option<(u64, Buffers)>,
    /// The target it was compiled for, when not the environment's (see `other_targets`).
    os: Option<TargetOs>,
    /// The program's root file, the module folders searched and the text of every open file it
    /// was compiled from (shared with the compiler's file system, so no copy).
    root: PathBuf,
    dirs: Vec<PathBuf>,
    files: BTreeMap<PathBuf, Rc<[u8]>>,
    /// The compile got far enough to record scopes and names (the text parsed), so its facts
    /// can answer for an edit that left the code before the cursor alone.
    usable: bool,
    /// Bytes of source the compile read (its programs' included): what dropping it would cost
    /// to redo.
    cost: usize,
    /// The program compiled without errors.
    complete: bool,
    /// Why it did not: the compiler's first error, then the independent errors the editor
    /// check of the other declarations and bodies found.
    errors: Vec<jaic::source::Diagnostic>,
    /// Lints found so far, by file.
    lints: BTreeMap<FileId, Vec<jailint::Lint>>,
    /// The imports that would declare the error's unknown name (worked out on first use: it
    /// reads the standard library).
    imports: Option<Vec<ImportSuggestion>>,
    /// `file`'s answers by the path asked for: canonicalizing every compiled file's path for
    /// each request was a quarter of a hover.
    found: RefCell<BTreeMap<PathBuf, Option<FileId>>>,
    /// The programs the metaprogram built (`compiler_create_workspace`, as a `first.jai` does):
    /// the files they compiled are answered from them (see `focus`).
    others: Vec<Program>,
}

/// A program one compile produced, with what is known of it so far. The answering one lives in
/// `Analysis`'s own fields; the others wait in `Analysis::others`.
struct Program {
    compiler: Compiler,
    complete: bool,
    errors: Vec<jaic::source::Diagnostic>,
    lints: BTreeMap<FileId, Vec<jailint::Lint>>,
    imports: Option<Vec<ImportSuggestion>>,
}

impl Analysis {
    /// Answer from the program that compiled `path`: the compiled root's own, or one a
    /// metaprogram built. The answering program stays when it compiled `path` too (a module
    /// both import).
    pub fn focus(&mut self, path: &Path) {
        if self.others.is_empty() || self.file(path).is_some() {
            return;
        }
        let Some(at) = self
            .others
            .iter()
            .position(|p| find_file(&p.compiler, path).is_some())
        else {
            return;
        };
        let other = &mut self.others[at];
        std::mem::swap(&mut self.compiler, &mut other.compiler);
        std::mem::swap(&mut self.complete, &mut other.complete);
        std::mem::swap(&mut self.errors, &mut other.errors);
        std::mem::swap(&mut self.lints, &mut other.lints);
        std::mem::swap(&mut self.imports, &mut other.imports);
        self.found.borrow_mut().clear();
    }

    /// jailint's findings in `path` (computed once per compile).
    pub fn lints(
        &mut self,
        path: &Path,
        config: &jailint::config::Config,
    ) -> Option<Vec<jailint::Lint>> {
        let file = self.file(path)?;
        if let Some(found) = self.lints.get(&file) {
            return Some(found.clone());
        }
        let found = jailint::lint_files(&mut self.compiler, &[file], config, self.complete);
        self.lints.insert(file, found.clone());
        Some(found)
    }

    /// The compile errors as (start, end, message) in `path`: each at its own span when that
    /// is in the file, else at the first of its notes that is (a module procedure the file
    /// called).
    pub fn check_errors(&self, path: &Path) -> Vec<(usize, usize, String)> {
        self.located(path, &self.errors)
    }

    /// The compiler's warnings (a print format with the wrong argument count, a body that can
    /// fall off its end), placed like the errors.
    pub fn check_warnings(&self, path: &Path) -> Vec<(usize, usize, String)> {
        self.located(path, &self.compiler.warnings)
    }

    fn located(
        &self,
        path: &Path,
        diagnostics: &[jaic::source::Diagnostic],
    ) -> Vec<(usize, usize, String)> {
        let Some(file) = self.file(path) else {
            return Vec::new();
        };
        let mut out: Vec<(usize, usize, String)> = Vec::new();
        let mut seen = HashSet::new();
        for error in diagnostics {
            let found = if error.span.file == file {
                let span = error.span;
                Some((
                    span.start as usize,
                    span.end as usize,
                    error.message.clone(),
                ))
            } else {
                error
                    .notes
                    .iter()
                    .find(|(span, _)| span.file == file)
                    .map(|(span, note)| {
                        (
                            span.start as usize,
                            span.end as usize,
                            format!("{}\n{note}", error.message),
                        )
                    })
            };
            if let Some(found) = found
                && seen.insert(found.clone())
            {
                out.push(found);
            }
        }
        out
    }

    /// The `#import`s that would declare the unknown name the compile error in `path` reports,
    /// best first, with the error's span as (start, end).
    pub fn missing_imports(
        &mut self,
        path: &Path,
    ) -> Option<(usize, usize, Vec<ImportSuggestion>)> {
        let file = self.file(path)?;
        let error = self.errors.first()?;
        if error.span.file != file {
            return None;
        }
        let span = error.span;
        let imports = self.imports.get_or_insert_with(|| {
            self.compiler
                .with_name_suggestion(error)
                .and_then(|d| d.fixes)
                .map(|fixes| fixes.imports)
                .unwrap_or_default()
        });
        Some((span.start as usize, span.end as usize, imports.clone()))
    }

    /// `offset` of `path` is code this compile skipped for its target: the file is not part of
    /// it, or an `#if` on `OS` or `CPU` did not take the branch it is in.
    pub fn skipped(&self, path: &Path, offset: usize) -> bool {
        self.skipped_span(path, offset).is_some()
    }

    /// The byte range around `offset` of `path` this compile did not check: an `#if` branch on
    /// `OS` or `CPU` it did not take, or the whole file when the compile did not reach it.
    pub fn skipped_span(&self, path: &Path, offset: usize) -> Option<(usize, usize)> {
        let Some(file) = self.file(path) else {
            return Some((0, usize::MAX));
        };
        self.compiler.ide.as_ref().and_then(|ide| {
            ide.skipped
                .iter()
                .find(|s| s.file == file && s.start as usize <= offset && offset <= s.end as usize)
                .map(|s| (s.start as usize, s.end as usize))
        })
    }

    pub fn file(&self, path: &Path) -> Option<FileId> {
        if let Some(&found) = self.found.borrow().get(path) {
            return found;
        }
        let found = find_file(&self.compiler, path);
        self.found.borrow_mut().insert(path.to_path_buf(), found);
        found
    }

    /// A span of any compiled file as (path, text, start, end); `None` for a span in no file
    /// (`Span::NONE`, which builtins carry).
    pub fn location(&self, span: jaic::source::Span) -> Option<(String, Rc<str>, usize, usize)> {
        let source = self.compiler.sources.try_get(span.file)?;
        Some((
            source.path.clone(),
            source.text.clone(),
            span.start as usize,
            span.end as usize,
        ))
    }

    /// Hover at `offset`: what it covers, its description and memory layout, and where what
    /// it describes is declared (one per line of the description for overloads).
    pub fn hover(&mut self, path: &Path, offset: usize) -> Option<HoverFacts> {
        let file = self.file(path)?;
        let hover = self.compiler.ide_hover(file, offset as u32)?;
        let decls = self
            .compiler
            .ide_hover_decls(file, offset as u32)
            .into_iter()
            .filter_map(|span| self.location(span))
            .collect();
        Some(HoverFacts {
            start: hover.span.start as usize,
            end: hover.span.end as usize,
            text: hover.text,
            layout: hover.layout,
            decls,
        })
    }

    /// The declarations `names` (`print`, `Basic.print`, `Type.field`) name as seen from
    /// `offset` of the file at `path`: where doc comment links lead. Falls back to the file's
    /// own scope when the file has no recorded scopes.
    pub fn resolve_names(&mut self, path: &Path, offset: usize, names: &[String]) -> Vec<Decl> {
        let Some(file) = self.file(path) else {
            return Vec::new();
        };
        let Some(scope) = self.compiler.ide_scope_at(file, offset as u32) else {
            return Vec::new();
        };
        let names: Vec<Sym> = names.iter().map(|n| Sym::intern(n)).collect();
        self.compiler
            .ide_resolve_path(scope, &names)
            .into_iter()
            .filter_map(|span| self.location(span))
            .collect()
    }

    /// The path of each file `names` are declared in, by file number.
    /// Moves the declarations of `names` in the file at `path` from the text this analysis
    /// compiled to where they are in `text`, the document as it is now: past the edited
    /// stretch (the longest common prefix and suffix of the two) by the change in length.
    pub fn shift_to(&self, path: &Path, text: &str, names: &mut [IdeName]) {
        let Some(file) = self.file(path) else {
            return;
        };
        let Some(source) = self.compiler.sources.try_get(file) else {
            return;
        };
        let (old, new) = (source.text.as_bytes(), text.as_bytes());
        if old == new {
            return;
        }
        let prefix = old.iter().zip(new).take_while(|(a, b)| a == b).count();
        let room = old.len().min(new.len()) - prefix;
        let suffix = old
            .iter()
            .rev()
            .zip(new.iter().rev())
            .take(room)
            .take_while(|(a, b)| a == b)
            .count();
        let tail = (old.len() - suffix) as u32;
        let delta = new.len() as i64 - old.len() as i64;
        let moved = |at: u32| {
            if at >= tail {
                (at as i64 + delta) as u32
            } else {
                at
            }
        };
        for name in names.iter_mut().filter(|n| n.decl.file == file) {
            name.decl.start = moved(name.decl.start);
            name.decl.end = moved(name.decl.end);
        }
    }

    pub fn decl_files(&self, names: &[IdeName]) -> BTreeMap<u32, String> {
        let mut out = BTreeMap::new();
        for name in names {
            let file = name.decl.file;
            if let std::collections::btree_map::Entry::Vacant(slot) = out.entry(file.0)
                && let Some(source) = self.compiler.sources.try_get(file)
            {
                slot.insert(source.path.clone());
            }
        }
        out
    }

    /// Declarations the identifier at `offset` names, as (path, text, start, end) with byte
    /// offsets into that file's text (which may be a module or stdlib file, not an open document).
    pub fn definition(
        &mut self,
        path: &Path,
        offset: usize,
    ) -> Vec<(String, Rc<str>, usize, usize)> {
        let Some(file) = self.file(path) else {
            return Vec::new();
        };
        self.compiler
            .ide_definition(file, offset as u32)
            .into_iter()
            .filter_map(|span| self.location(span))
            .collect()
    }

    /// The members of the enum an inferred `.NAME` at `offset` is expected to be.
    pub fn complete_inferred(&mut self, path: &Path, offset: usize) -> Option<Vec<IdeName>> {
        let file = self.file(path)?;
        Some(self.compiler.ide_inferred_members(file, offset as u32))
    }

    /// Completion candidates at `offset`, members of `chain` when it is not empty.
    pub fn complete(&mut self, path: &Path, offset: usize, chain: &[&str]) -> Option<Vec<IdeName>> {
        let file = self.file(path)?;
        let scope = self.compiler.ide_scope_at(file, offset as u32)?;
        if chain.is_empty() {
            return Some(self.compiler.ide_visible(scope, file, offset as u32));
        }
        let names: Vec<Sym> = chain.iter().map(|n| Sym::intern(n)).collect();
        let receiver = self.compiler.ide_receiver(scope, &names)?;
        Some(self.compiler.ide_members(receiver))
    }
}

#[derive(Default)]
pub struct Cache {
    entries: Vec<Analysis>,
    /// The target that last answered about skipped code in each file: asked first next time,
    /// so the one compile kept for another target keeps answering.
    pub answered: BTreeMap<PathBuf, TargetOs>,
    /// Canonical forms of the paths compiled, so a request does not ask the file system again.
    canonical: BTreeMap<PathBuf, PathBuf>,
    /// Programs compiled so far.
    compiles: usize,
}

fn hash(
    root: &Path,
    dirs: &[PathBuf],
    files: &BTreeMap<PathBuf, Rc<[u8]>>,
    os: Option<TargetOs>,
) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = crate::fast_hash::FastHasher::default();
    format!("{os:?}").hash(&mut h);
    root.hash(&mut h);
    dirs.hash(&mut h);
    files.hash(&mut h);
    h.finish()
}

/// A key for inputs by the map of texts they share rather than its contents: equal only for
/// the very same map (or one at the same address, which `Analysis::quick` keeps alive).
fn quick_key(root: &Path, dirs: &[PathBuf], files: &Buffers, os: Option<TargetOs>) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = crate::fast_hash::FastHasher::default();
    os.map(|os| os as u8).hash(&mut h);
    h.write(root.as_os_str().as_encoded_bytes());
    for dir in dirs {
        h.write(dir.as_os_str().as_encoded_bytes());
    }
    (Rc::as_ptr(files) as usize).hash(&mut h);
    h.finish()
}

impl Cache {
    /// Programs compiled so far, and compiles alive now.
    pub fn counts(&self) -> (usize, usize) {
        (self.compiles, self.entries.len())
    }

    /// Whether these inputs, as the very buffers given, have a compile kept for `os`.
    pub fn has(
        &self,
        root: &Path,
        dirs: &[PathBuf],
        files: &Buffers,
        os: Option<TargetOs>,
    ) -> bool {
        let quick = quick_key(root, dirs, files, os);
        self.entries
            .iter()
            .any(|a| a.quick.as_ref().is_some_and(|(k, _)| *k == quick))
    }

    /// Files appeared or went away: canonical paths are asked for again.
    pub fn forget_paths(&mut self) {
        self.canonical.clear();
    }

    /// Drop the lints found so far (their settings changed); the compiles stay.
    pub fn forget_lints(&mut self) {
        for entry in &mut self.entries {
            entry.lints.clear();
        }
    }

    /// Drop compiles until one fewer than `CACHED` is left. The one kept is the newest that
    /// checked of `root` (what completion falls back to while its text does not parse), else
    /// the costliest that checked (a project's compile, not a stray file's that takes
    /// milliseconds to redo), else the newest. A compile for another target makes room only
    /// among those (one is kept), so asking about a skipped branch does not cost the program's
    /// own compile.
    fn make_room(&mut self, os: Option<TargetOs>, root: &Path) {
        if os.is_some() {
            // The oldest compile for another target goes; the newest stays, so hovers that go
            // back and forth between a Windows and a Linux branch do not recompile each time.
            while self.entries.iter().filter(|a| a.os.is_some()).count() >= ALTERNATES {
                let oldest = self.entries.iter().position(|a| a.os.is_some());
                self.entries.remove(oldest.expect("counted"));
            }
            return;
        }
        while self.entries.iter().filter(|a| a.os.is_none()).count() >= CACHED {
            let own = |a: &Analysis| a.usable && a.os.is_none();
            let keep = self
                .entries
                .iter()
                .rposition(|a| own(a) && a.root == root)
                .or_else(|| {
                    (0..self.entries.len())
                        .filter(|&i| own(&self.entries[i]))
                        .max_by_key(|&i| self.entries[i].cost)
                })
                .or_else(|| self.entries.iter().rposition(|a| a.os.is_none()));
            let victim = (0..self.entries.len())
                .find(|&i| Some(i) != keep && self.entries[i].os.is_none())
                .unwrap_or(0);
            self.entries.remove(victim);
        }
    }

    /// A compile of `root` that already knows the code before `prefix.len()` of `path`: the
    /// other open files are as they were compiled and `path` starts with `prefix` (what is
    /// typed after that point is not in its facts, and does not need to be). Completion uses it
    /// instead of compiling again for every keystroke; the names declared by edits since that
    /// compile show up once the next one has run.
    pub fn compiled_before(
        &mut self,
        env: &Environment,
        root: &Path,
        dirs: &[PathBuf],
        files: Buffers,
        path: &Path,
        prefix: &[u8],
    ) -> Option<&mut Analysis> {
        let canonical: BTreeMap<PathBuf, Rc<[u8]>> = files
            .iter()
            .map(|(p, t)| (env.fs.canonical(p), t.clone()))
            .collect();
        let path = env.fs.canonical(path);
        let at = self.entries.iter().rposition(|a| {
            a.usable
                && a.os.is_none()
                && a.root == root
                && a.dirs == dirs
                && a.files.len() == canonical.len()
                && a.files.iter().all(|(p, bytes)| match canonical.get(p) {
                    Some(new) if *p == path => bytes.starts_with(prefix) && new.starts_with(prefix),
                    Some(new) => bytes == new,
                    None => false,
                })
        })?;
        let analysis = &mut self.entries[at];
        analysis.focus(&path);
        Some(analysis)
    }

    /// The analysis of `root` with `files` open (compiled now unless cached).
    pub fn analyze(
        &mut self,
        env: &Environment,
        root: &Path,
        dirs: &[PathBuf],
        files: Buffers,
    ) -> &mut Analysis {
        self.analyze_for(env, root, dirs, files, None)
    }

    /// Like `analyze`, compiled for `os` instead of the environment's target when it is `Some`.
    pub fn analyze_for(
        &mut self,
        env: &Environment,
        root: &Path,
        dirs: &[PathBuf],
        files: Buffers,
        os: Option<TargetOs>,
    ) -> &mut Analysis {
        // The same buffers as last time (the session hands out its inputs again until a
        // document changes): no need to hash megabytes of text to know it is the same program.
        let quick = quick_key(root, dirs, &files, os);
        if let Some(at) = self
            .entries
            .iter()
            .position(|a| a.quick.as_ref().is_some_and(|(k, _)| *k == quick))
        {
            let entry = self.entries.remove(at);
            self.entries.push(entry);
            return self.entries.last_mut().expect("just pushed");
        }
        let given = files.clone();
        let files: BTreeMap<PathBuf, Rc<[u8]>> = files
            .iter()
            .map(|(p, t)| {
                let canonical = match self.canonical.get(p) {
                    Some(c) => c.clone(),
                    None => {
                        let c = env.fs.canonical(p);
                        self.canonical.insert(p.clone(), c.clone());
                        c
                    }
                };
                (canonical, t.clone())
            })
            .collect();
        let key = hash(root, dirs, &files, os);
        if let Some(at) = self.entries.iter().position(|a| a.key == key) {
            let mut entry = self.entries.remove(at);
            entry.quick = Some((quick, given));
            self.entries.push(entry);
        } else {
            // A compile of the same program from text that parses will check, and everything the
            // older one can answer (completion after an unchanged prefix) it answers too, so the
            // older one goes now instead of staying alive through the whole new compile.
            if files.values().all(|bytes| {
                std::str::from_utf8(bytes).is_ok_and(|text| parse_error(text).is_none())
            }) {
                self.entries
                    .retain(|a| !(a.root == root && a.dirs == dirs && a.os == os));
            }
            self.make_room(os, root);
            self.compiles += 1;
            let snapshot = files.clone();
            let (compiler, errors, others) = compile(env, root, dirs, files, os);
            let cost = std::iter::once(&compiler)
                .chain(others.iter().map(|p| &p.compiler))
                .flat_map(|c| {
                    (0..c.sources.len() as u32).map(|f| c.sources.get(FileId(f)).text.len())
                })
                .sum();
            let usable = errors.is_empty()
                || compiler
                    .ide
                    .as_ref()
                    .is_some_and(|ide| !ide.scopes.is_empty());
            self.entries.push(Analysis {
                compiler,
                key,
                quick: Some((quick, given)),
                os,
                root: root.to_path_buf(),
                dirs: dirs.to_vec(),
                files: snapshot,
                usable,
                cost,
                complete: errors.is_empty(),
                errors,
                lints: BTreeMap::new(),
                imports: None,
                found: RefCell::default(),
                others,
            });
        }
        self.entries.last_mut().expect("just pushed")
    }
}

/// `path`'s id among the files `compiler` compiled.
fn find_file(compiler: &Compiler, path: &Path) -> Option<FileId> {
    let files = || (0..compiler.sources.len() as u32).map(FileId);
    files()
        .find(|&f| Path::new(&compiler.sources.get(f).path) == path)
        .or_else(|| {
            let want = compiler.fs.canonical(path);
            files().find(|&f| {
                compiler
                    .fs
                    .canonical(Path::new(&compiler.sources.get(f).path))
                    == want
            })
        })
}

/// The editor facts a compile records for the files under `prefix`, its compile-time output
/// going to `host`, with what jailint needs (expression types, casts, uses) when `lint`.
fn editor_facts(prefix: &str, host: &Rc<RefCell<SandboxHost>>, lint: bool) -> Box<IdeFacts> {
    let mut facts = IdeFacts::new(vec![prefix.to_string()]);
    facts.output = Some(host.clone());
    facts.lint = lint;
    Box::new(facts)
}

/// Gives each compiler a metaprogram creates editor facts, and keeps it when it is done.
struct Built {
    prefix: String,
    host: Rc<RefCell<SandboxHost>>,
    lint: bool,
    done: Rc<RefCell<Vec<Compiler>>>,
}

impl WorkspaceObserver for Built {
    fn created(&mut self, compiler: &mut Compiler) {
        compiler.ide = Some(editor_facts(&self.prefix, &self.host, self.lint));
    }

    fn finished(&mut self, compiler: Box<Compiler>, _failed: bool) {
        self.done.borrow_mut().push(*compiler);
    }
}

/// After a compile: its errors (the first it stopped at, then the independent ones among the
/// declarations and bodies it did not get to), with every body checked and the polymorphs
/// nothing called sketched.
fn check_rest(
    compiler: &mut Compiler,
    mut errors: Vec<jaic::source::Diagnostic>,
) -> Vec<jaic::source::Diagnostic> {
    let mut seen: HashSet<_> = errors.iter().map(|e| (e.span, e.message.clone())).collect();
    for e in compiler.ide_check_all() {
        if seen.insert((e.span, e.message.clone())) {
            errors.push(e);
        }
    }
    compiler.ide_sketch_polymorphs();
    errors
}

/// The compiled program and its errors, and the programs its metaprogram built.
fn compile(
    env: &Environment,
    root: &Path,
    dirs: &[PathBuf],
    files: BTreeMap<PathBuf, Rc<[u8]>>,
    os: Option<TargetOs>,
) -> (Compiler, Vec<jaic::source::Diagnostic>, Vec<Program>) {
    let dir = root.parent().map(Path::to_path_buf).unwrap_or_default();
    let fs: Rc<dyn FileSystem> = Rc::new(OverlayFs {
        base: env.fs.clone(),
        dir: dir.clone(),
        files,
        canonical: RefCell::default(),
    });
    let options = || {
        let mut options = (env.options)(root);
        crate::modules::splice(&mut options.import_paths, dirs);
        if let Some(os) = os {
            options.os = os;
            options.cpu = match os {
                TargetOs::Windows | TargetOs::Linux => TargetCpu::X64,
                TargetOs::MacOS => TargetCpu::Arm64,
                TargetOs::Wasm => TargetCpu::Wasm,
            };
            options.long_double = jaic::sema::long_double_for(os, options.cpu, false);
        }
        options
    };
    let mut compiler = Compiler::new(options(), fs.clone());
    // Compile-time output goes nowhere: stdout may be the protocol channel.
    let host = Rc::new(RefCell::new(SandboxHost::with_files(
        fs,
        &dir.to_string_lossy(),
    )));
    compiler.interp.host = Box::new(SharedHost(host.clone()));
    compiler.interp.block_budget = Some(BLOCK_BUDGET);
    let mut prefix = env.fs.canonical(&dir).to_string_lossy().into_owned();
    if !prefix.ends_with('/') {
        prefix.push('/');
    }
    // Metaprograms may compile other programs (`compiler_create_workspace`): those are checked
    // the same way, with no output and the same host, and their compile-time code draws from
    // this compiler's budget.
    let workspace_host = host.clone();
    // A compile for another target only answers hover and definition: lints come from the
    // environment's own.
    let lint = os.is_none();
    let built = Rc::new(RefCell::new(Vec::new()));
    let workspaces = Workspaces::new(BuildEnv {
        unwritten_output_hint: None,
        fs: compiler.fs.clone(),
        options: options(),
        backend: None,
        command_line: Vec::new(),
        make_host: Box::new(move |_| Box::new(SharedHost(workspace_host.clone()))),
        report: Box::new(|_| {}),
        observer: Some(Box::new(Built {
            prefix: prefix.clone(),
            host: host.clone(),
            lint,
            done: built.clone(),
        })),
    });
    compiler.attach_workspaces(workspaces);
    compiler.ide = Some(editor_facts(&prefix, &host, lint));
    let errors = compiler
        .compile_program(root)
        .err()
        .into_iter()
        .map(|e| *e)
        .collect::<Vec<_>>();
    // Workspaces no message loop drove compile once the metaprogram is done, as in a build.
    // Their stopping error is in their own facts, so the message here adds nothing.
    if errors.is_empty()
        && let Some(workspaces) = compiler.interp.workspaces.clone()
    {
        let _ = jaic::build::finish_all(&workspaces, &mut compiler.interp.block_budget);
    }
    let errors = check_rest(&mut compiler, errors);
    let mut others: Vec<Program> = built
        .take()
        .into_iter()
        .map(|mut compiler| {
            // The metaprogram's code spent the budget; checking the rest gets its own.
            compiler.interp.block_budget = Some(BLOCK_BUDGET);
            let errors = check_rest(&mut compiler, Vec::new());
            Program {
                compiler,
                complete: errors.is_empty(),
                errors,
                lints: BTreeMap::new(),
                imports: None,
            }
        })
        .collect();
    // Compile-time code is over once everything is checked. Each program holds the build
    // state, which would otherwise outlive the compile by hundreds of MB.
    // Nothing runs the lowered bodies either: hovers and lints answer from the facts.
    compiler.release_build_state();
    compiler.release_lowered_code();
    for other in &mut others {
        other.compiler.release_build_state();
        other.compiler.release_lowered_code();
    }
    // Every interpreter on the host is closed: what compile-time code left allocated goes too.
    host.borrow_mut().release_memory();
    (compiler, errors, others)
}

thread_local! {
    /// Where the texts parsed lately fail (by hash and length): hover, references, diagnostics
    /// and lints each ask about the same open document, and a parse of a large file is a
    /// transient tree of tens of MiB each time.
    static PARSES: std::cell::RefCell<Vec<(u64, usize, Option<usize>)>> = const { std::cell::RefCell::new(Vec::new()) };
    /// The text asked about last and its answer: requests on one document ask about the same
    /// text again and again, and comparing bytes is several times faster than hashing them.
    static LAST_PARSE: std::cell::RefCell<(String, Option<usize>)> = const { std::cell::RefCell::new((String::new(), None)) };
}

/// Parse results remembered: one per open document and a few for the texts the repair of a
/// broken document tries. Each is 24 bytes; with fewer than there were open documents, every
/// new compile of a large project parsed all its files again to see that they parse.
const PARSES_KEPT: usize = 1024;

pub(crate) fn text_key(text: &str) -> (u64, usize) {
    use std::hash::{Hash, Hasher};
    let mut h = crate::fast_hash::FastHasher::default();
    text.hash(&mut h);
    (h.finish(), text.len())
}

/// Remember that `text` parses (`None`) or fails at a byte offset, for `parse_error`.
pub fn note_parse(text: &str, error: Option<usize>) {
    let (hash, len) = text_key(text);
    PARSES.with(|p| {
        let mut p = p.borrow_mut();
        p.retain(|e| (e.0, e.1) != (hash, len));
        if p.len() >= PARSES_KEPT {
            p.remove(0);
        }
        p.push((hash, len, error));
    });
}

/// Where `text` fails to lex or parse (a byte offset), if it does.
pub fn parse_error(text: &str) -> Option<usize> {
    if let Some(found) = LAST_PARSE.with(|last| {
        let last = last.borrow();
        (last.0 == text).then_some(last.1)
    }) {
        return found;
    }
    let (hash, len) = text_key(text);
    let error = match PARSES.with(|p| {
        p.borrow()
            .iter()
            .find(|e| (e.0, e.1) == (hash, len))
            .map(|e| e.2)
    }) {
        Some(found) => found,
        None => {
            let error = jaic::parser::parse_file(FileId(0), text)
                .err()
                .map(|d| d.span.start as usize);
            note_parse(text, error);
            error
        }
    };
    LAST_PARSE.with(|last| {
        let mut last = last.borrow_mut();
        last.0.clear();
        last.0.push_str(text);
        last.1 = error;
    });
    error
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_results_are_remembered_by_text() {
        let good = "main :: () { x := 1; }\n";
        let bad = "main :: () { x := ; }\n";
        assert_eq!(parse_error(good), None);
        let at = parse_error(bad);
        assert!(at.is_some());
        // Both are answered from memory now, and more texts than are kept push the oldest out.
        assert_eq!(parse_error(good), None);
        assert_eq!(parse_error(bad), at);
        for n in 0..=PARSES_KEPT {
            assert_eq!(parse_error(&format!("f{n} :: () {{}}\n")), None);
        }
        assert_eq!(parse_error(bad), at);
    }
}
