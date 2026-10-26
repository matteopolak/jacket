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
//! With `platform_leak` on, each root whose entry file is portable is compiled again for every
//! target operating system ([`leak_pass`]), keeping going past failing `#assert`s.
//!
//! Lints are reported for the listed files and the files listed files `#load`. A file several
//! roots compile is reported from the first root (modules before programs, then by path).
use crate::config::{Config, Configs};
use crate::{Level, Lint, lint_files};
use jaic::build::{BuildEnv, WorkspaceObserver, Workspaces};
use jaic::fxhash::{HashMap, HashSet};
use jaic::ide::IdeRoot;
use jaic::interp::{SandboxHost, SharedHost};
use jaic::paths::CanonCache;
use jaic::project::{
    self, DiskFs, LoadCache, Program, ProgramInputs, ProjectFs, Root, RootOptions, program_for,
    resolve_program,
};
use jaic::sema::{
    Compiler, FileSystem, ImportEdge, NativeFs, Options as CompilerOptions, ProgramSource, TargetOs,
};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::{Condvar, Mutex};

/// Most roots compiled at once when the number is left to `jailint`: each holds a whole
/// compiler, so memory grows with it long before the CPUs run out.
pub const MAX_AUTO_JOBS: usize = 8;

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
    /// Compile this many roots at once; 0 picks one per CPU, at most [`MAX_AUTO_JOBS`].
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
    /// Every `#assert` that failed in a compile, with the imports that led to its file. A normal
    /// compile stops at its first; the `platform_leak` compiles keep going.
    pub failed_asserts: Vec<AssertFailure>,
    /// Notes about the `jacket.toml` files nested below the first root.
    pub config_notes: Vec<String>,
    /// Nested `jacket.toml` files that could not be read; their files used the first root's settings.
    pub config_errors: Vec<String>,
}

/// An `#assert` that failed while compiling `root` for `os`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssertFailure {
    pub root: PathBuf,
    pub os: TargetOs,
    pub path: PathBuf,
    /// Byte range of the assert's condition in `path`.
    pub start: usize,
    pub end: usize,
    /// The compiler's text (`#assert failed: ...`).
    pub message: String,
    /// The condition names `OS` or is `false`: the failure is about the operating system.
    pub about_os: bool,
    /// The `#import` or `#load` statements that led from the root's entry to `path`, outermost
    /// first: each is in the file named, at the byte range given.
    pub chain: Vec<ImportSite>,
}

/// One `#import` or `#load` statement.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImportSite {
    pub path: PathBuf,
    pub start: usize,
    pub end: usize,
}

/// The sites that loaded `file`, outermost first: each file's first request, followed up to a
/// file nothing asked for.
fn import_chain(edges: &[ImportEdge], file: &Path) -> Vec<ImportSite> {
    let mut chain = Vec::new();
    let mut seen: HashSet<&Path> = HashSet::default();
    seen.insert(file);
    let mut at = file;
    while let Some(edge) = edges
        .iter()
        .find(|e| e.to == at && !seen.contains(e.from.as_path()))
    {
        chain.push(ImportSite {
            path: edge.from.clone(),
            start: edge.start,
            end: edge.end,
        });
        seen.insert(&edge.from);
        at = &edge.from;
    }
    chain.reverse();
    chain
}

