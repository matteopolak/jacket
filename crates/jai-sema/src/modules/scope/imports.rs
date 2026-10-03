//! Retain canonical imported declaration identities in lexical namespaces.
use super::*;
use jai_source::ModuleId;

impl FileScope<'_> {
    /// Source namespace members keep the real nominal owner, including a ready
    /// type alias. Runtime values never supply a substitute namespace owner.
    pub(crate) fn imported_source_member_owner(
        &self,
        binding: GraphBinding,
        span: Span,
    ) -> Result<Option<(TypeId, Symbol)>, Diagnostic> {
        let GraphBinding::SourceMember {
            declaration,
            member,
        } = binding
        else {
            return Ok(None);
        };
        if self.declarations.graph.declaration(declaration).is_none() {
            return Err(Diagnostic::new(
                span,
                "imported namespace member has an unavailable defining declaration",
            ));
        }
        let owner = self
            .declarations
            .nominals
            .declarations
            .get(&declaration)
            .copied()
            .or_else(|| match self.declarations.values.get(&declaration) {
                Some(Binding::Type(ty)) => Some(*ty),
                _ => None,
            });
        Ok(owner.map(|owner| (owner, member)))
    }

    pub(crate) fn imported_record_template(
        &self,
        binding: GraphBinding,
        span: Span,
    ) -> Result<DeclarationId, Diagnostic> {
        let GraphBinding::Declaration(id) = binding else {
            return Err(Diagnostic::new(
                span,
                "imported application base is not a record template",
            ));
        };
        aggregates::parameterized::template_origin_declaration(self.declarations.graph, id, span)
    }
    pub(crate) fn scoped_import_module(
        &self,
        span: Span,
        specialization: Option<&jai_modules::SourceSpecializationKey>,
    ) -> Result<ModuleId, Diagnostic> {
        self.declarations
            .graph
            .scoped_import_for(self.file, span, specialization)
            .ok_or_else(|| {
                Diagnostic::new(
                    span,
                    "scoped import source dependency is pending graph discovery",
                )
            })
    }

    pub(crate) fn module_exports(&self, module: ModuleId) -> Vec<(Symbol, GraphBinding)> {
        let Some(module) = self.declarations.graph.module(module) else {
            return vec![];
        };
        let mut exports: Vec<_> = module
            .exports()
            .iter()
            .map(|(&name, &binding)| (name, binding))
            .collect();
        exports.sort_by(|(left, _), (right, _)| {
            self.declarations
                .graph
                .symbols()
                .name(*left)
                .cmp(self.declarations.graph.symbols().name(*right))
        });
        exports
    }

    pub(crate) fn module_placeholder_exports(
        &self,
        module: ModuleId,
        _span: Span,
    ) -> Vec<(Symbol, jai_modules::PlaceholderId)> {
        self.declarations.graph.module_placeholder_exports(module)
    }

    pub(crate) fn imported_placeholder_binding(
        &self,
        placeholder: jai_modules::PlaceholderId,
        span: Span,
    ) -> Result<GraphBinding, Diagnostic> {
        self.declarations
            .graph
            .lookup_placeholder_export(placeholder)
            .map_err(|error| self.namespace_lookup_error(error, span))
    }

    pub(crate) fn namespace_binding(
        &self,
        module: ModuleId,
        members: &[Symbol],
        span: Span,
    ) -> Result<GraphBinding, Diagnostic> {
        self.declarations
            .graph
            .lookup_module(module, members)
            .map_err(|error| self.namespace_lookup_error(error, span))
    }

    fn namespace_lookup_error(&self, error: LookupError, span: Span) -> Diagnostic {
        if let LookupError::UnfilledPlaceholder(placeholder) = error {
            return super::super::placeholder_demands::unfilled_placeholder(
                self.declarations.graph,
                placeholder,
                span,
            );
        }
        Diagnostic::new(
            span,
            match error {
                LookupError::PrivateMember { name, .. } => format!(
                    "module member '{}' is private",
                    self.declarations.graph.symbols().name(name)
                ),
                LookupError::UnknownMember { name, .. } => format!(
                    "unknown module member '{}'",
                    self.declarations.graph.symbols().name(name)
                ),
                LookupError::NotNamespace(name) => format!(
                    "'{}' is not a namespace",
                    self.declarations.graph.symbols().name(name)
                ),
                _ => "invalid imported module namespace".into(),
            },
        )
    }

    pub(crate) fn imported_type(
        &self,
        binding: GraphBinding,
        span: Span,
    ) -> Result<TypeId, Diagnostic> {
        match binding {
            GraphBinding::Declaration(id) => {
                self.declarations.nominals.declarations.get(&id).copied()
            }
            GraphBinding::Parameter(id) => self
                .declarations
                .nominals
                .module_parameter_types
                .get(&id)
                .copied(),
            _ => None,
        }
        .ok_or_else(|| Diagnostic::new(span, "imported declaration does not denote a type"))
    }

    pub(crate) fn imported_callable(
        &self,
        binding: GraphBinding,
        span: Span,
    ) -> Result<Vec<DeclarationId>, Diagnostic> {
        match binding {
            GraphBinding::OverloadSet(id) => self.checked_callable_members(
                self.declarations
                    .graph
                    .overload_set(id)
                    .expect("retained graph overload set")
                    .declarations(),
                span,
            ),
            GraphBinding::Declaration(id) => {
                if let Some(targets) = self.declarations.callable_aliases.get(&id) {
                    return Ok(targets.clone());
                }
                if self.declarations.signatures.contains_key(&id)
                    || self.declarations.generics.borrow().is_template(id)
                {
                    return Ok(vec![id]);
                }
                Err(Diagnostic::new(
                    span,
                    "imported declaration is not a procedure",
                ))
            }
            _ => Err(Diagnostic::new(span, "imported binding is not a procedure")),
        }
    }

    pub(crate) fn imported_expanded_procedure(
        &self,
        binding: GraphBinding,
        span: Span,
    ) -> Result<Option<(DeclarationId, FileInstanceId, syntax::Procedure)>, Diagnostic> {
        self.expanded_binding_procedure(binding, span)
    }

    pub(crate) fn imported_foreign_library(
        &self,
        binding: GraphBinding,
        span: Span,
    ) -> Result<jai_ir::ForeignLibrary, Diagnostic> {
        let GraphBinding::Declaration(id) = binding else {
            return Err(Diagnostic::new(
                span,
                "imported binding does not denote a foreign library",
            ));
        };
        let declaration = self
            .declarations
            .graph
            .declaration(id)
            .ok_or_else(|| Diagnostic::new(span, "invalid imported library declaration"))?;
        super::super::foreign_libraries::declaration(self.declarations.graph, declaration)
            .map_err(|error| Diagnostic::new(span, error.message))
    }

    pub(crate) fn imported_value(
        &self,
        binding: GraphBinding,
        span: Span,
    ) -> Result<Binding, Diagnostic> {
        match binding {
            GraphBinding::Module(module) => Ok(Binding::Namespace(module)),
            GraphBinding::SourceMember { .. } => {
                if let Some((owner, member)) = self.imported_source_member_owner(binding, span)?
                    && let Some(value) = self
                        .declarations
                        .nominals
                        .enums
                        .get(&owner)
                        .and_then(|info| info.members.get(&member))
                {
                    return Ok(Binding::Enum(aggregates::EnumConstant {
                        ty: owner,
                        value: *value,
                    }));
                }
                Ok(Binding::Imported(binding))
            }
            GraphBinding::Declaration(id) => {
                if let Some(binding) = self.declarations.values.get(&id) {
                    return Ok(binding.clone());
                }
                if let Some(ty) = self.declarations.nominals.declarations.get(&id) {
                    return Ok(Binding::Type(*ty));
                }
                if let Some(signature) = self.declarations.signatures.get(&id) {
                    return Ok(Binding::Procedure {
                        procedure: signature.id,
                        ty: signature.ty,
                    });
                }
                // Templates, aliases and not-yet-evaluated semantic constants
                // remain owned by their defining declaration's worklist.
                Ok(Binding::Imported(binding))
            }
            GraphBinding::OverloadSet(_) | GraphBinding::StorageMember(_) => {
                Ok(Binding::Imported(binding))
            }
            GraphBinding::Parameter(id) => {
                let parameter =
                    self.declarations.graph.parameter(id).ok_or_else(|| {
                        Diagnostic::new(span, "invalid imported module parameter")
                    })?;
                match &parameter.value {
                    jai_modules::ParameterValue::Scalar(value) => {
                        Ok(Binding::Constant(value.clone()))
                    }
                    jai_modules::ParameterValue::Type(_) => {
                        self.imported_type(binding, span).map(Binding::Type)
                    }
                    jai_modules::ParameterValue::Enumeration(value) => {
                        let ty = self
                            .declarations
                            .nominals
                            .declarations
                            .get(&value.declaration)
                            .copied()
                            .ok_or_else(|| {
                                Diagnostic::new(
                                    span,
                                    "imported enum parameter has no canonical nominal declaration",
                                )
                            })?;
                        Ok(Binding::Enum(aggregates::EnumConstant {
                            ty,
                            value: value.value,
                        }))
                    }
                    _ => Ok(Binding::Imported(binding)),
                }
            }
        }
    }
}

