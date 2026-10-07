//! Whole-program compilation: load Preload, Runtime_Support and the main
//! file, run top-level directives, and lower everything reachable from the
//! exported entry points.
use super::scope::{EntityId, EntityKind, EntityState, Resolved, ScopeKind};
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

/// How many functions, globals and foreign symbols the program had at one point.
#[derive(Clone, Copy, Debug)]
pub struct ProgramMark {
    funcs: usize,
    globals: usize,
    foreigns: usize,
}

impl Compiler {
    /// Load the bootstrap modules (Preload, Runtime_Support).
    pub fn load_bootstrap(&mut self) -> Result<()> {
        let span = Span::NONE;
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
            return err(Span::NONE, "no main module to add sources to");
        };
        self.added_sources += 1;
        match source {
            ProgramSource::File(path) => self.load_file(path, m, Span::NONE),
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
        // Last: the runtime information above lists only what the program reaches.
        self.check_unreferenced()
    }

    /// Type-check the declarations that nothing the program reaches names but that
    /// `Build_Options.dead_code_elimination` still wants checked: by default everything declared
    /// in the program's own files (globals, constants, struct constants, nested and top-level
    /// procedure bodies), with `.NONE` the modules' too. Polymorphic procedures and macros wait
    /// for a use, as always. Declarations and bodies are checked in creation order, so the first
    /// error reported does not depend on hashing. The code this lowers is dropped from compiled
    /// output unless reachable code uses it (`drop_unreferenced_code`).
    fn check_unreferenced(&mut self) -> Result<()> {
        // The program may have set its own workspace's options while it compiled.
        if let Some(workspaces) = &self.interp.workspaces
            && let Some(mode) = crate::build::dead_code_setting(workspaces, self.workspace)
        {
            self.options.dead_code = mode;
        }
        if self.options.dead_code == DeadCode::All {
            return Ok(());
        }
        self.unreferenced_from.get_or_insert(ProgramMark {
            funcs: self.program.funcs.len(),
            globals: self.program.globals.len(),
            foreigns: self.program.foreigns.len(),
        });
        // Checking creates more of both (a body's nested procedures, the declarations of an
        // `#insert`): go on until a round finds nothing new.
        let (mut entity_index, mut proc_index) = (0, 0);
        while entity_index < self.entities.len() || proc_index < self.procs.len() {
            while entity_index < self.entities.len() {
                let id = EntityId(entity_index as u32);
                entity_index += 1;
                if self.unreferenced_entity_wanted(id) {
                    // A struct constant may name the struct's members (`size_of(type_of(info))`),
                    // which are only known while or after the struct is laid out; a use of the
                    // struct lays it out first, so do the same here.
                    let scope = self.entity(id).scope;
                    if let ScopeKind::Struct(t) = self.scope(scope).kind
                        && let TypeKind::Struct(s) = self.types.kind(t).clone()
                    {
                        let span = self.entity(id).span;
                        self.layout_struct(s, span)?;
                    }
                    self.resolve_entity(id)?;
                }
            }
            while proc_index < self.procs.len() {
                let id = ProcId(proc_index as u32);
                proc_index += 1;
                self.check_unreferenced_proc(id)?;
            }
        }
        Ok(())
    }

    /// Whether unreferenced declarations of `module` are type-checked.
    fn checks_unreferenced_in(&self, module: ModuleId) -> bool {
        match self.options.dead_code {
            DeadCode::None => true,
            DeadCode::ModulesOnly => Some(module) == self.main_module,
            DeadCode::All => false,
        }
    }

    fn unreferenced_entity_wanted(&self, id: EntityId) -> bool {
        let e = self.entity(id);
        let EntityKind::Decl {
            decl, ..
        } = &e.kind
        else {
            return false;
        };
        if !matches!(e.state, EntityState::Unresolved) {
            return false;
        }
        let scope = self.scope(e.scope);
        if !self.checks_unreferenced_in(scope.module) || self.inside_instance(e.scope) {
            return false;
        }
        let is_const = decl.kind == ast::DeclKind::Const;
        match scope.kind {
            // Globals and constants alike.
            ScopeKind::Module | ScopeKind::File => true,
            // Struct constants; members are checked by laying the struct out.
            ScopeKind::Struct(_) => is_const,
            // A procedure declared in a body; the body's other constants are checked where used.
            ScopeKind::Proc | ScopeKind::Block => {
                is_const
                    && matches!(
                        decl.value.as_ref().map(|v| &v.kind),
                        Some(ast::ExprKind::Proc(_))
                    )
            }
            ScopeKind::Root | ScopeKind::StructParams | ScopeKind::Macro => false,
        }
    }

