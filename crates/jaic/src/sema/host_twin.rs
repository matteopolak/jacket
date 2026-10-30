//! The host twin: a second compiler, with the host `OS` and `CPU`, that runs plain `#run` code
//! when this one cross-compiles. See `docs/compiler/compile-time-host-execution.md`.
use super::lower::Operand;
use super::scope::ScopeKind;
use super::{
    Compiler, Diagnostic, FileInfo, HashSet, ModuleId, Options, ProgramSource, Result, ScopeId,
    Span, TypeId, ast, ir,
};

/// Where a cross build keeps its twin.
#[derive(Default)]
pub(crate) struct TwinSlot {
    compiler: Option<Box<Compiler>>,
    /// The twin could not be built (or the program does not load on the host); plain `#run`
    /// then executes on the target compiler.
    unavailable: bool,
    /// How many files of the source table were searched for `#no_reset`, and whether one has it.
    scanned: u32,
    no_reset_declared: bool,
}

/// `Compiler`-module procedures that read or change state the target program owns; a run that
/// names one stays on the target compiler (`Operation::needs_target` catches the calls a
/// procedure makes on the way).
const TARGET_STATE_OPERATIONS: [&str; 5] = [
    "compiler_set_type_info_flags",
    "add_data_segment",
    "add_global_data",
    "get_runtime_info",
    "get_type_table",
];

/// Decides from the type of a run expression whether it runs; see `check_run_gated`.
pub(super) type RunGate<'a> = dyn FnMut(&mut Compiler, TypeId) -> bool + 'a;

/// `JAIC_NO_HOST_TWIN=1` turns the twin off, for timing and bisecting.
fn disabled_by_environment() -> bool {
    static DISABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *DISABLED.get_or_init(|| {
        jacket_config::env::NO_HOST_TWIN
            .os()
            .is_some_and(|v| v != "0")
    })
}

/// The identifier-like words of `text`.
fn words(text: &str) -> HashSet<&str> {
    text.split(|c: char| !(c.is_alphanumeric() || c == '_'))
        .filter(|word| !word.is_empty())
        .collect()
}

impl Compiler {
    pub(super) fn check_run_inner_all(
        &mut self,
        scope: ScopeId,
        body: &ast::RunBody,
        expected: Option<TypeId>,
        span: Span,
    ) -> Result<Vec<Operand>> {
        self.check_run_gated(scope, body, expected, span, None)
    }

    /// Like `check_run_inner_all`; `gate` sees the type of a run expression before it runs, and
    /// declining it (`false`) skips the run, returning no operands.
    pub(super) fn check_run_gated(
        &mut self,
        scope: ScopeId,
        body: &ast::RunBody,
        expected: Option<TypeId>,
        span: Span,
        gate: Option<&mut RunGate>,
    ) -> Result<Vec<Operand>> {
        let tscope = self.thunk_scope(scope);
        let file = self.scope_file(scope);
        match body {
            ast::RunBody::Expr(e) => {
                let mut f = self.thunk_ctx(ir::FuncOrigin::Run, file);
                let op = self.check_expr(&mut f, tscope, e, expected)?;
                match op {
                    Operand::Const {
                        ..
                    }
                    | Operand::Type(_)
                    | Operand::Procs(_) => Ok(vec![op]),
                    other => {
                        let ty = other.ty();
                        if let Some(gate) = gate
                            && !gate(self, ty)
                        {
                            return Ok(Vec::new());
                        }
                        let result = self.run_thunk_all(f, other, span);
                        if result.is_err() {
                            self.ed.run_failed(span, ty);
                        }
                        result
                    }
                }
            }
            ast::RunBody::Block(block) => {
                let mut f = self.thunk_ctx(ir::FuncOrigin::Run, file);
                let inner = self.new_block_scope(tscope, block.span);
                self.check_block_stmts(&mut f, inner, &block.stmts)?;
                self.ide_require_parsed_body(block.span)?;
                if !f.b.is_terminated() {
                    self.emit_defers(&mut f, 0, span)?;
                }
                self.run_thunk_all(f, Operand::Void, span)
            }
        }
    }

    fn is_cross_build(&self) -> bool {
        self.options.is_cross_build()
    }

