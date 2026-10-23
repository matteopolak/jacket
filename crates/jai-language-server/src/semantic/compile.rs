//! Running a compile for an analysis.
use super::overlay::{OverlayFs, Reads};
use super::{BLOCK_BUDGET, BuildFacts, BuiltProgram, Environment, Program};
use jaic::build::{BuildEnv, WorkspaceObserver, Workspaces};
use jaic::fxhash::HashSet;
use jaic::ide::{IdeFacts, IdeRoot};
use jaic::interp::{SandboxHost, SharedHost};
use jaic::sema::{Compiler, FileSystem, TargetOs};
use jaic::source::FileId;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// `path`'s id among the files `compiler` compiled.
pub(super) fn find_file(compiler: &Compiler, path: &Path) -> Option<FileId> {
    let files = || (0..compiler.src.sources.len() as u32).map(FileId);
    files()
        .find(|&f| compiler.src.sources.get(f).path.as_file() == Some(path))
        .or_else(|| {
            let want = compiler.src.fs.canonical(path);
            files().find(|&f| {
                compiler
                    .src
                    .sources
                    .get(f)
                    .path
                    .as_file()
                    .is_some_and(|file| compiler.src.fs.canonical(file) == want)
            })
        })
}

/// The editor facts a compile records for the files under `root`, its compile-time output
/// going to `host`, with what jailint needs (expression types, casts, uses) when `lint`.
fn editor_facts(
    root: &Path,
    open: &[PathBuf],
    host: &Rc<RefCell<SandboxHost>>,
    lint: bool,
    untrusted: bool,
) -> IdeFacts {
    let mut facts = IdeFacts::new(vec![IdeRoot::under(root)]).with_files(open.to_vec());
    facts.output = Some(host.clone());
    facts.lint = lint;
    facts.untrusted = untrusted;
    facts
}

/// Gives each compiler a metaprogram creates editor facts, and keeps it when it is done.
struct Built {
    /// What `add_build_file` / `add_build_string` added, by workspace.
    added: Rc<RefCell<BTreeMap<i64, Vec<jaic::sema::ProgramSource>>>>,
    root: PathBuf,
    open: Vec<PathBuf>,
    host: Rc<RefCell<SandboxHost>>,
    lint: bool,
    untrusted: bool,
    done: Rc<RefCell<Vec<Compiler>>>,
}

impl WorkspaceObserver for Built {
    fn created(&mut self, compiler: &mut Compiler) {
        compiler.ed.record(editor_facts(
            &self.root,
            &self.open,
            &self.host,
            self.lint,
            self.untrusted,
        ));
    }

    fn finished(&mut self, compiler: Box<Compiler>, _failed: bool) {
        self.done.borrow_mut().push(*compiler);
    }

