//! Whole-program compilation: load Preload, Runtime_Support and the main
//! file, run top-level directives, and lower everything reachable from the
//! exported entry points.
use super::scope::Resolved;
use super::*;
use std::path::Path;

/// One input of a program: a file on the import file system or source text.
#[derive(Clone, Debug)]
pub enum ProgramSource {
    File(std::path::PathBuf),
    String(String),
    /// Source text added to a module of the workspace (`add_build_string` with a message).
    ModuleString(String, u32),
}

impl Compiler {
    /// Load the bootstrap modules (Preload, Runtime_Support).
    pub fn load_bootstrap(&mut self) -> Result<()> {
        let span = Span::default();
        if let Some(preload) = self.options.preload.clone() {
            let m = self.load_module("Preload", &preload, Vec::new(), span)?;
            self.preload = Some(m);
        }
        if self.options.runtime_support {
            let Some(entry) = self.find_module("Runtime_Support", Path::new("")) else {
                return err(span, "Runtime_Support module not found on the import path");
            };
            let params = vec![
                (
                    Sym::intern("DEFINE_SYSTEM_ENTRY_POINT"),
                    Value::Bool(true),
                    TypeId::BOOL,
                ),
                (
                    Sym::intern("DEFINE_INITIALIZATION"),
                    Value::Bool(true),
                    TypeId::BOOL,
                ),
                (
                    Sym::intern("ENABLE_BACKTRACE_ON_CRASH"),
                    Value::Bool(false),
                    TypeId::BOOL,
                ),
                (
                    Sym::intern("TEMPORARY_STORAGE_SIZE"),
                    Value::Int(self.options.temporary_storage_size as i128),
                    TypeId::S64,
                ),
            ];
            let m = self.load_module("Runtime_Support", &entry, params, span)?;
            self.runtime_support = Some(m);
        }
        Ok(())
    }

    /// Compile a program whose main file is `path`.
    pub fn compile_program(&mut self, path: &Path) -> Result<()> {
        self.compile_sources(&[ProgramSource::File(path.to_path_buf())])
    }

    /// Compile a program made of several files and source strings, all loaded
    /// into the main module (a workspace's `add_build_file`/`add_build_string`).
    pub fn compile_sources(&mut self, sources: &[ProgramSource]) -> Result<()> {
        self.begin_sources(sources)?;
        self.finish_program()
    }

    /// Load the bootstrap modules and `sources`, then run every top-level
    /// directive: the program is parsed and its compile-time code has run.
    pub fn begin_sources(&mut self, sources: &[ProgramSource]) -> Result<()> {
        let result = self.begin_sources_inner(sources);
        self.with_deferred_errors(result)
    }

    /// A failure while top-level items still wait for a retry may be a consequence of
    /// one of them (a missing import leaves a name undefined): show why they failed.
    fn with_deferred_errors<T>(&mut self, result: Result<T>) -> Result<T> {
        result.map_err(|mut e| {
            for d in std::mem::take(&mut self.deferred_errors) {
                e.notes.push((
                    d.span,
                    format!("a top-level item failed earlier: {}", d.message),
                ));
                e.notes.extend(d.notes);
            }
            e
        })
    }

    fn begin_sources_inner(&mut self, sources: &[ProgramSource]) -> Result<()> {
        self.load_bootstrap()?;
        let m = self.new_module("main", None, Vec::new());
        self.main_module = Some(m);
        for source in sources {
            self.add_source(source)?;
        }
        self.settle()
    }

    /// Load one more file or string into the main module; `settle` runs it.
    pub fn add_source(&mut self, source: &ProgramSource) -> Result<()> {
        let Some(m) = self.main_module else {
            return err(Span::default(), "no main module to add sources to");
        };
        self.added_sources += 1;
        match source {
            ProgramSource::File(path) => self.load_file(path, m, Span::default()),
            ProgramSource::String(text) => {
                let label = format!("<added string {}>", self.added_sources);
                self.load_string(&label, text, m)
            }
            ProgramSource::ModuleString(text, module) => {
                let label = format!("<added string {}>", self.added_sources);
                self.load_string(&label, text, ModuleId(*module))
            }
        }
    }

