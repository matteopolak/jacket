//! Reading source files, loading modules and strings, and creating modules.
use super::filesystem::find_module_in;
use crate::sema::Namespaces;
use crate::sema::SourceState;
use crate::sema::scope::{PendingState, ScopeKind};
use crate::sema::{
    BootstrapModule, Compiler, Diagnostic, FileId, FileInfo, HashSet, Module, ModuleId, PathBuf,
    Rc, Result, ScopeId, SourcePath, Span, Sym, TypeId, Value, ast, err,
};
use std::path::Path;

/// A cached source request may be in a recursive load, complete, or failed.
/// Failure must never be mistaken for an already-loaded successful file.
pub(in crate::sema) enum FileLoadState {
    Loading,
    Ready,
    Failed(Box<Diagnostic>),
}

impl Compiler {
    fn read_source(&mut self, path: &Path, span: Span) -> Result<FileId> {
        let Some(bytes) = self.src.fs.read(path) else {
            return Err(Box::new(self.src.unreadable_file(path, span)));
        };
        let text: Rc<str> = String::from_utf8_lossy(&bytes).into();
        Ok(self.src.sources.add(path, text))
    }

    /// Distribution boundary for library-owned helpers, checked before cache hits.
    /// Canonical paths cover relative traversal and native filesystem aliases alike.
    fn check_source_access(&self, path: &Path, span: Span) -> Result<()> {
        if span == Span::NONE && self.ed.is_selected_entry(&self.src.canon(path)) {
            return Ok(());
        }
        let Some(stdlib) = self.options.stdlib_root() else {
            return Ok(());
        };
        let internal = self.src.canon(&stdlib.join("internal"));
        if !self.src.canon(path).starts_with(&internal) {
            return Ok(());
        }
        let from_library = self
            .src
            .sources
            .try_get(self.src.source_origin(span.file))
            .is_some_and(|source| {
                self.src
                    .canon(&source.path.to_path())
                    .starts_with(self.src.canon(stdlib))
            });
        if from_library {
            return Ok(());
        }
        err(
            span,
            "standard-library internal sources are private; import a public library module",
        )
    }

    /// Load a file into `module`, declaring its top-level statements.
    pub fn load_file(&mut self, path: &Path, module: ModuleId, span: Span) -> Result<()> {
        self.check_source_access(path, span)?;
        let path = self.src.canon(path);
        let key = (path.clone(), module);
        if let Some(state) = self.src.file_by_path.get(&key) {
            return match state {
                FileLoadState::Loading | FileLoadState::Ready => Ok(()),
                FileLoadState::Failed(error) => Err(error.clone()),
            };
        }
        self.src
            .file_by_path
            .insert(key.clone(), FileLoadState::Loading);
        let result = self.load_file_inner(&path, module, span);
        let state = match &result {
            Ok(()) => FileLoadState::Ready,
            Err(error) => FileLoadState::Failed(error.clone()),
        };
        self.src.file_by_path.insert(key, state);
        result
    }

    fn load_file_inner(&mut self, path: &Path, module: ModuleId, span: Span) -> Result<()> {
        let parsing = crate::progress::enter(crate::progress::Phase::Parsing);
        if self.ed.allows_parse_ahead()
            && let Some(prepared) = self.src.ahead.as_ref().and_then(|a| a.claim(path))
        {
            let (text, line_starts, tree) = prepared.into_parts();
            let file = self
                .src
                .sources
                .add_file(crate::source::SourceFile::with_lines(
                    path.into(),
                    text,
                    line_starts,
                ));
            let ast = tree.finish(file);
            drop(parsing);
            crate::progress::file_loaded();
            return self.declare_loaded(path, module, file, &ast);
        }
        let file = self.read_source(path, span)?;
        let text = self.src.sources.get(file).text.clone();
        let shared;
        let own;
        let ast = if let Some(parsed) = self.ed.parse(path, file, &text) {
            shared = parsed;
            &shared.file
        } else {
            own = match self.library_platform(path) {
                Some(target) => crate::parser::parse_file_for_target(file, &text, target),
                None => crate::parser::parse_file(file, &text),
            }
            .map_err(Box::new)?;
            if let Some(ahead) = &self.src.ahead {
                ahead.discover(&own, path);
            }
            &own
        };
        drop(parsing);
        crate::progress::file_loaded();
        self.declare_loaded(path, module, file, ast)
    }

    /// Enter the parsed file `file` (read from `path`) into `module`.
    fn declare_loaded(
        &mut self,
        path: &Path,
        module: ModuleId,
        file: FileId,
        ast: &ast::File,
    ) -> Result<()> {
        let module_scope = self.ns.modules[module.0 as usize].scope;
        let file_scope = self.new_scope(ScopeKind::File, Some(module_scope), module, Some(file));
        self.src.files.push(FileInfo {
            id: file,
            path: SourcePath::File(path.to_path_buf()),
            module,
            scope: file_scope,
        });
        self.ns.modules[module.0 as usize].files.push(file);
        self.declare_stmts(module_scope, file_scope, &ast.stmts, ast::ScopeKind::Export)?;
        Ok(())
    }