/// The failed asserts of `compiler`, once each, with their import chains.
fn failures_of(compiler: &Compiler, root: &Path) -> Vec<AssertFailure> {
    let edges = compiler.import_edges();
    let mut seen = HashSet::default();
    compiler
        .failed_asserts()
        .into_iter()
        .filter(|f| seen.insert((f.path.clone(), f.start)))
        .map(|f| AssertFailure {
            root: root.to_path_buf(),
            os: compiler.options.os,
            chain: import_chain(&edges, &f.path),
            path: f.path,
            start: f.start,
            end: f.end,
            message: f.message,
            about_os: f.about_os,
        })
        .collect()
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

fn canonical(path: &Path) -> PathBuf {
    DiskFs.key(path)
}

/// Which root reports each file. A file several roots compile is reported from the first of
/// them (by position in the sorted root list), so a root need not lint a file an earlier one
/// compiled; it only has to know which files those were. Roots run concurrently, so each
/// publishes the files it compiled as soon as it has, and then waits for every earlier root to
/// have done the same.
struct Claims {
    state: Mutex<ClaimState>,
    published: Condvar,
}

struct ClaimState {
    /// The lowest published root that compiled each file.
    owner: HashMap<PathBuf, usize>,
    done: Vec<bool>,
    /// Roots `0..prefix` have all published.
    prefix: usize,
}

impl Claims {
    fn new(roots: usize) -> Claims {
        Claims {
            state: Mutex::new(ClaimState {
                owner: HashMap::default(),
                done: vec![false; roots],
                prefix: 0,
            }),
            published: Condvar::new(),
        }
    }

    /// Record the files root `index` compiled. Publishing twice does nothing.
    fn publish(&self, index: usize, files: impl IntoIterator<Item = PathBuf>) {
        let mut state = self.state.lock().expect("claims lock");
        if state.done[index] {
            return;
        }
        for file in files {
            let slot = state.owner.entry(file).or_insert(index);
            *slot = (*slot).min(index);
        }
        state.done[index] = true;
        while state.prefix < state.done.len() && state.done[state.prefix] {
            state.prefix += 1;
        }
        self.published.notify_all();
    }

    /// Wait until every root before `index` has published, then say which of `files` root
    /// `index` reports.
    fn owned(&self, index: usize, files: &BTreeSet<PathBuf>) -> HashSet<PathBuf> {
        let mut state = self.state.lock().expect("claims lock");
        while state.prefix < index {
            state = self.published.wait(state).expect("claims lock");
        }
        files
            .iter()
            .filter(|file| state.owner.get(*file) == Some(&index))
            .cloned()
            .collect()
    }
}

/// Publishes nothing on behalf of a root that stops before publishing, so the roots after it
/// are not left waiting.
struct Release<'a> {
    claims: &'a Claims,
    index: usize,
}

impl Drop for Release<'_> {
    fn drop(&mut self) {
        self.claims.publish(self.index, std::iter::empty());
    }
}

/// The `.jai` files under `path`, sorted, excluding what the configuration excludes.
fn expand(path: &Path, config: &Configs, out: &mut BTreeSet<PathBuf>) {
    if config.excluded(&canonical(path)) {
        return;
    }
    if path.is_dir() {
        let files = jaic::paths::jai_files_under(path, &mut |p| config.excluded(&canonical(p)));
        out.extend(files.iter().map(|f| canonical(f)));
    } else if path.is_file() {
        out.insert(canonical(path));
    }
}

/// What the import folders of a program rooted at `path` are made of: the `import_path` of the
/// nearest `jacket.toml`, the `-import_dir`s of the run and the standard library.
fn program_inputs(options: &Options, configs: &Configs, path: &Path) -> ProgramInputs {
    ProgramInputs {
        configured: configs.at(path).import_dirs.clone(),
        extra: options.import_dirs.clone(),
        stdlib: options.stdlib.clone(),
    }
}

/// The program the linter compiles `file` as, with its import folders.
fn program_of(options: &Options, configs: &Configs, file: &Path, cache: &mut LoadCache) -> Program {
    let inputs = program_inputs(options, configs, file);
    let mut dirs = inputs.configured.clone();
    dirs.extend(inputs.extra.iter().cloned());
    dirs.push(inputs.stdlib.clone());
    resolve_program(&DiskFs, file, &RootOptions::linter(dirs), &inputs, cache)
}

/// The module search path `file`'s program is compiled with, before the compiler gets it.
pub fn root_import_paths(options: &Options, file: &Path) -> Vec<PathBuf> {
    let configs = Configs::new(&options.config);
    program_of(
        options,
        &configs,
        &canonical(file),
        &mut LoadCache::default(),
    )
    .import_dirs
}