    /// Is `scope` part of a polymorphic instance (a struct's or a procedure's) or of a macro
    /// expansion? What is declared there is checked as that instance is used.
    fn inside_instance(&self, mut scope: ScopeId) -> bool {
        loop {
            let s = self.scope(scope);
            match s.kind {
                ScopeKind::Macro | ScopeKind::StructParams => return true,
                ScopeKind::Struct(t) => {
                    if let TypeKind::Struct(st) = self.types.kind(t)
                        && self.types.struct_info(*st).poly_parent.is_some()
                    {
                        return true;
                    }
                }
                ScopeKind::Proc => {
                    if s.proc.is_some_and(|p| self.proc(p).bindings.is_some()) {
                        return true;
                    }
                }
                ScopeKind::Module | ScopeKind::File | ScopeKind::Root => return false,
                ScopeKind::Block => {}
            }
            match s.parent {
                Some(parent) => scope = parent,
                None => return false,
            }
        }
    }

    /// Check one procedure nothing called: its header, and its body unless it is polymorphic
    /// or a macro.
    fn check_unreferenced_proc(&mut self, id: ProcId) -> Result<()> {
        let p = self.proc(id);
        if p.is_macro
            || p.bindings.is_some()
            || p.target.is_some()
            || p.body_state != procs::BodyState::NotNeeded
            || !self.checks_unreferenced_in(self.scope(p.scope).module)
            || self.inside_instance(p.scope)
        {
            return Ok(());
        }
        let span = p.span;
        self.refresh_implicit_poly(id)?;
        if self.proc(id).is_poly {
            return Ok(());
        }
        if self.proc(id).lit.body.is_none() {
            // `#foreign`, `#compiler`, `#intrinsic`: only the types. Reserving a target would
            // add a foreign symbol (and link its library).
            self.signature(id, span)?;
            return Ok(());
        }
        self.proc_func(id, span)?;
        self.drain_bodies()
    }

    /// Before writing output: what `check_unreferenced` lowered is not part of the program unless
    /// code from before it reaches it. Such functions are dropped, such globals keep only their
    /// size (they are zero-filled and point at nothing), and such foreign symbols no longer pull
    /// in their libraries. Code from before the mark cannot name newer functions except through
    /// globals, which compile-time code may have written since (`#no_reset`), so the walk
    /// starts from all older functions and globals.
    fn drop_unreferenced_code(&mut self) {
        let Some(mark) = self.unreferenced_from.take() else {
            return;
        };
        let program = &mut self.program;
        let mut funcs: Vec<bool> = (0..program.funcs.len()).map(|i| i < mark.funcs).collect();
        let mut globals: Vec<bool> = (0..program.globals.len())
            .map(|i| i < mark.globals)
            .collect();
        let mut foreigns: Vec<bool> = (0..program.foreigns.len())
            .map(|i| i < mark.foreigns)
            .collect();
        for (i, f) in program.funcs.iter().enumerate() {
            if f.as_ref()
                .is_some_and(|f| matches!(f.linkage, ir::Linkage::Export(_)))
            {
                funcs[i] = true;
            }
        }
        for (i, g) in program.globals.iter().enumerate() {
            globals[i] |= g.export.is_some();
        }
        // Native code calls the check reporter from its failure paths, not through an `Inst`.
        if let Some(handler) = program.check_failed {
            funcs[handler.0 as usize] = true;
        }
        let mut func_work: Vec<usize> = (0..funcs.len()).filter(|&i| funcs[i]).collect();
        let mut global_work: Vec<usize> = (0..globals.len()).filter(|&i| globals[i]).collect();
        let mark_live = |live: &mut Vec<bool>, work: &mut Vec<usize>, i: usize| {
            if !live[i] {
                live[i] = true;
                work.push(i);
            }
        };
        while !func_work.is_empty() || !global_work.is_empty() {
            while let Some(g) = global_work.pop() {
                for r in &program.globals[g].relocs {
                    match r.target {
                        ir::RelocTarget::Func(f) => {
                            mark_live(&mut funcs, &mut func_work, f.0 as usize)
                        }
                        ir::RelocTarget::Global(h) => {
                            mark_live(&mut globals, &mut global_work, h.0 as usize)
                        }
                        ir::RelocTarget::Foreign(x) => foreigns[x.0 as usize] = true,
                    }
                }
            }
            while let Some(f) = func_work.pop() {
                let Some(func) = &program.funcs[f] else {
                    continue;
                };
                for inst in func.blocks.iter().flat_map(|b| &b.insts) {
                    match inst {
                        ir::Inst::Call(call) => match call.callee {
                            ir::Callee::Func(g) => {
                                mark_live(&mut funcs, &mut func_work, g.0 as usize)
                            }
                            ir::Callee::Foreign(x) => foreigns[x.0 as usize] = true,
                            ir::Callee::Indirect(..) => {}
                        },
                        ir::Inst::FuncAddr {
                            func, ..
                        } => mark_live(&mut funcs, &mut func_work, func.0 as usize),
                        ir::Inst::GlobalAddr {
                            global, ..
                        } => mark_live(&mut globals, &mut global_work, global.0 as usize),
                        ir::Inst::ForeignAddr {
                            foreign, ..
                        } => foreigns[foreign.0 as usize] = true,
                        _ => {}
                    }
                }
            }
        }
        for (f, live) in program.funcs.iter_mut().zip(funcs) {
            if !live {
                *f = None;
            }
        }
        for (g, live) in program.globals.iter_mut().zip(globals) {
            if !live {
                g.init = Vec::new();
                g.relocs = Vec::new();
            }
        }
        for (x, live) in program.foreigns.iter_mut().zip(foreigns) {
            if !live {
                x.library = None;
            }
        }
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
        if self.program.stack_trace.is_some() && self.program.file_paths.len() == self.sources.len()
        {
            return;
        }
        let Some(layout) = self.trace_layout(span) else {
            return;
        };
        self.program.file_paths = (0..self.sources.len())
            .map(|i| self.sources.get(FileId(i as u32)).path.clone())
            .collect();
        self.program.stack_trace = Some(layout);
    }

