//! From paths on the command line to lints: which files to compile as what, and which
//! compiled files to report on.
//!
//! - A directory stands for the `.jai` files under it; a file for itself.
//! - Files another listed file `#load`s are compiled as part of it, not on their own.
//! - A file a module's `module.jai` `#load`s is compiled as part of that module even when only
//!   the file is listed: as a program of its own, its exported procedures would look unused.
//! - A file a nearby program `#load`s (in its directory or up to two above) is compiled as part
//!   of that program too, so listing only `basics.jai` of a program that loads it lints it in
//!   context; only the listed files are reported.
//! - `module.jai` (and a `.jai` file directly in an import directory, such as the stdlib) is
//!   a module: it is compiled by importing it from an empty program, and every procedure in
//!   it is checked, called or not.
//! - Anything else is a program and is compiled from that file.
//!
//! Lints are reported for the listed files and the files listed files `#load`. A file several
//! roots compile is reported from the first root (modules before programs, then by path).
use crate::config::Config;
use crate::{Lint, lint_files};
use jaic::build::{BuildEnv, WorkspaceObserver, Workspaces};
use jaic::interp::{SandboxHost, SharedHost};
use jaic::sema::{Compiler, FileSystem, NativeFs, Options as CompilerOptions, ProgramSource};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// Basic blocks compile-time code may run per root (a `#run` may loop forever).
const BLOCK_BUDGET: u64 = 200_000_000;

pub struct Options {
    /// Compiler source/target/runtime settings, cloned for each root and workspace.
    pub compiler_options: CompilerOptions,
    /// Optional actual enclosing program supplying imports, module arguments and source origin.
    /// `paths` still determines which files receive lint reports.
    pub source: Option<PathBuf>,
    pub paths: Vec<PathBuf>,
    /// Extra import directories, searched after the root's `modules/` folder.
    pub import_dirs: Vec<PathBuf>,
    pub stdlib: PathBuf,
    pub config: Config,
    /// Compile this many roots at once.
    pub jobs: usize,
    /// Print each root as it is compiled.
    pub verbose: bool,
    /// Additional source strings, in the program's scope (same as jaic -add).
    pub add_sources: Vec<String>,
}

pub struct Outcome {
    pub lints: Vec<Lint>,
    /// Text of every file a lint points into, by path.
    pub texts: BTreeMap<String, Rc<str>>,
    /// Roots the compiler could not finish, with its first error (their lints may be fewer).
    pub incomplete: Vec<(PathBuf, String)>,
    pub roots: usize,
    /// Actual compiler contexts, including child workspaces and failed body checks.
    pub contexts: Vec<CheckedContext>,
    /// Requested/load-discovered files absent from every compiled context (e.g. inactive loads).
    pub unvisited_files: Vec<PathBuf>,
}

#[derive(Clone, Debug)]
pub struct CheckedContext {
    pub root: PathBuf,
    pub os: jaic::sema::TargetOs,
    pub cpu: jaic::sema::TargetCpu,
    pub complete: bool,
    /// Source files visited by linting; complete=false never establishes full body coverage.
    pub visited_files: Vec<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Root {
    /// A module: its entry file, the name to import it by and the directory holding it.
    Module {
        entry: PathBuf,
        name: String,
        dir: PathBuf,
    },
    Program(PathBuf),
}

impl Root {
    fn path(&self) -> &Path {
        match self {
            Root::Module {
                entry, ..
            } => entry,
            Root::Program(p) => p,
        }
    }
}

fn canonical(path: &Path) -> PathBuf {
    std::fs::canonicalize(path)
        .or_else(|_| std::path::absolute(path))
        .unwrap_or_else(|_| path.to_path_buf())
}

/// The `.jai` files under `path`, sorted, excluding what the configuration excludes.
fn expand(path: &Path, config: &Config, out: &mut BTreeSet<PathBuf>) {
    if config.excluded(&canonical(path)) {
        return;
    }
    if path.is_dir() {
        let Ok(entries) = std::fs::read_dir(path) else {
            return;
        };
        let mut children: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
        children.sort();
        for child in children {
            let hidden = child
                .file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with('.'));
            if !hidden && (child.is_dir() || child.extension().is_some_and(|e| e == "jai")) {
                expand(&child, config, out);
            }
        }
    } else if path.is_file() {
        out.insert(canonical(path));
    }
}

