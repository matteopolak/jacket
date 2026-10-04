//! Record body declarations retain an owned lexical namespace separate from fields.
use super::*;

impl Resolver<'_> {
    pub(crate) fn ready_namespace_member(&self, ty: TypeId, name: Symbol) -> Option<Binding> {
        if let Some(binding) = self
            .meta
            .local_declarations
            .record_namespaces
            .get(&ty)
            .and_then(|namespace| namespace.get(&name))
            .cloned()
        {
            return Some(binding);
        }
        let scope = self.meta.local_declarations.namespace_scopes.get(&ty)?;
        let depth = self
            .local_scopes
            .frames
            .iter()
            .position(|frame| frame.id == *scope)?;
        self.scopes[depth].get(&name).cloned()
    }

    pub(super) fn define_local_record_body(
        &mut self,
        source: RecordSource<'_>,
        members: &[syntax::RecordMember],
    ) -> Result<TypeId, Diagnostic> {
        let RecordSource {
            id, ..
        } = source;
        if members.len() > 65_536 {
            return Err(Diagnostic::new(
                source.span,
                "selected record source exceeds compiler declaration budget",
            ));
        }
        if members
            .iter()
            .all(|member| matches!(member, syntax::RecordMember::Field(_)))
        {
            let fields: Vec<_> = members
                .iter()
                .filter_map(|member| match member {
                    syntax::RecordMember::Field(field) => Some(FieldSource::Named(field.clone())),
                    _ => None,
                })
                .collect();
            let ty = self.define_local_record(source, &fields, members, false)?;
            if !self
                .meta
                .local_declarations
                .selected_record_sources
                .contains_key(&ty)
            {
                let environment = Arc::new(self.capture_local_source_environment());
                self.meta
                    .local_declarations
                    .namespace_sources
                    .entry(ty)
                    .or_insert_with(|| environment.clone());
                self.meta.local_declarations.selected_record_sources.insert(
                    ty,
                    Arc::new(SelectedLocalRecordSource {
                        declaration: id,
                        owner: ty,
                        members: members.to_vec().into(),
                        environment,
                    }),
                );
            }
            return Ok(ty);
        }

        let ty = self.meta.local_declarations.entries[&id].nominal.unwrap();
        self.scopes.push(HashMap::new());
        if let Some(scope) = self
            .meta
            .local_declarations
            .namespace_scopes
            .get(&ty)
            .copied()
        {
            self.local_scopes.next_scope = self.local_scopes.next_scope.max(scope.ordinal + 1);
            self.local_scopes.frames.push(ScopeFrame {
                id: scope,
                declarations: HashMap::new(),
                operators: vec![],
                operator_imports: vec![],
                runtime: HashMap::new(),
                external: HashMap::new(),
                runtime_symbols: HashSet::new(),
                imports: imports::ScopedImportEnvironment::default(),
                import_aliases: HashSet::new(),
                using_bindings: HashMap::new(),
                using_pending: HashSet::new(),
                using_placeholders: HashMap::new(),
                using_origins: HashMap::new(),
                using_operator_declarations: vec![],
            });
        } else {
            self.push_local_scope();
            self.local_scopes.frames.last_mut().unwrap().id.owner = LexicalScopeOwner::Record(ty);
            self.meta
                .local_declarations
                .namespace_scopes
                .insert(ty, self.local_scopes.frames.last().unwrap().id);
        }
        let result = (|| {
            // Selection registers the original unconditional and chosen AST
            // nodes. Rebuilding statements here would change their identities.
            let selected = match self
                .meta
                .local_declarations
                .selected_record_sources
                .get(&ty)
            {
                Some(source) => source.members.clone(),
                None => self.select_local_record_members(members)?.into(),
            };
            let depth = self.local_scopes.frames.len() - 1;
            let mut fields = Vec::new();
            let mut declarations = Vec::new();
            let mut names = HashSet::new();
            for member in selected.iter() {
                let (member_name, member_span) = match member {
                    syntax::RecordMember::Field(field) => {
                        fields.push(FieldSource::Named(field.clone()));
                        (field.name, field.span)
                    }
                    syntax::RecordMember::AnonymousRecord(record) => {
                        fields.push(FieldSource::AnonymousRecord(record.as_ref().clone()));
                        continue;
                    }
                    syntax::RecordMember::Placement(_) | syntax::RecordMember::Using(_) => continue,
                    syntax::RecordMember::Assert {
                        ..
                    }
                    | syntax::RecordMember::DefaultOverride {
                        ..
                    } => continue,
                    syntax::RecordMember::Conditional {
                        span, ..
                    }
                    | syntax::RecordMember::CompileTimeCases {
                        span, ..
                    } => {
                        return Err(Diagnostic::new(
                            *span,
                            "record conditional was not selected before shape binding",
                        ));
                    }
                    syntax::RecordMember::Import(value) => {
                        return Err(Diagnostic::new(
                            value.span,
                            "record import requires a checked record-source namespace producer",
                        ));
                    }
                    syntax::RecordMember::Insert(value) => {
                        return Err(Diagnostic::new(
                            value.span,
                            "local record #insert requires record member expansion",
                        ));
                    }
                    member => {
                        let syntax = DeclarationSyntax::from_record_member(member)
                            .expect("selected record declaration retains its source syntax");
                        let name = syntax.name();
                        let span = syntax.span();
                        if syntax.operator().is_some() {
                            let declaration = self.local_scopes.frames[depth]
                                .operators
                                .iter()
                                .find(|declaration| declaration.syntax.span() == span)
                                .expect(
                                    "selected operator retains its original declaration identity",
                                );
                            declarations.push(declaration.clone());
                            continue;
                        }
                        declarations
                            .push(self.local_scopes.frames[depth].declarations[&name].clone());
                        (name, span)
                    }
                };
                if !names.insert(member_name) {
                    return Err(Diagnostic::new(member_span, "duplicate record member name"));
                }
            }
            let has_namespace_members = !declarations.is_empty();
            self.meta
                .local_declarations
                .method_phases
                .insert(ty, MethodPhase::TypesOnly);
            self.resolve_registered_local_declarations()?;
            self.define_local_record_shape(source, &fields, &selected, has_namespace_members)?;
            self.meta.local_declarations.method_phases.remove(&ty);
            // Descriptor queries require the completed field shape. Resolve
            // remaining semantic constants before publishing the namespace.
            let methods: Vec<_> = declarations
                .iter()
                .filter(|declaration| {
                    matches!(
                        declaration.syntax,
                        DeclarationSyntax::Procedure(_) | DeclarationSyntax::Prototype(_)
                    )
                })
                .cloned()
                .collect();
            self.finish_local_record_dependencies(ty, depth, &methods, &selected)?;
            let ready_declarations: Vec<_> = declarations
                .iter()
                .filter(|declaration| {
                    !matches!(&declaration.syntax, DeclarationSyntax::Constant(constant)
                        if matches!(constant.initializer.kind, syntax::ExpressionKind::ShortLambda(_)))
                })
                .cloned()
                .collect();
            self.resolve_local_declaration_batch(depth, &ready_declarations)?;
            for member in selected.iter() {
                if let syntax::RecordMember::Using(directive) = member {
                    let names = crate::record_using::target_names(&directive.target)?;
                    if !fields
                        .iter()
                        .any(|field| field.as_ref().name() == Some(names[0]))
                    {
                        let target = self.expr(&directive.target)?;
                        if !matches!(target, Expr::Type(_)) {
                            return Err(Diagnostic::new(
                                directive.span,
                                "record namespace using requires a checked type target",
                            ));
                        }
                        self.using_directive(directive)?;
                    }
                }
            }
            self.check_local_record_assertions(&selected)?;
            if !methods.is_empty() && !self.meta.local_declarations.method_body_demand.covers_all()
            {
                // A cached enclosing body must not hide unused local method
                // bodies from the later full source sweep.
                self.meta
                    .local_declarations
                    .pending_local_method_records
                    .insert(ty);
            }
            self.meta
                .local_declarations
                .record_namespaces
                .insert(ty, self.scopes.last().unwrap().clone());
            let environment = Arc::new(self.capture_local_source_environment());
            self.meta
                .local_declarations
                .namespace_sources
                .insert(ty, environment.clone());
            self.meta
                .local_declarations
                .selected_record_sources
                .entry(ty)
                .or_insert_with(|| {
                    Arc::new(SelectedLocalRecordSource {
                        declaration: id,
                        owner: ty,
                        members: selected,
                        environment,
                    })
                });
            Ok(ty)
        })();
        self.meta.local_declarations.method_phases.remove(&ty);
        self.scopes.pop();
        self.pop_local_scope();
        result
    }

    pub(crate) fn type_namespace_member(
        &mut self,
        ty: TypeId,
        name: Symbol,
        span: Span,
    ) -> Result<Option<Binding>, Diagnostic> {
        if let Some(binding) = self.specialized_record_member(ty, name, span)? {
            return Ok(Some(binding));
        }
        if let Some(binding) = self.ready_namespace_member(ty, name) {
            return Ok(Some(binding));
        }
        if let Some(scope) = self
            .meta
            .local_declarations
            .namespace_scopes
            .get(&ty)
            .copied()
            && let Some(depth) = self
                .local_scopes
                .frames
                .iter()
                .position(|frame| frame.id == scope)
        {
            if let Some(binding) = self.scopes[depth].get(&name).cloned() {
                return Ok(Some(binding));
            }
            if let Some(declaration) = self.local_scopes.frames[depth]
                .declarations
                .get(&name)
                .cloned()
            {
                return self
                    .resolve_local_declaration(depth, &declaration)
                    .map(Some);
            }
        }
        if let Some(value) = self
            .meta
            .local_declarations
            .enum_member_value(ty, name)
            .or_else(|| {
                self.graph_scope
                    .and_then(|scope| scope.enum_member_value(ty, name))
            })
        {
            return Ok(Some(Binding::Enum(modules::aggregates::EnumConstant {
                ty,
                value,
            })));
        }
        Ok(None)
    }

    pub(crate) fn namespace_binding(
        &mut self,
        path: &syntax::NamePath,
        span: Span,
    ) -> Result<Option<Binding>, Diagnostic> {
        if path.members.is_empty() {
            return Ok(None);
        }
        if let Some(binding) = self.resolve_local_name(path.root, span)? {
            let binding = match binding {
                Binding::Imported(binding) => self.imported_binding_value(binding, span)?,
                binding => binding,
            };
            if !matches!(binding, Binding::Type(_) | Binding::Namespace(_)) {
                return Ok(None);
            }
            return self.walk_namespace(binding, &path.members, span).map(Some);
        }
        let Some(scope) = self.graph_scope else {
            return Ok(None);
        };
        for prefix in 0..path.members.len() {
            let type_path = syntax::NamePath {
                root: path.root,
                members: path.members[..prefix].to_vec(),
            };
            if let Ok(ty) = scope.type_name(&type_path, span) {
                let Some(first) = self.type_namespace_member(ty, path.members[prefix], span)?
                else {
                    continue;
                };
                return self
                    .walk_namespace(first, &path.members[prefix + 1..], span)
                    .map(Some);
            }
        }
        Ok(None)
    }

    fn walk_namespace(
        &mut self,
        mut binding: Binding,
        members: &[Symbol],
        span: Span,
    ) -> Result<Binding, Diagnostic> {
        for &name in members {
            binding = match binding {
                Binding::Namespace(module) => {
                    let scope = self.graph_scope.ok_or_else(|| {
                        Diagnostic::new(span, "imported namespaces require a source graph")
                    })?;
                    let binding = scope.namespace_binding(module, &[name], span)?;
                    self.imported_binding_value(binding, span)?
                }
                Binding::Type(ty) => {
                    self.type_namespace_member(ty, name, span)?.ok_or_else(|| {
                        Diagnostic::new(
                            span,
                            format!(
                                "type declaration has no namespace member '{}'",
                                self.symbols.name(name)
                            ),
                        )
                    })?
                }
                _ => {
                    return Err(Diagnostic::new(
                        span,
                        "record namespace path crosses a value declaration",
                    ));
                }
            };
        }
        Ok(binding)
    }
}