    /// `context.stack_trace`'s place and its node types' layout, from their declarations;
    /// `None` when they are missing or their fields are not the sizes the nodes are written
    /// with.
    fn trace_layout(&mut self, span: Span) -> Option<ir::TraceLayout> {
        let context = self.context_type(span).ok()?;
        let node = self.preload_type("Stack_Trace_Node", span).ok()?;
        let info = self.preload_type("Stack_Trace_Procedure_Info", span).ok()?;
        let location = self.location_layout(span).ok()?;
        let field = |s: &mut Self, ty: TypeId, name: &str, size: u64| -> Option<u64> {
            let (offset, fty) = s.member_offset(ty, name, span).ok()??;
            (s.size_of(fty, span).ok()? == size).then_some(offset)
        };
        let location_ty = self.preload_type("Source_Code_Location", span).ok()?;
        let (location_at, ty) = self.member_offset(info, "location", span).ok()??;
        if ty != location_ty {
            return None;
        }
        Some(ir::TraceLayout {
            context: field(self, context, "stack_trace", 8)?,
            node: ir::TraceNodeLayout {
                size: self.size_of(node, span).ok()?,
                next: field(self, node, "next", 8)?,
                info: field(self, node, "info", 8)?,
                hash: field(self, node, "hash", 8)?,
                call_depth: field(self, node, "call_depth", 4)?,
                line_number: field(self, node, "line_number", 4)?,
            },
            info: ir::TraceInfoLayout {
                size: self.size_of(info, span).ok()?,
                name: field(self, info, "name", 16)?,
                path: location_at + location.path,
                line: location_at + location.line,
                column: location_at + location.column,
                procedure_address: field(self, info, "procedure_address", 8)?,
            },
        })
    }

    /// Before a backend writes the program: compiled code keeps `context.stack_trace` itself
    /// (`stack_trace::instrument`; the interpreter does it at run time). Idempotent.
    pub fn prepare_compiled_output(&mut self) {
        self.bake_no_reset_globals();
        self.drop_unreferenced_code();
        self.enable_stack_traces(Span::NONE);
        if let Some(layout) = self.program.stack_trace {
            crate::stack_trace::instrument(&mut self.program, &layout);
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
            let Ok(Value::Bytes(agg)) = self.read_aggregate(addr, ty, Span::NONE) else {
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
            // Compile-time-only program: everything already ran. (`#program_export` procedures
            // alone do not make it a program to run.)
            let declares_main = match self.main_module {
                Some(m) => {
                    let scope = self.modules[m.0 as usize].scope;
                    !self.lookup(scope, Sym::intern("main"))?.is_empty()
                }
                None => false,
            };
            if self.exports.is_empty() || !declares_main {
                return Ok(0);
            }
            return err(
                Span::NONE,
                "`main` is declared, but no entry point calls it (is Runtime_Support loaded?)",
            );
        };
        self.interp.compile_time = false;
        self.enable_stack_traces(Span::NONE);
        if let Err(trap) = self.interp.reset_globals(&self.program) {
            return Err(Box::new(self.runtime_error(&trap)));
        }
        let result = self.interp.call(&self.program, main, &[argc, argv]);
        match result {
            Ok(values) => Ok(values.first().map_or(0, |&v| v as u32 as i32)),
            Err(trap) => Err(Box::new(self.runtime_error(&trap))),
        }
    }

    /// A trap that stopped the running program, as a diagnostic of kind `Runtime` (unless the
    /// trap has a more specific kind, such as a foreign procedure the host cannot provide).
    fn runtime_error(&self, trap: &crate::interp::Trap) -> Diagnostic {
        let d = self.trap_diagnostic(trap, "runtime error", None);
        if d.kind == DiagnosticKind::Other {
            d.with_kind(DiagnosticKind::Runtime)
        } else {
            d
        }
    }
}