impl FileScope<'_> {
    pub(crate) fn using_graph_binding(&self, path: &NamePath) -> Option<GraphBinding> {
        self.declarations.graph.lookup(self.file, path).ok()
    }

    pub(crate) fn using_enum_members(&self, ty: TypeId) -> Vec<(Symbol, IntegerValue)> {
        let Some(info) = self.declarations.nominals.enums.get(&ty) else {
            return Vec::new();
        };
        let mut values: Vec<_> = info
            .members
            .iter()
            .map(|(&name, &value)| (name, value))
            .collect();
        values.sort_by(|(a, _), (b, _)| {
            self.declarations
                .graph
                .symbols()
                .name(*a)
                .cmp(self.declarations.graph.symbols().name(*b))
        });
        values
    }
}

impl FileScope<'_> {
    pub(crate) fn using_source_owner(&self, ty: TypeId) -> Option<DeclarationId> {
        self.declarations
            .nominals
            .declarations
            .iter()
            .filter_map(|(&id, &candidate)| {
                if candidate != ty {
                    return None;
                }
                let declaration = self.declarations.graph.declaration(id)?;
                matches!(
                    declaration.syntax().kind,
                    FileDeclarationKind::Enum(_) | FileDeclarationKind::Record(_)
                )
                .then_some(id)
            })
            .min_by_key(|id| id.index())
    }
}

impl FileScope<'_> {
    pub(crate) fn using_source_publication(
        &self,
        procedure: Option<ProcedureId>,
        span: Span,
        specialization: Option<&jai_modules::SourceSpecializationKey>,
    ) -> Option<&jai_modules::UsingPublication> {
        let publication =
            self.declarations
                .graph
                .using_publication(self.file, span, specialization)?;
        let owner = publication.owner?;
        let actual = specialization.map(|key| key.declaration()).or_else(|| {
            let procedure = procedure?;
            self.declarations
                .signatures
                .iter()
                .find_map(|(&id, signature)| (signature.id == procedure).then_some(id))
                .or_else(|| {
                    self.declarations
                        .generics
                        .borrow()
                        .source_declaration(procedure)
                })
        })?;
        (actual == owner).then_some(publication)
    }
}
