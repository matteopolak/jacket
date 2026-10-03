//! Bootstrap planning and lifecycle of a retained source discovery session.
use super::*;

impl Builder<'_> {
    pub(crate) fn build(self, path: &Path) -> Result<ModuleGraph, GraphError> {
        self.build_with_bootstrap(path, PreludeSource::Disabled)
    }
    pub(crate) fn build_with_bootstrap(
        self,
        path: &Path,
        prelude: PreludeSource,
    ) -> Result<ModuleGraph, GraphError> {
        self.build_with_bootstrap_options(
            path,
            BootstrapOptions {
                prelude,
                runtime_support: None,
            },
        )
    }
    pub(crate) fn build_with_bootstrap_options(
        mut self,
        path: &Path,
        bootstrap: BootstrapOptions,
    ) -> Result<ModuleGraph, GraphError> {
        self.initialize(path, bootstrap)?;
        match self.advance()? {
            DiscoveryStatus::Complete => Ok(self.graph),
            DiscoveryStatus::Awaiting {
                ..
            } => {
                let diagnostic = self
                    .pending_diagnostics
                    .first()
                    .expect("awaiting discovery has a located dependency")
                    .diagnostic
                    .clone();
                let rendered = diagnostic.render(&self.graph.sources);
                Err(GraphError::Pending {
                    diagnostic,
                    rendered,
                })
            }
        }
    }
    pub(crate) fn initialize(
        &mut self,
        path: &Path,
        bootstrap: BootstrapOptions,
    ) -> Result<(), GraphError> {
        let path = self.canonical(path)?;
        let root = self.graph.root;
        self.modules.insert(
            ModuleKey {
                path: path.clone(),
                arguments: None,
            },
            root,
        );
        if bootstrap.runtime_support.is_some() && bootstrap.prelude == PreludeSource::Disabled {
            return Err(GraphError::Prelude(PreludeError::InvalidConfiguration(
                "Runtime_Support bootstrap requires actual Preload source",
            )));
        }
        if let Some(preload) = bootstrap
            .prelude
            .resolve(&self.options.import_dirs, self.provider)
            .map_err(GraphError::Prelude)?
        {
            if preload == path {
                // Checking Preload itself must not parse/reserve its declarations twice.
                self.graph.prelude = Some(root);
                // Runtime_Support can select dependencies using Preload constants.
                // Publish this actual root scope before loading the automatic runtime.
                self.entry_modules.push((root, path.clone()));
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
                self.graph.prelude = Some(module);
                self.modules.insert(
                    ModuleKey {
                        path: preload.clone(),
                        arguments: None,
                    },
                    module,
                );
                self.requests.insert(module, (None, None));
                self.program.insert(preload.clone(), None);
                self.entry_modules.push((module, preload));
            }
        }
        if let Some(runtime) = bootstrap.runtime_support {
            let runtime_path = runtime
                .source
                .resolve(&self.options.import_dirs, self.provider)
                .map_err(GraphError::Prelude)?;
            if runtime.parameters.temporary_storage_size < 0 {
                return Err(GraphError::Prelude(PreludeError::InvalidConfiguration(
                    "Runtime_Support temporary storage size must be nonnegative",
                )));
            }
            let arguments = [
                (
                    "DEFINE_SYSTEM_ENTRY_POINT",
                    runtime.parameters.define_system_entry_point,
                ),
                (
                    "DEFINE_INITIALIZATION",
                    runtime.parameters.define_initialization,
                ),
                (
                    "ENABLE_BACKTRACE_ON_CRASH",
                    runtime.parameters.enable_backtrace_on_crash,
                ),
            ]
            .into_iter()
            .map(|(name, value)| Argument {
                name: Some(self.graph.symbols.intern(name)),
                value: ParameterValue::Scalar(jai_eval::Value::Bool(value)),
            })
            .collect::<Vec<_>>();
            let mut arguments = arguments;
            arguments.push(Argument {
                name: Some(self.graph.symbols.intern("TEMPORARY_STORAGE_SIZE")),
                value: ParameterValue::Scalar(jai_eval::Value::Int(
                    jai_types::Integer::checked(
                        jai_types::IntegerType::S32,
                        runtime.parameters.temporary_storage_size as i128,
                    )
                    .expect("validated Runtime_Support temporary storage size"),
                )),
            });
            if runtime_path == path {
                self.graph.runtime_support = Some(root);
                self.requests.insert(root, (Some(arguments), None));
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
                self.graph.runtime_support = Some(module);
                self.modules.insert(
                    ModuleKey {
                        path: runtime_path.clone(),
                        arguments: Some(arguments.clone()),
                    },
                    module,
                );
                self.requests.insert(module, (Some(arguments), None));
                self.program.insert(runtime_path.clone(), None);
                self.entry_modules.push((module, runtime_path));
            }
        }
        if !self.entry_modules.iter().any(|(module, _)| *module == root) {
            self.entry_modules.push((root, path));
        }
        for (key, module) in &self.modules {
            let mut key = key.clone();
            if let Some((arguments, _)) = self.requests.get(module) {
                key.arguments = arguments.clone();
            }
            self.graph.source_requests.insert(*module, key);
        }
        Ok(())
    }
    pub(crate) fn discovery_complete(&self) -> bool {
        self.entry_modules
            .iter()
            .all(|(module, _)| self.completed_modules.contains(module))
            && self.pending.values().all(Vec::is_empty)
            && self.pending_specializations.is_empty()
            && self.using_requests.is_ready()
            && self.insertion_requests.is_ready()
            && self.pending_inserted_modules.is_empty()
    }
    pub(crate) fn advance(&mut self) -> Result<DiscoveryStatus, GraphError> {
        // Iterate configured entry modules in bootstrap order. Each module's
        // own dependency worklist reaches a fixed point before it suspends.
        self.pending_diagnostics.clear();
        for (module, path) in self.entry_modules.clone() {
            if self.completed_modules.contains(&module) {
                continue;
            }
            match self.expand_module(module, &path) {
                Ok(()) => {}
                Err(GraphError::Pending {
                    diagnostic, ..
                }) => self.pending_diagnostics.push(DeferredDependency {
                    module,
                    diagnostic,
                }),
                Err(error) => return Err(error),
            }
        }
        for module in std::mem::take(&mut self.pending_inserted_modules) {
            if self.completed_modules.contains(&module) {
                continue;
            }
            let entry = self.graph.modules[module.index()]
                .entry
                .expect("insertion destination has a discovered module entry");
            let source = self.graph.files[entry.index()].source;
            let path = self
                .graph
                .sources
                .get(source)
                .expect("original module entry source")
                .path()
                .to_owned();
            match self.expand_module(module, &path) {
                Ok(()) => {}
                Err(GraphError::Pending {
                    diagnostic, ..
                }) => {
                    self.pending_diagnostics.push(DeferredDependency {
                        module,
                        diagnostic,
                    });
                    self.pending_inserted_modules.push(module);
                }
                Err(error) => return Err(error),
            }
        }
        for key in std::mem::take(&mut self.pending_specializations) {
            match self.expand_specialization(&key) {
                Ok(()) => self.graph.source_specializations.push(key),
                Err(GraphError::Pending {
                    diagnostic, ..
                }) => {
                    let module = self.graph.files[self
                        .graph
                        .declaration(key.declaration())
                        .expect("validated specialization source")
                        .file()
                        .index()]
                    .module;
                    self.pending_diagnostics.push(DeferredDependency {
                        module,
                        diagnostic,
                    });
                    self.pending_specializations.push(key);
                }
                Err(error) => return Err(error),
            }
        }
        for dependency in self.using_dependencies() {
            if !self.pending_diagnostics.iter().any(|pending| {
                pending.module == dependency.module && pending.diagnostic == dependency.diagnostic
            }) {
                self.pending_diagnostics.push(dependency);
            }
        }
        for request in self
            .insertion_requests
            .requests
            .iter()
            .filter(|request| request.publication.is_none())
        {
            self.pending_diagnostics.push(DeferredDependency {
                module: self.graph.files[request.file.index()].module,
                diagnostic: self.graph.diagnostic(
                    request.location,
                    "declaration insertion requires a typed captured Code response",
                ),
            });
        }
        self.refresh_import_exports()?;
        if self.discovery_complete() {
            self.graph
                .validate_operator_aliases()
                .map_err(|diagnostic| {
                    let rendered = diagnostic.render(&self.graph.sources);
                    GraphError::Located {
                        diagnostic,
                        rendered,
                    }
                })?;
            return Ok(DiscoveryStatus::Complete);
        }
        Ok(DiscoveryStatus::Awaiting {
            conditions: self
                .conditions
                .iter()
                .filter(|condition| condition.selected.is_none())
                .map(|condition| condition.id)
                .collect(),
            dependencies: self
                .pending_diagnostics
                .iter()
                .filter(|dependency| {
                    !self
                        .conditions
                        .iter()
                        .any(|condition| condition.location == dependency.diagnostic.location)
                })
                .cloned()
                .collect(),
        })
    }
    pub(super) fn expand_specialization(
        &mut self,
        key: &SourceSpecializationKey,
    ) -> Result<(), GraphError> {
        let declaration = self
            .graph
            .declaration(key.declaration())
            .expect("validated specialization source");
        let file = declaration.file();
        let module = self.graph.files[file.index()].module;
        let declaration = declaration.syntax().clone();
        self.active_specialization = Some(key.clone());
        self.active_modules.insert(module);
        let result = self.scoped_declaration(file, &declaration);
        self.active_modules.remove(&module);
        self.active_specialization = None;
        result
    }
}
