//! Scoped imports preserve graph identities and their source-position environment.
use super::*;
use jai_modules::Binding as GraphBinding;

/// A source namespace prefix keeps unavailable marker links separate from
/// actual imported bindings. Cloning it preserves the declaration's prefix.
#[derive(Clone, Default)]
pub(super) struct ScopedImportEnvironment {
    bindings: HashMap<Symbol, GraphBinding>,
    placeholders: HashMap<Symbol, jai_modules::PlaceholderId>,
}
impl std::ops::Deref for ScopedImportEnvironment {
    type Target = HashMap<Symbol, GraphBinding>;
    fn deref(&self) -> &Self::Target {
        &self.bindings
    }
}
impl std::ops::DerefMut for ScopedImportEnvironment {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.bindings
    }
}
impl ScopedImportEnvironment {
    pub(super) fn contains_key(&self, name: &Symbol) -> bool {
        self.bindings.contains_key(name) || self.placeholders.contains_key(name)
    }
    pub(super) fn placeholder(&self, name: Symbol) -> Option<jai_modules::PlaceholderId> {
        self.placeholders.get(&name).copied()
    }
    pub(super) fn placeholders(
        &self,
    ) -> impl Iterator<Item = (Symbol, jai_modules::PlaceholderId)> + '_ {
        self.placeholders.iter().map(|(&name, &id)| (name, id))
    }
}

#[derive(Clone, Default)]
pub(super) struct UsingPrefixEnvironment {
    bindings: HashMap<Symbol, Binding>,
    pending: HashSet<Symbol>,
    placeholders: HashMap<Symbol, jai_modules::PlaceholderId>,
    origins: HashMap<Symbol, Span>,
    operator_declarations: Vec<jai_source::DeclarationId>,
}

impl UsingPrefixEnvironment {
    pub(super) fn compiler_retention_entries(&self) -> usize {
        self.bindings
            .capacity()
            .saturating_add(self.pending.capacity())
            .saturating_add(self.placeholders.capacity())
            .saturating_add(self.origins.capacity())
            .saturating_add(self.operator_declarations.len())
    }

    pub(super) fn from_frame(frame: &ScopeFrame) -> Self {
        Self {
            bindings: frame.using_bindings.clone(),
            pending: frame.using_pending.clone(),
            placeholders: frame.using_placeholders.clone(),
            origins: frame.using_origins.clone(),
            operator_declarations: frame.using_operator_declarations.clone(),
        }
    }

    pub(super) fn restore(self, frame: &mut ScopeFrame) {
        frame.using_bindings = self.bindings;
        frame.using_pending = self.pending;
        frame.using_placeholders = self.placeholders;
        frame.using_origins = self.origins;
        frame.using_operator_declarations = self.operator_declarations;
    }

    fn insert(
        &mut self,
        name: Symbol,
        binding: Option<Binding>,
        span: Span,
        allow_identical_static: bool,
    ) -> Result<(), Diagnostic> {
        if let Some(&origin) = self.origins.get(&name) {
            let identical_static = matches!(
                (self.bindings.get(&name), binding.as_ref()),
                (Some(Binding::Imported(previous)), Some(Binding::Imported(binding)))
                    if previous == binding
            );
            if origin != span && !(allow_identical_static && identical_static) {
                return Err(Diagnostic::new(
                    span,
                    "duplicate using member in lexical scope",
                ));
            }
            // Replaying a checked prefix must not replace a materialized place
            // by the pending alias that preceded its source statement.
            if binding.is_none() && self.bindings.contains_key(&name) {
                return Ok(());
            }
        }
        self.origins.insert(name, span);
        if let Some(binding) = binding {
            self.placeholders.remove(&name);
            self.pending.remove(&name);
            self.bindings.insert(name, binding);
        } else {
            self.pending.insert(name);
        }
        Ok(())
    }

    fn extend_operators(&mut self, declarations: &[jai_source::DeclarationId]) {
        for &declaration in declarations {
            if !self.operator_declarations.contains(&declaration) {
                self.operator_declarations.push(declaration);
            }
        }
    }
    fn insert_placeholder(
        &mut self,
        name: Symbol,
        marker: jai_modules::PlaceholderId,
        span: Span,
        allow_identical: bool,
    ) -> Result<(), Diagnostic> {
        if let Some(&origin) = self.origins.get(&name)
            && origin != span
            && !(allow_identical && self.placeholders.get(&name) == Some(&marker))
        {
            return Err(Diagnostic::new(
                span,
                "duplicate using member in lexical scope",
            ));
        }
        self.origins.insert(name, span);
        self.pending.remove(&name);
        self.bindings.remove(&name);
        self.placeholders.insert(name, marker);
        Ok(())
    }
}