    /// Remember a source the program was given, and hand it to the twin if one exists. Only a
    /// cross build keeps anything.
    pub(super) fn share_source_with_twin(&mut self, source: &ProgramSource) {
        if self.meta.twin || !self.is_cross_build() {
            return;
        }
        // Sources that name target-side ids (`Code`, a file or module of this compiler) have no
        // counterpart in the twin.
        if !matches!(source, ProgramSource::File(_) | ProgramSource::String(_)) {
            return;
        }
        self.src.program_sources.push(source.clone());
        if let Some(twin) = self.ct.twin.compiler.as_mut()
            && twin
                .add_source(source)
                .and_then(|()| twin.expand_all())
                .is_err()
        {
            self.ct.twin.unavailable = true;
        }
    }

    fn host_twin_wanted(&self, flags: &[ast::Ident]) -> bool {
        !self.meta.twin
            && !self.ct.twin.unavailable
            && self.is_cross_build()
            && !self.ed.is_live()
            && !flags.iter().any(|flag| flag.name.as_str() == "host")
            && self.ct.interp.host.native_linking()
            && !disabled_by_environment()
    }

    /// Make the twin: bootstrap modules and the program's declarations in a compiler with the
    /// host `OS` and `CPU`. Top-level code is not settled and no body is lowered.
    fn build_twin(&mut self) -> Result<Box<Compiler>> {
        let host = self.options.host;
        let (os, cpu) = (host.os, host.cpu);
        let mut options: Options = self.options.clone();
        options.os = os;
        options.cpu = cpu;
        options.long_double = host.long_double;
        let mut twin = Box::new(Compiler::new(options, self.src.fs.clone()));
        twin.meta.twin = true;
        twin.meta.workspace = self.meta.workspace;
        // One file table, so a span of either compiler names the same file in both.
        twin.src.sources = self.src.sources.share();
        let workspaces = self.ct.interp.workspaces.clone();
        let provider = match &workspaces {
            Some(shared) => crate::build::host_provider(shared, os),
            None => self.ct.interp.host.isolated_host().ok_or_else(|| {
                Box::new(Diagnostic::error(
                    Span::NONE,
                    "this compiler host provides no host interpreter",
                ))
            })?,
        };
        twin.ct.interp = crate::interp::Interp::new(provider);
        twin.ct.interp.in_host_twin = true;
        if let Some(shared) = workspaces {
            twin.attach_workspaces(shared);
        }
        let sources = self.src.program_sources.clone();
        twin.prepare_sources(&sources)?;
        twin.expand_all()?;
        twin.apply_pokes()?;
        Ok(twin)
    }

    /// Run a plain `#run` in the twin. `None`: this run stays on the target compiler (not a
    /// cross build, or its scope has no counterpart in the host program).
    pub(super) fn run_on_host_twin(
        &mut self,
        scope: ScopeId,
        body: &ast::RunBody,
        flags: &[ast::Ident],
        span: Span,
    ) -> Result<Option<Vec<Operand>>> {
        if !self.host_twin_wanted(flags) || self.runs_need_target(span) {
            return Ok(None);
        }
        let mut twin = match self.ct.twin.compiler.take() {
            Some(twin) => twin,
            None => match self.build_twin() {
                Ok(twin) => twin,
                Err(reason) => {
                    if jacket_config::env::HOST_TWIN_DEBUG.os().is_some() {
                        eprintln!("jaic: no host twin: {}", self.render(&reason));
                    }
                    self.ct.twin.unavailable = true;
                    return Ok(None);
                }
            },
        };
        let result = self.run_in_twin(&mut twin, scope, body, span);
        // The twin ran this evaluation instead of the target compiler: its cost is recorded here.
        self.meta.perf.runs.append(&mut twin.meta.perf.runs);
        if twin.ct.interp.twin_declined {
            // The run needed state of the target program, which the twin holds its own copy
            // of; whatever it wrote there is meaningless. Every later run stays on the target,
            // so none of them sees the twin's copy.
            self.ct.twin.unavailable = true;
            return Ok(None);
        }
        self.ct.twin.compiler = Some(twin);
        result
    }