/// Roots to compile and the files to report on.
fn plan(options: &Options, configs: &Configs) -> (Vec<Program>, BTreeSet<PathBuf>) {
    let mut listed = BTreeSet::new();
    for p in &options.paths {
        expand(p, configs, &mut listed);
    }
    let mut loaded_by_listed = BTreeSet::new();
    for f in &listed {
        loaded_by_listed.extend(project::reachable_loads(&DiskFs, f, usize::MAX));
    }
    let mut loader_cache = LoadCache::default();
    let mut roots: Vec<Program> = if let Some(source) = &options.source {
        let source = canonical(source);
        let inputs = program_inputs(options, configs, &source);
        vec![program_for(Root::Program(source), &inputs)]
    } else {
        listed
            .iter()
            .filter(|f| !loaded_by_listed.contains(*f))
            .map(|f| program_of(options, configs, f, &mut loader_cache))
            .collect()
    };
    roots.sort_by(|a, b| a.root.cmp(&b.root));
    roots.dedup_by(|a, b| a.root == b.root);
    let mut report = listed;
    report.extend(loaded_by_listed);
    report.retain(|f| !configs.excluded(f));
    (roots, report)
}

struct RootResult {
    lints: Vec<Lint>,
    texts: BTreeMap<String, Rc<str>>,
    error: Option<String>,
    contexts: Vec<CheckedContext>,
    failures: Vec<AssertFailure>,
}

/// The native file system with relative paths taken from `base` (the root's directory), as if
/// the compiler ran there: metaprograms name files relative to where they are built from.
struct RootedFs {
    base: PathBuf,
    virtual_source: Option<(PathBuf, Vec<u8>)>,
    /// Paths that exist, resolved: the compiler asks about the same few hundred files for
    /// every access check and import.
    resolved: CanonCache,
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
        let path = self.at(path);
        // Not there: the compiler's lexical form, asked for again next time.
        self.resolved
            .canonicalize(&path)
            .unwrap_or_else(|_| NativeFs.canonical(&path))
    }

    fn list_dir(&self, path: &Path) -> Vec<(String, bool)> {
        NativeFs.list_dir(&self.at(path))
    }
}

/// Compilers of finished workspaces, with whether each failed.
type Finished = Rc<RefCell<Vec<(Box<Compiler>, bool)>>>;

/// Collects the compilers of the workspaces a metaprogram creates, with lint facts on.
struct Collect {
    roots: Vec<IdeRoot>,
    /// Record lint facts; otherwise keep going past failing `#assert`s.
    lint: bool,
    done: Finished,
}

impl WorkspaceObserver for Collect {
    fn created(&mut self, compiler: &mut Compiler) {
        if self.lint {
            crate::facts::enable(compiler, self.roots.clone());
        } else {
            compiler.keep_going_past_asserts();
        }
    }

    fn finished(&mut self, compiler: Box<Compiler>, failed: bool) {
        self.done.borrow_mut().push((compiler, failed));
    }
}

/// A root's compiler after compiling, with the workspaces its metaprogram created.
struct Compiled {
    compiler: Box<Compiler>,
    workspaces: Rc<RefCell<Workspaces>>,
    done: Finished,
    reports: Rc<RefCell<Vec<String>>>,
    /// The first error of the root's own compile.
    error: Option<String>,
}

