//! Resolving `#import` statements: instances, program origins, groups and provided modules.
use super::filesystem::import_entry;
use super::parameters::INFERRED_PARAM;
use crate::sema::{
    BootstrapModule, Compiler, ModuleId, ProvidedImport, ProvidedKind, Rc, Result, ScopeId, Sym,
    TypeId, Value, ast, err, scope,
};
use std::path::Path;

impl Compiler {
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
    pub(in crate::sema) fn lookup_import(
        &mut self,
        scope: ScopeId,
        index: usize,
    ) -> Result<Option<ModuleId>> {
        match self.import_module(scope, index) {
            Err(error) => self.ed.keep_going_once(error).map(|()| None),
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
        let file = self.scope_file(from_scope);
        let dir = self.file_dir(file);
        let mut params = Vec::new();
        for (position, arg) in import.params.iter().enumerate() {
            // `.Member` needs the parameter's type, known only once the module is read.
            let (value, ty) = if let ast::ExprKind::InferredMember(member) = &arg.value.kind {
                (
                    Value::String(format!("{INFERRED_PARAM}{}", member.name).as_bytes().into()),
                    TypeId::VOID,
                )
            } else {
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
            };
            let name = arg
                .name
                .map_or_else(|| Sym::intern(&format!("${position}")), |n| n.name);
            params.push((name, value, ty));
        }
        match &import.source {
            ast::ImportSource::Module(name)
                if BootstrapModule::named(name) == Some(BootstrapModule::RuntimeSupport)
                    && self.runtime_support.is_some() =>
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
                let host = self.modules[host.0 as usize].name.clone();
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
                let label = format!("import string {}", self.sources.len());
                self.load_string(&label, source, id)?;
                self.validate_module_arguments(id, import.span)?;
                Ok(id)
            }
        }
    }

    /// PROGRAM arguments are workspace inputs, unlike ordinary module arguments.
    /// Use lexical module ownership: loaded/generated main files share the main
    /// module, whereas even anonymous or private imported scopes do not.
    pub(super) fn validate_import_program_origin(
        &self,
        scope: ScopeId,
        import: &ast::Import,
    ) -> Result<()> {
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

    pub(super) fn validate_or_queue_import_groups(
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
    pub(super) fn validate_import_groups(
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
            ProvidedKind::PathToFile => Some(self.canon(Path::new(&provided.value))),
            ProvidedKind::PathToDirectory => {
                Some(self.canon(&Path::new(&provided.value).join("module.jai")))
            }
            ProvidedKind::FullText => {
                if let Some(&id) = self.provided_texts.get(&provided.value) {
                    self.validate_or_queue_import_groups(id, import)?;
                    return Ok(Some(id));
                }
                let id = self.new_module(name, None, params.to_vec());
                self.modules[id.0 as usize]
                    .argument_imports
                    .push(Rc::new(import.clone()));
                let label = format!("provided import {name}");
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
        match &self.ct.interp.workspaces {
            Some(w) => crate::build::import_rules(w, self.workspace),
            None => (Vec::new(), Vec::new()),
        }
    }
}