    /// The platform to read `path` for when it belongs to the standard library, whose dead
    /// file-level `#if` branches are then skipped while parsing. User code is parsed in full, so
    /// a syntax error in code the target does not build is still reported.
    fn library_platform(&mut self, path: &Path) -> Option<crate::parser::PlatformTarget> {
        let stdlib = self.options.stdlib_root()?.to_path_buf();
        if !path.starts_with(self.src.canon(&stdlib)) {
            return None;
        }
        Some(crate::parser::PlatformTarget {
            os: crate::target::os_member(self.options.os),
            cpu: crate::target::cpu_member(self.options.cpu),
        })
    }

    pub fn file_dir(&self, file: FileId) -> PathBuf {
        let path = self
            .src
            .sources
            .get(self.src.source_origin(file))
            .path
            .to_path();
        path.parent().map(Path::to_path_buf).unwrap_or_default()
    }

    /// Create a module and load its entry file.
    pub fn load_module(
        &mut self,
        name: &str,
        entry: &Path,
        params: Vec<(Sym, Value, TypeId)>,
        span: Span,
    ) -> Result<ModuleId> {
        self.load_module_request(name, entry, params, span, None)
    }

    pub(super) fn load_module_request(
        &mut self,
        name: &str,
        entry: &Path,
        params: Vec<(Sym, Value, TypeId)>,
        span: Span,
        import: Option<&ast::Import>,
    ) -> Result<ModuleId> {
        self.check_source_access(entry, span)?;
        // Jai identifies an instance by the argument list as written. Reordering
        // named arguments creates another instance, even when the effective values match.
        // Keep positional names and explicit/default argument differences in the key too.
        let key = (self.src.canon(entry), params.clone());
        if let Some(&id) = self.ns.module_cache.get(&key) {
            if let Some(import) = import {
                self.validate_or_queue_import_groups(id, import)?;
            }
            return self.ns.module_load_result(id);
        }
        // Program parameters (`#import "Basic"()(MEMORY_DEBUGGER=true)`) configure the
        // one instance every importer shares.
        if !params.is_empty()
            && let Some(id) = self.program_instance(&key.0, &params, span, import)?
        {
            return self.ns.module_load_result(id);
        }
        // A plain `#import "M"` joins an instance already configured elsewhere
        // (`#import "Basic"()(MEMORY_DEBUGGER=true)` in the program applies to all), but not
        // one whose ordinary parameters were set: that is another instance of the module.
        if params.is_empty()
            && let Some(id) = self
                .ns
                .module_cache
                .iter()
                .filter(|((path, _), _)| *path == key.0)
                .map(|(_, &id)| id)
                .filter(|id| {
                    let m = &self.ns.modules[id.0 as usize];
                    m.params.iter().all(|(n, ..)| m.program_params.contains(n))
                })
                .min_by_key(|id| id.0)
        {
            return self.ns.module_load_result(id);
        }
        let id = ModuleId(self.ns.modules.len() as u32);
        let scope = self.new_scope(ScopeKind::Module, Some(self.ns.root_scope), id, None);
        crate::progress::module_created();
        self.ns.modules.push(Module {
            loading: true,
            argument_span: span,
            argument_imports: import.cloned().map(Rc::new).into_iter().collect(),
            arguments_validated: false,
            name: name.to_string(),
            path: Some(entry.to_path_buf()),
            scope,
            params,
            param_entities: Vec::new(),
            program_params: Vec::new(),
            parameter_contracts: Vec::new(),
            exported_usings: Vec::new(),
            exported_using_imports: Vec::new(),
            using_modules: Vec::new(),
            files: Vec::new(),
        });
        self.ns.module_cache.insert(key, id);
        let loaded = self.load_file(entry, id, span);
        self.ns.modules[id.0 as usize].loading = false;
        if let Err(error) = loaded {
            self.ns.module_failures.insert(id, error.clone());
            return Err(error);
        }
        self.apply_program_params(id);
        if self
            .scope(scope)
            .pending
            .iter()
            .all(|p| p.state == PendingState::Done)
            && let Err(error) = self.validate_module_arguments(id, span)
        {
            self.ns.module_failures.insert(id, error.clone());
            return Err(error);
        }
        Ok(id)
    }

