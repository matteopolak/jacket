use super::*;
use jai_source::Identities;
use jai_syntax::{FileItem, ImportDeclaration, ImportMode};
use std::{collections::HashSet, fs};
pub(super) struct Builder {
    graph: ModuleGraph,
    identities: Identities,
    options: GraphOptions,
    sources: HashMap<PathBuf, SourceId>,
    syntax: HashMap<SourceId, ParsedFile>,
    modules: HashMap<PathBuf, ModuleId>,
    files: HashMap<(ModuleId, PathBuf), FileInstanceId>,
    active_modules: HashSet<ModuleId>,
    active_files: HashSet<(ModuleId, PathBuf)>,
}
impl Builder {
    pub(super) fn new(options: GraphOptions) -> Self {
        let mut identities = Identities::default();
        let unit = identities.unit();
        let root = identities.module();
        let scope = identities.scope();
        Self {
            graph: ModuleGraph {
                unit,
                root,
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
                imports: vec![],
                loads: vec![],
            },
            identities,
            options,
            sources: HashMap::new(),
            syntax: HashMap::new(),
            modules: HashMap::new(),
            files: HashMap::new(),
            active_modules: HashSet::new(),
            active_files: HashSet::new(),
        }
    }
    pub(super) fn build(mut self, path: &Path) -> Result<ModuleGraph, GraphError> {
        let path = Self::canonical(path)?;
        let root = self.graph.root;
        self.modules.insert(path.clone(), root);
        self.active_modules.insert(root);
        self.module_files(root, &path)?;
        self.active_modules.remove(&root);
        Ok(self.graph)
    }
    fn canonical(path: &Path) -> Result<PathBuf, GraphError> {
        path.canonicalize().map_err(|cause| GraphError::Io {
            path: path.to_owned(),
            cause,
        })
    }
    fn located(&self, location: SourceSpan, message: impl Into<String>) -> GraphError {
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
        let bytes = fs::read(path).map_err(|cause| GraphError::Io {
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
    fn module_files(&mut self, module: ModuleId, path: &Path) -> Result<(), GraphError> {
        let mut imports = Vec::new();
        let entry = self.file(module, path, None, &mut imports)?;
        self.graph.modules[module.index()].entry = Some(entry);
        for (file, import) in imports {
            self.import(file, &import)?;
        }
        Ok(())
    }
    fn file(
        &mut self,
        module: ModuleId,
        path: &Path,
        origin: Option<SourceSpan>,
        imports: &mut Vec<(FileInstanceId, ImportDeclaration)>,
    ) -> Result<FileInstanceId, GraphError> {
        let path = Self::canonical(path)?;
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
        for item in syntax.items() {
            match item {
                FileItem::Declaration(syntax) => {
                    let declaration = Declaration {
                        id: self.identities.declaration(),
                        file: id,
                        syntax: syntax.clone(),
                    };
                    let name = declaration.name();
                    let declaration_id = declaration.id;
                    self.bind(
                        id,
                        syntax.visibility,
                        name,
                        Binding::Declaration(declaration_id),
                        syntax.location,
                        false,
                    )?;
                    self.graph.files[id.0].declarations.push(declaration_id);
                    self.graph.declarations.push(declaration);
                }
                FileItem::Load(load) => {
                    let target = self.file(
                        module,
                        &path.parent().unwrap().join(&load.target),
                        Some(load.location),
                        imports,
                    )?;
                    self.graph.loads.push(LoadEdge {
                        file: id,
                        target,
                        location: load.location,
                    });
                }
                FileItem::Import(import) => imports.push((id, import.clone())),
                FileItem::Parameters(parameters) => {
                    return Err(self.located(
                        parameters.location,
                        "module parameters are not implemented in the module graph",
                    ));
                }
                FileItem::Scope { .. } => {}
            }
        }
        self.active_files.remove(&key);
        Ok(id)
    }
    fn bind(
        &mut self,
        file: FileInstanceId,
        visibility: Visibility,
        name: Symbol,
        binding: Binding,
        location: SourceSpan,
        idempotent: bool,
    ) -> Result<(), GraphError> {
        let module = self.graph.files[file.0].module;
        let bindings = if visibility == Visibility::File {
            &mut self.graph.files[file.0].private
        } else {
            &mut self.graph.modules[module.index()].bindings
        };
        if let Some(previous) = bindings.get(&name) {
            if !idempotent || *previous != binding {
                return Err(self.located(
                    location,
                    format!(
                        "conflicting declaration or import '{}'",
                        self.graph.symbols.name(name)
                    ),
                ));
            }
        } else {
            bindings.insert(name, binding);
        }
        if visibility == Visibility::Export {
            self.graph.modules[module.index()]
                .exports
                .insert(name, binding);
        }
        Ok(())
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
            ImportMode::File => Self::canonical(&parent.join(&import.target)),
            ImportMode::Directory => {
                Self::canonical(&parent.join(&import.target).join("module.jai"))
            }
            ImportMode::Search => {
                for directory in &self.options.import_dirs {
                    for candidate in [
                        directory.join(format!("{}.jai", import.target)),
                        directory.join(&import.target).join("module.jai"),
                    ] {
                        if candidate.is_file() {
                            return Self::canonical(&candidate);
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
        if import.arguments.instance.is_some() || import.arguments.program.is_some() {
            return Err(self.located(
                import.location,
                "parameterized imports are not implemented in the module graph",
            ));
        }
        let path = self.import_path(file, import)?;
        let module = if let Some(&module) = self.modules.get(&path) {
            if self.active_modules.contains(&module) {
                return Err(self.cycle(DependencyKind::Import, path, Some(import.location)));
            }
            module
        } else {
            let module = self.identities.module();
            self.graph.modules.push(ModuleInstance {
                id: module,
                scope: self.identities.scope(),
                entry: None,
                files: vec![],
                bindings: HashMap::new(),
                exports: HashMap::new(),
            });
            self.modules.insert(path.clone(), module);
            self.active_modules.insert(module);
            self.module_files(module, &path)?;
            self.active_modules.remove(&module);
            module
        };
        self.graph.imports.push(ImportEdge {
            file,
            module,
            location: import.location,
        });
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
        }
        Ok(())
    }
}