    fn source_added(&mut self, workspace: i64, source: &jaic::sema::ProgramSource) {
        self.added
            .borrow_mut()
            .entry(workspace)
            .or_default()
            .push(source.clone());
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

/// Without the errors that only say compile-time code was not run (an untrusted workspace):
/// what it would have produced is unknown, not wrong. Code a `#insert` would have generated
/// may declare any name, so a file that has one refused cannot report names as unknown.
fn known(
    compiler: &Compiler,
    mut errors: Vec<jaic::source::Diagnostic>,
) -> Vec<jaic::source::Diagnostic> {
    use jaic::source::DiagnosticKind;
    let refused = |file: FileId| {
        compiler
            .ed
            .facts()
            .is_some_and(|ide| ide.refused.iter().any(|s| s.file == file))
            && compiler.src.sources.get(file).text.contains("#insert")
    };
    errors.retain(|e| {
        e.kind != DiagnosticKind::Untrusted
            && !(matches!(e.kind, DiagnosticKind::UnknownIdentifier { .. }) && refused(e.span.file))
    });
    errors
}

/// `root` as `<folder>/Name/module.jai` with `folder` one of `import_paths`: the module's name
/// and that folder.
fn module_entry(
    fs: &dyn FileSystem,
    root: &Path,
    import_paths: &[PathBuf],
) -> Option<(String, PathBuf)> {
    if root.file_name()? != "module.jai" {
        return None;
    }
    let module = root.parent()?;
    let folder = module.parent()?;
    let key = crate::document::file_key(fs, folder);
    import_paths
        .iter()
        .any(|dir| crate::document::file_key(fs, dir) == key)
        .then(|| {
            Some((
                module.file_name()?.to_str()?.to_owned(),
                folder.to_path_buf(),
            ))
        })
        .flatten()
}

/// What one compile produced.
pub(super) struct Compiled {
    pub(super) compiler: Compiler,
    pub(super) errors: Vec<jaic::source::Diagnostic>,
    /// The programs its metaprogram built.
    pub(super) others: Vec<Program>,
    /// What the metaprogram did, when it ran to the end.
    pub(super) facts: Option<BuildFacts>,
    /// The paths it asked about (and later queries ask about).
    pub(super) reads: Rc<RefCell<Reads>>,
}

/// Compile `root` with `files` over the environment's file system.
pub(super) fn compile(
    env: &Environment,
    root: &Path,
    dirs: &[PathBuf],
    files: BTreeMap<PathBuf, Rc<[u8]>>,
    os: Option<TargetOs>,
    parses: &Rc<RefCell<jaic::parser::ParseReuse>>,
    untrusted: bool,
) -> Compiled {
    let dir = root.parent().map(Path::to_path_buf).unwrap_or_default();
    let open: Vec<PathBuf> = files.keys().cloned().collect();
    let options = || {
        let mut options = (env.options)(root);
        crate::modules::splice(&mut options.import_paths, dirs);
        // The target `jaic -check -os <os>` would use.
        jaic::target::select_os_cpu(&mut options, os, None);
        options
    };
    // A module's `module.jai` is checked as the module a program imports, not as a program:
    // `#module_parameters` and the parameters' values only exist there.
    let mut files = files;
    let stub = module_entry(&*env.fs, root, &options().import_paths).map(|(name, folder)| {
        let stub = folder.join(".jailsp-module-root.jai");
        let text = format!("#import \"{name}\";\nmain :: () {{}}\n");
        files.insert(
            crate::document::file_key(&*env.fs, &stub),
            Rc::from(text.into_bytes()),
        );
        stub
    });
    let reads = Rc::new(RefCell::new(Reads::default()));
    let fs: Rc<dyn FileSystem> = Rc::new(OverlayFs {
        base: env.fs.clone(),
        dir: dir.clone(),
        files,
        canonical: jaic::paths::CanonCache::new(),
        reads: reads.clone(),
    });
    let mut compiler = Compiler::new(options(), fs.clone());
    // Compile-time output goes nowhere: stdout may be the protocol channel.
    let host = Rc::new(RefCell::new(SandboxHost::with_files(
        fs,
        &dir.to_string_lossy(),
    )));
    compiler.ct.interp.host = Box::new(SharedHost(host.clone()));
    compiler.ct.interp.block_budget = Some(BLOCK_BUDGET);
    // Spelled like the compiler's source paths, which come through the overlay's keys.
    let dir_key = crate::document::file_key(&*env.fs, &dir);
    // Metaprograms may compile other programs (`compiler_create_workspace`): those are checked
    // the same way, with no output and the same host, and their compile-time code draws from
    // this compiler's budget.
    let workspace_host = host.clone();
    // A compile for another target only answers hover and definition: lints come from the
    // environment's own.
    let lint = os.is_none();
    let built = Rc::new(RefCell::new(Vec::new()));
    let added = Rc::new(RefCell::new(BTreeMap::new()));
    let workspaces = Workspaces::new(BuildEnv {
        unwritten_output_hint: None,
        fs: compiler.src.fs.clone(),
        options: options(),
        backend: None,
        command_line: Vec::new(),
        make_host: Box::new(move |_| Box::new(SharedHost(workspace_host.clone()))),
        report: Box::new(|_, _| {}),
        observer: Some(Box::new(Built {
            added: added.clone(),
            root: dir_key.clone(),
            open: open.clone(),
            host: host.clone(),
            lint,
            untrusted,
            done: built.clone(),
        })),
    });
    compiler.attach_workspaces(workspaces);
    compiler
        .ed
        .record(editor_facts(&dir_key, &open, &host, lint, untrusted));
    let number = parses.borrow_mut().begin();
    compiler.ed.parse_reuse = Some((parses.clone(), number));
    let errors = compiler
        .compile_program(stub.as_deref().unwrap_or(root))
        .err()
        .into_iter()
        .map(|e| *e)
        .collect::<Vec<_>>();
    // A metaprogram that failed or ran out of budget did not finish what it did: its facts
    // would be a prefix of the real ones.
    let ran_to_the_end = errors.is_empty();
    // Workspaces no message loop drove compile once the metaprogram is done, as in a build.
    // Their stopping error is in their own facts, so the message here adds nothing.
    if errors.is_empty()
        && let Some(workspaces) = compiler.ct.interp.workspaces.clone()
    {
        let _ = jaic::build::finish_all(&workspaces, &mut compiler.ct.interp.block_budget);
    }
    let errors = check_rest(&mut compiler, errors);
    let errors = known(&compiler, errors);
    let base_paths = (env.options)(root).import_paths;
    let mut recorded: Vec<(i64, BuiltProgram)> = Vec::new();
    let mut others: Vec<Program> = built
        .take()
        .into_iter()
        .map(|mut compiler| {
            let id = compiler.meta.workspace;
            let mut program = BuiltProgram::default();
            for source in added.borrow().get(&id).into_iter().flatten() {
                match source {
                    jaic::sema::ProgramSource::File(path) => {
                        program
                            .files
                            .push(crate::document::path_key(&env.fs.canonical(path)));
                    }
                    _ => program.strings += 1,
                }
            }
            program.import_dirs = compiler
                .options
                .import_paths
                .iter()
                .filter(|p| !base_paths.contains(p))
                .map(|p| crate::document::path_key(&env.fs.canonical(p)))
                .collect();
            recorded.push((id, program));
            // The metaprogram's code spent the budget; checking the rest gets its own.
            compiler.ct.interp.block_budget = Some(BLOCK_BUDGET);
            let errors = check_rest(&mut compiler, Vec::new());
            let errors = known(&compiler, errors);
            Program {
                compiler,
                // Core does not expose a generated program's selected entry. Never borrow
                // the outer root or infer it from loaded files.
                entry: None,
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
    recorded.sort_by_key(|(id, _)| *id);
    let facts = ran_to_the_end.then(|| BuildFacts {
        programs: recorded.into_iter().map(|(_, p)| p).collect(),
    });
    Compiled {
        compiler,
        errors,
        others,
        facts,
        reads,
    }
}