/// Files `file` `#load`s, directly or not (whatever `#if` they sit in).
fn loads(file: &Path, seen: &mut BTreeSet<PathBuf>) {
    let Ok(text) = std::fs::read_to_string(file) else {
        return;
    };
    let Ok(tokens) = jaic::lexer::lex(jaic::source::FileId(0), &text) else {
        return;
    };
    let dir = file.parent().unwrap_or(Path::new("."));
    for w in tokens.windows(2) {
        if let (jaic::lexer::Tok::Directive(d), jaic::lexer::Tok::Str(s)) = (&w[0].tok, &w[1].tok)
            && d.as_str() == "load"
        {
            let target = canonical(&dir.join(String::from_utf8_lossy(s).as_ref()));
            if target.is_file() && seen.insert(target.clone()) {
                loads(&target, seen);
            }
        }
    }
}

/// The module `file` is the entry of, if it is one: `module.jai`, or a file directly in an
/// import directory.
fn module_of(file: &Path, import_dirs: &[PathBuf]) -> Option<Root> {
    let mut dirs: Vec<_> = import_dirs.iter().map(|dir| canonical(dir)).collect();
    dirs.sort_by_key(|dir| std::cmp::Reverse(dir.components().count()));
    for dir in dirs {
        if let Ok(relative) = file.strip_prefix(&dir) {
            let name = if relative
                .file_name()
                .is_some_and(|name| name == "module.jai")
            {
                relative.parent()?.to_path_buf()
            } else {
                relative.with_extension("")
            };
            return Some(Root::Module {
                entry: file.to_path_buf(),
                name: name.to_string_lossy().replace('\\', "/"),
                dir,
            });
        }
    }
    let dir = file.parent()?;
    if file.file_name().is_some_and(|n| n == "module.jai") {
        return Some(Root::Module {
            entry: file.to_path_buf(),
            name: dir.file_name()?.to_string_lossy().into_owned(),
            dir: dir.parent()?.to_path_buf(),
        });
    }
    let in_import_dir = import_dirs.iter().any(|d| canonical(d) == dir)
        || dir.file_name().is_some_and(|n| n == "modules");
    in_import_dir.then(|| Root::Module {
        entry: file.to_path_buf(),
        name: file
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned(),
        dir: dir.to_path_buf(),
    })
}

/// For a file that is not a module entry: the module whose `module.jai` (in the file's directory
/// or one above it) `#load`s it.
fn enclosing_module(file: &Path, import_dirs: &[PathBuf]) -> Option<Root> {
    for dir in file.parent()?.ancestors() {
        let entry = dir.join("module.jai");
        if !entry.is_file() {
            continue;
        }
        let entry = canonical(&entry);
        let mut loaded = BTreeSet::new();
        loads(&entry, &mut loaded);
        return if loaded.contains(file) {
            module_of(&entry, import_dirs)
        } else {
            None
        };
    }
    None
}

/// How far above a file `enclosing_program` looks for a program that loads it.
const LOADER_SEARCH_LEVELS: usize = 3;

/// For a file that is neither a module nor loaded by a listed file: the program that `#load`s it,
/// searching the `.jai` files in its directory and the directories above (a few levels). Of the
/// files that load it, the one no other loader loads is the root; ties go to the first path.
fn enclosing_program(
    file: &Path,
    cache: &mut BTreeMap<PathBuf, BTreeSet<PathBuf>>,
) -> Option<Root> {
    let mut loaders: Vec<PathBuf> = Vec::new();
    for dir in file.parent()?.ancestors().take(LOADER_SEARCH_LEVELS) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };
        let mut candidates: Vec<PathBuf> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|e| e == "jai") && p.is_file())
            .map(|p| canonical(&p))
            .filter(|p| p != file)
            .collect();
        candidates.sort();
        for c in candidates {
            let loaded = cache.entry(c.clone()).or_insert_with(|| {
                let mut set = BTreeSet::new();
                loads(&c, &mut set);
                set
            });
            if loaded.contains(file) {
                loaders.push(c);
            }
        }
    }
    let root = loaders
        .iter()
        .find(|l| {
            !loaders
                .iter()
                .any(|o| o != *l && cache.get(o).is_some_and(|set| set.contains(*l)))
        })
        .or(loaders.first())?;
    Some(Root::Program(root.clone()))
}

