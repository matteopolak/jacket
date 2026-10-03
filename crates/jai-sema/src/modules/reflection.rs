//! Keep declaration spellings in reflection without replacing nominal identities.
use super::*;
use jai_modules::Binding as GraphBinding;
use jai_types::{
    ReflectedEnumMember, ReflectedEnumMetadata, ReflectedFieldMetadata, ReflectedRecordMetadata,
    ReflectionMetadata,
};

impl super::aggregates::types::Nominals<'_> {
    pub(crate) fn install_reflection_header(
        &self,
        header: TypeId,
        types: &mut TypeRegistry,
    ) -> Result<(), jai_types::TypeError> {
        types.record_definition(header)?;
        types.bind_runtime_type_header(header)?;
        if self
            .reflection_header
            .get()
            .is_some_and(|previous| previous != header)
        {
            return Err(jai_types::TypeError::WrongKind(header));
        }
        if let Some(any) = types.any_type()
            && matches!(types.record_storage_definition(any), Err(jai_types::TypeError::Incomplete(owner)) if owner == any)
        {
            types.define_any(any, header)?;
        }
        self.reflection_header.set(Some(header));
        Ok(())
    }
    pub(crate) fn reserve_any_for_graph(
        &self,
        graph: &ModuleGraph,
        types: &mut TypeRegistry,
    ) -> Result<TypeId, jai_types::TypeError> {
        let any = types.reserve_any();
        if let Some(header) = self.reflection_header.get() {
            self.install_reflection_header(header, types)?;
        } else if graph.prelude().is_none() {
            self.generated_reflection_schema(types)?;
        }
        Ok(any)
    }
    pub(crate) fn runtime_type_for_graph(
        &self,
        graph: &ModuleGraph,
        types: &mut TypeRegistry,
    ) -> Result<TypeId, jai_types::TypeError> {
        if let Some(header) = self.reflection_header.get() {
            self.install_reflection_header(header, types)?;
        } else if graph.prelude().is_none() {
            self.generated_reflection_schema(types)?;
        }
        // During source Preload reservation, its nominal header is adopted
        // before signatures or runtime storage are verified. Never substitute
        // the generated catalog for a designated source Preload.
        Ok(types.meta_type())
    }
    pub(crate) fn generated_reflection_schema(
        &self,
        types: &mut TypeRegistry,
    ) -> Result<std::sync::Arc<crate::reflection::schema::TypeInfoSchema>, jai_types::TypeError>
    {
        if let Some(schema) = self.generated_reflection.get() {
            return Ok(std::sync::Arc::clone(schema));
        }
        let schema = std::sync::Arc::new(crate::reflection::schema::TypeInfoSchema::new(types)?);
        self.install_reflection_header(schema.header, types)?;
        // One immutable catalog is shared by annotations and materialization.
        let _ = self
            .generated_reflection
            .set(std::sync::Arc::clone(&schema));
        Ok(schema)
    }

    pub(crate) fn generated_reflection_type(
        &self,
        graph: &ModuleGraph,
        file: FileInstanceId,
        path: &NamePath,
        types: &mut TypeRegistry,
        span: Span,
    ) -> Result<Option<TypeId>, LocatedDiagnostic> {
        if !matches!(
            graph.lookup(
                file,
                &NamePath {
                    root: path.root,
                    members: Vec::new()
                }
            ),
            Err(jai_modules::LookupError::UnknownName(_))
        ) {
            return Ok(None);
        }
        if path.members.is_empty() && graph.symbols().name(path.root) == "Code" {
            return Ok(Some(types.code_type()));
        }
        if graph.prelude().is_some()
            || !crate::reflection::schema::TypeInfoSchema::recognizes_name(path, graph.symbols())
        {
            return Ok(None);
        }
        let schema = self
            .generated_reflection_schema(types)
            .map_err(|error| located(graph, file, Diagnostic::new(span, error.to_string())))?;
        Ok(schema.type_name(path, graph.symbols()))
    }
}