    /// Expand declarations and run pending top-level directives.
    pub fn settle(&mut self) -> Result<()> {
        self.expand_all()?;
        self.apply_pokes()?;
        self.run_top_level()
    }

    /// Load sources that compile-time code added to this compiler's own
    /// workspace (`add_build_string(s)` from inside a `#run`).
    pub(super) fn pull_workspace_sources(&mut self) -> Result<()> {
        let Some(workspaces) = self.interp.workspaces.clone() else {
            return Ok(());
        };
        let added = crate::build::take_own_sources(&workspaces, self.workspace);
        if added.is_empty() {
            return Ok(());
        }
        for source in &added {
            self.add_source(source)?;
        }
        self.expand_all()
    }

    /// Lower everything reachable from the program's exports.
    pub fn finish_program(&mut self) -> Result<()> {
        let result = self.finish_program_inner();
        self.with_deferred_errors(result)
    }

    fn finish_program_inner(&mut self) -> Result<()> {
        // A program made only of `#run`/`#assert` directives has nothing to lower.
        let Some(m) = self.main_module else {
            return Ok(());
        };
        let scope = self.modules[m.0 as usize].scope;
        if self.lookup(scope, Sym::intern("main"))?.is_empty() {
            return Ok(());
        }
        let mut i = 0;
        while i < self.export_entities.len() {
            let e = self.export_entities[i];
            self.resolve_entity(e)?;
            i += 1;
        }
        let mut i = 0;
        while i < self.exports.len() {
            let p = self.exports[i];
            self.proc_func(p, self.proc(p).span)?;
            i += 1;
        }
        self.drain_bodies()?;
        self.fill_runtime_info();
        Ok(())
    }

    /// The `main` procedure of the main module.
    pub fn program_main(&mut self, span: Span) -> Result<ProcId> {
        let Some(m) = self.main_module else {
            return err(span, "no main module");
        };
        let scope = self.modules[m.0 as usize].scope;
        let ids = self.lookup(scope, Sym::intern("main"))?;
        for id in ids {
            match self.resolve_entity(id)? {
                Resolved::Proc(p) => return Ok(p),
                Resolved::Const {
                    value: Value::Proc(p),
                    ..
                } => return Ok(p),
                _ => {}
            }
        }
        err(span, "the program has no 'main :: () { ... }' procedure")
    }

    /// The exported IR function named `name` (e.g. `"main"`).
    pub fn exported_func(&self, name: &str) -> Option<ir::FuncId> {
        self.exports
            .iter()
            .find(|&&p| self.proc(p).export.as_deref() == Some(name))
            .and_then(|&p| match self.proc(p).target {
                Some(procs::ProcTarget::Func(f)) => Some(f),
                _ => None,
            })
    }

    /// Run the compiled program in the interpreter; returns its exit code.
    pub fn run_program(&mut self) -> Result<i32> {
        let Some(main) = self.exported_func("main") else {
            if self.exports.is_empty() {
                // Compile-time-only program: everything already ran.
                return Ok(0);
            }
            return err(
                Span::default(),
                "no exported 'main' (is Runtime_Support loaded?)",
            );
        };
        self.interp.compile_time = false;
        let result = self.interp.call(&self.program, main, &[0, 0]);
        match result {
            Ok(values) => Ok(values.first().map_or(0, |&v| v as u32 as i32)),
            Err(trap) => {
                let mut msg = format!("runtime error: {}", trap.message);
                if let Some((file, line, col)) = trap.loc {
                    msg = format!(
                        "{}:{line}:{col}: {msg}",
                        self.sources.get(FileId(file)).path
                    );
                }
                err(Span::default(), msg)
            }
        }
    }
}