/// Roots to compile and the files to report on.
fn plan(options: &Options) -> (Vec<Root>, BTreeSet<PathBuf>) {
    let mut listed = BTreeSet::new();
    for p in &options.paths {
        expand(p, &options.config, &mut listed);
    }
    let mut loaded_by_listed = BTreeSet::new();
    for f in &listed {
        loads(f, &mut loaded_by_listed);
    }
    let mut import_dirs = options.import_dirs.clone();
    import_dirs.push(options.stdlib.clone());
    let mut loader_cache = BTreeMap::new();
    let mut roots: Vec<Root> = if let Some(source) = &options.source {
        vec![Root::Program(canonical(source))]
    } else {
        listed
            .iter()
            .filter(|f| !loaded_by_listed.contains(*f))
            .map(|f| {
                enclosing_module(f, &import_dirs)
                    .or_else(|| module_of(f, &import_dirs))
                    .or_else(|| enclosing_program(f, &mut loader_cache))
                    .unwrap_or_else(|| Root::Program(f.clone()))
            })
            .collect()
    };
    roots.sort();
    roots.dedup();
    let mut report = listed;
    report.extend(loaded_by_listed);
    report.retain(|f| !options.config.excluded(f));
    (roots, report)
}

struct RootResult {
    lints: Vec<Lint>,
    texts: BTreeMap<String, Rc<str>>,
    error: Option<String>,
    contexts: Vec<CheckedContext>,
}

/// The native file system with relative paths taken from `base` (the root's directory), as if
/// the compiler ran there: metaprograms name files relative to where they are built from.
struct RootedFs {
    base: PathBuf,
    virtual_source: Option<(PathBuf, Vec<u8>)>,
}

impl RootedFs {
    fn at(&self, path: &Path) -> PathBuf {
        if path.is_relative() {
            self.base.join(path)
        } else {
            path.to_path_buf()
        }
    }
}

impl FileSystem for RootedFs {
    fn read(&self, path: &Path) -> Option<Vec<u8>> {
        if let Some((source, text)) = &self.virtual_source
            && self.at(path) == *source
        {
            return Some(text.clone());
        }
        NativeFs.read(&self.at(path))
    }

    fn is_file(&self, path: &Path) -> bool {
        if self
            .virtual_source
            .as_ref()
            .is_some_and(|(source, _)| self.at(path) == *source)
        {
            return true;
        }
        NativeFs.is_file(&self.at(path))
    }

    fn is_dir(&self, path: &Path) -> bool {
        NativeFs.is_dir(&self.at(path))
    }

    fn canonical(&self, path: &Path) -> PathBuf {
        NativeFs.canonical(&self.at(path))
    }

    fn list_dir(&self, path: &Path) -> Vec<(String, bool)> {
        NativeFs.list_dir(&self.at(path))
    }
}

/// Compilers of finished workspaces, with whether each failed.
type Finished = Rc<RefCell<Vec<(Box<Compiler>, bool)>>>;

/// Collects the compilers of the workspaces a metaprogram creates, with lint facts on.
struct Collect {
    prefixes: Vec<String>,
    done: Finished,
}

impl WorkspaceObserver for Collect {
    fn created(&mut self, compiler: &mut Compiler) {
        crate::facts::enable(compiler, self.prefixes.clone());
    }

    fn finished(&mut self, compiler: Box<Compiler>, failed: bool) {
        self.done.borrow_mut().push((compiler, failed));
    }
}