    /// Bind each written argument to one declared slot. Library provenance does
    /// not relax this contract; excess positions and duplicate bindings are errors.
    pub(in crate::sema) fn validate_module_arguments(
        &mut self,
        id: ModuleId,
        span: Span,
    ) -> Result<()> {
        let module = &self.ns.modules[id.0 as usize];
        if module.arguments_validated {
            return Ok(());
        }
        let span = if span == Span::NONE {
            module.argument_span
        } else {
            span
        };
        let mut used = HashSet::default();
        for (name, ..) in &module.params {
            let position = if let Some(position) = name.as_str().strip_prefix('$') {
                position
                    .parse::<usize>()
                    .ok()
                    .filter(|&p| p < module.param_entities.len())
            } else {
                module
                    .param_entities
                    .iter()
                    .position(|&id| self.entity(id).name == *name)
            };
            let Some(position) = position else {
                return err(
                    span,
                    format!("argument `{name}` is not a parameter of this module"),
                );
            };
            if !used.insert(position) {
                return err(
                    span,
                    format!(
                        "module parameter `{}` is supplied more than once",
                        self.entity(module.param_entities[position]).name
                    ),
                );
            }
        }
        self.ns.modules[id.0 as usize].arguments_validated = true;
        Ok(())
    }

    /// Find `Name.jai` or `Name/module.jai` on the import path (and next to `from_dir`).
    pub fn find_module(&self, name: &str, from_dir: &Path) -> Option<PathBuf> {
        // The public protocol name denotes the selected bootstrap even when a local
        // module happens to have that name. Otherwise its types and storage diverge.
        if BootstrapModule::named(name) == Some(BootstrapModule::Preload)
            && let Some(preload) = &self.options.preload
        {
            return Some(preload.clone());
        }
        find_module_in(&*self.src.fs, &self.options.import_paths, name, from_dir)
    }

    /// Where the runtime's entry file is, on the import path.
    pub fn runtime_support_entry(&self) -> Option<PathBuf> {
        self.find_module(BootstrapModule::RuntimeSupport.name(), Path::new(""))
    }

    /// A fresh module with no files yet.
    pub fn new_module(
        &mut self,
        name: &str,
        path: Option<PathBuf>,
        params: Vec<(Sym, Value, TypeId)>,
    ) -> ModuleId {
        let id = ModuleId(self.ns.modules.len() as u32);
        let scope = self.new_scope(ScopeKind::Module, Some(self.ns.root_scope), id, None);
        self.ns.modules.push(Module {
            loading: false,
            argument_span: Span::NONE,
            argument_imports: Vec::new(),
            arguments_validated: false,
            name: name.into(),
            path,
            scope,
            params,
            param_entities: Vec::new(),
            program_params: Vec::new(),
            parameter_contracts: Vec::new(),
            exported_usings: Vec::new(),
            exported_using_imports: Vec::new(),
            using_modules: Vec::new(),
            files: Vec::new(),
        });
        id
    }

    /// Add source text (an `#import,string` or a workspace build string) as a
    /// file of `module`. `label` names it in diagnostics, written inside angle brackets.
    pub fn load_string(&mut self, label: &str, source: &str, module: ModuleId) -> Result<()> {
        let scope = self.ns.modules[module.0 as usize].scope;
        self.load_string_in_scope(label, source, scope)
    }

    pub(in crate::sema) fn load_string_in_scope(
        &mut self,
        label: &str,
        source: &str,
        target: ScopeId,
    ) -> Result<()> {
        self.ct.interp.invalidate_callable_inspections();
        let module = self.scope(target).module;
        let name = SourcePath::virtual_source(label);
        let file = self.src.sources.add(name.clone(), source.into());
        let mut origin_scope = Some(target);
        while let Some(scope) = origin_scope {
            if let Some(origin) = self.scope(scope).file {
                self.src
                    .origins
                    .insert(file, self.src.source_origin(origin));
                break;
            }
            origin_scope = self.scope(scope).parent;
        }
        let ast = crate::parser::parse_file(file, source).map_err(Box::new)?;
        self.diag.dependencies.source_scopes.insert(file, target);
        let file_scope = self.new_scope(ScopeKind::File, Some(target), module, Some(file));
        self.src.files.push(FileInfo {
            id: file,
            path: name,
            module,
            scope: file_scope,
        });
        self.ns.modules[module.0 as usize].files.push(file);
        self.declare_stmts(target, file_scope, &ast.stmts, ast::ScopeKind::Export)?;
        Ok(())
    }
}

impl Namespaces {
    pub(super) fn module_load_result(&self, id: ModuleId) -> Result<ModuleId> {
        match self.module_failures.get(&id) {
            Some(error) => Err(error.clone()),
            None => Ok(id),
        }
    }
}

impl SourceState {
    fn source_origin(&self, file: FileId) -> FileId {
        self.origins.get(&file).copied().unwrap_or(file)
    }
}