impl LocalDeclarationRegistry {
    pub(crate) fn enum_members(&self, ty: TypeId) -> Option<&[(Symbol, Integer)]> {
        self.enumerations
            .get(&ty)
            .map(|info| info.values.as_slice())
    }
}

impl Resolver<'_> {
    pub(crate) fn using_type_members(
        &mut self,
        ty: TypeId,
        span: Span,
    ) -> Result<Vec<(Symbol, Binding)>, Diagnostic> {
        let mut names = Vec::new();
        if let Some(values) = self.meta.local_declarations.enum_members(ty) {
            names.extend(values.iter().map(|(name, _)| *name));
        } else if let Some(info) = self.meta.record_specializations.member_enum(ty) {
            names.extend(info.values.iter().map(|(name, _)| *name));
        } else if let Some(scope) = self.graph_scope {
            names.extend(
                scope
                    .using_enum_members(ty)
                    .into_iter()
                    .map(|(name, _)| name),
            );
        }
        if let Some(bindings) = self.meta.local_declarations.record_namespaces.get(&ty) {
            names.extend(bindings.keys().copied());
        }
        if let Some(scope) = self.meta.local_declarations.namespace_scopes.get(&ty)
            && let Some(frame) = self
                .local_scopes
                .frames
                .iter()
                .find(|frame| frame.id == *scope)
        {
            names.extend(frame.declarations.keys().copied());
        }
        if let Some(members) = self.meta.record_specializations.source_member_names(ty) {
            names.extend(members.iter().copied());
        }
        names.sort_by(|a, b| self.symbols.name(*a).cmp(self.symbols.name(*b)));
        names.dedup();
        let mut bindings = Vec::with_capacity(names.len());
        for name in names {
            if let Some(binding) = self.type_namespace_member(ty, name, span)? {
                bindings.push((name, binding));
            }
        }
        Ok(bindings)
    }
}