/// Compile `root` with lint facts and lint the files in `report` it compiled, following the
/// workspaces its metaprogram creates (`build.jai` compiling the real program).
fn lint_root(root: &Root, options: &Options, report: &BTreeSet<PathBuf>) -> RootResult {
    let main_dir = match root {
        Root::Module {
            dir, ..
        } => dir.clone(),
        Root::Program(p) => p.parent().map(Path::to_path_buf).unwrap_or_default(),
    };
    let mut copts = options.compiler_options.clone();
    copts.import_paths.insert(0, main_dir.join("modules"));
    if let Root::Module {
        dir, ..
    } = root
    {
        copts.import_paths.push(dir.clone());
    }
    copts
        .import_paths
        .extend(options.import_dirs.iter().cloned());
    copts.import_paths.push(options.stdlib.clone());
    // RootedFs resolves program-generated relative paths from the source root. Import
    // directories are invocation configuration, so anchor them before that rebasing.
    // Also anchor nonexistent roots: they must not accidentally find a source-local shadow.
    for path in &mut copts.import_paths {
        *path = canonical(path);
    }
    copts
        .preload
        .get_or_insert_with(|| options.stdlib.join(jaic::PRELOAD_ENTRY));
    let fs: Rc<dyn FileSystem> = Rc::new(RootedFs {
        base: main_dir.clone(),
        virtual_source: match root {
            Root::Module {
                name,
                dir,
                ..
            } => Some((
                dir.join(".jailint-module-root.jai"),
                format!("#import \"{name}\";\n").into_bytes(),
            )),
            Root::Program(_) => None,
        },
    });
    let host = Rc::new(RefCell::new(SandboxHost::with_files(
        fs.clone(),
        &main_dir.to_string_lossy(),
    )));
    let prefixes: Vec<String> = report
        .iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect();
    let done = Rc::new(RefCell::new(Vec::new()));
    let reports = Rc::new(RefCell::new(Vec::<String>::new()));
    let workspace_host = host.clone();
    let sink = reports.clone();
    let workspaces = Workspaces::new(BuildEnv {
        unwritten_output_hint: None,
        fs: fs.clone(),
        options: copts.clone(),
        backend: None,
        command_line: Vec::new(),
        make_host: Box::new(move |_| Box::new(SharedHost(workspace_host.clone()))),
        report: Box::new(move |text| sink.borrow_mut().push(text.to_string())),
        observer: Some(Box::new(Collect {
            prefixes: prefixes.clone(),
            done: done.clone(),
        })),
    });
    let mut compiler = Box::new(Compiler::new(copts, fs));
    compiler.interp.host = Box::new(SharedHost(host));
    compiler.interp.block_budget = Some(BLOCK_BUDGET);
    compiler.attach_workspaces(workspaces.clone());
    crate::facts::enable(&mut compiler, prefixes);
    let result = match root {
        Root::Program(path) => {
            let sources: Vec<_> = std::iter::once(ProgramSource::File(path.clone()))
                .chain(
                    options
                        .add_sources
                        .iter()
                        .cloned()
                        .map(ProgramSource::String),
                )
                .collect();
            compiler.compile_sources(&sources)
        }
        Root::Module {
            entry,
            dir,
            ..
        } => {
            // Discover bootstrap imports before supplying a fallback root import:
            // their dependencies may carry required parameters. Do not execute
            // top-level code or finish an empty program first: either can freeze
            // Context before the requested module registers its extensions.
            let sources: Vec<_> = std::iter::once(ProgramSource::String("main :: () {}".into()))
                .chain(
                    options
                        .add_sources
                        .iter()
                        .cloned()
                        .map(ProgramSource::String),
                )
                .collect();
            compiler.prepare_sources(&sources).and_then(|()| {
                compiler.expand_all()?;
                if !compiler.modules.iter().any(|module| {
                    module
                        .path
                        .as_ref()
                        .is_some_and(|path| canonical(path) == *entry)
                }) {
                    compiler
                        .add_source(&ProgramSource::File(dir.join(".jailint-module-root.jai")))?;
                }
                compiler.settle()?;
                compiler.finish_program()
            })
        }
    };
    let mut error = result.err().map(|e| e.message.clone());
    let top_complete = error.is_none();
    if top_complete
        && let Err(message) =
            jaic::build::finish_all(&workspaces, &mut compiler.interp.block_budget)
    {
        error = Some(message);
    }
    // The workspaces' programs first: a file both compile is the target program's.
    let mut compilers: Vec<(Box<Compiler>, bool)> = std::mem::take(&mut *done.borrow_mut());
    if error.is_none()
        && let Some((_, true)) = compilers.iter().find(|(_, failed)| *failed)
    {
        error = reports
            .borrow()
            .first()
            .map(|r| r.lines().next().unwrap_or_default().to_string())
            .or_else(|| Some("a workspace failed to compile".into()));
    }
    compilers.reverse();
    compilers.push((compiler, !top_complete));
    let mut lints = Vec::new();
    let mut texts = BTreeMap::new();
    let mut contexts = Vec::new();
    for (mut compiler, failed) in compilers {
        let diagnostics = compiler.ide_check_all();
        let complete = !failed && diagnostics.is_empty();
        if error.is_none()
            && let Some(diagnostic) = diagnostics.first()
        {
            error = Some(diagnostic.message.clone());
        }
        // Facts cover every requested file, including loads outside the root folder.
        let files: Vec<jaic::source::FileId> = (0..compiler.sources.len() as u32)
            .map(jaic::source::FileId)
            .filter(|&f| {
                let path = &compiler.sources.get(f).path;
                let path_buf = canonical(Path::new(path));
                report.contains(&path_buf)
            })
            .collect();
        for &f in &files {
            let src = compiler.sources.get(f);
            texts.insert(
                canonical(Path::new(&src.path))
                    .to_string_lossy()
                    .into_owned(),
                src.text.clone(),
            );
        }
        contexts.push(CheckedContext {
            root: root.path().to_path_buf(),
            os: compiler.options.os,
            cpu: compiler.options.cpu,
            complete,
            visited_files: files
                .iter()
                .map(|&file| canonical(Path::new(&compiler.sources.get(file).path)))
                .collect(),
        });
        let mut found = lint_files(&mut compiler, &files, &options.config, complete);
        for l in &mut found {
            l.path = canonical(Path::new(&l.path)).to_string_lossy().into_owned();
        }
        lints.extend(found);
        // Drop the compiler now: they hold the workspace registry, which held them.
        drop(compiler);
    }
    RootResult {
        lints,
        texts,
        error,
        contexts,
    }
}

