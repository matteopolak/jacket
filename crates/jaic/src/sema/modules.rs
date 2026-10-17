//! Source loading: files, `#load`, `#import`, module instances and the
//! declaration pass that enters top-level statements into scopes.
use super::scope::{EntityKind, ImportEntry, Pending, PendingState, ScopeKind};
use super::*;
use std::path::Path;

/// Source access. Native builds read the filesystem; the browser supplies a
/// virtual file system containing the workspace, prelude and stdlib.
pub trait FileSystem {
    fn read(&self, path: &Path) -> Option<Vec<u8>>;
    fn is_file(&self, path: &Path) -> bool;
    fn is_dir(&self, path: &Path) -> bool;

    fn canonical(&self, path: &Path) -> PathBuf {
        normalize(path)
    }

    /// Names (and whether each is a directory) directly inside `path`; used by the sandbox host.
    fn list_dir(&self, _path: &Path) -> Vec<(String, bool)> {
        Vec::new()
    }
}

pub struct NativeFs;

/// A module-owned header contract, evaluated in its declaring file namespace.
/// Registration must not consume a default while an importing type is forming.
pub struct ParameterContract {
    entity: EntityId,
    pattern: ast::Expr,
    scope: ScopeId,
    state: ParameterContractState,
}

enum ParameterContractState {
    Waiting,
    Checking,
    Checked,
    Failed(Box<Diagnostic>),
}

/// A cached source request may be in a recursive load, complete, or failed.
/// Failure must never be mistaken for an already-loaded successful file.
pub(super) enum FileLoadState {
    Loading,
    Ready,
    Failed(Box<Diagnostic>),
}

impl FileSystem for NativeFs {
    fn read(&self, path: &Path) -> Option<Vec<u8>> {
        std::fs::read(path).ok()
    }

    fn is_file(&self, path: &Path) -> bool {
        path.is_file()
    }

    fn is_dir(&self, path: &Path) -> bool {
        path.is_dir()
    }

    fn canonical(&self, path: &Path) -> PathBuf {
        crate::canonicalize(path).unwrap_or_else(|_| normalize(path))
    }

    fn list_dir(&self, path: &Path) -> Vec<(String, bool)> {
        let Ok(entries) = std::fs::read_dir(path) else {
            return Vec::new();
        };
        entries
            .filter_map(|e| e.ok())
            .map(|e| {
                (
                    e.file_name().to_string_lossy().into_owned(),
                    e.path().is_dir(),
                )
            })
            .collect()
    }
}

#[derive(Default)]
pub struct VirtualFs {
    pub files: HashMap<PathBuf, Rc<[u8]>>,
}

impl VirtualFs {
    pub fn insert(&mut self, path: impl Into<PathBuf>, bytes: impl Into<Rc<[u8]>>) {
        self.files.insert(normalize(&path.into()), bytes.into());
    }
}

impl FileSystem for VirtualFs {
    fn read(&self, path: &Path) -> Option<Vec<u8>> {
        self.files.get(&normalize(path)).map(|b| b.to_vec())
    }

    fn is_file(&self, path: &Path) -> bool {
        self.files.contains_key(&normalize(path))
    }

    fn is_dir(&self, path: &Path) -> bool {
        let p = normalize(path);
        self.files.keys().any(|k| k.starts_with(&p) && k != &p)
    }

    fn list_dir(&self, path: &Path) -> Vec<(String, bool)> {
        let p = normalize(path);
        let mut out: std::collections::BTreeMap<String, bool> = Default::default();
        for key in self.files.keys() {
            let Ok(rest) = key.strip_prefix(&p) else {
                continue;
            };
            let mut parts = rest.components();
            let Some(first) = parts.next() else {
                continue;
            };
            let nested = parts.next().is_some();
            let name = first.as_os_str().to_string_lossy().into_owned();
            let entry = out.entry(name).or_insert(false);
            *entry |= nested;
        }
        out.into_iter().collect()
    }
}

/// Lexically normalize `a/./b/../c` without touching the filesystem.
pub fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                // `/..` is `/`, as on a real filesystem; a relative path keeps leading `..`.
                if out.has_root() && out.parent().is_none() {
                } else if out
                    .components()
                    .next_back()
                    .is_none_or(|c| c == std::path::Component::ParentDir)
                    || !out.pop()
                {
                    out.push("..");
                }
            }
            c => out.push(c.as_os_str()),
        }
    }
    out
}

/// Find `Name.jai` or `Name/module.jai` in `from_dir/modules`, then on `import_paths`.
/// The public `Preload` protocol name also aliases the single bootstrap entry.
pub fn find_module_in(
    fs: &dyn FileSystem,
    import_paths: &[PathBuf],
    name: &str,
    from_dir: &Path,
) -> Option<PathBuf> {
    let mut dirs: Vec<PathBuf> = vec![from_dir.join("modules")];
    dirs.extend(import_paths.iter().cloned());
    for dir in &dirs {
        let file = dir.join(format!("{name}.jai"));
        if fs.is_file(&file) {
            return Some(file);
        }
        let entry = dir.join(name).join("module.jai");
        if fs.is_file(&entry) {
            return Some(entry);
        }
    }
    if name == "Preload" {
        dirs.iter().find_map(|dir| crate::find_preload(fs, dir))
    } else {
        None
    }
}

/// The entry file an `#import` written in a file of `dir` loads: a module found on the import
/// path, `#import,file` relative to `dir`, or `#import,dir`'s `module.jai`. `None` for an
/// unknown module or `#import,string`. (The file itself may not exist for `,file`/`,dir`.)
pub fn import_entry(
    fs: &dyn FileSystem,
    import_paths: &[PathBuf],
    source: &ast::ImportSource,
    dir: &Path,
) -> Option<PathBuf> {
    match source {
        ast::ImportSource::Module(name) => find_module_in(fs, import_paths, name, dir),
        ast::ImportSource::File(path) => Some(dir.join(&**path)),
        ast::ImportSource::Dir(path) => Some(dir.join(&**path).join("module.jai")),
        ast::ImportSource::String(_) => None,
    }
}

/// The module names a directory offers: `Name.jai` files and `Name/module.jai` folders.
fn modules_in(fs: &dyn FileSystem, dir: &Path) -> Vec<String> {
    fs.list_dir(dir)
        .into_iter()
        .filter_map(|(name, is_dir)| {
            if is_dir {
                fs.is_file(&dir.join(&name).join("module.jai"))
                    .then_some(name)
            } else {
                name.strip_suffix(".jai").map(str::to_string)
            }
        })
        .collect()
}

/// Marks an import argument `.Member` whose type comes from the module parameter.
const INFERRED_PARAM: &str = "\0inferred.";

impl Compiler {
    /// For a missing module that is one of jaic's `stdlib/Extensions/` modules imported without
    /// the `Extensions/` path (or the folder itself, or the old `Jaic_Extensions`): `d` with that
    /// help, which replaces the general search help. `Err(d)` unchanged otherwise.
    fn with_extension_help(
        &self,
        d: Diagnostic,
        name: &str,
        span: Span,
    ) -> std::result::Result<Diagnostic, Diagnostic> {
        let folder = crate::STDLIB_EXTENSIONS_DIR;
        let Some(stdlib) = self.options.stdlib_root() else {
            return Err(d);
        };
        if name == folder {
            let mut modules: Vec<String> = self
                .stdlib_modules_under(stdlib, folder)
                .into_iter()
                .map(|(m, _)| format!("`{m}`"))
                .collect();
            modules.sort();
            if modules.is_empty() {
                return Err(d);
            }
            return Ok(d.with_help(format!(
                "`{folder}` is the folder of jaic's extension modules, not a module: import one of them by its path: {}",
                modules.join(", ")
            )));
        }
        if name.contains('/') {
            return Err(d);
        }
        // `Jaic_Extensions` held only Long_Double, which is now a module of its own.
        let wanted = if name == "Jaic_Extensions" {
            "Long_Double"
        } else {
            name
        };
        let path = format!("{folder}/{wanted}");
        let exists = self.fs.is_file(&stdlib.join(&path).join("module.jai"))
            || self.fs.is_file(&stdlib.join(format!("{path}.jai")));
        if !exists {
            return Err(d);
        }
        let message = if wanted == name {
            format!(
                "`{name}` is a jaic extension (not official Jai), imported by its path: `#import \"{path}\";`"
            )
        } else {
            format!("`{name}` was replaced by `{path}`: `#import \"{path}\";`")
        };
        Ok(
            match self.sources.snippet(span).find(&format!("\"{name}\"")) {
                Some(at) => {
                    let start = span.start as usize + at + 1;
                    d.with_fix(
                        message,
                        Span::new(span.file, start, start + name.len()),
                        path,
                    )
                }
                None => d.with_help(message),
            },
        )
    }