#[derive(Clone)]
pub(crate) struct ImportEnvironment {
    bindings: ScopedImportEnvironment,
    aliases: HashSet<Symbol>,
    operator_modules: Vec<jai_source::ModuleId>,
    using: UsingPrefixEnvironment,
}

impl Resolver<'_> {
    pub(crate) fn checked_using_publication(
        &self,
        span: Span,
    ) -> Option<jai_modules::UsingPublication> {
        let scope = self.graph_scope?;
        let specialization = self.meta.source_specialization_keys.get(&self.procedure);
        std::iter::once(self.procedure)
            .chain(
                self.local_scopes
                    .frames
                    .iter()
                    .rev()
                    .filter_map(|frame| match frame.id.owner {
                        LexicalScopeOwner::Procedure(owner) => Some(owner),
                        LexicalScopeOwner::Record(_) => None,
                    }),
            )
            .find_map(|owner| {
                let key =
                    specialization.or_else(|| self.meta.source_specialization_keys.get(&owner));
                scope
                    .using_source_publication(Some(owner), span, key)
                    .cloned()
            })
    }

    pub(crate) fn local_import_environment(&mut self) -> ImportEnvironment {
        while self.local_scopes.frames.len() < self.scopes.len() {
            self.push_local_scope();
        }
        let frame = self.local_scopes.frames.last().unwrap();
        ImportEnvironment {
            bindings: frame.imports.clone(),
            aliases: frame.import_aliases.clone(),
            operator_modules: frame.operator_imports.clone(),
            using: UsingPrefixEnvironment::from_frame(frame),
        }
    }

    pub(crate) fn restore_local_import_environment(&mut self, environment: ImportEnvironment) {
        let frame = self
            .local_scopes
            .frames
            .last_mut()
            .expect("import environment has a lexical frame");
        frame.imports = environment.bindings;
        frame.import_aliases = environment.aliases;
        frame.operator_imports = environment.operator_modules;
        environment.using.restore(frame);
    }

    pub(super) fn extend_checked_using_prefix(
        &self,
        environment: &mut UsingPrefixEnvironment,
        directive: &syntax::UsingDirective,
    ) -> Result<(), Diagnostic> {
        let Some(publication) = self.checked_using_publication(directive.span) else {
            return Ok(());
        };
        let allow_identical_static = matches!(directive.selection, syntax::UsingSelection::All);
        for &(name, binding) in &publication.bindings {
            if allow_identical_static
                && environment.origins.get(&name).is_some_and(|&origin| {
                    origin != directive.span
                        && self
                            .checked_using_publication(origin)
                            .is_some_and(|prior| prior.bindings.contains(&(name, binding)))
                })
            {
                continue;
            }
            environment.insert(
                name,
                Some(Binding::Imported(binding)),
                directive.span,
                allow_identical_static,
            )?;
        }
        for &(_, destination) in &publication.aliases {
            environment.insert(destination, None, directive.span, false)?;
        }
        for &(name, marker) in &publication.placeholders {
            environment.insert_placeholder(name, marker, directive.span, allow_identical_static)?;
        }
        environment.extend_operators(&publication.selected_operator_declarations);
        Ok(())
    }

    pub(crate) fn bind_checked_using_prefix(
        &mut self,
        directive: &syntax::UsingDirective,
    ) -> Result<(), Diagnostic> {
        while self.local_scopes.frames.len() < self.scopes.len() {
            self.push_local_scope();
        }
        let mut environment =
            UsingPrefixEnvironment::from_frame(self.local_scopes.frames.last().unwrap());
        self.extend_ready_checked_using_prefix(&mut environment, directive)?;
        environment.restore(self.local_scopes.frames.last_mut().unwrap());
        Ok(())
    }

    fn extend_ready_checked_using_prefix(
        &mut self,
        environment: &mut UsingPrefixEnvironment,
        directive: &syntax::UsingDirective,
    ) -> Result<(), Diagnostic> {
        self.extend_checked_using_prefix(environment, directive)?;
        let Some(publication) = self.checked_using_publication(directive.span) else {
            return Ok(());
        };
        for &(name, binding) in &publication.bindings {
            if let GraphBinding::StorageMember(id) = binding {
                if environment
                    .bindings
                    .get(&name)
                    .is_some_and(|binding| matches!(binding, Binding::Storage(_)))
                {
                    continue;
                }
                let binding = self.imported_storage_member_value(id, directive.span)?;
                environment.insert(name, Some(binding), directive.span, false)?;
            }
        }
        if publication.aliases.is_empty() {
            return Ok(());
        }
        let Some(ty) = self.checked_using_static_target(directive)? else {
            return Ok(());
        };
        for (source, destination) in publication.aliases {
            if let Some(binding) = self.type_namespace_member(ty, source, directive.span)? {
                environment.insert(destination, Some(binding), directive.span, false)?;
            }
        }
        Ok(())
    }

    // A static namespace can be prepared from its original declaration. A
    // runtime target stays pending until the source statement creates its place.
    fn checked_using_static_target(
        &mut self,
        directive: &syntax::UsingDirective,
    ) -> Result<Option<TypeId>, Diagnostic> {
        let path = match &directive.target.kind {
            syntax::ExpressionKind::Name(root) => syntax::NamePath {
                root: *root,
                members: vec![],
            },
            syntax::ExpressionKind::QualifiedName(path) => path.clone(),
            _ => return Ok(None),
        };
        let mut candidate = false;
        for depth in (0..self.scopes.len()).rev() {
            if let Some(binding) = self.scopes[depth].get(&path.root) {
                candidate = matches!(binding, Binding::Type(_));
                break;
            }
            if let Some(frame) = self.local_scopes.frames.get(depth) {
                if let Some(declaration) = frame.declarations.get(&path.root) {
                    candidate = matches!(
                        declaration.syntax,
                        DeclarationSyntax::Record(_)
                            | DeclarationSyntax::Enum(_)
                            | DeclarationSyntax::Alias(_)
                    );
                    break;
                }
                if frame.runtime.contains_key(&path.root)
                    || frame.runtime_symbols.contains(&path.root)
                {
                    break;
                }
                if let Some(binding) = frame.using_bindings.get(&path.root) {
                    candidate = matches!(binding, Binding::Type(_));
                    break;
                }
                if frame.using_pending.contains(&path.root)
                    || frame.using_placeholders.contains_key(&path.root)
                    || frame.imports.contains_key(&path.root)
                {
                    break;
                }
            }
        }
        if !candidate {
            return Ok(None);
        }
        let Some(Binding::Type(mut ty)) =
            self.resolve_local_name(path.root, directive.target.span)?
        else {
            return Ok(None);
        };
        for name in path.members {
            let Some(Binding::Type(member)) =
                self.type_namespace_member(ty, name, directive.target.span)?
            else {
                return Ok(None);
            };
            ty = member;
        }
        Ok(Some(ty))
    }

    pub(crate) fn bind_checked_using_environment(
        &mut self,
        span: Span,
        bindings: Vec<(Symbol, Binding)>,
        operators: Vec<jai_source::DeclarationId>,
        allow_identical_static: bool,
    ) -> Result<(), Diagnostic> {
        while self.local_scopes.frames.len() < self.scopes.len() {
            self.push_local_scope();
        }
        let depth = self.local_scopes.frames.len() - 1;
        let mut environment = UsingPrefixEnvironment::from_frame(&self.local_scopes.frames[depth]);
        for (name, binding) in bindings {
            let frame = &self.local_scopes.frames[depth];
            if self.scopes[depth].contains_key(&name)
                || frame.declarations.contains_key(&name)
                || frame.runtime.contains_key(&name)
                || frame.runtime_symbols.contains(&name)
                || frame.import_aliases.contains(&name)
            {
                return Err(Diagnostic::new(
                    span,
                    format!(
                        "using member '{}' duplicates a local declaration",
                        self.symbols.name(name)
                    ),
                ));
            }
            if allow_identical_static
                && environment.origins.get(&name).is_some_and(|&origin| {
                    origin != span
                        && self.checked_using_publication(origin).is_some_and(|prior| {
                            self.checked_using_publication(span).is_some_and(|current| {
                                current.bindings.iter().any(|&(member, source)| {
                                    member == name && prior.bindings.contains(&(member, source))
                                })
                            })
                        })
                })
            {
                continue;
            }
            environment.insert(name, Some(binding), span, allow_identical_static)?;
        }
        environment.extend_operators(&operators);
        // Only declarations whose original prefix included this directive gain
        // its actual aliases. Future directives and earlier declarations keep
        // their own environments.
        let frame = &mut self.local_scopes.frames[depth];
        for declaration in frame
            .declarations
            .values_mut()
            .chain(frame.operators.iter_mut())
        {
            for (&name, &origin) in &declaration.using.origins {
                if origin == span
                    && let Some(binding) = environment.bindings.get(&name)
                {
                    declaration.using.bindings.insert(name, binding.clone());
                    declaration.using.pending.remove(&name);
                }
            }
        }
        environment.restore(frame);
        Ok(())
    }

    pub(crate) fn bind_checked_using_placeholders(
        &mut self,
        span: Span,
        rows: &[(Symbol, jai_modules::PlaceholderId)],
        allow_identical: bool,
    ) -> Result<(), Diagnostic> {
        if rows.is_empty() {
            return Ok(());
        }
        while self.local_scopes.frames.len() < self.scopes.len() {
            self.push_local_scope();
        }
        let depth = self.local_scopes.frames.len() - 1;
        let mut environment = UsingPrefixEnvironment::from_frame(&self.local_scopes.frames[depth]);
        for &(name, marker) in rows {
            let frame = &self.local_scopes.frames[depth];
            if self.scopes[depth].contains_key(&name)
                || frame.declarations.contains_key(&name)
                || frame.runtime.contains_key(&name)
                || frame.runtime_symbols.contains(&name)
                || frame.import_aliases.contains(&name)
            {
                return Err(Diagnostic::new(
                    span,
                    "using placeholder duplicates a local declaration",
                ));
            }
            environment.insert_placeholder(name, marker, span, allow_identical)?;
        }
        let frame = &mut self.local_scopes.frames[depth];
        for declaration in frame
            .declarations
            .values_mut()
            .chain(frame.operators.iter_mut())
        {
            for &(name, marker) in rows {
                if declaration.using.origins.get(&name) == Some(&span) {
                    declaration.using.placeholders.insert(name, marker);
                }
            }
        }
        environment.restore(frame);
        Ok(())
    }
    pub(super) fn extend_scoped_imports(
        &self,
        bindings: &mut ScopedImportEnvironment,
        import: &syntax::ScopedImportDeclaration,
    ) -> Result<(), Diagnostic> {
        let scope = self.graph_scope.ok_or_else(|| {
            Diagnostic::new(import.span, "scoped imports require a source module graph")
        })?;
        let module = scope.scoped_import_module(
            import.span,
            self.meta.source_specialization_keys.get(&self.procedure),
        )?;
        if import.using || import.namespace.is_none() {
            for (name, binding) in scope.module_exports(module) {
                if let Some(marker) = bindings.placeholder(name)
                    && scope.imported_placeholder_binding(marker, import.span)? != binding
                {
                    return Err(Diagnostic::new(
                        import.span,
                        "ambiguous imported placeholder",
                    ));
                }
                if let Some(previous) = bindings.insert(name, binding)
                    && previous != binding
                {
                    return Err(Diagnostic::new(
                        import.span,
                        format!("ambiguous imported member '{}'", self.symbols.name(name)),
                    ));
                }
            }
            for (name, marker) in scope.module_placeholder_exports(module, import.span) {
                if let Some(previous) = bindings.placeholders.insert(name, marker)
                    && previous != marker
                {
                    return Err(Diagnostic::new(
                        import.span,
                        "ambiguous imported placeholder",
                    ));
                }
                if let Some(&previous) = bindings.get(&name)
                    && scope.imported_placeholder_binding(marker, import.span)? != previous
                {
                    return Err(Diagnostic::new(import.span, "ambiguous imported member"));
                }
            }
        }
        if let Some(name) = import.namespace
            && (bindings.placeholder(name).is_some()
                || bindings
                    .insert(name, GraphBinding::Module(module))
                    .is_some())
        {
            return Err(Diagnostic::new(
                import.span,
                format!(
                    "duplicate scoped import alias '{}'",
                    self.symbols.name(name)
                ),
            ));
        }
        Ok(())
    }

    pub(crate) fn bind_scoped_import(
        &mut self,
        import: &syntax::ScopedImportDeclaration,
    ) -> Result<(), Diagnostic> {
        while self.local_scopes.frames.len() < self.scopes.len() {
            self.push_local_scope();
        }
        let depth = self.local_scopes.frames.len() - 1;
        if let Some(name) = import.namespace {
            let frame = &self.local_scopes.frames[depth];
            if self.scopes[depth].contains_key(&name)
                || frame.declarations.contains_key(&name)
                || frame.runtime.contains_key(&name)
                || frame.import_aliases.contains(&name)
                || frame.using_bindings.contains_key(&name)
                || frame.using_pending.contains(&name)
                || frame.using_placeholders.contains_key(&name)
            {
                return Err(Diagnostic::new(
                    import.span,
                    "scoped import alias duplicates a local declaration",
                ));
            }
        }
        let mut imports = self.local_scopes.frames[depth].imports.clone();
        self.extend_scoped_imports(&mut imports, import)?;
        self.local_scopes.frames[depth].imports = imports;
        let mut operator_imports = self.local_scopes.frames[depth].operator_imports.clone();
        self.extend_operator_imports(&mut operator_imports, import)?;
        self.local_scopes.frames[depth].operator_imports = operator_imports;
        if let Some(name) = import.namespace {
            self.local_scopes.frames[depth].import_aliases.insert(name);
        }
        Ok(())
    }

    pub(crate) fn refresh_local_import_environments(
        &mut self,
        statements: &[syntax::Statement],
    ) -> Result<(), Diagnostic> {
        let depth = self.local_scopes.frames.len() - 1;
        let mut imports = self.local_scopes.frames[depth].imports.clone();
        let mut operator_imports = self.local_scopes.frames[depth].operator_imports.clone();
        let mut using = UsingPrefixEnvironment::from_frame(&self.local_scopes.frames[depth]);
        for statement in statements {
            if let syntax::StatementKind::Import(import) = &statement.kind {
                self.extend_scoped_imports(&mut imports, import)?;
                self.extend_operator_imports(&mut operator_imports, import)?;
            } else if let syntax::StatementKind::Using(directive) = &statement.kind {
                self.extend_ready_checked_using_prefix(&mut using, directive)?;
            } else if let Some(syntax) = DeclarationSyntax::from_statement(statement) {
                let frame = &mut self.local_scopes.frames[depth];
                let declaration = if syntax.operator().is_some() {
                    frame
                        .operators
                        .iter_mut()
                        .find(|declaration| declaration.syntax.span() == syntax.span())
                } else {
                    frame.declarations.get_mut(&syntax.name())
                };
                if let Some(declaration) = declaration {
                    declaration.imports = imports.clone();
                    declaration.operator_imports = operator_imports.clone();
                    declaration.using = using.clone();
                }
            }
            if let Some(directive) = statement.using_declaration_directive() {
                self.extend_ready_checked_using_prefix(&mut using, &directive)?;
            }
        }
        Ok(())
    }

    pub(crate) fn local_import_alias_reserved(&self, name: Symbol) -> bool {
        self.local_scopes
            .frames
            .last()
            .is_some_and(|frame| frame.import_aliases.contains(&name))
    }

    pub(crate) fn local_using_name_reserved(&self, name: Symbol) -> bool {
        self.local_scopes.frames.last().is_some_and(|frame| {
            frame.using_bindings.contains_key(&name)
                || frame.using_pending.contains(&name)
                || frame.using_placeholders.contains_key(&name)
        })
    }

    pub(crate) fn lexical_using_binding_ready(
        &self,
        name: Symbol,
        span: Span,
    ) -> Result<Option<Binding>, Diagnostic> {
        for depth in (0..self.scopes.len()).rev() {
            if self.scopes[depth].contains_key(&name) {
                return Ok(None);
            }
            if let Some(frame) = self.local_scopes.frames.get(depth) {
                if frame.declarations.contains_key(&name)
                    || frame.runtime.contains_key(&name)
                    || frame.runtime_symbols.contains(&name)
                {
                    return Ok(None);
                }
                if let Some(binding) = frame.using_bindings.get(&name) {
                    return Ok(Some(binding.clone()));
                }
                if let Some(&marker) = frame.using_placeholders.get(&name) {
                    let scope = self.graph_scope.ok_or_else(|| {
                        Diagnostic::new(span, "using placeholder requires its source graph")
                    })?;
                    return Ok(Some(Binding::Imported(
                        scope.imported_placeholder_binding(marker, span)?,
                    )));
                }
                if frame.using_pending.contains(&name) {
                    return Err(Diagnostic::new(
                        span,
                        "using place alias is unavailable before its source statement",
                    ));
                }
                if frame.imports.contains_key(&name) {
                    return Ok(None);
                }
            }
        }
        Ok(None)
    }

    pub(crate) fn lexical_graph_binding(
        &mut self,
        path: &syntax::NamePath,
        span: Span,
    ) -> Result<Option<GraphBinding>, Diagnostic> {
        let Some(binding) = self.resolve_local_name(path.root, span)? else {
            return Ok(None);
        };
        self.imported_path_binding(binding, &path.members, span)
    }

    pub(crate) fn lexical_imported_value_root(
        &mut self,
        path: &syntax::NamePath,
        span: Span,
    ) -> Result<Option<(Binding, Vec<Symbol>)>, Diagnostic> {
        let Some(binding) = self.resolve_local_name(path.root, span)? else {
            return Ok(None);
        };
        match binding {
            Binding::Imported(GraphBinding::Module(module)) | Binding::Namespace(module) => {
                if path.members.is_empty() {
                    return Ok(None);
                }
                let scope = self.graph_scope.ok_or_else(|| {
                    Diagnostic::new(span, "imported namespaces require a source graph")
                })?;
                let mut first_error = None;
                for prefix in (1..=path.members.len()).rev() {
                    match scope.namespace_binding(module, &path.members[..prefix], span) {
                        Ok(binding) => {
                            if matches!(binding, GraphBinding::Module(_)) {
                                continue;
                            }
                            return Ok(Some((
                                self.imported_binding_value(binding, span)?,
                                path.members[prefix..].to_vec(),
                            )));
                        }
                        Err(error) => {
                            first_error.get_or_insert(error);
                        }
                    }
                }
                match first_error {
                    Some(error) => Err(error),
                    None => Ok(None),
                }
            }
            Binding::Imported(binding) => Ok(Some((
                self.imported_binding_value(binding, span)?,
                path.members.clone(),
            ))),
            _ => {
                let ready = self.lexical_using_binding_ready(path.root, span)?;
                if let Some(Binding::Storage(storage)) = &ready {
                    self.check_local_storage_capture(*storage, span)?;
                }
                Ok(ready.map(|binding| (binding, path.members.clone())))
            }
        }
    }

    pub(crate) fn lexical_graph_binding_ready(
        &self,
        path: &syntax::NamePath,
        span: Span,
    ) -> Result<Option<GraphBinding>, Diagnostic> {
        for depth in (0..self.scopes.len()).rev() {
            if let Some(binding) = self.scopes[depth].get(&path.root).cloned() {
                return self.imported_path_binding(binding, &path.members, span);
            }
            if let Some(frame) = self.local_scopes.frames.get(depth) {
                if frame.declarations.contains_key(&path.root)
                    || frame.runtime.contains_key(&path.root)
                    || frame.runtime_symbols.contains(&path.root)
                {
                    return Ok(None);
                }
                if let Some(binding) = frame.using_bindings.get(&path.root) {
                    return self.imported_path_binding(binding.clone(), &path.members, span);
                }
                if let Some(&marker) = frame.using_placeholders.get(&path.root) {
                    let scope = self.graph_scope.ok_or_else(|| {
                        Diagnostic::new(span, "using placeholder requires its source graph")
                    })?;
                    let binding = scope.imported_placeholder_binding(marker, span)?;
                    return self.imported_path_binding(
                        Binding::Imported(binding),
                        &path.members,
                        span,
                    );
                }
                if frame.using_pending.contains(&path.root) {
                    return Err(Diagnostic::new(
                        span,
                        "using place alias is unavailable before its source statement",
                    ));
                }
                if let Some(&binding) = frame.imports.get(&path.root) {
                    return self.imported_path_binding(
                        Binding::Imported(binding),
                        &path.members,
                        span,
                    );
                }
                if let Some(marker) = frame.imports.placeholder(path.root) {
                    let scope = self.graph_scope.ok_or_else(|| {
                        Diagnostic::new(span, "imported placeholder requires its source graph")
                    })?;
                    let binding = scope.imported_placeholder_binding(marker, span)?;
                    return self.imported_path_binding(
                        Binding::Imported(binding),
                        &path.members,
                        span,
                    );
                }
            }
        }
        Ok(None)
    }

    fn imported_path_binding(
        &self,
        binding: Binding,
        members: &[Symbol],
        span: Span,
    ) -> Result<Option<GraphBinding>, Diagnostic> {
        let (module, graph_binding) = match binding {
            Binding::Namespace(module) => (Some(module), GraphBinding::Module(module)),
            Binding::Imported(GraphBinding::Module(module)) => {
                (Some(module), GraphBinding::Module(module))
            }
            Binding::Imported(binding) => (None, binding),
            _ => return Ok(None),
        };
        if members.is_empty() {
            return Ok(Some(graph_binding));
        }
        let Some(module) = module else {
            return Ok(None);
        };
        let scope = self
            .graph_scope
            .ok_or_else(|| Diagnostic::new(span, "imported namespaces require a source graph"))?;
        match scope.namespace_binding(module, members, span) {
            Ok(binding) => Ok(Some(binding)),
            Err(error) => {
                for prefix in (1..members.len()).rev() {
                    if let Ok(binding) = scope.namespace_binding(module, &members[..prefix], span)
                        && (scope.imported_type(binding, span).is_ok()
                            || scope.imported_value(binding, span).is_ok_and(|binding| {
                                !matches!(binding, Binding::Namespace(_) | Binding::Imported(_))
                            }))
                    {
                        return Ok(None);
                    }
                }
                Err(error)
            }
        }
    }

    pub(crate) fn imported_binding_value(
        &mut self,
        binding: GraphBinding,
        span: Span,
    ) -> Result<Binding, Diagnostic> {
        if let GraphBinding::StorageMember(id) = binding {
            return self.imported_storage_member_value(id, span);
        }
        if let GraphBinding::SourceMember { member, .. } = binding {
            let scope = self.graph_scope.ok_or_else(|| {
                Diagnostic::new(span, "imported source members require a source graph")
            })?;
            let (owner, _) = scope
                .imported_source_member_owner(binding, span)?
                .ok_or_else(|| {
                    Diagnostic::new(span, "imported source member has no nominal owner")
                })?;
            return self
                .type_namespace_member(owner, member, span)?
                .ok_or_else(|| {
                    Diagnostic::new(
                        span,
                        format!(
                            "imported type has no namespace member '{}'",
                            self.symbols.name(member)
                        ),
                    )
                });
        }
        self.imported_binding_value_ready(binding, span)
    }

    pub(crate) fn imported_binding_value_ready(
        &self,
        binding: GraphBinding,
        span: Span,
    ) -> Result<Binding, Diagnostic> {
        if let GraphBinding::StorageMember(_) = binding {
            return Err(Diagnostic::new(
                span,
                "imported storage member metadata is not ready",
            ));
        }
        if let GraphBinding::SourceMember { member, .. } = binding {
            let scope = self.graph_scope.ok_or_else(|| {
                Diagnostic::new(span, "imported source members require a source graph")
            })?;
            if let Some((owner, _)) = scope.imported_source_member_owner(binding, span)? {
                if let Some(value) = self.ready_namespace_member(owner, member) {
                    return Ok(value);
                }
                if let Some(value) = self
                    .meta
                    .local_declarations
                    .enum_member_value(owner, member)
                    .or_else(|| scope.enum_member_value(owner, member))
                {
                    return Ok(Binding::Enum(modules::aggregates::EnumConstant {
                        ty: owner,
                        value,
                    }));
                }
            }
            return Err(Diagnostic::new(
                span,
                "imported source member metadata is not ready",
            ));
        }
        if let GraphBinding::Declaration(id) = binding
            && let Some(context) = self.compile_time
            && context.deferred.contains(&id)
        {
            let mut pending = context.pending_constants.borrow_mut();
            if !pending.contains(&id) {
                pending.push(id);
            }
            return Err(Diagnostic::new(
                span,
                "imported constant is pending typed compile-time evaluation",
            ));
        }
        let scope = self
            .graph_scope
            .ok_or_else(|| Diagnostic::new(span, "imported values require a source graph"))?;
        let value = scope.imported_value(binding, span)?;
        if matches!(value, Binding::Imported(_)) {
            return Err(Diagnostic::new(
                span,
                "imported declaration requires a concrete callable specialization or a ready compile-time value",
            ));
        }
        Ok(value)
    }
}