    /// Whether the run at `span` must execute on this compiler because it works on state the
    /// program being built owns, and the twin would change only its own copy:
    /// - a `#no_reset` global anywhere in the program (compiled output starts from what
    ///   compile-time code stored in it, and later runs read it back);
    /// - a direct call of a `Compiler` operation on the program's type table or global data
    ///   (a call inside a procedure is caught when the twin's interpreter makes it).
    fn runs_need_target(&mut self, span: Span) -> bool {
        let sources = &self.src.sources;
        while (self.ct.twin.scanned as usize) < sources.len() {
            let file = sources.get(crate::source::FileId(self.ct.twin.scanned));
            self.ct.twin.no_reset_declared |= file.text.contains("#no_reset");
            self.ct.twin.scanned += 1;
        }
        if self.ct.twin.no_reset_declared {
            return true;
        }
        let text = self.src.sources.snippet(span);
        words(text)
            .iter()
            .any(|word| TARGET_STATE_OPERATIONS.contains(word))
    }

    fn run_in_twin(
        &mut self,
        twin: &mut Compiler,
        scope: ScopeId,
        body: &ast::RunBody,
        span: Span,
    ) -> Result<Option<Vec<Operand>>> {
        let Some(twin_scope) = self.map_scope_to_twin(twin, scope, span) else {
            return Ok(None);
        };
        let nested = self.meta.perf.running();
        self.ct.interp.lend_budget(&mut twin.ct.interp, nested);
        let before = (
            twin.ct.executions,
            twin.ct.traps,
            twin.epochs.placeholder_misses,
        );
        let mut declined = false;
        let result = {
            let mut gate = |twin: &mut Compiler, ty: TypeId| {
                let importable = self.can_import_host_type(twin, ty, span);
                declined = !importable;
                importable
            };
            twin.check_run_gated(twin_scope, body, None, span, Some(&mut gate))
        };
        if declined {
            // Nothing ran: the target compiler runs it.
            return Ok(None);
        }
        if result.is_err() && twin.ct.executions == before.0 {
            // The run failed to check before anything ran: it names something the host program
            // does not have (a member a module exports only for the target's OS, say). The
            // target compiler checks it against the target and reports what is really wrong.
            return Ok(None);
        }
        self.ct.interp.take_back_budget(&twin.ct.interp);
        // A retry decision must see that code ran, and what it was waiting for.
        self.ct.executions += twin.ct.executions - before.0;
        self.ct.traps += twin.ct.traps - before.1;
        self.epochs.placeholder_misses += twin.epochs.placeholder_misses - before.2;
        self.load_twin_sources(twin)?;
        match result {
            Ok(operands) => self.import_host_results(twin, operands, span).map(Some),
            Err(diagnostic) => Err(diagnostic),
        }
    }

    /// Load the sources the twin's code added to the workspace into the target as well.
    fn load_twin_sources(&mut self, twin: &mut Compiler) -> Result<()> {
        for (source, scope) in std::mem::take(&mut twin.src.twin_outbox) {
            match (&source, scope) {
                (ProgramSource::CodeString(text, _), Some(scope)) => {
                    let Some(own) = self.scope_from_twin(twin, scope) else {
                        return super::err(
                            Span::NONE,
                            "add_build_string: the Code scope has no counterpart in the target program",
                        );
                    };
                    self.src.added += 1;
                    let label = format!("added code string {}", self.src.added);
                    self.load_string_in_scope(&label, text, own)?;
                }
                (ProgramSource::ModuleString(..), _) => {
                    return super::err(
                        Span::NONE,
                        "add_build_string: a module scope cannot be named from a plain #run on the host",
                    );
                }
                _ => self.add_source_local(&source)?,
            }
        }
        self.expand_all()
    }

    /// The target's file scope for a file scope of the twin.
    fn scope_from_twin(&self, twin: &Compiler, scope: ScopeId) -> Option<ScopeId> {
        let info = twin.src.files.iter().find(|file| file.scope == scope)?;
        let host = &twin.ns.modules[info.module.0 as usize];
        let main = twin.ns.main_module == Some(info.module);
        self.src
            .files
            .iter()
            .find(|file| {
                if file.path != info.path {
                    return false;
                }
                if main || self.ns.main_module == Some(file.module) {
                    return main && self.ns.main_module == Some(file.module);
                }
                let own = &self.ns.modules[file.module.0 as usize];
                own.path == host.path && own.params == host.params
            })
            .map(|file| file.scope)
    }