/// A root's lints, file texts and compile error, as they cross threads.
type Sent = (
    Vec<Lint>,
    BTreeMap<String, String>,
    Option<String>,
    Vec<CheckedContext>,
);

/// Lint what `options` names.
pub fn run(options: &Options) -> Outcome {
    let (roots, report) = plan(options);
    let mut results: Vec<Option<Sent>> = (0..roots.len()).map(|_| None).collect();
    let next = std::sync::atomic::AtomicUsize::new(0);
    let collected = std::sync::Mutex::new(Vec::new());
    let jobs = options.jobs.max(1).min(roots.len().max(1));
    std::thread::scope(|s| {
        for _ in 0..jobs {
            let (roots, report, next, collected) = (&roots, &report, &next, &collected);
            // Checking recurses on the syntax tree: give it the compiler's stack.
            std::thread::Builder::new()
                .stack_size(1 << 30)
                .spawn_scoped(s, move || {
                    loop {
                        let i = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        let Some(root) = roots.get(i) else {
                            break;
                        };
                        if options.verbose {
                            eprintln!("jailint: checking {}", root.path().display());
                        }
                        let r = lint_root(root, options, report);
                        // Texts cross threads as `String` (`Rc` cannot).
                        let texts = r
                            .texts
                            .into_iter()
                            .map(|(p, t)| (p, t.to_string()))
                            .collect();
                        collected
                            .lock()
                            .expect("no thread panicked holding the lock")
                            .push((i, r.lints, texts, r.error, r.contexts));
                    }
                })
                .expect("spawn a checking thread");
        }
    });
    for (i, lints, texts, error, contexts) in collected.into_inner().unwrap_or_default() {
        results[i] = Some((lints, texts, error, contexts));
    }
    let mut lints = Vec::new();
    let mut texts: BTreeMap<String, Rc<str>> = BTreeMap::new();
    let mut incomplete = Vec::new();
    let mut owner: BTreeMap<String, usize> = BTreeMap::new();
    for (i, r) in results.iter().enumerate() {
        let Some((_, t, error, _)) = r else {
            continue;
        };
        for path in t.keys() {
            owner.entry(path.clone()).or_insert(i);
        }
        if let Some(e) = error {
            incomplete.push((roots[i].path().to_path_buf(), e.clone()));
        }
    }
    let mut contexts = Vec::new();
    for (i, r) in results.into_iter().enumerate() {
        let Some((ls, t, _, compiled)) = r else {
            continue;
        };
        contexts.extend(compiled);
        for l in ls {
            if owner.get(&l.path) == Some(&i) {
                lints.push(l);
            }
        }
        for (p, text) in t {
            if owner.get(&p) == Some(&i) {
                texts.entry(p).or_insert_with(|| Rc::from(text));
            }
        }
    }
    lints.sort_by(|a, b| (&a.path, a.start, a.rule).cmp(&(&b.path, b.start, b.rule)));
    lints.dedup_by(|a, b| {
        a.path == b.path
            && a.start == b.start
            && a.end == b.end
            && a.rule == b.rule
            && a.message == b.message
    });
    let visited: BTreeSet<_> = contexts
        .iter()
        .flat_map(|context| context.visited_files.iter().cloned())
        .collect();
    let unvisited_files: Vec<_> = report.difference(&visited).cloned().collect();
    if incomplete.is_empty()
        && let Some(source) = &options.source
    {
        for path in options.paths.iter().filter(|path| path.is_file()) {
            let path = canonical(path);
            if unvisited_files.contains(&path) {
                incomplete.push((source.clone(), format!("context did not visit requested file `{}`; choose its actual enclosing program and target", path.display())));
            }
        }
    }
    Outcome {
        lints,
        texts,
        incomplete,
        roots: roots.len(),
        contexts,
        unvisited_files,
    }
}