/// Compile `root` for `os` (the options' own when `None`) on x64: the CPU is left out of what a
/// leak check can depend on. With a `report` set the compile
/// records lint facts for those files; without one it records every failing `#assert` instead
/// of stopping at the first and records no lint facts.
fn compile_root(
    program: &Program,
    options: &Options,
    os: Option<TargetOs>,
    report: Option<&BTreeSet<PathBuf>>,
    canon: &CanonCache,
) -> Compiled {
    let root = &program.root;
    let main_dir = root.main_dir();
    let mut copts = options.compiler_options.clone();
    if let Some(os) = os {
        jaic::target::select_os_cpu(&mut copts, Some(os), Some(jaic::sema::TargetCpu::X64));
    }
    copts.import_paths.clone_from(&program.import_dirs);
    // RootedFs resolves program-generated relative paths from the source root. Import
    // directories are invocation configuration, so anchor them before that rebasing.
    // Also anchor nonexistent roots: they must not accidentally find a source-local shadow.
    for path in &mut copts.import_paths {
        *path = canon.get_or_insert_with(path, canonical);
    }
    if copts.preload.is_none() {
        copts.set_stdlib(&jaic::sema::NativeFs, &options.stdlib);
    }
    let fs: Rc<dyn FileSystem> = Rc::new(RootedFs {
        resolved: CanonCache::new(),
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
    let roots: Vec<IdeRoot> =
        report.map_or_else(Vec::new, |r| r.iter().map(IdeRoot::under).collect());
    let lint = report.is_some();
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
        report: Box::new(move |_, text| sink.borrow_mut().push(text.to_string())),
        observer: Some(Box::new(Collect {
            roots: roots.clone(),
            lint,
            done: done.clone(),
        })),
    });
    let mut compiler = Box::new(Compiler::new(copts, fs));
    compiler.ct.interp.host = Box::new(SharedHost(host));
    compiler.ct.interp.block_budget = Some(BLOCK_BUDGET);
    compiler.attach_workspaces(workspaces.clone());
    if lint {
        crate::facts::enable(&mut compiler, roots);
    } else {
        compiler.keep_going_past_asserts();
    }
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
            // Discover bootstrap imports before adding the root import that loads the module:
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
                if !compiler.ns.modules.iter().any(|module| {
                    module
                        .path
                        .as_ref()
                        .is_some_and(|path| canon.get_or_insert_with(path, canonical) == *entry)
                }) {
                    compiler
                        .add_source(&ProgramSource::File(dir.join(".jailint-module-root.jai")))?;
                }
                compiler.settle()?;
                compiler.finish_program()
            })
        }
    };
    Compiled {
        compiler,
        workspaces,
        done,
        reports,
        error: result.err().map(|e| e.message.clone()),
    }
}

/// Compile `root` with lint facts and lint the files in `report` it compiled, following the
/// workspaces its metaprogram creates (`build.jai` compiling the real program).
fn lint_root(
    root: &Program,
    index: usize,
    options: &Options,
    configs: &Configs,
    report: &BTreeSet<PathBuf>,
    canon: &CanonCache,
    claims: &Claims,
) -> RootResult {
    let _release = Release {
        claims,
        index,
    };
    let Compiled {
        mut compiler,
        workspaces,
        done,
        reports,
        error,
    } = compile_root(root, options, None, Some(report), canon);
    let mut error = error;
    let top_complete = error.is_none();
    if top_complete
        && let Err(message) =
            jaic::build::finish_all(&workspaces, &mut compiler.ct.interp.block_budget)
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
    // Check every program first: which files this root compiled settles which it reports.
    struct Checked {
        compiler: Box<Compiler>,
        complete: bool,
        /// Facts cover every requested file, including loads outside the root folder.
        files: Vec<(jaic::source::FileId, PathBuf)>,
    }
    let mut checked = Vec::new();
    let mut failures = Vec::new();
    for (mut compiler, failed) in compilers {
        failures.extend(failures_of(&compiler, root.path()));
        let diagnostics = compiler.ide_check_all();
        let complete = !failed && diagnostics.is_empty();
        if error.is_none()
            && let Some(diagnostic) = diagnostics.first()
        {
            error = Some(diagnostic.message.clone());
        }
        let files = (0..compiler.src.sources.len() as u32)
            .map(jaic::source::FileId)
            .filter_map(|f| {
                let file = compiler.src.sources.get(f).path.as_file()?;
                let path = canon.get_or_insert_with(file, canonical);
                report.contains(&path).then_some((f, path))
            })
            .collect();
        checked.push(Checked {
            compiler,
            complete,
            files,
        });
    }
    let compiled: BTreeSet<PathBuf> = checked
        .iter()
        .flat_map(|c| c.files.iter().map(|(_, path)| path.clone()))
        .collect();
    claims.publish(index, compiled.iter().cloned());
    let owned = claims.owned(index, &compiled);
    let mut lints = Vec::new();
    let mut texts = BTreeMap::new();
    let mut contexts = Vec::new();
    for Checked {
        mut compiler,
        complete,
        files,
    } in checked
    {
        for (f, path) in &files {
            if owned.contains(path) {
                texts.insert(
                    path.to_string_lossy().into_owned(),
                    compiler.src.sources.get(*f).text.clone(),
                );
            }
        }
        contexts.push(CheckedContext {
            root: root.path().to_path_buf(),
            os: compiler.options.os,
            cpu: compiler.options.cpu,
            complete,
            visited_files: files.iter().map(|(_, path)| path.clone()).collect(),
        });
        // Files an earlier root compiled are linted, and reported, from that root.
        let own: Vec<jaic::source::FileId> = files
            .iter()
            .filter(|(_, path)| owned.contains(path))
            .map(|&(f, _)| f)
            .collect();
        if !own.is_empty() {
            let mut found = lint_files(&mut compiler, &own, configs, complete);
            for l in &mut found {
                l.path = canon
                    .get_or_insert_with(Path::new(&l.path), canonical)
                    .to_string_lossy()
                    .into_owned();
            }
            lints.extend(found);
        }
        // Drop the compiler now: they hold the workspace registry, which held them.
        drop(compiler);
    }
    RootResult {
        lints,
        texts,
        error,
        contexts,
        failures,
    }
}

