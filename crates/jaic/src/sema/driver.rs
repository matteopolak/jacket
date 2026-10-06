//! Whole-program compilation: load Preload, Runtime_Support and the main
//! file, run top-level directives, and lower everything reachable from the
//! exported entry points.
use super::scope::{EntityId, EntityKind, Resolved, ScopeKind};
use super::*;
use crate::types::TypeKind;
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

    /// Lower the procedures reachable from the program's exports, without generating
    /// the program: their bodies may declare more (`#insert,scope(...)`) for a
    /// metaprogram to see before it stops adding code.
    /// Lenient: a body that fails (it may need a `#placeholder` the metaprogram defines
    /// later) stays queued, and `finish_program` reports it if it still fails.
    pub fn lower_reachable(&mut self) {
        let _ = self.lower_reachable_inner(true);
    }

    /// False when there is no program to lower.
    fn lower_reachable_inner(&mut self, lenient: bool) -> Result<bool> {
        // A program made only of `#run`/`#assert` directives has nothing to lower.
        let Some(m) = self.main_module else {
            return Ok(false);
        };
        let scope = self.modules[m.0 as usize].scope;
        if self.lookup(scope, Sym::intern("main"))?.is_empty() {
            return Ok(false);
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
        if lenient {
            // A metaprogram looks for procedures by their notes (`@glsl`, `@thread`) whether
            // or not the program calls them: lower those whose headers it was shown. Only the
            // program's own files count; a noted procedure in an imported module stays unchecked
            // until something calls it (Vk-Engine's `Common` has an uncalled `@PrintLike`
            // procedure whose body names a procedure that does not exist).
            let noted: Vec<ProcId> = (self.export.pending_procs())
                .filter(|&p| {
                    let proc = self.proc(p);
                    Some(self.scope(proc.scope).module) == self.main_module
                        && (!proc.lit.header.notes.is_empty()
                            || self.proc_decl_notes.contains_key(&p))
                })
                .collect();
            for p in noted {
                let _ = self.proc_func(p, self.proc(p).span);
            }
            self.drain_bodies_lenient();
        } else {
            self.drain_bodies()?;
        }
        Ok(true)
    }

    fn finish_program_inner(&mut self) -> Result<()> {
        // No more code is coming: items still waiting for a `#placeholder` fail now.
        self.placeholders_final = true;
        self.settle()?;
        let lowered = self.lower_reachable_inner(false)?;
        self.check_declared_structs()?;
        if lowered {
            self.fill_runtime_info();
        }
        Ok(())
    }

    /// Lay out every top-level struct and union declaration, also those nothing uses.
    /// Jai type-checks all declarations (dead-code elimination only skips procedure
    /// bodies), so a member typed by an undefined name is an error even in a struct the
    /// program never touches; demand-driven checking alone would miss it.
    /// Polymorphic structs are checked per instance, when one is made.
    fn check_declared_structs(&mut self) -> Result<()> {
        let mut index = 0;
        while index < self.entities.len() {
            let id = EntityId(index as u32);
            index += 1;
            let e = self.entity(id);
            if !matches!(
                self.scope(e.scope).kind,
                ScopeKind::Module | ScopeKind::File | ScopeKind::Root
            ) {
                continue;
            }
            let EntityKind::Decl {
                decl, ..
            } = &e.kind
            else {
                continue;
            };
            let is_plain_struct = decl.kind == ast::DeclKind::Const
                && matches!(
                    decl.value.as_ref().map(|v| &v.kind),
                    Some(ast::ExprKind::Struct(lit)) if lit.params.is_empty()
                );
            if !is_plain_struct {
                continue;
            }
            let span = e.span;
            if let Resolved::Const {
                value: Value::Type(t),
                ..
            } = self.resolve_entity(id)?
                && let TypeKind::Struct(s) = self.types.kind(t).clone()
            {
                self.layout_struct(s, span)?;
            }
        }
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

    /// Tell the interpreter where `context.stack_trace` is and how to name source files.
    pub(super) fn enable_stack_traces(&mut self, span: Span) {
        if !self.options.stack_trace {
            return;
        }
        if self.program.stack_trace_offset.is_some()
            && self.program.file_paths.len() == self.sources.len()
        {
            return;
        }
        let Ok(context) = self.context_type(span) else {
            return;
        };
        let Ok(Some((path, _))) = self.find_member(context, Sym::intern("stack_trace"), span)
        else {
            return;
        };
        let offset = path
            .iter()
            .map(|step| {
                if let structs::PathStep::Offset(o) = step {
                    *o
                } else {
                    0
                }
            })
            .sum();
        self.program.file_paths = (0..self.sources.len())
            .map(|i| self.sources.get(FileId(i as u32)).path.clone())
            .collect();
        self.program.stack_trace_offset = Some(offset);
    }

    /// Before a backend writes the program: compiled code keeps `context.stack_trace` itself
    /// (`stack_trace::instrument`; the interpreter does it at run time). Idempotent.
    pub fn prepare_compiled_output(&mut self) {
        self.bake_no_reset_globals();
        self.enable_stack_traces(Span::default());
        if let Some(offset) = self.program.stack_trace_offset {
            crate::stack_trace::instrument(&mut self.program, offset);
        }
        if self.options.debug_info {
            self.collect_debug_types();
        }
    }

    /// A `#no_reset` global keeps what compile-time code stored in it: its initializer becomes
    /// the interpreter's current bytes, with pointers frozen into relocations the way `#run`
    /// results are. Globals no compile-time code touched keep their declared initializer.
    fn bake_no_reset_globals(&mut self) {
        for (global, ty) in std::mem::take(&mut self.no_reset_globals) {
            let Some(addr) = self.interp.materialized_global(global) else {
                continue;
            };
            let Ok(Value::Bytes(agg)) = self.read_aggregate(addr, ty, Span::default()) else {
                continue;
            };
            let g = &mut self.program.globals[global.0 as usize];
            g.init = agg.bytes.clone();
            g.relocs = agg.relocs.clone();
        }
    }

    /// Run the compiled program in the interpreter; returns its exit code.
    pub fn run_program(&mut self) -> Result<i32> {
        self.run_program_with_args(&[])
    }

    /// Run the compiled program with a C-style `argc`/`argv` (what
    /// `get_command_line_arguments` returns). `args[0]` is conventionally the program name;
    /// an empty slice passes `argc = 0, argv = null`. The strings live for the whole process.
    pub fn run_program_with_args(&mut self, args: &[String]) -> Result<i32> {
        let (argc, argv) = if args.is_empty() {
            (0, 0)
        } else {
            let pointers: Vec<u64> = args
                .iter()
                .map(|arg| {
                    let mut bytes = arg.as_bytes().to_vec();
                    bytes.push(0);
                    Box::leak(bytes.into_boxed_slice()).as_ptr() as u64
                })
                .chain(std::iter::once(0))
                .collect();
            let argv = Box::leak(pointers.into_boxed_slice()).as_ptr() as u64;
            (args.len() as u64, argv)
        };
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
        self.enable_stack_traces(Span::default());
        if let Err(trap) = self.interp.reset_globals(&self.program) {
            return err(Span::default(), format!("runtime error: {}", trap.message));
        }
        let result = self.interp.call(&self.program, main, &[argc, argv]);
        match result {
            Ok(values) => Ok(values.first().map_or(0, |&v| v as u32 as i32)),
            Err(trap) => {
                let span = trap.loc.map_or(Span::default(), |(file, line, col)| {
                    let at = self.sources.get(FileId(file)).offset_of(line, col) as usize;
                    Span::new(FileId(file), at, at + 1)
                });
                err(span, format!("runtime error: {}", trap.message))
            }
        }
    }
}
