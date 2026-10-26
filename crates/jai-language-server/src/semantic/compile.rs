//! Running a compile for an analysis.
use super::overlay::{OverlayFs, Reads};
use super::{BuildFacts, BuiltProgram, Environment, Program};
use jaic::build::WorkspaceObserver;
use jaic::fxhash::HashSet;
use jaic::ide::lint_compile::{CheckLimits, LintCompile, LintRoot, LintSetup};
use jaic::ide::{IdeFacts, IdeRoot};
use jaic::interp::SandboxHost;
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

/// Without the errors that only say compile-time code was not run (an untrusted workspace, or a
/// foreign procedure the editor's sandbox does not provide, which a build runs for real) or could
/// not know its result (a pending or unavailable compile-time service): what it would have
/// produced is unknown, not wrong. Code a `#insert` would have generated
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
        !(e.kind.is_unknown_result()
            || (matches!(e.kind, DiagnosticKind::UnknownIdentifier { .. }) && refused(e.span.file)))
    });
    errors
}

/// Whether `text` uses the directive `#module_parameters` (a mention in a comment or string
/// does not make a file a module).
fn writes_module_parameters(text: &str) -> bool {
    if !text.contains("#module_parameters") {
        return false;
    }
    jaic::lexer::Lexed::new(FileId(0), text).tokens.iter().any(
        |t| matches!(&t.tok, jaic::lexer::Tok::Directive(d) if d.as_str() == "module_parameters"),
    )
}

/// The first line of `root`, or the start of the first file when it was not loaded.
fn first_line(compiler: &Compiler, root: &Path) -> jaic::source::Span {
    let Some(file) = find_file(compiler, root) else {
        return jaic::source::Span::new(FileId(0), 0, 0);
    };
    let text = &compiler.src.sources.get(file).text;
    jaic::source::Span::new(file, 0, text.find('\n').unwrap_or(text.len()))
}

/// What one compile produced.
pub(super) struct Compiled {
    pub(super) compiler: Compiler,
    pub(super) errors: Vec<jaic::source::Diagnostic>,
    /// The programs its metaprogram built.
    pub(super) others: Vec<Program>,
    /// What the metaprogram did, when it ran to the end.
    pub(super) facts: Option<BuildFacts>,
    /// The wall-clock limit passed: what the compile found is partial.
    pub(super) stopped: bool,
    /// The paths it asked about (and later queries ask about).
    pub(super) reads: Rc<RefCell<Reads>>,
}

/// How much a compile is trusted and how far it may run compile-time code.
#[derive(Clone, Copy)]
pub(super) struct Mode {
    pub(super) untrusted: bool,
    pub(super) limits: CheckLimits,
}