    /// `module `X` not found`, with where jaic looked and the closest module name there.
    fn module_not_found(&self, name: &str, from_dir: &Path, span: Span) -> Diagnostic {
        let mut dirs = vec![from_dir.join("modules")];
        dirs.extend(self.options.import_paths.iter().cloned());
        dirs.dedup();
        let mut available: Vec<String> =
            dirs.iter().flat_map(|d| modules_in(&*self.fs, d)).collect();
        available.sort();
        available.dedup();
        let d = Diagnostic::error(span, format!("module `{name}` not found"))
            .with_label("no module of this name");
        let mut d = match self.with_extension_help(d, name, span) {
            Ok(d) => return d,
            Err(d) => d,
        };
        // A misspelled `Extensions/Name`: the closest extension module.
        if name.starts_with(&format!("{}/", crate::STDLIB_EXTENSIONS_DIR))
            && let Some(stdlib) = self.options.stdlib_root()
        {
            available.extend(
                self.stdlib_modules_under(stdlib, crate::STDLIB_EXTENSIONS_DIR)
                    .into_iter()
                    .map(|(m, _)| m),
            );
        }
        let near =
            crate::suggest::closest(name, available.iter().map(String::as_str)).map(str::to_string);
        let near_found = near.is_some();
        if let Some(near) = near {
            let quoted = self.sources.snippet(span).find(&format!("\"{name}\""));
            d = match quoted {
                Some(at) => {
                    let start = span.start as usize + at + 1;
                    let at = Span::new(span.file, start, start + name.len());
                    d.with_fix(
                        format!("a module with a similar name exists: `{near}`"),
                        at,
                        near,
                    )
                }
                None => d.with_help(format!("a module with a similar name exists: `{near}`")),
            };
        }
        let searched: Vec<String> = dirs
            .iter()
            .map(|d| format!("    {}", self.fs.canonical(d).display()))
            .collect();
        d = d.with_note(
            Span::NONE,
            format!(
                "looked for `{name}.jai` and `{name}/module.jai` in:\n{}",
                searched.join("\n")
            ),
        );
        if near_found {
            return d;
        }
        // The module may be nearby, in a folder jaic does not search.
        if let Some(found) = self.module_nearby(name, from_dir) {
            let shown = crate::display_path(&found);
            let shown = if shown.is_empty() {
                ".".to_string()
            } else {
                shown
            };
            return d.with_help(format!(
                "a module `{name}` exists in `{shown}`, which is not searched: pass `-import_dir {shown}`, or move it into a `modules` folder next to the main file"
            ));
        }
        d.with_help(
            "a module of your own goes in a `modules` folder next to the main file, or in a directory passed with `-import_dir`",
        )
    }

    /// A directory near `from_dir` (it, its parents up to three levels, and their immediate
    /// subdirectories) that holds the module `name`.
    fn module_nearby(&self, name: &str, from_dir: &Path) -> Option<PathBuf> {
        let holds = |dir: &Path| {
            self.fs.is_file(&dir.join(format!("{name}.jai")))
                || self.fs.is_file(&dir.join(name).join("module.jai"))
        };
        let start = self.fs.canonical(from_dir);
        for ancestor in start.ancestors().take(4) {
            if holds(ancestor) {
                return Some(ancestor.to_path_buf());
            }
            let mut subdirs: Vec<String> = self
                .fs
                .list_dir(ancestor)
                .into_iter()
                .filter(|(n, is_dir)| *is_dir && !n.starts_with('.'))
                .map(|(n, _)| n)
                .collect();
            subdirs.sort();
            for sub in subdirs {
                let dir = ancestor.join(sub);
                if holds(&dir) {
                    return Some(dir);
                }
            }
        }
        None
    }

    /// `#load` (or another read) of a file that is not there, with a close name if one is.
    fn unreadable_file(&self, path: &Path, span: Span) -> Diagnostic {
        let snippet = self.sources.snippet(span);
        // An `#import,file` or `#import,dir` span covers the directive, not only its path.
        let import = snippet.starts_with("#import");
        let shown = match snippet.split_once('"') {
            Some((_, rest)) if import => rest.trim_end_matches([';', '"', ' ']).to_string(),
            _ => snippet.trim_matches('"').to_string(),
        };
        let shown = if shown.is_empty() || shown.contains('\n') {
            path.display().to_string()
        } else {
            shown
        };
        let relative = if import {
            "`#import,file` and `#import,dir` paths are relative to the file that imports them"
        } else {
            "`#load` paths are relative to the file that loads them"
        };
        if import && snippet.contains(",dir") {
            let folder = path.parent().unwrap_or(Path::new("."));
            let message = if self.fs.is_dir(folder) {
                format!("`{shown}` has no module.jai")
            } else {
                format!("folder `{shown}` does not exist")
            };
            return Diagnostic::error(span, message)
                .with_kind(DiagnosticKind::MissingFile)
                .with_note(Span::NONE, format!("looked for {}", path.display()))
                .with_help(relative);
        }
        if self.fs.is_dir(path) {
            return Diagnostic::error(span, format!("`{shown}` is a directory, not a file"))
                .with_help(
                    "`#load` takes a .jai file; `#import,dir` imports a directory's module.jai",
                );
        }
        let mut d = Diagnostic::error(span, format!("file `{shown}` does not exist"))
            .with_kind(DiagnosticKind::MissingFile);
        let dir = path.parent().unwrap_or(Path::new("."));
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let files: Vec<String> = self
            .fs
            .list_dir(dir)
            .into_iter()
            .filter(|(n, is_dir)| !is_dir && n.ends_with(".jai"))
            .map(|(n, _)| n)
            .collect();
        if let Some(near) = crate::suggest::similar_entry(&name, files.iter().map(String::as_str)) {
            d = d.with_help(format!("a file with a similar name exists: `{near}`"));
        }
        d.with_note(Span::NONE, format!("looked for {}", path.display()))
            .with_help(relative)
    }

    fn read_source(&mut self, path: &Path, span: Span) -> Result<FileId> {
        let Some(bytes) = self.fs.read(path) else {
            return Err(Box::new(self.unreadable_file(path, span)));
        };
        let text: Rc<str> = String::from_utf8_lossy(&bytes).into();
        Ok(self.sources.add(path.display().to_string(), text))
    }

