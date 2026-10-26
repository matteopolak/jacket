//! Compile one root with lint facts, or for the platform-leak pass.
use super::Options;
use super::plan::canonical;
use super::rooted_fs::RootedFs;
use jaic::build::{SharedWorkspaces, WorkspaceObserver};
use jaic::ide::IdeRoot;
use jaic::ide::lint_compile::{LintCompile, LintRoot, LintSetup};
use jaic::interp::SandboxHost;
use jaic::paths::CanonCache;
use jaic::project::{Program, Root};
use jaic::sema::{Compiler, FileSystem, ProgramSource, TargetOs};
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::rc::Rc;

/// Compilers of finished workspaces, with whether each failed.
pub(super) type Finished = Rc<RefCell<Vec<(Box<Compiler>, bool)>>>;

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
pub(super) struct Compiled {
    pub(super) compiler: Box<Compiler>,
    pub(super) workspaces: SharedWorkspaces,
    pub(super) done: Finished,
    pub(super) reports: Rc<RefCell<Vec<String>>>,
    /// The first error of the root's own compile.
    pub(super) error: Option<String>,
}

/// Compile `root` for `os` (the options' own when `None`; the compiler picks the CPU). With a
/// `report` set the compile records lint facts for those files; without one it records every
/// failing `#assert` instead of stopping at the first and records no lint facts.
pub(super) fn compile_root(
    program: &Program,
    options: &Options,
    os: Option<TargetOs>,
    report: Option<&BTreeSet<PathBuf>>,
    canon: &CanonCache,
) -> Compiled {
    let root = &program.root;
    let main_dir = root.main_dir();
    let mut copts = options.compiler_options.clone();
    jaic::target::select_os_cpu(&mut copts, os, None);
    // RootedFs resolves program-generated relative paths from the source root. Import
    // directories are invocation configuration, so anchor them before that rebasing.
    // Also anchor nonexistent roots: they must not accidentally find a source-local shadow.
    let mut anchored = program.clone();
    for path in &mut anchored.import_dirs {
        *path = canon.get_or_insert_with(path, canonical);
    }
    copts.use_program(&jaic::sema::NativeFs, &anchored);
    let fs: Rc<dyn FileSystem> = Rc::new(RootedFs {
        resolved: CanonCache::new(),
        base: main_dir.clone(),
        virtual_source: root
            .module_stub()
            .map(|(path, text)| (path, text.into_bytes())),
    });
    let roots: Vec<IdeRoot> =
        report.map_or_else(Vec::new, |r| r.iter().map(IdeRoot::under).collect());
    let lint = report.is_some();
    let done = Rc::new(RefCell::new(Vec::new()));
    let reports = Rc::new(RefCell::new(Vec::<String>::new()));
    let sink = reports.clone();
    let mut started = LintCompile::start(LintSetup {
        fs: fs.clone(),
        options: copts,
        host: Rc::new(RefCell::new(SandboxHost::with_files(
            fs.clone(),
            &main_dir.to_string_lossy(),
        ))),
        observer: Some(Box::new(Collect {
            roots: roots.clone(),
            lint,
            done: done.clone(),
        })),
        report: Box::new(move |_, text| sink.borrow_mut().push(text.to_string())),
    });
    if lint {
        crate::facts::enable(&mut started.compiler, roots);
    } else {
        started.compiler.keep_going_past_asserts();
    }
    let extra: Vec<_> = options
        .add_sources
        .iter()
        .cloned()
        .map(ProgramSource::String)
        .collect();
    let stub = root.module_stub().map(|(path, _)| path);
    let lint_root = match (root, &stub) {
        (
            Root::Module {
                entry, ..
            },
            Some(stub),
        ) => LintRoot::Module {
            entry,
            stub,
        },
        (root, _) => LintRoot::Program(root.path()),
    };
    let result = started.compile(lint_root, &extra);
    Compiled {
        compiler: started.compiler,
        workspaces: started.workspaces,
        done,
        reports,
        error: result.err().map(|e| e.message.clone()),
    }
}