/// Compile `root` with `files` over the environment's file system.
pub(super) fn compile(
    env: &Environment,
    root: &Path,
    dirs: &[PathBuf],
    files: BTreeMap<PathBuf, Rc<[u8]>>,
    os: Option<TargetOs>,
    parses: &Rc<RefCell<jaic::parser::ParseReuse>>,
    mode: Mode,
) -> Compiled {
    let Mode {
        untrusted,
        limits,
    } = mode;
    let dir = root.parent().map(Path::to_path_buf).unwrap_or_default();
    let open: Vec<PathBuf> = files.keys().cloned().collect();
    let options = || {
        let mut options = (env.options)(root);
        jaic::project::add_import_dirs(&mut options.import_paths, dirs);
        // The target `jaic -check -os <os>` would use.
        jaic::target::select_os_cpu(&mut options, os, None);
        options
    };
    // A module's `module.jai` is checked as the module a program imports, not as a program:
    // `#module_parameters` and the parameters' values only exist there.
    let mut files = files;
    let declares_parameters = files
        .iter()
        .find(|(path, _)| path.as_path() == jaic::paths::file_identity(&*env.fs, root))
        .is_some_and(|(_, text)| writes_module_parameters(&String::from_utf8_lossy(text)));
    let stub = jaic::project::module_import(
        root,
        &options().import_paths,
        declares_parameters,
        &|path| jaic::paths::file_identity(&*env.fs, path),
    )
    .map(|(import, folder)| {
        let stub = jaic::project::module_root_stub(&folder);
        files.insert(
            jaic::paths::file_identity(&*env.fs, &stub),
            Rc::from(jaic::project::module_root_stub_text(&import).into_bytes()),
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
    // Trusted compile-time code may ask services and read the real environment (a database
    // URL, say); the rest of the sandbox stays.
    let host = Rc::new(RefCell::new(SandboxHost::with_files(
        fs.clone(),
        &dir.to_string_lossy(),
    )));
    if !untrusted {
        let mut host = host.borrow_mut();
        host.services.clone_from(&env.services);
        host.real_environment = true;
    }
    // Spelled like the compiler's source paths, which come through the overlay's keys.
    let dir_key = jaic::paths::file_identity(&*env.fs, &dir);
    // A compile for another target only answers hover and definition: lints come from the
    // environment's own.
    let lint = os.is_none();
    let built = Rc::new(RefCell::new(Vec::new()));
    let added = Rc::new(RefCell::new(BTreeMap::new()));
    // Metaprograms may compile other programs (`compiler_create_workspace`): those are checked
    // the same way, with no output and the same host, and their compile-time code is bounded
    // by the same limits.
    let mut started = LintCompile::start(LintSetup {
        fs: fs.clone(),
        options: options(),
        host: host.clone(),
        observer: Some(Box::new(Built {
            added: added.clone(),
            root: dir_key.clone(),
            open: open.clone(),
            host: host.clone(),
            lint,
            untrusted,
            done: built.clone(),
        })),
        report: Box::new(|_, _| {}),
        limits,
    });
    started
        .compiler
        .ed
        .record(editor_facts(&dir_key, &open, &host, lint, untrusted));
    let number = parses.borrow_mut().begin();
    started.compiler.ed.parse_reuse = Some((parses.clone(), number));
    let lint_root = match &stub {
        Some(stub) => LintRoot::Module {
            entry: root,
            stub,
        },
        None => LintRoot::Program(root),
    };
    let errors = started
        .compile(lint_root, &[])
        .err()
        .into_iter()
        .map(|e| *e)
        .collect::<Vec<_>>();
    // A metaprogram that failed or ran out of budget did not finish what it did: its facts
    // would be a prefix of the real ones.
    let ran_to_the_end = errors.is_empty();
    // Workspaces no message loop drove compile once the metaprogram is done, as in a build.
    // Their stopping error is in their own facts, so the message here adds nothing.
    if errors.is_empty() {
        let _ = started.finish_workspaces();
    }
    let mut compiler = *started.compiler;
    // Workspaces kept for `get_type_table` after they completed go to the observer now.
    if let Some(workspaces) = compiler.ct.interp.workspaces.clone() {
        jaic::build::hand_over_kept(&workspaces);
    }
    let errors = check_rest(&mut compiler, errors);
    let errors = known(&compiler, errors);
    // The wall-clock limit passed: the compile-time code that ran into it said nothing (its
    // result is unknown, not wrong), and this is the one notice that results are partial.
    let stopped = host.borrow_mut().has_stopped();
    if stopped {
        let notice = limits.stopped_notice(first_line(&compiler, root));
        compiler.diag.warnings.push(notice);
    }
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
                            .push(jaic::paths::file_identity(&*env.fs, path));
                    }
                    _ => program.strings += 1,
                }
            }
            program.import_dirs = compiler
                .options
                .import_paths
                .iter()
                .filter(|p| !base_paths.contains(p))
                .map(|p| jaic::paths::file_identity(&*env.fs, p))
                .collect();
            recorded.push((id, program));
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
                enriched: None,
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
    let facts = (ran_to_the_end && !stopped).then(|| BuildFacts {
        programs: recorded.into_iter().map(|(_, p)| p).collect(),
    });
    Compiled {
        compiler,
        errors,
        others,
        facts,
        stopped,
        reads,
    }
}
