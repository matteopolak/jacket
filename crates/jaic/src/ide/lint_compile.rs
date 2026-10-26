//! The compile the language server and the linter both run to get facts about a root: the
//! compiler over a file system, a sandbox host for compile-time code, the registry of workspaces
//! a metaprogram creates, and the budget that stops a `#run` that never ends. Building it here,
//! once, is what makes a file lint the same in the editor and on the command line; the tools add
//! only what is theirs (their observer of the workspaces, their facts) before compiling.
use crate::build::{BuildEnv, ReportSink, SharedWorkspaces, WorkspaceObserver, Workspaces};
use crate::interp::{SandboxHost, SharedHost};
use crate::sema::{Compiler, FileSystem, Options, ProgramSource};
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

/// Basic blocks compile-time code may run per compile: a `#run` may loop forever.
pub const BLOCK_BUDGET: u64 = 200_000_000;

/// What a lint compile starts from.
#[derive(Clone, Copy, Debug)]
pub enum LintRoot<'a> {
    /// A program compiled from this file.
    Program(&'a Path),
    /// A module: its entry file and the stub that imports it
    /// ([`crate::project::module_root_stub`]), which the file system must serve.
    Module { entry: &'a Path, stub: &'a Path },
}

/// A compiler ready to compile one root, with the host and workspaces it shares with the
/// metaprograms it runs.
pub struct LintCompile {
    pub compiler: Box<Compiler>,
    pub workspaces: SharedWorkspaces,
    /// The host of the root's compile-time code and of every workspace's.
    pub host: Rc<RefCell<SandboxHost>>,
}

/// What [`LintCompile::start`] needs.
pub struct LintSetup {
    pub fs: Rc<dyn FileSystem>,
    /// The options of the root's compiler and of the workspaces its metaprogram creates.
    pub options: Options,
    /// The host of the root's compile-time code and of every workspace's.
    pub host: Rc<RefCell<SandboxHost>>,
    /// Sees each workspace compiler as it is made and when it is done.
    pub observer: Option<Box<dyn WorkspaceObserver>>,
    /// Where diagnostics of the workspaces' compilations go.
    pub report: ReportSink,
}

impl LintCompile {
    pub fn start(setup: LintSetup) -> Self {
        let LintSetup {
            fs,
            options,
            host,
            observer,
            report,
        } = setup;
        let workspace_host = host.clone();
        let workspaces = Workspaces::new(BuildEnv {
            unwritten_output_hint: None,
            fs: fs.clone(),
            options: options.clone(),
            backend: None,
            command_line: Vec::new(),
            make_host: Box::new(move |_| Box::new(SharedHost(workspace_host.clone()))),
            report,
            observer,
        });
        let mut compiler = Box::new(Compiler::new(options, fs));
        compiler.ct.interp.host = Box::new(SharedHost(host.clone()));
        compiler.ct.interp.block_budget = Some(BLOCK_BUDGET);
        compiler.attach_workspaces(workspaces.clone());
        Self {
            compiler,
            workspaces,
            host,
        }
    }

    /// Compile `root`, with `extra` sources in the program's scope.
    pub fn compile(&mut self, root: LintRoot, extra: &[ProgramSource]) -> crate::sema::Result<()> {
        match root {
            LintRoot::Program(path) => {
                let sources: Vec<_> = std::iter::once(ProgramSource::File(path.to_path_buf()))
                    .chain(extra.iter().cloned())
                    .collect();
                self.compiler.compile_sources(&sources)
            }
            LintRoot::Module {
                entry,
                stub,
            } => self.compiler.compile_module_root(entry, stub, extra),
        }
    }

    /// Drive the workspaces no message loop drives to their end, as a build does; their
    /// stopping errors are in their own facts. The first error is returned.
    pub fn finish_workspaces(&mut self) -> Result<(), String> {
        crate::build::finish_all(&self.workspaces, &mut self.compiler.ct.interp.block_budget)
    }
}
