//! The host twin: a second compiler, with the host `OS` and `CPU`, that runs plain `#run` code
//! when this one cross-compiles. See `docs/compiler/compile-time-host-execution.md`.
use super::lower::Operand;
use super::scope::ScopeKind;
use super::{
    Compiler, Diagnostic, FileInfo, HashSet, ModuleId, Options, ProgramSource, Result, ScopeId,
    Span, TypeId, ast, compiler_host,
};
use crate::source::SourcePath;

/// Where a cross build keeps its twin.
#[derive(Default)]
pub(crate) struct TwinSlot {
    compiler: Option<Box<Compiler>>,
    /// The twin could not be built (or the program does not load on the host); plain `#run`
    /// then executes on the target compiler.
    unavailable: bool,
}

/// `JAIC_NO_HOST_TWIN=1` turns the twin off, for timing and bisecting.
fn disabled_by_environment() -> bool {
    static DISABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *DISABLED.get_or_init(|| std::env::var_os("JAIC_NO_HOST_TWIN").is_some_and(|v| v != "0"))
}

/// The identifier-like words of `text`.
fn words(text: &str) -> HashSet<&str> {
    text.split(|c: char| !(c.is_alphanumeric() || c == '_'))
        .filter(|word| !word.is_empty())
        .collect()
}

impl Compiler {
    fn is_cross_build(&self) -> bool {
        (self.options.os, self.options.cpu) != compiler_host()
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
        let (os, cpu) = compiler_host();
        let mut options: Options = self.options.clone();
        options.os = os;
        options.cpu = cpu;
        options.long_double = Options::host().long_double;
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
        if !self.host_twin_wanted(flags) {
            return Ok(None);
        }
        let mut twin = match self.ct.twin.compiler.take() {
            Some(twin) => twin,
            None => match self.build_twin() {
                Ok(twin) => twin,
                Err(reason) => {
                    if std::env::var_os("JAIC_HOST_TWIN_DEBUG").is_some() {
                        eprintln!("jaic: no host twin: {}", self.render(&reason));
                    }
                    self.ct.twin.unavailable = true;
                    return Ok(None);
                }
            },
        };
        let result = self.run_in_twin(&mut twin, scope, body, span);
        self.ct.twin.compiler = Some(twin);
        result
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
        twin.ct.interp.block_budget = self.ct.interp.block_budget;
        let before = (
            twin.ct.executions,
            twin.ct.traps,
            twin.epochs.placeholder_misses,
        );
        let result = twin.check_run_inner_all(twin_scope, body, None, span);
        self.ct.interp.block_budget = twin.ct.interp.block_budget;
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
        let find = |twin: &Compiler| {
            twin.src
                .files
                .iter()
                .find(|file: &&FileInfo| file.module == twin_module && file.path == path)
                .map(|file| file.scope)
        };
        if let Some(found) = find(twin) {
            return Some(found);
        }
        let SourcePath::File(file) = &path else {
            return None;
        };
        twin.load_file(file, twin_module, Span::NONE).ok()?;
        find(twin)
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