impl FileScope<'_> {
    pub(crate) fn generated_reflection_schema(
        &self,
        types: &mut TypeRegistry,
        span: Span,
    ) -> Result<Option<std::sync::Arc<crate::reflection::schema::TypeInfoSchema>>, Diagnostic> {
        if self.declarations.graph.prelude().is_some() {
            return Ok(None);
        }
        self.declarations
            .nominals
            .generated_reflection_schema(types)
            .map(Some)
            .map_err(|error| Diagnostic::new(span, error.to_string()))
    }
    pub(crate) fn generated_reflection_type(
        &self,
        path: &NamePath,
        types: &mut TypeRegistry,
        span: Span,
    ) -> Result<Option<TypeId>, Diagnostic> {
        self.declarations
            .nominals
            .generated_reflection_type(self.declarations.graph, self.file, path, types, span)
            .map_err(|error| Diagnostic::at_source(error.location, error.message))
    }
    pub(crate) fn expanded_procedure(
        &self,
        path: &syntax::NamePath,
        span: Span,
    ) -> Result<Option<(jai_source::DeclarationId, FileInstanceId, syntax::Procedure)>, Diagnostic>
    {
        let Ok(binding) = self.declarations.graph.lookup(self.file, path) else {
            return Ok(None);
        };
        self.expanded_binding_procedure(binding, span)
    }

    pub(crate) fn expanded_procedures(
        &self,
        path: &syntax::NamePath,
        span: Span,
    ) -> Result<Vec<(jai_source::DeclarationId, FileInstanceId, syntax::Procedure)>, Diagnostic>
    {
        let Ok(binding) = self.declarations.graph.lookup(self.file, path) else {
            return Ok(Vec::new());
        };
        self.expanded_binding_procedures(binding, span)
    }

    /// Collection dispatch matches these genuine source candidates before
    /// selecting a protocol; ordinary macro calls retain their own boundary.
    pub(crate) fn expanded_binding_procedures(
        &self,
        binding: GraphBinding,
        span: Span,
    ) -> Result<Vec<(jai_source::DeclarationId, FileInstanceId, syntax::Procedure)>, Diagnostic>
    {
        let ids = match binding {
            GraphBinding::Declaration(id) => self
                .declarations
                .callable_aliases
                .get(&id)
                .cloned()
                .unwrap_or_else(|| vec![id]),
            GraphBinding::OverloadSet(id) => self
                .declarations
                .graph
                .overload_set(id)
                .ok_or_else(|| Diagnostic::new(span, "invalid source expansion overload set"))?
                .declarations()
                .to_vec(),
            GraphBinding::Module(_)
            | GraphBinding::Parameter(_)
            | GraphBinding::SourceMember {
                ..
            }
            | GraphBinding::StorageMember(_) => return Ok(Vec::new()),
        };
        let mut expanded = Vec::new();
        for id in ids {
            let declaration = self
                .declarations
                .graph
                .declaration(id)
                .ok_or_else(|| Diagnostic::new(span, "invalid source expansion declaration"))?;
            if let FileDeclarationKind::Procedure(procedure) = &declaration.syntax().kind
                && procedure.expands
            {
                expanded.push((id, declaration.file(), procedure.clone()));
            }
        }
        Ok(expanded)
    }

    /// Expanded source bodies deliberately have no runtime procedure signature.
    pub(crate) fn expanded_binding_procedure(
        &self,
        binding: GraphBinding,
        span: Span,
    ) -> Result<Option<(jai_source::DeclarationId, FileInstanceId, syntax::Procedure)>, Diagnostic>
    {
        let declarations = match binding {
            GraphBinding::Declaration(id) => self
                .declarations
                .callable_aliases
                .get(&id)
                .cloned()
                .unwrap_or_else(|| vec![id]),
            GraphBinding::OverloadSet(id) => self
                .declarations
                .graph
                .overload_set(id)
                .expect("retained overload set")
                .declarations()
                .to_vec(),
            GraphBinding::Module(_)
            | GraphBinding::Parameter(_)
            | GraphBinding::SourceMember {
                ..
            }
            | GraphBinding::StorageMember(_) => return Ok(None),
        };
        let mut expanded = Vec::new();
        for id in &declarations {
            let declaration = self
                .declarations
                .graph
                .declaration(*id)
                .expect("callable declaration");
            if let FileDeclarationKind::Procedure(procedure) = &declaration.syntax().kind
                && procedure.expands
            {
                expanded.push((*id, declaration.file(), procedure.clone()));
            }
        }
        if expanded.is_empty() {
            return Ok(None);
        }
        if declarations.len() != 1 {
            return Err(Diagnostic::new(
                span,
                "overloaded #expand procedures require macro argument matching",
            ));
        }
        Ok(expanded.pop())
    }
    pub(crate) fn code_origin(
        &self,
    ) -> (FileInstanceId, jai_source::ScopeId, jai_source::SourceId) {
        let file = self
            .declarations
            .graph
            .file(self.file)
            .expect("defining file");
        (self.file, file.scope(), file.source())
    }
    pub(crate) fn code_file(self, file: FileInstanceId) -> Self {
        Self {
            file,
            substitution: None,
            ..self
        }
    }
    pub(crate) fn reflection_metadata(&self, types: &TypeRegistry) -> ReflectionMetadata {
        let mut metadata = ReflectionMetadata::default();
        let graph = self.declarations.graph;
        for (&ty, record) in &self.declarations.nominals.records {
            let declaration = graph
                .declaration(record.declaration)
                .expect("record declaration");
            let source = graph
                .sources()
                .get(declaration.location().source)
                .expect("record source")
                .text();
            if let FileDeclarationKind::Record(syntax) = &graph
                .declaration(record.declaration)
                .expect("record declaration")
                .syntax()
                .kind
            {
                metadata.name(ty, graph.symbols().name(syntax.name).as_bytes());
                let textual_flags = syntax.attributes.iter().fold(
                    if syntax.kind == jai_types::RecordKind::Union {
                        2
                    } else {
                        0
                    },
                    |flags, attribute| {
                        flags
                            | match attribute {
                                syntax::RecordAttribute::NoPadding => 4,
                                syntax::RecordAttribute::TypeInfoNone => 8,
                                syntax::RecordAttribute::Alignment(_)
                                | syntax::RecordAttribute::Reflection(_) => 0,
                            }
                    },
                );
                metadata.record(
                    ty,
                    ReflectedRecordMetadata {
                        notes: reflected_notes(&syntax.notes, source),
                        unsupported_members: syntax.members.iter().any(|member| {
                            !matches!(
                                member,
                                syntax::RecordMember::Field(_) | syntax::RecordMember::Placement(_)
                            )
                        }),
                        textual_flags,
                        status_flags: 0,
                        nontextual_flags: if syntax.kind == jai_types::RecordKind::Union {
                            64
                        } else {
                            0
                        },
                    },
                );
            }
            for field in &record.fields {
                metadata.field(
                    field.id,
                    ReflectedFieldMetadata {
                        name: Some(graph.symbols().name(field.name).as_bytes().into()),
                        using: field.syntax.using,
                    },
                );
                metadata.field_notes(field.id, reflected_notes(&field.syntax.notes, source));
            }
        }
        for declaration in graph.declarations() {
            let Some(&ty) = self
                .declarations
                .nominals
                .declarations
                .get(&declaration.id())
            else {
                continue;
            };
            match &declaration.syntax().kind {
                FileDeclarationKind::Enum(syntax) => {
                    let enumeration = &self.declarations.nominals.enums[&ty];
                    metadata.name(ty, graph.symbols().name(syntax.name).as_bytes());
                    metadata.enumeration(
                        ty,
                        ReflectedEnumMetadata {
                            members: syntax
                                .members
                                .iter()
                                .zip(enumeration.values.iter())
                                .map(|(member, &value)| ReflectedEnumMember {
                                    name: Some(graph.symbols().name(member.name).as_bytes().into()),
                                    value,
                                })
                                .collect(),
                            flags: enumeration.flags,
                        },
                    );
                }
                FileDeclarationKind::TypeAlias(syntax)
                    if matches!(types.kind(ty), Ok(TypeKind::Distinct(_))) =>
                {
                    metadata.name(ty, graph.symbols().name(syntax.name).as_bytes())
                }
                _ => {}
            }
        }
        metadata
    }
}

fn reflected_notes(notes: &[syntax::NoteSyntax], source: &str) -> Box<[Box<[u8]>]> {
    notes
        .iter()
        .map(|note| {
            note.span
                .text(source)
                .strip_prefix('@')
                .expect("note token")
                .as_bytes()
                .into()
        })
        .collect()
}