    /// Distribution boundary for library-owned helpers, checked before cache hits.
    /// Canonical paths cover relative traversal and native filesystem aliases alike.
    fn check_source_access(&self, path: &Path, span: Span) -> Result<()> {
        if span == Span::NONE
            && self.ide.is_some()
            && self.ide_selected_entry.as_ref() == Some(&self.fs.canonical(path))
        {
            return Ok(());
        }
        let Some(stdlib) = self.options.stdlib_root() else {
            return Ok(());
        };
        let internal = self.fs.canonical(&stdlib.join("internal"));
        if !self.fs.canonical(path).starts_with(&internal) {
            return Ok(());
        }
        let from_library = self
            .sources
            .try_get(self.source_origin(span.file))
            .is_some_and(|source| {
                self.fs
                    .canonical(Path::new(&source.path))
                    .starts_with(self.fs.canonical(stdlib))
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
        let path = self.fs.canonical(path);
        let key = (path.clone(), module);
        if let Some(state) = self.file_by_path.get(&key) {
            return match state {
                FileLoadState::Loading | FileLoadState::Ready => Ok(()),
                FileLoadState::Failed(error) => Err(error.clone()),
            };
        }
        self.file_by_path
            .insert(key.clone(), FileLoadState::Loading);
        let result = self.load_file_inner(&path, module, span);
        let state = match &result {
            Ok(()) => FileLoadState::Ready,
            Err(error) => FileLoadState::Failed(error.clone()),
        };
        self.file_by_path.insert(key, state);
        result
    }

    fn load_file_inner(&mut self, path: &Path, module: ModuleId, span: Span) -> Result<()> {
        let file = self.read_source(path, span)?;
        let text = self.sources.get(file).text.clone();
        let shared;
        let own;
        let ast = if let Some(ide) = self.ide.as_mut() {
            shared = match &self.parse_reuse {
                Some((reuse, compile)) => reuse.borrow_mut().parse(*compile, path, file, &text),
                None => {
                    let parsed = crate::parser::parse_file_ide(file, &text);
                    Rc::new(crate::parser::SharedParse {
                        file: parsed.file,
                        diagnostics: parsed.diagnostics,
                    })
                }
            };
            ide.parse_errors.extend(shared.diagnostics.iter().cloned());
            ide.errors.extend(shared.diagnostics.iter().cloned());
            &shared.file
        } else {
            own = crate::parser::parse_file(file, &text).map_err(Box::new)?;
            &own
        };
        let module_scope = self.modules[module.0 as usize].scope;
        let file_scope = self.new_scope(ScopeKind::File, Some(module_scope), module, Some(file));
        self.files.push(FileInfo {
            id: file,
            path: path.to_path_buf(),
            module,
            scope: file_scope,
        });
        self.modules[module.0 as usize].files.push(file);
        self.declare_stmts(module_scope, file_scope, &ast.stmts, ast::ScopeKind::Export)?;
        Ok(())
    }

    pub fn file_dir(&self, file: FileId) -> PathBuf {
        let path = PathBuf::from(&self.sources.get(self.source_origin(file)).path);
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

    fn load_module_request(
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
        let key = (self.fs.canonical(entry), params.clone());
        if let Some(&id) = self.module_cache.get(&key) {
            if let Some(import) = import {
                self.validate_or_queue_import_groups(id, import)?;
            }
            return self.module_load_result(id);
        }
        // Program parameters (`#import "Basic"()(MEMORY_DEBUGGER=true)`) configure the
        // one instance every importer shares.
        if !params.is_empty()
            && let Some(id) = self.program_instance(&key.0, &params, span, import)?
        {
            return self.module_load_result(id);
        }
        // A plain `#import "M"` joins an instance already configured elsewhere
        // (`#import "Basic"()(MEMORY_DEBUGGER=true)` in the program applies to all), but not
        // one whose ordinary parameters were set: that is another instance of the module.
        if params.is_empty()
            && let Some(id) = self
                .module_cache
                .iter()
                .filter(|((path, _), _)| *path == key.0)
                .map(|(_, &id)| id)
                .filter(|id| {
                    let m = &self.modules[id.0 as usize];
                    m.params.iter().all(|(n, ..)| m.program_params.contains(n))
                })
                .min_by_key(|id| id.0)
        {
            return self.module_load_result(id);
        }
        let id = ModuleId(self.modules.len() as u32);
        let scope = self.new_scope(ScopeKind::Module, Some(self.root_scope), id, None);
        self.modules.push(Module {
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
            files: Vec::new(),
        });
        self.module_cache.insert(key, id);
        let loaded = self.load_file(entry, id, span);
        self.modules[id.0 as usize].loading = false;
        if let Err(error) = loaded {
            self.module_failures.insert(id, error.clone());
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
            self.module_failures.insert(id, error.clone());
            return Err(error);
        }
        Ok(id)
    }

    /// Bind each written argument to one declared slot. Library provenance does
    /// not relax this contract; excess positions and duplicate bindings are errors.
    pub(super) fn validate_module_arguments(&mut self, id: ModuleId, span: Span) -> Result<()> {
        let module = &self.modules[id.0 as usize];
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
        self.modules[id.0 as usize].arguments_validated = true;
        Ok(())
    }

    fn module_load_result(&self, id: ModuleId) -> Result<ModuleId> {
        match self.module_failures.get(&id) {
            Some(error) => Err(error.clone()),
            None => Ok(id),
        }
    }

    /// Record the named parameters of an import so a program parameter among them reaches
    /// the module's shared instance before the import itself is resolved.
    fn note_program_params(&mut self, file_scope: ScopeId, import: &ast::Import) {
        if Some(self.scope(file_scope).module) != self.main_module {
            return;
        }
        let Some(start) = import.program_params_start else {
            return;
        };
        let arguments = &import.params[start..];
        let ast::ImportSource::Module(name) = &import.source else {
            return;
        };
        if arguments.iter().all(|a| a.name.is_none()) {
            return;
        }
        let file = self.scope(file_scope).file.unwrap_or_default();
        let Some(entry) = self.find_module(name, &self.file_dir(file)) else {
            return;
        };
        let path = self.fs.canonical(&entry);
        for arg in arguments {
            if let Some(n) = arg.name {
                self.program_param_settings.push((
                    path.clone(),
                    n.name,
                    arg.value.clone(),
                    file_scope,
                ));
            }
        }
        let loaded: Vec<ModuleId> = self
            .module_cache
            .iter()
            .filter(|((p, _), _)| *p == path)
            .map(|(_, &id)| id)
            .collect();
        for id in loaded {
            self.apply_program_params(id);
        }
    }

    /// Give a module instance's not-yet-used program parameters the values imports set.
    fn apply_program_params(&mut self, module: ModuleId) {
        let Some(path) = self.modules[module.0 as usize].path.clone() else {
            return;
        };
        let path = self.fs.canonical(&path);
        for (p, name, value, scope) in self.program_param_settings.clone() {
            if p != path
                || !self.modules[module.0 as usize]
                    .program_params
                    .contains(&name)
            {
                continue;
            }
            let entity = self.modules[module.0 as usize]
                .param_entities
                .iter()
                .copied()
                .find(|&e| self.entity(e).name == name);
            let Some(entity) = entity else {
                continue;
            };
            if !matches!(self.entity(entity).state, scope::EntityState::Unresolved) {
                continue;
            }
            // The value is evaluated where the import wrote it; a written parameter type
            // would not resolve there, so typed parameters wait for the import itself.
            if let EntityKind::Decl {
                decl, ..
            } = &mut self.entity_mut(entity).kind
                && decl.ty.is_none()
            {
                Rc::make_mut(decl).value = Some(value);
                self.entity_mut(entity).home = scope;
            }
        }
    }

    /// The loaded instance of the module at `path` when `params` only set its program
    /// parameters; their values are applied to it. A parameter the module already used
    /// with a different value is an error.
    fn program_instance(
        &mut self,
        path: &Path,
        params: &[(Sym, Value, TypeId)],
        span: Span,
        import: Option<&ast::Import>,
    ) -> Result<Option<ModuleId>> {
        let Some(id) = self
            .module_cache
            .iter()
            .filter(|((p, _), _)| p == path)
            .map(|(_, &id)| id)
            .min_by_key(|id| id.0)
        else {
            return Ok(None);
        };
        let module = &self.modules[id.0 as usize];
        if !params
            .iter()
            .all(|(n, ..)| module.program_params.contains(n))
        {
            return Ok(None);
        }
        if let Some(import) = import {
            self.validate_or_queue_import_groups(id, import)?;
        }
        self.module_load_result(id)?;
        for (name, value, ty) in params {
            let entity = self.modules[id.0 as usize]
                .param_entities
                .iter()
                .copied()
                .find(|&e| self.entity(e).name == *name);
            let Some(entity) = entity else {
                continue;
            };
            if let scope::EntityState::Done(resolved) = &self.entity(entity).state {
                let same =
                    matches!(resolved, scope::Resolved::Const { value: v, .. } if v == value);
                if !same {
                    return err(
                        span,
                        format!(
                            "module parameter `{name}` is set here after the module already used another value"
                        ),
                    );
                }
                continue;
            }
            // An import may have bound the value without resolving the entity
            // yet. Deferred contract checking must not let a later importer
            // overwrite that explicit binding merely because it is still lazy.
            if let EntityKind::Const {
                value: bound, ..
            } = &self.entity(entity).kind
            {
                if bound != value {
                    return err(
                        span,
                        format!(
                            "module parameter `{name}` is set here after the module already used another value"
                        ),
                    );
                }
                continue;
            }
            self.entity_mut(entity).kind = EntityKind::Const {
                value: value.clone(),
                ty: *ty,
            };
        }
        Ok(Some(id))
    }

    /// Find `Name.jai` or `Name/module.jai` on the import path (and next to `from_dir`).
    pub fn find_module(&self, name: &str, from_dir: &Path) -> Option<PathBuf> {
        // The public protocol name denotes the selected bootstrap even when a local
        // module happens to have that name. Otherwise its types and storage diverge.
        if name == "Preload"
            && let Some(preload) = &self.options.preload
        {
            return Some(preload.clone());
        }
        find_module_in(&*self.fs, &self.options.import_paths, name, from_dir)
    }

    /// Declare top-level statements. Returns the visibility state at the end.
    pub fn declare_stmts(
        &mut self,
        scope: ScopeId,
        file_scope: ScopeId,
        stmts: &[ast::Stmt],
        mut vis: ast::ScopeKind,
    ) -> Result<ast::ScopeKind> {
        let mut failed = None;
        for stmt in stmts {
            match self.declare_stmt(scope, file_scope, stmt, vis) {
                Ok(next) => vis = next,
                Err(error) if self.ide.is_some() => {
                    self.ide.as_mut().unwrap().errors.push((*error).clone());
                    failed.get_or_insert(error);
                }
                Err(error) => return Err(error),
            }
        }
        failed.map_or(Ok(vis), Err)
    }

    fn target_scope(&self, scope: ScopeId, file_scope: ScopeId, vis: ast::ScopeKind) -> ScopeId {
        if vis == ast::ScopeKind::File && self.scope(scope).kind == ScopeKind::Module {
            file_scope
        } else {
            scope
        }
    }

    pub fn declare_stmt(
        &mut self,
        scope: ScopeId,
        file_scope: ScopeId,
        stmt: &ast::Stmt,
        vis: ast::ScopeKind,
    ) -> Result<ast::ScopeKind> {
        let target = self.target_scope(scope, file_scope, vis);
        let exported = vis == ast::ScopeKind::Export;
        let file = self.scope(file_scope).file.unwrap_or_default();
        if matches!(self.scope(scope).kind, ScopeKind::Struct(_)) {
            // Struct bodies: only constants are entities; fields are laid out separately.
            match &stmt.kind {
                ast::StmtKind::Decl(decl) if decl.kind == ast::DeclKind::Const => {
                    for (index, name) in decl.names.iter().enumerate() {
                        self.check_redeclared(scope, name.name, name.span, Some(decl))?;
                        self.add_entity(
                            scope,
                            name.name,
                            name.span,
                            EntityKind::Decl {
                                decl: decl.clone(),
                                index,
                            },
                            true,
                        );
                    }
                }
                ast::StmtKind::StaticIf(_) => {
                    self.push_pending(
                        scope,
                        Pending {
                            stmt: stmt.clone(),
                            exported,
                            file_scope,
                            state: PendingState::Waiting,
                        },
                    );
                }
                _ => {}
            }
            return Ok(vis);
        }
        match &stmt.kind {
            ast::StmtKind::Scope(kind) => return Ok(*kind),
            ast::StmtKind::Decl(decl) => {
                // `using _ :: struct {...}` may appear many times: each gets its own name.
                let entity_name = |c: &Self, name: Sym| {
                    if decl.using && name.as_str() == "_" {
                        Sym::intern(&format!("__using_{}", c.entities.len()))
                    } else {
                        name
                    }
                };
                let mut using_name = None;
                for (index, name) in decl.names.iter().enumerate() {
                    let named = entity_name(self, name.name);
                    using_name.get_or_insert(named);
                    self.check_redeclared(target, named, name.span, Some(decl))?;
                    let id = self.add_entity(
                        target,
                        named,
                        name.span,
                        EntityKind::Decl {
                            decl: decl.clone(),
                            index,
                        },
                        exported,
                    );
                    self.entity_mut(id).file_private = vis == ast::ScopeKind::File;
                    self.entity_mut(id).home = file_scope;
                    if let Some(ast::Expr {
                        kind: ast::ExprKind::Proc(lit),
                        ..
                    }) = &decl.value
                        && lit.header.flags.program_export.is_some()
                    {
                        self.export_entities.push(id);
                    }
                    // `L :: #library,link_always "x"` is linked even if nothing names it.
                    if let Some(ast::Expr {
                        kind: ast::ExprKind::UnknownDirective(directive),
                        ..
                    }) = &decl.value
                        && directive
                            .flags
                            .iter()
                            .any(|f| f.name.as_str() == "link_always")
                    {
                        self.export_entities.push(id);
                    }
                    if !decl.notes.is_empty() || decl.kind == ast::DeclKind::Const {
                        self.note_declaration(id, decl);
                    }
                }
                // A procedure's body `#import`s are visible to the whole module (`#if`-guarded
                // ones once their branch is checked).
                if let Some(ast::Expr {
                    kind: ast::ExprKind::Proc(lit),
                    ..
                }) = &decl.value
                    && let Some(body) = &lit.body
                {
                    self.hoist_body_imports(file_scope, &body.stmts, true);
                }
                if decl.using
                    && decl.kind == ast::DeclKind::Const
                    && let [name] = decl.names.as_slice()
                    && let Some(ast::Expr {
                        kind: ast::ExprKind::Enum(lit),
                        ..
                    }) = &decl.value
                    && lit
                        .items
                        .iter()
                        .all(|i| matches!(i, ast::EnumItem::Member(_)))
                {
                    // `using E :: enum { A; B; }`: the member names are known now, so declare
                    // `A :: E.A;` aliases at once; other declarations (even other enums'
                    // values) may refer to them before the enum itself is checked.
                    for item in &lit.items {
                        let ast::EnumItem::Member(member) = item else {
                            continue;
                        };
                        let span = member.name.span;
                        let alias = const_alias(
                            member.name,
                            ast::Expr {
                                kind: ast::ExprKind::Member(
                                    Box::new(ast::Expr {
                                        kind: ast::ExprKind::Ident(name.name),
                                        span,
                                    }),
                                    member.name,
                                ),
                                span,
                            },
                        );
                        let id = self.add_entity(
                            target,
                            member.name.name,
                            span,
                            EntityKind::Decl {
                                decl: alias,
                                index: 0,
                            },
                            exported,
                        );
                        self.entity_mut(id).file_private = vis == ast::ScopeKind::File;
                        self.entity_mut(id).home = file_scope;
                    }
                } else if decl.using
                    && let [name] = decl.names.as_slice()
                {
                    // `using E :: enum {...}`: also bring the type's members into scope.
                    self.push_pending(
                        target,
                        Pending {
                            stmt: ast::Stmt {
                                kind: ast::StmtKind::Using(Box::new(ast::UsingStmt {
                                    value: ast::Expr {
                                        kind: ast::ExprKind::Ident(using_name.unwrap_or(name.name)),
                                        span: name.span,
                                    },
                                    filter: ast::UsingFilter::None,
                                })),
                                span: stmt.span,
                                notes: Default::default(),
                            },
                            exported,
                            file_scope,
                            state: PendingState::Waiting,
                        },
                    );
                }
            }
            ast::StmtKind::Import(import) => {
                self.validate_import_program_origin(file_scope, import)?;
                self.note_program_params(file_scope, import);
                if let Some(name) = import.name {
                    self.check_redeclared(target, name.name, name.span, None)?;
                    let id = self.add_entity(
                        target,
                        name.name,
                        name.span,
                        EntityKind::Import(import.clone()),
                        exported,
                    );
                    self.entity_mut(id).home = file_scope;
                    if let Some(filter) = &import.using {
                        // `using Name :: #import "M"` also brings the module's names into scope;
                        // at module level (exported) they become the module's members too.
                        if exported && self.scope(target).kind == ScopeKind::Module {
                            let index = self.scope(target).imports.len();
                            let module = self.scope(target).module;
                            self.modules[module.0 as usize]
                                .exported_using_imports
                                .push(index);
                        }
                        self.push_import(
                            target,
                            ImportEntry {
                                import: import.clone(),
                                module: None,
                                loading: false,
                                from_scope: file_scope,
                                filter: filter.clone(),
                            },
                        );
                    }
                } else {
                    // A top-level anonymous import contributes to the containing
                    // module's import environment even after #scope_file. The
                    // directive still governs named aliases and declarations.
                    // Keep contextual imports in their existing scope, and keep
                    // from_scope below for paths/parameters written in this file.
                    let target = if self.scope(scope).kind == ScopeKind::Module {
                        scope
                    } else {
                        target
                    };
                    // Anonymous `using,only/except(...) #import` exposes the
                    // selected original declarations just like its named form,
                    // without introducing a public namespace alias.
                    if import.using.is_some()
                        && exported
                        && self.scope(target).kind == ScopeKind::Module
                    {
                        let index = self.scope(target).imports.len();
                        let module = self.scope(target).module;
                        self.modules[module.0 as usize]
                            .exported_using_imports
                            .push(index);
                    }
                    self.push_import(
                        target,
                        ImportEntry {
                            import: import.clone(),
                            module: None,
                            loading: false,
                            from_scope: file_scope,
                            filter: import.using.clone().unwrap_or(ast::UsingFilter::None),
                        },
                    );
                }
            }
            ast::StmtKind::Load {
                path,
                span,
            } => {
                let module = self.scope(scope).module;
                let full = self.file_dir(file).join(&**path);
                self.load_file(&full, module, *span)?;
            }
            ast::StmtKind::StaticIf(_)
            | ast::StmtKind::StaticSwitch(_)
            | ast::StmtKind::Insert(_) => {
                self.push_pending(
                    target,
                    Pending {
                        stmt: stmt.clone(),
                        exported,
                        file_scope,
                        state: PendingState::Waiting,
                    },
                );
            }
            ast::StmtKind::Run(expr) => self.top_level_runs.push(((**expr).clone(), file_scope)),
            ast::StmtKind::Assert(boxed) => {
                let crate::ast::AssertStmt {
                    cond,
                    message,
                    args,
                } = &**boxed;
                self.asserts
                    .push((cond.clone(), message.clone(), args.clone(), file_scope))
            }
            ast::StmtKind::AddContext(decl) => {
                self.add_contexts.push((decl.clone(), file_scope));
                // Context already made (a `#run` needed it): extend it while its layout
                // is still open.
                if let Some(ty) = self.context_type {
                    let s = self.types.as_struct(ty).unwrap();
                    if self.types.struct_info(s).layout != crate::types::LayoutState::Pending {
                        return err(
                            stmt.span,
                            "#add_context after the Context type was laid out (by compile-time code that ran earlier)",
                        );
                    }
                    if let Some(src) = self.struct_asts.get_mut(&s) {
                        src.extra.push((decl.clone(), file_scope));
                    }
                }
            }
            ast::StmtKind::ModuleParameters(boxed) => {
                let crate::ast::ModuleParametersStmt {
                    params,
                    runtime_params,
                    body,
                } = &**boxed;
                let module = self.scope(scope).module;
                let eval_scope = file_scope_for_eval(self, scope, file_scope);
                for import in self.modules[module.0 as usize].argument_imports.clone() {
                    let ordinary = params
                        .iter()
                        .filter_map(|p| p.name.map(|n| n.name))
                        .collect::<Vec<_>>();
                    let program = runtime_params
                        .iter()
                        .filter_map(|p| p.name.map(|n| n.name))
                        .collect::<Vec<_>>();
                    self.validate_import_groups(&import, &ordinary, &program)?;
                }
                for (position, param) in params.iter().chain(runtime_params).enumerate() {
                    self.declare_module_parameter(scope, module, position, param, eval_scope)?;
                }
                self.validate_module_arguments(module, stmt.span)?;
                // Reject unknown/duplicate arguments after all header names are known,
                // before checking omissions. Program parameters may bind later.
                for (position, param) in params.iter().enumerate() {
                    if param.default.is_none()
                        && let Some(name) = param.name
                        && !self.modules[module.0 as usize]
                            .params
                            .iter()
                            .any(|(n, ..)| *n == name.name || n.as_str() == format!("${position}"))
                    {
                        return err(
                            param.span,
                            format!("missing argument for module parameter `{}`", name.name),
                        );
                    }
                }
                let program = runtime_params.iter().filter_map(|p| p.name.map(|n| n.name));
                self.modules[module.0 as usize]
                    .program_params
                    .extend(program);
                // Registering a typed slot may reenter the loader. Check any
                // requests queued during registration before releasing provenance.
                for import in self.modules[module.0 as usize].argument_imports.clone() {
                    self.validate_or_queue_import_groups(module, &import)?;
                }
                self.modules[module.0 as usize].argument_imports.clear();
                // The trailing block declares names the parameter defaults may use.
                if let Some(body) = body {
                    self.declare_stmts(scope, file_scope, &body.stmts, vis)?;
                }
                // Keep contracts separate from declaration registration. Forming
                // an importing hook type may itself need an unrelated public
                // declaration from this module. Checking the default here would
                // consume it before the configured import can finish binding.
                for param in params.iter().chain(runtime_params) {
                    if let Some(pattern) = param.ty.as_ref().filter(|p| procs::has_poly(p)) {
                        let name = param.name.unwrap().name;
                        let id = self.modules[module.0 as usize]
                            .param_entities
                            .iter()
                            .copied()
                            .find(|&id| self.entity(id).name == name)
                            .unwrap();
                        self.modules[module.0 as usize].parameter_contracts.push(
                            ParameterContract {
                                entity: id,
                                pattern: pattern.clone(),
                                scope: eval_scope,
                                state: ParameterContractState::Waiting,
                            },
                        );
                    }
                }
            }
            ast::StmtKind::Placeholder(names) => {
                for name in names {
                    self.add_entity(
                        target,
                        name.name,
                        name.span,
                        EntityKind::Placeholder,
                        exported,
                    );
                }
            }
            ast::StmtKind::Directive(directive) if directive.name.name.as_str() == "poke_name" => {
                let [
                    module,
                    ast::Expr {
                        kind: ast::ExprKind::Ident(poked),
                        ..
                    },
                ] = &directive.args[..]
                else {
                    return err(stmt.span, "#poke_name takes a module and a name");
                };
                self.pokes.push((module.clone(), *poked, file_scope));
            }
            // An unnamed `#library,link_always "c++";` is linked although nothing refers to it.
            ast::StmtKind::Directive(directive)
                if decls::is_library_directive(directive.name.name.as_str())
                    && directive
                        .flags
                        .iter()
                        .any(|f| f.name.as_str() == "link_always") =>
            {
                let ast::DirectiveStmt {
                    name,
                    flags,
                    args,
                } = &**directive;
                self.declare_library(
                    file_scope,
                    name.name.as_str(),
                    flags,
                    args.first(),
                    stmt.span,
                )?;
            }
            ast::StmtKind::Directive(_) | ast::StmtKind::Empty => {}
            ast::StmtKind::Using(boxed) => {
                let crate::ast::UsingStmt {
                    value,
                    filter,
                } = &**boxed;
                // `using Module;` / `using SomeStruct;` at file scope: resolved lazily.
                self.push_pending(
                    target,
                    Pending {
                        stmt: ast::Stmt {
                            kind: ast::StmtKind::Using(Box::new(ast::UsingStmt {
                                value: value.clone(),
                                filter: filter.clone(),
                            })),
                            span: stmt.span,
                            notes: Default::default(),
                        },
                        exported,
                        file_scope,
                        state: PendingState::Waiting,
                    },
                );
            }
            _ => return err(stmt.span, "this statement is not allowed at file scope"),
        }
        Ok(vis)
    }

    /// Apply `#poke_name Module name;`: the declarations called `name` visible at the directive
    /// also become declarations of `Module`, joining its overload set.
    pub fn apply_pokes(&mut self) -> Result<()> {
        while let Some((module_expr, name, file_scope)) = self.pokes.pop() {
            let lower::Operand::Module(module) = self.eval_const(file_scope, &module_expr, None)?
            else {
                return err(module_expr.span, "#poke_name needs a module");
            };
            let module_scope = self.modules[module.0 as usize].scope;
            // The module sees the same entity (a copy would resolve to a second,
            // distinct type).
            for id in self.lookup(file_scope, name)? {
                if !self
                    .scope(module_scope)
                    .names
                    .get(&name)
                    .is_some_and(|n| n.contains(&id))
                {
                    self.bind_name(module_scope, name, id);
                }
            }
        }
        Ok(())
    }

    fn declare_module_parameter(
        &mut self,
        scope: ScopeId,
        module: ModuleId,
        position: usize,
        param: &ast::Param,
        eval_scope: ScopeId,
    ) -> Result<()> {
        let Some(name) = param.name else {
            return err(param.span, "module parameter needs a name");
        };
        let provided = self.modules[module.0 as usize]
            .params
            .iter()
            .find(|(n, ..)| *n == name.name || n.as_str() == format!("${position}"))
            .map(|(_, v, ty)| (v.clone(), *ty));
        let decl = Rc::new(ast::Decl {
            id: ast::AstId::fresh(),
            names: vec![name],
            kind: ast::DeclKind::Const,
            // `X: $I/interface T = v` takes its type from the value.
            ty: param.ty.clone().filter(|t| !procs::has_poly(t)),
            value: param.default.clone(),
            using: false,
            as_: false,
            backtick: false,
            notes: ast::Notes::default(),
            span: param.span,
            rare: None,
        });
        let id = self.add_entity(
            scope,
            name.name,
            name.span,
            EntityKind::Decl {
                decl,
                index: 0,
            },
            false,
        );
        // Parameter storage belongs to the module; its declaration expressions
        // retain the lexical file environment, just like ordinary declarations.
        self.entity_mut(id).home = eval_scope;
        self.modules[module.0 as usize].param_entities.push(id);
        self.modules[module.0 as usize].arguments_validated = false;
        if let Some((Value::String(text), _)) = &provided
            && let Some(member) = std::str::from_utf8(text)
                .ok()
                .and_then(|t| t.strip_prefix(INFERRED_PARAM))
        {
            // Typecheck `.Member` against the parameter's declared type.
            let EntityKind::Decl {
                decl, ..
            } = &mut self.entity_mut(id).kind
            else {
                unreachable!()
            };
            let decl = Rc::make_mut(decl);
            // `P := Kind.A` has no written type: use the default's (`type_of(Kind.A)`).
            if decl.ty.is_none()
                && let Some(default) = decl.value.take()
            {
                let span = default.span;
                decl.ty = Some(ast::Expr {
                    kind: ast::ExprKind::Call {
                        callee: Box::new(ast::Expr {
                            kind: ast::ExprKind::Ident(Sym::intern("type_of")),
                            span,
                        }),
                        args: vec![ast::Arg {
                            name: None,
                            target: None,
                            context: false,
                            spread: false,
                            value: default,
                        }],
                        hint: ast::CallHint::None,
                    },
                    span,
                });
            }
            decl.value = Some(ast::Expr {
                kind: ast::ExprKind::InferredMember(ast::Ident {
                    name: Sym::intern(member),
                    span: param.span,
                }),
                span: param.span,
            });
            return Ok(());
        }
        if let Some((value, _)) = &provided
            && let Some(wanted) = literal_param_class(param)
            && let Some(given) = value_class(value)
            && wanted != given
            && !(wanted == "float" && given == "integer")
        {
            return err(
                param.span,
                format!(
                    "module parameter `{}` expects {wanted}, but the import gives {given}",
                    name.name
                ),
            );
        }
        if let Some((value, ty)) = provided {
            // Convert the original constant rather than reconstructing a literal AST:
            // reconstruction loses lexical precision, relocation identity and declared types.
            if let Some(target) = param.ty.as_ref().filter(|t| !procs::has_poly(t)) {
                let expected = self.eval_type(eval_scope, target)?;
                let actual = if ty == TypeId::VOID {
                    self.type_of_value(&value)
                } else {
                    ty
                };
                let mut f = self.thunk_ctx(ir::FuncOrigin::ConstInit, self.scope_file(eval_scope));
                let op = self.convert(
                    &mut f,
                    lower::Operand::Const {
                        ty: actual,
                        value,
                        untyped: ty == TypeId::VOID,
                    },
                    expected,
                    param.span,
                )?;
                let op = if op.is_const() {
                    op
                } else {
                    self.run_thunk(f, op, param.span)?
                };
                let Some(value) = op.const_value() else {
                    return err(
                        param.span,
                        "module parameter conversion did not produce a constant",
                    );
                };
                self.entity_mut(id).kind = EntityKind::Const {
                    value,
                    ty: expected,
                };
                return Ok(());
            }
            // A parameter without a written type is a constant like `NAME :: value`: given an
            // untyped number it stays untyped, so `MAX_FRAMES := 3` passes to a `u64`.
            let untyped = ty == TypeId::VOID
                && param.ty.is_none()
                && matches!(value, Value::Int(_) | Value::Float(_, _));
            let entity = self.entity_mut(id);
            entity.kind = EntityKind::Const {
                value,
                ty,
            };
            entity.untyped_const = untyped;
        }
        Ok(())
    }

    /// Hook for declaration notes (e.g. `@Something` on procedures); currently informational.
    fn note_declaration(&mut self, _id: EntityId, _decl: &ast::Decl) {
    }

    /// Expand one pending item (`#if`, top-level `#insert`, file-scope `using`).
    pub(super) fn expand_pending_item(
        &mut self,
        scope: ScopeId,
        file_scope: ScopeId,
        stmt: &ast::Stmt,
        exported: bool,
    ) -> Result<()> {
        let vis = if exported {
            ast::ScopeKind::Export
        } else if scope == file_scope {
            ast::ScopeKind::File
        } else {
            ast::ScopeKind::Module
        };
        match &stmt.kind {
            ast::StmtKind::StaticIf(boxed) => {
                let crate::ast::StaticIfStmt {
                    cond,
                    then_branch,
                    else_branch,
                } = &**boxed;
                let taken =
                    self.eval_static_condition(file_scope_for_eval(self, scope, file_scope), cond)?;
                let (branch, skipped) = if taken {
                    (then_branch, else_branch)
                } else {
                    (else_branch, then_branch)
                };
                self.ide_note_skipped(cond, skipped);
                self.declare_stmts(scope, file_scope, branch, vis)?;
            }
            ast::StmtKind::StaticSwitch(boxed) => {
                let crate::ast::SwitchStmt {
                    value,
                    cases,
                    ..
                } = &**boxed;
                let eval_scope = file_scope_for_eval(self, scope, file_scope);
                if let Some(case) = self.static_switch_case(eval_scope, value, cases)? {
                    self.declare_stmts(scope, file_scope, &case.body, vis)?;
                }
            }
            ast::StmtKind::Insert(boxed) => {
                let crate::ast::InsertStmt {
                    value, ..
                } = &**boxed;
                let stmts =
                    self.eval_insert_stmts(file_scope_for_eval(self, scope, file_scope), value)?;
                self.declare_stmts(scope, file_scope, &stmts, vis)?;
            }
            ast::StmtKind::Using(boxed) => {
                let crate::ast::UsingStmt {
                    value,
                    filter,
                } = &**boxed;
                let eval_scope = file_scope_for_eval(self, scope, file_scope);
                let entry = self.using_target(eval_scope, value)?;
                let entry = self.filter_using_entry(eval_scope, entry, filter, stmt.span)?;
                if exported
                    && entry.member_type().is_some()
                    && self.scope(scope).kind == ScopeKind::Module
                {
                    let module = self.scope(scope).module;
                    self.modules[module.0 as usize]
                        .exported_usings
                        .push(entry.clone());
                }
                self.push_using(scope, entry);
            }
            _ => unreachable!("only conditional items are pending"),
        }
        Ok(())
    }

    /// Resolve the module behind import entry `index` of `scope`, loading it on first use.
    pub fn import_module(&mut self, scope: ScopeId, index: usize) -> Result<Option<ModuleId>> {
        let known = &self.scope(scope).imports[index];
        if let Some(module) = known.module {
            return Ok(Some(module));
        }
        if known.loading {
            return Ok(None);
        }
        let entry = known.clone();
        self.scope_mut(scope).imports[index].loading = true;
        let resolved = (|| {
            let filter = self.resolve_name_filter(entry.from_scope, &entry.filter)?;
            self.scope_mut(scope).imports[index].filter = filter;
            self.resolve_import(entry.from_scope, &entry.import)
        })();
        self.scope_mut(scope).imports[index].loading = false;
        let module = resolved?;
        self.scope_mut(scope).imports[index].module = Some(module);
        self.ide_note_filter_names(&scope::UsingEntry::Module(module, None), &entry.filter);
        Ok(Some(module))
    }

    /// A failed import still contributes its diagnostic in editor analysis, but
    /// cannot hide independent names from the imports that did resolve. Direct
    /// import admission and normal compilation remain strict.
    pub(super) fn lookup_import(
        &mut self,
        scope: ScopeId,
        index: usize,
    ) -> Result<Option<ModuleId>> {
        match self.import_module(scope, index) {
            Err(error) if self.ide.is_some() => {
                let errors = &mut self.ide.as_mut().unwrap().errors;
                if !errors
                    .iter()
                    .any(|known| known.span == error.span && known.message == error.message)
                {
                    errors.push(*error);
                }
                Ok(None)
            }
            result => result,
        }
    }

    pub fn resolve_import(
        &mut self,
        from_scope: ScopeId,
        import: &ast::Import,
    ) -> Result<ModuleId> {
        let module = self.resolve_import_instance(from_scope, import)?;
        self.ide_note_module_arguments(module, import);
        Ok(module)
    }

    fn resolve_import_instance(
        &mut self,
        from_scope: ScopeId,
        import: &ast::Import,
    ) -> Result<ModuleId> {
        self.validate_import_program_origin(from_scope, import)?;
        let file = self.scope(from_scope).file.unwrap_or_default();
        let dir = self.file_dir(file);
        let mut params = Vec::new();
        for (position, arg) in import.params.iter().enumerate() {
            // `.Member` needs the parameter's type, known only once the module is read.
            let (value, ty) = match &arg.value.kind {
                ast::ExprKind::InferredMember(member) => (
                    Value::String(format!("{INFERRED_PARAM}{}", member.name).as_bytes().into()),
                    TypeId::VOID,
                ),
                _ => {
                    // Aggregates (`.[...]`), enums and typed numbers (`cast(u8) 3`) carry
                    // their type along. Untyped numbers (`3`, `N` for `N :: 3`) and other
                    // scalars get `VOID` and take their type from the value.
                    let (value, ty, untyped) = self.eval_const_literal(from_scope, &arg.value)?;
                    let typed = matches!(value, Value::Bytes(_))
                        || matches!(self.types.kind(ty), crate::types::TypeKind::Enum(_))
                        || (!untyped && matches!(value, Value::Int(_) | Value::Float(_, _)));
                    (
                        value,
                        if typed {
                            ty
                        } else {
                            TypeId::VOID
                        },
                    )
                }
            };
            let name = arg
                .name
                .map_or_else(|| Sym::intern(&format!("${position}")), |n| n.name);
            params.push((name, value, ty));
        }
        match &import.source {
            ast::ImportSource::Module(name)
                if &**name == "Runtime_Support" && self.runtime_support.is_some() =>
            {
                // One runtime per program; explicit arguments must describe that
                // configured instance rather than silently changing or ignoring it.
                let id = self.runtime_support.unwrap();
                self.validate_or_queue_import_groups(id, import)?;
                let entities = self.modules[id.0 as usize].param_entities.clone();
                for (name, value, _) in params {
                    let entity = if let Some(position) = name.as_str().strip_prefix('$') {
                        position
                            .parse::<usize>()
                            .ok()
                            .and_then(|p| entities.get(p).copied())
                    } else {
                        entities
                            .iter()
                            .copied()
                            .find(|id| self.entity(*id).name == name)
                    };
                    let Some(entity) = entity else {
                        return err(
                            import.span,
                            format!("argument `{name}` is not a parameter of this module"),
                        );
                    };
                    let expected = self.resolve_entity(entity)?;
                    if !matches!(expected, scope::Resolved::Const { value: ref existing, .. } if *existing == value)
                    {
                        return err(
                            import.span,
                            format!(
                                "argument `{name}` conflicts with the configured runtime instance"
                            ),
                        );
                    }
                }
                Ok(id)
            }
            ast::ImportSource::Module(name) => {
                let host = self.scope(from_scope).module;
                let host = self.modules[host.0 as usize].name.to_string();
                // `remap_import`: the latest rule for this importer (or any importer) wins.
                let (remaps, provided) = self.import_rules();
                let wanted: String = remaps
                    .iter()
                    .rev()
                    .find(|(h, n, _)| n == &**name && (h.is_empty() || *h == host))
                    .map_or_else(|| name.to_string(), |(_, _, r)| r.clone());
                if let Some(error) = self.ide_missing_module(from_scope, import.span, &wanted, &dir)
                {
                    // A newly supplied import wins even if filesystem discovery
                    // missed earlier in this analysis. Evaluate it normally.
                    if let Some(id) =
                        self.provided_module(&provided, &host, name, &dir, &params, import)?
                    {
                        return Ok(id);
                    }
                    self.failed_imports.push((host, name.to_string()));
                    return Err(error);
                }
                let Some(entry) = self.find_module(&wanted, &dir) else {
                    if let Some(id) =
                        self.provided_module(&provided, &host, name, &dir, &params, import)?
                    {
                        return Ok(id);
                    }
                    self.failed_imports.push((host, name.to_string()));
                    let error = self.module_not_found(&wanted, &dir, import.span);
                    self.ide_record_missing_module(from_scope, import.span, &wanted, &dir, &error);
                    return Err(Box::new(error));
                };
                self.load_module_request(&wanted, &entry, params, import.span, Some(import))
            }
            ast::ImportSource::File(path) | ast::ImportSource::Dir(path) => {
                let entry =
                    import_entry(&*self.fs, &self.options.import_paths, &import.source, &dir)
                        .unwrap_or_default();
                self.load_module_request(path, &entry, params, import.span, Some(import))
            }
            ast::ImportSource::String(source) => {
                let id = self.new_module("string", None, params);
                self.modules[id.0 as usize]
                    .argument_imports
                    .push(Rc::new(import.clone()));
                let label = format!("<import string {}>", self.sources.len());
                self.load_string(&label, source, id)?;
                self.validate_module_arguments(id, import.span)?;
                Ok(id)
            }
        }
    }

    /// PROGRAM arguments are workspace inputs, unlike ordinary module arguments.
    /// Use lexical module ownership: loaded/generated main files share the main
    /// module, whereas even anonymous or private imported scopes do not.
    fn validate_import_program_origin(&self, scope: ScopeId, import: &ast::Import) -> Result<()> {
        let supplies_program_arguments = import
            .program_params_start
            .is_some_and(|start| start < import.params.len());
        if supplies_program_arguments && Some(self.scope(scope).module) != self.main_module {
            return err(
                import.span,
                "PROGRAM parameters may only be supplied by the main program of this workspace",
            );
        }
        Ok(())
    }

    fn validate_or_queue_import_groups(
        &mut self,
        id: ModuleId,
        import: &ast::Import,
    ) -> Result<()> {
        let module = &mut self.modules[id.0 as usize];
        if !module.arguments_validated {
            // A recursive or deferred header has not registered its slots yet.
            // Retain each request for validation before that header binds values.
            module.argument_imports.push(Rc::new(import.clone()));
            return Ok(());
        }
        let module = &self.modules[id.0 as usize];
        let ordinary = module
            .param_entities
            .iter()
            .map(|&id| self.entity(id).name)
            .filter(|name| !module.program_params.contains(name))
            .collect::<Vec<_>>();
        self.validate_import_groups(import, &ordinary, &module.program_params)
    }

    /// Written groups bind only their declared category, including on cache hits.
    fn validate_import_groups(
        &self,
        import: &ast::Import,
        ordinary: &[Sym],
        program: &[Sym],
    ) -> Result<()> {
        let split = import.program_params_start.unwrap_or(import.params.len());
        for (args, names, label) in [
            (&import.params[..split], ordinary, "module"),
            (&import.params[split..], program, "program"),
        ] {
            if args.len() > names.len() {
                return err(
                    import.span,
                    format!(
                        "Too many {label} parameters: Wanted {}, got {}.",
                        names.len(),
                        args.len()
                    ),
                );
            }
            for arg in args {
                if let Some(name) = arg.name
                    && !names.contains(&name.name)
                {
                    let belongs_to_other_list =
                        ordinary.contains(&name.name) || program.contains(&name.name);
                    return err(
                        name.span,
                        if belongs_to_other_list {
                            format!(
                                "argument `{}` is not a {label} parameter of this module",
                                name.name
                            )
                        } else {
                            format!("argument `{}` is not a parameter of this module", name.name)
                        },
                    );
                }
            }
        }
        Ok(())
    }

    /// The module a metaprogram supplied with `provide_import` for `host` importing `name`.
    fn provided_module(
        &mut self,
        provided: &[ProvidedImport],
        host: &str,
        name: &str,
        dir: &Path,
        params: &[(Sym, Value, TypeId)],
        import: &ast::Import,
    ) -> Result<Option<ModuleId>> {
        let span = import.span;
        let Some(provided) = provided
            .iter()
            .rev()
            .find(|p| p.target == name && (p.host.is_empty() || p.host == host))
            .cloned()
        else {
            return Ok(None);
        };
        let entry = match provided.kind {
            ProvidedKind::ShortName => self.find_module(&provided.value, dir),
            ProvidedKind::PathToFile => Some(self.fs.canonical(Path::new(&provided.value))),
            ProvidedKind::PathToDirectory => Some(
                self.fs
                    .canonical(&Path::new(&provided.value).join("module.jai")),
            ),
            ProvidedKind::FullText => {
                if let Some(&id) = self.provided_texts.get(&provided.value) {
                    self.validate_or_queue_import_groups(id, import)?;
                    return Ok(Some(id));
                }
                let id = self.new_module(name, None, params.to_vec());
                self.modules[id.0 as usize]
                    .argument_imports
                    .push(Rc::new(import.clone()));
                let label = format!("<provided import {name}>");
                self.load_string(&label, &provided.value, id)?;
                self.validate_module_arguments(id, span)?;
                self.provided_texts.insert(provided.value.clone(), id);
                return Ok(Some(id));
            }
        };
        match entry {
            Some(entry) if self.fs.is_file(&entry) => Ok(Some(self.load_module_request(
                name,
                &entry,
                params.to_vec(),
                span,
                Some(import),
            )?)),
            _ => Ok(None),
        }
    }

    /// The `remap_import` and `provide_import` rules of this compiler's workspace.
    fn import_rules(&self) -> (Vec<(String, String, String)>, Vec<ProvidedImport>) {
        match &self.interp.workspaces {
            Some(w) => crate::build::import_rules(w, self.workspace),
            None => (Vec::new(), Vec::new()),
        }
    }

    /// A fresh module with no files yet.
    pub fn new_module(
        &mut self,
        name: &str,
        path: Option<PathBuf>,
        params: Vec<(Sym, Value, TypeId)>,
    ) -> ModuleId {
        let id = ModuleId(self.modules.len() as u32);
        let scope = self.new_scope(ScopeKind::Module, Some(self.root_scope), id, None);
        self.modules.push(Module {
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
            files: Vec::new(),
        });
        id
    }

    /// Add source text (an `#import,string` or a workspace build string) as a
    /// file of `module`. `label` names it in diagnostics.
    pub fn load_string(&mut self, label: &str, source: &str, module: ModuleId) -> Result<()> {
        let scope = self.modules[module.0 as usize].scope;
        self.load_string_in_scope(label, source, scope)
    }

    pub(super) fn load_string_in_scope(
        &mut self,
        label: &str,
        source: &str,
        target: ScopeId,
    ) -> Result<()> {
        self.interp.invalidate_callable_inspections();
        let module = self.scope(target).module;
        let path = PathBuf::from(label);
        let file = self.sources.add(label.to_string(), source.into());
        let mut origin_scope = Some(target);
        while let Some(scope) = origin_scope {
            if let Some(origin) = self.scope(scope).file {
                self.source_origins.insert(file, self.source_origin(origin));
                break;
            }
            origin_scope = self.scope(scope).parent;
        }
        let ast = crate::parser::parse_file(file, source).map_err(Box::new)?;
        self.dependencies.source_scopes.insert(file, target);
        let file_scope = self.new_scope(ScopeKind::File, Some(target), module, Some(file));
        self.files.push(FileInfo {
            id: file,
            path,
            module,
            scope: file_scope,
        });
        self.modules[module.0 as usize].files.push(file);
        self.declare_stmts(target, file_scope, &ast.stmts, ast::ScopeKind::Export)?;
        Ok(())
    }

    fn source_origin(&self, file: FileId) -> FileId {
        self.source_origins.get(&file).copied().unwrap_or(file)
    }

    /// Validate registered header contracts only after their declarations and
    /// program settings exist. A reentrant expansion does not restart the same
    /// contract; resolving a genuinely cyclic parameter still fails normally.
    fn validate_parameter_contracts(&mut self, module: ModuleId) -> Result<()> {
        if self.modules[module.0 as usize].loading {
            return Ok(());
        }
        self.apply_program_params(module);
        let mut index = 0;
        while index < self.modules[module.0 as usize].parameter_contracts.len() {
            let contract = &self.modules[module.0 as usize].parameter_contracts[index];
            match &contract.state {
                ParameterContractState::Checked | ParameterContractState::Checking => {
                    index += 1;
                    continue;
                }
                ParameterContractState::Failed(error) => return Err(error.clone()),
                ParameterContractState::Waiting => {}
            }
            let (entity, pattern, eval_scope) =
                (contract.entity, contract.pattern.clone(), contract.scope);
            self.modules[module.0 as usize].parameter_contracts[index].state =
                ParameterContractState::Checking;
            let result = (|| {
                let scope::Resolved::Const {
                    value,
                    ty,
                } = self.resolve_entity(entity)?
                else {
                    return err(pattern.span, "a module parameter must bind a constant");
                };
                let represented = if let Value::Type(t) = value {
                    Some(t)
                } else {
                    None
                };
                let actual = if ty == TypeId::VOID {
                    self.type_of_value(&value)
                } else {
                    ty
                };
                let mut bindings = Vec::new();
                self.match_pattern_value(&pattern, actual, &mut bindings, eval_scope, represented)?;
                let scope = self.modules[module.0 as usize].scope;
                for (name, value, ty) in bindings {
                    self.add_const(scope, name, pattern.span, value, ty);
                }
                Ok(())
            })();
            let contract = &mut self.modules[module.0 as usize].parameter_contracts[index];
            match result {
                Ok(()) => contract.state = ParameterContractState::Checked,
                Err(error) => {
                    contract.state = ParameterContractState::Failed(error.clone());
                    return Err(error);
                }
            }
            index += 1;
        }
        Ok(())
    }

    /// Load every module reachable through imports and expand all pending
    /// top-level items, until nothing changes.
    pub fn expand_all(&mut self) -> Result<()> {
        // First the plain `#if`s and imports everywhere (they decide which files and
        // `#add_context`s exist), then everything else.
        // Only unsettled module and file scopes are visited, in scope order (scopes made on the
        // way are visited too); a scope whose items and imports are all done has nothing to do.
        let mut next = 0;
        while let Some(sid) = self.unsettled.range(ScopeId(next)..).next().copied() {
            next = sid.0 + 1;
            self.validate_parameter_contracts(self.scope(sid).module)?;
            self.expand_plain_ifs(sid)?;
            for i in 0..self.scope(sid).imports.len() {
                self.import_module(sid, i)?;
            }
        }
        let mut next = 0;
        loop {
            while let Some(sid) = self.unsettled.range(ScopeId(next)..).next().copied() {
                next = sid.0 + 1;
                self.validate_parameter_contracts(self.scope(sid).module)?;
                self.expand_pending(sid)?;
                for i in 0..self.scope(sid).imports.len() {
                    self.import_module(sid, i)?;
                }
                if self.settled(sid) {
                    self.unsettled.remove(&sid);
                }
            }
            // Named imports (`X :: #import`) load eagerly too, so every module's
            // `#add_context` is known before the Context type is laid out. The mark moves past
            // an import only once it resolved: one that failed (in a nested call whose caller
            // went on) is tried, and reports its error, again.
            let mut index = self.named_imports_done;
            while index < self.entities.len() {
                let id = EntityId(index as u32);
                let e = self.entity(id);
                let required_library = match &e.kind {
                    EntityKind::Decl {
                        decl, ..
                    } => matches!(
                        decl.value.as_ref().map(|value| &value.kind),
                        Some(ast::ExprKind::UnknownDirective(directive))
                            if decls::is_library_directive(directive.name.name.as_str())
                                && directive.flags.iter().any(|flag| flag.name.as_str() == "link_always")
                    ),
                    _ => false,
                };
                // Required library metadata must exist before a #run native
                // lookup, just like unnamed link_always declarations. Resolving
                // a literal library name does not lower exports or freeze Context.
                if (matches!(e.kind, EntityKind::Import(_)) || required_library)
                    && matches!(
                        self.scope(e.scope).kind,
                        ScopeKind::Module | ScopeKind::File
                    )
                {
                    self.resolve_entity(id)?;
                }
                index += 1;
                self.named_imports_done = self.named_imports_done.max(index);
            }
            // Deferred items wait until no body is mid-lowering (an outer `expand_all`).
            let lowering = self
                .procs
                .iter()
                .any(|p| p.body_state == super::procs::BodyState::Lowering);
            if !self.deferred_pending.is_empty() && !lowering {
                let deferred = std::mem::take(&mut self.deferred_pending);
                self.deferred_errors.clear();
                let retrying = std::mem::replace(&mut self.retrying_pending, true);
                let result = deferred
                    .into_iter()
                    .try_for_each(|sid| self.expand_pending(sid));
                self.retrying_pending = retrying;
                result?;
                continue;
            }
            // Resolving a named import may have loaded a module: its scopes come next.
            if self.unsettled.range(ScopeId(next)..).next().is_none() {
                let mut index = 0;
                while index < self.modules.len() {
                    if !self.modules[index].loading {
                        self.validate_module_arguments(ModuleId(index as u32), Span::NONE)?;
                        self.validate_parameter_contracts(ModuleId(index as u32))?;
                    }
                    index += 1;
                }
                // A contract may import its declaring interface or a hook type.
                // Newly registered scopes must participate in normal expansion.
                if self.unsettled.range(ScopeId(next)..).next().is_some() {
                    continue;
                }
                return Ok(());
            }
        }
    }
}

/// Conditions inside a module-level pending item are evaluated from the file scope
/// that wrote them, so file-private names are visible.
pub(super) fn file_scope_for_eval(c: &Compiler, scope: ScopeId, file_scope: ScopeId) -> ScopeId {
    if c.scope(scope).kind == ScopeKind::Module {
        file_scope
    } else {
        scope
    }
}

/// `name :: value;`, a synthesized constant declaration.
fn const_alias(name: ast::Ident, value: ast::Expr) -> Rc<ast::Decl> {
    Rc::new(ast::Decl {
        id: ast::AstId::fresh(),
        names: vec![name],
        kind: ast::DeclKind::Const,
        ty: None,
        span: value.span,
        value: Some(value),
        using: false,
        as_: false,
        backtick: false,
        notes: ast::Notes::default(),
        rare: None,
    })
}

/// What a module parameter holds, from its written type or its literal default: `bool`,
/// `string`, `integer` or `float`. Other parameters are not checked here.
fn literal_param_class(param: &ast::Param) -> Option<&'static str> {
    use ast::ExprKind as E;
    if let Some(E::Ident(t)) = param.ty.as_ref().map(|t| &t.kind) {
        return match t.as_str() {
            "bool" => Some("bool"),
            "string" => Some("string"),
            "int" | "s8" | "s16" | "s32" | "s64" | "u8" | "u16" | "u32" | "u64" => Some("integer"),
            "float" | "float32" | "float64" => Some("float"),
            _ => None,
        };
    }
    if param.ty.is_some() {
        return None;
    }
    let mut default = param.default.as_ref()?;
    while let E::Unary(ast::UnOp::Neg | ast::UnOp::Plus, inner) = &default.kind {
        default = inner;
    }
    match default.kind {
        E::Bool(_) => Some("bool"),
        E::Str(_) => Some("string"),
        E::Int(_) => Some("integer"),
        E::Float(_) => Some("float"),
        _ => None,
    }
}

fn value_class(value: &Value) -> Option<&'static str> {
    match value {
        Value::Bool(_) => Some("bool"),
        Value::String(_) => Some("string"),
        Value::Int(_) => Some("integer"),
        Value::Float(_, _) => Some("float"),
        _ => None,
    }
}
