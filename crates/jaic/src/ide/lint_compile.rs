//! The compile the language server and the linter both run to get facts about a root: the
//! compiler over a file system, a sandbox host for compile-time code, the registry of workspaces
//! a metaprogram creates, and the limits that stop a `#run` that never ends and a check that
//! takes too long. Building it here,
//! once, is what makes a file lint the same in the editor and on the command line; the tools add
//! only what is theirs (their observer of the workspaces, their facts) before compiling.
use crate::build::{BuildEnv, ReportSink, SharedWorkspaces, WorkspaceObserver, Workspaces};
use crate::interp::{SandboxHost, SharedHost};
use crate::sema::{Compiler, FileSystem, Options, ProgramSource};
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

/// What bounds a check. The person who opens a project sets these (the editor's settings, the
/// linter's command line), never the project: a project must not be able to raise them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CheckLimits {
    /// Basic blocks one top-level compile-time evaluation may run: a `#run` may loop forever.
    pub eval_steps: u64,
    /// Seconds the whole check may take before it stops with partial results; `None`: no limit.
    pub seconds: Option<f64>,
    /// Evaluations that ran at least this many blocks are worth showing the cost of.
    pub hint_blocks: u64,
}

impl CheckLimits {
    /// The blocks one evaluation may run unless told otherwise.
    pub const DEFAULT_EVAL_STEPS: u64 = 100_000_000;

    /// The seconds a check may take unless told otherwise.
    pub const DEFAULT_SECONDS: f64 = 10.0;

    /// The variable that replaces `DEFAULT_SECONDS` (`0`: no limit). CI sets it to `0`: shared
    /// runners are too slow and uneven for a wall-clock limit to mean anything there.
    pub const SECONDS_VARIABLE: &str = "JACKET_CHECK_TIME_LIMIT";

    /// The seconds a check may take when no flag or setting says: `SECONDS_VARIABLE` when it
    /// holds a number, `DEFAULT_SECONDS` otherwise.
    pub fn default_seconds() -> f64 {
        std::env::var(Self::SECONDS_VARIABLE)
            .ok()
            .and_then(|value| value.trim().parse().ok())
            .unwrap_or(Self::DEFAULT_SECONDS)
    }

    /// The blocks from which an evaluation shows its cost unless told otherwise.
    pub const DEFAULT_HINT_BLOCKS: u64 = 1_000_000;

    /// `blocks` per evaluation (`0`: no limit) and `seconds` per check (`0` or less, or not a
    /// number: no limit); the cost threshold stays at its default.
    pub fn new(blocks: u64, seconds: f64) -> Self {
        Self {
            eval_steps: if blocks == 0 {
                u64::MAX
            } else {
                blocks
            },
            seconds: (seconds > 0.0 && seconds.is_finite()).then_some(seconds),
            ..Self::default()
        }
    }
}

impl Default for CheckLimits {
    fn default() -> Self {
        Self {
            eval_steps: Self::DEFAULT_EVAL_STEPS,
            seconds: Some(Self::default_seconds()).filter(|s| *s > 0.0 && s.is_finite()),
            hint_blocks: Self::DEFAULT_HINT_BLOCKS,
        }
    }
}

impl CheckLimits {
    /// The notice for a check that hit its wall-clock limit, at `span`.
    pub fn stopped_notice(&self, span: crate::source::Span) -> crate::source::Diagnostic {
        let seconds = self.seconds.unwrap_or_default();
        crate::source::Diagnostic::warning(
            span,
            format!("checking stopped after {seconds} s; results are partial"),
        )
        .with_help("raise the check time limit to check for longer")
    }
}

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
    pub limits: CheckLimits,
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
    /// What bounds the compile.
    pub limits: CheckLimits,
}

impl LintCompile {
    pub fn start(setup: LintSetup) -> Self {
        let LintSetup {
            fs,
            options,
            host,
            observer,
            report,
            limits,
        } = setup;
        let mut options = options;
        options.eval_step_limit = Some(limits.eval_steps);
        if let Some(seconds) = limits.seconds {
            host.borrow_mut()
                .stop_after(std::time::Duration::from_secs_f64(seconds));
        }
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
        compiler.ct.interp.block_budget = Some(limits.eval_steps);
        compiler.attach_workspaces(workspaces.clone());
        Self {
            compiler,
            workspaces,
            host,
            limits,
        }
    }

    /// Whether the wall-clock limit passed and the check stopped early: what it found is
    /// partial, and the one notice for it is [`CheckLimits::stopped_notice`].
    pub fn stopped(&self) -> bool {
        self.host.borrow_mut().has_stopped()
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