/// A root's lints, file texts and compile error, as they cross threads.
type Sent = (
    Vec<Lint>,
    BTreeMap<String, String>,
    Option<String>,
    Vec<CheckedContext>,
    Vec<AssertFailure>,
);

/// Lint what `options` names.
pub fn run(options: &Options) -> Outcome {
    let configs = Configs::new(&options.config);
    let (roots, report) = plan(options, &configs);
    let mut results: Vec<Option<Sent>> = (0..roots.len()).map(|_| None).collect();
    let next = std::sync::atomic::AtomicUsize::new(0);
    let collected = std::sync::Mutex::new(Vec::new());
    let canon = CanonCache::new();
    let claims = Claims::new(roots.len());
    let wanted = if options.jobs == 0 {
        jaic::pool::cores().min(MAX_AUTO_JOBS)
    } else {
        options.jobs
    };
    let jobs = wanted.min(roots.len().max(1));
    // The compiler's pool runs the checkers (its threads have the compiler's large stack, which
    // checking recurses on); the job count is its size unless something started it already.
    jaic::pool::configure(wanted);
    jaic::pool::get().scope(|s| {
        for _ in 0..jobs {
            let (roots, report, next, collected) = (&roots, &report, &next, &collected);
            let (canon, claims, configs) = (&canon, &claims, &configs);
            s.spawn(move |_| {
                loop {
                    let i = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let Some(root) = roots.get(i) else {
                        break;
                    };
                    if options.verbose {
                        eprintln!("jailint: checking {}", root.path().display());
                    }
                    let r = lint_root(root, i, options, configs, report, canon, claims);
                    // Texts cross threads as `String` (`Rc` cannot).
                    let texts = r
                        .texts
                        .into_iter()
                        .map(|(p, t)| (p, t.to_string()))
                        .collect();
                    collected
                        .lock()
                        .expect("no thread panicked holding the lock")
                        .push((i, r.lints, texts, r.error, r.contexts, r.failures));
                }
            });
        }
    });
    for (i, lints, texts, error, contexts, failures) in collected.into_inner().unwrap_or_default() {
        results[i] = Some((lints, texts, error, contexts, failures));
    }
    let mut lints = Vec::new();
    let mut texts: BTreeMap<String, Rc<str>> = BTreeMap::new();
    let mut incomplete = Vec::new();
    let mut owner: BTreeMap<String, usize> = BTreeMap::new();
    for (i, r) in results.iter().enumerate() {
        let Some((_, t, error, _, _)) = r else {
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
    let mut failed_asserts = Vec::new();
    for (i, r) in results.into_iter().enumerate() {
        let Some((ls, t, _, compiled, failures)) = r else {
            continue;
        };
        contexts.extend(compiled);
        failed_asserts.extend(failures);
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
    leak_pass(
        (&roots, &report),
        options,
        &configs,
        &canon,
        &mut lints,
        &mut texts,
        &mut failed_asserts,
    );
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
        failed_asserts,
        config_notes: configs.take_notes(),
        config_errors: configs.take_errors(),
    }
}

/// What one portable root's compile for one system found.
type LeakRun = (usize, TargetOs, Vec<ImportEdge>, Vec<AssertFailure>);

/// `platform_leak`: compile each root whose entry file is portable once per target system, with
/// failing `#assert`s recorded rather than stopping the compile, and report the imports (or
/// files) that portable code should not have, as [`rules::platform_leak`] explains.
fn leak_pass(
    (roots, report): (&[Program], &BTreeSet<PathBuf>),
    options: &Options,
    configs: &Configs,
    canon: &CanonCache,
    lints: &mut Vec<Lint>,
    texts: &mut BTreeMap<String, Rc<str>>,
    failed_asserts: &mut Vec<AssertFailure>,
) {
    use crate::rules::platform_leak::{Leak, Probe, analyze};
    let config = &options.config;
    let Some(rule) = crate::RULES.iter().find(|r| r.name == "platform_leak") else {
        return;
    };
    if config.portability.portable.is_empty()
        || !report
            .iter()
            .any(|f| configs.at(f).level_at(rule, f) != Level::Allow)
    {
        return;
    }
    let probe = Probe::new(config);
    let mut jobs: Vec<(usize, TargetOs)> = Vec::new();
    for (i, root) in roots.iter().enumerate() {
        let entry = root.path();
        if configs.at(entry).level_at(rule, entry) == Level::Allow {
            continue;
        }
        for &os in config.portability.targets() {
            if probe.portable(entry, os) {
                jobs.push((i, os));
            }
        }
    }
    let next = std::sync::atomic::AtomicUsize::new(0);
    let collected: Mutex<Vec<LeakRun>> = Mutex::new(Vec::new());
    let wanted = if options.jobs == 0 {
        jaic::pool::cores().min(MAX_AUTO_JOBS)
    } else {
        options.jobs
    };
    let workers = wanted.min(jobs.len().max(1));
    jaic::pool::get().scope(|s| {
        for _ in 0..workers {
            let (jobs, next, collected) = (&jobs, &next, &collected);
            s.spawn(move |_| {
                loop {
                    let i = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let Some(&(index, os)) = jobs.get(i) else {
                        break;
                    };
                    let root = &roots[index];
                    if options.verbose {
                        eprintln!(
                            "jailint: checking {} for {}",
                            root.path().display(),
                            jaic::target::os_name(os)
                        );
                    }
                    let Compiled {
                        mut compiler,
                        workspaces,
                        done,
                        error,
                        ..
                    } = compile_root(root, options, Some(os), None, canon);
                    if error.is_none() {
                        let _ = jaic::build::finish_all(
                            &workspaces,
                            &mut compiler.ct.interp.block_budget,
                        );
                    }
                    let mut compilers = std::mem::take(&mut *done.borrow_mut());
                    compilers.push((compiler, false));
                    let mut edges = Vec::new();
                    let mut failures = Vec::new();
                    for (compiler, _) in &compilers {
                        edges.extend(compiler.import_edges());
                        failures.extend(failures_of(compiler, root.path()));
                    }
                    drop(compilers);
                    collected
                        .lock()
                        .expect("no thread panicked holding the lock")
                        .push((index, os, edges, failures));
                }
            });
        }
    });
    let mut runs = collected.into_inner().unwrap_or_default();
    runs.sort_by(|a, b| (a.0, jaic::target::os_name(a.1)).cmp(&(b.0, jaic::target::os_name(b.1))));
    let mut read: BTreeMap<PathBuf, Option<Rc<str>>> = BTreeMap::new();
    let mut text_of = |path: &Path, texts: &BTreeMap<String, Rc<str>>| -> Option<Rc<str>> {
        read.entry(path.to_path_buf())
            .or_insert_with(|| {
                texts
                    .get(path.to_string_lossy().as_ref())
                    .cloned()
                    .or_else(|| std::fs::read_to_string(path).ok().map(Rc::from))
            })
            .clone()
    };
    let mut seen = HashSet::default();
    for (_, os, edges, failures) in runs {
        let asserts: Vec<_> = failures
            .iter()
            .filter(|f| f.about_os)
            .map(|f| jaic::sema::FailedAssert {
                path: f.path.clone(),
                start: f.start,
                end: f.end,
                message: f.message.clone(),
                about_os: true,
            })
            .collect();
        for leak in analyze(&probe, os, &edges, &asserts) {
            let (path, start, assert) = match &leak {
                Leak::Import {
                    from,
                    start,
                    assert,
                    ..
                } => (from, *start, assert),
                Leak::Own {
                    assert,
                } => (&assert.path, assert.start, assert),
            };
            if !report.contains(path)
                || !seen.insert((path.clone(), start, os as u8))
                || configs.at(path).level_at(rule, path) == Level::Allow
            {
                continue;
            }
            let (Some(text), Some(assert_text)) =
                (text_of(path, texts), text_of(&assert.path, texts))
            else {
                continue;
            };
            if allowed_by_comment(rule, &text, start) {
                continue;
            }
            let lint = leak_lint(&leak, os, &probe, configs, rule, &assert_text);
            texts
                .entry(path.to_string_lossy().into_owned())
                .or_insert(text);
            lints.push(lint);
        }
        failed_asserts.extend(failures);
    }
}

/// Whether a `// jailint: allow(platform_leak)` (or the category, or `all`) covers `offset`.
fn allowed_by_comment(rule: &crate::RuleInfo, text: &str, offset: usize) -> bool {
    let file = jaic::source::FileId(0);
    let (tokens, comments, errors) = jaic::lexer::lex_with_trivia(file, text);
    if !errors.is_empty() {
        return false;
    }
    let (ast, tokens) = jaic::parser::parse_tokens(file, text, tokens);
    let Ok(ast) = ast else {
        return false;
    };
    crate::suppress::Suppressions::new(text, &tokens, &comments, &ast).allows_rule(rule, offset)
}

/// The finding for one leak on `os`.
fn leak_lint(
    leak: &crate::rules::platform_leak::Leak,
    os: TargetOs,
    probe: &crate::rules::platform_leak::Probe,
    configs: &Configs,
    rule: &'static crate::RuleInfo,
    assert_text: &str,
) -> Lint {
    use crate::rules::platform_leak::{Leak, display_name, line_col, system_list};
    let os_name = jaic::target::os_name(os);
    let assert = match leak {
        Leak::Import {
            assert, ..
        }
        | Leak::Own {
            assert,
        } => assert,
    };
    let (line, column) = line_col(assert_text, assert.start);
    let shown = assert
        .path
        .strip_prefix(&configs.base().root)
        .unwrap_or(&assert.path);
    let reason = assert
        .message
        .strip_prefix("#assert failed: ")
        .unwrap_or(&assert.message);
    let notes = vec![format!(
        "the `#assert` at {}:{line}:{column} fails on {os_name}: {reason}",
        jaic::paths::slashed(&shown)
    )];
    match leak {
        Leak::Import {
            from,
            start,
            end,
            to,
            ..
        } => Lint {
            rule: rule.name,
            level: configs.at(from).level_at(rule, from),
            path: from.to_string_lossy().into_owned(),
            start: *start,
            end: *end,
            message: format!("{} is not available on {os_name}", display_name(to)),
            label: Some(format!(
                "only builds on {}",
                system_list(&probe.systems(to))
            )),
            help: Some(
                "guard the import with an `#if OS ==` check, or list this file under `[lint.portability.platforms]`"
                    .into(),
            ),
            notes,
            fix: None,
        },
        Leak::Own {
            assert,
        } => Lint {
            rule: rule.name,
            level: configs.at(&assert.path).level_at(rule, &assert.path),
            path: assert.path.to_string_lossy().into_owned(),
            start: assert.start,
            end: assert.end,
            message: format!("{} is not available on {os_name}", display_name(&assert.path)),
            label: Some(format!("fails on {os_name}")),
            help: Some(
                "list the file under `[lint.portability.platforms]` or add `// jailint: platforms(...)` at its top"
                    .into(),
            ),
            notes: Vec::new(),
            fix: None,
        },
    }
}
