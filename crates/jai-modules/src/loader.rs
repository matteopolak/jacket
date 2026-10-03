mod bootstrap_imports;
mod callable_aliases;
mod collisions;
mod declaration_insertions;
mod insertion_admission;
mod session;
mod suspended_imports;
use super::*;
use crate::params::{Argument, ModuleKey};
use jai_source::Identities;
use jai_syntax::{FileItem, ImportDeclaration, ImportMode};
use std::collections::HashSet;
type ParameterRequests = (Option<Vec<Argument>>, Option<Vec<Argument>>);
#[derive(Clone)]
pub(super) struct Builder<'a> {
    provider: &'a dyn SourceProvider,
    pub(super) graph: ModuleGraph,
    pub(super) conditions: Vec<DeferredCondition>,
    pub(super) cases: Vec<DeferredCase>,
    pub(super) active_specialization: Option<SourceSpecializationKey>,
    pub(super) pending_specializations: Vec<SourceSpecializationKey>,
    pub(super) using_requests: crate::using_requests::UsingRequestStore,
    pub(super) insertion_requests: crate::declaration_insertions::InsertionRequestStore,
    pending_inserted_modules: Vec<ModuleId>,
    pub(super) semantic_parameters:
        std::cell::RefCell<crate::parameter_requests::ParameterRequestStore>,
    identities: Identities,
    options: GraphOptions,
    sources: HashMap<PathBuf, SourceId>,
    syntax: HashMap<SourceId, ParsedFile>,
    modules: HashMap<ModuleKey, ModuleId>,
    requests: HashMap<ModuleId, ParameterRequests>,
    program: HashMap<PathBuf, Option<Vec<Argument>>>,
    pending: HashMap<ModuleId, Vec<(FileInstanceId, PathBuf, FileItem)>>,
    initialized_program: HashSet<PathBuf>,
    completed_modules: HashSet<ModuleId>,
    entry_modules: Vec<(ModuleId, PathBuf)>,
    initialized_files: HashSet<FileInstanceId>,
    file_origins: HashMap<FileInstanceId, Option<SourceSpan>>,
    pending_diagnostics: Vec<DeferredDependency>,
    first_import: HashMap<PathBuf, SourceSpan>,
    pending_imports: HashMap<PathBuf, SourceSpan>,
    files: HashMap<(ModuleId, PathBuf), FileInstanceId>,
    active_modules: HashSet<ModuleId>,
    active_files: HashSet<(ModuleId, PathBuf)>,
    callable_aliases: Vec<callable_aliases::DeferredAlias>,
}
impl<'a> Builder<'a> {
    pub(super) fn new(options: GraphOptions, provider: &'a dyn SourceProvider) -> Self {
        Self::new_with_target(options, provider, None)
    }
    pub(super) fn new_with_target(
        options: GraphOptions,
        provider: &'a dyn SourceProvider,
        target: Option<jai_types::BuildTarget>,
    ) -> Self {
        let mut identities = Identities::default();
        let unit = identities.unit();
        let root = identities.module();
        let scope = identities.scope();
        Self {
            provider,
            graph: ModuleGraph {
                unit,
                target,
                root,
                prelude: None,
                runtime_support: None,
                sources: SourceMap::default(),
                symbols: Symbols::default(),
                files: vec![],
                modules: vec![ModuleInstance {
                    id: root,
                    scope,
                    entry: None,
                    files: vec![],
                    bindings: HashMap::new(),
                    exports: HashMap::new(),
                }],
                declarations: vec![],
                placeholders: Default::default(),
                imports: vec![],
                scoped_imports: vec![],
                source_conditions: vec![],
                source_cases: vec![],
                dependency_templates: vec![],
                source_specializations: vec![],
                using_publications: vec![],
                storage_members: Default::default(),
                loads: vec![],
                runs: vec![],
                insertions: vec![],
                insertion_publications: vec![],
                context_fields: vec![],
                parameters: vec![],
                source_requests: HashMap::new(),
                overload_sets: vec![],
            },
            identities,
            conditions: vec![],
            cases: vec![],
            active_specialization: None,
            pending_specializations: vec![],
            using_requests: crate::using_requests::UsingRequestStore::default(),
            insertion_requests: crate::declaration_insertions::InsertionRequestStore::default(),
            pending_inserted_modules: vec![],
            semantic_parameters: std::cell::RefCell::default(),
            options,
            sources: HashMap::new(),
            syntax: HashMap::new(),
            modules: HashMap::new(),
            requests: HashMap::new(),
            program: HashMap::new(),
            pending: HashMap::new(),
            initialized_program: HashSet::new(),
            completed_modules: HashSet::new(),
            entry_modules: vec![],
            initialized_files: HashSet::new(),
            file_origins: HashMap::new(),
            pending_diagnostics: vec![],
            first_import: HashMap::new(),
            pending_imports: HashMap::new(),
            files: HashMap::new(),
            active_modules: HashSet::new(),
            active_files: HashSet::new(),
            callable_aliases: Vec::new(),
        }
    }
    fn canonical(&self, path: &Path) -> Result<PathBuf, GraphError> {
        self.provider
            .canonicalize(path)
            .map_err(|cause| GraphError::Io {
                path: path.to_owned(),
                cause,
            })
    }
    pub(super) fn located(&self, location: SourceSpan, message: impl Into<String>) -> GraphError {
        let diagnostic = self.graph.diagnostic(location, message);
        let rendered = diagnostic.render(&self.graph.sources);
        GraphError::Located {
            diagnostic,
            rendered,
        }
    }
    fn cycle(
        &self,
        kind: DependencyKind,
        path: PathBuf,
        location: Option<SourceSpan>,
    ) -> GraphError {
        let directive = match kind {
            DependencyKind::Load => "#load",
            DependencyKind::Import => "#import",
        };
        let message = format!("cyclic {directive} dependency on {}", path.display());
        let rendered = if let Some(location) = location {
            self.graph
                .diagnostic(location, &message)
                .render(&self.graph.sources)
        } else {
            message
        };
        GraphError::Cycle {
            kind,
            path,
            location,
            rendered,
        }
    }
    fn parse(&mut self, path: &Path) -> Result<ParsedFile, GraphError> {
        if let Some(id) = self.sources.get(path) {
            return Ok(self.syntax[id].clone());
        }
        let bytes = self.provider.read(path).map_err(|cause| GraphError::Io {
            path: path.to_owned(),
            cause,
        })?;
        let text = jai_lexer::decode_source(&bytes)
            .map_err(|diagnostic| GraphError::Decode {
                path: path.to_owned(),
                diagnostic,
            })?
            .into_owned();
        let id = self.graph.sources.insert(path.to_owned(), text);
        let file =
            jai_syntax::parse_file(self.graph.sources.get(id).unwrap(), &mut self.graph.symbols)
                .map_err(|diagnostic| {
                    let rendered = diagnostic.render(&self.graph.sources);
                    GraphError::Located {
                        diagnostic,
                        rendered,
                    }
                })?;
        self.sources.insert(path.to_owned(), id);
        self.syntax.insert(id, file.clone());
        Ok(file)
    }
    fn expand_module(&mut self, module: ModuleId, path: &Path) -> Result<(), GraphError> {
        // A module has its own bound request context. A caller's procedure
        // specialization must not become this module's source identity.
        let specialization = self.active_specialization.take();
        self.active_modules.insert(module);
        let result = self.module_files(module, path);
        self.active_modules.remove(&module);
        self.active_specialization = specialization;
        if result.is_ok() {
            self.completed_modules.insert(module);
        }
        result
    }
    fn module_files(&mut self, module: ModuleId, path: &Path) -> Result<(), GraphError> {
        let mut imports = Vec::new();
        let entry = self.file(module, path, None, &mut imports)?;
        self.graph.modules[module.index()].entry = Some(entry);
        loop {
            let mut progressed = false;
            let mut initialization_pending = None;
            self.prepare_callable_aliases(module)?;
            for file in self.graph.modules[module.index()].files.clone() {
                let before = (self.initialized_files.len(), self.graph.parameters.len());
                match self.initialize_file(file) {
                    Ok(()) => {}
                    Err(error @ GraphError::Pending { .. }) => {
                        initialization_pending.get_or_insert(error);
                    }
                    Err(error) => return Err(error),
                }
                progressed |= before != (self.initialized_files.len(), self.graph.parameters.len());
            }
            let mut work: std::collections::VecDeque<_> =
                self.pending.remove(&module).unwrap_or_default().into();
            if work.is_empty() {
                if let Some(error) = initialization_pending {
                    return Err(error);
                }
                break;
            }
            let mut deferred = Vec::new();
            let mut first_pending = None;
            while let Some((file, path, item)) = work.pop_front() {
                match self.dependencies(file, &path, std::slice::from_ref(&item), &mut imports) {
                    Ok(()) => {
                        progressed = true;
                        if let Some(added) = self.pending.remove(&module) {
                            for dependency in added.into_iter().rev() {
                                work.push_front(dependency);
                            }
                        }
                    }
                    Err(error @ GraphError::Pending { .. }) => {
                        first_pending.get_or_insert(error);
                        deferred.push((file, path, item));
                    }
                    Err(error) => return Err(error),
                }
            }
            self.pending.entry(module).or_default().extend(deferred);
            if !progressed {
                return Err(first_pending.expect("no progress requires a pending dependency"));
            }
        }
        self.finish_callable_aliases(module)
    }
    fn file(
        &mut self,
        module: ModuleId,
        path: &Path,
        origin: Option<SourceSpan>,
        _imports: &mut Vec<(FileInstanceId, ImportDeclaration)>,
    ) -> Result<FileInstanceId, GraphError> {
        let path = self.canonical(path)?;
        let key = (module, path.clone());
        if self.active_files.contains(&key) {
            return Err(self.cycle(DependencyKind::Load, path, origin));
        }
        if let Some(&file) = self.files.get(&key) {
            return Ok(file);
        }
        self.active_files.insert(key.clone());
        let syntax = self.parse(&path)?;
        let id = FileInstanceId(self.graph.files.len());
        self.graph.files.push(FileInstance {
            id,
            module,
            source: syntax.source(),
            scope: self.identities.scope(),
            private: HashMap::new(),
            declarations: vec![],
            syntax: syntax.clone(),
        });
        self.graph.modules[module.index()].files.push(id);
        self.files.insert(key.clone(), id);
        let parameters = syntax.items().iter().find_map(|item| {
            if let FileItem::Parameters(p) = item {
                Some(p)
            } else {
                None
            }
        });
        if let Some(parameters) = parameters {
            if self.graph.modules[module.index()].files[0] != id {
                return Err(self.located(
                    parameters.location,
                    "#module_parameters must occur in the module entry file",
                ));
            }
            self.register_declarations(id, &parameters.declarations)?;
        }
        self.register_declarations(id, syntax.items())?;
        self.prepare_callable_aliases(module)?;
        self.pending.entry(module).or_default().extend(
            syntax
                .items()
                .iter()
                .cloned()
                .map(|item| (id, path.clone(), item)),
        );
        self.file_origins.insert(id, origin);
        self.active_files.remove(&key);
        match self.initialize_file(id) {
            Ok(()) | Err(GraphError::Pending { .. }) => Ok(id),
            Err(error) => Err(error),
        }
    }
    fn initialize_file(&mut self, id: FileInstanceId) -> Result<(), GraphError> {
        if self.initialized_files.contains(&id) {
            return Ok(());
        }
        let file = &self.graph.files[id.index()];
        let module = file.module;
        let source = file.source;
        let syntax = file.syntax.clone();
        let path = self.graph.sources.get(source).unwrap().path().to_owned();
        let parameters = syntax.items().iter().find_map(|item| {
            if let FileItem::Parameters(parameters) = item {
                Some(parameters)
            } else {
                None
            }
        });
        if let Some(parameters) = parameters {
            let (instance, program) = self.requests.get(&module).cloned().unwrap_or_default();
            self.bind_parameters(id, parameters, instance.as_deref(), program.as_deref())?;
            if self.initialized_program.insert(path.clone()) {
                let bound = self
                    .graph
                    .parameters
                    .iter()
                    .filter(|p| p.module == module && p.program_wide)
                    .map(|p| Argument {
                        name: Some(p.name),
                        value: p.value.clone(),
                    })
                    .collect();
                self.program.insert(path, Some(bound));
            }
        } else if self.graph.modules[module.index()].files[0] == id
            && self.requests.get(&module).is_some_and(|(a, b)| {
                a.as_ref().is_some_and(|v| !v.is_empty())
                    || b.as_ref().is_some_and(|v| !v.is_empty())
            })
        {
            return Err(self.located(
                self.file_origins
                    .get(&id)
                    .copied()
                    .flatten()
                    .unwrap_or(SourceSpan {
                        source,
                        span: jai_source::Span::default(),
                    }),
                "arguments supplied to a module with no #module_parameters",
            ));
        }
        self.initialized_files.insert(id);
        Ok(())
    }
    fn register_declarations(
        &mut self,
        file: FileInstanceId,
        items: &[FileItem],
    ) -> Result<(), GraphError> {
        for item in items {
            let declaration = match item {
                FileItem::Declaration(syntax)
                | FileItem::UsingDeclaration {
                    declaration: syntax,
                    ..
                } => Some(syntax),
                _ => None,
            };
            if let Some(syntax) = declaration {
                if let FileDeclarationKind::Placeholder(marker) = &syntax.kind {
                    self.reserve_placeholder(
                        file,
                        marker.name,
                        syntax.visibility,
                        syntax.location,
                    )?;
                    continue;
                }
                let declaration = Declaration {
                    id: self.identities.declaration(),
                    file,
                    syntax: syntax.clone(),
                };
                let id = declaration.id;
                let name = declaration.name();
                let discarded_using_owner = matches!(item, FileItem::UsingDeclaration { .. })
                    && self.graph.symbols.name(name) == "_";
                self.graph.declarations.push(declaration);
                if !discarded_using_owner
                    && !matches!(&syntax.kind, FileDeclarationKind::Procedure(procedure) if procedure.operator.is_some())
                    && !matches!(&syntax.kind, FileDeclarationKind::OperatorAlias(_))
                {
                    self.bind(
                        file,
                        syntax.visibility,
                        name,
                        Binding::Declaration(id),
                        syntax.location,
                        false,
                    )?;
                }
                self.graph.files[file.0].declarations.push(id);
                if let Some(publication) = self
                    .graph
                    .insertion_publications
                    .iter_mut()
                    .find(|publication| publication.file == file)
                {
                    publication.declarations.push(id);
                }
            }
        }
        Ok(())
    }
    fn dependencies(
        &mut self,
        file: FileInstanceId,
        path: &Path,
        items: &[FileItem],
        imports: &mut Vec<(FileInstanceId, ImportDeclaration)>,
    ) -> Result<(), GraphError> {
        for item in items {
            let module = self.graph.files[file.index()].module;
            self.prepare_callable_aliases(module)?;
            match item {
                FileItem::Load(load) => {
                    let module = self.graph.files[file.0].module;
                    let target = self.file(
                        module,
                        &path.parent().unwrap().join(&load.target),
                        Some(load.location),
                        imports,
                    )?;
                    let mut reachable = vec![target];
                    let mut visited = HashSet::new();
                    while let Some(candidate) = reachable.pop() {
                        if candidate == file {
                            return Err(self.cycle(
                                DependencyKind::Load,
                                self.graph
                                    .sources
                                    .get(self.graph.files[target.0].source)
                                    .unwrap()
                                    .path()
                                    .to_owned(),
                                Some(load.location),
                            ));
                        }
                        if visited.insert(candidate) {
                            reachable.extend(
                                self.graph
                                    .loads
                                    .iter()
                                    .filter(|edge| edge.file == candidate)
                                    .map(|edge| edge.target),
                            );
                        }
                    }
                    self.graph.loads.push(LoadEdge {
                        file,
                        target,
                        location: load.location,
                    });
                }
                FileItem::Import(import) => self.import(file, import)?,
                FileItem::Using {
                    directive,
                    visibility,
                    location,
                } => {
                    self.defer_using(
                        file,
                        directive,
                        *visibility,
                        *location,
                        DiscoveryConditionContext::File,
                    )?;
                }
                FileItem::UsingDeclaration {
                    declaration,
                    location,
                    ..
                } => {
                    self.scoped_declaration(file, declaration)?;
                    let directive = item.using_declaration_directive().ok_or_else(|| {
                        self.located(*location, "using requires one named declaration")
                    })?;
                    self.defer_using_declaration(
                        file,
                        &directive,
                        declaration.visibility,
                        *location,
                        DiscoveryConditionContext::File,
                        UsingDeclarationSource::File(Box::new(declaration.clone())),
                    )?;
                }
                FileItem::CompileTimeCases { cases, location } => {
                    let choice = match self.selected_case(file, cases.span) {
                        Some(choice) => choice,
                        None => match self.constant_case(file, &cases.header()) {
                            Ok(choice) => {
                                self.record_case_selection(
                                    file,
                                    *location,
                                    self.specialization_for_span(file, cases.span).cloned(),
                                    choice,
                                    SourceConditionOrigin::Scalar,
                                );
                                choice
                            }
                            Err(GraphError::Pending { .. } | GraphError::Unsupported { .. }) => {
                                return Err(self.defer_case(
                                    file,
                                    &cases.header(),
                                    DiscoveryConditionContext::File,
                                ));
                            }
                            Err(error) => return Err(error),
                        },
                    };
                    let selected = cases.selected_body(choice).ok_or_else(|| {
                        self.located(
                            *location,
                            "compile-time case selection has no valid fallthrough body",
                        )
                    })?;
                    self.register_declarations(file, &selected)?;
                    let module = self.graph.files[file.0].module;
                    self.pending.entry(module).or_default().extend(
                        selected
                            .into_iter()
                            .map(|item| (file, path.to_owned(), item)),
                    );
                }
                FileItem::Conditional {
                    condition,
                    then_items,
                    else_items,
                    location: _,
                } => {
                    let decision = if let Some(selected) =
                        self.selected_condition(file, condition.span)
                    {
                        selected
                    } else {
                        let value = match self.constant_expression(file, condition, &mut Vec::new())
                        {
                            Ok(value) => value,
                            Err(GraphError::Pending { .. } | GraphError::Unsupported { .. }) => {
                                return Err(self.defer_condition(
                                    file,
                                    condition,
                                    DiscoveryConditionContext::File,
                                ));
                            }
                            Err(error) => return Err(error),
                        };
                        match value {
                            jai_eval::Value::Bool(v) => v,
                            jai_eval::Value::Literal(v) => v != 0,
                            jai_eval::Value::Int(v) => v.value() != 0,
                            jai_eval::Value::Float(v) => v.to_f64() != 0.0,
                            jai_eval::Value::WeakFloat(v) => {
                                v.round(v.default_type(), condition.span)
                                    .map_err(|d| {
                                        self.located(
                                            SourceSpan {
                                                source: self.graph.files[file.index()].source,
                                                span: d.span,
                                            },
                                            d.message,
                                        )
                                    })?
                                    .to_f64()
                                    != 0.0
                            }
                        }
                    };
                    self.record_selection(file, condition.span, decision);
                    let selected = if decision { then_items } else { else_items };
                    self.register_declarations(file, selected)?;
                    let module = self.graph.files[file.0].module;
                    self.pending.entry(module).or_default().extend(
                        selected
                            .iter()
                            .cloned()
                            .map(|item| (file, path.to_owned(), item)),
                    );
                }
                FileItem::Parameters(parameters) => {
                    if !self.graph.files[file.0].syntax.items().iter().any(|item| matches!(item, FileItem::Parameters(p) if p.location == parameters.location)) {
                        return Err(self.located(parameters.location, "#module_parameters cannot occur in a conditional or nested declaration block"));
                    }
                    let module = self.graph.files[file.0].module;
                    self.pending.entry(module).or_default().extend(
                        parameters
                            .declarations
                            .iter()
                            .cloned()
                            .map(|item| (file, path.to_owned(), item)),
                    );
                }
                FileItem::ContextField {
                    declaration,
                    location,
                } => self.graph.context_fields.push(ContextField {
                    file,
                    syntax: declaration.clone(),
                    location: *location,
                }),
                FileItem::Insert {
                    directive,
                    location,
                } => self.defer_insertion(file, directive, *location),
                FileItem::Run(syntax) => self.graph.runs.push(FileRun {
                    file,
                    syntax: syntax.clone(),
                }),
                FileItem::Declaration(declaration) => self.scoped_declaration(file, declaration)?,
                FileItem::Assert { .. } | FileItem::Scope { .. } => {}
            }
        }
        Ok(())
    }
    pub(super) fn bind(
        &mut self,
        file: FileInstanceId,
        visibility: Visibility,
        name: Symbol,
        binding: Binding,
        location: SourceSpan,
        idempotent: bool,
    ) -> Result<(), GraphError> {
        self.validate_placeholder_binding(file, visibility, name, binding, location, idempotent)?;
        let module = self.graph.files[file.0].module;
        let previous = if visibility == Visibility::File {
            self.graph.files[file.0].private.get(&name).copied()
        } else {
            self.graph.modules[module.index()]
                .bindings
                .get(&name)
                .copied()
        };
        let destination = if visibility == Visibility::File {
            callable_aliases::Destination::File(file)
        } else {
            callable_aliases::Destination::Module(module)
        };
        self.merge_callable_binding(destination, previous, binding, location, name, idempotent)?;
        if visibility == Visibility::Export {
            // Merge only exported declarations. The module's visible overload
            // set may also include module-private members.
            let previous = self.graph.modules[module.index()]
                .exports
                .get(&name)
                .copied();
            self.merge_callable_binding(
                callable_aliases::Destination::Exports(module),
                previous,
                binding,
                location,
                name,
                idempotent,
            )?;
        }
        self.insertion_binding(file, visibility, name, binding, location)
    }
    fn merge_binding(
        &mut self,
        previous: Option<Binding>,
        binding: Binding,
        location: SourceSpan,
        name: Symbol,
        idempotent: bool,
    ) -> Result<Binding, GraphError> {
        let Some(previous) = previous else {
            return Ok(binding);
        };
        if idempotent && previous == binding {
            return Ok(previous);
        }
        let left = self.procedure_declarations(previous);
        let right = self.procedure_declarations(binding);
        if let (Some(mut members), Some(right)) = (left, right) {
            members.extend(right);
            members.sort_unstable_by_key(|id| id.index());
            members.dedup();
            if let Some(existing) = self
                .graph
                .overload_sets
                .iter()
                .find(|set| set.declarations.as_ref() == members.as_slice())
            {
                return Ok(Binding::OverloadSet(existing.id));
            }
            let id = OverloadSetId(self.graph.overload_sets.len());
            self.graph.overload_sets.push(OverloadSet {
                id,
                declarations: members.into_boxed_slice(),
            });
            return Ok(Binding::OverloadSet(id));
        }
        Err(self.binding_collision(previous, binding, name, location))
    }
    fn procedure_declarations(&self, binding: Binding) -> Option<Vec<DeclarationId>> {
        let members = match binding {
            Binding::Declaration(id) => vec![id],
            Binding::OverloadSet(id) => self.graph.overload_sets.get(id.0)?.declarations.to_vec(),
            _ => return None,
        };
        members
            .iter()
            .all(|id| {
                self.graph.declaration(*id).is_some_and(|declaration| {
                    matches!(
                        declaration.syntax.kind,
                        FileDeclarationKind::Procedure(_)
                            | FileDeclarationKind::ProcedurePrototype(_)
                    )
                })
            })
            .then_some(members)
    }
    fn import_path(
        &self,
        file: FileInstanceId,
        import: &ImportDeclaration,
    ) -> Result<PathBuf, GraphError> {
        let source = self
            .graph
            .sources
            .get(self.graph.files[file.0].source)
            .unwrap();
        let parent = source.path().parent().unwrap();
        match import.mode {
            ImportMode::File => self
                .canonical(&parent.join(&import.target))
                .map_err(|error| self.located(import.location, error.to_string())),
            ImportMode::Directory => self
                .canonical(&parent.join(&import.target).join("module.jai"))
                .map_err(|error| self.located(import.location, error.to_string())),
            ImportMode::Search => {
                for directory in &self.options.import_dirs {
                    for candidate in [
                        directory.join(format!("{}.jai", import.target)),
                        directory.join(&import.target).join("module.jai"),
                    ] {
                        if self.provider.is_file(&candidate) {
                            return self.canonical(&candidate);
                        }
                    }
                }
                Err(self.located(
                    import.location,
                    format!(
                        "module '{}' not found in configured import directories",
                        import.target
                    ),
                ))
            }
            ImportMode::String => Err(self.located(
                import.location,
                "string imports are not implemented in the module graph",
            )),
        }
    }
    fn import(
        &mut self,
        file: FileInstanceId,
        import: &ImportDeclaration,
    ) -> Result<(), GraphError> {
        let module = self.import_module_with_scope(
            file,
            import,
            suspended_imports::ImportBindingScope::File,
        )?;
        self.publish_import(file, import, module)
    }
    pub(super) fn import_module(
        &mut self,
        file: FileInstanceId,
        import: &ImportDeclaration,
    ) -> Result<ModuleId, GraphError> {
        self.import_module_with_scope(file, import, suspended_imports::ImportBindingScope::Lexical)
    }
    fn import_module_with_scope(
        &mut self,
        file: FileInstanceId,
        import: &ImportDeclaration,
        scope: suspended_imports::ImportBindingScope,
    ) -> Result<ModuleId, GraphError> {
        if let Some(module) = self.shared_prelude_import(import)? {
            return Ok(module);
        }
        let path = self.import_path(file, import)?;
        if self
            .modules
            .iter()
            .any(|(key, module)| key.path == path && self.active_modules.contains(module))
        {
            return Err(self.cycle(DependencyKind::Import, path, Some(import.location)));
        }
        self.first_import
            .entry(path.clone())
            .or_insert(import.location);
        if self
            .pending_imports
            .get(&path)
            .is_some_and(|location| *location != import.location)
        {
            let diagnostic = self.graph.diagnostic(
                import.location,
                "earlier import of this module is pending compile-time arguments or dependencies",
            );
            let rendered = diagnostic.render(&self.graph.sources);
            return Err(GraphError::Pending {
                diagnostic,
                rendered,
            });
        }
        self.pending_imports.insert(path.clone(), import.location);
        let arguments =
            self.evaluate_arguments(file, &import.arguments.instance, import.location)?;
        let supplied_program =
            self.evaluate_arguments(file, &import.arguments.program, import.location)?;
        if supplied_program.is_some()
            && self.program.contains_key(&path)
            && self.first_import.get(&path) != Some(&import.location)
        {
            return Err(self.located(import.location, "program module parameters may be supplied once, before every other import of that module"));
        }
        if supplied_program.is_some() && self.graph.files[file.0].module != self.graph.root {
            return Err(self.located(
                import.location,
                "program module parameters must be set by the application module",
            ));
        }
        let program = self
            .program
            .entry(path.clone())
            .or_insert_with(|| supplied_program.clone())
            .clone();
        let key = ModuleKey {
            path: path.clone(),
            arguments: arguments.clone(),
        };
        let module = if let Some(&module) = self.modules.get(&key) {
            if self.active_modules.contains(&module) {
                return Err(self.cycle(DependencyKind::Import, path, Some(import.location)));
            }
            module
        } else {
            if self
                .modules
                .iter()
                .any(|(key, module)| key.path == path && self.active_modules.contains(module))
            {
                return Err(self.cycle(DependencyKind::Import, path, Some(import.location)));
            }
            let module = self.identities.module();
            self.graph.modules.push(ModuleInstance {
                id: module,
                scope: self.identities.scope(),
                entry: None,
                files: vec![],
                bindings: HashMap::new(),
                exports: HashMap::new(),
            });
            self.graph.source_requests.insert(module, key.clone());
            self.modules.insert(key, module);
            self.requests.insert(module, (arguments, program));
            module
        };
        if !self.completed_modules.contains(&module) {
            self.expand_import_module(file, import, module, &path, scope)?;
        }
        self.pending_imports.remove(&path);
        Ok(module)
    }
    fn bind_import(
        &mut self,
        file: FileInstanceId,
        import: &ImportDeclaration,
        module: ModuleId,
    ) -> Result<(), GraphError> {
        if let Some(name) = import.namespace {
            self.bind(
                file,
                import.visibility,
                name,
                Binding::Module(module),
                import.location,
                true,
            )?;
        }
        if import.namespace.is_none() || import.using {
            let mut exports: Vec<_> = self.graph.modules[module.index()]
                .exports
                .iter()
                .map(|(&name, &binding)| (name, binding))
                .collect();
            exports.sort_by(|(left, _), (right, _)| {
                self.graph
                    .symbols
                    .name(*left)
                    .cmp(self.graph.symbols.name(*right))
            });
            for (name, binding) in exports {
                self.bind(
                    file,
                    import.visibility,
                    name,
                    binding,
                    import.location,
                    true,
                )?;
            }
            for (name, placeholder) in self.graph.module_placeholder_exports(module) {
                self.link_placeholder_import(
                    file,
                    import.visibility,
                    name,
                    placeholder,
                    import.location,
                )?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_marker_replay_precedes_collision_with_a_real_filler_from_another_source() {
        let path = std::path::Path::new("/jai-placeholder-replay/main.jai");
        let mut overlay = SourceOverlay::new();
        overlay
            .insert(path, b"#placeholder ANSWER;".to_vec())
            .unwrap();
        let mut discovery = GraphDiscovery::new(path, GraphOptions::default(), &overlay).unwrap();
        discovery.advance().unwrap();
        let marker = discovery.graph().placeholders()[0].clone();
        assert!(discovery.graph().declarations().is_empty());
        let builder = &mut discovery.builder;
        let source = builder.graph.sources.insert(
            "/jai-placeholder-replay/generated.jai".into(),
            "ANSWER::42;".into(),
        );
        assert_ne!(source, marker.location().source);
        let parsed = jai_syntax::parse_file(
            builder.graph.sources.get(source).unwrap(),
            &mut builder.graph.symbols,
        )
        .unwrap();
        let jai_syntax::FileItem::Declaration(syntax) = &parsed.items()[0] else {
            panic!()
        };
        let id = builder.identities.declaration();
        builder.graph.declarations.push(Declaration {
            id,
            file: marker.file(),
            syntax: syntax.clone(),
        });
        builder
            .bind(
                marker.file(),
                syntax.visibility,
                marker.name(),
                Binding::Declaration(id),
                syntax.location,
                false,
            )
            .unwrap();
        builder
            .reserve_placeholder(
                marker.file(),
                marker.name(),
                marker.visibility(),
                marker.location(),
            )
            .unwrap();
        assert_eq!(builder.graph.placeholders().len(), 1);
        assert_eq!(
            builder.graph.placeholder_binding(marker.id()),
            Some(Binding::Declaration(id))
        );
        assert!(
            builder
                .reserve_placeholder(
                    marker.file(),
                    marker.name(),
                    marker.visibility(),
                    syntax.location
                )
                .is_err()
        );
    }
}