    /// In the twin: take what its code added to the workspace, load it here, and queue it for
    /// the target.
    pub(super) fn pull_sources_for_twin(&mut self) -> Result<bool> {
        let Some(workspaces) = self.ct.interp.workspaces.clone() else {
            return Ok(false);
        };
        let added = crate::build::take_own_sources(&workspaces, self.meta.workspace);
        if added.is_empty() {
            return Ok(false);
        }
        for source in added {
            self.add_source_local(&source)?;
            let scope = match &source {
                ProgramSource::CodeString(_, code) => {
                    self.pr.code_scopes.get(code.0 as usize).copied()
                }
                _ => None,
            };
            self.src.twin_outbox.push((source, scope));
        }
        self.expand_all()?;
        Ok(true)
    }

    /// The twin's scope that stands for `scope`, which must be a file scope or lie inside
    /// procedures whose own names the run does not use.
    fn map_scope_to_twin(
        &mut self,
        twin: &mut Compiler,
        scope: ScopeId,
        span: Span,
    ) -> Option<ScopeId> {
        let text = self.src.sources.snippet(span).to_owned();
        // A `#code` literal in the run names a scope of the target program.
        if text.contains("#code") {
            return None;
        }
        let used = words(&text);
        let mut current = scope;
        loop {
            let found = self.scope(current);
            match found.kind {
                ScopeKind::File => break,
                ScopeKind::Block | ScopeKind::Proc => {
                    if !found.usings.is_empty()
                        || !found.fallbacks.is_empty()
                        || found.inserted_into.is_some()
                        || found.names.keys().any(|name| used.contains(name.as_str()))
                    {
                        return None;
                    }
                    current = found.parent?;
                }
                _ => return None,
            }
        }
        let (path, module) = {
            let info = self.src.files.iter().find(|file| file.scope == current)?;
            (info.path.clone(), info.module)
        };
        let twin_module = self.map_module_to_twin(twin, module)?;
        // A file the twin did not load itself is one the host program does not have: an OS- or
        // CPU-specific file the `#if` chain left out. Loading it would hand the twin the target's
        // libraries and declarations, so the run stays on the target compiler.
        let twin_scope = twin
            .src
            .files
            .iter()
            .find(|file: &&FileInfo| file.module == twin_module && file.path == path)
            .map(|file| file.scope)?;
        // The run may sit in an `#if` branch only the target takes, where the file declares names
        // (an import, a constant) the host program's copy of the file does not have.
        // The file's names live in its scope and, up to the module, in the scopes above it.
        let declared = |compiler: &Compiler, mut scope: ScopeId| -> HashSet<String> {
            let mut names = HashSet::<String>::default();
            loop {
                let found = compiler.scope(scope);
                names.extend(found.names.keys().map(|name| name.as_str().to_owned()));
                names.extend(
                    found
                        .imports
                        .iter()
                        .filter_map(|entry| entry.import.name)
                        .map(|name| name.name.as_str().to_owned()),
                );
                match (found.kind, found.parent) {
                    (ScopeKind::File, Some(parent)) => scope = parent,
                    _ => return names,
                }
            }
        };
        let host_names = declared(twin, twin_scope);
        let missing = declared(self, current)
            .iter()
            .any(|name| used.contains(name.as_str()) && !host_names.contains(name));
        (!missing).then_some(twin_scope)
    }

    /// The twin's instance of `module`: the same entry file with the same arguments.
    fn map_module_to_twin(&mut self, twin: &mut Compiler, module: ModuleId) -> Option<ModuleId> {
        if Some(module) == self.ns.main_module {
            return twin.ns.main_module;
        }
        let found = &self.ns.modules[module.0 as usize];
        let path = found.path.clone()?;
        let params = found.params.clone();
        let name = found.name.clone();
        // An argument that names one of this compiler's types means nothing to the twin.
        let portable = params.iter().all(|(_, value, ty)| {
            ty.0 <= TypeId::POLY_PARAM.0
                && matches!(
                    value,
                    super::Value::Int(_)
                        | super::Value::Bool(_)
                        | super::Value::String(_)
                        | super::Value::Float(..)
                )
        });
        if !portable {
            return None;
        }
        let key = (twin.src.canon(&path), params.clone());
        if let Some(&id) = twin.ns.module_cache.get(&key) {
            return Some(id);
        }
        twin.load_module(&name, &path, params, Span::NONE).ok()
    }
}
