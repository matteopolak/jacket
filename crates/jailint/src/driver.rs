//! From paths on the command line to lints: which files to compile as what, and which
//! compiled files to report on.
//!
//! - A directory stands for the `.jai` files under it; a file for itself.
//! - Files another listed file `#load`s are compiled as part of it, not on their own.
//! - A file a module's `module.jai` `#load`s is compiled as part of that module even when only
//!   the file is listed: as a program of its own, its exported procedures would look unused.
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
    pub paths: Vec<PathBuf>,
    /// Extra import directories, searched after the root's `modules/` folder.
    pub import_dirs: Vec<PathBuf>,
    pub stdlib: PathBuf,
    pub config: Config,
    /// Compile this many roots at once.
    pub jobs: usize,
    /// Print each root as it is compiled.
    pub verbose: bool,
}

pub struct Outcome {
    pub lints: Vec<Lint>,
    /// Text of every file a lint points into, by path.
    pub texts: BTreeMap<String, Rc<str>>,
    /// Roots the compiler could not finish, with its first error (their lints may be fewer).
    pub incomplete: Vec<(PathBuf, String)>,
    pub roots: usize,
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
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
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
    let mut roots: Vec<Root> = listed
        .iter()
        .filter(|f| !loaded_by_listed.contains(*f))
        .map(|f| {
            module_of(f, &import_dirs)
                .or_else(|| enclosing_module(f, &import_dirs))
                .unwrap_or_else(|| Root::Program(f.clone()))
        })
        .collect();
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
}

/// The native file system with relative paths taken from `base` (the root's directory), as if
/// the compiler ran there: metaprograms name files relative to where they are built from.
struct RootedFs {
    base: PathBuf,
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
        NativeFs.read(&self.at(path))
    }

    fn is_file(&self, path: &Path) -> bool {
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
    prefix: String,
    done: Finished,
}

impl WorkspaceObserver for Collect {
    fn created(&mut self, compiler: &mut Compiler) {
        compiler.interp.block_budget = Some(BLOCK_BUDGET);
        crate::facts::enable(compiler, vec![self.prefix.clone()]);
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
    let mut copts = CompilerOptions::host();
    copts.import_paths = vec![main_dir.join("modules")];
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
    copts.preload = Some(options.stdlib.join("prelude/module.jai"));
    let fs: Rc<dyn FileSystem> = Rc::new(RootedFs {
        base: main_dir.clone(),
    });
    let host = Rc::new(RefCell::new(SandboxHost::with_files(
        fs.clone(),
        &main_dir.to_string_lossy(),
    )));
    let prefix = match root {
        Root::Module {
            entry, ..
        } if entry.file_name().is_some_and(|n| n == "module.jai") => {
            format!("{}/", entry.parent().unwrap_or(entry).to_string_lossy())
        }
        Root::Module {
            entry, ..
        } => entry.to_string_lossy().into_owned(),
        Root::Program(p) => {
            format!("{}/", p.parent().unwrap_or(p).to_string_lossy())
        }
    };
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
            prefix: prefix.clone(),
            done: done.clone(),
        })),
    });
    let mut compiler = Box::new(Compiler::new(copts, fs));
    compiler.interp.host = Box::new(SharedHost(host));
    compiler.interp.block_budget = Some(BLOCK_BUDGET);
    compiler.attach_workspaces(workspaces.clone());
    crate::facts::enable(&mut compiler, vec![prefix.clone()]);
    let result = match root {
        Root::Program(p) => compiler.compile_program(p),
        Root::Module {
            name, ..
        } => compiler.compile_sources(&[ProgramSource::String(format!(
            "#import \"{name}\";\nmain :: () {{}}\n"
        ))]),
    };
    let mut error = result.err().map(|e| e.message.clone());
    let top_complete = error.is_none();
    if top_complete && let Err(message) = jaic::build::finish_all(&workspaces) {
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
    let mut seen: BTreeSet<PathBuf> = BTreeSet::new();
    for (mut compiler, failed) in compilers {
        compiler.ide_check_all();
        // Facts were recorded for files under the prefix only.
        let files: Vec<jaic::source::FileId> = (0..compiler.sources.len() as u32)
            .map(jaic::source::FileId)
            .filter(|&f| {
                let path = &compiler.sources.get(f).path;
                let path_buf = canonical(Path::new(path));
                path.starts_with(&prefix) && report.contains(&path_buf) && seen.insert(path_buf)
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
        let mut found = lint_files(&mut compiler, &files, &options.config, !failed);
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
    }
}

/// A root's lints, file texts and compile error, as they cross threads.
type Sent = (Vec<Lint>, BTreeMap<String, String>, Option<String>);

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
                            .push((i, r.lints, texts, r.error));
                    }
                })
                .expect("spawn a checking thread");
        }
    });
    for (i, lints, texts, error) in collected.into_inner().unwrap_or_default() {
        results[i] = Some((lints, texts, error));
    }
    let mut lints = Vec::new();
    let mut texts: BTreeMap<String, Rc<str>> = BTreeMap::new();
    let mut incomplete = Vec::new();
    let mut owner: BTreeMap<String, usize> = BTreeMap::new();
    for (i, r) in results.iter().enumerate() {
        let Some((_, t, error)) = r else {
            continue;
        };
        for path in t.keys() {
            owner.entry(path.clone()).or_insert(i);
        }
        if let Some(e) = error {
            incomplete.push((roots[i].path().to_path_buf(), e.clone()));
        }
    }
    for (i, r) in results.into_iter().enumerate() {
        let Some((ls, t, _)) = r else {
            continue;
        };
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
    Outcome {
        lints,
        texts,
        incomplete,
        roots: roots.len(),
    }
}
